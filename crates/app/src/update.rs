//! #309: Search for new versions, only with the consent of the user.
//!
//! This is the only network access of VivaPlan (exception to #213). It is limited three ways:
//! - the setting `updateCheck` must be `true` (the user said «Ja»); without it no search starts (AK4);
//! - only the desktop app has the updater plugin; the engine, the command line and `vivaplan mcp` have none (AK7,
//!   `tools/check-network-free.sh`);
//! - the user interface has no updater permission: it only calls the commands `update_check` and `update_install`
//!   of this module, the content security policy stays without external sources (#157/AK2).
//!
//! The search itself is a [`Search`] in the state of the app: the real app installs the one of the updater plugin
//! ([`install`]), the tests a stand-in that counts the requests.

use std::future::Future;
use std::pin::Pin;
use std::sync::Mutex;

use serde::Serialize;
use serde_json::{Map, Value};
use tauri::{AppHandle, Manager, Runtime};

/// Setting with the answer to the question at the first start: `true` (search), `false` (never); missing = not
/// asked yet. The user interface asks once and changes it in the settings (tab «Allgemein»).
pub const CONSENT_KEY: &str = "updateCheck";

/// A new version found by the search: shown with «Jetzt installieren» and «Später».
#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct Offer {
    pub version: String,
    /// Changes of the version; the release workflow writes them as JSON object per language (`tools/update_manifest.py`).
    pub notes: Option<String>,
    pub date: Option<String>,
}

/// Why an installation failed; `key` is a key of the text table, `reason` the text of the updater.
#[derive(Serialize, Debug, Clone, PartialEq)]
pub struct Failure {
    pub key: &'static str,
    pub reason: String,
}

type BoxFuture<T> = Pin<Box<dyn Future<Output = T> + Send>>;

/// Search for updates and installation of the version found; the only code that reaches the network.
pub struct Search {
    pub check: Box<dyn Fn() -> BoxFuture<Result<Option<Offer>, String>> + Send + Sync>,
    pub install: Box<dyn Fn() -> BoxFuture<Result<(), Failure>> + Send + Sync>,
}

/// AK4: the search runs only with the consent of the user (setting exactly `true`).
pub fn consented(settings: &Map<String, Value>) -> bool {
    settings.get(CONSENT_KEY) == Some(&Value::Bool(true))
}

/// AK3/AK4/AK5: searches for a new version if the user agreed. Without consent nothing happens (no network); an
/// error of the search (no internet, server) lands in the log and counts as «no new version».
pub async fn check(settings: &Map<String, Value>, search: Option<&Search>) -> Option<Offer> {
    if !consented(settings) {
        return None;
    }
    let search = search?;
    match (search.check)().await {
        Ok(offer) => offer,
        Err(e) => {
            eprintln!("Update-Suche fehlgeschlagen: {e}"); // i18n: allow (log)
            None
        }
    }
}

/// Installs the version found before; without consent or without a search an error, never network.
pub async fn install(settings: &Map<String, Value>, search: Option<&Search>) -> Result<(), Failure> {
    let refused = |reason: &str| Failure {
        key: "update.failed",
        reason: reason.to_string(),
    };
    if !consented(settings) {
        return Err(refused("no consent"));
    }
    let search = search.ok_or_else(|| refused("no updater"))?;
    (search.install)().await.inspect_err(|f| {
        eprintln!("Update nicht installiert: {}", f.reason); // i18n: allow (log)
    })
}

/// AK6: an invalid signature has its own text; every other error the general one.
pub fn failure(e: &tauri_plugin_updater::Error) -> Failure {
    use tauri_plugin_updater::Error as E;
    let key = match e {
        E::Minisign(_)
        | E::SignatureUtf8(_)
        | E::Base64(_)
        | E::SignedVersionMismatch { .. }
        | E::MissingSignedVersion => "update.badSignature",
        _ => "update.failed",
    };
    Failure {
        key,
        reason: e.to_string(),
    }
}

/// The version found by the last search of the real app, kept for the installation.
#[derive(Default)]
struct Pending(Mutex<Option<tauri_plugin_updater::Update>>);

