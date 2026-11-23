//! #196: Switch «Immobilien nie verkaufen» (`realestate.neverSell`): no planned sale, no emergency sale, no sale for
//! lack of affordability at a renewal; purchases with a date take place, purchases and rents «nach Verkauf» do not;
//! with too little liquid wealth the run counts as depleted.
//!
//! Base plan (made up, as in `sim_rules.rs`): start 2026-11, retirement and AHV January 2027; without taxes,
//! inflation and returns; income 10'000 + PK pension 1'000 + AHV 1'500 per month; wealth as cash.

use engine::api;
use engine::model::Plan;
use engine::sim::{EventKind, SimResult, Simulator};
use engine::time::YearMonth;
use engine::validate::Severity;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};

fn ym(y: i32, m: u32) -> YearMonth {
    YearMonth::new(y, m)
}

fn base() -> Value {
    let mut v: Value = serde_json::from_str(include_str!("../assets/new.vplan")).unwrap();
    v["returns"]["fixed"] = json!({"stocks": 0, "bonds": 0, "inflation": 0, "saron": 0});
    v["taxes"] = json!({"multiplier": 1, "income": [[0, 0]], "wealth": [[0, 0]], "capital": [[0, 0]],
        "propertyGain": {"brackets": [[0, 0]]}});
    v["flows"]["spending"] = json!(1000);
    v["flows"]["spendingIncludesHousing"] = json!(false);
    v["flows"]["incomeIndexed"] = json!(false);
    v["flows"]["income"] = json!(10_000);
    v["wealth"] = json!({"stocks": 0, "bonds": 0, "cash": 1_000_000, "stockShare": 0.5, "reserveMonths": 0});
    v["milestones"][1] = json!({"id": "retire", "kind": "date", "name": "Pensionierung", "date": "2027-01"});
    v["milestones"][2] = json!({"id": "ahv", "kind": "date", "name": "AHV", "date": "2027-01"});
    v["household"]["persons"][0]["birth"] = json!("1962-01");
    v["pension"]["pk"] = json!({"capital": 240_000, "at": "retire", "lumpsum": 0, "conversion": 0.05});
    v["pension"]["ahv"] = json!({"monthly": 1500, "at": "ahv", "indexed": true});
    v["realestate"]["emergencyMonths"] = json!(0);
    v["realestate"]["affordability"] = json!({"sustainability": 0.25, "imputedRate": 0.0625, "fixCosts": 0.01,
        "wealthContribution": 0.03, "maxLoanToValue": 0.8});
    v
}

/// Owned house «alt» (worth 900'000) with a mortgage of `debt` at 2 % (new period 2027-11 at 3 %, then 4 %).
fn house(sell: &str, debt: f64) -> Value {
    json!({"id": "alt", "name": "Haus", "status": "owned", "price": 600_000, "worth": 900_000, "sell": sell,
        "fixCosts": 100, "inflation": false,
        "mortgage": {"tranches": [{"name": "T1", "amount": debt, "start": "2025-11", "periods": [
            {"type": "fixed", "term": 2, "rate": 0.02}, {"type": "fixed", "term": 1, "rate": 0.03},
            {"type": "fixed", "term": 10, "rate": 0.04}]}]}})
}

fn planned(id: &str, buy: &str) -> Value {
    json!({"id": id, "name": id, "status": "planned", "price": 300_000, "buy": buy, "sell": "never", "fixCosts": 0,
        "inflation": false})
}

fn rent(from: &str) -> Value {
    json!({"name": "Wohnung", "status": "rent", "price": 0, "fixCosts": 0, "inflation": false, "from": from, "rent": 1500})
}

fn with(props: Value, never_sell: bool) -> Value {
    let mut v = base();
    v["realestate"]["properties"] = props;
    if never_sell {
        v["realestate"]["neverSell"] = json!(true);
    }
    v
}

fn run(v: &Value) -> SimResult {
    let plan: Plan = serde_json::from_value(crate::any_age(v.clone())).unwrap();
    let sim = Simulator::new(&plan).unwrap();
    let path = engine::market::fixed_path(&sim.fixed, sim.start, sim.months());
    sim.run(&path, true)
}

fn sales(r: &SimResult) -> usize {
    r.events
        .iter()
        .filter(|e| matches!(e.kind, EventKind::Sale | EventKind::EmergencySale))
        .count()
}

