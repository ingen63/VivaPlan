// #87: Pure helper functions of the user interface (without DOM and without plan state). In the browser an ordinary
// script before app.js; the functions are then global. In Node, `node --test` checks them via module.exports
// (tests in e2e/unit/util.test.js; not under ui/, because the app ships that folder).
"use strict";

/** Escapes HTML special characters. */
const esc = s => String(s ?? "").replace(/[&<>"]/g, c => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" }[c]));
const clamp = (v, a, b) => Math.min(b, Math.max(a, v));

// ---------- Months ----------
const isYM = s => typeof s === "string" && /^\d{4}-\d{2}$/.test(s);
/** «YYYY-MM» as a running month number (year × 12 + month − 1); otherwise NaN. */
function ym2i(s) { const m = /^(\d{4})-(\d{2})$/.exec(s || ""); return m ? (+m[1]) * 12 + (+m[2] - 1) : NaN; }
function i2ym(i) { const y = Math.floor(i / 12), mo = i - y * 12 + 1; return `${y}-${String(mo).padStart(2, "0")}`; }
/** «YYYY-MM» as «MM.YYYY» for display. */
function ymShow(s) { return isYM(s) ? `${s.slice(5)}.${s.slice(0, 4)}` : "—"; }
/** Input «M.YYYY» or «MM/YYYY» as «YYYY-MM»; invalid: null. */
function parseShow(s) {
  const m = /^\s*(\d{1,2})\s*[.\/]\s*(\d{4})\s*$/.exec(s || "");
  if (!m || +m[1] < 1 || +m[1] > 12) return null;
  return `${m[2]}-${String(+m[1]).padStart(2, "0")}`;
}
/** Number of quarter ends (March, June, September, December) in the months a to b − 1. */
function quarterEnds(a, b) { let n = 0; for (let m = a; m < b; m++) if (m % 3 === 2) n++; return n; }

// ---------- Numbers ----------
/** Number from an input with thousands separators (’ ' space) and decimal comma; empty or invalid: null.
 *  #140: only digits with at most one decimal separator, no exponent or hex notation («1e5», «0x10»). */
