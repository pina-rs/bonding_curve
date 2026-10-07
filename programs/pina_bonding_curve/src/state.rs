//! Account layouts, PDA seeds, and limits.
//!
//! Pina accounts are packed, fixed width, and decoded at their exact size, so
//! the field order of every account in this module is part of the on-chain
//! ABI. The first byte of each account is its `CurveAccount` discriminator and
//! the second is the schema version recorded by `pina migrations`.

// Pina's schema macros generate zero-copy companion types and accessors without
// doc comments. Every hand-written item and field in this module is documented;
// the allowance only covers the generated companions.
#![allow(missing_docs)]

use pina::*;

/// Seed prefix for [`LaunchConfig`] PDAs: `[b"config", authority, index]`.
pub const CONFIG_SEED: &[u8] = b"config";

/// Seed prefix for [`Launch`] PDAs: `[b"launch", base_mint]`.
pub const LAUNCH_SEED: &[u8] = b"launch";

/// Seed prefix for launch vault PDAs: `[b"launch_vault", launch, mint]`.
pub const LAUNCH_VAULT_SEED: &[u8] = b"launch_vault";

/// Seed of the program's single Pina AMM pool-creator authority PDA.
pub const AMM_AUTHORITY_SEED: &[u8] = b"amm_authority";

/// Denominator for every rate: `1_000_000` parts per million.
pub const FEE_RATE_DENOMINATOR: u32 = 1_000_000;

/// The highest trading fee a launch may open with (99%), for anti-sniping.
pub const MAX_START_FEE_RATE: u32 = 990_000;

/// The highest trading fee a launch may settle at (10%).
pub const MAX_END_FEE_RATE: u32 = 100_000;

/// The highest share of the raised quote a migration fee may take (10%).
pub const MAX_MIGRATION_FEE_RATE: u32 = 100_000;

/// The Pina AMM's permanently locked liquidity. A graduation must mint more.
pub const AMM_MINIMUM_LIQUIDITY: u64 = 1_000;

/// The largest base-mint decimals a configuration may require.
pub const MAX_BASE_DECIMALS: u8 = 18;

/// Account-type discriminators for every account this program owns.
#[discriminator]
pub enum CurveAccount {
	/// A partner's launch configuration. See `LaunchConfig`.
	LaunchConfig = 1,
	/// One token's launch. See `Launch`.
	Launch = 2,
}

/// Where a launch is in its life.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LaunchStatus {
	/// Buys and sells move along the curve.
	Trading = 0,
	/// The migration price was reached; only graduation is possible.
	Completed = 1,
	/// Liquidity has moved into the Pina AMM.
	Graduated = 2,
}

impl LaunchStatus {
	/// Decode a stored status byte.
	#[must_use]
	pub const fn from_u8(value: u8) -> Option<Self> {
		match value {
			0 => Some(Self::Trading),
			1 => Some(Self::Completed),
			2 => Some(Self::Graduated),
			_ => None,
		}
	}
}

/// Who receives the Pina AMM pool's creator fees after graduation.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PoolCreatorMode {
	/// The launch's creator.
	Creator = 0,
	/// The configuration's partner.
	Partner = 1,
}

impl PoolCreatorMode {
	/// Decode a stored mode byte.
	#[must_use]
	pub const fn from_u8(value: u8) -> Option<Self> {
		match value {
			0 => Some(Self::Creator),
			1 => Some(Self::Partner),
			_ => None,
		}
	}
}