fn purchases(r: &SimResult) -> Vec<String> {
    r.events
        .iter()
        .filter(|e| e.kind == EventKind::Purchase)
        .map(|e| e.text.to_string())
        .collect()
}

// #196/AK1: The field is missing in older files and then off; «aus» is not written, «an» is kept.
#[test]
fn issue_196_ak1_field_defaults_to_off_and_is_saved() {
    for file in ["assets/example.vplan", "assets/new.vplan"] {
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(file);
        let value = api::read_plan(&path).unwrap();
        assert!(value["realestate"].get("neverSell").is_none(), "{file}");
        let plan: Plan = serde_json::from_value(value).unwrap();
        assert!(!plan.realestate.never_sell, "{file}");
        let back = serde_json::to_value(&plan).unwrap();
        assert!(back["realestate"].get("neverSell").is_none(), "{file}");
    }
    let mut plan: Plan = serde_json::from_value(base()).unwrap();
    plan.realestate.never_sell = true;
    let text = serde_json::to_string(&plan).unwrap();
    assert!(text.contains("\"neverSell\":true"));
    let again: Plan = serde_json::from_str(&text).unwrap();
    assert!(again.realestate.never_sell);
}

// #196/AK3: no planned sale; interest, amortization and costs run until the end of the plan.
#[test]
fn issue_196_ak3_no_planned_sale_and_costs_continue() {
    let sold = run(&with(json!([house("2030-01", 300_000.0)]), false));
    assert_eq!(sales(&sold), 1);
    assert_eq!(sold.records.last().unwrap().mortgage, 0.0);
    let kept = run(&with(json!([house("2030-01", 300_000.0)]), true));
    assert_eq!(sales(&kept), 0);
    let last = kept.records.last().unwrap();
    assert_eq!(last.mortgage, 300_000.0);
    assert_eq!(last.property_value, 900_000.0);
    // Interest 4 % on 300'000 plus fix costs 100 per month
    assert!(
        last.housing >= 100.0 + 300_000.0 * 0.04 / 12.0 - 1e-6,
        "{}",
        last.housing
    );
    assert!(kept.records.iter().all(|r| r.housing > 0.0));
}

// #196/AK3: no emergency sale, even when the emergency sale criterion is met from the start.
#[test]
fn issue_196_ak3_no_emergency_sale() {
    let mut on = with(json!([house("never", 300_000.0)]), true);
    on["realestate"]["emergencyMonths"] = json!(1000);
    on["flows"]["income"] = json!(0);
    let mut off = on.clone();
    off["realestate"]["neverSell"] = json!(false);
    assert_eq!(run(&off).summary.emergency_sales, 1);
    let r = run(&on);
    assert_eq!((r.summary.emergency_sales, sales(&r)), (0, 0));
}

// #196/AK3: not affordable at the renewal and the lump sum exceeds the wealth: the mortgage is renewed in full, an
// event reports «nicht tragbar», no sale follows.
#[test]
fn issue_196_ak3_unaffordable_renewal_renews_in_full() {
    let mut v = with(json!([house("never", 680_800.0)]), true);
    v["wealth"]["cash"] = json!(30_000);
    v["flows"]["spending"] = json!(11_000);
    let r = run(&v);
    let n = &r.renewals[0];
    assert_eq!(n.month, ym(2027, 11));
    assert!(!n.affordable);
    assert_eq!(n.emergency_sale, None);
    assert_eq!((n.lump_sum, n.debt_before, n.debt_after), (0.0, 680_800.0, 680_800.0));
    assert_eq!(
        (r.summary.affordability_sales, r.summary.emergency_sales, sales(&r)),
        (0, 0, 0)
    );
    let e = r
        .events
        .iter()
        .find(|e| e.kind == EventKind::MortgageRenewal && e.month == ym(2027, 11))
        .unwrap();
    assert_eq!(e.text.code(), "event_mortgage_unaffordable_kept");
    assert!(e.text.to_string().contains("not affordable"), "{}", e.text);
    // The new period runs on the full debt (3 %).
    let rate = r
        .property_events
        .iter()
        .find(|p| p.month == ym(2027, 11) && p.rate.is_some())
        .unwrap()
        .rate;
    assert_eq!(rate, Some(0.03));
    assert_eq!(r.records.last().unwrap().mortgage, 680_800.0);
}

