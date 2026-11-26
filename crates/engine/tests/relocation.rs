//! #151: Time-dependent place of residence in the mode «Wohnort»: `taxes.location` is the first row (from the start),
//! `taxes.relocations` the further rows. Income and wealth tax follow the place of residence on 31 December (StHG
//! Art. 4b para. 1, Art. 17 para. 1), the lump-sum tax the place in the month of the payout, the property gain tax of a
//! property without its own canton the canton of residence in the month of the sale.

use engine::model::{Church, Plan, TaxLocation};
use engine::sim::{SimResult, Simulator};
use engine::tax::{self, property_gain_tax_full};
use engine::taxdata;
use engine::time::YearMonth;
use engine::validate::{validate, Severity};
use serde_json::{json, Value};

fn loc(canton: &str, bfs: u32) -> TaxLocation {
    TaxLocation {
        canton: canton.into(),
        municipality: Some(bfs),
        church: Church::None,
    }
}

fn zurich() -> Value {
    json!({ "canton": "ZH", "municipality": 261, "church": "none" })
}

fn zug_from(from: &str) -> Value {
    json!({ "from": from, "canton": "ZG", "municipality": 1711, "church": "none" })
}

/// Person A (born 1975-01) with earned income until the end of 2040, a lump sum of 300 000 at 65 (2040-01) and
/// wealth; place of residence Zurich from the start (2039-01) until the end (2042-12).
fn plan() -> Value {
    json!({
        "project": { "name": "Umzug" },
        "household": { "persons": [ { "id": "A", "name": "A", "birth": "1975-01" } ] },
        "milestones": [
            { "id": "start", "name": "Start", "kind": "date", "date": "2039-01", "fixed": true },
            { "id": "retire", "name": "Bezug Pensionskasse", "kind": "age", "person": "A", "years": 65 },
            { "id": "ahv", "name": "AHV-Bezug", "kind": "age", "person": "A", "years": 65 },
            { "id": "end", "name": "Ende", "kind": "date", "date": "2042-12", "fixed": true }
        ],
        "pension": { "pk": { "capital": 300_000, "at": "retire", "lumpsum": 1.0, "conversion": 0 },
                     "ahv": { "monthly": 2000, "at": "ahv" } },
        "flows": { "income": 8000, "spending": 5000 },
        "wealth": { "stocks": 0, "bonds": 0, "cash": 900_000, "stockShare": 0, "reserveMonths": 0 },
        "returns": { "source": "fixed", "fixed": { "stocks": 0, "bonds": 0, "inflation": 0, "saron": 0 } },
        "taxes": { "multiplier": 1.0, "income": [[0, 0.1]], "wealth": [[0, 0.0]], "capital": [[0, 0.0]],
                   "mode": "location", "location": zurich() }
    })
}

fn run(v: &Value) -> SimResult {
    let plan: Plan = serde_json::from_value(v.clone()).unwrap();
    let s = Simulator::new(&plan).unwrap();
    let path = engine::market::fixed_path(&s.fixed, s.start, s.months());
    s.run(&path, true)
}

fn december(r: &SimResult, year: i32) -> &engine::sim::MonthRecord {
    r.records.iter().find(|x| x.month == YearMonth::new(year, 12)).unwrap()
}

fn capital_tax(r: &SimResult, year: i32) -> f64 {
    r.records
        .iter()
        .filter(|x| x.month.year() == year)
        .map(|x| x.tax_capital)
        .sum()
}

fn issues(v: &Value) -> Vec<(&'static str, Option<String>)> {
    let plan: Plan = serde_json::from_value(v.clone()).unwrap();
    validate(&plan)
        .into_iter()
        .filter(|i| i.severity == Severity::Error)
        .map(|i| (i.message.code(), i.field))
        .collect()
}

