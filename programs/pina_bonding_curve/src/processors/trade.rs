//! `Buy` and `Sell`.

use pina::*;

use super::common::ConfigSnapshot;
use super::common::LaunchSnapshot;
use super::common::now;
use super::common::with_launch_signer;
use crate::ID;
use crate::curve;
use crate::errors::CurveError;
use crate::events::Completed;
use crate::events::Traded;
use crate::instructions::BuyInstruction;
use crate::instructions::CurveInstruction;
use crate::instructions::SellInstruction;
use crate::math::fee_ceil;
use crate::math::fee_rate_at;
use crate::math::gross_up;
use crate::math::share_floor;
use crate::state::Launch;
use crate::state::LaunchStatus;
use crate::token::transfer;
use crate::token::transfer_signed;

/// Accounts for `Buy` and `Sell`.
#[derive(Accounts, Debug)]
pub struct TradeAccounts<'a> {
	/// The trader, who owns the account the sold token comes from.
	#[pina(validate(signer))]
	pub trader: &'a AccountView,
	/// The launch's configuration.
	pub config: &'a AccountView,
	/// The launch.
	pub launch: &'a mut AccountView,
	/// The trader's base token account: credited on a buy, debited on a sale.
	pub trader_base: &'a mut AccountView,
	/// The trader's quote token account: debited on a buy, credited on a sale.
	pub trader_quote: &'a mut AccountView,
	/// The launch's base vault.
	pub base_vault: &'a mut AccountView,
	/// The launch's quote vault.
	pub quote_vault: &'a mut AccountView,
	/// Token program that owns the base mint.
	pub base_token_program: &'a AccountView,
	/// Token program that owns the quote mint.
	pub quote_token_program: &'a AccountView,
}

/// What one trade changes, applied to the launch before any transfer.
struct Settlement {
	sqrt_price: u128,
	quote_reserve: u64,
	creator_fee: u64,
	partner_fee: u64,
	completed: bool,
}

