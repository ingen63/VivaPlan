//! The monthly simulation.
//!
//! Order per calendar month:
//! 1. Inflation: every month by (1 + inflation)^(1/12) (spending, rents, property values); AHV
//!    follows in January with the inflation accrued since the previous January.
//! 2. Events: one-off amounts, retirement (lump sum, pension), AHV start, properties
//!    (emergency sale from the previous month, purchase or rent after a sale, purchase, mortgage periods,
//!    amortization, sale), pillar 3a.
//! 3. Cash flows: income, pensions, inflows minus living costs and housing costs → cash.
//!    Living costs include neither rent nor mortgage interest; interest accrues monthly and
//!    is paid together with amortization at quarter end (March, June, September, December).
//! 4. Returns on stocks and bonds (as long as the portfolio is positive).
//! 5. Pension fund: contributions; interest in December.
//! 6. December: assess the year's income and wealth tax; paid in May of the following year
//!    (an open tax is settled in the last month of the plan).
//! 7. Check for an emergency sale (carried out in the following month).
//! 8. Withdrawal strategy: «Standard» (cash reserve, the rest by stock share, monthly) or from
//!    retirement «3-Topf» (cash, bonds, stocks; refill in January).
//!
//! Second person: income, pension fund and AHV run per person; taxes, wealth and
//! spending apply to the household. The withdrawal strategies (except «Standard») start as soon as the first
//! person with a payout date is retired. Once both are entitled to an AHV pension, their pensions at 65 are capped for
//! married couples at `household.ahvCoupleCap` (#191: before the reduction or supplement, see [`crate::ahv::Calc`]).
//!
//! #191: The AHV pension is entered at the reference age (65); an early payout reduces it, a deferral increases it
//! by the rates of [`crate::ahv`] as fixed amounts in francs, for the whole payout duration and after the cap for married
//! couples (leaflet 3.04); the 13th pension rests on the pensions paid.
//!
//! #190: From 2026 the AHV pays a 13th pension in December: one twelfth of the (capped) household AHV paid
//! from January to December, each monthly share rounded to the centime, the sum to whole francs (circular
//! «13. Altersrente», margin numbers 4001–4005). It is only paid if a pension is due in December.
//!
//! Fixed withdrawal rate, guardrails, VPW and endowment determine living costs from the first person's retirement:
//! annual withdrawal = living costs + housing costs + pillar 3a contributions − income and pensions (before taxes). Sale proceeds reserved
//! for a purchase do not count towards the wealth used for withdrawals. With «3-Topf» and guardrails (#6) the
//! guardrails steer spending and the buckets steer investment; the buckets' net need is then based on the
//! guardrails' living costs.

use serde::{Deserialize, Serialize};

use crate::market::MarketPath;
use crate::message::Message;
use crate::model::{
    Affordability, AfterLast, AmortMode, AmountMode, Buckets, PayoutUse, Plan, Property, PropertyStatus, RateKind,
    ReturnSource, StateInput, StateSeries, Strategy, Tranche,
};
use crate::resolve::{Curve, Resolver};
use crate::tax;
use crate::time::YearMonth;
use crate::validate::{self, Severity};
use crate::{Error, Result};

/// Fixed cash reserve (months) for all strategies except «Standard».
pub const FIXED_RESERVE_MONTHS: f64 = 3.0;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum EventKind {
    Extra,
    Retirement,
    AhvStart,
    Purchase,
    RentStart,
    Sale,
    EmergencySale,
    MortgageRenewal,
    MortgageRepayment,
    /// Switch from indirect to direct amortization in the month of the AHV payout (#50).
    AmortizationSwitch,
    Pillar3aPayout,
    Buckets,
    Withdrawal,
    Depleted,
}

/// #65: Rules of the property gain tax for one property.
#[derive(Clone, Debug)]
struct GainRule {
    def: Option<crate::model::PropertyGainTax>,
    /// Tax multiplier; it only has an effect if the rules set `use_multiplier`.
    multiplier: f64,
    /// Holding period in months (rule set of the canton); otherwise in full years (own rules).
    months: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct Event {
    pub month: YearMonth,
    pub kind: EventKind,
    /// #171: Serialized as `text` (German), `code` and `params`.
    #[serde(flatten)]
    pub text: Message,
}

/// #133: Cash flows and valuation changes of one month. The change in total wealth against the previous month
/// is exactly [`Ledger::sum`]; all other bookings (buying and selling investments, refilling cash, repaying the
/// mortgage, paying into and out of pillar 3a, buying and selling property) are transfers and leave the total unchanged.
/// Internal only (checked in the tests), not part of the result.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Ledger {
    /// Earned income, PK pension and AHV.
    pub income: f64,
    /// One-off amounts (positive: inflow, negative: expense).
    pub extras: f64,
    /// Savings from the plan.
    pub savings: f64,
    /// Living costs (outflow, counted positive).
    pub living: f64,
    /// Housing costs as paid: rent, fix costs, mortgage interest, amortization (outflow, counted positive).
    pub housing: f64,
    /// Amortization within housing costs: lowers the mortgage or raises pillar 3a, so it stays in wealth.
    pub amortization: f64,
    /// All taxes paid (outflow, counted positive).
    pub taxes: f64,
    /// Returns on stocks and bonds.
    pub market: f64,
    /// Contributions to the pension fund.
    pub pk_contributions: f64,
    /// Interest credited by the pension fund (in December and at payout).
    pub pk_interest: f64,
    /// Pillar 3a interest.
    pub p3a_interest: f64,
    /// Change in property value (inflation).
    pub revaluation: f64,
    /// PK capital converted into a pension (leaves wealth, counted positive).
    pub annuity: f64,
}

impl Ledger {
    /// Change in total wealth according to the ledger.
    pub fn sum(&self) -> f64 {
        self.income + self.extras + self.savings - self.living - self.housing + self.amortization - self.taxes
            + self.market
            + self.pk_contributions
            + self.pk_interest
            + self.p3a_interest
            + self.revaluation
            - self.annuity
    }
}

/// Deserializable for the evaluation of saved results (#209); fields missing in older results are 0.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct MonthRecord {
    pub month: YearMonth,
    pub age: f64,
    pub cash: f64,
    pub stocks: f64,
    pub bonds: f64,
    pub liquid: f64,
    pub property_equity: f64,
    pub pension_fund: f64,
    pub pillar3a: f64,
    pub total: f64,
    /// Pillar 3a contribution this month (counts as spending, #52).
    pub pillar3a_in: f64,
    /// Income including pensions.
    pub income: f64,
    /// Living costs including housing costs.
    pub spending: f64,
    pub taxes: f64,
    /// Breakdown of the month's taxes: income, wealth, lump sum (PK, 3a), property gain.
    pub tax_income: f64,
    pub tax_wealth: f64,
    pub tax_capital: f64,
    pub tax_gain: f64,
    /// #109: Income and wealth tax assessed in December for the tax year (paid in May of the
    /// following year, see `tax_income`/`tax_wealth`). The yearly values show it in the row of the tax year.
    pub assessed_income: f64,
    pub assessed_wealth: f64,
    /// #131: Income, deductions by origin and taxable income of the tax year; only in the month of the assessment
    /// (December) of plans with deductions (mode «Wohnort», `taxes.deductions` or `taxes.deductPillar3a`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tax_basis: Option<TaxBasis>,
    /// Planned living costs from the plan (nominal, without housing costs).
    pub planned_spending: f64,
    /// Withdrawal from wealth: actual living costs and housing costs minus income (before taxes).
    /// With guardrails and similar strategies the strategy sets it, otherwise it follows the planned spending.
    pub withdrawal: f64,
    /// Breakdown: earned income, PK pension, AHV, housing costs (interest, amortization, fix costs, rent).
    pub earned: f64,
    pub pk_pension: f64,
    pub ahv: f64,
    pub housing: f64,
    /// Market value of the properties and mortgages.
    pub property_value: f64,
    pub mortgage: f64,
    /// Price index since the start (1 = prices of the start month); value / index = today's purchasing power.
    pub price_index: f64,
    /// Performance of stocks and bonds since the start (1 = start month), nominal.
    pub stock_index: f64,
    pub bond_index: f64,
    /// #133: Ledger of the month (internal only, not part of the result).
    #[serde(skip)]
    pub ledger: Ledger,
    /// #205: The floor of the spending rule sets the living costs this month (internal only, not part of the result).
    #[serde(skip)]
    pub at_floor: bool,
}

/// #131: Basis of the income tax of a year: income, deductions by origin, taxable income for canton and federal tax.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct TaxBasis {
    /// Income of the year (earned income + PK pension + AHV).
    pub income: f64,
    /// Proposal of the canton as applied (share of the year with a row «Vorschlag des Kantons»).
    pub proposal_canton: f64,
    pub proposal_federal: f64,
    /// Share of the year's months with a row «Vorschlag des Kantons» (0 to 1).
    pub proposal_share: f64,
    /// Proposal of the canton for the whole year, split by kind (`social`, `insurance`, `savingsInterest`,
    /// `professional`); empty without a proposal in the year.
    pub parts_canton: Vec<(String, f64)>,
    pub parts_federal: Vec<(String, f64)>,
    /// Own value (amount and share of the income), for canton and federal tax.
    pub own: f64,
    /// #230: Additional amount to the proposal of the canton (share of the year with such a row), for canton and
    /// federal tax.
    pub extra: f64,
    /// #230: One-off deductions of the tax year, for canton and federal tax.
    pub one_off: f64,
    /// Pillar 3a contributions, at most the maximum per person.
    pub pillar3a: f64,
    /// Taxable income, never negative; with own tables both are the same.
    pub taxable_canton: f64,
    pub taxable_federal: f64,
    /// #151: Place of residence of the tax year (on 31 December); only if the place changes during the plan.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub residence: Option<crate::model::TaxLocation>,
    /// #229: Tax value of the owned properties on 31 December; only if a property has a tax value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub property_tax_value: Option<f64>,
    /// #231: Wealth, deductions from wealth by origin and taxable wealth on 31 December; only if the plan has
    /// deductions from wealth.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wealth: Option<WealthBasis>,
}

/// #231: Basis of the wealth tax of a year with deductions from wealth.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct WealthBasis {
    /// Wealth before the deductions (properties at their tax value, #229).
    pub wealth: f64,
    /// Row of the deductions from wealth on 31 December (prices of the start × index).
    pub ongoing: f64,
    /// One-off deductions from wealth of the tax year (nominal).
    pub one_off: f64,
    /// Wealth minus deductions, never negative. In the mode «Wohnort» the social deductions of the canton are applied
    /// on top inside the wealth curves.
    pub taxable: f64,
}

/// #131: Deductions of a tax year collected month by month: months with a proposal of the canton, own value and
/// pillar 3a contributions per person.
#[derive(Clone, Copy, Debug, Default)]
struct YearDeductions {
    months: u32,
    canton_months: u32,
    /// Sum of the own amounts per year over the months (divided by `months` at the end of the year).
    own_amount: f64,
    /// Sum of share × income of the months.
    own_rate: f64,
    /// #230: Sum of the additional amounts per year of the months with a proposal of the canton (divided by `months`).
    extra: f64,
    /// #230: One-off deductions of the tax year (nominal); set at the assessment, not in the estimate of the net need.
    one_off: f64,
    pillar3a: [f64; 2],
}

/// #151: Compiled place of residence: applies from month `from` (index of `YearMonth`) until the next row.
#[derive(Clone, Debug)]
struct CLocation {
    from: i32,
    place: crate::model::TaxLocation,
    resolved: crate::taxdata::Resolved,
}

/// #131: Compiled row of the deductions from income.
#[derive(Clone, Copy, Debug)]
struct CDeduction {
    from: i32,
    kind: crate::model::DeductionKind,
    amount: f64,
    rate: f64,
    extra: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum PropertyEventKind {
    Purchase,
    Sale,
    EmergencySale,
    /// Renewal of a mortgage period (also with a lump sum, or not renewed because of an emergency sale).
    Renewal,
}

/// Property event with its amounts: purchase, sale, emergency sale and mortgage renewal in one list.
/// Amounts that do not apply are missing.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PropertyEvent {
    pub month: YearMonth,
    pub kind: PropertyEventKind,
    pub property: String,
    /// Purchase: price, equity (price − mortgage) and mortgage.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub price: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub equity: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mortgage: Option<f64>,
    /// Sale, emergency sale: sale proceeds (market value), mortgage paid off, property gain tax, net proceeds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proceeds: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debt_repaid: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gain_tax: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub net: Option<f64>,
    /// Renewal: affordability, mortgage before and after, lump sum, rate of the new period (average of the tranches).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub affordable: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debt_before: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub debt_after: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lump_sum: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rate: Option<f64>,
    /// Emergency sale: reason («wealth < 0» or «emergency sale criterion»); for a renewal, why it did not happen.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<Message>,
    /// Sale: what happens to the proceeds («Verkaufserlös reserviert für Kauf im Folgemonat» or withdrawal strategy applied at once).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<Message>,
}

impl PropertyEvent {
    fn new(month: YearMonth, kind: PropertyEventKind, property: &str) -> Self {
        PropertyEvent {
            month,
            kind,
            property: property.to_string(),
            price: None,
            equity: None,
            mortgage: None,
            proceeds: None,
            debt_repaid: None,
            gain_tax: None,
            net: None,
            affordable: None,
            debt_before: None,
            debt_after: None,
            lump_sum: None,
            rate: None,
            reason: None,
            note: None,
        }
    }
}

/// Basis of the bucket sizes in the 3-bucket strategy: net need per year = living costs + housing +
/// income tax − regular income (AHV, PK pension, earned income). Not included: one-off expenses,
/// amortization, property purchases, special taxes (property gain, lump sum and wealth tax).
#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NeedBasis {
    /// Reference year of the values.
    pub year: i32,
    /// «Januar, geplant» or «ab <Monat>, auf 12 Monate hochgerechnet».
    pub source: Message,
    /// Source of the living costs: «geplant» or «Guardrails» (#6).
    pub living_source: String,
    pub living: f64,
    pub housing: f64,
    /// Income tax on the (extrapolated) annual income.
    pub income_tax: f64,
    /// AHV pension.
    pub ahv: f64,
    /// PK pension.
    pub pk: f64,
    /// Earned income.
    pub earned: f64,
    /// Pillar 3a contributions (#52).
    pub pillar3a: f64,
    /// Number of months the values were extrapolated from (12 = full year).
    pub months: u32,
}

impl NeedBasis {
    pub fn expenses(&self) -> f64 {
        self.living + self.housing + self.income_tax + self.pillar3a
    }
    /// Regular income: pensions and earned income.
    pub fn receipts(&self) -> f64 {
        self.ahv + self.pk + self.earned
    }
    /// Net need per year; never negative.
    pub fn need(&self) -> f64 {
        (self.expenses() - self.receipts()).max(0.0)
    }
}

/// Transfer between the buckets (1 = cash, 2 = bonds, 3 = stocks; 0 = from outside, e.g. an inflow).
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BucketMove {
    pub from: u8,
    pub to: u8,
    pub amount: f64,
}

/// One application of the 3-bucket strategy (January refill, special case A, B or C) with derivation, return
/// criterion, transfers and bucket balances.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BucketRefill {
    pub month: YearMonth,
    /// Description of the occasion.
    pub reason: Message,
    /// Case triggered: «Januar», «A», «B» or «C».
    pub trigger: String,
    /// For A: months of the year already past, by which the target sizes are reduced (otherwise 0).
    pub passed_months: u32,
    pub basis: NeedBasis,
    pub need: f64,
    pub target1: f64,
    pub target2: f64,
    /// Stock return for the return condition (last 12 months, including the lead-in before the start) and its basis.
    pub stock_return: f64,
    pub return_basis: Message,
    pub threshold: f64,
    /// Stock return ≥ threshold (sales from bucket 3 are allowed). For A and C the criterion is ignored.
    pub condition_met: bool,
    /// The criterion was ignored (A, B and C).
    pub condition_ignored: bool,
    pub moves: Vec<BucketMove>,
    /// This month's expenses (ongoing and one-off), paid from bucket 1 after the refill.
    pub payment: f64,
    /// Balance of bucket 1 (cash), bucket 2 (bonds), bucket 3 (stocks) before and after the refill, each before the
    /// month's expenses.
    pub before: [f64; 3],
    pub after: [f64; 3],
}

/// Affordability check at the renewal of a mortgage period.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Renewal {
    pub month: YearMonth,
    pub property: String,
    /// The mortgage was affordable (no action needed).
    pub affordable: bool,
    /// Lump sum from wealth (0 if none was needed or none was possible).
    pub lump_sum: f64,
    pub debt_before: f64,
    /// Mortgage after the renewal (unchanged on an emergency sale; the property is sold in the following month).
    pub debt_after: f64,
    /// Reason for the emergency sale: «wealth < 0» or «emergency sale criterion»; `None`: no emergency sale.
    pub emergency_sale: Option<Message>,
}

pub(crate) fn is_zero_count(n: &usize) -> bool {
    *n == 0
}

/// #214: Receives the months of a run in order (see [`Simulator::run_observed`] and `measure`). `summary` is the state
/// after the month: counters and the month of depletion so far.
pub trait MonthObserver {
    fn month(&mut self, record: &MonthRecord, summary: &Summary);
}

/// Deserializable for the evaluation of saved results (#209); fields missing in older results are 0.
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Summary {
    /// Emergency sales in this run (for any reason).
    pub emergency_sales: usize,
    /// Lump sums because of insufficient affordability at renewals.
    pub lump_sums: usize,
    /// Emergency sales because of insufficient affordability at renewals.
    pub affordability_sales: usize,
    /// Renewals that were not affordable and were renewed in full only because of «Immobilien nie verkaufen» (#199).
    /// Missing in the JSON when 0.
    #[serde(skip_serializing_if = "is_zero_count")]
    pub unaffordable_renewals: usize,
    pub end_total: f64,
    pub end_liquid: f64,
    /// Total wealth and liquid wealth at the start of the simulation (before the first month), nominal.
    pub start_total: f64,
    pub start_liquid: f64,
    /// First month without liquid wealth and without property.
    pub depleted: Option<YearMonth>,
    pub total_at_retirement: Option<f64>,
    pub min_liquid: f64,
    pub min_liquid_month: Option<YearMonth>,
    pub pension_fund_monthly: f64,
    pub lumpsum_net: f64,
    pub lumpsum_tax: f64,
    pub taxes_total: f64,
    /// Total wealth at the end in prices of the start month.
    pub end_total_real: f64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SimResult {
    pub label: Message,
    pub summary: Summary,
    pub events: Vec<Event>,
    /// All renewals of mortgage periods with the result of the affordability check.
    pub renewals: Vec<Renewal>,
    /// All property events (purchase, sale, emergency sale, renewal) in chronological order.
    pub property_events: Vec<PropertyEvent>,
    /// Applications of the 3-bucket strategy with derivation (empty for other strategies).
    pub bucket_refills: Vec<BucketRefill>,
    /// Only filled when computed with `record = true`.
    pub records: Vec<MonthRecord>,
    /// Pillar 3a payout plan per account (#55); only with `record = true`. Not part of the saved
    /// results; the preview passes it on separately.
    #[serde(skip)]
    pub pillar3a: Vec<Pillar3aView>,
}

/// #191: AHV pension of one person in the editor (preview): entered pension at the reference age, rate and the
/// adjusted pension the simulation computes with; for married couples the capped share once both pensions run.
/// Amounts in prices of the start.
#[derive(Clone, Debug, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AhvView {
    /// Index of the person (0 = first person).
    pub person: usize,
    pub entered: f64,
    /// Reduction (negative) or supplement for the payout before or after the reference age.
    pub rate: f64,
    /// Monthly pension the simulation computes with.
    pub monthly: f64,
    pub start: Option<YearMonth>,
    /// Married couples: month from which both are entitled (payout or reference age) and the pensions at 65 together
    /// exceed the cap.
    pub capped_from: Option<YearMonth>,
    /// Married couples: pension of this person after the cap and the reduction or supplement, once every early payout
    /// and deferral is over (#191, leaflet 3.04).
    pub capped: Option<f64>,
}

