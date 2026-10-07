//! Command implementations.

use pina_bonding_curve_client::accounts::LaunchConfig;
use pina_bonding_curve_client::addresses::PINA_AMM_ID;
use pina_bonding_curve_client::addresses::amm_authority;
use pina_bonding_curve_client::addresses::graduation_pool;
use pina_bonding_curve_client::addresses::launch_vault;
use pina_bonding_curve_client::events::Claimed;
use pina_bonding_curve_client::events::ConfigCreated;
use pina_bonding_curve_client::events::Graduated;
use pina_bonding_curve_client::events::Traded;
use pina_bonding_curve_client::instructions::Buy;
use pina_bonding_curve_client::instructions::BuyInstructionData;
use pina_bonding_curve_client::instructions::ClaimCreatorAllocation;
use pina_bonding_curve_client::instructions::ClaimCreatorAllocationInstructionData;
use pina_bonding_curve_client::instructions::ClaimCreatorFees;
use pina_bonding_curve_client::instructions::ClaimCreatorFeesInstructionData;
use pina_bonding_curve_client::instructions::ClaimPartnerFees;
use pina_bonding_curve_client::instructions::ClaimPartnerFeesInstructionData;
use pina_bonding_curve_client::instructions::CreateConfig;
use pina_bonding_curve_client::instructions::CreateLaunch;
use pina_bonding_curve_client::instructions::CreateLaunchInstructionData;
use pina_bonding_curve_client::instructions::Graduate;
use pina_bonding_curve_client::instructions::GraduateInstructionData;
use pina_bonding_curve_client::instructions::Sell;
use pina_bonding_curve_client::instructions::SellInstructionData;
use pina_bonding_curve_client::instructions::SetLaunchCreator;
use pina_bonding_curve_client::instructions::SetLaunchCreatorInstructionData;
use serde_json::json;
use solana_instruction::Instruction;
use solana_keypair::Keypair;
use solana_pubkey::Pubkey;
use solana_signer::Signer;

use crate::accounts::ConfigView;
use crate::accounts::LaunchView;
use crate::accounts::MINT_LEN;
use crate::accounts::NATIVE_MINT;
use crate::accounts::Status;
use crate::accounts::TOKEN_2022_PROGRAM;
use crate::accounts::TOKEN_PROGRAM;
use crate::accounts::associated_token_address;
use crate::accounts::create_account;
use crate::accounts::create_associated_token_account;
use crate::accounts::initialize_mint;
use crate::accounts::launch_address;
use crate::accounts::load_config;
use crate::accounts::load_launch;
use crate::accounts::token_balance;
use crate::accounts::token_program_of;
use crate::accounts::wrap_sol;
use crate::cli::BuyArgs;
use crate::cli::ClaimCommand;
use crate::cli::Command;
use crate::cli::ConfigCommand;
use crate::cli::CreateConfigArgs;
use crate::cli::CreateLaunchArgs;
use crate::cli::LaunchCommand;
use crate::cli::QuoteArgs;
use crate::cli::SellArgs;
use crate::cli::SetCreatorArgs;
use crate::context::Context;
use crate::design;
use crate::error::CliError;
use crate::output;
use crate::terms::Terms;

const BASIS_POINTS: u128 = 10_000;
const PARTS_PER_MILLION: f64 = 1_000_000.0;
const CONFIG_CREATED_DISCRIMINATOR: u8 = 1;
const TRADED_DISCRIMINATOR: u8 = 3;
const GRADUATED_DISCRIMINATOR: u8 = 5;
const CLAIMED_DISCRIMINATOR: u8 = 6;

/// Run one command.
pub fn run(context: &Context, command: Command) -> Result<(), CliError> {
	match command {
		Command::Design(command) => design::run(&command, context.json),
		Command::Config(ConfigCommand::Create(args)) => create_config(context, &args),
		Command::Config(ConfigCommand::Show(args)) => {
			print_config(context, &load_config(context, &args.config)?);
			Ok(())
		}
		Command::Launch(LaunchCommand::Create(args)) => create_launch(context, &args),
		Command::Launch(LaunchCommand::Show(args)) => show_launch(context, &args.base_mint),
		Command::Launch(LaunchCommand::SetCreator(args)) => set_creator(context, &args),
		Command::Quote(args) => quote(context, &args),
		Command::Buy(args) => buy(context, &args),
		Command::Sell(args) => sell(context, &args),
		Command::Graduate(args) => graduate(context, &args.base_mint),
		Command::Claim(command) => claim(context, &command),
	}
}

