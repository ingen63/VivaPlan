//! #214: Measurement layer. The fixed metrics of the optimization (`optimize::evaluate`) and of `batch::summarize`
//! come out of the building blocks exactly as before (bit for bit, compared with the previous computation from the
//! monthly records); free metrics such as consumption in the first 15 years of retirement.

use std::path::{Path, PathBuf};

use engine::batch::{self, Paths};
use engine::measure::{Aggregate, Condition, Measurement, Metric, MetricValue, Op, Quantity, Reduce, Window};
use engine::model::Plan;
use engine::optimize::{self, Objective, END_PERCENTS};
use engine::sim::{SimResult, Simulator};
use engine::time::YearMonth;
use serde_json::{json, Value};

use crate::plan_builder::{parametric, PlanBuilder};

fn library() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("data")
}

fn plan_of(v: &Value) -> Plan {
    serde_json::from_value(v.clone()).expect("plan")
}

/// Runs `f` with the plan's simulator and paths.
fn with_paths<T>(plan: &Plan, f: impl FnOnce(&Simulator, &Paths) -> T) -> T {
    let sim = Simulator::new(plan).expect("simulator");
    let data = batch::load_series(plan, &library(), Some(&library())).expect("series");
    let paths = batch::path_source(plan, &sim, data.as_ref()).expect("paths");
    f(&sim, &paths)
}

/// The example plan with parametric paths and the given strategy and floor.
fn example(paths: usize, strategy: &str, floor: f64) -> Value {
    let mut v = PlanBuilder::example().parametric(parametric(paths, 7)).build();
    v["withdrawal"]["strategy"] = json!(strategy);
    v["withdrawal"]["floor"] = json!(floor);
    v["scenarios"]["sweeps"] = json!([]);
    v
}

// ---------------------------------------------------------------------------------------------
// The previous computation of `optimize::evaluate` from the monthly records (until #214), unchanged as reference.
// ---------------------------------------------------------------------------------------------

struct OldStats {
    ruined: bool,
    annual: Vec<f64>,
    vol: f64,
    draw_rate: f64,
    emergency: bool,
    planned: Vec<f64>,
    end_total: f64,
    floor_share: f64,
}

fn old_path_stats(sim: &Simulator, res: &SimResult, retire: YearMonth, o: &Objective) -> OldStats {
    let ruined = res.summary.depleted.is_some_and(|m| match o.ruin_age {
        Some(age) => sim.age_at(m) <= age,
        None => true,
    });
    let depleted = res.summary.depleted;
    let mut annual = Vec::new();
    let mut planned = Vec::new();
    let (mut sum, mut plan_sum, mut n) = (0.0, 0.0, 0);
    let (mut floor_months, mut months) = (0usize, 0usize);
    for r in res.records.iter().filter(|r| r.month >= retire) {
        months += 1;
        floor_months += usize::from(r.at_floor);
        let spent = if depleted.is_some_and(|d| r.month >= d) {
            r.spending.min(r.income).max(0.0)
        } else {
            r.spending
        };
        sum += spent / r.price_index;
        plan_sum += (r.planned_spending + r.housing) / r.price_index;
        n += 1;
        if n == 12 {
            annual.push(sum);
            planned.push(plan_sum);
            sum = 0.0;
            plan_sum = 0.0;
            n = 0;
        }
    }
    let changes: Vec<f64> = annual
        .windows(2)
        .filter(|w| w[0] > 1e-9)
        .map(|w| ((w[1] - w[0]) / w[0]).abs())
        .collect();
    let vol = if changes.is_empty() {
        0.0
    } else {
        changes.iter().sum::<f64>() / changes.len() as f64
    };
    let first: Vec<_> = res.records.iter().filter(|r| r.month >= retire).take(12).collect();
    let draw_rate = first.first().map_or(0.0, |r| {
        let start = r.liquid + r.spending - r.income + r.taxes;
        let draw: f64 = first.iter().map(|r| r.spending - r.income).sum();
        if start > 0.0 {
            draw.max(0.0) / start
        } else {
            0.0
        }
    });
    OldStats {
        ruined,
        annual,
        vol,
        draw_rate,
        emergency: res.summary.emergency_sales > 0,
        planned,
        end_total: res.summary.end_total,
        floor_share: if months > 0 {
            floor_months as f64 / months as f64
        } else {
            0.0
        },
    }
}

