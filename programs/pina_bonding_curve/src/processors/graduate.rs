//! `Graduate`: move a completed launch into a Pina AMM pool.

use pina::*;
use pina_amm_cpi::CreatePool;
use pina_amm_cpi::CreatePoolIx;
use pina_amm_cpi::ProgramAccount;

use super::common::ConfigSnapshot;
use super::common::LaunchSnapshot;
use super::common::with_launch_signer;
use crate::ID;
use crate::curve::seed_pool;
use crate::errors::CurveError;
use crate::events::Graduated;
use crate::instructions::GraduateInstruction;
use crate::math::share_floor;
use crate::math::sqrt_floor;
use crate::state::AMM_MINIMUM_LIQUIDITY;
use crate::state::AmmAuthority;
use crate::state::Launch;
use crate::state::LaunchStatus;
use crate::state::PoolCreatorMode;
use crate::token::vault_amount;

/// The Pina AMM's creator fee mode that pins the creator fee to token 0.
const AMM_CREATOR_FEE_TOKEN_0: u8 = 1;
/// The Pina AMM's creator fee mode that pins the creator fee to token 1.
const AMM_CREATOR_FEE_TOKEN_1: u8 = 2;

/// Accounts for `Graduate`.
///
/// Graduation is permissionless: anyone can pay to move a completed launch.
#[derive(Accounts, Debug)]
pub struct GraduateAccounts<'a> {
	/// Pays rent for the pool and any LP token accounts it creates.
	#[pina(validate(signer, writable))]
	pub payer: &'a AccountView,
	/// The launch's configuration.
	pub config: &'a AccountView,
	/// The completed launch.
	pub launch: &'a mut AccountView,
	/// The launch's base mint; written when surplus base is burned.
	pub base_mint: &'a mut AccountView,
	/// The launch's quote mint.
	pub quote_mint: &'a AccountView,
	/// The launch's base vault.
	pub base_vault: &'a mut AccountView,
	/// The launch's quote vault.
	pub quote_vault: &'a mut AccountView,
	/// This program's AMM authority PDA: `[b"amm_authority"]`.
	pub amm_authority: &'a AccountView,
	/// The Pina AMM program.
	pub amm_program: &'a AccountView,
	/// The configuration's Pina AMM fee tier.
	pub amm_config: &'a AccountView,
	/// The pool to create: `[b"pool", amm_config, mint_0, mint_1]` in the AMM.
	pub pool: &'a mut AccountView,
	/// The pool's LP mint to create.
	pub lp_mint: &'a mut AccountView,
	/// The pool's vault for the smaller of the two mints.
	pub pool_vault_0: &'a mut AccountView,
	/// The pool's vault for the larger of the two mints.
	pub pool_vault_1: &'a mut AccountView,
	/// The launch's associated LP token account, created by the AMM.
	pub launch_lp_token: &'a mut AccountView,
	/// Token program that owns the base mint.
	pub base_token_program: &'a AccountView,
	/// Token program that owns the quote mint.
	pub quote_token_program: &'a AccountView,
	/// SPL Token, which owns every Pina AMM LP mint.
	#[pina(validate(program = token::ID))]
	pub lp_token_program: &'a AccountView,
	/// The associated token account program.
	#[pina(validate(program = associated_token_account::ID))]
	pub associated_token_program: &'a AccountView,
	/// The system program.
	#[pina(validate(program = system::ID))]
	pub system_program: &'a AccountView,
	/// The launch creator. Required when the creator's LP share is not zero.
	pub creator: Option<&'a AccountView>,
	/// The creator's associated LP token account, created here when needed.
	pub creator_lp_token: Option<&'a mut AccountView>,
	/// The configuration's partner. Required when the partner's LP share is
	/// not zero.
	pub partner: Option<&'a AccountView>,
	/// The partner's associated LP token account, created here when needed.
	pub partner_lp_token: Option<&'a mut AccountView>,
}

/// What a graduation moves, computed before any CPI.
struct Plan {
	pool_base: u64,
	pool_quote: u64,
	migration_fee: u64,
	burned_base: u64,
}

