//! FEA-53: Anlagestrategie. Die Planung wählt je einen Index für Aktien und Obligationen; daraus entsteht die Zeitreihe
//! der Simulation.
//!
//! - Kombinationen, die es als ausgelieferte Reihe gibt, liefern genau diese Reihe (Datenbibliothek, sonst die
//!   eingebaute): S&P 500 mit US-Obligationen = «Shiller CHF» (monatlich), S&P 500 mit Obligationen der
//!   Eidgenossenschaft = «Shiller CHF + CH-Obligationen» (monatlich ab 1988).
//! - Alle anderen Kombinationen sind jährlich (Entscheid zu FEA-53: jährlich, sobald ein Teil jährlich ist), über die
//!   Jahre, für die alle Teile Werte haben. Monatsreihen werden zu Kalenderjahren verknüpft.
//! - Teuerung aus der Quelle der Aktien; SARON (Jahresmittel) aus den SNB-Werten in «Shiller CHF».

use std::path::Path;

use crate::jst::{self, JstData};
use crate::library::{self, SeriesPart};
use crate::market::{DataSet, Freq, Meta};
use crate::model::{BondIndex, Investment, RegionWeights, StockIndex};
use crate::time::YearMonth;
use crate::{Error, Result};

/// Verweis einer Strategie an Stelle einer Zeitreihe (Kurzinfo in der Oberfläche): `strategy:` + JSON der Strategie.
pub const PREFIX: &str = "strategy:";

fn invalid(s: impl Into<String>) -> Error {
    Error::Invalid(s.into())
}

pub fn stock_name(s: StockIndex) -> &'static str {
    match s {
        StockIndex::ChJst => "Schweiz (JST)",
        StockIndex::WorldJst => "Welt (JST)",
        StockIndex::WorldCustom => "Welt individuell (JST)",
        StockIndex::Sp500 => "S&P 500 (Shiller)",
    }
}

pub fn bond_name(b: BondIndex) -> &'static str {
    match b {
        BondIndex::Ch => "Eidgenossenschaft 10 Jahre",
        BondIndex::Usa => "USA 10 Jahre (Shiller)",
        BondIndex::ChShort => "Eidgenossenschaft kurzfristig",
    }
}

/// Kurzer Name der Strategie.
pub fn label(s: &Investment) -> String {
    let stocks = match s.stocks {
        StockIndex::WorldCustom => {
            let w = &s.weights;
            let p = |v: f64| format!("{}", (v * 100.0).round());
            format!(
                "Welt individuell (CH {} %, USA {} %, EU {} %, Asien-Pazifik {} %)",
                p(w.ch),
                p(w.usa),
                p(w.eu),
                p(w.apac)
            )
        }
        x => stock_name(x).to_string(),
    };
    format!("Aktien {stocks}, Obligationen {}", bond_name(s.bonds))
}

/// Fehler in den Gewichten von «Welt individuell»: jedes zwischen 0 und 100 %, zusammen 100 %.
pub fn weight_issue(w: &RegionWeights) -> Option<String> {
    let all = [w.ch, w.usa, w.eu, w.apac];
    if all.iter().any(|v| !v.is_finite() || !(0.0..=1.0).contains(v)) {
        return Some("Jedes Gewicht der Regionen muss zwischen 0 % und 100 % liegen.".into());
    }
    let sum: f64 = all.iter().sum();
    ((sum - 1.0).abs() > 1e-6).then(|| {
        format!(
            "Die Gewichte der Regionen ergeben {} % statt 100 %.",
            (sum * 1000.0).round() / 10.0
        )
    })
}

/// Ausgelieferte Reihe aus der Datenbibliothek (falls vorhanden), sonst die eingebaute.
fn builtin_series(file: &str, lib: Option<&Path>) -> Result<DataSet> {
    if let Some(p) = lib.map(|d| d.join(file)).filter(|p| p.exists()) {
        return DataSet::load_csv(&p);
    }
    let (_, text) = library::BUILTIN
        .iter()
        .find(|(f, _)| *f == file)
        .ok_or_else(|| invalid(format!("Reihe «{file}» ist nicht ausgeliefert")))?;
    DataSet::parse_csv(file.trim_end_matches(".csv"), text)
}

/// Jahreswerte ab einem Jahr.
#[derive(Clone, Debug, PartialEq)]
struct Yearly {
    first: i32,
    values: Vec<f64>,
}

