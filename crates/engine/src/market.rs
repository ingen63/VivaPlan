//! Renditepfade: fixe Werte, historische Zeitreihen (jährlich oder monatlich) und deren Fortschreibung.

use std::collections::VecDeque;
use std::path::Path;

use crate::model::{Extrapolation, ExtrapolationMethod};
use crate::resolve::Curve;
use crate::time::YearMonth;
use crate::{Error, Result};

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Freq {
    #[default]
    Yearly,
    Monthly,
}

/// Eine historische Zeitreihe. Jahreswerte sind Jahresrenditen; Monatswerte sind Monatsrenditen,
/// ausser Inflation und SARON: diese sind immer Jahressätze. Fehlende SARON-Werte sind `NaN`
/// (dort gilt der fixe Wert).
#[derive(Clone, Debug, Default)]
pub struct DataSet {
    pub name: String,
    pub meta: Meta,
    pub freq: Freq,
    /// Erster Monat (bei Jahresdaten: Januar des ersten Jahres).
    pub first: YearMonth,
    pub stocks: Vec<f64>,
    pub bonds: Vec<f64>,
    pub inflation: Vec<f64>,
    pub saron: Option<Vec<f64>>,
}

/// Angaben aus den Kommentarzeilen am Anfang der CSV-Datei (`# Titel: …`, `# Währung: CHF`, …).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Meta {
    pub title: Option<String>,
    pub currency: Option<String>,
    pub source: Option<String>,
    /// Kurze Erklärung der Reihe (höchstens 2 Zeilen) für das Info-Popup (`# Info: …`).
    pub info: Option<String>,
    /// Übrige Kommentarzeilen (`# Hinweis: …` oder ohne Schlüssel).
    pub notes: Vec<String>,
}

impl Meta {
    fn add_line(&mut self, line: &str) {
        let line = line.trim();
        if line.is_empty() {
            return;
        }
        if let Some((k, v)) = line.split_once(':') {
            let v = v.trim().to_string();
            match k.trim().to_lowercase().as_str() {
                "titel" | "title" => return self.title = Some(v),
                "währung" | "waehrung" | "currency" => return self.currency = Some(v.to_uppercase()),
                "quelle" | "source" => return self.source = Some(v),
                "info" | "erklärung" | "erklaerung" => return self.info = Some(v),
                "hinweis" | "note" => return self.notes.push(v),
                _ => {}
            }
        }
        self.notes.push(line.to_string());
    }

    fn write(&self, out: &mut String) {
        let mut line = |k: &str, v: &str| out.push_str(&format!("# {k}: {v}\n"));
        if let Some(v) = &self.title {
            line("Titel", v);
        }
        if let Some(v) = &self.currency {
            line("Währung", v);
        }
        if let Some(v) = &self.info {
            line("Info", v);
        }
        if let Some(v) = &self.source {
            line("Quelle", v);
        }
        for v in &self.notes {
            line("Hinweis", v);
        }
    }
}

impl DataSet {
    pub fn load_csv(path: &Path) -> Result<DataSet> {
        let text = std::fs::read_to_string(path)
            .map_err(|e| Error::Io(format!("Zeitreihe {} nicht lesbar: {e}", path.display())))?;
        let name = path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        DataSet::parse_csv(&name, &text)
    }

