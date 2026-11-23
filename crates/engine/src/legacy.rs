//! Import of a configuration of the previous Python program (`config.json`, age-based).
//!
//! Ages are converted to calendar months via the start age and the start month; this needs
//! the calendar year of the simulation start. Deviations from the old model (mainly properties)
//! are reported in `notes`.

use std::path::Path;

use serde_json::{Map, Value};

use crate::model::*;
use crate::time::{TimeRef, YearMonth};
use crate::{Error, Result};

pub struct LegacyImport {
    pub plan: Plan,
    pub notes: Vec<crate::message::Message>,
}

struct Ctx<'a> {
    root: &'a Value,
    start: YearMonth,
    start_age: f64,
    end: YearMonth,
    notes: Vec<crate::message::Message>,
}

fn node<'a>(root: &'a Value, path: &str) -> Option<&'a Value> {
    path.split('.').try_fold(root, |v, k| v.get(k))
}

fn num(v: &Value) -> Option<f64> {
    match v {
        Value::Number(n) => n.as_f64(),
        Value::String(s) => s.trim().parse().ok(),
        Value::Bool(b) => Some(if *b { 1.0 } else { 0.0 }),
        _ => None,
    }
}

fn truthy(v: Option<&Value>, default: bool) -> bool {
    match v {
        Some(Value::Bool(b)) => *b,
        Some(Value::String(s)) => s.eq_ignore_ascii_case("true"),
        _ => default,
    }
}

impl<'a> Ctx<'a> {
    fn get(&self, path: &str) -> Option<&'a Value> {
        node(self.root, path)
    }
    fn f(&self, path: &str) -> Option<f64> {
        self.get(path).and_then(num)
    }
    fn f_or(&self, path: &str, default: f64) -> f64 {
        self.f(path).unwrap_or(default)
    }

    /// Replaces `{path}` in keys by the value at that path (like `replace_variables`).
    fn key_age(&self, key: &str) -> Result<f64> {
        let k = key.trim();
        let resolved = if k.starts_with('{') && k.ends_with('}') {
            self.f(&k[1..k.len() - 1])
                .ok_or_else(|| Error::Invalid(msg!(legacy_variable_undefined, name = k)))?
        } else {
            k.parse()
                .map_err(|_| Error::Invalid(msg!(legacy_not_an_age, value = k)))?
        };
        Ok(resolved)
    }

    /// Like `age2months` in the old program (Python rounds half months to the even number).
    fn month_of_age(&self, age: f64) -> YearMonth {
        self.start
            .plus(((age - self.start_age) * 12.0).round_ties_even() as i32)
    }

    fn series(&self, path: &str, map: impl Fn(f64) -> f64) -> Result<Option<Series>> {
        let Some(v) = self.get(path) else {
            return Ok(None);
        };
        match v {
            Value::Object(o) => {
                let mut rows = vec![];
                for (k, val) in o {
                    let m = self.month_of_age(self.key_age(k)?);
                    let x = num(val).ok_or_else(|| {
                        Error::Invalid(msg!(legacy_not_a_number, path = path, value = val.to_string()))
                    })?;
                    rows.push((TimeRef::date(m), map(x)));
                }
                rows.sort_by_key(|r| r.0.as_date());
                Ok(Some(Series::timed(rows)))
            }
            other => Ok(num(other).map(|x| Series::Number(map(x)))),
        }
    }

    fn series_or_zero(&self, path: &str) -> Result<Series> {
        Ok(self.series(path, |x| x)?.unwrap_or_default())
    }

    fn table(&self, path: &str) -> Vec<(f64, f64)> {
        let mut t: Vec<(f64, f64)> = self
            .get(path)
            .and_then(Value::as_object)
            .map(|o| {
                o.iter()
                    .filter_map(|(k, v)| Some((k.trim().parse().ok()?, num(v)?)))
                    .collect()
            })
            .unwrap_or_default();
        t.sort_by(|a, b| a.0.total_cmp(&b.0));
        t
    }

    fn age_ref(&self, v: Option<&Value>) -> Option<TimeRef> {
        let age = v.and_then(num)?;
        let m = self.month_of_age(age);
        if m > self.end {
            Some(TimeRef::never())
        } else {
            Some(TimeRef::date(m))
        }
    }
}

