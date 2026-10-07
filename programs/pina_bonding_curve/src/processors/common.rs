//! Configuration and launch loading shared by the processors.

use pina::sysvars::Sysvar;
use pina::sysvars::clock::Clock;
use pina::*;

use crate::ID;
use crate::curve::Curve;
use crate::curve::MAX_SEGMENTS;
use crate::errors::CurveError;
use crate::state::Launch;
use crate::state::LaunchConfig;
use crate::state::LaunchStatus;

/// The current Unix timestamp.
pub(crate) fn now() -> Result<i64, ProgramError> {
	Ok(Clock::get()?.unix_timestamp)
}

/// A copy of a configuration, taken so the borrow ends before any CPI.
#[derive(Clone, Copy)]
pub(crate) struct ConfigSnapshot {
	pub authority: Address,
	pub quote_mint: Address,
	pub amm_config: Address,
	pub curve: Curve,
	pub migration_sqrt_price: u128,
	pub total_supply: u64,
	pub creator_allocation: u64,
	pub creator_vesting_cliff: u64,
	pub creator_vesting_duration: u64,
	pub fee_decay_duration: u64,
	pub start_fee_rate: u32,
	pub end_fee_rate: u32,
	pub creator_fee_share: u32,
	pub migration_fee_rate: u32,
	pub creator_lp_share: u32,
	pub partner_lp_share: u32,
	pub base_decimals: u8,
	pub pool_creator_mode: u8,
}

impl ConfigSnapshot {
	/// Load a configuration. The typed load checks ownership, discriminator,
	/// schema version, and size; only this program creates accounts that
	/// pass it, and every one was validated when it was created.
	pub fn load(config: &AccountView) -> Result<Self, ProgramError> {
		let state = config.as_account::<LaunchConfig>(&ID)?;
		let mut uppers = [0u128; MAX_SEGMENTS];
		let mut liquidities = [0u128; MAX_SEGMENTS];
		for index in 0..MAX_SEGMENTS {
			uppers[index] = state.curve_sqrt_prices[index].get();
			liquidities[index] = state.curve_liquidities[index].get();
		}
		Ok(Self {
			authority: state.authority,
			quote_mint: state.quote_mint,
			amm_config: state.amm_config,
			curve: Curve::new(
				state.sqrt_start_price.get(),
				uppers,
				liquidities,
				state.segment_count,
			)?,
			migration_sqrt_price: state.migration_sqrt_price.get(),
			total_supply: state.total_supply.get(),
			creator_allocation: state.creator_allocation.get(),
			creator_vesting_cliff: state.creator_vesting_cliff.get(),
			creator_vesting_duration: state.creator_vesting_duration.get(),
			fee_decay_duration: state.fee_decay_duration.get(),
			start_fee_rate: state.start_fee_rate.get(),
			end_fee_rate: state.end_fee_rate.get(),
			creator_fee_share: state.creator_fee_share.get(),
			migration_fee_rate: state.migration_fee_rate.get(),
			creator_lp_share: state.creator_lp_share.get(),
			partner_lp_share: state.partner_lp_share.get(),
			base_decimals: state.base_decimals,
			pool_creator_mode: state.pool_creator_mode,
		})
	}
}

/// A copy of a launch, taken so the borrow ends before any CPI.
#[derive(Clone, Copy)]
pub(crate) struct LaunchSnapshot {
	pub creator: Address,
	pub base_mint: Address,
	pub quote_mint: Address,
	pub base_vault: Address,
	pub quote_vault: Address,
	pub sqrt_price: u128,
	pub quote_reserve: u64,
	pub partner_fees: u64,
	pub creator_fees: u64,
	pub creator_claimed: u64,
	pub activation_time: i64,
	pub status: LaunchStatus,
	pub bump: u8,
}

impl LaunchSnapshot {
	/// Load a launch and require it to follow `config`.
	pub fn load(launch: &AccountView, config: &AccountView) -> Result<Self, ProgramError> {
		let state = launch.as_account::<Launch>(&ID)?;
		if &state.config != config.address() {
			return Err(CurveError::AccountMismatch.into());
		}
		Ok(Self {
			creator: state.creator,
			base_mint: state.base_mint,
			quote_mint: state.quote_mint,
			base_vault: state.base_vault,
			quote_vault: state.quote_vault,
			sqrt_price: state.sqrt_price.get(),
			quote_reserve: state.quote_reserve.get(),
			partner_fees: state.partner_fees.get(),
			creator_fees: state.creator_fees.get(),
			creator_claimed: state.creator_claimed.get(),
			activation_time: state.activation_time.get(),
			status: LaunchStatus::from_u8(state.status).ok_or(CurveError::AccountMismatch)?,
			bump: state.bump,
		})
	}

	/// Require `account` to be this launch's base vault.
	pub fn assert_base_vault(&self, account: &AccountView) -> ProgramResult {
		if account.address() != &self.base_vault {
			return Err(CurveError::AccountMismatch.into());
		}
		Ok(())
	}

	/// Require `account` to be this launch's quote vault.
	pub fn assert_quote_vault(&self, account: &AccountView) -> ProgramResult {
		if account.address() != &self.quote_vault {
			return Err(CurveError::AccountMismatch.into());
		}
		Ok(())
	}
}

/// Run `body` with the launch's PDA signer.
pub(crate) fn with_launch_signer<R>(
	snapshot: &LaunchSnapshot,
	body: impl FnOnce(&Signer<'_, '_>) -> Result<R, ProgramError>,
) -> Result<R, ProgramError> {
	let seeds = Launch::seeds(&snapshot.base_mint).with_bump(snapshot.bump);
	let signer = seeds.to_signer();
	body(&signer.as_signer())
}