/// #151/AK2, AK9: A move in June 2041 from Zurich to Zug: the income and wealth tax of 2041 is that of Zug for the
/// whole year (place of residence on 31 December), the year 2040 stays with Zurich.
#[test]
fn issue_151_ak2_ak9_move_in_june_whole_year_at_new_place() {
    let mut moved = plan();
    moved["taxes"]["relocations"] = json!([zug_from("2041-06")]);
    let mut zug = plan();
    zug["taxes"]["location"] = json!({ "canton": "ZG", "municipality": 1711, "church": "none" });
    let (rm, rz, rzh) = (run(&moved), run(&zug), run(&plan()));
    // 2040: as with Zurich throughout (also the wealth tax), differs from Zug.
    let (m40, zh40, zg40) = (december(&rm, 2040), december(&rzh, 2040), december(&rz, 2040));
    assert_eq!(m40.assessed_income, zh40.assessed_income);
    assert_eq!(m40.assessed_wealth, zh40.assessed_wealth);
    assert!(
        zg40.assessed_income < zh40.assessed_income - 1000.0,
        "Zug must be cheaper"
    );
    // 2041: the whole year in Zug – the income tax equals that of a plan with Zug throughout (same income).
    let (m41, zg41) = (december(&rm, 2041), december(&rz, 2041));
    assert!(m41.assessed_income > 0.0);
    assert!((m41.assessed_income - zg41.assessed_income).abs() < 1e-6);
    // Wealth tax 2041 by the tariff of Zug on the wealth of the plan with the move.
    let zg = taxdata::builtin().resolve(&loc("ZG", 1711), false).unwrap();
    let want = zg.wealth_tax(m41.liquid, 1.0);
    assert!(want > 0.0);
    assert!(
        (m41.assessed_wealth - want).abs() < 0.01,
        "{} instead of {want}",
        m41.assessed_wealth
    );
    // A move in January 2042 changes nothing in 2041.
    let mut later = plan();
    later["taxes"]["relocations"] = json!([zug_from("2042-01")]);
    assert_eq!(
        december(&run(&later), 2041).assessed_income,
        december(&rzh, 2041).assessed_income
    );
}

/// #151/AK3, AK9: The lump sum (January 2040) is taxed at the place of residence in the month of the payout: a move
/// after the payout (June 2040) leaves it in Zurich, a move before (December 2039) or in the same month (January 2040)
/// moves it to Zug, although the income tax of 2039 or 2040 follows 31 December.
#[test]
fn issue_151_ak3_ak9_lump_sum_before_and_after_the_move() {
    let table = |l: TaxLocation| taxdata::builtin().resolve(&l, false).unwrap().capital;
    let zh = tax::tax(&table(loc("ZH", 261)), 300_000.0, 1.0);
    let zg = tax::tax(&table(loc("ZG", 1711)), 300_000.0, 1.0);
    assert!(zg < zh - 1000.0);
    for (from, want) in [("2040-06", zh), ("2039-12", zg), ("2040-01", zg)] {
        let mut v = plan();
        v["taxes"]["relocations"] = json!([zug_from(from)]);
        let got = capital_tax(&run(&v), 2040);
        assert!((got - want).abs() < 0.02, "move {from}: {got} instead of {want}");
    }
    // Back to Zurich in February 2040: the payout in January stays in Zug, the year 2040 is assessed in Zurich.
    let mut v = plan();
    v["taxes"]["relocations"] =
        json!([zug_from("2039-12"), { "from": "2040-02", "canton": "ZH", "municipality": 261 }]);
    let r = run(&v);
    assert!((capital_tax(&r, 2040) - zg).abs() < 0.02);
    assert_eq!(
        december(&r, 2040).assessed_income,
        december(&run(&plan()), 2040).assessed_income
    );
}

/// Plan with a house without its own canton (`canton`: None) or in `canton`, sold in June 2041.
fn gain_plan(canton: Option<&str>, move_from: &str, to: (&str, u32)) -> Value {
    let mut v = plan();
    let mut house = json!({ "id": "h", "name": "Haus", "status": "owned", "price": 500_000, "worth": 800_000,
        "bought": "2030-01", "sell": "2041-06", "fixCosts": 0, "inflation": false });
    if let Some(c) = canton {
        house["canton"] = json!(c);
    }
    v["realestate"] = json!({ "properties": [house] });
    v["taxes"]["relocations"] = json!([{ "from": move_from, "canton": to.0, "municipality": to.1 }]);
    v
}

fn sale_tax(v: &Value) -> f64 {
    run(v)
        .records
        .iter()
        .find(|x| x.month == YearMonth::new(2041, 6))
        .unwrap()
        .tax_gain
}

fn expected_gain(residence: TaxLocation, canton: Option<&str>) -> f64 {
    let (rule, m) = taxdata::builtin().property_gain(&residence, canton).unwrap();
    // bought 2030-01, sold 2041-06: 137 months; gain 300 000 on 500 000
    property_gain_tax_full(Some(&rule), 300_000.0, 500_000.0, Some(137), m)
}

