//! Piecewise concentrated-liquidity price curves.
//!
//! A curve is a list of up to [`MAX_SEGMENTS`] contiguous segments. Each
//! segment holds constant-product liquidity `L` between two square-root
//! prices, exactly like a concentrated-liquidity position, so within a segment
//! the curve behaves like `x * y = L^2`. One segment from the start price to a
//! high upper bound reproduces a classic virtual-reserve bonding curve; more
//! segments approximate any increasing price schedule.
//!
//! Prices are quote base units per base base unit. Square-root prices are
//! Q64.64 fixed point (`sqrt_price = sqrt(price) * 2^64`). For a segment with
//! liquidity `L` moving between square-root prices `a < b`:
//!
//! ```text
//! quote = L * (b - a) / 2^64
//! base  = L * (b - a) * 2^64 / (a * b)
//! ```
//!
//! Buying spends quote and raises the price; selling returns base and lowers
//! it. Every rounding choice favours the curve, so the quote it holds always
//! covers the exact integral of the price curve up to the current price.

use pina::ProgramError;

use crate::errors::CurveError;
use crate::math::Q64;
use crate::math::Rounding;
use crate::math::mul_div;
use crate::math::mul_shr_64;
use crate::math::narrow;

/// The most segments a curve may have.
pub const MAX_SEGMENTS: usize = 16;

/// Smallest supported square-root price: a price of about `2^-64`.
pub const MIN_SQRT_PRICE: u128 = 4_295_048_016;

/// Largest supported square-root price: a price of about `2^64`.
pub const MAX_SQRT_PRICE: u128 = 79_226_673_521_066_979_257_578_248_091;

/// One segment of constant liquidity between two square-root prices.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Segment {
	/// Square-root price where the segment starts.
	pub lower: u128,
	/// Square-root price where the segment ends.
	pub upper: u128,
	/// Liquidity `L` across the segment.
	pub liquidity: u128,
}

/// A validated curve.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Curve {
	start: u128,
	uppers: [u128; MAX_SEGMENTS],
	liquidities: [u128; MAX_SEGMENTS],
	count: usize,
}

impl Curve {
	/// Validate and build a curve.
	///
	/// The start price and every upper bound must lie within
	/// [`MIN_SQRT_PRICE`, `MAX_SQRT_PRICE`], upper bounds must strictly
	/// increase from the start price, every used segment needs positive
	/// liquidity, and unused slots must be zero so each curve has one
	/// canonical encoding.
	pub fn new(
		start: u128,
		uppers: [u128; MAX_SEGMENTS],
		liquidities: [u128; MAX_SEGMENTS],
		count: u8,
	) -> Result<Self, ProgramError> {
		let count = usize::from(count);
		if count == 0 || count > MAX_SEGMENTS || !(MIN_SQRT_PRICE..MAX_SQRT_PRICE).contains(&start)
		{
			return Err(CurveError::InvalidCurve.into());
		}
		let mut lower = start;
		for index in 0..MAX_SEGMENTS {
			let (upper, liquidity) = (uppers[index], liquidities[index]);
			if index >= count {
				if upper != 0 || liquidity != 0 {
					return Err(CurveError::InvalidCurve.into());
				}
				continue;
			}
			if upper <= lower || upper > MAX_SQRT_PRICE || liquidity == 0 {
				return Err(CurveError::InvalidCurve.into());
			}
			lower = upper;
		}
		Ok(Self {
			start,
			uppers,
			liquidities,
			count,
		})
	}

	/// The square-root price a new launch starts at.
	#[must_use]
	pub const fn start(&self) -> u128 {
		self.start
	}

	/// The number of segments.
	#[must_use]
	pub const fn len(&self) -> usize {
		self.count
	}

	/// Whether the curve has no segments; never true for a validated curve.
	#[must_use]
	pub const fn is_empty(&self) -> bool {
		self.count == 0
	}

