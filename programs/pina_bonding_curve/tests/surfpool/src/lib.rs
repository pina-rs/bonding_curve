#![cfg(test)]

//! End-to-end tests for the Pina Bonding Curve.
//!
//! Every test starts an isolated, offline Surfnet with the compiled bonding
//! curve and the compiled Pina AMM (at the revision pinned in `amm.rev`), and
//! drives the curve only through the generated Rust client. Graduation
//! therefore exercises the real cross-program call into the AMM.
//!
//! Run with `devenv shell test:surfpool`.

mod cli;
mod harness;
mod journeys;

use harness::AMM_PROGRAM;
use harness::Harness;
use harness::MintExtension;
use harness::SYSTEM_PROGRAM;
use harness::TOKEN_2022_PROGRAM;
use harness::TOKEN_PROGRAM;
use harness::ata;
use pina_bonding_curve_client::PINA_BONDING_CURVE_ID;
use pina_bonding_curve_client::accounts::Launch;
use pina_bonding_curve_client::accounts::LaunchConfig;
use pina_bonding_curve_client::instructions::Buy;
use pina_bonding_curve_client::instructions::BuyInstructionData;
use pina_bonding_curve_client::instructions::ClaimCreatorAllocation;
use pina_bonding_curve_client::instructions::ClaimCreatorAllocationInstructionData;
use pina_bonding_curve_client::instructions::ClaimCreatorFees;
use pina_bonding_curve_client::instructions::ClaimCreatorFeesInstructionData;
use pina_bonding_curve_client::instructions::ClaimPartnerFees;
use pina_bonding_curve_client::instructions::ClaimPartnerFeesInstructionData;
use pina_bonding_curve_client::instructions::CreateConfig;
use pina_bonding_curve_client::instructions::CreateConfigInstructionData;
use pina_bonding_curve_client::instructions::CreateLaunch;
use pina_bonding_curve_client::instructions::CreateLaunchInstructionData;
use pina_bonding_curve_client::instructions::Graduate;
use pina_bonding_curve_client::instructions::GraduateInstructionData;
use pina_bonding_curve_client::instructions::Sell;
use pina_bonding_curve_client::instructions::SellInstructionData;
use pina_bonding_curve_client::instructions::SetLaunchCreator;
use pina_bonding_curve_client::instructions::SetLaunchCreatorInstructionData;
use pina_test::AccountMeta;
use pina_test::Instruction;
use pina_test::Keypair;
use pina_test::Pubkey;
use pina_test::Signer;

/// Square-root price (Q64.64) of 2.8e-5 quote units per base unit: a billion
/// six-decimal tokens valued at about 28 units of a nine-decimal quote token.
const SQRT_START_PRICE: u128 = 97_610_000_000_000_000;
const LIQUIDITY: u128 = 5_000_000_000_000;
const THRESHOLD: u64 = 85_000_000_000;
const TOTAL_SUPPLY: u64 = 1_000_000_000_000_000;
const CREATOR_ALLOCATION: u64 = 50_000_000_000_000;
const VESTING_SECONDS: u64 = 3_600;
const START_FEE_RATE: u32 = 500_000;
const END_FEE_RATE: u32 = 10_000;
const FEE_DECAY_SECONDS: u64 = 60;
const CREATOR_FEE_SHARE: u32 = 500_000;
const MIGRATION_FEE_RATE: u32 = 20_000;
const AMM_TIER: u16 = 100;
const DENOMINATOR: u128 = 1_000_000;

/// Program error codes, mirrored from `PinaBondingCurveError`.
mod code {
	pub const INVALID_FEE_RATES: u32 = 1;
	pub const INVALID_SUPPLY: u32 = 2;
	pub const INVALID_MIGRATION_THRESHOLD: u32 = 3;
	pub const INVALID_AMM_CONFIG: u32 = 6;
	pub const UNSUPPORTED_MINT: u32 = 7;
	pub const INVALID_BASE_MINT: u32 = 8;
	pub const UNAUTHORIZED: u32 = 9;
	pub const NOT_TRADING: u32 = 11;
	pub const NOT_ACTIVE: u32 = 12;
	pub const NOT_COMPLETED: u32 = 13;
	pub const SLIPPAGE_EXCEEDED: u32 = 14;
	pub const NOTHING_TO_CLAIM: u32 = 20;
	pub const MISSING_LP_ACCOUNT: u32 = 21;
}

/// The economic terms a test configuration uses.
#[derive(Clone, Copy)]
struct Terms {
	threshold: u64,
	total_supply: u64,
	start_fee_rate: u32,
	fee_decay: u64,
	creator_lp_share: u32,
	partner_lp_share: u32,
	pool_creator_mode: u8,
	/// Overrides the creator-vesting duration; `None` keeps the suite default.
	vesting_seconds: Option<u64>,
}

impl Default for Terms {
	fn default() -> Self {
		Self {
			threshold: THRESHOLD,
			total_supply: TOTAL_SUPPLY,
			start_fee_rate: START_FEE_RATE,
			fee_decay: FEE_DECAY_SECONDS,
			creator_lp_share: 0,
			partner_lp_share: 0,
			pool_creator_mode: 0,
			vesting_seconds: None,
		}
	}
}

/// Pina AMM accounts shared by every test.
struct Amm {
	tier: Pubkey,
}

