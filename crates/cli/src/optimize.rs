//! Befehl `optimize`: beste Entnahmestrategie für ein Basisszenario suchen.

use std::path::Path;

use engine::api;
use engine::model::{Goal, Plan, ReturnSource};
use engine::optimize::{self as opt, Objective, Outcome, Request};

use crate::{chf, library_dir, opt as option, positional, R};

fn num(args: &[String], name: &str) -> Result<Option<f64>, Box<dyn std::error::Error>> {
    option(args, &[name])
        .map(|s| s.replace(',', ".").trim_end_matches('%').parse::<f64>())
        .transpose()
        .map_err(|_| format!("{name}: Zahl erwartet").into())
}

fn pct1(v: f64) -> String {
    format!("{:.1} %", v * 100.0)
}

fn metrics_line(label: &str, m: &opt::Metrics) -> String {
    format!(
        "    {label:<12} Ruin {:>7}   Median {:>9}   P10 {:>9}   Schwankung {:>7}   Entnahmerate {:>6}   Notverkauf {:>6}",
        pct1(m.ruin),
        chf(m.median),
        chf(m.p10),
        pct1(m.volatility),
        pct1(m.draw_rate),
        pct1(m.emergency_share)
    )
}

fn outcome_text(o: &Outcome, obj: &Objective) -> String {
    let ok = if o.feasible {
        ""
    } else if obj.goal == Goal::KeepProperty {
        "  – Nebenbedingungen NICHT eingehalten (Ruingrenze oder Mindestkonsum)"
    } else {
        "  – Ruingrenze NICHT eingehalten"
    };
    let params: Vec<String> = o
        .params
        .iter()
        .map(|(_, label, v, percent)| {
            if *percent {
                format!("{label} {:.1} %", v * 100.0)
            } else {
                format!("{label} {v:.1}")
            }
        })
        .collect();
    let target = if obj.goal == Goal::KeepProperty {
        format!(
            "    Ziel (Prüfung): ungeplanter Immobilienverkauf in {} der Läufe",
            pct1(o.validated.emergency_share)
        )
    } else {
        format!("    Ziel (Prüfung): {} pro Jahr", chf(o.validated.score(obj)))
    };
    format!(
        "\n{}{ok}\n    {}\n{}\n{}\n{target}",
        o.strategy,
        params.join(", "),
        metrics_line("Optimierung", &o.fitted),
        metrics_line("Prüfung", &o.validated)
    )
}

/// Anfrage aus den gespeicherten Einstellungen der Planung und den Optionen der Kommandozeile.
fn request_from(plan: &Plan, args: &[String]) -> Result<Request, Box<dyn std::error::Error>> {
    // Gespeicherte Einstellungen der Planung gelten als Ausgangspunkt, Optionen überschreiben sie.
    let mut req = plan.optimization.as_ref().map_or_else(Request::default, Request::from);
    if let Some(v) = num(args, "--max-ruin")? {
        req.max_ruin = v / 100.0;
    }
    if let Some(g) = option(args, &["--goal"]) {
        req.goal = match g.as_str() {
            "1" | "consumption" => Goal::Consumption,
            "2" | "keepProperty" | "keep-property" => Goal::KeepProperty,
            _ => return Err("--goal: 1 (consumption) oder 2 (keepProperty) erwartet".into()),
        };
    }
    if let Some(v) = num(args, "--min-consumption")? {
        req.min_consumption = v / 100.0;
    }
    if let Some(v) = num(args, "--ruin-age")? {
        req.ruin_age = Some(v);
    }
    if let Some(v) = num(args, "--weight-median")? {
        req.weight_median = v;
    }
    if let Some(v) = num(args, "--vol-weight")? {
        req.vol_weight = v;
    }
    if let Some(v) = num(args, "--max-vol")? {
        req.max_vol = Some(v / 100.0);
    }
    if let Some(v) = num(args, "--paths")? {
        req.paths = v as usize;
    }
    if let Some(v) = num(args, "--check-paths")? {
        req.check_paths = v as usize;
    }
    if let Some(v) = num(args, "--samples")? {
        req.samples = v as usize;
    }
    if let Some(v) = num(args, "--seed")? {
        req.seed = v as u64;
    }
    if let Some(s) = option(args, &["--strategy"]) {
        req.strategies = s.split(',').map(str::to_string).collect();
    }
    Ok(req)
}

