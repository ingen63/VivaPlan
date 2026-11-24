//! #209: Yearly values, digest and comparison of results in the engine (`api::yearly`, `api::digest`, `api::compare`),
//! read from results as the user interface keeps and saves them (JSON).

use std::path::{Path, PathBuf};

use engine::api;
use engine::digest::{self, NextStep, ResultKind, Verdict};
use serde_json::{json, Value};

use crate::plan_builder::{parametric, PlanBuilder};

fn library() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data")
}

/// The example plan with fixed returns (one run) and without variations.
fn fixed_plan() -> Value {
    let mut v = PlanBuilder::example().build();
    v["returns"]["source"] = json!("fixed");
    v["scenarios"]["sweeps"] = json!([]);
    v
}

/// The example plan with 300 parametric paths and without variations.
fn mc_plan() -> Value {
    let mut v = PlanBuilder::example().parametric(parametric(300, 7)).build();
    v["scenarios"]["sweeps"] = json!([]);
    v
}

/// Answer of `run_simulation` as the user interface receives it (through JSON text).
fn result(plan: &Value) -> Value {
    let out = api::run(plan, &library(), Some(&library()));
    assert!(out.error.is_none(), "{:?}", out.error);
    serde_json::from_str(&serde_json::to_string(&out).unwrap()).unwrap()
}

fn num(v: &Value) -> f64 {
    v.as_f64().unwrap_or_else(|| panic!("number: {v}"))
}

/// #209/AK1: One row per calendar year; balances of the last month of the year, flows as monthly averages and sums,
/// taxes as sums, the withdrawal rate against total wealth at the start of the year; real = nominal in prices of the
/// start month month by month.
#[test]
fn issue_209_ak1_yearly_values_per_calendar_year_from_the_months() {
    let run = result(&fixed_plan())["single"].clone();
    let records = run["records"].as_array().unwrap();
    let y = serde_json::to_value(api::yearly(&run).unwrap()).unwrap();
    let (nominal, real) = (y["nominal"].as_array().unwrap(), y["real"].as_array().unwrap());
    let years: Vec<&str> = records.iter().map(|r| &r["month"].as_str().unwrap()[..4]).collect();
    let mut distinct = years.clone();
    distinct.dedup();
    assert_eq!(nominal.len(), distinct.len());
    assert_eq!(real.len(), distinct.len());
    assert_eq!(
        nominal.iter().map(|r| r["months"].as_u64().unwrap()).sum::<u64>(),
        records.len() as u64
    );
    let mut start_total = num(&run["summary"]["startTotal"]);
    let mut with_rate = 0;
    for (row, year) in nominal.iter().zip(&distinct) {
        assert_eq!(row["year"].to_string(), *year);
        let rs: Vec<&Value> = records
            .iter()
            .filter(|r| r["month"].as_str().unwrap().starts_with(year))
            .collect();
        let last = rs[rs.len() - 1];
        let n = rs.len() as f64;
        let sum = |k: &str| rs.iter().map(|r| num(&r[k])).sum::<f64>();
        assert_eq!(num(&row["total"]), num(&last["total"]));
        assert_eq!(num(&row["age"]), num(&last["age"]));
        assert!((num(&row["draw"]) - sum("withdrawal") / n).abs() < 1e-6);
        assert!((num(&row["flows"]["taxes"]) - sum("taxes")).abs() < 1e-6);
        assert!((num(&row["flows"]["ahv"]) - sum("ahv")).abs() < 1e-6);
        assert!(
            (num(&row["taxTotal"])
                - sum("taxCapital")
                - sum("taxGain")
                - sum("assessedIncome")
                - sum("assessedWealth"))
            .abs()
                < 1e-6
        );
        let year_draw = sum("withdrawal") / n * 12.0;
        match row["rate"].as_f64() {
            Some(rate) => {
                with_rate += 1;
                assert!((rate - year_draw / start_total).abs() < 1e-12, "{year}");
            }
            None => assert!(year_draw <= 0.0 || start_total <= 0.0, "{year}"),
        }
        start_total = num(&last["total"]);
    }
    assert!(with_rate > 3);
    // real: the balance at the end of the year in prices of the start month
    for (r, n) in real.iter().zip(nominal) {
        let last = records
            .iter()
            .rev()
            .find(|x| x["month"].as_str().unwrap().starts_with(&r["year"].to_string()))
            .unwrap();
        assert!((num(&r["total"]) - num(&n["total"]) / num(&last["priceIndex"])).abs() < 1e-6);
    }
}

