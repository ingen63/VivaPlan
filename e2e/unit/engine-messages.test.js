// #171: Messages of the engine in the user interface. The engine writes an example of every code (code, English
// template, parameters, English text since #176) to engine-messages.json (Rust test
// `issue_171_ak3_examples_file_is_current`); here `engineText` of util.js renders each example with the English text
// table and must give exactly the text of the engine.
// Run: `node --test e2e/unit/` (or `make ui-unit`).
const { test } = require("node:test");
const assert = require("node:assert/strict");
const { de } = require("./tr.js");
const en = require("../../crates/app/ui/i18n/en.json");
const u = require("../../crates/app/ui/util.js");
const examples = require("./engine-messages.json");

const lookup = table => key => table[key];

test("#176/AK1: the English table renders every example to the text of the engine", () => {
  assert.ok(examples.length > 300, `${examples.length}`);
  const wrong = examples
    .map(e => ({ code: e.code, got: u.engineText({ code: e.code, params: e.params, text: "" }, lookup(en)), want: e.text }))
    .filter(x => x.got !== x.want);
  assert.deepEqual(wrong, []);
});

test("#171/AK3: every code has a template in the table, and every engine key has a code", () => {
  const codes = new Set(examples.map(e => e.code));
  assert.deepEqual(examples.filter(e => !(`engine.${e.code}` in de)).map(e => e.code), []);
  assert.deepEqual(examples.filter(e => en[`engine.${e.code}`] !== e.template).map(e => e.code), []);
  assert.deepEqual(Object.keys(de).filter(k => k.startsWith("engine.") && !codes.has(k.slice(7))), []);
});

test("#171/AK2: an unknown code or a missing template shows the text sent along", () => {
  assert.equal(u.engineText({ code: "no_such_code", params: {}, text: "Text des Rechenkerns" }, lookup(de)), "Text des Rechenkerns");
  assert.equal(u.engineText({ code: "event_extra", params: { label: { code: "gone", text: "alt" }, amount: 5, currency: "CHF" } }, lookup(de)), "alt: 5 CHF");
  assert.equal(u.engineText({ message: "Meldung ohne Code" }, lookup(de)), "Meldung ohne Code");
  assert.equal(u.engineText("Zeichenkette", lookup(de)), "Zeichenkette");
  assert.equal(u.engineText(null, lookup(de)), "");
});

test("#171/AK2: numbers in the Swiss format, as the engine renders them", () => {
  const t = { "engine.x": "{n} {x:int} {x:fix0} {r:pct1} {r:pct2} {r:pct0} {m:ym} {s} {l} {neg:int} {nested} {gone} {z:lines}", "engine.cancelled": "Abgebrochen" };
  const m = {
    code: "x",
    params: { n: 3, x: 1234567.5, r: 0.0425, m: "2031-07", s: "Anna", l: ["A", "B"], neg: -2500.4, nested: { code: "cancelled", params: {} }, z: ["a", "b"] },
  };
  // ties are rounded to the even digit like `format!` in Rust: 1234567.5 → 1234568, 4.25 → 4.2
  assert.equal(u.engineText(m, lookup(t)), "3 1'234'568 1234568 4.2 4.25 4 07.2031 Anna A, B -2'500 Abgebrochen {gone} a\nb");
  assert.equal(u.fixedEven(2.5, 0), "2");
  assert.equal(u.fixedEven(3.5, 0), "4");
  assert.equal(u.fixedEven(0.125, 2), "0.12");
  assert.equal(u.fixedEven(1.005, 2), "1.00");
  assert.equal(u.groupInt(-0.4), "0");
  assert.equal(u.groupInt(999.5), "1'000");
});

test("#232/AK5: earned income in messages per year, also inside a nested message; the engine stays per month", () => {
  const m = { code: "income_after_pk", params: { at: "2031-07", amount: 2500 }, text: "" };
  assert.equal(u.engineText(u.incomeYearly(m), lookup(de)), "Das Erwerbseinkommen beträgt nach dem Bezug der Pensionskasse (07.2031) noch 30'000 pro Jahr.");
  assert.equal(u.engineText(u.incomeYearly({ ...m, code: "income_after_pk_indexed" }), lookup(en)),
    "After the pension fund payout (07.2031), earned income is still 30'000 per year in prices at the simulation start.");
  const labeled = { code: "labeled", params: { what: "B", problem: m } };
  assert.match(u.engineText(u.incomeYearly(labeled), lookup(de)), /^B: .* noch 30'000 pro Jahr\.$/);
  // without the conversion the template of the engine (per month) applies, and the message itself is not changed
  assert.equal(u.engineText(m, lookup(en)), "After the pension fund payout (07.2031), earned income is still 2'500 per month.");
  assert.equal(m.params.amount, 2500);
  assert.equal(u.incomeYearly(null), null);
});
