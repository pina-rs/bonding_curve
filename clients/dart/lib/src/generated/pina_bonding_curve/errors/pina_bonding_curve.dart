// Auto-generated. Do not edit.
// ignore_for_file: type=lint, constant_identifier_names

/// Error codes for the PinaBondingCurve program.

/// Curve segments must be non-empty, strictly increasing, in range, and have positive liquidity.
/// Message: "Curve segments must be non-empty, strictly increasing, in range, and have positive liquidity."
const int pinaBondingCurveErrorInvalidCurve = 0x0; // 0

/// Fee rates are out of range or the start fee is below the end fee without a decay.
/// Message: "Fee rates are out of range or the start fee is below the end fee without a decay."
const int pinaBondingCurveErrorInvalidFeeRates = 0x1; // 1

/// The total supply cannot cover the sale, the AMM seed, and the creator allocation.
/// Message: "The total supply cannot cover the sale, the AMM seed, and the creator allocation."
const int pinaBondingCurveErrorInvalidSupply = 0x2; // 2

/// The migration threshold is zero, too small to move the price, or beyond the curve.
/// Message: "The migration threshold is zero, too small to move the price, or beyond the curve."
const int pinaBondingCurveErrorInvalidMigrationThreshold = 0x3; // 3

/// A share is above 100%, or the creator and partner LP shares exceed 100% together.
/// Message: "A share is above 100%, or the creator and partner LP shares exceed 100% together."
const int pinaBondingCurveErrorInvalidShares = 0x4; // 4

/// The vesting cliff and duration do not fit a timestamp.
/// Message: "The vesting cliff and duration do not fit a timestamp."
const int pinaBondingCurveErrorInvalidVesting = 0x5; // 5

/// The AMM fee tier is not a Pina AMM tier restricted to this program.
/// Message: "The AMM fee tier is not a Pina AMM tier restricted to this program."
const int pinaBondingCurveErrorInvalidAmmConfig = 0x6; // 6

/// The mint's token program or Token-2022 extensions are not supported.
/// Message: "The mint's token program or Token-2022 extensions are not supported."
const int pinaBondingCurveErrorUnsupportedMint = 0x7; // 7

/// The base mint must be new: no supply, no freeze authority, the creator as mint authority, and the configured decimals.
/// Message: "The base mint must be new: no supply, no freeze authority, the creator as mint authority, and the configured decimals."
const int pinaBondingCurveErrorInvalidBaseMint = 0x8; // 8

/// The signer is not the authority this instruction requires.
/// Message: "The signer is not the authority this instruction requires."
const int pinaBondingCurveErrorUnauthorized = 0x9; // 9

/// A vault, mint, configuration, or pool account does not belong to this launch.
/// Message: "A vault, mint, configuration, or pool account does not belong to this launch."
const int pinaBondingCurveErrorAccountMismatch = 0xa; // 10

/// The launch is not trading.
/// Message: "The launch is not trading."
const int pinaBondingCurveErrorNotTrading = 0xb; // 11

/// The launch has not reached its activation time.
/// Message: "The launch has not reached its activation time."
const int pinaBondingCurveErrorNotActive = 0xc; // 12

/// The launch has not completed, so it cannot graduate.
/// Message: "The launch has not completed, so it cannot graduate."
const int pinaBondingCurveErrorNotCompleted = 0xd; // 13

/// The result is worse than the caller's slippage limit.
/// Message: "The result is worse than the caller's slippage limit."
const int pinaBondingCurveErrorSlippageExceeded = 0xe; // 14

/// An amount is zero or the trade rounds down to nothing.
/// Message: "An amount is zero or the trade rounds down to nothing."
const int pinaBondingCurveErrorZeroAmount = 0xf; // 15

/// The curve cannot absorb a sale this large.
/// Message: "The curve cannot absorb a sale this large."
const int pinaBondingCurveErrorInsufficientLiquidity = 0x10; // 16

/// An intermediate value overflowed or did not fit its type.
/// Message: "An intermediate value overflowed or did not fit its type."
const int pinaBondingCurveErrorMathOverflow = 0x11; // 17

