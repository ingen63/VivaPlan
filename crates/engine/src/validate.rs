//! Prüfung einer Planung: Fehler verhindern die Simulation, Hinweise nicht.

use serde::Serialize;

use crate::model::{person_milestone_name, person_milestones, PayoutAt, PayoutUse, AHV_NAME, PK_NAME};
use crate::model::{
    AmortMode, AmountMode, GainModel, MilestoneKind, Plan, PropertyStatus, RateKind, ReturnSource, Series, SeriesDef,
    Strategy, Withdrawal,
};
use crate::resolve::{Curve, Resolver};
use crate::time::{TimeRef, YearMonth};

/// Höchstes zulässiges Alter am Simulationsende.
const MAX_AGE: i32 = 120;
/// Längster erlaubter Personenname.
const MAX_NAME: usize = 60;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Error,
    Warning,
}

#[derive(Clone, Debug, Serialize)]
pub struct Issue {
    pub severity: Severity,
    /// Abschnitt im Editor (`general`, `milestones`, `pension`, `flows`, `wealth`, `returns`,
    /// `withdrawal`, `optimize`, `taxes`, `realestate`).
    pub section: &'static str,
    pub message: String,
    /// Pfad des betroffenen Feldes in der Planung (zum Beispiel `household.persons.0.name`); die Oberfläche zeigt die
    /// Meldung dann direkt am Feld.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
}

struct Issues(Vec<Issue>);

impl Issues {
    fn err(&mut self, section: &'static str, message: impl Into<String>) {
        self.0.push(Issue {
            severity: Severity::Error,
            section,
            message: message.into(),
            field: None,
        });
    }
    /// Fehler an einem bestimmten Feld der Planung.
    fn err_at(&mut self, section: &'static str, field: impl Into<String>, message: impl Into<String>) {
        self.0.push(Issue {
            severity: Severity::Error,
            section,
            message: message.into(),
            field: Some(field.into()),
        });
    }
    fn warn(&mut self, section: &'static str, message: impl Into<String>) {
        self.0.push(Issue {
            severity: Severity::Warning,
            section,
            message: message.into(),
            field: None,
        });
    }
}

pub fn validate(plan: &Plan) -> Vec<Issue> {
    let mut v = Issues(vec![]);
    // Ältere Säule 3a an der Hypothek wie beim Laden übernehmen (FEA-37).
    let plan = crate::upgrade::normalized(plan);
    check(&plan, &mut v);
    v.0
}

/// Parameter der Guardrails (Strategie «Guardrails» und 3-Topf mit Guardrails).
fn check_guardrails(v: &mut Issues, w: &Withdrawal) {
    let g = w.guardrails.clone().unwrap_or_default();
    if !(g.initial_rate > 0.0 && g.initial_rate <= 1.0) {
        v.err("withdrawal", "Die Startrate muss zwischen 0 % und 100 % liegen.");
    }
    if g.upper < 0.0 || g.lower < 0.0 || g.lower >= 1.0 {
        v.err(
            "withdrawal",
            "Die Leitplanken müssen grösser oder gleich 0 sein (unten kleiner als 100 %).",
        );
    }
    if !(0.0..1.0).contains(&g.adjust) {
        v.err("withdrawal", "Die Anpassung muss zwischen 0 % und 100 % liegen.");
    }
}

/// Unter- und Obergrenze der Lebenshaltung.
fn check_limits(v: &mut Issues, w: &Withdrawal) {
    if w.floor.is_some_and(|f| f < 0.0) || w.ceiling.is_some_and(|c| c < 0.0) {
        v.err(
            "withdrawal",
            "Unter- und Obergrenze der Ausgaben dürfen nicht negativ sein.",
        );
    }
    if let (Some(f), Some(c)) = (w.floor, w.ceiling) {
        if c > 0.0 && f > c {
            v.err("withdrawal", "Die Untergrenze der Ausgaben liegt über der Obergrenze.");
        }
    }
}

