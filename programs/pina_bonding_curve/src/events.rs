//! Events written to the transaction log.
//!
//! Each event is emitted with Pina's `emit` helper as a `Program data:` log
//! line. Generated clients decode them with `parsePinaBondingCurveEventsFromLogs`
//! (TypeScript and Dart). Always pass the complete, ordered logs of one
//! transaction so events are attributed to the program that emitted them.

// Pina's schema macros generate zero-copy companion types and accessors without
// doc comments. Every hand-written item and field in this module is documented;
// the allowance only covers the generated companions.
#![allow(missing_docs)]

use pina::*;

/// Event discriminators. Values are part of the wire format.
#[discriminator]
pub enum CurveEvent {
	/// A configuration was created. See `ConfigCreated`.
	ConfigCreated = 1,
	/// A launch was created. See `LaunchCreated`.
	LaunchCreated = 2,
	/// A buy or sell settled. See `Traded`.
	Traded = 3,
	/// A launch reached its migration price. See `Completed`.
	Completed = 4,
	/// A launch moved into the Pina AMM. See `Graduated`.
	Graduated = 5,
	/// Fees or a vested allocation were paid out. See `Claimed`.
	Claimed = 6,
}

/// Emitted by `CreateConfig`.
#[event(discriminator = CurveEvent)]
pub struct ConfigCreated {
	/// The new configuration.
	pub config: Address,
	/// The partner who owns it.
	pub authority: Address,
	/// The token every launch raises.
	pub quote_mint: Address,
	/// The Pina AMM fee tier launches graduate into.
	pub amm_config: Address,
	/// Square-root price (Q64.64) at which launches complete.
	pub migration_sqrt_price: u128,
	/// Base sold along the curve.
	pub sale_supply: u64,
	/// Base reserved to seed the AMM pool.
	pub migration_supply: u64,
}

/// Emitted by `CreateLaunch`.
#[event(discriminator = CurveEvent)]
pub struct LaunchCreated {
	/// The new launch.
	pub launch: Address,
	/// Its configuration.
	pub config: Address,
	/// Its creator.
	pub creator: Address,
	/// The launched token.
	pub base_mint: Address,
	/// Base minted into the launch vault.
	pub total_supply: u64,
	/// Unix timestamp when trading opens.
	pub activation_time: i64,
}

/// Emitted by `Buy` and `Sell`.
#[event(discriminator = CurveEvent)]
pub struct Traded {
	/// The launch traded against.
	pub launch: Address,
	/// The trader who signed.
	pub trader: Address,
	/// `1` for a buy and `0` for a sale.
	pub is_buy: u8,
	/// Base bought or sold.
	pub base_amount: u64,
	/// Quote the buyer paid, fee included, or the seller received, fee deducted.
	pub quote_amount: u64,
	/// Trading fee, in quote.
	pub fee: u64,
	/// The creator's part of `fee`.
	pub creator_fee: u64,
	/// Square-root price (Q64.64) after the trade.
	pub sqrt_price: u128,
	/// Quote backing the curve after the trade.
	pub quote_reserve: u64,
}

/// Emitted by the `Buy` that reaches the migration price.
#[event(discriminator = CurveEvent)]
pub struct Completed {
	/// The completed launch.
	pub launch: Address,
	/// Quote backing the curve at completion.
	pub quote_reserve: u64,
}

/// Emitted by `Graduate`.
#[event(discriminator = CurveEvent)]
pub struct Graduated {
	/// The graduated launch.
	pub launch: Address,
	/// The Pina AMM pool it seeded.
	pub pool: Address,
	/// Base deposited into the pool.
	pub pool_base: u64,
	/// Quote deposited into the pool.
	pub pool_quote: u64,
	/// Migration fee taken from the raised quote.
	pub migration_fee: u64,
	/// Base burned because the pool did not need it.
	pub burned_base: u64,
	/// LP burned, locking that liquidity forever.
	pub burned_lp: u64,
	/// LP sent to the creator.
	pub creator_lp: u64,
	/// LP sent to the partner.
	pub partner_lp: u64,
}

/// Emitted by `ClaimPartnerFees`, `ClaimCreatorFees`, and
/// `ClaimCreatorAllocation`.
#[event(discriminator = CurveEvent)]
pub struct Claimed {
	/// The launch paid from.
	pub launch: Address,
	/// The signer who claimed.
	pub claimant: Address,
	/// `0` partner fees, `1` creator fees, `2` vested creator allocation.
	pub kind: u8,
	/// Amount paid: quote for fees, base for the allocation.
	pub amount: u64,
}
