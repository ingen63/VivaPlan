//! Second person (double income, rules for married couples), withdrawal strategies and evaluation over all runs.

use std::path::Path;

use engine::batch;
use engine::market::DataSet;
use engine::model::Plan;
use engine::sim::{EventKind, SimResult, Simulator};
use engine::validate::{self, Severity};
use serde_json::{json, Value};

/// 10 years from January 2030, no returns, no inflation, no taxes, retirement at the start.
fn base() -> Value {
    json!({
        "project": { "name": "Test" },
        "household": { "persons": [ { "id": "A", "name": "A", "birth": "1965-01" } ] },
        "milestones": [
            { "id": "start", "name": "Start", "kind": "date", "date": "2030-01" },
            { "id": "retire", "name": "Pensionierung", "kind": "date", "date": "2030-01" },
            { "id": "end", "name": "Ende", "kind": "date", "date": "2039-12" }
        ],
        "pension": {
            "pk": { "capital": 0, "at": "retire", "lumpsum": 0, "conversion": 0 },
            "ahv": { "monthly": 0, "at": "never" }
        },
        "flows": { "income": 0, "spending": 5000 },
        "wealth": { "stocks": 0, "bonds": 0, "cash": 1_000_000, "stockShare": 1, "reserveMonths": 0 },
        "returns": { "source": "fixed", "fixed": { "stocks": 0, "bonds": 0, "inflation": 0 } },
        "taxes": { "income": [], "wealth": [], "capital": [] }
    })
}

/// #191: Like [`with_partner`], B the same age as A, so that an AHV pension from the start (2030-01) is paid at 65 for
/// both (no reduction for an early payout).
fn with_partner_65(v: Value) -> Value {
    let mut v = with_partner(v);
    v["household"]["persons"][1]["birth"] = json!("1965-01");
    v
}

fn with_partner(mut v: Value) -> Value {
    v["household"]["persons"] = json!([
        { "id": "A", "name": "A", "birth": "1965-01" },
        { "id": "B", "name": "B", "birth": "1967-01" }
    ]);
    v["partner"] = json!({
        "income": 0,
        "pk": { "capital": 0, "at": "retire", "lumpsum": 0, "conversion": 0 },
        "ahv": { "monthly": 0, "at": "never" }
    });
    v
}

fn plan(v: Value) -> Plan {
    serde_json::from_value(crate::any_age(v)).expect("Planung gültig")
}

fn run(v: Value) -> SimResult {
    let p = plan(v);
    let sim = Simulator::new(&p).unwrap();
    let paths = batch::market_paths(&p, &sim, None).unwrap();
    sim.run(&paths[0], true)
}

// ---------------------------------------------------------------------------------------------
// Second person
// ---------------------------------------------------------------------------------------------

#[test]
fn partner_income_adds_to_household() {
    let mut v = with_partner(base());
    v["flows"]["income"] = json!(3_000);
    v["partner"]["income"] = json!(2_000);
    v["milestones"][1]["date"] = json!("2040-01");
    let res = run(v);
    let r = &res.records[0];
    assert!((r.earned - 5_000.0).abs() < 1e-9);
    assert!((r.income - 5_000.0).abs() < 1e-9);
    // Income covers spending: wealth stays.
    assert!((res.summary.end_total - 1_000_000.0).abs() < 1e-6);
}

#[test]
fn partner_without_second_person_is_an_error() {
    let mut v = base();
    v["partner"] = with_partner(base())["partner"].clone();
    let issues = validate::validate(&plan(v));
    assert!(issues
        .iter()
        .any(|i| i.severity == Severity::Error && i.section == "general"));
}

#[test]
fn more_than_two_persons_is_an_error() {
    let mut v = with_partner(base());
    v["household"]["persons"]
        .as_array_mut()
        .unwrap()
        .push(json!({ "id": "C", "name": "C", "birth": "1990-01" }));
    let issues = validate::validate(&plan(v));
    assert!(issues
        .iter()
        .any(|i| i.severity == Severity::Error && i.message.contains("two")));
}

