//! #65/AK4: Property gain tax per canton against values computed by hand. Invented cases; the rules come from the
//! ESTV «Steuermäppchen» on the property gain tax, tax period 2025, table 3 (page per canton in the comment), and for
//! ZG, FR, VS, BL, BS, VD and SG from the cantonal law (source with each case). Each value is computed by hand from the
//! tariff, the surcharge or reduction by holding period and, where the canton requires it, the tax multiplier (here
//! 1.8). The holding period is given in months.

use engine::model::{PropertyGainTax, TaxLocation};
use engine::tax::property_gain_tax_full;
use engine::taxdata;

fn rule(canton: &str) -> PropertyGainTax {
    let loc = TaxLocation {
        canton: "ZH".into(),
        ..Default::default()
    };
    taxdata::builtin().property_gain(&loc, Some(canton)).unwrap().0
}

fn tax(canton: &str, gain: f64, cost: f64, months: i32) -> f64 {
    property_gain_tax_full(Some(&rule(canton)), gain, cost, Some(months), 1.0)
}

fn close(a: f64, b: f64) {
    assert!((a - b).abs() < 0.01, "{a} != {b}");
}

/// § 199 StG ZG (BGS 632.1): rate = return per year, 10 % to 60 %; from 12 years the maximum rate falls by 2.5
/// points per year.
#[test]
fn issue_65_ak4_zg_rate_is_annual_return_with_falling_maximum() {
    // 100 000 on 400 000 in 5 years: 25 % / 5 = 5 %, minimum rate 10 %
    close(tax("ZG", 100_000.0, 400_000.0, 60), 10_000.0);
    // 300 000 on 400 000 in 24 months: 75 % / 2 = 37.5 %
    close(tax("ZG", 300_000.0, 400_000.0, 24), 112_500.0);
    // 1 million in 2 years: 125 % per year, maximum rate 60 %
    close(tax("ZG", 1_000_000.0, 400_000.0, 24), 600_000.0);
    // 1.2 million in 20 years: 300 % / 20 = 15 %
    close(tax("ZG", 1_200_000.0, 400_000.0, 240), 180_000.0);
}

#[test]
fn issue_65_ak4_zg_maximum_falls_after_twelve_years() {
    // 4 million on 400 000: return per year above every maximum rate
    close(tax("ZG", 4_000_000.0, 400_000.0, 11 * 12), 2_400_000.0); // 60 %
    close(tax("ZG", 4_000_000.0, 400_000.0, 12 * 12), 2_300_000.0); // 57.5 %
    close(tax("ZG", 4_000_000.0, 400_000.0, 20 * 12), 1_500_000.0); // 37.5 %
    close(tax("ZG", 4_000_000.0, 400_000.0, 25 * 12), 1_000_000.0); // 25 %
    close(tax("ZG", 4_000_000.0, 400_000.0, 40 * 12), 1_000_000.0); // 25 %
}

#[test]
fn issue_65_ak4_zg_exempt_below_5000() {
    assert_eq!(tax("ZG", 4_999.0, 100_000.0, 60), 0.0);
    close(tax("ZG", 5_000.0, 100_000.0, 12), 500.0); // 5 % / 1 = 5 % → minimum rate 10 %
}

/// Art. 51 DStG FR (RSF 631.1): 22 % up to 2 years, 20 % up to 4, 18 % up to 6; exemption limit 6000; above 400 000
/// francs and under 5 years a surcharge of 40 % on the exceeding part.
#[test]
fn issue_65_ak4_fr_rate_by_duration_and_exempt_below() {
    assert_eq!(tax("FR", 5_999.0, 0.0, 12), 0.0);
    close(tax("FR", 6_000.0, 0.0, 12), 1_320.0);
    close(tax("FR", 100_000.0, 0.0, 3 * 12), 20_000.0);
    close(tax("FR", 100_000.0, 0.0, 20 * 12), 10_000.0);
}

#[test]
fn issue_65_ak4_fr_surcharge_on_gain_above_400000_within_five_years() {
    // 450 000 after 1 year: 22 % = 99 000, on 50 000 additionally 22 % × 40 % = 4 400
    close(tax("FR", 450_000.0, 0.0, 12), 103_400.0);
    // exactly 400 000: no surcharge
    close(tax("FR", 400_000.0, 0.0, 12), 88_000.0);
    // after 5 years no surcharge: 450 000 at 18 %
    close(tax("FR", 450_000.0, 0.0, 60), 81_000.0);
}

