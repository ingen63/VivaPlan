//! The data model of a plan (project file `.vplan`, JSON).
//!
//! Amounts are monthly, rates annual as decimals (0.05 = 5 %), unless noted otherwise.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::time::{TimeRef, YearMonth};

pub const FORMAT: &str = "vivaplan/1";
/// legacy: RetirementPlanner – identifier of older files; valid when reading, [`FORMAT`] is written.
pub const LEGACY_FORMAT: &str = "pensionsplaner/1";

fn default_format() -> String {
    FORMAT.to_string()
}

/// #144: Message for a format identifier this version does not know (the same text as `unknownFormatText` in
/// `crates/app/ui/util.js`).
pub fn unknown_format(format: &str) -> crate::message::Message {
    msg!(format_unknown, format = format)
}

/// #144: Checks the format identifier of a plan. Valid are [`FORMAT`], [`LEGACY_FORMAT`] and no identifier; any other
/// comes from a newer version (or is not a plan) and is rejected.
pub fn check_format(plan: &serde_json::Value) -> Result<(), crate::Error> {
    match plan.get("format") {
        None => Ok(()),
        Some(serde_json::Value::String(f)) if f == FORMAT || f == LEGACY_FORMAT => Ok(()),
        Some(serde_json::Value::String(f)) => Err(crate::Error::Invalid(unknown_format(f))),
        Some(other) => Err(crate::Error::Invalid(unknown_format(&other.to_string()))),
    }
}
fn one() -> f64 {
    1.0
}
fn is_false(b: &bool) -> bool {
    !*b
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Plan {
    #[serde(default = "default_format")]
    pub format: String,
    pub project: Project,
    pub household: Household,
    pub milestones: Vec<Milestone>,
    pub pension: Pension,
    /// Second person (double income): own income, pension fund and AHV.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub partner: Option<Partner>,
    pub flows: Flows,
    pub wealth: Wealth,
    pub returns: Returns,
    #[serde(default)]
    pub withdrawal: Withdrawal,
    pub taxes: Taxes,
    #[serde(default)]
    pub realestate: RealEstate,
    #[serde(default)]
    pub scenarios: Scenarios,
    /// Saved settings of the optimization (user interface and `optimize`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub optimization: Option<OptimizerSettings>,
    /// Pillar 3a, at most one account per person (#52).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pillar3a: Vec<Pillar3aAccount>,
    /// Application-wide limits from the settings (#25). The user interface passes them with every call; they
    /// do not belong to the plan and are never saved. If they are missing, the default values apply.
    #[serde(default, skip_serializing)]
    #[schemars(skip)]
    pub limits: crate::settings::Limits,
}

/// Goal of the optimization.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum Goal {
    /// Goal 1: consumption with a limited risk of ruin.
    #[default]
    Consumption,
    /// Goal 2: no emergency sale of properties, i.e. as few unplanned sales (emergency sales) as possible.
    KeepProperty,
}

/// #197: Which property setting the optimization and the comparison of strategies use.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum PropertyMode {
    /// As in the plan (switch «Immobilien nie verkaufen» as set there).
    #[default]
    Planned,
    /// With the switch «Immobilien nie verkaufen» on (#196).
    NeverSell,
    /// Every strategy twice: as planned and with «Immobilien nie verkaufen».
    Both,
}

impl PropertyMode {
    fn is_planned(&self) -> bool {
        *self == PropertyMode::Planned
    }
}

/// Settings of the optimization. Shares as numbers (0.05 = 5 %); `strategies` empty = all.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct OptimizerSettings {
    /// Highest allowed probability of ruin.
    pub max_ruin: f64,
    /// Goal of the optimization (default: consumption).
    pub goal: Goal,
    /// Goal «Kein Notverkauf von Immobilien»: the median annual consumption is at least this share of the planned spending.
    pub min_consumption: f64,
    /// Ruin counts only up to this age of the main person (`None` = until the end of the simulation).
    pub ruin_age: Option<f64>,
    /// Weight of the median in the consumption goal (0 to 1); the rest goes to the 10 % percentile.
    pub weight_median: f64,
    /// Penalty for the fluctuation of consumption.
    pub vol_weight: f64,
    /// Highest allowed fluctuation (`None` = no limit).
    pub max_vol: Option<f64>,
    /// Paths of the search and of the check, random starting points per strategy.
    pub paths: usize,
    pub check_paths: usize,
    pub samples: usize,
    /// Keys of the strategies (`guardrails`, `vpw` …).
    pub strategies: Vec<String>,
    /// Ranges of the parameters per strategy (key of the strategy, then of the parameter).
    /// Without an entry the default ranges apply.
    #[serde(skip_serializing_if = "std::collections::BTreeMap::is_empty")]
    pub ranges: std::collections::BTreeMap<String, std::collections::BTreeMap<String, ParamRange>>,
    /// #197: Property setting of the runs (as planned, never sell, both); not written if «as planned».
    #[serde(skip_serializing_if = "PropertyMode::is_planned")]
    pub property: PropertyMode,
}

/// Range of a parameter in the optimization, in the parameter's unit
/// (years in years, shares as numbers: 0.05 = 5 %).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase", default)]
pub struct ParamRange {
    /// The parameter stays at the plan's value and is not varied.
    pub fixed: Option<bool>,
    /// Lower and upper limit of the search; if not given, the default range.
    pub lo: Option<f64>,
    pub hi: Option<f64>,
}

