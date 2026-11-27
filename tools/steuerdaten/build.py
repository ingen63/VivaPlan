#!/usr/bin/env python3
"""#63: Builds the tax data file for the mode «Wohnort» from the ESTV base data.

Inputs (ESTV tax calculator, «Grunddaten abrufen», one tax year):
  --steuerfuesse  Excel export «Steuerfüsse», whole of Switzerland (xlsx)
  --tarife        tariffs for income and wealth (CSV, columns see README.md)
  --abzuege       deductions for income and wealth, whole of Switzerland (CSV)
  --kapitalregeln rules of the cantons for lump sums from pension savings (JSON, default kapitalregeln.json, #64)
Output: JSON with the income tariffs (on the taxable income), the standard deduction rules, the wealth curves (social
deductions included), the curves for lump sums (#64) per canton and for the federal tax, and the tax multipliers per
municipality. #65: the rule sets of the property gain tax per canton come from `grundstueckgewinn.py`.

Standard library only; the xlsx is read directly from the ZIP. Usage and procedure: README.md.
"""

import argparse
import csv
import math
import json
import os
import re
import statistics
import sys
import xml.etree.ElementTree as ET
import zipfile
from collections import defaultdict

NAMES = {
    "ZH": "Zürich", "BE": "Bern", "LU": "Luzern", "UR": "Uri", "SZ": "Schwyz", "OW": "Obwalden",
    "NW": "Nidwalden", "GL": "Glarus", "ZG": "Zug", "FR": "Freiburg", "SO": "Solothurn",
    "BS": "Basel-Stadt", "BL": "Basel-Landschaft", "SH": "Schaffhausen", "AR": "Appenzell Ausserrhoden",
    "AI": "Appenzell Innerrhoden", "SG": "St. Gallen", "GR": "Graubünden", "AG": "Aargau", "TG": "Thurgau",
    "TI": "Tessin", "VD": "Waadt", "VS": "Wallis", "NE": "Neuenburg", "GE": "Genf", "JU": "Jura",
}

# Grid points of the curves (francs). Between the points the engine interpolates the rate linearly, above the last
# point the last rate applies.
INCOME_GRID = (
    list(range(0, 100_000, 2_500))
    + list(range(100_000, 300_000, 10_000))
    + list(range(300_000, 1_000_000, 50_000))
    + [1_000_000, 1_500_000, 2_000_000, 3_000_000, 5_000_000, 10_000_000]
)
WEALTH_GRID = (
    list(range(0, 500_000, 25_000))
    + list(range(500_000, 3_000_000, 100_000))
    + list(range(3_000_000, 10_000_000, 500_000))
    + [10_000_000, 20_000_000, 50_000_000, 100_000_000]
)
# #64: grid points of the curves for lump sums from pension savings.
CAPITAL_GRID = (
    list(range(0, 200_000, 10_000))
    + list(range(200_000, 1_000_000, 25_000))
    + list(range(1_000_000, 3_000_000, 100_000))
    + list(range(3_000_000, 10_000_000, 500_000))
    + [10_000_000, 20_000_000, 50_000_000]
)
RULES = os.path.join(os.path.dirname(os.path.abspath(__file__)), "kapitalregeln.json")
# #287: source files from the laws per canton and tax year (`<canton>_tarife.csv`, `<canton>_abzuege.csv`); they
# replace the ESTV rows of their canton (`CH` = federal).
SOURCES = os.path.join(os.path.dirname(os.path.abspath(__file__)), "quellen")
# #292: cantons whose tariffs and deductions may still come from the ESTV tables (Valais until #296); the ESTV rows
# of all other cantons are ignored.
ESTV_ONLY = {"VS"}

