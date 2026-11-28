//! #63: Built-in tax data file for the mode «Wohnort» (place of residence).
//!
//! The file `data/steuern-<year>.json` is built by the tool `tools/steuerdaten/build.py` from the ESTV base data. It
//! holds per canton the income tariff as a curve `[taxable income, rate]` of the simple tax (tax multiplier 100 %),
//! separately for single persons and married couples, the standard deductions as rules, the wealth curves (rate on
//! the wealth before the social deductions) and the tax multipliers per municipality; plus the tariff and the
//! deductions of the direct federal tax.
//!
//! Income tax = cantonal curve(taxable income canton) × (multiplier canton + church) plus municipal
//! curve(taxable income canton) × multiplier municipality plus federal curve(taxable income federal).
//!
//! The municipal curve is the cantonal one, except in Schwyz and Valais. Wealth tax = wealth curve × (multiplier
//! canton + municipality + church), without federal tax.
//!
//! #297: Valais reduces the income tax of married couples by a share of the tax (after the multiplier), at least and
//! at most an indexed amount, separately for canton and municipality (`incomeRebate`); the church tax is not reduced.
//!
//! #64: Lump sums from pension savings = capital curve of the canton × (multiplier canton + church) plus capital curve
//! of the municipality × multiplier municipality plus federal capital curve (one fifth of the federal tariff, without
//! deductions). The curves hold the rate of the simple tax on the whole lump sum; the multipliers are those of the
//! income tax.
//!
//! #65: Property gain tax: the rule set of the canton in which the property lies (`propertyGain`, format of
//! [`crate::model::PropertyGainTax`]); cantons that compute with the tax multiplier take that of the municipality of
//! residence for a property in the canton of residence, otherwise the median of the canton (`gainMultiplier`).

use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

use crate::model::{Church, DeductionRule, ManualMultipliers, PropertyGainTax, TaxLocation};
use crate::tax::{rate_at, tax_indexed};
use crate::{Error, Result};

const BUILTIN: &str = include_str!("../data/steuern-2026.json");

/// Rate table `[amount, rate]` as in [`crate::model::Taxes`].
pub type Table = Vec<(f64, f64)>;

/// Curves for single persons and married couples: pairs `[amount, rate]`.
#[derive(Clone, Debug, Deserialize)]
pub struct Curves {
    pub single: Table,
    pub married: Table,
}

impl Curves {
    pub fn get(&self, married: bool) -> &[(f64, f64)] {
        if married {
            &self.married
        } else {
            &self.single
        }
    }
}

/// A standard deduction: ESTV id, kind (`insurance`, `savingsInterest`, `social`) and rule.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct NamedRule {
    pub id: String,
    pub kind: String,
    #[serde(flatten)]
    pub rule: DeductionRule,
}

/// Deductions of a marital status: variant without (`retired`) and with contributions to the 2nd and 3rd pillar
/// (`working`).
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Variants {
    pub retired: Vec<NamedRule>,
    pub working: Vec<NamedRule>,
}

/// Flat deduction for professional expenses; `approx`: the ESTV data only name a maximum (fixed deduction).
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EarnedRule {
    #[serde(flatten)]
    pub rule: DeductionRule,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub approx: bool,
}

/// Reference amount of the second-earner deduction.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SecondEarnerReference {
    /// The lower earned income after the flat deduction for professional expenses (federal, most cantons).
    #[default]
    Lower,
    /// The sum of the earned incomes (Bern).
    Total,
}

/// Second-earner deduction for married couples with two earned incomes: the rule applied to the reference amount, at
/// most the lower earned income after the flat deduction for professional expenses.
#[derive(Clone, Debug, Default, PartialEq, Deserialize, Serialize)]
pub struct SecondEarner {
    #[serde(flatten)]
    pub rule: DeductionRule,
    #[serde(default)]
    pub reference: SecondEarnerReference,
}

