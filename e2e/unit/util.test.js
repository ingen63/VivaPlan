// #87: Unit tests of the pure helper functions of the user interface (crates/app/ui/util.js), without a browser.
// Run: `node --test e2e/unit/` (or `make ui-unit`).
const { test } = require("node:test");
const assert = require("node:assert/strict");
require("./tr.js"); // #170: texts from the text table
const u = require("../../crates/app/ui/util.js");

// Thousands separator ' or ’ depending on the ICU version
const T = "['’]";

test("#87/AK2: esc escapes HTML special characters", () => {
  assert.equal(u.esc('<a href="x">&</a>'), "&lt;a href=&quot;x&quot;&gt;&amp;&lt;/a&gt;");
  assert.equal(u.esc(null), "");
  assert.equal(u.esc(5), "5");
});

test("clamp limits to the range", () => {
  assert.equal(u.clamp(5, 0, 3), 3);
  assert.equal(u.clamp(-1, 0, 3), 0);
  assert.equal(u.clamp(2, 0, 3), 2);
});

test("months: ym2i and i2ym are inverse to each other, invalid inputs yield NaN", () => {
  assert.equal(u.ym2i("2030-01"), 2030 * 12);
  assert.equal(u.ym2i("2030-12"), 2030 * 12 + 11);
  assert.ok(Number.isNaN(u.ym2i("2030-1")));
  assert.ok(Number.isNaN(u.ym2i(undefined)));
  for (const s of ["1999-12", "2000-01", "2045-06"]) assert.equal(u.i2ym(u.ym2i(s)), s);
  assert.equal(u.isYM("2030-07"), true);
  assert.equal(u.isYM("07.2030"), false);
  assert.equal(u.isYM(203007), false);
});

test("months: display MM.YYYY and input M.YYYY or MM/YYYY", () => {
  assert.equal(u.ymShow("2030-07"), "07.2030");
  assert.equal(u.ymShow("x"), "—");
  assert.equal(u.parseShow("7.2030"), "2030-07");
  assert.equal(u.parseShow(" 12 / 2031 "), "2031-12");
  assert.equal(u.parseShow("13.2030"), null);
  assert.equal(u.parseShow("0.2030"), null);
  assert.equal(u.parseShow("2030-07"), null);
});

test("quarterEnds counts March, June, September and December", () => {
  const jan = u.ym2i("2030-01");
  assert.equal(u.quarterEnds(jan, jan + 12), 4);
  assert.equal(u.quarterEnds(jan, jan + 2), 0);
  assert.equal(u.quarterEnds(jan, jan + 3), 1);
  assert.equal(u.quarterEnds(jan + 5, jan + 5), 0);
});

test("parseNum reads thousands separators and decimal comma, empty is null", () => {
  assert.equal(u.parseNum("1'234'567"), 1234567);
  assert.equal(u.parseNum("1’234.5"), 1234.5);
  assert.equal(u.parseNum("12,5"), 12.5);
  assert.equal(u.parseNum(" -3 "), -3);
  assert.equal(u.parseNum(""), null);
  assert.equal(u.parseNum("-"), null);
  assert.equal(u.parseNum("."), null);
  assert.equal(u.parseNum("abc"), null);
  assert.equal(u.parseNum("1e400"), null);
});

test("number formats: integers, millions, percent", () => {
  assert.match(u.fmtInt(1234567.4), new RegExp(`^1${T}234${T}567$`));
  assert.equal(u.fmtMio(2_500_000), "2.5 Mio");
  assert.equal(u.fmtMio(-1_000_000), "-1 Mio");
  assert.equal(u.fmtMio(250_000), "250k");
  assert.equal(u.fmtPct(0.05), "5");
  assert.equal(u.fmtPct(0.012345), "1.235");
});

test("optNum and optRound", () => {
  assert.equal(u.optNum("2,5", 1), 2.5);
  assert.equal(u.optNum("", 7), 7);
  assert.equal(u.optNum("x", 7), 7);
  assert.equal(u.optRound(0.1 + 0.2), 0.3);
});

test("rateAt interpolates linearly and continues constantly outside", () => {
  const t = [[100_000, 0.1], [0, 0.0]];
  assert.equal(u.rateAt(t, -5), 0);
  assert.equal(u.rateAt(t, 0), 0);
  assert.equal(u.rateAt(t, 50_000), 0.05);
  assert.equal(u.rateAt(t, 1e9), 0.1);
  assert.equal(u.rateAt([], 1000), 0);
  assert.deepEqual(t, [[100_000, 0.1], [0, 0.0]], "the table stays unchanged");
});

