---
pina_bonding_curve_cpi: fix
---

# Update Pina AMM to 0.2.1 and regenerate CPI docs

Regenerate `pina_bonding_curve_cpi` with `pina` CLI 0.25.1, adding program-level doc comments to `PINA_BONDING_CURVE_ID` with no wire or public API changes. The vendored `pina_amm` CPI crate is re-imported against `pina-rs/amm` v0.2.1 (`0ca40de5`) on `pina` 0.25, dropping the duplicate `pina 0.23.0` dependency from `Cargo.lock` so the deployed program binary shrinks by 48 bytes (`116,240 -> 116,192 B`) and `graduate` drops 16.84% in compute units (`68,581 -> 57,032 CU`). No API or source changes occur in the Rust, TypeScript, or Dart clients or the CLI.
