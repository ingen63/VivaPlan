"""Reports German comments and German test names (#165).

Called through tools/check-english.sh; see there for usage and exit codes.

What is checked (Rust, JavaScript/TypeScript, Python, shell):
- comments: line comments, block comments, Rust doc comments, Python docstrings;
- test names: Rust functions marked with a test attribute, Python `def test_*`, and the name strings of
  JavaScript `test(…)`, `it(…)` and `describe(…)` calls (including `test.describe(…)`).
Not checked: string literals (user interface texts), identifiers other than test names, golden files.

A text counts as German if it contains an umlaut or sharp s, or a word from GERMAN_WORDS. Inside comments, code
spans in backticks, quotations in «…» (user interface labels), URLs and words in capitals (AHV, PK, MIT) are ignored.
A line containing the marker `check-english: allow` is never reported.
"""

import argparse
import os
import re
import subprocess
import sys

# Paths not yet converted to English (transition). Every package conversion removes its path (#166, #167, #168; #169 done).
# With an empty list, the check runs in `make test-quiet` and in the workflow (option --gate).
PENDING = [
    "crates/engine/",
    "crates/app/",
    "e2e/",
]

# Directories checked by default, relative to the repository root.
SCOPE = ["crates", "e2e", "fuzz", "tools"]

# Never checked: golden files and generated or third-party content.
EXCLUDED = ["crates/engine/tests/golden/"]
EXCLUDED_DIRS = {"node_modules", "target", "test-results", "playwright-report", "corpus", "artifacts", ".git"}

ALLOW_MARKER = "check-english: allow"

# German words that do not occur as ordinary English words in code comments. Words with umlauts are hit anyway;
# they are listed for test names, which spell umlauts as ae, oe, ue (variants are added below).
GERMAN_WORDS = set(
    """
    der die das dem den ein eine einen einem einer eines und oder aber nicht kein keine keinen keiner
    ist sind wird werden wurde wurden sein kann können muss müssen soll sollen darf gibt gilt bleibt fehlt
    steht liegt zeigt prüft setzt liefert rechnet berechnet ergibt enthält meldet bleiben laufen läuft
    für mit auf aus bei nach vom zum zur beim im um zu vor über unter bis ohne gegen durch zwischen seit
    wenn dann sonst weil dass damit sowie bzw usw auch noch nur schon immer wieder bereits mehr weniger
    hier dort oben unten wie wo wer sich sie wir ich jede jeder jedes jeden alle allen diese dieser dieses diesem
    diesen zwei drei vier erste ersten zweite zweiten letzte letzten neue neuen neuer gleich gleiche ganze ganzen
    höchstens mindestens genau etwa rund leer leere richtig falsch wahr denn doch je
    monat monate monaten monats jahr jahre jahren jahres tage woche
    entnahme entnahmen vermögen betrag beträge steuer steuern rente renten ausgaben einnahmen einkommen
    hypothek zins zinsen miete planung planungen lauf läufe wert werte zeitreihe zeitreihen prüfung fehler
    hinweis hinweise datei dateien feld felder zeile zeilen spalte spalten liste eintrag ergebnis ergebnisse
    beispiel anteil anteile guthaben bezug säule pensionskasse kapital teuerung rendite renditen aktien
    obligationen immobilie immobilien verkauf kauf tragbarkeit steuerfuss oberfläche ansicht knopf grafik
    grafiken szenario szenarien variante varianten abschnitt hilfe zeitpunkt beginn ende summe faktor
    konto kanton gemeinde vorsorge lebenshaltung wohnkosten erwerb pensionierung meilenstein meilensteine
    berechnung prüfen laden speichern öffnen zeigen setzen lesen schreiben rechnen ändern
    entfernen erzeugen melden geplant geplante fixe fixen von heisst gelten alles zuerst ziel anlegen aufruf
    aufbau monatlich jährlich netto brutto bedarf topf daten des vorher danach beide beiden weiter eigene eigener
    suche wechsel ereignis ereignisse verlauf kriterium kriterien protokoll vorgabe vorgaben januar februar märz
    mai juni juli oktober dezember dauer sekunden minuten stunden kurz lang leerer erwartete durchschnitt zahl
    """.split()
)
# Endings of German compounds and derived words that English words of this length do not have.
GERMAN_ENDINGS = (
    "ung", "ungen", "keit", "keiten", "heit", "heiten", "schaft", "lich", "liche", "lichen", "isch", "ische",
    "zahl", "zahlen", "funktion", "funktionen", "daten", "werte", "jahr", "monat", "betrag", "steuer", "topf", "buch",
    "iert", "ierte", "ierten", "ieren", "weg", "zeichen", "felder", "varianz", "rendite", "renditen", "vorgabe",
    "vorgaben", "bezug", "verlauf", "strategien", "quote", "objekten", "strom",
)
_UMLAUT_SPELLING = str.maketrans({"ä": "ae", "ö": "oe", "ü": "ue", "ß": "ss"})
GERMAN_WORDS |= {w.translate(_UMLAUT_SPELLING) for w in GERMAN_WORDS}