impl Yearly {
    fn last(&self) -> i32 {
        self.first + self.values.len() as i32 - 1
    }
    fn at(&self, y: i32) -> f64 {
        self.values[(y - self.first) as usize]
    }
}

/// Monatsrenditen zu Kalenderjahren verknüpft (nur ganze Jahre).
fn compound_years(first: YearMonth, monthly: &[f64]) -> Yearly {
    let mut out = Yearly {
        first: if first.month() == 1 {
            first.year()
        } else {
            first.year() + 1
        },
        values: vec![],
    };
    let mut i = (out.first * 12 - first.0).max(0) as usize;
    while i + 12 <= monthly.len() {
        out.values
            .push(monthly[i..i + 12].iter().map(|r| 1.0 + r).product::<f64>() - 1.0);
        i += 12;
    }
    out
}

/// Jahreswert aus Monatswerten, die schon Jahressätze sind (Inflation: Dezember gegenüber Vorjahr; SARON: Mittel
/// der zwölf Monate, `NaN` wenn ein Monat fehlt).
fn yearly_rates(first: YearMonth, monthly: &[f64], december_only: bool) -> Yearly {
    let c = compound_years(first, monthly);
    let start = (c.first * 12 - first.0) as usize;
    let values = (0..c.values.len())
        .map(|k| {
            let m = &monthly[start + 12 * k..start + 12 * k + 12];
            if december_only {
                m[11]
            } else {
                m.iter().sum::<f64>() / 12.0
            }
        })
        .collect();
    Yearly { first: c.first, values }
}

fn yearly_of(ds: &DataSet, col: &[f64]) -> Yearly {
    match ds.freq {
        Freq::Yearly => Yearly {
            first: ds.first.year(),
            values: col.to_vec(),
        },
        Freq::Monthly => compound_years(ds.first, col),
    }
}

/// Aktien einer Region (Länder nach BIP gewichtet), ab 1900.
fn region(jst: &JstData, countries: &[&str]) -> Result<Yearly> {
    Ok(Yearly {
        first: jst::FIRST_YEAR,
        values: jst.region_stocks(countries)?,
    })
}

/// «Welt individuell»: jedes Jahr die Regionen mit den festen Gewichten (jährlich angepasst).
fn custom_world(jst: &JstData, w: &RegionWeights) -> Result<Yearly> {
    let parts = [
        (w.ch, jst::SWITZERLAND),
        (w.usa, jst::USA),
        (w.eu, jst::EU),
        (w.apac, jst::APAC),
    ];
    let mut values = vec![0.0; (jst.last_year() - jst::FIRST_YEAR + 1) as usize];
    for (weight, r) in parts.iter().filter(|(w, _)| *w > 0.0) {
        for (v, x) in values.iter_mut().zip(region(jst, r.countries)?.values) {
            *v += weight * x;
        }
    }
    Ok(Yearly {
        first: jst::FIRST_YEAR,
        values,
    })
}

/// Die Zeitreihe zur Strategie. `lib`: Ordner der Datenbibliothek (für neuere Shiller- und SNB-Daten).
pub fn dataset(s: &Investment, lib: Option<&Path>) -> Result<DataSet> {
    dataset_with_parts(s, lib).map(|(ds, _)| ds)
}

/// FEA-53/AK8: Aktien und Obligationen mit ihren eigenen Jahren (die Teuerung stammt aus der Quelle der Aktien).
fn parts(s: &Investment, stocks: (i32, i32), bonds: (i32, i32)) -> Vec<SeriesPart> {
    vec![
        SeriesPart {
            label: format!("Aktien {}", stock_name(s.stocks)),
            first: stocks.0,
            last: stocks.1,
        },
        SeriesPart {
            label: format!("Obligationen {}", bond_name(s.bonds)),
            first: bonds.0,
            last: bonds.1,
        },
    ]
}

/// Jahre einer Reihe (erstes und letztes Jahr mit Werten).
fn years_of(ds: &DataSet) -> (i32, i32) {
    (ds.first.year(), ds.last().year())
}

