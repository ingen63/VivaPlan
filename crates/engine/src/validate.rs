//! Validation of a plan: errors prevent the simulation, warnings do not.

use serde::ser::SerializeMap;
use serde::{Serialize, Serializer};

use crate::message::{arg, Arg, Message};

use crate::model::{fixed_milestone_message, person_milestones, PayoutAt, PayoutUse};
use crate::model::{
    AmortMode, AmountMode, GainModel, MilestoneKind, Plan, PropertyStatus, RateKind, ReturnSource, Series, SeriesDef,
    Strategy, Withdrawal,
};
use crate::resolve::{Curve, Resolver};
use crate::tax;
use crate::time::{TimeRef, YearMonth};

/// Highest allowed age at the end of the simulation.
const MAX_AGE: i32 = 120;

/// #139: Monte Carlo computes at most this many path-months (paths × months of the plan). Measured (18.11.2026): about
/// 23 bytes per path and month (bands nominal and real as `f32`, plus sort columns and results per run); at
/// the limit the heaviest test plan (married couple, four properties, 100 years) needs about 1.3 GB. Above it the
/// app crashed at 1'000'000 paths for lack of memory.
pub const MAX_PATH_MONTHS: u64 = 50_000_000;

/// Longest allowed person name.
const MAX_NAME: usize = 60;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
}

/// Serialized as `{ severity, section, message, code, params, field? }`: `message` is the German text, `code` and
/// `params` let the user interface translate it (#171).
#[derive(Clone, Debug)]
pub struct Issue {
    pub severity: Severity,
    /// Section in the editor (`general`, `milestones`, `pension`, `flows`, `wealth`, `returns`,
    /// `withdrawal`, `optimize`, `taxes`, `realestate`).
    pub section: &'static str,
    pub message: Message,
    /// Path of the affected field in the plan (for example `household.persons.0.name`); the user interface then shows the
    /// message right at the field.
    pub field: Option<String>,
}

impl Serialize for Issue {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let mut m = s.serialize_map(None)?;
        m.serialize_entry("severity", &self.severity)?;
        m.serialize_entry("section", self.section)?;
        m.serialize_entry("message", self.message.text())?;
        m.serialize_entry("code", self.message.code())?;
        m.serialize_entry("params", &self.message.params())?;
        if let Some(f) = &self.field {
            m.serialize_entry("field", f)?;
        }
        m.end()
    }
}

/// #171: A name of the validation (series, point in time) as a parameter of a message.
fn term(m: Message) -> Arg {
    arg(&m)
}

struct Issues(Vec<Issue>);

impl Issues {
    fn err(&mut self, section: &'static str, message: Message) {
        self.0.push(Issue {
            severity: Severity::Error,
            section,
            message,
            field: None,
        });
    }
    /// Error at a specific field of the plan.
    fn err_at(&mut self, section: &'static str, field: impl Into<String>, message: Message) {
        self.0.push(Issue {
            severity: Severity::Error,
            section,
            message,
            field: Some(field.into()),
        });
    }
    /// Warning at a specific field of the plan.
    fn warn_at(&mut self, section: &'static str, field: impl Into<String>, message: Message) {
        self.0.push(Issue {
            severity: Severity::Warning,
            section,
            message,
            field: Some(field.into()),
        });
    }
    fn warn(&mut self, section: &'static str, message: Message) {
        self.0.push(Issue {
            severity: Severity::Warning,
            section,
            message,
            field: None,
        });
    }
}

pub fn validate(plan: &Plan) -> Vec<Issue> {
    let mut v = Issues(vec![]);
    // Take over older pillar 3a on the mortgage as when loading (#52).
    let plan = crate::upgrade::normalized(plan);
    check(&plan, &mut v);
    v.0
}

/// #63/AK8: Mode «Wohnort»: the canton must be in the tax data and the municipality must belong to it.
fn check_tax_location(v: &mut Issues, loc: Option<&crate::model::TaxLocation>) {
    let data = crate::taxdata::builtin();
    let canton = loc.map(|l| l.canton.as_str()).unwrap_or("");
    if canton.is_empty() {
        v.err_at("taxes", "taxes.location.canton", msg!(tax_canton_missing));
        return;
    }
    if data.canton(canton).is_none() {
        v.err_at(
            "taxes",
            "taxes.location.canton",
            msg!(tax_canton_unknown, canton = canton),
        );
        return;
    }
    match loc.and_then(|l| l.municipality) {
        None => v.err_at("taxes", "taxes.location.municipality", msg!(tax_municipality_missing)),
        Some(bfs) if data.municipality(bfs).is_none_or(|m| m.2 != canton) => v.err_at(
            "taxes",
            "taxes.location.municipality",
            msg!(tax_municipality_not_in_canton, bfs = bfs, canton = canton),
        ),
        Some(_) => {}
    }
}

/// Parameters of the guardrails (strategy «Guardrails» and «3-Topf» with guardrails).
fn check_guardrails(v: &mut Issues, w: &Withdrawal) {
    let g = w.guardrails.clone().unwrap_or_default();
    if !(g.initial_rate > 0.0 && g.initial_rate <= 1.0) {
        v.err("withdrawal", msg!(guardrails_initial_rate));
    }
    if g.upper < 0.0 || g.lower < 0.0 || g.lower >= 1.0 {
        v.err("withdrawal", msg!(guardrails_bands));
    }
    if !(0.0..1.0).contains(&g.adjust) {
        v.err("withdrawal", msg!(guardrails_adjust));
    }
}

/// Floor and ceiling of the living costs.
fn check_limits(v: &mut Issues, w: &Withdrawal) {
    if w.floor.is_some_and(|f| f < 0.0) || w.ceiling.is_some_and(|c| c < 0.0) {
        v.err("withdrawal", msg!(spending_limits_negative));
    }
    if let (Some(f), Some(c)) = (w.floor, w.ceiling) {
        if c > 0.0 && f > c {
            v.err("withdrawal", msg!(spending_floor_above_ceiling));
        }
    }
}

