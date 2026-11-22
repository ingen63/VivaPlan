//! Command `optimize`: search the best withdrawal strategy for a base scenario.

use std::path::Path;

use engine::api;
use engine::model::{Goal, Plan, PropertyMode, ReturnSource};
use engine::optimize::{self as opt, Objective, Outcome, Request};

use crate::{chf, library_dir, opt as option, positional, R};

fn num(args: &[String], name: &str) -> Result<Option<f64>, Box<dyn std::error::Error>> {
    option(args, &[name])
        .map(|s| s.replace(',', ".").trim_end_matches('%').parse::<f64>())
        .transpose()
        .map_err(|_| format!("{name}: number expected").into())
}

fn pct1(v: f64) -> String {
    format!("{:.1} %", v * 100.0)
}

fn metrics_line(label: &str, m: &opt::Metrics) -> String {
    format!(
        "    {label:<12} Ruin {:>7}   Median {:>9}   P10 {:>9}   Volatility {:>7}   Withdrawal rate {:>6}   Emergency sale {:>6}",
        pct1(m.ruin),
        chf(m.median),
        chf(m.p10),
        pct1(m.volatility),
        pct1(m.draw_rate),
        pct1(m.emergency_share)
    )
}

/// #197: Nominal end wealth at 1, 5, 50, 95 and 99 %.
fn end_line(m: &opt::Metrics) -> String {
    let parts: Vec<String> = opt::END_PERCENTS
        .iter()
        .zip(m.end_total)
        .map(|(p, v)| format!("{p} % {}", chf(v)))
        .collect();
    format!("    {:<12} {}", "End wealth", parts.join("   "))
}

/// #205: Share of the months from retirement at the floor («–» without a floor), with the hint from 50 %.
fn floor_line(m: &opt::Metrics) -> String {
    match m.floor_share {
        None => format!("    {:<12} –", "At floor"),
        Some(v) if v >= opt::FLOOR_HINT_SHARE => format!(
            "    {:<12} {} of the months from retirement – the floor mostly sets the spending; the strategy has little effect.",
            "At floor",
            pct1(v)
        ),
        Some(v) => format!("    {:<12} {} of the months from retirement", "At floor", pct1(v)),
    }
}

/// #197: Name of the strategy, with the property setting if the result shows it.
fn outcome_name(o: &Outcome, property: PropertyMode) -> String {
    match (property, o.never_sell) {
        (PropertyMode::Planned, _) => o.strategy.to_string(),
        (_, true) => format!("{} (never sell properties)", o.strategy),
        (_, false) => format!("{} (as planned)", o.strategy),
    }
}

fn outcome_text(o: &Outcome, obj: &Objective, property: PropertyMode, compare: bool) -> String {
    let ok = if o.feasible {
        ""
    } else if obj.goal == Goal::KeepProperty {
        "  – constraints NOT met (ruin limit or minimum consumption)"
    } else {
        "  – ruin limit NOT met"
    };
    let list = |ps: &[(String, engine::message::Message, f64, bool)]| {
        ps.iter()
            .map(|(_, label, v, percent)| {
                if *percent {
                    format!("{label} {:.1} %", v * 100.0)
                } else {
                    format!("{label} {v:.1}")
                }
            })
            .collect::<Vec<_>>()
            .join(", ")
    };
    // #204: the optimization also lists the fixed parameters (plan values), as the app does.
    let params = if compare {
        list(&o.fixed)
    } else if o.fixed.is_empty() {
        list(&o.params)
    } else {
        format!("{}\n    fixed: {}", list(&o.params), list(&o.fixed))
    };
    let target = if obj.goal == Goal::KeepProperty {
        format!(
            "    Goal (check): unplanned property sale in {} of the runs",
            pct1(o.validated.emergency_share)
        )
    } else {
        format!("    Goal (check): {} per year", chf(o.validated.score(obj)))
    };
    let lines = if compare {
        metrics_line("Plan values", &o.validated)
    } else {
        format!(
            "{}\n{}",
            metrics_line("Optimization", &o.fitted),
            metrics_line("Check", &o.validated)
        )
    };
    format!(
        "\n{}{ok}\n    {}\n{lines}\n{}\n{}\n{target}",
        outcome_name(o, property),
        params,
        end_line(&o.validated),
        floor_line(&o.validated)
    )
}

