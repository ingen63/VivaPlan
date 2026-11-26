"""#287/AK5: Tests of the comparison of two tax data files (invented values).

Run: `python3 -I -m unittest discover -s tools/steuerdaten`
"""

import copy
import io
import json
import os
import sys
import tempfile
import unittest
from contextlib import redirect_stdout

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import vergleich  # noqa: E402


def sample():
    curve = {"single": [[0, 0.0], [50_000, 0.05], [100_000, 0.08]], "married": [[0, 0.0], [50_000, 0.03]]}
    return {
        "federal": copy.deepcopy(curve),
        "federalDeductions": {"earned": {"base": 0, "rate": 0.03, "min": 2000, "max": 4000}},
        "cantons": [{"id": "ZH", "income": copy.deepcopy(curve), "wealth": copy.deepcopy(curve),
                     "deductions": {"earned": {"base": 0, "rate": 0.03, "min": 2000, "max": 4000}},
                     "capitalRule": "Regel"},
                    {"id": "BE", "income": copy.deepcopy(curve), "wealth": copy.deepcopy(curve)}],
        "municipalities": [[1, "Ort", "ZH", [1, 1, 0, 0, 0], [1, 1, 0, 0, 0]]],
    }


class Compare(unittest.TestCase):
    def test_issue_287_ak5_equal_files_have_no_difference(self):
        self.assertEqual(vergleich.compare(sample(), sample()), [])

    def test_issue_287_ak5_rates_beyond_the_tolerance_are_reported(self):
        """#287/AK5: a rate more than 1 % off is reported, one within 1 % and tiny absolute ones are not."""
        new = sample()
        new["cantons"][0]["income"]["single"][1][1] = 0.0506  # +1.2 %
        new["cantons"][0]["income"]["single"][2][1] = 0.0805  # +0.6 %
        new["cantons"][0]["wealth"]["married"][1][1] = 0.03005  # below the absolute tolerance
        lines = vergleich.compare(sample(), new)
        self.assertEqual(len(lines), 1)
        self.assertTrue(lines[0].startswith("ZH income single 50000: 0.05 -> 0.0506"), lines)
        self.assertEqual(vergleich.compare(sample(), new, tol=0.02), [])

    def test_issue_287_ak5_rules_points_and_cantons(self):
        """#287/AK5: changed rules, missing points, missing curves and cantons are reported; the municipalities
        are not compared (#285)."""
        new = sample()
        new["federalDeductions"]["earned"]["max"] = 4100
        new["cantons"][0]["capitalRule"] = "Andere Regel"
        del new["cantons"][0]["wealth"]
        new["cantons"][1]["income"]["single"].pop()
        new["cantons"].append({"id": "AG"})
        new["municipalities"] = []
        lines = vergleich.compare(sample(), new)
        self.assertIn("CH federalDeductions", lines[0])
        self.assertIn("BE income single 100000: point only in old", lines)
        self.assertIn("AG: canton only in new", lines)
        self.assertIn("ZH wealth: missing in new", lines)
        self.assertTrue(any(line.startswith("ZH capitalRule") for line in lines))
        self.assertEqual(vergleich.compare(sample(), new, only={"BE"}), ["BE income single 100000: point only in old"])

    def test_issue_287_ak5_main_exit_code(self):
        with tempfile.TemporaryDirectory() as d:
            a, b = os.path.join(d, "a.json"), os.path.join(d, "b.json")
            new = sample()
            new["federal"]["single"][1][1] = 0.06
            for path, data in ((a, sample()), (b, new)):
                with open(path, "w", encoding="utf-8") as f:
                    json.dump(data, f)
            out = io.StringIO()
            with redirect_stdout(out):
                self.assertEqual(vergleich.main([a, a]), 0)
                self.assertEqual(vergleich.main([a, b, "--kanton", "ZH"]), 0)
                self.assertEqual(vergleich.main([a, b]), 1)
            self.assertIn("no differences", out.getvalue())
            self.assertIn("CH federal single 50000", out.getvalue())


if __name__ == "__main__":
    unittest.main()
