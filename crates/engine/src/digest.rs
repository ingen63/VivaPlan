//! #209: Evaluation of computed results for the user interface, the command line and the MCP server: yearly values,
//! digest (key figures with the result in words as a structured verdict) and the comparison of several results.
//!
//! The functions read results as the user interface keeps and saves them (JSON of [`crate::api::RunOut`] and
//! [`crate::sim::SimResult`]); fields missing in results of older versions count as missing or 0. Amounts come from the
//! building blocks of the measurement layer (#214): [`Quantity::value`] for a quantity of the month, nominal or real
//! ([`measure::in_prices`]). The verdict is a category with its values, not a sentence: the user interface words it in
//! its language, the command line and the MCP server in English.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::fieldpath::Change;
use crate::measure::{self, Quantity};
use crate::model::TaxLocation;
use crate::sim::{MonthRecord, Summary, WealthBasis};
use crate::time::YearMonth;

/// Yearly values of a run, nominal and in prices of the start month.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Yearly {
    pub nominal: Vec<YearRow>,
    pub real: Vec<YearRow>,
}

/// Values of a calendar year: balances at the end of the year, flows as the monthly average of the year, taxes as sum
/// of the year, and the sums of the year for the bar charts (`flows`).
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct YearRow {
    pub year: i32,
    /// Months of the year in the plan (fewer than 12 in the first and last year).
    pub months: usize,
    /// Age of the first person at the end of the year.
    pub age: f64,
    pub liquid: f64,
    pub property: f64,
    /// Pension fund and pillar 3a.
    pub pension: f64,
    pub total: f64,
    /// Monthly average: planned living costs, housing costs, care home costs (#271) and pillar 3a contributions (#52).
    pub spending: f64,
    /// #271: Monthly average of the care home costs; missing without a care home.
    #[serde(skip_serializing_if = "is_zero")]
    pub care: f64,
    /// Monthly average of the withdrawal from wealth.
    pub draw: f64,
    /// Monthly average: income and withdrawal.
    pub receipts: f64,
    /// Withdrawal of the year, nominal; an incomplete year extrapolated to 12 months.
    pub year_draw: f64,
    /// Withdrawal rate: `year_draw` relative to total wealth at the start of the year (nominal). `null` without a
    /// withdrawal, or with a withdrawal but no wealth at the start of the year.
    pub rate: Option<f64>,
    /// Receipts minus spending (monthly average).
    pub deviation: f64,
    /// Inflation of the year: price index at the end of the year against the end of the previous year.
    pub inflation: f64,
    /// #149/AK6: Taxable yield of the wealth of the tax year (part of the income of `tax_income`); missing without a
    /// yield.
    #[serde(skip_serializing_if = "is_zero")]
    pub tax_yield: f64,
    pub tax_income: f64,
    pub tax_wealth: f64,
    pub tax_capital: f64,
    pub tax_gain: f64,
    pub tax_total: f64,
    /// #131/AK14: Assessment of the tax year (income, deductions by origin, taxable income).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub basis: Option<YearTaxBasis>,
    /// #151/AK7: Place of residence of the tax year.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub residence: Option<TaxLocation>,
    pub flows: YearFlows,
}

/// Assessment of a tax year, amounts nominal or real like the row.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct YearTaxBasis {
    pub income: f64,
    pub proposal_canton: f64,
    pub proposal_federal: f64,
    pub own: f64,
    pub extra: f64,
    pub one_off: f64,
    pub pillar3a: f64,
    pub taxable_canton: f64,
    pub taxable_federal: f64,
    /// #271: Care home costs of the year and their deduction as costs of illness (canton, federal tax).
    #[serde(skip_serializing_if = "is_zero")]
    pub care: f64,
    #[serde(skip_serializing_if = "is_zero")]
    pub care_canton: f64,
    #[serde(skip_serializing_if = "is_zero")]
    pub care_federal: f64,
    /// #229/AK7: Tax value of the properties (only if a property has one).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub property_tax_value: Option<f64>,
    /// #231/AK5: Deductions from wealth and taxable wealth (only with deductions from wealth).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wealth: Option<WealthBasis>,
}