    /// CSV mit Kopfzeile. Erste Spalte: Jahr (`1990`) oder Monat (`1990-01`).
    /// Weitere Spalten nach Namen: Stocks/Aktien, Bonds/Obligationen, Inflation, Saron (darf leer sein).
    /// Zeilen mit `#` am Anfang sind Kommentare; vor der Kopfzeile enthalten sie die [`Meta`]-Angaben.
    pub fn parse_csv(name: &str, text: &str) -> Result<DataSet> {
        let mut meta = Meta::default();
        let mut lines = text
            .lines()
            .enumerate()
            .filter(|(_, l)| !l.trim().is_empty())
            .filter(
                |(_, l)| match l.trim_start().trim_start_matches('\u{feff}').strip_prefix('#') {
                    Some(c) => {
                        meta.add_line(c);
                        false
                    }
                    None => true,
                },
            );
        let (_, header) = lines
            .next()
            .ok_or_else(|| Error::Invalid(format!("Zeitreihe {name}: Datei ist leer")))?;
        let cols: Vec<String> = header
            .split(',')
            .map(|c| c.trim().trim_start_matches('\u{feff}').to_lowercase())
            .collect();
        let find = |names: &[&str]| cols.iter().position(|c| names.contains(&c.as_str()));
        let ci = |names: &[&str], label: &str| {
            find(names).ok_or_else(|| Error::Invalid(format!("Zeitreihe {name}: Spalte «{label}» fehlt")))
        };
        let c_stocks = ci(&["stocks", "aktien"], "Stocks")?;
        let c_bonds = ci(&["bonds", "obligationen"], "Bonds")?;
        let c_infl = ci(&["inflation"], "Inflation")?;
        let c_saron = find(&["saron"]);

        let mut ds = DataSet {
            name: name.to_string(),
            saron: c_saron.map(|_| vec![]),
            ..Default::default()
        };
        let mut prev: Option<YearMonth> = None;
        for (lineno, line) in lines {
            let cells: Vec<&str> = line.split(',').map(str::trim).collect();
            let bad = |what: &str| Error::Invalid(format!("Zeitreihe {name}, Zeile {}: {what}", lineno + 1));
            let date = cells.first().copied().unwrap_or("");
            let (m, freq) = if date.contains('-') {
                (date.parse::<YearMonth>().map_err(|e| bad(&e))?, Freq::Monthly)
            } else {
                // ISS-14: Jahre wie bei Monatsdaten vierstellig; sonst überläuft der Monatsindex.
                let y: i32 = date
                    .parse()
                    .ok()
                    .filter(|y| (1000..=9999).contains(y))
                    .ok_or_else(|| bad(&format!("«{date}» ist kein Jahr (1000 bis 9999)")))?;
                (YearMonth::new(y, 1), Freq::Yearly)
            };
            match prev {
                None => {
                    ds.first = m;
                    ds.freq = freq;
                }
                Some(p) => {
                    let step = if ds.freq == Freq::Yearly { 12 } else { 1 };
                    if freq != ds.freq || p.0.checked_add(step) != Some(m.0) {
                        return Err(bad("Datum fehlt oder ist nicht fortlaufend"));
                    }
                }
            }
            prev = Some(m);
            let num = |c: usize| -> Result<f64> {
                let s = cells.get(c).copied().unwrap_or("");
                s.parse::<f64>().map_err(|_| bad(&format!("«{s}» ist keine Zahl")))
            };
            ds.stocks.push(num(c_stocks)?);
            ds.bonds.push(num(c_bonds)?);
            ds.inflation.push(num(c_infl)?);
            if let (Some(c), Some(v)) = (c_saron, ds.saron.as_mut()) {
                let empty = cells.get(c).is_none_or(|s| s.is_empty());
                v.push(if empty { f64::NAN } else { num(c)? });
            }
        }
        if ds.stocks.is_empty() {
            return Err(Error::Invalid(format!("Zeitreihe {name}: keine Daten")));
        }
        ds.meta = meta;
        Ok(ds)
    }

    /// CSV im Format von [`DataSet::parse_csv`], mit den Meta-Angaben als Kommentarzeilen.
    pub fn to_csv(&self) -> String {
        let mut out = String::new();
        self.meta.write(&mut out);
        let date_col = if self.freq == Freq::Monthly { "Date" } else { "Years" };
        out.push_str(&format!("{date_col},Stocks,Bonds,Inflation"));
        out.push_str(if self.saron.is_some() { ",Saron\n" } else { "\n" });
        let num = |v: f64| {
            if v.is_nan() {
                return String::new();
            }
            let s = format!("{v:.10}");
            let s = s.trim_end_matches('0').trim_end_matches('.');
            if s == "-0" {
                "0".into()
            } else {
                s.to_string()
            }
        };
        for i in 0..self.len() {
            let date = match self.freq {
                Freq::Monthly => self.first.plus(i as i32).iso(),
                Freq::Yearly => (self.first.year() + i as i32).to_string(),
            };
            out.push_str(&format!(
                "{date},{},{},{}",
                num(self.stocks[i]),
                num(self.bonds[i]),
                num(self.inflation[i])
            ));
            if let Some(s) = &self.saron {
                out.push_str(&format!(",{}", num(s[i])));
            }
            out.push('\n');
        }
        out
    }

    /// Währung gemäss Meta-Angabe.
    pub fn currency(&self) -> Option<&str> {
        self.meta.currency.as_deref()
    }