/// #191: The AHV pensions per person as the simulation computes with them (see [`AhvView`]); also for a plan with
/// errors elsewhere, as long as the payouts can be resolved (otherwise empty). Without inflation.
pub fn ahv_view(plan: &Plan) -> Vec<AhvView> {
    let Ok(r) = Resolver::new(plan) else {
        return vec![];
    };
    let persons = &plan.household.persons;
    let mut list = vec![(persons.first(), &plan.pension.ahv)];
    if let (Some(q), Some(p2)) = (&plan.partner, persons.get(1)) {
        list.push((Some(p2), &q.ahv));
    }
    let mut people = vec![];
    for (p, ahv) in list {
        let (Some(p), Ok(at)) = (p, r.resolve(&ahv.at)) else {
            return vec![];
        };
        people.push(crate::ahv::Person::new(p.birth, ahv.monthly, at, false));
    }
    let married = people.len() > 1 && plan.household.married;
    let cap = married.then_some(plan.household.ahv_couple_cap);
    // From the month both are entitled; the amounts once every early payout and deferral is over.
    let both = married
        .then(|| people.iter().map(|p| p.entitled_from()).collect::<Option<Vec<_>>>())
        .flatten()
        .and_then(|v| v.into_iter().max());
    let first = people.iter().filter_map(|p| p.entitled_from()).min();
    let last = people.iter().filter_map(|p| p.at.map(|a| a.max(p.reference))).max();
    let mut finals = vec![None; people.len()];
    if let (Some(from), Some(to)) = (first, last.max(both)) {
        let mut calc = crate::ahv::Calc::new(people.clone(), cap);
        // bounded (extreme dates in a plan): at most 100 years of months
        for k in 0..=from.until(to).clamp(0, 1200) {
            calc.step(from.plus(k), k == 0);
        }
        for (i, f) in finals.iter_mut().enumerate() {
            *f = Some(calc.paid(i));
        }
    }
    let capped = both.filter(|_| people.iter().map(|p| p.entered).sum::<f64>() > plan.household.ahv_couple_cap);
    people
        .iter()
        .enumerate()
        .map(|(i, p)| AhvView {
            person: i,
            entered: p.entered,
            rate: p.rate(),
            monthly: crate::ahv::adjusted(p.entered, p.offset()),
            start: p.at,
            capped_from: capped,
            capped: capped.and(finals[i]),
        })
        .collect()
}

/// Pillar 3a of one account in the preview (#55).
#[derive(Clone, Debug, Default, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Pillar3aView {
    pub person: String,
    /// Balance of every month from the simulation start before that month's payouts, up to the AHV payout.
    pub balances: Vec<f64>,
    pub payouts: Vec<Pillar3aPayoutView>,
    /// Payout of the remainder at the AHV payout.
    pub rest: Option<Pillar3aRest>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Pillar3aPayoutView {
    /// Row in the payout plan of the plan.
    pub index: usize,
    pub month: YearMonth,
    pub before: f64,
    pub paid: f64,
    pub after: f64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Pillar3aRest {
    pub month: YearMonth,
    pub amount: f64,
}

// ---------------------------------------------------------------------------------------------
// Compiled plan
// ---------------------------------------------------------------------------------------------

#[derive(Clone, Debug)]
struct SPeriod {
    start: YearMonth,
    end: YearMonth,
    index: usize,
    kind: RateKind,
    rate: f64,
    margin: f64,
    amort_q: f64,
    direct: bool,
}

impl SPeriod {
    fn rate(&self, saron: f64) -> f64 {
        match self.kind {
            RateKind::Fixed => self.rate,
            RateKind::Saron => saron.max(0.0) + self.margin,
        }
    }
}

#[derive(Clone, Debug)]
struct CTranche {
    name: String,
    amount: f64,
    periods: Vec<SPeriod>,
    repay_at: Option<YearMonth>,
    /// Person (index) whose pillar 3a receives the indirect amortization (#52).
    account: usize,
}

/// Point in time that results from a sale during the simulation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Trigger {
    None,
    /// After the sale of the last owned property.
    LastSale,
    /// After the sale of a specific property (index).
    SaleOf(usize),
}

/// #171: Basis of the stock return of the 3-bucket strategy as a message («last 12 months», «since start»).
fn return_basis(basis: &str) -> Message {
    if basis == "since start" {
        msg!(return_since_start)
    } else {
        msg!(return_last_12_months)
    }
}

/// Pillar 3a of one person (#52).
#[derive(Clone, Debug, Default)]
struct CAccount {
    /// Suffix for event texts with two persons (` (name)` as a message, otherwise empty).
    who: crate::message::Arg,
    balance: f64,
    rate_m: f64,
    /// Contribution per month (one twelfth of the annual contribution), until before the AHV payout.
    contribution_m: f64,
    /// Person (id), for displaying the payout plan.
    person: String,
    /// Planned payouts: month, amount (missing: whole balance), repay mortgage, row in the plan.
    payouts: Vec<(YearMonth, Option<f64>, bool, usize)>,
    /// AHV payout: end of contributions and of indirect amortization, payout of the remainder.
    ahv: Option<YearMonth>,
    rest_repay: bool,
}

#[derive(Clone, Debug)]
struct CProp {
    name: String,
    status: PropertyStatus,
    price: f64,
    bought: Option<YearMonth>,
    worth: f64,
    /// #229: Ratio of the tax value to the market value (1.0 without a tax value).
    tax_ratio: f64,
    infl_value: bool,
    infl_fix_costs: bool,
    infl_rent: bool,
    fix_costs: f64,
    buy: Option<YearMonth>,
    sell: Option<YearMonth>,
    rent_from: Option<YearMonth>,
    rent_until: Option<YearMonth>,
    rent: f64,
    /// Purchase or rent «nach Verkauf».
    buy_trigger: Trigger,
    rent_trigger: Trigger,
    /// Mortgage at purchase: maximum loan-to-value instead of the entered amounts.
    max_loan: bool,
    /// Periods, if the purchase date is known in advance (owned, planned purchase with date).
    tranches: Vec<CTranche>,
    /// Definition for purchases «nach Verkauf»; the periods are created only at the purchase.
    tranche_defs: Vec<Tranche>,
}

/// Income and pension of one person.
struct PersonCfg {
    birth: YearMonth,
    income: Curve,
    /// #58: Income in prices of the start, extrapolated with the path's price index.
    income_indexed: bool,
    /// #236: First month without earned income (milestone «Bezug Pensionskasse» of the person), if the switch is on.
    income_end: Option<YearMonth>,
    pk_contribution: Curve,
    pk_interest: Curve,
    pk_capital: f64,
    pk_lumpsum: f64,
    pk_conversion: f64,
    pk_at: Option<YearMonth>,
    /// #191: AHV of the person (entered pension at 65, payout, reference age; see [`crate::ahv::Calc`]).
    ahv: crate::ahv::Person,
}

#[derive(Clone, Copy, Debug)]
enum Rule {
    Fixed {
        rate: f64,
    },
    Guardrails {
        initial: f64,
        upper: f64,
        lower: f64,
        adjust: f64,
    },
    Vpw {
        horizon_age: f64,
        real_return: f64,
    },
    Endowment {
        rate: f64,
        smoothing: f64,
    },
}

/// Share of wealth that uses it up evenly in `years` years at real return `r`.
fn vpw_rate(r: f64, years: f64) -> f64 {
    let n = years.max(1.0);
    let rate = if r.abs() < 1e-9 {
        1.0 / n
    } else {
        r / (1.0 - (1.0 + r).powf(-n))
    };
    rate.clamp(0.0, 1.0)
}

/// Fixed withdrawal rate or guardrails.
#[derive(Clone, Copy, Debug)]
struct Dynamic {
    rule: Rule,
    floor: Option<f64>,
    ceiling: Option<f64>,
}

pub struct Simulator {
    currency: String,
    pub start: YearMonth,
    pub end: YearMonth,
    months: usize,
    /// Main person, then (if present) the second person.
    people: Vec<PersonCfg>,
    /// Cap on a married couple's AHV pensions per month.
    couple_cap: Option<f64>,
    /// #150: Unmarried couples pay the lump-sum tax per person; otherwise the household's lump sums add up.
    capital_per_person: bool,
    spending: Curve,
    /// #53: Housing share of the entered living costs (prices of the start month) and factor of the variation.
    housing_share: f64,
    spending_factor: f64,
    savings: Curve,
    stock_share: Curve,
    /// #218: the stock share as a characteristic curve over a state variable (then `stock_share` is not used).
    share_curve: Option<crate::state::StateLine>,
    /// #218: remaining spending minus earned income from month `k` until the end (prices of the start month), only
    /// with a curve over `fundedRatio`.
    need_rest: Vec<f64>,
    reserve: Curve,
    emergency: Curve,
    /// #196: Never sell a property (switch «Immobilien nie verkaufen»).
    never_sell: bool,
    stocks0: f64,
    bonds0: f64,
    cash0: f64,
    extras: Vec<(YearMonth, f64, String)>,
    props: Vec<CProp>,
    /// Pillar 3a per person (index as in `people`).
    accounts: Vec<CAccount>,
    /// Ids of the persons (for assigning the tranches to pillar 3a).
    person_ids: Vec<String>,
    taxes: crate::model::Taxes,
    /// #63: Tables and deductions of the place of residence (mode «Wohnort»); empty: own tables. #151: one row per
    /// place of residence, sorted by month; the first applies from the start.
    locations: Vec<CLocation>,
    /// #65: Property gain tax per property (index as in `props`); #151: per place of residence (index as in
    /// `locations`), with own rules a single entry.
    gain_rules: Vec<Vec<GainRule>>,
    /// #131: Rows of the deductions from income, sorted by month; pillar 3a deduction and its maximum per person.
    deductions: Vec<CDeduction>,
    /// #230: One-off deductions from income per tax year (nominal).
    one_off: Vec<(i32, f64)>,
    /// #231: Rows of the deductions from wealth (month, amount in prices of the start), sorted by month; one-off
    /// deductions from wealth per tax year (nominal). Both empty: wealth tax as before.
    wealth_deductions: Vec<(i32, f64)>,
    one_off_wealth: Vec<(i32, f64)>,
    deduct_pillar3a: bool,
    pillar3a_max: f64,
    /// #152: Persons without a pension fund (large pillar 3a) and its maximum.
    pillar3a_large: [bool; 2],
    pillar3a_max_large: f64,
    /// #131: Record the basis of the income tax (plans with deductions).
    tax_basis: bool,
    /// #229: A property (owned or planned) has a tax value; the basis then also shows the tax value of the properties.
    tax_values: bool,
    pub fixed: crate::market::FixedCurves,
    pub source: ReturnSource,
    afford: Affordability,
    /// 3-bucket strategy from retirement.
    buckets: Option<Buckets>,
    dynamic: Option<Dynamic>,
}

impl Simulator {
    pub fn new(plan: &Plan) -> Result<Self> {
        // Take over older pillar 3a on the mortgage as when loading (#52).
        let normalized = crate::upgrade::normalized(plan);
        let plan: &Plan = &normalized;
        let errors: Vec<Message> = validate::validate(plan)
            .into_iter()
            .filter(|i| i.severity == Severity::Error)
            .map(|i| i.message)
            .collect();
        if !errors.is_empty() {
            return Err(Error::Invalid(msg!(plan_errors, errors = errors)));
        }
        let w = &plan.withdrawal;
        let buckets = (w.strategy == Strategy::ThreeBuckets).then(|| w.buckets.clone().unwrap_or_default());
        let dynamic = match w.strategy {
            Strategy::FixedRate => Some(Rule::Fixed {
                rate: w.fixed_rate.clone().unwrap_or_default().rate,
            }),
            // #6: With «3-Topf» and guardrails, the guardrails steer spending and the buckets steer investment.
            _ if w.strategy == Strategy::Guardrails || w.buckets_with_guardrails() => {
                let g = w.guardrails.clone().unwrap_or_default();
                Some(Rule::Guardrails {
                    initial: g.initial_rate,
                    upper: g.upper,
                    lower: g.lower,
                    adjust: g.adjust,
                })
            }
            Strategy::Vpw => {
                let v = w.vpw.clone().unwrap_or_default();
                Some(Rule::Vpw {
                    horizon_age: v.horizon_age,
                    real_return: v.real_return,
                })
            }
            Strategy::Endowment => {
                let e = w.endowment.clone().unwrap_or_default();
                Some(Rule::Endowment {
                    rate: e.rate,
                    smoothing: e.smoothing,
                })
            }
            _ => None,
        }
        .map(|rule| Dynamic {
            rule,
            floor: w.floor.filter(|f| *f > 0.0),
            ceiling: w.ceiling.filter(|c| *c > 0.0),
        });
        let r = Resolver::new(plan)?;
        let start = r.milestone("start")?;
        let end = r.milestone("end")?;
        let curve = |s: &crate::model::Series| Curve::compile(s, &r);
        let person_cfg = |index: usize,
                          birth: YearMonth,
                          income: &crate::model::Series,
                          income_indexed: bool,
                          income_ends: bool,
                          pk: &crate::model::PensionFund,
                          ahv: &crate::model::Ahv|
         -> Result<PersonCfg> {
            let ahv_at = r.resolve(&ahv.at)?;
            // #236: the person's own milestone «Bezug Pensionskasse» (`retire`, `retire2`)
            let income_end = if income_ends {
                Some(r.milestone(crate::model::person_milestones(index).0)?)
            } else {
                None
            };
            Ok(PersonCfg {
                birth,
                income: curve(income)?,
                income_indexed,
                income_end,
                pk_contribution: curve(&pk.contribution)?,
                pk_interest: curve(&pk.interest)?,
                pk_capital: pk.capital,
                pk_lumpsum: pk.lumpsum,
                pk_conversion: pk.conversion,
                pk_at: r.resolve(&pk.at)?,
                ahv: crate::ahv::Person::new(birth, ahv.monthly, ahv_at, ahv.indexed),
            })
        };
        let persons = &plan.household.persons;
        let mut people = vec![person_cfg(
            0,
            persons[0].birth,
            &plan.flows.income,
            plan.flows.income_indexed,
            plan.flows.income_ends_at_retirement,
            &plan.pension.pk,
            &plan.pension.ahv,
        )?];
        if let (Some(q), Some(p2)) = (&plan.partner, persons.get(1)) {
            people.push(person_cfg(
                1,
                p2.birth,
                &q.income,
                q.income_indexed,
                q.income_ends_at_retirement,
                &q.pk,
                &q.ahv,
            )?);
        }
        let couple = people.len() > 1 && plan.household.married;
        let separate = people.len() > 1 && !plan.household.married;
        let mut taxes = plan.taxes.clone();
        if let (true, Some(c)) = (couple, &plan.taxes.couple) {
            taxes.income = c.income.clone();
            taxes.wealth = c.wealth.clone();
            taxes.capital = c.capital.clone();
        }
        // #63: Mode «Wohnort»: tariffs and deductions for married couples with a second person and `household.married`.
        // #151: further places of residence from their point in time (stable order: for the same month the later row).
        let locations = match plan.taxes.mode {
            crate::model::TaxMode::Own => vec![],
            crate::model::TaxMode::Location => {
                let data = crate::taxdata::builtin();
                let first = plan.taxes.location.clone().unwrap_or_default();
                let mut out = vec![CLocation {
                    from: i32::MIN,
                    resolved: data.resolve(&first, couple)?,
                    place: first,
                }];
                for row in &plan.taxes.relocations {
                    if let Some(m) = r.resolve(&row.from)? {
                        out.push(CLocation {
                            from: m.0,
                            resolved: data.resolve(&row.place, couple)?,
                            place: row.place.clone(),
                        });
                    }
                }
                out.sort_by_key(|l| l.from);
                out
            }
        };
        // #65: Mode «Wohnort» without own rules: rule set of the canton in which the property lies.
        let canton_rules = plan.taxes.mode == crate::model::TaxMode::Location && !plan.taxes.property_gain_own;
        // #151: per place of residence (canton of the property if empty, multiplier of the municipality).
        let gain_rules = plan
            .realestate
            .properties
            .iter()
            .map(|p| {
                // Rented properties are never sold; their canton does not matter.
                if canton_rules && p.status != crate::model::PropertyStatus::Rent {
                    locations
                        .iter()
                        .map(|l| {
                            let (def, multiplier) =
                                crate::taxdata::builtin().property_gain(&l.place, p.canton.as_deref())?;
                            Ok(GainRule {
                                def: Some(def),
                                multiplier,
                                months: true,
                            })
                        })
                        .collect::<Result<Vec<_>>>()
                } else {
                    Ok(vec![GainRule {
                        def: plan.taxes.property_gain.clone(),
                        multiplier: plan.taxes.multiplier,
                        months: false,
                    }])
                }
            })
            .collect::<Result<Vec<_>>>()?;
        // #131: Without rows as before: in the mode «Wohnort» the proposal of the canton throughout, otherwise none.
        let deductions = match &plan.taxes.deductions {
            Some(rows) => {
                let mut out = Vec::with_capacity(rows.len());
                for d in rows {
                    if let Some(m) = r.resolve(&d.from)? {
                        out.push(CDeduction {
                            from: m.0,
                            kind: d.kind,
                            amount: d.amount,
                            rate: d.rate,
                            extra: d.extra,
                        });
                    }
                }
                // Sort stably: for the same month the later row applies.
                out.sort_by_key(|d| d.from);
                out
            }
            None if !locations.is_empty() => vec![CDeduction {
                from: i32::MIN,
                kind: crate::model::DeductionKind::Canton,
                amount: 0.0,
                rate: 0.0,
                extra: 0.0,
            }],
            None => vec![],
        };
        // #230: one-off deductions per tax year (several in the same year are added).
        let mut one_off: Vec<(i32, f64)> = Vec::new();
        for d in &plan.taxes.one_off_deductions {
            match one_off.iter_mut().find(|(y, _)| *y == d.year) {
                Some((_, a)) => *a += d.amount,
                None => one_off.push((d.year, d.amount)),
            }
        }
        // #231: deductions from wealth: rows sorted by month (stably: for the same month the later row applies),
        // one-off deductions per tax year (several in the same year are added).
        let mut wealth_deductions = Vec::with_capacity(plan.taxes.wealth_deductions.len());
        for d in &plan.taxes.wealth_deductions {
            if let Some(m) = r.resolve(&d.from)? {
                wealth_deductions.push((m.0, d.amount));
            }
        }
        wealth_deductions.sort_by_key(|d| d.0);
        let mut one_off_wealth: Vec<(i32, f64)> = Vec::new();
        for d in &plan.taxes.one_off_wealth_deductions {
            match one_off_wealth.iter_mut().find(|(y, _)| *y == d.year) {
                Some((_, a)) => *a += d.amount,
                None => one_off_wealth.push((d.year, d.amount)),
            }
        }
        let tax_values = plan
            .realestate
            .properties
            .iter()
            .any(|p| p.status != PropertyStatus::Rent && p.tax_value.is_some());
        let tax_basis = !locations.is_empty()
            || plan.taxes.deductions.is_some()
            || plan.taxes.deduct_pillar3a
            || !one_off.is_empty()
            || !plan.taxes.wealth_deductions.is_empty()
            || !one_off_wealth.is_empty()
            || tax_values;
        let props = plan
            .realestate
            .properties
            .iter()
            .map(|p| compile_property(p, plan, &r, end))
            .collect::<Result<Vec<_>>>()?;
        // Pillar 3a: one account per person (empty if not given), so indirect amortization always has a target.
        let accounts = persons
            .iter()
            .take(people.len())
            .enumerate()
            .map(|(i, person)| {
                let who = if people.len() > 1 {
                    crate::message::arg(&msg!(suffix_name, name = person.name))
                } else {
                    crate::message::Arg::Null
                };
                let ahv = r.milestone(crate::model::person_milestones(i).1).ok();
                match plan.pillar3a.iter().find(|a| a.person == person.id) {
                    None => CAccount {
                        who,
                        ahv,
                        person: person.id.clone(),
                        ..Default::default()
                    },
                    Some(a) => {
                        // #55: Point in time as a date or as the age of the account holder.
                        let mut payouts: Vec<_> = a
                            .payouts
                            .iter()
                            .enumerate()
                            .filter_map(|(j, x)| {
                                Some((
                                    x.month(Some(person.birth))?.max(start),
                                    x.amount,
                                    x.usage == PayoutUse::Repay,
                                    j,
                                ))
                            })
                            .collect();
                        payouts.sort_by_key(|x| (x.0, x.3));
                        CAccount {
                            who,
                            person: person.id.clone(),
                            balance: a.balance,
                            rate_m: crate::market::monthly(a.rate),
                            contribution_m: a.contribution / 12.0,
                            payouts,
                            ahv,
                            rest_repay: a.usage == PayoutUse::Repay,
                        }
                    }
                }
            })
            .collect();
        Ok(Simulator {
            currency: plan.project.currency.clone(),
            start,
            end,
            months: (start.until(end) + 1) as usize,
            people,
            couple_cap: couple.then_some(plan.household.ahv_couple_cap),
            capital_per_person: separate,
            spending: curve(&plan.flows.spending)?,
            housing_share: if plan.flows.spending_includes_housing {
                crate::housing::start_housing(plan, &r)
            } else {
                0.0
            },
            spending_factor: plan.flows.spending_factor.unwrap_or(1.0),
            savings: plan
                .flows
                .savings
                .as_ref()
                .map(curve)
                .transpose()?
                .unwrap_or(Curve::constant(0.0)),
            stock_share: match &plan.wealth.stock_share {
                StateSeries::Series(s) => curve(s)?,
                StateSeries::Curve(_) => Curve::constant(0.0),
            },
            share_curve: match &plan.wealth.stock_share {
                StateSeries::Curve(c) => Some(crate::state::StateLine::compile(c)?),
                StateSeries::Series(_) => None,
            },
            need_rest: vec![],
            // Adjustable only with «Standard»; otherwise a fixed assumption (#18).
            reserve: if w.strategy == Strategy::Standard {
                curve(&plan.wealth.reserve_months)?
            } else {
                Curve::constant(FIXED_RESERVE_MONTHS)
            },
            emergency: curve(&plan.realestate.emergency_months)?,
            never_sell: plan.realestate.never_sell,
            stocks0: plan.wealth.stocks,
            bonds0: plan.wealth.bonds,
            cash0: plan.wealth.cash,
            // Amounts before the start are booked in the start month.
            extras: plan
                .flows
                .extras
                .iter()
                .map(|x| (x.date.max(start), x.amount, x.label.clone()))
                .collect(),
            props,
            accounts,
            person_ids: persons.iter().map(|p| p.id.clone()).collect(),
            taxes,
            locations,
            gain_rules,
            deductions,
            one_off,
            wealth_deductions,
            one_off_wealth,
            deduct_pillar3a: plan.taxes.deduct_pillar3a,
            pillar3a_max: plan.limits.pillar3a_max,
            pillar3a_large: [0, 1].map(|i| persons.get(i).is_some_and(|p| p.no_pension_fund)),
            pillar3a_max_large: plan.limits.pillar3a_max_large,
            tax_basis,
            tax_values,
            fixed: crate::market::FixedCurves {
                stocks: curve(&plan.returns.fixed.stocks)?,
                bonds: curve(&plan.returns.fixed.bonds)?,
                inflation: curve(&plan.returns.fixed.inflation)?,
                saron: curve(&plan.returns.fixed.saron)?,
            },
            source: plan.returns.source,
            afford: plan.realestate.affordability.clone(),
            buckets,
            dynamic,
        }
        .with_need_rest())
    }

