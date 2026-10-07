//! Command-line arguments.
//!
//! Amounts are always raw base units (the integer a token account stores),
//! never UI amounts, so a command means the same thing for every mint. Prices
//! are quote base units per base base unit, the same unit the program uses.

use std::path::PathBuf;

use clap::Args;
use clap::Parser;
use clap::Subcommand;
use solana_pubkey::Pubkey;

/// Command-line interface for the Pina Bonding Curve.
#[derive(Debug, Parser)]
#[command(name = "pina-curve", version, propagate_version = true, about)]
pub struct Cli {
	/// Options shared by every command.
	#[command(flatten)]
	pub global: GlobalArgs,

	/// The command to run.
	#[command(subcommand)]
	pub command: Command,
}

/// Options shared by every command.
#[derive(Debug, Args)]
pub struct GlobalArgs {
	/// RPC endpoint: `mainnet`, `devnet`, `testnet`, `localhost`, or an https
	/// URL.
	#[arg(
		short = 'u',
		long,
		global = true,
		default_value = "devnet",
		env = "SOLANA_URL"
	)]
	pub url: String,

	/// Fee payer and signer keypair, in the Solana CLI's JSON byte-array
	/// format.
	#[arg(
		short = 'k',
		long,
		global = true,
		default_value = "~/.config/solana/id.json",
		env = "PINA_CURVE_KEYPAIR"
	)]
	pub keypair: String,

	/// Simulate and print the result instead of sending a transaction.
	#[arg(long, global = true)]
	pub simulate: bool,

	/// Print machine-readable JSON.
	#[arg(long, global = true)]
	pub json: bool,
}

/// Top-level commands.
#[derive(Debug, Subcommand)]
pub enum Command {
	/// Convert prices and size curve segments, offline.
	#[command(subcommand)]
	Design(DesignCommand),
	/// Create or inspect a launchpad configuration.
	#[command(subcommand)]
	Config(ConfigCommand),
	/// Create, inspect, or hand over a launch.
	#[command(subcommand)]
	Launch(LaunchCommand),
	/// Quote a buy or a sale without sending it.
	Quote(QuoteArgs),
	/// Buy a launch's token with the quote token.
	Buy(BuyArgs),
	/// Sell a launch's token back to the curve.
	Sell(SellArgs),
	/// Move a completed launch's liquidity into a Pina AMM pool.
	Graduate(LaunchArgs),
	/// Claim fees or a vested allocation.
	#[command(subcommand)]
	Claim(ClaimCommand),
}

/// `pina-curve design ...`
#[derive(Debug, Subcommand)]
pub enum DesignCommand {
	/// Convert a price to the Q64.64 square-root price the program stores.
	SqrtPrice(PriceArgs),
	/// Convert a Q64.64 square-root price back to a price.
	Price(SqrtPriceArgs),
	/// Size one segment: the liquidity that raises `--quote` between two
	/// prices.
	Segment(SegmentArgs),
}

/// `pina-curve design sqrt-price`
#[derive(Debug, Args)]
pub struct PriceArgs {
	/// Price in quote base units per base base unit, such as `0.000028`.
	#[arg(long)]
	pub price: f64,
}

/// `pina-curve design price`
#[derive(Debug, Args)]
pub struct SqrtPriceArgs {
	/// Square-root price as a Q64.64 integer.
	#[arg(long)]
	pub sqrt_price: u128,
}

/// `pina-curve design segment`
#[derive(Debug, Args)]
pub struct SegmentArgs {
	/// Price where the segment starts.
	#[arg(long)]
	pub from_price: f64,
	/// Price where the segment ends.
	#[arg(long)]
	pub to_price: f64,
	/// Quote base units the segment raises from start to end.
	#[arg(long)]
	pub quote: u64,
}

/// `pina-curve config ...`
#[derive(Debug, Subcommand)]
pub enum ConfigCommand {
	/// Create a configuration owned by the signer. With `--simulate`, preview
	/// the supplies and migration price the program derives.
	Create(CreateConfigArgs),
	/// Print a configuration.
	Show(ConfigArgs),
}

/// `pina-curve config create`
#[derive(Debug, Args)]
pub struct CreateConfigArgs {
	/// Configuration index; one authority can own many configurations.
	#[arg(long)]
	pub index: u64,
	/// The token every launch raises, such as wrapped SOL.
	#[arg(long)]
	pub quote_mint: Pubkey,
	/// The Pina AMM fee tier graduated pools use. It must be restricted to
	/// this program's AMM authority.
	#[arg(long)]
	pub amm_config: Pubkey,
	/// JSON file with the curve and the economic terms; see `docs/cli.md`.
	#[arg(long)]
	pub terms: PathBuf,
}

/// A configuration, addressed by its address.
#[derive(Debug, Args)]
pub struct ConfigArgs {
	/// Configuration address.
	#[arg(long)]
	pub config: Pubkey,
}