# Standard deductions from the income, without children: per marital status and variant a list of groups; per group
# the first deduction found applies. Variant «retired»: insurance premiums and savings interest without contributions
# to the 2nd and 3rd pillar (the next available variant otherwise); variant «working»: with contributions.
_SOCIAL_SINGLE = [["SozLedig_EK"], ["SozAlleinEigenemHaushalt_EK"]]
_SOCIAL_MARRIED = [["SozVerheiratet_EK"], ["SozZusAbzug_EK"]]
INCOME_DEDUCTIONS = {
    "single": {
        "retired": [
            ["KKSparLedigOhneBVGS3a_EK", "KKSparLedigMitBVGS3a_EK", "PrivVersSparLedigMitBVGS3a_EK",
             "KKPrivVersLedig_EK", "KKLedig_EK"],
            ["PrivVersLedig_EK"],
            ["SparzinsenLedig_EK"],
        ] + _SOCIAL_SINGLE,
        "working": [
            ["KKSparLedigMitBVGS3a_EK", "KKSparLedigOhneBVGS3a_EK", "PrivVersSparLedigMitBVGS3a_EK",
             "KKPrivVersLedig_EK", "KKLedig_EK"],
            ["PrivVersLedig_EK"],
            ["SparzinsenLedig_EK"],
        ] + _SOCIAL_SINGLE,
    },
    "married": {
        "retired": [
            ["KKSparVerhOhneBVGS3a_EK", "KKSparzVerhBeideOhneBVGS3a_EK", "KKSparzVerhMitBVGS3a_EK",
             "PrivVersSparzVerhMitBVGS3a_EK", "KKPrivVersVerheiratet_EK", "KKVerheiratet_EK"],
            ["PrivVersVerheiratet_EK"],
            ["SparzinsenVerheiratet_EK"],
        ] + _SOCIAL_MARRIED,
        "working": [
            ["KKSparzVerhMitBVGS3a_EK", "KKSparzVerhBeideMitBVGS3a_EK", "KKSparVerhOhneBVGS3a_EK",
             "PrivVersSparzVerhMitBVGS3a_EK", "KKPrivVersVerheiratet_EK", "KKVerheiratet_EK"],
            ["PrivVersVerheiratet_EK"],
            ["SparzinsenVerheiratet_EK"],
        ] + _SOCIAL_MARRIED,
    },
}
WEALTH_DEDUCTIONS = {
    "single": [["SozLedigOhneKinder_VM", "SozLedigMitOhneKinder_VM"]],
    "married": [["SozVerheiratet_VM"]],
}
# Flat deduction for other professional expenses on the earned income
EARNED_DEDUCTION = "HauptErw_EK"
# Second-earner deduction for married couples with two earned incomes. Reference: the lower earned income after the
# flat deduction for professional expenses; Bern: the total earned income (2 % of it). Derived from the ESTV tax
# calculator 2026 (ZH, BE, LU, AG, SG, AR, GL, NE and federal, 23.11.2026).
SECOND_EARNER_DEDUCTION = "ZweitVerdiener_EK"
SECOND_EARNER_TOTAL = {"BE"}

GAIN_SOURCE = "Steuergesetze der Kantone (Stand 2025/2026), Artikel je Kanton im Bericht (#287–#291)"
SOURCE = ("Steuergesetze und Verordnungen von Bund und Kantonen (#292); Steuerfüsse und Wallis: ESTV, Steuerrechner, "
          "Grunddaten")
NOTE = ("Schätzung für die Planung mit den Standardabzügen des Kantons und des Bundes für Alleinstehende oder "
        "Verheiratete ohne Kinder, ohne individuelle Abzüge.")


# ---------------------------------------------------------------- inputs


