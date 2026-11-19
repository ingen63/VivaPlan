//! Reference computation «Test Retirement.ods» (old program as a spreadsheet, golden reference).
//! Sheet 1: returns 0 %, sheet 2: 5 % return (halved from the event «Derisk») and 10 % PK interest.
//! Row r of the spreadsheet corresponds to month r − 3 from January 2030; column E is the wealth.
//! The expectations are read from the computed values of the spreadsheet.

use std::path::{Path, PathBuf};

use calamine::{open_workbook_auto, Data, Reader};
use engine::batch;
use engine::model::Plan;
use engine::sim::Simulator;
use serde_json::{json, Value};

fn ods() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/golden/Test Retirement.ods")
}

struct Sheet(Vec<Vec<Data>>);

impl Sheet {
    fn load(name: &str) -> Sheet {
        let mut wb = open_workbook_auto(ods()).expect("ODS lesbar");
        let range = wb.worksheet_range(name).expect("Blatt vorhanden");
        assert_eq!(range.start(), Some((0, 0)));
        Sheet(range.rows().map(|r| r.to_vec()).collect())
    }
    /// Cell in spreadsheet notation: column A = 0, row 1 = first row.
    fn num(&self, col: char, row: usize) -> f64 {
        match self.0.get(row - 1).and_then(|r| r.get(col as usize - 'A' as usize)) {
            Some(Data::Float(f)) => *f,
            Some(Data::Int(i)) => *i as f64,
            _ => 0.0,
        }
    }
    fn text(&self, col: char, row: usize) -> String {
        match self.0.get(row - 1).and_then(|r| r.get(col as usize - 'A' as usize)) {
            Some(Data::String(s)) => s.clone(),
            _ => String::new(),
        }
    }
    fn rows(&self) -> usize {
        self.0.len()
    }
    /// Row with the event in column A.
    fn event_row(&self, name: &str) -> usize {
        (3..=self.rows())
            .find(|&r| self.text('A', r) == name)
            .unwrap_or_else(|| panic!("Ereignis {name}"))
    }
}

fn month(k: usize) -> String {
    format!("{}-{:02}", 2030 + k / 12, 1 + k % 12)
}

/// The same plan as the sheet: row r ⇒ month r − 3.
fn plan_of(s: &Sheet, pk_interest: f64) -> (Value, usize, usize, usize, usize) {
    let k = |row: usize| row - 3;
    let (early, legal, amort) = (
        k(s.event_row("Early")),
        k(s.event_row("Legal")),
        k(s.event_row("Amortization")),
    );
    let sale = (3..=s.rows())
        .find(|&r| s.text('B', r) == "Sell")
        .map(k)
        .expect("Verkauf");
    let last = s.rows() - 3;
    let g = s.num('G', 2);
    let derisk = k(s.event_row("Derisk"));
    let stocks = json!({ "mode": "timed", "rows": [ { "from": "start", "value": g }, { "from": "derisk", "value": g * s.num('V', 2) } ] });
    let v = json!({
        "project": { "name": "ODS", "currency": "CHF" },
        "household": { "persons": [ { "id": "A", "name": "A", "birth": "1980-01" } ] },
        "milestones": [
            { "id": "start", "name": "Start", "kind": "date", "date": month(0) },
            { "id": "derisk", "name": "Rendite halbiert", "kind": "date", "date": month(derisk) },
            { "id": "retire", "name": "Early", "kind": "date", "date": month(early) },
            { "id": "ahv", "name": "Legal", "kind": "date", "date": month(legal) },
            { "id": "amort", "name": "Amortisation", "kind": "date", "date": month(amort) },
            { "id": "sale", "name": "Verkauf", "kind": "date", "date": month(sale) },
            { "id": "end", "name": "Ende", "kind": "date", "date": month(last) }
        ],
        "pension": {
            "pk": { "capital": s.num('I', 2), "at": "retire", "lumpsum": s.num('M', 2), "conversion": s.num('N', 2),
                    "contribution": s.num('K', 2), "interest": pk_interest },
            "ahv": { "monthly": s.num('O', 2), "at": "ahv" }
        },
        "flows": {
            "income": 0,
            "spending": { "mode": "timed", "rows": [
                { "from": "start", "value": 0 }, { "from": "retire", "value": s.num('P', 2) },
                { "from": "ahv", "value": s.num('P', legal + 3) },
                { "from": "sale", "value": s.num('P', sale + 3) } ] },
            "savings": { "mode": "timed", "rows": [ { "from": "start", "value": s.num('H', 2) }, { "from": "retire", "value": 0 } ] },
            "extras": [
                { "date": month(early), "amount": s.num('L', 2), "label": "Extra" },
                { "date": month(amort), "amount": -s.num('S', 2), "label": "Amortisation" },
                { "date": month(sale), "amount": s.num('T', sale + 3), "label": "Hausgewinn" } ]
        },
        "wealth": { "stocks": s.num('E', 2), "bonds": 0, "cash": 0, "stockShare": 1, "reserveMonths": 0 },
        "returns": { "source": "fixed", "fixed": { "stocks": stocks, "bonds": 0, "inflation": 0, "saron": 0 } },
        "taxes": { "income": [], "wealth": [], "capital": [] }
    });
    (v, early, legal, amort, sale)
}

