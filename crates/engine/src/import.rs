//! Import von Zeitreihen: Shiller-Daten (Excel/ODS), SNB-Datenwürfel (CSV) und Umrechnung in CHF.
//!
//! Shiller («Irrational Exuberance», `ie_data.xls`): S&P Composite und 10-jährige US-Staatsanleihen,
//! monatlich ab 1871 in USD. Die SNB-Würfel liefern den Kurs USD/CHF (`devkum`), den Landesindex der
//! Konsumentenpreise (`plkopr`), den SARON (`zimoma`) und die Renditen der Obligationen der
//! Eidgenossenschaft (`rendoblim`).

use std::collections::BTreeMap;
use std::path::Path;

use calamine::{open_workbook_auto, Data, Reader};

use crate::market::{DataSet, Freq, Meta};
use crate::time::YearMonth;
use crate::{Error, Result};

/// Monatswerte mit Lücken.
pub type Monthly = BTreeMap<YearMonth, f64>;

fn invalid(s: impl Into<String>) -> Error {
    Error::Invalid(s.into())
}

// ---------- Shiller ----------

/// Die benötigten Spalten der Shiller-Tabelle, Monat für Monat.
#[derive(Clone, Debug, Default)]
pub struct ShillerRaw {
    pub cpi: Monthly,
    /// Real Total Return Price (Aktien inkl. Dividenden, real).
    pub real_tr_price: Monthly,
    /// Real Total Bond Returns (Index, real).
    pub real_tr_bonds: Monthly,
}

/// Liest das Blatt «Data» der Shiller-Datei (`.xls`, `.xlsx` oder `.ods`).
pub fn read_shiller(path: &Path) -> Result<ShillerRaw> {
    let mut wb = open_workbook_auto(path).map_err(|e| Error::Io(format!("{}: {e}", path.display())))?;
    let range = wb
        .worksheet_range("Data")
        .map_err(|e| invalid(format!("{}: Blatt «Data» fehlt ({e})", path.display())))?;
    let rows: Vec<Vec<Data>> = range.rows().map(|r| r.to_vec()).collect();
    parse_shiller(&rows)
}

fn cell_text(c: &Data) -> String {
    match c {
        Data::String(s) => s.trim().to_string(),
        _ => String::new(),
    }
}

fn cell_num(c: Option<&Data>) -> Option<f64> {
    match c? {
        Data::Float(v) => Some(*v),
        Data::Int(v) => Some(*v as f64),
        Data::String(s) => s.trim().parse().ok(),
        _ => None,
    }
    .filter(|v: &f64| v.is_finite())
}

/// Shiller schreibt den Monat als Dezimalzahl: 1871.01 = Januar, 2025.1 = Oktober.
fn shiller_month(v: f64) -> Option<YearMonth> {
    let y = v.floor();
    let m = ((v - y) * 100.0).round() as u32;
    (1..=12).contains(&m).then(|| YearMonth::new(y as i32, m))
}

pub fn parse_shiller(rows: &[Vec<Data>]) -> Result<ShillerRaw> {
    // Kopfzeile: erste Spalte «Date»; die Spaltennamen stehen über mehrere Zeilen verteilt.
    let head = rows
        .iter()
        .position(|r| r.first().is_some_and(|c| cell_text(c) == "Date"))
        .ok_or_else(|| invalid("Shiller-Datei: Kopfzeile «Date» nicht gefunden"))?;
    let width = rows[head].len();
    let label = |c: usize| {
        let mut parts = vec![];
        for r in rows[..=head].iter().rev().take(6) {
            let t = r.get(c).map(cell_text).unwrap_or_default();
            if !t.is_empty() {
                parts.push(t);
            }
        }
        parts.reverse();
        parts
            .join(" ")
            .to_lowercase()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    };
    let labels: Vec<String> = (0..width).map(label).collect();
    let col = |want: &str| {
        labels
            .iter()
            .position(|l| l.contains(want))
            .ok_or_else(|| invalid(format!("Shiller-Datei: Spalte «{want}» nicht gefunden")))
    };
    let c_cpi = col("cpi")?;
    let c_tr = col("real total return price")?;
    let c_bond = col("real total bond returns")?;

    let mut raw = ShillerRaw::default();
    for r in &rows[head + 1..] {
        let Some(m) = cell_num(r.first()).and_then(shiller_month) else {
            continue;
        };
        for (c, map) in [
            (c_cpi, &mut raw.cpi),
            (c_tr, &mut raw.real_tr_price),
            (c_bond, &mut raw.real_tr_bonds),
        ] {
            if let Some(v) = cell_num(r.get(c)).filter(|v| *v > 0.0) {
                map.insert(m, v);
            }
        }
    }
    if raw.cpi.len() < 13 {
        return Err(invalid("Shiller-Datei: zu wenige Monatswerte"));
    }
    Ok(raw)
}