def read_xlsx(path):
    """First sheet as a list of rows (dict column -> text)."""
    ns = {"m": "http://schemas.openxmlformats.org/spreadsheetml/2006/main"}
    tag = "{%s}" % ns["m"]
    z = zipfile.ZipFile(path)
    strings = []
    if "xl/sharedStrings.xml" in z.namelist():
        for si in ET.fromstring(z.read("xl/sharedStrings.xml")).findall("m:si", ns):
            strings.append("".join(t.text or "" for t in si.iter(tag + "t")))
    sheet = sorted(n for n in z.namelist() if n.startswith("xl/worksheets/sheet"))[0]
    out = []
    for r in ET.fromstring(z.read(sheet)).iter(tag + "row"):
        row = {}
        for c in r.findall("m:c", ns):
            col = re.match(r"[A-Z]+", c.get("r")).group()
            v, t = c.find("m:v", ns), c.get("t")
            if t == "inlineStr":
                row[col] = "".join(x.text or "" for x in c.iter(tag + "t"))
            elif v is None:
                row[col] = None
            elif t == "s":
                row[col] = strings[int(v.text)]
            else:
                row[col] = v.text
        out.append(row)
    return out


def read_multipliers(path):
    """Tax multipliers per municipality. Columns: A canton id, B canton, C BFS id, D municipality, E–I income (canton,
    municipality, church reformed, Roman Catholic, Christ Catholic), J–N wealth (same)."""
    rows = read_xlsx(path)
    year = None
    for r in rows[:3]:
        a = r.get("A")
        if a and re.fullmatch(r"\d{4}(\.0)?", a):
            year = int(float(a))
    head = next(i for i, r in enumerate(rows) if r.get("C") == "BfS-Id")
    num = lambda v: round(float(v or 0) / 100, 6)  # noqa: E731
    munis = []
    for r in rows[head + 1:]:
        if not r.get("B") or not r.get("C"):
            continue
        munis.append([
            int(float(r["C"])), r["D"], r["B"],
            [num(r.get(c)) for c in "EFGHI"],
            [num(r.get(c)) for c in "JKLMN"],
        ])
    munis.sort(key=lambda m: (m[2], m[1]))
    return year, munis


def read_csv(path):
    with open(path, encoding="utf-8") as f:
        return list(csv.DictReader(f))


def read_sources(directory):
    """#287: Tariff and deduction rows from the source files of a directory (columns as the ESTV tables plus
    `Quelle`). Returns the tariff rows, the deduction rows and the cantons covered. Every row needs a source and must
    belong to the canton of its file name; a canton needs both files (its ESTV rows are dropped as a whole). A missing
    directory covers no canton."""
    tariffs, deductions_, files = [], [], defaultdict(set)
    if not directory or not os.path.isdir(directory):
        return tariffs, deductions_, set()
    for name in sorted(os.listdir(directory)):
        m = re.fullmatch(r"([A-Z]{2})_(tarife|abzuege)\.csv", name)
        if not m:
            continue
        canton, kind = m.groups()
        if canton != "CH" and canton not in NAMES:
            raise ValueError(f"{name}: unknown canton {canton}")
        rows = read_csv(os.path.join(directory, name))
        for i, r in enumerate(rows, start=2):
            if r.get("Kanton") != canton:
                raise ValueError(f"{name}, line {i}: canton {r.get('Kanton')} instead of {canton}")
            if not (r.get("Quelle") or "").strip():
                raise ValueError(f"{name}, line {i}: source missing")
        (tariffs if kind == "tarife" else deductions_).extend(rows)
        files[canton].add(kind)
    for canton, kinds in files.items():
        if kinds != {"tarife", "abzuege"}:
            missing = ({"tarife", "abzuege"} - kinds).pop()
            raise ValueError(f"{canton}: source file {canton}_{missing}.csv missing")
    return tariffs, deductions_, set(files)


def merge_sources(estv_rows, source_rows, covered, estv_only=None):
    """#287: The ESTV rows of the cantons not covered by a source file, then the rows of the source files. #292: with
    `estv_only`, ESTV rows are kept only for these cantons (the others must come from source files)."""
    keep = [r for r in estv_rows if r["Kanton"] not in covered]
    if estv_only is not None:
        keep = [r for r in keep if r["Kanton"] in estv_only]
    return keep + source_rows


def check_coverage(covered, estv_only):
    """#292: Every authority (`CH` and the 26 cantons) except those in `estv_only` needs source files."""
    missing = sorted(({"CH"} | set(NAMES)) - set(covered) - set(estv_only))
    if missing:
        raise ValueError(f"source files missing for {', '.join(missing)} (only {', '.join(sorted(estv_only))} may "
                         "come from the ESTV tables)")