/// Sums of a calendar year for the bar charts «income and spending» and «withdrawal».
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct YearFlows {
    pub earned: f64,
    pub pk: f64,
    pub ahv: f64,
    /// Living costs without housing costs and pillar 3a contributions.
    pub living: f64,
    pub housing: f64,
    /// #271: Care home costs; missing without a care home.
    #[serde(skip_serializing_if = "is_zero")]
    pub care: f64,
    pub p3a: f64,
    pub taxes: f64,
    /// Earned income and pensions.
    pub income: f64,
    /// Living costs, housing costs, care home costs, pillar 3a contributions and taxes.
    pub spending: f64,
    /// Liquid wealth before the first month of the year.
    pub liquid_start: f64,
    /// Spending and taxes not covered by earned income and pensions.
    pub draw: f64,
    /// `draw` (at least 0) relative to `liquid_start`, per 12 months; 0 without liquid wealth.
    pub rate: f64,
}

fn is_zero(v: &f64) -> bool {
    *v == 0.0
}

/// A run as the user interface keeps it: summary and monthly values.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct StoredRun {
    pub summary: Summary,
    pub records: Vec<MonthRecord>,
}

/// Yearly values of a run, nominal and real.
pub fn yearly(run: &StoredRun) -> Yearly {
    Yearly {
        nominal: years(&run.records, run.summary.start_total, false),
        real: years(&run.records, run.summary.start_total, true),
    }
}

/// Values per calendar year. `start_total`: total wealth before the first month (basis of the withdrawal rate in the
/// first year). The order of the additions is that of the former computation in the user interface, so that the
/// display stays the same to the last digit (#209/AK3).
fn years(records: &[MonthRecord], start_total: f64, real: bool) -> Vec<YearRow> {
    let none = Summary::default();
    let q = |quantity: Quantity, r: &MonthRecord| quantity.value(real, r, &none);
    let nominal = |quantity: Quantity, r: &MonthRecord| quantity.value(false, r, &none);
    let at = |x: f64, r: &MonthRecord| measure::in_prices(x, real, r);
    let sum = |rs: &[MonthRecord], quantity: Quantity| rs.iter().fold(0.0, |a, r| a + q(quantity, r));
    let avg = |rs: &[MonthRecord], quantity: Quantity| sum(rs, quantity) / rs.len() as f64;
    let mut out = vec![];
    let mut prev_last: Option<&MonthRecord> = None;
    for rs in records.chunk_by(|a, b| a.month.year() == b.month.year()) {
        let (first, last) = (&rs[0], &rs[rs.len() - 1]);
        let n = rs.len() as f64;
        let planned = avg(rs, Quantity::PlannedSpending);
        let housing = avg(rs, Quantity::Housing);
        let income = avg(rs, Quantity::Income);
        let draw = avg(rs, Quantity::Withdrawal);
        let p3a_in = avg(rs, Quantity::Pillar3aIn);
        let care = rs.iter().fold(0.0, |a, r| a + at(r.care, r)) / n;
        // #271: without a care home `care` is 0 and the sum the same as before
        let spending = planned + housing + care + p3a_in;
        let receipts = income + draw;
        let prev_index = prev_last.map_or(1.0, |r| r.price_index);
        let year_draw = rs.iter().fold(0.0, |a, r| a + nominal(Quantity::Withdrawal, r)) / n * 12.0;
        let base_total = prev_last.map_or(start_total, |r| r.total);
        prev_last = Some(last);
        let (tax_income, tax_wealth) = (sum(rs, Quantity::AssessedIncome), sum(rs, Quantity::AssessedWealth));
        let (tax_capital, tax_gain) = (sum(rs, Quantity::TaxCapital), sum(rs, Quantity::TaxGain));
        let tax_yield = rs.iter().fold(0.0, |a, r| a + at(r.assessed_yield, r));
        let assessed = rs.iter().find(|r| r.tax_basis.is_some());
        let basis = assessed
            .and_then(|r| r.tax_basis.as_ref().map(|b| (r, b)))
            .map(|(r, b)| YearTaxBasis {
                income: at(b.income, r),
                proposal_canton: at(b.proposal_canton, r),
                proposal_federal: at(b.proposal_federal, r),
                own: at(b.own, r),
                extra: at(b.extra, r),
                one_off: at(b.one_off, r),
                pillar3a: at(b.pillar3a, r),
                taxable_canton: at(b.taxable_canton, r),
                taxable_federal: at(b.taxable_federal, r),
                care: at(b.care, r),
                care_canton: at(b.care_canton, r),
                care_federal: at(b.care_federal, r),
                property_tax_value: b.property_tax_value.map(|v| at(v, r)),
                wealth: b.wealth.map(|w| WealthBasis {
                    wealth: at(w.wealth, r),
                    ongoing: at(w.ongoing, r),
                    one_off: at(w.one_off, r),
                    taxable: at(w.taxable, r),
                }),
            });
        let residence = assessed.and_then(|r| r.tax_basis.as_ref()?.residence.clone());
        let mut f = YearFlows {
            earned: 0.0,
            pk: 0.0,
            ahv: 0.0,
            living: 0.0,
            housing: 0.0,
            care: 0.0,
            p3a: 0.0,
            taxes: 0.0,
            income: 0.0,
            spending: 0.0,
            liquid_start: at(
                nominal(Quantity::Liquid, first) + nominal(Quantity::Spending, first)
                    - nominal(Quantity::Income, first)
                    + nominal(Quantity::Taxes, first),
                first,
            ),
            draw: 0.0,
            rate: 0.0,
        };
        for r in rs {
            f.earned += q(Quantity::Earned, r);
            f.pk += q(Quantity::PkPension, r);
            f.ahv += q(Quantity::Ahv, r);
            f.housing += q(Quantity::Housing, r);
            f.p3a += q(Quantity::Pillar3aIn, r);
            f.care += at(r.care, r);
            f.living += at(
                nominal(Quantity::Spending, r)
                    - nominal(Quantity::Housing, r)
                    - nominal(Quantity::Pillar3aIn, r)
                    - r.care,
                r,
            );
            f.taxes += q(Quantity::Taxes, r);
        }
        f.income = f.earned + f.pk + f.ahv;
        f.spending = f.living + f.housing + f.care + f.p3a + f.taxes;
        f.draw = f.spending - f.earned - f.pk - f.ahv;
        f.rate = if f.liquid_start > 0.0 {
            f.draw.max(0.0) / f.liquid_start * 12.0 / n
        } else {
            0.0
        };
        out.push(YearRow {
            year: last.month.year(),
            months: rs.len(),
            age: last.age,
            liquid: q(Quantity::Liquid, last),
            property: q(Quantity::PropertyEquity, last),
            pension: at(
                nominal(Quantity::PensionFund, last) + nominal(Quantity::Pillar3a, last),
                last,
            ),
            total: q(Quantity::Total, last),
            spending,
            care,
            draw,
            receipts,
            year_draw,
            rate: (year_draw > 0.0 && base_total > 0.0).then(|| year_draw / base_total),
            deviation: receipts - spending,
            inflation: last.price_index / prev_index - 1.0,
            tax_yield,
            tax_income,
            tax_wealth,
            tax_capital,
            tax_gain,
            tax_total: tax_income + tax_wealth + tax_capital + tax_gain,
            basis,
            residence,
            flows: f,
        });
    }
    out
}

