"""#182/AK1: writes the common review table of the translations (DE, EN, FR, IT side by side).

Usage:  python3 tools/i18n_review_table.py [--root DIR] [--out FILE]
Default output: local/i18n-review/tabelle.csv (semicolon, UTF-8 with BOM, so that Excel opens it directly), and the
same rows in parts for a language model in local/i18n-review/teile/teil-NN.txt (CSV as text: Gemini opens an uploaded .csv as data
for its code tool and does not read it; critical rows first, option --part-chars), and the binding translations of the glossary (docs/GLOSSAR.md, section of the translations) as
local/i18n-review/glossar.md.

Rows:
- every key of crates/app/ui/i18n/de.json with the texts of de, en, fr and it;
- every block of the help files (one folder per language): a heading, the keyword line below it, and each
  paragraph (text between blank lines). The help files have the same structure in every language, so the n-th block
  of one language is the n-th block of the others; a file with a different number of blocks is an error.

Columns: key or location (help: file and block number §n), place in the app, «kritisch» (x for the critical texts of
#182/AK3), DE, EN, FR, IT, and the empty columns «Korrektur EN», «Korrektur FR», «Korrektur IT», «Bemerkung».

Critical texts: keys starting with engine., errors., disclaimer., field., input., validation., issues.; keys used as the
label of an input field or as a hint in the editor (calls F, Chk, Series and hints in crates/app/ui/app.js); every block
of 60-referenz.md; the keyword lines of all help files.
"""

import argparse
import csv
import json
import pathlib
import re
import sys

LANGS = ("de", "en", "fr", "it")
HEADER = ["Schlüssel/Fundstelle", "Ort", "kritisch", "DE", "EN", "FR", "IT",
          "Korrektur EN", "Korrektur FR", "Korrektur IT", "Bemerkung"]
CRITICAL_PREFIXES = ("engine", "errors", "disclaimer", "field", "input", "validation", "issues")
CRITICAL_HELP_FILES = ("60-referenz.md",)
# Labels and hints of input fields in the editor: F("path", tr("key"), …), Chk(…), Series(…), hint: tr("key"),
# <p class="hint">${tr("key")…
FIELD_LABEL = re.compile(r'\b(?:F|Chk|Series)\((?:"[^"]*"|`[^`]*`)\s*,\s*tr\("([^"]+)"')
FIELD_HINT = re.compile(r'(?:hint:\s*|class="hint">\$\{[^}]*?)tr\("([^"]+)"')
HEADING = re.compile(r"^#+\s")
ANCHOR = re.compile(r"^@\S+\s+")
# «Stichworte:», «Keywords:», «Mots-clés :», «Parole chiave:» right below a heading.
KEYWORDS = re.compile(r"^[^\W\d][\w\- ]{0,20}?\s?:\s")

