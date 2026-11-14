#!/usr/bin/env bash
# Baut die Desktop-App (nur macOS) und ersetzt /Applications/VivaPlan.app.
# Wird vom Git-Hook post-commit im Hintergrund gestartet, geht aber auch von Hand:
#   tools/build-install.sh
# Läuft schon ein Build, wird nur vorgemerkt; der laufende Build baut danach noch einmal,
# damit am Schluss immer der neueste Commit installiert ist.
# Protokoll: target/build-install.log

set -uo pipefail

root="$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"
cd "$root"

[ "$(uname)" = "Darwin" ] || { echo "build-install: nur unter macOS"; exit 0; }

app_name="VivaPlan.app"
bundle="$root/target/release/bundle/macos/$app_name"
dest="/Applications/$app_name"
lock="$root/target/.build-install.lock"
pending="$root/target/.build-install.pending"

mkdir -p "$root/target"

# Nur ein Build gleichzeitig; ein weiterer Aufruf merkt sich den Wunsch vor
if ! mkdir "$lock" 2>/dev/null; then
  touch "$pending"
  echo "build-install: Build läuft bereits, neuer Build ist vorgemerkt."
  exit 0
fi
trap 'rmdir "$lock" 2>/dev/null' EXIT

while :; do
  rm -f "$pending"
  sha="$(git rev-parse --short HEAD)"
  echo "build-install: $(date '+%Y-%m-%d %H:%M:%S') baue Commit $sha ..."

  if [ ! -d crates/app/node_modules ]; then
    (cd crates/app && npm install) || { echo "build-install: npm install fehlgeschlagen."; exit 1; }
  fi

  if ! (cd crates/app && npx tauri build --bundles app); then
    echo "build-install: Build von $sha fehlgeschlagen, installierte App bleibt unverändert."
    exit 1
  fi

  # Erst daneben kopieren, dann austauschen: bei einem Fehler bleibt die alte App erhalten
  tmp="/Applications/.$app_name.neu"
  rm -rf "$tmp"
  if ditto "$bundle" "$tmp" && rm -rf "$dest" && mv "$tmp" "$dest"; then
    echo "build-install: $(date '+%Y-%m-%d %H:%M:%S') Commit $sha installiert in $dest."
  else
    rm -rf "$tmp"
    echo "build-install: Installation nach $dest fehlgeschlagen."
    exit 1
  fi

  [ -e "$pending" ] || break
done