// #196/AK3: 3-bucket strategy: when all buckets are empty there is no emergency sale (phase 8 instead).
#[test]
fn issue_196_ak3_buckets_do_not_sell() {
    let mut v = with(json!([house("never", 300_000.0)]), true);
    v["withdrawal"]["strategy"] = json!("three_buckets");
    v["withdrawal"]["buckets"] = json!({"cashYears": 1, "bondYears": 2, "refillThreshold": 0});
    v["wealth"]["cash"] = json!(50_000);
    v["flows"]["income"] = json!(0);
    v["flows"]["spending"] = json!(8_000);
    let mut off = v.clone();
    off["realestate"]["neverSell"] = json!(false);
    assert!(run(&off).summary.emergency_sales > 0);
    let r = run(&v);
    assert_eq!(sales(&r), 0);
    assert!(r
        .events
        .iter()
        .any(|e| e.kind == EventKind::Buckets && e.text.code() == "event_buckets_empty"));
    assert!(r.summary.depleted.is_some());
}

// #196/AK4: a purchase with a date takes place; purchases and rents «nach Verkauf» do not.
#[test]
fn issue_196_ak4_purchase_with_date_takes_place() {
    let props = json!([
        house("2030-01", 0.0),
        planned("datiert", "2032-01"),
        planned("folge", "afterSale:alt"),
        rent("afterSale")
    ]);
    let off = run(&with(props.clone(), false));
    assert_eq!(purchases(&off).len(), 2, "{:?}", purchases(&off));
    let on = run(&with(props, true));
    let bought = purchases(&on);
    assert_eq!(bought.len(), 1, "{bought:?}");
    assert!(bought[0].contains("datiert"), "{bought:?}");
    assert!(!on.events.iter().any(|e| e.kind == EventKind::RentStart));
    assert_eq!(on.records.last().unwrap().property_value, 1_200_000.0);
}

// #196/AK5: without enough liquid wealth the run counts as depleted although a property is still owned.
#[test]
fn issue_196_ak5_depleted_with_property() {
    let mut v = with(json!([house("never", 300_000.0)]), true);
    v["wealth"]["cash"] = json!(100_000);
    v["flows"]["income"] = json!(0);
    v["flows"]["spending"] = json!(6_000);
    v["realestate"]["emergencyMonths"] = json!(3);
    let r = run(&v);
    assert!(r.summary.depleted.is_some());
    assert!(r.events.iter().any(|e| e.kind == EventKind::Depleted));
    assert_eq!(sales(&r), 0);
    // Without the switch the emergency sale rescues the plan here.
    v["realestate"]["neverSell"] = json!(false);
    let r = run(&v);
    assert!(r.summary.emergency_sales > 0);
}

fn library() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data")
}

/// Example plan with bootstrap over Shiller CHF (200 runs).
fn example(never_sell: bool) -> engine::batch::BatchSummary {
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("assets/example.vplan");
    let mut v = api::read_plan(&path).unwrap();
    v["returns"]["series"] = json!("library:Shiller CHF.csv");
    v["returns"]["source"] = json!("bootstrap");
    v["returns"]["bootstrap"] = json!({"paths": 200, "block": 12, "seed": 2026});
    v["realestate"]["neverSell"] = json!(never_sell);
    let out = api::run(&v, &library(), Some(&library()));
    assert!(out.error.is_none(), "{:?}", out.error);
    out.batch.unwrap().summary
}

// #196/AK5: the example plan has a lower success rate with the switch and no run with an emergency sale.
// slow: two Monte Carlo runs of the example plan
#[test]
fn issue_196_ak5_example_plan_lower_success_without_sales() {
    let off = example(false);
    let on = example(true);
    assert!(
        on.success_rate < off.success_rate,
        "{} vs. {}",
        on.success_rate,
        off.success_rate
    );
    assert_eq!((on.emergency_sale_runs, on.affordability_sale_runs), (0, 0));
}

