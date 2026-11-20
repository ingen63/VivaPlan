"""#180: checks the translations of the user interface and of the help against the German texts.

Usage:  python3 tools/check_i18n.py [--root DIR] [--hints]
Exit code: 0 no finding, 1 findings (each with file, key or line and the reason), 2 unreadable input.

Checked, for each target language (en, fr, it):
- the text table of each language (crates/app/ui/i18n/) against de.json, key by key;
- the help files of each language against the German ones; terms and values section by section (between two
  headings).

Checks (the name in brackets appears in each finding and in the exceptions file):
- [keys]       the target table has the same keys as de.json (AK1);
- [terms]      a glossary term of the German text appears in the target text in one of its fixed forms, and no
               forbidden variant appears (AK2; glossary in docs/GLOSSAR.md, read with tools/glossary.py);
- [values]     placeholders ({n}), numbers, CHF, % (and so years) match the German text (AK3); in the help, every
               number of the German section appears in the target section;
- [leftover]   a target text identical to the German one (AK4); in the help, a line of letters copied verbatim;
- [structure]  per help file: headings (level and anchor), links and table rows in the same number and order (AK5);
- [address]    no informal address in French and Italian (tu, ti, ...) and no German quotes „…“ (AK6);
- [length]     a text more than 40 % longer than the German one is a hint, not a finding (AK7; --hints lists them).

Glossary comparison (docs/GLOSSAR.md, «Übersetzungen»): terms case-insensitive as substring, abbreviations
case-sensitive as whole word, ' equals ’, the longest match wins. Forbidden variants follow the same rule as their
table (abbreviations as whole word, otherwise as substring).

Findings that are not clearly wrong are listed with a reason in tools/i18n_check_exceptions.json; an exception
that no longer matches any finding is itself a finding, so the list stays short.
"""

import argparse
import json
import pathlib
import re
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
import glossary  # noqa: E402

ROOT = pathlib.Path(__file__).resolve().parent.parent
TARGETS = ["en", "fr", "it"]
I18N = "crates/app/ui/i18n"
HELP = "crates/app/ui/hilfe"
GLOSSARY = "docs/GLOSSAR.md"
EXCEPTIONS = "tools/i18n_check_exceptions.json"
CHECKS = {"keys", "terms", "values", "leftover", "structure", "address", "length"}
LENGTH_FACTOR = 1.4
LENGTH_MIN = 20  # German texts shorter than this are not compared by length (one word against one word)

PLACEHOLDER = re.compile(r"\{[A-Za-z_][A-Za-z0-9_]*(?::[A-Za-z0-9_]+)?\}")
CODE = re.compile(r"`[^`\n]*`")
TAG = re.compile(r"<[^<>]*>")
LINK = re.compile(r"\]\(([^()\s]*)\)")
HEADING = re.compile(r"^(#{1,6})\s+(?:@(\S+)\s*)?(.*)$")
# thousands separated by apostrophe or (narrow, no-break) space; a decimal comma counts as a decimal point
SEPARATOR = "'’ \u00a0\u202f"
NUMBER = re.compile(r"(?<![\w.,])(?:\d{1,3}(?:[" + SEPARATOR + r"]\d{3})+(?!\d)|\d+)(?:[.,]\d+)*")
CHF = re.compile(r"\bCHF\b")
LETTERS = re.compile(r"[^\W\d_]{2,}")
DOTTED = re.compile(r"\w+(?:\.\w+)+")
# informal address (AK6): pronouns and possessives of the second person singular
INFORMAL = {
    "fr": re.compile(r"\b(?:tu|toi|ton|ta|tes|te)\b|\bt'(?=\w)", re.IGNORECASE),
    "it": re.compile(r"\b(?:tu|ti|tuo|tua|tuoi|tue)\b", re.IGNORECASE),
}
GERMAN_QUOTE = "„"


