//! #158: Common building kit for test plans as JSON.
//!
//! `PlanBuilder::new` returns a plan with all required fields: one person «A» (born 1965-01), milestones
//! «start», «retire» and «end» as dates, no pension, no income and spending, no wealth, fixed returns
//! of 0, no taxes and open age limits like `crate::any_age`. If a required field is added, it is added
//! here. The building blocks change only the named fields; numbers stay in the type passed (integer or
//! fractional), so the JSON matches exactly what was previously written by hand.

use serde_json::{json, Value};

/// Test plan as JSON, assembled block by block; `build` returns the JSON.
pub struct PlanBuilder(Value);

impl PlanBuilder {
    /// Plan with all required fields (see module description), ten years from January 2030.
    pub fn new(name: &str) -> Self {
        Self(crate::any_age(json!({
            "project": { "name": name },
            "household": { "persons": [ { "id": "A", "name": "A", "birth": "1965-01" } ] },
            "milestones": [
                { "id": "start", "name": "Start", "kind": "date", "date": "2030-01" },
                { "id": "retire", "name": "Pensionierung", "kind": "date", "date": "2030-01" },
                { "id": "end", "name": "Ende", "kind": "date", "date": "2039-12" }
            ],
            "pension": {
                "pk": { "capital": 0, "at": "retire", "lumpsum": 0, "conversion": 0 },
                "ahv": { "monthly": 0, "at": "never" }
            },
            "flows": { "income": 0, "spending": 0 },
            "wealth": { "stocks": 0, "bonds": 0, "cash": 0, "stockShare": 1, "reserveMonths": 0 },
            "returns": { "source": "fixed", "fixed": { "stocks": 0, "bonds": 0, "inflation": 0 } },
            "taxes": { "income": [], "wealth": [], "capital": [], "propertyGain": { "brackets": [], "reduction": 0.0 } }
        })))
    }

    /// Example plan of the app (`api::example_plan`), unchanged as the starting point.
    pub fn example() -> Self {
        Self(engine::api::example_plan())
    }

    /// Currency in the project.
    pub fn currency(mut self, currency: &str) -> Self {
        self.0["project"]["currency"] = json!(currency);
        self
    }

    /// Dates of the milestones «start», «retire» and «end» (format `YYYY-MM`).
    pub fn dates(mut self, start: &str, retire: &str, end: &str) -> Self {
        for (i, date) in [start, retire, end].into_iter().enumerate() {
            self.0["milestones"][i]["date"] = json!(date);
        }
        self
    }

    /// Couple: second person «B» with birth month `birth`.
    #[allow(dead_code)] // building block for further test files
    pub fn couple(mut self, birth: &str) -> Self {
        let persons = self.0["household"]["persons"].as_array_mut().expect("Personen");
        persons.push(json!({ "id": "B", "name": "B", "birth": birth }));
        self
    }

    /// Marital status of the household (`household.married`).
    #[allow(dead_code)] // building block for further test files
    pub fn married(mut self, married: bool) -> Self {
        self.0["household"]["married"] = json!(married);
        self
    }

    /// Income and spending per month.
    pub fn flows(mut self, income: impl Into<Value>, spending: impl Into<Value>) -> Self {
        self.0["flows"] = json!({ "income": income.into(), "spending": spending.into() });
        self
    }

    /// Wealth: stocks, bonds, cash, stock share and reserve in months.
    pub fn wealth(
        mut self,
        stocks: impl Into<Value>,
        bonds: impl Into<Value>,
        cash: impl Into<Value>,
        stock_share: impl Into<Value>,
        reserve_months: impl Into<Value>,
    ) -> Self {
        self.0["wealth"] = json!({ "stocks": stocks.into(), "bonds": bonds.into(), "cash": cash.into(),
            "stockShare": stock_share.into(), "reserveMonths": reserve_months.into() });
        self
    }

    /// Fixed returns and inflation; the chosen source stays.
    pub fn fixed_returns(
        mut self,
        stocks: impl Into<Value>,
        bonds: impl Into<Value>,
        inflation: impl Into<Value>,
    ) -> Self {
        self.0["returns"]["fixed"] =
            json!({ "stocks": stocks.into(), "bonds": bonds.into(), "inflation": inflation.into() });
        self
    }

    /// Series for historical returns (for example `library:Shiller CHF.csv`).
    pub fn series(mut self, series: &str) -> Self {
        self.0["returns"]["series"] = json!(series);
        self
    }

    /// Monte Carlo parametric with the parameter block `params` (see `parametric`).
    pub fn parametric(mut self, params: Value) -> Self {
        self.0["returns"]["source"] = json!("parametric");
        self.0["returns"]["parametric"] = params;
        self
    }

    /// Monte Carlo bootstrap with block length in months.
    pub fn bootstrap(mut self, paths: usize, block: usize, seed: u64) -> Self {
        self.0["returns"]["source"] = json!("bootstrap");
        self.0["returns"]["bootstrap"] = json!({ "paths": paths, "block": block, "seed": seed });
        self
    }

    /// Append a property (see `owned_home`, `rented_home`).
    #[allow(dead_code)] // building block for further test files
    pub fn property(mut self, property: Value) -> Self {
        let estate = &mut self.0["realestate"];
        if estate["properties"].is_null() {
            estate["properties"] = json!([]);
        }
        estate["properties"].as_array_mut().expect("Immobilien").push(property);
        self
    }

    /// Set any top-level field (for example `taxes` or `scenarios`).
    pub fn set(mut self, key: &str, value: Value) -> Self {
        self.0[key] = value;
        self
    }

    pub fn build(self) -> Value {
        self.0
    }
}

/// Usual parameter block for parametric Monte Carlo: return 6 %, volatility 15 %, normal distribution, no
/// jumps.
pub fn parametric(paths: usize, seed: u64) -> Value {
    json!({ "paths": paths, "mu": 0.06, "sigma": 0.15, "df": 0, "jumpLambda": 0, "jumpMu": 0, "jumpSigma": 0, "seed": seed })
}

/// Own home without a mortgage, never sold.
#[allow(dead_code)] // building block for further test files
pub fn owned_home(price: impl Into<Value>, worth: impl Into<Value>, fix_costs: impl Into<Value>) -> Value {
    json!({ "id": "h", "name": "Haus", "status": "owned", "price": price.into(), "worth": worth.into(),
        "fixCosts": fix_costs.into(), "sell": "never" })
}

/// Rented flat from the milestone or date `from`.
#[allow(dead_code)] // building block for further test files
pub fn rented_home(from: &str, rent: impl Into<Value>) -> Value {
    json!({ "name": "Miete", "status": "rent", "from": from, "rent": rent.into() })
}