fn check(plan: &Plan, v: &mut Issues) {
    if plan.household.persons.is_empty() {
        v.err("general", "Mindestens eine Person ist nötig.");
        return;
    }
    match (plan.household.persons.len(), &plan.partner) {
        (n, _) if n > 2 => v.err("general", "Es sind höchstens zwei Personen möglich."),
        (1, Some(_)) => v.err(
            "general",
            "Die Angaben zur zweiten Person gehören zu einer zweiten Person im Haushalt.",
        ),
        (2, None) => v.warn(
            "general",
            "Die zweite Person hat weder Einkommen noch Vorsorge erfasst und bleibt unberücksichtigt.",
        ),
        _ => {}
    }
    for (i, p) in plan.household.persons.iter().enumerate() {
        let len = p.name.trim().chars().count();
        if len == 0 {
            v.err_at(
                "general",
                format!("household.persons.{i}.name"),
                format!("Person {}: Der Name fehlt (Pflichtangabe).", i + 1),
            );
        } else if len > MAX_NAME {
            v.err_at(
                "general",
                format!("household.persons.{i}.name"),
                format!(
                    "Person {}: Der Name darf höchstens {MAX_NAME} Zeichen lang sein.",
                    i + 1
                ),
            );
        }
    }
    if plan.household.ahv_couple_cap <= 0.0 {
        v.err("pension", "Die Plafonierung der AHV-Renten muss grösser als 0 sein.");
    }
    let r = match Resolver::new(plan) {
        Ok(r) => r,
        Err(e) => {
            v.err("milestones", e.to_string());
            return;
        }
    };
    let (start, end) = match (r.milestone("start"), r.milestone("end")) {
        (Ok(s), Ok(e)) => (s, e),
        _ => {
            v.err(
                "milestones",
                "Die Meilensteine «start» und «end» müssen vorhanden sein.",
            );
            return;
        }
    };
    if end <= start {
        v.err("milestones", "Das Simulationsende muss nach dem Start liegen.");
        return;
    }
    if start.until(end) > 12 * 100 {
        v.err("milestones", "Der Simulationszeitraum ist länger als 100 Jahre.");
    }
    for (i, p) in plan.household.persons.iter().enumerate() {
        let who = if p.name.trim().is_empty() {
            format!("Person {}", i + 1)
        } else {
            p.name.clone()
        };
        if p.birth > start {
            v.err_at(
                "general",
                format!("household.persons.{i}.birth"),
                format!(
                    "{who}: Das Geburtsdatum ({}) liegt nach dem Simulationsstart ({start}).",
                    p.birth
                ),
            );
        } else if p.birth.until(end) > 12 * MAX_AGE {
            v.err_at(
                "general",
                format!("household.persons.{i}.birth"),
                format!("{who}: Am Simulationsende wäre die Person über {MAX_AGE} Jahre alt."),
            );
        }
    }
    for ms in &plan.milestones {
        if let MilestoneKind::Age { years, .. } = &ms.kind {
            if !(0.0..=130.0).contains(years) {
                v.err("milestones", format!("«{}»: Alter {years} ist nicht möglich.", ms.name));
            }
        }
        // Start und Ende selbst liegen nie davor bzw. danach.
        let Ok(m) = r.milestone(&ms.id) else { continue };
        if m < start {
            v.warn(
                "milestones",
                format!("«{}» ({m}) liegt vor dem Simulationsstart.", ms.name),
            );
        } else if m > end {
            v.warn(
                "milestones",
                format!("«{}» ({m}) liegt nach dem Simulationsende.", ms.name),
            );
        }
    }

    let resolve = |v: &mut Issues, section: &'static str, what: &str, t: &TimeRef| -> Option<YearMonth> {
        match r.resolve(t) {
            Ok(m) => m,
            Err(e) => {
                v.err(section, format!("{what}: {e}"));
                None
            }
        }
    };
    let series = |v: &mut Issues, section: &'static str, what: &str, s: &Series, range: Option<(f64, f64)>| {
        let values: Vec<f64> = match s {
            Series::Number(x) | Series::Def(SeriesDef::Const { value: x }) => vec![*x],
            Series::Def(SeriesDef::Timed { rows }) => {
                if rows.is_empty() {
                    v.err(section, format!("{what}: keine Einträge."));
                }
                let mut first: Option<YearMonth> = None;
                for row in rows {
                    if let Some(m) = resolve(v, section, what, &row.from) {
                        first = Some(first.map_or(m, |f| f.min(m)));
                    }
                }
                if first.is_some_and(|f| f > start) {
                    v.warn(
                        section,
                        format!("{what}: kein Wert ab Simulationsstart – bis zum ersten Eintrag gilt 0."),
                    );
                }
                rows.iter().map(|r| r.value).collect()
            }
        };
        if let Some((lo, hi)) = range {
            if values.iter().any(|x| *x < lo || *x > hi) {
                v.err(section, format!("{what}: Werte müssen zwischen {lo} und {hi} liegen."));
            }
        }
    };

    // Vorsorge
    let pk = &plan.pension.pk;
    let retire = resolve(v, "pension", "Bezug Pensionskasse", &pk.at);
    resolve(v, "pension", "AHV-Bezug", &plan.pension.ahv.at);
    if !(0.0..=1.0).contains(&pk.lumpsum) {
        v.err("pension", "Kapitalbezug muss zwischen 0 % und 100 % liegen.");
    }
    if pk.conversion > 0.08 {
        v.warn(
            "pension",
            format!("Umwandlungssatz {:.2} % ist ungewöhnlich hoch.", pk.conversion * 100.0),
        );
    }
    series(v, "pension", "Beiträge Pensionskasse", &pk.contribution, None);
    series(
        v,
        "pension",
        "Verzinsung Pensionskasse",
        &pk.interest,
        Some((-0.5, 1.0)),
    );
    let mut retire_dates = vec![retire];
    if let Some(q) = &plan.partner {
        retire_dates.push(resolve(v, "pension", "Bezug Pensionskasse (Person 2)", &q.pk.at));
        resolve(v, "pension", "AHV-Bezug (Person 2)", &q.ahv.at);
        if !(0.0..=1.0).contains(&q.pk.lumpsum) {
            v.err("pension", "Person 2: Kapitalbezug muss zwischen 0 % und 100 % liegen.");
        }
        if q.pk.conversion > 0.08 {
            v.warn(
                "pension",
                format!(
                    "Person 2: Umwandlungssatz {:.2} % ist ungewöhnlich hoch.",
                    q.pk.conversion * 100.0
                ),
            );
        }
        series(
            v,
            "pension",
            "Beiträge Pensionskasse (Person 2)",
            &q.pk.contribution,
            None,
        );
        series(
            v,
            "pension",
            "Verzinsung Pensionskasse (Person 2)",
            &q.pk.interest,
            Some((-0.5, 1.0)),
        );
        series(v, "flows", "Einkommen (Person 2)", &q.income, None);
    }
    let retire_of = [retire_dates[0], retire_dates.get(1).copied().flatten()];
    let pk_lumpsum = [pk.lumpsum, plan.partner.as_ref().map_or(0.0, |q| q.pk.lumpsum)];
    let retire = retire_dates.iter().flatten().max().copied();

    // FEA-38: Feste Meilensteine je Person innerhalb der Grenzen aus den Einstellungen (FEA-24).
    let lim = plan.limits;
    for problem in lim.problems() {
        v.err("general", format!("Einstellungen: {problem}"));
    }
    for (i, p) in plan.household.persons.iter().take(2).enumerate() {
        let (pk_id, ahv_id) = person_milestones(i);
        for (id, base, lo, hi) in [
            (pk_id, PK_NAME, lim.pk_min, lim.pk_max),
            (ahv_id, AHV_NAME, lim.ahv_min, lim.ahv_max),
        ] {
            let Ok(m) = r.milestone(id) else { continue };
            let age = f64::from(p.birth.until(m)) / 12.0;
            if age < lo - 1e-9 || age > hi + 1e-9 {
                let name = person_milestone_name(base, i);
                let msg = format!(
                    "«{name}»: Alter {} liegt ausserhalb der Grenzen {lo} bis {hi} Jahre (Einstellungen).",
                    (age * 10.0).round() / 10.0
                );
                match plan.milestones.iter().position(|x| x.id == id) {
                    Some(k) => v.err_at("milestones", format!("milestones.{k}.years"), msg),
                    None => v.err("milestones", msg),
                }
            }
        }
        // FEA-38: Erwerbseinkommen nach dem Bezug der Pensionskasse (Warnung).
        let (income, indexed) = if i == 0 {
            (Some(&plan.flows.income), plan.flows.income_indexed)
        } else {
            (
                plan.partner.as_ref().map(|q| &q.income),
                plan.partner.as_ref().is_some_and(|q| q.income_indexed),
            )
        };
        // FEA-43: Mit der Teuerung steht der Betrag in Preisen des Starts (die Teuerung hängt vom Pfad ab).
        let prices = if indexed {
            " in Preisen des Simulationsstarts"
        } else {
            ""
        };
        if let (Some(income), Ok(pk_at)) = (income, r.milestone(pk_id)) {
            if let Ok(c) = Curve::compile(income, &r) {
                if let Some(x) = c.first_above(pk_at.max(start), end, 0.005) {
                    let who = if plan.household.persons.len() > 1 {
                        format!("{}: ", p.name)
                    } else {
                        String::new()
                    };
                    v.warn(
                        "flows",
                        format!(
                            "{who}Das Erwerbseinkommen beträgt nach dem Bezug der Pensionskasse ({pk_at}) noch {} pro Monat{prices}.",
                            chf(x)
                        ),
                    );
                }
            }
        }
    }

    // Geldflüsse und Vermögen
    series(v, "flows", "Einkommen", &plan.flows.income, None);
    series(v, "flows", "Ausgaben", &plan.flows.spending, None);
    // FEA-39: Die Lebenshaltungskosten enthalten die Wohnkosten; jede Zeile muss mindestens den Wohnanteil enthalten.
    if plan.flows.spending_includes_housing {
        let w = crate::housing::start_housing(plan, &r);
        let check = |v: &mut Issues, field: String, x: f64| {
            if x + 0.005 < w {
                v.err_at(
                    "flows",
                    field,
                    format!(
                        "Die Lebenshaltungskosten ({}) sind kleiner als die Wohnkosten zum Simulationsstart ({}), die sie enthalten.",
                        chf(x),
                        chf(w)
                    ),
                );
            }
        };
        match &plan.flows.spending {
            Series::Number(x) | Series::Def(SeriesDef::Const { value: x }) => {
                check(v, "flows.spending.value".into(), *x)
            }
            Series::Def(SeriesDef::Timed { rows }) => {
                for (i, row) in rows.iter().enumerate() {
                    check(v, format!("flows.spending.rows.{i}.value"), row.value);
                }
            }
        }
    }
    if let Some(s) = &plan.flows.savings {
        series(v, "flows", "Weitere Zuflüsse", s, None);
    }
    series(v, "wealth", "Aktienanteil", &plan.wealth.stock_share, Some((0.0, 1.0)));
    series(
        v,
        "wealth",
        "Cash-Reserve",
        &plan.wealth.reserve_months,
        Some((0.0, 1200.0)),
    );
    for x in &plan.flows.extras {
        if x.date > end {
            v.warn(
                "flows",
                format!("Einmaliger Betrag «{}» liegt nach dem Simulationsende.", x.label),
            );
        }
    }

    // Renditen
    let ret = &plan.returns;
    series(v, "returns", "Aktienrendite", &ret.fixed.stocks, Some((-0.99, 10.0)));
    series(
        v,
        "returns",
        "Obligationenrendite",
        &ret.fixed.bonds,
        Some((-0.99, 10.0)),
    );
    series(v, "returns", "Inflation", &ret.fixed.inflation, Some((-0.99, 10.0)));
    series(v, "returns", "SARON", &ret.fixed.saron, Some((-0.99, 10.0)));
    // FEA-53: Gewichte von «Welt individuell»
    if let Some(st) = ret
        .strategy
        .as_ref()
        .filter(|s| s.stocks == crate::model::StockIndex::WorldCustom)
    {
        if let Some(e) = crate::strategy::weight_issue(&st.weights) {
            v.err_at("wealth", "returns.strategy.weights", e);
        }
    }
    match ret.source {
        ReturnSource::Fixed => {}
        ReturnSource::Historical | ReturnSource::Rolling => {
            if ret.series.is_none() && ret.strategy.is_none() {
                v.err("returns", "Für historische Renditen muss eine Zeitreihe gewählt sein.");
            }
            if ret.source == ReturnSource::Historical && ret.start_date.is_none() {
                v.err("returns", "Der historische Startmonat fehlt.");
            }
        }
        ReturnSource::Bootstrap => {
            if ret.series.is_none() && ret.strategy.is_none() {
                v.err(
                    "returns",
                    "Für Monte Carlo aus der Historie muss eine Zeitreihe gewählt sein.",
                );
            }
            let b = ret.bootstrap.clone().unwrap_or_default();
            if b.paths == 0 || b.paths > 1_000_000 {
                v.err("returns", "Die Anzahl Pfade muss zwischen 1 und 1'000'000 liegen.");
            }
            if b.block < 1.0 {
                v.err("returns", "Die mittlere Blocklänge muss mindestens 1 Monat betragen.");
            }
        }
        ReturnSource::Parametric => {
            let q = ret.parametric.clone().unwrap_or_default();
            if q.paths == 0 || q.paths > 1_000_000 {
                v.err("returns", "Die Anzahl Pfade muss zwischen 1 und 1'000'000 liegen.");
            }
            if q.sigma < 0.0 || q.jump_sigma < 0.0 || q.jump_lambda < 0.0 {
                v.err("returns", "Volatilität und Sprungrate dürfen nicht negativ sein.");
            }
            if q.df != 0.0 && q.df <= 2.0 {
                v.err(
                    "returns",
                    "Die Freiheitsgrade müssen grösser als 2 sein (0 = Normalverteilung).",
                );
            }
        }
    }
    match plan.withdrawal.strategy {
        Strategy::Standard => {}
        Strategy::ThreeBuckets => {
            let b = plan.withdrawal.buckets.clone().unwrap_or_default();
            if b.cash_years < 0.0 || b.bond_years < 0.0 {
                v.err("withdrawal", "Die Töpfe dürfen nicht negativ sein.");
            }
            if b.cash_years + b.bond_years > 15.0 {
                v.warn(
                    "withdrawal",
                    "Topf 1 + Topf 2 decken mehr als 15 Jahre – wenig Aktienanteil übrig.",
                );
            }
            if retire.is_none() {
                v.warn(
                    "withdrawal",
                    "Die 3-Topf-Strategie beginnt mit der Pensionierung; ohne Bezugsdatum gilt durchgehend «Standard».",
                );
            }
            // FEA-05: Steuern die Guardrails die Ausgaben, gelten ihre Prüfungen und die der Grenzen.
            if plan.withdrawal.buckets_with_guardrails() {
                check_guardrails(v, &plan.withdrawal);
                check_limits(v, &plan.withdrawal);
            }
        }
        Strategy::FixedRate | Strategy::Guardrails | Strategy::Vpw | Strategy::Endowment => {
            let w = &plan.withdrawal;
            let rate_ok = |r: f64| r > 0.0 && r <= 1.0;
            if plan.withdrawal.strategy == Strategy::FixedRate {
                let r = w.fixed_rate.clone().unwrap_or_default().rate;
                if !rate_ok(r) {
                    v.err("withdrawal", "Die Entnahmerate muss zwischen 0 % und 100 % liegen.");
                } else if r > 0.07 {
                    v.warn(
                        "withdrawal",
                        format!(
                            "Eine Entnahmerate von {:.1} % ist hoch; das Vermögen schrumpft bei längerer Dauer rasch.",
                            r * 100.0
                        ),
                    );
                }
            } else if plan.withdrawal.strategy == Strategy::Vpw {
                let x = w.vpw.clone().unwrap_or_default();
                if x.horizon_age <= 0.0 || x.real_return <= -1.0 {
                    v.err("withdrawal", "VPW: Planungsalter und Realrendite sind ungültig.");
                }
            } else if plan.withdrawal.strategy == Strategy::Endowment {
                let x = w.endowment.clone().unwrap_or_default();
                if !rate_ok(x.rate) || !(0.0..=1.0).contains(&x.smoothing) {
                    v.err(
                        "withdrawal",
                        "Endowment: Rate und Glättung müssen zwischen 0 % und 100 % liegen.",
                    );
                }
            } else {
                check_guardrails(v, w);
            }
            check_limits(v, w);
            if retire.is_none() {
                v.warn("withdrawal", "Die Entnahmestrategie beginnt mit der Pensionierung; ohne Bezugsdatum gelten die geplanten Ausgaben durchgehend.");
            }
        }
    }

    // Szenarien
    match crate::scenarios::parse(&plan.scenarios.sweeps)
        .and_then(|sw| crate::scenarios::count_combinations(&sw).map(|_| sw))
    {
        Err(e) => v.err("scenarios", e.to_string()),
        Ok(sw) => {
            if sw.iter().any(|s| s.param == "wealth.stockShare") && plan.withdrawal.strategy == Strategy::ThreeBuckets {
                v.warn("scenarios", "Mit der 3-Topf-Strategie wirkt der Aktienanteil nur bis zur Pensionierung; danach bestimmen die Töpfe die Aufteilung.");
            }
            if sw.iter().any(|s| s.param == "returns.startDate") && ret.source != ReturnSource::Historical {
                v.warn(
                    "scenarios",
                    "Der historische Startmonat wirkt nur mit der Quelle «Historisch ab Startzeitpunkt».",
                );
            }
        }
    }

    // Gespeicherte Einstellungen der Optimierung
    if let Some(o) = &plan.optimization {
        if !(0.0..=1.0).contains(&o.max_ruin) {
            v.warn(
                "optimize",
                "Optimierung: Die höchste Ruinwahrscheinlichkeit muss zwischen 0 % und 100 % liegen.",
            );
        }
        if o.paths == 0 || o.check_paths == 0 {
            v.warn("optimize", "Optimierung: Die Anzahl Pfade muss grösser als 0 sein.");
        }
        for s in crate::optimize::all_strategies() {
            if let Err(e) = crate::optimize::split_defs(s, &o.ranges) {
                v.warn("optimize", format!("Optimierung: {e}"));
            }
        }
    }

    // ISS-18: Steuerfuss und Satztabellen (Einkommen, Vermögen, Kapitalbezug, auch Verheiratete). Ohne Prüfung ergab
    // ein Satz von −10¹⁵ ungültige Zahlen in der Simulation.
    let t = &plan.taxes;
    if !(0.0..=10.0).contains(&t.multiplier) {
        v.err_at(
            "taxes",
            "taxes.multiplier",
            "Steuerfuss: Der Faktor muss zwischen 0 und 10 liegen.",
        );
    }
    let mut tables: Vec<(&str, &str, &[(f64, f64)])> = vec![
        ("income", "Einkommenssteuer", &t.income),
        ("wealth", "Vermögenssteuer", &t.wealth),
        ("capital", "Kapitalbezugssteuer", &t.capital),
    ];
    if let Some(c) = &t.couple {
        tables.push(("couple.income", "Einkommenssteuer (Verheiratete)", &c.income));
        tables.push(("couple.wealth", "Vermögenssteuer (Verheiratete)", &c.wealth));
        tables.push(("couple.capital", "Kapitalbezugssteuer (Verheiratete)", &c.capital));
    }
    for (key, name, rows) in tables {
        for (i, &(from, rate)) in rows.iter().enumerate() {
            if !(from.is_finite() && from >= 0.0) {
                v.err_at(
                    "taxes",
                    format!("taxes.{key}.{i}.0"),
                    format!("{name}, Stufe {}: Der Betrag muss 0 oder grösser sein.", i + 1),
                );
            }
            if !(0.0..=1.0).contains(&rate) {
                v.err_at(
                    "taxes",
                    format!("taxes.{key}.{i}.1"),
                    format!(
                        "{name}, Stufe {}: Der Steuersatz muss zwischen 0 % und 100 % liegen.",
                        i + 1
                    ),
                );
            }
        }
    }

    // Grundstückgewinnsteuer ist Pflicht, sobald eine Immobilie mit Status «Besitz» oder «Kauf geplant» vorkommt
    // (Mietobjekte lösen sie nicht aus). Regeln mit Satz 0 % zählen als gesetzt.
    if plan.taxes.property_gain.is_none()
        && plan
            .realestate
            .properties
            .iter()
            .any(|p| p.status != PropertyStatus::Rent)
    {
        v.err(
            "taxes",
            "Grundstückgewinnsteuer fehlt: Die Planung enthält eine Immobilie im Besitz oder mit geplantem Kauf. Bitte unter «Steuern» die Vorlage Zürich, die Vorlage Aargau oder eigene Regeln (auch mit Satz 0 %) erfassen.",
        );
    }

    // Grundstückgewinnsteuer: Zu- und Abschläge nach Besitzdauer brauchen das Kaufdatum
    if let Some(g) = &plan.taxes.property_gain {
        if g.duration
            .iter()
            .any(|&(y, c)| !y.is_finite() || y < 0.0 || !c.is_finite() || c <= -1.0)
        {
            v.err("taxes", "Grundstückgewinnsteuer: Besitzdauer und Zu- oder Abschlag sind ungültig (Jahre ab 0, Abschlag kleiner als 100 %).");
        }
        if g.model == GainModel::Aargau {
            if g.duration_rates.is_empty() {
                v.err(
                    "taxes",
                    "Grundstückgewinnsteuer (Modell Aargau): Es ist kein Steuersatz nach Besitzdauer erfasst.",
                );
            }
            if g.duration_rates
                .iter()
                .any(|&(y, r)| !y.is_finite() || y < 0.0 || !r.is_finite() || !(0.0..=1.0).contains(&r))
            {
                v.err("taxes", "Grundstückgewinnsteuer (Modell Aargau): Besitzdauer und Steuersatz sind ungültig (Jahre ab 0, Satz zwischen 0 und 100 %).");
            }
        }
        if !g.duration.is_empty() || (g.model == GainModel::Aargau && !g.duration_rates.is_empty()) {
            for p in plan
                .realestate
                .properties
                .iter()
                .filter(|p| p.status == PropertyStatus::Owned && p.bought.is_none())
            {
                v.warn(
                    "realestate",
                    format!("{}: Das Kaufdatum fehlt. Ohne Kaufdatum gilt kein Zu- oder Abschlag (Modell Zürich) bzw. der höchste Satz (Modell Aargau) bei der Grundstückgewinnsteuer.", p.name),
                );
            }
        }
    }

    // Immobilien
    let mut indirect_per_person = [0.0f64; 2];
    series(
        v,
        "realestate",
        "Notverkauf-Schwelle",
        &plan.realestate.emergency_months,
        None,
    );
    // Verweise «nach Verkauf von …» prüfen.
    let mut ids = std::collections::HashSet::new();
    for p in &plan.realestate.properties {
        if let Some(id) = &p.id {
            if !ids.insert(id.as_str()) {
                v.err(
                    "realestate",
                    format!("{}: Kennung «{id}» ist doppelt vergeben.", p.name),
                );
            }
        }
    }
    for p in &plan.realestate.properties {
        for t in [&p.buy, &p.from].into_iter().flatten() {
            let Some(id) = t.sale_of() else { continue };
            match plan.realestate.properties.iter().find(|q| q.id.as_deref() == Some(id)) {
                None => v.err(
                    "realestate",
                    format!(
                        "{}: Die Immobilie, nach deren Verkauf gekauft bzw. gemietet wird, gibt es nicht mehr.",
                        p.name
                    ),
                ),
                Some(q) if std::ptr::eq(q, p) => v.err(
                    "realestate",
                    format!("{}: Kauf nach dem eigenen Verkauf ist nicht möglich.", p.name),
                ),
                Some(q) if q.status == PropertyStatus::Rent => v.err(
                    "realestate",
                    format!("{}: «{}» ist eine Miete und wird nie verkauft.", p.name, q.name),
                ),
                _ => {}
            }
        }
    }
    for (i, p) in plan.realestate.properties.iter().enumerate() {
        let n = &p.name;
        let buy = p.buy.as_ref().and_then(|t| resolve(v, "realestate", n, t));
        if let Some(t) = &p.sell {
            let sell = resolve(v, "realestate", n, t);
            if sell.is_some_and(|sell| sell < start) {
                v.warn(
                    "realestate",
                    format!("{n}: Verkaufszeitpunkt liegt vor dem Simulationsstart – das Objekt wird nicht zu diesem Zeitpunkt verkauft. Der Verkauf müsste im Simulationszeitraum liegen."),
                );
            }
            // Der Verkauf muss nach dem Kauf liegen (Besitz: Kaufdatum, Kauf geplant: «Kauf ab»). Ein Kauf «nach Verkauf von …»
            // ist beim Erfassen nicht bestimmbar (`buy` fehlt dann), dort entfällt die Prüfung.
            let bought = match p.status {
                PropertyStatus::Owned => p.bought,
                PropertyStatus::Planned => buy,
                PropertyStatus::Rent => None,
            };
            if let (Some(sell), Some(bought)) = (sell, bought) {
                if sell <= bought {
                    let what = if p.status == PropertyStatus::Owned {
                        "dem Kaufdatum"
                    } else {
                        "«Kauf ab»"
                    };
                    v.err_at(
                        "realestate",
                        format!("realestate.properties.{i}.sell"),
                        format!("{n}: Der Verkauf muss nach {what} liegen."),
                    );
                }
            }
        }
        match p.status {
            PropertyStatus::Rent => {
                match &p.from {
                    Some(t) => {
                        resolve(v, "realestate", n, t);
                    }
                    None => v.err("realestate", format!("{n}: «Miete ab» fehlt.")),
                }
                if let Some(t) = &p.until {
                    resolve(v, "realestate", n, t);
                }
                if p.rent.is_none() {
                    v.err("realestate", format!("{n}: Miete fehlt."));
                }
                continue;
            }
            PropertyStatus::Planned if buy.is_some_and(|b| b < start) => {
                v.warn(
                    "realestate",
                    format!("{n}: Kaufzeitpunkt liegt vor dem Simulationsstart – das Objekt wird nie gekauft. Bereits gekaufte Objekte als «Besitz» erfassen."),
                );
            }
            PropertyStatus::Planned if buy.is_none() && !p.buy.as_ref().is_some_and(TimeRef::is_dynamic) => {
                v.warn(
                    "realestate",
                    format!("{n}: kein Kaufzeitpunkt – das Objekt wird nie gekauft."),
                );
            }
            _ => {}
        }
        let Some(mg) = &p.mortgage else { continue };
        let total: f64 = mg.tranches.iter().map(|t| t.amount).sum();
        let max_loan = mg.amount_mode == AmountMode::MaxLoan;
        if max_loan && p.status != PropertyStatus::Planned {
            v.warn(
                "realestate",
                format!("{n}: «80 % des Kaufpreises» gilt nur für geplante Käufe; es zählen die erfassten Beträge."),
            );
        }
        // Bei «80 % des Kaufpreises» sind die Beträge nur Anteile und werden nicht mit dem Preis verglichen.
        if !max_loan && p.status == PropertyStatus::Planned {
            if total > p.price {
                v.err("realestate", format!("{n}: Hypothek ist höher als der Kaufpreis."));
            }
            if total > p.price * 0.8 {
                v.warn(
                    "realestate",
                    format!("{n}: Belehnung über 80 % – Banken verlangen meist 20 % Eigenmittel."),
                );
            }
        }
        for t in &mg.tranches {
            let tn = if mg.tranches.len() > 1 {
                format!("{n}, {}", t.name)
            } else {
                n.clone()
            };
            if t.periods.is_empty() {
                v.err("realestate", format!("{tn}: keine Hypothekarperiode erfasst."));
            }
            if p.status == PropertyStatus::Owned {
                match t.start {
                    None => v.err("realestate", format!("{tn}: Beginn der laufenden Periode fehlt.")),
                    Some(s) if s > start => {
                        v.warn("realestate", format!("{tn}: Beginn liegt nach dem Simulationsstart."))
                    }
                    _ => {}
                }
            }
            let mut m = if p.status == PropertyStatus::Owned {
                t.start
            } else {
                buy
            };
            // FEA-37: Konto der indirekten Amortisation; FEA-36: indirekt nur bis vor dem AHV-Bezug dieser Person.
            let account = match &t.pillar3a {
                None => Some(0),
                Some(id) => {
                    let i = plan.household.persons.iter().position(|q| &q.id == id);
                    if i.is_none() {
                        v.err(
                            "realestate",
                            format!("{tn}: Die Person «{id}» für die Säule 3a gibt es nicht."),
                        );
                    }
                    i
                }
            };
            let ahv_of = account.and_then(|i| r.milestone(person_milestones(i).1).ok());
            let mut max_q: f64 = 0.0;
            for (k, d) in t.periods.iter().enumerate() {
                if d.term > 100.0 {
                    v.err(
                        "realestate",
                        format!("{tn}, Periode {}: Laufzeit über 100 Jahre.", k + 1),
                    );
                }
                if d.term <= 0.0 {
                    v.err(
                        "realestate",
                        format!("{tn}, Periode {}: Laufzeit muss grösser als 0 sein.", k + 1),
                    );
                }
                if d.kind == RateKind::Fixed && d.rate > 0.1 {
                    v.warn(
                        "realestate",
                        format!(
                            "{tn}, Periode {}: Zins {:.2} % ist ungewöhnlich hoch.",
                            k + 1,
                            d.rate * 100.0
                        ),
                    );
                }
                if d.kind == RateKind::Saron && d.margin > 0.03 {
                    v.warn(
                        "realestate",
                        format!(
                            "{tn}, Periode {}: SARON-Marge {:.2} % ist ungewöhnlich hoch.",
                            k + 1,
                            d.margin * 100.0
                        ),
                    );
                }
                let period_end = m.map(|s| s.plus(((d.term * 12.0).round() as i32).max(1)));
                if d.amort_mode == AmortMode::Indirect
                    && d.amort > 0.0
                    && period_end.is_some_and(|e| e > start)
                    && !(m.is_some() && ahv_of.is_some() && m >= ahv_of)
                {
                    max_q = max_q.max(d.amort);
                }
                m = period_end;
            }
            if let Some(i) = account {
                indirect_per_person[i.min(1)] += max_q * 4.0;
            }
        }
    }

    // Säule 3a (FEA-37): Konten, Einzahlung, Auszahlungsplan; Maximalbetrag mit der indirekten Amortisation.
    let lim = plan.limits;
    for (k, acc) in plan.pillar3a.iter().enumerate() {
        let f = |name: &str| format!("pillar3a.{k}.{name}");
        let Some(i) = plan.household.persons.iter().position(|q| q.id == acc.person) else {
            v.err(
                "pillar3a",
                format!("Säule 3a: Die Person «{}» gibt es nicht.", acc.person),
            );
            continue;
        };
        let who = if plan.household.persons.len() > 1 {
            format!("Säule 3a {}", plan.household.persons[i].name)
        } else {
            "Säule 3a".to_string()
        };
        if plan.pillar3a[..k].iter().any(|a| a.person == acc.person) {
            v.err("pillar3a", format!("{who}: Pro Person ist nur ein 3a-Konto möglich."));
        }
        if acc.balance < 0.0 {
            v.err_at(
                "pillar3a",
                f("balance"),
                format!("{who}: Das Guthaben darf nicht negativ sein."),
            );
        }
        if !(-0.5..=1.0).contains(&acc.rate) {
            v.err_at(
                "pillar3a",
                f("rate"),
                format!("{who}: Der Zins muss zwischen −50 % und 100 % liegen."),
            );
        }
        if !(acc.contribution >= 0.0 && acc.contribution <= lim.pillar3a_max) {
            v.err_at(
                "pillar3a",
                f("contribution"),
                format!("{who}: Die Einzahlung muss zwischen 0 und dem Maximalbetrag von {} pro Jahr liegen (Einstellungen).", chf(lim.pillar3a_max)),
            );
        }
        let ahv = r.milestone(person_milestones(i).1).ok();
        let indirect = indirect_per_person[i.min(1)];
        if acc.contribution + indirect > lim.pillar3a_max + 0.005 && lim.pillar3a_max >= 0.0 {
            v.warn(
                "pillar3a",
                format!(
                    "{who}: Einzahlung ({}) und indirekte Amortisation ({}) übersteigen zusammen den Maximalbetrag von {} pro Jahr.",
                    chf(acc.contribution),
                    chf(indirect),
                    chf(lim.pillar3a_max)
                ),
            );
        }
        let repay_possible = indirect > 0.0;
        // Ohne Tranche mit indirekter Amortisation gibt es nichts zurückzuzahlen: Die Auszahlung geht ins Vermögen.
        if acc.usage == PayoutUse::Repay && !repay_possible {
            v.warn("pillar3a", format!("{who}: «Hypothek zurückzahlen» setzt eine Tranche mit indirekter Amortisation voraus; der Rest geht ins freie Vermögen."));
        }
        for (j, x) in acc.payouts.iter().enumerate() {
            // FEA-41: Zeitpunkt als Datum oder Alter; die Meldung steht am jeweiligen Feld.
            let at_field = match x.kind {
                PayoutAt::Date => format!("pillar3a.{k}.payouts.{j}.date"),
                PayoutAt::Age => format!("pillar3a.{k}.payouts.{j}.years"),
            };
            let Some(date) = x.month(Some(plan.household.persons[i].birth)) else {
                let what = match x.kind {
                    PayoutAt::Date => "Das Datum",
                    PayoutAt::Age => "Das Alter (0 bis 150 Jahre)",
                };
                v.err_at(
                    "pillar3a",
                    at_field,
                    format!("{who}: {what} der {}. Auszahlung fehlt oder ist ungültig.", j + 1),
                );
                continue;
            };
            if let Some(a) = ahv {
                let earliest = a.plus(-60);
                if date < earliest {
                    v.err_at(
                        "pillar3a",
                        at_field,
                        format!("{who}: Die Auszahlung im {date} liegt mehr als 5 Jahre vor dem AHV-Bezug ({a}); frühestens im {earliest}."),
                    );
                } else if date > a {
                    v.warn("pillar3a", format!("{who}: Die Auszahlung im {date} liegt nach dem AHV-Bezug ({a}); das Guthaben ist dann schon ausbezahlt."));
                }
            }
            if x.amount.is_some_and(|b| !(b > 0.0)) {
                v.err_at(
                    "pillar3a",
                    format!("pillar3a.{k}.payouts.{j}.amount"),
                    format!("{who}: Der Betrag der Auszahlung im {} muss grösser als 0 sein.", date),
                );
            }
            if x.usage == PayoutUse::Repay && !repay_possible {
                v.warn("pillar3a", format!("{who}: «Hypothek zurückzahlen» (Auszahlung im {}) setzt eine Tranche mit indirekter Amortisation voraus; sie geht ins freie Vermögen.", date));
            }
            if let (Some(pm), true) = (retire_of[i.min(1)], pk_lumpsum[i.min(1)] > 0.0) {
                if pm.year() == date.year() {
                    v.warn("pillar3a", format!("{who}: Auszahlung im {} und PK-Kapitalbezug im selben Jahr – sie werden für die Steuerprogression zusammengerechnet.", date));
                }
            }
        }
    }
    // Indirekte Amortisation ohne erfasstes 3a-Konto: Maximalbetrag trotzdem prüfen.
    for (i, p) in plan.household.persons.iter().take(2).enumerate() {
        if plan.pillar3a.iter().any(|a| a.person == p.id) {
            continue;
        }
        let indirect = indirect_per_person[i];
        if indirect > lim.pillar3a_max + 0.005 {
            let who = if plan.household.persons.len() > 1 {
                format!("Säule 3a {}", p.name)
            } else {
                "Säule 3a".to_string()
            };
            v.warn(
                "pillar3a",
                format!(
                    "{who}: Einzahlung (0) und indirekte Amortisation ({}) übersteigen zusammen den Maximalbetrag von {} pro Jahr.",
                    chf(indirect),
                    chf(lim.pillar3a_max)
                ),
            );
        }
    }
}

