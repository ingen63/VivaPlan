"""Tests for tools/check-english.sh (#165/AK3, AK4, AK5).

Run: python3 -m unittest discover -s tools -p 'test_check_english.py'
Each case writes source files into a temporary Git repository and runs the script on it (option --root).
"""

import contextlib
import io
import pathlib
import subprocess
import sys
import tempfile
import textwrap
import unittest

TOOLS = pathlib.Path(__file__).resolve().parent
SCRIPT = TOOLS / "check-english.sh"
sys.path.insert(0, str(TOOLS))
import check_english  # noqa: E402


class Repo:
    def __init__(self, files):
        self.dir = tempfile.TemporaryDirectory()
        self.root = pathlib.Path(self.dir.name)
        subprocess.run(["git", "init", "-q", str(self.root)], check=True)
        for rel, content in files.items():
            path = self.root / rel
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(textwrap.dedent(content).lstrip("\n"), encoding="utf-8")

    def run(self, *args):
        return subprocess.run(
            ["bash", str(SCRIPT), "--root", str(self.root), *args], capture_output=True, text=True
        )

    def close(self):
        self.dir.cleanup()


def check(files, *args):
    repo = Repo(files)
    try:
        return repo.run(*args)
    finally:
        repo.close()


def hits(result):
    return [line for line in result.stdout.splitlines() if ": comment: " in line or ": test name: " in line]


class GermanTextTest(unittest.TestCase):
    # #165/AK3: The example from the issue: a German comment is reported with file and line, an English one is not.
    def test_ak3_example_from_issue(self):
        r = check({"crates/demo/src/lib.rs": """
            // AHV pension per month
            fn ahv() {}
            // Entnahme pro Monat
            fn withdrawal() {}
            """})
        self.assertEqual(r.returncode, 1, r.stdout + r.stderr)
        self.assertEqual(hits(r), ["crates/demo/src/lib.rs:3: comment: // Entnahme pro Monat"])

    # #165/AK3: English comments only: no hit, exit code 0.
    def test_ak3_english_passes(self):
        r = check({"crates/demo/src/lib.rs": """
            //! Pillar 3a accounts (#52).
            /// Monthly AHV pension at the start of the payout.
            fn ahv() {} // capped for married couples
            /* Block comment about the pension fund. */
            """})
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        self.assertEqual(hits(r), [])

    # #165/AK3: Umlauts and German words are found in all comment forms of all languages.
    def test_ak3_comment_forms(self):
        r = check({
            "crates/demo/src/lib.rs": """
                /// Säule 3a
                /* Block
                   mit Text */
                fn a() {}
                """,
            "e2e/a.spec.js": """
                const x = 1; // Wert setzen
                /** Hilfe öffnen */
                """,
            "tools/a.py": '''
                """Prüft die Datei."""
                x = 1  # und weiter
                ''',
            "tools/a.sh": """
                #!/usr/bin/env bash
                echo hi # Ausgabe der Zeile
                """,
            "tools/hook": """
                #!/bin/sh
                # Läuft vor dem Push
                """,
        }, "--all")
        self.assertEqual(
            hits(r),
            [
                "crates/demo/src/lib.rs:1: comment: /// Säule 3a",
                "crates/demo/src/lib.rs:3: comment: mit Text",
                "e2e/a.spec.js:1: comment: // Wert setzen",
                "e2e/a.spec.js:2: comment: * Hilfe öffnen",
                "tools/a.py:1: comment: Prüft die Datei.",
                "tools/a.py:2: comment: # und weiter",
                "tools/a.sh:2: comment: # Ausgabe der Zeile",
                "tools/hook:2: comment: # Läuft vor dem Push",
            ],
        )

    # #165/AK3: German test names in Rust, Python and JavaScript are reported; other identifiers are not checked.
    def test_ak3_test_names(self):
        r = check({
            "crates/demo/tests/t.rs": """
                fn hilfsfunktion_fuer_tests() {}
                #[test]
                fn oeffnen_und_speichern() {}
                #[test]
                #[should_panic]
                fn issue_27_ak2_rejects_unknown_command() {}
                #[tokio::test]
                async fn laden_ohne_datei() {}
                """,
            "tools/test_x.py": """
                def test_nur_exakte_namen(self):
                    pass
                def test_exact_names_only(self):
                    pass
                """,
            "e2e/x.spec.js": """
                test.describe("Optimum je Perzentil (#27)", () => {
                  test("AK2: Abbrechen beendet die Optimierung", async () => {});
                  test("AK3: cancel stops the optimisation", async () => {});
                });
                it('Zeile löschen', () => {});
                """,
        }, "--all")
        self.assertEqual(
            hits(r),
            [
                "crates/demo/tests/t.rs:3: test name: oeffnen_und_speichern",
                "crates/demo/tests/t.rs:8: test name: laden_ohne_datei",
                "e2e/x.spec.js:1: test name: Optimum je Perzentil (#27)",
                "e2e/x.spec.js:2: test name: AK2: Abbrechen beendet die Optimierung",
                "e2e/x.spec.js:5: test name: Zeile löschen",
                "tools/test_x.py:1: test name: test_nur_exakte_namen",
            ],
        )


