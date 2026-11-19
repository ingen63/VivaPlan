//! Return paths: fixed values, historical series (annual or monthly) and their extrapolation.

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

/// A historical series. Annual values are annual returns; monthly values are monthly returns,
/// except inflation and SARON: these are always annual rates. Missing SARON values are `NaN`
/// (the fixed value applies there).
#[derive(Clone, Debug, Default)]
pub struct DataSet {
    pub name: String,
    pub meta: Meta,
    pub freq: Freq,
    /// First month (for annual data: January of the first year).
    pub first: YearMonth,
    pub stocks: Vec<f64>,
    pub bonds: Vec<f64>,
    pub inflation: Vec<f64>,
    pub saron: Option<Vec<f64>>,
    /// #100: whole stock series of the index (also beyond the years in common with the bonds), in the
    /// frequency the bootstrap uses; for the suggested values of the parametric model.
    pub stock_series: Option<StockSeries>,
}

/// #100: Stock returns of an index (monthly or annual returns from `first`).
#[derive(Clone, Debug, PartialEq)]
pub struct StockSeries {
    pub freq: Freq,
    pub first: YearMonth,
    pub values: Vec<f64>,
}

/// Data from the comment lines at the start of the CSV file (`# Titel: …`, `# Währung: CHF`, …).
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Meta {
    pub title: Option<String>,
    pub currency: Option<String>,
    pub source: Option<String>,
    /// Short explanation of the series (at most 2 lines) for the info popup (`# Info: …`).
    pub info: Option<String>,
    /// Other comment lines (`# Hinweis: …` or without a key).
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
        let text = std::fs::read_to_string(path).map_err(|e| {
            Error::Io(msg!(
                series_unreadable,
                path = path.display().to_string(),
                detail = e.to_string()
            ))
        })?;
        let name = path
            .file_stem()
            .map(|s| s.to_string_lossy().into_owned())
            .unwrap_or_default();
        DataSet::parse_csv(&name, &text)
    }

    /// CSV with a header row. First column: year (`1990`) or month (`1990-01`).
    /// Further columns by name: `Stocks`/`Aktien`, `Bonds`/`Obligationen`, `Inflation`, `Saron` (may be empty).
    /// Lines starting with `#` are comments; before the header row they contain the [`Meta`] data.
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
            .ok_or_else(|| Error::Invalid(msg!(series_file_empty, name = name)))?;
        let cols: Vec<String> = header
            .split(',')
            .map(|c| c.trim().trim_start_matches('\u{feff}').to_lowercase())
            .collect();
        let find = |names: &[&str]| cols.iter().position(|c| names.contains(&c.as_str()));
        let ci = |names: &[&str], label: &str| {
            find(names).ok_or_else(|| Error::Invalid(msg!(series_column_missing, name = name, column = label)))
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
            let bad = |what: crate::message::Message| {
                Error::Invalid(msg!(series_line_problem, name = name, row = lineno + 1, problem = what))
            };
            let date = cells.first().copied().unwrap_or("");
            let (m, freq) = if date.contains('-') {
                (
                    date.parse::<YearMonth>()
                        .map_err(|_| bad(msg!(month_invalid, value = date)))?,
                    Freq::Monthly,
                )
            } else {
                // #60: Years four digits as with monthly data; otherwise the month index overflows.
                let y: i32 = date
                    .parse()
                    .ok()
                    .filter(|y| (1000..=9999).contains(y))
                    .ok_or_else(|| bad(msg!(year_invalid, value = date)))?;
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
                        return Err(bad(msg!(series_dates_not_continuous)));
                    }
                }
            }
            prev = Some(m);
            let num = |c: usize| -> Result<f64> {
                let s = cells.get(c).copied().unwrap_or("");
                s.parse::<f64>().map_err(|_| bad(msg!(not_a_number, value = s)))
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
            return Err(Error::Invalid(msg!(series_no_data, name = name)));
        }
        ds.meta = meta;
        Ok(ds)
    }

    /// CSV in the format of [`DataSet::parse_csv`], with the meta data as comment lines.
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

    /// Currency according to the meta data.
    pub fn currency(&self) -> Option<&str> {
        self.meta.currency.as_deref()
    }

    /// Mean annual values over the whole series: geometric return of stocks and bonds,
    /// geometric inflation (from the annual rates, or for monthly data from the December values).
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

    /// Last month with data (for annual data: December of the last year).
    pub fn last(&self) -> YearMonth {
        match self.freq {
            Freq::Yearly => YearMonth::new(self.first.year() + self.len() as i32 - 1, 12),
            Freq::Monthly => self.first.plus(self.len() as i32 - 1),
        }
    }
}

