#!/usr/bin/env python3
"""#226: lists every Rust test over its run time limit that has no reason comment.

Reads the run times from the nextest JUnit file (profile ci, `target/nextest/ci/junit.xml`) and assigns every test to
a kind with its own limit (test profile: debug build with opt-level 1):

- unit test (library or program target, `src/`): 0.1 s
- integration test (target under `tests/`): 1 s
- test that starts a program or a server: 15 s. These are the tests of the targets in PROGRAM_TARGETS (they start
  the `vivaplan` binary) and tests whose function body uses a marker of PROGRAM_MARKERS (development server, Tauri
  test app).

A test over its limit is accepted when the comment and attribute lines right above its `fn` contain
`// slow: <reason>`. The finding is a hint, not an error: the exit code is 0 unless the JUnit file cannot be read.

Usage:  python3 tools/slow_tests.py [JUNIT] [--root DIR] [--markdown]
  JUNIT       default target/nextest/ci/junit.xml
  --root      repository directory (default: the one of this script)
  --markdown  output for the GitHub run summary
"""

import argparse
import pathlib
import re
import sys
import xml.etree.ElementTree as ET

UNIT, INTEGRATION, PROGRAM = "unit", "integration", "program"
LIMITS = {UNIT: 0.1, INTEGRATION: 1.0, PROGRAM: 15.0}
KIND_NAMES = {UNIT: "Unit-Test", INTEGRATION: "Integrationstest", PROGRAM: "startet Programm/Server"}
# nextest binary ids whose tests all start a program
PROGRAM_TARGETS = {"vivaplan-cli::cli", "vivaplan-cli::mcp"}
# package -> pattern in the body of a test function that starts a program or a server
PROGRAM_MARKERS = {
    "vivaplan-cli": re.compile(r"Command::new|CARGO_BIN_EXE|\bserve\("),
    "vivaplan-app": re.compile(r"\b(setup|setup_with|mock_builder)\("),
}
REASON = re.compile(r"//\s*slow:\s*\S")


def packages(root):
    """Package name -> crate folder, from crates/*/Cargo.toml."""
    out = {}
    for toml in sorted(root.glob("crates/*/Cargo.toml")):
        m = re.search(r'^name\s*=\s*"([^"]+)"', toml.read_text(encoding="utf-8"), re.M)
        if m:
            out[m.group(1)] = toml.parent
    return out


def candidate_files(crate, target):
    """Source files of a nextest target: library/program -> src/, test target -> tests/ (the engine has a single
    target `all` with every file as a module, so the module path decides between files)."""
    folder = "src" if target is None or target.startswith("bin/") else "tests"
    return sorted((crate / folder).rglob("*.rs"))


def find_test(files, test_path):
    """(file, lines, index of the fn line) of the test function, or None. Several hits: the file whose path contains
    the most module names of the test path wins."""
    *modules, name = test_path.split("::")
    fn = re.compile(r"\bfn\s+" + re.escape(name) + r"\s*[(<]")
    hits = []
    for f in files:
        lines = f.read_text(encoding="utf-8").splitlines()
        for i, line in enumerate(lines):
            if fn.search(line):
                hits.append((f, lines, i))
    if not hits:
        return None
    return max(hits, key=lambda h: sum(m in h[0].parts or m == h[0].stem for m in modules))


def has_reason(lines, i):
    """`// slow: <reason>` among the comment and attribute lines right above line i."""
    j = i - 1
    while j >= 0:
        s = lines[j].strip()
        if not (s.startswith("//") or s.startswith("#[") or s.startswith("#![")):
            return False
        if REASON.search(s):
            return True
        j -= 1
    return False


def body(lines, i):
    """Text of the function starting at line i, up to its closing brace (braces in strings are not special-cased)."""
    depth, out, opened = 0, [], False
    for line in lines[i:]:
        out.append(line)
        depth += line.count("{") - line.count("}")
        opened = opened or "{" in line
        if opened and depth <= 0:
            break
    return "\n".join(out)


def kind_of(binary, package, src):
    if binary in PROGRAM_TARGETS:
        return PROGRAM
    marker = PROGRAM_MARKERS.get(package)
    if src and marker and marker.search(body(*src[1:])):
        return PROGRAM
    target = binary.split("::", 1)[1] if "::" in binary else None
    return UNIT if target is None or target.startswith("bin/") else INTEGRATION


def check(junit, root):
    """All tests over their limit: dicts with test, kind, limit, time, reason (bool) and place (file:line or None)."""
    root = pathlib.Path(root)
    crates = packages(root)
    out = []
    for tc in ET.parse(junit).getroot().iter("testcase"):
        binary, name = tc.get("classname", ""), tc.get("name", "")
        t = float(tc.get("time", 0) or 0)
        package = binary.split("::", 1)[0]
        target = binary.split("::", 1)[1] if "::" in binary else None
        crate = crates.get(package)
        src = find_test(candidate_files(crate, target), name) if crate else None
        kind = kind_of(binary, package, src)
        if t <= LIMITS[kind]:
            continue
        place = f"{src[0].relative_to(root)}:{src[2] + 1}" if src else None
        out.append({"test": f"{binary} {name}", "kind": kind, "limit": LIMITS[kind], "time": t,
                    "reason": bool(src and has_reason(src[1], src[2])), "place": place})
    return sorted(out, key=lambda r: -r["time"])


def report(rows, markdown=False):
    open_ = [r for r in rows if not r["reason"]]
    lines = []
    if markdown:
        lines += ["### Laufzeitgrenzen der Tests (#226)", "",
                  f"{len(open_)} Tests über ihrer Grenze ohne `// slow: <Grund>`, {len(rows) - len(open_)} begründet "
                  "(Grenzen: Unit-Test 0,1 s, Integrationstest 1 s, startet Programm/Server 15 s; Hinweis, kein Fehler)."]
        if open_:
            lines += ["", "| Test | Art | Grenze | Dauer | Ort |", "|---|---|---:|---:|---|"]
            lines += [f"| `{r['test']}` | {KIND_NAMES[r['kind']]} | {r['limit']:g} s | {r['time']:.2f} s | "
                      f"{r['place'] or '–'} |" for r in open_]
    else:
        lines += [f"{r['time']:8.2f} s  > {r['limit']:g} s  {r['test']}  ({r['place'] or 'Quelle nicht gefunden'})"
                  for r in open_]
        lines.append(f"{len(open_)} Tests über ihrer Grenze ohne // slow: <Grund>, {len(rows) - len(open_)} begründet.")
    return "\n".join(lines)


def main(argv=None):
    here = pathlib.Path(__file__).resolve().parent.parent
    p = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    p.add_argument("junit", nargs="?", default=None)
    p.add_argument("--root", default=str(here))
    p.add_argument("--markdown", action="store_true")
    a = p.parse_args(argv)
    root = pathlib.Path(a.root)
    junit = pathlib.Path(a.junit) if a.junit else root / "target" / "nextest" / "ci" / "junit.xml"
    try:
        rows = check(junit, root)
    except (OSError, ET.ParseError) as e:
        print(f"Keine Laufzeiten: {junit} nicht lesbar ({e}).", file=sys.stderr)
        return 2
    print(report(rows, a.markdown))
    return 0


if __name__ == "__main__":
    sys.exit(main())