class Finding:
    def __init__(self, file, where, check, subject, reason, line=None):
        self.file, self.where, self.check, self.subject = file, where, check, subject
        self.reason, self.line = reason, line
        self.words = set()  # leftover: the words that an «identical» exception must allow

    @property
    def lang(self):
        return self.file.split("/")[-2] if self.file.endswith(".md") else pathlib.Path(self.file).stem

    def __str__(self):
        place = f"{self.file}:{self.line}" if self.line else self.file
        return f"{place}: [{self.check}] {self.where}: {self.reason}"


# --- text helpers ---


def norm(text):
    return text.replace("’", "'")


def visible(text):
    """Text without placeholders, code spans, HTML tags and Markdown link targets."""
    text = PLACEHOLDER.sub(" ", text)
    text = CODE.sub(" ", text)
    text = TAG.sub(" ", text)
    return LINK.sub("] ", text)


def numbers(text):
    return [re.sub("[" + SEPARATOR + "]", "", n).replace(",", ".") for n in NUMBER.findall(visible(text))]


def word_pattern(form, abbreviation, inflected=False):
    """Abbreviations as whole word with case; terms as substring without case. With `inflected` (terms of the
    target languages) each word of four letters or more may end differently in up to three letters (plural,
    agreement: «stratégies», «serie temporali» for «stratégie», «serie temporale»)."""
    if abbreviation:
        return re.compile(r"(?<!\w)" + re.escape(norm(form)) + r"(?!\w)")
    if not inflected:
        return re.compile(re.escape(norm(form)), re.IGNORECASE)
    parts = re.split(r"(\w+)", norm(form))
    pattern = "".join(
        re.escape(part[:-1]) + r"\w{0,3}" if i % 2 and len(part) >= 4 else re.escape(part)
        for i, part in enumerate(parts)
    )
    return re.compile(pattern, re.IGNORECASE)


class Glossary:
    """Glossary rows with compiled patterns: (name, forms per language, patterns per language, forbidden variants)."""

    def __init__(self, tables):
        self.rows = []
        self.abbreviations = set()
        for table, rows in tables.items():
            abbreviation = table == "Kürzel"
            if abbreviation:
                self.abbreviations.update(form for row in rows for lang in glossary.LANGS for form in row["forms"][lang])
            for row in rows:
                forms = {
                    lang: [word_pattern(f, abbreviation, inflected=lang != "de") for f in row["forms"][lang]]
                    for lang in glossary.LANGS
                }
                forbidden = [(lang, form, word_pattern(form, abbreviation)) for lang, form in row["forbidden"]]
                self.rows.append((row["forms"]["de"][0], row["forms"], forms, forbidden))

    def terms_in(self, de_text):
        """Indices of the rows whose German form appears; overlapping matches keep the longest."""
        text = norm(visible(de_text))
        matches = []
        for i, (_, _, forms, _) in enumerate(self.rows):
            for pattern in forms["de"]:
                matches += [(m.start(), m.end(), i) for m in pattern.finditer(text)]
        matches.sort(key=lambda m: (m[0] - m[1], m[0]))
        taken, found = [], set()
        for start, end, i in matches:
            if any(start < e and s < end for s, e in taken):
                continue
            taken.append((start, end))
            found.add(i)
        return found

    def check(self, de_text, target, lang):
        """Yields (subject, reason) for missing terms and forbidden variants."""
        text = norm(visible(target))
        for i in sorted(self.terms_in(de_text)):
            name, raw, forms, _ = self.rows[i]
            if not any(p.search(text) for p in forms[lang]):
                wanted = " / ".join(f"«{f}»" for f in raw[lang])
                yield name, f"«{name}» without {wanted}"
        for name, raw, _, forbidden in self.rows:
            for flang, form, pattern in forbidden:
                if flang == lang and pattern.search(text):
                    yield name, f"forbidden variant «{form}» (for «{name}»: {raw[lang][0]})"


# --- checks shared by the tables and the help ---


