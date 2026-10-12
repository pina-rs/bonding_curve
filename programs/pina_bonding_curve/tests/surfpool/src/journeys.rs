//! Every instruction, in one ordered journey, through every client that can
//! talk to a live Surfnet.
//!
//! The Rust journey drives all nine instructions through the generated Rust
//! client in a single lifecycle, which also gives the performance benchmark
//! one deterministic sample per instruction. The TypeScript journey drives
//! the same lifecycle through the generated `@pina-rs/bonding-curve`
//! client: the Rust side prepares the AMM tier, token fixtures, and every
//! derived address, then asserts the resulting on-chain state, so the wire
//! format the TypeScript client produces is proven end to end.
//!
//! The Dart client is codecs-only by design (it ships no RPC or transaction
//! dependency), so its surface stays covered by its unit tests; the CLI
//! already drives the program end to end in [`crate::cli`].

use std::fs;
use std::path::PathBuf;
use std::process::Command;

use pina_bonding_curve_client::accounts::Launch;
use pina_bonding_curve_client::accounts::LaunchConfig;
use pina_bonding_curve_client::instructions::ClaimCreatorAllocation;
use pina_bonding_curve_client::instructions::ClaimCreatorAllocationInstructionData;
use pina_bonding_curve_client::instructions::ClaimCreatorFees;
use pina_bonding_curve_client::instructions::ClaimCreatorFeesInstructionData;
use pina_bonding_curve_client::instructions::ClaimPartnerFees;
use pina_bonding_curve_client::instructions::ClaimPartnerFeesInstructionData;
use pina_bonding_curve_client::instructions::SetLaunchCreator;
use pina_bonding_curve_client::instructions::SetLaunchCreatorInstructionData;
use pina_test::Keypair;
use pina_test::Pubkey;
use pina_test::Signer;

use crate::Config;
use crate::Terms;
use crate::amm_authority;
use crate::buy_instruction;
use crate::create_amm_tier;
use crate::create_config_instruction;
use crate::create_launch_instruction;
use crate::graduate_instruction;
use crate::harness::Harness;
use crate::harness::MintExtension;
use crate::harness::TOKEN_2022_PROGRAM;
use crate::harness::TOKEN_PROGRAM;
use crate::harness::ata;
use crate::launch_state;
use crate::sell_instruction;
use crate::trader;
use crate::vault;

/// Fixed seeds for every keypair that feeds a PDA derivation, so the journey
/// costs the same compute units on every run and the performance benchmark
/// compares programs, not addresses.
const PARTNER_SEED: [u8; 32] = *b"pina curve journey partner 00010";
const QUOTE_MINT_SEED: [u8; 32] = *b"pina curve journey quote 0000001";
const BASE_MINT_SEED: [u8; 32] = *b"pina curve journey base 00000001";
/// The tier index the journeys create their configuration under.
const JOURNEY_TIER: u16 = 101;

/// A configuration whose creator allocation vests immediately, so the journey
/// can claim it without moving the clock.
fn journey_terms() -> Terms {
	Terms {
		threshold: crate::THRESHOLD,
		total_supply: crate::TOTAL_SUPPLY,
		start_fee_rate: crate::START_FEE_RATE,
		fee_decay: crate::FEE_DECAY_SECONDS,
		creator_lp_share: 0,
		partner_lp_share: 0,
		pool_creator_mode: 0,
		vesting_seconds: Some(0),
	}
}

/// Create the journey's quote mint with a fixed keypair.
fn journey_quote_mint(h: &Harness) -> Pubkey {
	let mint = Keypair::new_from_array(QUOTE_MINT_SEED);
	h.create_mint_with_authority(&mint, &TOKEN_PROGRAM, 9, &h.payer().pubkey(), &[])
		.expect("quote mint");
	mint.pubkey()
}