    /// #218: fills `need_rest` if the stock share is a curve over `fundedRatio` (sum from the back).
    fn with_need_rest(mut self) -> Self {
        if self
            .share_curve
            .as_ref()
            .is_some_and(|c| c.input == StateInput::FundedRatio)
        {
            let mut rest = vec![0.0; self.months + 1];
            for k in (0..self.months).rev() {
                let m = self.start.plus(k as i32);
                rest[k] = rest[k + 1] + self.planned_living(m) - self.earned(m, 1.0);
            }
            self.need_rest = rest;
        }
        self
    }

    /// Planned living costs without housing costs in month `m` in prices of the start month (#53): entered amount
    /// minus housing share, times the factor of the variation.
    fn planned_living(&self, m: YearMonth) -> f64 {
        (self.spending.at(m) - self.housing_share) * self.spending_factor
    }

    /// Earned income of the household in this month; `infl_idx` is the path's price index since the start. If a
    /// person's income rises with inflation (#58), their amount is in prices of the start.
    fn earned(&self, m: YearMonth, infl_idx: f64) -> f64 {
        (0..self.people.len()).map(|i| self.person_earned(i, m, infl_idx)).sum()
    }

    /// Net earned income of person `i` in month `m`; #236: 0 from the end of the earned income on.
    fn person_earned(&self, i: usize, m: YearMonth, infl_idx: f64) -> f64 {
        let p = &self.people[i];
        if p.income_end.is_some_and(|end| m >= end) {
            return 0.0;
        }
        p.income.at(m) * if p.income_indexed { infl_idx } else { 1.0 }
    }

    /// #151: Index of the place of residence in month `m` (in `locations`); 0 with own tables.
    fn location_index(&self, m: YearMonth) -> usize {
        self.locations.partition_point(|l| l.from <= m.0).max(1) - 1
    }

    /// #151: Place of residence in month `m` (mode «Wohnort»), `None` with own tables.
    fn location_at(&self, m: YearMonth) -> Option<&CLocation> {
        self.locations.get(self.location_index(m))
    }

    /// #151: Place of residence of the tax year of month `m`: the one on 31 December (StHG Art. 4b para. 1, Art. 17
    /// para. 1).
    fn location_of_year(&self, m: YearMonth) -> Option<&CLocation> {
        self.location_at(YearMonth::new(m.year(), 12))
    }

    /// #131: Row of the deductions from income that applies in month `m` (`None` before the first row).
    fn deduction_at(&self, m: YearMonth) -> Option<&CDeduction> {
        match self.deductions.partition_point(|d| d.from <= m.0) {
            0 => None,
            i => Some(&self.deductions[i - 1]),
        }
    }

    /// #230: One-off deductions from income of the tax year `year` (nominal).
    fn one_off_deduction(&self, year: i32) -> f64 {
        self.one_off.iter().filter(|(y, _)| *y == year).map(|(_, a)| a).sum()
    }

    /// #231: Taxable wealth on 31 December of the year of month `m`: `wealth` minus the row of the deductions from
    /// wealth of December (prices of the start × `index`) and the one-off deductions of the year (nominal), never
    /// negative. Without deductions from wealth exactly `wealth` and no basis (as before #231).
    fn taxable_wealth(&self, m: YearMonth, wealth: f64, index: f64) -> (f64, Option<WealthBasis>) {
        if self.wealth_deductions.is_empty() && self.one_off_wealth.is_empty() {
            return (wealth, None);
        }
        let december = YearMonth::new(m.year(), 12).0;
        let ongoing = match self.wealth_deductions.partition_point(|d| d.0 <= december) {
            0 => 0.0,
            i => self.wealth_deductions[i - 1].1 * index,
        };
        let one_off: f64 = self
            .one_off_wealth
            .iter()
            .filter(|(y, _)| *y == m.year())
            .map(|(_, a)| a)
            .sum();
        let taxable = (wealth - ongoing - one_off).max(0.0);
        (
            taxable,
            Some(WealthBasis {
                wealth,
                ongoing,
                one_off,
                taxable,
            }),
        )
    }

    /// #131: Deductible pillar 3a contributions of person `i` per year (at most the maximum of the settings); the only
    /// place of the limit. #152: without a pension fund (large pillar 3a) 20 % of the earned income `earned` of the
    /// year, at most the maximum of the large pillar 3a. `index`: price index (#66).
    fn pillar3a_cap(&self, i: usize, earned: f64, index: f64) -> f64 {
        if self.pillar3a_large[i] {
            (LARGE_PILLAR3A_SHARE * earned).min(self.pillar3a_max_large * index)
        } else {
            self.pillar3a_max * index
        }
    }

    /// #131: Adds month `m` with the income `income` to the deductions of the tax year.
    fn add_deduction_month(&self, acc: &mut YearDeductions, m: YearMonth, income: f64) {
        acc.months += 1;
        match self.deduction_at(m) {
            Some(d) if d.kind == crate::model::DeductionKind::Canton => {
                // Own tables: no proposal (#131).
                if !self.locations.is_empty() {
                    acc.canton_months += 1;
                    acc.extra += d.extra;
                }
            }
            Some(d) => {
                acc.own_amount += d.amount;
                acc.own_rate += d.rate * income;
            }
            None => {}
        }
    }

    /// Income tax of the year of month `m` on the income `income` (earned income + PK pension + AHV) with the
    /// deductions `acc` (#131) and its basis; `earned`: earned income per person (proposal of the canton, #63).
    /// `index`: price index for the amounts (#66). #151: with the place of residence on 31 December, for the whole year.
    fn income_tax(
        &self,
        m: YearMonth,
        income: f64,
        earned: &[f64],
        acc: &YearDeductions,
        index: f64,
    ) -> (f64, TaxBasis) {
        let months = f64::from(acc.months.max(1));
        let share = f64::from(acc.canton_months) / months;
        let own = acc.own_amount / months * index + acc.own_rate;
        // #230: additional amount to the proposal of the canton (prices of the start) and one-off deductions (nominal)
        let extra = acc.extra / months * index;
        let one_off = acc.one_off;
        let pillar3a = if self.deduct_pillar3a {
            (0..self.people.len().min(2))
                .map(|i| acc.pillar3a[i].min(self.pillar3a_cap(i, earned.get(i).copied().unwrap_or(0.0), index)))
                .sum()
        } else {
            0.0
        };
        let mut basis = TaxBasis {
            income,
            proposal_share: share,
            own,
            extra,
            one_off,
            pillar3a,
            ..Default::default()
        };
        let location = self.location_of_year(m);
        // #151: the place of residence of the year in the result only if it changes during the plan.
        if self.locations.len() > 1 {
            basis.residence = location.map(|l| l.place.clone());
        }
        let tax = match location.map(|l| &l.resolved) {
            Some(r) => {
                if share > 0.0 {
                    let (pc, pf) = r.proposal(income, earned, index);
                    basis.proposal_canton = share * pc;
                    basis.proposal_federal = share * pf;
                    basis.parts_canton = r.canton_deductions.parts(income, earned, index);
                    basis.parts_federal = r.federal_deductions.parts(income, earned, index);
                }
                let common = own + extra + one_off + pillar3a;
                basis.taxable_canton = (income - (basis.proposal_canton + common)).max(0.0);
                basis.taxable_federal = (income - (basis.proposal_federal + common)).max(0.0);
                r.income_tax_on(basis.taxable_canton, basis.taxable_federal, index)
            }
            None => {
                let d = own + extra + one_off + pillar3a;
                // Without deductions exactly the income as before #131.
                let taxable = if d == 0.0 { income } else { (income - d).max(0.0) };
                basis.taxable_canton = taxable;
                basis.taxable_federal = taxable;
                tax::tax_indexed(&self.taxes.income, taxable, self.taxes.multiplier, index)
            }
        };
        (tax, basis)
    }

    /// Wealth tax of the year of month `m` (#151: place of residence on 31 December).
    fn wealth_tax(&self, m: YearMonth, wealth: f64, index: f64) -> f64 {
        match self.location_of_year(m).map(|l| &l.resolved) {
            Some(r) => r.wealth_tax(wealth, index),
            None => tax::tax_indexed(&self.taxes.wealth, wealth, self.taxes.multiplier, index),
        }
    }

    /// #205: The spending rule has a floor (fixed rate, guardrails, VPW, endowment, 3 buckets with guardrails; floor > 0).
    pub fn has_floor(&self) -> bool {
        self.dynamic.is_some_and(|d| d.floor.is_some())
    }

    /// Last payout of a pension fund (retirement of the household).
    pub fn retire_at(&self) -> Option<YearMonth> {
        self.people.iter().filter_map(|p| p.pk_at).max()
    }

    pub fn months(&self) -> usize {
        self.months
    }

    /// Planned pillar 3a contributions in month `m` (until before the person's AHV payout).
    fn pillar3a_contribution(&self, m: YearMonth) -> f64 {
        self.accounts
            .iter()
            .filter(|a| a.ahv.is_none_or(|h| m < h))
            .map(|a| a.contribution_m)
            .sum()
    }

    /// #131: Planned pillar 3a contribution of person `i` in month `m`.
    fn pillar3a_contribution_of(&self, i: usize, m: YearMonth) -> f64 {
        self.accounts
            .get(i)
            .filter(|a| a.ahv.is_none_or(|h| m < h))
            .map_or(0.0, |a| a.contribution_m)
    }

    pub fn age_at(&self, m: YearMonth) -> f64 {
        self.people[0].birth.until(m) as f64 / 12.0
    }

    pub fn run(&self, path: &MarketPath, record: bool) -> SimResult {
        self.run_observed(path, record, None)
    }

    /// #214: Like [`Simulator::run`]; `observer` receives every month as it is computed (also without `record`), so
    /// that metrics are collected without keeping the monthly records.
    pub fn run_observed(&self, path: &MarketPath, record: bool, observer: Option<&mut dyn MonthObserver>) -> SimResult {
        assert!(
            path.stocks.len() >= self.months,
            "Renditepfad ist kürzer als der Simulationszeitraum"
        );
        Run::new(self, path, record).execute(observer)
    }
}

fn compile_property(p: &Property, plan: &Plan, r: &Resolver, end: YearMonth) -> Result<CProp> {
    let res = |t: &Option<crate::time::TimeRef>| -> Result<Option<YearMonth>> {
        Ok(match t {
            Some(t) => r.resolve(t)?,
            None => None,
        })
    };
    let buy = res(&p.buy)?;
    // #196: With «Immobilien nie verkaufen» the planned sale is dropped; the mortgage runs until the end.
    let sell = if plan.realestate.never_sell {
        None
    } else {
        res(&p.sell)?
    };
    let mut tranches = vec![];
    if p.status != PropertyStatus::Rent {
        if let Some(mg) = &p.mortgage {
            let limit = match p.status {
                PropertyStatus::Owned => sell.map_or(end.plus(1), |s| s.min(end.plus(1))),
                _ => end.plus(1),
            };
            for t in &mg.tranches {
                let first = match p.status {
                    PropertyStatus::Owned => t
                        .start
                        .ok_or_else(|| Error::Invalid(msg!(tranche_period_start_missing, name = p.name)))?,
                    _ => match buy {
                        Some(b) => b,
                        None => continue,
                    },
                };
                let mut c = schedule(t, first, limit);
                c.account = account_of(t, plan);
                tranches.push(c);
            }
        }
    }
    let trigger = |t: &Option<crate::time::TimeRef>| -> Result<Trigger> {
        let Some(t) = t else { return Ok(Trigger::None) };
        if t.is_after_sale() {
            return Ok(Trigger::LastSale);
        }
        match t.sale_of() {
            None => Ok(Trigger::None),
            Some(id) => plan
                .realestate
                .properties
                .iter()
                .position(|q| q.id.as_deref() == Some(id))
                .map(Trigger::SaleOf)
                .ok_or_else(|| Error::Invalid(msg!(property_reference_missing, name = p.name, id = id))),
        }
    };
    Ok(CProp {
        buy_trigger: if p.status == PropertyStatus::Planned {
            trigger(&p.buy)?
        } else {
            Trigger::None
        },
        rent_trigger: if p.status == PropertyStatus::Rent {
            trigger(&p.from)?
        } else {
            Trigger::None
        },
        max_loan: p
            .mortgage
            .as_ref()
            .is_some_and(|m| m.amount_mode == AmountMode::MaxLoan),
        tranche_defs: p.mortgage.as_ref().map_or(vec![], |m| m.tranches.clone()),
        name: p.name.clone(),
        status: p.status,
        price: p.price,
        bought: p.bought,
        worth: p.worth.unwrap_or(p.price),
        tax_ratio: match (p.status, p.tax_value) {
            (PropertyStatus::Owned, Some(t)) => t.ratio(p.worth.unwrap_or(p.price)),
            (PropertyStatus::Planned, Some(t)) => t.ratio(p.price),
            _ => 1.0,
        },
        infl_value: p.inflation_value.unwrap_or(p.inflation),
        infl_fix_costs: p.inflation_fix_costs.unwrap_or(p.inflation),
        infl_rent: p.inflation_rent.unwrap_or(true),
        fix_costs: p.fix_costs,
        buy,
        sell,
        rent_from: res(&p.from)?,
        rent_until: res(&p.until)?,
        rent: p.rent.unwrap_or(0.0),
        tranches,
    })
}

/// Person (index) whose pillar 3a receives a tranche's indirect amortization; if not given, the first person.
fn account_of(t: &Tranche, plan: &Plan) -> usize {
    t.pillar3a
        .as_deref()
        .and_then(|id| plan.household.persons.iter().position(|p| p.id == id))
        .unwrap_or(0)
        .min(1)
}

