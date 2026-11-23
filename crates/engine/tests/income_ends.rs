//! #236: Earned income ends with the person's milestone «Bezug Pensionskasse» (switch per person).
//!
//! Made-up plans that can be recomputed by hand: no returns, no inflation; earned income per month.

use engine::model::Plan;
use engine::sim::{MonthRecord, SimResult, Simulator, TaxBasis};
use engine::taxdata;
use engine::time::YearMonth;
use engine::validate::{validate, Severity};
use serde_json::{json, Value};

/// One person «A», born 1980-01, start 2040-01, «Bezug Pensionskasse» and AHV payout at 65 (2045-01), end 2047-12,
/// income 8'000 per month, no taxes.
fn base() -> Value {
    json!({
        "project": { "name": "Ende Erwerb" },
        "household": { "persons": [ { "id": "A", "name": "A", "birth": "1980-01" } ] },
        "milestones": [
            { "id": "start", "kind": "date", "date": "2040-01", "fixed": true },
            { "id": "retire", "kind": "age", "person": "A", "years": 65 },
            { "id": "ahv", "kind": "age", "person": "A", "years": 65 },
            { "id": "end", "kind": "date", "date": "2047-12", "fixed": true }
        ],
        "pension": { "pk": { "capital": 0, "at": "retire", "lumpsum": 0, "conversion": 0 }, "ahv": { "monthly": 0, "at": "ahv" } },
        "flows": { "income": 8000, "spending": 4000 },
        "wealth": { "stocks": 0, "bonds": 0, "cash": 100_000, "stockShare": 0, "reserveMonths": 0 },
        "returns": { "source": "fixed", "fixed": { "stocks": 0, "bonds": 0, "inflation": 0, "saron": 0 } },
        "taxes": { "income": [], "wealth": [], "capital": [] }
    })
}

/// [`base`] with a second person «B», born 1981-01 («Bezug Pensionskasse» at 65 = 2046-01), income 3'000 per month.
fn couple() -> Value {
    let mut v = base();
    v["household"]["persons"] =
        json!([ { "id": "A", "name": "A", "birth": "1980-01" }, { "id": "B", "name": "B", "birth": "1981-01" } ]);
    v["milestones"].as_array_mut().unwrap().extend([
        json!({ "id": "retire2", "kind": "age", "person": "B", "years": 65 }),
        json!({ "id": "ahv2", "kind": "age", "person": "B", "years": 65 }),
    ]);
    v["partner"] = json!({ "income": 3000,
        "pk": { "capital": 0, "at": "retire2", "lumpsum": 0, "conversion": 0 }, "ahv": { "monthly": 0, "at": "ahv2" } });
    v
}

fn plan(v: &Value) -> Plan {
    serde_json::from_value(v.clone()).expect("plan readable")
}

fn run(v: &Value) -> SimResult {
    let s = Simulator::new(&plan(v)).expect("plan valid");
    let path = engine::market::fixed_path(&s.fixed, s.start, s.months());
    s.run(&path, true)
}

fn rec(r: &SimResult, y: i32, m: u32) -> &MonthRecord {
    r.records
        .iter()
        .find(|x| x.month == YearMonth::new(y, m))
        .expect("month in the period")
}

fn earned(v: &Value, y: i32, m: u32) -> f64 {
    rec(&run(v), y, m).earned
}

fn close(a: f64, b: f64, what: &str) {
    assert!((a - b).abs() < 1e-6 * b.abs().max(1.0), "{what}: {a} instead of {b}");
}

fn warns_income_after_pk(v: &Value) -> bool {
    validate(&plan(v))
        .iter()
        .any(|i| i.severity == Severity::Warning && i.section == "flows")
}

