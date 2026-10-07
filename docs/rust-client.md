# Rust client

`pina_bonding_curve_client` is the off-chain Rust client, generated from the program's IDL. It builds `solana_instruction::Instruction` values, decodes accounts and events, and exposes the program's errors.

```toml
[dependencies]
pina_bonding_curve_client = "0.1"
solana-instruction = "3.4"
solana-pubkey = "4"
```

For on-chain callers use [`pina_bonding_curve_cpi`](cpi.md) instead; it is `no_std` and builds CPIs rather than transactions.

## Layout

| Module         | Contents                                                                                                       |
| -------------- | -------------------------------------------------------------------------------------------------------------- |
| crate root     | `PINA_BONDING_CURVE_ID`                                                                                        |
| `instructions` | One account struct and one `*InstructionData` per instruction, plus `Migrate`, Pina's reserved migration route |
| `accounts`     | `LaunchConfig` and `Launch` decoders and PDA helpers                                                           |
| `events`       | `ConfigCreated`, `LaunchCreated`, `Traded`, `Completed`, `Graduated`, `Claimed`                                |
| `errors`       | `PinaBondingCurveError`                                                                                        |

## Units

Amounts are token base units. A price is quote base units per base base unit, and the program stores its square root as a Q64.64 integer (`sqrt(price) * 2^64`) in a `u128`. Fee rates and shares are parts per million, so `10_000` is 1%.

```rust
/// Price in quote base units per base base unit.
fn price(sqrt_price: u128) -> f64 {
	let root = sqrt_price as f64 / 18_446_744_073_709_551_616.0; // 2^64
	root * root
}
```

## Derive addresses

```rust
use pina_bonding_curve_client::accounts::{Launch, LaunchConfig};
use pina_bonding_curve_client::addresses::{amm_authority, launch_vault};

let (config, _) = LaunchConfig::find_pda(&partner, 0);
let (launch, _) = Launch::find_pda(&base_mint);
let base_vault = launch_vault(&launch, &base_mint);
let quote_vault = launch_vault(&launch, &quote_mint);
let amm_authority = amm_authority();
```

A configuration is `[b"config", authority, index]` with the index as a little-endian `u64`, so one partner can own many. The vaults and the AMM authority hold no data of their own, so the IDL cannot describe them; the hand-written `addresses` module derives them. A launch records both vault addresses, so once it exists you can read them from the account instead.

## Read a launch

```rust
use pina_bonding_curve_client::PINA_BONDING_CURVE_ID;
use pina_bonding_curve_client::accounts::{Launch, LaunchConfig};

let account = rpc.get_account(&launch)?;
assert_eq!(account.owner, PINA_BONDING_CURVE_ID, "not a Pina Bonding Curve account");
let state = Launch::from_bytes(&account.data)?;

let account = rpc.get_account(&state.config)?;
assert_eq!(account.owner, PINA_BONDING_CURVE_ID, "not a Pina Bonding Curve account");
let config = LaunchConfig::from_bytes(&account.data)?;

println!("status {}", state.status); // 0 trading, 1 completed, 2 graduated
println!("price {}", price(state.sqrt_price.get()));
println!("raised {} of {}", state.quote_reserve.get(), config.migration_quote_threshold.get());
println!("unclaimed fees {} partner, {} creator", state.partner_fees.get(), state.creator_fees.get());
```

`from_bytes` checks the discriminator, schema version, and length. It cannot know where the bytes came from, so check the owner first, as above. Numeric fields are little-endian wrappers; call `.get()` to read them. `try_from_bytes` returns a typed error instead, which tells you when a newer program version wrote the account and the client needs upgrading.

A configuration's curve is its first `segment_count` segments. Each segment ends at a square-root price and carries a liquidity; unused slots are zero.

```rust
let segments = usize::from(config.segment_count);
let uppers = &config.curve_sqrt_prices[..segments];
let liquidities = &config.curve_liquidities[..segments];
for (upper, liquidity) in uppers.iter().zip(liquidities) {
	println!("up to price {} with liquidity {}", price(upper.get()), liquidity.get());
}
```

## Buy and sell

```rust
use pina_bonding_curve_client::instructions::{Buy, BuyInstructionData};

let data = BuyInstructionData::new(|data| {
	data.quote_amount_in.set(1_000_000_000); // the most to spend, fee included
	data.minimum_base_out.set(minimum_base_out);
})?;
let instruction = Buy::new(
	trader,
	state.config,
	launch,
	trader_base,  // credited with base
	trader_quote, // debited for quote
	state.base_vault,
	state.quote_vault,
	base_token_program,
	quote_token_program,
)
.instruction(data);
```