/// Periods of a tranche from `first` until before `limit`; after the last defined period the mortgage is
/// renewed (`renew`) or the remaining debt is repaid at the end (`repay`).
fn schedule(t: &crate::model::Tranche, first: YearMonth, limit: YearMonth) -> CTranche {
    let mut periods = vec![];
    let mut m = first;
    let mut i = 0;
    while m < limit && !t.periods.is_empty() && i < 1000 {
        if i >= t.periods.len() && t.after == AfterLast::Repay {
            break;
        }
        let d = &t.periods[i.min(t.periods.len() - 1)];
        let end = m.plus(((d.term * 12.0).round() as i32).max(1));
        periods.push(SPeriod {
            start: m,
            end,
            index: i,
            kind: d.kind,
            rate: d.rate,
            margin: d.margin,
            amort_q: d.amort,
            direct: d.amort_mode == AmortMode::Direct,
        });
        m = end;
        i += 1;
    }
    let repay_at = match (t.after, periods.last()) {
        (AfterLast::Repay, Some(last)) if last.end < limit => Some(last.end),
        _ => None,
    };
    CTranche {
        name: t.name.clone(),
        amount: t.amount,
        periods,
        repay_at,
        account: 0,
    }
}

// ---------------------------------------------------------------------------------------------
// One run
// ---------------------------------------------------------------------------------------------

struct PropState {
    active: bool,
    done: bool,
    price_paid: f64,
    /// Purchase month (for the holding period); unknown for owned property without a date.
    bought: Option<YearMonth>,
    balances: Vec<f64>,
    emergency_pending: bool,
    /// Reason for the emergency sale that was initiated.
    emergency_reason: Option<Message>,
    /// Periods of a purchase «nach Verkauf».
    sched: Option<Vec<CTranche>>,
    /// Month of the sale.
    sold: Option<YearMonth>,
    /// Purchase «nach Verkauf von …» was attempted.
    attempted: bool,
    /// Accrued mortgage interest not yet paid (due at quarter end).
    int_accr: f64,
}

/// #190: First year with a 13th AHV pension (first payout December 2026).
pub const AHV13_FROM_YEAR: i32 = 2026;

/// #152: Large pillar 3a (without a pension fund): contributions and deduction up to this share of the earned income
/// (BVV 3 Art. 7 para. 1 lit. b).
pub const LARGE_PILLAR3A_SHARE: f64 = 0.2;

/// #190: Share of the 13th AHV pension of one month in centimes: one twelfth of the monthly pension, rounded to the
/// centime (half away from zero).
fn ahv13_share_centimes(monthly: f64) -> f64 {
    (monthly / 12.0 * 100.0).round()
}

struct Run<'a> {
    s: &'a Simulator,
    path: &'a MarketPath,
    record: bool,
    cash: f64,
    stocks: f64,
    bonds: f64,
    people: Vec<PersonState>,
    /// #191: AHV of the household (cap, reduction and supplement as the AHV computes them).
    ahv: crate::ahv::Calc,
    /// #190: Monthly shares of the 13th AHV pension accrued in the current calendar year, in centimes.
    ahv13_centimes: f64,
    /// Taxes from the month's events (3a payout, property gain).
    event_tax: f64,
    infl_idx: f64,
    /// Price index at the last January or the start (basis of the annual adjustments).
    infl_idx_jan: f64,
    /// Mortgage interest that fell due on a sale in the middle of a quarter.
    interest_settled: f64,
    /// Assessed income and wealth tax not yet paid.
    tax_pending: f64,
    /// Shares of `tax_pending` (income and wealth tax) and of the current month's taxes by kind.
    pending_income: f64,
    pending_wealth: f64,
    month_capital_tax: f64,
    month_gain_tax: f64,
    year_income: f64,
    /// #63: Earned income of the calendar year per person (mode «Wohnort»: deductions).
    year_earned: [f64; 2],
    /// #131: Deductions of the calendar year (rows, pillar 3a).
    year_ded: YearDeductions,
    /// Lump sums of the calendar year per taxpayer: one entry for the household, or one per person
    /// for unmarried couples (#150).
    year_capital: Vec<f64>,
    props: Vec<PropState>,
    /// Pillar 3a balance per person and whether the remainder has already been paid out (#52).
    p3a: Vec<f64>,
    p3a_done: Vec<bool>,
    /// Pillar 3a payout plan per account for display (#55), only with `record`.
    p3a_view: Vec<Pillar3aView>,
    /// Pillar 3a contribution in this month.
    month_p3a_in: f64,
    /// Month in which, after the sale of the last property, a property is bought or rented.
    after_sale: Option<YearMonth>,
    renting: bool,
    events: Vec<Event>,
    renewals: Vec<Renewal>,
    property_events: Vec<PropertyEvent>,
    bucket_refills: Vec<BucketRefill>,
    /// Basis of the bucket sizes of the current year (set in January or at trigger A).
    basis: Option<NeedBasis>,
    /// External inflows of this month that are booked into bucket 3 after the returns (until then in `cash`).
    inflow3: f64,
    /// Booking journal of this month's inflows (from, to, amount, occasion); 0 = external, 4 = reservation. Written to
    /// the log by `flush_inflow`.
    inflow_log: Vec<(u8, u8, f64, Message)>,
    /// One-off expenses of this month outside the events (rest of the equity at a purchase, lump sums and
    /// mortgage repayments). They belong to the month's expenses and are paid only after the look-ahead.
    month_out: f64,
    /// #133: Ledger of the current month.
    ledger: Ledger,
    /// Reasons for special case A in this month.
    a_reasons: Vec<Message>,
    /// Reserved sale proceeds for a planned purchase; held in `cash`, but belongs to no bucket.
    reserved: f64,
    /// All buckets empty, no property left: bucket 1 may be negative, buckets 2 and 3 are shut down.
    phase8: bool,
    /// Stock index at the end of every month (for the return of the last 12 months), starting with the path's lead-in
    /// (#103): first the level before the first lead-in month, then one per lead-in month; the level at the end of
    /// month `k` is at `hist_off + k`.
    stock_hist: Vec<f64>,
    hist_off: usize,
    /// Amortization share of this month's ongoing housing costs (not part of the need).
    month_amort: f64,
    /// Occasions on which the withdrawal strategy is applied at once in this month (inflows, sale without a follow-up purchase).
    apply_reasons: Vec<Message>,
    /// The withdrawal strategy is applied this month in addition to the January application.
    apply_now: bool,
    /// The strategy started for the first time this month (retirement); this is already in the log.
    strategy_started: bool,
    /// Month in which sale proceeds were reserved for a purchase in the following month.
    reserved_sale: Option<YearMonth>,
    /// Reserved sale proceeds of the current month (stay liquid, are not invested).
    reserved_cash: f64,
    records: Vec<MonthRecord>,
    summary: Summary,
    /// «3-Topf»: buckets are split (from retirement).
    buckets_on: bool,
    /// Stock return in the current calendar year (factor) and in the previous year.
    stock_year: f64,
    stock_prev_year: f64,
    stock_idx: f64,
    bond_idx: f64,
    /// Ongoing housing costs of this month (for the living costs of the spending rule in the net need, #6).
    month_running: f64,
    /// Fixed rate or guardrails: annual withdrawal (nominal), initial rate and whether it applies.
    budget: f64,
    budget_rate0: f64,
    budget_on: bool,
    /// #218: state for the characteristic curve of the stock share (only if the plan has one).
    share: Option<crate::state::Tracker>,
}

/// Pension fund and AHV of one person in the run.
#[derive(Clone, Debug, Default)]
struct PersonState {
    pk: f64,
    /// Accrued PK interest, credited in December or at payout.
    pk_accrued: f64,
    retired: bool,
    pk_pension: f64,
}

impl<'a> Run<'a> {
    fn new(s: &'a Simulator, path: &'a MarketPath, record: bool) -> Self {
        let props = s
            .props
            .iter()
            .map(|p| {
                let owned = p.status == PropertyStatus::Owned;
                PropState {
                    active: owned,
                    done: false,
                    price_paid: p.price,
                    bought: if owned { p.bought } else { None },
                    balances: p.tranches.iter().map(|t| if owned { t.amount } else { 0.0 }).collect(),
                    emergency_pending: false,
                    emergency_reason: None,
                    sched: None,
                    sold: None,
                    attempted: false,
                    int_accr: 0.0,
                }
            })
            .collect();
        Run {
            s,
            path,
            record,
            cash: s.cash0,
            stocks: s.stocks0,
            bonds: s.bonds0,
            people: s
                .people
                .iter()
                .map(|p| PersonState {
                    pk: p.pk_capital,
                    ..Default::default()
                })
                .collect(),
            ahv: crate::ahv::Calc::new(s.people.iter().map(|p| p.ahv.clone()).collect(), s.couple_cap),
            ahv13_centimes: 0.0,
            event_tax: 0.0,
            infl_idx: 1.0,
            infl_idx_jan: 1.0,
            interest_settled: 0.0,
            tax_pending: 0.0,
            pending_income: 0.0,
            pending_wealth: 0.0,
            month_capital_tax: 0.0,
            month_gain_tax: 0.0,
            year_income: 0.0,
            year_earned: [0.0; 2],
            year_ded: YearDeductions::default(),
            year_capital: vec![0.0; s.people.len().max(1)],
            props,
            p3a: s.accounts.iter().map(|a| a.balance).collect(),
            p3a_done: vec![false; s.accounts.len()],
            p3a_view: s
                .accounts
                .iter()
                .map(|a| Pillar3aView {
                    person: a.person.clone(),
                    ..Default::default()
                })
                .collect(),
            month_p3a_in: 0.0,
            after_sale: None,
            renting: false,
            events: vec![],
            renewals: vec![],
            property_events: vec![],
            bucket_refills: vec![],
            basis: None,
            inflow3: 0.0,
            inflow_log: vec![],
            month_out: 0.0,
            ledger: Ledger::default(),
            a_reasons: vec![],
            reserved: 0.0,
            phase8: false,
            stock_hist: pre_levels(&path.pre_stocks, s.months),
            hist_off: path.pre_stocks.len() + 1,
            month_amort: 0.0,
            apply_reasons: vec![],
            apply_now: false,
            strategy_started: false,
            reserved_sale: None,
            reserved_cash: 0.0,
            records: if record { Vec::with_capacity(s.months) } else { vec![] },
            summary: Summary {
                min_liquid: f64::INFINITY,
                ..Default::default()
            },
            buckets_on: false,
            stock_year: 1.0,
            stock_prev_year: 0.0,
            stock_idx: 1.0,
            bond_idx: 1.0,
            month_running: 0.0,
            budget: 0.0,
            budget_rate0: 0.0,
            budget_on: false,
            share: s.share_curve.as_ref().map(|_| crate::state::Tracker::default()),
        }
    }

    fn pk_pension(&self) -> f64 {
        self.people.iter().map(|p| p.pk_pension).sum()
    }

    /// AHV pensions of the household; for married couples together at most the cap.
    fn ahv(&self) -> f64 {
        self.ahv.total()
    }

    /// #190: Accrues the share of the 13th AHV pension for month `m` (one twelfth of the household AHV `monthly`,
    /// rounded to the centime) and returns the 13th pension in December: the year's shares, rounded to whole francs.
    /// Not before [`AHV13_FROM_YEAR`]; nothing without a pension in December.
    fn thirteenth_ahv(&mut self, m: YearMonth, monthly: f64) -> f64 {
        if m.year() < AHV13_FROM_YEAR {
            return 0.0;
        }
        self.ahv13_centimes += ahv13_share_centimes(monthly);
        if !m.is_december() {
            return 0.0;
        }
        let centimes = std::mem::take(&mut self.ahv13_centimes);
        if monthly > 0.0 {
            (centimes / 100.0).round()
        } else {
            0.0
        }
    }

    /// At least one person with a payout date is retired (start of the withdrawal phase for all strategies except
    /// «Standard», #6/#47).
    fn any_retired(&self) -> bool {
        self.people
            .iter()
            .zip(&self.s.people)
            .any(|(p, c)| c.pk_at.is_some() && p.retired)
    }

    fn event(&mut self, month: YearMonth, kind: EventKind, text: Message) {
        self.events.push(Event { month, kind, text });
    }

    fn worth(&self, i: usize) -> f64 {
        let p = &self.s.props[i];
        let base = if p.status == PropertyStatus::Owned {
            p.worth
        } else {
            p.price
        };
        if p.infl_value {
            base * self.infl_idx
        } else {
            base
        }
    }

    /// #133: Market value of the owned properties.
    fn active_worth(&self) -> f64 {
        (0..self.props.len())
            .filter(|&i| self.props[i].active)
            .map(|i| self.worth(i))
            .sum()
    }

    fn equity(&self) -> f64 {
        (0..self.props.len())
            .filter(|&i| self.props[i].active)
            .map(|i| self.worth(i) - self.props[i].balances.iter().sum::<f64>())
            .sum()
    }

    /// #229: Tax value of the owned properties (market value × ratio of the tax value).
    fn tax_worth(&self) -> f64 {
        (0..self.props.len())
            .filter(|&i| self.props[i].active)
            .map(|i| self.worth(i) * self.s.props[i].tax_ratio)
            .sum()
    }

    /// #229: Taxable property wealth: tax value minus mortgage (without a tax value the same as [`Self::equity`]).
    fn tax_equity(&self) -> f64 {
        (0..self.props.len())
            .filter(|&i| self.props[i].active)
            .map(|i| self.worth(i) * self.s.props[i].tax_ratio - self.props[i].balances.iter().sum::<f64>())
            .sum()
    }

    /// Lump sums in the same calendar year are added together for the rate: for the household, or per
    /// person (`person`) for unmarried couples (#150). #64: In the mode «Wohnort» the table of the place of residence
    /// applies (multipliers included); unmarried couples pay per person with the tariff for single persons.
    /// #151: place of residence in the month `m` of the payout (StHG Art. 4b para. 1).
    fn capital_tax(&mut self, person: usize, m: YearMonth, amount: f64) -> f64 {
        let idx = self.tax_index();
        let (table, multiplier) = match self.s.location_at(m) {
            Some(l) => (&l.resolved.capital, 1.0),
            None => (&self.s.taxes.capital, self.s.taxes.multiplier),
        };
        let k = if self.s.capital_per_person {
            person.min(self.year_capital.len() - 1)
        } else {
            0
        };
        let before = tax::tax_indexed(table, self.year_capital[k], multiplier, idx);
        let after = tax::tax_indexed(table, self.year_capital[k] + amount, multiplier, idx);
        self.year_capital[k] += amount;
        // #134: With a table whose tax falls (warning of the validation) the increment would be negative.
        (after - before).max(0.0)
    }

    /// #66: Price index for the amounts of the tax tables (cumulative inflation since the simulation start) if the
    /// plan chose «tariffs with inflation»; otherwise 1.0 (tables in nominal francs).
    fn tax_index(&self) -> f64 {
        if self.s.taxes.indexed {
            self.infl_idx
        } else {
            1.0
        }
    }

    fn sell(&mut self, i: usize, m: YearMonth, kind: EventKind) {
        let worth = self.worth(i);
        let debt: f64 = self.props[i].balances.iter().sum();
        // Holding period in months (#65: rule set of the canton) or in full years (own rules, as before); unknown for
        // owned property without a purchase date (no surcharge or reduction).
        // #151: rules of the place of residence in the month of the sale (one entry with own rules).
        let rules = &self.s.gain_rules[i];
        let rule = &rules[self.s.location_index(m).min(rules.len() - 1)];
        let months = self.props[i]
            .bought
            .map(|b| b.until(m).max(0))
            .map(|h| if rule.months { h } else { h / 12 * 12 });
        let paid = self.props[i].price_paid;
        let gain_tax = tax::property_gain_tax_full(rule.def.as_ref(), worth - paid, paid, months, rule.multiplier);
        self.cash += worth - debt - gain_tax;
        self.ledger.taxes += gain_tax;
        self.interest_settled += std::mem::take(&mut self.props[i].int_accr);
        self.event_tax += gain_tax;
        self.month_gain_tax += gain_tax;
        let ps = &mut self.props[i];
        let reason = ps.emergency_reason.take();
        ps.active = false;
        ps.done = true;
        ps.emergency_pending = false;
        ps.sold = Some(m);
        ps.balances.iter_mut().for_each(|b| *b = 0.0);
        let emergency = kind == EventKind::EmergencySale;
        let name = self.s.props[i].name.clone();
        let mut pe = PropertyEvent::new(
            m,
            if kind == EventKind::EmergencySale {
                PropertyEventKind::EmergencySale
            } else {
                PropertyEventKind::Sale
            },
            &name,
        );
        pe.proceeds = Some(worth);
        pe.debt_repaid = Some(debt);
        pe.gain_tax = Some(gain_tax);
        pe.net = Some(worth - debt - gain_tax);
        if kind == EventKind::EmergencySale {
            self.summary.emergency_sales += 1;
            pe.reason = Some(reason.unwrap_or_else(|| msg!(reason_emergency_criterion)));
        }
        // Follow-up purchase next month: reserve the proceeds, apply the withdrawal strategy only after the purchase.
        // Otherwise the strategy is applied at once in the month of the sale.
        let net = worth - debt - gain_tax;
        if net > 0.0 {
            if self.follow_up_purchase(i, m) {
                self.reserved_sale = Some(m);
                self.reserved_cash += net;
                self.reserved += net;
                self.mark_reservation(net, msg!(why_sale, name = name));
                pe.note = Some(msg!(note_sale_reserved));
                let text = msg!(
                    event_sale_reserved,
                    name = name,
                    amount = net,
                    currency = self.s.currency
                );
                self.event(m, EventKind::Withdrawal, text);
            } else {
                self.apply_reasons.push(msg!(reason_sale_no_follow_up, name = name));
                pe.note = Some(msg!(note_withdrawal_applied));
                self.mark_inflow(net, msg!(why_sale, name = name));
            }
        }
        self.property_events.push(pe);
        self.event(
            m,
            kind,
            if emergency {
                msg!(
                    event_emergency_sale,
                    name = name,
                    value = worth,
                    debt = debt,
                    tax = gain_tax,
                    currency = self.s.currency
                )
            } else {
                msg!(
                    event_sale,
                    name = name,
                    value = worth,
                    debt = debt,
                    tax = gain_tax,
                    currency = self.s.currency
                )
            },
        );
        if !self.props.iter().any(|p| p.active) {
            self.after_sale = Some(m.plus(1));
        }
    }

    /// Is a purchase of another property planned in the month after the sale of property `sold` (month `m`)?
    /// Counts the fixed purchase month M+1 and purchases «nach Verkauf» of this property or of the last property.
    fn follow_up_purchase(&self, sold: usize, m: YearMonth) -> bool {
        let other_active = self.props.iter().enumerate().any(|(j, p)| j != sold && p.active);
        self.s.props.iter().enumerate().any(|(j, p)| {
            if j == sold || p.status != PropertyStatus::Planned {
                return false;
            }
            let st = &self.props[j];
            if st.active || st.done || st.attempted {
                return false;
            }
            match p.buy_trigger {
                Trigger::None => p.buy == Some(m.plus(1)),
                Trigger::SaleOf(s) => s == sold,
                Trigger::LastSale => !other_active,
            }
        })
    }