fn amm_authority() -> Pubkey {
	Pubkey::find_program_address(&[b"amm_authority"], &PINA_BONDING_CURVE_ID).0
}

/// Create a Pina AMM fee tier through its raw instruction layout:
/// `[0, index u16, trade u32, protocol u32, creator u32, authority, pool_creator_authority]`.
fn create_amm_tier(h: &Harness, index: u16, pool_creator_authority: &Pubkey) -> Pubkey {
	let admin = h.funded_keypair().expect("amm admin");
	h.set_upgrade_authority(&AMM_PROGRAM, &admin.pubkey())
		.expect("amm authority");
	let tier = Pubkey::find_program_address(&[b"amm_config", &index.to_le_bytes()], &AMM_PROGRAM).0;
	let mut data = vec![0u8];
	data.extend_from_slice(&index.to_le_bytes());
	data.extend_from_slice(&2_500u32.to_le_bytes());
	data.extend_from_slice(&160_000u32.to_le_bytes());
	data.extend_from_slice(&5_000u32.to_le_bytes());
	data.extend_from_slice(admin.pubkey().as_ref());
	data.extend_from_slice(pool_creator_authority.as_ref());
	let instruction = Instruction::new_with_bytes(
		AMM_PROGRAM,
		&data,
		vec![
			AccountMeta::new(h.payer().pubkey(), true),
			AccountMeta::new_readonly(admin.pubkey(), true),
			AccountMeta::new_readonly(Harness::program_data(&AMM_PROGRAM), false),
			AccountMeta::new(tier, false),
			AccountMeta::new_readonly(SYSTEM_PROGRAM, false),
		],
	);
	h.send(&[instruction], &[&admin]).expect("create amm tier");
	tier
}

fn setup_amm(h: &Harness) -> Amm {
	Amm {
		tier: create_amm_tier(h, AMM_TIER, &amm_authority()),
	}
}

/// A partner's configuration and the quote mint it raises.
struct Config {
	partner: Keypair,
	address: Pubkey,
	quote_mint: Pubkey,
	amm_tier: Pubkey,
}

fn create_config_instruction(
	h: &Harness,
	partner: &Keypair,
	index: u64,
	quote_mint: &Pubkey,
	amm_tier: &Pubkey,
	terms: Terms,
) -> Instruction {
	let mut uppers = [0u128; 16];
	let mut liquidities = [0u128; 16];
	uppers[0] = SQRT_START_PRICE * 8;
	liquidities[0] = LIQUIDITY;
	let data = CreateConfigInstructionData::new(|data| {
		data.index.set(index);
		data.sqrt_start_price.set(SQRT_START_PRICE);
		for slot in 0..16 {
			data.curve_sqrt_prices[slot].set(uppers[slot]);
			data.curve_liquidities[slot].set(liquidities[slot]);
		}
		data.migration_quote_threshold.set(terms.threshold);
		data.total_supply.set(terms.total_supply);
		data.creator_allocation.set(CREATOR_ALLOCATION);
		data.creator_vesting_cliff.set(0);
		data.creator_vesting_duration
			.set(terms.vesting_seconds.unwrap_or(VESTING_SECONDS));
		data.fee_decay_duration.set(terms.fee_decay);
		data.start_fee_rate.set(terms.start_fee_rate);
		data.end_fee_rate.set(END_FEE_RATE);
		data.creator_fee_share.set(CREATOR_FEE_SHARE);
		data.migration_fee_rate.set(MIGRATION_FEE_RATE);
		data.creator_lp_share.set(terms.creator_lp_share);
		data.partner_lp_share.set(terms.partner_lp_share);
		data.segment_count = 1;
		data.base_decimals = 6;
		data.pool_creator_mode = terms.pool_creator_mode;
	})
	.expect("config data");
	CreateConfig::new(
		h.payer().pubkey(),
		partner.pubkey(),
		LaunchConfig::find_pda(&partner.pubkey(), index).0,
		*quote_mint,
		TOKEN_PROGRAM,
		*amm_tier,
	)
	.instruction(data)
}

fn create_config(h: &Harness, amm: &Amm, terms: Terms) -> Config {
	let partner = h.funded_keypair().expect("partner");
	let quote_mint = h.create_mint(&TOKEN_PROGRAM, 9, &[]).expect("quote mint");
	h.send(
		&[create_config_instruction(
			h,
			&partner,
			0,
			&quote_mint,
			&amm.tier,
			terms,
		)],
		&[&partner],
	)
	.expect("create config");
	Config {
		address: LaunchConfig::find_pda(&partner.pubkey(), 0).0,
		partner,
		quote_mint,
		amm_tier: amm.tier,
	}
}

/// One launch and the accounts tests touch.
struct LaunchFixture {
	creator: Keypair,
	base_mint: Pubkey,
	launch: Pubkey,
	base_vault: Pubkey,
	quote_vault: Pubkey,
}

fn vault(launch: &Pubkey, mint: &Pubkey) -> Pubkey {
	Pubkey::find_program_address(
		&[b"launch_vault", launch.as_ref(), mint.as_ref()],
		&PINA_BONDING_CURVE_ID,
	)
	.0
}

