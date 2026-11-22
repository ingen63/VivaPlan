//! Tax computation with simple rate tables.

use crate::model::{GainModel, PropertyGainTax};

/// Rate for an amount: interpolated linearly between the brackets, continued constantly outside.
pub fn rate_at(table: &[(f64, f64)], x: f64) -> f64 {
    if table.is_empty() {
        return 0.0;
    }
    let mut t = table.to_vec();
    t.sort_by(|a, b| a.0.total_cmp(&b.0));
    if x <= t[0].0 {
        return t[0].1;
    }
    for w in t.windows(2) {
        let ((x0, r0), (x1, r1)) = (w[0], w[1]);
        if x <= x1 {
            if x1 == x0 {
                return r1;
            }
            return r0 + (r1 - r0) * (x - x0) / (x1 - x0);
        }
    }
    t[t.len() - 1].1
}

/// #134: Does the tax `x × rate_at(table, x)` fall anywhere when the amount rises? Returns the row (index in the
/// entered order) at which it falls, otherwise `None`.
///
/// Between two brackets the rate is linear, so the tax is quadratic and its derivative `r(x) + x × s` linear; it
/// suffices to check the derivative at both ends of the section. Outside the brackets the rate is constant (≥ 0).
/// Several brackets with the same amount are a jump: [`rate_at`] takes the rate of the first on the bracket and right
/// after it that of the last of these rows (stable sort). If the last is lower, the tax falls.
pub fn falling_row(table: &[(f64, f64)]) -> Option<usize> {
    let mut idx: Vec<usize> = (0..table.len()).collect();
    idx.sort_by(|&a, &b| table[a].0.total_cmp(&table[b].0));
    const EPS: f64 = 1e-12;
    let mut first = 0;
    for k in 0..idx.len() {
        let (x, r) = table[idx[k]];
        if k > 0 && table[idx[k - 1]].0 != x {
            first = k;
        }
        // Jump at the end of a group of equal amounts
        let group_ends = k + 1 == idx.len() || table[idx[k + 1]].0 != x;
        if group_ends && k > first && x > 0.0 && r < table[idx[first]].1 - EPS {
            return Some(idx[k]);
        }
        // Section up to the next bracket (from the last row of this group to the first of the next)
        if group_ends && k + 1 < idx.len() {
            let (x1, r1) = table[idx[k + 1]];
            let s = (r1 - r) / (x1 - x);
            if r + x * s < -EPS || r1 + x1 * s < -EPS {
                return Some(idx[k + 1]);
            }
        }
    }
    None
}

/// Tax = amount × rate × tax multiplier, rounded to centimes.
pub fn tax(table: &[(f64, f64)], x: f64, multiplier: f64) -> f64 {
    if x < 0.0 {
        return 0.0;
    }
    round2(x * rate_at(table, x) * multiplier)
}

/// #66: Like [`tax`], but the amounts of the table are scaled by the price index `index` (1.0 = simulation start);
/// the rates stay the same. Equivalently, the rate applies to the amount in prices of the start. If `index` is not
/// positive, the table applies unadjusted.
pub fn tax_indexed(table: &[(f64, f64)], x: f64, multiplier: f64, index: f64) -> f64 {
    if x < 0.0 {
        return 0.0;
    }
    let real = if index > 0.0 { x / index } else { x };
    round2(x * rate_at(table, real) * multiplier)
}

/// Property gain tax: the gain is taxed in brackets (width and rate per bracket), then reduced.
pub fn property_gain_tax(def: Option<&PropertyGainTax>, gain: f64) -> f64 {
    property_gain_tax_held(def, gain, None, 1.0)
}

/// Like [`property_gain_tax`], with a surcharge or reduction by the holding period in full years
/// (`None`: unknown, then there is no surcharge or reduction).
/// `multiplier` is the tax multiplier; it only has an effect if the rules set `use_multiplier`.
pub fn property_gain_tax_held(def: Option<&PropertyGainTax>, gain: f64, years: Option<i32>, multiplier: f64) -> f64 {
    property_gain_tax_full(def, gain, 0.0, years.map(|y| y * 12), multiplier)
}

/// Value of the last row `[from years, value]` whose years are reached.
fn by_duration(rows: &[(f64, f64)], years: f64) -> Option<f64> {
    rows.iter()
        .filter(|&&(from, _)| from <= years + 1e-9)
        .max_by(|a, b| a.0.total_cmp(&b.0))
        .map(|&(_, v)| v)
}

/// Tax on `gain` by brackets `[width, rate]` in the entered order; above the last bracket its rate applies.
fn by_brackets(brackets: &[(f64, f64)], gain: f64) -> f64 {
    let mut rest = gain;
    let mut tax = 0.0;
    for &(width, rate) in brackets {
        let part = rest.min(width.max(0.0));
        tax += rate * part;
        rest -= part;
        if rest <= 0.0 {
            break;
        }
    }
    if let Some(&(_, last)) = brackets.last() {
        tax += last * rest.max(0.0);
    }
    tax
}