#[test]
fn each_person_retires_on_own_date_and_lump_sums_are_added() {
    let mut v = with_partner(base());
    v["milestones"]
        .as_array_mut()
        .unwrap()
        .push(json!({ "id": "retireB", "name": "Pensionierung B", "kind": "age", "person": "B", "years": 66 }));
    v["milestones"][1]["date"] = json!("2031-01");
    v["pension"]["pk"] = json!({ "capital": 500_000, "at": "retire", "lumpsum": 0.0, "conversion": 0.06 });
    // B is 2 years younger: 66 years = January 2033
    v["partner"]["pk"] = json!({ "capital": 300_000, "at": "retireB", "lumpsum": 1.0, "conversion": 0.06 });
    v["taxes"]["capital"] = json!([[0, 0.1]]);
    let res = run(v);
    let retirements: Vec<_> = res.events.iter().filter(|e| e.kind == EventKind::Retirement).collect();
    assert_eq!(retirements.len(), 2);
    assert_eq!(retirements[0].month.to_string(), "01.2031");
    assert_eq!(retirements[1].month.to_string(), "01.2033");
    // Pension A: 500'000 × 6 % / 12 = 2'500; capital B net 270'000
    assert!((res.summary.pension_fund_monthly - 2_500.0).abs() < 1e-6);
    assert!((res.summary.lumpsum_net - 270_000.0).abs() < 1e-6);
}

#[test]
fn couple_ahv_is_capped_but_single_pension_is_not() {
    let mut v = with_partner_65(base());
    v["pension"]["ahv"] = json!({ "monthly": 2_520, "at": "start" });
    v["partner"]["ahv"] = json!({ "monthly": 2_520, "at": "start" });
    v["household"]["ahvCoupleCap"] = json!(3_780);
    let res = run(v.clone());
    assert!((res.records[0].ahv - 3_780.0).abs() < 1e-9, "{}", res.records[0].ahv);

    // Only one pension due: no cut.
    v["partner"]["ahv"]["at"] = json!("never");
    let res = run(v.clone());
    assert!((res.records[0].ahv - 2_520.0).abs() < 1e-9);

    // Not married: no cap.
    v["partner"]["ahv"]["at"] = json!("start");
    v["household"]["married"] = json!(false);
    let res = run(v);
    assert!((res.records[0].ahv - 5_040.0).abs() < 1e-9);
}

/// #145: Married couple with cap 3'780 and inflation 3 % (fixed); AHV in month `k` (0 = January 2030).
fn capped_ahv(a_indexed: bool, b_indexed: bool, b_at: &str) -> SimResult {
    // #191: B reaches 65 at its AHV payout (2030-01 or 2032-01)
    let mut v = if b_at == "start" {
        with_partner_65(base())
    } else {
        with_partner(base())
    };
    v["milestones"]
        .as_array_mut()
        .unwrap()
        .push(json!({ "id": "b_ahv", "name": "AHV B", "kind": "date", "date": "2032-01" }));
    v["returns"]["fixed"]["inflation"] = json!(0.03);
    v["pension"]["ahv"] = json!({ "monthly": 2_520, "at": "start", "indexed": a_indexed });
    v["partner"]["ahv"] = json!({ "monthly": 2_520, "at": b_at, "indexed": b_indexed });
    v["household"]["ahvCoupleCap"] = json!(3_780);
    run(v)
}

fn ahv_in(res: &SimResult, k: usize) -> f64 {
    res.records[k].ahv
}

// #145/AK1, AK2: Both pensions indexed: the cap rises once per January by 3 %, not by 3 % per person.
// 01.2031: 3'780 × 1.03 = 3'893.40; 01.2033: 3'780 × 1.03³ = 4'130.51 (previously 4'010.20 and 4'513.52).
// November instead of December: the December contains the 13th AHV pension (#190).
#[test]
fn issue_145_ak2_couple_cap_follows_inflation_once_per_year() {
    let res = capped_ahv(true, true, "start");
    assert!((ahv_in(&res, 10) - 3_780.0).abs() < 0.01, "{}", ahv_in(&res, 10));
    assert!((ahv_in(&res, 12) - 3_893.40).abs() < 0.01, "{}", ahv_in(&res, 12));
    assert!((ahv_in(&res, 36) - 4_130.51).abs() < 0.01, "{}", ahv_in(&res, 36));
}