    /// Mittlere Jahreswerte über die ganze Reihe: geometrische Rendite Aktien und Obligationen,
    /// geometrische Inflation (aus den Jahressätzen bzw. bei Monatsdaten aus den Dezemberwerten).
    pub fn averages(&self) -> (f64, f64, f64) {
        let per_year = if self.freq == Freq::Monthly { 12.0 } else { 1.0 };
        let years = self.len() as f64 / per_year;
        let cagr = |v: &[f64]| (v.iter().map(|r| (1.0 + r).ln()).sum::<f64>() / years).exp() - 1.0;
        let infl: Vec<f64> = match self.freq {
            Freq::Yearly => self.inflation.clone(),
            Freq::Monthly => (0..self.len())
                .filter(|&i| self.first.plus(i as i32).month() == 12)
                .map(|i| self.inflation[i])
                .collect(),
        };
        let infl_avg = if infl.is_empty() {
            f64::NAN
        } else {
            (infl.iter().map(|r| (1.0 + r).ln()).sum::<f64>() / infl.len() as f64).exp() - 1.0
        };
        (cagr(&self.stocks), cagr(&self.bonds), infl_avg)
    }

    pub fn len(&self) -> usize {
        self.stocks.len()
    }
    pub fn is_empty(&self) -> bool {
        self.stocks.is_empty()
    }

    /// Letzter Monat mit Daten (bei Jahresdaten: Dezember des letzten Jahres).
    pub fn last(&self) -> YearMonth {
        match self.freq {
            Freq::Yearly => YearMonth::new(self.first.year() + self.len() as i32 - 1, 12),
            Freq::Monthly => self.first.plus(self.len() as i32 - 1),
        }
    }
}

/// Werte pro Simulationsmonat `k` (0 = Start).
#[derive(Clone, Debug)]
pub struct MarketPath {
    pub label: String,
    /// Monatsrenditen.
    pub stocks: Vec<f64>,
    pub bonds: Vec<f64>,
    /// Jahressätze.
    pub inflation: Vec<f64>,
    pub saron: Vec<f64>,
}

/// Jahressatz → Monatssatz; Sätze unter −100 % gelten als −100 % (statt einer ungültigen Zahl).
pub fn monthly(yearly: f64) -> f64 {
    (1.0 + yearly).max(0.0).powf(1.0 / 12.0) - 1.0
}

/// Kurven für fixe Werte (Jahressätze).
pub struct FixedCurves {
    pub stocks: Curve,
    pub bonds: Curve,
    pub inflation: Curve,
    pub saron: Curve,
}

pub fn fixed_path(c: &FixedCurves, start: YearMonth, months: usize) -> MarketPath {
    let mut p = MarketPath::with_capacity("Fixe Renditen".into(), months);
    for k in 0..months {
        let m = start.plus(k as i32);
        p.stocks.push(monthly(c.stocks.at(m)));
        p.bonds.push(monthly(c.bonds.at(m)));
        p.inflation.push(c.inflation.at(m));
        p.saron.push(c.saron.at(m));
    }
    p
}

impl MarketPath {
    fn push_fixed(&mut self, c: &FixedCurves, m: YearMonth) {
        self.stocks.push(monthly(c.stocks.at(m)));
        self.bonds.push(monthly(c.bonds.at(m)));
        self.inflation.push(c.inflation.at(m));
        self.saron.push(c.saron.at(m));
    }

    pub(crate) fn with_capacity(label: String, n: usize) -> Self {
        MarketPath {
            label,
            stocks: Vec::with_capacity(n),
            bonds: Vec::with_capacity(n),
            inflation: Vec::with_capacity(n),
            saron: Vec::with_capacity(n),
        }
    }
}

