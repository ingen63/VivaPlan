"""#63/AK11: Tests of the tool that builds the tax data file (invented tariffs and deductions).

Run: `python3 -I -m unittest discover -s tools/steuerdaten`
"""

import json
import math
import os
import sys
import tempfile
import unittest
import zipfile

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import build  # noqa: E402


def row(canton, target, kind, ttype, split, group, i, amount, pct, taxes=0, formula=""):
    return {"Kanton": canton, "Hoheit": target, "Steuerart": kind, "Tariftyp": ttype, "Splitting": str(split),
            "Gruppe": group, "Zeile": str(i), "Betrag": str(amount), "Prozent": str(pct), "Steuer": str(taxes),
            "Formel": formula}


def ded(canton, target, ident, fmt, amount="0", pct="0", lo="0", hi="0", kind="EINKOMMENSSTEUER"):
    return {"Kanton": canton, "Hoheit": target, "Steuerart": kind, "ID": ident, "Format": fmt,
            "Betrag": amount, "Prozent": pct, "Minimum": lo, "Maximum": hi}


class TariffTypes(unittest.TestCase):
    def test_zuerich_brackets(self):
        t = build.Tariff("ZUERICH", 0, [(10_000, 0, 0, ""), (10_000, 5, 0, ""), (1e10, 10, 0, "")])
        self.assertAlmostEqual(t.simple(15_000), 250)
        self.assertAlmostEqual(t.simple(30_000), 500 + 1000)

    def test_bund_base_plus_marginal_rate(self):
        t = build.Tariff("BUND", 0, [(0, 0, 0, ""), (10_000, 2, 0, ""), (20_000, 4, 200, "")])
        self.assertEqual(t.simple(5_000), 0)
        self.assertAlmostEqual(t.simple(15_000), 100)
        self.assertAlmostEqual(t.simple(25_000), 400)

    def test_flat_and_freiburg_average_rate(self):
        self.assertAlmostEqual(build.Tariff("FLATTAX", 0, [(0, 1.5, 0, "")]).simple(10_000), 150)
        t = build.Tariff("FREIBURG", 0, [(0, 0, 0, ""), (10_000, 2, 0, ""), (20_000, 4, 0, "")])
        self.assertAlmostEqual(t.simple(15_000), 15_000 * 0.03)
        self.assertAlmostEqual(t.simple(50_000), 50_000 * 0.04)

    def test_formula_uses_natural_log(self):
        t = build.Tariff("FORMEL", 0, [(0, 0, 0, ""), (1000, 0, 0, "0.01 * $wert$ * (log $wert$ - 1)")])
        self.assertAlmostEqual(t.simple(5000), 0.01 * 5000 * (math.log(5000) - 1))
        self.assertEqual(t.simple(500), 0.0)
        with self.assertRaises(ValueError):
            build.evaluate_formula("__import__('os')", 1)

    def test_splitting_for_married_only(self):
        t = build.Tariff("ZUERICH", 2, [(10_000, 0, 0, ""), (1e10, 10, 0, "")])
        self.assertAlmostEqual(t.tax(40_000, married=False), 3000)
        # rate of 20 000 (5 %) on 40 000
        self.assertAlmostEqual(t.tax(40_000, married=True), 2000)

    def test_brackets_with_base_amount_count_as_bund(self):
        t = build.load_tariffs([row("XX", "KANTON", "EINKOMMENSSTEUER", "ZUERICH", 0, "ALLE", 0, 0, 0, 0),
                                row("XX", "KANTON", "EINKOMMENSSTEUER", "ZUERICH", 0, "ALLE", 1, 100, 2, 10)])
        self.assertEqual(t[("XX", "KANTON", "EINKOMMENSSTEUER")][0][1].kind, "BUND")


class Groups(unittest.TestCase):
    def test_pick_single_and_married(self):
        a, b = build.Tariff("FLATTAX", 0, [(0, 1, 0, "")]), build.Tariff("FLATTAX", 0, [(0, 2, 0, "")])
        groups = [("LEDIG_ALLEINE,LEDIG_KONKUBINAT,LEDIG_OHNE_KINDER", a),
                  ("LEDIG_ALLEINE,LEDIG_KONKUBINAT,VERHEIRATET,LEDIG_MIT_KINDER", b)]
        self.assertIs(build.pick(groups, False), a)
        self.assertIs(build.pick(groups, True), b)
        self.assertIs(build.pick([("ALLE", a)], True), a)
        self.assertIsNone(build.pick([], True))
        with self.assertRaises(ValueError):
            build.pick([("LEDIG_MIT_KINDER", a)], False)


