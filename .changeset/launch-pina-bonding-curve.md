---
bonding_curve:
  bump: minor
  type: feat
  version: "0.1.0"
---

# Launch the Pina Bonding Curve program, clients, and CLI

The first release of the Pina Bonding Curve: self-serve launchpads built with Pina 0.23 that graduate into the Pina AMM.

- **Program** (`CurveqeE6jzkyHQMcWaGENzd7jrd8m9R1u4dZ17GnSa9`): immutable partner configurations with up to 16 concentrated-liquidity curve segments in Q64.64 square-root prices, fixed-supply launches that revoke the mint authority, buys and sells with a linearly decaying anti-sniping fee split between creator and partner, completion at the derived migration price with partial fills, permissionless graduation into a Pina AMM tier restricted to the curve's AMM authority PDA (price-continuous seeding, migration fee, LP payout or burn, surplus base burn), vested creator allocations, and fee claims. A buy costs about 8,400 compute units.
- **`pina_bonding_curve_client`**, **`pina_bonding_curve_cpi`**, **`@pina-rs/bonding-curve`**, and **`pina_bonding_curve`** (Dart): generated from the program IDL with the version-0 migration baseline.
- **`pina_bonding_curve_cli`**: the `pina-curve` binary, which designs curves offline, previews configurations by simulation, quotes trades by simulation, applies slippage limits, and wraps SOL for wrapped-SOL launches.

The Pina AMM dependency is pinned in `amm.rev` and vendored as a generated CPI crate. Unit and property tests cover the curve math and rounding; an end-to-end Surfpool suite deploys the curve beside the pinned AMM, drives every instruction through the generated Rust client and the CLI, and enforces compute-unit ceilings.