// #196/AK6: the validation points out sales and purchases or rents «nach Verkauf» that are not carried out.
#[test]
fn issue_196_ak6_validation_warns_about_ignored_sales() {
    let props = json!([
        house("2030-01", 0.0),
        planned("datiert", "2032-01"),
        planned("folge", "afterSale:alt"),
        rent("afterSale")
    ]);
    let codes = |never_sell: bool| -> Vec<(String, Severity)> {
        api::validate(&with(props.clone(), never_sell))
            .into_iter()
            .filter(|i| i.message.code().starts_with("property_never_sell"))
            .map(|i| (i.message.to_string(), i.severity))
            .collect()
    };
    assert!(codes(false).is_empty());
    let on = codes(true);
    assert_eq!(on.len(), 3, "{on:?}");
    assert!(on.iter().all(|(_, s)| *s == Severity::Warning));
    assert!(on[0].0.starts_with("Haus:") && on[0].0.contains("sale is not carried out"));
    assert!(on[1].0.starts_with("folge:") && on[2].0.starts_with("Wohnung:"));
    // A plan with the switch but without sales: no warning.
    let quiet = api::validate(&with(json!([house("never", 0.0), planned("datiert", "2032-01")]), true));
    assert!(!quiet
        .iter()
        .any(|i| i.message.code().starts_with("property_never_sell")));
}

// #196/AK8: switched off, the run is identical to a plan without the field (golden files: `golden.rs`).
#[test]
fn issue_196_ak8_off_equals_missing_field() {
    let props = json!([house("2030-01", 300_000.0), planned("folge", "afterSale:alt")]);
    let missing = with(props.clone(), false);
    let mut off = missing.clone();
    off["realestate"]["neverSell"] = json!(false);
    let (a, b) = (run(&missing), run(&off));
    assert_eq!(serde_json::to_value(&a).unwrap(), serde_json::to_value(&b).unwrap());
    assert_eq!(sales(&a), 1);
}

/// Plan in which the renewal 2027-11 is not affordable; with `cash` the lump sum can or cannot be paid.
fn unaffordable(cash: f64, never_sell: bool) -> Value {
    let mut v = with(json!([house("never", 680_800.0)]), never_sell);
    v["wealth"]["cash"] = json!(cash);
    v["flows"]["spending"] = json!(11_000);
    v
}

fn kept_events(r: &SimResult) -> usize {
    r.events
        .iter()
        .filter(|e| e.text.code() == "event_mortgage_unaffordable_kept")
        .count()
}

// #199/AK1: each renewal kept only because of the switch (event «nicht tragbar … voll erneuert») is counted; a paid
// lump sum is not, it stays a lump sum.
#[test]
fn issue_199_ak1_counts_renewals_kept_by_never_sell() {
    let kept = run(&unaffordable(30_000.0, true));
    assert!(kept.summary.unaffordable_renewals >= 1);
    assert_eq!(kept.summary.unaffordable_renewals, kept_events(&kept));
    assert_eq!(
        kept.summary.unaffordable_renewals,
        kept.renewals
            .iter()
            .filter(|n| !n.affordable && n.lump_sum == 0.0)
            .count()
    );
    assert_eq!(kept.summary.lump_sums, 0);
    let paid = run(&unaffordable(1_000_000.0, true));
    assert!(paid.renewals[0].lump_sum > 0.0);
    assert_eq!(paid.summary.unaffordable_renewals, kept_events(&paid));
    assert_eq!(paid.summary.unaffordable_renewals, 0);
    assert!(paid.summary.lump_sums >= 1);
}

// #199/AK5: without the switch the count is always 0 (here an emergency sale instead), and the field is missing.
#[test]
fn issue_199_ak5_zero_without_switch_and_missing_in_json() {
    let off = run(&unaffordable(30_000.0, false));
    assert_eq!(off.summary.unaffordable_renewals, 0);
    assert_eq!(off.summary.affordability_sales, 1);
    let json = serde_json::to_value(&off.summary).unwrap();
    assert!(json.get("unaffordableRenewals").is_none(), "{json}");
    let kept = serde_json::to_value(&run(&unaffordable(30_000.0, true)).summary).unwrap();
    assert!(kept["unaffordableRenewals"].as_u64().unwrap() >= 1, "{kept}");
}

// #199/AK2, AK3, AK5: the example plan with bootstrap counts runs with such a renewal only with the switch on; the
// field is missing in the batch JSON when 0.
// slow: two Monte Carlo runs of the example plan
#[test]
fn issue_199_ak2_example_plan_counts_runs() {
    let on = example(true);
    assert!(on.unaffordable_renewal_runs > 0 && on.unaffordable_renewal_runs <= on.runs);
    assert_eq!(
        serde_json::to_value(&on).unwrap()["unaffordableRenewalRuns"],
        json!(on.unaffordable_renewal_runs)
    );
    let off = example(false);
    assert_eq!(off.unaffordable_renewal_runs, 0);
    assert!(serde_json::to_value(&off)
        .unwrap()
        .get("unaffordableRenewalRuns")
        .is_none());
}