/// Art. 52 StG VS (SGS 642.1): 12/18/24 % per 50 000, from the 6th year 4 % less per year, from 25 years 1/2/3 %;
/// surcharges in the first 5 years (table: 1st year +60 %, 2nd +50 %). Bracket tariff by the amount of the gain
/// assumed.
#[test]
fn issue_65_ak4_vs_reduction_from_sixth_year_and_fixed_rates_after_25() {
    // 120 000: 50 000 × 12 % + 50 000 × 18 % + 20 000 × 24 % = 19 800
    close(tax("VS", 120_000.0, 0.0, 5 * 12), 19_800.0);
    close(tax("VS", 120_000.0, 0.0, 6 * 12), 19_800.0 * 0.96);
    close(tax("VS", 120_000.0, 0.0, 10 * 12), 19_800.0 * 0.80);
    close(tax("VS", 120_000.0, 0.0, 24 * 12), 19_800.0 * 0.24);
    // from 25 years: 50 000 × 1 % + 50 000 × 2 % + 20 000 × 3 % = 2 100
    close(tax("VS", 120_000.0, 0.0, 25 * 12), 2_100.0);
}

#[test]
fn issue_65_ak4_vs_surcharge_in_first_five_years() {
    close(tax("VS", 120_000.0, 0.0, 6), 19_800.0 * 1.6);
    close(tax("VS", 120_000.0, 0.0, 12), 19_800.0 * 1.5);
    close(tax("VS", 120_000.0, 0.0, 4 * 12), 19_800.0 * 1.1);
}

/// § 80 StG BL (SGS 331): rate 3 % + 0.03 % per 100 francs up to 12 % at 30 000, up to 20 % at 70 000, 25 % from
/// 120 000; under 5 years +1 2/3 % on the tax per missing month.
#[test]
fn issue_65_ak4_bl_rate_curve_and_monthly_surcharge() {
    // 30 000 × 12 %
    close(tax("BL", 30_000.0, 0.0, 10 * 12), 3_600.0);
    // 200 000 × 25 %
    close(tax("BL", 200_000.0, 0.0, 10 * 12), 50_000.0);
    // 30 000, 54 months: 6 missing months = +10 %
    close(tax("BL", 30_000.0, 0.0, 54), 3_600.0 * 1.10);
}

/// § 109 StG BS: tariff of the tax administration from 2023 for permanently owner-occupied residential property:
/// 30 % up to 5 years, then −0.9 points per year, from 25 years 12 %. Exemption limit 500 francs.
#[test]
fn issue_65_ak4_bs_owner_occupied_tariff() {
    close(tax("BS", 100_000.0, 0.0, 5 * 12 - 1), 30_000.0); // in the 5th year
    close(tax("BS", 100_000.0, 0.0, 6 * 12 - 1), 29_100.0); // in the 6th year
    close(tax("BS", 100_000.0, 0.0, 15 * 12 - 1), 21_000.0); // in the 15th year
    close(tax("BS", 100_000.0, 0.0, 25 * 12), 12_000.0);
    assert_eq!(tax("BS", 499.0, 0.0, 12), 0.0);
}

/// Art. 72 LI VD (RSF 642.11): 30 % under 1 year, 27 % up to 2, 22 % up to 4, 10 % up to 20, 7 % from 24 years; the
/// years of owner occupation count twice (para. 4); the tax data file assumes owner occupation during the whole holding
/// period (holding period × 2). Gains under 5000 francs are tax-free (Art. 62).
#[test]
fn issue_65_ak4_vd_rate_by_duration_with_double_counting() {
    close(tax("VD", 100_000.0, 0.0, 3), 30_000.0); // 3 months = 6 months counted
    close(tax("VD", 100_000.0, 0.0, 6), 27_000.0); // 6 months = 1 year counted
    close(tax("VD", 100_000.0, 0.0, 18), 22_000.0); // 3 years counted
    close(tax("VD", 100_000.0, 0.0, 9 * 12 + 6), 10_000.0); // 19 years counted
    close(tax("VD", 100_000.0, 0.0, 24 * 12), 7_000.0);
    assert_eq!(tax("VD", 4_999.0, 0.0, 12), 0.0);
}

fn tax_m(canton: &str, gain: f64, months: i32, multiplier: f64) -> f64 {
    property_gain_tax_full(Some(&rule(canton)), gain, 0.0, Some(months), multiplier)
}