/// The graduation would seed the AMM with too little liquidity.
/// Message: "The graduation would seed the AMM with too little liquidity."
const int pinaBondingCurveErrorInsufficientMigrationLiquidity = 0x12; // 18

/// The pool creator mode must be 0 (creator) or 1 (partner).
/// Message: "The pool creator mode must be 0 (creator) or 1 (partner)."
const int pinaBondingCurveErrorInvalidPoolCreatorMode = 0x13; // 19

/// There is nothing to claim yet.
/// Message: "There is nothing to claim yet."
const int pinaBondingCurveErrorNothingToClaim = 0x14; // 20

/// A creator or partner LP account is required because its LP share is not zero.
/// Message: "A creator or partner LP account is required because its LP share is not zero."
const int pinaBondingCurveErrorMissingLpAccount = 0x15; // 21

/// Map of error codes to human-readable messages.
const Map<int, String> _pinaBondingCurveErrorMessages = {
  pinaBondingCurveErrorInvalidCurve: 'Curve segments must be non-empty, strictly increasing, in range, and have positive liquidity.',
  pinaBondingCurveErrorInvalidFeeRates: 'Fee rates are out of range or the start fee is below the end fee without a decay.',
  pinaBondingCurveErrorInvalidSupply: 'The total supply cannot cover the sale, the AMM seed, and the creator allocation.',
  pinaBondingCurveErrorInvalidMigrationThreshold: 'The migration threshold is zero, too small to move the price, or beyond the curve.',
  pinaBondingCurveErrorInvalidShares: 'A share is above 100%, or the creator and partner LP shares exceed 100% together.',
  pinaBondingCurveErrorInvalidVesting:
      'The vesting cliff and duration do not fit a timestamp.',
  pinaBondingCurveErrorInvalidAmmConfig:
      'The AMM fee tier is not a Pina AMM tier restricted to this program.',
  pinaBondingCurveErrorUnsupportedMint:
      'The mint\'s token program or Token-2022 extensions are not supported.',
  pinaBondingCurveErrorInvalidBaseMint: 'The base mint must be new: no supply, no freeze authority, the creator as mint authority, and the configured decimals.',
  pinaBondingCurveErrorUnauthorized:
      'The signer is not the authority this instruction requires.',
  pinaBondingCurveErrorAccountMismatch: 'A vault, mint, configuration, or pool account does not belong to this launch.',
  pinaBondingCurveErrorNotTrading: 'The launch is not trading.',
  pinaBondingCurveErrorNotActive:
      'The launch has not reached its activation time.',
  pinaBondingCurveErrorNotCompleted:
      'The launch has not completed, so it cannot graduate.',
  pinaBondingCurveErrorSlippageExceeded:
      'The result is worse than the caller\'s slippage limit.',
  pinaBondingCurveErrorZeroAmount:
      'An amount is zero or the trade rounds down to nothing.',
  pinaBondingCurveErrorInsufficientLiquidity:
      'The curve cannot absorb a sale this large.',
  pinaBondingCurveErrorMathOverflow:
      'An intermediate value overflowed or did not fit its type.',
  pinaBondingCurveErrorInsufficientMigrationLiquidity:
      'The graduation would seed the AMM with too little liquidity.',
  pinaBondingCurveErrorInvalidPoolCreatorMode:
      'The pool creator mode must be 0 (creator) or 1 (partner).',
  pinaBondingCurveErrorNothingToClaim: 'There is nothing to claim yet.',
  pinaBondingCurveErrorMissingLpAccount: 'A creator or partner LP account is required because its LP share is not zero.',
};

/// Get the error message for a PinaBondingCurve program error code.
String? getPinaBondingCurveErrorMessage(int code) {
  return _pinaBondingCurveErrorMessages[code];
}

/// Check if an error code belongs to the PinaBondingCurve program.
bool isPinaBondingCurveError(int code) {
  return _pinaBondingCurveErrorMessages.containsKey(code);
}
