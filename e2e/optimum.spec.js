// Optimaler Parameterwert je Perzentil aus den Variationen (crates/app/ui/optimum.js), ohne Browser.
const { test, expect } = require("@playwright/test");
const Optimum = require("../crates/app/ui/optimum.js");

/** Szenario-Ergebnis mit Perzentilen: `cells` = { prozent: [Endvermögen, Erschöpfungsmonat|null] }. */
const res = (values, cells) => ({
  values,
  percentiles: Object.entries(cells).map(([percent, [endTotal, depleted]]) => ({ percent: +percent, endTotal, depleted })),
});
const one = (param, rows) => ({ params: [param], results: rows.map(([v, cells]) => res([v], cells)) });
const SPEND = "flows.spendingFactor";
const range = [0.5, 0.6, 0.7, 0.8, 0.9, 1.0, 1.1];

test.describe("Optimum je Perzentil", () => {
  test("Ausgabefaktor: höchster tragbarer Wert, zwischen zwei Schritten interpoliert", () => {
    // 1. Perzentil: bei 70 % Endvermögen +30'000, bei 80 % −90'000 (aufgebraucht) → Nullstelle bei 72.5 %
    const sc = one(SPEND, range.map(v => [v, { 1: v <= 0.7 ? [(0.7 - v) * 1e5 + 30_000, null] : [30_000 - (v - 0.7) * 1.2e6, "2046-05"] }]));
    const o = Optimum.find(sc, 1);
    expect(o.items).toHaveLength(1);
    expect(o.items[0].kind).toBe("approx");
    expect(o.items[0].v).toBeCloseTo(0.725, 6);
    expect(o.items[0].reaches).toBe(true);
  });

  test("Ausgabefaktor: Randfälle «reicht nirgends» und «reicht überall»", () => {
    const none = one(SPEND, range.map(v => [v, { 1: [-1000, "2040-01"] }]));
    expect(Optimum.find(none, 1).items[0]).toMatchObject({ kind: "below", v: 0.5, reaches: false });
    const all = one(SPEND, range.map(v => [v, { 1: [500_000, null] }]));
    expect(Optimum.find(all, 1).items[0]).toMatchObject({ kind: "atLeast", v: 1.1 });
  });

  test("Ausgabefaktor: nicht monotone Reihe nimmt den höchsten tragbaren Schritt", () => {
    const sc = one(SPEND, [[0.8, { 5: [10, null] }], [0.9, { 5: [-5, "2045-01"] }], [1.0, { 5: [20, null] }], [1.1, { 5: [-50, "2040-01"] }]]);
    const o = Optimum.find(sc, 5).items[0];
    expect(o.kind).toBe("approx");
    expect(o.v).toBeCloseTo(1.0 + 0.1 * 20 / 70, 9);
  });

  test("Aktienanteil: tragbar vor nicht tragbar, dann höchstes Endvermögen", () => {
    const sc = one("wealth.stockShare", [
      [0.3, { 50: [200_000, null] }], [0.5, { 50: [900_000, null] }], [0.7, { 50: [700_000, null] }], [1.0, { 50: [5_000_000, "2050-01"] }]]);
    const o = Optimum.find(sc, 50).items[0];
    expect(o).toMatchObject({ v: 0.5, kind: "exact", reaches: true });
  });

  test("Aktienanteil: reicht nirgends → spätestes Alter der Erschöpfung; Rand wird gekennzeichnet", () => {
    const sc = one("wealth.stockShare", [
      [0.3, { 1: [-300, "2041-03"] }], [0.5, { 1: [-200, "2044-07"] }], [0.7, { 1: [-100, "2046-01"] }], [1.0, { 1: [-50, "2049-09"] }]]);
    const o = Optimum.find(sc, 1).items[0];
    // 100 % ist die natürliche Grenze eines Anteils, kein ausgeschöpfter Bereich (#34)
    expect(o).toMatchObject({ v: 1.0, kind: "exact", reaches: false });
  });

  // Kriterien: #34/AK1 #34/AK2 #34/AK3 #34/AK4 #34/AK6
  test("#34: Anteil an der natürlichen Grenze 0 % / 100 % ohne «≥» / «≤», sonst weiter Randkennzeichnung", () => {
    const lump = rows => ({ params: ["pension.pk.lumpsum"], results: rows.map(([v, e]) => res([v], { 5: [e, null] })) });
    expect(Optimum.find(lump([[0, 100], [0.5, 200], [1, 900]]), 5).items[0]).toMatchObject({ v: 1, kind: "exact" });
    expect(Optimum.find(lump([[0, 900], [0.5, 200], [1, 100]]), 5).items[0]).toMatchObject({ v: 0, kind: "exact" });
    // Bereich 0 bis 50 %: der Rand 50 % ist keine natürliche Grenze
    expect(Optimum.find(lump([[0, 100], [0.25, 200], [0.5, 900]]), 5).items[0]).toMatchObject({ v: 0.5, kind: "edgeHigh" });
    expect(Optimum.find(lump([[0.5, 900], [0.75, 200], [1, 100]]), 5).items[0]).toMatchObject({ v: 0.5, kind: "edgeLow" });
    // Zwei Variationen: Aktienanteil 100 % neben dem Ausgaben-Faktor
    const two = { params: [SPEND, "wealth.stockShare"], results: [[0.9, 0.5], [0.9, 1], [1, 0.5], [1, 1]].map(([f, a]) => res([f, a], { 5: [a === 1 ? 500 : 100, null] })) };
    expect(Optimum.find(two, 5).items[1]).toMatchObject({ v: 1, kind: "exact" });
    // Ausgaben-Faktor am oberen Rand: weiterhin «≥»
    expect(Optimum.find(two, 5).items[0]).toMatchObject({ kind: "atLeast" });
  });

  test("Kapitalbezug PK: wie Aktienanteil", () => {
    const sc = one("pension.pk.lumpsum", [[0, { 5: [300, null] }], [0.5, { 5: [800, null] }], [1, { 5: [100, null] }]]);
    expect(Optimum.find(sc, 5).items[0]).toMatchObject({ v: 0.5, kind: "exact" });
  });

  test("Parameter ohne sinnvolles Kriterium: kein Optimum (na)", () => {
    for (const p of ["returns.startDate", "milestone.retire", "withdrawal.rate"]) {
      const sc = one(p, [["a", { 1: [1, null] }], ["b", { 1: [2, null] }]]);
      expect(Optimum.find(sc, 1).items[0].kind).toBe("na");
    }
  });

  test("zwei Variationen mit Ausgabefaktor: je Wert des anderen Parameters der höchste tragbare Faktor, dann die beste Kombination", () => {
    // Pro Aktienanteil kippt das 1. Perzentil bei einem anderen Faktor: 30 % → ab 1.0, 60 % → zwischen 0.9 und 1.0, 100 % → ab 0.7
    const limit = { 0.3: 0.85, 0.6: 0.95, 1: 0.65 };
    const results = [];
    for (const share of [0.3, 0.6, 1]) for (const f of [0.5, 0.6, 0.7, 0.8, 0.9, 1.0, 1.1]) {
      const end = (limit[share] - f) * 1e6;
      results.push(res([share, f], { 1: [end, end >= 0 ? null : "2045-01"] }));
    }
    const o = Optimum.find({ params: ["wealth.stockShare", SPEND], results }, 1);
    const [share, f] = o.items;
    expect(share).toMatchObject({ param: "wealth.stockShare", v: 0.6 });
    expect(f.param).toBe(SPEND);
    expect(f.kind).toBe("approx");
    expect(f.v).toBeCloseTo(0.95, 9);
    // Reihenfolge der Parameter spielt keine Rolle
    const swapped = Optimum.find({ params: [SPEND, "wealth.stockShare"], results: results.map(r => ({ ...r, values: [r.values[1], r.values[0]] })) }, 1);
    expect(swapped.items[0]).toMatchObject({ param: SPEND, kind: "approx" });
    expect(swapped.items[1]).toMatchObject({ param: "wealth.stockShare", v: 0.6 });
  });

  test("zwei Variationen mit Ausgabefaktor: reicht nirgends → «<» mit dem tiefsten Wert", () => {
    const results = [];
    for (const share of [0.3, 0.6]) for (const f of [0.5, 0.6]) results.push(res([share, f], { 1: [-100 * share, share === 0.6 ? "2048-01" : "2041-01"] }));
    const o = Optimum.find({ params: ["wealth.stockShare", SPEND], results }, 1);
    expect(o.items[1]).toMatchObject({ kind: "below", v: 0.5, reaches: false });
    expect(o.items[0].v).toBe(0.6); // Der Wert, bei dem es am längsten reicht
  });

  test("zwei Variationen ohne Ausgabefaktor: beste Kombination nach denselben Kriterien", () => {
    const results = [];
    for (const share of [0.3, 0.6, 1]) for (const lump of [0, 0.5, 1]) {
      const end = 1000 - Math.abs(share - 0.6) * 1000 - Math.abs(lump - 0.5) * 400;
      results.push(res([share, lump], { 50: [end, null] }));
    }
    const o = Optimum.find({ params: ["wealth.stockShare", "pension.pk.lumpsum"], results }, 50);
    expect(o.items.map(i => i.v)).toEqual([0.6, 0.5]);
    expect(o.items.every(i => i.kind === "exact")).toBe(true);
  });

  test("fehlende Daten: kein Ergebnis", () => {
    expect(Optimum.find(null, 1)).toBeNull();
    expect(Optimum.find({ params: [SPEND], results: [{ values: [1], error: "x" }] }, 1)).toBeNull();
    expect(Optimum.find({ params: [SPEND], results: [{ values: [1], percentiles: [] }] }, 1)).toBeNull();
  });
});
