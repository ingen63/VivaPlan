//! #306: Which amounts of a plan grow with inflation.
//!
//! As in reality, these amounts are in prices of the simulation start and grow with the inflation of the path: living
//! costs, care home, earned income, pension fund contributions, pillar 3a contribution and maximum, AHV pension, tax
//! tables and ongoing deductions, one-off amounts, one-off deductions from income and wealth, value and fix costs of
//! the properties. A rent grows until the month of moving in and then stays nominal. Always nominal: pension fund
//! pension, mortgage, pension fund and pillar 3a balances with their interest.
//!
//! Each of the amounts above has a flag; `false` is an exception «nominal, without inflation» (stress tests, under
//! «Experte»). With [`Plan::indexed_by_default`] a missing flag means «with inflation». Older plans without the marker
//! keep the defaults from before #306 (so that their results and the golden files stay the same); [`crate::upgrade`]
//! writes those defaults as explicit flags and sets the marker.

use crate::model::Plan;

/// How a rent follows inflation.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RentIndexation {
    /// #306/AK3: the rent (prices of the simulation start) grows until the month of moving in, then stays nominal.
    UntilMoveIn,
    /// The rent grows with inflation throughout (older plans).
    Always,
    /// The rent stays nominal throughout.
    Nominal,
}

/// Inflation of the amounts of a property.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PropertyIndexation {
    pub value: bool,
    pub fix_costs: bool,
    pub rent: RentIndexation,
}

/// Which amounts grow with inflation (`true`) and which stay nominal.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Indexation {
    pub spending: bool,
    /// Care home per person (`household.persons`).
    pub care: Vec<bool>,
    /// Earned income: first person, partner.
    pub income: [bool; 2],
    /// Pension fund contributions: first person, partner.
    pub pk_contribution: [bool; 2],
    /// AHV pension: first person, partner.
    pub ahv: [bool; 2],
    /// Pillar 3a contribution per account (`pillar3a`).
    pub pillar3a: Vec<bool>,
    pub pillar3a_max: bool,
    /// Tax tables and ongoing deductions from income and wealth.
    pub taxes: bool,
    pub extras: bool,
    pub one_off: bool,
    pub one_off_wealth: bool,
    /// Per property (`realestate.properties`).
    pub properties: Vec<PropertyIndexation>,
}

impl Indexation {
    /// Rules of `plan`.
    pub fn of(plan: &Plan) -> Self {
        let new = plan.indexed_by_default;
        // A flag that is missing: «with inflation» with the marker, otherwise the earlier default `legacy`.
        let flag = |f: Option<bool>, legacy: bool| f.unwrap_or(new || legacy);
        let partner = plan.partner.as_ref();
        let taxes = flag(plan.taxes.indexed, false);
        Indexation {
            spending: flag(plan.flows.spending_indexed, true),
            care: plan
                .household
                .persons
                .iter()
                .map(|p| p.care_home.as_ref().is_none_or(|c| flag(c.indexed, true)))
                .collect(),
            income: [
                flag(plan.flows.income_indexed, false),
                flag(partner.and_then(|q| q.income_indexed), false),
            ],
            pk_contribution: [
                flag(plan.pension.pk.contribution_indexed, false),
                flag(partner.and_then(|q| q.pk.contribution_indexed), false),
            ],
            ahv: [
                flag(plan.pension.ahv.indexed, false),
                flag(partner.and_then(|q| q.ahv.indexed), false),
            ],
            pillar3a: plan
                .pillar3a
                .iter()
                .map(|a| flag(a.contribution_indexed, false))
                .collect(),
            pillar3a_max: flag(plan.taxes.pillar3a_max_indexed, taxes),
            taxes,
            extras: flag(plan.flows.extras_indexed, false),
            one_off: flag(plan.taxes.one_off_indexed, false),
            one_off_wealth: flag(plan.taxes.one_off_wealth_indexed, false),
            properties: plan
                .realestate
                .properties
                .iter()
                .map(|p| PropertyIndexation {
                    value: flag(p.inflation_value, p.inflation),
                    fix_costs: flag(p.inflation_fix_costs, p.inflation),
                    rent: match (p.inflation_rent, new) {
                        (Some(false), _) => RentIndexation::Nominal,
                        (None, true) => RentIndexation::UntilMoveIn,
                        _ => RentIndexation::Always,
                    },
                })
                .collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn plan(marker: bool) -> Plan {
        let mut v = crate::api::example_plan();
        v["indexedByDefault"] = json!(marker);
        for (ptr, key) in [
            ("/flows", "incomeIndexed"),
            ("/taxes", "indexed"),
            ("/pension/ahv", "indexed"),
            ("/partner", "incomeIndexed"),
            ("/partner/ahv", "indexed"),
        ] {
            if let Some(o) = v.pointer_mut(ptr).and_then(|x| x.as_object_mut()) {
                o.remove(key);
            }
        }
        for p in v["realestate"]["properties"].as_array_mut().unwrap() {
            let o = p.as_object_mut().unwrap();
            for k in ["inflation", "inflationValue", "inflationFixCosts", "inflationRent"] {
                o.remove(k);
            }
        }
        serde_json::from_value(v).unwrap()
    }

    /// #306/AK6: Without flags a new plan has no exception.
    #[test]
    fn issue_306_ak6_marker_without_flags_indexes_everything() {
        let ix = Indexation::of(&plan(true));
        assert!(ix.spending && ix.taxes && ix.extras && ix.one_off && ix.one_off_wealth && ix.pillar3a_max);
        assert_eq!(ix.income, [true, true]);
        assert_eq!(ix.pk_contribution, [true, true]);
        assert_eq!(ix.ahv, [true, true]);
        assert!(ix.pillar3a.iter().all(|&b| b));
        assert!(ix
            .properties
            .iter()
            .all(|p| p.value && p.fix_costs && p.rent == RentIndexation::UntilMoveIn));
    }

    /// #306/AK5: Without the marker the defaults from before #306 apply.
    #[test]
    fn issue_306_ak5_without_marker_keeps_the_old_defaults() {
        let ix = Indexation::of(&plan(false));
        assert!(ix.spending, "living costs grew with inflation before #306");
        assert!(!ix.taxes && !ix.extras && !ix.one_off && !ix.one_off_wealth && !ix.pillar3a_max);
        assert_eq!(ix.income, [false, false]);
        assert_eq!(ix.pk_contribution, [false, false]);
        assert_eq!(ix.ahv, [false, false]);
        assert!(ix
            .properties
            .iter()
            .all(|p| !p.value && !p.fix_costs && p.rent == RentIndexation::Always));
    }

    /// #306/AK4: A flag `false` is an exception with and without the marker; the 3a maximum follows the tariffs
    /// without the marker.
    #[test]
    fn issue_306_ak4_false_is_an_exception() {
        let mut p = plan(true);
        p.flows.spending_indexed = Some(false);
        p.taxes.indexed = Some(false);
        let ix = Indexation::of(&p);
        assert!(!ix.spending && !ix.taxes && ix.pillar3a_max);
        let mut old = plan(false);
        old.taxes.indexed = Some(true);
        assert!(Indexation::of(&old).pillar3a_max);
        if let Some(pr) = old.realestate.properties.first_mut() {
            pr.inflation_rent = Some(false);
            assert_eq!(Indexation::of(&old).properties[0].rent, RentIndexation::Nominal);
        }
    }
}