fn create_launch_instruction(
	h: &Harness,
	config: &Config,
	creator: &Pubkey,
	base_mint: &Pubkey,
	activation_time: u64,
) -> Instruction {
	let launch = Launch::find_pda(base_mint).0;
	CreateLaunch::new(
		h.payer().pubkey(),
		*creator,
		config.address,
		*base_mint,
		config.quote_mint,
		vault(&launch, base_mint),
		vault(&launch, &config.quote_mint),
		TOKEN_2022_PROGRAM,
		TOKEN_PROGRAM,
	)
	.instruction(
		CreateLaunchInstructionData::new(|data| data.activation_time.set(activation_time))
			.expect("launch data"),
	)
}

fn create_launch_at(h: &Harness, config: &Config, activation_time: u64) -> LaunchFixture {
	let creator = h.funded_keypair().expect("creator");
	let mint = Keypair::new();
	h.create_mint_with_authority(
		&mint,
		&TOKEN_2022_PROGRAM,
		6,
		&creator.pubkey(),
		&[MintExtension::MetadataPointer],
	)
	.expect("base mint");
	let base_mint = mint.pubkey();
	h.send(
		&[create_launch_instruction(
			h,
			config,
			&creator.pubkey(),
			&base_mint,
			activation_time,
		)],
		&[&creator],
	)
	.expect("create launch");
	let launch = Launch::find_pda(&base_mint).0;
	LaunchFixture {
		base_vault: vault(&launch, &base_mint),
		quote_vault: vault(&launch, &config.quote_mint),
		creator,
		base_mint,
		launch,
	}
}

fn create_launch(h: &Harness, config: &Config) -> LaunchFixture {
	create_launch_at(h, config, 0)
}

fn launch_state(h: &Harness, launch: &Pubkey) -> LaunchState {
	let account = h.account(launch).expect("launch account");
	assert_eq!(
		account.owner, PINA_BONDING_CURVE_ID,
		"launch must be program owned"
	);
	let state = Launch::from_bytes(&account.data).expect("decode launch");
	LaunchState {
		config: state.config,
		creator: state.creator,
		pool: state.pool,
		sqrt_price: state.sqrt_price.get(),
		quote_reserve: state.quote_reserve.get(),
		partner_fees: state.partner_fees.get(),
		creator_fees: state.creator_fees.get(),
		status: state.status,
	}
}

/// The launch fields the tests assert on.
struct LaunchState {
	config: Pubkey,
	creator: Pubkey,
	pool: Pubkey,
	sqrt_price: u128,
	quote_reserve: u64,
	partner_fees: u64,
	creator_fees: u64,
	status: u8,
}

impl LaunchState {
	/// Quote the vault must hold: the curve reserve plus unclaimed fees.
	fn quote_owed(&self) -> u64 {
		self.quote_reserve + self.creator_fees + self.partner_fees
	}
}

/// A trader holding `quote` and an empty base account.
fn trader(h: &Harness, config: &Config, launch: &LaunchFixture, quote: u64) -> Keypair {
	let trader = h.funded_keypair().expect("trader");
	h.mint_to_owner(&config.quote_mint, &trader.pubkey(), &TOKEN_PROGRAM, quote)
		.expect("fund quote");
	h.create_ata(&trader.pubkey(), &launch.base_mint, &TOKEN_2022_PROGRAM)
		.expect("base account");
	trader
}

fn buy_instruction(
	config: &Config,
	launch: &LaunchFixture,
	trader: &Pubkey,
	quote_in: u64,
	minimum_out: u64,
) -> Instruction {
	Buy::new(
		*trader,
		config.address,
		launch.launch,
		ata(trader, &launch.base_mint, &TOKEN_2022_PROGRAM),
		ata(trader, &config.quote_mint, &TOKEN_PROGRAM),
		launch.base_vault,
		launch.quote_vault,
		TOKEN_2022_PROGRAM,
		TOKEN_PROGRAM,
	)
	.instruction(
		BuyInstructionData::new(|data| {
			data.quote_amount_in.set(quote_in);
			data.minimum_base_out.set(minimum_out);
		})
		.expect("buy data"),
	)
}

fn sell_instruction(
	config: &Config,
	launch: &LaunchFixture,
	trader: &Pubkey,
	base_in: u64,
	minimum_out: u64,
) -> Instruction {
	Sell::new(
		*trader,
		config.address,
		launch.launch,
		ata(trader, &launch.base_mint, &TOKEN_2022_PROGRAM),
		ata(trader, &config.quote_mint, &TOKEN_PROGRAM),
		launch.base_vault,
		launch.quote_vault,
		TOKEN_2022_PROGRAM,
		TOKEN_PROGRAM,
	)
	.instruction(
		SellInstructionData::new(|data| {
			data.base_amount_in.set(base_in);
			data.minimum_quote_out.set(minimum_out);
		})
		.expect("sell data"),
	)
}

/// The AMM accounts a graduation creates.
struct PoolAddresses {
	pool: Pubkey,
	lp_mint: Pubkey,
	vault_0: Pubkey,
	vault_1: Pubkey,
	mint_0: Pubkey,
}

