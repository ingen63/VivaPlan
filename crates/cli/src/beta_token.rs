//! #270: `beta-token`, the developer tool for the beta tokens: creates the key pair and issues tokens. It is not part of
//! the installation packages (`tools/cli-sidecar.mjs` copies only `vivaplan`). Instructions: `docs/ENTWICKLUNG.md`,
//! section «Beta-Token».

use std::io::Read;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use engine::beta;

const USAGE: &str = "beta-token – key pair and beta tokens of VivaPlan (developer tool, not shipped)

  beta-token keygen --out <file>
      creates a new key pair: writes the private key into <file> (outside the repository, never shared) and prints the
      public key for PUBLIC_KEY in crates/engine/src/beta.rs
  beta-token issue --key <file> --who <initials> --until <YYYY-MM-DD> [--id <id>]
      prints a token for the tester with the initials, redeemable until the date (inclusive); without --id a random one
  beta-token check <token> [--public <key>] [--today <YYYY-MM-DD>]
      checks a token against the built-in public key (or <key>) and shows its content
";

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args, &repo_root()) {
        Ok(out) => {
            print!("{out}");
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("Error: {e}");
            ExitCode::FAILURE
        }
    }
}

/// Root of the repository the tool was built from; the private key must not be written below it.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Runs a command; the text for the standard output.
fn run(args: &[String], repo: &Path) -> Result<String, String> {
    let opt = |name: &str| args.windows(2).find(|w| w[0] == name).map(|w| w[1].as_str());
    match args.first().map(String::as_str) {
        Some("keygen") => {
            let out = opt("--out").ok_or("keygen needs --out <file>")?;
            keygen(Path::new(out), repo, &random_bytes()?)
        }
        Some("issue") => {
            let key = opt("--key").ok_or("issue needs --key <file>")?;
            let who = opt("--who").ok_or("issue needs --who <initials>")?;
            let until = opt("--until").ok_or("issue needs --until <YYYY-MM-DD>")?;
            let text = std::fs::read_to_string(key).map_err(|e| format!("{key}: {e}"))?;
            let seed =
                beta::parse_private_key(&text).ok_or(format!("{key}: not a private key of beta-token keygen"))?;
            let id = match opt("--id") {
                Some(id) => id.to_string(),
                None => random_id(&random_bytes()?),
            };
            let token = beta::issue(&seed, &id, who, until).map_err(|m| m.text().to_string())?;
            Ok(format!("{token}\n"))
        }
        Some("check") => {
            let token = args.get(1).ok_or("check needs a token")?;
            let key = match opt("--public") {
                Some(hex) => {
                    Some(beta::parse_key(hex).ok_or("--public: not a public key (64 hexadecimal characters)")?)
                }
                None => beta::key(),
            };
            let today = opt("--today").map_or_else(beta::today, str::to_string);
            let claims = beta::verify(token, key.as_ref()).map_err(|m| m.text().to_string())?;
            let redeemable = match beta::redeem(token, key.as_ref(), &today) {
                Ok(_) => "yes".to_string(),
                Err(m) => format!("no – {}", m.text()),
            };
            Ok(format!(
                "valid: id {}, initials {}, redeemable until {} (inclusive)\nredeemable on {today}: {redeemable}\n",
                claims.id,
                claims.who,
                beta::swiss_date(&claims.until)
            ))
        }
        Some("--help" | "-h" | "help") => Ok(USAGE.to_string()),
        _ => Err(format!("unknown command\n\n{USAGE}")),
    }
}

/// Writes a new private key into `out` (not below `repo`, never over an existing file) and returns the instructions
/// with the public key.
fn keygen(out: &Path, repo: &Path, seed: &[u8; 32]) -> Result<String, String> {
    if out.exists() {
        return Err(format!("{} exists already; it is not replaced", out.display()));
    }
    let folder = match out.parent().filter(|p| !p.as_os_str().is_empty()) {
        Some(p) => p.to_path_buf(),
        None => std::env::current_dir().map_err(|e| e.to_string())?,
    };
    let folder = std::fs::canonicalize(&folder).map_err(|e| format!("{}: {e}", folder.display()))?;
    if let Ok(repo) = std::fs::canonicalize(repo) {
        if folder.starts_with(&repo) {
            return Err(format!(
                "{} is inside the repository {}; keep the private key outside of it",
                out.display(),
                repo.display()
            ));
        }
    }
    let (private, public) = beta::key_pair(seed);
    let text = format!("# VivaPlan beta tokens: private key. Never into the repository, never share it.\n{private}\n");
    write_private(out, &text).map_err(|e| format!("{}: {e}", out.display()))?;
    Ok(format!(
        "Private key written to {}.\nPublic key (put it into PUBLIC_KEY in crates/engine/src/beta.rs):\n{public}\n",
        out.display()
    ))
}

/// Writes the key file, readable only by its owner where the system knows that.
fn write_private(path: &Path, text: &str) -> std::io::Result<()> {
    let mut options = std::fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    std::os::unix::fs::OpenOptionsExt::mode(&mut options, 0o600);
    std::io::Write::write_all(&mut options.open(path)?, text.as_bytes())
}

