//! Program error codes.
//!
//! Every variant is returned as `ProgramError::Custom(code)`. Codes are part of
//! the public interface, so a variant's value never changes once released.
//! Each variant's doc comment is one sentence on one line because generated
//! clients use that line as the error message.

use pina::*;

/// Errors returned by the Pina Bonding Curve.
#[error]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CurveError {
	/// Curve segments must be non-empty, strictly increasing, in range, and have positive liquidity.
	InvalidCurve = 0,
	/// Fee rates are out of range or the start fee is below the end fee without a decay.
	InvalidFeeRates = 1,
	/// The total supply cannot cover the sale, the AMM seed, and the creator allocation.
	InvalidSupply = 2,
	/// The migration threshold is zero, too small to move the price, or beyond the curve.
	InvalidMigrationThreshold = 3,
	/// A share is above 100%, or the creator and partner LP shares exceed 100% together.
	InvalidShares = 4,
	/// The vesting cliff and duration do not fit a timestamp.
	InvalidVesting = 5,
	/// The AMM fee tier is not a Pina AMM tier restricted to this program.
	InvalidAmmConfig = 6,
	/// The mint's token program or Token-2022 extensions are not supported.
	UnsupportedMint = 7,
	/// The base mint must be new: no supply, no freeze authority, the creator as mint authority, and the configured decimals.
	InvalidBaseMint = 8,
	/// The signer is not the authority this instruction requires.
	Unauthorized = 9,
	/// A vault, mint, configuration, or pool account does not belong to this launch.
	AccountMismatch = 10,
	/// The launch is not trading.
	NotTrading = 11,
	/// The launch has not reached its activation time.
	NotActive = 12,
	/// The launch has not completed, so it cannot graduate.
	NotCompleted = 13,
	/// The result is worse than the caller's slippage limit.
	SlippageExceeded = 14,
	/// An amount is zero or the trade rounds down to nothing.
	ZeroAmount = 15,
	/// The curve cannot absorb a sale this large.
	InsufficientLiquidity = 16,
	/// An intermediate value overflowed or did not fit its type.
	MathOverflow = 17,
	/// The graduation would seed the AMM with too little liquidity.
	InsufficientMigrationLiquidity = 18,
	/// The pool creator mode must be 0 (creator) or 1 (partner).
	InvalidPoolCreatorMode = 19,
	/// There is nothing to claim yet.
	NothingToClaim = 20,
	/// A creator or partner LP account is required because its LP share is not zero.
	MissingLpAccount = 21,
	/// The new creator may not be the default address, which can never sign or claim.
	DefaultCreator = 22,
	/// The launch price no longer matches the migration price derived from its configuration.
	MigrationPriceMismatch = 23,
}
