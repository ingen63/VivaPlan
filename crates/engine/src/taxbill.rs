//! #274: Deductions from the last tax bill. The user enters a few numbers of the last final assessment (tax year, total
//! of the income, taxable income of the canton, taxable wealth and optionally the total of the taxes); the engine
//! derives the deductions until the retirement and continues them after it:
//!
//! - **Until the retirement:** deduction = total of the income − taxable income, never negative. It becomes the own
//!   value of the row from the start (prices of the start). The pillar 3a contributions of the plan are deducted
//!   separately («Säule 3a abziehen»), so the row holds the deduction without them (else they would count twice).
//! - **From the retirement:** proposal of the canton without earned income (`retired`) plus the personal rest; rest =
//!   deduction − standard deduction of the canton with earned income (`working`, household with the flat deduction
//!   for professional expenses and the second-earner deduction of every person) − pillar 3a, never negative. The rest
//!   is the additional amount of the row «Vorschlag des Kantons» from the retirement.
//! - **Retired before the start:** one row from the start with the deduction, no rest.
//! - **Taxable wealth:** difference to the taxable wealth of the plan on 31 December of the first year (after the
//!   social deductions estimated from the wealth curve) and a proposal (share of the tax value of the properties or
//!   a deduction from wealth); the user interface takes it over only after a confirmation.
//! - **Check:** with the total of the taxes the own estimate for the same taxable values and the deviation; no
//!   correction factor.
//!
//! The standard deductions come from a run of the plan with constant returns in which the proposal of the canton
//! applies throughout; amounts in prices of the start.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::message::Message;
use crate::model::{DeductionKind, IncomeDeduction, Plan, ReturnSource, TaxMode};
use crate::sim::{MonthRecord, Simulator, WealthTaxYear};
use crate::time::{TimeRef, YearMonth};
use crate::validate::{Issue, Severity};

/// From this deviation of the own estimate from the total of the taxes the dialog names possible causes.
pub const DEVIATION_HINT: f64 = 0.15;
/// Below this difference of the taxable wealth there is no proposal.
pub const WEALTH_TOLERANCE: f64 = 1000.0;

/// Numbers of the tax bill as the dialog sends them; missing values are reported at the field.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TaxBill {
    pub year: Option<i32>,
    /// Total of the income (before the deductions).
    pub total_income: Option<f64>,
    /// Taxable income of the canton.
    pub taxable_income: Option<f64>,
    pub taxable_wealth: Option<f64>,
    /// Optional: total of the taxes (canton, municipality, church and federal tax).
    pub total_tax: Option<f64>,
}

#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaxBillOut {
    /// Errors at the fields of the dialog (`field`: `year`, `totalIncome`, `taxableIncome`, `taxableWealth`,
    /// `totalTax`).
    pub issues: Vec<Issue>,
    /// The plan cannot be computed (for example without a municipality).
    pub error: Option<Message>,
    pub result: Option<TaxBillResult>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaxBillResult {
    pub year: i32,
    /// Deduction of the tax bill: total of the income − taxable income, never negative.
    pub deduction: f64,
    /// Pillar 3a contributions the plan deducts in the year `working_year` (0 without «Säule 3a abziehen»).
    pub pillar3a: f64,
    /// Own value of the row from the start: deduction − pillar 3a, never negative.
    pub own: f64,
    /// Standard deduction of the canton with earned income in the first year of the plan with earned income (or the
    /// first year); `None` if retired before the start.
    pub working: Option<f64>,
    pub working_year: Option<i32>,
    /// Personal rest: deduction − standard deduction with earned income − pillar 3a, never negative.
    pub rest: f64,
    /// Standard deduction of the canton without earned income in the first full year after the retirement.
    pub retired: Option<f64>,
    pub retired_year: Option<i32>,
    /// The household is retired at the start: one row from the start, no rest.
    pub retired_before_start: bool,
    /// Rows of the deductions from income to take over.
    pub deductions: Vec<IncomeDeduction>,
    pub wealth: WealthCheck,
    pub check: Option<TaxCheck>,
}

/// Taxable wealth of the tax bill compared with the plan.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WealthCheck {
    /// Year of the plan's value (31 December of the first year of the plan).
    pub year: i32,
    /// Taxable wealth of the plan after the social deductions of the canton (estimated from the wealth curve).
    pub plan: f64,
    /// Social deductions from wealth taken off the plan's value.
    pub allowance: f64,
    pub bill: f64,
    /// Tax bill − plan.
    pub difference: f64,
    pub proposal: Option<WealthProposal>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub enum WealthProposal {
    /// Tax value of the owned properties as this share of the market value.
    TaxValueRate { rate: f64, properties: Vec<usize> },
    /// Row of the deductions from wealth from the start with this amount per year (prices of the start).
    WealthDeduction { amount: f64 },
}

/// Own estimate for the taxable values of the tax bill compared with its total of the taxes.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TaxCheck {
    /// Income tax (canton, municipality, church; federal tax on the same taxable income) and wealth tax.
    pub estimate: f64,
    pub total: f64,
    /// estimate / total − 1.
    pub deviation: f64,
    /// |deviation| ≥ [`DEVIATION_HINT`].
    pub hint: bool,
}

