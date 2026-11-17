//! Datenbibliothek: ein Ordner mit Zeitreihen (CSV). Planungen verweisen mit `library:<datei>`
//! darauf, unabhängig davon, wo die Projektdatei liegt.

use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::import::{self, SnbCube, SnbData};
use crate::market::{DataSet, Freq};
use crate::time::YearMonth;
use crate::{Error, Result};

pub const PREFIX: &str = "library:";

/// US-Aktien in CHF kombiniert mit Schweizer Bundesobligationen.
pub const CH_BONDS_FILE: &str = "Shiller CHF + CH-Obligationen.csv";

/// #68: Welt-Aktienindex (JST, nach BIP gewichtet) mit Obligationen der Eidgenossenschaft, jährlich ab 1900.
pub const WORLD_FILE: &str = "Welt (JST).csv";

/// Unterordner der Bibliothek mit den Quelldaten der SNB.
pub const SOURCES_DIR: &str = "quellen";

/// Zeitreihen, die mit dem Programm ausgeliefert werden. #90: «Pictet CH» gehört nicht mehr dazu (keine offene
/// Lizenz); eine früher angelegte Kopie im Bibliotheksordner bleibt unangetastet.
pub const BUILTIN: [(&str, &str); 4] = [
    ("Shiller CHF.csv", include_str!("../data/Shiller CHF.csv")),
    (CH_BONDS_FILE, include_str!("../data/Shiller CHF + CH-Obligationen.csv")),
    ("Shiller USD.csv", include_str!("../data/Shiller USD.csv")),
    (WORLD_FILE, include_str!("../data/Welt (JST).csv")),
];

/// `# Info:` der ausgelieferten Reihe mit diesem Namen (Dateiname ohne Endung); für Dateien, die vor der Angabe angelegt wurden.
fn builtin_info(name: &str) -> Option<String> {
    let (_, text) = BUILTIN
        .iter()
        .find(|(file, _)| file.strip_suffix(".csv") == Some(name))?;
    crate::market::DataSet::parse_csv(name, text).ok()?.meta.info
}

