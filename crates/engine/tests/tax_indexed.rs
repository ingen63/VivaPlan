//! #66: Tax tariffs with inflation (compensation of bracket creep).
//!
//! Made-up plan: birth 1980-01, start 2040-01, pension fund lump sum at 65 (2045-01), end 2047-12, no returns;
//! earned income indexed to inflation, tax multiplier 1.

use std::path::Path;

use engine::legacy;
use engine::model::Plan;
use engine::sim::{MonthRecord, SimResult, Simulator};
use engine::tax;
use engine::time::YearMonth;
use serde_json::{json, Value};

fn base(inflation: f64, indexed: Option<bool>) -> Value {
    let mut v = json!({
        "project": { "name": "Tarife mit der Teuerung" },
        "household": { "persons": [ { "id": "A", "name": "A", "birth": "1980-01" } ] },
        "milestones": [
            { "id": "start", "name": "Start", "kind": "date", "date": "2040-01", "fixed": true },
            { "id": "retire", "name": "Bezug Pensionskasse", "kind": "age", "person": "A", "years": 65 },
            { "id": "ahv", "name": "AHV-Bezug", "kind": "age", "person": "A", "years": 65 },
            { "id": "end", "name": "Ende", "kind": "date", "date": "2047-12", "fixed": true }
        ],
        "pension": { "pk": { "capital": 500_000, "at": "retire", "lumpsum": 1.0, "conversion": 0 },
                     "ahv": { "monthly": 0, "at": "ahv" } },
        "flows": { "income": 10_000, "incomeIndexed": true, "spending": 4000 },
        "wealth": { "stocks": 0, "bonds": 0, "cash": 1_000_000, "stockShare": 0, "reserveMonths": 0 },
        "returns": { "source": "fixed", "fixed": { "stocks": 0, "bonds": 0, "inflation": inflation, "saron": 0 } },
        "taxes": { "multiplier": 1.0,
                   "income": [[0, 0.0], [200_000, 0.10]],
                   "wealth": [[0, 0.0], [4_000_000, 0.01]],
                   "capital": [[0, 0.0], [1_000_000, 0.10]] }
    });
    if let Some(b) = indexed {
        v["taxes"]["indexed"] = json!(b);
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
    r.records
        .iter()
        .find(|x| x.month == YearMonth::new(y, m))
        .expect("month in range")
}

fn close(a: f64, b: f64, tol: f64, what: &str) {
    assert!((a - b).abs() <= tol, "{what}: {a} instead of {b}");
}

/// Linear rate of the table `[[0, 0], [top, max]]` below the upper bracket.
fn linear(x: f64, top: f64, max: f64) -> f64 {
    max * (x / top).min(1.0)
}

/// #66/AK2: The example from the issue. Table `[[0, 0], [200000, 0.10]]`, inflation 2 % per year, year 10: the
/// bracket is at 243 799, and 121 899 has the rate 5 % like 100 000 in the start year (not 6.09 %).
#[test]
fn issue_66_ak2_example_from_issue() {
    let t = [(0.0, 0.0), (200_000.0, 0.10)];
    let idx = 1.02f64.powi(10);
    close(200_000.0 * idx, 243_799.0, 0.5, "bracket in year 10");
    let x = 100_000.0 * idx;
    close(x, 121_899.0, 0.5, "income in year 10");
    close(
        tax::tax_indexed(&t, x, 1.0, idx),
        x * 0.05,
        0.01,
        "rate 5 % with inflation",
    );
    close(tax::rate_at(&t, x), 0.0609, 0.0001, "rate without adjustment");
    close(
        tax::tax_indexed(&t, x, 1.0, 1.0),
        tax::tax(&t, x, 1.0),
        1e-9,
        "index 1 = unadjusted tax",
    );
    // At the shifted bracket the top rate applies, and it stays above.
    close(
        tax::tax_indexed(&t, 200_000.0 * idx, 1.0, idx),
        200_000.0 * idx * 0.10,
        0.01,
        "at the bracket",
    );
    close(tax::tax_indexed(&t, 1e7, 1.0, idx), 1e6, 0.01, "rate stays 10 %");
    // The multiplier applies as before.
    close(tax::tax_indexed(&t, x, 2.0, idx), x * 0.10, 0.01, "multiplier 2");
    assert_eq!(tax::tax_indexed(&t, -1.0, 1.0, idx), 0.0);
}

/// #66: An index <= 0 (invalid) is treated as no adjustment.
#[test]
fn issue_66_nonpositive_index_falls_back_to_unadjusted_tax() {
    let t = [(0.0, 0.0), (200_000.0, 0.10)];
    for idx in [0.0, -1.5] {
        close(
            tax::tax_indexed(&t, 100_000.0, 1.0, idx),
            tax::tax(&t, 100_000.0, 1.0),
            1e-9,
            "index <= 0",
        );
    }
}

/// #66/AK1: Income tax and lump-sum tax use the brackets shifted by the cumulative simulated inflation; the rates
/// stay the same.
#[test]
fn issue_66_ak1_income_and_capital_follow_inflation() {
    let r = run(&base(0.02, Some(true)));
    // Income tax of 2043, paid in May 2044: income of the year, rate at the index of December 2043.
    let year: f64 = r
        .records
        .iter()
        .filter(|x| x.month.year() == 2043)
        .map(|x| x.earned)
        .sum();
    let idx = rec(&r, 2043, 12).price_index;
    assert!(idx > 1.07, "index after four years: {idx}");
    let expected = year * linear(year / idx, 200_000.0, 0.10);
    close(rec(&r, 2044, 5).tax_income, expected, 0.02, "income tax 2043");
    // The real income is the same every year, so the rate stays near 6 % (120 000 of 200 000 × 10 %; slightly
    // below, because the income rises during the year and the December index applies).
    close(expected / year, 0.06, 0.001, "rate as in the start year");
    let off = run(&base(0.02, Some(false)));
    assert!(
        rec(&off, 2044, 5).tax_income / year > 0.064,
        "without adjustment the rate rises"
    );

    // Lump sum 2045-01: rate at the index of the month of the payout.
    let m = rec(&r, 2045, 1);
    let expected = 500_000.0 * linear(500_000.0 / m.price_index, 1_000_000.0, 0.10);
    close(m.tax_capital, expected, 0.02, "lump-sum tax");
    assert!(m.tax_capital < 25_000.0, "lower than without adjustment (25 000)");
}

/// #66/AK1: The brackets of the wealth tax grow too; with inflation and the option off the tax stays higher, without
/// inflation it is the same.
#[test]
fn issue_66_ak1_wealth_follows_inflation() {
    let on = run(&base(0.02, Some(true)));
    let off = run(&base(0.02, Some(false)));
    let (w_on, w_off) = (rec(&on, 2044, 5).tax_wealth, rec(&off, 2044, 5).tax_wealth);
    assert!(w_on > 0.0 && w_on < w_off, "wealth tax with {w_on}, without {w_off}");
    for (a, b) in run(&base(0.0, Some(true)))
        .records
        .iter()
        .zip(&run(&base(0.0, Some(false))).records)
    {
        assert_eq!(a.taxes, b.taxes, "no difference without inflation ({})", a.month);
    }
}

/// #66/AK1: The estimated income tax in the need of the 3-bucket strategy uses the same shifted brackets. Pension
/// fund pension 120 000 per year from 2045 (nominal, not indexed), no earned income: in January 2047 the rate applies to the amount in
/// prices of the start, without the option it stays 6 %.
#[test]
fn issue_66_ak1_bucket_need_income_tax_follows_inflation() {
    let with_buckets = |indexed| {
        let mut v = base(0.02, Some(indexed));
        v["pension"]["pk"] = json!({ "capital": 500_000, "at": "retire", "lumpsum": 0, "conversion": 0.24 });
        v["flows"]["income"] = json!(0);
        v["withdrawal"] = json!({ "strategy": "three_buckets", "buckets": { "cashYears": 2, "bondYears": 1, "refillThreshold": -1 } });
        run(&v)
    };
    let january = |r: &SimResult, y: i32| {
        r.bucket_refills
            .iter()
            .find(|x| x.month == YearMonth::new(y, 1) && x.trigger == "Januar")
            .unwrap_or_else(|| panic!("no January refill {y}"))
            .basis
            .clone()
    };
    let (on, off) = (with_buckets(true), with_buckets(false));
    let (b_on, b_off) = (january(&on, 2047), january(&off, 2047));
    let income = b_on.earned + b_on.ahv + b_on.pk;
    close(income, 120_000.0, 0.01, "pension fund pension");
    let idx = rec(&on, 2047, 1).price_index;
    assert!(idx > 1.13, "index after seven years: {idx}");
    close(
        b_on.income_tax,
        income * linear(income / idx, 200_000.0, 0.10),
        0.01,
        "with the option",
    );
    close(b_off.income_tax, income * 0.06, 0.01, "without the option");
}

/// #66/AK4: Existing plans without the field compute like with the option off, i.e. unchanged in nominal francs;
/// the field is written only if set. New plans have the option on.
#[test]
fn issue_66_ak4_default_off_for_existing_plans_on_for_new() {
    let missing = run(&base(0.02, None));
    let off = run(&base(0.02, Some(false)));
    for (a, b) in missing.records.iter().zip(&off.records) {
        assert_eq!(a.taxes, b.taxes, "{}", a.month);
    }
    // Without the field the nominal brackets apply (lump sum 500 000 → 5 % = 25 000).
    close(
        rec(&missing, 2045, 1).tax_capital,
        25_000.0,
        0.01,
        "lump sum without adjustment",
    );

    let plan: Plan = serde_json::from_value(base(0.02, None)).unwrap();
    assert_eq!(plan.taxes.indexed, None);
    assert!(!engine::indexation::Indexation::of(&plan).taxes);
    assert!(serde_json::to_value(&plan).unwrap()["taxes"].get("indexed").is_none());
    let plan: Plan = serde_json::from_value(base(0.02, Some(true))).unwrap();
    assert_eq!(serde_json::to_value(&plan).unwrap()["taxes"]["indexed"], json!(true));

    let new: Plan = serde_json::from_str(include_str!("../assets/new.vplan")).unwrap();
    assert!(
        engine::indexation::Indexation::of(&new).taxes,
        "new plans: «Tarife mit der Teuerung» on"
    );
}

/// #66/AK4: The import of an old `config.json` sets the option off.
#[test]
fn issue_66_ak4_legacy_import_option_off() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let text = std::fs::read_to_string(dir.join("legacy_fixed.json")).unwrap();
    let imp = legacy::import(&text, 2026, &dir, "Test").unwrap();
    assert!(!engine::indexation::Indexation::of(&imp.plan).taxes);
    assert!(serde_json::to_value(&imp.plan).unwrap()["taxes"]
        .get("indexed")
        .is_none());
}