/// Monatliche Reihe in USD: nominale Monatsrenditen (Gesamtrendite aus den realen Indizes × CPI),
/// Inflation als Veränderung des CPI gegenüber dem Vorjahresmonat.
pub fn shiller_dataset(raw: &ShillerRaw) -> Result<DataSet> {
    let ok = |m: YearMonth| {
        [&raw.cpi, &raw.real_tr_price, &raw.real_tr_bonds]
            .iter()
            .all(|s| s.contains_key(&m))
    };
    let first_data = *raw.cpi.keys().next().expect("geprüft");
    let last_data = *raw.cpi.keys().next_back().expect("geprüft");
    // Erster Monat mit Vormonat und Vorjahresmonat; letzter Monat ohne Lücke bis dahin.
    let first = first_data.plus(12);
    let mut ds = DataSet {
        name: "Shiller USD".into(),
        freq: Freq::Monthly,
        first,
        ..Default::default()
    };
    let mut m = first;
    while m <= last_data && ok(m) && ok(m.plus(-1)) && raw.cpi.contains_key(&m.plus(-12)) {
        let nominal = |s: &Monthly| s[&m] * raw.cpi[&m] / (s[&m.plus(-1)] * raw.cpi[&m.plus(-1)]) - 1.0;
        ds.stocks.push(nominal(&raw.real_tr_price));
        ds.bonds.push(nominal(&raw.real_tr_bonds));
        ds.inflation.push(raw.cpi[&m] / raw.cpi[&m.plus(-12)] - 1.0);
        m = m.plus(1);
    }
    if ds.is_empty() {
        return Err(invalid("Shiller-Datei: keine durchgehenden Monatswerte"));
    }
    ds.meta = Meta {
        title: Some("Shiller: S&P Composite und 10-jährige US-Staatsanleihen (USD)".into()),
        currency: Some("USD".into()),
        source: Some("Robert J. Shiller, «Irrational Exuberance», Online-Daten (ie_data)".into()),
        info: Some(INFO_USD.into()),
        notes: vec![
            "Aktien: Gesamtrendite inkl. Dividenden; Obligationen: Gesamtrendite 10-jähriger Staatsanleihen.".into(),
            "Monatswerte aus Monatsdurchschnitten der Kurse; Inflation: US-CPI gegenüber Vorjahresmonat.".into(),
        ],
    };
    Ok(ds)
}

// Kurze Erklärungen der ausgelieferten Reihen (`# Info:` in den Dateien unter crates/engine/data/).
const INFO_USD: &str =
    "Shiller: S&P Composite und 10-jährige US-Staatsanleihen in USD, Gesamtrendite inkl. Dividenden. Teuerung: US-CPI.";
const INFO_CHF: &str = "Shiller: S&P Composite und 10-jährige US-Staatsanleihen, in CHF umgerechnet (SNB-Wechselkurs), ohne Währungsabsicherung. Teuerung: Landesindex, SARON ab 06.1999.";
const INFO_CH_BONDS: &str = "Shiller: US-Aktien (S&P Composite) in CHF, ohne Währungsabsicherung. Obligationen: 10-jährige Bundesobligationen (SNB). Teuerung: Landesindex, SARON ab 06.1999.";

// ---------- SNB ----------