`Sell` takes the same accounts. Its data is the exact base to sell and the least quote to accept after the fee:

```rust
use pina_bonding_curve_client::instructions::{Sell, SellInstructionData};

let data = SellInstructionData::new(|data| {
	data.base_amount_in.set(base_amount_in);
	data.minimum_quote_out.set(minimum_quote_out); // after the fee
})?;
let instruction = Sell::new(
	trader,
	state.config,
	launch,
	trader_base,
	trader_quote,
	state.base_vault,
	state.quote_vault,
	base_token_program,
	quote_token_program,
)
.instruction(data);
```

Both token accounts must already exist; the program does not create them, so prepend an idempotent associated-token-account creation as the CLI does. Each token program is the one that owns its mint, SPL Token or Token-2022. Trading opens at the launch's `activation_time`. The fee starts at the configuration's `start_fee_rate` and falls linearly to `end_fee_rate` over `fee_decay_duration` seconds.

A `Buy` that reaches the migration price completes the launch and charges only for the quote it used; the trader keeps the rest of `quote_amount_in`. The launch then stops trading until someone graduates it.

## Quote a trade

The program has no quote instruction. Simulate the trade with a zero limit and read the `Traded` event it emits; `pina-curve quote` works this way.

```rust
use pina_bonding_curve_client::PINA_BONDING_CURVE_ID;
use pina_bonding_curve_client::events::{TRADED_DISCRIMINATOR, Traded};
use solana_message::Message;
use solana_rpc_client_api::config::RpcSimulateTransactionConfig;
use solana_transaction::Transaction;

let mut instructions = create_trader_accounts;
instructions.push(buy_with_no_limit); // `minimum_base_out` set to 0
let transaction = Transaction::new_unsigned(Message::new(&instructions, Some(&trader)));
let result = rpc
	.simulate_transaction_with_config(
		&transaction,
		RpcSimulateTransactionConfig {
			sig_verify: false,
			replace_recent_blockhash: true,
			..RpcSimulateTransactionConfig::default()
		},
	)?
	.value;
if let Some(error) = result.err {
	return Err(format!("simulation failed: {error}").into());
}
let events = program_events(&result.logs.unwrap_or_default(), &PINA_BONDING_CURVE_ID);
let record = events
	.iter()
	.find(|bytes| bytes.first() == Some(&TRADED_DISCRIMINATOR))
	.ok_or("the simulation emitted no Traded event")?;
let traded = Traded::from_bytes(record)?;

// Accept up to 1% less base than the simulation returned.
let minimum_base_out = u64::try_from(u128::from(traded.base_amount.get()) * 99 / 100)?;
```

