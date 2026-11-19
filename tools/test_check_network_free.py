"""Tests for tools/check-network-free.sh (#157/AK1).

Usage: python3 -m unittest discover -s tools -p 'test_check_network_free.py'
The cases check given lists in test mode (`--liste`), without cargo.
"""

import pathlib
import subprocess
import tempfile
import unittest

SKRIPT = pathlib.Path(__file__).resolve().parent / "check-network-free.sh"


def pruefe(zeilen):
    with tempfile.NamedTemporaryFile("w", suffix=".txt", delete=False) as f:
        f.write("\n".join(zeilen) + "\n")
        pfad = f.name
    try:
        return subprocess.run(["bash", str(SKRIPT), "--liste", pfad], capture_output=True, text=True)
    finally:
        pathlib.Path(pfad).unlink()


class NetzwerkfreiheitTest(unittest.TestCase):
    # #157/AK1: without a blocked library the check passes.
    def test_ak1_saubere_liste_besteht(self):
        r = pruefe(["vivaplan-app v1.0.0 (/pfad)", "serde v1.0.229", "tokio v1.0.0 (*)", "http v1.4.0"])
        self.assertEqual(r.returncode, 0, r.stderr)

    # #157/AK1: a hit from the blocklist fails the check and is named.
    def test_ak1_treffer_schlaegt_fehl(self):
        for crate in ["reqwest", "hyper", "ureq", "curl", "tokio-tungstenite", "tauri-plugin-http", "tauri-plugin-updater"]:
            with self.subTest(crate=crate):
                r = pruefe(["serde v1.0.229", f"{crate} v0.1.0", f"{crate} v0.1.0 (*)"])
                self.assertEqual(r.returncode, 1)
                self.assertEqual(r.stderr.count(f"  {crate}\n"), 1, r.stderr)

    # #157/AK1: comparison by exact names, not substrings (dispatch2 is not h2, hyperx is not hyper).
    def test_ak1_nur_exakte_namen(self):
        r = pruefe(["dispatch2 v0.3.0", "hyperx v1.0.0", "curl-parser v0.1.0", "my-reqwest v0.1.0"])
        self.assertEqual(r.returncode, 0, r.stderr)

    # #157/AK1: wrong usage is a separate error (2), not a silent pass.
    def test_ak1_fehlende_liste(self):
        r = subprocess.run(["bash", str(SKRIPT), "--liste", "/gibt/es/nicht"], capture_output=True, text=True)
        self.assertEqual(r.returncode, 2)


if __name__ == "__main__":
    unittest.main()
