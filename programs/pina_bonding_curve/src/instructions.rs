//! Instruction discriminators and data layouts.
//!
//! Every instruction's data is its one-byte discriminator followed by the
//! packed little-endian fields of its data struct. The account list for each
//! instruction lives next to its handler in `crate::processors`.

// Pina's schema macros generate zero-copy companion types and accessors without
// doc comments. Every hand-written item and field in this module is documented;
// the allowance only covers the generated companions.
#![allow(missing_docs)]

use pina::*;

use crate::ID;
use crate::errors::CurveError;
use crate::processors::*;
use crate::state::*;

/// Upper bound on the reserved `Migrate` instruction's rent top-up: roughly
/// 6,960 lamports per byte an account grows when its layout changes.
pub const MAX_MIGRATION_LAMPORTS: u64 = 100_000;

/// Instruction discriminators. Values are part of the wire format.
#[discriminator(
	entrypoint,
	migrations(LaunchConfig, Launch),
	migrations_max_lamports = MAX_MIGRATION_LAMPORTS
)]
pub enum CurveInstruction {
	/// Create a launch configuration. Anyone can create one for themselves.
	CreateConfig = 0,
	/// Launch a token under a configuration.
	CreateLaunch = 1,
	/// Spend quote to buy base along the curve.
	#[dispatch(accounts = TradeAccounts)]
	Buy = 2,
	/// Sell base back to the curve for quote.
	#[dispatch(accounts = TradeAccounts)]
	Sell = 3,
	/// Move a completed launch into a Pina AMM pool.
	Graduate = 4,
	/// Pay a launch's partner fees to the configuration's authority.
	#[dispatch(accounts = ClaimFeesAccounts)]
	ClaimPartnerFees = 5,
	/// Pay a launch's creator fees to its creator.
	#[dispatch(accounts = ClaimFeesAccounts)]
	ClaimCreatorFees = 6,
	/// Pay the creator's vested allocation.
	ClaimCreatorAllocation = 7,
	/// Hand a launch's creator rights to another address.
	SetLaunchCreator = 8,
}

/// Data for `CurveInstruction::CreateConfig`.
#[instruction(discriminator = CurveInstruction::CreateConfig)]
pub struct CreateConfigInstruction {
	/// The partner's configuration index; part of the PDA seeds.
	pub index: u64,
	/// Square-root price (Q64.64) every launch starts at.
	pub sqrt_start_price: u128,
	/// Upper square-root price of each segment; unused slots must be zero.
	pub curve_sqrt_prices: [u128; 16],
	/// Liquidity of each segment; unused slots must be zero.
	pub curve_liquidities: [u128; 16],
	/// Quote a launch raises before it completes.
	pub migration_quote_threshold: u64,
	/// Base minted for each launch.
	pub total_supply: u64,
	/// Base reserved for each launch's creator.
	pub creator_allocation: u64,
	/// Seconds after activation before the creator allocation starts vesting.
	pub creator_vesting_cliff: u64,
	/// Seconds over which the creator allocation vests after the cliff.
	pub creator_vesting_duration: u64,
	/// Seconds over which the trading fee falls from its start to end rate.
	pub fee_decay_duration: u64,
	/// Trading fee at activation, in parts per million.
	pub start_fee_rate: u32,
	/// Trading fee after the decay, in parts per million.
	pub end_fee_rate: u32,
	/// The creator's share of every fee, in parts per million.
	pub creator_fee_share: u32,
	/// Fee on the raised quote at graduation, in parts per million.
	pub migration_fee_rate: u32,
	/// Share of graduation LP sent to the creator, in parts per million.
	pub creator_lp_share: u32,
	/// Share of graduation LP sent to the partner, in parts per million.
	pub partner_lp_share: u32,
	/// Number of curve segments in use, from 1 to 16.
	#[pina(validate(value >= 1 && value <= 16, error = CurveError::InvalidCurve))]
	pub segment_count: u8,
	/// Decimals every launch's base mint must have.
	#[pina(validate(value <= 18, error = CurveError::InvalidBaseMint))]
	pub base_decimals: u8,
	/// `0` pays the AMM pool's creator fees to the launch creator, `1` to the partner.
	#[pina(validate(value <= 1, error = CurveError::InvalidPoolCreatorMode))]
	pub pool_creator_mode: u8,
}

/// Data for `CurveInstruction::CreateLaunch`.
#[instruction(discriminator = CurveInstruction::CreateLaunch)]
pub struct CreateLaunchInstruction {
	/// Unix timestamp when trading opens; zero or a past time opens it now.
	pub activation_time: u64,
}

/// Data for `CurveInstruction::Buy`.
#[instruction(discriminator = CurveInstruction::Buy)]
pub struct BuyInstruction {
	/// Most quote to spend, fees included. A buy that completes the launch
	/// spends only what it needs.
	#[pina(validate(value != 0, error = CurveError::ZeroAmount))]
	pub quote_amount_in: u64,
	/// Least base the buyer will accept.
	pub minimum_base_out: u64,
}

/// Data for `CurveInstruction::Sell`.
#[instruction(discriminator = CurveInstruction::Sell)]
pub struct SellInstruction {
	/// Exact base to sell.
	#[pina(validate(value != 0, error = CurveError::ZeroAmount))]
	pub base_amount_in: u64,
	/// Least quote the seller will accept, after fees.
	pub minimum_quote_out: u64,
}

/// Data for `CurveInstruction::Graduate`.
#[instruction(discriminator = CurveInstruction::Graduate)]
pub struct GraduateInstruction {}

/// Data for `CurveInstruction::ClaimPartnerFees`.
#[instruction(discriminator = CurveInstruction::ClaimPartnerFees)]
pub struct ClaimPartnerFeesInstruction {}

/// Data for `CurveInstruction::ClaimCreatorFees`.
#[instruction(discriminator = CurveInstruction::ClaimCreatorFees)]
pub struct ClaimCreatorFeesInstruction {}

/// Data for `CurveInstruction::ClaimCreatorAllocation`.
#[instruction(discriminator = CurveInstruction::ClaimCreatorAllocation)]
pub struct ClaimCreatorAllocationInstruction {}

/// Data for `CurveInstruction::SetLaunchCreator`.
#[instruction(discriminator = CurveInstruction::SetLaunchCreator)]
pub struct SetLaunchCreatorInstruction {
	/// New creator of the launch.
	pub new_creator: Address,
}
