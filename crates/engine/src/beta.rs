//! #270: Beta token for the one-time activation of the desktop app and of `vivaplan mcp` during the closed beta.
//!
//! A token is one short line: `VP1.beta.<id>.<initials>.<YYYY-MM-DD>.<signature>`. `VP1` is the format version,
//! `beta` the channel, `<id>` the identifier of the token, `<initials>` the initials of the tester (never a full name),
//! the date the deadline for redeeming it (inclusive, UTC) and the signature an Ed25519 signature (base64url without
//! padding) of everything before the last dot. The check runs locally, without network.
//!
//! A token is needed only once: redeeming it writes the activation file [`ACTIVATION_FILE`] into the data folder of
//! the app (shared by the app and `vivaplan mcp`). An activation stays valid when the deadline of its token is over.
//! The token is a hurdle, not a protection: the code is public (GPL).
//!
//! The public key is [`PUBLIC_KEY`]; the private key never enters the repository (`docs/ENTWICKLUNG.md`, «Beta-Token»).
//! The tests use their own test key ([`TEST_SEED`]).

use std::path::Path;

pub use ed25519_dalek::VerifyingKey;
use ed25519_dalek::{Signature, Signer, SigningKey};
use serde::{Deserialize, Serialize};

use crate::message::Message;

// ------------------------------------------------------------------------------------------------------------------
// PUBLIC KEY OF THE BETA TOKENS
// Replace the placeholder with the public key that `beta-token keygen` prints (64 hexadecimal characters); see
// `docs/ENTWICKLUNG.md`, section «Beta-Token». As long as the placeholder is here, no token is accepted.
// ------------------------------------------------------------------------------------------------------------------
pub const PUBLIC_KEY: &str = "f07d5c0bef016aad376b025a74b720dee9f0c35c3bfaef999850f5111ae0767e";

/// Seed of the test key pair. Only for tests and the development server; never the real key.
pub const TEST_SEED: [u8; 32] = *b"vivaplan-beta-test-key-NOT-REAL!";

/// Environment variable that makes a debug build check against the test key (tests of `vivaplan mcp`); release builds
/// ignore it.
pub const TEST_KEY_VAR: &str = "VIVAPLAN_BETA_TEST_KEY";

/// File with the activation in the data folder of the app.
pub const ACTIVATION_FILE: &str = "beta-aktivierung.json";

/// Format version and channel at the start of every token.
const PREFIX: &str = "VP1.beta";

/// Whether this build needs an activation (Cargo feature `beta-token`, on by default until 1.0.0).
pub fn required() -> bool {
    cfg!(feature = "beta-token")
}

/// Content of a token.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Claims {
    /// Identifier of the token.
    pub id: String,
    /// Initials of the tester.
    pub who: String,
    /// Deadline for redeeming the token, `YYYY-MM-DD` (inclusive).
    pub until: String,
}

/// An activation as saved in [`ACTIVATION_FILE`] and shown in «Über VivaPlan».
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Activation {
    /// The redeemed token.
    pub token: String,
    /// Day of the activation, `YYYY-MM-DD`.
    pub activated: String,
}

/// Status for the user interface (`beta_status`): whether an activation is needed and the activation if there is one.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub required: bool,
    pub active: bool,
    pub who: Option<String>,
    pub id: Option<String>,
    pub activated: Option<String>,
}

impl Status {
    /// Status of the data folder `dir` with the key `key`.
    pub fn of(dir: Option<&Path>, key: Option<&VerifyingKey>) -> Status {
        let found = dir.and_then(|d| activation(d, key));
        Status {
            required: required(),
            active: !required() || found.is_some(),
            who: found.as_ref().map(|(c, _)| c.who.clone()),
            id: found.as_ref().map(|(c, _)| c.id.clone()),
            activated: found.map(|(_, a)| a.activated),
        }
    }

    /// Status after redeeming a token now.
    pub fn activated(claims: &Claims, activation: &Activation) -> Status {
        Status {
            required: required(),
            active: true,
            who: Some(claims.who.clone()),
            id: Some(claims.id.clone()),
            activated: Some(activation.activated.clone()),
        }
    }
}

