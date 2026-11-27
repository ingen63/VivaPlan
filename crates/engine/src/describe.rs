//! #219: Catalog for AIs (`vivaplan describe`, later a tool of the MCP server #210). It tells a client without the
//! source code what it can change, vary and measure: the fields of the plan with type, unit, range and meaning, the
//! forms of a course over time, the state variables of characteristic curves, the building blocks of metrics, the forms
//! of the space of variations and of the objective, three example experiments, the measured microseconds per run and
//! the limits of the model.
//!
//! The catalog is generated from the JSON schema and the code: paths, types, allowed values and most meanings come from
//! the schema of the plan ([`crate::model::Plan`]), of a metric ([`crate::measure::Metric`]) and of the experiment
//! ([`crate::experiment::Experiment`]); units and ranges come from the table [`UNITS`], missing meanings from
//! [`MEANINGS`]. The tests check both tables against the schema in both directions.
//!
//! The catalog stays small (under 20 000 characters, also as JSON); single sections with [`Section`].

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::OnceLock;
use std::time::Instant;

use serde::Serialize;
use serde_json::{json, Value};

use crate::message::Message;
use crate::model::Plan;

/// Format of the answer.
pub const FORMAT: &str = "vivaplan-describe/1";

/// Most characters of a meaning (longer texts are cut at a word; the full text is in `vivaplan schema`).
pub const MEANING_CHARS: usize = 58;

/// Sections of the catalog (`vivaplan describe <section>`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum Section {
    /// Fields of the plan with type, unit, range and meaning.
    Fields,
    /// Forms of a course over time and points in time of field paths.
    Series,
    /// State variables of characteristic curves.
    State,
    /// Building blocks of metrics.
    Metrics,
    /// Forms of the experiment: space of variations, constraints, objective, search, budget.
    Experiment,
    /// Three example experiments.
    Examples,
    /// Measured microseconds per run.
    Performance,
    /// Limits of the model.
    Limits,
    /// #304: Every value of a choice field with its label in the app and its meaning.
    Choices,
}

impl Section {
    pub const ALL: [Section; 9] = [
        Section::Fields,
        Section::Series,
        Section::State,
        Section::Metrics,
        Section::Experiment,
        Section::Examples,
        Section::Performance,
        Section::Limits,
        Section::Choices,
    ];

    pub fn name(self) -> &'static str {
        match self {
            Section::Fields => "fields",
            Section::Series => "series",
            Section::State => "state",
            Section::Metrics => "metrics",
            Section::Experiment => "experiment",
            Section::Examples => "examples",
            Section::Performance => "performance",
            Section::Limits => "limits",
            Section::Choices => "choices",
        }
    }

    /// The section of a name (`None`: unknown).
    pub fn parse(name: &str) -> Option<Section> {
        Section::ALL.into_iter().find(|s| s.name() == name)
    }
}

// ---------------------------------------------------------------------------------------------
// Answer
// ---------------------------------------------------------------------------------------------

/// The catalog; a section is missing if another one was asked for.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Catalog {
    pub format: &'static str,
    /// Version of the engine.
    pub engine: &'static str,
    /// Names of all sections.
    pub sections: Vec<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<Fields>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub series: Option<SeriesForms>,
    #[serde(skip_serializing_if = "Option::is_none", serialize_with = "as_map_opt")]
    pub state: Option<Vec<Item>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metrics: Option<MetricBlocks>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub experiment: Option<ExperimentForms>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub examples: Option<Vec<Example>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub performance: Option<Performance>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limits: Option<Vec<&'static str>>,
    /// #304: only as the section `choices` (the whole catalog stays small).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub choices: Option<crate::choices::Choices>,
}

/// A name with its meaning.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Item {
    pub name: String,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub meaning: String,
}

fn item(name: impl Into<String>, meaning: impl Into<String>) -> Item {
    Item {
        name: name.into(),
        meaning: meaning.into(),
    }
}

/// Fields of the plan, grouped by the object that holds them (path = group + `.` + name; `*` in a path stands for an
/// index of a list, `pillar3a.0.rate`). Each field is a short row `name: type, unit, range – meaning`; the meaning
/// only in the section `fields` (the whole catalog stays small).
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Fields {
    pub notes: Vec<&'static str>,
    /// Units with their meaning.
    #[serde(serialize_with = "as_map")]
    pub units: Vec<Item>,
    /// Form of a row.
    pub row: &'static str,
    /// Group (`""` for the top level) → rows.
    pub groups: BTreeMap<String, Vec<String>>,
}

impl Fields {
    /// Field list of the catalog, with the meanings or without.
    pub fn new(meanings: bool) -> Fields {
        let mut groups: BTreeMap<String, Vec<String>> = BTreeMap::new();
        for f in fields() {
            let (group, name) = f.path.rsplit_once('.').unwrap_or(("", &f.path));
            let kind = if f.values.is_empty() {
                f.kind.clone()
            } else {
                f.values.join("|")
            };
            let parts: Vec<&str> = [kind.as_str(), f.unit, f.range]
                .into_iter()
                .filter(|p| !p.is_empty())
                .collect();
            let mut row = format!("{name}: {}", parts.join(", "));
            if meanings && !f.meaning.is_empty() {
                row.push_str(" – ");
                row.push_str(&f.meaning);
            }
            groups.entry(group.to_string()).or_default().push(row);
        }
        Fields {
            notes: vec![
                "Set with vivaplan set <plan> <path>=<value> or in an experiment (set, vary); * is a list index.",
                "type series: a course over time (section series); one step with path@<point in time>.",
                "Meanings: vivaplan describe fields; full texts and structure: vivaplan schema.",
                "Values a|b with the labels of the app: describe choices.",
            ],
            units: UNIT_LEGEND.iter().map(|&(u, m)| item(u, m)).collect(),
            row: if meanings {
                "name: type (an enum as its values a|b), unit, range – meaning"
            } else {
                "name: type (an enum as its values a|b), unit, range"
            },
            groups,
        }
    }
}

fn as_map_opt<S: serde::Serializer>(items: &Option<Vec<Item>>, s: S) -> Result<S::Ok, S::Error> {
    as_map(items.as_deref().unwrap_or_default(), s)
}

/// A list of names with meanings as a JSON object in the order of the list (smaller than a list of objects).
fn as_map<S: serde::Serializer>(items: &[Item], s: S) -> Result<S::Ok, S::Error> {
    use serde::ser::SerializeMap;
    let mut m = s.serialize_map(Some(items.len()))?;
    for i in items {
        m.serialize_entry(&i.name, &i.meaning)?;
    }
    m.end()
}