	/// The segment at `index`, which must be below [`Self::len`].
	#[must_use]
	pub const fn segment(&self, index: usize) -> Segment {
		Segment {
			lower: if index == 0 {
				self.start
			} else {
				self.uppers[index - 1]
			},
			upper: self.uppers[index],
			liquidity: self.liquidities[index],
		}
	}
}

/// Quote that moves the price from `lower` to `upper` in one segment:
/// `L * (upper - lower) / 2^64`.
pub fn quote_delta(
	lower: u128,
	upper: u128,
	liquidity: u128,
	rounding: Rounding,
) -> Result<u64, ProgramError> {
	let span = upper.checked_sub(lower).ok_or(CurveError::MathOverflow)?;
	narrow(mul_shr_64(liquidity, span, rounding)?)
}

/// Base that moves the price between `lower` and `upper` in one segment:
/// `L * (upper - lower) * 2^64 / (lower * upper)`.
///
/// Evaluated as `L * 2^64 / lower - L * 2^64 / upper`, each term rounded once
/// in the requested direction, so the result is within two base units of the
/// exact value.
pub fn base_delta(
	lower: u128,
	upper: u128,
	liquidity: u128,
	rounding: Rounding,
) -> Result<u64, ProgramError> {
	if upper < lower {
		return Err(CurveError::MathOverflow.into());
	}
	let (inner, outer) = match rounding {
		Rounding::Down => (Rounding::Down, Rounding::Up),
		Rounding::Up => (Rounding::Up, Rounding::Down),
	};
	let from_lower = mul_div(liquidity, Q64, lower, inner)?;
	let from_upper = mul_div(liquidity, Q64, upper, outer)?;
	narrow(from_lower.saturating_sub(from_upper))
}

/// The square-root price after `quote` is spent from `sqrt_price`, rounded
/// down so the buyer never receives more base than they paid for.
pub fn next_sqrt_price_from_quote(
	sqrt_price: u128,
	liquidity: u128,
	quote: u64,
) -> Result<u128, ProgramError> {
	let step = (u128::from(quote) << 64) / liquidity;
	sqrt_price
		.checked_add(step)
		.ok_or_else(|| CurveError::MathOverflow.into())
}

/// The square-root price after `base` is sold at `sqrt_price`, rounded up so
/// the seller never receives more quote than the base is worth:
/// `L * s / (L + base * s / 2^64)`.
pub fn next_sqrt_price_from_base(
	sqrt_price: u128,
	liquidity: u128,
	base: u64,
) -> Result<u128, ProgramError> {
	let added = mul_shr_64(u128::from(base), sqrt_price, Rounding::Down)?;
	let denominator = liquidity
		.checked_add(added)
		.ok_or(CurveError::MathOverflow)?;
	mul_div(liquidity, sqrt_price, denominator, Rounding::Up)
}

/// The outcome of spending quote on the curve.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Buy {
	/// Base the buyer receives.
	pub base_out: u64,
	/// Quote the curve consumed. Less than the amount offered when the buy
	/// reached the migration price.
	pub quote_used: u64,
	/// The square-root price after the buy.
	pub sqrt_price: u128,
	/// Whether the buy reached the migration price.
	pub completed: bool,
}

/// Spend up to `quote_in` from `sqrt_price`, stopping at `cap`.
pub fn buy(curve: &Curve, sqrt_price: u128, quote_in: u64, cap: u128) -> Result<Buy, ProgramError> {
	let mut price = sqrt_price;
	let mut remaining = quote_in;
	let mut base_out = 0u64;
	let mut index = 0;
	while index < curve.len() && curve.segment(index).upper <= price {
		index += 1;
	}
	while remaining > 0 && price < cap && index < curve.len() {
		let segment = curve.segment(index);
		let target = segment.upper.min(cap);
		let needed = quote_delta(price, target, segment.liquidity, Rounding::Up)?;
		let (next, spent) = if remaining >= needed {
			index += 1;
			(target, needed)
		} else {
			let next = next_sqrt_price_from_quote(price, segment.liquidity, remaining)?;
			(next.min(target), remaining)
		};
		let received = base_delta(price, next, segment.liquidity, Rounding::Down)?;
		base_out = base_out
			.checked_add(received)
			.ok_or(CurveError::MathOverflow)?;
		remaining -= spent;
		price = next;
	}
	Ok(Buy {
		base_out,
		quote_used: quote_in - remaining,
		sqrt_price: price,
		completed: price >= cap,
	})
}