test("#87/AK2: path access getPath and setPath", () => {
  const o = { a: { b: [{ c: 1 }] } };
  assert.equal(u.getPath(o, "a.b.0.c"), 1);
  assert.equal(u.getPath(o, "a.x.y"), undefined);
  assert.equal(u.getPath(null, "a"), undefined);
  u.setPath(o, "a.b.0.c", 5);
  u.setPath(o, "a.d", "neu");
  assert.deepEqual(o, { a: { b: [{ c: 5 }], d: "neu" } });
});

test("file names: baseName and seriesShort", () => {
  assert.equal(u.baseName("/x/y/Planung.vplan"), "Planung.vplan");
  assert.equal(u.baseName("C:\\x\\Planung.vplan"), "Planung.vplan");
  assert.equal(u.baseName(null), "");
  assert.equal(u.seriesShort("library:Shiller CHF.csv"), "Shiller CHF");
  assert.equal(u.seriesShort("/daten/Eigene.CSV"), "Eigene");
});

test("#87/AK2: duration short and as an estimate with a range", () => {
  assert.equal(u.fmtShort(0.4), "unter 1 s");
  assert.equal(u.fmtShort(45.4), "45 s");
  assert.equal(u.fmtShort(89), "89 s");
  assert.equal(u.fmtShort(90), "2 min");
  assert.equal(u.fmtShort(5399), "90 min");
  assert.equal(u.fmtShort(9000), "2.5 h");
  assert.equal(u.fmtEstimate(0.5), "<b>unter 1 Sekunde</b>");
  assert.equal(u.fmtEstimate(20), 'ca. <b>20 s</b> <span class="computed">(meist 12 s bis 32 s)</span>');
  assert.match(u.fmtEstimate(100, [0.7, 1.5]), /ca\. <b>2 min<\/b>.*meist 70 s bis 3 min/);
});

test("#87/AK2: duration estimate of the optimization", () => {
  const b = { us: 100, gen: 20, fixedPaths: null };
  const c = { paths: 1000, checkPaths: 2000, strategies: ["standard", "vpw"] };
  // 50 evaluations × 1000 paths × 80 µs + 1000 × 20 µs + 3 × 2000 × 100 µs = 4.62 s
  assert.equal(u.optSecondsFor(b, c, 50), 4.62);
  // fixed number of paths (rolling time windows) replaces both path counts: 50×300×80 + 300×20 + 3×300×100 µs
  assert.equal(u.optSecondsFor({ ...b, fixedPaths: 300 }, c, 50), 1.296);
  // generating costs more than the whole run: computing share at least 0.1 µs
  assert.ok(u.optSecondsFor({ us: 10, gen: 50, fixedPaths: null }, c, 10) > 0);
});

test("levels of a variation, also over months", () => {
  assert.equal(u.sweepSteps(0.4, 0.6, 0.1), 3);
  assert.equal(u.sweepSteps(0.2, 0.8, 0.3), 3);
  assert.equal(u.sweepSteps(1, 0, 0.1), 0);
  assert.equal(u.sweepSteps(0, 1, 0), 0);
  assert.equal(u.sweepSteps("2030-01", "2031-01", 6, true), 3);
  assert.equal(u.sweepSteps("2031-01", "2030-01", 6, true), 0);
});

test("#118/AK3, AK4: sortBy sorts numbers by value, texts alphabetically, missing values always at the end", () => {
  const v = x => x;
  assert.deepEqual(u.sortBy([10, 9, 100, null, 2], v), [2, 9, 10, 100, null]);
  assert.deepEqual(u.sortBy([10, 9, 100, null, 2], v, -1), [100, 10, 9, 2, null]);
  assert.deepEqual(u.sortBy(["b", undefined, "Ä", "a"], v), ["a", "Ä", "b", undefined]);
  assert.deepEqual(u.sortBy([NaN, 1, "", 3], v, -1), [3, 1, NaN, ""]);
  // «keine Erschöpfung» as Infinity counts as the latest value
  assert.deepEqual(u.sortBy([Infinity, 24000, 23990], v), [23990, 24000, Infinity]);
  // stable: equal values keep their order
  const l = [{ k: 1, n: "x" }, { k: 0, n: "y" }, { k: 1, n: "z" }];
  assert.deepEqual(u.sortBy(l, o => o.k).map(o => o.n), ["y", "x", "z"]);
  assert.deepEqual(u.sortBy(l, o => o.k, -1).map(o => o.n), ["x", "z", "y"]);
});

