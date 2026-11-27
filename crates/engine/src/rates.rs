//! #307: Swiss interest environment of a historical month: short-term rate (SARON, before its introduction the
//! JST rate of short-term Swiss Confederation paper) and yield of 10-year Swiss Confederation bonds.
//!
//! Frequency of the sources: JST (`bill_rate`, `bond_rate` of Switzerland) has one value per calendar year (until 2020);
//! it applies to every month of the year. The SNB yields of the 10-year bonds (`rendoblim`, from 01.1988) and SARON
//! (`zimoma`, from 06.1999) are monthly. Annual series take the annual value: JST, after its last year the mean of the
//! twelve SNB months.

use std::path::Path;

use crate::import::SnbData;
use crate::jst::JstData;
use crate::market::{DataSet, Freq};
use crate::time::YearMonth;
use crate::Result;

/// Historical rates of Switzerland (annual rates as shares).
pub struct RateHistory {
    jst: JstData,
    snb: SnbData,
}

impl RateHistory {
    /// The bundled data, or the SNB data of the data library.
    pub fn load(library: Option<&Path>) -> Result<RateHistory> {
        Ok(RateHistory {
            jst: JstData::builtin(),
            snb: match library {
                Some(d) => crate::library::snb_data(d)?,
                None => SnbData::builtin(),
            },
        })
    }

    pub fn new(jst: JstData, snb: SnbData) -> RateHistory {
        RateHistory { jst, snb }
    }

    fn jst_short(&self, year: i32) -> Option<f64> {
        self.jst.row("CHE", year)?.bill_rate
    }

    fn jst_yield10(&self, year: i32) -> Option<f64> {
        self.jst.row("CHE", year)?.bond_rate
    }

    /// Mean of the twelve monthly SNB values of a year in percent, as a share; `None` if a month is missing.
    fn snb_year(values: &crate::import::Monthly, year: i32) -> Option<f64> {
        let mut sum = 0.0;
        for m in 1..=12 {
            sum += values.get(&YearMonth::new(year, m))?;
        }
        Some(sum / 12.0 / 100.0)
    }

    /// Short-term rate of a month: SARON, before it the JST rate of its year.
    pub fn short_month(&self, m: YearMonth) -> Option<f64> {
        match self.snb.saron.get(&m) {
            Some(v) => Some(v / 100.0),
            None => self.jst_short(m.year()),
        }
    }

    /// Yield of 10-year Swiss Confederation bonds in a month: SNB, before it the JST yield of its year.
    pub fn yield10_month(&self, m: YearMonth) -> Option<f64> {
        match self.snb.ch_yield10.get(&m) {
            Some(v) => Some(v / 100.0),
            None => self.jst_yield10(m.year()),
        }
    }

    /// Short-term rate of a calendar year: JST, after its last year the mean of SARON.
    pub fn short_year(&self, year: i32) -> Option<f64> {
        self.jst_short(year).or_else(|| Self::snb_year(&self.snb.saron, year))
    }

    /// Yield of 10-year bonds of a calendar year: JST, after its last year the mean of the SNB yields.
    pub fn yield10_year(&self, year: i32) -> Option<f64> {
        self.jst_yield10(year)
            .or_else(|| Self::snb_year(&self.snb.ch_yield10, year))
    }

