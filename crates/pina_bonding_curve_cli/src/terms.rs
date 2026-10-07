//! The terms file `config create` reads.
//!
//! A configuration has more parameters than flags read comfortably, and a
//! partner usually reviews and reuses the same terms across environments, so
//! they live in a JSON file. Square-root prices and liquidities are Q64.64 and
//! 128-bit wide; write them as decimal strings because JSON numbers lose
//! precision above 2^53 in most tools.
//!
//! ```json
//! {
//!   "sqrt_start_price": "97610000000000000",
//!   "segments": [{ "sqrt_price": "780880000000000000", "liquidity": "5000000000000" }],
//!   "migration_quote_threshold": 85000000000,
//!   "total_supply": 1000000000000000,
//!   "base_decimals": 6,
//!   "start_fee_rate": 500000,
//!   "end_fee_rate": 10000,
//!   "fee_decay_duration": 60
//! }
//! ```

use std::fmt;
use std::path::Path;

use pina_bonding_curve_client::instructions::CreateConfigInstructionData;
use serde::Deserialize;
use serde::Deserializer;
use serde::de;

use crate::error::CliError;

/// Most segments one curve can have.
pub const MAX_SEGMENTS: usize = 16;

/// Who receives the graduated pool's creator fees.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PoolCreator {
	/// Each launch's creator.
	#[default]
	Creator,
	/// The configuration's authority.
	Partner,
}

/// One curve segment: liquidity that applies up to `sqrt_price`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Segment {
	/// Upper square-root price of the segment (Q64.64).
	#[serde(deserialize_with = "u128_from_string_or_number")]
	pub sqrt_price: u128,
	/// Liquidity across the segment.
	#[serde(deserialize_with = "u128_from_string_or_number")]
	pub liquidity: u128,
}

/// The economic terms of a configuration. Field names and units match the
/// program's `CreateConfig` instruction; rates and shares are parts per
/// million and durations are seconds.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Terms {
	/// Square-root price where every launch starts (Q64.64).
	#[serde(deserialize_with = "u128_from_string_or_number")]
	pub sqrt_start_price: u128,
	/// One to sixteen segments with strictly increasing upper prices.
	pub segments: Vec<Segment>,
	/// Quote reserve at which a launch completes.
	pub migration_quote_threshold: u64,
	/// Fixed supply minted for every launch, in base units.
	pub total_supply: u64,
	/// Decimals every launched mint must have.
	pub base_decimals: u8,
	/// Base units reserved for the creator, vesting from each launch's activation.
	#[serde(default)]
	pub creator_allocation: u64,
	/// Seconds after activation before any allocation vests.
	#[serde(default)]
	pub creator_vesting_cliff: u64,
	/// Seconds after the cliff over which the allocation vests linearly.
	#[serde(default)]
	pub creator_vesting_duration: u64,
	/// Trading fee at activation.
	pub start_fee_rate: u32,
	/// Trading fee once the decay ends.
	pub end_fee_rate: u32,
	/// Seconds over which the fee falls from the start to the end rate.
	#[serde(default)]
	pub fee_decay_duration: u64,
	/// Creator's share of trading and migration fees.
	#[serde(default)]
	pub creator_fee_share: u32,
	/// Share of the raised quote taken as a fee at graduation.
	#[serde(default)]
	pub migration_fee_rate: u32,
	/// Share of the graduated pool's LP paid to the creator; the rest of the
	/// LP not paid out is burned.
	#[serde(default)]
	pub creator_lp_share: u32,
	/// Share of the graduated pool's LP paid to the partner.
	#[serde(default)]
	pub partner_lp_share: u32,
	/// Who receives the graduated pool's creator fees.
	#[serde(default)]
	pub pool_creator: PoolCreator,
}

impl Terms {
	/// Read and check a terms file.
	pub fn load(path: &Path) -> Result<Self, CliError> {
		let failure = |reason: String| {
			CliError::Terms {
				path: path.display().to_string(),
				reason,
			}
		};
		let contents = std::fs::read_to_string(path).map_err(|error| failure(error.to_string()))?;
		let terms: Self =
			serde_json::from_str(&contents).map_err(|error| failure(error.to_string()))?;
		if terms.segments.is_empty() || terms.segments.len() > MAX_SEGMENTS {
			return Err(failure(format!(
				"expected 1 to {MAX_SEGMENTS} segments, found {}",
				terms.segments.len()
			)));
		}
		Ok(terms)
	}

