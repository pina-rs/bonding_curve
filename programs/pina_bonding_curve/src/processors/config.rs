//! `CreateConfig`.

use pina::*;
use pina_amm_cpi::accounts::AmmConfig;

use crate::ID;
use crate::curve::Curve;
use crate::curve::MAX_SEGMENTS;
use crate::curve::base_for_quote;
use crate::curve::derive_quantities;
use crate::errors::CurveError;
use crate::events::ConfigCreated;
use crate::instructions::CreateConfigInstruction;
use crate::instructions::CreateConfigInstructionZc;
use crate::math::Rounding;
use crate::math::share_floor;
use crate::math::sqrt_floor;
use crate::state::AMM_MINIMUM_LIQUIDITY;
use crate::state::AmmAuthority;
use crate::state::FEE_RATE_DENOMINATOR;
use crate::state::LaunchConfig;
use crate::state::MAX_END_FEE_RATE;
use crate::state::MAX_MIGRATION_FEE_RATE;
use crate::state::MAX_START_FEE_RATE;
use crate::token::assert_supported_mint;

/// Accounts for `CreateConfig`.
#[derive(Accounts, Debug)]
pub struct CreateConfigAccounts<'a> {
	/// Pays the configuration's rent.
	#[pina(validate(signer, writable))]
	pub payer: &'a AccountView,
	/// The partner who owns the configuration.
	#[pina(validate(signer))]
	pub authority: &'a AccountView,
	/// The configuration PDA to create: `[b"config", authority, index]`.
	pub config: &'a mut AccountView,
	/// The token every launch raises.
	pub quote_mint: &'a AccountView,
	/// Token program that owns `quote_mint`.
	pub quote_token_program: &'a AccountView,
	/// A Pina AMM fee tier restricted to this program's AMM authority PDA.
	pub amm_config: &'a AccountView,
	/// The system program.
	#[pina(validate(program = system::ID))]
	pub system_program: &'a AccountView,
}

/// Reject fee, share, and vesting parameters outside their limits.
fn validate_terms(args: &CreateConfigInstructionZc) -> ProgramResult {
	let (start, end) = (args.start_fee_rate.get(), args.end_fee_rate.get());
	let decays = args.fee_decay_duration.get() > 0;
	if start > MAX_START_FEE_RATE
		|| end > MAX_END_FEE_RATE
		|| start < end
		|| (start > end) != decays
	{
		return Err(CurveError::InvalidFeeRates.into());
	}
	if args.migration_fee_rate.get() > MAX_MIGRATION_FEE_RATE {
		return Err(CurveError::InvalidFeeRates.into());
	}
	let lp_shares = u64::from(args.creator_lp_share.get()) + u64::from(args.partner_lp_share.get());
	if args.creator_fee_share.get() > FEE_RATE_DENOMINATOR
		|| lp_shares > u64::from(FEE_RATE_DENOMINATOR)
	{
		return Err(CurveError::InvalidShares.into());
	}
	let cliff =
		i64::try_from(args.creator_vesting_cliff.get()).map_err(|_| CurveError::InvalidVesting)?;
	let duration = i64::try_from(args.creator_vesting_duration.get())
		.map_err(|_| CurveError::InvalidVesting)?;
	cliff
		.checked_add(duration)
		.ok_or(CurveError::InvalidVesting)?;
	Ok(())
}

/// Require `amm_config` to be a Pina AMM tier whose pools only this program's
/// AMM authority may create, so no one can create a launch's pool first.
fn assert_restricted_amm_tier(amm_config: &AccountView) -> ProgramResult {
	amm_config
		.assert_owner(&pina_amm_cpi::PINA_AMM_ID)
		.map_err(|_| CurveError::InvalidAmmConfig)?;
	let tier = {
		let data = amm_config.try_borrow()?;
		AmmConfig::parse(&data).ok_or(CurveError::InvalidAmmConfig)?
	};
	let (authority, _) = AmmAuthority::try_find_pda(&ID).ok_or(CurveError::InvalidAmmConfig)?;
	if tier.pool_creator_authority != authority {
		return Err(CurveError::InvalidAmmConfig.into());
	}
	Ok(())
}