test("#144/AK2: message on an unknown format identifier names the identifier", () => {
  assert.deepEqual(u.KNOWN_FORMATS, ["vivaplan/1", "pensionsplaner/1"]);
  assert.equal(u.unknownFormatText("vivaplan/2"), "Die Datei hat das Format «vivaplan/2». Sie braucht eine neuere Version von VivaPlan und wird nicht geöffnet.");
});

test("#140/AK1: amount and integer do not take a dot with three digits as a decimal number", () => {
  for (const t of ["1.234", "12.500", "1.234.567", "50.000", "-1.250", "1'234.567"]) {
    assert.equal(u.parseAmount(t), null, t);
    assert.equal(u.ambiguousThousands(t), true, t);
  }
  assert.match(u.ambiguousText(), /Tausendertrennzeichen/);
});

test("#140/AK2: exponent, hex and text notation are rejected in all number fields", () => {
  for (const t of ["0x10", "1e5", "1E5", "Infinity", "-Infinity", "NaN", "12abc", "1.2.3", "--1"]) {
    assert.equal(u.parseNum(t), null, t);
    assert.equal(u.parseAmount(t), null, t);
    assert.equal(u.optNum(t, 7), 7, t);
  }
});

test("#140/AK3: valid notations stay valid", () => {
  const cases = { "1'234.50": 1234.5, "1’234,5": 1234.5, "12,5": 12.5, "1 234": 1234, "-3": -3, "+4": 4, "1234": 1234, "0.5": 0.5, ".5": 0.5, "7258": 7258 };
  for (const [t, v] of Object.entries(cases)) {
    assert.equal(u.parseNum(t), v, t);
    assert.equal(u.parseAmount(t), v, t);
  }
  // percent and year fields read with parseNum: «1.25» stays 1.25, also «12.500» (12.5 %)
  assert.equal(u.parseNum("1.25"), 1.25);
  assert.equal(u.parseNum("12.500"), 12.5);
  assert.equal(u.optNum("2,5", 1), 2.5);
});

// #184: names of the example plan in the language of the user interface
const EXAMPLE = JSON.parse(require("fs").readFileSync(require("path").join(__dirname, "../../crates/engine/assets/example.vplan"), "utf8"));
const TABLES = Object.fromEntries(["de", "en", "fr", "it"].map(l => [l, require(`../../crates/app/ui/i18n/${l}.json`)]));
/** All names and labels of a plan (any depth), with their paths. */
function names(v, at = "", out = {}) {
  if (Array.isArray(v)) v.forEach((x, i) => names(x, `${at}${i}.`, out));
  else if (v && typeof v === "object") for (const [k, x] of Object.entries(v)) {
    if ((k === "name" || k === "label") && typeof x === "string") out[at + k] = x; else names(x, `${at}${k}.`, out);
  }
  return out;
}

test("#184/AK1: every name of the example plan is in the language of the table, no German name stays", () => {
  const german = new Set(Object.entries(TABLES.de).filter(([k]) => k.startsWith("example.")).map(([, v]) => v));
  assert.equal(german.size, 8);
  for (const l of ["en", "fr", "it"]) {
    const got = names(u.localizeExample(EXAMPLE, k => TABLES[l][k]));
    const left = Object.entries(got).filter(([, v]) => german.has(v) || (/Tranche/.test(v) && !/^Tranche \d$/.test(v)));
    assert.deepEqual(left, [], l);
    for (const k of Object.keys(TABLES.de).filter(k => k.startsWith("example."))) assert.ok(Object.values(got).includes(TABLES[l][k]), `${l}: ${k}`);
  }
  // German: the file already holds the German names
  assert.deepEqual(u.localizeExample(EXAMPLE, k => TABLES.de[k]), EXAMPLE);
});

test("#184/AK2: the example plans of the four languages differ only in names", () => {
  const strip = p => JSON.parse(JSON.stringify(p, (k, v) => (k === "name" || k === "label") && typeof v === "string" ? "" : v));
  for (const l of ["en", "fr", "it"]) assert.deepEqual(strip(u.localizeExample(EXAMPLE, k => TABLES[l][k])), strip(EXAMPLE), l);
  assert.equal(u.localizeExample(EXAMPLE, () => undefined).project.name, EXAMPLE.project.name, "missing text keeps the name");
});
