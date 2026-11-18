{ pkgs, inputs, ... }:
let
  custom = inputs.ifiokjr-nixpkgs.packages.${pkgs.stdenv.hostPlatform.system};

  # nixpkgs in devenv.lock ships pnpm 12.3.4. Pin the official 12.21.0
  # executable: `packageManager` requires that exact release. Standalone pnpm
  # (@pnpm/exe) before 10.34.8, and 11.x before 11.28.8, bundles a Node that
  # trusts compromised eMudhra certificates (GHSA-w8f5-6m3h-r453). pnpm 12 does
  # not. This binding is what `packages` uses; `with pkgs` does not override it.
  pnpmVersion = "12.21.0";
  pnpmExe =
    let
      sources = {
        x86_64-linux = {
          url = "https://registry.npmjs.org/@pnpm/exe.linux-x64/-/exe.linux-x64-${pnpmVersion}.tgz";
          hash = "sha512-8eufCjHXspKbNS/kh8q4Q0gqJoOuvC4Q5pgos6XS8By/nAMtP+jf8F1Cw9er840kfAX+PJ/ts+A9qnLE91rvXQ==";
        };
        aarch64-linux = {
          url = "https://registry.npmjs.org/@pnpm/exe.linux-arm64/-/exe.linux-arm64-${pnpmVersion}.tgz";
          hash = "sha512-4Oj/iF2TDCe2lcH0n4TupPR5sSdrNLclJB6xBElLMPCUOAvBhdf2rRlSaO5M4erJ6g4Fyy7YTXTpcmfFsKyxEA==";
        };
        x86_64-darwin = {
          url = "https://registry.npmjs.org/@pnpm/exe.darwin-x64/-/exe.darwin-x64-${pnpmVersion}.tgz";
          hash = "sha512-EzpvcsaE6hrrgYeEiAFYd2rSgFybYyxXhkAY7K3y6Hg3loVfcyJ81hHczpFzi8lW5RXtUlFECRPZ8ZAMRFQYYg==";
        };
        aarch64-darwin = {
          url = "https://registry.npmjs.org/@pnpm/exe.darwin-arm64/-/exe.darwin-arm64-${pnpmVersion}.tgz";
          hash = "sha512-F5OyMX/6zLSowpTZEQRojbCmPS1Pq+fyRaIinslsIe6Glh3qbyQ52u+Fe0eEzSaRuIbWZkXmAAXSWh6Q5Wrlgg==";
        };
      };
      system = pkgs.stdenv.hostPlatform.system;
    in
    sources.${system} or (throw "pnpm ${pnpmVersion} has no binary for ${system}");
  pnpm = pkgs.stdenv.mkDerivation {
    pname = "pnpm";
    version = pnpmVersion;
    src = pkgs.fetchurl pnpmExe;
    sourceRoot = "package";

    nativeBuildInputs =
      pkgs.lib.optionals pkgs.stdenv.hostPlatform.isLinux [ pkgs.autoPatchelfHook ]
      ++ [ pkgs.makeWrapper ];
    buildInputs = pkgs.lib.optionals pkgs.stdenv.hostPlatform.isLinux [ pkgs.stdenv.cc.cc.lib ];

    dontBuild = true;
    dontConfigure = true;
    dontStrip = true;

    installPhase = ''
      runHook preInstall
      install -Dm755 pnpm $out/bin/pnpm
      makeWrapper $out/bin/pnpm $out/bin/pnpx --add-flag dlx
      ln -s pnpm $out/bin/pn
      ln -s pnpx $out/bin/pnx
      runHook postInstall
    '';

    meta = {
      description = "Fast, disk space efficient package manager for JavaScript";
      homepage = "https://pnpm.io/";
      license = pkgs.lib.licenses.mit;
      mainProgram = "pnpm";
      platforms = [
        "x86_64-linux"
        "aarch64-linux"
        "x86_64-darwin"
        "aarch64-darwin"
      ];
    };
  };