/// A field of the plan.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Field {
    pub path: String,
    /// `number`, `integer`, `string`, `boolean`, `enum`, `series` (a course over time), `series|curve` (also a
    /// characteristic curve), `array` (a table), `object` (a map).
    #[serde(rename = "type")]
    pub kind: String,
    /// Allowed values of an `enum`.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub values: Vec<String>,
    #[serde(skip_serializing_if = "str::is_empty")]
    pub unit: &'static str,
    #[serde(skip_serializing_if = "str::is_empty")]
    pub range: &'static str,
    pub meaning: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeriesForms {
    /// Forms of a value of type `series` (`mode`).
    #[serde(serialize_with = "as_map")]
    pub forms: Vec<Item>,
    /// Points in time of the forms and of field paths.
    #[serde(serialize_with = "as_map")]
    pub points_in_time: Vec<Item>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MetricBlocks {
    /// A metric as JSON; `name` for constraints and objective.
    pub shape: Value,
    #[serde(serialize_with = "as_map")]
    pub quantities: Vec<Item>,
    #[serde(serialize_with = "as_map")]
    pub window: Vec<Item>,
    #[serde(serialize_with = "as_map")]
    pub reduce: Vec<Item>,
    #[serde(serialize_with = "as_map")]
    pub aggregate: Vec<Item>,
    #[serde(serialize_with = "as_map")]
    pub ops: Vec<Item>,
    pub notes: Vec<&'static str>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExperimentForms {
    /// Keys of an experiment.
    #[serde(serialize_with = "as_map")]
    pub keys: Vec<Item>,
    /// Forms of an entry of `vary`.
    #[serde(serialize_with = "as_map")]
    pub vary: Vec<Item>,
    /// Forms of `objective` (exactly one).
    #[serde(serialize_with = "as_map")]
    pub objective: Vec<Item>,
    /// A constraint as JSON.
    pub constraint: Value,
    /// Methods of `search.method`.
    #[serde(serialize_with = "as_map")]
    pub methods: Vec<Item>,
    /// Defaults of `search` and `budget`.
    pub defaults: Value,
    pub max_variants: usize,
    /// From this estimated duration (seconds) only with `"confirm": true`.
    pub confirm_seconds: f64,
}

/// An example experiment.
#[derive(Clone, Debug, Serialize)]
pub struct Example {
    pub name: &'static str,
    pub question: &'static str,
    pub experiment: Value,
}

/// Measured computing time on this machine.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Performance {
    /// `given` (the plan of the call) or `example` (the example plan with parametric returns, as in the examples).
    pub plan: &'static str,
    /// Microseconds per run (all cores together), including generating the return path; `None` if the measurement
    /// failed (`error`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub us_per_run: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub us_generate: Option<f64>,
    pub cores: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub months: Option<usize>,
    /// Duration of the measurement in milliseconds.
    pub measured_ms: f64,
    pub estimate: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<Message>,
}

// ---------------------------------------------------------------------------------------------
// Units and meanings (code)
// ---------------------------------------------------------------------------------------------

/// Units of the field list.
pub const UNIT_LEGEND: &[(&str, &str)] = &[
    ("CHF", "amount in the plan's currency"),
    ("CHF/month", "amount per month"),
    ("CHF/year", "amount per year"),
    ("CHF/quarter", "amount per quarter"),
    ("share", "fraction, 0.6 = 60 %"),
    ("rate/year", "fraction per year, 0.02 = 2 % p.a."),
    ("factor", "multiplier, 1 = unchanged"),
    ("age", "age in decimal years, 64.5 = 64 years 6 months"),
    ("years", "duration in years"),
    ("months", "duration in months"),
    ("count", "number"),
    ("year", "calendar year"),
];