/// Smallest acceptable output after `slippage_bps` of tolerance.
pub fn minimum_after_slippage(amount: u64, slippage_bps: u16) -> Result<u64, CliError> {
	let remaining = BASIS_POINTS
		.checked_sub(u128::from(slippage_bps))
		.ok_or(CliError::InvalidSlippage(slippage_bps))?;
	Ok(u64::try_from(u128::from(amount) * remaining / BASIS_POINTS).unwrap_or(u64::MAX))
}

fn report(context: &Context, action: &str, signature: Option<String>, details: &serde_json::Value) {
	if context.json {
		println!(
			"{}",
			json!({ "action": action, "signature": signature, "details": details })
		);
		return;
	}
	match signature {
		Some(signature) => println!("{action}: {signature}"),
		None => println!("{action}: simulated"),
	}
	output::print(details, false, "  ");
}

/// Simulate `instructions`, return the bytes of the first event with
/// `discriminator` and the compute units used, then send the same
/// instructions unless `--simulate` was given.
fn simulate_then_send(
	context: &Context,
	instructions: &[Instruction],
	extra_signers: &[&Keypair],
	discriminator: u8,
) -> Result<(Vec<u8>, u64, Option<String>), CliError> {
	let simulation = context.simulate(instructions, extra_signers)?;
	let event = simulation
		.events
		.into_iter()
		.find(|bytes| bytes.first() == Some(&discriminator))
		.ok_or(CliError::MissingEvent)?;
	let signature = if context.simulate_only {
		None
	} else {
		context.send(instructions, extra_signers)?
	};
	Ok((event, simulation.units, signature))
}

fn create_config(context: &Context, args: &CreateConfigArgs) -> Result<(), CliError> {
	let terms = Terms::load(&args.terms)?;
	let signer = context.signer();
	let address = LaunchConfig::find_pda(&signer, args.index).0;
	let quote_program = token_program_of(context, &args.quote_mint)?;
	let instruction = CreateConfig::new(
		signer,
		signer,
		address,
		args.quote_mint,
		quote_program,
		args.amm_config,
	)
	.instruction(terms.instruction_data(args.index)?);
	let (event, units, signature) =
		simulate_then_send(context, &[instruction], &[], CONFIG_CREATED_DISCRIMINATOR)?;
	let created = ConfigCreated::from_bytes(&event).map_err(|_| CliError::MissingEvent)?;
	let migration_sqrt_price = created.migration_sqrt_price.get();
	report(
		context,
		"config created",
		signature,
		&json!({
			"config": address.to_string(),
			"sale_supply": created.sale_supply.get(),
			"migration_supply": created.migration_supply.get(),
			"migration_sqrt_price": migration_sqrt_price.to_string(),
			"migration_price": design::price(migration_sqrt_price),
			"compute_units": units,
		}),
	);
	Ok(())
}