def origin_report(covered):
    """#287: One report line per authority: data from the source files (laws) or from the ESTV tables."""
    return [f"{cid} origin: {'laws (source files)' if cid in covered else 'ESTV tables'}"
            for cid in ["CH"] + sorted(NAMES)]


# ---------------------------------------------------------------- tariffs


def evaluate_formula(formula, x):
    expr = formula.replace("$wert$", "x")
    expr = re.sub(r"\blog\s+x\b", "log(x)", expr)
    if not re.fullmatch(r"[\d\s.+\-*/()xlog]+", expr):
        raise ValueError(f"unknown formula: {formula}")
    return eval(expr, {"__builtins__": {}}, {"x": x, "log": math.log})  # noqa: S307 (numbers, x and log only)


class Tariff:
    """An ESTV tariff: type (ZUERICH, BUND, FLATTAX, FREIBURG, FORMEL), splitting and rows."""

    def __init__(self, kind, splitting, rows):
        self.kind = kind
        self.splitting = splitting
        self.rows = rows  # [(amount, percent, tax, formula)]

    def simple(self, x):
        """Simple tax (tax multiplier 100 %) on the amount x, without splitting."""
        if x <= 0:
            return 0.0
        rows = self.rows
        if self.kind == "ZUERICH":  # brackets «for the next CHF / additional %»
            tax, rest = 0.0, x
            for width, pct, _, _ in rows:
                part = min(rest, width)
                tax += part * pct / 100
                rest -= part
                if rest <= 0:
                    break
            return tax
        if self.kind == "BUND":  # base amount from the bracket plus marginal rate
            row = None
            for r in rows:
                if r[0] <= x:
                    row = r
            if row is None:
                return 0.0
            return row[2] + (x - row[0]) * row[1] / 100
        if self.kind == "FLATTAX":
            return x * rows[0][1] / 100
        if self.kind == "FREIBURG":  # average rate, linear between the rows
            pts = [(r[0], r[1]) for r in rows]
            if x >= pts[-1][0]:
                return x * pts[-1][1] / 100
            for (x0, r0), (x1, r1) in zip(pts, pts[1:]):
                if x0 <= x < x1:
                    rate = r0 if x1 == x0 else r0 + (r1 - r0) * (x - x0) / (x1 - x0)
                    return x * rate / 100
            return 0.0
        if self.kind == "FORMEL":
            row = None
            for r in rows:
                if r[0] <= x:
                    row = r
            if row is None or not row[3]:
                return 0.0
            return max(0.0, evaluate_formula(row[3], x))
        raise ValueError(f"unknown tariff type {self.kind}")

    def tax(self, x, married):
        """Tax with splitting for married couples (rate of the divided amount on the whole amount)."""
        s = self.splitting if married and self.splitting > 0 else 1.0
        if x <= 0:
            return 0.0
        if s == 1.0:
            return self.simple(x)
        return self.simple(x / s) / (x / s) * x


def load_tariffs(rows):
    groups = defaultdict(list)
    for r in rows:
        key = (r["Kanton"], r["Hoheit"], r["Steuerart"], r["Tariftyp"], float(r["Splitting"]), r["Gruppe"])
        groups[key].append((int(r["Zeile"]), float(r["Betrag"]), float(r["Prozent"]), float(r["Steuer"]), r["Formel"]))
    out = defaultdict(list)  # (canton, authority, kind of tax) -> [(group, Tariff)]
    for (canton, target, kind, ttype, split, group), lines in groups.items():
        lines.sort()
        rows = [line[1:] for line in lines]
        if ttype == "ZUERICH" and any(r[2] for r in rows):
            ttype = "BUND"  # brackets with a base amount
        out[(canton, target, kind)].append((group, Tariff(ttype, split, rows)))
    return out