/// Unit and range of every field of type number, integer or series: (path, unit, range). Ranges are those the
/// validation accepts or, where it accepts more, the sensible ones.
pub const UNITS: &[(&str, &str, &str)] = &[
    ("flows.extras.*.amount", "CHF", "+ inflow, − outflow"),
    ("flows.income", "CHF/month", ">= 0"),
    ("flows.savings", "CHF/month", ">= 0"),
    ("flows.spending", "CHF/month", ">= 0"),
    ("flows.spendingFactor", "factor", "> 0"),
    ("household.ahvCoupleCap", "CHF/month", ">= 0"),
    ("household.persons.*.careHome.age", "age", "0..150"),
    ("household.persons.*.careHome.monthly", "CHF/month", ">= 0"),
    ("milestones.*.years", "age", "0..120"),
    ("optimization.checkPaths", "count", ">= 1"),
    ("optimization.maxRuin", "share", "0..1"),
    ("optimization.maxVol", "share", ">= 0"),
    ("optimization.minConsumption", "share", "0..1"),
    ("optimization.paths", "count", ">= 1"),
    ("optimization.ruinAge", "age", "0..120"),
    ("optimization.samples", "count", ">= 0"),
    ("optimization.volWeight", "factor", ">= 0"),
    ("optimization.weightMedian", "share", "0..1"),
    ("partner.ahv.monthly", "CHF/month", ">= 0"),
    ("partner.income", "CHF/month", ">= 0"),
    ("partner.pk.capital", "CHF", ">= 0"),
    ("partner.pk.contribution", "CHF/month", ">= 0"),
    ("partner.pk.conversion", "rate/year", "0..0.1"),
    ("partner.pk.interest", "rate/year", "-0.1..0.1"),
    ("partner.pk.lumpsum", "share", "0..1"),
    ("pension.ahv.monthly", "CHF/month", ">= 0"),
    ("pension.pk.capital", "CHF", ">= 0"),
    ("pension.pk.contribution", "CHF/month", ">= 0"),
    ("pension.pk.conversion", "rate/year", "0..0.1"),
    ("pension.pk.interest", "rate/year", "-0.1..0.1"),
    ("pension.pk.lumpsum", "share", "0..1"),
    ("pillar3a.*.balance", "CHF", ">= 0"),
    ("pillar3a.*.contribution", "CHF/year", "0..legal maximum"),
    ("pillar3a.*.payouts.*.amount", "CHF", ">= 0"),
    ("pillar3a.*.payouts.*.years", "age", "0..120"),
    ("pillar3a.*.rate", "rate/year", "-0.5..0.5"),
    ("pillar3a.*.tranches.count", "count", "1..6"),
    ("pillar3a.*.tranches.fromAge", "age", "60..65"),
    ("realestate.affordability.fixCosts", "share", "0..1"),
    ("realestate.affordability.fixedMargin", "rate/year", ">= 0"),
    ("realestate.affordability.imputedRate", "rate/year", "0..0.2"),
    ("realestate.affordability.maxLoanToValue", "share", "0..1"),
    ("realestate.affordability.maxLoanToValueRetired", "share", "0..1"),
    ("realestate.affordability.sustainability", "share", "0..1, of income"),
    ("realestate.affordability.wealthContribution", "share", "unused"),
    ("realestate.affordability.wealthDrawdownRate", "rate/year", "0..1"),
    ("realestate.emergencyMonths", "months", ">= 0"),
    ("realestate.properties.*.fixCosts", "CHF/month", ">= 0"),
    ("realestate.properties.*.mortgage.pillar3a.balance", "CHF", ">= 0"),
    (
        "realestate.properties.*.mortgage.pillar3a.rate",
        "rate/year",
        "-0.5..0.5",
    ),
    (
        "realestate.properties.*.mortgage.tranches.*.amount",
        "CHF",
        ">= 0; share with amountMode maxLoan",
    ),
    (
        "realestate.properties.*.mortgage.tranches.*.periods.*.amort",
        "CHF/quarter",
        ">= 0",
    ),
    (
        "realestate.properties.*.mortgage.tranches.*.periods.*.margin",
        "rate/year",
        "-0.05..0.1",
    ),
    (
        "realestate.properties.*.mortgage.tranches.*.periods.*.rate",
        "rate/year",
        "0..0.2",
    ),
    (
        "realestate.properties.*.mortgage.tranches.*.periods.*.term",
        "years",
        "> 0",
    ),
    ("realestate.properties.*.price", "CHF", ">= 0"),
    ("realestate.properties.*.rent", "CHF/month", ">= 0"),
    (
        "realestate.properties.*.taxValue.value",
        "CHF or share",
        ">= 0; share 0..1",
    ),
    ("realestate.properties.*.worth", "CHF", ">= 0"),
    ("returns.bootstrap.block", "months", ">= 1"),
    ("returns.bootstrap.paths", "count", "1..100000"),
    ("returns.bootstrap.seed", "count", ">= 0"),
    ("returns.extrapolation.range", "years", ">= 1"),
    ("returns.fixed.bonds", "rate/year", "-1..1"),
    ("returns.fixed.inflation", "rate/year", "-0.1..0.2"),
    ("returns.fixed.saron", "rate/year", "-0.05..0.2"),
    ("returns.realWageGrowth", "rate/year", "-0.02..0.03"),
    ("returns.fixed.stocks", "rate/year", "-1..1"),
    ("returns.parametric.df", "count", "0 or > 2"),
    ("returns.parametric.jumpLambda", "count", ">= 0, per year"),
    ("returns.parametric.jumpMu", "factor", "log jump size, -1..0"),
    ("returns.parametric.jumpSigma", "factor", ">= 0, log"),
    ("returns.parametric.mu", "rate/year", "-0.2..0.3"),
    ("returns.parametric.paths", "count", "1..100000"),
    ("returns.parametric.realShortRate", "rate/year", "optional"),
    ("returns.parametric.sampler", "count", "internal"),
    ("returns.parametric.seed", "count", ">= 0"),
    ("returns.parametric.sigma", "rate/year", "0..1"),
    ("returns.strategy.weights.apac", "share", "0..1, sum 1"),
    ("returns.strategy.weights.ch", "share", "0..1, sum 1"),
    ("returns.strategy.weights.eu", "share", "0..1, sum 1"),
    ("returns.strategy.weights.usa", "share", "0..1, sum 1"),
    ("taxes.deductions.*.amount", "CHF/year", ">= 0"),
    ("taxes.deductions.*.extra", "CHF/year", ">= 0"),
    ("taxes.deductions.*.rate", "share", "0..1"),
    ("taxes.deductions.*.taxBill", "year", "tax year"),
    ("taxes.investmentYield.bonds", "rate/year", "0..1"),
    ("taxes.investmentYield.stocks", "rate/year", "0..1"),
    ("taxes.location.municipality", "count", "BFS number"),
    ("taxes.multiplier", "factor", ">= 0"),
    ("taxes.oneOffDeductions.*.amount", "CHF", ">= 0"),
    ("taxes.oneOffDeductions.*.year", "year", ""),
    ("taxes.oneOffWealthDeductions.*.amount", "CHF", ">= 0"),
    ("taxes.oneOffWealthDeductions.*.year", "year", ""),
    ("taxes.propertyGain.allowance", "CHF", ">= 0"),
    ("taxes.propertyGain.exemptBelow", "CHF", ">= 0"),
    ("taxes.propertyGain.maxShare", "share", "0..1"),
    ("taxes.propertyGain.reduction", "share", "0..1"),
    ("taxes.relocations.*.municipality", "count", "BFS number"),
    ("taxes.wealthDeductions.*.amount", "CHF", ">= 0"),
    ("wealth.bonds", "CHF", ">= 0"),
    ("wealth.cash", "CHF", ">= 0"),
    ("wealth.reserveMonths", "months", ">= 0"),
    ("wealth.stockShare", "share", "0..1"),
    ("wealth.stocks", "CHF", ">= 0"),
    ("withdrawal.buckets.bondYears", "years", ">= 0"),
    ("withdrawal.buckets.cashYears", "years", ">= 0"),
    ("withdrawal.buckets.refillThreshold", "rate/year", "-1..1"),
    ("withdrawal.buckets.reserveMonths", "months", ">= 1"),
    ("withdrawal.ceiling", "share", ">= 0, 0 = none"),
    ("withdrawal.endowment.rate", "rate/year", "0..1"),
    ("withdrawal.endowment.smoothing", "share", "0..1"),
    ("withdrawal.fixedRate.rate", "rate/year", "0..1"),
    ("withdrawal.floor", "share", ">= 0, 0 = none"),
    ("withdrawal.guardrails.adjust", "share", "0..1"),
    ("withdrawal.guardrails.initialRate", "rate/year", "0..1"),
    ("withdrawal.guardrails.lower", "share", "0..1"),
    ("withdrawal.guardrails.upper", "share", "0..1"),
    ("withdrawal.vpw.horizonAge", "age", "0..120"),
    ("withdrawal.vpw.realReturn", "rate/year", "-0.1..0.2"),
];

