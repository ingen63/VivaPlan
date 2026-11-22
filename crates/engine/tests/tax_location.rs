//! #63: Income and wealth tax by canton and municipality (mode «Wohnort»).
//!
//! The reference values come from the tax calculator of the ESTV, tax year 2026 (income and wealth tax, pension
//! income, age 60, no children, no denomination; with married couples the second person has no income). The cases
//! are made up. Compared are federal, cantonal and municipal tax; poll taxes are not part of the income tax.
//! Reference values for working persons are still missing (#63/AK6, open part).

use engine::model::{Church, Plan, TaxLocation, TaxMode};
use engine::sim::{MonthRecord, SimResult, Simulator};
use engine::tax;
use engine::taxdata::{self, combine};
use engine::time::YearMonth;
use engine::validate::{validate, Severity};
use serde_json::{json, Value};

/// Canton, BFS number, married, income, wealth, ESTV: federal, canton, municipality (income),
/// canton, municipality (wealth).
type Case = (&'static str, u32, bool, f64, f64, [f64; 5]);

const CASES: [Case; 25] = [
    (
        "ZH",
        261,
        false,
        80_000.0,
        500_000.0,
        [1218.0, 3783.0, 4739.0, 284.0, 355.0],
    ),
    (
        "ZH",
        261,
        false,
        150_000.0,
        1_200_000.0,
        [6779.0, 10109.0, 12663.0, 1174.0, 1470.0],
    ),
    (
        "ZH",
        261,
        false,
        300_000.0,
        4_000_000.0,
        [25747.0, 26991.0, 33810.0, 7532.0, 9435.0],
    ),
    (
        "ZH",
        261,
        true,
        100_000.0,
        300_000.0,
        [1429.0, 3892.0, 4875.0, 66.0, 83.0],
    ),
    (
        "ZH",
        261,
        true,
        220_000.0,
        2_000_000.0,
        [13388.0, 14073.0, 17629.0, 2461.0, 3082.0],
    ),
    (
        "BE",
        351,
        false,
        80_000.0,
        500_000.0,
        [1218.0, 8458.0, 4378.0, 1052.0, 544.0],
    ),
    (
        "BE",
        351,
        false,
        150_000.0,
        1_200_000.0,
        [6779.0, 19920.0, 10311.0, 3367.0, 1743.0],
    ),
    (
        "BE",
        351,
        false,
        300_000.0,
        4_000_000.0,
        [25747.0, 47322.0, 24496.0, 14190.0, 7345.0],
    ),
    (
        "BE",
        351,
        true,
        100_000.0,
        300_000.0,
        [1429.0, 8851.0, 4582.0, 495.0, 256.0],
    ),
    (
        "BE",
        351,
        true,
        220_000.0,
        2_000_000.0,
        [13388.0, 28040.0, 14515.0, 6344.0, 3284.0],
    ),
    (
        "LU",
        1061,
        false,
        80_000.0,
        500_000.0,
        [1218.0, 4341.0, 4341.0, 475.0, 475.0],
    ),
    (
        "LU",
        1061,
        false,
        150_000.0,
        1_200_000.0,
        [6779.0, 9548.0, 9548.0, 1236.0, 1236.0],
    ),
    (
        "LU",
        1061,
        false,
        300_000.0,
        4_000_000.0,
        [25747.0, 21912.0, 21912.0, 4281.0, 4281.0],
    ),
    (
        "LU",
        1061,
        true,
        100_000.0,
        300_000.0,
        [1429.0, 4367.0, 4367.0, 190.0, 190.0],
    ),
    (
        "LU",
        1061,
        true,
        220_000.0,
        2_000_000.0,
        [13388.0, 13804.0, 13804.0, 2039.0, 2039.0],
    ),
    (
        "AG",
        4001,
        false,
        80_000.0,
        500_000.0,
        [1218.0, 4817.0, 4490.0, 390.0, 364.0],
    ),
    (
        "AG",
        4001,
        false,
        150_000.0,
        1_200_000.0,
        [6779.0, 11853.0, 11047.0, 1532.0, 1428.0],
    ),
    (
        "AG",
        4001,
        false,
        300_000.0,
        4_000_000.0,
        [25747.0, 27920.0, 26022.0, 6146.0, 5729.0],
    ),
    (
        "AG",
        4001,
        true,
        100_000.0,
        300_000.0,
        [1429.0, 4285.0, 3994.0, 29.0, 27.0],
    ),
    (
        "AG",
        4001,
        true,
        220_000.0,
        2_000_000.0,
        [13388.0, 15506.0, 14452.0, 2636.0, 2457.0],
    ),
    (
        "SG",
        3203,
        false,
        80_000.0,
        500_000.0,
        [1218.0, 5063.0, 6654.0, 759.0, 997.0],
    ),
    (
        "SG",
        3203,
        false,
        150_000.0,
        1_200_000.0,
        [6779.0, 11925.0, 15673.0, 2008.0, 2639.0],
    ),
    (
        "SG",
        3203,
        false,
        300_000.0,
        4_000_000.0,
        [25747.0, 26431.0, 34738.0, 7006.0, 9208.0],
    ),
    (
        "SG",
        3203,
        true,
        100_000.0,
        300_000.0,
        [1429.0, 4675.0, 6145.0, 268.0, 352.0],
    ),
    (
        "SG",
        3203,
        true,
        220_000.0,
        2_000_000.0,
        [13388.0, 15941.0, 20952.0, 3302.0, 4340.0],
    ),
];

fn loc(canton: &str, bfs: u32, church: Church) -> TaxLocation {
    TaxLocation {
        canton: canton.into(),
        municipality: Some(bfs),
        church,
    }
}

/// #63/AK6: Five cases each in five cantons deviate at most ±5 % from the ESTV tax calculator (income and wealth
/// separately), for retired persons with the standard deductions of the canton (variant «without contributions to
/// the 2nd and 3rd pillar»). Same cases as in VivaPlan-Private (there at most 0.6 %).
#[test]
fn issue_63_ak6_reference_cases_retired_match_estv_within_five_percent() {
    let data = taxdata::builtin();
    let mut worst: f64 = 0.0;
    for (canton, bfs, married, income, wealth, r) in CASES {
        let res = data.resolve(&loc(canton, bfs, Church::None), married).unwrap();
        let (my_i, my_w) = (res.income_tax(income, &[], 1.0), res.wealth_tax(wealth, 1.0));
        let (ref_i, ref_w) = (r[0] + r[1] + r[2], r[3] + r[4]);
        let what = format!(
            "{canton} {} {income}/{wealth}",
            if married { "married" } else { "single" }
        );
        assert!(
            (my_i / ref_i - 1.0).abs() <= 0.05,
            "{what}: income {my_i} instead of {ref_i}"
        );
        assert!(
            (my_w / ref_w - 1.0).abs() <= 0.05,
            "{what}: wealth {my_w} instead of {ref_w}"
        );
        worst = worst.max((my_i / ref_i - 1.0).abs()).max((my_w / ref_w - 1.0).abs());
    }
    // Regression: as precise as in Private (deductions as rules instead of included in the curves)
    assert!(worst < 0.01, "largest deviation {worst}");
}

/// #63/AK2: All 26 cantons and all municipalities of the tax data file are selectable and compute.
#[test]
fn issue_63_ak2_all_cantons_and_municipalities_resolve() {
    let data = taxdata::builtin();
    assert_eq!(data.year, 2026);
    let out = taxdata::locations();
    assert_eq!(out.cantons.len(), 26);
    let n: usize = out.cantons.iter().map(|c| c.municipalities.len()).sum();
    assert_eq!(n, data.municipalities.len());
    assert!(n > 2000, "municipalities: {n}");
    for c in &out.cantons {
        assert!(!c.municipalities.is_empty(), "{} without municipalities", c.id);
        for (bfs, name, _, _) in &c.municipalities {
            for married in [false, true] {
                let r = data
                    .resolve(&loc(&c.id, *bfs, Church::Reformed), married)
                    .unwrap_or_else(|e| panic!("{name}: {e}"));
                let (i, w) = (
                    r.income_tax(100_000.0, &[100_000.0], 1.0),
                    r.wealth_tax(1_000_000.0, 1.0),
                );
                assert!(i > 0.0 && i < 50_000.0, "{name}: income tax {i}");
                assert!((0.0..20_000.0).contains(&w), "{name}: wealth tax {w}");
            }
        }
    }
}

/// #63/AK3: Taxable income = income − standard deductions, never negative; variant «with contributions to the 2nd
/// and 3rd pillar» and flat deduction for professional expenses with earned income; canton and federal tax with
/// their own deductions. Example Zurich: professional expenses 3 %, at least 2000, at most 4000.
#[test]
fn issue_63_ak3_taxable_income_with_standard_deductions() {
    let data = taxdata::builtin();
    let r = data.resolve(&loc("ZH", 261, Church::None), false).unwrap();
    let cd = &r.canton_deductions;
    // Example of the issue: 100 000 earned income → 3000 professional expenses
    assert_eq!(cd.earned.earned_amount(100_000.0, 1.0), 3000.0);
    assert_eq!(cd.earned.earned_amount(30_000.0, 1.0), 2000.0);
    assert_eq!(cd.earned.earned_amount(500_000.0, 1.0), 4000.0);
    assert_eq!(cd.earned.earned_amount(0.0, 1.0), 0.0);
    // Zurich, single: insurance premiums 2900 (with 2nd/3rd pillar) or 4350 (without)
    let (c, f) = r.taxable(100_000.0, &[100_000.0], 1.0);
    assert_eq!(c, 100_000.0 - 2900.0 - 3000.0);
    // federal tax: 1800 with, 2700 without contributions; professional expenses 3 %, 2000 to 4000
    assert_eq!(f, 100_000.0 - 1800.0 - 3000.0);
    let (c, f) = r.taxable(100_000.0, &[], 1.0);
    assert_eq!((c, f), (100_000.0 - 4350.0, 100_000.0 - 2700.0));
    // never negative
    assert_eq!(r.taxable(1000.0, &[], 1.0), (0.0, 0.0));
    assert_eq!(r.income_tax(1000.0, &[], 1.0), 0.0);
    // Married, both working: professional expenses per person; married deductions of the canton
    let m = data.resolve(&loc("ZH", 261, Church::None), true).unwrap();
    let (c, _) = m.taxable(150_000.0, &[100_000.0, 50_000.0], 1.0);
    assert_eq!(c, 150_000.0 - 5800.0 - 3000.0 - 2000.0);
    // canton BL: only a maximum is known (fixed deduction, marked), at most the earned income
    let bl = data.canton("BL").unwrap();
    assert!(bl.deductions.earned.approx && !data.canton("ZH").unwrap().deductions.earned.approx);
    assert_eq!(bl.deductions.earned.rule.earned_amount(300.0, 1.0), 300.0);
    // The deductions lower the tax: working with the same income pays less than without deductions
    assert!(r.income_tax(100_000.0, &[100_000.0], 1.0) < r.income_tax(100_000.0, &[], 1.0));
}

/// #63/AK4: Income tax = cantonal curve × (multiplier canton + church) + municipal curve × multiplier municipality
/// + federal curve, each on its own taxable income; wealth tax = curve × (canton + municipality + church).
#[test]
fn issue_63_ak4_formula_with_multipliers_church_and_federal() {
    let data = taxdata::builtin();
    let zh = data.canton("ZH").unwrap();
    let m = data.municipality(261).unwrap();
    let (ek, vm) = (m.3, m.4);
    for church in [Church::None, Church::Reformed, Church::Catholic, Church::ChristCatholic] {
        let ch = |x: &[f64; 5]| match church {
            Church::None => 0.0,
            Church::Reformed => x[2],
            Church::Catholic => x[3],
            Church::ChristCatholic => x[4],
        };
        let r = data.resolve(&loc("ZH", 261, church), false).unwrap();
        for x in [30_000.0, 87_654.0, 250_000.0, 2_000_000.0] {
            let (tc, tf) = r.taxable(x, &[x], 1.0);
            let want = tc * tax::rate_at(&zh.income.single, tc) * (ek[0] + ek[1] + ch(&ek))
                + tf * tax::rate_at(&data.federal.single, tf);
            assert!((r.income_tax(x, &[x], 1.0) - want).abs() < 0.02, "income {x}");
            let want = x * 10.0 * tax::rate_at(&zh.wealth.single, x * 10.0) * (vm[0] + vm[1] + ch(&vm));
            assert!((r.wealth_tax(x * 10.0, 1.0) - want).abs() < 0.02, "wealth {x}");
        }
    }
    // With church tax the tax is higher
    let none = data.resolve(&loc("ZH", 261, Church::None), false).unwrap();
    let reformed = data.resolve(&loc("ZH", 261, Church::Reformed), false).unwrap();
    assert!(reformed.income_tax(100_000.0, &[], 1.0) > none.income_tax(100_000.0, &[], 1.0));
    // Own municipal curve (Schwyz): the municipal multiplier applies to the municipal curve
    let sz = data.canton("SZ").unwrap();
    let gem = sz.municipal_income.as_ref().expect("municipal curve of Schwyz");
    let mu = data.municipalities.iter().find(|m| m.2 == "SZ").unwrap();
    let r = data.resolve(&loc("SZ", mu.0, Church::None), true).unwrap();
    let x = 120_000.0;
    let want = tax::rate_at(&sz.income.married, x) * mu.3[0] + tax::rate_at(&gem.married, x) * mu.3[1];
    assert!((tax::rate_at(&r.cantonal, x) - want).abs() < 1e-12);
    assert!(data.canton("ZH").unwrap().municipal_income.is_none());
}

#[test]
fn combine_is_exact_between_points() {
    let a = [(0.0, 0.0), (100.0, 0.1), (300.0, 0.2)];
    let b = [(0.0, 0.05), (200.0, 0.01)];
    let c = combine(&[(&a, 1.5), (&b, 1.0)]);
    for x in [0.0, 50.0, 150.0, 250.0, 1000.0] {
        let want = 1.5 * tax::rate_at(&a, x) + tax::rate_at(&b, x);
        assert!((tax::rate_at(&c, x) - want).abs() < 1e-12, "{x}");
    }
}

// ---------------------------------------------------------------- simulation and validation

fn base(married: bool) -> Value {
    let mut v = json!({
        "project": { "name": "Wohnort" },
        "household": { "persons": [ { "id": "A", "name": "A", "birth": "1980-01" } ] },
        "milestones": [
            { "id": "start", "name": "Start", "kind": "date", "date": "2040-01", "fixed": true },
            { "id": "retire", "name": "Bezug Pensionskasse", "kind": "age", "person": "A", "years": 65 },
            { "id": "ahv", "name": "AHV-Bezug", "kind": "age", "person": "A", "years": 65 },
            { "id": "end", "name": "Ende", "kind": "date", "date": "2042-12", "fixed": true }
        ],
        "pension": { "pk": { "capital": 0, "at": "retire", "lumpsum": 0, "conversion": 0 },
                     "ahv": { "monthly": 0, "at": "ahv" } },
        "flows": { "income": 10_000, "spending": 4000 },
        "wealth": { "stocks": 0, "bonds": 0, "cash": 1_000_000, "stockShare": 0, "reserveMonths": 0 },
        "returns": { "source": "fixed", "fixed": { "stocks": 0, "bonds": 0, "inflation": 0, "saron": 0 } },
        "taxes": { "multiplier": 2.0,
                   "income": [[0, 0.5]], "wealth": [[0, 0.5]], "capital": [[0, 0.0]],
                   "mode": "location", "location": { "canton": "ZH", "municipality": 261, "church": "reformed" } }
    });
    if married {
        v["household"]["persons"] = json!([
            { "id": "A", "name": "A", "birth": "1980-01" },
            { "id": "B", "name": "B", "birth": "1980-01" }
        ]);
        v["household"]["married"] = json!(true);
        v["partner"] = json!({ "income": 0, "pk": { "capital": 0, "at": "retire", "lumpsum": 0, "conversion": 0 },
                               "ahv": { "monthly": 0, "at": "ahv" } });
    }
    v
}

fn run(v: &Value) -> SimResult {
    let plan: Plan = serde_json::from_value(v.clone()).expect("plan readable");
    let s = Simulator::new(&plan).expect("plan valid");
    let path = engine::market::fixed_path(&s.fixed, s.start, s.months());
    s.run(&path, true)
}

fn rec(r: &SimResult, y: i32, m: u32) -> &MonthRecord {
    r.records.iter().find(|x| x.month == YearMonth::new(y, m)).unwrap()
}

/// #63/AK3, AK4, AK5: The simulation computes in the mode «Wohnort» with the place of residence, the earned income
/// of the year and the standard deductions, without the own tables and multiplier; married couples (second person
/// and `household.married`) with the tariffs and deductions for married couples.
#[test]
fn issue_63_ak5_simulation_uses_location_and_married_tariffs() {
    let data = taxdata::builtin();
    for married in [false, true] {
        let r = run(&base(married));
        let year: f64 = r
            .records
            .iter()
            .filter(|x| x.month.year() == 2040)
            .map(|x| x.earned)
            .sum();
        assert!((year - 120_000.0).abs() < 1e-6);
        let res = data.resolve(&loc("ZH", 261, Church::Reformed), married).unwrap();
        let earned = if married { vec![year, 0.0] } else { vec![year] };
        let got = rec(&r, 2041, 5).tax_income;
        assert!(
            (got - res.income_tax(year, &earned, 1.0)).abs() < 0.02,
            "married {married}: {got}"
        );
        assert!(got < 0.5 * year, "own table (50 % × 2) does not apply");
        let wealth = rec(&r, 2041, 5).tax_wealth;
        assert!(wealth > 0.0 && wealth < 10_000.0, "wealth tax {wealth}");
    }
    let (s, m) = (run(&base(false)), run(&base(true)));
    assert!(
        rec(&m, 2041, 5).tax_income < rec(&s, 2041, 5).tax_income,
        "tariff for married couples is lower"
    );
    // Unmarried couple: tariffs for single persons
    let mut v = base(true);
    v["household"]["married"] = json!(false);
    assert!((rec(&run(&v), 2041, 5).tax_income - rec(&s, 2041, 5).tax_income).abs() < 0.02);
}

/// #66/AK3 in the mode «Wohnort»: with «tariffs with inflation» the amounts of the tariffs and of the deductions
/// grow with the price index. An income indexed to inflation then pays a tax that grows exactly with inflation.
#[test]
fn issue_66_ak3_location_mode_follows_inflation() {
    let data = taxdata::builtin();
    let res = data.resolve(&loc("ZH", 261, Church::Reformed), false).unwrap();
    let idx = 1.02_f64.powi(10);
    for (income, earned) in [
        (100_000.0, vec![100_000.0]),
        (60_000.0, vec![]),
        (300_000.0, vec![200_000.0]),
    ] {
        let scaled: Vec<f64> = earned.iter().map(|e| e * idx).collect();
        let t0 = res.income_tax(income, &earned, 1.0);
        let t10 = res.income_tax(income * idx, &scaled, idx);
        assert!((t10 / idx - t0).abs() < 0.05, "income {income}: {t10} / {idx} vs {t0}");
        let w0 = res.wealth_tax(income * 10.0, 1.0);
        assert!((res.wealth_tax(income * 10.0 * idx, idx) / idx - w0).abs() < 0.05);
        // without the index the tax grows faster (bracket creep)
        assert!(res.income_tax(income * idx, &scaled, 1.0) > t10 + 1.0);
    }
    // In the simulation: income and tariffs with 2 % inflation, the tax of the second year is the first × 1.02
    let mut v = base(false);
    v["flows"]["incomeIndexed"] = json!(true);
    v["returns"]["fixed"]["inflation"] = json!(0.02);
    v["taxes"]["indexed"] = json!(true);
    let r = run(&v);
    let (t1, t2) = (rec(&r, 2041, 5).tax_income, rec(&r, 2042, 5).tax_income);
    assert!((t2 / t1 - 1.02).abs() < 1e-3, "{t1} → {t2}");
    v["taxes"]["indexed"] = json!(false);
    let r = run(&v);
    assert!(rec(&r, 2042, 5).tax_income / rec(&r, 2041, 5).tax_income > 1.021);
}

/// #63/AK7: Existing plans without a mode compute with their own tables; mode and place of residence are only
/// written when set. The bundled plans open in the mode «Eigene Tabellen».
#[test]
fn issue_63_ak7_existing_plans_keep_own_tables() {
    let mut v = base(false);
    let t = v["taxes"].as_object_mut().unwrap();
    t.remove("mode");
    t.remove("location");
    let plan: Plan = serde_json::from_value(v.clone()).unwrap();
    assert_eq!(plan.taxes.mode, TaxMode::Own);
    let out = serde_json::to_value(&plan).unwrap();
    assert!(out["taxes"].get("mode").is_none() && out["taxes"].get("location").is_none());
    // own table 50 % × multiplier 2 = the whole income
    let r = run(&v);
    assert!((rec(&r, 2041, 5).tax_income - 120_000.0).abs() < 0.01);
    for name in ["new.vplan", "example.vplan"] {
        let text = std::fs::read_to_string(format!("{}/assets/{name}", env!("CARGO_MANIFEST_DIR"))).unwrap();
        let p: Plan = serde_json::from_str(&text).unwrap();
        assert_eq!(p.taxes.mode, TaxMode::Own, "{name}");
    }
    // the mode «Wohnort» is written and read again
    let plan: Plan = serde_json::from_value(base(false)).unwrap();
    let out = serde_json::to_value(&plan).unwrap();
    assert_eq!(out["taxes"]["mode"], "location");
    assert_eq!(
        out["taxes"]["location"],
        json!({ "canton": "ZH", "municipality": 261, "church": "reformed" })
    );
}

/// #63/AK8: If the canton is missing or the municipality does not belong to the canton in the mode «Wohnort», the
/// validation reports an error in the section «Steuern».
#[test]
fn issue_63_ak8_missing_canton_or_wrong_municipality_is_an_error() {
    let errors = |v: &Value| -> Vec<(&'static str, Option<String>)> {
        let plan: Plan = serde_json::from_value(v.clone()).unwrap();
        validate(&plan)
            .into_iter()
            .filter(|i| i.severity == Severity::Error && i.section == "taxes")
            .map(|i| (i.message.code(), i.field))
            .collect()
    };
    assert!(errors(&base(false)).is_empty());
    let mut v = base(false);
    // own tables with invalid rates are not checked in the mode «Wohnort» (hidden)
    v["taxes"]["income"] = json!([[0, 5.0]]);
    assert!(errors(&v).is_empty());

    v["taxes"]["location"] = json!({ "canton": "", "church": "none" });
    let e = errors(&v);
    assert_eq!(e, vec![("tax_canton_missing", Some("taxes.location.canton".into()))]);
    v["taxes"]["location"] = json!({ "canton": "XY" });
    assert_eq!(
        errors(&v),
        vec![("tax_canton_unknown", Some("taxes.location.canton".into()))]
    );
    v["taxes"]["location"] = json!({ "canton": "ZH" });
    assert_eq!(
        errors(&v),
        vec![("tax_municipality_missing", Some("taxes.location.municipality".into()))]
    );
    // municipality of another canton (Bern in the canton of Zurich), unknown number
    for bfs in [351, 999_999] {
        v["taxes"]["location"] = json!({ "canton": "ZH", "municipality": bfs });
        assert_eq!(
            errors(&v),
            vec![(
                "tax_municipality_not_in_canton",
                Some("taxes.location.municipality".into())
            )]
        );
        let plan: Plan = serde_json::from_value(v.clone()).unwrap();
        assert!(Simulator::new(&plan).is_err());
    }
    v["taxes"].as_object_mut().unwrap().remove("location");
    assert_eq!(errors(&v)[0].1.as_deref(), Some("taxes.location.canton"));
    // The simulation rejects the plan
    let plan: Plan = serde_json::from_value(v).unwrap();
    assert!(Simulator::new(&plan).is_err());
}

/// #63/AK8: `TaxData::resolve` reports the same errors as the validation (unknown canton, missing municipality,
/// municipality of another canton).
#[test]
fn issue_63_ak8_resolve_rejects_incomplete_location() {
    let data = taxdata::builtin();
    let code = |l: TaxLocation| data.resolve(&l, false).unwrap_err().message().code();
    assert_eq!(code(loc("XY", 261, Church::None)), "tax_canton_unknown");
    let mut l = loc("ZH", 261, Church::None);
    l.municipality = None;
    assert_eq!(code(l), "tax_municipality_missing");
    assert_eq!(code(loc("ZH", 351, Church::None)), "tax_municipality_not_in_canton");
}

/// #63/AK3: The estimated income tax in the need of the 3-bucket strategy uses the place of residence with the
/// earned income per person (variant «working», professional expenses), extrapolated to the year.
#[test]
fn issue_63_ak3_bucket_need_uses_location_tax_with_earned_income() {
    let mut v = base(false);
    v["household"]["persons"][0]["birth"] = json!("1975-01");
    v["flows"]["income"] = json!(12_000);
    v["pension"]["pk"] = json!({ "capital": 500_000, "at": "retire", "lumpsum": 0, "conversion": 0.06 });
    v["withdrawal"] =
        json!({ "strategy": "three_buckets", "buckets": { "cashYears": 2, "bondYears": 1, "refillThreshold": -1 } });
    // start in July: the need of the start extrapolates six months to the year
    v["milestones"][0]["date"] = json!("2040-07");
    let r = run(&v);
    let b = r.bucket_refills.first().expect("refill at the start").basis.clone();
    assert_eq!(b.months, 6);
    assert!((b.earned - 144_000.0).abs() < 0.01, "earned {}", b.earned);
    let res = taxdata::builtin()
        .resolve(&loc("ZH", 261, Church::Reformed), false)
        .unwrap();
    let income = b.earned + b.ahv + b.pk;
    assert!(
        (b.income_tax - res.income_tax(income, &[b.earned], 1.0)).abs() < 0.02,
        "{}",
        b.income_tax
    );
    // without the earned income per person the deductions would be those of a retired person
    assert!((b.income_tax - res.income_tax(income, &[], 1.0)).abs() > 10.0);
}
