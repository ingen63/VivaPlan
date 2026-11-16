// Oberfläche von VivaPlan: Editor, Live-Vorschau und Ergebnisansicht.
"use strict";
/** Technische Obergrenze der Kombinationen der Variationen (wie im Rechenkern, Speicherschutz). */
const MAX_COMBINATIONS = 1_000_000;
/** Ab dieser geschätzten Rechenzeit (Sekunden) fragt die App vor dem Start nach. */
const LONG_RUN_SECONDS = 60;
const SWEEP_PARAMS = [
  { id: "wealth.stockShare", label: "Aktienanteil", type: "pct", def: [0.3, 1.0, 0.1] },
  { id: "pension.pk.lumpsum", label: "Kapitalbezug Pensionskasse", type: "pct", def: [0, 1, 0.25] },
  { id: "returns.startDate", label: "Historischer Startmonat", type: "month", def: ["1970-01", "2010-01", 12] },
  { id: "milestone.retire", label: "Datum Bezug Pensionskasse", type: "month", def: ["2026-09", "2028-03", 3] },
  { id: "flows.spendingFactor", label: "Ausgaben-Faktor", type: "pct", def: [0.9, 1.1, 0.05] },
  { id: "withdrawal.rate", label: "Entnahmerate (fixe Rate / Guardrails-Start)", type: "pct", def: [0.03, 0.05, 0.005] }
];
const SECTIONS = [
  { id: "general", title: "Haushalt & Projekt" },
  { id: "milestones", title: "Meilensteine" },
  { id: "pension", title: "Vorsorge" },
  { id: "pillar3a", title: "Säule 3a" },
  { id: "flows", title: "Einnahmen & Ausgaben" },
  { id: "wealth", title: "Vermögen & Anlage" },
  { id: "returns", title: "Simulation" },
  { id: "withdrawal", title: "Entnahmestrategie" },
  { id: "optimize", title: "Optimierung" },
  { id: "taxes", title: "Steuern" },
  { id: "realestate", title: "Immobilien" },
  { id: "scenarios", title: "Szenarien & Simulation" }
];

let previewBasis = null;
let state = null, currentPath = null, dirty = false, seriesInfo = null, lastRun = null, library = null;
/** Planung, mit der `lastRun` berechnet wurde (Stand des Laufs, nicht des Editors); `null`: nicht bekannt. */
let lastRunPlan = null;
let current = "general";
let flashId = null;
let lastSim = null;

// ---------- Hilfsfunktionen ----------
const $ = s => document.querySelector(s);
/** Schliesst offene Info-Popups (auch bei Maus- oder Tastaturfokus); `true`, wenn eines offen war. */
function closeInfoPops() {
  const shown = p => p.classList.contains("open") || p.matches(":hover, :focus-within");
  const pops = [...document.querySelectorAll(".infopop")].filter(shown), was = pops.length > 0;
  pops.forEach(p => { p.classList.remove("open"); p.classList.add("closed"); if (p.contains(document.activeElement)) document.activeElement.blur(); });
  return was;
}
document.addEventListener("focusout", e => e.target.closest?.(".infopop")?.classList.remove("closed"));
document.addEventListener("mouseout", e => { const p = e.target.closest?.(".infopop"); if (p && !p.contains(e.relatedTarget)) p.classList.remove("closed"); });
/** Info-Symbol (ⓘ) mit Popup: öffnet bei Maus, Tastaturfokus und Klick, bleibt offen solange Maus oder Fokus darauf sind, Esc schliesst. */
const infoPop = (html, cls = "") => `<span class="infopop" tabindex="0" role="button" aria-label="Erklärung anzeigen" aria-haspopup="true" data-action="info-toggle">ⓘ<span class="infobody ${cls}" role="tooltip">${html}</span></span>`;
/** Kurze Erklärung einer Reihe (höchstens 2 Zeilen): `# Info:` der Datei, sonst Quelle und Hinweise, gekürzt; ohne Angaben kein Symbol. */
function seriesTip(i) {
  const text = i?.info || [i?.source, ...(i?.notes || [])].filter(Boolean).join(" ");
  return text ? infoPop(`<span class="two">${esc(text)}</span>`, "short") : "";
}
/** Simulationsstart als MM.JJJJ für Bezeichnungen («… per Simulationsstart (11.2026)»). */
const startTxt = () => ymShow(i2ym(startM()));
const cur = () => state.project.currency;
const fmtMoney = v => `${fmtInt(v)} ${cur()}`;

// Pfadzugriff auf die Planung (getPath, setPath in util.js)
function getP(path) { return getPath(state, path); }
function setP(path, v) { setPath(state, path, v); }

// ---------- Meilensteine & Zeit ----------
function person(id) { return state.household.persons.find(p => p.id === id) || state.household.persons[0]; }
function msMonth(id) {
  const ms = state.milestones.find(m => m.id === id);
  if (!ms) return NaN;
  if (ms.kind === "date") return ym2i(ms.date);
  const b = ym2i(person(ms.person).birth);
  return b + Math.round((+ms.years || 0) * 12);
}
const saleOf = ref => typeof ref === "string" && ref.startsWith("afterSale:") ? ref.slice(10) : null;
const propById = id => state.realestate.properties.find(p => p.id === id);
function refMonth(ref) { if (ref === "never" || ref === "afterSale" || saleOf(ref) || ref == null) return Infinity; return isYM(ref) ? ym2i(ref) : msMonth(ref); }
const startM = () => msMonth("start");
const endM = () => msMonth("end");
function ageAt(m, pid) { return (m - ym2i(person(pid ?? state.household.persons[0].id).birth)) / 12; }
const ageStr = m => ageAt(m).toLocaleString("de-CH", { minimumFractionDigits: 1, maximumFractionDigits: 1 });
function msName(ref) { if (ref === "never") return "nie"; if (ref === "afterSale") return "nach Verkauf"; if (saleOf(ref)) return `nach Verkauf von ${propById(saleOf(ref))?.name ?? "?"}`; if (isYM(ref)) return ymShow(ref); return state.milestones.find(m => m.id === ref)?.name ?? "?"; }

function seriesAt(ser, m) {
  if (!ser) return 0;
  if (ser.mode === "const") return +ser.value || 0;
  let best = null, bm = -Infinity;
  for (const r of ser.rows) { const rm = refMonth(r.from); if (rm <= m && rm >= bm) { bm = rm; best = r; } }
  return best ? +best.value || 0 : 0;
}
const tax = (table, x) => x <= 0 ? 0 : x * rateAt(table, x) * (+state.taxes.multiplier || 1);

// Hypothek: eine oder mehrere Tranchen, jede mit einer Abfolge von Perioden. Die erste Periode beginnt am
// Startdatum (bzw. beim Kauf), jede weitere am Ende der vorherigen. Amortisation als fixer Betrag pro Quartal,
// fällig Ende März/Juni/September/Dezember; direkt (reduziert die Schuld) oder indirekt (Einzahlung in die Säule 3a).
const saron = () => +state.returns.fixed.saron || 0;
const perRate = d => d.type === "saron" ? Math.max(0, saron() + (+d.margin || 0)) : +d.rate || 0;
const newPeriod = (o = {}) => ({ type: "fixed", term: 5, rate: 0.015, margin: 0.009, amort: 0, amortMode: "direct", ...o });
const newMortgage = () => ({ tranches: [ { name: "Tranche 1", amount: 400000, after: "renew", periods: [newPeriod({ term: 10 })] } ] });

function trancheSchedule(p, t) {
  if (p.status === "rent" || !t?.periods?.length) return [];
  const s0 = startM(), e = endM(), planned = p.status === "planned";
  let m = planned ? refMonth(p.buy) : ym2i(t.start);
  if (!Number.isFinite(m) || !(e > s0)) return [];
  const lim = planned ? e + 1 : Math.min(e + 1, refMonth(p.sell));
  const out = []; let amount = +t.amount || 0;
  // FEA-36: Indirekte Amortisation nur bis vor dem AHV-Bezug der Person der Säule 3a, danach direkt.
  const ahv = msMonth(ahvIdOf(t.pillar3a));
  for (let i = 0; i < 100 && m < lim; i++) {
    if (i >= t.periods.length && t.after !== "renew") break;
    const d = t.periods[Math.min(i, t.periods.length - 1)];
    const end = m + Math.max(1, Math.round((+d.term || 0) * 12));
    const from = planned ? m : Math.max(m, s0), past = end <= from;
    const q = +d.amort || 0, indirect = d.amortMode === "indirect", direct = !indirect || m >= ahv;
    const directFrom = direct ? from : Math.max(from, Number.isFinite(ahv) ? ahv : Infinity);
    const a0 = past ? null : amount, a1 = past ? null : Math.max(0, a0 - (directFrom < end ? q * quarterEnds(directFrom, end) : 0));
    out.push({ i, start: m, end, from, d, rate: perRate(d), q, direct, toAhv: indirect && !direct && ahv < end, a0, a1, past, renewal: i >= t.periods.length });
    if (!past) amount = a1;
    m = end;
  }
  return out;
}
const mortTotal = p => (p.mortgage?.tranches || []).reduce((a, t) => a + (+t.amount || 0), 0);

// ---------- Rechenkern ----------
let lastIssues = [], previewError = null, previewSeq = 0, previewTimer = 0;
const TICK_KINDS = ["mortgageRenewal", "mortgageRepayment", "amortizationSwitch", "pillar3aPayout", "rentStart"];

/** Ergebnis des Rechenkerns in die Form bringen, die Grafiken und Anzeigen verwenden. */
function adapt(r) {
  const out = { m: [], liquid: [], prop: [], pk: [], total: [], pots: [], events: [], p3a: [], label: r.label };
  // FEA-39: Wohnanteil = erfasste Lebenshaltung im Startmonat − Lebenshaltung ohne Wohnen der Simulation
  const r0 = r.records[0];
  out.housingShare = r0 && state.flows.spendingIncludesHousing ? seriesAt(state.flows.spending, ym2i(r0.month)) * (state.flows.spendingFactor ?? 1) - r0.plannedSpending : 0;
  for (const x of r.records) {
    out.pots.push([x.cash, x.bonds, x.stocks]);
    const liquid = Math.max(0, x.liquid), prop = Math.max(0, x.propertyEquity), pk = x.pensionFund + x.pillar3a;
    out.m.push(ym2i(x.month)); out.liquid.push(liquid); out.prop.push(prop); out.pk.push(pk); out.total.push(liquid + prop + pk);
  }
  for (const e of r.events) {
    out.events.push({ m: ym2i(e.month), label: e.text, kind: e.kind, tick: TICK_KINDS.includes(e.kind), bad: e.kind === "emergencySale" || e.kind === "depleted" });
    if (e.kind === "pillar3aPayout") out.p3a.push({ m: ym2i(e.month), text: e.text });
  }
  // Kauf, Verkauf und Notverkauf von Immobilien für die Zeitleiste
  out.propEvents = (r.propertyEvents || []).filter(e => e.kind !== "renewal").map(e => ({ m: ym2i(e.month), kind: e.kind, property: e.property }));
  const s = r.summary;
  out.depleted = s.depleted ? ym2i(s.depleted) : null;
  out.minLiquid = s.minLiquid; out.minLiquidM = s.minLiquidMonth ? ym2i(s.minLiquidMonth) : startM();
  out.totalAtRetire = s.totalAtRetirement; out.pkPension = s.pensionFundMonthly;
  out.lumpNet = s.lumpsumNet; out.lumpTax = s.lumpsumTax; out.retireM = refMonth(state.pension.pk.at);
  return out;
}

/** `data-preview` auf <html>: «pending», solange eine Vorschau aussteht, sonst «idle» (FEA-87: Signal für die Oberflächentests). */
function requestPreview() {
  requestPreview.n ??= 0;
  clearTimeout(previewTimer);
  document.documentElement.dataset.preview = "pending";
  const req = ++requestPreview.n;
  previewTimer = setTimeout(async () => {
    const seq = ++previewSeq;
    try {
      const r = await Engine.call("preview", { plan: state, path: currentPath });
      if (seq !== previewSeq) return;
      lastIssues = r.issues; previewError = r.error; lastSim = r.result ? adapt(r.result) : null; if (lastSim) lastSim.p3aView = r.pillar3a || []; previewBasis = r.basis;
    } catch (e) {
      if (seq !== previewSeq) return;
      previewError = errText(e); lastSim = null;
    }
    renderDerived();
    if (req === requestPreview.n) document.documentElement.dataset.preview = "idle"; // keine neuere Anfrage
  }, 120);
}

/** Prüfmeldungen des Rechenkerns plus Hinweise aus der Vorschau. */
function collectIssues() {
  const v = lastIssues.map(i => ({ sev: i.severity === "error" ? "err" : "warn", sec: i.section, text: esc(i.message) }));
  if (previewError && !v.some(x => x.sev === "err")) v.push({ sev: "err", sec: "general", text: esc(previewError) });
  const sc = seriesInfo && !seriesInfo.error && state.returns.source !== "fixed" && seriesInfo.currency;
  if (sc && sc !== cur()) v.push({ sev: "warn", sec: "returns", text: `Die Zeitreihe ist in ${esc(sc)}, die Planung in ${esc(cur())}. Renditen und Inflation gelten dann für ${esc(sc)}.` });
  if (lastSim) {
    if (lastSim.depleted !== null) v.push({ sev: "err", sec: "flows", text: `Vorschau: Vermögen aufgebraucht im ${ymShow(i2ym(lastSim.depleted))} (Alter ${ageStr(lastSim.depleted)}).` });
    for (const ev of lastSim.events) if (ev.kind === "emergencySale") v.push({ sev: "warn", sec: "realestate", text: `Vorschau: Notverkauf im ${ymShow(i2ym(ev.m))} (Alter ${ageStr(ev.m)}).` });
  }
  return v;
}

/** Fehlende Teile ergänzen, damit der Editor jede gültige Projektdatei anzeigen kann. */
const SERIES_PATHS = ["flows.income", "flows.spending", "pension.pk.contribution", "pension.pk.interest", "wealth.stockShare", "wealth.reserveMonths", "realestate.emergencyMonths",
  "partner.income", "partner.pk.contribution", "partner.pk.interest"];
/** FEA-43: Beschriftung der Auswahl beim Erwerbseinkommen jeder Person. */
const INCOME_INDEXED = "Steigt mit der Teuerung (Beträge in heutigen Preisen)";
const newPartner = (pid = "B") => ({ income: { mode: "const", value: 0 }, incomeIndexed: true,
  pk: { capital: 0, at: "retire2", lumpsum: 0, conversion: 0.05, contribution: { mode: "const", value: 0 }, interest: { mode: "const", value: 0.01 } },
  ahv: { monthly: 2000, at: "ahv2", indexed: true } });
function newPropId(p = state) {
  const used = new Set(p.realestate.properties.map(q => q.id));
  let id; do { id = "obj" + Math.random().toString(36).slice(2, 7); } while (used.has(id));
  return id;
}
const FIXED_RESERVE_MONTHS = 3;
// FEA-17: Planungen mit anderer Strategie als «Standard» und einer Reserve ungleich 3 Monaten rechnen mit 3 Monaten.
function reserveDiffers(p) {
  if ((p.withdrawal?.strategy ?? "standard") === "standard") return false;
  const r = p.wealth?.reserveMonths;
  const vals = r == null ? [] : typeof r !== "object" ? [r] : r.mode === "const" ? [r.value] : (r.rows || []).map(x => x.value);
  return vals.some(v => v !== FIXED_RESERVE_MONTHS);
}
/**
 * FEA-45: Haftungsausschluss. Die Kurzfassung muss beim ersten Start bestätigt werden; die Bestätigung liegt in den
 * Einstellungen (nicht in der Planung) mit der Version des Textes. Ändert sich der Text, die Version erhöhen: Dann wird
 * erneut gefragt. Der vollständige Text steht in der Hilfe (`hilfe/00-einstieg.md`, Abschnitt «@disclaimer»).
 */
const DISCLAIMER_VERSION = 1, DISCLAIMER_KEY = "disclaimerAccepted";
const DISCLAIMER_SHORT = "VivaPlan ist ein kostenloses Hilfsmittel zur persönlichen Planung. Alle Ergebnisse sind Modellrechnungen auf Basis Ihrer Eingaben und vereinfachter Annahmen. Sie können Fehler enthalten und ersetzen keine Finanz-, Steuer- oder Rechtsberatung. Prüfen Sie wichtige Entscheidungen mit einer Fachperson. Die Nutzung erfolgt auf eigene Verantwortung.";
const DISCLAIMER_FOOT = "Modellrechnung ohne Gewähr. Keine Finanz- oder Steuerberatung.";
const disclaimerPending = () => Pref.get(DISCLAIMER_KEY, 0) !== DISCLAIMER_VERSION;
/** Zeigt die Kurzfassung, bis das Kästchen angekreuzt und «Weiter» gedrückt ist (Esc schliesst den Dialog nicht). */
function disclaimerGate() {
  const d = $("#disclaimer"), check = $("#disclaimerCheck"), ok = $("#disclaimerOk");
  $("#disclaimerText").textContent = DISCLAIMER_SHORT;
  check.checked = false; ok.disabled = true;
  check.onchange = () => { ok.disabled = !check.checked; };
  d.oncancel = e => e.preventDefault();
  return new Promise(res => {
    d.onclose = () => {
      if (d.returnValue !== "ok" || !check.checked) { d.returnValue = ""; d.showModal(); return; }
      Pref.set(DISCLAIMER_KEY, DISCLAIMER_VERSION); res();
    };
    d.returnValue = ""; d.showModal();
  });
}
/** FEA-58: Einstellung «Hinweis zur Cash-Reserve gezeigt»; der Hinweis erscheint nur beim ersten Laden einer betroffenen Planung. */
const RESERVE_NOTICE_KEY = "reserveNoticeShown";
async function reserveNotice(p) {
  if (!reserveDiffers(p) || Pref.get(RESERVE_NOTICE_KEY, false)) return;
  Pref.set(RESERVE_NOTICE_KEY, true);
  await ask("Hinweis zur Cash-Reserve", `<p>Diese Planung verwendet eine andere Entnahmestrategie als «Standard» und eine Cash-Reserve ungleich ${FIXED_RESERVE_MONTHS} Monaten. Die Reserve ist neu ein Parameter der Strategie «Standard»; alle anderen Strategien rechnen mit einer festen Reserve von ${FIXED_RESERVE_MONTHS} Monaten.</p><p>Die Ergebnisse dieser Planung können sich dadurch leicht ändern. Die gespeicherte Reserve bleibt in der Datei erhalten und gilt wieder, sobald Sie zu «Standard» wechseln.</p>`);
}
function normalize(p) {
  delete p.conventions;
  p.withdrawal ??= { strategy: "standard" };
  p.withdrawal.buckets ??= { cashYears: 2, bondYears: 6, refillThreshold: 0, reserveMonths: 3 };
  p.withdrawal.buckets.reserveMonths ??= 3;
  p.withdrawal.fixedRate ??= { rate: 0.04 };
  p.withdrawal.guardrails ??= { initialRate: 0.05, upper: 0.2, lower: 0.2, adjust: 0.1 };
  p.withdrawal.vpw ??= { horizonAge: 100, realReturn: 0.02 };
  p.withdrawal.endowment ??= { rate: 0.04, smoothing: 0.3 };
  p.withdrawal.floor ??= 0; p.withdrawal.ceiling ??= 0;
  // Meilenstein «Alter von» ohne Person (entsteht beim Umschalten von «Datum»): erste Person verwenden
  for (const m of p.milestones || []) if (m.kind === "age") { m.person ??= p.household?.persons?.[0]?.id; m.years ??= 65; }
  p.household.married ??= true; p.household.ahvCoupleCap ??= 3780;
  if (!p.household.persons[1]) delete p.partner;
  if (p.taxes.couple) p.taxes.couple = { income: p.taxes.couple.income ?? [], wealth: p.taxes.couple.wealth ?? [], capital: p.taxes.couple.capital ?? [] };
  p.scenarios ??= { sweeps: [] }; p.scenarios.sweeps ??= [];
  p.flows.extras ??= [];
  p.pillar3a ??= [];
  for (const a of p.pillar3a) { a.payouts ??= []; a.use ??= "wealth"; a.contribution ??= 0; a.rate ??= 0; a.balance ??= 0; }
  p.realestate ??= {}; p.realestate.properties ??= [];
  for (const q of p.realestate.properties) if (q.status !== "rent") q.sell ??= "never";
  p.realestate.emergencyMonths ??= 0;
  p.realestate.affordability ??= { sustainability: 1 / 3, imputedRate: 0.05, fixCosts: 0.01, wealthContribution: 0.03, maxLoanToValue: 0.8 };
  p.returns.extrapolation ??= { method: "fixed", range: 30 };
  p.returns.bootstrap ??= { paths: 50000, block: 12, seed: 42 };
  // FEA-78: neue Planungen folgen den Vorschlagswerten des Aktienindex (`auto`); ältere setzt upgrade_plan
  p.returns.parametric ??= { paths: 50000, mu: 0.07, sigma: 0.15, df: 6, jumpLambda: 0.05, jumpMu: -0.3, jumpSigma: 0.15, auto: true };
  p.returns.parametric.seed ??= 42; p.returns.bootstrap.seed ??= 42;
  p.returns.parametric.auto ??= p.returns.source !== "parametric";
  p.returns.fixed.saron ??= 0;
  // FEA-53: Anlagestrategie (ältere Planungen: aus der Zeitreihe übernommen, siehe upgrade_plan)
  p.returns.strategy ??= { stocks: "worldJst", bonds: "ch" };
  p.returns.strategy.weights ??= { ch: 0.2, usa: 0.4, eu: 0.3, apac: 0.1 };
  for (const k of ["stocks", "bonds", "inflation", "saron"]) {
    const v = p.returns.fixed[k];
    if (v && typeof v === "object" && v.mode === "const") p.returns.fixed[k] = v.value;
  }
  for (const path of SERIES_PATHS) {
    const ks = path.split("."), last = ks.pop(), o = ks.reduce((a, k) => a?.[k], p);
    if (o == null) continue;
    const v = o[last];
    if (v == null || typeof v === "number") o[last] = { mode: "const", value: v ?? 0 };
  }
  for (const pr of p.realestate.properties) {
    pr.id ??= newPropId(p); pr.fixCosts ??= 0; pr.inflation ??= false;
    pr.inflationValue ??= pr.inflation; pr.inflationFixCosts ??= pr.inflation; pr.inflationRent ??= true;
    if (pr.status === "rent") continue;
    pr.mortgage ??= { tranches: [] };
    for (const t of pr.mortgage.tranches) { t.name ??= "Tranche"; t.after ??= "renew"; t.periods = t.periods.map(d => newPeriod(d)); }
  }
  return p;
}

/**
 * Geladene Planung übernehmen: zuerst auf den aktuellen Stand bringen (FEA-37, FEA-38; Hinweise anzeigen), dann
 * normalisieren. Liefert die Hinweise.
 */
async function adopt(plan) {
  let notes = [];
  try { const r = await Engine.call("upgrade_plan", { plan }); plan = r.plan; notes = r.notes || []; }
  catch { /* ohne Rechenkern: unverändert übernehmen */ }
  state = normalize(plan);
  return notes;
}
async function showNotes(notes) {
  if (notes?.length) await ask("Hinweise zur Planung", `<p>Die Planung wurde auf den aktuellen Stand gebracht:</p><ul>${notes.map(n => `<li>${esc(n)}</li>`).join("")}</ul>`);
}

async function refreshLibrary() {
  try { library = await Engine.call("library_list"); }
  catch (e) { library = { error: errText(e), entries: [] }; }
  if (current === "returns") renderSection();
}

/** FEA-53: Verweis auf die Zeitreihe der Planung: die Anlagestrategie (`strategy:<JSON>`), bei älteren Planungen die Datei. */
function seriesRef() {
  const r = state.returns;
  return r.series || (r.strategy ? "strategy:" + JSON.stringify(r.strategy) : null);
}
// FEA-78: Parameter des Aktienmodells mit Vorschlagswerten je Aktienindex (Feld → Darstellung)
const PARAM_KEYS = { mu: "pct", sigma: "pct", df: "num", jumpLambda: "num", jumpMu: "pct", jumpSigma: "pct" };
const paramSuggestion = () => seriesInfo && !seriesInfo.error ? seriesInfo.parametric?.stocks ?? null : null;
const fmtParam = (k, v) => PARAM_KEYS[k] === "pct" ? `${fmtPct(v)} %` : (+v).toLocaleString("de-CH", { maximumFractionDigits: 2 });
/** FEA-78: Folgen die Parameter dem Index (`auto`), die Vorschlagswerte des gewählten Index übernehmen. Ohne
 *  Änderungsmarke und ohne eigenen Schritt im Verlauf (der aktuelle Schritt erhält dieselben Werte): Der Rechenkern
 *  rechnet bei `auto` ohnehin mit den Vorschlagswerten. */
function followSuggestion() {
  const q = state?.returns.parametric, s = paramSuggestion();
  if (!q?.auto || !s) return;
  const vals = Object.fromEntries(Object.keys(PARAM_KEYS).map(k => [k, s[k]]));
  if (Object.keys(vals).every(k => q[k] === vals[k])) return;
  Object.assign(q, vals);
  History.patch(o => { if (o.returns?.parametric?.auto) Object.assign(o.returns.parametric, vals); });
}
let seriesInfoRef;
async function refreshSeriesInfo() {
  seriesInfo = null;
  const s = seriesRef();
  seriesInfoRef = s;
  if (s) {
    try { seriesInfo = await Engine.call("series_info", { path: s, planPath: currentPath }); }
    catch (e) { seriesInfo = { error: errText(e) }; }
  }
  if (seriesInfoRef !== s) return; // eine neuere Abfrage läuft
  followSuggestion();
  // Welche Felder «Simulation» zeigt, hängt von der Zeitreihe ab.
  if (current === "returns") renderSection(); else renderDerived();
}

// ---------- Formular-Bausteine ----------
const UNIT = { chf: () => cur(), chfopt: () => cur(), pct: () => "%", months: () => "Monate", years: () => "Jahre", num: () => "" };
function showVal(type, v) {
  if (type === "chf") return fmtInt(v ?? 0);
  if (type === "chfopt") return v == null ? "" : fmtInt(v);
  if (type === "pct") return fmtPct(v ?? 0);
  if (type === "month") return ymShow(v);
  return v ?? "";
}
// ---------- Pflichteingaben (FEA-13; Liste in docs/Pflichteingaben.md) ----------
const REQUIRED_PATHS = [/^household\.persons\.\d\.(name|birth)$/, /^household\.ahvCoupleCap$/, /^(partner|pension)\.pk\.(capital|conversion|lumpsum)$/,
  /^(partner|pension)\.ahv\.monthly$/, /^wealth\.(stocks|bonds|cash)$/, /^taxes\.multiplier$/];
const isRequired = path => REQUIRED_PATHS.some(r => r.test(path));
const reqMark = path => isRequired(path) ? ` <span class="req" title="Pflichtfeld">*</span>` : "";
/** Vom Benutzer eingegebene, nicht lesbare Werte (Feldpfad → Abschnitt und Meldung); die Planung behält den letzten gültigen Wert. */
const invalidFields = new Map();
/** Fehlermeldungen des Rechenkerns und ungültige Eingaben direkt am Feld anzeigen. */
// ISS-13: Solange in ein Feld getippt wird, wartet alles (Vorschau, berechnete Werte, Meldungen, Prüfliste); erst beim
// Verlassen (Tab, Klick daneben, Enter) wird neu gerechnet und die Meldung am Feld gezeigt oder entfernt.
let heldField = null, heldShown = false, pendingRefresh = false;
function holdField(el) {
  if (heldField === el.dataset.path) return;
  heldField = el.dataset.path;
  heldShown = !!(el.closest(".inp") || el).nextElementSibling?.classList.contains("ferr") || el.classList.contains("invalid");
}
function releaseField(el) {
  if (heldField !== el.dataset.path) return;
  heldField = null;
  el.classList.toggle("invalid", invalidFields.has(el.dataset.path));
  if (pendingRefresh) { pendingRefresh = false; scheduleRefresh(); }
  applyFieldIssues();
}
function applyFieldIssues() {
  document.querySelectorAll("#main .ferr").forEach(e => e.remove());
  const msgs = new Map();
  for (const i of lastIssues) if (i.field && i.severity === "error" && !msgs.has(i.field)) msgs.set(i.field, i.message);
  for (const [path, i] of invalidFields) msgs.set(path, i.message);
  if (heldField) { if (!msgs.has(heldField)) heldShown = false; else if (!heldShown) msgs.delete(heldField); }
  for (const [path, text] of msgs) {
    const el = [...document.querySelectorAll("#main [data-path]")].find(x => x.dataset.path === path);
    if (el) (el.closest(".inp") || el).insertAdjacentHTML("afterend", `<span class="ferr" role="alert">${esc(text)}</span>`);
  }
}
/** Alle Fehler der Planung: ungültige Eingaben und Fehler des Rechenkerns (frisch geprüft). */
async function planErrors() {
  const list = [...invalidFields].map(([field, i]) => ({ section: i.section, message: i.message, field }));
  try { for (const i of await Engine.call("validate", { plan: state })) if (i.severity === "error") list.push(i); }
  catch (e) { list.push({ section: "general", message: errText(e) }); }
  return list;
}
/** `true` (mit Meldung und Sprung zum ersten Fehler), wenn die Planung Fehler hat: «Simulation starten» und «Optimieren» sind dann gesperrt. */
async function blockedByErrors() {
  const errs = await planErrors();
  if (!errs.length) return false;
  const first = errs[0], sec = SECTIONS.find(s => s.id === first.section) || SECTIONS[0];
  if (!$("#results").hidden) showResults(false);
  goto(sec.id);
  toast(`Die Planung hat ${errs.length} Fehler. Erster Fehler im Abschnitt «${sec.title}»: ${first.message}`);
  document.documentElement.dataset.blocked = String(errs.length);
  return true;
}
function F(path, label, type = "chf", o = {}) {
  const v = getP(path), unit = o.unit ?? (UNIT[type] ? UNIT[type]() : "");
  const cls = ["chf", "chfopt", "pct", "num", "months", "years"].includes(type) ? "num" : "";
  const input = `<div class="inp"><input class="${cls}" data-path="${path}" data-type="${type}" value="${esc(showVal(type, v))}" ${type === "month" ? 'placeholder="MM.JJJJ"' : ""}>${unit ? `<span class="unit">${unit}</span>` : ""}</div>`;
  if (o.bare) return input;
  return `<label class="field"><span>${label}${reqMark(path)}</span>${input}${o.hint ? `<span class="computed">${o.hint}</span>` : ""}</label>`;
}
function Sel(path, opts, o = {}) {
  const v = getP(path);
  return `<select class="plain" data-path="${path}" data-type="select" ${o.rerender ? "data-rerender=1" : ""}>${opts.map(([k, l]) => `<option value="${esc(k)}" ${k === v ? "selected" : ""}>${esc(l)}</option>`).join("")}</select>`;
}
function Chk(path, label) { return `<label class="check"><input type="checkbox" data-path="${path}" data-type="bool" ${getP(path) ? "checked" : ""}> ${label}</label>`; }
function Ref(path, o = {}) {
  const v = getP(path), date = isYM(v);
  const opts = state.milestones.map(ms => `<option value="${ms.id}" ${ms.id === v ? "selected" : ""}>${esc(ms.name)}</option>`).join("");
  return `<div class="refwrap"><select class="plain" data-path="${path}" data-type="ref">
    ${o.never ? `<option value="never" ${v === "never" ? "selected" : ""}>nie</option>` : ""}
    ${o.afterSale ? `<option value="afterSale" ${v === "afterSale" ? "selected" : ""}>nach Verkauf der letzten Immobilie</option>` : ""}
    ${o.afterSale ? state.realestate.properties.filter(q => q.status !== "rent" && q.id !== o.self).map(q => `<option value="afterSale:${esc(q.id)}" ${v === "afterSale:" + q.id ? "selected" : ""}>nach Verkauf von «${esc(q.name)}»</option>`).join("") : ""}
    <optgroup label="Meilenstein">${opts}</optgroup>
    <option value="__date" ${date ? "selected" : ""}>Fixes Datum…</option></select>
    ${date ? F(path, "", "month", { bare: true }) : ""}</div>`;
}
function Series(path, label, type, note = "") {
  const s = getP(path), timed = s.mode === "timed";
  let body;
  if (!timed) body = `<div class="rows">${F(path + ".value", "Wert", type)}</div>`;
  else body = `<div class="rows">${s.rows.map((r, i) => `<div class="row"><span class="ab">ab</span>${Ref(`${path}.rows.${i}.from`)}${F(`${path}.rows.${i}.value`, "", type, { bare: true })}<button class="btn ghost small" title="Zeile entfernen" data-action="row-del" data-path="${path}.rows" data-i="${i}">✕</button></div>`).join("")}
    <div><button class="btn small" data-action="row-add" data-path="${path}.rows">+ Zeile</button></div></div>`;
  return `<div class="series"><div class="shead"><span class="t">${label}</span><span class="computed">${note}</span>
    <div class="seg"><button class="${timed ? "" : "on"}" data-action="series-mode" data-path="${path}" data-mode="const">Konstant</button><button class="${timed ? "on" : ""}" data-action="series-mode" data-path="${path}" data-mode="timed">Zeitabhängig</button></div></div>
    <div class="sbody">${body}<div class="miniwrap"><svg class="mini zoomable" data-mini="${path}" data-type="${type}"></svg><div class="tip"></div>
      <div class="minizoom">${zoomButtons("mini-zoom", miniZoom[path]?.years, `data-path="${path}"`)}<span>Mausrad: zoomen · Ziehen: verschieben</span></div></div></div></div>`;
}
const C = (key, arg = "") => `<span class="computed" data-computed="${key}" data-arg="${arg}"></span>`;

// ---------- Abschnitte ----------
const R = {};
R.general = () => `
  <h2>Haushalt & Projekt</h2><p class="lead">Grunddaten der Planung. Pflichtfelder sind mit <span class="req">*</span> gekennzeichnet. Alle Zeitangaben erfolgen als Monat und Jahr (MM.JJJJ); das Alter wird daraus berechnet.</p>
  <div class="card"><h3>Projekt</h3><div class="grid2">${F("project.name", "Projektname", "text")}
    <label class="field"><span>Währung</span>${Sel("project.currency", [["CHF", "CHF – Schweizer Franken"], ["EUR", "EUR – Euro"], ["USD", "USD – US-Dollar"]], { rerender: 1 })}</label></div>
    <p class="hint">Alle Beträge werden in der Projektwährung erfasst. Die Indizes der Anlagestrategie sind in CHF; bei einer anderen Projektwährung erscheint ein Hinweis.</p></div>
  <div class="card"><h3>Personen</h3><div class="persons">
    <div class="person">${state.partner ? `<b>Person 1</b>` : ""}${F("household.persons.0.name", "Name", "text")}${F("household.persons.0.birth", "Geburt", "month")}${C("age0")}</div>
    ${state.partner
      ? `<div class="person"><div class="phead"><b>Person 2</b><button class="btn ghost small" data-action="partner-del" title="Zweite Person entfernen">✕</button></div>${F("household.persons.1.name", "Name", "text")}${F("household.persons.1.birth", "Geburt", "month")}${C("age1")}</div>`
      : `<div class="person ghost"><button class="btn" data-action="partner-add">+ Zweite Person</button><span class="computed">Double Income: eigenes Einkommen, eigene Pensionskasse und AHV</span></div>`}
  </div>
  ${state.partner ? `<div style="margin-top:14px">${Chk("household.married", "Verheiratet (Ehepaar-Regeln)")}</div>
    <div class="grid2" style="margin-top:10px">${state.household.married ? F("household.ahvCoupleCap", "AHV-Renten zusammen höchstens, pro Monat", "chf") : ""}</div>
    <p class="hint">${state.household.married ? "Verheiratet: Sind beide AHV-Renten fällig, werden sie zusammen auf den angegebenen Betrag begrenzt (150 % der Maximalrente). Steuern werden gemeinsam berechnet; unter «Steuern» können eigene Tabellen für Verheiratete hinterlegt werden." : "Nicht verheiratet: AHV-Renten werden nicht begrenzt; Steuern werden gemeinsam nach den Tabellen unter «Steuern» berechnet."} Vermögen und Ausgaben gelten für den Haushalt. Hinterlassenenrenten werden nicht berechnet.</p>` : ""}</div>
  <div class="card"><h3>Simulationszeitraum</h3>
    <div class="grid2"><div>Start: ${C("ms", "start")}</div><div>Ende: ${C("ms", "end")}</div></div>
    <p class="hint">Start und Ende sind Meilensteine. <a href="#" data-action="goto" data-sec="milestones">In Meilensteine bearbeiten →</a></p></div>`;

R.milestones = () => { const l = limits(); return `
  <h2>Meilensteine</h2><p class="lead">Benannte Zeitpunkte, auf die sich alle anderen Eingaben beziehen können («ab Bezug Pensionskasse»). Verschiebt sich ein Meilenstein, verschieben sich alle Werte, die ihn verwenden.</p>
  <div class="card"><table class="tbl"><tr><th>Name</th><th>Festgelegt als</th><th>Wert</th><th>Ergibt</th><th></th></tr>
  ${state.milestones.map((ms, i) => { const sys = sysMs(ms); return `<tr class="${ms.id === flashId ? "flash" : ""}">
    <td>${sys ? `<span title="Fester Meilenstein; der Name lässt sich nicht ändern.">${esc(ms.name)}</span><br><span class="computed">${sys.kind === "ahv" ? `${l.ahvMin} bis ${l.ahvMax}` : `${l.pkMin} bis ${l.pkMax}`} Jahre</span>` : F(`milestones.${i}.name`, "", "text", { bare: true })}</td>
    <td>${Sel(`milestones.${i}.kind`, [["date", "Datum"], ["age", "Alter von"]], { rerender: 1 })}</td>
    <td style="width:190px">${ms.kind === "date" ? F(`milestones.${i}.date`, "", "month", { bare: true }) : `<div class="refwrap">${sys ? `<span class="computed" style="padding:8px 4px">${esc(person(ms.person).name || "Person " + (sys.index + 1))}</span>` : Sel(`milestones.${i}.person`, state.household.persons.map(p => [p.id, p.name]))}${F(`milestones.${i}.years`, "", "years", { bare: true, unit: "J." })}</div>`}</td>
    <td>${C("ms", ms.id)}</td>
    <td>${msLocked(ms) ? `<button class="btn ghost small" disabled title="${esc(sys ? `«${ms.name}» ist für jede Person fest vorhanden und lässt sich nicht löschen; der Zeitpunkt ist innerhalb der Grenzen frei wählbar.` : "Simulationsstart und -ende lassen sich nicht löschen.")}">✕</button>`
      : `<button class="btn ghost small" data-action="ms-del" data-i="${i}" title="Löschen">✕</button>`}</td></tr>`; }).join("")}
  </table><div style="margin-top:10px"><button class="btn small" data-action="ms-add">+ Meilenstein</button></div>
  <p class="hint">Alter wird mit Dezimalstellen akzeptiert, z. B. 64.5 Jahre = 64 Jahre und 6 Monate.</p>
  <p class="hint">«Bezug Pensionskasse» (${l.pkMin} bis ${l.pkMax} Jahre) und «AHV-Bezug» (${l.ahvMin} bis ${l.ahvMax} Jahre) sind für jede Person fest vorhanden und lassen sich nicht löschen. Die Grenzen sind in den <a href="#" data-action="settings">Einstellungen</a> änderbar. Das Ende des Erwerbseinkommens legen Sie unter «Einnahmen &amp; Ausgaben» fest.</p></div>`; };

function PensionCards(root, who) {
  const pk = `${root === "partner" ? "partner" : "pension"}.pk`, ahv = `${root === "partner" ? "partner" : "pension"}.ahv`, t = who ? ` · ${esc(who)}` : "";
  return `<div class="card"><h3>Pensionskasse${t}</h3><div class="grid2">
    ${F(`${pk}.capital`, `Altersguthaben per Simulationsstart (${startTxt()})`, "chf")}
    ${fixedAt(root === "partner" ? 1 : 0, "pk")}
    ${F(`${pk}.lumpsum`, "Anteil Kapitalbezug", "pct")}
    ${F(`${pk}.conversion`, "Umwandlungssatz", "pct")}</div>
    <div style="margin-top:14px">${Series(`${pk}.contribution`, "Beiträge pro Monat", "chf")}${Series(`${pk}.interest`, "Verzinsung pro Jahr", "pct")}</div></div>
  <div class="card"><h3>AHV${t}</h3><div class="grid2">${F(`${ahv}.monthly`, "Rente pro Monat bei Bezugsbeginn", "chf")}
    ${fixedAt(root === "partner" ? 1 : 0, "ahv")}</div>
    <div style="margin-top:10px">${Chk(`${ahv}.indexed`, "Rente jährlich an die Teuerung anpassen")}</div></div>`;
}
/** FEA-38: Pensionskasse und AHV beginnen an den festen Meilensteinen der Person. */
function fixedAt(index, kind) {
  const id = index ? (kind === "pk" ? "retire2" : "ahv2") : (kind === "pk" ? "retire" : "ahv");
  return `<label class="field"><span>Bezug ab</span><span class="computed" style="padding:8px 0">Meilenstein «${esc(state.milestones.find(m => m.id === id)?.name ?? id)}» · ${C("ms", id)} · <a href="#" data-action="goto" data-sec="milestones">ändern</a></span></label>`;
}
R.pension = () => `
  <h2>Vorsorge</h2><p class="lead">Pensionskasse (2. Säule) und AHV (1. Säule)${state.partner ? " für beide Personen" : ""}. Die Säule 3a hat einen eigenen Abschnitt.</p>
  ${PensionCards("pension", state.partner ? state.household.persons[0].name : "")}
  ${state.partner ? PensionCards("partner", state.household.persons[1].name) : ""}
  <p class="hint">${C("pkresult")}</p>`;

// ---------- Säule 3a (FEA-37) ----------
const newAccount = person => ({ person, balance: 0, rate: 0.01, contribution: 0, payouts: [], use: "wealth" });
/** Id des Meilensteins «AHV-Bezug» der Person (Tranche ohne Angabe: erste Person). */
const ahvIdOf = pid => state.household.persons.findIndex(p => p.id === pid) === 1 ? "ahv2" : "ahv";
/** «Hypothek zurückzahlen» setzt eine Tranche mit indirekter Amortisation in dieses Konto voraus. */
const hasIndirectFor = pid => state.realestate.properties.some(p => p.status !== "rent" && (p.mortgage?.tranches || []).some(t =>
  (t.pillar3a ?? state.household.persons[0].id) === pid && t.periods.some(d => d.amortMode === "indirect" && +d.amort > 0)));
// FEA-41: Monat einer 3a-Auszahlung (Datum oder Alter der Kontoinhaberin bzw. des Kontoinhabers).
function payoutMonth(x, pid) {
  if (x.kind === "age") { const b = ym2i(person(pid).birth), y = +x.years; return Number.isFinite(b) && x.years !== "" && x.years != null && Number.isFinite(y) ? b + Math.round(y * 12) : NaN; }
  return ym2i(x.date);
}
const p3aViewOf = pid => lastSim?.p3aView?.find(v => v.person === pid) ?? null;
R.pillar3a = () => {
  const l = limits();
  const cards = state.household.persons.slice(0, 2).map((pp, pi) => {
    const k = state.pillar3a.findIndex(a => a.person === pp.id), who = state.household.persons.length > 1 ? ` · ${esc(pp.name || "Person " + (pi + 1))}` : "";
    const ahvId = pi ? "ahv2" : "ahv", ahv = msMonth(ahvId);
    if (k < 0) return `<div class="card"><h3>Säule 3a${who}</h3><p class="hint">Kein 3a-Konto erfasst.</p><button class="btn small" data-action="p3a-add" data-person="${esc(pp.id)}">+ 3a-Konto</button></div>`;
    const A = `pillar3a.${k}`, acc = state.pillar3a[k];
    const uses = hasIndirectFor(pp.id) ? [["wealth", "Ins freie Vermögen"], ["repay", "Hypothek zurückzahlen"]] : [["wealth", "Ins freie Vermögen"]];
    const earliest = Number.isFinite(ahv) ? ymShow(i2ym(ahv - 60)) : "?";
    const events = (lastSim?.p3a || []).filter(e => state.household.persons.length < 2 || e.text.includes(`(${pp.name})`));
    return `<div class="card"><h3>Säule 3a${who}<button class="btn ghost small" style="float:right" data-action="list-del" data-path="pillar3a" data-i="${k}" title="3a-Konto entfernen">✕</button></h3>
      <div class="grid3">${F(A + ".balance", `Guthaben per Simulationsstart (${startTxt()})`, "chf")}${F(A + ".rate", "Verzinsung / Rendite p.a.", "pct")}
        ${F(A + ".contribution", "Einzahlung pro Jahr", "chf", { hint: `0 bis ${fmtInt(l.pillar3aMax)}; monatlich ein Zwölftel bis vor dem AHV-Bezug (${Number.isFinite(ahv) ? ymShow(i2ym(ahv)) : "?"}); zählt zu den Ausgaben` })}</div>
      <h4>Auszahlungsplan</h4>
      <table class="tbl"><tr><th>Zeitpunkt</th><th>Ergibt</th><th>Betrag</th><th>Verwendung</th><th title="aus der Vorschau">Guthaben vor Auszahlung → Rest danach</th><th></th></tr>
      ${(acc.payouts || []).map((x, j) => {
        const P = `${A}.payouts.${j}`;
        return `<tr data-p3a-row="${j}"><td style="width:220px"><div class="refwrap">${Sel(P + ".kind", [["date", "Datum"], ["age", "Alter"]], { rerender: 1 })}${x.kind === "age" ? F(P + ".years", "", "years", { bare: true, unit: "J." }) : F(P + ".date", "", "month", { bare: true })}</div></td>
        <td>${C("p3aat", `${k}.${j}`)}</td>
        <td style="width:170px">${F(P + ".amount", "", "chfopt", { bare: true })}</td>
        <td>${Sel(P + ".use", uses)}</td>
        <td style="white-space:nowrap">${C("p3apay", `${k}.${j}`)}</td>
        <td><button class="btn ghost small" data-action="list-del" data-path="${A}.payouts" data-i="${j}" title="Zeile entfernen">✕</button></td></tr>`;
      }).join("")}</table>
      <div style="margin-top:10px"><button class="btn small" data-action="p3a-payout-add" data-k="${k}">+ Auszahlung</button></div>
      <p class="hint">${C("p3arest", k)}</p>
      <div class="grid2" style="margin-top:10px"><label class="field"><span>Rest im AHV-Bezug (${Number.isFinite(ahv) ? ymShow(i2ym(ahv)) : "?"})</span>${Sel(A + ".use", uses)}</label></div>
      <p class="hint">Auszahlungen frühestens 5 Jahre vor dem AHV-Bezug (ab ${earliest}); ohne Betrag wird das ganze Guthaben ausbezahlt. Der Rest wird spätestens im Monat des AHV-Bezugs ausbezahlt. Jede Auszahlung ist ein Kapitalbezug (Kapitalbezugssteuer, im selben Jahr mit dem PK-Kapitalbezug zusammengerechnet). «Hypothek zurückzahlen» gibt es nur mit einer Tranche mit indirekter Amortisation in dieses Konto.</p>
      <p class="hint">${lastSim ? (events.length ? events.map(e => `Vorschau ${ymShow(i2ym(e.m))}: ${esc(e.text)}`).join("<br>") : "Vorschau: keine Auszahlung im Simulationszeitraum.") : ""}</p></div>`;
  }).join("");
  return `<h2>Säule 3a</h2><p class="lead">Gebundene Vorsorge je Person: Guthaben, Einzahlung und Auszahlungsplan. Die indirekte Amortisation einer Hypothek fliesst bis zum AHV-Bezug in dieses Konto.</p>${cards}
    <p class="hint">Der Maximalbetrag der Einzahlung (${fmtInt(l.pillar3aMax)} pro Jahr, zusammen mit der indirekten Amortisation) ist in den <a href="#" data-action="settings">Einstellungen</a> änderbar.</p>`;
};

R.flows = () => `
  <h2>Einnahmen & Ausgaben</h2><p class="lead">Monatliche Beträge. Ausgaben werden in heutigen Preisen erfasst und mit der Inflation fortgeschrieben.</p>
  <div class="card">${Series("flows.income", state.partner ? `Erwerbseinkommen netto / Monat · ${esc(state.household.persons[0].name)}` : "Erwerbseinkommen netto / Monat", "chf")}
    <div style="margin:-4px 0 12px">${Chk("flows.incomeIndexed", INCOME_INDEXED)}</div>
    ${state.partner ? `${Series("partner.income", `Erwerbseinkommen netto / Monat · ${esc(state.household.persons[1].name)}`, "chf")}
    <div style="margin:-4px 0 12px">${Chk("partner.incomeIndexed", INCOME_INDEXED)}</div>` : ""}
    ${Series("flows.spending", "Lebenshaltungskosten / Monat inklusive Miete bzw. Wohnkosten (Zins, Amortisation, Nebenkosten)", "chf", state.partner ? "Haushalt, in heutigen Preisen" : "in heutigen Preisen")}
    <p class="hint">${C("housingshare")}</p></div>
  <div class="card"><h3>Einmalige Beträge <span class="sub">positiv = Zufluss, negativ = Ausgabe</span></h3>
    <table class="tbl"><tr><th>Datum</th><th>Betrag</th><th>Beschreibung</th><th></th></tr>
    ${state.flows.extras.map((x, i) => `<tr><td style="width:130px">${F(`flows.extras.${i}.date`, "", "month", { bare: true })}</td><td style="width:200px">${F(`flows.extras.${i}.amount`, "", "chf", { bare: true })}</td><td>${F(`flows.extras.${i}.label`, "", "text", { bare: true })}</td>
      <td><button class="btn ghost small" data-action="list-del" data-path="flows.extras" data-i="${i}">✕</button></td></tr>`).join("")}</table>
    <div style="margin-top:10px"><button class="btn small" data-action="extra-add">+ Einmaliger Betrag</button></div></div>`;

R.wealth = () => `
  <h2>Vermögen & Anlage</h2><p class="lead">Bestände per Simulationsstart und Aufteilung des Portfolios.</p>
  <div class="card"><h3>Bestände per Simulationsstart (${startTxt()})</h3><div class="grid3">${F("wealth.stocks", "Aktien", "chf", { hint: C("wquote", "stocks") })}${F("wealth.bonds", "Obligationen", "chf", { hint: C("wquote", "bonds") })}${F("wealth.cash", "Cash", "chf")}</div>
    <p class="hint">${C("wealthsum")}</p></div>
  <div class="card">${Series("wealth.stockShare", "Aktienquote (Aktien am Portfolio aus Aktien und Obligationen)", "pct")}
    <p class="hint">Cash zählt nicht zum Portfolio. Bis zur Pensionierung (Bezug Pensionskasse der ersten Person) hält die App so viel Cash wie oben erfasst, mit der Teuerung fortgeschrieben; ein Überschuss geht nach der Aktienquote ins Portfolio, ein Fehlbetrag wird daraus nachgefüllt. Ab der Pensionierung bestimmt die Entnahmestrategie das Cash: «Standard» mit der Cash-Reserve (Abschnitt «Entnahmestrategie»), alle anderen mit ${FIXED_RESERVE_MONTHS} Monaten bzw. die 3-Topf-Strategie mit ihren Töpfen.</p></div>
  ${strategyCard()}`;

/** FEA-53: Indizes der Anlagestrategie: [Schlüssel, Name, Erklärung]. */
const STOCK_INDICES = [
  ["worldJst", "Welt (JST)", "Aktien von 11 Ländern (Schweiz, USA, EU, Japan, Australien), nach BIP gewichtet, in CHF ohne Währungsabsicherung. Jährlich 1900–2020."],
  ["worldCustom", "Welt individuell (JST)", "Wie «Welt (JST)», aber mit eigenen Gewichten für Schweiz, USA, EU und Asien-Pazifik. Jährlich 1900–2020."],
  ["chJst", "Schweiz (JST)", "Schweizer Aktien (Gesamtrendite). Jährlich 1900–2020."],
  ["sp500", "S&P 500 (Shiller)", "US-Aktien in CHF, ohne Währungsabsicherung. Monatlich ab 1922."]
];
const BOND_INDICES = [
  ["ch", "Eidgenossenschaft 10 Jahre", "10-jährige Obligationen der Eidgenossenschaft: JST bis 1988, danach SNB. Jährlich 1900–2024; mit S&P 500 monatlich ab 1988."],
  ["usa", "USA 10 Jahre (Shiller)", "10-jährige US-Staatsanleihen in CHF, ohne Währungsabsicherung. Monatlich ab 1922."],
  ["chShort", "Eidgenossenschaft kurzfristig", "Kurzfristige Papiere der Eidgenossenschaft (etwa 3 Monate): JST bis 2020, danach SARON. Rendite praktisch wie der Geldmarktsatz, kaum Schwankung. Jährlich 1900–2025."]
];
const REGION_KEYS = ["ch", "usa", "eu", "apac"];
function strategyCard() {
  const s = state.returns.strategy, desc = (list, k) => esc(list.find(x => x[0] === k)?.[2] || "");
  const weights = s.stocks !== "worldCustom" ? "" : `<div class="grid2">${F("returns.strategy.weights.ch", "Schweiz", "pct")}${F("returns.strategy.weights.usa", "USA", "pct")}
      ${F("returns.strategy.weights.eu", "EU", "pct", { hint: "Deutschland, Frankreich, Italien, Spanien, Niederlande, Belgien, Grossbritannien" })}${F("returns.strategy.weights.apac", "Asien-Pazifik", "pct", { hint: "Japan, Australien" })}</div>
    <span data-path="returns.strategy.weights"></span>
    <p class="hint">${C("wsum")} Innerhalb der EU und von Asien-Pazifik sind die Länder nach BIP gewichtet. Ändert sich das Gewicht der EU, ändert sich der Anteil jedes EU-Landes im gleichen Verhältnis (EU 40 % statt 20 %: jedes EU-Land doppelt so viel).</p>`;
  const info = infoPop(`<p>Die Rendite von Aktien und Obligationen stammt aus den gewählten Indizes; die Methode (fixe Renditen, historischer Verlauf, Monte Carlo) steht unter «Simulation».</p>
    <p>Ist ein Teil jährlich, rechnet die Strategie mit Jahreswerten über die Jahre, für die beide Indizes Werte haben. Alle Indizes sind in CHF; die Teuerung kommt aus der Quelle der Aktien.</p>
    <p>Die JST-Daten (Jordà-Schularick-Taylor Macrohistory Database) stehen unter der Lizenz CC BY-NC-SA 4.0.</p>`);
  return `<div class="card"><h3>Anlagestrategie ${info}</h3>
    <div class="grid2"><label class="field"><span>Aktien</span>${Sel("returns.strategy.stocks", STOCK_INDICES.map(([k, l]) => [k, l]), { rerender: 1 })}<span class="computed">${desc(STOCK_INDICES, s.stocks)}</span></label>
      <label class="field"><span>Obligationen</span>${Sel("returns.strategy.bonds", BOND_INDICES.map(([k, l]) => [k, l]), { rerender: 1 })}<span class="computed">${desc(BOND_INDICES, s.bonds)}</span></label></div>
    ${weights}
    <p class="hint">${C("strategyinfo")}</p>
    <div class="libbtns"><button class="btn small" data-action="import-shiller">Shiller-Daten aktualisieren…</button>
      <button class="btn small" data-action="import-snb">SNB-Daten aktualisieren…</button></div></div>`;
}

const SOURCES = [
  ["fixed", "Fixe Renditen", "Konstante Jahresrenditen. Gut für schnelle Abschätzungen.", true],
  ["historical", "Historisch ab Startzeitpunkt", "Ein historischer Verlauf ab einem gewählten Jahr bzw. Monat.", true],
  ["rolling", "Alle historischen Startzeitpunkte", "Jeder mögliche Start wird gerechnet (ersetzt «Iteration»).", true],
  ["bootstrap", "Monte Carlo · historisch", "Zufällige Blöcke aus der Zeitreihe (Länge zufällig), alle Werte aus demselben Monat.", true],
  ["parametric", "Monte Carlo · parametrisch", "Aktien mit Student-t-Verteilung und Kurssprüngen, Obligationen und Inflation mit der Korrelation der Anlagestrategie.", true]
];
const paramJoint = () => seriesInfo && !seriesInfo.error ? seriesInfo.parametric?.joint ?? null : null;
/** Welche Eingaben bei der gewählten Quelle tatsächlich verwendet werden. */
function returnsUse() {
  const r = state.returns, src = r.source, i = seriesInfo && !seriesInfo.error ? seriesInfo : null;
  const usesSaron = state.realestate.properties.some(p => p.status !== "rent" && (p.mortgage?.tranches || []).some(t => t.periods.some(d => d.type === "saron")));
  const withSeries = ["historical", "rolling", "bootstrap"].includes(src);
  // SARON aus der Reihe: nur ausserhalb ihres SARON-Zeitraums gilt der fixe Wert.
  const saronGap = !i || !i.saronRange || i.saronRange[0] > i.first || i.saronRange[1] < i.last;
  let extrapolate = false;
  if (src === "historical") {
    const need = i && isYM(r.startDate) ? ym2i(r.startDate) + (endM() - startM()) : Infinity;
    extrapolate = !i || need > ym2i(i.last);
  }
  return {
    series: withSeries,
    extrapolate,
    fixedStocks: src === "fixed" || (extrapolate && r.extrapolation.method === "fixed"),
    // FEA-79: Das parametrische Modell zieht Obligationen und Inflation aus der Reihe; fix nur ohne Reihe.
    fixedBonds: src === "fixed" || (src === "parametric" && !paramJoint()) || (extrapolate && r.extrapolation.method === "fixed"),
    fixedInflation: src === "fixed" || (src === "parametric" && !paramJoint()) || (extrapolate && r.extrapolation.method === "fixed"),
    fixedSaron: usesSaron && (!withSeries || saronGap || extrapolate),
    usesSaron
  };
}
R.returns = () => {
  const r = state.returns, src = r.source, u = returnsUse();
  const fields = [u.fixedStocks && F("returns.fixed.stocks", "Aktien", "pct"), u.fixedBonds && F("returns.fixed.bonds", "Obligationen", "pct"),
    u.fixedInflation && F("returns.fixed.inflation", "Inflation", "pct"), u.fixedSaron && F("returns.fixed.saron", "SARON (für SARON-Hypotheken)", "pct")].filter(Boolean);
  const fixedTitle = { fixed: "Fixe Renditen pro Jahr", parametric: u.fixedBonds ? "Obligationen, Inflation und SARON pro Jahr" : "SARON pro Jahr" }[src]
    || (u.fixedStocks ? "Werte für die Fortschreibung pro Jahr" : "SARON pro Jahr");
  const fixedHint = src === "fixed" ? "" : src === "parametric" ? (u.fixedBonds ? "Ohne Zeitreihe erzeugt das Modell nur Aktienrenditen; diese Werte gelten konstant." : "Der SARON gilt konstant.")
    : [u.fixedStocks && "Aktien, Obligationen und Inflation gelten, nachdem die Zeitreihe endet.", u.fixedSaron && "Der SARON gilt ausserhalb des SARON-Zeitraums der Zeitreihe."].filter(Boolean).join(" ");
  const fixed = fields.length ? `<div class="card"><h3>${fixedTitle}</h3><div class="grid2">${fields.join("")}</div>${fixedHint ? `<p class="hint">${fixedHint}</p>` : ""}</div>` : "";
  const series = u.series ? `<div class="card"><h3>Renditen aus der Anlagestrategie</h3><p style="margin:0">${C("strategyinfo")} <a href="#" data-action="goto" data-sec="wealth">Ändern unter «Vermögen &amp; Anlage»</a></p></div>` : "";
  let params = "";
  if (src === "historical") params = `<div class="card"><h3>Start</h3><div class="grid2">${F("returns.startDate", "Historischer Startmonat", "month")}</div>
    <p class="hint">Der Simulationsstart ${C("ms", "start")} wird auf diesen historischen Zeitpunkt gelegt. Bei Jahresdaten zählt nur das Jahr.</p></div>`;
  if (src === "rolling") params = `<div class="card"><h3>Durchlauf</h3><p style="margin:0">${C("rolling")}</p></div>`;
  if (src === "bootstrap") params = `<div class="card"><h3>Bootstrap</h3><div class="grid3">${F("returns.bootstrap.paths", "Anzahl Pfade", "num")}${F("returns.bootstrap.block", "Mittlere Blocklänge", "months")}${F("returns.bootstrap.seed", "Zufallsstartwert", "num")}</div>
    <p class="hint">Jeder Pfad wird aus Abschnitten der Zeitreihe zusammengesetzt. Ein Abschnitt endet in jedem Monat mit Wahrscheinlichkeit 1 / Blocklänge, dann folgt ein zufälliger neuer Startmonat (stationärer Bootstrap). Aktien, Obligationen, Inflation und SARON stammen immer aus demselben historischen Monat; Krisen behalten so ihren Verlauf. Bei Jahresdaten gilt die Blocklänge in ganzen Jahren.</p>
    <p class="hint">Gleicher Zufallsstartwert ⇒ gleiche Pfade. Szenarien verwenden dieselben Pfade, damit Unterschiede von den Parametern stammen.</p></div>`;
  if (src === "parametric") {
    const P = (k, label) => F("returns.parametric." + k, label, PARAM_KEYS[k], { hint: C("psug", k) });
    params = `<div class="card"><h3>Aktienmodell</h3><p class="hint" style="margin-top:0">${C("pauto")}</p><div class="grid3">${F("returns.parametric.paths", "Anzahl Pfade", "num")}${P("mu", "Erwartete Rendite (Drift)")}${P("sigma", "Volatilität")}
    ${P("df", "Freiheitsgrade (Fat Tails)")}${P("jumpLambda", "Kurssprünge pro Jahr")}${P("jumpMu", "Mittlere Sprunghöhe (log)")}
    ${P("jumpSigma", "Streuung der Sprunghöhe")}${F("returns.parametric.seed", "Zufallsstartwert", "num")}</div>
    <p class="hint">Tagesschritte (21 pro Monat), Student-t-Schocks auf Varianz 1 skaliert (0 Freiheitsgrade = Normalverteilung), Poisson-Kurssprünge. Eine Sprunghöhe von −30 % (log) entspricht etwa −26 % Kursverlust.</p></div>
    <div class="card"><h3>Obligationen und Inflation <span class="sub">aus der Anlagestrategie, nur Anzeige</span></h3><div id="paramJoint">${C("pjoint")}</div></div>`;
  }
  const extra = !u.extrapolate ? "" : `<div class="card"><h3>Fortschreibung <span class="sub">die Zeitreihe endet vor dem Simulationsende</span></h3>
    <div class="grid2"><label class="field"><span>Methode</span>${Sel("returns.extrapolation.method", [["fixed", "Fixe Werte"], ["average", "Gleitender Durchschnitt"], ["geometric", "Gleitender geometrischer Durchschnitt"]], { rerender: 1 })}</label>
    ${r.extrapolation.method === "fixed" ? "" : F("returns.extrapolation.range", "Durchschnitt über", "num", { unit: "Jahre" })}</div></div>`;
  return `<h2>Simulation</h2><p class="lead">Wie die Renditen und die Inflation über die Zeit verlaufen: fixe Werte, ein historischer Verlauf oder Monte Carlo. Die Indizes wählen Sie bei der Anlagestrategie unter «Vermögen &amp; Anlage». Angezeigt sind nur die Angaben, die für die gewählte Methode gelten.</p>
    <div class="card"><div class="choices">${SOURCES.map(([k, t, d, ok]) => `<div class="choice ${k === src ? "on" : ""} ${ok ? "" : "off"}" ${ok ? `data-action="set" data-path="returns.source" data-value="${k}"` : ""}><b>${t}</b><small>${d}</small></div>`).join("")}</div></div>
    ${series}${params}${extra}${fixed}
    <p class="hint">Die Teuerung wirkt jeden Monat (Jahresinflation geometrisch auf Monate umgerechnet); die AHV wird im Januar angepasst. ${src !== "fixed" ? `Die Vorschau rechts rechnet einen Lauf mit konstanten Werten: ${C("basis")}` : ""}</p>`;
};

const STRATS = [
  ["standard", "Standard", "Cash-Reserve in Monaten der ungedeckten Ausgaben, Rest nach Aktienanteil investiert, monatliches Rebalancing.", true],
  ["three_buckets", "3-Topf-Strategie", "Ab Pensionierung: Topf 1 Cash, Topf 2 Obligationen, Topf 3 Aktien. Nachfüllen im Januar und nach Zuflüssen, Notauffüllung von Topf 1 unterjährig.", true],
  ["guardrails", "Guardrails", "Ab Pensionierung: Startentnahme in % des Vermögens, danach mit der Teuerung angepasst; Leitplanken kürzen oder erhöhen die Ausgaben (Guyton-Klinger).", true],
  ["fixed_rate", "Fixe Entnahmerate", "Ab Pensionierung gibt der Haushalt jedes Jahr im Januar einen festen Anteil des liquiden Vermögens aus.", true],
  ["vpw", "Variable Prozententnahme (VPW)", "Jedes Jahr wird der Anteil des Vermögens entnommen, der es bis zum Planungsalter gleichmässig aufbraucht; der Anteil steigt mit dem Alter.", true],
  ["endowment", "Endowment-Regel", "Entnahme = Vorjahr mit Teuerung, gemischt mit einer Rate des aktuellen Vermögens (Yale/Tobin). Glättet die Ausgaben.", true]
];
R.withdrawal = () => {
  const w = state.withdrawal;
  return `<h2>Entnahmestrategie</h2><p class="lead">Wie nach der Pensionierung Geld aus dem Vermögen entnommen und umgeschichtet wird.</p>
  <div class="card"><div class="choices">${STRATS.map(([k, t, d, ok]) => `<div class="choice ${k === w.strategy ? "on" : ""} ${ok ? "" : "off"}" ${ok ? `data-action="set" data-path="withdrawal.strategy" data-value="${k}"` : ""}><b>${t}</b><small>${d}</small></div>`).join("")}</div></div>
  ${w.strategy === "three_buckets" ? `<div class="card"><h3>Töpfe</h3><div class="grid3">${F("withdrawal.buckets.cashYears", "Topf 1 · Cash", "years")}${F("withdrawal.buckets.bondYears", "Topf 2 · Obligationen", "years")}${F("withdrawal.buckets.refillThreshold", "Aus Topf 3 nur verkaufen, wenn Aktienrendite (12 Monate) ≥", "pct")}${F("withdrawal.buckets.reserveMonths", "Reserve in Topf 1 unterjährig", "months")}</div>
    <div class="buckets">
      <div class="bucket"><div class="bn">Topf 1 · Cash</div><div class="bv">${C("bucket", "cash")}</div><div class="bar" style="background:var(--c3)"></div></div>
      <div class="bucket"><div class="bn">Topf 2 · Obligationen</div><div class="bv">${C("bucket", "bond")}</div><div class="bar" style="background:var(--c2)"></div></div>
      <div class="bucket"><div class="bn">Topf 3 · Aktien</div><div class="bv">${C("bucket", "stock")}</div><div class="bar" style="background:var(--c1)"></div></div></div>
    <p class="hint">Aufteilung bei Pensionierung gemäss Vorschau (fixe Renditen). Die Töpfe sind Jahre der <b>ungedeckten</b> Ausgaben (Ausgaben inkl. Wohnkosten − Einkommen und Renten).</p>
    <p class="hint">Regeln: Topf 1 zahlt die laufenden Ausgaben. Jeden Januar wird Topf 1 aus Topf 2 auf das Ziel gebracht, Topf 2 aus den Aktien – aber nur, wenn die Aktien im Vorjahr mindestens die Schwelle erreicht haben. Überschüsse fliessen in die Aktien. Reicht Topf 1 zwischendurch nicht, wird aus Topf 2 und dann aus den Aktien entnommen. Vor der Pensionierung gilt die Cash-Regel aus «Vermögen & Anlage».</p>
    <div class="grid2"><label class="field"><span>Ausgaben ab Pensionierung</span>${Sel("withdrawal.buckets.spending", [["planned", "wie geplant"], ["guardrails", "mit Guardrails steuern"]], { rerender: 1 })}</label></div>
    <p class="hint">${bucketsGuardrails() ? "Die Guardrails bestimmen die Lebenshaltung, die Töpfe die Anlage. Die Topfziele rechnen mit der Lebenshaltung der Guardrails." : "Die Ausgaben bleiben wie geplant. Mit Guardrails folgen sie dem Vermögen, die Töpfe bleiben."}</p></div>
    ${bucketsGuardrails() ? GuardrailsCard() + WithdrawalLimits() : ""}`
    : w.strategy === "fixed_rate" ? `<div class="card"><h3>Fixe Entnahmerate</h3><div class="grid3">${F("withdrawal.fixedRate.rate", "Entnahmerate pro Jahr", "pct")}</div>
      <p class="hint">${C("wdinit")}</p>
      <p class="hint">Jeden Januar (und bei Pensionierung) wird die Jahresentnahme neu auf diese Rate des liquiden Vermögens gesetzt. Die Entnahme ist <b>Lebenshaltung + Wohnkosten − Einkommen und Renten</b> (vor Steuern): Die Lebenshaltung ergibt sich daraus und folgt dem Vermögen; geplante Ausgaben gelten nur bis zur Pensionierung. Das Vermögen wird so nie ganz aufgebraucht – dafür schwanken die Ausgaben. Mit einer Untergrenze lässt sich das begrenzen.</p></div>${WithdrawalLimits()}`
    : w.strategy === "guardrails" ? GuardrailsCard() + WithdrawalLimits()
    : w.strategy === "vpw" ? `<div class="card"><h3>Variable Prozententnahme <span class="sub">VPW</span></h3><div class="grid2">${F("withdrawal.vpw.horizonAge", "Vermögen soll reichen bis Alter", "num", { unit: "J." })}${F("withdrawal.vpw.realReturn", "Angenommene Realrendite pro Jahr", "pct")}</div>
      <p class="hint">${C("wdinit")}</p>
      <p class="hint">Jeden Januar wird der Anteil des liquiden Vermögens entnommen, der es bei der angenommenen Realrendite bis zum Planungsalter gleichmässig aufbraucht (Annuität). Der Anteil steigt mit dem Alter; im Planungsalter wäre alles aufgebraucht. Die Ausgaben folgen dem Vermögen und schwanken entsprechend. Eine Untergrenze begrenzt das.</p></div>${WithdrawalLimits()}`
    : w.strategy === "endowment" ? `<div class="card"><h3>Endowment-Regel <span class="sub">Yale / Tobin</span></h3><div class="grid2">${F("withdrawal.endowment.rate", "Zielrate des Vermögens", "pct")}${F("withdrawal.endowment.smoothing", "Gewicht des aktuellen Vermögens", "pct")}</div>
      <p class="hint">${C("wdinit")}</p>
      <p class="hint">Entnahme im Januar = (1 − Gewicht) × Vorjahresentnahme mit Teuerung + Gewicht × Zielrate × aktuelles Vermögen. Bei 100 % entspricht das der fixen Rate, bei kleinem Gewicht schwankt die Entnahme wenig, folgt dem Vermögen aber langsam.</p></div>${WithdrawalLimits()}`
    : `<div class="card"><h3>Standard</h3>${Series("wealth.reserveMonths", "Cash-Reserve", "months", "ab der Pensionierung, Monate der ungedeckten Ausgaben (Ausgaben − Einkommen)")}
      <p class="hint">Der Aktienanteil steht unter <a href="#" data-action="goto" data-sec="wealth">Vermögen & Anlage</a>. Alle anderen Strategien rechnen mit einer festen Reserve von ${FIXED_RESERVE_MONTHS} Monaten.</p></div>`}`;
};

/** FEA-05: 3-Topf, deren Ausgaben die Guardrails steuern. */
const bucketsGuardrails = (w = state.withdrawal) => w?.strategy === "three_buckets" && w.buckets?.spending === "guardrails";
/** Schlüssel der Entnahmestrategie einer Planung; 3-Topf mit Guardrails hat einen eigenen (wie in der Optimierung). */
const stratKey = w => bucketsGuardrails(w) ? "three_buckets_guardrails" : w?.strategy;
function GuardrailsCard() {
  return `<div class="card"><h3>Guardrails <span class="sub">nach Guyton-Klinger</span></h3><div class="grid2">${F("withdrawal.guardrails.initialRate", "Startrate bei Pensionierung", "pct")}${F("withdrawal.guardrails.adjust", "Anpassung bei Leitplanke", "pct")}
      ${F("withdrawal.guardrails.upper", "Obere Leitplanke (Rate höher als Startrate um)", "pct")}${F("withdrawal.guardrails.lower", "Untere Leitplanke (Rate tiefer als Startrate um)", "pct")}</div>
      <p class="hint">${C("wdinit")}</p>
      <p class="hint">Bei der Pensionierung der ersten Person wird die Startrate auf das liquide Vermögen angewendet (ohne einen für einen Kauf reservierten Verkaufserlös). Jeden Januar wird die Entnahme mit der Teuerung angepasst – ausser nach einem Verlustjahr der Aktien, solange die aktuelle Rate über der Startrate liegt. Übersteigt die aktuelle Rate (Entnahme / Vermögen) die Startrate um die obere Leitplanke, wird die Entnahme gekürzt; liegt sie um die untere Leitplanke darunter, wird sie erhöht. Entnahme = <b>Lebenshaltung + Wohnkosten − Einkommen und Renten</b> (vor Steuern).</p></div>`;
}
function WithdrawalLimits() {
  return `<div class="card"><h3>Grenzen der Lebenshaltung <span class="sub">Anteil der geplanten Ausgaben, 0 = keine Grenze</span></h3><div class="grid2">${F("withdrawal.floor", "Untergrenze", "pct")}${F("withdrawal.ceiling", "Obergrenze", "pct")}</div>
    <p class="hint">Beispiel: Untergrenze 80 % – die Lebenshaltung sinkt nie unter 80 % der geplanten Ausgaben (in heutigen Preisen, mit der Teuerung angepasst), auch wenn die Strategie weniger erlauben würde. Dann kann das Vermögen aufgebraucht werden.</p></div>`;
}
function TaxTable(key, label, xlabel) {
  return `<div class="series"><div class="shead"><span class="t">${label}</span></div><div class="sbody"><div><table class="tbl"><tr><th>ab ${xlabel}</th><th>Steuersatz</th><th></th></tr>
  ${getP("taxes." + key).map((r, i) => `<tr><td>${F(`taxes.${key}.${i}.0`, "", "chf", { bare: true })}</td><td>${F(`taxes.${key}.${i}.1`, "", "pct", { bare: true })}</td><td><button class="btn ghost small" data-action="list-del" data-path="taxes.${key}" data-i="${i}">✕</button></td></tr>`).join("")}</table>
  <div style="margin-top:8px"><button class="btn small" data-action="tax-add" data-key="${key}">+ Stufe</button></div></div>
  <svg class="mini" data-taxmini="${key}" viewBox="0 0 300 64" preserveAspectRatio="none" style="height:110px"></svg></div></div>`;
}
// ---------- Optimierung ----------
const OPT_STRATS = [["standard", "Standard"], ["three_buckets", "3-Topf"], ["three_buckets_guardrails", "3-Topf + Guardrails"], ["guardrails", "Guardrails"], ["fixed_rate", "Fixe Rate"], ["vpw", "VPW"], ["endowment", "Endowment"]];
const OPT_DEFAULT = { goal: "consumption", minConsumption: 90, maxRuin: 5, ruinAge: "", weightMedian: 50, volWeight: 1, maxVol: "", paths: 5000, checkPaths: 10000, samples: 30, strategies: OPT_STRATS.map(x => x[0]).filter(k => k !== "three_buckets_guardrails"), ranges: {} };
let optCfg = { ...OPT_DEFAULT };
const OPT_ACTIONS = {};
let lastOpt = null, optBusy = false, optError = null;
// Die Einstellungen liegen in der Planung (`optimization`, Anteile als Dezimalzahl) und werden mit ihr gespeichert.
function optFromPlan() {
  const o = state.optimization; if (!o) return { ...OPT_DEFAULT };
  const d = OPT_DEFAULT;
  return { goal: o.goal ?? d.goal, minConsumption: optRound((o.minConsumption ?? 0.9) * 100), maxRuin: optRound((o.maxRuin ?? 0.05) * 100), ruinAge: o.ruinAge ?? "", weightMedian: optRound((o.weightMedian ?? 0.5) * 100), volWeight: o.volWeight ?? 1,
    maxVol: o.maxVol == null ? "" : optRound(o.maxVol * 100), paths: o.paths || d.paths, checkPaths: o.checkPaths || d.checkPaths, samples: o.samples ?? d.samples,
    strategies: o.strategies?.length ? o.strategies : d.strategies.slice(), ranges: JSON.parse(JSON.stringify(o.ranges ?? {})) };
}
function optToPlan(c) {
  return { ...(c.goal && c.goal !== "consumption" ? { goal: c.goal } : {}), ...(c.minConsumption !== 90 ? { minConsumption: optRound(optNum(c.minConsumption, 90) / 100) } : {}),
    maxRuin: optRound(c.maxRuin / 100), ruinAge: c.ruinAge === "" ? null : optNum(c.ruinAge, null), weightMedian: optRound(clamp(c.weightMedian / 100, 0, 1)), volWeight: c.volWeight,
    maxVol: c.maxVol === "" ? null : optRound(optNum(c.maxVol, 0) / 100), paths: c.paths, checkPaths: c.checkPaths, samples: c.samples, strategies: c.strategies,
    ...(Object.keys(c.ranges ?? {}).length ? { ranges: c.ranges } : {}) };
}
/** Schlüssel der Planung ohne die Optimierungs-Einstellungen: Sie ändern das Ergebnis einer früheren Optimierung nicht. */
const optKey = () => JSON.stringify({ ...state, optimization: undefined });
function saveOptCfg() {
  const next = optToPlan(readOptCfg());
  if (JSON.stringify(next) !== JSON.stringify(state.optimization)) { state.optimization = next; markDirty(); }
}
document.addEventListener("change", e => {
  if (e.target?.id && /^opt(S)?-/.test(e.target.id) && e.target.closest("#main")) saveOptCfg();
  if (e.target?.id === "opt-goal") renderSection();
});
document.addEventListener("input", e => { if (e.target?.id && /^opt-/.test(e.target.id) && e.target.closest("#main")) updateEstimates(); });
document.addEventListener("change", e => { if (e.target?.id && /^optS-/.test(e.target.id) && e.target.closest("#main")) updateEstimates(); });

// ---------- Dauerschätzung ----------
// Gemessen wird die Zeit eines Laufs dieser Planung auf diesem Rechner (Befehl bench_plan). Daraus rechnen wir die Läufe
// hoch, die Szenarien und Optimierung brauchen. Die Suchschritte je Strategie stammen aus Messungen mit 30 Startpunkten.
const OPT_SEARCH_EVALS = { standard: 22, fixed_rate: 14, three_buckets: 46, three_buckets_guardrails: 110, endowment: 40, vpw: 74, guardrails: 90 };
// ----- Fortschritt und Abbruch laufender Berechnungen -----
let job = null;
/** Kennung der nächsten Berechnung; mit der Zeit begonnen, damit sie nach einem Neuladen nicht mit einer alten zusammenfällt. */
let jobSeq = Date.now();
/** HTML der Fortschrittsanzeige; `jobTick` füllt sie während der Berechnung. */
const jobCard = (title, note = "") => `<div class="card job"><h3 style="margin:0">${title}</h3><progress id="jobBar" max="1000" value="0"></progress>
  <div class="row"><span class="hint" id="jobTxt" style="margin:0">Startet…</span><button class="btn small" data-action="job-cancel" id="jobCancel">Abbrechen</button></div>${note ? `<p class="hint" style="margin:0">${note}</p>` : ""}</div>`;
async function jobTick() {
  if (!job) return;
  let p; try { p = await Engine.call("progress_get", { jobId: job.id }); } catch { return; }
  const bar = document.getElementById("jobBar"), txt = document.getElementById("jobTxt");
  if (!job || !bar || !txt) return;
  const frac = p.total > 0 ? Math.min(0.99, p.done / p.total) : 0, el = (performance.now() - job.t0) / 1000;
  bar.value = Math.round(frac * 1000);
  txt.textContent = p.cancelled ? "Wird abgebrochen…" : `${Math.round(frac * 100)} % · ${fmtShort(el)} vergangen` + (frac > 0.05 && el > 2 ? ` · noch ca. ${fmtShort(el / frac - el)}` : "");
}
/** Führt `fn` aus und zeigt dabei den Fortschritt; gibt `undefined` zurück, wenn abgebrochen wurde. */
async function withJob(fn) {
  job = { t0: performance.now(), id: ++jobSeq };
  job.timer = setInterval(jobTick, 250); document.documentElement.dataset.job = "on";
  try { return await fn(); } finally { clearInterval(job.timer); job = null; document.documentElement.dataset.job = "off"; }
}
const isCancelled = e => errText(e) === "Abgebrochen";
const jobCancel = async () => {
  const b = document.getElementById("jobCancel"); if (b) b.disabled = true;
  try { await Engine.call("cancel_job", { jobId: job?.id }); } catch (e) { toast(errText(e)); }
};
// `off`: Testzugang #nobench (FEA-87). Ohne Messung zeigt die App «Dauer wird geschätzt…» und rechnet ohne Rückfrage.
const bench = { key: "", us: null, gen: 0, fixedPaths: null, cores: 0, busy: false, error: null, off: false };
const benchKey = () => JSON.stringify({ ...state, optimization: undefined, scenarios: undefined });
/** Die Pfade der Suche werden einmal erzeugt und dann je Kandidat nur gerechnet; die Prüfpfade entstehen bei jeder Auswertung neu. */
/** Auswertungen je Strategie; mit weniger variierten Parametern sind es weniger Suchschritte. */
function optEvals(k, c) {
  const s = optCat.data?.find(x => x.key === k), search = OPT_SEARCH_EVALS[k] ?? 60;
  if (!s) return c.samples + 2 + search;
  const varied = s.params.filter(d => !(c.ranges?.[k]?.[d.key]?.fixed ?? d.fixed)).length, usual = Math.max(1, s.params.filter(d => !d.fixed).length);
  return varied === 0 ? 1 : c.samples + 2 + search * varied / usual;
}
function optSeconds(c) { return optSecondsFor(bench, c, c.strategies.reduce((a, k) => a + optEvals(k, c), 0)); }
/** Geschätzte Rechenzeit, ab der in der Karte «Umfang» ein Warnsymbol erscheint (die Rückfrage vor dem Start hat ihre eigene Grenze). */
const SLOW_RUN_SECONDS = 90;
function scenSeconds() {
  const sw = state.scenarios.sweeps, per = bench.fixedPaths ?? runsPerScenario(), combos = sw.map(sweepCount).reduce((a, b) => a * b, 1);
  return { runs: (sw.length ? combos : 1) * per, sec: (sw.length ? combos : 1) * per * bench.us / 1e6 };
}
function updateEstimates() {
  const o = document.getElementById("est-opt"), sc = document.getElementById("est-scen");
  const er = document.getElementById("est-run");
  if (er) { const { sec } = bench.us == null ? { sec: null } : scenSeconds(); er.textContent = sec == null ? "" : sec < 1 ? "Dauer: unter 1 s" : `Dauer: ca. ${fmtShort(sec)}`; }
  const wait = bench.error ? `Dauer nicht schätzbar (${esc(bench.error)}).` : bench.us == null ? "Dauer wird geschätzt…" : null;
  if (o) {
    const c = document.getElementById("opt-maxRuin") ? readOptCfg() : optFromPlan();
    o.innerHTML = wait ?? (c.strategies.length && c.paths > 0 && c.checkPaths > 0
      ? `Geschätzte Dauer der Optimierung: ${fmtEstimate(optSeconds(c))}. Sie steigt mit der Anzahl Pfade, Startpunkte und Strategien.` : "");
  }
  const ew = document.getElementById("est-warn");
  if (ew) {
    const sec = bench.us == null ? null : scenSeconds().sec, slow = sec != null && sec > SLOW_RUN_SECONDS;
    ew.innerHTML = slow ? "⚠" : "";
    ew.title = slow ? `Die geschätzte Rechenzeit beträgt ca. ${fmtShort(sec)} und liegt über ${SLOW_RUN_SECONDS} s.` : "";
    ew.setAttribute("role", slow ? "img" : "presentation"); ew.setAttribute("aria-label", ew.title);
  }
  if (sc) {
    const { runs, sec } = scenSeconds(), plain = !state.scenarios.sweeps.length;
    sc.innerHTML = wait ?? `Geschätzte Dauer ${plain ? "der Simulation" : "von «Simulation starten» (Szenarien)"}: ${fmtEstimate(sec, [0.7, 1.5])}${plain ? "" : `, ${fmtInt(runs)} Läufe`}.`;
  }
}
async function ensureBench() {
  const key = benchKey();
  if (bench.off || job || bench.busy || (bench.key === key && (bench.us != null || bench.error))) return;
  bench.busy = true;
  try {
    const r = await Engine.call("bench_plan", { plan: state, path: currentPath });
    Object.assign(bench, { key, us: r.usPerRun, gen: r.usGenerate, fixedPaths: r.fixedPaths, cores: r.cores, error: null });
  } catch (e) { Object.assign(bench, { key, us: null, error: errText(e) }); }
  bench.busy = false;
  updateEstimates();
  if (benchKey() !== bench.key) scheduleBench();
}
/** Die Messung läuft kurz nach der letzten Änderung, damit sie beim Tippen nicht ständig startet. */
function scheduleBench() { if (bench.off) return; clearTimeout(scheduleBench.t); scheduleBench.t = setTimeout(ensureBench, 800); }
function optInput(id, label, unit, hint = "") {
  return `<label class="field"><span>${label}</span><div class="inp"><input class="num" id="opt-${id}" value="${esc(optCfg[id])}">${unit ? `<span class="unit">${unit}</span>` : ""}</div>${hint ? `<span class="computed">${hint}</span>` : ""}</label>`;
}
// ----- Parameter der Strategien: fix oder in einer Bandbreite variieren -----
// Der Katalog (Befehl opt_params) nennt je Strategie die Parameter mit Einheit, Standard-Bandbreite und Wert der Planung.
const optCat = { key: "", data: null, busy: false };
const optCatKey = () => JSON.stringify([state.withdrawal, state.wealth.stockShare, state.wealth.reserveMonths]);
async function ensureOptCatalog() {
  const key = optCatKey();
  if (optCat.busy || (optCat.key === key && optCat.data)) return;
  optCat.busy = true;
  try { optCat.data = await Engine.call("opt_params", { plan: state }); optCat.key = key; } catch (e) { noteError(errText(e)); }
  optCat.busy = false;
  if (current === "optimize" && !optBusy) renderSection();
  else updateEstimates();
}
const rangeOf = (s, d) => {
  const r = optCfg.ranges?.[s]?.[d.key] ?? {};
  return { fixed: r.fixed ?? d.fixed, lo: r.lo ?? d.lo, hi: r.hi ?? d.hi };
};
const fmtNum = v => v.toLocaleString("de-CH", { maximumFractionDigits: 2 });
const rangeShow = (d, v) => d.percent ? optRound(v * 100) : v;
const rangeUnit = d => d.unit === "%" ? "%" : d.unit;
const variedParams = s => s.params.filter(d => !rangeOf(s.key, d).fixed);
function optRangesHtml() {
  if (!optCat.data) return `<p class="hint">Parameter werden geladen…</p>`;
  return optCat.data.map(s => {
    const n = variedParams(s).length, sel = optCfg.strategies.includes(s.key);
    const rows = s.params.map(d => {
      const r = rangeOf(s.key, d), id = `optR-${s.key}-${d.key}`, off = r.fixed ? "disabled" : "";
      return `<tr><td>${esc(d.label)}</td><td class="r">${d.current == null ? "–" : `${fmtNum(rangeShow(d, d.current))} ${rangeUnit(d)}`}</td>
        <td><label class="check"><input type="checkbox" id="${id}-var" ${r.fixed ? "" : "checked"}> variieren</label></td>
        <td style="white-space:nowrap">von <input class="num" id="${id}-lo" value="${rangeShow(d, r.lo)}" ${off} style="width:68px"> bis <input class="num" id="${id}-hi" value="${rangeShow(d, r.hi)}" ${off} style="width:68px"> ${rangeUnit(d)}</td></tr>`;
    }).join("");
    return `<details class="optrange" ${sel ? "" : "style=\"opacity:.7\""}><summary><b>${esc(s.name)}</b> <span class="computed">${n === 0 ? "alle Parameter fix" : `${n} von ${s.params.length} Parametern variiert`}${sel ? "" : " · Strategie nicht gewählt"}</span></summary>
      <table class="tbl"><tr><th>Parameter</th><th class="r">in der Planung</th><th></th><th>Bandbreite der Suche</th></tr>${rows}</table>
      <div style="margin:6px 0 10px"><button class="btn small" data-action="opt-range-reset" data-s="${s.key}">Standard-Bandbreiten</button></div></details>`;
  }).join("");
}
function onRangeInput(e) {
  const m = /^optR-([a-z_]+)-([A-Za-z]+)-(var|lo|hi)$/.exec(e.target.id || ""); if (!m || !optCat.data) return;
  const [, sk, pk, what] = m, d = optCat.data.find(s => s.key === sk)?.params.find(p => p.key === pk); if (!d) return;
  const cur = optCfg.ranges?.[sk]?.[pk] ?? {}, id = `optR-${sk}-${pk}`;
  const next = { ...cur };
  if (what === "var") { next.fixed = !e.target.checked; for (const x of ["lo", "hi"]) document.getElementById(`${id}-${x}`).disabled = next.fixed; }
  else {
    const v = optNum(e.target.value, NaN); if (!Number.isFinite(v)) return;
    next[what] = d.percent ? optRound(v / 100) : v;
  }
  optCfg.ranges = { ...(optCfg.ranges ?? {}), [sk]: { ...(optCfg.ranges?.[sk] ?? {}), [pk]: next } };
  saveOptCfg(); updateEstimates();
  const s = optCat.data.find(x => x.key === sk), sum = e.target.closest("details")?.querySelector("summary .computed");
  if (sum) { const n = variedParams(s).length; sum.textContent = (n === 0 ? "alle Parameter fix" : `${n} von ${s.params.length} Parametern variiert`) + (optCfg.strategies.includes(sk) ? "" : " · Strategie nicht gewählt"); }
}
document.addEventListener("change", e => { if (/^optR-/.test(e.target.id || "") && e.target.closest("#main")) onRangeInput(e); });
OPT_ACTIONS["opt-range-reset"] = d => {
  const r = { ...(optCfg.ranges ?? {}) }; delete r[d.s]; optCfg.ranges = r; saveOptCfg(); renderSection();
};
const optParam = ([, label, v, pc]) => `${label} ${pc ? (v * 100).toLocaleString("de-CH", { maximumFractionDigits: 1 }) + " %" : v.toLocaleString("de-CH", { maximumFractionDigits: 1 })}`;
function optMetricsRow(name, m, goal, best, extra = "", cls = "") {
  return `<tr class="${cls}"><td><b>${esc(name)}</b></td><td class="r">${pctTxt(m.ruin)}</td><td class="r">${fmtInt(m.median)}</td><td class="r">${fmtInt(m.p10)}</td><td class="r">${pctTxt(m.volatility)}</td>
    <td class="r">${fmtInt(goal)}<div style="height:4px;background:var(--accent);opacity:.5;width:${Math.max(0, Math.min(100, goal / best * 100))}%;margin-top:2px;border-radius:2px"></div></td><td class="r">${pctTxt(m.drawRate ?? 0)}</td><td>${extra}</td></tr>`;
}
/** Ergebnis für Ziel 2 «Kein Notverkauf von Immobilien»: je Strategie der Anteil der Läufe mit ungeplantem Verkauf. */
function optResultKeep(r, q, stale) {
  const row = (name, m, extra, cls = "") => `<tr class="${cls}"><td><b>${esc(name)}</b></td><td class="r"><b>${pctTxt(m.emergencyShare)}</b></td><td class="r">${pctTxt(m.ruin)}</td><td class="r">${fmtInt(m.median)}</td>
    <td class="r">${m.planned > 0 ? pctTxt(m.median / m.planned) : "–"}</td><td>${extra}</td></tr>`;
  const none = !r.outcomes.some(x => x.feasible);
  return `<div class="card"><h3>Ergebnis · Ziel 2 «Kein Notverkauf von Immobilien» <span class="sub">geprüft auf ${fmtInt(r.checkPaths)} unabhängigen Pfaden · Konsum pro Jahr in Preisen ${ymShow(i2ym(startM()))}</span></h3>
    ${stale ? `<p class="hint" style="color:var(--warn);margin:0 0 8px">Die Planung wurde seit dieser Optimierung geändert – bitte neu starten.</p>` : ""}
    ${none ? `<p class="hint" style="color:var(--warn);margin:0 0 8px">Keine Strategie erfüllt die Nebenbedingungen (Ruin höchstens ${pctTxt(q.maxRuin)}, Median des Konsums mindestens ${pctTxt(q.minConsumption)} der geplanten Ausgaben). Angezeigt ist die Strategie, die ihnen am nächsten kommt.</p>` : ""}
    <div class="tscroll"><table class="tbl"><tr><th>Strategie</th><th class="r" title="Anteil der Läufe mit mindestens einem ungeplanten Immobilienverkauf (Notverkauf)">Ungeplanter Verkauf</th><th class="r">Ruin</th><th class="r">Median Konsum</th><th class="r" title="Median des Konsums im Verhältnis zu den geplanten Ausgaben">Anteil der Planung</th><th></th></tr>
    ${row(`Ausgangslage (${r.reference.strategy})`, r.reference.metrics, "wie in der Planung")}
    ${r.outcomes.map((x, i) => row(x.strategy + (i === 0 && x.feasible ? " ★" : ""), x.validated,
      `<small>${x.params.map(optParam).join(", ")}</small>${x.fixed?.length ? `<br><small class="computed">fix: ${x.fixed.map(optParam).join(", ")}</small>` : ""}<div style="margin-top:4px">${x.feasible ? "" : `<span style="color:var(--err)">Nebenbedingungen nicht eingehalten</span> `}<button class="btn small" data-action="opt-apply" data-i="${i}">In Planung übernehmen</button></div>`, x.feasible ? "" : "off")).join("")}
    </table></div>
    <p class="hint">Reihenfolge: zuerst der geringste Anteil an Läufen mit ungeplantem Verkauf, bei Gleichstand die geringere Ruinwahrscheinlichkeit. Ruin = Anteil der Pfade, in denen das Vermögen${q.ruinAge ? ` bis Alter ${q.ruinAge}` : " bis zum Simulationsende"} aufgebraucht ist. Rechenzeit ${(r.elapsedMs / 1000).toFixed(1)} s. Dies ist eine Modellrechnung, keine Anlageberatung.</p></div>`;
}
R.optimize = () => {
  optCfg = optFromPlan();
  const o = lastOpt, stale = o && optKey() !== o.planJson;
  const single = ["fixed", "historical"].includes(state.returns.source);
  let res = "";
  if (optBusy) res = jobCard("Optimierung läuft…", "Je nach Anzahl Pfade und Strategien dauert das einige Sekunden bis eine Minute. Die Berechnung läuft lokal auf diesem Rechner.");
  else if (optError) res = `<div class="card errcard">${esc(optError)}</div>`;
  else if (o) {
    const r = o.report, q = o.req, w = q.weightMedian;
    const goal = m => (w * m.median + (1 - w) * m.p10) * Math.max(0, 1 - q.volWeight * m.volatility);
    const best = Math.max(1, goal(r.reference.metrics), ...r.outcomes.map(x => goal(x.validated)));
    if (q.goal === "keepProperty") res = optResultKeep(r, q, stale);
    else res = `<div class="card"><h3>Ergebnis <span class="sub">geprüft auf ${fmtInt(r.checkPaths)} unabhängigen Pfaden · Konsum pro Jahr in Preisen ${ymShow(i2ym(startM()))}, inkl. Wohnkosten, vor Steuern</span></h3>
      ${stale ? `<p class="hint" style="color:var(--warn);margin:0 0 8px">Die Planung wurde seit dieser Optimierung geändert – bitte neu starten.</p>` : ""}
      <div class="tscroll"><table class="tbl"><tr><th>Strategie</th><th class="r">Ruin</th><th class="r">Median</th><th class="r">P10</th><th class="r">Schwankung</th><th class="r">Ziel</th><th class="r" title="Entnahme für Lebenshaltung und Wohnen minus Einnahmen (vor Steuern) im ersten Jahr der Pensionierung, in Prozent des liquiden Vermögens (Median über die Pfade)">Entnahmerate</th><th>Parameter</th></tr>
      ${optMetricsRow(`Ausgangslage (${r.reference.strategy})`, r.reference.metrics, goal(r.reference.metrics), best, "wie in der Planung", r.reference.metrics.ruin > q.maxRuin ? "" : "")}
      ${r.outcomes.map((x, i) => optMetricsRow(x.strategy + (i === 0 && x.feasible ? " ★" : ""), x.validated, goal(x.validated), best,
        `<small>${x.params.map(optParam).join(", ")}</small>${x.fixed?.length ? `<br><small class="computed">fix: ${x.fixed.map(optParam).join(", ")}</small>` : ""}<div style="margin-top:4px">${x.feasible ? "" : `<span style="color:var(--err)">Ruingrenze nicht eingehalten</span> `}<button class="btn small" data-action="opt-apply" data-i="${i}">In Planung übernehmen</button></div>`, x.feasible ? "" : "off")).join("")}
      </table></div>
      <p class="hint">Ruin = Anteil der Pfade, in denen das Vermögen${q.ruinAge ? ` bis Alter ${q.ruinAge}` : " bis zum Simulationsende"} aufgebraucht ist (Grenze ${pctTxt(q.maxRuin)}). Median und P10: mittlerer bzw. schlechter Jahreskonsum über alle Pfade und Jahre ab der Pensionierung. Schwankung: mittlere Änderung des Jahreskonsums gegenüber dem Vorjahr. Ziel = (${Math.round(w * 100)} % Median + ${Math.round((1 - w) * 100)} % P10) × (1 − ${q.volWeight} × Schwankung); Strategien, die die Ruingrenze nicht einhalten, sind ausgegraut. Fixe Parameter (unter «Parameter der Strategien») bleiben wie in der Planung, auch Untergrenze und Obergrenze der Lebenshaltung, wenn Sie sie nicht variieren lassen.</p>
      <p class="hint">Gesucht wurde auf ${fmtInt(r.fitPaths)} Pfaden (Zufallssuche und lokale Verfeinerung, mit Sicherheitsabstand zur Ruingrenze), geprüft auf anderen Pfaden. Rechenzeit ${(r.elapsedMs / 1000).toFixed(1)} s. Ein Optimum gilt nur für diese Eingaben und Renditen – am besten mit einer zweiten Renditequelle (Bootstrap und parametrisch) gegenprüfen. Dies ist eine Modellrechnung, keine Anlageberatung.</p></div>`;
  }
  return `<h2>Optimierung</h2><p class="lead">Beste Entnahmestrategie für dieses Basisszenario: Der Ruin soll unter der Grenze bleiben, der Konsum möglichst hoch und gleichmässig sein.</p>
  ${single ? `<div class="card errcard">Die Renditequelle «${state.returns.source === "fixed" ? "fixe Renditen" : "historischer Verlauf"}» ergibt nur einen Pfad – Ruin ist dann 0 % oder 100 %. Bitte unter «Simulation» Monte Carlo oder rollierende Verläufe wählen.</div>` : ""}
  <div class="card"><h3>Ziel</h3>
    <label class="field" style="max-width:520px"><span>Optimierungsziel</span><select class="plain" id="opt-goal">
      <option value="consumption" ${optCfg.goal === "consumption" ? "selected" : ""}>1 · Konsum bei begrenztem Ruinrisiko</option>
      <option value="keepProperty" ${optCfg.goal === "keepProperty" ? "selected" : ""} ${hasProperties() ? "" : "disabled"}>2 · Kein Notverkauf von Immobilien</option></select>
      ${hasProperties() ? "" : `<span class="computed">Ziel 2 braucht mindestens eine Immobilie in der Planung.</span>`}</label>
    <div class="grid3" style="margin-top:10px">
    ${optInput("maxRuin", "Höchste Ruinwahrscheinlichkeit", "%")}
    ${optInput("ruinAge", "Ruin zählt bis Alter", "J.", "leer = bis Simulationsende")}
    ${optCfg.goal === "keepProperty" ? optInput("minConsumption", "Mindestkonsum (Median)", "%", "Anteil der geplanten Ausgaben ab Pensionierung")
      : `${optInput("weightMedian", "Gewicht des Medians im Konsumziel", "%", "Rest: 10-%-Perzentil")}
    ${optInput("volWeight", "Abzug für Schwankung", "×", "0 = ignorieren")}
    ${optInput("maxVol", "Höchste Schwankung", "%", "leer = keine Grenze")}`}</div>
    ${optCfg.goal === "keepProperty" ? `<p class="hint">Gesucht wird die Strategie mit dem geringsten Anteil an Läufen, in denen eine Immobilie ungeplant (Notverkauf) verkauft wird. Bei gleichem Anteil entscheidet die geringere Ruinwahrscheinlichkeit.</p>` : ""}
    <h3 style="margin-top:14px">Strategien</h3>
    <div style="display:flex;flex-wrap:wrap;gap:6px 20px">${OPT_STRATS.map(([k, t]) => `<label class="check"><input type="checkbox" id="optS-${k}" ${optCfg.strategies.includes(k) ? "checked" : ""}> ${t}</label>`).join("")}</div>
    <h3 style="margin-top:14px">Parameter der Strategien</h3>
    <p class="hint" style="margin:0 0 6px">Je Strategie legen Sie fest, welche Parameter fix auf dem Wert der Planung bleiben und welche in welcher Bandbreite variiert werden. Die Bandbreite gilt in der Einheit des Parameters: Jahre in Jahren, Prozent in Prozent.</p>
    ${optRangesHtml()}
    <details style="margin-top:12px"><summary class="hint">Erweitert</summary><div class="grid3" style="margin-top:8px">
      ${optInput("paths", "Pfade für die Suche", "", "mehr = genauer, langsamer")}${optInput("checkPaths", "Pfade für die Prüfung", "")}${optInput("samples", "Zufällige Startpunkte pro Strategie", "")}</div></details>
    <div style="margin-top:14px"><button class="btn primary" data-action="opt-run" ${optBusy || single ? "disabled" : ""}>${optBusy ? "Läuft…" : "Optimierung starten"}</button></div>
    <p class="hint est" id="est-opt" style="margin:8px 0 0"></p></div>
  ${res}${o && !optBusy && !optError ? `<footer class="rfoot" id="optFoot">${esc(DISCLAIMER_FOOT)}</footer>` : ""}`;
};
function readOptCfg() {
  const v = id => document.getElementById("opt-" + id)?.value ?? optCfg[id];
  const goalEl = document.getElementById("opt-goal");
  optCfg = { goal: goalEl ? goalEl.value : optCfg.goal, minConsumption: optNum(v("minConsumption"), 90), maxRuin: optNum(v("maxRuin"), 5), ruinAge: String(v("ruinAge")).trim(), weightMedian: optNum(v("weightMedian"), 50), volWeight: optNum(v("volWeight"), 1),
    maxVol: String(v("maxVol")).trim(), paths: Math.round(optNum(v("paths"), 5000)), checkPaths: Math.round(optNum(v("checkPaths"), 10000)), samples: Math.round(optNum(v("samples"), 30)),
    strategies: OPT_STRATS.map(x => x[0]).filter(k => document.getElementById("optS-" + k)?.checked), ranges: optCfg.ranges ?? {} };
  return optCfg;
}
OPT_ACTIONS["opt-run"] = async () => {
  if (await blockedByErrors()) return;
  const c = readOptCfg();
  if (!c.strategies.length) return toast("Bitte mindestens eine Strategie wählen.");
  const req = { goal: c.goal, minConsumption: c.minConsumption / 100, strategies: c.strategies, maxRuin: c.maxRuin / 100, ruinAge: c.ruinAge === "" ? null : optNum(c.ruinAge, null), weightMedian: clamp(c.weightMedian / 100, 0, 1),
    volWeight: c.volWeight, maxVol: c.maxVol === "" ? null : optNum(c.maxVol, 0) / 100, paths: c.paths, checkPaths: c.checkPaths, samples: c.samples, seed: 1, ranges: c.ranges };
  saveOptCfg();
  optBusy = true; optError = null; renderSection();
  const planJson = optKey();
  try { const report = await withJob(() => Engine.call("optimize", { plan: state, request: req, path: currentPath, jobId: job.id })); lastOpt = { report, req, planJson }; }
  catch (e) { if (isCancelled(e)) toast("Optimierung abgebrochen."); else optError = errText(e); }
  optBusy = false;
  if (current === "optimize") renderSection();
  document.documentElement.dataset.optimized = optError ? "error" : "ok";
};
OPT_ACTIONS["opt-apply"] = d => {
  const x = lastOpt?.report.outcomes[+d.i]; if (!x) return;
  const pt = x.patch;
  state.withdrawal = { ...state.withdrawal, ...pt.withdrawal };
  state.wealth.stockShare = pt.wealth.stockShare;
  state.wealth.reserveMonths = pt.wealth.reserveMonths;
  markDirty(); lastOpt.planJson = optKey(); renderSection();
  toast(`Strategie «${x.strategy}» mit den optimierten Parametern in die Planung übernommen.`);
};

// ----- Grundstückgewinnsteuer (Regeln je Kanton, editierbar) -----
/** Vorlage Kanton Zürich: Tarif § 225 StG, Zu- und Abschläge nach Besitzdauer, Freigrenze (StA eForm. 225.1, Stand 09/2025). */
const GAIN_ZH = () => ({
  brackets: [[4000, 0.10], [6000, 0.15], [8000, 0.20], [12000, 0.25], [20000, 0.30], [50000, 0.35], [100000, 0.40]],
  duration: [[0, 0.5], [1, 0.25], [2, 0], ...Array.from({ length: 16 }, (_, i) => [5 + i, -Math.round((0.05 + 0.03 * i) * 100) / 100])],
  exemptBelow: 5000, reduction: 0
});
/** Vorlage Kanton Aargau: Steuersatz auf den Gewinn nach Besitzdauer, § 109 StG (40 % im 1. Besitzjahr, 5 % ab 25 vollen Jahren). */
const GAIN_AG = () => ({
  model: "aargau",
  durationRates: Array.from({ length: 26 }, (_, y) => [y, y <= 10 ? Math.round((40 - 2 * y)) / 100 : y <= 24 ? (30 - y) / 100 : 0.05]),
  brackets: [], duration: [], exemptBelow: 0, reduction: 0, useMultiplier: false
});
function gainCard() {
  const g = state.taxes.propertyGain, P = "taxes.propertyGain";
  const head = `<h3>Grundstückgewinnsteuer</h3>`;
  const btns = (del) => `<button class="btn small" data-action="gain-zh">Vorlage Kanton Zürich übernehmen</button> <button class="btn small" data-action="gain-ag">Vorlage Kanton Aargau übernehmen</button>${del ? ` <button class="btn small" data-action="gain-del">Regeln entfernen</button>` : ""}`;
  const needed = state.realestate.properties.some(p => p.status !== "rent");
  if (!g) return `<div class="card${needed ? " errcard" : ""}">${head}<p class="hint" style="margin:0 0 8px">${needed
    ? `<b>Pflicht:</b> Die Planung enthält eine Immobilie im Besitz oder mit geplantem Kauf. Bitte Vorlage Zürich, Vorlage Aargau oder eigene Regeln erfassen (auch mit Satz 0 %), sonst lässt sich die Simulation nicht starten.`
    : `Ohne Immobilie im Besitz oder mit geplantem Kauf sind keine Regeln nötig. Die Regeln sind je Kanton verschieden.`}</p>
    ${btns(false)} <button class="btn small" data-action="gain-own">Eigene Regeln erfassen</button></div>`;
  const rows = (key, cols) => (g[key] ?? []).map((r, i) => `<tr><td>${F(`${P}.${key}.${i}.0`, "", cols[0], { bare: true, unit: cols[2] })}</td><td>${F(`${P}.${key}.${i}.1`, "", "pct", { bare: true })}</td>
    <td><button class="btn ghost small" data-action="list-del" data-path="${P}.${key}" data-i="${i}">✕</button></td></tr>`).join("");
  const aargau = g.model === "aargau";
  const model = `<div class="seg" style="margin-bottom:10px"><button class="${aargau ? "" : "on"}" data-action="gain-model" data-v="zurich">Modell Zürich</button><button class="${aargau ? "on" : ""}" data-action="gain-model" data-v="aargau">Modell Aargau (eigene Eingabe)</button></div>`;
  const tables = aargau
    ? `<div><table class="tbl"><tr><th>Besitzdauer ab vollen Jahren</th><th>Steuersatz auf den Gewinn</th><th></th></tr>${rows("durationRates", ["num", "", "J."])}</table>
      <div style="margin-top:8px"><button class="btn small" data-action="gain-add" data-key="durationRates">+ Zeile</button></div>
      <p class="hint">Der Steuersatz hängt von der Besitzdauer ab und gilt für den ganzen Gewinn (Modell Aargau, § 109 StG: 40 % im ersten Besitzjahr, danach sinkend bis 5 %). Es gilt die letzte Zeile, deren Jahre erreicht sind. Die Besitzdauer folgt aus dem Kaufdatum der Immobilie; ohne Kaufdatum gilt der höchste Satz.</p></div>`
    : `<div class="grid2"><div><table class="tbl"><tr><th>Stufe: Gewinnteil bis</th><th>Steuersatz</th><th></th></tr>${rows("brackets", ["chf", "", cur()])}</table>
      <div style="margin-top:8px"><button class="btn small" data-action="gain-add" data-key="brackets">+ Stufe</button></div>
      <p class="hint">Der Gewinn wird stufenweise besteuert: Die erste Stufe gilt für die ersten Franken, die nächste für die weiteren usw. Über der letzten Stufe gilt deren Satz.</p></div>
    <div><table class="tbl"><tr><th>Besitzdauer ab vollen Jahren</th><th>Zuschlag / Abzug</th><th></th></tr>${rows("duration", ["num", "", "J."])}</table>
      <div style="margin-top:8px"><button class="btn small" data-action="gain-add" data-key="duration">+ Zeile</button></div>
      <p class="hint">Veränderung der Steuer nach Besitzdauer: positiv = Zuschlag (Kurzbesitz), negativ = Abzug (lange Besitzdauer). Es gilt die letzte Zeile, deren Jahre erreicht sind. Die Besitzdauer folgt aus dem Kaufdatum der Immobilie.</p></div></div>`;
  return `<div class="card">${head}${model}${tables}
    <div class="grid3" style="margin-top:10px">${F(P + ".exemptBelow", "Gewinne unter … sind steuerfrei", "chf")}${F(P + ".reduction", "Feste Ermässigung", "pct")}</div>
    <div style="margin-top:10px">${Chk(P + ".useMultiplier", "Mit dem Steuerfuss (Kanton + Gemeinde) multiplizieren")}</div>
    <div style="margin-top:10px">${btns(true)}</div></div>`;
}
Object.assign(OPT_ACTIONS, {
  "gain-zh": () => { state.taxes.propertyGain = { ...GAIN_ZH(), model: "zurich", useMultiplier: !!state.taxes.propertyGain?.useMultiplier }; markDirty(); renderSection(); },
  "gain-ag": () => { state.taxes.propertyGain = { ...GAIN_AG(), useMultiplier: !!state.taxes.propertyGain?.useMultiplier }; markDirty(); renderSection(); },
  "gain-own": () => { state.taxes.propertyGain = { ...GAIN_AG(), durationRates: [[0, 0.2]] }; markDirty(); renderSection(); },
  "gain-model": d => {
    const g = state.taxes.propertyGain; g.model = d.v;
    if (d.v === "aargau" && !g.durationRates?.length) g.durationRates = GAIN_AG().durationRates;
    if (d.v === "zurich" && !g.brackets?.length) g.brackets = GAIN_ZH().brackets;
    markDirty(); renderSection();
  },
  "gain-del": () => { delete state.taxes.propertyGain; markDirty(); renderSection(); },
  "gain-add": d => {
    const t = (state.taxes.propertyGain[d.key] ??= []), last = t[t.length - 1];
    t.push(d.key === "brackets" ? [last ? last[0] : 10000, last ? last[1] : 0.1] : [last ? last[0] + 1 : 0, last ? last[1] : 0]);
    markDirty(); renderSection();
  }
});
R.taxes = () => `<h2>Steuern</h2><p class="lead">Einfache Satztabellen; zwischen den Stufen wird linear interpoliert und mit dem Steuerfuss multipliziert.</p>
  <div class="card"><div class="grid3">${F("taxes.multiplier", "Steuerfuss (Kanton + Gemeinde)", "num", { unit: "×" })}</div></div>
  <div class="card">${TaxTable("income", "Einkommenssteuer", "Einkommen")}${TaxTable("wealth", "Vermögenssteuer", "Vermögen")}${TaxTable("capital", "Kapitalbezugssteuer", "Kapitalbezug")}</div>
  ${gainCard()}
  ${state.partner && state.household.married ? `<div class="card"><h3>Verheiratete</h3>
    <label class="check"><input type="checkbox" data-action="couple-tax" ${state.taxes.couple ? "checked" : ""}> Eigene Tabellen für Verheiratete verwenden</label>
    <p class="hint">Ohne eigene Tabellen gelten die obigen für den ganzen Haushalt (gemeinsames Einkommen und Vermögen). Die Tabellen für Verheiratete ersetzen sie bei zwei Personen.</p>
    ${state.taxes.couple ? TaxTable("couple.income", "Einkommenssteuer (Verheiratete)", "Einkommen") + TaxTable("couple.wealth", "Vermögenssteuer (Verheiratete)", "Vermögen") + TaxTable("couple.capital", "Kapitalbezugssteuer (Verheiratete)", "Kapitalbezug") : ""}</div>` : ""}`;

const PSTAT = [["owned", "Besitz"], ["planned", "Kauf geplant"], ["rent", "Miete geplant"]];
/** FEA-36: Periode, die erst ab dem AHV-Bezug beginnt (dann nur direkte Amortisation). */
function periodAfterAhv(p, t, k) {
  const r = trancheSchedule(p, t).find(x => x.i === k);
  return !!r && r.start >= msMonth(ahvIdOf(t.pillar3a));
}
function MortgageUI(p, i) {
  const M = `realestate.properties.${i}.mortgage`, mg = p.mortgage, multi = mg.tranches.length > 1, owned = p.status === "owned";
  const anyIndirect = mg.tranches.some(t => t.periods.some(d => d.amortMode === "indirect" && d.amort > 0));
  const maxLoan = !owned && mg.amountMode === "maxLoan";
  const mode = owned ? "" : `<div class="grid2" style="margin-bottom:10px"><label class="field"><span>Hypothek beim Kauf</span>${Sel(M + ".amountMode", [["fixed", "Betrag wie erfasst"], ["maxLoan", "80 % des Kaufpreises (Minimum Eigenmittel), gekürzt falls nicht tragbar"]], { rerender: 1 })}</label></div>
    ${maxLoan && multi ? `<p class="hint" style="margin-top:-4px">Die Hypothek wird im Verhältnis der Anteile auf die Tranchen verteilt.</p>` : ""}`;
  const tranches = mg.tranches.map((t, ti) => {
    const T = `${M}.tranches.${ti}`;
    const rows = t.periods.map((d, k) => {
      const D = `${T}.periods.${k}`;
      return `<tr><td style="white-space:nowrap">${k ? `Anschl. ${k}` : owned ? "Laufend" : "Erste"}</td>
        <td>${Sel(D + ".type", [["fixed", "Fest"], ["saron", "SARON"]], { rerender: 1 })}</td>
        <td>${F(D + ".term", "", "years", { bare: true, unit: "J." })}</td>
        <td>${d.type === "saron" ? F(D + ".margin", "", "pct", { bare: true, unit: "% Marge" }) : F(D + ".rate", "", "pct", { bare: true })}</td>
        <td><div class="refwrap">${F(D + ".amort", "", "chf", { bare: true })}${periodAfterAhv(p, t, k) ? `<span class="computed" style="padding:8px 4px" title="Ab dem AHV-Bezug ist nur direkte Amortisation möglich.">direkt (ab AHV)</span>` : Sel(D + ".amortMode", [["direct", "direkt"], ["indirect", "indirekt"]], { rerender: 1 })}</div></td>
        <td style="white-space:nowrap">${C("mrow", `${i}:${ti}:${k}`)}</td>
        <td>${k ? `<button class="btn ghost small" data-action="list-del" data-path="${T}.periods" data-i="${k}" title="Periode entfernen">✕</button>` : ""}</td></tr>`;
    }).join("");
    return `<div class="tranche">
      ${multi ? `<div class="thead">${F(T + ".name", "", "text", { bare: true })}<button class="btn ghost small" data-action="list-del" data-path="${M}.tranches" data-i="${ti}" title="Tranche entfernen">✕</button></div>` : ""}
      <div class="grid3">${maxLoan ? (multi ? F(T + ".amount", "Anteil (Verhältnis)", "num") : `<label class="field"><span>Betrag bei Kauf</span><span class="computed" style="padding:8px 0">wird beim Kauf berechnet</span></label>`) : F(T + ".amount", owned ? "Betrag heute" : "Betrag bei Kauf", "chf")}
        ${owned ? F(T + ".start", "Beginn der laufenden Periode", "month") : `<label class="field"><span>Beginn</span><span class="computed" style="padding:8px 0">beim Kauf (${esc(msName(p.buy))})</span></label>`}
        <label class="field"><span>Nach der letzten Periode</span>${Sel(T + ".after", [["renew", "Wie letzte Periode verlängern"], ["repay", "Restschuld zurückzahlen"]])}</label>
        ${state.household.persons.length > 1 && t.periods.some(d => d.amortMode === "indirect") ? `<label class="field"><span>Säule 3a für die indirekte Amortisation</span>${Sel(T + ".pillar3a", state.household.persons.map(q => [q.id, q.name || q.id]))}</label>` : ""}</div>
      <div style="overflow-x:auto"><table class="tbl mtbl"><colgroup><col style="width:68px"><col style="width:90px"><col style="width:72px"><col style="width:112px"><col style="width:206px"><col><col style="width:28px"></colgroup><tr><th>Periode</th><th>Art</th><th>Laufzeit</th><th>Zins p.a.</th><th>Amortisation pro Quartal</th><th>Zeitraum / Restschuld</th><th></th></tr>${rows}</table></div>
      <div style="margin-top:8px"><button class="btn small" data-action="period-add" data-i="${i}" data-t="${ti}">+ Anschlusshypothek</button></div></div>`;
  }).join("");
  const p3a = anyIndirect ? `<p class="hint">Indirekte Amortisation fliesst in die <a href="#" data-action="goto" data-sec="pillar3a">Säule 3a</a> und ist nur bis zum AHV-Bezug möglich; ab dem AHV-Bezug wird direkt amortisiert. Perioden, die ab dem AHV-Bezug beginnen, amortisieren immer direkt.</p>` : "";
  return `<h4>Hypothek</h4>${mode}${tranches}
    <div style="margin:4px 0 0"><button class="btn small" data-action="tranche-add" data-i="${i}">+ Tranche (Aufteilung)</button></div>
    <p class="hint">${C("mort", i)}</p>${p3a}`;
}
R.realestate = () => `<h2>Immobilien</h2><p class="lead">Eigene, geplante und gemietete Objekte. Kauf- und Verkaufszeitpunkte können Meilensteine sein.</p>
  <div class="card"><h3>Notverkauf</h3>${Series("realestate.emergencyMonths", "Schwelle", "months", "Notverkauf, wenn das liquide Vermögen weniger Monate ungedeckter Ausgaben deckt")}
    <p class="hint">Nach dem Verkauf der letzten Immobilie wird ein Objekt mit Kauf «nach Verkauf» gekauft, sofern tragbar; sonst beginnen die Mieten «nach Verkauf».</p></div>
  <div class="card"><h3>Tragbarkeit <span class="sub">für Käufe «nach Verkauf»</span></h3><div class="grid3">
    ${F("realestate.affordability.sustainability", "Höchstens vom Einkommen", "pct")}${F("realestate.affordability.imputedRate", "Kalkulatorischer Zins", "pct")}${F("realestate.affordability.fixCosts", "Nebenkosten p.a. (vom Preis)", "pct")}
    ${F("realestate.affordability.maxLoanToValue", "Höchste Belehnung", "pct")}</div>
    <p class="hint">Angerechnet wird nur das Einkommen (Erwerb und Renten), nicht die Entnahme aus dem Vermögen. Reicht es nicht, wird die Hypothek gekürzt; die Simulation zeigt die Einmalrückzahlung beim Kauf als Ereignis.</p></div>
  ${state.realestate.properties.map((p, i) => {
    const P = `realestate.properties.${i}`;
    let body = "";
    if (p.status === "rent") body = `<div class="grid2"><label class="field"><span>Miete ab</span>${Ref(P + ".from", { afterSale: true, self: p.id })}</label>${F(P + ".rent", "Miete pro Monat (heutiger Wert)", "chf")}</div>
      <div style="margin-top:10px">${Chk(P + ".inflationRent", "Miete mit Inflation anpassen")}</div>`;
    else body = `<div class="grid2 pgrid">
      ${p.status === "owned" ? F(P + ".bought", "Kaufdatum", "month") : `<label class="field"><span>Kauf ab</span>${Ref(P + ".buy", { afterSale: true, self: p.id })}</label>`}
      ${F(P + ".price", p.status === "owned" ? "Kaufpreis (bezahlt)" : "Kaufpreis", "chf")}
      <label class="field"><span>Verkauf ab</span>${Ref(P + ".sell", { never: true })}</label>
      ${F(P + ".fixCosts", "Nebenkosten / Monat", "chf")}</div>
      <div class="grid3" style="margin-top:10px">${Chk(P + ".inflationValue", "Wert mit Inflation anpassen")}${Chk(P + ".inflationFixCosts", "Nebenkosten mit Inflation anpassen")}</div>
      ${MortgageUI(p, i)}`;
    return `<div class="prop"><div class="phead">${F(P + ".name", "", "text", { bare: true })}
      ${p.status === "owned" ? `<div class="pworth">${F(P + ".worth", `Marktwert per Simulationsstart (${startTxt()})`, "chf")}</div>` : ""}
      <div class="seg">${PSTAT.map(([k, l]) => `<button class="${p.status === k ? "on" : ""}" data-action="pstat" data-i="${i}" data-v="${k}">${l}</button>`).join("")}</div>
      <button class="btn ghost small" data-action="list-del" data-path="realestate.properties" data-i="${i}" title="Objekt entfernen">✕</button></div>${body}</div>`;
  }).join("")}
  <button class="btn" data-action="prop-add">+ Objekt</button>`;

R.scenarios = () => `<h2>Szenarien & Simulation</h2><p class="lead">Parameter, die systematisch variiert werden.</p>
  <div class="card"><p style="margin:0">«Simulation starten» rechnet die Planung wie erfasst und zusätzlich jede Kombination der Variationen mit allen Renditepfaden. Die Ergebnisse erscheinen als Tabelle in der Ergebnisansicht.</p></div>
  <div class="card"><h3>Variationen</h3><table class="tbl"><tr><th>Parameter</th><th>Von</th><th>Bis</th><th>Schritt</th><th>Läufe</th><th></th></tr>
  ${state.scenarios.sweeps.map((s, i) => {
    const t = SWEEP_PARAMS.find(p => p.id === s.param)?.type || "pct", P = `scenarios.sweeps.${i}`;
    return `<tr><td>${Sel(P + ".param", SWEEP_PARAMS.map(p => [p.id, p.label]), { rerender: 1 })}</td><td>${F(P + ".from", "", t, { bare: true })}</td><td>${F(P + ".to", "", t, { bare: true })}</td>
      <td>${F(P + ".step", "", t === "month" ? "months" : t, { bare: true, unit: t === "month" ? "Mt." : undefined })}</td><td>${C("sweep", i)}</td>
      <td><button class="btn ghost small" data-action="list-del" data-path="scenarios.sweeps" data-i="${i}">✕</button></td></tr>`;
  }).join("")}</table>
  <div style="margin-top:10px"><button class="btn small" data-action="sweep-add">+ Variation</button></div></div>
  <div class="card"><h3>Umfang</h3><p style="margin:0">${C("runs")} <span id="est-warn" class="timewarn"></span></p>
    <p class="hint est" id="est-scen" style="margin:6px 0 0"></p>
    <p class="hint">Der Rechenkern verteilt die Läufe auf alle Prozessorkerne. Die Ergebnisse (Perzentile, Erfolgswahrscheinlichkeit) erscheinen in einer eigenen Ergebnisansicht.</p></div>`;

// ---------- Berechnete Anzeigen ----------
function sweepCount(s) {
  return sweepSteps(s.from, s.to, s.step, SWEEP_PARAMS.find(p => p.id === s.param)?.type === "month");
}
function rollingCount() {
  const i = seriesInfo; if (!i || i.error) return 0;
  const n = endM() - startM() + 1;
  if (i.monthly) return Math.max(0, i.entries - n + 1);
  return Math.max(0, i.entries - (Math.floor(endM() / 12) - Math.floor(startM() / 12) + 1) + 1);
}
function runsPerScenario() {
  const r = state.returns;
  return { fixed: 1, historical: 1, rolling: rollingCount(), bootstrap: r.bootstrap.paths, parametric: r.parametric.paths }[r.source];
}
const COMPUTED = {
  // FEA-41: Auszahlungsplan der Säule 3a (Zeile «k.j»): Monat und Alter, Guthaben vor und nach der Auszahlung aus der Vorschau.
  p3aat: a => {
    const [k, j] = a.split(".").map(Number), acc = state.pillar3a[k], x = acc?.payouts[j];
    const m = x ? payoutMonth(x, acc.person) : NaN;
    return Number.isFinite(m) ? `<span data-p3a="at"><b>${ymShow(i2ym(m))}</b> · Alter ${ageAt(m, acc.person).toLocaleString("de-CH", { minimumFractionDigits: 1, maximumFractionDigits: 1 })}</span>` : `<span data-p3a="at">ungültig</span>`;
  },
  p3apay: a => {
    const [k, j] = a.split(".").map(Number), pv = p3aViewOf(state.pillar3a[k]?.person)?.payouts.find(o => o.index === j);
    return `<span data-p3a="before">${pv ? fmtInt(pv.before) : "–"}</span> → <span data-p3a="after">${pv ? fmtInt(pv.after) : "–"}</span>`;
  },
  p3arest: k => {
    const acc = state.pillar3a[+k], pi = state.household.persons.findIndex(q => q.id === acc?.person), ahv = msMonth(pi === 1 ? "ahv2" : "ahv");
    const view = p3aViewOf(acc?.person);
    return `Guthaben, das im AHV-Bezug (${Number.isFinite(ahv) ? ymShow(i2ym(ahv)) : "?"}) ausbezahlt wird: <b data-p3a="rest">${view ? fmtInt(view.rest?.amount ?? 0) + " " + cur() : "–"}</b>`;
  },
  housingshare: () => {
    if (!state.flows.spendingIncludesHousing) return "";
    const w = lastSim?.housingShare;
    return `Enthält die Wohnkosten (Miete bzw. Zins, Amortisation, Nebenkosten). Wohnanteil zum Simulationsstart: <b>${w != null ? fmtMoney(Math.round(w)) : "–"}</b> pro Monat; er wird von jeder Zeile abgezogen, gerechnet wird mit den simulierten Wohnkosten.`;
  },
  ms: id => { const m = msMonth(id); return Number.isFinite(m) ? `<b>${ymShow(i2ym(m))}</b> · Alter ${ageStr(m)}` : "ungültig"; },
  age1: () => { const s = startM(); return Number.isFinite(s) && state.household.persons[1] ? `Alter bei Simulationsstart: <b>${ageAt(s, state.household.persons[1].id).toLocaleString("de-CH", { minimumFractionDigits: 1, maximumFractionDigits: 1 })}</b>` : ""; },
  wdinit: () => { const e = lastSim?.events.find(x => x.kind === "withdrawal"); return e ? `Vorschau: ${esc(e.label)} (${ymShow(i2ym(e.m))}).` : "Vorschau: keine Entnahme im Simulationszeitraum (Pensionierung ausserhalb?)."; },
  age0: () => { const s = startM(); return Number.isFinite(s) ? `Alter bei Simulationsstart: <b>${ageStr(s)}</b>` : ""; },
  pkresult: () => lastSim ? `${state.partner ? "Haushalt, bei Pensionierung" : "Bei Pensionierung"}: Kapitalbezug netto <b>${fmtMoney(lastSim.lumpNet)}</b> (Steuer ${fmtMoney(lastSim.lumpTax)}), Rente <b>${fmtMoney(lastSim.pkPension)}</b> / Monat.` : "",
  // FEA-57: Anteil von Aktien bzw. Obligationen am Portfolio (ohne Cash), ganzzahlig; ohne Portfolio keine Angabe.
  wquote: arg => {
    const w = state.wealth, s = Math.max(0, +w.stocks || 0), b = Math.max(0, +w.bonds || 0);
    return s + b > 0 ? `${Math.round(100 * (arg === "stocks" ? s : b) / (s + b))} % des Portfolios` : "";
  },
  wealthsum: () => { const w = state.wealth; return `Total <b>${fmtMoney(w.stocks + w.bonds + w.cash)}</b>`; },
  rolling: () => {
    if (!seriesInfo) return "Die Zeitreihe der Anlagestrategie wird geladen.";
    if (seriesInfo.error) return `<span style="color:var(--err)">${esc(seriesInfo.error)}</span>`;
    return `<b>${fmtInt(rollingCount())}</b> Startzeitpunkte decken den Zeitraum von ${((endM() - startM()) / 12).toFixed(1)} Jahren vollständig ab.`;
  },
  // FEA-53: Summe der Gewichte von «Welt individuell»
  wsum: () => {
    const w = state.returns.strategy?.weights || {}, sum = REGION_KEYS.reduce((a, k) => a + (+w[k] || 0), 0);
    return `Summe <b${Math.abs(sum - 1) > 1e-6 ? ' style="color:var(--err)"' : ""}>${fmtPct(sum)} %</b> (muss 100 % ergeben).`;
  },
  // FEA-53: Kurzinfo der Zeitreihe zur Strategie
  strategyinfo: () => {
    const i = seriesInfo;
    if (!i) return "";
    if (i.error) return i.error.includes("Gewichte") ? "" : `<span style="color:var(--err)">${esc(i.error)}</span>`; // Gewichte: Meldung am Feld
    const p = v => v == null || Number.isNaN(v) ? "–" : `${(v * 100).toLocaleString("de-CH", { minimumFractionDigits: 1, maximumFractionDigits: 1 })} %`;
    // FEA-53/AK8: Die Mittelwerte gelten für den gemeinsamen Zeitraum; ein längerer Index wird darauf gekürzt.
    const from = +String(i.first).slice(0, 4), to = +String(i.last).slice(0, 4), span = `${from}–${to}`;
    const cut = (i.parts || []).filter(x => x.first < from || x.last > to);
    const clip = cut.length ? `<span class="cliphint" id="strategyClip">⚠ ${cut.map(x => `${esc(x.label)} reichen von ${x.first} bis ${x.last}`).join(", ")} und werden auf den gemeinsamen Zeitraum ${span} gekürzt. Die Simulation (Monte Carlo und historische Verläufe) nutzt nur diesen Zeitraum.</span>` : "";
    return `${i.monthly ? "Monatlich" : "Jährlich"} ${ymShow(i.first)} – ${ymShow(i.last)}. Mittlere Jahresrendite${i.parts?.length ? ` im gemeinsamen Zeitraum ${span} von Aktien und Obligationen` : ""}: Aktien <b>${p(i.avgStocks)}</b>, Obligationen <b>${p(i.avgBonds)}</b>, Teuerung <b>${p(i.avgInflation)}</b>. ${seriesTip(i)}${clip}`;
  },
  seriesinfo: () => {
    if (!seriesInfo) return "";
    if (seriesInfo.error) return `<span style="color:var(--err)">${esc(seriesInfo.error)}</span>`;
    return "";
  },
  basis: () => esc(previewBasis || "die fixen Werte."),
  // FEA-78: Vorschlagswert neben einem Feld des Aktienmodells (nur bei eigenen Werten)
  psug: k => {
    const q = state.returns.parametric, s = paramSuggestion();
    return !s || q.auto ? "" : `Vorschlag: ${fmtParam(k, s[k])}`;
  },
  pauto: () => {
    const q = state.returns.parametric, s = paramSuggestion();
    if (!seriesInfo) return "Die Vorschlagswerte werden geladen.";
    if (!s) return "Für diese Zeitreihe gibt es keine Vorschlagswerte; es gelten die eingegebenen Werte.";
    const st = state.returns.strategy, y = seriesInfo.parametric.stockYears;
    const name = state.returns.series ? seriesShort(state.returns.series) : STOCK_INDICES.find(x => x[0] === st?.stocks)?.[1] ?? "–";
    return q.auto ? `Die Werte folgen dem Aktienindex <b>${esc(name)}</b>: Vorschlagswerte aus den Jahren ${y[0]}–${y[1]}, so gewählt, dass die Jahresrenditen dem historischen Bootstrap entsprechen. Ändern Sie einen Wert, gelten Ihre eigenen Werte.`
      : `Eigene Werte. Neben den Feldern stehen die Vorschlagswerte für <b>${esc(name)}</b>. <button class="btn small" data-action="param-suggest">Vorschlagswerte übernehmen</button>`;
  },
  // FEA-79: Mittel, Streuung und Korrelation von Aktien, Obligationen und Inflation der Reihe
  pjoint: () => {
    const j = paramJoint();
    if (!seriesInfo) return "";
    if (!j) return "Ohne Zeitreihe gelten für Obligationen und Inflation die fixen Werte.";
    const p = v => `${(v * 100).toLocaleString("de-CH", { minimumFractionDigits: 1, maximumFractionDigits: 1 })} %`;
    const c = v => v.toLocaleString("de-CH", { minimumFractionDigits: 2, maximumFractionDigits: 2 }).replace("-", "−");
    const names = ["Aktien", "Obligationen", "Inflation"];
    return `<table class="tbl num"><thead><tr><th>Korrelation</th>${names.map(n => `<th>${n}</th>`).join("")}<th>Mittel</th><th>Streuung</th></tr></thead><tbody>
      ${names.map((n, a) => `<tr><td>${n}</td>${names.map((_, b) => `<td>${a === b ? "1" : c(j.corr[a][b])}</td>`).join("")}<td>${a === 1 ? p(j.bondsMean) : a === 2 ? p(j.inflationMean) : "Modell"}</td><td>${a === 1 ? p(j.bondsSd) : a === 2 ? p(j.inflationSd) : "Modell"}</td></tr>`).join("")}</tbody></table>
      <p class="hint">Jahreswerte ${j.first}–${j.last} der Reihe, die auch der historische Bootstrap verwendet. Jedes Jahr ziehen Obligationen und Inflation einen Wert, der mit der Aktienrendite desselben Jahres korreliert ist; die Jahre sind voneinander unabhängig. Der SARON bleibt fix.</p>`;
  },
  bucket: k => {
    const rm = refMonth(state.pension.pk.at); if (!lastSim) return "–";
    const i = lastSim.m.indexOf(rm); if (i < 0) return "–";
    return fmtMoney(lastSim.pots[i][{ cash: 0, bond: 1, stock: 2 }[k]]);
  },
  mrow: a => {
    const [i, ti, k] = a.split(":").map(Number), p = state.realestate.properties[i];
    const r = trancheSchedule(p, p.mortgage.tranches[ti]).find(x => x.i === k);
    if (!r) return `<span style="color:var(--faint)">${refMonth(p.buy) === Infinity ? "beginnt beim Kauf" : "nach Verkauf / Simulationsende"}</span>`;
    if (p.mortgage.amountMode === "maxLoan" && p.status !== "owned") return `${ymShow(i2ym(r.start))} – ${ymShow(i2ym(r.end))}`;
    const span = `${ymShow(i2ym(r.start))} – ${ymShow(i2ym(r.end))}`;
    if (r.past) return `${span} <span style="color:var(--faint)">· abgelaufen</span>`;
    return `${span}${r.d.type === "saron" ? ` · ${fmtPct(r.rate)} %` : ""}<br><b>${fmtMoney(r.a1)}</b>${!r.direct && r.q ? ` <span style="color:var(--faint)">(+ 3a${r.toAhv ? " bis AHV" : ""})</span>` : ""}`;
  },
  mort: i => {
    const p = state.realestate.properties[i]; if (!p?.mortgage) return "";
    if (p.status !== "owned" && p.mortgage.amountMode === "maxLoan") {
      const a = state.realestate.affordability;
      return `Hypothek beim Kauf höchstens <b>${fmtMoney(p.price * a.maxLoanToValue)}</b> (${fmtPct(a.maxLoanToValue)} % von ${fmtMoney(p.price)}, ohne Teuerung); gekürzt, falls die Tragbarkeit nicht reicht (Einmalrückzahlung aus dem Vermögen). Die Simulation zeigt den Betrag als Ereignis.`;
    }
    const base = p.status === "owned" ? p.worth : p.price, total = mortTotal(p), multi = p.mortgage.tranches.length > 1;
    let interest = 0, direct = 0, indirect = 0; const parts = [];
    for (const t of p.mortgage.tranches) {
      const sch = trancheSchedule(p, t), cur = sch.find(x => !x.past), last = sch[sch.length - 1];
      if (cur) {
        interest += cur.a0 * cur.rate / 12; if (cur.direct) direct += cur.q; else indirect += cur.q;
        let s = `${multi ? `${esc(t.name)}: ` : ""}${p.status === "owned" ? "laufend" : "erste Periode"} bis <b>${ymShow(i2ym(cur.end))}</b>`;
        if (p.status === "owned") s += ` (Restlaufzeit ${((cur.end - cur.from) / 12).toLocaleString("de-CH", { maximumFractionDigits: 1 })} J.)`;
        if (last && t.after === "repay" && last.end <= endM()) s += `, danach Rückzahlung von ${fmtMoney(last.a1 ?? t.amount)}`;
        else if (last?.renewal) s += ", danach Verlängerung zu gleichen Konditionen";
        parts.push(s);
      }
    }
    return `Total <b>${fmtMoney(total)}</b> · Belehnung <b>${(total / base * 100).toLocaleString("de-CH", { maximumFractionDigits: 1 })} %</b> · Zins ${p.status === "owned" ? "heute" : "beim Kauf"} <b>${fmtMoney(interest)}</b> / Monat · Amortisation pro Quartal: ${fmtMoney(direct)} direkt${indirect ? `, ${fmtMoney(indirect)} indirekt` : ""}<br>${parts.join("<br>")}`;
  },
  sweep: i => fmtInt(sweepCount(state.scenarios.sweeps[i])),
  runs: () => {
    const sw = state.scenarios.sweeps.map(sweepCount), combos = sw.reduce((a, b) => a * b, 1), per = runsPerScenario();
    const over = combos > MAX_COMBINATIONS ? ` <span style="color:var(--err)">· technische Grenze: höchstens ${fmtInt(MAX_COMBINATIONS)} Kombinationen möglich (jetzt ${fmtInt(combos)})</span>` : "";
    return `${sw.length ? sw.map(fmtInt).join(" × ") + " Kombinationen" : "1 Szenario"} × ${fmtInt(per)} ${per === 1 ? "Lauf" : "Läufe"} = <b>${fmtInt(combos * per)} Simulationsläufe</b>${over}`;
  }
};
/** Abschnitt des Editors zu einem Hauptschlüssel der Planung (für die Meldungen beim Löschen eines Meilensteins). */
const SECTION_OF = { household: "Haushalt & Projekt", pension: "Vorsorge", partner: "Vorsorge", pillar3a: "Säule 3a", flows: "Einnahmen & Ausgaben", wealth: "Vermögen & Anlage", returns: "Simulation",
  withdrawal: "Entnahmestrategie", optimization: "Optimierung", taxes: "Steuern", realestate: "Immobilien", scenarios: "Szenarien & Simulation" };
/** Verwendungen eines Meilensteins ausserhalb der Meilenstein-Tabelle: [{ section, set(wert) }]. */
function milestoneRefs(id) {
  const out = [];
  const walk = (o, top) => {
    if (!o || typeof o !== "object") return;
    for (const [k, v] of Object.entries(o)) {
      if (top === "" && k === "milestones") continue;
      if (v === id) out.push({ section: SECTION_OF[top || k] || "Planung", parent: o, key: k });
      else walk(v, top || k);
    }
  };
  walk(state, "");
  return out;
}
const countRefs = id => milestoneRefs(id).length;
/** FEA-38: feste Meilensteine einer Person («Bezug Pensionskasse», «AHV-Bezug»); `null` für alle anderen. */
const SYS_MS = { retire: { kind: "pk", index: 0 }, ahv: { kind: "ahv", index: 0 }, retire2: { kind: "pk", index: 1 }, ahv2: { kind: "ahv", index: 1 } };
const sysMs = ms => SYS_MS[ms.id] ?? null;
/** Start, Ende und die festen Meilensteine jeder Person lassen sich nicht löschen. */
const msLocked = ms => !!ms.fixed || !!sysMs(ms);

// ---------- Rendering ----------
function renderNav(vals) {
  $("#nav").innerHTML = SECTIONS.map((s, i) => {
    const e = vals.filter(v => v.sec === s.id && v.sev === "err").length, w = vals.filter(v => v.sec === s.id && v.sev === "warn").length;
    return `<a class="${s.id === current ? "active" : ""}" data-action="goto" data-sec="${s.id}"><span class="num">${i + 1}</span>${s.title}${e ? `<span class="badge err">${e}</span>` : w ? `<span class="badge warn">${w}</span>` : ""}</a>`;
  }).join("") + `<div class="navfoot">Daten bleiben lokal auf diesem Rechner. Keine Verbindung ins Internet.</div>`;
}
function renderSection() {
  const m = $("#main"), y = m.scrollTop;
  m.innerHTML = R[current]();
  m.scrollTop = y;
  refresh();
  if (current === "optimize" || current === "scenarios") { updateEstimates(); ensureBench(); }
  if (current === "optimize") ensureOptCatalog();
}
function goto(sec) { current = sec; $("#main").scrollTop = 0; renderSection(); }

let raf = 0;
function scheduleRefresh() { cancelAnimationFrame(raf); raf = requestAnimationFrame(refresh); }
function refresh() { if (state && seriesRef() !== seriesInfoRef) refreshSeriesInfo(); renderDerived(); requestPreview(); }
function renderDerived() {
  if (!state) return;
  const vals = collectIssues();
  renderNav(vals); renderVal(vals); renderTimeline(); renderChart("#chart", "#tip", lastSim); renderKpis();
  $("#previewSub").textContent = `${previewBasis ? previewBasis.replace(/^Vorschau mit /, "Einzellauf mit ").replace(/\.$/, "") : "Einzellauf mit fixen Renditen"} · bei jeder Änderung neu gerechnet`;
  document.querySelectorAll("[data-computed]").forEach(el => { el.innerHTML = COMPUTED[el.dataset.computed](el.dataset.arg); });
  document.querySelectorAll("svg[data-mini]").forEach(drawMini);
  document.querySelectorAll("svg[data-taxmini]").forEach(drawTaxMini);
  if (!job && benchKey() !== bench.key) scheduleBench(); else updateEstimates();
  applyFieldIssues();
}

function renderVal(vals) {
  const order = { err: 0, warn: 1 };
  vals.sort((a, b) => order[a.sev] - order[b.sev]);
  $("#val").innerHTML = vals.length ? vals.map(v => `<li class="${v.sev}" data-action="goto" data-sec="${v.sec}"><span class="ic">${v.sev === "err" ? "!" : "▲"}</span><span>${v.text}</span><span class="sec">${SECTIONS.find(s => s.id === v.sec).title}</span></li>`).join("")
    : `<li class="ok"><span class="ic">✓</span><span>Keine Fehler gefunden.</span></li>`;
}

function renderKpis() {
  const s = lastSim, k = $("#kpis");
  if (!s) { k.innerHTML = `<div class="kpi bad"><div class="k">Vorschau</div><div class="v">–</div><div class="s">${previewError ? "siehe Prüfung" : "wird gerechnet…"}</div></div>`; return; }
  const last = s.total[s.total.length - 1], e = endM();
  k.innerHTML = `
    <div class="kpi"><div class="k">Vermögen bei Pensionierung</div><div class="v">${s.totalAtRetire != null ? fmtMio(s.totalAtRetire) : "–"}</div><div class="s">${ymShow(i2ym(s.retireM))}</div></div>
    <div class="kpi ${last > 0 ? "good" : "bad"}"><div class="k">Vermögen am Ende</div><div class="v">${fmtMio(last)}</div><div class="s">Alter ${ageStr(e)}, nominal</div></div>
    <div class="kpi ${s.depleted !== null ? "bad" : "good"}"><div class="k">Geld reicht bis</div><div class="v">${s.depleted !== null ? "Alter " + ageStr(s.depleted) : "Ende ✓"}</div><div class="s">${s.depleted !== null ? ymShow(i2ym(s.depleted)) : "über den ganzen Zeitraum"}</div></div>
    <div class="kpi"><div class="k">Tiefstes liquides Vermögen</div><div class="v">${fmtMio(Math.max(0, s.minLiquid))}</div><div class="s">Alter ${ageStr(s.minLiquidM)}</div></div>`;
}

const css = n => getComputedStyle(document.documentElement).getPropertyValue(n).trim();
function niceStep(max, n = 4) { const raw = max / n, p = Math.pow(10, Math.floor(Math.log10(raw || 1))); return [1, 2, 2.5, 5, 10].map(x => x * p).find(x => raw <= x); }

function renderChart(svgSel, tipSel, s) {
  const svg = $(svgSel); if (!svg) return;
  const W = svg.clientWidth || 360, H = svg.clientHeight || 250;
  svg.setAttribute("viewBox", `0 0 ${W} ${H}`);
  if (!s) { svg.innerHTML = ""; return; }
  const L = 48, Rr = 10, T = 10, Bm = 26, n = s.m.length;
  const maxV = Math.max(1, ...s.total), step = niceStep(maxV), top = Math.ceil(maxV / step) * step;
  const x = i => L + i / (n - 1) * (W - L - Rr), y = v => T + (1 - v / top) * (H - T - Bm);
  let g = "";
  for (let v = 0; v <= top + 1e-6; v += step) g += `<line x1="${L}" x2="${W - Rr}" y1="${y(v)}" y2="${y(v)}" stroke="${css("--grid")}"/><text x="${L - 6}" y="${y(v) + 4}" text-anchor="end" font-size="10.5" fill="${css("--muted")}">${fmtMio(v)}</text>`;
  for (let i = 0; i < n; i++) { const mm = s.m[i]; if (mm % 60 === 0) g += `<text x="${x(i)}" y="${H - 8}" text-anchor="middle" font-size="10.5" fill="${css("--muted")}">${mm / 12}</text>`; }
  const stack = [s.pk, s.prop, s.liquid], cols = ["--c3", "--c2", "--c1"];
  let base = new Array(n).fill(0), areas = "";
  stack.forEach((arr, k) => {
    const topv = arr.map((v, i) => base[i] + v);
    const d = `M${topv.map((v, i) => `${x(i).toFixed(1)},${y(v).toFixed(1)}`).join("L")}L${base.map((v, i) => [i, v]).reverse().map(([i, v]) => `${x(i).toFixed(1)},${y(v).toFixed(1)}`).join("L")}Z`;
    areas += `<path d="${d}" fill="${css(cols[k])}" fill-opacity=".85"/>`;
    base = topv;
  });
  let ms = "";
  for (const m of state.milestones) { if (m.fixed) continue; const mm = msMonth(m.id), i = mm - s.m[0]; if (i > 0 && i < n) ms += `<line x1="${x(i)}" x2="${x(i)}" y1="${T}" y2="${H - Bm}" stroke="${css("--muted")}" stroke-dasharray="3 3" stroke-opacity=".6"/>`; }
  if (s.depleted !== null) { const i = s.depleted - s.m[0]; ms += `<line x1="${x(i)}" x2="${x(i)}" y1="${T}" y2="${H - Bm}" stroke="${css("--err")}" stroke-width="1.5"/>`; }
  svg.innerHTML = g + areas + ms + `<line id="hov" y1="${T}" y2="${H - Bm}" stroke="${css("--text")}" stroke-opacity=".5" visibility="hidden"/>`;
  svg.onmousemove = ev => {
    const r = svg.getBoundingClientRect(), px = (ev.clientX - r.left) * W / r.width;
    const i = clamp(Math.round((px - L) / (W - L - Rr) * (n - 1)), 0, n - 1), hov = svg.querySelector("#hov");
    hov.setAttribute("x1", x(i)); hov.setAttribute("x2", x(i)); hov.setAttribute("visibility", "visible");
    const tip = $(tipSel); tip.style.display = "block";
    tip.innerHTML = `<b>${ymShow(i2ym(s.m[i]))}</b> · Alter ${ageStr(s.m[i])}<br>Total ${fmtMoney(s.total[i])}<br><span style="color:var(--c1)">■</span> Liquid ${fmtMoney(s.liquid[i])}<br><span style="color:var(--c2)">■</span> Immobilien ${fmtMoney(s.prop[i])}<br><span style="color:var(--c3)">■</span> Vorsorge ${fmtMoney(s.pk[i])}`;
    const left = x(i) * r.width / W; tip.style.left = (left > r.width / 2 ? left - tip.offsetWidth - 10 : left + 10) + "px"; tip.style.top = "8px";
  };
  svg.onmouseleave = () => { $(tipSel).style.display = "none"; svg.querySelector("#hov")?.setAttribute("visibility", "hidden"); };
}

function renderTimeline() {
  const svg = $("#tl"), W = svg.clientWidth || 900, s0 = startM(), e = endM();
  $("#tlRange").textContent = Number.isFinite(s0) && e > s0 ? `${ymShow(i2ym(s0))} – ${ymShow(i2ym(e))} · ${((e - s0) / 12).toFixed(1)} Jahre` : "Zeitraum ungültig";
  if (!(e > s0)) { svg.innerHTML = ""; return; }
  const P = 14, x = m => P + (m - s0) / (e - s0) * (W - 2 * P), base = 66;
  let h = `<line x1="${P}" x2="${W - P}" y1="${base}" y2="${base}" stroke="${css("--border")}" stroke-width="4" stroke-linecap="round"/>`;
  const ret = refMonth(state.pension.pk.at);
  if (ret > s0 && ret < e) h += `<line x1="${x(ret)}" x2="${W - P}" y1="${base}" y2="${base}" stroke="${css("--accent")}" stroke-opacity=".35" stroke-width="4" stroke-linecap="round"/>`;
  for (let y = Math.ceil(s0 / 60) * 5; y * 12 <= e; y += 5) { const m = y * 12; h += `<line x1="${x(m)}" x2="${x(m)}" y1="${base + 4}" y2="${base + 9}" stroke="${css("--faint")}"/><text x="${x(m)}" y="${base + 21}" font-size="10.5" text-anchor="middle" fill="${css("--faint")}">${y} · ${Math.round(ageAt(m))} J.</text>`; }
  for (const xx of state.flows.extras) { const m = ym2i(xx.date); if (m >= s0 && m <= e) h += `<rect x="${x(m) - 3.5}" y="${base - 3.5}" width="7" height="7" transform="rotate(45 ${x(m)} ${base})" fill="${css("--warn")}"><title>${esc(xx.label)}: ${fmtMoney(xx.amount)} (${ymShow(xx.date)}, Alter ${ageStr(m)})</title></rect>`; }
  for (const ev of lastSim?.events || []) if (ev.tick) h += `<line x1="${x(ev.m)}" x2="${x(ev.m)}" y1="${base - 8}" y2="${base + 8}" stroke="${css("--c2")}" stroke-width="2.5"><title>${esc(ev.label)} (${ymShow(i2ym(ev.m))}, Alter ${ageStr(ev.m)})</title></line>`;
  if (lastSim?.depleted != null) { const m = lastSim.depleted; h += `<line x1="${x(m)}" x2="${x(m)}" y1="${base - 9}" y2="${base + 9}" stroke="${css("--err")}" stroke-width="3"><title>Geld aufgebraucht</title></line>`; }
  const items = state.milestones.map(ms => ({ ms, m: msMonth(ms.id) })).filter(o => Number.isFinite(o.m)).sort((a, b) => a.m - b.m);
  const lanes = [-Infinity, -Infinity, -Infinity];
  for (const { ms, m } of items) {
    const label = `${ms.name} · ${ymShow(i2ym(m))}`, w = label.length * 6.3 + 6, px = x(clamp(m, s0, e));
    const anchorEnd = px + w > W - P;
    const lx0 = anchorEnd ? px - w : px;
    let lane = lanes.findIndex(r => r < lx0 - 6); if (lane < 0) lane = lanes.indexOf(Math.min(...lanes));
    lanes[lane] = lx0 + w;
    const ly = 14 + lane * 15;
    h += `<g class="ms" data-action="ms-goto" data-id="${ms.id}"><line x1="${px}" x2="${px}" y1="${ly + 3}" y2="${base}" stroke="${css("--faint")}" stroke-width="1"/>
      <circle cx="${px}" cy="${base}" r="6" fill="${css(ms.fixed ? "--muted" : "--accent")}" stroke="${css("--panel")}" stroke-width="2"/>
      <text class="lbl" x="${anchorEnd ? px - 4 : px + 4}" y="${ly}" font-size="11.5" text-anchor="${anchorEnd ? "end" : "start"}" fill="${css("--text")}"><tspan font-weight="600">${esc(ms.name)}</tspan> <tspan fill="${css("--muted")}">${ymShow(i2ym(m))} · ${ageStr(m)} J.</tspan></text>
      <rect x="${lx0}" y="${ly - 12}" width="${w}" height="16" fill="transparent"/></g>`;
  }
  // Immobilienereignisse (nur mit Simulationsergebnis): Dreieck unter der Linie, Beschriftung darunter, gestaffelt.
  // Form und Lage unterscheiden sie von den Meilensteinen (Kreis auf der Linie, Beschriftung darüber).
  const pev = (lastSim?.propEvents || []).filter(o => o.m >= s0 && o.m <= e).sort((a, b) => a.m - b.m);
  const TL_EV = { purchase: "Kauf", sale: "Verkauf", emergencySale: "Notverkauf" }, compact = pev.length > 15;
  const evLanes = [-Infinity, -Infinity, -Infinity], top = 104;
  for (const o of pev) {
    const label = `${TL_EV[o.kind]} ${o.property} ${ymShow(i2ym(o.m))}`, w = label.length * 6.1 + 6, px = x(o.m), color = css(o.kind === "emergencySale" ? "--warn" : "--ok");
    const anchorEnd = px + w > W - P, lx0 = anchorEnd ? px - w : px;
    let lane = evLanes.findIndex(r => r < lx0 - 6); if (lane < 0) lane = evLanes.indexOf(Math.min(...evLanes));
    evLanes[lane] = lx0 + w;
    const ly = top + lane * 14;
    h += `<g class="pev" data-kind="${o.kind}"><title>${esc(label)} (Alter ${ageStr(o.m)})</title>
      ${compact ? "" : `<line x1="${px}" x2="${px}" y1="${base + 12}" y2="${ly - 9}" stroke="${color}" stroke-width="1" stroke-opacity=".6"/>`}
      <path d="M${px},${base + 3} l-5,9 h10 z" fill="${color}" stroke="${css("--panel")}" stroke-width="1.5"/>
      ${compact ? "" : `<text class="pelbl" x="${anchorEnd ? px - 4 : px + 4}" y="${ly}" font-size="11" text-anchor="${anchorEnd ? "end" : "start"}" fill="${color}">${esc(label)}</text>`}</g>`;
  }
  svg.setAttribute("height", pev.length && !compact ? top + 3 * 14 - 4 : 96);
  svg.innerHTML = h;
}

/** Sichtbarer Bereich je Mini-Grafik (Monatsindizes); ohne Eintrag der ganze Zeitraum. */
const miniZoom = {};
/** Gemeinsam für Editor- und Ergebnisgrafiken: Zoom-Schaltflächen, Vorgabe «die ersten N Jahre» und sichtbarer Bereich. */
const ZOOMS = [["0", "Alles"], ["10", "10 J."], ["5", "5 J."], ["2", "2 J."]];
const zoomButtons = (action, active, attrs = "") => ZOOMS.map(([y, l]) => `<button data-action="${action}" ${attrs} data-years="${y}" class="${String(active ?? 0) === y ? "on" : ""}">${l}</button>`).join("");
const presetZoom = years => (years ? { from: startM(), to: Math.min(endM(), startM() + years * 12), years } : null);
function zoomRange(z) {
  const s0 = startM(), e = endM();
  if (!z) return [s0, e];
  const span = clamp(z.to - z.from, 6, e - s0), from = clamp(z.from, s0, e - span);
  return [from, from + span];
}
const miniRange = path => zoomRange(miniZoom[path]);
function miniValue(t, v) { return t === "pct" ? fmtPct(v) + " %" : t === "months" ? `${v} Mt.` : t === "years" ? `${v} J.` : fmtInt(v); }
function drawMini(svg) {
  const path = svg.dataset.mini, ser = getP(path), t = svg.dataset.type;
  if (!ser || !(endM() > startM())) { svg.innerHTML = ""; return; }
  const [a, b] = miniRange(path), W = svg.clientWidth || 220, H = svg.clientHeight || 96, T = 14, Bm = 16;
  svg.setAttribute("viewBox", `0 0 ${W} ${H}`);
  // Exakte Stufen: Wert am Anfang des Ausschnitts und bei jeder Änderung darin.
  const changes = ser.mode === "timed" ? ser.rows.map(r => refMonth(r.from)).filter(m => m > a && m <= b) : [];
  const pts = [a, ...[...new Set(changes)].sort((x, y) => x - y)], vals = pts.map(m => seriesAt(ser, m));
  const max = Math.max(...vals, 1e-9), min = Math.min(0, ...vals);
  const X = m => (m - a) / (b - a) * W, Y = v => T + (1 - (v - min) / (max - min || 1)) * (H - T - Bm);
  let d = `M0,${Y(vals[0])}`;
  pts.forEach((m, i) => { if (i > 0) d += `H${X(m).toFixed(1)}V${Y(vals[i]).toFixed(1)}`; });
  d += `H${W}`;
  const years = (b - a) / 12, step = years > 25 ? 10 : years > 10 ? 5 : years > 4 ? 2 : 1;
  let g = "";
  for (let y = Math.ceil(a / 12); y * 12 <= b; y++) {
    const x = X(y * 12);
    if (y % step === 0) g += `<line x1="${x}" x2="${x}" y1="${T}" y2="${H - Bm}" stroke="${css("--grid")}"/><text x="${x}" y="${H - 4}" font-size="9.5" text-anchor="middle" fill="${css("--faint")}">${y}</text>`;
    else if (years <= 4) g += `<line x1="${x}" x2="${x}" y1="${H - Bm - 3}" y2="${H - Bm}" stroke="${css("--faint")}"/>`;
  }
  for (const m of state.milestones) {
    if (m.fixed) continue;
    const mm = msMonth(m.id);
    if (mm > a && mm < b) g += `<line x1="${X(mm)}" x2="${X(mm)}" y1="${T}" y2="${H - Bm}" stroke="${css("--accent")}" stroke-opacity=".35" stroke-dasharray="2 3"><title>${esc(m.name)}</title></line>`;
  }
  const dots = pts.slice(1).map((m, i) => `<circle cx="${X(m)}" cy="${Y(vals[i + 1])}" r="3" fill="${css("--accent")}"/>`).join("");
  svg.innerHTML = g + `<path d="${d}V${H - Bm}H0Z" fill="${css("--accent")}" fill-opacity=".12"/><path d="${d}" fill="none" stroke="${css("--accent")}" stroke-width="2"/>${dots}
    <text x="4" y="10" font-size="9.5" fill="${css("--muted")}" style="font-variant-numeric:tabular-nums">max ${miniValue(t, max)}</text>
    <line class="hov" y1="${T}" y2="${H - Bm}" stroke="${css("--text")}" stroke-opacity=".4" visibility="hidden"/>`;

  const tip = svg.parentElement.querySelector(".tip"), monthAt = ev => {
    const r = svg.getBoundingClientRect();
    return clamp(Math.round(a + (ev.clientX - r.left) / r.width * (b - a)), a, b);
  };
  const setZoom = (from, to) => {
    miniZoom[path] = { from, to, years: null };
    document.querySelectorAll(`[data-action="mini-zoom"][data-path="${path}"]`).forEach(x => x.classList.remove("on"));
    drawMini(svg);
  };
  svg.onwheel = ev => {
    ev.preventDefault();
    const c = monthAt(ev), f = ev.deltaY > 0 ? 1.25 : 0.8, span = clamp((b - a) * f, 6, endM() - startM());
    setZoom(c - (c - a) * span / (b - a), c - (c - a) * span / (b - a) + span);
  };
  svg.onmousedown = ev => {
    const x0 = ev.clientX, w = svg.getBoundingClientRect().width;
    svg.style.cursor = "grabbing";
    const move = e2 => { const dm = Math.round((x0 - e2.clientX) / w * (b - a)); setZoom(a + dm, b + dm); };
    const up = () => { svg.style.cursor = ""; window.removeEventListener("mousemove", move); window.removeEventListener("mouseup", up); };
    window.addEventListener("mousemove", move); window.addEventListener("mouseup", up);
  };
  svg.ondblclick = () => { delete miniZoom[path]; renderDerived(); document.querySelectorAll(`[data-action="mini-zoom"][data-path="${path}"]`).forEach(x => x.classList.toggle("on", x.dataset.years === "0")); };
  svg.onmousemove = ev => {
    if (ev.buttons) return;
    const m = monthAt(ev), hov = svg.querySelector(".hov");
    hov.setAttribute("x1", X(m)); hov.setAttribute("x2", X(m)); hov.setAttribute("visibility", "visible");
    tip.style.display = "block";
    tip.innerHTML = `${ymShow(i2ym(m))} · Alter ${ageStr(m)}<br><b>${miniValue(t, seriesAt(ser, m))}</b>`;
    const left = X(m) * svg.getBoundingClientRect().width / W;
    tip.style.left = (left > W / 2 ? left - tip.offsetWidth - 8 : left + 8) + "px"; tip.style.top = "4px";
  };
  svg.onmouseleave = () => { tip.style.display = "none"; svg.querySelector(".hov")?.setAttribute("visibility", "hidden"); };
}
function drawTaxMini(svg) {
  const t = getP("taxes." + svg.dataset.taxmini); if (!t.length) { svg.innerHTML = ""; return; }
  const xmax = Math.max(...t.map(r => r[0])) * 1.25 || 1, N = 100, vals = [];
  for (let i = 0; i <= N; i++) vals.push(rateAt(t, xmax * i / N));
  const ymax = Math.max(...vals, 1e-9), Y = v => 58 - v / ymax * 50;
  const d = vals.map((v, i) => `${i ? "L" : "M"}${i * 3},${Y(v).toFixed(2)}`).join("");
  svg.innerHTML = `<path d="${d}" fill="none" stroke="${css("--accent")}" stroke-width="2" vector-effect="non-scaling-stroke"/><text x="4" y="11" font-size="10" fill="${css("--muted")}">Satz bis ${fmtPct(ymax)} %</text>`;
}

// ---------- Ereignisse ----------
/** Meldet der Desktop-App, ob ungespeicherte Änderungen vorliegen (für die Rückfrage beim Beenden). */
function pushDirty() { if (Engine.inApp) Engine.call("set_dirty", { dirty }).catch(() => {}); }
function markDirty() { dirty = true; $("#dirty").textContent = "● ungespeichert"; pushDirty(); History.note(); stepTouched(); }

// ---------- Rückgängig / Wiederholen ----------
/** Verlauf der Planung als JSON-Schnappschüsse; schnelle Eingaben werden zusammengefasst. */
const History = {
  stack: [], pos: -1, timer: 0,
  reset() { clearTimeout(this.timer); this.stack = [JSON.stringify(state)]; this.pos = 0; invalidFields.clear(); },
  note() { clearTimeout(this.timer); this.timer = setTimeout(() => this.commit(), 400); },
  commit() {
    clearTimeout(this.timer);
    const j = JSON.stringify(state);
    if (j === this.stack[this.pos]) return;
    this.stack = this.stack.slice(0, this.pos + 1); this.stack.push(j);
    if (this.stack.length > 200) this.stack.shift();
    this.pos = this.stack.length - 1;
  },
  /** Ändert den aktuellen Schritt, ohne einen neuen anzulegen (abgeleitete Werte, FEA-78). */
  patch(f) {
    if (this.pos < 0 || !this.stack[this.pos]) return;
    const o = JSON.parse(this.stack[this.pos]); f(o); this.stack[this.pos] = JSON.stringify(o);
  },
  undo() { this.commit(); if (this.pos <= 0) return toast("Nichts mehr rückgängig zu machen."); this.pos--; this.apply(); },
  redo() { this.commit(); if (this.pos >= this.stack.length - 1) return toast("Nichts zu wiederholen."); this.pos++; this.apply(); },
  apply() {
    state = JSON.parse(this.stack[this.pos]); invalidFields.clear(); dirty = true; $("#dirty").textContent = "● ungespeichert"; pushDirty();
    if (!$("#results").hidden) showResults(false);
    renderSection(); refreshSeriesInfo();
  }
};

// ---------- Tastenkürzel ----------
const IS_MAC = /Mac|iPhone|iPad/.test(navigator.platform || navigator.userAgent);
const MOD = IS_MAC ? "⌘" : "Ctrl+", SHIFT = IS_MAC ? "⇧" : "Shift+";
const SHORTCUTS = [
  [`${MOD}N`, "Neue Planung", "new"], [`${MOD}O`, "Öffnen", "open"], [`${MOD}S`, "Speichern", "save"], [`${SHIFT}${MOD}S`, "Speichern unter", "saveas"],
  [`${MOD}I`, "Alte config.json importieren", "import"], [`${MOD},`, "Einstellungen", "settings"], [`${MOD}↩`, "Simulation starten", "run"], ["Esc", "Zurück zum Editor", "back"],
  [`${MOD}Z`, "Rückgängig", "undo"], [`${SHIFT}${MOD}Z`, "Wiederholen", "redo"],
  [`${MOD}1 … ${MOD}9, ${MOD}0`, "Abschnitt 1–10 im Editor bzw. Registerkarte in den Ergebnissen", null],
  [`${MOD}↑ / ${MOD}↓`, "Vorheriger / nächster Abschnitt", null], [`${MOD}/`, "Diese Übersicht", "help"],
  ["F1", "Hilfe zum aktuellen Thema (Feld, Abschnitt, Grafik)", null], [`${SHIFT}${MOD}?`, "Handbuch", null]
];
let lastCommand = { id: "", t: 0 };
/** Führt einen Befehl aus (Menü der Desktop-App oder Tastatur); doppelte Auslösung wird ignoriert. */
function command(id) {
  const now = performance.now();
  if (lastCommand.id === id && now - lastCommand.t < 250) return;
  lastCommand = { id, t: now };
  if ($("#ask").open && id !== "help") return;
  if ($("#welcome").open) return;
  if ($("#disclaimer").open) return;
  if (id === "undo") return History.undo();
  if (id === "redo") return History.redo();
  if (id === "help") return ask("Tastenkürzel", `<div class="kbd">${SHORTCUTS.map(([k, t]) => `<kbd>${esc(k)}</kbd><span>${esc(t)}</span>`).join("")}</div>`);
  if (id === "about") return showAbout();
  if (id === "welcome") return showWelcome();
  if (id === "steps") return Steps.show();
  if (id === "back") { if (!$("#results").hidden) showResults(false); return; }
  ACTIONS[id]?.({});
}
async function showAbout() {
  let info = null;
  try { info = await Engine.call("app_info"); } catch { /* ohne Rechenkern keine Angaben */ }
  const built = info?.buildEpoch ? new Date(info.buildEpoch * 1000).toLocaleString("de-CH", { dateStyle: "medium", timeStyle: "medium" }) : "nicht bekannt (Entwicklungsserver)";
  ask("Über VivaPlan", `<p style="margin-top:0">Pensionsplanung für die Schweiz: Monat für Monat durchgerechnet, mit Monte Carlo und Szenarien.</p>
    <table class="tbl" id="aboutTbl"><tr><td class="l">Version</td><td class="l"><b>${esc(info?.version ?? "unbekannt")}</b></td></tr>
    <tr><td class="l">Gebaut am</td><td class="l"><b>${esc(built)}</b></td></tr>
    <tr><td class="l">Website</td><td class="l">vivaplan-app.ch</td></tr></table>
    <p class="hint">Alle Daten bleiben auf diesem Rechner. Die App verbindet sich nie mit dem Internet.</p>
    <p class="hint" id="aboutDisclaimer">${esc(DISCLAIMER_FOOT)} <a href="#" data-action="about-disclaimer">Vollständiger Haftungsausschluss</a></p>`);
}
function gotoIndex(i) {
  if (!$("#results").hidden) { const b = [...document.querySelectorAll('[data-action="rtab"]')][i]; if (b) b.click(); return; }
  if (SECTIONS[i]) goto(SECTIONS[i].id);
}
document.addEventListener("keydown", e => {
  const mod = IS_MAC ? e.metaKey : e.ctrlKey, k = e.key.toLowerCase();
  if (e.key === "Enter" && e.target.matches?.("input[data-path]")) releaseField(e.target); // ISS-13
  if (e.key === "Escape" && closeInfoPops()) return;
  if (e.key === "Escape" && !document.querySelector("dialog[open]")) return command("back");
  if (!mod || e.altKey) return;
  const map = { n: "new", o: "open", s: e.shiftKey ? "saveas" : "save", i: "import", enter: "run", "/": "help", ",": "settings", y: "redo", z: e.shiftKey ? "redo" : "undo" };
  if (map[k]) { e.preventDefault(); return command(map[k]); }
  if (/^[0-9]$/.test(e.key)) { e.preventDefault(); return gotoIndex(e.key === "0" ? 9 : +e.key - 1); }
  if ((e.key === "ArrowDown" || e.key === "ArrowUp") && $("#results").hidden && !e.target.matches("input, select, textarea")) {
    e.preventDefault();
    const i = SECTIONS.findIndex(s => s.id === current) + (e.key === "ArrowDown" ? 1 : -1);
    if (SECTIONS[i]) goto(SECTIONS[i].id);
  }
});
// Menü der Desktop-App
if (window.__TAURI__?.event) window.__TAURI__.event.listen("menu", ev => command(ev.payload));
/** Beenden mit ungespeicherten Änderungen: speichern, verwerfen oder abbrechen. */
async function confirmQuit() {
  if (dirty) {
    const r = await ask("Ungespeicherte Änderungen", "Die Planung hat ungespeicherte Änderungen. Vor dem Beenden speichern?", null, "Speichern", "Verwerfen");
    if (r === null) return;
    if (r === true) { await save(false); if (dirty) return; }
  }
  Engine.call("quit_app").catch(e => toast(errText(e)));
}
if (window.__TAURI__?.event) window.__TAURI__.event.listen("quit-requested", confirmQuit);
async function importData(cmd, title, help) {
  if (!Engine.inApp) return toast(`Der Import ist in der Desktop-App verfügbar (oder: vivaplan ${cmd.replace("_", "-")}).`);
  if (!await ask(`${title} aktualisieren`, `<p>${help}</p><p>Weiter zur Dateiauswahl?</p>`)) return;
  try {
    const r = await Engine.call(cmd); if (!r) return;
    const lines = r.written.map(([f, i]) => `<p><b>${esc(f)}</b>: ${ymShow(i.first)} – ${ymShow(i.last)}${i.saronRange ? `, SARON bis ${ymShow(i.saronRange[1])}` : ""}</p>`);
    await ask("Import abgeschlossen", [...lines, ...r.notes.map(n => `<p>${esc(n)}</p>`)].join("") || "<p>Keine Dateien geändert.</p>");
    await refreshLibrary(); refreshSeriesInfo();
  } catch (e) { toast(errText(e)); }
}

function toast(msg) { const t = $("#toast"); t.textContent = msg; t.classList.add("show"); clearTimeout(toast.t); toast.t = setTimeout(() => t.classList.remove("show"), 3200); }

function readInput(el) {
  const t = el.dataset.type;
  if (t === "bool") return el.checked;
  if (t === "select" || t === "radio" || t === "text") return el.value;
  if (t === "month") return parseShow(el.value);
  if (t === "chfopt" && el.value.trim() === "") return undefined;
  const v = parseNum(el.value);
  if (v === null) return null;
  return t === "pct" ? v / 100 : v;
}
function onEdit(el) {
  const path = el.dataset.path; if (!path) return;
  if (el.dataset.type === "ref") {
    const prev = getP(path);
    setP(path, el.value === "__date" ? i2ym(Number.isFinite(refMonth(prev)) && refMonth(prev) !== Infinity ? refMonth(prev) : startM()) : el.value);
    markDirty(); renderSection(); return;
  }
  const v = readInput(el);
  // ISS-13: nur Textfelder warten; Kästchen und Optionsfelder wirken sofort
  const typing = document.activeElement === el && el.matches("input:not([type=checkbox]):not([type=radio])");
  if (typing) holdField(el);
  if (v === null) {
    invalidFields.set(path, { section: current, message: el.dataset.type === "month" ? "Ungültiges Datum (MM.JJJJ)." : "Ungültige Zahl." });
    if (!typing) { el.classList.add("invalid"); applyFieldIssues(); }
    return;
  }
  invalidFields.delete(path);
  if (!typing) el.classList.remove("invalid");
  const kindMs = /^milestones\.\d+\.kind$/.test(path) ? getP(path.replace(/\.kind$/, "")) : null;
  const ergibt = kindMs ? msMonth(kindMs.id) : NaN; // Zeitpunkt vor dem Wechsel (Spalte «Ergibt»)
  const kindPay = /^pillar3a\.\d+\.payouts\.\d+\.kind$/.test(path) ? getP(path.replace(/\.kind$/, "")) : null;
  const payPid = kindPay ? state.pillar3a[+path.split(".")[1]].person : null, payAt = kindPay ? payoutMonth(kindPay, payPid) : NaN;
  setP(path, v); markDirty();
  // FEA-78: eine eigene Eingabe im Aktienmodell beendet das Folgen der Vorschlagswerte
  if (/^returns\.parametric\.(mu|sigma|df|jumpLambda|jumpMu|jumpSigma)$/.test(path)) state.returns.parametric.auto = false;
  if (/\.amortMode$/.test(path) && v === "indirect") {
    // FEA-37: Die indirekte Amortisation braucht ein 3a-Konto (erste Person bzw. gewählte Person der Tranche).
    const t = getP(path.split(".").slice(0, 6).join(".")), pid = t?.pillar3a ?? state.household.persons[0].id;
    if (!state.pillar3a.some(a => a.person === pid)) state.pillar3a.push(newAccount(pid));
  }
  if (/^milestones\.\d+\.kind$/.test(path)) { // «Alter von» braucht Person und Jahre, «Datum» ein Datum
    const m = getP(path.replace(/\.kind$/, ""));
    // Der Zeitpunkt bleibt derselbe: «Alter von» übernimmt das Alter, «Datum» das Datum aus der Spalte «Ergibt»
    if (v === "age") {
      if (sysMs(m)) m.person = state.household.persons[sysMs(m).index]?.id;
      m.person ??= state.household.persons[0]?.id;
      m.years = Number.isFinite(ergibt) ? Math.round((ergibt - ym2i(person(m.person).birth)) / 12 * 100) / 100 : m.years ?? 65;
    } else m.date = Number.isFinite(ergibt) ? i2ym(ergibt) : m.date ?? i2ym(startM() + 60);
  }
  if (kindPay) { // FEA-41: wie bei Meilensteinen bleibt der Zeitpunkt beim Wechsel gleich
    if (v === "age") { kindPay.years = Number.isFinite(payAt) ? Math.round(ageAt(payAt, payPid) * 100) / 100 : 60; delete kindPay.date; }
    else { kindPay.date = i2ym(Number.isFinite(payAt) ? payAt : startM()); delete kindPay.years; delete kindPay.kind; }
  }
  if (path === "returns.series" || (path.startsWith("returns.strategy") && el.dataset.rerender)) refreshSeriesInfo();
  if (path.startsWith("scenarios.sweeps.") && path.endsWith(".param")) {
    const sp = SWEEP_PARAMS.find(p => p.id === v), s = getP(path.replace(/\.param$/, ""));
    [s.from, s.to, s.step] = sp.def;
  }
  if (el.dataset.rerender) renderSection(); else if (typing) pendingRefresh = true; else scheduleRefresh();
}
document.addEventListener("input", e => { const el = e.target; if (el.matches("input[data-path]:not([type=checkbox]):not([type=radio])")) onEdit(el); });
document.addEventListener("change", e => { const el = e.target; if (el.matches("select[data-path], input[type=checkbox][data-path], input[type=radio][data-path]")) onEdit(el); });
document.addEventListener("focusout", e => {
  const el = e.target; if (!el.matches("input[data-path]")) return;
  releaseField(el);
  if (el.classList.contains("invalid")) return;
  const t = el.dataset.type; if (["chf", "chfopt", "pct", "month"].includes(t)) el.value = showVal(t, getP(el.dataset.path));
});

/**
 * Leere Planung mit Standardwerten für «Neu»: Start im aktuellen Monat, eine Person ohne Namen, keine Beträge, keine
 * Immobilien, keine Variationen. Renditen, Steuertabellen und Strategieparameter behalten die Standardwerte der Vorlage.
 */
function blankPlan(base, now = new Date()) {
  const p = JSON.parse(JSON.stringify(base)), cur = now.getFullYear() * 12 + now.getMonth(), start = i2ym(cur);
  const zero = { mode: "const", value: 0 };
  p.project = { ...p.project, name: "Neue Planung" };
  p.household = { persons: [{ id: "A", name: "", birth: i2ym(cur - 60 * 12) }] };
  p.milestones = [
    { id: "start", name: "Simulationsstart", kind: "date", date: start, fixed: true },
    { id: "retire", name: "Bezug Pensionskasse", kind: "age", person: "A", years: 65 },
    { id: "ahv", name: "AHV-Bezug", kind: "age", person: "A", years: 65 },
    { id: "end", name: "Simulationsende", kind: "age", person: "A", years: 95, fixed: true },
  ];
  delete p.partner;
  p.pillar3a = [];
  p.pension = { ...p.pension, pk: { ...p.pension.pk, capital: 0, at: "retire", lumpsum: 0, contribution: { ...zero }, interest: { mode: "const", value: 0.02 } },
    ahv: { ...p.pension.ahv, monthly: 0, at: "ahv" } };
  p.flows = { income: { ...zero }, incomeIndexed: true, spending: { ...zero }, extras: [], spendingIncludesHousing: true };
  p.wealth = { ...p.wealth, stocks: 0, bonds: 0, cash: 0 };
  p.realestate = { ...p.realestate, properties: [] };
  p.scenarios = { ...p.scenarios, sweeps: [] };
  delete p.optimization;
  return p;
}
const ACTIONS = {
  goto: d => goto(d.sec),
  "ms-goto": d => { flashId = d.id; goto("milestones"); flashId = null; },
  set: d => { setP(d.path, d.value); markDirty(); renderSection(); if (d.path === "returns.source") refreshSeriesInfo(); },
  // FEA-78: wieder den Vorschlagswerten des Aktienindex folgen
  "param-suggest": () => { state.returns.parametric.auto = true; followSuggestion(); markDirty(); renderSection(); },
  "series-mode": d => {
    const s = getP(d.path); if (s.mode === d.mode) return;
    if (d.mode === "timed") { s.rows = [{ from: "start", value: s.value ?? 0 }]; delete s.value; }
    else { s.value = s.rows?.[0]?.value ?? 0; delete s.rows; }
    s.mode = d.mode; markDirty(); renderSection();
  },
  "row-add": d => { const rows = getP(d.path), last = rows[rows.length - 1]; rows.push({ from: i2ym(Math.max(startM(), refMonth(last?.from ?? "start") === Infinity ? startM() : refMonth(last?.from ?? "start")) + 12), value: last?.value ?? 0 }); markDirty(); renderSection(); },
  "row-del": d => { getP(d.path).splice(+d.i, 1); markDirty(); renderSection(); },
  "list-del": d => {
    if (d.path === "realestate.properties") {
      const id = state.realestate.properties[+d.i].id;
      const users = state.realestate.properties.filter(q => q.buy === "afterSale:" + id || q.from === "afterSale:" + id);
      if (users.length) return toast(`Wird noch verwendet von: ${users.map(q => q.name).join(", ")}.`);
    }
    getP(d.path).splice(+d.i, 1); markDirty(); renderSection();
  },
  "extra-add": () => { state.flows.extras.push({ date: i2ym(startM() + 12), amount: -10000, label: "Neuer Betrag" }); markDirty(); renderSection(); },
  "tax-add": d => { const t = getP("taxes." + d.key), last = t[t.length - 1] || [0, 0]; t.push([last[0] * 2 || 100000, last[1]]); markDirty(); renderSection(); },
  "ms-add": () => { state.milestones.splice(state.milestones.length - 1, 0, { id: "ms" + Date.now().toString(36), name: "Neuer Meilenstein", kind: "date", date: i2ym(startM() + 60) }); markDirty(); renderSection(); },
  "ms-del": async d => {
    const ms = state.milestones[+d.i];
    if (!ms || msLocked(ms)) return;
    const refs = milestoneRefs(ms.id);
    if (!refs.length) { state.milestones.splice(+d.i, 1); markDirty(); renderSection(); return; }
    const m = msMonth(ms.id);
    if (!Number.isFinite(m)) return toast(`«${ms.name}» hat kein gültiges Datum und lässt sich nicht umstellen.`);
    const date = i2ym(m), sections = [...new Set(refs.map(r => r.section))];
    const ok = await ask("Meilenstein löschen", `«${esc(ms.name)}» wird in ${refs.length} ${refs.length === 1 ? "Eingabe" : "Eingaben"} verwendet (Abschnitte: ${sections.map(esc).join(", ")}). Beim Löschen werden diese Eingaben auf das feste Datum ${ymShow(date)} umgestellt.`, null, "Löschen und umstellen");
    if (!ok) return;
    History.commit();
    for (const r of refs) r.parent[r.key] = date;
    // Zeitreihen: gleicher Zeitpunkt in mehreren Zeilen → die spätere Zeile gilt
    let dups = 0;
    const fix = o => {
      if (!o || typeof o !== "object") return;
      if (o.mode === "timed" && Array.isArray(o.rows) && o.rows.some(r => r.from === date)) {
        const last = new Map(); o.rows.forEach((r, i) => last.set(refMonth(r.from), i));
        const keep = o.rows.filter((r, i) => last.get(refMonth(r.from)) === i);
        dups += o.rows.length - keep.length; o.rows = keep;
      }
      for (const [k, v] of Object.entries(o)) if (!(o === state && k === "milestones")) fix(v);
    };
    state.milestones.splice(+d.i, 1);
    fix(state);
    markDirty(); History.commit(); renderSection();
    toast(`${refs.length} ${refs.length === 1 ? "Eingabe" : "Eingaben"} auf ${ymShow(date)} umgestellt.${dups ? ` Doppelter Zeitpunkt in einer Reihe: nur die spätere Zeile bleibt (${dups} entfernt).` : ""}`);
  },
  "p3a-add": d => { state.pillar3a.push(newAccount(d.person)); markDirty(); renderSection(); },
  "p3a-payout-add": d => {
    // FEA-41: ein Jahr nach der letzten Zeile (ohne Zeile: frühester erlaubter Monat), höchstens im AHV-Bezug; Betrag
    // = Guthaben laut Vorschau zu diesem Zeitpunkt nach den früheren Auszahlungen, abgerundet auf 1'000.
    const a = state.pillar3a[+d.k], pi = state.household.persons.findIndex(q => q.id === a.person), ahv = msMonth(pi === 1 ? "ahv2" : "ahv");
    const s0 = startM(), months = a.payouts.map(x => payoutMonth(x, a.person)).filter(Number.isFinite);
    let m = months.length ? Math.max(...months) + 12 : Math.max(s0, Number.isFinite(ahv) ? ahv - 60 : s0);
    if (Number.isFinite(ahv)) m = Math.min(m, ahv);
    const last = a.payouts.at(-1), row = { use: "wealth" };
    if (last?.kind === "age") Object.assign(row, { kind: "age", years: Math.round(ageAt(m, a.person) * 100) / 100 }); else row.date = i2ym(m);
    const bal = p3aViewOf(a.person)?.balances?.[m - s0];
    if (Number.isFinite(bal) && bal >= 1000) row.amount = Math.floor(bal / 1000) * 1000;
    a.payouts.push(row);
    markDirty(); renderSection();
  },
  settings: () => showSettings(),
  "partner-add": () => {
    const a = state.household.persons[0], id = a.id === "B" ? "C" : "B";
    state.household.persons.push({ id, name: "Person 2", birth: a.birth });
    state.partner = newPartner();
    const at = state.milestones.length - 1;
    state.milestones.splice(at, 0, { id: "retire2", name: "Bezug Pensionskasse Person 2", kind: "age", person: id, years: 65 }, { id: "ahv2", name: "AHV-Bezug Person 2", kind: "age", person: id, years: 65 });
    markDirty(); renderSection();
  },
  "partner-del": () => {
    const b = state.household.persons[1], saved = state.partner;
    state.partner = null;
    const own = state.milestones.filter(m => m.kind === "age" && m.person === b.id);
    const used = own.find(m => countRefs(m.id));
    if (used) { state.partner = saved; return toast(`«${used.name}» wird noch verwendet; bitte zuerst dort ersetzen.`); }
    state.milestones = state.milestones.filter(m => !own.includes(m));
    state.household.persons.splice(1, 1); delete state.partner; delete state.taxes.couple;
    state.pillar3a = state.pillar3a.filter(a => a.person !== b.id);
    markDirty(); renderSection();
  },
  "couple-tax": () => {
    if (state.taxes.couple) delete state.taxes.couple;
    else state.taxes.couple = { income: structuredClone(state.taxes.income), wealth: structuredClone(state.taxes.wealth), capital: structuredClone(state.taxes.capital) };
    markDirty(); renderSection();
  },
  pstat: d => {
    const p = state.realestate.properties[+d.i]; p.status = d.v;
    if (d.v === "rent") { p.from ??= "retire"; p.rent ??= 2000; }
    else { p.price ??= 800000; p.fixCosts ??= 400; p.inflation ??= true; p.mortgage ??= newMortgage(); }
    if (d.v === "owned") { p.worth ??= p.price; p.sell ??= "never"; p.mortgage.tranches.forEach(t => { t.start ??= i2ym(startM()); }); }
    if (d.v === "planned") { p.buy ??= "retire"; p.sell ??= "never"; }
    markDirty(); renderSection();
  },
  "prop-add": () => { state.realestate.properties.push({ id: newPropId(), name: "Neues Objekt", status: "planned", buy: "retire", sell: "never", price: 800000, fixCosts: 400, inflation: true, inflationValue: true, inflationFixCosts: true, inflationRent: true, mortgage: newMortgage() }); markDirty(); renderSection(); },
  "period-add": d => { const ps = state.realestate.properties[+d.i].mortgage.tranches[+d.t].periods, last = ps[ps.length - 1]; ps.push(newPeriod(last ? { ...last, term: 5 } : {})); markDirty(); renderSection(); },
  "tranche-add": d => {
    const p = state.realestate.properties[+d.i], ts = p.mortgage.tranches;
    ts.push({ name: `Tranche ${ts.length + 1}`, amount: 100000, ...(p.status === "owned" ? { start: i2ym(startM()) } : {}), after: "renew", periods: [newPeriod()] });
    markDirty(); renderSection();
  },
  "sweep-add": () => { const sp = SWEEP_PARAMS.find(p => !state.scenarios.sweeps.some(s => s.param === p.id)) || SWEEP_PARAMS[0]; state.scenarios.sweeps.push({ param: sp.id, from: sp.def[0], to: sp.def[1], step: sp.def[2] }); markDirty(); renderSection(); },
  json: () => { $("#dlgPre").textContent = JSON.stringify(state, null, 2); $("#dlg").showModal(); },
  close: () => $("#dlg").close(),
  copy: () => { navigator.clipboard?.writeText(JSON.stringify(state, null, 2)).then(() => toast("In die Zwischenablage kopiert."), () => toast("Kopieren nicht möglich.")); },
  download: () => download(),
  new: async () => {
    if (!await confirmDiscard()) return;
    await adopt(blankPlan(await Engine.call("example_plan"))); setFile(null); History.reset(); goto("general"); refreshSeriesInfo();
  },
  open: async () => {
    if (!Engine.inApp) return $("#fileInput").click();
    if (!await confirmDiscard()) return;
    try { await showOpened(await Engine.call("open_plan")); } catch (e) { toast(errText(e)); }
  },
  save: () => save(false),
  saveas: () => save(true),
  import: async () => {
    if (!Engine.inApp) return toast("Der Import alter Konfigurationen ist in der Desktop-App verfügbar.");
    if (!await confirmDiscard()) return;
    const y = await ask("Alte config.json importieren", "Das alte Format kennt nur Startalter und Startmonat. In welchem <b>Kalenderjahr</b> beginnt die Simulation?", String(new Date().getFullYear()));
    if (!y) return;
    if (!/^\d{4}$/.test(y)) return toast("Bitte ein Jahr wie 2026 eingeben.");
    try {
      const r = await Engine.call("import_legacy", { startYear: +y }); if (!r) return;
      const notes = [...r.notes, ...await adopt(r.plan)]; setFile(null, true); History.reset(); goto("general"); refreshSeriesInfo();
      if (notes.length) await ask("Hinweise zum Import", notes.map(n => `<p>${esc(n)}</p>`).join(""));
    } catch (e) { toast(errText(e)); }
  },
  "import-shiller": () => importData("import_shiller", "Shiller-Daten",
    "Die neueste Datei <code>ie_data.xls</code> gibt es auf der Website von Robert Shiller (Yale). Der Import ersetzt «Shiller USD.csv» und «Shiller CHF.csv» in der Bibliothek."),
  "import-snb": () => importData("import_snb", "SNB-Daten",
    "Auf data.snb.ch die Datenwürfel <b>devkum</b> (Devisenkurse), <b>plkopr</b> (Landesindex der Konsumentenpreise), <b>zimoma</b> (SARON) bzw. <b>rendoblim</b> (Renditen der Eidgenossenschaft) als CSV herunterladen und hier wählen (eine oder mehrere Dateien). Die CHF-Reihen werden danach neu berechnet."),
  run: () => runSimulation(),
  help: () => command("help"),
  about: () => command("about"),
  // FEA-45/AK4: aus dem Dialog «Über VivaPlan» zum vollständigen Haftungsausschluss in der Hilfe
  "about-disclaimer": () => { $("#ask").close(); Help.show("disclaimer"); },
  back: () => showResults(false),
  detail: d => showDetail(+d.p),
  rtab: d => { resultTab = d.tab; renderResults(lastRun); },
  real: d => { realView = d.v === "1"; renderResults(lastRun); },
  "bk-toggle": d => { bucketOpen.has(d.y) ? bucketOpen.delete(d.y) : bucketOpen.add(d.y); renderResults(lastRun); },
  "info-toggle": (d, el) => { const open = !el.classList.contains("open"); document.querySelectorAll(".infopop.open").forEach(x => x.classList.remove("open")); if (open) el.classList.add("open"); },
  "tax-cols": () => { showTaxCols = !showTaxCols; Pref.set("showTaxCols", showTaxCols); renderResults(lastRun); },
  comp: d => { compPercent = d.v === "1"; renderResults(lastRun); },
  mkt: d => { mktPct = d.v === "pct"; renderResults(lastRun); },
  "mkt-log": d => { mktLog = d.v === "1"; renderResults(lastRun); },
  // Zoom der Ergebnisgrafiken: gilt für alle Registerkarten und bleibt beim Wechsel erhalten
  "var-opt": () => { const o = Optimum.find(lastRun?.scenarios, 50); if (o) pickVariation(o.items.map(it => it.grid)); },
  "var-plan": () => pickVariation(state.scenarios.sweeps.map(() => null)),
  "res-zoom": d => { stepZoom(+d.years); redrawResults(); },
  "mini-zoom": d => {
    const y = +d.years;
    if (y) miniZoom[d.path] = presetZoom(y);
    else delete miniZoom[d.path];
    document.querySelectorAll(`[data-action="mini-zoom"][data-path="${d.path}"]`).forEach(x => x.classList.toggle("on", x.dataset.years === d.years));
    renderDerived();
  }
};
/** Übernimmt eine geöffnete Planung ({path, plan}) in den Editor. */
async function showOpened(r) {
  if (!r) return;
  const notes = await adopt(r.plan); setFile(r.path); History.reset(); goto("general"); refreshSeriesInfo();
  toast(`«${baseName(r.path)}» geöffnet.`);
  await showNotes(notes);
  await reserveNotice(state);
}
// FEA-34: Doppelklick auf eine Planung im Finder, während die App läuft.
if (window.__TAURI__?.event) window.__TAURI__.event.listen("open-file", async ev => {
  if (!await confirmDiscard()) return;
  try { await showOpened(await Engine.call("open_path", { path: ev.payload })); } catch (e) { toast(errText(e)); }
});
/** FEA-34: Fenstertitel «VivaPlan - ‹Dateiname›», ohne Datei nur «VivaPlan». */
function showTitle(name) {
  document.title = name ? `VivaPlan - ${name}` : "VivaPlan";
  if (Engine.inApp) Engine.call("set_window_title", { path: currentPath }).catch(() => {});
}
function setFile(path, isDirty = false) {
  currentPath = path; dirty = isDirty;
  showTitle(path ? baseName(path) : null);
  $("#fileName").textContent = path ? baseName(path) : "Neue Planung";
  $("#dirty").textContent = isDirty ? "● ungespeichert" : "";
  pushDirty();
}
async function confirmDiscard() {
  return !dirty || !!await ask("Ungespeicherte Änderungen", "Die aktuelle Planung hat ungespeicherte Änderungen. Trotzdem fortfahren?");
}
async function save(as) {
  if (!Engine.inApp) { download(); setFile(currentPath); return; }
  try {
    const p = await Engine.call("save_plan", { plan: state, path: as ? null : currentPath });
    if (!p) return;
    setFile(p); toast(`Gespeichert: ${baseName(p)}`);
  } catch (e) { toast(errText(e)); }
}
/** Eigener Dialog (statt prompt/confirm). Mit `value` gibt es ein Eingabefeld. */
function ask(title, html, value = null, okLabel = "OK", discardLabel = null) {
  return new Promise(res => {
    const d = $("#ask"), inp = $("#askInput"), dis = d.querySelector('button[value="discard"]');
    dis.hidden = !discardLabel; if (discardLabel) dis.textContent = discardLabel;
    d.querySelector('button[value="ok"]').textContent = okLabel;
    $("#askTitle").textContent = title; $("#askText").innerHTML = html;
    inp.hidden = value === null; inp.value = value ?? ""; d.returnValue = "";
    // FEA-87: Das Ereignis «close» kommt verzögert. Ist der Dialog schon wieder offen (nächste Frage), gehört es zur vorherigen.
    d.onclose = () => { if (!d.open) res(d.returnValue === "discard" ? "discard" : d.returnValue === "ok" ? (value === null ? true : inp.value.trim()) : null); };
    d.showModal(); if (value !== null) inp.select();
  });
}
function download() {
  const a = document.createElement("a");
  a.href = URL.createObjectURL(new Blob([JSON.stringify(state, null, 2)], { type: "application/json" }));
  a.download = (state.project.name || "planung").replace(/[^\w\-äöüÄÖÜ ]/g, "") + ".vplan"; a.click();
  setTimeout(() => URL.revokeObjectURL(a.href), 1000);
}
Object.assign(ACTIONS, OPT_ACTIONS, { "job-cancel": jobCancel });
document.addEventListener("change", e => { if (/^opt(S)?-/.test(e.target.id || "")) readOptCfg(); });
document.addEventListener("click", e => {
  // Klick ausserhalb schliesst geöffnete Info-Popups; im Popup bleibt Text markier- und kopierbar.
  if (!e.target.closest(".infopop")) document.querySelectorAll(".infopop.open").forEach(x => x.classList.remove("open"));
  if (e.target.closest(".infobody")) return;
  const el = e.target.closest("[data-action]"); if (!el) return;
  const fn = ACTIONS[el.dataset.action]; if (!fn) return;
  // Kontrollkästchen behalten ihren Zustand (preventDefault würde das Häkchen zurücksetzen).
  if (el.type !== "checkbox") e.preventDefault();
  fn(el.dataset, el);
});
$("#fileInput").addEventListener("change", async e => {
  const f = e.target.files[0]; if (!f) return;
  try {
    const obj = JSON.parse(await f.text());
    // FEA-34: alte Kennung pensionsplaner/1 bleibt lesbar (legacy: RetirementPlanner).
    if (obj.format !== "vivaplan/1" && obj.format !== "pensionsplaner/1") throw 0;
    const notes = await adopt(obj); setFile(null); History.reset(); $("#fileName").textContent = f.name; showTitle(f.name); goto("general"); refreshSeriesInfo();
    toast(`«${f.name}» geöffnet.`);
    await showNotes(notes);
    await reserveNotice(state);
  } catch { toast("Datei ist keine Projektdatei im neuen Format."); }
  e.target.value = "";
});
window.addEventListener("resize", () => { renderDerived(); if (lastRun) renderResults(lastRun); });
matchMedia("(prefers-color-scheme: dark)").addEventListener("change", () => { renderDerived(); if (lastRun) renderResults(lastRun); });

// ---------- Einstellungen (FEA-24) ----------
/** Standardwerte der Einstellungen; der Rechenkern kennt dieselben (crates/engine/src/settings.rs). */
const LIMIT_DEFAULTS = { ahvMin: 63, ahvMax: 70, pkMin: 58, pkMax: 65, pillar3aMax: 7258 };
const LIMIT_FIELDS = [["ahvMin", "AHV-Bezug: untere Grenze", "Jahre"], ["ahvMax", "AHV-Bezug: obere Grenze", "Jahre"],
  ["pkMin", "Bezug Pensionskasse: untere Grenze", "Jahre"], ["pkMax", "Bezug Pensionskasse: obere Grenze", "Jahre"],
  ["pillar3aMax", "Säule 3a: Maximalbetrag der Einzahlung pro Jahr", null]];
/** Geltende Grenzen: gespeicherte Werte, sonst die Standardwerte. Sie gelten für alle Planungen dieses Geräts. */
function limits() {
  const saved = Pref.get("limits", null), l = { ...LIMIT_DEFAULTS };
  if (saved && typeof saved === "object") for (const k of Object.keys(LIMIT_DEFAULTS)) if (Number.isFinite(saved[k])) l[k] = saved[k];
  return l;
}
Engine.limits = limits;
/** Prüfung wie im Rechenkern (`Limits::problems`). */
function limitProblems(l) {
  const out = [], whole = x => Number.isInteger(x) && x >= 0 && x <= 130;
  for (const [what, lo, hi] of [["AHV-Bezug", l.ahvMin, l.ahvMax], ["Bezug Pensionskasse", l.pkMin, l.pkMax]]) {
    if (!whole(lo) || !whole(hi)) out.push(`${what}: Die Grenzen müssen ganze Jahre zwischen 0 und 130 sein.`);
    else if (lo > hi) out.push(`${what}: Die untere Grenze (${lo}) liegt über der oberen Grenze (${hi}).`);
  }
  if (!(Number.isFinite(l.pillar3aMax) && l.pillar3aMax >= 0)) out.push("Säule 3a: Der Maximalbetrag muss 0 oder mehr sein.");
  return out;
}
/** FEA-86: Rückfrage zum Speichern vor der Simulation (Standard: ein). */
const askSaveOn = () => Pref.get("askSaveBeforeRun", true) !== false;
function settingsFields(l) {
  return `<label class="check" style="grid-column:1/-1"><input type="checkbox" id="set-askSave" ${askSaveOn() ? "checked" : ""}> Vor der Simulation fragen, ob die Planung gespeichert werden soll</label>`
    + LIMIT_FIELDS.map(([k, label, unit]) => `<label class="field"><span>${label}</span><div class="inp"><input class="num" id="set-${k}" value="${esc(k === "pillar3aMax" ? fmtInt(l[k]) : String(l[k]))}"><span class="unit">${unit ?? cur()}</span></div><span class="computed">Standard ${k === "pillar3aMax" ? fmtInt(LIMIT_DEFAULTS[k]) : LIMIT_DEFAULTS[k]}</span></label>`).join("");
}
function readSettings() {
  const l = {};
  for (const [k] of LIMIT_FIELDS) { const v = parseNum($("#set-" + k).value); l[k] = v === null ? NaN : v; }
  return l;
}
/** Dialog «Einstellungen»: gilt für alle Planungen, wird lokal gespeichert, nie in der Planungsdatei. */
function showSettings() {
  const d = $("#settings");
  $("#settingsBody").innerHTML = settingsFields(limits()); $("#settingsErr").textContent = "";
  d.showModal();
}
$("#settingsSave").addEventListener("click", e => {
  const l = readSettings(), problems = limitProblems(l);
  if (problems.length) { e.preventDefault(); $("#settingsErr").innerHTML = problems.map(esc).join("<br>"); return; }
  Pref.set("limits", l); Pref.set("askSaveBeforeRun", $("#set-askSave").checked);
  $("#settings").close(); toast("Einstellungen gespeichert.");
  renderSection();
});
$("#settingsDefaults").addEventListener("click", async e => {
  e.preventDefault();
  if (!await ask("Standardwerte", "Alle Einstellungen auf die Standardwerte zurücksetzen?", null, "Zurücksetzen")) return;
  Pref.set("limits", { ...LIMIT_DEFAULTS }); Pref.set("askSaveBeforeRun", true);
  $("#settingsBody").innerHTML = settingsFields(limits()); $("#settingsErr").textContent = "";
  renderSection();
});

// ---------- Ergebnisansicht ----------
/**
 * Dauerhafte Einstellungen der Oberfläche. In der Desktop-App liegen sie als Datei im Datenverzeichnis (Befehle
 * `settings_get`/`settings_set`), im Browser-Entwicklungsmodus im Browserspeicher. Nicht lesbar: es gelten die Standardwerte.
 */
const Pref = {
  cache: {},
  async load() {
    try { this.cache = (Engine.inApp ? await Engine.call("settings_get") : JSON.parse(localStorage.getItem("rp.settings") || "{}")) || {}; }
    catch { this.cache = {}; }
  },
  get(key, fallback) { return key in this.cache ? this.cache[key] : fallback; },
  set(key, value) {
    this.cache[key] = value;
    if (Engine.inApp) Engine.call("settings_set", { key, value }).catch(() => { /* gilt dann nur für diese Sitzung */ });
    else try { localStorage.setItem("rp.settings", JSON.stringify(this.cache)); } catch { /* wie oben */ }
  },
};
let showTaxCols = false;
/** Jahre, deren 3-Topf-Herleitung in den Jahreswerten aufgeklappt ist. */
const bucketOpen = new Set();
let resultTab = "overview", realView = false, compPercent = false, detailP = 50, mktPct = false, mktLog = false, resultZoom = null, compareOpen = false;
function showResults(on) {
  if (!on) unpark();
  $("#results").hidden = !on; $(".layout").hidden = on; $("#timeline").hidden = on;
  if (on) $("#resultsPick").innerHTML = "";
  if (!on) { lastRun = null; renderDerived(); }
}
/** Fragt vor einer Berechnung über ${LONG_RUN_SECONDS} s geschätzter Dauer nach; `false`, wenn der Benutzer abbricht. */
async function confirmLongRun() {
  if (bench.us == null) await ensureBench();
  if (bench.us == null) return true;
  const { runs, sec } = scenSeconds();
  if (!(sec > LONG_RUN_SECONDS)) return true;
  return !!await ask("Lange Berechnung", `Die Berechnung dauert voraussichtlich ca. <b>${esc(fmtShort(sec))}</b> (${fmtInt(runs)} ${runs === 1 ? "Simulationslauf" : "Simulationsläufe"}). Sie lässt sich während der Berechnung abbrechen.`, null, "Trotzdem starten");
}
/**
 * FEA-86: Hat die Planung ungespeicherte Änderungen, fragt die App vor der Simulation, ob gespeichert werden soll.
 * Nur in der Desktop-App (im Browser ist «Speichern» ein Download); `askSaveDev` in den Einstellungen schaltet es für Tests ein.
 * `false`, wenn der Benutzer abbricht oder das Speichern nicht zustande kommt.
 */
async function confirmSaveBeforeRun() {
  if (!dirty || !askSaveOn() || !(Engine.inApp || Pref.get("askSaveDev", false))) return true;
  const r = await ask("Planung speichern?", `Die Planung hat ungespeicherte Änderungen${currentPath ? "" : " und noch keinen Dateinamen"}. Vor der Simulation speichern?`, null, "Speichern", "Ohne Speichern simulieren");
  if (r === null) return false;
  if (r === true) { await save(false); if (dirty) return false; }
  return true;
}
async function runSimulation(tab = "overview", { noConfirm = false } = {}) {
  if (await blockedByErrors()) return;
  if (!await confirmSaveBeforeRun()) return;
  if (!noConfirm && !await confirmLongRun()) return;
  unpark(); showResults(true); resultTab = tab; detailP = 50; resetVariation();
  $("#resultsTitle").textContent = "Simulation läuft…"; $("#resultsInfo").textContent = ""; $("#resultsTabs").innerHTML = "";
  $("#resultsBody").innerHTML = jobCard("Simulation läuft…", "Die Berechnung läuft lokal auf diesem Rechner.");
  try {
    const planAtRun = JSON.parse(JSON.stringify(state));
    lastRun = await withJob(() => Engine.call("run_simulation", { plan: state, path: currentPath, jobId: job.id }));
    lastRunPlan = planAtRun;
    if (isCancelled(lastRun.error)) { lastRun = null; showResults(false); return toast("Simulation abgebrochen."); }
    renderResults(lastRun); document.documentElement.dataset.rendered = resultTab; rememberRun(lastRun);
    if (!lastRun.error) Steps.mark("run"); }
  catch (e) { $("#resultsTitle").textContent = "Fehler"; $("#resultsBody").innerHTML = `<div class="card errcard">${esc(errText(e))}</div>`; }
}
const kpi = (k, v, s = "", cls = "") => `<div class="kpi ${cls}"><div class="k">${k}</div><div class="v">${v}</div><div class="s">${s}</div></div>`;
/** Legende; Eintrag [Farbe, Text, Deckkraft, Linienart]: mit Linienart «solid» oder «dashed» ein Linien-Symbol statt des Kästchens. */
const legend = items => `<div class="legend">${items.map(([c, t, o, line]) => `<span>${line
  ? `<svg class="lsym" width="22" height="10" viewBox="0 0 22 10" aria-hidden="true"><line x1="0" x2="22" y1="5" y2="5" stroke="var(${c})" stroke-width="2" ${line === "dashed" ? 'stroke-dasharray="5 3"' : ""}/></svg>`
  : `<i style="background:var(${c})${o ? `;opacity:${o}` : ""}"></i>`}${t}</span>`).join("")}</div>`;
const WEALTH_LEGEND = legend([["--c1", "Liquides Vermögen"], ["--c2", "Immobilien (netto)"], ["--c3", "Vorsorge (PK, 3a)"]]);
const atAge = month => `${ymShow(month)} (Alter ${ageStr(ym2i(month))})`;
/** Hat die Planung Hypotheken, deren Erneuerung auf Tragbarkeit geprüft wird? */
const hasProperties = () => state.realestate.properties.some(p => p.status !== "rent");
/** Verkäufe von Immobilien eines Laufs, je Verkauf eine Zeile «Name: MM.JJJJ»; Notverkäufe mit Warnsymbol, ohne Verkauf «–». */
function propertySales(res) {
  const sales = (res?.propertyEvents || []).filter(e => e.kind === "sale" || e.kind === "emergencySale").sort((a, b) => ym2i(a.month) - ym2i(b.month));
  return sales.length ? sales.map(e => `<div>${esc(e.property)}: ${ymShow(e.month)}${e.kind === "emergencySale" ? ` ${houseWarn("Notverkauf")}` : ""}</div>`).join("") : "–";
}
/** Immobilienereignisse eines Laufs, chronologisch: Kauf, Verkauf, Notverkauf (mit Grund) und Hypothekenerneuerung mit ihren Beträgen. */
function propertyEvents(res) {
  const evs = [...(res.propertyEvents || [])].sort((a, b) => ym2i(a.month) - ym2i(b.month));
  const n = v => fmtInt(v), part = (label, v) => `<span class="pe"><span class="k">${label}</span> ${v}</span>`;
  const TYPE = { purchase: "Kauf", sale: "Verkauf", emergencySale: "Notverkauf", renewal: "Hypothekenerneuerung" };
  const detail = e => {
    if (e.kind === "purchase") return [part("Kaufpreis", n(e.price)), part("Eigenmittel", n(e.equity)), part("Hypothek", n(e.mortgage))];
    if (e.kind === "sale" || e.kind === "emergencySale") {
      const p = [part("Verkaufserlös", n(e.proceeds)), part("abgelöste Hypothek", n(e.debtRepaid))];
      if (e.gainTax > 0) p.push(part("Grundstückgewinnsteuer", n(e.gainTax)));
      p.push(part("Nettoerlös", `<b>${n(e.net)}</b>`));
      if (e.reason) p.push(part("Grund", `<span class="bad">${esc(e.reason)}</span>`));
      if (e.note) p.push(part("Hinweis", esc(e.note)));
      return p;
    }
    const p = [part("Tragbarkeit", e.affordable ? "gegeben" : `<span class="bad">nicht gegeben</span>`)];
    if (e.reason) p.push(part("Hypothek", `<span class="bad">nicht erneuert</span> (${n(e.debtBefore)}), Notverkauf: ${esc(e.reason)}`));
    else p.push(part("Hypothek vorher / nachher", `${n(e.debtBefore)} / ${n(e.debtAfter)}`));
    if (e.lumpSum > 0) p.push(part("Einmalzahlung (Tragbarkeit)", `<b>${n(e.lumpSum)}</b>`));
    if (e.rate != null) p.push(part("Zinssatz", `${(e.rate * 100).toLocaleString("de-CH", { maximumFractionDigits: 2 })} %`));
    return p;
  };
  const body = evs.length
    ? `<div class="tscroll"><table class="tbl"><tr><th class="l">Zeitpunkt</th><th class="l">Ereignis</th><th class="l">Immobilie</th><th class="l">Beträge in ${cur()}</th></tr>
      ${evs.map(e => `<tr><td class="l">${atAge(e.month)}</td><td class="l ${e.kind === "emergencySale" ? "bad" : ""}">${TYPE[e.kind]}</td><td class="l">${esc(e.property)}</td><td class="l pes">${detail(e).join("")}</td></tr>`).join("")}</table></div>
      <p class="hint">Beträge nominal. Bei jeder Hypothekenerneuerung wird die Tragbarkeit mit dem Einkommen geprüft. Ist sie nicht gegeben, zahlt die Simulation die minimal nötige Einmalzahlung aus dem Vermögen, sofern das Vermögen danach nicht negativ ist und das Notverkaufskriterium nicht auslöst. Sonst wird die Hypothek nicht erneuert und ein Notverkauf eingeleitet. Eigenmittel beim Kauf = Kaufpreis − Hypothek.</p>`
    : `<p class="hint" style="margin:0">Keine Immobilienereignisse.</p>`;
  return card("Immobilien", "Kauf, Verkauf, Notverkauf und Hypothekenerneuerung", body);
}
/**
 * 3-Topf-Strategie: Herleitung der Topfgrössen (Ausgaben, abgezogene Einnahmen, Netto-Bedarf, Ziele), Renditekriterium,
 * ausgelöster Fall (Januar, A, B, C), Umbuchungen und Topfstände vor und nach der Auffüllung (jeweils vor den Ausgaben des Monats).
 * Die Auffüllungen eines Jahres erscheinen aufklappbar in den Jahreswerten (`yearTable`).
 */
function bucketDerivation(rs) {
  const n = v => fmtInt(v);
  const pct = v => `${(v * 100).toLocaleString("de-CH", { maximumFractionDigits: 1 })} %`;
  const CASE = { Januar: "Januar-Auffüllung", A: "A · Start / PK-Kapitalbezug / Neustart", B: "B · Zufluss", C1: "C1 · Vorausschau, Kriterium erfüllt", C2: "C2 · Vorausschau, Kriterium nicht erfüllt" };
  const side = x => (x === 0 ? "extern" : x === 4 ? "Reservierung" : `Topf ${x}`);
  const rows = rs.map(r => {
    const b = r.basis, inflow = r.trigger === "B";
    const cond = inflow ? "–" : r.conditionIgnored ? `<small>ignoriert</small>` : `${r.conditionMet ? "erfüllt" : `<span class="bad">nicht erfüllt</span>`}<br><small>${pct(r.stockReturn)} (${esc(r.returnBasis)}) ${r.conditionMet ? "≥" : "<"} ${pct(r.threshold)}</small>`;
    const moves = r.moves?.length ? r.moves.map(x => `${side(x.from)} → ${side(x.to)}: ${n(x.amount)}`).join("<br>") : "–";
    const basis = inflow ? `<td colspan="8" class="l"><small>keine Auffüllung von Topf 1 und 2</small></td>`
      : `<td>${n(b.living)}${b.livingSource === "Guardrails" ? "<br><small>Guardrails</small>" : ""}</td><td>${n(b.housing)}</td><td>${n(b.pillar3a ?? 0)}</td><td>${n(b.incomeTax)}</td><td>${n(b.ahv)}</td><td>${n(b.pk)}</td><td>${n(b.earned)}</td><td><b>${n(r.need)}</b></td>`;
    const targets = inflow ? "<td>–</td><td>–</td>" : `<td>${n(r.target1)}</td><td>${n(r.target2)}</td>`;
    const passed = (r.trigger === "A" || r.trigger === "C1") && r.passedMonths > 0 ? `<br><small>Ziele um ${r.passedMonths} Monate gekürzt</small>` : "";
    const base = inflow || !b.source ? "" : `<br><small>${esc(b.source)}</small>`;
    return `<tr><td class="l">${atAge(r.month)}</td><td class="l"><b>${esc(CASE[r.trigger] || r.trigger)}</b><br><small>${esc(r.reason)}</small>${base}${passed}</td>
      ${basis}${targets}<td class="l">${cond}</td>
      <td class="l">${moves}${r.payment > 0 && r.trigger !== "B" ? `<br><small>vor den Ausgaben des Monats (${n(r.payment)})</small>` : ""}</td>
      <td>${n(r.before[0])} / ${n(r.before[1])} / ${n(r.before[2])}</td><td>${n(r.after[0])} / ${n(r.after[1])} / ${n(r.after[2])}</td></tr>`;
  }).join("");
  return `<div class="tscroll"><table class="tbl num"><tr><th class="l">Zeitpunkt</th><th class="l">Ausgelöst durch</th>
      <th title="Geplante Lebenshaltung; bei 3-Topf mit Guardrails die Lebenshaltung der Guardrails (vermerkt)">Lebenshaltung</th><th>Wohnen</th><th title="Einzahlungen in die Säule 3a (FEA-37)">3a</th><th title="Steuer auf dem (hochgerechneten) Jahreseinkommen">Einkommens&shy;steuer</th><th title="abgezogene Einnahmen: AHV-Rente pro Jahr">AHV</th><th title="abgezogene Einnahmen: PK-Rente pro Jahr">PK-Rente</th><th title="abgezogene Einnahmen: Erwerbseinkommen pro Jahr">Erwerb</th><th title="Lebenshaltung + Wohnen + 3a + Einkommenssteuer − AHV − PK-Rente − Erwerbseinkommen">Netto-Bedarf</th>
      <th>Ziel Topf 1</th><th>Ziel Topf 2</th><th class="l" title="Aktienrendite der letzten 12 Monate (vor dem Start aus dem Vorlauf des Renditepfads) gegenüber der Schwelle: Nur bei erfüllter Bedingung wird in der Januar-Auffüllung und in C1 aus Topf 3 verkauft. A und C2 ignorieren sie.">Renditekriterium</th><th class="l" title="Umbuchungen zwischen den Töpfen">Umbuchungen</th><th title="Topf 1 / Topf 2 / Topf 3 vor der Auffüllung">Vorher T1 / T2 / T3</th><th title="Topf 1 / Topf 2 / Topf 3 nach der Auffüllung">Nachher T1 / T2 / T3</th></tr>${rows}</table></div>`;
}
const bucketHint = () => `<p class="hint">Beträge in ${cur()}, nominal, pro Jahr; die Topfstände gelten vor den Ausgaben des Monats. <b>Netto-Bedarf</b> = Lebenshaltung + Wohnen (ohne Amortisation) + Einzahlung in die Säule 3a + Einkommenssteuer − AHV − PK-Rente − Erwerbseinkommen, aus den geplanten Werten des laufenden Jahres (bei 3-Topf mit Guardrails: Lebenshaltung gemäss Guardrails, nach Unter- und Obergrenze); einmalige Ausgaben, Anschaffungen, Käufe und Sondersteuern zählen nicht. Ziel Topf 1 und 2 = Jahre × Netto-Bedarf. <b>Januar-Auffüllung:</b> Topf 1 zuerst aus Topf 2, der Rest aus Topf 3, danach Topf 2 aus Topf 3; ist das Renditekriterium nicht erfüllt, wird nichts aus Topf 3 verkauft und Topf 2 nicht nachgefüllt. Ein Überschuss in Topf 1 oder 2 geht in Topf 3. <b>A:</b> beim Start der Entnahmephase (erste Pensionierung), bei einem PK-Kapitalbezug und beim Neustart nach «alle Töpfe leer»: Das Vermögen (Kapitalbezug) geht zuerst in Topf 3, dann wie im Januar ohne Renditekriterium; die Ziele sind um die vergangenen Monate des Jahres gekürzt, der Netto-Bedarf aus den Monaten ab dem Auslöser auf 12 Monate hochgerechnet. <b>B:</b> Zuflüsse gehen direkt an ihr Ziel und erscheinen als Buchung «extern → Topf 3» (einmalige Zuflüsse, 3a, Verkaufserlöse ohne Folgekauf, Notverkäufe), «extern → Reservierung» (Verkauf mit Kauf im Folgemonat) oder «Reservierung → Topf 3» (Überschuss nach dem Kauf); Topf 1 und 2 werden nicht aufgefüllt. «Vorher» zeigt den Stand vor dem Zufluss. <b>C (Vorausschau):</b> Deckt Topf 1 die Ausgaben des Monats nicht, gilt bei erfülltem Renditekriterium <b>C1</b>: Ablauf wie die Januar-Auffüllung, die Ziele um die seit Januar vergangenen Monate gekürzt und das Ziel von Topf 1 um die einmaligen Ausgaben des Monats erhöht, ohne Überschussbuchung in Topf 3. Bei nicht erfülltem Kriterium gilt <b>C2</b>: Topf 1 wird aus Topf 2, dann Topf 3 so aufgefüllt, dass er die Ausgaben deckt und danach noch (Min(Reserve, Monate bis Januar) − 1) Monatsbedarfe enthält; Topf 2 wird nicht aufgefüllt. Die Überschussbuchung in Topf 3 gibt es nur in der Januar-Auffüllung. Reichen alle Töpfe nicht, wird eine Immobilie notverkauft; ohne Immobilie darf Topf 1 negativ werden, bis das Vermögen wieder positiv ist.</p>`;
/** 3-Topf-Buchhaltung (Auffüllungen, Zuflüsse, Reservierungen) steht in den Jahreswerten; die Meldungen der Defizitphase bleiben. */
const isBucketBooking = e => (e.kind === "buckets" && !/^3-Topf: alle Töpfe leer/.test(e.text))
  || (state.withdrawal.strategy === "three_buckets" && e.kind === "withdrawal" && /reserviert für Kauf im Folgemonat/.test(e.text));
function eventList(evs) {
  evs = evs.filter(e => !isBucketBooking(e));
  return evs.length ? `<ul class="evlist">${evs.map(e => `<li><span>${atAge(e.month)}</span><span>${esc(e.text)}</span></li>`).join("")}</ul>` : `<p class="hint" style="margin:0">Keine Ereignisse.</p>`;
}
/** Betrag nominal oder in Preisen des Startmonats. */
const rv = (r, x) => realView ? x / r.priceIndex : x;
const realNote = () => realView ? `in heutiger Kaufkraft (Preise ${ymShow(i2ym(startM()))})` : "nominal";
const xLabel = m => `${m / 12} · ${Math.round(ageAt(m))} J.`;
const card = (title, sub, body, cls = "") => `<div class="card ${cls}"><h3>${title}${sub ? ` <span class="sub">${sub}</span>` : ""}</h3>${body}</div>`;
/**
 * Zoom der Ergebnisgrafiken, gemeinsam für alle: { from, to, years } in Monaten seit Planstart; `years` ist die gewählte
 * Breite einer Schaltfläche, bei einer Mausauswahl null (benutzerdefiniert); null = ganzer Zeitraum.
 */
const resultView = () => (resultZoom ? [startM() + resultZoom.from, startM() + resultZoom.to] : null);
const redrawResults = () => { if (compareOpen) compareRuns(); else if (lastRun) renderResults(lastRun); };
/** Schaltfläche «N J.»: Breite N Jahre, zentriert auf die Mitte des aktuellen Bereichs (ohne Zoom: der Laufzeit), im Zeitraum gehalten. */
function stepZoom(years) {
  const total = endM() - startM(), w = years * 12;
  if (!years || w >= total) { resultZoom = null; return; }
  const mid = resultZoom ? (resultZoom.from + resultZoom.to) / 2 : total / 2;
  const from = clamp(Math.round(mid - w / 2), 0, total - w);
  resultZoom = { from, to: from + w, years };
}
/** Mausauswahl (Monate seit Planstart): Auswahlen unter einem Jahr werden ignoriert, die ganze Laufzeit setzt zurück. */
function setCustomZoom(from, to) {
  const total = endM() - startM(), a = Math.max(0, Math.round(from)), b = Math.min(total, Math.round(to));
  if (b - a < 12) return;
  resultZoom = a <= 0 && b >= total ? null : { from: a, to: b, years: null };
  redrawResults();
}
const zoomAbs = (a, b) => setCustomZoom(a - startM(), b - startM());
/** Zoom-Schaltflächen der Ergebnisgrafiken; derselbe Zustand in allen Grafiken. Nach einer Mausauswahl ist keine Schaltfläche aktiv. */
const resultZoomBar = () => {
  const custom = resultZoom && resultZoom.years == null, v = resultView();
  const note = custom ? `Benutzerdefiniert: ${((v[0] - startM()) / 12).toLocaleString("de-CH", { maximumFractionDigits: 1 })} – ${((v[1] - startM()) / 12).toLocaleString("de-CH", { maximumFractionDigits: 1 })} J. nach Start` : "Bereich mit der Maus aufziehen · gilt für alle Ergebnisgrafiken";
  return `<div class="minizoom">${zoomButtons("res-zoom", resultZoom ? (resultZoom.years ?? -1) : 0)}<span>${note}</span></div>`;
};
const chartBox = (id, h = 300, zoom = false) => `${zoom ? resultZoomBar() : ""}<div class="chartwrap"><svg id="${id}" class="bigchart" style="height:${h}px"></svg><div id="${id}-tip" class="tip"></div></div>`;
const FAN_LEGEND = legend([["--c1", "5.–95. Perzentil", ".25"], ["--c1", "25.–75. Perzentil", ".5"], ["--c1", "Median", null, "solid"], ["--warn", "1. und 99. Perzentil", null, "dashed"], ["--err", "Schlechtester Wert", null, "solid"]])
  + `<p class="hint">Die Linie «Schlechtester Wert» zeigt für jeden Monat den tiefsten Wert aller Läufe; das kann in jedem Monat ein anderer Lauf sein. Der einzelne schlechteste Lauf (nach Endvermögen) ist unter «Einzelne Läufe» abrufbar.</p>`;
const pName = p => p === 0 ? "Schlechtester Lauf" : `${p}. Perzentil`;
const pctTxt = v => `${(v * 100).toLocaleString("de-CH", { maximumFractionDigits: 1 })} %`;

/**
 * Jahreswerte: Bestände am Jahresende, Geldflüsse als Monatsdurchschnitt des Jahres, Steuern als Jahressumme.
 * `start`: Gesamt- und liquides Vermögen zum Simulationsstart (Grundlage der Entnahmerate im ersten Jahr).
 * `refills`: 3-Topf-Auffüllungen des Laufs; Jahre mit Auffüllung haben eine aufklappbare Zeile mit der Herleitung.
 */
function yearTable(recs, start, refills = []) {
  const by = new Map();
  for (const r of recs) { const y = r.month.slice(0, 4); if (!by.has(y)) by.set(y, []); by.get(y).push(r); }
  const sum = (rs, f) => rs.reduce((a, r) => a + rv(r, f(r)), 0), avg = (rs, f) => sum(rs, f) / rs.length;
  const cell = (v, bold) => `<td>${Math.abs(v) < 0.5 ? "" : bold ? `<b>${fmtInt(v)}</b>` : fmtInt(v)}</td>`;
  /** Entnahmerate: jährliche Entnahme (nominal, unvollständige Jahre auf 12 Monate hochgerechnet) / Vermögen am Jahresanfang. */
  const rateCell = (draw, base) => !(draw > 0) ? "<td></td>" : !(base > 0) ? "<td>–</td>" : `<td>${(draw / base * 100).toLocaleString("de-CH", { minimumFractionDigits: 1, maximumFractionDigits: 1 })} %</td>`;
  const tx = showTaxCols, cols = 12 + (tx ? 5 : 0);
  const refillsOf = y => refills.filter(r => r.month.slice(0, 4) === y);
  let prevLast = null;
  const rows = [...by.entries()].map(([y, rs]) => {
    const last = rs[rs.length - 1], part = rs.length < 12 ? `<span title="Unvollständiges Jahr: ${rs.length} Monate" style="color:var(--muted)">*</span>` : "";
    const planned = avg(rs, r => r.plannedSpending), housing = avg(rs, r => r.housing), income = avg(rs, r => r.income), draw = avg(rs, r => r.withdrawal);
    // FEA-37: Einzahlungen in die Säule 3a zählen zu den Ausgaben.
    const p3aIn = avg(rs, r => r.pillar3aIn ?? 0);
    const total = planned + housing + p3aIn, receipts = income + draw;
    const prev = prevLast ? prevLast.priceIndex : 1, infl = last.priceIndex / prev - 1;
    const yearDraw = rs.reduce((a, r) => a + r.withdrawal, 0) / rs.length * 12;
    const baseTotal = prevLast ? prevLast.total : start?.total;
    prevLast = last;
    const rf = refillsOf(y), open = bucketOpen.has(y);
    const toggle = rf.length ? `<button class="btn ghost small bkbtn" data-action="bk-toggle" data-y="${y}" aria-expanded="${open}" title="Herleitung der 3-Topf-Auffüllungen dieses Jahres ${open ? "ausblenden" : "anzeigen"}">${open ? "▾" : "▸"}</button>` : "";
    return `<tr><td>${toggle}${y}${part}</td><td>${last.age.toFixed(1)}</td><td>${fmtInt(rv(last, last.liquid))}</td><td>${fmtInt(rv(last, last.propertyEquity))}</td><td>${fmtInt(rv(last, last.pensionFund + last.pillar3a))}</td><td><b>${fmtInt(rv(last, last.total))}</b></td>
      ${cell(total, true)}${cell(draw)}${cell(receipts, true)}${rateCell(yearDraw, baseTotal)}${cell(receipts - total)}<td>${(infl * 100).toLocaleString("de-CH", { minimumFractionDigits: 1, maximumFractionDigits: 1 })} %</td>
      ${tx ? `${cell(sum(rs, r => r.assessedIncome ?? 0))}${cell(sum(rs, r => r.assessedWealth ?? 0))}${cell(sum(rs, r => r.taxCapital))}${cell(sum(rs, r => r.taxGain))}${cell(sum(rs, r => (r.assessedIncome ?? 0) + (r.assessedWealth ?? 0) + r.taxCapital + r.taxGain), true)}` : ""}</tr>`
      + (rf.length && open ? `<tr class="bk"><td colspan="${cols}" class="l">${bucketDerivation(rf)}</td></tr>` : "");
  }).join("");
  const taxHint = tx ? ` Die <b>Steuern</b> stehen in der Zeile des <b>Steuerjahres</b>, für das sie gelten: Einkommens- und Vermögenssteuer werden im Dezember veranlagt, aber erst im Mai des Folgejahres bezahlt (der Cash-Bestand sinkt erst dann). Kapitalbezugs- und Grundstückgewinnsteuer stehen im Jahr des Bezugs bzw. Verkaufs. Die Steuer des letzten, unvollständigen Jahres fehlt, wenn die Planung vor Dezember endet. Die Summe über alle Jahre bleibt gleich.` : "";
  return `<div class="seg" style="margin-bottom:8px"><button class="${tx ? "on" : ""}" data-action="tax-cols" aria-pressed="${tx}" title="Spalten mit den Steuern des Jahres ein- oder ausblenden">${tx ? "Steuern ausblenden" : "Steuern anzeigen"}</button></div>
    <div class="tscroll"><table class="tbl num yt"><tr><th>Jahr</th><th>Alter</th><th title="Vermögen am Jahresende: liquide Mittel">Liquid</th><th title="Vermögen am Jahresende: Immobilien abzüglich Hypotheken">Immobilien</th><th title="Vermögen am Jahresende: Pensionskasse und Säule 3a">Vorsorge</th><th title="Vermögen am Jahresende: Summe">Total</th>
      <th title="Monatsdurchschnitt des Jahres: geplante Ausgaben plus Miete bzw. Hypothekarzins, Amortisation und übrige Wohnkosten, plus Einzahlung in die Säule 3a">Ausgaben total</th><th title="Monatsdurchschnitt des Jahres: was dem Vermögen entnommen wird (tatsächliche Ausgaben minus Einkommen, vor Steuern)">Entnahme</th><th title="Monatsdurchschnitt des Jahres: Einkommen plus Entnahme aus dem Vermögen">Einkommen total</th><th title="Jährliche Entnahme / Gesamtvermögen am Jahresanfang (Total des Vorjahres)">Entnahmerate</th>
      <th title="Monatsdurchschnitt des Jahres: Einkommen total minus Ausgaben total. 0 = wie geplant, negativ = Kürzung gegenüber dem Plan, positiv = höhere Ausgaben als geplant">Abweichung vom Plan</th><th title="Veränderung des Preisindex im Kalenderjahr (im ersten Jahr seit Start)">Inflation<br>pro Jahr</th>
      ${tx ? `<th class="tx" title="Steuer des Steuerjahres der Zeile (veranlagt im Dezember, bezahlt im Mai des Folgejahres); Kapitalbezugs- und Grundstückgewinnsteuer im Jahr des Bezugs bzw. Verkaufs.">Steuer<br>Einkommen</th><th class="tx">Steuer<br>Vermögen</th><th class="tx">Steuer<br>Kapital Bezug</th><th class="tx">Steuer<br>Grundstück</th><th class="tx">Steuer<br>Total</th>` : ""}</tr>${rows}</table></div>
    <p class="hint">Die <b>Vermögen</b> (Liquid, Immobilien, Vorsorge, Total) sind <b>Stände am Jahresende</b>. <b>Ausgaben total</b>, <b>Entnahme</b>, <b>Einkommen total</b> und <b>Abweichung vom Plan</b> sind <b>Monatsdurchschnitte</b> des Kalenderjahrs.${taxHint} <b>Ausgaben total</b> = geplante Ausgaben aus der Planung plus Miete bzw. Hypothekarzins und übrige Wohnkosten plus Einzahlung in die Säule 3a. <b>Entnahme</b> = was dem Vermögen entnommen wird (tatsächliche Ausgaben minus Einkommen, vor Steuern). <b>Einkommen total</b> = Einkommen plus Entnahme. <b>Abweichung vom Plan</b> = Einkommen total − Ausgaben total: bei 0 verläuft alles wie geplant; bei Guardrails &amp; Co. zeigt ein negativer Wert, um wie viel die Strategie die Ausgaben kürzt. <b>Inflation pro Jahr</b> = Veränderung des Preisindex im Kalenderjahr (im ersten Jahr seit Simulationsstart). <b>Entnahmerate</b> = jährliche Entnahme / Gesamtvermögen am Jahresanfang. Die jährliche Entnahme ist die Summe der Monatsentnahmen des Jahres (bei unvollständigen Jahren der Monatsdurchschnitt mal 12), das Vermögen am Jahresanfang der Wert des Vorjahres, im ersten Jahr der Stand zum Simulationsstart; gerechnet wird immer mit nominalen Werten. Ist die Entnahme null oder negativ, bleibt die Zelle leer; ist das Vermögen null oder negativ, steht «–». Ein Stern (*) kennzeichnet ein unvollständiges Jahr (erstes und letztes).</p>${refills.length ? `<p class="hint"><b>3-Topf-Strategie:</b> Jahre mit Auffüllung haben vorn einen Pfeil (▸); er klappt die Herleitung der Topfgrössen und der Auffüllungen dieses Jahres auf.</p>${bucketHint()}` : ""}`;
}
function singleKpis(s) {
  const dep = s.depleted ? ym2i(s.depleted) : null;
  return `<div class="rkpis">
    ${kpi("Vermögen bei Pensionierung", s.totalAtRetirement != null ? fmtMio(s.totalAtRetirement) : "–", ymShow(i2ym(refMonth(state.pension.pk.at))))}
    ${kpi("Vermögen am Ende", fmtMio(realView ? s.endTotalReal : s.endTotal), `Alter ${ageStr(endM())}, ${realView ? "heutige Kaufkraft" : "nominal"}`, s.endTotal > 0 ? "good" : "bad")}
    ${kpi("Geld reicht bis", dep !== null ? "Alter " + ageStr(dep) : "Ende ✓", dep !== null ? ymShow(s.depleted) : "über den ganzen Zeitraum", dep !== null ? "bad" : "good")}
    ${kpi("PK-Rente", fmtMoney(s.pensionFundMonthly), `pro Monat · Kapital netto ${fmtMio(s.lumpsumNet)}`)}
    ${kpi("Steuern total", fmtMio(s.taxesTotal), "Einkommen, Vermögen, Kapital, Grundstückgewinn")}</div>`;
}

/** Summen pro Kalenderjahr für die Balkendiagramme. */
function yearly(recs) {
  const by = new Map();
  for (const r of recs) {
    const y = r.month.slice(0, 4);
    if (!by.has(y)) by.set(y, { y, earned: 0, pk: 0, ahv: 0, living: 0, housing: 0, p3a: 0, taxes: 0, liquidStart: rv(r, r.liquid + r.spending - r.income + r.taxes), months: 0 });
    const a = by.get(y);
    a.earned += rv(r, r.earned); a.pk += rv(r, r.pkPension); a.ahv += rv(r, r.ahv);
    const p3a = r.pillar3aIn ?? 0;
    a.housing += rv(r, r.housing); a.p3a += rv(r, p3a); a.living += rv(r, r.spending - r.housing - p3a); a.taxes += rv(r, r.taxes); a.months++;
  }
  return [...by.values()].map(a => ({ ...a, draw: a.living + a.housing + a.p3a + a.taxes - a.earned - a.pk - a.ahv }));
}

/** Diagramme eines einzelnen Laufs (Einzellauf oder Perzentil-Lauf). */
function runChartsHtml(res, p) {
  const recs = res.records, hasProp = recs.some(r => r.propertyValue > 0), hasPk = recs.some(r => r.pensionFund > 0 || r.pillar3a > 0);
  return card("Vermögensverlauf", realNote(), chartBox(`${p}-wealth`, 300, true) + WEALTH_LEGEND)
    + card("Liquides Vermögen nach Anlage", realNote(), `<div class="seg" style="margin-bottom:8px"><button class="${compPercent ? "" : "on"}" data-action="comp" data-v="0">Beträge</button><button class="${compPercent ? "on" : ""}" data-action="comp" data-v="1">Anteile in %</button></div>`
      + chartBox(`${p}-comp`, 300, true) + legend([["--c1", "Aktien"], ["--c2", "Obligationen"], ["--c3", "Cash"]]))
    + card("Einnahmen und Ausgaben pro Jahr", realNote(), chartBox(`${p}-flows`, 300, true)
      + legend([["--c2", "Erwerbseinkommen"], ["--c1", "PK-Rente"], ["--c4", "AHV"], ["--c3", "Lebenshaltung"], ["--warn", "Wohnkosten"], ["--err", "Steuern"]])
      + `<p class="hint">Linke Säule Einnahmen, rechte Säule Ausgaben. Kalenderjahre; das erste und letzte Jahr sind unvollständig.</p>`)
    + card("Entnahme aus dem Vermögen", "pro Jahr: Ausgaben inkl. Steuern − Einnahmen, im Verhältnis zum liquiden Vermögen", chartBox(`${p}-draw`, 260, true)
      + `<p class="hint">Die Entnahmerate zeigt, welcher Anteil des liquiden Vermögens pro Jahr verbraucht wird (Faustregel: unter 4 % gilt als nachhaltig).</p>`)
    + (hasProp ? card("Immobilien", realNote(), chartBox(`${p}-prop`, 260, true) + legend([["--c2", "Marktwert"], ["--err", "Hypothek"], ["--c1", "Eigenkapital"]])) : "")
    + (hasPk ? card("Vorsorge", realNote(), chartBox(`${p}-pk`, 240, true) + legend([["--c3", "Pensionskasse"], ["--c2", "Säule 3a"]])) : "")
    + card("Aktien, Obligationen und Inflation", `${mktPct ? "Änderung gegenüber dem Vorjahresmonat in %" : "Wertentwicklung seit Start = 100"}, ${realNote()}`,
      `<div class="seg" style="margin-bottom:8px"><button class="${mktPct ? "" : "on"}" data-action="mkt" data-v="idx">Wertentwicklung (Start = 100)</button><button class="${mktPct ? "on" : ""}" data-action="mkt" data-v="pct">Änderung in %</button></div>`
      + (mktPct ? "" : ` <div class="seg" style="margin-bottom:8px"><button class="${mktLog ? "" : "on"}" data-action="mkt-log" data-v="0">Linear</button><button class="${mktLog ? "on" : ""}" data-action="mkt-log" data-v="1">Logarithmisch</button></div>`)
      + chartBox(`${p}-mkt`, 240, true)
      + legend([["--c1", "Aktien"], ["--c2", "Obligationen"], ["--muted", "Inflation (Preisindex)"]])
      + `<p class="hint">${mktPct ? "Änderung der letzten 12 Monate in Prozent (die ersten 12 Monate haben noch keinen Wert). Die logarithmische Darstellung gibt es nur bei der Wertentwicklung. " : "Verlauf der Renditen dieses Laufs als Index (Start = 100, Ausschüttungen wiederangelegt); die Inflation läuft als Preisindex mit. Logarithmisch zeigen gleiche Abstände gleiche prozentuale Änderungen. "}Im Tooltip steht zusätzlich die Änderung der letzten 12 Monate, bei der Inflation in Klammern.</p>`);
}
function drawRunCharts(res, p) {
  const recs = res.records, x = recs.map(r => ym2i(r.month));
  const tipHead = i => `<b>${ymShow(recs[i].month)}</b> · Alter ${ageStr(x[i])}<br>`;
  const marks = res.events.filter(e => ["retirement", "ahvStart", "sale", "emergencySale", "purchase", "depleted"].includes(e.kind)).map(e => ({ x: ym2i(e.month), label: `${atAge(e.month)}: ${e.text}`, color: e.kind === "depleted" || e.kind === "emergencySale" ? "--err" : "--muted" }));
  const w = k => recs.map(r => rv(r, k(r)));
  Charts.time($(`#${p}-wealth`), $(`#${p}-wealth-tip`), { x, xFmt: xLabel, view: resultView(), onZoom: zoomAbs, yFmt: fmtMio, marks,
    stack: [{ color: "--c3", values: w(r => Math.max(0, r.pensionFund + r.pillar3a)) }, { color: "--c2", values: w(r => Math.max(0, r.propertyEquity)) }, { color: "--c1", values: w(r => Math.max(0, r.liquid)) }],
    tip: i => tipHead(i) + `Total ${fmtMoney(rv(recs[i], recs[i].total))}<br><span style="color:var(--c1)">■</span> Liquid ${fmtMoney(rv(recs[i], recs[i].liquid))}<br><span style="color:var(--c2)">■</span> Immobilien ${fmtMoney(rv(recs[i], recs[i].propertyEquity))}<br><span style="color:var(--c3)">■</span> Vorsorge ${fmtMoney(rv(recs[i], recs[i].pensionFund + recs[i].pillar3a))}` });
  Charts.time($(`#${p}-comp`), $(`#${p}-comp-tip`), { x, xFmt: xLabel, view: resultView(), onZoom: zoomAbs, yFmt: fmtMio, percent: compPercent,
    stack: [{ color: "--c3", values: w(r => r.cash) }, { color: "--c2", values: w(r => r.bonds) }, { color: "--c1", values: w(r => r.stocks) }],
    tip: i => { const r = recs[i], t = Math.max(1, r.cash + r.bonds + r.stocks), q = v => compPercent ? `${(Math.max(0, v) / t * 100).toFixed(1)} %` : fmtMoney(rv(r, v));
      return tipHead(i) + `<span style="color:var(--c1)">■</span> Aktien ${q(r.stocks)}<br><span style="color:var(--c2)">■</span> Obligationen ${q(r.bonds)}<br><span style="color:var(--c3)">■</span> Cash ${q(r.cash)}`; } });
  // Balkendiagramme: nur die Jahre, die den Zoombereich berühren
  const view = resultView(), allYears = yearly(recs);
  const ys = view ? allYears.filter(a => +a.y * 12 + 11 >= view[0] && +a.y * 12 <= view[1]) : allYears;
  Charts.bars($(`#${p}-flows`), $(`#${p}-flows-tip`), { cats: ys.map(a => a.y), onSelect: (a, b) => setCustomZoom(+ys[a].y * 12 - startM(), +ys[b].y * 12 + 11 - startM()), yFmt: fmtMio,
    series: [{ group: "in", color: "--c2", values: ys.map(a => a.earned) }, { group: "in", color: "--c1", values: ys.map(a => a.pk) }, { group: "in", color: "--c4", values: ys.map(a => a.ahv) },
      { group: "out", color: "--c3", values: ys.map(a => a.living) }, { group: "out", color: "--warn", values: ys.map(a => a.housing) }, { group: "out", color: "--c4", values: ys.map(a => a.p3a) }, { group: "out", color: "--err", values: ys.map(a => a.taxes) }],
    tip: i => { const a = ys[i]; return `<b>${a.y}</b> (${a.months} Monate)<br>Einnahmen ${fmtMoney(a.earned + a.pk + a.ahv)}<br>· Erwerb ${fmtMoney(a.earned)}<br>· PK-Rente ${fmtMoney(a.pk)}<br>· AHV ${fmtMoney(a.ahv)}<br>Ausgaben ${fmtMoney(a.living + a.housing + a.p3a + a.taxes)}<br>· Lebenshaltung ${fmtMoney(a.living)}<br>· Wohnen ${fmtMoney(a.housing)}<br>${a.p3a ? `· Säule 3a ${fmtMoney(a.p3a)}<br>` : ""}· Steuern ${fmtMoney(a.taxes)}`; } });
  const rate = ys.map(a => a.liquidStart > 0 ? Math.max(0, a.draw) / a.liquidStart * 12 / a.months : 0);
  Charts.bars($(`#${p}-draw`), $(`#${p}-draw-tip`), { cats: ys.map(a => a.y), onSelect: (a, b) => setCustomZoom(+ys[a].y * 12 - startM(), +ys[b].y * 12 + 11 - startM()), yFmt: v => `${(v * 100).toFixed(1)} %`,
    series: [{ color: "--accent", values: rate }],
    tip: i => `<b>${ys[i].y}</b><br>Entnahme ${fmtMoney(Math.max(0, ys[i].draw))}<br>Liquides Vermögen zu Jahresbeginn ${fmtMoney(ys[i].liquidStart)}<br><b>Entnahmerate ${pctTxt(rate[i])}</b> (auf 12 Monate)` });
  Charts.time($(`#${p}-prop`), $(`#${p}-prop-tip`), { x, xFmt: xLabel, view: resultView(), onZoom: zoomAbs, yFmt: fmtMio, marks,
    lines: [{ color: "--c2", values: w(r => r.propertyValue) }, { color: "--err", values: w(r => r.mortgage) }, { color: "--c1", values: w(r => r.propertyEquity), dash: "5 3" }],
    tip: i => tipHead(i) + `Marktwert ${fmtMoney(rv(recs[i], recs[i].propertyValue))}<br>Hypothek ${fmtMoney(rv(recs[i], recs[i].mortgage))}<br>Eigenkapital ${fmtMoney(rv(recs[i], recs[i].propertyEquity))}` });
  const idx = (k, i) => 100 * recs[i][k] / (realView ? recs[i].priceIndex : 1);
  const last12 = (k, i) => { const a = i >= 12 ? recs[i - 12][k] / (realView ? recs[i - 12].priceIndex : 1) : 1, b = recs[i][k] / (realView ? recs[i].priceIndex : 1); return b / a - 1; };
  const infl12 = i => i >= 12 ? recs[i].priceIndex / recs[i - 12].priceIndex - 1 : NaN;
  const pct12 = (k, i) => i >= 12 ? 100 * last12(k, i) : NaN;
  const priceIdx = i => 100 * recs[i].priceIndex;
  const inflTxt = i => Number.isFinite(infl12(i)) ? pctTxt(infl12(i)) : "–";
  const stockTxt = (k, i) => i >= 12 ? pctTxt(last12(k, i)) : "–";
  const all = recs.map((_, i) => i);
  Charts.time($(`#${p}-mkt`), $(`#${p}-mkt-tip`), { x, xFmt: xLabel, view: resultView(), onZoom: zoomAbs, marks, log: mktLog && !mktPct,
    yFmt: mktPct ? v => `${v.toLocaleString("de-CH", { maximumFractionDigits: 1 })} %` : v => v.toLocaleString("de-CH", { maximumFractionDigits: 0 }),
    lines: mktPct
      ? [{ color: "--c1", values: all.map(i => pct12("stockIndex", i)) }, { color: "--c2", values: all.map(i => pct12("bondIndex", i)) }, { color: "--muted", dash: "5 3", width: 1.5, values: all.map(i => 100 * infl12(i)) }]
      : [{ color: "--c1", values: all.map(i => idx("stockIndex", i)) }, { color: "--c2", values: all.map(i => idx("bondIndex", i)) }, { color: "--muted", dash: "5 3", width: 1.5, values: all.map(priceIdx) }],
    tip: i => tipHead(i) + (mktPct ? "" : `<span style="color:var(--c1)">■</span> Aktien ${idx("stockIndex", i).toFixed(1)} (12 Monate ${stockTxt("stockIndex", i)})<br><span style="color:var(--c2)">■</span> Obligationen ${idx("bondIndex", i).toFixed(1)} (12 Monate ${stockTxt("bondIndex", i)})<br><span style="color:var(--muted)">■</span> Preisindex ${priceIdx(i).toFixed(1)} (Inflation 12 Monate ${inflTxt(i)})`)
      + (mktPct ? `<span style="color:var(--c1)">■</span> Aktien ${stockTxt("stockIndex", i)}<br><span style="color:var(--c2)">■</span> Obligationen ${stockTxt("bondIndex", i)}<br><span style="color:var(--muted)">■</span> Inflation ${inflTxt(i)}` : "") });
  Charts.time($(`#${p}-pk`), $(`#${p}-pk-tip`), { x, xFmt: xLabel, view: resultView(), onZoom: zoomAbs, yFmt: fmtMio,
    lines: [{ color: "--c3", values: w(r => r.pensionFund) }, { color: "--c2", values: w(r => r.pillar3a) }],
    tip: i => tipHead(i) + `Pensionskasse ${fmtMoney(rv(recs[i], recs[i].pensionFund))}<br>Säule 3a ${fmtMoney(rv(recs[i], recs[i].pillar3a))}` });
}

// ---------- Monte Carlo und mehrere Läufe ----------
function histQuantile(h, q) {
  if (!h) return NaN;
  const total = h.counts.reduce((a, b) => a + b, 0); let acc = 0;
  for (let i = 0; i < h.counts.length; i++) { acc += h.counts[i]; if (acc >= q * total) return (h.edges[i] + h.edges[i + 1]) / 2; }
  return h.edges[h.edges.length - 1];
}
function batchChartsHtml(b) {
  const st = b.stats;
  return card("Bandbreite des Gesamtvermögens", `${realNote()}, über alle Läufe`, chartBox("mc-fan", 300, true)
      + FAN_LEGEND)
    + card("Wahrscheinlichkeit, dass das Geld reicht", "Anteil der Läufe, in denen das Vermögen bis zu diesem Zeitpunkt reicht", chartBox("mc-surv", 260, true))
    + card("Verteilung des Endvermögens", realNote(), chartBox("mc-end", 260) + legend([["--text", "1., 5., 50. (Median), 95. und 99. Perzentil", null, "dashed"]]))
    + (st.atRetirement ? card("Verteilung des Vermögens bei Pensionierung", "nominal", chartBox("mc-ret", 240)) : "")
    + (st.depletionAges.length ? card("Alter bei Erschöpfung", "Anzahl Läufe, in denen das Vermögen in diesem Altersjahr aufgebraucht ist", chartBox("mc-dep", 220)) : "");
}
function drawBatchCharts(b) {
  const bands = realView && b.bands.real ? b.bands.real : b.bands, x = bands.months.map(ym2i), st = b.stats;
  Charts.time($("#mc-fan"), $("#mc-fan-tip"), { x, xFmt: xLabel, view: resultView(), onZoom: zoomAbs, yFmt: fmtMio,
    bands: [{ lo: bands.p5, hi: bands.p95, color: "--c1", opacity: 0.18 }, { lo: bands.p25, hi: bands.p75, color: "--c1", opacity: 0.35 }],
    lines: [{ color: "--c1", values: bands.p50 }, { color: "--warn", dash: "5 3", width: 1.5, values: bands.p99 }, { color: "--warn", dash: "5 3", width: 1.5, values: bands.p1 }, { color: "--err", width: 1.5, values: bands.min }],
    tip: i => `<b>${ymShow(bands.months[i])}</b> · Alter ${ageStr(x[i])}<br>99. ${fmtMoney(bands.p99[i])}<br>95. ${fmtMoney(bands.p95[i])}<br>75. ${fmtMoney(bands.p75[i])}<br><b>Median ${fmtMoney(bands.p50[i])}</b><br>25. ${fmtMoney(bands.p25[i])}<br>5. ${fmtMoney(bands.p5[i])}<br>1. ${fmtMoney(bands.p1[i])}<br><span style="color:var(--err)">Schlechtester ${fmtMoney(bands.min[i])}</span>` });
  const surv = st.survival.map(v => v * 100);
  // Achse auf den relevanten Bereich zoomen (mindestens 10 Prozentpunkte)
  const sMin = Math.min(...surv), sLo = Math.max(0, Math.min(90, Math.floor((sMin - 2) / 5) * 5));
  Charts.time($("#mc-surv"), $("#mc-surv-tip"), { x, xFmt: xLabel, view: resultView(), onZoom: zoomAbs, yFmt: v => `${v} %`, range: [sLo, 100],
    lines: [{ color: "--ok", values: surv }],
    tip: i => `<b>${ymShow(bands.months[i])}</b> · Alter ${ageStr(x[i])}<br>Geld reicht in <b>${surv[i].toLocaleString("de-CH", { maximumFractionDigits: 1 })} %</b> der Läufe` });
  const h = realView ? st.endTotalReal : st.endTotal;
  const marks = [[0.01, "1."], [0.05, "5."], [0.5, "Median"], [0.95, "95."], [0.99, "99."]].map(([q, l]) => ({ value: histQuantile(h, q), label: l }));
  Charts.histogram($("#mc-end"), $("#mc-end-tip"), h, { xFmt: fmtMio, marks });
  Charts.histogram($("#mc-ret"), $("#mc-ret-tip"), st.atRetirement, { xFmt: fmtMio, color: "--c2", marks: [{ value: histQuantile(st.atRetirement, 0.5), label: "Median" }] });
  const runs = b.summary.runs;
  Charts.bars($("#mc-dep"), $("#mc-dep-tip"), { cats: st.depletionAges.map(([a]) => String(a)), yFmt: v => fmtInt(v),
    series: [{ color: "--err", values: st.depletionAges.map(([, n]) => n) }],
    tip: i => `Alter <b>${st.depletionAges[i][0]}</b><br>${fmtInt(st.depletionAges[i][1])} Läufe (${pctTxt(st.depletionAges[i][1] / runs)})` });
}

// ---------- Szenarien ----------
const SWEEP_SHOW = { pct: v => `${fmtPct(v)} %`, month: v => ymShow(v) };
const sweepType = p => SWEEP_PARAMS.find(x => x.id === p)?.type || "pct";
const sweepLabel = p => SWEEP_PARAMS.find(x => x.id === p)?.label || p;
/** Aktueller Wert eines Variationsparameters in der Planung (undefined, wenn er nicht als Einzelwert vorliegt). */
function sweepNow(id) {
  const one = v => (v && typeof v === "object" ? (v.mode === "const" ? v.value : undefined) : v);
  if (id === "wealth.stockShare") return one(state.wealth.stockShare);
  if (id === "pension.pk.lumpsum") return one(state.pension.pk.lumpsum);
  if (id === "returns.startDate") return state.returns.startDate;
  if (id === "flows.spendingFactor") return 1;
  if (id === "withdrawal.rate") return state.withdrawal.fixedRate?.rate;
  if (id === "milestone.retire") { const m = state.milestones.find(x => x.id === "retire"); return m?.kind === "date" ? m.date : undefined; }
}
/** Die Werte der Variationen, die für diesen Lauf gelten (z. B. «Aktienanteil 60 %, Pensionierungsdatum 03.2027»). */
function variationText() {
  return state.scenarios.sweeps.map(s => {
    const v = sweepNow(s.param);
    return v == null ? null : `${sweepLabel(s.param)} ${SWEEP_SHOW[sweepType(s.param)](v)}`;
  }).filter(Boolean).join(", ");
}
/** Zufallspfade haben keine aussagekräftige Bezeichnung; beim Einzellauf stehen stattdessen die Werte der Variationen. */
const randomPath = label => /^(Bootstrap-Pfad|Zufallspfad) \d+$/.test(label || "");
const runInfo = label => randomPath(label) ? variationText() : label;
/** Optimaler Wert einer Variation für ein Perzentil, lesbar: «73 %» (Schritt), «≈ 73 %» (interpoliert), «< 50 %», «≥ 110 %». */
function optimumValueText(item) {
  if (item.kind === "na") return "–";
  const t = sweepType(item.param);
  const v = t === "pct" && typeof item.v === "number" ? Math.round(item.v * 1000) / 1000 : item.v;
  const s = SWEEP_SHOW[t](v);
  const pre = { exact: "", approx: "≈ ", below: "< ", atLeast: "≥ ", edgeLow: "≤ ", edgeHigh: "≥ " }[item.kind] ?? "";
  return pre + s + (item.reaches ? "" : " (reicht nirgends)");
}
/** Optimale Werte aller Variationen für das Perzentil (aus den Szenarien); leer ohne Variationen. */
function optimumText(sc, percent) {
  const o = Optimum.find(sc, percent);
  return o ? o.items.map(it => `${sweepLabel(it.param)} ${optimumValueText(it)}`).join(", ") : "";
}
// ----- Einzellauf mit Variationen: eine Variationsstufe wählen und ihren Lauf ansehen -----
let variationSel = null;          // gewählte Werte je Variation (null = Planwerte)
const variationRuns = new Map();  // Auswahl → Ergebnis des Einzellaufs dieser Stufe
const selKey = sel => JSON.stringify(sel);
const singleOf = r => (variationSel && variationRuns.get(selKey(variationSel))) || r.single;
const batchRuns = () => variationSel && variationRuns.get(selKey(variationSel));
const detailsOf = r => batchRuns()?.details || r.batch.details;
const resetVariation = () => { variationSel = null; variationRuns.clear(); };
/** Werte der Variationen für diesen Lauf: die gewählte Stufe, sonst die Planwerte. */
function selectedVariationText() {
  return state.scenarios.sweeps.map((s, i) => {
    const v = variationSel?.[i] ?? sweepNow(s.param);
    return v == null ? null : `${sweepLabel(s.param)} ${SWEEP_SHOW[sweepType(s.param)](v)}`;
  }).filter(Boolean).join(", ");
}
function variationPicker(r) {
  const sc = r.scenarios, o = Optimum.find(sc, 50);
  const selects = sc.params.map((p, i) => {
    const t = sweepType(p), now = sweepNow(p), cur = variationSel?.[i];
    const vals = [...new Set(sc.results.filter(x => !x.error).map(x => JSON.stringify(x.values[i])))].map(JSON.parse);
    return `<label class="field"><span>${esc(sweepLabel(p))}</span><select data-action="var-sel" data-i="${i}">
      <option value="plan" ${cur == null ? "selected" : ""}>Planwert${now != null ? ` (${SWEEP_SHOW[t](now)})` : ""}</option>
      ${vals.map(v => `<option value="${esc(JSON.stringify(v))}" ${JSON.stringify(v) === JSON.stringify(cur) ? "selected" : ""}>${SWEEP_SHOW[t](v)}</option>`).join("")}</select></label>`;
  }).join("");
  const opt = optimumText(sc, 50);
  const sub = r.single ? "Einzellauf: Grafiken, Kennzahlen und Jahreswerte gelten für die gewählte Stufe"
    : "Einzelne Läufe und Jahreswerte zeigen die Läufe der gewählten Stufe; die übrigen Reiter zeigen die Planwerte";
  return card("Variation wählen", sub,
    `<div class="grid3">${selects}</div>
    <p class="computed" style="margin:8px 0 0">${opt ? `Optimum: <b>${esc(opt)}</b> ` : ""}${o && o.items.every(it => it.grid != null) ? `<button class="btn small" data-action="var-opt" title="Den Lauf des optimalen Variationsschritts anzeigen">Optimum anzeigen</button>` : ""}
    ${variationSel ? `<button class="btn small" data-action="var-plan" title="Zurück zu den Planwerten">Planwerte</button>` : ""}</p>`);
}
const sameValue = (a, b) => typeof a === "number" && typeof b === "number" ? Math.abs(a - b) < 1e-9 : JSON.stringify(a) === JSON.stringify(b);
/** Berechnete Stufe zu einer Auswahl; nicht gewählte Variationen gelten mit ihrem Planwert. */
function levelOf(sc, sel) {
  const want = sc.params.map((p, i) => sel[i] ?? sweepNow(p));
  return sc.results.find(x => !x.error && x.values.every((v, i) => sameValue(v, want[i])));
}
/** Einzelne Läufe und Jahreswerte einer Stufe: die Läufe der Perzentile werden aus den gespeicherten Pfadnummern nachgerechnet. */
async function pickBatchVariation(sel) {
  const sc = lastRun.scenarios;
  variationSel = sel.every(v => v == null) ? null : sel;
  if (variationSel && !variationRuns.has(selKey(variationSel))) {
    const level = levelOf(sc, variationSel);
    if (!level || !level.percentiles?.length) {
      variationSel = null;
      toast("Diese Kombination wurde nicht berechnet; bitte eine Stufe der Variationen wählen.");
      return renderResults(lastRun);
    }
    const picks = level.percentiles.filter(p => [0, 1, 5, 50, 95, 99].includes(p.percent)).map(p => [p.percent, p.index]);
    const plan = JSON.parse(JSON.stringify(state));
    plan.scenarios = { sweeps: [], select: sc.params.map((p, i) => ({ param: p, value: variationSel[i] ?? sweepNow(p) })) };
    let out;
    try { out = await Engine.call("rerun_paths", { plan, picks, path: currentPath }); } catch (e) { out = { error: errText(e) }; }
    if (out.error) { variationSel = null; toast("Die Läufe der Variation konnten nicht gerechnet werden: " + out.error); return renderResults(lastRun); }
    // Weicht ein Endvermögen vom gespeicherten Wert ab (z. B. geänderte Zeitreihe), bleibt die Anzeige verfügbar, mit Hinweis
    const off = out.details.filter(([pc, res]) => {
      const e = level.percentiles.find(p => p.percent === pc)?.endTotal;
      return e != null && Math.abs(res.summary.endTotal - e) > 1e-6 * Math.max(1, Math.abs(e));
    }).map(([pc]) => pc);
    variationRuns.set(selKey(variationSel), { details: out.details, off });
  }
  renderResults(lastRun);
}
async function pickVariation(sel) {
  const sc = lastRun?.scenarios;
  if (sc && lastRun.batch) return pickBatchVariation(sel);
  if (!sc || !lastRun.single) return;
  variationSel = sel.every(v => v == null) ? null : sel;
  if (variationSel && !variationRuns.has(selKey(variationSel))) {
    const plan = JSON.parse(JSON.stringify(state));
    plan.scenarios = { sweeps: [], select: sc.params.map((p, i) => (variationSel[i] == null ? null : { param: p, value: variationSel[i] })).filter(Boolean) };
    const out = await Engine.call("run_simulation", { plan, path: currentPath });
    if (out.error || !out.single) { variationSel = null; toast("Die Variation konnte nicht gerechnet werden: " + (out.error || "kein Ergebnis")); }
    else variationRuns.set(selKey(variationSel), out.single);
  }
  renderResults(lastRun);
}
document.addEventListener("change", e => {
  if (!e.target.matches?.('select[data-action="var-sel"]')) return;
  pickVariation([...document.querySelectorAll('select[data-action="var-sel"]')].map(el => (el.value === "plan" ? null : JSON.parse(el.value))));
});
/** Zeile zu einem Perzentil: optimale Variationswerte, bei historischen Läufen zusätzlich der Startmonat. */
const percentInfo = (r, percent, label) => [optimumText(r.scenarios, percent), randomPath(label) ? "" : label].filter(Boolean).join(" · ");
function scenarioTable(sc) {
  const types = sc.params.map(sweepType);
  const best = Math.max(...sc.results.map(r => r.error ? -1 : r.successRate));
  const single = sc.results.every(r => r.runs <= 1);
  const rows = sc.results.map(r => {
    const vals = r.values.map((v, i) => `<td class="l">${SWEEP_SHOW[types[i]](v)}</td>`).join("");
    if (r.error) return `<tr>${vals}<td class="l" colspan="6" style="color:var(--err)">${esc(r.error)}</td></tr>`;
    const ok = r.successRate, cls = ok >= 0.95 ? "good" : ok < 0.8 ? "bad" : "";
    return `<tr class="${r.successRate === best ? "best" : ""}">${vals}
      <td class="${cls}">${single ? (ok ? "✓" : "✗") : pctTxt(ok)}</td>
      <td>${r.atRetirement != null ? fmtInt(r.atRetirement) : "–"}</td>
      ${single ? `<td colspan="3">${fmtInt(realView ? r.p50Real : r.p50)}</td>` : `<td>${fmtInt(r.p5)}</td><td><b>${fmtInt(realView ? r.p50Real : r.p50)}</b></td><td>${fmtInt(r.p95)}</td>`}
      <td>${r.earliestDepletion ? `Alter ${ageStr(ym2i(r.earliestDepletion))}` : "–"}</td></tr>`;
  }).join("");
  const heads = sc.params.map(p => `<th class="l">${esc(sweepLabel(p))}</th>`).join("");
  return card("Szenarien", `${fmtInt(sc.results.length)} Kombinationen × ${fmtInt(sc.results[0]?.runs || 0)} ${single ? "Lauf" : "Läufe"}`,
    `<div class="tscroll"><table class="tbl num sctbl"><tr>${heads}<th>${single ? "Reicht" : "Erfolgsquote"}</th><th>Vermögen bei Pensionierung${single ? "" : " (Median)"}</th>${single ? `<th colspan="3">Endvermögen</th>` : `<th>Ende 5. Perz.</th><th>Ende Median${realView ? " (heutige Kaufkraft)" : ""}</th><th>Ende 95. Perz.</th>`}<th>Früheste Erschöpfung</th></tr>${rows}</table></div>
    <p class="hint">Beträge nominal in ${cur()}${realView ? ", Median am Ende in heutiger Kaufkraft" : ""}. Hervorgehoben: höchste Erfolgsquote. Alle Szenarien verwenden dieselben Renditepfade.</p>`);
}
const sweepNames = sc => sc.params.map(p => esc(sweepLabel(p))).join(" × ");
function scenarioChartsHtml(sc) {
  const ok = sc.results.filter(r => !r.error);
  if (!ok.length) return "";
  const variation = `<div class="card"><p style="margin:0"><b>Dargestellte ${sc.params.length === 1 ? "Variation" : "Variationen"}:</b> ${sweepNames(sc)}</p></div>`;
  if (sc.params.length === 1) {
    const single = ok.every(r => r.runs <= 1);
    return variation + (single ? "" : card("Erfolgsquote je Variante", `Variation: ${esc(sweepLabel(sc.params[0]))}`, chartBox("sc-succ", 240)))
      + card("Endvermögen je Variante", `Variation: ${esc(sweepLabel(sc.params[0]))} · ${realView ? "Median in heutiger Kaufkraft" : "nominal, Median mit 5.–95. Perzentil"}`, chartBox("sc-end", 280))
      + card("Vermögen bei Pensionierung je Variante", `Variation: ${esc(sweepLabel(sc.params[0]))} · Median, nominal`, chartBox("sc-ret", 220));
  }
  if (sc.params.length === 2) return variation + card("Erfolgsquote", `Variationen: ${sweepNames(sc)}`, heat(sc, r => r.successRate, pctTxt, "high"))
    + card("Endvermögen (Median)", `Variationen: ${sweepNames(sc)} · ${realNote()}`, heat(sc, r => realView ? r.p50Real : r.p50, fmtMio, "high"));
  return card("Grafiken", "", `<p class="hint" style="margin:0">Grafiken gibt es für eine oder zwei Variationen; bei ${sc.params.length} Variationen siehe Tabelle.</p>`);
}
function heat(sc, f, fmt, good, range) {
  const [p1, p2] = sc.params, t1 = sweepType(p1), t2 = sweepType(p2);
  const xs = [...new Set(sc.results.map(r => JSON.stringify(r.values[0])))].map(JSON.parse);
  const ys = [...new Set(sc.results.map(r => JSON.stringify(r.values[1])))].map(JSON.parse);
  const find = (xi, yi) => sc.results.find(r => JSON.stringify(r.values[0]) === JSON.stringify(xs[xi]) && JSON.stringify(r.values[1]) === JSON.stringify(ys[yi]));
  return Charts.heatmap({ xs: xs.map(SWEEP_SHOW[t1]), ys: ys.map(SWEEP_SHOW[t2]), xLabel: sweepLabel(p1), yLabel: sweepLabel(p2),
    value: (xi, yi) => { const r = find(xi, yi); return r && !r.error ? f(r) : NaN; }, fmt, good, lo: range?.[0], hi: range?.[1] });
}
function drawScenarioCharts(sc) {
  if (sc.params.length !== 1) return;
  const ok = sc.results.filter(r => !r.error), xs = ok.map(r => SWEEP_SHOW[sweepType(sc.params[0])](r.values[0]));
  const head = i => `<b>${esc(sweepLabel(sc.params[0]))}: ${xs[i]}</b><br>`;
  Charts.category($("#sc-succ"), $("#sc-succ-tip"), { xs, yFmt: v => `${v} %`, range: [Math.max(0, Math.min(90, Math.floor((Math.min(...ok.map(r => r.successRate * 100)) - 2) / 5) * 5)), 100],
    lines: [{ color: "--ok", values: ok.map(r => r.successRate * 100) }], tip: i => head(i) + `Erfolgsquote ${pctTxt(ok[i].successRate)}` });
  Charts.category($("#sc-end"), $("#sc-end-tip"), { xs, yFmt: fmtMio,
    bands: realView ? [] : [{ lo: ok.map(r => r.p5), hi: ok.map(r => r.p95), color: "--c1", opacity: 0.18 }],
    lines: [{ color: "--c1", values: ok.map(r => realView ? r.p50Real : r.p50) }],
    tip: i => head(i) + (realView ? `Median ${fmtMoney(ok[i].p50Real)}` : `95. ${fmtMoney(ok[i].p95)}<br><b>Median ${fmtMoney(ok[i].p50)}</b><br>5. ${fmtMoney(ok[i].p5)}`) });
  Charts.category($("#sc-ret"), $("#sc-ret-tip"), { xs, yFmt: fmtMio, lines: [{ color: "--c2", values: ok.map(r => r.atRetirement ?? 0) }],
    tip: i => head(i) + `Vermögen bei Pensionierung ${fmtMoney(ok[i].atRetirement ?? 0)}` });
}

// ---------- Aufbau ----------
/**
 * FEA-56: Einordnung der Erfolgsquote mit den Grenzen der Ampel. Massgebend ist der angezeigte, auf 0,1 % gerundete Wert:
 * 95,0 % ist «hoch», 80,0 % «mittel».
 */
function rateClass(rate) {
  const shown = Number((rate * 100).toFixed(1));
  return shown >= 95 ? "hoch" : shown >= 80 ? "mittel" : "tief";
}
const helpLink = (topic, text, title = "") => `<a data-action="help-topic" data-topic="${topic}"${title ? ` title="${esc(title)}"` : ""}>${text}</a>`;
/** FEA-56: Ergebnis in Worten über den Kennzahlen der Übersicht, gebildet aus denselben Werten wie die Kennzahlen. */
function verdictHtml(r, sg) {
  const lines = [];
  let weak;
  if (sg) {
    const s = sg.summary;
    weak = !!s.depleted;
    lines.push(weak ? `Das Vermögen ist im ${ymShow(s.depleted)} (Alter ${ageStr(ym2i(s.depleted))}) aufgebraucht.`
      : `Das Vermögen reicht bis zum Ende der Planung und beträgt dann <b>${fmtMoney(realView ? s.endTotalReal : s.endTotal)}</b>${realView ? " in heutiger Kaufkraft" : ""}.`);
  } else {
    const s = r.batch.summary, ed = s.earliestDepletion, end = endM(), cls = rateClass(s.successRate);
    weak = cls !== "hoch";
    lines.push(`In <b>${pctTxt(s.successRate)}</b> (${helpLink("r-overview", cls, "Wie viel Erfolgsquote ist genug?")}) der Läufe reicht das Vermögen bis ${ymShow(i2ym(end))} (Alter ${ageStr(end)}).`);
    if (ed) lines.push(`Im ungünstigsten Lauf ist das Geld mit Alter ${ageStr(ym2i(ed[0]))} aufgebraucht.`);
  }
  const next = weak
    ? `Nächster Schritt: Die ${helpLink("s-optimize", "Optimierung")} sucht eine Entnahmestrategie, die besser zu Ihren Zielen passt.`
    : `Nächster Schritt: Mit ${helpLink("s-scenarios", "Szenarien")} prüfen Sie, wie stark das Ergebnis von Ausgaben, Aktienanteil oder Kapitalbezug abhängt.`;
  return `<div class="card verdict" id="verdict"><p>${lines.join(" ")}</p><p class="next">${next}</p></div>`;
}
function renderResults(r) {
  compareOpen = false;
  const info = `${r.elapsedMs < 1 ? r.elapsedMs.toFixed(2) : r.elapsedMs.toFixed(0)} ms`;
  if (r.error) {
    $("#resultsTitle").textContent = "Simulation nicht möglich"; $("#resultsInfo").textContent = ""; $("#resultsTabs").innerHTML = "";
    $("#resultsBody").innerHTML = `<div class="card errcard"><p style="margin-top:0">${esc(r.error)}</p>
      <ul class="val">${r.issues.filter(i => i.severity === "error").map(i => `<li class="err" data-action="back"><span class="ic">!</span><span>${esc(i.message)}</span></li>`).join("")}</ul></div>`;
    return;
  }
  const tabs = [["overview", "Übersicht"], ["charts", "Grafiken"]];
  if (r.batch) tabs.push(["runs", "Einzelne Läufe"]);
  if (r.scenarios) tabs.push(["scenarios", "Szenarien"]);
  tabs.push(["table", "Jahreswerte"], ["params", "Parameter"]);
  if (!tabs.some(([k]) => k === resultTab)) resultTab = "overview";
  // Einzellauf mit Variationen: gewählte Stufe (sonst der Planlauf)
  const sg = r.single ? singleOf(r) : null;
  $("#resultsTitle").textContent = sg ? ["Einzellauf", runInfo(sg.label), r.scenarios ? selectedVariationText() : ""].filter(Boolean).join(" · ") : `${fmtInt(r.batch.summary.runs)} Läufe`;
  const varied = r.batch && r.scenarios && !!variationSel;
  $("#resultsPick").innerHTML = r.single && r.scenarios ? variationPicker(r)
    : r.batch && r.scenarios && (resultTab === "runs" || resultTab === "table") ? variationPicker(r)
    : varied && resultTab !== "params" ? `<div class="card variednote"><span class="muted">Planwerte, nicht die gewählte Variation (${esc(selectedVariationText())}). Die gewählte Stufe zeigen «Einzelne Läufe» und «Jahreswerte».</span></div>` : "";
  $("#resultsInfo").textContent = sg ? `1 Lauf · ${info}` : info;
  $("#resultsTabs").innerHTML = `<div class="seg tabs">${tabs.map(([k, t], i) => `<button class="${k === resultTab ? "on" : ""}" data-action="rtab" data-tab="${k}" title="${t} (${MOD}${i + 1})">${t}</button>`).join("")}</div>
    <div class="seg"><button class="${realView ? "" : "on"}" data-action="real" data-v="0" title="Beträge wie sie dann anfallen">Nominal</button><button class="${realView ? "on" : ""}" data-action="real" data-v="1" title="Beträge in Preisen des Simulationsstarts (um die Teuerung bereinigt)">Heutige Kaufkraft</button></div>`;
  const body = $("#resultsBody");
  const detailRun = () => sg || (r.batch && (detailsOf(r).find(([q]) => q === detailP)?.[1] || detailsOf(r)[0][1]));
  if (resultTab === "overview") {
    if (sg) {
      const res = sg;
      body.innerHTML = verdictHtml(r, res) + singleKpis(res.summary) + card("Vermögensverlauf", realNote(), chartBox("ov-wealth", 300, true) + WEALTH_LEGEND) + propertyEvents(res) + card("Ereignisse", "", eventList(res.events));
      drawRunCharts({ ...res, records: res.records }, "ov");
    } else {
      const b = r.batch, s = b.summary, pc = q => s.percentiles.find(x => x.percent === q), ed = s.earliestDepletion;
      body.innerHTML = verdictHtml(r, null) + `<div class="rkpis">
        ${kpi("Erfolgsquote", pctTxt(s.successRate), "Vermögen reicht bis zum Ende", s.successRate >= 0.95 ? "good" : s.successRate < 0.8 ? "bad" : "")}
        ${kpi("Endvermögen Median", fmtMio(pc(50).endTotal), esc(percentInfo(r, 50, pc(50).label)))}
        ${kpi("Endvermögen 5. Perzentil", fmtMio(pc(5).endTotal), esc(percentInfo(r, 5, pc(5).label)), pc(5).endTotal > 0 ? "" : "bad")}
        ${kpi("Endvermögen 1. Perzentil", fmtMio(pc(1).endTotal), esc(percentInfo(r, 1, pc(1).label)), pc(1).endTotal > 0 ? "" : "bad")}
        ${kpi("Schlechtester Lauf", fmtMio(pc(0).endTotal), esc(percentInfo(r, 0, pc(0).label)), pc(0).endTotal > 0 ? "" : "bad")}
        ${kpi("Endvermögen 95. Perzentil", fmtMio(pc(95).endTotal), esc(percentInfo(r, 95, pc(95).label)))}
        ${kpi("Endvermögen 99. Perzentil", fmtMio(pc(99).endTotal), esc(percentInfo(r, 99, pc(99).label)))}
        ${kpi("Früheste Erschöpfung", ed ? "Alter " + ageStr(ym2i(ed[0])) : "keine", ed ? `${ymShow(ed[0])} · ${esc(ed[1])}` : "in keinem Lauf", ed ? "bad" : "good")}
        ${hasProperties() ? kpi("Notverkäufe", `${fmtInt(s.emergencySaleRuns)} Läufe`, `${pctTxt(s.emergencySaleRuns / s.runs)} der Läufe${s.affordabilitySaleRuns ? `, davon ${fmtInt(s.affordabilitySaleRuns)} wegen fehlender Tragbarkeit bei der Erneuerung` : ""}`, s.emergencySaleRuns > 0 ? "bad" : "good") : ""}</div>`
        + card("Bandbreite des Gesamtvermögens", `${realNote()}, über alle Läufe`, chartBox("mc-fan", 300, true) + FAN_LEGEND)
        + card("Endvermögen nach Perzentil", "nominal", `<table class="tbl num"><tr><th class="l">Perzentil</th><th>Endvermögen</th>${hasProperties() ? `<th class="l" title="Alle Verkäufe von Immobilien in diesem Lauf mit Monat; Notverkäufe (Notverkaufskriterium oder fehlende Tragbarkeit bei einer Hypothekenerneuerung) tragen das Warnsymbol">Immobilienverkäufe</th>` : ""}<th class="l">Variation</th></tr>
          ${s.percentiles.map(p => `<tr><td class="l">${p.percent === 0 ? "Schlechtester Lauf" : p.percent + "."}</td><td>${fmtInt(p.endTotal)}</td>${hasProperties() ? `<td class="l">${propertySales(b.details.find(([q]) => q === p.percent)?.[1])}</td>` : ""}<td class="l">${esc(percentInfo(r, p.percent, p.label))}</td></tr>`).join("")}</table>
          ${r.scenarios ? `<p class="hint">Variation: optimaler Wert je Perzentil aus den Szenarien. «≈» ist zwischen zwei Schritten interpoliert, «&lt;» und «≥» zeigen, dass der Bereich der Variation nicht reicht bzw. ausgeschöpft ist; bei einem Anteil steht 0 % oder 100 % ohne Zeichen, denn mehr oder weniger gibt es nicht. Ausgaben-Faktor: höchster Wert, bei dem das Vermögen bis zum Ende reicht; Aktienanteil und Kapitalbezug: tragbarer Wert mit dem höchsten Endvermögen.</p>` : ""}`);
      drawBatchCharts(b);
    }
  } else if (resultTab === "charts") {
    if (sg) { body.innerHTML = runChartsHtml(sg, "rc"); drawRunCharts(sg, "rc"); }
    else { body.innerHTML = batchChartsHtml(r.batch); drawBatchCharts(r.batch); }
  } else if (resultTab === "runs") {
    const res = detailRun();
    $("#resultsPick").innerHTML = (r.scenarios ? variationPicker(r) : "") + runPicker(r, res);
    body.innerHTML = runChartsHtml(res, "dr") + propertyEvents(res) + card("Ereignisse", "", eventList(res.events));
    drawRunCharts(res, "dr");
  } else if (resultTab === "scenarios") {
    body.innerHTML = scenarioTable(r.scenarios) + scenarioChartsHtml(r.scenarios);
    drawScenarioCharts(r.scenarios);
  } else if (resultTab === "params") {
    body.innerHTML = paramsHtml(lastRunPlan);
  } else {
    const res = detailRun();
    if (r.batch) $("#resultsPick").innerHTML = (r.scenarios ? variationPicker(r) : "") + runPicker(r, res);
    const info = r.batch ? [pName(detailP), percentInfo(r, detailP, res.label)].filter(Boolean).join(" · ") : "";
    body.innerHTML = card("Jahreswerte", `${realNote()}, Beträge in ${cur()}${info ? ` · ${esc(info)}` : ""}`, yearTable(res.records, { total: res.summary.startTotal, liquid: res.summary.startLiquid }, res.bucketRefills || []), "yearcard");
  }
  if (savedView) $("#resultsPick").insertAdjacentHTML("afterbegin", `<div class="card savedbar"><span>Gespeichertes Ergebnis vom <b>${esc(runDate(savedView.created))}</b> · ${esc(savedView.name)}. Es gilt die Planung von damals.</span><span class="spacer"></span>
    <button class="btn small" data-action="runs-open">← Zur Liste</button><button class="btn small primary" data-action="run-restore" title="Die Planung dieses Ergebnisses in den Editor laden">Planung in den Editor übernehmen</button></div>`);
}
/**
 * Parameter einer Planung als Abschnitte [Titel, [[Bezeichnung, Wert], …]], gegliedert wie die Editorabschnitte.
 * Personennamen fehlen absichtlich (nur der Geburtsmonat), technische Werte nur, soweit sie das Ergebnis reproduzierbar machen.
 */
function paramSections(plan) {
  const c = plan.project?.currency || "";
  const money = v => v == null ? "–" : `${fmtInt(v)} ${c}`, pc = v => v == null ? "–" : `${fmtPct(v)} %`, num = v => v == null ? "–" : String(v);
  const yn = v => v ? "ja" : "nein";
  const msName = id => plan.milestones?.find(m => m.id === id)?.name;
  const at = r => isYM(r) ? ymShow(r) : (msName(r) ?? String(r ?? "–"));
  const ser = (v, f) => !v || typeof v !== "object" ? (v == null ? "–" : f(v))
    : v.mode === "const" ? f(v.value) : (v.rows || []).map(r => `ab ${at(r.from)}: ${f(r.value)}`).join("; ") || "–";
  const table = rows => (rows || []).map(([x, r]) => `ab ${fmtInt(x)}: ${fmtPct(r)} %`).join("; ") || "–";
  const out = [];
  const persons = plan.household?.persons || [];
  out.push(["Haushalt", [["Währung", c], ...persons.map((p, i) => [`Person ${i + 1}: Geburt`, ymShow(p.birth)]),
    ...(persons.length > 1 ? [["Verheiratet", yn(plan.household?.married)], ...(plan.household?.married ? [["AHV-Renten zusammen höchstens, pro Monat", money(plan.household.ahvCoupleCap)]] : [])] : [])]]);
  out.push(["Meilensteine", (plan.milestones || []).map(m => [m.name, m.kind === "date" ? ymShow(m.date) : `Alter ${m.years} J. (Person ${Math.max(1, persons.findIndex(p => p.id === m.person) + 1)})`])]);
  const pens = (pre, o, ahv) => [
    [`${pre}Pensionskasse: Kapital`, money(o.pk?.capital)], [`${pre}Pensionskasse: Bezug ab`, at(o.pk?.at)], [`${pre}Kapitalbezug`, pc(o.pk?.lumpsum)],
    [`${pre}Umwandlungssatz`, pc(o.pk?.conversion)], [`${pre}PK-Beitrag pro Monat`, ser(o.pk?.contribution, money)], [`${pre}PK-Zins`, ser(o.pk?.interest, pc)],
    [`${pre}AHV pro Monat`, money(ahv?.monthly)], [`${pre}AHV ab`, at(ahv?.at)], [`${pre}AHV indexiert`, yn(ahv?.indexed)]];
  const part = plan.partner;
  out.push(["Vorsorge", [...pens(part ? "Person 1: " : "", plan.pension || {}, plan.pension?.ahv),
    ...(part ? pens("Person 2: ", part, part.ahv) : [])]]);
  out.push(["Einnahmen und Ausgaben", [["Einkommen pro Monat", ser(plan.flows?.income, money)], ...(part ? [["Einkommen Person 2 pro Monat", ser(part.income, money)]] : []),
    ["Ausgaben pro Monat", ser(plan.flows?.spending, money)],
    ...(plan.flows?.extras || []).map(e => [`Einmalbetrag ${ymShow(e.date)}${e.label ? " " + e.label : ""}`, money(e.amount)])]]);
  const w = plan.wealth || {};
  out.push(["Vermögen und Anlage", [["Aktien", money(w.stocks)], ["Obligationen", money(w.bonds)], ["Cash", money(w.cash)],
    ["Aktienanteil", ser(w.stockShare, pc)]]]);
  const r = plan.returns || {}, f = r.fixed || {};
  const src = r.source;
  const rrows = [["Renditequelle", sourceName(src)]];
  if (src === "fixed") rrows.push(["Aktien pro Jahr", pc(f.stocks)], ["Obligationen pro Jahr", pc(f.bonds)], ["Inflation pro Jahr", pc(f.inflation)], ["SARON", pc(f.saron)]);
  else {
    if (r.series && src !== "parametric") rrows.push(["Zeitreihe", seriesShort(r.series)]);
    else if (r.strategy && src !== "parametric") {
      const name = (list, k) => list.find(x => x[0] === k)?.[1] ?? k, st = r.strategy, wt = st.weights || {};
      rrows.push(["Aktien (Anlagestrategie)", name(STOCK_INDICES, st.stocks)], ["Obligationen (Anlagestrategie)", name(BOND_INDICES, st.bonds)]);
      if (st.stocks === "worldCustom") rrows.push(["Gewichte", `Schweiz ${pc(wt.ch)}, USA ${pc(wt.usa)}, EU ${pc(wt.eu)}, Asien-Pazifik ${pc(wt.apac)}`]);
    }
    if (src === "historical") rrows.push(["Historischer Startmonat", ymShow(r.startDate)]);
    if (src === "bootstrap") rrows.push(["Pfade", num(r.bootstrap?.paths)], ["Blocklänge (Monate)", num(r.bootstrap?.block)], ["Zufallsstartwert", num(r.bootstrap?.seed)]);
    if (src === "parametric") { const q = r.parametric || {};
      rrows.push(["Pfade", num(q.paths)], ["Parameter", q.auto ? "Vorschlagswerte des Aktienindex" : "eigene Werte"], ["Erwartete Aktienrendite", pc(q.mu)], ["Schwankung", pc(q.sigma)], ["Freiheitsgrade", num(q.df)], ["Kurssprünge pro Jahr", num(q.jumpLambda)], ["Sprunghöhe (Mittel)", pc(q.jumpMu)], ["Sprunghöhe (Streuung)", pc(q.jumpSigma)], ["Zufallsstartwert", num(q.seed)]);
      if (r.strategy) { const st = r.strategy, name = (list, k) => list.find(x => x[0] === k)?.[1] ?? k;
        rrows.push(["Aktien (Anlagestrategie)", name(STOCK_INDICES, st.stocks)], ["Obligationen und Inflation", `zufällig, korreliert gemäss ${name(BOND_INDICES, st.bonds)}`]); }
      else rrows.push(["Obligationen pro Jahr", pc(f.bonds)], ["Inflation pro Jahr", pc(f.inflation)]); }
    if (r.extrapolation) rrows.push(["Fortschreibung", `${r.extrapolation.method ?? "–"}${r.extrapolation.range ? ` (${r.extrapolation.range} J.)` : ""}`]);
  }
  out.push(["Renditequelle und Zeitreihe", rrows]);
  out.push(["Entnahmestrategie", [["Strategie", stratName(stratKey(plan.withdrawal))], ...stratParams(plan)]]);
  const props = plan.realestate?.properties || [];
  const STATUS = { owned: "im Besitz", planned: "geplant", rent: "Miete" };
  out.push(["Immobilien", [["Notverkauf bei Vermögen unter (Monate Ausgaben)", ser(plan.realestate?.emergencyMonths, num)], ...props.flatMap(p => {
    const nm = p.name || "Immobilie";
    return [[`${nm}: Status`, STATUS[p.status] || p.status], ...(p.price != null ? [[`${nm}: Kaufpreis`, money(p.price)]] : []), ...(p.worth != null ? [[`${nm}: Marktwert`, money(p.worth)]] : []),
      ...(p.buy ? [[`${nm}: Kauf`, at(p.buy)]] : []), ...(p.sell ? [[`${nm}: Verkauf`, at(p.sell)]] : []),
      ...(p.fixCosts != null ? [[`${nm}: Nebenkosten pro Monat`, money(p.fixCosts)]] : []), ...(p.rent != null ? [[`${nm}: Miete pro Monat`, money(p.rent)]] : []),
      ...(p.mortgage?.tranches || []).map(t => [`${nm}: ${t.name || "Tranche"}`, `${money(t.amount)}${t.start ? ` ab ${ymShow(t.start)}` : ""}; ${(t.periods || []).map(d => `${d.type === "saron" ? "SARON" : "fix"} ${num(d.term)} J., Zins ${pc(d.rate)}, Marge ${pc(d.margin)}, Amortisation ${money(d.amort)} (${d.amortMode === "indirect" ? "indirekt" : "direkt"})`).join(" → ")}`]),
    ];
  })]]);
  out.push(["Säule 3a", (plan.pillar3a || []).flatMap(a => {
    const nm = (plan.household?.persons || []).find(q => q.id === a.person)?.name || a.person;
    return [[`${nm}: Guthaben, Zins, Einzahlung pro Jahr`, `${money(a.balance)}, ${pc(a.rate)}, ${money(a.contribution ?? 0)}`],
      ...(a.payouts || []).map(x => [`${nm}: Auszahlung ${x.kind === "age" ? `mit ${x.years} J.` : ymShow(x.date)}`, `${x.amount != null ? money(x.amount) : "ganzes Guthaben"} (${x.use === "repay" ? "Hypothek" : "Vermögen"})`])];
  })]);
  const t = plan.taxes || {}, g = t.propertyGain;
  out.push(["Steuerannahmen", [["Steuerfuss", num(t.multiplier)], ["Einkommenssteuer", table(t.income)], ["Vermögenssteuer", table(t.wealth)], ["Kapitalbezugssteuer", table(t.capital)],
    ["Grundstückgewinnsteuer", g ? `${g.model === "aargau" ? "Modell Aargau" : "Stufen"}, Ermässigung ${pc(g.reduction ?? 0)}${g.exemptBelow ? `, steuerfrei unter ${money(g.exemptBelow)}` : ""}` : "keine"]]]);
  out.push(["Variationen", (plan.scenarios?.sweeps || []).length ? plan.scenarios.sweeps.map(sw => [sweepLabel(sw.param), `${SWEEP_SHOW[sweepType(sw.param)]?.(sw.from) ?? sw.from} bis ${SWEEP_SHOW[sweepType(sw.param)]?.(sw.to) ?? sw.to}, Schritt ${sw.step}`]) : [["Variationen", "keine"]]]);
  return out;
}
/** Reiter «Parameter»: schreibgeschützte Zusammenfassung der Planung im Stand des Laufs. */
function paramsHtml(plan) {
  if (!plan) return card("Parameter", "", `<p style="margin:0">Nicht verfügbar.</p>`);
  return `<p class="hint" style="margin:0 0 10px">Die Parameter im Stand dieses Laufs (nicht der aktuelle Stand im Editor). Nur zur Ansicht.</p>`
    + paramSections(plan).map(([title, rows]) => card(title, "", `<table class="tbl params">${rows.map(([k, v]) => `<tr><td class="l">${esc(k)}</td><td class="l">${esc(v)}</td></tr>`).join("")}</table>`)).join("");
}
/** Auswahl des Laufs (Perzentil) samt den Werten der Variationen; steht oberhalb des scrollenden Bereichs. */
function runPicker(r, res) {
  const info = [pName(detailP), percentInfo(r, detailP, res.label)].filter(Boolean).join(" · ");
  return card("Lauf wählen", "", `<div class="seg">${detailsOf(r).map(([p]) => `<button class="${p === detailP ? "on" : ""}" data-action="detail" data-p="${p}">${pName(p)}</button>`).join("")}</div>
    ${batchRuns()?.off?.length ? `<p class="hint" style="color:var(--warn,#b45309)">Hinweis: Das Endvermögen von ${batchRuns().off.map(pName).join(", ")} weicht von der Szenarientabelle ab (zum Beispiel, weil sich eine Zeitreihe geändert hat). Die Anzeige bleibt verfügbar.</p>` : ""}
    <p class="computed" style="margin-bottom:0"><b>${esc(info)}</b> · Endvermögen ${fmtMoney(res.summary.endTotal)}${res.summary.depleted ? ` · aufgebraucht ${atAge(res.summary.depleted)}` : ""}</p>`);
}
function showDetail(p) { detailP = p; bucketOpen.clear(); if (lastRun) renderResults(lastRun); }

// ---------- Gespeicherte Simulationen ----------
let savedView = null, parkedState = null;
const runsSel = new Set(), RUN_COLORS = ["--c1", "--c2", "--c3", "--warn", "--ok", "--err"];
const runDate = iso => new Date(iso).toLocaleString("de-CH", { dateStyle: "short", timeStyle: "short" });
const stratName = id => id === "three_buckets_guardrails" ? "3-Topf + Guardrails" : STRATS.find(x => x[0] === id)?.[1] || id;
const sourceName = id => SOURCES.find(x => x[0] === id)?.[1] || id;
/** Beim Ansehen einer gespeicherten Simulation gilt deren Planung; der Editor-Stand wird solange zurückgestellt. */
function unpark() { if (parkedState) { state = parkedState; parkedState = null; } savedView = null; }
/** Wert einer Zeitreihe der Planung als Text: konstant → Wert, sonst «zeitabhängig». */
const serTxt = (v, f) => v && typeof v === "object" ? (v.mode === "const" ? f(v.value) : "zeitabhängig") : v == null ? "–" : f(v);
/** Parameter der Entnahmestrategie einer Planung als Paare [Bezeichnung, Wert] (Bezeichnungen und Einheiten wie im Editor). */
function stratParams(plan) {
  const w = plan.withdrawal || {}, ws = plan.wealth || {}, pc = v => `${fmtPct(v)} %`, yrs = v => `${v} J.`;
  const share = (label = "Aktienanteil") => [label, serTxt(ws.stockShare, pc)];
  const limit = v => v ? pc(v) : "keine";
  const g = w.guardrails || {};
  const guard = () => [["Startrate bei Pensionierung", pc(g.initialRate)], ["Obere Leitplanke", pc(g.upper)], ["Untere Leitplanke", pc(g.lower)], ["Anpassung bei Leitplanke", pc(g.adjust)], ["Untergrenze", limit(w.floor)], ["Obergrenze", limit(w.ceiling)]];
  switch (w.strategy) {
    case "three_buckets": { const b = w.buckets || {};
      return [["Topf 1 · Cash", yrs(b.cashYears)], ["Topf 2 · Obligationen", yrs(b.bondYears)], ["Aus Topf 3 nur verkaufen, wenn Aktienrendite (12 Monate) ≥", pc(b.refillThreshold ?? 0)], ...(ws.stockShare != null ? [share("Aktienanteil Restvermögen")] : []),
        ...(bucketsGuardrails(w) ? guard() : [])]; }
    case "guardrails": return [...guard(), share()];
    case "fixed_rate": return [["Entnahmerate pro Jahr", pc((w.fixedRate || {}).rate)], share()];
    case "vpw": { const v = w.vpw || {}; return [["Vermögen soll reichen bis Alter", `${v.horizonAge} J.`], ["Angenommene Realrendite pro Jahr", pc(v.realReturn)], share()]; }
    case "endowment": { const e = w.endowment || {}; return [["Zielrate des Vermögens", pc(e.rate)], ["Gewicht des aktuellen Vermögens", pc(e.smoothing)], share()]; }
    default: return [share(), ["Cash-Reserve", serTxt(ws.reserveMonths, v => `${v} Monate`)]];
  }
}
/** Name der verwendeten Zeitreihe (ohne Ordner und Endung); `null`, wenn die Renditequelle keine Zeitreihe hat. */
function seriesLabel() {
  const r = state.returns;
  if (r.source === "fixed" || r.source === "parametric" || !seriesRef()) return null;
  return (seriesInfo && !seriesInfo.error && (seriesInfo.title || seriesInfo.name)) || (r.series ? seriesShort(r.series) : null);
}
/** Anzeige der Spalte «Zeitreihe» (Kopfdaten ohne Angabe: «–»). */
const seriesCell = m => m.seriesName === undefined ? "–" : m.seriesName ? esc(m.seriesName)
  : m.source === "fixed" ? `<span class="muted">feste Renditen</span>` : m.source === "parametric" ? `<span class="muted">parametrisch</span>` : "–";
/** Anteil der Läufe mit Notverkauf, ab dem das Warnsymbol erscheint (später einstellbar, FEA-24). */
const EMERGENCY_WARN_SHARE = 0.05;
/** Warnsymbol (Haus mit Warnzeichen) für den ungeplanten Verkauf einer Immobilie; `tip` ist der Tooltip. */
const houseWarn = tip => `<span class="hwarn" role="img" aria-label="${esc(tip)}" title="${esc(tip)}">🏠⚠️</span>`;
/** Symbol in der Liste der Ergebnisse: Einzellauf mit Notverkauf, Monte Carlo ab der Schwelle; alte Einträge ohne Angabe: keines. */
function emergencyMark(m) {
  if (m.kind === "single") return m.emergencySaleRuns > 0 ? ` ${houseWarn("Notverkauf einer Immobilie in diesem Lauf")}` : "";
  if (m.kind !== "batch" || !(m.runs > 0) || !(m.emergencySaleRuns > 0)) return "";
  const share = m.emergencySaleRuns / m.runs;
  return share >= EMERGENCY_WARN_SHARE ? ` ${houseWarn(`Notverkauf einer Immobilie in ${pctTxt(share)} der Läufe (${fmtInt(m.emergencySaleRuns)} von ${fmtInt(m.runs)} Läufen)`)}` : "";
}
/** Strategiename mit den Parametern des Ergebnisses als Tooltip (auch per Tastatur erreichbar). */
const stratCell = (m, cls = "") => {
  const tip = m.params ? m.params.map(([k, v]) => `${k}: ${v}`).join("\n") : "Parameter nicht verfügbar";
  return `<td class="ptip${cls}" tabindex="0" data-tip="${esc(tip)}" aria-label="${esc(stratName(m.strategy) + ": " + tip.replace(/\n/g, ", "))}">${esc(stratName(m.strategy))}</td>`;
};
function runMeta(r) {
  const base = { source: state.returns.source, strategy: stratKey(state.withdrawal), seriesName: seriesLabel(), params: stratParams(state) };
  if (r.single) return { ...base, kind: "single", runs: 1, endTotal: r.single.summary.endTotal, depleted: r.single.summary.depleted || null, emergencySaleRuns: r.single.summary.emergencySales > 0 ? 1 : 0 };
  if (r.batch) {
    const s = r.batch.summary, pc = q => s.percentiles.find(x => x.percent === q)?.endTotal;
    return { ...base, kind: "batch", runs: s.runs, successRate: s.successRate, emergencySaleRuns: s.emergencySaleRuns, end50: pc(50), end5: pc(5), end1: pc(1), end0: pc(0), end95: pc(95), earliest: s.earliestDepletion ? s.earliestDepletion[0] : null };
  }
  return { ...base, kind: "scenarios", runs: r.scenarios.results.length };
}
const runKind = m => m.kind === "single" ? "Einzellauf" : m.kind === "scenarios" ? `Szenarien (${m.runs} Varianten)` : `${fmtInt(m.runs)} Läufe · ${sourceName(m.source)}`;
async function rememberRun(r) {
  if (r.error || (!r.single && !r.batch && !r.scenarios)) return;
  try {
    const plan = JSON.parse(JSON.stringify(state)), m = runMeta(r);
    const base = (currentPath && baseName(currentPath).replace(/\.[rv]plan$/, "")) || plan.project.name || "Planung";
    const meta = { id: Date.now().toString(36) + Math.random().toString(36).slice(2, 8), created: new Date().toISOString(), name: `${base} · ${runKind(m)} · ${runDate(new Date().toISOString())}`, note: "", ...m };
    await Runs.add(meta, { plan, result: r });
    document.documentElement.dataset.saved = (+document.documentElement.dataset.saved || 0) + 1;
  } catch (e) { noteError("Ergebnis nicht gespeichert: " + errText(e)); }
}
let runsIds = [];
function runsBar() {
  const k = runsSel.size, n = runsIds.length, canCompare = k >= 2 && k <= RUN_COLORS.length;
  $("#resultsTabs").innerHTML = `<button class="btn small" data-action="runs-compare" ${canCompare ? "" : "disabled"} title="Zwei bis ${RUN_COLORS.length} Ergebnisse markieren">Vergleichen (${k})</button>
    <button class="btn small" data-action="runs-delete" ${k < 1 ? "disabled" : ""}>Markierte löschen (${k})</button>
    <button class="btn small" data-action="runs-delete-all" ${n < 1 ? "disabled" : ""}>Alle löschen</button>`;
  const all = document.querySelector('input[data-action="runs-all"]');
  if (all) { all.checked = n > 0 && k === n; all.indeterminate = k > 0 && k < n; }
}
async function openRuns() {
  showResults(true); lastRun = null; savedView = null; compareOpen = false;
  $("#resultsTitle").textContent = "Gespeicherte Ergebnisse";
  let list = [];
  try { list = await Runs.list(); } catch (e) { noteError(errText(e)); }
  for (const id of [...runsSel]) if (!list.some(m => m.id === id)) runsSel.delete(id);
  runsIds = list.map(m => m.id);
  $("#resultsInfo").textContent = `${list.length} ${list.length === 1 ? "Eintrag" : "Einträge"} · nur auf diesem Rechner`;
  runsBar();
  const cell = m => (m.kind === "batch" ? `<span title="Median ${fmtMio(m.end50)}">${pctTxt(m.successRate)} · 5 %: ${fmtMio(m.end5)}</span>` : m.kind === "single" ? `Endvermögen ${fmtMio(m.endTotal)}` : "–") + emergencyMark(m);
  $("#resultsBody").innerHTML = list.length ? `<div class="card"><table class="tbl" id="runsTbl">
    <tr><th><input type="checkbox" data-action="runs-all" title="Alle markieren oder Markierung aufheben"></th><th>Name</th><th>Notiz</th><th>Datum</th><th>Zeitreihe</th><th>Strategie</th><th>Ergebnis</th><th></th></tr>
    ${list.map(m => `<tr><td><input type="checkbox" data-action="run-sel" data-id="${m.id}" ${runsSel.has(m.id) ? "checked" : ""} title="Zum Vergleichen (2 bis ${RUN_COLORS.length}) oder Löschen markieren"></td>
      <td><input class="runedit" data-id="${m.id}" data-f="name" value="${esc(m.name)}" title="Name ändern"></td>
      <td><input class="runedit" data-id="${m.id}" data-f="note" value="${esc(m.note)}" placeholder="Notiz…"></td>
      <td>${esc(runDate(m.created))}</td><td>${seriesCell(m)}</td>${stratCell(m)}<td>${cell(m)}<br><small class="muted">${esc(runKind(m))}</small></td>
      <td style="white-space:nowrap"><button class="btn small" data-action="run-view" data-id="${m.id}">Öffnen</button> <button class="btn small" data-action="run-del" data-id="${m.id}" title="Dieses Ergebnis löschen">Löschen</button></td></tr>`).join("")}
    </table><p class="hint">Jedes Ergebnis wird automatisch hier abgelegt, mit der Planung von damals. Name und Notiz lassen sich direkt in der Liste ändern. Zum Löschen beliebig viele Einträge markieren (oder oben «Alle löschen»); zum Vergleichen zwei bis ${RUN_COLORS.length} Einträge markieren.</p></div>`
    : `<div class="card"><p style="margin:0">Noch kein Ergebnis vorhanden. Jedes Ergebnis erscheint nach dem Start einer Simulation automatisch in dieser Liste.</p></div>`;
  document.documentElement.dataset.rendered = "runs";
}
async function viewRun(id) {
  const [meta] = (await Runs.list()).filter(m => m.id === id), data = await Runs.get(id);
  if (!meta || !data) { toast("Das Ergebnis ist nicht mehr vorhanden."); return openRuns(); }
  resetVariation();
  if (!parkedState) parkedState = state;
  state = normalize(JSON.parse(JSON.stringify(data.plan)));
  savedView = meta; lastRun = data.result; lastRunPlan = state; resultTab = "overview"; detailP = 50;
  showResults(true); renderResults(lastRun);
  document.documentElement.dataset.rendered = "saved";
}
const flat = (o, p = "", out = {}) => {
  if (o && typeof o === "object") { const ks = Object.keys(o); if (!ks.length) out[p] = Array.isArray(o) ? "[]" : "{}"; for (const k of ks) flat(o[k], p ? `${p}.${k}` : k, out); }
  else out[p] = o;
  return out;
};
async function compareRuns() {
  const metas = (await Runs.list()).filter(m => runsSel.has(m.id)).reverse().slice(0, RUN_COLORS.length);
  const datas = await Promise.all(metas.map(m => Runs.get(m.id)));
  lastRun = null; savedView = null; compareOpen = true;
  $("#resultsTitle").textContent = `Vergleich von ${metas.length} Ergebnissen`; $("#resultsInfo").textContent = "nominal";
  $("#resultsTabs").innerHTML = `<button class="btn small" data-action="runs-open">← Zur Liste</button>`;
  const sw = i => `<i class="sw" style="background:var(${RUN_COLORS[i]})"></i>`;
  const num = (v, f = fmtMio) => v == null ? "–" : f(v);
  const rows = metas.map((m, i) => `<tr><td class="l">${sw(i)}${esc(m.name)}<br><small class="muted">${esc(runKind(m))}</small></td><td class="l">${seriesCell(m)}</td>${stratCell(m, " l")}
    <td class="r">${m.kind === "batch" ? pctTxt(m.successRate) : "–"}</td>
    <td class="r">${m.kind === "single" ? num(m.endTotal) : num(m.end50)}</td><td class="r">${num(m.end5)}</td><td class="r">${num(m.end1)}</td><td class="r">${num(m.end0)}</td>
    <td class="r">${m.kind === "single" ? (m.depleted ? ymShow(m.depleted) : "keine") : m.kind === "batch" ? (m.earliest ? ymShow(m.earliest) : "keine") : "–"}</td></tr>`).join("");
  // Verlauf des Gesamtvermögens (Median bzw. Einzellauf) nach Jahren seit Simulationsstart
  const series = datas.map(d => d.result.batch ? { med: d.result.batch.bands.p50, low: d.result.batch.bands.p5 } : d.result.single ? { med: d.result.single.records.map(r => r.total) } : null);
  const n = Math.max(0, ...series.map(x => x ? x.med.length : 0));
  const pad = v => Array.from({ length: n }, (_, i) => i < v.length ? v[i] : NaN);
  const lines = [];
  series.forEach((x, i) => { if (!x) return; lines.push({ color: RUN_COLORS[i], values: pad(x.med) }); if (x.low) lines.push({ color: RUN_COLORS[i], dash: "5 3", width: 1.3, values: pad(x.low) }); });
  const flats = datas.map(d => flat(d.plan)), keys = [...new Set(flats.flatMap(f => Object.keys(f)))].filter(k => k !== "project.name" && flats.some(f => JSON.stringify(f[k]) !== JSON.stringify(flats[0][k])));
  const show = v => v === undefined ? "—" : esc(typeof v === "object" ? JSON.stringify(v) : String(v));
  $("#resultsBody").innerHTML = card("Kennzahlen", "Endvermögen nominal", `<table class="tbl"><tr><th class="l">Ergebnis</th><th class="l">Zeitreihe</th><th class="l">Strategie</th><th class="r">Erfolgsquote</th><th class="r">Median bzw. Endvermögen</th><th class="r">5. Perzentil</th><th class="r">1. Perzentil</th><th class="r">Schlechtester Lauf</th><th class="r">Früheste Erschöpfung</th></tr>${rows}</table>`)
    + (lines.length ? card("Gesamtvermögen im Vergleich", "durchgezogen: Median bzw. Einzellauf, gestrichelt: 5. Perzentil", chartBox("cmp-wealth", 300, true) + legend(metas.map((m, i) => [RUN_COLORS[i], esc(m.name)]))) : "")
    + card("Unterschiede in der Planung", "gegenüber dem ersten Ergebnis in der Reihenfolge der Liste",
      keys.length ? `<table class="tbl"><tr><th class="l">Eingabe</th>${metas.map((m, i) => `<th class="l">${sw(i)}${esc(m.name)}</th>`).join("")}</tr>${keys.slice(0, 60).map(k => `<tr><td class="l"><code>${esc(k)}</code></td>${flats.map(f => `<td class="l">${show(f[k])}</td>`).join("")}</tr>`).join("")}</table>${keys.length > 60 ? `<p class="hint">… und ${keys.length - 60} weitere Unterschiede.</p>` : ""}`
        : `<p style="margin:0">Die Planungen sind identisch.</p>`);
  if (lines.length) {
    const x = Array.from({ length: n }, (_, i) => i);
    Charts.time($("#cmp-wealth"), $("#cmp-wealth-tip"), { x, xFmt: m => `${m / 12} J.`, view: resultZoom ? [resultZoom.from, resultZoom.to] : null, onZoom: setCustomZoom, yFmt: fmtMio, lines,
      tip: i => `<b>Jahr ${(i / 12).toLocaleString("de-CH", { maximumFractionDigits: 1 })}</b>` + series.map((q, k) => q && Number.isFinite(pad(q.med)[i]) ? `<br><span style="color:var(${RUN_COLORS[k]})">■</span> ${esc(metas[k].name)}: ${fmtMoney(q.med[i])}` : "").join("") });
  }
  document.documentElement.dataset.rendered = "compare";
}
Object.assign(ACTIONS, {
  "runs-open": () => openRuns(),
  "run-view": d => viewRun(d.id),
  "run-sel": d => {
    const el = document.querySelector(`input[data-action="run-sel"][data-id="${d.id}"]`);
    el.checked ? runsSel.add(d.id) : runsSel.delete(d.id); runsBar();
  },
  "runs-all": () => {
    const on = document.querySelector('input[data-action="runs-all"]').checked;
    runsSel.clear(); if (on) runsIds.forEach(id => runsSel.add(id));
    document.querySelectorAll('input[data-action="run-sel"]').forEach(el => { el.checked = on; });
    runsBar();
  },
  "runs-delete-all": async () => {
    const n = runsIds.length; if (!n) return;
    if (!await ask("Alle Ergebnisse löschen", `Alle ${n} gespeicherten ${n === 1 ? "Ergebnis wird" : "Ergebnisse werden"} gelöscht. Das lässt sich nicht rückgängig machen.`)) return;
    await Runs.remove([...runsIds]); runsSel.clear(); openRuns();
  },
  "runs-compare": () => compareRuns(),
  "run-del": async d => {
    const [m] = (await Runs.list()).filter(x => x.id === d.id);
    if (!await ask("Ergebnis löschen", `«${esc(m?.name || "")}» wird gelöscht. Das lässt sich nicht rückgängig machen.`)) return;
    await Runs.remove([d.id]); runsSel.delete(d.id); openRuns();
  },
  "runs-delete": async () => {
    const ids = [...runsSel]; if (!ids.length) return;
    if (!await ask("Ergebnisse löschen", `${ids.length} markierte ${ids.length === 1 ? "Ergebnis wird" : "Ergebnisse werden"} gelöscht. Das lässt sich nicht rückgängig machen.`)) return;
    await Runs.remove(ids); runsSel.clear(); openRuns();
  },
  "run-restore": async () => {
    if (!await confirmDiscard()) return;
    parkedState = null; savedView = null; lastRun = null;
    showResults(false); setFile(null, true); History.reset(); goto("general"); refreshSeriesInfo();
    toast("Planung des gespeicherten Ergebnisses geladen.");
  }
});
document.addEventListener("change", e => {
  const el = e.target.closest(".runedit"); if (!el) return;
  Runs.update(el.dataset.id, { [el.dataset.f]: el.value.trim() });
});

// ---------- Willkommen (FEA-54) und Erste Schritte (FEA-55) ----------
/** Lädt die Beispielplanung als neue, ungespeicherte Planung (nach Rückfrage bei ungespeicherten Änderungen). */
async function loadExample() {
  if (!await confirmDiscard()) return false;
  await adopt(await Engine.call("example_plan")); setFile(null); History.reset(); goto("general"); refreshSeriesInfo();
  return true;
}
/**
 * Willkommensdialog: beim allerersten Start und über Hilfe → «Willkommen …». Esc oder Schliessen lässt alles, wie es ist
 * (beim ersten Start ist das die Beispielplanung, also wie «Beispiel erkunden», nur ohne Hilfe).
 */
async function showWelcome() {
  const d = $("#welcome"); if (d.open) return;
  d.returnValue = "";
  const choice = await new Promise(res => { d.onclose = () => res(d.returnValue); d.showModal(); });
  if (choice === "blank") return ACTIONS.new();
  if (choice !== "explore" && choice !== "own") return;
  if (!await loadExample()) return;
  if (choice === "explore") return Help.show("start");
  Steps.reset();
}
/** Schritte der Checkliste: die Abschnitte des Schnellstarts und zuletzt «Simulation starten». */
const STEPS = [["general", "s-general"], ["milestones", "s-milestones"], ["pension", "s-pension"], ["flows", "s-flows"],
  ["wealth", "s-wealth"], ["returns", "s-returns"], ["taxes", "s-taxes"], ["run", "quickstart"]]
  .map(([id, topic]) => ({ id, topic, title: SECTIONS.find(s => s.id === id)?.title ?? "Simulation starten" }));
/**
 * Checkliste «Erste Schritte». Der Fortschritt gilt für den Benutzer (Einstellung «firstSteps»), nicht für eine Planung.
 * Ein Abschnitt ist erledigt, sobald darin ein Wert geändert oder er von Hand abgehakt wurde; Rückgängig nimmt den Haken
 * nicht weg. «Simulation starten» ist erledigt nach einer Simulation ohne Abbruch.
 */
const Steps = {
  get() {
    const v = Pref.get("firstSteps", null) || {};
    return { done: Array.isArray(v.done) ? v.done.filter(id => STEPS.some(s => s.id === id)) : [], hidden: v.hidden === true, complete: v.complete === true };
  },
  put(v, open) { Pref.set("firstSteps", v); this.render(open); },
  /** Hakt einen Schritt ab; sind damit alle erledigt, kommt einmal ein Hinweis und die Liste klappt zu. */
  mark(id, on = true) {
    const v = this.get();
    if (on === v.done.includes(id)) return;
    v.done = on ? [...v.done, id] : v.done.filter(x => x !== id);
    const all = STEPS.every(s => v.done.includes(s.id));
    if (all && !v.complete) { v.complete = true; toast("Alle ersten Schritte erledigt. Die Checkliste ist jetzt zugeklappt."); return this.put(v, false); }
    this.put(v);
  },
  show() { const v = this.get(); v.hidden = false; this.put(v, true); this.reveal(); },
  hide() { const v = this.get(); v.hidden = true; this.put(v); },
  reset() { this.put({ done: [], hidden: false, complete: false }, true); this.reveal(); },
  /** Die Liste steht unter Vorschau und Kennzahlen, oft ausserhalb des sichtbaren Bereichs: dorthin scrollen, Fokus auf den Kopf. */
  reveal() {
    const el = $("#steps"); if (!el || el.hidden) return;
    el.scrollIntoView({ block: "nearest" });
    el.querySelector("summary")?.focus({ preventScroll: true });
  },
  /** `open`: aufgeklappt (true), zugeklappt (false) oder wie bisher (undefined; Standard: offen, bis alles erledigt ist). */
  render(open) {
    const el = $("#steps"); if (!el) return;
    const v = this.get(), was = el.querySelector("details");
    if (open === undefined) open = was ? was.open : !v.complete;
    el.hidden = v.hidden;
    el.innerHTML = `<details${open ? " open" : ""}><summary>Erste Schritte <span class="prog" id="stepsProg">${v.done.length} von ${STEPS.length}</span>
      <button type="button" class="x" data-action="steps-hide" title="Ausblenden (wieder einblenden: Hilfe → Erste Schritte)" aria-label="Erste Schritte ausblenden">✕</button></summary>
      <ol>${STEPS.map((s, i) => { const done = v.done.includes(s.id); return `<li class="${done ? "done" : ""}" data-step="${s.id}">
        <input type="checkbox" data-action="steps-check" data-step="${s.id}"${done ? " checked" : ""} aria-label="${esc(s.title)} erledigt">
        <a data-action="steps-go" data-step="${s.id}">${i + 1}. ${esc(s.title)}</a><button type="button" class="hq" data-help="${s.topic}" title="Hilfe zu diesem Schritt" aria-label="Hilfe">ⓘ</button></li>`; }).join("")}</ol>
      ${v.complete ? `<p class="stepsdone">✓ Alle Schritte erledigt.</p>` : ""}</details>`;
  },
};
/** Eine Änderung im Editor hakt den Abschnitt ab, in dem sie geschieht. */
function stepTouched() {
  if ($(".layout").hidden || !STEPS.some(s => s.id === current)) return;
  if (!Steps.get().done.includes(current)) Steps.mark(current);
}
Object.assign(ACTIONS, {
  "steps-hide": () => Steps.hide(),
  "steps-check": (d, el) => Steps.mark(d.step, el.checked),
  "steps-go": d => d.step === "run" ? runSimulation() : goto(d.step),
  "help-topic": d => Help.show(d.topic),
});

// ---------- Start ----------
(async () => {
  const initial = location.hash.slice(1).split("/");
  await Pref.load();
  showTaxCols = Pref.get("showTaxCols", false) === true;
  if (SECTIONS.some(s => s.id === initial[0])) current = initial[0];
  let startup = null;
  try {
    // Desktop-App: zuletzt geöffnete Planung, sonst die mitgelieferte Beispielplanung (FEA-02)
    if (Engine.inApp) startup = await Engine.call("startup_plan");
    var startNotes = await adopt(startup ? startup.plan : await Engine.call("example_plan"));
  } catch (e) { document.body.innerHTML = `<p style="padding:24px">Der Rechenkern ist nicht erreichbar: ${esc(errText(e))}</p>`; return; }
  for (const src of ["historical", "rolling", "bootstrap", "parametric"]) if (initial.includes(src)) state.returns.source = src;
  if (initial.includes("buckets")) state.withdrawal.strategy = "three_buckets";
  if (initial.includes("fixedrate")) state.withdrawal.strategy = "fixed_rate";
  if (initial.includes("vpw")) state.withdrawal.strategy = "vpw";
  if (initial.includes("endowment")) state.withdrawal.strategy = "endowment";
  if (initial.includes("guardrails")) state.withdrawal.strategy = "guardrails";
  // FEA-87: Testzugang ohne Messung der Rechenzeit; sie liefe sonst in jedem Test im Hintergrund mit
  if (initial.includes("nobench")) bench.off = true;
  // FEA-87: Ergebnisse im Testzugang mit 500 Pfaden und festem Zufallsstartwert: schnell und bei jedem Lauf gleich. Die Tests
  // prüfen Darstellung und Ablauf; 50 000 Pfade machten sie von der Rechnerlast abhängig.
  if (initial[0] === "results") for (const k of ["bootstrap", "parametric"]) Object.assign(state.returns[k], { paths: 500, seed: 42 });
  if (!IS_MAC) document.querySelectorAll("[title]").forEach(el => { el.title = el.title.replace(/⇧⌘/g, "Ctrl+Shift+").replace(/⌘↩/g, "Ctrl+Enter").replace(/⌘/g, "Ctrl+"); });
  setFile(startup?.path ?? null); History.reset();
  if (initial.includes("partner")) { ACTIONS["partner-add"](); state.partner.income = { mode: "const", value: 3000 }; History.reset(); }
  if (initial.includes("nosweep")) state.scenarios.sweeps = [];
  if (initial.includes("gainzh")) { ACTIONS["gain-zh"](); History.reset(); }
  if (initial.includes("optset")) { state.optimization = optToPlan({ ...OPT_DEFAULT, maxRuin: 2, weightMedian: 70, maxVol: 12, strategies: ["guardrails", "vpw"] }); History.reset(); }
  renderSection(); refreshSeriesInfo(); refreshLibrary(); Steps.render();
  // FEA-45: Haftungsausschluss vor allem anderen bestätigen (Desktop-App; im Browser nur mit dem Testzugang #disclaimer).
  if ((Engine.inApp || initial.includes("disclaimer")) && disclaimerPending()) {
    document.documentElement.dataset.disclaimer = "open";
    await disclaimerGate();
    document.documentElement.dataset.disclaimer = "ok";
  }
  // FEA-54: Willkommensdialog beim allerersten Start, nach dem Haftungsausschluss (Testzugang im Browser: #welcome). Gemerkt wird das sofort, damit er
  // auch dann nicht wiederkommt, wenn die App ohne Wahl beendet wird.
  if (startup?.first || initial[0] === "welcome") { Pref.set("welcomeShown", true); showWelcome(); }
  if (startup?.notice) ask("Planung beim Start", `<p>${esc(startup.notice)}</p>`);
  else if (startup?.path) { await showNotes(startNotes); reserveNotice(state); }
  // Testzugang: Einstellung ändern und prüfen, dass sie in der Planung landet
  if (initial.includes("optedit")) {
    const el = $("#opt-maxRuin"); el.value = "3"; el.dispatchEvent(new Event("change", { bubbles: true }));
    document.documentElement.dataset.optsaved = JSON.stringify(state.optimization);
    document.documentElement.dataset.optdirty = dirty ? "1" : "0";
  }
  // Testzugang: Bandbreiten bearbeiten (Topf 1 fix, Topf 2 von 0 bis 2 Jahre, Nachfüllschwelle −30 % bis +10 %)
  if (initial.includes("rangeedit")) (async () => {
    while (!optCat.data) await new Promise(r => setTimeout(r, 50));
    const set = (id, v) => { const el = document.getElementById(id); el[el.type === "checkbox" ? "checked" : "value"] = v; el.dispatchEvent(new Event("change", { bubbles: true })); };
    set("optR-three_buckets-cashYears-var", false); set("optR-three_buckets-bondYears-lo", "0"); set("optR-three_buckets-bondYears-hi", "2");
    set("optR-three_buckets-refillThreshold-lo", "-30"); set("optR-three_buckets-refillThreshold-hi", "10");
    document.documentElement.dataset.ranges = JSON.stringify(state.optimization.ranges);
    document.documentElement.dataset.dirtyr = dirty ? "1" : "0";
    document.documentElement.dataset.rangesum = document.querySelector("details.optrange:nth-of-type(2) summary .computed")?.textContent ?? "";
  })();
  // Entwicklungs- und Testzugang: #results/<quelle>/<registerkarte>/real
  if (initial.includes("real")) realView = true;
  if (initial.includes("sweep2")) state.scenarios.sweeps.push({ param: "flows.spendingFactor", from: 0.9, to: 1.1, step: 0.1 });
  if (initial.includes("nosweep")) state.scenarios.sweeps = [];
  if (initial.includes("mem")) Runs.memoryOnly();
  if (initial[0] === "about") showAbout();
  if (initial[0] === "runs") openRuns();
  // Testzugang: zwei Simulationen ablegen, dann vergleichen (…/view: erste ansehen, …/del: eine löschen). 500 statt 50 000 Pfade mit
  // festem Startwert wie #results (FEA-87): Die Tests prüfen Liste, Vergleich und Darstellung, keine Kennzahlen (ISS-16).
  if (initial[0] === "runsflow") (async () => {
    state.returns.source = "bootstrap"; Object.assign(state.returns.bootstrap, { paths: 500, seed: 42 });
    await rememberRun(await Engine.call("run_simulation", { plan: state, path: null }));
    setP("wealth.stockShare.value", 0.3);
    await rememberRun(await Engine.call("run_simulation", { plan: state, path: null }));
    const list = await Runs.list();
    if (initial.includes("view")) return viewRun(list[1].id);
    if (initial.includes("del")) { await Runs.remove([list[0].id]); return openRuns(); }
    list.forEach(m => runsSel.add(m.id)); compareRuns();
  })();
  if (initial[0] === "results") runSimulation(["charts", "runs", "scenarios", "table"].find(t => initial.includes(t)) || "overview", { noConfirm: true });
  document.documentElement.dataset.ready = "1";
})();