/// Meanings of fields whose schema has no description (or only an unclear one).
pub const MEANINGS: &[(&str, &str)] = &[
    ("flows.extras.*.amount", "One-off amount in this month."),
    ("flows.extrasIndexed", "With inflation."),
    ("flows.incomeIndexed", "With inflation."),
    ("flows.spendingIndexed", "With inflation."),
    ("household.persons.*.careHome.indexed", "With inflation."),
    ("indexedByDefault", "Missing inflation flags mean yes (set by program)."),
    ("partner.ahv.indexed", "With inflation."),
    ("partner.incomeIndexed", "With inflation."),
    ("partner.pk.contributionIndexed", "With inflation."),
    ("pension.ahv.indexed", "With inflation."),
    ("pension.pk.contributionIndexed", "With inflation."),
    ("pillar3a.*.contributionIndexed", "With inflation, at most the maximum."),
    (
        "realestate.properties.*.inflation",
        "Older field for value and fix costs.",
    ),
    ("realestate.properties.*.inflationFixCosts", "With inflation."),
    (
        "realestate.properties.*.inflationRent",
        "Missing: until moving in; true: always; false: nominal.",
    ),
    ("realestate.properties.*.inflationValue", "With inflation."),
    ("taxes.oneOffIndexed", "With inflation."),
    ("taxes.oneOffWealthIndexed", "With inflation."),
    ("taxes.pillar3aMaxIndexed", "With inflation."),
    ("flows.extras.*.date", "Month of the one-off amount (YYYY-MM)."),
    ("flows.extras.*.label", "Label of the one-off amount."),
    (
        "flows.incomeEndsAtRetirement",
        "Earned income ends at retirement, whatever the series says.",
    ),
    (
        "flows.spendingIncludesHousing",
        "Spending includes the housing costs of the start month.",
    ),
    ("format", "Version of the file format (set by the program)."),
    ("household.persons.*.birth", "Month of birth (YYYY-MM)."),
    (
        "household.persons.*.id",
        "Id of the person (A, B), used by milestones and accounts.",
    ),
    ("household.persons.*.name", "Name of the person."),
    (
        "household.persons.*.noPensionFund",
        "No pension fund: large pillar 3a; pension fund counts as 0.",
    ),
    ("milestones.*.fixed", "Fixed milestone (start, end): cannot be removed."),
    (
        "milestones.*.id",
        "Id, used as a point in time (`@retire`, `from: retire`).",
    ),
    ("milestones.*.date", "Month with `kind = date` (YYYY-MM)."),
    ("milestones.*.kind", "`date` or `age` of a person."),
    ("milestones.*.person", "Person (id) with `kind = age`."),
    ("milestones.*.years", "Age with `kind = age`."),
    ("optimization.checkPaths", "Paths of the check of the best values."),
    ("optimization.samples", "Random starting points per strategy."),
    (
        "partner.ahv.at",
        "Start of the AHV pension (milestone id, age or YYYY-MM).",
    ),
    (
        "partner.ahv.referenceAge",
        "Pension already adjusted to the reference age (set by program).",
    ),
    (
        "partner.ahv.thirteenthNoted",
        "13th AHV pension noted (set by the program).",
    ),
    (
        "partner.incomeEndsAtRetirement",
        "Earned income ends at retirement, whatever the series says.",
    ),
    (
        "pension.ahv.at",
        "Start of the AHV pension (milestone id, age or YYYY-MM).",
    ),
    (
        "pension.ahv.referenceAge",
        "Pension already adjusted to the reference age (set by program).",
    ),
    (
        "pension.ahv.thirteenthNoted",
        "13th AHV pension noted (set by the program).",
    ),
    (
        "pillar3a.*.payouts.*.use",
        "Repay mortgage, to wealth or home (early withdrawal at any age).",
    ),
    ("project.currency", "Currency of all amounts (CHF)."),
    ("project.name", "Name of the plan."),
    (
        "realestate.affordability.fixedMargin",
        "Margin of `market` over bonds 10 y.",
    ),
    (
        "realestate.affordability.fixCosts",
        "Fix costs per year, share of the price.",
    ),
    ("realestate.affordability.wealthContribution", "Unused."),
    (
        "realestate.affordability.imputedRate",
        "Imputed mortgage rate of the affordability check.",
    ),
    (
        "realestate.affordability.maxLoanToValue",
        "Highest mortgage as a share of the value.",
    ),
    (
        "realestate.affordability.maxLoanToValueRetired",
        "Highest mortgage share once everyone is retired.",
    ),
    (
        "realestate.affordability.sustainability",
        "Highest share of income for housing costs.",
    ),
    (
        "realestate.affordability.wealthDrawdown",
        "Count a share of the free wealth as income in the affordability.",
    ),
    (
        "realestate.affordability.wealthDrawdownRate",
        "Share of free wealth counted as income per year.",
    ),
    (
        "realestate.defaultTaxValue",
        "Without a tax value: the default of 50 % of the market value.",
    ),
    ("realestate.neverSell", "Never sell properties (no emergency sales)."),
    (
        "realestate.properties.*.buy",
        "Purchase (planned): milestone id, age or YYYY-MM.",
    ),
    (
        "realestate.properties.*.canton",
        "Canton of the property for the property gain tax.",
    ),
    (
        "realestate.properties.*.fixCosts",
        "Fix costs per month in prices at the simulation start.",
    ),
    (
        "realestate.properties.*.mortgage.pillar3a.balance",
        "Pledged pillar 3a: balance (older plans).",
    ),
    (
        "realestate.properties.*.mortgage.pillar3a.payout",
        "Pledged pillar 3a: payout (older plans).",
    ),
    (
        "realestate.properties.*.mortgage.pillar3a.rate",
        "Pledged pillar 3a: interest (older plans).",
    ),
    (
        "realestate.properties.*.mortgage.pillar3a.use",
        "Pledged pillar 3a: use (older plans).",
    ),
    (
        "realestate.properties.*.mortgage.tranches.*.after",
        "At the end of the last period: renew or repay.",
    ),
    (
        "realestate.properties.*.mortgage.tranches.*.name",
        "Name of the tranche.",
    ),
    (
        "realestate.properties.*.mortgage.tranches.*.periods.*.amortMode",
        "Amortization direct or via pillar 3a.",
    ),
    (
        "realestate.properties.*.mortgage.tranches.*.periods.*.type",
        "Fixed, from the market (`market`), SARON.",
    ),
    ("realestate.properties.*.name", "Name of the property."),
    (
        "realestate.properties.*.sell",
        "Sale (owned, planned): milestone id, age or YYYY-MM.",
    ),
    ("realestate.properties.*.status", "Owned, planned purchase or rented."),
    ("realestate.properties.*.until", "Rent until (status `rent`)."),
    ("returns.bootstrap.paths", "Number of paths (Monte Carlo)."),
    (
        "returns.extrapolation.method",
        "Returns after the end of the historical data.",
    ),
    ("returns.fixed.bonds", "Bond return per year (source fixed)."),
    ("returns.fixed.inflation", "Inflation per year (source fixed)."),
    ("returns.realWageGrowth", "Real wage growth for the AHV mixed index."),
    (
        "returns.fixed.saron",
        "SARON per year (source fixed), for SARON mortgages.",
    ),
    ("returns.fixed.stocks", "Stock return per year (source fixed)."),
    ("returns.parametric.auto", "Parameters fitted to the historical data."),
    (
        "returns.parametric.jumpSigma",
        "Standard deviation of the logarithmic jump size.",
    ),
    ("returns.parametric.paths", "Number of paths (Monte Carlo)."),
    ("returns.parametric.realShortRate", "SARON − inflation (empty: series)."),
    (
        "returns.parametric.sampler",
        "Version of the random method (set by the program).",
    ),
    ("returns.parametric.seed", "Random seed (same value ⇒ same paths)."),
    ("returns.saronSeries", "Series of the SARON (library file)."),
    ("returns.source", "Source of the returns; app labels: describe choices."),
    ("returns.strategy.weights.ch", "Switzerland."),
    ("returns.strategy.weights.usa", "United States."),
    ("scenarios.sweeps", "Variations of the user interface (list)."),
    ("taxes.capital", "Tax table for lump sums: [from amount, rate]."),
    ("taxes.couple.capital", "Tax table for lump sums, married couples."),
    ("taxes.couple.income", "Tax table for income, married couples."),
    ("taxes.couple.wealth", "Tax table for wealth, married couples."),
    ("taxes.deductPillar3a", "Deduct pillar 3a contributions from income."),
    ("taxes.deductions.*.extra", "Own value: additional amount per year."),
    (
        "taxes.deductions.*.from",
        "Applies from (milestone id, age or YYYY-MM).",
    ),
    (
        "taxes.deductions.*.taxBill",
        "Origin: taken over from the tax bill of this tax year.",
    ),
    ("taxes.income", "Tax table for income: [from amount, rate]."),
    ("taxes.indexed", "Tax brackets and ongoing deductions with inflation."),
    (
        "taxes.investmentYield.bonds",
        "Taxable coupons of the bonds per year (share).",
    ),
    (
        "taxes.investmentYield.stocks",
        "Taxable dividends of the stocks per year (share).",
    ),
    ("taxes.oneOffDeductions.*.amount", "One-off deduction from income."),
    ("taxes.oneOffDeductions.*.label", "Label of the deduction."),
    ("taxes.oneOffDeductions.*.year", "Tax year of the deduction."),
    (
        "taxes.oneOffWealthDeductions.*.amount",
        "One-off deduction from wealth.",
    ),
    ("taxes.oneOffWealthDeductions.*.label", "Label of the deduction."),
    ("taxes.oneOffWealthDeductions.*.year", "Tax year of the deduction."),
    (
        "taxes.propertyGain.brackets",
        "Brackets of the property gain tax (Zurich model).",
    ),
    (
        "taxes.propertyGain.duration",
        "Change of the tax by holding period: [from years, change].",
    ),
    (
        "taxes.propertyGain.durationAbove",
        "Change by holding period above a gain (St. Gallen).",
    ),
    (
        "taxes.propertyGain.longBrackets",
        "Brackets from a holding period on (Valais).",
    ),
    (
        "taxes.propertyGain.returnMaxDuration",
        "Model «return»: holding period of the highest rate.",
    ),
    (
        "taxes.propertyGain.shortSurcharge",
        "Surcharge for short holding (Fribourg).",
    ),
    (
        "taxes.propertyGainOwn",
        "Own property gain tax instead of the canton's.",
    ),
    ("taxes.relocations.*.church", "Denomination for the church tax."),
    ("taxes.relocations.*.from", "Move from (milestone id, age or YYYY-MM)."),
    ("taxes.wealth", "Tax table for wealth: [from amount, rate]."),
    (
        "taxes.wealthDeductions.*.amount",
        "Deduction from wealth (prices at the simulation start).",
    ),
    (
        "taxes.wealthDeductions.*.from",
        "Applies from (milestone id, age or YYYY-MM).",
    ),
    ("wealth.bonds", "Bonds at the start."),
    ("wealth.cash", "Cash at the start."),
    ("wealth.stocks", "Stocks at the start."),
    (
        "withdrawal.buckets.spending",
        "Spending of the bucket strategy: planned or with guardrails.",
    ),
    ("withdrawal.endowment.rate", "Withdrawal rate of the endowment rule."),
    (
        "withdrawal.fixedRate.rate",
        "Withdrawal rate of liquid wealth per year.",
    ),
    ("withdrawal.guardrails.initialRate", "Initial withdrawal rate."),
    (
        "withdrawal.guardrails.lower",
        "Lower guardrail: relative downward deviation of the rate.",
    ),
    (
        "withdrawal.strategy",
        "Withdrawal strategy after retirement; labels: describe choices.",
    ),
];

