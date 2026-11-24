//! #253: «Ohne Pensionskasse» switched on: the person's pension fund counts as 0 (no capital, contributions, pension or
//! lump sum with its capital tax); the entered values stay in the file and count again once the switch is off.
//!
//! Made-up plan that can be recomputed by hand: start 2030-01, end 2032-12, person «A» born 1966-01 with payout at 65
//! (2031-01), no returns, no inflation, own tables with a flat capital tax of 10 %, no income or wealth tax.

use engine::model::Plan;
use engine::sim::{SimResult, Simulator};
use engine::time::YearMonth;
use engine::validate::validate;
use serde_json::{json, Value};

/// Person «A» with pension fund capital 500'000, conversion rate 5 %, lump sum share `lumpsum`, contributions
/// `contribution` per month and the switch «ohne Pensionskasse» = `off`.
fn plan(lumpsum: f64, contribution: f64, off: bool) -> Value {
    json!({
        "project": { "name": "Ohne Pensionskasse" },
        "household": { "persons": [ { "id": "A", "name": "A", "birth": "1966-01", "noPensionFund": off } ] },
        "milestones": [
            { "id": "start", "name": "Start", "kind": "date", "date": "2030-01", "fixed": true },
            { "id": "retire", "name": "Bezug Pensionskasse", "kind": "age", "person": "A", "years": 65 },
            { "id": "ahv", "name": "AHV-Bezug", "kind": "age", "person": "A", "years": 65 },
            { "id": "end", "name": "Ende", "kind": "date", "date": "2032-12", "fixed": true }
        ],
        "pension": { "pk": { "capital": 500_000, "at": "retire", "lumpsum": lumpsum, "conversion": 0.05,
                             "contribution": contribution, "interest": 0 },
                     "ahv": { "monthly": 0, "at": "ahv" } },
        "flows": { "income": 0, "spending": 1000 },
        "wealth": { "stocks": 0, "bonds": 0, "cash": 1_000_000, "stockShare": 0, "reserveMonths": 0 },
        "returns": { "source": "fixed", "fixed": { "stocks": 0, "bonds": 0, "inflation": 0, "saron": 0 } },
        "taxes": { "multiplier": 1.0, "income": [[0, 0.0]], "wealth": [], "capital": [[0, 0.1]],
                   "propertyGain": { "brackets": [[0, 0]], "reduction": 0.0 } }
    })
}

fn run(v: &Value) -> SimResult {
    let plan: Plan = serde_json::from_value(v.clone()).expect("plan readable");
    let s = Simulator::new(&plan).expect("plan valid");
    let path = engine::market::fixed_path(&s.fixed, s.start, s.months());
    s.run(&path, true)
}

fn close(a: f64, b: f64, what: &str) {
    assert!((a - b).abs() < 1e-6 * b.abs().max(1.0), "{what}: {a} instead of {b}");
}

/// Sum of the pension fund contributions over all months.
fn contributions(r: &SimResult) -> f64 {
    r.records.iter().map(|x| x.ledger.pk_contributions).sum()
}

/// #253/AK3: Example of the issue: capital 500'000, conversion rate 5 %. Switch off: pension 500'000 × 5 % / 12 =
/// 2'083.33 per month as before; switch on: pension 0 and no lump sum.
#[test]
fn issue_253_ak3_pension_zero_with_switch() {
    let on = run(&plan(0.0, 0.0, false));
    close(
        on.summary.pension_fund_monthly,
        500_000.0 * 0.05 / 12.0,
        "pension without switch",
    );
    let off = run(&plan(0.0, 0.0, true));
    close(off.summary.pension_fund_monthly, 0.0, "pension with switch");
    close(off.summary.lumpsum_net, 0.0, "lump sum with switch");
    let jan = off.records.iter().find(|x| x.month == YearMonth::new(2030, 1)).unwrap();
    close(jan.pension_fund, 0.0, "pension fund capital with switch");
    assert!(off.records.iter().all(|x| x.pk_pension == 0.0 && x.pension_fund == 0.0));
}

