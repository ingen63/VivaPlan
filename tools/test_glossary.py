"""Tests for the translation glossary in docs/GLOSSAR.md and its reader tools/glossary.py (#179).

Run: python3 -m unittest discover -s tools -p 'test_glossary.py'
"""

import pathlib
import sys
import unittest

TOOLS = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(TOOLS))
import glossary  # noqa: E402

TEXT = (glossary.ROOT / "docs" / "GLOSSAR.md").read_text(encoding="utf-8")
SOURCES = ("AHVG", "BVG", "DBG", "StHG", "ZGB", "MB 3.01", "MB 3.04", "ISO 4217", glossary.NO_SOURCE)

SAMPLE = """# Glossar

## Übersetzungen

### Kürzel

| DE | EN | FR | IT | Quelle | verbotene Varianten |
|---|---|---|---|---|---|
| AHV | AHV | AVS | AVS | AHVG Titel | fr: AHV; it: AHV |

### Fachbegriffe

| DE | EN | FR | IT | Quelle | verbotene Varianten |
|---|---|---|---|---|---|
| Vermögen | wealth | fortune | sostanza | keine amtliche Quelle | fr: patrimoine |
"""


def rows(table):
    return glossary.parse(TEXT)[table]


class ProjectGlossary(unittest.TestCase):
    def test_issue_179_ak1_section_has_both_tables_with_six_columns(self):
        # #179/AK1: translations section with both tables and the six required columns.
        tables = glossary.parse(TEXT)
        self.assertEqual(sorted(tables), ["Fachbegriffe", "Kürzel"])

    def test_issue_179_ak2_minimum_terms_present(self):
        # #179/AK2: the terms named in the issue have a row (abbreviations in either table).
        german = {form for table in ("Kürzel", "Fachbegriffe") for row in rows(table) for form in row["forms"]["de"]}
        required = [
            "AHV", "PK", "BVG", "Säule 3a", "Umwandlungssatz", "Kapitalbezug", "Kapitalbezugssteuer",
            "Eigenmietwert", "Hypothek", "Amortisation", "Teuerung", "Entnahme", "Leitplanke",
            "Monte-Carlo-Erfolgsquote",
        ]
        self.assertEqual([term for term in required if term not in german], [])

    def test_issue_179_ak2_german_terms_unique(self):
        german = [form for row in rows("Fachbegriffe") + rows("Kürzel") for form in row["forms"]["de"]]
        self.assertEqual(sorted({f for f in german if german.count(f) > 1}), [])

    def test_issue_179_ak3_every_source_is_known_or_marked(self):
        # #179/AK3: each source names a listed source or is marked as having no official source.
        bad = []
        for table in ("Kürzel", "Fachbegriffe"):
            for row in rows(table):
                for part in row["source"]:
                    text = part.split(":", 1)[1].strip() if part.startswith(("EN:", "FR:", "IT:")) else part
                    if not text.startswith(SOURCES):
                        bad.append((row["forms"]["de"][0], part))
        self.assertEqual(bad, [])

    def test_issue_179_ak3_official_sources_named_in_legend(self):
        for code in SOURCES[:-1]:
            self.assertIn(f"- `{code}`", TEXT)

    def test_issue_179_ak4_abbreviations_have_form_per_language(self):
        # #179/AK4: own list of abbreviations with their form per language (AHV/AVS/AVS).
        by_de = {row["forms"]["de"][0]: row["forms"] for row in rows("Kürzel")}
        for abbreviation in ("AHV", "BVG", "PK", "3a", "CHF"):
            self.assertIn(abbreviation, by_de)
        self.assertEqual([by_de["AHV"][lang][0] for lang in glossary.LANGS], ["AHV", "AHV", "AVS", "AVS"])
        self.assertEqual([by_de["BVG"][lang][0] for lang in ("fr", "it")], ["LPP", "LPP"])

    def test_issue_179_ak5_cli_reads_glossary(self):
        # #179/AK5: the table is machine-readable without errors.
        self.assertEqual(glossary.main([]), 0)


class Reader(unittest.TestCase):
    def test_sample_rows(self):
        tables = glossary.parse(SAMPLE)
        self.assertEqual(tables["Kürzel"][0]["forms"]["fr"], ["AVS"])
        self.assertEqual(tables["Kürzel"][0]["forbidden"], [("fr", "AHV"), ("it", "AHV")])
        self.assertTrue(tables["Kürzel"][0]["official"])
        self.assertFalse(tables["Fachbegriffe"][0]["official"])

    def test_forms_split_and_dash(self):
        self.assertEqual(glossary.split_forms("a; b c;d"), ["a", "b c", "d"])
        self.assertEqual(glossary.split_forms("-"), [])

    def assertRejects(self, text, message):
        with self.assertRaises(glossary.GlossaryError) as ctx:
            glossary.parse(text)
        self.assertIn(message, str(ctx.exception))

    def test_missing_section(self):
        self.assertRejects("# Glossar\n", "section")

    def test_wrong_cell_count(self):
        self.assertRejects(SAMPLE.replace("| sostanza |", "| sostanza | extra |"), "cells")

    def test_empty_cell(self):
        self.assertRejects(SAMPLE.replace("| fortune |", "|  |"), "empty cell")

    def test_wrong_header(self):
        self.assertRejects(SAMPLE.replace("| Quelle |", "| Source |", 1), "header")

    def test_bad_forbidden_language(self):
        self.assertRejects(SAMPLE.replace("fr: patrimoine", "es: patrimonio"), "forbidden")

    def test_row_without_trailing_pipe(self):
        self.assertRejects(SAMPLE.replace("fr: patrimoine |", "fr: patrimoine"), "end with")

    def test_section_ends_at_next_heading(self):
        tables = glossary.parse(SAMPLE + "\n## Weiteres\n\n| x | y |\n|---|---|\n| 1 | 2 |\n")
        self.assertEqual(len(tables["Fachbegriffe"]), 1)


if __name__ == "__main__":
    unittest.main()
