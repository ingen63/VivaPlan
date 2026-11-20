//! #69: Investment strategy. The plan chooses one index each for stocks and bonds; from them the series of the
//! simulation is built.
//!
//! - Combinations that exist as a bundled series give exactly that series (data library, otherwise the
//!   built-in one): S&P 500 with US bonds = «Shiller CHF» (monthly), S&P 500 with Swiss Confederation
//!   bonds = «Shiller CHF + CH-Obligationen» (monthly from 1988).
//! - All other combinations are annual (decision on #69: annual as soon as one part is annual), over the
//!   years for which all parts have values. Monthly series are linked to calendar years.
//! - Inflation from the source of the stocks; SARON (annual average) from the SNB values in «Shiller CHF».

use std::path::Path;

use crate::jst::{self, JstData};
use crate::library::{self, SeriesPart};
use crate::market::{DataSet, Freq, Meta, StockSeries};
use crate::message::Message;
use crate::model::{BondIndex, Investment, RegionWeights, StockIndex};
use crate::time::YearMonth;
use crate::{Error, Result};

/// Reference of a strategy instead of a series (short info in the user interface): `strategy:` + JSON of the strategy.
pub const PREFIX: &str = "strategy:";

fn invalid(m: crate::message::Message) -> Error {
    Error::Invalid(m)
}

/// #177: Name of the stock index (`engine.index_*` in the text tables).
pub fn stock_name(s: StockIndex) -> Message {
    match s {
        StockIndex::ChJst => msg!(index_ch_jst),
        StockIndex::WorldJst => msg!(index_world_jst),
        StockIndex::WorldCustom => msg!(index_world_custom),
        StockIndex::Sp500 => msg!(index_sp500),
    }
}

/// #177: Name of the bond index.
pub fn bond_name(b: BondIndex) -> Message {
    match b {
        BondIndex::Ch => msg!(index_confed_10),
        BondIndex::Usa => msg!(index_us_10),
        BondIndex::ChShort => msg!(index_confed_short),
    }
}

/// Short name of the strategy.
pub fn label(s: &Investment) -> Message {
    let stocks = match s.stocks {
        StockIndex::WorldCustom => {
            let w = &s.weights;
            let p = |v: f64| (v * 100.0).round();
            msg!(
                index_world_custom_weights,
                ch = p(w.ch),
                usa = p(w.usa),
                eu = p(w.eu),
                apac = p(w.apac)
            )
        }
        x => stock_name(x),
    };
    msg!(investment_label, stocks = stocks, bonds = bond_name(s.bonds))
}

/// Errors in the weights of «Welt individuell»: each between 0 and 100 %, together 100 %.
pub fn weight_issue(w: &RegionWeights) -> Option<crate::message::Message> {
    let all = [w.ch, w.usa, w.eu, w.apac];
    if all.iter().any(|v| !v.is_finite() || !(0.0..=1.0).contains(v)) {
        return Some(msg!(weights_range));
    }
    let sum: f64 = all.iter().sum();
    ((sum - 1.0).abs() > 1e-6).then(|| msg!(weights_sum, sum = (sum * 1000.0).round() / 10.0))
}

/// Bundled series from the data library (if present), otherwise the built-in one.
fn builtin_series(file: &str, lib: Option<&Path>) -> Result<DataSet> {
    if let Some(p) = lib.map(|d| d.join(file)).filter(|p| p.exists()) {
        return DataSet::load_csv(&p);
    }
    let (_, text) = library::BUILTIN
        .iter()
        .find(|(f, _)| *f == file)
        .ok_or_else(|| invalid(msg!(series_not_shipped, file = file)))?;
    DataSet::parse_csv(file.trim_end_matches(".csv"), text)
}

/// Annual values from a year.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct Yearly {
    pub(crate) first: i32,
    pub(crate) values: Vec<f64>,
}

impl Yearly {
    fn last(&self) -> i32 {
        self.first + self.values.len() as i32 - 1
    }
    fn at(&self, y: i32) -> f64 {
        self.values[(y - self.first) as usize]
    }
}