/// Ein Datenwürfel der SNB (CSV-Download von data.snb.ch): Werte pro Dimensions-Kombination.
#[derive(Clone, Debug, Default)]
pub struct SnbCube {
    pub id: String,
    pub published: Option<String>,
    /// Schlüssel: Dimensionswerte mit «/» verbunden, z. B. `M0/USD1`.
    pub series: BTreeMap<String, Monthly>,
}

impl SnbCube {
    pub fn get(&self, key: &str) -> Option<&Monthly> {
        self.series.get(key)
    }
}

pub fn read_snb(path: &Path) -> Result<SnbCube> {
    let text = std::fs::read_to_string(path).map_err(|e| Error::Io(format!("{}: {e}", path.display())))?;
    parse_snb(&text).map_err(|e| invalid(format!("{}: {e}", path.display())))
}

pub fn parse_snb(text: &str) -> Result<SnbCube> {
    let mut cube = SnbCube::default();
    let mut header: Option<Vec<String>> = None;
    for line in text.lines() {
        let cells: Vec<String> = line
            .split(';')
            .map(|c| c.trim().trim_start_matches('\u{feff}').trim_matches('"').to_string())
            .collect();
        if cells.iter().all(String::is_empty) {
            continue;
        }
        let Some(h) = &header else {
            match cells[0].as_str() {
                "CubeId" => cube.id = cells.get(1).cloned().unwrap_or_default(),
                "PublishingDate" => cube.published = cells.get(1).cloned(),
                "Date" => header = Some(cells),
                _ => {}
            }
            continue;
        };
        if cells.len() != h.len() {
            return Err(invalid(format!("SNB-Daten: Zeile «{line}» passt nicht zur Kopfzeile")));
        }
        let value = cells.last().expect("nicht leer");
        if value.is_empty() {
            continue;
        }
        let m: YearMonth = cells[0]
            .parse()
            .map_err(|e: String| invalid(format!("SNB-Daten: {e}")))?;
        let v: f64 = value
            .parse()
            .map_err(|_| invalid(format!("SNB-Daten: «{value}» ist keine Zahl")))?;
        let key = cells[1..cells.len() - 1].join("/");
        cube.series.entry(key).or_default().insert(m, v);
    }
    if cube.id.is_empty() || header.is_none() {
        return Err(invalid("keine SNB-Datei (CubeId bzw. Kopfzeile fehlt)"));
    }
    Ok(cube)
}

/// Die SNB-Reihen für die Umrechnung in CHF.
#[derive(Clone, Debug, Default)]
pub struct SnbData {
    /// CHF für 1 USD, Monatsmittel.
    pub usd_chf: Monthly,
    /// Landesindex der Konsumentenpreise.
    pub cpi: Monthly,
    /// SARON in Prozent (Monatswert).
    pub saron: Monthly,
    /// Rendite 10-jähriger Obligationen der Eidgenossenschaft in Prozent (Monatswert).
    pub ch_yield10: Monthly,
    /// Publikationsdatum pro Würfel, für die Quellenangabe.
    pub published: BTreeMap<String, String>,
}

const CUBE_FX: (&str, &str) = ("devkum", "M0/USD1");
const CUBE_CPI: (&str, &str) = ("plkopr", "LD2010100");
const CUBE_SARON: (&str, &str) = ("zimoma", "SARON");
const CUBE_BONDS: (&str, &str) = ("rendoblim", "10J");

/// Eingebaute SNB-Daten (Auszug der benötigten Reihen), damit der Import ohne Download funktioniert.
pub const BUILTIN_SNB: [(&str, &str); 4] = [
    ("snb-devkum.csv", include_str!("../data/quellen/snb-devkum.csv")),
    ("snb-plkopr.csv", include_str!("../data/quellen/snb-plkopr.csv")),
    ("snb-zimoma.csv", include_str!("../data/quellen/snb-zimoma.csv")),
    ("snb-rendoblim.csv", include_str!("../data/quellen/snb-rendoblim.csv")),
];