// ---------------------------------------------------------------------------------------------
// Walking the schema
// ---------------------------------------------------------------------------------------------

/// Schema of the plan.
fn plan_schema() -> &'static Value {
    static SCHEMA: OnceLock<Value> = OnceLock::new();
    SCHEMA.get_or_init(|| serde_json::to_value(schemars::schema_for!(Plan)).expect("schema is serializable"))
}

fn resolve<'a>(root: &'a Value, node: &'a Value) -> &'a Value {
    match node.get("$ref").and_then(Value::as_str) {
        Some(r) => r
            .strip_prefix("#/definitions/")
            .and_then(|name| root["definitions"].get(name))
            .unwrap_or(&Value::Null),
        None => node,
    }
}

/// The node and all alternatives it combines, each with its references followed.
fn alternatives<'a>(root: &'a Value, node: &'a Value) -> Vec<&'a Value> {
    let node = resolve(root, node);
    let mut out = vec![node];
    for key in ["allOf", "anyOf", "oneOf"] {
        for sub in node.get(key).and_then(Value::as_array).into_iter().flatten() {
            out.extend(alternatives(root, sub));
        }
    }
    out
}

/// The definition a node refers to, directly or as the only member of `allOf`.
fn referenced(node: &Value) -> Option<&str> {
    node.get("$ref")
        .or_else(|| node.get("allOf").and_then(|a| a.get(0)).and_then(|a| a.get("$ref")))
        .and_then(Value::as_str)
        .and_then(|r| r.strip_prefix("#/definitions/"))
}

/// Course over time (`Series`) or parameter with a characteristic curve (`StateSeries`), directly or in a combination.
fn series_kind(root: &Value, node: &Value) -> Option<&'static str> {
    match referenced(node) {
        Some("Series") => return Some("series"),
        Some("StateSeries") => return Some("series|curve"),
        _ => {}
    }
    let node = resolve(root, node);
    ["anyOf", "oneOf"]
        .iter()
        .filter_map(|k| node.get(*k).and_then(Value::as_array))
        .flatten()
        .find_map(|n| series_kind(root, n))
}

/// Title and description of a node (schemars splits the doc comment into both).
fn doc(node: &Value) -> String {
    let part = |k: &str| node.get(k).and_then(Value::as_str).unwrap_or("").trim().to_string();
    let (t, d) = (part("title"), part("description"));
    match (t.is_empty(), d.is_empty()) {
        (false, false) => format!("{t} {d}"),
        (false, true) => t,
        _ => d,
    }
}