/// #66/AK3: The adjustment applies to the tables of married couples too (mode «Eigene Tabellen»).
#[test]
fn issue_66_ak3_couple_tables_follow_inflation() {
    let married = |indexed| {
        let mut v = base(0.02, Some(indexed));
        v["household"]["persons"] = json!([
            { "id": "A", "name": "A", "birth": "1980-01" },
            { "id": "B", "name": "B", "birth": "1980-01" }
        ]);
        v["household"]["married"] = json!(true);
        v["partner"] = json!({ "income": 0, "pk": { "capital": 0, "at": "retire", "lumpsum": 0, "conversion": 0 },
                               "ahv": { "monthly": 0, "at": "ahv" } });
        v["taxes"]["couple"] = json!({ "income": [[0, 0.0], [400_000, 0.10]], "wealth": [[0, 0.0], [8_000_000, 0.01]],
                                       "capital": [[0, 0.0], [2_000_000, 0.10]] });
        run(&v)
    };
    let (on, off) = (married(true), married(false));
    let m = rec(&on, 2045, 1);
    let expected = 500_000.0 * linear(500_000.0 / m.price_index, 2_000_000.0, 0.10);
    close(m.tax_capital, expected, 0.02, "lump sum with the couple table");
    close(
        rec(&off, 2045, 1).tax_capital,
        12_500.0,
        0.01,
        "couple table without adjustment",
    );
    // Income tax 2043 with the couple table.
    let year: f64 = on
        .records
        .iter()
        .filter(|x| x.month.year() == 2043)
        .map(|x| x.earned)
        .sum();
    let idx = rec(&on, 2043, 12).price_index;
    close(
        rec(&on, 2044, 5).tax_income,
        year * linear(year / idx, 400_000.0, 0.10),
        0.02,
        "income tax with the couple table",
    );
    assert!(rec(&on, 2044, 5).tax_wealth < rec(&off, 2044, 5).tax_wealth);
}

/// #66/AK5: The property gain tax stays unchanged; its tariff refers to the single gain.
#[test]
fn issue_66_ak5_property_gain_tax_unchanged() {
    let with_house = |indexed| {
        let mut v = base(0.02, Some(indexed));
        v["realestate"] = json!({ "properties": [
            { "id": "h", "name": "Haus", "status": "owned", "price": 500_000, "worth": 800_000, "sell": "2046-06",
              "fixCosts": 0, "inflation": true }
        ] });
        v["taxes"]["propertyGain"] = json!({ "brackets": [[100_000, 0.1], [200_000, 0.2]], "reduction": 0.0 });
        run(&v)
    };
    let (on, off) = (with_house(true), with_house(false));
    let (g_on, g_off) = (rec(&on, 2046, 6).tax_gain, rec(&off, 2046, 6).tax_gain);
    assert!(g_on > 0.0, "sale with a gain");
    assert_eq!(g_on, g_off, "property gain tax equal");
}
