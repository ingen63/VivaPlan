// VivaPlan user interface: editor, live preview and results view.
"use strict";
/** Technical upper limit of the combinations of the variations (as in the engine, memory protection). */
const MAX_COMBINATIONS = 1_000_000;
/** From this estimated computing time (seconds) on, the app asks before starting. */
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
/** Plan with which `lastRun` was calculated (state of the run, not of the editor); `null`: not known. */
let lastRunPlan = null;
let current = "general";
let flashId = null;
let lastSim = null;

// ---------- Helpers ----------
const $ = s => document.querySelector(s);
/** Closes open info popups (also on mouse or keyboard focus); `true` if one was open. */
function closeInfoPops() {
  const shown = p => p.classList.contains("open") || p.matches(":hover, :focus-within");
  const pops = [...document.querySelectorAll(".infopop")].filter(shown), was = pops.length > 0;
  pops.forEach(p => { p.classList.remove("open"); p.classList.add("closed"); if (p.contains(document.activeElement)) document.activeElement.blur(); });
  return was;
}
document.addEventListener("focusout", e => e.target.closest?.(".infopop")?.classList.remove("closed"));
document.addEventListener("mouseout", e => { const p = e.target.closest?.(".infopop"); if (p && !p.contains(e.relatedTarget)) p.classList.remove("closed"); });
/** Info icon (ⓘ) with popup: opens on mouse, keyboard focus and click, stays open as long as mouse or focus are on it, Esc closes. */
const infoPop = (html, cls = "") => `<span class="infopop" tabindex="0" role="button" aria-label="Erklärung anzeigen" aria-haspopup="true" data-action="info-toggle">ⓘ<span class="infobody ${cls}" role="tooltip">${html}</span></span>`;
/** Short explanation of a series (at most 2 lines): `# Info:` of the file, otherwise source and notes, shortened; without details no icon. */
function seriesTip(i) {
  const text = i?.info || [i?.source, ...(i?.notes || [])].filter(Boolean).join(" ");
  return text ? infoPop(`<span class="two">${esc(text)}</span>`, "short") : "";
}
/** Simulation start as MM.YYYY for labels («… per Simulationsstart (11.2026)»). */
const startTxt = () => ymShow(i2ym(startM()));
const cur = () => state.project.currency;
const fmtMoney = v => `${fmtInt(v)} ${cur()}`;

// Path access to the plan (getPath, setPath in util.js)
function getP(path) { return getPath(state, path); }
function setP(path, v) { setPath(state, path, v); }

// ---------- Milestones & time ----------
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

// Mortgage: one or more tranches, each with a sequence of periods. The first period starts at the
// start date (or at the purchase), each further one at the end of the previous one. Amortization as a fixed amount per quarter,
// due at the end of March/June/September/December; direct (reduces the debt) or indirect (contribution to pillar 3a).
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
  // #50: Indirect amortization only until before the AHV payout of the pillar 3a person, direct afterwards.
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

// ---------- Engine ----------
let lastIssues = [], previewError = null, previewSeq = 0, previewTimer = 0;
const TICK_KINDS = ["mortgageRenewal", "mortgageRepayment", "amortizationSwitch", "pillar3aPayout", "rentStart"];

/** Brings the engine result into the form used by charts and displays. */
function adapt(r) {
  const out = { m: [], liquid: [], prop: [], pk: [], total: [], pots: [], events: [], p3a: [], label: r.label };
  // #53: housing share = entered spending in the start month − spending without housing of the simulation
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
  // Purchase, sale and emergency sale of properties for the timeline
  out.propEvents = (r.propertyEvents || []).filter(e => e.kind !== "renewal").map(e => ({ m: ym2i(e.month), kind: e.kind, property: e.property }));
  const s = r.summary;
  out.depleted = s.depleted ? ym2i(s.depleted) : null;
  out.minLiquid = s.minLiquid; out.minLiquidM = s.minLiquidMonth ? ym2i(s.minLiquidMonth) : startM();
  out.totalAtRetire = s.totalAtRetirement; out.pkPension = s.pensionFundMonthly;
  out.lumpNet = s.lumpsumNet; out.lumpTax = s.lumpsumTax; out.retireM = refMonth(state.pension.pk.at);
  return out;
}

/** `data-preview` on <html>: «pending» while a preview is outstanding, otherwise «idle» (#111: signal for the user interface tests). */
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
    if (req === requestPreview.n) document.documentElement.dataset.preview = "idle"; // no newer request
  }, 120);
}

/** Validation messages of the engine plus notes from the preview. */
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

/** Adds missing parts so that the editor can show every valid project file. */
const SERIES_PATHS = ["flows.income", "flows.spending", "pension.pk.contribution", "pension.pk.interest", "wealth.stockShare", "wealth.reserveMonths", "realestate.emergencyMonths",
  "partner.income", "partner.pk.contribution", "partner.pk.interest"];
/** #58: Labels of the choice for the earned income of each person. */
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
// #18: Plans with a strategy other than «Standard» and a reserve other than 3 months calculate with 3 months.
function reserveDiffers(p) {
  if ((p.withdrawal?.strategy ?? "standard") === "standard") return false;
  const r = p.wealth?.reserveMonths;
  const vals = r == null ? [] : typeof r !== "object" ? [r] : r.mode === "const" ? [r.value] : (r.rows || []).map(x => x.value);
  return vals.some(v => v !== FIXED_RESERVE_MONTHS);
}
/**
 * #61: Disclaimer. The short version must be confirmed at the first start; the confirmation is stored in the
 * settings (not in the plan) with the version of the text. If the text changes, increase the version: then the user is
 * asked again. The full text is in the help (`hilfe/00-einstieg.md`, section «@disclaimer»).
 */