def pick(tariffs, married):
    """Tariff for single persons without children or for married couples from the ESTV groups."""
    if not tariffs:
        return None
    for group, t in tariffs:
        if group == "ALLE":
            return t
    groups = [(set(g.split(",")), t) for g, t in tariffs]
    if married:
        for g, t in groups:
            if "VERHEIRATET" in g:
                return t
    else:
        for want in ("LEDIG_OHNE_KINDER", "LEDIG_KONKUBINAT"):
            for g, t in groups:
                if want in g and "VERHEIRATET" not in g:
                    return t
    raise ValueError(f"no group for {'married' if married else 'single'}: {[g for g, _ in tariffs]}")


# ---------------------------------------------------------------- deductions


def rule_of(row):
    """Deduction as a rule `{base, rate, min, max}`: amount = base + rate × reference, at least min, at most max
    (0 = no maximum). The reference is the income (general deductions) or the earned income (professional expenses).
    A percentage with the format STANDARDIZED takes the amount of the ESTV data as the minimum. Returns the rule and
    whether it is an approximation (only a maximum is known; it applies as a fixed deduction)."""
    fmt = set(row["Format"].split(","))
    amount, pct = float(row["Betrag"]), float(row["Prozent"]) / 100
    lo, hi = float(row["Minimum"]), float(row["Maximum"])
    if "PERCENT" in fmt:
        if "STANDARDIZED" in fmt:
            lo = amount
        return {"base": 0, "rate": pct, "min": lo if ("MINIMUM" in fmt or "STANDARDIZED" in fmt) else 0,
                "max": hi if "MAXIMUM" in fmt else 0}, False
    if "STANDARDIZED" in fmt:
        return {"base": amount, "rate": 0, "min": 0, "max": 0}, False
    if "MAXIMUM" in fmt:
        return {"base": hi, "rate": 0, "min": 0, "max": 0}, False
    if "MINIMUM" in fmt:
        return {"base": lo, "rate": 0, "min": 0, "max": 0}, False
    return {"base": amount, "rate": 0, "min": 0, "max": 0}, False


def rule_amount(rule, reference):
    """Amount of a rule for a reference amount (same formula as the engine, index 1)."""
    v = max(rule["base"] + rule["rate"] * reference, rule["min"])
    return min(v, rule["max"]) if rule["max"] > 0 else v


def kind_of(ident):
    """Kind of a deduction for the display (#131): social deduction, savings interest or insurance premiums."""
    if ident.startswith("Soz"):
        return "social"
    if ident.startswith("Sparzinsen"):
        return "savingsInterest"
    return "insurance"


def deduction_rules(table, spec):
    """Rules of the deductions found for a specification (list of groups of alternatives)."""
    out = []
    for alternatives in spec:
        for ident in alternatives:
            if ident in table:
                rule, _ = rule_of(table[ident])
                out.append({"id": ident, "kind": kind_of(ident), **rule})
                break
    return out


def earned_rule(row):
    """Flat deduction «other professional expenses» as `{base, rate, min, max}` and the flag `approx` if the ESTV
    data only name a maximum (taken as a fixed deduction of that amount)."""
    rule, _ = rule_of(row)
    fmt = set(row["Format"].split(","))
    if "PERCENT" not in fmt and "MAXIMUM" in fmt:
        hi = float(row["Maximum"])
        return {"base": hi, "rate": 0, "min": 0, "max": hi, "approx": True}
    return rule


def deductions(table, label, report):
    """Income deduction rules of one authority: per marital status and variant, and the professional expenses."""
    out = {}
    for status, variants in INCOME_DEDUCTIONS.items():
        out[status] = {variant: deduction_rules(table, spec) for variant, spec in variants.items()}
        for variant, rules in out[status].items():
            report.append(f"{label} {status} {variant}: {[r['id'] for r in rules]}")
    row = table.get(EARNED_DEDUCTION)
    if row is None:
        raise ValueError(f"{label}: flat deduction for professional expenses missing")
    out["earned"] = earned_rule(row)
    report.append(f"{label} earned: {out['earned']}")
    second = second_earner_rule(table.get(SECOND_EARNER_DEDUCTION), label)
    if second is not None:
        out["secondEarner"] = second
    report.append(f"{label} second earner: {second}")
    return out


