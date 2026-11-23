"""Tests for tools/slow_tests.py (#226). Usage: python3 -m unittest discover -s tools -p 'test_slow_tests.py'"""

import io
import pathlib
import sys
import tempfile
import unittest
from contextlib import redirect_stdout

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import slow_tests  # noqa: E402

ENGINE_LIB = """\
#[cfg(test)]
mod tests {
    #[test]
    fn quick() {}

    #[test]
    fn heavy_unit() {
        let x = 1;
    }

    // Covers all cantons.
    // slow: every canton is computed once
    #[test]
    fn justified_unit() {}
}
"""
ENGINE_IT = """\
#[test]
fn fast_it() {}

#[test]
fn slow_it() {}

/// Doc comment
// slow: 600 mutated plans
#[test]
#[ignore = "x"]
fn justified_it() {}

// slow:
#[test]
fn empty_reason_it() {}
"""
CLI_IT = """\
#[test]
fn starts_the_binary() {
    let _ = std::process::Command::new(env!("CARGO_BIN_EXE_vivaplan"));
}
"""
CLI_MAIN = """\
mod tests {
    #[test]
    fn serves_files() {
        std::thread::spawn(move || serve(port, ui, lib));
    }

    #[test]
    fn parses_a_path() {
        let a = 1;
    }
}
"""
APP_MAIN = """\
mod tests {
    fn setup() {}

    #[test]
    fn opens_a_file() {
        let (app, w, _) = setup();
    }
}
"""


def case(binary, name, time):
    return f'<testcase classname="{binary}" name="{name}" time="{time}"/>'