UMLAUT = re.compile(r"[äöüÄÖÜß]")
WORD = re.compile(r"[A-Za-zÄÖÜäöüß]+")
IGNORED_IN_COMMENTS = [
    re.compile(r"`[^`]*`"),  # code spans
    re.compile(r"«[^»]*»"),  # user interface labels
    re.compile(r"\b[a-z][a-z0-9+.-]*://\S+"),  # URLs
]

LANGUAGES = {".rs": "rust", ".js": "js", ".mjs": "js", ".cjs": "js", ".ts": "js", ".py": "python", ".sh": "shell"}


def language_of(path):
    lang = LANGUAGES.get(os.path.splitext(path)[1])
    if lang or "." in os.path.basename(path):
        return lang
    try:
        with open(path, encoding="utf-8") as f:
            first = f.readline()
    except (OSError, UnicodeDecodeError):
        return None
    if first.startswith("#!"):
        if "python" in first:
            return "python"
        if re.search(r"\b(ba|z)?sh\b", first):
            return "shell"
    return None


def is_german(text, comment=True):
    """True if the text looks German. Comments ignore code spans, «…» labels, URLs and capitalised words."""
    if comment:
        for pattern in IGNORED_IN_COMMENTS:
            text = pattern.sub(" ", text)
    if UMLAUT.search(text):
        return True
    for word in WORD.findall(text):
        if comment and len(word) > 1 and word.isupper():
            continue
        if german_word(word.lower()):
            return True
    return False


def german_word(word):
    return word in GERMAN_WORDS or (len(word) >= 7 and word.endswith(GERMAN_ENDINGS))


def identifier_is_german(name):
    """True if a snake_case or camelCase identifier contains a German word."""
    if UMLAUT.search(name):
        return True
    spaced = re.sub(r"([a-z0-9])([A-Z])", r"\1 \2", name).replace("_", " ")
    return any(german_word(w.lower()) for w in spaced.split())


# --- Scanner: splits source text into comments (with line numbers) while skipping string literals. ---


class Scan:
    def __init__(self, text):
        self.text = text
        self.i = 0
        self.line = 1
        self.comments = []  # (line, text)
        self.strings = []  # (line, text, start offset)

    def peek(self, k=0):
        j = self.i + k
        return self.text[j] if j < len(self.text) else ""

    def startswith(self, s):
        return self.text.startswith(s, self.i)

    def advance(self, n=1):
        for _ in range(n):
            if self.i < len(self.text):
                if self.text[self.i] == "\n":
                    self.line += 1
                self.i += 1

    def take_until(self, end):
        """Consumes text up to and including `end`; returns the text before it."""
        j = self.text.find(end, self.i)
        j = len(self.text) if j < 0 else j
        part = self.text[self.i : j]
        self.advance(len(part) + len(end))
        return part

    def add_comment(self, line, text):
        for k, part in enumerate(text.split("\n")):
            self.comments.append((line + k, part))

    def quoted(self, quote, escapes=True):
        """Consumes a string literal starting at the opening quote; returns (line, content)."""
        line = self.line
        start = self.i + len(quote)
        self.advance(len(quote))
        while self.i < len(self.text):
            if escapes and self.peek() == "\\":
                self.advance(2)
            elif self.startswith(quote):
                content = self.text[start : self.i]
                self.advance(len(quote))
                return line, content
            else:
                self.advance()
        return line, self.text[start:]


