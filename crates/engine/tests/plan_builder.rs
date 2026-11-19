//! #158: Gemeinsamer Baukasten für Testplanungen als JSON.
//!
//! `PlanBuilder::new` liefert eine Planung mit allen Pflichtfeldern: eine Person «A» (geboren 1965-01), Meilensteine
//! «start», «retire» und «end» als Datum, keine Vorsorge, keine Einnahmen und Ausgaben, kein Vermögen, feste Renditen
//! von 0, keine Steuern und offene Altersgrenzen wie `crate::any_age`. Kommt ein Pflichtfeld dazu, wird es hier
//! nachgetragen. Die Bausteine ändern nur die genannten Felder; Zahlen bleiben im übergebenen Typ (ganz oder
//! gebrochen), damit das JSON genau dem bisher von Hand geschriebenen entspricht.

use serde_json::{json, Value};

/// Testplanung als JSON, Baustein für Baustein zusammengesetzt; `build` gibt das JSON zurück.
pub struct PlanBuilder(Value);

impl PlanBuilder {
    /// Planung mit allen Pflichtfeldern (siehe Modulbeschreibung), zehn Jahre ab Januar 2030.
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

    /// Beispielplanung der App (`api::example_plan`), unverändert als Ausgangspunkt.
    pub fn example() -> Self {
        Self(engine::api::example_plan())
    }

    /// Währung im Projekt.
    pub fn currency(mut self, currency: &str) -> Self {
        self.0["project"]["currency"] = json!(currency);
        self
    }

    /// Daten der Meilensteine «start», «retire» und «end» (Format `JJJJ-MM`).
    pub fn dates(mut self, start: &str, retire: &str, end: &str) -> Self {
        for (i, date) in [start, retire, end].into_iter().enumerate() {
            self.0["milestones"][i]["date"] = json!(date);
        }
        self
    }

    /// Paar: zweite Person «B» mit Geburtsmonat `birth`.
    #[allow(dead_code)] // Baustein für weitere Testdateien
    pub fn couple(mut self, birth: &str) -> Self {
        let persons = self.0["household"]["persons"].as_array_mut().expect("Personen");
        persons.push(json!({ "id": "B", "name": "B", "birth": birth }));
        self
    }

    /// Einnahmen und Ausgaben pro Monat.
    pub fn flows(mut self, income: impl Into<Value>, spending: impl Into<Value>) -> Self {
        self.0["flows"] = json!({ "income": income.into(), "spending": spending.into() });
        self
    }

    /// Vermögen: Aktien, Obligationen, Bargeld, Aktienanteil und Reserve in Monaten.
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

    /// Feste Renditen und Inflation; die gewählte Quelle bleibt.
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

    /// Datenreihe für historische Renditen (zum Beispiel `library:Shiller CHF.csv`).
    pub fn series(mut self, series: &str) -> Self {
        self.0["returns"]["series"] = json!(series);
        self
    }

    /// Monte Carlo parametrisch mit dem Parameterblock `params` (siehe `parametric`).
    pub fn parametric(mut self, params: Value) -> Self {
        self.0["returns"]["source"] = json!("parametric");
        self.0["returns"]["parametric"] = params;
        self
    }

    /// Monte Carlo Bootstrap mit Blocklänge in Monaten.
    pub fn bootstrap(mut self, paths: usize, block: usize, seed: u64) -> Self {
        self.0["returns"]["source"] = json!("bootstrap");
        self.0["returns"]["bootstrap"] = json!({ "paths": paths, "block": block, "seed": seed });
        self
    }

    /// Immobilie anhängen (siehe `owned_home`, `rented_home`).
    #[allow(dead_code)] // Baustein für weitere Testdateien
    pub fn property(mut self, property: Value) -> Self {
        let estate = &mut self.0["realestate"];
        if estate["properties"].is_null() {
            estate["properties"] = json!([]);
        }
        estate["properties"].as_array_mut().expect("Immobilien").push(property);
        self
    }

    /// Beliebiges Feld der obersten Ebene setzen (zum Beispiel `taxes` oder `scenarios`).
    pub fn set(mut self, key: &str, value: Value) -> Self {
        self.0[key] = value;
        self
    }

    pub fn build(self) -> Value {
        self.0
    }
}

/// Üblicher Parameterblock für Monte Carlo parametrisch: Rendite 6 %, Schwankung 15 %, Normalverteilung, keine
/// Sprünge.
pub fn parametric(paths: usize, seed: u64) -> Value {
    json!({ "paths": paths, "mu": 0.06, "sigma": 0.15, "df": 0, "jumpLambda": 0, "jumpMu": 0, "jumpSigma": 0, "seed": seed })
}

/// Eigenheim ohne Hypothek, nie verkauft.
#[allow(dead_code)] // Baustein für weitere Testdateien
pub fn owned_home(price: impl Into<Value>, worth: impl Into<Value>, fix_costs: impl Into<Value>) -> Value {
    json!({ "id": "h", "name": "Haus", "status": "owned", "price": price.into(), "worth": worth.into(),
        "fixCosts": fix_costs.into(), "sell": "never" })
}

/// Mietwohnung ab dem Meilenstein oder Datum `from`.
#[allow(dead_code)] // Baustein für weitere Testdateien
pub fn rented_home(from: &str, rent: impl Into<Value>) -> Value {
    json!({ "name": "Miete", "status": "rent", "from": from, "rent": rent.into() })
}
