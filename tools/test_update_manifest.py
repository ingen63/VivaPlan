"""Tests for tools/update_manifest.py (#309/AK10).

Usage: python3 -m unittest discover -s tools -p 'test_update_manifest.py'
"""

import datetime
import json
import pathlib
import sys
import tempfile
import unittest

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import update_manifest  # noqa: E402

NOW = datetime.datetime(2026, 11, 27, 12, 0, tzinfo=datetime.timezone.utc)


def changelogs(root):
    heads = {"de": "Neu, geändert, behoben", "en": "New, changed, fixed",
             "fr": "Nouveautés, modifications, corrections", "it": "Novità, modifiche, correzioni"}
    for lang, head in heads.items():
        name = "CHANGELOG.md" if lang == "de" else f"CHANGELOG.{lang}.md"
        (root / name).write_text(f"# Changelog\n\n## Version 9.0.0-beta.1\n\n### {head}\n- {lang} (#309)\n",
                                 encoding="utf-8")


class ManifestTest(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = pathlib.Path(self.tmp.name)
        self.files = self.root / "flach"
        self.files.mkdir()
        changelogs(self.root)

    def tearDown(self):
        self.tmp.cleanup()

    def put(self, name, sig=True):
        (self.files / name).write_bytes(b"x")
        if sig:
            (self.files / (name + ".sig")).write_text(f"SIG-{name}\n", encoding="utf-8")

    # #309/AK10: macOS, Windows and Linux each get the update file with its signature and the download address.
    def test_ak10_three_platforms_with_signatures(self):
        for name in ["VivaPlan.app.tar.gz", "VivaPlan_9.0.0-beta.1_x64-setup.exe", "VivaPlan_9.0.0-beta.1_x64_de-DE.msi",
                     "VivaPlan_9.0.0-beta.1_amd64.AppImage"]:
            self.put(name)
        self.put("VivaPlan_9.0.0-beta.1_aarch64.dmg", sig=False)  # no update file: ignored
        m = update_manifest.manifest(self.files, "v9.0.0-beta.1", "owner/repo", self.root, NOW)
        self.assertEqual(m["version"], "9.0.0-beta.1")
        self.assertEqual(m["pub_date"], "2026-11-27T12:00:00Z")
        self.assertEqual(sorted(m["platforms"]), ["darwin-aarch64", "linux-x86_64", "windows-x86_64"])
        win = m["platforms"]["windows-x86_64"]
        self.assertEqual(win["signature"], "SIG-VivaPlan_9.0.0-beta.1_x64-setup.exe")
        self.assertEqual(win["url"], "https://github.com/owner/repo/releases/download/v9.0.0-beta.1/"
                                     "VivaPlan_9.0.0-beta.1_x64-setup.exe")
        notes = json.loads(m["notes"])
        self.assertEqual(sorted(notes), ["de", "en", "fr", "it"])
        self.assertIn("de (#309)", notes["de"])

    # #309/AK6, AK10: an update file without signature stops the release instead of shipping an unsigned update.
    def test_ak10_missing_signature_fails(self):
        self.put("VivaPlan_9.0.0-beta.1_amd64.AppImage", sig=False)
        with self.assertRaises(ValueError):
            update_manifest.manifest(self.files, "v9.0.0-beta.1", "owner/repo", self.root, NOW)
        self.assertEqual(update_manifest.main([str(self.files), "--tag", "v9.0.0-beta.1", "--repo", "o/r",
                                               "--root", str(self.root)]), 1)

    # #309/AK10: without any update file (no signing key in the run) there is no manifest.
    def test_ak10_no_update_files_fails(self):
        with self.assertRaises(ValueError):
            update_manifest.manifest(self.files, "v9.0.0-beta.1", "owner/repo", self.root, NOW)

    # #309/AK10: the MSI counts only without NSIS installer; the manifest is written to the given file.
    def test_ak10_msi_fallback_and_output_file(self):
        self.put("VivaPlan_9.0.0_x64_de-DE.msi")
        out = self.root / "latest.json"
        rc = update_manifest.main([str(self.files), "--tag", "v9.0.0-beta.1", "--repo", "o/r", "--root",
                                   str(self.root), "-o", str(out)])
        self.assertEqual(rc, 0)
        m = json.loads(out.read_text(encoding="utf-8"))
        self.assertTrue(m["platforms"]["windows-x86_64"]["url"].endswith(".msi"))

    # #309/AK10: a release for all (--complete) needs all three platforms; a partial build gives no manifest.
    def test_ak10_complete_needs_every_platform(self):
        self.put("VivaPlan.app.tar.gz")
        self.put("VivaPlan_9.0.0_amd64.AppImage")
        with self.assertRaises(ValueError) as e:
            update_manifest.manifest(self.files, "v9.0.0-beta.1", "o/r", self.root, NOW, complete=True)
        self.assertIn("windows-x86_64", str(e.exception))
        self.put("VivaPlan_9.0.0_x64-setup.exe")
        m = update_manifest.manifest(self.files, "v9.0.0-beta.1", "o/r", self.root, NOW, complete=True)
        self.assertEqual(len(m["platforms"]), 3)


if __name__ == "__main__":
    unittest.main()