class SlowTests(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        self.root = pathlib.Path(self.tmp.name)
        files = {
            "crates/engine/Cargo.toml": '[package]\nname = "vivaplan-engine"\n',
            "crates/engine/src/lib.rs": ENGINE_LIB,
            "crates/engine/tests/robust.rs": ENGINE_IT,
            "crates/cli/Cargo.toml": '[package]\nname = "vivaplan-cli"\n',
            "crates/cli/tests/cli.rs": CLI_IT,
            "crates/cli/src/devserver.rs": CLI_MAIN,
            "crates/app/Cargo.toml": '[package]\nname = "vivaplan-app"\n',
            "crates/app/src/main.rs": APP_MAIN,
        }
        for path, text in files.items():
            (self.root / path).parent.mkdir(parents=True, exist_ok=True)
            (self.root / path).write_text(text)

    def tearDown(self):
        self.tmp.cleanup()

    def junit(self, *cases):
        f = self.root / "junit.xml"
        f.write_text("<testsuites><testsuite>" + "".join(cases) + "</testsuite></testsuites>")
        return f

    def rows(self, *cases):
        return {r["test"].split(" ", 1)[1]: r for r in slow_tests.check(self.junit(*cases), self.root)}

    def test_issue_226_ak3_tests_under_their_limit_are_not_listed(self):
        rows = self.rows(case("vivaplan-engine", "tests::quick", 0.1), case("vivaplan-engine::robust", "fast_it", 0.9),
                         case("vivaplan-cli::cli", "starts_the_binary", 14.9))
        self.assertEqual(rows, {})

    def test_issue_226_ak1_ak3_tests_over_their_limit_without_reason_with_name_and_duration(self):
        rows = self.rows(case("vivaplan-engine", "tests::heavy_unit", 0.25), case("vivaplan-engine::robust", "slow_it", 1.5))
        self.assertEqual(rows["tests::heavy_unit"]["kind"], slow_tests.UNIT)
        self.assertEqual(rows["tests::heavy_unit"]["time"], 0.25)
        self.assertEqual(rows["tests::heavy_unit"]["place"], "crates/engine/src/lib.rs:7")
        self.assertFalse(rows["tests::heavy_unit"]["reason"])
        self.assertEqual(rows["slow_it"]["kind"], slow_tests.INTEGRATION)
        self.assertFalse(rows["slow_it"]["reason"])
        out = slow_tests.report(list(rows.values()))
        self.assertIn("vivaplan-engine tests::heavy_unit", out)
        self.assertIn("0.25 s", out)
        self.assertIn("vivaplan-engine::robust slow_it", out)

    def test_issue_226_ak3_tests_over_their_limit_with_reason_are_accepted(self):
        rows = self.rows(case("vivaplan-engine", "tests::justified_unit", 3), case("vivaplan-engine::robust", "justified_it", 90))
        self.assertTrue(rows["tests::justified_unit"]["reason"])
        self.assertTrue(rows["justified_it"]["reason"])  # doc comment and attributes in between are fine
        out = slow_tests.report(list(rows.values()))
        self.assertIn("0 Tests über ihrer Grenze", out)
        self.assertNotIn("justified", out)

    def test_issue_226_ak3_a_reason_needs_text(self):
        rows = self.rows(case("vivaplan-engine::robust", "empty_reason_it", 2))
        self.assertFalse(rows["empty_reason_it"]["reason"])

    def test_tests_that_start_a_program_or_server_have_the_long_limit(self):
        rows = self.rows(case("vivaplan-cli::cli", "starts_the_binary", 16), case("vivaplan-cli::bin/vivaplan", "devserver::tests::serves_files", 2),
                         case("vivaplan-cli::bin/vivaplan", "devserver::tests::parses_a_path", 0.2),
                         case("vivaplan-app::bin/vivaplan-app", "tests::opens_a_file", 14))
        self.assertEqual(rows["starts_the_binary"]["kind"], slow_tests.PROGRAM)
        self.assertNotIn("devserver::tests::serves_files", rows)  # 2 s is under 15 s
        self.assertEqual(rows["devserver::tests::parses_a_path"]["kind"], slow_tests.UNIT)
        self.assertNotIn("tests::opens_a_file", rows)  # Tauri test app

    def test_the_module_path_picks_the_file_in_a_single_test_target(self):
        # The engine builds every file in tests/ as a module of the target `all`.
        (self.root / "crates/engine/tests/other.rs").write_text("// slow: other file\n#[test]\nfn slow_it() {}\n")
        rows = self.rows(case("vivaplan-engine::all", "robust::slow_it", 2), case("vivaplan-engine::all", "other::slow_it", 2))
        self.assertEqual(rows["robust::slow_it"]["place"], "crates/engine/tests/robust.rs:5")
        self.assertFalse(rows["robust::slow_it"]["reason"])
        self.assertEqual(rows["other::slow_it"]["place"], "crates/engine/tests/other.rs:3")
        self.assertTrue(rows["other::slow_it"]["reason"])

    def test_unknown_source_is_still_listed(self):
        rows = self.rows(case("vivaplan-engine::gone", "vanished", 5))
        self.assertIsNone(rows["vanished"]["place"])
        self.assertFalse(rows["vanished"]["reason"])

    def test_issue_226_ak2_markdown_for_the_run_summary(self):
        rows = slow_tests.check(self.junit(case("vivaplan-engine::robust", "slow_it", 1.5),
                                           case("vivaplan-engine::robust", "justified_it", 9)), self.root)
        out = slow_tests.report(rows, markdown=True)
        self.assertIn("### Laufzeitgrenzen der Tests (#226)", out)
        self.assertIn("1 Tests über ihrer Grenze ohne `// slow: <Grund>`, 1 begründet", out)
        self.assertIn("| `vivaplan-engine::robust slow_it` | Integrationstest | 1 s | 1.50 s | crates/engine/tests/robust.rs:5 |", out)

    def test_main_reads_the_file_and_reports_a_missing_one(self):
        self.junit(case("vivaplan-engine::robust", "slow_it", 1.5))
        buf = io.StringIO()
        with redirect_stdout(buf):
            self.assertEqual(slow_tests.main([str(self.root / "junit.xml"), "--root", str(self.root)]), 0)
        self.assertIn("slow_it", buf.getvalue())
        self.assertEqual(slow_tests.main([str(self.root / "missing.xml"), "--root", str(self.root)]), 2)


if __name__ == "__main__":
    unittest.main()
