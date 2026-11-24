//! #218: characteristic curves: the stock share as support points over a state variable, evaluated when the portfolio
//! is rebalanced.
//!
//! Made-up plan: one person born 1960-01 (retired), start 2030-01, end 2049-12, spending 4000 per month, liquid wealth
//! 1.2 million, no returns, no inflation, no pensions and no taxes; the withdrawal rate at the start is thus 4 %.

use engine::api;
use engine::model::Plan;
use engine::sim::{MonthRecord, SimResult, Simulator};
use engine::time::YearMonth;
use serde_json::{json, Value};

fn base() -> Value {
    json!({
        "project": { "name": "Kennlinien" },
        "household": { "persons": [ { "id": "A", "name": "A", "birth": "1960-01" } ] },
        "milestones": [
            { "id": "start", "kind": "date", "date": "2030-01", "fixed": true },
            { "id": "retire", "kind": "age", "person": "A", "years": 65 },
            { "id": "ahv", "kind": "age", "person": "A", "years": 65 },
            { "id": "end", "kind": "date", "date": "2049-12", "fixed": true }
        ],
        "pension": { "pk": { "capital": 0, "at": "retire", "lumpsum": 0, "conversion": 0 }, "ahv": { "monthly": 0, "at": "ahv" } },
        "flows": { "income": 0, "spending": 4000 },
        "wealth": { "stocks": 600_000, "bonds": 600_000, "cash": 0, "stockShare": 0.5, "reserveMonths": 0 },
        "returns": { "source": "fixed", "fixed": { "stocks": 0, "bonds": 0, "inflation": 0, "saron": 0 } },
        "taxes": { "income": [], "wealth": [], "capital": [] }
    })
}

fn with_curve(input: &str, points: Value, eval: &str, f: impl FnOnce(&mut Value)) -> Value {
    let mut v = base();
    v["wealth"]["stockShare"] = json!({ "mode": "curve", "input": input, "points": points, "eval": eval });
    f(&mut v);
    v
}

fn run(v: &Value) -> SimResult {
    let p: Plan = serde_json::from_value(v.clone()).expect("plan readable");
    let s = Simulator::new(&p).expect("plan valid");
    let path = engine::market::fixed_path(&s.fixed, s.start, s.months());
    s.run(&path, true)
}

fn month<'a>(r: &'a SimResult, ym: &str) -> &'a MonthRecord {
    let m: YearMonth = ym.parse().unwrap();
    r.records.iter().find(|x| x.month == m).expect("month in the plan")
}

/// Stock share of the invested wealth at the end of month `ym`.
fn share(r: &SimResult, ym: &str) -> f64 {
    let m = month(r, ym);
    m.stocks / (m.stocks + m.bonds)
}

fn errors(v: &Value) -> Vec<String> {
    api::validate(v)
        .into_iter()
        .filter(|i| i.severity == engine::validate::Severity::Error)
        .map(|i| i.message.to_string())
        .collect()
}

fn close(a: f64, b: f64) -> bool {
    (a - b).abs() < 1e-9
}

/// Value of the line through (x1, a) and (x2, b) at x, limited to the two ends.
fn line(x: f64, (x1, a): (f64, f64), (x2, b): (f64, f64)) -> f64 {
    let t = ((x - x1) / (x2 - x1)).clamp(0.0, 1.0);
    a + (b - a) * t
}

// #218/AK3: support points [[0.02, 0.8], [0.06, 0.4]] over the withdrawal rate; a rate of 4 % gives 0.6.
#[test]
fn issue_218_ak3_withdrawal_rate_4_percent_gives_0_6() {
    let v = with_curve("withdrawalRate", json!([[0.02, 0.8], [0.06, 0.4]]), "yearly", |_| {});
    assert!(errors(&v).is_empty(), "{:?}", errors(&v));
    let r = run(&v);
    // At the start: 12 × 4000 / 1.2 million = 4 %.
    assert!(close(share(&r, "2030-01"), 0.6), "{}", share(&r, "2030-01"));
    // A year later, after the withdrawal of January: 48'000 / (1.2 million − 13 × 4000).
    let rate = 48_000.0 / (1_200_000.0 - 52_000.0);
    let expected = line(rate, (0.02, 0.8), (0.06, 0.4));
    assert!(
        close(share(&r, "2031-01"), expected),
        "{} vs {expected}",
        share(&r, "2031-01")
    );
}

