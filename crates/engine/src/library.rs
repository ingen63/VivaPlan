//! Data library: a folder with series (CSV). Plans refer to it with `library:<file>`,
//! independent of where the project file is.

use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::import::{self, SnbCube, SnbData};
use crate::market::{DataSet, Freq};
use crate::message::Message;
use crate::time::YearMonth;
use crate::{Error, Result};

pub const PREFIX: &str = "library:";

/// US stocks in CHF combined with Swiss federal bonds.
pub const CH_BONDS_FILE: &str = "Shiller CHF + CH-Obligationen.csv";

/// #68: World stock index (JST, GDP-weighted) with Swiss Confederation bonds, annual from 1900.
pub const WORLD_FILE: &str = "Welt (JST).csv";

/// Subfolder of the library with the SNB source data.
pub const SOURCES_DIR: &str = "quellen";

/// Series bundled with the program. #90: «Pictet CH» is no longer part of it (no open
/// licence); a copy created earlier in the library folder stays untouched.
pub const BUILTIN: [(&str, &str); 4] = [
    ("Shiller CHF.csv", include_str!("../data/Shiller CHF.csv")),
    (CH_BONDS_FILE, include_str!("../data/Shiller CHF + CH-Obligationen.csv")),
    ("Shiller USD.csv", include_str!("../data/Shiller USD.csv")),
    (WORLD_FILE, include_str!("../data/Welt (JST).csv")),
];

/// `# Info:` of the bundled series with this name (file name without extension); for files created before the field existed.
fn builtin_info(name: &str) -> Option<Message> {
    let (_, text) = BUILTIN
        .iter()
        .find(|(file, _)| file.strip_suffix(".csv") == Some(name))?;
    crate::market::DataSet::parse_csv(name, text).ok()?.meta.info
}

/// Path to a series: `library:<file>` in the library folder, otherwise absolute or relative
/// to the directory of the project file.
pub fn resolve(series: &str, base_dir: &Path, library: Option<&Path>) -> Result<PathBuf> {
    if let Some(file) = series.strip_prefix(PREFIX) {
        let lib = library.ok_or_else(|| Error::Invalid(msg!(library_unknown, file = file)))?;
        if file.contains(['/', '\\']) || file.starts_with('.') {
            return Err(Error::Invalid(msg!(library_name_invalid, file = file)));
        }
        return Ok(lib.join(file));
    }
    let p = Path::new(series);
    Ok(if p.is_absolute() {
        p.to_path_buf()
    } else {
        base_dir.join(p)
    })
}

/// Creates missing bundled series and SNB source data; existing files stay unchanged.
pub fn seed(dir: &Path) -> Result<()> {
    let io = |e: std::io::Error| crate::io_error(dir, e);
    let src = dir.join(SOURCES_DIR);
    std::fs::create_dir_all(&src).map_err(io)?;
    for (name, text) in BUILTIN {
        write_new(&dir.join(name), text)?;
    }
    for (name, text) in import::BUILTIN_SNB {
        write_new(&src.join(name), text)?;
    }
    Ok(())
}

fn write_new(path: &Path, text: &str) -> Result<()> {
    if path.exists() {
        return Ok(());
    }
    std::fs::write(path, text).map_err(|e| crate::io_error(path, e))
}

/// #69/AK8: A part of an investment strategy with its own years. If the range is larger than the common
/// period of the series, the part is cut for the simulation.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeriesPart {
    /// For example «Bonds Swiss Confederation 10 years» (#177: message with code).
    pub label: Message,
    pub first: i32,
    pub last: i32,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeriesInfo {
    pub name: String,
    /// #177: Title, source, info and notes as messages (texts of the bundled series with their code).
    pub title: Option<Message>,
    pub currency: Option<String>,
    pub source: Option<Message>,
    /// Short explanation (at most 2 lines): `# Info:` of the file; for bundled series without their own, the bundled one.
    pub info: Option<Message>,
    pub notes: Vec<Message>,
    pub monthly: bool,
    pub first: YearMonth,
    pub last: YearMonth,
    pub entries: usize,
    pub has_saron: bool,
    /// First and last month with a SARON value.
    pub saron_range: Option<(YearMonth, YearMonth)>,
    /// Mean annual return (geometric) of stocks, bonds; mean inflation.
    pub avg_stocks: f64,
    pub avg_bonds: f64,
    pub avg_inflation: f64,
    /// Parts of the investment strategy with their own years (empty for a single series).
    pub parts: Vec<SeriesPart>,
    /// #100/#101: Suggested values of the parametric model (only in the short info about the plan).
    pub parametric: Option<crate::calibrate::Suggestion>,
}