    /// Completes a series with the interest environment of its months or years: missing SARON values (`NaN` or no
    /// column) get the short-term rate, and the yield of the 10-year bonds is added. Values the sources do not cover
    /// stay `NaN` (the fixed values apply there).
    pub fn complete(&self, ds: &mut DataSet) {
        let n = ds.len();
        let at = |i: usize| match ds.freq {
            Freq::Monthly => ds.first.plus(i as i32),
            Freq::Yearly => YearMonth::new(ds.first.year() + i as i32, 1),
        };
        let short = |m: YearMonth| match ds.freq {
            Freq::Monthly => self.short_month(m),
            Freq::Yearly => self.short_year(m.year()),
        };
        let long = |m: YearMonth| match ds.freq {
            Freq::Monthly => self.yield10_month(m),
            Freq::Yearly => self.yield10_year(m.year()),
        };
        let mut saron = ds.saron.clone().unwrap_or_else(|| vec![f64::NAN; n]);
        saron.resize(n, f64::NAN);
        for (i, v) in saron.iter_mut().enumerate() {
            if v.is_nan() {
                *v = short(at(i)).unwrap_or(f64::NAN);
            }
        }
        let yield10 = (0..n).map(|i| long(at(i)).unwrap_or(f64::NAN)).collect();
        ds.saron = Some(saron);
        ds.yield10 = Some(yield10);
        // The notes of the bundled series said that the fixed value applies before SARON.
        for note in &mut ds.meta.notes {
            if note.code() == "series_note_chf_inflation" {
                *note = msg!(series_note_chf_inflation_rates);
            }
        }
        if !ds.meta.notes.iter().any(|n| n.code() == "series_note_rates") {
            ds.meta.notes.push(msg!(series_note_rates));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn history() -> RateHistory {
        RateHistory::new(JstData::builtin(), SnbData::builtin())
    }

    #[test]
    fn issue_307_monthly_values_from_snb_and_annual_values_from_jst() {
        let h = history();
        // JST 1974: bill rate 5.97 %, bond yield 7.15 % (data/quellen/jst-r6.csv), the same in every month
        assert!((h.short_month(YearMonth::new(1974, 3)).unwrap() - 0.0597).abs() < 1e-6);
        assert!((h.short_month(YearMonth::new(1974, 11)).unwrap() - 0.0597).abs() < 1e-6);
        assert!((h.yield10_month(YearMonth::new(1974, 3)).unwrap() - 0.0715).abs() < 1e-6);
        assert!((h.short_year(1974).unwrap() - 0.0597).abs() < 1e-6);
        // SNB from 1988: 10 years in 01.1988 4.092 % (data/quellen/snb-rendoblim.csv)
        assert!((h.yield10_month(YearMonth::new(1988, 1)).unwrap() - 0.04092).abs() < 1e-9);
        // After the last JST year the annual value is the mean of the SNB months
        let y = h.jst.last_year() + 1;
        let mean: f64 = (1..=12).map(|m| h.snb.saron[&YearMonth::new(y, m)]).sum::<f64>() / 1200.0;
        assert!((h.short_year(y).unwrap() - mean).abs() < 1e-12);
        assert!(h.short_month(YearMonth::new(1800, 1)).is_none());
    }

    #[test]
    fn issue_307_ak1_complete_fills_saron_gaps_and_adds_the_yield() {
        let h = history();
        let mut ds = DataSet {
            freq: Freq::Monthly,
            first: YearMonth::new(1974, 11),
            stocks: vec![0.0; 3],
            bonds: vec![0.0; 3],
            inflation: vec![0.0; 3],
            saron: Some(vec![f64::NAN, 0.09, f64::NAN]),
            ..Default::default()
        };
        h.complete(&mut ds);
        let s = ds.saron.as_ref().unwrap();
        // Nov. 1974 from JST 1974, Dec. stays, Jan. 1975 from JST 1975 (2.75 %)
        assert!((s[0] - 0.0597).abs() < 1e-6);
        assert_eq!(s[1], 0.09);
        assert!((s[2] - 0.0275).abs() < 1e-6);
        let y = ds.yield10.as_ref().unwrap();
        assert!((y[2] - 0.0644).abs() < 1e-6);
        assert!(ds.meta.notes.iter().any(|n| n.code() == "series_note_rates"));
        // Without a SARON column; annual series
        let mut ds = DataSet {
            freq: Freq::Yearly,
            first: YearMonth::new(1799, 1),
            stocks: vec![0.0; 2],
            bonds: vec![0.0; 2],
            inflation: vec![0.0; 2],
            ..Default::default()
        };
        h.complete(&mut ds);
        assert!(ds.saron.as_ref().unwrap()[0].is_nan(), "vor den Daten");
        assert!(ds.yield10.as_ref().unwrap()[1].is_nan(), "1800 ohne Werte");
    }
}