# Place in the app by the first part of the key (German, for the reviewers).
PLACES = {
    "about": "Dialog Über VivaPlan", "actions": "Editor: Aktionen und Meldungen", "aside": "Editor: Seitenleiste",
    "batch": "Ergebnisse: Variation", "boot": "Programmstart", "buckets": "Ergebnisse: Töpfe",
    "chart": "Grafiken", "cmp": "Ergebnisse: Vergleich", "cmpPlan": "Ergebnisse: Vergleich der Planungen",
    "common": "allgemein", "computed": "Editor: berechnete Werte", "derived": "Editor: abgeleitete Werte",
    "dialog": "Dialoge", "disclaimer": "Haftungsausschluss", "dlg": "Dialoge: Schaltflächen",
    "engine": "Meldungen des Rechenkerns", "errors": "Prüfmeldungen", "estimate": "Optimierung: Dauer",
    "fan": "Grafik: Fächer", "field": "Editor: Eingabefelder", "file": "Datei: Öffnen und Speichern",
    "flows": "Editor: Einnahmen & Ausgaben", "fmt": "Zahlenformat", "gain": "Editor: Grundstückgewinnsteuer",
    "general": "Editor: Haushalt & Projekt", "guardrails": "Editor: Leitplanken", "help": "Hilfe",
    "helpHeading": "Hilfe: Überschriften", "history": "Rückgängig/Wiederholen", "import": "Import config.json",
    "index": "Renditen: Indizes", "input": "Editor: Eingabefelder", "issues": "Editor: Prüfung",
    "job": "Berechnung: Fortschritt", "kpi": "Ergebnisse: Kennzahlen", "language": "Einstellungen: Sprache",
    "legend": "Grafiken: Legende", "limits": "Einstellungen: Grenzen", "menu": "Menüs",
    "milestone": "Feste Meilensteine", "milestones": "Editor: Meilensteine", "mini": "Editor: Minigrafik",
    "mortgage": "Editor: Hypotheken", "nav": "Navigation", "notes": "Hinweise zur Planung",
    "opt": "Optimierung", "optRanges": "Optimierung: Bandbreiten", "optResult": "Optimierung: Ergebnis",
    "optimize": "Optimierung", "optimum": "Optimierung: Ergebnis", "page": "Seiten: Kopfzeile",
    "params": "Ergebnisse: Parameter", "pension": "Editor: Pensionskasse", "pillar3a": "Editor: Säule 3a",
    "planHidden": "Planung: interne Felder", "propEvents": "Ergebnisse: Immobilien", "property": "Editor: Immobilien",
    "quit": "Dialog Beenden", "realestate": "Editor: Immobilien", "ref": "Editor: Zeitpunkt",
    "refs": "Editor: Verweise", "reserve": "Editor: Cash-Reserve", "results": "Ergebnisse",
    "returns": "Editor: Renditen", "run": "Simulation", "runCharts": "Ergebnisse: Grafiken",
    "runPicker": "Ergebnisse: Auswahl", "runTip": "Ergebnisse: Tooltip", "runs": "Ergebnisse: Liste",
    "runsDelete": "Ergebnisse: Löschen", "scen": "Ergebnisse: Szenarien", "scenarios": "Editor: Szenarien",
    "section": "Editor: Abschnitte", "series": "Editor: Zeitreihen", "settings": "Einstellungen",
    "shortcuts": "Tastenkürzel", "single": "Ergebnisse: Einzellauf", "source": "Editor: Renditequelle",
    "startup": "Programmstart", "steps": "Erste Schritte", "strat": "Strategien", "stratParams": "Strategie: Parameter",
    "strategy": "Editor: Anlagestrategie", "sweep": "Optimierung: Parameter", "taxTable": "Editor: Steuertabelle",
    "taxes": "Editor: Steuern", "timeline": "Zeitleiste", "unit": "Einheiten", "validation": "Editor: Prüfung",
    "variation": "Ergebnisse: Variation", "verdict": "Ergebnisse: Beurteilung", "wealth": "Editor: Vermögen",
    "welcome": "Willkommen", "withdrawal": "Editor: Entnahme", "yearTable": "Ergebnisse: Jahrestabelle",
    "zoom": "Grafiken: Zoom",
}


def field_keys(app_js):
    """Keys used as label or hint of an input field in the editor."""
    return set(FIELD_LABEL.findall(app_js)) | set(FIELD_HINT.findall(app_js))


def text_rows(tables, fields):
    """One row per key of the German table."""
    rows = []
    for key in tables["de"]:
        prefix = key.split(".")[0]
        critical = prefix in CRITICAL_PREFIXES or key in fields
        rows.append([key, PLACES.get(prefix, prefix), "x" if critical else ""]
                    + [tables[lang].get(key, "") for lang in LANGS])
    return rows


def help_blocks(text):
    """Blocks of a help file: each heading and each keyword line on its own, else text between blank lines.

    Returns (kind, text) with kind «heading», «keywords» or «text». The keyword line is the line «Word:» right
    after a heading in the same paragraph (same position in every language)."""
    blocks = []
    for para in re.split(r"\n\s*\n", text.strip("\n")):
        rest = []
        after_heading = False
        for line in para.split("\n"):
            if HEADING.match(line):
                if rest:
                    blocks.append(("text", "\n".join(rest)))
                    rest = []
                blocks.append(("heading", line))
                after_heading = True
            elif after_heading and KEYWORDS.match(line):
                blocks.append(("keywords", line))
                after_heading = False
            else:
                rest.append(line)
                after_heading = False
        if rest:
            blocks.append(("text", "\n".join(rest)))
    return blocks


