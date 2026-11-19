#!/usr/bin/env bash
# #157: checks that no network library is among the app's dependencies (promise «keine Netzwerkzugriffe»).
#
# Usage:  tools/check-network-free.sh               cargo tree for the three packages and all desktop targets
#         tools/check-network-free.sh `--liste` FILE  checks a given list in cargo tree format (test mode)
#
# Only normal and build dependencies are checked (-e normal,build), no dev dependencies.
# Targets: the desktop platforms that are built (pakete.yml), x86_64 and aarch64 each. Not `--target all`:
# tauri pulls in reqwest and hyper for Android/iOS; mobile devices are not built.
# Comparison by exact crate names (first field per line), no substrings: `dispatch2` does not match `h2`.
# Exit code: 0 no hit, 1 hits, 2 usage or cargo error.
set -euo pipefail

SPERRLISTE=(
  reqwest hyper hyper-util hyper-tls hyper-rustls h2 h3 ureq curl curl-sys isahc surf attohttpc
  tungstenite tokio-tungstenite async-tungstenite
  tauri-plugin-http tauri-plugin-updater tauri-plugin-websocket tauri-plugin-upload
)
PAKETE=(vivaplan-app vivaplan-engine vivaplan-cli)
ZIELE=(
  x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu
  x86_64-pc-windows-msvc aarch64-pc-windows-msvc
  x86_64-apple-darwin aarch64-apple-darwin
)

# Reads lines `name vX.Y.Z …` from stdin and reports every blocklist hit exactly once.
pruefe() {
  awk -v sperre="${SPERRLISTE[*]}" '
    BEGIN { n = split(sperre, s, " "); for (i = 1; i <= n; i++) gesperrt[s[i]] = 1 }
    NF && ($1 in gesperrt) && !($1 in gemeldet) { gemeldet[$1] = 1; print $1; treffer = 1 }
    END { exit treffer ? 1 : 0 }
  '
}

if [ "${1:-}" = "--liste" ]; then
  [ $# -eq 2 ] && [ -r "$2" ] || { echo "Aufruf: $0 --liste DATEI" >&2; exit 2; }
  liste=$(cat "$2")
else
  [ $# -eq 0 ] || { echo "Aufruf: $0 [--liste DATEI]" >&2; exit 2; }
  cd "$(git rev-parse --show-toplevel)"
  liste=""
  for ziel in "${ZIELE[@]}"; do
    for paket in "${PAKETE[@]}"; do
      teil=$(cargo tree -p "$paket" -e normal,build --target "$ziel" --prefix none --format '{p}') \
        || { echo "cargo tree fehlgeschlagen ($paket, $ziel)" >&2; exit 2; }
      liste+="$teil"$'\n'
    done
  done
fi

if treffer=$(printf '%s\n' "$liste" | pruefe); then
  echo "Netzwerkfreiheit: keine gesperrte Bibliothek gefunden."
else
  echo "Netzwerkfreiheit verletzt, gesperrte Bibliotheken in den Abhängigkeiten:" >&2
  printf '  %s\n' $treffer >&2
  echo "Herkunft zeigen: cargo tree -e normal,build --target <Ziel> -i <Crate>" >&2
  exit 1
fi
