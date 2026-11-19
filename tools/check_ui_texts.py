"""#170/AK1: reports German texts of the user interface outside the text tables (crates/app/ui/i18n/*.json).

Usage:  python3 tools/check_ui_texts.py [--root DIR]
Exit code: 0 no hit, 1 hits (with file and line).

Checked:
- crates/app/ui/*.js: string literals and the text of template literals (outside HTML tags; the values of the
  attributes title, aria-label, placeholder and alt are checked as well);
- crates/app/ui/index.html: text between tags and the attributes above (the page fills them with `data-i18n…`);
- crates/app/src/*.rs without the tests: string literals (native menus, file dialogs, notices).

A text counts as German with the rules of tools/check_english.py (umlaut, German word, German ending) or a word of
UI_WORDS. A line containing «i18n: allow» is not reported (e.g. texts of the engine compared until #171, names of
files, messages that only appear in logs).
"""

import argparse
import os
import pathlib
import re
import subprocess
import sys

sys.path.insert(0, str(pathlib.Path(__file__).resolve().parent))
from check_english import is_german, scan_rust  # noqa: E402

ALLOW = "i18n: allow"
# Short or lowercase German words of the user interface that check_english does not know.
UI_WORDS = {
    "ab", "je", "ja", "nein", "nie", "heute", "direkt", "indirekt", "abgelaufen", "vergangen", "ungespeichert",
    "nirgends", "reicht", "aufgebraucht", "absteigend", "aufsteigend", "gerechnet", "zeitabhängig", "fest",
}
TEXT_ATTRS = ("title", "aria-label", "placeholder", "alt")
TAG = re.compile(r"<[^<>]*>|<[^<>]*$|^[^<>]*>")
ATTR = re.compile(r'\b(' + "|".join(TEXT_ATTRS) + r')="([^"$]*)"')
WORD = re.compile(r"[A-Za-zÄÖÜäöüß]+")


def german(text):
    return is_german(text, comment=False) or any(w.lower() in UI_WORDS for w in WORD.findall(text))


# --- JavaScript: string literals and template literals (with nesting) ---


def js_segments(text):
    """Yields (offset, text) of string literals and of the literal parts of template literals."""
    out, n = [], len(text)

    def skip_string(i, q):
        j = i + 1
        while j < n:
            if text[j] == "\\":
                j += 2
                continue
            if text[j] == q:
                return j + 1
            j += 1
        return n

    def template(i):
        j, start = i + 1, i + 1
        while j < n:
            c = text[j]
            if c == "\\":
                j += 2
                continue
            if c == "`":
                out.append((start, text[start:j]))
                return j + 1
            if c == "$" and text.startswith("{", j + 1):
                out.append((start, text[start:j]))
                j = code(j + 2, True) + 1
                start = j
                continue
            j += 1
        return n

    def code(j, inner):
        depth, last = 0, ""
        while j < n:
            c = text[j]
            if text.startswith("//", j):
                k = text.find("\n", j)
                j = n if k < 0 else k
                continue
            if text.startswith("/*", j):
                k = text.find("*/", j)
                j = n if k < 0 else k + 2
                continue
            if c in "\"'":
                e = skip_string(j, c)
                out.append((j + 1, text[j + 1 : e - 1]))
                j, last = e, '"'
                continue
            if c == "`":
                j, last = template(j), "`"
                continue
            if c == "/" and (last == "" or last in "(,=:[!&|?{};+-*%<>~^"):
                k, in_class = j + 1, False
                while k < n and text[k] != "\n":
                    ch = text[k]
                    if ch == "\\":
                        k += 2
                        continue
                    k += 1
                    if ch == "[":
                        in_class = True
                    elif ch == "]":
                        in_class = False
                    elif ch == "/" and not in_class:
                        break
                j, last = k, "/"
                continue
            if inner:
                if c == "{":
                    depth += 1
                elif c == "}":
                    if depth == 0:
                        return j
                    depth -= 1
            if not c.isspace():
                last = c
            j += 1
        return n

    code(0, False)
    return out


def markup_texts(s):
    """Text outside tags and the values of the text attributes."""
    return [TAG.sub(" ", s)] + [m.group(2) for m in ATTR.finditer(s)]


def check_js(path, text):
    hits = []
    lines = text.split("\n")
    for offset, seg in js_segments(text):
        line = text.count("\n", 0, offset) + 1
        if ALLOW in lines[line - 1]:
            continue
        if any(german(t) for t in markup_texts(seg)):
            hits.append((line, seg.strip()[:80]))
    return hits


def check_html(path, text):
    hits = []
    body = text[text.find("<body") :] if "<body" in text else text
    start_line = text.count("\n", 0, len(text) - len(body))
    body = re.sub(r"<script\b[\s\S]*?</script>|<!--[\s\S]*?-->", lambda m: "\n" * m.group(0).count("\n"), body)
    for i, line in enumerate(body.split("\n"), start=start_line + 1):
        if ALLOW not in line and any(german(t) for t in markup_texts(line)):
            hits.append((i, line.strip()[:80]))
    return hits


def check_rust(path, text):
    cut = text.find("#[cfg(test)]\nmod tests")
    code = text if cut < 0 else text[:cut]
    lines = code.split("\n")
    hits = []
    for line, content, _ in scan_rust(code).strings:
        if ALLOW in lines[line - 1] or not german(content):
            continue
        hits.append((line, content[:80]))
    return hits


def files(root):
    ui = root / "crates/app/ui"
    yield from ((p, check_js) for p in sorted(ui.glob("*.js")))
    yield ui / "index.html", check_html
    yield from ((p, check_rust) for p in sorted((root / "crates/app/src").glob("*.rs")))


def main(argv):
    parser = argparse.ArgumentParser(prog="check_ui_texts.py", description=__doc__.split("\n")[0])
    parser.add_argument("--root", help="repository root (default: git top level of the current directory)")
    args = parser.parse_args(argv)
    root = pathlib.Path(
        args.root
        or subprocess.run(["git", "rev-parse", "--show-toplevel"], capture_output=True, text=True).stdout.strip()
        or os.getcwd()
    )
    hits = 0
    for path, check in files(root):
        if not path.exists():
            continue
        for line, text in check(path, path.read_text(encoding="utf-8")):
            print(f"{path.relative_to(root)}:{line}: {text}")
            hits += 1
    if hits:
        print(f"{hits} German texts outside the text table (crates/app/ui/i18n/de.json).")
    return 1 if hits else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
