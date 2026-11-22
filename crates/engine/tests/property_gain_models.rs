//! #65/AK2: The models and supplements of the property gain tax (`tax::property_gain_tax_full`) one by one, with
//! invented rules. The cases per canton are in `tax_gain_cantons.rs`.

use engine::model::{GainModel, PropertyGainTax};
use engine::tax::{property_gain_tax_full, property_gain_tax_held};

fn zurich() -> PropertyGainTax {
    PropertyGainTax {
        model: GainModel::Zurich,
        brackets: vec![(10_000.0, 0.1), (10_000.0, 0.2)],
        ..Default::default()
    }
}

fn close(a: f64, b: f64) {
    assert!((a - b).abs() < 0.005, "{a} != {b}");
}

fn full(def: &PropertyGainTax, gain: f64, cost: f64, months: Option<i32>) -> f64 {
    property_gain_tax_full(Some(def), gain, cost, months, 1.0)
}

#[test]
fn issue_65_ak2_allowance_is_deducted_before_tax() {
    let def = PropertyGainTax {
        allowance: 5_000.0,
        ..zurich()
    };
    // (15 000 − 5 000) = 10 000 at 10 %
    close(full(&def, 15_000.0, 0.0, None), 1_000.0);
    // gain not above the allowance: no tax
    assert_eq!(full(&def, 5_000.0, 0.0, None), 0.0);
}

#[test]
fn issue_65_ak2_uniform_rate_applies_only_above_threshold() {
    let def = PropertyGainTax {
        uniform_above: Some((20_000.0, 0.3)),
        ..zurich()
    };
    // 30 000 > 20 000: whole gain at 30 %
    close(full(&def, 30_000.0, 0.0, None), 9_000.0);
    // exactly on the threshold: brackets 1 000 + 2 000
    close(full(&def, 20_000.0, 0.0, None), 3_000.0);
}

#[test]
fn issue_65_ak2_curve_model_uses_rate_for_whole_gain() {
    let def = PropertyGainTax {
        model: GainModel::Curve,
        rate_curve: vec![(0.0, 0.1), (100_000.0, 0.3)],
        ..Default::default()
    };
    // rate at 50 000: 0.2 → 10 000
    close(full(&def, 50_000.0, 0.0, None), 10_000.0);
}

#[test]
fn issue_65_ak2_return_model_rate_from_return_per_year_clamped() {
    let def = PropertyGainTax {
        model: GainModel::Return,
        return_bounds: Some((0.01, 0.9)),
        ..Default::default()
    };
    let t = |m| full(&def, 100_000.0, 400_000.0, m);
    // return 25 %: 24 months = 2 years → 12.5 %
    close(t(Some(24)), 12_500.0);
    // 60 months: 5 years (started months) → 5 %
    close(t(Some(60)), 5_000.0);
    // 61 months: rounded up to 6 years → 25 % / 6
    close(t(Some(61)), 100_000.0 * 0.25 / 6.0);
    // holding period unknown: highest rate
    close(t(None), 90_000.0);
    // very short: upper bound 0.9; very long: lower bound 0.01
    close(t(Some(1)), 90_000.0);
    close(t(Some(1200)), 1_000.0);
    // without investment costs: highest rate
    close(full(&def, 100_000.0, 0.0, Some(24)), 90_000.0);
}

#[test]
fn issue_65_ak2_return_model_maximum_by_holding_period() {
    let def = PropertyGainTax {
        model: GainModel::Return,
        return_bounds: Some((0.1, 0.6)),
        return_max_duration: vec![(0.0, 0.6), (10.0, 0.4)],
        ..Default::default()
    };
    // return 100 % per year, capped: 60 % before 10 years, 40 % from 10 years
    close(full(&def, 1_000_000.0, 100_000.0, Some(108)), 600_000.0);
    close(full(&def, 2_000_000.0, 100_000.0, Some(120)), 800_000.0);
}

#[test]
fn issue_65_ak2_gain_duration_changes_gain_by_holding_period_in_months() {
    let def = PropertyGainTax {
        gain_duration: vec![(0.0, 0.0), (10.0, -0.1)],
        ..zurich()
    };
    // 9 years 11 months: no change yet; 10 years: gain 10 % lower
    close(full(&def, 10_000.0, 0.0, Some(119)), 1_000.0);
    close(full(&def, 10_000.0, 0.0, Some(120)), 900.0);
    // without holding period no change
    close(full(&def, 10_000.0, 0.0, None), 1_000.0);
    // a change of −100 % leaves nothing to tax
    let all = PropertyGainTax {
        gain_duration: vec![(0.0, -1.0)],
        ..zurich()
    };
    assert_eq!(full(&all, 10_000.0, 0.0, Some(12)), 0.0);
}

