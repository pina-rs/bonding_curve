//! Drives the compiled `pina-curve` binary against a live Surfnet.
//!
//! Build the CLI first (`cargo build -p pina_bonding_curve_cli` from the
//! repository root); `devenv shell test:surfpool` does this before running
//! the suite.

use std::path::Path;
use std::path::PathBuf;
use std::process::Command;

use pina_test::Keypair;
use pina_test::Pubkey;
use pina_test::Signer;

use crate::AMM_TIER;
use crate::amm_authority;
use crate::create_amm_tier;
use crate::harness::Harness;
use crate::harness::NATIVE_MINT;
use crate::harness::TOKEN_2022_PROGRAM;
use crate::harness::TOKEN_PROGRAM;
use crate::harness::ata;

fn binary() -> PathBuf {
	std::env::var_os("PINA_CURVE_CLI").map_or_else(
		|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../../../target/debug/pina-curve"),
		PathBuf::from,
	)
}

/// A scratch directory unique to `signer`.
fn scratch(signer: &Keypair) -> PathBuf {
	let directory = std::env::temp_dir().join(format!("pina-curve-cli-{}", signer.pubkey()));
	std::fs::create_dir_all(&directory).expect("scratch directory");
	directory
}

/// Run `pina-curve --json <args>` as `signer` and parse its JSON output.
fn run(h: &Harness, signer: &Keypair, args: &[&str]) -> serde_json::Value {
	let keypair = scratch(signer).join("signer.json");
	std::fs::write(
		&keypair,
		serde_json::to_string(&signer.to_bytes().to_vec()).expect("keypair json"),
	)
	.expect("write keypair");
	let output = Command::new(binary())
		.arg("--url")
		.arg(h.rpc_url())
		.arg("--keypair")
		.arg(&keypair)
		.arg("--json")
		.args(args)
		.output()
		.unwrap_or_else(|error| {
			panic!(
				"run {}: {error}; build it with `cargo build -p pina_bonding_curve_cli`",
				binary().display()
			)
		});
	assert!(
		output.status.success(),
		"pina-curve {args:?} failed:\n{}",
		String::from_utf8_lossy(&output.stderr)
	);
	serde_json::from_slice(&output.stdout).unwrap_or_else(|error| {
		panic!(
			"pina-curve {args:?} printed invalid JSON ({error}):\n{}",
			String::from_utf8_lossy(&output.stdout)
		)
	})
}

/// Write a terms file with a flat 1% fee, so quotes and trades agree exactly.
fn write_terms(directory: &Path, pool_creator: &str) -> PathBuf {
	let path = directory.join("terms.json");
	let terms = serde_json::json!({
		"sqrt_start_price": crate::SQRT_START_PRICE.to_string(),
		"segments": [{
			"sqrt_price": (crate::SQRT_START_PRICE * 8).to_string(),
			"liquidity": crate::LIQUIDITY.to_string(),
		}],
		"migration_quote_threshold": crate::THRESHOLD,
		"total_supply": crate::TOTAL_SUPPLY,
		"base_decimals": 6,
		"creator_allocation": crate::CREATOR_ALLOCATION,
		"creator_vesting_duration": 3600,
		"start_fee_rate": 10_000,
		"end_fee_rate": 10_000,
		"creator_fee_share": 500_000,
		"migration_fee_rate": 20_000,
		"partner_lp_share": 100_000,
		"pool_creator": pool_creator,
	});
	std::fs::write(&path, terms.to_string()).expect("write terms");
	path
}

fn text<'a>(value: &'a serde_json::Value, key: &str) -> &'a str {
	value[key]
		.as_str()
		.unwrap_or_else(|| panic!("`{key}` missing from {value}"))
}

fn number(value: &serde_json::Value, key: &str) -> u64 {
	value[key]
		.as_u64()
		.unwrap_or_else(|| panic!("`{key}` missing from {value}"))
}