/// Result in words as a category (#72): for a single run whether the wealth lasts; for several runs the class of the
/// success rate (limits of the traffic light, decisive is the value rounded to 0.1 %: 95.0 % is high, 80.0 % medium).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Verdict {
    /// Single run: the wealth lasts until the end of the plan.
    Lasts,
    /// Single run: the wealth is depleted before the end.
    Depleted,
    /// Success rate at least 95 %.
    High,
    /// Success rate at least 80 %.
    Medium,
    /// Success rate below 80 %.
    Low,
}

/// Suggested next step: variations for a sound result, the optimization for a weak one.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum NextStep {
    Scenarios,
    Optimize,
}

/// End wealth of a percentile, nominal and in today's purchasing power (with the inflation of the run).
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EndWealth {
    /// Percentile; 0 is the worst run. A single run counts as the median (50).
    pub percent: u32,
    pub nominal: Option<f64>,
    pub real: Option<f64>,
}

/// Digest of a result: key figures and the verdict.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Digest {
    pub verdict: Verdict,
    pub next: NextStep,
    /// Number of runs (1 for a single run).
    pub runs: usize,
    /// Share of runs in which the wealth lasts (several runs only).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub success_rate: Option<f64>,
    /// First month without funds: of the single run, or the earliest over all runs.
    pub depleted: Option<YearMonth>,
    /// End wealth: of the single run (as percentile 50), or per percentile. #265: `nominal` is the run of the
    /// percentile in the order by nominal end wealth, `real` the run in the order by end wealth in today's purchasing
    /// power (see `real_order`); both ascend with the percentile.
    pub end_wealth: Vec<EndWealth>,
    /// Several runs (#265): `true` if the real values of `end_wealth` come from the order by real end wealth, `false`
    /// for results saved before #265, whose real value is that of the run of the nominal order (need not ascend).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub real_order: Option<bool>,
    /// Single run: total wealth at retirement.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_at_retirement: Option<f64>,
    /// Single run: sum of the taxes.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub taxes_total: Option<f64>,
    /// Several runs: runs with at least one emergency sale of a property.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emergency_sale_runs: Option<usize>,
}