/// The key the tokens are checked with: [`PUBLIC_KEY`], in a debug build with [`TEST_KEY_VAR`] set the test key.
/// `None` while [`PUBLIC_KEY`] is the placeholder.
pub fn key() -> Option<VerifyingKey> {
    if cfg!(debug_assertions) && std::env::var_os(TEST_KEY_VAR).is_some_and(|v| !v.is_empty()) {
        return Some(test_key());
    }
    parse_key(PUBLIC_KEY)
}

/// Public key of the test key pair.
pub fn test_key() -> VerifyingKey {
    SigningKey::from_bytes(&TEST_SEED).verifying_key()
}

/// A public key from 64 hexadecimal characters; `None` for anything else (the placeholder) or a weak key.
pub fn parse_key(hex: &str) -> Option<VerifyingKey> {
    let bytes: [u8; 32] = from_hex(hex.trim())?.try_into().ok()?;
    VerifyingKey::from_bytes(&bytes).ok().filter(|k| !k.is_weak())
}

/// A key pair from 32 random bytes: (private key as 64 hexadecimal characters, public key as the same).
pub fn key_pair(seed: &[u8; 32]) -> (String, String) {
    let signing = SigningKey::from_bytes(seed);
    (to_hex(signing.as_bytes()), to_hex(signing.verifying_key().as_bytes()))
}

/// The private key (seed) from a key file: 64 hexadecimal characters; lines starting with `#` are comments.
pub fn parse_private_key(text: &str) -> Option<[u8; 32]> {
    let hex: String = text
        .lines()
        .filter(|l| !l.trim_start().starts_with('#'))
        .collect::<String>()
        .split_whitespace()
        .collect();
    from_hex(&hex)?.try_into().ok()
}

/// Signs a new token with the private key `seed`. `id` and `who`: 1 to 16 letters, digits or hyphens; `until` a date
/// `YYYY-MM-DD`.
pub fn issue(seed: &[u8; 32], id: &str, who: &str, until: &str) -> Result<String, Message> {
    for (field, value) in [("id", id), ("who", who)] {
        if !valid_part(value) {
            return Err(msg!(beta_token_field_invalid, field = field, value = value));
        }
    }
    if !valid_date(until) {
        return Err(msg!(beta_token_date_invalid, value = until));
    }
    let body = format!("{PREFIX}.{id}.{who}.{until}");
    let signature = SigningKey::from_bytes(seed).sign(body.as_bytes());
    Ok(format!("{body}.{}", base64url(&signature.to_bytes())))
}

/// Checks format and signature of a token (not the deadline). Spaces and line breaks inside are ignored (a token
/// copied from an e-mail).
pub fn verify(token: &str, key: Option<&VerifyingKey>) -> Result<Claims, Message> {
    let token: String = token.split_whitespace().collect();
    let key = key.ok_or_else(|| msg!(beta_no_key))?;
    let (body, signature) = token.rsplit_once('.').ok_or_else(|| msg!(beta_token_malformed))?;
    let parts: Vec<&str> = body.split('.').collect();
    let [version, channel, id, who, until] = parts[..] else {
        return Err(msg!(beta_token_malformed));
    };
    if format!("{version}.{channel}") != PREFIX || !valid_part(id) || !valid_part(who) || !valid_date(until) {
        return Err(msg!(beta_token_malformed));
    }
    let signature = from_base64url(signature)
        .and_then(|b| <[u8; 64]>::try_from(b).ok())
        .ok_or_else(|| msg!(beta_token_malformed))?;
    key.verify_strict(body.as_bytes(), &Signature::from_bytes(&signature))
        .map_err(|_| msg!(beta_token_invalid))?;
    Ok(Claims {
        id: id.into(),
        who: who.into(),
        until: until.into(),
    })
}

/// Checks a token for redeeming on the day `today` (`YYYY-MM-DD`): format, signature and deadline.
pub fn redeem(token: &str, key: Option<&VerifyingKey>, today: &str) -> Result<Claims, Message> {
    let claims = verify(token, key)?;
    if today > claims.until.as_str() {
        return Err(msg!(beta_token_expired, until = swiss_date(&claims.until)));
    }
    Ok(claims)
}

