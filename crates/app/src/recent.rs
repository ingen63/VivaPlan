//! #227: list of the plans opened or saved last («Datei → Zuletzt geöffnet»). The list holds only paths and lives
//! in the local settings of the app; the most recent plan comes first.

use std::path::Path;

use serde_json::{Map, Value};

/// Setting with the list of recent plans (array of paths, most recent first).
pub const KEY: &str = "recentPlans";
/// Older setting with only the plan last opened (before #227); taken over as the first entry.
pub const LEGACY_KEY: &str = "lastPlan";
/// Maximum number of entries.
pub const MAX: usize = 10;

/// Recent plans from the settings. Settings that only know `lastPlan` yield that path as the only entry.
pub fn list(settings: &Map<String, Value>) -> Vec<String> {
    match settings.get(KEY) {
        Some(Value::Array(a)) => a
            .iter()
            .filter_map(Value::as_str)
            .filter(|p| !p.is_empty())
            .take(MAX)
            .map(str::to_string)
            .collect(),
        _ => settings
            .get(LEGACY_KEY)
            .and_then(Value::as_str)
            .filter(|p| !p.is_empty())
            .map(|p| vec![p.to_string()])
            .unwrap_or_default(),
    }
}

/// Puts `path` in the first place: an existing entry moves up, the oldest drops out beyond [`MAX`].
pub fn push(mut list: Vec<String>, path: &str) -> Vec<String> {
    list.retain(|p| p != path);
    list.insert(0, path.to_string());
    list.truncate(MAX);
    list
}

/// Writes the list into the settings; the older setting `lastPlan` is removed (the list replaces it).
pub fn store(settings: &mut Map<String, Value>, list: &[String]) {
    settings.remove(LEGACY_KEY);
    settings.insert(KEY.to_string(), Value::from(list.to_vec()));
}

fn stem(path: &str) -> String {
    let p = Path::new(path);
    match p.extension().and_then(|e| e.to_str()) {
        Some(e) if e.eq_ignore_ascii_case("vplan") => p.file_stem(),
        _ => p.file_name(),
    }
    .map(|n| n.to_string_lossy().into_owned())
    .unwrap_or_else(|| path.to_string())
}

fn parent(path: &str) -> String {
    Path::new(path)
        .parent()
        .map(|d| d.to_string_lossy().into_owned())
        .unwrap_or_default()
}

fn folder_name(path: &str) -> String {
    Path::new(path)
        .parent()
        .and_then(Path::file_name)
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_else(|| parent(path))
}

/// Menu texts: file name without `.vplan`; if two entries have the same name, the folder is added (its name, or
/// the whole folder path if the folder names are equal too).
pub fn labels(list: &[String]) -> Vec<String> {
    let stems: Vec<String> = list.iter().map(|p| stem(p)).collect();
    let folders: Vec<String> = list.iter().map(|p| folder_name(p)).collect();
    let count = |v: &[String], f: &dyn Fn(usize) -> bool| (0..v.len()).filter(|&j| f(j)).count();
    (0..list.len())
        .map(|i| {
            if count(&stems, &|j| stems[j] == stems[i]) < 2 {
                return stems[i].clone();
            }
            let same_folder = count(&stems, &|j| stems[j] == stems[i] && folders[j] == folders[i]);
            let folder = if same_folder < 2 {
                folders[i].clone()
            } else {
                parent(&list[i])
            };
            format!("{} ({folder})", stems[i])
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn map(v: Value) -> Map<String, Value> {
        match v {
            Value::Object(m) => m,
            _ => unreachable!(),
        }
    }

    fn paths(n: usize) -> Vec<String> {
        (0..n).map(|i| format!("/plans/p{i}.vplan")).collect()
    }

    /// #227/AK1: file name without `.vplan`, the folder only when two entries have the same name.
    #[test]
    fn issue_227_ak1_labels_without_extension_and_folder_for_equal_names() {
        let l = vec![
            "/a/Familie.vplan".to_string(),
            "/b/Familie.vplan".to_string(),
            "/a/Allein.VPLAN".to_string(),
            "/a/alt.rplan".to_string(),
        ];
        assert_eq!(labels(&l), ["Familie (a)", "Familie (b)", "Allein", "alt.rplan"]);
        // equal folder names too: the whole folder path
        let l = vec!["/x/k/Plan.vplan".to_string(), "/y/k/Plan.vplan".to_string()];
        assert_eq!(labels(&l), ["Plan (/x/k)", "Plan (/y/k)"]);
        assert!(labels(&[]).is_empty());
    }

    /// #227/AK2: the newest entry comes first, existing entries move up, no duplicates, at most 10.
    #[test]
    fn issue_227_ak2_push_moves_up_without_duplicates_and_keeps_ten() {
        let l = push(Vec::new(), "/a.vplan");
        let l = push(l, "/b.vplan");
        assert_eq!(l, ["/b.vplan", "/a.vplan"]);
        assert_eq!(push(l, "/a.vplan"), ["/a.vplan", "/b.vplan"]);
        let ten = paths(10);
        let l = push(ten.clone(), "/neu.vplan");
        assert_eq!(l.len(), MAX);
        assert_eq!(l[0], "/neu.vplan");
        assert_eq!(l[1..], ten[..9], "der älteste fällt weg");
        // an entry already in a full list moves up, nothing drops out
        let l = push(ten.clone(), &ten[9]);
        assert_eq!(l.len(), MAX);
        assert_eq!(l[0], ten[9]);
    }

    /// #227/AK6: the list from the settings; only `lastPlan` becomes the first entry, an existing list wins.
    #[test]
    fn issue_227_ak6_list_from_settings_and_legacy_last_plan() {
        assert!(list(&Map::new()).is_empty());
        assert_eq!(list(&map(json!({"lastPlan": "/a.vplan"}))), ["/a.vplan"]);
        assert!(list(&map(json!({"lastPlan": ""}))).is_empty());
        let s = map(json!({"lastPlan": "/alt.vplan", "recentPlans": ["/b.vplan", 3, "", "/c.vplan"]}));
        assert_eq!(list(&s), ["/b.vplan", "/c.vplan"]);
        // an empty list (cleared) does not bring back `lastPlan`
        assert!(list(&map(json!({"lastPlan": "/alt.vplan", "recentPlans": []}))).is_empty());
        let mut long = Map::new();
        long.insert(KEY.into(), Value::from(paths(12)));
        assert_eq!(list(&long).len(), MAX);
    }

    /// #227/AK6: storing replaces `lastPlan` and keeps the other settings (e.g. «Darstellung», #220).
    #[test]
    fn issue_227_ak6_store_replaces_last_plan_and_keeps_other_settings() {
        let mut s = map(json!({"lastPlan": "/alt.vplan", "theme": "dark"}));
        store(&mut s, &["/n.vplan".to_string()]);
        assert_eq!(Value::Object(s), json!({"theme": "dark", "recentPlans": ["/n.vplan"]}));
    }
}