fn create_launch(context: &Context, args: &CreateLaunchArgs) -> Result<(), CliError> {
	let config = load_config(context, &args.config)?;
	let signer = context.signer();
	let quote_program = token_program_of(context, &config.quote_mint)?;
	let mut instructions = Vec::new();
	let new_mint = args.base_mint.is_none().then(Keypair::new);
	let (base_mint, base_program) = match (&new_mint, args.base_mint) {
		(Some(mint), _) => {
			let program = if args.token_2022 {
				TOKEN_2022_PROGRAM
			} else {
				TOKEN_PROGRAM
			};
			let lamports = context.minimum_balance(MINT_LEN)?;
			instructions.push(create_account(
				&signer,
				&mint.pubkey(),
				lamports,
				MINT_LEN,
				&program,
			));
			instructions.push(initialize_mint(
				&program,
				&mint.pubkey(),
				&signer,
				config.base_decimals,
			));
			(mint.pubkey(), program)
		}
		(None, Some(mint)) => (mint, token_program_of(context, &mint)?),
		(None, None) => unreachable!("a new mint is generated when none is given"),
	};
	let launch = launch_address(&base_mint);
	let data =
		CreateLaunchInstructionData::new(|data| data.activation_time.set(args.activation_time))
			.map_err(|_| CliError::InvalidInstructionData)?;
	instructions.push(
		CreateLaunch::new(
			signer,
			signer,
			config.address,
			base_mint,
			config.quote_mint,
			launch_vault(&launch, &base_mint),
			launch_vault(&launch, &config.quote_mint),
			base_program,
			quote_program,
		)
		.instruction(data),
	);
	let extra_signers: Vec<&Keypair> = new_mint.iter().collect();
	let signature = context.send(&instructions, &extra_signers)?;
	report(
		context,
		"launch created",
		signature,
		&json!({
			"launch": launch.to_string(),
			"base_mint": base_mint.to_string(),
			"total_supply": config.total_supply,
		}),
	);
	Ok(())
}

fn set_creator(context: &Context, args: &SetCreatorArgs) -> Result<(), CliError> {
	let launch = launch_address(&args.base_mint);
	let data = SetLaunchCreatorInstructionData::new(|data| data.new_creator = args.new_creator)
		.map_err(|_| CliError::InvalidInstructionData)?;
	let signature = context.send(
		&[SetLaunchCreator::new(context.signer(), launch).instruction(data)],
		&[],
	)?;
	report(
		context,
		"creator updated",
		signature,
		&json!({ "launch": launch.to_string(), "creator": args.new_creator.to_string() }),
	);
	Ok(())
}

/// Everything one trade needs.
struct Market {
	launch: LaunchView,
	base_program: Pubkey,
	quote_program: Pubkey,
	trader_base: Pubkey,
	trader_quote: Pubkey,
}

impl Market {
	fn load(context: &Context, base_mint: &Pubkey) -> Result<Self, CliError> {
		let launch = load_launch(context, base_mint)?;
		let base_program = token_program_of(context, &launch.base_mint)?;
		let quote_program = token_program_of(context, &launch.quote_mint)?;
		let trader = context.signer();
		Ok(Self {
			trader_base: associated_token_address(&trader, &launch.base_mint, &base_program),
			trader_quote: associated_token_address(&trader, &launch.quote_mint, &quote_program),
			launch,
			base_program,
			quote_program,
		})
	}

	/// Instructions that create the trader's token accounts and, for a buy
	/// with wrapped SOL, wrap the amount being spent.
	fn prefix(&self, trader: &Pubkey, wrap: u64) -> Vec<Instruction> {
		let mut instructions = vec![
			create_associated_token_account(
				trader,
				trader,
				&self.launch.base_mint,
				&self.base_program,
			),
			create_associated_token_account(
				trader,
				trader,
				&self.launch.quote_mint,
				&self.quote_program,
			),
		];
		if wrap > 0 && self.launch.quote_mint == NATIVE_MINT {
			instructions.extend(wrap_sol(trader, &self.trader_quote, wrap));
		}
		instructions
	}

	fn buy(
		&self,
		trader: &Pubkey,
		quote_in: u64,
		minimum_out: u64,
	) -> Result<Instruction, CliError> {
		let data = BuyInstructionData::new(|data| {
			data.quote_amount_in.set(quote_in);
			data.minimum_base_out.set(minimum_out);
		})
		.map_err(|_| CliError::InvalidInstructionData)?;
		Ok(Buy::new(
			*trader,
			self.launch.config,
			self.launch.address,
			self.trader_base,
			self.trader_quote,
			self.launch.base_vault,
			self.launch.quote_vault,
			self.base_program,
			self.quote_program,
		)
		.instruction(data))
	}

	fn sell(
		&self,
		trader: &Pubkey,
		base_in: u64,
		minimum_out: u64,
	) -> Result<Instruction, CliError> {
		let data = SellInstructionData::new(|data| {
			data.base_amount_in.set(base_in);
			data.minimum_quote_out.set(minimum_out);
		})
		.map_err(|_| CliError::InvalidInstructionData)?;
		Ok(Sell::new(
			*trader,
			self.launch.config,
			self.launch.address,
			self.trader_base,
			self.trader_quote,
			self.launch.base_vault,
			self.launch.quote_vault,
			self.base_program,
			self.quote_program,
		)
		.instruction(data))
	}
}