/// A configuration created by a seeded partner under a fresh restricted tier.
fn journey_config(h: &Harness, quote_mint: &Pubkey) -> Config {
	let tier = create_amm_tier(h, JOURNEY_TIER, &amm_authority());
	let partner = Keypair::new_from_array(PARTNER_SEED);
	h.fund(&partner.pubkey(), 1_000_000_000)
		.expect("fund partner");
	h.send(
		&[create_config_instruction(
			h,
			&partner,
			0,
			quote_mint,
			&tier,
			journey_terms(),
		)],
		&[&partner],
	)
	.expect("create config");
	Config {
		address: LaunchConfig::find_pda(&partner.pubkey(), 0).0,
		partner,
		quote_mint: *quote_mint,
		amm_tier: tier,
	}
}

/// A launch created by a seeded creator over a seeded base mint.
fn journey_launch(h: &Harness, config: &Config) -> crate::LaunchFixture {
	let creator = Keypair::new_from_array(*b"pina curve journey creator 00100");
	h.fund(&creator.pubkey(), 1_000_000_000)
		.expect("fund creator");
	let mint = Keypair::new_from_array(BASE_MINT_SEED);
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
			0,
		)],
		&[&creator],
	)
	.expect("create launch");
	let launch = Launch::find_pda(&base_mint).0;
	crate::LaunchFixture {
		base_vault: vault(&launch, &base_mint),
		quote_vault: vault(&launch, &config.quote_mint),
		creator,
		base_mint,
		launch,
	}
}

/// Derive the AMM accounts a graduation creates, from the tier and mints.
fn pool_addresses_from(
	tier: &Pubkey,
	base_mint: &Pubkey,
	quote_mint: &Pubkey,
) -> crate::PoolAddresses {
	let (mint_0, mint_1) = if base_mint < quote_mint {
		(base_mint, quote_mint)
	} else {
		(quote_mint, base_mint)
	};
	let pool = Pubkey::find_program_address(
		&[b"pool", tier.as_ref(), mint_0.as_ref(), mint_1.as_ref()],
		&crate::harness::AMM_PROGRAM,
	)
	.0;
	let vault = |mint: &Pubkey| {
		Pubkey::find_program_address(
			&[b"pool_vault", pool.as_ref(), mint.as_ref()],
			&crate::harness::AMM_PROGRAM,
		)
		.0
	};
	crate::PoolAddresses {
		lp_mint: Pubkey::find_program_address(
			&[b"pool_lp_mint", pool.as_ref()],
			&crate::harness::AMM_PROGRAM,
		)
		.0,
		vault_0: vault(mint_0),
		vault_1: vault(mint_1),
		pool,
		mint_0: *mint_0,
	}
}

