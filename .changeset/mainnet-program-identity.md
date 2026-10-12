---
bonding_curve: major
---

# Move the program to its mainnet identity and graduate into the mainnet AMM

The program id is now `Curve1GqpqznjxpJzH313L9CRhAehHaikb4SJMFQeBvP`, so every PDA moves with it: the AMM authority that graduation tiers must restrict pool creation to is now `HUjATWLMQR85P71zHVoHgbRPTY7xx4gEdyDg7EMin7XX`. `pina migrations create` rebound the unpublished migration history to the new id; `publications.json` still has no receipts.

Graduation now calls the Pina AMM at `pAMMzVPRwtrxQs2CjLTUUpfCAp9XdvVmwwVhpAuCobi`. `amm.rev` pins the AMM commit that carries that identity, and `vendor/pina_amm` is re-imported from its IDL. The CPI, Rust, TypeScript, and Dart clients are regenerated with both ids, and `addresses.rs`, the docs, the website, and the tests name the new addresses.

The deployed binary now embeds a `security.txt` through `solana-security-txt` 1.1.3, only under `bpf-entrypoint`. It names the project, the private advisory form as contact, `SECURITY.md` as policy, and the source repository.
