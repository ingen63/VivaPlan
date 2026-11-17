//! Auflösen von Meilensteinen und Zeitbezügen; zeitabhängige Werte als schnelle Nachschlagekurven.

use std::collections::HashMap;

use crate::model::{MilestoneKind, Plan, Series, SeriesDef};
use crate::time::{TimeRef, YearMonth};
use crate::{Error, Result};

pub struct Resolver {
    births: HashMap<String, YearMonth>,
    milestones: HashMap<String, YearMonth>,
    names: HashMap<String, String>,
}

impl Resolver {
    pub fn new(plan: &Plan) -> Result<Self> {
        let births: HashMap<_, _> = plan.household.persons.iter().map(|p| (p.id.clone(), p.birth)).collect();
        let mut milestones = HashMap::new();
        let mut names = HashMap::new();
        for ms in &plan.milestones {
            let m = match &ms.kind {
                MilestoneKind::Date { date } => *date,
                MilestoneKind::Age { person, years } => {
                    let birth = births.get(person).ok_or_else(|| {
                        Error::Invalid(format!("Meilenstein «{}»: unbekannte Person «{person}»", ms.name))
                    })?;
                    birth.plus((years * 12.0).round() as i32)
                }
            };
            if milestones.insert(ms.id.clone(), m).is_some() {
                return Err(Error::Invalid(format!(
                    "Meilenstein-ID «{}» ist doppelt vergeben",
                    ms.id
                )));
            }
            names.insert(ms.id.clone(), ms.name.clone());
        }
        // #51: Die festen Meilensteine jeder Person sind immer vorhanden; fehlen sie (ältere Planungen, Tests),
        // gelten sie mit dem Standardalter.
        for (i, p) in plan.household.persons.iter().take(2).enumerate() {
            let (pk, ahv) = crate::model::person_milestones(i);
            for (id, base) in [(pk, crate::model::PK_NAME), (ahv, crate::model::AHV_NAME)] {
                if !milestones.contains_key(id) {
                    milestones.insert(
                        id.to_string(),
                        p.birth.plus((crate::settings::DEFAULT_AGE * 12.0) as i32),
                    );
                    names.insert(id.to_string(), crate::model::person_milestone_name(base, i));
                }
            }
        }
        Ok(Resolver {
            births,
            milestones,
            names,
        })
    }

    pub fn milestone(&self, id: &str) -> Result<YearMonth> {
        self.milestones
            .get(id)
            .copied()
            .ok_or_else(|| Error::Invalid(format!("Meilenstein «{id}» fehlt")))
    }

    pub fn milestone_name<'a>(&'a self, id: &'a str) -> &'a str {
        self.names.get(id).map(String::as_str).unwrap_or(id)
    }

    /// `None` bedeutet «nie» (bzw. «nach Verkauf», das erst während der Simulation feststeht).
    pub fn resolve(&self, r: &TimeRef) -> Result<Option<YearMonth>> {
        if r.is_never() || r.is_dynamic() {
            return Ok(None);
        }
        if let Some(d) = r.as_date() {
            return Ok(Some(d));
        }
        self.milestones
            .get(&r.0)
            .map(|m| Some(*m))
            .ok_or_else(|| Error::Invalid(format!("Unbekannter Zeitbezug «{}»", r.0)))
    }

    pub fn birth(&self, person: &str) -> Option<YearMonth> {
        self.births.get(person).copied()
    }
}

/// Stufenfunktion über Kalendermonate: der zuletzt gültige Wert, vor dem ersten Eintrag 0.
#[derive(Clone, Debug)]
pub struct Curve {
    points: Vec<(i32, f64)>,
}

impl Curve {
    pub fn constant(v: f64) -> Self {
        Curve {
            points: vec![(i32::MIN, v)],
        }
    }

    pub fn compile(s: &Series, r: &Resolver) -> Result<Self> {
        match s {
            Series::Number(v) | Series::Def(SeriesDef::Const { value: v }) => Ok(Curve::constant(*v)),
            Series::Def(SeriesDef::Timed { rows }) => {
                let mut points = Vec::with_capacity(rows.len());
                for row in rows {
                    if let Some(m) = r.resolve(&row.from)? {
                        points.push((m.0, row.value));
                    }
                }
                // Stabil sortieren: bei gleichem Monat gilt der spätere Eintrag.
                points.sort_by_key(|p| p.0);
                Ok(Curve { points })
            }
        }
    }

