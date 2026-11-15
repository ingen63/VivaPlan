//! FEA-51: Jordà-Schularick-Taylor Macrohistory Database (Release 6, Jahreswerte bis 2020).
//!
//! Aus den Länderwerten entstehen Aktien- und Obligationenrenditen in CHF (ohne Währungsabsicherung) und daraus nach
//! BIP gewichtete Regionen und ein Welt-Aktienindex. Die Rohdaten (`data/quellen/JSTdatasetR6.xlsx`) stehen unter
//! der Lizenz CC BY-NC-SA 4.0; ausgeliefert wird ein Auszug der benötigten Spalten (`data/quellen/jst-r6.csv`).
//!
//! Rechenweg (`docs/FEA-51 FEA-52 Weltindex JST.md`):
//! - Rendite in CHF: `(1 + r) × k(t) / k(t−1) − 1` mit `k` = CHF je Einheit Landeswährung (aus den Kursen gegenüber
//!   dem USD, `xrusd`).
//! - Gewicht eines Landes im Jahr `t`: reales BIP des Vorjahres (BIP pro Kopf nach Maddison × Bevölkerung). Die
//!   nominale BIP-Spalte ist je Land unterschiedlich skaliert und taugt nicht für einen Vergleich.
//! - Länder ohne Rendite in einem Jahr fallen in diesem Jahr weg; die übrigen Gewichte ergeben wieder 100 %.

use std::collections::BTreeMap;
use std::path::Path;

use calamine::{open_workbook_auto, Data, Reader};

use crate::import::{bond_return, SnbData};
use crate::market::{DataSet, Freq, Meta};
use crate::time::YearMonth;
use crate::{Error, Result};

/// Rohdatei der Datenbank (Ordner `quellen` der Bibliothek).
pub const XLSX: &str = "JSTdatasetR6.xlsx";
/// Auszug der benötigten Spalten, mit dem Programm ausgeliefert.
pub const EXTRACT: &str = "jst-r6.csv";
const BUILTIN: &str = include_str!("../data/quellen/jst-r6.csv");

/// Spalten des Auszugs (nach Jahr und Land).
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

/// Quellenangabe (Zitate gemäss macrohistory.net).
pub const SOURCE: &str = "Jordà-Schularick-Taylor Macrohistory Database, Release 6 (macrohistory.net/database, abgerufen 13.11.2026), Lizenz CC BY-NC-SA 4.0. Zitate: Jordà, Schularick, Taylor (2017), «Macrofinancial History and the New Business Cycle Facts», NBER Macroeconomics Annual 2016; Jordà, Knoll, Kuvshinov, Schularick, Taylor (2019), «The Rate of Return on Everything, 1870–2015», Quarterly Journal of Economics";

