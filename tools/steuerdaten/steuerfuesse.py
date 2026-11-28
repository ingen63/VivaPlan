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
- GE: decree of the cantonal government on the «centimes additionnels» of the year (HTML page of the official
  collection of laws, `rsg_d3_05p30.htm`): municipal surcharge in hundredths of the cantonal base tax; the cantonal
  rate is a single value (`--ge-canton`, default 1.475). No church tax (voluntary in GE).
- VD: table of the municipal tax decrees of the year (Excel of the canton, converted to CSV by hand, e.g. with
  LibreOffice): column «Pour-cent total» in percent of the cantonal base tax; the cantonal coefficient is a single
  value (`--vd-canton`, default 1.55). No church tax.
- FR: open data of the statistical office of the canton (`18_03_coefficients`, CSV with «;»): municipal coefficient
  for income and wealth in percent of the cantonal base tax, with BFS numbers; the latest year up to the year of the
  reference is taken. The cantonal coefficient is a single value (`--fr-canton`, default 0.96 for income). No church
  tax.

The other 21 cantons are described in `quellen/<year>/kantone.json` (source, cantonal multiplier and a pattern for the
rows, see `read_table`) and read by one general reader (`--kantone`, files in `--ordner`). PDFs whose tables the reader
here cannot take apart are converted to text first with `pdf2txt.swift` (PDFKit of macOS).

Values a source does not have (church tax of GE, VD) are not compared. FR: church tax from the parish PDFs of the tax
administration (`--fr-ref`, `--fr-kath`). `--ausnahmen`: values of single municipalities decided after the cantonal
source was published, each with its source (`quellen/<year>/ausnahmen.json`).

Output: one line per difference and a summary per canton. Exit code 1 if a value differs or a municipality is missing.

Usage: python3 -I tools/steuerdaten/steuerfuesse.py REFERENCE.json [--zh CSV] [--be PDF] [--ge HTML] [--vd CSV]  «check-english: allow» (path)
       [--fr CSV] [--zh-canton 0.95] [--ge-canton 1.475] [--vd-canton 1.55] [--fr-canton 0.96]
       [--fr-ref PDF] [--fr-kath PDF] [--ausnahmen JSON]  «check-english: allow» (option names)
       [--kantone JSON --ordner FOLDER [--nur XX]]  «check-english: allow» (option names)