pub fn cmd_optimize(args: &[String]) -> R {
    let path = positional(args).ok_or("Projektdatei fehlt")?;
    let base_dir = Path::new(path).parent().map(Path::to_path_buf).unwrap_or_default();
    let base = api::read_plan(Path::new(path))?;
    let plan: Plan = serde_json::from_value(base.clone())?;
    if matches!(plan.returns.source, ReturnSource::Fixed | ReturnSource::Historical) {
        eprintln!(
            "Hinweis: Mit der Renditequelle «{}» gibt es nur einen Pfad; Ruin ist dann 0 % oder 100 %. \
             Für sinnvolle Ergebnisse Bootstrap, parametrisch oder rollierend wählen.",
            if plan.returns.source == ReturnSource::Fixed {
                "fix"
            } else {
                "historisch"
            }
        );
    }
    let req = request_from(&plan, args)?;
    let obj = req.objective();
    let report = opt::run_request(&base, &base_dir, Some(&library_dir(args)), &req)?;

    println!(
        "Optimierung von «{path}»: Pensionierung {}, {} Pfade (Suche), {} (Prüfung)",
        report.retire, report.fit_paths, report.check_paths
    );
    if obj.goal == Goal::KeepProperty {
        println!(
            "Ziel 2 «Kein Notverkauf von Immobilien»: möglichst wenige ungeplante Verkäufe, bei Gleichstand geringste Ruinwahrscheinlichkeit; \
             Nebenbedingungen: Ruin ≤ {}, Median des Konsums ≥ {} der geplanten Ausgaben",
            pct1(obj.max_ruin),
            pct1(obj.min_consumption)
        );
    } else {
        println!(
            "Ziel: Ruin ≤ {}{}; maximiere {:.0} % Median + {:.0} % P10 des realen Konsums, Abzug {:.0} × Schwankung{}",
            pct1(obj.max_ruin),
            obj.ruin_age
                .map_or(" (bis Simulationsende)".to_string(), |a| format!(" bis Alter {a}")),
            obj.weight_median * 100.0,
            (1.0 - obj.weight_median) * 100.0,
            obj.vol_weight,
            obj.max_vol
                .map_or(String::new(), |v| format!("; Schwankung ≤ {}", pct1(v)))
        );
    }
    println!("Konsum in Preisen des Startmonats, pro Jahr, inkl. Wohnkosten, vor Steuern.");
    println!("\nAusgangslage ({}, unverändert)", report.reference.strategy);
    println!("{}", metrics_line("Prüfung", &report.reference.metrics));
    for o in &report.outcomes {
        println!("{}", outcome_text(o, &obj));
    }
    if let Some(best) = report.outcomes.first() {
        println!(
            "\nBeste Strategie nach der Prüfung: {}{}",
            best.strategy,
            if best.feasible {
                ""
            } else {
                " (keine hält die Ruingrenze ein)"
            }
        );
    }
    println!("Rechenzeit: {:.1} s", report.elapsed_ms / 1e3);
    if let Some(json) = option(args, &["--json"]) {
        std::fs::write(&json, serde_json::to_string_pretty(&report.outcomes)?)?;
        println!("Ergebnis: {json}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    //! FEA-82: Optionen und Ausgabe des Befehls `optimize`, ohne die Optimierung selbst zu rechnen.
    use super::*;
    use engine::optimize::Metrics;

    fn args(list: &[&str]) -> Vec<String> {
        list.iter().map(|s| s.to_string()).collect()
    }

    fn plan() -> Plan {
        serde_json::from_value(api::example_plan()).unwrap()
    }

    fn metrics(ruin: f64) -> Metrics {
        Metrics {
            runs: 100,
            ruin,
            median: 100_000.0,
            p10: 80_000.0,
            volatility: 0.05,
            draw_rate: 0.04,
            emergency_share: 0.125,
            planned: 120_000.0,
        }
    }

    fn outcome(feasible: bool) -> Outcome {
        Outcome {
            strategy: "Guardrails".into(),
            params: vec![
                ("rate".into(), "Startrate".into(), 0.045, true),
                ("cashYears".into(), "Topf 1".into(), 2.5, false),
            ],
            fixed: vec![],
            fitted: metrics(0.02),
            validated: metrics(0.03),
            feasible,
            evaluations: 10,
            patch: serde_json::json!({}),
        }
    }

    #[test]
    fn every_option_overrides_the_saved_settings() {
        let a = args(&[
            "p.vplan",
            "--max-ruin",
            "7,5%",
            "--goal",
            "2",
            "--min-consumption",
            "85",
            "--ruin-age",
            "90",
            "--weight-median",
            "0.6",
            "--vol-weight",
            "2",
            "--max-vol",
            "25",
            "--paths",
            "111",
            "--check-paths",
            "222",
            "--samples",
            "5",
            "--seed",
            "9",
            "--strategy",
            "standard,vpw",
        ]);
        let r = request_from(&plan(), &a).unwrap();
        assert!((r.max_ruin - 0.075).abs() < 1e-12);
        assert_eq!(r.goal, Goal::KeepProperty);
        assert!((r.min_consumption - 0.85).abs() < 1e-12);
        assert_eq!(r.ruin_age, Some(90.0));
        assert_eq!((r.weight_median, r.vol_weight), (0.6, 2.0));
        assert!((r.max_vol.unwrap() - 0.25).abs() < 1e-12);
        assert_eq!((r.paths, r.check_paths, r.samples, r.seed), (111, 222, 5, 9));
        assert_eq!(r.strategies, ["standard", "vpw"]);
    }

    #[test]
    fn without_options_the_defaults_stay_and_bad_values_are_errors() {
        let r = request_from(&plan(), &args(&["p.vplan"])).unwrap();
        assert_eq!(r.goal, Goal::Consumption);
        assert_eq!(r.max_vol, None);
        let goal = request_from(&plan(), &args(&["p.vplan", "--goal", "3"])).unwrap_err();
        assert!(goal.to_string().contains("--goal"));
        let n = request_from(&plan(), &args(&["p.vplan", "--paths", "viele"])).unwrap_err();
        assert_eq!(n.to_string(), "--paths: Zahl erwartet");
        for g in ["1", "consumption"] {
            assert_eq!(
                request_from(&plan(), &args(&["p.vplan", "--goal", g])).unwrap().goal,
                Goal::Consumption
            );
        }
        for g in ["keepProperty", "keep-property"] {
            assert_eq!(
                request_from(&plan(), &args(&["p.vplan", "--goal", g])).unwrap().goal,
                Goal::KeepProperty
            );
        }
    }

    #[test]
    fn metrics_line_shows_all_six_figures() {
        let l = metrics_line("Prüfung", &metrics(0.02));
        assert!(l.starts_with("    Prüfung"), "{l}");
        for part in [
            "Ruin   2.0 %",
            "Schwankung   5.0 %",
            "Entnahmerate  4.0 %",
            "Notverkauf 12.5 %",
        ] {
            assert!(l.contains(part), "{part} in {l}");
        }
        assert!(l.contains("Median") && l.contains("P10"));
    }

    #[test]
    fn outcome_text_names_strategy_parameters_and_the_goal() {
        let consumption = request_from(&plan(), &args(&["p.vplan"])).unwrap().objective();
        let keep = request_from(&plan(), &args(&["p.vplan", "--goal", "2"]))
            .unwrap()
            .objective();
        let ok = outcome_text(&outcome(true), &consumption);
        assert!(
            ok.starts_with("\nGuardrails\n    Startrate 4.5 %, Topf 1 2.5\n"),
            "{ok}"
        );
        assert!(ok.contains("Optimierung") && ok.contains("Prüfung"));
        assert!(ok.contains("Ziel (Prüfung): ") && ok.ends_with(" pro Jahr"), "{ok}");
        assert!(!ok.contains("NICHT"));
        let bad = outcome_text(&outcome(false), &consumption);
        assert!(
            bad.starts_with("\nGuardrails  – Ruingrenze NICHT eingehalten\n"),
            "{bad}"
        );
        let bad_keep = outcome_text(&outcome(false), &keep);
        assert!(bad_keep.contains("Nebenbedingungen NICHT eingehalten"), "{bad_keep}");
        assert!(
            bad_keep.ends_with("ungeplanter Immobilienverkauf in 12.5 % der Läufe"),
            "{bad_keep}"
        );
        // ohne Verletzung der Nebenbedingungen steht bei Ziel 2 keine Warnung
        assert!(!outcome_text(&outcome(true), &keep).contains("NICHT"));
    }
}