/// Monthly returns linked to calendar years (whole years only).
pub(crate) fn compound_years(first: YearMonth, monthly: &[f64]) -> Yearly {
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

/// Annual value from monthly values that are already annual rates (inflation: December against the previous year; SARON: mean
/// of the twelve months, `NaN` if a month is missing).
pub(crate) fn yearly_rates(first: YearMonth, monthly: &[f64], december_only: bool) -> Yearly {
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

/// Stocks of a region (countries weighted by GDP), from 1900.
fn region(jst: &JstData, countries: &[&str]) -> Result<Yearly> {
    Ok(Yearly {
        first: jst::FIRST_YEAR,
        values: jst.region_stocks(countries)?,
    })
}

/// «Welt individuell»: every year the regions with the fixed weights (rebalanced annually).
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

/// The series for the strategy. `lib`: folder of the data library (for newer Shiller and SNB data).
pub fn dataset(s: &Investment, lib: Option<&Path>) -> Result<DataSet> {
    dataset_with_parts(s, lib).map(|(ds, _)| ds)
}

/// #69/AK8: Stocks and bonds with their own years (inflation comes from the source of the stocks).
fn parts(s: &Investment, stocks: (i32, i32), bonds: (i32, i32)) -> Vec<SeriesPart> {
    vec![
        SeriesPart {
            label: msg!(investment_part_stocks, name = stock_name(s.stocks)),
            first: stocks.0,
            last: stocks.1,
        },
        SeriesPart {
            label: msg!(investment_part_bonds, name = bond_name(s.bonds)),
            first: bonds.0,
            last: bonds.1,
        },
    ]
}

/// Years of a series (first and last year with values).
fn years_of(ds: &DataSet) -> (i32, i32) {
    (ds.first.year(), ds.last().year())
}

/// The series for the strategy and its parts with their own years; the series covers the common years.
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
        ds.name = label(s).to_string();
        let own = years_of(&ds);
        // S&P 500 with Swiss Confederation bonds: the stocks go back further than the SNB yields.
        let shiller = if s.bonds == BondIndex::Ch {
            builtin_series("Shiller CHF.csv", lib)?
        } else {
            ds.clone()
        };
        let stocks = years_of(&shiller);
        // #100: whole stock series, monthly like the bootstrap of this series
        ds.stock_series = Some(StockSeries {
            freq: Freq::Monthly,
            first: shiller.first,
            values: shiller.stocks.clone(),
        });
        return Ok((ds, parts(s, stocks, own)));
    }

    let jst = JstData::builtin();
    let snb = match lib {
        Some(d) => library::snb_data(d)?,
        None => crate::import::SnbData::builtin(),
    };
    let shiller = builtin_series("Shiller CHF.csv", lib)?;

    let mut sources: Vec<Message> = vec![];
    let (stocks, inflation) = match s.stocks {
        StockIndex::Sp500 => {
            sources.push(shiller.meta.source.clone().unwrap_or_default());
            (
                yearly_of(&shiller, &shiller.stocks),
                yearly_rates(shiller.first, &shiller.inflation, true),
            )
        }
        x => {
            sources.push(msg!(series_source_jst));
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
                            .ok_or_else(|| invalid(msg!(ch_inflation_missing, year = y)))
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
            sources.push(msg!(
                series_source_snb,
                source = msg!(series_source_jst),
                cubes = "rendoblim"
            ));
            let (first, values) = jst::ch_bonds_yearly(&jst, &snb)?;
            Yearly { first, values }
        }
        BondIndex::ChShort => {
            sources.push(msg!(
                series_source_snb,
                source = msg!(series_source_jst),
                cubes = "zimoma (SARON)"
            ));
            let (first, values) = jst::ch_short_yearly(&jst, &snb)?;
            Yearly { first, values }
        }
    };
    let saron = yearly_rates(shiller.first, shiller.saron.as_deref().unwrap_or(&[]), false);

    let first = stocks.first.max(bonds.first).max(inflation.first);
    let last = stocks.last().min(bonds.last()).min(inflation.last());
    if last < first {
        return Err(invalid(msg!(strategy_no_common_years, strategy = label(s))));
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
        msg!(
            investment_note_parts,
            stocks = stock_name(s.stocks),
            bonds = bond_name(s.bonds)
        ),
        msg!(investment_note_saron),
    ];
    if s.bonds == BondIndex::ChShort {
        notes.push(msg!(investment_note_confed_short, year = jst.last_year()));
    }
    if matches!(
        s.stocks,
        StockIndex::ChJst | StockIndex::WorldJst | StockIndex::WorldCustom
    ) || matches!(s.bonds, BondIndex::Ch | BondIndex::ChShort)
    {
        notes.push(msg!(investment_note_jst_license));
    }
    let parts = parts(s, (stocks.first, stocks.last()), (bonds.first, bonds.last()));
    let source = sources
        .into_iter()
        .reduce(|a, b| msg!(series_source_list, source = a, other = b));
    let ds = DataSet {
        name: label(s).to_string(),
        meta: Meta {
            title: Some(label(s)),
            currency: Some("CHF".into()),
            source,
            info: Some(msg!(
                investment_info,
                stocks = stock_name(s.stocks),
                bonds = bond_name(s.bonds),
                first = first,
                last = last
            )),
            notes,
        },
        freq: Freq::Yearly,
        first: YearMonth::new(first, 1),
        stocks: years.clone().map(|y| stocks.at(y)).collect(),
        bonds: years.clone().map(|y| bonds.at(y)).collect(),
        inflation: years.clone().map(|y| inflation.at(y)).collect(),
        saron: Some(years.map(saron_at).collect()),
        stock_series: Some(StockSeries {
            freq: Freq::Yearly,
            first: YearMonth::new(stocks.first, 1),
            values: stocks.values.clone(),
        }),
    };
    Ok((ds, parts))
}

/// Series for a reference `strategy:<JSON>`; `None` if the reference is not a strategy.
pub fn from_reference(reference: &str, lib: Option<&Path>) -> Option<Result<DataSet>> {
    Some(from_reference_with_parts(reference, lib)?.map(|(ds, _)| ds))
}

/// Like [`from_reference`], with the parts of the strategy and their own years (#69/AK8).
pub fn from_reference_with_parts(reference: &str, lib: Option<&Path>) -> Option<Result<(DataSet, Vec<SeriesPart>)>> {
    let json = reference.strip_prefix(PREFIX)?;
    Some(
        serde_json::from_str::<Investment>(json)
            .map_err(|e| invalid(msg!(strategy_unreadable, detail = e.to_string())))
            .and_then(|s| dataset_with_parts(&s, lib)),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compound_years_takes_whole_calendar_years_only() {
        // Start in March: the first whole year starts in the following January.
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
        assert!(weight_issue(&w(1.2, -0.2, 0.0, 0.0)).unwrap().contains("between"));
        assert!(weight_issue(&w(f64::NAN, 1.0, 0.0, 0.0)).is_some());
    }

    #[test]
    fn labels() {
        let s = Investment::default();
        assert_eq!(label(&s), "Stocks World (JST), bonds Swiss Confederation 10 years");
        let c = Investment {
            stocks: StockIndex::WorldCustom,
            ..Default::default()
        };
        assert!(label(&c).contains("CH 20 %, USA 40 %, EU 30 %, Asia-Pacific 10 %"));
    }
}