/// Standard deductions from the income of one authority (canton or federal).
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Deductions {
    pub single: Variants,
    pub married: Variants,
    pub earned: EarnedRule,
    /// Second-earner deduction; missing if the authority has none (Thurgau).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub second_earner: Option<SecondEarner>,
}

impl Deductions {
    /// Rules for the marital status.
    pub fn applied(&self, married: bool) -> Applied {
        let v = if married { &self.married } else { &self.single };
        let rules = |r: &[NamedRule]| r.iter().map(|n| n.rule.clone()).collect();
        let kinds = |r: &[NamedRule]| r.iter().map(|n| n.kind.clone()).collect();
        Applied {
            retired: rules(&v.retired),
            working: rules(&v.working),
            earned: self.earned.rule.clone(),
            retired_kinds: kinds(&v.retired),
            working_kinds: kinds(&v.working),
            second_earner: self.second_earner.clone().filter(|_| married),
        }
    }
}

/// Deduction rules of one authority for one marital status.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Applied {
    pub retired: Vec<DeductionRule>,
    pub working: Vec<DeductionRule>,
    pub earned: DeductionRule,
    /// #131: Kind of each rule (`social`, `insurance`, `savingsInterest`), in the order of `retired` and `working`.
    pub retired_kinds: Vec<String>,
    pub working_kinds: Vec<String>,
    /// Second-earner deduction; only for married couples.
    pub second_earner: Option<SecondEarner>,
}