/// Die Zeitreihe zur Strategie und ihre Teile mit den eigenen Jahren; die Reihe umfasst die gemeinsamen Jahre.
pub fn dataset_with_parts(s: &Investment, lib: Option<&Path>) -> Result<(DataSet, Vec<SeriesPart>)> {
    if s.stocks == StockIndex::WorldCustom {
        if let Some(e) = weight_issue(&s.weights) {
            return Err(invalid(e));
        }
    }
    let shipped = match (s.stocks, s.bonds) {
        (StockIndex::Sp500, BondIndex::Usa) => Some("Shiller CHF.csv"),
        (StockIndex::Sp500, BondIndex::Ch) => Some(library::CH_BONDS_FILE),
        _ => None,
    };
    if let Some(file) = shipped {
        let mut ds = builtin_series(file, lib)?;
        ds.name = label(s);
        let own = years_of(&ds);
        // S&P 500 mit Obligationen der Eidgenossenschaft: die Aktien reichen weiter zurück als die SNB-Renditen.
        let stocks = if s.bonds == BondIndex::Ch {
            years_of(&builtin_series("Shiller CHF.csv", lib)?)
        } else {
            own
        };
        return Ok((ds, parts(s, stocks, own)));
    }

    let jst = JstData::builtin();
    let snb = match lib {
        Some(d) => library::snb_data(d)?,
        None => crate::import::SnbData::builtin(),
    };
    let shiller = builtin_series("Shiller CHF.csv", lib)?;

    let mut sources: Vec<String> = vec![];
    let (stocks, inflation) = match s.stocks {
        StockIndex::Sp500 => {
            sources.push(shiller.meta.source.clone().unwrap_or_default());
            (
                yearly_of(&shiller, &shiller.stocks),
                yearly_rates(shiller.first, &shiller.inflation, true),
            )
        }
        x => {
            sources.push(jst::SOURCE.to_string());
            let stocks = match x {
                StockIndex::ChJst => region(&jst, jst::SWITZERLAND.countries)?,
                StockIndex::WorldJst => {
                    let all: Vec<&str> = jst::REGIONS.iter().flat_map(|r| r.countries.iter().copied()).collect();
                    region(&jst, &all)?
                }
                _ => custom_world(&jst, &s.weights)?,
            };
            let inflation = Yearly {
                first: jst::FIRST_YEAR,
                values: (jst::FIRST_YEAR..=jst.last_year())
                    .map(|y| {
                        jst.ch_inflation(y)
                            .ok_or_else(|| invalid(format!("Teuerung Schweiz {y} fehlt")))
                    })
                    .collect::<Result<_>>()?,
            };
            (stocks, inflation)
        }
    };
    let bonds = match s.bonds {
        BondIndex::Usa => {
            sources.push(shiller.meta.source.clone().unwrap_or_default());
            yearly_of(&shiller, &shiller.bonds)
        }
        BondIndex::Ch => {
            sources.push(format!("{}; SNB (data.snb.ch): rendoblim", jst::SOURCE));
            let (first, values) = jst::ch_bonds_yearly(&jst, &snb)?;
            Yearly { first, values }
        }
        BondIndex::ChShort => {
            sources.push(format!("{}; SNB (data.snb.ch): zimoma (SARON)", jst::SOURCE));
            let (first, values) = jst::ch_short_yearly(&jst, &snb)?;
            Yearly { first, values }
        }
    };
    let saron = yearly_rates(shiller.first, shiller.saron.as_deref().unwrap_or(&[]), false);

    let first = stocks.first.max(bonds.first).max(inflation.first);
    let last = stocks.last().min(bonds.last()).min(inflation.last());
    if last < first {
        return Err(invalid(format!(
            "Die Indizes der Strategie «{}» haben keine gemeinsamen Jahre.",
            label(s)
        )));
    }
    let years = first..=last;
    let saron_at = |y: i32| {
        if saron.values.is_empty() || y < saron.first || y > saron.last() {
            f64::NAN
        } else {
            saron.at(y)
        }
    };
    sources.dedup();
    let mut notes = vec![
        format!(
            "Aktien: {}; Obligationen: {}; jährlich (Monatswerte zu Kalenderjahren verknüpft), in CHF.",
            stock_name(s.stocks),
            bond_name(s.bonds)
        ),
        "SARON: Jahresmittel der SNB-Monatswerte, vor 2000 gilt der fixe Wert.".into(),
    ];
    if s.bonds == BondIndex::ChShort {
        notes.push(format!(
            "Eidgenossenschaft kurzfristig: JST (Satz für etwa 3 Monate) bis {}, danach Jahresmittel des SARON; die Rendite entspricht praktisch dem Geldmarktsatz.",
            jst.last_year()
        ));
    }
    if matches!(
        s.stocks,
        StockIndex::ChJst | StockIndex::WorldJst | StockIndex::WorldCustom
    ) || matches!(s.bonds, BondIndex::Ch | BondIndex::ChShort)
    {
        notes.push("JST-Daten: Lizenz CC BY-NC-SA 4.0 (nicht kommerziell).".into());
    }
    let parts = parts(s, (stocks.first, stocks.last()), (bonds.first, bonds.last()));
    let ds = DataSet {
        name: label(s),
        meta: Meta {
            title: Some(label(s)),
            currency: Some("CHF".into()),
            source: Some(sources.join("; ")),
            info: Some(format!(
                "Aktien {}, Obligationen {}: jährlich {first}–{last}, in CHF ohne Währungsabsicherung.",
                stock_name(s.stocks),
                bond_name(s.bonds)
            )),
            notes,
        },
        freq: Freq::Yearly,
        first: YearMonth::new(first, 1),
        stocks: years.clone().map(|y| stocks.at(y)).collect(),
        bonds: years.clone().map(|y| bonds.at(y)).collect(),
        inflation: years.clone().map(|y| inflation.at(y)).collect(),
        saron: Some(years.map(saron_at).collect()),
    };
    Ok((ds, parts))
}

