// #156: Shared tax cases (crates/engine/tests/fixtures/tax-cases.json) against rateAt and taxAt of the user interface.
// The same cases run in the engine (crates/engine/tests/tax_parity.rs), so both compute the tax alike.
// Run: `node --test e2e/unit/` (or `make ui-unit`).
const { test } = require("node:test");
const assert = require("node:assert/strict");
require("./tr.js");
const u = require("../../crates/app/ui/util.js");
const { cases } = require("../../crates/engine/tests/fixtures/tax-cases.json");

test("#156/AK1: the user interface computes the shared tax cases like the engine", () => {
  assert.ok(cases.length >= 10);
  for (const c of cases) {
    const rate = u.rateAt(c.table, c.amount);
    assert.ok(Math.abs(rate - c.rate) < 1e-12, `${c.name}: rate ${rate}`);
    const got = "multiplier" in c ? u.taxAt(c.table, c.amount, c.multiplier) : u.taxAt(c.table, c.amount);
    if (c.tax === null) assert.ok(Number.isNaN(got), `${c.name}: no tax for an invalid multiplier, got ${got}`);
    else assert.ok(Math.abs(got - c.tax) < 1e-9, `${c.name}: tax ${got}, expected ${c.tax}`);
  }
});

test("#156/AK2: a tax multiplier 0 gives no tax, a missing one counts as 1", () => {
  const t = [[0, 0], [100_000, 0.1]];
  assert.equal(u.taxAt(t, 200_000, 0), 0);
  assert.equal(u.taxAt(t, 200_000), 20_000);
  assert.ok(Number.isNaN(u.taxAt(t, 200_000, null)));
});

test("#156: round2 rounds ties to the even centime like the engine", () => {
  assert.equal(u.round2(0.125), 0.12);
  assert.equal(u.round2(0.375), 0.38);
  assert.equal(u.round2(12.357345), 12.36);
  assert.equal(u.round2(-0.125), -0.12);
  assert.equal(u.round2(0), 0);
});