/// A simulated trade.
struct TradeQuote {
	base_amount: u64,
	quote_amount: u64,
	fee: u64,
	sqrt_price: u128,
	units: u64,
}

impl TradeQuote {
	fn details(&self, launch: &LaunchView, is_buy: bool) -> serde_json::Value {
		json!({
			"launch": launch.address.to_string(),
			"side": if is_buy { "buy" } else { "sell" },
			"base_amount": self.base_amount,
			"quote_amount": self.quote_amount,
			"fee": self.fee,
			"price_after": design::price(self.sqrt_price),
			"compute_units": self.units,
		})
	}
}

/// Simulate a trade with an open limit and read the `Traded` event.
fn simulate_trade(
	context: &Context,
	market: &Market,
	is_buy: bool,
	amount: u64,
) -> Result<TradeQuote, CliError> {
	let trader = context.signer();
	let mut instructions = market.prefix(&trader, if is_buy { amount } else { 0 });
	instructions.push(if is_buy {
		market.buy(&trader, amount, 0)?
	} else {
		market.sell(&trader, amount, 0)?
	});
	let simulation = context.simulate(&instructions, &[])?;
	let event = simulation
		.events
		.iter()
		.find(|bytes| bytes.first() == Some(&TRADED_DISCRIMINATOR))
		.ok_or(CliError::MissingEvent)?;
	let record = Traded::from_bytes(event).map_err(|_| CliError::MissingEvent)?;
	Ok(TradeQuote {
		base_amount: record.base_amount.get(),
		quote_amount: record.quote_amount.get(),
		fee: record.fee.get(),
		sqrt_price: record.sqrt_price.get(),
		units: simulation.units,
	})
}

fn quote(context: &Context, args: &QuoteArgs) -> Result<(), CliError> {
	let market = Market::load(context, &args.base_mint)?;
	let (is_buy, amount) = match (args.buy, args.sell) {
		(Some(amount), _) => (true, amount),
		(None, Some(amount)) => (false, amount),
		(None, None) => unreachable!("clap requires one side"),
	};
	let quote = simulate_trade(context, &market, is_buy, amount)?;
	report(
		context,
		"quote",
		None,
		&quote.details(&market.launch, is_buy),
	);
	Ok(())
}

fn buy(context: &Context, args: &BuyArgs) -> Result<(), CliError> {
	let market = Market::load(context, &args.base_mint)?;
	let quote = simulate_trade(context, &market, true, args.quote_amount)?;
	let trader = context.signer();
	let mut instructions = market.prefix(&trader, args.quote_amount);
	instructions.push(market.buy(
		&trader,
		args.quote_amount,
		minimum_after_slippage(quote.base_amount, args.slippage_bps)?,
	)?);
	let signature = context.send(&instructions, &[])?;
	report(
		context,
		"bought",
		signature,
		&quote.details(&market.launch, true),
	);
	Ok(())
}

fn sell(context: &Context, args: &SellArgs) -> Result<(), CliError> {
	let market = Market::load(context, &args.base_mint)?;
	let quote = simulate_trade(context, &market, false, args.base_amount)?;
	let trader = context.signer();
	let mut instructions = market.prefix(&trader, 0);
	instructions.push(market.sell(
		&trader,
		args.base_amount,
		minimum_after_slippage(quote.quote_amount, args.slippage_bps)?,
	)?);
	let signature = context.send(&instructions, &[])?;
	report(
		context,
		"sold",
		signature,
		&quote.details(&market.launch, false),
	);
	Ok(())
}

