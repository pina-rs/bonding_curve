//! Address derivation and account decoding.

use pina_bonding_curve_client::PINA_BONDING_CURVE_ID;
use pina_bonding_curve_client::accounts::Launch;
use pina_bonding_curve_client::accounts::LaunchConfig;
use solana_instruction::AccountMeta;
use solana_instruction::Instruction;
use solana_pubkey::Pubkey;

use crate::context::Context;
use crate::error::CliError;

/// The original SPL Token program, which owns every Pina AMM LP mint.
pub const TOKEN_PROGRAM: Pubkey =
	Pubkey::from_str_const("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
/// The Token-2022 program.
pub const TOKEN_2022_PROGRAM: Pubkey =
	Pubkey::from_str_const("TokenzQdBNbLqP5VEhdkAS6EPFLC1PHnBqCXEpPxuEb");
/// The associated token account program.
pub const ATA_PROGRAM: Pubkey =
	Pubkey::from_str_const("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");
/// The system program.
pub const SYSTEM_PROGRAM: Pubkey = Pubkey::from_str_const("11111111111111111111111111111111");
/// Wrapped SOL.
pub const NATIVE_MINT: Pubkey =
	Pubkey::from_str_const("So11111111111111111111111111111111111111112");

/// Byte offset of the `amount` field in an SPL Token or Token-2022 account.
const TOKEN_AMOUNT_OFFSET: usize = 64;
/// Size of a mint without extensions, for SPL Token and Token-2022 alike.
pub const MINT_LEN: u64 = 82;

/// `owner`'s associated token account for `mint`.
pub fn associated_token_address(owner: &Pubkey, mint: &Pubkey, token_program: &Pubkey) -> Pubkey {
	Pubkey::find_program_address(
		&[owner.as_ref(), token_program.as_ref(), mint.as_ref()],
		&ATA_PROGRAM,
	)
	.0
}

/// Create `owner`'s associated token account for `mint` unless it exists.
pub fn create_associated_token_account(
	payer: &Pubkey,
	owner: &Pubkey,
	mint: &Pubkey,
	token_program: &Pubkey,
) -> Instruction {
	Instruction::new_with_bytes(
		ATA_PROGRAM,
		&[1],
		vec![
			AccountMeta::new(*payer, true),
			AccountMeta::new(associated_token_address(owner, mint, token_program), false),
			AccountMeta::new_readonly(*owner, false),
			AccountMeta::new_readonly(*mint, false),
			AccountMeta::new_readonly(SYSTEM_PROGRAM, false),
			AccountMeta::new_readonly(*token_program, false),
		],
	)
}

/// System program `CreateAccount`.
pub fn create_account(
	payer: &Pubkey,
	account: &Pubkey,
	lamports: u64,
	space: u64,
	owner: &Pubkey,
) -> Instruction {
	let mut data = 0u32.to_le_bytes().to_vec();
	data.extend_from_slice(&lamports.to_le_bytes());
	data.extend_from_slice(&space.to_le_bytes());
	data.extend_from_slice(owner.as_ref());
	Instruction::new_with_bytes(
		SYSTEM_PROGRAM,
		&data,
		vec![
			AccountMeta::new(*payer, true),
			AccountMeta::new(*account, true),
		],
	)
}

/// Token `InitializeMint2` with no freeze authority.
pub fn initialize_mint(
	token_program: &Pubkey,
	mint: &Pubkey,
	authority: &Pubkey,
	decimals: u8,
) -> Instruction {
	let mut data = vec![20, decimals];
	data.extend_from_slice(authority.as_ref());
	data.push(0);
	Instruction::new_with_bytes(*token_program, &data, vec![AccountMeta::new(*mint, false)])
}

/// Move `lamports` into a wrapped-SOL token account and sync its balance.
pub fn wrap_sol(owner: &Pubkey, account: &Pubkey, lamports: u64) -> [Instruction; 2] {
	let mut transfer = 2u32.to_le_bytes().to_vec();
	transfer.extend_from_slice(&lamports.to_le_bytes());
	[
		Instruction::new_with_bytes(
			SYSTEM_PROGRAM,
			&transfer,
			vec![
				AccountMeta::new(*owner, true),
				AccountMeta::new(*account, false),
			],
		),
		Instruction::new_with_bytes(
			TOKEN_PROGRAM,
			&[17],
			vec![AccountMeta::new(*account, false)],
		),
	]
}

/// The token program that owns `mint`.
pub fn token_program_of(context: &Context, mint: &Pubkey) -> Result<Pubkey, CliError> {
	Ok(context.account(mint)?.owner)
}

/// Token balance of `account`; zero when the account does not exist.
pub fn token_balance(context: &Context, account: &Pubkey) -> Result<u64, CliError> {
	if !context.exists(account)? {
		return Ok(0);
	}
	let data = context.account(account)?.data;
	data.get(TOKEN_AMOUNT_OFFSET..TOKEN_AMOUNT_OFFSET + 8)
		.and_then(|bytes| bytes.try_into().ok())
		.map(u64::from_le_bytes)
		.ok_or(CliError::InvalidAccountData(*account))
}

/// A decoded configuration.
pub struct ConfigView {
	/// The configuration's address.
	pub address: Pubkey,
	/// The partner who owns the configuration and earns its partner fees.
	pub authority: Pubkey,
	/// The token every launch raises.
	pub quote_mint: Pubkey,
	/// The Pina AMM tier graduated pools use.
	pub amm_config: Pubkey,
	/// Start square-root price (Q64.64).
	pub sqrt_start_price: u128,
	/// Square-root price at which launches complete (Q64.64).
	pub migration_sqrt_price: u128,
	/// Upper square-root price of each segment in use.
	pub segment_prices: Vec<u128>,
	/// Liquidity of each segment in use.
	pub segment_liquidities: Vec<u128>,
	/// Fixed supply of every launch.
	pub total_supply: u64,
	/// Base units the curve sells before completing.
	pub sale_supply: u64,
	/// Base units reserved for the graduated pool.
	pub migration_supply: u64,
	/// Base units reserved for the creator.
	pub creator_allocation: u64,
	/// Quote reserve at which a launch completes.
	pub migration_quote_threshold: u64,
	/// Seconds before the allocation starts vesting.
	pub creator_vesting_cliff: u64,
	/// Seconds over which the allocation vests.
	pub creator_vesting_duration: u64,
	/// Seconds over which the fee decays.
	pub fee_decay_duration: u64,
	/// Configuration index.
	pub index: u64,
	/// Fee at activation, parts per million.
	pub start_fee_rate: u32,
	/// Fee after the decay, parts per million.
	pub end_fee_rate: u32,
	/// Creator's fee share, parts per million.
	pub creator_fee_share: u32,
	/// Migration fee, parts per million.
	pub migration_fee_rate: u32,
	/// Creator's LP share, parts per million.
	pub creator_lp_share: u32,
	/// Partner's LP share, parts per million.
	pub partner_lp_share: u32,
	/// Decimals every launched mint has.
	pub base_decimals: u8,
	/// Who receives the graduated pool's creator fees: 0 creator, 1 partner.
	pub pool_creator_mode: u8,
}

/// Fetch and decode a configuration, checking the program owns it.
pub fn load_config(context: &Context, address: &Pubkey) -> Result<ConfigView, CliError> {
	let account = context.account(address)?;
	require_owner(address, &account.owner)?;
	let config = LaunchConfig::from_bytes(&account.data)
		.map_err(|_| CliError::InvalidAccountData(*address))?;
	let count = usize::from(config.segment_count);
	Ok(ConfigView {
		address: *address,
		authority: config.authority,
		quote_mint: config.quote_mint,
		amm_config: config.amm_config,
		sqrt_start_price: config.sqrt_start_price.get(),
		migration_sqrt_price: config.migration_sqrt_price.get(),
		segment_prices: config.curve_sqrt_prices[..count]
			.iter()
			.copied()
			.map(u128::from)
			.collect(),
		segment_liquidities: config.curve_liquidities[..count]
			.iter()
			.copied()
			.map(u128::from)
			.collect(),
		total_supply: config.total_supply.get(),
		sale_supply: config.sale_supply.get(),
		migration_supply: config.migration_supply.get(),
		creator_allocation: config.creator_allocation.get(),
		migration_quote_threshold: config.migration_quote_threshold.get(),
		creator_vesting_cliff: config.creator_vesting_cliff.get(),
		creator_vesting_duration: config.creator_vesting_duration.get(),
		fee_decay_duration: config.fee_decay_duration.get(),
		index: config.index.get(),
		start_fee_rate: config.start_fee_rate.get(),
		end_fee_rate: config.end_fee_rate.get(),
		creator_fee_share: config.creator_fee_share.get(),
		migration_fee_rate: config.migration_fee_rate.get(),
		creator_lp_share: config.creator_lp_share.get(),
		partner_lp_share: config.partner_lp_share.get(),
		base_decimals: config.base_decimals,
		pool_creator_mode: config.pool_creator_mode,
	})
}

/// A launch's lifecycle stage.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
	/// The curve is open.
	Trading,
	/// The threshold was reached; waiting for graduation.
	Completed,
	/// Liquidity moved to the Pina AMM.
	Graduated,
}