function parseNum(s) {
  s = String(s).replace(/[’'\s]/g, "").replace(",", ".");
  if (!/^[+-]?(\d+\.?\d*|\.\d+)$/.test(s)) return null;
  const v = Number(s); return Number.isFinite(v) ? v : null;
}
/** #140: A point followed by exactly three digits («1.234», «12.500») is ambiguous in an amount or an integer:
 *  usually a thousands separator. Such inputs are not silently taken as a decimal number. */
const ambiguousThousands = s => /\d\.\d{3}(?!\d)/.test(String(s).replace(/[’'\s]/g, ""));
/** Amount (CHF) or integer (e.g. number of paths): like `parseNum`, but ambiguous inputs yield null. */
function parseAmount(s) { return ambiguousThousands(s) ? null : parseNum(s); }
const ambiguousText = () => tr("input.ambiguous");
const fmtInt = v => Math.round(v).toLocaleString("de-CH");
/** #143/AK2: Taxes are estimates; the user interface shows them rounded to 100 francs (the engine computes exactly). */
const roundTax = v => Math.round(v / 100) * 100 || 0;
const fmtTax = v => fmtInt(roundTax(v));
/** #143/AK2: Engine message with the parameter `tax` rounded to 100 francs for display (events of payouts and sales). */
/** #232: The engine reports earned income per month; the user interface names it per year (amount × 12) with its own
 *  text (`textKey`), also inside a nested message. The engine, its texts and the golden files stay per month. */
const YEARLY_MESSAGES = { income_after_pk: "flows.incomeAfterPk", income_after_pk_indexed: "flows.incomeAfterPkIndexed" };
function incomeYearly(m) {
  if (!m || typeof m !== "object" || !m.params || typeof m.params !== "object") return m;
  const params = Object.fromEntries(Object.entries(m.params).map(([k, v]) => [k, v && typeof v === "object" && !Array.isArray(v) ? incomeYearly(v) : v]));
  const key = YEARLY_MESSAGES[m.code];
  if (key && typeof params.amount === "number") return { ...m, params: { ...params, amount: params.amount * 12 }, textKey: key };
  return { ...m, params };
}
const taxRounded = m => m && typeof m === "object" && m.params && typeof m.params.tax === "number"
  ? { ...m, params: { ...m.params, tax: roundTax(m.params.tax) } } : m;
function fmtMio(v) { return Math.abs(v) >= 1e6 ? tr("fmt.mio", { v: (v / 1e6).toLocaleString("de-CH", { maximumFractionDigits: 2 }) }) : `${fmtInt(v / 1000)}k`; }
/** Share as a percentage without sign (0.05 → «5»). */
const fmtPct = v => `${(v * 100).toLocaleString("de-CH", { maximumFractionDigits: 3 })}`;
/** Number from a field of the optimization; otherwise the default `d`. */
const optNum = (v, d) => { const x = parseNum(v); return x === null ? d : x; }; // #140: like parseNum (no «1e5», «0x10»)
const optRound = x => Math.round(x * 1e6) / 1e6;
/** Tax rate of a rate table [[amount, rate], …]: linear between the steps, constant outside. Like `tax::rate_at` in the
 * engine (#156): on a step with several rows the first applies, right after it the last. */
function rateAt(table, x) {
  const t = [...table].sort((a, b) => a[0] - b[0]);
  if (!t.length || x <= t[0][0]) return t.length ? t[0][1] : 0;
  for (let i = 1; i < t.length; i++) {
    const [x0, r0] = t[i - 1], [x1, r1] = t[i];
    if (x <= x1) return x1 === x0 ? r1 : r0 + (r1 - r0) * (x - x0) / (x1 - x0);
  }
  return t[t.length - 1][1];
}
/** Rounds to centimes, ties to the even centime (like `tax::round2` in the engine). */
function round2(v) {
  const y = v * 100, f = Math.floor(y), d = y - f;
  return (d > 0.5 || (d === 0.5 && f % 2 !== 0) ? f + 1 : f) / 100;
}
/** #156: Tax = amount × rate × tax multiplier, rounded to centimes, like `tax::tax` in the engine. A missing multiplier
 * counts as 1 (default of the engine), a multiplier 0 gives no tax; a value that is not a number the engine refuses,
 * here the result is NaN. */
function taxAt(table, x, multiplier) {
  const m = multiplier === undefined ? 1 : typeof multiplier === "number" ? multiplier : NaN;
  if (x < 0) return 0;
  return round2(x * rateAt(table, x) * m);
}

// ---------- Paths and file names ----------
/** Value under a dot path («a.b.0.c»); if a link is missing: undefined. */
function getPath(obj, path) { return path.split(".").reduce((o, k) => (o == null ? undefined : o[k]), obj); }
/** Sets the value under a dot path; the links before it must exist. */
function setPath(obj, path, v) {
  const ks = path.split("."); let o = obj;
  for (let i = 0; i < ks.length - 1; i++) o = o[ks[i]];
  o[ks[ks.length - 1]] = v;
}
const baseName = p => String(p || "").split(/[\\/]/).pop();
/** Display name of a time series: without folder, the internal prefix «library:» and the extension «.csv». */
const seriesShort = p => baseName(String(p || "").replace(/^library:/, "")).replace(/\.csv$/i, "");

// ---------- Duration ----------
/** Short duration in seconds: «unter 1 s», «45 s», «12 min», «2,5 h». */
function fmtShort(sec) {
  if (sec < 1) return tr("fmt.underOneSecond");
  if (sec < 90) return `${Math.round(sec)} s`;
  if (sec < 5400) return `${Math.round(sec / 60)} min`;
  return `${(sec / 3600).toLocaleString("de-CH", { maximumFractionDigits: 1 })} h`;
}
/** «ca. 12 s (7 s bis 20 s)»: rough estimate with a range, because the search steps of the optimization depend on the case. */
const fmtEstimate = (sec, spread = [0.6, 1.6]) => sec < 1 ? tr("fmt.underOneSecondLong") : `${tr("fmt.approx")} <b>${fmtShort(sec)}</b> <span class="computed">${tr("fmt.estimateRange", { short: fmtShort(sec * spread[0]), short2: fmtShort(sec * spread[1]) })}</span>`;
/**
 * Estimated duration of the optimization in seconds from the measurement `b` ({ us, gen, fixedPaths }: µs per run, of which generating
 * the path, fixed number of paths), the settings `c` ({ paths, checkPaths, strategies }) and the sum of the
 * evaluations over all strategies. The search paths are created once, the check paths anew for every check.
 */
function optSecondsFor(b, c, evals) {
  const fit = b.fixedPaths ?? c.paths, check = b.fixedPaths ?? c.checkPaths, n = c.strategies.length;
  const sim = Math.max(0.1, b.us - b.gen);
  return (evals * fit * sim + fit * b.gen + (n + 1) * check * b.us) / 1e6;
}
/** Number of steps of a variation from `from` to `to` with step `step` (months as «YYYY-MM» if `month`). */
function sweepSteps(from, to, step, month = false) {
  if (month) { const a = ym2i(from), b = ym2i(to); return a <= b && step > 0 ? Math.floor((b - a) / step) + 1 : 0; }
  return to >= from && step > 0 ? Math.floor((to - from) / step + 1e-9) + 1 : 0;
}
/**
 * #118: Sorts a list stably by a value. `dir` 1 ascending, −1 descending. Texts alphabetically (de-CH), numbers
 * by value. Missing values (null, undefined, NaN, "") are at the end in both directions.
 */
function sortBy(list, valOf, dir = 1) {
  const miss = v => v == null || v === "" || (typeof v === "number" && Number.isNaN(v));
  return list.map((x, i) => ({ x, i, v: valOf(x) })).sort((a, b) => {
    const ma = miss(a.v), mb = miss(b.v);
    if (ma || mb) return ma === mb ? a.i - b.i : ma ? 1 : -1;
    const c = typeof a.v === "string" || typeof b.v === "string" ? String(a.v).localeCompare(String(b.v), "de-CH") : a.v < b.v ? -1 : a.v > b.v ? 1 : 0;
    return c * dir || a.i - b.i;
  }).map(o => o.x);
}

// ---------- File format (#144) ----------
/** Format ids this version opens; «pensionsplaner/1» is the old id (legacy: RetirementPlanner). */
const KNOWN_FORMATS = ["vivaplan/1", "pensionsplaner/1"];
/** Message for an unknown format id; the same text as in the engine (`model::unknown_format`). */
const unknownFormatText = f => tr("file.unknownFormat", { f });

// #171: Messages of the engine `{code, params, text}`. The text comes from the text table (key `engine.<code>`), formatted
// exactly as the engine renders its German text (crates/engine/src/message.rs): placeholder `{name}` or `{name:format}`;
// without a format a text as is, a number as printed by the engine, a list joined by ", ", a nested message from its
// own template; `int` rounded with thousands separator ', `fix0`–`fix2` fixed decimals (ties to even, as in Rust),
// `pct0`–`pct2` the same for a fraction × 100, `ym` a month YYYY-MM as MM.YYYY, `lines` a list one per line.
// Unknown code (or no table entry): the text sent along. `lookup(key)` returns the template or undefined.

/** Rounded integer with thousands separator ' (12'500), like `message::group` of the engine. */
function groupInt(v) {
  const n = Math.sign(v) * Math.round(Math.abs(v));
  return (n < 0 ? "-" : "") + String(Math.abs(n)).replace(/\B(?=(\d{3})+(?!\d))/g, "'");
}

/** `x.toFixed(d)`, but an exact tie is rounded to the even digit like `format!("{:.d}")` in Rust. */
function fixedEven(x, d) {
  const r = x.toFixed(d);
  if (!Number.isFinite(x)) return r;
  const long = x.toFixed(d + 60), dot = long.indexOf("."), keep = d === 0 ? dot : dot + 1 + d;
  return /^50*$/.test(long.slice(d === 0 ? dot + 1 : keep)) && /[13579]$/.test(r) ? long.slice(0, keep) : r;
}

function engineValue(v, format, lookup) {
  if (v === null || v === undefined) return "";
  if (Array.isArray(v)) return v.map(x => engineValue(x, format, lookup)).join(format === "lines" ? "\n" : ", ");
  if (typeof v === "object") return engineText(v, lookup);
  if (typeof v === "string") {
    const i = v.indexOf("-");
    return format === "ym" && i >= 0 ? `${v.slice(i + 1)}.${v.slice(0, i)}` : v;
  }
  if (typeof v !== "number") return String(v);
  if (format === "int") return groupInt(v);
  const f = /^(fix|pct)([012])$/.exec(format);
  return f ? fixedEven(f[1] === "pct" ? v * 100 : v, +f[2]) : String(v);
}

function engineText(m, lookup) {
  if (m === null || m === undefined) return "";
  if (typeof m !== "object") return String(m);
  const template = m.code ? lookup(m.textKey ?? `engine.${m.code}`) : undefined;
  if (template === undefined) return String(m.text ?? m.message ?? "");
  const params = m.params || {};
  return template.replace(/\{(\w+)(?::(\w+))?\}/g, (all, k, f) => (k in params ? engineValue(params[k], f || "", lookup) : all));
}

/**
 * #184: Names of the example plan in the language of the user interface. The plan file holds the German names; on
 * opening they are replaced by the texts `example.*` (`lookup` = key → text). Only names change, never a number, so
 * the example plan gives the same results in every language. Tranches keep «Tranche 1/2» (the same in all languages).
 * A saved plan is never passed through here, so its names stay as they are.
 */
const EXAMPLE_NAMES = [
  ["project.name", "example.name"],
  ["household.persons.0.name", "example.person"],
  ["flows.extras.0.label", "example.car"],
  ["flows.extras.1.label", "example.trip"],
  ["realestate.properties.0.name", "example.house"],
  ["realestate.properties.1.name", "example.condo"],
  ["realestate.properties.2.name", "example.rental"],
];
function localizeExample(plan, lookup) {
  const p = JSON.parse(JSON.stringify(plan));
  const put = (path, key) => { if (typeof getPath(p, path) === "string" && lookup(key) !== undefined) setPath(p, path, lookup(key)); };
  for (const [path, key] of EXAMPLE_NAMES) put(path, key);
  const i = (p.milestones || []).findIndex(m => m.id === "sell_house");
  if (i >= 0) put(`milestones.${i}.name`, "example.age75");
  return p;
}

if (typeof module !== "undefined") {
  module.exports = {
    localizeExample,
    esc, clamp, isYM, ym2i, i2ym, ymShow, parseShow, quarterEnds, parseNum, fmtInt, roundTax, fmtTax, taxRounded, fmtMio, fmtPct, optNum, optRound,
    rateAt, round2, taxAt, getPath, setPath, baseName, seriesShort, fmtShort, fmtEstimate, optSecondsFor, sweepSteps, sortBy,
    KNOWN_FORMATS, unknownFormatText, ambiguousThousands, parseAmount, ambiguousText, engineText, groupInt, fixedEven, incomeYearly,
  };
}
