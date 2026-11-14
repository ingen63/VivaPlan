#!/usr/bin/env python3
"""Abdeckung des geänderten Codes (FEA-28).

Liest eine lcov-Datei und vergleicht sie mit den geänderten Rust-Zeilen gegenüber einem Basis-Commit.
Ausgabe als Markdown (für die GitHub-Zusammenfassung und das Terminal). Ein Hinweis, keine Sperre: Der Aufruf
endet immer mit Exit-Code 0, ausser bei falscher Bedienung.

Aufruf: coverage_diff.py <lcov.info> [<Basis, Vorgabe origin/main>]
Mit der Basis wird `git diff -U0 <Basis>...HEAD` verwendet; ohne Commits seit der Basis zusätzlich der Arbeitsbaum
(`git diff -U0 <Basis>`), damit der lokale Befehl auch ungespeicherte Änderungen zeigt.
"""
import re
import subprocess
import sys
from collections import defaultdict


def changed_lines(base, working_tree):
    """Geänderte bzw. neue Zeilen je Rust-Datei: {Pfad: Menge von Zeilennummern}."""
    spec = [base] if working_tree else [f"{base}...HEAD"]
    out = subprocess.run(["git", "diff", "-U0", "--no-color", *spec, "--", "*.rs"], capture_output=True, text=True, check=True).stdout
    files, cur = defaultdict(set), None
    for line in out.splitlines():
        if line.startswith("+++ "):
            cur = None if line == "+++ /dev/null" else line[6:]
        elif line.startswith("@@") and cur:
            m = re.match(r"@@ -\d+(?:,\d+)? \+(\d+)(?:,(\d+))? @@", line)
            start, count = int(m.group(1)), int(m.group(2) if m.group(2) is not None else 1)
            files[cur].update(range(start, start + count))
    return files


def read_lcov(path):
    """Ausführbare Zeilen je Datei: {Pfad (relativ zum Repository): {Zeile: Treffer}}."""
    root = subprocess.run(["git", "rev-parse", "--show-toplevel"], capture_output=True, text=True, check=True).stdout.strip() + "/"
    data, cur = {}, None
    for line in open(path, encoding="utf-8"):
        line = line.strip()
        if line.startswith("SF:"):
            f = line[3:]
            cur = data.setdefault(f[len(root):] if f.startswith(root) else f, {})
        elif line.startswith("DA:") and cur is not None:
            n, hits = line[3:].split(",")[:2]
            cur[int(n)] = max(cur.get(int(n), 0), int(hits))
    return data


def ranges(nums):
    nums, out = sorted(nums), []
    for n in nums:
        if out and n == out[-1][1] + 1:
            out[-1][1] = n
        else:
            out.append([n, n])
    return ", ".join(str(a) if a == b else f"{a}–{b}" for a, b in out)


def main():
    if len(sys.argv) < 2:
        print(__doc__)
        return 2
    lcov, base = sys.argv[1], sys.argv[2] if len(sys.argv) > 2 else "origin/main"
    changed = changed_lines(base, working_tree=False)
    for f, s in changed_lines(base, working_tree=True).items():
        changed.setdefault(f, set()).update(s)
    changed = {f: s for f, s in changed.items() if s}
    print("### Abdeckung des geänderten Codes\n")
    if not changed:
        print("Keine Rust-Änderungen.")
        return 0
    cov = read_lcov(lcov)
    total = covered = 0
    missing = {}
    for f, lines in sorted(changed.items()):
        exe = {n: h for n, h in cov.get(f, {}).items() if n in lines}
        total += len(exe)
        covered += sum(1 for h in exe.values() if h > 0)
        miss = [n for n, h in exe.items() if h == 0]
        if miss:
            missing[f] = miss
    if total == 0:
        print("Keine ausführbaren Rust-Zeilen geändert (nur Kommentare, Tests oder Dateien ausserhalb der Messung).")
        return 0
    pct = 100 * covered / total
    print(f"**{pct:.1f} %** der geänderten ausführbaren Zeilen werden von Tests ausgeführt ({covered} von {total}). Das ist ein Hinweis, keine Sperre; die Untergrenze von 80 % gilt je Paket.\n")
    if missing:
        print("Nicht abgedeckte Zeilen:\n")
        for f, miss in missing.items():
            print(f"- `{f}`: {ranges(miss)}")
    else:
        print("Alle geänderten ausführbaren Zeilen sind abgedeckt.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