`program_events` is defined under [Events](#events). The simulation runs the real program against current state, so it includes the fee at that moment and stops at the migration price exactly as the real trade would. For a sale, read `quote_amount` instead and pass the reduced value as `minimum_quote_out`. With wrapped SOL as the quote token, wrap the amount before the buy in both the simulated and the real transaction.

## Create a configuration

Anyone can create a configuration; its signer becomes the partner who earns partner fees. A configuration cannot change after it is created.

```rust
use pina_bonding_curve_client::accounts::LaunchConfig;
use pina_bonding_curve_client::instructions::{CreateConfig, CreateConfigInstructionData};

let index = 0;
let data = CreateConfigInstructionData::new(|data| {
	data.index.set(index);
	data.sqrt_start_price.set(sqrt_start_price);
	data.curve_sqrt_prices[0].set(sqrt_end_price); // one segment
	data.curve_liquidities[0].set(liquidity);
	data.segment_count = 1;
	data.migration_quote_threshold.set(migration_quote_threshold);
	data.total_supply.set(total_supply);
	data.creator_allocation.set(creator_allocation);
	data.creator_vesting_duration.set(365 * 24 * 60 * 60); // no cliff, one year
	data.start_fee_rate.set(500_000); // 50% at activation...
	data.end_fee_rate.set(10_000); // ...falling to 1%...
	data.fee_decay_duration.set(120); // ...over two minutes
	data.creator_fee_share.set(500_000); // the creator earns half of every fee
	data.migration_fee_rate.set(20_000); // 2% of the raise at graduation
	data.base_decimals = 6;
})?;
let instruction = CreateConfig::new(
	payer,
	partner, // signs; owns the configuration and earns its partner fees
	LaunchConfig::find_pda(&partner, index).0,
	quote_mint,
	quote_token_program,
	amm_config,
)
.instruction(data);
```

Every field starts at zero, so the example leaves the vesting cliff, both LP shares, and `pool_creator_mode` (`0` pays the graduated pool's creator fees to the launch creator, `1` to the partner) at zero. With no LP shares, graduation burns every LP token the pool mints for the launch, locking that liquidity forever. `amm_config` must be a Pina AMM fee tier whose pool-creator authority is the curve's AMM authority PDA, so nobody can create a launch's pool before it graduates.

`pina-curve design segment` sizes a segment's liquidity from two human prices and the quote it should raise, and `pina-curve config create --simulate` shows the sale supply, migration supply, and migration price the program derives. The program rejects a configuration whose total supply cannot cover the sale, the pool seed, and the creator allocation.

## Create a launch

```rust
use pina_bonding_curve_client::instructions::{CreateLaunch, CreateLaunchInstructionData};

let data = CreateLaunchInstructionData::new(|data| data.activation_time.set(0))?; // open now
let instruction = CreateLaunch::new(
	payer,
	creator, // signs as the base mint's mint authority
	config,
	base_mint,
	quote_mint,
	base_vault,
	quote_vault,
	base_token_program,
	quote_token_program,
)
.instruction(data);
```

`CreateLaunch::new` derives the launch PDA and fills in the system program. The base mint must be new: zero supply, no freeze authority, the creator as its mint authority, and the configuration's `base_decimals`. Create and initialize it earlier in the same transaction. The program mints the configuration's fixed supply into the base vault, then revokes the mint authority. A Token-2022 base mint may carry only metadata, group, interest-bearing, and scaled-UI-amount extensions.

## Graduate

Graduation is permissionless: once a launch has completed, any payer can move it into a Pina AMM pool at the curve's final price.

```rust
use pina_bonding_curve_client::addresses::{PINA_AMM_ID, amm_authority, graduation_pool};
use pina_bonding_curve_client::instructions::{Graduate, GraduateInstructionData};
use solana_pubkey::Pubkey;

const TOKEN_PROGRAM: Pubkey = Pubkey::from_str_const("TokenkegQfeZyiNwAJbNbGKPFXCWuBvf9Ss623VQ5DA");
const ATA_PROGRAM: Pubkey = Pubkey::from_str_const("ATokenGPvbdGVxr1b2hvZbsiqW5xWH25efTNsLJA8knL");

let pool = graduation_pool(&config.amm_config, &state.base_mint, &state.quote_mint);
// Every Pina AMM LP mint belongs to SPL Token.
let lp_account = |owner: &Pubkey| {
	Pubkey::find_program_address(
		&[owner.as_ref(), TOKEN_PROGRAM.as_ref(), pool.lp_mint.as_ref()],
		&ATA_PROGRAM,
	)
	.0
};

let mut accounts = Graduate::new(
	payer,
	state.config,
	launch,
	state.base_mint,
	state.quote_mint,
	state.base_vault,
	state.quote_vault,
	amm_authority(),
	PINA_AMM_ID,
	config.amm_config,
	pool.pool,
	pool.lp_mint,
	pool.vault_0,
	pool.vault_1,
	lp_account(&launch),
	base_token_program,
	quote_token_program,
);
if config.creator_lp_share.get() > 0 {
	accounts.creator = Some(state.creator);
	accounts.creator_lp_token = Some(lp_account(&state.creator));
}
if config.partner_lp_share.get() > 0 {
	accounts.partner = Some(config.authority);
	accounts.partner_lp_token = Some(lp_account(&config.authority));
}
let instruction = accounts.instruction(GraduateInstructionData::new(|_| {})?);
```

`Pubkey`'s `Ord` compares bytes, which is exactly the AMM's mint order. `Graduate::new` fills in SPL Token as the LP token program, plus the associated token and system programs. The four trailing accounts are optional: the program needs a pair only when the configuration's matching LP share is non-zero, and the builder sends the curve's program id in place of any you leave as `None`. The program creates the LP token accounts it pays into.

The graduated pool is an ordinary Pina AMM pool, and the launch records its address in `pool`. Trade it with the AMM's own client.

## Claims and creator rights

| Instruction              | Signer                        | Effect                                                             |
| ------------------------ | ----------------------------- | ------------------------------------------------------------------ |
| `ClaimPartnerFees`       | the configuration's authority | Pays the launch's unclaimed partner fees, in quote                 |
| `ClaimCreatorFees`       | the launch's creator          | Pays the launch's unclaimed creator fees, in quote                 |
| `ClaimCreatorAllocation` | the launch's creator          | Pays the vested, unclaimed part of the creator allocation, in base |
| `SetLaunchCreator`       | the launch's creator          | Hands the creator's fee and allocation rights to `new_creator`     |

```rust
use pina_bonding_curve_client::instructions::{ClaimCreatorFees, ClaimCreatorFeesInstructionData};

let instruction = ClaimCreatorFees::new(
	state.creator, // signs
	state.config,
	launch,
	state.quote_vault,
	creator_quote, // any quote token account
	quote_token_program,
)
.instruction(ClaimCreatorFeesInstructionData::new(|_| {})?);
```

An instruction without arguments still takes its data struct; build it with an empty closure. The allocation vests linearly over `creator_vesting_duration` seconds, starting `creator_vesting_cliff` seconds after activation. A claim with nothing owed fails with `NothingToClaim`.

```rust
use pina_bonding_curve_client::instructions::{SetLaunchCreator, SetLaunchCreatorInstructionData};

let data = SetLaunchCreatorInstructionData::new(|data| data.new_creator = successor)?;
let instruction = SetLaunchCreator::new(state.creator, launch).instruction(data);
```

## Events

The Rust events module decodes one record at a time. Each record starts with a one-byte discriminator and a one-byte schema version. Find the curve's `Program data:` lines in a transaction's logs, base64-decode them, and match on the first byte:

| First byte | Event           |
| ---------- | --------------- |
| `1`        | `ConfigCreated` |
| `2`        | `LaunchCreated` |
| `3`        | `Traded`        |
| `4`        | `Completed`     |
| `5`        | `Graduated`     |
| `6`        | `Claimed`       |

```rust
use pina_bonding_curve_client::events::{TRADED_DISCRIMINATOR, Traded};

if bytes.first() == Some(&TRADED_DISCRIMINATOR) {
	let trade = Traded::from_bytes(&bytes)?;
	let side = if trade.is_buy == 1 { "bought" } else { "sold" };
	println!("{side} {} base for {} quote, fee {}", trade.base_amount.get(), trade.quote_amount.get(), trade.fee.get());
}
```

The events module exports every discriminator, such as `TRADED_DISCRIMINATOR` above. On a buy, `Traded::quote_amount` is what the trader paid, fee included; on a sale it is what the trader received, after the fee. The `Buy` that completes a launch emits `Completed` after its `Traded`.

Only attribute a `Program data:` line to the curve while the curve is the innermost invoked program in the log frames. This matters for `Graduate`: the Pina AMM logs its own events inside it, and the AMM's `PoolCreated` also starts with `1`.

```rust
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use solana_pubkey::Pubkey;

/// The `Program data:` records that `program` itself emitted, in order.
fn program_events(logs: &[String], program: &Pubkey) -> Vec<Vec<u8>> {
	let program = program.to_string();
	let mut stack: Vec<&str> = Vec::new();
	let mut events = Vec::new();
	for line in logs {
		let Some(rest) = line.strip_prefix("Program ") else {
			continue;
		};
		if let Some(data) = rest.strip_prefix("data: ") {
			if stack.last() == Some(&program.as_str()) {
				events.extend(STANDARD.decode(data.trim()).ok());
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
```

It needs `base64 = "0.22"`. The CLI's `program_events` function (`crates/pina_bonding_curve_cli/src/context.rs`) implements the same rule and has tests for it.

## Errors

The program returns every error as `ProgramError::Custom(code)`, with codes 0 to 21. `PinaBondingCurveError` names them and carries each one's message. It derives `num_traits::FromPrimitive`, so decoding needs `num-traits = "0.2"`:

```rust
use num_traits::FromPrimitive;
use pina_bonding_curve_client::errors::PinaBondingCurveError;

match PinaBondingCurveError::from_u32(code) {
	Some(PinaBondingCurveError::SlippageExceeded) => println!("the price moved; quote again"),
	Some(error) => println!("{error}"),
	None => println!("{code} is not a Pina Bonding Curve error"),
}
```

A custom error reported for a curve instruction can come from a program the curve calls: a token program during any transfer, or the Pina AMM during `Graduate`. The transaction logs name the program that failed; decode the code with that program's error table.

## A complete reference

The end-to-end suite in `programs/pina_bonding_curve/tests/surfpool` uses this client for every instruction against the real program and the real Pina AMM, including configuration validation, SPL Token and Token-2022 launches, fee decay, completion, graduation with LP shares, and claims. The `pina-curve` CLI (`crates/pina_bonding_curve_cli`) uses it for every command, including quoting by simulation and building graduation accounts; it is the most complete example of the client in use.
