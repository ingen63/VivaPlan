"""Tests for tools/i18n_review_table.py (#182/AK1).

Run: python3 -m unittest discover -s tools -p 'test_i18n_review_table.py'
Each case writes text tables and help files into a temporary folder and builds the table from it (option --root).
"""

import contextlib
import csv
import io
import json
import pathlib
import sys
import tempfile
import unittest

TOOLS = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(TOOLS))
import i18n_review_table  # noqa: E402

UI = pathlib.Path("crates/app/ui")
HELP = {
    "de": "# Editor\n\n## @s-general Haushalt\nStichworte: Name, Geburt\n\nErster Absatz.\nZweite Zeile.\n\n- Punkt\n",
    "en": "# Editor\n\n## @s-general Household\nKeywords: name, birth\n\nFirst paragraph.\nSecond line.\n\n- Item\n",
    "fr": "# Éditeur\n\n## @s-general Ménage\nMots-clés : nom, naissance\n\nPremier.\nDeuxième.\n\n- Point\n",
    "it": "# Editor\n\n## @s-general Economia\nParole chiave: nome, nascita\n\nPrimo.\nSeconda.\n\n- Punto\n",
}
TABLES = {
    "de": {"general.name": "Name", "engine.warn": "Warnung", "menu.file": "Datei"},
    "en": {"general.name": "Name", "engine.warn": "Warning", "menu.file": "File"},
    "fr": {"general.name": "Nom", "engine.warn": "Avertissement", "menu.file": "Fichier"},
    "it": {"general.name": "Nome", "engine.warn": "Avviso", "menu.file": "File"},
}
APP_JS = 'x = F("project.name", tr("general.name"), "text");\n'


def run(help_files=HELP):
    with tempfile.TemporaryDirectory() as d:
        root = pathlib.Path(d)
        for lang, table in TABLES.items():
            p = root / UI / "i18n" / f"{lang}.json"
            p.parent.mkdir(parents=True, exist_ok=True)
            p.write_text(json.dumps(table, ensure_ascii=False), encoding="utf-8")
        for lang, text in help_files.items():
            p = root / UI / "hilfe" / lang / "10-editor.md"
            p.parent.mkdir(parents=True, exist_ok=True)
            p.write_text(text, encoding="utf-8")
        (root / UI / "app.js").write_text(APP_JS, encoding="utf-8")
        out = root / "tabelle.csv"
        err = io.StringIO()
        with contextlib.redirect_stdout(io.StringIO()), contextlib.redirect_stderr(err):
            code = i18n_review_table.main(["--root", str(root), "--out", str(out)])
        raw = out.read_bytes() if out.exists() else b""
        rows = list(csv.reader(io.StringIO(raw.decode("utf-8-sig")), delimiter=";")) if raw else []
        return code, rows, raw, err.getvalue()


class ReviewTable(unittest.TestCase):
    def test_issue_182_ak1_header_and_one_row_per_key(self):
        code, rows, raw, _ = run()
        self.assertEqual(code, 0)
        self.assertTrue(raw.startswith(b"\xef\xbb\xbf"), "BOM for Excel")
        self.assertEqual(rows[0], i18n_review_table.HEADER)
        by_key = {r[0]: r for r in rows[1:]}
        self.assertEqual(by_key["engine.warn"][3:], ["Warnung", "Warning", "Avertissement", "Avviso", "", "", "", ""])
        self.assertEqual(by_key["menu.file"][1], "Menüs")

    def test_issue_182_ak1_critical_texts(self):
        _, rows, _, _ = run()
        crit = {r[0]: r[2] for r in rows[1:]}
        self.assertEqual(crit["engine.warn"], "x")  # message of the engine
        self.assertEqual(crit["general.name"], "x")  # label of an input field
        self.assertEqual(crit["menu.file"], "")

    def test_issue_182_ak1_help_blocks_aligned(self):
        _, rows, _, _ = run()
        help_rows = [r for r in rows[1:] if r[0].startswith("hilfe/")]
        self.assertEqual([r[0] for r in help_rows], [f"hilfe/10-editor.md §{i}" for i in range(1, 6)])
        keywords = help_rows[2]
        self.assertEqual(keywords[2], "x")
        self.assertEqual(keywords[5], "Mots-clés : nom, naissance")
        self.assertEqual(keywords[1], "Hilfe › Editor › Haushalt")
        self.assertEqual(help_rows[3][4], "First paragraph.\nSecond line.")
        self.assertEqual(help_rows[3][2], "")

    def test_issue_182_ak1_different_structure_is_an_error(self):
        broken = dict(HELP, fr=HELP["fr"] + "\nAbsatz zu viel.\n")
        code, rows, _, err = run(broken)
        self.assertEqual(code, 1)
        self.assertEqual(rows, [])
        self.assertIn("10-editor.md", err)

    def test_issue_182_ak1_parts_critical_first_and_limited(self):
        def row(key, crit, text):
            return [key, "", "x" if crit else "", text, text, text, text]
        rows = [row("a", False, "aa"), row("b", True, "bbb"), row("c", False, "c" * 30), row("d", True, "d")]
        parts = i18n_review_table.split_parts(rows, 20)
        self.assertEqual([[r[0] for r in p] for p in parts], [["b", "d"], ["a"], ["c"]])
        self.assertEqual(sum(len(p) for p in parts), len(rows))


class GlossaryExcerpt(unittest.TestCase):
    def test_issue_182_ak1_glossary_section_of_the_translations(self):
        with tempfile.TemporaryDirectory() as d:
            root = pathlib.Path(d)
            (root / "docs").mkdir()
            (root / "docs" / "GLOSSAR.md").write_text(
                "# G\n\n## Code\nx\n\n## Übersetzungen\n\n| DE | EN |\n\n## Danach\ny\n", encoding="utf-8")
            self.assertEqual(i18n_review_table.glossary_translations(root), "## Übersetzungen\n\n| DE | EN |\n")
            self.assertIsNone(i18n_review_table.glossary_translations(root / "missing"))


if __name__ == "__main__":
    unittest.main()
