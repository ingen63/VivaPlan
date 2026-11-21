#!/usr/bin/env bash
# #185: generates all app icons from the two SVG sources in crates/app/icons/:
#   icon.svg        full icon (dotted path), used from 48 px upwards
#   icon-small.svg  simplified icon (no path, larger sun), used for 16, 24 and 32 px
# Output: 32x32.png, 64x64.png, 128x128.png, 128x128@2x.png, icon.png (512), icon.ico, icon.icns,
# and the favicon of the user interface (crates/app/ui/favicon.svg, a copy of icon-small.svg).
# Tools: `npx tauri icon` (renders the SVG, @tauri-apps/cli in crates/app), python3 (ICO), iconutil (macOS, ICNS).
# Option --preview: also writes docs/bilder/icon-vorschau.png (16/32/128/512 px on light and dark; needs ImageMagick).
set -euo pipefail
root="$(cd "$(dirname "$0")/.." && pwd)"
app="$root/crates/app"
icons="$app/icons"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

cd "$app"
[ -d node_modules/@tauri-apps/cli ] || npm ci
render() { # render <svg> <dir> <sizes...>
  local svg="$1" dir="$2"; shift 2
  local args=(); for s in "$@"; do args+=(-p "$s"); done
  npx tauri icon "$svg" -o "$dir" "${args[@]}" >/dev/null 2>&1
}
render icons/icon-small.svg "$tmp/small" 16 24 32
render icons/icon.svg "$tmp/big" 48 64 128 256 512 1024
small() { echo "$tmp/small/${1}x${1}.png"; }
big() { echo "$tmp/big/${1}x${1}.png"; }

cp "$(small 32)" "$icons/32x32.png"
cp "$(big 64)" "$icons/64x64.png"
cp "$(big 128)" "$icons/128x128.png"
cp "$(big 256)" "$icons/128x128@2x.png"
cp "$(big 512)" "$icons/icon.png"
cp icons/icon-small.svg ui/favicon.svg

# ICO with PNG entries (Windows Vista and later): small source up to 32 px, full source above.
python3 - "$icons/icon.ico" "$(small 16)" "$(small 24)" "$(small 32)" "$(big 48)" "$(big 64)" "$(big 256)" <<'PY'
import struct, sys
out, pngs = sys.argv[1], [open(p, "rb").read() for p in sys.argv[2:]]
head = struct.pack("<HHH", 0, 1, len(pngs))
offset, entries = 6 + 16 * len(pngs), b""
for data in pngs:
    w, h = struct.unpack(">II", data[16:24])
    entries += struct.pack("<BBBBHHII", w % 256, h % 256, 0, 0, 1, 32, len(data), offset)
    offset += len(data)
open(out, "wb").write(head + entries + b"".join(pngs))
PY

# ICNS via iconutil (macOS)
set_dir="$tmp/icon.iconset"; mkdir "$set_dir"
cp "$(small 16)" "$set_dir/icon_16x16.png";   cp "$(small 32)" "$set_dir/icon_16x16@2x.png"
cp "$(small 32)" "$set_dir/icon_32x32.png";   cp "$(big 64)" "$set_dir/icon_32x32@2x.png"
cp "$(big 128)" "$set_dir/icon_128x128.png";  cp "$(big 256)" "$set_dir/icon_128x128@2x.png"
cp "$(big 256)" "$set_dir/icon_256x256.png";  cp "$(big 512)" "$set_dir/icon_256x256@2x.png"
cp "$(big 512)" "$set_dir/icon_512x512.png";  cp "$(big 1024)" "$set_dir/icon_512x512@2x.png"
iconutil -c icns "$set_dir" -o "$icons/icon.icns"

if [ "${1:-}" = "--preview" ]; then
  row() { # row <background> <file>: native sizes 16/32/128/512 plus 16 and 32 px magnified
    magick -size 1100x600 xc:"$1" \
      \( "$(small 16)" -filter point -resize 800% \) -geometry +30+40 -composite \
      \( "$(small 32)" -filter point -resize 400% \) -geometry +30+210 -composite \
      "$(small 16)" -geometry +40+500 -composite "$(small 32)" -geometry +80+492 -composite \
      "$(big 128)" -geometry +150+444 -composite "$(big 512)" -geometry +460+44 -composite "$2"
  }
  row "#f2f2f2" "$tmp/light.png"; row "#1e1e1e" "$tmp/dark.png"
  magick "$tmp/light.png" "$tmp/dark.png" -append "$root/docs/bilder/icon-vorschau.png"
fi
echo "Icons written to $icons"