    /// Purchases «nach Verkauf von …» in the month after the respective sale. Afterwards, if no
    /// property is owned any more: buy the next property «nach Verkauf», if affordable and
    /// financeable; otherwise the rents «nach Verkauf» start.
    fn after_sales(&mut self, m: YearMonth) {
        let s = self.s;
        for i in 0..s.props.len() {
            let Trigger::SaleOf(j) = s.props[i].buy_trigger else {
                continue;
            };
            let ps = &self.props[i];
            if ps.active || ps.done || ps.attempted || !self.props[j].sold.is_some_and(|sm| m > sm) {
                continue;
            }
            self.props[i].attempted = true;
            if !self.try_buy(i, m) {
                // Purchase not financeable: the purchase is dropped, the follow-up rules apply in the following month
                // (rent «nach Verkauf» of this property, further purchases «nach Verkauf» of it).
                self.props[i].done = true;
                self.props[i].sold = Some(m);
                let text = msg!(event_purchase_dropped, name = s.props[i].name);
                self.event(m, EventKind::Purchase, text);
                if !self.props.iter().any(|p| p.active) {
                    self.after_sale.get_or_insert(m);
                }
            }
        }
        if !self.after_sale.is_some_and(|a| m >= a) {
            return;
        }
        self.after_sale = None;
        if self.props.iter().any(|p| p.active) {
            return;
        }
        for i in 0..s.props.len() {
            let p = &s.props[i];
            if p.buy_trigger == Trigger::LastSale && !self.props[i].active && !self.props[i].done && self.try_buy(i, m)
            {
                return;
            }
        }
        if !self.renting && s.props.iter().any(|p| p.rent_trigger == Trigger::LastSale) {
            self.renting = true;
            let names: Vec<&str> = s
                .props
                .iter()
                .filter(|p| p.rent_trigger == Trigger::LastSale)
                .map(|p| p.name.as_str())
                .collect();
            self.event(m, EventKind::RentStart, msg!(event_rent_start, names = names));
        }
    }

    /// Ongoing housing costs per month (interest, amortization, fix costs) of all active properties, to estimate the
    /// emergency sale criterion after a lump sum. The balances of property `only` are scaled
    /// by `keep`; its tranches come from `only_tranches` (they are taken out during the renewal).
    fn running_estimate(&self, m: YearMonth, k: usize, only: usize, only_tranches: &[CTranche], keep: f64) -> f64 {
        let s = self.s;
        let saron = self.path.saron[k];
        let mut total = 0.0;
        for (j, p) in s.props.iter().enumerate() {
            if !self.props[j].active {
                continue;
            }
            total += p.fix_costs * if p.infl_fix_costs { self.infl_idx } else { 1.0 };
            let tranches = if j == only {
                only_tranches
            } else {
                self.props[j].sched.as_deref().unwrap_or(&p.tranches)
            };
            for (t, tr) in tranches.iter().enumerate() {
                let b = self.props[j].balances[t] * if j == only { keep } else { 1.0 };
                let per = tr
                    .periods
                    .iter()
                    .find(|x| x.start <= m && m < x.end)
                    .or(tr.periods.last());
                if let Some(x) = per {
                    total += b * x.rate(saron) / 12.0;
                    if x.amort_q > 0.0 && b > 0.0 {
                        total += x.amort_q / 3.0;
                    }
                }
            }
        }
        total
    }

    /// Rate of the new period: average of the tranches, weighted by their balances (times `keep`).
    fn renewal_rate(&self, i: usize, m: YearMonth, k: usize, tranches: &[CTranche], keep: f64) -> f64 {
        let saron = self.path.saron[k];
        let (mut sum, mut weighted) = (0.0, 0.0);
        for (t, tr) in tranches.iter().enumerate() {
            let b = self.props[i].balances[t] * keep;
            if let Some(x) = tr
                .periods
                .iter()
                .find(|x| x.start <= m && m < x.end)
                .or(tr.periods.last())
            {
                sum += b;
                weighted += b * x.rate(saron);
            }
        }
        if sum > 0.0 {
            weighted / sum
        } else {
            0.0
        }
    }

    /// Records a renewal in the log (`renewals`) and in the common list of property events.
    fn push_renewal(&mut self, record: Renewal, rate: f64) {
        let mut pe = PropertyEvent::new(record.month, PropertyEventKind::Renewal, &record.property);
        pe.affordable = Some(record.affordable);
        pe.debt_before = Some(record.debt_before);
        pe.debt_after = Some(record.debt_after);
        pe.lump_sum = (record.lump_sum > 0.0).then_some(record.lump_sum);
        pe.rate = Some(rate);
        pe.reason = record.emergency_sale.clone();
        self.property_events.push(pe);
        self.renewals.push(record);
    }

    /// Affordability at the renewal of a mortgage period (imputed rate, income only).
    /// - Affordable: nothing to do.
    /// - Not affordable: minimal lump sum so that it is affordable again. It is paid from wealth
    ///   and the mortgage is renewed at a reduced amount if wealth is >= 0 afterwards **and** the emergency sale
    ///   criterion is not triggered after the payment (estimated with the planned spending and the housing costs after the payment).
    /// - Otherwise: no payment, the mortgage is not renewed and an emergency sale is initiated (following month).
    ///
    /// Returns `true` if the mortgage is renewed.
    fn renewal_affordability(&mut self, i: usize, m: YearMonth, k: usize, tranches: &[CTranche]) -> bool {
        let s = self.s;
        let p = &s.props[i];
        let a = &s.afford;
        let debt: f64 = self.props[i].balances.iter().sum();
        if debt <= 0.0 {
            return true;
        }
        let name = p.name.clone();
        let income_m = s.earned(m, self.infl_idx) + self.pk_pension() + self.ahv();
        let fix = if p.fix_costs > 0.0 {
            p.fix_costs * 12.0 * if p.infl_fix_costs { self.infl_idx } else { 1.0 }
        } else {
            self.worth(i) * a.fix_costs
        };
        let max = ((a.sustainability * income_m * 12.0 - fix) / a.imputed_rate).max(0.0);
        let lump = debt - max;
        let mut record = Renewal {
            month: m,
            property: name.clone(),
            affordable: lump < 0.5,
            lump_sum: 0.0,
            debt_before: debt,
            debt_after: debt,
            emergency_sale: None,
        };
        if lump < 0.5 {
            let rate = self.renewal_rate(i, m, k, tranches, 1.0);
            self.push_renewal(record, rate);
            return true;
        }
        let liquid = self.cash + self.stocks + self.bonds;
        let keep = max / debt;
        // After the payment: wealth >= 0 and emergency sale criterion not triggered?
        let reason = if liquid - lump < 0.0 {
            Some("wealth < 0")
        } else {
            let shortfall =
                s.planned_living(m) * self.infl_idx + self.running_estimate(m, k, i, tranches, keep) - income_m;
            (liquid - lump - s.emergency.at(m) * shortfall < 0.0).then_some("emergency sale criterion")
        };
        if reason.is_some() && s.never_sell {
            // #196: No sale: the mortgage is renewed in full; the event reports «nicht tragbar».
            let text = msg!(
                event_mortgage_unaffordable_kept,
                name = name,
                lump = lump,
                currency = self.s.currency
            );
            self.event(m, EventKind::MortgageRenewal, text);
            self.summary.unaffordable_renewals += 1;
            let rate = self.renewal_rate(i, m, k, tranches, 1.0);
            self.push_renewal(record, rate);
            return true;
        }
        if let Some(reason) = reason {
            let below_zero = reason == "wealth < 0";
            let reason = if below_zero {
                msg!(reason_wealth_negative)
            } else {
                msg!(reason_emergency_criterion)
            };
            let text = if below_zero {
                msg!(
                    event_mortgage_unaffordable_wealth,
                    name = name,
                    lump = lump,
                    wealth = liquid,
                    currency = self.s.currency
                )
            } else {
                msg!(
                    event_mortgage_unaffordable_criterion,
                    name = name,
                    lump = lump,
                    currency = self.s.currency
                )
            };
            self.event(m, EventKind::MortgageRenewal, text);
            self.props[i].emergency_pending = true;
            self.props[i].emergency_reason = Some(reason.clone());
            self.summary.affordability_sales += 1;
            record.emergency_sale = Some(reason);
            let rate = self.renewal_rate(i, m, k, tranches, 1.0);
            self.push_renewal(record, rate);
            return false;
        }
        let rate = self.renewal_rate(i, m, k, tranches, keep);
        self.props[i].balances.iter_mut().for_each(|b| *b *= keep);
        self.cash -= lump;
        self.month_out += lump;
        self.summary.lump_sums += 1;
        let text = msg!(
            event_mortgage_lump_sum,
            name = name,
            lump = lump,
            debt = max,
            currency = self.s.currency
        );
        self.event(m, EventKind::MortgageRenewal, text);
        record.lump_sum = lump;
        record.debt_after = max;
        self.push_renewal(record, rate);
        true
    }

    /// Purchase with affordability check. If the desired mortgage is not affordable, it is reduced
    /// and the difference is paid from wealth, as far as it suffices.
    fn try_buy(&mut self, i: usize, m: YearMonth) -> bool {
        let s = self.s;
        let p = &s.props[i];
        let a = &s.afford;
        let price = if p.infl_value { p.price * self.infl_idx } else { p.price };
        let entered: f64 = p.tranche_defs.iter().map(|t| t.amount).sum();
        let liquid = self.cash + self.stocks + self.bonds;
        // Affordability: income only (earned income, pensions), no withdrawal from wealth.
        let income = (s.earned(m, self.infl_idx) + self.pk_pension() + self.ahv()) * 12.0;
        let fix = if p.fix_costs > 0.0 {
            p.fix_costs * 12.0
        } else {
            price * a.fix_costs
        };
        // Desired mortgage: maximum loan-to-value or the entered amounts, at most the loan-to-value limit.
        let ltv = price * a.max_loan_to_value;
        let cap = if p.max_loan { ltv } else { entered.min(ltv) };
        let max = ((a.sustainability * income - fix) / a.imputed_rate).max(0.0);
        let mortgage = cap.min(max);
        // One-off repayment: part of the desired mortgage paid from wealth for affordability.
        let lump = cap - mortgage;
        if liquid < price - mortgage {
            self.event(
                m,
                EventKind::Purchase,
                msg!(
                    event_purchase_impossible,
                    name = p.name,
                    needed = (price - mortgage),
                    available = liquid,
                    currency = self.s.currency
                ),
            );
            return false;
        }
        // The mortgage is split across the tranches in proportion to the entered amounts.
        let n = p.tranche_defs.len().max(1) as f64;
        let share = |t: &Tranche| {
            if entered > 0.0 {
                t.amount / entered
            } else {
                1.0 / n
            }
        };
        let sched: Vec<CTranche> = p
            .tranche_defs
            .iter()
            .map(|t| {
                let mut c = schedule(t, m, s.end.plus(1));
                c.amount = mortgage * share(t);
                c.account = t
                    .pillar3a
                    .as_deref()
                    .and_then(|id| s.person_ids.iter().position(|p| p == id))
                    .unwrap_or(0)
                    .min(1);
                c
            })
            .collect();
        let ps = &mut self.props[i];
        ps.balances = sched.iter().map(|t| t.amount).collect();
        ps.sched = Some(sched);
        ps.active = true;
        ps.price_paid = price;
        ps.bought = Some(m);
        self.pay_equity(price - mortgage);
        self.renting = false;
        let mut pe = PropertyEvent::new(m, PropertyEventKind::Purchase, &p.name);
        pe.price = Some(price);
        pe.equity = Some(price - mortgage);
        pe.mortgage = Some(mortgage);
        self.property_events.push(pe);
        self.event(
            m,
            EventKind::Purchase,
            if lump >= 0.5 {
                msg!(
                    event_purchase_lump_sum,
                    name = p.name,
                    price = price,
                    mortgage = mortgage,
                    lump = lump,
                    currency = self.s.currency
                )
            } else {
                msg!(
                    event_purchase,
                    name = p.name,
                    price = price,
                    mortgage = mortgage,
                    currency = self.s.currency
                )
            },
        );
        true
    }

    /// Housing costs of this month (including amortization) and smoothed ongoing costs for the reserve.
    fn properties(&mut self, k: usize, m: YearMonth) -> (f64, f64) {
        let s = self.s;
        let saron = self.path.saron[k];
        let mut cost = 0.0;
        let mut running = 0.0;
        self.month_amort = 0.0;
        for i in 0..self.props.len() {
            if self.props[i].emergency_pending && self.props[i].active {
                self.sell(i, m, EventKind::EmergencySale);
            }
        }
        self.after_sales(m);
        for i in 0..self.props.len() {
            let p = &s.props[i];
            match p.status {
                PropertyStatus::Planned
                    if p.max_loan && !self.props[i].active && !self.props[i].done && p.buy == Some(m) =>
                {
                    self.try_buy(i, m);
                }
                PropertyStatus::Planned if !self.props[i].active && !self.props[i].done && p.buy == Some(m) => {
                    let price = if p.infl_value { p.price * self.infl_idx } else { p.price };
                    let ps = &mut self.props[i];
                    ps.active = true;
                    ps.price_paid = price;
                    ps.bought = Some(m);
                    for (b, t) in ps.balances.iter_mut().zip(&p.tranches) {
                        *b = t.amount;
                    }
                    let debt: f64 = ps.balances.iter().sum();
                    self.pay_equity(price - debt);
                    self.renting = false;
                    let mut pe = PropertyEvent::new(m, PropertyEventKind::Purchase, &p.name);
                    pe.price = Some(price);
                    pe.equity = Some(price - debt);
                    pe.mortgage = Some(debt);
                    self.property_events.push(pe);
                    self.event(
                        m,
                        EventKind::Purchase,
                        msg!(
                            event_purchase,
                            name = p.name,
                            price = price,
                            mortgage = debt,
                            currency = self.s.currency
                        ),
                    );
                }
                PropertyStatus::Rent => {
                    let from = match p.rent_trigger {
                        Trigger::None => p.rent_from.is_some_and(|f| m >= f),
                        Trigger::LastSale => self.renting,
                        Trigger::SaleOf(j) => self.props[j].sold.is_some_and(|sm| m > sm),
                    };
                    let until = p.rent_until.is_none_or(|u| m < u);
                    if from && until {
                        let rent = p.rent * if p.infl_rent { self.infl_idx } else { 1.0 };
                        cost += rent;
                        running += rent;
                    }
                }
                _ => {}
            }
            if !self.props[i].active {
                continue;
            }
            let fix = p.fix_costs * if p.infl_fix_costs { self.infl_idx } else { 1.0 };
            cost += fix;
            running += fix;
            // Periods of a purchase «nach Verkauf» are held in the run state; take them out briefly.
            let dynamic = self.props[i].sched.take();
            let tranches = dynamic.as_deref().unwrap_or(&p.tranches);
            // When a tranche renews its period, the bank re-checks the affordability of the whole mortgage.
            let renews = k > 0
                && tranches.iter().enumerate().any(|(j, t)| {
                    self.props[i].balances[j] > 0.0 && t.periods.iter().any(|x| x.index > 0 && x.start == m)
                });
            // Not affordable and not payable: the mortgage is not renewed (emergency sale initiated).
            let renewed = !renews || self.renewal_affordability(i, m, k, tranches);
            for (j, t) in tranches.iter().enumerate() {
                let per = t.periods.iter().find(|x| x.start <= m && m < x.end);
                let b = self.props[i].balances[j];
                if let Some(x) = per {
                    if x.index > 0 && x.start == m && b > 0.0 && k > 0 && renewed {
                        let tn = if tranches.len() > 1 {
                            crate::message::arg(&msg!(suffix_name, name = t.name))
                        } else {
                            crate::message::Arg::Null
                        };
                        self.event(
                            m,
                            EventKind::MortgageRenewal,
                            msg!(
                                event_mortgage_period,
                                name = p.name,
                                tranche = tn,
                                rate = x.rate(saron),
                                amount = b,
                                currency = self.s.currency
                            ),
                        );
                    }
                }
                if t.repay_at == Some(m) && b > 0.0 {
                    self.cash -= b;
                    self.month_out += b;
                    self.props[i].balances[j] = 0.0;
                    self.event(
                        m,
                        EventKind::MortgageRepayment,
                        msg!(
                            event_mortgage_repaid,
                            name = p.name,
                            amount = b,
                            currency = self.s.currency
                        ),
                    );
                    continue;
                }
                // #50: In the month of the AHV payout an indirectly amortized tranche switches to direct; shown as an event
                // if it was amortized indirectly before and amortization continues from now on.
                if k > 0 && b > 0.0 {
                    let a = t.account.min(s.accounts.len().saturating_sub(1));
                    if let Some(c) = s.accounts.get(a).filter(|c| c.ahv == Some(m)) {
                        let was_indirect = t.periods.iter().any(|x| !x.direct && x.amort_q > 0.0 && x.start < m);
                        if let Some(x) = per.filter(|x| x.amort_q > 0.0 && was_indirect) {
                            let tn = if tranches.len() > 1 {
                                crate::message::arg(&msg!(suffix_name, name = t.name))
                            } else {
                                crate::message::Arg::Null
                            };
                            self.event(
                                m,
                                EventKind::AmortizationSwitch,
                                msg!(
                                    event_amortization_direct,
                                    name = p.name,
                                    tranche = tn,
                                    who = c.who,
                                    amount = (x.amort_q * 4.0),
                                    currency = self.s.currency
                                ),
                            );
                        }
                    }
                }
                let rate = per.or(t.periods.last()).map_or(0.0, |x| x.rate(saron));
                let interest = b * rate / 12.0;
                self.props[i].int_accr += interest;
                running += interest;
                if let Some(x) = per {
                    if x.amort_q > 0.0 && b > 0.0 {
                        running += x.amort_q / 3.0;
                        self.month_amort += x.amort_q / 3.0;
                        if m.is_quarter_end() {
                            // #50: Indirect only until before the person's AHV payout; direct afterwards.
                            let a = t.account.min(s.accounts.len().saturating_sub(1));
                            let after_ahv = s.accounts.get(a).is_some_and(|c| c.ahv.is_some_and(|h| m >= h));
                            if x.direct || after_ahv || s.accounts.is_empty() {
                                let a = x.amort_q.min(b);
                                self.props[i].balances[j] -= a;
                                cost += a;
                                self.ledger.amortization += a;
                            } else {
                                self.p3a[a] += x.amort_q;
                                // #131: indirect amortization is a pillar 3a contribution
                                if let Some(y) = self.year_ded.pillar3a.get_mut(a) {
                                    *y += x.amort_q;
                                }
                                cost += x.amort_q;
                                self.ledger.amortization += x.amort_q;
                            }
                        }
                    }
                }
            }
            self.props[i].sched = dynamic;
            // Interest accrues monthly and is paid at quarter end (the remainder at the end of the plan).
            if m.is_quarter_end() || k + 1 == s.months {
                cost += std::mem::take(&mut self.props[i].int_accr);
            }
            if p.status != PropertyStatus::Rent && p.sell == Some(m) {
                self.sell(i, m, EventKind::Sale);
            }
        }
        cost += std::mem::take(&mut self.interest_settled);
        (cost, running)
    }

