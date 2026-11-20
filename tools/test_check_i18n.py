"""Tests for tools/check_i18n.py (#180).

Run: python3 -m unittest discover -s tools -p 'test_check_i18n.py'
Each case writes a small glossary, text tables and help files into a temporary folder and runs the check on it
(option --root). The base files are free of findings; a case changes one file and expects one kind of finding.
"""

import contextlib
import io
import json
import pathlib
import sys
import tempfile
import unittest

TOOLS = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(TOOLS))
import check_i18n  # noqa: E402
import glossary  # noqa: E402

GLOSSARY = """# Glossar

## Übersetzungen

### Kürzel

| DE | EN | FR | IT | Quelle | verbotene Varianten |
|---|---|---|---|---|---|
| AHV | AHV | AVS | AVS | AHVG | fr: AHV; it: AHV |

### Fachbegriffe

| DE | EN | FR | IT | Quelle | verbotene Varianten |
|---|---|---|---|---|---|
| Umwandlungssatz | conversion rate | taux de conversion | aliquota di conversione | BVG Art. 14 | - |
| Kapitalbezugssteuer; Kapitalsteuer | lump-sum tax | impôt sur les prestations en capital | imposta sulle prestazioni in capitale | DBG | it: imposta sul capitale |
| Zeitreihe | time series | série temporelle | serie temporale | keine amtliche Quelle | - |
"""

TABLES = {
    "de": {"pk.rate": "Umwandlungssatz {v} % ab 2027", "ahv.start": "AHV-Bezug", "help": "Siehe «Hilfe»."},
    "en": {"pk.rate": "Conversion rate {v} % from 2027", "ahv.start": "AHV start", "help": "See “Help”."},
    "fr": {"pk.rate": "Taux de conversion {v} % dès 2027", "ahv.start": "Début AVS", "help": "Voir «Aide»."},
    "it": {"pk.rate": "Aliquota di conversione {v} % dal 2027", "ahv.start": "Inizio AVS", "help": "Vedi «Aiuto»."},
}

HELP = {
    "de": """# Editor

## @s-pk Pensionskasse
Der Umwandlungssatz ist 6.8 % und gilt ab 2027. Siehe [Steuern](#s-taxes).

| Feld | Wert |
|---|---|
| Satz | 6.8 % |

## @s-taxes Steuern
Die Kapitalbezugssteuer auf CHF 100'000. Die Zeitreihen liegen im Datenordner.
""",
    "en": """# Editor

## @s-pk Pension fund
The conversion rate is 6.8 % and applies from 2027. See [Taxes](#s-taxes).

| Field | Value |
|---|---|
| Rate | 6.8 % |

## @s-taxes Taxes
The lump-sum tax on CHF 100'000. The time series are in the data folder.
""",
    "fr": """# Éditeur

## @s-pk Caisse de pension
Le taux de conversion est de 6.8 % et s’applique dès 2027. Voir [Impôts](#s-taxes).

| Champ | Valeur |
|---|---|
| Taux | 6.8 % |

## @s-taxes Impôts
L’impôt sur les prestations en capital sur CHF 100'000. Les séries temporelles se trouvent dans le dossier.
""",
    "it": """# Editor

## @s-pk Cassa pensioni
L’aliquota di conversione è 6.8 % e vale dal 2027. Vedi [Imposte](#s-taxes).

| Campo | Valore |
|---|---|
| Aliquota | 6.8 % |

## @s-taxes Imposte
L’imposta sulle prestazioni in capitale su CHF 100 000. Le serie temporali si trovano nella cartella.
""",
}


def run(tables=None, help_files=None, exceptions=None, args=()):
    """Runs the check on the base files, changed by `tables` (texts by language and key) and `help_files`
    (text by language); returns (exit code, output)."""
    with tempfile.TemporaryDirectory() as d:
        root = pathlib.Path(d)
        (root / "docs").mkdir()
        (root / "docs/GLOSSAR.md").write_text(GLOSSARY, encoding="utf-8")
        i18n = root / check_i18n.I18N
        i18n.mkdir(parents=True)
        for lang, base in TABLES.items():
            table = dict(base, **(tables or {}).get(lang, {}))
            table = {k: v for k, v in table.items() if v is not None}
            (i18n / f"{lang}.json").write_text(json.dumps(table, ensure_ascii=False), encoding="utf-8")
        for lang, text in HELP.items():
            folder = root / check_i18n.HELP / lang
            folder.mkdir(parents=True)
            (folder / "10-editor.md").write_text((help_files or {}).get(lang, text), encoding="utf-8")
        if exceptions is not None:
            (root / "tools").mkdir()
            (root / check_i18n.EXCEPTIONS).write_text(json.dumps(exceptions, ensure_ascii=False), encoding="utf-8")
        out = io.StringIO()
        with contextlib.redirect_stdout(out):
            code = check_i18n.main(["--root", str(root), *args])
        return code, out.getvalue()