/// The outcome of selling base to the curve.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Sell {
	/// Quote the seller receives before fees.
	pub quote_out: u64,
	/// The square-root price after the sale.
	pub sqrt_price: u128,
}

/// Sell exactly `base_in` from `sqrt_price`.
///
/// # Errors
///
/// [`CurveError::InsufficientLiquidity`] when the curve cannot absorb the
/// whole amount before reaching its start price.
pub fn sell(curve: &Curve, sqrt_price: u128, base_in: u64) -> Result<Sell, ProgramError> {
	let mut price = sqrt_price;
	let mut remaining = base_in;
	let mut quote_out = 0u64;
	let mut index = curve.len();
	while index > 0 && curve.segment(index - 1).lower >= price {
		index -= 1;
	}
	while remaining > 0 && index > 0 {
		let segment = curve.segment(index - 1);
		let needed = base_delta(segment.lower, price, segment.liquidity, Rounding::Up)?;
		let (next, spent) = if remaining >= needed {
			index -= 1;
			(segment.lower, needed)
		} else {
			let next = next_sqrt_price_from_base(price, segment.liquidity, remaining)?;
			(next.max(segment.lower), remaining)
		};
		let received = quote_delta(next, price, segment.liquidity, Rounding::Down)?;
		quote_out = quote_out
			.checked_add(received)
			.ok_or(CurveError::MathOverflow)?;
		remaining -= spent;
		price = next;
	}
	if remaining > 0 {
		return Err(CurveError::InsufficientLiquidity.into());
	}
	Ok(Sell {
		quote_out,
		sqrt_price: price,
	})
}

/// Quote the curve must hold at `sqrt_price`, rounded down: the integral of
/// the price curve from the start price.
pub fn quote_floor(curve: &Curve, sqrt_price: u128) -> Result<u64, ProgramError> {
	let mut total = 0u64;
	for index in 0..curve.len() {
		let segment = curve.segment(index);
		if segment.lower >= sqrt_price {
			break;
		}
		let upper = segment.upper.min(sqrt_price);
		total = total
			.checked_add(quote_delta(
				segment.lower,
				upper,
				segment.liquidity,
				Rounding::Down,
			)?)
			.ok_or(CurveError::MathOverflow)?;
	}
	Ok(total)
}

/// Quantities a launch configuration derives from its curve.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Quantities {
	/// The square-root price at which a launch completes.
	pub migration_sqrt_price: u128,
	/// Base sold between the start and migration prices, rounded up.
	pub sale_supply: u64,
	/// Quote raised between the start and migration prices, rounded up.
	pub migration_quote: u64,
	/// Base that seeds the AMM pool with `migration_quote` at the migration
	/// price, rounded up.
	pub migration_supply: u64,
}