/// `pina-curve launch ...`
#[derive(Debug, Subcommand)]
pub enum LaunchCommand {
	/// Launch a token: mint its fixed supply into the curve and revoke the
	/// mint authority.
	Create(CreateLaunchArgs),
	/// Print a launch, its price, and its progress toward graduation.
	Show(LaunchArgs),
	/// Hand the creator's fee and allocation rights to another address.
	SetCreator(SetCreatorArgs),
}

/// `pina-curve launch create`
#[derive(Debug, Args)]
pub struct CreateLaunchArgs {
	/// Configuration the launch follows.
	#[arg(long)]
	pub config: Pubkey,
	/// An existing mint to launch: empty, without a freeze authority, and
	/// controlled by the signer. Omit it to create a new mint.
	#[arg(long)]
	pub base_mint: Option<Pubkey>,
	/// Create the new mint under Token-2022 instead of SPL Token.
	#[arg(long, conflicts_with = "base_mint")]
	pub token_2022: bool,
	/// Unix timestamp when trading opens. Defaults to now.
	#[arg(long, default_value_t = 0)]
	pub activation_time: u64,
}

/// A launch, addressed by its token's mint.
#[derive(Debug, Args)]
pub struct LaunchArgs {
	/// The launched token's mint.
	#[arg(long)]
	pub base_mint: Pubkey,
}

/// `pina-curve launch set-creator`
#[derive(Debug, Args)]
pub struct SetCreatorArgs {
	/// The launched token's mint.
	#[arg(long)]
	pub base_mint: Pubkey,
	/// New creator.
	#[arg(long)]
	pub new_creator: Pubkey,
}

/// `pina-curve quote`
#[derive(Debug, Args)]
#[command(group(clap::ArgGroup::new("side").required(true).args(["buy", "sell"])))]
pub struct QuoteArgs {
	/// The launched token's mint.
	#[arg(long)]
	pub base_mint: Pubkey,
	/// Quote a buy spending this many quote base units, fee included.
	#[arg(long)]
	pub buy: Option<u64>,
	/// Quote selling this many base units.
	#[arg(long)]
	pub sell: Option<u64>,
}

/// `pina-curve buy`
#[derive(Debug, Args)]
pub struct BuyArgs {
	/// The launched token's mint.
	#[arg(long)]
	pub base_mint: Pubkey,
	/// Quote base units to spend, fee included. A buy that completes the
	/// launch spends only what it uses.
	#[arg(long)]
	pub quote_amount: u64,
	/// Slippage tolerance in basis points (`100` is 1%).
	#[arg(long, default_value_t = 100)]
	pub slippage_bps: u16,
}

/// `pina-curve sell`
#[derive(Debug, Args)]
pub struct SellArgs {
	/// The launched token's mint.
	#[arg(long)]
	pub base_mint: Pubkey,
	/// Base units to sell.
	#[arg(long)]
	pub base_amount: u64,
	/// Slippage tolerance in basis points (`100` is 1%).
	#[arg(long, default_value_t = 100)]
	pub slippage_bps: u16,
}

/// `pina-curve claim ...`
#[derive(Debug, Subcommand)]
pub enum ClaimCommand {
	/// Claim the partner's trading and migration fees. The signer must be the
	/// configuration authority.
	PartnerFees(LaunchArgs),
	/// Claim the creator's trading and migration fees. The signer must be the
	/// launch creator.
	CreatorFees(LaunchArgs),
	/// Claim the creator allocation vested so far. The signer must be the
	/// launch creator.
	Allocation(LaunchArgs),
}

#[cfg(test)]
mod tests {
	use clap::CommandFactory;

	use super::*;

	#[test]
	fn command_definition_is_consistent() {
		Cli::command().debug_assert();
	}

	#[test]
	fn quote_requires_exactly_one_side() {
		let mint = Pubkey::new_unique().to_string();
		let base = ["pina-curve", "quote", "--base-mint", &mint];
		assert!(Cli::try_parse_from(base).is_err());
		assert!(Cli::try_parse_from([&base[..], &["--buy", "5"]].concat()).is_ok());
		assert!(Cli::try_parse_from([&base[..], &["--buy", "5", "--sell", "5"]].concat()).is_err());
	}

	#[test]
	fn a_new_mint_and_an_existing_mint_are_exclusive() {
		let config = Pubkey::new_unique().to_string();
		let mint = Pubkey::new_unique().to_string();
		let base = ["pina-curve", "launch", "create", "--config", &config];
		assert!(Cli::try_parse_from([&base[..], &["--token-2022"]].concat()).is_ok());
		assert!(
			Cli::try_parse_from([&base[..], &["--token-2022", "--base-mint", &mint]].concat())
				.is_err()
		);
	}
}