const DISCLAIMER_VERSION = 1, DISCLAIMER_KEY = "disclaimerAccepted";
const DISCLAIMER_SHORT = "VivaPlan ist ein kostenloses Hilfsmittel zur persönlichen Planung. Alle Ergebnisse sind Modellrechnungen auf Basis Ihrer Eingaben und vereinfachter Annahmen. Sie können Fehler enthalten und ersetzen keine Finanz-, Steuer- oder Rechtsberatung. Prüfen Sie wichtige Entscheidungen mit einer Fachperson. Die Nutzung erfolgt auf eigene Verantwortung.";
const DISCLAIMER_FOOT = "Modellrechnung ohne Gewähr. Keine Finanz- oder Steuerberatung.";
const disclaimerPending = () => Pref.get(DISCLAIMER_KEY, 0) !== DISCLAIMER_VERSION;
/** Shows the short version until the box is ticked and «Weiter» is pressed (Esc does not close the dialog). */
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
/** #74: Setting «notice about the cash reserve shown»; the notice appears only when an affected plan is loaded for the first time. */
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
  // Milestone «Alter von» without a person (created when switching from «Datum»): use the first person
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
  // #100: new plans follow the suggested values of the stock index (`auto`); upgrade_plan sets older ones.
  // #132: `sampler` = method of random sampling (montecarlo::SAMPLER); upgrade_plan marks older plans
  p.returns.parametric ??= { paths: 50000, mu: 0.07, sigma: 0.15, df: 6, jumpLambda: 0.05, jumpMu: -0.3, jumpSigma: 0.15, auto: true, sampler: 1 };
  p.returns.parametric.seed ??= 42; p.returns.bootstrap.seed ??= 42;
  p.returns.parametric.auto ??= p.returns.source !== "parametric";
  p.returns.fixed.saron ??= 0;
  // #69: investment strategy (older plans: taken over from the time series, see upgrade_plan)
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
 * Takes over a loaded plan: first bring it up to date (#52, #51; show notes), then
 * normalize it. Returns the notes.
 */
async function adopt(plan) {
  let notes = [];
  try { const r = await Engine.call("upgrade_plan", { plan }); plan = r.plan; notes = r.notes || []; }
  catch { /* without the engine: take over unchanged */ }
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

/** #69: Reference to the time series of the plan: the investment strategy (`strategy:<JSON>`), for older plans the file. */
function seriesRef() {
  const r = state.returns;
  return r.series || (r.strategy ? "strategy:" + JSON.stringify(r.strategy) : null);
}
// #100: Parameters of the stock model with suggested values per stock index (field → display)
const PARAM_KEYS = { mu: "pct", sigma: "pct", df: "num", jumpLambda: "num", jumpMu: "pct", jumpSigma: "pct" };
const paramSuggestion = () => seriesInfo && !seriesInfo.error ? seriesInfo.parametric?.stocks ?? null : null;
const fmtParam = (k, v) => PARAM_KEYS[k] === "pct" ? `${fmtPct(v)} %` : (+v).toLocaleString("de-CH", { maximumFractionDigits: 2 });
/** #100: If the parameters follow the index (`auto`), take over the suggested values of the chosen index. Without
 *  change mark and without a step of its own in the history (the current step gets the same values): the engine
 *  calculates with the suggested values anyway when `auto` is set. */
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
  if (seriesInfoRef !== s) return; // a newer query is running
  followSuggestion();
  // Which fields «Simulation» shows depends on the time series.
  if (current === "returns") renderSection(); else renderDerived();
}

// ---------- Form building blocks ----------
const UNIT = { chf: () => cur(), chfopt: () => cur(), pct: () => "%", months: () => "Monate", years: () => "Jahre", num: () => "" };
function showVal(type, v) {
  if (type === "chf") return fmtInt(v ?? 0);
  if (type === "chfopt") return v == null ? "" : fmtInt(v);
  if (type === "pct") return fmtPct(v ?? 0);
  if (type === "month") return ymShow(v);
  return v ?? "";
}
// ---------- Required inputs (#14; list in docs/Pflichteingaben.md) ----------
const REQUIRED_PATHS = [/^household\.persons\.\d\.(name|birth)$/, /^household\.ahvCoupleCap$/, /^(partner|pension)\.pk\.(capital|conversion|lumpsum)$/,
  /^(partner|pension)\.ahv\.monthly$/, /^wealth\.(stocks|bonds|cash)$/, /^taxes\.multiplier$/];
const isRequired = path => REQUIRED_PATHS.some(r => r.test(path));
const reqMark = path => isRequired(path) ? ` <span class="req" title="Pflichtfeld">*</span>` : "";
/** Values entered by the user that cannot be read (field path → section and message); the plan keeps the last valid value. */
const invalidFields = new Map();
/** Shows error messages of the engine and invalid inputs directly at the field. */
// #57: While typing in a field, everything waits (preview, computed values, messages, checklist); only when
// leaving it (Tab, click elsewhere, Enter) is the plan recalculated and the message at the field shown or removed.
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
/** All errors of the plan: invalid inputs and errors of the engine (freshly validated). */
async function planErrors() {
  const list = [...invalidFields].map(([field, i]) => ({ section: i.section, message: i.message, field }));
  try { for (const i of await Engine.call("validate", { plan: state })) if (i.severity === "error") list.push(i); }
  catch (e) { list.push({ section: "general", message: errText(e) }); }
  return list;
}
/** `true` (with a message and a jump to the first error) if the plan has errors: «Simulation starten» and «Optimieren» are then blocked. */
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
  const input = `<div class="inp"><input class="${cls}" data-path="${path}" data-type="${type}"${o.int ? ' data-int="1"' : ""} value="${esc(showVal(type, v))}" ${type === "month" ? 'placeholder="MM.JJJJ"' : ""}>${unit ? `<span class="unit">${unit}</span>` : ""}</div>`;
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

// ---------- Sections ----------
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
/** #51: Pension fund and AHV start at the fixed milestones of the person. */
function fixedAt(index, kind) {
  const id = index ? (kind === "pk" ? "retire2" : "ahv2") : (kind === "pk" ? "retire" : "ahv");
  return `<label class="field"><span>Bezug ab</span><span class="computed" style="padding:8px 0">Meilenstein «${esc(state.milestones.find(m => m.id === id)?.name ?? id)}» · ${C("ms", id)} · <a href="#" data-action="goto" data-sec="milestones">ändern</a></span></label>`;
}
R.pension = () => `
  <h2>Vorsorge</h2><p class="lead">Pensionskasse (2. Säule) und AHV (1. Säule)${state.partner ? " für beide Personen" : ""}. Die Säule 3a hat einen eigenen Abschnitt.</p>
  ${PensionCards("pension", state.partner ? state.household.persons[0].name : "")}
  ${state.partner ? PensionCards("partner", state.household.persons[1].name) : ""}
  <p class="hint">${C("pkresult")}</p>`;

// ---------- Pillar 3a (#52) ----------
const newAccount = person => ({ person, balance: 0, rate: 0.01, contribution: 0, payouts: [], use: "wealth" });
/** Id of the milestone «AHV-Bezug» of the person (tranche without a person: first person). */
const ahvIdOf = pid => state.household.persons.findIndex(p => p.id === pid) === 1 ? "ahv2" : "ahv";
/** «Hypothek zurückzahlen» requires a tranche with indirect amortization into this account. */
const hasIndirectFor = pid => state.realestate.properties.some(p => p.status !== "rent" && (p.mortgage?.tranches || []).some(t =>
  (t.pillar3a ?? state.household.persons[0].id) === pid && t.periods.some(d => d.amortMode === "indirect" && +d.amort > 0)));
// #55: Month of a 3a payout (date or age of the account holder).
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

/** #69: Indices of the investment strategy: [key, name, explanation]. */
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
/** #119: Abbreviation of each return source for the name of a saved result. */
const SOURCE_SHORT = { fixed: "Fix", historical: "Hist.", rolling: "Alle Starts", bootstrap: "MC hist.", parametric: "MC param." };
const paramJoint = () => seriesInfo && !seriesInfo.error ? seriesInfo.parametric?.joint ?? null : null;
/** Which inputs are actually used with the chosen source. */
function returnsUse() {
  const r = state.returns, src = r.source, i = seriesInfo && !seriesInfo.error ? seriesInfo : null;
  const usesSaron = state.realestate.properties.some(p => p.status !== "rent" && (p.mortgage?.tranches || []).some(t => t.periods.some(d => d.type === "saron")));
  const withSeries = ["historical", "rolling", "bootstrap"].includes(src);
  // SARON from the series: only outside its SARON period the fixed value applies.
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
    // #101: The parametric model takes bonds and inflation from the series; fixed only without a series.
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
  if (src === "bootstrap") params = `<div class="card"><h3>Bootstrap</h3><div class="grid3">${F("returns.bootstrap.paths", "Anzahl Pfade", "num", { int: true })}${F("returns.bootstrap.block", "Mittlere Blocklänge", "months")}${F("returns.bootstrap.seed", "Zufallsstartwert", "num", { int: true })}</div>
    <p class="hint">Jeder Pfad wird aus Abschnitten der Zeitreihe zusammengesetzt. Ein Abschnitt endet in jedem Monat mit Wahrscheinlichkeit 1 / Blocklänge, dann folgt ein zufälliger neuer Startmonat (stationärer Bootstrap). Aktien, Obligationen, Inflation und SARON stammen immer aus demselben historischen Monat; Krisen behalten so ihren Verlauf. Bei Jahresdaten gilt die Blocklänge in ganzen Jahren.</p>
    <p class="hint">Gleicher Zufallsstartwert ⇒ gleiche Pfade. Szenarien verwenden dieselben Pfade, damit Unterschiede von den Parametern stammen.</p></div>`;
  if (src === "parametric") {
    const P = (k, label) => F("returns.parametric." + k, label, PARAM_KEYS[k], { hint: C("psug", k) });
    params = `<div class="card"><h3>Aktienmodell</h3><p class="hint" style="margin-top:0">${C("pauto")}</p><div class="grid3">${F("returns.parametric.paths", "Anzahl Pfade", "num", { int: true })}${P("mu", "Erwartete Rendite (Drift)")}${P("sigma", "Volatilität")}
    ${P("df", "Freiheitsgrade (Fat Tails)")}${P("jumpLambda", "Kurssprünge pro Jahr")}${P("jumpMu", "Mittlere Sprunghöhe (log)")}
    ${P("jumpSigma", "Streuung der Sprunghöhe")}${F("returns.parametric.seed", "Zufallsstartwert", "num", { int: true })}</div>
    <p class="hint">Tagesschritte (21 pro Monat), Student-t-Schocks auf Varianz 1 skaliert (0 Freiheitsgrade = Normalverteilung), Poisson-Kurssprünge. Eine Sprunghöhe von −30 % (log) entspricht etwa −26 % Kursverlust.</p></div>
    <div class="card"><h3>Obligationen und Inflation <span class="sub">aus der Anlagestrategie, nur Anzeige</span></h3><div id="paramJoint">${C("pjoint")}</div></div>`;
  }
  const extra = !u.extrapolate ? "" : `<div class="card"><h3>Fortschreibung <span class="sub">die Zeitreihe endet vor dem Simulationsende</span></h3>
    <div class="grid2"><label class="field"><span>Methode</span>${Sel("returns.extrapolation.method", [["fixed", "Fixe Werte"], ["average", "Gleitender Durchschnitt"], ["geometric", "Gleitender geometrischer Durchschnitt"]], { rerender: 1 })}</label>
    ${r.extrapolation.method === "fixed" ? "" : F("returns.extrapolation.range", "Durchschnitt über", "num", { unit: "Jahre", int: true })}</div></div>`;
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
/** #119: Abbreviation of each strategy (3 buckets with guardrails separately) for the name of a saved result. */
const STRAT_SHORT = { standard: "Standard", three_buckets: "3-Topf", three_buckets_guardrails: "3-Topf + Guard", guardrails: "Guardrails", fixed_rate: "Fixe Rate", vpw: "VPW", endowment: "Endowment" };
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

/** #6: 3 buckets whose spending is controlled by the guardrails. */
const bucketsGuardrails = (w = state.withdrawal) => w?.strategy === "three_buckets" && w.buckets?.spending === "guardrails";
/** Key of the withdrawal strategy of a plan; 3 buckets with guardrails has its own (as in the optimization). */
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
// ---------- Optimization ----------
const OPT_STRATS = [["standard", "Standard"], ["three_buckets", "3-Topf"], ["three_buckets_guardrails", "3-Topf + Guardrails"], ["guardrails", "Guardrails"], ["fixed_rate", "Fixe Rate"], ["vpw", "VPW"], ["endowment", "Endowment"]];
const OPT_DEFAULT = { goal: "consumption", minConsumption: 90, maxRuin: 5, ruinAge: "", weightMedian: 50, volWeight: 1, maxVol: "", paths: 5000, checkPaths: 10000, samples: 30, strategies: OPT_STRATS.map(x => x[0]).filter(k => k !== "three_buckets_guardrails"), ranges: {} };
let optCfg = { ...OPT_DEFAULT };
const OPT_ACTIONS = {};
let lastOpt = null, optBusy = false, optError = null;
// The settings are stored in the plan (`optimization`, shares as decimals) and are saved with it.
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
/** Key of the plan without the optimization settings: they do not change the result of an earlier optimization. */
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

// ---------- Duration estimate ----------
// We measure the time of one run of this plan on this computer (command bench_plan). From it we extrapolate the runs
// that scenarios and optimization need. The search steps per strategy come from measurements with 30 starting points.
const OPT_SEARCH_EVALS = { standard: 22, fixed_rate: 14, three_buckets: 46, three_buckets_guardrails: 110, endowment: 40, vpw: 74, guardrails: 90 };
// ----- Progress and cancelling of running calculations -----
let job = null;
/** Id of the next calculation; starts from the time so that after a reload it does not coincide with an old one. */
let jobSeq = Date.now();
/** HTML of the progress display; `jobTick` fills it during the calculation. */
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
/** Runs `fn` and shows the progress meanwhile; returns `undefined` if it was cancelled. */
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
// `off`: test access #nobench (#111). Without a measurement the app shows «Dauer wird geschätzt…» and calculates without asking.
const bench = { key: "", us: null, gen: 0, fixedPaths: null, cores: 0, busy: false, error: null, off: false };
const benchKey = () => JSON.stringify({ ...state, optimization: undefined, scenarios: undefined });
/** The paths of the search are generated once and then only calculated per candidate; the check paths are created anew for every evaluation. */
/** Evaluations per strategy; with fewer varied parameters there are fewer search steps. */
function optEvals(k, c) {
  const s = optCat.data?.find(x => x.key === k), search = OPT_SEARCH_EVALS[k] ?? 60;
  if (!s) return c.samples + 2 + search;
  const varied = s.params.filter(d => !(c.ranges?.[k]?.[d.key]?.fixed ?? d.fixed)).length, usual = Math.max(1, s.params.filter(d => !d.fixed).length);
  return varied === 0 ? 1 : c.samples + 2 + search * varied / usual;
}
function optSeconds(c) { return optSecondsFor(bench, c, c.strategies.reduce((a, k) => a + optEvals(k, c), 0)); }
/** Estimated computing time from which a warning icon appears in the card «Umfang» (the question before the start has its own limit). */
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
/** The measurement runs shortly after the last change so that it does not start constantly while typing. */
function scheduleBench() { if (bench.off) return; clearTimeout(scheduleBench.t); scheduleBench.t = setTimeout(ensureBench, 800); }
function optInput(id, label, unit, hint = "") {
  return `<label class="field"><span>${label}</span><div class="inp"><input class="num" id="opt-${id}" value="${esc(optCfg[id])}">${unit ? `<span class="unit">${unit}</span>` : ""}</div>${hint ? `<span class="computed">${hint}</span>` : ""}</label>`;
}
// ----- Parameters of the strategies: fixed or varied within a range -----
// The catalog (command opt_params) lists per strategy the parameters with unit, default range and value of the plan.
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
/** Result for goal 2 «Kein Notverkauf von Immobilien»: per strategy the share of runs with an unplanned sale. */
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
/** #140: Integer of the optimization; «50.000» is ambiguous and is not taken as 50 (the previous value stays). */
function optInt(text, prev) {
  if (ambiguousThousands(text)) { toast(AMBIGUOUS_TEXT); return prev; }
  return Math.round(optNum(text, prev));
}
function readOptCfg() {
  const v = id => document.getElementById("opt-" + id)?.value ?? optCfg[id];
  const goalEl = document.getElementById("opt-goal");
  optCfg = { goal: goalEl ? goalEl.value : optCfg.goal, minConsumption: optNum(v("minConsumption"), 90), maxRuin: optNum(v("maxRuin"), 5), ruinAge: String(v("ruinAge")).trim(), weightMedian: optNum(v("weightMedian"), 50), volWeight: optNum(v("volWeight"), 1),
    maxVol: String(v("maxVol")).trim(), paths: optInt(v("paths"), optCfg.paths ?? 5000), checkPaths: optInt(v("checkPaths"), optCfg.checkPaths ?? 10000), samples: optInt(v("samples"), optCfg.samples ?? 30),
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

// ----- Property gains tax (rules per canton, editable) -----
/** #138: Templates of the property gains tax from the engine (crates/engine/assets/gain-templates.json): Zurich
 *  (tariff § 225 StG, StA eForm. 225.1, as of 09/2025) and Aargau (§ 109 StG). Loaded at startup. */
let gainTemplates = null;
const GAIN_ZH = () => structuredClone(gainTemplates.zurich);
const GAIN_AG = () => structuredClone(gainTemplates.aargau);
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
  "gain-zh": () => { state.taxes.propertyGain = { ...GAIN_ZH(), useMultiplier: !!state.taxes.propertyGain?.useMultiplier }; markDirty(); renderSection(); },
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
/** #50: Period that only starts from the AHV payout (then only direct amortization). */
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

// ---------- Computed displays ----------
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
  // #55: Payout plan of pillar 3a (row «k.j»): month and age, balance before and after the payout from the preview.
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
  // #73: Share of stocks or bonds in the portfolio (without cash), integer; without a portfolio no value.
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
  // #69: Sum of the weights of «Welt individuell»
  wsum: () => {
    const w = state.returns.strategy?.weights || {}, sum = REGION_KEYS.reduce((a, k) => a + (+w[k] || 0), 0);
    return `Summe <b${Math.abs(sum - 1) > 1e-6 ? ' style="color:var(--err)"' : ""}>${fmtPct(sum)} %</b> (muss 100 % ergeben).`;
  },
  // #69: Short info on the time series of the strategy
  strategyinfo: () => {
    const i = seriesInfo;
    if (!i) return "";
    if (i.error) return i.error.includes("Gewichte") ? "" : `<span style="color:var(--err)">${esc(i.error)}</span>`; // weights: message at the field
    const p = v => v == null || Number.isNaN(v) ? "–" : `${(v * 100).toLocaleString("de-CH", { minimumFractionDigits: 1, maximumFractionDigits: 1 })} %`;
    // #69/AK8: The means apply to the common period; a longer index is cut to it.
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
  // #100: Suggested value next to a field of the stock model (only with own values)
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
  // #101: Mean, spread and correlation of stocks, bonds and inflation of the series
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
/** Section of the editor for a main key of the plan (for the messages when deleting a milestone). */
const SECTION_OF = { household: "Haushalt & Projekt", pension: "Vorsorge", partner: "Vorsorge", pillar3a: "Säule 3a", flows: "Einnahmen & Ausgaben", wealth: "Vermögen & Anlage", returns: "Simulation",
  withdrawal: "Entnahmestrategie", optimization: "Optimierung", taxes: "Steuern", realestate: "Immobilien", scenarios: "Szenarien & Simulation" };
/** Uses of a milestone outside the milestone table: [{ section, set(value) }]. */
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
/** #51: Fixed milestones of a person («Bezug Pensionskasse», «AHV-Bezug»); `null` for all others. */
const SYS_MS = { retire: { kind: "pk", index: 0 }, ahv: { kind: "ahv", index: 0 }, retire2: { kind: "pk", index: 1 }, ahv2: { kind: "ahv", index: 1 } };
const sysMs = ms => SYS_MS[ms.id] ?? null;
/** Start, end and the fixed milestones of each person cannot be deleted. */
const msLocked = ms => !!ms.fixed || !!sysMs(ms);

// ---------- Rendering ----------
/**
 * #112: The menu is built once and afterwards only updated (active section, number of errors and warnings).
 * Rebuilding it on every preview replaced the entries; if that happened between pressing and releasing the mouse button, the
 * click was lost.
 */
function renderNav(vals) {
  const nav = $("#nav");
  if (nav.querySelectorAll("a[data-sec]").length !== SECTIONS.length)
    nav.innerHTML = SECTIONS.map((s, i) => `<a data-action="goto" data-sec="${s.id}"><span class="num">${i + 1}</span>${s.title}</a>`).join("")
      + `<div class="navfoot">Daten bleiben lokal auf diesem Rechner. Keine Verbindung ins Internet.</div>`;
  for (const a of nav.querySelectorAll("a[data-sec]")) {
    const id = a.dataset.sec, e = vals.filter(v => v.sec === id && v.sev === "err").length, w = vals.filter(v => v.sec === id && v.sev === "warn").length;
    a.classList.toggle("active", id === current);
    const want = e ? `<span class="badge err">${e}</span>` : w ? `<span class="badge warn">${w}</span>` : "", old = a.querySelector(".badge");
    if ((old?.outerHTML ?? "") === want) continue;
    old?.remove();
    if (want) a.insertAdjacentHTML("beforeend", want);
  }
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
  const html = vals.length ? vals.map(v => `<li class="${v.sev}" data-action="goto" data-sec="${v.sec}"><span class="ic">${v.sev === "err" ? "!" : "▲"}</span><span>${v.text}</span><span class="sec">${SECTIONS.find(s => s.id === v.sec).title}</span></li>`).join("")
    : `<li class="ok"><span class="ic">✓</span><span>Keine Fehler gefunden.</span></li>`;
  // #112: only rebuild if the messages change (otherwise a click on a message would be lost)
  if (html !== renderVal.html) { $("#val").innerHTML = html; renderVal.html = html; }
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
  // #114: Reference line at 0 as in the result charts
  g += `<line class="ref" x1="${L}" x2="${W - Rr}" y1="${y(0)}" y2="${y(0)}" stroke="${css("--text")}" stroke-opacity=".6" stroke-width="2"/>`;
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
  // Property events (only with a simulation result): triangle below the line, label underneath, staggered.
  // Shape and position distinguish them from the milestones (circle on the line, label above).
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

/** Visible range per mini chart (month indices); without an entry the whole period. */
const miniZoom = {};
/** Shared by editor and result charts: zoom buttons, default «the first N years» and visible range. */
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
  // Exact steps: value at the start of the section and at every change within it.
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

// ---------- Events ----------
/** Tells the desktop app whether there are unsaved changes (for the question when quitting). */
function pushDirty() { if (Engine.inApp) Engine.call("set_dirty", { dirty }).catch(() => {}); }
function markDirty() { dirty = true; $("#dirty").textContent = "● ungespeichert"; pushDirty(); History.note(); stepTouched(); }

// ---------- Undo / redo ----------
/** History of the plan as JSON snapshots; quick inputs are merged. */
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
  /** Changes the current step without creating a new one (derived values, #100). */
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

// ---------- Keyboard shortcuts ----------
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
/** Runs a command (menu of the desktop app or keyboard); a duplicate trigger is ignored. */
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
  try { info = await Engine.call("app_info"); } catch { /* without the engine no details */ }
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
  if (e.key === "Enter" && e.target.matches?.("input[data-path]")) releaseField(e.target); // #57
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
// Menu of the desktop app
if (window.__TAURI__?.event) window.__TAURI__.event.listen("menu", ev => command(ev.payload));
/** Quitting with unsaved changes: save, discard or cancel. */
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
  // #140: Amounts and integers do not take «1.234» as 1.234
  const v = t === "chf" || t === "chfopt" || el.dataset.int ? parseAmount(el.value) : parseNum(el.value);
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
  // #57: only text fields wait; checkboxes and radio buttons take effect immediately
  const typing = document.activeElement === el && el.matches("input:not([type=checkbox]):not([type=radio])");
  if (typing) holdField(el);
  if (v === null) {
    const ambiguous = (["chf", "chfopt"].includes(el.dataset.type) || el.dataset.int) && ambiguousThousands(el.value);
    invalidFields.set(path, { section: current, message: el.dataset.type === "month" ? "Ungültiges Datum (MM.JJJJ)." : ambiguous ? AMBIGUOUS_TEXT : "Ungültige Zahl." });
    if (!typing) { el.classList.add("invalid"); applyFieldIssues(); }
    return;
  }
  invalidFields.delete(path);
  if (!typing) el.classList.remove("invalid");
  const kindMs = /^milestones\.\d+\.kind$/.test(path) ? getP(path.replace(/\.kind$/, "")) : null;
  const resulting = kindMs ? msMonth(kindMs.id) : NaN; // point in time before the change (column «Ergibt»)
  const kindPay = /^pillar3a\.\d+\.payouts\.\d+\.kind$/.test(path) ? getP(path.replace(/\.kind$/, "")) : null;
  const payPid = kindPay ? state.pillar3a[+path.split(".")[1]].person : null, payAt = kindPay ? payoutMonth(kindPay, payPid) : NaN;
  setP(path, v); markDirty();
  // #100: an own input in the stock model ends following the suggested values
  if (/^returns\.parametric\.(mu|sigma|df|jumpLambda|jumpMu|jumpSigma)$/.test(path)) state.returns.parametric.auto = false;
  if (/\.amortMode$/.test(path) && v === "indirect") {
    // #52: Indirect amortization needs a 3a account (first person or chosen person of the tranche).
    const t = getP(path.split(".").slice(0, 6).join(".")), pid = t?.pillar3a ?? state.household.persons[0].id;
    if (!state.pillar3a.some(a => a.person === pid)) state.pillar3a.push(newAccount(pid));
  }
  if (/^milestones\.\d+\.kind$/.test(path)) { // «Alter von» needs a person and years, «Datum» a date
    const m = getP(path.replace(/\.kind$/, ""));
    // The point in time stays the same: «Alter von» takes over the age, «Datum» the date from the column «Ergibt»
    if (v === "age") {
      if (sysMs(m)) m.person = state.household.persons[sysMs(m).index]?.id;
      m.person ??= state.household.persons[0]?.id;
      m.years = Number.isFinite(resulting) ? Math.round((resulting - ym2i(person(m.person).birth)) / 12 * 100) / 100 : m.years ?? 65;
    } else m.date = Number.isFinite(resulting) ? i2ym(resulting) : m.date ?? i2ym(startM() + 60);
  }
  if (kindPay) { // #55: as with milestones the point in time stays the same on change
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
 * Empty plan with default values for «Neu»: start in the current month, one person without a name, no amounts, no
 * properties, no variations. Returns, tax tables and strategy parameters keep the default values of the template.
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
  // #100: follow the suggested values of the stock index again
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
    // Time series: same point in time in several rows → the later row applies
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
    // #55: one year after the last row (without a row: earliest allowed month), at most at the AHV payout; amount
    // = balance according to the preview at that point in time after the earlier payouts, rounded down to 1'000.
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
  // #61/AK4: from the dialog «Über VivaPlan» to the full disclaimer in the help
  "about-disclaimer": () => { $("#ask").close(); Help.show("disclaimer"); },
  back: () => showResults(false),
  detail: d => showDetail(+d.p),
  rtab: d => { resultTab = d.tab; renderResults(lastRun); },
  real: d => { realView = d.v === "1"; redrawResults(); },
  "surv-scale": d => { survFull = d.v === "0"; redrawResults(); },
  // #115: Tabs and selections in the comparison
  "cmp-tab": d => { compareTab = d.tab; compareRuns(); },
  "cmp-liquid": d => { cmpLiquid = d.v; compareRuns(); },
  "cmp-flow": d => { cmpFlow = d.v; compareRuns(); },
  "cmp-p": d => { cmpP = +d.p; compareRuns(); },
  // #118: first click ascending, each further one reverses the direction
  "cmp-sort": d => { cmpSort = cmpSort?.col === d.col ? { col: d.col, dir: -cmpSort.dir } : { col: d.col, dir: 1 }; compareRuns(); },
  "bk-toggle": d => { bucketOpen.has(d.y) ? bucketOpen.delete(d.y) : bucketOpen.add(d.y); renderResults(lastRun); },
  "info-toggle": (d, el) => { const open = !el.classList.contains("open"); document.querySelectorAll(".infopop.open").forEach(x => x.classList.remove("open")); if (open) el.classList.add("open"); },
  "tax-cols": () => { showTaxCols = !showTaxCols; Pref.set("showTaxCols", showTaxCols); renderResults(lastRun); },
  comp: d => { compPercent = d.v === "1"; renderResults(lastRun); },
  // #122: applies in the single result and in the comparison
  mkt: d => { mktPct = d.v === "pct"; redrawResults(); },
  "mkt-log": d => { mktLog = d.v === "1"; redrawResults(); },
  // Zoom of the result charts: applies to all tabs and is kept when switching
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
/** Takes over an opened plan ({path, plan}) into the editor. */
async function showOpened(r) {
  if (!r) return;
  const notes = await adopt(r.plan); setFile(r.path); History.reset(); goto("general"); refreshSeriesInfo();
  toast(`«${baseName(r.path)}» geöffnet.`);
  await showNotes(notes);
  await reserveNotice(state);
}
// #44: Double-click on a plan in the Finder while the app is running.
if (window.__TAURI__?.event) window.__TAURI__.event.listen("open-file", async ev => {
  if (!await confirmDiscard()) return;
  try { await showOpened(await Engine.call("open_path", { path: ev.payload })); } catch (e) { toast(errText(e)); }
});
/** #44: Window title «VivaPlan - ‹file name›», without a file just «VivaPlan». */
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
/** Own dialog (instead of prompt/confirm). With `value` there is an input field. */
function ask(title, html, value = null, okLabel = "OK", discardLabel = null) {
  return new Promise(res => {
    const d = $("#ask"), inp = $("#askInput"), dis = d.querySelector('button[value="discard"]');
    dis.hidden = !discardLabel; if (discardLabel) dis.textContent = discardLabel;
    d.querySelector('button[value="ok"]').textContent = okLabel;
    $("#askTitle").textContent = title; $("#askText").innerHTML = html;
    inp.hidden = value === null; inp.value = value ?? ""; d.returnValue = "";
    // #111: The event «close» arrives late. If the dialog is already open again (next question), it belongs to the previous one.
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
  // A click outside closes open info popups; inside the popup text stays selectable and copyable.
  if (!e.target.closest(".infopop")) document.querySelectorAll(".infopop.open").forEach(x => x.classList.remove("open"));
  if (e.target.closest(".infobody")) return;
  const el = e.target.closest("[data-action]"); if (!el) return;
  const fn = ACTIONS[el.dataset.action]; if (!fn) return;
  // Checkboxes keep their state (preventDefault would reset the tick).
  if (el.type !== "checkbox") e.preventDefault();
  fn(el.dataset, el);
});
$("#fileInput").addEventListener("change", async e => {
  const f = e.target.files[0]; if (!f) return;
  try {
    const obj = JSON.parse(await f.text());
    // #44: old format id pensionsplaner/1 stays readable (legacy: RetirementPlanner). #144: another format id comes
    // from a newer version and is rejected with its own message.
    if (typeof obj?.format === "string" && !KNOWN_FORMATS.includes(obj.format)) throw new Error(unknownFormatText(obj.format));
    if (!KNOWN_FORMATS.includes(obj?.format)) throw 0;
    const notes = await adopt(obj); setFile(null); History.reset(); $("#fileName").textContent = f.name; showTitle(f.name); goto("general"); refreshSeriesInfo();
    toast(`«${f.name}» geöffnet.`);
    await showNotes(notes);
    await reserveNotice(state);
  } catch (err) { toast(err instanceof Error && err.message.startsWith("Die Datei hat das Format") ? err.message : "Datei ist keine Projektdatei im neuen Format."); }
  e.target.value = "";
});
window.addEventListener("resize", () => { renderDerived(); if (lastRun) renderResults(lastRun); });
matchMedia("(prefers-color-scheme: dark)").addEventListener("change", () => { renderDerived(); if (lastRun) renderResults(lastRun); });

// ---------- Settings (#25) ----------
/** Default values of the settings; the engine knows the same ones (crates/engine/src/settings.rs). */
const LIMIT_DEFAULTS = { ahvMin: 63, ahvMax: 70, pkMin: 58, pkMax: 65, pillar3aMax: 7258 };
const LIMIT_FIELDS = [["ahvMin", "AHV-Bezug: untere Grenze", "Jahre"], ["ahvMax", "AHV-Bezug: obere Grenze", "Jahre"],
  ["pkMin", "Bezug Pensionskasse: untere Grenze", "Jahre"], ["pkMax", "Bezug Pensionskasse: obere Grenze", "Jahre"],
  ["pillar3aMax", "Säule 3a: Maximalbetrag der Einzahlung pro Jahr", null]];
/** Applicable limits: saved values, otherwise the defaults. They apply to all plans on this device. */
function limits() {
  const saved = Pref.get("limits", null), l = { ...LIMIT_DEFAULTS };
  if (saved && typeof saved === "object") for (const k of Object.keys(LIMIT_DEFAULTS)) if (Number.isFinite(saved[k])) l[k] = saved[k];
  return l;
}
Engine.limits = limits;
/** Validation as in the engine (`Limits::problems`). */
function limitProblems(l) {
  const out = [], whole = x => Number.isInteger(x) && x >= 0 && x <= 130;
  for (const [what, lo, hi] of [["AHV-Bezug", l.ahvMin, l.ahvMax], ["Bezug Pensionskasse", l.pkMin, l.pkMax]]) {
    if (!whole(lo) || !whole(hi)) out.push(`${what}: Die Grenzen müssen ganze Jahre zwischen 0 und 130 sein.`);
    else if (lo > hi) out.push(`${what}: Die untere Grenze (${lo}) liegt über der oberen Grenze (${hi}).`);
  }
  if (!(Number.isFinite(l.pillar3aMax) && l.pillar3aMax >= 0)) out.push("Säule 3a: Der Maximalbetrag muss 0 oder mehr sein.");
  return out;
}
/** #110: Question about saving before the simulation (default: on). */
const askSaveOn = () => Pref.get("askSaveBeforeRun", true) !== false;
function settingsFields(l) {
  return `<label class="check" style="grid-column:1/-1"><input type="checkbox" id="set-askSave" ${askSaveOn() ? "checked" : ""}> Vor der Simulation fragen, ob die Planung gespeichert werden soll</label>`
    + LIMIT_FIELDS.map(([k, label, unit]) => `<label class="field"><span>${label}</span><div class="inp"><input class="num" id="set-${k}" value="${esc(k === "pillar3aMax" ? fmtInt(l[k]) : String(l[k]))}"><span class="unit">${unit ?? cur()}</span></div><span class="computed">Standard ${k === "pillar3aMax" ? fmtInt(LIMIT_DEFAULTS[k]) : LIMIT_DEFAULTS[k]}</span></label>`).join("");
}
function readSettings() {
  const l = {};
  // #140: The maximum amount of pillar 3a is an amount («7.258» is ambiguous)
  for (const [k] of LIMIT_FIELDS) { const t = $("#set-" + k).value, v = k === "pillar3aMax" ? parseAmount(t) : parseNum(t); l[k] = v === null ? NaN : v; }
  return l;
}
/** Dialog «Einstellungen»: applies to all plans, is stored locally, never in the plan file. */
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

// ---------- Results view ----------
/**
 * Persistent settings of the user interface. In the desktop app they are a file in the data directory (commands
 * `settings_get`/`settings_set`), in the browser development mode in browser storage. Not readable: the defaults apply.
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
    if (Engine.inApp) Engine.call("settings_set", { key, value }).catch(() => { /* then applies only to this session */ });
    else try { localStorage.setItem("rp.settings", JSON.stringify(this.cache)); } catch { /* as above */ }
  },
};
let showTaxCols = false;
/** Years whose 3-bucket derivation is expanded in the yearly values. */
const bucketOpen = new Set();
let resultTab = "overview", realView = false, compPercent = false, detailP = 50, mktPct = false, mktLog = false, resultZoom = null, compareOpen = false;
function showResults(on) {
  if (!on) unpark();
  $("#results").hidden = !on; $(".layout").hidden = on; $("#timeline").hidden = on;
  if (on) $("#resultsPick").innerHTML = "";
  if (!on) { lastRun = null; renderDerived(); }
}
/** Asks before a calculation of more than ${LONG_RUN_SECONDS} s estimated duration; `false` if the user cancels. */
async function confirmLongRun() {
  if (bench.us == null) await ensureBench();
  if (bench.us == null) return true;
  const { runs, sec } = scenSeconds();
  if (!(sec > LONG_RUN_SECONDS)) return true;
  return !!await ask("Lange Berechnung", `Die Berechnung dauert voraussichtlich ca. <b>${esc(fmtShort(sec))}</b> (${fmtInt(runs)} ${runs === 1 ? "Simulationslauf" : "Simulationsläufe"}). Sie lässt sich während der Berechnung abbrechen.`, null, "Trotzdem starten");
}
/**
 * #110: If the plan has unsaved changes, the app asks before the simulation whether to save.
 * Only in the desktop app (in the browser «Speichern» is a download); `askSaveDev` in the settings turns it on for tests.
 * `false` if the user cancels or saving does not succeed.
 */
async function confirmSaveBeforeRun() {
  if (!dirty || !askSaveOn() || !(Engine.inApp || Pref.get("askSaveDev", false))) return true;
  const r = await ask("Planung speichern?", `Die Planung hat ungespeicherte Änderungen${currentPath ? "" : " und noch keinen Dateinamen"}. Vor der Simulation speichern?`, null, "Speichern", "Ohne Speichern simulieren");
  if (r === null) return false;
  if (r === true) { await save(false); if (dirty) return false; }
  return true;
}
// #125: Header of the results. For a result first its name (saved or just stored), then the kind
// of run; the name is shortened and is shown in full in the tooltip. Without a name only the kind.
let lastRunName = null, resultHead = "";
function plainTitle(t) { const el = $("#resultsTitle"); el.textContent = t; el.removeAttribute("title"); }
function setResultsTitle(head = resultHead) {
  resultHead = head;
  const name = savedView ? savedView.name : lastRunName;
  if (!name) return plainTitle(head);
  const el = $("#resultsTitle");
  el.innerHTML = `<span class="rtname">${esc(name)}</span><span class="rtsep"> · </span><span class="rtkind">${esc(head)}</span>`;
  el.title = name;
}
async function runSimulation(tab = "overview", { noConfirm = false } = {}) {
  if (await blockedByErrors()) return;
  if (!await confirmSaveBeforeRun()) return;
  if (!noConfirm && !await confirmLongRun()) return;
  unpark(); showResults(true); resultTab = tab; detailP = 50; resetVariation(); lastRunName = null;
  plainTitle("Simulation läuft…"); $("#resultsInfo").textContent = ""; $("#resultsTabs").innerHTML = "";
  $("#resultsBody").innerHTML = jobCard("Simulation läuft…", "Die Berechnung läuft lokal auf diesem Rechner.");
  try {
    const planAtRun = JSON.parse(JSON.stringify(state));
    lastRun = await withJob(() => Engine.call("run_simulation", { plan: state, path: currentPath, jobId: job.id }));
    lastRunPlan = planAtRun;
    if (isCancelled(lastRun.error)) { lastRun = null; showResults(false); return toast("Simulation abgebrochen."); }
    renderResults(lastRun); document.documentElement.dataset.rendered = resultTab; rememberRun(lastRun);
    if (!lastRun.error) Steps.mark("run"); }
  catch (e) { plainTitle("Fehler"); $("#resultsBody").innerHTML = `<div class="card errcard">${esc(errText(e))}</div>`; }
}
const kpi = (k, v, s = "", cls = "") => `<div class="kpi ${cls}"><div class="k">${k}</div><div class="v">${v}</div><div class="s">${s}</div></div>`;
/** Legend; entry [color, text, opacity, line style]: with line style «solid» or «dashed» a line symbol instead of the box. */
const legend = items => `<div class="legend">${items.map(([c, t, o, line]) => `<span>${line
  ? `<svg class="lsym" width="22" height="10" viewBox="0 0 22 10" aria-hidden="true"><line x1="0" x2="22" y1="5" y2="5" stroke="var(${c})" stroke-width="2" ${line === "dashed" ? 'stroke-dasharray="5 3"' : ""}/></svg>`
  : `<i style="background:var(${c})${o ? `;opacity:${o}` : ""}"></i>`}${t}</span>`).join("")}</div>`;
const WEALTH_LEGEND = legend([["--c1", "Liquides Vermögen"], ["--c2", "Immobilien (netto)"], ["--c3", "Vorsorge (PK, 3a)"]]);
const atAge = month => `${ymShow(month)} (Alter ${ageStr(ym2i(month))})`;
/** Does the plan have mortgages whose renewal is checked for affordability? */
const hasProperties = () => state.realestate.properties.some(p => p.status !== "rent");
/** Property sales of a run, one row «Name: MM.YYYY» per sale; emergency sales with warning icon, without a sale «–». */
function propertySales(res) {
  const sales = (res?.propertyEvents || []).filter(e => e.kind === "sale" || e.kind === "emergencySale").sort((a, b) => ym2i(a.month) - ym2i(b.month));
  return sales.length ? sales.map(e => `<div>${esc(e.property)}: ${ymShow(e.month)}${e.kind === "emergencySale" ? ` ${houseWarn("Notverkauf")}` : ""}</div>`).join("") : "–";
}
/** Property events of a run, chronological: purchase, sale, emergency sale (with reason) and mortgage renewal with their amounts. */
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
 * 3-bucket strategy: derivation of the bucket sizes (spending, deducted income, net need, targets), return criterion,
 * triggered case (January, A, B, C), transfers and bucket balances before and after the refill (each before the spending of the month).
 * The refills of a year appear expandable in the yearly values (`yearTable`).
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
      <th title="Geplante Lebenshaltung; bei 3-Topf mit Guardrails die Lebenshaltung der Guardrails (vermerkt)">Lebenshaltung</th><th>Wohnen</th><th title="Einzahlungen in die Säule 3a">3a</th><th title="Steuer auf dem (hochgerechneten) Jahreseinkommen">Einkommens&shy;steuer</th><th title="abgezogene Einnahmen: AHV-Rente pro Jahr">AHV</th><th title="abgezogene Einnahmen: PK-Rente pro Jahr">PK-Rente</th><th title="abgezogene Einnahmen: Erwerbseinkommen pro Jahr">Erwerb</th><th title="Lebenshaltung + Wohnen + 3a + Einkommenssteuer − AHV − PK-Rente − Erwerbseinkommen">Netto-Bedarf</th>
      <th>Ziel Topf 1</th><th>Ziel Topf 2</th><th class="l" title="Aktienrendite der letzten 12 Monate (vor dem Start aus dem Vorlauf des Renditepfads) gegenüber der Schwelle: Nur bei erfüllter Bedingung wird in der Januar-Auffüllung und in C1 aus Topf 3 verkauft. A und C2 ignorieren sie.">Renditekriterium</th><th class="l" title="Umbuchungen zwischen den Töpfen">Umbuchungen</th><th title="Topf 1 / Topf 2 / Topf 3 vor der Auffüllung">Vorher T1 / T2 / T3</th><th title="Topf 1 / Topf 2 / Topf 3 nach der Auffüllung">Nachher T1 / T2 / T3</th></tr>${rows}</table></div>`;
}
const bucketHint = () => `<p class="hint">Beträge in ${cur()}, nominal, pro Jahr; die Topfstände gelten vor den Ausgaben des Monats. <b>Netto-Bedarf</b> = Lebenshaltung + Wohnen (ohne Amortisation) + Einzahlung in die Säule 3a + Einkommenssteuer − AHV − PK-Rente − Erwerbseinkommen, aus den geplanten Werten des laufenden Jahres (bei 3-Topf mit Guardrails: Lebenshaltung gemäss Guardrails, nach Unter- und Obergrenze); einmalige Ausgaben, Anschaffungen, Käufe und Sondersteuern zählen nicht. Ziel Topf 1 und 2 = Jahre × Netto-Bedarf. <b>Januar-Auffüllung:</b> Topf 1 zuerst aus Topf 2, der Rest aus Topf 3, danach Topf 2 aus Topf 3; ist das Renditekriterium nicht erfüllt, wird nichts aus Topf 3 verkauft und Topf 2 nicht nachgefüllt. Ein Überschuss in Topf 1 oder 2 geht in Topf 3. <b>A:</b> beim Start der Entnahmephase (erste Pensionierung), bei einem PK-Kapitalbezug und beim Neustart nach «alle Töpfe leer»: Das Vermögen (Kapitalbezug) geht zuerst in Topf 3, dann wie im Januar ohne Renditekriterium; die Ziele sind um die vergangenen Monate des Jahres gekürzt, der Netto-Bedarf aus den Monaten ab dem Auslöser auf 12 Monate hochgerechnet. <b>B:</b> Zuflüsse gehen direkt an ihr Ziel und erscheinen als Buchung «extern → Topf 3» (einmalige Zuflüsse, 3a, Verkaufserlöse ohne Folgekauf, Notverkäufe), «extern → Reservierung» (Verkauf mit Kauf im Folgemonat) oder «Reservierung → Topf 3» (Überschuss nach dem Kauf); Topf 1 und 2 werden nicht aufgefüllt. «Vorher» zeigt den Stand vor dem Zufluss. <b>C (Vorausschau):</b> Deckt Topf 1 die Ausgaben des Monats nicht, gilt bei erfülltem Renditekriterium <b>C1</b>: Ablauf wie die Januar-Auffüllung, die Ziele um die seit Januar vergangenen Monate gekürzt und das Ziel von Topf 1 um die einmaligen Ausgaben des Monats erhöht, ohne Überschussbuchung in Topf 3. Bei nicht erfülltem Kriterium gilt <b>C2</b>: Topf 1 wird aus Topf 2, dann Topf 3 so aufgefüllt, dass er die Ausgaben deckt und danach noch (Min(Reserve, Monate bis Januar) − 1) Monatsbedarfe enthält; Topf 2 wird nicht aufgefüllt. Die Überschussbuchung in Topf 3 gibt es nur in der Januar-Auffüllung. Reichen alle Töpfe nicht, wird eine Immobilie notverkauft; ohne Immobilie darf Topf 1 negativ werden, bis das Vermögen wieder positiv ist.</p>`;
/** 3-bucket bookkeeping (refills, inflows, reservations) is in the yearly values; the messages of the deficit phase stay. */
const isBucketBooking = e => (e.kind === "buckets" && !/^3-Topf: alle Töpfe leer/.test(e.text))
  || (state.withdrawal.strategy === "three_buckets" && e.kind === "withdrawal" && /reserviert für Kauf im Folgemonat/.test(e.text));
function eventList(evs) {
  evs = evs.filter(e => !isBucketBooking(e));
  return evs.length ? `<ul class="evlist">${evs.map(e => `<li><span>${atAge(e.month)}</span><span>${esc(e.text)}</span></li>`).join("")}</ul>` : `<p class="hint" style="margin:0">Keine Ereignisse.</p>`;
}
/** Amount nominal or in prices of the start month. */
const rv = (r, x) => realView ? x / r.priceIndex : x;
const realNote = () => realView ? `in heutiger Kaufkraft (Preise ${ymShow(i2ym(startM()))})` : "nominal";
const xLabel = m => `${m / 12} · ${Math.round(ageAt(m))} J.`;
const card = (title, sub, body, cls = "") => `<div class="card ${cls}"><h3>${title}${sub ? ` <span class="sub">${sub}</span>` : ""}</h3>${body}</div>`;
/**
 * Zoom of the result charts, shared by all: { from, to, years } in months since plan start; `years` is the chosen
 * width of a button, null for a mouse selection (custom); null = whole period.
 */
const resultView = () => (resultZoom ? [startM() + resultZoom.from, startM() + resultZoom.to] : null);
const redrawResults = () => { if (compareOpen) compareRuns(); else if (lastRun) renderResults(lastRun); };
/** Button «N J.»: width N years, centered on the middle of the current range (without zoom: of the duration), kept within the period. */
function stepZoom(years) {
  const total = endM() - startM(), w = years * 12;
  if (!years || w >= total) { resultZoom = null; return; }
  const mid = resultZoom ? (resultZoom.from + resultZoom.to) / 2 : total / 2;
  const from = clamp(Math.round(mid - w / 2), 0, total - w);
  resultZoom = { from, to: from + w, years };
}
/** Mouse selection (months since plan start): selections under one year are ignored, the whole duration resets. */
function setCustomZoom(from, to) {
  const total = endM() - startM(), a = Math.max(0, Math.round(from)), b = Math.min(total, Math.round(to));
  if (b - a < 12) return;
  resultZoom = a <= 0 && b >= total ? null : { from: a, to: b, years: null };
  redrawResults();
}
const zoomAbs = (a, b) => setCustomZoom(a - startM(), b - startM());
/** Zoom buttons of the result charts; the same state in all charts. After a mouse selection no button is active. */
const resultZoomBar = () => {
  const custom = resultZoom && resultZoom.years == null, v = resultView();
  const note = custom ? `Benutzerdefiniert: ${((v[0] - startM()) / 12).toLocaleString("de-CH", { maximumFractionDigits: 1 })} – ${((v[1] - startM()) / 12).toLocaleString("de-CH", { maximumFractionDigits: 1 })} J. nach Start` : "Bereich mit der Maus aufziehen · gilt für alle Ergebnisgrafiken";
  return `<div class="minizoom">${zoomButtons("res-zoom", resultZoom ? (resultZoom.years ?? -1) : 0)}<span>${note}</span></div>`;
};
const chartBox = (id, h = 300, zoom = false) => `${zoom ? resultZoomBar() : ""}<div class="chartwrap"><svg id="${id}" class="bigchart" style="height:${h}px"></svg><div id="${id}-tip" class="tip"></div></div>`;
/** Note on the final wealth per percentile in today's purchasing power (comparison, #126). */
const REAL_PCT_HINT = "In heutiger Kaufkraft gilt je Perzentil das Endvermögen des Laufs dieses Perzentils";
const FAN_LEGEND = legend([["--c1", "5.–95. Perzentil", ".25"], ["--c1", "25.–75. Perzentil", ".5"], ["--c1", "Median", null, "solid"], ["--warn", "1. und 99. Perzentil", null, "dashed"], ["--err", "Schlechtester Wert", null, "solid"]])
  + `<p class="hint">Die Linie «Schlechtester Wert» zeigt für jeden Monat den tiefsten Wert aller Läufe; das kann in jedem Monat ein anderer Lauf sein. Der einzelne schlechteste Lauf (nach Endvermögen) ist unter «Einzelne Läufe» abrufbar.</p>`;
const pName = p => p === 0 ? "Schlechtester Lauf" : `${p}. Perzentil`;
const pctTxt = v => `${(v * 100).toLocaleString("de-CH", { maximumFractionDigits: 1 })} %`;

/**
 * Yearly values: balances at year end, flows as the monthly average of the year, taxes as yearly sum.
 * `start`: total and liquid wealth at the simulation start (basis of the withdrawal rate in the first year).
 * `refills`: 3-bucket refills of the run; years with a refill have an expandable row with the derivation.
 */
/**
 * Values of the yearly values per calendar year (nominal or in today's purchasing power depending on the toggle). Basis of the table
 * «Jahreswerte» and of the yearly values in the comparison (#115), so that both show the same numbers.
 */
function yearValues(recs, start) {
  const by = new Map();
  for (const r of recs) { const y = r.month.slice(0, 4); if (!by.has(y)) by.set(y, []); by.get(y).push(r); }
  const sum = (rs, f) => rs.reduce((a, r) => a + rv(r, f(r)), 0), avg = (rs, f) => sum(rs, f) / rs.length;
  let prevLast = null;
  return [...by.entries()].map(([y, rs]) => {
    const last = rs[rs.length - 1];
    const planned = avg(rs, r => r.plannedSpending), housing = avg(rs, r => r.housing), income = avg(rs, r => r.income), draw = avg(rs, r => r.withdrawal);
    // #52: Contributions to pillar 3a count as spending.
    const p3aIn = avg(rs, r => r.pillar3aIn ?? 0);
    const spending = planned + housing + p3aIn, receipts = income + draw;
    const prev = prevLast ? prevLast.priceIndex : 1;
    /** Withdrawal rate: yearly withdrawal (nominal, incomplete years extrapolated to 12 months) / wealth at the start of the year. */
    const yearDraw = rs.reduce((a, r) => a + r.withdrawal, 0) / rs.length * 12;
    const baseTotal = prevLast ? prevLast.total : start?.total;
    prevLast = last;
    const taxIncome = sum(rs, r => r.assessedIncome ?? 0), taxWealth = sum(rs, r => r.assessedWealth ?? 0), taxCapital = sum(rs, r => r.taxCapital), taxGain = sum(rs, r => r.taxGain);
    return { y, months: rs.length, age: last.age, liquid: rv(last, last.liquid), property: rv(last, last.propertyEquity), pension: rv(last, last.pensionFund + last.pillar3a),
      total: rv(last, last.total), spending, draw, receipts, yearDraw, rate: yearDraw > 0 && baseTotal > 0 ? yearDraw / baseTotal : yearDraw > 0 ? NaN : null,
      deviation: receipts - spending, inflation: last.priceIndex / prev - 1, taxIncome, taxWealth, taxCapital, taxGain, taxTotal: taxIncome + taxWealth + taxCapital + taxGain };
  });
}
const rateTxt = r => r == null ? "" : Number.isNaN(r) ? "–" : `${(r * 100).toLocaleString("de-CH", { minimumFractionDigits: 1, maximumFractionDigits: 1 })} %`;
function yearTable(recs, start, refills = []) {
  const cell = (v, bold) => `<td>${Math.abs(v) < 0.5 ? "" : bold ? `<b>${fmtInt(v)}</b>` : fmtInt(v)}</td>`;
  const tx = showTaxCols, cols = 12 + (tx ? 5 : 0);
  const refillsOf = y => refills.filter(r => r.month.slice(0, 4) === y);
  const rows = yearValues(recs, start).map(v => {
    const y = v.y, part = v.months < 12 ? `<span title="Unvollständiges Jahr: ${v.months} Monate" style="color:var(--muted)">*</span>` : "";
    const rf = refillsOf(y), open = bucketOpen.has(y);
    const toggle = rf.length ? `<button class="btn ghost small bkbtn" data-action="bk-toggle" data-y="${y}" aria-expanded="${open}" title="Herleitung der 3-Topf-Auffüllungen dieses Jahres ${open ? "ausblenden" : "anzeigen"}">${open ? "▾" : "▸"}</button>` : "";
    return `<tr><td>${toggle}${y}${part}</td><td>${v.age.toFixed(1)}</td><td>${fmtInt(v.liquid)}</td><td>${fmtInt(v.property)}</td><td>${fmtInt(v.pension)}</td><td><b>${fmtInt(v.total)}</b></td>
      ${cell(v.spending, true)}${cell(v.draw)}${cell(v.receipts, true)}<td>${rateTxt(v.rate)}</td>${cell(v.deviation)}<td>${(v.inflation * 100).toLocaleString("de-CH", { minimumFractionDigits: 1, maximumFractionDigits: 1 })} %</td>
      ${tx ? `${cell(v.taxIncome)}${cell(v.taxWealth)}${cell(v.taxCapital)}${cell(v.taxGain)}${cell(v.taxTotal, true)}` : ""}</tr>`
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

/** Sums per calendar year for the bar charts. */
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

/** Charts of a single run (single run or percentile run). */
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
      mktSwitch()
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
  // Bar charts: only the years that touch the zoom range
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
    // #114: Reference line at start = 100 or at 0 % change
    ref: mktPct ? 0 : 100,
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

// ---------- Monte Carlo and multiple runs ----------
function histQuantile(h, q) {
  if (!h) return NaN;
  const total = h.counts.reduce((a, b) => a + b, 0); let acc = 0;
  for (let i = 0; i < h.counts.length; i++) { acc += h.counts[i]; if (acc >= q * total) return (h.edges[i] + h.edges[i + 1]) / 2; }
  return h.edges[h.edges.length - 1];
}
// #121: Axis of the chart «Wahrscheinlichkeit, dass das Geld reicht»: default 80–100 %, switchable to 0–100 %;
// the choice applies in the single result and in the comparison until it is changed.
const SURV_LO = 80;
let survFull = false;
const survRange = () => [survFull ? 0 : SURV_LO, 100];
const survSwitch = () => `<div class="seg" style="margin-bottom:8px"><button class="${survFull ? "" : "on"}" data-action="surv-scale" data-v="80">80–100 %</button><button class="${survFull ? "on" : ""}" data-action="surv-scale" data-v="0">0–100 %</button></div>`;
const survClipNote = min => !survFull && min < SURV_LO
  ? `<p class="hint survclip">Die Kurve fällt unter ${SURV_LO} % (tiefster Wert ${min.toLocaleString("de-CH", { maximumFractionDigits: 1 })} %) und ist unten abgeschnitten. «0–100 %» zeigt sie ganz.</p>` : "";
/** #122: Switches of the chart «Aktien, Obligationen und Inflation», the same in the single result and in the comparison. */
const mktSwitch = () => `<div class="seg" style="margin-bottom:8px"><button class="${mktPct ? "" : "on"}" data-action="mkt" data-v="idx">Wertentwicklung (Start = 100)</button><button class="${mktPct ? "on" : ""}" data-action="mkt" data-v="pct">Änderung in %</button></div>`
  + (mktPct ? "" : ` <div class="seg" style="margin-bottom:8px"><button class="${mktLog ? "" : "on"}" data-action="mkt-log" data-v="0">Linear</button><button class="${mktLog ? "on" : ""}" data-action="mkt-log" data-v="1">Logarithmisch</button></div>`);
function batchChartsHtml(b) {
  const st = b.stats;
  return card("Bandbreite des Gesamtvermögens", `${realNote()}, über alle Läufe`, chartBox("mc-fan", 300, true)
      + FAN_LEGEND)
    + card("Wahrscheinlichkeit, dass das Geld reicht", "Anteil der Läufe, in denen das Vermögen bis zu diesem Zeitpunkt reicht", survSwitch() + chartBox("mc-surv", 260, true) + survClipNote(Math.min(...st.survival) * 100))
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
  Charts.time($("#mc-surv"), $("#mc-surv-tip"), { x, xFmt: xLabel, view: resultView(), onZoom: zoomAbs, yFmt: v => `${v} %`, range: survRange(),
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

// ---------- Scenarios ----------
const SWEEP_SHOW = { pct: v => `${fmtPct(v)} %`, month: v => ymShow(v) };
const sweepType = p => SWEEP_PARAMS.find(x => x.id === p)?.type || "pct";
const sweepLabel = p => SWEEP_PARAMS.find(x => x.id === p)?.label || p;
/** Current value of a variation parameter in the plan (undefined if it is not present as a single value). */
function sweepNow(id) {
  const one = v => (v && typeof v === "object" ? (v.mode === "const" ? v.value : undefined) : v);
  if (id === "wealth.stockShare") return one(state.wealth.stockShare);
  if (id === "pension.pk.lumpsum") return one(state.pension.pk.lumpsum);
  if (id === "returns.startDate") return state.returns.startDate;
  if (id === "flows.spendingFactor") return 1;
  if (id === "withdrawal.rate") return state.withdrawal.fixedRate?.rate;
  if (id === "milestone.retire") { const m = state.milestones.find(x => x.id === "retire"); return m?.kind === "date" ? m.date : undefined; }
}
/** The values of the variations that apply to this run (e.g. «Aktienanteil 60 %, Pensionierungsdatum 03.2027»). */
function variationText() {
  return state.scenarios.sweeps.map(s => {
    const v = sweepNow(s.param);
    return v == null ? null : `${sweepLabel(s.param)} ${SWEEP_SHOW[sweepType(s.param)](v)}`;
  }).filter(Boolean).join(", ");
}
/** Random paths have no meaningful label; for a single run the values of the variations are shown instead. */
const randomPath = label => /^(Bootstrap-Pfad|Zufallspfad) \d+$/.test(label || "");
const runInfo = label => randomPath(label) ? variationText() : label;
/** Optimal value of a variation for a percentile, readable: «73 %» (step), «≈ 73 %» (interpolated), «< 50 %», «≥ 110 %». */
function optimumValueText(item) {
  if (item.kind === "na") return "–";
  const t = sweepType(item.param);
  const v = t === "pct" && typeof item.v === "number" ? Math.round(item.v * 1000) / 1000 : item.v;
  const s = SWEEP_SHOW[t](v);
  const pre = { exact: "", approx: "≈ ", below: "< ", atLeast: "≥ ", edgeLow: "≤ ", edgeHigh: "≥ " }[item.kind] ?? "";
  return pre + s + (item.reaches ? "" : " (reicht nirgends)");
}
/** Optimal values of all variations for the percentile (from the scenarios); empty without variations. */
function optimumText(sc, percent) {
  const o = Optimum.find(sc, percent);
  return o ? o.items.map(it => `${sweepLabel(it.param)} ${optimumValueText(it)}`).join(", ") : "";
}
// ----- Single run with variations: choose a variation step and view its run -----
let variationSel = null;          // chosen values per variation (null = plan values)
const variationRuns = new Map();  // selection → result of the single run of this step
const selKey = sel => JSON.stringify(sel);
const singleOf = r => (variationSel && variationRuns.get(selKey(variationSel))) || r.single;
const batchRuns = () => variationSel && variationRuns.get(selKey(variationSel));
const detailsOf = r => batchRuns()?.details || r.batch.details;
const resetVariation = () => { variationSel = null; variationRuns.clear(); };
/** Values of the variations for this run: the chosen step, otherwise the plan values. */
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
/** Calculated step for a selection; variations not chosen apply with their plan value. */
function levelOf(sc, sel) {
  const want = sc.params.map((p, i) => sel[i] ?? sweepNow(p));
  return sc.results.find(x => !x.error && x.values.every((v, i) => sameValue(v, want[i])));
}
/** Single runs and yearly values of a step: the runs of the percentiles are recalculated from the saved path numbers. */
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
    // If a final wealth differs from the saved value (e.g. changed time series), the display stays available, with a notice
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
/** Row for a percentile: optimal variation values, for historical runs additionally the start month. */
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

// ---------- Layout ----------
/**
 * #72: Classification of the success rate with the limits of the traffic light. Decisive is the displayed value rounded to 0.1 %:
 * 95.0 % is «hoch», 80.0 % «mittel».
 */
function rateClass(rate) {
  const shown = Number((rate * 100).toFixed(1));
  return shown >= 95 ? "hoch" : shown >= 80 ? "mittel" : "tief";
}
const helpLink = (topic, text, title = "") => `<a data-action="help-topic" data-topic="${topic}"${title ? ` title="${esc(title)}"` : ""}>${text}</a>`;
/** #72: Result in words above the key figures of the overview, formed from the same values as the key figures. */
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
    plainTitle("Simulation nicht möglich"); $("#resultsInfo").textContent = ""; $("#resultsTabs").innerHTML = "";
    $("#resultsBody").innerHTML = `<div class="card errcard"><p style="margin-top:0">${esc(r.error)}</p>
      <ul class="val">${r.issues.filter(i => i.severity === "error").map(i => `<li class="err" data-action="back"><span class="ic">!</span><span>${esc(i.message)}</span></li>`).join("")}</ul></div>`;
    return;
  }
  const tabs = [["overview", "Übersicht"], ["charts", "Grafiken"]];
  if (r.batch) tabs.push(["runs", "Einzelne Läufe"]);
  if (r.scenarios) tabs.push(["scenarios", "Szenarien"]);
  tabs.push(["table", "Jahreswerte"], ["params", "Parameter"]);
  if (!tabs.some(([k]) => k === resultTab)) resultTab = "overview";
  // Single run with variations: chosen step (otherwise the plan run)
  const sg = r.single ? singleOf(r) : null;
  setResultsTitle(sg ? ["Einzellauf", runInfo(sg.label), r.scenarios ? selectedVariationText() : ""].filter(Boolean).join(" · ") : `${fmtInt(r.batch.summary.runs)} Läufe`);
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
 * Parameters of a plan as sections [title, [[label, value], …]], structured like the editor sections.
 * Person names are intentionally missing (only the birth month), technical values only as far as they make the result reproducible.
 * `full` (#123, comparison of the plans): additionally the remaining inputs of the editor (affordability, inflation of the
 * properties, end of the mortgage, rest of pillar 3a, property gains tax by holding period).
 */
function paramSections(plan, full = false) {
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
  if (full) {
    const add = (title, rows) => { const sec = out.find(x => x[0] === title); if (sec) sec[1].push(...rows); else if (rows.length) out.push([title, rows]); };
    const a = plan.realestate?.affordability || {};
    add("Immobilien", [["Tragbarkeit: Höchstens vom Einkommen", pc(a.sustainability)], ["Tragbarkeit: Kalkulatorischer Zins", pc(a.imputedRate)],
      ["Tragbarkeit: Nebenkosten p.a. (vom Preis)", pc(a.fixCosts)], ["Tragbarkeit: Höchste Belehnung", pc(a.maxLoanToValue)],
      ...props.flatMap(p => { const nm = p.name || "Immobilie"; return [
        ...(p.status === "owned" ? [[`${nm}: Kaufdatum`, ymShow(p.bought)]] : []),
        ...(p.status === "rent" ? [[`${nm}: Miete ab`, at(p.from)], [`${nm}: Miete mit Inflation anpassen`, yn(p.inflationRent)]]
          : [[`${nm}: Wert mit Inflation anpassen`, yn(p.inflationValue)], [`${nm}: Nebenkosten mit Inflation anpassen`, yn(p.inflationFixCosts)]]),
        ...(p.mortgage?.tranches || []).map(t => [`${nm}: ${t.name || "Tranche"}: nach der letzten Periode`, t.after === "repay" ? "Restschuld zurückzahlen" : "wie letzte Periode verlängern"])]; })]);
    add("Säule 3a", (plan.pillar3a || []).map(x => [`${(plan.household?.persons || []).find(q => q.id === x.person)?.name || x.person}: Rest im AHV-Bezug`, x.use === "repay" ? "Hypothek" : "Vermögen"]));
    // Time-dependent values of the strategy as a list instead of «zeitabhängig», so that differences are visible
    const ws = plan.wealth || {}, strat = out.find(x => x[0] === "Entnahmestrategie");
    if (strat) strat[1] = strat[1].map(([k, v]) => k === "Cash-Reserve" ? [k, ser(ws.reserveMonths, x => `${x} Monate`)] : k.startsWith("Aktienanteil") ? [k, ser(ws.stockShare, pc)] : [k, v]);
    if (g) add("Steuerannahmen", [["Grundstückgewinnsteuer mit dem Steuerfuss multiplizieren", yn(g.useMultiplier)],
      ...(g.durationRates?.length ? [["Grundstückgewinnsteuer nach Besitzdauer", g.durationRates.map(([y, r]) => `ab ${y} J.: ${fmtPct(r)} %`).join("; ")]] : [])]);
  }
  return out;
}
/** Tab «Parameter»: read-only summary of the plan in the state of the run. */
function paramsHtml(plan) {
  if (!plan) return card("Parameter", "", `<p style="margin:0">Nicht verfügbar.</p>`);
  return `<p class="hint" style="margin:0 0 10px">Die Parameter im Stand dieses Laufs (nicht der aktuelle Stand im Editor). Nur zur Ansicht.</p>`
    + paramSections(plan).map(([title, rows]) => card(title, "", `<table class="tbl params">${rows.map(([k, v]) => `<tr><td class="l">${esc(k)}</td><td class="l">${esc(v)}</td></tr>`).join("")}</table>`)).join("");
}
/** Selection of the run (percentile) together with the values of the variations; sits above the scrolling area. */
function runPicker(r, res) {
  const info = [pName(detailP), percentInfo(r, detailP, res.label)].filter(Boolean).join(" · ");
  return card("Lauf wählen", "", `<div class="seg">${detailsOf(r).map(([p]) => `<button class="${p === detailP ? "on" : ""}" data-action="detail" data-p="${p}">${pName(p)}</button>`).join("")}</div>
    ${batchRuns()?.off?.length ? `<p class="hint" style="color:var(--warn,#b45309)">Hinweis: Das Endvermögen von ${batchRuns().off.map(pName).join(", ")} weicht von der Szenarientabelle ab (zum Beispiel, weil sich eine Zeitreihe geändert hat). Die Anzeige bleibt verfügbar.</p>` : ""}
    <p class="computed" style="margin-bottom:0"><b>${esc(info)}</b> · Endvermögen ${fmtMoney(res.summary.endTotal)}${res.summary.depleted ? ` · aufgebraucht ${atAge(res.summary.depleted)}` : ""}</p>`);
}
function showDetail(p) { detailP = p; bucketOpen.clear(); if (lastRun) renderResults(lastRun); }

// ---------- Saved simulations ----------
let savedView = null, parkedState = null;
/** #120: Maximum number of results in the comparison; each has its own color (--r1 … --r10 in index.html). */
const CMP_MAX = 10;
const runsSel = new Set(), RUN_COLORS = Array.from({ length: CMP_MAX }, (_, i) => `--r${i + 1}`);
const runDate = iso => new Date(iso).toLocaleString("de-CH", { dateStyle: "short", timeStyle: "short" });
const stratName = id => id === "three_buckets_guardrails" ? "3-Topf + Guardrails" : STRATS.find(x => x[0] === id)?.[1] || id;
const sourceName = id => SOURCES.find(x => x[0] === id)?.[1] || id;
/** Name of a new result: plan · source (for scenarios «Szenarien») · strategy · date, time. */
const runTitle = (base, m, iso) => [base, m.kind === "scenarios" ? "Szenarien" : SOURCE_SHORT[m.source] || sourceName(m.source), STRAT_SHORT[m.strategy] || stratName(m.strategy), runDate(iso)].join(" · ");
/** While viewing a saved simulation its plan applies; the editor state is set aside meanwhile. */
function unpark() { if (parkedState) { state = parkedState; parkedState = null; } savedView = null; }
/** Value of a time series of the plan as text: constant → value, otherwise «zeitabhängig». */
const serTxt = (v, f) => v && typeof v === "object" ? (v.mode === "const" ? f(v.value) : "zeitabhängig") : v == null ? "–" : f(v);
/** Parameters of the withdrawal strategy of a plan as pairs [label, value] (labels and units as in the editor). */
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
/** Name of the time series used (without folder and extension); `null` if the return source has no time series. */
function seriesLabel() {
  const r = state.returns;
  if (r.source === "fixed" || r.source === "parametric" || !seriesRef()) return null;
  return (seriesInfo && !seriesInfo.error && (seriesInfo.title || seriesInfo.name)) || (r.series ? seriesShort(r.series) : null);
}
/** Display of the column «Zeitreihe» (header data without a value: «–»). */
const seriesCell = m => m.seriesName === undefined ? "–" : m.seriesName ? esc(m.seriesName)
  : m.source === "fixed" ? `<span class="muted">feste Renditen</span>` : m.source === "parametric" ? `<span class="muted">parametrisch</span>` : "–";
/** Share of runs with an emergency sale from which the warning icon appears (configurable later, #25). */
const EMERGENCY_WARN_SHARE = 0.05;
/** Warning icon (house with warning sign) for the unplanned sale of a property; `tip` is the tooltip. */
const houseWarn = tip => `<span class="hwarn" role="img" aria-label="${esc(tip)}" title="${esc(tip)}">🏠⚠️</span>`;
/** Icon in the list of results: single run with emergency sale, Monte Carlo from the threshold; old entries without a value: none. */
function emergencyMark(m) {
  if (m.kind === "single") return m.emergencySaleRuns > 0 ? ` ${houseWarn("Notverkauf einer Immobilie in diesem Lauf")}` : "";
  if (m.kind !== "batch" || !(m.runs > 0) || !(m.emergencySaleRuns > 0)) return "";
  const share = m.emergencySaleRuns / m.runs;
  return share >= EMERGENCY_WARN_SHARE ? ` ${houseWarn(`Notverkauf einer Immobilie in ${pctTxt(share)} der Läufe (${fmtInt(m.emergencySaleRuns)} von ${fmtInt(m.runs)} Läufen)`)}` : "";
}
/** Strategy name with the parameters of the result as tooltip (also reachable by keyboard). */
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
    const now = new Date().toISOString();
    const meta = { id: Date.now().toString(36) + Math.random().toString(36).slice(2, 8), created: now, name: runTitle(base, m, now), note: "", ...m };
    await Runs.add(meta, { plan, result: r });
    // #125: the result just shown gets its name into the header
    if (lastRun === r && !savedView) { lastRunName = meta.name; if (resultHead) setResultsTitle(); }
    document.documentElement.dataset.saved = (+document.documentElement.dataset.saved || 0) + 1;
  } catch (e) {
    // #147: report visibly (e.g. browser storage full); the user interface stays usable, the result stays displayed
    toast("Ergebnis nicht in der Liste gespeichert: " + errText(e));
    noteError("Ergebnis nicht gespeichert: " + errText(e));
  }
}
let runsIds = [];
function runsBar() {
  const k = runsSel.size, n = runsIds.length, canCompare = k >= 2 && k <= CMP_MAX;
  $("#resultsTabs").innerHTML = `<button class="btn small" data-action="runs-compare" ${canCompare ? "" : "disabled"} title="Zwei bis ${CMP_MAX} Ergebnisse markieren">Vergleichen (${k})</button>${k > CMP_MAX ? `<span class="cmpmax hint" role="status">Höchstens ${CMP_MAX} Ergebnisse lassen sich vergleichen.</span>` : ""}
    <button class="btn small" data-action="runs-delete" ${k < 1 ? "disabled" : ""}>Markierte löschen (${k})</button>
    <button class="btn small" data-action="runs-delete-all" ${n < 1 ? "disabled" : ""}>Alle löschen</button>`;
  const all = document.querySelector('input[data-action="runs-all"]');
  if (all) { all.checked = n > 0 && k === n; all.indeterminate = k > 0 && k < n; }
}
async function openRuns() {
  showResults(true); lastRun = null; savedView = null; compareOpen = false;
  plainTitle("Gespeicherte Ergebnisse");
  let list = [];
  try { list = await Runs.list(); } catch (e) { noteError(errText(e)); }
  for (const id of [...runsSel]) if (!list.some(m => m.id === id)) runsSel.delete(id);
  runsIds = list.map(m => m.id);
  $("#resultsInfo").textContent = `${list.length} ${list.length === 1 ? "Eintrag" : "Einträge"} · nur auf diesem Rechner`;
  runsBar();
  const cell = m => (m.kind === "batch" ? `<span title="Median ${fmtMio(m.end50)}">${pctTxt(m.successRate)} · 5 %: ${fmtMio(m.end5)}</span>` : m.kind === "single" ? `Endvermögen ${fmtMio(m.endTotal)}` : "–") + emergencyMark(m);
  $("#resultsBody").innerHTML = list.length ? `<div class="card"><table class="tbl" id="runsTbl">
    <tr><th><input type="checkbox" data-action="runs-all" title="Alle markieren oder Markierung aufheben"></th><th>Name</th><th>Notiz</th><th>Datum</th><th>Zeitreihe</th><th>Strategie</th><th>Ergebnis</th><th></th></tr>
    ${list.map(m => `<tr><td><input type="checkbox" data-action="run-sel" data-id="${m.id}" ${runsSel.has(m.id) ? "checked" : ""} title="Zum Vergleichen (2 bis ${CMP_MAX}) oder Löschen markieren"></td>
      <td><input class="runedit" data-id="${m.id}" data-f="name" value="${esc(m.name)}" title="Name ändern"></td>
      <td><input class="runedit" data-id="${m.id}" data-f="note" value="${esc(m.note)}" placeholder="Notiz…"></td>
      <td>${esc(runDate(m.created))}</td><td>${seriesCell(m)}</td>${stratCell(m)}<td>${cell(m)}<br><small class="muted">${esc(runKind(m))}</small></td>
      <td style="white-space:nowrap"><button class="btn small" data-action="run-view" data-id="${m.id}">Öffnen</button> <button class="btn small" data-action="run-del" data-id="${m.id}" title="Dieses Ergebnis löschen">Löschen</button></td></tr>`).join("")}
    </table><p class="hint">Jedes Ergebnis wird automatisch hier abgelegt, mit der Planung von damals. Name und Notiz lassen sich direkt in der Liste ändern. Zum Löschen beliebig viele Einträge markieren (oder oben «Alle löschen»); zum Vergleichen zwei bis ${CMP_MAX} Einträge markieren.</p></div>`
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
// ---------- Comparison of saved results (#115) ----------
// Tabs as for the single result: overview (key figures, range, percentiles, probability, depletion),
// trajectories (charts of the single run or median run), yearly values (one column per result) and plan (differences).
// The results are aligned by months or years since their start.
let compareTab = "overview", cmpYearKey = "total", cmpLiquid = "liquid", cmpFlow = "in", cmpP = 50;
/** #118: Sorting of the key figures in the comparison: { col, dir } (dir 1 ascending, −1 descending); null = order of the list. */
let cmpSort = null;
const CMP_TABS = [["overview", "Übersicht"], ["charts", "Verläufe"], ["table", "Jahreswerte"], ["plan", "Planung"]];
const CMP_YEAR = [["total", "Gesamtvermögen (Total)"], ["liquid", "Liquides Vermögen"], ["property", "Immobilien"], ["pension", "Vorsorge"],
  ["receipts", "Einkommen total"], ["spending", "Ausgaben total"], ["draw", "Entnahme"], ["rate", "Entnahmerate"], ["taxTotal", "Steuern total"]];
const CMP_LIQUID = [["liquid", "Total"], ["stocks", "Aktien"], ["bonds", "Obligationen"], ["cash", "Cash"]];
const CMP_FLOW = [["in", "Einnahmen"], ["out", "Ausgaben"], ["taxes", "Steuern"]];
/**
 * What a saved result provides for the comparison. `run` is the run for trajectories and yearly values: for a single run
 * this one, for multiple runs the run of the chosen percentile (`cmpP`, like «Lauf wählen» in the single result); if this
 * percentile is missing, the median run (`pMissing`).
 */
function cmpItem(m, d, i) {
  const res = d?.result || {}, batch = res.batch || null, single = res.single || null;
  const det = q => batch?.details?.find(([p]) => p === q)?.[1];
  const exact = batch ? det(cmpP) : null;
  const run = single || exact || (batch && (det(50) || batch.details?.[0]?.[1])) || null;
  return { m, i, color: RUN_COLORS[i], name: m.name, batch, single, run, pMissing: !!batch && !exact, scenariosOnly: !batch && !single };
}
/** Selection of the percentile for trajectories and yearly values; only for results with multiple runs. */
function cmpPicker(items) {
  const qs = [...new Set(items.filter(it => it.batch).flatMap(it => (it.batch.details || []).map(([p]) => p)))].sort((a, b) => a - b);
  if (!qs.length) return "";
  if (!qs.includes(cmpP)) cmpP = qs.includes(50) ? 50 : qs[0];
  return card("Lauf wählen", "gilt für alle Ergebnisse mit mehreren Läufen", `<div class="seg">${qs.map(p => `<button class="${p === cmpP ? "on" : ""}" data-action="cmp-p" data-p="${p}">${pName(p)}</button>`).join("")}</div>`);
}
/** Which run is shown per result, with a remark on single runs and missing percentiles. */
function cmpRunNotes(items) {
  const runs = items.filter(it => it.run), out = [];
  if (runs.some(it => it.batch)) out.push(`Bei mehreren Läufen: ${pName(cmpP)}${cmpP === 50 ? " (Median)" : ""}.`);
  const singles = runs.filter(it => it.single);
  if (singles.length) out.push(`Einzellauf (ohne Perzentile): ${singles.map(it => esc(it.name)).join(", ")}.`);
  const miss = runs.filter(it => it.pMissing);
  if (miss.length) out.push(`Ohne ${pName(cmpP)}, deshalb Median-Lauf: ${miss.map(it => esc(it.name)).join(", ")}.`);
  items.filter(it => !it.run).forEach(it => out.push(`Ohne Lauf (nur Szenarien): ${esc(it.name)}.`));
  return out;
}
const cmpSel = (action, opts, cur) => `<div class="seg" style="margin-bottom:8px">${opts.map(([k, t]) => `<button class="${k === cur ? "on" : ""}" data-action="${action}" data-v="${k}">${t}</button>`).join("")}</div>`;
const cmpNote = list => list.length ? `<p class="hint cmpnote">${list.join(" ")}</p>` : "";
const cmpLegend = items => legend(items.map(it => [it.color, esc(it.name)]));
/** Series of equal length (months since start); missing months stay gaps. */
const cmpPad = (n, v) => Array.from({ length: n }, (_, k) => k < v.length ? v[k] : NaN);
const cmpX = n => Array.from({ length: n }, (_, k) => k);
const cmpTime = (id, n, lines, tip, extra = {}) => Charts.time($(`#${id}`), $(`#${id}-tip`), { x: cmpX(n), xFmt: m => `${m / 12} J.`,
  view: resultZoom ? [resultZoom.from, resultZoom.to] : null, onZoom: setCustomZoom, lines, tip: k => `<b>Jahr ${(k / 12).toLocaleString("de-CH", { maximumFractionDigits: 1 })}</b>${tip(k)}`, ...extra });
const cmpRow = (it, v, f) => Number.isFinite(v) ? `<br><span style="color:var(${it.color})">■</span> ${esc(it.name)}: ${f(v)}` : "";
/** Calendar years equal: year number; otherwise «n. Jahr» since start (the year number is in the tooltip of the cell). */
function cmpYears(items, valuesOf) {
  const per = items.map(it => it.run ? valuesOf(it) : null);
  const firsts = per.filter(Boolean).map(v => v[0]?.y);
  const same = firsts.every(y => y === firsts[0]);
  const n = Math.max(0, ...per.map(v => v ? v.length : 0));
  return { per, n, same, label: k => same ? (per.find(v => v && v[k])?.[k].y ?? "") : `${k + 1}. Jahr` };
}
async function compareRuns() {
  const metas = (await Runs.list()).filter(m => runsSel.has(m.id)).reverse().slice(0, CMP_MAX);
  const datas = await Promise.all(metas.map(m => Runs.get(m.id)));
  lastRun = null; savedView = null; compareOpen = true;
  const items = metas.map((m, i) => cmpItem(m, datas[i], i));
  if (!CMP_TABS.some(([k]) => k === compareTab)) compareTab = "overview";
  plainTitle(`Vergleich von ${metas.length} Ergebnissen`); $("#resultsInfo").textContent = realView ? "heutige Kaufkraft" : "nominal";
  $("#resultsPick").innerHTML = "";
  $("#resultsTabs").innerHTML = `<button class="btn small" data-action="runs-open">← Zur Liste</button>
    <div class="seg tabs">${CMP_TABS.map(([k, t]) => `<button class="${k === compareTab ? "on" : ""}" data-action="cmp-tab" data-tab="${k}">${t}</button>`).join("")}</div>
    <div class="seg"><button class="${realView ? "" : "on"}" data-action="real" data-v="0" title="Beträge wie sie dann anfallen">Nominal</button><button class="${realView ? "on" : ""}" data-action="real" data-v="1" title="Beträge in Preisen des jeweiligen Simulationsstarts (jedes Ergebnis mit seiner eigenen Inflation)">Heutige Kaufkraft</button></div>`;
  const body = $("#resultsBody");
  if (compareTab === "overview") cmpOverview(body, items);
  else if (compareTab === "charts") cmpCharts(body, items);
  else if (compareTab === "table") cmpTable(body, items);
  else cmpPlan(body, items, datas);
  document.documentElement.dataset.rendered = "compare";
}
const cmpSw = it => `<i class="sw" style="background:var(${it.color})"></i>`;
const cmpReal = () => realView ? "heutige Kaufkraft (jedes Ergebnis mit seiner eigenen Inflation)" : "nominal";
function cmpOverview(body, items) {
  const num = v => v == null || !Number.isFinite(v) ? "–" : fmtMio(v);
  /** Final wealth of a percentile from the summary, converted to real with the inflation of the run of this percentile.
   *  Results before #126 have the real value only in the detail runs (0, 1, 5, 50, 95, 99). */
  const endAt = (it, q) => {
    if (it.single) return q === 50 ? (realView ? it.single.summary.endTotalReal : it.single.summary.endTotal) : null;
    if (!it.batch) return null;
    const pc = it.batch.summary.percentiles.find(p => p.percent === q);
    if (!realView) return pc?.endTotal ?? null;
    return pc?.endTotalReal ?? it.batch.details?.find(([p]) => p === q)?.[1]?.summary.endTotalReal ?? null;
  };
  const notes = [];
  // #118: Key figures sortable by column; colors and order of the other cards stay
  const depAt = it => { const d = it.single ? it.single.summary.depleted : it.batch ? it.batch.summary.earliestDepletion?.[0] : undefined;
    return !it.single && !it.batch ? null : d ? ym2i(d) : Infinity; };
  const cols = [["name", "Ergebnis", "l", it => it.name], ["series", "Zeitreihe", "l", it => it.m.seriesName || (it.m.source === "fixed" ? "feste Renditen" : it.m.source === "parametric" ? "parametrisch" : null)],
    ["strat", "Strategie", "l", it => stratName(it.m.strategy)], ["succ", "Erfolgsquote", "r", it => it.batch ? it.batch.summary.successRate : null],
    ["p50", "Median bzw. Endvermögen", "r", it => endAt(it, 50)], ["p5", "5. Perzentil", "r", it => endAt(it, 5)], ["p1", "1. Perzentil", "r", it => endAt(it, 1)],
    ["p0", "Schlechtester Lauf", "r", it => endAt(it, 0)], ["dep", "Früheste Erschöpfung", "r", depAt]];
  const sc = cmpSort && cols.find(c => c[0] === cmpSort.col);
  const sorted = sc ? sortBy(items, sc[3], cmpSort.dir) : items;
  const head = cols.map(([k, t, al]) => { const on = sc && sc[0] === k, dir = on ? cmpSort.dir : 0;
    return `<th class="${al} sorth" aria-sort="${dir > 0 ? "ascending" : dir < 0 ? "descending" : "none"}"><button class="sortbtn" data-action="cmp-sort" data-col="${k}" title="Nach ${esc(t)} sortieren${on ? (dir > 0 ? " (absteigend)" : " (aufsteigend)") : ""}">${t}<span class="sorti${on ? " on" : ""}" aria-hidden="true">${dir > 0 ? "▲" : dir < 0 ? "▼" : "↕"}</span></button></th>`; }).join("");
  const rows = sorted.map(it => { const m = it.m; return `<tr><td class="l">${cmpSw(it)}${esc(m.name)}<br><small class="muted">${esc(runKind(m))}</small></td><td class="l">${seriesCell(m)}</td>${stratCell(m, " l")}
    <td class="r">${it.batch ? pctTxt(it.batch.summary.successRate) : "–"}</td>
    <td class="r">${num(endAt(it, 50))}</td><td class="r">${num(endAt(it, 5))}</td><td class="r">${num(endAt(it, 1))}</td><td class="r">${num(endAt(it, 0))}</td>
    <td class="r">${it.single ? (it.single.summary.depleted ? ymShow(it.single.summary.depleted) : "keine") : it.batch ? (it.batch.summary.earliestDepletion ? ymShow(it.batch.summary.earliestDepletion[0]) : "keine") : "–"}</td></tr>`; }).join("");
  const scen = items.filter(it => it.scenariosOnly);
  if (scen.length) notes.push(`Nur als Kennzahl (Ergebnis mit Szenarien ohne Einzel- oder Monte-Carlo-Lauf): ${scen.map(it => esc(it.name)).join(", ")}.`);
  // Range: median (or single run) and 5th/95th percentile
  const bandOf = it => { if (!it.batch) return null; const b = it.batch.bands; return realView ? b.real || null : b; };
  const noReal = items.filter(it => it.batch && realView && !it.batch.bands.real);
  if (noReal.length) notes.push(`Ohne Werte in heutiger Kaufkraft (älteres Ergebnis): ${noReal.map(it => esc(it.name)).join(", ")}.`);
  const partReal = realView ? items.filter(it => it.batch && it.batch.bands.real && it.batch.summary.percentiles.some(p => p.endTotalReal == null)) : [];
  if (partReal.length) notes.push(`In heutiger Kaufkraft nur für den schlechtesten Lauf, das 1., 5., 50., 95. und 99. Perzentil (älteres Ergebnis): ${partReal.map(it => esc(it.name)).join(", ")}.`);
  const singles = items.filter(it => it.single);
  if (singles.length) notes.push(`Einzellauf ohne Perzentile (in der Bandbreite nur der eine Lauf; keine Perzentile, keine Wahrscheinlichkeit und kein Alter bei Erschöpfung): ${singles.map(it => esc(it.name)).join(", ")}.`);
  const fan = items.map(it => it.single ? { med: it.single.records.map(r => rv(r, r.total)) } : bandOf(it) ? { med: bandOf(it).p50, lo: bandOf(it).p5, hi: bandOf(it).p95 } : null);
  const n = Math.max(0, ...fan.map(f => f ? f.med.length : 0));
  const fanLines = [];
  fan.forEach((f, k) => { if (!f) return; fanLines.push({ color: items[k].color, values: cmpPad(n, f.med) });
    if (f.lo) fanLines.push({ color: items[k].color, dash: "5 3", width: 1.2, values: cmpPad(n, f.lo) }, { color: items[k].color, dash: "2 3", width: 1.2, values: cmpPad(n, f.hi) }); });
  // Percentiles: all percentiles of the results as rows
  const batches = items.filter(it => it.batch);
  const qs = [...new Set(batches.flatMap(it => it.batch.summary.percentiles.map(p => p.percent)))].sort((a, b) => b - a);
  const pTable = batches.length ? `<div class="tscroll"><table class="tbl num"><tr><th class="l">Perzentil</th>${batches.map(it => `<th class="cmph">${cmpSw(it)}${esc(it.name)}</th>`).join("")}</tr>
    ${qs.map(q => `<tr><td class="l">${q === 0 ? "Schlechtester Lauf" : q + "."}</td>${batches.map(it => { const v = endAt(it, q); return `<td>${v == null ? "–" : fmtInt(v)}</td>`; }).join("")}</tr>`).join("")}</table></div>`
      + (realView ? `<p class="hint">${REAL_PCT_HINT}; die Reihenfolge kann deshalb von der nominalen abweichen.</p>` : "")
    : `<p class="hint" style="margin:0">Kein Ergebnis mit mehreren Läufen.</p>`;
  // Probability that the money lasts
  const surv = items.map(it => it.batch ? it.batch.stats.survival.map(v => v * 100) : null);
  const ns = Math.max(0, ...surv.map(v => v ? v.length : 0));
  // Age at depletion: share of runs per year of age (results with different numbers of runs stay comparable)
  const ages = [...new Set(batches.flatMap(it => it.batch.stats.depletionAges.map(([a]) => a)))].sort((a, b) => a - b);
  const depShare = it => ages.map(a => (it.batch.stats.depletionAges.find(([x]) => x === a)?.[1] || 0) / it.batch.summary.runs);
  body.innerHTML = card("Kennzahlen", `Endvermögen ${cmpReal()}`, `<div class="tscroll"><table class="tbl cmpkpi"><tr>${head}</tr>${rows}</table></div>`
      + (realView ? `<p class="hint">${REAL_PCT_HINT}.</p>` : ""))
    + cmpNote(notes)
    + (fanLines.length ? card("Bandbreite des Gesamtvermögens", `${cmpReal()} · durchgezogen: Median bzw. Einzellauf, gestrichelt: 5. Perzentil, gepunktet: 95. Perzentil`, chartBox("cmp-wealth", 300, true) + cmpLegend(items.filter((_, k) => fan[k]))) : "")
    + card("Endvermögen nach Perzentil", cmpReal(), pTable)
    + (batches.length ? card("Wahrscheinlichkeit, dass das Geld reicht", "Anteil der Läufe, in denen das Vermögen bis zu diesem Zeitpunkt reicht", survSwitch() + chartBox("cmp-surv", 260, true) + cmpLegend(batches)
      + survClipNote(Math.min(...surv.filter(Boolean).flat()))) : "")
    + (batches.length ? card("Alter bei Erschöpfung", "Anteil der Läufe, in denen das Vermögen in diesem Altersjahr aufgebraucht ist",
      ages.length ? chartBox("cmp-dep", 220) + cmpLegend(batches) : `<p class="hint" style="margin:0">In keinem Lauf aufgebraucht.</p>`) : "");
  if (fanLines.length) cmpTime("cmp-wealth", n, fanLines, k => fan.map((f, j) => f ? cmpRow(items[j], cmpPad(n, f.med)[k], fmtMoney) : "").join(""), { yFmt: fmtMio });
  if (batches.length) cmpTime("cmp-surv", ns, surv.map((v, j) => v && { color: items[j].color, values: cmpPad(ns, v) }).filter(Boolean),
    k => surv.map((v, j) => v ? cmpRow(items[j], cmpPad(ns, v)[k], x => `${x.toLocaleString("de-CH", { maximumFractionDigits: 1 })} %`) : "").join(""), { yFmt: v => `${v} %`, range: survRange() });
  if (ages.length) {
    const shares = batches.map(depShare);
    Charts.bars($("#cmp-dep"), $("#cmp-dep-tip"), { cats: ages.map(String), minBar: 3, yFmt: v => `${(v * 100).toLocaleString("de-CH", { maximumFractionDigits: 1 })} %`,
      series: batches.map((it, j) => ({ group: String(j), color: it.color, values: shares[j] })),
      tip: k => `Alter <b>${ages[k]}</b>` + batches.map((it, j) => cmpRow(it, shares[j][k], pctTxt)).join("") });
  }
}
function cmpCharts(body, items) {
  const runs = items.filter(it => it.run);
  const notes = cmpRunNotes(items);
  if (!runs.length) { body.innerHTML = cmpNote(notes) || `<p class="hint">Keine Verläufe vorhanden.</p>`; return; }
  const hasProp = runs.filter(it => it.run.records.some(r => r.propertyValue > 0)), hasPk = runs.filter(it => it.run.records.some(r => r.pensionFund > 0 || r.pillar3a > 0));
  const noProp = runs.filter(it => !hasProp.includes(it)), noPk = runs.filter(it => !hasPk.includes(it));
  const n = Math.max(...runs.map(it => it.run.records.length));
  const series = (it, f) => cmpPad(n, it.run.records.map(r => rv(r, f(r))));
  const LQ = { liquid: r => r.liquid, stocks: r => r.stocks, bonds: r => r.bonds, cash: r => r.cash };
  // Yearly values of the bars: income, spending, taxes; withdrawal rate as in the single result
  const yrs = cmpYears(runs, it => yearly(it.run.records));
  const flowOf = a => cmpFlow === "in" ? a.earned + a.pk + a.ahv : cmpFlow === "out" ? a.living + a.housing + a.p3a + a.taxes : a.taxes;
  const rateOf = a => a.liquidStart > 0 ? Math.max(0, a.draw) / a.liquidStart * 12 / a.months : 0;
  const yearSeries = f => yrs.per.map(v => v ? Array.from({ length: yrs.n }, (_, k) => v[k] ? f(v[k]) : NaN) : null).filter(Boolean);
  const cats = Array.from({ length: yrs.n }, (_, k) => String(yrs.label(k)));
  const idx = (it, key) => cmpPad(n, it.run.records.map(r => 100 * r[key] / (realView ? r.priceIndex : 1)));
  body.innerHTML = cmpPicker(items) + cmpNote(notes)
    + card("Gesamtvermögen", cmpReal(), chartBox("cmp-total", 280, true) + cmpLegend(runs))
    + card("Liquides Vermögen nach Anlage", cmpReal(), cmpSel("cmp-liquid", CMP_LIQUID, cmpLiquid) + chartBox("cmp-liq", 280, true) + cmpLegend(runs))
    + card("Einnahmen und Ausgaben pro Jahr", `${cmpReal()} · Kalenderjahre, das erste und letzte Jahr sind unvollständig`, cmpSel("cmp-flow", CMP_FLOW, cmpFlow) + chartBox("cmp-flows", 260) + cmpLegend(runs))
    + card("Entnahme aus dem Vermögen", "pro Jahr: Ausgaben inkl. Steuern − Einnahmen, im Verhältnis zum liquiden Vermögen", chartBox("cmp-draw", 240) + cmpLegend(runs))
    + (hasProp.length ? card("Immobilien", `${cmpReal()} · Eigenkapital (Marktwert − Hypothek)`, chartBox("cmp-prop", 240, true) + cmpLegend(hasProp) + cmpNote(noProp.length ? [`Ohne Immobilie: ${noProp.map(it => esc(it.name)).join(", ")}.`] : [])) : "")
    + (hasPk.length ? card("Vorsorge", `${cmpReal()} · Pensionskasse und Säule 3a`, chartBox("cmp-pk", 240, true) + cmpLegend(hasPk) + cmpNote(noPk.length ? [`Ohne Vorsorgevermögen: ${noPk.map(it => esc(it.name)).join(", ")}.`] : [])) : "")
    + card("Aktien, Obligationen und Inflation", `${mktPct ? "Änderung gegenüber dem Vorjahresmonat in %" : "Wertentwicklung seit Start = 100"}, ${cmpReal()} · durchgezogen: Aktien, gestrichelt: Obligationen, gepunktet: Inflation${mktPct ? "" : " (Preisindex)"}`,
      mktSwitch() + chartBox("cmp-mkt", 260, true) + cmpLegend(runs));
  const money = (id, f, list = runs) => cmpTime(id, n, list.map(it => ({ color: it.color, values: series(it, f) })), k => list.map(it => cmpRow(it, series(it, f)[k], fmtMoney)).join(""), { yFmt: fmtMio });
  money("cmp-total", r => r.total);
  money("cmp-liq", LQ[cmpLiquid] || LQ.liquid);
  if (hasProp.length) money("cmp-prop", r => r.propertyEquity, hasProp);
  if (hasPk.length) money("cmp-pk", r => r.pensionFund + r.pillar3a, hasPk);
  const flows = yearSeries(flowOf), rates = yearSeries(rateOf);
  Charts.bars($("#cmp-flows"), $("#cmp-flows-tip"), { cats, yFmt: fmtMio, minBar: 3, series: runs.map((it, j) => ({ group: String(j), color: it.color, values: flows[j].map(v => Number.isFinite(v) ? v : 0) })),
    tip: k => `<b>${cats[k]}</b>` + runs.map((it, j) => cmpRow(it, flows[j][k], fmtMoney)).join("") });
  Charts.bars($("#cmp-draw"), $("#cmp-draw-tip"), { cats, yFmt: v => `${(v * 100).toFixed(1)} %`, minBar: 3, series: runs.map((it, j) => ({ group: String(j), color: it.color, values: rates[j].map(v => Number.isFinite(v) ? v : 0) })),
    tip: k => `<b>${cats[k]}</b>` + runs.map((it, j) => cmpRow(it, rates[j][k], pctTxt)).join("") });
  // #122: as in the single result, performance (linear or logarithmic) or change over the last 12 months in %
  const real = (r, k) => r[k] / (realView ? r.priceIndex : 1);
  const chg = (it, k) => cmpPad(n, it.run.records.map((r, i) => i >= 12 ? 100 * (real(r, k) / real(it.run.records[i - 12], k) - 1) : NaN));
  const infl = it => cmpPad(n, it.run.records.map((r, i) => i >= 12 ? 100 * (r.priceIndex / it.run.records[i - 12].priceIndex - 1) : NaN));
  const price = it => cmpPad(n, it.run.records.map(r => 100 * r.priceIndex));
  const mser = runs.map(it => mktPct ? { s: chg(it, "stockIndex"), b: chg(it, "bondIndex"), p: infl(it) } : { s: idx(it, "stockIndex"), b: idx(it, "bondIndex"), p: price(it) });
  const mk = [];
  runs.forEach((it, j) => mk.push({ color: it.color, values: mser[j].s }, { color: it.color, dash: "6 3", width: 1.5, values: mser[j].b }, { color: it.color, dash: "2 3", width: 1.2, values: mser[j].p }));
  const f1 = v => Number.isFinite(v) ? v.toLocaleString("de-CH", { maximumFractionDigits: 1 }) + (mktPct ? " %" : "") : "–";
  cmpTime("cmp-mkt", n, mk, k => runs.map((it, j) => cmpRow(it, mser[j].s[k], () => mktPct ? `Aktien ${f1(mser[j].s[k])}, Obligationen ${f1(mser[j].b[k])}, Inflation ${f1(mser[j].p[k])}`
    : `Aktien ${f1(mser[j].s[k])}, Obligationen ${f1(mser[j].b[k])}, Preisindex ${f1(mser[j].p[k])}`)).join(""),
    { ref: mktPct ? 0 : 100, log: mktLog && !mktPct, yFmt: mktPct ? v => `${v.toLocaleString("de-CH", { maximumFractionDigits: 1 })} %` : v => v.toLocaleString("de-CH", { maximumFractionDigits: 0 }) });
}
function cmpTable(body, items) {
  const runs = items.filter(it => it.run);
  const yrs = cmpYears(runs, it => yearValues(it.run.records, { total: it.run.summary.startTotal, liquid: it.run.summary.startLiquid }));
  const key = cmpYearKey, label = CMP_YEAR.find(([k]) => k === key)?.[1] || "";
  const cell = (v, y) => `<td title="${esc(String(y))}">${key === "rate" ? rateTxt(v) : !Number.isFinite(v) ? "–" : fmtInt(v)}</td>`;
  const notes = cmpRunNotes(items);
  const rows = Array.from({ length: yrs.n }, (_, k) => `<tr><td>${yrs.label(k)}</td>${yrs.per.map(v => v[k] ? cell(v[k][key], v[k].y) : "<td>–</td>").join("")}</tr>`).join("");
  body.innerHTML = cmpPicker(items) + cmpNote(notes) + card("Jahreswerte", `${label} · ${cmpReal()}, Beträge in ${cur()}${runs.some(it => it.batch) ? ` · ${pName(cmpP)}` : ""}`,
    `<label class="field" style="max-width:320px"><span>Grösse</span><select data-action="cmp-year">${CMP_YEAR.map(([k, t]) => `<option value="${k}" ${k === key ? "selected" : ""}>${t}</option>`).join("")}</select></label>
    <div class="tscroll"><table class="tbl num yt cmpyear"><tr><th>${yrs.same ? "Jahr" : "Jahr seit Start"}</th>${runs.map(it => `<th class="cmph">${cmpSw(it)}${esc(it.name)}</th>`).join("")}</tr>${rows}</table></div>
    <p class="hint">Dieselben Werte wie in den Jahreswerten der einzelnen Ergebnisse: Vermögen sind Stände am Jahresende, Einkommen, Ausgaben und Entnahme Monatsdurchschnitte des Kalenderjahrs, Steuern die Summe des Steuerjahres.${yrs.same ? "" : " Die Ergebnisse beginnen in verschiedenen Jahren und sind nach Jahren seit dem Start ausgerichtet; die Jahreszahl steht im Tooltip der Zelle."}</p>`, "yearcard");
}
document.addEventListener("change", e => { if (e.target.matches?.('select[data-action="cmp-year"]')) { cmpYearKey = e.target.value; compareRuns(); } });
/**
 * #123: Fields of the plan that do not appear in the comparison (technical or without an input field), with reason. All other
 * inputs are shown by `paramSections(plan, true)` with the labels of the editor; a user interface test checks the coverage.
 */
const PLAN_HIDDEN = [
  [/^format$/, "Dateiformat"], [/^project\.name$/, "Name der Planung, steht im Namen des Ergebnisses"], [/(^|\.)id$/, "interne Kennung"],
  [/^household\.persons\.\d+\.name$/, "Personennamen werden absichtlich nicht gezeigt"], [/^milestones\.\d+\.fixed$/, "fester Meilenstein (nicht löschbar)"],
  [/\.mode$/, "Art der Zeitreihe; verglichen werden die Werte"], [/^flows\.spendingIncludesHousing$/, "kein Eingabefeld"],
  [/^realestate\.affordability\.wealthContribution$/, "kein Eingabefeld"], [/^withdrawal\.buckets\.reserveMonths$/, "kein Eingabefeld"],
  [/^realestate\.properties\.\d+\.inflation$/, "älteres Feld, ersetzt durch «Wert/Nebenkosten mit Inflation anpassen»"],
  [/^returns\.parametric\.sampler$/, "kein Eingabefeld (Verfahren der Zufallsziehung, #132)"],
];
function cmpPlan(body, items, datas) {
  // Sections in the order of the editor, within them the labels in the order of their first occurrence
  const secs = new Map();
  datas.forEach((d, j) => paramSections(d.plan || {}, true).forEach(([title, rows]) => {
    if (!secs.has(title)) secs.set(title, new Map());
    const m = secs.get(title);
    rows.forEach(([k, v]) => { if (!m.has(k)) m.set(k, []); m.get(k)[j] = v; });
  }));
  const groups = [...secs].map(([title, m]) => [title, [...m].filter(([, vs]) => datas.some((_, j) => (vs[j] ?? "—") !== (vs[0] ?? "—")))]).filter(([, rows]) => rows.length);
  const n = groups.reduce((a, [, rows]) => a + rows.length, 0), cols = items.length + 1;
  body.innerHTML = card("Unterschiede in der Planung", "gegenüber dem ersten Ergebnis in der Reihenfolge der Liste, mit den Bezeichnungen des Editors",
    n ? `<div class="tscroll"><table class="tbl cmpplan"><tr><th class="l">Eingabe</th>${items.map(it => `<th class="l cmph">${cmpSw(it)}${esc(it.name)}</th>`).join("")}</tr>
      ${groups.map(([title, rows]) => `<tr class="grp"><th class="l" colspan="${cols}">${esc(title)}</th></tr>${rows.map(([k, vs]) => `<tr><td class="l">${esc(k)}</td>${items.map((_, j) => `<td class="l">${esc(vs[j] ?? "—")}</td>`).join("")}</tr>`).join("")}`).join("")}</table></div>`
      : `<p style="margin:0">Die Planungen sind identisch.</p>`);
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

// ---------- Welcome (#70) and first steps (#71) ----------
/** Loads the example plan as a new, unsaved plan (after asking if there are unsaved changes). */
async function loadExample() {
  if (!await confirmDiscard()) return false;
  await adopt(await Engine.call("example_plan")); setFile(null); History.reset(); goto("general"); refreshSeriesInfo();
  return true;
}
/**
 * Welcome dialog: at the very first start and via «Hilfe → Willkommen …». Esc or closing leaves everything as it is
 * (at the first start that is the example plan, i.e. like «Beispiel erkunden», only without help).
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
/** Steps of the checklist: the sections of the quick start and finally «Simulation starten». */
const STEPS = [["general", "s-general"], ["milestones", "s-milestones"], ["pension", "s-pension"], ["flows", "s-flows"],
  ["wealth", "s-wealth"], ["returns", "s-returns"], ["taxes", "s-taxes"], ["run", "quickstart"]]
  .map(([id, topic]) => ({ id, topic, title: SECTIONS.find(s => s.id === id)?.title ?? "Simulation starten" }));
/**
 * Checklist «Erste Schritte». The progress applies to the user (setting «firstSteps»), not to a plan.
 * A section is done as soon as a value in it was changed or it was ticked by hand; undo does not remove the
 * tick. «Simulation starten» is done after a simulation without cancel.
 */
const Steps = {
  get() {
    const v = Pref.get("firstSteps", null) || {};
    return { done: Array.isArray(v.done) ? v.done.filter(id => STEPS.some(s => s.id === id)) : [], hidden: v.hidden === true, complete: v.complete === true };
  },
  put(v, open) { Pref.set("firstSteps", v); this.render(open); },
  /** Ticks a step; if all are done with it, a notice appears once and the list collapses. */
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
  /** The list sits below the preview and key figures, often outside the visible area: scroll there, focus on the header. */
  reveal() {
    const el = $("#steps"); if (!el || el.hidden) return;
    el.scrollIntoView({ block: "nearest" });
    el.querySelector("summary")?.focus({ preventScroll: true });
  },
  /** `open`: expanded (true), collapsed (false) or as before (undefined; default: open until everything is done). */
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
/** A change in the editor ticks the section in which it happens. */
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
    // Desktop app: plan last opened, otherwise the bundled example plan (#4)
    if (Engine.inApp) startup = await Engine.call("startup_plan");
    var startNotes = await adopt(startup ? startup.plan : await Engine.call("example_plan"));
    gainTemplates = await Engine.call("gain_templates");
  } catch (e) { document.body.innerHTML = `<p style="padding:24px">Der Rechenkern ist nicht erreichbar: ${esc(errText(e))}</p>`; return; }
  for (const src of ["historical", "rolling", "bootstrap", "parametric"]) if (initial.includes(src)) state.returns.source = src;
  if (initial.includes("buckets")) state.withdrawal.strategy = "three_buckets";
  if (initial.includes("fixedrate")) state.withdrawal.strategy = "fixed_rate";
  if (initial.includes("vpw")) state.withdrawal.strategy = "vpw";
  if (initial.includes("endowment")) state.withdrawal.strategy = "endowment";
  if (initial.includes("guardrails")) state.withdrawal.strategy = "guardrails";
  // #111: Test access without measuring the computing time; otherwise it would run in the background in every test
  if (initial.includes("nobench")) bench.off = true;
  // #111: Results in test access with 500 paths and a fixed random seed: fast and the same in every run. The tests
  // check display and flow; 50 000 paths made them depend on the computer's load.
  if (initial[0] === "results") for (const k of ["bootstrap", "parametric"]) Object.assign(state.returns[k], { paths: 500, seed: 42 });
  if (!IS_MAC) document.querySelectorAll("[title]").forEach(el => { el.title = el.title.replace(/⇧⌘/g, "Ctrl+Shift+").replace(/⌘↩/g, "Ctrl+Enter").replace(/⌘/g, "Ctrl+"); });
  setFile(startup?.path ?? null); History.reset();
  if (initial.includes("partner")) { ACTIONS["partner-add"](); state.partner.income = { mode: "const", value: 3000 }; History.reset(); }
  if (initial.includes("nosweep")) state.scenarios.sweeps = [];
  if (initial.includes("gainzh")) { ACTIONS["gain-zh"](); History.reset(); }
  if (initial.includes("optset")) { state.optimization = optToPlan({ ...OPT_DEFAULT, maxRuin: 2, weightMedian: 70, maxVol: 12, strategies: ["guardrails", "vpw"] }); History.reset(); }
  renderSection(); refreshSeriesInfo(); refreshLibrary(); Steps.render();
  // #141: Smoke test of the shipped app (`--smoke`): calculate the example plan with fixed returns and report; without
  // disclaimer and welcome, without writing settings. The app then exits.
  if (window.__VIVAPLAN_SMOKE__) {
    let ok = false, message;
    try {
      const plan = await Engine.call("example_plan");
      plan.returns.source = "fixed"; plan.scenarios.sweeps = [];
      const r = await Engine.call("run_simulation", { plan, path: null });
      ok = !r.error && !!r.single && r.single.records.length > 0 && window.__errors.length === 0;
      message = r.error || (window.__errors.length ? window.__errors.join("; ") : `${r.single.records.length} Monate gerechnet`);
    } catch (e) { message = errText(e); }
    await Engine.call("smoke_done", { ok, message });
    return;
  }
  // #61: Confirm the disclaimer before everything else (desktop app; in the browser only with the test access #disclaimer).
  if ((Engine.inApp || initial.includes("disclaimer")) && disclaimerPending()) {
    document.documentElement.dataset.disclaimer = "open";
    await disclaimerGate();
    document.documentElement.dataset.disclaimer = "ok";
  }
  // #70: Welcome dialog at the very first start, after the disclaimer (test access in the browser: #welcome). This is remembered immediately so that it
  // does not come back even if the app is quit without a choice.
  if (startup?.first || initial[0] === "welcome") { Pref.set("welcomeShown", true); showWelcome(); }
  if (startup?.notice) ask("Planung beim Start", `<p>${esc(startup.notice)}</p>`);
  else if (startup?.path) { await showNotes(startNotes); reserveNotice(state); }
  // Test access: change a setting and check that it ends up in the plan
  if (initial.includes("optedit")) {
    const el = $("#opt-maxRuin"); el.value = "3"; el.dispatchEvent(new Event("change", { bubbles: true }));
    document.documentElement.dataset.optsaved = JSON.stringify(state.optimization);
    document.documentElement.dataset.optdirty = dirty ? "1" : "0";
  }
  // Test access: edit ranges (bucket 1 fixed, bucket 2 from 0 to 2 years, refill threshold −30 % to +10 %)
  if (initial.includes("rangeedit")) (async () => {
    while (!optCat.data) await new Promise(r => setTimeout(r, 50));
    const set = (id, v) => { const el = document.getElementById(id); el[el.type === "checkbox" ? "checked" : "value"] = v; el.dispatchEvent(new Event("change", { bubbles: true })); };
    set("optR-three_buckets-cashYears-var", false); set("optR-three_buckets-bondYears-lo", "0"); set("optR-three_buckets-bondYears-hi", "2");
    set("optR-three_buckets-refillThreshold-lo", "-30"); set("optR-three_buckets-refillThreshold-hi", "10");
    document.documentElement.dataset.ranges = JSON.stringify(state.optimization.ranges);
    document.documentElement.dataset.dirtyr = dirty ? "1" : "0";
    document.documentElement.dataset.rangesum = document.querySelector("details.optrange:nth-of-type(2) summary .computed")?.textContent ?? "";
  })();
  // Development and test access: #results/<source>/<tab>/real
  if (initial.includes("real")) realView = true;
  if (initial.includes("sweep2")) state.scenarios.sweeps.push({ param: "flows.spendingFactor", from: 0.9, to: 1.1, step: 0.1 });
  if (initial.includes("nosweep")) state.scenarios.sweeps = [];
  if (initial.includes("mem")) Runs.memoryOnly();
  if (initial[0] === "about") showAbout();
  if (initial[0] === "runs") openRuns();
  // Test access: store two simulations, then compare (…/view: view the first, …/del: delete one). 500 instead of 50 000 paths with
  // a fixed seed like #results (#111): the tests check list, comparison and display, not key figures (#79).
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