/// #151/AK4: A property without its own canton pays the property gain tax of the canton of residence in the month of
/// the sale (move to Fribourg before or after the sale); with its own canton the canton of the property applies.
#[test]
fn issue_151_ak4_gain_tax_follows_residence_in_month_of_sale() {
    let zh = expected_gain(loc("ZH", 261), None);
    let fr = expected_gain(loc("FR", 2196), None);
    assert!((zh - fr).abs() > 1000.0);
    assert!((sale_tax(&gain_plan(None, "2041-01", ("FR", 2196))) - fr).abs() < 0.01);
    assert!((sale_tax(&gain_plan(None, "2041-06", ("FR", 2196))) - fr).abs() < 0.01);
    assert!((sale_tax(&gain_plan(None, "2041-07", ("FR", 2196))) - zh).abs() < 0.01);
    // Own canton Zurich: after the move to Fribourg the median multiplier of Zurich (no longer the municipality).
    let own = sale_tax(&gain_plan(Some("ZH"), "2041-01", ("FR", 2196)));
    assert!((own - expected_gain(loc("FR", 2196), Some("ZH"))).abs() < 0.01);
    let own_before = sale_tax(&gain_plan(Some("ZH"), "2041-07", ("FR", 2196)));
    assert!((own_before - zh).abs() < 0.01);
    // Own rules (`propertyGainOwn`, #65/AK5): the same rule after one or two moves (300 000 × 20 % × multiplier 1).
    let own_rule = json!({ "brackets": [[1e12, 0.2]] });
    for moves in [
        json!([{ "from": "2041-01", "canton": "FR", "municipality": 2196 }]),
        json!([zug_from("2040-01"), { "from": "2041-01", "canton": "FR", "municipality": 2196 }]),
    ] {
        let mut v = gain_plan(None, "2041-01", ("FR", 2196));
        v["taxes"]["relocations"] = moves;
        v["taxes"]["propertyGain"] = own_rule.clone();
        v["taxes"]["propertyGainOwn"] = json!(true);
        assert!((sale_tax(&v) - 60_000.0).abs() < 0.01, "{}", sale_tax(&v));
    }
}

/// #151/AK5: The proposal of the canton follows the place of residence of the tax year; an own value stays.
#[test]
fn issue_151_ak5_deductions_follow_residence_own_value_stays() {
    let mut moved = plan();
    moved["taxes"]["relocations"] = json!([zug_from("2041-06")]);
    let mut zug = plan();
    zug["taxes"]["location"] = json!({ "canton": "ZG", "municipality": 1711, "church": "none" });
    let b = december(&run(&moved), 2041).tax_basis.clone().unwrap();
    let z = december(&run(&zug), 2041).tax_basis.clone().unwrap();
    let zh = december(&run(&plan()), 2041).tax_basis.clone().unwrap();
    assert_eq!(b.parts_canton, z.parts_canton);
    assert_eq!(b.proposal_canton, z.proposal_canton);
    assert_ne!(b.proposal_canton, zh.proposal_canton);
    // Own value of 12 000 per year: the same after the move.
    let rows = json!([{ "from": "start", "kind": "own", "amount": 12_000, "rate": 0 }]);
    moved["taxes"]["deductions"] = rows.clone();
    let mut own_zh = plan();
    own_zh["taxes"]["deductions"] = rows;
    let b = december(&run(&moved), 2041).tax_basis.clone().unwrap();
    assert_eq!(b.own, 12_000.0);
    assert_eq!(b.proposal_canton, 0.0);
    assert_eq!(
        b.taxable_canton,
        december(&run(&own_zh), 2041).tax_basis.clone().unwrap().taxable_canton
    );
}

