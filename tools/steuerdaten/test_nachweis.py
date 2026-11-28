"""#285: Tests of the proof of origin of the tax data (invented files).

Run: `python3 -I -m unittest discover -s tools/steuerdaten`
"""

import hashlib
import os
import sys
import tempfile
import unittest

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import nachweis  # noqa: E402


class Proof(unittest.TestCase):
    # #285/AK2: every file used by the description and the exceptions must be listed with its hash; derived files
    # need a listed original; a source of the ESTV fails.
    def test_issue_285_ak2_sources_listed_with_hash(self):
        folder = tempfile.mkdtemp()
        for name, text in (("XA.pdf", "pdf"), ("XA.txt", "text"), ("XB.csv", "csv")):
            with open(os.path.join(folder, name), "w", encoding="utf-8") as f:
                f.write(text)
        digest = {n: hashlib.sha256(open(os.path.join(folder, n), "rb").read()).hexdigest()
                  for n in ("XA.pdf", "XA.txt", "XB.csv")}
        specs = {"XA": {"file": "XA.txt"}, "XB": {"file": "XB.csv"}, "_comment": {"file": "x"}}
        exceptions = {"XB": {"9001": {"municipal": 1.0, "source": "s", "beleg": "XB.csv"}}, "_comment": "x"}
        used = nachweis.used_files(specs, exceptions)
        self.assertEqual(used, {"XA.txt": "XA", "XB.csv": "exception XB 9001"})
        quellen = {"quellen": [{"datei": "XA.pdf", "url": "https://ex.ch/a.pdf", "sha256": digest["XA.pdf"]},
                               {"datei": "XB.csv", "url": "https://ex.ch/b.csv", "sha256": digest["XB.csv"]}],
                   "abgeleitet": [{"datei": "XA.txt", "aus": "XA.pdf", "sha256": digest["XA.txt"]}]}
        self.assertTrue(all(ok for ok, _ in nachweis.check_sources(folder, quellen, used)))
        # a changed file, an unlisted file and an ESTV address fail
        quellen["quellen"][1]["sha256"] = "0" * 64
        quellen["quellen"][0]["url"] = "https://swisstaxcalculator.estv.admin.ch/x"
        results = nachweis.check_sources(folder, quellen, dict(used, **{"XC.csv": "XC"}))
        failed = [text for ok, text in results if not ok]
        self.assertEqual(len(failed), 3, failed)


if __name__ == "__main__":
    unittest.main()