#[test]
#[ignore = "run with `devenv shell test:surfpool`"]
fn the_cli_runs_a_launch_from_configuration_to_graduation() {
	pina_test::run(async {
		let h = Harness::start().await.expect("start");
		let tier = create_amm_tier(&h, AMM_TIER, &amm_authority());
		let partner = h.funded_keypair().expect("partner");
		let creator = h.funded_keypair().expect("creator");
		let quote_mint = h.create_mint(&TOKEN_PROGRAM, 9, &[]).expect("quote mint");
		let terms = write_terms(&scratch(&partner), "partner");
		let tier = tier.to_string();
		let quote = quote_mint.to_string();
		let create_args = [
			"config",
			"create",
			"--index",
			"4",
			"--quote-mint",
			&quote,
			"--amm-config",
			&tier,
			"--terms",
			terms.to_str().expect("utf-8 path"),
		];

		// `--simulate` previews the derived supplies without creating anything.
		let preview = run(&h, &partner, &[&["--simulate"], &create_args[..]].concat());
		assert!(preview["signature"].is_null());
		let sale_supply = number(&preview["details"], "sale_supply");
		assert!(sale_supply > 0);
		let config = text(&preview["details"], "config").to_owned();
		assert!(
			h.account(&config.parse().expect("config address"))
				.is_none()
		);

		let created = run(&h, &partner, &create_args);
		assert_eq!(created["details"], {
			let mut expected = preview["details"].clone();
			expected["compute_units"] = created["details"]["compute_units"].clone();
			expected
		});
		let shown = run(&h, &partner, &["config", "show", "--config", &config]);
		assert_eq!(text(&shown, "authority"), partner.pubkey().to_string());
		assert_eq!(number(&shown, "sale_supply"), sale_supply);
		assert_eq!(text(&shown, "pool_creator"), "partner");

		let launched = run(
			&h,
			&creator,
			&["launch", "create", "--config", &config, "--token-2022"],
		);
		let base_mint = text(&launched["details"], "base_mint").to_owned();
		let base: Pubkey = base_mint.parse().expect("base mint");
		assert_eq!(h.mint_supply(&base), crate::TOTAL_SUPPLY);
		assert_eq!(h.mint_authority(&base), None);

		h.mint_to_owner(
			&quote_mint,
			&creator.pubkey(),
			&TOKEN_PROGRAM,
			1_000_000_000_000,
		)
		.expect("fund creator");
		let quoted = run(
			&h,
			&creator,
			&["quote", "--base-mint", &base_mint, "--buy", "1000000000"],
		);
		let quoted_base = number(&quoted["details"], "base_amount");
		assert_eq!(number(&quoted["details"], "fee"), 10_000_000);
		run(
			&h,
			&creator,
			&[
				"buy",
				"--base-mint",
				&base_mint,
				"--quote-amount",
				"1000000000",
			],
		);
		let creator_base = ata(&creator.pubkey(), &base, &TOKEN_2022_PROGRAM);
		assert_eq!(h.token_balance(&creator_base), quoted_base);

		let half = (quoted_base / 2).to_string();
		let sold = run(
			&h,
			&creator,
			&["sell", "--base-mint", &base_mint, "--base-amount", &half],
		);
		assert!(number(&sold["details"], "quote_amount") > 0);
		let trading = run(&h, &creator, &["launch", "show", "--base-mint", &base_mint]);
		assert_eq!(text(&trading, "status"), "trading");
		assert!(trading["progress"].as_f64().expect("progress") > 0.0);

		run(
			&h,
			&creator,
			&[
				"buy",
				"--base-mint",
				&base_mint,
				"--quote-amount",
				"500000000000",
			],
		);
		let graduated = run(&h, &partner, &["graduate", "--base-mint", &base_mint]);
		assert!(number(&graduated["details"], "partner_lp") > 0);
		assert_eq!(number(&graduated["details"], "creator_lp"), 0);
		let pool = text(&graduated["details"], "pool").to_owned();
		let shown = run(&h, &creator, &["launch", "show", "--base-mint", &base_mint]);
		assert_eq!(text(&shown, "status"), "graduated");
		assert_eq!(text(&shown, "pool"), pool);

		let creator_fees = run(
			&h,
			&creator,
			&["claim", "creator-fees", "--base-mint", &base_mint],
		);
		assert!(number(&creator_fees["details"], "amount") > 0);
		let partner_fees = run(
			&h,
			&partner,
			&["claim", "partner-fees", "--base-mint", &base_mint],
		);
		assert!(number(&partner_fees["details"], "amount") > 0);
		let allocation = run(
			&h,
			&creator,
			&["claim", "allocation", "--base-mint", &base_mint],
		);
		assert!(number(&allocation["details"], "amount") > 0);

		let successor = Keypair::new().pubkey().to_string();
		run(
			&h,
			&creator,
			&[
				"launch",
				"set-creator",
				"--base-mint",
				&base_mint,
				"--new-creator",
				&successor,
			],
		);
		let shown = run(&h, &creator, &["launch", "show", "--base-mint", &base_mint]);
		assert_eq!(text(&shown, "creator"), successor);
		h.stop().expect("stop");
	});
}

#[test]
#[ignore = "run with `devenv shell test:surfpool`"]
fn the_cli_wraps_sol_for_launches_that_raise_sol() {
	pina_test::run(async {
		let h = Harness::start().await.expect("start");
		let tier = create_amm_tier(&h, AMM_TIER, &amm_authority()).to_string();
		let partner = h.funded_keypair().expect("partner");
		let terms = write_terms(&scratch(&partner), "creator");
		let native = NATIVE_MINT.to_string();
		let created = run(
			&h,
			&partner,
			&[
				"config",
				"create",
				"--index",
				"0",
				"--quote-mint",
				&native,
				"--amm-config",
				&tier,
				"--terms",
				terms.to_str().expect("utf-8 path"),
			],
		);
		let config = text(&created["details"], "config").to_owned();
		let launched = run(&h, &partner, &["launch", "create", "--config", &config]);
		let base_mint = text(&launched["details"], "base_mint").to_owned();

		let buyer = h.funded_keypair().expect("buyer");
		run(
			&h,
			&buyer,
			&[
				"buy",
				"--base-mint",
				&base_mint,
				"--quote-amount",
				"100000000",
			],
		);
		let base: Pubkey = base_mint.parse().expect("base mint");
		assert!(h.token_balance(&ata(&buyer.pubkey(), &base, &TOKEN_PROGRAM)) > 0);
		// The CLI wrapped exactly what the buy spent.
		assert_eq!(
			h.token_balance(&ata(&buyer.pubkey(), &NATIVE_MINT, &TOKEN_PROGRAM)),
			0
		);
		h.stop().expect("stop");
	});
}

#[test]
#[ignore = "run with `devenv shell test:surfpool`"]
fn the_design_helpers_need_no_cluster() {
	let output = Command::new(binary())
		.args([
			"--json",
			"design",
			"segment",
			"--from-price",
			"0.000028",
			"--to-price",
			"0.001792",
			"--quote",
			"85000000000",
		])
		.output()
		.unwrap_or_else(|error| panic!("run {}: {error}", binary().display()));
	assert!(
		output.status.success(),
		"{}",
		String::from_utf8_lossy(&output.stderr)
	);
	let value: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json");
	let liquidity: u128 = text(&value, "liquidity").parse().expect("liquidity");
	assert!(liquidity > 0);
}