def scan_rust(text):
    s = Scan(text)
    while s.i < len(text):
        c = s.peek()
        if s.startswith("//"):
            line = s.line
            s.add_comment(line, s.take_until("\n"))
        elif s.startswith("/*"):
            line = s.line
            s.advance(2)
            depth, start = 1, s.i
            while s.i < len(text) and depth:
                if s.startswith("/*"):
                    depth += 1
                    s.advance(2)
                elif s.startswith("*/"):
                    depth -= 1
                    s.advance(2)
                else:
                    s.advance()
            s.add_comment(line, text[start : s.i - 2])
        elif re.match(r'b?r(#*)"', text[s.i : s.i + 40]) and not _ident_char(text, s.i - 1):
            m = re.match(r'b?r(#*)"', text[s.i : s.i + 40])
            start, line = s.i, s.line
            s.advance(len(m.group(0)))
            s.strings.append((line, s.take_until('"' + m.group(1)), start))
        elif c == '"':
            start = s.i
            line, content = s.quoted('"')
            s.strings.append((line, content, start))
        elif c == "'":
            m = re.match(r"'(\\u\{[0-9a-fA-F]+\}|\\.|[^\\'\n])'", text[s.i : s.i + 12])
            s.advance(len(m.group(0)) if m else 1)  # char literal, or a lifetime
        else:
            s.advance()
    return s


def _ident_char(text, j):
    return j >= 0 and (text[j].isalnum() or text[j] == "_")


def scan_js(text):
    s = Scan(text)
    last = ""  # last significant code character, to tell regex literals from division
    while s.i < len(text):
        c = s.peek()
        if s.startswith("//"):
            line = s.line
            s.add_comment(line, s.take_until("\n"))
        elif s.startswith("/*"):
            line = s.line
            s.advance(2)
            s.add_comment(line, s.take_until("*/"))
        elif c in "\"'`":
            start = s.i
            line, content = s.quoted(c)
            s.strings.append((line, content, start))
            last = c
        elif c == "/" and (last == "" or last in "(,=:[!&|?{};+-*%<>~^"):
            s.advance()
            in_class = False
            while s.i < len(text) and s.peek() != "\n":
                ch = s.peek()
                if ch == "\\":
                    s.advance(2)
                    continue
                s.advance()
                if ch == "[":
                    in_class = True
                elif ch == "]":
                    in_class = False
                elif ch == "/" and not in_class:
                    break
            last = "/"
        else:
            if not c.isspace():
                last = c
            s.advance()
    return s


def scan_python(text):
    s = Scan(text)
    while s.i < len(text):
        c = s.peek()
        if c == "#":
            line = s.line
            s.add_comment(line, s.take_until("\n"))
        elif c in "\"'":
            # String prefixes (f, r, b, …) are consumed as ordinary characters before the quote. A backslash keeps
            # the next character inside the string even in raw strings.
            statement_start = _docstring_position(text, s.i)
            quote = c * 3 if s.startswith(c * 3) else c
            start = s.i
            line, content = s.quoted(quote)
            if len(quote) == 3 and statement_start:
                s.add_comment(line, content)  # docstring
            else:
                s.strings.append((line, content, start))
        else:
            s.advance()
    return s


def _docstring_position(text, i):
    """True if a string at offset i stands where Python expects a docstring: at the start of the file (after
    comments) or right after a line ending in ':'."""
    line_start = text.rfind("\n", 0, i) + 1
    if text[line_start:i].strip().lower() not in ("", "r", "u"):
        return False
    for line in reversed(text[:line_start].split("\n")):
        stripped = line.strip()
        if stripped and not stripped.startswith("#"):
            return stripped.endswith(":")
    return True


def scan_shell(text):
    s = Scan(text)
    heredoc = None
    while s.i < len(text):
        c = s.peek()
        if heredoc and (s.i == 0 or text[s.i - 1] == "\n"):
            line_text = text[s.i : text.find("\n", s.i) if "\n" in text[s.i :] else len(text)]
            if line_text.strip() == heredoc:
                heredoc = None
            s.take_until("\n")
            continue
        if c == "#" and (s.i == 0 or text[s.i - 1] in " \t\n;(|&"):
            line = s.line
            s.add_comment(line, s.take_until("\n"))
        elif c in "\"'":
            start = s.i
            line, content = s.quoted(c, escapes=(c == '"'))
            s.strings.append((line, content, start))
        elif c == "\\":
            s.advance(2)
        elif s.startswith("<<") and not s.startswith("<<<"):
            m = re.match(r"<<-?\s*(['\"]?)(\w+)\1", text[s.i :])
            if m:
                heredoc = m.group(2)
                s.advance(len(m.group(0)))
            else:
                s.advance(2)
        else:
            s.advance()
    return s


SCANNERS = {"rust": scan_rust, "js": scan_js, "python": scan_python, "shell": scan_shell}