/// A partner's launch configuration. Immutable once created.
///
/// Every launch under a configuration shares its curve, fees, allocations, and
/// graduation terms, so traders can judge a launchpad by its configuration
/// instead of auditing each token.
#[account(discriminator = CurveAccount)]
#[pda(seeds = [CONFIG_SEED, authority: Address, index: u64], bump = bump)]
pub struct LaunchConfig {
	/// The partner: receives the partner's share of fees and graduation LP.
	pub authority: Address,
	/// The token every launch raises.
	pub quote_mint: Address,
	/// The Pina AMM fee tier launches graduate into.
	pub amm_config: Address,
	/// Square-root price (Q64.64) every launch starts at.
	pub sqrt_start_price: u128,
	/// Square-root price (Q64.64) at which a launch completes. Derived.
	pub migration_sqrt_price: u128,
	/// Upper square-root price of each curve segment; unused slots are zero.
	pub curve_sqrt_prices: [u128; 16],
	/// Liquidity of each curve segment; unused slots are zero.
	pub curve_liquidities: [u128; 16],
	/// Base minted for each launch.
	pub total_supply: u64,
	/// Base sold along the curve up to the migration price. Derived.
	pub sale_supply: u64,
	/// Base reserved to seed the AMM pool. Derived.
	pub migration_supply: u64,
	/// Base reserved for each launch's creator, released by vesting.
	pub creator_allocation: u64,
	/// Quote a launch raises before it completes.
	pub migration_quote_threshold: u64,
	/// Seconds after activation before the creator allocation starts vesting.
	pub creator_vesting_cliff: u64,
	/// Seconds over which the creator allocation vests after the cliff.
	pub creator_vesting_duration: u64,
	/// Seconds over which the trading fee falls from its start to end rate.
	pub fee_decay_duration: u64,
	/// The partner's configuration index; part of the PDA seeds.
	pub index: u64,
	/// Trading fee when a launch activates, in parts per million.
	pub start_fee_rate: u32,
	/// Trading fee after the decay, in parts per million.
	pub end_fee_rate: u32,
	/// The creator's share of every trading and migration fee, in parts per
	/// million. The partner receives the rest.
	pub creator_fee_share: u32,
	/// Fee on the raised quote at graduation, in parts per million.
	pub migration_fee_rate: u32,
	/// Share of graduation LP sent to the creator, in parts per million.
	pub creator_lp_share: u32,
	/// Share of graduation LP sent to the partner, in parts per million. LP
	/// not sent to the creator or partner is burned, locking it forever.
	pub partner_lp_share: u32,
	/// Decimals every launch's base mint must have.
	pub base_decimals: u8,
	/// Number of curve segments in use.
	pub segment_count: u8,
	/// `PoolCreatorMode` wire value.
	pub pool_creator_mode: u8,
	/// Canonical bump of this configuration's PDA.
	pub bump: u8,
}

/// One token's launch.
#[account(discriminator = CurveAccount)]
#[pda(seeds = [LAUNCH_SEED, base_mint: Address], bump = bump)]
pub struct Launch {
	/// The configuration this launch follows.
	pub config: Address,
	/// The creator: receives the creator's share of fees and the vested
	/// allocation.
	pub creator: Address,
	/// The launched token.
	pub base_mint: Address,
	/// The raised token, copied from the configuration.
	pub quote_mint: Address,
	/// Launch-owned token account holding unsold base.
	pub base_vault: Address,
	/// Launch-owned token account holding raised quote and unpaid fees.
	pub quote_vault: Address,
	/// The Pina AMM pool, once graduated; otherwise the default address.
	pub pool: Address,
	/// Current square-root price (Q64.64).
	pub sqrt_price: u128,
	/// Quote backing the curve, excluding fees.
	pub quote_reserve: u64,
	/// Partner fees in the quote vault, not yet claimed.
	pub partner_fees: u64,
	/// Creator fees in the quote vault, not yet claimed.
	pub creator_fees: u64,
	/// Creator allocation already claimed.
	pub creator_claimed: u64,
	/// Unix timestamp when trading opens and vesting is measured from.
	pub activation_time: i64,
	/// `LaunchStatus` wire value.
	pub status: u8,
	/// Canonical bump of this launch's PDA.
	pub bump: u8,
}

/// Data-free PDA seeds for a launch vault.
#[pda(seeds = [LAUNCH_VAULT_SEED, launch: Address, mint: Address])]
pub struct LaunchVault {}

/// Data-free PDA that signs as the Pina AMM pool-creator authority.
#[pda(seeds = [AMM_AUTHORITY_SEED])]
pub struct AmmAuthority {}