fn issue(field: &str, message: Message) -> Issue {
    Issue {
        severity: Severity::Error,
        section: "taxes",
        message,
        field: Some(field.to_string()),
    }
}

/// Checks the numbers of the tax bill: required values present, amounts not negative, tax year not after the start.
pub fn check(bill: &TaxBill, start: YearMonth) -> Vec<Issue> {
    let mut out = Vec::new();
    match bill.year {
        None => out.push(issue("year", msg!(tax_bill_missing))),
        Some(y) if y > start.year() => {
            out.push(issue("year", msg!(tax_bill_year_after_start, year = y, start = start)))
        }
        Some(_) => {}
    }
    let amounts = [
        ("totalIncome", bill.total_income, true),
        ("taxableIncome", bill.taxable_income, true),
        ("taxableWealth", bill.taxable_wealth, true),
        ("totalTax", bill.total_tax, false),
    ];
    for (field, v, required) in amounts {
        match v {
            None if required => out.push(issue(field, msg!(tax_bill_missing))),
            Some(x) if x.is_nan() || x < 0.0 => out.push(issue(field, msg!(tax_bill_negative))),
            _ => {}
        }
    }
    out
}

/// Deduction of the tax bill, own value of the row from the start and personal rest (all never negative): deduction
/// = `total` − `taxable`; own value = deduction − `pillar3a` (deducted separately); rest = own value − `working`
/// (standard deduction with earned income; `None`: retired, no rest).
pub fn amounts(total: f64, taxable: f64, pillar3a: f64, working: Option<f64>) -> (f64, f64, f64) {
    let deduction = (total - taxable).max(0.0);
    let own = (deduction - pillar3a).max(0.0);
    let rest = working.map_or(0.0, |w| (own - w).max(0.0));
    (deduction, own, rest)
}

/// Sums of one calendar year of the run.
struct Year<'a> {
    year: i32,
    months: usize,
    earned: f64,
    december: Option<&'a MonthRecord>,
}

fn years(records: &[MonthRecord]) -> Vec<Year<'_>> {
    let mut out: Vec<Year> = Vec::new();
    for r in records {
        if out.last().is_none_or(|y| y.year != r.month.year()) {
            out.push(Year {
                year: r.month.year(),
                months: 0,
                earned: 0.0,
                december: None,
            });
        }
        let y = out.last_mut().expect("pushed");
        y.months += 1;
        y.earned += r.earned;
        if r.month.month() == 12 {
            y.december = Some(r);
        }
    }
    out
}

/// Proposal of the canton of the year (full year, prices of the start) and the deducted pillar 3a contributions.
fn proposal_of(y: &Year, indexed: bool) -> Option<(f64, f64)> {
    let r = y.december?;
    let b = r.tax_basis.as_ref()?;
    let index = if indexed { r.price_index } else { 1.0 };
    let proposal: f64 = b.parts_canton.iter().map(|p| p.1).sum();
    Some((proposal / index, b.pillar3a / index))
}

/// Takes over the tax bill `bill` for the plan `plan` (see the module documentation).
pub fn tax_bill(plan: &Value, bill: &Value) -> TaxBillOut {
    let bill: TaxBill = serde_json::from_value(bill.clone()).unwrap_or_default();
    let plan = match crate::api::parse(plan) {
        Ok(p) => p,
        Err(i) => {
            return TaxBillOut {
                error: Some(i.message),
                ..Default::default()
            }
        }
    };
    compute(&plan, &bill)
}

