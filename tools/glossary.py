"""Reader for the translation glossary in docs/GLOSSAR.md (#179).

The translations section holds two Markdown tables (abbreviations and terms) with six columns: German, English,
French, Italian, source, forbidden variants. The format is described in that section; the check of the
translations (#180) reads the tables with `parse`.

Run: python3 tools/glossary.py   (prints the number of rows per table)
"""

import pathlib
import sys

ROOT = pathlib.Path(__file__).resolve().parent.parent
SECTION = "## Übersetzungen"
HEADER = ["DE", "EN", "FR", "IT", "Quelle", "verbotene Varianten"]
LANGS = ["de", "en", "fr", "it"]
NO_SOURCE = "keine amtliche Quelle"


class GlossaryError(ValueError):
    pass


def split_forms(cell):
    """Splits a cell into its forms (separator "; "); a single "-" means none."""
    if cell == "-":
        return []
    return [form.strip() for form in cell.split(";") if form.strip()]


def parse_forbidden(cell):
    """Splits a forbidden-variants cell into (language, variant) tuples."""
    result = []
    for entry in split_forms(cell):
        lang, sep, form = entry.partition(":")
        lang, form = lang.strip(), form.strip()
        if not sep or lang not in LANGS[1:] or not form:
            raise GlossaryError(f"forbidden variant '{entry}' is not '<en|fr|it>: <form>'")
        result.append((lang, form))
    return result


def parse(text):
    """Returns {table name: [row]}; a row is a dict with the forms per language, the source and the forbidden
    variants. Raises GlossaryError for any deviation from the format."""
    if SECTION not in text:
        raise GlossaryError(f"section '{SECTION}' missing")
    body = text.split(SECTION, 1)[1]
    next_section = body.find("\n## ")
    if next_section >= 0:
        body = body[:next_section]
    tables = {}
    table = None
    expect = None
    for number, line in enumerate(body.splitlines(), 1):
        if line.startswith("### "):
            table = line[4:].strip()
            expect = "header"
            continue
        if not line.startswith("|"):
            if expect == "rows" and table in tables:
                expect = None
            continue
        if not line.rstrip().endswith("|"):
            raise GlossaryError(f"{table}, line {number}: row does not end with '|'")
        cells = [cell.strip() for cell in line.strip()[1:-1].split("|")]
        if expect == "header":
            if cells != HEADER:
                raise GlossaryError(f"{table}: header is {cells}, expected {HEADER}")
            expect = "separator"
            continue
        if expect == "separator":
            if any(set(cell) - {"-", ":"} or not cell for cell in cells) or len(cells) != len(HEADER):
                raise GlossaryError(f"{table}: separator line expected")
            expect = "rows"
            tables[table] = []
            continue
        if expect != "rows":
            raise GlossaryError(f"line {number}: table row outside a table")
        if len(cells) != len(HEADER):
            raise GlossaryError(f"{table}, line {number}: {len(cells)} cells instead of {len(HEADER)}")
        if any(not cell for cell in cells):
            raise GlossaryError(f"{table}, line {number}: empty cell")
        forms = {lang: split_forms(cells[i]) for i, lang in enumerate(LANGS)}
        if any(not forms[lang] for lang in LANGS):
            raise GlossaryError(f"{table}, line {number}: a language column has no form")
        tables[table].append(
            {
                "forms": forms,
                "source": [part.strip() for part in cells[4].split(";") if part.strip()],
                "official": not cells[4].startswith(NO_SOURCE),
                "forbidden": parse_forbidden(cells[5]),
            }
        )
    if not tables:
        raise GlossaryError("no table found")
    return tables


def main(argv=None):
    path = ROOT / "docs" / "GLOSSAR.md"
    try:
        tables = parse(path.read_text(encoding="utf-8"))
    except GlossaryError as err:
        print(f"{path}: {err}")
        return 1
    for name, rows in tables.items():
        official = sum(row["official"] for row in rows)
        print(f"{name}: {len(rows)} rows, {official} with official source")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
