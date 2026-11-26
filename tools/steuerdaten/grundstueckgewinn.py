"""#65: Rule sets of the property gain tax for all 26 cantons.

Source: ESTV, «Steuermäppchen» on the property gain tax («Grundstückgewinnsteuer»), tax period 2025
(as of 26.11.2025), table 3 «Berechnung der Grundstückgewinnsteuer». The page is given with each canton; ZG, FR, VS,
BS, BL, SG and VD were also checked against the cantonal law (as of November 2026); ZH (#287) and LU, UR,
SZ, OW, NW (#288) name the law article.

Each rule is an object in the format of `PropertyGainTax` of the engine (camelCase):
- model «zurich»: `brackets` [width, rate], optionally `uniformAbove` [from gain, rate for the whole gain];
- model «aargau»: `durationRates` [from years, rate];
- model «curve»: `rateCurve` [gain, rate];
- model «return»: rate = annual return on the investment costs, `returnBounds` [min, max], `returnMaxDuration`
  [from years, highest rate];
- `shortSurcharge` [threshold, surcharge, years], `longBrackets` [from years, brackets], `durationAbove` [threshold,
  rows like `duration`] for the part of the gain above the threshold;
- `duration` [from years, surcharge or reduction of the tax], `gainDuration` [from years, change of the gain];
- `exemptBelow` exemption limit, `allowance` allowance, `maxShare` maximum burden;
- `useMultiplier`: simple tax × tax multiplier (canton and municipality).

Per canton also `text` (short description, German, shown in the user interface as `propertyGainRule.<canton>`),
`source` (page or law) and, where simplified, `approx` (`propertyGainApprox.<canton>`).
"""

BIG = 1e12  # last bracket without upper limit


def steps(start_year, first, step, last, end_year=None):
    """Rows [year, value]: `first` from `start_year`, `step` for each further year, until `last` is reached."""
    rows, y, v = [], start_year, first
    while True:
        if (step < 0 and v < last) or (step > 0 and v > last):
            v = last
        rows.append([y, round(v, 6)])
        if abs(v - last) < 1e-9 or (end_year is not None and y >= end_year):
            break
        y += 1
        v += step
    return rows


def monthly_surcharge(months, per_month):
    """Surcharge per month by which the holding period is shorter than `months`: rows in years (month / 12)."""
    return [[round(m / 12, 6), round((months - m) * per_month, 6)] for m in range(months)] + [[months / 12, 0.0]]


ZH_BRACKETS = [[4000, 0.10], [6000, 0.15], [8000, 0.20], [12000, 0.25], [20000, 0.30], [50000, 0.35], [BIG, 0.40]]