def second_earner_rule(row, label):
    """Second-earner deduction as `{base, rate, min, max, reference}`; `reference` is `lower` (lower earned income
    after the professional expenses) or `total` (sum of the earned incomes). The deduction is at most the lower earned
    income after the professional expenses. None if the ESTV data have no such deduction."""
    if row is None:
        return None
    rule, _ = rule_of(row)
    return {**rule, "reference": "total" if label in SECOND_EARNER_TOTAL else "lower"}


def total_deduction(table, spec, base):
    """Sum of the deductions of a specification for the amount `base` (wealth curves)."""
    return sum(rule_amount(r, base) for r in deduction_rules(table, spec))


# ---------------------------------------------------------------- curves


def tariff_curve(tariff, married, grid):
    """Rate of the simple tax (tax multiplier 100 %) on the taxable amount, per grid point."""
    return [[x, round(tariff.tax(x, married) / x, 7) if x > 0 else 0.0] for x in grid]


def wealth_curve(tariff, married, table, spec, grid):
    """Rate of the simple tax on the wealth before the social deductions, per grid point."""
    pts = []
    for x in grid:
        if x <= 0:
            pts.append([0, 0.0])
            continue
        tax = tariff.tax(max(0.0, x - total_deduction(table, spec, x)), married)
        pts.append([x, round(tax / x, 7)])
    return pts


# ---------------------------------------------------------------- lump sums (#64)


def capital_rule(rule, married):
    """Rule with the values for the marital status (`single`, `married` override)."""
    r = {k: v for k, v in rule.items() if k not in ("single", "married", "municipal")}
    r.update(rule.get("married" if married else "single", {}))
    return r


def capital_tax(rule, tariffs, canton, married, x):
    """Simple tax (tax multiplier 100 %) on a lump sum from pension savings by the rule of the canton.

    Rule `table`: tariff «Kapitalauszahlung aus Vorsorge» of the ESTV. Rule `income`: share `share` of the tax by the
    income tariff, rate determined for `x / divisor` (without deductions, married couples with the splitting of the
    tariff); with `parts` (#288, Zug) instead the sum over the parts of the tariff tax on each part of the amount
    (`upTo`: upper limit of the part, missing for the last one) times its `share`. The rate lies between `min` and
    `max` if given; `rebate` reduces by a share of the amount, at most `max`.
    """
    if x <= 0:
        return 0.0
    r = capital_rule(rule, married)
    target = r.get("target", "KANTON")
    if r["kind"] == "table":
        t = pick(tariffs.get((canton, target, "VORSORGESTEUER")) or tariffs.get((canton, "KANTON", "VORSORGESTEUER")),
                 married)
        if t is None:
            raise ValueError(f"{canton}: tariff for lump sums missing")
        rate = t.tax(x, married) / x * r.get("share", 1.0)
    elif r["kind"] == "income" and r.get("parts"):
        t = pick(tariffs.get((canton, target, "EINKOMMENSSTEUER")), married)
        tax, lower = 0.0, 0.0
        for part in r["parts"]:
            upper = min(x, part.get("upTo", x))
            if upper > lower:
                tax += (t.tax(upper, married) - t.tax(lower, married)) * part["share"]
            lower = max(lower, upper)
        rate = tax / x
    elif r["kind"] == "income":
        t = pick(tariffs.get((canton, target, "EINKOMMENSSTEUER")), married)
        d = r.get("divisor", 1)
        rate = t.tax(x / d, married) / (x / d) * r.get("share", 1.0)
    else:
        raise ValueError(f"{canton}: unknown rule {r['kind']}")
    if r.get("min") is not None:
        rate = max(rate, r["min"])
    if r.get("max") is not None:
        rate = min(rate, r["max"])
    tax = x * rate
    if r.get("rebate"):
        tax -= min(x * r["rebate"]["rate"], r["rebate"].get("max", float("inf")))
    return max(tax, 0.0)