/// Historischer Pfad. Bei Jahresdaten gilt pro Kalenderjahr der Simulation ein Jahreswert, beginnend
/// mit dem Jahr von `hist_start`; bei Monatsdaten läuft die Reihe ab `hist_start` Monat für Monat.
/// Reicht die Reihe nicht bis zum Simulationsende, wird sie gemäss `ex` fortgeschrieben.
pub fn historical_path(
    ds: &DataSet,
    hist_start: YearMonth,
    sim_start: YearMonth,
    months: usize,
    ex: &Extrapolation,
    fixed: &FixedCurves,
) -> Result<MarketPath> {
    if hist_start < ds.first || hist_start > ds.last() {
        return Err(Error::Invalid(format!(
            "Historischer Start {hist_start} liegt ausserhalb der Zeitreihe {} ({} – {})",
            ds.name,
            ds.first,
            ds.last()
        )));
    }
    let sim_end = sim_start.plus(months as i32 - 1);
    let label = match ds.freq {
        Freq::Yearly => format!("Historisch ab {}", hist_start.year()),
        Freq::Monthly => format!("Historisch ab {hist_start}"),
    };
    let mut p = MarketPath::with_capacity(label, months);
    // Fortschreibung mit fixen Werten: die Werte des jeweiligen Monats (zeitabhängig möglich).
    let fixed_ex = ex.method == ExtrapolationMethod::Fixed;
    match ds.freq {
        Freq::Yearly => {
            let offset = (hist_start.year() - ds.first.year()) as usize;
            let needed = offset + (sim_end.year() - sim_start.year()) as usize + 1;
            let fy = |c: &Curve| c.at(sim_start);
            let stocks = extend(&ds.stocks, needed, ex, fy(&fixed.stocks), 1);
            let bonds = extend(&ds.bonds, needed, ex, fy(&fixed.bonds), 1);
            let infl = extend(&ds.inflation, needed, ex, fy(&fixed.inflation), 1);
            let saron = ds.saron.as_ref().map(|s| extend(s, needed, ex, fy(&fixed.saron), 1));
            for k in 0..months {
                let m = sim_start.plus(k as i32);
                let i = offset + (m.year() - sim_start.year()) as usize;
                if i >= ds.len() && fixed_ex {
                    p.push_fixed(fixed, m);
                    continue;
                }
                p.stocks.push(monthly(stocks[i]));
                p.bonds.push(monthly(bonds[i]));
                p.inflation.push(infl[i]);
                p.saron.push(or_fixed(saron.as_deref(), i, || fixed.saron.at(m)));
            }
        }
        Freq::Monthly => {
            let offset = hist_start.0 - ds.first.0;
            let needed = offset as usize + months;
            let fm = |c: &Curve| monthly(c.at(sim_start));
            let stocks = extend(&ds.stocks, needed, ex, fm(&fixed.stocks), 12);
            let bonds = extend(&ds.bonds, needed, ex, fm(&fixed.bonds), 12);
            // Inflation monatlich als Jahressatz geliefert; für die Fortschreibung genügt der Jahressatz.
            let infl = extend(&ds.inflation, needed, ex, fixed.inflation.at(sim_start), 12);
            let saron = ds
                .saron
                .as_ref()
                .map(|s| extend(s, needed, ex, fixed.saron.at(sim_start), 12));
            for k in 0..months {
                let i = offset as usize + k;
                if i >= ds.len() && fixed_ex {
                    p.push_fixed(fixed, sim_start.plus(k as i32));
                    continue;
                }
                p.stocks.push(stocks[i]);
                p.bonds.push(bonds[i]);
                p.inflation.push(infl[i]);
                p.saron.push(or_fixed(saron.as_deref(), i, || {
                    fixed.saron.at(sim_start.plus(k as i32))
                }));
            }
        }
    }
    Ok(p)
}

/// Alle historischen Startzeitpunkte, für die die Reihe den ganzen Simulationszeitraum abdeckt.
pub fn rolling_starts(ds: &DataSet, sim_start: YearMonth, months: usize) -> Vec<YearMonth> {
    let sim_end = sim_start.plus(months as i32 - 1);
    match ds.freq {
        Freq::Yearly => {
            let years = (sim_end.year() - sim_start.year() + 1) as usize;
            if ds.len() < years {
                return vec![];
            }
            (0..=ds.len() - years)
                .map(|i| YearMonth::new(ds.first.year() + i as i32, sim_start.month()))
                .collect()
        }
        Freq::Monthly => {
            if ds.len() < months {
                return vec![];
            }
            (0..=ds.len() - months).map(|i| ds.first.plus(i as i32)).collect()
        }
    }
}

/// Wert `i` der Reihe; fehlt die Reihe oder der Wert (`NaN`), gilt der fixe Wert.
fn or_fixed(values: Option<&[f64]>, i: usize, fixed: impl FnOnce() -> f64) -> f64 {
    match values.map(|v| v[i]) {
        Some(v) if !v.is_nan() => v,
        _ => fixed(),
    }
}