impl<'a> ProcessAccountInfos<'a> for CreateConfigAccounts<'a> {
	fn process(self, data: &[u8]) -> ProgramResult {
		let args = CreateConfigInstruction::try_from_bytes(data)?;
		validate_terms(args)?;
		let mut uppers = [0u128; MAX_SEGMENTS];
		let mut liquidities = [0u128; MAX_SEGMENTS];
		for index in 0..MAX_SEGMENTS {
			uppers[index] = args.curve_sqrt_prices[index].get();
			liquidities[index] = args.curve_liquidities[index].get();
		}
		let curve = Curve::new(
			args.sqrt_start_price.get(),
			uppers,
			liquidities,
			args.segment_count,
		)?;
		let quantities = derive_quantities(&curve, args.migration_quote_threshold.get())?;

		let required = quantities
			.sale_supply
			.checked_add(quantities.migration_supply)
			.and_then(|value| value.checked_add(args.creator_allocation.get()))
			.ok_or(CurveError::InvalidSupply)?;
		if args.total_supply.get() < required {
			return Err(CurveError::InvalidSupply.into());
		}

		// The graduation pool must mint more LP than the AMM locks forever,
		// even after the migration fee.
		let pool_quote = quantities
			.migration_quote
			.checked_sub(share_floor(
				quantities.migration_quote,
				args.migration_fee_rate.get(),
			)?)
			.ok_or(CurveError::MathOverflow)?;
		let pool_base =
			base_for_quote(pool_quote, quantities.migration_sqrt_price, Rounding::Down)?;
		if sqrt_floor(u128::from(pool_base) * u128::from(pool_quote))
			<= u128::from(AMM_MINIMUM_LIQUIDITY)
		{
			return Err(CurveError::InsufficientMigrationLiquidity.into());
		}

		assert_supported_mint(self.quote_mint, self.quote_token_program)?;
		assert_restricted_amm_tier(self.amm_config)?;

		let authority = *self.authority.address();
		let quote_mint = *self.quote_mint.address();
		let amm_config = *self.amm_config.address();
		let index = args.index.get();
		CreateProgramAccount {
			account: self.config,
			payer: self.payer,
			owner: &ID,
			seeds: &LaunchConfig::seeds(&authority, index).as_slices(),
		}
		.invoke_with_bump::<LaunchConfig>(|config, bump| {
			config.authority = authority;
			config.quote_mint = quote_mint;
			config.amm_config = amm_config;
			config.sqrt_start_price = args.sqrt_start_price;
			config
				.migration_sqrt_price
				.set(quantities.migration_sqrt_price);
			config.curve_sqrt_prices = args.curve_sqrt_prices;
			config.curve_liquidities = args.curve_liquidities;
			config.total_supply = args.total_supply;
			config.sale_supply.set(quantities.sale_supply);
			config.migration_supply.set(quantities.migration_supply);
			config.creator_allocation = args.creator_allocation;
			config.migration_quote_threshold = args.migration_quote_threshold;
			config.creator_vesting_cliff = args.creator_vesting_cliff;
			config.creator_vesting_duration = args.creator_vesting_duration;
			config.fee_decay_duration = args.fee_decay_duration;
			config.index.set(index);
			config.start_fee_rate = args.start_fee_rate;
			config.end_fee_rate = args.end_fee_rate;
			config.creator_fee_share = args.creator_fee_share;
			config.migration_fee_rate = args.migration_fee_rate;
			config.creator_lp_share = args.creator_lp_share;
			config.partner_lp_share = args.partner_lp_share;
			config.base_decimals = args.base_decimals;
			config.segment_count = args.segment_count;
			config.pool_creator_mode = args.pool_creator_mode;
			config.bump = bump;
			Ok(())
		})?;

		ConfigCreated::emit(|event| {
			event.config = *self.config.address();
			event.authority = authority;
			event.quote_mint = quote_mint;
			event.amm_config = amm_config;
			event
				.migration_sqrt_price
				.set(quantities.migration_sqrt_price);
			event.sale_supply.set(quantities.sale_supply);
			event.migration_supply.set(quantities.migration_supply);
			Ok(())
		})
	}
}
