//! `pina-curve`: a command-line interface for the Pina Bonding Curve.
//!
//! The binary is a thin layer over the generated `pina_bonding_curve_client`
//! crate. It derives every PDA, vault, pool address, and associated token
//! account, so commands take the values a person actually chooses: a terms
//! file, a launch's mint, and amounts. Trades are quoted by simulating the
//! instruction and reading the event the program emits, then sent with a
//! slippage limit derived from that quote.
//!
//! Run `pina-curve --help` for the command reference, or read
//! `docs/cli.md` in the repository.

mod accounts;
mod cli;
mod commands;
mod context;
mod design;
mod error;
mod output;
mod terms;

use clap::Parser;

use crate::cli::Cli;
use crate::cli::Command;
use crate::context::Context;

fn main() {
	let cli = Cli::parse();
	let result = match cli.command {
		// Curve design is arithmetic only: it needs no RPC or keypair.
		Command::Design(command) => design::run(&command, cli.global.json),
		command => Context::new(&cli.global).and_then(|context| commands::run(&context, command)),
	};
	if let Err(error) = result {
		eprintln!("error: {error}");
		std::process::exit(1);
	}
}
