#!/usr/bin/env bash
# Bootstrap a Cursor cloud agent so it can build, lint, and test this repo.
#
# The project runs through the locked devenv (Nix). This VM image has no
# systemd, so Nix is installed single-user. The script is idempotent: a second
# run reuses Nix, the pinned devenv, and already-installed packages.
#
# CI installs the same devenv revision from devenv.lock and the same Cachix
# cache. See .github/workflows/ci.yml.

set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

export USER="${USER:-$(id -un)}"
export HOME="${HOME:-$(getent passwd "$USER" | cut -d: -f6)}"

nix_sh="$HOME/.nix-profile/etc/profile.d/nix.sh"

load_nix() {
	if [ ! -f "$nix_sh" ]; then
		return 1
	fi
	# nix.sh does nothing unless both HOME and USER are set.
	# shellcheck disable=SC1090
	. "$nix_sh"
}

install_nix() {
	if load_nix && command -v nix >/dev/null 2>&1; then
		return 0
	fi
	echo "Installing single-user Nix (no systemd on this image)..."
	curl --proto '=https' --tlsv1.2 -fsSL https://nixos.org/nix/install |
		sh -s -- --no-daemon --yes --no-channel-add
	load_nix
	command -v nix >/dev/null 2>&1
}

write_nix_conf() {
	local dir="$HOME/.config/nix"
	local conf="$dir/nix.conf"
	mkdir -p "$dir"
	# Keep the substituter and flake settings aligned with CI.
	cat >"$conf" <<'EOF'
experimental-features = nix-command flakes
accept-flake-config = true
substituters = https://cache.nixos.org https://devenv.cachix.org
trusted-public-keys = cache.nixos.org-1:6NCHdD59X431o0gWypbMrAURkbJ16ZPMQFGspcDShjY= devenv.cachix.org-1:w1cLUi8dv3hnoSPGAuibQv+f9TZLr6cv/Hm9XgU50cw=
EOF
}

install_profile_hook() {
	# Login shells (install and start run as `bash -lc`) source /etc/profile.d.
	local hook=/etc/profile.d/nix-single-user.sh
	local tmp
	tmp="$(mktemp)"
	cat >"$tmp" <<'EOF'
# Loaded for login shells. Single-user Nix lives in the invoking user's profile.
export USER="${USER:-$(id -un)}"
if [ -n "${HOME:-}" ] && [ -f "$HOME/.nix-profile/etc/profile.d/nix.sh" ]; then
	. "$HOME/.nix-profile/etc/profile.d/nix.sh"
fi
EOF
	sudo tee "$hook" >/dev/null <"$tmp"
	sudo chmod 644 "$hook"
	rm -f "$tmp"
}

install_wrappers() {
	# Non-interactive shells do not source ~/.bashrc or ~/.profile. Wrappers
	# on the default PATH load Nix, then exec the real binaries.
	local name target
	for name in nix devenv; do
		target="$HOME/.nix-profile/bin/$name"
		sudo tee "/usr/local/bin/$name" >/dev/null <<EOF
#!/usr/bin/env bash
export USER="\${USER:-\$(id -un)}"
export HOME="\${HOME:-/home/\$USER}"
if [ -f "\$HOME/.nix-profile/etc/profile.d/nix.sh" ]; then
	. "\$HOME/.nix-profile/etc/profile.d/nix.sh"
fi
exec "\$HOME/.nix-profile/bin/${name}" "\$@"
EOF
		sudo chmod 755 "/usr/local/bin/$name"
		if [ ! -x "$target" ] && [ "$name" = "nix" ]; then
			echo "Nix binary missing at $target" >&2
			exit 1
		fi
	done
}

install_devenv() {
	local rev stamp
	rev="$(
		nix --extra-experimental-features 'nix-command flakes' eval --impure --raw --expr '
			let lock = builtins.fromJSON (builtins.readFile ./devenv.lock);
			in lock.nodes.devenv.locked.rev
		'
	)"
	stamp="$HOME/.local/share/pina-cloud-agent/devenv-rev"
	if [ -x "$HOME/.nix-profile/bin/devenv" ] && [ "$(cat "$stamp" 2>/dev/null || true)" = "$rev" ]; then
		echo "devenv $rev already installed"
		return 0
	fi
	echo "Installing devenv $rev..."
	if nix profile list 2>/dev/null | grep -q 'devenv'; then
		nix profile remove '.*devenv.*' || nix profile remove 0
	fi
	nix profile add --accept-flake-config "github:cachix/devenv/${rev}#devenv"
	mkdir -p "$(dirname "$stamp")"
	printf '%s\n' "$rev" >"$stamp"
}

install_nix
write_nix_conf
install_profile_hook
install_devenv
install_wrappers

echo "Installing workspace JavaScript and Dart dependencies..."
devenv shell -- install:all

echo "Cloud agent install complete."