/// #65: Property gain tax with the holding period in months (`None`: unknown) and the investment costs `cost` (for
/// the model «return»). Order: exemption limit, allowance, change of the gain by holding period, tax by the model,
/// surcharge above a threshold for a short holding period, surcharge or reduction by holding period, tax multiplier
/// (if `use_multiplier`), fixed reduction, maximum burden. The years of the tables are compared with the holding
/// period in years (months / 12). Without a purchase date there is no surcharge or reduction; the model «aargau»
/// then takes the highest rate, the model «return» the highest rate of `return_bounds`.
pub fn property_gain_tax_full(
    def: Option<&PropertyGainTax>,
    gain: f64,
    cost: f64,
    months: Option<i32>,
    multiplier: f64,
) -> f64 {
    let Some(def) = def else { return 0.0 };
    if gain <= 0.0 || gain < def.exempt_below {
        return 0.0;
    }
    let years = months.map(|m| f64::from(m.max(0)) / 12.0);
    let factor = if def.use_multiplier { multiplier } else { 1.0 };
    let mut g = gain - def.allowance;
    if g <= 0.0 {
        return 0.0;
    }
    if let Some(y) = years {
        g *= 1.0 + by_duration(&def.gain_duration, y).unwrap_or(0.0);
    }
    if g <= 0.0 {
        return 0.0;
    }
    let tax = match def.model {
        GainModel::Aargau => {
            // Direct rate by holding period; without a purchase date the highest rate.
            let rate = match years {
                Some(y) => by_duration(&def.duration_rates, y),
                None => def.duration_rates.iter().map(|&(_, r)| r).max_by(f64::total_cmp),
            }
            .unwrap_or(0.0);
            g * rate
        }
        GainModel::Curve => g * rate_at(&def.rate_curve, g),
        GainModel::Return => {
            let (lo, mut hi) = def.return_bounds.unwrap_or((0.0, 1.0));
            // Holding period: up to 5 years in started months, above in started years.
            let held = months.map(|m| {
                let m = f64::from(m.max(1));
                if m <= 60.0 {
                    m / 12.0
                } else {
                    (m / 12.0).ceil()
                }
            });
            if let Some(h) = held {
                hi = by_duration(&def.return_max_duration, h).unwrap_or(hi);
            }
            // Without investment costs the return is infinite: highest rate.
            let rate = match held {
                Some(h) => (gain / cost.max(0.0) / h).clamp(lo, hi.max(lo)),
                None => hi,
            };
            g * rate
        }
        GainModel::Zurich => match def.uniform_above {
            Some((from, rate)) if g > from => g * rate,
            _ => match (&def.long_brackets, years) {
                (Some((from, long)), Some(y)) if y >= *from => by_brackets(long, g),
                _ => by_brackets(&def.brackets, g),
            },
        },
    };
    // The model «aargau» has its rates by holding period already (own rules keep their `duration` rows unused).
    let adjust = match years {
        Some(y) if def.model != GainModel::Aargau => {
            let low = by_duration(&def.duration, y).unwrap_or(0.0);
            match &def.duration_above {
                // Shares of the tax on the gain up to the threshold and above it
                // (all of it up to the threshold if the gain does not exceed it)
                Some((limit, rows)) => {
                    let w = (limit / g).min(1.0);
                    low * w + by_duration(rows, y).unwrap_or(0.0) * (1.0 - w)
                }
                _ => low,
            }
        }
        _ => 0.0,
    };
    // Surcharge on the part of the gain above the threshold for a short holding period (at the average rate).
    let extra = match (def.short_surcharge, years) {
        (Some((from, add, max_years)), Some(y)) if y < max_years => (g - from).max(0.0) * (tax / g) * add,
        _ => 0.0,
    };
    let mut tax = (tax + extra) * (1.0 + adjust) * factor * (1.0 - def.reduction);
    if def.max_share > 0.0 {
        tax = tax.min(gain * def.max_share);
    }
    // #134: never negative, even with negative rates or a reduction above 100 %
    round2(tax).max(0.0)
}

pub fn round2(v: f64) -> f64 {
    (v * 100.0).round_ties_even() / 100.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn interpolation() {
        let t = [(0.0, 0.0), (50_000.0, 0.04), (100_000.0, 0.06)];
        assert_eq!(rate_at(&t, -5.0), 0.0);
        assert!((rate_at(&t, 75_000.0) - 0.05).abs() < 1e-12);
        assert_eq!(rate_at(&t, 1e9), 0.06);
        assert!((tax(&t, 75_000.0, 2.0) - 7_500.0).abs() < 1e-9);
    }

    #[test]
    fn gain_tax_brackets() {
        let def = PropertyGainTax {
            brackets: vec![(4000.0, 0.1), (6000.0, 0.15)],
            reduction: 0.5,
            ..Default::default()
        };
        // 4000 × 10 % + 1000 × 15 % = 550, halved = 275
        assert!((property_gain_tax(Some(&def), 5000.0) - 275.0).abs() < 1e-9);
    }

    #[test]
    fn aargau_rate_with_multiplier_and_reduction() {
        let def = PropertyGainTax {
            model: GainModel::Aargau,
            duration_rates: vec![(0.0, 0.3), (5.0, 0.2)],
            use_multiplier: true,
            reduction: 0.1,
            ..Default::default()
        };
        // 10 000 × 20 % (from 5 years) × tax multiplier 1.5 × (1 − 10 %) = 2700
        assert!((property_gain_tax_held(Some(&def), 10_000.0, Some(7), 1.5) - 2700.0).abs() < 1e-9);
        // 10 000 × 30 % × 1.5 × 0.9 = 4050
        assert!((property_gain_tax_held(Some(&def), 10_000.0, Some(2), 1.5) - 4050.0).abs() < 1e-9);
    }
}
