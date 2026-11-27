---
bonding_curve: patch
---

# Track Pina AMM 0.2.1 and Pina CLI 0.25.1

Update `amm.rev` to `pina-rs/amm` v0.2.1 (`0ca40de5`) and pin a dedicated `ifiokjr-nixpkgs-pina` input at `d224cba7` so the dev shell uses `pina` 0.25.1 while keeping `agave` 4.2.2 and `monochange` 0.21.0 unchanged. Re-importing `vendor/pina_amm` with `pina 0.25.1` drops the duplicate `pina 0.23.0` dependency from `Cargo.lock` and propagates top-level program docs into `pina_bonding_curve_cpi` without changing any client wire format or API.