/// Verlängert `values` auf `needed` Einträge (längere Reihen bleiben unverändert). `per_year` = Werte pro Jahr (für das
/// Durchschnittsfenster `ex.range` Jahre).
fn extend(values: &[f64], needed: usize, ex: &Extrapolation, fixed: f64, per_year: usize) -> Vec<f64> {
    let mut out = values.to_vec();
    if out.len() >= needed {
        return out;
    }
    let range = (ex.range.max(1) * per_year).min(values.len().max(1));
    // Fenster der letzten `range` Werte (bei `Fixed` unbenutzt).
    let mut window: VecDeque<f64> = values[values.len().saturating_sub(range)..].iter().copied().collect();
    while out.len() < needed {
        let v = match ex.method {
            ExtrapolationMethod::Fixed => fixed,
            ExtrapolationMethod::Average => window.iter().sum::<f64>() / window.len() as f64,
            ExtrapolationMethod::Geometric => {
                window
                    .iter()
                    .map(|v| 1.0 + v)
                    .product::<f64>()
                    .powf(1.0 / window.len() as f64)
                    - 1.0
            }
        };
        window.pop_front();
        window.push_back(v);
        out.push(v);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    const CSV: &str =
        "Years,Stocks,Bonds,Inflation,,\n2000,0.1,0.02,0.01,,\n2001,-0.1,0.03,0.02,,\n2002,0.2,0.01,0.0,,\n";

    #[test]
    fn parse_yearly_csv() {
        let ds = DataSet::parse_csv("t", CSV).unwrap();
        assert_eq!(ds.freq, Freq::Yearly);
        assert_eq!(ds.first, YearMonth::new(2000, 1));
        assert_eq!(ds.last(), YearMonth::new(2002, 12));
        assert_eq!(ds.stocks, vec![0.1, -0.1, 0.2]);
    }

    #[test]
    fn rejects_garbage() {
        let err = DataSet::parse_csv("t", "Years,Stocks,Bonds,Inflation\n2000,+AC0-0.1,0,0\n").unwrap_err();
        assert!(err.to_string().contains("Zeile 2"));
    }

    #[test]
    fn geometric_extension() {
        let ex = Extrapolation {
            method: ExtrapolationMethod::Geometric,
            range: 2,
        };
        let v = extend(&[0.1, 0.21], 3, &ex, 0.0, 1);
        let expected = (1.1f64 * 1.21).sqrt() - 1.0;
        assert!((v[2] - expected).abs() < 1e-12);
    }

    #[test]
    fn yearly_path_by_calendar_year() {
        let ds = DataSet::parse_csv("t", CSV).unwrap();
        let fixed = FixedCurves {
            stocks: Curve::constant(0.0),
            bonds: Curve::constant(0.0),
            inflation: Curve::constant(0.0),
            saron: Curve::constant(0.0),
        };
        // Start im November 2030: Nov/Dez nutzen 2000, ab Januar 2031 das Jahr 2001.
        let p = historical_path(
            &ds,
            YearMonth::new(2000, 11),
            YearMonth::new(2030, 11),
            3,
            &Default::default(),
            &fixed,
        )
        .unwrap();
        assert!((p.stocks[0] - monthly(0.1)).abs() < 1e-15);
        assert!((p.stocks[2] - monthly(-0.1)).abs() < 1e-15);
        assert_eq!(rolling_starts(&ds, YearMonth::new(2030, 11), 3).len(), 2);
    }

    fn flat() -> FixedCurves {
        FixedCurves {
            stocks: Curve::constant(0.0),
            bonds: Curve::constant(0.0),
            inflation: Curve::constant(0.0),
            saron: Curve::constant(0.0),
        }
    }

    fn close(a: f64, b: f64) {
        assert!((a - b).abs() < 1e-12, "{a} != {b}");
    }

    // FEA-29: Name, leere Reihe, Mittelwerte für Jahres- und Monatsreihen.
    #[test]
    fn name_empty_and_averages() {
        let y = DataSet::parse_csv("Jahre", CSV).unwrap();
        assert_eq!(y.name, "Jahre");
        assert!(DataSet::default().is_empty() && !y.is_empty());
        let (s, b, i) = y.averages();
        close(s, (1.1f64 * 0.9 * 1.2).powf(1.0 / 3.0) - 1.0);
        close(b, (1.02f64 * 1.03 * 1.01).powf(1.0 / 3.0) - 1.0);
        close(i, (1.01f64 * 1.02 * 1.0).powf(1.0 / 3.0) - 1.0);

        // 24 Monate ab Januar 2000: Aktien 1 % pro Monat, Inflation (Jahressatz) im Dezember 2 % bzw. 5 %.
        let mut csv = String::from("Date,Stocks,Bonds,Inflation\n");
        for k in 0..24 {
            let m = YearMonth::new(2000, 1).plus(k);
            let infl = match k {
                11 => 0.02,
                23 => 0.05,
                _ => 0.5,
            };
            csv.push_str(&format!("{},0.01,0.002,{infl}\n", m.iso()));
        }
        let m = DataSet::parse_csv("m", &csv).unwrap();
        let (s, b, i) = m.averages();
        close(s, 1.01f64.powi(12) - 1.0);
        close(b, 1.002f64.powi(12) - 1.0);
        close(i, (1.02f64 * 1.05).sqrt() - 1.0);
        // Ohne Dezember keine Inflation
        let short = DataSet::parse_csv("k", "Date,Stocks,Bonds,Inflation\n2000-01,0.01,0,0.02\n").unwrap();
        assert!(short.averages().2.is_nan());
    }

    #[test]
    fn historical_start_must_lie_in_the_series() {
        let ds = DataSet::parse_csv("t", CSV).unwrap();
        let path =
            |start: YearMonth| historical_path(&ds, start, YearMonth::new(2030, 1), 12, &Default::default(), &flat());
        assert!(path(YearMonth::new(1999, 12)).is_err());
        assert!(path(YearMonth::new(2002, 12)).is_ok());
        assert!(path(YearMonth::new(2003, 1)).is_err());
    }

    #[test]
    fn short_series_are_extended_from_the_first_entry() {
        let avg = Extrapolation {
            method: ExtrapolationMethod::Average,
            range: 30,
        };
        // Jahresreihe ab dem ersten Jahr: 3 Jahre Daten, 5 Jahre Simulation.
        let ds = DataSet::parse_csv("t", CSV).unwrap();
        let p = historical_path(&ds, YearMonth::new(2000, 1), YearMonth::new(2030, 1), 60, &avg, &flat()).unwrap();
        assert_eq!(p.stocks.len(), 60);
        close(p.stocks[36], monthly((0.1 - 0.1 + 0.2) / 3.0));
        // Monatsreihe ab dem ersten Monat: 3 Monate Daten, 5 Monate Simulation.
        let m = DataSet::parse_csv(
            "m",
            "Date,Stocks,Bonds,Inflation\n2000-01,0.01,0,0\n2000-02,0.02,0,0\n2000-03,0.03,0,0\n",
        )
        .unwrap();
        let p = historical_path(&m, YearMonth::new(2000, 1), YearMonth::new(2030, 1), 5, &avg, &flat()).unwrap();
        close(p.stocks[3], 0.02);
    }

    #[test]
    fn rolling_starts_need_the_whole_period() {
        let ds = DataSet::parse_csv("t", CSV).unwrap();
        // 2 Kalenderjahre: Starts 2000 und 2001
        assert_eq!(rolling_starts(&ds, YearMonth::new(2030, 1), 24).len(), 2);
        // 3 Kalenderjahre (genau die Länge der Reihe): ein Start
        assert_eq!(
            rolling_starts(&ds, YearMonth::new(2030, 1), 36),
            vec![YearMonth::new(2000, 1)]
        );
        assert!(rolling_starts(&ds, YearMonth::new(2030, 1), 48).is_empty());
        let m = DataSet::parse_csv("m", "Date,Stocks,Bonds,Inflation\n2000-01,0.01,0,0\n2000-02,0.02,0,0\n").unwrap();
        assert_eq!(
            rolling_starts(&m, YearMonth::new(2030, 1), 2),
            vec![YearMonth::new(2000, 1)]
        );
        assert_eq!(rolling_starts(&m, YearMonth::new(2030, 1), 1).len(), 2);
        assert!(rolling_starts(&m, YearMonth::new(2030, 1), 3).is_empty());
    }

    #[test]
    fn extension_window_length_and_methods() {
        // Monatswerte: Fenster = range Jahre × 12 Werte.
        let mut v = vec![0.0; 12];
        v.extend([0.12; 12]);
        let avg = Extrapolation {
            method: ExtrapolationMethod::Average,
            range: 1,
        };
        let out = extend(&v, 25, &avg, 0.0, 12);
        assert_eq!(out.len(), 25);
        close(out[24], 0.12);
        let out = extend(
            &[0.0, 0.3],
            3,
            &Extrapolation {
                method: ExtrapolationMethod::Average,
                range: 2,
            },
            0.0,
            1,
        );
        close(out[2], 0.15);
        let fixed = Extrapolation {
            method: ExtrapolationMethod::Fixed,
            range: 1,
        };
        assert_eq!(extend(&[0.5, 0.6], 4, &fixed, 0.07, 1), vec![0.5, 0.6, 0.07, 0.07]);
        assert_eq!(extend(&[0.5, 0.6], 1, &fixed, 0.07, 1), vec![0.5, 0.6]);
    }
}