/// Short meaning: without the issue number in front, the first sentence, at most [`MEANING_CHARS`] characters.
pub fn short(text: &str) -> String {
    let mut words: Vec<String> = vec![];
    for w in text.split_whitespace() {
        if !is_issue_ref(w) {
            words.push(w.to_string());
        } else if w.starts_with('(') && w.ends_with(':') {
            // "Empty (#170): …" keeps its colon.
            if let Some(last) = words.last_mut() {
                last.push(':');
            }
        }
    }
    let mut t = words.join(" ");
    // "63: …" in front
    if let Some((head, rest)) = t.split_once(": ") {
        if !head.is_empty() && head.chars().all(|c| c.is_ascii_digit()) {
            t = rest.to_string();
        }
    }
    // The first sentence; "p.a. " and "e.g. " do not end one.
    let mut from = 0;
    while let Some(i) = t[from..].find(". ").map(|i| i + from) {
        if ["p.a", "e.g", "i.e"].iter().any(|a| t[..i].ends_with(a)) {
            from = i + 2;
            continue;
        }
        t.truncate(i + 1);
        break;
    }
    if t.chars().count() > MEANING_CHARS {
        let cut: String = t.chars().take(MEANING_CHARS - 1).collect();
        let at = cut.rfind(' ').unwrap_or(cut.len());
        t = format!("{}…", cut[..at].trim_end_matches([',', ';', ' ']));
    }
    t
}

/// A reference to an issue in a doc comment: `#218:`, `(#170)`, `(#170):`, `#52;`.
fn is_issue_ref(word: &str) -> bool {
    let w = word.trim_start_matches('(').trim_end_matches([')', ':', ';', ',']);
    w.strip_prefix('#')
        .is_some_and(|n| !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()))
}

/// Meaning of a field from the schema: its own text or that of the definition it refers to.
fn schema_meaning(root: &Value, nodes: &[&Value]) -> String {
    for n in nodes {
        let own = doc(n);
        if !own.is_empty() {
            return own;
        }
        if referenced(n).is_some() {
            let def = doc(resolve(root, n.get("allOf").and_then(|a| a.get(0)).unwrap_or(n)));
            if !def.is_empty() {
                return def;
            }
        }
    }
    String::new()
}

/// A leaf of the schema: path, type and allowed values, meaning from the schema.
#[derive(Clone, Debug, PartialEq)]
pub struct Leaf {
    pub path: String,
    pub kind: String,
    pub values: Vec<String>,
    pub meaning: String,
    /// #304: Meaning of single values from the schema (doc comment of an enum variant); values without one are missing.
    pub value_meanings: BTreeMap<String, String>,
}

fn join(path: &str, key: &str) -> String {
    if path.is_empty() {
        key.to_string()
    } else {
        format!("{path}.{key}")
    }
}

fn walk<'a>(root: &'a Value, nodes: &[&'a Value], path: &str, out: &mut Vec<Leaf>) {
    if let Some(kind) = nodes.iter().find_map(|n| series_kind(root, n)) {
        out.push(Leaf {
            path: path.to_string(),
            kind: kind.to_string(),
            values: vec![],
            meaning: schema_meaning(root, nodes),
            value_meanings: BTreeMap::new(),
        });
        return;
    }
    let alts: Vec<&Value> = nodes.iter().flat_map(|n| alternatives(root, n)).collect();
    let mut props: BTreeMap<&str, Vec<&Value>> = BTreeMap::new();
    let mut items = vec![];
    let mut types = vec![];
    let mut values = vec![];
    let mut value_meanings = BTreeMap::new();
    let mut map = false;
    for a in &alts {
        for (k, p) in a.get("properties").and_then(Value::as_object).into_iter().flatten() {
            props.entry(k.as_str()).or_default().push(p);
        }
        if let Some(i) = a.get("items").filter(|i| i.is_object()) {
            items.push(i);
        }
        map |= a.get("additionalProperties").is_some_and(Value::is_object);
        match a.get("type") {
            Some(Value::String(t)) => types.push(t.as_str()),
            Some(Value::Array(ts)) => types.extend(ts.iter().filter_map(Value::as_str)),
            _ => {}
        }
        for v in a.get("enum").and_then(Value::as_array).into_iter().flatten() {
            let v = v.as_str().map_or_else(|| v.to_string(), str::to_string);
            if !values.contains(&v) {
                values.push(v);
            }
        }
        // #304: an alternative with a single value carries the meaning of that value.
        if let Some([v]) = a.get("enum").and_then(Value::as_array).map(Vec::as_slice) {
            let text = doc(a);
            if !text.is_empty() {
                let v = v.as_str().map_or_else(|| v.to_string(), str::to_string);
                value_meanings.entry(v).or_insert(text);
            }
        }
    }
    if !props.is_empty() {
        for (k, ns) in props {
            walk(root, &ns, &join(path, k), out);
        }
        return;
    }
    let records = items
        .iter()
        .flat_map(|i| alternatives(root, i))
        .any(|a| a.get("properties").is_some());
    if records {
        walk(root, &items, &join(path, "*"), out);
        return;
    }
    let kind = if !values.is_empty() {
        "enum"
    } else if types.contains(&"array") {
        "array"
    } else if map || types.contains(&"object") {
        "object"
    } else if types.contains(&"number") {
        "number"
    } else {
        types.iter().copied().find(|t| *t != "null").unwrap_or("any")
    };
    out.push(Leaf {
        path: path.to_string(),
        kind: kind.to_string(),
        values,
        meaning: schema_meaning(root, nodes),
        value_meanings,
    });
}

/// All leaves of the schema of the plan, sorted by path. A course over time and a characteristic curve are one leaf
/// (their forms are in the section `series`), a list of records has `*` as its index, a table (list of lists) is one
/// leaf of type `array`.
pub fn schema_leaves() -> Vec<Leaf> {
    let root = plan_schema();
    let mut out = vec![];
    walk(root, &[root], "", &mut out);
    out.sort_by(|a, b| a.path.cmp(&b.path));
    out
}

/// The fields of the plan: leaves of the schema with unit and range from [`UNITS`] and the meaning from the schema or
/// from [`MEANINGS`].
pub fn fields() -> Vec<Field> {
    let units: BTreeMap<&str, (&str, &str)> = UNITS.iter().map(|&(p, u, r)| (p, (u, r))).collect();
    let meanings: BTreeMap<&str, &str> = MEANINGS.iter().copied().collect();
    schema_leaves()
        .into_iter()
        .map(|l| {
            let (unit, range) = units.get(l.path.as_str()).copied().unwrap_or(("", ""));
            let meaning = meanings
                .get(l.path.as_str())
                .map_or_else(|| short(&l.meaning), |m| m.to_string());
            Field {
                path: l.path,
                kind: l.kind,
                values: l.values,
                unit,
                range,
                meaning,
            }
        })
        .collect()
}

// ---------------------------------------------------------------------------------------------
// Sections from the schemas of metric, experiment and state variables
// ---------------------------------------------------------------------------------------------