fn check(plan: &Plan, v: &mut Issues) {
    if plan.household.persons.is_empty() {
        v.err("general", msg!(persons_none));
        return;
    }
    match (plan.household.persons.len(), &plan.partner) {
        (n, _) if n > 2 => v.err("general", msg!(persons_too_many)),
        (1, Some(_)) => v.err("general", msg!(partner_without_person)),
        (2, None) => v.warn("general", msg!(partner_missing)),
        _ => {}
    }
    for (i, p) in plan.household.persons.iter().enumerate() {
        let len = p.name.trim().chars().count();
        if len == 0 {
            v.err_at(
                "general",
                format!("household.persons.{i}.name"),
                msg!(person_name_missing, n = i + 1),
            );
        } else if len > MAX_NAME {
            v.err_at(
                "general",
                format!("household.persons.{i}.name"),
                msg!(person_name_too_long, n = i + 1, max = MAX_NAME),
            );
        }
    }
    if plan.household.ahv_couple_cap <= 0.0 {
        v.err("pension", msg!(ahv_cap_not_positive));
    }
    let r = match Resolver::new(plan) {
        Ok(r) => r,
        Err(e) => {
            v.err("milestones", e.message());
            return;
        }
    };
    let (start, end) = match (r.milestone("start"), r.milestone("end")) {
        (Ok(s), Ok(e)) => (s, e),
        _ => {
            v.err("milestones", msg!(milestones_start_end_missing));
            return;
        }
    };
    if end <= start {
        v.err("milestones", msg!(end_before_start));
        return;
    }
    if start.until(end) > 12 * 100 {
        v.err("milestones", msg!(period_too_long));
    }
    // #139: Limit the memory of the Monte Carlo computation (paths × months)
    let months = u64::try_from(start.until(end) + 1).unwrap_or(0);
    let mc = match plan.returns.source {
        ReturnSource::Bootstrap => Some((
            "returns.bootstrap.paths",
            plan.returns.bootstrap.clone().unwrap_or_default().paths,
        )),
        ReturnSource::Parametric => Some((
            "returns.parametric.paths",
            plan.returns.parametric.clone().unwrap_or_default().paths,
        )),
        _ => None,
    };
    if let Some((field, paths)) = mc {
        let paths = paths as u64;
        if months > 0 && paths.saturating_mul(months) > MAX_PATH_MONTHS {
            let max = MAX_PATH_MONTHS / months;
            v.err_at(
                "returns",
                field,
                msg!(paths_memory, paths = paths, months = months, max = max),
            );
        }
    }
    for (i, p) in plan.household.persons.iter().enumerate() {
        let who = if p.name.trim().is_empty() {
            arg(&msg!(person_n, n = i + 1))
        } else {
            arg(&p.name)
        };
        if p.birth > start {
            v.err_at(
                "general",
                format!("household.persons.{i}.birth"),
                msg!(birth_after_start, who = who, birth = p.birth, start = start),
            );
        } else if p.birth.until(end) > 12 * MAX_AGE {
            v.err_at(
                "general",
                format!("household.persons.{i}.birth"),
                msg!(person_too_old, who = who, max = MAX_AGE),
            );
        }
    }
    for ms in &plan.milestones {
        if let MilestoneKind::Age { years, .. } = &ms.kind {
            if !(0.0..=130.0).contains(years) {
                v.err(
                    "milestones",
                    msg!(milestone_age_invalid, name = ms.name_arg(), years = years),
                );
            }
        }
        // Start and end themselves never lie before or after.
        let Ok(m) = r.milestone(&ms.id) else { continue };
        if m < start {
            v.warn(
                "milestones",
                msg!(milestone_before_start, name = ms.name_arg(), month = m),
            );
        } else if m > end {
            v.warn("milestones", msg!(milestone_after_end, name = ms.name_arg(), month = m));
        }
    }

    let resolve = |v: &mut Issues, section: &'static str, what: Arg, t: &TimeRef| -> Option<YearMonth> {
        match r.resolve(t) {
            Ok(m) => m,
            Err(e) => {
                v.err(section, msg!(labeled, what = what, problem = e.message()));
                None
            }
        }
    };
    let series = |v: &mut Issues, section: &'static str, what: Arg, s: &Series, range: Option<(f64, f64)>| {
        let values: Vec<f64> = match s {
            Series::Number(x) | Series::Def(SeriesDef::Const { value: x }) => vec![*x],
            Series::Def(SeriesDef::Timed { rows }) => {
                if rows.is_empty() {
                    v.err(section, msg!(series_empty, what = what));
                }
                let mut first: Option<YearMonth> = None;
                for row in rows {
                    if let Some(m) = resolve(v, section, what.clone(), &row.from) {
                        first = Some(first.map_or(m, |f| f.min(m)));
                    }
                }
                if first.is_some_and(|f| f > start) {
                    v.warn(section, msg!(series_late_start, what = what));
                }
                rows.iter().map(|r| r.value).collect()
            }
        };
        if let Some((lo, hi)) = range {
            if values.iter().any(|x| *x < lo || *x > hi) {
                v.err(section, msg!(series_out_of_range, what = what, lo = lo, hi = hi));
            }
        }
    };

    // Pension
    let pk = &plan.pension.pk;
    let retire = resolve(v, "pension", term(msg!(term_pk_payout)), &pk.at);
    resolve(v, "pension", term(msg!(term_ahv_payout)), &plan.pension.ahv.at);
    if !(0.0..=1.0).contains(&pk.lumpsum) {
        v.err("pension", msg!(pk_lumpsum_range));
    }
    if pk.conversion > 0.08 {
        v.warn("pension", msg!(pk_conversion_high, rate = pk.conversion));
    }
    series(v, "pension", term(msg!(term_pk_contribution)), &pk.contribution, None);
    series(
        v,
        "pension",
        term(msg!(term_pk_interest)),
        &pk.interest,
        Some((-0.5, 1.0)),
    );
    let mut retire_dates = vec![retire];
    if let Some(q) = &plan.partner {
        retire_dates.push(resolve(v, "pension", term(msg!(term_pk_payout_2)), &q.pk.at));
        resolve(v, "pension", term(msg!(term_ahv_payout_2)), &q.ahv.at);
        if !(0.0..=1.0).contains(&q.pk.lumpsum) {
            v.err("pension", msg!(pk_lumpsum_range_2));
        }
        if q.pk.conversion > 0.08 {
            v.warn("pension", msg!(pk_conversion_high_2, rate = q.pk.conversion));
        }
        series(
            v,
            "pension",
            term(msg!(term_pk_contribution_2)),
            &q.pk.contribution,
            None,
        );
        series(
            v,
            "pension",
            term(msg!(term_pk_interest_2)),
            &q.pk.interest,
            Some((-0.5, 1.0)),
        );
        series(v, "flows", term(msg!(term_income_2)), &q.income, None);
    }
    let retire_of = [retire_dates[0], retire_dates.get(1).copied().flatten()];
    let pk_lumpsum = [pk.lumpsum, plan.partner.as_ref().map_or(0.0, |q| q.pk.lumpsum)];
    let retire = retire_dates.iter().flatten().max().copied();

    // #51: Fixed milestones per person within the limits from the settings (#25).
    let lim = plan.limits;
    for problem in lim.problems() {
        v.err("general", msg!(settings_problem, problem = problem));
    }
    for (i, p) in plan.household.persons.iter().take(2).enumerate() {
        let (pk_id, ahv_id) = person_milestones(i);
        for (id, lo, hi) in [(pk_id, lim.pk_min, lim.pk_max), (ahv_id, lim.ahv_min, lim.ahv_max)] {
            let Ok(m) = r.milestone(id) else { continue };
            let age = f64::from(p.birth.until(m)) / 12.0;
            if age < lo - 1e-9 || age > hi + 1e-9 {
                let name = fixed_milestone_message(id).map(|m| arg(&m));
                let msg = msg!(
                    milestone_age_limits,
                    name = name,
                    age = (age * 10.0).round() / 10.0,
                    lo = lo,
                    hi = hi
                );
                match plan.milestones.iter().position(|x| x.id == id) {
                    Some(k) => v.err_at("milestones", format!("milestones.{k}.years"), msg),
                    None => v.err("milestones", msg),
                }
            }
        }
        // #191/AK19, AK20: AHV payout by the person's own age (leaflet 3.04: early payout from 63, deferral 1 to 5 years).
        let ahv = if i == 0 {
            Some(&plan.pension.ahv)
        } else {
            plan.partner.as_ref().map(|q| &q.ahv)
        };
        if let Some(at) = ahv.and_then(|a| r.resolve(&a.at).ok().flatten()) {
            let person = crate::ahv::Person::new(p.birth, 0.0, Some(at), false);
            let offset = person.offset();
            let who = if plan.partner.is_some() {
                arg(&msg!(suffix_person, n = i + 1))
            } else {
                crate::message::Arg::Null
            };
            if !(-24..=60).contains(&offset) {
                let age = f64::from(p.birth.until(at)) / 12.0;
                v.warn(
                    "pension",
                    msg!(ahv_payout_age_law, who = who, age = (age * 10.0).round() / 10.0),
                );
            } else if (1..crate::ahv::MIN_DEFERRAL_MONTHS).contains(&offset) {
                v.warn("pension", msg!(ahv_deferral_short, who = who, months = offset));
            }
        }
        // #51: Earned income after the pension fund payout (warning).
        let (income, indexed) = if i == 0 {
            (Some(&plan.flows.income), plan.flows.income_indexed)
        } else {
            (
                plan.partner.as_ref().map(|q| &q.income),
                plan.partner.as_ref().is_some_and(|q| q.income_indexed),
            )
        };
        // #58: Indexed to inflation, the amount is in prices of the start (inflation depends on the path).
        // #171: Indexed and nominal are two messages.
        if let (Some(income), Ok(pk_at)) = (income, r.milestone(pk_id)) {
            if let Ok(c) = Curve::compile(income, &r) {
                if let Some(x) = c.first_above(pk_at.max(start), end, 0.005) {
                    let text = if indexed {
                        msg!(income_after_pk_indexed, at = pk_at, amount = x)
                    } else {
                        msg!(income_after_pk, at = pk_at, amount = x)
                    };
                    v.warn(
                        "flows",
                        if plan.household.persons.len() > 1 {
                            msg!(labeled, what = p.name, problem = text)
                        } else {
                            text
                        },
                    );
                }
            }
        }
    }

    // Cash flows and wealth
    series(v, "flows", term(msg!(term_income)), &plan.flows.income, None);
    series(v, "flows", term(msg!(term_spending)), &plan.flows.spending, None);
    // #53: Living costs include housing costs; every row must contain at least the housing share.
    if plan.flows.spending_includes_housing {
        let w = crate::housing::start_housing(plan, &r);
        let check = |v: &mut Issues, field: String, x: f64| {
            if x + 0.005 < w {
                v.err_at("flows", field, msg!(spending_below_housing, spending = x, housing = w));
            }
        };
        match &plan.flows.spending {
            Series::Number(x) | Series::Def(SeriesDef::Const { value: x }) => {
                check(v, "flows.spending.value".into(), *x)
            }
            Series::Def(SeriesDef::Timed { rows }) => {
                for (i, row) in rows.iter().enumerate() {
                    check(v, format!("flows.spending.rows.{i}.value"), row.value);
                }
            }
        }
    }
    if let Some(s) = &plan.flows.savings {
        series(v, "flows", term(msg!(term_savings)), s, None);
    }
    series(
        v,
        "wealth",
        term(msg!(term_stock_share)),
        &plan.wealth.stock_share,
        Some((0.0, 1.0)),
    );
    series(
        v,
        "wealth",
        term(msg!(term_reserve)),
        &plan.wealth.reserve_months,
        Some((0.0, 1200.0)),
    );
    for x in &plan.flows.extras {
        if x.date > end {
            v.warn("flows", msg!(extra_after_end, label = x.label));
        }
    }

    // Returns
    let ret = &plan.returns;
    series(
        v,
        "returns",
        term(msg!(term_stock_return)),
        &ret.fixed.stocks,
        Some((-0.99, 10.0)),
    );
    series(
        v,
        "returns",
        term(msg!(term_bond_return)),
        &ret.fixed.bonds,
        Some((-0.99, 10.0)),
    );
    series(
        v,
        "returns",
        term(msg!(term_inflation)),
        &ret.fixed.inflation,
        Some((-0.99, 10.0)),
    );
    series(
        v,
        "returns",
        term(msg!(term_saron)),
        &ret.fixed.saron,
        Some((-0.99, 10.0)),
    );
    // #69: Weights of «Welt individuell»
    if let Some(st) = ret
        .strategy
        .as_ref()
        .filter(|s| s.stocks == crate::model::StockIndex::WorldCustom)
    {
        if let Some(e) = crate::strategy::weight_issue(&st.weights) {
            v.err_at("wealth", "returns.strategy.weights", e);
        }
    }
    match ret.source {
        ReturnSource::Fixed => {}
        ReturnSource::Historical | ReturnSource::Rolling => {
            if ret.series.is_none() && ret.strategy.is_none() {
                v.err("returns", msg!(historical_series_missing));
            }
            if ret.source == ReturnSource::Historical && ret.start_date.is_none() {
                v.err("returns", msg!(historical_start_missing));
            }
        }
        ReturnSource::Bootstrap => {
            if ret.series.is_none() && ret.strategy.is_none() {
                v.err("returns", msg!(bootstrap_series_missing));
            }
            let b = ret.bootstrap.clone().unwrap_or_default();
            if b.paths == 0 || b.paths > 1_000_000 {
                v.err("returns", msg!(paths_range));
            }
            if b.block < 1.0 {
                v.err("returns", msg!(block_length_min));
            }
        }
        ReturnSource::Parametric => {
            let q = ret.parametric.clone().unwrap_or_default();
            if q.paths == 0 || q.paths > 1_000_000 {
                v.err("returns", msg!(paths_range));
            }
            if q.sigma < 0.0 || q.jump_sigma < 0.0 || q.jump_lambda < 0.0 {
                v.err("returns", msg!(volatility_negative));
            }
            if q.df != 0.0 && q.df <= 2.0 {
                v.err("returns", msg!(degrees_of_freedom));
            }
        }
    }
    match plan.withdrawal.strategy {
        Strategy::Standard => {}
        Strategy::ThreeBuckets => {
            let b = plan.withdrawal.buckets.clone().unwrap_or_default();
            if b.cash_years < 0.0 || b.bond_years < 0.0 {
                v.err("withdrawal", msg!(buckets_negative));
            }
            if b.cash_years + b.bond_years > 15.0 {
                v.warn("withdrawal", msg!(buckets_too_large));
            }
            if retire.is_none() {
                v.warn("withdrawal", msg!(buckets_without_retirement));
            }
            // #6: If the guardrails steer spending, their checks and those of the limits apply.
            if plan.withdrawal.buckets_with_guardrails() {
                check_guardrails(v, &plan.withdrawal);
                check_limits(v, &plan.withdrawal);
            }
        }
        Strategy::FixedRate | Strategy::Guardrails | Strategy::Vpw | Strategy::Endowment => {
            let w = &plan.withdrawal;
            let rate_ok = |r: f64| r > 0.0 && r <= 1.0;
            if plan.withdrawal.strategy == Strategy::FixedRate {
                let r = w.fixed_rate.clone().unwrap_or_default().rate;
                if !rate_ok(r) {
                    v.err("withdrawal", msg!(withdrawal_rate_range));
                } else if r > 0.07 {
                    v.warn("withdrawal", msg!(withdrawal_rate_high, rate = r));
                }
            } else if plan.withdrawal.strategy == Strategy::Vpw {
                let x = w.vpw.clone().unwrap_or_default();
                if x.horizon_age <= 0.0 || x.real_return <= -1.0 {
                    v.err("withdrawal", msg!(vpw_invalid));
                }
            } else if plan.withdrawal.strategy == Strategy::Endowment {
                let x = w.endowment.clone().unwrap_or_default();
                if !rate_ok(x.rate) || !(0.0..=1.0).contains(&x.smoothing) {
                    v.err("withdrawal", msg!(endowment_invalid));
                }
            } else {
                check_guardrails(v, w);
            }
            check_limits(v, w);
            if retire.is_none() {
                v.warn("withdrawal", msg!(withdrawal_without_retirement));
            }
        }
    }

    // Scenarios
    match crate::scenarios::parse(&plan.scenarios.sweeps)
        .and_then(|sw| crate::scenarios::count_combinations(&sw).map(|_| sw))
    {
        Err(e) => v.err("scenarios", e.message()),
        Ok(sw) => {
            if sw.iter().any(|s| s.param == "wealth.stockShare") && plan.withdrawal.strategy == Strategy::ThreeBuckets {
                v.warn("scenarios", msg!(scenario_stock_share_buckets));
            }
            if sw.iter().any(|s| s.param == "returns.startDate") && ret.source != ReturnSource::Historical {
                v.warn("scenarios", msg!(scenario_start_date_source));
            }
        }
    }

    // Saved settings of the optimization
    if let Some(o) = &plan.optimization {
        if !(0.0..=1.0).contains(&o.max_ruin) {
            v.warn("optimize", msg!(optimize_max_ruin));
        }
        if o.paths == 0 || o.check_paths == 0 {
            v.warn("optimize", msg!(optimize_paths));
        }
        for s in crate::optimize::all_strategies() {
            if let Err(e) = crate::optimize::split_defs(s, &o.ranges) {
                v.warn("optimize", msg!(optimize_problem, problem = e.message()));
            }
        }
    }

    // #98: Tax multiplier and rate tables (income, wealth, lump sum, also married). Without the check a
    // rate of −10¹⁵ produced invalid numbers in the simulation.
    let t = &plan.taxes;
    if !(0.0..=10.0).contains(&t.multiplier) {
        v.err_at("taxes", "taxes.multiplier", msg!(tax_multiplier_range));
    }
    /// Key in the path, name in the message, brackets [(amount, rate)].
    type Table<'a> = (&'a str, Message, &'a [(f64, f64)]);
    // #63, #64: In the mode «Wohnort» the own tables for income, wealth and lump sums do not apply (hidden, kept in
    // the file).
    let own = t.mode.is_own();
    let mut tables: Vec<Table> = Vec::new();
    if own {
        tables.push(("capital", msg!(term_capital_tax), &t.capital));
        tables.push(("income", msg!(term_income_tax), &t.income));
        tables.push(("wealth", msg!(term_wealth_tax), &t.wealth));
        if let Some(c) = &t.couple {
            tables.push(("couple.income", msg!(term_income_tax_married), &c.income));
            tables.push(("couple.wealth", msg!(term_wealth_tax_married), &c.wealth));
            tables.push(("couple.capital", msg!(term_capital_tax_married), &c.capital));
        }
    }
    if !own {
        check_tax_location(v, t.location.as_ref());
    }
    for (key, name, rows) in tables {
        // #134: The tax (amount × rate) must not fall with a rising amount; only a warning, the computation runs.
        if rows
            .iter()
            .all(|&(from, rate)| from.is_finite() && from >= 0.0 && (0.0..=1.0).contains(&rate))
        {
            if let Some(i) = tax::falling_row(rows) {
                v.warn_at(
                    "taxes",
                    format!("taxes.{key}.{i}.1"),
                    msg!(tax_rate_falling, table = name, row = i + 1),
                );
            }
        }
        for (i, &(from, rate)) in rows.iter().enumerate() {
            if !(from.is_finite() && from >= 0.0) {
                v.err_at(
                    "taxes",
                    format!("taxes.{key}.{i}.0"),
                    msg!(tax_amount_negative, table = name, row = i + 1),
                );
            }
            if !(0.0..=1.0).contains(&rate) {
                v.err_at(
                    "taxes",
                    format!("taxes.{key}.{i}.1"),
                    msg!(tax_rate_range, table = name, row = i + 1),
                );
            }
        }
    }

    // The property gain tax is required as soon as a property with status «Besitz» or «Kauf geplant» occurs
    // (rented properties do not trigger it). Rules with a rate of 0 % count as set.
    if plan.taxes.property_gain.is_none()
        && plan
            .realestate
            .properties
            .iter()
            .any(|p| p.status != PropertyStatus::Rent)
    {
        v.err("taxes", msg!(gain_tax_missing));
    }

    // Property gain tax: surcharges and reductions by holding period need the purchase date
    if let Some(g) = &plan.taxes.property_gain {
        if g.duration
            .iter()
            .any(|&(y, c)| !y.is_finite() || y < 0.0 || !c.is_finite() || c <= -1.0)
        {
            v.err("taxes", msg!(gain_tax_duration_invalid));
        }
        if g.model == GainModel::Aargau {
            if g.duration_rates.is_empty() {
                v.err("taxes", msg!(gain_tax_aargau_empty));
            }
            if g.duration_rates
                .iter()
                .any(|&(y, r)| !y.is_finite() || y < 0.0 || !r.is_finite() || !(0.0..=1.0).contains(&r))
            {
                v.err("taxes", msg!(gain_tax_aargau_invalid));
            }
        }
        if !g.duration.is_empty() || (g.model == GainModel::Aargau && !g.duration_rates.is_empty()) {
            for p in plan
                .realestate
                .properties
                .iter()
                .filter(|p| p.status == PropertyStatus::Owned && p.bought.is_none())
            {
                v.warn("realestate", msg!(property_bought_missing, name = p.name));
            }
        }
    }

    // Properties
    let mut indirect_per_person = [0.0f64; 2];
    series(
        v,
        "realestate",
        term(msg!(term_emergency_months)),
        &plan.realestate.emergency_months,
        None,
    );
    // Check references «nach Verkauf von …».
    let mut ids = std::collections::HashSet::new();
    for p in &plan.realestate.properties {
        if let Some(id) = &p.id {
            if !ids.insert(id.as_str()) {
                v.err("realestate", msg!(property_id_duplicate, name = p.name, id = id));
            }
        }
    }
    for p in &plan.realestate.properties {
        for t in [&p.buy, &p.from].into_iter().flatten() {
            let Some(id) = t.sale_of() else { continue };
            match plan.realestate.properties.iter().find(|q| q.id.as_deref() == Some(id)) {
                None => v.err("realestate", msg!(property_after_sale_missing, name = p.name)),
                Some(q) if std::ptr::eq(q, p) => v.err("realestate", msg!(property_after_own_sale, name = p.name)),
                Some(q) if q.status == PropertyStatus::Rent => v.err(
                    "realestate",
                    msg!(property_after_rent_sale, name = p.name, other = q.name),
                ),
                _ => {}
            }
        }
    }
    // #196: With «Immobilien nie verkaufen» planned sales and purchases or rents «nach Verkauf» are not carried out.
    if plan.realestate.never_sell {
        for (i, p) in plan.realestate.properties.iter().enumerate() {
            let sells = p.status != PropertyStatus::Rent && p.sell.as_ref().is_some_and(|t| !t.is_never());
            if sells {
                let field = format!("realestate.properties.{i}.sell");
                v.warn_at("realestate", field, msg!(property_never_sell_sale, name = p.name));
            }
            let trigger = match p.status {
                PropertyStatus::Planned => p.buy.as_ref(),
                PropertyStatus::Rent => p.from.as_ref(),
                PropertyStatus::Owned => None,
            };
            if trigger.is_some_and(|t| t.is_dynamic()) {
                v.warn("realestate", msg!(property_never_sell_after_sale, name = p.name));
            }
        }
    }
    for (i, p) in plan.realestate.properties.iter().enumerate() {
        let n = &p.name;
        let buy = p.buy.as_ref().and_then(|t| resolve(v, "realestate", arg(n), t));
        if let Some(t) = &p.sell {
            let sell = resolve(v, "realestate", arg(n), t);
            if sell.is_some_and(|sell| sell < start) {
                v.warn("realestate", msg!(property_sale_before_start, name = n));
            }
            // The sale must come after the purchase (owned: purchase date, purchase planned: «Kauf ab»). A purchase «nach Verkauf von …»
            // cannot be determined at entry (`buy` is then missing); the check is skipped there.
            let bought = match p.status {
                PropertyStatus::Owned => p.bought,
                PropertyStatus::Planned => buy,
                PropertyStatus::Rent => None,
            };
            if let (Some(sell), Some(bought)) = (sell, bought) {
                if sell <= bought {
                    let text = if p.status == PropertyStatus::Owned {
                        msg!(property_sale_before_bought, name = n)
                    } else {
                        msg!(property_sale_before_buy, name = n)
                    };
                    v.err_at("realestate", format!("realestate.properties.{i}.sell"), text);
                }
            }
        }
        match p.status {
            PropertyStatus::Rent => {
                match &p.from {
                    Some(t) => {
                        resolve(v, "realestate", arg(n), t);
                    }
                    None => v.err("realestate", msg!(property_rent_from_missing, name = n)),
                }
                if let Some(t) = &p.until {
                    resolve(v, "realestate", arg(n), t);
                }
                if p.rent.is_none() {
                    v.err("realestate", msg!(property_rent_missing, name = n));
                }
                continue;
            }
            PropertyStatus::Planned if buy.is_some_and(|b| b < start) => {
                v.warn("realestate", msg!(property_buy_before_start, name = n));
            }
            PropertyStatus::Planned if buy.is_none() && !p.buy.as_ref().is_some_and(TimeRef::is_dynamic) => {
                v.warn("realestate", msg!(property_buy_missing, name = n));
            }
            _ => {}
        }
        let Some(mg) = &p.mortgage else { continue };
        let total: f64 = mg.tranches.iter().map(|t| t.amount).sum();
        let max_loan = mg.amount_mode == AmountMode::MaxLoan;
        if max_loan && p.status != PropertyStatus::Planned {
            v.warn("realestate", msg!(mortgage_max_loan_not_planned, name = n));
        }
        // With «80 % des Kaufpreises» the amounts are only shares and are not compared with the price.
        if !max_loan && p.status == PropertyStatus::Planned {
            if total > p.price {
                v.err("realestate", msg!(mortgage_above_price, name = n));
            }
            if total > p.price * 0.8 {
                v.warn("realestate", msg!(mortgage_above_80, name = n));
            }
        }
        for t in &mg.tranches {
            let tn = if mg.tranches.len() > 1 {
                format!("{n}, {}", t.name)
            } else {
                n.clone()
            };
            if t.periods.is_empty() {
                v.err("realestate", msg!(tranche_no_period, name = tn));
            }
            if p.status == PropertyStatus::Owned {
                match t.start {
                    None => v.err("realestate", msg!(tranche_start_missing, name = tn)),
                    Some(s) if s > start => v.warn("realestate", msg!(tranche_start_after_start, name = tn)),
                    _ => {}
                }
            }
            let mut m = if p.status == PropertyStatus::Owned {
                t.start
            } else {
                buy
            };
            // #52: Account of the indirect amortization; #50: indirect only until before this person's AHV payout.
            let account = match &t.pillar3a {
                None => Some(0),
                Some(id) => {
                    let i = plan.household.persons.iter().position(|q| &q.id == id);
                    if i.is_none() {
                        v.err("realestate", msg!(tranche_pillar3a_person_missing, name = tn, id = id));
                    }
                    i
                }
            };
            let ahv_of = account.and_then(|i| r.milestone(person_milestones(i).1).ok());
            let mut max_q: f64 = 0.0;
            for (k, d) in t.periods.iter().enumerate() {
                if d.term > 100.0 {
                    v.err("realestate", msg!(period_term_too_long, name = tn, period = k + 1));
                }
                if d.term <= 0.0 {
                    v.err("realestate", msg!(period_term_not_positive, name = tn, period = k + 1));
                }
                if d.kind == RateKind::Fixed && d.rate > 0.1 {
                    v.warn(
                        "realestate",
                        msg!(period_rate_high, name = tn, period = k + 1, rate = d.rate),
                    );
                }
                if d.kind == RateKind::Saron && d.margin > 0.03 {
                    v.warn(
                        "realestate",
                        msg!(period_margin_high, name = tn, period = k + 1, margin = d.margin),
                    );
                }
                let period_end = m.map(|s| s.plus(((d.term * 12.0).round() as i32).max(1)));
                if d.amort_mode == AmortMode::Indirect
                    && d.amort > 0.0
                    && period_end.is_some_and(|e| e > start)
                    && !(m.is_some() && ahv_of.is_some() && m >= ahv_of)
                {
                    max_q = max_q.max(d.amort);
                }
                m = period_end;
            }
            if let Some(i) = account {
                indirect_per_person[i.min(1)] += max_q * 4.0;
            }
        }
    }

    // Pillar 3a (#52): accounts, contribution, payout plan; maximum amount together with the indirect amortization.
    let lim = plan.limits;
    for (k, acc) in plan.pillar3a.iter().enumerate() {
        let f = |name: &str| format!("pillar3a.{k}.{name}");
        let Some(i) = plan.household.persons.iter().position(|q| q.id == acc.person) else {
            v.err("pillar3a", msg!(pillar3a_person_missing, person = acc.person));
            continue;
        };
        let who = if plan.household.persons.len() > 1 {
            msg!(pillar3a_of, name = plan.household.persons[i].name)
        } else {
            msg!(pillar3a)
        };
        if plan.pillar3a[..k].iter().any(|a| a.person == acc.person) {
            v.err("pillar3a", msg!(pillar3a_duplicate, who = who));
        }
        if acc.balance < 0.0 {
            v.err_at("pillar3a", f("balance"), msg!(pillar3a_balance_negative, who = who));
        }
        if !(-0.5..=1.0).contains(&acc.rate) {
            v.err_at("pillar3a", f("rate"), msg!(pillar3a_rate_range, who = who));
        }
        if !(acc.contribution >= 0.0 && acc.contribution <= lim.pillar3a_max) {
            v.err_at(
                "pillar3a",
                f("contribution"),
                msg!(pillar3a_contribution_range, who = who, max = lim.pillar3a_max),
            );
        }
        let ahv = r.milestone(person_milestones(i).1).ok();
        let indirect = indirect_per_person[i.min(1)];
        if acc.contribution + indirect > lim.pillar3a_max + 0.005 && lim.pillar3a_max >= 0.0 {
            v.warn(
                "pillar3a",
                msg!(
                    pillar3a_over_max,
                    who = who,
                    contribution = acc.contribution,
                    indirect = indirect,
                    max = lim.pillar3a_max
                ),
            );
        }
        let repay_possible = indirect > 0.0;
        // Without a tranche with indirect amortization there is nothing to repay: the payout goes into wealth.
        if acc.usage == PayoutUse::Repay && !repay_possible {
            v.warn("pillar3a", msg!(pillar3a_repay_without_indirect, who = who));
        }
        for (j, x) in acc.payouts.iter().enumerate() {
            // #55: Point in time as a date or age; the message is at the respective field.
            let at_field = match x.kind {
                PayoutAt::Date => format!("pillar3a.{k}.payouts.{j}.date"),
                PayoutAt::Age => format!("pillar3a.{k}.payouts.{j}.years"),
            };
            let Some(date) = x.month(Some(plan.household.persons[i].birth)) else {
                let text = match x.kind {
                    PayoutAt::Date => msg!(pillar3a_payout_date_invalid, who = who, n = j + 1),
                    PayoutAt::Age => msg!(pillar3a_payout_age_invalid, who = who, n = j + 1),
                };
                v.err_at("pillar3a", at_field, text);
                continue;
            };
            if let Some(a) = ahv {
                let earliest = a.plus(-60);
                if date < earliest {
                    v.err_at(
                        "pillar3a",
                        at_field,
                        msg!(
                            pillar3a_payout_too_early,
                            who = who,
                            date = date,
                            ahv = a,
                            earliest = earliest
                        ),
                    );
                } else if date > a {
                    v.warn(
                        "pillar3a",
                        msg!(pillar3a_payout_after_ahv, who = who, date = date, ahv = a),
                    );
                }
            }
            if x.amount.is_some_and(|b| b.is_nan() || b <= 0.0) {
                v.err_at(
                    "pillar3a",
                    format!("pillar3a.{k}.payouts.{j}.amount"),
                    msg!(pillar3a_payout_amount, who = who, date = date),
                );
            }
            if x.usage == PayoutUse::Repay && !repay_possible {
                v.warn(
                    "pillar3a",
                    msg!(pillar3a_payout_repay_without_indirect, who = who, date = date),
                );
            }
            if let (Some(pm), true) = (retire_of[i.min(1)], pk_lumpsum[i.min(1)] > 0.0) {
                if pm.year() == date.year() {
                    v.warn(
                        "pillar3a",
                        msg!(pillar3a_payout_same_year_as_pk, who = who, date = date),
                    );
                }
            }
        }
    }
    // Indirect amortization without an entered 3a account: check the maximum amount anyway.
    for (i, p) in plan.household.persons.iter().take(2).enumerate() {
        if plan.pillar3a.iter().any(|a| a.person == p.id) {
            continue;
        }
        let indirect = indirect_per_person[i];
        if indirect > lim.pillar3a_max + 0.005 {
            let who = if plan.household.persons.len() > 1 {
                msg!(pillar3a_of, name = p.name)
            } else {
                msg!(pillar3a)
            };
            v.warn(
                "pillar3a",
                msg!(
                    pillar3a_over_max,
                    who = who,
                    contribution = 0,
                    indirect = indirect,
                    max = lim.pillar3a_max
                ),
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};

    /// #171/AK1: an issue carries the German text, the code and the parameters as an object.
    #[test]
    fn issue_171_ak1_issue_serializes_code_and_params() {
        let i = Issue {
            severity: Severity::Error,
            section: "general",
            message: msg!(person_name_missing, n = 2),
            field: Some("household.persons.1.name".into()),
        };
        assert_eq!(
            serde_json::to_value(&i).unwrap(),
            json!({ "severity": "error", "section": "general", "message": "Person 2: the name is missing (required).",
                "code": "person_name_missing", "params": { "n": 2 }, "field": "household.persons.1.name" })
        );
    }

    // Example plan: start 2026-11, birth 1966-11, retirement and AHV 2031-11, end 2061-11.
    fn base() -> Value {
        serde_json::from_str(include_str!("../assets/new.vplan")).unwrap()
    }
    fn messages(v: &Value) -> Vec<String> {
        let plan: Plan = serde_json::from_value(v.clone()).unwrap();
        validate(&plan).into_iter().map(|i| i.message.to_string()).collect()
    }
    /// Whether a message contains the text.
    fn has(v: &Value, text: &str) -> bool {
        messages(v).iter().any(|m| m.contains(text))
    }
    fn with(f: impl FnOnce(&mut Value)) -> Value {
        let mut v = base();
        f(&mut v);
        v
    }
    fn set_end(v: &mut Value, date: &str) {
        v["milestones"][3] =
            json!({"id": "end", "kind": "date", "name": "Simulationsende", "date": date, "fixed": true});
    }

    // #40: Limit values of the checks (the limit value itself is allowed in each case).
    #[test]
    fn period_birth_and_milestones() {
        assert!(messages(&base()).is_empty(), "{:?}", messages(&base()));
        const LONG: &str = "longer than 100 years";
        assert!(!has(&with(|v| set_end(v, "2126-11")), LONG));
        assert!(has(&with(|v| set_end(v, "2126-12")), LONG));
        // Birth in the start month is possible
        assert!(!has(
            &with(|v| v["household"]["persons"][0]["birth"] = json!("2026-11")),
            "after the simulation start"
        ));
        assert!(has(
            &with(|v| v["household"]["persons"][0]["birth"] = json!("2026-12")),
            "after the simulation start"
        ));
        // exactly 120 years at the end
        const OLD: &str = "over 120 years old";
        assert!(!has(&with(|v| set_end(v, "2086-11")), OLD));
        assert!(has(&with(|v| set_end(v, "2086-12")), OLD));
        // Milestones before the start or after the end; the end itself is allowed
        let ms =
            |d: &str| with(|v| v["milestones"][2] = json!({"id": "ahv", "kind": "date", "name": "AHV", "date": d}));
        assert!(has(&ms("2026-10"), "is before the simulation start"));
        assert!(!has(&ms("2061-11"), "is after the simulation end"));
        assert!(has(&ms("2061-12"), "is after the simulation end"));
    }

    #[test]
    fn pension_series_and_extras() {
        const HIGH: &str = "Conversion rate of 9.00 %";
        assert!(!has(
            &with(|v| v["pension"]["pk"]["conversion"] = json!(0.08)),
            "unusually high"
        ));
        assert!(has(&with(|v| v["pension"]["pk"]["conversion"] = json!(0.09)), HIGH));
        let partner = |c: f64| {
            with(|v| {
                v["household"]["persons"] =
                    json!([{"id": "A", "name": "A", "birth": "1966-11"}, {"id": "B", "name": "B", "birth": "1968-01"}]);
                v["partner"] = json!({"income": 1000, "pk": {"capital": 0, "at": "retire", "lumpsum": 0, "conversion": c},
                                      "ahv": {"monthly": 0, "at": "ahv", "indexed": true}});
            })
        };
        assert!(!has(&partner(0.08), "unusually high"));
        assert!(has(&partner(0.09), "Person 2: conversion rate of 9.00 %"));
        // Series only after the start: warning; from the start: none
        const LATE: &str = "no value from the simulation start";
        let income = |from: &str| {
            with(|v| v["flows"]["income"] = json!({"mode": "timed", "rows": [{"from": from, "value": 1}]}))
        };
        assert!(has(&income("2027-01"), LATE));
        assert!(!has(&income("2026-11"), LATE));
        assert!(!has(&income("2020-01"), LATE));
        let extra = |d: &str| with(|v| v["flows"]["extras"] = json!([{"date": d, "amount": 1000, "label": "Erbe"}]));
        assert!(!has(&extra("2061-11"), "“Erbe” is after"));
        assert!(has(&extra("2061-12"), "“Erbe” is after"));
    }

    #[test]
    fn return_sources() {
        let boot = |paths: u64, block: f64| {
            with(|v| {
                v["returns"]["source"] = json!("bootstrap");
                v["returns"]["series"] = json!("x.csv");
                v["returns"]["bootstrap"] = json!({"paths": paths, "block": block, "seed": 1});
            })
        };
        // The range 1 to 1'000'000 applies; that 1'000'000 paths need too much memory for this plan duration
        // is a separate message (#139).
        assert!(
            messages(&boot(1_000_000, 1.0))
                .iter()
                .all(|m| m.starts_with("Too many paths")),
            "{:?}",
            messages(&boot(1_000_000, 1.0))
        );
        assert!(has(&boot(1_000_001, 1.0), "number of paths"));
        assert!(has(&boot(10, 0.9), "block length"));
        let para = |sigma: f64, js: f64, jl: f64| {
            with(|v| {
                v["returns"]["source"] = json!("parametric");
                v["returns"]["parametric"] =
                    json!({"paths": 10, "mu": 0.05, "sigma": sigma, "df": 0, "jumpSigma": js, "jumpLambda": jl});
            })
        };
        const NEG: &str = "must not be negative";
        assert!(!has(&para(0.0, 0.0, 0.0), NEG));
        assert!(has(&para(-0.1, 0.0, 0.0), NEG));
        assert!(has(&para(0.1, -0.1, 0.0), NEG));
        assert!(has(&para(0.1, 0.0, -0.1), NEG));
    }

    #[test]
    fn withdrawal_strategies() {
        let w = |strategy: &str, f: &dyn Fn(&mut Value)| {
            with(|v| {
                v["withdrawal"]["strategy"] = json!(strategy);
                f(&mut v["withdrawal"]);
            })
        };
        const MANY: &str = "more than 15 years";
        assert!(!has(
            &w("three_buckets", &|x| x["buckets"] =
                json!({"cashYears": 10, "bondYears": 5})),
            MANY
        ));
        assert!(has(
            &w("three_buckets", &|x| x["buckets"] =
                json!({"cashYears": 10, "bondYears": 5.5})),
            MANY
        ));
        assert!(!has(
            &w("fixed_rate", &|x| x["fixedRate"] = json!({"rate": 0.07})),
            "is high"
        ));
        assert!(has(
            &w("fixed_rate", &|x| x["fixedRate"] = json!({"rate": 0.08})),
            "withdrawal rate of 8.0 % is high"
        ));
        const VPW: &str = "VPW:";
        assert!(!has(&w("vpw", &|_| {}), VPW));
        assert!(!has(&w("vpw", &|x| x["vpw"]["realReturn"] = json!(0.5)), VPW));
        assert!(has(&w("vpw", &|x| x["vpw"]["realReturn"] = json!(-1.0)), VPW));
        assert!(has(&w("vpw", &|x| x["vpw"]["horizonAge"] = json!(0)), VPW));
        const END: &str = "Endowment:";
        assert!(!has(&w("endowment", &|_| {}), END));
        assert!(has(&w("endowment", &|x| x["endowment"]["smoothing"] = json!(1.5)), END));
        assert!(has(&w("endowment", &|x| x["endowment"]["rate"] = json!(0)), END));
        const RAILS: &str = "guardrails must be";
        let g = |upper: f64, lower: f64| {
            w("guardrails", &move |x| {
                x["guardrails"]["upper"] = json!(upper);
                x["guardrails"]["lower"] = json!(lower);
            })
        };
        assert!(!has(&g(0.0, 0.0), RAILS));
        assert!(has(&g(-0.1, 0.2), RAILS));
        assert!(has(&g(0.2, -0.1), RAILS));
        assert!(has(&g(0.2, 1.0), RAILS));
        assert!(!has(&g(0.2, 0.99), RAILS));
        const ABOVE: &str = "spending floor is above";
        let fc = |floor: f64, ceiling: f64| {
            w("fixed_rate", &move |x| {
                x["floor"] = json!(floor);
                x["ceiling"] = json!(ceiling);
            })
        };
        assert!(has(&fc(5000.0, 4000.0), ABOVE));
        assert!(!has(&fc(4000.0, 4000.0), ABOVE));
        assert!(!has(&fc(5000.0, 0.0), ABOVE));
    }

    #[test]
    fn scenarios_and_optimization() {
        const START: &str = "only has an effect with the source “Historical";
        let sweep = |source: &str| {
            with(|v| {
                v["returns"]["source"] = json!(source);
                v["returns"]["series"] = json!("x.csv");
                v["returns"]["startDate"] = json!("1990-01");
                v["scenarios"]["sweeps"] =
                    json!([{"param": "returns.startDate", "from": "1990-01", "to": "1991-01", "step": 12}]);
            })
        };
        assert!(has(&sweep("rolling"), START));
        assert!(!has(&sweep("historical"), START));
        const PATHS: &str = "number of paths must be greater than 0";
        let opt = |paths: u64, check: u64| with(|v| v["optimization"] = json!({"paths": paths, "checkPaths": check}));
        assert!(!has(&opt(100, 100), PATHS));
        assert!(has(&opt(0, 100), PATHS));
        assert!(has(&opt(100, 0), PATHS));
    }

    fn house(status: &str, extra: Value) -> Value {
        let mut p =
            json!({"id": "h", "name": "Haus", "status": status, "price": 1_000_000, "fixCosts": 0, "inflation": false});
        for (k, x) in extra.as_object().unwrap() {
            p[k] = x.clone();
        }
        p
    }
    fn with_props(props: Vec<Value>) -> Value {
        with(|v| {
            v["realestate"]["properties"] = Value::Array(props);
            v["taxes"]["propertyGain"] = json!({"brackets": [[0, 0.2]]});
        })
    }
    fn tranche(amount: f64, start: Option<&str>, periods: Value) -> Value {
        let mut t = json!({"name": "T1", "amount": amount, "periods": periods});
        if let Some(s) = start {
            t["start"] = json!(s);
        }
        t
    }

    #[test]
    fn property_gain_tax_rules() {
        let aargau = |rates: Value, model: &str| {
            with(|v| {
                v["realestate"]["properties"] = json!([house("owned", json!({"sell": "never", "worth": 1_000_000}))]);
                v["taxes"]["propertyGain"] = json!({"model": model, "durationRates": rates, "brackets": [[0, 0.2]]});
            })
        };
        assert!(has(
            &aargau(json!([[0, 1.5]]), "aargau"),
            "holding period and tax rate are invalid"
        ));
        assert!(has(
            &aargau(json!([[-1, 0.2]]), "aargau"),
            "holding period and tax rate are invalid"
        ));
        assert!(!has(&aargau(json!([[0, 0.2]]), "aargau"), "are invalid"));
        // Purchase date missing: warning only in the Aargau model with rates (or with surcharges/reductions)
        const BOUGHT: &str = "the purchase date is missing";
        assert!(has(&aargau(json!([[0, 0.2]]), "aargau"), BOUGHT));
        assert!(!has(&aargau(json!([[0, 0.2]]), "zurich"), BOUGHT));
    }

    #[test]
    fn property_references_and_dates() {
        let own = |buy: &str| {
            with_props(vec![
                house("planned", json!({"buy": buy})),
                json!({"id": "m", "name": "Miete", "status": "rent", "price": 0, "fixCosts": 0, "inflation": false, "from": "start", "rent": 2000}),
            ])
        };
        assert!(has(&own("afterSale:h"), "a purchase after its own sale"));
        assert!(has(&own("afterSale:m"), "“Miete” is rented"));
        assert!(!has(&own("2030-01"), "rented and is never"));
        const EARLY_SELL: &str = "the sale date is before the simulation start";
        let sell = |d: &str| with_props(vec![house("owned", json!({"sell": d, "worth": 1}))]);
        assert!(has(&sell("2026-10"), EARLY_SELL));
        assert!(!has(&sell("2026-11"), EARLY_SELL));
        const EARLY_BUY: &str = "the purchase date is before the simulation start";
        let buy = |d: &str| with_props(vec![house("planned", json!({"buy": d}))]);
        assert!(has(&buy("2026-10"), EARLY_BUY));
        assert!(!has(&buy("2026-11"), EARLY_BUY));
        assert!(!has(&buy("2030-01"), "never bought"));
        const NO_BUY: &str = "no purchase date";
        assert!(has(&buy("never"), NO_BUY));
        assert!(has(&with_props(vec![house("planned", json!({}))]), NO_BUY));
        assert!(!has(&buy("afterSale"), NO_BUY));
    }

    #[test]
    fn mortgage_amounts() {
        let fixed = json!([{"type": "fixed", "term": 10, "rate": 0.02}]);
        let planned = |amount: f64, mode: &str| {
            with_props(vec![house(
                "planned",
                json!({"buy": "2030-01",
                "mortgage": {"amountMode": mode, "tranches": [tranche(amount, None, fixed.clone())]}}),
            )])
        };
        const HIGHER: &str = "the mortgage is higher than the purchase price";
        const LTV: &str = "loan-to-value above 80 %";
        assert!(has(&planned(1_000_001.0, "fixed"), HIGHER));
        assert!(!has(&planned(1_000_000.0, "fixed"), HIGHER));
        assert!(has(&planned(800_001.0, "fixed"), LTV));
        assert!(!has(&planned(800_000.0, "fixed"), LTV));
        assert!(!has(&planned(0.9, "maxLoan"), LTV));
        assert!(!has(&planned(0.9, "maxLoan"), "only applies to planned purchases"));
        let owned = |mode: &str, tranches: Value| {
            with_props(vec![house(
                "owned",
                json!({"sell": "never", "worth": 1,
                "mortgage": {"amountMode": mode, "tranches": tranches}}),
            )])
        };
        assert!(has(
            &owned("maxLoan", json!([tranche(1.0, Some("2026-01"), fixed.clone())])),
            "only applies to planned purchases"
        ));
        // Price and loan-to-value are checked only for planned purchases.
        let high = owned("fixed", json!([tranche(1_200_000.0, Some("2026-01"), fixed.clone())]));
        assert!(!has(&high, LTV) && !has(&high, HIGHER));
        // Start of the period for owned property
        assert!(has(
            &owned("fixed", json!([tranche(1.0, None, fixed.clone())])),
            "Haus: start of the current period missing"
        ));
        assert!(has(
            &owned("fixed", json!([tranche(1.0, Some("2026-12"), fixed.clone())])),
            "the start is after the simulation start"
        ));
        assert!(!has(
            &owned("fixed", json!([tranche(1.0, Some("2026-11"), fixed.clone())])),
            "the start is after"
        ));
        // With several tranches the name of the tranche is in the message
        let mut t2 = tranche(1.0, None, fixed.clone());
        t2["name"] = json!("T2");
        assert!(has(
            &owned("fixed", json!([tranche(1.0, Some("2026-01"), fixed.clone()), t2])),
            "Haus, T2: start of the current period missing"
        ));
    }

    #[test]
    fn mortgage_periods() {
        let owned = |periods: Value| {
            with_props(vec![house(
                "owned",
                json!({"sell": "never", "worth": 1,
                "mortgage": {"tranches": [tranche(100_000.0, Some("2026-01"), periods)]}}),
            )])
        };
        assert!(!has(
            &owned(json!([{"type": "fixed", "term": 100, "rate": 0.02}])),
            "over 100 years"
        ));
        assert!(has(
            &owned(json!([{"type": "fixed", "term": 101, "rate": 0.02}])),
            "period 1: term over 100 years"
        ));
        assert!(!has(
            &owned(json!([{"type": "fixed", "term": 5, "rate": 0.1}])),
            "unusually high"
        ));
        assert!(has(
            &owned(json!([{"type": "fixed", "term": 5, "rate": 0.11}])),
            "interest rate of 11.00 % is unusually high"
        ));
        assert!(!has(
            &owned(json!([{"type": "saron", "term": 5, "rate": 0.2, "margin": 0.03}])),
            "unusually high"
        ));
        assert!(has(
            &owned(json!([{"type": "saron", "term": 5, "margin": 0.031}])),
            "SARON margin of 3.10 %"
        ));
        assert!(!has(
            &owned(json!([{"type": "fixed", "term": 5, "rate": 0.2, "margin": 0.05}])),
            "SARON-Marge"
        ));

        // #50/#52: Indirect amortization counts towards the 3a maximum until before the AHV payout (2031-11).
        let ind = |term: f64, amort: f64, mode: &str| {
            owned(json!([{"type": "fixed", "term": term, "rate": 0.01, "amort": amort, "amortMode": mode}]))
        };
        const MAX: &str = "together exceed the maximum amount";
        // 3a maximum 7258 per year = 1814.5 per quarter
        assert!(!has(&ind(3.0, 1814.5, "indirect"), MAX));
        assert!(has(
            &ind(3.0, 1815.0, "indirect"),
            "indirect amortization (7'260) together exceed the maximum amount of 7'258"
        ));
        assert!(!has(&ind(3.0, 1815.0, "direct"), MAX));
        // Period ended before the start: does not count
        let ended = with_props(vec![house(
            "owned",
            json!({"sell": "never", "worth": 1,
            "mortgage": {"tranches": [tranche(1.0, Some("2010-01"), json!([{"type": "fixed", "term": 5, "rate": 0.01, "amort": 5000, "amortMode": "indirect"}]))]}}),
        )]);
        assert!(!has(&ended, MAX));
        // Limit: a period ending exactly in the start month 2026-11 no longer counts, one month later it does.
        let ends = |from: &str| {
            with_props(vec![house(
                "owned",
                json!({"sell": "never", "worth": 1,
            "mortgage": {"tranches": [tranche(1.0, Some(from), json!([{"type": "fixed", "term": 5, "rate": 0.01, "amort": 5000, "amortMode": "indirect"}]))]}}),
            )])
        };
        assert!(!has(&ends("2021-11"), MAX));
        assert!(has(&ends("2021-12"), MAX));
        // #50: A period from the AHV payout is direct and does not count; one month before it does.
        assert!(!has(&ends("2031-11"), MAX));
        assert!(has(&ends("2031-10"), MAX));
    }

    /// Plan with a 3a account of the person (example «Neu»: start 2026-11, birth 1966-11, AHV payout 2031-11).
    fn with_3a(account: Value) -> Value {
        with(|v| v["pillar3a"] = json!([account]))
    }

    // #52/AK4, AK5: Payout plan at the earliest 5 years before the AHV payout, amount greater than 0.
    #[test]
    fn pillar3a_payout_plan() {
        let pay = |date: &str| {
            with_3a(
                json!({"person": "A", "balance": 10_000, "rate": 0.01, "payouts": [{"date": date, "amount": 5000}]}),
            )
        };
        const EARLY: &str =
            "is more than 5 years before the start of the AHV pension (11.2031); at the earliest in 11.2026";
        assert!(!has(&pay("2026-11"), EARLY), "genau 5 Jahre vorher ist gültig");
        assert!(has(&pay("2026-10"), EARLY));
        assert!(messages(&pay("2026-10"))
            .iter()
            .any(|m| m.contains("Pillar 3a: the payout in 10.2026")));
        assert!(has(&pay("2031-12"), "is after the start of the AHV pension"));
        assert!(!has(&pay("2031-11"), "is after the start of the AHV pension"));
        let zero =
            with_3a(json!({"person": "A", "balance": 1, "rate": 0, "payouts": [{"date": "2030-01", "amount": 0}]}));
        assert!(has(&zero, "must be greater than 0"));
        // #88: a negative amount is also invalid (condition `b.is_nan() || b <= 0.0`).
        let negative =
            with_3a(json!({"person": "A", "balance": 1, "rate": 0, "payouts": [{"date": "2030-01", "amount": -1}]}));
        assert!(has(&negative, "must be greater than 0"));
        let positive =
            with_3a(json!({"person": "A", "balance": 1, "rate": 0, "payouts": [{"date": "2030-01", "amount": 0.01}]}));
        assert!(!has(&positive, "must be greater than 0"));
        let whole = with_3a(json!({"person": "A", "balance": 1, "rate": 0, "payouts": [{"date": "2030-01"}]}));
        assert!(!has(&whole, "must be greater than 0"), "ohne Betrag: ganzes Guthaben");
    }

    // #52: Account, interest, usage, PK lump sum in the same year.
    #[test]
    fn pillar3a_account() {
        assert!(has(
            &with_3a(json!({"person": "X", "balance": 1, "rate": 0})),
            "the person “X” does not exist"
        ));
        assert!(has(
            &with(|v| v["pillar3a"] = json!([{"person": "A"}, {"person": "A"}])),
            "only one 3a account"
        ));
        assert!(has(
            &with_3a(json!({"person": "A", "balance": -1})),
            "balance must not be negative"
        ));
        assert!(has(
            &with_3a(json!({"person": "A", "rate": 1.5})),
            "interest rate must be between"
        ));
        assert!(has(
            &with_3a(json!({"person": "A", "use": "repay"})),
            "requires a tranche with indirect amortization"
        ));
        assert!(
            !has(&with_3a(json!({"person": "A"})), "requires a tranche"),
            "Vorgabe: ins freie Vermögen"
        );
        let indirect = |use_: &str| {
            let mut v = with_props(vec![house(
                "owned",
                json!({"sell": "never", "worth": 1,
                "mortgage": {"tranches": [tranche(100_000.0, Some("2026-01"),
                    json!([{"type": "fixed", "term": 3, "rate": 0.01, "amort": 1000, "amortMode": "indirect"}]))]}}),
            )]);
            v["pillar3a"] = json!([{"person": "A", "use": use_, "payouts": [{"date": "2030-01", "use": use_}]}]);
            v
        };
        assert!(!has(&indirect("repay"), "requires a tranche"));
        let same = |date: &str, lumpsum: f64| {
            let mut v = with_3a(json!({"person": "A", "payouts": [{"date": date}]}));
            v["pension"]["pk"]["lumpsum"] = json!(lumpsum);
            v
        };
        const SAME: &str = "in the same year";
        assert!(has(&same("2031-03", 0.5), SAME));
        assert!(!has(&same("2031-03", 0.0), SAME));
        assert!(!has(&same("2030-12", 0.5), SAME));
    }

    // #52/AK7 and #52/AK10: Contribution between 0 and the maximum (settings), together with the indirect amortization (warning, no block).
    #[test]
    fn pillar3a_contribution() {
        let c = |x: f64| with_3a(json!({"person": "A", "contribution": x}));
        const RANGE: &str = "the contribution must be between 0 and the maximum amount of 7'258 per year";
        assert!(!has(&c(0.0), RANGE));
        assert!(!has(&c(7258.0), RANGE));
        assert!(has(&c(7258.5), RANGE));
        assert!(has(&c(-1.0), RANGE));
        // Maximum from the settings (#25)
        let mut higher = c(7500.0);
        higher["limits"] = json!({"pillar3aMax": 8000});
        assert!(!has(&higher, "the contribution must"));
        // Contribution and indirect amortization together: warning, no block
        let mut both = with_props(vec![house(
            "owned",
            json!({"sell": "never", "worth": 1,
            "mortgage": {"tranches": [tranche(100_000.0, Some("2026-01"),
                json!([{"type": "fixed", "term": 3, "rate": 0.01, "amort": 1000, "amortMode": "indirect"}]))]}}),
        )]);
        both["pillar3a"] = json!([{"person": "A", "contribution": 3258}]);
        assert!(!has(&both, "together exceed"));
        both["pillar3a"][0]["contribution"] = json!(3259);
        let w = Plan::from_json(&both.to_string()).unwrap();
        let hit = validate(&w)
            .into_iter()
            .find(|i| i.message.contains("together exceed"))
            .unwrap();
        assert_eq!(hit.severity, Severity::Warning);
        assert!(hit
            .message
            .contains("contribution (3'259) and indirect amortization (4'000)"));
    }

    // --- #124/AK7: Pillar 3a with two persons, hint «Hypothek zurückzahlen», amounts in francs ---

    /// Second person B (birth 1970-11, AHV payout 2035-11).
    fn couple(v: &mut Value) {
        v["household"]["persons"] = json!([
            { "id": "A", "name": "Anna", "birth": "1966-11" },
            { "id": "B", "name": "Ben", "birth": "1970-11" }
        ]);
        let ms = v["milestones"].as_array_mut().unwrap();
        ms.insert(
            3,
            json!({ "id": "retire2", "name": "Bezug Pensionskasse 2", "kind": "age", "person": "B", "years": 65 }),
        );
        ms.insert(
            4,
            json!({ "id": "ahv2", "name": "AHV-Bezug 2", "kind": "age", "person": "B", "years": 65 }),
        );
        v["partner"] = json!({ "pk": { "capital": 0, "at": "retire2", "lumpsum": 0, "conversion": 0 },
            "ahv": { "monthly": 0, "at": "ahv2" } });
    }

    /// House with one tranche, indirectly amortized with `amort` per quarter into the pillar 3a of `person`.
    fn indirect_house(v: &mut Value, person: Option<&str>, amort: f64) {
        let mut t = json!({ "name": "T", "amount": 400_000, "start": "2026-11",
            "periods": [ { "type": "fixed", "term": 10, "rate": 0.01, "amort": amort, "amortMode": "indirect" } ] });
        if let Some(p) = person {
            t["pillar3a"] = json!(p);
        }
        v["realestate"]["properties"] = json!([ { "id": "h", "name": "Haus", "status": "owned", "price": 500_000,
            "worth": 800_000, "sell": "never", "fixCosts": 0, "mortgage": { "tranches": [t] } } ]);
    }

    const OVER: &str = "contribution (0) and indirect amortization (8'000) together exceed the maximum amount";

    #[test]
    fn issue_124_ak7_indirect_amortization_per_person() {
        // Tranche into the 3a of B (8'000 per year): the warning names B, not A.
        let v = with(|v| {
            couple(v);
            indirect_house(v, Some("B"), 2000.0);
        });
        assert!(has(&v, &format!("Pillar 3a Ben: {OVER}")), "{:?}", messages(&v));
        assert!(!has(&v, "Pillar 3a Anna"), "{:?}", messages(&v));
        // A has an account, B does not: for B the warning without an account still applies.
        let v = with(|v| {
            couple(v);
            indirect_house(v, Some("B"), 2000.0);
            v["pillar3a"] = json!([{ "person": "A", "balance": 0, "rate": 0 }]);
        });
        assert!(has(&v, &format!("Pillar 3a Ben: {OVER}")), "{:?}", messages(&v));
        // One person: without names.
        let v = with(|v| indirect_house(v, None, 2000.0));
        assert!(
            messages(&v)
                .iter()
                .any(|m| m.starts_with(&format!("Pillar 3a: {OVER}"))),
            "{:?}",
            messages(&v)
        );
    }

    #[test]
    fn issue_124_ak7_repay_needs_an_indirect_tranche() {
        const REPAY: &str = "“Repay mortgage” (payout in 01.2032) requires a tranche with indirect amortization";
        let account = |usage: &str| {
            json!([{ "person": "B", "balance": 10_000, "rate": 0,
                "payouts": [{ "date": "2032-01", "amount": 5_000, "use": usage }] }])
        };
        let v = with(|v| {
            couple(v);
            v["pillar3a"] = account("repay");
        });
        assert!(has(&v, &format!("Pillar 3a Ben: {REPAY}")), "{:?}", messages(&v));
        let v = with(|v| {
            couple(v);
            v["pillar3a"] = account("wealth");
        });
        assert!(!has(&v, "Repay mortgage"), "{:?}", messages(&v));
    }

    #[test]
    fn issue_124_ak7_chf_formats_with_thousands_separator() {
        assert_eq!(crate::message::group(0.0), "0");
        assert_eq!(crate::message::group(-0.4), "0");
        assert_eq!(crate::message::group(-1234.4), "-1'234");
        assert_eq!(crate::message::group(1_234_567.0), "1'234'567");
    }
}