/// Zeitreihe zu einem Verweis `strategy:<JSON>`; `None`, wenn der Verweis keine Strategie ist.
pub fn from_reference(reference: &str, lib: Option<&Path>) -> Option<Result<DataSet>> {
    Some(from_reference_with_parts(reference, lib)?.map(|(ds, _)| ds))
}

/// Wie [`from_reference`], mit den Teilen der Strategie und ihren eigenen Jahren (FEA-53/AK8).
pub fn from_reference_with_parts(reference: &str, lib: Option<&Path>) -> Option<Result<(DataSet, Vec<SeriesPart>)>> {
    let json = reference.strip_prefix(PREFIX)?;
    Some(
        serde_json::from_str::<Investment>(json)
            .map_err(|e| invalid(format!("Strategie nicht lesbar: {e}")))
            .and_then(|s| dataset_with_parts(&s, lib)),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compound_years_takes_whole_calendar_years_only() {
        // Start im März: das erste ganze Jahr beginnt im folgenden Januar.
        let monthly = vec![0.01; 10 + 24 + 5];
        let y = compound_years(YearMonth::new(2000, 3), &monthly);
        assert_eq!((y.first, y.values.len()), (2001, 2));
        assert!((y.values[0] - (1.01f64.powi(12) - 1.0)).abs() < 1e-12);
        let y = compound_years(YearMonth::new(2000, 1), &[0.0; 12]);
        assert_eq!((y.first, y.last()), (2000, 2000));
    }

    #[test]
    fn yearly_rates_december_or_mean() {
        let mut m: Vec<f64> = (0..12).map(|i| i as f64).collect();
        m.extend([1.0; 12]);
        let dec = yearly_rates(YearMonth::new(2000, 1), &m, true);
        assert_eq!(dec.values, vec![11.0, 1.0]);
        let mean = yearly_rates(YearMonth::new(2000, 1), &m, false);
        assert_eq!(mean.values, vec![5.5, 1.0]);
    }

    #[test]
    fn weights_must_be_between_0_and_1_and_add_up_to_1() {
        let w = |ch, usa, eu, apac| RegionWeights { ch, usa, eu, apac };
        assert_eq!(weight_issue(&w(0.25, 0.25, 0.25, 0.25)), None);
        assert_eq!(weight_issue(&RegionWeights::default()), None);
        assert!(weight_issue(&w(0.5, 0.5, 0.5, 0.0)).unwrap().contains("150 %"));
        assert!(weight_issue(&w(1.2, -0.2, 0.0, 0.0)).unwrap().contains("zwischen"));
        assert!(weight_issue(&w(f64::NAN, 1.0, 0.0, 0.0)).is_some());
    }

    #[test]
    fn labels() {
        let s = Investment::default();
        assert_eq!(label(&s), "Aktien Welt (JST), Obligationen Eidgenossenschaft 10 Jahre");
        let c = Investment {
            stocks: StockIndex::WorldCustom,
            ..Default::default()
        };
        assert!(label(&c).contains("CH 20 %, USA 40 %, EU 30 %, Asien-Pazifik 10 %"));
    }
}
