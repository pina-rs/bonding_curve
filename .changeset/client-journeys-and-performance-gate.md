---
bonding_curve: minor
---

# Client journeys and a performance gate

- Every instruction now has one ordered journey on a live Surfnet through the generated Rust client (`every_instruction_runs_in_one_journey`) and through the generated TypeScript client (`typescript_client_runs_every_instruction`), which the Rust side drives with funded fixtures — the restricted AMM tier, both mints, and every derived address — and then checks the resulting on-chain state, so the TypeScript client's wire format is proven end to end against the deployed program beside the pinned AMM. The TypeScript journey asserts its PDA helpers reproduce the Rust-derived addresses before sending anything.
- The journeys' keypairs are seeded, so identical binaries measure identical compute units.
- A `performance` workflow benchmarks the pull request against its base (each side against its own `amm.rev`): deployed binary size plus every instruction's compute units (median of three journey runs), posted as one consolidated PR comment. The policy fails the check on any binary-size increase or a compute-unit regression above 2%, unless the pull request carries `performance-approved`. `scripts/benchmark.ts` and `scripts/compare-benchmarks.ts` run the same pipeline locally.
- `docs/performance.md` documents the pipeline and the policy.
