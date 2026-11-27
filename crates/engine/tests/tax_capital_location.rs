//! #64: Tax on lump sums from pension savings by canton and municipality (mode «Wohnort»).
//!
//! Reference values: tax calculator of the ESTV, tax year 2026, «Kapitalauszahlung aus Vorsorge», age 65, no
//! children, no denomination (with married couples also the second person). The cases are made up. Compared is the
//! sum of federal, cantonal and municipal tax.

use engine::model::{Church, Plan, TaxLocation};
use engine::sim::{SimResult, Simulator};
use engine::tax;
use engine::taxdata;
use engine::time::YearMonth;
use engine::validate::{validate, Severity};
use serde_json::{json, Value};

/// Canton, BFS number, married, lump sum, ESTV: federal, canton, municipality.
type Case = (&'static str, u32, bool, f64, [f64; 3]);

const CASES: [Case; 48] = [
    ("ZH", 261, false, 100_000.0, [537.0, 1900.0, 2380.0]),
    ("ZH", 261, false, 500_000.0, [10501.0, 10906.0, 13661.0]),
    ("ZH", 261, false, 1_000_000.0, [23000.0, 38418.0, 48124.0]),
    ("ZH", 261, true, 100_000.0, [363.0, 1900.0, 2380.0]),
    ("ZH", 261, true, 500_000.0, [10176.0, 9500.0, 11900.0]),
    ("ZH", 261, true, 1_000_000.0, [23000.0, 25593.0, 32059.0]),
    ("BE", 351, false, 100_000.0, [537.0, 2696.0, 1395.0]),
    ("BE", 351, false, 500_000.0, [10501.0, 20267.0, 10491.0]),
    ("BE", 351, false, 1_000_000.0, [23000.0, 48202.0, 24952.0]),
    ("BE", 351, true, 100_000.0, [363.0, 2265.0, 1173.0]),
    ("BE", 351, true, 500_000.0, [10176.0, 18110.0, 9374.0]),
    ("BE", 351, true, 1_000_000.0, [23000.0, 44742.0, 23161.0]),
    ("LU", 1061, false, 100_000.0, [537.0, 1508.0, 1508.0]),
    ("LU", 1061, false, 500_000.0, [10501.0, 9628.0, 9628.0]),
    ("LU", 1061, false, 1_000_000.0, [23000.0, 19778.0, 19778.0]),
    ("LU", 1061, true, 100_000.0, [363.0, 1508.0, 1508.0]),
    ("LU", 1061, true, 500_000.0, [10176.0, 9628.0, 9628.0]),
    ("LU", 1061, true, 1_000_000.0, [23000.0, 19778.0, 19778.0]),
    ("AG", 4001, false, 100_000.0, [537.0, 2144.0, 1998.0]),
    ("AG", 4001, false, 500_000.0, [10501.0, 15216.0, 14182.0]),
    ("AG", 4001, false, 1_000_000.0, [23000.0, 32211.0, 30022.0]),
    ("AG", 4001, true, 100_000.0, [363.0, 1473.0, 1373.0]),
    ("AG", 4001, true, 500_000.0, [10176.0, 13754.0, 12819.0]),
    ("AG", 4001, true, 1_000_000.0, [23000.0, 30432.0, 28363.0]),
    ("SG", 3203, false, 100_000.0, [537.0, 2310.0, 3036.0]),
    ("SG", 3203, false, 500_000.0, [10501.0, 11550.0, 15180.0]),
    ("SG", 3203, false, 1_000_000.0, [23000.0, 23100.0, 30360.0]),
    ("SG", 3203, true, 100_000.0, [363.0, 2100.0, 2760.0]),
    ("SG", 3203, true, 500_000.0, [10176.0, 10500.0, 13800.0]),
    ("SG", 3203, true, 1_000_000.0, [23000.0, 21000.0, 27600.0]),
    ("GE", 6621, false, 100_000.0, [537.0, 2662.0, 926.0]),
    ("GE", 6621, false, 500_000.0, [10501.0, 19699.0, 6851.0]),
    ("GE", 6621, false, 1_000_000.0, [23000.0, 43085.0, 14984.0]),
    ("GE", 6621, true, 100_000.0, [363.0, 1755.0, 610.0]),
    ("GE", 6621, true, 500_000.0, [10176.0, 17289.0, 6013.0]),
    ("GE", 6621, true, 1_000_000.0, [23000.0, 39399.0, 13702.0]),
    ("FR", 2196, false, 100_000.0, [537.0, 1500.0, 1200.0]),
    ("FR", 2196, false, 500_000.0, [10501.0, 20000.0, 16000.0]),
    ("FR", 2196, false, 1_000_000.0, [23000.0, 45000.0, 36000.0]),
    ("FR", 2196, true, 100_000.0, [363.0, 1300.0, 1040.0]),
    ("FR", 2196, true, 500_000.0, [10176.0, 19500.0, 15600.0]),
    ("FR", 2196, true, 1_000_000.0, [23000.0, 44500.0, 35600.0]),
    ("TI", 5002, false, 100_000.0, [537.0, 2000.0, 1860.0]),
    ("TI", 5002, false, 500_000.0, [10501.0, 12871.0, 11970.0]),
    ("TI", 5002, false, 1_000_000.0, [23000.0, 30000.0, 27900.0]),
    ("TI", 5002, true, 100_000.0, [363.0, 2000.0, 1860.0]),
    ("TI", 5002, true, 500_000.0, [10176.0, 10000.0, 9300.0]),
    ("TI", 5002, true, 1_000_000.0, [23000.0, 27989.0, 26030.0]),
];

fn loc(canton: &str, bfs: u32, church: Church) -> TaxLocation {
    TaxLocation {
        canton: canton.into(),
        municipality: Some(bfs),
        church,
    }
}

/// #64/AK4: 100 000, 500 000 and 1 000 000 francs, single and married, in eight cantons: at most ±5 % from the ESTV
/// tax calculator.
#[test]
fn issue_64_ak4_reference_cases_match_estv_within_five_percent() {
    let data = taxdata::builtin();
    let mut cantons = std::collections::BTreeSet::new();
    for (canton, bfs, married, amount, r) in CASES {
        let capital = data.resolve(&loc(canton, bfs, Church::None), married).unwrap().capital;
        let mine = tax::tax(&capital, amount, 1.0);
        let want = r[0] + r[1] + r[2];
        assert!(
            (mine / want - 1.0).abs() <= 0.05,
            "{canton} {} {amount}: {mine} instead of {want}",
            if married { "married" } else { "single" }
        );
        cantons.insert(canton);
    }
    assert!(cantons.len() >= 5);
}

/// #64/AK2: canton × (multiplier canton + church) + municipality × multiplier municipality + federal (one fifth of
/// the tariff, without deductions); the multipliers are those of the income tax.
#[test]
fn issue_64_ak2_formula_with_church_and_one_fifth_federal() {
    let data = taxdata::builtin();
    let zh = data.canton("ZH").unwrap();
    let m = data.municipality(261).unwrap().3;
    for married in [false, true] {
        for (church, ch) in [
            (Church::None, 0.0),
            (Church::Reformed, m[2]),
            (Church::Catholic, m[3]),
            (Church::ChristCatholic, m[4]),
        ] {
            let capital = data.resolve(&loc("ZH", 261, church), married).unwrap().capital;
            let curve = zh.capital.get(married);
            let fed = data.federal_capital.get(married);
            for x in [80_000.0, 400_000.0, 1_500_000.0] {
                let want = tax::rate_at(curve, x) * (m[0] + ch + m[1]) + tax::rate_at(fed, x);
                assert!((tax::rate_at(&capital, x) - want).abs() < 1e-12, "{x}");
            }
        }
    }
    // federal: one fifth of the tariff of the direct federal tax (ESTV: 10 501 at 500 000, single)
    let fed = tax::tax(&data.federal_capital.single, 500_000.0, 1.0);
    assert!((fed - 10_501.0).abs() < 2.0, "federal {fed}");
    // with church tax the lump sum costs more
    let none = data.resolve(&loc("ZH", 261, Church::None), false).unwrap().capital;
    let reformed = data.resolve(&loc("ZH", 261, Church::Reformed), false).unwrap().capital;
    assert!(tax::tax(&reformed, 500_000.0, 1.0) > tax::tax(&none, 500_000.0, 1.0));
    // own municipal curve (Schwyz): municipality × own curve, canton × cantonal curve
    let sz = data.canton("SZ").unwrap();
    let bfs = data.municipalities.iter().find(|x| x.2 == "SZ").unwrap();
    let cg = sz
        .capital_municipal
        .as_ref()
        .expect("Schwyz has its own municipal rule");
    let c = data.resolve(&loc("SZ", bfs.0, Church::None), false).unwrap().capital;
    let x = 300_000.0;
    let want = tax::rate_at(&sz.capital.single, x) * bfs.3[0]
        + tax::rate_at(&cg.single, x) * bfs.3[1]
        + tax::rate_at(&data.federal_capital.single, x);
    assert!((tax::rate_at(&c, x) - want).abs() < 1e-12);
}

/// #64/AK3: All 26 cantons have a rule; only Valais is approximated, with a note in the data file and in the list for
/// the user interface (Zug follows the law since #288).
#[test]
fn issue_64_ak3_every_canton_has_a_rule_and_approximations_are_marked() {
    let data = taxdata::builtin();
    assert_eq!(data.cantons.len(), 26);
    for c in &data.cantons {
        assert!(!c.capital_rule.is_empty(), "{}: rule missing", c.id);
        assert!(c.capital.single.len() > 10 && c.capital.married.len() > 10);
    }
    let out = taxdata::locations();
    let approx: Vec<&str> = out
        .cantons
        .iter()
        .filter(|c| c.capital_approx.is_some())
        .map(|c| c.id.as_str())
        .collect();
    let mut sorted = approx.clone();
    sorted.sort_unstable();
    assert_eq!(sorted, ["VS"]);
    let json = serde_json::to_value(&out).unwrap();
    let vs = json["cantons"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["id"] == "VS")
        .unwrap();
    assert!(vs["capitalRule"].as_str().is_some_and(|s| !s.is_empty()));
    assert!(vs["capitalApprox"].as_str().is_some_and(|s| !s.is_empty()));
    let zh = json["cantons"]
        .as_array()
        .unwrap()
        .iter()
        .find(|c| c["id"] == "ZH")
        .unwrap();
    assert!(zh.get("capitalApprox").is_none());
}

// ---------------------------------------------------------------- simulation

/// Plan with a lump sum of 500 000 (person A) in January 2040; `second`: lump sum of a second person in the same
/// month (`Some(married)`).
fn plan(location: bool, second: Option<bool>, b_lump: f64) -> Value {
    let mut v = json!({
        "project": { "name": "Kapitalbezug" },
        "household": { "persons": [ { "id": "A", "name": "A", "birth": "1975-01" } ] },
        "milestones": [
            { "id": "start", "name": "Start", "kind": "date", "date": "2039-01", "fixed": true },
            { "id": "retire", "name": "Bezug Pensionskasse", "kind": "age", "person": "A", "years": 65 },
            { "id": "ahv", "name": "AHV-Bezug", "kind": "age", "person": "A", "years": 65 },
            { "id": "end", "name": "Ende", "kind": "date", "date": "2041-12", "fixed": true }
        ],
        "pension": { "pk": { "capital": 500_000, "at": "retire", "lumpsum": 1.0, "conversion": 0 },
                     "ahv": { "monthly": 0, "at": "ahv" } },
        "flows": { "income": 0, "spending": 3000 },
        "wealth": { "stocks": 0, "bonds": 0, "cash": 500_000, "stockShare": 0, "reserveMonths": 0 },
        "returns": { "source": "fixed", "fixed": { "stocks": 0, "bonds": 0, "inflation": 0, "saron": 0 } },
        "taxes": { "multiplier": 2.0, "income": [[0, 0.0]], "wealth": [[0, 0.0]], "capital": [[0, 0.1]] }
    });
    if location {
        v["taxes"]["mode"] = json!("location");
        v["taxes"]["location"] = json!({ "canton": "ZH", "municipality": 261 });
    }
    if let Some(married) = second {
        v["household"]["persons"] = json!([
            { "id": "A", "name": "A", "birth": "1975-01" },
            { "id": "B", "name": "B", "birth": "1975-01" }
        ]);
        v["household"]["married"] = json!(married);
        v["partner"] = json!({ "income": 0,
                               "pk": { "capital": b_lump, "at": "retire", "lumpsum": 1.0, "conversion": 0 },
                               "ahv": { "monthly": 0, "at": "ahv" } });
    }
    v
}

fn run(v: &Value) -> SimResult {
    let plan: Plan = serde_json::from_value(v.clone()).unwrap();
    let s = Simulator::new(&plan).unwrap();
    let path = engine::market::fixed_path(&s.fixed, s.start, s.months());
    s.run(&path, true)
}

fn capital_tax_2040(v: &Value) -> f64 {
    run(v)
        .records
        .iter()
        .filter(|x| x.month.year() == 2040)
        .map(|x| x.tax_capital)
        .sum()
}

fn zh(married: bool) -> Vec<(f64, f64)> {
    taxdata::builtin()
        .resolve(&loc("ZH", 261, Church::None), married)
        .unwrap()
        .capital
}

/// #64/AK1, AK5: In the mode «Wohnort» the lump sum is taxed by the place of residence, without own table and
/// multiplier; in the mode «Eigene Tabellen» unchanged (table × multiplier).
#[test]
fn issue_64_ak1_ak5_simulation_uses_location_or_own_table() {
    // own tables: 500 000 × 10 % × multiplier 2
    assert!((capital_tax_2040(&plan(false, None, 0.0)) - 100_000.0).abs() < 0.01);
    assert!((capital_tax_2040(&plan(false, Some(true), 0.0)) - 100_000.0).abs() < 0.01);
    for married in [false, true] {
        let second = married.then_some(true);
        let got = capital_tax_2040(&plan(true, second, 0.0));
        let want = tax::tax(&zh(married), 500_000.0, 1.0);
        assert!((got - want).abs() < 0.02, "married {married}: {got} instead of {want}");
        // ESTV: single 10 501 + 10 906 + 13 661, married 10 176 + 9 500 + 11 900
        let estv = if married { 31_576.0 } else { 35_068.0 };
        assert!((got / estv - 1.0).abs() < 0.01, "married {married}: {got}");
    }
}

/// #64 (addition from #150): Married couples add the lump sums of the household and pay with the tariff for married
/// couples; unmarried couples pay per person with the tariff for single persons.
#[test]
fn issue_64_unmarried_couple_pays_per_person_married_per_household() {
    let (a, b) = (500_000.0, 300_000.0);
    let married = capital_tax_2040(&plan(true, Some(true), b));
    let want = tax::tax(&zh(true), a + b, 1.0);
    assert!((married - want).abs() < 0.05, "married: {married} instead of {want}");
    let unmarried = capital_tax_2040(&plan(true, Some(false), b));
    let single = zh(false);
    let want = tax::tax(&single, a, 1.0) + tax::tax(&single, b, 1.0);
    assert!(
        (unmarried - want).abs() < 0.05,
        "unmarried: {unmarried} instead of {want}"
    );
    // progression: per person is cheaper than the sum with the tariff for single persons
    assert!(unmarried < tax::tax(&single, a + b, 1.0));
}

/// #64 with #66: With «Tarife mit der Teuerung» the amounts of the curve grow with the price index of the month of
/// the payout; the rates stay the same.
#[test]
fn issue_64_tariffs_follow_inflation() {
    let mut v = plan(true, None, 0.0);
    v["returns"]["fixed"]["inflation"] = json!(0.03);
    v["taxes"]["indexed"] = json!(true);
    let r = run(&v);
    let m = r.records.iter().find(|x| x.month == YearMonth::new(2040, 1)).unwrap();
    assert!(m.price_index > 1.02, "index {}", m.price_index);
    let table = zh(false);
    let want = tax::tax_indexed(&table, 500_000.0, 1.0, m.price_index);
    assert!(
        (m.tax_capital - want).abs() < 0.02,
        "{} instead of {want}",
        m.tax_capital
    );
    assert!(m.tax_capital < tax::tax(&table, 500_000.0, 1.0));
}

/// #64/AK1: In the mode «Wohnort» the own lump-sum tables are not checked (hidden, kept in the file); in the mode
/// «Eigene Tabellen» an invalid rate is still an error.
#[test]
fn issue_64_ak1_own_capital_table_not_checked_in_location_mode() {
    let mut v = plan(true, Some(true), 0.0);
    v["taxes"]["capital"] = json!([[0, 2.0]]);
    v["taxes"]["couple"] = json!({ "income": [[0, 0.0]], "wealth": [[0, 0.0]], "capital": [[0, 2.0]] });
    let errors = |v: &Value| {
        let plan: Plan = serde_json::from_value(v.clone()).unwrap();
        validate(&plan)
            .into_iter()
            .filter(|i| i.severity == Severity::Error && i.field.as_deref().is_some_and(|f| f.contains("capital")))
            .count()
    };
    assert_eq!(errors(&v), 0);
    v["taxes"]["mode"] = json!("own");
    assert_eq!(errors(&v), 2);
}

/// #288/AK4: Zug, simple cantonal tax on lump sums (StG ZG § 37 Abs. 2: 30 % of the tariff tax on the first 230 600
/// francs, 40 % on the rest, rate at least 1 %) against the tax calculator of the canton of Zug, 2026, 26.11.2026:
/// married, lump sum, simple tax. Form «Kapitalabfindungen aus Vorsorge» on steuern.zg.ch. «check-english: allow»
const ZG_CALCULATOR: [(bool, f64, f64); 10] = [
    (false, 30_000.0, 300.00),
    (false, 60_000.0, 765.30),
    (false, 100_000.0, 1_690.35),
    (false, 500_000.0, 13_655.55),
    (false, 2_000_000.0, 61_655.55),
    (false, 5_000_000.0, 157_655.55),
    (true, 50_000.0, 500.00),
    (true, 200_000.0, 3_380.70),
    (true, 300_000.0, 6_834.70),
    (true, 1_000_000.0, 29_234.70),
];

#[test]
fn issue_288_ak4_zug_lump_sums_match_the_cantonal_calculator() {
    let zg = taxdata::builtin().canton("ZG").unwrap().clone();
    assert!(zg.capital_approx.is_none());
    for (married, amount, want) in ZG_CALCULATOR {
        let mine = tax::tax(zg.capital.get(married), amount, 1.0);
        assert!(
            (mine / want - 1.0).abs() <= 0.005,
            "{} {amount}: {mine} instead of {want}",
            if married { "married" } else { "single" }
        );
    }
}

/// #296/AK4: Lump sums in Sion from the tax calculator of the canton of Valais 2026 (single, 65 years, male):
/// `(amount, canton, municipality)`.
const VS_SION_CALCULATOR: [(f64, f64, f64); 3] = [
    (200_000.0, 4_009.60, 4_400.00),
    (500_000.0, 16_342.00, 17_073.65),
    (1_000_000.0, 40_000.00, 40_000.00),
];

/// #296/AK4: Cantonal and municipal tax on lump sums in Sion match the cantonal calculator within ±5 % (the
/// resolved curve also holds one fifth of the federal tariff, which is subtracted here).
#[test]
fn issue_296_ak4_valais_lump_sums_match_the_cantonal_calculator() {
    let data = taxdata::builtin();
    let capital = data.resolve(&loc("VS", 6266, Church::None), false).unwrap().capital;
    for (amount, canton, municipality) in VS_SION_CALCULATOR {
        let federal = tax::tax(data.federal_capital.get(false), amount, 1.0);
        let mine = tax::tax(&capital, amount, 1.0) - federal;
        let want = canton + municipality;
        assert!((mine / want - 1.0).abs() <= 0.05, "{amount}: {mine} instead of {want}");
    }
}
