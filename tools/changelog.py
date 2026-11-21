"""#188: release notes in four languages.

CHANGELOG.md (German) is authoritative; CHANGELOG.en.md, CHANGELOG.fr.md and CHANGELOG.it.md translate it for every
version from FIRST_TRANSLATED on (older versions stay German only).

Usage:
  python3 tools/changelog.py check              findings with file and version; exit 1 if any, 2 unreadable input
  python3 tools/changelog.py notes <version>    text of the GitHub release for one version (v prefix allowed);
                                                [-o FILE] writes it to a file; exit 1 if a language lacks the version
  python3 tools/changelog.py title <version>    title of the GitHub release, language-neutral: VivaPlan 1.0.0 Beta 2

Checks (the name in brackets appears in each finding):
- [versions]  the translation has the same version headings in the same order as the German file (AK3);
- [sections]  each translated entry (all four languages) uses only the section headings of its language, in the
              fixed order, at least one (AK2); the translation has the same sections per version as German (AK3);
- [issues]    each section of a version names the same issue numbers in every language (AK3);
- [terms]     no forbidden variant of docs/GLOSSAR.md appears in a translation (AK4).
"""

import argparse
import pathlib
import re
import sys

TOOLS = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(TOOLS))
import check_i18n  # noqa: E402
import glossary  # noqa: E402

ROOT = TOOLS.parent
FIRST_TRANSLATED = "1.0.0-beta.1"
LANGS = ["de", "en", "fr", "it"]
TARGETS = LANGS[1:]
FILES = {lang: "CHANGELOG.md" if lang == "de" else f"CHANGELOG.{lang}.md" for lang in LANGS}
# Section headings per language in their fixed order; the index is the section's identity across languages.
SECTIONS = {
    "de": ["Kann Ergebnisse ändern", "Neu, geändert, behoben", "Tests und Entwicklung"],
    "en": ["May change results", "New, changed, fixed", "Tests and development"],
    "fr": ["Peut modifier les résultats", "Nouveautés, modifications, corrections", "Tests et développement"],
    "it": ["Può modificare i risultati", "Novità, modifiche, correzioni", "Test e sviluppo"],
}
# Order of the release text: English open first, the other languages folded.
NOTES_ORDER = [("en", None), ("de", "Deutsch"), ("fr", "Français"), ("it", "Italiano")]

VERSION_RE = re.compile(r"^## \S+ (\d+\.\d+\.\d+(?:-[0-9A-Za-z.]+)?)\b")
ISSUE_RE = re.compile(r"#(\d+)")


class Finding:
    def __init__(self, file, where, check, reason):
        self.file, self.where, self.check, self.reason = file, where, check, reason

    def __str__(self):
        return f"{self.file} {self.where} [{self.check}]: {self.reason}"


def parse(text):
    """Returns [(version, heading line, body lines)] in file order; the body ends before the next version heading."""
    entries = []
    for line in text.splitlines():
        m = VERSION_RE.match(line)
        if m:
            entries.append((m.group(1), line, []))
        elif line.startswith("## "):
            raise ValueError(f"heading without version: {line!r}")
        elif entries:
            entries[-1][2].append(line)
    return entries


def translated(entries):
    """The entries from the newest down to FIRST_TRANSLATED inclusive."""
    result = []
    for entry in entries:
        result.append(entry)
        if entry[0] == FIRST_TRANSLATED:
            return result
    raise ValueError(f"version {FIRST_TRANSLATED} missing")


def sections(body):
    """Returns [(heading, issue numbers)] for the "### " sections of an entry; text before the first is ignored."""
    result = []
    for line in body:
        if line.startswith("### "):
            result.append((line[4:].strip(), set()))
        elif result:
            result[-1][1].update(int(n) for n in ISSUE_RE.findall(line))
    return result


def check_structure(file, lang, version, secs):
    """Yields findings for unknown, misordered or missing section headings (AK2)."""
    known = SECTIONS[lang]
    if not secs:
        yield Finding(file, version, "sections", "no section («### …»)")
        return
    indices = []
    for heading, _ in secs:
        if heading not in known:
            yield Finding(file, version, "sections", f"unknown section «{heading}» (allowed: {', '.join(known)})")
        else:
            indices.append(known.index(heading))
    if indices != sorted(set(indices)):
        yield Finding(file, version, "sections", f"sections not in the order {', '.join(known)} or repeated")


def forbidden_patterns(root):
    """Forbidden glossary variants as (language, form, compiled pattern); abbreviations match whole words only."""
    tables = glossary.parse((root / check_i18n.GLOSSARY).read_text(encoding="utf-8"))
    result = []
    for table, rows in tables.items():
        for row in rows:
            for lang, form in row["forbidden"]:
                result.append((lang, form, check_i18n.word_pattern(form, table == "Kürzel")))
    return result


def load(root):
    """Returns the parsed entries per language; for de only the versions down to FIRST_TRANSLATED."""
    data = {}
    for lang in LANGS:
        path = root / FILES[lang]
        if not path.exists():
            raise ValueError(f"{FILES[lang]} missing")
        entries = parse(path.read_text(encoding="utf-8"))
        data[lang] = translated(entries) if lang == "de" else entries
    return data