fn engine_liquid(v: &Value) -> Vec<f64> {
    let p: Plan = serde_json::from_value(v.clone()).expect("Planung gültig");
    let sim = Simulator::new(&p).unwrap();
    let paths = batch::market_paths(&p, &sim, None).unwrap();
    sim.run(&paths[0], true).records.iter().map(|r| r.liquid).collect()
}

fn close(a: f64, b: f64, what: &str) {
    assert!(
        (a - b).abs() <= 0.01 + 1e-9 * b.abs(),
        "{what}: Programm {a} statt Tabelle {b}"
    );
}

/// Sheet 1 (without return): all 604 months match to the centime, also after amortization and the house sale
/// (the spreadsheet no longer cuts wealth off at 0 and sells the house only once).
#[test]
fn sheet1_matches_month_by_month_over_the_whole_horizon() {
    let s = Sheet::load("Sheet1");
    let (v, early, legal, amort, sale) = plan_of(&s, 0.0);
    assert!(errors(&v).is_empty(), "{:?}", errors(&v));
    let liquid = engine_liquid(&v);
    assert!(early < legal && legal < amort && amort < sale);
    assert_eq!(liquid.len(), s.rows() - 2);
    for (k, &l) in liquid.iter().enumerate() {
        close(l, s.num('E', k + 3), &format!("Monat {} (Zeile {})", month(k), k + 3));
    }
    // At the amortization wealth is briefly negative (mortgage 500'000 > wealth); the sale covers it.
    assert!(s.num('E', amort + 3) < 0.0 && liquid[amort] < 0.0);
}

/// The house is sold exactly once (previously twice, because the flag «Sold» was not set), and the sale
/// brings the full market value after the amortization (equity + mortgage).
#[test]
fn sheet1_sells_the_house_exactly_once_for_the_full_value() {
    let s = Sheet::load("Sheet1");
    let sells: Vec<usize> = (3..=s.rows()).filter(|&r| s.text('B', r) == "Sell").collect();
    assert_eq!(sells.len(), 1, "Verkäufe laut Tabelle: {sells:?}");
    let r = sells[0];
    assert_eq!(s.num('T', r), s.num('T', 2) + s.num('S', 2));
    assert_eq!(s.num('Q', r), 650.0);
    assert!((r + 1..=s.rows()).all(|x| s.num('T', x) == 0.0 && s.num('Q', x) == 0.0));
}

/// The pension depends on the remaining capital (1 − lump-sum share), not on the payout share.
#[test]
fn sheet1_pension_is_based_on_the_remaining_capital() {
    let s = Sheet::load("Sheet1");
    let r = s.event_row("Early");
    let capital = s.num('I', r - 1);
    let share = s.num('M', 2);
    assert!((s.num('N', r) - capital * (1.0 - share) * s.num('N', 2) / 12.0).abs() < 0.01);
    assert!((s.num('M', r) - capital * share).abs() < 0.01);
}

/// Sheet 2 (5 % return, halved from row 153): exactly the same month by month until retirement.
#[test]
fn sheet2_matches_until_early_retirement() {
    let s = Sheet::load("Sheet2");
    let (v, early, _, _, _) = plan_of(&s, s.num('J', 2));
    let liquid = engine_liquid(&v);
    for (k, &l) in liquid.iter().enumerate().take(early) {
        close(l, s.num('E', k + 3), &format!("Monat {}", month(k)));
    }
}

/// Sheet 2, PK interest 10 %: the spreadsheet credits the interest only in the January rows, namely on balance
/// plus contribution. The program accrues interest monthly on the balance at the start of the month and credits it in December.
/// Both methods give a similar, not equal capital; the difference is recorded here.
#[test]
fn sheet2_pension_capital_differs_only_by_interest_convention() {
    let s = Sheet::load("Sheet2");
    let (v, early, _, _, _) = plan_of(&s, s.num('J', 2));
    let p: Plan = serde_json::from_value(v).unwrap();
    let sim = Simulator::new(&p).unwrap();
    let paths = batch::market_paths(&p, &sim, None).unwrap();
    let res = sim.run(&paths[0], true);
    let sheet_lump = s.num('M', early + 3);
    let engine_lump = res.summary.lumpsum_net;
    let rel = (engine_lump - sheet_lump) / sheet_lump;
    println!(
        "PK-Kapitalbezug: Tabelle {sheet_lump:.0}, Programm {engine_lump:.0} ({:.2} %)",
        rel * 100.0
    );
    assert!(rel.abs() < 0.10, "Abweichung {:.2} %", rel * 100.0);
    // Without PK interest there would be no difference: sheet 1 (J = 0) matches to the centime
}

fn errors(v: &Value) -> Vec<String> {
    engine::api::validate(v)
        .into_iter()
        .filter(|i| i.severity == engine::validate::Severity::Error)
        .map(|i| i.message)
        .collect()
}