impl SnbData {
    /// Übernimmt die benötigten Reihen aus den Würfeln; ein späterer Würfel ersetzt einen früheren.
    pub fn from_cubes(cubes: &[SnbCube]) -> Result<SnbData> {
        let mut d = SnbData::default();
        for c in cubes {
            let (target, key) = match c.id.as_str() {
                id if id == CUBE_FX.0 => (&mut d.usd_chf, CUBE_FX.1),
                id if id == CUBE_CPI.0 => (&mut d.cpi, CUBE_CPI.1),
                id if id == CUBE_SARON.0 => (&mut d.saron, CUBE_SARON.1),
                id if id == CUBE_BONDS.0 => (&mut d.ch_yield10, CUBE_BONDS.1),
                id => {
                    return Err(invalid(format!(
                        "SNB-Würfel «{id}» wird nicht verwendet (erwartet: devkum, plkopr, zimoma, rendoblim)"
                    )))
                }
            };
            let s = c
                .get(key)
                .ok_or_else(|| invalid(format!("SNB-Würfel {}: Reihe {key} fehlt", c.id)))?;
            *target = s.clone();
            match &c.published {
                Some(p) => d.published.insert(c.id.clone(), p.clone()),
                None => d.published.remove(&c.id),
            };
        }
        Ok(d)
    }

    pub fn builtin() -> SnbData {
        let cubes: Vec<SnbCube> = BUILTIN_SNB
            .iter()
            .map(|(_, t)| parse_snb(t).expect("eingebaute SNB-Daten sind gültig"))
            .collect();
        SnbData::from_cubes(&cubes).expect("eingebaute SNB-Daten sind vollständig")
    }

    /// Nur die benötigten Reihen, im SNB-Format (für `crates/engine/data/quellen`).
    pub fn extract_csv(cube: &SnbCube) -> Result<String> {
        let key = [CUBE_FX, CUBE_CPI, CUBE_SARON, CUBE_BONDS]
            .iter()
            .find(|(id, _)| *id == cube.id)
            .map(|(_, k)| *k)
            .ok_or_else(|| invalid(format!("SNB-Würfel «{}» wird nicht verwendet", cube.id)))?;
        let s = cube
            .get(key)
            .ok_or_else(|| invalid(format!("SNB-Würfel {}: Reihe {key} fehlt", cube.id)))?;
        let dims: Vec<&str> = key.split('/').collect();
        let mut out = format!("\"CubeId\";\"{}\"\n", cube.id);
        if let Some(p) = &cube.published {
            out.push_str(&format!("\"PublishingDate\";\"{p}\"\n"));
        }
        let dim_names: Vec<String> = (0..dims.len()).map(|i| format!("\"D{i}\"")).collect();
        out.push_str(&format!("\n\"Date\";{};\"Value\"\n", dim_names.join(";")));
        let dim_vals: Vec<String> = dims.iter().map(|d| format!("\"{d}\"")).collect();
        for (m, v) in s {
            out.push_str(&format!("\"{}\";{};\"{v}\"\n", m.iso(), dim_vals.join(";")));
        }
        Ok(out)
    }
}

