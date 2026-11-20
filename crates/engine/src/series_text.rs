//! #177: Texts of the bundled time series (title, source, info, notes in the comment lines of the CSV files) as
//! messages. The engine writes the English template of the code; files written by older versions (data library) hold
//! the German texts below. Both are recognized when a file is read, so that the user interface translates them by the
//! code (`engine.<code>` in the text tables); any other text becomes `plain_text` and is shown as it is.

use crate::message::{Arg, Code, Message, Params};

/// Codes of the texts that can appear in the comment lines of a bundled series, with the German text of older files
/// (the same text as `engine.<code>` in the table `de`).
pub const STORED_DE: &[(Code, &str)] = &[
    (Code::series_title_shiller_usd, "Shiller: S&P Composite und 10-jährige US-Staatsanleihen (USD)"),
    (Code::series_title_shiller_chf, "Shiller: S&P Composite und 10-jährige US-Staatsanleihen, in CHF"),
    (Code::series_title_ch_bonds, "US-Aktien in CHF und Schweizer Bundesobligationen"),
    (
        Code::series_title_world,
        "Welt (JST): Aktien nach BIP gewichtet, Obligationen der Eidgenossenschaft, in CHF",
    ),
    (
        Code::series_info_shiller_usd,
        "Shiller: S&P Composite und 10-jährige US-Staatsanleihen in USD, Gesamtrendite inkl. Dividenden. Teuerung: US-CPI.",
    ),
    (
        Code::series_info_shiller_chf,
        "Shiller: S&P Composite und 10-jährige US-Staatsanleihen, in CHF umgerechnet (SNB-Wechselkurs), ohne Währungsabsicherung. Teuerung: Landesindex, SARON ab 06.1999.",
    ),
    (
        Code::series_info_ch_bonds,
        "Shiller: US-Aktien (S&P Composite) in CHF, ohne Währungsabsicherung. Obligationen: 10-jährige Bundesobligationen (SNB). Teuerung: Landesindex, SARON ab 06.1999.",
    ),
    (
        Code::series_info_world,
        "Welt-Aktienindex aus den JST-Daten (CH, USA, EU, Japan, Australien), nach BIP gewichtet, in CHF ohne Währungsabsicherung. Obligationen: Eidgenossenschaft 10 Jahre. Teuerung: CH.",
    ),
    (
        Code::series_note_total_return,
        "Aktien: Gesamtrendite inkl. Dividenden; Obligationen: Gesamtrendite 10-jähriger Staatsanleihen.",
    ),
    (
        Code::series_note_usd_monthly,
        "Monatswerte aus Monatsdurchschnitten der Kurse; Inflation: US-CPI gegenüber Vorjahresmonat.",
    ),
    (
        Code::series_note_chf_conversion,
        "Renditen der US-Anlagen in CHF umgerechnet (Monatsmittel USD/CHF, ohne Währungsabsicherung).",
    ),
    (
        Code::series_note_chf_inflation,
        "Inflation: Landesindex der Konsumentenpreise gegenüber Vorjahresmonat; SARON ab Einführung (06.1999), vorher gilt der fixe Wert.",
    ),
    (
        Code::series_note_ch_bonds_stocks,
        "Aktien: S&P Composite in CHF (ohne Währungsabsicherung); Obligationen: 10-jährige Obligationen der Eidgenossenschaft.",
    ),
    (
        Code::series_note_ch_bonds_return,
        "Obligationen: Gesamtrendite einer 10-jährigen Obligation zu pari aus den monatlichen SNB-Renditen (Coupon plus Kursänderung), ohne Kosten.",
    ),
    (
        Code::series_note_world_stocks,
        "Aktien: Gesamtrendite der Länder in CHF, gewichtet mit dem realen BIP des Vorjahres (Maddison); Deutschland 1923 ausgeschlossen (Hyperinflation), Deutschland 1945–1946 und Japan 1946–1947 ohne Werte.",
    ),
    (
        Code::series_note_world_bonds,
        "Obligationen: JST bis {year}, danach aus den monatlichen SNB-Renditen (10 Jahre); 1915 Coupon statt Rendite (Börse geschlossen).",
    ),
    (
        Code::series_note_world_license,
        "Lizenz: CC BY-NC-SA 4.0 (nicht kommerziell, Weitergabe unter gleicher Lizenz).",
    ),
    (
        Code::series_source_shiller,
        "Robert J. Shiller, «Irrational Exuberance», Online-Daten (ie_data)",
    ),
    (
        Code::series_source_jst,
        "Jordà-Schularick-Taylor Macrohistory Database, Release 6 (macrohistory.net/database, abgerufen 13.11.2026), Lizenz CC BY-NC-SA 4.0. Zitate: Jordà, Schularick, Taylor (2017), «Macrofinancial History and the New Business Cycle Facts», NBER Macroeconomics Annual 2016; Jordà, Knoll, Kuvshinov, Schularick, Taylor (2019), «The Rate of Return on Everything, 1870–2015», Quarterly Journal of Economics",
    ),
    (Code::series_source_as_of, "{source} (Stand {dates})"),
    (Code::series_source_snb, "{source}; SNB (data.snb.ch): {cubes}"),
    (Code::series_source_list, "{source}; {other}"),
];

/// Parameters that hold a source themselves (recognized recursively). In a list of sources both must be known
/// texts; before «SNB …» or «(as of …)» an own source may also be a plain text.
const NESTED: [&str; 2] = ["source", "other"];