fn graduate(context: &Context, base_mint: &Pubkey) -> Result<(), CliError> {
	let launch = load_launch(context, base_mint)?;
	let config = load_config(context, &launch.config)?;
	let pool = graduation_pool(&config.amm_config, &launch.base_mint, &launch.quote_mint);
	let mut accounts = Graduate::new(
		context.signer(),
		launch.config,
		launch.address,
		launch.base_mint,
		launch.quote_mint,
		launch.base_vault,
		launch.quote_vault,
		amm_authority(),
		PINA_AMM_ID,
		config.amm_config,
		pool.pool,
		pool.lp_mint,
		pool.vault_0,
		pool.vault_1,
		associated_token_address(&launch.address, &pool.lp_mint, &TOKEN_PROGRAM),
		token_program_of(context, &launch.base_mint)?,
		token_program_of(context, &launch.quote_mint)?,
	);
	if config.creator_lp_share > 0 {
		accounts.creator = Some(launch.creator);
		accounts.creator_lp_token = Some(associated_token_address(
			&launch.creator,
			&pool.lp_mint,
			&TOKEN_PROGRAM,
		));
	}
	if config.partner_lp_share > 0 {
		accounts.partner = Some(config.authority);
		accounts.partner_lp_token = Some(associated_token_address(
			&config.authority,
			&pool.lp_mint,
			&TOKEN_PROGRAM,
		));
	}
	let data =
		GraduateInstructionData::new(|_| {}).map_err(|_| CliError::InvalidInstructionData)?;
	let (event, units, signature) = simulate_then_send(
		context,
		&[accounts.instruction(data)],
		&[],
		GRADUATED_DISCRIMINATOR,
	)?;
	let graduated = Graduated::from_bytes(&event).map_err(|_| CliError::MissingEvent)?;
	report(
		context,
		"graduated",
		signature,
		&json!({
			"launch": launch.address.to_string(),
			"pool": pool.pool.to_string(),
			"lp_mint": pool.lp_mint.to_string(),
			"pool_base": graduated.pool_base.get(),
			"pool_quote": graduated.pool_quote.get(),
			"migration_fee": graduated.migration_fee.get(),
			"burned_base": graduated.burned_base.get(),
			"burned_lp": graduated.burned_lp.get(),
			"creator_lp": graduated.creator_lp.get(),
			"partner_lp": graduated.partner_lp.get(),
			"compute_units": units,
		}),
	);
	Ok(())
}

fn claim(context: &Context, command: &ClaimCommand) -> Result<(), CliError> {
	let (base_mint, action) = match command {
		ClaimCommand::PartnerFees(args) => (args.base_mint, "partner fees claimed"),
		ClaimCommand::CreatorFees(args) => (args.base_mint, "creator fees claimed"),
		ClaimCommand::Allocation(args) => (args.base_mint, "allocation claimed"),
	};
	let launch = load_launch(context, &base_mint)?;
	let signer = context.signer();
	let instructions = match command {
		ClaimCommand::PartnerFees(_) | ClaimCommand::CreatorFees(_) => {
			let program = token_program_of(context, &launch.quote_mint)?;
			let destination = associated_token_address(&signer, &launch.quote_mint, &program);
			let prefix =
				create_associated_token_account(&signer, &signer, &launch.quote_mint, &program);
			let claim_instruction = if matches!(command, ClaimCommand::PartnerFees(_)) {
				ClaimPartnerFees::new(
					signer,
					launch.config,
					launch.address,
					launch.quote_vault,
					destination,
					program,
				)
				.instruction(
					ClaimPartnerFeesInstructionData::new(|_| {})
						.map_err(|_| CliError::InvalidInstructionData)?,
				)
			} else {
				ClaimCreatorFees::new(
					signer,
					launch.config,
					launch.address,
					launch.quote_vault,
					destination,
					program,
				)
				.instruction(
					ClaimCreatorFeesInstructionData::new(|_| {})
						.map_err(|_| CliError::InvalidInstructionData)?,
				)
			};
			[prefix, claim_instruction]
		}
		ClaimCommand::Allocation(_) => {
			let program = token_program_of(context, &launch.base_mint)?;
			let destination = associated_token_address(&signer, &launch.base_mint, &program);
			[
				create_associated_token_account(&signer, &signer, &launch.base_mint, &program),
				ClaimCreatorAllocation::new(
					signer,
					launch.config,
					launch.address,
					launch.base_vault,
					destination,
					program,
				)
				.instruction(
					ClaimCreatorAllocationInstructionData::new(|_| {})
						.map_err(|_| CliError::InvalidInstructionData)?,
				),
			]
		}
	};
	let (event, _, signature) =
		simulate_then_send(context, &instructions, &[], CLAIMED_DISCRIMINATOR)?;
	let claimed = Claimed::from_bytes(&event).map_err(|_| CliError::MissingEvent)?;
	report(
		context,
		action,
		signature,
		&json!({ "launch": launch.address.to_string(), "amount": claimed.amount.get() }),
	);
	Ok(())
}