/// Rechnet eine Monatsreihe in USD in CHF um: Renditen × Veränderung des Kurses USD/CHF (ohne
/// Währungsabsicherung), Inflation aus dem Landesindex (Vorjahresmonat), SARON in Spalte «Saron»
/// (vor dessen Einführung leer). Die Reihe beginnt im ersten Monat, für den alle Angaben vorliegen.
pub fn to_chf(usd: &DataSet, snb: &SnbData) -> Result<DataSet> {
    if usd.freq != Freq::Monthly {
        return Err(invalid("Die Umrechnung in CHF ist nur für Monatsreihen möglich."));
    }
    if usd.currency().is_some_and(|c| c != "USD") {
        return Err(invalid(format!("Die Reihe {} ist nicht in USD.", usd.name)));
    }
    let have = |m: YearMonth| {
        snb.usd_chf.contains_key(&m)
            && snb.usd_chf.contains_key(&m.plus(-1))
            && snb.cpi.contains_key(&m)
            && snb.cpi.contains_key(&m.plus(-12))
    };
    let idx: Vec<usize> = (0..usd.len()).filter(|&i| have(usd.first.plus(i as i32))).collect();
    let (Some(&a), Some(&b)) = (idx.first(), idx.last()) else {
        return Err(invalid("Kurs USD/CHF bzw. Landesindex decken die Reihe nicht ab."));
    };
    if b - a + 1 != idx.len() {
        return Err(invalid("Kurs USD/CHF bzw. Landesindex haben Lücken."));
    }
    let mut ds = DataSet {
        name: "Shiller CHF".into(),
        freq: Freq::Monthly,
        first: usd.first.plus(a as i32),
        saron: Some(vec![]),
        ..Default::default()
    };
    for i in a..=b {
        let m = usd.first.plus(i as i32);
        let fx = snb.usd_chf[&m] / snb.usd_chf[&m.plus(-1)];
        ds.stocks.push((1.0 + usd.stocks[i]) * fx - 1.0);
        ds.bonds.push((1.0 + usd.bonds[i]) * fx - 1.0);
        ds.inflation.push(snb.cpi[&m] / snb.cpi[&m.plus(-12)] - 1.0);
        let s = snb.saron.get(&m).map_or(f64::NAN, |v| v / 100.0);
        ds.saron.as_mut().expect("gesetzt").push(s);
    }
    let mut notes = vec![
        "Renditen der US-Anlagen in CHF umgerechnet (Monatsmittel USD/CHF, ohne Währungsabsicherung).".into(),
        "Inflation: Landesindex der Konsumentenpreise gegenüber Vorjahresmonat; SARON ab Einführung (06.1999), vorher gilt der fixe Wert.".into(),
    ];
    notes.extend(usd.meta.notes.iter().filter(|n| n.starts_with("Aktien")).cloned());
    let mut source = format!(
        "{}; SNB (data.snb.ch): devkum, plkopr, zimoma",
        usd.meta.source.as_deref().unwrap_or(&usd.name)
    );
    if !snb.published.is_empty() {
        let stand: Vec<String> = snb
            .published
            .iter()
            .filter(|(id, _)| [CUBE_FX.0, CUBE_CPI.0, CUBE_SARON.0].contains(&id.as_str()))
            .map(|(id, p)| format!("{id} {p}"))
            .collect();
        source.push_str(&format!(" (Stand {})", stand.join(", ")));
    }
    ds.meta = Meta {
        title: Some("Shiller: S&P Composite und 10-jährige US-Staatsanleihen, in CHF".into()),
        currency: Some("CHF".into()),
        source: Some(source),
        info: Some(INFO_CHF.into()),
        notes,
    };
    Ok(ds)
}

/// Monatsrendite einer 10-jährigen Obligation zu pari (jährlicher Coupon = Rendite des Vormonats `y0`),
/// einen Monat später zur Rendite `y1` bewertet, plus aufgelaufener Coupon. Gleiches Vorgehen wie
/// Shiller für die US-Staatsanleihen. Renditen als Dezimalzahl, auch negativ.
pub fn bond_return(y0: f64, y1: f64) -> f64 {
    let n = 10.0 - 1.0 / 12.0;
    let disc = (1.0 + y1).powf(-n);
    // Barwert der Coupons: Rentenbarwertfaktor, für Renditen nahe 0 als Grenzwert n.
    let annuity = if y1.abs() < 1e-9 { n } else { (1.0 - disc) / y1 };
    y0 * annuity + disc - 1.0 + y0 / 12.0
}