/// 32 random bytes of the operating system.
fn random_bytes() -> Result<[u8; 32], String> {
    let mut b = [0; 32];
    std::fs::File::open("/dev/urandom")
        .and_then(|mut f| f.read_exact(&mut b))
        .map_err(|e| format!("no random numbers (/dev/urandom): {e}"))?;
    Ok(b)
}

/// Identifier of a token from random bytes: 6 upper case hexadecimal characters.
fn random_id(bytes: &[u8; 32]) -> String {
    bytes[..3].iter().map(|b| format!("{b:02X}")).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(a: &[&str]) -> Vec<String> {
        a.iter().map(|s| s.to_string()).collect()
    }

    // #270/AK1, AK11: keygen writes the private key outside the repository only; issue makes a token that the matching
    // public key accepts; check shows it.
    #[test]
    fn issue_270_ak1_keygen_issue_check() {
        let d = tempfile::tempdir().unwrap();
        let repo = d.path().join("repo");
        std::fs::create_dir(&repo).unwrap();
        let file = d.path().join("beta.key");
        let out = keygen(&file, &repo, &[5; 32]).unwrap();
        let public = out.lines().last().unwrap().to_string();
        assert_eq!(
            beta::parse_key(&public),
            Some(beta::parse_key(&beta::key_pair(&[5; 32]).1).unwrap())
        );
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(std::fs::metadata(&file).unwrap().permissions().mode() & 0o777, 0o600);
        }
        // Never over an existing file, never into the repository.
        assert!(keygen(&file, &repo, &[6; 32]).unwrap_err().contains("exists already"));
        assert!(keygen(&repo.join("k.key"), &repo, &[6; 32])
            .unwrap_err()
            .contains("inside the repository"));
        assert!(!repo.join("k.key").exists());
        assert!(keygen(&d.path().join("missing/k.key"), &repo, &[6; 32]).is_err());

        let key = file.to_str().unwrap();
        let token = run(
            &args(&[
                "issue",
                "--key",
                key,
                "--who",
                "AB",
                "--until",
                "2099-12-31",
                "--id",
                "T7",
            ]),
            &repo,
        )
        .unwrap();
        assert!(token.starts_with("VP1.beta.T7.AB.2099-12-31."), "{token}");
        let checked = run(
            &args(&["check", token.trim(), "--public", &public, "--today", "2026-11-25"]),
            &repo,
        )
        .unwrap();
        assert!(
            checked.contains("initials AB") && checked.contains("31.12.2099") && checked.contains(": yes"),
            "{checked}"
        );
        let late = run(
            &args(&["check", token.trim(), "--public", &public, "--today", "2100-01-01"]),
            &repo,
        )
        .unwrap();
        assert!(late.contains(": no – "), "{late}");
        // Without --id a random one.
        let random = run(
            &args(&["issue", "--key", key, "--who", "AB", "--until", "2099-12-31"]),
            &repo,
        )
        .unwrap();
        assert_eq!(random.split('.').nth(2).unwrap().len(), 6);
    }

    // #270: wrong calls end with a message.
    #[test]
    fn issue_270_tool_errors() {
        let d = tempfile::tempdir().unwrap();
        let repo = d.path();
        let bad = d.path().join("bad.key");
        std::fs::write(&bad, "nonsense").unwrap();
        let bad = bad.to_str().unwrap();
        for (a, expected) in [
            (vec!["keygen"], "--out"),
            (vec!["issue", "--who", "AB", "--until", "2099-12-31"], "--key"),
            (vec!["issue", "--key", bad, "--until", "2099-12-31"], "--who"),
            (vec!["issue", "--key", bad, "--who", "AB"], "--until"),
            (
                vec!["issue", "--key", bad, "--who", "AB", "--until", "2099-12-31"],
                "not a private key",
            ),
            (
                vec!["issue", "--key", "/missing/k", "--who", "AB", "--until", "2099-12-31"],
                "/missing/k",
            ),
            (vec!["check"], "needs a token"),
            (vec!["check", "VP1.beta.x", "--public", "zz"], "--public"),
            (vec!["frobnicate"], "unknown command"),
        ] {
            let e = run(&args(&a), repo).unwrap_err();
            assert!(e.contains(expected), "{a:?}: {e}");
        }
        let seed = d.path().join("ok.key");
        std::fs::write(&seed, "11".repeat(32)).unwrap();
        let e = run(
            &args(&[
                "issue",
                "--key",
                seed.to_str().unwrap(),
                "--who",
                "A.B",
                "--until",
                "2099-12-31",
            ]),
            repo,
        )
        .unwrap_err();
        assert!(e.contains("letters, digits or hyphens"), "{e}");
        assert!(run(&args(&["--help"]), repo).unwrap().contains("keygen"));
        assert_eq!(random_id(&[0xab; 32]), "ABABAB");
        assert!(random_bytes().is_ok());
        assert!(repo_root().join("Cargo.toml").is_file());
    }
}