class Deductions(unittest.TestCase):
    def test_rule_formats(self):
        # fixed amount, only a maximum (fixed deduction), percentage with limits
        self.assertEqual(build.rule_of(ded("XX", "KANTON", "A", "STANDARDIZED", "5300"))[0],
                         {"base": 5300.0, "rate": 0, "min": 0, "max": 0})
        self.assertEqual(build.rule_of(ded("XX", "KANTON", "A", "MAXIMUM", hi="2900"))[0],
                         {"base": 2900.0, "rate": 0, "min": 0, "max": 0})
        r = build.rule_of(ded("XX", "KANTON", "A", "PERCENT,MINIMUM,MAXIMUM", pct="20", lo="4300", hi="10000"))[0]
        self.assertEqual(r, {"base": 0, "rate": 0.2, "min": 4300.0, "max": 10000.0})
        self.assertEqual(build.rule_amount(r, 10_000), 4300)
        self.assertEqual(build.rule_amount(r, 30_000), 6000)
        self.assertEqual(build.rule_amount(r, 100_000), 10_000)

    def test_earned_rule_variants(self):
        """#63/AK3: flat deduction for professional expenses: percentage with limits, with the minimum of the ESTV
        data (STANDARDIZED), only a maximum (fixed deduction, marked as an approximation)."""
        zh = build.earned_rule(ded("ZH", "KANTON", "HauptErw_EK", "PERCENT,MINIMUM,MAXIMUM", pct="3", lo="2000",
                                   hi="4000"))
        self.assertEqual(zh, {"base": 0, "rate": 0.03, "min": 2000.0, "max": 4000.0})
        self.assertEqual(build.rule_amount(zh, 100_000), 3000)  # example of AK3
        ai = build.earned_rule(ded("AI", "KANTON", "HauptErw_EK", "STANDARDIZED,PERCENT,MAXIMUM", "1000", "5",
                                   hi="5000"))
        self.assertEqual(ai, {"base": 0, "rate": 0.05, "min": 1000.0, "max": 5000.0})
        sz = build.earned_rule(ded("SZ", "KANTON", "HauptErw_EK", "PERCENT,MAXIMUM", pct="20", hi="6900"))
        self.assertEqual(sz, {"base": 0, "rate": 0.2, "min": 0, "max": 6900.0})
        bl = build.earned_rule(ded("BL", "KANTON", "HauptErw_EK", "MAXIMUM", hi="500"))
        self.assertEqual(bl, {"base": 500.0, "rate": 0, "min": 0, "max": 500.0, "approx": True})

    def test_variants_retired_and_working(self):
        """#63/AK3: insurance premiums «without» contributions to the 2nd and 3rd pillar for retired persons, «with»
        for working persons; the next available variant otherwise."""
        table = {r["ID"]: r for r in [
            ded("XX", "KANTON", "KKSparLedigOhneBVGS3a_EK", "MAXIMUM", hi="4350"),
            ded("XX", "KANTON", "KKSparLedigMitBVGS3a_EK", "MAXIMUM", hi="2900"),
            ded("XX", "KANTON", "SozLedig_EK", "STANDARDIZED", "5300"),
            ded("XX", "KANTON", "KKSparzVerhMitBVGS3a_EK", "MAXIMUM", hi="5800"),
            ded("XX", "KANTON", "HauptErw_EK", "PERCENT,MINIMUM,MAXIMUM", pct="3", lo="2000", hi="4000"),
        ]}
        d = build.deductions(table, "XX", [])
        self.assertEqual([(r["id"], r["kind"], r["base"]) for r in d["single"]["retired"]],
                         [("KKSparLedigOhneBVGS3a_EK", "insurance", 4350.0), ("SozLedig_EK", "social", 5300.0)])
        self.assertEqual(d["single"]["working"][0]["base"], 2900.0)
        # married: no variant «without», the variant «with» applies for both
        self.assertEqual([r["base"] for r in d["married"]["retired"]], [5800.0])
        self.assertEqual([r["base"] for r in d["married"]["working"]], [5800.0])
        self.assertEqual(d["earned"]["rate"], 0.03)
        with self.assertRaises(ValueError):
            build.deductions({}, "XX", [])

    def test_kind_of(self):
        self.assertEqual(build.kind_of("SparzinsenLedig_EK"), "savingsInterest")
        self.assertEqual(build.kind_of("SozZusAbzug_EK"), "social")
        self.assertEqual(build.kind_of("PrivVersLedig_EK"), "insurance")


def minimal_inputs():
    tariffs, deductions = [], []
    for c in ["CH"] + sorted(build.NAMES):
        target = "BUND" if c == "CH" else "KANTON"
        tariffs.append(row(c, target, "EINKOMMENSSTEUER", "FLATTAX", 0, "ALLE", 0, 0, 10))
        if c != "CH":
            tariffs.append(row(c, target, "VERMOEGENSSTEUER", "FLATTAX", 0, "ALLE", 0, 0, 0.5))
        deductions.append(ded(c, target, "HauptErw_EK", "PERCENT,MINIMUM,MAXIMUM", pct="3", lo="2000", hi="4000"))
    tariffs.append(row("SZ", "GEMEINDE", "EINKOMMENSSTEUER", "FLATTAX", 0, "ALLE", 0, 0, 5))
    deductions += [ded("ZH", "KANTON", "KKSparLedigOhneBVGS3a_EK", "MAXIMUM", hi="5000"),
                   ded("ZH", "KANTON", "SozLedigOhneKinder_VM", "STANDARDIZED", "100000", kind="VERMOEGENSSTEUER")]
    return tariffs, deductions


