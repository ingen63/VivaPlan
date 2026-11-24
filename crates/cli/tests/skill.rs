//! #208: The agent skill `vivaplan-skill/SKILL.md` matches the command line it explains.

// #116: memory limit per test process (engine::test_alloc).
#[global_allocator]
static TEST_ALLOC: engine::test_alloc::Limited = engine::test_alloc::Limited;

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};
use std::process::Command;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn skill() -> String {
    std::fs::read_to_string(root().join("vivaplan-skill/SKILL.md")).unwrap()
}

/// The command line without the real data folder of the app (as in `cli.rs`, #206).
fn vivaplan(args: &[&str]) -> String {
    let home = root().join("target/vivaplan-cli-no-home");
    let o = Command::new(env!("CARGO_BIN_EXE_vivaplan"))
        .args(args)
        .env("HOME", &home)
        .env("APPDATA", home.join("AppData").join("Roaming"))
        .env_remove("XDG_DATA_HOME")
        .current_dir(root())
        .output()
        .unwrap();
    assert!(o.status.success(), "{}", String::from_utf8_lossy(&o.stderr));
    String::from_utf8_lossy(&o.stdout).into_owned()
}

fn is_word(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '-' || c == '_'
}

/// Commands named in a text as `vivaplan <command>` (lowercase word after the program name).
fn commands(text: &str) -> BTreeSet<String> {
    text.match_indices("vivaplan ")
        .filter_map(|(i, m)| {
            let w: String = text[i + m.len()..].chars().take_while(|&c| is_word(c)).collect();
            (w.chars().next().is_some_and(|c| c.is_ascii_lowercase())).then_some(w)
        })
        .collect()
}

/// Options named in a text: `--name` and the short option `-o`.
fn options(text: &str) -> BTreeSet<String> {
    let chars: Vec<char> = text.chars().collect();
    let mut out = BTreeSet::new();
    for i in 0..chars.len() {
        let before_ok = i == 0 || !(is_word(chars[i - 1]));
        if !before_ok || chars[i] != '-' {
            continue;
        }
        let long = chars.get(i + 1) == Some(&'-');
        let start = if long { i + 2 } else { i + 1 };
        let name: String = chars[start..].iter().take_while(|&&c| is_word(c)).collect();
        if !name.starts_with(|c: char| c.is_ascii_lowercase()) {
            continue;
        }
        if long {
            out.insert(format!("--{name}"));
        } else if name == "o" {
            out.insert("-o".into());
        }
    }
    out
}

/// What the skill names but the help does not: commands as `vivaplan <command>`, options anywhere in the help.
fn missing_in_help(skill: &str, help: &str) -> Vec<String> {
    let mut missing: Vec<String> = commands(skill)
        .into_iter()
        .filter(|c| !help.contains(&format!("vivaplan {c}")))
        .map(|c| format!("command {c}"))
        .collect();
    let help_options = options(help);
    missing.extend(
        options(skill)
            .into_iter()
            .filter(|o| !help_options.contains(o))
            .map(|o| format!("option {o}")),
    );
    missing
}

// #208/AK3: All commands and options the skill names occur in `vivaplan --help`.
#[test]
fn issue_208_ak3_commands_and_options_of_the_skill_are_in_the_help() {
    let help = vivaplan(&["--help"]);
    assert!(help.contains("vivaplan --help"), "the help names itself");
    let s = skill();
    assert_eq!(missing_in_help(&s, &help), Vec::<String>::new());
    // The skill explains the commands of the issue.
    let named = commands(&s);
    for c in [
        "run", "set", "evaluate", "describe", "validate", "optimize", "diff", "schema",
    ] {
        assert!(named.contains(c), "{c} missing in the skill");
    }
    for o in [
        "--json",
        "--metric",
        "--yes",
        "--plan",
        "-o",
        "--overwrite",
        "--compare",
    ] {
        assert!(options(&s).contains(o), "{o} missing in the skill");
    }
}

