"""Tests for the overview page and the diff coverage (#39, #41, #42). Usage: python3 -m unittest discover tools/report"""
import os
import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, os.path.dirname(__file__))
import report  # noqa: E402  (changes into the repository directory)
import coverage_diff  # noqa: E402


class Status(unittest.TestCase):
    def test_playwright_run_time_comes_from_the_run_not_from_the_file(self):
        # #89/AK3: results.json is downloaded in the pipeline; the file time would be that of the download
        self.assertRegex(report.run_time("2026-11-14T22:01:05.375Z"), r"^\d\d\.\d\d\.\d{4} \d\d:\d\d$")
        self.assertIsNone(report.run_time(None))
        self.assertIsNone(report.run_time("no date"))

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
        self.assertEqual(report.status_fit(pk(95, 79.9), mut(0)), report.FAIL)  # packages are not averaged
        self.assertEqual(report.status_fit(pk(95, 85, 79.9), mut(0)), report.FAIL)  # #105/AK3: the command line counts too

    def test_junit_with_a_failing_test_is_reported_by_name(self):
        xml = '<testsuites><testsuite><testcase classname="engine::x" name="broken" time="0.1"><failure/></testcase><testcase classname="engine::x" name="ok" time="0.2"/><testcase classname="engine::x" name="wobbly" time="0.3"><flakyFailure/></testcase></testsuite></testsuites>'
        with tempfile.TemporaryDirectory() as d:
            (Path(d) / "junit.xml").write_text(xml)
            cwd = os.getcwd()
            os.chdir(d)
            try:
                j = report.read_junit()
            finally:
                os.chdir(cwd)
        self.assertEqual(j["failed"], ["engine::x broken"])
        self.assertEqual(j["flaky"], ["engine::x wobbly"])
        self.assertEqual(report.status_hygiene(j), report.FAIL)


