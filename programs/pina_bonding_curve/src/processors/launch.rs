//! `CreateLaunch` and `SetLaunchCreator`.

use pina::*;

use super::common::ConfigSnapshot;
use super::common::now;
use crate::ID;
use crate::errors::CurveError;
use crate::events::LaunchCreated;
use crate::events::LaunchCreatorChanged;
use crate::instructions::CreateLaunchInstruction;
use crate::instructions::SetLaunchCreatorInstruction;
use crate::state::Launch;
use crate::state::LaunchStatus;
use crate::state::LaunchVault;
use crate::token::assert_new_base_mint;
use crate::token::assert_token_program;
use crate::token::create_vault;

/// Accounts for `CreateLaunch`.
#[derive(Accounts, Debug)]
pub struct CreateLaunchAccounts<'a> {
	/// Pays rent for the launch and its vaults.
	#[pina(validate(signer, writable))]
	pub payer: &'a AccountView,
	/// The creator, who is the base mint's mint authority until this
	/// instruction revokes it.
	#[pina(validate(signer))]
	pub creator: &'a AccountView,
	/// The configuration the launch follows.
	pub config: &'a AccountView,
	/// The base mint: new, empty, controlled by the creator, and sorting after
	/// the quote mint by address bytes.
	pub base_mint: &'a mut AccountView,
	/// The configuration's quote mint.
	pub quote_mint: &'a AccountView,
	/// The launch PDA to create: `[b"launch", base_mint]`.
	pub launch: &'a mut AccountView,
	/// The base vault PDA to create: `[b"launch_vault", launch, base_mint]`.
	pub base_vault: &'a mut AccountView,
	/// The quote vault PDA to create: `[b"launch_vault", launch, quote_mint]`.
	pub quote_vault: &'a mut AccountView,
	/// Token program that owns `base_mint`.
	pub base_token_program: &'a AccountView,
	/// Token program that owns `quote_mint`.
	pub quote_token_program: &'a AccountView,
	/// The system program.
	#[pina(validate(program = system::ID))]
	pub system_program: &'a AccountView,
}

/// Accounts for `SetLaunchCreator`.
#[derive(Accounts, Debug)]
pub struct SetLaunchCreatorAccounts<'a> {
	/// The launch's current creator.
	#[pina(validate(signer))]
	pub creator: &'a AccountView,
	/// The launch.
	pub launch: &'a mut AccountView,
	/// The new creator, who must accept the handover by signing. The appended
	/// wire slot is optional; processing rejects an omitted acceptance.
	#[pina(validate(signer))]
	pub new_creator_signer: Option<&'a AccountView>,
}

