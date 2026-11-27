//! #67: Jordà-Schularick-Taylor Macrohistory Database (Release 6, annual values until 2020).
//!
//! From the country values, stock and bond returns in CHF (without currency hedging) are derived, and from them
//! GDP-weighted regions and a world stock index. The raw data (`data/quellen/JSTdatasetR6.xlsx`) is under
//! the licence CC BY-NC-SA 4.0; an extract of the required columns is bundled (`data/quellen/jst-r6.csv`).
//!
//! Computation (`docs/Weltindex JST.md`):
//! - Return in CHF: `(1 + r) × k(t) / k(t−1) − 1` with `k` = CHF per unit of local currency (from the rates against
//!   the USD, `xrusd`).
//! - Weight of a country in year `t`: real GDP of the previous year (GDP per capita after Maddison × population). The
//!   nominal GDP column is scaled differently per country and is not suitable for a comparison.
//! - Countries without a return in a year drop out in that year; the remaining weights add up to 100 % again.

use std::collections::BTreeMap;
use std::path::Path;

use calamine::{open_workbook_auto, Data, Reader};

use crate::import::{bond_return, SnbData};
use crate::market::{DataSet, Freq, Meta};
use crate::time::YearMonth;
use crate::{Error, Result};

/// Raw file of the database (folder `quellen` of the library).
pub const XLSX: &str = "JSTdatasetR6.xlsx";
/// Extract of the required columns, bundled with the program.
pub const EXTRACT: &str = "jst-r6.csv";
const BUILTIN: &str = include_str!("../data/quellen/jst-r6.csv");

/// Columns of the extract (after year and country).
const COLUMNS: [&str; 8] = [
    "pop",
    "rgdpmad",
    "xrusd",
    "cpi",
    "eq_tr",
    "bond_tr",
    "bond_rate",
    "bill_rate",
];

/// Values of a country in one year (missing = `None`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Row {
    pub pop: Option<f64>,
    pub rgdpmad: Option<f64>,
    /// Local currency per USD, end of year.
    pub xrusd: Option<f64>,
    pub cpi: Option<f64>,
    pub eq_tr: Option<f64>,
    pub bond_tr: Option<f64>,
    pub bond_rate: Option<f64>,
    /// #76: Return of short-term government paper (about 3 months), as a share.
    pub bill_rate: Option<f64>,
}

impl Row {
    fn get(&self, i: usize) -> Option<f64> {
        [
            self.pop,
            self.rgdpmad,
            self.xrusd,
            self.cpi,
            self.eq_tr,
            self.bond_tr,
            self.bond_rate,
            self.bill_rate,
        ][i]
    }
    fn set(&mut self, i: usize, v: Option<f64>) {
        let f = match i {
            0 => &mut self.pop,
            1 => &mut self.rgdpmad,
            2 => &mut self.xrusd,
            3 => &mut self.cpi,
            4 => &mut self.eq_tr,
            5 => &mut self.bond_tr,
            6 => &mut self.bond_rate,
            _ => &mut self.bill_rate,
        };
        *f = v;
    }
}

/// A region of countries (ISO codes as in the database).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Region {
    pub id: &'static str,
    pub name: &'static str,
    pub countries: &'static [&'static str],
}

pub const SWITZERLAND: Region = Region {
    id: "ch",
    name: "Schweiz",
    countries: &["CHE"],
};
pub const USA: Region = Region {
    id: "usa",
    name: "USA",
    countries: &["USA"],
};
/// «EU» according to #69: Germany, France, Italy, Spain, Netherlands, Belgium, United Kingdom.
pub const EU: Region = Region {
    id: "eu",
    name: "EU",
    countries: &["DEU", "FRA", "ITA", "ESP", "NLD", "BEL", "GBR"],
};
/// Asia-Pacific: Japan and Australia (the only countries of the region in the database).
pub const APAC: Region = Region {
    id: "apac",
    name: "Asien-Pazifik",
    countries: &["JPN", "AUS"],
};
/// Regions of the world index (#67).
pub const REGIONS: [Region; 4] = [SWITZERLAND, USA, EU, APAC];

/// First year of the world index: from 1900 all regions have stock returns.
pub const FIRST_YEAR: i32 = 1900;