/// Aargau (AG), ESTV «Steuermäppchen» 2025 p. 19: rate by holding period 40 % to 5 %.
#[test]
fn issue_65_ak4_ag_hand_values() {
    // gain 100 000, held 30 months: 100 000 × 36 % = 36 000
    close(tax_m("AG", 100000.0, 30, 1.0), 36000.00);
    // gain 100 000, held 15 years: 100 000 × 15 % = 15 000
    close(tax_m("AG", 100000.0, 180, 1.0), 15000.00);
}

/// Appenzell Innerrhoden (AI), ESTV «Steuermäppchen» 2025 p. 18: bracket tariff 10 % to 40 %, surcharge of 1 % per month under 3 years, reduction up to 50 %.
#[test]
fn issue_65_ak4_ai_hand_values() {
    // gain 80 000, held 18 months: 10 % × 4 000 + 15 % × 6 000 + 20 % × 8 000 + 25 % × 12 000 + 30 % × 20 000 + 35 % × 30 000 × (1 +0.18) = 26 432
    close(tax_m("AI", 80000.0, 18, 1.0), 26432.00);
    // gain 80 000, held 12 years: 10 % × 4 000 + 15 % × 6 000 + 20 % × 8 000 + 25 % × 12 000 + 30 % × 20 000 + 35 % × 30 000 × (1 -0.26) = 16 576
    close(tax_m("AI", 80000.0, 144, 1.0), 16576.00);
}

/// Appenzell Ausserrhoden (AR), ESTV «Steuermäppchen» 2025 p. 17: 30 % of the gain, surcharge under 4 years, reduction from 11 years up to 50 %.
#[test]
fn issue_65_ak4_ar_hand_values() {
    // gain 60 000, held 30 months: 30 % × 60 000 × (1 +0.1) = 19 800
    close(tax_m("AR", 60000.0, 30, 1.0), 19800.00);
    // gain 60 000, held 15 years: 30 % × 60 000 × (1 -0.125) = 15 750
    close(tax_m("AR", 60000.0, 180, 1.0), 15750.00);
}

/// Bern (BE), ESTV «Steuermäppchen» 2025 p. 9: simple tax 1.44 % to 8.1 % × tax multiplier, surcharge under 5 years, gain from 5 years 2 % lower per year (at most 70 %).
#[test]
fn issue_65_ak4_be_hand_values() {
    // gain 60 000, held 30 months: 1.44 % × 2 800 + 2.4 % × 2 800 + 4.08 % × 8 200 + 4.92 % × 13 400 + 6.41 % × 26 800 + 7.26 % × 6 000 × (1 +0.35) × tax multiplier 1.8 = 7909.26
    close(tax_m("BE", 60000.0, 30, 1.8), 7909.26);
    // gain 60 000, held 12 years: gain -24 % → 45 600 1.44 % × 2 800 + 2.4 % × 2 800 + 4.08 % × 8 200 + 4.92 % × 13 400 + 6.41 % × 18 400 × tax multiplier 1.8 = 4105.44
    close(tax_m("BE", 60000.0, 144, 1.8), 4105.44);
}

/// Geneva (GE), ESTV «Steuermäppchen» 2025 p. 23: rate by holding period 50 % to 2 %.
#[test]
fn issue_65_ak4_ge_hand_values() {
    // gain 100 000, held 30 months: 100 000 × 40 % = 40 000
    close(tax_m("GE", 100000.0, 30, 1.0), 40000.00);
    // gain 100 000, held 12 years: 100 000 × 10 % = 10 000
    close(tax_m("GE", 100000.0, 144, 1.0), 10000.00);
}

/// Glarus (GL), ESTV «Steuermäppchen» 2025 p. 13: bracket tariff 10 % to 30 %, surcharge under 4 years, reduction up to 90 % from 30 years.
#[test]
fn issue_65_ak4_gl_hand_values() {
    // gain 40 000, held 30 months: 10 % × 5 000 + 15 % × 5 000 + 20 % × 5 000 + 25 % × 5 000 + 30 % × 20 000 × (1 +0.15) = 10 925
    close(tax_m("GL", 40000.0, 30, 1.0), 10925.00);
    // gain 40 000, held 12 years: 10 % × 5 000 + 15 % × 5 000 + 20 % × 5 000 + 25 % × 5 000 + 30 % × 20 000 × (1 -0.26) = 7 030
    close(tax_m("GL", 40000.0, 144, 1.0), 7030.00);
}