def capital_curves(rule, tariffs, canton, scale=1.0):
    """Rate of the simple tax on the whole lump sum per grid point, for single persons and married couples."""
    return {status: [[x, round(capital_tax(rule, tariffs, canton, married, x) * scale / x, 7) if x > 0 else 0.0]
                     for x in CAPITAL_GRID]
            for status, married in (("single", False), ("married", True))}


def capital(c, rule, tariffs, munis, report):
    """Adds the lump-sum fields of a canton: `capital`, if needed `capitalMunicipal`, `capitalRule`, `capitalApprox`."""
    cid = c["id"]
    scale = 1.0
    if "cantonMultiplier" in rule:
        # cantonal tax with its own multiplier: scale the curve so that curve × cantonal multiplier fits
        kts = {m[3][0] for m in munis if m[2] == cid}
        if len(kts) != 1:
            raise ValueError(f"{cid}: cantonMultiplier needs a uniform cantonal tax multiplier")
        scale = rule["cantonMultiplier"] / kts.pop()
    c["capital"] = capital_curves(rule, tariffs, cid, scale)
    if "municipal" in rule or scale != 1.0:
        c["capitalMunicipal"] = capital_curves(rule.get("municipal", rule), tariffs, cid)
    c["capitalRule"] = rule["text"]
    if rule.get("approx"):
        c["capitalApprox"] = rule["approx"]
    report.append(f"{cid} capital: {rule['text']}" + (f" ({rule['source']})" if rule.get("source") else "")
                  + (f" (approx: {rule['approx']})" if rule.get("approx") else ""))


def property_gain(c, rule, munis, report):
    """#65: Adds the property gain fields of a canton: `propertyGain` (the rule for the engine), `propertyGainRule`,
    `propertyGainApprox` if the rule is simplified, and `gainMultiplier`, the median tax multiplier of canton and
    municipality (income tax) for properties outside the canton of residence."""
    cid = c["id"]
    rule = dict(rule)
    c["propertyGainRule"] = rule.pop("text")
    source = rule.pop("source")
    approx = rule.pop("approx", None)
    if approx:
        c["propertyGainApprox"] = approx
    c["propertyGain"] = rule
    rates = [m[3][0] + m[3][1] for m in munis if m[2] == cid]
    if not rates:
        raise ValueError(f"canton {cid}: no municipality for the tax multiplier of the property gain tax")
    c["gainMultiplier"] = round(statistics.median(rates), 4)
    note = f" (approx: {approx})" if approx else ""
    report.append(f"{cid} property gain: {c['propertyGainRule']} ({source}){note}")


def read_rules(path):
    """Capital rules per canton; keys starting with `_` are comments."""
    with open(path, encoding="utf-8") as f:
        return {k: v for k, v in json.load(f).items() if not k.startswith("_")}