def check_values(de_text, target, exact):
    """Yields (subject, reason) for differing placeholders, numbers, CHF and %."""
    de_ph, tr_ph = sorted(PLACEHOLDER.findall(de_text)), sorted(PLACEHOLDER.findall(target))
    if de_ph != tr_ph:
        missing = sorted(set(de_ph) - set(tr_ph))
        extra = sorted(set(tr_ph) - set(de_ph))
        detail = ", ".join([f"missing {p}" for p in missing] + [f"extra {p}" for p in extra]) or "count differs"
        yield "placeholder", f"placeholders differ ({detail})"
    de_num, tr_num = numbers(de_text), numbers(target)
    if exact:
        if sorted(de_num) != sorted(tr_num):
            yield "number", f"numbers differ: de {sorted(de_num)}, target {sorted(tr_num)}"
        for name, pattern in (("CHF", CHF), ("%", re.compile("%"))):
            a, b = len(pattern.findall(visible(de_text))), len(pattern.findall(visible(target)))
            if a != b:
                yield name, f"«{name}» {a}× in German, {b}× in the target"
    else:
        missing = sorted(set(de_num) - set(tr_num))
        if missing:
            yield "number", f"numbers missing: {missing}"
        for name, pattern in (("CHF", CHF), ("%", re.compile("%"))):
            if pattern.search(visible(de_text)) and not pattern.search(visible(target)):
                yield name, f"«{name}» missing"


def check_address(target, lang):
    text = visible(target)
    if GERMAN_QUOTE in text:
        yield "„", "German quotation mark „"
    pattern = INFORMAL.get(lang)
    if pattern:
        for m in pattern.finditer(text):
            yield m.group(0).lower(), f"informal address «{m.group(0)}»"


def words(text, abbreviations):
    """Words of two letters or more, without abbreviations of the glossary and names with dots (config.json)."""
    text = DOTTED.sub(" ", visible(text))
    return {w for w in LETTERS.findall(text) if w not in abbreviations}


def too_long(de_text, target):
    a, b = len(visible(de_text).strip()), len(visible(target).strip())
    return a >= LENGTH_MIN and b > LENGTH_FACTOR * a


# --- text tables ---


def check_table(rel, de, tr, lang, terms):
    findings, hints = [], []
    for key in sorted(set(de) - set(tr)):
        findings.append(Finding(rel, key, "keys", "missing", "key missing"))
    for key in sorted(set(tr) - set(de)):
        findings.append(Finding(rel, key, "keys", "extra", "key not in de.json"))
    for key in sorted(set(de) & set(tr)):
        a, b = de[key], tr[key]
        if not isinstance(a, str) or not isinstance(b, str):
            continue
        for subject, reason in terms.check(a, b, lang):
            findings.append(Finding(rel, key, "terms", subject, reason))
        for subject, reason in check_values(a, b, exact=True):
            findings.append(Finding(rel, key, "values", subject, reason))
        if a == b and words(a, terms.abbreviations):
            finding = Finding(rel, key, "leftover", "text", f"same text as German: «{a[:60]}»")
            finding.words = words(a, terms.abbreviations)
            findings.append(finding)
        for subject, reason in check_address(b, lang):
            findings.append(Finding(rel, key, "address", subject, reason))
        if too_long(a, b):
            hints.append(Finding(rel, key, "length", "text", f"{len(b)} characters against {len(a)} in German"))
    return findings, hints


# --- help ---


class Section:
    def __init__(self, line, level, anchor, title):
        self.line, self.level, self.anchor, self.title = line, level, anchor, title
        self.lines = []  # (line number, text)

    @property
    def text(self):
        return "\n".join(t for _, t in self.lines)

    def where(self, index):
        return f"§{self.anchor}" if self.anchor else f"§{index}"


def sections(text):
    """Splits a help file at its headings (outside code blocks); section 0 is the text before the first one."""
    result = [Section(1, 0, None, "")]
    fenced = False
    for number, line in enumerate(text.splitlines(), 1):
        if line.startswith("```"):
            fenced = not fenced
        m = None if fenced else HEADING.match(line)
        if m:
            result.append(Section(number, len(m.group(1)), m.group(2), m.group(3)))
            result[-1].lines.append((number, m.group(3)))
        else:
            result[-1].lines.append((number, line))
    return result


