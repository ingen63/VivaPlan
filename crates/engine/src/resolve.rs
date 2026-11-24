//! Resolving milestones and time references; time-dependent values as fast lookup curves.

use std::collections::HashMap;

use crate::model::{Interpolation, MilestoneKind, Plan, Series, SeriesDef, SeriesPoint};
use crate::time::{TimeRef, YearMonth};
use crate::{Error, Result};

pub struct Resolver {
    births: HashMap<String, YearMonth>,
    /// #217: person of an age in a support point without a person.
    first_person: Option<String>,
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
                        Error::Invalid(msg!(milestone_person_unknown, name = ms.name_arg(), person = person))
                    })?;
                    birth.plus((years * 12.0).round() as i32)
                }
            };
            if milestones.insert(ms.id.clone(), m).is_some() {
                return Err(Error::Invalid(msg!(milestone_id_duplicate, id = ms.id)));
            }
            names.insert(ms.id.clone(), ms.label().into_owned());
        }
        // #51: The fixed milestones of every person always exist; if they are missing (older plans, tests),
        // they apply with the default age.
        for (i, p) in plan.household.persons.iter().take(2).enumerate() {
            let (pk, ahv) = crate::model::person_milestones(i);
            for id in [pk, ahv] {
                if !milestones.contains_key(id) {
                    milestones.insert(
                        id.to_string(),
                        p.birth.plus((crate::settings::DEFAULT_AGE * 12.0) as i32),
                    );
                    let name = crate::model::fixed_milestone_name(id).unwrap_or_else(|| id.to_string());
                    names.insert(id.to_string(), name);
                }
            }
        }
        Ok(Resolver {
            births,
            first_person: plan.household.persons.first().map(|p| p.id.clone()),
            milestones,
            names,
        })
    }

    pub fn milestone(&self, id: &str) -> Result<YearMonth> {
        self.milestones
            .get(id)
            .copied()
            .ok_or_else(|| Error::Invalid(msg!(milestone_missing, id = id)))
    }

    pub fn milestone_name<'a>(&'a self, id: &'a str) -> &'a str {
        self.names.get(id).map(String::as_str).unwrap_or(id)
    }

    /// `None` means «nie» (or «nach Verkauf», which is only known during the simulation).
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
            .ok_or_else(|| Error::Invalid(msg!(time_ref_unknown, reference = r.0)))
    }

    pub fn birth(&self, person: &str) -> Option<YearMonth> {
        self.births.get(person).copied()
    }

    /// #217: month of a support point (date, milestone or age of a person; the first person without a person).
    pub fn point(&self, p: &SeriesPoint) -> Result<YearMonth> {
        match (&p.at, p.age) {
            (Some(at), None) => self
                .resolve(at)?
                .ok_or_else(|| Error::Invalid(msg!(series_point_not_fixed, reference = at.0))),
            (None, Some(years)) => {
                if !(0.0..=MAX_POINT_AGE).contains(&years) {
                    return Err(Error::Invalid(msg!(series_point_age_invalid, years = years)));
                }
                let person = p.person.as_deref().or(self.first_person.as_deref()).unwrap_or("");
                let birth = self
                    .births
                    .get(person)
                    .ok_or_else(|| Error::Invalid(msg!(series_point_person_unknown, person = person)))?;
                Ok(birth.plus((years * 12.0).round() as i32))
            }
            _ => Err(Error::Invalid(msg!(series_point_when))),
        }
    }
}

/// #217: highest age of a support point.
pub const MAX_POINT_AGE: f64 = 150.0;

/// #217: at most this many months of a straight line get a value of their own; a longer line goes in coarser steps
/// (only with points in time far apart, the simulation lasts at most 100 years).
const MAX_LINE_MONTHS: i32 = 12 * 200;

/// #217: the values of a straight line from `(m1, a)` to `(m2, b)` after `m1` up to and with `m2`.
fn push_line(points: &mut Vec<(i32, f64)>, (m1, a): (i32, f64), (m2, b): (i32, f64)) {
    let span = i64::from(m2) - i64::from(m1);
    if span > 1 {
        let step = (span + i64::from(MAX_LINE_MONTHS) - 1) / i64::from(MAX_LINE_MONTHS);
        let mut k = step;
        while k < span {
            points.push(((i64::from(m1) + k) as i32, a + (b - a) * (k as f64 / span as f64)));
            k += step;
        }
    }
    points.push((m2, b));
}