/// A result as the user interface keeps and saves it (fields of [`crate::api::RunOut`] that the evaluation reads).
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct StoredResult {
    pub single: Option<StoredRun>,
    pub batch: Option<StoredBatch>,
}

/// Several runs as saved (without the bands and histograms, which the evaluation does not read).
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct StoredBatch {
    pub summary: StoredBatchSummary,
    pub stats: StoredStats,
    pub details: Vec<(u32, StoredRun)>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct StoredBatchSummary {
    pub runs: usize,
    pub success_rate: f64,
    pub percentiles: Vec<StoredPercentile>,
    /// Percentiles in the order by real end wealth; missing in results before #265.
    pub percentiles_real: Vec<StoredPercentile>,
    /// Month and reason (the reason is not read).
    pub earliest_depletion: Option<(YearMonth, Value)>,
    pub emergency_sale_runs: usize,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct StoredPercentile {
    pub percent: u32,
    pub end_total: f64,
    /// Missing in results before #126.
    pub end_total_real: Option<f64>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct StoredStats {
    pub survival: Vec<f64>,
    pub depletion_ages: Vec<(u32, usize)>,
}

/// #272/AK3: Estimated tax of a full calendar year for the card «Steuern»: income and wealth tax of the tax year (as
/// on the tax assessment, without lump sums and property gains) in prices of the start month, rounded to 100.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaxEstimate {
    /// `first`: first full calendar year of the plan; `retired`: first full calendar year after the retirement.
    pub kind: &'static str,
    pub year: i32,
    pub amount: f64,
}

/// #272/AK3: Estimated tax of the first full calendar year and of the first full calendar year after the retirement
/// `retire` (the household's last payout of a pension fund). Only the first if the retirement is before the start,
/// missing, or the two years are the same; none without a full calendar year.
pub fn tax_estimates(records: &[MonthRecord], retire: Option<YearMonth>) -> Vec<TaxEstimate> {
    let rows = years(records, 0.0, true);
    let full = |y: &&YearRow| y.months == 12;
    let estimate = |kind, y: &YearRow| TaxEstimate {
        kind,
        year: y.year,
        amount: ((y.tax_income + y.tax_wealth) / 100.0).round() * 100.0,
    };
    let Some(first) = rows.iter().find(full) else {
        return vec![];
    };
    let mut out = vec![estimate("first", first)];
    if let Some(at) = retire {
        // the whole year after the retirement: January at or after the month of the retirement; with the retirement
        // at or before the start that is the first full year, then only one estimate
        let after = rows.iter().filter(full).find(|y| YearMonth::new(y.year, 1) >= at);
        if let Some(y) = after.filter(|y| y.year != first.year) {
            out.push(estimate("retired", y));
        }
    }
    out
}

/// Class of a success rate; decisive is the value rounded to 0.1 % as displayed (#72).
pub fn rate_class(success_rate: f64) -> Verdict {
    let shown: f64 = format!("{:.1}", success_rate * 100.0).parse().unwrap_or(0.0);
    if shown >= 95.0 {
        Verdict::High
    } else if shown >= 80.0 {
        Verdict::Medium
    } else {
        Verdict::Low
    }
}

impl StoredBatch {
    /// End wealth per percentile: nominal of the nominal order, real of the real order (#265). Results before #265
    /// take the real value of the run of the nominal order; results before #126 from the detail runs (#128).
    fn end_wealth(&self) -> Vec<EndWealth> {
        let real_of = |p: &StoredPercentile| {
            if !self.summary.percentiles_real.is_empty() {
                return self
                    .summary
                    .percentiles_real
                    .iter()
                    .find(|q| q.percent == p.percent)
                    .and_then(|q| q.end_total_real);
            }
            p.end_total_real.or_else(|| {
                self.details
                    .iter()
                    .find(|(q, _)| *q == p.percent)
                    .map(|(_, d)| d.summary.end_total_real)
            })
        };
        self.summary
            .percentiles
            .iter()
            .map(|p| EndWealth {
                percent: p.percent,
                nominal: Some(p.end_total),
                real: real_of(p),
            })
            .collect()
    }
}

/// Digest of a result: the single run if there is one, otherwise the several runs; `None` without either (error, or only
/// variations).
pub fn digest(result: &StoredResult) -> Option<Digest> {
    if let Some(run) = &result.single {
        let s = &run.summary;
        let next = if s.depleted.is_some() {
            NextStep::Optimize
        } else {
            NextStep::Scenarios
        };
        return Some(Digest {
            verdict: if s.depleted.is_some() {
                Verdict::Depleted
            } else {
                Verdict::Lasts
            },
            next,
            runs: 1,
            success_rate: None,
            depleted: s.depleted,
            end_wealth: vec![EndWealth {
                percent: 50,
                nominal: Some(s.end_total),
                real: Some(s.end_total_real),
            }],
            total_at_retirement: s.total_at_retirement,
            taxes_total: Some(s.taxes_total),
            emergency_sale_runs: None,
            real_order: None,
        });
    }
    let b = result.batch.as_ref()?;
    let s = &b.summary;
    let verdict = rate_class(s.success_rate);
    Some(Digest {
        verdict,
        next: if verdict == Verdict::High {
            NextStep::Scenarios
        } else {
            NextStep::Optimize
        },
        runs: s.runs,
        success_rate: Some(s.success_rate),
        depleted: s.earliest_depletion.as_ref().map(|(m, _)| *m),
        end_wealth: b.end_wealth(),
        total_at_retirement: None,
        taxes_total: None,
        emergency_sale_runs: Some(s.emergency_sale_runs),
        real_order: Some(!s.percentiles_real.is_empty()),
    })
}

/// A result in the comparison with the plan it was computed with.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(default)]
pub struct CompareInput {
    pub result: StoredResult,
    pub plan: Option<Value>,
}

/// What a result holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ResultKind {
    Single,
    Batch,
    /// Only the metrics of the variations, no run.
    Scenarios,
}

/// Key figures of a result in the comparison.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompareItem {
    pub kind: ResultKind,
    pub digest: Option<Digest>,
    /// Depletion for sorting: the month, `null` if the money lasts in every run (sorts last); missing for a result
    /// without a run.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub depletion: Option<Option<YearMonth>>,
    /// Share of the runs whose money runs out at the respective age of [`Comparison::depletion_ages`] (several runs
    /// only, otherwise empty).
    pub depletion_shares: Vec<f64>,
    /// Differences of the plan against the plan of the first result.
    pub plan_changes: Vec<Change>,
}

