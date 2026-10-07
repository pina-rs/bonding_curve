# `pina_bonding_curve_client`

Rust client for the [Pina Bonding Curve](https://github.com/pina-rs/bonding_curve), a Solana launchpad that sells a new token along a configurable price curve and then graduates it into the [Pina AMM](https://github.com/pina-rs/amm). Generated from the program's IDL: instruction builders, account decoders, PDA helpers, events, and error codes.

```toml
[dependencies]
pina_bonding_curve_client = "0.1"
```

```rust
use pina_bonding_curve_client::instructions::{Buy, BuyInstructionData};

let data = BuyInstructionData::new(|data| {
	data.quote_amount_in.set(1_000_000_000);
	data.minimum_base_out.set(990_000);
})?;
let instruction = Buy::new(
	trader, config, launch, trader_base, trader_quote, base_vault, quote_vault,
	base_token_program, quote_token_program,
)
.instruction(data);
```

Decoders check discriminators and schema versions; check an account's owner against `PINA_BONDING_CURVE_ID` before trusting it. Full guide, including quoting trades and graduating launches: [docs/rust-client.md](https://github.com/pina-rs/bonding_curve/blob/main/docs/rust-client.md).