impl Status {
	/// The lowercase name printed by `launch show`.
	pub fn name(self) -> &'static str {
		match self {
			Self::Trading => "trading",
			Self::Completed => "completed",
			Self::Graduated => "graduated",
		}
	}
}

/// A decoded launch.
pub struct LaunchView {
	/// The launch's address.
	pub address: Pubkey,
	/// Its configuration.
	pub config: Pubkey,
	/// Its creator.
	pub creator: Pubkey,
	/// The launched token.
	pub base_mint: Pubkey,
	/// The raised token.
	pub quote_mint: Pubkey,
	/// Vault of unsold base.
	pub base_vault: Pubkey,
	/// Vault of raised quote and unclaimed fees.
	pub quote_vault: Pubkey,
	/// The graduated pool, or the default address.
	pub pool: Pubkey,
	/// Current square-root price (Q64.64).
	pub sqrt_price: u128,
	/// Quote backing the curve.
	pub quote_reserve: u64,
	/// Unclaimed partner fees.
	pub partner_fees: u64,
	/// Unclaimed creator fees.
	pub creator_fees: u64,
	/// Allocation already claimed.
	pub creator_claimed: u64,
	/// When trading opened.
	pub activation_time: i64,
	/// Lifecycle stage.
	pub status: Status,
}