/// Ersetzt die Obligationen einer CHF-Monatsreihe durch 10-jährige Obligationen der
/// Eidgenossenschaft (Gesamtrendite aus den SNB-Renditen). Die Reihe wird auf den Zeitraum gekürzt,
/// für den beide Angaben vorliegen.
pub fn with_ch_bonds(chf: &DataSet, snb: &SnbData) -> Result<DataSet> {
    if chf.freq != Freq::Monthly || chf.currency() != Some("CHF") {
        return Err(invalid(
            "Schweizer Obligationen lassen sich nur mit einer CHF-Monatsreihe kombinieren.",
        ));
    }
    let y = &snb.ch_yield10;
    let have = |m: YearMonth| y.contains_key(&m) && y.contains_key(&m.plus(-1));
    let idx: Vec<usize> = (0..chf.len()).filter(|&i| have(chf.first.plus(i as i32))).collect();
    let (Some(&a), Some(&b)) = (idx.first(), idx.last()) else {
        return Err(invalid("Die Renditen der Eidgenossenschaft decken die Reihe nicht ab."));
    };
    if b - a + 1 != idx.len() {
        return Err(invalid("Die Renditen der Eidgenossenschaft haben Lücken."));
    }
    let mut ds = DataSet {
        name: "Shiller CHF + CH-Obligationen".into(),
        freq: Freq::Monthly,
        first: chf.first.plus(a as i32),
        saron: chf.saron.as_ref().map(|s| s[a..=b].to_vec()),
        stocks: chf.stocks[a..=b].to_vec(),
        inflation: chf.inflation[a..=b].to_vec(),
        ..Default::default()
    };
    for i in a..=b {
        let m = chf.first.plus(i as i32);
        ds.bonds.push(bond_return(y[&m.plus(-1)] / 100.0, y[&m] / 100.0));
    }
    let mut source = format!(
        "{}; SNB (data.snb.ch): rendoblim",
        chf.meta.source.as_deref().unwrap_or(&chf.name)
    );
    if let Some(p) = snb.published.get(CUBE_BONDS.0) {
        source.push_str(&format!(" (Stand {p})"));
    }
    let mut notes = vec![
        "Aktien: S&P Composite in CHF (ohne Währungsabsicherung); Obligationen: 10-jährige Obligationen der Eidgenossenschaft.".into(),
        "Obligationen: Gesamtrendite einer 10-jährigen Obligation zu pari aus den monatlichen SNB-Renditen (Coupon plus Kursänderung), ohne Kosten.".into(),
    ];
    notes.extend(chf.meta.notes.iter().filter(|n| n.starts_with("Inflation")).cloned());
    ds.meta = Meta {
        title: Some("US-Aktien in CHF und Schweizer Bundesobligationen".into()),
        currency: Some("CHF".into()),
        source: Some(source),
        info: Some(INFO_CH_BONDS.into()),
        notes,
    };
    Ok(ds)
}

#[cfg(test)]
mod tests {
    #[test]
    fn imported_series_use_the_info_of_the_shipped_files() {
        let info = |file: &str| {
            let text = crate::library::BUILTIN.iter().find(|(f, _)| *f == file).unwrap().1;
            crate::market::DataSet::parse_csv(file, text)
                .unwrap()
                .meta
                .info
                .unwrap()
        };
        assert_eq!(info("Shiller USD.csv"), INFO_USD);
        assert_eq!(info("Shiller CHF.csv"), INFO_CHF);
        assert_eq!(info(crate::library::CH_BONDS_FILE), INFO_CH_BONDS);
    }

    use super::*;

    fn f(v: f64) -> Data {
        Data::Float(v)
    }
    fn s(v: &str) -> Data {
        Data::String(v.into())
    }

    #[test]
    fn shiller_months() {
        assert_eq!(shiller_month(1871.01), Some(YearMonth::new(1871, 1)));
        assert_eq!(shiller_month(2025.1), Some(YearMonth::new(2025, 10)));
        assert_eq!(shiller_month(2025.12), Some(YearMonth::new(2025, 12)));
    }

    #[test]
    fn shiller_returns_from_real_indices() {
        let mut rows = vec![
            vec![s(""), s("Consumer"), s("Real Total"), s("Real Total")],
            vec![s("Date"), s("CPI"), s("Return Price"), s("Bond Returns")],
        ];
        // CPI steigt 1 % pro Monat; realer Aktienindex +2 %, realer Obligationenindex konstant.
        for i in 0..14 {
            let m = 2000.0 + (i % 12 + 1) as f64 / 100.0 + (i / 12) as f64;
            rows.push(vec![
                f(m),
                f(100.0 * 1.01f64.powi(i)),
                f(50.0 * 1.02f64.powi(i)),
                f(10.0),
            ]);
        }
        let ds = shiller_dataset(&parse_shiller(&rows).unwrap()).unwrap();
        assert_eq!(ds.first, YearMonth::new(2001, 1));
        assert_eq!(ds.len(), 2);
        assert!((ds.stocks[0] - (1.02 * 1.01 - 1.0)).abs() < 1e-12);
        assert!((ds.bonds[0] - 0.01).abs() < 1e-12);
        assert!((ds.inflation[0] - (1.01f64.powi(12) - 1.0)).abs() < 1e-12);
    }

