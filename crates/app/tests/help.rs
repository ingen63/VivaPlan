//! #173: The help per language (`ui/hilfe/<language>/`).
//!
//! - AK1: every translated file has the same chapter, topics (ids in the same order), subheadings, cross references
//!   and keyword lines as the German file; languages in `COMPLETE` have every German file.
//! - AK3: the screen labels quoted in the help of a language (quote marks in `LABEL_QUOTES`) are texts of the text
//!   table of that language. Other quotes in the help (examples, inputs, questions) use other quote marks.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

const GERMAN: &str = "de";
/// Languages whose help has every German file (otherwise the help falls back to German per file).
const COMPLETE: &[&str] = &[];
/// Quote marks of the screen labels per language.
const LABEL_QUOTES: &[(&str, char, char)] = &[("en", '“', '”')];
/// Markers of the keyword line (first line of a topic; the same list as `KEYWORDS` in help.js).
const KEYWORD_MARKERS: &[&str] = &["Stichworte:", "Keywords:", "Mots-clés:", "Parole chiave:"];

fn help_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("ui/hilfe")
}

/// Markdown files of a language folder, by name.
fn files(language: &str) -> BTreeMap<String, String> {
    let dir = help_dir().join(language);
    let mut out = BTreeMap::new();
    for entry in std::fs::read_dir(&dir).unwrap_or_else(|e| panic!("{}: {e}", dir.display())) {
        let path = entry.unwrap().path();
        if path.extension().is_some_and(|x| x == "md") {
            let name = path.file_name().unwrap().to_string_lossy().into_owned();
            out.insert(name, std::fs::read_to_string(&path).unwrap());
        }
    }
    out
}

/// Language folders besides German.
fn other_languages() -> Vec<String> {
    let mut v: Vec<String> = std::fs::read_dir(help_dir())
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.is_dir())
        .map(|p| p.file_name().unwrap().to_string_lossy().into_owned())
        .filter(|l| l != GERMAN)
        .collect();
    v.sort();
    v
}

/// Structure of a topic that has to be the same in every language.
#[derive(Debug, PartialEq)]
struct Topic {
    id: String,
    subheadings: usize,
    links: BTreeSet<String>,
    keywords: bool,
}

/// Chapter heading present and topics («## @id Title») with their structure; code blocks are skipped.
fn structure(text: &str) -> (bool, Vec<Topic>) {
    let mut chapter = false;
    let mut topics: Vec<Topic> = Vec::new();
    let mut code = false;
    let mut first_line = false;
    for line in text.lines() {
        if line.starts_with("```") {
            code = !code;
            continue;
        }
        if code {
            continue;
        }
        if line.starts_with("# ") {
            chapter = true;
            continue;
        }
        if let Some(rest) = line.strip_prefix("## @") {
            let id = rest.split_whitespace().next().unwrap_or("").to_string();
            topics.push(Topic {
                id,
                subheadings: 0,
                links: BTreeSet::new(),
                keywords: false,
            });
            first_line = true;
            continue;
        }
        let Some(t) = topics.last_mut() else { continue };
        if first_line && KEYWORD_MARKERS.iter().any(|m| line.starts_with(m)) {
            t.keywords = true;
        }
        first_line = false;
        if line.starts_with("### ") || line.starts_with("#### ") {
            t.subheadings += 1;
        }
        for part in line.split("](#").skip(1) {
            if let Some((target, _)) = part.split_once(')') {
                t.links.insert(target.to_string());
            }
        }
    }
    (chapter, topics)
}

#[test]
fn issue_173_ak1_every_language_has_the_files_topics_and_anchors_of_the_german_help() {
    let german = files(GERMAN);
    assert!(german.len() >= 7, "{german:?}");
    // No help files outside the language folders
    let loose: Vec<String> = std::fs::read_dir(help_dir())
        .unwrap()
        .map(|e| e.unwrap().path())
        .filter(|p| p.is_file())
        .map(|p| p.display().to_string())
        .collect();
    assert_eq!(loose, Vec::<String>::new());
    let languages = other_languages();
    for l in COMPLETE {
        assert!(languages.iter().any(|x| x == l), "folder hilfe/{l} missing");
    }
    let mut problems = Vec::new();
    for l in &languages {
        let own = files(l);
        for (name, text) in &own {
            let Some(de) = german.get(name) else {
                problems.push(format!("{l}/{name}: no German file of this name"));
                continue;
            };
            let (de_chapter, de_topics) = structure(de);
            let (chapter, topics) = structure(text);
            assert!(de_chapter && !de_topics.is_empty(), "de/{name}");
            if !chapter {
                problems.push(format!("{l}/{name}: chapter heading missing"));
            }
            let ids = |v: &[Topic]| v.iter().map(|t| t.id.clone()).collect::<Vec<_>>();
            if ids(&topics) != ids(&de_topics) {
                problems.push(format!(
                    "{l}/{name}: topics {:?} instead of {:?}",
                    ids(&topics),
                    ids(&de_topics)
                ));
                continue;
            }
            for (t, d) in topics.iter().zip(&de_topics) {
                if t != d {
                    problems.push(format!("{l}/{name}: topic {}: {t:?} instead of {d:?}", d.id));
                }
            }
        }
        if COMPLETE.contains(&l.as_str()) {
            for name in german.keys().filter(|n| !own.contains_key(*n)) {
                problems.push(format!("{l}/{name} missing"));
            }
        }
    }
    assert_eq!(problems, Vec::<String>::new());
}

