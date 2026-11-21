"""#185: static checks of the app icon (sources, generated files, favicon). Run: python3 -m unittest tools/test_icons.py"""
import json
import struct
import unittest
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
APP = ROOT / "crates" / "app"
ICONS = APP / "icons"
COLORS = ["#f28c38", "#ffd27a", "#c0502a", "#6b2a2a"]  # background, sun, back range, front range


def png_size(data: bytes) -> tuple[int, int]:
    assert data[:8] == b"\x89PNG\r\n\x1a\n", "not a PNG"
    return struct.unpack(">II", data[16:24])


class IconTest(unittest.TestCase):
    def test_issue_185_ak1_svg_sources_show_evening_glow(self):
        for name in ("icon.svg", "icon-small.svg"):
            svg = (ICONS / name).read_text()
            for color in COLORS:
                self.assertIn(color, svg, f"{name}: color {color}")
            # flat colors only, no text (no font licence), two mountain ranges
            for banned in ("Gradient", "<text", "<image"):
                self.assertNotIn(banned, svg, name)
            self.assertEqual(svg.count("<polygon"), 2, name)
        self.assertIn("stroke-dasharray", (ICONS / "icon.svg").read_text())  # dotted path
        self.assertNotIn("stroke-dasharray", (ICONS / "icon-small.svg").read_text())  # small one without path

    def test_issue_185_ak2_configured_icons_exist_with_expected_sizes(self):
        conf = json.loads((APP / "tauri.conf.json").read_text())
        expected = {"32x32.png": 32, "128x128.png": 128, "128x128@2x.png": 256}
        for rel in conf["bundle"]["icon"]:
            path = APP / rel
            self.assertTrue(path.is_file(), rel)
            if path.name in expected:
                self.assertEqual(png_size(path.read_bytes()), (expected[path.name],) * 2, rel)
        self.assertEqual(png_size((ICONS / "icon.png").read_bytes()), (512, 512))
        self.assertEqual((ICONS / "icon.icns").read_bytes()[:4], b"icns")

    def test_issue_185_ak2_ico_contains_small_and_large_sizes(self):
        data = (ICONS / "icon.ico").read_bytes()
        _, kind, count = struct.unpack("<HHH", data[:6])
        self.assertEqual(kind, 1)
        sizes = []
        for i in range(count):
            _, _, _, _, _, _, length, offset = struct.unpack("<BBBBHHII", data[6 + 16 * i : 22 + 16 * i])
            sizes.append(png_size(data[offset : offset + length])[0])
        self.assertEqual(sizes, [16, 24, 32, 48, 64, 256])

    def test_issue_185_favicon_is_the_small_icon(self):
        self.assertEqual((APP / "ui" / "favicon.svg").read_bytes(), (ICONS / "icon-small.svg").read_bytes())
        self.assertIn('<link rel="icon" type="image/svg+xml" href="favicon.svg">', (APP / "ui" / "index.html").read_text())


if __name__ == "__main__":
    unittest.main()