/// Werte eines Landes in einem Jahr (fehlend = `None`).
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Row {
    pub pop: Option<f64>,
    pub rgdpmad: Option<f64>,
    /// Landeswährung je USD, Ende Jahr.
    pub xrusd: Option<f64>,
    pub cpi: Option<f64>,
    pub eq_tr: Option<f64>,
    pub bond_tr: Option<f64>,
    pub bond_rate: Option<f64>,
    /// FEA-59: Rendite kurzfristiger Staatspapiere (etwa 3 Monate), als Anteil.
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

/// Eine Region aus Ländern (ISO-Codes wie in der Datenbank).
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
/// «EU» gemäss FEA-53: Deutschland, Frankreich, Italien, Spanien, Niederlande, Belgien, Grossbritannien.
pub const EU: Region = Region {
    id: "eu",
    name: "EU",
    countries: &["DEU", "FRA", "ITA", "ESP", "NLD", "BEL", "GBR"],
};
/// Asien-Pazifik: Japan und Australien (die einzigen Länder der Region in der Datenbank).
pub const APAC: Region = Region {
    id: "apac",
    name: "Asien-Pazifik",
    countries: &["JPN", "AUS"],
};
/// Regionen des Weltindexes (FEA-51).
pub const REGIONS: [Region; 4] = [SWITZERLAND, USA, EU, APAC];

/// Erstes Jahr des Weltindexes: ab 1900 haben alle Regionen Aktienrenditen.
pub const FIRST_YEAR: i32 = 1900;

/// Länderjahre, die nicht verwendet werden.
pub const EXCLUDED: [(&str, i32, &str); 1] = [(
    "DEU",
    1923,
    "Hyperinflation: Der Wechselkurs der Datenbank passt im Jahr 1923 nicht zur Jahresrendite (in USD ergäben sich +830 %).",
)];

/// Welche Rendite.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Asset {
    Stocks,
    Bonds,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct JstData {
    rows: BTreeMap<(String, i32), Row>,
}

fn invalid(s: impl Into<String>) -> Error {
    Error::Invalid(s.into())
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
    /// Liest die Tabelle der Datenbank (`JSTdatasetR6.xlsx`, erstes Blatt).
    pub fn read_xlsx(path: &Path) -> Result<JstData> {
        let mut wb = open_workbook_auto(path).map_err(|e| Error::Io(format!("{}: {e}", path.display())))?;
        let range = wb
            .worksheet_range_at(0)
            .ok_or_else(|| invalid(format!("{}: kein Tabellenblatt", path.display())))?
            .map_err(|e| invalid(format!("{}: {e}", path.display())))?;
        let rows: Vec<Vec<Data>> = range.rows().map(|r| r.to_vec()).collect();
        JstData::from_rows(&rows)
    }

    /// Erste Zeile: Spaltennamen (`year`, `iso` und die Spalten des Auszugs). Nur die Länder der Regionen.
    pub fn from_rows(rows: &[Vec<Data>]) -> Result<JstData> {
        let header: Vec<String> = rows
            .first()
            .ok_or_else(|| invalid("JST-Daten: Tabelle ist leer"))?
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
                .ok_or_else(|| invalid(format!("JST-Daten: Spalte «{name}» fehlt")))
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
            return Err(invalid("JST-Daten: keine Werte für die Länder der Regionen"));
        }
        Ok(d)
    }

    /// Auszug im CSV-Format (`year,iso,pop,…`), leere Zellen für fehlende Werte.
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
        let header = lines.next().ok_or_else(|| invalid("JST-Auszug ist leer"))?;
        let expected = format!("year,iso,{}", COLUMNS.join(","));
        if header.trim() != expected {
            return Err(invalid(format!("JST-Auszug: Kopfzeile «{header}» statt «{expected}»")));
        }
        let mut d = JstData::default();
        for (n, line) in lines.enumerate() {
            let cells: Vec<&str> = line.split(',').map(str::trim).collect();
            let bad = || invalid(format!("JST-Auszug, Zeile {}: «{line}»", n + 2));
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

    /// Der ausgelieferte Auszug.
    pub fn builtin() -> JstData {
        JstData::parse_csv(BUILTIN).expect("ausgelieferter JST-Auszug ist gültig")
    }

    pub fn row(&self, iso: &str, year: i32) -> Option<&Row> {
        self.rows.get(&(iso.to_string(), year))
    }

    /// Letztes Jahr mit Werten.
    pub fn last_year(&self) -> i32 {
        self.rows.keys().map(|(_, y)| *y).max().unwrap_or(FIRST_YEAR)
    }

    /// CHF je Einheit Landeswährung, Ende Jahr.
    fn chf_per_lcu(&self, iso: &str, year: i32) -> Option<f64> {
        Some(self.row("CHE", year)?.xrusd? / self.row(iso, year)?.xrusd?)
    }

    /// Jahresrendite eines Landes in CHF (ohne Währungsabsicherung); `None` bei fehlenden Angaben oder einem
    /// ausgeschlossenen Länderjahr ([`EXCLUDED`]).
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

    /// Gewicht eines Landes im Jahr `year`: reales BIP des Vorjahres (BIP pro Kopf nach Maddison × Bevölkerung).
    pub fn gdp_weight(&self, iso: &str, year: i32) -> Option<f64> {
        let r = self.row(iso, year - 1)?;
        Some(r.rgdpmad? * r.pop?).filter(|w| *w > 0.0)
    }

    /// Nach BIP gewichtete Rendite in CHF über die Länder mit Werten; `None`, wenn keines Werte hat.
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

    /// Gewichte (Anteile, Summe 1) der Länder mit Werten im Jahr `year`.
    pub fn weights(&self, countries: &[&str], year: i32, asset: Asset) -> Vec<(String, f64)> {
        let w: Vec<(String, f64)> = countries
            .iter()
            .filter(|c| self.chf_return(c, year, asset).is_some())
            .filter_map(|c| Some((c.to_string(), self.gdp_weight(c, year)?)))
            .collect();
        let total: f64 = w.iter().map(|(_, v)| v).sum();
        w.into_iter().map(|(c, v)| (c, v / total)).collect()
    }

    /// Länderjahre ohne Rendite im Zeitraum (sie fallen aus der Gewichtung).
    pub fn gaps(&self, countries: &[&str], from: i32, to: i32, asset: Asset) -> Vec<(String, i32)> {
        (from..=to)
            .flat_map(|y| countries.iter().map(move |c| (c.to_string(), y)))
            .filter(|(c, y)| self.chf_return(c, *y, asset).is_none() || self.gdp_weight(c, *y).is_none())
            .collect()
    }

    /// Aktienrenditen einer Region in CHF, Jahr für Jahr ab [`FIRST_YEAR`] bis zum letzten Jahr der Daten.
    pub fn region_stocks(&self, countries: &[&str]) -> Result<Vec<f64>> {
        (FIRST_YEAR..=self.last_year())
            .map(|y| {
                self.weighted(countries, y, Asset::Stocks).ok_or_else(|| {
                    invalid(format!(
                        "JST-Daten: keine Aktienrendite für {} im Jahr {y}",
                        countries.join(", ")
                    ))
                })
            })
            .collect()
    }

    /// Teuerung in der Schweiz (Konsumentenpreise, Jahresmittel gegenüber Vorjahr).
    pub fn ch_inflation(&self, year: i32) -> Option<f64> {
        Some(self.row("CHE", year)?.cpi? / self.row("CHE", year - 1)?.cpi? - 1.0)
    }
}

/// Erstes Jahr, für das die SNB-Renditen ein ganzes Kalenderjahr abdecken (Dezember des Vorjahres bis Dezember).
fn snb_first_full_year(snb: &SnbData) -> Option<i32> {
    // Ein ganzes Jahr braucht die Rendite vom Dezember des Vorjahres.
    Some(snb.ch_yield10.keys().next()?.year() + 1)
}

/// FEA-52: Gesamtrendite 10-jähriger Obligationen der Eidgenossenschaft pro Kalenderjahr: aus JST bis zum Jahr vor dem
/// ersten ganzen SNB-Jahr, danach aus den monatlichen SNB-Renditen (Monatsrenditen zum Jahr verknüpft). 1915 fehlt in
/// JST (Börse geschlossen); dort gilt der Coupon (`bond_rate`) als Rendite. Liefert das erste Jahr und die Werte.
pub fn ch_bonds_yearly(jst: &JstData, snb: &SnbData) -> Result<(i32, Vec<f64>)> {
    let switch = snb_first_full_year(snb).ok_or_else(|| invalid("SNB-Renditen der Eidgenossenschaft fehlen"))?;
    let y = &snb.ch_yield10;
    let mut out = vec![];
    for year in FIRST_YEAR.. {
        let v = if year < switch {
            let row = jst
                .row("CHE", year)
                .ok_or_else(|| invalid(format!("JST-Daten: Schweiz {year} fehlt")))?;
            row.bond_tr
                .or(row.bond_rate)
                .ok_or_else(|| invalid(format!("JST-Daten: Obligationen Schweiz {year} fehlen")))?
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

/// FEA-59: Rendite kurzfristiger Papiere der Eidgenossenschaft (etwa 3 Monate) pro Kalenderjahr: aus JST (`bill_rate`)
/// ab [`FIRST_YEAR`] bis zum letzten JST-Jahr, danach das Jahresmittel des SARON (SNB, nur ganze Jahre). Bei so kurzer
/// Laufzeit entspricht die Rendite dem Satz; Kursschwankungen fallen weg. Liefert das erste Jahr und die Werte.
pub fn ch_short_yearly(jst: &JstData, snb: &SnbData) -> Result<(i32, Vec<f64>)> {
    let switch = jst.last_year() + 1;
    let mut out = vec![];
    for year in FIRST_YEAR..switch {
        out.push(
            jst.row("CHE", year)
                .and_then(|r| r.bill_rate)
                .ok_or_else(|| invalid(format!("JST-Daten: kurzfristige Zinsen Schweiz {year} fehlen")))?,
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

/// Erklärung der Reihe «Welt (JST)» (höchstens 2 Zeilen).
pub const INFO_WORLD: &str = "Welt-Aktienindex aus den JST-Daten (CH, USA, EU, Japan, Australien), nach BIP gewichtet, in CHF ohne Währungsabsicherung. Obligationen: Eidgenossenschaft 10 Jahre. Teuerung: CH.";

/// FEA-52: Jahresreihe «Welt (JST)» in CHF: Aktien = Weltindex (alle Regionen, nach BIP gewichtet), Obligationen =
/// 10-jährige Obligationen der Eidgenossenschaft ([`ch_bonds_yearly`]), Teuerung = Schweizer Konsumentenpreise (JST).
pub fn world_dataset(jst: &JstData, snb: &SnbData) -> Result<DataSet> {
    let countries: Vec<&str> = all_countries().collect();
    let stocks = jst.region_stocks(&countries)?;
    let (_, bonds) = ch_bonds_yearly(jst, snb)?;
    let n = stocks.len().min(bonds.len());
    let inflation: Vec<f64> = (0..n)
        .map(|i| {
            let y = FIRST_YEAR + i as i32;
            jst.ch_inflation(y)
                .ok_or_else(|| invalid(format!("JST-Daten: Teuerung Schweiz {y} fehlt")))
        })
        .collect::<Result<_>>()?;
    let mut source = format!("{SOURCE}; SNB (data.snb.ch): rendoblim");
    if let Some(p) = snb.published.get("rendoblim") {
        source.push_str(&format!(" (Stand {p})"));
    }
    Ok(DataSet {
        name: "Welt (JST)".into(),
        meta: Meta {
            title: Some("Welt (JST): Aktien nach BIP gewichtet, Obligationen der Eidgenossenschaft, in CHF".into()),
            currency: Some("CHF".into()),
            source: Some(source),
            info: Some(INFO_WORLD.into()),
            notes: vec![
                "Aktien: Gesamtrendite der Länder in CHF, gewichtet mit dem realen BIP des Vorjahres (Maddison); Deutschland 1923 ausgeschlossen (Hyperinflation), Deutschland 1945–1946 und Japan 1946–1947 ohne Werte.".into(),
                format!(
                    "Obligationen: JST bis {}, danach aus den monatlichen SNB-Renditen (10 Jahre); 1915 Coupon statt Rendite (Börse geschlossen).",
                    snb_first_full_year(snb).unwrap_or(FIRST_YEAR) - 1
                ),
                "Lizenz: CC BY-NC-SA 4.0 (nicht kommerziell, Weitergabe unter gleicher Lizenz).".into(),
            ],
        },
        freq: Freq::Yearly,
        first: YearMonth::new(FIRST_YEAR, 1),
        stocks: stocks[..n].to_vec(),
        bonds: bonds[..n].to_vec(),
        inflation,
        saron: None,
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
        // CHF 2 je USD → 2.2; Land: 4 je USD → 4 (CHF je Einheit 0.5 → 0.55, +10 %).
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
        // Vorjahr: CHE 1 × 10 = 10, USA 3 × 10 = 30 → 25 % / 75 %
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

    /// Zeile der Rohtabelle: Jahr, Land und die acht Werte.
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
        vals[2] = s(" 2.5 "); // xrusd als Text
        vals[3] = s("keine Zahl"); // cpi: unlesbarer Text → fehlt
        vals[4] = Data::Float(f64::NAN); // eq_tr: nicht endlich → fehlt
        vals[5] = Data::Bool(true); // bond_tr: anderer Zellentyp → fehlt

        // Reihenfolge: Hauptzeile, Jahr als Ganzzahl und Land mit Leerzeichen, Land ausserhalb der Regionen,
        // Land kein Text, ohne Jahr, Jahr unlesbar, kurze Zeile ohne Land
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
        // Text im Jahr wird als Zahl gelesen
        let rows = vec![header(), raw_row(s("1999"), s("CHE"), empty8())];
        assert!(JstData::from_rows(&rows).unwrap().row("CHE", 1999).is_some());
    }

    #[test]
    fn from_rows_reports_empty_tables_missing_columns_and_missing_countries() {
        let msg = |r: Result<JstData>| match r {
            Err(Error::Invalid(m)) => m,
            other => panic!("Fehler erwartet: {other:?}"),
        };
        assert!(msg(JstData::from_rows(&[])).contains("leer"));
        let mut no_iso = header();
        no_iso.remove(1);
        assert!(msg(JstData::from_rows(&[no_iso])).contains("«iso»"));
        let mut no_bill = header();
        no_bill.pop();
        assert!(msg(JstData::from_rows(&[no_bill])).contains("«bill_rate»"));
        // Nur Kopfzeile oder nur fremde Länder
        assert!(msg(JstData::from_rows(&[header()])).contains("keine Werte"));
        let other = vec![header(), raw_row(Data::Int(2000), s("XXX"), empty8())];
        assert!(msg(JstData::from_rows(&other)).contains("keine Werte"));
        // Kopfzellen, die kein Text sind, zählen als unbenannt
        let mut odd = header();
        odd[0] = Data::Int(5);
        assert!(msg(JstData::from_rows(&[odd])).contains("«year»"));
    }

    #[test]
    fn last_year_of_empty_data_is_the_first_year() {
        assert_eq!(JstData::default().last_year(), FIRST_YEAR);
    }

    #[test]
    fn region_stocks_name_the_missing_year() {
        // Nur 1900 hat eine Rendite; 1901 fehlt → Fehler mit Land und Jahr
        let d = data("1899,CHE,1,1,1,,,,,\n1900,CHE,1,1,1,,0.1,,,\n1901,CHE,1,1,1,,,,,\n");
        match d.region_stocks(&["CHE"]) {
            Err(Error::Invalid(m)) => assert!(m.contains("CHE") && m.contains("1901"), "{m}"),
            other => panic!("Fehler erwartet: {other:?}"),
        }
        // Ohne Lücke: ein Wert je Jahr ab FIRST_YEAR bis zum letzten Jahr
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
        // SNB ab Januar 1901 → erstes ganzes Jahr 1902; 1900 und 1901 aus JST (1901 ohne bond_tr → Coupon)
        let d = data("1900,CHE,1,1,1,,,0.07,0.03,\n1901,CHE,1,1,1,,,,0.04,\n");
        let mut snb = SnbData::default();
        months(&mut snb, (1901, 1), (1902, 12), 2.0);
        let (first, v) = ch_bonds_yearly(&d, &snb).unwrap();
        assert_eq!((first, v.len()), (1900, 3));
        assert_eq!((v[0], v[1]), (0.07, 0.04));
        let monthly = 1.0 + bond_return(0.02, 0.02);
        assert!((v[2] - (monthly.powi(12) - 1.0)).abs() < 1e-12, "{}", v[2]);
        // Die Zinsen sind in Prozent angegeben: 2 % → 0.02, nicht 2
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
        months(&mut snb, (1902, 1), (1903, 12), 2.0); // Wechsel 1903; JST-Jahr 1901 fehlt
        assert!(msg(ch_bonds_yearly(&d, &snb)).contains("1901"));
        let d = data("1900,CHE,1,1,1,,,,,\n"); // weder bond_tr noch bond_rate
        let mut snb = SnbData::default();
        months(&mut snb, (1901, 1), (1901, 12), 2.0); // Wechsel 1902
        assert!(msg(ch_bonds_yearly(&d, &snb)).contains("Obligationen Schweiz 1900"));
    }

    #[test]
    fn ch_short_uses_bill_rate_then_the_yearly_mean_of_saron_in_percent() {
        let d = data("1900,CHE,1,1,1,,,,,0.03\n1901,CHE,1,1,1,,,,,0.04\n");
        let mut snb = SnbData::default();
        // 1902: Jahresmittel (6 × 1 + 6 × 3) / 12 = 2 % → 0.02; 1903 unvollständig → Ende
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
        // Fehlt bill_rate in einem JST-Jahr, nennt der Fehler das Jahr
        let d = data("1900,CHE,1,1,1,,,,,0.03\n1901,CHE,1,1,1,,,,,\n");
        match ch_short_yearly(&d, &snb) {
            Err(Error::Invalid(m)) => assert!(m.contains("1901"), "{m}"),
            other => panic!("Fehler erwartet: {other:?}"),
        }
    }
}