def tables(secs):
    """Row counts of the tables in order (a table is a run of lines starting with '|')."""
    counts, run = [], 0
    for sec in secs:
        for _, line in sec.lines + [(0, "")]:
            if line.startswith("|"):
                run += 1
            elif run:
                counts.append(run)
                run = 0
    return counts


def first_difference(a, b):
    for i, (x, y) in enumerate(zip(a, b)):
        if x != y:
            return i, x, y
    return min(len(a), len(b)), (a[len(b)] if len(a) > len(b) else None), (b[len(a)] if len(b) > len(a) else None)


def check_help_file(rel, de_text, tr_text, lang, terms):
    findings, hints = [], []
    de_secs, tr_secs = sections(de_text), sections(tr_text)
    # AK5: structure
    de_heads = [(s.level, s.anchor) for s in de_secs[1:]]
    tr_heads = [(s.level, s.anchor) for s in tr_secs[1:]]
    if de_heads != tr_heads:
        i, x, y = first_difference(de_heads, tr_heads)
        line = tr_secs[i + 1].line if i + 1 < len(tr_secs) else None
        findings.append(Finding(rel, f"heading {i + 1}", "structure", "headings",
                                f"headings differ ({len(de_heads)} in German, {len(tr_heads)} here; "
                                f"first difference: German {x}, here {y})", line))
    de_links, tr_links = LINK.findall(de_text), LINK.findall(tr_text)
    if de_links != tr_links:
        i, x, y = first_difference(de_links, tr_links)
        findings.append(Finding(rel, f"link {i + 1}", "structure", "links",
                                f"links differ ({len(de_links)} in German, {len(tr_links)} here; "
                                f"first difference: German {x}, here {y})"))
    de_tables, tr_tables = tables(de_secs), tables(tr_secs)
    if de_tables != tr_tables:
        findings.append(Finding(rel, "tables", "structure", "tables",
                                f"table rows differ: German {de_tables}, here {tr_tables}"))
    # AK2, AK3, AK7 section by section (only when the headings match, otherwise the sections are not comparable)
    if de_heads == tr_heads:
        for i, (a, b) in enumerate(zip(de_secs, tr_secs)):
            where = a.where(i)
            for subject, reason in terms.check(a.text, b.text, lang):
                findings.append(Finding(rel, where, "terms", subject, reason, b.line))
            for subject, reason in check_values(a.text, b.text, exact=False):
                findings.append(Finding(rel, where, "values", subject, reason, b.line))
            if too_long(a.text, b.text):
                hints.append(Finding(rel, where, "length", "text",
                                     f"{len(b.text)} characters against {len(a.text)} in German", b.line))
    # AK4: lines copied verbatim from the German file; AK6: address and quotes
    de_lines = {line.strip() for line in de_text.splitlines()}
    for sec_index, sec in enumerate(tr_secs):
        for number, line in sec.lines:
            stripped = line.strip()
            if stripped in de_lines and words(stripped, terms.abbreviations) and not stripped.startswith("|"):
                finding = Finding(rel, sec.where(sec_index), "leftover", stripped[:60],
                                  f"line identical to German: «{stripped[:60]}»", number)
                finding.words = words(stripped, terms.abbreviations)
                findings.append(finding)
            for subject, reason in check_address(line, lang):
                findings.append(Finding(rel, sec.where(sec_index), "address", subject, reason, number))
    return findings, hints


# --- exceptions and main ---


def load_exceptions(path):
    """Returns (identical, findings): groups of words a text may share with German (AK4) and single findings.
    File format: {"identical": [{"langs": [..], "words": [..], "reason": ".."}],
                  "findings": [{"file": .., "where": .., "check": .., "subject": .. (optional), "reason": ..}]}"""
    if not path.exists():
        return [], []
    data = json.loads(path.read_text(encoding="utf-8"))
    identical, entries = data.get("identical", []), data.get("findings", [])
    for group in identical:
        if set(group) != {"langs", "words", "reason"} or not group["reason"].strip() or \
                set(group["langs"]) - set(TARGETS):
            raise ValueError(f"{path.name}: invalid «identical» group {group} (needs langs, words, reason)")
    for entry in entries:
        missing = {"file", "where", "check", "reason"} - set(entry)
        if missing or entry["check"] not in CHECKS or not entry["reason"].strip():
            raise ValueError(f"{path.name}: invalid entry {entry} (needs file, where, check, reason)")
    return identical, entries