/// Values per simulation month `k` (0 = start).
#[derive(Clone, Debug)]
pub struct MarketPath {
    pub label: crate::message::Message,
    /// Monthly returns.
    pub stocks: Vec<f64>,
    pub bonds: Vec<f64>,
    /// Annual rates.
    pub inflation: Vec<f64>,
    pub saron: Vec<f64>,
    /// Lead-in (#103): monthly stock returns of up to 12 months before the start, the oldest first. It only serves the
    /// return criterion of the 3-bucket strategy (last 12 months) and changes no wealth.
    pub pre_stocks: Vec<f64>,
}

/// Number of lead-in months before the simulation start (#103).
pub const PRE_MONTHS: usize = 12;

/// Annual rate → monthly rate; rates below −100 % count as −100 % (instead of an invalid number).
pub fn monthly(yearly: f64) -> f64 {
    (1.0 + yearly).max(0.0).powf(1.0 / 12.0) - 1.0
}

/// Curves for fixed values (annual rates).
pub struct FixedCurves {
    pub stocks: Curve,
    pub bonds: Curve,
    pub inflation: Curve,
    pub saron: Curve,
}

pub fn fixed_path(c: &FixedCurves, start: YearMonth, months: usize) -> MarketPath {
    let mut p = MarketPath::with_capacity(msg!(path_fixed), months);
    for k in 0..months {
        let m = start.plus(k as i32);
        p.stocks.push(monthly(c.stocks.at(m)));
        p.bonds.push(monthly(c.bonds.at(m)));
        p.inflation.push(c.inflation.at(m));
        p.saron.push(c.saron.at(m));
    }
    // Lead-in with the rate of the start month (no curve is defined before the start)
    p.pre_stocks = vec![monthly(c.stocks.at(start)); PRE_MONTHS];
    p
}

impl MarketPath {
    fn push_fixed(&mut self, c: &FixedCurves, m: YearMonth) {
        self.stocks.push(monthly(c.stocks.at(m)));
        self.bonds.push(monthly(c.bonds.at(m)));
        self.inflation.push(c.inflation.at(m));
        self.saron.push(c.saron.at(m));
    }

    pub(crate) fn with_capacity(label: crate::message::Message, n: usize) -> Self {
        MarketPath {
            label,
            stocks: Vec::with_capacity(n),
            bonds: Vec::with_capacity(n),
            inflation: Vec::with_capacity(n),
            saron: Vec::with_capacity(n),
            pre_stocks: Vec::new(),
        }
    }
}

/// Historical path. With annual data one annual value applies per calendar year of the simulation, starting
/// with the year of `hist_start`; with monthly data the series runs month by month from `hist_start`.
/// If the series does not reach the end of the simulation, it is extrapolated according to `ex`.
pub fn historical_path(
    ds: &DataSet,
    hist_start: YearMonth,
    sim_start: YearMonth,
    months: usize,
    ex: &Extrapolation,
    fixed: &FixedCurves,
) -> Result<MarketPath> {
    if hist_start < ds.first || hist_start > ds.last() {
        return Err(Error::Invalid(msg!(
            historical_start_outside,
            start = hist_start,
            name = ds.name,
            first = ds.first,
            last = ds.last()
        )));
    }
    let sim_end = sim_start.plus(months as i32 - 1);
    let label = match ds.freq {
        Freq::Yearly => msg!(path_historical_year, year = hist_start.year()),
        Freq::Monthly => msg!(path_historical, start = hist_start),
    };
    let mut p = MarketPath::with_capacity(label, months);
    // Extrapolation with fixed values: the values of the respective month (may depend on time).
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
            // Lead-in: the months before the start belong, as in the path, to the annual value of their calendar year
            for j in (1..=PRE_MONTHS as i32).rev() {
                let m = sim_start.plus(-j);
                let i = offset as i64 + (m.year() - sim_start.year()) as i64;
                if i < 0 {
                    continue;
                }
                p.pre_stocks.push(monthly(ds.stocks[i as usize]));
            }
        }
        Freq::Monthly => {
            let offset = hist_start.0 - ds.first.0;
            let needed = offset as usize + months;
            let fm = |c: &Curve| monthly(c.at(sim_start));
            let stocks = extend(&ds.stocks, needed, ex, fm(&fixed.stocks), 12);
            let bonds = extend(&ds.bonds, needed, ex, fm(&fixed.bonds), 12);
            // Inflation delivered monthly as an annual rate; for the extrapolation the annual rate suffices.
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
            // Lead-in: the real months before the historical start, as far as the series contains them
            let from = (offset - PRE_MONTHS as i32).max(0) as usize;
            p.pre_stocks.extend_from_slice(&ds.stocks[from..offset as usize]);
        }
    }
    Ok(p)
}

/// All historical start dates for which the series covers the whole simulation period.
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

/// Value `i` of the series; if the series or the value (`NaN`) is missing, the fixed value applies.
fn or_fixed(values: Option<&[f64]>, i: usize, fixed: impl FnOnce() -> f64) -> f64 {
    match values.map(|v| v[i]) {
        Some(v) if !v.is_nan() => v,
        _ => fixed(),
    }
}