impl<'a> ProcessAccountInfos<'a> for CreateLaunchAccounts<'a> {
	fn process(self, data: &[u8]) -> ProgramResult {
		let args = CreateLaunchInstruction::try_from_bytes(data)?;
		let config = ConfigSnapshot::load(self.config)?;
		if self.quote_mint.address() != &config.quote_mint {
			return Err(CurveError::AccountMismatch.into());
		}
		assert_token_program(self.quote_token_program)?;
		self.quote_mint
			.assert_owner(self.quote_token_program.address())
			.map_err(|_| CurveError::UnsupportedMint)?;
		let creator = *self.creator.address();

		// A mint key can sign, but Graduate also needs the base mint as a
		// mutable account. An LP recipient cannot alias that mandatory slot.
		if (config.creator_lp_share > 0 && self.base_mint.address() == &creator)
			|| (config.partner_lp_share > 0 && self.base_mint.address() == &config.authority)
		{
			return Err(CurveError::InvalidLpRecipient.into());
		}
		assert_new_base_mint(
			self.base_mint,
			self.base_token_program,
			&creator,
			config.base_decimals,
		)?;

		// The AMM has one canonical pool per unordered pair and tier. Assign
		// the larger mint to the base role before accepting any launch funds,
		// so a reciprocal launch can never reserve the same graduation pool.
		if self.base_mint.address() <= self.quote_mint.address() {
			return Err(CurveError::InvalidMintOrder.into());
		}

		let current = now()?;
		let requested =
			i64::try_from(args.activation_time.get()).map_err(|_| CurveError::MathOverflow)?;
		let activation_time = requested.max(current);

		let launch_address = *self.launch.address();
		let base_mint = *self.base_mint.address();
		let quote_mint = config.quote_mint;
		let (base_vault, base_vault_bump) =
			LaunchVault::try_find_pda(&launch_address, &base_mint, &ID)
				.ok_or(CurveError::AccountMismatch)?;
		let (quote_vault, quote_vault_bump) =
			LaunchVault::try_find_pda(&launch_address, &quote_mint, &ID)
				.ok_or(CurveError::AccountMismatch)?;
		if self.base_vault.address() != &base_vault || self.quote_vault.address() != &quote_vault {
			return Err(CurveError::AccountMismatch.into());
		}

		let config_address = *self.config.address();
		CreateProgramAccount {
			account: self.launch,
			payer: self.payer,
			owner: &ID,
			seeds: &Launch::seeds(&base_mint).as_slices(),
		}
		.invoke_with_bump::<Launch>(|launch, bump| {
			launch.config = config_address;
			launch.creator = creator;
			launch.base_mint = base_mint;
			launch.quote_mint = quote_mint;
			launch.base_vault = base_vault;
			launch.quote_vault = quote_vault;
			launch.sqrt_price.set(config.curve.start());
			launch.activation_time.set(activation_time);
			launch.status = LaunchStatus::Trading as u8;
			launch.bump = bump;
			Ok(())
		})?;

		create_vault(
			self.payer,
			self.base_vault,
			self.base_mint,
			&launch_address,
			self.base_token_program,
			base_vault_bump,
		)?;
		create_vault(
			self.payer,
			self.quote_vault,
			self.quote_mint,
			&launch_address,
			self.quote_token_program,
			quote_vault_bump,
		)?;

		// Mint the whole fixed supply into the vault, then remove the mint
		// authority so no more can ever exist.
		token::instructions::MintTo::new(
			self.base_mint,
			self.base_vault,
			self.creator,
			config.total_supply,
		)
		.invoke_with_program(self.base_token_program.address())?;
		token::instructions::SetAuthority::new(
			self.base_mint,
			self.creator,
			token::instructions::AuthorityType::MintTokens,
			None,
		)
		.invoke_with_program(self.base_token_program.address())?;

		LaunchCreated::emit(|event| {
			event.launch = launch_address;
			event.config = config_address;
			event.creator = creator;
			event.base_mint = base_mint;
			event.total_supply.set(config.total_supply);
			event.activation_time.set(activation_time);
			Ok(())
		})
	}
}

impl<'a> ProcessAccountInfos<'a> for SetLaunchCreatorAccounts<'a> {
	fn process(self, data: &[u8]) -> ProgramResult {
		let args = SetLaunchCreatorInstruction::try_from_bytes(data)?;
		// The default address can never sign, so creator fees and the vested
		// allocation would become unclaimable, and a launch whose LP share is
		// not zero could never graduate.
		if args.new_creator == Address::default() {
			return Err(CurveError::DefaultCreator.into());
		}
		let launch_address = *self.launch.address();
		// The launch PDA can never sign a transaction, and its LP payout would
		// target the same token account the AMM mints graduation LP into,
		// which the runtime rejects as a duplicate writable account. Accepting
		// it would let a creator make a launch whose LP share is not zero
		// impossible to graduate, locking every trader's quote forever.
		if args.new_creator == launch_address {
			return Err(CurveError::LaunchAsCreator.into());
		}

		let accepting_creator = self
			.new_creator_signer
			.ok_or(ProgramError::MissingRequiredSignature)?;

		if &args.new_creator != accepting_creator.address() {
			return Err(CurveError::AccountMismatch.into());
		}
		let previous_creator;
		{
			let mut launch = self.launch.as_account_mut::<Launch>(&ID)?;
			if &launch.creator != self.creator.address() {
				return Err(CurveError::Unauthorized.into());
			}

			// Other mandatory writable graduation accounts are PDAs that cannot
			// accept this handover. The base mint is the exception: its key can
			// still sign after the token program takes ownership.
			if args.new_creator == launch.base_mint {
				return Err(CurveError::InvalidLpRecipient.into());
			}
			previous_creator = launch.creator;
			launch.creator = args.new_creator;
		}
		LaunchCreatorChanged::emit(|event| {
			event.launch = launch_address;
			event.previous_creator = previous_creator;
			event.new_creator = args.new_creator;
			Ok(())
		})
	}
}
