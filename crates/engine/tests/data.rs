//! Data and statistics: import against the original columns, CSV robustness, distributions of the
//! Monte Carlo paths, the same paths in all scenarios.

use std::path::{Path, PathBuf};

use calamine::{open_workbook_auto, Data, Reader};
use engine::market::{DataSet, Freq};
use engine::model::{BootstrapParams, ParametricParams, Plan};
use engine::montecarlo::{Bootstrap, Parametric};
use engine::resolve::Curve;
use engine::time::YearMonth;
use engine::{api, import, market};
use serde_json::json;

/// #94: Shiller's original spreadsheet only exists locally (not in the repository). If it is missing, `None` is returned, and the test
/// is skipped with a message.
fn shiller_file() -> Option<PathBuf> {
    let p = Path::new(env!("CARGO_MANIFEST_DIR")).join("data/quellen/shiller-index.ods");
    if p.exists() {
        Some(p)
    } else {
        eprintln!("übersprungen: {} fehlt (#94)", p.display());
        None
    }
}

fn num(c: Option<&Data>) -> Option<f64> {
    match c? {
        Data::Float(v) => Some(*v),
        Data::Int(v) => Some(*v as f64),
        _ => None,
    }
}

/// Independent check of the Shiller import with the raw columns: stocks = (P + D/12) / P₋₁ − 1,
/// bonds = Shiller's «Monthly Total Bond Returns» of the previous month (column R), inflation = CPI / CPI₋₁₂ − 1.
#[test]
fn shiller_import_matches_raw_columns() {
    let Some(shiller) = shiller_file() else {
        return;
    };
    let mut wb = open_workbook_auto(&shiller).unwrap();
    let range = wb.worksheet_range("Data").unwrap();
    let mut raw = std::collections::BTreeMap::new();
    for r in range.rows() {
        let Some(x) = num(r.first()) else { continue };
        let y = x.floor();
        let m = ((x - y) * 100.0).round() as u32;
        if (1..=12).contains(&m) {
            raw.insert(
                YearMonth::new(y as i32, m),
                (num(r.get(1)), num(r.get(2)), num(r.get(4)), num(r.get(17))),
            );
        }
    }
    let ds = import::shiller_dataset(&import::read_shiller(&shiller).unwrap()).unwrap();
    let (mut n_s, mut n_b) = (0, 0);
    for i in 0..ds.len() {
        let m = ds.first.plus(i as i32);
        let (p, d, cpi, _) = raw[&m];
        let (p0, _, _, bond0) = raw[&m.plus(-1)];
        let cpi12 = raw[&m.plus(-12)].2.unwrap();
        assert!(
            (ds.inflation[i] - (cpi.unwrap() / cpi12 - 1.0)).abs() < 1e-9,
            "Inflation {m}"
        );
        if let (Some(p), Some(d), Some(p0)) = (p, d, p0) {
            assert!((ds.stocks[i] - ((p + d / 12.0) / p0 - 1.0)).abs() < 1e-9, "Aktien {m}");
            n_s += 1;
        }
        if let Some(b) = bond0 {
            assert!((ds.bonds[i] - (b - 1.0)).abs() < 1e-9, "Obligationen {m}");
            n_b += 1;
        }
    }
    assert!(n_s > 1800 && n_b > 1800, "{n_s} {n_b}");
}

#[test]
fn csv_parser_is_robust() {
    let ok = "\u{feff}# Titel: Test\n# Währung: chf\n\nDate,Stocks,Bonds,Inflation,Saron\n2000-01,0.01,0.0,0.02,\n2000-02,-0.02,0.001,0.02,0.005\n";
    let ds = DataSet::parse_csv("t", ok).unwrap();
    assert_eq!(ds.currency(), Some("CHF"));
    assert_eq!((ds.freq, ds.len()), (Freq::Monthly, 2));
    assert!(ds.saron.as_ref().unwrap()[0].is_nan());
    for (bad, why) in [
        ("", "empty"),
        ("Date,Stocks,Bonds\n2000-01,0,0\n", "Inflation fehlt"),
        ("Date,Stocks,Bonds,Inflation\n2000-01,0,0,0\n2000-03,0,0,0\n", "gaps"),
        ("Date,Stocks,Bonds,Inflation\n2000-01,0,x,0\n", "keine Zahl"),
        ("Date,Stocks,Bonds,Inflation\n2000-13,0,0,0\n", "Monat 13"),
        ("Years,Stocks,Bonds,Inflation\n2000,0,0,0\n2000-02,0,0,0\n", "gemischt"),
        ("Date,Stocks,Bonds,Inflation\n", "keine Daten"),
    ] {
        assert!(DataSet::parse_csv("t", bad).is_err(), "{why}");
    }
}