impl TradeAccounts<'_> {
	/// Load and check everything both directions need.
	fn prepare(&self) -> Result<(ConfigSnapshot, LaunchSnapshot, u32), ProgramError> {
		let config = ConfigSnapshot::load(self.config)?;
		let launch = LaunchSnapshot::load(self.launch, self.config)?;
		launch.assert_base_vault(self.base_vault)?;
		launch.assert_quote_vault(self.quote_vault)?;
		if launch.status != LaunchStatus::Trading {
			return Err(CurveError::NotTrading.into());
		}
		let current = now()?;
		if current < launch.activation_time {
			return Err(CurveError::NotActive.into());
		}
		let rate = fee_rate_at(
			config.start_fee_rate,
			config.end_fee_rate,
			config.fee_decay_duration,
			launch.activation_time,
			current,
		);
		Ok((config, launch, rate))
	}

	/// Commit a settlement to the launch account.
	fn settle(&mut self, launch: &LaunchSnapshot, settlement: &Settlement) -> ProgramResult {
		let mut state = self.launch.as_account_mut::<Launch>(&ID)?;
		state.sqrt_price.set(settlement.sqrt_price);
		state.quote_reserve.set(settlement.quote_reserve);
		state.creator_fees.set(
			launch
				.creator_fees
				.checked_add(settlement.creator_fee)
				.ok_or(CurveError::MathOverflow)?,
		);
		state.partner_fees.set(
			launch
				.partner_fees
				.checked_add(settlement.partner_fee)
				.ok_or(CurveError::MathOverflow)?,
		);
		if settlement.completed {
			state.status = LaunchStatus::Completed as u8;
		}
		Ok(())
	}

	fn buy(mut self, quote_amount_in: u64, minimum_base_out: u64) -> ProgramResult {
		let (config, launch, rate) = self.prepare()?;
		let fee = fee_ceil(quote_amount_in, rate)?;
		let net = quote_amount_in
			.checked_sub(fee)
			.ok_or(CurveError::ZeroAmount)?;
		let result = curve::buy(
			&config.curve,
			launch.sqrt_price,
			net,
			config.migration_sqrt_price,
		)?;
		if result.base_out == 0 {
			return Err(CurveError::ZeroAmount.into());
		}
		if result.base_out < minimum_base_out {
			return Err(CurveError::SlippageExceeded.into());
		}
		// A buy that completes the launch only pays for the quote it used.
		let (paid, fee) = if result.quote_used < net {
			let gross = gross_up(result.quote_used, rate)?.min(quote_amount_in);
			let fee = gross
				.checked_sub(result.quote_used)
				.ok_or(CurveError::MathOverflow)?;
			(gross, fee)
		} else {
			(quote_amount_in, fee)
		};
		let creator_fee = share_floor(fee, config.creator_fee_share)?;
		let settlement = Settlement {
			sqrt_price: result.sqrt_price,
			quote_reserve: launch
				.quote_reserve
				.checked_add(result.quote_used)
				.ok_or(CurveError::MathOverflow)?,
			creator_fee,
			partner_fee: fee
				.checked_sub(creator_fee)
				.ok_or(CurveError::MathOverflow)?,
			completed: result.completed,
		};
		self.settle(&launch, &settlement)?;

		transfer(
			self.trader_quote,
			self.quote_vault,
			self.trader,
			self.quote_token_program,
			paid,
		)?;
		with_launch_signer(&launch, |signer| {
			transfer_signed(
				self.base_vault,
				self.trader_base,
				self.launch,
				self.base_token_program,
				result.base_out,
				signer,
			)
		})?;

		self.emit(true, result.base_out, paid, fee, &settlement)?;
		if settlement.completed {
			Completed::emit(|event| {
				event.launch = *self.launch.address();
				event.quote_reserve.set(settlement.quote_reserve);
				Ok(())
			})?;
		}
		Ok(())
	}

	fn sell(mut self, base_amount_in: u64, minimum_quote_out: u64) -> ProgramResult {
		let (config, launch, rate) = self.prepare()?;
		let result = curve::sell(&config.curve, launch.sqrt_price, base_amount_in)?;
		let fee = fee_ceil(result.quote_out, rate)?;
		let received = result
			.quote_out
			.checked_sub(fee)
			.filter(|value| *value > 0)
			.ok_or(CurveError::ZeroAmount)?;
		if received < minimum_quote_out {
			return Err(CurveError::SlippageExceeded.into());
		}
		let creator_fee = share_floor(fee, config.creator_fee_share)?;
		let settlement = Settlement {
			sqrt_price: result.sqrt_price,
			quote_reserve: launch
				.quote_reserve
				.checked_sub(result.quote_out)
				.ok_or(CurveError::InsufficientLiquidity)?,
			creator_fee,
			partner_fee: fee
				.checked_sub(creator_fee)
				.ok_or(CurveError::MathOverflow)?,
			completed: false,
		};
		self.settle(&launch, &settlement)?;

		transfer(
			self.trader_base,
			self.base_vault,
			self.trader,
			self.base_token_program,
			base_amount_in,
		)?;
		with_launch_signer(&launch, |signer| {
			transfer_signed(
				self.quote_vault,
				self.trader_quote,
				self.launch,
				self.quote_token_program,
				received,
				signer,
			)
		})?;

		self.emit(false, base_amount_in, received, fee, &settlement)
	}

	fn emit(
		&self,
		is_buy: bool,
		base: u64,
		quote: u64,
		fee: u64,
		settlement: &Settlement,
	) -> ProgramResult {
		Traded::emit(|event| {
			event.launch = *self.launch.address();
			event.trader = *self.trader.address();
			event.is_buy = u8::from(is_buy);
			event.base_amount.set(base);
			event.quote_amount.set(quote);
			event.fee.set(fee);
			event.creator_fee.set(settlement.creator_fee);
			event.sqrt_price.set(settlement.sqrt_price);
			event.quote_reserve.set(settlement.quote_reserve);
			Ok(())
		})
	}
}

impl<'a> ProcessAccountInfos<'a> for TradeAccounts<'a> {
	/// Routes to the buy or sell handler from the instruction's discriminator.
	fn process(self, data: &[u8]) -> ProgramResult {
		if data.first().copied() == Some(CurveInstruction::Buy as u8) {
			let args = BuyInstruction::try_from_bytes(data)?;
			return self.buy(args.quote_amount_in.get(), args.minimum_base_out.get());
		}
		let args = SellInstruction::try_from_bytes(data)?;
		self.sell(args.base_amount_in.get(), args.minimum_quote_out.get())
	}
}