class Requirements(unittest.TestCase):
    ISSUE = {"number": 7, "title": "Example", "state": "OPEN", "labels": [], "body": "Text\n\n- [ ] one\n- [x] two\n- [ ] three\n"}

    def test_two_of_three_criteria_with_a_marked_test(self):
        marks = {(7, 1): ["a.rs:3"], (7, 3): ["b.rs:9"]}
        rows = report.requirements([self.ISSUE], marks)
        kinds = [k for k, _, _ in rows[0]["items"]]
        self.assertEqual(kinds, ["test", "manual", "test"])  # 2 of 3 covered, the ticked one only manually

    def test_issue_117_ak5_markers_in_comments_and_test_names(self):
        # #117/AK5: `#<issue>/AK<n>` in a comment or test name, `issue_<issue>_ak<n>_…` in a Rust test name
        # The examples are concatenated so that the page does not count them as tests of real issues.
        self.assertEqual(report.MARK.search("// #115/" "AK2 checks").groups()[:2], ("115", "2"))
        self.assertEqual(report.NAMEMARK.search("fn issue_115_" "ak2_rounding()").groups()[:2], ("115", "2"))
        marks = report.scan_marks("a.rs", ["// #7/" "AK1 checks", "fn issue_7_" "ak3_rounding() {}"], {})
        self.assertEqual(sorted(marks), [(7, 1), (7, 3)])

    def test_issue_117_ak5_old_ids_are_no_longer_recognised(self):
        # #117/AK5: the former forms (prefix with hyphen and number, as a Rust name with underscore) no longer count;
        # HTML entities are not a number. The old ids are concatenated so that `git grep` does not find them.
        src = ["// FE" "A-100/AK2", "fn fe" "a_100_ak2_x() {}", "// IS" "S-3/AK1", "x = '&#39;/AK1'"]
        self.assertEqual(report.scan_marks("a.rs", src, {}), {})
        self.assertEqual(report.scan_marks("e2e/x.spec.js", ['test.describe("FE" "A-56 old", () => {', '  test("AK1 x", async () => {});', "});"], {}), {})

    def test_marks_are_found_in_unit_tests_fuzz_targets_and_tests_of_the_tools(self):
        # Issue #0 does not exist, so these examples never count for a real criterion.
        files = {"crates/engine/tests/a.rs": "// #0/AK1", "e2e/x.spec.js": "// #0/AK2", "e2e/unit/x.test.js": "// #0/AK3",
                 "fuzz/fuzz_targets/x.rs": "// #0/AK4", "tools/test_x.py": "# #0/AK5", "tools/report/test_y.py": "# #0/AK6",
                 "tools/check_x.py": "# #0/AK7", "e2e/node_modules/p/test_z.js": "// #0/AK8",
                 "fuzz/target/test_t.py": "# #0/AK9"}
        with tempfile.TemporaryDirectory() as d:
            for name, text in files.items():
                (Path(d) / name).parent.mkdir(parents=True, exist_ok=True)
                (Path(d) / name).write_text(text + "\n")
            cwd = os.getcwd()
            os.chdir(d)
            try:
                marks = report.test_marks()
            finally:
                os.chdir(cwd)
        # The tools themselves (check_x.py), node_modules and build folders do not count
        self.assertEqual(sorted(marks), [(0, 1), (0, 2), (0, 3), (0, 4), (0, 5), (0, 6)])

    def test_leading_zero_in_a_mark_is_the_same_issue(self):
        marks = report.scan_marks("e2e/x.spec.js", ['test("#07/' 'AK1 …", () => {});', "// #7/" "AK3"], {})
        kinds = [k for k, _, _ in report.requirements([self.ISSUE], marks)[0]["items"]]
        self.assertEqual(kinds, ["test", "manual", "test"])

    def test_issue_117_ak5_bare_ak_in_a_describe_block_with_one_issue_number(self):
        src = ['test.describe("Result in words (#83)", () => {',
               '  test("AK1, AK3: several runs", async () => {',
               '  });',
               '  test.describe("Subgroup without a number", () => {',
               '    test("AK2 in the inner block", async () => {});',
               '  });',
               '});',
               'test("AK4 outside any block", async () => {});',
               'test.describe("#81 and #82", () => {',
               '  test("AK5 ambiguous", async () => {});',
               '});']
        marks = report.scan_marks("e2e/x.spec.js", src, {})
        self.assertEqual(sorted(marks), [(83, 1), (83, 2), (83, 3)])
        self.assertEqual(marks[(83, 1)], ["e2e/x.spec.js:2"])
        # Rust files have no describe blocks
        self.assertEqual(report.scan_marks("a.rs", src, {}), {})

    def test_issues_without_id_in_the_title_are_counted_by_number(self):
        # #117/AK5: the mapping uses the number, not the title
        rows = report.requirements([dict(self.ISSUE, title="Title without prefix")], {(7, 2): ["t.rs:1"]})
        self.assertEqual(rows[0]["nr"], 7)
        self.assertEqual([k for k, _, _ in rows[0]["items"]], ["none", "test", "none"])

    def test_issue_113_ak3_checked_criteria_count_as_checked_not_manual(self):
        c = lambda day, body: {"createdAt": f"{day}T10:00:00Z", "body": body}
        issue = dict(self.ISSUE, body="- [x] a\n- [x] b\n- [x] c\n- [x] d\n- [ ] e\n", comments=[
            c("2026-11-15", "Prüfung #7 (15.11.2026)\n\n- **AK1 erfüllt.** Test grün.\n- **AK2 nicht ganz erfüllt.** Rest.\n- AK3, AK4 ok: gelesen."),
            c("2026-11-16", "Nacharbeit AK2 in abc1234: erledigt.\n\n- **AK2 erfüllt.**"),
            c("2026-11-16", "Prüfung vom 16.11.2026 durch den Anwender in der App: AK4 ist in Ordnung."),
            c("2026-11-17", "**Erneute Prüfung**\n\n- AK3 offen: Gegenprobe fehlt."),
        ])
        items = report.requirements([issue], {(7, 4): ["t.rs:1"]})[0]["items"]
        self.assertEqual([k for k, _, _ in items], ["checked", "checked", "manual", "test", "none"])
        self.assertEqual(items[1][2], ["Prüfung vom 16.11.2026"])

    def test_issue_113_ak3_summary_comments_mark_all_criteria_except_the_listed_ones(self):
        summary = "Prüfung vom 15.11.2026 (Claude).\n\nErgebnis: 2 von 3 Kriterien ok.\n\nNicht (voll) belegt:\n- AK2 nicht geprüft: fehlt"
        self.assertEqual(sorted(report.checked([{"createdAt": "2026-11-15", "body": summary}], 3)), [1, 3])
        self.assertEqual(sorted(report.checked([{"createdAt": "2026-11-15", "body": "**Prüfung (Einzelprüfung auf Anweisung)**\n\nAlle drei Kriterien erfüllt:\n- AK1, AK2: x"}], 3)), [1, 2, 3])
        self.assertEqual(report.checked([{"createdAt": "2026-11-15", "body": "**Prüfung**\n\nNoch nicht umgesetzt, alle fünf Kriterien offen."}], 5), {})
        self.assertEqual(report.checked([{"createdAt": "2026-11-15", "body": "Umgesetzt in abc1234. Abnahme ausstehend.\n- AK1 erfüllt"}], 3), {})

    def test_issue_192_ak1_lists_and_ranges_in_comments_and_test_names(self):
        # #192/AK1: the examples of the issue; the marks are concatenated so that they do not count as real tests
        cases = {"// #98/" "AK1–AK3 checks": {(98, 1), (98, 2), (98, 3)},
                 "// #98/" "AK1-AK3": {(98, 1), (98, 2), (98, 3)},
                 "// #76/" "AK1, AK3": {(76, 1), (76, 3)},
                 "// #85/" "AK1 und AK2": {(85, 1), (85, 2)},
                 "fn issue_53_" "ak4_ak5_rounding() {}": {(53, 4), (53, 5)},
                 "// #7/" "AK1 und #8/" "AK2": {(7, 1), (8, 2)},
                 "// #7/" "AK5–AK2 backwards": {(7, 5), (7, 2)}}
        for line, want in cases.items():
            with self.subTest(line=line):
                self.assertEqual(set(report.scan_marks("a.rs", [line], {})), want)
        # JS (comment and test title) and Python (comment and test name)
        js = report.scan_marks("e2e/x.spec.js", ['test("#98/' 'AK1–AK3 x", () => {});', "// #76/" "AK1, AK3"], {})
        self.assertEqual(sorted(js), [(76, 1), (76, 3), (98, 1), (98, 2), (98, 3)])
        py = report.scan_marks("tools/test_x.py", ["    def test_issue_53_" "ak4_ak5_x(self):", "        # #85/" "AK1 und AK2"], {})
        self.assertEqual(sorted(py), [(53, 4), (53, 5), (85, 1), (85, 2)])

    def test_issue_192_ak2_bare_list_in_an_e2e_describe_block_counts_for_both(self):
        # #192/AK2
        src = ['test.describe("Language (#170)", () => {', '  test("AK3, AK9: menu", async () => {});',
               '  test("AK1–AK2 range", async () => {});', "});"]
        self.assertEqual(sorted(report.scan_marks("e2e/x.spec.js", src, {})), [(170, 1), (170, 2), (170, 3), (170, 9)])

    def test_issue_192_ak3_orphan_marks_are_listed_with_location_issue_0_excepted(self):
        # #192/AK3: issue missing or criterion missing → orphan; #0 never
        marks = {(7, 3): ["a.rs:1"], (7, 4): ["b.rs:2", "c.rs:3"], (99, 1): ["d.rs:4"], (0, 12): ["e.rs:5"]}
        self.assertEqual(report.orphans([self.ISSUE], marks), [(7, 4, ["b.rs:2", "c.rs:3"]), (99, 1, ["d.rs:4"])])

    def test_issue_192_ak6_criterion_added_in_a_comment_exists(self):
        # #192/AK6: like the added criteria of #69 (AK8) and #170 (AK9)
        issue = dict(self.ISSUE, comments=[
            {"createdAt": "2026-11-20T10:00:00Z", "body": "**Ergänzung:** neu\n\n- [x] AK4: added later\n- [ ] AK2: no replacement"},
            {"createdAt": "2026-11-21T10:00:00Z", "body": "- [ ] without number\n- AK5 erfüllt"}])
        crit = report.criteria(issue)
        self.assertEqual(sorted(crit), [1, 2, 3, 4])
        self.assertEqual(crit[2], ("x", "two"))  # a comment does not replace a criterion of the text
        self.assertEqual(crit[4], ("x", "AK4: added later"))
        marks = {(7, 4): ["t.rs:1"], (7, 5): ["t.rs:2"]}
        self.assertEqual(report.orphans([issue], marks), [(7, 5, ["t.rs:2"])])
        items = report.requirements([issue], marks)[0]["items"]
        self.assertEqual([k for k, _, _ in items], ["none", "manual", "none", "test"])

    def test_escapes_list_found_where_and_regression_test(self):
        bug = {"number": 9, "title": "Bug x", "state": "OPEN", "labels": [{"name": "bug"}, {"name": "found: usage"}],
               "body": "Text\n\n**Welcher Test hätte ihn fangen müssen?** A rounding test.\n\n**Regressionstest:** crates/engine/tests/x.rs\n"}
        other = {"number": 10, "title": "Bug y", "state": "OPEN", "labels": [{"name": "bug"}], "body": ""}
        rows = report.escapes([bug, other])
        self.assertEqual(len(rows), 1)
        self.assertEqual((rows[0]["found"], rows[0]["which"], rows[0]["reg"]), ("usage", "A rounding test.", "crates/engine/tests/x.rs"))