    #[test]
    fn snb_cube_and_chf() {
        let fx = parse_snb("\u{feff}\"CubeId\";\"devkum\"\n\"PublishingDate\";\"2026-11-02 14:30\"\n\n\"Date\";\"D0\";\"D1\";\"Value\"\n\"2000-12\";\"M0\";\"USD1\";\"2\"\n\"2001-01\";\"M0\";\"USD1\";\"2.2\"\n\"2001-01\";\"M1\";\"USD1\";\n").unwrap();
        assert_eq!(fx.get("M0/USD1").unwrap().len(), 2);
        let cpi = parse_snb("\"CubeId\";\"plkopr\"\n\n\"Date\";\"D0\";\"Value\"\n\"2000-01\";\"LD2010100\";\"100\"\n\"2001-01\";\"LD2010100\";\"102\"\n").unwrap();
        let snb = SnbData::from_cubes(&[fx, cpi]).unwrap();
        let usd = DataSet {
            name: "u".into(),
            freq: Freq::Monthly,
            first: YearMonth::new(2001, 1),
            stocks: vec![0.1],
            bonds: vec![0.0],
            inflation: vec![0.03],
            ..Default::default()
        };
        let chf = to_chf(&usd, &snb).unwrap();
        assert!((chf.stocks[0] - (1.1 * 1.1 - 1.0)).abs() < 1e-12);
        assert!((chf.bonds[0] - 0.1).abs() < 1e-12);
        assert!((chf.inflation[0] - 0.02).abs() < 1e-12);
        assert!(chf.saron.as_ref().unwrap()[0].is_nan());
        assert_eq!(chf.currency(), Some("CHF"));
        // Hin und zurück über CSV.
        let back = DataSet::parse_csv("x", &chf.to_csv()).unwrap();
        assert_eq!(back.meta, chf.meta);
        assert!((back.stocks[0] - chf.stocks[0]).abs() < 1e-9);
        assert!(back.saron.unwrap()[0].is_nan());
    }

    #[test]
    fn builtin_snb_data() {
        let d = SnbData::builtin();
        assert!(d.usd_chf.contains_key(&YearMonth::new(1921, 1)));
        assert!(d.cpi.contains_key(&YearMonth::new(1921, 1)));
        assert!(d.saron.contains_key(&YearMonth::new(2000, 1)));
        assert!(d.ch_yield10.contains_key(&YearMonth::new(1988, 1)));
    }

    #[test]
    fn par_bond_returns() {
        // Unveränderte Rendite: Kurs bleibt bei pari, Ertrag = ein Monatscoupon.
        for y in [0.04, 0.0, -0.005] {
            assert!((bond_return(y, y) - y / 12.0).abs() < 1e-12, "{y}");
        }
        // Rendite steigt um 1 Prozentpunkt: Verlust etwa Duration (~8.5) × 1 %.
        let r = bond_return(0.02, 0.03);
        assert!(r < -0.08 && r > -0.09, "{r}");
    }

    fn shiller_rows(months: usize) -> Vec<Vec<Data>> {
        // Kopfzeile in der ersten Zeile (keine Zeilen darüber).
        let mut rows = vec![vec![
            s("Date"),
            s("CPI"),
            s("Real Total Return Price"),
            s("Real Total Bond Returns"),
        ]];
        for i in 0..months {
            let m = 2000.0 + (i % 12 + 1) as f64 / 100.0 + (i / 12) as f64;
            rows.push(vec![f(m), f(100.0 + i as f64), f(50.0), f(10.0)]);
        }
        rows
    }