#[test]
fn issue_65_ak2_max_share_caps_tax_at_share_of_gain() {
    let def = PropertyGainTax {
        max_share: 0.15,
        ..zurich()
    };
    // 1 000 + 2 000 + 20 000 × 20 % = 7 000 > 15 % of 40 000 = 6 000
    close(full(&def, 40_000.0, 0.0, None), 6_000.0);
    close(full(&def, 20_000.0, 0.0, None), 3_000.0);
}

#[test]
fn issue_65_ak2_duration_above_splits_the_adjustment_by_gain_share() {
    let def = PropertyGainTax {
        model: GainModel::Zurich,
        brackets: vec![(1e12, 0.2)],
        duration: vec![(10.0, -0.4)],
        duration_above: Some((100_000.0, vec![(10.0, -0.1)])),
        ..Default::default()
    };
    // up to the threshold only `duration`: 50 000 × 20 % × 0.6
    close(full(&def, 50_000.0, 0.0, Some(120)), 6_000.0);
    // 200 000 × 20 % = 40 000; half −40 %, half −10 %: 40 000 × (1 − 0.25) = 30 000
    close(full(&def, 200_000.0, 0.0, Some(120)), 30_000.0);
    // before the holding period of 10 years no adjustment
    close(full(&def, 200_000.0, 0.0, Some(60)), 40_000.0);
}

#[test]
fn issue_65_ak2_short_surcharge_above_threshold() {
    let def = PropertyGainTax {
        model: GainModel::Aargau,
        duration_rates: vec![(0.0, 0.2)],
        short_surcharge: Some((100_000.0, 0.5, 5.0)),
        ..Default::default()
    };
    // 150 000 × 20 % = 30 000; on 50 000 additionally 20 % × 50 % = 5 000
    close(full(&def, 150_000.0, 0.0, Some(59)), 35_000.0);
    // from 5 years, at the threshold or without holding period: no surcharge
    close(full(&def, 150_000.0, 0.0, Some(60)), 30_000.0);
    close(full(&def, 100_000.0, 0.0, Some(12)), 20_000.0);
    close(full(&def, 150_000.0, 0.0, None), 30_000.0);
}

#[test]
fn issue_65_ak2_long_brackets_from_holding_period() {
    let def = PropertyGainTax {
        long_brackets: Some((25.0, vec![(10_000.0, 0.01), (10_000.0, 0.02)])),
        ..zurich()
    };
    close(full(&def, 20_000.0, 0.0, Some(299)), 3_000.0);
    close(full(&def, 20_000.0, 0.0, Some(300)), 300.0);
    close(full(&def, 20_000.0, 0.0, None), 3_000.0);
}

#[test]
fn issue_65_ak2_surcharge_by_months_and_multiplier() {
    // surcharge per started month: 1 % per month under 3 months
    let def = PropertyGainTax {
        brackets: vec![(1e12, 0.1)],
        duration: vec![(0.0, 0.03), (1.0 / 12.0, 0.02), (2.0 / 12.0, 0.01), (0.25, 0.0)],
        use_multiplier: true,
        ..Default::default()
    };
    let t = |m| property_gain_tax_full(Some(&def), 100_000.0, 0.0, Some(m), 2.0);
    close(t(0), 20_600.0);
    close(t(1), 20_400.0);
    close(t(2), 20_200.0);
    close(t(3), 20_000.0);
}

/// #65/AK5: Rules of existing plans compute as before: the model «aargau» ignores `duration` rows, full years.
#[test]
fn issue_65_ak5_existing_rules_unchanged() {
    let ag = PropertyGainTax {
        model: GainModel::Aargau,
        duration_rates: vec![(0.0, 0.3), (5.0, 0.2)],
        duration: vec![(0.0, 0.5)],
        ..Default::default()
    };
    close(property_gain_tax_held(Some(&ag), 10_000.0, Some(7), 1.0), 2_000.0);
    close(property_gain_tax_held(Some(&ag), 10_000.0, None, 1.0), 3_000.0);
    let zh = PropertyGainTax {
        duration: vec![(0.0, 0.5), (2.0, 0.0), (5.0, -0.2)],
        exempt_below: 5_000.0,
        ..zurich()
    };
    close(property_gain_tax_held(Some(&zh), 20_000.0, Some(1), 1.0), 4_500.0);
    close(property_gain_tax_held(Some(&zh), 20_000.0, Some(6), 1.0), 2_400.0);
    assert_eq!(property_gain_tax_held(Some(&zh), 4_999.0, Some(6), 1.0), 0.0);
    assert_eq!(property_gain_tax_held(None, 20_000.0, Some(6), 1.0), 0.0);
}
