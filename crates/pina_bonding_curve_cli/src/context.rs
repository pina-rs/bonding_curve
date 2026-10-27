//! RPC access, signing, simulation, and event extraction.

use base64::Engine;
use pina_bonding_curve_client::PINA_BONDING_CURVE_ID;
use solana_commitment_config::CommitmentConfig;
use solana_instruction::Instruction;
use solana_keypair::Keypair;
use solana_message::Message;
use solana_pubkey::Pubkey;
use solana_rpc_client::rpc_client::RpcClient;
use solana_rpc_client_api::config::RpcSimulateTransactionConfig;
use solana_signer::Signer;
use solana_transaction::Transaction;

use crate::cli::GlobalArgs;
use crate::error::CliError;

const MAINNET: &str = "https://api.mainnet-beta.solana.com";
const DEVNET: &str = "https://api.devnet.solana.com";
const TESTNET: &str = "https://api.testnet.solana.com";
const LOCALHOST: &str = "http://127.0.0.1:8899";

/// An account fetched over RPC.
pub struct FetchedAccount {
	/// The program that owns the account.
	pub owner: Pubkey,
	/// The account's data.
	pub data: Vec<u8>,
}

/// The result of a successful simulation.
pub struct Simulation {
	/// Compute units consumed.
	pub units: u64,
	/// Every `Program data:` record the bonding curve itself emitted, in order.
	pub events: Vec<Vec<u8>>,
}

/// Everything a command needs to read the chain and sign for the user.
pub struct Context {
	rpc: RpcClient,
	signer: Keypair,
	/// Print JSON instead of text.
	pub json: bool,
	/// Simulate instead of sending.
	pub simulate_only: bool,
}

impl Context {
	/// Resolve the endpoint and load the signer.
	pub fn new(args: &GlobalArgs) -> Result<Self, CliError> {
		Ok(Self {
			rpc: RpcClient::new_with_commitment(
				resolve_endpoint(&args.url)?,
				CommitmentConfig::confirmed(),
			),
			signer: load_keypair(&args.keypair)?,
			json: args.json,
			simulate_only: args.simulate,
		})
	}

	/// The signer's address.
	pub fn signer(&self) -> Pubkey {
		self.signer.pubkey()
	}

	/// Fetch an account, failing when it does not exist.
	pub fn account(&self, address: &Pubkey) -> Result<FetchedAccount, CliError> {
		self.rpc
			.get_account_with_commitment(address, CommitmentConfig::confirmed())
			.map_err(|error| CliError::Rpc(error.to_string()))?
			.value
			.map(|account| {
				FetchedAccount {
					owner: account.owner,
					data: account.data,
				}
			})
			.ok_or(CliError::AccountNotFound(*address))
	}

	/// Whether an account exists.
	pub fn exists(&self, address: &Pubkey) -> Result<bool, CliError> {
		Ok(self
			.rpc
			.get_account_with_commitment(address, CommitmentConfig::confirmed())
			.map_err(|error| CliError::Rpc(error.to_string()))?
			.value
			.is_some())
	}

	/// Lamports that keep an account of `space` bytes rent exempt.
	pub fn minimum_balance(&self, space: u64) -> Result<u64, CliError> {
		let space = usize::try_from(space).map_err(|error| CliError::Rpc(error.to_string()))?;
		self.rpc
			.get_minimum_balance_for_rent_exemption(space)
			.map_err(|error| CliError::Rpc(error.to_string()))
	}

	fn transaction(
		&self,
		instructions: &[Instruction],
		extra_signers: &[&Keypair],
	) -> Result<Transaction, CliError> {
		let blockhash = self
			.rpc
			.get_latest_blockhash()
			.map_err(|error| CliError::Rpc(error.to_string()))?;
		let message = Message::new(instructions, Some(&self.signer.pubkey()));
		let mut signers = vec![&self.signer];
		signers.extend_from_slice(extra_signers);
		let mut transaction = Transaction::new_unsigned(message);
		transaction
			.try_sign(&signers, blockhash)
			.map_err(|error| CliError::TransactionFailed(error.to_string()))?;
		Ok(transaction)
	}