/// #60/AK1, AK2: Year numbers outside 1000 to 9999 give an error message with the line number instead of a
/// crash (finding of the fuzzer `parse_csv`: overflow in `YearMonth::new`).
#[test]
fn issue_60_extreme_years_are_errors_not_crashes() {
    let err = |year: &str| {
        let csv = format!("Years,Stocks,Bonds,Inflation\n{year},0,0,0\n");
        DataSet::parse_csv("t", &csv).unwrap_err().to_string()
    };
    for year in ["99999999999", "2147483647", "999", "10000", "0"] {
        let e = err(year);
        assert!(
            e.contains("line 2") && e.contains(&format!("“{year}” is not a year (1000 to 9999)")),
            "{year}: {e}"
        );
    }
    // With «-» the value counts as a month date and is rejected there.
    for year in ["-2147483648", "-1"] {
        let e = err(year);
        assert!(
            e.contains("line 2") && e.contains("is not a valid month"),
            "{year}: {e}"
        );
    }
    // Limits valid, also as a sequence
    let ds = DataSet::parse_csv("t", "Years,Stocks,Bonds,Inflation\n1000,0,0,0\n1001,0,0,0\n").unwrap();
    assert_eq!(
        (ds.freq, ds.len(), ds.first),
        (Freq::Yearly, 2, YearMonth::new(1000, 1))
    );
    assert_eq!(
        DataSet::parse_csv("t", "Years,Stocks,Bonds,Inflation\n9999,0,0,0\n")
            .unwrap()
            .first,
        YearMonth::new(9999, 1)
    );
    // AK2: the month index does not overflow, also directly with extreme years
    assert_eq!(YearMonth::new(i32::MAX, 12), YearMonth(i32::MAX));
    assert_eq!(YearMonth::new(i32::MIN, 1), YearMonth(i32::MIN));
    assert_eq!(YearMonth::new(2000, 1).plus(-1), YearMonth::new(1999, 12));
}

fn fixed() -> market::FixedCurves {
    market::FixedCurves {
        stocks: Curve::constant(0.0),
        bonds: Curve::constant(0.0),
        inflation: Curve::constant(0.0),
        saron: Curve::constant(0.0),
    }
}

/// Over many paths the bootstrap matches the distribution of the series: same mean,
/// same spread of the monthly returns, mean block length as set.
#[test]
fn bootstrap_reproduces_marginal_distribution() {
    let ds = DataSet::load_csv(&Path::new(env!("CARGO_MANIFEST_DIR")).join("data/Shiller CHF.csv")).unwrap();
    let f = fixed();
    let b = Bootstrap {
        data: &ds,
        fixed: &f,
        start: YearMonth::new(2030, 1),
        months: 360,
        params: BootstrapParams {
            paths: 2_000,
            block: 24.0,
            seed: 5,
        },
    };
    let mean = |v: &[f64]| v.iter().sum::<f64>() / v.len() as f64;
    let sd = |v: &[f64]| {
        let m = mean(v);
        (v.iter().map(|x| (x - m).powi(2)).sum::<f64>() / v.len() as f64).sqrt()
    };
    let mut all = vec![];
    let mut blocks = 0usize;
    let positions: std::collections::HashMap<u64, usize> =
        ds.stocks.iter().enumerate().map(|(i, x)| (x.to_bits(), i)).collect();
    for i in 0..b.len() {
        let p = b.path(i);
        for k in 0..p.stocks.len() {
            if k == 0 || positions[&p.stocks[k].to_bits()] != (positions[&p.stocks[k - 1].to_bits()] + 1) % ds.len() {
                blocks += 1;
            }
        }
        all.extend(p.stocks);
    }
    assert!(
        (mean(&all) - mean(&ds.stocks)).abs() < 0.0005,
        "{} {}",
        mean(&all),
        mean(&ds.stocks)
    );
    assert!((sd(&all) / sd(&ds.stocks) - 1.0).abs() < 0.02);
    let avg_block = all.len() as f64 / blocks as f64;
    assert!((avg_block - 24.0).abs() < 1.5, "mittlere Blocklänge {avg_block}");
}