impl Default for OptimizerSettings {
    fn default() -> Self {
        OptimizerSettings {
            max_ruin: 0.05,
            goal: Goal::Consumption,
            min_consumption: 0.9,
            ruin_age: None,
            weight_median: 0.5,
            vol_weight: 1.0,
            max_vol: None,
            paths: 5000,
            check_paths: 10_000,
            samples: 30,
            strategies: vec![],
            ranges: Default::default(),
            property: PropertyMode::Planned,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub name: String,
    #[serde(default = "default_currency")]
    pub currency: String,
}
fn default_currency() -> String {
    "CHF".into()
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Household {
    /// First person = main person (`pension`, `flows.income`), second person = `partner`.
    pub persons: Vec<Person>,
    /// Married: rules for couples (AHV cap, tax tables for married couples).
    #[serde(default = "yes")]
    pub married: bool,
    /// Cap on the two AHV pensions together per month (150 % of the maximum pension).
    #[serde(default = "default_couple_cap")]
    pub ahv_couple_cap: f64,
}

fn yes() -> bool {
    true
}
/// 150 % of the AHV maximum pension of 2 520 CHF (as of 2025).
fn default_couple_cap() -> f64 {
    3780.0
}

/// Income and pension of the second person; ages refer to `household.persons[1]`.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Partner {
    /// Net earned income per month (taxable).
    #[serde(default)]
    pub income: Series,
    /// #58: Earned income rises with inflation (amounts in prices of the simulation start); missing: no.
    #[serde(default, skip_serializing_if = "is_false")]
    pub income_indexed: bool,
    pub pk: PensionFund,
    pub ahv: Ahv,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Person {
    pub id: String,
    pub name: String,
    pub birth: YearMonth,
    /// #152: Gainfully employed without a pension fund (large pillar 3a): contributions and deduction up to 20 % of the
    /// earned income, at most `limits.pillar3aMaxLarge`. Missing: no.
    #[serde(default, skip_serializing_if = "is_false")]
    pub no_pension_fund: bool,
}

/// German name of the milestone `retire` as older plans stored it (formerly «Pensionierung», #51); only for the
/// upgrade, the engine names it in English (#176).
pub const PK_NAME: &str = "Bezug Pensionskasse";
/// German name of the milestone `ahv` as older plans stored it (upgrade only).
pub const AHV_NAME: &str = "AHV-Bezug";

/// Fixed milestones of a person (#51): ids for «Bezug Pensionskasse» and «AHV-Bezug». First person `retire`
/// and `ahv`, second person `retire2` and `ahv2`. They always exist and cannot be deleted.
pub fn person_milestones(index: usize) -> (&'static str, &'static str) {
    if index == 0 {
        ("retire", "ahv")
    } else {
        ("retire2", "ahv2")
    }
}

/// German name of a fixed milestone of person `index` as older plans stored it (second person with a suffix;
/// upgrade only).
pub fn person_milestone_name(base: &str, index: usize) -> String {
    if index == 0 {
        base.to_string()
    } else {
        format!("{base} Person 2")
    }
}

/// German name of the milestone `start` as older plans stored it (upgrade only).
pub const START_NAME: &str = "Simulationsstart";
/// German name of the milestone `end` as older plans stored it (upgrade only).
pub const END_NAME: &str = "Simulationsende";

/// #170: German default name of a fixed milestone as older plans stored it (`start`, `end`, `retire`, `ahv`,
/// `retire2`, `ahv2`); the upgrade removes it.
pub fn stored_milestone_name(id: &str) -> Option<String> {
    match id {
        "start" => Some(START_NAME.to_string()),
        "end" => Some(END_NAME.to_string()),
        _ => (0..2).find_map(|i| {
            let (pk, ahv) = person_milestones(i);
            (id == pk)
                .then(|| person_milestone_name(PK_NAME, i))
                .or_else(|| (id == ahv).then(|| person_milestone_name(AHV_NAME, i)))
        }),
    }
}

/// #176: Default name of a fixed milestone in the texts of the engine (English, the text of
/// [`fixed_milestone_message`]). Plans store no name for them; the user interface shows the name from its text
/// table by the id.
pub fn fixed_milestone_name(id: &str) -> Option<String> {
    fixed_milestone_message(id).map(|m| m.to_string())
}

/// #171: Name of a fixed milestone as a message (the user interface translates it).
pub fn fixed_milestone_message(id: &str) -> Option<crate::message::Message> {
    Some(match id {
        "start" => msg!(milestone_start),
        "end" => msg!(milestone_end),
        "retire" => msg!(milestone_retire),
        "ahv" => msg!(milestone_ahv),
        "retire2" => msg!(milestone_retire2),
        "ahv2" => msg!(milestone_ahv2),
        _ => return None,
    })
}

/// Named point in time. `start` and `end` must exist.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Milestone {
    pub id: String,
    /// Empty for the fixed milestones (#170): their name comes from the id, see [`Milestone::label`].
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub name: String,
    #[serde(flatten)]
    pub kind: MilestoneKind,
    #[serde(default, skip_serializing_if = "is_false")]
    pub fixed: bool,
}

impl Milestone {
    /// Name in the texts of the engine: the stored name, for a fixed milestone without a name its default name
    /// (#170, English since #176), otherwise the id.
    pub fn label(&self) -> std::borrow::Cow<'_, str> {
        if !self.name.is_empty() {
            return std::borrow::Cow::Borrowed(&self.name);
        }
        fixed_milestone_name(&self.id).map_or(std::borrow::Cow::Borrowed(self.id.as_str()), std::borrow::Cow::Owned)
    }

    /// #171: Name as a parameter of a message: the name entered, otherwise the fixed name as a message.
    pub fn name_arg(&self) -> crate::message::Arg {
        match fixed_milestone_message(&self.id) {
            Some(m) if self.name.is_empty() => crate::message::arg(&m),
            _ => crate::message::arg(&*self.label()),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum MilestoneKind {
    /// Fixes Datum.
    Date { date: YearMonth },
    /// Age of a person (decimal years, 64.5 = 64 years 6 months).
    Age { person: String, years: f64 },
}

/// A value that is constant or changes from certain points in time.
/// A plain number counts as a constant value.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum Series {
    Number(f64),
    Def(SeriesDef),
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "mode", rename_all = "lowercase")]
pub enum SeriesDef {
    Const { value: f64 },
    Timed { rows: Vec<SeriesRow> },
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct SeriesRow {
    pub from: TimeRef,
    pub value: f64,
}

impl Default for Series {
    fn default() -> Self {
        Series::Number(0.0)
    }
}

impl Series {
    pub fn constant(v: f64) -> Self {
        Series::Def(SeriesDef::Const { value: v })
    }
    pub fn timed(rows: Vec<(TimeRef, f64)>) -> Self {
        Series::Def(SeriesDef::Timed {
            rows: rows
                .into_iter()
                .map(|(from, value)| SeriesRow { from, value })
                .collect(),
        })
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Pension {
    pub pk: PensionFund,
    pub ahv: Ahv,
}

/// Pension fund (2nd pillar).
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PensionFund {
    /// Retirement capital at the start of the simulation.
    pub capital: f64,
    /// Point in time of the payout (pension and capital).
    pub at: TimeRef,
    /// Lump-sum share 0..1.
    pub lumpsum: f64,
    /// Conversion rate for the pension share.
    pub conversion: f64,
    /// Contributions per month until the payout.
    #[serde(default)]
    pub contribution: Series,
    /// Interest per year, credited in December.
    #[serde(default)]
    pub interest: Series,
}

/// AHV (1st pillar).
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Ahv {
    /// Monthly pension at the start of payout.
    pub monthly: f64,
    pub at: TimeRef,
    /// Annual adjustment to inflation (in December).
    #[serde(default)]
    pub indexed: bool,
    /// #190: The plan knows the 13th AHV pension (marker of [`crate::upgrade`] for the note when opening an older
    /// plan; no switch: the simulation always pays the 13th pension from December 2026).
    #[serde(default, skip_serializing_if = "is_false")]
    pub thirteenth_noted: bool,
    /// #191: `monthly` (person and partner) is the pension at the reference age 65; the simulation reduces or increases
    /// it itself (marker of [`crate::upgrade`]; older plans held the already adjusted pension).
    #[serde(default, skip_serializing_if = "is_false")]
    pub reference_age: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Flows {
    /// Net earned income per month (taxable).
    #[serde(default)]
    pub income: Series,
    /// #58: Earned income rises with inflation (amounts in prices of the simulation start); missing: no.
    #[serde(default, skip_serializing_if = "is_false")]
    pub income_indexed: bool,
    /// Living costs per month in prices at the simulation start.
    pub spending: Series,
    /// #53: Living costs include rent or housing costs. The simulation deducts the housing share
    /// (housing costs in the start month, see [`crate::housing`]) and computes with the simulated housing costs. If the
    /// field is missing (older plans), the living costs apply without housing costs.
    #[serde(default, skip_serializing_if = "is_false")]
    pub spending_includes_housing: bool,
    /// Factor on the living costs without housing (variation «Ausgaben-Faktor»); missing: 1.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub spending_factor: Option<f64>,
    /// Further inflows per month (not taxable).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub savings: Option<Series>,
    #[serde(default)]
    pub extras: Vec<Extra>,
}

/// One-off amount; positive = inflow.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Extra {
    pub date: YearMonth,
    pub amount: f64,
    #[serde(default)]
    pub label: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Wealth {
    pub stocks: f64,
    pub bonds: f64,
    pub cash: f64,
    /// Stock share of the invested wealth 0..1.
    pub stock_share: Series,
    /// Cash reserve in months of uncovered spending.
    #[serde(default)]
    pub reserve_months: Series,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ReturnSource {
    Fixed,
    Historical,
    Rolling,
    Bootstrap,
    Parametric,
}

/// #69: Index for the stocks of the investment strategy.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum StockIndex {
    /// Swiss stocks (JST), annual from 1900.
    ChJst,
    /// World stock index (JST, weighted by GDP), annual from 1900.
    WorldJst,
    /// World with custom weights of the regions (JST), annual from 1900.
    WorldCustom,
    /// S&P 500 (Shiller) in CHF, monthly from 1922.
    Sp500,
}

/// #69: Index for the bonds of the investment strategy.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum BondIndex {
    /// 10-year Swiss Confederation bonds (JST until 1988, SNB afterwards).
    Ch,
    /// 10-year US Treasury bonds (Shiller) in CHF.
    Usa,
    /// #76: short-term Swiss Confederation paper (about 3 months): JST until 2020, SARON afterwards.
    ChShort,
}

/// #69: Weights of the regions for «Welt individuell» (shares, together 1).
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RegionWeights {
    pub ch: f64,
    pub usa: f64,
    /// Germany, France, Italy, Spain, Netherlands, Belgium, United Kingdom (weighted by GDP among themselves).
    pub eu: f64,
    /// Japan and Australia (weighted by GDP among themselves).
    pub apac: f64,
}

impl Default for RegionWeights {
    fn default() -> Self {
        RegionWeights {
            ch: 0.2,
            usa: 0.4,
            eu: 0.3,
            apac: 0.1,
        }
    }
}

/// #69: Investment strategy: where the returns of stocks and bonds come from.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Investment {
    pub stocks: StockIndex,
    pub bonds: BondIndex,
    /// Only for [`StockIndex::WorldCustom`].
    #[serde(default)]
    pub weights: RegionWeights,
}

impl Default for Investment {
    fn default() -> Self {
        Investment {
            stocks: StockIndex::WorldJst,
            bonds: BondIndex::Ch,
            weights: RegionWeights::default(),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Returns {
    pub source: ReturnSource,
    pub fixed: FixedReturns,
    /// #69: Investment strategy; applies if `series` is missing.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub strategy: Option<Investment>,
    /// Series (path to the CSV file, relative to the project file); older plans (before #69).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub series: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub saron_series: Option<String>,
    /// Historical start month (source `historical`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_date: Option<YearMonth>,
    /// Extrapolation if the series ends before the end of the simulation.
    #[serde(default)]
    pub extrapolation: Extrapolation,
    /// Monte Carlo from history (source `bootstrap`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bootstrap: Option<BootstrapParams>,
    /// Monte Carlo with a stock model (source `parametric`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parametric: Option<ParametricParams>,
}

fn default_paths() -> usize {
    10_000
}
fn default_seed() -> u64 {
    42
}
fn default_block() -> f64 {
    12.0
}

/// Stationary bootstrap: blocks of random length from the series.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct BootstrapParams {
    #[serde(default = "default_paths")]
    pub paths: usize,
    /// Mean block length in months.
    #[serde(default = "default_block")]
    pub block: f64,
    /// Seed of the random number generator (same value ⇒ same paths).
    #[serde(default = "default_seed")]
    pub seed: u64,
}

impl Default for BootstrapParams {
    fn default() -> Self {
        BootstrapParams {
            paths: default_paths(),
            block: default_block(),
            seed: default_seed(),
        }
    }
}

/// Stock model of the previous program: Student-t shocks and price jumps, daily steps. With a series
/// (investment strategy) bonds and inflation are random and correlated with the stocks (#101), otherwise they follow
/// the fixed values like SARON.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ParametricParams {
    #[serde(default = "default_paths")]
    pub paths: usize,
    /// Expected annual return (drift).
    pub mu: f64,
    /// Annual volatility.
    pub sigma: f64,
    /// Degrees of freedom of the Student-t distribution (0 = normal distribution).
    pub df: f64,
    /// Expected number of price jumps per year.
    #[serde(default)]
    pub jump_lambda: f64,
    /// Mean logarithmic jump size (−0.3 ≈ −26 %).
    #[serde(default)]
    pub jump_mu: f64,
    #[serde(default)]
    pub jump_sigma: f64,
    #[serde(default = "default_seed")]
    pub seed: u64,
    /// #100: The model parameters follow the suggested values of the stock index (not changed by hand). If the
    /// flag is missing (older plans), the saved values apply.
    #[serde(default)]
    pub auto: bool,
    /// #132: Sampling method with which the plan was last saved. If it is missing (0), the plan dates
    /// from before #132, and `upgrade` mentions the changed computation on opening. The computation always uses the current
    /// method [`crate::montecarlo::SAMPLER`].
    #[serde(default)]
    pub sampler: u32,
}

impl Default for ParametricParams {
    fn default() -> Self {
        ParametricParams {
            paths: default_paths(),
            mu: 0.07,
            sigma: 0.15,
            df: 6.0,
            jump_lambda: 0.05,
            jump_mu: -0.3,
            jump_sigma: 0.15,
            seed: default_seed(),
            auto: false,
            sampler: crate::montecarlo::SAMPLER,
        }
    }
}

/// Annual values. In `fixed` mode the returns; otherwise substitute values (e.g. SARON without a series).
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FixedReturns {
    pub stocks: Series,
    pub bonds: Series,
    pub inflation: Series,
    #[serde(default)]
    pub saron: Series,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Extrapolation {
    pub method: ExtrapolationMethod,
    /// Number of years for averages.
    pub range: usize,
}

impl Default for Extrapolation {
    fn default() -> Self {
        Extrapolation {
            method: ExtrapolationMethod::Fixed,
            range: 30,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ExtrapolationMethod {
    /// Fixed values from `returns.fixed`.
    Fixed,
    /// Moving arithmetic average.
    Average,
    /// Moving geometric average.
    Geometric,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Withdrawal {
    #[serde(default)]
    pub strategy: Strategy,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub buckets: Option<Buckets>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fixed_rate: Option<FixedRate>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub guardrails: Option<Guardrails>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vpw: Option<Vpw>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub endowment: Option<Endowment>,
    /// Floor of the living costs (share of the planned spending, 0 = none); only guardrails and fixed rate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub floor: Option<f64>,
    /// Ceiling of the living costs (share of the planned spending, 0 = none).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ceiling: Option<f64>,
}

/// Fixed withdrawal rate: from retirement, every January this share of liquid wealth is
/// spent (living costs + housing costs − income and pensions, before taxes).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct FixedRate {
    pub rate: f64,
}

impl Default for FixedRate {
    fn default() -> Self {
        FixedRate { rate: 0.04 }
    }
}

/// Guardrails after Guyton-Klinger: initial withdrawal in percent of wealth at retirement, afterwards
/// adjusted annually with inflation. If the current rate exceeds the initial rate by `upper`, it is
/// cut by `adjust`; if it falls below the initial rate by `lower`, it is raised by `adjust`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Guardrails {
    pub initial_rate: f64,
    /// Upper guardrail: relative upward deviation of the rate (0.2 = 20 %).
    pub upper: f64,
    pub lower: f64,
    /// Adjustment of spending when a guardrail is reached (0.1 = 10 %).
    pub adjust: f64,
}

impl Default for Guardrails {
    fn default() -> Self {
        Guardrails {
            initial_rate: 0.05,
            upper: 0.2,
            lower: 0.2,
            adjust: 0.1,
        }
    }
}

/// Variable percentage withdrawal (VPW): every January the share of liquid wealth is withdrawn
/// that uses it up evenly by the planning age (annuity with the assumed real return).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Vpw {
    /// Age of the main person up to which wealth should last.
    pub horizon_age: f64,
    /// Assumed real annual return of wealth (0.02 = 2 %).
    pub real_return: f64,
}

impl Default for Vpw {
    fn default() -> Self {
        Vpw {
            horizon_age: 100.0,
            real_return: 0.02,
        }
    }
}

/// Endowment rule (Yale/Tobin): withdrawal = (1 − `smoothing`) × previous year's withdrawal with inflation
/// + `smoothing` × `rate` × current wealth. Smooths fluctuations of wealth.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Endowment {
    pub rate: f64,
    /// Weight of current wealth (0 = previous year with inflation only, 1 = fixed rate).
    pub smoothing: f64,
}

impl Default for Endowment {
    fn default() -> Self {
        Endowment {
            rate: 0.04,
            smoothing: 0.3,
        }
    }
}

/// 3-bucket strategy from retirement. Targets in years of uncovered spending (spending − income).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Buckets {
    /// Bucket 1: cash.
    pub cash_years: f64,
    /// Bucket 2: bonds.
    pub bond_years: f64,
    /// Return criterion: in the January refill bucket 3 is sold only if the stock return of the last 12 months is at least this high.
    #[serde(default)]
    pub refill_threshold: f64,
    /// Refill of bucket 1 during the year (case C): after the month's expenses at least this many monthly needs
    /// minus one remain in bucket 1 (at most the months until January).
    #[serde(default = "default_reserve_months")]
    pub reserve_months: f64,
    /// #6: Spending from retirement as planned or steered by guardrails (parameters from `withdrawal.guardrails`,
    /// limits from `withdrawal.floor` and `withdrawal.ceiling`).
    #[serde(default, skip_serializing_if = "BucketSpending::is_planned")]
    pub spending: BucketSpending,
}

/// Spending rule of the 3-bucket strategy (#6).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum BucketSpending {
    /// Planned spending.
    #[default]
    Planned,
    /// Guardrails after Guyton-Klinger.
    Guardrails,
}

impl BucketSpending {
    pub fn is_planned(&self) -> bool {
        *self == BucketSpending::Planned
    }
}

fn default_reserve_months() -> f64 {
    3.0
}

impl Default for Buckets {
    fn default() -> Self {
        Buckets {
            cash_years: 2.0,
            bond_years: 6.0,
            refill_threshold: 0.0,
            reserve_months: 3.0,
            spending: BucketSpending::Planned,
        }
    }
}

impl Withdrawal {
    /// #6: 3-bucket strategy whose spending is steered by the guardrails.
    pub fn buckets_with_guardrails(&self) -> bool {
        self.strategy == Strategy::ThreeBuckets
            && self
                .buckets
                .as_ref()
                .is_some_and(|b| b.spending == BucketSpending::Guardrails)
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Strategy {
    /// Cash reserve, the rest invested by stock share, monthly rebalancing.
    #[default]
    Standard,
    ThreeBuckets,
    Guardrails,
    FixedRate,
    Vpw,
    Endowment,
}

/// Tax tables: pairs `[from amount, rate]`, interpolated linearly, multiplied by `multiplier`.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Taxes {
    /// Tax multiplier (canton + municipality).
    #[serde(default = "one")]
    pub multiplier: f64,
    pub income: Vec<(f64, f64)>,
    pub wealth: Vec<(f64, f64)>,
    /// Lump sums (PK, 3a); payouts in the same year are added together.
    pub capital: Vec<(f64, f64)>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub property_gain: Option<PropertyGainTax>,
    /// Tables for married couples; apply if the plan has a second person and `household.married`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub couple: Option<TaxTables>,
    /// #66: The amounts of the tables for income, wealth and lump sums grow with the cumulative simulated inflation
    /// since the simulation start (compensation of bracket creep); the rates stay the same. Missing: no.
    #[serde(default, skip_serializing_if = "is_false")]
    pub indexed: bool,
    /// #63: Source of the income and wealth tax: own tables (default, not written) or the place of residence.
    #[serde(default, skip_serializing_if = "TaxMode::is_own")]
    pub mode: TaxMode,
    /// #63: Place of residence for the mode `location`; missing details are reported by the validation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<TaxLocation>,
    /// #151: Further places of residence in the mode `location`, each from its point in time until the next row;
    /// `location` is the first row (from the start). Missing: one place of residence for the whole plan.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub relocations: Vec<Relocation>,
    /// #65: In the mode «Wohnort», own rules for the property gain tax (`property_gain`) instead of the rule set of
    /// the canton in which the property lies. Missing: no.
    #[serde(default, skip_serializing_if = "is_false")]
    pub property_gain_own: bool,
    /// #131: Deduct the contributions to pillar 3a of a year from the taxable income, at most the maximum per person
    /// (settings, `limits.pillar3aMax`). Missing: no.
    #[serde(default, skip_serializing_if = "is_false")]
    pub deduct_pillar3a: bool,
    /// #131: Deductions from income, time-dependent (a row applies from its point in time until the next row).
    /// Missing: as before #131 – in the mode «Wohnort» the proposal of the canton throughout, with own tables none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub deductions: Option<Vec<IncomeDeduction>>,
    /// #230: One-off deductions from income: each counts in its tax year in addition to the row that applies.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub one_off_deductions: Vec<OneOffDeduction>,
    /// #231: Deductions from taxable wealth, time-dependent (a row applies from its point in time until the next row;
    /// the row of 31 December counts). In the mode «Wohnort» in addition to the social deductions of the canton in
    /// the wealth curves; with own tables the whole deduction. Missing: none.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub wealth_deductions: Vec<WealthDeduction>,
    /// #231: One-off deductions from taxable wealth: each counts on 31 December of its tax year (nominal).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub one_off_wealth_deductions: Vec<OneOffDeduction>,
}

/// #231: Row of the deductions from wealth: amount per year in prices of the simulation start (grows with «tariffs
/// with inflation», #66), for example debts other than the mortgage.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct WealthDeduction {
    pub from: TimeRef,
    #[serde(default)]
    pub amount: f64,
}

/// #230: One-off deduction from income in a tax year, nominal (for example a pension fund buy-in or a renovation).
/// #231: also from wealth (on 31 December of the tax year).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct OneOffDeduction {
    pub year: i32,
    #[serde(default)]
    pub label: String,
    #[serde(default)]
    pub amount: f64,
}

/// #131: Origin of a row of the deductions from income.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum DeductionKind {
    /// Proposal of the canton: the deduction rules of the tax data file (#63), only in the mode «Wohnort».
    Canton,
    /// Own value: `amount` per year plus `rate` × income, for canton and federal tax.
    #[default]
    Own,
}

/// #131: Row of the deductions from income.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct IncomeDeduction {
    pub from: TimeRef,
    #[serde(default)]
    pub kind: DeductionKind,
    /// Own value: amount per year in prices of the simulation start (grows with «tariffs with inflation», #66).
    #[serde(default)]
    pub amount: f64,
    /// Own value: additionally this share of the income (0.1 = 10 %).
    #[serde(default)]
    pub rate: f64,
    /// #230: Proposal of the canton: additional amount per year in prices of the simulation start (grows with «tariffs
    /// with inflation», #66), for canton and federal tax. `amount` is not reused, old plans may hold values there.
    #[serde(default, skip_serializing_if = "is_zero")]
    pub extra: f64,
}

/// #63: Source of the income and wealth tax.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum TaxMode {
    /// Own tables with tax multiplier.
    #[default]
    Own,
    /// Canton, municipality and church tax; tariffs, deductions and multipliers from the tax data file.
    Location,
}