/// #236/AK1: Switch on: earned income 0 from the month of «Bezug Pensionskasse» (2045-01), also if the series still
/// has a value afterwards (row «ab Bezug Pensionskasse 5'000»).
#[test]
fn issue_236_ak1_income_ends_with_the_pension_fund_payout() {
    let mut v = base();
    v["flows"]["incomeEndsAtRetirement"] = json!(true);
    let r = run(&v);
    close(rec(&r, 2044, 12).earned, 8000.0, "month before the payout");
    close(rec(&r, 2045, 1).earned, 0.0, "month of the payout");
    close(rec(&r, 2047, 12).earned, 0.0, "last month");
    v["flows"]["income"] =
        json!({ "mode": "timed", "rows": [ { "from": "start", "value": 8000 }, { "from": "retire", "value": 5000 } ] });
    close(earned(&v, 2045, 1), 0.0, "series with a value after the payout");
    close(earned(&v, 2046, 6), 0.0, "series with a value after the payout, later");
    // The yearly income: 2044 12 × 8'000, 2045 nothing.
    let r = run(&v);
    let year = |y: i32| {
        r.records
            .iter()
            .filter(|x| x.month.year() == y)
            .map(|x| x.earned)
            .sum::<f64>()
    };
    close(year(2044), 96_000.0, "2044");
    close(year(2045), 0.0, "2045");
}

/// #236/AK2: Switch off (also missing): the series applies after the payout as before.
#[test]
fn issue_236_ak2_switch_off_follows_the_series() {
    let mut v = base();
    v["flows"]["income"] =
        json!({ "mode": "timed", "rows": [ { "from": "start", "value": 8000 }, { "from": "retire", "value": 5000 } ] });
    close(earned(&v, 2045, 1), 5000.0, "missing");
    v["flows"]["incomeEndsAtRetirement"] = json!(false);
    close(earned(&v, 2045, 1), 5000.0, "off");
    close(earned(&v, 2047, 12), 5000.0, "off, last month");
}

/// #236/AK3: The switch applies per person to their own milestone (`retire` for person 1, `retire2` for person 2);
/// moving the milestone moves the end of the income.
#[test]
fn issue_236_ak3_per_person_own_milestone() {
    let mut v = couple();
    v["partner"]["incomeEndsAtRetirement"] = json!(true);
    // only person 2 (retire2 = 2046-01): person 1 keeps 8'000 after 2045-01
    close(earned(&v, 2045, 6), 8000.0 + 3000.0, "person 2 still working");
    close(earned(&v, 2046, 1), 8000.0, "person 2 from retire2");
    // moving retire2 to 62 (2043-01)
    v["milestones"][4]["years"] = json!(62);
    close(earned(&v, 2042, 12), 11_000.0, "before the moved milestone");
    close(earned(&v, 2043, 1), 8000.0, "from the moved milestone");
    // only person 1 (retire = 2045-01)
    let mut v = couple();
    v["flows"]["incomeEndsAtRetirement"] = json!(true);
    close(earned(&v, 2044, 12), 11_000.0, "both working");
    close(earned(&v, 2045, 1), 3000.0, "person 1 from retire");
    // moving retire to 63 (2043-01)
    v["milestones"][1]["years"] = json!(63);
    close(earned(&v, 2043, 1), 3000.0, "from the moved milestone");
    // both
    v["partner"]["incomeEndsAtRetirement"] = json!(true);
    close(earned(&v, 2046, 1), 0.0, "both ended");
}

/// #236/AK4: Older plans without the switch read it as off and do not write it; the result stays as before.
#[test]
fn issue_236_ak4_older_plans_open_with_the_switch_off() {
    let p = plan(&couple());
    assert!(!p.flows.income_ends_at_retirement);
    assert!(!p.partner.as_ref().unwrap().income_ends_at_retirement);
    let out = serde_json::to_value(&p).unwrap();
    assert!(out["flows"].get("incomeEndsAtRetirement").is_none());
    assert!(out["partner"].get("incomeEndsAtRetirement").is_none());
    // the bundled example plans have no switch (golden files unchanged)
    for file in ["example.vplan", "new.vplan"] {
        let text = std::fs::read_to_string(format!("{}/assets/{file}", env!("CARGO_MANIFEST_DIR"))).unwrap();
        assert!(!text.contains("incomeEndsAtRetirement"), "{file}");
    }
}

