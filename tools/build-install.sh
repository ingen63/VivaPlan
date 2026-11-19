#!/usr/bin/env bash
# Builds the desktop app (macOS only) and replaces /Applications/VivaPlan.app.
# Started in the background by the git hook post-commit, but also works by hand:
#   tools/build-install.sh
# If a build is already running, a new one is only queued; the running build then builds once more,
# so that the newest commit is always installed in the end.
# Log: target/build-install.log

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

# Only one build at a time; a further call queues the request
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

  # Copy next to it first, then swap: on error the old app stays
  tmp="/Applications/.$app_name.new"
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