def help_rows(help_dir):
    """One row per block of every German help file, aligned with the other languages."""
    rows, errors = [], []
    for de_file in sorted((help_dir / "de").glob("*.md")):
        name = de_file.name
        per_lang = {}
        for lang in LANGS:
            path = help_dir / lang / name
            per_lang[lang] = help_blocks(path.read_text(encoding="utf-8")) if path.exists() else []
        counts = {lang: len(b) for lang, b in per_lang.items()}
        if len(set(counts.values())) != 1:
            errors.append(f"hilfe/{name}: unterschiedlich viele Blöcke {counts}")
            continue
        kinds = {lang: [k for k, _ in b] for lang, b in per_lang.items()}
        if any(kinds[lang] != kinds["de"] for lang in LANGS):
            errors.append(f"hilfe/{name}: Überschriften und Stichworte an verschiedenen Stellen")
            continue
        title, section = "", ""
        for i, (kind, de_text) in enumerate(per_lang["de"]):
            if kind == "heading":
                heading = ANCHOR.sub("", HEADING.sub("", de_text)).strip()
                if de_text.startswith("# "):
                    title, section = heading, ""
                else:
                    section = heading
            place = " › ".join(p for p in ("Hilfe", title, section) if p)
            critical = kind == "keywords" or name in CRITICAL_HELP_FILES
            rows.append([f"hilfe/{name} §{i + 1}", place, "x" if critical else ""]
                        + [per_lang[lang][i][1] for lang in LANGS])
    return rows, errors


def build(root):
    ui = root / "crates" / "app" / "ui"
    tables = {lang: json.loads((ui / "i18n" / f"{lang}.json").read_text(encoding="utf-8")) for lang in LANGS}
    app_js = ui / "app.js"
    fields = field_keys(app_js.read_text(encoding="utf-8")) if app_js.exists() else set()
    rows = text_rows(tables, fields)
    hrows, errors = help_rows(ui / "hilfe")
    return rows + hrows, errors


def split_parts(rows, max_chars):
    """Rows for a reviewing language model in parts: critical rows first, then the others, each part with at most
    max_chars characters of text (DE to IT); a single longer row gets a part of its own."""
    ordered = [r for r in rows if r[2]] + [r for r in rows if not r[2]]
    parts, part, size = [], [], 0
    for r in ordered:
        n = sum(len(c) for c in r[3:7])
        if part and size + n > max_chars:
            parts.append(part)
            part, size = [], 0
        part.append(r)
        size += n
    if part:
        parts.append(part)
    return parts


def write_csv(path, rows):
    path.parent.mkdir(parents=True, exist_ok=True)
    with path.open("w", encoding="utf-8-sig", newline="") as f:
        w = csv.writer(f, delimiter=";", lineterminator="\r\n")
        w.writerow(HEADER)
        for r in rows:
            w.writerow(r + ["", "", "", ""])


def glossary_translations(root):
    """Section «Übersetzungen» of docs/GLOSSAR.md (binding translations, #179), or None without it."""
    path = root / "docs" / "GLOSSAR.md"
    if not path.exists():
        return None
    text = path.read_text(encoding="utf-8")
    start = text.find("\n## Übersetzungen")
    if start < 0:
        return None
    end = text.find("\n## ", start + 1)
    return text[start + 1:end if end >= 0 else None].rstrip("\n") + "\n"


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--root", default=str(pathlib.Path(__file__).resolve().parent.parent))
    ap.add_argument("--out", default=None)
    ap.add_argument("--part-chars", type=int, default=100000,
                    help="text per part in teile/ (characters of DE to IT, default 100000)")
    args = ap.parse_args(argv)
    root = pathlib.Path(args.root)
    rows, errors = build(root)
    for e in errors:
        print(e, file=sys.stderr)
    if errors:
        return 1
    out = pathlib.Path(args.out) if args.out else root / "local" / "i18n-review" / "tabelle.csv"
    write_csv(out, rows)
    parts_dir = out.parent / "teile"
    for old in [*parts_dir.glob("teil-*.csv"), *parts_dir.glob("teil-*.txt")]:
        old.unlink()
    parts = split_parts(rows, args.part_chars)
    for i, part in enumerate(parts, 1):
        write_csv(parts_dir / f"teil-{i:02d}.txt", part)
    glossary = glossary_translations(root)
    if glossary:
        (out.parent / "glossar.md").write_text(glossary, encoding="utf-8")
    critical = sum(1 for r in rows if r[2])
    print(f"{out}: {len(rows)} Zeilen, davon {critical} kritisch; {len(parts)} Teile in {parts_dir}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