/// Run all nine instructions through the generated Rust client, in order, on
/// one launch, asserting the state each one leaves behind.
#[test]
#[ignore = "run with `devenv shell test:surfpool`"]
fn every_instruction_runs_in_one_journey() {
	pina_test::run(async {
		let h = Harness::start().await.expect("start");
		let quote_mint = journey_quote_mint(&h);
		let config = journey_config(&h, &quote_mint);
		let launch = journey_launch(&h, &config);
		assert_eq!(launch_state(&h, &launch.launch).status, 0);

		let buyer = trader(&h, &config, &launch, 10_000_000_000);
		h.send(
			&[buy_instruction(
				&config,
				&launch,
				&buyer.pubkey(),
				2_000_000_000,
				0,
			)],
			&[&buyer],
		)
		.expect("buy");
		h.send(
			&[sell_instruction(
				&config,
				&launch,
				&buyer.pubkey(),
				100_000_000,
				0,
			)],
			&[&buyer],
		)
		.expect("sell");
		assert!(launch_state(&h, &launch.launch).quote_reserve > 0);

		let whale = trader(&h, &config, &launch, 1_000_000_000_000);
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
		assert_eq!(launch_state(&h, &launch.launch).status, 1);

		h.send(&[graduate_instruction(&h, &config, &launch, false)], &[])
			.expect("graduate");
		assert_eq!(launch_state(&h, &launch.launch).status, 2);

		let partner_quote = h
			.create_ata(&config.partner.pubkey(), &config.quote_mint, &TOKEN_PROGRAM)
			.expect("partner account");
		h.send(
			&[ClaimPartnerFees::new(
				config.partner.pubkey(),
				config.address,
				launch.launch,
				launch.quote_vault,
				partner_quote,
				TOKEN_PROGRAM,
			)
			.instruction(ClaimPartnerFeesInstructionData::new(|_| {}).expect("partner data"))],
			&[&config.partner],
		)
		.expect("claim partner fees");
		let state = launch_state(&h, &launch.launch);
		assert_eq!(state.partner_fees, 0);

		let creator_quote = h
			.create_ata(&launch.creator.pubkey(), &config.quote_mint, &TOKEN_PROGRAM)
			.expect("creator account");
		h.send(
			&[ClaimCreatorFees::new(
				launch.creator.pubkey(),
				config.address,
				launch.launch,
				launch.quote_vault,
				creator_quote,
				TOKEN_PROGRAM,
			)
			.instruction(ClaimCreatorFeesInstructionData::new(|_| {}).expect("creator data"))],
			&[&launch.creator],
		)
		.expect("claim creator fees");

		let creator_base = h
			.create_ata(
				&launch.creator.pubkey(),
				&launch.base_mint,
				&TOKEN_2022_PROGRAM,
			)
			.expect("allocation account");
		h.send(
			&[ClaimCreatorAllocation::new(
				launch.creator.pubkey(),
				config.address,
				launch.launch,
				launch.base_vault,
				creator_base,
				TOKEN_2022_PROGRAM,
			)
			.instruction(
				ClaimCreatorAllocationInstructionData::new(|_| {}).expect("allocation data"),
			)],
			&[&launch.creator],
		)
		.expect("claim allocation");
		assert_eq!(h.token_balance(&creator_base), crate::CREATOR_ALLOCATION);

		let successor = h.funded_keypair().expect("successor");
		h.send(
			&[
				SetLaunchCreator::new(launch.creator.pubkey(), launch.launch).instruction(
					SetLaunchCreatorInstructionData::new(|data| {
						data.new_creator = successor.pubkey();
					})
					.expect("data"),
				),
			],
			&[&launch.creator],
		)
		.expect("set launch creator");
		assert_eq!(launch_state(&h, &launch.launch).creator, successor.pubkey());
		h.stop().expect("stop");
	});
}