	/// Encode `CreateConfig` instruction data for configuration `index`.
	pub fn instruction_data(&self, index: u64) -> Result<CreateConfigInstructionData, CliError> {
		CreateConfigInstructionData::new(|data| {
			data.index.set(index);
			data.sqrt_start_price.set(self.sqrt_start_price);
			for (slot, segment) in self.segments.iter().enumerate() {
				data.curve_sqrt_prices[slot].set(segment.sqrt_price);
				data.curve_liquidities[slot].set(segment.liquidity);
			}
			data.migration_quote_threshold
				.set(self.migration_quote_threshold);
			data.total_supply.set(self.total_supply);
			data.creator_allocation.set(self.creator_allocation);
			data.creator_vesting_cliff.set(self.creator_vesting_cliff);
			data.creator_vesting_duration
				.set(self.creator_vesting_duration);
			data.fee_decay_duration.set(self.fee_decay_duration);
			data.start_fee_rate.set(self.start_fee_rate);
			data.end_fee_rate.set(self.end_fee_rate);
			data.creator_fee_share.set(self.creator_fee_share);
			data.migration_fee_rate.set(self.migration_fee_rate);
			data.creator_lp_share.set(self.creator_lp_share);
			data.partner_lp_share.set(self.partner_lp_share);
			// `load` bounds the segment count to 16.
			data.segment_count = self.segments.len() as u8;
			data.base_decimals = self.base_decimals;
			data.pool_creator_mode = match self.pool_creator {
				PoolCreator::Creator => 0,
				PoolCreator::Partner => 1,
			};
		})
		.map_err(|_| CliError::InvalidInstructionData)
	}
}

/// Accept a 128-bit integer as a decimal string or as a JSON number.
fn u128_from_string_or_number<'de, D: Deserializer<'de>>(
	deserializer: D,
) -> Result<u128, D::Error> {
	struct Visitor;

	impl de::Visitor<'_> for Visitor {
		type Value = u128;

		fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
			formatter.write_str("an unsigned 128-bit integer as a decimal string or a number")
		}

		fn visit_u64<E: de::Error>(self, value: u64) -> Result<u128, E> {
			Ok(u128::from(value))
		}

		fn visit_u128<E: de::Error>(self, value: u128) -> Result<u128, E> {
			Ok(value)
		}

		fn visit_str<E: de::Error>(self, value: &str) -> Result<u128, E> {
			value.parse().map_err(E::custom)
		}
	}

	deserializer.deserialize_any(Visitor)
}

#[cfg(test)]
mod tests {
	use pina_bonding_curve_client::instructions::CreateConfig;

	use super::*;

	fn parse(json: &str) -> Result<Terms, serde_json::Error> {
		serde_json::from_str(json)
	}

	const MINIMAL: &str = r#"{
		"sqrt_start_price": "97610000000000000",
		"segments": [{ "sqrt_price": "780880000000000000", "liquidity": 5000000000000 }],
		"migration_quote_threshold": 85000000000,
		"total_supply": 1000000000000000,
		"base_decimals": 6,
		"start_fee_rate": 10000,
		"end_fee_rate": 10000
	}"#;

	#[test]
	fn reads_strings_and_numbers_and_applies_defaults() {
		let terms = parse(MINIMAL).expect("terms");
		assert_eq!(terms.sqrt_start_price, 97_610_000_000_000_000);
		assert_eq!(terms.segments[0].liquidity, 5_000_000_000_000);
		assert_eq!(terms.creator_allocation, 0);
		assert_eq!(terms.pool_creator, PoolCreator::Creator);
		let key = solana_pubkey::Pubkey::new_unique();
		let instruction = CreateConfig::new(key, key, key, key, key, key)
			.instruction(terms.instruction_data(7).expect("data"));
		// Discriminator, then the u64 index, then the u128 start price.
		assert_eq!(instruction.data[0], 0);
		assert_eq!(instruction.data[1..9], 7u64.to_le_bytes());
		assert_eq!(
			instruction.data[9..25],
			97_610_000_000_000_000u128.to_le_bytes()
		);
	}

	#[test]
	fn keeps_full_128_bit_precision() {
		let json = MINIMAL.replace(
			"\"780880000000000000\"",
			"\"79226673521066979257578248091\"",
		);
		let terms = parse(&json).expect("terms");
		assert_eq!(
			terms.segments[0].sqrt_price,
			79_226_673_521_066_979_257_578_248_091
		);
	}

	#[test]
	fn rejects_unknown_fields_and_bad_integers() {
		let typo = MINIMAL.replace("\"end_fee_rate\"", "\"end_fee\"");
		assert!(parse(&typo).is_err());
		let negative = MINIMAL.replace("\"97610000000000000\"", "\"-1\"");
		assert!(parse(&negative).is_err());
		let partner = MINIMAL.replace(
			"\"end_fee_rate\": 10000",
			"\"end_fee_rate\": 10000, \"pool_creator\": \"partner\"",
		);
		assert_eq!(
			parse(&partner).expect("partner").pool_creator,
			PoolCreator::Partner
		);
	}
}
