# Repository agent instructions

## Pre-release protocol policy

This repository is pre-release. The program has no release tag and no supported deployment.

- Breaking changes are allowed while the protocol is designed. Use canonical, unversioned names; do not keep compatibility aliases for unreleased designs.
- This policy ends once a `v*` tag exists **and** the program is live as a supported deployment. After that, incompatible changes need an explicit compatibility plan approved by the maintainer.

## Program changes

- Run commands through `devenv shell <script>`; see `devenv.nix` for the full list.
- After changing any account, instruction, or event, run `pina migrations create --project programs/pina_bonding_curve`, then `devenv shell generate:clients`, and commit `migrations/` with the regenerated clients.
- Never edit files under `clients/**/generated/`, `clients/dart/lib/src/generated/`, `.pina-generated.json`, `vendor/pina_amm/`, or `migrations/` by hand. Client manifests and entrypoints (`Cargo.toml`, `package.json`, `pubspec.yaml`, `src/lib.rs`, `src/index.ts`) are hand-owned. `tools/normalize-generated.mjs` is the only post-processing step, and it has its own test.
- Every hand-written public item and field carries a doc comment. Error variants keep a one-line doc comment: generated clients use that line as the error message.
- Doc comments on accounts, instructions, and events flow into every generated client. Use plain code spans there, never rustdoc intra-doc links.
- Keep arithmetic checked and rounded in the curve's favour. Add a property test for any new formula; `reserves_always_cover_the_curve` must keep passing.
- Any change to compute cost must keep `compute_units_stay_within_budget` passing; raise a ceiling only with a measured reason.

## The Pina AMM dependency

Graduation calls the [Pina AMM](https://github.com/pina-rs/amm). `amm.rev` pins the AMM commit; `vendor/pina_amm` is its CPI crate, imported from that commit's IDL.

- To move to a newer AMM, update `amm.rev`, then run `devenv shell fetch:amm` and `devenv shell generate:clients`, and commit `amm.rev` with the re-imported `vendor/pina_amm`.
- Set `PINA_AMM_SOURCE` to a local AMM checkout to build against unmerged AMM changes; never commit a revision that is not on the AMM's `main`.
- The end-to-end suite deploys the pinned AMM beside the curve, so graduation is always tested against the real program.

## Verification before a pull request

```sh
devenv shell lint:all        # Pina security lints, clippy, rustdoc, dprint, actionlint, tsc, dart analyze, monochange
devenv shell check:clients   # generated clients and the vendored AMM crate match their sources
devenv shell test:unit
devenv shell test:surfpool   # real SBF artifacts on an offline Surfnet, including the CLI
```

## Release intent

Every pull request that changes a published package adds a changeset in `.changeset/`. User-visible changes add a second `user` changeset with a `## User impact` section in plain language. See `docs/releasing.md`.

## Secrets

Never commit keypairs. Program and upgrade-authority keypairs live outside the repository; `*-keypair.json` is ignored.