/// Run all nine instructions through the generated TypeScript client against
/// one live Surfnet, then check the state it left behind from Rust.
#[test]
#[ignore = "run with `devenv shell test:surfpool`"]
fn typescript_client_runs_every_instruction() {
	pina_test::run(async {
		let h = Harness::start().await.expect("start");
		// The TypeScript journey creates the configuration and launch itself, so
		// the Rust side only prepares what they build on: the AMM tier, the two
		// mints, and every derived address. All keypairs are seeded, so both
		// sides derive the same addresses independently.
		let quote_mint = journey_quote_mint(&h);
		let tier = create_amm_tier(&h, JOURNEY_TIER + 1, &amm_authority());
		let partner = Keypair::new_from_array(PARTNER_SEED);
		h.fund(&partner.pubkey(), 1_000_000_000)
			.expect("fund partner");
		let creator = Keypair::new_from_array(*b"pina curve journey creator 00100");
		h.fund(&creator.pubkey(), 1_000_000_000)
			.expect("fund creator");
		let base_mint_keypair = Keypair::new_from_array(BASE_MINT_SEED);
		h.create_mint_with_authority(
			&base_mint_keypair,
			&TOKEN_2022_PROGRAM,
			6,
			&creator.pubkey(),
			&[MintExtension::MetadataPointer],
		)
		.expect("base mint");
		let base_mint = base_mint_keypair.pubkey();
		let launch = Launch::find_pda(&base_mint).0;
		let base_vault = vault(&launch, &base_mint);
		let quote_vault = vault(&launch, &quote_mint);

		// The TypeScript side trades and claims with the creator, so it needs
		// funded token accounts the program does not create itself.
		let creator_quote = h
			.mint_to_owner(
				&quote_mint,
				&creator.pubkey(),
				&TOKEN_PROGRAM,
				500_000_000_000,
			)
			.expect("fund creator quote");
		let creator_base = h
			.create_ata(&creator.pubkey(), &base_mint, &TOKEN_2022_PROGRAM)
			.expect("creator base account");
		let partner_quote = h
			.create_ata(&partner.pubkey(), &quote_mint, &TOKEN_PROGRAM)
			.expect("partner account");

		let addresses = pool_addresses_from(&tier, &base_mint, &quote_mint);
		let directory = std::env::temp_dir().join(format!("pina-curve-journey-{launch}"));
		fs::create_dir_all(&directory).expect("journey directory");
		let fixtures = directory.join("fixtures.json");
		let journey = serde_json::json!({
			"programData": Harness::program_data(&crate::harness::AMM_PROGRAM).to_string(),
			"partnerKey": partner.to_bytes().to_vec(),
			"creatorKey": creator.to_bytes().to_vec(),
			"configIndex": 0,
			"quoteMint": quote_mint.to_string(),
			"baseMint": base_mint.to_string(),
			"creatorQuote": creator_quote.to_string(),
			"creatorBase": creator_base.to_string(),
			"partnerQuote": partner_quote.to_string(),
			"launch": launch.to_string(),
			"baseVault": base_vault.to_string(),
			"quoteVault": quote_vault.to_string(),
			"ammAuthority": amm_authority().to_string(),
			"ammProgram": crate::harness::AMM_PROGRAM.to_string(),
			"ammConfig": tier.to_string(),
			"pool": addresses.pool.to_string(),
			"lpMint": addresses.lp_mint.to_string(),
			"poolVault0": addresses.vault_0.to_string(),
			"poolVault1": addresses.vault_1.to_string(),
			"launchLpToken": ata(&launch, &addresses.lp_mint, &TOKEN_PROGRAM).to_string(),
			"terms": {
				"threshold": crate::THRESHOLD,
				"totalSupply": crate::TOTAL_SUPPLY,
				"startFeeRate": crate::START_FEE_RATE,
				"feeDecay": crate::FEE_DECAY_SECONDS,
			},
		});
		fs::write(
			&fixtures,
			serde_json::to_string(&journey).expect("fixtures json"),
		)
		.expect("write fixtures");

		// `.../programs/pina_bonding_curve/tests/surfpool` -> the repository root.
		let repository = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
			.ancestors()
			.nth(4)
			.expect("repository root")
			.to_path_buf();
		let output = Command::new("pnpm")
			.current_dir(&repository)
			.args([
				"--dir",
				"clients/typescript/pina_bonding_curve",
				"run",
				"journey",
				"--",
				"--rpc",
			])
			.arg(h.rpc_url())
			.arg("--fixtures")
			.arg(&fixtures)
			.output()
			.unwrap_or_else(|error| panic!("run the TypeScript journey: {error}"));
		assert!(
			output.status.success(),
			"the TypeScript journey failed:\n{}\n{}",
			String::from_utf8_lossy(&output.stdout),
			String::from_utf8_lossy(&output.stderr),
		);

		let state = launch_state(&h, &launch);
		assert_eq!(state.status, 2, "the journey graduated the launch");
		assert_eq!(
			state.partner_fees, 0,
			"the journey claimed the partner fees"
		);
		// The creator's base account also holds the base the journey bought, so
		// it must hold strictly more than the allocation alone.
		assert!(
			h.token_balance(&creator_base) > crate::CREATOR_ALLOCATION,
			"the journey claimed the creator allocation"
		);
		h.stop().expect("stop");
	});
}