/// `start_year`: calendar year of the simulation start (the old format only knows the start month).
/// `base_dir`: directory against which relative paths (e.g. `./data/MSCI.csv`) are resolved.
pub fn import(text: &str, start_year: i32, base_dir: &Path, name: &str) -> Result<LegacyImport> {
    let root: Value =
        serde_json::from_str(text).map_err(|e| Error::Invalid(msg!(legacy_unreadable, detail = e.to_string())))?;
    if node(&root, "General").is_none() {
        return Err(Error::Invalid(msg!(legacy_general_missing)));
    }
    let start_age = node(&root, "General.StartAge").and_then(num).unwrap_or(50.0);
    let start_month = node(&root, "General.StartMonth").and_then(num).unwrap_or(1.0) as u32;
    let start = YearMonth::new(start_year, start_month);
    let mut c = Ctx {
        root: &root,
        start,
        start_age,
        end: start,
        notes: vec![],
    };
    let end_age = c.f_or("General.EndAge", start_age + 30.0);
    c.end = c.month_of_age(end_age);
    let early = c.f_or("Pension.EarlyRetirement", 65.0);
    let legal = c.f_or("Pension.LegalRetirement", 65.0);

    let milestones = vec![
        Milestone {
            id: "start".into(),
            name: String::new(), // #170: fixed milestone, name by id
            kind: MilestoneKind::Date { date: start },
            fixed: true,
        },
        Milestone {
            id: "retire".into(),
            name: String::new(), // #170: fixed milestone, name by id
            kind: MilestoneKind::Date {
                date: c.month_of_age(early),
            },
            fixed: false,
        },
        Milestone {
            id: "ahv".into(),
            name: String::new(), // #170: fixed milestone, name by id
            kind: MilestoneKind::Date {
                date: c.month_of_age(legal),
            },
            fixed: false,
        },
        Milestone {
            id: "end".into(),
            name: String::new(), // #170: fixed milestone, name by id
            kind: MilestoneKind::Date { date: c.end },
            fixed: true,
        },
    ];

    // Pension fund
    let lumpsum = match c.get("Pension.Private.LumpsumRatio") {
        Some(Value::Object(o)) => {
            if o.len() > 1 {
                c.notes.push(msg!(legacy_lumpsums_merged));
            }
            o.values().filter_map(num).sum()
        }
        Some(v) => num(v).unwrap_or(0.0),
        None => 0.0,
    };
    let pension = Pension {
        pk: PensionFund {
            capital: c.f_or("Pension.Private.Capital", 0.0),
            at: TimeRef::milestone("retire"),
            lumpsum,
            conversion: c.f_or("Pension.Private.ConversionRate", 0.0),
            contribution: c.series_or_zero("Pension.Private.Contribution")?,
            interest: c.series_or_zero("Pension.Private.Interest")?,
        },
        ahv: Ahv {
            monthly: c.f_or("Pension.Legal", 0.0),
            at: TimeRef::milestone("ahv"),
            indexed: true,
            thirteenth_noted: false,
            reference_age: false,
        },
    };

    // Cash flows
    let mut extras = vec![];
    match c.get("MoneyFlows.Extra") {
        Some(Value::Object(o)) => {
            for (k, v) in o {
                let amount = num(v).unwrap_or(0.0);
                if amount != 0.0 {
                    let m = c.month_of_age(c.key_age(k)?).max(start);
                    extras.push(Extra {
                        date: m,
                        amount,
                        label: "Einmaliger Betrag".into(),
                    });
                }
            }
        }
        Some(v) => {
            if let Some(a) = num(v).filter(|a| *a != 0.0) {
                extras.push(Extra {
                    date: start,
                    amount: a,
                    label: "Einmaliger Betrag".into(),
                });
            }
        }
        None => {}
    }
    let flows = Flows {
        income: c.series_or_zero("MoneyFlows.Income")?,
        // In the old program the income stayed the same in nominal terms (#58).
        income_indexed: false,
        spending: c.series_or_zero("MoneyFlows.Spendings")?,
        savings: c.series("MoneyFlows.Savings", |x| x)?,
        extras,
        // In the old program spending was without housing costs; the takeover on opening converts it (#53).
        spending_includes_housing: false,
        spending_factor: None,
    };

    // Wealth
    let stocks = c.f_or("WealthManagement.Stocks", 0.0);
    let bonds = c.f_or("WealthManagement.Bonds", 0.0);
    // Without a value from the start, the old program used today's ratio of stocks to bonds.
    let initial_share = if stocks + bonds > 0.0 {
        stocks / (stocks + bonds)
    } else {
        0.5
    };
    let stock_share = match c.series("WealthManagement.PortfolioBalance", |x| x)? {
        Some(Series::Def(SeriesDef::Timed { mut rows })) => {
            if rows.first().and_then(|r| r.from.as_date()).is_some_and(|m| m > start) {
                rows.insert(
                    0,
                    SeriesRow {
                        from: TimeRef::date(start),
                        value: initial_share,
                    },
                );
            }
            Series::Def(SeriesDef::Timed { rows })
        }
        Some(s) => s,
        None => Series::Number(initial_share),
    };
    let wealth = Wealth {
        stocks,
        bonds,
        cash: c.f_or("WealthManagement.Cash", 0.0),
        stock_share,
        reserve_months: c.series_or_zero("WealthManagement.CashThresholdMonths")?,
    };

    // Returns
    let yearly = truthy(c.get("WealthManagement.YearlyPerformance"), true);
    let to_yearly = move |x: f64| if yearly { x } else { (1.0 + x).powi(12) - 1.0 };
    let fixed = FixedReturns {
        stocks: c
            .series("WealthManagement.StockPerformance", to_yearly)?
            .unwrap_or_default(),
        bonds: c
            .series("WealthManagement.BondPerformance", to_yearly)?
            .unwrap_or_default(),
        inflation: c.series("WealthManagement.Inflation", |x| x)?.unwrap_or_default(),
        saron: Series::Number(0.0),
    };
    // «----» (no historical year) is not a number and therefore also gives `None`.
    let hist_year = c.get("WealthManagement.HistoricalYear").and_then(num);
    let extrapolation = Extrapolation {
        method: match c.get("WealthManagement.Extrapolation").and_then(Value::as_str) {
            Some("Average") => ExtrapolationMethod::Average,
            Some("Weighted Average") => ExtrapolationMethod::Geometric,
            _ => ExtrapolationMethod::Fixed,
        },
        range: c.f_or("WealthManagement.AverageRange", 30.0) as usize,
    };
    let parametric = c.get("MonteCarlo").map(|_| ParametricParams {
        paths: c.f_or("MonteCarlo.Iterations", 10_000.0) as usize,
        mu: c.f_or("MonteCarlo.AnnualPerformance", 0.105),
        sigma: c.f_or("MonteCarlo.AnnualVolatility", 0.148),
        df: c.f_or("MonteCarlo.StudentFatTails", 6.0),
        jump_lambda: c.f_or("MonteCarlo.JumpIntensity", 0.05),
        jump_mu: c.f_or("MonteCarlo.JumpSize", -0.3),
        jump_sigma: c.f_or("MonteCarlo.JumpVolatility", 0.15),
        ..Default::default()
    });
    let mut returns = match hist_year {
        None => Returns {
            source: ReturnSource::Fixed,
            fixed,
            strategy: None,
            series: None,
            saron_series: None,
            start_date: None,
            extrapolation,
            bootstrap: None,
            parametric: parametric.clone(),
        },
        Some(y) => {
            let file = c
                .get("WealthManagement.HistoricalData")
                .and_then(Value::as_str)
                .unwrap_or("./data/MSCI.csv");
            let full = base_dir.join(file);
            let full = full.canonicalize().unwrap_or(full);
            Returns {
                source: ReturnSource::Historical,
                fixed,
                strategy: None,
                series: Some(full.to_string_lossy().into_owned()),
                saron_series: None,
                start_date: Some(YearMonth::new(y as i32, start_month)),
                extrapolation,
                bootstrap: None,
                parametric: parametric.clone(),
            }
        }
    };
    match c.get("CalculationMode").and_then(Value::as_str) {
        Some(m) if m.eq_ignore_ascii_case("montecarlo") => {
            returns.source = ReturnSource::Parametric;
            c.notes.push(msg!(legacy_monte_carlo));
        }
        Some(m) if !m.eq_ignore_ascii_case("single") => {
            c.notes.push(msg!(legacy_iteration));
        }
        _ => {}
    }

    // Taxes
    let sales = c.table("Taxes.Sales");
    let taxes = Taxes {
        multiplier: c.f_or("Taxes.TaxRate", 2.0),
        income: c.table("Taxes.Income"),
        wealth: c.table("Taxes.Capital"),
        capital: c.table("Taxes.PensionCapital"),
        property_gain: (!sales.is_empty()).then(|| PropertyGainTax {
            brackets: sales,
            reduction: c.f_or("Taxes.SalesTaxReduction", 0.0),
            ..Default::default()
        }),
        couple: None,
        indexed: false,
        mode: TaxMode::Own,
        location: None,
        relocations: vec![],
        property_gain_own: false,
        deduct_pillar3a: false,
        deductions: None,
    };

    // Properties (approximately)
    let props_json: Vec<Map<String, Value>> = c
        .get("RealEstate.Properties")
        .and_then(Value::as_array)
        .map(|a| a.iter().filter_map(|v| v.as_object().cloned()).collect())
        .unwrap_or_default();
    let mut properties = vec![];
    let buy_after_sell = truthy(c.get("RealEstate.BuyAfterSell"), true);
    for p in &props_json {
        let s = |k: &str| p.get(k);
        let name = s("Name").and_then(Value::as_str).unwrap_or("Objekt").to_string();
        let price = s("Price").and_then(num).unwrap_or(0.0);
        let fix_costs = s("FixCosts").and_then(num).unwrap_or(price * 0.01 / 12.0);
        let inflation = truthy(s("InflationCorrection"), false);
        let status = s("Status").and_then(Value::as_str).unwrap_or("Planned");
        let mortgage = |owned: bool| -> Option<Mortgage> {
            let m = s("Mortage")?.as_object()?;
            let g = |k: &str| m.get(k).and_then(num);
            let amort = g("Amortization").unwrap_or(0.0);
            Some(Mortgage {
                amount_mode: AmountMode::Fixed,
                tranches: vec![Tranche {
                    name: "Tranche 1".into(),
                    amount: g("Value").unwrap_or(0.0),
                    start: owned.then(|| g("StartAge").map_or(start, |a| c.month_of_age(a))),
                    after: AfterLast::Renew,
                    periods: vec![Period {
                        kind: RateKind::Fixed,
                        term: g("Term").unwrap_or(10.0),
                        rate: g("Interest").unwrap_or(0.05),
                        margin: 0.0,
                        amort: amort * 3.0 / 4.0,
                        amort_mode: AmortMode::Direct,
                    }],
                    pillar3a: None,
                }],
                pillar3a: None,
            })
        };
        // Without a purchase age the old program bought or rented after the sale of the last property.
        let after_sale = || {
            if buy_after_sell {
                TimeRef::after_sale()
            } else {
                TimeRef::never()
            }
        };
        let buy = c.age_ref(s("BuyAge")).unwrap_or_else(after_sale);
        let prop = match status {
            "Owned" => Property {
                id: None,
                name,
                status: PropertyStatus::Owned,
                price,
                worth: s("Worth").and_then(num),
                buy: None,
                bought: None,
                canton: None,
                sell: Some(c.age_ref(s("SellAge")).unwrap_or_else(TimeRef::never)),
                from: None,
                until: None,
                rent: None,
                fix_costs,
                inflation,
                inflation_value: None,
                inflation_fix_costs: None,
                inflation_rent: None,
                mortgage: mortgage(true),
                tax_value: None,
            },
            "Planned" => Property {
                id: None,
                name,
                status: PropertyStatus::Planned,
                price,
                worth: None,
                buy: Some(buy),
                bought: None,
                canton: None,
                sell: None,
                from: None,
                until: None,
                rent: None,
                fix_costs,
                inflation,
                inflation_value: None,
                inflation_fix_costs: None,
                inflation_rent: None,
                mortgage: mortgage(false),
                tax_value: None,
            },
            "PlannedForRent" => Property {
                id: None,
                name,
                status: PropertyStatus::Rent,
                price: 0.0,
                worth: None,
                buy: None,
                bought: None,
                canton: None,
                sell: None,
                from: Some(c.age_ref(s("BuyAge")).unwrap_or_else(TimeRef::after_sale)),
                until: None,
                rent: Some(-s("RentalIncome").and_then(num).unwrap_or(0.0)),
                fix_costs: 0.0,
                inflation,
                inflation_value: None,
                inflation_fix_costs: None,
                inflation_rent: None,
                mortgage: None,
                tax_value: None,
            },
            other => {
                c.notes.push(msg!(legacy_property_ignored, name = name, status = other));
                continue;
            }
        };
        properties.push(prop);
    }
    if !properties.is_empty() {
        c.notes.push(msg!(legacy_properties_approximate));
    }
    let d = Affordability::default();
    let realestate = RealEstate {
        never_sell: false,
        emergency_months: c.series_or_zero("RealEstate.ThresholdMonths")?,
        properties,
        affordability: Affordability {
            sustainability: c.f_or("RealEstate.Affordability.Sustainability", d.sustainability),
            imputed_rate: c.f_or("RealEstate.Affordability.MortageInterest", d.imputed_rate),
            fix_costs: c.f_or("RealEstate.Affordability.FixCosts", d.fix_costs),
            wealth_contribution: c.f_or("RealEstate.Affordability.CapitalContribution", d.wealth_contribution),
            max_loan_to_value: d.max_loan_to_value,
        },
    };

    // #161: negate as a floating-point number first, then clamp. `as i32` caps an extreme start age at `i32::MIN`;
    // the minus afterwards would overflow. This way it gives a very distant month like `YearMonth::plus` instead of a crash.
    let birth = start.plus((-start_age * 12.0).round_ties_even() as i32);
    let plan = Plan {
        format: FORMAT.into(),
        project: Project {
            name: name.into(),
            currency: "CHF".into(),
        },
        household: Household {
            persons: vec![Person {
                id: "A".into(),
                name: "Person A".into(),
                birth,
                no_pension_fund: false,
            }],
            married: true,
            ahv_couple_cap: 3780.0,
        },
        milestones,
        pension,
        partner: None,
        flows,
        wealth,
        returns,
        withdrawal: Withdrawal::default(),
        taxes,
        realestate,
        scenarios: Scenarios::default(),
        optimization: None,
        pillar3a: vec![],
        limits: Default::default(),
    };
    Ok(LegacyImport { plan, notes: c.notes })
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn ym(y: i32, m: u32) -> YearMonth {
        YearMonth::new(y, m)
    }
    fn imp(cfg: Value) -> LegacyImport {
        import(&cfg.to_string(), 2030, Path::new("."), "Test").unwrap()
    }
    fn js<T: serde::Serialize>(x: &T) -> Value {
        serde_json::to_value(x).unwrap()
    }

    // #40: Conversion of a made-up configuration of the previous program, field by field.
    // Start January 2030 at age 60 ⇒ age a corresponds to 2030-01 + (a − 60) × 12 months.
    #[test]
    fn full_config() {
        let cfg = json!({
            "CalculationMode": "MonteCarlo",
            "General": {"StartAge": 60, "StartMonth": 1, "EndAge": 70},
            "Pension": {
                "EarlyRetirement": 62.5, "LegalRetirement": 65,
                "Legal": 2000,
                "Private": {"Capital": 500000, "LumpsumRatio": {"62": 0.25, "63": 0.25}, "ConversionRate": 0.05,
                            "Contribution": {"60": 1000, "{Pension.EarlyRetirement}": 0}, "Interest": 0.01}
            },
            "MoneyFlows": {"Income": {"60": 8000, "62.5": 0}, "Spendings": 6000, "Savings": 500,
                           "Extra": {"61": 10000, "64": 0, "55": -5000}},
            "WealthManagement": {
                "Stocks": 300000, "Bonds": 100000, "Cash": true,
                "PortfolioBalance": {"65": 0.4}, "CashThresholdMonths": 6,
                "YearlyPerformance": false, "StockPerformance": 0.01, "BondPerformance": {"60": 0.002},
                "Inflation": 0.015, "HistoricalYear": "----",
                "Extrapolation": "Average", "AverageRange": 20
            },
            "MonteCarlo": {"Iterations": 500, "AnnualPerformance": 0.08, "AnnualVolatility": 0.2,
                           "StudentFatTails": 5, "JumpIntensity": 0.1, "JumpSize": -0.2, "JumpVolatility": 0.25},
            "Taxes": {"TaxRate": 1.5, "Income": {"100000": 0.1, "0": 0}, "Capital": {"0": 0.001},
                      "PensionCapital": {"0": 0.02}, "Sales": {"0": 0.3, "50000": 0.2}, "SalesTaxReduction": 0.1},
            "RealEstate": {
                "ThresholdMonths": 3,
                "Properties": [
                    {"Name": "Haus", "Status": "Owned", "Price": 1200000, "Worth": 1500000, "SellAge": 70,
                     "InflationCorrection": "True",
                     "Mortage": {"Value": 600000, "Interest": 0.02, "Term": 5, "Amortization": 4000, "StartAge": 58}},
                    {"Name": "Wohnung", "Status": "Planned", "Price": 600000, "BuyAge": 71},
                    {"Name": "Miete", "Status": "PlannedForRent", "RentalIncome": -2500, "BuyAge": 66},
                    {"Name": "Zelt", "Status": "Camping"}
                ]
            }
        });
        let r = imp(cfg);
        let p = &r.plan;
        let date = |id: &str| match &p.milestones.iter().find(|m| m.id == id).unwrap().kind {
            MilestoneKind::Date { date } => *date,
            k => panic!("{k:?}"),
        };
        assert_eq!(date("start"), ym(2030, 1));
        assert_eq!(date("retire"), ym(2032, 7));
        assert_eq!(date("ahv"), ym(2035, 1));
        assert_eq!(date("end"), ym(2040, 1));
        assert_eq!(p.household.persons[0].birth, ym(1970, 1));

        // Pension fund: two lump sums ⇒ one payout of 50 % with a note
        assert_eq!(p.pension.pk.lumpsum, 0.5);
        assert!(r.notes.iter().any(|n| n.contains("Multiple lump-sum withdrawals")));
        assert_eq!(
            js(&p.pension.pk.contribution),
            json!({"mode": "timed", "rows": [{"from": "2030-01", "value": 1000.0}, {"from": "2032-07", "value": 0.0}]})
        );
        assert_eq!(js(&p.pension.pk.interest), json!(0.01));

        // Cash flows: amounts of 0 are dropped, dates before the start apply from the start
        assert_eq!(
            js(&p.flows.income),
            json!({"mode": "timed", "rows": [{"from": "2030-01", "value": 8000.0}, {"from": "2032-07", "value": 0.0}]})
        );
        assert_eq!(js(&p.flows.spending), json!(6000.0));
        assert_eq!(js(&p.flows.savings), json!(500.0));
        let extras: Vec<_> = p.flows.extras.iter().map(|e| (e.date, e.amount)).collect();
        assert_eq!(extras, vec![(ym(2030, 1), -5000.0), (ym(2031, 1), 10000.0)]);

        // Wealth: stock share before the first entry = today's ratio 300/(300+100)
        assert_eq!(p.wealth.cash, 1.0);
        assert_eq!(
            js(&p.wealth.stock_share),
            json!({"mode": "timed", "rows": [{"from": "2030-01", "value": 0.75}, {"from": "2035-01", "value": 0.4}]})
        );
        assert_eq!(js(&p.wealth.reserve_months), json!(6.0));

        // Returns: monthly values converted to annual values, inflation unchanged
        let stocks = js(&p.returns.fixed.stocks).as_f64().unwrap();
        assert!((stocks - (1.01f64.powi(12) - 1.0)).abs() < 1e-12);
        let bonds = js(&p.returns.fixed.bonds)["rows"][0]["value"].as_f64().unwrap();
        assert!((bonds - (1.002f64.powi(12) - 1.0)).abs() < 1e-12);
        assert_eq!(js(&p.returns.fixed.inflation), json!(0.015));
        assert_eq!(p.returns.source, ReturnSource::Parametric);
        assert!(p.returns.series.is_none());
        assert_eq!(p.returns.extrapolation.method, ExtrapolationMethod::Average);
        assert_eq!(p.returns.extrapolation.range, 20);
        let q = p.returns.parametric.as_ref().unwrap();
        assert_eq!(
            (q.paths, q.mu, q.sigma, q.df, q.jump_lambda, q.jump_mu, q.jump_sigma),
            (500, 0.08, 0.2, 5.0, 0.1, -0.2, 0.25)
        );
        assert!(r.notes.iter().any(|n| n.contains("Monte Carlo imported")));

        // Taxes: tables sorted, property gain tax with reduction
        assert_eq!(p.taxes.multiplier, 1.5);
        assert_eq!(p.taxes.income, vec![(0.0, 0.0), (100000.0, 0.1)]);
        assert_eq!(p.taxes.wealth, vec![(0.0, 0.001)]);
        assert_eq!(p.taxes.capital, vec![(0.0, 0.02)]);
        let g = p.taxes.property_gain.as_ref().unwrap();
        assert_eq!(g.brackets, vec![(0.0, 0.3), (50000.0, 0.2)]);
        assert_eq!(g.reduction, 0.1);

        // Properties: unknown status ignored with a note
        assert_eq!(p.realestate.properties.len(), 3);
        assert!(r.notes.iter().any(|n| n.contains("“Zelt”")));
        assert!(r.notes.iter().any(|n| n.contains("approximately")));
        assert_eq!(js(&p.realestate.emergency_months), json!(3.0));
        let house = &p.realestate.properties[0];
        assert_eq!(house.status, PropertyStatus::Owned);
        assert_eq!(house.worth, Some(1_500_000.0));
        assert!(house.inflation);
        assert_eq!(house.fix_costs, 1_200_000.0 * 0.01 / 12.0);
        // Sale at 70 = end of simulation: still a date
        assert_eq!(house.sell, Some(TimeRef::date(ym(2040, 1))));
        let t = &house.mortgage.as_ref().unwrap().tranches[0];
        assert_eq!((t.amount, t.start), (600000.0, Some(ym(2028, 1))));
        assert_eq!(
            (t.periods[0].rate, t.periods[0].term, t.periods[0].amort),
            (0.02, 5.0, 3000.0)
        );
        // Purchase at 71 is after the end: never
        let flat = &p.realestate.properties[1];
        assert_eq!(flat.buy, Some(TimeRef::never()));
        assert!(!flat.inflation);
        assert_eq!(flat.mortgage.as_ref().map(|m| m.tranches.len()), None);
        let rental = &p.realestate.properties[2];
        assert_eq!(rental.status, PropertyStatus::Rent);
        assert_eq!(rental.rent, Some(2500.0));
        assert_eq!(rental.from, Some(TimeRef::date(ym(2036, 1))));
    }

    // Age keys: a reference only with both curly braces.
    #[test]
    fn age_keys_need_both_braces() {
        let root = json!({"Pension": {"EarlyRetirement": 63}});
        let c = Ctx {
            root: &root,
            start: ym(2030, 1),
            start_age: 50.0,
            end: ym(2060, 1),
            notes: vec![],
        };
        assert_eq!(c.key_age(" {Pension.EarlyRetirement} ").unwrap(), 63.0);
        assert_eq!(c.key_age("64.5").unwrap(), 64.5);
        // Only one brace: not a reference but an invalid age.
        for k in ["{Pension.EarlyRetirement", "Pension.EarlyRetirement}"] {
            assert!(c.key_age(k).unwrap_err().to_string().contains("is not an age"), "{k}");
        }
        assert!(c.key_age("{Fehlt}").unwrap_err().to_string().contains("not defined"));
    }

    // Defaults and other variants of the same data.
    #[test]
    fn defaults_and_variants() {
        let r = imp(json!({
            "CalculationMode": "Iteration",
            "General": {"StartAge": 50},
            "Pension": {"Private": {"LumpsumRatio": {"65": 0.3}}},
            "MoneyFlows": {"Extra": 7000},
            "WealthManagement": {"PortfolioBalance": {"50": 0.6}, "StockPerformance": 0.05,
                                 "YearlyPerformance": "false", "HistoricalYear": 1990,
                                 "HistoricalData": "x.csv", "Extrapolation": "Weighted Average"},
            "RealEstate": {"BuyAfterSell": false, "Properties": [{"Name": "Später", "Price": 400000}]}
        }));
        let p = &r.plan;
        // Without an end age: 30 years
        assert!(p
            .milestones
            .iter()
            .any(|m| m.id == "end" && matches!(m.kind, MilestoneKind::Date { date } if date == ym(2060, 1))));
        assert_eq!(p.pension.pk.lumpsum, 0.3);
        assert!(!r.notes.iter().any(|n| n.contains("Multiple lump-sum withdrawals")));
        assert!(r.notes.iter().any(|n| n.contains("“Iteration”")));
        assert_eq!(p.returns.source, ReturnSource::Historical);
        assert_eq!(p.returns.start_date, Some(ym(1990, 1)));
        assert_eq!(p.returns.extrapolation.method, ExtrapolationMethod::Geometric);
        // «false» as text counts as a monthly value
        let stocks = js(&p.returns.fixed.stocks).as_f64().unwrap();
        assert!((stocks - (1.05f64.powi(12) - 1.0)).abs() < 1e-12);
        let extras: Vec<_> = p.flows.extras.iter().map(|e| (e.date, e.amount)).collect();
        assert_eq!(extras, vec![(ym(2030, 1), 7000.0)]);
        // First entry from the start: no additional entry; without wealth share 0.5 as the default
        assert_eq!(
            js(&p.wealth.stock_share),
            json!({"mode": "timed", "rows": [{"from": "2030-01", "value": 0.6}]})
        );
        assert!(p.taxes.property_gain.is_none());
        let später = &p.realestate.properties[0];
        assert_eq!(später.status, PropertyStatus::Planned);
        assert_eq!(später.buy, Some(TimeRef::never()));

        let r = imp(
            json!({"CalculationMode": "single", "General": {}, "MoneyFlows": {"Extra": 0},
                           "WealthManagement": {"PortfolioBalance": 0.3}}),
        );
        assert!(r.notes.is_empty(), "{:?}", r.notes);
        assert!(r.plan.flows.extras.is_empty());
        assert_eq!(js(&r.plan.wealth.stock_share), json!(0.3));
        assert_eq!(r.plan.returns.source, ReturnSource::Fixed);
        assert!(r.plan.returns.parametric.is_none());

        let r = imp(json!({"General": {}, "WealthManagement": {"Stocks": 0, "Bonds": 0},
                           "RealEstate": {"Properties": [{"Name": "Nachher", "Price": 1}]}}));
        assert_eq!(js(&r.plan.wealth.stock_share), json!(0.5));
        let share = |stocks: f64, bonds: f64| {
            js(
                &imp(json!({"General": {}, "WealthManagement": {"Stocks": stocks, "Bonds": bonds}}))
                    .plan
                    .wealth
                    .stock_share,
            )
        };
        assert_eq!(share(100.0, 300.0), json!(0.25));
        assert_eq!(share(0.0, 100.0), json!(0.0));
        // Monte Carlo without data: defaults of the previous program
        let q = imp(json!({"General": {}, "MonteCarlo": {}}))
            .plan
            .returns
            .parametric
            .unwrap();
        assert_eq!(
            (q.paths, q.mu, q.sigma, q.df, q.jump_lambda, q.jump_mu, q.jump_sigma),
            (10_000, 0.105, 0.148, 6.0, 0.05, -0.3, 0.15)
        );
        assert_eq!(r.plan.realestate.properties[0].buy, Some(TimeRef::after_sale()));
    }
}