/// Betrag mit Tausendertrennzeichen («12'500»).
fn chf(v: f64) -> String {
    let n = v.round() as i64;
    let digits = n.unsigned_abs().to_string();
    let mut out = String::new();
    for (i, c) in digits.chars().enumerate() {
        if i > 0 && (digits.len() - i) % 3 == 0 {
            out.push('\'');
        }
        out.push(c);
    }
    if n < 0 {
        format!("-{out}")
    } else {
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};

    // Beispielplanung: Start 2026-11, Geburt 1966-11, Pensionierung und AHV 2031-11, Ende 2061-11.
    fn base() -> Value {
        serde_json::from_str(include_str!("../assets/Neu.vplan")).unwrap()
    }
    fn messages(v: &Value) -> Vec<String> {
        let plan: Plan = serde_json::from_value(v.clone()).unwrap();
        validate(&plan).into_iter().map(|i| i.message).collect()
    }
    /// Ob eine Meldung den Text enthält.
    fn has(v: &Value, text: &str) -> bool {
        messages(v).iter().any(|m| m.contains(text))
    }
    fn with(f: impl FnOnce(&mut Value)) -> Value {
        let mut v = base();
        f(&mut v);
        v
    }
    fn set_end(v: &mut Value, date: &str) {
        v["milestones"][3] =
            json!({"id": "end", "kind": "date", "name": "Simulationsende", "date": date, "fixed": true});
    }

    // FEA-29: Grenzwerte der Prüfungen (der Grenzwert selbst ist jeweils erlaubt).
    #[test]
    fn period_birth_and_milestones() {
        assert!(messages(&base()).is_empty(), "{:?}", messages(&base()));
        const LONG: &str = "länger als 100 Jahre";
        assert!(!has(&with(|v| set_end(v, "2126-11")), LONG));
        assert!(has(&with(|v| set_end(v, "2126-12")), LONG));
        // Geburt im Startmonat ist möglich
        assert!(!has(
            &with(|v| v["household"]["persons"][0]["birth"] = json!("2026-11")),
            "nach dem Simulationsstart"
        ));
        assert!(has(
            &with(|v| v["household"]["persons"][0]["birth"] = json!("2026-12")),
            "nach dem Simulationsstart"
        ));
        // genau 120 Jahre am Ende
        const OLD: &str = "über 120 Jahre alt";
        assert!(!has(&with(|v| set_end(v, "2086-11")), OLD));
        assert!(has(&with(|v| set_end(v, "2086-12")), OLD));
        // Meilensteine vor dem Start bzw. nach dem Ende; das Ende selbst ist erlaubt
        let ms =
            |d: &str| with(|v| v["milestones"][2] = json!({"id": "ahv", "kind": "date", "name": "AHV", "date": d}));
        assert!(has(&ms("2026-10"), "liegt vor dem Simulationsstart"));
        assert!(!has(&ms("2061-11"), "liegt nach dem Simulationsende"));
        assert!(has(&ms("2061-12"), "liegt nach dem Simulationsende"));
    }

    #[test]
    fn pension_series_and_extras() {
        const HIGH: &str = "Umwandlungssatz 9.00 %";
        assert!(!has(
            &with(|v| v["pension"]["pk"]["conversion"] = json!(0.08)),
            "ungewöhnlich hoch"
        ));
        assert!(has(&with(|v| v["pension"]["pk"]["conversion"] = json!(0.09)), HIGH));
        let partner = |c: f64| {
            with(|v| {
                v["household"]["persons"] =
                    json!([{"id": "A", "name": "A", "birth": "1966-11"}, {"id": "B", "name": "B", "birth": "1968-01"}]);
                v["partner"] = json!({"income": 1000, "pk": {"capital": 0, "at": "retire", "lumpsum": 0, "conversion": c},
                                      "ahv": {"monthly": 0, "at": "ahv", "indexed": true}});
            })
        };
        assert!(!has(&partner(0.08), "ungewöhnlich hoch"));
        assert!(has(&partner(0.09), "Person 2: Umwandlungssatz 9.00 %"));
        // Zeitreihe erst nach dem Start: Hinweis; ab Start: keiner
        const LATE: &str = "kein Wert ab Simulationsstart";
        let income = |from: &str| {
            with(|v| v["flows"]["income"] = json!({"mode": "timed", "rows": [{"from": from, "value": 1}]}))
        };
        assert!(has(&income("2027-01"), LATE));
        assert!(!has(&income("2026-11"), LATE));
        assert!(!has(&income("2020-01"), LATE));
        let extra = |d: &str| with(|v| v["flows"]["extras"] = json!([{"date": d, "amount": 1000, "label": "Erbe"}]));
        assert!(!has(&extra("2061-11"), "«Erbe» liegt nach"));
        assert!(has(&extra("2061-12"), "«Erbe» liegt nach"));
    }

    #[test]
    fn return_sources() {
        let boot = |paths: u64, block: f64| {
            with(|v| {
                v["returns"]["source"] = json!("bootstrap");
                v["returns"]["series"] = json!("x.csv");
                v["returns"]["bootstrap"] = json!({"paths": paths, "block": block, "seed": 1});
            })
        };
        assert!(
            messages(&boot(1_000_000, 1.0)).is_empty(),
            "{:?}",
            messages(&boot(1_000_000, 1.0))
        );
        assert!(has(&boot(1_000_001, 1.0), "Anzahl Pfade"));
        assert!(has(&boot(10, 0.9), "Blocklänge"));
        let para = |sigma: f64, js: f64, jl: f64| {
            with(|v| {
                v["returns"]["source"] = json!("parametric");
                v["returns"]["parametric"] =
                    json!({"paths": 10, "mu": 0.05, "sigma": sigma, "df": 0, "jumpSigma": js, "jumpLambda": jl});
            })
        };
        const NEG: &str = "dürfen nicht negativ sein";
        assert!(!has(&para(0.0, 0.0, 0.0), NEG));
        assert!(has(&para(-0.1, 0.0, 0.0), NEG));
        assert!(has(&para(0.1, -0.1, 0.0), NEG));
        assert!(has(&para(0.1, 0.0, -0.1), NEG));
    }

    #[test]
    fn withdrawal_strategies() {
        let w = |strategy: &str, f: &dyn Fn(&mut Value)| {
            with(|v| {
                v["withdrawal"]["strategy"] = json!(strategy);
                f(&mut v["withdrawal"]);
            })
        };
        const MANY: &str = "mehr als 15 Jahre";
        assert!(!has(
            &w("three_buckets", &|x| x["buckets"] =
                json!({"cashYears": 10, "bondYears": 5})),
            MANY
        ));
        assert!(has(
            &w("three_buckets", &|x| x["buckets"] =
                json!({"cashYears": 10, "bondYears": 5.5})),
            MANY
        ));
        assert!(!has(
            &w("fixed_rate", &|x| x["fixedRate"] = json!({"rate": 0.07})),
            "ist hoch"
        ));
        assert!(has(
            &w("fixed_rate", &|x| x["fixedRate"] = json!({"rate": 0.08})),
            "Entnahmerate von 8.0 % ist hoch"
        ));
        const VPW: &str = "VPW:";
        assert!(!has(&w("vpw", &|_| {}), VPW));
        assert!(!has(&w("vpw", &|x| x["vpw"]["realReturn"] = json!(0.5)), VPW));
        assert!(has(&w("vpw", &|x| x["vpw"]["realReturn"] = json!(-1.0)), VPW));
        assert!(has(&w("vpw", &|x| x["vpw"]["horizonAge"] = json!(0)), VPW));
        const END: &str = "Endowment:";
        assert!(!has(&w("endowment", &|_| {}), END));
        assert!(has(&w("endowment", &|x| x["endowment"]["smoothing"] = json!(1.5)), END));
        assert!(has(&w("endowment", &|x| x["endowment"]["rate"] = json!(0)), END));
        const RAILS: &str = "Leitplanken";
        let g = |upper: f64, lower: f64| {
            w("guardrails", &move |x| {
                x["guardrails"]["upper"] = json!(upper);
                x["guardrails"]["lower"] = json!(lower);
            })
        };
        assert!(!has(&g(0.0, 0.0), RAILS));
        assert!(has(&g(-0.1, 0.2), RAILS));
        assert!(has(&g(0.2, -0.1), RAILS));
        assert!(has(&g(0.2, 1.0), RAILS));
        assert!(!has(&g(0.2, 0.99), RAILS));
        const ABOVE: &str = "Untergrenze der Ausgaben liegt über";
        let fc = |floor: f64, ceiling: f64| {
            w("fixed_rate", &move |x| {
                x["floor"] = json!(floor);
                x["ceiling"] = json!(ceiling);
            })
        };
        assert!(has(&fc(5000.0, 4000.0), ABOVE));
        assert!(!has(&fc(4000.0, 4000.0), ABOVE));
        assert!(!has(&fc(5000.0, 0.0), ABOVE));
    }

    #[test]
    fn scenarios_and_optimization() {
        const START: &str = "wirkt nur mit der Quelle «Historisch";
        let sweep = |source: &str| {
            with(|v| {
                v["returns"]["source"] = json!(source);
                v["returns"]["series"] = json!("x.csv");
                v["returns"]["startDate"] = json!("1990-01");
                v["scenarios"]["sweeps"] =
                    json!([{"param": "returns.startDate", "from": "1990-01", "to": "1991-01", "step": 12}]);
            })
        };
        assert!(has(&sweep("rolling"), START));
        assert!(!has(&sweep("historical"), START));
        const PATHS: &str = "Anzahl Pfade muss grösser als 0";
        let opt = |paths: u64, check: u64| with(|v| v["optimization"] = json!({"paths": paths, "checkPaths": check}));
        assert!(!has(&opt(100, 100), PATHS));
        assert!(has(&opt(0, 100), PATHS));
        assert!(has(&opt(100, 0), PATHS));
    }

    fn house(status: &str, extra: Value) -> Value {
        let mut p =
            json!({"id": "h", "name": "Haus", "status": status, "price": 1_000_000, "fixCosts": 0, "inflation": false});
        for (k, x) in extra.as_object().unwrap() {
            p[k] = x.clone();
        }
        p
    }
    fn with_props(props: Vec<Value>) -> Value {
        with(|v| {
            v["realestate"]["properties"] = Value::Array(props);
            v["taxes"]["propertyGain"] = json!({"brackets": [[0, 0.2]]});
        })
    }
    fn tranche(amount: f64, start: Option<&str>, periods: Value) -> Value {
        let mut t = json!({"name": "T1", "amount": amount, "periods": periods});
        if let Some(s) = start {
            t["start"] = json!(s);
        }
        t
    }

    #[test]
    fn property_gain_tax_rules() {
        let aargau = |rates: Value, model: &str| {
            with(|v| {
                v["realestate"]["properties"] = json!([house("owned", json!({"sell": "never", "worth": 1_000_000}))]);
                v["taxes"]["propertyGain"] = json!({"model": model, "durationRates": rates, "brackets": [[0, 0.2]]});
            })
        };
        assert!(has(
            &aargau(json!([[0, 1.5]]), "aargau"),
            "Besitzdauer und Steuersatz sind ungültig"
        ));
        assert!(has(
            &aargau(json!([[-1, 0.2]]), "aargau"),
            "Besitzdauer und Steuersatz sind ungültig"
        ));
        assert!(!has(&aargau(json!([[0, 0.2]]), "aargau"), "sind ungültig"));
        // Kaufdatum fehlt: Hinweis nur im Modell Aargau mit Sätzen (bzw. mit Zu-/Abschlägen)
        const BOUGHT: &str = "Das Kaufdatum fehlt";
        assert!(has(&aargau(json!([[0, 0.2]]), "aargau"), BOUGHT));
        assert!(!has(&aargau(json!([[0, 0.2]]), "zurich"), BOUGHT));
    }

    #[test]
    fn property_references_and_dates() {
        let own = |buy: &str| {
            with_props(vec![
                house("planned", json!({"buy": buy})),
                json!({"id": "m", "name": "Miete", "status": "rent", "price": 0, "fixCosts": 0, "inflation": false, "from": "start", "rent": 2000}),
            ])
        };
        assert!(has(&own("afterSale:h"), "Kauf nach dem eigenen Verkauf"));
        assert!(has(&own("afterSale:m"), "«Miete» ist eine Miete"));
        assert!(!has(&own("2030-01"), "Miete und wird nie"));
        const EARLY_SELL: &str = "Verkaufszeitpunkt liegt vor dem Simulationsstart";
        let sell = |d: &str| with_props(vec![house("owned", json!({"sell": d, "worth": 1}))]);
        assert!(has(&sell("2026-10"), EARLY_SELL));
        assert!(!has(&sell("2026-11"), EARLY_SELL));
        const EARLY_BUY: &str = "Kaufzeitpunkt liegt vor dem Simulationsstart";
        let buy = |d: &str| with_props(vec![house("planned", json!({"buy": d}))]);
        assert!(has(&buy("2026-10"), EARLY_BUY));
        assert!(!has(&buy("2026-11"), EARLY_BUY));
        assert!(!has(&buy("2030-01"), "nie gekauft"));
        const NO_BUY: &str = "kein Kaufzeitpunkt";
        assert!(has(&buy("never"), NO_BUY));
        assert!(has(&with_props(vec![house("planned", json!({}))]), NO_BUY));
        assert!(!has(&buy("afterSale"), NO_BUY));
    }

    #[test]
    fn mortgage_amounts() {
        let fixed = json!([{"type": "fixed", "term": 10, "rate": 0.02}]);
        let planned = |amount: f64, mode: &str| {
            with_props(vec![house(
                "planned",
                json!({"buy": "2030-01",
                "mortgage": {"amountMode": mode, "tranches": [tranche(amount, None, fixed.clone())]}}),
            )])
        };
        const HIGHER: &str = "Hypothek ist höher als der Kaufpreis";
        const LTV: &str = "Belehnung über 80 %";
        assert!(has(&planned(1_000_001.0, "fixed"), HIGHER));
        assert!(!has(&planned(1_000_000.0, "fixed"), HIGHER));
        assert!(has(&planned(800_001.0, "fixed"), LTV));
        assert!(!has(&planned(800_000.0, "fixed"), LTV));
        assert!(!has(&planned(0.9, "maxLoan"), LTV));
        assert!(!has(&planned(0.9, "maxLoan"), "gilt nur für geplante Käufe"));
        let owned = |mode: &str, tranches: Value| {
            with_props(vec![house(
                "owned",
                json!({"sell": "never", "worth": 1,
                "mortgage": {"amountMode": mode, "tranches": tranches}}),
            )])
        };
        assert!(has(
            &owned("maxLoan", json!([tranche(1.0, Some("2026-01"), fixed.clone())])),
            "gilt nur für geplante Käufe"
        ));
        // Preis und Belehnung werden nur bei geplanten Käufen geprüft.
        let high = owned("fixed", json!([tranche(1_200_000.0, Some("2026-01"), fixed.clone())]));
        assert!(!has(&high, LTV) && !has(&high, HIGHER));
        // Beginn der Periode bei Besitz
        assert!(has(
            &owned("fixed", json!([tranche(1.0, None, fixed.clone())])),
            "Haus: Beginn der laufenden Periode fehlt"
        ));
        assert!(has(
            &owned("fixed", json!([tranche(1.0, Some("2026-12"), fixed.clone())])),
            "Beginn liegt nach dem Simulationsstart"
        ));
        assert!(!has(
            &owned("fixed", json!([tranche(1.0, Some("2026-11"), fixed.clone())])),
            "Beginn liegt nach"
        ));
        // Bei mehreren Tranchen steht der Name der Tranche in der Meldung
        let mut t2 = tranche(1.0, None, fixed.clone());
        t2["name"] = json!("T2");
        assert!(has(
            &owned("fixed", json!([tranche(1.0, Some("2026-01"), fixed.clone()), t2])),
            "Haus, T2: Beginn der laufenden Periode fehlt"
        ));
    }

    #[test]
    fn mortgage_periods() {
        let owned = |periods: Value| {
            with_props(vec![house(
                "owned",
                json!({"sell": "never", "worth": 1,
                "mortgage": {"tranches": [tranche(100_000.0, Some("2026-01"), periods)]}}),
            )])
        };
        assert!(!has(
            &owned(json!([{"type": "fixed", "term": 100, "rate": 0.02}])),
            "über 100 Jahre"
        ));
        assert!(has(
            &owned(json!([{"type": "fixed", "term": 101, "rate": 0.02}])),
            "Periode 1: Laufzeit über 100 Jahre"
        ));
        assert!(!has(
            &owned(json!([{"type": "fixed", "term": 5, "rate": 0.1}])),
            "ungewöhnlich hoch"
        ));
        assert!(has(
            &owned(json!([{"type": "fixed", "term": 5, "rate": 0.11}])),
            "Zins 11.00 % ist ungewöhnlich hoch"
        ));
        assert!(!has(
            &owned(json!([{"type": "saron", "term": 5, "rate": 0.2, "margin": 0.03}])),
            "ungewöhnlich hoch"
        ));
        assert!(has(
            &owned(json!([{"type": "saron", "term": 5, "margin": 0.031}])),
            "SARON-Marge 3.10 %"
        ));
        assert!(!has(
            &owned(json!([{"type": "fixed", "term": 5, "rate": 0.2, "margin": 0.05}])),
            "SARON-Marge"
        ));

        // FEA-36/FEA-37: Indirekte Amortisation zählt bis vor dem AHV-Bezug (2031-11) zum 3a-Maximalbetrag.
        let ind = |term: f64, amort: f64, mode: &str| {
            owned(json!([{"type": "fixed", "term": term, "rate": 0.01, "amort": amort, "amortMode": mode}]))
        };
        const MAX: &str = "übersteigen zusammen den Maximalbetrag";
        // 3a-Maximalbetrag 7258 pro Jahr = 1814.5 pro Quartal
        assert!(!has(&ind(3.0, 1814.5, "indirect"), MAX));
        assert!(has(
            &ind(3.0, 1815.0, "indirect"),
            "indirekte Amortisation (7'260) übersteigen zusammen den Maximalbetrag von 7'258"
        ));
        assert!(!has(&ind(3.0, 1815.0, "direct"), MAX));
        // Periode vor dem Start beendet: zählt nicht
        let ended = with_props(vec![house(
            "owned",
            json!({"sell": "never", "worth": 1,
            "mortgage": {"tranches": [tranche(1.0, Some("2010-01"), json!([{"type": "fixed", "term": 5, "rate": 0.01, "amort": 5000, "amortMode": "indirect"}]))]}}),
        )]);
        assert!(!has(&ended, MAX));
        // Grenze: Eine Periode, die genau im Startmonat 2026-11 endet, zählt nicht mehr, einen Monat später schon.
        let ends = |from: &str| {
            with_props(vec![house(
                "owned",
                json!({"sell": "never", "worth": 1,
            "mortgage": {"tranches": [tranche(1.0, Some(from), json!([{"type": "fixed", "term": 5, "rate": 0.01, "amort": 5000, "amortMode": "indirect"}]))]}}),
            )])
        };
        assert!(!has(&ends("2021-11"), MAX));
        assert!(has(&ends("2021-12"), MAX));
        // FEA-36: Eine Periode ab dem AHV-Bezug ist direkt und zählt nicht; einen Monat davor schon.
        assert!(!has(&ends("2031-11"), MAX));
        assert!(has(&ends("2031-10"), MAX));
    }

    /// Planung mit 3a-Konto der Person (Beispiel «Neu»: Start 2026-11, Geburt 1966-11, AHV-Bezug 2031-11).
    fn with_3a(account: Value) -> Value {
        with(|v| v["pillar3a"] = json!([account]))
    }

    // FEA-37/AK4, AK5: Auszahlungsplan frühestens 5 Jahre vor dem AHV-Bezug, Betrag grösser als 0.
    #[test]
    fn pillar3a_payout_plan() {
        let pay = |date: &str| {
            with_3a(
                json!({"person": "A", "balance": 10_000, "rate": 0.01, "payouts": [{"date": date, "amount": 5000}]}),
            )
        };
        const EARLY: &str = "liegt mehr als 5 Jahre vor dem AHV-Bezug (11.2031); frühestens im 11.2026";
        assert!(!has(&pay("2026-11"), EARLY), "genau 5 Jahre vorher ist gültig");
        assert!(has(&pay("2026-10"), EARLY));
        assert!(messages(&pay("2026-10"))
            .iter()
            .any(|m| m.contains("Säule 3a: Die Auszahlung im 10.2026")));
        assert!(has(&pay("2031-12"), "liegt nach dem AHV-Bezug"));
        assert!(!has(&pay("2031-11"), "liegt nach dem AHV-Bezug"));
        let zero =
            with_3a(json!({"person": "A", "balance": 1, "rate": 0, "payouts": [{"date": "2030-01", "amount": 0}]}));
        assert!(has(&zero, "muss grösser als 0 sein"));
        let whole = with_3a(json!({"person": "A", "balance": 1, "rate": 0, "payouts": [{"date": "2030-01"}]}));
        assert!(!has(&whole, "muss grösser als 0 sein"), "ohne Betrag: ganzes Guthaben");
    }

    // FEA-37: Konto, Zins, Verwendung, PK-Kapitalbezug im selben Jahr.
    #[test]
    fn pillar3a_account() {
        assert!(has(
            &with_3a(json!({"person": "X", "balance": 1, "rate": 0})),
            "Die Person «X» gibt es nicht"
        ));
        assert!(has(
            &with(|v| v["pillar3a"] = json!([{"person": "A"}, {"person": "A"}])),
            "nur ein 3a-Konto"
        ));
        assert!(has(
            &with_3a(json!({"person": "A", "balance": -1})),
            "Guthaben darf nicht negativ"
        ));
        assert!(has(&with_3a(json!({"person": "A", "rate": 1.5})), "Zins muss zwischen"));
        assert!(has(
            &with_3a(json!({"person": "A", "use": "repay"})),
            "setzt eine Tranche mit indirekter Amortisation voraus"
        ));
        assert!(
            !has(&with_3a(json!({"person": "A"})), "setzt eine Tranche"),
            "Vorgabe: ins freie Vermögen"
        );
        let indirect = |use_: &str| {
            let mut v = with_props(vec![house(
                "owned",
                json!({"sell": "never", "worth": 1,
                "mortgage": {"tranches": [tranche(100_000.0, Some("2026-01"),
                    json!([{"type": "fixed", "term": 3, "rate": 0.01, "amort": 1000, "amortMode": "indirect"}]))]}}),
            )]);
            v["pillar3a"] = json!([{"person": "A", "use": use_, "payouts": [{"date": "2030-01", "use": use_}]}]);
            v
        };
        assert!(!has(&indirect("repay"), "setzt eine Tranche"));
        let same = |date: &str, lumpsum: f64| {
            let mut v = with_3a(json!({"person": "A", "payouts": [{"date": date}]}));
            v["pension"]["pk"]["lumpsum"] = json!(lumpsum);
            v
        };
        const SAME: &str = "im selben Jahr";
        assert!(has(&same("2031-03", 0.5), SAME));
        assert!(!has(&same("2031-03", 0.0), SAME));
        assert!(!has(&same("2030-12", 0.5), SAME));
    }

    // FEA-37/AK: Einzahlung zwischen 0 und dem Maximalbetrag (Einstellungen), mit der indirekten Amortisation zusammen.
    #[test]
    fn pillar3a_contribution() {
        let c = |x: f64| with_3a(json!({"person": "A", "contribution": x}));
        const RANGE: &str = "Die Einzahlung muss zwischen 0 und dem Maximalbetrag von 7'258 pro Jahr liegen";
        assert!(!has(&c(0.0), RANGE));
        assert!(!has(&c(7258.0), RANGE));
        assert!(has(&c(7258.5), RANGE));
        assert!(has(&c(-1.0), RANGE));
        // Maximalbetrag aus den Einstellungen (FEA-24)
        let mut higher = c(7500.0);
        higher["limits"] = json!({"pillar3aMax": 8000});
        assert!(!has(&higher, "Die Einzahlung muss"));
        // Einzahlung und indirekte Amortisation zusammen: Warnung, keine Sperre
        let mut both = with_props(vec![house(
            "owned",
            json!({"sell": "never", "worth": 1,
            "mortgage": {"tranches": [tranche(100_000.0, Some("2026-01"),
                json!([{"type": "fixed", "term": 3, "rate": 0.01, "amort": 1000, "amortMode": "indirect"}]))]}}),
        )]);
        both["pillar3a"] = json!([{"person": "A", "contribution": 3258}]);
        assert!(!has(&both, "übersteigen zusammen"));
        both["pillar3a"][0]["contribution"] = json!(3259);
        let w = Plan::from_json(&both.to_string()).unwrap();
        let hit = validate(&w)
            .into_iter()
            .find(|i| i.message.contains("übersteigen zusammen"))
            .unwrap();
        assert_eq!(hit.severity, Severity::Warning);
        assert!(hit
            .message
            .contains("Einzahlung (3'259) und indirekte Amortisation (4'000)"));
    }
}
