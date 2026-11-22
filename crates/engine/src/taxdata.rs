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

use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

use crate::model::{Church, DeductionRule, TaxLocation};
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

/// Standard deductions from the income of one authority (canton or federal).
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Deductions {
    pub single: Variants,
    pub married: Variants,
    pub earned: EarnedRule,
}

impl Deductions {
    /// Rules for the marital status.
    pub fn applied(&self, married: bool) -> Applied {
        let v = if married { &self.married } else { &self.single };
        let rules = |r: &[NamedRule]| r.iter().map(|n| n.rule.clone()).collect();
        Applied {
            retired: rules(&v.retired),
            working: rules(&v.working),
            earned: self.earned.rule.clone(),
        }
    }
}

/// Deduction rules of one authority for one marital status.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Applied {
    pub retired: Vec<DeductionRule>,
    pub working: Vec<DeductionRule>,
    pub earned: DeductionRule,
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
        general + earned.iter().map(|&e| self.earned.earned_amount(e, index)).sum::<f64>()
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
    pub deductions: Deductions,
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
    pub cantons: Vec<Canton>,
    pub municipalities: Vec<Municipality>,
}

/// The built-in tax data file.
pub fn builtin() -> &'static TaxData {
    static DATA: OnceLock<TaxData> = OnceLock::new();
    DATA.get_or_init(|| serde_json::from_str(BUILTIN).expect("mitgelieferte Steuerdatei ist gültig"))
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
        let (ek, vm) = (&muni.3, &muni.4);
        let kt = canton.income.get(married);
        let gem = canton.municipal_income.as_ref().map_or(kt, |c| c.get(married));
        Ok(Resolved {
            cantonal: combine(&[(kt, ek[0] + church(ek, loc.church)), (gem, ek[1])]),
            federal: self.federal.get(married).to_vec(),
            wealth: combine(&[(canton.wealth.get(married), vm[0] + vm[1] + church(vm, loc.church))]),
            canton_deductions: canton.deductions.applied(married),
            federal_deductions: self.federal_deductions.applied(married),
        })
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
    pub canton_deductions: Applied,
    pub federal_deductions: Applied,
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
        tax_indexed(&self.cantonal, c, 1.0, index) + tax_indexed(&self.federal, f, 1.0, index)
    }

    /// Wealth tax of a year.
    pub fn wealth_tax(&self, wealth: f64, index: f64) -> f64 {
        tax_indexed(&self.wealth, wealth, 1.0, index)
    }
}

/// Sum of weighted rate tables on the united grid points. Because the engine interpolates linearly between the grid
/// points, the sum is exact.
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
    pub cantons: Vec<CantonOut>,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CantonOut {
    pub id: String,
    pub name: String,
    pub deductions: Deductions,
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
}
