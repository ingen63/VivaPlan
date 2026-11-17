//! #44: Planungen im alten Format (Endung `.rplan`, Kennung `pensionsplaner/1`) bleiben lesbar
//! (legacy: RetirementPlanner). Neue Dateien tragen die Kennung `vivaplan/1`.

use std::path::Path;

use engine::api;
use engine::model::{FORMAT, LEGACY_FORMAT};
use serde_json::Value;

fn dir() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
}

fn sha256_hex(bytes: &[u8]) -> String {
    // Kleine SHA-256-Umsetzung, damit der Test ohne zusätzliche Abhängigkeit auskommt.
    const K: [u32; 64] = [
        0x428a2f98, 0x71374491, 0xb5c0fbcf, 0xe9b5dba5, 0x3956c25b, 0x59f111f1, 0x923f82a4, 0xab1c5ed5, 0xd807aa98,
        0x12835b01, 0x243185be, 0x550c7dc3, 0x72be5d74, 0x80deb1fe, 0x9bdc06a7, 0xc19bf174, 0xe49b69c1, 0xefbe4786,
        0x0fc19dc6, 0x240ca1cc, 0x2de92c6f, 0x4a7484aa, 0x5cb0a9dc, 0x76f988da, 0x983e5152, 0xa831c66d, 0xb00327c8,
        0xbf597fc7, 0xc6e00bf3, 0xd5a79147, 0x06ca6351, 0x14292967, 0x27b70a85, 0x2e1b2138, 0x4d2c6dfc, 0x53380d13,
        0x650a7354, 0x766a0abb, 0x81c2c92e, 0x92722c85, 0xa2bfe8a1, 0xa81a664b, 0xc24b8b70, 0xc76c51a3, 0xd192e819,
        0xd6990624, 0xf40e3585, 0x106aa070, 0x19a4c116, 0x1e376c08, 0x2748774c, 0x34b0bcb5, 0x391c0cb3, 0x4ed8aa4a,
        0x5b9cca4f, 0x682e6ff3, 0x748f82ee, 0x78a5636f, 0x84c87814, 0x8cc70208, 0x90befffa, 0xa4506ceb, 0xbef9a3f7,
        0xc67178f2,
    ];
    let mut h: [u32; 8] = [
        0x6a09e667, 0xbb67ae85, 0x3c6ef372, 0xa54ff53a, 0x510e527f, 0x9b05688c, 0x1f83d9ab, 0x5be0cd19,
    ];
    let mut msg = bytes.to_vec();
    msg.push(0x80);
    while msg.len() % 64 != 56 {
        msg.push(0);
    }
    msg.extend_from_slice(&((bytes.len() as u64) * 8).to_be_bytes());
    for chunk in msg.chunks(64) {
        let mut w = [0u32; 64];
        for i in 0..16 {
            w[i] = u32::from_be_bytes(chunk[i * 4..i * 4 + 4].try_into().unwrap());
        }
        for i in 16..64 {
            let s0 = w[i - 15].rotate_right(7) ^ w[i - 15].rotate_right(18) ^ (w[i - 15] >> 3);
            let s1 = w[i - 2].rotate_right(17) ^ w[i - 2].rotate_right(19) ^ (w[i - 2] >> 10);
            w[i] = w[i - 16].wrapping_add(s0).wrapping_add(w[i - 7]).wrapping_add(s1);
        }
        let mut v = h;
        for i in 0..64 {
            let s1 = v[4].rotate_right(6) ^ v[4].rotate_right(11) ^ v[4].rotate_right(25);
            let ch = (v[4] & v[5]) ^ (!v[4] & v[6]);
            let t1 = v[7]
                .wrapping_add(s1)
                .wrapping_add(ch)
                .wrapping_add(K[i])
                .wrapping_add(w[i]);
            let s0 = v[0].rotate_right(2) ^ v[0].rotate_right(13) ^ v[0].rotate_right(22);
            let maj = (v[0] & v[1]) ^ (v[0] & v[2]) ^ (v[1] & v[2]);
            let t2 = s0.wrapping_add(maj);
            v = [
                t1.wrapping_add(t2),
                v[0],
                v[1],
                v[2],
                v[3].wrapping_add(t1),
                v[4],
                v[5],
                v[6],
            ];
        }
        for i in 0..8 {
            h[i] = h[i].wrapping_add(v[i]);
        }
    }
    h.iter().map(|x| format!("{x:08x}")).collect()
}

/// Ergebnis eines Laufs ohne die gemessene Rechenzeit.
fn result(plan: &Value) -> Value {
    let mut r = serde_json::to_value(api::run(plan, dir(), Some(&dir().join("data")))).unwrap();
    r.as_object_mut().unwrap().remove("elapsedMs");
    r
}