impl Applied {
    /// Total deduction from the income `income` (earned income + pension fund pension + AHV) of a year. `earned`
    /// holds the earned income per person: with earned income the variant «working» applies, otherwise «retired»;
    /// each earning person gets the flat deduction for professional expenses on their earned income. `index` scales
    /// the amounts (#66).
    pub fn total(&self, income: f64, earned: &[f64], index: f64) -> f64 {
        let working = earned.iter().any(|&e| e > 0.0);
        let general: f64 = if working { &self.working } else { &self.retired }
            .iter()
            .map(|r| r.amount(income, index))
            .sum();
        general
            + earned.iter().map(|&e| self.earned.earned_amount(e, index)).sum::<f64>()
            + self.second_earner_amount(earned, index)
    }
    /// Second-earner deduction: only for married couples (rule present) in which both persons have earned income; at
    /// most the lower earned income after the flat deduction for professional expenses.
    pub fn second_earner_amount(&self, earned: &[f64], index: f64) -> f64 {
        let Some(s) = &self.second_earner else {
            return 0.0;
        };
        if earned.len() < 2 || earned.iter().any(|&e| e <= 0.0) {
            return 0.0;
        }
        let lower = earned
            .iter()
            .map(|&e| (e - self.earned.earned_amount(e, index)).max(0.0))
            .fold(f64::INFINITY, f64::min);
        let reference = match s.reference {
            SecondEarnerReference::Lower => lower,
            SecondEarnerReference::Total => earned.iter().sum(),
        };
        s.rule.amount(reference, index).min(lower)
    }
    /// #131: The same deduction as [`Applied::total`], split by kind (`social`, `insurance`, `savingsInterest` and
    /// `professional` for the flat deduction for professional expenses); kinds without an amount are left out.
    pub fn parts(&self, income: f64, earned: &[f64], index: f64) -> Vec<(String, f64)> {
        let working = earned.iter().any(|&e| e > 0.0);
        let (rules, kinds) = if working {
            (&self.working, &self.working_kinds)
        } else {
            (&self.retired, &self.retired_kinds)
        };
        let mut out: Vec<(String, f64)> = Vec::new();
        let mut add = |kind: &str, amount: f64| {
            if amount == 0.0 {
                return;
            }
            match out.iter_mut().find(|p| p.0 == kind) {
                Some(p) => p.1 += amount,
                None => out.push((kind.to_string(), amount)),
            }
        };
        for (k, r) in rules.iter().enumerate() {
            add(kinds.get(k).map_or("social", String::as_str), r.amount(income, index));
        }
        for &e in earned {
            add("professional", self.earned.earned_amount(e, index));
        }
        add("secondEarner", self.second_earner_amount(earned, index));
        out
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Canton {
    pub id: String,
    pub name: String,
    /// Tariff of the income tax on the taxable income.
    pub income: Curves,
    /// Rate on the wealth before the social deductions (included).
    pub wealth: Curves,
    /// Own tariff of the municipal income tax (Schwyz, Valais; otherwise the cantonal one applies).
    #[serde(default)]
    pub municipal_income: Option<Curves>,
    /// #296: Index of the municipal tariff per municipality (Valais): `(BFS number, factor)`; the municipal tariff (income
    /// and lump sums) applies to the amount divided by the factor. Missing municipality: factor 1.
    #[serde(default)]
    pub municipal_index: Vec<(u32, f64)>,
    pub deductions: Deductions,
    /// #297: Reduction of the income tax of married couples (Valais), for canton and municipality.
    #[serde(default)]
    pub income_rebate: IncomeRebate,
    /// #64: Rate of the simple tax on lump sums from pension savings (multiplier 100 %), on the whole lump sum.
    pub capital: Curves,
    /// #64: Own capital curve of the municipal tax (otherwise the cantonal one applies).
    #[serde(default)]
    pub capital_municipal: Option<Curves>,
    /// #64: Rule by which the canton taxes lump sums (German text of the tool).
    pub capital_rule: String,
    /// #64: Note if the rule is only approximated; missing for an exact rule.
    #[serde(default)]
    pub capital_approx: Option<String>,
    /// #65: Rule set of the property gain tax.
    pub property_gain: PropertyGainTax,
    /// #65: The rule as text (German; the user interface shows `propertyGainRule.<canton>`).
    pub property_gain_rule: String,
    /// #65: Note if the rule is only approximated.
    #[serde(default)]
    pub property_gain_approx: Option<String>,
    /// #65: Median tax multiplier (canton + municipality) for a property outside the canton of residence.
    pub gain_multiplier: f64,
}

/// #297: Reduction of the income tax: `rate` of the tax after the multiplier, at least `min`, at most `max` (0 = no
/// maximum), never more than the tax; `min` and `max` in prices of the tax data (indexed like the tariff).
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct TaxRebate {
    pub rate: f64,
    pub min: f64,
    pub max: f64,
    /// Article of the law and publication of the indexed amounts.
    #[serde(default)]
    pub source: String,
}

impl TaxRebate {
    /// Reduction of the tax `tax`; `index`: price index for `min` and `max` (1.0 = prices of the tax data).
    pub fn amount(&self, tax: f64, index: f64) -> f64 {
        if tax <= 0.0 {
            return 0.0;
        }
        let mut v = (self.rate * tax).max(self.min * index);
        if self.max > 0.0 {
            v = v.min(self.max * index);
        }
        v.min(tax)
    }
}

/// #297: Reductions of the income tax of married couples, per authority (missing: none).
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct IncomeRebate {
    #[serde(default)]
    pub canton: Option<TaxRebate>,
    #[serde(default)]
    pub municipal: Option<TaxRebate>,
}

/// Tax multipliers of a municipality as shares (0.95 = 95 %): canton, municipality, reformed, Roman Catholic, Christ
/// Catholic.
pub type Multipliers = [f64; 5];

/// Municipality: BFS number, name, canton, multipliers income, multipliers wealth.
#[derive(Clone, Debug, Deserialize)]
pub struct Municipality(pub u32, pub String, pub String, pub Multipliers, pub Multipliers);

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TaxData {
    pub year: i32,
    pub source: String,
    pub note: String,
    /// Tariff of the direct federal tax on the taxable income.
    pub federal: Curves,
    pub federal_deductions: Deductions,
    /// #64: Federal tax on lump sums from pension savings (one fifth of the tariff, without deductions).
    pub federal_capital: Curves,
    /// #65: Source of the rule sets of the property gain tax.
    pub property_gain_source: String,
    pub cantons: Vec<Canton>,
    pub municipalities: Vec<Municipality>,
}

/// The built-in tax data file.
pub fn builtin() -> &'static TaxData {
    static DATA: OnceLock<TaxData> = OnceLock::new();
    DATA.get_or_init(|| serde_json::from_str(BUILTIN).expect("mitgelieferte Steuerdatei ist gültig"))
}

/// Index of the church tax of the denomination in [`Multipliers`].
fn church_index(c: Church) -> Option<usize> {
    match c {
        Church::None => None,
        Church::Reformed => Some(2),
        Church::Catholic => Some(3),
        Church::ChristCatholic => Some(4),
    }
}

/// #285: the multipliers of the municipality with those entered by hand put in (canton, municipality, church of the
/// denomination).
fn by_hand(m: &Multipliers, hand: Option<&ManualMultipliers>, c: Church) -> Multipliers {
    let mut m = *m;
    if let Some(h) = hand {
        if let Some(v) = h.canton {
            m[0] = v;
        }
        if let Some(v) = h.municipal {
            m[1] = v;
        }
        if let (Some(v), Some(i)) = (h.church, church_index(c)) {
            m[i] = v;
        }
    }
    m
}

/// Multiplier of the church tax for the denomination.
fn church(m: &Multipliers, c: Church) -> f64 {
    match c {
        Church::None => 0.0,
        Church::Reformed => m[2],
        Church::Catholic => m[3],
        Church::ChristCatholic => m[4],
    }
}

impl TaxData {
    pub fn canton(&self, id: &str) -> Option<&Canton> {
        self.cantons.iter().find(|c| c.id == id)
    }