/// Step function over calendar months: the last valid value, 0 before the first entry.
#[derive(Clone, Debug, PartialEq)]
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
                // Sort stably: for the same month the later entry applies.
                points.sort_by_key(|p| p.0);
                Ok(Curve { points })
            }
            Series::Def(SeriesDef::Linear { start, end }) => {
                let (m1, m2) = (r.point(start)?, r.point(end)?);
                if m2 < m1 {
                    return Err(Error::Invalid(msg!(linear_end_before_start, start = m1, end = m2)));
                }
                let mut points = vec![(i32::MIN, start.value)];
                push_line(&mut points, (m1.0, start.value), (m2.0, end.value));
                Ok(Curve { points })
            }
            Series::Def(SeriesDef::Table {
                interpolation,
                points: rows,
            }) => {
                let mut support = Vec::with_capacity(rows.len());
                for row in rows {
                    support.push((r.point(row)?.0, row.value));
                }
                // Sort stably: for the same month the later entry applies.
                support.sort_by_key(|p| p.0);
                let Some(first) = support.first() else {
                    return Ok(Curve { points: vec![] });
                };
                let mut points = vec![(i32::MIN, first.1)];
                for pair in support.windows(2) {
                    match interpolation {
                        Interpolation::Linear if pair[0].0 < pair[1].0 => push_line(&mut points, pair[0], pair[1]),
                        _ => points.push(pair[1]),
                    }
                }
                Ok(Curve { points })
            }
            Series::Def(SeriesDef::Factor { base, factor }) => {
                let mut c = Curve::compile(factor, r)?;
                for p in &mut c.points {
                    p.1 *= base;
                }
                Ok(c)
            }
        }
    }

    pub fn at(&self, m: YearMonth) -> f64 {
        match self.points.partition_point(|p| p.0 <= m.0) {
            0 => 0.0,
            i => self.points[i - 1].1,
        }
    }

    /// First value above `eps` between `from` and `to` (both inclusive), without going month by month.
    pub fn first_above(&self, from: YearMonth, to: YearMonth, eps: f64) -> Option<f64> {
        if from > to {
            return None;
        }
        std::iter::once(self.at(from))
            .chain(self.points.iter().filter(|p| from.0 < p.0 && p.0 <= to.0).map(|p| p.1))
            .find(|x| *x > eps)
    }

    /// First month from which a value applies (`None` for constant values).
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

    // #40: Names, birth dates, first valid month and values of a curve.
    #[test]
    fn names_births_first_and_values() {
        let r = Resolver {
            births: HashMap::from([("p1".to_string(), YearMonth::new(1965, 4))]),
            first_person: None,
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

    // #51: first value above the threshold in the range (both edges inclusive).
    #[test]
    fn first_above_checks_range_edges_and_eps() {
        let c = Curve {
            points: vec![(10, 0.0), (20, 0.005), (30, 7.0), (40, 9.0)],
        };
        // The initial value at `from` counts (here 0 before the first entry).
        assert_eq!(c.first_above(YearMonth(5), YearMonth(15), 0.005), None);
        // A value exactly at eps is not above it.
        assert_eq!(c.first_above(YearMonth(20), YearMonth(25), 0.005), None);
        // An entry exactly at the upper edge counts; the first hit is returned.
        assert_eq!(c.first_above(YearMonth(10), YearMonth(30), 0.005), Some(7.0));
        assert_eq!(c.first_above(YearMonth(10), YearMonth(29), 0.005), None);
        // A value that already applies at `from` (entry before `from`).
        assert_eq!(c.first_above(YearMonth(35), YearMonth(35), 0.005), Some(7.0));
        // An entry exactly at `from` is contained in `at(from)`, not needed twice.
        assert_eq!(c.first_above(YearMonth(40), YearMonth(40), 0.005), Some(9.0));
        // from > to: empty.
        assert_eq!(c.first_above(YearMonth(40), YearMonth(10), -1.0), None);
        assert_eq!(
            Curve::constant(2.0).first_above(YearMonth(0), YearMonth(0), 1.0),
            Some(2.0)
        );
    }
}