def build(tariff_rows, deduction_rows, year, munis, capital_rules=None, gain_rules=None):
    tariffs = load_tariffs(tariff_rows)
    deds = defaultdict(dict)
    for r in deduction_rows:
        deds[(r["Kanton"], r["Hoheit"], r["Steuerart"])][r["ID"]] = r
    report = []

    def income(canton, target):
        res = {}
        for status, married in (("single", False), ("married", True)):
            t = pick(tariffs.get((canton, target, "EINKOMMENSSTEUER")), married)
            if t is None:
                return None
            report.append(f"{canton} {target} income {status}: {t.kind}, splitting {t.splitting}")
            res[status] = tariff_curve(t, married, INCOME_GRID)
        return res

    def wealth(canton):
        res = {}
        table = deds.get((canton, "KANTON", "VERMOEGENSSTEUER"), {})
        for status, married in (("single", False), ("married", True)):
            t = pick(tariffs.get((canton, "KANTON", "VERMOEGENSSTEUER")), married)
            if t is None:
                return None
            spec = WEALTH_DEDUCTIONS[status]
            report.append(f"{canton} wealth {status}: {t.kind}, deductions {[r['id'] for r in deduction_rules(table, spec)]}")
            res[status] = wealth_curve(t, married, table, spec, WEALTH_GRID)
        return res

    federal = income("CH", "BUND")
    federal_deductions = deductions(deds.get(("CH", "BUND", "EINKOMMENSSTEUER"), {}), "CH", report)
    cantons = []
    for cid in sorted(NAMES):
        c = {"id": cid, "name": NAMES[cid], "income": income(cid, "KANTON"), "wealth": wealth(cid)}
        muni = income(cid, "GEMEINDE")
        if muni:
            c["municipalIncome"] = muni
        if not c["income"] or not c["wealth"]:
            raise ValueError(f"canton {cid}: tariff missing")
        c["deductions"] = deductions(deds.get((cid, "KANTON", "EINKOMMENSSTEUER"), {}), cid, report)
        if capital_rules is not None:
            if cid not in capital_rules:
                raise ValueError(f"canton {cid}: capital rule missing")
            capital(c, capital_rules[cid], tariffs, munis, report)
        if gain_rules is not None:
            if cid not in gain_rules:
                raise ValueError(f"canton {cid}: property gain rule missing")
            property_gain(c, gain_rules[cid], munis, report)
        cantons.append(c)
    if capital_rules is not None:
        # #64: federal tax on lump sums: one fifth of the tariff of the direct federal tax, without deductions
        fed = tariffs[("CH", "BUND", "EINKOMMENSSTEUER")]
        federal_capital = {status: [[x, round(pick(fed, married).tax(x, married) / 5 / x, 7) if x > 0 else 0.0]
                                    for x in CAPITAL_GRID]
                           for status, married in (("single", False), ("married", True))}
    data = {
        "year": year,
        "source": SOURCE,
        "note": NOTE,
        "federal": federal,
        "federalDeductions": federal_deductions,
        "cantons": cantons,
        "municipalities": munis,
    }
    if capital_rules is not None:
        data["federalCapital"] = federal_capital
    if gain_rules is not None:
        data["propertyGainSource"] = GAIN_SOURCE
    return data, report


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--steuerfuesse", required=True)
    ap.add_argument("--tarife", required=True)
    ap.add_argument("--abzuege", required=True)
    ap.add_argument("--kapitalregeln", default=RULES, help="rules of the cantons for lump sums (#64)")
    ap.add_argument("--quellen", help="directory of the source files from the laws (#287, default quellen/<year>)")
    ap.add_argument("--out", required=True)
    ap.add_argument("--bericht", help="file for the report of the chosen tariffs and deductions")
    a = ap.parse_args(argv)
    year, munis = read_multipliers(a.steuerfuesse)
    sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
    import grundstueckgewinn  # noqa: E402 (next to this file, also with python3 -I)

    src_tariffs, src_deductions, covered = read_sources(a.quellen or os.path.join(SOURCES, str(year)))
    check_coverage(covered, ESTV_ONLY)
    data, report = build(merge_sources(read_csv(a.tarife), src_tariffs, covered, ESTV_ONLY),
                         merge_sources(read_csv(a.abzuege), src_deductions, covered, ESTV_ONLY), year, munis,
                         read_rules(a.kapitalregeln), grundstueckgewinn.rules())
    report = origin_report(covered) + report
    with open(a.out, "w", encoding="utf-8") as f:
        json.dump(data, f, ensure_ascii=False, separators=(",", ":"))
        f.write("\n")
    if a.bericht:
        with open(a.bericht, "w", encoding="utf-8") as f:
            f.write("\n".join(report) + "\n")
    print(f"{a.out}: tax year {year}, {len(data['cantons'])} cantons, {len(munis)} municipalities", file=sys.stderr)


if __name__ == "__main__":
    main()
