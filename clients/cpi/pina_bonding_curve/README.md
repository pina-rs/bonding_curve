# `pina_bonding_curve_cpi`

`no_std` CPI client for calling the [Pina Bonding Curve](https://github.com/pina-rs/bonding_curve) from another Solana program built with [Pina](https://github.com/pina-rs/pina). Generated from the program's IDL.

```toml
[dependencies]
pina_bonding_curve_cpi = "0.1"
```

```rust
use pina_bonding_curve_cpi::{Buy, BuyIx, ProgramAccount};

let curve = ProgramAccount::try_new(self.curve_program)?;
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
	ix: BuyIx { quote_amount_in, minimum_base_out },
}
.invoke_signed(&curve, &[trader_seeds.to_signer().as_signer()])?;
```

Full guide, including the account checks a calling program must make and creating launches under a PDA: [docs/cpi.md](https://github.com/pina-rs/bonding_curve/blob/main/docs/cpi.md).