/// Country-years that are not used.
pub const EXCLUDED: [(&str, i32, &str); 1] = [(
    "DEU",
    1923,
    "Hyperinflation: Der Wechselkurs der Datenbank passt im Jahr 1923 nicht zur Jahresrendite (in USD ergäben sich +830 %).",
)];

/// Which return.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Asset {
    Stocks,
    Bonds,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct JstData {
    rows: BTreeMap<(String, i32), Row>,
}

fn invalid(m: crate::message::Message) -> Error {
    Error::Invalid(m)
}

fn num(c: Option<&Data>) -> Option<f64> {
    match c? {
        Data::Float(v) => Some(*v),
        Data::Int(v) => Some(*v as f64),
        Data::String(s) => s.trim().parse().ok(),
        _ => None,
    }
    .filter(|v: &f64| v.is_finite())
}

fn all_countries() -> impl Iterator<Item = &'static str> {
    REGIONS.iter().flat_map(|r| r.countries.iter().copied())
}

impl JstData {
    /// Reads the table of the database (`JSTdatasetR6.xlsx`, first sheet).
    pub fn read_xlsx(path: &Path) -> Result<JstData> {
        let mut wb = open_workbook_auto(path).map_err(|e| crate::io_error(path, e))?;
        let range = wb
            .worksheet_range_at(0)
            .ok_or_else(|| invalid(msg!(jst_no_sheet, path = path.display().to_string())))?
            .map_err(|e| crate::file_invalid(path, e.to_string()))?;
        let rows: Vec<Vec<Data>> = range.rows().map(|r| r.to_vec()).collect();
        JstData::from_rows(&rows)
    }

    /// First row: column names (`year`, `iso` and the columns of the extract). Only the countries of the regions.
    pub fn from_rows(rows: &[Vec<Data>]) -> Result<JstData> {
        let header: Vec<String> = rows
            .first()
            .ok_or_else(|| invalid(msg!(jst_table_empty)))?
            .iter()
            .map(|c| match c {
                Data::String(s) => s.trim().to_string(),
                _ => String::new(),
            })
            .collect();
        let col = |name: &str| {
            header
                .iter()
                .position(|h| h == name)
                .ok_or_else(|| invalid(msg!(jst_column_missing, column = name)))
        };
        let (c_year, c_iso) = (col("year")?, col("iso")?);
        let cols: Vec<usize> = COLUMNS.iter().map(|n| col(n)).collect::<Result<_>>()?;
        let mut d = JstData::default();
        for r in &rows[1..] {
            let iso = match r.get(c_iso) {
                Some(Data::String(s)) => s.trim(),
                _ => continue,
            };
            if !all_countries().any(|c| c == iso) {
                continue;
            }
            let Some(year) = num(r.get(c_year)).map(|y| y as i32) else {
                continue;
            };
            let mut row = Row::default();
            for (i, &c) in cols.iter().enumerate() {
                row.set(i, num(r.get(c)));
            }
            d.rows.insert((iso.to_string(), year), row);
        }
        if d.rows.is_empty() {
            return Err(invalid(msg!(jst_no_region_values)));
        }
        Ok(d)
    }

    /// Extract in CSV format (`year,iso,pop,…`), empty cells for missing values.
    pub fn to_csv(&self) -> String {
        let mut out = format!("year,iso,{}\n", COLUMNS.join(","));
        for ((iso, year), row) in &self.rows {
            out.push_str(&format!("{year},{iso}"));
            for i in 0..COLUMNS.len() {
                out.push(',');
                if let Some(v) = row.get(i) {
                    out.push_str(&format!("{v}"));
                }
            }
            out.push('\n');
        }
        out
    }

    pub fn parse_csv(text: &str) -> Result<JstData> {
        let mut lines = text.lines().filter(|l| !l.trim().is_empty());
        let header = lines.next().ok_or_else(|| invalid(msg!(jst_extract_empty)))?;
        let expected = format!("year,iso,{}", COLUMNS.join(","));
        if header.trim() != expected {
            return Err(invalid(msg!(jst_extract_header, header = header, expected = expected)));
        }
        let mut d = JstData::default();
        for (n, line) in lines.enumerate() {
            let cells: Vec<&str> = line.split(',').map(str::trim).collect();
            let bad = || invalid(msg!(jst_extract_line, row = n + 2, line = line));
            if cells.len() != COLUMNS.len() + 2 {
                return Err(bad());
            }
            let year: i32 = cells[0].parse().map_err(|_| bad())?;
            let mut row = Row::default();
            for i in 0..COLUMNS.len() {
                let s = cells[i + 2];
                row.set(
                    i,
                    if s.is_empty() {
                        None
                    } else {
                        Some(s.parse().map_err(|_| bad())?)
                    },
                );
            }
            d.rows.insert((cells[1].to_string(), year), row);
        }
        Ok(d)
    }