    /// Pillar 3a (#52): interest, contribution until before the AHV payout, planned payouts and payout of the remainder in
    /// the month of the AHV payout. Returns the month's contribution (it counts as spending).
    fn pillar3a(&mut self, m: YearMonth) -> f64 {
        let s = self.s;
        let mut paid_in = 0.0;
        for a in 0..s.accounts.len() {
            let c = &s.accounts[a];
            if self.p3a_done[a] {
                continue;
            }
            self.ledger.p3a_interest += self.p3a[a] * c.rate_m;
            self.p3a[a] *= 1.0 + c.rate_m;
            if c.contribution_m > 0.0 && c.ahv.is_none_or(|h| m < h) {
                self.p3a[a] += c.contribution_m;
                paid_in += c.contribution_m;
                if let Some(y) = self.year_ded.pillar3a.get_mut(a) {
                    *y += c.contribution_m;
                }
            }
            if self.record {
                self.p3a_view[a].balances.push(self.p3a[a]);
            }
            for &(date, amount, repay, j) in &c.payouts {
                if date != m || c.ahv.is_some_and(|h| date > h) {
                    continue;
                }
                let available = self.p3a[a];
                let gross = amount.unwrap_or(available).min(available);
                if let Some(want) = amount.filter(|w| *w > available + 0.005) {
                    self.event(
                        m,
                        EventKind::Pillar3aPayout,
                        msg!(
                            event_pillar3a_short,
                            who = c.who,
                            planned = want,
                            available = available,
                            currency = self.s.currency
                        ),
                    );
                }
                self.pay_out_3a(a, m, gross, repay, msg!(payout));
                if self.record {
                    let after = self.p3a[a];
                    self.p3a_view[a].payouts.push(Pillar3aPayoutView {
                        index: j,
                        month: m,
                        before: available,
                        paid: gross,
                        after,
                    });
                }
            }
            if c.ahv.is_some_and(|h| m >= h) {
                let rest = self.p3a[a];
                if rest > 0.005 {
                    self.pay_out_3a(a, m, rest, c.rest_repay, msg!(payout_rest));
                }
                if self.record {
                    self.p3a_view[a].rest = Some(Pillar3aRest {
                        month: m,
                        amount: rest.max(0.0),
                    });
                }
                self.p3a[a] = 0.0;
                self.p3a_done[a] = true;
            }
        }
        paid_in
    }

    /// Payout from pillar 3a with lump-sum tax; if requested, first to the tranches with indirect
    /// amortization into this account, the rest into wealth.
    fn pay_out_3a(&mut self, a: usize, m: YearMonth, gross: f64, repay: bool, what: Message) {
        let s = self.s;
        if gross <= 0.0 {
            return;
        }
        self.p3a[a] -= gross;
        let t = self.capital_tax(a, m, gross);
        self.ledger.taxes += t;
        self.event_tax += t;
        self.month_capital_tax += t;
        let mut net = gross - t;
        let mut repaid = 0.0;
        if repay {
            for i in 0..self.props.len() {
                if !self.props[i].active {
                    continue;
                }
                let tr = self.props[i].sched.as_deref().unwrap_or(&s.props[i].tranches);
                let mut order: Vec<usize> = (0..tr.len()).filter(|&j| tr[j].account == a).collect();
                order.sort_by_key(|&j| tr[j].periods.iter().all(|x| x.direct));
                for j in order {
                    let r = net.min(self.props[i].balances[j]);
                    self.props[i].balances[j] -= r;
                    net -= r;
                    repaid += r;
                }
            }
        }
        self.inflow(net, msg!(why_pillar3a));
        let who = s.accounts[a].who.clone();
        self.event(
            m,
            EventKind::Pillar3aPayout,
            msg!(
                event_pillar3a_payout,
                who = who,
                what = what,
                amount = gross,
                tax = t,
                repaid = repaid,
                currency = self.s.currency
            ),
        );
    }

    /// Stock share for the rebalancing in month `m` (index `k`), between 0 and 1: the course over time or (#218) the
    /// characteristic curve with the state of the run; `first` is the withdrawal per month before the first recorded
    /// month (start of the simulation).
    fn stock_share(&mut self, m: YearMonth, k: usize, first: f64) -> f64 {
        let s = self.s;
        let (Some(c), Some(mut t)) = (&s.share_curve, self.share.take()) else {
            return s.stock_share.at(m).clamp(0.0, 1.0);
        };
        if t.due(m.year(), c.monthly) {
            let x = self.state_input(c.input, m, k, &mut t, first);
            t.value = c.at(x).clamp(0.0, 1.0);
        }
        let share = t.value;
        self.share = Some(t);
        share
    }

    /// #218: value of a state variable in month `m` (index `k`).
    fn state_input(&self, input: StateInput, m: YearMonth, k: usize, t: &mut crate::state::Tracker, first: f64) -> f64 {
        use crate::state::ratio;
        let liquid = self.cash + self.stocks + self.bonds;
        match input {
            StateInput::Age => f64::from(self.s.people[0].birth.until(m)) / 12.0,
            StateInput::LiquidReal => liquid / self.infl_idx,
            StateInput::WithdrawalRate => ratio(t.withdrawal_year(first), liquid),
            StateInput::FundedRatio => ratio(liquid / self.infl_idx, self.remaining_gap(m, k)),
            StateInput::StockReturn12m => self.stock_return(k).0,
            StateInput::Drawdown => {
                let i = (self.hist_off + k).min(self.stock_hist.len() - 1);
                t.drawdown(&self.stock_hist, i)
            }
        }
    }

    /// #218: remaining gap from month `m` (index `k`) until the end in prices of the start month: spending minus
    /// earned income, minus the pensions as known today (AHV and PK, from their start), without discounting.
    fn remaining_gap(&self, m: YearMonth, k: usize) -> f64 {
        let s = self.s;
        let end = s.start.plus(s.months as i32);
        let mut cuts = vec![m];
        for c in &s.people {
            for at in [c.pk_at, c.ahv.at, Some(c.ahv.reference)].into_iter().flatten() {
                if m < at && at < end {
                    cuts.push(at);
                }
            }
        }
        cuts.sort();
        cuts.dedup();
        let mut pensions = 0.0;
        for (j, &a) in cuts.iter().enumerate() {
            let b = cuts.get(j + 1).copied().unwrap_or(end);
            let (ahv, pk) = self.pensions_at(a);
            pensions += (ahv + pk) * f64::from(a.until(b));
        }
        s.need_rest[k] - pensions / self.infl_idx
    }

    fn rebalance(&mut self, reserve: f64, share: f64) {
        let total = self.cash + self.stocks + self.bonds;
        if total - reserve < 0.0 {
            self.cash = total;
            self.stocks = 0.0;
            self.bonds = 0.0;
        } else {
            let invest = total - reserve;
            self.cash = reserve;
            self.stocks = invest * share;
            self.bonds = invest * (1.0 - share);
        }
    }

    /// AHV and PK pension in month `mm` for planning the current year: running pensions as today, those not yet
    /// started from their date (PK pension estimated from today's balance).
    fn pensions_at(&self, mm: YearMonth) -> (f64, f64) {
        let mut pk = 0.0;
        for (st, c) in self.people.iter().zip(&self.s.people) {
            pk += if st.retired {
                st.pk_pension
            } else if c.pk_at.is_some_and(|a| mm >= a) {
                (st.pk + st.pk_accrued) * (1.0 - c.pk_lumpsum) * c.pk_conversion / 12.0
            } else {
                0.0
            };
        }
        (self.ahv.projected(mm), pk)
    }