fn pool_addresses(config: &Config, launch: &LaunchFixture) -> PoolAddresses {
	let (mint_0, mint_1) = if launch.base_mint < config.quote_mint {
		(launch.base_mint, config.quote_mint)
	} else {
		(config.quote_mint, launch.base_mint)
	};
	let pool = Pubkey::find_program_address(
		&[
			b"pool",
			config.amm_tier.as_ref(),
			mint_0.as_ref(),
			mint_1.as_ref(),
		],
		&AMM_PROGRAM,
	)
	.0;
	let vault = |mint: &Pubkey| {
		Pubkey::find_program_address(&[b"pool_vault", pool.as_ref(), mint.as_ref()], &AMM_PROGRAM).0
	};
	PoolAddresses {
		lp_mint: Pubkey::find_program_address(&[b"pool_lp_mint", pool.as_ref()], &AMM_PROGRAM).0,
		vault_0: vault(&mint_0),
		vault_1: vault(&mint_1),
		pool,
		mint_0,
	}
}

fn graduate_instruction(
	h: &Harness,
	config: &Config,
	launch: &LaunchFixture,
	with_lp_recipients: bool,
) -> Instruction {
	let addresses = pool_addresses(config, launch);
	let mut accounts = Graduate::new(
		h.payer().pubkey(),
		config.address,
		launch.launch,
		launch.base_mint,
		config.quote_mint,
		launch.base_vault,
		launch.quote_vault,
		amm_authority(),
		AMM_PROGRAM,
		config.amm_tier,
		addresses.pool,
		addresses.lp_mint,
		addresses.vault_0,
		addresses.vault_1,
		ata(&launch.launch, &addresses.lp_mint, &TOKEN_PROGRAM),
		TOKEN_2022_PROGRAM,
		TOKEN_PROGRAM,
	);
	if with_lp_recipients {
		accounts.creator = Some(launch.creator.pubkey());
		accounts.creator_lp_token = Some(ata(
			&launch.creator.pubkey(),
			&addresses.lp_mint,
			&TOKEN_PROGRAM,
		));
		accounts.partner = Some(config.partner.pubkey());
		accounts.partner_lp_token = Some(ata(
			&config.partner.pubkey(),
			&addresses.lp_mint,
			&TOKEN_PROGRAM,
		));
	}
	accounts.instruction(GraduateInstructionData::new(|_| {}).expect("graduate data"))
}

/// Buy until the launch completes and return the buyer.
fn complete(h: &Harness, config: &Config, launch: &LaunchFixture) -> Keypair {
	let whale = trader(h, config, launch, 1_000_000_000_000);
	h.send(
		&[buy_instruction(
			config,
			launch,
			&whale.pubkey(),
			500_000_000_000,
			0,
		)],
		&[&whale],
	)
	.expect("completing buy");
	assert_eq!(
		launch_state(h, &launch.launch).status,
		1,
		"launch must be completed"
	);
	whale
}

/// Settle the anti-sniping fee so trades pay the end rate.
fn past_fee_decay(h: &Harness) {
	h.advance_seconds(i64::try_from(FEE_DECAY_SECONDS).expect("seconds") + 5)
		.expect("advance clock");
}

#[test]
#[ignore = "run with `devenv shell test:surfpool`"]
fn configurations_validate_their_terms() {
	pina_test::run(async {
		let h = Harness::start().await.expect("start");
		let amm = setup_amm(&h);
		let partner = h.funded_keypair().expect("partner");
		let quote = h.create_mint(&TOKEN_PROGRAM, 9, &[]).expect("quote");
		let attempt = |index: u64, tier: &Pubkey, terms: Terms| {
			create_config_instruction(&h, &partner, index, &quote, tier, terms)
		};

		let open_tier = create_amm_tier(&h, 1, &Pubkey::default());
		h.expect_custom_error(
			&[attempt(0, &open_tier, Terms::default())],
			&[&partner],
			code::INVALID_AMM_CONFIG,
		);
		h.expect_custom_error(
			&[attempt(
				0,
				&amm.tier,
				Terms {
					threshold: u64::MAX / 2,
					..Terms::default()
				},
			)],
			&[&partner],
			code::INVALID_MIGRATION_THRESHOLD,
		);
		h.expect_custom_error(
			&[attempt(
				0,
				&amm.tier,
				Terms {
					total_supply: 100,
					..Terms::default()
				},
			)],
			&[&partner],
			code::INVALID_SUPPLY,
		);
		h.expect_custom_error(
			&[attempt(
				0,
				&amm.tier,
				Terms {
					fee_decay: 0,
					..Terms::default()
				},
			)],
			&[&partner],
			code::INVALID_FEE_RATES,
		);

		h.send(&[attempt(0, &amm.tier, Terms::default())], &[&partner])
			.expect("create config");
		let address = LaunchConfig::find_pda(&partner.pubkey(), 0).0;
		let account = h.account(&address).expect("config");
		let config = LaunchConfig::from_bytes(&account.data).expect("decode config");
		assert_eq!(config.authority, partner.pubkey());
		assert_eq!(config.amm_config, amm.tier);
		assert!(config.migration_sqrt_price.get() > SQRT_START_PRICE);
		let sale = config.sale_supply.get();
		let migration = config.migration_supply.get();
		assert!(sale + migration + CREATOR_ALLOCATION <= TOTAL_SUPPLY);
		h.stop().expect("stop");
	});
}