impl TaxMode {
    pub fn is_own(&self) -> bool {
        *self == TaxMode::Own
    }
}

/// #63: Denomination for the church tax.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum Church {
    #[default]
    None,
    Reformed,
    Catholic,
    ChristCatholic,
}

/// #63: Place of residence for the tax.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TaxLocation {
    /// Canton abbreviation, for example «ZH».
    #[serde(default)]
    pub canton: String,
    /// BFS number of the municipality.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub municipality: Option<u32>,
    #[serde(default)]
    pub church: Church,
}

/// #151: Change of the place of residence: from `from` the place `place` applies (until the next row).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Relocation {
    pub from: TimeRef,
    #[serde(flatten)]
    pub place: TaxLocation,
}

/// #63: Rule of a deduction from the income: `base + rate × reference`, at least `min`, at most `max` (0 = no
/// maximum). The reference is the income before deductions, for the flat deduction for professional expenses the
/// earned income. Amounts in prices of the tax year of the data; with «tariffs with inflation» (#66) they grow with
/// the price index.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct DeductionRule {
    #[serde(default)]
    pub base: f64,
    #[serde(default)]
    pub rate: f64,
    #[serde(default)]
    pub min: f64,
    #[serde(default)]
    pub max: f64,
}

impl DeductionRule {
    /// Deduction for the reference amount `reference`; `index` scales the amounts (1.0 = prices of the data).
    pub fn amount(&self, reference: f64, index: f64) -> f64 {
        let v = (self.base * index + self.rate * reference).max(self.min * index);
        if self.max > 0.0 {
            v.min(self.max * index)
        } else {
            v
        }
    }