/// The launch PDA of `base_mint`.
pub fn launch_address(base_mint: &Pubkey) -> Pubkey {
	Launch::find_pda(base_mint).0
}

/// Fetch and decode the launch of `base_mint`, checking the program owns it.
pub fn load_launch(context: &Context, base_mint: &Pubkey) -> Result<LaunchView, CliError> {
	let address = launch_address(base_mint);
	let account = context.account(&address)?;
	require_owner(&address, &account.owner)?;
	let launch =
		Launch::from_bytes(&account.data).map_err(|_| CliError::InvalidAccountData(address))?;
	let status = match launch.status {
		0 => Status::Trading,
		1 => Status::Completed,
		2 => Status::Graduated,
		_ => return Err(CliError::InvalidAccountData(address)),
	};
	Ok(LaunchView {
		address,
		config: launch.config,
		creator: launch.creator,
		base_mint: launch.base_mint,
		quote_mint: launch.quote_mint,
		base_vault: launch.base_vault,
		quote_vault: launch.quote_vault,
		pool: launch.pool,
		sqrt_price: launch.sqrt_price.get(),
		quote_reserve: launch.quote_reserve.get(),
		partner_fees: launch.partner_fees.get(),
		creator_fees: launch.creator_fees.get(),
		creator_claimed: launch.creator_claimed.get(),
		activation_time: launch.activation_time.get(),
		status,
	})
}

fn require_owner(address: &Pubkey, owner: &Pubkey) -> Result<(), CliError> {
	if owner == &PINA_BONDING_CURVE_ID {
		return Ok(());
	}
	Err(CliError::WrongOwner {
		address: *address,
		owner: *owner,
		expected: PINA_BONDING_CURVE_ID,
	})
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn mint_initialization_has_no_freeze_authority() {
		let authority = Pubkey::new_unique();
		let instruction = initialize_mint(&TOKEN_PROGRAM, &Pubkey::new_unique(), &authority, 6);
		assert_eq!(instruction.data[..2], [20, 6]);
		assert_eq!(&instruction.data[2..34], authority.as_ref());
		assert_eq!(instruction.data[34], 0);
		assert_eq!(instruction.data.len(), 35);
	}
}
