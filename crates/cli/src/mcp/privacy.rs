//! #210/AK5: Data economy in the answers of the MCP server: names of persons and freely entered labels become neutral
//! ones («Person 1», «Property 1»), unless the server runs with `--names`; answers name only the file name, never the
//! folder (it contains the user name). See `docs/VivaPlan in Claude (MCP).md`, section 6.

use std::path::{Path, PathBuf};

use serde_json::Value;

/// Keys of the plan whose text the user enters freely.
const FREE_TEXT_KEYS: &[&str] = &["name", "label"];

/// Neutral word for an entry of a list of the plan, by the name of the list.
fn entry_word(list: &str) -> &'static str {
    match list {
        "persons" => "Person",
        "properties" => "Property",
        "tranches" => "Tranche",
        "milestones" => "Milestone",
        "extras" => "Expense",
        "pillar3a" => "Account",
        _ => "Entry",
    }
}

/// Neutral label of the free text in the object at `segments` (path from the root of the plan).
fn neutral_label(segments: &[String]) -> String {
    match segments {
        [.., list, index] if index.parse::<usize>().is_ok() => {
            let n = index.parse::<usize>().unwrap_or(0) + 1;
            format!("{} {n}", entry_word(list))
        }
        [.., last] if last == "project" => "Plan".into(),
        [.., last] if last == "partner" => "Person 2".into(),
        _ => "Entry".into(),
    }
}

/// Every free text of the plan: path of its object, original and neutral label.
fn free_texts(v: &Value, segments: &mut Vec<String>, out: &mut Vec<(Vec<String>, String, String)>) {
    match v {
        Value::Object(o) => {
            for (k, x) in o {
                if let (true, Some(s)) = (FREE_TEXT_KEYS.contains(&k.as_str()), x.as_str()) {
                    if !s.trim().is_empty() {
                        out.push((segments.clone(), s.to_string(), neutral_label(segments)));
                    }
                    continue;
                }
                segments.push(k.clone());
                free_texts(x, segments, out);
                segments.pop();
            }
        }
        Value::Array(a) => {
            for (i, x) in a.iter().enumerate() {
                segments.push(i.to_string());
                free_texts(x, segments, out);
                segments.pop();
            }
        }
        _ => {}
    }
}

/// The plan with neutral labels in place of names and free texts (structurally, each at its place).
pub(crate) fn neutral_plan(plan: &Value) -> Value {
    let mut out = plan.clone();
    let mut found = Vec::new();
    free_texts(plan, &mut Vec::new(), &mut found);
    for (segments, _, neutral) in found {
        let mut target = &mut out;
        for s in &segments {
            target = match target {
                Value::Object(o) => o.get_mut(s.as_str()).expect("path from the same plan"),
                Value::Array(a) => &mut a[s.parse::<usize>().expect("index")],
                _ => unreachable!("path from the same plan"),
            };
        }
        if let Some(o) = target.as_object_mut() {
            for key in FREE_TEXT_KEYS {
                if o.get(*key)
                    .and_then(Value::as_str)
                    .is_some_and(|s| !s.trim().is_empty())
                {
                    o.insert((*key).into(), Value::String(neutral.clone()));
                }
            }
        }
    }
    out
}

/// Whether `s` is an absolute path on any of the supported systems.
fn is_absolute(s: &str) -> bool {
    let b = s.as_bytes();
    s.starts_with('/')
        || s.starts_with("\\\\")
        || (b.len() > 2 && b[0].is_ascii_alphabetic() && b[1] == b':' && (b[2] == b'\\' || b[2] == b'/'))
}

/// File name of an absolute path (both separators).
fn file_name(s: &str) -> &str {
    s.rsplit(['/', '\\']).next().unwrap_or(s)
}

/// Replaces `needle` in `hay` where it stands as a whole word (no letter or digit directly before or after).
fn replace_word(hay: &str, needle: &str, with: &str) -> String {
    let mut out = String::with_capacity(hay.len());
    let mut rest = hay;
    while let Some(i) = rest.find(needle) {
        let before = rest[..i].chars().next_back();
        let after = rest[i + needle.len()..].chars().next();
        let boundary = |c: Option<char>| c.is_none_or(|c| !c.is_alphanumeric());
        out.push_str(&rest[..i]);
        if boundary(before) && boundary(after) {
            out.push_str(with);
        } else {
            out.push_str(needle);
        }
        rest = &rest[i + needle.len()..];
    }
    out.push_str(rest);
    out
}

/// Replaces names and paths in every text of an answer.
#[derive(Default)]
pub(crate) struct Redactor {
    /// Name or word of a name → neutral label; longest first.
    names: Vec<(String, String)>,
    /// Path or folder → what remains of it; longest first.
    paths: Vec<(String, String)>,
}

impl Redactor {
    /// Without names (`--names`) only the paths are replaced.
    pub(crate) fn new(home: Option<PathBuf>) -> Self {
        let mut r = Redactor::default();
        if let Some(h) = home.map(|h| h.display().to_string()).filter(|h| h.len() > 1) {
            r.paths.push((h, "~".into()));
        }
        r
    }

    /// Home folder of the user as the operating system gives it.
    pub(crate) fn home() -> Option<PathBuf> {
        std::env::var_os("HOME")
            .or_else(|| std::env::var_os("USERPROFILE"))
            .map(PathBuf::from)
    }

