"""#176/AK2: compares the golden files of two versions without their text fields.

Usage:  python3 tools/golden_numbers.py [--base REV] [--dir DIR]
        (REV: git revision of the old files, default HEAD; DIR: default crates/engine/tests/golden)

Both versions must have the same structure (keys, list lengths, types) and exactly the same numbers, booleans and
null values. Strings may differ (texts of the engine, English since #176), except strings that are months (YYYY-MM)
or dates: those count as values. The report names the keys whose strings differ, so the diff can be checked to
contain only text fields.
Exit code: 0 only texts differ, 1 another difference (with its path).
"""

import argparse
import collections
import json
import pathlib
import re
import subprocess
import sys

MONTH = re.compile(r"^\d{4}-\d{2}(-\d{2})?$")


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


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--base", default="HEAD")
    ap.add_argument("--dir", default="crates/engine/tests/golden")
    a = ap.parse_args(argv)
    root = pathlib.Path(subprocess.run(["git", "rev-parse", "--show-toplevel"], capture_output=True, text=True,
                                       check=True).stdout.strip())
    files = sorted((root / a.dir).glob("*.json"))
    report = new_report()
    for f in files:
        rel = f.relative_to(root).as_posix()
        old = json.loads(subprocess.run(["git", "show", f"{a.base}:{rel}"], cwd=root, capture_output=True,
                                        text=True, check=True).stdout)
        compare(old, json.loads(f.read_text()), f.stem, report)
    print(f"{len(files)} files, {report['numbers']} numbers compared against {a.base}")
    print("strings changed (key: count): " + (", ".join(f"{k}: {n}" for k, n in sorted(report["texts"].items()))
                                              or "none"))
    if report["values"]:
        print(f"{len(report['values'])} differences outside text fields, e.g.:")
        for line in report["values"][:20]:
            print("  " + line)
        return 1
    print("OK: all numbers, keys, list lengths and months unchanged")
    return 0


if __name__ == "__main__":
    sys.exit(main())