// #218/AK2: «yearly» keeps the value of the first rebalancing of a year, «monthly» evaluates every month.
#[test]
fn issue_218_ak2_yearly_and_monthly_evaluation() {
    let points = json!([[0.0, 1.0], [0.1, 0.0]]);
    let yearly = run(&with_curve("withdrawalRate", points.clone(), "yearly", |_| {}));
    let monthly = run(&with_curve("withdrawalRate", points.clone(), "monthly", |_| {}));
    let at = |liquid: f64| line(48_000.0 / liquid, (0.0, 1.0), (0.1, 0.0));
    for (ym, months) in [("2030-06", 5.0), ("2030-12", 11.0)] {
        assert!(close(share(&yearly, ym), at(1_200_000.0)), "{ym}");
        // Liquid wealth before the rebalancing of the month, withdrawal of the last 12 months extrapolated.
        let liquid = 1_200_000.0 - (months + 1.0) * 4000.0;
        assert!(close(share(&monthly, ym), at(liquid)), "{ym}: {}", share(&monthly, ym));
    }
    assert!(close(share(&yearly, "2031-03"), share(&yearly, "2031-01")));
    assert!(share(&monthly, "2031-03") < share(&monthly, "2031-01"));
    // Without «eval» the curve is evaluated once per year.
    let mut v = with_curve("withdrawalRate", points, "yearly", |_| {});
    v["wealth"]["stockShare"].as_object_mut().unwrap().remove("eval");
    let default = run(&v);
    assert!(close(share(&default, "2030-12"), share(&yearly, "2030-12")));
    // A file round trip keeps the curve.
    let p: Plan = serde_json::from_value(v.clone()).unwrap();
    let back = serde_json::to_value(&p).unwrap();
    assert_eq!(back["wealth"]["stockShare"]["input"], json!("withdrawalRate"));
    assert_eq!(back["wealth"]["stockShare"]["points"], json!([[0.0, 1.0], [0.1, 0.0]]));
    assert_eq!(
        api::upgrade_plan(&v).plan["wealth"]["stockShare"],
        v["wealth"]["stockShare"]
    );
}

// #218/AK1: age, liquid wealth in prices of the start and the funded ratio.
#[test]
fn issue_218_ak1_age_liquid_real_and_funded_ratio() {
    // Age 70 at the start.
    let r = run(&with_curve("age", json!([[60, 0.9], [80, 0.3]]), "yearly", |_| {}));
    assert!(close(share(&r, "2030-01"), 0.6), "{}", share(&r, "2030-01"));
    assert!(close(share(&r, "2035-01"), 0.45), "{}", share(&r, "2035-01"));

    // Liquid wealth in prices of the start: with 10 % inflation divided by the price index.
    let points = json!([[1_000_000, 0.4], [1_400_000, 0.8]]);
    let r = run(&with_curve("liquidReal", points.clone(), "yearly", |_| {}));
    assert!(close(share(&r, "2030-01"), 0.6));
    let r = run(&with_curve("liquidReal", points, "yearly", |v| {
        v["returns"]["fixed"]["inflation"] = json!(0.1)
    }));
    let m = month(&r, "2031-01");
    let expected = line(m.liquid / 1.1, (1_000_000.0, 0.4), (1_400_000.0, 0.8));
    assert!(
        (share(&r, "2031-01") - expected).abs() < 1e-6,
        "{} vs {expected}",
        share(&r, "2031-01")
    );

    // Funded ratio: 1.2 million / (240 months × 4000) = 1.25.
    let points = json!([[1.0, 0.4], [2.0, 0.8]]);
    let r = run(&with_curve("fundedRatio", points.clone(), "yearly", |_| {}));
    assert!(close(share(&r, "2030-01"), 0.5), "{}", share(&r, "2030-01"));
    // A year later, after the withdrawal of January: (1.2 million − 13 × 4000) / (228 × 4000).
    let expected = line(1_148_000.0 / 912_000.0, (1.0, 0.4), (2.0, 0.8));
    assert!(close(share(&r, "2031-01"), expected), "{}", share(&r, "2031-01"));
    // An AHV pension of 1000 reduces the gap to 240 × 3000.
    let r = run(&with_curve("fundedRatio", points.clone(), "yearly", |v| {
        v["pension"]["ahv"]["monthly"] = json!(1000)
    }));
    let expected = line(1_200_000.0 / 720_000.0, (1.0, 0.4), (2.0, 0.8));
    assert!(
        close(share(&r, "2030-01"), expected),
        "{} vs {expected}",
        share(&r, "2030-01")
    );
    // A pension fund pension of 1000 from 2035-01 (not yet retired): 240 × 4000 − 180 × 1000; first evaluation in
    // the first month after its withdrawal.
    let r = run(&with_curve("fundedRatio", points.clone(), "yearly", |v| {
        v["pension"]["pk"] =
            json!({ "capital": 240_000, "at": "2035-01", "lumpsum": 0, "conversion": 0.05, "interest": 0 })
    }));
    let expected = line(1_196_000.0 / 780_000.0, (1.0, 0.4), (2.0, 0.8));
    assert!(
        close(share(&r, "2030-01"), expected),
        "{} vs {expected}",
        share(&r, "2030-01")
    );
    // Earned income and pensions that cover everything: infinitely funded, the last value.
    let r = run(&with_curve("fundedRatio", points, "yearly", |v| {
        v["pension"]["ahv"]["monthly"] = json!(5000)
    }));
    assert!(close(share(&r, "2030-01"), 0.8));
}

