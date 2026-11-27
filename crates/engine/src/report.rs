//! #207: Machine-readable results of the command line (`--json`): a fixed format per command, marked with `format`
//! (`vivaplan-run/1`, …) and documented in the help (reference). Compact: amounts rounded to whole currency units,
//! shares to four decimals, monthly values reduced to one row per year; a run of the example plan stays below
//! 20 000 characters.

use serde::Serialize;
use serde_json::{json, Value};

use crate::api::{BatchOut, RunOut, ScenariosOut};
use crate::message::Message;
use crate::sim::SimResult;
use crate::time::YearMonth;
use crate::validate::{Issue, Severity};

pub const RUN_FORMAT: &str = "vivaplan-run/1";
pub const VALIDATE_FORMAT: &str = "vivaplan-validate/1";
pub const BENCH_FORMAT: &str = "vivaplan-bench/1";
pub const OPTIMIZE_FORMAT: &str = "vivaplan-optimize/1";
pub const DIFF_FORMAT: &str = "vivaplan-diff/1";

/// Amount rounded to whole units.
fn amount(x: f64) -> i64 {
    x.round() as i64
}

/// Share (0..1) rounded to four decimals.
fn share(x: f64) -> f64 {
    (x * 1e4).round() / 1e4
}

/// A table: column names and one row per year.
#[derive(Debug, Serialize)]
pub struct Table {
    pub columns: Vec<&'static str>,
    pub rows: Vec<Vec<Value>>,
}

/// Result of `vivaplan run --json`.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RunReport {
    pub format: &'static str,
    /// Error that stopped the computation (`null` if computed).
    pub error: Option<Message>,
    /// Validation messages (errors and notes).
    pub issues: Vec<Issue>,
    /// With exactly one return path.
    pub single: Option<SingleReport>,
    /// With several return paths.
    pub batch: Option<BatchReport>,
    /// With variations (`scenarios.sweeps`).
    pub scenarios: Option<ScenariosReport>,
    pub elapsed_ms: i64,
    /// #214: Metrics given with `--metric` (missing without).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metrics: Option<Vec<crate::measure::MetricValue>>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SingleReport {
    pub label: String,
    pub start_total: i64,
    pub end_total: i64,
    /// End wealth in prices of the start month.
    pub end_total_real: i64,
    pub end_liquid: i64,
    pub total_at_retirement: Option<i64>,
    /// First month without funds (`null`: wealth lasts until the end).
    pub depleted: Option<YearMonth>,
    pub min_liquid: i64,
    pub min_liquid_month: Option<YearMonth>,
    pub pension_fund_monthly: i64,
    pub lumpsum_net: i64,
    pub lumpsum_tax: i64,
    pub taxes_total: i64,
    pub emergency_sales: usize,
    pub unaffordable_renewals: usize,
    /// Per year: wealth at the end of the year, income, spending, taxable income of the tax year (#281, canton) and
    /// taxes summed over the year; #271: with a care home
    /// also the column `care` (care home costs, part of `spending`); #149: with a taxable yield of the wealth also the
    /// column `taxableYield` (part of the taxable income of the tax year, not of `income`).
    pub years: Table,
    /// Events: month and text.
    pub events: Vec<(YearMonth, String)>,
}

const SINGLE_COLUMNS: [&str; 11] = [
    "year",
    "age",
    "liquid",
    "propertyEquity",
    "pensionFund",
    "pillar3a",
    "total",
    "income",
    "spending",
    "taxableIncome",
    "taxes",
];

