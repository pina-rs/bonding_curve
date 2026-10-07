# Calling the curve from another program

`pina_bonding_curve_cpi` is a `no_std` crate for on-chain callers. Each instruction is a struct of `&AccountView` fields plus an `ix` struct of arguments. Field names follow the IDL, so numbered fields have no underscore before the digit (`pool_vault0`, `pool_vault1`); build it and call `.invoke(&program)` or `.invoke_signed(&program, signers)`.

```toml
[dependencies]
pina = { version = "0.23", default-features = false, features = ["derive", "token"] }
pina_bonding_curve_cpi = "0.1"
```

The crate itself needs no `pina` features. The examples below use `derive` for `#[derive(Accounts)]` and `#[pda]`, and `token` to read token accounts.

## Validate the program account first

```rust
use pina_bonding_curve_cpi::ProgramAccount;

let curve = ProgramAccount::try_new(self.curve_program)?; // checks address and executable flag
```

`ProgramAccount` is `pina::Program<PinaBondingCurve>`: it refuses any account that is not the deployed curve, so a caller cannot redirect your CPI to a look-alike program. This matters most when you sign with a PDA, because `invoke_signed` lends the PDA's authority to the program you call. When the curve's address arrives as instruction data rather than as an account, check it with `pina_bonding_curve_cpi::is_expected_program`.

## Pin the deployment at compile time

`ProgramAccount` trusts whatever `PINA_BONDING_CURVE_ID` the dependency carries. Name the deployment you trust in your own source, and fail the build if the two ever differ:

```rust
use pina::Address;
use pina::address;

/// The Pina Bonding Curve deployment this program trusts: a compile-time
/// trust boundary.
const TRUSTED_CURVE_PROGRAM_ID: Address = address!("CurveqeE6jzkyHQMcWaGENzd7jrd8m9R1u4dZ17GnSa9");

const fn same_address(left: &Address, right: &Address) -> bool {
	let (left, right) = (left.as_array(), right.as_array());
	let mut index = 0;
	while index < left.len() {
		if left[index] != right[index] {
			return false;
		}
		index += 1;
	}
	true
}

// A dependency update that points the CPI crate elsewhere now fails the build.
const _: () = assert!(
	same_address(
		&pina_bonding_curve_cpi::PINA_BONDING_CURVE_ID,
		&TRUSTED_CURVE_PROGRAM_ID
	),
	"pina_bonding_curve_cpi targets an untrusted program",
);
```

`Address` equality is not a `const fn`, hence the byte loop. A swapped, vendored, or regenerated crate can no longer retarget your CPIs without a reviewed change to your own code.

## Buy and sell for a PDA

A vault or router that trades for its users keeps their tokens in accounts owned by one of its PDAs, and signs `Buy` and `Sell` as that PDA:

```rust
use pina::*;
use pina_bonding_curve_cpi::Buy;
use pina_bonding_curve_cpi::BuyIx;
use pina_bonding_curve_cpi::ProgramAccount;

/// The PDA that owns a vault's token accounts and trades for it.
#[pda(seeds = [b"trader", vault: Address])]
pub struct VaultTrader {}

/// Accounts for this program's own `BuyLaunch` instruction.
#[derive(Accounts, Debug)]
pub struct BuyLaunchAccounts<'a> {
	/// The vault this program keeps for a user.
	pub vault: &'a AccountView,
	/// The vault's `VaultTrader` PDA.
	pub trader: &'a AccountView,
	/// The launch's configuration.
	pub config: &'a AccountView,
	/// The launch.
	pub launch: &'a mut AccountView,
	/// The PDA's token account for the base mint.
	pub trader_base: &'a mut AccountView,
	/// The PDA's token account for the quote mint.
	pub trader_quote: &'a mut AccountView,
	/// The launch's base vault.
	pub base_vault: &'a mut AccountView,
	/// The launch's quote vault.
	pub quote_vault: &'a mut AccountView,
	/// Token program that owns the base mint.
	pub base_token_program: &'a AccountView,
	/// Token program that owns the quote mint.
	pub quote_token_program: &'a AccountView,
	/// The Pina Bonding Curve program.
	pub curve_program: &'a AccountView,
}

impl BuyLaunchAccounts<'_> {
	/// Spend up to `quote_amount_in` of the vault's quote. Returns the base
	/// received and the quote actually spent.
	fn buy(&self, quote_amount_in: u64, minimum_base_out: u64) -> Result<(u64, u64), ProgramError> {
		let curve = ProgramAccount::try_new(self.curve_program)?;
		self.vault.assert_owner(&crate::ID)?;
		let (trader, bump) = VaultTrader::try_find_pda(self.vault.address(), &crate::ID)
			.ok_or(ProgramError::InvalidSeeds)?;
		self.trader.assert_address(&trader)?;

		// The curve credits `trader_base` and debits `trader_quote`, and only
		// the token program looks at who owns them: require the PDA to.
		let base_before = balance_of(self.trader_base, self.base_token_program, &trader)?;
		let quote_before = balance_of(self.trader_quote, self.quote_token_program, &trader)?;

		let seeds = VaultTrader::seeds(self.vault.address()).with_bump(bump);
		Buy {
			trader: self.trader,
			config: self.config,
			launch: self.launch,
			trader_base: self.trader_base,
			trader_quote: self.trader_quote,
			base_vault: self.base_vault,
			quote_vault: self.quote_vault,
			base_token_program: self.base_token_program,
			quote_token_program: self.quote_token_program,
			ix: BuyIx {
				quote_amount_in,
				minimum_base_out,
			},
		}
		.invoke_signed(&curve, &[seeds.to_signer().as_signer()])?;

		// A buy that completes the launch spends less than `quote_amount_in`.
		let received = balance_of(self.trader_base, self.base_token_program, &trader)?
			.checked_sub(base_before)
			.ok_or(ProgramError::ArithmeticOverflow)?;
		let spent = quote_before
			.checked_sub(balance_of(
				self.trader_quote,
				self.quote_token_program,
				&trader,
			)?)
			.ok_or(ProgramError::ArithmeticOverflow)?;
		Ok((received, spent))
	}
}

/// The balance of token account `account`, which must belong to `owner`.
fn balance_of(
	account: &AccountView,
	token_program: &AccountView,
	owner: &Address,
) -> Result<u64, ProgramError> {
	let token = account.as_token_account_for_program(token_program.address())?;
	if token.owner() != owner {
		return Err(ProgramError::IllegalOwner);
	}
	Ok(token.amount())
}
```

The curve validates the launch, its configuration, its vaults, and both token programs itself. It does not check who owns `trader_base` and `trader_quote`; the token program only requires the trader to own the account it debits. Check both yourself, as `balance_of` does, or a caller can send the bought base, or a sale's quote, to an account they control.