RUST_TEST_ATTR = re.compile(r"^\s*#\[(\w+::)*(test|rstest|test_case)\b")
RUST_FN = re.compile(r"^\s*(pub(\([^)]*\))?\s+)?(async\s+)?fn\s+(\w+)")
PY_TEST = re.compile(r"^\s*(async\s+)?def\s+(test\w*)")
JS_TEST_CALL = re.compile(r"\b(test|it|describe)(\.(describe|only|skip|serial|fixme|parallel|todo))*\s*\(\s*$")


def test_names(lang, text, scan):
    """Yields (line, name, is_identifier) for every test name."""
    lines = text.split("\n")
    # lines that start inside a multi-line string literal (test fixtures) hold no test names
    in_string = {n for line, content, _ in scan.strings for n in range(line + 1, line + 1 + content.count("\n"))}
    if lang == "rust":
        pending = False
        for n, line in enumerate(lines, 1):
            if n in in_string:
                continue
            if RUST_TEST_ATTR.match(line):
                pending = True
                continue
            m = RUST_FN.match(line)
            if m:
                if pending:
                    yield n, m.group(4), True
                pending = False
            elif line.strip() and not line.strip().startswith(("#[", "//")):
                pending = False
    elif lang == "python":
        for n, line in enumerate(lines, 1):
            m = PY_TEST.match(line)
            if m and n not in in_string:
                yield n, m.group(2), True
    elif lang == "js":
        for line, content, start in scan.strings:
            if JS_TEST_CALL.search(text[max(0, start - 40) : start]):
                yield line, content, False


def check_file(path, rel):
    lang = language_of(path)
    if not lang:
        return []
    try:
        with open(path, encoding="utf-8") as f:
            text = f.read()
    except (OSError, UnicodeDecodeError):
        return []
    lines = text.split("\n")
    scan = SCANNERS[lang](text)
    hits = []
    for line, comment in scan.comments:
        if is_german(comment):
            hits.append((line, "comment", comment.strip()))
    for line, name, ident in test_names(lang, text, scan):
        if identifier_is_german(name) if ident else is_german(name, comment=False):
            hits.append((line, "test name", name))
    result = []
    for line, kind, snippet in sorted(set(hits)):
        if ALLOW_MARKER in lines[line - 1]:
            continue
        result.append(f"{rel}:{line}: {kind}: {snippet[:100]}")
    return result


def list_files(root, paths):
    out = subprocess.run(
        ["git", "-C", root, "ls-files", "--cached", "--others", "--exclude-standard", "-z", "--", *paths],
        capture_output=True,
        text=True,
    )
    if out.returncode != 0:
        sys.exit(f"git ls-files failed: {out.stderr.strip()}")
    files = sorted(set(f for f in out.stdout.split("\0") if f))
    return [f for f in files if not EXCLUDED_DIRS.intersection(f.split("/")[:-1])]


def main(argv):
    parser = argparse.ArgumentParser(prog="check-english.sh", description="Reports German comments and test names.")
    parser.add_argument("--all", action="store_true", help="also check paths on the transition list")
    parser.add_argument("--gate", action="store_true", help="check only once the transition list is empty")
    parser.add_argument("--root", help="repository root (default: git top level of the current directory)")
    parser.add_argument("paths", nargs="*", help="files or directories relative to the root (default: %s)" % " ".join(SCOPE))
    try:
        args = parser.parse_args(argv)
    except SystemExit as e:
        return 2 if e.code else 0
    root = args.root
    if not root:
        top = subprocess.run(["git", "rev-parse", "--show-toplevel"], capture_output=True, text=True)
        if top.returncode != 0:
            print("not inside a git repository", file=sys.stderr)
            return 2
        root = top.stdout.strip()
    if not os.path.isdir(root):
        print(f"root not found: {root}", file=sys.stderr)
        return 2
    if args.gate and PENDING:
        print(f"English check skipped: {len(PENDING)} paths still on the transition list.")
        return 0
    pending = [] if args.all else PENDING
    files = [
        f
        for f in list_files(root, args.paths or SCOPE)
        if not any(f.startswith(p) for p in EXCLUDED + pending)
    ]
    hits = []
    for rel in files:
        hits += check_file(os.path.join(root, rel), rel)
    for hit in hits:
        print(hit)
    note = f" ({len(pending)} paths on the transition list skipped)" if pending else ""
    if hits:
        print(f"German text found: {len(hits)} lines in {len(files)} files checked{note}.", file=sys.stderr)
        print(f"Translate, or mark a false positive with «{ALLOW_MARKER}» on the line.", file=sys.stderr)
        return 1
    print(f"English check: no German comments or test names in {len(files)} files{note}.")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