/// Text of a table entry as shown on the screen (without HTML tags and entities, spaces collapsed).
fn plain(s: &str) -> String {
    let mut out = String::new();
    let mut tag = false;
    for c in s.chars() {
        match c {
            '<' => tag = true,
            '>' if tag => tag = false,
            _ if !tag => out.push(c),
            _ => {}
        }
    }
    let out = out
        .replace("&nbsp;", " ")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&amp;", "&");
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Does the label match the table text? Placeholders `{…}` stand for any text; a trailing «:» or «…» of the table text
/// may be left out; a label ending in «…» quotes the beginning of a text.
fn matches(label: &str, text: &str) -> bool {
    let text = text.trim();
    if let Some(start) = label.strip_suffix('…') {
        let start = start.trim_end();
        return !start.is_empty() && glob(start, text, true);
    }
    [
        text,
        text.trim_end_matches(':').trim_end(),
        text.trim_end_matches('…').trim_end(),
    ]
    .iter()
    .any(|t| glob(label, t, false))
}

/// `label` equals `pattern` (or its beginning, with `prefix`), where every `{…}` in `pattern` matches any text.
fn glob(label: &str, pattern: &str, prefix: bool) -> bool {
    let mut parts: Vec<&str> = Vec::new();
    let mut rest = pattern;
    while let Some(i) = rest.find('{') {
        let Some(j) = rest[i..].find('}') else { break };
        parts.push(&rest[..i]);
        rest = &rest[i + j + 1..];
    }
    parts.push(rest);
    if parts.len() == 1 {
        return if prefix {
            pattern.starts_with(label)
        } else {
            pattern == label
        };
    }
    // with placeholders: the literal parts in order
    let mut pos = 0;
    for (k, part) in parts.iter().enumerate() {
        let last = k == parts.len() - 1;
        if k == 0 {
            if !label.starts_with(part) {
                return prefix && part.starts_with(label);
            }
            pos = part.len();
        } else if last && !prefix {
            return label.len() >= pos + part.len() && label.ends_with(part);
        } else {
            match label[pos..].find(part) {
                Some(i) => pos += i + part.len(),
                None => return prefix,
            }
        }
    }
    true
}

#[test]
fn issue_173_ak3_quoted_screen_labels_are_in_the_text_table() {
    for &(language, open, close) in LABEL_QUOTES {
        if !help_dir().join(language).is_dir() {
            continue;
        }
        let path = Path::new(env!("CARGO_MANIFEST_DIR")).join(format!("ui/i18n/{language}.json"));
        let table: BTreeMap<String, String> = serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();
        let texts: Vec<String> = table.values().map(|v| plain(v)).filter(|v| !v.is_empty()).collect();
        let mut quoted = 0;
        let mut missing = Vec::new();
        for (name, text) in files(language) {
            for piece in text.split(open).skip(1) {
                let Some((label, _)) = piece.split_once(close) else {
                    missing.push(format!(
                        "{language}/{name}: quote not closed: {}",
                        piece.chars().take(40).collect::<String>()
                    ));
                    continue;
                };
                quoted += 1;
                // Menu paths «File → Settings…»: every part is a label.
                for part in label.split(" → ") {
                    let part = part.trim();
                    if !texts.iter().any(|t| matches(part, t)) {
                        missing.push(format!("{language}/{name}: {open}{part}{close}"));
                    }
                }
            }
        }
        if COMPLETE.contains(&language) {
            assert!(quoted >= 100, "{language}: only {quoted} quoted labels");
        }
        assert_eq!(missing, Vec::<String>::new());
    }
}

#[test]
fn issue_173_ak3_label_matching() {
    assert!(matches("Start simulation", "Start simulation"));
    assert!(matches("Settings", "Settings…"));
    assert!(matches("Stock allocation", "Stock allocation:"));
    assert!(matches("Bucket 1: Cash", "Bucket {n}: Cash"));
    assert!(matches("since 2000", "since {year}"));
    assert!(matches(
        "Withdrawal strategy applied after …",
        "Withdrawal strategy applied after {x} years"
    ));
    assert!(!matches("Start", "Start simulation"));
    assert!(!matches("Run simulation", "Start simulation"));
    assert!(!matches("Bucket 1: Bonds", "Bucket {n}: Cash"));
    assert_eq!(plain("<b>Total</b>&nbsp;spending"), "Total spending");
}
