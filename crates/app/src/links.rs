//! #237: External addresses the app may open in the default browser of the system. The fixed list stands in one
//! place, `ui/links.json` (id → address), compiled into the app; the user interface reads the same file. The command
//! `open_link` takes only an id of this list, never an address, so the user interface cannot open anything else.
//! A new address is one more line in `ui/links.json`. #240: the introductory videos, one per length and language
//! (`video-<2min|10min>-<de|en|fr|it>`); the user interface picks the id of the language set.

use std::collections::BTreeMap;
use std::sync::OnceLock;

/// The fixed list of addresses (id → address).
const LINKS: &str = include_str!("../ui/links.json");

/// The parsed list; `ui/links.json` is checked by the tests.
pub fn all() -> &'static BTreeMap<String, String> {
    static L: OnceLock<BTreeMap<String, String>> = OnceLock::new();
    L.get_or_init(|| serde_json::from_str(LINKS).expect("links.json is valid JSON"))
}

/// Address of the entry `id`; an id not in the list (or any address passed instead) is rejected.
pub fn url(id: &str) -> Result<&'static str, String> {
    all()
        .get(id)
        .map(String::as_str)
        .ok_or_else(|| format!("unknown link: {id}"))
}

/// Opens the entry `id` with `open` (the default browser in the app, a stand-in in the tests).
pub fn open_with(id: &str, open: impl FnOnce(&str) -> Result<(), String>) -> Result<(), String> {
    open(url(id)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// #237/AK2: the donation link is in the list with the fixed address.
    #[test]
    fn issue_237_ak2_donate_link_in_list() {
        assert_eq!(url("donate"), Ok("https://buymeacoffee.com/vivaplan"));
    }

    /// #237: every entry is an https address on a fixed host, without spaces; ids are short words.
    #[test]
    fn issue_237_list_is_well_formed() {
        assert!(!all().is_empty());
        for (id, url) in all() {
            assert!(
                !id.is_empty() && id.chars().all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'),
                "{id}"
            );
            let host = url
                .strip_prefix("https://")
                .unwrap_or_else(|| panic!("{id}: nur https"));
            assert!(!host.is_empty() && !host.starts_with('/'), "{id}: Host fehlt");
            assert!(!url.contains(char::is_whitespace), "{id}");
        }
    }

    /// #237: an id outside the list and any address are rejected; the opener is never called.
    #[test]
    fn issue_237_rejects_unknown_ids_and_addresses() {
        for bad in [
            "",
            "unknown",
            "DONATE",
            "https://buymeacoffee.com/vivaplan",
            "https://example.com",
            "file:///etc/passwd",
            "../donate",
        ] {
            let r = open_with(bad, |_| panic!("darf nicht öffnen: {bad}"));
            assert!(r.is_err(), "{bad}");
        }
    }

    /// #237/AK2: a known id passes exactly its address to the opener; its error is returned.
    #[test]
    fn issue_237_ak2_opens_listed_address() {
        let mut seen = None;
        assert_eq!(
            open_with("donate", |u| {
                seen = Some(u.to_string());
                Ok(())
            }),
            Ok(())
        );
        assert_eq!(seen.as_deref(), Some("https://buymeacoffee.com/vivaplan"));
        assert_eq!(
            open_with("donate", |_| Err("kein Browser".into())),
            Err("kein Browser".into())
        );
    }

    /// #240/AK4: the list holds exactly the donation link and the eight videos (two lengths, four languages).
    #[test]
    fn issue_240_ak4_list_holds_exactly_these_addresses() {
        let mut want = vec![("donate".to_string(), "https://buymeacoffee.com/vivaplan".to_string())];
        for lang in ["de", "en", "fr", "it"] {
            for len in ["2min", "10min"] {
                want.push((
                    format!("video-{len}-{lang}"),
                    format!("https://archive.org/details/vivaplan-video-{lang}/vivaplan-{len}.mp4"),
                ));
            }
        }
        want.sort();
        let got: Vec<_> = all().iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        assert_eq!(got, want);
    }

    /// #240/AK4: other video ids and the video addresses themselves are rejected; the opener is never called.
    #[test]
    fn issue_240_ak4_rejects_other_videos() {
        for bad in [
            "video-2min",
            "video-2min-es",
            "video-5min-de",
            "VIDEO-2MIN-DE",
            "https://archive.org/details/vivaplan-video-de/vivaplan-2min.mp4",
            "https://archive.org/details/vivaplan-video-de",
        ] {
            let r = open_with(bad, |_| panic!("darf nicht öffnen: {bad}"));
            assert!(r.is_err(), "{bad}");
        }
        let mut seen = None;
        assert_eq!(
            open_with("video-10min-fr", |u| {
                seen = Some(u.to_string());
                Ok(())
            }),
            Ok(())
        );
        assert_eq!(
            seen.as_deref(),
            Some("https://archive.org/details/vivaplan-video-fr/vivaplan-10min.mp4")
        );
    }
}