    /// The bundled extract.
    pub fn builtin() -> JstData {
        JstData::parse_csv(BUILTIN).expect("ausgelieferter JST-Auszug ist gültig")
    }

    pub fn row(&self, iso: &str, year: i32) -> Option<&Row> {
        self.rows.get(&(iso.to_string(), year))
    }

    /// Last year with values.
    pub fn last_year(&self) -> i32 {
        self.rows.keys().map(|(_, y)| *y).max().unwrap_or(FIRST_YEAR)
    }

    /// CHF per unit of local currency, end of year.
    fn chf_per_lcu(&self, iso: &str, year: i32) -> Option<f64> {
        Some(self.row("CHE", year)?.xrusd? / self.row(iso, year)?.xrusd?)
    }

    /// Annual return of a country in CHF (without currency hedging); `None` for missing data or an
    /// excluded country-year ([`EXCLUDED`]).
    pub fn chf_return(&self, iso: &str, year: i32, asset: Asset) -> Option<f64> {
        if EXCLUDED.iter().any(|(c, y, _)| *c == iso && *y == year) {
            return None;
        }
        let row = self.row(iso, year)?;
        let r = match asset {
            Asset::Stocks => row.eq_tr?,
            Asset::Bonds => row.bond_tr?,
        };
        Some((1.0 + r) * self.chf_per_lcu(iso, year)? / self.chf_per_lcu(iso, year - 1)? - 1.0)
    }

    /// Weight of a country in year `year`: real GDP of the previous year (GDP per capita after Maddison × population).
    pub fn gdp_weight(&self, iso: &str, year: i32) -> Option<f64> {
        let r = self.row(iso, year - 1)?;
        Some(r.rgdpmad? * r.pop?).filter(|w| *w > 0.0)
    }

    /// GDP-weighted return in CHF over the countries with values; `None` if none has values.
    pub fn weighted(&self, countries: &[&str], year: i32, asset: Asset) -> Option<f64> {
        let (mut sum, mut weights) = (0.0, 0.0);
        for c in countries {
            if let (Some(r), Some(w)) = (self.chf_return(c, year, asset), self.gdp_weight(c, year)) {
                sum += w * r;
                weights += w;
            }
        }
        (weights > 0.0).then(|| sum / weights)
    }

    /// Weights (shares, sum 1) of the countries with values in year `year`.
    pub fn weights(&self, countries: &[&str], year: i32, asset: Asset) -> Vec<(String, f64)> {
        let w: Vec<(String, f64)> = countries
            .iter()
            .filter(|c| self.chf_return(c, year, asset).is_some())
            .filter_map(|c| Some((c.to_string(), self.gdp_weight(c, year)?)))
            .collect();
        let total: f64 = w.iter().map(|(_, v)| v).sum();
        w.into_iter().map(|(c, v)| (c, v / total)).collect()
    }

    /// Country-years without a return in the period (they drop out of the weighting).
    pub fn gaps(&self, countries: &[&str], from: i32, to: i32, asset: Asset) -> Vec<(String, i32)> {
        (from..=to)
            .flat_map(|y| countries.iter().map(move |c| (c.to_string(), y)))
            .filter(|(c, y)| self.chf_return(c, *y, asset).is_none() || self.gdp_weight(c, *y).is_none())
            .collect()
    }

    /// Stock returns of a region in CHF, year by year from [`FIRST_YEAR`] to the last year of the data.
    pub fn region_stocks(&self, countries: &[&str]) -> Result<Vec<f64>> {
        (FIRST_YEAR..=self.last_year())
            .map(|y| {
                self.weighted(countries, y, Asset::Stocks)
                    .ok_or_else(|| invalid(msg!(jst_no_stock_return, countries = countries, year = y)))
            })
            .collect()
    }

