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
import grundstueckgewinn  # noqa: E402


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

    def test_second_earner_rule(self):
        """#63/AK3: second-earner deduction: percentage of the lower earned income with limits (federal), fixed
        amount (ZH), percentage of the total earned income (BE), none without a row (TG)."""
        ch = build.second_earner_rule(ded("CH", "BUND", "ZweitVerdiener_EK", "PERCENT,MINIMUM,MAXIMUM", pct="50",
                                          lo="8600", hi="14100"), "CH")
        self.assertEqual(ch, {"base": 0, "rate": 0.5, "min": 8600.0, "max": 14100.0, "reference": "lower"})
        zh = build.second_earner_rule(ded("ZH", "KANTON", "ZweitVerdiener_EK", "MAXIMUM", hi="6200"), "ZH")
        self.assertEqual(zh, {"base": 6200.0, "rate": 0, "min": 0, "max": 0, "reference": "lower"})
        be = build.second_earner_rule(ded("BE", "KANTON", "ZweitVerdiener_EK", "PERCENT,MAXIMUM", pct="2",
                                          hi="9500"), "BE")
        self.assertEqual(be["reference"], "total")
        self.assertIsNone(build.second_earner_rule(None, "TG"))

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


class Capital(unittest.TestCase):
    """#64: Rules for lump sums from pension savings."""

    def tariffs(self):
        rows = [row("XX", "KANTON", "EINKOMMENSSTEUER", "ZUERICH", 2, "ALLE", 0, 10_000, 0),
                row("XX", "KANTON", "EINKOMMENSSTEUER", "ZUERICH", 2, "ALLE", 1, 1e10, 10),
                row("XX", "KANTON", "VORSORGESTEUER", "ZUERICH", 0, "ALLE", 0, 0, 0, 0),
                row("XX", "KANTON", "VORSORGESTEUER", "ZUERICH", 0, "ALLE", 1, 100_000, 2, 1000)]
        return build.load_tariffs(rows)

    def test_income_rule_with_share_divisor_min_max(self):
        t = self.tariffs()
        rule = {"kind": "income", "share": 0.5, "divisor": 2}
        # rate for 50 000: (40 000 × 10 %) / 50 000 = 8 %, half of it
        self.assertAlmostEqual(build.capital_tax(rule, t, "XX", False, 100_000), 100_000 * 0.04)
        self.assertAlmostEqual(build.capital_tax({**rule, "max": 0.03}, t, "XX", False, 100_000), 3000)
        self.assertAlmostEqual(build.capital_tax({**rule, "min": 0.05}, t, "XX", False, 100_000), 5000)
        # married: splitting 2 of the tariff, rate for 25 000 = 6 %, half of it
        self.assertAlmostEqual(build.capital_tax(rule, t, "XX", True, 100_000), 100_000 * 0.03)
        self.assertEqual(build.capital_tax(rule, t, "XX", False, 0), 0.0)

    def test_table_with_base_amount_and_overrides(self):
        t = self.tariffs()
        # brackets with a base amount count as type BUND: 1000 + 50 000 × 2 %
        rule = {"kind": "table", "married": {"rebate": {"rate": 0.002, "max": 500}}}
        self.assertAlmostEqual(build.capital_tax(rule, t, "XX", False, 150_000), 2000)
        self.assertAlmostEqual(build.capital_tax(rule, t, "XX", True, 150_000), 1700)
        self.assertAlmostEqual(build.capital_tax(rule, t, "XX", True, 1_000_000), 19_000 - 500)

    def test_unknown_rule_and_missing_table_are_errors(self):
        t = self.tariffs()
        with self.assertRaises(ValueError):
            build.capital_tax({"kind": "flat"}, t, "XX", False, 1000)
        with self.assertRaises(ValueError):
            build.capital_tax({"kind": "table"}, t, "YY", False, 1000)

    def test_rules_file_covers_all_cantons(self):
        rules = build.read_rules(build.RULES)
        self.assertEqual(set(rules), set(build.NAMES))
        for cid, r in rules.items():
            self.assertTrue(r["text"], cid)
            self.assertIn(r["kind"], ("table", "income"), cid)
        # #288: Zug follows the law (StG ZG § 37 Abs. 2)
        self.assertEqual(sorted(c for c, r in rules.items() if r.get("approx")), ["VS"])

    def test_build_writes_curves_rule_and_federal_fifth(self):
        tariffs, deductions = minimal_inputs()
        rules = {c: {"kind": "income", "share": 0.2, "text": "ein Fünftel"} for c in build.NAMES}
        rules["SZ"] = {"kind": "income", "share": 0.5, "text": "Hälfte", "approx": "angenähert",
                       "municipal": {"kind": "income", "target": "GEMEINDE"}}
        rules["FR"] = {"kind": "income", "cantonMultiplier": 1.0, "text": "Steuerfuss 100 %"}
        munis = [[1, "Musterdorf", "FR", [0.8, 1, 0, 0, 0], [0.8, 1, 0, 0, 0]]]
        data, report = build.build(tariffs, deductions, 2030, munis, rules)
        zh = next(c for c in data["cantons"] if c["id"] == "ZH")
        # 10 % flat tariff, one fifth of it
        self.assertAlmostEqual(next(p for p in zh["capital"]["single"] if p[0] == 500_000)[1], 0.02)
        self.assertEqual(zh["capitalRule"], "ein Fünftel")
        self.assertNotIn("capitalMunicipal", zh)
        self.assertNotIn("capitalApprox", zh)
        sz = next(c for c in data["cantons"] if c["id"] == "SZ")
        self.assertAlmostEqual(next(p for p in sz["capitalMunicipal"]["married"] if p[0] == 100_000)[1], 0.05)
        self.assertEqual(sz["capitalApprox"], "angenähert")
        fr = next(c for c in data["cantons"] if c["id"] == "FR")
        # curve scaled by 1.0 / 0.8 so that curve × 0.8 gives the tax with multiplier 100 %
        self.assertAlmostEqual(next(p for p in fr["capital"]["single"] if p[0] == 100_000)[1], 0.125)
        self.assertAlmostEqual(next(p for p in fr["capitalMunicipal"]["single"] if p[0] == 100_000)[1], 0.1)
        # federal: one fifth of the federal tariff (10 %)
        self.assertAlmostEqual(next(p for p in data["federalCapital"]["married"] if p[0] == 1_000_000)[1], 0.02)
        self.assertTrue(any("capital" in line for line in report))
        # without rules no lump-sum fields
        data, _ = build.build(tariffs, deductions, 2030, munis)
        self.assertNotIn("federalCapital", data)
        # missing canton or non-uniform cantonal multiplier
        with self.assertRaises(ValueError):
            build.build(tariffs, deductions, 2030, munis, {k: v for k, v in rules.items() if k != "JU"})
        with self.assertRaises(ValueError):
            build.build(tariffs, deductions, 2030, munis + [[2, "Beispielwil", "FR", [0.9] * 5, [0.9] * 5]], rules)


