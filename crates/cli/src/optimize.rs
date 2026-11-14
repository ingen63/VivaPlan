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

fn show(label: &str, m: &opt::Metrics) {
    println!(
        "    {label:<12} Ruin {:>7}   Median {:>9}   P10 {:>9}   Schwankung {:>7}   Entnahmerate {:>6}   Notverkauf {:>6}",
        pct1(m.ruin),
        chf(m.median),
        chf(m.p10),
        pct1(m.volatility),
        pct1(m.draw_rate),
        pct1(m.emergency_share)
    );
}

fn print_outcome(o: &Outcome, obj: &Objective) {
    let ok = if o.feasible {
        ""
    } else if obj.goal == Goal::KeepProperty {
        "  – Nebenbedingungen NICHT eingehalten (Ruingrenze oder Mindestkonsum)"
    } else {
        "  – Ruingrenze NICHT eingehalten"
    };
    println!("\n{}{ok}", o.strategy);
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
    println!("    {}", params.join(", "));
    show("Optimierung", &o.fitted);
    show("Prüfung", &o.validated);
    if obj.goal == Goal::KeepProperty {
        println!(
            "    Ziel (Prüfung): ungeplanter Immobilienverkauf in {} der Läufe",
            pct1(o.validated.emergency_share)
        );
    } else {
        println!("    Ziel (Prüfung): {} pro Jahr", chf(o.validated.score(obj)));
    }
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
    show("Prüfung", &report.reference.metrics);
    for o in &report.outcomes {
        print_outcome(o, &obj);
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