/// Request from the plan's saved settings and the command line options.
fn request_from(plan: &Plan, args: &[String]) -> Result<Request, Box<dyn std::error::Error>> {
    // The plan's saved settings are the starting point; options override them.
    let mut req = plan.optimization.as_ref().map_or_else(Request::default, Request::from);
    if let Some(v) = num(args, "--max-ruin")? {
        req.max_ruin = v / 100.0;
    }
    if let Some(g) = option(args, &["--goal"]) {
        req.goal = match g.as_str() {
            "1" | "consumption" => Goal::Consumption,
            "2" | "keepProperty" | "keep-property" => Goal::KeepProperty,
            _ => return Err("--goal: 1 (consumption) or 2 (keepProperty) expected".into()),
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
    // #197: comparison without search and property setting
    req.compare = args.iter().any(|a| a == "--compare");
    if let Some(p) = option(args, &["--property"]) {
        req.property = match p.as_str() {
            "planned" => PropertyMode::Planned,
            "never-sell" | "neverSell" => PropertyMode::NeverSell,
            "both" => PropertyMode::Both,
            _ => return Err("--property: planned, never-sell or both expected".into()),
        };
    }
    Ok(req)
}

pub fn cmd_optimize(args: &[String]) -> R {
    let path = positional(args).ok_or("Project file missing")?;
    let base_dir = Path::new(path).parent().map(Path::to_path_buf).unwrap_or_default();
    let base = api::read_plan(Path::new(path))?;
    let plan: Plan = serde_json::from_value(base.clone())?;
    if matches!(plan.returns.source, ReturnSource::Fixed | ReturnSource::Historical) {
        eprintln!(
            "Note: with the return source “{}” there is only one path; ruin is then 0 % or 100 %. \
             For meaningful results, choose bootstrap, parametric or rolling.",
            if plan.returns.source == ReturnSource::Fixed {
                "fixed"
            } else {
                "historical"
            }
        );
    }
    let req = request_from(&plan, args)?;
    let obj = req.objective();
    let report = opt::run_request(&base, &base_dir, Some(&library_dir(args)), &req)?;

    if report.compare {
        println!(
            "Comparison of strategies without optimization of “{path}”: retirement {}, {} paths (as in run)",
            report.retire, report.check_paths
        );
    } else {
        println!(
            "Optimization of “{path}”: retirement {}, {} paths (search), {} (check)",
            report.retire, report.fit_paths, report.check_paths
        );
    }
    match report.property {
        PropertyMode::Planned => {}
        PropertyMode::NeverSell => println!("Properties: never sell (switch on for all strategies)"),
        PropertyMode::Both => println!("Properties: every strategy as planned and with “never sell”"),
    }
    if obj.goal == Goal::KeepProperty {
        println!(
            "Goal 2 “No emergency sale of real estate”: as few unplanned sales as possible, on a tie the lowest probability of ruin; \
             constraints: ruin ≤ {}, median consumption ≥ {} of the planned spending",
            pct1(obj.max_ruin),
            pct1(obj.min_consumption)
        );
    } else {
        println!(
            "Goal: ruin ≤ {}{}; maximize {:.0} % median + {:.0} % P10 of real consumption, minus {:.0} × volatility{}",
            pct1(obj.max_ruin),
            obj.ruin_age
                .map_or(" (until the simulation end)".to_string(), |a| format!(" until age {a}")),
            obj.weight_median * 100.0,
            (1.0 - obj.weight_median) * 100.0,
            obj.vol_weight,
            obj.max_vol
                .map_or(String::new(), |v| format!("; volatility ≤ {}", pct1(v)))
        );
    }
    println!("Consumption in prices of the start month, per year, incl. housing costs, before taxes.");
    println!("\nStarting point ({}, unchanged)", report.reference.strategy);
    println!("{}", metrics_line("Check", &report.reference.metrics));
    println!("{}", end_line(&report.reference.metrics));
    println!("{}", floor_line(&report.reference.metrics));
    for o in &report.outcomes {
        println!("{}", outcome_text(o, &obj, report.property, report.compare));
    }
    if let Some(best) = report.outcomes.first() {
        println!(
            "\nBest strategy after the check: {}{}",
            outcome_name(best, report.property),
            if best.feasible {
                ""
            } else {
                " (none meets the ruin limit)"
            }
        );
    }
    println!("Computing time: {:.1} s", report.elapsed_ms / 1e3);
    if let Some(json) = option(args, &["--json"]) {
        std::fs::write(&json, serde_json::to_string_pretty(&report.outcomes)?)?;
        println!("Result: {json}");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    //! #106: options and output of the command `optimize`, without running the optimization itself.
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
            end_total: [1.0, 5.0, 50.0, 95.0, 99.0],
            floor_share: None,
        }
    }

    fn outcome(feasible: bool) -> Outcome {
        Outcome {
            strategy: engine::msg!(opt_strategy_guardrails),
            params: vec![
                ("rate".into(), engine::msg!(opt_param_initial_rate), 0.045, true),
                ("cashYears".into(), engine::msg!(opt_param_cash_years), 2.5, false),
            ],
            fixed: vec![],
            fitted: metrics(0.02),
            validated: metrics(0.03),
            feasible,
            evaluations: 10,
            patch: serde_json::json!({}),
            never_sell: false,
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
        let n = request_from(&plan(), &args(&["p.vplan", "--paths", "many"])).unwrap_err();
        assert_eq!(n.to_string(), "--paths: number expected");
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
        let l = metrics_line("Check", &metrics(0.02));
        assert!(l.starts_with("    Check"), "{l}");
        for part in [
            "Ruin   2.0 %",
            "Volatility   5.0 %",
            "Withdrawal rate  4.0 %",
            "Emergency sale 12.5 %",
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
        let ok = outcome_text(&outcome(true), &consumption, PropertyMode::Planned, false);
        assert!(
            ok.starts_with("\nGuardrails\n    Initial rate 4.5 %, Bucket 1: cash 2.5\n"),
            "{ok}"
        );
        assert!(ok.contains("Optimization") && ok.contains("Check"));
        assert!(ok.contains("Goal (check): ") && ok.ends_with(" per year"), "{ok}");
        assert!(!ok.contains("NOT"));
        let bad = outcome_text(&outcome(false), &consumption, PropertyMode::Planned, false);
        assert!(bad.starts_with("\nGuardrails  – ruin limit NOT met\n"), "{bad}");
        let bad_keep = outcome_text(&outcome(false), &keep, PropertyMode::Planned, false);
        assert!(bad_keep.contains("constraints NOT met"), "{bad_keep}");
        assert!(
            bad_keep.ends_with("unplanned property sale in 12.5 % of the runs"),
            "{bad_keep}"
        );
        // without a violated constraint, goal 2 shows no warning
        assert!(!outcome_text(&outcome(true), &keep, PropertyMode::Planned, false).contains("NOT"));
    }

    // #197/AK4: options --compare and --property, output of the comparison
    #[test]
    fn issue_197_ak4_compare_and_property_options() {
        let r = request_from(&plan(), &args(&["p.vplan"])).unwrap();
        assert!(!r.compare);
        assert_eq!(r.property, PropertyMode::Planned);
        let r = request_from(&plan(), &args(&["--compare", "p.vplan", "--property", "both"])).unwrap();
        assert!(r.compare);
        assert_eq!(r.property, PropertyMode::Both);
        for (v, m) in [
            ("planned", PropertyMode::Planned),
            ("never-sell", PropertyMode::NeverSell),
            ("neverSell", PropertyMode::NeverSell),
        ] {
            assert_eq!(
                request_from(&plan(), &args(&["p.vplan", "--property", v]))
                    .unwrap()
                    .property,
                m
            );
        }
        let e = request_from(&plan(), &args(&["p.vplan", "--property", "sometimes"])).unwrap_err();
        assert!(e.to_string().starts_with("--property:"), "{e}");
        // --compare takes no value: the file after it is still the project file
        let a = args(&["--compare", "p.vplan"]);
        assert_eq!(crate::positional(&a).map(String::as_str), Some("p.vplan"));
    }

    #[test]
    fn issue_197_ak4_output_shows_end_wealth_and_variant() {
        let l = end_line(&metrics(0.02));
        assert!(l.starts_with("    End wealth"), "{l}");
        for p in ["1 % ", "5 % ", "50 % ", "95 % ", "99 % "] {
            assert!(l.contains(p), "{p} in {l}");
        }
        let mut o = outcome(true);
        assert_eq!(outcome_name(&o, PropertyMode::Planned), "Guardrails");
        assert_eq!(outcome_name(&o, PropertyMode::Both), "Guardrails (as planned)");
        o.never_sell = true;
        assert_eq!(
            outcome_name(&o, PropertyMode::NeverSell),
            "Guardrails (never sell properties)"
        );
        // comparison: one line with the plan values, the fixed values as parameters
        o.fixed = vec![("rate".into(), engine::msg!(opt_param_initial_rate), 0.05, true)];
        let obj = request_from(&plan(), &args(&["p.vplan"])).unwrap().objective();
        let t = outcome_text(&o, &obj, PropertyMode::Both, true);
        assert!(t.contains("Initial rate 5.0 %"), "{t}");
        assert!(t.contains("Plan values") && !t.contains("Optimization"), "{t}");
        assert!(t.contains("End wealth"), "{t}");
    }

    #[test]
    fn issue_205_ak1_ak2_ak3_floor_line_with_share_and_hint() {
        let mut m = metrics(0.02);
        // AK3: without a floor «–» and no hint
        assert_eq!(floor_line(&m), "    At floor     –");
        // AK1: share without hint below 50 %
        m.floor_share = Some(0.25);
        let l = floor_line(&m);
        assert!(
            l.contains("25.0 % of the months from retirement") && !l.contains("mostly"),
            "{l}"
        );
        // AK2: hint from 50 %
        m.floor_share = Some(0.5);
        let l = floor_line(&m);
        assert!(
            l.contains("50.0 %") && l.contains("the floor mostly sets the spending"),
            "{l}"
        );
        // in the text of a strategy (comparison and optimization)
        let mut o = outcome(true);
        o.validated.floor_share = Some(1.0);
        let obj = request_from(&plan(), &args(&["p.vplan"])).unwrap().objective();
        for compare in [true, false] {
            let t = outcome_text(&o, &obj, PropertyMode::Planned, compare);
            assert!(t.contains("At floor     100.0 %") && t.contains("little effect"), "{t}");
        }
    }
}