/// #209/AK1: The saved run (JSON) gives the same yearly values as the run in memory, and a run of an older version
/// without the newer fields (pillar 3a contributions, assessment) is still read.
#[test]
fn issue_209_ak1_yearly_values_of_saved_and_older_runs() {
    let out = api::run(&fixed_plan(), &library(), Some(&library()));
    let single = out.single.unwrap();
    let stored = digest::StoredRun {
        summary: single.summary.clone(),
        records: single.records.clone(),
    };
    let saved = serde_json::to_value(&single).unwrap();
    assert_eq!(digest::yearly(&stored), api::yearly(&saved).unwrap());
    let mut old = saved.clone();
    for r in old["records"].as_array_mut().unwrap() {
        let r = r.as_object_mut().unwrap();
        for k in ["pillar3aIn", "taxBasis", "assessedIncome", "assessedWealth"] {
            r.remove(k);
        }
    }
    old["summary"].as_object_mut().unwrap().remove("startTotal");
    let y = api::yearly(&old).unwrap();
    assert_eq!(y.nominal.len(), digest::yearly(&stored).nominal.len());
    assert!(y.nominal.iter().all(|r| r.basis.is_none()));
    // without the start wealth the first year has no withdrawal rate
    assert_eq!(y.nominal[0].rate, None);
    // not a run: a message, no panic
    assert!(api::yearly(&json!({ "records": "x" })).is_err());
    assert!(api::yearly(&json!({})).unwrap().nominal.is_empty());
}

/// #209/AK1: The verdict classifies the success rate like the traffic light at the displayed, rounded value (#72/AK2).
#[test]
fn issue_209_ak1_verdict_limits_at_the_rounded_success_rate() {
    let classes: Vec<Verdict> = [0.95, 0.9496, 0.9494, 0.8, 0.7996, 0.7994, 0.0]
        .into_iter()
        .map(digest::rate_class)
        .collect();
    use Verdict::*;
    assert_eq!(classes, [High, High, Medium, Medium, Medium, Low, Low]);
    for (rate, next) in [
        (0.96, NextStep::Scenarios),
        (0.9, NextStep::Optimize),
        (0.5, NextStep::Optimize),
    ] {
        let d = api::digest(&json!({ "batch": { "summary": { "runs": 10, "successRate": rate } } }))
            .unwrap()
            .unwrap();
        assert_eq!((d.success_rate, d.next, d.runs), (Some(rate), next, 10));
    }
}

/// #209/AK1: Digest of several runs: success rate, earliest depletion, end wealth per percentile nominal and real;
/// for a result before #126 the real value from the detail runs (#128).
#[test]
fn issue_209_ak1_digest_of_several_runs() {
    let res = result(&mc_plan());
    let s = &res["batch"]["summary"];
    let d = api::digest(&res).unwrap().unwrap();
    assert_eq!(d.runs, 300);
    assert_eq!(d.success_rate, Some(num(&s["successRate"])));
    assert_eq!(d.verdict, digest::rate_class(num(&s["successRate"])));
    assert_eq!(
        d.depleted.map(|m| m.iso()),
        s["earliestDepletion"][0].as_str().map(str::to_string)
    );
    let percents: Vec<u64> = s["percentiles"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["percent"].as_u64().unwrap())
        .collect();
    assert_eq!(
        d.end_wealth.iter().map(|e| e.percent as u64).collect::<Vec<_>>(),
        percents
    );
    for (e, p) in d.end_wealth.iter().zip(s["percentiles"].as_array().unwrap()) {
        assert_eq!(e.nominal, Some(num(&p["endTotal"])));
        assert_eq!(e.real, Some(num(&p["endTotalReal"])));
    }
    let mut old = res.clone();
    for p in old["batch"]["summary"]["percentiles"].as_array_mut().unwrap() {
        p.as_object_mut().unwrap().remove("endTotalReal");
    }
    let d = api::digest(&old).unwrap().unwrap();
    let detail = |q: u32| {
        res["batch"]["details"]
            .as_array()
            .unwrap()
            .iter()
            .find(|x| x[0] == json!(q))
            .map(|x| num(&x[1]["summary"]["endTotalReal"]))
    };
    for e in &d.end_wealth {
        assert_eq!(e.real, detail(e.percent), "{}", e.percent);
    }
    assert!(d.end_wealth.iter().any(|e| e.real.is_none()));
}