def run_checks(root):
    data = load(root)
    findings = []
    de = data["de"]
    for version, _, body in de:
        findings += check_structure(FILES["de"], "de", version, sections(body))
    forbidden = forbidden_patterns(root)
    for lang in TARGETS:
        file = FILES[lang]
        entries = data[lang]
        de_versions = [v for v, _, _ in de]
        versions = [v for v, _, _ in entries]
        if versions != de_versions:
            missing = [v for v in de_versions if v not in versions]
            extra = [v for v in versions if v not in de_versions]
            reason = f"versions {versions}, German {de_versions}"
            if missing:
                reason += f"; missing {missing}"
            if extra:
                reason += f"; not in German {extra}"
            findings.append(Finding(file, "file", "versions", reason))
        by_version = {v: body for v, _, body in entries}
        for version, _, de_body in de:
            if version not in by_version:
                continue
            secs = sections(by_version[version])
            findings += check_structure(file, lang, version, secs)
            de_secs = {SECTIONS["de"].index(h): issues for h, issues in sections(de_body) if h in SECTIONS["de"]}
            tr_secs = {SECTIONS[lang].index(h): issues for h, issues in secs if h in SECTIONS[lang]}
            if sorted(de_secs) != sorted(tr_secs):
                findings.append(Finding(file, version, "sections",
                                        f"sections {[SECTIONS[lang][i] for i in sorted(tr_secs)]}, German "
                                        f"{[SECTIONS['de'][i] for i in sorted(de_secs)]}"))
            for i in sorted(set(de_secs) & set(tr_secs)):
                if de_secs[i] != tr_secs[i]:
                    missing = sorted(de_secs[i] - tr_secs[i])
                    extra = sorted(tr_secs[i] - de_secs[i])
                    parts = []
                    if missing:
                        parts.append("missing " + ", ".join(f"#{n}" for n in missing))
                    if extra:
                        parts.append("not in German " + ", ".join(f"#{n}" for n in extra))
                    findings.append(Finding(file, version, "issues", f"«{SECTIONS[lang][i]}»: {'; '.join(parts)}"))
        for version, heading, body in entries:
            text = check_i18n.norm("\n".join([heading] + body))
            for flang, form, pattern in forbidden:
                if flang == lang and pattern.search(text):
                    findings.append(Finding(file, version, "terms", f"forbidden variant «{form}»"))
    return findings


TITLE_RE = re.compile(r"^v?(\d+\.\d+\.\d+)(?:-([A-Za-z]+)\.?(\d+)?)?$")


def title(version):
    """Release title from the version: product name and version only, so it reads the same in every language;
    a pre-release label is capitalised and its number separated (1.0.0-beta.2 -> VivaPlan 1.0.0 Beta 2)."""
    m = TITLE_RE.match(version)
    if not m:
        raise ValueError(f"version {version!r} is not X.Y.Z or X.Y.Z-label.N")
    parts = ["VivaPlan", m.group(1)]
    if m.group(2):
        parts.append(m.group(2).capitalize())
    if m.group(3):
        parts.append(m.group(3))
    return " ".join(parts)


def notes(root, version):
    """The release text of one version: English open, German, French and Italian in <details> blocks."""
    version = version[1:] if version.startswith("v") else version
    parts = []
    for lang, label in NOTES_ORDER:
        path = root / FILES[lang]
        entries = {v: body for v, _, body in parse(path.read_text(encoding="utf-8"))}
        if version not in entries:
            raise ValueError(f"{FILES[lang]}: version {version} missing")
        body = "\n".join(entries[version]).strip()
        if label is None:
            parts.append(body)
        else:
            parts.append(f"<details>\n<summary>{label}</summary>\n\n{body}\n\n</details>")
    return "\n\n".join(parts) + "\n"


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("--root", default=str(ROOT))
    sub = ap.add_subparsers(dest="command", required=True)
    sub.add_parser("check")
    n = sub.add_parser("notes")
    n.add_argument("version")
    n.add_argument("-o", "--output")
    sub.add_parser("title").add_argument("version")
    args = ap.parse_args(argv)
    root = pathlib.Path(args.root)
    if args.command == "title":
        try:
            print(title(args.version))
        except ValueError as err:
            print(f"changelog: {err}", file=sys.stderr)
            return 1
        return 0
    if args.command == "notes":
        try:
            text = notes(root, args.version)
        except (OSError, ValueError) as err:
            print(f"changelog: {err}", file=sys.stderr)
            return 1
        if args.output:
            pathlib.Path(args.output).write_text(text, encoding="utf-8")
        else:
            sys.stdout.write(text)
        return 0
    try:
        findings = run_checks(root)
    except (OSError, ValueError, glossary.GlossaryError) as err:
        print(f"changelog: {err}")
        return 2
    for finding in findings:
        print(finding)
    print(f"changelog: {len(findings)} findings")
    return 1 if findings else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