class ExemptionsTest(unittest.TestCase):
    # #165/AK3: User interface texts in string literals are exempt, including text that looks like a comment.
    def test_ak3_strings_are_exempt(self):
        r = check({
            "crates/demo/src/lib.rs": """
                const T: &str = "Entnahme pro Monat // für die Übersicht";
                const R: &str = r#"Säule "3a" /* nicht */"#;
                fn f<'a>(x: &'a str) -> char { let _ = '"'; 'ä' } // English after a char literal
                """,
            "crates/demo/ui/app.js": """
                const label = "Vermögen über die Zeit"; // label of the chart
                const t = `Hilfe ${x} öffnen`;
                const re = /["']/g; // quote characters
                """,
            "tools/r.py": '''
                HTML = """
                <h1>Übersicht der Läufe</h1>
                """
                print('# kein Kommentar')
                ''',
            "tools/s.sh": """
                echo "Prüfung # läuft" $# '# auch nicht'
                cat <<'EOF'
                # Überschrift im Text
                EOF
                """,
        }, "--all")
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)

    # #165/AK3: Test names inside multi-line string literals (fixtures of other tests) are not test names.
    def test_ak3_test_names_in_fixtures_are_exempt(self):
        r = check({
            "crates/demo/tests/t.rs": """
                const SRC: &str = r#"
                #[test]
                fn laden_und_speichern() {}
                "#;
                """,
            "tools/test_y.py": '''
                FIXTURE = """
                def test_nur_exakte_namen(self):
                    pass
                """
                ''',
        }, "--all")
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)

    # #165/AK3: «…» labels, code spans, URLs and words in capitals inside comments do not count.
    def test_ak3_quotations_in_comments(self):
        r = check({"crates/demo/src/lib.rs": """
            // the «Säule 3a» card shows `Vermögen` (see https://de.wikipedia.org/wiki/Säule)
            // MIT licence, AUS and DEN are country codes
            """})
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)

    # #165/AK3: The line marker suppresses a false positive; other lines are still reported.
    def test_ak3_allow_marker(self):
        r = check({"crates/demo/src/lib.rs": """
            // Grüezi is a greeting check-english: allow
            #[test]
            fn gruezi_und_tschuess() {} // check-english: allow
            // Grüezi without the marker
            """})
        self.assertEqual(hits(r), ["crates/demo/src/lib.rs:4: comment: // Grüezi without the marker"])

    # #165/AK3: Golden files and generated directories are exempt.
    def test_ak3_golden_and_generated_are_exempt(self):
        r = check({
            "crates/engine/tests/golden/case.rs": "// Entnahme pro Monat\n",
            "e2e/node_modules/pkg/index.js": "// Entnahme pro Monat\n",
            "fuzz/target/x.rs": "// Entnahme pro Monat\n",
        }, "--all")
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)

    # #165/AK3: Only crates/, e2e/, fuzz/ and tools/ are checked by default, and only source files.
    def test_ak3_scope(self):
        r = check({
            "docs/x.rs": "// Entnahme pro Monat\n",
            "crates/demo/README.md": "Entnahme pro Monat\n",
            "crates/demo/data.json": '{"Entnahme": "pro Monat"}\n',
        })
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)


class TransitionTest(unittest.TestCase):
    FILES = {"crates/engine/src/lib.rs": "// Entnahme pro Monat\n", "crates/demo/src/lib.rs": "// per month\n"}

    # #165/AK4: The transition list holds all paths not yet converted.
    def test_ak4_transition_list_holds_unconverted_paths(self):
        self.assertEqual(
            sorted(check_english.PENDING),
            sorted(["crates/engine/", "crates/app/", "e2e/"]),
        )

    # #165/AK4: Paths on the transition list are skipped; --all checks them anyway.
    def test_ak4_pending_paths_are_skipped(self):
        r = check(self.FILES)
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        self.assertIn("transition list", r.stdout)
        r = check(self.FILES, "--all")
        self.assertEqual(r.returncode, 1)
        self.assertEqual(hits(r), ["crates/engine/src/lib.rs:1: comment: // Entnahme pro Monat"])

    # #165/AK4: An explicit path limits the check, e.g. to one package during its conversion.
    def test_ak4_explicit_path(self):
        r = check(self.FILES, "--all", "crates/demo")
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)

    # #165/AK4: --gate (make test-quiet, workflow) skips while the list is not empty ...
    def test_ak4_gate_skips_while_list_not_empty(self):
        r = check(self.FILES, "--gate")
        self.assertEqual(r.returncode, 0, r.stdout + r.stderr)
        self.assertIn("skipped", r.stdout)

    # #165/AK4: ... and checks everything once the list is empty.
    def test_ak4_gate_checks_with_empty_list(self):
        repo = Repo(self.FILES)
        saved = check_english.PENDING
        check_english.PENDING = []
        try:
            out = io.StringIO()
            with contextlib.redirect_stdout(out), contextlib.redirect_stderr(io.StringIO()):
                code = check_english.main(["--gate", "--root", str(repo.root)])
        finally:
            check_english.PENDING = saved
            repo.close()
        self.assertEqual(code, 1)
        self.assertIn("crates/engine/src/lib.rs:1: comment: // Entnahme pro Monat", out.getvalue())

    # #165/AK4: Wrong usage is an error of its own (2), not a silent pass.
    def test_ak4_usage_error(self):
        r = subprocess.run(["bash", str(SCRIPT), "--root", "/does/not/exist"], capture_output=True, text=True)
        self.assertEqual(r.returncode, 2)
        r = subprocess.run(["bash", str(SCRIPT), "--unknown"], capture_output=True, text=True)
        self.assertEqual(r.returncode, 2)


if __name__ == "__main__":
    unittest.main()
