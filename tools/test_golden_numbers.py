"""Tests of tools/golden_numbers.py (#176/AK2)."""

import pathlib
import sys
import unittest

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from golden_numbers import compare, new_report  # noqa: E402


def run(old, new):
    r = new_report()
    compare(old, new, "f", r)
    return r


class CompareTest(unittest.TestCase):
    def test_issue_176_ak2_only_texts_differ(self):
        r = run({"events": [{"text": "Kauf", "amount": 1.5}], "label": "Fixe Renditen"},
                {"events": [{"text": "Purchase", "amount": 1.5}], "label": "Fixed returns"})
        self.assertEqual(r["values"], [])
        self.assertEqual(dict(r["texts"]), {"text": 1, "label": 1})
        self.assertEqual(r["numbers"], 1)

    def test_issue_176_ak2_a_changed_number_is_reported(self):
        r = run({"a": [1, 2.0]}, {"a": [1, 2.0000001]})
        self.assertEqual(r["values"], ["f.a[1]: 2.0 instead of 2.0000001"])

    def test_issue_176_ak2_structure_types_and_months_count_as_values(self):
        r = run({"a": [1], "m": "2030-01", "b": 1, "c": True}, {"a": [1, 2], "m": "2030-02", "x": 0, "c": 1})
        self.assertEqual(r["values"], [
            "f.a: 1 entries instead of 2",
            "f.b: only in old",
            "f.c: type bool instead of int",
            "f.m: '2030-01' instead of '2030-02'",
            "f.x: only in new",
        ])

    def test_issue_176_ak2_int_and_float_of_the_same_value_are_equal(self):
        self.assertEqual(run({"a": 2}, {"a": 2.0})["values"], [])


if __name__ == "__main__":
    unittest.main()
