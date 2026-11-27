#!/usr/bin/env bash
# #157: checks that no network library is among the app's dependencies (promise «keine Netzwerkzugriffe»).
# #309: the only exception is the search for updates of the desktop app (only with consent): in vivaplan-app the
# plugin tauri-plugin-updater may be a direct dependency, and the network libraries below it are allowed. The engine
# and the command line (with `vivaplan mcp`) stay without any network library, the updater included.
#
# Usage:  tools/check-network-free.sh                    cargo tree for the three packages and all desktop targets
#         tools/check-network-free.sh `--liste` FILE      checks a given list in cargo tree format (test mode, strict)
#         tools/check-network-free.sh `--app-liste` FILE  the same with the exception of the app (test mode)
#
# Lists carry the depth in front of the name (`cargo tree --prefix depth`): `1tauri-plugin-updater v2.0.0`; lines
# without a depth count as depth 0. Below the exception (greater depth) every crate is allowed; a crate repeated
# elsewhere with `(*)` is checked there again.
#
# Only normal and build dependencies are checked (-e normal,build), no dev dependencies.
# Targets: the desktop platforms that are built (pakete.yml), x86_64 and aarch64 each. Not `--target all`:
# tauri pulls in reqwest and hyper for Android/iOS; mobile devices are not built.
# Comparison by exact crate names (first field per line), no substrings: `dispatch2` does not match `h2`.
# Exit code: 0 no hit, 1 hits, 2 usage or cargo error.
set -euo pipefail

BLOCKLIST=(
  reqwest hyper hyper-util hyper-tls hyper-rustls h2 h3 ureq curl curl-sys isahc surf attohttpc
  tungstenite tokio-tungstenite async-tungstenite
  tauri-plugin-http tauri-plugin-updater tauri-plugin-websocket tauri-plugin-upload
)
PACKAGES=(vivaplan-app vivaplan-engine vivaplan-cli)
TARGETS=(
  x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu
  x86_64-pc-windows-msvc aarch64-pc-windows-msvc
  x86_64-apple-darwin aarch64-apple-darwin
)

# #309: allowed as direct dependency of the app only, together with everything below it.
APP_EXCEPTION=tauri-plugin-updater

# Reads lines `[depth]name vX.Y.Z …` from stdin and reports every blocklist hit exactly once.
# $1: crate allowed as direct dependency (depth 1) with its subtree, empty = strict.
check() {
  awk -v blocklist="${BLOCKLIST[*]}" -v allowed="${1:-}" '
    BEGIN { n = split(blocklist, s, " "); for (i = 1; i <= n; i++) blocked[s[i]] = 1; under = -1 }
    NF {
      line = $0; depth = 0
      if (match(line, /^[0-9]+/)) { depth = substr(line, 1, RLENGTH) + 0; line = substr(line, RLENGTH + 1) }
      split(line, f, " "); name = f[1]
      if (under >= 0 && depth > under) next
      under = -1
      if (allowed != "" && name == allowed && depth == 1) { under = depth; next }
      if ((name in blocked) && !(name in reported)) { reported[name] = 1; print name; hit = 1 }
    }
    END { exit hit ? 1 : 0 }
  '
}

hits=""
if [ "${1:-}" = "--liste" ] || [ "${1:-}" = "--app-liste" ]; then
  [ $# -eq 2 ] && [ -r "$2" ] || { echo "Aufruf: $0 --liste|--app-liste DATEI" >&2; exit 2; }
  allowed=""
  [ "$1" = "--app-liste" ] && allowed=$APP_EXCEPTION
  hits=$(check "$allowed" < "$2") || true
else
  [ $# -eq 0 ] || { echo "Aufruf: $0 [--liste|--app-liste DATEI]" >&2; exit 2; }
  cd "$(git rev-parse --show-toplevel)"
  for target in "${TARGETS[@]}"; do
    for package in "${PACKAGES[@]}"; do
      part=$(cargo tree -p "$package" -e normal,build --target "$target" --prefix depth --format '{p}') \
        || { echo "cargo tree fehlgeschlagen ($package, $target)" >&2; exit 2; }
      allowed=""
      [ "$package" = vivaplan-app ] && allowed=$APP_EXCEPTION
      hits+=$'\n'$(printf '%s\n' "$part" | check "$allowed") || true
    done
  done
fi
hits=$(printf '%s\n' $hits | sort -u)

if [ -z "$hits" ]; then
  echo "Netzwerkfreiheit: keine gesperrte Bibliothek gefunden."
else
  echo "Netzwerkfreiheit verletzt, gesperrte Bibliotheken in den Abhängigkeiten:" >&2
  printf '  %s\n' $hits >&2
  echo "Herkunft zeigen: cargo tree -e normal,build --target <Ziel> -i <Crate>" >&2
  exit 1
fi