    pub fn municipality(&self, bfs: u32) -> Option<&Municipality> {
        self.municipalities.iter().find(|m| m.0 == bfs)
    }

    /// Tables and deduction rules of a place of residence (multipliers already included). Errors: canton unknown,
    /// municipality missing or not in the canton.
    pub fn resolve(&self, loc: &TaxLocation, married: bool) -> Result<Resolved> {
        let canton = self
            .canton(&loc.canton)
            .ok_or_else(|| Error::Invalid(msg!(tax_canton_unknown, canton = loc.canton.clone())))?;
        let bfs = loc
            .municipality
            .ok_or_else(|| Error::Invalid(msg!(tax_municipality_missing)))?;
        let muni = self.municipality(bfs).filter(|m| m.2 == canton.id).ok_or_else(|| {
            Error::Invalid(msg!(
                tax_municipality_not_in_canton,
                bfs = bfs,
                canton = canton.id.clone()
            ))
        })?;
        let hand = loc.multipliers.as_ref();
        let (ek, vm) = (&by_hand(&muni.3, hand, loc.church), &by_hand(&muni.4, hand, loc.church));
        let kt = canton.income.get(married);
        let factor = canton.municipal_index.iter().find(|m| m.0 == bfs).map_or(1.0, |m| m.1);
        let gem = stretch(canton.municipal_income.as_ref().map_or(kt, |c| c.get(married)), factor);
        let ck = canton.capital.get(married);
        let cg = stretch(canton.capital_municipal.as_ref().map_or(ck, |c| c.get(married)), factor);
        // #297: reductions for married couples on the tax of canton and municipality (after the multiplier)
        let mut rebates = Vec::new();
        if married {
            let r = &canton.income_rebate;
            if let Some(c) = &r.canton {
                rebates.push(Rebate {
                    table: combine(&[(kt, ek[0])]),
                    rule: c.clone(),
                });
            }
            if let Some(m) = &r.municipal {
                rebates.push(Rebate {
                    table: combine(&[(&gem, ek[1])]),
                    rule: m.clone(),
                });
            }
        }
        Ok(Resolved {
            cantonal: combine(&[(kt, ek[0] + church(ek, loc.church)), (&gem, ek[1])]),
            rebates,
            federal: self.federal.get(married).to_vec(),
            wealth: combine(&[(canton.wealth.get(married), vm[0] + vm[1] + church(vm, loc.church))]),
            capital: combine(&[
                (ck, ek[0] + church(ek, loc.church)),
                (&cg, ek[1]),
                (self.federal_capital.get(married), 1.0),
            ]),
            canton_deductions: canton.deductions.applied(married),
            federal_deductions: self.federal_deductions.applied(married),
        })
    }