/// Walk the curve until `threshold` quote has been raised.
///
/// # Errors
///
/// [`CurveError::InvalidMigrationThreshold`] when the threshold is zero, too
/// small to move the price, or beyond the curve's last segment.
pub fn derive_quantities(curve: &Curve, threshold: u64) -> Result<Quantities, ProgramError> {
	if threshold == 0 {
		return Err(CurveError::InvalidMigrationThreshold.into());
	}
	let mut remaining = threshold;
	let mut sale_supply = 0u64;
	let mut migration_quote = 0u64;
	for index in 0..curve.len() {
		let segment = curve.segment(index);
		let full = quote_delta(
			segment.lower,
			segment.upper,
			segment.liquidity,
			Rounding::Up,
		)?;
		let upper = if remaining >= full {
			segment.upper
		} else {
			next_sqrt_price_from_quote(segment.lower, segment.liquidity, remaining)?
		};
		if upper <= segment.lower {
			return Err(CurveError::InvalidMigrationThreshold.into());
		}
		sale_supply = sale_supply
			.checked_add(base_delta(
				segment.lower,
				upper,
				segment.liquidity,
				Rounding::Up,
			)?)
			.ok_or(CurveError::MathOverflow)?;
		migration_quote = migration_quote
			.checked_add(quote_delta(
				segment.lower,
				upper,
				segment.liquidity,
				Rounding::Up,
			)?)
			.ok_or(CurveError::MathOverflow)?;
		remaining = remaining.saturating_sub(full);
		if upper < segment.upper || remaining == 0 {
			return Ok(Quantities {
				migration_sqrt_price: upper,
				sale_supply,
				migration_quote,
				migration_supply: base_for_quote(migration_quote, upper, Rounding::Up)?,
			});
		}
	}
	Err(CurveError::InvalidMigrationThreshold.into())
}

/// Base worth `quote` at `sqrt_price`: `quote * 2^128 / sqrt_price^2`.
pub fn base_for_quote(
	quote: u64,
	sqrt_price: u128,
	rounding: Rounding,
) -> Result<u64, ProgramError> {
	let partial = mul_div(u128::from(quote), Q64, sqrt_price, rounding)?;
	narrow(mul_div(partial, Q64, sqrt_price, rounding)?)
}

/// Quote worth `base` at `sqrt_price`: `base * sqrt_price^2 / 2^128`.
pub fn quote_for_base(
	base: u64,
	sqrt_price: u128,
	rounding: Rounding,
) -> Result<u64, ProgramError> {
	let partial = mul_shr_64(u128::from(base), sqrt_price, rounding)?;
	narrow(mul_shr_64(partial, sqrt_price, rounding)?)
}

/// How a completed launch seeds its AMM pool.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Seed {
	/// Base deposited into the pool.
	pub base: u64,
	/// Quote deposited into the pool.
	pub quote: u64,
	/// Base left over and burned.
	pub base_surplus: u64,
	/// Quote left over, added to the migration fee.
	pub quote_surplus: u64,
}

/// Seed a pool at `sqrt_price` from `available_base` and `pool_quote`.
///
/// The pool opens at the curve's final price. Whichever side is in excess at
/// that price is the surplus: excess base is burned, and excess quote (only
/// ever rounding dust from many trades) joins the migration fee.
pub fn seed_pool(
	available_base: u64,
	pool_quote: u64,
	sqrt_price: u128,
) -> Result<Seed, ProgramError> {
	let ideal_base = base_for_quote(pool_quote, sqrt_price, Rounding::Down)?;
	let seed = if ideal_base <= available_base {
		Seed {
			base: ideal_base,
			quote: pool_quote,
			base_surplus: available_base - ideal_base,
			quote_surplus: 0,
		}
	} else {
		let quote = quote_for_base(available_base, sqrt_price, Rounding::Up)?.min(pool_quote);
		Seed {
			base: available_base,
			quote,
			base_surplus: 0,
			quote_surplus: pool_quote - quote,
		}
	};
	if seed.base == 0 || seed.quote == 0 {
		return Err(CurveError::InsufficientMigrationLiquidity.into());
	}
	Ok(seed)
}

#[cfg(test)]
mod tests {
	extern crate std;

	use proptest::prelude::*;

	use super::*;