def findings(out):
    return [line for line in out.splitlines() if not line.startswith(("i18n-check:", "hint:"))]


class CheckI18n(unittest.TestCase):
    def test_issue_180_ak1_clean_files_pass_and_a_finding_names_file_key_and_reason(self):
        code, out = run()
        self.assertEqual((code, findings(out)), (0, []), out)
        code, out = run(tables={"fr": {"pk.rate": "Taux de conversion {v} % dès 2028"}})
        self.assertEqual(code, 1)
        self.assertEqual(len(findings(out)), 1, out)
        self.assertIn("crates/app/ui/i18n/fr.json: [values] pk.rate: numbers differ", out)

    def test_issue_180_ak1_missing_and_extra_keys_are_findings(self):
        code, out = run(tables={"en": {"help": None, "new": "New"}})
        self.assertEqual(code, 1)
        self.assertIn("en.json: [keys] help: key missing", out)
        self.assertIn("en.json: [keys] new: key not in de.json", out)

    def test_issue_180_ak2_missing_glossary_term_is_a_finding(self):
        code, out = run(tables={"fr": {"pk.rate": "Taux {v} % dès 2027"}})
        self.assertEqual(code, 1)
        self.assertIn("fr.json: [terms] pk.rate: «Umwandlungssatz» without «taux de conversion»", out)

    def test_issue_180_ak2_forbidden_variant_and_abbreviation_as_whole_word(self):
        code, out = run(tables={"it": {"ahv.start": "Inizio AHV"}})
        self.assertIn("it.json: [terms] ahv.start: «AHV» without «AVS»", out)
        self.assertIn("it.json: [terms] ahv.start: forbidden variant «AHV»", out)
        # an abbreviation inside a word is no match: «Mahvel» does not contain the abbreviation AHV
        code, out = run(tables={"de": {"x": "Mahvel"}, "en": {"x": "mahvel"}, "fr": {"x": "mahvel"},
                                "it": {"x": "mahvel"}})
        self.assertEqual((code, findings(out)), (0, []), out)

    def test_issue_180_ak2_help_is_compared_section_by_section(self):
        it = HELP["it"].replace("imposta sulle prestazioni in capitale", "imposta sul capitale")
        code, out = run(help_files={"it": it})
        self.assertEqual(code, 1)
        self.assertIn("hilfe/it/10-editor.md:10: [terms] §s-taxes: «Kapitalbezugssteuer» without", out)
        self.assertIn("§s-taxes: forbidden variant «imposta sul capitale»", out)
        # the term of section s-taxes must be in that section, not elsewhere in the file
        fr = HELP["fr"].replace("séries temporelles se trouvent", "données se trouvent")
        fr = fr.replace("Le taux de conversion", "La série temporelle, le taux de conversion")
        code, out = run(help_files={"fr": fr})
        self.assertIn("hilfe/fr/10-editor.md:10: [terms] §s-taxes: «Zeitreihe» without «série temporelle»", out)

    def test_issue_180_ak3_placeholders_numbers_chf_and_percent(self):
        code, out = run(tables={"en": {"pk.rate": "Conversion rate {n} from 2027"}})
        self.assertIn("en.json: [values] pk.rate: placeholders differ (missing {v}, extra {n})", out)
        self.assertIn("en.json: [values] pk.rate: «%» 1× in German, 0× in the target", out)
        # Swiss number formats count as equal: 100'000, 100 000, 6,8 and 6.8
        code, out = run(tables={"de": {"n": "CHF 100 000 und 6,8"}, "en": {"n": "CHF 100'000 and 6.8"},
                                "fr": {"n": "CHF 100’000 et 6.8"}, "it": {"n": "CHF 100'000 e 6,8"}})
        self.assertEqual((code, findings(out)), (0, []), out)

    def test_issue_180_ak3_help_numbers_of_a_section_must_appear_in_the_target_section(self):
        en = HELP["en"].replace("CHF 100'000", "CHF 10'000")
        code, out = run(help_files={"en": en})
        self.assertEqual(code, 1)
        self.assertIn("hilfe/en/10-editor.md:10: [values] §s-taxes: numbers missing: ['100000']", out)

    def test_issue_180_ak4_text_identical_to_german_unless_allowed(self):
        code, out = run(tables={"fr": {"ahv.start": "AHV-Bezug"}})
        self.assertIn("fr.json: [leftover] ahv.start: same text as German: «AHV-Bezug»", out)
        ok = {lang: {"ok": "OK"} for lang in TABLES}
        code, out = run(tables=ok)
        self.assertIn("en.json: [leftover] ok: same text as German: «OK»", out)
        allowed = {"identical": [{"langs": ["en", "fr", "it"], "words": ["OK"], "reason": "Fremdwort"}]}
        code, out = run(tables=ok, exceptions=allowed)
        self.assertEqual((code, findings(out)), (0, []), out)
        code, out = run(tables=ok, exceptions={"identical": [{"langs": ["fr"], "words": ["OK"], "reason": "x"}]})
        self.assertIn("en.json: [leftover] ok: same text as German: «OK»", out)
        self.assertNotIn("fr.json: [leftover]", out)
        # an allowed word that no text needs any more is reported
        code, out = run(exceptions=allowed)
        self.assertIn("[leftover] identical 1: allowed word «OK» no longer needed", out)

    def test_issue_180_ak4_help_line_copied_from_german(self):
        german = HELP["de"].splitlines()[10]
        it = HELP["it"].replace("L’imposta sulle prestazioni in capitale su CHF 100 000. Le serie temporali si "
                                "trovano nella cartella.", german)
        code, out = run(help_files={"it": it})
        self.assertIn("hilfe/it/10-editor.md:11: [leftover] §s-taxes: line identical to German", out)

    def test_issue_180_ak5_help_headings_links_and_table_rows_match(self):
        en = HELP["en"].replace("## @s-taxes", "## @s-tax")
        code, out = run(help_files={"en": en})
        self.assertIn("hilfe/en/10-editor.md:10: [structure] heading 3: headings differ", out)
        fr = HELP["fr"].replace("(#s-taxes)", "(#s-pk)")
        code, out = run(help_files={"fr": fr})
        self.assertIn("hilfe/fr/10-editor.md: [structure] link 1: links differ", out)
        it = HELP["it"].replace("| Aliquota | 6.8 % |", "| Aliquota | 6.8 % |\n| Inizio | 2027 |")
        code, out = run(help_files={"it": it})
        self.assertIn("hilfe/it/10-editor.md: [structure] tables: table rows differ: German [3], here [4]", out)

    def test_issue_180_ak6_informal_address_and_german_quotes(self):
        code, out = run(tables={"fr": {"help": "Voir «Aide», tu peux"}, "it": {"help": "Vedi «Aiuto», ti aiuta"},
                                "en": {"help": "See „Help“."}})
        self.assertIn("fr.json: [address] help: informal address «tu»", out)
        self.assertIn("it.json: [address] help: informal address «ti»", out)
        self.assertIn("en.json: [address] help: German quotation mark „", out)
        code, out = run(tables={"de": {"help": "Siehe „Hilfe“."}})
        self.assertIn("de.json: [address] help: German quotation mark „", out)
        # «tutto», «tutela» are no informal address
        code, out = run(tables={"it": {"help": "Vedi «Aiuto» per tutto e la tutela"}})
        self.assertNotIn("[address]", out)

    def test_issue_180_ak7_long_text_is_a_hint_not_a_finding(self):
        long = "Voir «Aide» – une explication bien plus longue que le texte allemand d’origine."
        code, out = run(tables={"de": {"help": "Siehe die «Hilfe» zu diesem Feld."}, "en": {"help": "See help."},
                                "fr": {"help": long}, "it": {"help": "Vedi «Aiuto» per il campo."}},
                        args=["--hints"])
        self.assertEqual(code, 0, out)
        self.assertIn("hint: crates/app/ui/i18n/fr.json: [length] help:", out)
        self.assertIn("1 length hints", out)

    def test_issue_180_ak8_exceptions_silence_a_finding_and_stale_ones_are_reported(self):
        entry = {"file": "crates/app/ui/i18n/fr.json", "where": "pk.rate", "check": "values", "subject": "number",
                 "reason": "Beispiel"}
        tables = {"fr": {"pk.rate": "Taux de conversion {v} % dès 2028"}}
        code, out = run(tables=tables, exceptions={"findings": [entry]})
        self.assertEqual((code, findings(out)), (0, []), out)
        code, out = run(exceptions={"findings": [entry]})
        self.assertEqual(code, 1)
        self.assertIn("tools/i18n_check_exceptions.json: [values] entry 1: exception no longer needed", out)
        code, out = run(exceptions={"findings": [dict(entry, reason="")]})
        self.assertEqual(code, 2)

    def test_issue_179_ak5_issue_180_ak2_the_real_glossary_is_read(self):
        text = (TOOLS.parent / "docs/GLOSSAR.md").read_text(encoding="utf-8")
        terms = check_i18n.Glossary(glossary.parse(text))
        self.assertGreater(len(terms.rows), 100)
        self.assertIn("AHV", terms.abbreviations)

    def test_issue_180_ak1_the_repository_is_clean(self):
        out = io.StringIO()
        with contextlib.redirect_stdout(out):
            code = check_i18n.main(["--root", str(TOOLS.parent)])
        self.assertEqual(code, 0, out.getvalue())


if __name__ == "__main__":
    unittest.main()