/// The text as a message: a text of a bundled series (English or German) with its code, otherwise `plain_text`.
pub fn recognize(text: &str) -> Message {
    known(text).unwrap_or_else(|| msg!(plain_text, text = text))
}

fn known(text: &str) -> Option<Message> {
    STORED_DE.iter().find_map(|&(code, de)| {
        [code.template(), de].into_iter().find_map(|t| {
            let mut params = vec![];
            let plain = code != Code::series_source_list;
            matches(&parts(t), text, plain, &mut params).then(|| Message::new(code, params))
        })
    })
}

enum Part<'a> {
    Text(&'a str),
    Param(&'static str),
}

/// The template in literal parts and placeholders (names of the placeholders are those of [`NESTED`] or static ones).
fn parts(template: &str) -> Vec<Part<'_>> {
    let mut out = vec![];
    let mut rest = template;
    while let Some(i) = rest.find('{') {
        let Some(j) = rest[i..].find('}') else { break };
        if i > 0 {
            out.push(Part::Text(&rest[..i]));
        }
        out.push(Part::Param(static_name(&rest[i + 1..i + j])));
        rest = &rest[i + j + 1..];
    }
    if !rest.is_empty() {
        out.push(Part::Text(rest));
    }
    out
}

fn static_name(name: &str) -> &'static str {
    ["source", "other", "cubes", "dates", "year"]
        .into_iter()
        .find(|n| *n == name)
        .unwrap_or("value")
}

/// Whether `text` matches the parts; the values of the placeholders go to `params`. A placeholder takes the longest
/// possible text: a nested source must be a known text (or, with `plain`, any text), any other value must not contain
/// «;» (separator of sources).
fn matches(parts: &[Part<'_>], text: &str, plain: bool, params: &mut Params) -> bool {
    match parts.split_first() {
        None => text.is_empty(),
        Some((Part::Text(t), rest)) => text.strip_prefix(t).is_some_and(|r| matches(rest, r, plain, params)),
        Some((Part::Param(name), rest)) => {
            let ends: Vec<usize> = match rest.first() {
                None => vec![text.len()],
                Some(Part::Text(t)) => text.rmatch_indices(t).map(|(i, _)| i).collect(),
                Some(Part::Param(_)) => return false,
            };
            for end in ends.into_iter().filter(|&e| e > 0) {
                let value = &text[..end];
                let arg = if NESTED.contains(name) {
                    match known(value) {
                        Some(m) => Arg::Msg(Box::new(m)),
                        None if plain => Arg::Msg(Box::new(msg!(plain_text, text = value))),
                        None => continue,
                    }
                } else if value.contains(';') {
                    continue;
                } else {
                    match value.parse::<i64>() {
                        Ok(n) if n.to_string() == value => Arg::Int(n),
                        _ => Arg::Text(value.to_string()),
                    }
                };
                params.push((*name, arg));
                if matches(rest, &text[end..], plain, params) {
                    return true;
                }
                params.pop();
            }
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn issue_177_ak1_german_and_english_texts_get_their_code() {
        for &(code, de) in STORED_DE {
            if de.contains('{') {
                continue;
            }
            assert_eq!(recognize(de).code(), code.as_str(), "{de}");
            assert_eq!(recognize(code.template()).code(), code.as_str());
        }
        let m = recognize("Obligationen: JST bis 1988, danach aus den monatlichen SNB-Renditen (10 Jahre); 1915 Coupon statt Rendite (Börse geschlossen).");
        assert_eq!(m.code(), "series_note_world_bonds");
        assert!(m.text().starts_with("Bonds: JST until 1988, then"), "{}", m.text());
        assert_eq!(recognize("Eigene Reihe").code(), "plain_text");
        assert_eq!(recognize("Eigene Reihe").text(), "Eigene Reihe");
    }

    #[test]
    fn issue_177_ak1_nested_sources_are_recognized() {
        let de = "Robert J. Shiller, «Irrational Exuberance», Online-Daten (ie_data); SNB (data.snb.ch): devkum, plkopr, zimoma (Stand devkum 2026-11-02 14:30, plkopr 2026-10-21 09:00); SNB (data.snb.ch): rendoblim (Stand 2025-09-01 14:29)";
        let m = recognize(de);
        assert_eq!(m.code(), "series_source_as_of");
        assert_eq!(
            m.text(),
            "Robert J. Shiller, “Irrational Exuberance”, online data (ie_data); SNB (data.snb.ch): devkum, plkopr, zimoma (as of devkum 2026-11-02 14:30, plkopr 2026-10-21 09:00); SNB (data.snb.ch): rendoblim (as of 2025-09-01 14:29)"
        );
        assert_eq!(recognize(m.text()), m);
        // A list of sources (investment strategy): JST, then JST with SNB bonds.
        let jst = msg!(series_source_jst);
        let list = msg!(
            series_source_list,
            source = jst.clone(),
            other = msg!(series_source_snb, source = jst, cubes = "rendoblim")
        );
        let back = recognize(list.text());
        assert_eq!((back.code() != "plain_text", back.text()), (true, list.text()));
        // An own source before the SNB part stays as it is; a list needs known sources.
        let own = recognize("Eigene Quelle; SNB (data.snb.ch): x");
        assert_eq!(own.code(), "series_source_snb");
        assert_eq!(own.text(), "Eigene Quelle; SNB (data.snb.ch): x");
        assert_eq!(recognize("Eigene Quelle; noch eine").code(), "plain_text");
    }
}