/// Values of an enumeration with their meanings (`oneOf` of single values, or `enum`).
fn enum_items(root: &Value, def: &str) -> Vec<Item> {
    let node = &root["definitions"][def];
    let mut out = vec![];
    for alt in alternatives(root, node) {
        let meaning = if alt.get("oneOf").is_some() {
            String::new()
        } else {
            short(&doc(alt))
        };
        for v in alt.get("enum").and_then(Value::as_array).into_iter().flatten() {
            let name = v.as_str().unwrap_or_default().to_string();
            // An `enum` with several values shares no meaning (the text of the type).
            let shared = alt["enum"].as_array().is_some_and(|a| a.len() > 1) || node.get("oneOf").is_none();
            out.push(item(name, if shared { String::new() } else { meaning.clone() }));
        }
    }
    out
}

/// Properties of a definition with their meanings.
fn property_items(root: &Value, def: &Value) -> Vec<Item> {
    let def = resolve(root, def);
    def.get("properties")
        .and_then(Value::as_object)
        .into_iter()
        .flatten()
        .map(|(k, p)| item(k.clone(), short(&schema_meaning(root, &[p]))))
        .collect()
}

fn series_forms() -> SeriesForms {
    let root = plan_schema();
    let mut forms = vec![item("<number>", "constant value")];
    for alt in root["definitions"]["SeriesDef"]["oneOf"]
        .as_array()
        .into_iter()
        .flatten()
    {
        let mode = alt["properties"]["mode"]["enum"][0].as_str().unwrap_or_default();
        let keys: Vec<&str> = alt["properties"]
            .as_object()
            .into_iter()
            .flatten()
            .map(|(k, _)| k.as_str())
            .filter(|k| *k != "mode")
            .collect();
        let text = short(&doc(alt));
        let meaning = if text.is_empty() {
            format!("keys {}", keys.join(", "))
        } else {
            format!("keys {}: {text}", keys.join(", "))
        };
        forms.push(item(mode, meaning));
    }
    forms.push(item(
        "curve",
        format!(
            "keys input, points [[x, y]…], eval (yearly|monthly): {} Only fields of type series|curve.",
            short(&doc(&root["definitions"]["StateCurve"]))
        ),
    ));
    forms.extend([
        item("rows", "timed: [{\"from\": <point in time>, \"value\": v}…]"),
        item(
            "points",
            "table, linear: {\"at\": <point in time>, \"value\": v} or {\"age\": 70, \"value\": v}",
        ),
    ]);
    let points_in_time = vec![
        item(
            "<milestone id>",
            "start, end and the plan's milestones (field milestones)",
        ),
        item("YYYY-MM", "a month"),
        item(
            "age:<years>",
            "age of the first person, in a field path: wealth.stockShare@age:70",
        ),
        item(
            "@<point in time>",
            "field path to a step of a course over time: wealth.stockShare@retire (set, vary)",
        ),
    ];
    SeriesForms { forms, points_in_time }
}

fn state_items() -> Vec<Item> {
    enum_items(plan_schema(), "StateInput")
}

fn metric_blocks() -> MetricBlocks {
    let root = &crate::measure::schema();
    let aggregate = root["definitions"]["Aggregate"]["oneOf"]
        .as_array()
        .into_iter()
        .flatten()
        .map(|a| {
            let kind = a["properties"]["kind"]["enum"][0].as_str().unwrap_or_default();
            let keys: Vec<&str> = a["properties"]
                .as_object()
                .into_iter()
                .flatten()
                .map(|(k, _)| k.as_str())
                .filter(|k| *k != "kind")
                .collect();
            let text = short(&doc(a));
            let meaning = match (keys.is_empty(), text.is_empty()) {
                (true, _) => text,
                (false, true) => format!("keys {}", keys.join(", ")),
                (false, false) => format!("keys {}: {text}", keys.join(", ")),
            };
            item(kind, meaning)
        })
        .collect();
    MetricBlocks {
        shape: json!({ "name": "cons15", "quantity": "consumption", "real": true,
            "window": { "from": "retirement", "years": 15 }, "reduce": "annualSums",
            "condition": { "op": "gt", "value": 0 }, "aggregate": { "kind": "mean" } }),
        quantities: enum_items(root, "Quantity"),
        window: property_items(root, &root["definitions"]["Window"]),
        reduce: enum_items(root, "Reduce"),
        aggregate,
        ops: enum_items(root, "Op"),
        notes: vec![
            "Amounts are nominal (CHF of each month); real: true divides by the price index (prices of the start month).",
            "Window points: start, retirement (last pension fund payout), end, a milestone id or YYYY-MM.",
            "condition is for reduce firstWhen, share and any (default: greater than 0).",
            "Probabilities and means come with a standard error (stdError).",
        ],
    }
}

fn experiment_forms() -> ExperimentForms {
    let root = crate::experiment::schema();
    let defaults = json!({
        "search": crate::experiment::ExperimentSearch::default(),
        "budget": crate::experiment::ExperimentBudget::default(),
    });
    ExperimentForms {
        keys: property_items(root, root),
        vary: vec![
            item(
                "{\"path\", \"values\": [..]}",
                "list of values (numbers, texts, objects)",
            ),
            item(
                "{\"path\", \"range\": [from, to, step]}",
                "grid in steps; pattern search: the interval",
            ),
            item(
                "{\"path\", \"follows\": <path>, \"offset\": x}",
                "coupled: value of another path plus offset",
            ),
            item(
                "{\"group\": [{\"set\", \"vary\"}..]}",
                "alternatives not combined with each other (e.g. one per strategy)",
            ),
        ],
        objective: property_items(root, &root["definitions"]["ExperimentObjective"]),
        constraint: json!({ "metric": "ruin", "op": "le", "value": 0.02 }),
        methods: enum_items(root, "Method"),
        defaults,
        max_variants: crate::experiment::MAX_VARIANTS,
        confirm_seconds: crate::experiment::CONFIRM_SECONDS,
    }
}

// ---------------------------------------------------------------------------------------------
// Examples, performance, limits
// ---------------------------------------------------------------------------------------------

/// Probability of ruin (wealth depleted before the end).
fn ruin() -> Value {
    json!({ "name": "ruin", "quantity": "depleted", "reduce": "any", "aggregate": { "kind": "probability" } })
}

/// Mean real annual consumption in the first 15 years from retirement.
fn cons15() -> Value {
    json!({ "name": "cons15", "quantity": "consumption", "real": true, "window": { "from": "retirement", "years": 15 },
        "reduce": "annualSums", "aggregate": { "kind": "mean" } })
}