fn show_launch(context: &Context, base_mint: &Pubkey) -> Result<(), CliError> {
	let launch = load_launch(context, base_mint)?;
	let config = load_config(context, &launch.config)?;
	let progress = if config.migration_quote_threshold == 0 {
		1.0
	} else {
		launch.quote_reserve as f64 / config.migration_quote_threshold as f64
	};
	let value = json!({
		"address": launch.address.to_string(),
		"config": launch.config.to_string(),
		"creator": launch.creator.to_string(),
		"base_mint": launch.base_mint.to_string(),
		"quote_mint": launch.quote_mint.to_string(),
		"status": launch.status.name(),
		"price": design::price(launch.sqrt_price),
		"sqrt_price": launch.sqrt_price.to_string(),
		"quote_reserve": launch.quote_reserve,
		"migration_quote_threshold": config.migration_quote_threshold,
		"progress": if launch.status == Status::Trading { progress.min(1.0) } else { 1.0 },
		"unsold_base": token_balance(context, &launch.base_vault)?,
		"partner_fees": launch.partner_fees,
		"creator_fees": launch.creator_fees,
		"creator_allocation_claimed": launch.creator_claimed,
		"activation_time": launch.activation_time,
		"pool": (launch.status == Status::Graduated).then(|| launch.pool.to_string()),
	});
	print_value(context, &value);
	Ok(())
}

fn print_config(context: &Context, config: &ConfigView) {
	let rate = |value: u32| f64::from(value) / PARTS_PER_MILLION;
	let segments: Vec<serde_json::Value> = config
		.segment_prices
		.iter()
		.zip(&config.segment_liquidities)
		.map(|(price, liquidity)| {
			json!({
				"sqrt_price": price.to_string(),
				"price": design::price(*price),
				"liquidity": liquidity.to_string(),
			})
		})
		.collect();
	let value = json!({
		"address": config.address.to_string(),
		"index": config.index,
		"authority": config.authority.to_string(),
		"quote_mint": config.quote_mint.to_string(),
		"amm_config": config.amm_config.to_string(),
		"start_price": design::price(config.sqrt_start_price),
		"migration_price": design::price(config.migration_sqrt_price),
		"segments": segments,
		"total_supply": config.total_supply,
		"sale_supply": config.sale_supply,
		"migration_supply": config.migration_supply,
		"creator_allocation": config.creator_allocation,
		"migration_quote_threshold": config.migration_quote_threshold,
		"creator_vesting_cliff": config.creator_vesting_cliff,
		"creator_vesting_duration": config.creator_vesting_duration,
		"start_fee": rate(config.start_fee_rate),
		"end_fee": rate(config.end_fee_rate),
		"fee_decay_duration": config.fee_decay_duration,
		"creator_fee_share": rate(config.creator_fee_share),
		"migration_fee": rate(config.migration_fee_rate),
		"creator_lp_share": rate(config.creator_lp_share),
		"partner_lp_share": rate(config.partner_lp_share),
		"base_decimals": config.base_decimals,
		"pool_creator": if config.pool_creator_mode == 1 { "partner" } else { "creator" },
	});
	print_value(context, &value);
}

fn print_value(context: &Context, value: &serde_json::Value) {
	output::print(value, context.json, "");
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn slippage_limits_round_against_the_trader() {
		assert_eq!(minimum_after_slippage(10_000, 100).expect("min"), 9_900);
		assert_eq!(minimum_after_slippage(9_999, 100).expect("min"), 9_899);
		assert_eq!(minimum_after_slippage(10_000, 10_000).expect("min"), 0);
		assert!(matches!(
			minimum_after_slippage(1, 10_001),
			Err(CliError::InvalidSlippage(10_001))
		));
	}
}