    /// Net need from the planned values of the current year from month `m` until December (at most until the end of the plan),
    /// extrapolated to 12 months; from 2026 the AHV with the 13th pension (#198). `housing_now` are this month's ongoing housing costs without amortization.
    fn plan_basis(&self, m: YearMonth, housing_now: f64, source: Message) -> NeedBasis {
        let s = self.s;
        let last = s.start.plus(s.months as i32 - 1);
        let (mut living, mut housing, mut ahv, mut pk, mut earned, mut p3a) = (0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        let mut per_person = [0.0; 2];
        // #131: the same deductions as the assessment
        let mut ded = YearDeductions::default();
        let mut n = 0u32;
        // #6: If a spending rule (guardrails) steers spending, the living costs come from it.
        let rule = s.dynamic.filter(|_| self.budget_on);
        for i in 0..(13 - m.month() as i32) {
            let mm = m.plus(i);
            if mm > last {
                break;
            }
            let (a, p) = self.pensions_at(mm);
            living += match &rule {
                Some(d) => self.rule_living(
                    d,
                    mm,
                    a + p + s.earned(mm, self.infl_idx),
                    self.month_running + s.pillar3a_contribution(mm),
                ),
                None => s.planned_living(mm) * self.infl_idx,
            };
            housing += housing_now;
            p3a += s.pillar3a_contribution(mm);
            ahv += a;
            pk += p;
            earned += s.earned(mm, self.infl_idx);
            for (i, e) in per_person.iter_mut().enumerate().take(s.people.len()) {
                *e += s.person_earned(i, mm, self.infl_idx);
            }
            // the share of the income includes the 13th AHV pension as in the assessment (#198)
            let a13 = if mm.year() >= AHV13_FROM_YEAR {
                a * 13.0 / 12.0
            } else {
                a
            };
            s.add_deduction_month(&mut ded, mm, a13 + p + s.earned(mm, self.infl_idx));
            for (i, y) in ded.pillar3a.iter_mut().enumerate().take(s.people.len()) {
                *y += s.pillar3a_contribution_of(i, mm);
            }
            n += 1;
        }
        let f = if n > 0 { 12.0 / f64::from(n) } else { 0.0 };
        // #198: From 2026 the AHV of a year includes the 13th pension: 13 × the monthly pension, extrapolated 12 months
        // × 13/12 (household AHV already capped, no second cap); also for the income tax.
        if m.year() >= AHV13_FROM_YEAR {
            ahv *= 13.0 / 12.0;
        }
        let per_person = per_person.map(|e| e * f);
        ded.own_rate *= f;
        ded.pillar3a = ded.pillar3a.map(|y| y * f);
        NeedBasis {
            year: m.year(),
            source,
            living_source: if rule.is_some() { "Guardrails" } else { "geplant" }.into(),
            living: living * f,
            housing: housing * f,
            income_tax: s
                .income_tax(
                    m,
                    (ahv + pk + earned) * f,
                    &per_person[..s.people.len().min(2)],
                    &ded,
                    self.tax_index(),
                )
                .0,
            ahv: ahv * f,
            pk: pk * f,
            earned: earned * f,
            pillar3a: p3a * f,
            months: n,
        }
    }

    /// Stock return for the return condition (#103): the last 12 months including the current one, as the ratio
    /// of the index levels; the path's lead-in supplies the months before the start. If it is missing entirely or partly (series
    /// starts later), the return since the earliest known level applies («since start»).
    fn stock_return(&self, k: usize) -> (f64, &'static str) {
        let h = &self.stock_hist;
        // Before the month's return step (preview) the current month is missing: then the last available one applies
        let i = (self.hist_off + k).min(h.len() - 1);
        if i >= 12 {
            (h[i] / h[i - 12] - 1.0, "last 12 months")
        } else {
            (h[i] / h[0] - 1.0, "since start")
        }
    }

    /// Transfers `amount` from bucket `from` to bucket `to` and records it (1 = cash, 2 = bonds, 3 = stocks).
    fn bucket_move(&mut self, from: u8, to: u8, amount: f64, moves: &mut Vec<BucketMove>) {
        if amount <= 1e-9 {
            return;
        }
        match from {
            1 => self.cash -= amount,
            2 => self.bonds -= amount,
            _ => self.stocks -= amount,
        }
        match to {
            1 => self.cash += amount,
            2 => self.bonds += amount,
            _ => self.stocks += amount,
        }
        moves.push(BucketMove { from, to, amount });
    }

    /// Inflow into cash. With an active 3-bucket strategy it belongs in bucket 3 (special case B).
    fn inflow(&mut self, amount: f64, why: Message) {
        self.cash += amount;
        self.mark_inflow(amount, why);
    }

    /// Notes an inflow already booked in `cash` for bucket 3; the transfer follows after the returns.
    fn mark_inflow(&mut self, amount: f64, why: Message) {
        self.mark_inflow_from(0, amount, why);
    }

    /// Like `mark_inflow`; `from` is 0 (external) or 4 (reservation).
    fn mark_inflow_from(&mut self, from: u8, amount: f64, why: Message) {
        if amount > 0.0 && self.buckets_active() {
            self.inflow3 += amount;
            self.inflow_log.push((from, 3, amount, why));
        }
    }

    /// Sale proceeds reserved for a purchase in the following month (belong to no bucket).
    fn mark_reservation(&mut self, amount: f64, why: Message) {
        if amount > 0.0 && self.buckets_active() {
            self.inflow_log.push((0, 4, amount, why));
        }
    }

    /// The 3-bucket strategy is running and buckets 2 and 3 are not shut down.
    fn buckets_active(&self) -> bool {
        self.s.buckets.is_some() && self.buckets_on && !self.phase8
    }

    /// Equity of a purchase: first from the reservation, the rest is charged to bucket 1.
    fn pay_equity(&mut self, equity: f64) {
        self.cash -= equity;
        let from_reservation = self.reserved.min(equity.max(0.0));
        self.reserved -= from_reservation;
        // The rest is a one-off expense of this month: noted, not paid from bucket 1 at once
        self.month_out += equity - from_reservation;
    }

    /// Log entry of the 3-bucket strategy.
    #[allow(clippy::too_many_arguments)]
    fn log_buckets(
        &mut self,
        b: &Buckets,
        m: YearMonth,
        k: usize,
        trigger: &str,
        reason: Message,
        passed: u32,
        ignored: bool,
        targets: (f64, f64),
        moves: Vec<BucketMove>,
        payment: f64,
        before: [f64; 3],
        after: [f64; 3],
        text: Message,
    ) {
        let (ret, ret_basis) = self.stock_return(k);
        let basis = self.basis.clone().unwrap_or_default();
        let need = basis.need();
        self.event(m, EventKind::Buckets, text);
        self.bucket_refills.push(BucketRefill {
            month: m,
            reason,
            trigger: trigger.to_string(),
            passed_months: passed,
            basis,
            need,
            target1: targets.0,
            target2: targets.1,
            stock_return: ret,
            return_basis: return_basis(ret_basis),
            threshold: b.refill_threshold,
            condition_met: ret >= b.refill_threshold,
            condition_ignored: ignored,
            moves,
            payment,
            before,
            after,
        });
    }

    /// Special case B: write inflows to the log and book the amounts for bucket 3 from `cash` to bucket 3 (no
    /// refill of bucket 1 or 2). The log shows bucket 1 without the inflow; the inflow appears as the booking
    /// «extern → Topf 3», «extern → Reservierung» or «Reservierung → Topf 3».
    /// `shift` converts the balance of bucket 1 in the log to «before expenses» (0 if `cash` is already kept that way).
    fn flush_inflow(&mut self, b: &Buckets, m: YearMonth, k: usize, pay: f64, shift: f64) {
        let amount = std::mem::take(&mut self.inflow3);
        let log = std::mem::take(&mut self.inflow_log);
        if log.is_empty() {
            return;
        }
        let t1 = self.cash + shift - amount;
        let t = self.target_pair(b, 0, 0.0);
        // Source of an inflow: 0 = external, 4 = reservation

        // Reservations (external → reservation): no bucket changes
        for (from, to, amt, why) in log.iter().filter(|x| x.1 == 4) {
            let st = [t1, self.bonds, self.stocks];
            let text = msg!(
                event_buckets_reservation_inflow,
                why = why,
                amount = (*amt),
                currency = self.s.currency
            );
            self.log_buckets(
                b,
                m,
                k,
                "B",
                msg!(reason_reservation_inflow, why = why),
                0,
                true,
                t,
                vec![BucketMove {
                    from: *from,
                    to: *to,
                    amount: *amt,
                }],
                pay,
                st,
                st,
                text,
            );
        }
        let to3: Vec<_> = log.iter().filter(|x| x.1 == 3).collect();
        if to3.is_empty() {
            return;
        }
        let why: Vec<Message> = to3.iter().map(|x| x.3.clone()).collect();
        let moves: Vec<BucketMove> = to3
            .iter()
            .map(|x| BucketMove {
                from: x.0,
                to: 3,
                amount: x.2,
            })
            .collect();
        let before = [t1, self.bonds, self.stocks];
        self.cash -= amount;
        self.stocks += amount;
        let after = [t1, self.bonds, self.stocks];
        let (amount_v, a1, a2, a3) = (amount, after[0], after[1], after[2]);
        let text = if to3.iter().all(|x| x.0 == 4) {
            msg!(
                event_buckets_inflow_reservation,
                why = why,
                amount = amount_v,
                pot1 = a1,
                pot2 = a2,
                pot3 = a3,
                currency = self.s.currency
            )
        } else {
            msg!(
                event_buckets_inflow_external,
                why = why,
                amount = amount_v,
                pot1 = a1,
                pot2 = a2,
                pot3 = a3,
                currency = self.s.currency
            )
        };
        self.log_buckets(
            b,
            m,
            k,
            "B",
            msg!(reason_inflow_pot3, why = why),
            0,
            true,
            t,
            moves,
            pay,
            before,
            after,
            text,
        );
    }

    /// Target sizes of buckets 1 and 2, reduced by `passed` months of the year already past.
    /// The target of bucket 1 also includes this month's one-off expenses `one_off`.
    fn target_pair(&self, b: &Buckets, passed: u32, one_off: f64) -> (f64, f64) {
        let need = self.basis.as_ref().map_or(0.0, NeedBasis::need);
        let t = |years: f64| (years * 12.0 - f64::from(passed)).max(0.0) / 12.0 * need;
        (t(b.cash_years) + one_off, t(b.bond_years))
    }

    /// Common refill of the buckets for the January refill, special case A and C1. Only in the January refill
    /// (`surplus`) does a surplus in bucket 1 or 2 go to bucket 3. Bucket 1 is refilled from bucket 2 first. Sales from bucket 3 (rest for bucket 1, then bucket 2) only
    /// if the return criterion is met or ignored (`ignore`).
    fn fill_pots(
        &mut self,
        b: &Buckets,
        m: YearMonth,
        k: usize,
        (trigger, reason, passed, ignore, pay): (&str, Message, u32, bool, f64),
        (one_off, surplus): (f64, bool),
        (before, mut moves): ([f64; 3], Vec<BucketMove>),
    ) {
        let (t1, t2) = self.target_pair(b, passed, one_off);
        let (ret, ret_basis) = self.stock_return(k);
        let met = ret >= b.refill_threshold;
        let sell_ok = met || ignore;
        // Surplus booking into bucket 3 only at the January refill
        if surplus && self.cash > t1 {
            self.bucket_move(1, 3, self.cash - t1, &mut moves);
        }
        let d1 = (t1 - self.cash).max(0.0);
        self.bucket_move(2, 1, d1.min(self.bonds.max(0.0)), &mut moves);
        if sell_ok {
            let d1 = (t1 - self.cash).max(0.0);
            self.bucket_move(3, 1, d1.min(self.stocks.max(0.0)), &mut moves);
            let d2 = (t2 - self.bonds).max(0.0);
            self.bucket_move(3, 2, d2.min(self.stocks.max(0.0)), &mut moves);
        }
        if surplus && self.bonds > t2 {
            self.bucket_move(2, 3, self.bonds - t2, &mut moves);
        }
        let basis = self.basis.clone().unwrap_or_default();
        let after = [self.cash, self.bonds, self.stocks];
        let crit = if ignore {
            msg!(criterion_ignored)
        } else {
            let basis = return_basis(ret_basis);
            if met {
                msg!(
                    criterion_met,
                    stock_return = ret,
                    basis = basis,
                    threshold = b.refill_threshold
                )
            } else {
                msg!(
                    criterion_not_met,
                    stock_return = ret,
                    basis = basis,
                    threshold = b.refill_threshold
                )
            }
        };
        let text = msg!(
            event_buckets_refill,
            reason = reason,
            need = (basis.need()),
            expenses = (basis.expenses()),
            receipts = (basis.receipts()),
            source = basis.source,
            target1 = t1,
            target2 = t2,
            criterion = crit,
            before1 = before[0],
            before2 = before[1],
            before3 = before[2],
            after1 = after[0],
            after2 = after[1],
            after3 = after[2],
            currency = self.s.currency
        );
        self.log_buckets(
            b,
            m,
            k,
            trigger,
            reason,
            passed,
            ignore,
            (t1, t2),
            moves,
            pay,
            before,
            after,
            text,
        );
    }

    /// Special case C: if bucket 1 does not cover this month's expenses (`pay` are already deducted), the procedure
    /// depends on the return criterion.
    /// **C1** (met): like the January refill (bucket 1 from bucket 2, rest from bucket 3, then bucket 2 from bucket 3), the targets
    /// reduced by the months passed since January, the target of bucket 1 plus the one-off expenses `one_off`; no
    /// surplus booking. If bucket 1 still does not cover the expenses afterwards, C2 follows.
    /// **C2** (not met): bucket 1 is refilled from bucket 2, then bucket 3, so that it covers the expenses and afterwards
    /// still holds (min(reserve, months until January) − 1) monthly needs; bucket 2 is not refilled.
    /// If that is not enough, a property is sold in an emergency sale (proceeds into bucket 3, then C again); without property
    /// phase 8 starts (bucket 1 may be negative, buckets 2 and 3 are shut down).
    fn cover_payments(&mut self, b: &Buckets, m: YearMonth, k: usize, pay: f64, one_off: f64) {
        for _ in 0..=self.props.len() {
            if self.cash - self.reserved >= -0.005 {
                return;
            }
            let (ret, _) = self.stock_return(k);
            if ret >= b.refill_threshold {
                // C1: procedure of the January refill on the balance before the expenses
                let passed = m.month() - 1;
                self.cash -= self.reserved;
                self.cash += pay;
                let before = [self.cash, self.bonds, self.stocks];
                let why = msg!(reason_lookahead_met, passed = passed);
                self.fill_pots(
                    b,
                    m,
                    k,
                    ("C1", why, passed, false, pay),
                    (one_off, false),
                    (before, vec![]),
                );
                self.cash -= pay;
                self.cash += self.reserved;
                if self.cash - self.reserved >= -0.005 {
                    return;
                }
            }
            // C2 (also as fallback when C1 is not enough)
            let free = self.cash - self.reserved;
            let month_need = self.basis.as_ref().map_or(0.0, NeedBasis::need) / 12.0;
            let months_to_jan = f64::from(13 - m.month());
            let keep = (b.reserve_months.min(months_to_jan) - 1.0).max(0.0) * month_need;
            let want = keep - free;
            let before = [free + pay, self.bonds, self.stocks];
            let mut moves: Vec<BucketMove> = vec![];
            self.bucket_move(2, 1, want.min(self.bonds.max(0.0)), &mut moves);
            let rest = (keep - (self.cash - self.reserved)).max(0.0);
            self.bucket_move(3, 1, rest.min(self.stocks.max(0.0)), &mut moves);
            let free_after = self.cash - self.reserved;
            if !moves.is_empty() {
                let text = msg!(
                    event_buckets_c2,
                    payment = pay,
                    before1 = before[0],
                    before2 = before[1],
                    before3 = before[2],
                    after1 = (free_after + pay),
                    after2 = self.bonds,
                    after3 = self.stocks,
                    currency = self.s.currency
                );
                let t = self.target_pair(b, 0, 0.0);
                let after = [free_after + pay, self.bonds, self.stocks];
                let why = msg!(reason_lookahead_not_met, months = months_to_jan);
                self.log_buckets(b, m, k, "C2", why, 0, false, t, moves, pay, before, after, text);
            }
            if free_after >= -0.005 {
                return;
            }
            // Buckets 2 and 3 are not enough: emergency sale of a property, otherwise phase 8
            if let Some(i) = self.props.iter().position(|p| p.active).filter(|_| !self.s.never_sell) {
                self.props[i].emergency_reason = Some(msg!(reason_pots_insufficient));
                self.sell(i, m, EventKind::EmergencySale);
                if self.reserved > 0.0 {
                    self.reserved = 0.0;
                    self.reserved_sale = None;
                }
                self.flush_inflow(b, m, k, pay, pay);
            } else {
                self.phase8 = true;
                let text = msg!(event_buckets_empty, cash = self.cash, currency = self.s.currency);
                self.event(m, EventKind::Buckets, text);
                return;
            }
        }
    }

    /// 3-bucket strategy for this month, after returns and taxes. `pay` are the month's expenses (ongoing and
    /// one-off), already deducted from `cash`. The decisions apply to the balance before these expenses
    /// (order: income, purchases, inflows B, January refill or A, look-ahead C, expenses); the transfers
    /// themselves follow only after the returns.
    fn bucket_step(&mut self, b: &Buckets, m: YearMonth, k: usize, pay: f64, one_off: f64, housing_now: f64) {
        // Phase 8: only bucket 1 counts; as soon as wealth is positive again, A starts anew
        let mut collect = false;
        if self.phase8 {
            if self.cash - self.reserved < 0.0 {
                return;
            }
            self.phase8 = false;
            self.a_reasons.push(msg!(reason_wealth_positive_again));
            collect = true;
        }
        if !self.buckets_on {
            collect = true;
            self.buckets_on = true;
            self.a_reasons.push(msg!(reason_withdrawal_start));
        }
        // Balance before the month's expenses; the reservation belongs to no bucket
        self.cash -= self.reserved;
        self.cash += pay;
        self.flush_inflow(b, m, k, pay, 0.0);
        let a = !self.a_reasons.is_empty();
        if a || m.month() == 1 {
            let (passed, trigger) = if a { (m.month() - 1, "A") } else { (0, "Januar") };
            let source = match (a, passed > 0) {
                (true, true) => msg!(basis_from_extrapolated, month = m),
                (true, false) => msg!(basis_from, month = m),
                _ => msg!(basis_january),
            };
            self.basis = Some(self.plan_basis(m, housing_now, source));
            let before = [self.cash, self.bonds, self.stocks];
            let mut moves: Vec<BucketMove> = vec![];
            if collect {
                // All liquid wealth into bucket 3 first
                self.bucket_move(1, 3, self.cash.max(0.0), &mut moves);
                self.bucket_move(2, 3, self.bonds.max(0.0), &mut moves);
            }
            let reason = if a {
                msg!(reasons, items = self.a_reasons)
            } else {
                msg!(reason_january)
            };
            self.fill_pots(
                b,
                m,
                k,
                (trigger, reason, passed, a, pay),
                (one_off, m.month() == 1),
                (before, moves),
            );
        }
        self.cash -= pay;
        self.cash += self.reserved;
        self.cover_payments(b, m, k, pay, one_off);
    }

    /// Living costs of one month by the spending rule: withdrawal / 12 + savings + income and pensions − housing costs,
    /// limited by floor and ceiling (shares of the planned spending).
    fn rule_living(&self, d: &Dynamic, m: YearMonth, income: f64, housing: f64) -> f64 {
        self.rule_living_bound(d, m, income, housing).0
    }

    /// As [`Self::rule_living`]; also tells whether the floor sets the living costs (#205).
    fn rule_living_bound(&self, d: &Dynamic, m: YearMonth, income: f64, housing: f64) -> (f64, bool) {
        let s = self.s;
        let planned = s.planned_living(m) * self.infl_idx;
        let mut l = (self.budget / 12.0 + s.savings.at(m) + income - housing).max(0.0);
        if let Some(c) = d.ceiling {
            l = l.min(c * planned);
        }
        let mut at_floor = false;
        if let Some(f) = d.floor {
            at_floor = f * planned > l;
            l = l.max(f * planned);
        }
        (l, at_floor)
    }

    /// Set the annual withdrawal at retirement and adjust it every January.
    fn update_budget(&mut self, d: &Dynamic, m: YearMonth, infl_factor: f64) {
        // #6: Sale proceeds reserved for a purchase do not count towards the wealth of the spending rule.
        let liquid = (self.cash + self.stocks + self.bonds - self.reserved).max(0.0);
        if !self.budget_on {
            self.budget_on = true;
            self.strategy_started = true;
            self.budget_rate0 = match d.rule {
                Rule::Fixed { rate } => rate,
                Rule::Guardrails { initial, .. } => initial,
                Rule::Vpw {
                    horizon_age,
                    real_return,
                } => vpw_rate(real_return, horizon_age - self.s.age_at(m)),
                Rule::Endowment { rate, .. } => rate,
            };
            self.budget = self.budget_rate0 * liquid;
            self.event(
                m,
                EventKind::Withdrawal,
                msg!(
                    event_withdrawal_start,
                    amount = self.budget,
                    rate = self.budget_rate0,
                    wealth = liquid,
                    currency = self.s.currency
                ),
            );
            return;
        }
        if m.month() != 1 && !self.apply_now {
            return;
        }
        match d.rule {
            Rule::Fixed { rate } => self.budget = rate * liquid,
            Rule::Vpw {
                horizon_age,
                real_return,
            } => {
                self.budget = vpw_rate(real_return, horizon_age - self.s.age_at(m)) * liquid;
            }
            Rule::Endowment { rate, smoothing } => {
                self.budget = (1.0 - smoothing) * self.budget * infl_factor + smoothing * rate * liquid;
            }
            Rule::Guardrails {
                upper, lower, adjust, ..
            } => {
                // #239: Without liquid wealth there is no withdrawal rate: the budget stays and no guardrail event
                // (the used-up wealth is reported separately).
                if liquid <= 0.0 {
                    return;
                }
                let current = |b: f64| b / liquid;
                // After a losing year no inflation adjustment while the rate is above the initial rate.
                if !(self.stock_prev_year < 0.0 && current(self.budget) > self.budget_rate0) {
                    self.budget *= infl_factor;
                }
                let rate = current(self.budget);
                if rate > self.budget_rate0 * (1.0 + upper) {
                    self.budget *= 1.0 - adjust;
                    self.event(
                        m,
                        EventKind::Withdrawal,
                        msg!(
                            event_guardrail_cut,
                            adjust = adjust,
                            amount = self.budget,
                            rate = rate,
                            currency = self.s.currency
                        ),
                    );
                } else if rate < self.budget_rate0 * (1.0 - lower) {
                    self.budget *= 1.0 + adjust;
                    self.event(
                        m,
                        EventKind::Withdrawal,
                        msg!(
                            event_guardrail_raise,
                            adjust = adjust,
                            amount = self.budget,
                            rate = rate,
                            currency = self.s.currency
                        ),
                    );
                }
            }
        }
    }

    fn execute(mut self, mut observer: Option<&mut dyn MonthObserver>) -> SimResult {
        let s = self.s;
        // Initial split with the values of the start month.
        {
            let m = s.start;
            let mut running = 0.0;
            for (i, p) in s.props.iter().enumerate() {
                if !self.props[i].active {
                    continue;
                }
                running += p.fix_costs;
                for (j, t) in p.tranches.iter().enumerate() {
                    if let Some(x) = t.periods.iter().find(|x| x.start <= m && m < x.end) {
                        running += self.props[i].balances[j] * x.rate(self.path.saron[0]) / 12.0 + x.amort_q / 3.0;
                    }
                }
            }
            // #54: Before retirement no withdrawal strategy applies; the holdings at the simulation start are
            // taken over unchanged. If someone is already retired, the strategy splits as before.
            if s.people.iter().any(|p| p.pk_at.is_some_and(|a| a <= m)) {
                let gap = (s.planned_living(m) + running - s.earned(m, self.infl_idx)).max(0.0);
                let share = self.stock_share(m, 0, gap);
                self.rebalance(gap * s.reserve.at(m), share);
            }
        }
        {
            let liquid = self.cash + self.stocks + self.bonds;
            let p3a: f64 = self.p3a.iter().sum();
            let pk: f64 = self.people.iter().map(|p| p.pk).sum();
            self.summary.start_liquid = liquid;
            self.summary.start_total = liquid + self.equity() + pk + p3a;
        }

        // #190/AK3: The months of the start year before the simulation start count for the 13th AHV pension with the
        // pensions due then (amounts at the start).
        if s.start.year() >= AHV13_FROM_YEAR {
            for month in 1..s.start.month() {
                let (a, _) = self.pensions_at(YearMonth::new(s.start.year(), month));
                self.ahv13_centimes += ahv13_share_centimes(a);
            }
        }
        for k in 0..s.months {
            let m = s.start.plus(k as i32);
            let december = m.is_december();
            // New month: collect occasions for applying the withdrawal strategy at once; sale proceeds reserved in the
            // previous month are applied to the surplus after the purchase (below, then the strategy).
            self.apply_reasons.clear();
            self.a_reasons.clear();
            self.month_out = 0.0;
            self.ledger = Ledger::default();
            self.inflow_log.clear();
            self.strategy_started = false;
            self.reserved_cash = 0.0;
            let mut leftover = false;
            if self.reserved_sale == Some(m.plus(-1)) {
                self.apply_reasons.push(msg!(reason_sale_surplus));
                self.reserved_sale = None;
                leftover = true;
            }
            let mut month_tax = 0.0;
            self.month_capital_tax = 0.0;
            self.month_gain_tax = 0.0;
            let mut infl_factor = 1.0;
            // Inflation monthly with the previous month's inflation. Amounts are in prices of the
            // start month. AHV and the withdrawal budget follow in January with the accrued inflation.
            if k > 0 {
                let before = self.active_worth();
                self.infl_idx *= (1.0 + self.path.inflation[k - 1]).max(0.0).powf(1.0 / 12.0);
                self.ledger.revaluation += self.active_worth() - before;
            }
            if m.month() == 1 && k > 0 {
                let factor = self.infl_idx / self.infl_idx_jan;
                self.infl_idx_jan = self.infl_idx;
                infl_factor = factor;
                // #145: The cap rises once per January as soon as at least one indexed pension is running (not per person).
                self.ahv.index(factor);
            }

            // Events
            let mut extras_out = 0.0;
            for (em, amount, label) in &s.extras {
                if *em == m {
                    self.cash += amount;
                    self.ledger.extras += amount;
                    if *amount < 0.0 {
                        extras_out -= amount;
                    }
                    if *amount > 0.0 {
                        self.mark_inflow(*amount, msg!(why_extra, label = label));
                        self.apply_reasons.push(msg!(reason_extra_inflow, label = label));
                    }
                    let text = msg!(
                        event_extra,
                        label = label,
                        amount = (*amount),
                        currency = self.s.currency
                    );
                    self.event(m, EventKind::Extra, text);
                }
            }
            for pi in 0..self.people.len() {
                let c = &s.people[pi];
                if !self.people[pi].retired && c.pk_at.is_some_and(|a| m >= a) {
                    let (pk, accrued) = {
                        let st = &mut self.people[pi];
                        st.retired = true;
                        let accrued = std::mem::take(&mut st.pk_accrued);
                        st.pk += accrued;
                        (st.pk, accrued)
                    };
                    self.ledger.pk_interest += accrued;
                    let lump = pk * c.pk_lumpsum;
                    let t = self.capital_tax(pi, m, lump);
                    self.ledger.taxes += t;
                    self.ledger.annuity += pk - lump;
                    month_tax += t;
                    self.month_capital_tax += t;
                    self.cash += lump - t;
                    if lump - t > 0.0 {
                        self.apply_reasons.push(msg!(reason_pk_payout));
                        self.a_reasons.push(msg!(reason_pk_lump_sum));
                        self.mark_inflow(lump - t, msg!(reason_pk_lump_sum));
                    }
                    let pension = pk * (1.0 - c.pk_lumpsum) * c.pk_conversion / 12.0;
                    self.people[pi].pk_pension = pension;
                    self.people[pi].pk = 0.0;
                    self.summary.lumpsum_net += lump - t;
                    self.summary.lumpsum_tax += t;
                    self.summary.pension_fund_monthly += pension;
                    let who = if s.people.len() > 1 {
                        crate::message::arg(&msg!(suffix_person, n = pi + 1))
                    } else {
                        crate::message::Arg::Null
                    };
                    self.event(
                        m,
                        EventKind::Retirement,
                        msg!(
                            event_pk_payout,
                            who = who,
                            lump = lump,
                            tax = t,
                            pension = pension,
                            currency = self.s.currency
                        ),
                    );
                }
            }
            // #191: AHV of the month (cap, reduction, supplement) and the start of a payout
            for pi in self.ahv.step(m, k == 0) {
                let who = if s.people.len() > 1 {
                    crate::message::arg(&msg!(suffix_person, n = pi + 1))
                } else {
                    crate::message::Arg::Null
                };
                let pension = self.ahv.paid(pi);
                self.event(
                    m,
                    EventKind::AhvStart,
                    msg!(
                        event_ahv_start,
                        who = who,
                        pension = pension,
                        currency = self.s.currency
                    ),
                );
            }
            let (prop_cost, prop_running) = self.properties(k, m);
            // #52: The pillar 3a contribution counts as spending (like housing costs).
            let p3a_in = self.pillar3a(m);
            self.month_p3a_in = p3a_in;
            if leftover {
                // Surplus of the reservation after the purchase: into bucket 3
                let rest = std::mem::take(&mut self.reserved);
                self.mark_inflow_from(4, rest, msg!(why_reservation_surplus));
            }
            // Order within the month: inflows, purchases/sales (above), then the strategy; once per month, also in January.
            self.apply_now = !self.apply_reasons.is_empty();

            // Cash flows
            let earned = s.earned(m, self.infl_idx);
            let pk_pension = self.pk_pension();
            let ahv_monthly = self.ahv();
            // #190: the 13th AHV pension in December counts as AHV income (also for the income tax).
            let ahv = ahv_monthly + self.thirteenth_ahv(m, ahv_monthly);
            let income = earned + pk_pension + ahv;
            let planned = s.planned_living(m) * self.infl_idx;
            // #6/#47: The spending rule applies from the retirement of the first person.
            let (living, at_floor) = match s.dynamic {
                Some(d) if self.any_retired() => {
                    self.update_budget(&d, m, infl_factor);
                    self.rule_living_bound(&d, m, income, prop_cost + p3a_in)
                }
                _ => (planned, false),
            };
            self.month_running = prop_running;
            self.cash += s.savings.at(m) + income - living - prop_cost - p3a_in;
            self.ledger.income += income;
            self.ledger.savings += s.savings.at(m);
            self.ledger.living += living;
            self.ledger.housing += prop_cost;
            self.year_income += income;
            s.add_deduction_month(&mut self.year_ded, m, income);
            // #63 proposal of the canton, #152 large pillar 3a: earned income per person of the tax year.
            for (i, e) in self.year_earned.iter_mut().enumerate().take(s.people.len()) {
                *e += s.person_earned(i, m, self.infl_idx);
            }

            // Returns
            let wealth = self.cash + self.stocks + self.bonds;
            if wealth > 0.0 {
                self.ledger.market += self.stocks * self.path.stocks[k] + self.bonds * self.path.bonds[k];
                self.stocks *= 1.0 + self.path.stocks[k];
                self.bonds *= 1.0 + self.path.bonds[k];
            }
            self.stock_idx *= 1.0 + self.path.stocks[k];
            self.stock_hist.push(self.stock_idx);
            self.bond_idx *= 1.0 + self.path.bonds[k];
            self.stock_year *= 1.0 + self.path.stocks[k];
            if december {
                self.stock_prev_year = self.stock_year - 1.0;
                self.stock_year = 1.0;
            }

            // Pension fund: interest monthly on the balance at the start of the month, credited in December
            for (st, c) in self.people.iter_mut().zip(&s.people) {
                if !st.retired {
                    st.pk_accrued += st.pk * c.pk_interest.at(m) / 12.0;
                    st.pk += c.pk_contribution.at(m);
                    self.ledger.pk_contributions += c.pk_contribution.at(m);
                    if december {
                        self.ledger.pk_interest += st.pk_accrued;
                        st.pk += st.pk_accrued;
                        st.pk_accrued = 0.0;
                    }
                }
            }

            // Taxes
            let mut paid = 0.0;
            let (mut paid_income, mut paid_wealth) = (0.0, 0.0);
            let (mut assessed_income, mut assessed_wealth) = (0.0, 0.0);
            let mut tax_basis = None;
            if m.month() == 5 {
                paid += std::mem::take(&mut self.tax_pending);
                paid_income += std::mem::take(&mut self.pending_income);
                paid_wealth += std::mem::take(&mut self.pending_wealth);
            }
            if december {
                // #229: the wealth tax uses the tax value of the properties instead of the market value
                let taxable_wealth = wealth + self.tax_equity();
                let idx = self.tax_index();
                self.year_ded.one_off = s.one_off_deduction(m.year());
                let (ti, mut basis) = s.income_tax(
                    m,
                    self.year_income,
                    &self.year_earned[..s.people.len().min(2)],
                    &self.year_ded,
                    idx,
                );
                // #231: minus the deductions from wealth (in the mode «Wohnort» in addition to those of the canton)
                let (taxable_wealth, wealth_basis) = s.taxable_wealth(m, taxable_wealth, idx);
                let tw = s.wealth_tax(m, taxable_wealth, idx);
                if s.tax_basis && self.record {
                    if s.tax_values {
                        basis.property_tax_value = Some(self.tax_worth());
                    }
                    basis.wealth = wealth_basis;
                    tax_basis = Some(basis);
                }
                self.tax_pending += ti + tw;
                self.pending_income += ti;
                self.pending_wealth += tw;
                (assessed_income, assessed_wealth) = (ti, tw);
                self.year_income = 0.0;
                self.year_earned = [0.0; 2];
                self.year_ded = YearDeductions::default();
                self.year_capital.fill(0.0);
            }
            if k + 1 == s.months {
                paid += std::mem::take(&mut self.tax_pending);
                paid_income += std::mem::take(&mut self.pending_income);
                paid_wealth += std::mem::take(&mut self.pending_wealth);
            }
            self.cash -= paid;
            self.ledger.taxes += paid;
            month_tax += paid;
            month_tax += std::mem::take(&mut self.event_tax);
            self.summary.taxes_total += month_tax;

            // Check for an emergency sale (with the pensions after any indexing)
            let liquid = self.cash + self.stocks + self.bonds;
            let shortfall = living + prop_running + p3a_in - (earned + self.pk_pension() + self.ahv());
            if !s.never_sell
                && liquid - s.emergency.at(m) * shortfall < 0.0
                && !self.props.iter().any(|p| p.emergency_pending)
            {
                if let Some(ps) = self.props.iter_mut().find(|p| p.active) {
                    ps.emergency_pending = true;
                    ps.emergency_reason = Some(msg!(reason_emergency_criterion));
                }
            }

            // #218: withdrawal of the month for the characteristic curve (expenses minus income).
            if let Some(t) = &mut self.share {
                t.push_withdrawal(living + prop_cost + paid - income - s.savings.at(m));
            }

            // Withdrawal strategy: «3-Topf» from retirement, otherwise «Standard»
            match &s.buckets {
                Some(b) if self.any_retired() => {
                    let one_off = extras_out + self.month_out;
                    let pay = living + prop_cost + p3a_in + paid + one_off;
                    let housing_now = (prop_running - self.month_amort).max(0.0);
                    self.bucket_step(b, m, k, pay, one_off, housing_now);
                }
                _ => {
                    // #54: Before retirement the app keeps the plan's cash amount (with inflation); the
                    // surplus goes into the portfolio, a shortfall is refilled from the portfolio. From
                    // retirement: the strategy's reserve («Standard» as entered, otherwise 3 months).
                    // Sale proceeds reserved for the purchase in the following month stay liquid (not invested).
                    let target = if self.any_retired() {
                        s.reserve.at(m) * shortfall.max(0.0)
                    } else {
                        s.cash0 * self.infl_idx
                    };
                    let reserve = target + self.reserved_cash;
                    let share = self.stock_share(m, k, 0.0);
                    self.rebalance(reserve, share);
                }
            }
            // Log an additional application outside January (in January the strategy applies anyway).
            if self.apply_now && m.month() != 1 && !self.strategy_started && s.dynamic.is_some() && self.any_retired() {
                let text = msg!(event_strategy_applied, reasons = self.apply_reasons);
                self.event(m, EventKind::Withdrawal, text);
            }

            // Evaluation (an emergency sale in the 3-bucket strategy has changed the cash)
            let liquid = self.cash + self.stocks + self.bonds;
            let equity = self.equity();
            let p3a: f64 = self.p3a.iter().sum();
            let pk: f64 = self.people.iter().map(|p| p.pk).sum();
            let total = liquid + equity + pk + p3a;
            if liquid < self.summary.min_liquid {
                self.summary.min_liquid = liquid;
                self.summary.min_liquid_month = Some(m);
            }
            // Depleted: debt or no more funds for uncovered expenses.
            if (liquid < -0.005 || (liquid <= 0.005 && shortfall > 0.0))
                && self.summary.depleted.is_none()
                && (s.never_sell || !self.props.iter().any(|p| p.active))
            {
                self.summary.depleted = Some(m);
                self.event(m, EventKind::Depleted, msg!(event_depleted));
            }
            if s.retire_at() == Some(m) {
                self.summary.total_at_retirement = Some(total);
            }
            self.summary.end_total = total;
            self.summary.end_total_real = total / self.infl_idx;
            self.summary.end_liquid = liquid;
            if self.record || observer.is_some() {
                let record = MonthRecord {
                    month: m,
                    age: s.age_at(m),
                    cash: self.cash,
                    stocks: self.stocks,
                    bonds: self.bonds,
                    liquid,
                    property_equity: equity,
                    pension_fund: pk,
                    pillar3a: p3a,
                    total,
                    pillar3a_in: p3a_in,
                    income,
                    spending: living + prop_cost + p3a_in,
                    taxes: month_tax,
                    tax_income: paid_income,
                    tax_wealth: paid_wealth,
                    tax_capital: self.month_capital_tax,
                    tax_gain: self.month_gain_tax,
                    assessed_income,
                    assessed_wealth,
                    tax_basis,
                    planned_spending: planned,
                    withdrawal: living + prop_cost + p3a_in - income,
                    earned,
                    pk_pension,
                    ahv,
                    housing: prop_cost,
                    property_value: (0..self.props.len())
                        .filter(|&i| self.props[i].active)
                        .map(|i| self.worth(i))
                        .sum(),
                    mortgage: self
                        .props
                        .iter()
                        .filter(|p| p.active)
                        .map(|p| p.balances.iter().sum::<f64>())
                        .sum(),
                    price_index: self.infl_idx,
                    stock_index: self.stock_idx,
                    bond_index: self.bond_idx,
                    ledger: self.ledger,
                    at_floor,
                };
                if let Some(o) = observer.as_deref_mut() {
                    o.month(&record, &self.summary);
                }
                if self.record {
                    self.records.push(record);
                }
            }
        }
        SimResult {
            label: self.path.label.clone(),
            summary: self.summary,
            events: self.events,
            renewals: self.renewals,
            property_events: self.property_events,
            bucket_refills: self.bucket_refills,
            records: self.records,
            pillar3a: self.p3a_view,
        }
    }
}

/// Index levels of the lead-in (#103): the level before the first lead-in month, then one per lead-in month; the last is 1
/// (level at the start). Room for `months` further months.
fn pre_levels(pre: &[f64], months: usize) -> Vec<f64> {
    let mut levels = Vec::with_capacity(pre.len() + 1 + months);
    levels.push(1.0);
    for r in pre.iter().rev() {
        let last = *levels.last().unwrap_or(&1.0);
        levels.push(last / (1.0 + r).max(1e-9));
    }
    levels.reverse();
    levels
}

#[cfg(test)]
mod tests_withdrawal {
    use super::vpw_rate;