fn old_quantile(sorted: &[f64], q: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    sorted[((sorted.len() - 1) as f64 * q).round() as usize]
}

/// The previous `evaluate` as a list of the values in the order of `Metrics`.
fn old_evaluate(plan: &Plan, o: &Objective) -> Vec<f64> {
    with_paths(plan, |sim, paths| {
        let retire = sim.retire_at().expect("retirement");
        let stats: Vec<OldStats> = (0..paths.len())
            .map(|i| old_path_stats(sim, &sim.run(&paths.get(i), true), retire, o))
            .collect();
        let n = stats.len().max(1);
        let sorted = |mut v: Vec<f64>| {
            v.sort_by(f64::total_cmp);
            v
        };
        let all = sorted(stats.iter().flat_map(|s| s.annual.iter().copied()).collect());
        let end = sorted(stats.iter().map(|s| s.end_total).collect());
        let mut out = vec![
            stats.len() as f64,
            stats.iter().filter(|s| s.ruined).count() as f64 / n as f64,
            old_quantile(&all, 0.5),
            old_quantile(&all, 0.1),
            stats.iter().map(|s| s.vol).sum::<f64>() / n as f64,
            old_quantile(&sorted(stats.iter().map(|s| s.draw_rate).collect()), 0.5),
            stats.iter().filter(|s| s.emergency).count() as f64 / n as f64,
            old_quantile(
                &sorted(stats.iter().flat_map(|s| s.planned.iter().copied()).collect()),
                0.5,
            ),
        ];
        out.extend(END_PERCENTS.map(|p| old_quantile(&end, f64::from(p) / 100.0)));
        out.push(if sim.has_floor() {
            stats.iter().map(|s| s.floor_share).sum::<f64>() / n as f64
        } else {
            -1.0
        });
        out
    })
}

fn new_evaluate(plan: &Plan, o: &Objective) -> Vec<f64> {
    let m = with_paths(plan, |_, paths| optimize::evaluate(plan, paths, o)).expect("evaluate");
    let mut out = vec![
        m.runs as f64,
        m.ruin,
        m.median,
        m.p10,
        m.volatility,
        m.draw_rate,
        m.emergency_share,
        m.planned,
    ];
    out.extend(m.end_total);
    out.push(m.floor_share.unwrap_or(-1.0));
    out
}

fn bits(v: &[f64]) -> Vec<u64> {
    v.iter().map(|x| x.to_bits()).collect()
}

/// #214/AK3: `evaluate` from the building blocks gives exactly the values of the previous computation from the monthly
/// records: several strategies, with and without floor, ruin age and properties.
#[test]
fn issue_214_ak3_evaluate_from_building_blocks_is_exact() {
    let mut cases: Vec<(String, Value)> = [
        ("standard", 0.0),
        ("guardrails", 0.8),
        ("fixed_rate", 0.9),
        ("vpw", 0.0),
        ("three_buckets", 0.0),
    ]
    .into_iter()
    .map(|(s, f)| (s.to_string(), example(40, s, f)))
    .collect();
    for (name, v) in crate::golden::cases() {
        if ["05-immobilie-hypothek", "07-bootstrap", "10-notverkauf"].contains(&name) {
            cases.push((name.to_string(), v));
        }
    }
    // The cases cover ruin (also only after the ruin age), emergency sales and the floor.
    let mut seen = [false; 4];
    for (name, v) in &cases {
        let plan = plan_of(v);
        for ruin_age in [None, Some(80.0)] {
            let o = Objective {
                ruin_age,
                ..Objective::default()
            };
            let (old, new) = (old_evaluate(&plan, &o), new_evaluate(&plan, &o));
            assert_eq!(bits(&old), bits(&new), "{name} {ruin_age:?}: {old:?} != {new:?}");
            seen[0] |= ruin_age.is_none() && new[1] > 0.0;
            seen[1] |= ruin_age.is_some() && new[1] > 0.0;
            seen[2] |= new[6] > 0.0;
            seen[3] |= new[13] > 0.0;
        }
    }
    assert_eq!(seen, [true; 4]);
}

