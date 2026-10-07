//! Offline curve design: price conversions and segment sizing.
//!
//! The program stores prices as Q64.64 square roots and sizes each segment by
//! its liquidity `L`. Within a segment from `sqrt_a` to `sqrt_b`:
//!
//! - quote raised = `L * (sqrt_b - sqrt_a) / 2^64`
//! - base sold = `L * 2^64 * (sqrt_b - sqrt_a) / (sqrt_a * sqrt_b)`
//!
//! These helpers turn human prices into those integers. Price conversion goes
//! through `f64`, which is precise to about one part in 10^15: plenty for
//! choosing a curve, and the program validates the exact integers anyway. Run
//! `config create --simulate` to see the exact supplies the program derives.

use serde_json::json;

use crate::cli::DesignCommand;
use crate::error::CliError;
use crate::output;

/// `2^64`, the Q64.64 scale.
const Q64: f64 = 18_446_744_073_709_551_616.0;
/// Smallest square-root price the program accepts.
pub const MIN_SQRT_PRICE: u128 = 4_295_048_016;
/// Largest square-root price the program accepts.
pub const MAX_SQRT_PRICE: u128 = 79_226_673_521_066_979_257_578_248_091;

/// Run one `design` command.
pub fn run(command: &DesignCommand, json: bool) -> Result<(), CliError> {
	let value = match command {
		DesignCommand::SqrtPrice(args) => {
			json!({ "price": args.price, "sqrt_price": sqrt_price(args.price)?.to_string() })
		}
		DesignCommand::Price(args) => {
			json!({ "sqrt_price": args.sqrt_price.to_string(), "price": price(args.sqrt_price) })
		}
		DesignCommand::Segment(args) => {
			let segment = segment(args.from_price, args.to_price, args.quote)?;
			json!({
				"sqrt_start_price": segment.lower.to_string(),
				"sqrt_price": segment.upper.to_string(),
				"liquidity": segment.liquidity.to_string(),
				"quote": args.quote,
				"approximate_base_sold": segment.base_sold,
			})
		}
	};
	output::print(&value, json, "");
	Ok(())
}

/// The Q64.64 square-root price of `price` quote units per base unit.
pub fn sqrt_price(price: f64) -> Result<u128, CliError> {
	if !price.is_finite() || price <= 0.0 {
		return Err(CliError::InvalidDesign(format!(
			"price must be a positive number, got {price}"
		)));
	}
	let scaled = price.sqrt() * Q64;
	// `as` saturates; the range check below rejects anything that did.
	let value = scaled.round() as u128;
	if !(MIN_SQRT_PRICE..=MAX_SQRT_PRICE).contains(&value) {
		return Err(CliError::InvalidDesign(format!(
			"price {price} is outside the range the program supports"
		)));
	}
	Ok(value)
}

/// The price a Q64.64 square-root price represents.
pub fn price(sqrt_price: u128) -> f64 {
	let root = sqrt_price as f64 / Q64;
	root * root
}

/// One sized segment.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SizedSegment {
	/// Square-root price where the segment starts.
	pub lower: u128,
	/// Square-root price where the segment ends.
	pub upper: u128,
	/// Liquidity that raises at least the requested quote across the segment.
	pub liquidity: u128,
	/// Base units the segment sells, approximately.
	pub base_sold: f64,
}

/// Size the segment that raises `quote` between two prices.
///
/// The liquidity rounds up, so the segment raises at least `quote`.
pub fn segment(from_price: f64, to_price: f64, quote: u64) -> Result<SizedSegment, CliError> {
	let lower = sqrt_price(from_price)?;
	let upper = sqrt_price(to_price)?;
	if upper <= lower {
		return Err(CliError::InvalidDesign(
			"the segment must end at a higher price than it starts".to_owned(),
		));
	}
	if quote == 0 {
		return Err(CliError::InvalidDesign(
			"the segment must raise some quote".to_owned(),
		));
	}
	// `quote < 2^64`, so `quote << 64` fits in 128 bits.
	let liquidity = (u128::from(quote) << 64).div_ceil(upper - lower);
	let base_sold = liquidity as f64 * Q64 * (upper - lower) as f64 / (lower as f64 * upper as f64);
	Ok(SizedSegment {
		lower,
		upper,
		liquidity,
		base_sold,
	})
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn price_conversion_round_trips() {
		for value in [2.8e-5, 1.0, 0.5, 1_234.5] {
			let root = sqrt_price(value).expect("in range");
			assert!((price(root) / value - 1.0).abs() < 1e-12, "{value}");
		}
		// 2.8e-5 is the reference curve's start price.
		assert!(
			sqrt_price(2.8e-5)
				.expect("start")
				.abs_diff(97_610_000_000_000_000)
				< 1_000_000_000_000
		);
	}

	#[test]
	fn rejects_prices_outside_the_program_range() {
		for value in [0.0, -1.0, f64::NAN, f64::INFINITY, 1e-40, 1e40] {
			assert!(sqrt_price(value).is_err(), "{value}");
		}
	}

	#[test]
	fn segments_raise_at_least_the_requested_quote() {
		let sized = segment(2.8e-5, 2.8e-5 * 64.0, 85_000_000_000).expect("segment");
		let raised = (sized.liquidity * (sized.upper - sized.lower)) >> 64;
		assert!(raised >= 85_000_000_000);
		assert!(raised - 85_000_000_000 <= 1);
		assert!(sized.base_sold > 0.0);
		assert!(segment(1.0, 1.0, 1).is_err());
		assert!(segment(1.0, 2.0, 0).is_err());
	}
}
