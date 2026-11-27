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
use engine::validate::{validate, validate_in, Severity};
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
// slow: computes the taxes of every municipality of Switzerland
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
    // Married, both working: professional expenses per person; married deductions of the canton; second-earner
    // deduction of Zurich 6200
    let m = data.resolve(&loc("ZH", 261, Church::None), true).unwrap();
    let (c, _) = m.taxable(150_000.0, &[100_000.0, 50_000.0], 1.0);
    assert_eq!(c, 150_000.0 - 5800.0 - 3000.0 - 2000.0 - 6200.0);
    // canton BL: only a maximum is known (fixed deduction, marked), at most the earned income
    let bl = data.canton("BL").unwrap();
    assert!(bl.deductions.earned.approx && !data.canton("ZH").unwrap().deductions.earned.approx);
    assert_eq!(bl.deductions.earned.rule.earned_amount(300.0, 1.0), 300.0);
    // The deductions lower the tax: working with the same income pays less than without deductions
    assert!(r.income_tax(100_000.0, &[100_000.0], 1.0) < r.income_tax(100_000.0, &[], 1.0));
}

/// #63/AK3: Second-earner deduction for married couples with two earned incomes, as in the ESTV tax calculator 2026
/// (23.11.2026): reference is the lower earned income after the professional expenses, at most that income; Bern 2 %
/// of the total earned income. Hand values from the calculator: net wages 54 243 and 18 640 (GL 3600 = minimum, AR
/// 2500 = minimum, federal 8600 = minimum); 54 243 and 2 796 (NE 25 % of 796 = 199, federal capped at 796);
/// 103 117 and 71 883 (BE 2 % of 175 000 = 3500, federal 14 100 = maximum, ZH 6200 fixed).
#[test]
fn issue_63_ak3_second_earner_deduction_as_estv() {
    let data = taxdata::builtin();
    let at = |canton: &str, bfs: u32, earned: &[f64]| {
        let r = data.resolve(&loc(canton, bfs, Church::None), true).unwrap();
        (
            r.canton_deductions.second_earner_amount(earned, 1.0).round(),
            r.federal_deductions.second_earner_amount(earned, 1.0).round(),
        )
    };
    assert_eq!(at("GL", 1632, &[54_243.0, 18_640.0]), (3600.0, 8600.0));
    assert_eq!(at("AR", 3001, &[54_243.0, 18_640.0]), (2500.0, 8600.0));
    assert_eq!(at("NE", 6458, &[54_243.0, 2_796.0]), (199.0, 796.0));
    assert_eq!(at("BE", 351, &[103_117.0, 71_883.0]), (3500.0, 14_100.0));
    assert_eq!(at("ZH", 261, &[103_117.0, 71_883.0]), (6200.0, 14_100.0));
    // Only with two earned incomes, only for married couples; Thurgau has none
    assert_eq!(at("ZH", 261, &[134_539.0, 0.0]), (0.0, 0.0));
    let single = data.resolve(&loc("ZH", 261, Church::None), false).unwrap();
    assert_eq!(
        single
            .canton_deductions
            .second_earner_amount(&[100_000.0, 50_000.0], 1.0),
        0.0
    );
    assert_eq!(at("TG", 4566, &[103_117.0, 71_883.0]).0, 0.0);
    // With the price index the amounts grow (#66)
    let r = data.resolve(&loc("ZH", 261, Church::None), true).unwrap();
    assert_eq!(
        r.canton_deductions
            .second_earner_amount(&[103_117.0, 71_883.0], 1.1)
            .round(),
        6820.0
    );
    // Listed as its own part of the proposal (#131)
    let parts = r.canton_deductions.parts(175_000.0, &[103_117.0, 71_883.0], 1.0);
    assert!(
        parts.iter().any(|p| p.0 == "secondEarner" && p.1 == 6200.0),
        "{parts:?}"
    );
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

    // #272/AK4: without canton and municipality the municipality is missing (the canton follows from it)
    v["taxes"]["location"] = json!({ "canton": "", "church": "none" });
    let e = errors(&v);
    assert_eq!(
        e,
        vec![("tax_municipality_missing", Some("taxes.location.municipality".into()))]
    );
    v["taxes"]["location"] = json!({ "canton": "", "municipality": 261 });
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
    assert_eq!(errors(&v)[0].1.as_deref(), Some("taxes.location.municipality")); // #272/AK4
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

/// Canton, BFS number, married, net earned income per person (as shown by the ESTV calculator after social
/// contributions; `0.0` for no second earner), wealth, ESTV: federal, canton, municipality (income), canton,
/// municipality (wealth).
type EmployedCase = (&'static str, u32, bool, [f64; 2], f64, [f64; 5]);

/// #63/AK6: Invented cases for employed persons, computed on 23.11.2026 in the ESTV tax calculator 2026 (user
/// interface, version 2.1.3): gross wage 70 000, 110 000, 210 000 (single), 150 000 (married, one earner) and
/// 115 000 + 80 000 (married, two earners); age 35, no children, no church, standard deductions only. The
/// calculator derives the net wage itself; that net wage is the earned income of the app. Personal tax not included.
const EMPLOYED: [EmployedCase; 25] = [
    (
        "ZH",
        261,
        false,
        [63_063.0, 0.0],
        50_000.0,
        [648.0, 2460.0, 3081.0, 0.0, 0.0],
    ),
    (
        "ZH",
        261,
        false,
        [98_632.0, 0.0],
        200_000.0,
        [2275.0, 5237.0, 6560.0, 57.0, 71.0],
    ),
    (
        "ZH",
        261,
        false,
        [189_259.0, 0.0],
        800_000.0,
        [10750.0, 13944.0, 17467.0, 604.0, 756.0],
    ),
    (
        "ZH",
        261,
        true,
        [134_539.0, 0.0],
        400_000.0,
        [3203.0, 6383.0, 7996.0, 114.0, 142.0],
    ),
    (
        "ZH",
        261,
        true,
        [103_117.0, 71_883.0],
        900_000.0,
        [5309.0, 9167.0, 11482.0, 632.0, 792.0],
    ),
    (
        "BE",
        351,
        false,
        [63_063.0, 0.0],
        50_000.0,
        [648.0, 5949.0, 3080.0, 0.0, 0.0],
    ),
    (
        "BE",
        351,
        false,
        [98_632.0, 0.0],
        200_000.0,
        [2275.0, 10957.0, 5672.0, 305.0, 158.0],
    ),
    (
        "BE",
        351,
        false,
        [189_259.0, 0.0],
        800_000.0,
        [10750.0, 26339.0, 13634.0, 1944.0, 1006.0],
    ),
    (
        "BE",
        351,
        true,
        [134_539.0, 0.0],
        400_000.0,
        [3203.0, 13535.0, 7007.0, 733.0, 380.0],
    ),
    (
        "BE",
        351,
        true,
        [103_117.0, 71_883.0],
        900_000.0,
        [5309.0, 19216.0, 9947.0, 2232.0, 1155.0],
    ),
    (
        "LU",
        1061,
        false,
        [63_063.0, 0.0],
        50_000.0,
        [648.0, 3014.0, 3014.0, 0.0, 0.0],
    ),
    (
        "LU",
        1061,
        false,
        [98_632.0, 0.0],
        200_000.0,
        [2275.0, 5522.0, 5522.0, 149.0, 149.0],
    ),
    (
        "LU",
        1061,
        false,
        [189_259.0, 0.0],
        800_000.0,
        [10750.0, 12349.0, 12349.0, 801.0, 801.0],
    ),
    (
        "LU",
        1061,
        true,
        [134_539.0, 0.0],
        400_000.0,
        [3203.0, 6644.0, 6644.0, 299.0, 299.0],
    ),
    (
        "LU",
        1061,
        true,
        [103_117.0, 71_883.0],
        900_000.0,
        [5309.0, 9280.0, 9280.0, 843.0, 843.0],
    ),
    (
        "AG",
        4001,
        false,
        [63_063.0, 0.0],
        50_000.0,
        [648.0, 3079.0, 2870.0, 0.0, 0.0],
    ),
    (
        "AG",
        4001,
        false,
        [98_632.0, 0.0],
        200_000.0,
        [2275.0, 6344.0, 5913.0, 50.0, 47.0],
    ),
    (
        "AG",
        4001,
        false,
        [189_259.0, 0.0],
        800_000.0,
        [10750.0, 15504.0, 14450.0, 873.0, 813.0],
    ),
    (
        "AG",
        4001,
        true,
        [134_539.0, 0.0],
        400_000.0,
        [3203.0, 6903.0, 6433.0, 111.0, 104.0],
    ),
    (
        "AG",
        4001,
        true,
        [103_117.0, 71_883.0],
        900_000.0,
        [5309.0, 10525.0, 9810.0, 823.0, 767.0],
    ),
    (
        "SG",
        3203,
        false,
        [63_063.0, 0.0],
        50_000.0,
        [648.0, 3276.0, 4306.0, 0.0, 0.0],
    ),
    (
        "SG",
        3203,
        false,
        [98_632.0, 0.0],
        200_000.0,
        [2275.0, 6676.0, 8774.0, 223.0, 293.0],
    ),
    (
        "SG",
        3203,
        false,
        [189_259.0, 0.0],
        800_000.0,
        [10750.0, 15606.0, 20511.0, 1294.0, 1701.0],
    ),
    (
        "SG",
        3203,
        true,
        [134_539.0, 0.0],
        400_000.0,
        [3203.0, 7523.0, 9888.0, 446.0, 587.0],
    ),
    (
        "SG",
        3203,
        true,
        [103_117.0, 71_883.0],
        900_000.0,
        [5309.0, 11156.0, 14662.0, 1339.0, 1760.0],
    ),
];

/// Income and wealth tax of the app for an employed case (net earned income per person, no pensions).
fn employed_taxes(data: &taxdata::TaxData, case: &EmployedCase) -> (f64, f64) {
    let (canton, bfs, married, earned, wealth, _) = *case;
    let res = data.resolve(&loc(canton, bfs, Church::None), married).unwrap();
    let persons: &[f64] = if married { &earned } else { &earned[..1] };
    let income: f64 = earned.iter().sum();
    (res.income_tax(income, persons, 1.0), res.wealth_tax(wealth, 1.0))
}

/// Relative deviation from the ESTV value; an ESTV value of 0 requires (almost) 0.
fn deviation(mine: f64, estv: f64) -> f64 {
    if estv == 0.0 {
        if mine < 1.0 {
            0.0
        } else {
            f64::INFINITY
        }
    } else {
        (mine / estv - 1.0).abs()
    }
}

/// #63/AK6: Employed persons (single, married with one or two earned incomes) deviate at most ±5 % from the ESTV tax
/// calculator (income and wealth separately), with the standard deductions of the canton (variant «with contributions
/// to the 2nd and 3rd pillar», flat deduction for professional expenses, second-earner deduction).
#[test]
fn issue_63_ak6_reference_cases_employed_match_estv_within_five_percent() {
    let data = taxdata::builtin();
    let mut worst: f64 = 0.0;
    for case in &EMPLOYED {
        let (canton, _, married, earned, wealth, r) = *case;
        let (my_i, my_w) = employed_taxes(data, case);
        let (ref_i, ref_w) = (r[0] + r[1] + r[2], r[3] + r[4]);
        let what = format!("{canton} married={married} {earned:?}/{wealth}");
        let (di, dw) = (deviation(my_i, ref_i), deviation(my_w, ref_w));
        assert!(di <= 0.05, "{what}: income {my_i} instead of {ref_i}");
        assert!(dw <= 0.05, "{what}: wealth {my_w} instead of {ref_w}");
        worst = worst.max(di).max(dw);
    }
    // Regression: measured at most 0.3 % (23.11.2026)
    assert!(worst < 0.01, "largest deviation {worst}");
}

/// Zurich, BFS 261, married, income, wealth (as in `CASES`); tax calculator of the canton of Zurich 2026 (on the
/// taxable income and wealth, rounded down to 100 by the calculator): canton, municipality (income), canton,
/// municipality (wealth). Calculated on 26.11.2026.
const ZH_CALCULATOR: [(bool, f64, f64, [f64; 4]); 5] = [
    (false, 80_000.0, 500_000.0, [3781.95, 4737.39, 283.10, 354.62]),
    (false, 150_000.0, 1_200_000.0, [10108.00, 12661.60, 1172.30, 1468.46]),
    (false, 300_000.0, 4_000_000.0, [26989.50, 33807.90, 7531.60, 9434.32]),
    (true, 100_000.0, 300_000.0, [3891.20, 4874.24, 65.55, 82.11]),
    (true, 220_000.0, 2_000_000.0, [14071.40, 17626.28, 2460.50, 3082.10]),
];

/// #287/AK6: The reference cases of Zurich also match the tax calculator of the canton of Zurich within ±5 %
/// (cantonal and municipal tax on income and on wealth; the federal tax follows from DBG Art. 36).
#[test]
fn issue_287_ak6_zurich_cases_match_the_cantonal_calculator() {
    let data = taxdata::builtin();
    for (married, income, wealth, r) in ZH_CALCULATOR {
        let res = data.resolve(&loc("ZH", 261, Church::None), married).unwrap();
        let (taxable, _) = res.taxable(income, &[], 1.0);
        let my_i = res.income_tax_on(taxable, 0.0, 1.0);
        let my_w = res.wealth_tax(wealth, 1.0);
        let (ref_i, ref_w) = (r[0] + r[1], r[2] + r[3]);
        assert!(
            (my_i / ref_i - 1.0).abs() <= 0.05,
            "{income}/{wealth}: income {my_i} instead of {ref_i}"
        );
        assert!(
            (my_w / ref_w - 1.0).abs() <= 0.05,
            "{income}/{wealth}: wealth {my_w} instead of {ref_w}"
        );
    }
}

/// #296/AK4: Cases from the tax calculator of the canton of Valais 2026 (single, no children, taxable income
/// 100000, wealth 500000): `(BFS, index of the municipality, [canton income, canton wealth, municipality income,
/// municipality wealth])`. Sion (index 176 %, coefficient 1.1) and Agarn (index 153 %, coefficient 1.3).
const VS_CALCULATOR: [(u32, u32, [f64; 4]); 2] = [
    (6266, 176, [8676.30, 1100.00, 8127.45, 1210.00]),
    (6101, 153, [8676.30, 1100.00, 10414.55, 1430.00]),
];

/// #296/AK4: The indexed tariffs of the canton and of two municipalities with different index match the tax
/// calculator of the canton of Valais within ±5 % (canton and municipality together, income and wealth; the
/// wealth tax of the municipality is not indexed).
#[test]
fn issue_296_ak4_valais_municipalities_match_the_cantonal_calculator() {
    let data = taxdata::builtin();
    for (bfs, index, r) in VS_CALCULATOR {
        let res = data.resolve(&loc("VS", bfs, Church::None), false).unwrap();
        let my_i = res.income_tax_on(100_000.0, 0.0, 1.0);
        // The wealth curve holds the deduction for single persons (LF VS Art. 59 Abs. 1 lit. b, 45000); the
        // calculator takes the entered wealth as taxable.
        let my_w = res.wealth_tax(545_000.0, 1.0);
        let (ref_i, ref_w) = (r[0] + r[2], r[1] + r[3]);
        assert!(
            (my_i / ref_i - 1.0).abs() <= 0.05,
            "{bfs} ({index} %): income {my_i} instead of {ref_i}"
        );
        assert!(
            (my_w / ref_w - 1.0).abs() <= 0.05,
            "{bfs} ({index} %): wealth {my_w} instead of {ref_w}"
        );
    }
}

/// #297/AK3: Married couples without children in Sion (BFS 6266, index 176 %, coefficient 1.1) from the tax
/// calculator of the canton of Valais (https://taxcalculator.apps.vs.ch/, «Natürliche Person», tax year 2026, retrieved
/// 27.11.2026, invented taxable incomes, no wealth): `(taxable income, [canton tax, canton reduction, municipal tax,
/// municipal reduction])`, tax before the reduction. 25000 reaches the minimum (680), 250000 the maximum (4900).
const VS_MARRIED_CALCULATOR: [(f64, [f64; 4]); 4] = [
    (25_000.0, [810.35, 680.00, 846.45, 680.00]),
    (60_000.0, [3590.80, 1256.80, 3739.75, 1308.90]),
    (120_000.0, [12201.00, 4270.35, 10434.05, 3651.95]),
    (250_000.0, [34057.00, 4900.00, 25603.35, 4900.00]),
];

/// #297/AK3: The tax of married couples in Valais after the reduction matches the cantonal calculator within ±5 %,
/// canton and municipality separately; the reductions themselves within one franc.
#[test]
fn issue_297_ak3_valais_married_match_the_cantonal_calculator() {
    let data = taxdata::builtin();
    let res = data.resolve(&loc("VS", 6266, Church::None), true).unwrap();
    assert_eq!(res.rebates.len(), 2, "canton and municipality");
    for (income, r) in VS_MARRIED_CALCULATOR {
        let parts: Vec<(f64, f64)> = res
            .rebates
            .iter()
            .map(|rb| {
                let tax = tax::tax_indexed(&rb.table, income, 1.0, 1.0);
                (tax, tax - rb.rule.amount(tax, 1.0))
            })
            .collect();
        for (k, (name, (tax, after))) in ["canton", "municipality"].iter().zip(parts).enumerate() {
            let (ref_tax, ref_rebate) = (r[2 * k], r[2 * k + 1]);
            let ref_after = ref_tax - ref_rebate;
            assert!(
                (tax - after - ref_rebate).abs() <= 1.0,
                "{income} {name}: reduction {} instead of {ref_rebate}",
                tax - after
            );
            assert!(
                (after / ref_after - 1.0).abs() <= 0.05,
                "{income} {name}: {after} instead of {ref_after}"
            );
        }
        let total = res.income_tax_on(income, 0.0, 1.0);
        let ref_total = r[0] - r[1] + r[2] - r[3];
        assert!(
            (total / ref_total - 1.0).abs() <= 0.05,
            "{income}: total {total} instead of {ref_total}"
        );
    }
}

/// #297/AK2: The reduction applies to married couples only, to canton and municipality (not to the church tax), and its
/// minimum and maximum follow the price index.
#[test]
fn issue_297_ak2_valais_reduction_for_married_couples_only() {
    let data = taxdata::builtin();
    let vs = data.canton("VS").unwrap();
    let rule = vs.income_rebate.canton.clone().unwrap();
    assert_eq!((rule.rate, rule.min, rule.max), (0.35, 680.0, 4900.0));
    assert!(rule.source.contains("Art. 32 Abs. 3 lit. a"), "{}", rule.source);
    assert!(vs
        .income_rebate
        .municipal
        .as_ref()
        .unwrap()
        .source
        .contains("Art. 178 Abs. 3 lit. a"));
    // single persons: no reduction
    let single = data.resolve(&loc("VS", 6266, Church::None), false).unwrap();
    assert!(single.rebates.is_empty());
    // married couples: tax after the reduction = tax of the curves minus both reductions
    let married = data.resolve(&loc("VS", 6266, Church::None), true).unwrap();
    let full = tax::tax_indexed(&married.cantonal, 60_000.0, 1.0, 1.0);
    let after = married.income_tax_on(60_000.0, 0.0, 1.0);
    assert!((full - after - 1256.80 - 1308.90).abs() <= 1.0, "{full} - {after}");
    // church tax (Sion, Roman Catholic 3 %) is not reduced: the difference to no denomination stays the full church tax
    let catholic = data.resolve(&loc("VS", 6266, Church::Catholic), true).unwrap();
    let church = catholic.income_tax_on(60_000.0, 0.0, 1.0) - after;
    assert!((church - 0.03 * 3590.80).abs() <= 1.0, "church {church}");
    // other cantons: no reduction
    assert!(data
        .resolve(&loc("ZH", 261, Church::None), true)
        .unwrap()
        .rebates
        .is_empty());
    // the index raises minimum and maximum
    assert_eq!(rule.amount(100.0, 1.0), 100.0, "never more than the tax");
    assert_eq!(rule.amount(1000.0, 1.0), 680.0);
    assert!((rule.amount(1000.0, 1.1) - 748.0).abs() < 1e-9);
    assert!((rule.amount(100_000.0, 1.1) - 5390.0).abs() < 1e-9);
    assert_eq!(rule.amount(10_000.0, 1.0), 3500.0);
    assert_eq!(rule.amount(0.0, 1.0), 0.0);
    let open = taxdata::TaxRebate {
        rate: 0.35,
        min: 0.0,
        max: 0.0,
        source: String::new(),
    };
    assert_eq!(open.amount(100_000.0, 1.0), 35_000.0, "max 0: no maximum");
}

/// #143/AK5: In the mode «Wohnort» the validation warns once the calendar year is later than the tax year of the
/// data + 1; the year is given, so the test does not depend on the clock. Own tables do not use the data.
#[test]
fn issue_143_ak5_warning_when_tax_data_outdated() {
    let data_year = taxdata::builtin().year;
    let outdated = |v: &Value, year: i32| {
        let plan: Plan = serde_json::from_value(v.clone()).unwrap();
        validate_in(&plan, year)
            .into_iter()
            .filter(|i| i.message.code() == "tax_data_outdated")
            .collect::<Vec<_>>()
    };
    let v = base(false);
    assert!(outdated(&v, data_year).is_empty());
    assert!(outdated(&v, data_year + 1).is_empty());
    let w = outdated(&v, data_year + 2);
    assert_eq!(w.len(), 1);
    assert_eq!(w[0].severity, Severity::Warning);
    assert_eq!(w[0].section, "taxes");
    let text = w[0].message.text();
    assert!(text.contains(&format!("tax year {data_year}")), "{text}");
    assert!(text.contains(&format!("in {}", data_year + 2)), "{text}");
    // own tables: no warning
    let mut own = base(false);
    own["taxes"]["mode"] = json!("own");
    assert!(outdated(&own, data_year + 5).is_empty());
    // validate() reads the clock: today the built-in data are current enough (no warning before the data year + 2)
    let plan: Plan = serde_json::from_value(v).unwrap();
    let now: i32 = engine::beta::today()[..4].parse().unwrap();
    let warned = validate(&plan).iter().any(|i| i.message.code() == "tax_data_outdated");
    assert_eq!(warned, now > data_year + 1);
}
