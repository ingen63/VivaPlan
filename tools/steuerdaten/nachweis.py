#!/usr/bin/env python3
"""#285: Proof that the tax data file is built only from open, official sources and contains no data of the ESTV
tax calculator (#283).

Checks (each line PASS or FAIL, exit code 1 if one fails):
1. every source file the description of the cantons (`kantone.json`) and the exceptions (`ausnahmen.json`) use is in
   the list of sources (`steuerfuesse_quellen.json`) with address, date and SHA-256, and the local file has that hash;
   derived files (text of a PDF, CSV of an Excel table) name their original;
2. no source comes from the ESTV (estv.admin.ch, estv2.admin.ch, swisstaxcalculator);
3. every row of the source files from the laws (`<KT>_tarife.csv`, `<KT>_abzuege.csv`) and every rule for lump sums
   names its legal source, none an ESTV calculation;
4. built again from these files, the tax data file is byte for byte the one in the repository; its `source` names no
   ESTV data.

Usage: python3 -I tools/steuerdaten/nachweis.py --ordner local/steuerquellen/<year>/steuerfuesse [--jahr <year>]  «check-english: allow» (option names)
"""

import argparse
import csv
import glob
import hashlib
import json
import os
import re
import subprocess
import sys
import tempfile

HERE = os.path.dirname(os.path.abspath(__file__))
ROOT = os.path.dirname(os.path.dirname(HERE))
ESTV = re.compile(r"estv\d?\.admin\.ch|swisstaxcalculator", re.I)
ESTV_TEXT = re.compile(r"ESTV-Rechnung|ESTV-Grunddaten|ESTV-Steuerrechner|aus den ESTV", re.I)


def sha256(path):
    with open(path, "rb") as f:
        return hashlib.sha256(f.read()).hexdigest()


def used_files(specs, exceptions):
    """{file: reason} of all source files the descriptions use."""
    out = {}
    for canton, spec in specs.items():
        if canton.startswith("_"):
            continue
        out[spec["file"]] = canton
        for extra in spec.get("also", []) + spec.get("wealth", {}).get("also", []):
            out[extra["file"]] = canton
    for canton, rows in exceptions.items():
        if canton.startswith("_"):
            continue
        for bfs, row in rows.items():
            out[row.get("beleg", f"(no beleg for {canton} {bfs})")] = f"exception {canton} {bfs}"
    return out


def check_sources(folder, quellen, used):
    results = []
    listed = {q["datei"]: q for q in quellen["quellen"]}
    derived = {d["datei"]: d for d in quellen["abgeleitet"]}
    for name, reason in sorted(used.items()):
        if name in derived:
            origin = derived[name]["aus"]
            ok = origin in listed
            results.append((ok, f"{name} ({reason}): derived from {origin}" + ("" if ok else ", original not listed")))
        elif name not in listed:
            results.append((False, f"{name} ({reason}): not in the list of sources"))
    for entry in list(listed.values()) + list(derived.values()):
        path = os.path.join(folder, entry["datei"])
        if not os.path.exists(path):
            results.append((False, f"{entry['datei']}: missing in {folder}"))
        elif sha256(path) != entry["sha256"]:
            results.append((False, f"{entry['datei']}: SHA-256 differs from the list"))
    if all(ok for ok, _ in results):
        results.append((True, f"{len(listed)} sources and {len(derived)} derived files present with their SHA-256"))
    estv = [q["datei"] for q in listed.values() if ESTV.search(q["url"])]
    results.append((not estv, "no source from the ESTV" if not estv else f"sources from the ESTV: {', '.join(estv)}"))
    return results


def check_laws(sources_dir):
    results = []
    rows = 0
    for path in sorted(glob.glob(os.path.join(sources_dir, "*_tarife.csv")) +
                       glob.glob(os.path.join(sources_dir, "*_abzuege.csv"))):
        with open(path, encoding="utf-8") as f:
            for i, row in enumerate(csv.DictReader(f), 2):
                rows += 1
                q = row.get("Quelle", "")
                if not q.strip() or ESTV_TEXT.search(q) or ESTV.search(q):
                    results.append((False, f"{os.path.basename(path)} line {i}: source «{q}»"))
    results.append((not results, f"{rows} rows of tariffs and deductions name their legal source"))
    with open(os.path.join(HERE, "kapitalregeln.json"), encoding="utf-8") as f:
        rules = {k: v for k, v in json.load(f).items() if not k.startswith("_")}
    bad = [c for c, r in rules.items() if not r.get("source") or ESTV_TEXT.search(r["source"])]
    results.append((not bad, f"{len(rules)} rules for lump sums name the law" + (f"; not: {', '.join(bad)}" if bad
                                                                                  else "")))
    return results


def check_build(folder, year):
    target = os.path.join(ROOT, "crates", "engine", "data", f"steuern-{year}.json")
    with tempfile.TemporaryDirectory() as tmp:
        out = os.path.join(tmp, "steuern.json")
        run = subprocess.run([sys.executable, "-I", os.path.join(HERE, "build.py"), "--steuerfuesse-ordner", folder,
                              "--jahr", str(year), "--out", out], capture_output=True, text=True)
        if run.returncode != 0:
            return [(False, "build failed: " + run.stderr.strip()[-300:])]
        same = sha256(out) == sha256(target)
        with open(out, encoding="utf-8") as f:
            data = json.load(f)
    text = f"built again from the sources, steuern-{year}.json "
    results = [(same, text + (f"is identical (SHA-256 {sha256(target)})" if same else "differs"))]
    estv = "ESTV" in data.get("source", "")
    results.append((not estv, f"source of the tax data file: «{data.get('source', '')}»"))
    return results


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--ordner", required=True, help="folder with the source files")
    ap.add_argument("--jahr", type=int, default=2026)
    a = ap.parse_args(argv)
    sources_dir = os.path.join(HERE, "quellen", str(a.jahr))
    with open(os.path.join(sources_dir, "steuerfuesse_quellen.json"), encoding="utf-8") as f:
        quellen = json.load(f)
    with open(os.path.join(sources_dir, "kantone.json"), encoding="utf-8") as f:
        specs = json.load(f)
    with open(os.path.join(sources_dir, "ausnahmen.json"), encoding="utf-8") as f:
        exceptions = json.load(f)
    results = (check_sources(a.ordner, quellen, used_files(specs, exceptions)) + check_laws(sources_dir)
               + check_build(a.ordner, a.jahr))
    for ok, text in results:
        print(("PASS " if ok else "FAIL ") + text)
    return 0 if all(ok for ok, _ in results) else 1


if __name__ == "__main__":
    sys.exit(main())