/// Three example experiments; they run on the example plan and, with its milestone ids, on most plans. Small budgets
/// (a few seconds); raise `budget.paths` and `budget.confirmPaths` for more precise values.
pub fn examples() -> Vec<Example> {
    let parametric = json!({ "returns.source": "parametric" });
    vec![
        Example {
            name: "spending",
            question: "How much can I spend with at most 5 % risk of ruin?",
            experiment: json!({
                "set": parametric,
                "vary": [ { "path": "flows.spendingFactor", "range": [0.8, 1.4, 0.1] } ],
                "metrics": [ ruin(), cons15() ],
                "constraints": [ { "metric": "ruin", "op": "le", "value": 0.05 } ],
                "objective": { "maximize": "cons15" },
                "budget": { "paths": 300, "confirmPaths": 1000, "top": 3 }
            }),
        },
        Example {
            name: "lumpSum",
            question: "Pension fund: pension, half or all as a lump sum? (no objective: all variants side by side)",
            experiment: json!({
                "set": parametric,
                "vary": [ { "path": "pension.pk.lumpsum", "values": [0, 0.5, 1] } ],
                "metrics": [ ruin(), cons15(),
                    { "name": "end10", "quantity": "total", "real": true, "reduce": "atEnd",
                      "aggregate": { "kind": "quantile", "q": 0.1 } } ],
                "budget": { "paths": 300 }
            }),
        },
        Example {
            name: "strategy",
            question: "Stock share before and after retirement and the withdrawal rule, ruin at most 2 %?",
            experiment: json!({
                "set": parametric,
                "vary": [
                    { "path": "wealth.stockShare@start", "values": [0.5, 0.7] },
                    { "path": "wealth.stockShare@retire", "follows": "wealth.stockShare@start", "offset": -0.2 },
                    { "group": [
                        { "set": { "withdrawal.strategy": "fixed_rate" },
                          "vary": [ { "path": "withdrawal.fixedRate.rate", "range": [0.03, 0.05, 0.01] } ] },
                        { "set": { "withdrawal.strategy": "guardrails" },
                          "vary": [ { "path": "withdrawal.guardrails.initialRate", "values": [0.04, 0.05] } ] }
                    ] }
                ],
                "metrics": [ ruin(), cons15() ],
                "constraints": [ { "metric": "ruin", "op": "le", "value": 0.02 } ],
                "objective": { "maximize": "cons15" },
                "budget": { "paths": 300, "confirmPaths": 1000, "top": 3 }
            }),
        },
    ]
}

/// Measures the microseconds per run of the plan (about half a second); `None`: the example plan.
pub fn performance(plan: Option<&Value>, base_dir: &Path, library: Option<&Path>) -> Performance {
    let example;
    let (v, which) = match plan {
        Some(p) => (p, "given"),
        None => {
            // Monte Carlo as in the examples (the example plan itself has fixed returns: one path).
            let mut e = crate::api::example_plan();
            e["returns"]["source"] = json!("parametric");
            example = e;
            (&example, "example")
        }
    };
    let t0 = Instant::now();
    let measured = crate::api::bench(v, base_dir, library);
    let measured_ms = (t0.elapsed().as_secs_f64() * 1e3).round();
    let tenth = |x: f64| (x * 10.0).round() / 10.0;
    let mut out = Performance {
        plan: which,
        us_per_run: None,
        us_generate: None,
        cores: rayon::current_num_threads(),
        months: None,
        measured_ms,
        estimate: "seconds ≈ variants × paths × (usPerRun − usGenerate) / 1e6 + paths × usGenerate / 1e6, \
                   search and confirmation; evaluate shows its own estimate",
        error: None,
    };
    match measured {
        Ok(b) => {
            out.us_per_run = Some(tenth(b.us_per_run));
            out.us_generate = Some(tenth(b.us_generate));
            out.cores = b.cores;
            out.months = Some(b.months);
        }
        Err(e) => out.error = Some(e),
    }
    out
}

/// Limits of the model (analysis, section 9).
pub const LIMITS: &[&str] = &[
    "Parametric returns are drawn independently each year: rules betting on recovery after crashes (curve over \
     drawdown) cannot show an advantage there; test them with historical paths, at the risk of fitting the past.",
    "The more you vary, the larger the selection error: the best of many variants is overestimated on its search \
     paths. Trust the confirmed values (independent paths), not the search values.",
    "Probabilities and means carry a standard error; differences between variants are more precise than single \
     values (same paths).",
    "Amounts are nominal unless a metric says real: true.",
    "Computing time: 10 000 variants × 10 000 paths take minutes; estimate first (performance), keep a budget.",
    "What the model does not know (care, inheritances, tax law beyond the tables) parametrization does not fix.",
    "Model calculation without guarantee. No tax or investment advice. Taxes are estimates.",
];

/// The catalog, all sections or one. The performance is measured on `plan` (`None`: the example plan) only if it is
/// part of the answer. #304: the section `choices` only when asked for, with the labels in `language` (de, en, fr, it;
/// another one: German).
pub fn describe(
    section: Option<Section>,
    plan: Option<&Value>,
    base_dir: &Path,
    library: Option<&Path>,
    language: &str,
) -> Catalog {
    let want = |s: Section| section.is_none_or(|x| x == s);
    Catalog {
        format: FORMAT,
        engine: env!("CARGO_PKG_VERSION"),
        sections: Section::ALL.iter().map(|s| s.name()).collect(),
        fields: want(Section::Fields).then(|| Fields::new(section == Some(Section::Fields))),
        series: want(Section::Series).then(series_forms),
        state: want(Section::State).then(state_items),
        metrics: want(Section::Metrics).then(metric_blocks),
        experiment: want(Section::Experiment).then(experiment_forms),
        examples: want(Section::Examples).then(examples),
        performance: want(Section::Performance).then(|| performance(plan, base_dir, library)),
        limits: want(Section::Limits).then(|| LIMITS.to_vec()),
        choices: (section == Some(Section::Choices)).then(|| crate::choices::choices(language)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn issue_219_ak1_short_meaning() {
        assert_eq!(short("#218: Stock share 0..1. More text."), "Stock share 0..1.");
        assert_eq!(
            short("63: Denomination for the church tax."),
            "Denomination for the church tax."
        );
        assert_eq!(short("Value 0.5 = half"), "Value 0.5 = half");
        assert_eq!(
            short("Margin p.a. on SARON (kind `saron`)."),
            "Margin p.a. on SARON (kind `saron`)."
        );
        assert_eq!(short("Share 0..1; #218: also a curve."), "Share 0..1; also a curve.");
        assert_eq!(short("Empty (#170): from the id."), "Empty: from the id.");
        let long = "word ".repeat(40);
        let s = short(&long);
        assert!(s.chars().count() <= MEANING_CHARS && s.ends_with('…'), "{s}");
    }

    #[test]
    fn issue_219_ak1_sections_by_name() {
        for s in Section::ALL {
            assert_eq!(Section::parse(s.name()), Some(s));
        }
        assert_eq!(Section::parse("nothing"), None);
    }
}