// #218/AK1: the withdrawal counts spending and taxes against earned income and savings; the funded ratio subtracts
// earned income and real pensions from the gap.
#[test]
fn issue_218_ak1_withdrawal_and_gap_count_income_taxes_and_inflation() {
    let points = json!([[0.0, 1.0], [0.1, 0.0]]);
    let r = run(&with_curve("withdrawalRate", points, "monthly", |v| {
        v["flows"]["income"] = json!(1000);
        v["flows"]["savings"] = json!(500);
        v["taxes"] = json!({ "multiplier": 1, "income": [[0, 0.1]], "wealth": [[0, 0.0]], "capital": [[0, 0.0]] });
    }));
    // 2031-06: the last 12 months contain the income tax of 2030, paid in May.
    let last: Vec<_> = r
        .records
        .iter()
        .filter(|x| x.month > YearMonth::new(2030, 6) && x.month <= YearMonth::new(2031, 6))
        .collect();
    assert_eq!(last.len(), 12);
    assert!(last.iter().any(|x| x.taxes > 0.0));
    let out: f64 = last.iter().map(|x| x.spending + x.taxes - x.income - 500.0).sum();
    let expected = line(out / month(&r, "2031-06").liquid, (0.0, 1.0), (0.1, 0.0));
    assert!(
        close(share(&r, "2031-06"), expected),
        "{} vs {expected}",
        share(&r, "2031-06")
    );

    // Earned income 1000: 1.2 million / (240 × 3000).
    let points = json!([[1.0, 0.4], [2.0, 0.8]]);
    let r = run(&with_curve("fundedRatio", points.clone(), "yearly", |v| {
        v["flows"]["income"] = json!(1000)
    }));
    let expected = line(1_200_000.0 / 720_000.0, (1.0, 0.4), (2.0, 0.8));
    assert!(
        close(share(&r, "2030-01"), expected),
        "{} vs {expected}",
        share(&r, "2030-01")
    );
    // A pension fund pension of 1000 (not indexed) with 10 % inflation: wealth and pension in prices of the start.
    let r = run(&with_curve("fundedRatio", points, "yearly", |v| {
        v["pension"]["pk"] =
            json!({ "capital": 240_000, "at": "retire", "lumpsum": 0, "conversion": 0.05, "interest": 0 });
        v["returns"]["fixed"]["inflation"] = json!(0.1);
    }));
    let gap = 228.0 * 4000.0 - 228.0 * 1000.0 / 1.1;
    let expected = line(month(&r, "2031-01").liquid / 1.1 / gap, (1.0, 0.4), (2.0, 0.8));
    assert!(
        (share(&r, "2031-01") - expected).abs() < 1e-6,
        "{} vs {expected}",
        share(&r, "2031-01")
    );
}