/// Redeems a token and saves the activation in the data folder `dir` (created if needed).
pub fn activate(
    dir: &Path,
    token: &str,
    key: Option<&VerifyingKey>,
    today: &str,
) -> Result<(Claims, Activation), Message> {
    let claims = redeem(token, key, today)?;
    let activation = Activation {
        token: token.split_whitespace().collect(),
        activated: today.into(),
    };
    let text = serde_json::to_string_pretty(&activation).expect("serialisable");
    let file = dir.join(ACTIVATION_FILE);
    std::fs::create_dir_all(dir)
        .and_then(|()| std::fs::write(&file, text + "\n"))
        .map_err(|e| crate::io_error(&file, e).message())?;
    Ok((claims, activation))
}

/// The activation in the data folder `dir`, if there is one with a token signed with `key`. The deadline of the token
/// does not count any more.
pub fn activation(dir: &Path, key: Option<&VerifyingKey>) -> Option<(Claims, Activation)> {
    let text = std::fs::read_to_string(dir.join(ACTIVATION_FILE)).ok()?;
    let activation: Activation = serde_json::from_str(&text).ok()?;
    let claims = verify(&activation.token, key).ok()?;
    Some((claims, activation))
}

/// Today (UTC) as `YYYY-MM-DD`.
pub fn today() -> String {
    let secs = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_secs());
    date_of_day((secs / 86_400) as i64)
}

/// Date `YYYY-MM-DD` of the day `days` after 1970-01-01 (civil calendar, H. Hinnant's algorithm).
fn date_of_day(days: i64) -> String {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z - era * 146_097;
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = yoe + era * 400 + i64::from(month <= 2);
    format!("{year:04}-{month:02}-{day:02}")
}

/// `YYYY-MM-DD` as `DD.MM.YYYY` (Swiss format in every language).
pub fn swiss_date(date: &str) -> String {
    match date.split('-').collect::<Vec<_>>()[..] {
        [y, m, d] => format!("{d}.{m}.{y}"),
        _ => date.into(),
    }
}

/// 1 to 16 ASCII letters, digits or hyphens.
fn valid_part(s: &str) -> bool {
    (1..=16).contains(&s.len()) && s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-')
}

/// A date `YYYY-MM-DD` of the years 2000 to 2999 that exists.
fn valid_date(s: &str) -> bool {
    let b = s.as_bytes();
    if b.len() != 10 || b[4] != b'-' || b[7] != b'-' {
        return false;
    }
    let num = |r: std::ops::Range<usize>| s.get(r).filter(|t| t.bytes().all(|c| c.is_ascii_digit()))?.parse().ok();
    let (Some(y), Some(m), Some(d)) = (num(0..4), num(5..7), num(8..10)) else {
        return false;
    };
    let leap = y % 4 == 0 && (y % 100 != 0 || y % 400 == 0);
    let days = match m {
        2 if leap => 29,
        2 => 28,
        4 | 6 | 9 | 11 => 30,
        1..=12 => 31,
        _ => 0,
    };
    (2000..=2999).contains(&y) && (1..=days).contains(&d)
}

fn to_hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn from_hex(s: &str) -> Option<Vec<u8>> {
    if !s.len().is_multiple_of(2) || !s.is_ascii() {
        return None;
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).ok())
        .collect()
}

const ALPHABET: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789-_";

/// Base64url without padding (RFC 4648, section 5).
fn base64url(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
    for chunk in bytes.chunks(3) {
        let n = chunk
            .iter()
            .enumerate()
            .fold(0u32, |n, (i, &b)| n | u32::from(b) << (16 - 8 * i));
        for i in 0..=chunk.len() {
            out.push(ALPHABET[(n >> (18 - 6 * i) & 63) as usize] as char);
        }
    }
    out
}

