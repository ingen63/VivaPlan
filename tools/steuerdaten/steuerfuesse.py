#!/usr/bin/env python3
"""#285 (way b3, proof of concept): tax multipliers of the municipalities from the cantons' own publications instead of
the ESTV tax calculator, compared with a tax data file (`steuern-<year>.json`).

Sources (downloaded by hand, not part of the repository, no network access here):
- ZH: open data of the statistical office of the canton, `kanton_zuerich_stf_aktuell.csv` (opendata.swiss): municipal
  multiplier without church and with each church in percent. The cantonal multiplier is not in the file; it is the
  single value set by the cantonal parliament (`--zh-canton`, default 0.95 for 2026).
- BE: yearly PDF of the cantonal finance administration on the municipal tax quotas and the church tax rates
  (`fv-stat-steueranlage_<year>_1.pdf`), table 2.1: municipal tax quota and church tax rates as multiples of the
  simple tax; the cantonal quota stands in section 1.2. The PDF has no BFS numbers: names are matched with the
  reference file.

Output: one line per difference and a summary per canton. Exit code 1 if a value differs or a municipality is missing.

Usage: python3 -I tools/steuerdaten/steuerfuesse.py REFERENCE.json [--zh CSV] [--be PDF] [--zh-canton 0.95]  «check-english: allow» (path)
"""

import argparse
import csv
import json
import re
import sys
import zlib

TOL = 0.0005  # multipliers are published with at most 4 decimals
CHURCHES = ("reformed", "catholic", "christ catholic")


def pct(v):
    return round(float(v) / 100, 6) if v not in (None, "") else None


def read_zh(path, canton):
    """{bfs: (name, [canton, municipal, reformed, catholic, christ catholic], notes)} from the ZH open data CSV.
    The church columns are municipal + church; the church share is the difference. Columns 2 and 3 are filled where a
    municipality has several parts with different multipliers; only part 1 is taken, the others are noted."""
    out = {}
    with open(path, encoding="utf-8-sig") as f:
        for r in csv.DictReader(f):
            base = pct(r["STF_O_KIRCHE1"])
            churches = [round(pct(r[c + "1"]) - base, 6) for c in ("STF_REF", "STF_KATH", "STF_CKRKATH")]
            notes = []
            if any(r.get(f"STF_{c}{n}") for c in ("O_KIRCHE", "REF", "KATH", "CKRKATH") for n in (2, 3)):
                notes.append("several parts with different multipliers, part 1 taken")
            if r.get("FLAG") and r["FLAG"] != "definitiv":
                notes.append(f"status {r['FLAG']}")
            out[int(r["BFSNR"])] = (r["COMMUNITY"], [canton, base] + churches, notes)
    return out


# --- PDF text without libraries: enough for the table of the Bern finance administration --------------------------

TOKEN = re.compile(
    rb"\[(?P<tj>(?:\\.|[^\]\\])*)\]\s*TJ"
    rb"|\((?P<s>(?:\\.|[^\\)])*)\)\s*Tj"
    rb"|(?P<td>-?[\d.]+\s+-?[\d.]+)\s+T[dD]"
    rb"|(?P<tm>(?:-?[\d.]+\s+){5}-?[\d.]+)\s+Tm"
    rb"|\bBT\b"
)
PIECE = re.compile(rb"\((?:\\.|[^\\)])*\)|-?[\d.]+")


def unescape(s):
    s = re.sub(rb"\\([0-7]{1,3})", lambda m: bytes([int(m.group(1), 8) & 0xFF]), s)
    s = re.sub(rb"\\(.)", lambda m: {b"n": b"\n", b"r": b"", b"t": b"\t"}.get(m.group(1), m.group(1)), s)
    # The font of the Bern PDF codes the ligatures fl and fi as 0x1F and 0x1D and the thin space as 0x1E.
    return s.decode("cp1252", errors="replace").replace("\x1f", "fl").replace("\x1d", "fi")