def rules():
    """The rule per canton id."""
    r = {}
    r["ZH"] = dict(
        model="zurich", brackets=ZH_BRACKETS, exemptBelow=5000,
        duration=[[0, 0.5], [1, 0.25], [2, 0.0]] + steps(5, -0.05, -0.03, -0.50),
        text="Stufentarif 10 % bis 40 %, Zuschlag bis 50 % unter 2 Jahren, Ermässigung bis 50 % ab 20 Jahren",
        source="StG ZH § 225 Abs. 1–3 und 5 (LS 631.1, Stand 1.4.2026)")
    r["BE"] = dict(
        model="zurich", useMultiplier=True, exemptBelow=5300,
        brackets=[[2800, 0.0144], [2800, 0.024], [8200, 0.0408], [13400, 0.0492], [26800, 0.0641], [80400, 0.0726],
                  [201100, 0.0781], [BIG, 0.081]],
        duration=[[0, 0.7], [1, 0.5], [2, 0.35], [3, 0.2], [4, 0.1], [5, 0.0]],
        gainDuration=steps(5, -0.10, -0.02, -0.70),
        text="einfache Steuer 1.44 % bis 8.1 % × Steuerfuss, Zuschlag unter 5 Jahren, Gewinn ab 5 Jahren je Jahr "
             "2 % tiefer (höchstens 70 %)",
        source="S. 9")
    # #288: income tax tariff 2026 (StG LU § 57 Abs. 1) times the fixed tax multiplier of 4.2 units (§ 23 GGStG)
    lu = 4.2
    r["LU"] = dict(
        model="zurich", exemptBelow=13000, maxShare=0.40,
        brackets=[[w, round(p * lu, 6)] for w, p in (
            [9900, 0.0], [2400, 0.005], [3200, 0.01], [1200, 0.02], [1200, 0.03], [2900, 0.04], [4300, 0.045],
            [85000, 0.05], [53800, 0.0525], [26200, 0.055], [BIG, 0.058])],
        uniformAbove=[2096800, round(0.057 * lu, 6)],
        duration=[[0, 0.5], [1, 0.4], [2, 0.3], [3, 0.2], [4, 0.1], [5, 0.0]] + steps(9, -0.01, -0.01, -0.25),
        text="Einkommenssteuertarif (einfache Steuer) × 4.2 (fester Steuerfuss), Zuschlag unter 6 Jahren, Ermässigung "
             "ab 9 Jahren bis 25 %, höchstens 40 % des Gewinns",
        source="GGStG LU §§ 22–24 (SRL 647, Stand 1.1.2025) mit StG LU § 57 Abs. 1 (SRL 620, Stand 1.10.2026)")
    r["UR"] = dict(
        model="aargau", allowance=10000,
        durationRates=[[y, round(0.31 - 0.01 * y, 4)] for y in range(21)],
        text="Satz nach Besitzdauer 31 % bis 11 %, Freibetrag 10 000 Franken",
        source="StG UR Art. 142 (RB 3.2211, Stand 1.1.2026)")
    r["SZ"] = dict(
        model="zurich", allowance=2000,
        brackets=[[3000, 0.08], [5000, 0.12], [7000, 0.16], [10000, 0.20], [15000, 0.25], [BIG, 0.30]],
        duration=[[0, 0.4], [1, 0.3], [2, 0.2], [3, 0.1], [4, 0.0]] + steps(5, -0.10, -0.03, -0.70),
        text="Stufentarif 8 % bis 30 %, Freibetrag 2 000 Franken, Zuschlag unter 4 Jahren, Ermässigung bis 70 %",
        source="StG SZ § 120 (SRSZ 172.200, Stand 1.2.2026)")
    r["OW"] = dict(
        model="zurich", useMultiplier=True, exemptBelow=5000, brackets=[[BIG, 0.02]],
        duration=[[0, 0.3], [1, 0.2], [2, 0.1], [3, 0.0]],
        text="einfache Steuer 2 % × Steuerfuss, Zuschlag unter 3 Jahren",
        source="StG OW Art. 155 und 156 (GDB 641.4, Stand 1.1.2026)")
    nw = [0.36, 0.33, 0.31, 0.29, 0.28, 0.27, 0.26, 0.25, 0.24, 0.23, 0.22]
    nw += [round(0.215 - 0.005 * i, 4) for i in range(19)]  # 11 to 29 years: 21.5 % to 12.5 %
    r["NW"] = dict(
        model="aargau", durationRates=[[y, v] for y, v in enumerate(nw)] + [[30, 0.12]],
        text="Satz nach Besitzdauer 36 % bis 12 %",
        source="StG NW Art. 151 (NG 521.1, Stand 1.1.2026)")
    r["GL"] = dict(
        model="zurich", exemptBelow=5000,
        brackets=[[5000, 0.10], [5000, 0.15], [5000, 0.20], [5000, 0.25], [BIG, 0.30]],
        duration=[[0, 0.3], [1, 0.2], [2, 0.15], [3, 0.1], [4, 0.0]] + steps(5, -0.05, -0.03, -0.35)
        + steps(16, -0.40, -0.05, -0.85) + [[30, -0.90]],
        text="Stufentarif 10 % bis 30 %, Zuschlag unter 4 Jahren, Ermässigung bis 90 % ab 30 Jahren",
        source="S. 13")
    r["ZG"] = dict(
        model="return", returnBounds=[0.10, 0.60], exemptBelow=5000,
        returnMaxDuration=[[0, 0.60]] + [[y, round(0.60 - 0.025 * (y - 11), 4)] for y in range(12, 25)] + [[25, 0.25]],
        text="Satz = Jahresrendite des Gewinns auf den Anlagekosten, mindestens 10 %, höchstens 60 %; der Höchstsatz "
             "sinkt ab 12 Jahren um 2.5 Punkte je Jahr auf 25 %",
        source="StG ZG § 199 (BGS 632.1, Stand 1.1.2026)")
    r["FR"] = dict(
        model="aargau", exemptBelow=6000, shortSurcharge=[400000, 0.40, 5],
        durationRates=[[0, 0.22], [2, 0.20], [4, 0.18], [6, 0.16], [8, 0.14], [10, 0.12], [15, 0.10]],
        text="Satz nach Besitzdauer 22 % bis 10 %, Zuschlag von 40 % auf dem Gewinn über 400 000 Franken bei weniger "
             "als 5 Jahren (je Immobilie gerechnet)",
        source="Art. 51 DStG, RSF 631.1")
    r["SO"] = dict(
        model="zurich", useMultiplier=True,
        brackets=[[12000, 0.0], [4000, 0.045], [4000, 0.05], [3000, 0.065], [2000, 0.08], [3000, 0.09],
                  [11000, 0.095], [15000, 0.10], [44000, 0.105], [BIG, 0.115]],
        uniformAbove=[310000, 0.105],
        duration=steps(6, -0.02, -0.02, -0.50),
        text="Einkommenssteuertarif für Alleinstehende × Steuerfuss, Ermässigung ab 6 Jahren bis 50 %",
        source="S. 14")
    bs = [[0, 0.30]] + [[k - 1, round(0.30 - 0.009 * (k - 5), 4)] for k in range(6, 25)] + [[24, 0.12]]
    r["BS"] = dict(
        model="aargau", exemptBelow=500, durationRates=bs,
        text="Satz nach Besitzdauer für selbstgenutztes Wohneigentum: 30 % bis 12 %",
        approx="Angenommen ist dauernd selbstgenutztes Wohneigentum (übrige Liegenschaften 60 % bis 12 %); die "
               "Ermässigung für wertvermehrende Aufwendungen fehlt.",
        source="S. 15")
    r["BL"] = dict(
        model="curve", rateCurve=[[0, 0.03], [30000, 0.12], [70000, 0.20], [120000, 0.25]],
        duration=monthly_surcharge(60, 1 / 60),
        text="Satz nach Gewinn 3 % bis 25 %, Zuschlag unter 5 Jahren je Monat 1 2/3 %",
        approx="Die Ermässigung für selbstgenutzte Liegenschaften nach 20 Jahren und die Anpassung der Anlagekosten "
               "an die Teuerung fehlen.",
        source="S. 16")
    r["SH"] = dict(
        model="zurich", useMultiplier=True, exemptBelow=5000, maxShare=0.50,
        brackets=[[2000, 0.02], [2000, 0.04], [2000, 0.06], [2000, 0.08], [7000, 0.10], [15000, 0.12], [15000, 0.14],
                  [15000, 0.16], [20000, 0.18], [BIG, 0.20]],
        uniformAbove=[100000, 0.15],
        duration=[[y / 2, round(0.5 - 0.05 * y, 4)] for y in range(11)] + steps(6, -0.05, -0.05, -0.60),
        text="einfache Steuer 2 % bis 20 % × Steuerfuss, Zuschlag unter 5 Jahren, Ermässigung bis 60 %, höchstens "
             "50 % des Gewinns",
        source="S. 17")
    r["AR"] = dict(
        model="zurich", exemptBelow=3000, brackets=[[BIG, 0.30]],
        duration=[[0, 0.5], [0.5, 0.35], [1, 0.2], [2, 0.1], [3, 0.05], [4, 0.0]] + steps(11, -0.025, -0.025, -0.50),
        text="30 % des Gewinns, Zuschlag unter 4 Jahren, Ermässigung ab 11 Jahren bis 50 %",
        source="S. 17")
    r["AI"] = dict(
        model="zurich", exemptBelow=4000, brackets=ZH_BRACKETS,
        duration=monthly_surcharge(36, 0.01) + steps(5, -0.05, -0.03, -0.50),
        text="Stufentarif 10 % bis 40 %, Zuschlag unter 3 Jahren je Monat 1 %, Ermässigung bis 50 %",
        source="S. 18")
    sg = [[2200, 0.0], [2800, 0.005], [2700, 0.01], [2100, 0.02], [2100, 0.03], [2600, 0.04], [3700, 0.05],
          [6900, 0.06], [9600, 0.07], [16000, 0.08], [21100, 0.09], [176200, 0.10], [208000, 0.105], [BIG, 0.11]]
    sg_short = [[0, 0.05], [1, 0.04], [2, 0.03], [3, 0.02], [4, 0.01], [5, 0.0]]
    r["SG"] = dict(
        model="zurich", brackets=[[w, round(v * 3.2, 6)] for w, v in sg], uniformAbove=[600000, 0.32],
        duration=sg_short + steps(16, -0.015, -0.015, -0.405),
        durationAbove=[500000, sg_short + steps(16, -0.01, -0.01, -0.20)],
        text="einfache Steuer 0 % bis 11 % mit Zuschlag des Kantons von 220 %, Zuschlag unter 5 Jahren, Ermässigung "
             "ab 16 Jahren bei Selbstnutzung 1.5 % je Jahr bis 40.5 % (Gewinnanteil bis 500 000 Franken), sonst 1 % "
             "je Jahr bis 20 %",
        approx="Angenommen ist Selbstnutzung während mindestens 15 Jahren; sonst gilt 1 % je Jahr, höchstens 20 %.",
        source="Art. 140 und 141 StG, sGS 811.1")
    r["GR"] = dict(
        model="zurich", exemptBelow=4600,
        brackets=[[9919, round(0.05 + 0.01 * i, 4)] for i in range(21)], uniformAbove=[208299, 0.15],
        duration=monthly_surcharge(24, 0.02) + steps(11, -0.015, -0.015, -0.51),
        text="Stufentarif 5 % bis 25 %, ab 208 299 Franken 15 %, Zuschlag unter 2 Jahren, Ermässigung bis 51 %",
        source="S. 19")
    r["AG"] = dict(
        model="aargau",
        durationRates=[[0, 0.40]] + [[y, round(0.40 - 0.02 * y, 4)] for y in range(1, 11)]
        + [[y, round(0.20 - 0.01 * (y - 10), 4)] for y in range(11, 26)],
        text="Satz nach Besitzdauer 40 % bis 5 %",
        source="S. 19")
    r["TG"] = dict(
        model="zurich", brackets=[[BIG, 0.40]],
        duration=monthly_surcharge(36, 0.01) + steps(6, -0.04, -0.04, -0.72),
        text="40 % des Gewinns, Zuschlag unter 3 Jahren je Monat 1 %, Ermässigung ab dem 6. Jahr bis 72 %",
        source="S. 20")
    r["TI"] = dict(
        model="aargau",
        durationRates=[[0, 0.31], [1, 0.30], [2, 0.29], [3, 0.28], [4, 0.27], [5, 0.26], [6, 0.23], [7, 0.20],
                       [8, 0.17], [9, 0.14], [10, 0.11], [11, 0.10], [12, 0.09], [13, 0.08], [14, 0.07], [15, 0.06],
                       [20, 0.05], [30, 0.04]],
        text="Satz nach Besitzdauer 31 % bis 4 %",
        source="S. 20")
    vd = [[0, 0.30], [1, 0.27], [2, 0.24], [3, 0.22], [4, 0.20], [5, 0.18], [6, 0.17], [7, 0.16], [8, 0.15],
          [10, 0.14], [12, 0.13], [14, 0.12], [16, 0.11], [18, 0.10], [20, 0.09], [22, 0.08], [24, 0.07]]
    r["VD"] = dict(
        model="aargau", exemptBelow=5000, durationRates=[[y / 2, v] for y, v in vd],
        text="Satz nach Besitzdauer 30 % bis 7 %, Jahre der Selbstnutzung zählen doppelt",
        approx="Angenommen ist Selbstnutzung während der ganzen Besitzdauer.",
        source="S. 21")
    r["VS"] = dict(
        model="zurich", brackets=[[50000, 0.12], [50000, 0.18], [BIG, 0.24]],
        longBrackets=[25, [[50000, 0.01], [50000, 0.02], [BIG, 0.03]]],
        duration=[[0, 0.6], [1, 0.5], [2, 0.3], [3, 0.2], [4, 0.1], [5, 0.0]] + steps(6, -0.04, -0.04, -0.76)
        + [[25, 0.0]],
        text="Stufentarif 12 % bis 24 %, Zuschlag in den ersten 5 Jahren, ab dem 6. Jahr 4 % weniger je Jahr, ab "
             "25 Jahren 1 % bis 3 %",
        approx="Angenommen ist ein Stufentarif nach Gewinnhöhe (nicht ein Satz für den ganzen Gewinn).",
        source="Art. 52 StG, SGS 642.1")
    r["NE"] = dict(
        model="zurich",
        brackets=[[5000, 0.10], [5000, 0.15], [20000, 0.20], [20000, 0.25], [25000, 0.30], [25000, 0.35],
                  [BIG, 0.40]],
        uniformAbove=[135000, 0.33],
        duration=[[0, 0.6], [1, 0.45], [2, 0.3], [3, 0.15], [4, 0.0]] + steps(5, -0.06, -0.06, -0.60),
        text="Stufentarif 10 % bis 40 %, ab 135 000 Franken 33 %, Zuschlag unter 4 Jahren, Ermässigung bis 60 %",
        source="S. 22")
    r["GE"] = dict(
        model="aargau",
        durationRates=[[0, 0.50], [2, 0.40], [4, 0.30], [6, 0.20], [8, 0.15], [10, 0.10], [25, 0.02]],
        text="Satz nach Besitzdauer 50 % bis 2 %",
        source="S. 23")
    r["JU"] = dict(
        model="zurich", useMultiplier=True, exemptBelow=4000,
        brackets=[[50000, 0.035], [50000, 0.045], [100000, 0.055], [BIG, 0.06]],
        duration=[[0, 0.5], [2, 0.25], [5, 0.0]] + steps(11, -0.01, -0.01, -0.30) + steps(41, -0.32, -0.02, -0.50),
        text="einfache Steuer 3.5 % bis 6 % × Steuerfuss, Zuschlag unter 5 Jahren, Ermässigung ab 11 Jahren bis 50 %",
        source="S. 23")
    return r
