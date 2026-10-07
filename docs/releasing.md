# Releasing

Every package in this repository releases together as the `bonding_curve` group: the program, the CPI, Rust, TypeScript, and Dart clients, and the CLI share one version and one `v{version}` tag. [Monochange](https://github.com/monochange/monochange) plans the release from changesets.

## Two audiences, two streams

| Stream  | Audience                     | Written to                                           | Change types                              |
| ------- | ---------------------------- | ---------------------------------------------------- | ----------------------------------------- |
| default | Contributors and integrators | `changelog.md`                                       | `breaking`, `feat`, `fix`, `docs`, `none` |
| `user`  | People using the curve       | `release-notes/v{version}.md` and the GitHub release | `user`                                    |

Developer changesets can name APIs, accounts, migrations, and tests. User changesets describe what someone can now do, in plain language, under a `## User impact` heading. A change that matters to both gets **two changesets**.

## Writing changesets

```sh
devenv shell monochange run change --type feat --reason "Report the fee rate in Traded events"
devenv shell monochange run change --type user --reason "See the fee rate you paid on every trade"
```

or create the files by hand in `.changeset/`:

```markdown
---
bonding_curve: feat
---

# Report the fee rate in Traded events

`Traded` gains a `fee_rate` field (ppm) after `fee`. `pina migrations create` records it as a new event schema version, and the TypeScript, Dart, Rust, and CPI clients are regenerated to match.
```

```markdown
---
bonding_curve: user
---

# See the fee rate you paid on every trade

## User impact

Every trade now records the fee rate it paid, so you can see how much the opening fee had fallen when your buy or sale went through.
```

| Type       | Bump  | Use for                                                                  |
| ---------- | ----- | ------------------------------------------------------------------------ |
| `breaking` | major | Changed account layouts, instruction data, account lists, or client APIs |
| `feat`     | minor | New instructions, events, client functions, or CLI commands              |
| `fix`      | patch | Corrections that keep every interface                                    |
| `user`     | none  | Public wording paired with one of the above                              |
| `docs`     | none  | Documentation only                                                       |
| `none`     | none  | Tooling, CI, and tests                                                   |

CI runs `monochange affected --verify` on pull requests, so a change to a published package without a changeset fails. Check your notes locally before pushing:

```sh
devenv shell monochange check
devenv shell monochange preview
devenv shell monochange notes --output user --target bonding_curve
```

## The release flow

1. Merging to `main` runs `release-pr.yml`, which opens or refreshes a `chore(release): prepare release` pull request with the new version, `changelog.md`, and the user release notes.
2. Merging that pull request tags `v{version}`, creates a draft GitHub release from the user notes, and dispatches `publish.yml` on the tag.
3. `publish.yml` publishes the crates to crates.io, `@pina-rs/bonding-curve` to npm, and `pina_bonding_curve` to pub.dev, all through trusted publishing (OIDC), then publishes the GitHub release.

The on-chain program is never published by CI; deploy it with [deploying.md](deploying.md).

## One-time registry setup

Trusted publishing needs each package to exist on its registry and to trust this repository's `publish.yml` in the `publisher` environment. A maintainer does this once:

1. Create a `publisher` environment in the repository settings and restrict it to tags matching `v*`.
2. **crates.io**: publish `pina_bonding_curve_cpi`, `pina_bonding_curve_client`, and `pina_bonding_curve_cli` once (for example with `monochange publish placeholder`), then add `pina-rs/bonding_curve`, workflow `publish.yml`, environment `publisher` as a trusted publisher on each crate.
3. **npm**: create `@pina-rs/bonding-curve` (or publish a placeholder), then add the same GitHub trusted publisher under the package's settings.
4. **pub.dev**: publish `pina_bonding_curve` once, then enable automated publishing from GitHub Actions for `pina-rs/bonding_curve` with the tag pattern `v{{version}}` and the `publisher` environment.

After that, releases need no stored registry tokens.