def matches(entry, finding):
    return (entry["file"] == finding.file and entry["where"] == finding.where and entry["check"] == finding.check
            and entry.get("subject", finding.subject) == finding.subject)


def run_checks(root):
    terms = Glossary(glossary.parse((root / GLOSSARY).read_text(encoding="utf-8")))
    findings, hints = [], []
    de = json.loads((root / I18N / "de.json").read_text(encoding="utf-8"))
    for lang in TARGETS:
        rel = f"{I18N}/{lang}.json"
        tr = json.loads((root / rel).read_text(encoding="utf-8"))
        f, h = check_table(rel, de, tr, lang, terms)
        findings += f
        hints += h
    for key, text in de.items():
        if isinstance(text, str) and GERMAN_QUOTE in visible(text):
            findings.append(Finding(f"{I18N}/de.json", key, "address", "„", "German quotation mark „"))
    de_help = root / HELP / "de"
    for de_file in sorted(de_help.glob("*.md")):
        if GERMAN_QUOTE in de_file.read_text(encoding="utf-8"):
            findings.append(Finding(f"{HELP}/de/{de_file.name}", "file", "address", "„", "German quotation mark „"))
        for lang in TARGETS:
            rel = f"{HELP}/{lang}/{de_file.name}"
            target = root / rel
            if not target.exists():
                findings.append(Finding(rel, "file", "structure", "file", "help file missing"))
                continue
            f, h = check_help_file(rel, de_file.read_text(encoding="utf-8"), target.read_text(encoding="utf-8"),
                                   lang, terms)
            findings += f
            hints += h
    return findings, hints


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--root", default=str(ROOT))
    ap.add_argument("--hints", action="store_true", help="list the length hints (AK7) one by one")
    args = ap.parse_args(argv)
    root = pathlib.Path(args.root)
    try:
        findings, hints = run_checks(root)
        identical, exceptions = load_exceptions(root / EXCEPTIONS)
    except (OSError, ValueError, glossary.GlossaryError) as err:
        print(f"i18n-check: {err}")
        return 2
    used, used_words = set(), set()
    open_findings = []
    for finding in findings:
        if finding.check == "leftover":
            allowed = {(w.lower(), g) for g, group in enumerate(identical) if finding.lang in group["langs"]
                       for w in group["words"]}
            if {w.lower() for w in finding.words} <= {w for w, _ in allowed}:
                used_words.update((w, g) for w, g in allowed if w in {x.lower() for x in finding.words})
                continue
        hit = [i for i, e in enumerate(exceptions) if matches(e, finding)]
        used.update(hit)
        if not hit:
            open_findings.append(finding)
    for i, entry in enumerate(exceptions):
        if i not in used:
            open_findings.append(Finding(EXCEPTIONS, f"entry {i + 1}", entry["check"], "stale",
                                         f"exception no longer needed ({entry['file']} {entry['where']})"))
    for g, group in enumerate(identical):
        for word in group["words"]:
            if (word.lower(), g) not in used_words:
                open_findings.append(Finding(EXCEPTIONS, f"identical {g + 1}", "leftover", "stale",
                                             f"allowed word «{word}» no longer needed"))
    for finding in open_findings:
        print(finding)
    if args.hints:
        for hint in hints:
            print(f"hint: {hint}")
    print(f"i18n-check: {len(open_findings)} findings, {len(used)} exceptions and {len(used_words)} allowed words used, "
          f"{len(hints)} length hints (more than {round((LENGTH_FACTOR - 1) * 100)} % longer"
          f"{'' if args.hints else '; list with --hints'})")
    return 1 if open_findings else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