// #145/AK3: Only the pension of A indexed, both running. 01.2031: A = 2'520 × 1.03 = 2'595.60, B = 2'520 fixed,
// sum 5'115.60; cap 3'780 × 1.03 = 3'893.40 → 3'893.40. Without a cap (unmarried) the full sum.
#[test]
fn issue_145_ak3_cap_with_only_one_indexed_pension() {
    let res = capped_ahv(true, false, "start");
    assert!((ahv_in(&res, 12) - 3_893.40).abs() < 0.01, "{}", ahv_in(&res, 12));
    // The cap only applies when it is below the sum: A 1'000 indexed, B 2'000 fixed.
    let mut v = with_partner_65(base());
    v["returns"]["fixed"]["inflation"] = json!(0.03);
    v["pension"]["ahv"] = json!({ "monthly": 1_000, "at": "start", "indexed": true });
    v["partner"]["ahv"] = json!({ "monthly": 2_000, "at": "start", "indexed": false });
    v["household"]["ahvCoupleCap"] = json!(3_780);
    let res = run(v);
    // 01.2031: 1'030 + 2'000 = 3'030 < 3'893.40
    assert!((ahv_in(&res, 12) - 3_030.0).abs() < 0.01, "{}", ahv_in(&res, 12));
}

// #145/AK4: The second pension starts in January 2032. 01.2031: only A, 2'520 × 1.03 = 2'595.60 (below the cap).
// 01.2032: A = 2'520 × 1.03² = 2'673.47, B starts with 2'520; sum 5'193.47, cap 3'780 × 1.03² = 4'010.20.
// 01.2033: both adjusted, cap 3'780 × 1.03³ = 4'130.51 (previously 4'254.42).
#[test]
fn issue_145_ak4_cap_when_second_pension_starts_later() {
    let res = capped_ahv(true, true, "b_ahv");
    assert!((ahv_in(&res, 12) - 2_595.60).abs() < 0.01, "{}", ahv_in(&res, 12));
    assert!((ahv_in(&res, 24) - 4_010.20).abs() < 0.01, "{}", ahv_in(&res, 24));
    assert!((ahv_in(&res, 36) - 4_130.51).abs() < 0.01, "{}", ahv_in(&res, 36));
}

#[test]
fn couple_tax_tables_replace_single_ones() {
    let mut v = with_partner(base());
    v["milestones"][1]["date"] = json!("2040-01");
    v["flows"]["income"] = json!(10_000);
    v["taxes"] = json!({
        "multiplier": 1.0,
        "income": [[0, 0.2]], "wealth": [], "capital": [],
        "couple": { "income": [[0, 0.1]], "wealth": [], "capital": [] }
    });
    let single = run(v.clone());
    assert!(
        (single.records[16].taxes - 120_000.0 * 0.1).abs() < 1e-6,
        "Verheiratetentarif"
    );
    v["household"]["married"] = json!(false);
    let unmarried = run(v);
    assert!((unmarried.records[16].taxes - 120_000.0 * 0.2).abs() < 1e-6);
}

#[test]
fn single_person_plan_ignores_couple_settings() {
    let mut v = base();
    v["taxes"]["couple"] = json!({ "income": [[0, 0.9]], "wealth": [], "capital": [] });
    v["flows"]["income"] = json!(1_000);
    v["milestones"][1]["date"] = json!("2040-01");
    v["taxes"]["income"] = json!([[0, 0.1]]);
    let res = run(v);
    assert!((res.records[16].taxes - 1_200.0).abs() < 1e-6);
}

// ---------------------------------------------------------------------------------------------
// Withdrawal strategies
// ---------------------------------------------------------------------------------------------

#[test]
fn fixed_rate_sets_spending_from_wealth() {
    let mut v = base();
    v["withdrawal"] = json!({ "strategy": "fixed_rate", "fixedRate": { "rate": 0.04 } });
    let res = run(v);
    // 4 % of 1'000'000 = 40'000 per year = 3'333.33 per month (instead of 5'000 planned)
    assert!((res.records[0].spending - 40_000.0 / 12.0).abs() < 1e-6);
    assert!((res.records[11].spending - 40_000.0 / 12.0).abs() < 1e-6);
    // January: 4 % of the remaining wealth (960'000)
    assert!(
        (res.records[12].spending - 38_400.0 / 12.0).abs() < 1e-6,
        "{}",
        res.records[12].spending
    );
    assert!(
        res.summary.depleted.is_none(),
        "eine Rate in Prozent des Vermögens erschöpft es nie"
    );
}

