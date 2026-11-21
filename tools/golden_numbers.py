"""#176/AK2: compares the golden files of two versions without their text fields.

Usage:  python3 tools/golden_numbers.py [--base REV] [--dir DIR] [--gate]
        (REV: git revision of the old files, default HEAD; DIR: default crates/engine/tests/golden)

Both versions must have the same structure (keys, list lengths, types) and exactly the same numbers, booleans and
null values. Strings may differ (texts of the engine, English since #176), except strings that are months (YYYY-MM)
or dates: those count as values. The report names the keys whose strings differ, so the diff can be checked to
contain only text fields.
A golden file that is new since REV is named but not compared; a deleted one counts as a difference.
Exit code: 0 only texts differ, 1 another difference (with its path).

#195: With --gate (push run of test-quality.yml) a difference is accepted when at least one commit in REV..HEAD has a
line `Golden: #<issue> <reason>` in its message (rule in #195); the reasons are listed. Exit code 1 only without one.
"""

import argparse
import collections
import json
import pathlib
import re
import subprocess
import sys

MONTH = re.compile(r"^\d{4}-\d{2}(-\d{2})?$")
# #195: the fixed rule for a justified change of golden numbers: a line of its own in a commit message
REASON = re.compile(r"^Golden: #\d+ +\S.*$", re.MULTILINE)
SHOWN = 50


def compare(old, new, path, report):
    """Compares `old` and `new`; differences go to `report` (keys `values`, `texts`, `numbers`)."""
    if isinstance(old, dict) and isinstance(new, dict):
        for k in sorted(set(old) | set(new)):
            if k not in old or k not in new:
                report["values"].append(f"{path}.{k}: only in {'new' if k in new else 'old'}")
            else:
                compare(old[k], new[k], f"{path}.{k}", report)
    elif isinstance(old, list) and isinstance(new, list):
        if len(old) != len(new):
            report["values"].append(f"{path}: {len(old)} entries instead of {len(new)}")
        for i, (a, b) in enumerate(zip(old, new)):
            compare(a, b, f"{path}[{i}]", report)
    elif isinstance(old, str) and isinstance(new, str):
        if old != new:
            if MONTH.match(old) or MONTH.match(new):
                report["values"].append(f"{path}: {old!r} instead of {new!r}")
            else:
                report["texts"][re.sub(r"\[\d+\]", "[]", path).rsplit(".", 1)[-1]] += 1
    elif type(old) is not type(new) and not (isinstance(old, (int, float)) and isinstance(new, (int, float))
                                               and not isinstance(old, bool) and not isinstance(new, bool)):
        report["values"].append(f"{path}: type {type(old).__name__} instead of {type(new).__name__}")
    else:
        if isinstance(old, (int, float)) and not isinstance(old, bool):
            report["numbers"] += 1
        if old != new:
            report["values"].append(f"{path}: {old!r} instead of {new!r}")


def new_report():
    return {"values": [], "texts": collections.Counter(), "numbers": 0}


def find_reasons(messages):
    """#195: the lines `Golden: #<issue> <reason>` in the given commit messages."""
    return [m.group(0).strip() for m in REASON.finditer(messages)]


def git(root, *args):
    return subprocess.run(["git", *args], cwd=root, capture_output=True, text=True, check=True).stdout


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--base", default="HEAD")
    ap.add_argument("--dir", default="crates/engine/tests/golden")
    ap.add_argument("--gate", action="store_true", help="#195: accept differences justified in REV..HEAD")
    a = ap.parse_args(argv)
    root = pathlib.Path(git(".", "rev-parse", "--show-toplevel").strip())
    rel_dir = (root / a.dir).relative_to(root).as_posix()
    files = sorted((root / a.dir).glob("*.json"))
    old_files = sorted(n for n in git(root, "ls-tree", "--name-only", f"{a.base}:{rel_dir}").splitlines()
                       if n.endswith(".json"))
    report = new_report()
    added = []
    for f in files:
        if f.name not in old_files:
            added.append(f.name)
            continue
        old = json.loads(git(root, "show", f"{a.base}:{rel_dir}/{f.name}"))
        compare(old, json.loads(f.read_text()), f.stem, report)
    for name in sorted(set(old_files) - {f.name for f in files}):
        report["values"].append(f"{pathlib.Path(name).stem}: file deleted")
    print(f"{len(files) - len(added)} files, {report['numbers']} numbers compared against {a.base}")
    if added:
        print("new files (not compared): " + ", ".join(added))
    print("strings changed (key: count): " + (", ".join(f"{k}: {n}" for k, n in sorted(report["texts"].items()))
                                              or "none"))
    if not report["values"]:
        print("OK: all numbers, keys, list lengths and months unchanged")
        return 0
    print(f"{len(report['values'])} differences outside text fields" + (", e.g.:" if len(report["values"]) > SHOWN
                                                                         else ":"))
    for line in report["values"][:SHOWN]:
        print("  " + line)
    if not a.gate:
        return 1
    reasons = find_reasons(git(root, "log", "--format=%B", f"{a.base}..HEAD"))
    if reasons:
        print("Justified by:")
        for r in reasons:
            print("  " + r)
        return 0
    print(f"FAILED: golden numbers changed without a line 'Golden: #<issue> <reason>' in a commit of {a.base}..HEAD")
    return 1


if __name__ == "__main__":
    sys.exit(main())
