// FEA-69: Reine Hilfsfunktionen der Oberfläche (ohne DOM und ohne Zustand der Planung). Im Browser ein gewöhnliches
// Skript vor app.js; die Funktionen sind dann global. In Node prüft `node --test` sie über module.exports
// (Tests in e2e/unit/util.test.js; nicht unter ui/, weil die App diesen Ordner ausliefert).
"use strict";

/** HTML-Sonderzeichen maskieren. */
const esc = s => String(s ?? "").replace(/[&<>"]/g, c => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" }[c]));
const clamp = (v, a, b) => Math.min(b, Math.max(a, v));

// ---------- Monate ----------
const isYM = s => typeof s === "string" && /^\d{4}-\d{2}$/.test(s);
/** «JJJJ-MM» als fortlaufende Monatsnummer (Jahr × 12 + Monat − 1); sonst NaN. */
function ym2i(s) { const m = /^(\d{4})-(\d{2})$/.exec(s || ""); return m ? (+m[1]) * 12 + (+m[2] - 1) : NaN; }
function i2ym(i) { const y = Math.floor(i / 12), mo = i - y * 12 + 1; return `${y}-${String(mo).padStart(2, "0")}`; }
/** «JJJJ-MM» als «MM.JJJJ» für die Anzeige. */
function ymShow(s) { return isYM(s) ? `${s.slice(5)}.${s.slice(0, 4)}` : "—"; }
/** Eingabe «M.JJJJ» oder «MM/JJJJ» als «JJJJ-MM»; ungültig: null. */
function parseShow(s) {
  const m = /^\s*(\d{1,2})\s*[.\/]\s*(\d{4})\s*$/.exec(s || "");
  if (!m || +m[1] < 1 || +m[1] > 12) return null;
  return `${m[2]}-${String(+m[1]).padStart(2, "0")}`;
}
/** Anzahl Quartalsenden (März, Juni, September, Dezember) in den Monaten a bis b − 1. */
function quarterEnds(a, b) { let n = 0; for (let m = a; m < b; m++) if (m % 3 === 2) n++; return n; }

// ---------- Zahlen ----------
/** Zahl aus einer Eingabe mit Tausendertrennzeichen (’ ' Leerzeichen) und Dezimalkomma; leer oder ungültig: null. */
function parseNum(s) {
  s = String(s).replace(/[’'\s]/g, "").replace(",", ".");
  if (s === "" || s === "-" || s === ".") return null;
  const v = Number(s); return Number.isFinite(v) ? v : null;
}
const fmtInt = v => Math.round(v).toLocaleString("de-CH");
function fmtMio(v) { return Math.abs(v) >= 1e6 ? `${(v / 1e6).toLocaleString("de-CH", { maximumFractionDigits: 2 })} Mio` : `${fmtInt(v / 1000)}k`; }
/** Anteil als Prozentzahl ohne Zeichen (0.05 → «5»). */
const fmtPct = v => `${(v * 100).toLocaleString("de-CH", { maximumFractionDigits: 3 })}`;
/** Zahl aus einem Feld der Optimierung; sonst der Standard `d`. */
const optNum = (v, d) => { const x = parseFloat(String(v).replace(",", ".")); return Number.isFinite(x) ? x : d; };
const optRound = x => Math.round(x * 1e6) / 1e6;
/** Steuersatz einer Satztabelle [[Betrag, Satz], …]: linear zwischen den Stufen, ausserhalb konstant. */
function rateAt(table, x) {
  const t = [...table].sort((a, b) => a[0] - b[0]);
  if (!t.length || x <= t[0][0]) return t.length ? t[0][1] : 0;
  for (let i = 1; i < t.length; i++) if (x < t[i][0]) { const [x0, r0] = t[i - 1], [x1, r1] = t[i]; return r0 + (r1 - r0) * (x - x0) / (x1 - x0); }
  return t[t.length - 1][1];
}

// ---------- Pfade und Dateinamen ----------
/** Wert unter einem Punkt-Pfad («a.b.0.c»); fehlt ein Glied: undefined. */
function getPath(obj, path) { return path.split(".").reduce((o, k) => (o == null ? undefined : o[k]), obj); }
/** Setzt den Wert unter einem Punkt-Pfad; die Glieder davor müssen bestehen. */
function setPath(obj, path, v) {
  const ks = path.split("."); let o = obj;
  for (let i = 0; i < ks.length - 1; i++) o = o[ks[i]];
  o[ks[ks.length - 1]] = v;
}
const baseName = p => String(p || "").split(/[\\/]/).pop();
/** Anzeigename einer Zeitreihe: ohne Ordner, das interne Präfix «library:» und die Endung «.csv». */
const seriesShort = p => baseName(String(p || "").replace(/^library:/, "")).replace(/\.csv$/i, "");

// ---------- Dauer ----------
/** Dauer in Sekunden kurz: «unter 1 s», «45 s», «12 min», «2,5 h». */
function fmtShort(sec) {
  if (sec < 1) return "unter 1 s";
  if (sec < 90) return `${Math.round(sec)} s`;
  if (sec < 5400) return `${Math.round(sec / 60)} min`;
  return `${(sec / 3600).toLocaleString("de-CH", { maximumFractionDigits: 1 })} h`;
}
/** «ca. 12 s (7 s bis 20 s)»: grobe Schätzung mit Spanne, weil die Suchschritte der Optimierung vom Fall abhängen. */
const fmtEstimate = (sec, spread = [0.6, 1.6]) => sec < 1 ? "<b>unter 1 Sekunde</b>" : `ca. <b>${fmtShort(sec)}</b> <span class="computed">(meist ${fmtShort(sec * spread[0])} bis ${fmtShort(sec * spread[1])})</span>`;
/**
 * Geschätzte Dauer der Optimierung in Sekunden aus der Messung `b` ({ us, gen, fixedPaths }: µs je Lauf, davon Erzeugen
 * des Pfads, feste Anzahl Pfade), den Einstellungen `c` ({ paths, checkPaths, strategies }) und der Summe der
 * Auswertungen über alle Strategien. Die Suchpfade entstehen einmal, die Prüfpfade bei jeder Prüfung neu.
 */
function optSecondsFor(b, c, evals) {
  const fit = b.fixedPaths ?? c.paths, check = b.fixedPaths ?? c.checkPaths, n = c.strategies.length;
  const sim = Math.max(0.1, b.us - b.gen);
  return (evals * fit * sim + fit * b.gen + (n + 1) * check * b.us) / 1e6;
}
/** Anzahl Stufen einer Variation von `from` bis `to` mit Schritt `step` (Monate als «JJJJ-MM», wenn `month`). */
function sweepSteps(from, to, step, month = false) {
  if (month) { const a = ym2i(from), b = ym2i(to); return a <= b && step > 0 ? Math.floor((b - a) / step) + 1 : 0; }
  return to >= from && step > 0 ? Math.floor((to - from) / step + 1e-9) + 1 : 0;
}

if (typeof module !== "undefined") {
  module.exports = {
    esc, clamp, isYM, ym2i, i2ym, ymShow, parseShow, quarterEnds, parseNum, fmtInt, fmtMio, fmtPct, optNum, optRound,
    rateAt, getPath, setPath, baseName, seriesShort, fmtShort, fmtEstimate, optSecondsFor, sweepSteps,
  };
}
