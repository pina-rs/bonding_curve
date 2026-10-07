//! Checked integer arithmetic: 256-bit intermediates, fees, and vesting.
//!
//! The curve stores square-root prices as Q64.64 fixed-point `u128` values, so
//! products of a price and a liquidity can reach 256 bits. This module keeps
//! the one 256-bit operation the program needs — `a * b / d` for `u128`
//! operands — small, allocation-free, and exact. When the product fits in
//! `u128`, which is the common case for realistic curves, the native division
//! is used instead of the 256-bit loop.

use pina::ProgramError;

use crate::errors::CurveError;
use crate::state::FEE_RATE_DENOMINATOR;

/// `2^64`: one in Q64.64 fixed point.
pub const Q64: u128 = 1 << 64;

const LOW_MASK: u128 = u64::MAX as u128;
const DENOMINATOR: u128 = FEE_RATE_DENOMINATOR as u128;

/// Which way an inexact division rounds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rounding {
	/// Toward zero.
	Down,
	/// Away from zero.
	Up,
}

/// The full 256-bit product of two `u128` values as `(high, low)` halves.
#[must_use]
pub const fn full_mul(a: u128, b: u128) -> (u128, u128) {
	let (a_high, a_low) = (a >> 64, a & LOW_MASK);
	let (b_high, b_low) = (b >> 64, b & LOW_MASK);
	let low_low = a_low * b_low;
	let low_high = a_low * b_high;
	let high_low = a_high * b_low;
	let high_high = a_high * b_high;
	// Each term is below 2^64, so the sum is below 3 * 2^64.
	let middle = (low_low >> 64) + (low_high & LOW_MASK) + (high_low & LOW_MASK);
	let low = (low_low & LOW_MASK) | (middle << 64);
	let high = high_high + (low_high >> 64) + (high_low >> 64) + (middle >> 64);
	(high, low)
}

/// `(high * 2^128 + low) / divisor`, rounded as requested.
///
/// Returns `None` when the divisor is zero or the quotient does not fit in
/// `u128`.
#[must_use]
pub fn div_wide(high: u128, low: u128, divisor: u128, rounding: Rounding) -> Option<u128> {
	if divisor == 0 || high >= divisor {
		return None;
	}
	let (quotient, remainder) = if high == 0 {
		(low / divisor, low % divisor)
	} else {
		// Restoring long division over the low 128 bits. The running
		// remainder stays below the divisor; `carry` records the bit shifted
		// out of it, in which case the true value already exceeds the divisor.
		let mut remainder = high;
		let mut quotient = 0u128;
		let mut bit = 128;
		while bit > 0 {
			bit -= 1;
			let carry = remainder >> 127;
			remainder = (remainder << 1) | ((low >> bit) & 1);
			quotient <<= 1;
			if carry == 1 || remainder >= divisor {
				remainder = remainder.wrapping_sub(divisor);
				quotient |= 1;
			}
		}
		(quotient, remainder)
	};
	match rounding {
		Rounding::Up if remainder != 0 => quotient.checked_add(1),
		_ => Some(quotient),
	}
}

/// `a * b / divisor` with a 256-bit intermediate.
pub fn mul_div(a: u128, b: u128, divisor: u128, rounding: Rounding) -> Result<u128, ProgramError> {
	let (high, low) = full_mul(a, b);
	div_wide(high, low, divisor, rounding).ok_or_else(|| CurveError::MathOverflow.into())
}

/// `a * b / 2^64` with a 256-bit intermediate.
pub fn mul_shr_64(a: u128, b: u128, rounding: Rounding) -> Result<u128, ProgramError> {
	let (high, low) = full_mul(a, b);
	if high >> 64 != 0 {
		return Err(CurveError::MathOverflow.into());
	}
	let shifted = (high << 64) | (low >> 64);
	if rounding == Rounding::Up && low & LOW_MASK != 0 {
		return shifted
			.checked_add(1)
			.ok_or_else(|| CurveError::MathOverflow.into());
	}
	Ok(shifted)
}

/// Convert a `u128` into `u64`, failing instead of truncating.
pub fn narrow(value: u128) -> Result<u64, ProgramError> {
	u64::try_from(value).map_err(|_| CurveError::MathOverflow.into())
}

/// `ceil(amount * rate / 1_000_000)`: a fee rounded in the curve's favour.
pub fn fee_ceil(amount: u64, rate: u32) -> Result<u64, ProgramError> {
	narrow((u128::from(amount) * u128::from(rate)).div_ceil(DENOMINATOR))
}

