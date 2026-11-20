//! #170: Texts of the native menus and file dialogs. They come from the same text tables as the user interface
//! (`ui/i18n/<language>.json`, flat keys, placeholders `{name}`), compiled into the app.

use std::collections::{BTreeMap, HashMap};
use std::sync::{Mutex, OnceLock};

/// Language used when the system language or the chosen one has no table.
pub const FALLBACK: &str = "de";
/// Choice «wie System»: the language follows the operating system.
pub const SYSTEM: &str = "system";
/// Key of the choice in the settings of the app (`settings_get`/`settings_set`), shared with the user interface.
pub const SETTING: &str = "language";

/// Available languages, each named in its own language (`ui/i18n/languages.json`).
const LANGUAGES: &str = include_str!("../ui/i18n/languages.json");
/// Text tables per language. A new language needs its file in `ui/i18n/`, an entry in `languages.json` and a row here
/// (the test `every_language_has_a_compiled_table` checks that the three agree).
const TABLES: &[(&str, &str)] = &[
    ("de", include_str!("../ui/i18n/de.json")),
    ("en", include_str!("../ui/i18n/en.json")),
];

/// Order of the languages in menus and settings.
const ORDER: [&str; 4] = ["de", "en", "fr", "it"];

fn parsed() -> &'static HashMap<&'static str, HashMap<String, String>> {
    static T: OnceLock<HashMap<&'static str, HashMap<String, String>>> = OnceLock::new();
    T.get_or_init(|| {
        TABLES
            .iter()
            .map(|(lang, text)| (*lang, serde_json::from_str(text).expect("text table is valid JSON")))
            .collect()
    })
}

/// Available languages (code, own name) in the order de, en, fr, it.
pub fn languages() -> Vec<(String, String)> {
    let names: BTreeMap<String, String> = serde_json::from_str(LANGUAGES).expect("languages.json is valid JSON");
    let mut v: Vec<(String, String)> = names.into_iter().collect();
    v.sort_by_key(|(code, _)| ORDER.iter().position(|o| o == code).unwrap_or(ORDER.len()));
    v
}

/// Language part of a locale: "fr_FR.UTF-8" → "fr", "de-CH" → "de".
pub fn language_of(locale: &str) -> String {
    locale
        .split(['-', '_', '.', '@'])
        .next()
        .unwrap_or_default()
        .to_lowercase()
}

/// Available language for a locale or a choice; German if there is no table for it.
pub fn pick(locale: Option<&str>) -> String {
    let lang = locale.map(language_of).unwrap_or_default();
    if parsed().contains_key(lang.as_str()) && languages().iter().any(|(c, _)| *c == lang) {
        lang
    } else {
        FALLBACK.to_string()
    }
}

/// Language of the operating system (sys-locale), reduced to an available language.
pub fn system_language() -> String {
    pick(sys_locale::get_locale().as_deref())
}

/// Text of the key in the language (fallback German, then the key itself).
pub fn tr(lang: &str, key: &str) -> String {
    let t = parsed();
    t.get(lang)
        .and_then(|m| m.get(key))
        .or_else(|| t.get(FALLBACK).and_then(|m| m.get(key)))
        .cloned()
        .unwrap_or_else(|| key.to_string())
}

/// Language and choice currently shown (menus, dialogs). The user interface reports them after every change.
pub struct UiLanguage {
    state: Mutex<(String, String)>,
}

impl UiLanguage {
    pub fn new(language: String, choice: String) -> Self {
        UiLanguage {
            state: Mutex::new((language, choice)),
        }
    }
    pub fn language(&self) -> String {
        self.state
            .lock()
            .map(|s| s.0.clone())
            .unwrap_or_else(|_| FALLBACK.to_string())
    }
    pub fn choice(&self) -> String {
        self.state
            .lock()
            .map(|s| s.1.clone())
            .unwrap_or_else(|_| SYSTEM.to_string())
    }
    pub fn set(&self, language: &str, choice: &str) {
        if let Ok(mut s) = self.state.lock() {
            *s = (pick(Some(language)), choice.to_string());
        }
    }
}