    /// #65: Rule set of the property gain tax and tax multiplier for a property in the canton `canton` (missing or
    /// empty: canton of residence). In the canton of residence the multiplier of the municipality of residence
    /// (canton + municipality) applies, otherwise the median of the canton.
    pub fn property_gain(&self, loc: &TaxLocation, canton: Option<&str>) -> Result<(PropertyGainTax, f64)> {
        let id = canton.filter(|c| !c.is_empty()).unwrap_or(&loc.canton);
        let c = self
            .canton(id)
            .ok_or_else(|| Error::Invalid(msg!(gain_tax_canton_unknown, canton = id)))?;
        let home = loc
            .municipality
            .and_then(|b| self.municipality(b))
            .filter(|m| m.2 == c.id);
        let multiplier = home.map_or(c.gain_multiplier, |m| {
            let m = by_hand(&m.3, loc.multipliers.as_ref(), loc.church);
            m[0] + m[1]
        });
        Ok((c.property_gain.clone(), multiplier))
    }
}

/// Tables and deduction rules of a place of residence, multipliers included.
#[derive(Clone, Debug)]
pub struct Resolved {
    /// Canton, church and municipality on the cantonal taxable income.
    pub cantonal: Table,
    /// Direct federal tax on the federal taxable income.
    pub federal: Table,
    /// Wealth tax (canton, municipality, church) on the wealth before deductions.
    pub wealth: Table,
    /// #64: Tax on lump sums from pension savings (canton, municipality, church, federal) on the whole lump sum.
    pub capital: Table,
    pub canton_deductions: Applied,
    pub federal_deductions: Applied,
    /// #297: Reductions of the income tax (married couples in Valais), each on its part of the cantonal tax.
    pub rebates: Vec<Rebate>,
}

/// #297: A reduction of the income tax: rule and the part of the tax it applies to (rate table, multiplier
/// included) on the cantonal taxable income.
#[derive(Clone, Debug)]
pub struct Rebate {
    pub table: Table,
    pub rule: TaxRebate,
}

impl Resolved {
    /// Taxable income for canton and federal tax: income minus the standard deductions, never negative.
    pub fn taxable(&self, income: f64, earned: &[f64], index: f64) -> (f64, f64) {
        (
            (income - self.canton_deductions.total(income, earned, index)).max(0.0),
            (income - self.federal_deductions.total(income, earned, index)).max(0.0),
        )
    }

    /// Income tax of a year (canton, municipality, church and federal tax). `index`: price index for the amounts of
    /// tariffs and deductions (#66; 1.0 = unadjusted).
    pub fn income_tax(&self, income: f64, earned: &[f64], index: f64) -> f64 {
        let (c, f) = self.taxable(income, earned, index);
        self.income_tax_on(c, f, index)
    }

    /// #131: Proposal of the canton: standard deductions for canton and federal tax (rules of the tax data file).
    pub fn proposal(&self, income: f64, earned: &[f64], index: f64) -> (f64, f64) {
        (
            self.canton_deductions.total(income, earned, index),
            self.federal_deductions.total(income, earned, index),
        )
    }