/// Inverse of [`base64url`]; `None` for another character, a wrong length or bits left over.
fn from_base64url(s: &str) -> Option<Vec<u8>> {
    if s.len() % 4 == 1 {
        return None;
    }
    let values: Vec<u32> = s
        .bytes()
        .map(|c| ALPHABET.iter().position(|&a| a == c).map(|p| p as u32))
        .collect::<Option<_>>()?;
    let mut out = Vec::with_capacity(values.len() * 3 / 4);
    for chunk in values.chunks(4) {
        let n = chunk.iter().enumerate().fold(0u32, |n, (i, &v)| n | v << (18 - 6 * i));
        let bytes = chunk.len() - 1;
        if n & ((1 << (24 - 8 * bytes)) - 1) != 0 {
            return None;
        }
        out.extend((0..bytes).map(|i| (n >> (16 - 8 * i)) as u8));
    }
    Some(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    const FAR: &str = "2099-12-31";

    fn token(until: &str) -> String {
        issue(&TEST_SEED, "T001", "AB", until).unwrap()
    }

    fn code(r: Result<impl std::fmt::Debug, Message>) -> String {
        r.unwrap_err().code().to_string()
    }

    // #270/AK1: a token issued with the test key and a deadline in the future is redeemed and activates the data
    // folder; the activation is found again (next start).
    #[test]
    fn issue_270_ak1_token_activates_and_stays_active() {
        let d = tempfile::tempdir().unwrap();
        let dir = d.path().join("new-folder");
        let key = test_key();
        let t = token(FAR);
        assert!(t.starts_with("VP1.beta.T001.AB.2099-12-31."), "{t}");
        assert!(t.len() < 130, "short: {}", t.len());
        assert!(activation(&dir, Some(&key)).is_none());
        let (claims, act) = activate(&dir, &t, Some(&key), "2026-11-25").unwrap();
        assert_eq!((claims.who.as_str(), claims.id.as_str()), ("AB", "T001"));
        assert_eq!(act.activated, "2026-11-25");
        let (again, saved) = activation(&dir, Some(&key)).unwrap();
        assert_eq!((again, saved), (claims, act));
        let s = Status::of(Some(&dir), Some(&key));
        assert!(s.active);
        assert_eq!(s.who.as_deref(), Some("AB"));
        assert_eq!(s.activated.as_deref(), Some("2026-11-25"));
    }

    // #270/AK1: the deadline counts inclusive; a token copied with spaces and line breaks is accepted.
    #[test]
    fn issue_270_ak1_deadline_inclusive_and_whitespace_ignored() {
        let key = test_key();
        let t = token("2026-11-25");
        assert!(redeem(&t, Some(&key), "2026-11-25").is_ok());
        let broken = format!(" {}\n{} \n", &t[..40], &t[40..]);
        assert_eq!(redeem(&broken, Some(&key), "2026-01-01").unwrap().until, "2026-11-25");
    }

    // #270/AK2: a token whose deadline is over is refused with the message that a new token is needed.
    #[test]
    fn issue_270_ak2_expired_token_is_refused() {
        let key = test_key();
        let e = redeem(&token("2026-11-24"), Some(&key), "2026-11-25").unwrap_err();
        assert_eq!(e.code(), "beta_token_expired");
        assert!(
            e.text().contains("24.11.2026") && e.text().contains("new token"),
            "{}",
            e.text()
        );
        let d = tempfile::tempdir().unwrap();
        assert!(activate(d.path(), &token("2020-01-01"), Some(&key), "2026-11-25").is_err());
        assert!(!d.path().join(ACTIVATION_FILE).exists());
    }

    // #270/AK3: a token with one character changed (anywhere) and a token of another key are refused.
    #[test]
    fn issue_270_ak3_changed_or_foreign_token_is_refused() {
        let key = test_key();
        let t = token(FAR);
        for i in 0..t.len() {
            let mut b = t.clone().into_bytes();
            b[i] = if b[i] == b'A' { b'B' } else { b'A' };
            let changed = String::from_utf8(b).unwrap();
            assert!(verify(&changed, Some(&key)).is_err(), "position {i}: {changed}");
        }
        let foreign = issue(&[9; 32], "T001", "AB", FAR).unwrap();
        assert_eq!(code(verify(&foreign, Some(&key))), "beta_token_invalid");
        // The same token with the other key: valid.
        assert!(verify(&foreign, Some(&SigningKey::from_bytes(&[9; 32]).verifying_key())).is_ok());
        for bad in [
            "",
            "abc",
            "VP1.beta.T001.AB.2099-12-31",
            "VP2.beta.T001.AB.2099-12-31.AAAA",
        ] {
            assert_eq!(code(verify(bad, Some(&key))), "beta_token_malformed", "{bad}");
        }
    }

    // #270/AK3: without a real public key (placeholder) no token is accepted.
    #[test]
    fn issue_270_ak3_placeholder_key_accepts_nothing() {
        assert!(parse_key("PLACEHOLDER-REPLACE-WITH-THE-PUBLIC-KEY-FROM-beta-token-keygen").is_none());
        assert_eq!(code(verify(&token(FAR), None)), "beta_no_key");
        // A weak key (all zeros) is no key either.
        assert!(parse_key(&"00".repeat(32)).is_none());
        let (_, public) = key_pair(&TEST_SEED);
        assert_eq!(parse_key(&public), Some(test_key()));
        assert_eq!(parse_key(&format!(" {public}\n")), Some(test_key()));
    }

    // #270/AK11: the built-in key is the placeholder or a valid key, never the test key; key files and the key pair.
    #[test]
    fn issue_270_ak11_public_key_is_not_the_test_key() {
        let (_, test_public) = key_pair(&TEST_SEED);
        assert_ne!(PUBLIC_KEY.to_lowercase(), test_public);
        assert!(PUBLIC_KEY.starts_with("PLACEHOLDER") || parse_key(PUBLIC_KEY).is_some());
        // Without the environment variable the debug build checks against the built-in key.
        if std::env::var_os(TEST_KEY_VAR).is_none() {
            assert_eq!(key(), parse_key(PUBLIC_KEY));
        }
        let (private, public) = key_pair(&[7; 32]);
        assert_eq!(private, "07".repeat(32));
        assert_eq!(public.len(), 64);
        let file = format!(
            "# VivaPlan beta token private key\n{}\n{}\n",
            &private[..32],
            &private[32..]
        );
        assert_eq!(parse_private_key(&file), Some([7; 32]));
        assert_eq!(parse_private_key("zz"), None);
        assert_eq!(parse_private_key(&"07".repeat(31)), None);
    }

    // #270/AK5: an activation stays valid when the deadline of its token is over; a broken or foreign activation
    // file is no activation.
    #[test]
    fn issue_270_ak5_activation_outlives_the_deadline() {
        let d = tempfile::tempdir().unwrap();
        let key = test_key();
        activate(d.path(), &token("2026-11-30"), Some(&key), "2026-11-25").unwrap();
        // The status does not look at the date at all: after the deadline the activation is still there.
        assert!(today().as_str() > "2026-11-24");
        assert!(Status::of(Some(d.path()), Some(&key)).active);
        let file = d.path().join(ACTIVATION_FILE);
        std::fs::write(&file, "no json").unwrap();
        assert!(activation(d.path(), Some(&key)).is_none());
        let foreign = Activation {
            token: issue(&[9; 32], "T001", "AB", FAR).unwrap(),
            activated: "2026-11-25".into(),
        };
        std::fs::write(&file, serde_json::to_string(&foreign).unwrap()).unwrap();
        assert!(activation(d.path(), Some(&key)).is_none());
        let s = Status::of(Some(d.path()), Some(&key));
        assert_eq!((s.active, s.who), (!required(), None));
        assert!(!Status::of(None, Some(&key)).active || !required());
    }

    // #270: fields of a token are checked when issuing.
    #[test]
    fn issue_270_issue_checks_the_fields() {
        assert_eq!(code(issue(&TEST_SEED, "", "AB", FAR)), "beta_token_field_invalid");
        assert_eq!(code(issue(&TEST_SEED, "T1", "A.B", FAR)), "beta_token_field_invalid");
        assert_eq!(
            code(issue(&TEST_SEED, "T1", &"A".repeat(17), FAR)),
            "beta_token_field_invalid"
        );
        assert!(issue(&TEST_SEED, "T-1", &"A".repeat(16), FAR).is_ok());
        for bad in [
            "2026-02-29",
            "2026-13-01",
            "2026-04-31",
            "26-01-01",
            "2026-1-01",
            "1999-12-31",
            "2026-01-0x",
        ] {
            assert_eq!(
                code(issue(&TEST_SEED, "T1", "AB", bad)),
                "beta_token_date_invalid",
                "{bad}"
            );
        }
        for good in ["2028-02-29", "2000-02-29", "2026-12-31", "2026-04-30"] {
            assert!(valid_date(good), "{good}");
        }
        assert!(!valid_date("2100-02-29"));
    }

    // #270/AK3: a token signed with the right key but with an invalid field is refused as malformed.
    #[test]
    fn issue_270_ak3_signed_but_invalid_fields_are_refused() {
        let signing = SigningKey::from_bytes(&TEST_SEED);
        let signed = |body: &str| format!("{body}.{}", base64url(&signing.sign(body.as_bytes()).to_bytes()));
        assert!(verify(&signed("VP1.beta.T1.AB.2099-12-31"), Some(&test_key())).is_ok());
        for body in [
            "VP1.beta.T_1.AB.2099-12-31",
            "VP1.beta.T1.A_B.2099-12-31",
            "VP1.beta.T1.AB.2099-13-31",
            "VP1.gamma.T1.AB.2099-12-31",
        ] {
            assert_eq!(
                code(verify(&signed(body), Some(&test_key()))),
                "beta_token_malformed",
                "{body}"
            );
        }
    }

    // #270/AK11: the test key only with the environment variable (debug build), otherwise the built-in key; the
    // feature switch decides whether an activation is needed.
    #[test]
    fn issue_270_ak11_test_key_only_with_the_variable() {
        std::env::set_var(TEST_KEY_VAR, "1");
        assert_eq!(key(), Some(test_key()));
        std::env::set_var(TEST_KEY_VAR, "");
        assert_eq!(key(), parse_key(PUBLIC_KEY));
        std::env::remove_var(TEST_KEY_VAR);
        assert_eq!(key(), parse_key(PUBLIC_KEY));
        assert_eq!(required(), cfg!(feature = "beta-token"));
        assert!(required(), "on by default until 1.0.0");
    }

    // #270: dates and base64url.
    #[test]
    fn issue_270_dates_and_encoding() {
        assert_eq!(date_of_day(0), "1970-01-01");
        assert_eq!(date_of_day(20_417), "2025-11-25");
        assert_eq!(date_of_day(11_016), "2000-02-29");
        // Century and era boundaries of the calendar.
        for (day, date) in [
            (47_540, "2100-02-28"),
            (47_541, "2100-03-01"),
            (84_065, "2200-03-01"),
            (157_053, "2399-12-31"),
            (157_113, "2400-02-29"),
            (157_114, "2400-03-01"),
        ] {
            assert_eq!(date_of_day(day), date);
        }
        // Separators of a date.
        for bad in ["2026x01-01", "2026-01x01", "2026-01-011"] {
            assert!(!valid_date(bad), "{bad}");
        }
        assert_eq!(today().len(), 10);
        assert!(
            today().as_str() >= "2026-11-25" && today().as_str() < "2100-01-01",
            "{}",
            today()
        );
        assert!(valid_date(&today()));
        assert_eq!(swiss_date("2026-12-31"), "31.12.2026");
        assert_eq!(swiss_date("x"), "x");
        for (raw, enc) in [
            (&b""[..], ""),
            (b"f", "Zg"),
            (b"fo", "Zm8"),
            (b"foo", "Zm9v"),
            (b"\xfb\xff", "-_8"),
        ] {
            assert_eq!(base64url(raw), enc);
            assert_eq!(from_base64url(enc).unwrap(), raw);
        }
        assert_eq!(from_base64url("Z"), None);
        assert_eq!(from_base64url("A"), None, "a single character is no byte");
        assert_eq!(from_base64url("AAAAA"), None);
        assert_eq!(from_base64url("Zh"), None, "bits left over");
        assert_eq!(from_base64url("Z+8"), None);
        assert_eq!(from_hex("0g"), None);
        assert_eq!(from_hex("abc"), None);
    }
}