#[test]
#[ignore = "run with `devenv shell test:surfpool`"]
fn launches_mint_a_fixed_supply_and_revoke_the_mint_authority() {
	pina_test::run(async {
		let h = Harness::start().await.expect("start");
		let amm = setup_amm(&h);
		let config = create_config(&h, &amm, Terms::default());
		let launch = create_launch(&h, &config);
		assert_eq!(h.token_balance(&launch.base_vault), TOTAL_SUPPLY);
		assert_eq!(h.mint_supply(&launch.base_mint), TOTAL_SUPPLY);
		assert_eq!(h.mint_authority(&launch.base_mint), None);
		let state = launch_state(&h, &launch.launch);
		assert_eq!(state.config, config.address);
		assert_eq!(state.creator, launch.creator.pubkey());
		assert_eq!(state.sqrt_price, SQRT_START_PRICE);
		assert_eq!(state.status, 0);

		// The creator must control the mint they launch.
		let creator = h.funded_keypair().expect("creator");
		let mint = Keypair::new();
		h.create_mint_with_authority(&mint, &TOKEN_2022_PROGRAM, 6, &h.payer().pubkey(), &[])
			.expect("mint");
		h.expect_custom_error(
			&[create_launch_instruction(
				&h,
				&config,
				&creator.pubkey(),
				&mint.pubkey(),
				0,
			)],
			&[&creator],
			code::INVALID_BASE_MINT,
		);

		// Transfer fees would break the curve's accounting.
		let taxed = Keypair::new();
		h.create_mint_with_authority(
			&taxed,
			&TOKEN_2022_PROGRAM,
			6,
			&creator.pubkey(),
			&[MintExtension::TransferFee],
		)
		.expect("taxed mint");
		h.expect_custom_error(
			&[create_launch_instruction(
				&h,
				&config,
				&creator.pubkey(),
				&taxed.pubkey(),
				0,
			)],
			&[&creator],
			code::UNSUPPORTED_MINT,
		);
		h.stop().expect("stop");
	});
}

#[test]
#[ignore = "run with `devenv shell test:surfpool`"]
fn trading_follows_the_curve_and_splits_fees() {
	pina_test::run(async {
		let h = Harness::start().await.expect("start");
		let amm = setup_amm(&h);
		let config = create_config(&h, &amm, Terms::default());
		let launch = create_launch(&h, &config);
		past_fee_decay(&h);
		let buyer = trader(&h, &config, &launch, 100_000_000_000);
		let base_account = ata(&buyer.pubkey(), &launch.base_mint, &TOKEN_2022_PROGRAM);
		let quote_account = ata(&buyer.pubkey(), &config.quote_mint, &TOKEN_PROGRAM);

		h.send(
			&[buy_instruction(
				&config,
				&launch,
				&buyer.pubkey(),
				1_000_000_000,
				0,
			)],
			&[&buyer],
		)
		.expect("buy");
		let bought = h.token_balance(&base_account);
		assert!(bought > 0);
		let after_buy = launch_state(&h, &launch.launch);
		let fee =
			u64::try_from((1_000_000_000u128 * u128::from(END_FEE_RATE)).div_ceil(DENOMINATOR))
				.expect("fee");
		assert_eq!(after_buy.quote_reserve, 1_000_000_000 - fee);
		assert_eq!(after_buy.creator_fees + after_buy.partner_fees, fee);
		assert_eq!(after_buy.creator_fees, fee / 2);
		assert!(after_buy.sqrt_price > SQRT_START_PRICE);
		assert_eq!(h.token_balance(&launch.quote_vault), after_buy.quote_owed());

		h.expect_custom_error(
			&[buy_instruction(
				&config,
				&launch,
				&buyer.pubkey(),
				1_000_000_000,
				bought * 2,
			)],
			&[&buyer],
			code::SLIPPAGE_EXCEEDED,
		);
		h.expect_custom_error(
			&[sell_instruction(
				&config,
				&launch,
				&buyer.pubkey(),
				bought,
				1_000_000_000,
			)],
			&[&buyer],
			code::SLIPPAGE_EXCEEDED,
		);

		let quote_before = h.token_balance(&quote_account);
		h.send(
			&[sell_instruction(
				&config,
				&launch,
				&buyer.pubkey(),
				bought,
				0,
			)],
			&[&buyer],
		)
		.expect("sell");
		let returned = h.token_balance(&quote_account) - quote_before;
		assert!(returned < 1_000_000_000, "a round trip never profits");
		// Rounding favors the curve, so selling everything back never pushes
		// the price below where it started.
		let after_sell = launch_state(&h, &launch.launch);
		assert!(after_sell.sqrt_price >= SQRT_START_PRICE);
		assert!(after_sell.sqrt_price < after_buy.sqrt_price);
		assert_eq!(
			h.token_balance(&launch.quote_vault),
			after_sell.quote_owed()
		);
		h.stop().expect("stop");
	});
}