    /// #131: Income tax of a year on the taxable income for canton (`canton`) and federal tax (`federal`).
    pub fn income_tax_on(&self, canton: f64, federal: f64, index: f64) -> f64 {
        let rebate: f64 = self
            .rebates
            .iter()
            .map(|r| r.rule.amount(tax_indexed(&r.table, canton, 1.0, index), index))
            .sum();
        tax_indexed(&self.cantonal, canton, 1.0, index) - rebate + tax_indexed(&self.federal, federal, 1.0, index)
    }

    /// Wealth tax of a year.
    pub fn wealth_tax(&self, wealth: f64, index: f64) -> f64 {
        tax_indexed(&self.wealth, wealth, 1.0, index)
    }

    /// #274: Social deductions from wealth estimated from the wealth curve (they are included in it): the wealth below
    /// which the curve has no tax, prices of the tax data. The tax at the first grid point with a rate is traced back
    /// with the marginal rate up to the next grid point, within the step from the last grid point without a rate.
    pub fn wealth_allowance(&self) -> f64 {
        let t = &self.wealth;
        let Some(k) = t.iter().position(|p| p.1 > 0.0) else {
            return t.last().map_or(0.0, |p| p.0);
        };
        if k == 0 {
            return 0.0;
        }
        let (x0, (x1, r1)) = (t[k - 1].0, t[k]);
        let tax1 = x1 * r1;
        let marginal = t.get(k + 1).map_or(0.0, |&(x2, r2)| (x2 * r2 - tax1) / (x2 - x1));
        if marginal > 0.0 {
            (x1 - tax1 / marginal).clamp(x0, x1)
        } else {
            x0
        }
    }
}

/// Sum of weighted rate tables on the united grid points. Because the engine interpolates linearly between the grid
/// points, the sum is exact.
/// #296: Curve of rates on amounts stretched by `factor`: the rate at `x × factor` is the rate of the curve at `x`
/// (indexed tariff of a municipality in Valais). Factor 1 leaves the curve unchanged.
pub fn stretch(curve: &[(f64, f64)], factor: f64) -> Vec<(f64, f64)> {
    curve.iter().map(|&(x, r)| (x * factor, r)).collect()
}

pub fn combine(parts: &[(&[(f64, f64)], f64)]) -> Table {
    let mut xs: Vec<f64> = parts.iter().flat_map(|(t, _)| t.iter().map(|p| p.0)).collect();
    xs.sort_by(f64::total_cmp);
    xs.dedup();
    xs.into_iter()
        .map(|x| (x, parts.iter().map(|(t, w)| w * rate_at(t, x)).sum()))
        .collect()
}

/// Cantons and municipalities for the selection in the user interface (command `tax_locations`).
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocationsOut {
    pub year: i32,
    pub source: String,
    pub note: String,
    pub federal_deductions: Deductions,
    pub property_gain_source: String,
    /// #273: Default tax value of properties as a share of the market value (the same for all cantons).
    pub tax_value_default: f64,
    pub cantons: Vec<CantonOut>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CantonOut {
    pub id: String,
    pub name: String,
    pub deductions: Deductions,
    /// #64: Rule for lump sums and the note if it is only approximated.
    pub capital_rule: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capital_approx: Option<String>,
    pub property_gain_rule: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub property_gain_approx: Option<String>,
    /// Municipalities: BFS number, name, multipliers income and wealth, sorted by name.
    pub municipalities: Vec<(u32, String, Multipliers, Multipliers)>,
}

pub fn locations() -> LocationsOut {
    let d = builtin();
    let mut cantons: Vec<CantonOut> = d
        .cantons
        .iter()
        .map(|c| {
            let mut municipalities: Vec<_> = d
                .municipalities
                .iter()
                .filter(|m| m.2 == c.id)
                .map(|m| (m.0, m.1.clone(), m.3, m.4))
                .collect();
            municipalities.sort_by(|a, b| a.1.cmp(&b.1));
            CantonOut {
                id: c.id.clone(),
                name: c.name.clone(),
                deductions: c.deductions.clone(),
                capital_rule: c.capital_rule.clone(),
                capital_approx: c.capital_approx.clone(),
                property_gain_rule: c.property_gain_rule.clone(),
                property_gain_approx: c.property_gain_approx.clone(),
                municipalities,
            }
        })
        .collect();
    cantons.sort_by(|a, b| a.name.cmp(&b.name));
    LocationsOut {
        year: d.year,
        source: d.source.clone(),
        note: d.note.clone(),
        federal_deductions: d.federal_deductions.clone(),
        tax_value_default: crate::model::DEFAULT_TAX_VALUE_RATE,
        property_gain_source: d.property_gain_source.clone(),
        cantons,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn combine_sums_weighted_rates_on_all_points() {
        let a = [(0.0, 0.0), (100.0, 0.1)];
        let b = [(0.0, 0.01), (50.0, 0.02)];
        let c = combine(&[(&a, 2.0), (&b, 1.0)]);
        assert_eq!(c.len(), 3);
        assert!((c[1].1 - (2.0 * 0.05 + 0.02)).abs() < 1e-12);
        assert!((c[2].1 - (2.0 * 0.1 + 0.02)).abs() < 1e-12);
    }

    #[test]
    fn applied_total_switches_variant_and_adds_earned_per_person() {
        let r = |base| DeductionRule {
            base,
            ..Default::default()
        };
        let a = Applied {
            retired: vec![r(4000.0), r(1000.0)],
            working: vec![r(3000.0)],
            earned: DeductionRule {
                rate: 0.03,
                min: 2000.0,
                max: 4000.0,
                ..Default::default()
            },
            ..Default::default()
        };
        assert_eq!(a.total(50_000.0, &[], 1.0), 5000.0);
        assert_eq!(a.total(50_000.0, &[0.0, 0.0], 1.0), 5000.0);
        // 3000 + 3 % of 100 000 + minimum 2000 of 10 000
        assert_eq!(
            a.total(110_000.0, &[100_000.0, 10_000.0], 1.0),
            3000.0 + 3000.0 + 2000.0
        );
        // with the price index the fixed amounts grow
        assert_eq!(a.total(50_000.0, &[], 2.0), 10_000.0);
        // ... also the minimum and the maximum of the professional expenses (3 % of 10 000 = 300 < 2 × 2000)
        assert_eq!(a.total(10_000.0, &[10_000.0], 2.0), 6000.0 + 4000.0);
        assert_eq!(a.total(500_000.0, &[500_000.0], 2.0), 6000.0 + 8000.0);
    }

    /// #131/AK5: Parts of the proposal by kind: amounts of the same kind add up, without earned income (also with
    /// zeros per person) the variant «retired», kinds without an amount are left out.
    #[test]
    fn issue_131_ak5_parts_by_kind() {
        let r = |base| DeductionRule {
            base,
            ..Default::default()
        };
        let kinds = |k: &[&str]| k.iter().map(|x| x.to_string()).collect();
        let a = Applied {
            retired: vec![r(4000.0), r(1000.0), r(0.0)],
            working: vec![r(3000.0)],
            earned: DeductionRule {
                rate: 0.03,
                ..Default::default()
            },
            retired_kinds: kinds(&["social", "social", "insurance"]),
            working_kinds: kinds(&["insurance"]),
            second_earner: None,
        };
        assert_eq!(a.parts(50_000.0, &[0.0], 1.0), vec![("social".to_string(), 5000.0)]);
        assert_eq!(
            a.parts(50_000.0, &[50_000.0], 1.0),
            vec![("insurance".to_string(), 3000.0), ("professional".to_string(), 1500.0)]
        );
        let total: f64 = a.parts(50_000.0, &[50_000.0], 1.0).iter().map(|p| p.1).sum();
        assert_eq!(total, a.total(50_000.0, &[50_000.0], 1.0));
    }
}
