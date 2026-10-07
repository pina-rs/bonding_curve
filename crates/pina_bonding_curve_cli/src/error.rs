//! Errors every command can report.

use solana_pubkey::Pubkey;
use thiserror::Error;

/// A failure the CLI reports to the user before exiting with status 1.
#[derive(Debug, Error)]
pub enum CliError {
	/// The `--url` value is neither a cluster name nor an HTTP(S) URL.
	#[error(
		"`{0}` is not a valid RPC endpoint; expected mainnet, devnet, testnet, localhost, or an \
		 https URL"
	)]
	InvalidEndpoint(String),
	/// A remote endpoint was given as plaintext HTTP.
	#[error("plaintext http is only allowed for localhost; use https for `{0}`")]
	InsecureEndpoint(String),
	/// The keypair file could not be read or parsed.
	#[error("could not load keypair `{path}`: {reason}")]
	Keypair {
		/// The path that was read.
		path: String,
		/// Why it failed.
		reason: String,
	},
	/// The terms file could not be read or parsed.
	#[error("could not load terms `{path}`: {reason}")]
	Terms {
		/// The path that was read.
		path: String,
		/// Why it failed.
		reason: String,
	},
	/// A curve-design input is outside the range the program accepts.
	#[error("{0}")]
	InvalidDesign(String),
	/// An RPC request failed.
	#[error("RPC request failed: {0}")]
	Rpc(String),
	/// An account the command needs does not exist.
	#[error("account {0} was not found")]
	AccountNotFound(Pubkey),
	/// An account is not owned by the program the command expected.
	#[error("account {address} is owned by {owner}, not {expected}")]
	WrongOwner {
		/// The account that was fetched.
		address: Pubkey,
		/// Its owner.
		owner: Pubkey,
		/// The owner the command required.
		expected: Pubkey,
	},
	/// Account data did not decode as the expected type.
	#[error("account {0} does not hold the expected data")]
	InvalidAccountData(Pubkey),
	/// The slippage tolerance is above 100%.
	#[error("slippage of {0} basis points is above 10,000 (100%)")]
	InvalidSlippage(u16),
	/// Simulation failed, so no transaction was sent.
	#[error("simulation failed: {error}\n{logs}")]
	SimulationFailed {
		/// The transaction error.
		error: String,
		/// The program logs, one per line.
		logs: String,
	},
	/// The simulated instruction did not emit the event the command reads.
	#[error("the simulation did not emit the expected Pina Bonding Curve event")]
	MissingEvent,
	/// The transaction was sent but failed.
	#[error("transaction failed: {0}")]
	TransactionFailed(String),
	/// Instruction data rejected the provided values.
	#[error("instruction data rejected the provided values")]
	InvalidInstructionData,
}
