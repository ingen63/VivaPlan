"""Tests für die Übersichtsseite und die Diff-Abdeckung (FEA-28, FEA-30, FEA-31). Aufruf: python3 -m unittest discover tools/report"""
import os
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, os.path.dirname(__file__))
import report  # noqa: E402  (wechselt in das Repository-Verzeichnis)
import coverage_diff  # noqa: E402


class Status(unittest.TestCase):
    def test_hygiene_follows_the_stated_rule(self):
        self.assertEqual(report.status_hygiene(None), report.NODATA)
        self.assertEqual(report.status_hygiene({"failed": [], "flaky": []}), report.OK)
        self.assertEqual(report.status_hygiene({"failed": [], "flaky": ["a"]}), report.HINT)
        self.assertEqual(report.status_hygiene({"failed": ["x"], "flaky": ["a"]}), report.FAIL)

    def test_fit_needs_both_packages_and_no_missed_mutants(self):
        pk = lambda a, b: {"pkgs": {"Engine (vivaplan-engine)": (a, 10), "App (vivaplan-app)": (b, 10)}}
        mut = lambda n: {"missed": n}
        self.assertEqual(report.status_fit(None, None), report.NODATA)
        self.assertEqual(report.status_fit(pk(90, 85), mut(0)), report.OK)
        self.assertEqual(report.status_fit(pk(90, 85), mut(2)), report.HINT)
        self.assertEqual(report.status_fit(pk(90, 85), None), report.HINT)
        self.assertEqual(report.status_fit(pk(95, 79.9), mut(0)), report.FAIL)  # Pakete werden nicht gemittelt

    def test_junit_with_a_failing_test_is_reported_by_name(self):
        xml = '<testsuites><testsuite><testcase classname="engine::x" name="kaputt" time="0.1"><failure/></testcase><testcase classname="engine::x" name="ok" time="0.2"/><testcase classname="engine::x" name="wackelt" time="0.3"><flakyFailure/></testcase></testsuite></testsuites>'
        with tempfile.TemporaryDirectory() as d:
            (Path(d) / "junit.xml").write_text(xml)
            cwd = os.getcwd()
            os.chdir(d)
            try:
                j = report.read_junit()
            finally:
                os.chdir(cwd)
        self.assertEqual(j["failed"], ["engine::x kaputt"])
        self.assertEqual(j["flaky"], ["engine::x wackelt"])
        self.assertEqual(report.status_hygiene(j), report.FAIL)


class Requirements(unittest.TestCase):
    ISSUE = {"number": 7, "title": "FEA-100 Beispiel", "state": "OPEN", "labels": [], "body": "Text\n\n- [ ] eins\n- [x] zwei\n- [ ] drei\n"}

    def test_two_of_three_criteria_with_a_marked_test(self):
        marks = {("FEA-100", 1): ["a.rs:3"], ("FEA-100", 3): ["b.rs:9"]}
        rows = report.requirements([self.ISSUE], marks)
        kinds = [k for k, _, _ in rows[0]["items"]]
        self.assertEqual(kinds, ["test", "manuell", "test"])  # 2 von 3 belegt, das abgehakte nur manuell

    def test_markers_in_comments_and_test_names(self):
        self.assertEqual(report.MARK.search("// FEA-100/AK2 prüft").group(0), "FEA-100/AK2")
        self.assertEqual(report.NAMEMARK.search("fn fea_100_ak2_rundung()").groups(), ("fea", "100", "2"))

    def test_escapes_list_found_where_and_regression_test(self):
        bug = {"number": 9, "title": "ISS-9 x", "state": "OPEN", "labels": [{"name": "bug"}, {"name": "gefunden:nutzung"}],
               "body": "Text\n\n**Welcher Test hätte ihn fangen müssen?** Ein Rundungstest.\n\n**Regressionstest:** crates/engine/tests/x.rs\n"}
        other = {"number": 10, "title": "ISS-10 y", "state": "OPEN", "labels": [{"name": "bug"}], "body": ""}
        rows = report.escapes([bug, other])
        self.assertEqual(len(rows), 1)
        self.assertEqual((rows[0]["found"], rows[0]["which"], rows[0]["reg"]), ("nutzung", "Ein Rundungstest.", "crates/engine/tests/x.rs"))


class DiffCoverage(unittest.TestCase):
    def test_ranges(self):
        self.assertEqual(coverage_diff.ranges([5, 3, 4, 9]), "3–5, 9")

    def test_base_comes_from_diff_base(self):
        # FEA-28: Im Workflow gibt DIFF_BASE den Stand vor dem Push vor; ohne Angabe gilt origin/main.
        with tempfile.NamedTemporaryFile("w", suffix=".info", delete=False) as f:
            f.write("SF:crates/engine/src/lib.rs\nDA:1,1\nend_of_record\n")
        old = os.environ.get("DIFF_BASE")
        try:
            os.environ["DIFF_BASE"] = "HEAD"
            self.assertEqual(report.read_diff_coverage(f.name)["base"], "HEAD")
            os.environ.pop("DIFF_BASE")
            r = report.read_diff_coverage(f.name)
            self.assertTrue(r is None or r["base"] == "origin/main")  # ohne origin/main (flacher Klon) keine Angabe
        finally:
            os.unlink(f.name)
            if old is not None:
                os.environ["DIFF_BASE"] = old


if __name__ == "__main__":
    unittest.main()