/// [`tax_bill`] for a parsed plan.
pub fn compute(plan: &Plan, bill: &TaxBill) -> TaxBillOut {
    if plan.taxes.mode != TaxMode::Location {
        return TaxBillOut {
            error: Some(msg!(tax_bill_mode)),
            ..Default::default()
        };
    }
    let mut p = plan.clone();
    p.returns.source = ReturnSource::Fixed;
    p.taxes.deductions = Some(vec![IncomeDeduction {
        from: TimeRef::milestone("start"),
        kind: DeductionKind::Canton,
        amount: 0.0,
        rate: 0.0,
        extra: 0.0,
        tax_bill: None,
    }]);
    let sim = match Simulator::new(&p) {
        Ok(s) => s,
        Err(e) => {
            let mut out = TaxBillOut {
                error: Some(e.message()),
                ..Default::default()
            };
            // the errors at the fields of the dialog nevertheless (the start month is not known)
            out.issues = check(bill, YearMonth::new(i32::MAX, 1));
            return out;
        }
    };
    let start = sim.start;
    let issues = check(bill, start);
    if !issues.is_empty() {
        return TaxBillOut {
            issues,
            ..Default::default()
        };
    }
    let (Some(year), Some(total), Some(taxable), Some(bill_wealth)) =
        (bill.year, bill.total_income, bill.taxable_income, bill.taxable_wealth)
    else {
        unreachable!("checked above");
    };
    let path = crate::market::fixed_path(&sim.fixed, start, sim.months());
    let run = sim.run(&path, true);
    let ys = years(&run.records);
    let indexed = plan.taxes.indexed;
    let retire = sim.retire_at();
    let retired_before_start = retire.is_none_or(|r| r <= start);

    // standard deduction with earned income: first full year with earned income, else the first year (with earned
    // income until the retirement the partial first year)
    let working_year = (!retired_before_start)
        .then(|| ys.iter().find(|y| y.months == 12 && y.earned > 0.0).or(ys.first()))
        .flatten();
    let working = working_year.and_then(|y| proposal_of(y, indexed));
    let pillar3a = working.map_or_else(
        || ys.first().and_then(|y| proposal_of(y, indexed)).map_or(0.0, |w| w.1),
        |w| w.1,
    );
    let (deduction, own, rest) = amounts(total, taxable, pillar3a, working.map(|w| w.0));
    // standard deduction without earned income: first full year from the retirement (as #272/AK3)
    let retired_year = retire.and_then(|at| {
        ys.iter()
            .find(|y| y.months == 12 && YearMonth::new(y.year, 1) >= at && y.earned == 0.0)
    });
    let retired = retired_year.and_then(|y| proposal_of(y, indexed)).map(|w| w.0);

    let row = |from: TimeRef, kind, amount, extra| IncomeDeduction {
        from,
        kind,
        amount,
        rate: 0.0,
        extra,
        tax_bill: Some(year),
    };
    let mut deductions = vec![row(TimeRef::milestone("start"), DeductionKind::Own, own, 0.0)];
    if !retired_before_start {
        let at = retire.expect("not retired before the start");
        deductions.push(row(retire_ref(plan, &sim, at), DeductionKind::Canton, 0.0, rest));
    }

    let residence = sim.residence_of_year(YearMonth::new(start.year(), 12));
    let allowance = residence.map_or(0.0, |r| r.wealth_allowance());
    let wealth = wealth_check(run.wealth_tax.first(), allowance, bill_wealth, start.year());
    let check = bill.total_tax.filter(|t| *t > 0.0).and_then(|t| {
        let r = residence?;
        let estimate = r.income_tax_on(taxable, taxable, 1.0) + r.wealth_tax(bill_wealth + allowance, 1.0);
        let deviation = estimate / t - 1.0;
        Some(TaxCheck {
            estimate,
            total: t,
            deviation,
            hint: deviation.abs() >= DEVIATION_HINT,
        })
    });
    TaxBillOut {
        issues: vec![],
        error: None,
        result: Some(TaxBillResult {
            year,
            deduction,
            pillar3a,
            own,
            working: working.map(|w| w.0),
            working_year: working.and(working_year.map(|y| y.year)),
            rest,
            retired,
            retired_year: retired.and(retired_year.map(|y| y.year)),
            retired_before_start,
            deductions,
            wealth,
            check,
        }),
    }
}

/// Point in time of the row from the retirement: the milestone «Bezug Pensionskasse» of the person who retires last
/// (`retire`, `retire2`) if it is the month `at`, else the month itself.
fn retire_ref(plan: &Plan, sim: &Simulator, at: YearMonth) -> TimeRef {
    let id = sim.last_retiree().map(|i| crate::model::person_milestones(i).0);
    let resolver = crate::resolve::Resolver::new(plan).ok();
    match (id, resolver) {
        (Some(id), Some(r)) if r.milestone(id).ok() == Some(at) => TimeRef::milestone(id),
        _ => TimeRef::date(at),
    }
}

/// Taxable wealth of the plan on 31 December of the first year `w` after the social deductions `allowance`, the
/// difference to the tax bill and the proposal.
fn wealth_check(w: Option<&WealthTaxYear>, allowance: f64, bill: f64, first_year: i32) -> WealthCheck {
    let Some(w) = w else {
        return WealthCheck {
            year: first_year,
            plan: 0.0,
            allowance,
            bill,
            difference: bill,
            proposal: None,
        };
    };
    let plan = (w.taxable - allowance).max(0.0);
    let difference = bill - plan;
    let market: f64 = w.properties.iter().map(|p| p.worth).sum();
    let tax_values: f64 = w.properties.iter().map(|p| p.tax_value).sum();
    let proposal = if difference.abs() < WEALTH_TOLERANCE {
        None
    } else if market > 0.0 {
        let rate = ((tax_values + difference) / market).clamp(0.0, 1.0);
        Some(WealthProposal::TaxValueRate {
            rate: (rate * 100.0).round() / 100.0,
            properties: w.properties.iter().map(|p| p.index).collect(),
        })
    } else if difference < 0.0 {
        Some(WealthProposal::WealthDeduction {
            amount: ((w.ongoing - difference) / 100.0).round() * 100.0,
        })
    } else {
        None
    };
    WealthCheck {
        year: w.year,
        plan,
        allowance,
        bill,
        difference,
        proposal,
    }
}
