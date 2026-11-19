"""Tests für tools/check-network-free.sh (#157/AK1).

Aufruf: python3 -m unittest discover -s tools -p 'test_check_network_free.py'
Die Fälle prüfen gestellte Listen im Testmodus (--liste), ohne cargo.
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
    # #157/AK1: Ohne gesperrte Bibliothek ist die Prüfung erfolgreich.
    def test_ak1_saubere_liste_besteht(self):
        r = pruefe(["vivaplan-app v1.0.0 (/pfad)", "serde v1.0.229", "tokio v1.0.0 (*)", "http v1.4.0"])
        self.assertEqual(r.returncode, 0, r.stderr)

    # #157/AK1: Ein Treffer aus der Sperrliste lässt die Prüfung fehlschlagen und wird genannt.
    def test_ak1_treffer_schlaegt_fehl(self):
        for crate in ["reqwest", "hyper", "ureq", "curl", "tokio-tungstenite", "tauri-plugin-http", "tauri-plugin-updater"]:
            with self.subTest(crate=crate):
                r = pruefe(["serde v1.0.229", f"{crate} v0.1.0", f"{crate} v0.1.0 (*)"])
                self.assertEqual(r.returncode, 1)
                self.assertEqual(r.stderr.count(f"  {crate}\n"), 1, r.stderr)

    # #157/AK1: Vergleich über exakte Namen, nicht Teilstrings (dispatch2 ist nicht h2, hyperx nicht hyper).
    def test_ak1_nur_exakte_namen(self):
        r = pruefe(["dispatch2 v0.3.0", "hyperx v1.0.0", "curl-parser v0.1.0", "my-reqwest v0.1.0"])
        self.assertEqual(r.returncode, 0, r.stderr)

    # #157/AK1: Falscher Aufruf ist ein eigener Fehler (2), kein stilles Bestehen.
    def test_ak1_fehlende_liste(self):
        r = subprocess.run(["bash", str(SKRIPT), "--liste", "/gibt/es/nicht"], capture_output=True, text=True)
        self.assertEqual(r.returncode, 2)


if __name__ == "__main__":
    unittest.main()