/// Grisons (GR), ESTV «Steuermäppchen» 2025 p. 19: bracket tariff 5 % to 25 %, from 208 299 francs 15 %, surcharge under 2 years, reduction up to 51 %.
#[test]
fn issue_65_ak4_gr_hand_values() {
    // gain 120 000, held 10 months: 5 % × 9 919 + 6 % × 9 919 + 7 % × 9 919 + 8 % × 9 919 + 9 % × 9 919 + 10 % × 9 919 + 11 % × 9 919 + 12 % × 9 919 + 13 % × 9 919 + 14 % × 9 919 + 15 % × 9 919 + 16 % × 9 919 + 17 % × 972 × (1 +0.28) = 16208.87
    close(tax_m("GR", 120000.0, 10, 1.0), 16208.87);
    // gain 120 000, held 15 years: 5 % × 9 919 + 6 % × 9 919 + 7 % × 9 919 + 8 % × 9 919 + 9 % × 9 919 + 10 % × 9 919 + 11 % × 9 919 + 12 % × 9 919 + 13 % × 9 919 + 14 % × 9 919 + 15 % × 9 919 + 16 % × 9 919 + 17 % × 972 × (1 -0.075) = 11713.44
    close(tax_m("GR", 120000.0, 180, 1.0), 11713.44);
}

/// Jura (JU), ESTV «Steuermäppchen» 2025 p. 23: simple tax 3.5 % to 6 % × tax multiplier, surcharge under 5 years, reduction from 11 years up to 50 %.
#[test]
fn issue_65_ak4_ju_hand_values() {
    // gain 60 000, held 30 months: 3.5 % × 50 000 + 4.5 % × 10 000 × (1 +0.25) × tax multiplier 1.8 = 4 950
    close(tax_m("JU", 60000.0, 30, 1.8), 4950.00);
    // gain 60 000, held 15 years: 3.5 % × 50 000 + 4.5 % × 10 000 × (1 -0.05) × tax multiplier 1.8 = 3 762
    close(tax_m("JU", 60000.0, 180, 1.8), 3762.00);
}

/// Lucerne (LU), ESTV «Steuermäppchen» 2025 p. 10: income tax tariff (simple tax) × tax multiplier, surcharge under 6 years, reduction from 9 years up to 25 %, at most 40 % of the gain.
#[test]
fn issue_65_ak4_lu_hand_values() {
    // gain 60 000, held 30 months: 0 % × 9 900 + 0.5 % × 2 400 + 1 % × 3 200 + 2 % × 1 100 + 3 % × 1 200 + 4 % × 2 900 + 4.5 % × 4 300 + 5 % × 35 000 × (1 +0.3) × tax multiplier 1.8 = 5057.91
    close(tax_m("LU", 60000.0, 30, 1.8), 5057.91);
    // gain 60 000, held 15 years: 0 % × 9 900 + 0.5 % × 2 400 + 1 % × 3 200 + 2 % × 1 100 + 3 % × 1 200 + 4 % × 2 900 + 4.5 % × 4 300 + 5 % × 35 000 × (1 -0.07) × tax multiplier 1.8 = 3618.35
    close(tax_m("LU", 60000.0, 180, 1.8), 3618.35);
}

/// Neuchâtel (NE), ESTV «Steuermäppchen» 2025 p. 22: bracket tariff 10 % to 40 %, from 135 000 francs 33 %, surcharge under 4 years, reduction up to 60 %.
#[test]
fn issue_65_ak4_ne_hand_values() {
    // gain 100 000, held 30 months: 10 % × 5 000 + 15 % × 5 000 + 20 % × 20 000 + 25 % × 20 000 + 30 % × 25 000 + 35 % × 25 000 × (1 +0.3) = 34 450
    close(tax_m("NE", 100000.0, 30, 1.0), 34450.00);
    // gain 100 000, held 12 years: 10 % × 5 000 + 15 % × 5 000 + 20 % × 20 000 + 25 % × 20 000 + 30 % × 25 000 + 35 % × 25 000 × (1 -0.48) = 13 780
    close(tax_m("NE", 100000.0, 144, 1.0), 13780.00);
}

/// Nidwalden (NW), ESTV «Steuermäppchen» 2025 p. 12: rate by holding period 36 % to 12 %.
#[test]
fn issue_65_ak4_nw_hand_values() {
    // gain 100 000, held 30 months: 100 000 × 31 % = 31 000
    close(tax_m("NW", 100000.0, 30, 1.0), 31000.00);
    // gain 100 000, held 12 years: 100 000 × 21 % = 21 000
    close(tax_m("NW", 100000.0, 144, 1.0), 21000.00);
}