/// Language at the start of the app for the menus, before the user interface reports its language: the saved choice
/// if it is a language, otherwise the language of the operating system.
pub fn startup(choice: Option<&str>, system: impl FnOnce() -> String) -> UiLanguage {
    match choice {
        Some(c) if c != SYSTEM => UiLanguage::new(pick(Some(c)), c.to_string()),
        _ => UiLanguage::new(system(), SYSTEM.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn language_of_ignores_the_region() {
        assert_eq!(language_of("fr_FR"), "fr");
        assert_eq!(language_of("en-US"), "en");
        assert_eq!(language_of("de_CH.UTF-8"), "de");
        assert_eq!(language_of("IT"), "it");
        assert_eq!(language_of(""), "");
    }

    #[test]
    fn issue_170_ak2_unknown_or_missing_languages_fall_back_to_german() {
        assert_eq!(pick(Some("de-CH")), "de");
        assert_eq!(pick(Some("ja_JP")), "de");
        assert_eq!(pick(None), "de");
        // a language of the list without a table yet is German as well
        for (code, _) in languages() {
            assert_eq!(pick(Some(&code)), code);
        }
        for code in ORDER {
            let expected = if parsed().contains_key(code) { code } else { FALLBACK };
            assert_eq!(pick(Some(&format!("{code}_FR"))), expected);
        }
        assert!(parsed().contains_key(system_language().as_str()));
    }

    #[test]
    fn every_language_has_a_compiled_table() {
        let listed: Vec<String> = languages().into_iter().map(|(c, _)| c).collect();
        let mut compiled: Vec<&str> = TABLES.iter().map(|(c, _)| *c).collect();
        compiled.sort_by_key(|c| ORDER.iter().position(|o| o == c));
        assert_eq!(listed, compiled);
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("ui/i18n");
        let mut files: Vec<String> = std::fs::read_dir(dir)
            .unwrap()
            .filter_map(|e| e.ok()?.file_name().into_string().ok())
            .filter_map(|n| n.strip_suffix(".json").map(str::to_string))
            .filter(|n| n != "languages")
            .collect();
        files.sort_by_key(|c| ORDER.iter().position(|o| o == c));
        assert_eq!(files, listed);
        assert_eq!(languages()[0], ("de".to_string(), "Deutsch".to_string()));
    }

    /// #170/AK4: every table has exactly the keys of the German table, none empty, with the same placeholders. The
    /// test reads every file in `ui/i18n/` (also a single table, and new tables without a change here).
    #[test]
    fn issue_170_ak4_tables_are_complete() {
        let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("ui/i18n");
        let read = |name: &str| -> BTreeMap<String, String> {
            let text = std::fs::read_to_string(dir.join(name)).unwrap();
            serde_json::from_str(&text).unwrap_or_else(|e| panic!("{name}: {e}"))
        };
        let placeholders = |s: &str| -> Vec<String> {
            let mut v: Vec<String> = s
                .split('{')
                .skip(1)
                .filter_map(|p| p.split_once('}').map(|(n, _)| n.to_string()))
                .filter(|n| !n.is_empty() && n.chars().all(|c| c.is_alphanumeric() || c == '_' || c == ':'))
                .collect();
            v.sort();
            v
        };
        let de = read("de.json");
        assert!(de.len() > 1000, "{}", de.len());
        let mut tables = 0;
        for entry in std::fs::read_dir(&dir).unwrap() {
            let name = entry.unwrap().file_name().into_string().unwrap();
            if !name.ends_with(".json") || name == "languages.json" {
                continue;
            }
            tables += 1;
            let t = read(&name);
            let missing: Vec<&String> = de.keys().filter(|k| !t.contains_key(*k)).collect();
            let extra: Vec<&String> = t.keys().filter(|k| !de.contains_key(*k)).collect();
            // #171: `engine.empty` is empty in every language (default of an engine field without a text).
            let empty: Vec<&String> = t
                .iter()
                .filter(|(k, v)| v.trim().is_empty() && !de[*k].is_empty())
                .map(|(k, _)| k)
                .collect();
            let params: Vec<&String> = t
                .iter()
                .filter(|(k, v)| de.get(*k).is_some_and(|d| placeholders(d) != placeholders(v)))
                .map(|(k, _)| k)
                .collect();
            assert!(missing.is_empty(), "{name}: fehlende Schlüssel {missing:?}");
            assert!(extra.is_empty(), "{name}: überzählige Schlüssel {extra:?}");
            assert!(empty.is_empty(), "{name}: leere Texte {empty:?}");
            assert!(params.is_empty(), "{name}: andere Platzhalter {params:?}");
        }
        assert_eq!(tables, languages().len());
    }

    /// #171/AK3, #176/AK1: Every message code of the engine has its template `engine.<code>` in the English table, and
    /// the template renders the example of the engine to exactly the (English) text of the engine. Keys `engine.*`
    /// without a code are not allowed. (The test of the user interface `e2e/unit/engine-messages.test.js` checks the same with
    /// `engineText` of util.js.)
    #[test]
    fn issue_171_ak3_engine_codes_render_the_engine_text() {
        use engine::message::{example, render, Code};
        let en = &parsed()["en"];
        let mut wrong = vec![];
        for &code in Code::ALL {
            let key = format!("engine.{}", code.as_str());
            let m = example(code);
            match en.get(&key) {
                Some(t) if render(t, m.args()) == m.text() => {}
                other => wrong.push(format!("{key}: {other:?}")),
            }
        }
        assert!(wrong.is_empty(), "{wrong:#?}");
        let stale: Vec<&String> = en
            .keys()
            .filter(|k| k.strip_prefix("engine.").is_some_and(|c| Code::parse(c).is_none()))
            .collect();
        assert!(stale.is_empty(), "{stale:?}");
    }

    /// #177/AK1: The German texts of the bundled series in older files (recognized by the engine) are the texts of
    /// the table `de`, so the German user interface shows them as before.
    #[test]
    fn issue_177_ak1_stored_german_series_texts_are_the_table_de() {
        let de = &parsed()["de"];
        for &(code, text) in engine::series_text::STORED_DE {
            assert_eq!(
                de.get(&format!("engine.{}", code.as_str())).map(String::as_str),
                Some(text)
            );
        }
    }

    #[test]
    fn tr_falls_back_to_german_and_to_the_key() {
        assert_eq!(tr("de", "menu.file"), "Datei");
        assert_eq!(tr("xx", "menu.file"), "Datei");
        assert_eq!(tr("de", "no.such.key"), "no.such.key");
        assert_eq!(tr("de", "menu.view.language"), "Sprache");
    }

    #[test]
    fn startup_language_uses_the_saved_choice_or_the_system() {
        let l = startup(Some("de"), || "xx".into());
        assert_eq!((l.language(), l.choice()), ("de".to_string(), "de".to_string()));
        let l = startup(Some(SYSTEM), || "de".into());
        assert_eq!((l.language(), l.choice()), ("de".to_string(), SYSTEM.to_string()));
        let l = startup(None, || "de".into());
        assert_eq!(l.choice(), SYSTEM);
        l.set("fr-CH", "fr");
        assert_eq!(l.language(), pick(Some("fr")));
        assert_eq!(l.choice(), "fr");
    }
}