	/// A single-segment curve shaped like a classic launch: about 830 million
	/// tokens (6 decimals) sold as the market value of a billion tokens rises
	/// from about 28 SOL to about 1,800 SOL, raising up to about 185 SOL.
	fn launch_curve() -> Curve {
		// sqrt(price) * 2^64 with price = 2.8e-5 lamports per base unit.
		let start = 97_610_000_000_000_000;
		let mut uppers = [0; MAX_SEGMENTS];
		let mut liquidities = [0; MAX_SEGMENTS];
		uppers[0] = start * 8;
		liquidities[0] = 5_000_000_000_000;
		Curve::new(start, uppers, liquidities, 1).expect("curve")
	}

	/// Three segments whose liquidity rises, so the price climbs fast and then
	/// slows.
	fn stepped_curve() -> Curve {
		let start = 97_610_000_000_000_000;
		let mut uppers = [0; MAX_SEGMENTS];
		let mut liquidities = [0; MAX_SEGMENTS];
		uppers[0] = start * 2;
		uppers[1] = start * 3;
		uppers[2] = start * 8;
		liquidities[0] = 1_000_000_000_000;
		liquidities[1] = 4_000_000_000_000;
		liquidities[2] = 9_000_000_000_000;
		Curve::new(start, uppers, liquidities, 3).expect("curve")
	}

	#[test]
	fn rejects_malformed_curves() {
		let mut uppers = [0; MAX_SEGMENTS];
		let mut liquidities = [0; MAX_SEGMENTS];
		let start = MIN_SQRT_PRICE;
		assert!(Curve::new(start, uppers, liquidities, 0).is_err());
		uppers[0] = start;
		liquidities[0] = 1;
		assert!(
			Curve::new(start, uppers, liquidities, 1).is_err(),
			"upper must exceed start"
		);
		uppers[0] = start + 1;
		liquidities[0] = 0;
		assert!(
			Curve::new(start, uppers, liquidities, 1).is_err(),
			"liquidity must be positive"
		);
		liquidities[0] = 1;
		assert!(Curve::new(start, uppers, liquidities, 1).is_ok());
		uppers[1] = start + 2;
		assert!(
			Curve::new(start, uppers, liquidities, 1).is_err(),
			"unused slots must be zero"
		);
		assert!(Curve::new(start - 1, [0; MAX_SEGMENTS], [0; MAX_SEGMENTS], 1).is_err());
	}

	#[test]
	fn a_full_buy_completes_at_the_derived_price() {
		for curve in [launch_curve(), stepped_curve()] {
			let quantities = derive_quantities(&curve, 85_000_000_000).expect("quantities");
			let result = buy(
				&curve,
				curve.start(),
				u64::MAX / 2,
				quantities.migration_sqrt_price,
			)
			.expect("buy");
			assert!(result.completed);
			assert_eq!(result.sqrt_price, quantities.migration_sqrt_price);
			assert_eq!(result.quote_used, quantities.migration_quote);
			assert!(result.base_out <= quantities.sale_supply);
			// Configuration rounds every segment up and the buy rounds down, each
			// within two base units.
			assert!(quantities.sale_supply - result.base_out <= curve.len() as u64 * 4);
		}
	}

	#[test]
	fn unreachable_thresholds_are_rejected() {
		let curve = launch_curve();
		assert!(derive_quantities(&curve, 0).is_err());
		assert!(derive_quantities(&curve, u64::MAX).is_err());
	}

	#[test]
	fn selling_more_than_was_bought_fails() {
		let curve = launch_curve();
		let bought = buy(&curve, curve.start(), 1_000_000_000, u128::MAX).expect("buy");
		assert!(sell(&curve, bought.sqrt_price, bought.base_out + 1_000_000).is_err());
		let sold = sell(&curve, bought.sqrt_price, bought.base_out).expect("sell");
		assert!(sold.quote_out <= 1_000_000_000);
	}