/// Price jumps: on average `jumpLambda` per year, size ≈ e^jumpMu.
#[test]
fn parametric_jumps_have_expected_frequency() {
    let f = fixed();
    let q = ParametricParams {
        paths: 4_000,
        mu: 0.0,
        sigma: 0.0,
        df: 0.0,
        jump_lambda: 0.5,
        jump_mu: -0.3,
        jump_sigma: 0.0,
        seed: 3,
        auto: false,
        ..Default::default()
    };
    let g = Parametric {
        fixed: &f,
        start: YearMonth::new(2030, 1),
        months: 12,
        params: q,
        correlated: None,
    };
    let jumps: usize = (0..g.len())
        .map(|i| g.path(i).stocks.iter().filter(|r| r.abs() > 1e-12).count())
        .sum();
    let per_year = jumps as f64 / g.len() as f64;
    assert!((per_year - 0.5).abs() < 0.05, "{per_year}");
    let p = (0..g.len())
        .map(|i| g.path(i))
        .find(|p| p.stocks.iter().any(|r| r.abs() > 1e-12))
        .unwrap();
    let r = p.stocks.iter().find(|r| r.abs() > 1e-12).unwrap();
    assert!((r - ((-0.3f64).exp() - 1.0)).abs() < 1e-9);
}

/// A scenario without a change (a single combination with the initial value) gives the same as
/// the plan itself: same paths, same metrics.
#[test]
fn scenario_with_base_value_equals_base_run() {
    let lib = Path::new(env!("CARGO_MANIFEST_DIR")).join("data");
    let mut plan = api::example_plan();
    plan["returns"]["source"] = json!("bootstrap");
    plan["returns"]["bootstrap"] = json!({ "paths": 500, "block": 12, "seed": 9 });
    let p: Plan = serde_json::from_value(plan.clone()).unwrap();
    let share = match &p.wealth.stock_share {
        engine::model::Series::Number(x) | engine::model::Series::Def(engine::model::SeriesDef::Const { value: x }) => {
            *x
        }
        _ => panic!("Beispiel: Aktienanteil konstant erwartet"),
    };
    plan["scenarios"]["sweeps"] = json!([{ "param": "wealth.stockShare", "from": share, "to": share, "step": 0.1 }]);
    let out = api::run(&plan, Path::new("/"), Some(&lib));
    assert!(out.error.is_none(), "{:?}", out.error);
    let base = out.batch.unwrap().summary;
    let sc = &out.scenarios.unwrap().results[0];
    assert_eq!(sc.runs, base.runs);
    assert_eq!(sc.success_rate, base.success_rate);
    let p50 = base.percentiles.iter().find(|x| x.percent == 50).unwrap().end_total;
    assert_eq!(sc.p50, p50);
}

/// Bands are ordered per month (5th ≤ 25th ≤ 50th ≤ 75th ≤ 95th percentile).
#[test]
fn bands_are_ordered() {
    let lib = Path::new(env!("CARGO_MANIFEST_DIR")).join("data");
    let mut plan = api::example_plan();
    plan["returns"]["source"] = json!("rolling");
    plan["scenarios"]["sweeps"] = json!([]);
    let out = api::run(&plan, Path::new("/"), Some(&lib));
    let b = out.batch.unwrap().bands;
    for k in 0..b.months.len() {
        assert!(
            b.p5[k] <= b.p25[k] && b.p25[k] <= b.p50[k] && b.p50[k] <= b.p75[k] && b.p75[k] <= b.p95[k],
            "{}",
            b.months[k]
        );
    }
}