// #208/AK3: The check finds a command or option that the help does not know (the test above can fail).
#[test]
fn issue_208_ak3_check_finds_unknown_commands_and_options() {
    let help = "vivaplan run <plan> [--json] [-o <file>]";
    assert_eq!(
        missing_in_help("`vivaplan run x --json -o y`", help),
        Vec::<String>::new()
    );
    assert_eq!(
        missing_in_help("`vivaplan simulate x --fast` and -o, not-an-option, --Upper", help),
        vec!["command simulate".to_string(), "option --fast".to_string()]
    );
}

// #208/AK3: The sections the skill names with `vivaplan describe <section>` exist.
#[test]
fn issue_208_ak3_describe_sections_of_the_skill_exist() {
    let out = vivaplan(&["describe", "limits"]);
    let known: BTreeSet<&str> = out
        .split_once("sections: ")
        .and_then(|(_, r)| r.split_once(" ("))
        .unwrap()
        .0
        .split(", ")
        .collect();
    let s = skill();
    let sections: BTreeSet<String> = s
        .match_indices("vivaplan describe ")
        .map(|(i, m)| s[i + m.len()..].chars().take_while(|&c| is_word(c)).collect())
        .filter(|w: &String| w.starts_with(|c: char| c.is_ascii_lowercase()))
        .collect();
    assert!(sections.len() >= 4, "{sections:?}");
    for w in sections {
        assert!(known.contains(w.as_str()), "{w} not in {known:?}");
    }
}

// #208/AK1: The metrics of the example «Does my wealth last until 95?» are valid and compute on the example plan.
#[test]
fn issue_208_ak1_example_metrics_compute_on_the_example_plan() {
    let s = skill();
    let metrics: Vec<&str> = s
        .match_indices("--metric '")
        .map(|(i, m)| {
            let rest = &s[i + m.len()..];
            &rest[..rest.find('\'').unwrap()]
        })
        .filter(|m| m.starts_with('{'))
        .collect();
    assert_eq!(metrics.len(), 2, "{metrics:?}");
    let dir = tempfile::tempdir().unwrap();
    let mut plan: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(root().join("crates/engine/assets/example.vplan")).unwrap())
            .unwrap();
    plan["returns"]["source"] = "parametric".into();
    plan["returns"]["parametric"]["paths"] = 50.into();
    let file = dir.path().join("plan.vplan");
    std::fs::write(&file, plan.to_string()).unwrap();
    let mut args = vec!["run", file.to_str().unwrap(), "--json"];
    for m in &metrics {
        args.extend(["--metric", m]);
    }
    let v: serde_json::Value = serde_json::from_str(&vivaplan(&args)).unwrap();
    assert!(v["error"].is_null(), "{v}");
    assert_eq!(v["batch"]["runs"], 50, "{v}");
    let out = v["metrics"].as_array().unwrap();
    assert_eq!(out.len(), 2);
    assert!(out.iter().all(|m| m["value"].is_number()), "{v}");
    let ruin = out[0]["value"].as_f64().unwrap();
    assert!((0.0..=1.0).contains(&ruin), "{ruin}");
}

// #208/AK1, AK2: Open skill format (name = folder, description) and the required contents.
#[test]
fn issue_208_ak1_ak2_format_and_contents() {
    let s = skill();
    let front = s.strip_prefix("---\n").and_then(|r| r.split_once("\n---\n")).unwrap().0;
    let field = |k: &str| {
        front
            .lines()
            .find_map(|l| l.strip_prefix(&format!("{k}: ")))
            .unwrap_or_else(|| panic!("{k} missing"))
            .to_string()
    };
    let name = field("name");
    assert_eq!(name, "vivaplan-skill", "the name equals the folder name");
    assert!(
        name.len() <= 64
            && name
                .chars()
                .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
    );
    let description = field("description");
    assert!(!description.is_empty() && description.len() <= 1024);
    // AK1: units (shares instead of percent), nominal and real, limits, disclaimer, language of the user.
    for t in [
        "fractions, not percent",
        "## Nominal and real",
        "## Limits and disclaimer",
        "Model calculation without\nguarantee. No tax or investment advice. Taxes are estimates.",
        "Answer in the language of the user",
    ] {
        assert!(s.contains(t), "{t:?} missing");
    }
    // AK2: data go to the provider of the AI.
    assert!(s.contains("is sent to the provider of this AI"));
}