impl SeriesInfo {
    pub fn of(ds: &DataSet) -> SeriesInfo {
        let (avg_stocks, avg_bonds, avg_inflation) = ds.averages();
        let saron_range = ds.saron.as_ref().and_then(|s| {
            let at = |i: usize| match ds.freq {
                Freq::Monthly => ds.first.plus(i as i32),
                Freq::Yearly => YearMonth::new(ds.first.year() + i as i32, 1),
            };
            let a = s.iter().position(|v| !v.is_nan())?;
            let b = s.iter().rposition(|v| !v.is_nan())?;
            Some((at(a), at(b)))
        });
        SeriesInfo {
            name: ds.name.clone(),
            title: ds.meta.title.clone(),
            currency: ds.meta.currency.clone(),
            source: ds.meta.source.clone(),
            info: ds.meta.info.clone().or_else(|| builtin_info(&ds.name)),
            notes: ds.meta.notes.clone(),
            monthly: ds.freq == Freq::Monthly,
            first: ds.first,
            last: ds.last(),
            entries: ds.len(),
            has_saron: saron_range.is_some(),
            saron_range,
            avg_stocks,
            avg_bonds,
            avg_inflation,
            parts: vec![],
            parametric: None,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Entry {
    /// Reference for the plan (`library:<file>`).
    pub reference: String,
    pub file: String,
    pub info: Option<SeriesInfo>,
    pub error: Option<Message>,
}

/// All series in the library folder, sorted by file name.
pub fn list(dir: &Path) -> Result<Vec<Entry>> {
    let rd = std::fs::read_dir(dir).map_err(|e| crate::io_error(dir, e))?;
    let mut files: Vec<PathBuf> = rd
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.is_file() && p.extension().is_some_and(|x| x.eq_ignore_ascii_case("csv")))
        .collect();
    files.sort();
    Ok(files
        .into_iter()
        .map(|p| {
            let file = p.file_name().unwrap_or_default().to_string_lossy().into_owned();
            let (info, error) = match DataSet::load_csv(&p) {
                Ok(ds) => (Some(SeriesInfo::of(&ds)), None),
                Err(e) => (None, Some(e.message())),
            };
            Entry {
                reference: format!("{PREFIX}{file}"),
                file,
                info,
                error,
            }
        })
        .collect())
}

/// SNB source data of the library (`quellen/snb-*.csv`), otherwise the built-in data.
pub fn snb_data(dir: &Path) -> Result<SnbData> {
    let src = dir.join(SOURCES_DIR);
    let mut cubes: Vec<SnbCube> = import::BUILTIN_SNB
        .iter()
        .map(|(_, t)| import::parse_snb(t))
        .collect::<Result<_>>()?;
    if let Ok(rd) = std::fs::read_dir(&src) {
        let mut files: Vec<PathBuf> = rd.filter_map(|e| e.ok().map(|e| e.path())).collect();
        files.sort();
        // Only the files of the SNB import (`snb-<cube>.csv`); other sources lie next to them (JST extract).
        let snb_file = |p: &&PathBuf| {
            p.extension().is_some_and(|x| x == "csv")
                && p.file_name().is_some_and(|n| n.to_string_lossy().starts_with("snb-"))
        };
        for p in files.iter().filter(snb_file) {
            cubes.push(import::read_snb(p)?);
        }
    }
    SnbData::from_cubes(&cubes)
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ImportReport {
    /// Written files (name in the library) with short info.
    pub written: Vec<(String, SeriesInfo)>,
    pub notes: Vec<Message>,
}

fn write_series(dir: &Path, file: &str, ds: &DataSet) -> Result<(String, SeriesInfo)> {
    let path = dir.join(file);
    std::fs::write(&path, ds.to_csv()).map_err(|e| crate::io_error(&path, e))?;
    Ok((file.to_string(), SeriesInfo::of(ds)))
}

/// Import a Shiller file: writes «Shiller USD.csv» and «Shiller CHF.csv» into the library.
pub fn import_shiller(dir: &Path, shiller: &Path) -> Result<ImportReport> {
    import_shiller_raw(dir, &import::read_shiller(shiller)?)
}

/// Like [`import_shiller`], with the columns already read (testable without a spreadsheet file).
fn import_shiller_raw(dir: &Path, raw: &import::ShillerRaw) -> Result<ImportReport> {
    let usd = import::shiller_dataset(raw)?;
    let snb = snb_data(dir)?;
    let chf = import::to_chf(&usd, &snb)?;
    let ch = import::with_ch_bonds(&chf, &snb)?;
    let mut notes = vec![];
    if chf.last() < usd.last() {
        notes.push(msg!(import_chf_ends, month = chf.last()));
    }
    Ok(ImportReport {
        written: vec![
            write_series(dir, "Shiller USD.csv", &usd)?,
            write_series(dir, "Shiller CHF.csv", &chf)?,
            write_series(dir, CH_BONDS_FILE, &ch)?,
        ],
        notes,
    })
}

/// Take over SNB cubes (CSV from data.snb.ch) and recompute the CHF series.
pub fn import_snb(dir: &Path, files: &[PathBuf]) -> Result<ImportReport> {
    let src = dir.join(SOURCES_DIR);
    std::fs::create_dir_all(&src).map_err(|e| crate::io_error(&src, e))?;
    let mut notes = vec![];
    for f in files {
        let cube = import::read_snb(f)?;
        let text = SnbData::extract_csv(&cube)?;
        let target = src.join(format!("snb-{}.csv", cube.id));
        std::fs::write(&target, text).map_err(|e| crate::io_error(&target, e))?;
        notes.push(match &cube.published {
            Some(p) => msg!(import_snb_cube_dated, id = cube.id, published = p),
            None => msg!(import_snb_cube, id = cube.id),
        });
    }
    let mut written = vec![];
    let usd_path = dir.join("Shiller USD.csv");
    if usd_path.exists() {
        let usd = DataSet::load_csv(&usd_path)?;
        let snb = snb_data(dir)?;
        let chf = import::to_chf(&usd, &snb)?;
        written.push(write_series(dir, "Shiller CHF.csv", &chf)?);
        written.push(write_series(dir, CH_BONDS_FILE, &import::with_ch_bonds(&chf, &snb)?)?);
    } else {
        notes.push(msg!(import_shiller_usd_missing));
    }
    Ok(ImportReport { written, notes })
}

#[cfg(test)]
mod tests {
    #[test]
    fn shipped_series_have_a_short_info_of_at_most_two_lines() {
        for (file, text) in BUILTIN {
            let ds = crate::market::DataSet::parse_csv(file, text).unwrap();
            let info = ds.meta.info.unwrap_or_else(|| panic!("{file}: «# Info:» fehlt"));
            assert!(
                info.chars().count() <= 190,
                "{file}: Info zu lang für 2 Zeilen ({})",
                info.chars().count()
            );
            // Files without info (older than the field) get the bundled one
            let name = file.strip_suffix(".csv").unwrap();
            assert_eq!(builtin_info(name).as_deref(), Some(info.as_str()));
        }
        assert_eq!(builtin_info("Eigene"), None);
    }

    use super::*;

    #[test]
    fn resolve_references() {
        let base = Path::new("/plaene");
        let lib = Path::new("/bib");
        assert_eq!(
            resolve("library:Shiller CHF.csv", base, Some(lib)).unwrap(),
            Path::new("/bib/Shiller CHF.csv")
        );
        assert_eq!(resolve("x.csv", base, None).unwrap(), Path::new("/plaene/x.csv"));
        assert!(resolve("library:../x.csv", base, Some(lib)).is_err());
        // #40: each of the two conditions alone suffices
        assert!(resolve("library:.x.csv", base, Some(lib)).is_err());
        assert!(resolve("library:a/x.csv", base, Some(lib)).is_err());
        assert!(resolve("library:a\\x.csv", base, Some(lib)).is_err());
        assert!(resolve("library:x.csv", base, None).is_err());
    }

    #[test]
    fn builtin_series_are_valid() {
        for (name, text) in BUILTIN {
            let ds = DataSet::parse_csv(name, text).unwrap();
            assert_eq!(
                ds.freq,
                if name == WORLD_FILE {
                    Freq::Yearly
                } else {
                    Freq::Monthly
                }
            );
            assert!(ds.currency().is_some());
        }
    }

    /// Temporary folder that is deleted when the returned guard is dropped (end of the test).
    fn scratch(tag: &str) -> (tempfile::TempDir, PathBuf) {
        let d = tempfile::Builder::new()
            .prefix(&format!("vivaplan-lib-{tag}-"))
            .tempdir()
            .unwrap();
        let p = d.path().to_path_buf();
        (d, p)
    }

    // #40: Short info of a monthly series with SARON (first and last value, NaN at the edge) and of an annual series.
    #[test]
    fn series_info_saron_range_and_frequency() {
        let monthly = DataSet::parse_csv(
            "m",
            "Date,Stocks,Bonds,Inflation,Saron\n2000-01,0.01,0,0,\n2000-02,0.01,0,0,0.01\n2000-03,0.01,0,0,0.02\n2000-04,0.01,0,0,\n",
        )
        .unwrap();
        let i = SeriesInfo::of(&monthly);
        assert!(i.monthly && i.has_saron);
        assert_eq!(i.saron_range, Some((YearMonth::new(2000, 2), YearMonth::new(2000, 3))));
        assert_eq!(i.entries, 4);

        let yearly = DataSet::parse_csv(
            "y",
            "Years,Stocks,Bonds,Inflation,Saron\n2000,0.05,0,0,\n2001,0.05,0,0,0.01\n2002,0.05,0,0,0.02\n",
        )
        .unwrap();
        let i = SeriesInfo::of(&yearly);
        assert!(!i.monthly);
        assert_eq!(i.saron_range, Some((YearMonth::new(2001, 1), YearMonth::new(2002, 1))));
    }

    #[test]
    fn list_reads_only_csv_files() {
        let (_tmp, dir) = scratch("list");
        std::fs::write(dir.join("notiz.txt"), "x").unwrap();
        std::fs::create_dir_all(dir.join("ordner.csv")).unwrap();
        std::fs::write(dir.join("b.csv"), "Date,Stocks,Bonds,Inflation\n2000-01,0.01,0,0\n").unwrap();
        std::fs::write(dir.join("a.CSV"), "kaputt").unwrap();
        let l = list(&dir).unwrap();
        assert_eq!(
            l.iter().map(|e| e.file.as_str()).collect::<Vec<_>>(),
            ["a.CSV", "b.csv"]
        );
        assert!(l[0].error.is_some() && l[0].info.is_none());
        assert_eq!(l[1].reference, "library:b.csv");
        assert!(l[1].info.is_some() && l[1].error.is_none());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn snb_data_uses_csv_sources_only() {
        let (_tmp, dir) = scratch("snb");
        let builtin = import::SnbData::builtin();
        assert_eq!(snb_data(&dir).unwrap().usd_chf, builtin.usd_chf);
        std::fs::create_dir_all(dir.join(SOURCES_DIR)).unwrap();
        std::fs::write(dir.join(SOURCES_DIR).join("liesmich.txt"), "keine SNB-Datei").unwrap();
        // Other sources in the folder (JST extract) are not SNB files.
        std::fs::write(dir.join(SOURCES_DIR).join("jst-r6.csv"), "year,iso\n").unwrap();
        assert_eq!(snb_data(&dir).unwrap().cpi, builtin.cpi);
        // A file of its own replaces the built-in series.
        std::fs::write(
            dir.join(SOURCES_DIR).join("snb-plkopr.csv"),
            "\"CubeId\";\"plkopr\"\n\n\"Date\";\"D0\";\"Value\"\n\"2000-01\";\"LD2010100\";\"100\"\n",
        )
        .unwrap();
        assert_eq!(snb_data(&dir).unwrap().cpi.len(), 1);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn shiller_import_notes_when_snb_data_end_earlier() {
        let (_tmp, dir) = scratch("shiller");
        let mut raw = import::ShillerRaw::default();
        // 2020-01 to 2099-12: longer than the built-in SNB data.
        for i in 0..960 {
            let m = YearMonth::new(2020, 1).plus(i);
            raw.cpi.insert(m, 100.0 + i as f64);
            raw.real_tr_price.insert(m, 50.0 + i as f64);
            raw.real_tr_bonds.insert(m, 10.0);
        }
        let r = import_shiller_raw(&dir, &raw).unwrap();
        assert_eq!(r.written.len(), 3);
        assert_eq!(r.notes.len(), 1, "{:?}", r.notes);
        assert!(r.notes[0].contains("The CHF series ends"));
        assert!(dir.join("Shiller USD.csv").exists() && dir.join(CH_BONDS_FILE).exists());

        // If the series does not extend beyond the SNB data, there is no warning.
        let snb = import::SnbData::builtin();
        let snb_end = *snb
            .usd_chf
            .keys()
            .next_back()
            .unwrap()
            .min(snb.cpi.keys().next_back().unwrap());
        raw.cpi.retain(|m, _| *m <= snb_end);
        let r = import_shiller_raw(&dir, &raw).unwrap();
        assert!(r.notes.is_empty(), "{:?}", r.notes);
        let _ = std::fs::remove_dir_all(&dir);
    }
}
