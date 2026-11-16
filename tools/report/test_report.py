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
    def test_playwright_run_time_comes_from_the_run_not_from_the_file(self):
        # FEA-71/AK3: results.json wird in der Pipeline heruntergeladen, die Dateizeit wäre die des Downloads
        self.assertRegex(report.run_time("2026-11-14T22:01:05.375Z"), r"^\d\d\.\d\d\.\d{4} \d\d:\d\d$")
        self.assertIsNone(report.run_time(None))
        self.assertIsNone(report.run_time("kein Datum"))

    def test_hygiene_follows_the_stated_rule(self):
        self.assertEqual(report.status_hygiene(None), report.NODATA)
        self.assertEqual(report.status_hygiene({"failed": [], "flaky": []}), report.OK)
        self.assertEqual(report.status_hygiene({"failed": [], "flaky": ["a"]}), report.HINT)
        self.assertEqual(report.status_hygiene({"failed": ["x"], "flaky": ["a"]}), report.FAIL)

    def test_fit_needs_both_packages_and_no_missed_mutants(self):
        pk = lambda a, b, c=90: {"pkgs": {"Engine (vivaplan-engine)": (a, 10), "App (vivaplan-app)": (b, 10), "Kommandozeile (vivaplan-cli)": (c, 10)}}
        mut = lambda n: {"missed": n}
        self.assertEqual(report.status_fit(None, None), report.NODATA)
        self.assertEqual(report.status_fit(pk(90, 85), mut(0)), report.OK)
        self.assertEqual(report.status_fit(pk(90, 85), mut(2)), report.HINT)
        self.assertEqual(report.status_fit(pk(90, 85), None), report.HINT)
        self.assertEqual(report.status_fit(pk(95, 79.9), mut(0)), report.FAIL)  # Pakete werden nicht gemittelt
        self.assertEqual(report.status_fit(pk(95, 85, 79.9), mut(0)), report.FAIL)  # FEA-81/AK3: auch die Kommandozeile zählt

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

    def test_fea_88_ak1_leading_zero_in_title_or_mark_is_the_same_issue(self):
        issue = dict(self.ISSUE, title="FEA-07 Beispiel")
        marks = report.scan_marks("e2e/x.spec.js", ['test("FEA-07/AK1 …", () => {});', "// FEA-7/AK3"], {})
        kinds = [k for k, _, _ in report.requirements([issue], marks)[0]["items"]]
        self.assertEqual(kinds, ["test", "manuell", "test"])

    def test_fea_88_ak2_bare_ak_in_a_describe_block_with_one_issue_id(self):
        src = ['test.describe("FEA-56 Ergebnis in Worten", () => {',
               '  test("AK1, AK3: mehrere Läufe", async () => {',
               '  });',
               '  test.describe("Untergruppe ohne ID", () => {',
               '    test("AK2 im inneren Block", async () => {});',
               '  });',
               '});',
               'test("AK4 ausserhalb jedes Blocks", async () => {});',
               'test.describe("FEA-54 und FEA-55", () => {',
               '  test("AK5 mehrdeutig", async () => {});',
               '});']
        marks = report.scan_marks("e2e/x.spec.js", src, {})
        self.assertEqual(sorted(marks), [("FEA-56", 1), ("FEA-56", 2), ("FEA-56", 3)])
        self.assertEqual(marks[("FEA-56", 1)], ["e2e/x.spec.js:2"])
        # in Rust-Dateien gibt es keine describe-Blöcke
        self.assertEqual(report.scan_marks("a.rs", src, {}), {})

    def test_fea_88_ak3_checked_criteria_count_as_checked_not_manual(self):
        c = lambda day, body: {"createdAt": f"{day}T10:00:00Z", "body": body}
        issue = dict(self.ISSUE, body="- [x] a\n- [x] b\n- [x] c\n- [x] d\n- [ ] e\n", comments=[
            c("2026-11-15", "Prüfung FEA-100 (15.11.2026)\n\n- **AK1 erfüllt.** Test grün.\n- **AK2 nicht ganz erfüllt.** Rest.\n- AK3, AK4 ok: gelesen."),
            c("2026-11-16", "Nacharbeit AK2 in abc1234: erledigt.\n\n- **AK2 erfüllt.**"),
            c("2026-11-16", "Prüfung vom 16.11.2026 durch den Anwender in der App: AK4 ist in Ordnung."),
            c("2026-11-17", "**Erneute Prüfung**\n\n- AK3 offen: Gegenprobe fehlt."),
        ])
        items = report.requirements([issue], {("FEA-100", 4): ["t.rs:1"]})[0]["items"]
        self.assertEqual([k for k, _, _ in items], ["geprueft", "geprueft", "manuell", "test", "ohne"])
        self.assertEqual(items[1][2], ["Prüfung vom 16.11.2026"])

    def test_fea_88_ak3_summary_comments_mark_all_criteria_except_the_listed_ones(self):
        summary = "Prüfung vom 15.11.2026 (Claude).\n\nErgebnis: 2 von 3 Kriterien ok.\n\nNicht (voll) belegt:\n- AK2 nicht geprüft: fehlt"
        self.assertEqual(sorted(report.checked([{"createdAt": "2026-11-15", "body": summary}], 3)), [1, 3])
        self.assertEqual(sorted(report.checked([{"createdAt": "2026-11-15", "body": "**Prüfung (Einzelprüfung auf Anweisung)**\n\nAlle drei Kriterien erfüllt:\n- AK1, AK2: x"}], 3)), [1, 2, 3])
        self.assertEqual(report.checked([{"createdAt": "2026-11-15", "body": "**Prüfung**\n\nNoch nicht umgesetzt, alle fünf Kriterien offen."}], 5), {})
        self.assertEqual(report.checked([{"createdAt": "2026-11-15", "body": "Umgesetzt in abc1234. Abnahme ausstehend.\n- AK1 erfüllt"}], 3), {})

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