/// #253/AK3: Lump sum 50 % and contributions 1'000 per month (12 months until the payout 2031-01): without the switch
/// capital 512'000, lump sum 256'000 with capital tax 25'600, pension 256'000 × 5 % / 12 = 1'066.67; with the switch
/// no contributions, no lump sum, no capital tax and no pension.
#[test]
fn issue_253_ak3_no_contributions_lump_sum_or_capital_tax() {
    let on = run(&plan(0.5, 1000.0, false));
    close(contributions(&on), 12_000.0, "contributions without switch");
    close(on.summary.lumpsum_tax, 25_600.0, "capital tax without switch");
    close(
        on.summary.lumpsum_net,
        256_000.0 - 25_600.0,
        "net lump sum without switch",
    );
    close(
        on.summary.pension_fund_monthly,
        256_000.0 * 0.05 / 12.0,
        "pension without switch",
    );
    let off = run(&plan(0.5, 1000.0, true));
    close(contributions(&off), 0.0, "contributions with switch");
    close(off.summary.lumpsum_tax, 0.0, "capital tax with switch");
    close(off.summary.lumpsum_net, 0.0, "net lump sum with switch");
    close(off.summary.pension_fund_monthly, 0.0, "pension with switch");
    assert!(off.records.iter().all(|x| x.tax_capital == 0.0));
}

/// #253/AK3: The entered values stay in the file (saved unchanged) and count again once the switch is off.
#[test]
fn issue_253_ak3_values_stay_in_the_file() {
    let v = plan(0.5, 1000.0, true);
    let p: Plan = serde_json::from_value(v.clone()).unwrap();
    let out = serde_json::to_value(&p).unwrap();
    assert_eq!(out["pension"]["pk"]["capital"], json!(500_000.0));
    assert_eq!(out["pension"]["pk"]["conversion"], json!(0.05));
    assert_eq!(out["pension"]["pk"]["lumpsum"], json!(0.5));
    let mut again = out.clone();
    again["household"]["persons"][0]["noPensionFund"] = json!(false);
    let (a, b) = (run(&again), run(&plan(0.5, 1000.0, false)));
    assert_eq!(a.summary.pension_fund_monthly, b.summary.pension_fund_monthly);
    assert_eq!(a.summary.lumpsum_tax, b.summary.lumpsum_tax);
}

/// #253/AK3: The warning of #152 (pension fund entered and «ohne Pensionskasse») no longer exists.
#[test]
fn issue_253_ak3_no_warning_pension_fund_and_switch() {
    let p: Plan = serde_json::from_value(plan(0.5, 1000.0, true)).unwrap();
    let issues = validate(&p);
    assert!(
        issues.iter().all(|i| i.message.code() != "pillar3a_large_with_pk"),
        "{:?}",
        issues.iter().map(|i| i.message.code()).collect::<Vec<_>>()
    );
}

/// #253/AK3: For the second person only their own pension fund counts as 0; person 1 keeps theirs.
#[test]
fn issue_253_ak3_second_person_only() {
    let mut v = plan(0.0, 0.0, false);
    v["household"]["persons"] = json!([
        { "id": "A", "name": "A", "birth": "1966-01" },
        { "id": "B", "name": "B", "birth": "1966-01", "noPensionFund": true } ]);
    v["milestones"].as_array_mut().unwrap().extend([
        json!({ "id": "retire2", "name": "Bezug Pensionskasse Person 2", "kind": "age", "person": "B", "years": 65 }),
        json!({ "id": "ahv2", "name": "AHV-Bezug Person 2", "kind": "age", "person": "B", "years": 65 }),
    ]);
    v["partner"] = json!({ "income": 0,
        "pk": { "capital": 300_000, "at": "retire2", "lumpsum": 0, "conversion": 0.05 },
        "ahv": { "monthly": 0, "at": "ahv2" } });
    close(run(&v).summary.pension_fund_monthly, 500_000.0 * 0.05 / 12.0, "only A");
    v["household"]["persons"][1]["noPensionFund"] = json!(false);
    close(run(&v).summary.pension_fund_monthly, 800_000.0 * 0.05 / 12.0, "A and B");
}