    /// Inflation in Switzerland (consumer prices, annual average against the previous year).
    pub fn ch_inflation(&self, year: i32) -> Option<f64> {
        Some(self.row("CHE", year)?.cpi? / self.row("CHE", year - 1)?.cpi? - 1.0)
    }
}

/// First year for which the SNB yields cover a whole calendar year (December of the previous year to December).
fn snb_first_full_year(snb: &SnbData) -> Option<i32> {
    // A whole year needs the yield of December of the previous year.
    Some(snb.ch_yield10.keys().next()?.year() + 1)
}

/// #68: Total return of 10-year Swiss Confederation bonds per calendar year: from JST until the year before the
/// first whole SNB year, afterwards from the monthly SNB yields (monthly returns linked to the year). 1915 is missing in
/// JST (exchange closed); there the coupon (`bond_rate`) counts as the return. Returns the first year and the values.
pub fn ch_bonds_yearly(jst: &JstData, snb: &SnbData) -> Result<(i32, Vec<f64>)> {
    let switch = snb_first_full_year(snb).ok_or_else(|| invalid(msg!(snb_bond_yields_missing)))?;
    let y = &snb.ch_yield10;
    let mut out = vec![];
    for year in FIRST_YEAR.. {
        let v = if year < switch {
            let row = jst
                .row("CHE", year)
                .ok_or_else(|| invalid(msg!(jst_ch_year_missing, year = year)))?;
            row.bond_tr
                .or(row.bond_rate)
                .ok_or_else(|| invalid(msg!(jst_ch_bonds_missing, year = year)))?
        } else {
            let months: Option<Vec<f64>> = (1..=12)
                .map(|m| {
                    let ym = YearMonth::new(year, m);
                    Some(bond_return(y.get(&ym.plus(-1))? / 100.0, y.get(&ym)? / 100.0))
                })
                .collect();
            match months {
                Some(ms) => ms.iter().map(|r| 1.0 + r).product::<f64>() - 1.0,
                None => break,
            }
        };
        out.push(v);
    }
    Ok((FIRST_YEAR, out))
}

/// #76: Return of short-term Swiss Confederation paper (about 3 months) per calendar year: from JST (`bill_rate`)
/// from [`FIRST_YEAR`] to the last JST year, afterwards the annual average of SARON (SNB, whole years only). With such a short
/// term the return equals the rate; price fluctuations drop out. Returns the first year and the values.
pub fn ch_short_yearly(jst: &JstData, snb: &SnbData) -> Result<(i32, Vec<f64>)> {
    let switch = jst.last_year() + 1;
    let mut out = vec![];
    for year in FIRST_YEAR..switch {
        out.push(
            jst.row("CHE", year)
                .and_then(|r| r.bill_rate)
                .ok_or_else(|| invalid(msg!(jst_ch_short_rates_missing, year = year)))?,
        );
    }
    for year in switch.. {
        let months: Option<Vec<f64>> = (1..=12)
            .map(|m| snb.saron.get(&YearMonth::new(year, m)).copied())
            .collect();
        match months {
            Some(ms) => out.push(ms.iter().sum::<f64>() / 12.0 / 100.0),
            None => break,
        }
    }
    Ok((FIRST_YEAR, out))
}