class PropertyGain(unittest.TestCase):
    """#65: Rule sets of the property gain tax."""

    MODELS = {"zurich", "aargau", "curve", "return"}
    KEYS = {"model", "brackets", "uniformAbove", "longBrackets", "durationRates", "rateCurve", "returnBounds",
            "returnMaxDuration", "shortSurcharge", "durationAbove", "duration", "gainDuration", "exemptBelow",
            "allowance", "maxShare", "useMultiplier", "text", "source", "approx"}

    def test_issue_65_ak2_rules_cover_all_cantons_with_known_fields(self):
        """#65/AK2: every canton has a rule set in one of the four models, with text and source."""
        rules = grundstueckgewinn.rules()
        self.assertEqual(set(rules), set(build.NAMES))
        for cid, r in rules.items():
            self.assertIn(r["model"], self.MODELS, cid)
            self.assertLessEqual(set(r), self.KEYS, cid)
            self.assertTrue(r["text"] and r["source"], cid)
            if r["model"] == "zurich":
                self.assertTrue(r["brackets"], cid)
            if r["model"] == "aargau":
                self.assertTrue(r["durationRates"], cid)
        self.assertEqual({r["model"] for r in rules.values()}, self.MODELS)
        self.assertEqual(sorted(c for c, r in rules.items() if r.get("useMultiplier")),
                         ["BE", "JU", "OW", "SH", "SO"])  # #288: Lucerne with the fixed multiplier 4.2

    def test_issue_65_ak3_approximations_are_marked(self):
        """#65/AK3: the simplified rules carry a note."""
        rules = grundstueckgewinn.rules()
        self.assertEqual(sorted(c for c, r in rules.items() if r.get("approx")), ["BL", "BS", "SG", "VD", "VS"])

    def test_steps_and_monthly_surcharge(self):
        self.assertEqual(grundstueckgewinn.steps(5, -0.05, -0.03, -0.12), [[5, -0.05], [6, -0.08], [7, -0.11],
                                                                           [8, -0.12]])
        self.assertEqual(grundstueckgewinn.steps(1, 0.1, 0.1, 1.0, end_year=2), [[1, 0.1], [2, 0.2]])
        self.assertEqual(grundstueckgewinn.monthly_surcharge(3, 0.01), [[0.0, 0.03], [round(1 / 12, 6), 0.02],
                                                                        [round(2 / 12, 6), 0.01], [0.25, 0.0]])

    def test_issue_65_ak1_build_writes_rule_text_approx_and_median_multiplier(self):
        """#65/AK1: the tax data file holds the rule, its text, the note and the median tax multiplier."""
        tariffs, deductions = minimal_inputs()
        rules = {c: {"model": "aargau", "durationRates": [[0, 0.3]], "text": f"Regel {c}", "source": "S. 1"}
                 for c in build.NAMES}
        rules["BE"] = {"model": "zurich", "brackets": [[1e12, 0.02]], "useMultiplier": True, "text": "Bern",
                       "source": "S. 9", "approx": "angenähert"}
        munis = [[1, "A", "BE", [1.0, 1.0, 0, 0, 0], [0] * 5], [2, "B", "BE", [1.0, 1.5, 0, 0, 0], [0] * 5],
                 [3, "C", "BE", [1.0, 2.0, 0, 0, 0], [0] * 5]]
        munis += [[10 + i, "X", c, [0.5, 0.5, 0, 0, 0], [0] * 5] for i, c in enumerate(build.NAMES) if c != "BE"]
        data, report = build.build(tariffs, deductions, 2030, munis, None, rules)
        be = next(c for c in data["cantons"] if c["id"] == "BE")
        self.assertEqual(be["propertyGain"], {"model": "zurich", "brackets": [[1e12, 0.02]], "useMultiplier": True})
        self.assertEqual(be["propertyGainRule"], "Bern")
        self.assertEqual(be["propertyGainApprox"], "angenähert")
        self.assertEqual(be["gainMultiplier"], 2.5)
        zh = next(c for c in data["cantons"] if c["id"] == "ZH")
        self.assertNotIn("propertyGainApprox", zh)
        self.assertEqual(zh["gainMultiplier"], 1.0)
        self.assertTrue(data["propertyGainSource"])
        self.assertTrue(any("property gain" in line and "S. 9" in line for line in report))
        # the input rules stay unchanged
        self.assertIn("text", rules["BE"])
        # without rules no property gain fields
        data, _ = build.build(tariffs, deductions, 2030, munis)
        self.assertNotIn("propertyGainSource", data)
        self.assertNotIn("propertyGain", data["cantons"][0])
        # missing canton or canton without municipality
        with self.assertRaises(ValueError):
            build.build(tariffs, deductions, 2030, munis, None, {k: v for k, v in rules.items() if k != "JU"})
        with self.assertRaises(ValueError):
            build.build(tariffs, deductions, 2030, [m for m in munis if m[2] != "ZH"], None, rules)

    def test_issue_65_ak2_tax_file_holds_the_rules(self):
        """#65/AK2: the built-in tax data file was built from grundstueckgewinn.py."""
        path = os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", "..", "crates", "engine", "data",
                            "steuern-2026.json")
        with open(path, encoding="utf-8") as f:
            data = json.load(f)
        rules = grundstueckgewinn.rules()
        for c in data["cantons"]:
            r = json.loads(json.dumps(rules[c["id"]]))
            self.assertEqual(c["propertyGainRule"], r.pop("text"), c["id"])
            r.pop("source")
            self.assertEqual(c.get("propertyGainApprox"), r.pop("approx", None), c["id"])
            self.assertEqual(c["propertyGain"], r, c["id"])
            self.assertGreater(c["gainMultiplier"], 0, c["id"])


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
            rules = os.path.join(d, "rules.json")
            with open(rules, "w", encoding="utf-8") as f:
                json.dump({"_info": "x", **{c: {"kind": "income", "text": c} for c in build.NAMES}}, f)
            orig = build.read_multipliers
            # one municipality per canton for the median tax multiplier of the property gain tax (#65)
            build.read_multipliers = lambda path: (2030, [[i, f"Ort {c}", c, [1, 1, 0, 0, 0], [1, 1, 0, 0, 0]]
                                                          for i, c in enumerate(sorted(build.NAMES))])
            try:
                out, rep = os.path.join(d, "out.json"), os.path.join(d, "report.txt")
                build.main(["--steuerfuesse", "x.xlsx", "--tarife", t, "--abzuege", a, "--kapitalregeln", rules,
                           "--out", out, "--bericht", rep])
            finally:
                build.read_multipliers = orig
            with open(out, encoding="utf-8") as f:
                data = json.load(f)
            self.assertEqual(data["year"], 2030)
            self.assertEqual(data["cantons"][0]["capitalRule"], data["cantons"][0]["id"])
            self.assertTrue(os.path.getsize(rep) > 0)
            # #65: the property gain rules come from grundstueckgewinn.py
            first = data["cantons"][0]
            self.assertEqual(first["propertyGainRule"], grundstueckgewinn.rules()[first["id"]]["text"])
            self.assertTrue(data["propertyGainSource"])


