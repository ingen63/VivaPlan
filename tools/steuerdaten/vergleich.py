#!/usr/bin/env python3
"""#287: Compares two tax data files (`steuern-<year>.json`) per canton and the federal tax.

Reports every curve point whose rate differs by more than the tolerance (relative, default 1 %; rates below 0.0001
count as equal if they differ by less than 0.0001), every deduction rule that differs, and missing or extra fields.
The tax multipliers of the municipalities are not compared (#285). Exit code 1 if there is a difference.

Usage: python3 -I tools/steuerdaten/vergleich.py OLD.json NEW.json [--kanton ZH --kanton CH] [--toleranz 0.01]  «check-english: allow» (option names as in build.py)
"""

import argparse
import json
import sys

CURVES = ("income", "municipalIncome", "wealth", "capital", "capitalMunicipal")
RULES = ("deductions", "propertyGain", "capitalRule", "propertyGainRule", "gainMultiplier")
FEDERAL_CURVES = ("federal", "federalCapital")
ABS_TOL = 0.0001


def differs(a, b, tol):
    """True if two rates differ by more than the relative tolerance (small rates: absolute)."""
    if abs(a - b) < ABS_TOL:
        return False
    return abs(a - b) > tol * max(abs(a), abs(b))


def compare_curves(label, old, new, tol):
    """Differences between two curve sets `{status: [[x, rate], …]}`."""
    out = []
    if old is None or new is None:
        if old != new:
            out.append(f"{label}: {'missing in new' if new is None else 'new, missing in old'}")
        return out
    for status in sorted(set(old) | set(new)):
        a, b = dict(map(tuple, old.get(status, []))), dict(map(tuple, new.get(status, [])))
        for x in sorted(set(a) | set(b)):
            if x not in a or x not in b:
                out.append(f"{label} {status} {x}: point only in {'old' if x in a else 'new'}")
            elif differs(a[x], b[x], tol):
                out.append(f"{label} {status} {x}: {a[x]} -> {b[x]} ({(b[x] - a[x]) / max(a[x], b[x]):+.1%})")
    return out


def compare_value(label, old, new):
    return [] if old == new else [f"{label}: {json.dumps(old, ensure_ascii=False)} -> "
                                  f"{json.dumps(new, ensure_ascii=False)}"]


def compare(old, new, tol=0.01, only=None):
    """All differences as text lines, per authority (`CH` = federal)."""
    out = []
    if only is None or "CH" in only:
        for key in FEDERAL_CURVES:
            out += compare_curves(f"CH {key}", old.get(key), new.get(key), tol)
        out += compare_value("CH federalDeductions", old.get("federalDeductions"), new.get("federalDeductions"))
    a = {c["id"]: c for c in old.get("cantons", [])}
    b = {c["id"]: c for c in new.get("cantons", [])}
    for cid in sorted(set(a) | set(b)):
        if only is not None and cid not in only:
            continue
        if cid not in a or cid not in b:
            out.append(f"{cid}: canton only in {'old' if cid in a else 'new'}")
            continue
        for key in CURVES:
            out += compare_curves(f"{cid} {key}", a[cid].get(key), b[cid].get(key), tol)
        for key in RULES:
            out += compare_value(f"{cid} {key}", a[cid].get(key), b[cid].get(key))
    return out


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("alt")
    ap.add_argument("neu")
    ap.add_argument("--kanton", action="append", help="only this canton (CH = federal), repeatable")
    ap.add_argument("--toleranz", type=float, default=0.01, help="relative tolerance of the rates (default 0.01)")
    a = ap.parse_args(argv)
    with open(a.alt, encoding="utf-8") as f:
        old = json.load(f)
    with open(a.neu, encoding="utf-8") as f:
        new = json.load(f)
    lines = compare(old, new, a.toleranz, set(a.kanton) if a.kanton else None)
    print("\n".join(lines) if lines else "no differences")
    return 1 if lines else 0


if __name__ == "__main__":
    sys.exit(main())