/// `floor(amount * rate / 1_000_000)`: a share that leaves the remainder with
/// the other party.
pub fn share_floor(amount: u64, rate: u32) -> Result<u64, ProgramError> {
	narrow(u128::from(amount) * u128::from(rate) / DENOMINATOR)
}

/// `ceil(net * 1_000_000 / (1_000_000 - rate))`: the smallest gross amount
/// whose fee at `rate` leaves at least `net`.
pub fn gross_up(net: u64, rate: u32) -> Result<u64, ProgramError> {
	let remaining = DENOMINATOR
		.checked_sub(u128::from(rate))
		.filter(|value| *value > 0)
		.ok_or(CurveError::InvalidFeeRates)?;
	narrow((u128::from(net) * DENOMINATOR).div_ceil(remaining))
}

/// The trading fee rate at `now`.
///
/// The rate starts at `start_rate` when the launch activates and falls in a
/// straight line to `end_rate` over `decay_seconds`, which makes the first
/// seconds of a launch expensive for sniping bots and cheap for everyone after.
#[must_use]
pub fn fee_rate_at(
	start_rate: u32,
	end_rate: u32,
	decay_seconds: u64,
	activation: i64,
	now: i64,
) -> u32 {
	if decay_seconds == 0 || start_rate <= end_rate {
		return end_rate;
	}
	let elapsed = u64::try_from(now.saturating_sub(activation)).unwrap_or(0);
	if elapsed >= decay_seconds {
		return end_rate;
	}
	let span = u128::from(start_rate - end_rate);
	let reduction = span * u128::from(elapsed) / u128::from(decay_seconds);
	// `reduction < span`, so the subtraction cannot underflow and fits u32.
	start_rate - u32::try_from(reduction).unwrap_or(start_rate - end_rate)
}

/// The creator allocation vested at `now`.
///
/// Nothing vests before `activation + cliff`; after that the allocation vests
/// linearly over `duration`. A zero duration vests everything at the cliff.
pub fn vested_amount(
	allocation: u64,
	activation: i64,
	cliff_seconds: u64,
	duration_seconds: u64,
	now: i64,
) -> Result<u64, ProgramError> {
	let cliff = i64::try_from(cliff_seconds).map_err(|_| CurveError::MathOverflow)?;
	let start = activation
		.checked_add(cliff)
		.ok_or(CurveError::MathOverflow)?;
	if now < start {
		return Ok(0);
	}
	if duration_seconds == 0 {
		return Ok(allocation);
	}
	let elapsed = u64::try_from(now - start)
		.map_err(|_| CurveError::MathOverflow)?
		.min(duration_seconds);
	narrow(u128::from(allocation) * u128::from(elapsed) / u128::from(duration_seconds))
}

/// Floor of the integer square root, by the shift-and-subtract method.
#[must_use]
pub const fn sqrt_floor(mut value: u128) -> u128 {
	let mut result = 0u128;
	let mut bit = 1u128 << 126;
	while bit > value {
		bit >>= 2;
	}
	while bit != 0 {
		if value >= result + bit {
			value -= result + bit;
			result = (result >> 1) + bit;
		} else {
			result >>= 1;
		}
		bit >>= 2;
	}
	result
}

#[cfg(test)]
mod tests {
	extern crate std;

	use proptest::prelude::*;

	use super::*;

	/// A slow but obviously correct 256-bit reference: schoolbook division
	/// on `[u64; 4]` little-endian limbs.
	fn reference_div(a: u128, b: u128, divisor: u128) -> Option<(u128, u128)> {
		let (high, low) = full_mul(a, b);
		if divisor == 0 || high >= divisor {
			return None;
		}
		let mut remainder = 0u128;
		let mut quotient = 0u128;
		for bit in (0..256).rev() {
			let value_bit = if bit >= 128 {
				(high >> (bit - 128)) & 1
			} else {
				(low >> bit) & 1
			};
			let overflow = remainder >> 127;
			remainder = (remainder << 1) | value_bit;
			if overflow == 1 || remainder >= divisor {
				remainder = remainder.wrapping_sub(divisor);
				if bit < 128 {
					quotient |= 1 << bit;
				}
			}
		}
		Some((quotient, remainder))
	}

	#[test]
	fn full_mul_matches_known_products() {
		assert_eq!(full_mul(0, u128::MAX), (0, 0));
		assert_eq!(full_mul(u128::MAX, 1), (0, u128::MAX));
		assert_eq!(full_mul(1 << 64, 1 << 64), (1, 0));
		assert_eq!(full_mul(u128::MAX, u128::MAX), (u128::MAX - 1, 1));
	}

