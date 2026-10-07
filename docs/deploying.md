# Deploying

This is the runbook for deploying the program and preparing the Pina AMM tier its launches graduate into. Deploying is a manual, deliberate step: no script, hook, or CI job deploys automatically.

## Keys

| Key               | Purpose                                                               | Where it lives                                                                           |
| ----------------- | --------------------------------------------------------------------- | ---------------------------------------------------------------------------------------- |
| Program keypair   | The program's address, `CurveqeE6jzkyHQMcWaGENzd7jrd8m9R1u4dZ17GnSa9` | Offline backup; locally at `target/deploy/pina_bonding_curve-keypair.json` (git-ignored) |
| Upgrade authority | Upgrades the program                                                  | A hardware wallet or multisig                                                            |
| Fee payer         | Pays deployment rent and fees                                         | Any funded wallet                                                                        |

Never commit a keypair. `pina keys show` compares the local keypair with `declare_id!` in `programs/pina_bonding_curve/src/lib.rs`. Unlike the AMM, the curve's upgrade authority has no other powers: there are no admin instructions.

## Deploy

The curve calls the Pina AMM during graduation, so deploy against a cluster where the AMM at `pAMMvXaqR2VVFqznf6dgvUFQjLXXAEeL9cb48cXsGeV` is live.

```sh
export PINA_CURVE_PROGRAM_KEYPAIR=~/secure/pina_bonding_curve-keypair.json
export PINA_CURVE_UPGRADE_AUTHORITY=~/secure/upgrade-authority.json
export PINA_CURVE_DEPLOY_KEYPAIR=~/.config/solana/id.json

devenv shell deploy:program devnet
devenv shell deploy:program mainnet-beta --allow-mainnet
```

`deploy:program` wraps `pina deploy --build`, which builds the artifact, prints the complete plan, asks for confirmation, and **records the migration publication** in `programs/pina_bonding_curve/migrations/publications.json`. Commit that file after every deployment: it freezes the schema versions now live on chain, so a later change to an account layout must ship a migration instead of silently rewriting history. Never deploy with `solana program deploy` directly.

Before upgrading a live deployment, rehearse it against recent traffic:

```sh
devenv shell deploy:program mainnet-beta --allow-mainnet --rehearse
```

## The AMM tier

Every configuration names a Pina AMM fee tier restricted to the curve's AMM authority PDA. Only the AMM's upgrade authority can create tiers, so whoever operates the AMM deployment creates one (or a few, with different fee levels) for the curve:

```sh
# The curve's AMM authority: PDA [b"amm_authority"] under the curve program.
pina-amm -u mainnet -k ~/secure/amm-upgrade-authority.json \
  config create --index 100 --trade-fee-rate 2500 --protocol-fee-rate 160000 \
  --creator-fee-rate 5000 --pool-creator-authority ALk5JUXnbaYwVeHG4ymAVVfPKW1xaUTCrSyrvKiq7CGZ \
  --authority <TIER_MULTISIG>
```

The PDA is deterministic. Derive it with any client (`findAmmAuthorityPda()` in TypeScript) or with the Solana CLI:

```console
$ solana find-program-derived-address CurveqeE6jzkyHQMcWaGENzd7jrd8m9R1u4dZ17GnSa9 string:amm_authority
ALk5JUXnbaYwVeHG4ymAVVfPKW1xaUTCrSyrvKiq7CGZ
```

A deployment with a different program id has a different PDA.

The tier's trade, protocol, and creator fee rates become every graduated pool's fees. Partners then pass the tier's address (printed by `pina-amm config show --index 100`) as `--amm-config`, and `CreateConfig` rejects any tier that is not restricted to the curve.

## Your first launchpad

With the program deployed and a tier available, create a configuration (see [launchpads.md](launchpads.md)) and launch a test token on devnet:

```sh
pina-curve -u devnet config create --index 0 --quote-mint So11111111111111111111111111111111111111112 \
  --amm-config <AMM_TIER> --terms terms.json
pina-curve -u devnet launch create --config <CONFIG>
pina-curve -u devnet buy --base-mint <MINT> --quote-amount 100000000
pina-curve -u devnet launch show --base-mint <MINT>
```

## Verify

```sh
pina-curve -u mainnet config show --config <CONFIG>
pina build --verify     # deterministic build to compare with the deployed bytes
```

## Running your own deployment

Nothing in the program is tied to this repository's deployment. To run a separate instance, generate a new identity with `pina keys new`, run `pina migrations create --project programs/pina_bonding_curve` to rebind the migration history to the new address, regenerate the clients, and deploy. The AMM program id is fixed in the vendored CPI crate, which `generate:clients` imports with `pina import --program-id`. To graduate into your own AMM deployment, change that id in `devenv.nix`, point `PINA_AMM_SOURCE` at your AMM checkout, run `devenv shell fetch:amm` and `devenv shell generate:clients`, and create the restricted tier in your AMM.