/// #236/AK5: The switch is saved and read again, the schema knows it, and the validation drops the warning
/// «Erwerbseinkommen nach dem Bezug der Pensionskasse» only for the person with the switch on.
#[test]
fn issue_236_ak5_saved_schema_and_validation() {
    let mut v = couple();
    v["flows"]["incomeEndsAtRetirement"] = json!(true);
    v["partner"]["incomeEndsAtRetirement"] = json!(true);
    let out = serde_json::to_value(plan(&v)).unwrap();
    assert_eq!(out["flows"]["incomeEndsAtRetirement"], true);
    assert_eq!(out["partner"]["incomeEndsAtRetirement"], true);
    let again: Plan = serde_json::from_value(out).unwrap();
    assert!(again.flows.income_ends_at_retirement && again.partner.unwrap().income_ends_at_retirement);

    let schema = engine::schema_json();
    assert_eq!(schema.matches("incomeEndsAtRetirement").count(), 2, "Flows and Partner");

    assert!(!warns_income_after_pk(&v), "both on: no warning");
    let mut one = v.clone();
    one["partner"]["incomeEndsAtRetirement"] = json!(false);
    assert!(warns_income_after_pk(&one), "person 2 off: warning");
    let single = base();
    assert!(warns_income_after_pk(&single), "off: warning as before");
    let mut single = single;
    single["flows"]["incomeEndsAtRetirement"] = json!(true);
    assert!(!warns_income_after_pk(&single), "on: no warning");
}

/// Plan for the tax consequences: start 2030-01, end 2031-12, own tables with a flat income tax of 10 %, income
/// 100'000 per year; «Bezug Pensionskasse» of person A at 60 = `retire_month` (birth 1970 in that month).
fn taxed(retire_month: u32) -> Value {
    let birth = format!("1970-{retire_month:02}");
    json!({
        "project": { "name": "Ende Erwerb Steuern" },
        "household": { "persons": [ { "id": "A", "name": "A", "birth": birth } ] },
        "milestones": [
            { "id": "start", "kind": "date", "date": "2030-01", "fixed": true },
            { "id": "retire", "kind": "age", "person": "A", "years": 60 },
            { "id": "ahv", "kind": "age", "person": "A", "years": 65 },
            { "id": "end", "kind": "date", "date": "2031-12", "fixed": true }
        ],
        "pension": { "pk": { "capital": 0, "at": "retire", "lumpsum": 0, "conversion": 0 },
                     "ahv": { "monthly": 0, "at": "ahv" } },
        "flows": { "income": 100_000.0 / 12.0, "spending": 1000 },
        "wealth": { "stocks": 0, "bonds": 0, "cash": 1_000_000, "stockShare": 0, "reserveMonths": 0 },
        "returns": { "source": "fixed", "fixed": { "stocks": 0, "bonds": 0, "inflation": 0, "saron": 0 } },
        "taxes": { "multiplier": 1.0, "income": [[0, 0.1]], "wealth": [], "capital": [[0, 0.0]],
                   "propertyGain": { "brackets": [[0, 0]], "reduction": 0.0 } }
    })
}

fn basis(r: &SimResult, y: i32) -> &TaxBasis {
    rec(r, y, 12).tax_basis.as_ref().expect("basis of the income tax")
}

fn part(parts: &[(String, f64)], k: &str) -> f64 {
    parts.iter().find(|p| p.0 == k).map_or(0.0, |p| p.1)
}

/// #236/AK6: Large pillar 3a: income 100'000, contribution 25'000, payout 2030-07. Switch on: 2030 earned 50'000 →
/// deduction 10'000 (20 %), 2031 no earned income → no deduction. Switch off: 20'000 in both years.
#[test]
fn issue_236_ak6_large_pillar3a_follows_the_effective_income() {
    let mut v = taxed(7);
    v["household"]["persons"][0]["noPensionFund"] = json!(true);
    v["taxes"]["deductPillar3a"] = json!(true);
    v["pillar3a"] = json!([ { "person": "A", "balance": 0, "rate": 0, "contribution": 25_000 } ]);
    let r = run(&v);
    close(basis(&r, 2030).pillar3a, 20_000.0, "off 2030");
    close(basis(&r, 2031).pillar3a, 20_000.0, "off 2031");
    v["flows"]["incomeEndsAtRetirement"] = json!(true);
    let r = run(&v);
    close(basis(&r, 2030).income, 50_000.0, "on: half a year of income");
    close(basis(&r, 2030).pillar3a, 10_000.0, "on 2030: 20 % of 50'000");
    close(basis(&r, 2031).pillar3a, 0.0, "on 2031: no earned income");
    close(rec(&r, 2030, 12).assessed_income, 4_000.0, "tax 10 % of 40'000");
}

