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
const AMBIGUOUS_TEXT = "Mehrdeutige Zahl: Ist der Punkt ein Tausendertrennzeichen? Bitte ohne Punkt (1234) oder mit Apostroph (1'234) eingeben.";
const fmtInt = v => Math.round(v).toLocaleString("de-CH");
function fmtMio(v) { return Math.abs(v) >= 1e6 ? `${(v / 1e6).toLocaleString("de-CH", { maximumFractionDigits: 2 })} Mio` : `${fmtInt(v / 1000)}k`; }
/** Share as a percentage without sign (0.05 → «5»). */
const fmtPct = v => `${(v * 100).toLocaleString("de-CH", { maximumFractionDigits: 3 })}`;
/** Number from a field of the optimization; otherwise the default `d`. */
const optNum = (v, d) => { const x = parseNum(v); return x === null ? d : x; }; // #140: like parseNum (no «1e5», «0x10»)
const optRound = x => Math.round(x * 1e6) / 1e6;
/** Tax rate of a rate table [[amount, rate], …]: linear between the steps, constant outside. */
function rateAt(table, x) {
  const t = [...table].sort((a, b) => a[0] - b[0]);
  if (!t.length || x <= t[0][0]) return t.length ? t[0][1] : 0;
  for (let i = 1; i < t.length; i++) if (x < t[i][0]) { const [x0, r0] = t[i - 1], [x1, r1] = t[i]; return r0 + (r1 - r0) * (x - x0) / (x1 - x0); }
  return t[t.length - 1][1];
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
  if (sec < 1) return "unter 1 s";
  if (sec < 90) return `${Math.round(sec)} s`;
  if (sec < 5400) return `${Math.round(sec / 60)} min`;
  return `${(sec / 3600).toLocaleString("de-CH", { maximumFractionDigits: 1 })} h`;
}
/** «ca. 12 s (7 s bis 20 s)»: rough estimate with a range, because the search steps of the optimization depend on the case. */
const fmtEstimate = (sec, spread = [0.6, 1.6]) => sec < 1 ? "<b>unter 1 Sekunde</b>" : `ca. <b>${fmtShort(sec)}</b> <span class="computed">(meist ${fmtShort(sec * spread[0])} bis ${fmtShort(sec * spread[1])})</span>`;
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
const unknownFormatText = f => `Die Datei hat das Format «${f}». Sie braucht eine neuere Version von VivaPlan und wird nicht geöffnet.`;

if (typeof module !== "undefined") {
  module.exports = {
    esc, clamp, isYM, ym2i, i2ym, ymShow, parseShow, quarterEnds, parseNum, fmtInt, fmtMio, fmtPct, optNum, optRound,
    rateAt, getPath, setPath, baseName, seriesShort, fmtShort, fmtEstimate, optSecondsFor, sweepSteps, sortBy,
    KNOWN_FORMATS, unknownFormatText, ambiguousThousands, parseAmount, AMBIGUOUS_TEXT,
  };
}
