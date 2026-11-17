//! Invarianten an vielen zufällig erzeugten Planungen (reproduzierbar über den Startwert):
//! keine ungültigen Zahlen, Bilanzen gehen auf, gleiche Eingaben ergeben gleiche Ergebnisse.

use engine::batch;
use engine::model::Plan;
use engine::montecarlo::Rng;
use engine::sim::{SimResult, Simulator};
use engine::validate::{self, Severity};
use serde_json::{json, Value};

const PLANS: u64 = 500;

struct Gen(Rng);

impl Gen {
    fn f(&mut self, a: f64, b: f64) -> f64 {
        a + (b - a) * self.0.uniform()
    }
    fn i(&mut self, a: i32, b: i32) -> i32 {
        a + self.0.below((b - a + 1) as usize) as i32
    }
    fn chance(&mut self, p: f64) -> bool {
        self.0.uniform() < p
    }
    fn month(&mut self, y0: i32, y1: i32) -> String {
        format!("{}-{:02}", self.i(y0, y1), self.i(1, 12))
    }
    fn pick<'a>(&mut self, xs: &[&'a str]) -> &'a str {
        xs[self.0.below(xs.len())]
    }
    /// Konstant oder zeitabhängig mit Meilensteinen und Daten.
    fn series(&mut self, a: f64, b: f64) -> Value {
        if self.chance(0.5) {
            return json!(self.f(a, b));
        }
        let mut rows = vec![json!({ "from": "start", "value": self.f(a, b) })];
        for _ in 0..self.i(0, 3) {
            let from = if self.chance(0.5) {
                json!(self.pick(&["retire", "ahv", "end"]))
            } else {
                json!(self.month(2026, 2060))
            };
            rows.push(json!({ "from": from, "value": self.f(a, b) }));
        }
        json!({ "mode": "timed", "rows": rows })
    }
    fn tax_table(&mut self, max_rate: f64) -> Value {
        let mut x = 0.0;
        let mut rows = vec![];
        for _ in 0..self.i(0, 4) {
            rows.push(json!([x, self.f(0.0, max_rate)]));
            x += self.f(10_000.0, 500_000.0);
        }
        json!(rows)
    }
    fn tranche(&mut self, owned: bool) -> Value {
        let periods: Vec<Value> = (0..self.i(1, 3))
            .map(|_| {
                json!({
                    "type": self.pick(&["fixed", "saron"]),
                    "term": self.i(1, 10),
                    "rate": self.f(0.0, 0.04),
                    "margin": self.f(0.0, 0.012),
                    "amort": if self.chance(0.5) { self.f(0.0, 3_000.0) } else { 0.0 },
                    "amortMode": self.pick(&["direct", "indirect"])
                })
            })
            .collect();
        let mut t = json!({
            "name": "T",
            "amount": self.f(10_000.0, 400_000.0),
            "after": self.pick(&["renew", "repay"]),
            "periods": periods
        });
        if owned {
            t["start"] = json!(self.month(2015, 2026));
        }
        t
    }
    fn property(&mut self, k: usize) -> Value {
        let status = self.pick(&["owned", "planned", "rent"]);
        let id = format!("p{k}");
        if status == "rent" {
            let from = self.pick(&["start", "afterSale", "retire"]);
            return json!({ "id": id, "name": format!("Miete {k}"), "status": "rent", "from": from,
                "rent": self.f(500.0, 4_000.0), "price": 0 });
        }
        let owned = status == "owned";
        let price = self.f(300_000.0, 1_500_000.0);
        let mut p = json!({
            "id": id, "name": format!("Objekt {k}"), "status": status, "price": price,
            "worth": price * self.f(0.8, 2.0), "fixCosts": self.f(0.0, 1_000.0),
            "inflation": self.chance(0.5),
            "sell": if self.chance(0.5) { json!(self.month(2027, 2055)) } else { json!("never") }
        });
        if !owned {
            p["buy"] = if self.chance(0.5) {
                json!("afterSale")
            } else {
                json!(self.month(2026, 2045))
            };
        }
        if self.chance(0.8) {
            let tranches: Vec<Value> = (0..self.i(1, 2)).map(|_| self.tranche(owned)).collect();
            let mut m = json!({ "tranches": tranches });
            if !owned && self.chance(0.3) {
                m["amountMode"] = json!("maxLoan");
            }
            if self.chance(0.5) {
                m["pillar3a"] = json!({ "balance": self.f(0.0, 80_000.0), "rate": self.f(0.0, 0.02),
                    "payout": self.pick(&["retire", "end", "2030-06"]), "use": self.pick(&["repay", "wealth"]) });
            }
            p["mortgage"] = m;
        }
        p
    }

    fn plan(&mut self, quiet: bool) -> Value {
        let birth = self.month(1955, 1975);
        let start = self.month(2026, 2027);
        let retire = self.month(2026, 2035);
        let end_age = self.i(80, 100);
        let mut v = crate::any_age(json!({
            "project": { "name": "Zufall", "currency": "CHF" },
            "household": { "persons": [ { "id": "A", "name": "A", "birth": birth } ] },
            "milestones": [
                { "id": "start", "name": "Start", "kind": "date", "date": start },
                { "id": "retire", "name": "Pensionierung", "kind": "date", "date": retire },
                { "id": "ahv", "name": "AHV", "kind": "age", "person": "A", "years": 65 },
                { "id": "end", "name": "Ende", "kind": "age", "person": "A", "years": end_age }
            ],
            "pension": {
                "pk": { "capital": self.f(0.0, 1_500_000.0), "at": "retire", "lumpsum": self.f(0.0, 1.0),
                        "conversion": self.f(0.03, 0.07), "contribution": self.series(0.0, 4_000.0),
                        "interest": self.series(0.0, 0.03) },
                "ahv": { "monthly": self.f(0.0, 2_520.0), "at": "ahv", "indexed": self.chance(0.7) }
            },
            "flows": {
                "income": self.series(0.0, 15_000.0),
                "spending": self.series(2_000.0, 12_000.0),
                "savings": self.series(0.0, 500.0),
                "extras": (0..self.i(0, 3)).map(|_| json!({ "date": self.month(2026, 2050),
                    "amount": self.f(-100_000.0, 200_000.0), "label": "Einmalig" })).collect::<Vec<_>>()
            },
            "wealth": { "stocks": self.f(0.0, 2_000_000.0), "bonds": self.f(0.0, 500_000.0),
                        "cash": self.f(0.0, 200_000.0), "stockShare": self.series(0.0, 1.0),
                        "reserveMonths": self.series(0.0, 36.0) },
            "returns": { "source": "fixed", "fixed": {
                "stocks": self.series(-0.1, 0.12), "bonds": self.series(-0.03, 0.05),
                "inflation": self.series(-0.01, 0.05), "saron": self.series(-0.0075, 0.03) } },
            "withdrawal": { "strategy": self.pick(&["standard", "three_buckets"]),
                "buckets": { "cashYears": self.f(0.0, 3.0), "bondYears": self.f(0.0, 8.0),
                             "refillThreshold": self.f(-0.1, 0.1) } },
            "taxes": { "multiplier": self.f(0.5, 2.5), "income": self.tax_table(0.12),
                       "wealth": self.tax_table(0.003), "capital": self.tax_table(0.08),
                       "propertyGain": { "brackets": [[4000, 0.1], [6000, 0.15], [1e9, 0.4]], "reduction": self.f(0.0, 0.5) } },
            "realestate": { "emergencyMonths": self.f(0.0, 36.0),
                "properties": (0..self.i(0, 3) as usize).map(|k| self.property(k)).collect::<Vec<_>>() }
        }));
        if quiet {
            // Für die Bilanzprüfung: ohne Renditen, Steuern, Pensionskasse und Immobilien.
            v["returns"]["fixed"]["stocks"] = json!(0);
            v["returns"]["fixed"]["bonds"] = json!(0);
            v["pension"]["pk"]["capital"] = json!(0);
            v["pension"]["pk"]["contribution"] = json!(0);
            v["taxes"] = json!({ "income": [], "wealth": [], "capital": [] });
            v["realestate"] = json!({ "properties": [] });
        }
        v
    }
}