    pub fn at(&self, m: YearMonth) -> f64 {
        match self.points.partition_point(|p| p.0 <= m.0) {
            0 => 0.0,
            i => self.points[i - 1].1,
        }
    }

    /// Erster Wert über `eps` zwischen `from` und `to` (beide inklusive), ohne Monat für Monat zu gehen.
    pub fn first_above(&self, from: YearMonth, to: YearMonth, eps: f64) -> Option<f64> {
        if from > to {
            return None;
        }
        std::iter::once(self.at(from))
            .chain(self.points.iter().filter(|p| from.0 < p.0 && p.0 <= to.0).map(|p| p.1))
            .find(|x| *x > eps)
    }

    /// Erster Monat, ab dem ein Wert gilt (`None` bei konstanten Werten).
    pub fn first(&self) -> Option<i32> {
        self.points.first().map(|p| p.0).filter(|m| *m != i32::MIN)
    }

    pub fn values(&self) -> impl Iterator<Item = f64> + '_ {
        self.points.iter().map(|p| p.1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn step_lookup() {
        let c = Curve {
            points: vec![(10, 1.0), (20, 2.0), (20, 3.0)],
        };
        assert_eq!(c.at(YearMonth(9)), 0.0);
        assert_eq!(c.at(YearMonth(10)), 1.0);
        assert_eq!(c.at(YearMonth(19)), 1.0);
        assert_eq!(c.at(YearMonth(20)), 3.0);
        assert_eq!(Curve::constant(5.0).at(YearMonth(-100)), 5.0);
    }

    // #40: Namen, Geburtsdaten, erster gültiger Monat und Werte einer Kurve.
    #[test]
    fn names_births_first_and_values() {
        let r = Resolver {
            births: HashMap::from([("p1".to_string(), YearMonth::new(1965, 4))]),
            milestones: HashMap::new(),
            names: HashMap::from([("retire".to_string(), "Pensionierung".to_string())]),
        };
        assert_eq!(r.milestone_name("retire"), "Pensionierung");
        assert_eq!(r.milestone_name("unbekannt"), "unbekannt");
        assert_eq!(r.birth("p1"), Some(YearMonth::new(1965, 4)));
        assert_eq!(r.birth("p2"), None);

        let c = Curve {
            points: vec![(10, 1.5), (20, -2.0)],
        };
        assert_eq!(c.first(), Some(10));
        assert_eq!(c.values().collect::<Vec<_>>(), vec![1.5, -2.0]);
        assert_eq!(Curve::constant(5.0).first(), None);
        assert_eq!(Curve::constant(5.0).values().collect::<Vec<_>>(), vec![5.0]);
        assert_eq!(Curve { points: vec![] }.first(), None);
    }

    // #51: erster Wert über der Schwelle im Bereich (beide Ränder inklusive).
    #[test]
    fn first_above_checks_range_edges_and_eps() {
        let c = Curve {
            points: vec![(10, 0.0), (20, 0.005), (30, 7.0), (40, 9.0)],
        };
        // Ausgangswert bei `from` zählt (hier 0 vor dem ersten Eintrag).
        assert_eq!(c.first_above(YearMonth(5), YearMonth(15), 0.005), None);
        // Wert genau auf eps ist nicht darüber.
        assert_eq!(c.first_above(YearMonth(20), YearMonth(25), 0.005), None);
        // Eintrag genau am oberen Rand zählt, der erste Treffer wird geliefert.
        assert_eq!(c.first_above(YearMonth(10), YearMonth(30), 0.005), Some(7.0));
        assert_eq!(c.first_above(YearMonth(10), YearMonth(29), 0.005), None);
        // Wert, der schon bei `from` gilt (Eintrag vor `from`).
        assert_eq!(c.first_above(YearMonth(35), YearMonth(35), 0.005), Some(7.0));
        // Eintrag genau bei `from` ist über `at(from)` enthalten, nicht doppelt nötig.
        assert_eq!(c.first_above(YearMonth(40), YearMonth(40), 0.005), Some(9.0));
        // from > to: leer.
        assert_eq!(c.first_above(YearMonth(40), YearMonth(10), -1.0), None);
        assert_eq!(
            Curve::constant(2.0).first_above(YearMonth(0), YearMonth(0), 1.0),
            Some(2.0)
        );
    }
}
