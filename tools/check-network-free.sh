#!/usr/bin/env bash
# #157: Prüft, dass keine Netzwerk-Bibliothek in den Abhängigkeiten der App steckt (Versprechen «keine Netzwerkzugriffe»).
#
# Aufruf:  tools/check-network-free.sh               cargo tree für die drei Pakete und alle Desktop-Ziele
#          tools/check-network-free.sh --liste DATEI  prüft eine gestellte Liste im Format von cargo tree (Testmodus)
#
# Geprüft werden nur normale und Build-Abhängigkeiten (-e normal,build), keine Entwicklungsabhängigkeiten.
# Ziele: die Desktop-Plattformen, für die gebaut wird (pakete.yml), je x86_64 und aarch64. Nicht `--target all`:
# tauri zieht für Android/iOS reqwest und hyper herein; für Mobilgeräte wird nicht gebaut.
# Vergleich über exakte Crate-Namen (erstes Feld je Zeile), keine Teilstrings: `dispatch2` trifft nicht `h2`.
# Fehlercode: 0 kein Treffer, 1 Treffer, 2 Aufruf- oder cargo-Fehler.
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

# Liest Zeilen «name vX.Y.Z …» von stdin und meldet jeden Sperrlisten-Treffer genau einmal.
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