#[test]
fn fixed_rate_counts_income_and_floor_applies() {
    let mut v = base();
    v["flows"]["income"] = json!(1_000);
    v["withdrawal"] = json!({ "strategy": "fixed_rate", "fixedRate": { "rate": 0.04 } });
    let res = run(v.clone());
    // Living costs = withdrawal + income
    assert!((res.records[0].spending - (40_000.0 / 12.0 + 1_000.0)).abs() < 1e-6);
    // Floor 100 % of the planned spending, ceiling not set
    v["withdrawal"]["floor"] = json!(1.0);
    let res = run(v.clone());
    assert!((res.records[0].spending - 5_000.0).abs() < 1e-6);
    // ceiling 50 %
    v["withdrawal"].as_object_mut().unwrap().remove("floor");
    v["withdrawal"]["ceiling"] = json!(0.5);
    let res = run(v);
    assert!((res.records[0].spending - 2_500.0).abs() < 1e-6);
}

#[test]
fn strategy_starts_with_retirement_only() {
    let mut v = base();
    v["milestones"][1]["date"] = json!("2031-01");
    v["withdrawal"] = json!({ "strategy": "fixed_rate", "fixedRate": { "rate": 0.04 } });
    let res = run(v);
    assert!(
        (res.records[0].spending - 5_000.0).abs() < 1e-6,
        "vor der Pensionierung gilt die Planung"
    );
    assert!((res.records[12].spending - 0.04 * 940_000.0 / 12.0).abs() < 1e-6);
}

#[test]
fn guardrails_cut_spending_when_rate_gets_too_high() {
    let mut v = base();
    v["withdrawal"] = json!({
        "strategy": "guardrails",
        "guardrails": { "initialRate": 0.05, "upper": 0.2, "lower": 0.2, "adjust": 0.1 }
    });
    let res = run(v);
    let monthly = |year: usize| res.records[year * 12].spending * 12.0;
    assert!((monthly(0) - 50_000.0).abs() < 1e-6);
    // Without return and inflation the rate rises from 5 % to over 6 % from the 5th year
    // (wealth 800'000 → 6.25 %): cut by 10 % to 45'000.
    assert!((monthly(3) - 50_000.0).abs() < 1e-6);
    assert!((monthly(4) - 45_000.0).abs() < 1e-6, "{}", monthly(4));
    let cuts = res
        .events
        .iter()
        .filter(|e| e.kind == EventKind::Withdrawal && e.text.contains("cut by"))
        .count();
    assert!(cuts >= 1);
}

#[test]
fn guardrails_raise_spending_after_strong_markets() {
    let mut v = base();
    v["wealth"]["stocks"] = json!(1_000_000);
    v["wealth"]["cash"] = json!(0);
    v["returns"]["fixed"]["stocks"] = json!(0.20);
    v["withdrawal"] = json!({
        "strategy": "guardrails",
        "guardrails": { "initialRate": 0.05, "upper": 0.2, "lower": 0.2, "adjust": 0.1 }
    });
    let res = run(v);
    assert!(res
        .events
        .iter()
        .any(|e| e.kind == EventKind::Withdrawal && e.text.contains("raised by")));
    assert!(res.records[24].spending > res.records[0].spending);
}

#[test]
fn guardrails_follow_inflation() {
    let mut v = base();
    v["returns"]["fixed"]["inflation"] = json!(0.02);
    v["returns"]["fixed"]["stocks"] = json!(0.02);
    v["wealth"]["stocks"] = json!(1_000_000);
    v["wealth"]["cash"] = json!(0);
    v["withdrawal"] = json!({
        "strategy": "guardrails",
        "guardrails": { "initialRate": 0.04, "upper": 0.9, "lower": 0.9, "adjust": 0.1 }
    });
    let res = run(v);
    // Guardrails far away: the withdrawal grows with inflation
    let a = res.records[0].spending;
    let b = res.records[12].spending;
    assert!((b / a - 1.02).abs() < 1e-9, "{}", b / a);
}