/// #214/AK3: The figures of `batch::summarize` expressed with the building blocks: success rate, end wealth at the
/// percentiles (with the run), earliest depletion and the counts of runs with lump sums and sales.
fn summary_metrics() -> Vec<Metric> {
    use Aggregate as A;
    use Quantity as Q;
    use Reduce as R;
    let never = Condition { op: Op::Le, value: 0.0 };
    let mut out = vec![Metric::new(
        Q::Depleted,
        R::AtEnd,
        A::Probability { condition: Some(never) },
    )];
    out.extend(
        [0, 1, 5, 10, 25, 50, 75, 90, 95, 99]
            .map(|p: u32| Metric::new(Q::Total, R::AtEnd, A::Quantile { q: p as f64 / 100.0 })),
    );
    out.push(Metric::new(Q::Depleted, R::FirstWhen, A::Min));
    for q in [
        Q::LumpSums,
        Q::AffordabilitySales,
        Q::EmergencySales,
        Q::UnaffordableRenewals,
    ] {
        out.push(Metric::new(q, R::AtEnd, A::Count { condition: None }));
    }
    out
}

#[test]
fn issue_214_ak3_summarize_from_building_blocks_is_exact() {
    let mut cases = vec![("example".to_string(), example(60, "guardrails", 0.8))];
    for (name, v) in crate::golden::cases() {
        if [
            "05-immobilie-hypothek",
            "07-bootstrap",
            "08-parametrisch",
            "10-notverkauf",
        ]
        .contains(&name)
        {
            cases.push((name.to_string(), v));
        }
    }
    for (name, v) in &cases {
        let plan = plan_of(v);
        with_paths(&plan, |sim, paths| {
            let results = batch::run_paths(sim, paths);
            let s = batch::summarize(&results);
            let m = Measurement::new(&plan, sim, summary_metrics()).expect("metrics");
            let v: Vec<MetricValue> = m.run(sim, paths).expect("run");
            assert_eq!(v[0].value.map(f64::to_bits), Some(s.success_rate.to_bits()), "{name}");
            for (k, p) in s.percentiles.iter().enumerate() {
                let x = &v[1 + k];
                assert_eq!(
                    x.value.map(f64::to_bits),
                    Some(p.end_total.to_bits()),
                    "{name} {}",
                    p.percent
                );
                assert_eq!(x.path, Some(p.index), "{name} {}", p.percent);
                let r = &results[p.index].summary;
                assert_eq!(p.end_total_real.map(f64::to_bits), Some(r.end_total_real.to_bits()));
            }
            let earliest = v[11].path.map(|i| results[i].summary.depleted.expect("depleted"));
            assert_eq!(earliest, s.earliest_depletion.map(|(d, _)| d), "{name}");
            let counts: Vec<usize> = v[12..].iter().map(|x| x.value.unwrap_or(-1.0) as usize).collect();
            assert_eq!(
                counts,
                [
                    s.lump_sum_runs,
                    s.affordability_sale_runs,
                    s.emergency_sale_runs,
                    s.unaffordable_renewal_runs
                ],
                "{name}"
            );
        });
    }
}

/// Real annual consumption from retirement, optionally only the first `years` years.
fn consumption(years: Option<u32>, aggregate: Aggregate) -> Metric {
    Metric::new(Quantity::Consumption, Reduce::AnnualSums, aggregate)
        .real()
        .window(Window {
            from: Some("retirement".into()),
            years,
            ..Window::default()
        })
}