	#[test]
	fn the_pool_opens_at_the_curve_price() {
		let curve = launch_curve();
		let quantities = derive_quantities(&curve, 85_000_000_000).expect("quantities");
		let price = quantities.migration_sqrt_price;
		let seed = seed_pool(
			quantities.migration_supply,
			quantities.migration_quote,
			price,
		)
		.expect("seed");
		assert_eq!(seed.quote, quantities.migration_quote);
		assert_eq!(seed.quote_surplus, 0);
		// The pool opens at the curve's final price to within one part in a
		// billion.
		let implied = quote_for_base(seed.base, price, Rounding::Down).expect("implied");
		assert!(implied <= seed.quote);
		assert!(
			(seed.quote - implied) * 1_000_000_000 <= seed.quote,
			"{implied} vs {}",
			seed.quote
		);

		// With less base than the ideal, the quote side gives way instead.
		let scarce = seed_pool(seed.base / 2, quantities.migration_quote, price).expect("seed");
		assert_eq!(scarce.base, seed.base / 2);
		assert!(scarce.quote_surplus > 0);
		assert_eq!(
			scarce.quote + scarce.quote_surplus,
			quantities.migration_quote
		);
	}

	fn curves() -> impl Strategy<Value = Curve> {
		prop_oneof![Just(launch_curve()), Just(stepped_curve())]
	}

	proptest! {
		#![proptest_config(ProptestConfig::with_cases(1_024))]

		/// However trades interleave, the quote the curve holds always covers
		/// the integral of the curve up to its current price, and no buyer
		/// receives base the curve has not priced.
		#[test]
		fn reserves_always_cover_the_curve(
			curve in curves(),
			trades in proptest::collection::vec((any::<bool>(), 1u64..50_000_000_000), 1..24),
		) {
			let quantities = derive_quantities(&curve, 85_000_000_000).expect("quantities");
			let cap = quantities.migration_sqrt_price;
			let mut price = curve.start();
			let mut reserve = 0u64;
			let mut circulating = 0u64;
			for (is_buy, amount) in trades {
				if is_buy {
					let result = buy(&curve, price, amount, cap).expect("buy");
					reserve += result.quote_used;
					circulating += result.base_out;
					price = result.sqrt_price;
				} else if circulating > 0 {
					let amount = amount % circulating + 1;
					let result = sell(&curve, price, amount).expect("a sale of bought base succeeds");
					prop_assert!(result.quote_out <= reserve);
					reserve -= result.quote_out;
					circulating -= amount;
					price = result.sqrt_price;
				}
				prop_assert!(price <= cap);
				prop_assert!(reserve >= quote_floor(&curve, price).expect("floor"));
				prop_assert!(circulating <= quantities.sale_supply);
			}
		}

		/// Buying and immediately selling back never returns more quote than
		/// it cost.
		#[test]
		fn round_trips_never_profit(curve in curves(), amount in 1u64..80_000_000_000) {
			let bought = buy(&curve, curve.start(), amount, u128::MAX).expect("buy");
			if bought.base_out > 0 {
				let sold = sell(&curve, bought.sqrt_price, bought.base_out).expect("sell");
				prop_assert!(sold.quote_out <= bought.quote_used);
			}
		}

		/// Splitting a buy can recover a few base units of rounding, but never
		/// more than the exact curve allows for the same quote.
		#[test]
		fn splitting_buys_never_beats_the_curve(curve in curves(), first in 1u64..40_000_000_000, second in 1u64..40_000_000_000) {
			let whole = buy(&curve, curve.start(), first + second, u128::MAX).expect("whole");
			let part = buy(&curve, curve.start(), first, u128::MAX).expect("first");
			let rest = buy(&curve, part.sqrt_price, second, u128::MAX).expect("second");
			prop_assert!(rest.sqrt_price <= whole.sqrt_price);
			let mut ceiling = 0u64;
			for index in 0..curve.len() {
				let segment = curve.segment(index);
				if segment.lower >= whole.sqrt_price {
					break;
				}
				let upper = segment.upper.min(whole.sqrt_price);
				ceiling += base_delta(segment.lower, upper, segment.liquidity, Rounding::Up).expect("ceiling");
			}
			prop_assert!(part.base_out + rest.base_out <= ceiling);
		}
	}
}
