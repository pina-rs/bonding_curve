//! `SweepQuoteDust`.

use pina::*;

use super::common::ConfigSnapshot;
use super::common::LaunchSnapshot;
use crate::ID;
use crate::errors::CurveError;
use crate::events::DustSwept;
use crate::instructions::SweepQuoteDustInstruction;
use crate::math::share_floor;
use crate::state::Launch;
use crate::token::vault_amount;

/// Accounts for `SweepQuoteDust`.
///
/// A quote transfer straight to a launch's vault lands above the launch's
/// accounting: it backs no reserve and pays no fee. This instruction splits
/// that dust into the creator's and partner's fee balances by the
/// configuration's `creator_fee_share`, the same rule every other fee follows.
/// Permissionless, and safe in every status: the reserve and the accrued fees
/// are never touched.
#[derive(Accounts, Debug)]
pub struct SweepQuoteDustAccounts<'a> {
	/// The launch's configuration.
	pub config: &'a AccountView,
	/// The launch.
	pub launch: &'a mut AccountView,
	/// The launch's quote vault.
	pub quote_vault: &'a AccountView,
	/// Token program that owns the quote mint.
	pub quote_token_program: &'a AccountView,
}

impl<'a> ProcessAccountInfos<'a> for SweepQuoteDustAccounts<'a> {
	fn process(self, data: &[u8]) -> ProgramResult {
		SweepQuoteDustInstruction::try_from_bytes(data)?;
		let config = ConfigSnapshot::load(self.config)?;
		let launch = LaunchSnapshot::load(self.launch, self.config)?;
		launch.assert_quote_vault(self.quote_vault)?;
		let dust = vault_amount(self.quote_vault, self.quote_token_program)?
			.checked_sub(launch.quote_reserve)
			.and_then(|value| value.checked_sub(launch.creator_fees))
			.and_then(|value| value.checked_sub(launch.partner_fees))
			.ok_or(CurveError::MathOverflow)?;
		if dust == 0 {
			return Err(CurveError::NothingToClaim.into());
		}
		let creator_fee = share_floor(dust, config.creator_fee_share)?;
		let partner_fee = dust
			.checked_sub(creator_fee)
			.ok_or(CurveError::MathOverflow)?;
		{
			let mut state = self.launch.as_account_mut::<Launch>(&ID)?;
			state.creator_fees.set(
				launch
					.creator_fees
					.checked_add(creator_fee)
					.ok_or(CurveError::MathOverflow)?,
			);
			state.partner_fees.set(
				launch
					.partner_fees
					.checked_add(partner_fee)
					.ok_or(CurveError::MathOverflow)?,
			);
		}
		DustSwept::emit(|event| {
			event.launch = *self.launch.address();
			event.amount.set(dust);
			event.creator_fee.set(creator_fee);
			event.partner_fee.set(partner_fee);
			Ok(())
		})
	}
}