	/// Simulate `instructions` without signing, and collect the bonding curve's
	/// events. Signer account metas are preserved, but every signature is zero.
	pub fn simulate(&self, instructions: &[Instruction]) -> Result<Simulation, CliError> {
		// The RPC receives the wire transaction and could broadcast it. Simulation
		// must never give it a valid signature, even with sig_verify disabled.
		let message = Message::new(instructions, Some(&self.signer.pubkey()));
		let transaction = Transaction::new_unsigned(message);
		let result = self
			.rpc
			.simulate_transaction_with_config(
				&transaction,
				RpcSimulateTransactionConfig {
					sig_verify: false,
					replace_recent_blockhash: true,
					commitment: Some(CommitmentConfig::confirmed()),
					..RpcSimulateTransactionConfig::default()
				},
			)
			.map_err(|error| CliError::Rpc(error.to_string()))?
			.value;
		let logs = result.logs.unwrap_or_default();
		if let Some(error) = result.err {
			return Err(CliError::SimulationFailed {
				error: error.to_string(),
				logs: logs.join("\n"),
			});
		}
		Ok(Simulation {
			units: result.units_consumed.unwrap_or_default(),
			events: program_events(&logs, &PINA_BONDING_CURVE_ID),
		})
	}

	/// Send `instructions`, signed by the signer and `extra_signers`, or only
	/// simulate them under `--simulate`.
	///
	/// Returns the transaction signature, or `None` for a simulation.
	pub fn send(
		&self,
		instructions: &[Instruction],
		extra_signers: &[&Keypair],
	) -> Result<Option<String>, CliError> {
		if self.simulate_only {
			let simulation = self.simulate(instructions)?;
			if !self.json {
				println!(
					"simulation succeeded using {} compute units",
					simulation.units
				);
			}
			return Ok(None);
		}
		let transaction = self.transaction(instructions, extra_signers)?;
		self.rpc
			.send_and_confirm_transaction(&transaction)
			.map(|signature| Some(signature.to_string()))
			.map_err(|error| CliError::TransactionFailed(error.to_string()))
	}
}

/// Map a cluster name or URL to an RPC endpoint.
pub fn resolve_endpoint(value: &str) -> Result<String, CliError> {
	let endpoint = match value {
		"mainnet" | "mainnet-beta" | "m" => MAINNET,
		"devnet" | "d" => DEVNET,
		"testnet" | "t" => TESTNET,
		"localhost" | "localnet" | "l" => LOCALHOST,
		url if url.starts_with("https://") => url,
		url if url.starts_with("http://") => {
			if !is_loopback_authority(&url["http://".len()..]) {
				return Err(CliError::InsecureEndpoint(url.to_owned()));
			}
			url
		}
		other => return Err(CliError::InvalidEndpoint(other.to_owned())),
	};
	Ok(endpoint.to_owned())
}

/// Whether the authority of a URL (everything after the scheme) names a
/// loopback host. Userinfo is rejected outright: in `localhost:1@example.com`
/// the real host is `example.com`.
fn is_loopback_authority(rest: &str) -> bool {
	let authority = rest.split(['/', '?', '#']).next().unwrap_or_default();
	if authority.contains('@') {
		return false;
	}
	let host = if let Some(bracketed) = authority.strip_prefix('[') {
		bracketed.split(']').next().unwrap_or_default()
	} else {
		authority.split(':').next().unwrap_or_default()
	};
	matches!(host, "localhost" | "127.0.0.1" | "::1")
}

/// Read a Solana CLI keypair file: a JSON array of 64 bytes.
pub fn load_keypair(path: &str) -> Result<Keypair, CliError> {
	let expanded = match path.strip_prefix("~/") {
		Some(rest) => {
			let home = std::env::var("HOME").map_err(|_| {
				CliError::Keypair {
					path: path.to_owned(),
					reason: "HOME is not set".to_owned(),
				}
			})?;
			format!("{home}/{rest}")
		}
		None => path.to_owned(),
	};
	let failure = |reason: String| {
		CliError::Keypair {
			path: path.to_owned(),
			reason,
		}
	};
	let contents =
		std::fs::read_to_string(&expanded).map_err(|error| failure(error.to_string()))?;
	let bytes: Vec<u8> =
		serde_json::from_str(&contents).map_err(|error| failure(error.to_string()))?;
	Keypair::try_from(bytes.as_slice()).map_err(|error| failure(error.to_string()))
}

/// The `Program data:` records `program` itself emitted.
///
/// Runtime logs frame every invocation with `Program <id> invoke [n]` and a
/// matching `success` or `failed` line. A data record belongs to whichever
/// program is innermost at that point, so records written by programs the curve
/// calls (such as the Pina AMM during graduation) are never attributed to it.
pub fn program_events(logs: &[String], program: &Pubkey) -> Vec<Vec<u8>> {
	let program = program.to_string();
	let mut stack: Vec<&str> = Vec::new();
	let mut events = Vec::new();
	for line in logs {
		let Some(rest) = line.strip_prefix("Program ") else {
			continue;
		};
		if let Some(data) = rest.strip_prefix("data: ") {
			if stack.last() == Some(&program.as_str())
				&& let Ok(bytes) = base64::engine::general_purpose::STANDARD.decode(data.trim())
			{
				events.push(bytes);
			}
			continue;
		}
		let Some((id, tail)) = rest.split_once(' ') else {
			continue;
		};
		if tail.starts_with("invoke [") {
			stack.push(id);
		} else if tail == "success" || tail.starts_with("failed") {
			stack.pop();
		}
	}
	events
}