class Build(unittest.TestCase):
    def test_income_curve_is_the_tariff_and_deductions_are_rules(self):
        """#63: The income curves are the tariff on the taxable income; the deductions are rules beside it."""
        tariffs, deductions = minimal_inputs()
        data, report = build.build(tariffs, deductions, 2030, [[1, "Musterdorf", "ZH", [1, 1, 0.1, 0.1, 0.1],
                                                                 [1, 1, 0, 0, 0]]])
        zh = next(c for c in data["cantons"] if c["id"] == "ZH")
        self.assertEqual(next(p for p in zh["income"]["single"] if p[0] == 50_000)[1], 0.1)
        self.assertEqual(zh["deductions"]["single"]["retired"],
                         [{"id": "KKSparLedigOhneBVGS3a_EK", "kind": "insurance", "base": 5000.0, "rate": 0,
                           "min": 0, "max": 0}])
        self.assertEqual(zh["deductions"]["earned"], {"base": 0, "rate": 0.03, "min": 2000.0, "max": 4000.0})
        # wealth: social deduction included in the rate before deductions
        self.assertAlmostEqual(next(p for p in zh["wealth"]["single"] if p[0] == 200_000)[1], 0.0025)
        self.assertEqual(data["year"], 2030)
        self.assertEqual(len(data["cantons"]), 26)
        self.assertEqual(data["federalDeductions"]["earned"]["max"], 4000.0)
        sz = next(c for c in data["cantons"] if c["id"] == "SZ")
        self.assertIn("municipalIncome", sz)
        self.assertNotIn("municipalIncome", zh)
        self.assertTrue(data["source"] and data["note"] and report)

    def test_missing_tariff_is_an_error(self):
        tariffs, deductions = minimal_inputs()
        tariffs = [t for t in tariffs if not (t["Kanton"] == "JU" and t["Steuerart"] == "VERMOEGENSSTEUER")]
        with self.assertRaises(ValueError):
            build.build(tariffs, deductions, 2030, [])


class Inputs(unittest.TestCase):
    def test_read_multipliers_from_xlsx(self):
        """Tax multipliers from a minimal Excel export (inline strings): year, BFS number, rates as shares."""
        cells = lambda r, vals: "".join(  # noqa: E731
            f'<c r="{col}{r}" t="inlineStr"><is><t>{v}</t></is></c>' if isinstance(v, str)
            else f'<c r="{col}{r}"><v>{v}</v></c>' for col, v in zip("ABCDEFGHIJKLMN", vals))
        rows = [(1, ["2026"]), (2, ["Kantons-Id", "Kanton", "BfS-Id", "Gemeinde"]),
                (3, [1, "ZH", 261, "Zürich", 95, 119, 10, 10.5, 14, 95, 119, 10, 10.5, 14])]
        sheet = ('<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main"><sheetData>'
                 + "".join(f'<row r="{r}">{cells(r, v)}</row>' for r, v in rows) + "</sheetData></worksheet>")
        with tempfile.TemporaryDirectory() as d:
            path = os.path.join(d, "rates.xlsx")
            with zipfile.ZipFile(path, "w") as z:
                z.writestr("xl/worksheets/sheet1.xml", sheet)
            year, munis = build.read_multipliers(path)
        self.assertEqual(year, 2026)
        self.assertEqual(munis, [[261, "Zürich", "ZH", [0.95, 1.19, 0.1, 0.105, 0.14], [0.95, 1.19, 0.1, 0.105, 0.14]]])

    def test_main_writes_file_and_report(self):
        tariffs, deductions = minimal_inputs()
        with tempfile.TemporaryDirectory() as d:
            def write_csv(name, rows):
                import csv
                path = os.path.join(d, name)
                with open(path, "w", encoding="utf-8", newline="") as f:
                    w = csv.DictWriter(f, fieldnames=list(rows[0]))
                    w.writeheader()
                    w.writerows(rows)
                return path
            t, a = write_csv("t.csv", tariffs), write_csv("a.csv", deductions)
            orig = build.read_multipliers
            build.read_multipliers = lambda path: (2030, [])
            try:
                out, rep = os.path.join(d, "out.json"), os.path.join(d, "report.txt")
                build.main(["--steuerfuesse", "x.xlsx", "--tarife", t, "--abzuege", a, "--out", out, "--bericht", rep])
            finally:
                build.read_multipliers = orig
            with open(out, encoding="utf-8") as f:
                self.assertEqual(json.load(f)["year"], 2030)
            self.assertTrue(os.path.getsize(rep) > 0)


if __name__ == "__main__":
    unittest.main()
