"""Tests of tools/golden_numbers.py (#176/AK2, #195)."""

import contextlib
import io
import json
import os
import pathlib
import subprocess
import sys
import tempfile
import unittest

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from golden_numbers import compare, find_reasons, main, new_report  # noqa: E402


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


class ReasonTest(unittest.TestCase):
    def test_issue_195_ak2_rule_golden_line_with_issue_and_reason(self):
        msg = "Fix (#190)\n\nGolden: #190 13. AHV-Rente im Dezember\n\nCo-Authored-By: x\n"
        self.assertEqual(find_reasons(msg), ["Golden: #190 13. AHV-Rente im Dezember"])

    def test_issue_195_ak2_lines_without_issue_reason_or_at_line_start_do_not_count(self):
        msg = "Golden: #190\nGolden: 13. AHV\n  Golden: #190 indented\nsee Golden: #190 text\ngolden: #190 x\n"
        self.assertEqual(find_reasons(msg), [])


# A golden file in miniature (one monthly value, two text fields), named like the property scenario
GOLDEN = {"months": [{"month": "2030-01", "cash": 1000.5, "event": "Kauf"}], "label": "Immobilie"}


class GateTest(unittest.TestCase):
    """#195/AK3: --gate in a temporary repository with two commits (before and after the push)."""

    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = pathlib.Path(self.tmp.name)
        self.cwd = os.getcwd()
        self.git("init", "-q")
        self.git("config", "user.email", "test@example.invalid")
        self.git("config", "user.name", "Test")
        self.dir = self.root / "crates/engine/tests/golden"
        self.dir.mkdir(parents=True)
        self.write(GOLDEN)
        self.commit("Golden files")
        self.base = self.git("rev-parse", "HEAD").strip()
        os.chdir(self.root)

    def tearDown(self):
        os.chdir(self.cwd)
        self.tmp.cleanup()

    def git(self, *args):
        return subprocess.run(["git", *args], cwd=self.root, capture_output=True, text=True, check=True).stdout

    def write(self, data, name="05-immobilie-hypothek.json"):
        (self.dir / name).write_text(json.dumps(data))

    def commit(self, message):
        self.git("add", "-A")
        self.git("commit", "-q", "-m", message)

    def gate(self):
        out = io.StringIO()
        with contextlib.redirect_stdout(out):
            code = main(["--base", self.base, "--gate"])
        return code, out.getvalue()

    def changed_month(self):
        data = json.loads(json.dumps(GOLDEN))
        data["months"][0]["cash"] = 1000.75
        return data

    def test_issue_195_ak3_changed_monthly_value_without_reason_fails(self):
        self.write(self.changed_month())
        self.commit("Change the engine (#190)")
        code, out = self.gate()
        self.assertEqual(code, 1)
        self.assertIn("05-immobilie-hypothek.months[0].cash: 1000.5 instead of 1000.75", out)  # #195/AK1: path
        self.assertIn("FAILED", out)

    def test_issue_195_ak3_changed_monthly_value_with_reason_in_one_of_the_commits_passes(self):
        self.write(self.changed_month())
        self.commit("Change the engine (#190)\n\nGolden: #190 13. AHV-Rente im Dezember")
        (self.root / "other.txt").write_text("x")
        self.commit("Something else")
        code, out = self.gate()
        self.assertEqual(code, 0)
        self.assertIn("05-immobilie-hypothek.months[0].cash", out)
        self.assertIn("Justified by:\n  Golden: #190 13. AHV-Rente im Dezember", out)

    def test_issue_195_ak3_reason_before_the_base_does_not_count(self):
        self.git("commit", "-q", "--allow-empty", "-m", "Old\n\nGolden: #1 old reason")
        self.base = self.git("rev-parse", "HEAD").strip()
        self.write(self.changed_month())
        self.commit("Change the engine (#190)")
        self.assertEqual(self.gate()[0], 1)

    def test_issue_195_ak3_only_text_fields_changed_passes_without_reason(self):
        data = json.loads(json.dumps(GOLDEN))
        data["months"][0]["event"] = "Purchase"
        data["label"] = "Property"
        self.write(data)
        self.commit("Texts in English (#176)")
        code, out = self.gate()
        self.assertEqual(code, 0)
        self.assertIn("OK: all numbers", out)

    def test_issue_195_ak3_new_file_is_named_deleted_file_fails(self):
        self.write(GOLDEN, "11-new.json")
        self.commit("New scenario")
        code, out = self.gate()
        self.assertEqual(code, 0)
        self.assertIn("new files (not compared): 11-new.json", out)
        (self.dir / "05-immobilie-hypothek.json").unlink()
        self.commit("Remove scenario")
        code, out = self.gate()
        self.assertEqual(code, 1)
        self.assertIn("05-immobilie-hypothek: file deleted", out)

    def test_issue_195_ak3_without_gate_any_difference_fails(self):
        self.write(self.changed_month())
        self.commit("Change\n\nGolden: #190 reason")
        with contextlib.redirect_stdout(io.StringIO()):
            self.assertEqual(main(["--base", self.base]), 1)


if __name__ == "__main__":
    unittest.main()
