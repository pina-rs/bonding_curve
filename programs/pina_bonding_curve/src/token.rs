//! Token-program helpers shared by the processors.
//!
//! Launches accept SPL Token and Token-2022 mints, but only Token-2022
//! extensions that cannot change how many tokens a transfer moves or who can
//! move them. Extensions are fixed when a mint is initialized, so checking the
//! policy once when a configuration or launch is created keeps every later
//! transfer exact.

use pina::sysvars::Sysvar;
use pina::sysvars::rent::Rent;
use pina::token_2022::state::ExtensionType;
use pina::*;

use crate::errors::CurveError;

/// Size of a token account without extensions. Vaults never need any: every
/// allowed mint extension requires no account extension.
pub const TOKEN_ACCOUNT_LEN: u64 = 165;

/// The two token programs a launch may use.
pub const TOKEN_PROGRAM_IDS: [Address; 2] = [token::ID, token_2022::ID];

/// Token-2022 mint extensions a launch accepts.
///
/// Metadata, groups, and display-only scaling never change transfer amounts or
/// authority. Transfer fees, hooks, permanent delegates, pausing,
/// non-transferability, confidential transfers, and close authorities do, so
/// they are rejected.
pub const ALLOWED_MINT_EXTENSIONS: [ExtensionType; 8] = [
	ExtensionType::MetadataPointer,
	ExtensionType::TokenMetadata,
	ExtensionType::GroupPointer,
	ExtensionType::TokenGroup,
	ExtensionType::GroupMemberPointer,
	ExtensionType::TokenGroupMember,
	ExtensionType::InterestBearingConfig,
	ExtensionType::ScaledUiAmount,
];

/// Require a supported token program.
pub fn assert_token_program(token_program: &AccountView) -> ProgramResult {
	token_program
		.assert_addresses(&TOKEN_PROGRAM_IDS)
		.map_err(|_| CurveError::UnsupportedMint)?;
	Ok(())
}

/// Require an initialized mint owned by `token_program` with only allowed
/// extensions.
pub fn assert_supported_mint(mint: &AccountView, token_program: &AccountView) -> ProgramResult {
	assert_token_program(token_program)?;
	let mint = mint
		.as_token_mint_for_program(token_program.address())
		.and_then(|mint| mint.assert_extensions_allowed(&ALLOWED_MINT_EXTENSIONS))
		.map_err(|_| CurveError::UnsupportedMint)?;
	if !mint.is_initialized() {
		return Err(CurveError::UnsupportedMint.into());
	}
	Ok(())
}

/// Token balance of a vault owned by `token_program`.
pub fn vault_amount(vault: &AccountView, token_program: &AccountView) -> Result<u64, ProgramError> {
	Ok(vault
		.as_token_account_for_program(token_program.address())?
		.amount())
}

/// Move `amount` tokens with transaction-level authority.
pub fn transfer(
	from: &AccountView,
	to: &AccountView,
	authority: &AccountView,
	token_program: &AccountView,
	amount: u64,
) -> ProgramResult {
	token::instructions::Transfer::new(from, to, authority, amount)
		.invoke_with_program(token_program.address())
}

/// Move `amount` tokens out of a launch vault, signed by the launch PDA.
pub fn transfer_signed(
	from: &AccountView,
	to: &AccountView,
	authority: &AccountView,
	token_program: &AccountView,
	amount: u64,
	signer: &Signer<'_, '_>,
) -> ProgramResult {
	token::instructions::Transfer::new(from, to, authority, amount)
		.invoke_signed_with_program(core::slice::from_ref(signer), token_program.address())
}

/// Allocate `space` bytes at a PDA and assign it to `owner`.
///
/// Anyone can send lamports to an address before it is created, and the
/// system program's `CreateAccount` rejects an address that already holds
/// lamports. Topping up, allocating, and assigning separately means a
/// pre-funded vault address can never block a launch.
pub fn create_pda_account(
	payer: &AccountView,
	account: &AccountView,
	space: u64,
	owner: &Address,
	signer: &Signer<'_, '_>,
) -> ProgramResult {
	let required = Rent::get()?
		.try_minimum_balance(usize::try_from(space).map_err(|_| CurveError::MathOverflow)?)?;
	let signers = core::slice::from_ref(signer);
	if account.lamports() == 0 {
		return system::instructions::CreateAccount {
			from: payer,
			to: account,
			lamports: required,
			space,
			owner,
		}
		.invoke_signed(signers);
	}

	let shortfall = required.saturating_sub(account.lamports());
	if shortfall > 0 {
		system::instructions::Transfer {
			from: payer,
			to: account,
			lamports: shortfall,
		}
		.invoke()?;
	}
	system::instructions::Allocate { account, space }.invoke_signed(signers)?;
	system::instructions::Assign { account, owner }.invoke_signed(signers)
}

/// Validate a new launch's base mint and return its token program.
///
/// The mint must be initialized with no supply, the creator as its mint
/// authority, no freeze authority, the configured decimals, and only allowed
/// extensions. That makes the launch the only source of supply once the
/// program revokes the mint authority.
pub fn assert_new_base_mint(
	mint: &AccountView,
	token_program: &AccountView,
	creator: &Address,
	decimals: u8,
) -> ProgramResult {
	assert_supported_mint(mint, token_program)?;
	let state = mint
		.as_token_mint_for_program(token_program.address())
		.map_err(|_| CurveError::UnsupportedMint)?;
	if state.supply() != 0
		|| state.decimals() != decimals
		|| state.freeze_authority().is_some()
		|| state.mint_authority() != Some(creator)
	{
		return Err(CurveError::InvalidBaseMint.into());
	}
	Ok(())
}

/// Create a launch vault: a token account at `[b"launch_vault", launch,
/// mint]` owned by the launch.
pub fn create_vault(
	payer: &AccountView,
	vault: &AccountView,
	mint: &AccountView,
	launch: &Address,
	token_program: &AccountView,
	bump: u8,
) -> ProgramResult {
	let seeds = crate::state::LaunchVault::seeds(launch, mint.address()).with_bump(bump);
	create_pda_account(
		payer,
		vault,
		TOKEN_ACCOUNT_LEN,
		token_program.address(),
		&seeds.to_signer().as_signer(),
	)?;
	token::instructions::InitializeAccount3::new(vault, mint, launch)
		.invoke_with_program(token_program.address())
}