/// #236/AK6: Proposal of the canton (Zurich, single): with the switch on and the payout at the start there is no
/// earned income, so no professional expenses and the variant without contributions (`retired`); with the switch off
/// the flat deduction of 3 % of 100'000 = 3'000 and the variant «erwerbstätig» (`working`).
#[test]
fn issue_236_ak6_professional_expenses_and_working_variant() {
    let mut v = taxed(1);
    v["taxes"]["mode"] = json!("location");
    v["taxes"]["location"] = json!({ "canton": "ZH", "municipality": 261, "church": "none" });
    v["taxes"]["deductions"] = json!([{ "from": "start", "kind": "canton" }]);
    let resolved = taxdata::builtin()
        .resolve(&serde_json::from_value(v["taxes"]["location"].clone()).unwrap(), false)
        .unwrap();
    let r = run(&v);
    let b = basis(&r, 2030);
    close(
        part(&b.parts_canton, "professional"),
        3000.0,
        "off: professional expenses",
    );
    close(
        b.proposal_canton,
        resolved.proposal(100_000.0, &[100_000.0], 1.0).0,
        "off: working",
    );
    v["flows"]["incomeEndsAtRetirement"] = json!(true);
    let r = run(&v);
    let b = basis(&r, 2030);
    close(b.income, 0.0, "on: no income");
    close(
        part(&b.parts_canton, "professional"),
        0.0,
        "on: no professional expenses",
    );
    let (retired, _) = resolved.proposal(0.0, &[0.0], 1.0);
    close(b.proposal_canton, retired, "on: variant without earned income");
    assert!(
        resolved.proposal(0.0, &[100_000.0], 1.0).0 != retired,
        "the variants differ (otherwise the test proves nothing)"
    );
}

/// #236/AK6: Second-earner deduction (married, Zurich): only while both persons have earned income. Person 2 with the
/// switch on and the payout at the start has none, so there is no second-earner deduction.
#[test]
fn issue_236_ak6_second_earner_deduction() {
    let mut v = taxed(1);
    v["household"]["persons"] = json!([
        { "id": "A", "name": "A", "birth": "1970-01" }, { "id": "B", "name": "B", "birth": "1970-01" } ]);
    v["household"]["married"] = json!(true);
    v["milestones"][1]["years"] = json!(65);
    v["milestones"].as_array_mut().unwrap().extend([
        json!({ "id": "retire2", "kind": "age", "person": "B", "years": 60 }),
        json!({ "id": "ahv2", "kind": "age", "person": "B", "years": 65 }),
    ]);
    v["partner"] = json!({ "income": 60_000.0 / 12.0,
        "pk": { "capital": 0, "at": "retire2", "lumpsum": 0, "conversion": 0 }, "ahv": { "monthly": 0, "at": "ahv2" } });
    v["taxes"]["mode"] = json!("location");
    v["taxes"]["location"] = json!({ "canton": "ZH", "municipality": 261, "church": "none" });
    v["taxes"]["deductions"] = json!([{ "from": "start", "kind": "canton" }]);
    let r = run(&v);
    assert!(
        part(&basis(&r, 2030).parts_canton, "secondEarner") > 0.0,
        "off: both earn"
    );
    v["partner"]["incomeEndsAtRetirement"] = json!(true);
    let r = run(&v);
    let b = basis(&r, 2030);
    close(b.income, 100_000.0, "on: only person 1");
    close(
        part(&b.parts_canton, "secondEarner"),
        0.0,
        "on: no second-earner deduction",
    );
}