#[cfg(test)]
mod tests {
	use std::io::BufRead;
	use std::io::BufReader;
	use std::io::Read;
	use std::io::Write;
	use std::net::TcpListener;
	use std::thread;
	use std::time::Duration;

	use solana_instruction::AccountMeta;

	use super::*;

	/// Record actual JSON-RPC requests and reject unexpected methods.
	fn recording_rpc(
		methods: &'static [&'static str],
		mut result: impl FnMut(&serde_json::Value) -> serde_json::Value + Send + 'static,
	) -> (RpcClient, thread::JoinHandle<Vec<serde_json::Value>>) {
		let listener = TcpListener::bind("127.0.0.1:0").expect("bind RPC");
		let address = listener.local_addr().expect("RPC address");
		let requests = thread::spawn(move || {
			let mut requests = Vec::new();

			for method in methods {
				let (mut stream, _) = listener.accept().expect("RPC connection");
				stream
					.set_read_timeout(Some(Duration::from_secs(5)))
					.expect("read timeout");
				let mut reader = BufReader::new(&mut stream);
				let mut length = None;

				loop {
					let mut line = String::new();
					assert_ne!(reader.read_line(&mut line).expect("header"), 0);

					if line == "\r\n" {
						break;
					}

					if let Some(value) = line.to_ascii_lowercase().strip_prefix("content-length:") {
						length = Some(value.trim().parse::<usize>().expect("body length"));
					}
				}

				let mut body = vec![0; length.expect("Content-Length")];
				reader.read_exact(&mut body).expect("request body");
				let request: serde_json::Value = serde_json::from_slice(&body).expect("JSON-RPC");
				assert_eq!(request["method"], *method);
				let response = serde_json::json!({
					"jsonrpc": "2.0", "id": request["id"],
					"result": result(&request)
				})
				.to_string();
				write!(
					stream,
					"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: \
					 {}\r\nConnection: close\r\n\r\n{response}",
					response.len()
				)
				.expect("RPC response");
				requests.push(request);
			}

			requests
		});
		let rpc = RpcClient::new_with_timeout(format!("http://{address}"), Duration::from_secs(5));

		(rpc, requests)
	}

	#[test]
	fn simulation_wire_payload_never_authorizes_a_transaction() {
		for simulate_only in [false, true] {
			let (rpc, requests) = recording_rpc(&["simulateTransaction"], |_| {
				serde_json::json!({ "context": { "slot": 1 }, "value": {
					"err": null, "logs": [], "unitsConsumed": 100
				} })
			});
			let signer = Keypair::new();
			let extra = Keypair::new();
			let instructions = [Instruction {
				program_id: PINA_BONDING_CURVE_ID,
				accounts: vec![AccountMeta::new_readonly(extra.pubkey(), true)],
				data: vec![1],
			}];
			let expected = Message::new(&instructions, Some(&signer.pubkey()));
			let context = Context {
				rpc,
				signer,
				json: true,
				simulate_only,
			};

			if simulate_only {
				assert_eq!(
					context.send(&instructions, &[&extra]).expect("preview"),
					None
				);
			} else {
				assert_eq!(context.simulate(&instructions).expect("quote").units, 100);
			}

			let request = requests
				.join()
				.expect("RPC requests")
				.pop()
				.expect("simulation");
			assert_eq!(request["params"][1]["sigVerify"], false);
			assert_eq!(request["params"][1]["replaceRecentBlockhash"], true);
			let wire = base64::engine::general_purpose::STANDARD
				.decode(request["params"][0].as_str().expect("base64 transaction"))
				.expect("decode transaction");
			assert_eq!(wire[0], 2, "preserve both required signers");
			assert!(wire[1..129].iter().all(|byte| *byte == 0));
			assert_eq!(&wire[129..], expected.serialize());
			assert!(Transaction::new_unsigned(expected).verify().is_err());
		}
	}

	#[test]
	fn broadcast_transactions_still_have_valid_signatures() {
		let context = Context {
			rpc: RpcClient::new_mock("succeeds".to_owned()),
			signer: Keypair::new(),
			json: true,
			simulate_only: false,
		};
		let extra = Keypair::new();
		let instruction = Instruction {
			program_id: PINA_BONDING_CURVE_ID,
			accounts: vec![AccountMeta::new_readonly(extra.pubkey(), true)],
			data: vec![1],
		};
		let transaction = context
			.transaction(&[instruction], &[&extra])
			.expect("sign");
		assert_eq!(transaction.signatures.len(), 2);
		transaction.verify().expect("valid broadcast signatures");
	}

	#[test]
	fn send_broadcasts_valid_signatures_and_confirms_them() {
		let signer = Keypair::new();
		let extra = Keypair::new();
		let instructions = [Instruction {
			program_id: PINA_BONDING_CURVE_ID,
			accounts: vec![AccountMeta::new_readonly(extra.pubkey(), true)],
			data: vec![1],
		}];
		let mut message = Message::new(&instructions, Some(&signer.pubkey()));
		message.recent_blockhash = [7; 32].into();
		let mut transaction = Transaction::new_unsigned(message);
		let (rpc, requests) = recording_rpc(
			&[
				"getLatestBlockhash",
				"sendTransaction",
				"getSignatureStatuses",
			],
			move |request| {
				match request["method"].as_str().expect("method") {
					"getLatestBlockhash" => {
						serde_json::json!({
							"context": { "slot": 1 }, "value": {
								"blockhash": transaction.message.recent_blockhash.to_string(),
								"lastValidBlockHeight": 100
							}
						})
					}
					"sendTransaction" => {
						assert_eq!(request["params"][1]["encoding"], "base64");
						let wire = base64::engine::general_purpose::STANDARD
							.decode(request["params"][0].as_str().expect("base64 transaction"))
							.expect("decode broadcast");
						assert_eq!(wire[0], 2);
						assert_eq!(&wire[129..], transaction.message.serialize());

						for (signature, bytes) in transaction
							.signatures
							.iter_mut()
							.zip(wire[1..129].as_chunks::<64>().0)
						{
							*signature = (*bytes).into();
						}

						transaction.verify().expect("valid broadcast signatures");
						serde_json::json!(transaction.signatures[0].to_string())
					}
					"getSignatureStatuses" => {
						assert_eq!(
							request["params"][0][0],
							transaction.signatures[0].to_string()
						);
						serde_json::json!({ "context": { "slot": 1 }, "value": [{
						"slot": 1, "confirmations": null, "err": null,
						"status": { "Ok": null }, "confirmationStatus": "finalized"
					}] })
					}
					_ => unreachable!("recording RPC rejects unexpected methods"),
				}
			},
		);
		let context = Context {
			rpc,
			signer,
			json: true,
			simulate_only: false,
		};
		let signature = context
			.send(&instructions, &[&extra])
			.expect("send and confirm");
		assert!(signature.is_some());
		assert_eq!(requests.join().expect("RPC requests").len(), 3);
	}

	#[test]
	fn resolves_cluster_names_and_rejects_remote_http() {
		assert_eq!(resolve_endpoint("devnet").expect("devnet"), DEVNET);
		assert_eq!(resolve_endpoint("l").expect("localhost"), LOCALHOST);
		assert_eq!(
			resolve_endpoint("https://rpc.example.com").expect("https"),
			"https://rpc.example.com"
		);
		assert!(resolve_endpoint("http://127.0.0.1:8899").is_ok());
		assert!(resolve_endpoint("http://localhost").is_ok());
		assert!(resolve_endpoint("http://[::1]:8899/").is_ok());
		for remote in [
			"http://localhost:8899@rpc.example.com",
			"http://localhost.example.com",
			"http://127.0.0.1.example.com:8899",
			"http://[::2]:8899",
		] {
			assert!(
				matches!(resolve_endpoint(remote), Err(CliError::InsecureEndpoint(_))),
				"{remote} must be rejected"
			);
		}
		assert!(matches!(
			resolve_endpoint("http://rpc.example.com"),
			Err(CliError::InsecureEndpoint(_))
		));
		assert!(matches!(
			resolve_endpoint("ftp://example.com"),
			Err(CliError::InvalidEndpoint(_))
		));
	}

	#[test]
	fn attributes_events_to_the_innermost_program() {
		let curve = Pubkey::new_unique();
		let other = Pubkey::new_unique();
		let record = |bytes: &[u8]| {
			format!(
				"Program data: {}",
				base64::engine::general_purpose::STANDARD.encode(bytes)
			)
		};
		let logs = vec![
			format!("Program {other} invoke [1]"),
			record(&[9]),
			format!("Program {curve} invoke [2]"),
			record(&[1, 2]),
			"Program log: unrelated".to_owned(),
			format!("Program {curve} consumed 10 of 200000 compute units"),
			format!("Program {curve} success"),
			record(&[8]),
			format!("Program {other} success"),
			format!("Program {curve} invoke [1]"),
			record(&[3]),
			format!("Program {curve} success"),
		];
		assert_eq!(program_events(&logs, &curve), vec![vec![1, 2], vec![3]]);
	}
}