	#[test]
	fn div_wide_rounds_and_rejects_overflow() {
		assert_eq!(div_wide(0, 7, 2, Rounding::Down), Some(3));
		assert_eq!(div_wide(0, 7, 2, Rounding::Up), Some(4));
		assert_eq!(div_wide(0, 8, 2, Rounding::Up), Some(4));
		assert_eq!(div_wide(1, 0, 2, Rounding::Down), Some(1 << 127));
		assert_eq!(div_wide(2, 0, 2, Rounding::Down), None);
		assert_eq!(div_wide(0, 1, 0, Rounding::Down), None);
	}

	#[test]
	fn mul_shr_64_rounds() {
		assert_eq!(mul_shr_64(Q64, 5, Rounding::Down).expect("exact"), 5);
		assert_eq!(mul_shr_64(3, Q64 / 2, Rounding::Down).expect("floor"), 1);
		assert_eq!(mul_shr_64(3, Q64 / 2, Rounding::Up).expect("ceil"), 2);
		assert!(mul_shr_64(u128::MAX, u128::MAX, Rounding::Down).is_err());
	}

	#[test]
	fn fee_rate_decays_linearly_to_the_end_rate() {
		assert_eq!(fee_rate_at(500_000, 10_000, 100, 1_000, 999), 500_000);
		assert_eq!(fee_rate_at(500_000, 10_000, 100, 1_000, 1_000), 500_000);
		assert_eq!(fee_rate_at(500_000, 10_000, 100, 1_000, 1_050), 255_000);
		assert_eq!(fee_rate_at(500_000, 10_000, 100, 1_000, 1_100), 10_000);
		assert_eq!(fee_rate_at(500_000, 10_000, 0, 1_000, 1_000), 10_000);
		assert_eq!(fee_rate_at(10_000, 10_000, 100, 1_000, 1_000), 10_000);
	}

	#[test]
	fn vesting_respects_the_cliff_and_duration() {
		assert_eq!(vested_amount(1_000, 100, 50, 100, 149).expect("before"), 0);
		assert_eq!(vested_amount(1_000, 100, 50, 100, 150).expect("cliff"), 0);
		assert_eq!(vested_amount(1_000, 100, 50, 100, 200).expect("half"), 500);
		assert_eq!(
			vested_amount(1_000, 100, 50, 100, 10_000).expect("all"),
			1_000
		);
		assert_eq!(
			vested_amount(1_000, 100, 0, 0, 100).expect("instant"),
			1_000
		);
	}

	#[test]
	fn fees_round_in_the_curves_favour() {
		assert_eq!(fee_ceil(1_000_001, 10_000).expect("fee"), 10_001);
		assert_eq!(share_floor(10_001, 500_000).expect("share"), 5_000);
		assert_eq!(gross_up(990_000, 10_000).expect("gross"), 1_000_000);
		assert!(gross_up(1, FEE_RATE_DENOMINATOR).is_err());
	}

	proptest! {
		#![proptest_config(ProptestConfig::with_cases(4_096))]

		#[test]
		fn mul_div_matches_the_reference(
			a in any::<u128>(),
			b in any::<u128>(),
			divisor in 1u128..,
		) {
			let expected = reference_div(a, b, divisor);
			let down = mul_div(a, b, divisor, Rounding::Down).ok();
			let up = mul_div(a, b, divisor, Rounding::Up).ok();
			match expected {
				None => prop_assert!(down.is_none()),
				Some((quotient, remainder)) => {
					prop_assert_eq!(down, Some(quotient));
					if remainder == 0 {
						prop_assert_eq!(up, Some(quotient));
					} else {
						prop_assert_eq!(up, quotient.checked_add(1));
					}
				}
			}
		}

		#[test]
		fn mul_div_handles_small_operands_natively(
			a in 0u128..(1 << 64),
			b in 0u128..(1 << 64),
			divisor in 1u128..,
		) {
			prop_assert_eq!(mul_div(a, b, divisor, Rounding::Down).ok(), Some(a * b / divisor));
			prop_assert_eq!(mul_div(a, b, divisor, Rounding::Up).ok(), Some((a * b).div_ceil(divisor)));
		}

		#[test]
		fn gross_up_always_covers_its_fee(net in 0u64..=u64::MAX / 1_000, rate in 0u32..990_000) {
			let gross = gross_up(net, rate).expect("gross");
			prop_assert!(gross - fee_ceil(gross, rate).expect("fee") >= net);
		}

		#[test]
		fn vesting_never_exceeds_the_allocation(
			allocation in any::<u64>(),
			cliff in 0u64..1_000_000,
			duration in 0u64..1_000_000,
			now in 0i64..10_000_000,
		) {
			let vested = vested_amount(allocation, 1_000, cliff, duration, now).expect("vested");
			prop_assert!(vested <= allocation);
		}
	}
}