    /// Flat deduction for professional expenses on the earned income of a year: no deduction without earned income,
    /// at most the earned income.
    pub fn earned_amount(&self, earned: f64, index: f64) -> f64 {
        if earned <= 0.0 {
            return 0.0;
        }
        self.amount(earned, index).min(earned)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TaxTables {
    pub income: Vec<(f64, f64)>,
    pub wealth: Vec<(f64, f64)>,
    pub capital: Vec<(f64, f64)>,
}

/// Model of the property gain tax.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum GainModel {
    /// Bracket tariff on the gain, then surcharge or reduction by holding period (canton of Zurich).
    #[default]
    Zurich,
    /// Direct tax rate on the gain by holding period (canton of Aargau, § 109 StG).
    Aargau,
    /// #65: Rate by the amount of the gain, interpolated linearly between the points of `rate_curve`
    /// (canton BL); the rate applies to the whole gain.
    Curve,
    /// #65: Rate = return per year (gain / investment costs / years held), limited by `return_bounds` (Zug).
    Return,
}

/// Property gain tax. Zurich model: brackets `[width of the bracket, rate]`, then surcharge or
/// reduction by holding period. Aargau model: rate per holding period. #65: models «curve» and «return» and
/// supplements for the rule sets of all cantons (tax data file); the order of the computation is described at
/// [`crate::tax::property_gain_tax_full`]. The rules differ per canton.
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct PropertyGainTax {
    #[serde(default)]
    pub model: GainModel,
    /// Aargau model: pairs `[from full years, tax rate on the gain]`. The last row whose years are reached
    /// applies. Without a purchase date the highest rate applies.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub duration_rates: Vec<(f64, f64)>,
    /// Additionally multiply the tax by the tax multiplier (`taxes.multiplier`).
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub use_multiplier: bool,
    #[serde(default)]
    pub brackets: Vec<(f64, f64)>,
    /// Fixed reduction of the tax, independent of the holding period (0.5 = 50 %).
    #[serde(default)]
    pub reduction: f64,
    /// Surcharges and reductions by holding period: pairs `[from full years, change of the tax]`
    /// (0.5 = surcharge of 50 %, −0.2 = reduction of 20 %). The last row whose years are reached
    /// applies. Without a purchase date of the property nothing is adjusted.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub duration: Vec<(f64, f64)>,
    /// Gains below this amount stay tax-free (0 = no exemption limit).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub exempt_below: f64,
    /// #65: Allowance, always deducted from the gain (0 = none).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub allowance: f64,
    /// #65: From this gain one rate applies to the whole gain: `[from gain, rate]` (Zurich model).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uniform_above: Option<(f64, f64)>,
    /// #65: Model «curve»: pairs `[gain, rate]`; the rate applies to the whole gain.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rate_curve: Vec<(f64, f64)>,
    /// #65: Model «return»: lowest and highest rate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub return_bounds: Option<(f64, f64)>,
    /// #65: Model «return»: highest rate by holding period, pairs `[from years, highest rate]` (replaces the highest
    /// rate of `return_bounds`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub return_max_duration: Vec<(f64, f64)>,
    /// #65: Change of the gain by holding period: pairs `[from years, change]` (−0.1 = gain 10 % lower).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub gain_duration: Vec<(f64, f64)>,
    /// #65: Maximum burden as a share of the gain (0 = none).
    #[serde(default, skip_serializing_if = "is_zero")]
    pub max_share: f64,
    /// #65: Surcharge on the part of the gain above a threshold for a short holding period:
    /// `[threshold, surcharge on the tax, holding period in years]` (Fribourg: 400 000, 0.4, 5).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub short_surcharge: Option<(f64, f64, f64)>,
    /// #65: From this holding period in years other brackets apply (Zurich model): `(from years, brackets)` (Valais).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub long_brackets: Option<(f64, Vec<(f64, f64)>)>,
    /// #65: Surcharges and reductions for the part of the gain above a threshold: `(threshold, rows as in
    /// `duration`)`. The tax is split in proportion to the parts of the gain (St. Gallen: reduction up to 500 000
    /// francs different from above).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub duration_above: Option<(f64, Vec<(f64, f64)>)>,
}

fn is_zero(v: &f64) -> bool {
    *v == 0.0
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct RealEstate {
    /// #196: Never sell a property: no planned sale, no emergency sale, no sale for lack of affordability at a
    /// renewal (the mortgage is renewed in full). Purchases and rents «nach Verkauf» do not take place.
    #[serde(default, skip_serializing_if = "is_false")]
    pub never_sell: bool,
    /// Emergency sale if liquid wealth no longer covers the uncovered spending of this number of months.
    #[serde(default)]
    pub emergency_months: Series,
    #[serde(default)]
    pub properties: Vec<Property>,
    /// Affordability for purchases «nach Verkauf».
    #[serde(default)]
    pub affordability: Affordability,
}

/// Banks' affordability calculation: (imputed rate × mortgage + fix costs) at most
/// `sustainability` × annual income (earned income and pensions, without withdrawal from wealth); mortgage at most 80 %.
/// If that is not enough, the mortgage is reduced (one-off repayment from wealth).
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Affordability {
    pub sustainability: f64,
    pub imputed_rate: f64,
    /// Fix costs per year in percent of the purchase price.
    pub fix_costs: f64,
    /// No longer used (affordability only uses income); kept for older files.
    pub wealth_contribution: f64,
    pub max_loan_to_value: f64,
}

impl Default for Affordability {
    fn default() -> Self {
        Affordability {
            sustainability: 1.0 / 3.0,
            imputed_rate: 0.05,
            fix_costs: 0.01,
            wealth_contribution: 0.03,
            max_loan_to_value: 0.8,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum PropertyStatus {
    Owned,
    Planned,
    Rent,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Property {
    /// Unique id, so that other objects can refer to it («nach Verkauf von …»).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub name: String,
    pub status: PropertyStatus,
    /// Purchase price (owned: price paid, for the property gain tax).
    #[serde(default)]
    pub price: f64,
    /// Market value today (owned only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub worth: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub buy: Option<TimeRef>,
    /// Purchase date (owned): determines the holding period for the property gain tax.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bought: Option<YearMonth>,
    /// #65: Canton in which the property lies, for the property gain tax in the mode «Wohnort» (missing or empty:
    /// canton of residence).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub canton: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sell: Option<TimeRef>,
    /// Rent from (status `rent`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from: Option<TimeRef>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub until: Option<TimeRef>,
    /// Rent per month in prices at the simulation start.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rent: Option<f64>,
    #[serde(default)]
    pub fix_costs: f64,
    /// Adjust value and fix costs with inflation (older field; applies where the individual choice is missing).
    #[serde(default)]
    pub inflation: bool,
    /// Adjust the value with inflation (missing: `inflation`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inflation_value: Option<bool>,
    /// Adjust fix costs with inflation (missing: `inflation`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inflation_fix_costs: Option<bool>,
    /// Adjust rent with inflation (missing: yes).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inflation_rent: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub mortgage: Option<Mortgage>,
    /// #229: Tax value for the wealth tax (owned and planned only); missing: the market value.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tax_value: Option<TaxValue>,
}

/// #229: How the tax value of a property is entered.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum TaxValueKind {
    /// Amount in francs, relative to the market value today (owned) or the purchase price (planned).
    #[default]
    Amount,
    /// Share of the market value (0.65 = 65 %).
    Rate,
}

/// #229: Tax value of a property (wealth tax value). It keeps its ratio to the market value: if the market value
/// grows with inflation, the tax value grows in the same ratio. Only the wealth tax uses it.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TaxValue {
    #[serde(default)]
    pub kind: TaxValueKind,
    /// Amount in francs (`amount`) or share (`rate`).
    pub value: f64,
}

impl TaxValue {
    /// Ratio of the tax value to the market value; `base` is the market value today (owned) or the purchase price
    /// (planned). An amount on a base of 0 has no ratio; the market value then applies.
    pub fn ratio(&self, base: f64) -> f64 {
        match self.kind {
            TaxValueKind::Rate => self.value,
            TaxValueKind::Amount if base > 0.0 => self.value / base,
            TaxValueKind::Amount => 1.0,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Mortgage {
    /// Mortgage at purchase: entered amounts or maximum loan-to-value (minimum equity).
    #[serde(default)]
    pub amount_mode: AmountMode,
    pub tranches: Vec<Tranche>,
    /// Deprecated (before #52): pillar 3a on the mortgage. Taken over into [`Plan::pillar3a`] when loading
    /// (see [`crate::upgrade`]) and not used afterwards.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pillar3a: Option<Pillar3a>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub enum AmountMode {
    /// Amounts of the tranches as entered.
    #[default]
    Fixed,
    /// Maximum loan-to-value (`affordability.maxLoanToValue`, i.e. minimum equity), reduced
    /// if not affordable. The amounts of the tranches then count as shares.
    MaxLoan,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum AfterLast {
    /// Renew the last period on the same terms.
    #[default]
    Renew,
    /// Repay the remaining debt from wealth.
    Repay,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Tranche {
    #[serde(default)]
    pub name: String,
    /// Amount today (owned) or at purchase (planned).
    pub amount: f64,
    /// Start of the current period (owned only).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start: Option<YearMonth>,
    #[serde(default)]
    pub after: AfterLast,
    pub periods: Vec<Period>,
    /// Person (id) whose pillar 3a receives the indirect amortization (#52); missing: first person.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub pillar3a: Option<String>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum RateKind {
    #[default]
    Fixed,
    Saron,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum AmortMode {
    #[default]
    Direct,
    /// Contribution into pillar 3a instead of reducing the debt.
    Indirect,
}

#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Period {
    #[serde(rename = "type", default)]
    pub kind: RateKind,
    /// Term in years.
    pub term: f64,
    /// Fixed interest p.a. (kind `fixed`).
    #[serde(default)]
    pub rate: f64,
    /// Margin p.a. on SARON (kind `saron`).
    #[serde(default)]
    pub margin: f64,
    /// Amortization per quarter (due March, June, September, December).
    #[serde(default)]
    pub amort: f64,
    #[serde(default)]
    pub amort_mode: AmortMode,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum PayoutUse {
    #[default]
    Repay,
    Wealth,
}

/// Deprecated pillar 3a on the mortgage (before #52); only for loading older files.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Pillar3a {
    pub balance: f64,
    pub rate: f64,
    pub payout: TimeRef,
    #[serde(rename = "use", default)]
    pub usage: PayoutUse,
}

fn wealth_use() -> PayoutUse {
    PayoutUse::Wealth
}

/// Pillar 3a of a person (#52): balance at the simulation start, contribution and payout plan. The remainder is
/// paid out at the latest in the month of the person's milestone «AHV-Bezug».
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Pillar3aAccount {
    /// Person (id from `household.persons`).
    pub person: String,
    /// Balance at the simulation start.
    #[serde(default)]
    pub balance: f64,
    /// Interest or return per year.
    #[serde(default)]
    pub rate: f64,
    /// Contribution per year (0 to the maximum), monthly one twelfth, until before the AHV payout. Counts as spending.
    #[serde(default)]
    pub contribution: f64,
    /// Planned payouts, at the earliest 5 years before the AHV payout.
    #[serde(default)]
    pub payouts: Vec<Pillar3aPayout>,
    /// Usage of the remainder in the month of the AHV payout.
    #[serde(rename = "use", default = "wealth_use")]
    pub usage: PayoutUse,
}

/// Point in time of a 3a payout (#55): fixed date or age of the account holder.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum PayoutAt {
    #[default]
    Date,
    Age,
}

impl PayoutAt {
    fn is_date(&self) -> bool {
        *self == PayoutAt::Date
    }
}

/// Planned payout from pillar 3a.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Pillar3aPayout {
    /// «date» (default, also for older plans) or «age».
    #[serde(default, skip_serializing_if = "PayoutAt::is_date")]
    pub kind: PayoutAt,
    /// Month of the payout with `kind = date`.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub date: Option<YearMonth>,
    /// Age in decimal years with `kind = age` (61.5 = 61 years 6 months).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub years: Option<f64>,
    /// Amount; missing: the whole balance.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub amount: Option<f64>,
    #[serde(rename = "use", default = "wealth_use")]
    pub usage: PayoutUse,
}

impl Pillar3aPayout {
    /// Month of the payout; `None` if date or age is missing or the age is not between 0 and 150 years.
    pub fn month(&self, birth: Option<YearMonth>) -> Option<YearMonth> {
        match self.kind {
            PayoutAt::Date => self.date,
            PayoutAt::Age => {
                let y = self.years.filter(|y| (0.0..=150.0).contains(y))?;
                Some(birth?.plus((y * 12.0).round() as i32))
            }
        }
    }
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct Scenarios {
    #[serde(default)]
    pub sweeps: Vec<serde_json::Value>,
}

impl Plan {
    pub fn from_json(text: &str) -> Result<Plan, crate::Error> {
        let v: serde_json::Value = serde_json::from_str(text)
            .map_err(|e| crate::Error::Invalid(msg!(plan_file_invalid, detail = e.to_string())))?;
        check_format(&v)?;
        serde_json::from_value(v).map_err(|e| crate::Error::Invalid(msg!(plan_file_invalid, detail = e.to_string())))
    }
    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).expect("Plan ist immer serialisierbar")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    const EXAMPLE: &str = include_str!("../assets/new.vplan");

    // #40: Default values when reading and compact output when writing.
    #[test]
    fn defaults_when_reading() {
        let mut v: serde_json::Value = serde_json::from_str(EXAMPLE).unwrap();
        v.as_object_mut().unwrap().remove("format");
        let plan = Plan::from_json(&v.to_string()).unwrap();
        assert_eq!(plan.format, FORMAT);

        let b: BootstrapParams = serde_json::from_value(json!({})).unwrap();
        assert_eq!(
            b,
            BootstrapParams {
                paths: 10_000,
                block: 12.0,
                seed: 42
            }
        );
        assert_eq!(BootstrapParams::default(), b);
        let p: ParametricParams = serde_json::from_value(json!({"mu": 0.07, "sigma": 0.15, "df": 6.0})).unwrap();
        assert_eq!((p.paths, p.seed), (10_000, 42));
        let d = ParametricParams::default();
        assert_eq!((d.paths, d.seed), (10_000, 42));
    }

    #[test]
    fn fixed_milestone_is_written_only_when_set() {
        let mut m = Milestone {
            id: "m".into(),
            name: "M".into(),
            kind: MilestoneKind::Date {
                date: YearMonth::new(2030, 1),
            },
            fixed: false,
        };
        assert!(serde_json::to_value(&m).unwrap().get("fixed").is_none());
        m.fixed = true;
        assert_eq!(serde_json::to_value(&m).unwrap()["fixed"], json!(true));
    }

    #[test]
    fn series_constructors() {
        assert_eq!(
            serde_json::to_value(Series::constant(2.5)).unwrap(),
            json!({"mode": "const", "value": 2.5})
        );
        assert_eq!(
            serde_json::to_value(Series::timed(vec![(TimeRef("2030-01".into()), 1.5)])).unwrap(),
            json!({"mode": "timed", "rows": [{"from": "2030-01", "value": 1.5}]})
        );
    }

    // #124/AK7: A payout by age keeps its kind when saving; the kind «date» (default) is omitted.
    #[test]
    fn issue_124_ak7_payout_kind_is_saved_only_for_age() {
        let age: Pillar3aPayout = serde_json::from_value(json!({"kind": "age", "years": 61.5})).unwrap();
        let date: Pillar3aPayout = serde_json::from_value(json!({"date": "2040-06"})).unwrap();
        assert_eq!(serde_json::to_value(&age).unwrap()["kind"], json!("age"));
        assert!(serde_json::to_value(&date).unwrap().get("kind").is_none());
    }

    /// #187/AK2: German names of the fixed milestones as older plans stored them, by id.
    #[test]
    fn issue_187_ak2_stored_milestone_names() {
        let name = |id: &str| stored_milestone_name(id);
        assert_eq!(name("start").as_deref(), Some(START_NAME));
        assert_eq!(name("end").as_deref(), Some(END_NAME));
        assert_eq!(name("retire").as_deref(), Some(PK_NAME));
        assert_eq!(name("ahv").as_deref(), Some(AHV_NAME));
        assert_eq!(name("retire2"), Some(format!("{PK_NAME} Person 2")));
        assert_eq!(name("ahv2"), Some(format!("{AHV_NAME} Person 2")));
        assert_eq!(name("house"), None);
    }

    /// #187/AK2: a fixed milestone without a name is the fixed name as a message, any other the name entered.
    #[test]
    fn issue_187_ak2_name_arg_of_milestones() {
        use crate::message::Arg;
        let ms = |id: &str, name: &str| Milestone {
            id: id.into(),
            name: name.into(),
            kind: MilestoneKind::Date {
                date: YearMonth::new(2040, 1),
            },
            fixed: false,
        };
        assert_eq!(ms("retire", "").name_arg(), Arg::Msg(Box::new(msg!(milestone_retire))));
        assert_eq!(ms("retire", "Frühpension").name_arg(), Arg::Text("Frühpension".into()));
        assert_eq!(ms("house", "Hauskauf").name_arg(), Arg::Text("Hauskauf".into()));
        assert_eq!(ms("house", "").name_arg(), Arg::Text("house".into()));
    }
}