fn run(v: &Value) -> Option<(Simulator, SimResult)> {
    let p: Plan = serde_json::from_value(v.clone()).expect("erzeugte Planung ist lesbar");
    if validate::validate(&p).iter().any(|i| i.severity == Severity::Error) {
        return None;
    }
    let sim = Simulator::new(&p).expect("ohne Prüffehler muss der Simulator starten");
    let path = batch::market_paths(&p, &sim, None).unwrap().swap_remove(0);
    let res = sim.run(&path, true);
    Some((sim, res))
}

#[test]
fn random_plans_keep_books_balanced() {
    let mut g = Gen(Rng::new(2026, 1));
    let mut ran = 0;
    for n in 0..PLANS {
        let v = g.plan(false);
        let Some((sim, res)) = run(&v) else { continue };
        ran += 1;
        let ctx = || format!("Planung {n}: {}", serde_json::to_string(&v).unwrap());
        assert_eq!(res.records.len(), sim.months(), "{}", ctx());
        for (k, r) in res.records.iter().enumerate() {
            assert_eq!(r.month, sim.start.plus(k as i32));
            for (name, x) in [
                ("cash", r.cash),
                ("stocks", r.stocks),
                ("bonds", r.bonds),
                ("equity", r.property_equity),
                ("pk", r.pension_fund),
                ("3a", r.pillar3a),
                ("total", r.total),
                ("income", r.income),
                ("spending", r.spending),
                ("taxes", r.taxes),
            ] {
                assert!(x.is_finite(), "{name} ungültig im {} – {}", r.month, ctx());
            }
            let tol = 1e-6 * r.total.abs().max(1.0);
            assert!(
                (r.liquid - (r.cash + r.stocks + r.bonds)).abs() < tol,
                "liquid {} – {}",
                r.month,
                ctx()
            );
            assert!(
                (r.total - (r.liquid + r.property_equity + r.pension_fund + r.pillar3a)).abs() < tol,
                "total {} – {}",
                r.month,
                ctx()
            );
            assert!(
                r.pension_fund >= -1e-9 && r.pillar3a >= -1e-9 && r.taxes >= -1e-9,
                "{} – {}",
                r.month,
                ctx()
            );
            if r.liquid > 1.0 {
                assert!(
                    r.stocks >= -1e-6 && r.bonds >= -1e-6,
                    "negative Anlage {} – {}",
                    r.month,
                    ctx()
                );
            }
        }
        let s = &res.summary;
        let taxes: f64 = res.records.iter().map(|r| r.taxes).sum();
        assert!(
            (s.taxes_total - taxes).abs() < 1e-6 * taxes.max(1.0),
            "Steuern total – {}",
            ctx()
        );
        if let Some(d) = s.depleted {
            assert!(d >= sim.start && d <= sim.end);
        }
        // Gleiche Eingaben ⇒ gleiche Ergebnisse.
        let (_, again) = run(&v).unwrap();
        assert_eq!(again.summary.end_total.to_bits(), s.end_total.to_bits(), "{}", ctx());
    }
    assert!(ran > PLANS / 2, "nur {ran} von {PLANS} Planungen ohne Prüffehler");
}