/// #151/AK6: The validation reports errors per row at the field and rows that start before the previous one.
#[test]
fn issue_151_ak6_validation_per_row_and_order() {
    let mut v = plan();
    v["taxes"]["relocations"] = json!([
        { "from": "2041-01", "canton": "", "church": "none" },
        { "from": "2040-06", "canton": "ZG", "municipality": 261 },
        { "from": "2042-01", "canton": "XX", "municipality": 1711 },
        { "from": "2042-02", "canton": "ZG" },
        { "from": "2042-03", "canton": "", "municipality": 1711 },
        { "from": "2038-01", "canton": "ZG", "municipality": 1711 }
    ]);
    let got = issues(&v);
    let f = |s: &str| Some(s.to_string());
    for want in [
        // #272/AK4: without canton and municipality the municipality is missing
        ("relocation_municipality_missing", f("taxes.relocations.0.municipality")),
        (
            "relocation_municipality_not_in_canton",
            f("taxes.relocations.1.municipality"),
        ),
        ("relocation_before_previous", f("taxes.relocations.1.from")),
        ("relocation_canton_unknown", f("taxes.relocations.2.canton")),
        ("relocation_municipality_missing", f("taxes.relocations.3.municipality")),
        ("relocation_canton_missing", f("taxes.relocations.4.canton")),
        ("relocation_before_previous", f("taxes.relocations.5.from")),
    ] {
        assert!(got.contains(&want), "{want:?} missing in {got:?}");
    }
    assert_eq!(got.len(), 7, "{got:?}");
    // The row number in the message counts the first row (taxes.location) as row 1.
    let parsed: Plan = serde_json::from_value(v.clone()).unwrap();
    let text = validate(&parsed)
        .into_iter()
        .find(|i| i.message.code() == "relocation_before_previous")
        .unwrap()
        .message
        .to_string();
    assert!(
        text.contains("row 3") && text.contains("06.2040") && text.contains("01.2041"),
        "{text}"
    );
    let texts: Vec<String> = validate(&parsed).into_iter().map(|i| i.message.to_string()).collect();
    for want in [
        "row 2: please choose the municipality",
        "row 6: please choose the canton",
        "row 3: the municipality 261",
        "row 4: the canton “XX”",
        "row 5: please choose the municipality",
    ] {
        assert!(texts.iter().any(|t| t.contains(want)), "{want} missing in {texts:?}");
    }
    // Correct rows, also in the same month and at an event: no error; an unknown event is reported.
    v["taxes"]["relocations"] = json!([zug_from("ahv"), zug_from("2040-01"), zug_from("2041-01")]);
    assert!(issues(&v).is_empty(), "{:?}", issues(&v));
    v["taxes"]["relocations"] = json!([zug_from("unbekannt")]);
    assert!(!issues(&v).is_empty());
    // With own tables the rows are neither checked nor applied.
    v["taxes"]["mode"] = json!("own");
    v["taxes"]["relocations"] = json!([{ "from": "2038-01", "canton": "" }]);
    assert!(issues(&v).is_empty(), "{:?}", issues(&v));
    assert_eq!(run(&v).summary.taxes_total, {
        let mut o = plan();
        o["taxes"]["mode"] = json!("own");
        run(&o).summary.taxes_total
    });
}

/// #151/AK7: The result names the place of residence of each year (tax basis in December), only if it changes.
#[test]
fn issue_151_ak7_result_names_residence_per_year() {
    let mut v = plan();
    v["taxes"]["relocations"] = json!([zug_from("2041-06")]);
    let r = run(&v);
    let place = |y| december(&r, y).tax_basis.clone().unwrap().residence.unwrap();
    assert_eq!(place(2040), loc("ZH", 261));
    assert_eq!(place(2041), loc("ZG", 1711));
    assert_eq!(place(2042), loc("ZG", 1711));
    let single = run(&plan());
    let b = december(&single, 2041).tax_basis.clone().unwrap();
    assert!(b.residence.is_none());
    assert!(!serde_json::to_string(&b).unwrap().contains("residence"));
}

/// #151/AK8: A single place of residence computes unchanged: the old form stays readable and is written unchanged,
/// empty rows and a row «never» change nothing, a further row with the same place only adds the place to the result.
#[test]
fn issue_151_ak8_single_residence_unchanged() {
    let base = run(&plan());
    let parsed: Plan = serde_json::from_value(plan()).unwrap();
    assert!(parsed.taxes.relocations.is_empty());
    let written = serde_json::to_value(&parsed.taxes).unwrap();
    assert!(written.get("relocations").is_none());
    assert_eq!(written["location"], zurich());
    for rows in [
        json!([]),
        json!([{ "from": "never", "canton": "ZG", "municipality": 1711 }]),
    ] {
        let mut v = plan();
        v["taxes"]["relocations"] = rows;
        let r = run(&v);
        assert_eq!(
            serde_json::to_value(&r.records).unwrap(),
            serde_json::to_value(&base.records).unwrap()
        );
    }
    let mut same = plan();
    same["taxes"]["relocations"] = json!([{ "from": "2040-06", "canton": "ZH", "municipality": 261 }]);
    let r = run(&same);
    assert_eq!(r.summary.taxes_total, base.summary.taxes_total);
    for (a, b) in r.records.iter().zip(&base.records) {
        assert_eq!(a.taxes, b.taxes);
        assert_eq!(a.total, b.total);
    }
    // Round trip with rows: written as rows, read again equally.
    let mut v = plan();
    v["taxes"]["relocations"] = json!([zug_from("2041-06")]);
    let p: Plan = serde_json::from_value(v).unwrap();
    let again: Plan = serde_json::from_value(serde_json::to_value(&p).unwrap()).unwrap();
    assert_eq!(again.taxes.relocations, p.taxes.relocations);
    assert_eq!(again.taxes.relocations[0].place, loc("ZG", 1711));
}