#[test]
fn invalid_strategy_parameters_are_reported() {
    let mut v = base();
    v["withdrawal"] = json!({ "strategy": "fixed_rate", "fixedRate": { "rate": 0.0 } });
    assert!(validate::validate(&plan(v.clone()))
        .iter()
        .any(|i| i.severity == Severity::Error && i.section == "withdrawal"));
    v["withdrawal"] = json!({ "strategy": "guardrails", "guardrails": { "initialRate": 0.05, "upper": 0.2, "lower": 0.2, "adjust": 1.5 } });
    assert!(validate::validate(&plan(v.clone()))
        .iter()
        .any(|i| i.severity == Severity::Error && i.section == "withdrawal"));
    v["withdrawal"] = json!({ "strategy": "fixed_rate", "fixedRate": { "rate": 0.04 }, "floor": 1.2, "ceiling": 0.8 });
    assert!(validate::validate(&plan(v))
        .iter()
        .any(|i| i.severity == Severity::Error && i.section == "withdrawal"));
}

// ---------------------------------------------------------------------------------------------
// Evaluation
// ---------------------------------------------------------------------------------------------

#[test]
fn records_carry_market_indices() {
    let mut v = base();
    v["returns"]["fixed"]["stocks"] = json!(0.06);
    v["returns"]["fixed"]["bonds"] = json!(0.02);
    v["milestones"][1]["date"] = json!("2040-01");
    v["flows"]["spending"] = json!(0);
    let res = run(v);
    assert!((res.records[11].stock_index - 1.06).abs() < 1e-9);
    assert!((res.records[11].bond_index - 1.02).abs() < 1e-9);
    assert!((res.records[0].stock_index - 1.06f64.powf(1.0 / 12.0)).abs() < 1e-9);
}

#[test]
fn bands_and_percentiles_include_extremes() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures");
    let ds = DataSet::load_csv(&dir.join("renditen-muster.csv")).unwrap();
    let mut v = base();
    v["milestones"][1]["date"] = json!("2040-01");
    v["flows"]["spending"] = json!(0);
    v["wealth"]["cash"] = json!(0);
    v["wealth"]["stocks"] = json!(100_000);
    v["returns"]["source"] = json!("bootstrap");
    v["returns"]["series"] = json!("renditen-muster.csv");
    v["returns"]["bootstrap"] = json!({ "paths": 300, "block": 12, "seed": 3 });
    let p = plan(v);
    let sim = Simulator::new(&p).unwrap();
    let paths = batch::path_source(&p, &sim, Some(&ds)).unwrap();
    let (results, bands) = batch::run_with_bands(&sim, &paths);
    let summary = batch::summarize(&results);
    let q = |x: u32| summary.percentiles.iter().find(|p| p.percent == x).unwrap();
    let worst = results
        .iter()
        .map(|r| r.summary.end_total)
        .fold(f64::INFINITY, f64::min);
    assert_eq!(q(0).end_total, worst);
    assert!(q(0).end_total <= q(1).end_total && q(1).end_total <= q(99).end_total);
    for k in 0..sim.months() {
        assert!(bands.min[k] <= bands.p1[k] && bands.p1[k] <= bands.p5[k], "{k}");
        assert!(bands.p95[k] <= bands.p99[k], "{k}");
    }
    let last = sim.months() - 1;
    assert!((bands.min[last] - worst).abs() < 0.01 * worst.abs().max(1.0));
    let real = bands.real.as_ref().unwrap();
    assert!(real.min[last] <= real.p1[last]);
}

// #134/AK3: Capital table [[0, 10 %], [300'000, 1 %]], two lump sums in the same year: A 200'000 in January,
// B 300'000 in October. Tax on 200'000 = 200'000 × 4 % = 8'000; on 500'000 together = 500'000 × 1 % = 5'000.
// The second payout used to give −3'000; now 0 (never negative). The validation warns about the table.
#[test]
fn issue_134_ak3_capital_tax_of_a_second_withdrawal_is_never_negative() {
    let mut v = with_partner(base());
    v["milestones"]
        .as_array_mut()
        .unwrap()
        .push(json!({ "id": "retireB", "name": "Pensionierung B", "kind": "date", "date": "2030-10" }));
    v["pension"]["pk"] = json!({ "capital": 200_000, "at": "retire", "lumpsum": 1.0, "conversion": 0.05 });
    v["partner"]["pk"] = json!({ "capital": 300_000, "at": "retireB", "lumpsum": 1.0, "conversion": 0.05 });
    v["taxes"] = json!({ "multiplier": 1.0, "income": [], "wealth": [], "capital": [[0, 0.10], [300_000, 0.01]] });
    let res = run(v.clone());
    assert!(
        (res.records[0].taxes - 8_000.0).abs() < 0.01,
        "{}",
        res.records[0].taxes
    );
    assert!(res.records.iter().all(|r| r.taxes >= 0.0), "keine negative Steuer");
    assert!(
        res.records[9].taxes.abs() < 0.01,
        "zweiter Bezug: {}",
        res.records[9].taxes
    );
    assert!(
        (res.summary.taxes_total - 8_000.0).abs() < 0.01,
        "{}",
        res.summary.taxes_total
    );
    let issues = validate::validate(&plan(v));
    assert!(issues.iter().any(|i| i.severity == Severity::Warning
        && i.field.as_deref() == Some("taxes.capital.1.1")
        && i.message.contains("the tax falls")));
}