"""

import argparse
import csv
import io
import json
import os
import re
import sys
import unicodedata
import zlib
from html import unescape as html_unescape

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
    rb"|/\S+\s+(?P<tf>-?[\d.]+)\s+Tf"
    rb"|\bBT\b"
)
PIECE = re.compile(rb"\((?:\\.|[^\\)])*\)|-?(?:\d+\.?\d*|\.\d+)")


def unescape(s):
    s = re.sub(rb"\\([0-7]{1,3})", lambda m: bytes([int(m.group(1), 8) & 0xFF]), s)
    s = re.sub(rb"\\(.)", lambda m: {b"n": b"\n", b"r": b"", b"t": b"\t"}.get(m.group(1), m.group(1)), s)
    # The font of the Bern PDF codes the ligatures fl and fi as 0x1F and 0x1D and the thin space as 0x1E.
    return s.decode("cp1252", errors="replace").replace("\x1f", "fl").replace("\x1d", "fi")


EM = 0.5  # estimated width of a character in em (no font metrics without a library)


def text_lines(content):
    """Lines of one page content stream: text pieces grouped by their baseline, left to right. Pieces that follow each
    other closely (estimated end of the previous one, `EM`) are joined directly, so digits or letters set one by one
    form one number or word again; other pieces are separated with « | »."""
    items, x, y, scale, size = [], 0.0, 0.0, 1.0, 1.0
    for m in TOKEN.finditer(content):
        if m.group(0) == b"BT":
            x = y = 0.0
        elif m.group("tf"):
            size = abs(float(m.group("tf"))) or 1.0
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
            items.append((round(y), x, text, size * scale))
    rows = {}
    for yy, xx, text, em in items:
        rows.setdefault(yy, []).append((xx, text, em))
    lines = []
    for _, pieces in sorted(rows.items(), reverse=True):
        line, end = "", None
        for xx, text, em in sorted(pieces, key=lambda p: p[0]):
            close = (end is not None and xx - end < 0.3 * em
                     and (line[-1:].isdigit() and text[:1].isdigit() or line[-1:].isalpha() and text[:1].isalpha()))
            line += text if close or end is None else " | " + text
            end = xx + len(text) * EM * em
        lines.append(line)
    return lines


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
    line = re.sub(r"\s+-\s+", "-", re.sub(r"\s+", " ", line))
    return re.sub(r"(\d) ?\. ?(\d)", r"\1.\2", line)  # «7 . 6» set digit by digit


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


BE_TYPOS = {"Rapperwil": "Rapperswil"}  # misspelt in the PDF 2026


def words(name):
    """Words of a name in lower case without accents, without a part in parentheses («Bargen (BE)») and without a
    leading article («Le Grand-Saconnex» = «Grand-Saconnex»); «St» and «Ste» written out."""
    name = re.sub(r"\(.*?\)", " ", name.lower()).replace("œ", "oe")
    name = "".join(c for c in unicodedata.normalize("NFKD", name) if not unicodedata.combining(c))
    w = [{"st": "saint", "ste": "sainte"}.get(x, x) for x in re.findall(r"[a-z0-9]+", name)]
    while len(w) > 1 and w[0] in ("le", "la", "les", "l"):
        w = w[1:]
    return w


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


def read_ge(path, canton):
    """{name: values} from the decree on the «centimes additionnels» (HTML): «Commune rate» pairs after the
    introduction, rates with a decimal comma; «Ville de Genève» is the municipality Genève. Churches: None."""
    with open(path, encoding="cp1252", errors="replace") as f:
        text = html_unescape(re.sub(r"<[^>]+>", " ", f.read()))
    text = re.sub(r"\s+", " ", text)
    start = text.find("Communes Taux")
    if start < 0:
        raise ValueError("table «Communes Taux» not found in the decree")
    body = text[start + len("Communes Taux"):]
    body = body[:body.find(" RSG ")] if " RSG " in body else body
    out = {}
    for name, rate in re.findall(r"([^\d()]+?)\s+(\d+(?:,\d+)?)(?:\s*\(\d+\))?(?=\s|$)", body):
        name = re.sub(r"^Ville de ", "", name.strip())
        out[name] = [canton, round(float(rate.replace(",", ".")) / 100, 6), None, None, None]
    return out


def read_vd(path, canton):
    """{name: values} from the table of the municipal tax decrees of VD (CSV): rows with the years of adoption and
    validity; column 6 is «Pour-cent total» (percent of the cantonal base tax). Fractions of a municipality with their
    own rate («Le Brassus (fraction)») are left out, as in the ESTV export. Churches: None."""
    out = {}
    with open(path, encoding="utf-8-sig") as f:
        for row in csv.reader(f):
            if (len(row) > 5 and row[0].strip() and "(fraction)" not in row[0]
                    and re.fullmatch(r"\s*\d{4}\s*", row[1] or "")):
                total = row[5].strip()
                if re.fullmatch(r"\d+(\.\d+)?", total):
                    out[row[0].strip()] = [canton, pct(total), None, None, None]
    return out


FR_INCOME = "Einkommens- und Vermoegenssteuer"


def read_fr(path, canton, year):
    """({bfs: (name, values, notes)}, year taken) from the open data of FR: the latest year up to `year`, the
    coefficient for income and wealth. Churches: None."""
    with open(path, encoding="utf-8-sig") as f:
        rows = [r for r in csv.DictReader(f, delimiter=";") if r["coefficients_de"].startswith(FR_INCOME)]
    years = sorted({int(r["annee"]) for r in rows if int(r["annee"]) <= year})
    if not years:
        raise ValueError(f"no year up to {year} in the FR data")
    taken = years[-1]
    notes = [] if taken == year else [f"source year {taken}, not {year}"]
    out = {int(r["commune_id"]): (r["commune"], [canton, pct(r["valeur"]), None, None, None], list(notes))
           for r in rows if int(r["annee"]) == taken}
    return out, taken


GROUPS = ("municipal", "reformed", "catholic", "christ")


def number(text):
    """A published number: decimal comma or point, thousands separators and footnote marks removed."""
    return float(re.sub(r"[\s'’]", "", text).replace(",", "."))


def read_table(path, spec):
    """{name: values} from a PDF (read with `pdf_lines`), an HTML page, a text or an Excel table saved as CSV, by the
    description `spec` of a canton (see `quellen/<year>/kantone.json`):
    - `row`: a regular expression with the groups `name` and `municipal`, optionally `reformed`, `catholic` and
      `christ` (church shares), applied to the whole text (several rows on one line are found as well); `rows`
      instead: a list of such expressions, one per municipality, where the shares stand in different parts of the
      text (AI); a group with the suffix `_add` or `_sub` is added to or subtracted from its share;
    - `scale` turns a published number into a share (0.01 for percent), `scales` per group; `canton` is the cantonal
      multiplier; `fixed`: shares that hold for the whole canton (e.g. one cantonal Kirchgemeinde);
    - `total`: the municipal group is the total of canton and municipality and the church groups are totals with the
      church, so the shares are differences;
    - `start`/`end` cut the text; `csv`/`delimiter` for CSV; `blocks` for a table set column by column (UR);
    - `skip`: names to leave out; `base_name`: sections in parentheses belong to the municipality; `aliases`."""
    if path.lower().endswith(".pdf"):
        text = "\n".join(flat(line) for line in pdf_lines(path))
    elif path.lower().endswith((".html", ".htm")):
        with open(path, encoding=spec.get("encoding", "utf-8"), errors="replace") as f:
            text = re.sub(r"\s+", " ", html_unescape(re.sub(r"<[^>]+>", " ", f.read())))
    else:
        with open(path, encoding=spec.get("encoding", "utf-8-sig"), errors="replace") as f:
            text = f.read()
    if spec.get("start"):
        i = text.find(spec["start"])
        if i < 0:
            raise ValueError(f"{path}: «{spec['start']}» not found")
        text = text[i + len(spec["start"]):]
    if spec.get("end") and spec["end"] in text:
        text = text[:text.find(spec["end"])]
    if spec.get("csv"):
        rows = csv.reader(io.StringIO(text), delimiter=spec.get("delimiter", ","))
        text = "\n".join(" ".join(c.strip() for c in row) for row in rows)
    scale, canton = spec.get("scale", 0.01), spec["canton"]
    if spec.get("blocks"):
        return read_blocks(text, spec)
    out = {}
    matches = [m for row in (spec.get("rows") or [spec["row"]]) for m in re.finditer(row, text, re.M)]
    for m in matches:
        # «Welschenrohr-⏎Gänsbrunnen»: a name broken at the hyphen
        name = re.sub(r"-\s+", "-", re.sub(r"\s+", " ", m.group("name"))).strip(" *-–")
        if spec.get("skip") and re.search(spec["skip"], name):  # e.g. averages of a district
            continue
        if spec.get("base_name"):  # «Haute-Sorne (Glovelier)»: a section of Haute-Sorne
            name = re.sub(r"\s*\(.*?\)", "", name).strip()
        name = spec.get("aliases", {}).get(name, name)
        got = m.groupdict()
        scales = [spec.get("scales", {}).get(g, scale) for g in GROUPS]
        values = [None if got.get(g) is None else round(number(got[g]) * s, 6) for g, s in zip(GROUPS, scales)]
        for i, g in enumerate(GROUPS):  # column added to a share (cantonal church, GR) or subtracted (rebate, NW)
            if got.get(g + "_add") is not None:
                values[i] = round(values[i] + number(got[g + "_add"]) * scales[i], 6)
            if got.get(g + "_sub") is not None:
                values[i] = round(values[i] - number(got[g + "_sub"]) * scales[i], 6)
        if spec.get("total"):
            base = values[0]
            values[0] = round(base - canton, 6)
            values[1:] = [None if v is None else round(v - base, 6) for v in values[1:]]
        for g, v in spec.get("fixed", {}).items():
            values[GROUPS.index(g)] = v
        if name and name not in out:
            out[name] = [canton] + values
        elif name:  # a further row of the same municipality (SG: churches only in the row of the first Kirchgemeinde)
            out[name] = [v if v is not None else w for v, w in zip(out[name], [canton] + values)]
    return out


def read_blocks(text, spec):
    """{name: values} from a table set column by column (UR): a line starts with the names of a block, followed by
    the values of the first name; each further line holds the values of the next name. `column` picks the value
    (-1: the last year); `group` says which share it is. Values like «1)» (footnote instead of a rate) are left
    out."""
    out, names, row = {}, [], 0
    group = GROUPS.index(spec.get("group", "municipal"))
    for line in text.splitlines():
        m = re.match(r"^([^\d%]+?)\s+(?=\d)", line)
        if m and not re.search(r"\d", m.group(1)):
            names, row = m.group(1).split(), 0
            line = line[m.end():]
        cells = re.findall(r"(\d+(?:\.\d+)?)%|\d+\)", line)
        if not names or not cells or row >= len(names):
            continue
        value = cells[spec.get("column", -1)]
        if value:
            values = [spec["canton"], None, None, None, None]
            values[1 + group] = round(number(value) * spec.get("scale", 0.01), 6)
            for g, v in spec.get("fixed", {}).items():
                values[1 + GROUPS.index(g)] = v
            name = spec.get("aliases", {}).get(names[row], names[row])
            out.setdefault(name, values)
        row += 1
    return out


def read_register(path):
    """[[bfs, name, canton]] of the official register of the municipalities of the BFS (snapshot as CSV of
    agvchapp.bfs.admin.ch, «Amtliches Gemeindeverzeichnis»): the rows of level 3, the canton from the parents."""
    with open(path, encoding="utf-8-sig") as f:
        rows = list(csv.DictReader(f))
    by_code = {r["HistoricalCode"]: r for r in rows if r["Level"] != "3"}  # codes repeat across the levels

    def canton(r):
        while r["Level"] != "1":
            r = by_code[r["Parent"]]
        return r["ShortName"]

    return sorted([int(r["BfsCode"]), r["Name"], canton(r)] for r in rows if r["Level"] == "3")


def read_source(canton, spec, folder, ref, year):
    """({bfs: (name, values, notes)}, unmatched names, notes) of one source of a canton (`reader` in the spec: `table`
    by default, or one of the readers of ZH, BE, GE, VD, FR and the FR parishes), names matched with `ref`."""
    path = os.path.join(folder, spec["file"])
    reader = spec.get("reader", "table")
    if reader == "zh":
        return read_zh(path, spec["canton"]), [], []
    if reader == "fr":
        return read_fr(path, spec["canton"], year)[0], [], []
    sections = {}
    if reader == "be":
        rows, quota, sections = read_be(pdf_lines(path), year)
        rows = {BE_TYPOS.get(k, k): v for k, v in rows.items()}
    elif reader == "ge":
        rows = read_ge(path, spec["canton"])
    elif reader == "vd":
        rows = read_vd(path, spec["canton"])
    elif reader == "fr_parishes":
        group = 1 + GROUPS.index(spec["group"])
        rows = {}
        for name, share in read_fr_parishes(path, spec.get("column", 2)).items():
            rows[name] = [spec["canton"], None, None, None, None]
            rows[name][group] = share
    else:
        rows = read_table(path, spec)
    if spec.get("suffix_names"):
        # two columns run together: «Biberist Bolken Balm bei Günsberg 100 100» belongs to the longest end of the
        # text that is the name of a municipality
        known = {keys(m[1])[0] for m in ref}
        renamed = {}
        for name, values in rows.items():
            w = name.split()
            hit = next((" ".join(w[i:]) for i in range(len(w))
                        if (keys(" ".join(w[i:])) or [None])[0] in known), name)
            renamed.setdefault(hit, values)
        rows = renamed
    matched, unmatched = match_names(rows, ref)
    for bfs, (name, values, notes) in matched.items():
        notes.extend(spec.get("notes", []))
        if name in sections:
            notes.append("church rates of the main part taken; other rates in " + ", ".join(sections[name]))
    if spec.get("parts") and unmatched:
        # rows for parts of a municipality (e.g. one row per Kirchgemeinde): not a gap, the whole is compared
        for bfs, (name, values, notes) in matched.items():
            parts = [u for u in unmatched if keys(u)[2] == keys(name)[2]]
            if parts:
                notes.append("further rows for parts: " + ", ".join(parts))
        unmatched = [u for u in unmatched if all(keys(u)[2] != keys(n)[2] for n, _, _ in matched.values())]
    notes = []
    if spec.get("subrows") and unmatched:
        # rows of Kirchgemeinden or parts that are not municipalities of their own
        notes.append("rows not compared (Kirchgemeinden or parts): " + ", ".join(unmatched))
        unmatched = []
    return matched, unmatched, notes


def merge_into(target, source):
    """Puts the shares a further source has (not None) into the rows of `target` ({bfs: values})."""
    for bfs, (_, values, _) in source.items():
        if bfs in target:
            for i in range(1, 5):
                if values[i] is not None:
                    target[bfs][i] = values[i]


def read_canton(canton, spec, folder, ref, year, exceptions=None):
    """(income {bfs: (name, values, notes)}, wealth {bfs: values}, unmatched, notes) of a canton: the main source,
    further sources for single shares (`also`, e.g. the church tax in a list of its own) and the wealth tax
    multipliers (`wealth`: the same as for income unless it names another cantonal multiplier `canton`, further
    sources `also` or `fixed` shares)."""
    matched, unmatched, notes = read_source(canton, spec, folder, ref, year)
    # exceptions first, so that further sources (church lists) also reach a municipality the main source lacks
    apply_exceptions(canton, matched, {canton: (exceptions or {}).get(canton, {})}, ref, spec.get("canton"))
    income = {bfs: values for bfs, (_, values, _) in matched.items()}
    for values in income.values():  # the readers of single cantons do not know `fixed`
        for g, v in spec.get("fixed", {}).items():
            values[1 + GROUPS.index(g)] = v
    for extra in spec.get("also", []):
        merge_into(income, read_source(canton, dict(extra, canton=spec.get("canton")), folder, ref, year)[0])
    wealth = {bfs: list(values) for bfs, values in income.items()}
    w = spec.get("wealth", {})
    for values in wealth.values():
        if "canton" in w:
            values[0] = w["canton"]
        for g, v in w.get("fixed", {}).items():
            values[1 + GROUPS.index(g)] = v
    for extra in w.get("also", []):
        merge_into(wealth, read_source(canton, dict(extra, canton=w.get("canton", spec.get("canton"))), folder,
                                       ref, year)[0])
    for bfs, (values, _) in (exceptions or {}).get(canton, {}).items():  # exceptions win over further sources
        for target in (income.get(bfs), wealth.get(bfs)):
            if target is not None:
                for g, v in values.items():
                    target[1 + GROUPS.index(g)] = v
    return matched, wealth, unmatched, notes


def read_specs(path):
    with open(path, encoding="utf-8") as f:
        return {c: s for c, s in json.load(f).items() if not c.startswith("_")}


def collect(specs, folder, register, year, exceptions=None, only=None):
    """[(canton, income {bfs: (name, values, notes)}, wealth {bfs: values}, unmatched, notes)] of all cantons
    described in `specs` whose source file lies in `folder`, with the exceptions put in."""
    out = []
    for canton, spec in sorted(specs.items()):
        if (only and canton not in only) or not os.path.exists(os.path.join(folder, spec["file"])):
            continue
        ref = [m for m in register if m[2] == canton]
        income, wealth, unmatched, notes = read_canton(canton, spec, folder, ref, year, exceptions)
        out.append((canton, income, wealth, unmatched, notes))
    return out


def multipliers(collected, register):
    """(municipalities [bfs, name, canton, income, wealth] of the register sorted by canton and name, report): every
    municipality of the register needs a source row; a church share no source has counts as 0 and is reported."""
    rows = {}
    for canton, income, wealth, _, _ in collected:
        for bfs, (_, values, notes) in income.items():
            rows[bfs] = (values, wealth.get(bfs, values), notes)
    out, report = [], []
    for bfs, name, canton in register:
        if bfs not in rows:
            report.append(f"{canton} {bfs} {name}: no tax multipliers in the sources")
            continue
        values, wealth, notes = rows[bfs]
        if values[1] is None:
            report.append(f"{canton} {bfs} {name}: no municipal tax multiplier in the sources")
            continue
        missing = [CHURCHES[i - 2] for i in range(2, 5) if values[i] is None or wealth[i] is None]
        if missing:
            report.append(f"{canton} {bfs} {name}: no rate in the sources for {', '.join(missing)}, 0 taken")
        clean = lambda v: [0.0 if x is None else round(x, 6) for x in v]  # noqa: E731
        out.append([bfs, name, canton, clean(values), clean(wealth)])
    out.sort(key=lambda m: (m[2], m[1]))
    return out, report


def read_fr_parishes(path, column=2):
    """{commune: share} from a PDF of the FR tax administration on the parish tax rates (reformed or Catholic): rows
    «commune | parish | income | wealth | profit | capital» in percent of the cantonal base tax; the income rate of
    the first parish of a commune (a commune with several parishes: the first listed); `column` 3 gives the wealth."""
    out = {}
    for line in pdf_lines(path):
        parts = [p.strip() for p in line.split("|") if p.strip()]
        first = next((i for i, p in enumerate(parts) if re.fullmatch(r"\d+(\.\d+)?", p)), None)
        # the parish name is missing where it is the commune itself: the rates then start one place earlier
        if first in (1, 2) and len(parts) >= first + 4 and not re.search(r"\d", parts[0]):
            out.setdefault(parts[0], round(float(parts[first + column - 2]) / 100, 6))
    # «Schmitten | 10.0 …» below «Schmitten (FR) | St. Antoni | …» is a further parish of that commune
    return {k: v for k, v in out.items()
            if not any(o != k and "(" in o and keys(o)[0] == keys(k)[0] for o in out)}


def read_exceptions(path):
    """{canton: {bfs: (values by group, source)}} from the JSON file of exceptions: values of single municipalities
    that the cantonal source does not have yet (decisions after its publication), each with its source."""
    with open(path, encoding="utf-8") as f:
        data = json.load(f)
    return {canton: {int(bfs): ({g: e[g] for g in GROUPS if g in e}, e["source"]) for bfs, e in rows.items()}
            for canton, rows in data.items() if not canton.startswith("_")}


def apply_exceptions(canton, source, exceptions, register, cantonal=None):
    """Puts the exceptions of a canton into its source rows (also municipalities the source lacks, then with the
    cantonal multiplier `cantonal` or that of another row), with a note."""
    for bfs, (values, origin) in exceptions.get(canton, {}).items():
        reg = next((m for m in register if m[0] == bfs), None)
        if bfs not in source and reg is None:
            continue
        if cantonal is None:
            cantonal = next(iter(source.values()))[1][0] if source else None
        name, row, notes = source.get(bfs, (reg[1], [cantonal, None, None, None, None], []))
        row = list(row)
        for g, v in values.items():
            row[1 + GROUPS.index(g)] = v
        source[bfs] = (name, row, notes + [f"exception: {origin}"])


LABELS = ("canton", "municipal") + CHURCHES


def compare(canton, source, reference, column=3, label="", known=()):
    """Differences of the income (`column` 3) or wealth (4) multipliers of one canton; returns (lines, counts).
    Shares in `known` (labels such as «reformed») differ on purpose from the reference and are not compared."""
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
        expected = ref[bfs][column]
        diffs = [f"{label}{LABELS[i]} {values[i]:g} (reference {expected[i]:g})"
                 for i in range(5) if values[i] is not None and LABELS[i] not in known
                 and abs(values[i] - expected[i]) > TOL]
        if diffs:
            lines.append(f"{canton} {bfs} {name}: " + ", ".join(diffs))
        else:
            equal += 1
        lines.extend(f"{canton} {bfs} {name}: note: {n}" for n in notes)
    return lines, {"reference": len(ref), "source": len(source), "equal": equal}


def read_reference(path):
    """(municipalities [bfs, name, canton, income, wealth], year) of a tax data file (JSON) or of the Excel export
    «Steuerfüsse» of the ESTV tax calculator (xlsx, only as a local cross-check, #283)."""
    if path.lower().endswith(".xlsx"):
        sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
        import build  # noqa: E402 (next to this file)
        year, munis = build.read_multipliers(path)
        return munis, year
    with open(path, encoding="utf-8") as f:
        data = json.load(f)
    return data["municipalities"], data.get("year", 2026)


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("reference", help="tax data file (JSON) or ESTV export (xlsx) to compare with")
    ap.add_argument("--kantone", help="JSON file describing the sources of the cantons")
    ap.add_argument("--ordner", default=".", help="folder with the source files named in --kantone")
    ap.add_argument("--gemeinden", help="BFS register of the municipalities (CSV); default: those of the reference")
    ap.add_argument("--ausnahmen", help="JSON file with exceptions per municipality")
    ap.add_argument("--nur", action="append", help="only these cantons")
    ap.add_argument("--zh")
    ap.add_argument("--be")
    ap.add_argument("--ge")
    ap.add_argument("--vd")
    ap.add_argument("--fr")
    ap.add_argument("--fr-ref", help="PDF of the FR reformed parish tax rates")
    ap.add_argument("--fr-kath", help="PDF of the FR Catholic parish tax rates")
    ap.add_argument("--zh-canton", type=float, default=0.95)
    ap.add_argument("--ge-canton", type=float, default=1.475)
    ap.add_argument("--vd-canton", type=float, default=1.55)
    ap.add_argument("--fr-canton", type=float, default=0.96)
    a = ap.parse_args(argv)
    reference, year = read_reference(a.reference)
    register = read_register(a.gemeinden) if a.gemeinden else [m[:3] for m in reference]
    specs = read_specs(a.kantone) if a.kantone else {}
    # single files given on the command line (instead of a description file)
    for canton, path, value in (("ZH", a.zh, a.zh_canton), ("BE", a.be, None), ("GE", a.ge, a.ge_canton),
                                ("VD", a.vd, a.vd_canton), ("FR", a.fr, a.fr_canton)):
        if path:
            spec = {"reader": canton.lower(), "file": os.path.abspath(path), "canton": value}
            if canton == "FR":
                spec["also"] = [{"reader": "fr_parishes", "file": os.path.abspath(f), "group": g}
                                for f, g in ((a.fr_ref, "reformed"), (a.fr_kath, "catholic")) if f]
            specs[canton] = spec
    exceptions = read_exceptions(a.ausnahmen) if a.ausnahmen else {}
    failed = False
    for canton, income, wealth, unmatched, notes in collect(specs, a.ordner, register, year, exceptions, a.nur):
        known = specs[canton].get("reference_differs", {})
        if known:
            notes = notes + [f"not compared with the reference: {', '.join(known['shares'])} ({known['reason']})"]
        known = tuple(known.get("shares", ()))
        lines, n = compare(canton, income, reference, known=known)
        wealth_rows = {bfs: (income[bfs][0], values, []) for bfs, values in wealth.items() if bfs in income}
        w_lines, _ = compare(canton, wealth_rows, reference, column=4, label="wealth ", known=known)
        lines += [x for x in w_lines if "wealth " in x]
        lines += [f"{canton} {name}: name not found in the reference" for name in unmatched]
        lines += [f"{canton}: note: {note}" for note in notes]
        print("\n".join(lines))
        differing = sum(1 for line in lines if ": note:" not in line)
        print(f"{canton}: {n['equal']} of {n['reference']} municipalities equal, {differing} differences or gaps")
        failed |= differing > 0
    return 1 if failed else 0


if __name__ == "__main__":
    sys.exit(main())