    #[test]
    fn vpw_rate_is_annuity_factor() {
        assert!((vpw_rate(0.0, 25.0) - 0.04).abs() < 1e-12);
        // 30 years at 3 %: known annuity factor 0.0510
        assert!((vpw_rate(0.03, 30.0) - 0.051019).abs() < 1e-5);
        // in the last year everything is withdrawn, never more than the wealth
        assert!(vpw_rate(0.02, 0.0) <= 1.0 && vpw_rate(0.02, 0.5) <= 1.0);
        assert!(vpw_rate(0.02, 20.0) > vpw_rate(0.02, 30.0));
    }
}

#[cfg(test)]
mod tests_schedule {
    use super::schedule;
    use crate::model::Tranche;
    use crate::time::YearMonth;

    fn tranche(periods: &[(f64, f64)], after: &str) -> Tranche {
        let periods: Vec<_> = periods
            .iter()
            .map(|(term, rate)| serde_json::json!({"type": "fixed", "term": term, "rate": rate}))
            .collect();
        serde_json::from_value(serde_json::json!({"name": "T", "amount": 1000, "after": after, "periods": periods}))
            .unwrap()
    }

    // #40: Periods until before `limit`, renewal or repayment after the last period.
    #[test]
    fn periods_until_the_limit() {
        let a = YearMonth::new(2030, 1);
        let s = schedule(
            &tranche(&[(1.0, 0.01), (2.0, 0.02)], "renew"),
            a,
            YearMonth::new(2036, 1),
        );
        let ends: Vec<_> = s.periods.iter().map(|p| (p.start, p.end, p.index, p.rate)).collect();
        assert_eq!(
            ends,
            vec![
                (a, YearMonth::new(2031, 1), 0, 0.01),
                (YearMonth::new(2031, 1), YearMonth::new(2033, 1), 1, 0.02),
                (YearMonth::new(2033, 1), YearMonth::new(2035, 1), 2, 0.02),
                (YearMonth::new(2035, 1), YearMonth::new(2037, 1), 3, 0.02),
            ]
        );
        assert_eq!(s.repay_at, None);
        // A period that ends exactly at the limit: no further one
        let s = schedule(&tranche(&[(1.0, 0.01)], "renew"), a, YearMonth::new(2032, 1));
        assert_eq!(s.periods.len(), 2);
        // Repayment: at the end of the last period if it lies before the limit
        let s = schedule(
            &tranche(&[(1.0, 0.01), (2.0, 0.02)], "repay"),
            a,
            YearMonth::new(2036, 1),
        );
        assert_eq!((s.periods.len(), s.repay_at), (2, Some(YearMonth::new(2033, 1))));
        let s = schedule(
            &tranche(&[(1.0, 0.01), (2.0, 0.02)], "repay"),
            a,
            YearMonth::new(2033, 1),
        );
        assert_eq!(s.repay_at, None);
        // At most 1000 periods
        let s = schedule(&tranche(&[(1.0 / 12.0, 0.01)], "renew"), a, YearMonth::new(2200, 1));
        assert_eq!(s.periods.len(), 1000);
        assert!(schedule(&tranche(&[], "renew"), a, YearMonth::new(2036, 1))
            .periods
            .is_empty());
    }
}

#[cfg(test)]
mod tests_pre_levels {
    use super::pre_levels;

    #[test]
    fn issue_103_pre_levels_walk_back_from_the_start_level() {
        // Lead-in +10 %, then +20 %: start = 1, before that 1/1.2, before that 1/1.2/1.1.
        let l = pre_levels(&[0.10, 0.20], 3);
        assert_eq!(l.len(), 3);
        assert!((l[2] - 1.0).abs() < 1e-12);
        assert!((l[1] - 1.0 / 1.2).abs() < 1e-12);
        assert!((l[0] - 1.0 / 1.2 / 1.1).abs() < 1e-12);
        assert_eq!(pre_levels(&[], 3), vec![1.0]);
    }

    #[test]
    fn issue_103_pre_levels_total_loss_stays_finite() {
        // A return of -100 % must not divide by 0.
        let l = pre_levels(&[-1.0], 1);
        assert_eq!(l.len(), 2);
        assert!(l[0].is_finite() && l[0] > 1e6);
        assert_eq!(l[1], 1.0);
    }
}

#[cfg(test)]
mod tests_ledger {
    use super::*;

    /// #133: Every field has its own power of two, so every wrong sign changes the sum.
    #[test]
    fn issue_133_ledger_sum_signs() {
        let l = Ledger {
            income: 1.0,
            extras: 2.0,
            savings: 4.0,
            living: 8.0,
            housing: 16.0,
            amortization: 32.0,
            taxes: 64.0,
            market: 128.0,
            pk_contributions: 256.0,
            pk_interest: 512.0,
            p3a_interest: 1024.0,
            revaluation: 2048.0,
            annuity: 4096.0,
        };
        let plus = 1.0 + 2.0 + 4.0 + 32.0 + 128.0 + 256.0 + 512.0 + 1024.0 + 2048.0;
        let minus = 8.0 + 16.0 + 64.0 + 4096.0;
        assert_eq!(l.sum(), plus - minus);
        assert_eq!(Ledger::default().sum(), 0.0);
    }
}

#[cfg(test)]
mod tests_deductions {
    use super::*;

    fn simulator(v: serde_json::Value) -> Simulator {
        Simulator::new(&serde_json::from_value(v).unwrap()).unwrap()
    }

    fn plan(deduct: bool) -> serde_json::Value {
        serde_json::json!({
            "project": { "name": "Abzug" },
            "household": { "persons": [ { "id": "A", "name": "A", "birth": "1980-01" } ] },
            "milestones": [
                { "id": "start", "kind": "date", "date": "2030-01" },
                { "id": "end", "kind": "date", "date": "2031-12" }
            ],
            "pension": { "pk": { "capital": 0, "at": "start", "lumpsum": 0, "conversion": 0 }, "ahv": { "monthly": 0, "at": "start" } },
            "flows": { "income": 0, "spending": 0 },
            "wealth": { "stocks": 0, "bonds": 0, "cash": 0, "stockShare": 0, "reserveMonths": 0 },
            "returns": { "source": "fixed", "fixed": { "stocks": 0, "bonds": 0, "inflation": 0 } },
            "taxes": { "multiplier": 1.0, "income": [[0, 0.1]], "wealth": [], "capital": [], "deductPillar3a": deduct },
            "limits": { "pillar3aMax": 7000 }
        })
    }

    /// #131/AK1: At most the maximum per person (settings), growing with the price index; switched off: nothing.
    #[test]
    fn issue_131_ak1_pillar3a_capped_per_person() {
        let s = simulator(plan(true));
        let acc = YearDeductions {
            months: 12,
            pillar3a: [9000.0, 0.0],
            ..Default::default()
        };
        let (t, b) = s.income_tax(YearMonth::new(2030, 12), 100_000.0, &[], &acc, 1.0);
        assert_eq!(b.pillar3a, 7000.0);
        assert!((t - 9300.0).abs() < 1e-9);
        assert!(
            (s.income_tax(YearMonth::new(2030, 12), 100_000.0, &[], &acc, 1.1)
                .1
                .pillar3a
                - 7700.0)
                .abs()
                < 1e-9
        );
        let acc = YearDeductions {
            months: 12,
            pillar3a: [5000.0, 0.0],
            ..Default::default()
        };
        assert_eq!(
            s.income_tax(YearMonth::new(2030, 12), 100_000.0, &[], &acc, 1.0)
                .1
                .pillar3a,
            5000.0
        );
        assert_eq!(
            simulator(plan(false))
                .income_tax(YearMonth::new(2030, 12), 100_000.0, &[], &acc, 1.0)
                .1
                .pillar3a,
            0.0
        );
    }

    /// #230/AK1: Hand value: a deduction of 20'000 plus «zusätzlich» 3'000 per year gives 23'000 (income 100'000,
    /// taxable 77'000); the extra is averaged over the months of the year and grows with the price index.
    #[test]
    fn issue_230_ak1_extra_hand_value() {
        let s = simulator(plan(false));
        let acc = YearDeductions {
            months: 12,
            own_amount: 12.0 * 20_000.0,
            extra: 12.0 * 3000.0,
            ..Default::default()
        };
        let (t, b) = s.income_tax(YearMonth::new(2030, 12), 100_000.0, &[], &acc, 1.0);
        assert_eq!(b.own + b.extra, 23_000.0);
        assert_eq!(b.taxable_canton, 77_000.0);
        assert!((t - 7700.0).abs() < 1e-9);
        let (_, b) = s.income_tax(YearMonth::new(2030, 12), 100_000.0, &[], &acc, 1.1);
        assert!((b.extra - 3300.0).abs() < 1e-9);
        // #230/AK2: one-off on top, nominal
        let acc = YearDeductions {
            one_off: 50_000.0,
            ..acc
        };
        let (_, b) = s.income_tax(YearMonth::new(2030, 12), 100_000.0, &[], &acc, 1.1);
        assert_eq!(b.one_off, 50_000.0);
    }

    /// #131/AK3: Before the first row there is no deduction; months without a row count for the share of the year.
    #[test]
    fn issue_131_ak3_deduction_row_lookup() {
        let mut v = plan(false);
        v["taxes"]["deductions"] = serde_json::json!([
            { "from": "2030-07", "kind": "own", "amount": 1200 },
            { "from": "2031-01", "kind": "own", "amount": 2400 } ]);
        let s = simulator(v);
        assert!(s.deduction_at(YearMonth::new(2030, 6)).is_none());
        assert_eq!(s.deduction_at(YearMonth::new(2030, 7)).unwrap().amount, 1200.0);
        assert_eq!(s.deduction_at(YearMonth::new(2031, 1)).unwrap().amount, 2400.0);
        let mut acc = YearDeductions::default();
        for m in 1..=12 {
            s.add_deduction_month(&mut acc, YearMonth::new(2030, m), 1000.0);
        }
        let (_, b) = s.income_tax(YearMonth::new(2030, 12), 12_000.0, &[], &acc, 1.0);
        assert_eq!(b.own, 600.0);
        assert_eq!(b.taxable_canton, 11_400.0);
    }
}