#[test]
#[ignore = "run with `devenv shell test:surfpool`"]
fn the_fee_starts_high_and_decays_and_trading_waits_for_activation() {
	pina_test::run(async {
		let h = Harness::start().await.expect("start");
		let amm = setup_amm(&h);
		let config = create_config(&h, &amm, Terms::default());
		let activation = u64::try_from(h.now() + 120).expect("timestamp");
		let launch = create_launch_at(&h, &config, activation);
		let buyer = trader(&h, &config, &launch, 10_000_000_000);
		h.expect_custom_error(
			&[buy_instruction(
				&config,
				&launch,
				&buyer.pubkey(),
				1_000_000_000,
				0,
			)],
			&[&buyer],
			code::NOT_ACTIVE,
		);

		h.advance_seconds(121).expect("activate");
		h.send(
			&[buy_instruction(
				&config,
				&launch,
				&buyer.pubkey(),
				1_000_000_000,
				0,
			)],
			&[&buyer],
		)
		.expect("early buy");
		let early = launch_state(&h, &launch.launch);
		let early_fee = early.creator_fees + early.partner_fees;
		assert!(
			early_fee > 400_000_000,
			"the opening fee is close to 50%: {early_fee}"
		);

		h.advance_seconds(i64::try_from(FEE_DECAY_SECONDS).expect("seconds"))
			.expect("decay");
		h.send(
			&[buy_instruction(
				&config,
				&launch,
				&buyer.pubkey(),
				1_000_000_000,
				0,
			)],
			&[&buyer],
		)
		.expect("late buy");
		let late = launch_state(&h, &launch.launch);
		assert_eq!(
			late.creator_fees + late.partner_fees - early_fee,
			10_000_000,
			"1% after the decay"
		);
		h.stop().expect("stop");
	});
}

#[test]
#[ignore = "run with `devenv shell test:surfpool`"]
fn completion_stops_trading_and_graduation_seeds_the_amm_at_the_curve_price() {
	pina_test::run(async {
		let h = Harness::start().await.expect("start");
		let amm = setup_amm(&h);
		let config = create_config(&h, &amm, Terms::default());
		let launch = create_launch(&h, &config);
		past_fee_decay(&h);
		h.expect_custom_error(
			&[graduate_instruction(&h, &config, &launch, false)],
			&[],
			code::NOT_COMPLETED,
		);

		let whale = trader(&h, &config, &launch, 1_000_000_000_000);
		let quote_account = ata(&whale.pubkey(), &config.quote_mint, &TOKEN_PROGRAM);
		h.send(
			&[buy_instruction(
				&config,
				&launch,
				&whale.pubkey(),
				500_000_000_000,
				0,
			)],
			&[&whale],
		)
		.expect("completing buy");
		let spent = 1_000_000_000_000 - h.token_balance(&quote_account);
		assert!(
			spent < 100_000_000_000,
			"a completing buy only pays for what it used: {spent}"
		);
		let completed = launch_state(&h, &launch.launch);
		assert_eq!(completed.status, 1);
		h.expect_custom_error(
			&[buy_instruction(&config, &launch, &whale.pubkey(), 1_000, 0)],
			&[&whale],
			code::NOT_TRADING,
		);

		let supply_before = h.mint_supply(&launch.base_mint);
		h.send(&[graduate_instruction(&h, &config, &launch, false)], &[])
			.expect("graduate");
		let addresses = pool_addresses(&config, &launch);
		let graduated = launch_state(&h, &launch.launch);
		assert_eq!(graduated.status, 2);
		assert_eq!(graduated.pool, addresses.pool);
		assert_eq!(graduated.quote_reserve, 0);

		// The pool opens at the curve's final price.
		let (pool_base, pool_quote) = if addresses.mint_0 == launch.base_mint {
			(
				h.token_balance(&addresses.vault_0),
				h.token_balance(&addresses.vault_1),
			)
		} else {
			(
				h.token_balance(&addresses.vault_1),
				h.token_balance(&addresses.vault_0),
			)
		};
		// This curve's square-root prices stay below 2^60, so the Q64 price
		// and the quote it implies for the pool's base fit in 128 bits.
		let price_q64 = (completed.sqrt_price * completed.sqrt_price) >> 64;
		let implied_quote = (u128::from(pool_base) * price_q64) >> 64;
		assert!(
			implied_quote.abs_diff(u128::from(pool_quote))
				<= u128::from(pool_quote) / 1_000_000 + 1,
			"pool quote {pool_quote} vs {implied_quote} at the curve's final price"
		);
		let migration_fee = u64::try_from(
			u128::from(completed.quote_reserve) * u128::from(MIGRATION_FEE_RATE) / DENOMINATOR,
		)
		.expect("fee");
		assert_eq!(pool_quote, completed.quote_reserve - migration_fee);
		assert_eq!(
			h.token_balance(&launch.quote_vault),
			graduated.quote_owed(),
			"only fees remain"
		);

		// Every LP unit was burned and every unneeded base unit with it; only
		// the creator's unvested allocation stays behind.
		let launch_lp = ata(&launch.launch, &addresses.lp_mint, &TOKEN_PROGRAM);
		assert_eq!(h.token_balance(&launch_lp), 0);
		assert_eq!(h.mint_supply(&addresses.lp_mint), 0);
		assert_eq!(h.token_balance(&launch.base_vault), CREATOR_ALLOCATION);
		assert!(h.mint_supply(&launch.base_mint) < supply_before);

		// The pool trades, and its creator fees belong to the launch creator.
		let pool_account = h.account(&addresses.pool).expect("pool");
		assert_eq!(&pool_account.data[34..66], launch.creator.pubkey().as_ref());
		let base_account = ata(&whale.pubkey(), &launch.base_mint, &TOKEN_2022_PROGRAM);
		let (quote_pool_vault, base_pool_vault) = if addresses.mint_0 == config.quote_mint {
			(addresses.vault_0, addresses.vault_1)
		} else {
			(addresses.vault_1, addresses.vault_0)
		};
		let mut data = vec![5u8];
		data.extend_from_slice(&1_000_000_000u64.to_le_bytes());
		data.extend_from_slice(&0u64.to_le_bytes());
		let swap = Instruction::new_with_bytes(
			AMM_PROGRAM,
			&data,
			vec![
				AccountMeta::new_readonly(whale.pubkey(), true),
				AccountMeta::new(addresses.pool, false),
				AccountMeta::new(quote_account, false),
				AccountMeta::new(base_account, false),
				AccountMeta::new(quote_pool_vault, false),
				AccountMeta::new(base_pool_vault, false),
				AccountMeta::new_readonly(TOKEN_PROGRAM, false),
				AccountMeta::new_readonly(TOKEN_2022_PROGRAM, false),
			],
		);
		let base_before = h.token_balance(&base_account);
		h.send(&[swap], &[&whale])
			.expect("swap on the graduated pool");
		assert!(h.token_balance(&base_account) > base_before);
		h.stop().expect("stop");
	});
}