// ---------------------------------------------------------------------------------------------
// #155: Lump sums of a married couple, taxed together per calendar year
// ---------------------------------------------------------------------------------------------

/// #155: Married couple from January 2030, A draws 200'000 at `date_a`, B 300'000 at `date_b`, both fully as a
/// lump sum. Capital table [[0, 4 %], [1'000'000, 8 %]] (rate interpolated linearly), multiplier 1, no other
/// taxes, flows or wealth. `married` is set explicitly: #150 taxes unmarried couples per person.
fn issue_155_married_lump_sums(date_a: &str, date_b: &str) -> SimResult {
    let mut v = crate::plan_builder::PlanBuilder::new("Test")
        .couple("1967-01")
        .married(true)
        .set(
            "partner",
            json!({ "income": 0, "ahv": { "monthly": 0, "at": "never" },
                "pk": { "capital": 300_000, "at": "pkB", "lumpsum": 1.0, "conversion": 0.05 } }),
        )
        .set(
            "taxes",
            json!({ "multiplier": 1.0, "income": [], "wealth": [], "capital": [[0, 0.04], [1_000_000, 0.08]] }),
        )
        .build();
    v["pension"]["pk"] = json!({ "capital": 200_000, "at": "pkA", "lumpsum": 1.0, "conversion": 0.05 });
    let milestones = v["milestones"].as_array_mut().unwrap();
    milestones.push(json!({ "id": "pkA", "name": "Bezug A", "kind": "date", "date": date_a }));
    milestones.push(json!({ "id": "pkB", "name": "Bezug B", "kind": "date", "date": date_b }));
    run(v)
}

/// Capital tax deducted per month index (0 = January 2030), only months with a deduction.
fn capital_tax_months(res: &SimResult) -> Vec<(usize, f64)> {
    res.records
        .iter()
        .enumerate()
        .filter(|(_, r)| r.tax_capital != 0.0)
        .map(|(k, r)| (k, r.tax_capital))
        .collect()
}

fn assert_months(actual: &[(usize, f64)], expected: &[(usize, f64)]) {
    assert_eq!(actual.len(), expected.len(), "{actual:?}");
    for (&(k, t), &(ke, te)) in actual.iter().zip(expected) {
        assert_eq!(k, ke, "{actual:?}");
        assert!((t - te).abs() < 0.01, "Monat {k}: {t} statt {te}");
    }
}

// #155/AK1: Both lump sums in 2030 (March, October). Tax on the year's sum 500'000 × 6 % = 30'000;
// first payout 200'000 × 4.8 % = 9'600, second = 30'000 − 9'600 = 20'400, each deducted in its month.
#[test]
fn issue_155_ak1_married_lump_sums_in_the_same_year_are_taxed_together() {
    let res = issue_155_married_lump_sums("2030-03", "2030-10");
    let months = capital_tax_months(&res);
    assert_months(&months, &[(2, 9_600.0), (9, 20_400.0)]);
    // Deducted in the month of the payout: also in the month's total tax.
    assert!(
        (res.records[2].taxes - 9_600.0).abs() < 0.01,
        "{}",
        res.records[2].taxes
    );
    assert!(
        (res.records[9].taxes - 20_400.0).abs() < 0.01,
        "{}",
        res.records[9].taxes
    );
    let year: f64 = res.records[..12].iter().map(|r| r.tax_capital).sum();
    assert!((year - 30_000.0).abs() < 0.01, "Jahressumme {year}");
    assert!(
        (res.summary.lumpsum_tax - 30_000.0).abs() < 0.01,
        "{}",
        res.summary.lumpsum_tax
    );
    assert!(
        (res.summary.lumpsum_net - 470_000.0).abs() < 0.01,
        "{}",
        res.summary.lumpsum_net
    );
}