def text_lines(content):
    """Lines of one page content stream: text pieces grouped by their baseline, left to right, joined with « | »."""
    items, x, y, scale = [], 0.0, 0.0, 1.0
    for m in TOKEN.finditer(content):
        if m.group(0) == b"BT":
            x = y = 0.0
        elif m.group("tm"):
            a, _, _, d, e, f = map(float, m.group("tm").split())
            scale, x, y = d or a or 1.0, e, f
        elif m.group("td"):
            dx, dy = map(float, m.group("td").split())
            x, y = x + dx * scale, y + dy * scale
        else:
            if m.group("tj") is not None:
                text = ""
                for p in PIECE.findall(m.group("tj")):
                    if p.startswith(b"("):
                        text += unescape(p[1:-1])
                    elif float(p) < -200:  # a wide kerning gap is a space
                        text += " "
            else:
                text = unescape(m.group("s"))
            items.append((round(y), x, text))
    rows = {}
    for yy, xx, text in items:
        rows.setdefault(yy, []).append((xx, text))
    return [" | ".join(t for _, t in sorted(v)) for _, v in sorted(rows.items(), reverse=True)]


def pdf_lines(path):
    with open(path, "rb") as f:
        data = f.read()
    lines = []
    for raw in re.findall(rb"stream\r?\n(.*?)\r?\nendstream", data, re.S):
        try:
            content = zlib.decompress(raw)
        except zlib.error:
            continue
        if b"Tj" in content or b"TJ" in content:
            lines.extend(text_lines(content))
    return lines


NUM = r"\d+\.\d+"
COUNT = r"\d+"  # counts, thousands separators removed by `flat`
# A row of table 2.1 after `flat`: name, quota natural and legal persons, property tax rate, two counts, the church
# rates (reformed, Roman Catholic, Christ Catholic) and the rate for legal persons. Rows of sections (Kirchgemeinden
# inside a municipality with other church rates) have no quotas and do not match.
BE_ROW = re.compile(rf"^(?P<name>\D+?)\**\s+(?P<nat>{NUM})\s+{NUM}\s+{NUM}\s+{COUNT}\s+{COUNT}"
                    rf"\s+(?P<ref>{NUM})\s+(?P<cat>{NUM})\s+(?P<chr>{NUM})\s+{NUM}$")
SECTION_ROW = re.compile(rf"^(?P<name>\D+?)\s+{COUNT}\s+{COUNT}\s+{NUM}\s+{NUM}\s+{NUM}\s+{NUM}$")


def flat(line):
    """A line with the pieces separated by single spaces; names split at a hyphen («Forst | - | Ebenau») joined."""
    # thousands separator (a thin space in the PDF, which Python counts as \s)
    line = re.sub(r"(\d)[\x1e'](\d)", r"\1\2", line)
    line = re.sub(r"\s*\|\s*", " ", line).strip()
    return re.sub(r"\s+-\s+", "-", re.sub(r"\s+", " ", line))


def read_be(lines, year):
    """({name: [canton, municipal, reformed, catholic, christ catholic]}, canton quota, {name: [sections]}) from the
    lines of the PDF; rows of table 2.1 (alphabetical), the first occurrence of each name. Sections are parts of a
    municipality with other church rates (listed below it)."""
    quota = None
    for line in lines:
        m = re.search(rf"\b{year}\s+(\d\.\d+)\s*/\s*\d\.\d+", line)
        if m:
            quota = float(m.group(1))
            break
    if quota is None:
        raise ValueError(f"cantonal tax quota {year} not found in the PDF")
    out, sections, in_table, last = {}, {}, False, None
    for line in lines:
        if line.startswith("Tabelle 2."):
            in_table = line.startswith("Tabelle 2.1")
        if not in_table:
            continue
        m = BE_ROW.match(flat(line))
        if m and m.group("name").strip() not in out:
            last = m.group("name").strip()
            out[last] = [quota, float(m.group("nat")),
                         float(m.group("ref")), float(m.group("cat")), float(m.group("chr"))]
        elif last and SECTION_ROW.match(flat(line)):
            sections.setdefault(last, []).append(SECTION_ROW.match(flat(line)).group("name").strip())
    return out, quota, sections