/// #68: Annual series «Welt (JST)» in CHF: stocks = world index (all regions, GDP-weighted), bonds =
/// 10-year Swiss Confederation bonds ([`ch_bonds_yearly`]), inflation = Swiss consumer prices (JST).
pub fn world_dataset(jst: &JstData, snb: &SnbData) -> Result<DataSet> {
    let countries: Vec<&str> = all_countries().collect();
    let stocks = jst.region_stocks(&countries)?;
    let (_, bonds) = ch_bonds_yearly(jst, snb)?;
    let n = stocks.len().min(bonds.len());
    let inflation: Vec<f64> = (0..n)
        .map(|i| {
            let y = FIRST_YEAR + i as i32;
            jst.ch_inflation(y)
                .ok_or_else(|| invalid(msg!(jst_ch_inflation_missing, year = y)))
        })
        .collect::<Result<_>>()?;
    // Source reference: citations according to macrohistory.net (#177: code series_source_jst).
    let mut source = msg!(series_source_snb, source = msg!(series_source_jst), cubes = "rendoblim");
    if let Some(p) = snb.published.get("rendoblim") {
        source = msg!(series_source_as_of, source = source, dates = p);
    }
    Ok(DataSet {
        name: "Welt (JST)".into(),
        meta: Meta {
            title: Some(msg!(series_title_world)),
            currency: Some("CHF".into()),
            source: Some(source),
            info: Some(msg!(series_info_world)),
            notes: vec![
                msg!(series_note_world_stocks),
                msg!(
                    series_note_world_bonds,
                    year = snb_first_full_year(snb).unwrap_or(FIRST_YEAR) - 1
                ),
                msg!(series_note_world_license),
            ],
        },
        freq: Freq::Yearly,
        first: YearMonth::new(FIRST_YEAR, 1),
        stocks: stocks[..n].to_vec(),
        bonds: bonds[..n].to_vec(),
        inflation,
        saron: None,
        yield10: None,
        stock_series: None,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn data(lines: &str) -> JstData {
        JstData::parse_csv(&format!("year,iso,{}\n{lines}", COLUMNS.join(","))).unwrap()
    }

    #[test]
    fn chf_return_converts_with_year_end_rates() {
        // CHF 2 per USD → 2.2; country: 4 per USD → 4 (CHF per unit 0.5 → 0.55, +10 %).
        let d = data(
            "1999,CHE,1,1,2,100,0,0,,\n2000,CHE,1,1,2.2,102,0.05,0.01,,\n1999,USA,1,1,4,,,,,\n2000,USA,1,1,4,,0.2,,,\n",
        );
        assert!((d.chf_return("USA", 2000, Asset::Stocks).unwrap() - (1.2 * 0.55 / 0.5 - 1.0)).abs() < 1e-12);
        assert!((d.chf_return("CHE", 2000, Asset::Stocks).unwrap() - 0.05).abs() < 1e-12);
        assert!(d.chf_return("USA", 2000, Asset::Bonds).is_none());
        assert!((d.ch_inflation(2000).unwrap() - 0.02).abs() < 1e-12);
    }

    #[test]
    fn weights_use_real_gdp_of_the_previous_year_and_skip_missing_countries() {
        // Previous year: CHE 1 × 10 = 10, USA 3 × 10 = 30 → 25 % / 75 %
        let d = data(
            "1999,CHE,10,1,1,,,,,\n2000,CHE,99,99,1,,0.1,,,\n1999,USA,10,3,1,,,,,\n2000,USA,1,1,1,,0.3,,,\n1999,DEU,10,5,1,,,,,\n2000,DEU,1,1,1,,,,,\n",
        );
        let r = d.weighted(&["CHE", "USA", "DEU"], 2000, Asset::Stocks).unwrap();
        assert!((r - (0.25 * 0.1 + 0.75 * 0.3)).abs() < 1e-12, "{r}");
        assert_eq!(d.weights(&["CHE", "USA", "DEU"], 2000, Asset::Stocks).len(), 2);
        assert_eq!(
            d.gaps(&["CHE", "DEU"], 2000, 2000, Asset::Stocks),
            vec![("DEU".to_string(), 2000)]
        );
        assert!(d.weighted(&["DEU"], 2000, Asset::Stocks).is_none());
    }

    #[test]
    fn excluded_country_years_have_no_return() {
        let d = data("1922,CHE,1,1,1,,,,,\n1923,CHE,1,1,1,,,,,\n1922,DEU,1,1,1,,,,,\n1923,DEU,1,1,1,,5,,,\n");
        assert!(d.chf_return("DEU", 1923, Asset::Stocks).is_none());
    }

    #[test]
    fn csv_round_trip_and_errors() {
        let d = data("2000,CHE,1.5,2,,,,0.01,0.04,0.02\n");
        assert_eq!(JstData::parse_csv(&d.to_csv()).unwrap(), d);
        assert!(JstData::parse_csv("").is_err());
        assert!(JstData::parse_csv("year,iso\n").is_err());
        assert!(JstData::parse_csv(&format!("year,iso,{}\n2000,CHE,x,,,,,,,\n", COLUMNS.join(","))).is_err());
        assert!(JstData::parse_csv(&format!("year,iso,{}\n2000,CHE\n", COLUMNS.join(","))).is_err());
    }

    fn s(t: &str) -> Data {
        Data::String(t.to_string())
    }

    fn header() -> Vec<Data> {
        let mut h = vec![s("year"), s("iso")];
        h.extend(COLUMNS.iter().map(|c| s(c)));
        h
    }

    /// Row of the raw table: year, country and the eight values.
    fn raw_row(year: Data, iso: Data, values: [Data; 8]) -> Vec<Data> {
        let mut r = vec![year, iso];
        r.extend(values);
        r
    }

    fn empty8() -> [Data; 8] {
        std::array::from_fn(|_| Data::Empty)
    }

    #[test]
    fn from_rows_reads_numbers_in_all_cell_types_and_skips_other_rows() {
        let mut vals = empty8();
        vals[0] = Data::Float(1.5); // pop
        vals[1] = Data::Int(7); // rgdpmad
        vals[2] = s(" 2.5 "); // xrusd as text
        vals[3] = s("keine Zahl"); // cpi: unreadable text → missing
        vals[4] = Data::Float(f64::NAN); // eq_tr: not finite → missing
        vals[5] = Data::Bool(true); // bond_tr: other cell type → missing

        // Order: main row, year as integer and country with spaces, country outside the regions,
        // country not text, without year, year unreadable, short row without country
        let rows = vec![
            header(),
            raw_row(Data::Float(2000.0), s("CHE"), vals),
            raw_row(Data::Int(2001), s(" USA "), empty8()),
            raw_row(Data::Float(2000.0), s("XXX"), empty8()),
            raw_row(Data::Float(2000.0), Data::Float(1.0), empty8()),
            raw_row(Data::Empty, s("DEU"), empty8()),
            raw_row(s("abc"), s("FRA"), empty8()),
            vec![Data::Float(2000.0)],
        ];
        let d = JstData::from_rows(&rows).unwrap();
        let r = d.row("CHE", 2000).unwrap();
        assert_eq!(r.pop, Some(1.5));
        assert_eq!(r.rgdpmad, Some(7.0));
        assert_eq!(r.xrusd, Some(2.5));
        assert_eq!((r.cpi, r.eq_tr, r.bond_tr, r.bond_rate), (None, None, None, None));
        assert_eq!(d.row("USA", 2001), Some(&Row::default()));
        assert_eq!(d.last_year(), 2001);
        assert!(d.row("XXX", 2000).is_none() && d.row("DEU", 2000).is_none() && d.row("FRA", 2000).is_none());
        // Text in the year is read as a number
        let rows = vec![header(), raw_row(s("1999"), s("CHE"), empty8())];
        assert!(JstData::from_rows(&rows).unwrap().row("CHE", 1999).is_some());
    }

    #[test]
    fn from_rows_reports_empty_tables_missing_columns_and_missing_countries() {
        let msg = |r: Result<JstData>| match r {
            Err(Error::Invalid(m)) => m,
            other => panic!("Fehler erwartet: {other:?}"),
        };
        assert!(msg(JstData::from_rows(&[])).contains("empty"));
        let mut no_iso = header();
        no_iso.remove(1);
        assert!(msg(JstData::from_rows(&[no_iso])).contains("“iso”"));
        let mut no_bill = header();
        no_bill.pop();
        assert!(msg(JstData::from_rows(&[no_bill])).contains("“bill_rate”"));
        // Only header row or only foreign countries
        assert!(msg(JstData::from_rows(&[header()])).contains("no values"));
        let other = vec![header(), raw_row(Data::Int(2000), s("XXX"), empty8())];
        assert!(msg(JstData::from_rows(&other)).contains("no values"));
        // Header cells that are not text count as unnamed
        let mut odd = header();
        odd[0] = Data::Int(5);
        assert!(msg(JstData::from_rows(&[odd])).contains("“year”"));
    }

    #[test]
    fn last_year_of_empty_data_is_the_first_year() {
        assert_eq!(JstData::default().last_year(), FIRST_YEAR);
    }

    #[test]
    fn region_stocks_name_the_missing_year() {
        // Only 1900 has a return; 1901 is missing → error with country and year
        let d = data("1899,CHE,1,1,1,,,,,\n1900,CHE,1,1,1,,0.1,,,\n1901,CHE,1,1,1,,,,,\n");
        match d.region_stocks(&["CHE"]) {
            Err(Error::Invalid(m)) => assert!(m.contains("CHE") && m.contains("1901"), "{m}"),
            other => panic!("Fehler erwartet: {other:?}"),
        }
        // Without a gap: one value per year from FIRST_YEAR to the last year
        let d = data("1899,CHE,1,1,1,,,,,\n1900,CHE,1,1,1,,0.1,,,\n1901,CHE,1,1,1,,0.2,,,\n");
        assert_eq!(d.region_stocks(&["CHE"]).unwrap().len(), 2);
    }

    fn months(snb: &mut SnbData, from: (i32, u32), to: (i32, u32), y: f64) {
        let mut ym = YearMonth::new(from.0, from.1);
        while ym <= YearMonth::new(to.0, to.1) {
            snb.ch_yield10.insert(ym, y);
            ym = ym.plus(1);
        }
    }

    #[test]
    fn ch_bonds_use_jst_before_and_snb_months_from_the_switch_year() {
        // SNB from January 1901 → first whole year 1902; 1900 and 1901 from JST (1901 without bond_tr → coupon)
        let d = data("1900,CHE,1,1,1,,,0.07,0.03,\n1901,CHE,1,1,1,,,,0.04,\n");
        let mut snb = SnbData::default();
        months(&mut snb, (1901, 1), (1902, 12), 2.0);
        let (first, v) = ch_bonds_yearly(&d, &snb).unwrap();
        assert_eq!((first, v.len()), (1900, 3));
        assert_eq!((v[0], v[1]), (0.07, 0.04));
        let monthly = 1.0 + bond_return(0.02, 0.02);
        assert!((v[2] - (monthly.powi(12) - 1.0)).abs() < 1e-12, "{}", v[2]);
        // The rates are given in percent: 2 % → 0.02, not 2
        assert!(v[2].abs() < 0.5);
    }

    #[test]
    fn ch_bonds_report_missing_snb_data_and_missing_jst_years() {
        let d = data("1900,CHE,1,1,1,,,0.07,0.03,\n");
        let msg = |r: Result<(i32, Vec<f64>)>| match r {
            Err(Error::Invalid(m)) => m,
            other => panic!("Fehler erwartet: {other:?}"),
        };
        assert!(msg(ch_bonds_yearly(&d, &SnbData::default())).contains("SNB"));
        let mut snb = SnbData::default();
        months(&mut snb, (1902, 1), (1903, 12), 2.0); // switch 1903; JST year 1901 missing
        assert!(msg(ch_bonds_yearly(&d, &snb)).contains("1901"));
        let d = data("1900,CHE,1,1,1,,,,,\n"); // neither bond_tr nor bond_rate
        let mut snb = SnbData::default();
        months(&mut snb, (1901, 1), (1901, 12), 2.0); // switch 1902
        assert!(msg(ch_bonds_yearly(&d, &snb)).contains("Swiss bonds 1900"));
    }

    #[test]
    fn ch_short_uses_bill_rate_then_the_yearly_mean_of_saron_in_percent() {
        let d = data("1900,CHE,1,1,1,,,,,0.03\n1901,CHE,1,1,1,,,,,0.04\n");
        let mut snb = SnbData::default();
        // 1902: annual average (6 × 1 + 6 × 3) / 12 = 2 % → 0.02; 1903 incomplete → end
        for m in 1..=12 {
            snb.saron
                .insert(YearMonth::new(1902, m), if m <= 6 { 1.0 } else { 3.0 });
        }
        snb.saron.insert(YearMonth::new(1903, 1), 9.0);
        let (first, v) = ch_short_yearly(&d, &snb).unwrap();
        assert_eq!(first, 1900);
        assert_eq!(v.len(), 3);
        assert_eq!((v[0], v[1]), (0.03, 0.04));
        assert!((v[2] - 0.02).abs() < 1e-12, "{}", v[2]);
        // If bill_rate is missing in a JST year, the error names the year
        let d = data("1900,CHE,1,1,1,,,,,0.03\n1901,CHE,1,1,1,,,,,\n");
        match ch_short_yearly(&d, &snb) {
            Err(Error::Invalid(m)) => assert!(m.contains("1901"), "{m}"),
            other => panic!("Fehler erwartet: {other:?}"),
        }
    }
}