    // #40: Zellen als Ganzzahl oder Text; Kopfzeile ganz oben; Werte ≤ 0 fehlen; mindestens 13 Monate.
    #[test]
    fn shiller_cells_header_and_minimum() {
        assert_eq!(cell_num(Some(&Data::Int(7))), Some(7.0));
        assert_eq!(cell_num(Some(&s(" 3.5 "))), Some(3.5));
        assert_eq!(cell_num(Some(&s("x"))), None);
        assert_eq!(cell_num(Some(&Data::Empty)), None);

        let raw = parse_shiller(&shiller_rows(13)).unwrap();
        assert_eq!(raw.cpi.len(), 13);
        assert!(parse_shiller(&shiller_rows(12)).is_err());

        let mut rows = shiller_rows(13);
        rows[2][3] = f(0.0);
        let raw = parse_shiller(&rows).unwrap();
        assert!(!raw.real_tr_bonds.contains_key(&YearMonth::new(2000, 2)));
        assert_eq!(raw.real_tr_bonds.len(), 12);
    }

    #[test]
    fn shiller_dataset_stops_at_the_first_gap() {
        let mut rows = shiller_rows(20);
        rows[17][2] = f(0.0); // Aktienindex fehlt im Mai 2001
        let ds = shiller_dataset(&parse_shiller(&rows).unwrap()).unwrap();
        assert_eq!(ds.name, "Shiller USD");
        assert_eq!(ds.first, YearMonth::new(2001, 1));
        assert_eq!(ds.len(), 4);
    }

    #[test]
    fn snb_files_need_id_and_header() {
        assert!(parse_snb("\"CubeId\";\"devkum\"\n").is_err());
        assert!(parse_snb("\"Date\";\"D0\";\"Value\"\n\"2000-01\";\"M0\";\"1\"\n").is_err());

        let dir = std::env::temp_dir().join(format!("pp-snb-unit-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("snb-devkum.csv");
        std::fs::write(&path, BUILTIN_SNB[0].1).unwrap();
        let cube = read_snb(&path).unwrap();
        assert_eq!(cube.id, "devkum");
        assert!(cube.get(CUBE_FX.1).is_some_and(|s| s.len() > 1000));
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn unknown_cube_and_csv_extract() {
        let unknown = SnbCube {
            id: "fremd".into(),
            ..Default::default()
        };
        let e = SnbData::from_cubes(std::slice::from_ref(&unknown))
            .unwrap_err()
            .to_string();
        assert!(e.contains("wird nicht verwendet"), "{e}");
        assert!(SnbData::extract_csv(&unknown).is_err());

        // Auszug und zurück: dieselbe Reihe für jeden der vier Würfel.
        for (_, text) in BUILTIN_SNB {
            let cube = parse_snb(text).unwrap();
            let back = parse_snb(&SnbData::extract_csv(&cube).unwrap()).unwrap();
            assert_eq!(back.id, cube.id);
            assert_eq!(back.published, cube.published);
            assert_eq!(back.series.len(), 1);
            let (key, series) = back.series.iter().next().unwrap();
            assert_eq!(cube.get(key), Some(series), "{}", cube.id);
        }
    }

    #[test]
    fn chf_and_ch_bond_series_names_and_checks() {
        let snb = SnbData::builtin();
        let usd = DataSet {
            name: "u".into(),
            freq: Freq::Monthly,
            first: YearMonth::new(2001, 1),
            stocks: vec![0.1, 0.0],
            bonds: vec![0.0, 0.0],
            inflation: vec![0.0, 0.0],
            ..Default::default()
        };
        let chf = to_chf(&usd, &snb).unwrap();
        assert_eq!(chf.name, "Shiller CHF");
        let ch = with_ch_bonds(&chf, &snb).unwrap();
        assert_eq!(ch.name, "Shiller CHF + CH-Obligationen");
        // Eine Monatsreihe in USD lässt sich nicht kombinieren.
        let mut in_usd = usd.clone();
        in_usd.meta.currency = Some("USD".into());
        assert!(with_ch_bonds(&in_usd, &snb).is_err());
    }
}