/// #44/AK3: Die alte .rplan-Datei ist unverändert (Prüfsumme), öffnet mit identischen Werten wie die .vplan und
/// ergibt dieselben Ergebnisse.
#[test]
fn rplan_bleibt_unveraendert_und_rechnet_wie_die_vplan() {
    let alt = dir().join("tests/fixtures/planung-altformat.rplan");
    assert_eq!(
        sha256_hex(&std::fs::read(&alt).unwrap()),
        "7024ac6dea1691efda90b191cbc807aa8232c1b0bad2f054b1596f603b36c2e1",
        "Die Testdatei im alten Format darf nicht verändert werden"
    );
    let rplan = api::read_plan(&alt).unwrap();
    assert_eq!(rplan["format"], LEGACY_FORMAT);
    let neu = api::read_plan(&dir().join("assets/beispiel.vplan")).unwrap();
    assert_eq!(neu["format"], FORMAT);
    let upgraded = api::upgrade_plan(&rplan).plan;
    assert_eq!(upgraded["format"], FORMAT);
    assert_eq!(upgraded, api::upgrade_plan(&neu).plan);
    assert_eq!(result(&rplan), result(&neu));
}

/// #44: Neue Dateien schreiben `vivaplan/1`, auch wenn die Planung noch die alte Kennung trägt.
#[test]
fn schreiben_setzt_die_neue_kennung() {
    assert_eq!(FORMAT, "vivaplan/1");
    let d = std::env::temp_dir().join(format!("vivaplan-legacy-format-{}", std::process::id()));
    std::fs::create_dir_all(&d).unwrap();
    let f = d.join("neu.vplan");
    let alt = api::read_plan(&dir().join("tests/fixtures/planung-altformat.rplan")).unwrap();
    api::write_plan(&f, &alt).unwrap();
    let wieder = api::read_plan(&f).unwrap();
    assert_eq!(wieder["format"], FORMAT);
    let mut ohne_kennung = alt.clone();
    ohne_kennung["format"] = Value::String(FORMAT.into());
    assert_eq!(wieder, ohne_kennung);
    // Eine andere Kennung bleibt, wie sie ist.
    api::write_plan(&f, &serde_json::json!({"format": "x/2"})).unwrap();
    assert_eq!(api::read_plan(&f).unwrap()["format"], "x/2");
    let _ = std::fs::remove_dir_all(&d);
}

#[test]
fn sha256_stimmt() {
    assert_eq!(
        sha256_hex(b"abc"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    assert_eq!(
        sha256_hex(b""),
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
}

/// #44: Das Upgrade ersetzt nur die alte Kennung, und zwar ohne Hinweis; fehlende, fremde oder nicht textuelle
/// Kennungen und Nicht-Objekte bleiben unverändert.
#[test]
fn upgrade_ersetzt_nur_die_alte_kennung() {
    let alt = api::upgrade_plan(&serde_json::json!({"format": LEGACY_FORMAT}));
    assert_eq!(alt.plan["format"], FORMAT);
    assert!(alt.notes.is_empty(), "der Kennungswechsel wird nicht gemeldet");
    let neu = api::upgrade_plan(&serde_json::json!({"format": FORMAT}));
    assert_eq!(neu.plan["format"], FORMAT);
    let fremd = api::upgrade_plan(&serde_json::json!({"format": "x/2"}));
    assert_eq!(fremd.plan["format"], "x/2");
    let zahl = api::upgrade_plan(&serde_json::json!({"format": 5}));
    assert_eq!(zahl.plan["format"], 5);
    let ohne = api::upgrade_plan(&serde_json::json!({"a": 1}));
    assert!(ohne.plan.get("format").is_none(), "keine Kennung wird erfunden");
    let kein_objekt = api::upgrade_plan(&serde_json::json!([LEGACY_FORMAT]));
    assert_eq!(kein_objekt.plan, serde_json::json!([LEGACY_FORMAT]));
}

/// #44: Beim Schreiben bleiben Planungen ohne Kennung, mit Nicht-Text-Kennung und Nicht-Objekte unverändert.
#[test]
fn schreiben_erfindet_keine_kennung() {
    let d = std::env::temp_dir().join(format!("vivaplan-legacy-format-b-{}", std::process::id()));
    std::fs::create_dir_all(&d).unwrap();
    let f = d.join("x.vplan");
    for v in [
        serde_json::json!({"a": 1}),
        serde_json::json!({"format": 7}),
        serde_json::json!([1, 2]),
    ] {
        api::write_plan(&f, &v).unwrap();
        let text = std::fs::read_to_string(&f).unwrap();
        assert_eq!(serde_json::from_str::<Value>(&text).unwrap(), v);
    }
    let _ = std::fs::remove_dir_all(&d);
}