/// #209/AK1: Digest of a single run: the wealth lasts (next step variations) or is depleted (next step optimization).
#[test]
fn issue_209_ak1_digest_of_a_single_run() {
    let res = result(&fixed_plan());
    let d = api::digest(&res).unwrap().unwrap();
    let s = &res["single"]["summary"];
    assert_eq!(d.runs, 1);
    assert_eq!(d.success_rate, None);
    assert_eq!(d.end_wealth[0].percent, 50);
    assert_eq!(d.end_wealth[0].nominal, Some(num(&s["endTotal"])));
    assert_eq!(d.end_wealth[0].real, Some(num(&s["endTotalReal"])));
    assert_eq!((d.verdict, d.next), (Verdict::Lasts, NextStep::Scenarios));
    let mut poor = fixed_plan();
    poor["flows"]["spending"] = json!(40_000);
    let d = api::digest(&result(&poor)).unwrap().unwrap();
    assert_eq!((d.verdict, d.next), (Verdict::Depleted, NextStep::Optimize));
    assert!(d.depleted.is_some());
    // without a run (error or only variations): no digest
    assert_eq!(api::digest(&json!({ "error": null })).unwrap(), None);
}

/// #209/AK1: Comparison: key figures per result, the percentiles and ages at depletion of all results, the share of
/// runs per age, the differences of the plans against the first.
#[test]
fn issue_209_ak1_comparison_of_several_results() {
    let (a, b) = (fixed_plan(), mc_plan());
    let (ra, rb) = (result(&a), result(&b));
    let c = api::compare(&json!([
        { "result": ra, "plan": a },
        { "result": rb, "plan": b },
        { "result": { "scenarios": {} }, "plan": null }
    ]))
    .unwrap();
    assert_eq!(
        c.items.iter().map(|i| i.kind).collect::<Vec<_>>(),
        [ResultKind::Single, ResultKind::Batch, ResultKind::Scenarios]
    );
    let mut want: Vec<u32> = rb["batch"]["summary"]["percentiles"]
        .as_array()
        .unwrap()
        .iter()
        .map(|p| p["percent"].as_u64().unwrap() as u32)
        .collect();
    want.sort_unstable_by(|x, y| y.cmp(x));
    assert_eq!(c.percentiles, want);
    let ages: Vec<(u32, usize)> = serde_json::from_value(rb["batch"]["stats"]["depletionAges"].clone()).unwrap();
    assert_eq!(c.depletion_ages, ages.iter().map(|(a, _)| *a).collect::<Vec<_>>());
    let shares = &c.items[1].depletion_shares;
    assert_eq!(shares.len(), ages.len());
    let failed = 1.0 - num(&rb["batch"]["summary"]["successRate"]);
    assert!(
        (shares.iter().sum::<f64>() - failed).abs() < 1e-9,
        "{shares:?} {failed}"
    );
    assert!(c.items[0].depletion_shares.is_empty());
    // depletion: month or «never» (null) per result with a run, missing without
    assert_eq!(c.items[0].depletion, Some(None));
    assert_eq!(
        c.items[1].depletion.flatten().map(|m| m.iso()),
        rb["batch"]["summary"]["earliestDepletion"][0]
            .as_str()
            .map(str::to_string)
    );
    assert_eq!(c.items[2].depletion, None);
    assert!(c.items[2].digest.is_none());
    assert_eq!(c.items[0].digest, api::digest(&ra).unwrap());
    // plan differences against the first
    assert!(c.items[0].plan_changes.is_empty());
    assert!(c.items[1]
        .plan_changes
        .iter()
        .any(|ch| ch.path == "returns.source" && ch.new == json!("parametric")));
    assert!(c.items[2].plan_changes.is_empty());
    let v = serde_json::to_value(&c).unwrap();
    assert!(v["items"][2].get("depletion").is_none());
    assert!(v["items"][0]["depletion"].is_null());
    assert!(api::compare(&json!({})).is_err());
}

/// #209/AK4: The answer of `run` stays as it was (the golden files compare it in full): yearly values and digest are
/// separate functions, not fields of the result.
#[test]
fn issue_209_ak4_run_answer_without_new_fields() {
    let res = result(&mc_plan());
    let mut keys: Vec<&String> = res.as_object().unwrap().keys().collect();
    keys.sort();
    assert_eq!(keys, ["batch", "elapsedMs", "error", "issues", "scenarios", "single"]);
    let mut keys: Vec<&String> = res["batch"].as_object().unwrap().keys().collect();
    keys.sort();
    assert_eq!(keys, ["bands", "details", "stats", "summary"]);
    for (_, run) in res["batch"]["details"]
        .as_array()
        .unwrap()
        .iter()
        .map(|d| (d[0].clone(), &d[1]))
    {
        assert!(run.get("yearly").is_none() && run.get("digest").is_none());
    }
}