#[test]
#[ignore = "run with `devenv shell test:surfpool`"]
fn graduation_pays_lp_shares_to_the_creator_and_partner() {
	pina_test::run(async {
		let h = Harness::start().await.expect("start");
		let amm = setup_amm(&h);
		let config = create_config(
			&h,
			&amm,
			Terms {
				creator_lp_share: 100_000,
				partner_lp_share: 200_000,
				pool_creator_mode: 1,
				..Terms::default()
			},
		);
		let launch = create_launch(&h, &config);
		past_fee_decay(&h);
		complete(&h, &config, &launch);
		h.expect_custom_error(
			&[graduate_instruction(&h, &config, &launch, false)],
			&[],
			code::MISSING_LP_ACCOUNT,
		);
		h.send(&[graduate_instruction(&h, &config, &launch, true)], &[])
			.expect("graduate");

		let addresses = pool_addresses(&config, &launch);
		let creator_lp = h.token_balance(&ata(
			&launch.creator.pubkey(),
			&addresses.lp_mint,
			&TOKEN_PROGRAM,
		));
		let partner_lp = h.token_balance(&ata(
			&config.partner.pubkey(),
			&addresses.lp_mint,
			&TOKEN_PROGRAM,
		));
		let minted = h.mint_supply(&addresses.lp_mint);
		assert_eq!(
			minted,
			creator_lp + partner_lp,
			"everything else was burned"
		);
		assert!(partner_lp >= creator_lp * 2 - 1 && partner_lp <= creator_lp * 2 + 1);
		let pool_account = h.account(&addresses.pool).expect("pool");
		assert_eq!(
			&pool_account.data[34..66],
			config.partner.pubkey().as_ref(),
			"mode 1 pays the partner"
		);
		h.stop().expect("stop");
	});
}