def write_rows(path, rows):
    import csv
    with open(path, "w", encoding="utf-8", newline="") as f:
        w = csv.DictWriter(f, fieldnames=list(rows[0]))
        w.writeheader()
        w.writerows(rows)


class Sources(unittest.TestCase):
    """#287: Source files from the laws replace the ESTV rows of their canton."""

    REPO = os.path.join(os.path.dirname(os.path.abspath(__file__)), "quellen", "2026")

    def test_issue_287_ak1_read_sources_checks_canton_and_source(self):
        """#287/AK1: every row of a source file names its source and belongs to the canton of the file."""
        with tempfile.TemporaryDirectory() as d:
            self.assertEqual(build.read_sources(os.path.join(d, "fehlt")), ([], [], set()))
            good = {**row("ZH", "KANTON", "EINKOMMENSSTEUER", "FLATTAX", 0, "ALLE", 0, 0, 7), "Quelle": "StG § 1"}
            write_rows(os.path.join(d, "ZH_tarife.csv"), [good])
            write_rows(os.path.join(d, "ZH_abzuege.csv"),
                       [{**ded("ZH", "KANTON", "HauptErw_EK", "STANDARDIZED", "100"), "Quelle": "StG § 26"}])
            write_rows(os.path.join(d, "notiz.csv"), [{"x": "1"}])  # other files are ignored
            tariffs, deductions, covered = build.read_sources(d)
            self.assertEqual((len(tariffs), len(deductions), covered), (1, 1, {"ZH"}))
            write_rows(os.path.join(d, "CH_abzuege.csv"),
                       [{**ded("CH", "BUND", "HauptErw_EK", "STANDARDIZED", "100"), "Quelle": "DBG Art. 26"}])
            with self.assertRaisesRegex(ValueError, "CH_tarife.csv missing"):
                build.read_sources(d)
            os.remove(os.path.join(d, "CH_abzuege.csv"))
            write_rows(os.path.join(d, "ZH_tarife.csv"), [{**good, "Quelle": " "}])
            with self.assertRaisesRegex(ValueError, "source missing"):
                build.read_sources(d)
            write_rows(os.path.join(d, "ZH_tarife.csv"), [{**good, "Kanton": "BE"}])
            with self.assertRaisesRegex(ValueError, "instead of ZH"):
                build.read_sources(d)
            write_rows(os.path.join(d, "XX_tarife.csv"), [{**good, "Kanton": "XX"}])
            with self.assertRaisesRegex(ValueError, "unknown canton"):
                build.read_sources(d)

    def test_issue_287_ak2_sources_replace_the_estv_rows_of_their_canton(self):
        """#287/AK2: cantons with a source file come from it, the others unchanged from the ESTV tables; the report
        names the origin per canton."""
        tariffs, deductions = minimal_inputs()
        src = [{**row("ZH", "KANTON", "EINKOMMENSSTEUER", "FLATTAX", 0, "ALLE", 0, 0, 7), "Quelle": "q"},
               {**row("ZH", "KANTON", "VERMOEGENSSTEUER", "FLATTAX", 0, "ALLE", 0, 0, 0.2), "Quelle": "q"}]
        merged = build.merge_sources(tariffs, src, {"ZH"})
        self.assertEqual([r["Prozent"] for r in merged if r["Kanton"] == "ZH"], ["7", "0.2"])
        self.assertEqual(len([r for r in merged if r["Kanton"] == "BE"]), 2)
        src_ded = [{**ded("ZH", "KANTON", "HauptErw_EK", "STANDARDIZED", "1500"), "Quelle": "q"}]
        data, _ = build.build(merged, build.merge_sources(deductions, src_ded, {"ZH"}), 2030, [])
        zh = next(c for c in data["cantons"] if c["id"] == "ZH")
        be = next(c for c in data["cantons"] if c["id"] == "BE")
        self.assertEqual(next(p for p in zh["income"]["single"] if p[0] == 50_000)[1], 0.07)
        self.assertEqual(next(p for p in be["income"]["single"] if p[0] == 50_000)[1], 0.1)
        self.assertEqual(zh["deductions"]["earned"]["base"], 1500.0)
        self.assertEqual(zh["deductions"]["single"]["retired"], [])  # the ESTV deductions of ZH are gone
        rep = build.origin_report({"CH", "ZH"})
        self.assertEqual(len(rep), 27)
        self.assertIn("ZH origin: laws (source files)", rep)
        self.assertIn("BE origin: ESTV tables", rep)

    def test_issue_287_ak2_main_reads_the_source_directory(self):
        """#287/AK2: `main` takes the rows of `--quellen` and writes the origin into the report."""
        tariffs, deductions = minimal_inputs()
        with tempfile.TemporaryDirectory() as d:
            t, a = os.path.join(d, "t.csv"), os.path.join(d, "a.csv")
            write_rows(t, tariffs)
            write_rows(a, deductions)
            q = os.path.join(d, "quellen")
            os.mkdir(q)
            write_rows(os.path.join(q, "ZH_tarife.csv"),
                       [{**row("ZH", "KANTON", "EINKOMMENSSTEUER", "FLATTAX", 0, "ALLE", 0, 0, 7), "Quelle": "q"},
                        {**row("ZH", "KANTON", "VERMOEGENSSTEUER", "FLATTAX", 0, "ALLE", 0, 0, 0.2), "Quelle": "q"}])
            write_rows(os.path.join(q, "ZH_abzuege.csv"),
                       [{**ded("ZH", "KANTON", "HauptErw_EK", "STANDARDIZED", "1500"), "Quelle": "q"}])
            rules = os.path.join(d, "rules.json")
            with open(rules, "w", encoding="utf-8") as f:
                json.dump({c: {"kind": "income", "text": c} for c in build.NAMES}, f)
            orig = build.read_multipliers
            build.read_multipliers = lambda path: (2030, [[i, f"Ort {c}", c, [1, 1, 0, 0, 0], [1, 1, 0, 0, 0]]
                                                          for i, c in enumerate(sorted(build.NAMES))])
            try:
                out, rep = os.path.join(d, "out.json"), os.path.join(d, "report.txt")
                build.main(["--steuerfuesse", "x.xlsx", "--tarife", t, "--abzuege", a, "--kapitalregeln", rules,
                            "--quellen", q, "--out", out, "--bericht", rep])
            finally:
                build.read_multipliers = orig
            with open(out, encoding="utf-8") as f:
                zh = next(c for c in json.load(f)["cantons"] if c["id"] == "ZH")
            self.assertEqual(next(p for p in zh["income"]["single"] if p[0] == 50_000)[1], 0.07)
            with open(rep, encoding="utf-8") as f:
                self.assertIn("ZH origin: laws (source files)", f.read())

    def test_issue_287_ak1_repository_sources_for_federal_and_zurich(self):
        """#287/AK1: the source files 2026 for the federal tax and Zurich exist and name a source per row."""
        tariffs, deductions, covered = build.read_sources(self.REPO)
        self.assertLessEqual({"CH", "ZH"}, covered)
        ids = {(r["Kanton"], r["ID"]) for r in deductions}
        for cid in ("CH", "ZH"):
            self.assertIn((cid, "HauptErw_EK"), ids)
            self.assertIn((cid, "ZweitVerdiener_EK"), ids)
            for ident in ("KKSparzVerhMitBVGS3a_EK", "KKSparVerhOhneBVGS3a_EK", "KKSparLedigMitBVGS3a_EK",
                          "KKSparLedigOhneBVGS3a_EK"):
                self.assertIn((cid, ident), ids)

    def test_issue_287_ak3_federal_tariff_amounts_are_consistent(self):
        """#287/AK3: DBG Art. 36: the amount of each bracket follows from the previous bracket and its rate (the law
        rounds to 5 centimes), and the tariff gives the amounts printed in the law."""
        tariffs, _, _ = build.read_sources(self.REPO)
        fed = build.load_tariffs(tariffs)[("CH", "BUND", "EINKOMMENSSTEUER")]
        for married in (False, True):
            t = build.pick(fed, married)
            for (a0, p0, t0, _), (a1, _, t1, _) in zip(t.rows, t.rows[1:]):
                if t1:
                    self.assertAlmostEqual(t0 + (a1 - a0) * p0 / 100, t1, delta=0.05, msg=(married, a1))
        single, married = build.pick(fed, False), build.pick(fed, True)
        self.assertAlmostEqual(single.tax(185_100, False), 10_936.55, places=2)
        self.assertAlmostEqual(single.tax(1_000_000, False), 91_310 + 206_000 * 0.115, places=2)
        self.assertAlmostEqual(married.tax(152_400, True), 5_692, places=2)

    def test_issue_287_ak4_zurich_tariffs_from_the_law(self):
        """#287/AK4: StG ZH § 35 and § 47 (2026): the brackets add up to the thresholds of the law."""
        tariffs, _, _ = build.read_sources(self.REPO)
        t = build.load_tariffs(tariffs)
        inc = t[("ZH", "KANTON", "EINKOMMENSSTEUER")]
        wealth = t[("ZH", "KANTON", "VERMOEGENSSTEUER")]
        # sum of the widths up to the top rate: 266 700 / 370 600 (income), 3 304 000 / 3 385 000 (wealth)
        for tariffs_, married, top in ((inc, False, 266_700), (inc, True, 370_600), (wealth, False, 3_304_000),
                                       (wealth, True, 3_385_000)):
            self.assertEqual(sum(r[0] for r in build.pick(tariffs_, married).rows[:-1]), top)
        self.assertAlmostEqual(build.pick(inc, False).tax(266_700, False), 24_655)

    def test_issue_287_ak4_zurich_rules_name_the_law(self):
        """#287/AK4: the lump-sum and property gain rules of Zurich name the article of the tax law."""
        self.assertTrue(build.read_rules(build.RULES)["ZH"]["source"].startswith("StG ZH § 37"))
        self.assertTrue(grundstueckgewinn.rules()["ZH"]["source"].startswith("StG ZH § 225"))

    CENTRAL = ("LU", "UR", "SZ", "OW", "NW", "ZG")

    def test_issue_288_ak1_central_switzerland_sources_are_complete(self):
        """#288/AK1: source files 2026 for LU, UR, SZ, OW, NW and ZG with tariffs for income and wealth, the
        deductions used and a source per row (checked by `read_sources`)."""
        tariffs, deductions, covered = build.read_sources(self.REPO)
        self.assertLessEqual(set(self.CENTRAL), covered)
        kinds = {(r["Kanton"], r["Steuerart"]) for r in tariffs}
        ids = {(r["Kanton"], r["ID"]) for r in deductions}
        for cid in self.CENTRAL:
            self.assertIn((cid, "EINKOMMENSSTEUER"), kinds)
            self.assertIn((cid, "VERMOEGENSSTEUER"), kinds)
            self.assertIn((cid, "HauptErw_EK"), ids)
            self.assertIn((cid, "ZweitVerdiener_EK"), ids)
            self.assertIn((cid, "KKSparLedigMitBVGS3a_EK"), ids)
            self.assertIn((cid, "KKSparVerhOhneBVGS3a_EK"), ids)
        self.assertIn(("SZ", "GEMEINDE", "EINKOMMENSSTEUER"),
                      {(r["Kanton"], r["Hoheit"], r["Steuerart"]) for r in tariffs})

    def test_issue_288_ak1_values_from_the_laws(self):
        """#288/AK1: spot checks against the laws: LU § 57 Abs. 1 (last bracket, then 5.7 % on the whole income), NW
        Art. 40 (divisor 1.85), OW Art. 37 Abs. 1 lit. e Ziff. 2 (10 000 francs also for single persons, missing in
        the ESTV tables), ZG Grundtarif 2026 of the tax administration."""
        tariffs, deductions, _ = build.read_sources(self.REPO)
        t = build.load_tariffs(tariffs)
        lu = build.pick(t[("LU", "KANTON", "EINKOMMENSSTEUER")], False)
        self.assertAlmostEqual(lu.tax(2_096_800, False) / 2_096_800, 0.057, places=3)
        nw = build.pick(t[("NW", "KANTON", "EINKOMMENSSTEUER")], True)
        self.assertAlmostEqual(nw.tax(185_000, True), 185_000 * nw.tax(100_000, False) / 100_000, places=2)
        ow = {r["ID"]: r for r in deductions if r["Kanton"] == "OW"}
        self.assertEqual(float(ow["SozLedig_EK"]["Betrag"]), 10_000)
        self.assertIn("Art. 37 Abs. 1 lit. e Ziff. 2", ow["SozLedig_EK"]["Quelle"])
        zg = build.pick(t[("ZG", "KANTON", "EINKOMMENSSTEUER")], False)
        self.assertAlmostEqual(zg.tax(149_900, False), 10_326.50, places=2)

    def test_issue_288_ak2_rules_name_the_law(self):
        """#288/AK2: the lump-sum and property gain rules of these cantons name the article of the law; Zug's lump-sum
        rule is no longer an approximation."""
        capital = build.read_rules(build.RULES)
        gain = grundstueckgewinn.rules()
        for cid in self.CENTRAL:
            self.assertRegex(capital[cid]["source"], rf"^StG {cid} (§|Art\.) \d+", cid)
            self.assertRegex(gain[cid]["source"], rf"^(GG)?StG {cid} (§|Art\.)", cid)
        self.assertNotIn("approx", capital["ZG"])

    def test_issue_288_ak4_zug_lump_sums_by_parts(self):
        """#288/AK4: Zug, StG § 37 Abs. 2: 30 % of the tariff tax on the first 230 600 francs and 40 % on the rest,
        at least 1 %; values of the tax calculator of the canton of Zug 2026 (simple tax)."""
        tariffs, _, _ = build.read_sources(self.REPO)
        t = build.load_tariffs(tariffs)
        rule = build.read_rules(build.RULES)["ZG"]
        for married, amount, want in ((False, 30_000, 300.00), (False, 500_000, 13_655.55),
                                      (True, 300_000, 6_834.70), (True, 1_000_000, 29_234.70)):
            self.assertAlmostEqual(build.capital_tax(rule, t, "ZG", married, amount), want, delta=0.05,
                                   msg=(married, amount))

    def test_issue_288_ak2_lucerne_property_gain_with_fixed_multiplier(self):
        """#288/AK2: Lucerne, GGStG §§ 22–24: income tariff 2026 × 4.2 without the multiplier of the municipality."""
        lu = grundstueckgewinn.rules()["LU"]
        self.assertFalse(lu.get("useMultiplier", False))
        self.assertEqual(lu["brackets"][3], [1200, 0.084])
        self.assertEqual(lu["uniformAbove"], [2096800, 0.2394])


if __name__ == "__main__":
    unittest.main()