class DiffCoverage(unittest.TestCase):
    def test_ranges(self):
        self.assertEqual(coverage_diff.ranges([5, 3, 4, 9]), "3–5, 9")

    def test_base_comes_from_diff_base(self):
        # #39: in the workflow DIFF_BASE sets the state before the push; without it, origin/main applies.
        with tempfile.NamedTemporaryFile("w", suffix=".info", delete=False) as f:
            f.write("SF:crates/engine/src/lib.rs\nDA:1,1\nend_of_record\n")
        old = os.environ.get("DIFF_BASE")
        try:
            os.environ["DIFF_BASE"] = "HEAD"
            self.assertEqual(report.read_diff_coverage(f.name)["base"], "HEAD")
            os.environ.pop("DIFF_BASE")
            r = report.read_diff_coverage(f.name)
            self.assertTrue(r is None or r["base"] == "origin/main")  # without origin/main (shallow clone) no value
        finally:
            os.unlink(f.name)
            if old is not None:
                os.environ["DIFF_BASE"] = old


class Page(unittest.TestCase):
    """The whole page is created in an empty working folder (without access to the repository and to GitHub)."""

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

    def test_issue_41_ak3_missed_mutants_are_listed_one_by_one_with_file_and_line(self):
        # #41/AK3
        def setup(root):
            m = root / "local" / "mutants.out"
            m.mkdir(parents=True)
            (m / "outcomes.json").write_text('{"caught": 7, "missed": 2, "unviable": 1, "timeout": 0}')
            (m / "missed.txt").write_text("crates/engine/src/a.rs:12:5: replace f -> bool with true\ncrates/engine/src/b.rs:40:9: delete -\n")
        page = self.render(setup)
        self.assertIn("Nicht erkannte Mutanten (2)", page)
        self.assertIn("crates/engine/src/a.rs:12:5: replace f -&gt; bool with true", page)
        self.assertIn("crates/engine/src/b.rs:40:9: delete -", page)

    def test_issue_41_ak4_without_a_mutation_run_the_area_says_no_data(self):
        # #41/AK4
        page = self.render()
        self.assertIn("kein Mutationslauf vorhanden", page)
        self.assertNotIn("Traceback", page)

    def test_issue_42_ak2_a_found_in_use_bug_shows_cause_and_regression_test(self):
        # #42/AK2
        import json
        bug = {"number": 9, "title": "Bug x", "state": "OPEN", "labels": [{"name": "bug"}, {"name": "found: usage"}],
               "body": "Text\n\n**Welcher Test hätte ihn fangen müssen?** A rounding test.\n\n**Regressionstest:** crates/engine/tests/x.rs\n"}
        page = self.render(gh=lambda cmd: json.dumps([bug]))
        outcome = page[page.index("4 Ergebnis"):page.index("Verlauf")]
        self.assertIn("usage", outcome)
        self.assertIn("A rounding test.", outcome)
        self.assertIn("crates/engine/tests/x.rs", outcome)

    def test_issue_42_ak3_comparison_with_the_previous_version_shows_changes_per_figure(self):
        # #42/AK3
        import json
        def setup(root):
            h = root / "local" / "test-quality" / "history"
            h.mkdir(parents=True)
            (h / "9.9.8.json").write_text(json.dumps({"coverage_engine": 80.0, "coverage_app": 85.0, "missed": 5, "flaky": 0, "criteria_share": 40.0}))
            m = root / "local" / "mutants.out"
            m.mkdir(parents=True)
            (m / "outcomes.json").write_text('{"caught": 7, "missed": 2}')
        page = self.render(setup)
        history = page[page.index("<h2>Verlauf</h2>"):]
        self.assertIn("Vergleich mit Version 9.9.8", history)
        self.assertIn("Nicht erkannte Mutanten", history)
        self.assertIn("3,0 ▲", history)  # 5 → 2 missed mutants: fewer is better

    def test_issue_192_ak3_page_lists_orphan_marks_with_location(self):
        # #192/AK3: the mark of a missing issue appears with its location; #0 does not
        import json
        def setup(root):
            (root / "crates" / "engine" / "tests").mkdir(parents=True)
            (root / "crates" / "engine" / "tests" / "a.rs").write_text("// #4711/" "AK2 x\n// #0/" "AK9 example\n")
        page = self.render(setup, gh=lambda cmd: json.dumps([]))
        self.assertIn("Verwaiste Kennungen (1)", page)
        self.assertIn("#4711/" "AK2</code>: <small><code>crates/engine/tests/a.rs:1</code>", page)
        self.assertNotIn("#0/" "AK9", page)

    def test_issue_42_ak4_without_gh_the_two_layers_say_no_data_and_the_rest_stays_complete(self):
        # #42/AK4
        page = self.render(gh=None)
        self.assertEqual(page.count("<code>gh</code> ist nicht angemeldet oder nicht erreichbar"), 2)
        for section in ("1 Hygiene", "2 Angemessenheit", "3 Anforderungsbezug", "4 Ergebnis", "Verlauf"):
            self.assertIn(section, page)


if __name__ == "__main__":
    unittest.main()