impl GraduateAccounts<'_> {
	fn plan(&self, config: &ConfigSnapshot, launch: &LaunchSnapshot) -> Result<Plan, ProgramError> {
		let migration_fee = share_floor(launch.quote_reserve, config.migration_fee_rate)?;
		let pool_quote = launch
			.quote_reserve
			.checked_sub(migration_fee)
			.ok_or(CurveError::MathOverflow)?;
		let unvested = config
			.creator_allocation
			.checked_sub(launch.creator_claimed)
			.ok_or(CurveError::MathOverflow)?;
		let available_base = vault_amount(self.base_vault, self.base_token_program)?
			.checked_sub(unvested)
			.ok_or(CurveError::MathOverflow)?;
		let seed = seed_pool(available_base, pool_quote, launch.sqrt_price)?;
		if sqrt_floor(u128::from(seed.base) * u128::from(seed.quote))
			<= u128::from(AMM_MINIMUM_LIQUIDITY)
		{
			return Err(CurveError::InsufficientMigrationLiquidity.into());
		}
		Ok(Plan {
			pool_base: seed.base,
			pool_quote: seed.quote,
			migration_fee: migration_fee
				.checked_add(seed.quote_surplus)
				.ok_or(CurveError::MathOverflow)?,
			burned_base: seed.base_surplus,
		})
	}

	/// Send `amount` LP to `owner`'s associated token account, creating it.
	fn pay_lp(
		&self,
		launch: &LaunchSnapshot,
		owner: Option<&AccountView>,
		destination: Option<&AccountView>,
		expected_owner: &Address,
		amount: u64,
	) -> ProgramResult {
		if amount == 0 {
			return Ok(());
		}
		let (Some(owner), Some(destination)) = (owner, destination) else {
			return Err(CurveError::MissingLpAccount.into());
		};
		if owner.address() != expected_owner {
			return Err(CurveError::AccountMismatch.into());
		}
		associated_token_account::instructions::CreateIdempotent {
			funding_account: self.payer,
			account: destination,
			wallet: owner,
			mint: self.lp_mint,
			system_program: self.system_program,
			token_program: self.lp_token_program,
		}
		.invoke()?;
		with_launch_signer(launch, |signer| {
			token::instructions::Transfer::new(
				self.launch_lp_token,
				destination,
				self.launch,
				amount,
			)
			.invoke_signed(core::slice::from_ref(signer))
		})
	}
}