in
{
  packages = with pkgs; [
    actionlint
    cargo-deny
    curl
    custom.agave
    custom.monochange
    custom.pina
    dart
    dprint
    git
    libiconv
    nixfmt
    nodejs_24
    pnpm
    rustup
    zlib
  ];

  env = {
    PINA_BPF_TOOLCHAIN = "nightly-2025-11-20";
    SBF_TOOLS_VERSION = "v1.54";
  };

  scripts = {
    # Build the Pina AMM at the revision pinned in `amm.rev`. The end-to-end
    # suite deploys it beside the curve, and `generate:clients` re-imports the
    # vendored AMM CPI crate from its IDL. Set PINA_AMM_SOURCE to a local
    # checkout to build that instead.
    "fetch:amm".exec = ''
      set -euo pipefail
      revision="$(tr -d '[:space:]' < amm.rev)"
      source="''${PINA_AMM_SOURCE:-$PWD/target/pina_amm_src}"
      if [ -z "''${PINA_AMM_SOURCE:-}" ]; then
        if [ ! -d "$source/.git" ]; then
          git clone --quiet https://github.com/pina-rs/amm.git "$source"
        fi
        git -C "$source" fetch --quiet origin "$revision"
        git -C "$source" checkout --quiet --detach "$revision"
      fi
      CARGO_TARGET_DIR="$PWD/target/pina_amm_build" RUST_LOG=error \
        pina build --project "$source/programs/pina_amm"
      mkdir -p target/deploy target/idl
      cp target/pina_amm_build/deploy/pina_amm.so target/deploy/pina_amm.so
      cp target/pina_amm_build/idl/pina_amm.json target/idl/pina_amm.json
    '';

    "install:all".exec = ''
      set -euo pipefail
      pnpm install --frozen-lockfile
      (cd clients/dart && dart pub get)
    '';

    "build:program".exec = ''
      set -euo pipefail
      RUST_LOG=error pina build --project programs/pina_bonding_curve
    '';

    "build:clients".exec = ''
      set -euo pipefail
      cargo build --workspace --locked
      pnpm --dir clients/typescript/pina_bonding_curve build
    '';

    # Regenerate every client from the program IDL. Generated sources are
    # never edited by hand; manifests and entrypoints are preserved.
    "generate:clients".exec = ''
      set -euo pipefail
      pina migrations check --project programs/pina_bonding_curve
      # Pina refuses to write through symlinks, and pnpm links the TypeScript
      # client's dependencies into its directory. Remove them first and
      # reinstall afterwards.
      rm -rf -- clients/typescript/pina_bonding_curve/node_modules
      pina generate --project programs/pina_bonding_curve --output clients --npx node
      if [ -f target/idl/pina_amm.json ]; then
        pina import pina_amm \
          --program-id pAMMzVPRwtrxQs2CjLTUUpfCAp9XdvVmwwVhpAuCobi \
          --idl target/idl/pina_amm.json \
          --output vendor >/dev/null
      fi
      pnpm install --frozen-lockfile >/dev/null
      node tools/normalize-generated.mjs clients/dart/lib
      dart format clients/dart >/dev/null
      dprint fmt --allow-no-files 'clients/**/*.{ts,json,toml}' >/dev/null
    '';

    "check:clients".exec = ''
      set -euo pipefail
      fetch:amm
      generate:clients
      if ! git diff --quiet -- clients vendor; then
        echo "Generated clients drifted from the program. Run generate:clients and commit the result." >&2
        git --no-pager diff --stat -- clients vendor >&2
        exit 1
      fi
    '';

    "test:unit".exec = ''
      set -euo pipefail
      node --test tools/normalize-generated.test.mjs tools/client-guides.test.mjs
      cargo test --workspace --all-features --locked
      pnpm --dir clients/typescript/pina_bonding_curve build
      pnpm --dir clients/typescript/pina_bonding_curve test
      pnpm --dir website test
      (cd clients/dart && dart test)
    '';

    # The end-to-end suite deploys the real SBF artifact to an offline
    # Surfnet and drives it through the generated Rust client and the CLI.
    "test:surfpool".exec = ''
      set -euo pipefail
      build:program
      fetch:amm
      cargo build -p pina_bonding_curve_cli --locked
      PINA_SBF_ARTIFACT="$PWD/target/deploy/pina_bonding_curve.so" \
        PINA_AMM_ARTIFACT="$PWD/target/deploy/pina_amm.so" \
        PINA_CURVE_CLI="$PWD/target/debug/pina-curve" \
        cargo test \
          --manifest-path programs/pina_bonding_curve/tests/surfpool/Cargo.toml \
          --locked \
          -- \
          --ignored \
          --nocapture
    '';

    "lint:all".exec = ''
      set -euo pipefail
      pina migrations check --project programs/pina_bonding_curve
      if [ "$(uname -s)-$(uname -m)" = "Darwin-x86_64" ]; then
        echo "Skipping Pina security lints: no Intel macOS lint driver is published."
      else
        rustup toolchain install --profile minimal
        if [ "$(uname -s)" = "Darwin" ]; then
          # Pina's lint driver puts the rustup toolchain's libLLVM on the dyld
          # path, which the Nix clang linker then loads and aborts on. Link
          # host build scripts with the system linker for this call only.
          CARGO_TARGET_AARCH64_APPLE_DARWIN_LINKER=/usr/bin/cc \
            pina lint --project programs/pina_bonding_curve
        else
          pina lint --project programs/pina_bonding_curve
        fi
      fi
      cargo clippy --workspace --all-features --all-targets --locked -- -D warnings
      cargo clippy \
        --manifest-path programs/pina_bonding_curve/tests/surfpool/Cargo.toml \
        --all-targets \
        --locked \
        -- \
        -D warnings
      RUSTDOCFLAGS="-D warnings" cargo doc --workspace --all-features --no-deps --locked
      dprint check
      actionlint
      pnpm --dir clients/typescript/pina_bonding_curve check
      pnpm --dir website check
      (cd clients/dart && dart analyze --fatal-infos --fatal-warnings)
      monochange check
    '';

    # bonding-curve.pina.rs: the landing page plus docs/ rendered as the
    # documentation. See website/README.md.
    "dev:website".exec = ''
      set -euo pipefail
      pnpm --dir website dev
    '';

    "build:website".exec = ''
      set -euo pipefail
      pnpm --dir website build
    '';

    # Builds and deploys the Cloudflare Worker. Needs CLOUDFLARE_API_TOKEN and
    # CLOUDFLARE_ACCOUNT_ID; the website workflow runs it on every push to main.
    "deploy:website".exec = ''
      set -euo pipefail
      # `run` is required: `pnpm deploy` is a built-in that shadows the script.
      pnpm --dir website run deploy
    '';

    "fix:format".exec = ''
      set -euo pipefail
      dprint fmt
      dart format clients/dart >/dev/null
    '';

    "security:audit".exec = ''
      set -euo pipefail
      cargo deny check -D warnings advisories bans sources
      pnpm audit --audit-level high
    '';

    # Everything CI runs, in the order it fails fastest.
    "verify:all".exec = ''
      set -euo pipefail
      lint:all
      check:clients
      build:website
      test:unit
      test:surfpool
      security:audit
    '';

    # Manual release step: never call from another script, hook, or CI job.
    # `pina deploy` records the migration publication receipt that freezes the
    # deployed schema versions; never deploy with `solana program deploy`
    # directly. See docs/deploying.md for the runbook.
    "deploy:program".exec = ''
      set -euo pipefail
      cluster="''${1:?usage: deploy:program <devnet|mainnet-beta> [extra pina deploy flags]}"
      shift
      pina deploy \
        --project programs/pina_bonding_curve \
        --build \
        --cluster "$cluster" \
        --program-keypair "''${PINA_CURVE_PROGRAM_KEYPAIR:?set PINA_CURVE_PROGRAM_KEYPAIR to the program-id keypair file}" \
        --upgrade-authority "''${PINA_CURVE_UPGRADE_AUTHORITY:?set PINA_CURVE_UPGRADE_AUTHORITY to the upgrade-authority keypair file}" \
        --payer "''${PINA_CURVE_DEPLOY_KEYPAIR:?set PINA_CURVE_DEPLOY_KEYPAIR to the fee-payer keypair file}" \
        "$@"
    '';
  };

  enterShell = ''
    export PATH="$PWD/node_modules/.bin:$PATH"
  '';
}