// #218/AK1: stock return of the last 12 months and drawdown from the highest level so far.
#[test]
fn issue_218_ak1_stock_return_and_drawdown() {
    // Flat stocks in 2030, from 2031 −20 % per year; evaluated in 2032-01 (13 months of losses).
    let falling = |v: &mut Value| {
        v["returns"]["fixed"]["stocks"] = json!({ "mode": "timed", "rows": [ { "from": "start", "value": 0 }, { "from": "2031-01", "value": -0.2 } ] })
    };
    let r = run(&with_curve(
        "stockReturn12m",
        json!([[-0.4, 1.0], [0.0, 0.5]]),
        "yearly",
        falling,
    ));
    assert!(close(share(&r, "2030-01"), 0.5));
    assert!(
        close(share(&r, "2032-01"), line(-0.2, (-0.4, 1.0), (0.0, 0.5))),
        "{}",
        share(&r, "2032-01")
    );

    let r = run(&with_curve(
        "drawdown",
        json!([[0.0, 0.5], [0.5, 1.0]]),
        "monthly",
        falling,
    ));
    assert!(close(share(&r, "2030-06"), 0.5));
    let dd = 1.0 - 0.8f64.powf(13.0 / 12.0);
    // Falling from the start: the lead-in of the path falls too, so the stock index is already below its peak at the
    // start.
    let early = run(&with_curve(
        "drawdown",
        json!([[0.0, 0.5], [0.5, 1.0]]),
        "yearly",
        |v| v["returns"]["fixed"]["stocks"] = json!(-0.2),
    ));
    assert!(share(&early, "2030-01") > 0.6, "{}", share(&early, "2030-01"));
    assert!(
        close(share(&r, "2032-01"), line(dd, (0.0, 0.5), (0.5, 1.0))),
        "{}",
        share(&r, "2032-01")
    );
}

// #218/AK1, AK2: the schema names the curve with its six state variables, each with a definition.
#[test]
fn issue_218_ak1_schema_defines_the_state_variables() {
    let schema: Value = serde_json::from_str(&engine::schema_json()).unwrap();
    let defs = &schema["definitions"];
    let text = defs["StateInput"].to_string();
    for input in [
        "age",
        "liquidReal",
        "withdrawalRate",
        "fundedRatio",
        "stockReturn12m",
        "drawdown",
    ] {
        assert!(text.contains(&format!("\"{input}\"")), "{input}: {text}");
    }
    let described = defs["StateInput"]["oneOf"].as_array().unwrap();
    assert_eq!(described.len(), 6, "{text}");
    assert!(described.iter().all(|d| d["description"].is_string()), "{text}");
    let curve = defs["StateCurve"].to_string();
    assert!(
        curve.contains("points") && curve.contains("eval") && curve.contains("rebalanced"),
        "{curve}"
    );
    assert!(defs["CurveMode"].to_string().contains("\"curve\""));
    assert!(defs["CurveEval"].to_string().contains("monthly"));
    assert!(defs["Wealth"]["properties"]["stockShare"]
        .to_string()
        .contains("StateSeries"));
}

// #218/AK2: the check: values between 0 and 1, at least one support point, unknown state variables; «@» refuses a curve.
#[test]
fn issue_218_ak2_check_of_the_curve() {
    let v = with_curve("withdrawalRate", json!([[0.02, 1.2], [0.06, 0.4]]), "yearly", |_| {});
    assert!(
        errors(&v).iter().any(|e| e.contains("between 0 and 1")),
        "{:?}",
        errors(&v)
    );
    let v = with_curve("withdrawalRate", json!([]), "yearly", |_| {});
    assert!(
        errors(&v).iter().any(|e| e.contains("support point")),
        "{:?}",
        errors(&v)
    );
    let v = with_curve("inflation", json!([[0.0, 0.5]]), "yearly", |_| {});
    assert!(serde_json::from_value::<Plan>(v).is_err());
    let v = with_curve("age", json!([[70, 0.5]]), "weekly", |_| {});
    assert!(serde_json::from_value::<Plan>(v).is_err());
    let v = with_curve("age", json!([[70, 0.5]]), "monthly", |_| {});
    assert!(serde_json::from_value::<Plan>(v.clone())
        .unwrap()
        .wealth
        .stock_share
        .as_series()
        .is_none());
    let e = api::set_fields(&v, &["wealth.stockShare@retire=0.5".to_string()]).unwrap_err();
    assert!(e.iter().any(|m| m.code() == "field_series_form_fixed"), "{e:?}");
    let set = api::set_fields(&v, &["wealth.stockShare=0.5".to_string()]).unwrap();
    assert_eq!(set["wealth"]["stockShare"], json!(0.5));
}

// #218/AK4: a curve with one support point computes like the constant value (the golden files check that plans without
// a curve compute unchanged).
#[test]
fn issue_218_ak4_plans_without_curve_are_unchanged() {
    let constant = run(&base());
    let same = run(&with_curve("age", json!([[0, 0.5]]), "yearly", |_| {}));
    for (a, b) in constant.records.iter().zip(&same.records) {
        assert!(close(a.stocks, b.stocks) && close(a.bonds, b.bonds), "{:?}", a.month);
    }
}