/// Extends `values` to `needed` entries (longer series stay unchanged). `per_year` = values per year (for the
/// averaging window of `ex.range` years).
fn extend(values: &[f64], needed: usize, ex: &Extrapolation, fixed: f64, per_year: usize) -> Vec<f64> {
    let mut out = values.to_vec();
    if out.len() >= needed {
        return out;
    }
    let range = (ex.range.max(1) * per_year).min(values.len().max(1));
    // Window of the last `range` values (unused with `Fixed`).
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
        // Start in November 2030: Nov/Dec use 2000, from January 2031 the year 2001.
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

    /// #103: Lead-in with annual data by calendar year: start November 2030 with the year 2001 ⇒ November and December
    /// 2029 from 2000, January to October 2030 from 2001; without a previous year in the series only the months of the start year.
    #[test]
    fn issue_103_yearly_pre_run_by_calendar_year() {
        let ds = DataSet::parse_csv("t", CSV).unwrap();
        let path = |hist: i32| {
            historical_path(
                &ds,
                YearMonth::new(hist, 11),
                YearMonth::new(2030, 11),
                3,
                &Default::default(),
                &flat(),
            )
            .unwrap()
        };
        let p = path(2001);
        assert_eq!(p.pre_stocks.len(), PRE_MONTHS);
        close(p.pre_stocks[0], monthly(0.1));
        close(p.pre_stocks[1], monthly(0.1));
        for r in &p.pre_stocks[2..] {
            close(*r, monthly(-0.1));
        }
        let p = path(2000);
        assert_eq!(p.pre_stocks.len(), 10, "Januar bis Oktober aus 2000");
        close(p.pre_stocks[0], monthly(0.1));
    }

    /// #103: Lead-in with monthly data: the real 12 months before the historical start, fewer at the start of the series.
    #[test]
    fn issue_103_monthly_pre_run_takes_the_months_before_the_start() {
        let ds = DataSet {
            name: "t".into(),
            freq: Freq::Monthly,
            first: YearMonth::new(2000, 1),
            stocks: (0..40).map(|i| i as f64 / 1000.0).collect(),
            bonds: vec![0.0; 40],
            inflation: vec![0.0; 40],
            ..Default::default()
        };
        let path = |hist: YearMonth| {
            historical_path(&ds, hist, YearMonth::new(2030, 7), 6, &Default::default(), &flat()).unwrap()
        };
        let p = path(YearMonth::new(2001, 9)); // Position 20
        let expected: Vec<f64> = (8..20).map(|i| i as f64 / 1000.0).collect();
        assert_eq!(p.pre_stocks, expected);
        close(p.stocks[0], 0.020);
        let p = path(YearMonth::new(2000, 6)); // Position 5
        assert_eq!(p.pre_stocks, vec![0.0, 0.001, 0.002, 0.003, 0.004]);
        assert!(path(YearMonth::new(2000, 1)).pre_stocks.is_empty());
    }

    /// #103: Fixed returns: lead-in with the rate of the start month.
    #[test]
    fn issue_103_fixed_pre_run_uses_the_start_rate() {
        let mut c = flat();
        c.stocks = Curve::constant(0.06);
        let p = fixed_path(&c, YearMonth::new(2030, 7), 3);
        assert_eq!(p.pre_stocks, vec![monthly(0.06); PRE_MONTHS]);
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

    // #40: Name, empty series, means for annual and monthly series.
    #[test]
    fn name_empty_and_averages() {
        let y = DataSet::parse_csv("Jahre", CSV).unwrap();
        assert_eq!(y.name, "Jahre");
        assert!(DataSet::default().is_empty() && !y.is_empty());
        let (s, b, i) = y.averages();
        close(s, (1.1f64 * 0.9 * 1.2).powf(1.0 / 3.0) - 1.0);
        close(b, (1.02f64 * 1.03 * 1.01).powf(1.0 / 3.0) - 1.0);
        close(i, (1.01f64 * 1.02 * 1.0).powf(1.0 / 3.0) - 1.0);

        // 24 months from January 2000: stocks 1 % per month, inflation (annual rate) in December 2 % or 5 %.
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
        // Without December no inflation
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
        // Annual series from the first year: 3 years of data, 5 years of simulation.
        let ds = DataSet::parse_csv("t", CSV).unwrap();
        let p = historical_path(&ds, YearMonth::new(2000, 1), YearMonth::new(2030, 1), 60, &avg, &flat()).unwrap();
        assert_eq!(p.stocks.len(), 60);
        close(p.stocks[36], monthly((0.1 - 0.1 + 0.2) / 3.0));
        // Monthly series from the first month: 3 months of data, 5 months of simulation.
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
        // 2 calendar years: starts 2000 and 2001
        assert_eq!(rolling_starts(&ds, YearMonth::new(2030, 1), 24).len(), 2);
        // 3 calendar years (exactly the length of the series): one start
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
        // Monthly values: window = range years × 12 values.
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