/// Pfad zu einer Zeitreihe: `library:<datei>` im Bibliotheksordner, sonst absolut oder relativ
/// zum Verzeichnis der Projektdatei.
pub fn resolve(series: &str, base_dir: &Path, library: Option<&Path>) -> Result<PathBuf> {
    if let Some(file) = series.strip_prefix(PREFIX) {
        let lib = library.ok_or_else(|| {
            Error::Invalid(format!(
                "Die Zeitreihe «{file}» liegt in der Datenbibliothek, deren Ordner hier nicht bekannt ist."
            ))
        })?;
        if file.contains(['/', '\\']) || file.starts_with('.') {
            return Err(Error::Invalid(format!(
                "Ungültiger Name in der Datenbibliothek: «{file}»"
            )));
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

/// Legt fehlende ausgelieferte Zeitreihen und SNB-Quelldaten an; vorhandene Dateien bleiben unverändert.
pub fn seed(dir: &Path) -> Result<()> {
    let io = |e: std::io::Error| Error::Io(format!("{}: {e}", dir.display()));
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
    std::fs::write(path, text).map_err(|e| Error::Io(format!("{}: {e}", path.display())))
}

/// #69/AK8: Ein Teil einer Anlagestrategie mit seinen eigenen Jahren. Ist der Bereich grösser als der gemeinsame
/// Zeitraum der Reihe, wird der Teil für die Simulation gekürzt.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeriesPart {
    /// Zum Beispiel «Obligationen Eidgenossenschaft 10 Jahre».
    pub label: String,
    pub first: i32,
    pub last: i32,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SeriesInfo {
    pub name: String,
    pub title: Option<String>,
    pub currency: Option<String>,
    pub source: Option<String>,
    /// Kurze Erklärung (höchstens 2 Zeilen): `# Info:` der Datei, bei ausgelieferten Reihen ohne eigene Angabe die ausgelieferte.
    pub info: Option<String>,
    pub notes: Vec<String>,
    pub monthly: bool,
    pub first: YearMonth,
    pub last: YearMonth,
    pub entries: usize,
    pub has_saron: bool,
    /// Erster und letzter Monat mit SARON-Wert.
    pub saron_range: Option<(YearMonth, YearMonth)>,
    /// Mittlere Jahresrendite (geometrisch) Aktien, Obligationen; mittlere Inflation.
    pub avg_stocks: f64,
    pub avg_bonds: f64,
    pub avg_inflation: f64,
    /// Teile der Anlagestrategie mit ihren eigenen Jahren (leer bei einer einzelnen Zeitreihe).
    pub parts: Vec<SeriesPart>,
    /// #100/#101: Vorschlagswerte des parametrischen Modells (nur bei der Kurzinfo zur Planung).
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
    /// Verweis für die Planung (`library:<datei>`).
    pub reference: String,
    pub file: String,
    pub info: Option<SeriesInfo>,
    pub error: Option<String>,
}

/// Alle Zeitreihen im Bibliotheksordner, nach Dateiname sortiert.
pub fn list(dir: &Path) -> Result<Vec<Entry>> {
    let rd = std::fs::read_dir(dir).map_err(|e| Error::Io(format!("{}: {e}", dir.display())))?;
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
                Err(e) => (None, Some(e.to_string())),
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

/// SNB-Quelldaten der Bibliothek (`quellen/snb-*.csv`), sonst die eingebauten.
pub fn snb_data(dir: &Path) -> Result<SnbData> {
    let src = dir.join(SOURCES_DIR);
    let mut cubes: Vec<SnbCube> = import::BUILTIN_SNB
        .iter()
        .map(|(_, t)| import::parse_snb(t))
        .collect::<Result<_>>()?;
    if let Ok(rd) = std::fs::read_dir(&src) {
        let mut files: Vec<PathBuf> = rd.filter_map(|e| e.ok().map(|e| e.path())).collect();
        files.sort();
        // Nur die Dateien des SNB-Imports (`snb-<würfel>.csv`); daneben liegen andere Quellen (JST-Auszug).
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
    /// Geschriebene Dateien (Name in der Bibliothek) mit Kurzinfo.
    pub written: Vec<(String, SeriesInfo)>,
    pub notes: Vec<String>,
}

fn write_series(dir: &Path, file: &str, ds: &DataSet) -> Result<(String, SeriesInfo)> {
    let path = dir.join(file);
    std::fs::write(&path, ds.to_csv()).map_err(|e| Error::Io(format!("{}: {e}", path.display())))?;
    Ok((file.to_string(), SeriesInfo::of(ds)))
}

/// Shiller-Datei importieren: schreibt «Shiller USD.csv» und «Shiller CHF.csv» in die Bibliothek.
pub fn import_shiller(dir: &Path, shiller: &Path) -> Result<ImportReport> {
    import_shiller_raw(dir, &import::read_shiller(shiller)?)
}

/// Wie [`import_shiller`], mit den bereits gelesenen Spalten (ohne Tabellendatei testbar).
fn import_shiller_raw(dir: &Path, raw: &import::ShillerRaw) -> Result<ImportReport> {
    let usd = import::shiller_dataset(raw)?;
    let snb = snb_data(dir)?;
    let chf = import::to_chf(&usd, &snb)?;
    let ch = import::with_ch_bonds(&chf, &snb)?;
    let mut notes = vec![];
    if chf.last() < usd.last() {
        notes.push(format!(
            "Die CHF-Reihe endet {}, weil die SNB-Daten (Kurs bzw. Landesindex) nicht weiter reichen. Neuere SNB-Daten lassen sich ebenfalls importieren.",
            chf.last()
        ));
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

/// SNB-Würfel (CSV von data.snb.ch) übernehmen und die CHF-Reihen neu berechnen.
pub fn import_snb(dir: &Path, files: &[PathBuf]) -> Result<ImportReport> {
    let src = dir.join(SOURCES_DIR);
    std::fs::create_dir_all(&src).map_err(|e| Error::Io(format!("{}: {e}", src.display())))?;
    let mut notes = vec![];
    for f in files {
        let cube = import::read_snb(f)?;
        let text = SnbData::extract_csv(&cube)?;
        let target = src.join(format!("snb-{}.csv", cube.id));
        std::fs::write(&target, text).map_err(|e| Error::Io(format!("{}: {e}", target.display())))?;
        notes.push(format!(
            "SNB-Würfel {} übernommen{}.",
            cube.id,
            cube.published.map(|p| format!(" (Stand {p})")).unwrap_or_default()
        ));
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
        notes.push("«Shiller USD.csv» fehlt in der Bibliothek; die CHF-Reihe wurde nicht neu berechnet.".into());
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
            // Dateien ohne Info (älter als die Angabe) erhalten die ausgelieferte
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
        // #40: jede der beiden Bedingungen allein genügt
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

    fn scratch(tag: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("pp-lib-unit-{tag}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    // #40: Kurzinfo einer Monatsreihe mit SARON (erster und letzter Wert, NaN am Rand) und einer Jahresreihe.
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
        let dir = scratch("list");
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
        let dir = scratch("snb");
        let builtin = import::SnbData::builtin();
        assert_eq!(snb_data(&dir).unwrap().usd_chf, builtin.usd_chf);
        std::fs::create_dir_all(dir.join(SOURCES_DIR)).unwrap();
        std::fs::write(dir.join(SOURCES_DIR).join("liesmich.txt"), "keine SNB-Datei").unwrap();
        // Andere Quellen im Ordner (JST-Auszug) sind keine SNB-Dateien.
        std::fs::write(dir.join(SOURCES_DIR).join("jst-r6.csv"), "year,iso\n").unwrap();
        assert_eq!(snb_data(&dir).unwrap().cpi, builtin.cpi);
        // Eine eigene Datei ersetzt die eingebaute Reihe.
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
        let dir = scratch("shiller");
        let mut raw = import::ShillerRaw::default();
        // 2020-01 bis 2099-12: länger als die eingebauten SNB-Daten.
        for i in 0..960 {
            let m = YearMonth::new(2020, 1).plus(i);
            raw.cpi.insert(m, 100.0 + i as f64);
            raw.real_tr_price.insert(m, 50.0 + i as f64);
            raw.real_tr_bonds.insert(m, 10.0);
        }
        let r = import_shiller_raw(&dir, &raw).unwrap();
        assert_eq!(r.written.len(), 3);
        assert_eq!(r.notes.len(), 1, "{:?}", r.notes);
        assert!(r.notes[0].contains("Die CHF-Reihe endet"));
        assert!(dir.join("Shiller USD.csv").exists() && dir.join(CH_BONDS_FILE).exists());

        // Reicht die Reihe nicht über die SNB-Daten hinaus, gibt es keinen Hinweis.
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