/// Comparison of several results side by side.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Comparison {
    pub items: Vec<CompareItem>,
    /// Percentiles of all results with several runs, highest first.
    pub percentiles: Vec<u32>,
    /// Ages (whole years) at which the money runs out in any run of any result, ascending.
    pub depletion_ages: Vec<u32>,
}

/// Compares results: key figures per result, the percentiles and ages at depletion of all results, the differences of
/// the plans against the first.
pub fn compare(inputs: &[CompareInput]) -> Comparison {
    let mut percentiles: Vec<u32> = inputs
        .iter()
        .filter_map(|i| i.result.batch.as_ref())
        .flat_map(|b| b.summary.percentiles.iter().map(|p| p.percent))
        .collect();
    percentiles.sort_unstable_by(|a, b| b.cmp(a));
    percentiles.dedup();
    let mut depletion_ages: Vec<u32> = inputs
        .iter()
        .filter_map(|i| i.result.batch.as_ref())
        .flat_map(|b| b.stats.depletion_ages.iter().map(|(a, _)| *a))
        .collect();
    depletion_ages.sort_unstable();
    depletion_ages.dedup();
    let first_plan = inputs.first().and_then(|i| i.plan.clone()).unwrap_or(Value::Null);
    let items = inputs
        .iter()
        .map(|i| {
            let r = &i.result;
            let (kind, depletion) = match (&r.single, &r.batch) {
                (Some(s), _) => (ResultKind::Single, Some(s.summary.depleted)),
                (None, Some(b)) => (
                    ResultKind::Batch,
                    Some(b.summary.earliest_depletion.as_ref().map(|(m, _)| *m)),
                ),
                (None, None) => (ResultKind::Scenarios, None),
            };
            let depletion_shares = match &r.batch {
                Some(b) if r.single.is_none() => depletion_ages
                    .iter()
                    .map(|a| {
                        let n = b
                            .stats
                            .depletion_ages
                            .iter()
                            .find(|(x, _)| x == a)
                            .map_or(0, |(_, n)| *n);
                        n as f64 / b.summary.runs as f64
                    })
                    .collect(),
                _ => vec![],
            };
            CompareItem {
                kind,
                digest: digest(r),
                depletion,
                depletion_shares,
                plan_changes: i
                    .plan
                    .as_ref()
                    .map(|p| crate::fieldpath::diff(&first_plan, p))
                    .unwrap_or_default(),
            }
        })
        .collect();
    Comparison {
        items,
        percentiles,
        depletion_ages,
    }
}