/// Obwalden (OW), ESTV «Steuermäppchen» 2025 p. 11: simple tax 2 % × tax multiplier, surcharge under 3 years.
#[test]
fn issue_65_ak4_ow_hand_values() {
    // gain 60 000, held 18 months: 2 % × 60 000 × (1 +0.2) × tax multiplier 1.8 = 2 592
    close(tax_m("OW", 60000.0, 18, 1.8), 2592.00);
    // gain 60 000, held 12 years: 2 % × 60 000 × tax multiplier 1.8 = 2 160
    close(tax_m("OW", 60000.0, 144, 1.8), 2160.00);
}

/// Schaffhausen (SH), ESTV «Steuermäppchen» 2025 p. 17: simple tax 2 % to 20 % × tax multiplier, surcharge under 5 years, reduction up to 60 %, at most 50 % of the gain.
#[test]
fn issue_65_ak4_sh_hand_values() {
    // gain 60 000, held 30 months: 2 % × 2 000 + 4 % × 2 000 + 6 % × 2 000 + 8 % × 2 000 + 10 % × 7 000 + 12 % × 15 000 + 14 % × 15 000 + 16 % × 15 000 × (1 +0.25) × tax multiplier 1.8 = 16 650
    close(tax_m("SH", 60000.0, 30, 1.8), 16650.00);
    // gain 60 000, held 12 years: 2 % × 2 000 + 4 % × 2 000 + 6 % × 2 000 + 8 % × 2 000 + 10 % × 7 000 + 12 % × 15 000 + 14 % × 15 000 + 16 % × 15 000 × (1 -0.35) × tax multiplier 1.8 = 8 658
    close(tax_m("SH", 60000.0, 144, 1.8), 8658.00);
}

/// Solothurn (SO), ESTV «Steuermäppchen» 2025 p. 14: income tax tariff for single persons × tax multiplier, reduction from 6 years up to 50 %.
#[test]
fn issue_65_ak4_so_hand_values() {
    // gain 60 000, held 30 months: 0 % × 12 000 + 4.5 % × 4 000 + 5 % × 4 000 + 6.5 % × 3 000 + 8 % × 2 000 + 9 % × 3 000 + 9.5 % × 11 000 + 10 % × 15 000 + 10.5 % × 6 000 × tax multiplier 1.8 = 7 524
    close(tax_m("SO", 60000.0, 30, 1.8), 7524.00);
    // gain 60 000, held 12 years: 0 % × 12 000 + 4.5 % × 4 000 + 5 % × 4 000 + 6.5 % × 3 000 + 8 % × 2 000 + 9 % × 3 000 + 9.5 % × 11 000 + 10 % × 15 000 + 10.5 % × 6 000 × (1 -0.14) × tax multiplier 1.8 = 6470.64
    close(tax_m("SO", 60000.0, 144, 1.8), 6470.64);
}

/// Schwyz (SZ), ESTV «Steuermäppchen» 2025 p. 11: bracket tariff 8 % to 30 %, allowance 2 000 francs, surcharge under 4 years, reduction up to 70 %.
#[test]
fn issue_65_ak4_sz_hand_values() {
    // gain 50 000, held 30 months: allowance 2 000 → 48 000 8 % × 3 000 + 12 % × 5 000 + 16 % × 7 000 + 20 % × 10 000 + 25 % × 15 000 + 30 % × 8 000 × (1 +0.2) = 12 132
    close(tax_m("SZ", 50000.0, 30, 1.0), 12132.00);
    // gain 50 000, held 12 years: allowance 2 000 → 48 000 8 % × 3 000 + 12 % × 5 000 + 16 % × 7 000 + 20 % × 10 000 + 25 % × 15 000 + 30 % × 8 000 × (1 -0.31) = 6975.9
    close(tax_m("SZ", 50000.0, 144, 1.0), 6975.90);
}

/// Thurgau (TG), ESTV «Steuermäppchen» 2025 p. 20: 40 % of the gain, surcharge of 1 % per month under 3 years, reduction from the 6th year up to 72 %.
#[test]
fn issue_65_ak4_tg_hand_values() {
    // gain 100 000, held 18 months: 40 % × 100 000 × (1 +0.18) = 47 200
    close(tax_m("TG", 100000.0, 18, 1.0), 47200.00);
    // gain 100 000, held 12 years: 40 % × 100 000 × (1 -0.28) = 28 800
    close(tax_m("TG", 100000.0, 144, 1.0), 28800.00);
}

