//! `ClaimPartnerFees`, `ClaimCreatorFees`, and `ClaimCreatorAllocation`.

use pina::*;

use super::common::ConfigSnapshot;
use super::common::LaunchSnapshot;
use super::common::now;
use super::common::with_launch_signer;
use crate::ID;
use crate::errors::CurveError;
use crate::events::Claimed;
use crate::instructions::ClaimCreatorAllocationInstruction;
use crate::instructions::ClaimCreatorFeesInstruction;
use crate::instructions::ClaimPartnerFeesInstruction;
use crate::instructions::CurveInstruction;
use crate::math::vested_amount;
use crate::state::Launch;
use crate::token::transfer_signed;

/// `Claimed::kind` for partner fees.
const KIND_PARTNER_FEES: u8 = 0;
/// `Claimed::kind` for creator fees.
const KIND_CREATOR_FEES: u8 = 1;
/// `Claimed::kind` for the vested creator allocation.
const KIND_CREATOR_ALLOCATION: u8 = 2;

/// Accounts for `ClaimPartnerFees` and `ClaimCreatorFees`.
#[derive(Accounts, Debug)]
pub struct ClaimFeesAccounts<'a> {
	/// The configuration's authority for partner fees, or the launch's
	/// creator for creator fees.
	#[pina(validate(signer))]
	pub claimant: &'a AccountView,
	/// The launch's configuration.
	pub config: &'a AccountView,
	/// The launch.
	pub launch: &'a mut AccountView,
	/// The launch's quote vault.
	pub quote_vault: &'a mut AccountView,
	/// Quote token account that receives the fees.
	pub destination: &'a mut AccountView,
	/// Token program that owns the quote mint.
	pub quote_token_program: &'a AccountView,
}

/// Accounts for `ClaimCreatorAllocation`.
#[derive(Accounts, Debug)]
pub struct ClaimCreatorAllocationAccounts<'a> {
	/// The launch's creator.
	#[pina(validate(signer))]
	pub creator: &'a AccountView,
	/// The launch's configuration.
	pub config: &'a AccountView,
	/// The launch.
	pub launch: &'a mut AccountView,
	/// The launch's base vault.
	pub base_vault: &'a mut AccountView,
	/// Base token account that receives the allocation.
	pub destination: &'a mut AccountView,
	/// Token program that owns the base mint.
	pub base_token_program: &'a AccountView,
}

fn emit_claim(launch: &Address, claimant: &Address, kind: u8, amount: u64) -> ProgramResult {
	Claimed::emit(|event| {
		event.launch = *launch;
		event.claimant = *claimant;
		event.kind = kind;
		event.amount.set(amount);
		Ok(())
	})
}

impl<'a> ProcessAccountInfos<'a> for ClaimFeesAccounts<'a> {
	/// Routes to the partner or creator claim from the instruction's
	/// discriminator.
	fn process(self, data: &[u8]) -> ProgramResult {
		let partner = data.first().copied() == Some(CurveInstruction::ClaimPartnerFees as u8);
		if partner {
			ClaimPartnerFeesInstruction::try_from_bytes(data)?;
		} else {
			ClaimCreatorFeesInstruction::try_from_bytes(data)?;
		}
		let launch = LaunchSnapshot::load(self.launch, self.config)?;
		launch.assert_quote_vault(self.quote_vault)?;
		let (owner, amount) = if partner {
			(
				ConfigSnapshot::load(self.config)?.authority,
				launch.partner_fees,
			)
		} else {
			(launch.creator, launch.creator_fees)
		};
		if self.claimant.address() != &owner {
			return Err(CurveError::Unauthorized.into());
		}
		if amount == 0 {
			return Err(CurveError::NothingToClaim.into());
		}
		{
			let mut state = self.launch.as_account_mut::<Launch>(&ID)?;
			if partner {
				state.partner_fees.set(0);
			} else {
				state.creator_fees.set(0);
			}
		}
		with_launch_signer(&launch, |signer| {
			transfer_signed(
				self.quote_vault,
				self.destination,
				self.launch,
				self.quote_token_program,
				amount,
				signer,
			)
		})?;
		emit_claim(
			self.launch.address(),
			&owner,
			if partner {
				KIND_PARTNER_FEES
			} else {
				KIND_CREATOR_FEES
			},
			amount,
		)
	}
}

impl<'a> ProcessAccountInfos<'a> for ClaimCreatorAllocationAccounts<'a> {
	fn process(self, data: &[u8]) -> ProgramResult {
		ClaimCreatorAllocationInstruction::try_from_bytes(data)?;
		let config = ConfigSnapshot::load(self.config)?;
		let launch = LaunchSnapshot::load(self.launch, self.config)?;
		launch.assert_base_vault(self.base_vault)?;
		if self.creator.address() != &launch.creator {
			return Err(CurveError::Unauthorized.into());
		}
		let vested = vested_amount(
			config.creator_allocation,
			launch.activation_time,
			config.creator_vesting_cliff,
			config.creator_vesting_duration,
			now()?,
		)?;
		let amount = vested
			.checked_sub(launch.creator_claimed)
			.filter(|value| *value > 0)
			.ok_or(CurveError::NothingToClaim)?;
		self.launch
			.as_account_mut::<Launch>(&ID)?
			.creator_claimed
			.set(vested);
		with_launch_signer(&launch, |signer| {
			transfer_signed(
				self.base_vault,
				self.destination,
				self.launch,
				self.base_token_program,
				amount,
				signer,
			)
		})?;
		emit_claim(
			self.launch.address(),
			&launch.creator,
			KIND_CREATOR_ALLOCATION,
			amount,
		)
	}
}