// #155/AK2: Lump sums in two years (March 2030, October 2031): each year is taxed on its own.
// 2030: 200'000 × 4.8 % = 9'600; 2031: 300'000 × 5.2 % = 15'600; together 25'200 instead of 30'000.
#[test]
fn issue_155_ak2_married_lump_sums_in_different_years_are_taxed_per_year() {
    let res = issue_155_married_lump_sums("2030-03", "2031-10");
    let months = capital_tax_months(&res);
    assert_months(&months, &[(2, 9_600.0), (21, 15_600.0)]);
    let y2030: f64 = res.records[..12].iter().map(|r| r.tax_capital).sum();
    let y2031: f64 = res.records[12..24].iter().map(|r| r.tax_capital).sum();
    assert!((y2030 - 9_600.0).abs() < 0.01, "2030: {y2030}");
    assert!((y2031 - 15_600.0).abs() < 0.01, "2031: {y2031}");
    assert!(
        (res.summary.lumpsum_tax - 25_200.0).abs() < 0.01,
        "{}",
        res.summary.lumpsum_tax
    );
}

// ---------------------------------------------------------------------------------------------
// #150: Lump sums of an unmarried couple, taxed per person
// ---------------------------------------------------------------------------------------------

/// #150: Couple from January 2030 with `married`, A draws 500'000 in March, B 500'000 in October 2030, both fully
/// as a lump sum. Capital table [[0, 4 %], [1'000'000, 8 %]] (rate interpolated linearly), multiplier 1, no
/// other taxes, flows or wealth.
fn issue_150_lump_sums(married: bool) -> SimResult {
    let mut v = crate::plan_builder::PlanBuilder::new("Test")
        .couple("1967-01")
        .married(married)
        .set(
            "partner",
            json!({ "income": 0, "ahv": { "monthly": 0, "at": "never" },
                "pk": { "capital": 500_000, "at": "pkB", "lumpsum": 1.0, "conversion": 0.05 } }),
        )
        .set(
            "taxes",
            json!({ "multiplier": 1.0, "income": [], "wealth": [], "capital": [[0, 0.04], [1_000_000, 0.08]] }),
        )
        .build();
    v["pension"]["pk"] = json!({ "capital": 500_000, "at": "pkA", "lumpsum": 1.0, "conversion": 0.05 });
    let milestones = v["milestones"].as_array_mut().unwrap();
    milestones.push(json!({ "id": "pkA", "name": "Bezug A", "kind": "date", "date": "2030-03" }));
    milestones.push(json!({ "id": "pkB", "name": "Bezug B", "kind": "date", "date": "2030-10" }));
    run(v)
}

// #150/AK1, AK3: Unmarried, 500'000 each in 2030. Each person pays the tax on 500'000 = 500'000 × 6 % = 30'000;
// together 2 × 30'000 = 60'000 (household total 1'000'000 × 8 % = 80'000 would be wrong).
#[test]
fn issue_150_ak1_unmarried_lump_sums_are_taxed_per_person() {
    let res = issue_150_lump_sums(false);
    assert_months(&capital_tax_months(&res), &[(2, 30_000.0), (9, 30_000.0)]);
    assert!(
        (res.summary.lumpsum_tax - 60_000.0).abs() < 0.01,
        "{}",
        res.summary.lumpsum_tax
    );
    assert!(
        (res.summary.lumpsum_net - 940_000.0).abs() < 0.01,
        "{}",
        res.summary.lumpsum_net
    );
}

// #150/AK2, AK3: Same plan married: the year's sum 1'000'000 × 8 % = 80'000; first 30'000, second 50'000.
#[test]
fn issue_150_ak3_married_lump_sums_stay_taxed_together() {
    let res = issue_150_lump_sums(true);
    assert_months(&capital_tax_months(&res), &[(2, 30_000.0), (9, 50_000.0)]);
    assert!(
        (res.summary.lumpsum_tax - 80_000.0).abs() < 0.01,
        "{}",
        res.summary.lumpsum_tax
    );
}