/// Ticino (TI), ESTV «Steuermäppchen» 2025 p. 20: rate by holding period 31 % to 4 %.
#[test]
fn issue_65_ak4_ti_hand_values() {
    // gain 100 000, held 30 months: 100 000 × 29 % = 29 000
    close(tax_m("TI", 100000.0, 30, 1.0), 29000.00);
    // gain 100 000, held 12 years: 100 000 × 9 % = 9 000
    close(tax_m("TI", 100000.0, 144, 1.0), 9000.00);
}

/// Uri (UR), ESTV «Steuermäppchen» 2025 p. 10: rate by holding period 31 % to 11 %, allowance 10 000 francs.
#[test]
fn issue_65_ak4_ur_hand_values() {
    // gain 100 000, held 30 months: allowance 10 000 → 90 000 90 000 × 29 % = 26 100
    close(tax_m("UR", 100000.0, 30, 1.0), 26100.00);
    // gain 100 000, held 12 years: allowance 10 000 → 90 000 90 000 × 19 % = 17 100
    close(tax_m("UR", 100000.0, 144, 1.0), 17100.00);
}

/// Zurich (ZH), ESTV «Steuermäppchen» 2025 p. 8–9: bracket tariff 10 % to 40 %, surcharge up to 50 % under 2 years, reduction up to 50 % from 20 years.
#[test]
fn issue_65_ak4_zh_hand_values() {
    // gain 100 000, held 18 months: 10 % × 4 000 + 15 % × 6 000 + 20 % × 8 000 + 25 % × 12 000 + 30 % × 20 000 + 35 % × 50 000 × (1 +0.25) = 36 750
    close(tax_m("ZH", 100000.0, 18, 1.0), 36750.00);
    // gain 100 000, held 12 years: 10 % × 4 000 + 15 % × 6 000 + 20 % × 8 000 + 25 % × 12 000 + 30 % × 20 000 + 35 % × 50 000 × (1 -0.26) = 21 756
    close(tax_m("ZH", 100000.0, 144, 1.0), 21756.00);
}

/// St. Gallen (SG), Art. 140 and 141 StG (sGS 811.1): simple tax 0 % to 11 % with a cantonal surcharge of 220 %
/// (factor 3.2), from 600 000 francs 10 % × 3.2 for the whole gain; under 5 years +1 % per missing year; above 15
/// years with owner occupation −1.5 % per further year (at most 40.5 %) on the part of the gain up to 500 000 francs,
/// above it −1 % (at most 20 %). The tax data file assumes owner occupation.
#[test]
fn issue_65_ak4_sg_hand_values() {
    // gain 100 000: 2 800 × 1.6 % + 2 700 × 3.2 % + 2 100 × 6.4 % + 2 100 × 9.6 % + 2 600 × 12.8 % + 3 700 × 16 %
    // + 6 900 × 19.2 % + 9 600 × 22.4 % + 16 000 × 25.6 % + 21 100 × 28.8 % + 28 200 × 32 % = 24 064
    close(tax_m("SG", 100_000.0, 30, 1.0), 24_064.0 * 1.03); // 2.5 years: +3 %
    close(tax_m("SG", 100_000.0, 20 * 12, 1.0), 24_064.0 * (1.0 - 0.075)); // 5 more years: −7.5 %
    close(tax_m("SG", 100_000.0, 60 * 12, 1.0), 24_064.0 * (1.0 - 0.405)); // maximum 40.5 %
                                                                           // gain 700 000 above 600 000: uniformly 32 % = 224 000. Part up to 500 000: −7.5 %, above −5 % (20 years):
                                                                           // 224 000 × (1 − (500/700 × 7.5 % + 200/700 × 5 %)) = 208 800
    close(tax_m("SG", 700_000.0, 20 * 12, 1.0), 208_800.0);
    // maximum values after 40 years: part up to 500 000 −40.5 %, above −20 %
    close(
        tax_m("SG", 700_000.0, 60 * 12, 1.0),
        224_000.0 * (1.0 - (500.0 / 700.0 * 0.405 + 200.0 / 700.0 * 0.20)),
    );
}

/// #65/AK4: every canton of the tax data file has a test in this file.
#[test]
fn issue_65_ak4_every_canton_has_hand_values() {
    let src = include_str!("tax_gain_cantons.rs");
    for c in &taxdata::builtin().cantons {
        let id = c.id.to_lowercase();
        assert!(
            src.contains(&format!("fn issue_65_ak4_{id}_")),
            "canton {} without hand values",
            c.id
        );
    }
}