    /// Names and free texts of a plan; of persons also each word of at least three letters (first or last name alone).
    pub(crate) fn add_plan(&mut self, plan: &Value) {
        let mut found = Vec::new();
        free_texts(plan, &mut Vec::new(), &mut found);
        for (segments, original, neutral) in found {
            let person = segments.iter().any(|s| s == "persons" || s == "partner");
            if original != neutral {
                self.names.push((original.trim().to_string(), neutral.clone()));
            }
            if person {
                for word in original.split_whitespace().filter(|w| w.chars().count() >= 3) {
                    if word != neutral && !neutral.split(' ').any(|n| n == word) {
                        self.names.push((word.to_string(), neutral.clone()));
                    }
                }
            }
        }
        self.names.sort_by_key(|(n, _)| std::cmp::Reverse(n.len()));
        self.names.dedup_by(|a, b| a.0 == b.0);
    }

    /// The file of a plan: the whole path becomes the file name, its folder disappears.
    pub(crate) fn add_file(&mut self, path: &Path) {
        let full = path.display().to_string();
        let name = path
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_default();
        self.paths.push((full, name));
        if let Some(dir) = path.parent() {
            self.add_dir(dir);
        }
    }

    /// A folder (of a plan, of the data library): paths in it keep only what follows the folder.
    pub(crate) fn add_dir(&mut self, dir: &Path) {
        let dir = dir.display().to_string();
        if dir.len() > 1 {
            self.paths
                .push((format!("{dir}{}", std::path::MAIN_SEPARATOR), String::new()));
            self.paths.push((dir, ".".into()));
        }
        self.paths.sort_by_key(|(p, _)| std::cmp::Reverse(p.len()));
        self.paths.dedup_by(|a, b| a.0 == b.0);
    }

    /// A text with neutral names and without folders.
    pub(crate) fn text(&self, s: &str) -> String {
        let mut out = s.to_string();
        for (p, with) in &self.paths {
            if out.contains(p.as_str()) {
                out = out.replace(p.as_str(), with);
            }
        }
        for (n, with) in &self.names {
            if out.contains(n.as_str()) {
                out = replace_word(&out, n, with);
            }
        }
        out
    }

    /// Every text in a value; a text that is an absolute path becomes the file name.
    pub(crate) fn value(&self, v: &mut Value) {
        match v {
            Value::String(s) => {
                *s = if is_absolute(s) {
                    file_name(s).to_string()
                } else {
                    self.text(s)
                };
            }
            Value::Array(a) => a.iter_mut().for_each(|x| self.value(x)),
            Value::Object(o) => o.values_mut().for_each(|x| self.value(x)),
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn plan() -> Value {
        json!({
            "project": { "name": "Plan Hans Muster" },
            "household": { "persons": [ { "id": "A", "name": "Hans Muster" }, { "id": "B", "name": "Eva" } ] },
            "partner": { "name": "Eva Beispiel" },
            "flows": { "extras": [ { "label": "Boot", "date": "2030-01" } ] },
            "milestones": [ { "id": "start", "name": "" }, { "id": "x", "name": "Umzug Tessin" } ],
            "realestate": { "properties": [ { "name": "Chalet", "mortgage": { "tranches": [ { "name": "Festhypothek UBS" } ] } } ] }
        })
    }

    /// #210/AK5: Each free text is replaced at its place by a neutral label; empty names stay empty.
    #[test]
    fn issue_210_ak5_neutral_labels_in_the_plan() {
        let n = neutral_plan(&plan());
        assert_eq!(n["project"]["name"], "Plan");
        assert_eq!(n["household"]["persons"][0]["name"], "Person 1");
        assert_eq!(n["household"]["persons"][1]["name"], "Person 2");
        assert_eq!(n["partner"]["name"], "Person 2");
        assert_eq!(n["flows"]["extras"][0]["label"], "Expense 1");
        assert_eq!(n["milestones"][0]["name"], "");
        assert_eq!(n["milestones"][1]["name"], "Milestone 2");
        assert_eq!(n["realestate"]["properties"][0]["name"], "Property 1");
        assert_eq!(
            n["realestate"]["properties"][0]["mortgage"]["tranches"][0]["name"],
            "Tranche 1"
        );
        assert!(!n.to_string().contains("Muster"));
    }

    /// #210/AK5: Names in messages are replaced as whole words, also the first or last name alone; paths become file names.
    #[test]
    fn issue_210_ak5_names_and_paths_in_texts() {
        let mut r = Redactor::new(Some(PathBuf::from("/Users/hans")));
        r.add_plan(&plan());
        r.add_file(Path::new("/Users/hans/Pläne/rente.vplan"));
        assert_eq!(
            r.text("Hans Muster: Rente ab 2030; Muster zahlt; Hansruedi bleibt"),
            "Person 1: Rente ab 2030; Person 1 zahlt; Hansruedi bleibt"
        );
        assert_eq!(
            r.text("Chalet verkauft (Festhypothek UBS)"),
            "Property 1 verkauft (Tranche 1)"
        );
        assert_eq!(
            r.text("File /Users/hans/Pläne/rente.vplan not found"),
            "File rente.vplan not found"
        );
        assert_eq!(r.text("in /Users/hans/Pläne/other.vplan"), "in other.vplan");
        assert_eq!(r.text("in /Users/hans/Desktop/x.csv"), "in ~/Desktop/x.csv");
        let mut v = json!({ "a": ["C:\\Users\\hans\\x.csv", "/opt/data/y.csv", "Eva Beispiel"], "b": 3 });
        r.value(&mut v);
        assert_eq!(v, json!({ "a": ["x.csv", "y.csv", "Person 2"], "b": 3 }));
    }

    /// #210/AK5: Without plans and home folder nothing changes.
    #[test]
    fn issue_210_ak5_nothing_to_replace() {
        let r = Redactor::new(None);
        assert_eq!(r.text("Plan 1 lasts"), "Plan 1 lasts");
        assert_eq!(replace_word("aXa", "X", "Y"), "aXa");
        assert_eq!(replace_word("X a X", "X", "Y"), "Y a Y");
    }
}