impl<'a> ProcessAccountInfos<'a> for GraduateAccounts<'a> {
	fn process(self, data: &[u8]) -> ProgramResult {
		GraduateInstruction::try_from_bytes(data)?;
		let config = ConfigSnapshot::load(self.config)?;
		let launch = LaunchSnapshot::load(self.launch, self.config)?;
		if launch.status != LaunchStatus::Completed {
			return Err(CurveError::NotCompleted.into());
		}
		// Only a completing buy sets `Completed`, and it stops exactly at the
		// derived price, so the two can never drift apart in practice. The
		// check keeps any future state bug from silently seeding the pool at a
		// stale price.
		if launch.sqrt_price != config.migration_sqrt_price {
			return Err(CurveError::MigrationPriceMismatch.into());
		}
		launch.assert_base_vault(self.base_vault)?;
		launch.assert_quote_vault(self.quote_vault)?;
		if self.base_mint.address() != &launch.base_mint
			|| self.quote_mint.address() != &launch.quote_mint
			|| self.amm_config.address() != &config.amm_config
		{
			return Err(CurveError::AccountMismatch.into());
		}
		let (amm_authority, amm_authority_bump) =
			AmmAuthority::try_find_pda(&ID).ok_or(CurveError::AccountMismatch)?;
		if self.amm_authority.address() != &amm_authority {
			return Err(CurveError::AccountMismatch.into());
		}
		let amm = ProgramAccount::try_new(self.amm_program)?;
		let plan = self.plan(&config, &launch)?;
		let creator_fee = share_floor(plan.migration_fee, config.creator_fee_share)?;
		let pool_creator = match PoolCreatorMode::from_u8(config.pool_creator_mode) {
			Some(PoolCreatorMode::Creator) => launch.creator,
			Some(PoolCreatorMode::Partner) => config.authority,
			None => return Err(CurveError::InvalidPoolCreatorMode.into()),
		};

		// Commit before any CPI: a failed CPI rolls everything back.
		{
			let mut state = self.launch.as_account_mut::<Launch>(&ID)?;
			state.status = LaunchStatus::Graduated as u8;
			state.pool = *self.pool.address();
			state.quote_reserve.set(0);
			state.creator_fees.set(
				launch
					.creator_fees
					.checked_add(creator_fee)
					.ok_or(CurveError::MathOverflow)?,
			);
			state.partner_fees.set(
				launch
					.partner_fees
					.checked_add(
						plan.migration_fee
							.checked_sub(creator_fee)
							.ok_or(CurveError::MathOverflow)?,
					)
					.ok_or(CurveError::MathOverflow)?,
			);
		}

		let base_is_0 = launch.base_mint.as_ref() < launch.quote_mint.as_ref();
		let (
			mint_0,
			mint_1,
			depositor_0,
			depositor_1,
			program_0,
			program_1,
			amount_0,
			amount_1,
			fee_mode,
		) = if base_is_0 {
			(
				&*self.base_mint,
				self.quote_mint,
				&*self.base_vault,
				&*self.quote_vault,
				self.base_token_program,
				self.quote_token_program,
				plan.pool_base,
				plan.pool_quote,
				AMM_CREATOR_FEE_TOKEN_1,
			)
		} else {
			(
				self.quote_mint,
				&*self.base_mint,
				&*self.quote_vault,
				&*self.base_vault,
				self.quote_token_program,
				self.base_token_program,
				plan.pool_quote,
				plan.pool_base,
				AMM_CREATOR_FEE_TOKEN_0,
			)
		};
		let launch_seeds = Launch::seeds(&launch.base_mint).with_bump(launch.bump);
		let launch_signer = launch_seeds.to_signer();
		let authority_seeds = AmmAuthority::seeds().with_bump(amm_authority_bump);
		let authority_signer = authority_seeds.to_signer();
		CreatePool {
			payer: self.payer,
			depositor: self.launch,
			pool_creator_authority: self.amm_authority,
			amm_config: self.amm_config,
			mint0: mint_0,
			mint1: mint_1,
			pool: self.pool,
			lp_mint: self.lp_mint,
			vault0: self.pool_vault_0,
			vault1: self.pool_vault_1,
			depositor_token0: depositor_0,
			depositor_token1: depositor_1,
			lp_owner: self.launch,
			lp_owner_token: self.launch_lp_token,
			token_program0: program_0,
			token_program1: program_1,
			lp_token_program: self.lp_token_program,
			associated_token_program: self.associated_token_program,
			system_program: self.system_program,
			ix: CreatePoolIx {
				amount0: amount_0,
				amount1: amount_1,
				creator: &pool_creator,
				creator_fee_mode: fee_mode,
			},
		}
		.invoke_signed(
			&amm,
			&[launch_signer.as_signer(), authority_signer.as_signer()],
		)?;

		// Split the LP the AMM minted to the launch. A partner who launches
		// under their own configuration is both recipients: the two payouts
		// would target the same associated token account, which the runtime
		// rejects as a duplicate writable account, so the shares merge into
		// one payment and the partner's accounts are omitted.
		let lp_total = self
			.launch_lp_token
			.as_token_account_for_program(&token::ID)?
			.amount();
		let (creator_lp, partner_lp) = if launch.creator == config.authority {
			let merged = share_floor(lp_total, config.creator_lp_share)?
				.checked_add(share_floor(lp_total, config.partner_lp_share)?)
				.ok_or(CurveError::MathOverflow)?;
			(merged, 0)
		} else {
			(
				share_floor(lp_total, config.creator_lp_share)?,
				share_floor(lp_total, config.partner_lp_share)?,
			)
		};
		let burned_lp = lp_total
			.checked_sub(creator_lp)
			.and_then(|value| value.checked_sub(partner_lp))
			.ok_or(CurveError::MathOverflow)?;
		self.pay_lp(
			&launch,
			self.creator,
			self.creator_lp_token.as_deref(),
			&launch.creator,
			creator_lp,
		)?;
		self.pay_lp(
			&launch,
			self.partner,
			self.partner_lp_token.as_deref(),
			&config.authority,
			partner_lp,
		)?;
		with_launch_signer(&launch, |signer| {
			if burned_lp > 0 {
				token::instructions::Burn::new(
					self.launch_lp_token,
					self.lp_mint,
					self.launch,
					burned_lp,
				)
				.invoke_signed(core::slice::from_ref(signer))?;
			}
			if plan.burned_base > 0 {
				token::instructions::Burn::new(
					self.base_vault,
					self.base_mint,
					self.launch,
					plan.burned_base,
				)
				.invoke_signed_with_program(
					core::slice::from_ref(signer),
					self.base_token_program.address(),
				)?;
			}
			Ok(())
		})?;

		Graduated::emit(|event| {
			event.launch = *self.launch.address();
			event.pool = *self.pool.address();
			event.pool_base.set(plan.pool_base);
			event.pool_quote.set(plan.pool_quote);
			event.migration_fee.set(plan.migration_fee);
			event.burned_base.set(plan.burned_base);
			event.burned_lp.set(burned_lp);
			event.creator_lp.set(creator_lp);
			event.partner_lp.set(partner_lp);
			Ok(())
		})
	}
}