#[test]
#[ignore = "run with `devenv shell test:surfpool`"]
fn fees_and_vested_allocations_are_claimed_only_by_their_owners() {
	pina_test::run(async {
		let h = Harness::start().await.expect("start");
		let amm = setup_amm(&h);
		let config = create_config(&h, &amm, Terms::default());
		let launch = create_launch(&h, &config);
		past_fee_decay(&h);
		let buyer = trader(&h, &config, &launch, 10_000_000_000);
		h.send(
			&[buy_instruction(
				&config,
				&launch,
				&buyer.pubkey(),
				5_000_000_000,
				0,
			)],
			&[&buyer],
		)
		.expect("buy");
		let state = launch_state(&h, &launch.launch);

		let partner_quote = h
			.create_ata(&config.partner.pubkey(), &config.quote_mint, &TOKEN_PROGRAM)
			.expect("partner account");
		let creator_quote = h
			.create_ata(&launch.creator.pubkey(), &config.quote_mint, &TOKEN_PROGRAM)
			.expect("creator account");
		let partner_claim = |signer: &Pubkey| {
			ClaimPartnerFees::new(
				*signer,
				config.address,
				launch.launch,
				launch.quote_vault,
				partner_quote,
				TOKEN_PROGRAM,
			)
			.instruction(ClaimPartnerFeesInstructionData::new(|_| {}).expect("data"))
		};
		let creator_claim = |signer: &Pubkey| {
			ClaimCreatorFees::new(
				*signer,
				config.address,
				launch.launch,
				launch.quote_vault,
				creator_quote,
				TOKEN_PROGRAM,
			)
			.instruction(ClaimCreatorFeesInstructionData::new(|_| {}).expect("data"))
		};
		h.expect_custom_error(
			&[partner_claim(&buyer.pubkey())],
			&[&buyer],
			code::UNAUTHORIZED,
		);
		h.expect_custom_error(
			&[creator_claim(&buyer.pubkey())],
			&[&buyer],
			code::UNAUTHORIZED,
		);
		h.send(
			&[partner_claim(&config.partner.pubkey())],
			&[&config.partner],
		)
		.expect("partner claim");
		h.send(
			&[creator_claim(&launch.creator.pubkey())],
			&[&launch.creator],
		)
		.expect("creator claim");
		assert_eq!(h.token_balance(&partner_quote), state.partner_fees);
		assert_eq!(h.token_balance(&creator_quote), state.creator_fees);
		h.expect_custom_error(
			&[creator_claim(&launch.creator.pubkey())],
			&[&launch.creator],
			code::NOTHING_TO_CLAIM,
		);

		// The allocation vests linearly from activation over an hour.
		let creator_base = h
			.create_ata(
				&launch.creator.pubkey(),
				&launch.base_mint,
				&TOKEN_2022_PROGRAM,
			)
			.expect("creator base");
		let claim_allocation = |signer: &Pubkey| {
			ClaimCreatorAllocation::new(
				*signer,
				config.address,
				launch.launch,
				launch.base_vault,
				creator_base,
				TOKEN_2022_PROGRAM,
			)
			.instruction(ClaimCreatorAllocationInstructionData::new(|_| {}).expect("data"))
		};
		h.expect_custom_error(
			&[claim_allocation(&buyer.pubkey())],
			&[&buyer],
			code::UNAUTHORIZED,
		);
		h.send(
			&[claim_allocation(&launch.creator.pubkey())],
			&[&launch.creator],
		)
		.expect("partial");
		let partial = h.token_balance(&creator_base);
		assert!(partial > 0 && partial < CREATOR_ALLOCATION / 2, "{partial}");
		h.advance_seconds(i64::try_from(VESTING_SECONDS).expect("seconds"))
			.expect("vest");
		h.send(
			&[claim_allocation(&launch.creator.pubkey())],
			&[&launch.creator],
		)
		.expect("rest");
		assert_eq!(h.token_balance(&creator_base), CREATOR_ALLOCATION);
		h.expect_custom_error(
			&[claim_allocation(&launch.creator.pubkey())],
			&[&launch.creator],
			code::NOTHING_TO_CLAIM,
		);

		// Creator rights move with `SetLaunchCreator`.
		let successor = h.funded_keypair().expect("successor");
		let set_creator = |signer: &Pubkey| {
			SetLaunchCreator::new(*signer, launch.launch).instruction(
				SetLaunchCreatorInstructionData::new(|data| data.new_creator = successor.pubkey())
					.expect("data"),
			)
		};
		h.expect_custom_error(
			&[set_creator(&buyer.pubkey())],
			&[&buyer],
			code::UNAUTHORIZED,
		);
		h.send(&[set_creator(&launch.creator.pubkey())], &[&launch.creator])
			.expect("set creator");
		assert_eq!(launch_state(&h, &launch.launch).creator, successor.pubkey());
		h.stop().expect("stop");
	});
}

/// Compute-unit ceilings per instruction, about 20% above the measured cost.
#[test]
#[ignore = "run with `devenv shell test:surfpool`"]
fn compute_units_stay_within_budget() {
	pina_test::run(async {
		let h = Harness::start().await.expect("start");
		let amm = setup_amm(&h);
		let config = create_config(&h, &amm, Terms::default());
		let launch = create_launch(&h, &config);
		past_fee_decay(&h);
		let buyer = trader(&h, &config, &launch, 10_000_000_000);
		let measure = |label: &str,
		               instruction: Instruction,
		               signers: &[&dyn Signer],
		               budget: u64| {
			let units = h
				.simulate(&[instruction], signers)
				.unwrap_or_else(|(error, logs)| panic!("{label}: {error:?}\n{}", logs.join("\n")));
			println!("{label}: {units} CU (budget {budget})");
			assert!(
				units <= budget,
				"{label} used {units} CU, above its {budget} CU budget"
			);
		};
		measure(
			"buy",
			buy_instruction(&config, &launch, &buyer.pubkey(), 1_000_000_000, 0),
			&[&buyer],
			10_000,
		);
		h.send(
			&[buy_instruction(
				&config,
				&launch,
				&buyer.pubkey(),
				1_000_000_000,
				0,
			)],
			&[&buyer],
		)
		.expect("buy");
		let held = h.token_balance(&ata(
			&buyer.pubkey(),
			&launch.base_mint,
			&TOKEN_2022_PROGRAM,
		));
		measure(
			"sell",
			sell_instruction(&config, &launch, &buyer.pubkey(), held / 2, 0),
			&[&buyer],
			11_000,
		);

		let creator = h.funded_keypair().expect("creator");
		let mint = Keypair::new();
		h.create_mint_with_authority(
			&mint,
			&TOKEN_2022_PROGRAM,
			6,
			&creator.pubkey(),
			&[MintExtension::MetadataPointer],
		)
		.expect("mint");
		measure(
			"create_launch",
			create_launch_instruction(&h, &config, &creator.pubkey(), &mint.pubkey(), 0),
			&[&creator],
			32_000,
		);

		complete(&h, &config, &launch);
		// Graduation searches for the pool, LP mint, vault, and token account
		// bumps of addresses derived from random mints, and each extra bump
		// attempt costs about 1,500 compute units, so its cost ranges from
		// about 65,000 to 90,000. The ceiling covers the spread.
		measure(
			"graduate",
			graduate_instruction(&h, &config, &launch, false),
			&[],
			130_000,
		);
		h.stop().expect("stop");
	});
}
