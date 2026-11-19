//! Housing share of the living costs (#53).
//!
//! The entered living costs include rent or the monthly property costs. For the simulation the
//! housing share contained in them is deducted and replaced by the simulated housing costs. The housing share is the
//! housing costs in the start month as a monthly average: rent, fix costs, mortgage interest (debt × rate / 12) and
//! amortization (quarterly amount / 3), direct and indirect. SARON is the plan's fixed value in the start month.

use crate::model::{AfterLast, Plan, PropertyStatus, RateKind};
use crate::resolve::{Curve, Resolver};
use crate::time::YearMonth;

/// Housing costs in the start month in prices of the start month (0 without property and rent).
pub fn start_housing(plan: &Plan, r: &Resolver) -> f64 {
    let Ok(start) = r.milestone("start") else { return 0.0 };
    let saron = Curve::compile(&plan.returns.fixed.saron, r)
        .map_or(0.0, |c| c.at(start))
        .max(0.0);
    let mut sum = 0.0;
    for p in &plan.realestate.properties {
        let resolve = |t: &Option<crate::time::TimeRef>| t.as_ref().and_then(|t| r.resolve(t).ok().flatten());
        match p.status {
            PropertyStatus::Rent => {
                // Rent «nach Verkauf» is not yet known at the start and does not count.
                let from = resolve(&p.from);
                let until = resolve(&p.until);
                if from.is_some_and(|f| f <= start) && until.is_none_or(|u| start < u) {
                    sum += p.rent.unwrap_or(0.0);
                }
            }
            PropertyStatus::Owned | PropertyStatus::Planned => {
                let first = match p.status {
                    PropertyStatus::Owned => None,
                    _ => match resolve(&p.buy) {
                        Some(b) if b == start => Some(b),
                        _ => continue,
                    },
                };
                if p.status == PropertyStatus::Owned && resolve(&p.sell).is_some_and(|s| s <= start) {
                    continue;
                }
                sum += p.fix_costs;
                let Some(mg) = &p.mortgage else { continue };
                for t in &mg.tranches {
                    let Some(mut m) = first.or(t.start) else { continue };
                    let mut i = 0usize;
                    // Period that contains the start month
                    while !t.periods.is_empty() && i < 1000 {
                        if i >= t.periods.len() && t.after == AfterLast::Repay {
                            break;
                        }
                        let d = &t.periods[i.min(t.periods.len() - 1)];
                        let end: YearMonth = m.plus(((d.term * 12.0).round() as i32).max(1));
                        if m <= start && start < end {
                            let rate = match d.kind {
                                RateKind::Fixed => d.rate,
                                RateKind::Saron => saron + d.margin,
                            };
                            sum += t.amount * rate / 12.0 + d.amort / 3.0;
                            break;
                        }
                        if m > start {
                            break;
                        }
                        m = end;
                        i += 1;
                    }
                }
            }
        }
    }
    sum
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn plan(props: serde_json::Value) -> Plan {
        serde_json::from_value(json!({
            "project": { "name": "W" },
            "household": { "persons": [ { "id": "A", "name": "A", "birth": "1980-01" } ] },
            "milestones": [ { "id": "start", "name": "S", "kind": "date", "date": "2040-01" },
                            { "id": "end", "name": "E", "kind": "date", "date": "2050-12" } ],
            "pension": { "pk": { "capital": 0, "at": "retire", "lumpsum": 0, "conversion": 0 }, "ahv": { "monthly": 0, "at": "ahv" } },
            "flows": { "spending": 8000 },
            "wealth": { "stocks": 0, "bonds": 0, "cash": 0, "stockShare": 0 },
            "returns": { "source": "fixed", "fixed": { "stocks": 0, "bonds": 0, "inflation": 0, "saron": 0.01 } },
            "taxes": { "income": [], "wealth": [], "capital": [] },
            "realestate": { "properties": props }
        }))
        .unwrap()
    }
    fn w(props: serde_json::Value) -> f64 {
        let p = plan(props);
        start_housing(&p, &Resolver::new(&p).unwrap())
    }

    // #53/AK2, AK3: interest 1'000, amortization 500, fix costs 500 = housing share 2'000.
    #[test]
    fn issue_53_owned_property() {
        let house = json!([{ "name": "H", "status": "owned", "price": 1, "fixCosts": 500, "sell": "never",
            "mortgage": { "tranches": [ { "amount": 600_000, "start": "2035-01",
                "periods": [ { "type": "fixed", "term": 10, "rate": 0.02, "amort": 1500 } ] } ] } }]);
        assert!((w(house) - 2000.0).abs() < 1e-9);
        // SARON with margin: 600'000 × (1 % + 1 %) / 12 = 1'000
        let saron = json!([{ "name": "H", "status": "owned", "price": 1, "fixCosts": 0,
            "mortgage": { "tranches": [ { "amount": 600_000, "start": "2039-01",
                "periods": [ { "type": "saron", "term": 5, "margin": 0.01 } ] } ] } }]);
        assert!((w(saron) - 1000.0).abs() < 1e-9);
        // The second period contains the start; an expired tranche with repayment does not count
        let later = json!([{ "name": "H", "status": "owned", "price": 1, "fixCosts": 0,
            "mortgage": { "tranches": [
                { "amount": 120_000, "start": "2030-01", "periods": [ { "term": 5, "rate": 0.05 }, { "term": 10, "rate": 0.01 } ] },
                { "amount": 120_000, "start": "2030-01", "after": "repay", "periods": [ { "term": 5, "rate": 0.05 } ] } ] } }]);
        assert!((w(later) - 100.0).abs() < 1e-9);
        // Sold before the start or in the start month: no housing costs
        let sold = json!([{ "name": "H", "status": "owned", "price": 1, "fixCosts": 500, "sell": "2040-01" }]);
        assert_eq!(w(sold), 0.0);
    }

    // #53/AK5: Rent from the start counts; later rent, rent «nach Verkauf» and a later planned purchase do not.
    #[test]
    fn issue_53_rent_and_planned() {
        assert_eq!(
            w(json!([{ "name": "M", "status": "rent", "from": "start", "rent": 2500 }])),
            2500.0
        );
        assert_eq!(
            w(json!([{ "name": "M", "status": "rent", "from": "2041-01", "rent": 2500 }])),
            0.0
        );
        assert_eq!(
            w(json!([{ "name": "M", "status": "rent", "from": "afterSale", "rent": 2500 }])),
            0.0
        );
        assert_eq!(
            w(json!([{ "name": "M", "status": "rent", "from": "2030-01", "until": "2040-01", "rent": 2500 }])),
            0.0
        );
        assert_eq!(
            w(json!([{ "name": "P", "status": "planned", "buy": "2041-01", "price": 1, "fixCosts": 400 }])),
            0.0
        );
        assert_eq!(
            w(json!([{ "name": "P", "status": "planned", "buy": "start", "price": 1, "fixCosts": 400 }])),
            400.0
        );
        assert_eq!(w(json!([])), 0.0);
    }

    // #124/AK8: Limits of the periods and the rent. A period ending exactly in the start month no longer counts; a
    // tranche «zurückzahlen» counts as long as its period runs; a tranche without periods contributes nothing; a rent
    // until after the start counts.
    #[test]
    fn issue_124_ak8_period_and_rent_limits() {
        let tranche = |t: serde_json::Value| json!([{ "name": "H", "status": "owned", "price": 1, "fixCosts": 0, "mortgage": { "tranches": [t] } }]);
        // 120'000 × 1 % / 12 = 100 (second period from 2040-01), not 120'000 × 5 % / 12 = 500
        let ends_at_start = tranche(
            json!({ "amount": 120_000, "start": "2030-01", "periods": [ { "term": 10, "rate": 0.05 }, { "term": 10, "rate": 0.01 } ] }),
        );
        assert!((w(ends_at_start) - 100.0).abs() < 1e-9);
        let repay_running = tranche(
            json!({ "amount": 120_000, "start": "2035-01", "after": "repay", "periods": [ { "term": 10, "rate": 0.01 } ] }),
        );
        assert!((w(repay_running) - 100.0).abs() < 1e-9);
        let no_periods = tranche(json!({ "amount": 120_000, "start": "2035-01", "after": "renew", "periods": [] }));
        assert_eq!(w(no_periods), 0.0);
        assert_eq!(
            w(json!([{ "name": "M", "status": "rent", "from": "2030-01", "until": "2041-01", "rent": 2500 }])),
            2500.0
        );
    }
}