Take the slippage limit from your user, who computes it off-chain by [simulating the trade](rust-client.md#quote-a-trade), and pass it through unchanged as `minimum_base_out` or `minimum_quote_out`. The curve enforces it in the same instruction and fails with `SlippageExceeded`; never replace a user's limit with zero. The curve sets no return data, and a `Buy` that reaches the migration price completes the launch and spends less than `quote_amount_in`, so measure what moved, as above, before you credit the user.

`Sell` takes the same accounts. Its arguments are the exact base to sell and the least quote to accept after the fee:

```rust
use pina_bonding_curve_cpi::{Sell, SellIx};

Sell {
	trader: self.trader,
	config: self.config,
	launch: self.launch,
	trader_base: self.trader_base,
	trader_quote: self.trader_quote,
	base_vault: self.base_vault,
	quote_vault: self.quote_vault,
	base_token_program: self.base_token_program,
	quote_token_program: self.quote_token_program,
	ix: SellIx {
		base_amount_in,
		minimum_quote_out,
	},
}
.invoke_signed(&curve, &[seeds.to_signer().as_signer()])?;
```

The repository's compute-unit tests hold `Buy` under 10,000 compute units and `Sell` under 11,000; budget for them on top of your own instruction's work.

## Create launches under a PDA

A program can be the creator of the launches it creates, which gives it the creator's share of fees and the vested creator allocation. The creator must sign `CreateLaunch` and must be the base mint's mint authority. `InitializeMint2` needs no signature from the mint authority, so your client creates the mint with your PDA as its authority earlier in the same transaction, and your program signs as the PDA:

```rust
use pina_bonding_curve_cpi::{CreateLaunch, CreateLaunchIx};

/// The PDA every launch this program creates names as its creator.
#[pda(seeds = [b"launcher"])]
pub struct Launcher {}

let curve = ProgramAccount::try_new(self.curve_program)?;
let (launcher, bump) = Launcher::try_find_pda(&crate::ID).ok_or(ProgramError::InvalidSeeds)?;
self.launcher.assert_address(&launcher)?;
let launcher_seeds = Launcher::seeds().with_bump(bump);

CreateLaunch {
	payer: self.payer,
	creator: self.launcher,
	config: self.config,
	base_mint: self.base_mint,
	quote_mint: self.quote_mint,
	launch: self.launch,
	base_vault: self.base_vault,
	quote_vault: self.quote_vault,
	base_token_program: self.base_token_program,
	quote_token_program: self.quote_token_program,
	system_program: self.system_program,
	ix: CreateLaunchIx { activation_time },
}
.invoke_signed(&curve, &[launcher_seeds.to_signer().as_signer()])?;
```

The payer is a transaction signer, and its signature passes through the CPI. The mint must also have zero supply, no freeze authority, and the configuration's base decimals; the curve mints the fixed supply into its own vault and then revokes the mint authority. `activation_time` is a Unix timestamp; zero or a past time opens trading at once.

From then on the PDA holds the launch's creator rights. It signs `ClaimCreatorFees` and `ClaimCreatorAllocation` to collect, and `SetLaunchCreator` to hand the rights to another address (`ix: SetLaunchCreatorIx { new_creator: &new_creator }`):

```rust
use pina_bonding_curve_cpi::{ClaimCreatorFees, ClaimCreatorFeesIx};

ClaimCreatorFees {
	claimant: self.launcher,
	config: self.config,
	launch: self.launch,
	quote_vault: self.quote_vault,
	destination: self.treasury_quote,
	quote_token_program: self.quote_token_program,
	ix: ClaimCreatorFeesIx,
}
.invoke_signed(&curve, &[launcher_seeds.to_signer().as_signer()])?;
```

The curve pays a claim into any token account for the right mint, so check that `destination` is yours as well.

## Graduate

Graduation takes no signer seeds: any payer can graduate a completed launch, and the curve checks every account. The `creator`, `creator_lp_token`, `partner`, and `partner_lp_token` fields are `Option`s; pass a pair when the configuration's matching LP share is non-zero, and `None` sends the curve's program id, which the curve reads as absent. `Graduate` itself calls the Pina AMM, which calls the token and associated token programs, so most integrations send it as a top-level instruction with the [Rust client](rust-client.md#graduate) instead of nesting it one level deeper.

## Reading curve accounts on chain

The crate's `accounts` module parses `Launch` by discriminator only. Always check ownership first:

```rust
use pina_bonding_curve_cpi::PINA_BONDING_CURVE_ID;
use pina_bonding_curve_cpi::accounts::Launch;

launch_account.assert_owner(&PINA_BONDING_CURVE_ID)?;
let launch = Launch::parse(&launch_account.try_borrow()?).ok_or(ProgramError::InvalidAccountData)?;
if launch.status != 0 {
	// 1 is completed and waiting to graduate; 2 is graduated, into the Pina AMM pool at `launch.pool`.
	return Err(ProgramError::InvalidAccountData);
}
```

`LaunchConfig` has no parser in this crate. Its sixteen-slot `u128` curve arrays are beyond what the CPI renderer decodes, so it exposes only `LaunchConfig::matches` and `LaunchConfig::LEN`, and `LaunchConfig::PARSER_UNSUPPORTED` records why. The curve validates every configuration it receives, so a caller rarely needs to read one.

## Keeping the client in sync

The CPI crate is generated from the curve's IDL. If you vendor it instead of depending on the published crate, regenerate it from the IDL of the curve version you target with `pina import pina_bonding_curve --program-id CurveqeE6jzkyHQMcWaGENzd7jrd8m9R1u4dZ17GnSa9 --idl <idl.json>`, which records the IDL hash it was built from. The compile-time assertion above then catches a regenerated crate that targets a different deployment.

This repository calls the Pina AMM the same way when a launch graduates. `programs/pina_bonding_curve/src/processors/graduate.rs` validates the AMM with `ProgramAccount::try_new` and signs `CreatePool` with two PDAs; it is a complete, tested example.
