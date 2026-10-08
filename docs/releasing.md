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

### The release-publish check

Every pull request that adds or changes a changeset, and the release pull request itself, runs CI's `release-publish` job. It creates the release commit the merge would produce on a disposable runner, checks each package's registry readiness, and dry-runs the publication. A release that could not publish fails there, before any tag exists. Run the same checks locally:

```sh
devenv shell monochange step publish-readiness --from HEAD --output /tmp/readiness.json
devenv shell monochange step publish-packages --dry-run --all
```

The on-chain program is never published by CI; deploy it with [deploying.md](deploying.md).

## One-time registry setup

Trusted publishing can only be configured on a package that already exists, so each new package needs a placeholder before its first release. Until then `release-publish` reports the package as `blocked`. A registry owner does this once, before merging the first release pull request:

1. Create a `publisher` environment in the repository settings and restrict it to tags matching `v*`.
2. Preview, then publish, the `0.0.0` placeholders with the owner's own registry credentials (`cargo login`, `npm login`, and `dart pub login`):

   ```sh
   devenv shell monochange step placeholder-publish --dry-run
   devenv shell monochange step placeholder-publish
   ```

   This reserves `pina_bonding_curve_cpi`, `pina_bonding_curve_client`, and `pina_bonding_curve_cli` on crates.io, `@pina-rs/bonding-curve` on npm, and `pina_bonding_curve` on pub.dev.
3. Register the trusted publisher on each package: repository `pina-rs/bonding_curve`, workflow `publish.yml`, environment `publisher`.
   - **crates.io**: each crate's settings, under Trusted Publishing.
   - **npm**: the package's settings, under Trusted Publisher (GitHub Actions), with **Allow npm publish** ticked. The workflow publishes with `npm publish`, which a stage-only publisher rejects. From a terminal, use npm 12 or later: `npm trust github @pina-rs/bonding-curve --file publish.yml --repo pina-rs/bonding_curve --env publisher --allow-publish --allow-stage-publish`. npm 11 cannot set these permissions and fails with a bare `400`.
   - **pub.dev**: the package's admin tab, enabling publishing from GitHub Actions with the tag pattern `v{{version}}` and the `publisher` environment.
4. Re-run `release-publish` on the release pull request and merge it once it passes.

After that, releases need no stored registry tokens. Agents never run the real placeholder or release publish, and never use a maintainer's registry credentials; they stop at the dry-runs above.
