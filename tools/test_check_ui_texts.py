"""Tests for tools/check_ui_texts.py (#170/AK1).

Run: python3 -m unittest discover -s tools -p 'test_check_ui_texts.py'
Each case writes files into a temporary folder and runs the check on it (option --root).
"""

import contextlib
import io
import pathlib
import sys
import tempfile
import textwrap
import unittest

TOOLS = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(TOOLS))
import check_ui_texts  # noqa: E402


def run(files):
    with tempfile.TemporaryDirectory() as d:
        root = pathlib.Path(d)
        for rel, content in files.items():
            p = root / rel
            p.parent.mkdir(parents=True, exist_ok=True)
            p.write_text(textwrap.dedent(content).lstrip("\n"), encoding="utf-8")
        out = io.StringIO()
        with contextlib.redirect_stdout(out):
            code = check_ui_texts.main(["--root", str(root)])
        return code, out.getvalue()


class CheckUiTexts(unittest.TestCase):
    def test_issue_170_ak1_german_strings_in_the_user_interface_are_reported(self):
        code, out = run({
            "crates/app/ui/app.js": """
                const a = tr("menu.file");
                const b = "Neue Planung";
                const c = `<b>${tr("x")}</b> ab ${n}`;
                const d = `<span title="Zeile entfernen">✕</span>`;
                const e = `<div class="btn small" data-action="run">${x}</div>`;
                """,
        })
        self.assertEqual(code, 1)
        self.assertIn("crates/app/ui/app.js:2: Neue Planung", out)
        self.assertIn("crates/app/ui/app.js:3:", out)
        self.assertIn("crates/app/ui/app.js:4:", out)
        self.assertNotIn("app.js:1:", out)
        self.assertNotIn("app.js:5:", out)

    def test_nested_templates_comments_and_regex_are_handled(self):
        code, out = run({
            "crates/app/ui/a.js": """
                // Kommentar mit Umlaut: ä
                const r = /^(Prüfung)/;
                const t = `${xs.map(x => `<li>${x}</li>`).join("")}`;
                const u = `${xs.map(x => `<li>Läufe ${x}</li>`).join("")}`;
                """,
        })
        self.assertEqual(code, 1)
        self.assertEqual(out.count("a.js:"), 1)
        self.assertIn("a.js:4:", out)

    def test_allow_marker_html_and_rust_without_tests(self):
        code, out = run({
            "crates/app/ui/a.js": 'const x = "Abgebrochen nicht"; // i18n: allow (engine)\n',
            "crates/app/ui/index.html": """
                <head><title>VivaPlan</title></head>
                <body>
                <button data-i18n="page.new"></button>
                <span title="Öffnen">x</span>
                <p>Willkommen bei VivaPlan</p>
                <script src="i18n.js" data-load="app.js"></script>
                </body>
                """,
            "crates/app/src/main.rs": """
                const T: &str = "menu.file";
                const U: &str = "Datei öffnen";
                #[cfg(test)]
                mod tests {
                    const V: &str = "Datei öffnen";
                }
                """,
        })
        self.assertEqual(code, 1)
        self.assertNotIn("a.js", out)
        self.assertIn("crates/app/ui/index.html:4:", out)
        self.assertIn("crates/app/ui/index.html:5:", out)
        self.assertNotIn("index.html:3:", out)
        self.assertIn("crates/app/src/main.rs:2: Datei öffnen", out)
        self.assertNotIn("main.rs:5", out)

    def test_clean_files_pass(self):
        code, out = run({"crates/app/ui/a.js": 'const a = tr("x", { n });\nconst b = "btn small";\n'})
        self.assertEqual((code, out), (0, ""))

    def test_the_repository_is_clean(self):
        out = io.StringIO()
        with contextlib.redirect_stdout(out):
            code = check_ui_texts.main(["--root", str(TOOLS.parent)])
        self.assertEqual(code, 0, out.getvalue())


if __name__ == "__main__":
    unittest.main()