/// Public key of the update signatures from `tauri.conf.json` (`plugins.updater.pubkey`); empty until set.
fn pubkey<R: Runtime>(app: &AppHandle<R>) -> String {
    app.config()
        .plugins
        .0
        .get("updater")
        .and_then(|u| u.get("pubkey"))
        .and_then(Value::as_str)
        .unwrap_or_default()
        .trim()
        .to_string()
}

/// Real app: search with the updater plugin (registered in `main`, not in the tests).
pub fn install_search<R: Runtime>(app: &AppHandle<R>) {
    use tauri_plugin_updater::UpdaterExt;
    app.manage(Pending::default());
    let (a, b) = (app.clone(), app.clone());
    app.manage(Search {
        check: Box::new(move || {
            let app = a.clone();
            Box::pin(async move {
                // without a public key no update could be verified: do not even ask the server
                if pubkey(&app).is_empty() {
                    return Err("no public key for update signatures in tauri.conf.json".into());
                }
                let found = app.updater().map_err(|e| e.to_string())?.check().await;
                let update = found.map_err(|e| e.to_string())?;
                let offer = update.as_ref().map(|u| Offer {
                    version: u.version.clone(),
                    notes: u.body.clone(),
                    date: u.date.map(|d| d.date().to_string()),
                });
                *app.state::<Pending>().0.lock().unwrap_or_else(|e| e.into_inner()) = update;
                Ok(offer)
            })
        }),
        install: Box::new(move || {
            let app = b.clone();
            Box::pin(async move {
                let update = app
                    .state::<Pending>()
                    .0
                    .lock()
                    .unwrap_or_else(|e| e.into_inner())
                    .take();
                let Some(update) = update else {
                    return Err(Failure {
                        key: "update.failed",
                        reason: "no update found".into(),
                    });
                };
                update
                    .download_and_install(|_, _| {}, || {})
                    .await
                    .map_err(|e| failure(&e))?;
                app.restart()
            })
        }),
    });
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    fn settings(v: Value) -> Map<String, Value> {
        v.as_object().unwrap().clone()
    }

    /// Stand-in for the network: counts the searches and installations.
    fn counting(result: Result<Option<Offer>, String>) -> (Search, Arc<AtomicUsize>) {
        let n = Arc::new(AtomicUsize::new(0));
        let (c, i) = (n.clone(), n.clone());
        let search = Search {
            check: Box::new(move || {
                c.fetch_add(1, Ordering::SeqCst);
                let r = result.clone();
                Box::pin(async move { r })
            }),
            install: Box::new(move || {
                i.fetch_add(1, Ordering::SeqCst);
                Box::pin(async { Ok(()) })
            }),
        };
        (search, n)
    }

    fn offer() -> Offer {
        Offer {
            version: "9.0.0".into(),
            notes: Some("neu".into()),
            date: None,
        }
    }

    // #309/AK4: without consent («Nein», no answer, anything but true) no search reaches the network.
    #[test]
    fn issue_309_ak4_no_network_without_consent() {
        for s in [
            json!({}),
            json!({ "updateCheck": false }),
            json!({ "updateCheck": "true" }),
            json!({ "updateCheck": 1 }),
            json!({ "updateCheck": null }),
        ] {
            let (search, n) = counting(Ok(Some(offer())));
            let found = tauri::async_runtime::block_on(check(&settings(s.clone()), Some(&search)));
            assert_eq!(found, None, "{s}");
            let installed = tauri::async_runtime::block_on(install(&settings(s.clone()), Some(&search)));
            assert!(installed.is_err(), "{s}");
            assert_eq!(n.load(Ordering::SeqCst), 0, "network reached with {s}");
        }
    }

    // #309/AK3: with consent the search runs and reports the version found.
    #[test]
    fn issue_309_ak3_search_with_consent() {
        let (search, n) = counting(Ok(Some(offer())));
        let yes = settings(json!({ "updateCheck": true }));
        assert_eq!(
            tauri::async_runtime::block_on(check(&yes, Some(&search))),
            Some(offer())
        );
        assert_eq!(n.load(Ordering::SeqCst), 1);
        assert_eq!(tauri::async_runtime::block_on(install(&yes, Some(&search))), Ok(()));
        assert_eq!(n.load(Ordering::SeqCst), 2);
        let (none, _) = counting(Ok(None));
        assert_eq!(tauri::async_runtime::block_on(check(&yes, Some(&none))), None);
    }

    // #309/AK5: no internet or a server error is no error for the user: no new version.
    #[test]
    fn issue_309_ak5_errors_are_silent() {
        let (search, n) = counting(Err("error sending request".into()));
        let yes = settings(json!({ "updateCheck": true }));
        assert_eq!(tauri::async_runtime::block_on(check(&yes, Some(&search))), None);
        assert_eq!(n.load(Ordering::SeqCst), 1);
        assert_eq!(tauri::async_runtime::block_on(check(&yes, None)), None);
        assert!(tauri::async_runtime::block_on(install(&yes, None)).is_err());
    }

    // #309/AK6: an invalid signature is named as such; other errors get the general text.
    #[test]
    fn issue_309_ak6_signature_error_has_its_own_text() {
        let bad = tauri_plugin_updater::Error::SignatureUtf8("bad".into());
        assert_eq!(failure(&bad).key, "update.badSignature");
        assert!(failure(&bad).reason.contains("bad"));
        let other = tauri_plugin_updater::Error::EmptyEndpoints;
        assert_eq!(failure(&other).key, "update.failed");
    }

    // #309/AK6: every signature-related error kind maps to the signature text, not only one of them.
    #[test]
    fn issue_309_ak6_all_signature_kinds_map_to_bad_signature() {
        use tauri_plugin_updater::Error as E;
        for e in [E::SignatureUtf8("x".into()), E::MissingSignedVersion] {
            assert_eq!(failure(&e).key, "update.badSignature", "{e}");
        }
    }

    // #309/AK4: an installation error of the updater reaches the caller unchanged (key and reason).
    #[test]
    fn issue_309_install_failure_is_passed_on() {
        let search = Search {
            check: Box::new(|| Box::pin(async { Ok(None) })),
            install: Box::new(|| {
                Box::pin(async {
                    Err(Failure {
                        key: "update.badSignature",
                        reason: "boom".into(),
                    })
                })
            }),
        };
        let yes = settings(json!({ "updateCheck": true }));
        let r = tauri::async_runtime::block_on(install(&yes, Some(&search)));
        assert_eq!(
            r,
            Err(Failure {
                key: "update.badSignature",
                reason: "boom".into()
            })
        );
    }

    // #309/AK4: refusals name their reason (no consent vs. no updater) and use the general text key.
    #[test]
    fn issue_309_install_refusal_reasons() {
        let no = settings(json!({}));
        let yes = settings(json!({ "updateCheck": true }));
        let (search, _) = counting(Ok(None));
        let a = tauri::async_runtime::block_on(install(&no, Some(&search))).unwrap_err();
        assert_eq!((a.key, a.reason.as_str()), ("update.failed", "no consent"));
        let b = tauri::async_runtime::block_on(install(&yes, None)).unwrap_err();
        assert_eq!((b.key, b.reason.as_str()), ("update.failed", "no updater"));
    }

    // #309/AK4: the real search without public key does not ask the server (error before any request), and an
    // installation without a found version fails; both run on the mock runtime without network.
    #[test]
    fn issue_309_real_search_without_pubkey_or_pending_update_fails_locally() {
        use tauri::test::{mock_builder, mock_context, noop_assets};
        let app = mock_builder()
            .build(mock_context(noop_assets()))
            .expect("mock app builds");
        let handle = app.handle().clone();
        assert_eq!(pubkey(&handle), "");
        install_search(&handle);
        let search = handle.state::<Search>();
        let checked = tauri::async_runtime::block_on((search.check)());
        assert_eq!(
            checked,
            Err("no public key for update signatures in tauri.conf.json".to_string())
        );
        let installed = tauri::async_runtime::block_on((search.install)());
        assert_eq!(
            installed,
            Err(Failure {
                key: "update.failed",
                reason: "no update found".into()
            })
        );
    }
}