def words(name):
    """Words of a name in lower case without umlauts, without a part in parentheses («Bargen (BE)»)."""
    name = re.sub(r"\(.*?\)", " ", name.lower())
    for a, b in (("ä", "a"), ("ö", "o"), ("ü", "u"), ("é", "e"), ("è", "e")):
        name = name.replace(a, b)
    return re.findall(r"[a-z0-9]+", name)


def keys(name):
    """Keys of a name from the most to the least exact: all words; first word with the initials of the others
    («Muri b. B.» = «Muri bei Bern»); first word alone."""
    w = words(name)
    return [" ".join(w), w[0] + " " + "".join(x[0] for x in w[1:]), w[0]] if w else []


def match_names(rows, reference):
    """{bfs: (name, values, notes)} for the rows of a source without BFS numbers. The PDF abbreviates names
    («Aeschi b. Sp.», «Büren a. A.»); a key counts only if it belongs to exactly one municipality of the reference
    and one row. Unmatched names are returned separately."""
    out, unmatched = {}, []
    for level in range(3):
        ref_keys, row_keys = {}, {}
        for m in reference:
            if m[0] not in out:
                ref_keys.setdefault(keys(m[1])[level], []).append(m[0])
        for name in rows:
            if all(v[0] != name for v in out.values()):
                row_keys.setdefault(keys(name)[level], []).append(name)
        for key, names in row_keys.items():
            if len(names) == 1 and len(ref_keys.get(key, [])) == 1:
                out[ref_keys[key][0]] = (names[0], rows[names[0]], [])
    unmatched = [n for n in rows if all(v[0] != n for v in out.values())]
    return out, unmatched


LABELS = ("canton", "municipal") + CHURCHES


def compare(canton, source, reference):
    """Differences of the income multipliers of one canton; returns (lines, counts)."""
    ref = {m[0]: m for m in reference if m[2] == canton}
    lines, equal = [], 0
    for bfs in sorted(set(ref) | set(source)):
        if bfs not in source:
            lines.append(f"{canton} {bfs} {ref[bfs][1]}: missing in the source")
            continue
        name, values, notes = source[bfs]
        if bfs not in ref:
            lines.append(f"{canton} {bfs} {name}: not in the reference")
            continue
        diffs = [f"{LABELS[i]} {values[i]:g} (reference {ref[bfs][3][i]:g})"
                 for i in range(5) if abs(values[i] - ref[bfs][3][i]) > TOL]
        if diffs:
            lines.append(f"{canton} {bfs} {name}: " + ", ".join(diffs))
        else:
            equal += 1
        lines.extend(f"{canton} {bfs} {name}: note: {n}" for n in notes)
    return lines, {"reference": len(ref), "source": len(source), "equal": equal}


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("reference")
    ap.add_argument("--zh")
    ap.add_argument("--be")
    ap.add_argument("--zh-canton", type=float, default=0.95)
    a = ap.parse_args(argv)
    with open(a.reference, encoding="utf-8") as f:
        data = json.load(f)
    reference, year = data["municipalities"], data.get("year", 2026)
    failed = False
    sources = []
    if a.zh:
        sources.append(("ZH", read_zh(a.zh, a.zh_canton), []))
    if a.be:
        rows, _, sections = read_be(pdf_lines(a.be), year)
        matched, unmatched = match_names(rows, [m for m in reference if m[2] == "BE"])
        for bfs, (name, values, notes) in matched.items():
            if name in sections:
                notes.append("church rates of the main part taken; other rates in " + ", ".join(sections[name]))
        sources.append(("BE", matched, unmatched))
    for canton, source, unmatched in sources:
        lines, n = compare(canton, source, reference)
        lines += [f"{canton} {name}: name not found in the reference" for name in unmatched]
        print("\n".join(lines))
        differing = sum(1 for line in lines if ": note:" not in line)
        print(f"{canton}: {n['equal']} of {n['reference']} municipalities equal, {differing} differences or gaps")
        failed |= differing > 0
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