/// #214/AK4: Median, P10 and mean of real consumption in the years 1–15 from retirement, checked against the monthly
/// records; over the whole duration median and P10 equal those of `optimize --compare`.
#[test]
fn issue_214_ak4_consumption_in_the_first_15_years() {
    let v = example(50, "guardrails", 0.8);
    let plan = plan_of(&v);
    let q = |q| Aggregate::Quantile { q };
    let metrics = vec![
        consumption(Some(15), q(0.5)),
        consumption(Some(15), q(0.1)),
        consumption(Some(15), Aggregate::Mean),
        consumption(None, q(0.5)),
        consumption(None, q(0.1)),
    ];
    let got = with_paths(&plan, |sim, paths| {
        let m = Measurement::new(&plan, sim, metrics).expect("metrics");
        let got = m.run(sim, paths).expect("run");
        // Reference from the monthly records: 15 blocks of 12 months from retirement.
        let retire = sim.retire_at().expect("retirement");
        let mut all = vec![];
        for i in 0..paths.len() {
            let r = sim.run(&paths.get(i), true);
            let d = r.summary.depleted;
            let months: Vec<f64> = r
                .records
                .iter()
                .filter(|x| x.month >= retire)
                .map(|x| {
                    let spent = if d.is_some_and(|d| x.month >= d) {
                        x.spending.min(x.income).max(0.0)
                    } else {
                        x.spending
                    };
                    spent / x.price_index
                })
                .collect();
            all.extend(
                months
                    .chunks_exact(12)
                    .take(15)
                    .map(|c| c.iter().fold(0.0, |a, b| a + b)),
            );
        }
        assert_eq!(got[0].values, all.len());
        assert_eq!(all.len(), 15 * paths.len());
        let mean = all.iter().sum::<f64>() / all.len() as f64;
        all.sort_by(f64::total_cmp);
        let at = |q: f64| all[((all.len() - 1) as f64 * q).round() as usize];
        assert_eq!(got[0].value, Some(at(0.5)));
        assert_eq!(got[1].value, Some(at(0.1)));
        assert!(
            (got[2].value.unwrap() - mean).abs() < 1e-6 * mean,
            "{:?} {mean}",
            got[2].value
        );
        assert!(got[1].value < got[0].value);
        got
    });
    // Whole duration: the same values as the strategy comparison (`optimize --compare`).
    let req = optimize::Request {
        compare: true,
        strategies: vec!["guardrails".into()],
        max_ruin: 0.5,
        ..optimize::Request::default()
    };
    let report = optimize::run_request(&v, &library(), Some(&library()), &req).expect("compare");
    let m = &report.outcomes[0].validated;
    assert_eq!(got[3].value, Some(m.median));
    assert_eq!(got[4].value, Some(m.p10));
}

/// #214/AK5: Probabilities come with the standard error √(p(1 − p)/n).
#[test]
fn issue_214_ak5_probability_with_standard_error() {
    let mut v = example(80, "standard", 0.0);
    v["flows"]["spending"] = json!(9000); // so that some runs are depleted
    let plan = plan_of(&v);
    let ruin = Metric::new(
        Quantity::Depleted,
        Reduce::Any,
        Aggregate::Probability { condition: None },
    );
    let got = with_paths(&plan, |sim, paths| {
        let m = Measurement::new(&plan, sim, vec![ruin]).expect("metrics");
        m.run(sim, paths).expect("run")
    });
    let p = got[0].value.expect("value");
    assert!(p > 0.0 && p < 1.0, "{p}");
    assert_eq!(got[0].values, 80);
    let se = (p * (1.0 - p) / 80.0).sqrt();
    assert_eq!(got[0].std_error, Some(se));
    let json = serde_json::to_value(&got[0]).unwrap();
    assert!(json["stdError"].as_f64().is_some(), "{json}");
}