class Page(unittest.TestCase):
    """Die ganze Seite wird in einem leeren Arbeitsordner erzeugt (ohne Zugriff auf das Repository und auf GitHub)."""

    def render(self, setup=None, gh=None):
        with tempfile.TemporaryDirectory() as d:
            root = Path(d)
            (root / "Cargo.toml").write_text('[workspace.package]\nversion = "9.9.9"\n')
            (root / ".github" / "workflows").mkdir(parents=True)
            (root / ".github" / "workflows" / "test-quality.yml").write_text("PROPTEST_CASES: ${{ x && '256' || '32' }}\n")
            if setup:
                setup(root)
            cwd, old_root, old_sh, old_argv = os.getcwd(), report.ROOT, report.sh, sys.argv
            os.chdir(root)
            report.ROOT = root
            report.sh = lambda *cmd, check=False: (gh(cmd) if cmd[0] == "gh" else "") if gh else None
            sys.argv = ["report.py", "--out", str(root / "out")]
            try:
                report.main()
            finally:
                os.chdir(cwd)
                report.ROOT, report.sh, sys.argv = old_root, old_sh, old_argv
            return (root / "out" / "index.html").read_text(encoding="utf-8")

    def test_fea_30_ak3_missed_mutants_are_listed_one_by_one_with_file_and_line(self):
        # FEA-30/AK3
        def setup(root):
            m = root / "local" / "mutants.out"
            m.mkdir(parents=True)
            (m / "outcomes.json").write_text('{"caught": 7, "missed": 2, "unviable": 1, "timeout": 0}')
            (m / "missed.txt").write_text("crates/engine/src/a.rs:12:5: replace f -> bool with true\ncrates/engine/src/b.rs:40:9: delete -\n")
        page = self.render(setup)
        self.assertIn("Nicht erkannte Mutanten (2)", page)
        self.assertIn("crates/engine/src/a.rs:12:5: replace f -&gt; bool with true", page)
        self.assertIn("crates/engine/src/b.rs:40:9: delete -", page)

    def test_fea_30_ak4_without_a_mutation_run_the_area_says_no_data(self):
        # FEA-30/AK4
        page = self.render()
        self.assertIn("kein Mutationslauf vorhanden", page)
        self.assertNotIn("Traceback", page)

    def test_fea_31_ak2_a_found_in_use_bug_shows_cause_and_regression_test(self):
        # FEA-31/AK2
        import json
        bug = {"number": 9, "title": "ISS-9 x", "state": "OPEN", "labels": [{"name": "bug"}, {"name": "gefunden:nutzung"}],
               "body": "Text\n\n**Welcher Test hätte ihn fangen müssen?** Ein Rundungstest.\n\n**Regressionstest:** crates/engine/tests/x.rs\n"}
        page = self.render(gh=lambda cmd: json.dumps([bug]))
        ergebnis = page[page.index("4 Ergebnis"):page.index("Verlauf")]
        self.assertIn("nutzung", ergebnis)
        self.assertIn("Ein Rundungstest.", ergebnis)
        self.assertIn("crates/engine/tests/x.rs", ergebnis)

    def test_fea_31_ak3_comparison_with_the_previous_version_shows_changes_per_figure(self):
        # FEA-31/AK3
        import json
        def setup(root):
            h = root / "local" / "test-quality" / "history"
            h.mkdir(parents=True)
            (h / "9.9.8.json").write_text(json.dumps({"coverage_engine": 80.0, "coverage_app": 85.0, "missed": 5, "flaky": 0, "criteria_share": 40.0}))
            m = root / "local" / "mutants.out"
            m.mkdir(parents=True)
            (m / "outcomes.json").write_text('{"caught": 7, "missed": 2}')
        page = self.render(setup)
        verlauf = page[page.index("<h2>Verlauf</h2>"):]
        self.assertIn("Vergleich mit Version 9.9.8", verlauf)
        self.assertIn("Nicht erkannte Mutanten", verlauf)
        self.assertIn("3,0 ▲", verlauf)  # 5 → 2 nicht erkannte Mutanten: weniger ist besser

    def test_fea_31_ak4_without_gh_the_two_layers_say_no_data_and_the_rest_stays_complete(self):
        # FEA-31/AK4
        page = self.render(gh=None)
        self.assertEqual(page.count("<code>gh</code> ist nicht angemeldet oder nicht erreichbar"), 2)
        for abschnitt in ("1 Hygiene", "2 Angemessenheit", "3 Anforderungsbezug", "4 Ergebnis", "Verlauf"):
            self.assertIn(abschnitt, page)


if __name__ == "__main__":
    unittest.main()