/// Ohne Renditen, Steuern, PK und Immobilien ändert sich das Cash genau um die Geldflüsse:
/// Einkommen (inkl. Renten) + weitere Zuflüsse + einmalige Beträge − Ausgaben. Umschichtungen
/// (Reserve, Töpfe) dürfen nichts verändern.
#[test]
fn cash_flows_are_conserved_without_returns() {
    let mut g = Gen(Rng::new(7, 2));
    let mut ran = 0;
    for n in 0..PLANS {
        let v = g.plan(true);
        let Some((sim, res)) = run(&v) else { continue };
        ran += 1;
        let p: Plan = serde_json::from_value(v.clone()).unwrap();
        let r = engine::resolve::Resolver::new(&p).unwrap();
        let savings = engine::resolve::Curve::compile(p.flows.savings.as_ref().unwrap(), &r).unwrap();
        let mut prev = p.wealth.cash + p.wealth.stocks + p.wealth.bonds;
        for rec in &res.records {
            let extras: f64 = p
                .flows
                .extras
                .iter()
                .filter(|x| x.date.max(sim.start) == rec.month)
                .map(|x| x.amount)
                .sum();
            let expected = prev + rec.income + savings.at(rec.month) + extras - rec.spending;
            assert!(
                (rec.liquid - expected).abs() < 1e-6 * expected.abs().max(1.0),
                "Planung {n}, {}: {} statt {expected} – {}",
                rec.month,
                rec.liquid,
                serde_json::to_string(&v).unwrap()
            );
            prev = rec.liquid;
        }
    }
    eprintln!("gerechnet: {ran} von {PLANS}");
    // #81/AK1: Planungen mit Prüffehlern werden übersprungen; mindestens die Hälfte muss gerechnet sein.
    assert!(ran > PLANS / 2, "nur {ran} von {PLANS} Planungen ohne Prüffehler");
}

/// Kauf und Verkauf zum Buchwert ohne Kosten ändern das Gesamtvermögen nicht.
#[test]
fn buying_and_selling_at_book_value_keeps_total() {
    const N: u64 = 100;
    let mut g = Gen(Rng::new(11, 3));
    let mut ran = 0;
    for _ in 0..N {
        let mut v = g.plan(true);
        let price = g.f(300_000.0, 900_000.0);
        v["wealth"]["cash"] = json!(2_000_000);
        v["wealth"]["stocks"] = json!(0);
        v["wealth"]["bonds"] = json!(0);
        v["flows"] = json!({ "income": 0, "spending": 0 });
        v["pension"]["ahv"]["monthly"] = json!(0);
        v["returns"]["fixed"]["inflation"] = json!(0);
        // #81/AK3: Ohne Grundstückgewinnsteuer scheiterte jede Planung an der Prüfung (Pflicht seit der Prüfung auf
        // Immobilien im Besitz oder mit geplantem Kauf); der Test rechnete keine einzige. Satz 0 % zählt als gesetzt.
        v["taxes"]["propertyGain"] = json!({ "brackets": [[1e9, 0.0]] });
        v["realestate"] = json!({ "properties": [ {
            "id": "a", "name": "A", "status": "planned", "buy": g.month(2028, 2030), "sell": g.month(2031, 2034),
            "price": price, "worth": price,
            "mortgage": { "tranches": [ { "amount": price * 0.6, "periods": [ { "term": 20, "rate": 0.0 } ] } ] }
        } ] });
        let Some((_, res)) = run(&v) else { continue };
        ran += 1;
        for rec in &res.records {
            assert!((rec.total - 2_000_000.0).abs() < 1e-6, "{}: {}", rec.month, rec.total);
        }
    }
    eprintln!("gerechnet: {ran} von {N}");
    // #81/AK1: Planungen mit Prüffehlern werden übersprungen; mindestens die Hälfte muss gerechnet sein.
    assert!(ran > N / 2, "nur {ran} von {N} Planungen ohne Prüffehler");
}