fn single(r: &SimResult) -> SingleReport {
    let s = &r.summary;
    let mut rows: Vec<Vec<Value>> = vec![];
    let mut flows = [0.0; 6];
    let care = r.records.iter().any(|m| m.care != 0.0);
    let taxable_yield = r.records.iter().any(|m| m.assessed_yield != 0.0);
    for (i, m) in r.records.iter().enumerate() {
        flows[0] += m.income;
        flows[1] += m.spending;
        flows[2] += m.taxes;
        flows[3] += m.care;
        flows[4] += m.assessed_yield;
        flows[5] += m.assessed_taxable;
        if m.month.is_december() || i + 1 == r.records.len() {
            let mut row = vec![
                json!(m.month.year()),
                json!((m.age * 10.0).round() / 10.0),
                json!(amount(m.liquid)),
                json!(amount(m.property_equity)),
                json!(amount(m.pension_fund)),
                json!(amount(m.pillar3a)),
                json!(amount(m.total)),
                json!(amount(flows[0])),
                json!(amount(flows[1])),
                json!(amount(flows[5])),
                json!(amount(flows[2])),
            ];
            if care {
                row.push(json!(amount(flows[3])));
            }
            if taxable_yield {
                row.push(json!(amount(flows[4])));
            }
            rows.push(row);
            flows = [0.0; 6];
        }
    }
    let mut columns = SINGLE_COLUMNS.to_vec();
    if care {
        columns.push("care");
    }
    if taxable_yield {
        columns.push("taxableYield");
    }
    SingleReport {
        label: r.label.text().to_string(),
        start_total: amount(s.start_total),
        end_total: amount(s.end_total),
        end_total_real: amount(s.end_total_real),
        end_liquid: amount(s.end_liquid),
        total_at_retirement: s.total_at_retirement.map(amount),
        depleted: s.depleted,
        min_liquid: amount(s.min_liquid),
        min_liquid_month: s.min_liquid_month,
        pension_fund_monthly: amount(s.pension_fund_monthly),
        lumpsum_net: amount(s.lumpsum_net),
        lumpsum_tax: amount(s.lumpsum_tax),
        taxes_total: amount(s.taxes_total),
        emergency_sales: s.emergency_sales,
        unaffordable_renewals: s.unaffordable_renewals,
        years: Table { columns, rows },
        events: r.events.iter().map(|e| (e.month, e.text.text().to_string())).collect(),
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PercentileReport {
    /// 0 = worst run.
    pub percent: u32,
    pub end_total: i64,
    pub end_total_real: Option<i64>,
    pub depleted: Option<YearMonth>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BatchReport {
    pub runs: usize,
    /// Share of runs in which wealth lasts until the end.
    pub success_rate: f64,
    pub earliest_depletion: Option<YearMonth>,
    pub emergency_sale_runs: usize,
    pub unaffordable_renewal_runs: usize,
    /// End wealth at the percentiles 0 (worst run), 1, 5, 10, 25, 50, 75, 90, 95, 99, with the runs ordered by the
    /// nominal end wealth (`endTotal` ascends; `endTotalReal` is the real value of the same run and need not ascend).
    pub percentiles: Vec<PercentileReport>,
    /// #265: The same with the runs ordered by the end wealth in today's purchasing power (`endTotalReal` ascends).
    /// The run of a percentile may differ between the orders: runs with high inflation are nominally high, really low.
    pub percentiles_real: Vec<PercentileReport>,
    /// Per year (last month of the year): percentiles of the total wealth (nominal), median in prices of the start
    /// month, and share of the runs with wealth left.
    pub years: Table,
}

fn percentile_reports(list: &[crate::batch::Percentile]) -> Vec<PercentileReport> {
    list.iter()
        .map(|p| PercentileReport {
            percent: p.percent,
            end_total: amount(p.end_total),
            end_total_real: p.end_total_real.map(amount),
            depleted: p.depleted,
        })
        .collect()
}

const BATCH_COLUMNS: [&str; 8] = ["year", "p5", "p25", "p50", "p75", "p95", "p50Real", "survival"];

fn batch(b: &BatchOut) -> BatchReport {
    let s = &b.summary;
    let n = b.bands.months.len();
    let rows = (0..n)
        .filter(|&k| b.bands.months[k].is_december() || k + 1 == n)
        .map(|k| {
            let real = b.bands.real.as_ref().map_or(f64::NAN, |r| r.p50[k]);
            vec![
                json!(b.bands.months[k].year()),
                json!(amount(b.bands.p5[k])),
                json!(amount(b.bands.p25[k])),
                json!(amount(b.bands.p50[k])),
                json!(amount(b.bands.p75[k])),
                json!(amount(b.bands.p95[k])),
                json!(amount(real)),
                json!(share(b.stats.survival.get(k).copied().unwrap_or(f64::NAN))),
            ]
        })
        .collect();
    BatchReport {
        runs: s.runs,
        success_rate: share(s.success_rate),
        earliest_depletion: s.earliest_depletion.as_ref().map(|(m, _)| *m),
        emergency_sale_runs: s.emergency_sale_runs,
        unaffordable_renewal_runs: s.unaffordable_renewal_runs,
        percentiles: percentile_reports(&s.percentiles),
        percentiles_real: percentile_reports(&s.percentiles_real),
        years: Table {
            columns: BATCH_COLUMNS.to_vec(),
            rows,
        },
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ScenarioReport {
    /// Values of the variations, in the order of `params`.
    pub values: Vec<Value>,
    pub runs: usize,
    pub success_rate: f64,
    pub p5: i64,
    pub p50: i64,
    pub p95: i64,
    pub p50_real: i64,
    pub at_retirement: Option<i64>,
    pub earliest_depletion: Option<YearMonth>,
    pub error: Option<Message>,
}

#[derive(Debug, Serialize)]
pub struct ScenariosReport {
    pub params: Vec<String>,
    pub results: Vec<ScenarioReport>,
}

fn scenarios(s: &ScenariosOut) -> ScenariosReport {
    ScenariosReport {
        params: s.params.clone(),
        results: s
            .results
            .iter()
            .map(|r| ScenarioReport {
                values: r.values.clone(),
                runs: r.runs,
                success_rate: share(r.success_rate),
                p5: amount(r.p5),
                p50: amount(r.p50),
                p95: amount(r.p95),
                p50_real: amount(r.p50_real),
                at_retirement: r.at_retirement.map(amount),
                earliest_depletion: r.earliest_depletion,
                error: r.error.clone(),
            })
            .collect(),
    }
}

/// The compact result of a computation.
pub fn run(out: &RunOut) -> RunReport {
    RunReport {
        format: RUN_FORMAT,
        error: out.error.clone(),
        issues: out.issues.clone(),
        single: out.single.as_ref().map(single),
        batch: out.batch.as_ref().map(batch),
        scenarios: out.scenarios.as_ref().map(scenarios),
        elapsed_ms: amount(out.elapsed_ms),
        metrics: None,
    }
}

/// Result of `vivaplan validate --json`.
#[derive(Debug, Serialize)]
pub struct ValidateReport {
    pub format: &'static str,
    /// No message with severity `error`.
    pub valid: bool,
    pub issues: Vec<Issue>,
}

pub fn validate(issues: Vec<Issue>) -> ValidateReport {
    ValidateReport {
        format: VALIDATE_FORMAT,
        valid: !issues.iter().any(|i| i.severity == Severity::Error),
        issues,
    }
}

/// Result of `vivaplan bench --json`.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BenchReport {
    pub format: &'static str,
    pub runs: usize,
    pub months: usize,
    pub elapsed_ms: f64,
    pub us_per_run: f64,
    pub cores: usize,
    /// End wealth of the last run (check value).
    pub check: i64,
}

impl BenchReport {
    pub fn new(runs: usize, months: usize, seconds: f64, cores: usize, check: f64) -> Self {
        let round1 = |x: f64| (x * 10.0).round() / 10.0;
        BenchReport {
            format: BENCH_FORMAT,
            runs,
            months,
            elapsed_ms: round1(seconds * 1e3),
            us_per_run: round1(seconds * 1e6 / runs.max(1) as f64),
            cores,
            check: amount(check),
        }
    }
}

/// Result of `vivaplan optimize --json`: the report of the optimization with `format` in front.
#[derive(Debug, Serialize)]
pub struct OptimizeReport<'a> {
    pub format: &'static str,
    #[serde(flatten)]
    pub report: &'a crate::optimize::Report,
}

pub fn optimize(report: &crate::optimize::Report) -> OptimizeReport<'_> {
    OptimizeReport {
        format: OPTIMIZE_FORMAT,
        report,
    }
}

/// Result of `vivaplan diff --json`.
#[derive(Debug, Serialize)]
pub struct DiffReport {
    pub format: &'static str,
    pub changes: Vec<crate::fieldpath::Change>,
}

pub fn diff(changes: Vec<crate::fieldpath::Change>) -> DiffReport {
    DiffReport {
        format: DIFF_FORMAT,
        changes,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    fn library() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("data")
    }

    /// #265/AK6: The report (`--json`) names the percentiles in both orders: `percentiles` ascends nominally,
    /// `percentilesReal` in today's purchasing power.
    // slow: run of the example plan with 60 paths
    #[test]
    fn issue_265_ak6_report_names_both_orders() {
        let mut plan = crate::api::example_plan();
        plan["scenarios"]["sweeps"] = json!([]);
        plan["returns"]["source"] = json!("parametric");
        plan["returns"]["parametric"] = json!({ "paths": 60, "mu": 0.06, "sigma": 0.15, "df": 0, "seed": 3 });
        // #303: the inflation of the former example strategy keeps the two orders apart
        plan["returns"]["strategy"] = json!({ "stocks": "sp500", "bonds": "usa" });
        // #306: and the former rules of inflation of the example (nominal earned income, rent with inflation)
        plan.as_object_mut().unwrap().remove("indexedByDefault");
        plan["pension"]["ahv"]["indexed"] = json!(true);
        for p in plan["realestate"]["properties"].as_array_mut().unwrap() {
            for k in ["inflation", "inflationFixCosts", "inflationRent", "inflationValue"] {
                p[k] = json!(true);
            }
        }
        let v = serde_json::to_value(run(&crate::api::run(&plan, Path::new("."), Some(&library())))).unwrap();
        let col = |list: &str, key: &str| -> Vec<i64> {
            v["batch"][list]
                .as_array()
                .unwrap()
                .iter()
                .map(|p| p[key].as_i64().unwrap())
                .collect()
        };
        let ascending = |x: Vec<i64>| x.windows(2).all(|w| w[0] <= w[1]);
        assert_eq!(col("percentilesReal", "percent"), col("percentiles", "percent"));
        assert!(ascending(col("percentiles", "endTotal")));
        assert!(ascending(col("percentilesReal", "endTotalReal")));
        assert!(
            !ascending(col("percentiles", "endTotalReal")),
            "Ordnungen gleich: Testfall ohne Aussage"
        );
    }

    /// #207/AK5: the result of `run` for the example plan stays below 20 000 characters (with variations), also
    /// with several paths.
    // slow: full run of the example plan with variations
    #[test]
    fn issue_207_ak5_run_report_of_the_example_is_short() {
        let plan = crate::api::example_plan();
        let text = serde_json::to_string(&run(&crate::api::run(&plan, Path::new("."), Some(&library())))).unwrap();
        assert!(text.len() < 20_000, "{} characters", text.len());
        let v: Value = serde_json::from_str(&text).unwrap();
        assert_eq!(v["format"], RUN_FORMAT);
        assert!(v["error"].is_null(), "{v}");
        assert!(v["single"]["endTotal"].is_i64());
        let years = &v["single"]["years"];
        assert_eq!(years["columns"][0], "year");
        assert_eq!(years["rows"][0].as_array().unwrap().len(), SINGLE_COLUMNS.len());
        assert_eq!(v["scenarios"]["params"][0], "wealth.stockShare");
        assert_eq!(v["scenarios"]["results"].as_array().unwrap().len(), 8);

        let mut multi = plan.clone();
        multi["returns"]["source"] = json!("parametric");
        multi["returns"]["parametric"]["paths"] = json!(200);
        let text = serde_json::to_string(&run(&crate::api::run(&multi, Path::new("."), Some(&library())))).unwrap();
        assert!(text.len() < 20_000, "{} characters", text.len());
        let v: Value = serde_json::from_str(&text).unwrap();
        assert!(v["single"].is_null());
        assert_eq!(v["batch"]["runs"], 200);
        assert_eq!(v["batch"]["percentiles"][0]["percent"], 0);
        assert_eq!(v["batch"]["years"]["columns"], json!(BATCH_COLUMNS));
        let rows = v["batch"]["years"]["rows"].as_array().unwrap();
        let last = rows.last().unwrap();
        assert!(last[7].as_f64().unwrap() <= 1.0);
        // One row per year, the last one for the last month (June 2065, not a December).
        assert_eq!(rows.first().unwrap()[0], 2026);
        assert_eq!(last[0], 2065);
        assert_eq!(rows.len(), 2065 - 2026 + 1);
    }

    /// #207/AK1: the yearly values of a single run: wealth at the end of the year, flows summed over the year.
    #[test]
    fn issue_207_ak1_single_run_years_sum_the_flows() {
        let mut plan = crate::api::example_plan();
        plan["scenarios"]["sweeps"] = json!([]);
        let out = crate::api::run(&plan, Path::new("."), Some(&library()));
        let res = out.single.as_ref().unwrap();
        let r = run(&out);
        let s = r.single.unwrap();
        assert!(r.scenarios.is_none());
        // The second year: a full year with income, spending and taxes.
        let year = res.records[0].month.year() + 1;
        let sum = |f: fn(&crate::sim::MonthRecord) -> f64| -> f64 {
            res.records.iter().filter(|m| m.month.year() == year).map(f).sum()
        };
        let row = &s.years.rows[1];
        assert_eq!(row[0], json!(year));
        assert!(sum(|m| m.taxes) > 0.0);
        assert_eq!(row[7], json!(amount(sum(|m| m.income))));
        assert_eq!(row[8], json!(amount(sum(|m| m.spending))));
        assert_eq!(row[10], json!(amount(sum(|m| m.taxes))));
        let last = res.records.last().unwrap();
        assert_eq!(s.years.rows.last().unwrap()[6], json!(amount(last.total)));
        assert_eq!(s.end_total, amount(res.summary.end_total));
        assert_eq!(s.events.len(), res.events.len());
    }

    /// #271/AK8: the report shows the care home costs in the column `care` only for a plan with a care home.
    #[test]
    fn issue_271_ak8_report_column_care() {
        let mut plan = crate::api::example_plan();
        plan["scenarios"]["sweeps"] = json!([]);
        let out = crate::api::run(&plan, Path::new("."), Some(&library()));
        assert!(!run(&out).single.unwrap().years.columns.contains(&"care"));
        plan["household"]["persons"][0]["careHome"] = json!({ "age": 85, "monthly": 9000, "sell": true });
        let out = crate::api::run(&plan, Path::new("."), Some(&library()));
        let res = out.single.as_ref().unwrap();
        let s = run(&out).single.unwrap();
        let col = s.years.columns.iter().position(|c| *c == "care").expect("column care");
        assert_eq!(col, SINGLE_COLUMNS.len());
        let (i, row) = s
            .years
            .rows
            .iter()
            .enumerate()
            .find(|(_, r)| r[col] != json!(0))
            .expect("a year with costs");
        let year = row[0].as_i64().unwrap() as i32;
        let sum: f64 = res
            .records
            .iter()
            .filter(|m| m.month.year() == year)
            .map(|m| m.care)
            .sum();
        assert!(i > 0 && sum > 0.0);
        assert_eq!(row[col], json!(amount(sum)));
    }

    /// #149/AK6: the report shows the taxable yield of the wealth in the column `taxableYield` only with a yield; the
    /// value of a year is the assessed yield of December.
    #[test]
    fn issue_149_ak6_report_column_taxable_yield() {
        let mut plan = crate::api::example_plan();
        plan["scenarios"]["sweeps"] = json!([]);
        let out = crate::api::run(&plan, Path::new("."), Some(&library()));
        assert!(!run(&out).single.unwrap().years.columns.contains(&"taxableYield"));
        plan["taxes"]["investmentYield"] = json!({ "stocks": 0.02, "bonds": 0.01 });
        let out = crate::api::run(&plan, Path::new("."), Some(&library()));
        let res = out.single.as_ref().unwrap();
        let s = run(&out).single.unwrap();
        let col = s
            .years
            .columns
            .iter()
            .position(|c| *c == "taxableYield")
            .expect("column taxableYield");
        assert_eq!(col, SINGLE_COLUMNS.len());
        let dec = res.records.iter().find(|m| m.month.is_december()).expect("a December");
        assert!(dec.assessed_yield > 0.0);
        let row = s.years.rows.iter().find(|r| r[0] == json!(dec.month.year())).unwrap();
        assert_eq!(row[col], json!(amount(dec.assessed_yield)));
    }

    /// #281/AK5: the report shows the taxable income of the tax year (canton) in the column `taxableIncome` directly
    /// before `taxes`; a year without an assessment (the last year without a December) shows 0.
    #[test]
    fn issue_281_ak5_report_column_taxable_income() {
        let mut plan = crate::api::example_plan();
        plan["scenarios"]["sweeps"] = json!([]);
        let out = crate::api::run(&plan, Path::new("."), Some(&library()));
        let res = out.single.as_ref().unwrap();
        let s = run(&out).single.unwrap();
        let col = s.years.columns.iter().position(|c| *c == "taxableIncome").unwrap();
        assert_eq!(s.years.columns[col + 1], "taxes");
        assert_eq!(s.years.columns[col - 1], "spending");
        let dec = res.records.iter().filter(|m| m.month.is_december()).nth(1).unwrap();
        assert!(dec.assessed_taxable > 0.0);
        let row = s.years.rows.iter().find(|r| r[0] == json!(dec.month.year())).unwrap();
        assert_eq!(row[col], json!(amount(dec.assessed_taxable)));
        // #281/AK3: the last year ends before December: no assessment, 0
        assert!(!res.records.last().unwrap().month.is_december());
        assert_eq!(s.years.rows.last().unwrap()[col], json!(0));
    }

    /// #207/AK1: the other formats.
    #[test]
    fn issue_207_ak1_validate_bench_and_error_formats() {
        let v = validate(crate::api::validate(&crate::api::example_plan()));
        assert!(v.valid);
        assert_eq!(v.format, VALIDATE_FORMAT);
        let mut bad = crate::api::example_plan();
        bad["taxes"]["multiplier"] = json!(-1);
        let v = validate(crate::api::validate(&bad));
        assert!(!v.valid);
        let b = serde_json::to_value(BenchReport::new(2000, 500, 0.5, 8, 1234.4)).unwrap();
        assert_eq!(
            b,
            json!({ "format": BENCH_FORMAT, "runs": 2000, "months": 500, "elapsedMs": 500.0, "usPerRun": 250.0, "cores": 8, "check": 1234 })
        );
        let mut broken = crate::api::example_plan();
        broken["wealth"]["cash"] = json!("x");
        let r = run(&crate::api::run(&broken, Path::new("."), Some(&library())));
        assert!(r.error.is_some() && r.single.is_none() && r.batch.is_none());
        let d = serde_json::to_string(&diff(vec![crate::fieldpath::Change {
            path: "a.b".into(),
            old: json!(1),
            new: json!(2),
        }]))
        .unwrap();
        assert_eq!(
            d,
            r#"{"format":"vivaplan-diff/1","changes":[{"path":"a.b","old":1,"new":2}]}"#
        );
        assert_eq!(amount(f64::NAN), 0);
        assert_eq!(share(0.123_456), 0.1235);
    }
}
