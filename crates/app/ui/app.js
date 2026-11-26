// VivaPlan user interface: editor, live preview and results view.
"use strict";
/** Technical upper limit of the combinations of the variations (as in the engine, memory protection). */
const MAX_COMBINATIONS = 1_000_000;
/** From this estimated computing time (seconds) on, the app asks before starting. */
const LONG_RUN_SECONDS = 60;
let SWEEP_PARAMS;
onLanguage(() => SWEEP_PARAMS = [
  { id: "wealth.stockShare", label: tr("sweep.stockShare"), type: "pct", def: [0.3, 1.0, 0.1] },
  { id: "pension.pk.lumpsum", label: tr("sweep.pkLumpsum"), type: "pct", def: [0, 1, 0.25] },
  { id: "returns.startDate", label: tr("sweep.histStart"), type: "month", def: ["1970-01", "2010-01", 12] },
  { id: "milestone.retire", label: tr("sweep.retireDate"), type: "month", def: ["2026-09", "2028-03", 3] },
  { id: "flows.spendingFactor", label: tr("sweep.spendingFactor"), type: "pct", def: [0.9, 1.1, 0.05] },
  { id: "withdrawal.rate", label: tr("sweep.withdrawalRate"), type: "pct", def: [0.03, 0.05, 0.005] }
]);
let SECTIONS;
onLanguage(() => SECTIONS = [
  { id: "general", title: tr("section.general") },
  { id: "milestones", title: tr("section.milestones") },
  { id: "pension", title: tr("section.pension") },
  { id: "pillar3a", title: tr("section.pillar3a") },
  { id: "flows", title: tr("section.flows") },
  { id: "wealth", title: tr("section.wealth") },
  { id: "returns", title: tr("section.returns") },
  { id: "withdrawal", title: tr("section.withdrawal") },
  { id: "optimize", title: tr("section.optimize") },
  { id: "taxes", title: tr("section.taxes") },
  { id: "realestate", title: tr("section.realestate") },
  { id: "scenarios", title: tr("section.scenarios") }
]);

let previewBasis = null;
let state = null, currentPath = null, dirty = false, seriesInfo = null, lastRun = null, library = null;
/** Plan with which `lastRun` was calculated (state of the run, not of the editor); `null`: not known. */
let lastRunPlan = null;
let current = "general";
let flashId = null;
let lastSim = null;
/** #191: AHV pension per person from the preview (entered, rate, effective, capped share); also with errors in the plan. */
let lastAhv = [];

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
const infoPop = (html, cls = "") => `<span class="infopop" tabindex="0" role="button" aria-label="${tr("common.showInfo")}" aria-haspopup="true" data-action="info-toggle">ⓘ<span class="infobody ${cls}" role="tooltip">${html}</span></span>`;
/** Short explanation of a series (at most 2 lines): `# Info:` of the file, otherwise source and notes, shortened; without details no icon. */
function seriesTip(i) {
  const text = msgText(i?.info) || [i?.source, ...(i?.notes || [])].map(msgText).filter(Boolean).join(" ");
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
/** #170/AK8: the fixed milestones store no name; it comes from the text table by the id. */
const FIXED_MS_IDS = ["start", "end", "retire", "ahv", "retire2", "ahv2"];
const msLabel = ms => (ms && FIXED_MS_IDS.includes(ms.id) ? tr(`milestone.${ms.id}`) : ms?.name);
function msName(ref) { if (ref === "never") return tr("ref.never"); if (ref === "afterSale") return tr("ref.afterSale"); if (saleOf(ref)) return tr("ref.afterSaleOf", { propById: propById(saleOf(ref))?.name ?? "?" }); if (isYM(ref)) return ymShow(ref); return msLabel(state.milestones.find(m => m.id === ref)) ?? "?"; }

/** #217: the forms «linear», «table» and «factor» (month by month in the mini chart, no dots). */
const NEW_FORMS = ["linear", "table", "factor"];
const isNewForm = ser => !!ser && typeof ser === "object" && NEW_FORMS.includes(ser.mode);
/** #243: the five forms of a course in the order of the selector. */
const SERIES_FORMS = ["const", "timed", ...NEW_FORMS];
const formName = mode => ({ const: tr("series.form.const"), timed: tr("series.form.timed"), linear: tr("series.form.linear"), table: tr("series.form.table"), factor: tr("series.form.factor"), curve: tr("series.form.curve") })[mode];
/** #218: characteristic curve over a state variable (only the stock share); the editor shows it read-only. */
const isCurve = ser => !!ser && typeof ser === "object" && ser.mode === "curve";
const STATE_INPUTS = ["age", "liquidReal", "withdrawalRate", "fundedRatio", "stockReturn12m", "drawdown"];
const stateName = input => STATE_INPUTS.includes(input) ? tr(`state.${input}`) : String(input ?? "?");
/** #218: input value of a support point in the unit of its state variable. */
function stateValue(input, x) {
  if (input === "age") return `${fmtNum(+x)} ${tr("unit.yearsShort")}`;
  if (input === "liquidReal") return fmtInt(+x);
  if (input === "fundedRatio") return fmtNum(+x);
  return `${fmtPct(+x)} %`;
}
/** #217: month of a support point (date, milestone or age of a person, the first person without one). */
function pointMonth(p) {
  if (p?.age == null) return refMonth(p?.at);
  const birth = state.household.persons.find(x => x.id === (p.person ?? state.household.persons[0]?.id))?.birth;
  return birth ? ym2i(birth) + Math.round(p.age * 12) : Infinity;
}
/** #217: value between support points sorted by month (the engine computes the same, here only for display). */
function pointsAt(pts, m, linear) {
  const s = pts.map(p => [pointMonth(p), +p.value || 0]).filter(p => Number.isFinite(p[0])).sort((x, y) => x[0] - y[0]);
  if (!s.length) return 0;
  let i = -1;
  s.forEach((p, k) => { if (p[0] <= m) i = k; });
  if (i < 0) return s[0][1];
  if (i === s.length - 1 || !linear) return s[i][1];
  const [[m1, a], [m2, b]] = [s[i], s[i + 1]];
  return a + (b - a) * (m - m1) / (m2 - m1);
}
function seriesAt(ser, m) {
  if (ser == null) return 0;
  if (typeof ser !== "object") return +ser || 0;
  if (ser.mode === "const") return +ser.value || 0;
  if (ser.mode === "linear") return pointsAt([ser.start, ser.end].filter(Boolean), m, true);
  if (ser.mode === "table") return pointsAt(ser.points || [], m, ser.interpolation !== "step");
  if (ser.mode === "factor") return (+ser.base || 0) * seriesAt(ser.factor, m);
  // #218: the course of a curve is only known in the computation; the value of its first support point.
  if (ser.mode === "curve") return +ser.points?.[0]?.[1] || 0;
  let best = null, bm = -Infinity;
  for (const r of ser.rows || []) { const rm = refMonth(r.from); if (rm <= m && rm >= bm) { bm = rm; best = r; } }
  return best ? +best.value || 0 : 0;
}
/** All values a course can take at its support points (for checks of the whole course). */
function seriesValues(ser) {
  if (ser == null) return [];
  if (typeof ser !== "object") return [ser];
  if (ser.mode === "const") return [ser.value];
  if (ser.mode === "linear") return [ser.start?.value, ser.end?.value];
  if (ser.mode === "table") return (ser.points || []).map(x => x.value);
  if (ser.mode === "factor") return seriesValues(ser.factor).map(x => (+ser.base || 0) * x);
  if (ser.mode === "curve") return (ser.points || []).map(p => p?.[1]);
  return (ser.rows || []).map(x => x.value);
}
const tax = (table, x) => taxAt(table, x, state.taxes.multiplier); // #156: like the engine (multiplier 0 → no tax)

// Mortgage: one or more tranches, each with a sequence of periods. The first period starts at the
// start date (or at the purchase), each further one at the end of the previous one. Amortization as a fixed amount per quarter,
// due at the end of March/June/September/December; direct (reduces the debt) or indirect (contribution to pillar 3a).
const saron = () => +state.returns.fixed.saron || 0;
const perRate = d => d.type === "saron" ? Math.max(0, saron() + (+d.margin || 0)) : +d.rate || 0;
const newPeriod = (o = {}) => ({ type: "fixed", term: 5, rate: 0.015, margin: 0.009, amort: 0, amortMode: "direct", ...o });
const newMortgage = () => ({ tranches: [ { name: tr("mortgage.tranche1"), amount: 400000, after: "renew", periods: [newPeriod({ term: 10 })] } ] });

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
/** #272/AK3: estimated tax of the first full calendar year and of the first after the retirement (engine preview). */
let taxEstimates = [];
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
  // #131: basis of the income tax per tax year (only plans with deductions)
  // #241: with the price index of December for the additional amount (prices of the start) in the details of a row
  out.taxBasis = new Map(r.records.filter(x => x.taxBasis).map(x => [x.month.slice(0, 4), { ...x.taxBasis, priceIndex: x.priceIndex }]));
  for (const e of r.events) {
    out.events.push({ m: ym2i(e.month), label: msgText(e), kind: e.kind, tick: TICK_KINDS.includes(e.kind), bad: e.kind === "emergencySale" || e.kind === "depleted" });
    if (e.kind === "pillar3aPayout") out.p3a.push({ m: ym2i(e.month), text: msgText(e) });
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
      lastIssues = r.issues; previewError = r.error; lastSim = r.result ? adapt(r.result) : null; taxEstimates = r.taxEstimates || []; if (lastSim) { lastSim.p3aView = r.pillar3a || []; lastSim.wealthTax = r.wealthTax || []; } lastAhv = r.ahv || []; previewBasis = r.basis;
    } catch (e) {
      if (seq !== previewSeq) return;
      previewError = errText(e); lastSim = null; lastAhv = []; taxEstimates = [];
    }
    renderDerived();
    if (req === requestPreview.n) document.documentElement.dataset.preview = "idle"; // no newer request
  }, 120);
}

/** Validation messages of the engine plus notes from the preview. */
function collectIssues() {
  const v = lastIssues.map(i => ({ sev: i.severity === "error" ? "err" : "warn", sec: i.section, text: esc(msgText(i)) }));
  if (previewError && !v.some(x => x.sev === "err")) v.push({ sev: "err", sec: "general", text: esc(errText(previewError)) });
  const sc = seriesInfo && !seriesInfo.error && state.returns.source !== "fixed" && seriesInfo.currency;
  if (sc && sc !== cur()) v.push({ sev: "warn", sec: "returns", text: tr("issues.seriesCurrency", { sc: esc(sc), esc: esc(cur()), sc2: esc(sc) }) });
  if (lastSim) {
    if (lastSim.depleted !== null) v.push({ sev: "err", sec: "flows", text: tr("issues.previewDepleted", { ymShow: ymShow(i2ym(lastSim.depleted)), depleted: ageStr(lastSim.depleted) }) });
    for (const ev of lastSim.events) if (ev.kind === "emergencySale") v.push({ sev: "warn", sec: "realestate", text: tr("issues.previewEmergency", { ymShow: ymShow(i2ym(ev.m)), m: ageStr(ev.m) }) });
  }
  return v;
}

/** Adds missing parts so that the editor can show every valid project file. */
const SERIES_PATHS = ["flows.income", "flows.spending", "pension.pk.contribution", "pension.pk.interest", "wealth.stockShare", "wealth.reserveMonths", "realestate.emergencyMonths",
  "partner.income", "partner.pk.contribution", "partner.pk.interest"];
/** #58: Labels of the choice for the earned income of each person. */
let INCOME_INDEXED;
onLanguage(() => INCOME_INDEXED = tr("flows.incomeIndexed"));
/** #236: Switch per person: the earned income ends with the milestone «Bezug Pensionskasse». */
const incomeEnds = path => `<div style="margin:-8px 0 12px">${Chk(path, tr("flows.incomeEndsAtRetirement"))}</div>`;
// #236/AK1: a new person's earned income ends with their pension fund payout
const newPartner = (pid = "B") => ({ income: { mode: "const", value: 0 }, incomeIndexed: true, incomeEndsAtRetirement: true,
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
  return seriesValues(r).some(v => v !== FIXED_RESERVE_MONTHS);
}
/**
 * #61: Disclaimer. The short version must be confirmed at the first start; the confirmation is stored in the
 * settings (not in the plan) with the version of the text. If the text changes, increase the version: then the user is
 * asked again. The full text is in the help (`hilfe/de/00-einstieg.md`, section «@disclaimer»).
 */
const DISCLAIMER_VERSION = 2, DISCLAIMER_KEY = "disclaimerAccepted"; // #143/AK7: version 2 names tax and investment advice
let DISCLAIMER_SHORT;
onLanguage(() => DISCLAIMER_SHORT = tr("disclaimer.short"));
let DISCLAIMER_FOOT;
onLanguage(() => DISCLAIMER_FOOT = tr("disclaimer.foot"));
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
/** #270: Beta token. Without an activation only this dialog is reachable: no opening, creating or calculating of a plan.
 *  The engine checks the token and saves the activation. The development server needs no activation; its test access
 *  #beta (`asIfLocked`) behaves like an app without one and checks with the test key. */
let betaLocked = false, betaInfo = null;
async function betaGate(asIfLocked = false) {
  const status = asIfLocked ? { required: true, active: false } : await Engine.call("beta_status");
  if (!status.required || status.active) return;
  betaLocked = true; document.documentElement.dataset.beta = "open";
  const d = $("#beta"), input = $("#betaToken"), err = $("#betaErr"), ok = $("#betaOk");
  d.oncancel = e => e.preventDefault();
  await new Promise(res => {
    $("#betaForm").onsubmit = async e => {
      e.preventDefault();
      ok.disabled = true; err.textContent = "";
      try { betaInfo = await Engine.call("beta_activate", { token: input.value }); d.close(); res(); }
      catch (x) { err.textContent = errText(x); input.focus(); }
      finally { ok.disabled = false; }
    };
    d.showModal(); input.focus();
  });
  betaLocked = false; document.documentElement.dataset.beta = "ok";
}
/** #74: Setting «notice about the cash reserve shown»; the notice appears only when an affected plan is loaded for the first time. */
const RESERVE_NOTICE_KEY = "reserveNoticeShown";
async function reserveNotice(p) {
  if (!reserveDiffers(p) || Pref.get(RESERVE_NOTICE_KEY, false)) return;
  Pref.set(RESERVE_NOTICE_KEY, true);
  await ask(tr("reserve.title"), `<p>${tr("reserve.text", { fixedReserveMonths: FIXED_RESERVE_MONTHS, fixedReserveMonths2: FIXED_RESERVE_MONTHS })}</p><p>${tr("reserve.note")}</p>`);
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
  p.taxes.investmentYield ??= { stocks: 0, bonds: 0 }; // #149/AK5: existing plans open with 0 %
  if (p.taxes.couple) p.taxes.couple = { income: p.taxes.couple.income ?? [], wealth: p.taxes.couple.wealth ?? [], capital: p.taxes.couple.capital ?? [] };
  p.scenarios ??= { sweeps: [] }; p.scenarios.sweeps ??= [];
  p.flows.extras ??= [];
  p.pillar3a ??= [];
  for (const a of p.pillar3a) { a.payouts ??= []; a.use ??= "wealth"; a.contribution ??= 0; a.rate ??= 0; a.balance ??= 0; }
  p.realestate ??= {}; p.realestate.properties ??= [];
  for (const q of p.realestate.properties) if (q.status !== "rent") q.sell ??= "never";
  p.realestate.emergencyMonths ??= 0;
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
    for (const t of pr.mortgage.tranches) { t.name ??= tr("mortgage.tranche"); t.after ??= "renew"; t.periods = t.periods.map(d => newPeriod(d)); }
  }
  return p;
}

/**
 * Takes over a loaded plan: first bring it up to date (#52, #51; show notes), then
 * normalize it. Returns the notes.
 */
async function adopt(plan, fromFile = true) {
  let notes = [];
  // #233/AK4: the affordability of an older file (before the upgrade, which leaves it as it is)
  const fileAfford = fromFile ? plan?.realestate?.affordability : null;
  try { const r = await Engine.call("upgrade_plan", { plan }); plan = r.plan; notes = r.notes || []; }
  catch { /* without the engine: take over unchanged */ }
  state = normalize(plan);
  taxSwitch = null; taxOpen.settings = taxOpen.expert = null; // #272: only for the plan of the switch
  delete state.realestate.affordability;
  const adopted = adoptAffordability(fileAfford);
  if (adopted) notes.push(adopted);
  restoreOpt();
  return notes;
}
async function showNotes(notes) {
  if (notes?.length) await ask(tr("notes.title"), `<p>${tr("notes.intro")}</p><ul>${notes.map(n => `<li>${esc(msgText(n))}</li>`).join("")}</ul>`);
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
    catch (e) { seriesInfo = { error: errText(e), code: e?.code }; }
  }
  if (seriesInfoRef !== s) return; // a newer query is running
  followSuggestion();
  // Which fields «Simulation» shows depends on the time series.
  if (current === "returns") renderSection(); else renderDerived();
}

// ---------- Form building blocks ----------
const UNIT = { chf: () => cur(), chfopt: () => cur(), chfyear: () => cur(), pct: () => "%", months: () => tr("unit.months"), years: () => tr("unit.years"), num: () => "" };
// #232: `chfyear` is an amount per year in the field, stored per month (field value / 12).
function showVal(type, v) {
  if (type === "chf") return fmtInt(v ?? 0);
  if (type === "chfyear") return fmtInt((v ?? 0) * 12);
  if (type === "chfopt") return v == null ? "" : fmtInt(v);
  if (type === "pct") return fmtPct(v ?? 0);
  if (type === "month") return ymShow(v);
  return v ?? "";
}
// ---------- Required inputs (#14; list in docs/Pflichteingaben.md) ----------
const REQUIRED_PATHS = [/^household\.persons\.\d\.(name|birth)$/, /^household\.ahvCoupleCap$/, /^(partner|pension)\.pk\.(capital|conversion|lumpsum)$/,
  /^(partner|pension)\.ahv\.monthly$/, /^wealth\.(stocks|bonds|cash)$/, /^taxes\.multiplier$/];
const isRequired = path => REQUIRED_PATHS.some(r => r.test(path));
const reqMark = path => isRequired(path) ? ` <span class="req" title="${tr("field.required")}">*</span>` : "";
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
  for (const i of lastIssues) if (i.field && i.severity === "error" && !msgs.has(i.field)) msgs.set(i.field, msgText(i));
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
  toast(tr("errors.blocked", { length: errs.length, title: sec.title, message: msgText(first) }));
  document.documentElement.dataset.blocked = String(errs.length);
  return true;
}
function F(path, label, type = "chf", o = {}) {
  const v = getP(path), unit = o.unit ?? (UNIT[type] ? UNIT[type]() : "");
  const cls = ["chf", "chfopt", "chfyear", "pct", "num", "months", "years"].includes(type) ? "num" : "";
  const input = `<div class="inp"><input class="${cls}" data-path="${path}" data-type="${type}"${o.int ? ' data-int="1"' : ""}${o.size ? ` size="${o.size}"` : ""} value="${esc(showVal(type, v))}" ${type === "month" ? `placeholder="${tr("input.datePlaceholder")}"` : ""}>${unit ? `<span class="unit">${unit}</span>` : ""}</div>${type === "chfyear" ? perMonth(path) : ""}`;
  if (o.bare) return input;
  return `<label class="field"><span>${label}${reqMark(path)}</span>${input}${o.hint ? `<span class="computed">${o.hint}</span>` : ""}</label>`;
}
function Sel(path, opts, o = {}) {
  const v = getP(path);
  return `<select class="plain" data-path="${path}" data-type="select" ${o.rerender ? "data-rerender=1" : ""}>${opts.map(([k, l]) => `<option value="${esc(k)}" ${k === v ? "selected" : ""}>${esc(l)}</option>`).join("")}</select>`;
}
function Chk(path, label, o = {}) { return `<label class="check"><input type="checkbox" data-path="${path}" data-type="bool" ${getP(path) ? "checked" : ""} ${o.rerender ? "data-rerender=1" : ""}> ${label}</label>`; }
function Ref(path, o = {}) {
  const v = getP(path), date = isYM(v);
  const opts = state.milestones.map(ms => `<option value="${ms.id}" ${ms.id === v ? "selected" : ""}>${esc(msLabel(ms))}</option>`).join("");
  return `<div class="refwrap"><select class="plain" data-path="${path}" data-type="ref">
    ${o.never ? `<option value="never" ${v === "never" ? "selected" : ""}>${tr("ref.never")}</option>` : ""}
    ${o.afterSale ? `<option value="afterSale" ${v === "afterSale" ? "selected" : ""}>${tr("ref.afterLastSale")}</option>` : ""}
    ${o.afterSale ? state.realestate.properties.filter(q => q.status !== "rent" && q.id !== o.self).map(q => `<option value="afterSale:${esc(q.id)}" ${v === "afterSale:" + q.id ? "selected" : ""}>${tr("ref.afterSaleOfName", { name: esc(q.name) })}</option>`).join("") : ""}
    <optgroup label="${tr("ref.milestone")}">${opts}</optgroup>
    <option value="__date" ${date ? "selected" : ""}>${tr("ref.fixedDate")}</option></select>
    ${date ? F(path, "", "month", { bare: true }) : ""}</div>`;
}
/** #243: a course as an object (a plain number becomes a constant, also the course of factors). */
function seriesDef(path) {
  let s = getP(path);
  if (s == null || typeof s !== "object") { s = { mode: "const", value: +s || 0 }; setP(path, s); }
  return s;
}
/** #243: selector of the form of a course. */
function seriesForms(path, mode, forms = SERIES_FORMS) {
  return `<div class="seg">${forms.map(f => `<button class="${f === mode ? "on" : ""}" data-action="series-mode" data-path="${path}" data-mode="${f}">${formName(f)}</button>`).join("")}</div>`;
}
const personLabel = (p, i) => p.name || tr("milestones.person") + (i + 1);
/** #243: point in time of a support point: age of a person, milestone or fixed date (with the age or the date beside it). */
function PointWhen(path) {
  const p = getP(path), persons = state.household.persons;
  const cur = p.age != null ? `age:${p.person ?? persons[0].id}` : isYM(p.at) ? "__date" : p.at;
  const opt = (v, l) => `<option value="${esc(v)}" ${v === cur ? "selected" : ""}>${esc(l)}</option>`;
  return `<div class="refwrap"><select class="plain" data-path="${path}" data-type="point">
    <optgroup label="${tr("series.age")}">${persons.map((x, i) => opt(`age:${x.id}`, tr("series.ageOf", { name: personLabel(x, i) }))).join("")}</optgroup>
    <optgroup label="${tr("ref.milestone")}">${state.milestones.map(ms => opt(ms.id, msLabel(ms))).join("")}</optgroup>
    ${opt("__date", tr("ref.fixedDate"))}</select>
    ${p.age != null ? F(path + ".age", "", "years", { bare: true }) : isYM(p.at) ? F(path + ".at", "", "month", { bare: true }) : ""}</div>`;
}
/** #243: «age:<person>», a milestone or «__date»; the point in time stays the same where it can. */
function setPointWhen(p, v) {
  const m = pointMonth(p), known = Number.isFinite(m) ? m : startM();
  delete p.at; delete p.age; delete p.person;
  if (v.startsWith("age:")) {
    const pid = v.slice(4);
    p.age = Math.max(0, Math.round(ageAt(known, pid) * 100) / 100);
    if (pid !== state.household.persons[0].id) p.person = pid;
  } else p.at = v === "__date" ? i2ym(known) : v;
}
/** #243: one support point: label, point in time, value and optionally a delete button. */
function PointRow(path, type, ab, del = "") {
  return `<div class="row${type === "chfyear" ? " peryear" : ""}" data-point="${path}"><span class="ab">${ab}</span>${PointWhen(path)}${F(path + ".value", "", type, { bare: true })}${del || "<span></span>"}</div>`;
}
/** Input fields of a course in its form (#243: also straight line, table and factor). */
function seriesBody(path, type) {
  const s = seriesDef(path);
  if (s.mode === "timed") return `<div class="rows">${s.rows.map((r, i) => `<div class="row${type === "chfyear" ? " peryear" : ""}"><span class="ab">${tr("common.from")}</span>${Ref(`${path}.rows.${i}.from`)}${F(`${path}.rows.${i}.value`, "", type, { bare: true })}<button class="btn ghost small" title="${tr("series.removeRow")}" data-action="row-del" data-path="${path}.rows" data-i="${i}">✕</button></div>`).join("")}
    <div><button class="btn small" data-action="row-add" data-path="${path}.rows">${tr("series.addRow")}</button></div></div>`;
  if (s.mode === "linear") return `<div class="rows" data-form="linear">${PointRow(path + ".start", type, tr("series.linearFrom"))}${PointRow(path + ".end", type, tr("series.linearTo"))}</div>`;
  if (s.mode === "table") {
    const pts = s.points || [], step = s.interpolation === "step";
    return `<div class="rows" data-form="table">${pts.map((_, i) => PointRow(`${path}.points.${i}`, type, tr("series.at"),
      pts.length > 1 ? `<button class="btn ghost small" title="${tr("series.removePoint")}" data-action="row-del" data-path="${path}.points" data-i="${i}">✕</button>` : "")).join("")}
      <div class="sfoot"><button class="btn small" data-action="point-add" data-path="${path}.points">${tr("series.addPoint")}</button>
      <span class="computed">${tr("series.between")}</span><div class="seg"><button class="${step ? "" : "on"}" data-action="set" data-path="${path}.interpolation" data-value="linear">${tr("series.interpLinear")}</button><button class="${step ? "on" : ""}" data-action="set" data-path="${path}.interpolation" data-value="step">${tr("series.interpStep")}</button></div></div></div>`;
  }
  if (s.mode === "factor") {
    const f = seriesDef(path + ".factor");
    return `<div class="rows" data-form="factor">${F(path + ".base", tr("series.base"), type)}
      <div class="shead sub"><span class="t">${tr("series.factor")}</span>${seriesForms(path + ".factor", f.mode, ["const", "timed", "linear", "table"])}</div>${seriesBody(path + ".factor", "pct")}</div>`;
  }
  if (s.mode === "curve") {
    const when = tr(s.eval === "monthly" ? "series.evalMonthly" : "series.evalYearly");
    return `<div class="rows" data-form="curve"><p class="computed shint">${tr("series.curveReadOnly", { input: esc(stateName(s.input)), eval: when })}</p>
      ${(s.points || []).map(p => `<div class="row" data-point="${path}"><span class="ab">${tr("series.at")}</span><span>${esc(stateValue(s.input, p?.[0]))}</span><span>${esc(miniValue(type, +p?.[1] || 0))}</span><span></span></div>`).join("")}</div>`;
  }
  return `<div class="rows">${F(path + ".value", tr("series.value"), type)}</div>`;
}
function Series(path, label, type, note = "", hint = "") {
  const s = seriesDef(path);
  // #218: a characteristic curve has no course over time, hence no mini chart.
  const mini = isCurve(s) ? "" : `<div class="miniwrap"><svg class="mini zoomable" data-mini="${path}" data-type="${type}"></svg><div class="tip"></div>
      <div class="minizoom">${zoomButtons("mini-zoom", miniZoom[path]?.years, `data-path="${path}"`)}<span>${tr("series.zoomHint")}</span></div></div>`;
  return `<div class="series"><div class="shead"><span class="t">${label}</span><span class="computed">${note}</span>
    ${seriesForms(path, s.mode)}</div>${hint ? `<p class="computed shint">${hint}</p>` : ""}
    <div class="sbody">${seriesBody(path, type)}${mini}</div></div>`;
}
/** #243: support points of a course sorted by time (a step's rows, the ends of a line or the table). */
function seriesPoints(s) {
  const pts = s.mode === "timed" ? (s.rows || []).map(r => ({ at: r.from, value: r.value }))
    : s.mode === "linear" ? [s.start, s.end].filter(Boolean) : s.mode === "table" ? s.points || [] : [];
  return pts.map(p => ({ ...p })).filter(p => Number.isFinite(pointMonth(p))).sort((x, y) => pointMonth(x) - pointMonth(y));
}
const scaleValues = (s, k) => JSON.parse(JSON.stringify(s), (key, v) => key === "value" && typeof v === "number" ? Math.round(v * k * 1e6) / 1e6 : v);
/**
 * #243: a course converted to another form, as close to the old values as the form allows: steps, line and table
 * take over the support points, a factor takes the value at the simulation start as base and the course divided by it.
 */
function convertSeries(s, mode) {
  const a = startM(), first = seriesAt(s, a), pts = seriesPoints(s), endM = msMonth("end");
  if (mode === "const") return { mode, value: s.mode === "timed" ? s.rows?.[0]?.value ?? 0 : first };
  if (mode === "timed") {
    const rows = pts.filter(p => pointMonth(p) > a).map(p => ({ from: p.age != null ? i2ym(pointMonth(p)) : p.at, value: p.value }));
    return { mode, rows: [{ from: "start", value: first }, ...rows] };
  }
  if (mode === "linear") return pts.length > 1 ? { mode, start: pts[0], end: pts[pts.length - 1] }
    : { mode, start: { at: "start", value: first }, end: { at: "end", value: Number.isFinite(endM) ? seriesAt(s, endM) : first } };
  if (mode === "table") return { mode, interpolation: s.mode === "timed" ? "step" : "linear", points: pts.length ? pts : [{ at: "start", value: first }] };
  return { mode, base: first, factor: first && s.mode !== "const" && s.mode !== "factor" ? scaleValues(s, 1 / first) : { mode: "const", value: 1 } };
}
/** #232: The amount per month next to a field per year («= 10 000 / Monat»). */
const perMonthText = v => tr("flows.perMonth", { v: fmtInt(v ?? 0) });
const perMonth = path => `<span class="computed permonth" data-permonth="${path}">${perMonthText(getP(path))}</span>`;
const C = (key, arg = "") => `<span class="computed" data-computed="${key}" data-arg="${arg}"></span>`;

// ---------- Sections ----------
const R = {};
R.general = () => `
  <h2>${tr("section.general")}</h2><p class="lead">${tr("general.intro")}</p>
  <div class="card"><h3>${tr("general.project")}</h3><div class="grid2">${F("project.name", tr("general.projectName"), "text")}
    <label class="field"><span>${tr("general.currency")}</span>${Sel("project.currency", [["CHF", tr("general.chf")], ["EUR", tr("general.eur")], ["USD", tr("general.usd")]], { rerender: 1 })}</label></div>
    <p class="hint">${tr("general.currencyHint")}</p></div>
  <div class="card"><h3>${tr("general.persons")}</h3><div class="persons">
    <div class="person">${state.partner ? `<b>${tr("general.person1")}</b>` : ""}${F("household.persons.0.name", tr("general.name"), "text")}${F("household.persons.0.birth", tr("general.birth"), "month")}${C("age0")}</div>
    ${state.partner
      ? `<div class="person"><div class="phead"><b>${tr("general.person2")}</b><button class="btn ghost small" data-action="partner-del" title="${tr("general.removePerson2")}">✕</button></div>${F("household.persons.1.name", tr("general.name"), "text")}${F("household.persons.1.birth", tr("general.birth"), "month")}${C("age1")}</div>`
      : `<div class="person ghost"><button class="btn" data-action="partner-add">${tr("general.addPerson2")}</button><span class="computed">${tr("general.doubleIncome")}</span></div>`}
  </div>
  <div style="margin-top:14px">${Chk("household.married", tr("general.married"))}</div>
  ${state.partner ? `<div class="grid2" style="margin-top:10px">${state.household.married ? F("household.ahvCoupleCap", tr("general.ahvCap"), "chf") : ""}</div>
    <p class="hint">${state.household.married ? tr("general.ahvCapMarried") : tr("general.ahvCapSingle")} ${tr("general.householdHint")}</p>`
    : `<p class="hint">${tr("general.marriedSingleHint")}</p>`}</div>
  <div class="card"><h3>${tr("general.period")}</h3>
    <div class="grid2"><div>${tr("general.start", { c: C("ms", "start") })}</div><div>${tr("general.end", { c: C("ms", "end") })}</div></div>
    <p class="hint">${tr("general.periodHint")}</p></div>`;

R.milestones = () => { const l = limits(); return `
  <h2>${tr("section.milestones")}</h2><p class="lead">${tr("milestones.intro")}</p>
  <div class="card"><table class="tbl"><tr><th>${tr("milestones.name")}</th><th>${tr("milestones.kind")}</th><th>${tr("milestones.value")}</th><th>${tr("milestones.result")}</th><th></th></tr>
  ${state.milestones.map((ms, i) => { const sys = sysMs(ms); return `<tr class="${ms.id === flashId ? "flash" : ""}">
    <td>${FIXED_MS_IDS.includes(ms.id) && !sys ? `<span title="${tr("milestones.fixedTitle")}">${esc(msLabel(ms))}</span>` : sys ? `<span title="${tr("milestones.fixedTitle")}">${esc(msLabel(ms))}</span><br><span class="computed">${sys.kind === "ahv" ? tr("milestones.ahvRange", { ahvMin: l.ahvMin, ahvMax: l.ahvMax }) : tr("milestones.pkRange", { pkMin: l.pkMin, pkMax: l.pkMax })} ${tr("unit.years")}</span>` : F(`milestones.${i}.name`, "", "text", { bare: true })}</td>
    <td>${Sel(`milestones.${i}.kind`, [["date", tr("milestones.date")], ["age", tr("milestones.ageOf")]], { rerender: 1 })}</td>
    <td style="width:190px">${ms.kind === "date" ? F(`milestones.${i}.date`, "", "month", { bare: true }) : `<div class="refwrap">${sys ? `<span class="computed" style="padding:8px 4px">${esc(person(ms.person).name || tr("milestones.person") + (sys.index + 1))}</span>` : Sel(`milestones.${i}.person`, state.household.persons.map(p => [p.id, p.name]))}${F(`milestones.${i}.years`, "", "years", { bare: true, unit: tr("unit.yearsShort") })}</div>`}</td>
    <td>${C("ms", ms.id)}</td>
    <td>${msLocked(ms) ? `<button class="btn ghost small" disabled title="${esc(sys ? tr("milestones.fixedPerPerson", { name: msLabel(ms) }) : tr("milestones.startEndFixed"))}">✕</button>`
      : `<button class="btn ghost small" data-action="ms-del" data-i="${i}" title="${tr("milestones.delete")}">✕</button>`}</td></tr>`; }).join("")}
  </table><div style="margin-top:10px"><button class="btn small" data-action="ms-add">${tr("milestones.add")}</button></div>
  <p class="hint">${tr("milestones.ageHint")}</p>
  <p class="hint">${tr("milestones.limitsHint", { pkMin: l.pkMin, pkMax: l.pkMax, ahvMin: l.ahvMin, ahvMax: l.ahvMax })}</p></div>`; };

function PensionCards(root, who) {
  const pk = `${root === "partner" ? "partner" : "pension"}.pk`, ahv = `${root === "partner" ? "partner" : "pension"}.ahv`, t = who ? ` · ${esc(who)}` : "";
  // #253: «Ohne Pensionskasse» at the top of the card; when on, the pension fund fields are grey and read-only
  // (the values stay in the plan, the engine counts the pension fund as 0).
  const n = root === "partner" ? 1 : 0, off = !!state.household.persons[n]?.noPensionFund;
  return `<div class="card" id="pkCard${n}"><h3>${tr("pension.pkTitle", { v: t })}</h3>
    ${Chk(`household.persons.${n}.noPensionFund`, tr("pension.noPensionFund"), { rerender: 1 })}
    <p class="hint">${tr("pension.noPensionFundHint", { max: fmtInt(limits().pillar3aMaxLarge) })}</p>
    <fieldset class="locked-area" id="pkBody${n}" ${off ? "disabled" : ""}><div class="grid2">
    ${F(`${pk}.capital`, tr("pension.pkCapital", { startTxt: startTxt() }), "chf")}
    ${fixedAt(n, "pk")}
    ${F(`${pk}.lumpsum`, tr("pension.lumpsum"), "pct")}
    ${F(`${pk}.conversion`, tr("pension.conversion"), "pct")}</div>
    <div style="margin-top:14px">${Series(`${pk}.contribution`, tr("pension.contribution"), "chf")}${Series(`${pk}.interest`, tr("pension.interest"), "pct")}</div></fieldset></div>
  <div class="card"><h3>${tr("common.ahv")}${t}</h3><div class="grid2"><div>${F(`${ahv}.monthly`, tr("pension.rentAtStart"), "chf", { hint: tr("pension.rentAtStartHint") })}${C("ahvEff", root === "partner" ? 1 : 0)}</div>
    ${fixedAt(root === "partner" ? 1 : 0, "ahv")}</div>
    <div style="margin-top:10px">${Chk(`${ahv}.indexed`, tr("pension.indexed"))}</div></div>
  ${CareCard(n, t)}`;
}
/** #271: care home of person `n`: entry (age, milestone or date), costs per month and «Wohneigentum verkaufen». */
function CareCard(n, t) {
  const P = `household.persons.${n}.careHome`, c = getP(P);
  const body = c ? `<div class="grid2">
      <label class="field"><span>${tr("care.entry")}</span>${CareWhen(P, n)}</label>
      ${F(P + ".monthly", tr("care.monthly"), "chf", { hint: tr("care.monthlyHint") })}</div>
    <div style="margin-top:10px">${Chk(P + ".sell", tr("care.sell"))}</div>
    <p class="hint">${tr(state.partner ? "care.hintCouple" : "care.hintSingle")}</p>` : `<p class="hint">${tr("care.none")}</p>`;
  return `<div class="card" id="careCard${n}"><h3>${tr("care.title", { v: t })}</h3>
    <label class="check"><input type="checkbox" data-action="care-toggle" data-n="${n}" ${c ? "checked" : ""}> ${tr("care.on")}</label>
    ${body}</div>`;
}
/** #271: entry into the care home: age of the person, a milestone or a fixed date (with the age or the date beside it). */
function CareWhen(path, n) {
  const c = getP(path), cur = c.at == null ? "age" : isYM(c.at) ? "__date" : c.at;
  const opt = (v, l) => `<option value="${esc(v)}" ${v === cur ? "selected" : ""}>${esc(l)}</option>`;
  return `<div class="refwrap"><select class="plain" data-path="${path}" data-type="carewhen" data-n="${n}">
    ${opt("age", tr("care.age"))}
    <optgroup label="${tr("ref.milestone")}">${state.milestones.map(ms => opt(ms.id, msLabel(ms))).join("")}</optgroup>
    ${opt("__date", tr("ref.fixedDate"))}</select>
    ${c.at == null ? F(path + ".age", "", "years", { bare: true, unit: tr("unit.yearsShort") }) : isYM(c.at) ? F(path + ".at", "", "month", { bare: true }) : ""}</div>`;
}
/** #271: «age», a milestone or «__date»; the point in time stays the same where it can. */
function setCareWhen(c, v, n) {
  const pid = state.household.persons[n].id, m = c.at == null ? ym2i(person(pid).birth) + Math.round((+c.age || 0) * 12) : refMonth(c.at);
  const known = Number.isFinite(m) && m !== Infinity ? m : startM();
  delete c.at; delete c.age;
  if (v === "age") c.age = Math.max(0, Math.round(ageAt(known, pid) * 100) / 100);
  else c.at = v === "__date" ? i2ym(known) : v;
}
/** #51: Pension fund and AHV start at the fixed milestones of the person. */
function fixedAt(index, kind) {
  const id = index ? (kind === "pk" ? "retire2" : "ahv2") : (kind === "pk" ? "retire" : "ahv");
  return `<label class="field"><span>${tr("pension.payoutFrom")}</span><span class="computed" style="padding:8px 0">${tr("pension.fixedAtText", { esc: esc(msLabel(state.milestones.find(m => m.id === id)) ?? id), c: C("ms", id) })}</span></label>`;
}
R.pension = () => `
  <h2>${tr("section.pension")}</h2><p class="lead">${tr("pension.subtitle")}${state.partner ? tr("pension.bothPersons") : ""}${tr("pension.pillar3aSeparate")}</p>
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
    const k = state.pillar3a.findIndex(a => a.person === pp.id), who = state.household.persons.length > 1 ? ` · ${esc(pp.name || tr("pillar3a.person") + (pi + 1))}` : "";
    const ahvId = pi ? "ahv2" : "ahv", ahv = msMonth(ahvId);
    if (k < 0) return `<div class="card"><h3>${tr("pillar3a.title", { who })}</h3><p class="hint">${tr("pillar3a.none")}</p><button class="btn small" data-action="p3a-add" data-person="${esc(pp.id)}">${tr("pillar3a.add")}</button></div>`;
    const A = `pillar3a.${k}`, acc = state.pillar3a[k];
    const uses = hasIndirectFor(pp.id) ? [["wealth", tr("pillar3a.toWealth")], ["repay", tr("pillar3a.repayMortgage")]] : [["wealth", tr("pillar3a.toWealth")]];
    const earliest = Number.isFinite(ahv) ? ymShow(i2ym(ahv - 60)) : "?";
    const events = (lastSim?.p3a || []).filter(e => state.household.persons.length < 2 || e.text.includes(`(${pp.name})`));
    return `<div class="card"><h3>${tr("pillar3a.title", { who })}<button class="btn ghost small" style="float:right" data-action="list-del" data-path="pillar3a" data-i="${k}" title="${tr("pillar3a.remove")}">✕</button></h3>
      <div class="grid3">${F(A + ".balance", tr("pillar3a.balance", { startTxt: startTxt() }), "chf")}${F(A + ".rate", tr("pillar3a.interest"), "pct")}
        ${F(A + ".contribution", tr("pillar3a.contribution"), "chf", { hint: tr(pp.noPensionFund ? "pillar3a.contributionHintLarge" : "pillar3a.contributionHint", { pillar3aMax: fmtInt(pp.noPensionFund ? l.pillar3aMaxLarge : l.pillar3aMax), ahv: Number.isFinite(ahv) ? ymShow(i2ym(ahv)) : "?" }) })}</div>
      <h4>${tr("pillar3a.payoutPlan")}</h4>
      <table class="tbl"><tr><th>${tr("pillar3a.when")}</th><th>${tr("pillar3a.result")}</th><th>${tr("pillar3a.amount")}</th><th>${tr("pillar3a.use")}</th><th title="${tr("pillar3a.fromPreview")}">${tr("pillar3a.balanceAfter")}</th><th></th></tr>
      ${(acc.payouts || []).map((x, j) => {
        const P = `${A}.payouts.${j}`;
        return `<tr data-p3a-row="${j}"><td style="width:220px"><div class="refwrap">${Sel(P + ".kind", [["date", tr("pillar3a.date")], ["age", tr("pillar3a.age")]], { rerender: 1 })}${x.kind === "age" ? F(P + ".years", "", "years", { bare: true, unit: tr("unit.yearsShort") }) : F(P + ".date", "", "month", { bare: true })}</div></td>
        <td>${C("p3aat", `${k}.${j}`)}</td>
        <td style="width:170px">${F(P + ".amount", "", "chfopt", { bare: true })}</td>
        <td>${Sel(P + ".use", uses)}</td>
        <td style="white-space:nowrap">${C("p3apay", `${k}.${j}`)}</td>
        <td><button class="btn ghost small" data-action="list-del" data-path="${A}.payouts" data-i="${j}" title="${tr("series.removeRow")}">✕</button></td></tr>`;
      }).join("")}</table>
      <div style="margin-top:10px"><button class="btn small" data-action="p3a-payout-add" data-k="${k}">${tr("pillar3a.addPayout")}</button></div>
      <p class="hint">${C("p3arest", k)}</p>
      <div class="grid2" style="margin-top:10px"><label class="field"><span>${tr("pillar3a.restAtAhv", { ahv: Number.isFinite(ahv) ? ymShow(i2ym(ahv)) : "?" })}</span>${Sel(A + ".use", uses)}</label></div>
      <p class="hint">${tr("pillar3a.payoutHint", { earliest })}</p>
      <p class="hint">${lastSim ? (events.length ? events.map(e => tr("pillar3a.preview", { ymShow: ymShow(i2ym(e.m)), text: esc(e.text) })).join("<br>") : tr("pillar3a.previewNone")) : ""}</p></div>`;
  }).join("");
  return `<h2>${tr("section.pillar3a")}</h2><p class="lead">${tr("pillar3a.intro")}</p>${cards}
    <p class="hint">${tr("pillar3a.maxHint", { pillar3aMax: fmtInt(l.pillar3aMax), pillar3aMaxLarge: fmtInt(l.pillar3aMaxLarge) })}</p>`;
};

R.flows = () => `
  <h2>${tr("section.flows")}</h2><p class="lead">${tr("flows.intro")}</p>
  <div class="card">${Series("flows.income", state.partner ? tr("flows.incomeOf", { esc: esc(state.household.persons[0].name) }) : tr("flows.income"), "chfyear", "", tr("flows.incomeHint"))}
    <div style="margin:-4px 0 12px">${Chk("flows.incomeIndexed", INCOME_INDEXED)}</div>${incomeEnds("flows.incomeEndsAtRetirement")}
    ${state.partner ? `${Series("partner.income", tr("flows.incomeOf", { esc: esc(state.household.persons[1].name) }), "chfyear", "", tr("flows.incomeHint"))}
    <div style="margin:-4px 0 12px">${Chk("partner.incomeIndexed", INCOME_INDEXED)}</div>${incomeEnds("partner.incomeEndsAtRetirement")}` : ""}
    ${Series("flows.spending", tr("flows.spending"), "chf", state.partner ? tr("flows.spendingNote") : tr("flows.todayPrices"))}
    <p class="hint">${C("housingshare")}</p></div>
  <div class="card"><h3>${tr("flows.oneOff")} <span class="sub">${tr("flows.oneOffHint")}</span></h3>
    <table class="tbl"><tr><th>${tr("flows.date")}</th><th>${tr("flows.amount")}</th><th>${tr("flows.description")}</th><th></th></tr>
    ${state.flows.extras.map((x, i) => `<tr><td style="width:130px">${F(`flows.extras.${i}.date`, "", "month", { bare: true })}</td><td style="width:200px">${F(`flows.extras.${i}.amount`, "", "chf", { bare: true })}</td><td>${F(`flows.extras.${i}.label`, "", "text", { bare: true })}</td>
      <td><button class="btn ghost small" data-action="list-del" data-path="flows.extras" data-i="${i}">✕</button></td></tr>`).join("")}</table>
    <div style="margin-top:10px"><button class="btn small" data-action="extra-add">${tr("flows.addOneOff")}</button></div></div>`;

R.wealth = () => `
  <h2>${tr("section.wealth")}</h2><p class="lead">${tr("wealth.intro")}</p>
  <div class="card"><h3>${tr("wealth.balances", { startTxt: startTxt() })}</h3><div class="grid3">${F("wealth.stocks", tr("wealth.stocks"), "chf", { hint: C("wquote", "stocks") })}${F("wealth.bonds", tr("wealth.bonds"), "chf", { hint: C("wquote", "bonds") })}${F("wealth.cash", tr("wealth.cash"), "chf")}</div>
    <p class="hint">${C("wealthsum")}</p></div>
  <div class="card">${Series("wealth.stockShare", tr("wealth.stockShare"), "pct")}
    <p class="hint">${tr("wealth.cashHint", { fixedReserveMonths: FIXED_RESERVE_MONTHS })}</p></div>
  ${strategyCard()}`;

/** #69: Indices of the investment strategy: [key, name, explanation]. */
let STOCK_INDICES;
onLanguage(() => STOCK_INDICES = [
  ["worldJst", tr("index.world"), tr("index.worldDesc")],
  ["worldCustom", tr("index.worldCustom"), tr("index.worldCustomDesc")],
  ["chJst", tr("index.swiss"), tr("index.swissDesc")],
  ["sp500", tr("index.sp500"), tr("index.sp500Desc")]
]);
let BOND_INDICES;
onLanguage(() => BOND_INDICES = [
  ["ch", tr("index.confed10"), tr("index.confed10Desc")],
  ["usa", tr("index.us10"), tr("index.us10Desc")],
  ["chShort", tr("index.confedShort"), tr("index.confedShortDesc")]
]);
const REGION_KEYS = ["ch", "usa", "eu", "apac"];
function strategyCard() {
  const s = state.returns.strategy, desc = (list, k) => esc(list.find(x => x[0] === k)?.[2] || "");
  const weights = s.stocks !== "worldCustom" ? "" : `<div class="grid2">${F("returns.strategy.weights.ch", tr("strategy.swiss"), "pct")}${F("returns.strategy.weights.usa", "USA", "pct")}
      ${F("returns.strategy.weights.eu", "EU", "pct", { hint: tr("strategy.eu") })}${F("returns.strategy.weights.apac", tr("strategy.apac"), "pct", { hint: tr("strategy.apacCountries") })}</div>
    <span data-path="returns.strategy.weights"></span>
    <p class="hint">${tr("strategy.weightsHint", { c: C("wsum") })}</p>`;
  const info = infoPop(`<p>${tr("strategy.info")}</p>
    <p>${tr("strategy.annualHint")}</p>
    <p>${tr("strategy.jstLicense")}</p>`);
  return `<div class="card"><h3>${tr("strategy.title", { info })}</h3>
    <div class="grid2"><label class="field"><span>${tr("strategy.stocks")}</span>${Sel("returns.strategy.stocks", STOCK_INDICES.map(([k, l]) => [k, l]), { rerender: 1 })}<span class="computed">${desc(STOCK_INDICES, s.stocks)}</span></label>
      <label class="field"><span>${tr("strategy.bonds")}</span>${Sel("returns.strategy.bonds", BOND_INDICES.map(([k, l]) => [k, l]), { rerender: 1 })}<span class="computed">${desc(BOND_INDICES, s.bonds)}</span></label></div>
    ${weights}
    <p class="hint">${C("strategyinfo")}</p>
    <div class="libbtns"><button class="btn small" data-action="import-shiller">${tr("strategy.updateShiller")}</button>
      <button class="btn small" data-action="import-snb">${tr("strategy.updateSnb")}</button></div></div>`;
}

let SOURCES;
onLanguage(() => SOURCES = [
  ["fixed", tr("source.fixed"), tr("source.fixedDesc"), true],
  ["historical", tr("source.historical"), tr("source.historicalDesc"), true],
  ["rolling", tr("source.rolling"), tr("source.rollingDesc"), true],
  ["bootstrap", tr("source.bootstrap"), tr("source.bootstrapDesc"), true],
  ["parametric", tr("source.parametric"), tr("source.parametricDesc"), true]
]);
/** #119: Abbreviation of each return source for the name of a saved result. */
let SOURCE_SHORT;
onLanguage(() => SOURCE_SHORT = { fixed: tr("source.fixedShort"), historical: tr("source.historicalShort"), rolling: tr("source.rollingShort"), bootstrap: "MC hist.", parametric: "MC param." });
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
  const fields = [u.fixedStocks && F("returns.fixed.stocks", tr("wealth.stocks"), "pct"), u.fixedBonds && F("returns.fixed.bonds", tr("wealth.bonds"), "pct"),
    u.fixedInflation && F("returns.fixed.inflation", tr("returns.inflation"), "pct"), u.fixedSaron && F("returns.fixed.saron", tr("returns.saron"), "pct")].filter(Boolean);
  const fixedTitle = { fixed: tr("returns.fixedTitle"), parametric: u.fixedBonds ? tr("returns.bondsInflSaron") : tr("returns.saronYear") }[src]
    || (u.fixedStocks ? tr("returns.extrapolation") : tr("returns.saronYear"));
  const fixedHint = src === "fixed" ? "" : src === "parametric" ? (u.fixedBonds ? tr("returns.stocksOnly") : tr("returns.saronConst"))
    : [u.fixedStocks && tr("returns.afterSeries"), u.fixedSaron && tr("returns.saronOutside")].filter(Boolean).join(" ");
  const fixed = fields.length ? `<div class="card"><h3>${fixedTitle}</h3><div class="grid2">${fields.join("")}</div>${fixedHint ? `<p class="hint">${fixedHint}</p>` : ""}</div>` : "";
  const series = u.series ? `<div class="card"><h3>${tr("returns.fromStrategy")}</h3><p style="margin:0">${tr("returns.changeStrategy", { c: C("strategyinfo") })}</p></div>` : "";
  let params = "";
  if (src === "historical") params = `<div class="card"><h3>${tr("returns.start")}</h3><div class="grid2">${F("returns.startDate", tr("sweep.histStart"), "month")}</div>
    <p class="hint">${tr("returns.startHint", { c: C("ms", "start") })}</p></div>`;
  if (src === "rolling") params = `<div class="card"><h3>${tr("returns.run")}</h3><p style="margin:0">${C("rolling")}</p></div>`;
  if (src === "bootstrap") params = `<div class="card"><h3>${tr("returns.bootstrap")}</h3><div class="grid3">${F("returns.bootstrap.paths", tr("returns.paths"), "num", { int: true })}${F("returns.bootstrap.block", tr("returns.blockLength"), "months")}${F("returns.bootstrap.seed", tr("returns.seed"), "num", { int: true })}</div>
    <p class="hint">${tr("returns.bootstrapHint")}</p>
    <p class="hint">${tr("returns.seedHint")}</p></div>`;
  if (src === "parametric") {
    const P = (k, label) => F("returns.parametric." + k, label, PARAM_KEYS[k], { hint: C("psug", k) });
    params = `<div class="card"><h3>${tr("returns.stockModel")}</h3><p class="hint" style="margin-top:0">${C("pauto")}</p><div class="grid3">${F("returns.parametric.paths", tr("returns.paths"), "num", { int: true })}${P("mu", tr("returns.drift"))}${P("sigma", tr("returns.volatility"))}
    ${P("df", tr("returns.dof"))}${P("jumpLambda", tr("returns.jumps"))}${P("jumpMu", tr("returns.jumpMu"))}
    ${P("jumpSigma", tr("returns.jumpSigma"))}${F("returns.parametric.seed", tr("returns.seed"), "num", { int: true })}</div>
    <p class="hint">${tr("returns.modelHint")}</p></div>
    <div class="card"><h3>${tr("returns.bondsInfl")} <span class="sub">${tr("returns.displayOnly")}</span></h3><div id="paramJoint">${C("pjoint")}</div></div>`;
  }
  const extra = !u.extrapolate ? "" : `<div class="card"><h3>${tr("returns.extrapolationTitle")} <span class="sub">${tr("returns.seriesEnds")}</span></h3>
    <div class="grid2"><label class="field"><span>${tr("returns.method")}</span>${Sel("returns.extrapolation.method", [["fixed", tr("returns.fixedValues")], ["average", tr("returns.movingAvg")], ["geometric", tr("returns.movingGeoAvg")]], { rerender: 1 })}</label>
    ${r.extrapolation.method === "fixed" ? "" : F("returns.extrapolation.range", tr("returns.avgOver"), "num", { unit: tr("unit.years"), int: true })}</div></div>`;
  return `<h2>${tr("section.returns")}</h2><p class="lead">${tr("returns.intro")}</p>
    <div class="card"><div class="choices">${SOURCES.map(([k, t, d, ok]) => `<div class="choice ${k === src ? "on" : ""} ${ok ? "" : "off"}" ${ok ? `data-action="set" data-path="returns.source" data-value="${k}"` : ""}><b>${t}</b><small>${d}</small></div>`).join("")}</div></div>
    ${series}${params}${extra}${fixed}
    <p class="hint">${tr("returns.inflationHint")} ${src !== "fixed" ? tr("returns.previewBasis", { c: C("basis") }) : ""}</p>`;
};

let STRATS;
onLanguage(() => STRATS = [
  ["standard", tr("strat.standard"), tr("strat.standardDesc"), true],
  ["three_buckets", tr("strat.buckets"), tr("strat.bucketsDesc"), true],
  ["guardrails", tr("strat.guardrails"), tr("strat.guardrailsDesc"), true],
  ["fixed_rate", tr("strat.fixedRate"), tr("strat.fixedRateDesc"), true],
  ["vpw", tr("strat.vpw"), tr("strat.vpwDesc"), true],
  ["endowment", tr("strat.endowment"), tr("strat.endowmentDesc"), true]
]);
/** #119: Abbreviation of each strategy (3 buckets with guardrails separately) for the name of a saved result. */
let STRAT_SHORT;
onLanguage(() => STRAT_SHORT = { standard: tr("strat.standard"), three_buckets: tr("strat.bucketsShort"), three_buckets_guardrails: tr("strat.bucketsGuardShort"), guardrails: tr("strat.guardrails"), fixed_rate: tr("strat.fixedRateShort"), vpw: "VPW", endowment: tr("strat.endowmentShort") });
R.withdrawal = () => {
  const w = state.withdrawal;
  return `<h2>${tr("section.withdrawal")}</h2><p class="lead">${tr("withdrawal.intro")}</p>
  <div class="card"><div class="choices">${STRATS.map(([k, t, d, ok]) => `<div class="choice ${k === w.strategy ? "on" : ""} ${ok ? "" : "off"}" ${ok ? `data-action="set" data-path="withdrawal.strategy" data-value="${k}"` : ""}><b>${t}</b><small>${d}</small></div>`).join("")}</div></div>
  ${w.strategy === "three_buckets" ? `<div class="card"><h3>${tr("withdrawal.buckets")}</h3><div class="grid3">${F("withdrawal.buckets.cashYears", tr("withdrawal.bucket1"), "years")}${F("withdrawal.buckets.bondYears", tr("withdrawal.bucket2"), "years")}${F("withdrawal.buckets.refillThreshold", tr("withdrawal.sellRule"), "pct")}${F("withdrawal.buckets.reserveMonths", tr("withdrawal.reserveIntra"), "months")}</div>
    <div class="buckets">
      <div class="bucket"><div class="bn">${tr("withdrawal.bucket1")}</div><div class="bv">${C("bucket", "cash")}</div><div class="bar" style="background:var(--c3)"></div></div>
      <div class="bucket"><div class="bn">${tr("withdrawal.bucket2")}</div><div class="bv">${C("bucket", "bond")}</div><div class="bar" style="background:var(--c2)"></div></div>
      <div class="bucket"><div class="bn">${tr("withdrawal.bucket3")}</div><div class="bv">${C("bucket", "stock")}</div><div class="bar" style="background:var(--c1)"></div></div></div>
    <p class="hint">${tr("withdrawal.splitHint")}</p>
    <p class="hint">${tr("withdrawal.rulesHint")}</p>
    <div class="grid2"><label class="field"><span>${tr("withdrawal.spending")}</span>${Sel("withdrawal.buckets.spending", [["planned", tr("withdrawal.asPlanned")], ["guardrails", tr("withdrawal.withGuardrails")]], { rerender: 1 })}</label></div>
    <p class="hint">${bucketsGuardrails() ? tr("withdrawal.guardrailsHint") : tr("withdrawal.plannedHint")}</p></div>
    ${bucketsGuardrails() ? GuardrailsCard() + WithdrawalLimits() : ""}`
    : w.strategy === "fixed_rate" ? `<div class="card"><h3>${tr("strat.fixedRate")}</h3><div class="grid3">${F("withdrawal.fixedRate.rate", tr("withdrawal.rate"), "pct")}</div>
      <p class="hint">${C("wdinit")}</p>
      <p class="hint">${tr("withdrawal.fixedRateHint")}</p></div>${WithdrawalLimits()}`
    : w.strategy === "guardrails" ? GuardrailsCard() + WithdrawalLimits()
    : w.strategy === "vpw" ? `<div class="card"><h3>${tr("withdrawal.vpw")} <span class="sub">VPW</span></h3><div class="grid2">${F("withdrawal.vpw.horizonAge", tr("withdrawal.untilAge"), "num", { unit: tr("unit.yearsShort") })}${F("withdrawal.vpw.realReturn", tr("withdrawal.realReturn"), "pct")}</div>
      <p class="hint">${C("wdinit")}</p>
      <p class="hint">${tr("withdrawal.vpwHint")}</p></div>${WithdrawalLimits()}`
    : w.strategy === "endowment" ? `<div class="card"><h3>${tr("strat.endowment")} <span class="sub">${tr("withdrawal.yale")}</span></h3><div class="grid2">${F("withdrawal.endowment.rate", tr("withdrawal.targetRate"), "pct")}${F("withdrawal.endowment.smoothing", tr("withdrawal.weight"), "pct")}</div>
      <p class="hint">${C("wdinit")}</p>
      <p class="hint">${tr("withdrawal.endowmentHint")}</p></div>${WithdrawalLimits()}`
    : `<div class="card"><h3>${tr("strat.standard")}</h3>${Series("wealth.reserveMonths", tr("withdrawal.reserve"), "months", tr("withdrawal.reserveHint"))}
      <p class="hint">${tr("withdrawal.stockShareHint", { fixedReserveMonths: FIXED_RESERVE_MONTHS })}</p></div>`}`;
};

/** #6: 3 buckets whose spending is controlled by the guardrails. */
const bucketsGuardrails = (w = state.withdrawal) => w?.strategy === "three_buckets" && w.buckets?.spending === "guardrails";
/** Key of the withdrawal strategy of a plan; 3 buckets with guardrails has its own (as in the optimization). */
const stratKey = w => bucketsGuardrails(w) ? "three_buckets_guardrails" : w?.strategy;
function GuardrailsCard() {
  return `<div class="card"><h3>${tr("strat.guardrails")} <span class="sub">${tr("guardrails.subtitle")}</span></h3><div class="grid2">${F("withdrawal.guardrails.initialRate", tr("guardrails.startRate"), "pct")}${F("withdrawal.guardrails.adjust", tr("guardrails.adjust"), "pct")}
      ${F("withdrawal.guardrails.upper", tr("guardrails.upper"), "pct")}${F("withdrawal.guardrails.lower", tr("guardrails.lower"), "pct")}</div>
      <p class="hint">${C("wdinit")}</p>
      <p class="hint">${tr("guardrails.hint")}</p></div>`;
}
function WithdrawalLimits() {
  return `<div class="card"><h3>${tr("limits.title")} <span class="sub">${tr("limits.hint")}</span></h3><div class="grid2">${F("withdrawal.floor", tr("limits.lower"), "pct")}${F("withdrawal.ceiling", tr("limits.upper"), "pct")}</div>
    <p class="hint">${tr("limits.example")}</p></div>`;
}
function TaxTable(key, label, xlabel) {
  return `<div class="series"><div class="shead"><span class="t">${label}</span></div><div class="sbody"><div><table class="tbl"><tr><th>${tr("taxTable.from", { xlabel })}</th><th>${tr("taxTable.rate")}</th><th></th></tr>
  ${getP("taxes." + key).map((r, i) => `<tr><td>${F(`taxes.${key}.${i}.0`, "", "chf", { bare: true })}</td><td>${F(`taxes.${key}.${i}.1`, "", "pct", { bare: true })}</td><td><button class="btn ghost small" data-action="list-del" data-path="taxes.${key}" data-i="${i}">✕</button></td></tr>`).join("")}</table>
  <div style="margin-top:8px"><button class="btn small" data-action="tax-add" data-key="${key}">${tr("taxTable.addStep")}</button></div></div>
  <svg class="mini" data-taxmini="${key}" viewBox="0 0 300 64" preserveAspectRatio="none" style="height:110px"></svg></div></div>`;
}
// ---------- Optimization ----------
let OPT_STRATS;
onLanguage(() => OPT_STRATS = [["standard", tr("strat.standard")], ["three_buckets", tr("strat.bucketsShort")], ["three_buckets_guardrails", tr("opt.bucketsGuardrails")], ["guardrails", tr("strat.guardrails")], ["fixed_rate", tr("strat.fixedRateShort")], ["vpw", "VPW"], ["endowment", tr("strat.endowmentShort")]]);
const OPT_DEFAULT = { goal: "consumption", minConsumption: 90, maxRuin: 5, ruinAge: "", weightMedian: 50, volWeight: 1, maxVol: "", paths: 5000, checkPaths: 10000, samples: 30, strategies: OPT_STRATS.map(x => x[0]).filter(k => k !== "three_buckets_guardrails"), ranges: {}, property: "planned" };
let optCfg = { ...OPT_DEFAULT };
const OPT_ACTIONS = {};
let lastOpt = null, optBusy = false, optError = null;
// The settings are stored in the plan (`optimization`, shares as decimals) and are saved with it.
function optFromPlan() {
  const o = state.optimization; if (!o) return { ...OPT_DEFAULT };
  const d = OPT_DEFAULT;
  return { goal: o.goal ?? d.goal, minConsumption: optRound((o.minConsumption ?? 0.9) * 100), maxRuin: optRound((o.maxRuin ?? 0.05) * 100), ruinAge: o.ruinAge ?? "", weightMedian: optRound((o.weightMedian ?? 0.5) * 100), volWeight: o.volWeight ?? 1,
    maxVol: o.maxVol == null ? "" : optRound(o.maxVol * 100), paths: o.paths || d.paths, checkPaths: o.checkPaths || d.checkPaths, samples: o.samples ?? d.samples,
    strategies: o.strategies?.length ? o.strategies : d.strategies.slice(), ranges: JSON.parse(JSON.stringify(o.ranges ?? {})), property: o.property ?? d.property };
}
function optToPlan(c) {
  return { ...(c.goal && c.goal !== "consumption" ? { goal: c.goal } : {}), ...(c.minConsumption !== 90 ? { minConsumption: optRound(optNum(c.minConsumption, 90) / 100) } : {}),
    maxRuin: optRound(c.maxRuin / 100), ruinAge: c.ruinAge === "" ? null : optNum(c.ruinAge, null), weightMedian: optRound(clamp(c.weightMedian / 100, 0, 1)), volWeight: c.volWeight,
    maxVol: c.maxVol === "" ? null : optRound(optNum(c.maxVol, 0) / 100), paths: c.paths, checkPaths: c.checkPaths, samples: c.samples, strategies: c.strategies,
    ...(Object.keys(c.ranges ?? {}).length ? { ranges: c.ranges } : {}), ...(c.property && c.property !== "planned" ? { property: c.property } : {}) };
}
/** Copy with sorted object keys: the same plan gives the same text after saving and opening (#201). */
const sortedKeys = v => Array.isArray(v) ? v.map(sortedKeys) : v && typeof v === "object" ? Object.fromEntries(Object.keys(v).sort().map(k => [k, sortedKeys(v[k])])) : v;
/** Key of the plan without the optimization settings: they do not change the result of an earlier optimization. */
// limits come from the settings, not the plan; #233: the affordability from the settings counts (as in the plan before)
const optKey = () => JSON.stringify(sortedKeys({ ...state, optimization: undefined, limits: undefined, realestate: { ...state.realestate, affordability: affordability() } }));
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
  <div class="row"><span class="hint" id="jobTxt" style="margin:0">${tr("job.starting")}</span><button class="btn small" data-action="job-cancel" id="jobCancel">${tr("job.cancel")}</button></div>${note ? `<p class="hint" style="margin:0">${note}</p>` : ""}</div>`;
async function jobTick() {
  if (!job) return;
  let p; try { p = await Engine.call("progress_get", { jobId: job.id }); } catch { return; }
  const bar = document.getElementById("jobBar"), txt = document.getElementById("jobTxt");
  if (!job || !bar || !txt) return;
  const frac = p.total > 0 ? Math.min(0.99, p.done / p.total) : 0, el = (performance.now() - job.t0) / 1000;
  bar.value = Math.round(frac * 1000);
  txt.textContent = p.cancelled ? tr("job.cancelling") : tr("job.elapsed", { pct: Math.round(frac * 100), elapsed: fmtShort(el) }) + (frac > 0.05 && el > 2 ? ` ${tr("job.remaining", { short: fmtShort(el / frac - el) })}` : "");
}
/** Runs `fn` and shows the progress meanwhile; returns `undefined` if it was cancelled. */
async function withJob(fn) {
  job = { t0: performance.now(), id: ++jobSeq };
  job.timer = setInterval(jobTick, 250); document.documentElement.dataset.job = "on";
  try { return await fn(); } finally { clearInterval(job.timer); job = null; document.documentElement.dataset.job = "off"; }
}
const isCancelled = e => e?.code === "cancelled"; // #171: by the code of the message
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
  if (er) { const { sec } = bench.us == null ? { sec: null } : scenSeconds(); er.textContent = sec == null ? "" : sec < 1 ? tr("estimate.underOne") : tr("estimate.approx", { sec: fmtShort(sec) }); }
  const wait = bench.error ? tr("estimate.notPossible", { error: esc(bench.error) }) : bench.us == null ? tr("estimate.running") : null;
  if (o) {
    const c = document.getElementById("opt-maxRuin") ? readOptCfg() : optFromPlan();
    o.innerHTML = wait ?? (c.strategies.length && c.paths > 0 && c.checkPaths > 0
      ? tr("estimate.optimization", { estimate: fmtEstimate(optSeconds(c)) }) : "");
  }
  const ew = document.getElementById("est-warn");
  if (ew) {
    const sec = bench.us == null ? null : scenSeconds().sec, slow = sec != null && sec > SLOW_RUN_SECONDS;
    ew.innerHTML = slow ? "⚠" : "";
    ew.title = slow ? tr("estimate.slowRun", { sec: fmtShort(sec), slowRunSeconds: SLOW_RUN_SECONDS }) : "";
    ew.setAttribute("role", slow ? "img" : "presentation"); ew.setAttribute("aria-label", ew.title);
  }
  if (sc) {
    const { runs, sec } = scenSeconds(), plain = !state.scenarios.sweeps.length;
    sc.innerHTML = wait ?? `${tr("estimate.title")} ${plain ? tr("estimate.simulation") : tr("estimate.scenarios")}: ${fmtEstimate(sec, [0.7, 1.5])}${plain ? "" : tr("estimate.runs", { runs: fmtInt(runs) })}.`;
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
const rangeUnit = d => msgText(d.unit);
const variedParams = s => s.params.filter(d => !rangeOf(s.key, d).fixed);
function optRangesHtml() {
  if (!optCat.data) return `<p class="hint">${tr("optRanges.loading")}</p>`;
  return optCat.data.map(s => {
    const n = variedParams(s).length, sel = optCfg.strategies.includes(s.key);
    const rows = s.params.map(d => {
      const r = rangeOf(s.key, d), id = `optR-${s.key}-${d.key}`, off = r.fixed ? "disabled" : "";
      return `<tr><td>${esc(msgText(d.label))}</td><td class="r">${d.current == null ? "–" : `${fmtNum(rangeShow(d, d.current))} ${rangeUnit(d)}`}</td>
        <td><label class="check"><input type="checkbox" id="${id}-var" ${r.fixed ? "" : "checked"}> ${tr("optRanges.vary")}</label></td>
        <td style="white-space:nowrap">${tr("optRanges.from")} <input class="num" id="${id}-lo" value="${rangeShow(d, r.lo)}" ${off} style="width:68px"> ${tr("optRanges.to")} <input class="num" id="${id}-hi" value="${rangeShow(d, r.hi)}" ${off} style="width:68px"> ${rangeUnit(d)}</td></tr>`;
    }).join("");
    return `<details class="optrange" ${sel ? "" : "style=\"opacity:.7\""}><summary><b>${esc(msgText(s.name))}</b> <span class="computed">${n === 0 ? tr("optRanges.allFixed") : tr("optRanges.varied", { n, length: s.params.length })}${sel ? "" : tr("optRanges.notChosen")}</span></summary>
      <table class="tbl"><tr><th>${tr("optRanges.params")}</th><th class="r">${tr("optRanges.inPlan")}</th><th></th><th>${tr("optRanges.range")}</th></tr>${rows}</table>
      <div style="margin:6px 0 10px"><button class="btn small" data-action="opt-range-reset" data-s="${s.key}">${tr("optRanges.defaults")}</button></div></details>`;
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
  if (sum) { const n = variedParams(s).length; sum.textContent = (n === 0 ? tr("optRanges.allFixed") : tr("optRanges.varied", { n, length: s.params.length })) + (optCfg.strategies.includes(sk) ? "" : tr("optRanges.notChosen")); }
}
document.addEventListener("change", e => { if (/^optR-/.test(e.target.id || "") && e.target.closest("#main")) onRangeInput(e); });
OPT_ACTIONS["opt-range-reset"] = d => {
  const r = { ...(optCfg.ranges ?? {}) }; delete r[d.s]; optCfg.ranges = r; saveOptCfg(); renderSection();
};
const optParam = p => `${esc(msgText(p[1]))} ${optVal(p)}`;
/** #197: Nominal end wealth at these percentiles (engine: `END_PERCENTS`). */
const OPT_END_P = [1, 5, 50, 95, 99];
const optEndCells = m => OPT_END_P.map((p, i) => `<td class="r">${m.endTotal ? fmtInt(m.endTotal[i]) : "–"}</td>`).join("");
/** #205: From this share of months at the floor the hint appears (engine: `FLOOR_HINT_SHARE`). */
const OPT_FLOOR_HINT = 0.5;
/** #205: Share of the months from retirement at the floor; «–» without a floor and in results saved before #205. */
const optFloorCell = m => `<td class="r optfloor">${m.floorShare == null ? "–" : pctTxt(m.floorShare)}</td>`;
const optFloorHint = m => m.floorShare >= OPT_FLOOR_HINT ? `<br><small class="optfloorhint" style="color:var(--warn)">${tr("optResult.floorHint")}</small>` : "";
const optFloorHead = () => `<th class="r" title="${tr("optResult.floorTitle")}">${tr("optResult.floor")}</th>`;
/** Two header rows: the columns `cols` (cells) span both, the end wealth gets a group heading over its percentiles. */
const optHead = cols => `<thead><tr>${cols.replace(/<th/g, '<th rowspan="2"')}<th colspan="5" style="text-align:center" title="${tr("optResult.endTitle")}">${tr("optResult.endWealth")}</th></tr>
  <tr>${OPT_END_P.map(p => `<th class="r">${p} %</th>`).join("")}</tr></thead>`;
/** Name of a result row; with the property setting «both» or «never sell» also the variant (#197). */
const optName = (r, x) => msgText(x.strategy) + (r.property && r.property !== "planned" ? ` · ${x.neverSell ? tr("optResult.neverSell") : tr("optResult.asPlanned")}` : "");
/** Parameters of a result: after an optimization the found ones (and the fixed ones), in a comparison the plan's values. */
const optParamsHtml = (r, x) => r.compare ? `<small>${x.fixed.map(optParam).join(", ")}</small>`
  : `<small>${x.params.map(optParam).join(", ")}</small>${x.fixed?.length ? `<br><small class="computed">fix: ${x.fixed.map(optParam).join(", ")}</small>` : ""}`;
/** #200: One group per result: the metrics row and below it a sub-row over the full table width
 * (parameters, button «In Planung übernehmen»); the group keeps both together visibly. */
const optGroup = (cells, span, sub, cls = "") => `<tbody class="optgrp ${cls}"><tr class="optmain">${cells}</tr><tr class="optsub"><td colspan="${span}">${sub}</td></tr></tbody>`;
/** Content of the sub-row of a strategy: parameters left, notice and button right. */
const optSub = (r, x, i, violated) => `<div class="optsubin"><div>${optParamsHtml(r, x)}${optFloorHint(x.validated)}</div><div class="optact">${x.feasible ? "" : `<span style="color:var(--err)">${violated}</span> `}<button class="btn small" data-action="opt-apply" data-i="${i}">${tr("optResult.adopt")}</button></div></div>`;
/** Result rows of a report: the reference (plan unchanged) and every strategy, each as a group (#200). */
const optRows = (r, cells, span, violated) => optGroup(cells(tr("optResult.reference", { strategy: esc(msgText(r.reference.strategy)) }), r.reference.metrics), span, `<small class="computed">${tr("optResult.asInPlan")}</small>${optFloorHint(r.reference.metrics)}`)
  + r.outcomes.map((x, i) => optGroup(cells(optName(r, x) + (i === 0 && x.feasible ? " ★" : ""), x.validated), span, optSub(r, x, i, violated), x.feasible ? "" : "off")).join("");
const optStale = stale => stale ? `<p class="hint" style="color:var(--warn);margin:0 0 8px">${tr("optResult.stale")}</p>` : "";
/** Result for goal 2 «Kein Notverkauf von Immobilien»: per strategy the share of runs with an unplanned sale. */
function optResultKeep(r, q, stale, start = startM()) {
  const cells = (name, m) => `<td><b>${esc(name)}</b></td><td class="r"><b>${pctTxt(m.emergencyShare)}</b></td><td class="r">${pctTxt(m.ruin)}</td><td class="r">${fmtInt(m.median)}</td>
    <td class="r">${m.planned > 0 ? pctTxt(m.median / m.planned) : "–"}</td>${optFloorCell(m)}${optEndCells(m)}`;
  const none = !r.outcomes.some(x => x.feasible);
  return `<div class="card optres"><h3>${r.compare ? tr("optResult.titleGoal2Compare") : tr("optResult.titleGoal2")} <span class="sub">${r.compare ? tr("optResult.compareChecked", { paths: fmtInt(r.checkPaths), ymShow: ymShow(i2ym(start)) }) : tr("optResult.checked", { checkPaths: fmtInt(r.checkPaths), ymShow: ymShow(i2ym(start)) })}</span></h3>
    ${optStale(stale)}
    ${none ? `<p class="hint" style="color:var(--warn);margin:0 0 8px">${tr("optResult.none", { maxRuin: pctTxt(q.maxRuin), minConsumption: pctTxt(q.minConsumption) })}</p>` : ""}
    <div class="tscroll"><table class="tbl opttbl">${optHead(`<th>${tr("optResult.strategy")}</th><th class="r" title="${tr("optResult.unplannedTitle")}">${tr("optResult.unplanned")}</th><th class="r">${tr("optResult.ruin")}</th><th class="r">${tr("optResult.medianConsumption")}</th><th class="r" title="${tr("optResult.medianTitle")}">${tr("optResult.shareOfPlan")}</th>${optFloorHead()}`)}
    ${optRows(r, cells, 6 + OPT_END_P.length, tr("optResult.constraintsViolated"))}
    </table></div>
    <p class="hint">${tr("optResult.order")}${q.ruinAge ? ` ${tr("optResult.untilAge", { ruinAge: q.ruinAge })}` : tr("optResult.untilEnd")} ${tr("optResult.footer", { v: (r.elapsedMs / 1000).toFixed(1) })}</p></div>`;
}
/** Result for goal 1 «Konsum»: metrics, combined goal value with bar, withdrawal rate. */
function optResultConsumption(r, q, stale, start = startM()) {
  const w = q.weightMedian;
  const goal = m => (w * m.median + (1 - w) * m.p10) * Math.max(0, 1 - q.volWeight * m.volatility);
  const best = Math.max(1, goal(r.reference.metrics), ...r.outcomes.map(x => goal(x.validated)));
  const cells = (name, m) => `<td><b>${esc(name)}</b></td><td class="r">${pctTxt(m.ruin)}</td><td class="r">${fmtInt(m.median)}</td><td class="r">${fmtInt(m.p10)}</td><td class="r">${pctTxt(m.volatility)}</td>
    <td class="r">${fmtInt(goal(m))}<div style="height:4px;background:var(--accent);opacity:.5;width:${Math.max(0, Math.min(100, goal(m) / best * 100))}%;margin-top:2px;border-radius:2px"></div></td><td class="r">${pctTxt(m.drawRate ?? 0)}</td>${optFloorCell(m)}${optEndCells(m)}`;
  return `<div class="card optres"><h3>${r.compare ? tr("optimize.compareResult") : tr("optimize.result")} <span class="sub">${r.compare ? tr("optimize.compareChecked", { paths: fmtInt(r.checkPaths), ymShow: ymShow(i2ym(start)) }) : tr("optimize.checked", { checkPaths: fmtInt(r.checkPaths), ymShow: ymShow(i2ym(start)) })}</span></h3>
    ${optStale(stale)}
    <div class="tscroll"><table class="tbl opttbl">${optHead(`<th>${tr("optResult.strategy")}</th><th class="r">${tr("optResult.ruin")}</th><th class="r">${tr("optimize.median")}</th><th class="r">P10</th><th class="r">${tr("optimize.volatility")}</th><th class="r">${tr("optimize.goal")}</th><th class="r" title="${tr("optimize.goalTitle")}">${tr("optimize.rate")}</th>${optFloorHead()}`)}
    ${optRows(r, cells, 8 + OPT_END_P.length, tr("optimize.ruinViolated"))}
    </table></div>
    <p class="hint">${tr("optimize.ruinDef")}${q.ruinAge ? ` ${tr("optResult.untilAge", { ruinAge: q.ruinAge })}` : tr("optResult.untilEnd")} ${tr("optimize.ruinFooter", { maxRuin: pctTxt(q.maxRuin), round: Math.round(w * 100), round2: Math.round((1 - w) * 100), volWeight: q.volWeight })}</p>
    <p class="hint">${r.compare ? tr("optimize.compareFooter", { v: (r.elapsedMs / 1000).toFixed(1) }) : tr("optimize.searchFooter", { fitPaths: fmtInt(r.fitPaths), v: (r.elapsedMs / 1000).toFixed(1) })}</p></div>`;
}
/** #200: Result card of an optimization or comparison, rendered from the report `r` and its request `q` only. */
const optResultCard = (r, q, stale, start) => q.goal === "keepProperty" ? optResultKeep(r, q, stale, start) : optResultConsumption(r, q, stale, start);
R.optimize = () => {
  optCfg = optFromPlan();
  const o = lastOpt, stale = o && optKey() !== o.planJson;
  const single = ["fixed", "historical"].includes(state.returns.source);
  let res = "";
  if (optBusy) res = jobCard(tr("optimize.running"), tr("optimize.runningHint"));
  else if (optError) res = `<div class="card errcard">${esc(optError)}</div>`;
  else if (o) res = optResultCard(o.report, o.req, stale);
  return `<h2>${tr("section.optimize")}</h2><p class="lead">${tr("optimize.intro")}</p>
  ${single ? `<div class="card errcard">${tr("optimize.singlePathPre")}${state.returns.source === "fixed" ? tr("optimize.fixedReturns") : tr("optimize.historicalPath")}${tr("optimize.singlePathPost")}</div>` : ""}
  <div class="card"><h3>${tr("optimize.goal")}</h3>
    <label class="field" style="max-width:520px"><span>${tr("optimize.goalTitle2")}</span><select class="plain" id="opt-goal">
      <option value="consumption" ${optCfg.goal === "consumption" ? "selected" : ""}>${tr("optimize.goal1")}</option>
      <option value="keepProperty" ${optCfg.goal === "keepProperty" ? "selected" : ""} ${hasProperties() ? "" : "disabled"}>${tr("optimize.goal2")}</option></select>
      ${hasProperties() ? "" : `<span class="computed">${tr("optimize.goal2NeedsProperty")}</span>`}</label>
    <div class="grid3" style="margin-top:10px">
    ${optInput("maxRuin", tr("optimize.maxRuin"), "%")}
    ${optInput("ruinAge", tr("optimize.ruinAge"), tr("unit.yearsShort"), tr("optimize.ruinAgeHint"))}
    ${optCfg.goal === "keepProperty" ? optInput("minConsumption", tr("optimize.minConsumption"), "%", tr("optimize.minConsumptionHint"))
      : `${optInput("weightMedian", tr("optimize.weightMedian"), "%", tr("optimize.weightMedianHint"))}
    ${optInput("volWeight", tr("optimize.volWeight"), "×", tr("optimize.volWeightHint"))}
    ${optInput("maxVol", tr("optimize.maxVol"), "%", tr("optimize.maxVolHint"))}`}</div>
    ${optCfg.goal === "keepProperty" ? `<p class="hint">${tr("optimize.goal2Intro")}</p>` : ""}
    <h3 style="margin-top:14px">${tr("optimize.strategies")}</h3>
    <div style="display:flex;flex-wrap:wrap;gap:6px 20px">${OPT_STRATS.map(([k, t]) => `<label class="check"><input type="checkbox" id="optS-${k}" ${optCfg.strategies.includes(k) ? "checked" : ""}> ${t}</label>`).join("")}</div>
    ${hasProperties() ? `<label class="field" style="max-width:520px;margin-top:10px"><span>${tr("optimize.property")}</span><select class="plain" id="opt-property">
      ${[["planned", "optimize.propertyPlanned"], ["neverSell", "optimize.propertyNeverSell"], ["both", "optimize.propertyBoth"]].map(([v, k]) => `<option value="${v}" ${optCfg.property === v ? "selected" : ""}>${tr(k)}</option>`).join("")}</select></label>` : ""}
    <h3 style="margin-top:14px">${tr("optimize.strategyParams")}</h3>
    <p class="hint" style="margin:0 0 6px">${tr("optimize.strategyParamsHint")}</p>
    ${optRangesHtml()}
    <details style="margin-top:12px"><summary class="hint">${tr("optimize.advanced")}</summary><div class="grid3" style="margin-top:8px">
      ${optInput("paths", tr("optimize.fitPaths"), "", tr("optimize.fitPathsHint"))}${optInput("checkPaths", tr("optimize.checkPaths"), "")}${optInput("samples", tr("optimize.samples"), "")}</div></details>
    <div style="margin-top:14px;display:flex;flex-wrap:wrap;gap:8px"><button class="btn primary" data-action="opt-run" ${optBusy || single ? "disabled" : ""}>${optBusy ? tr("optimize.runningShort") : tr("optimize.start")}</button>
      <button class="btn" data-action="opt-compare" title="${tr("optimize.compareTitle")}" ${optBusy || single ? "disabled" : ""}>${tr("optimize.compare")}</button></div>
    <p class="hint est" id="est-opt" style="margin:8px 0 0"></p></div>
  ${res}${o && !optBusy && !optError ? `<footer class="rfoot" id="optFoot">${esc(DISCLAIMER_FOOT)}</footer>` : ""}`;
};
/** #140: Integer of the optimization; «50.000» is ambiguous and is not taken as 50 (the previous value stays). */
function optInt(text, prev) {
  if (ambiguousThousands(text)) { toast(ambiguousText()); return prev; }
  return Math.round(optNum(text, prev));
}
function readOptCfg() {
  const v = id => document.getElementById("opt-" + id)?.value ?? optCfg[id];
  const goalEl = document.getElementById("opt-goal");
  optCfg = { goal: goalEl ? goalEl.value : optCfg.goal, minConsumption: optNum(v("minConsumption"), 90), maxRuin: optNum(v("maxRuin"), 5), ruinAge: String(v("ruinAge")).trim(), weightMedian: optNum(v("weightMedian"), 50), volWeight: optNum(v("volWeight"), 1),
    maxVol: String(v("maxVol")).trim(), paths: optInt(v("paths"), optCfg.paths ?? 5000), checkPaths: optInt(v("checkPaths"), optCfg.checkPaths ?? 10000), samples: optInt(v("samples"), optCfg.samples ?? 30),
    strategies: OPT_STRATS.map(x => x[0]).filter(k => document.getElementById("optS-" + k)?.checked), ranges: optCfg.ranges ?? {}, property: v("property") ?? "planned" };
  return optCfg;
}
/** Optimization, or with `compare` (#197) the comparison of the strategies with the plan's values without search. */
async function optStart(compare) {
  if (await blockedByErrors()) return;
  const c = readOptCfg();
  if (!c.strategies.length) return toast(tr("optimize.chooseStrategy"));
  const req = { goal: c.goal, minConsumption: c.minConsumption / 100, strategies: c.strategies, maxRuin: c.maxRuin / 100, ruinAge: c.ruinAge === "" ? null : optNum(c.ruinAge, null), weightMedian: clamp(c.weightMedian / 100, 0, 1),
    volWeight: c.volWeight, maxVol: c.maxVol === "" ? null : optNum(c.maxVol, 0) / 100, paths: c.paths, checkPaths: c.checkPaths, samples: c.samples, seed: 1, ranges: c.ranges,
    compare, property: hasProperties() ? c.property : "planned" };
  saveOptCfg();
  optBusy = true; optError = null; renderSection();
  const planJson = optKey(), plan = withAffordability(state), path = currentPath;
  let done = null;
  try { const report = await withJob(() => Engine.call("optimize", { plan: state, request: req, path: currentPath, jobId: job.id })); lastOpt = done = { report, req, planJson }; }
  catch (e) { if (isCancelled(e)) toast(tr("optimize.cancelled")); else optError = errText(e); }
  optBusy = false;
  if (current === "optimize") renderSection();
  // #201: only a finished optimization or comparison is kept, not a cancelled or failed one
  if (done) await rememberOpt(done, plan, path);
  document.documentElement.dataset.optimized = optError ? "error" : "ok";
}
/** #201: Short hash (cyrb53) of a text: finds saved optimizations matching the plan from their header data alone. */
function textHash(s) {
  let h1 = 0xdeadbeef, h2 = 0x41c6ce57;
  for (let i = 0; i < s.length; i++) { const c = s.charCodeAt(i); h1 = Math.imul(h1 ^ c, 2654435761); h2 = Math.imul(h2 ^ c, 1597334677); }
  h1 = Math.imul(h1 ^ (h1 >>> 16), 2246822507) ^ Math.imul(h2 ^ (h2 >>> 13), 3266489909);
  h2 = Math.imul(h2 ^ (h2 >>> 16), 2246822507) ^ Math.imul(h1 ^ (h1 >>> 13), 3266489909);
  return (4294967296 * (2097151 & h2) + (h1 >>> 0)).toString(36);
}
/** Value of an optimization parameter as text (percent or number). */
const optVal = ([, , v, pc]) => pc ? (v * 100).toLocaleString("de-CH", { maximumFractionDigits: 1 }) + " %" : v.toLocaleString("de-CH", { maximumFractionDigits: 1 });
/** #201: Header data of a saved optimization: best strategy (first outcome) with its ruin and goal value. */
function optMeta(o, plan) {
  const r = o.report, best = r.outcomes[0], v = best?.validated || {};
  return { kind: "optimization", compare: !!r.compare, goal: o.req.goal, source: plan.returns.source, seriesName: seriesLabel(),
    strategy: best ? stratKey(best.patch?.withdrawal) : null, neverSell: !!best?.neverSell, feasible: !!best?.feasible,
    ruin: v.ruin ?? null, emergencyShare: v.emergencyShare ?? null, runs: r.checkPaths, planHash: textHash(o.planJson),
    params: best ? [...(best.params || []), ...(best.fixed || [])].map(p => [msgText(p[1]), optVal(p)]) : null };
}
/** #201: Keeps a finished optimization in «Gespeicherte Ergebnisse», with report, request and the plan at its start. */
async function rememberOpt(o, plan, path) {
  try {
    const m = optMeta(o, plan), now = new Date().toISOString();
    const base = (path && baseName(path).replace(/\.[rv]plan$/, "")) || plan.project.name || tr("refs.plan");
    const meta = { id: Date.now().toString(36) + Math.random().toString(36).slice(2, 8), created: now, name: runTitle(base, m, now), note: "", ...m };
    await Runs.add(meta, { plan, opt: o });
    document.documentElement.dataset.saved = (+document.documentElement.dataset.saved || 0) + 1;
  } catch (e) {
    toast(tr("runs.notListed") + errText(e));
    noteError(tr("runs.notSaved") + errText(e));
  }
}
/** #201: After opening a plan the newest saved optimization matching it becomes the result of the section «Optimierung». */
async function restoreOpt() {
  const key = optKey(), hash = textHash(key);
  try {
    for (const m of await Runs.list()) {
      if (m.kind !== "optimization" || m.planHash !== hash) continue;
      const d = await Runs.get(m.id);
      if (!d?.opt || d.opt.planJson !== key) continue;
      if (optKey() !== key || optBusy) return; // another plan opened meanwhile, or a new optimization running
      lastOpt = { report: d.opt.report, req: d.opt.req, planJson: key }; optError = null;
      if (current === "optimize" && $("#results").hidden) renderSection();
      return;
    }
  } catch { /* without the list there is nothing to show; no message (#201/AK8) */ }
  finally { document.documentElement.dataset.optRestored = (+document.documentElement.dataset.optRestored || 0) + 1; }
}
OPT_ACTIONS["opt-run"] = () => optStart(false);
OPT_ACTIONS["opt-compare"] = () => optStart(true);
OPT_ACTIONS["opt-apply"] = async d => {
  // #201: from a saved optimization; the plan of that time must match the open plan, otherwise ask first
  if (savedOpt) {
    const o = savedOpt, x = o.opt.report.outcomes[+d.i]; if (!x) return;
    const same = optKey() === o.opt.planJson;
    if (!same && !await ask(tr("savedOpt.otherTitle"), tr("savedOpt.otherText"), null, tr("optResult.adopt"))) return;
    optPatch(x.patch); markDirty();
    if (same) lastOpt = { ...o.opt, planJson: optKey() };
    showResults(false); goto("optimize");
    toast(o.opt.report.compare ? tr("optimize.appliedCompare", { strategy: msgText(x.strategy) }) : tr("optimize.applied", { strategy: msgText(x.strategy) }));
    return;
  }
  const x = lastOpt?.report.outcomes[+d.i]; if (!x) return;
  optPatch(x.patch);
  markDirty(); lastOpt.planJson = optKey(); renderSection();
  toast(lastOpt.report.compare ? tr("optimize.appliedCompare", { strategy: msgText(x.strategy) }) : tr("optimize.applied", { strategy: msgText(x.strategy) }));
};
/** Takes the parameters of an optimization result over into the open plan. */
function optPatch(pt) {
  state.withdrawal = { ...state.withdrawal, ...pt.withdrawal };
  state.wealth.stockShare = pt.wealth.stockShare;
  state.wealth.reserveMonths = pt.wealth.reserveMonths;
  // #197: a result computed with «Immobilien nie verkaufen» switches it on in the plan
  if (pt.neverSell) state.realestate.neverSell = true;
}

// ----- Property gains tax (rules per canton, editable) -----
/** #138: Templates of the property gains tax from the engine (crates/engine/assets/gain-templates.json): Zurich
 *  (tariff § 225 StG, StA eForm. 225.1, as of 09/2025) and Aargau (§ 109 StG). Loaded at startup. */
let gainTemplates = null;
const GAIN_ZH = () => structuredClone(gainTemplates.zurich);
const GAIN_AG = () => structuredClone(gainTemplates.aargau);
function gainCard() {
  const g = state.taxes.propertyGain, P = "taxes.propertyGain";
  const head = `<h3>${tr("gain.title")}</h3>`;
  const btns = (del) => `<button class="btn small" data-action="gain-zh">${tr("gain.templateZh")}</button> <button class="btn small" data-action="gain-ag">${tr("gain.templateAg")}</button>${del ? ` <button class="btn small" data-action="gain-del">${tr("gain.remove")}</button>` : ""}`;
  const needed = state.realestate.properties.some(p => p.status !== "rent");
  if (!g) return `<div class="card${needed ? " errcard" : ""}">${head}<p class="hint" style="margin:0 0 8px">${needed
    ? tr("gain.required")
    : tr("gain.notRequired")}</p>
    ${btns(false)} <button class="btn small" data-action="gain-own">${tr("gain.custom")}</button></div>`;
  const rows = (key, cols) => (g[key] ?? []).map((r, i) => `<tr><td>${F(`${P}.${key}.${i}.0`, "", cols[0], { bare: true, unit: cols[2] })}</td><td>${F(`${P}.${key}.${i}.1`, "", "pct", { bare: true })}</td>
    <td><button class="btn ghost small" data-action="list-del" data-path="${P}.${key}" data-i="${i}">✕</button></td></tr>`).join("");
  const aargau = g.model === "aargau";
  const model = `<div class="seg" style="margin-bottom:10px"><button class="${aargau ? "" : "on"}" data-action="gain-model" data-v="zurich">${tr("gain.modelZh")}</button><button class="${aargau ? "on" : ""}" data-action="gain-model" data-v="aargau">${tr("gain.modelAg")}</button></div>`;
  const tables = aargau
    ? `<div><table class="tbl"><tr><th>${tr("gain.holdingYears")}</th><th>${tr("gain.rate")}</th><th></th></tr>${rows("durationRates", ["num", "", tr("unit.yearsShort")])}</table>
      <div style="margin-top:8px"><button class="btn small" data-action="gain-add" data-key="durationRates">${tr("series.addRow")}</button></div>
      <p class="hint">${tr("gain.zhHint")}</p></div>`
    : `<div class="grid2"><div><table class="tbl"><tr><th>${tr("gain.step")}</th><th>${tr("taxTable.rate")}</th><th></th></tr>${rows("brackets", ["chf", "", cur()])}</table>
      <div style="margin-top:8px"><button class="btn small" data-action="gain-add" data-key="brackets">${tr("taxTable.addStep")}</button></div>
      <p class="hint">${tr("gain.agHint")}</p></div>
    <div><table class="tbl"><tr><th>${tr("gain.holdingYears")}</th><th>${tr("gain.surcharge")}</th><th></th></tr>${rows("duration", ["num", "", tr("unit.yearsShort")])}</table>
      <div style="margin-top:8px"><button class="btn small" data-action="gain-add" data-key="duration">${tr("series.addRow")}</button></div>
      <p class="hint">${tr("gain.surchargeHint")}</p></div></div>`;
  return `<div class="card">${head}${model}${tables}
    <div class="grid3" style="margin-top:10px">${F(P + ".exemptBelow", tr("gain.exempt"), "chf")}${F(P + ".reduction", tr("gain.reduction"), "pct")}</div>
    <div style="margin-top:10px">${Chk(P + ".useMultiplier", tr("gain.multiplier"))}</div>
    <p class="hint gaincheck">${tr("gain.title")}: ${tr("taxes.checkBeforeDecision")}</p>
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
  "gain-own-toggle": () => {
    if (state.taxes.propertyGainOwn) delete state.taxes.propertyGainOwn; else state.taxes.propertyGainOwn = true;
    markDirty(); renderSection();
  },
  "gain-del": () => { delete state.taxes.propertyGain; markDirty(); renderSection(); },
  "gain-add": d => {
    const t = (state.taxes.propertyGain[d.key] ??= []), last = t[t.length - 1];
    t.push(d.key === "brackets" ? [last ? last[0] : 10000, last ? last[1] : 0.1] : [last ? last[0] + 1 : 0, last ? last[1] : 0]);
    markDirty(); renderSection();
  }
});
/** #63: Cantons and municipalities of the tax data file (command `tax_locations`), loaded at startup so that the
 *  overview and the tab «Parameter» name the municipality without opening the section «Steuern». */
let taxLoc = null;
let CHURCH;
onLanguage(() => CHURCH = [["none", tr("church.none")], ["reformed", tr("church.reformed")], ["catholic", tr("church.catholic")], ["christCatholic", tr("church.christCatholic")]]);
const isLocation = plan => plan?.taxes?.mode === "location";
/** #65/AK5: the own rules of the property gains tax apply in the mode «Eigene Tabellen» or with the check box. */
const gainOwn = (plan = state) => !isLocation(plan) || !!plan.taxes.propertyGainOwn;
/** #65/AK1, AK3: canton of the property and the rule that applies there (mode «Wohnort», canton rules). */
function propertyGainRow(p, P) {
  if (gainOwn() || p.status === "rent" || !taxLoc) return "";
  const home = state.taxes.location?.canton, c = taxLoc.cantons.find(x => x.id === (p.canton || home));
  // #151/AK4: with a change of the place of residence the canton of residence in the month of the sale
  const homeName = taxLoc.cantons.find(x => x.id === home)?.name, moving = !!state.taxes.relocations?.length;
  return `<div class="grid2" style="margin-top:10px"><label class="field"><span>${tr("realestate.gainCanton")}</span>${Sel(P + ".canton", [["", moving ? tr("realestate.gainCantonHomeAtSale") : homeName ? tr("realestate.gainCantonHomeName", { name: homeName }) : tr("realestate.gainCantonHome")], ...taxLoc.cantons.map(x => [x.id, x.name])], { rerender: 1 })}</label></div>
    ${c ? `<p class="hint" data-gain-rule>${tr("realestate.gainRule", { canton: esc(c.name), rule: tr(`propertyGainRule.${c.id}`) })}</p>` : `<p class="hint">${tr("realestate.gainCantonMissing")}</p>`}
    ${c?.propertyGainApprox ? `<p class="hint warn" data-gain-approx>⚠ ${tr("realestate.gainApprox", { note: tr(`propertyGainApprox.${c.id}`) })}</p>` : ""}`;
}
/** #273: default share of the market value for the tax value, from the engine (`tax_locations`), as text. */
const taxValueDefaultPc = () => taxLoc?.taxValueDefault != null ? `${fmtPct(taxLoc.taxValueDefault)} %` : "…";
/** #229/AK1: tax value for the wealth tax (owned and planned): none (market value), amount or share of the market value.
 *  #273/AK2, AK3: in plans with the default (new plans) «none» is the default share; otherwise a hint offers it. */
function taxValueRow(p, i, P, o = {}) {
  const kind = p.taxValue?.kind ?? "", dflt = !!state.realestate.defaultTaxValue;
  const opts = [["", dflt ? tr("realestate.taxValueDefault", { pc: taxValueDefaultPc() }) : tr("realestate.taxValueNone")], ["amount", tr("realestate.taxValueAmount")], ["rate", tr("realestate.taxValueRate")]];
  const sel = `<select class="plain" data-action-change="tax-value-kind" data-i="${i}">${opts.map(([k, l]) => `<option value="${k}" ${k === kind ? "selected" : ""}>${esc(l)}</option>`).join("")}</select>`;
  const value = kind ? F(`${P}.taxValue.value`, "", kind === "rate" ? "pct" : "chf", { bare: true }) : "";
  // #249: in the card «Abzüge vom Vermögen» with the name of the property as label and the hint only once;
  // #259/AK1: a label from the plan's data is marked as such
  return `<div class="grid2" style="margin-top:10px" data-tax-value="${i}"><label class="field"><span${o.userText ? " data-no-tr" : ""}>${o.label ?? tr("realestate.taxValue")}</span><div class="refwrap tv">${sel}${value}</div></label></div>
    ${o.hint === false ? "" : `<p class="hint">${tr("realestate.taxValueHint")}</p>`}
    ${o.hint === false || dflt || kind ? "" : `<p class="hint" data-tax-value-offer="${i}">${tr("realestate.taxValueOffer", { pc: taxValueDefaultPc() })} <a href="#" data-action="tax-value-default">${tr("realestate.taxValueOfferLink")}</a></p>`}`;
}
/** #229: changing the kind keeps the tax value: an amount becomes its share of the market value (planned: purchase price) and back. */
document.addEventListener("change", e => {
  const el = e.target; if (!el.matches?.('select[data-action-change="tax-value-kind"]')) return;
  const p = state.realestate.properties[+el.dataset.i], old = p.taxValue, kind = el.value;
  const base = (p.status === "owned" ? p.worth ?? p.price : p.price) || 0;
  const share = !old ? 1 : old.kind === "rate" ? old.value : base > 0 ? old.value / base : 1;
  if (!kind) delete p.taxValue;
  else p.taxValue = { kind, value: kind === "rate" ? Math.round(share * 10000) / 10000 : Math.round(share * base) };
  invalidFields.delete(`realestate.properties.${el.dataset.i}.taxValue.value`);
  markDirty(); renderSection();
});
/** #65/AK5: card «Grundstückgewinnsteuer» in the mode «Wohnort»: check box for own rules, then the own rules. */
function locGainCard() {
  const own = gainOwn();
  return `<div class="card" id="gain-location"><h3>${tr("gain.title")}</h3>
    <label class="check"><input type="checkbox" id="gain-own" data-action="gain-own-toggle" ${own ? "checked" : ""}> ${tr("gain.ownRules")}</label>
    <p class="hint">${own ? tr("gain.ownRulesHint") : tr("gain.cantonRulesHint")}</p></div>${own ? gainCard() : ""}`;
}
/** #63/AK10: Place of residence as text: «Zürich (ZH), Kirchensteuer keine»; an unknown number with the canton. */
function locationText(loc = {}) {
  const c = taxLoc?.cantons.find(x => x.id === loc.canton), m = c?.municipalities.find(x => x[0] === loc.municipality);
  const church = CHURCH.find(x => x[0] === (loc.church ?? "none"))?.[1] ?? loc.church;
  return tr("params.locationText", { name: m ? m[1] : loc.municipality ?? "–", canton: loc.canton || "–", church });
}
/** #63/AK9: multipliers of a municipality (income or wealth) with the church of the chosen denomination. */
function multiplierText(what, m, church) {
  const k = CHURCH.findIndex(c => c[0] === (church ?? "none"));
  return tr("taxes.multipliers", { what, canton: fmtPct(m[0]), municipality: fmtPct(m[1]) }) + (k > 0 ? tr("taxes.multiplierChurch", { church: fmtPct(m[k + 1]) }) : "");
}
/** #151: places of residence as rows: the first is `taxes.location` (from the start), the others
 *  `taxes.relocations` (from a year or an event); `path` is the object of the row in the plan. */
function locRows(plan = state) {
  const t = plan?.taxes ?? {};
  return [{ path: "taxes.location", loc: t.location ?? {}, from: null }, ...(t.relocations ?? []).map((r, i) => ({ path: `taxes.relocations.${i}`, loc: r, from: r.from }))];
}
/** #272/AK1: all municipalities of the tax year as «Name (KT)» for the search; the canton follows from the municipality. */
let muniAllOf = null, muniAllCache = [];
function muniAll() {
  if (muniAllOf !== taxLoc) { muniAllOf = taxLoc; muniAllCache = (taxLoc?.cantons ?? []).flatMap(c => c.municipalities.map(m => ({ label: `${m[1]} (${c.id})`, name: m[1], canton: c.id, bfs: m[0] }))).sort((a, b) => a.label.localeCompare(b.label, "de-CH")); }
  return muniAllCache;
}
/** #272/AK1: municipality of an input: «Name (KT)» or a name that is unique (upper and lower case ignored); with a
 *  name in several cantons the canton of the row (`canton`) decides. */
function findMuni(text, canton) {
  const t = text.trim().toLowerCase(), all = muniAll();
  const exact = all.find(m => m.label.toLowerCase() === t);
  if (exact) return exact;
  const named = all.filter(m => m.name.toLowerCase() === t);
  return named.length === 1 ? named[0] : named.find(m => m.canton === canton) ?? null;
}
/** #63/AK1, AK9: one row of the card «Wohnort»: municipality (#272/AK1: search over all municipalities, the canton
 *  follows from it), canton as text and church tax; multipliers and the rule of the canton for lump sums (#64/AK1,
 *  AK3, with ⚠ if only approximated). #151/AK1: the first row from the start, the others with «ab» (year or event)
 *  and ✕; the first row keeps the ids of #63. */
function locRow({ path, loc, from }, i) {
  const canton = taxLoc?.cantons.find(c => c.id === loc.canton), muni = canton?.municipalities.find(m => m[0] === loc.municipality);
  const earned = canton?.deductions.earned, id = k => i ? `${k}-${i}` : k;
  const when = from == null ? `<span class="computed">${tr("deductions.fromStart")}</span>` : Ref(`${path}.from`);
  return `<div class="locrow" data-loc-row="${i}"><div class="row"><span class="ab">${tr("common.from")}</span>${when}
      ${i ? `<button class="btn ghost small" title="${tr("series.removeRow")}" data-action="loc-del" data-i="${i - 1}">✕</button>` : ""}</div>
    <div class="grid3">
    <label class="field"><span>${tr("taxes.municipality")}</span><div class="inp" data-path="${path}.municipality"><input class="loc-muni" id="${id("tax-muni")}" data-loc="${path}" list="tax-munis" autocomplete="off" placeholder="${tr("taxes.municipalitySearch")}" value="${esc(muni ? `${muni[1]} (${canton.id})` : "")}"></div></label>
    <label class="field"><span>${tr("taxes.canton")}</span><span class="computed" style="padding:8px 0" id="${id("tax-canton")}" data-path="${path}.canton">${canton ? esc(canton.name) : "–"}</span></label>
    <label class="field"><span>${tr("taxes.church")}</span>${Sel(`${path}.church`, CHURCH, { rerender: 1 })}</label></div>
    ${muni ? `<p class="hint" id="${id("tax-multipliers")}">${multiplierText(tr("taxes.income"), muni[2], loc.church)}; ${multiplierText(tr("taxes.wealth"), muni[3], loc.church)}.</p>` : ""}
    ${earned?.approx ? `<p class="hint warn" id="${id("tax-earned-approx")}">⚠ ${tr("taxes.earnedApprox", { max: fmtInt(earned.max) })}</p>` : ""}
    ${canton ? `<p class="hint" id="${id("tax-capital-rule")}">${tr("taxes.capitalRule", { rule: tr(`capitalRule.${canton.id}`) })}</p>` : ""}
    ${canton?.capitalApprox ? `<p class="hint warn" id="${id("tax-capital-approx")}">⚠ ${tr("taxes.capitalApprox", { note: tr(`capitalApprox.${canton.id}`) })}</p>` : ""}</div>`;
}
/** #272/AK3: the two estimated taxes (first full calendar year, first full calendar year after the retirement) in
 *  prices of the start, rounded to 100 by the engine; without a preview a dash. */
function taxFigures() {
  if (!taxEstimates.length) return `<span data-tax-figures-none>${tr("taxes.figuresNone")}</span>`;
  return taxEstimates.map(e => `<span class="taxfig" data-tax-figure="${e.kind}"><span class="lbl">${tr(`taxes.figure.${e.kind}`, { year: e.year })}</span><b>${fmtInt(e.amount)} ${UNIT.chf()}</b></span>`).join("");
}
/** #272/AK6: before and after of the switch to the place of residence (not saved): estimate of the first year with the
 *  own tables; the hint after the choice of the municipality. */
let taxSwitch = null;
function taxSwitchHint() {
  const after = taxEstimates.find(e => e.kind === "first");
  if (!taxSwitch || !isLocation(state) || state.taxes.location?.municipality == null || !after) return "";
  return `<p class="hint" id="tax-switch-hint">${tr("taxes.switchResult", { year: after.year, before: fmtInt(taxSwitch.amount), after: fmtInt(after.amount) })}</p>`;
}
const taxFiguresBlock = () => `<div class="taxfigs" id="tax-figures">${C("taxFigures")}</div>
    <p class="hint">${tr("taxes.figuresNote")}</p>${C("taxSwitchHint")}`;
/** #63/AK1, AK9: Card «Wohnort» with the tax year, the source and the note that the tax is an estimate with the
 *  standard deductions. #151/AK1: one row per place of residence, «Wohnort hinzufügen» adds a row. #272/AK3: the two
 *  estimated taxes at the top. */
function taxLocationCard() {
  return `<div class="card" id="tax-location"><h3>${tr("taxes.location")}</h3>
    ${taxFiguresBlock()}
    <div style="margin:8px 0 12px"><button class="btn small" data-action="taxbill-open" id="taxbill-open">${tr("taxBill.button")}</button></div>
    <div class="rows">${locRows().map(locRow).join("")}
    <datalist id="tax-munis">${muniAll().map(m => `<option value="${esc(m.label)}"></option>`).join("")}</datalist>
    <div><button class="btn small" data-action="loc-add">${tr("taxes.locationAdd")}</button></div></div>
    <p class="hint" id="tax-location-rule">${tr("taxes.locationRule")}</p>
    <p class="hint lumpcheck">${tr("taxes.lumpsumTax")}: ${tr("taxes.checkBeforeDecision")}</p>
    <p class="hint" id="tax-source">${tr("taxes.locationSource", { year: taxLoc?.year ?? "–", source: esc(taxLoc?.source ?? "") })}</p></div>`;
}
/** #274: dialog «Aus der Steuerrechnung übernehmen»: the numbers of the last tax bill (not saved, kept while the app
 *  is open), the result of the engine (`tax_bill`) and whether the proposal for the taxable wealth is taken over. */
const taxBill = { year: "", totalIncome: "", taxableIncome: "", taxableWealth: "", totalTax: "", out: null, wealth: false, seq: 0 };
const TAXBILL_FIELDS = [["year", "taxBill.year", true], ["totalIncome", "taxBill.totalIncome"], ["taxableIncome", "taxBill.taxableIncome"],
  ["taxableWealth", "taxBill.taxableWealth"], ["totalTax", "taxBill.totalTax"]];
/** #274: the inputs as numbers (empty: missing, the engine reports it at the field) and the fields with an unreadable
 *  number. */
function taxBillInput() {
  const bill = {}, bad = new Map();
  for (const [k, , int] of TAXBILL_FIELDS) {
    const t = String(taxBill[k]).trim();
    if (t === "") continue;
    const v = parseAmount(t);
    if (v === null || int && !Number.isInteger(v)) bad.set(k, ambiguousThousands(t) ? ambiguousText() : tr("input.invalidNumber"));
    else bill[k] = v;
  }
  return { bill, bad };
}
/** #274: result part of the dialog: deductions until and from the retirement, taxable wealth with the proposal,
 *  check with the total of the taxes. */
function taxBillResult() {
  const out = taxBill.out, r = out?.result;
  if (out?.error) return `<p class="hint warn" id="taxbill-error">⚠ ${esc(msgText(out.error))}</p>`;
  if (!r) return "";
  const k = v => fmtInt(v);
  const rows = [`<p id="taxbill-deduction"><b>${tr("taxBill.deduction", { amount: k(r.deduction) })}</b>${r.pillar3a ? `<br><span class="sub">${tr("taxBill.pillar3a", { amount: k(r.pillar3a) })}</span>` : ""}</p>`];
  if (r.retiredBeforeStart) rows.push(`<p id="taxbill-until">${tr("taxBill.fromStart", { amount: k(r.own) })}</p><p class="hint" id="taxbill-retired">${tr("taxBill.retiredBeforeStart")}</p>`);
  else {
    rows.push(`<p id="taxbill-until">${tr("taxBill.untilRetirement", { amount: k(r.own) })}</p>`);
    rows.push(`<p id="taxbill-from">${r.retired != null ? tr("taxBill.fromRetirement", { retired: k(r.retired), year: r.retiredYear, rest: k(r.rest), total: k(r.retired + r.rest) }) : tr("taxBill.fromRetirementShort", { rest: k(r.rest) })}</p>`);
    if (r.working != null) rows.push(`<p class="hint" id="taxbill-rest">${tr("taxBill.rest", { working: k(r.working), year: r.workingYear })}</p>`);
  }
  const w = r.wealth, prop = w.proposal;
  const wealthText = tr("taxBill.wealthCompare", { year: w.year, plan: k(w.plan), allowance: k(w.allowance), bill: k(w.bill), difference: (w.difference > 0 ? "+" : "") + k(w.difference) });
  const proposal = prop ? `<label class="check" id="taxbill-wealth-proposal"><input type="checkbox" id="taxbill-wealth"${taxBill.wealth ? " checked" : ""}> ${prop.kind === "taxValueRate" ? tr("taxBill.wealthRate", { pc: fmtPct(prop.rate) }) : tr("taxBill.wealthDeduction", { amount: k(prop.amount) })}</label>`
    : `<p class="hint" id="taxbill-wealth-none">${tr(w.difference >= 1000 ? "taxBill.wealthHigher" : "taxBill.wealthSmall")}</p>`;
  rows.push(`<div id="taxbill-wealth-block" style="margin-top:12px"><b>${tr("taxBill.wealthTitle")}</b><p>${wealthText}</p>${proposal}</div>`);
  const c = r.check;
  if (c) rows.push(`<div id="taxbill-check" style="margin-top:12px"><b>${tr("taxBill.checkTitle")}</b><p>${tr("taxBill.check", { estimate: k(c.estimate), total: k(c.total), deviation: (c.deviation > 0 ? "+" : "") + (c.deviation * 100).toLocaleString("de-CH", { maximumFractionDigits: 1 }) })}</p>
    ${c.hint ? `<p class="hint warn" id="taxbill-check-hint">⚠ ${tr("taxBill.checkHint")}</p>` : ""}</div>`);
  return rows.join("");
}
/** #274: errors at the fields (unreadable number in the app, the others from the engine). */
function taxBillErrors() {
  const { bad } = taxBillInput(), errs = new Map(bad);
  for (const i of taxBill.out?.issues ?? []) if (i.field && !errs.has(i.field)) errs.set(i.field, msgText(i));
  for (const [k] of TAXBILL_FIELDS) {
    const el = $(`#taxbill-${k}`), e = $(`#taxbill-err-${k}`), text = errs.get(k);
    el?.classList.toggle("invalid", !!text);
    if (e) { e.textContent = text ?? ""; e.hidden = !text; }
  }
  return errs;
}
function taxBillRender() {
  const out = $("#taxbill-result"); if (out) out.innerHTML = taxBillResult();
  const errs = taxBillErrors(), ok = $("#taxbill-apply");
  if (ok) ok.disabled = !taxBill.out?.result || errs.size > 0;
}
/** #274: asks the engine for the result of the inputs; an older answer than the last request is dropped. */
async function taxBillCompute() {
  const { bill, bad } = taxBillInput(), seq = ++taxBill.seq;
  if (bad.size) { taxBill.out = null; taxBillRender(); return; }
  try {
    const out = await Engine.call("tax_bill", { plan: state, bill });
    if (seq === taxBill.seq) { taxBill.out = out; taxBillRender(); }
  } catch (e) { if (seq === taxBill.seq) { taxBill.out = { issues: [], error: { code: "raw_error", params: { detail: errText(e) }, text: errText(e) }, result: null }; taxBillRender(); } }
}
function taxBillOpen() {
  const d = $("#taxbill"), field = ([k, label]) => `<label class="field"><span>${tr(label)}</span><div class="inp"><input class="num" id="taxbill-${k}" data-taxbill="${k}" value="${esc(taxBill[k])}">${k === "year" ? "" : `<span class="unit">${UNIT.chf()}</span>`}</div><span class="ferr" id="taxbill-err-${k}" role="alert" hidden></span></label>`;
  $("#taxbillForm").innerHTML = `<b id="taxbillTitle">${tr("taxBill.title")}</b><p class="hint">${tr("taxBill.intro")}</p>
    <div class="grid2">${TAXBILL_FIELDS.map(field).join("")}</div>
    <div id="taxbill-result" aria-live="polite"></div>
    <p class="hint">${tr("taxBill.replaceNote")}</p>
    <div class="askbtns"><button class="btn primary" id="taxbill-apply" value="ok" disabled>${tr("taxBill.apply")}</button><button class="btn" value="cancel">${tr("job.cancel")}</button></div>`;
  d.returnValue = ""; taxBill.wealth = false;
  d.onclose = () => { if (d.returnValue === "ok") taxBillApply(); };
  d.showModal();
  taxBill.out = null; taxBillRender(); taxBillCompute();
}
/** #274: takes over the rows of the deductions from income and, if ticked, the proposal for the taxable wealth. */
function taxBillApply() {
  const r = taxBill.out?.result;
  if (!r) return;
  state.taxes.deductions = structuredClone(r.deductions);
  const p = r.wealth.proposal;
  if (p && taxBill.wealth) {
    if (p.kind === "taxValueRate") for (const i of p.properties) state.realestate.properties[i].taxValue = { kind: "rate", value: p.rate };
    else {
      const rows = state.taxes.wealthDeductions ??= [];
      if (rows[0]?.from === "start") rows[0].amount = p.amount; else rows.unshift({ from: "start", amount: p.amount });
    }
  }
  markDirty(); renderSection(); toast(tr("taxBill.applied", { year: r.year }));
}
document.addEventListener("input", e => {
  const k = e.target?.dataset?.taxbill;
  if (!k) return;
  taxBill[k] = e.target.value;
  clearTimeout(taxBillCompute.t); taxBillCompute.t = setTimeout(taxBillCompute, 250);
});
document.addEventListener("change", e => { if (e.target?.id === "taxbill-wealth") taxBill.wealth = e.target.checked; });
document.addEventListener("change", e => {
  const el = e.target, path = el?.dataset?.loc;
  if (!path || !el.classList.contains("loc-muni")) return;
  if (path === "taxes.location") state.taxes.location ??= { canton: "", church: "none" };
  const loc = getP(path), m = findMuni(el.value, loc?.canton);
  // #272/AK1: the canton follows from the municipality; without a match the municipality is missing
  if (m) { loc.canton = m.canton; loc.municipality = m.bfs; } else delete loc.municipality;
  markDirty(); renderSection();
});
/** #131: rows of the deductions from income; without `taxes.deductions` as before: in the mode «Wohnort» the proposal
 *  of the canton throughout, otherwise none (shown, but only stored after «Abzüge anpassen»). */
const dedImplicit = () => isLocation(state) ? [{ from: "start", kind: "canton" }] : [];
const dedRows = () => state.taxes.deductions ?? dedImplicit();
const DED_PARTS = ["social", "insurance", "savingsInterest", "professional", "secondEarner"];
/** #131/AK5, #241/AK3, AK4: proposal of the canton for the first year of row `i` (from the preview): year and the
 *  amounts per part of the deduction (with «zusätzlich» of the row), each for canton and federal tax; null without
 *  preview. */
function dedFirstYear(i) {
  const r = dedRows()[+i];
  if (!r || !lastSim?.taxBasis) return null;
  const from = Math.max(refMonth(r.from), startM());
  if (!Number.isFinite(from)) return null;
  const y = String(Math.floor(from / 12)), b = lastSim.taxBasis.get(y);
  if (!b || !b.partsCanton.length && !b.partsFederal.length) return null;
  const sum = (list, k) => list.filter(p => p[0] === k).reduce((a, p) => a + p[1], 0);
  const extra = (r.extra ?? 0) * (state.taxes.indexed ? b.priceIndex ?? 1 : 1);
  const parts = [...DED_PARTS.map(k => [tr(`deductions.part.${k}`), sum(b.partsCanton, k), sum(b.partsFederal, k)]), [tr("deductions.originExtra"), extra, extra]]
    .filter(p => p[1] !== 0 || p[2] !== 0);
  return { year: y, parts, canton: parts.reduce((a, p) => a + p[1], 0), federal: parts.reduce((a, p) => a + p[2], 0) };
}
/** #241/AK3: sum of the proposal for canton and federal tax in the head line of row `i`. */
function dedSum(i) {
  const d = dedFirstYear(i);
  return d ? tr("deductions.sum", { canton: fmtInt(d.canton), federal: fmtInt(d.federal) }) : tr("deductions.proposalNone");
}
/** #241/AK4: table of the parts of the deduction (rows) for canton and federal tax (columns) with the total. */
function dedParts(i) {
  const d = dedFirstYear(i);
  if (!d) return `<p class="computed">${tr("deductions.proposalNone")}</p>`;
  const row = (label, c, f, cls = "") => `<tr${cls}><td>${label}</td><td class="r">${fmtInt(c)}</td><td class="r">${fmtInt(f)}</td></tr>`;
  return `<table class="tbl dedparts"><tr><th>${tr("deductions.detailsYear", { year: d.year })}</th><th class="r">${tr("deductions.colCanton")}</th><th class="r">${tr("deductions.colFederal")}</th></tr>
    ${d.parts.map(p => row(...p)).join("")}${row(tr("deductions.total"), d.canton, d.federal, ' class="total"')}</table>`;
}
/** #230/AK2: list «Einmalige Abzüge» (tax year, description, nominal amount), like «Einmalige Beträge» in the flows.
 *  #231/AK1: the same list for the deductions from wealth (`key` oneOffWealthDeductions). */
function oneOffDeductions(key = "oneOffDeductions", hint = tr("deductions.oneOffHint")) {
  const P = `taxes.${key}`, list = state.taxes[key] ?? [], wealth = key !== "oneOffDeductions";
  const rows = list.map((x, i) => `<tr data-oneoff-row="${i}"><td style="width:110px">${F(`${P}.${i}.year`, "", "num", { bare: true, int: true })}</td><td>${F(`${P}.${i}.label`, "", "text", { bare: true })}</td><td style="width:200px">${F(`${P}.${i}.amount`, "", "chf", { bare: true })}</td>
      <td><button class="btn ghost small" title="${tr("series.removeRow")}" data-action="list-del" data-path="${P}" data-i="${i}">✕</button></td></tr>`).join("");
  // #241/AK7: in the card «Abzüge vom Einkommen» under a subtitle of their own
  const head = wealth ? `<b>${tr("deductions.oneOff")}</b> <span class="sub">${hint}</span>` : `<h4 class="dedsubtitle">${tr("deductions.oneOff")}</h4><p class="hint">${hint}</p>`;
  return `<div id="${wealth ? "wded-oneoff" : "ded-oneoff"}" style="margin-top:12px">${head}
    ${list.length ? `<table class="tbl"><tr><th>${tr("deductions.taxYear")}</th><th>${tr("flows.description")}</th><th>${tr("flows.amount")}</th><th></th></tr>${rows}</table>` : ""}
    <div style="margin-top:8px"><button class="btn small" data-action="ded-oneoff-add" data-key="${key}">${tr("deductions.addOneOff")}</button></div></div>`;
}
/** #231/AK1–AK3: card «Abzüge vom Vermögen»: time-dependent rows «zusätzlich … pro Jahr» (layout .dedrow as the
 *  deductions from income, #228) and the list of one-off deductions; in the mode «Wohnort» in addition to the social
 *  deductions of the canton (hint), with own tables the whole deduction. */
/** #249/AK1, AK3: open details and chosen tax year of the composition (only for the display, not saved). */
const wtax = { open: false, year: null };
document.addEventListener("toggle", e => { if (e.target?.dataset?.wtaxDetails != null) wtax.open = e.target.open; }, true);
document.addEventListener("change", e => {
  if (e.target?.id !== "wtax-year") return;
  wtax.year = +e.target.value;
  const out = $('[data-computed="wtaxTable"]'); if (out) out.innerHTML = wtaxTable();
});
/** #249: composition of the taxable wealth of tax year `y` from the preview (first tax year without `y`). */
const wtaxYearOf = y => (lastSim?.wealthTax || []).find(w => y == null || w.year === y) ?? null;
/** #249/AK7: estimated taxable wealth of the first tax year with the share of the properties, warnings for
 *  properties without a tax value (link to the field) and, in the mode «Wohnort», the note on cantonal allowances. */
function wtaxHint() {
  const w = wtaxYearOf(null);
  if (!w) return tr("wealthTax.none");
  const k = v => fmtInt(Math.round(v / 1000) * 1000);
  const props = w.properties.reduce((a, p) => a + p.taxValue - p.mortgage, 0);
  let html = tr("wealthTax.estimate", { year: w.year, amount: k(w.taxable) }) + (w.properties.length ? tr("wealthTax.ofWhichProperties", { amount: k(props) }) : "");
  if (isLocation(state)) html += ` ${tr("wealthTax.locationNote")}`;
  for (const p of w.properties.filter(p => !p.hasTaxValue))
    html += `<br><span class="warn" data-wtax-missing="${p.index}">⚠ ${tr("wealthTax.noTaxValue", { name: esc(p.name), amount: fmtInt(p.worth) })} <a href="#" data-action="wtax-edit" data-i="${p.index}">${tr("wealthTax.noTaxValueLink")}</a></span>`;
  return html;
}
/** #249/AK1, AK2: rows of the composition (wealth without properties, tax value and mortgage per property, deductions,
 *  taxable wealth) for the chosen tax year; the rows add up to the taxable wealth of the year table. */
function wtaxTable() {
  const w = wtaxYearOf(wtax.year) ?? wtaxYearOf(null);
  if (!w) return `<p class="computed">${tr("wealthTax.none")}</p>`;
  const row = (label, v, cls = "") => `<tr${cls}><td>${label}</td><td class="r">${fmtInt(v)}</td></tr>`;
  const rows = [row(tr("wealthTax.liquid"), w.liquid)];
  for (const p of w.properties) {
    // #273/AK5: the origin «Vorgabe» of the default tax value
    rows.push(p.defaultTaxValue ? row(tr("wealthTax.taxValueDefault", { name: esc(p.name), pc: taxValueDefaultPc() }), p.taxValue, ` data-wtax-default="${p.index}"`)
      : p.hasTaxValue ? row(tr("wealthTax.taxValue", { name: esc(p.name) }), p.taxValue)
      : row(`${tr("wealthTax.marketValue", { name: esc(p.name) })} <span class="warn">⚠</span>`, p.taxValue, ` data-wtax-market="${p.index}"`));
    rows.push(row(tr("wealthTax.mortgage", { name: esc(p.name) }), p.mortgage));
  }
  if (w.ongoing) rows.push(row(tr("wealthTax.ongoing"), w.ongoing));
  if (w.oneOff) rows.push(row(tr("wealthTax.oneOff"), w.oneOff));
  rows.push(row(tr("wealthTax.taxable"), w.taxable, ' class="total" data-wtax-taxable'));
  return `<table class="tbl dedparts" data-wtax-year="${w.year}"><tr><th>${tr("wealthTax.year", { year: w.year })}</th><th class="r" data-no-tr>${UNIT.chf()}</th></tr>${rows.join("")}</table>
    ${isLocation(state) ? `<p class="hint">${tr("wealthTax.socialNote")}</p>` : ""}`;
}
/** #231, #249: card «Abzüge vom Vermögen»: estimated taxable wealth (AK7), composition per tax year with the tax value
 *  of the properties (AK1, AK3), the rows of the deductions and the one-off deductions. */
/** #149/AK1: taxable yield of the wealth per asset class (stocks, bonds; cash has none). */
function investmentYieldCard() {
  const info = infoPop(`<p>${tr("investmentYield.hint")}</p><p>${tr("investmentYield.defaults")}</p>`);
  return `<div class="card" id="tax-investment-yield"><h3>${tr("investmentYield.title")}${info}</h3>
    <div class="grid3">${F("taxes.investmentYield.stocks", tr("investmentYield.stocks"), "pct", { unit: tr("investmentYield.unit") })}${F("taxes.investmentYield.bonds", tr("investmentYield.bonds"), "pct", { unit: tr("investmentYield.unit") })}</div>
    <p class="hint">${tr("investmentYield.note")}</p></div>`;
}
function wealthDeductionsCard() {
  const P = "taxes.wealthDeductions", rows = state.taxes.wealthDeductions ?? [];
  const body = rows.map((r, i) => `<div class="dedrow" data-wded-row="${i}"><span class="ab">${tr("common.from")}</span><div class="dedwhen">${Ref(`${P}.${i}.from`)}</div>
    <div class="dedval"><span class="dedextra">${tr("deductions.extra")}</span>${F(`${P}.${i}.amount`, "", "chf", { bare: true, unit: tr("deductions.perYear") })}</div>
    <button class="btn ghost small" title="${tr("series.removeRow")}" data-action="row-del" data-path="${P}" data-i="${i}">✕</button></div>`).join("");
  const y0 = Math.floor(startM() / 12), y1 = Math.floor(endM() / 12);
  const years = Number.isFinite(y0) && Number.isFinite(y1) ? Array.from({ length: Math.max(0, y1 - y0 + 1) }, (_, k) => y0 + k) : [];
  const yearSel = years.length > 1 ? `<label class="field" style="max-width:200px"><span>${tr("wealthTax.chooseYear")}</span><select class="plain" id="wtax-year">${years.map(y => `<option value="${y}"${y === wtax.year ? " selected" : ""}>${y}</option>`).join("")}</select></label>` : "";
  // #249/AK3: the same field «Steuerwert» as in the section «Immobilien» for every owned or planned property
  const props = state.realestate.properties.map((p, i) => [p, i]).filter(([p]) => p.status !== "rent");
  const taxValues = props.length ? `<div data-wtax-fields style="margin-top:12px"><b>${tr("wealthTax.taxValuesTitle")}</b>${props.map(([p, i]) => `<div data-wtax-field="${i}">${taxValueRow(p, i, `realestate.properties.${i}`, { label: esc(p.name || tr("realestate.taxValue")), userText: !!p.name, hint: false })}</div>`).join("")}
    <p class="hint">${tr("realestate.taxValueHint")}</p></div>` : "";
  const details = `<details class="deddetails" data-wtax-details style="margin-bottom:12px"${wtax.open ? " open" : ""}><summary>${tr("deductions.details")}</summary><div style="margin-top:8px">${yearSel}</div>${C("wtaxTable")}${taxValues}</details>`;
  return `<div class="card" id="tax-wealth-deductions"><h3>${tr("wealthDeductions.title")}</h3>
    <p class="hint" id="wtax-hint">${C("wtaxHint")}</p>
    ${details}
    <div class="rows">${body || `<p class="computed">${tr("wealthDeductions.none")}</p>`}
    <div><button class="btn small" data-action="wded-add">${tr("series.addRow")}</button></div></div>
    ${oneOffDeductions("oneOffWealthDeductions", tr("wealthDeductions.oneOffHint"))}
    <p class="hint" id="wded-hint">${isLocation(state) ? tr("wealthDeductions.hintLocation") : tr("wealthDeductions.hintOwn")}</p></div>`;
}
/** #241/AK5: rows whose field «zusätzlich» is shown without a stored amount (after «+ Zusätzlicher Abzug»), and
 *  #241/AK4: rows with open details; both only for the display, not saved. */
const dedExtraShown = new Set(), dedDetailsOpen = new Set();
document.addEventListener("toggle", e => {
  const i = e.target?.dataset?.dedDetails;
  if (i == null) return;
  if (e.target.open) dedDetailsOpen.add(i); else dedDetailsOpen.delete(i);
}, true);
/** #131/AK3–AK10, #241: card «Abzüge vom Einkommen» with the pillar 3a deduction: per row a head line (from, kind,
 *  sum or own value), the additional amount and the details; explanations in ⓘ, one-off deductions below. */
function deductionsCard() {
  const loc = isLocation(state), stored = !!state.taxes.deductions, rows = dedRows(), P = "taxes.deductions";
  // #151: ⚠ if the flat deduction is approximated at one of the places of residence
  const earned = loc ? locRows().map(r => taxLoc?.cantons.find(c => c.id === r.loc.canton)?.deductions.earned).find(e => e?.approx) : null;
  const kinds = [["canton", tr("deductions.canton")], ["own", tr("deductions.own")]];
  const body = rows.map((r, i) => {
    const R = `${P}.${i}`, canton = loc && r.kind === "canton"; // AK11: with own tables no proposal
    // #228: own grid (.dedrow) so that the value gets the remaining width; #241/AK3: from, kind and value in one line
    const kind = !loc ? "" : stored ? Sel(`${R}.kind`, kinds, { rerender: 1 }) : `<span class="computed">${tr("deductions.canton")}</span>`;
    const value = canton
      ? `<span class="computed" data-ded-proposal="${i}">${C("dedSum", i)}</span>`
      : `${F(`${R}.amount`, "", "chf", { bare: true, unit: tr("deductions.perYear") })}${F(`${R}.rate`, "", "pct", { bare: true, unit: tr("deductions.ofIncome") })}`;
    // #230/AK1, #241/AK5: additional amount per year to the proposal of the canton (only stored rows can be edited),
    // the field only with an amount ≠ 0 or after «+ Zusätzlicher Abzug»
    const extra = !canton || !stored ? ""
      : (r.extra ?? 0) !== 0 || dedExtraShown.has(i)
        ? `<div class="dedextrarow"><span class="dedextra">${tr("deductions.extraLabel")}</span>${F(`${R}.extra`, "", "chf", { bare: true, unit: tr("deductions.perYear") })}</div>`
        : `<button class="btn ghost small" data-action="ded-extra-show" data-i="${i}">${tr("deductions.addExtra")}</button>`;
    // #241/AK4: parts of the proposal per row, closed by default
    const details = canton ? `<details class="deddetails" data-ded-details="${i}"${dedDetailsOpen.has(String(i)) ? " open" : ""}><summary>${tr("deductions.details")}</summary>${C("dedParts", i)}</details>` : "";
    // #274/AK7: origin of a row taken over from the tax bill
    const origin = r.taxBill ? `<span class="sub" data-ded-taxbill="${i}" title="${esc(tr("taxBill.origin", { year: r.taxBill }))}">${tr("taxBill.origin", { year: r.taxBill })}</span>` : "";
    return `<div class="dedblock" data-ded-row="${i}"><div class="dedrow inc${loc ? canton ? "" : " ownval" : " nokind"}"><span class="ab">${tr("common.from")}</span><div class="dedwhen">${stored ? Ref(`${R}.from`) : `<span class="computed">${tr("deductions.fromStart")}</span>`}</div>
      <div class="dedkind">${kind}${origin}</div><div class="dedval">${value}</div>
      ${stored && rows.length > 1 ? `<button class="btn ghost small" title="${tr("series.removeRow")}" data-action="row-del" data-path="${P}" data-i="${i}">✕</button>` : "<span></span>"}</div>
      ${extra || details ? `<div class="dedsub">${extra}${details}</div>` : ""}
      ${canton && earned?.approx ? `<p class="hint warn" data-ded-approx>⚠ ${tr("taxes.earnedApprox", { max: fmtInt(earned.max) })}</p>` : ""}</div>`;
  }).join("");
  const own = rows.map((r, i) => [r, i]).filter(([r]) => !loc || r.kind !== "canton");
  // #251: with «Säule 3a abziehen» the 3a contributions of the assessment are taken off (else they count twice)
  const calc = stored && own.length ? `<div class="dedcalc" id="ded-calc"><b>${tr("deductions.calcTitle")}</b><div class="grid2">
      <label class="field"><span>${tr("deductions.calcIncome")}</span><div class="inp"><input class="num" id="ded-calc-income" value="${esc(dedCalc.income)}"><span class="unit">${UNIT.chf()}</span></div></label>
      <label class="field"><span>${tr("deductions.calcTaxable")}</span><div class="inp"><input class="num" id="ded-calc-taxable" value="${esc(dedCalc.taxable)}"><span class="unit">${UNIT.chf()}</span></div></label>
      <label class="field"><span>${tr("deductions.calcPillar3a")}</span><div class="inp"><input class="num" id="ded-calc-p3a" value="${esc(dedCalc.p3a)}"><span class="unit">${UNIT.chf()}</span></div></label>
      <label class="field"><span>${tr("deductions.calcResult")}</span><span class="computed" id="ded-calc-result">${dedCalcText()}</span></label></div>
      <p class="hint warn" id="ded-calc-p3a-hint"${dedCalcP3aMissing() ? "" : " hidden"}>⚠ ${tr("deductions.calcPillar3aHint")}</p>
      <div class="row">${own.length > 1 ? `<select class="plain" id="ded-calc-row">${own.map(([r, i]) => `<option value="${i}">${tr("deductions.rowN", { n: i + 1 })}</option>`).join("")}</select>` : ""}
      <button class="btn small" data-action="ded-calc-apply" data-i="${own[0][1]}">${tr("deductions.calcApply")}</button></div></div>` : "";
  // #241/AK6: the explanations in ⓘ instead of paragraphs
  const info = infoPop(`<p id="ded-hint">${loc ? tr("deductions.hintLocation") : tr("deductions.hintOwn")}</p>
    ${own.length || !loc ? `<p id="ded-own-hint">${tr("deductions.ownHint")}</p>` : ""}`);
  const p3aInfo = infoPop(`<p id="ded-p3a-hint">${tr("deductions.pillar3aMax", { max: fmtInt(limits().pillar3aMax), maxLarge: fmtInt(limits().pillar3aMaxLarge) })}</p><p>${tr("deductions.pillar3aHint")}</p>`);
  return `<div class="card" id="tax-deductions"><h3>${tr("deductions.title")}${info}</h3>
    <div class="rows">${body || `<p class="computed">${tr("deductions.none")}</p>`}
    <div>${stored ? `<button class="btn small" data-action="ded-add">${tr("series.addRow")}</button>` : `<button class="btn small" data-action="ded-edit">${tr("deductions.edit")}</button>`}</div></div>
    ${calc}
    <div class="dedp3a" style="margin-top:10px">${Chk("taxes.deductPillar3a", tr("deductions.pillar3a"), { rerender: true })}${p3aInfo}</div>
    ${oneOffDeductions()}</div>`;
}
/** #131/AK7: input help «eigener Wert» from income and taxable income of the last assessment (not saved). */
const dedCalc = { income: "", taxable: "", p3a: "" };
const numIn = t => { const x = parseNum(String(t ?? "")); return Number.isFinite(x) ? x : NaN; };
/** #251: the 3a contributions of the assessment, taken off only with «Säule 3a abziehen» (empty = 0). */
const dedCalcP3a = () => state.taxes.deductPillar3a ? (numIn(dedCalc.p3a) || 0) : 0;
const dedCalcP3aMissing = () => !!state.taxes.deductPillar3a && !Number.isFinite(numIn(dedCalc.p3a));
const dedCalcValue = () => { const a = numIn(dedCalc.income), b = numIn(dedCalc.taxable); return Number.isFinite(a) && Number.isFinite(b) ? Math.max(0, a - b - dedCalcP3a()) : NaN; };
const dedCalcText = () => { const v = dedCalcValue(); return Number.isFinite(v) ? fmtInt(v) : "–"; };
const DED_CALC = { "ded-calc-income": "income", "ded-calc-taxable": "taxable", "ded-calc-p3a": "p3a" };
document.addEventListener("input", e => {
  const key = DED_CALC[e.target?.id];
  if (!key) return;
  dedCalc[key] = e.target.value;
  const out = $("#ded-calc-result"); if (out) out.textContent = dedCalcText();
  const hint = $("#ded-calc-p3a-hint"); if (hint) hint.hidden = !dedCalcP3aMissing();
});
/** #272/AK2: the assumptions in words: deductions, tariffs with inflation, pillar 3a deduction, tax value of the
 *  properties and taxable yield of the wealth. */
function taxAssumptions() {
  const t = state.taxes, rows = dedRows(), loc = isLocation(state);
  const canton = loc && rows.length && rows.every(r => r.kind === "canton");
  const extra = rows.some(r => (r.extra ?? 0) !== 0) || !!t.oneOffDeductions?.length;
  const ded = canton ? tr(rows.some(r => r.from === "retire") ? "taxes.assume.dedCantonRetire" : "taxes.assume.dedCanton") + (extra ? tr("taxes.assume.dedExtra") : "")
    : rows.some(r => r.kind === "canton" || r.amount || r.rate) || extra ? tr("taxes.assume.dedOwn") : tr("taxes.assume.dedNone");
  const y = t.investmentYield ?? {}, bill = rows.find(r => r.taxBill)?.taxBill;
  return [bill ? tr("taxBill.assume", { year: bill }) : ded, tr(t.indexed ? "taxes.assume.indexed" : "taxes.assume.notIndexed"), tr(t.deductPillar3a ? "taxes.assume.p3aYes" : "taxes.assume.p3aNo"),
    state.realestate.defaultTaxValue ? tr("taxes.assume.taxValueDefault", { pc: taxValueDefaultPc() }) : tr("taxes.assume.taxValueMarket"),
    tr("taxes.assume.yield", { stocks: fmtPct(y.stocks ?? 0), bonds: fmtPct(y.bonds ?? 0) })].join(" · ");
}
/** #272/AK2: open state of the settings below «Annahmen» and of «Experte» (only for the display, not saved); `null`:
 *  closed in the mode «Wohnort», open with own tables. */
const taxOpen = { settings: null, expert: null };
document.addEventListener("toggle", e => { const k = e.target?.dataset?.taxOpen; if (k) taxOpen[k] = e.target.open; }, true);
/** Errors at a field inside the settings open them, so that the message is visible. */
const taxSettingsError = () => [...lastIssues.filter(i => i.severity === "error").map(i => i.field), ...invalidFields.keys()]
  .some(f => /^taxes\.(?!location|relocations)|^realestate\.properties\.\d+\.taxValue/.test(f ?? ""));
R.taxes = () => {
  const own = !isLocation(state);
  const note = own ? tr("taxes.estimateNote") : tr("taxes.estimateNoteLocation");
  // #63, #64: in the mode «Wohnort» the tables for income, wealth and lump sums are hidden (they stay in the file).
  // #65: the property gains tax follows the canton of each property; own rules and their multiplier only with the
  // check box.
  const tables = TaxTable("income", tr("taxes.incomeTax"), tr("taxes.income")) + TaxTable("wealth", tr("taxes.wealthTax"), tr("taxes.wealth"));
  const coupleTables = state.taxes.couple ? TaxTable("couple.income", tr("taxes.incomeTaxMarried"), tr("taxes.income")) + TaxTable("couple.wealth", tr("taxes.wealthTaxMarried"), tr("taxes.wealth")) + TaxTable("couple.capital", tr("taxes.lumpsumTaxMarried"), tr("taxes.lumpsum")) : "";
  // #272/AK5, AK6: own tables only in plans that use them, under «Experte»; the card offers the switch
  const ownCard = `<div class="card" id="tax-own"><h3>${tr("taxes.modeOwn")} <span class="sub">${tr("taxes.expert")}</span></h3>
    ${taxFiguresBlock()}
    <p class="hint">${tr("taxes.ownHint")}</p>
    <div><button class="btn" data-action="tax-to-location" id="tax-to-location">${tr("taxes.switchToLocation")}</button></div></div>`;
  const open = k => (taxOpen[k] ?? own) || (k === "settings" && taxSettingsError());
  const settings = `<details class="taxsettings" id="tax-settings" data-tax-open="settings"${open("settings") ? " open" : ""}>
    <summary class="card taxassume" id="tax-assumptions"><b>${tr("taxes.assumptions")}</b> <span data-tax-assumptions>${C("taxAssumptions")}</span></summary>
    ${deductionsCard()}
    ${wealthDeductionsCard()}
    ${investmentYieldCard()}
    <div class="card">${own || gainOwn() ? `<div class="grid3" style="margin-bottom:10px">${F("taxes.multiplier", own ? tr("taxes.multiplier") : tr("taxes.multiplierLocation"), "num", { unit: "×" })}</div>` : ""}
    ${own ? "" : `<p class="hint">${gainOwn() ? tr("taxes.locationOwnTablesHint") : tr("taxes.locationCantonGainHint")}</p>`}
    <div>${Chk("taxes.indexed", tr("taxes.indexed"))}</div>
    <p class="hint">${own ? tr("taxes.indexedHint") : tr("taxes.indexedHintLocation")}</p></div>
    ${own ? "" : locGainCard()}</details>`;
  const expert = own ? `<details class="taxsettings" id="tax-expert" data-tax-open="expert"${open("expert") ? " open" : ""}><summary class="card taxassume"><b>${tr("taxes.expert")}:</b> ${tr("taxes.modeOwn")}</summary>
    <div class="card">${tables}${TaxTable("capital", tr("taxes.lumpsumTax"), tr("taxes.lumpsum"))}
    <p class="hint lumpcheck">${tr("taxes.lumpsumTax")}: ${tr("taxes.checkBeforeDecision")}</p></div>
    ${gainCard()}
    ${state.household.married ? `<div class="card"><h3>${tr("taxes.married")}</h3>
    <label class="check"><input type="checkbox" data-action="couple-tax" ${state.taxes.couple ? "checked" : ""}> ${tr("taxes.marriedTables")}</label>
    <p class="hint">${tr("taxes.marriedHint")}</p>
    ${coupleTables}</div>` : ""}</details>` : "";
  return `<h2>${tr("section.taxes")}</h2><p class="lead">${own ? tr("taxes.intro") : tr("taxes.introLocation")}</p>
  ${own ? ownCard : taxLocationCard()}
  ${settings}
  ${expert}
  <div class="card taxnote" id="taxEstimateNote"><p class="hint">${note} ${helpLink("tax-accuracy", tr("taxes.accuracyLink"))}</p></div>`;
};

let PSTAT;
onLanguage(() => PSTAT = [["owned", tr("property.owned")], ["planned", tr("property.planned")], ["rent", tr("property.rent")]]);
/** #50: Period that only starts from the AHV payout (then only direct amortization). */
function periodAfterAhv(p, t, k) {
  const r = trancheSchedule(p, t).find(x => x.i === k);
  return !!r && r.start >= msMonth(ahvIdOf(t.pillar3a));
}
function MortgageUI(p, i) {
  const M = `realestate.properties.${i}.mortgage`, mg = p.mortgage, multi = mg.tranches.length > 1, owned = p.status === "owned";
  const anyIndirect = mg.tranches.some(t => t.periods.some(d => d.amortMode === "indirect" && d.amort > 0));
  const maxLoan = !owned && mg.amountMode === "maxLoan";
  const mode = owned ? "" : `<div class="grid2" style="margin-bottom:10px"><label class="field"><span>${tr("mortgage.atPurchase")}</span>${Sel(M + ".amountMode", [["fixed", tr("mortgage.asEntered")], ["maxLoan", tr("mortgage.eightyPct")]], { rerender: 1 })}</label></div>
    ${maxLoan && multi ? `<p class="hint" style="margin-top:-4px">${tr("mortgage.splitHint")}</p>` : ""}`;
  const tranches = mg.tranches.map((t, ti) => {
    const T = `${M}.tranches.${ti}`;
    const rows = t.periods.map((d, k) => {
      const D = `${T}.periods.${k}`;
      return `<tr><td style="white-space:nowrap">${k ? tr("mortgage.followUp", { k }) : owned ? tr("mortgage.current") : tr("mortgage.first")}</td>
        <td>${Sel(D + ".type", [["fixed", tr("mortgage.fixed")], ["saron", "SARON"]], { rerender: 1 })}</td>
        <td>${F(D + ".term", "", "years", { bare: true, unit: tr("unit.yearsShort") })}</td>
        <td>${d.type === "saron" ? F(D + ".margin", "", "pct", { bare: true, unit: tr("mortgage.margin") }) : F(D + ".rate", "", "pct", { bare: true })}</td>
        <td><div class="refwrap">${F(D + ".amort", "", "chf", { bare: true })}${periodAfterAhv(p, t, k) ? `<span class="computed" style="padding:8px 4px" title="${tr("mortgage.directOnly")}">${tr("mortgage.directFromAhv")}</span>` : Sel(D + ".amortMode", [["direct", tr("mortgage.direct")], ["indirect", tr("mortgage.indirect")]], { rerender: 1 })}</div></td>
        <td>${C("mrow", `${i}:${ti}:${k}`)}</td>
        <td>${k ? `<button class="btn ghost small" data-action="list-del" data-path="${T}.periods" data-i="${k}" title="${tr("mortgage.removePeriod")}">✕</button>` : ""}</td></tr>`;
    }).join("");
    return `<div class="tranche">
      ${multi ? `<div class="thead">${F(T + ".name", "", "text", { bare: true })}<button class="btn ghost small" data-action="list-del" data-path="${M}.tranches" data-i="${ti}" title="${tr("mortgage.removeTranche")}">✕</button></div>` : ""}
      <div class="grid3">${maxLoan ? (multi ? F(T + ".amount", tr("mortgage.share"), "num") : `<label class="field"><span>${tr("mortgage.amountAtPurchase")}</span><span class="computed" style="padding:8px 0">${tr("mortgage.computedAtPurchase")}</span></label>`) : F(T + ".amount", owned ? tr("mortgage.amountToday") : tr("mortgage.amountAtPurchase"), "chf")}
        ${owned ? F(T + ".start", tr("mortgage.currentStart"), "month") : `<label class="field"><span>${tr("mortgage.start")}</span><span class="computed" style="padding:8px 0">${tr("mortgage.atPurchaseDate", { esc: esc(msName(p.buy)) })}</span></label>`}
        <label class="field" style="grid-column:span 2"><span>${tr("mortgage.afterLast")}</span>${Sel(T + ".after", [["renew", tr("mortgage.extend")], ["repay", tr("mortgage.repay")]])}</label>
        ${state.household.persons.length > 1 && t.periods.some(d => d.amortMode === "indirect") ? `<label class="field"><span>${tr("mortgage.pillar3a")}</span>${Sel(T + ".pillar3a", state.household.persons.map(q => [q.id, q.name || q.id]))}</label>` : ""}</div>
      <div style="overflow-x:auto"><table class="tbl mtbl"><colgroup><col style="width:68px"><col style="width:96px"><col style="width:82px"><col style="width:130px"><col style="width:212px"><col><col style="width:36px"></colgroup><tr><th>${tr("mortgage.period")}</th><th>${tr("mortgage.type")}</th><th>${tr("mortgage.term")}</th><th>${tr("mortgage.rate")}</th><th>${tr("mortgage.amortQuarter")}</th><th>${tr("mortgage.periodDebt")}</th><th></th></tr>${rows}</table></div>
      <div style="margin-top:8px"><button class="btn small" data-action="period-add" data-i="${i}" data-t="${ti}">${tr("mortgage.addFollowUp")}</button></div></div>`;
  }).join("");
  const p3a = anyIndirect ? `<p class="hint">${tr("mortgage.indirectHint")}</p>` : "";
  return `<h4>${tr("mortgage.title")}</h4>${mode}${tranches}
    <div style="margin:4px 0 0"><button class="btn small" data-action="tranche-add" data-i="${i}">${tr("mortgage.addTranche")}</button></div>
    <p class="hint">${C("mort", i)}</p>${p3a}`;
}
// #196: «Immobilien nie verkaufen» at the top; when on, the rest of the tab is grey and read-only (values are kept).
R.realestate = () => `<h2>${tr("section.realestate")}</h2><p class="lead">${tr("realestate.intro")}</p>
  <div class="card" id="neverSellCard">${Chk("realestate.neverSell", tr("realestate.neverSell"), { rerender: 1 })}
    <p class="hint">${tr("realestate.neverSellHint")}</p></div>
  <fieldset class="locked-area" id="realestateBody" ${state.realestate.neverSell ? "disabled" : ""}>
  <div class="card"><h3>${tr("realestate.emergency")}</h3>${Series("realestate.emergencyMonths", tr("realestate.threshold"), "months", tr("realestate.emergencyHint"))}
    <p class="hint">${tr("realestate.afterSaleHint")}</p></div>
  <p class="hint" id="affordNote">${tr("realestate.affordabilityNote", { values: affordText(affordability()) })}</p>
  ${state.realestate.properties.map((p, i) => {
    const P = `realestate.properties.${i}`;
    let body = "";
    if (p.status === "rent") body = `<div class="grid2"><label class="field"><span>${tr("realestate.rentFrom")}</span>${Ref(P + ".from", { afterSale: true, self: p.id })}</label>${F(P + ".rent", tr("realestate.rentMonth"), "chf")}</div>
      <div style="margin-top:10px">${Chk(P + ".inflationRent", tr("realestate.rentIndexed"))}</div>`;
    else body = `<div class="grid2 pgrid">
      ${p.status === "owned" ? F(P + ".bought", tr("realestate.buyDate"), "month") : `<label class="field"><span>${tr("realestate.buyFrom")}</span>${Ref(P + ".buy", { afterSale: true, self: p.id })}</label>`}
      ${F(P + ".price", p.status === "owned" ? tr("realestate.pricePaid") : tr("realestate.price"), "chf")}
      <label class="field"><span>${tr("realestate.sellFrom")}</span>${Ref(P + ".sell", { never: true })}</label>
      ${F(P + ".fixCosts", tr("realestate.costsMonth"), "chf")}</div>
      <div class="grid3" style="margin-top:10px">${Chk(P + ".inflationValue", tr("realestate.valueIndexed"))}${Chk(P + ".inflationFixCosts", tr("realestate.costsIndexed"))}</div>
      ${taxValueRow(p, i, P)}
      ${propertyGainRow(p, P)}
      ${MortgageUI(p, i)}`;
    return `<div class="prop"><div class="phead">${F(P + ".name", "", "text", { bare: true, size: Math.max(20, (p.name || "").length + 4) })}
      ${p.status === "owned" ? `<div class="pworth">${F(P + ".worth", tr("realestate.marketValue", { startTxt: startTxt() }), "chf")}</div>` : ""}
      <div class="seg">${PSTAT.map(([k, l]) => `<button class="${p.status === k ? "on" : ""}" data-action="pstat" data-i="${i}" data-v="${k}">${l}</button>`).join("")}</div>
      <button class="btn ghost small" data-action="list-del" data-path="realestate.properties" data-i="${i}" title="${tr("realestate.remove")}">✕</button></div>${body}</div>`;
  }).join("")}
  <button class="btn" data-action="prop-add">${tr("realestate.add")}</button></fieldset>`;

R.scenarios = () => `<h2>${tr("section.scenarios")}</h2><p class="lead">${tr("scenarios.intro")}</p>
  <div class="card"><p style="margin:0">${tr("scenarios.runHint")}</p></div>
  <div class="card"><h3>${tr("scenarios.variations")}</h3><table class="tbl swtbl"><colgroup><col><col style="width:110px"><col style="width:110px"><col style="width:110px"><col style="width:96px"><col style="width:36px"></colgroup><tr><th>${tr("scenarios.param")}</th><th>${tr("scenarios.from")}</th><th>${tr("scenarios.to")}</th><th>${tr("scenarios.step")}</th><th>${tr("scenarios.runs")}</th><th></th></tr>
  ${state.scenarios.sweeps.map((s, i) => {
    const t = SWEEP_PARAMS.find(p => p.id === s.param)?.type || "pct", P = `scenarios.sweeps.${i}`;
    return `<tr><td>${Sel(P + ".param", SWEEP_PARAMS.map(p => [p.id, p.label]), { rerender: 1 })}</td><td>${F(P + ".from", "", t, { bare: true })}</td><td>${F(P + ".to", "", t, { bare: true })}</td>
      <td>${F(P + ".step", "", t === "month" ? "months" : t, { bare: true, unit: t === "month" ? tr("unit.monthsShort") : undefined })}</td><td>${C("sweep", i)}</td>
      <td><button class="btn ghost small" data-action="list-del" data-path="scenarios.sweeps" data-i="${i}">✕</button></td></tr>`;
  }).join("")}</table>
  <div style="margin-top:10px"><button class="btn small" data-action="sweep-add">${tr("scenarios.add")}</button></div></div>
  <div class="card"><h3>${tr("scenarios.size")}</h3><p style="margin:0">${C("runs")} <span id="est-warn" class="timewarn"></span></p>
    <p class="hint est" id="est-scen" style="margin:6px 0 0"></p>
    <p class="hint">${tr("scenarios.coresHint")}</p></div>`;

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
  taxFigures: () => taxFigures(),
  taxSwitchHint: () => taxSwitchHint(),
  taxAssumptions: () => taxAssumptions(),
  dedSum: i => dedSum(i),
  dedParts: i => dedParts(i),
  wtaxHint: () => wtaxHint(),
  wtaxTable: () => wtaxTable(),
  // #55: Payout plan of pillar 3a (row «k.j»): month and age, balance before and after the payout from the preview.
  p3aat: a => {
    const [k, j] = a.split(".").map(Number), acc = state.pillar3a[k], x = acc?.payouts[j];
    const m = x ? payoutMonth(x, acc.person) : NaN;
    return Number.isFinite(m) ? `<span data-p3a="at">${tr("computed.msAge", { ymShow: ymShow(i2ym(m)), m: ageAt(m, acc.person).toLocaleString("de-CH", { minimumFractionDigits: 1, maximumFractionDigits: 1 }) })}</span>` : `<span data-p3a="at">${tr("computed.invalid")}</span>`;
  },
  p3apay: a => {
    const [k, j] = a.split(".").map(Number), pv = p3aViewOf(state.pillar3a[k]?.person)?.payouts.find(o => o.index === j);
    return `<span data-p3a="before">${pv ? fmtInt(pv.before) : "–"}</span> → <span data-p3a="after">${pv ? fmtInt(pv.after) : "–"}</span>`;
  },
  p3arest: k => {
    const acc = state.pillar3a[+k], pi = state.household.persons.findIndex(q => q.id === acc?.person), ahv = msMonth(pi === 1 ? "ahv2" : "ahv");
    const view = p3aViewOf(acc?.person);
    return `${tr("computed.pillar3aRest", { ahv: Number.isFinite(ahv) ? ymShow(i2ym(ahv)) : "?" })} <b data-p3a="rest">${view ? fmtInt(view.rest?.amount ?? 0) + " " + cur() : "–"}</b>`;
  },
  housingshare: () => {
    if (!state.flows.spendingIncludesHousing) return "";
    const w = lastSim?.housingShare;
    return tr("computed.spendingHousing", { v: w != null ? fmtMoney(Math.round(w)) : "–" });
  },
  // #191: AHV pension the simulation computes with (reduction or supplement, for couples the capped share)
  ahvEff: arg => {
    const v = lastAhv.find(x => x.person === Number(arg));
    if (!v || !v.start) return "";
    const money = x => `${x.toLocaleString("de-CH", { minimumFractionDigits: 2, maximumFractionDigits: 2 })} ${cur()}`;
    const rate = `${v.rate < 0 ? "−" : "+"}${fmtPct(Math.abs(v.rate))}`;
    const eff = v.rate === 0 ? tr("computed.ahvEffective", { money: money(v.monthly) }) : tr("computed.ahvEffectiveRate", { rate, money: money(v.monthly) });
    return eff + (v.capped != null && v.cappedFrom ? tr("computed.ahvCapped", { ym: ymShow(v.cappedFrom), money: money(v.capped) }) : "");
  },
  ms: id => { const m = msMonth(id); return Number.isFinite(m) ? tr("computed.msAge", { ymShow: ymShow(i2ym(m)), m: ageStr(m) }) : tr("computed.invalid"); },
  age1: () => { const s = startM(); return Number.isFinite(s) && state.household.persons[1] ? tr("computed.ageAtStart", { s: ageAt(s, state.household.persons[1].id).toLocaleString("de-CH", { minimumFractionDigits: 1, maximumFractionDigits: 1 }) }) : ""; },
  wdinit: () => { const e = lastSim?.events.find(x => x.kind === "withdrawal"); return e ? tr("computed.preview", { label: esc(e.label), ymShow: ymShow(i2ym(e.m)) }) : tr("computed.previewNone"); },
  age0: () => { const s = startM(); return Number.isFinite(s) ? tr("computed.ageAtStart", { s: ageStr(s) }) : ""; },
  pkresult: () => lastSim ? `${state.partner ? tr("computed.householdAtRetire") : tr("computed.atRetire")}${tr("computed.pkAtRetire", { lumpNet: fmtMoney(lastSim.lumpNet), lumpTax: `${fmtTax(lastSim.lumpTax)} ${cur()}`, pkPension: fmtMoney(lastSim.pkPension) })} ${tr("taxes.checkBeforeDecision")}` : "", // #143/AK2, AK6
  // #73: Share of stocks or bonds in the portfolio (without cash), integer; without a portfolio no value.
  wquote: arg => {
    const w = state.wealth, s = Math.max(0, +w.stocks || 0), b = Math.max(0, +w.bonds || 0);
    return s + b > 0 ? tr("computed.portfolioShare", { round: Math.round(100 * (arg === "stocks" ? s : b) / (s + b)) }) : "";
  },
  wealthsum: () => { const w = state.wealth; return tr("computed.total", { money: fmtMoney(w.stocks + w.bonds + w.cash) }); },
  rolling: () => {
    if (!seriesInfo) return tr("computed.seriesLoading");
    if (seriesInfo.error) return `<span style="color:var(--err)">${esc(seriesInfo.error)}</span>`;
    return tr("computed.rollingCover", { int: fmtInt(rollingCount()), v: ((endM() - startM()) / 12).toFixed(1) });
  },
  // #69: Sum of the weights of «Welt individuell»
  wsum: () => {
    const w = state.returns.strategy?.weights || {}, sum = REGION_KEYS.reduce((a, k) => a + (+w[k] || 0), 0);
    return `${tr("computed.sum")} <b${Math.abs(sum - 1) > 1e-6 ? ' style="color:var(--err)"' : ""}>${tr("computed.sumMust", { sum: fmtPct(sum) })}`;
  },
  // #69: Short info on the time series of the strategy
  strategyinfo: () => {
    const i = seriesInfo;
    if (!i) return "";
    if (i.error) return ["weights_range", "weights_sum"].includes(i.code) ? "" : `<span style="color:var(--err)">${esc(i.error)}</span>`; // weights: message at the field
    const p = v => v == null || Number.isNaN(v) ? "–" : `${(v * 100).toLocaleString("de-CH", { minimumFractionDigits: 1, maximumFractionDigits: 1 })} %`;
    // #69/AK8: The means apply to the common period; a longer index is cut to it.
    const from = +String(i.first).slice(0, 4), to = +String(i.last).slice(0, 4), span = `${from}–${to}`;
    const cut = (i.parts || []).filter(x => x.first < from || x.last > to);
    const clip = cut.length ? `<span class="cliphint" id="strategyClip">⚠ ${cut.map(x => tr("computed.seriesRange", { label: esc(msgText(x.label)), first: x.first, last: x.last })).join(", ")} ${tr("computed.commonRange", { span })}</span>` : "";
    return `${i.monthly ? tr("computed.monthly") : tr("computed.annual")} ${tr("computed.indexRange", { first: ymShow(i.first), last: ymShow(i.last) })}${i.parts?.length ? ` ${tr("computed.inCommon", { span })}` : ""}${tr("computed.averages", { avgStocks: p(i.avgStocks), avgBonds: p(i.avgBonds), avgInflation: p(i.avgInflation), i: seriesTip(i), clip })}`;
  },
  seriesinfo: () => {
    if (!seriesInfo) return "";
    if (seriesInfo.error) return `<span style="color:var(--err)">${esc(seriesInfo.error)}</span>`;
    return "";
  },
  basis: () => esc(previewBasis ? msgText(previewBasis) : tr("computed.fixedValues")),
  // #100: Suggested value next to a field of the stock model (only with own values)
  psug: k => {
    const q = state.returns.parametric, s = paramSuggestion();
    return !s || q.auto ? "" : tr("computed.suggestion", { k: fmtParam(k, s[k]) });
  },
  pauto: () => {
    const q = state.returns.parametric, s = paramSuggestion();
    if (!seriesInfo) return tr("computed.suggestionLoading");
    if (!s) return tr("computed.noSuggestion");
    const st = state.returns.strategy, y = seriesInfo.parametric.stockYears;
    const name = state.returns.series ? seriesShort(state.returns.series) : STOCK_INDICES.find(x => x[0] === st?.stocks)?.[1] ?? "–";
    return q.auto ? tr("computed.followIndex", { name: esc(name), v: y[0], v2: y[1] })
      : `${tr("computed.ownValues", { name: esc(name) })} <button class="btn small" data-action="param-suggest">${tr("computed.adoptSuggestion")}</button>`;
  },
  // #101: Mean, spread and correlation of stocks, bonds and inflation of the series
  pjoint: () => {
    const j = paramJoint();
    if (!seriesInfo) return "";
    if (!j) return tr("computed.noSeries");
    const p = v => `${(v * 100).toLocaleString("de-CH", { minimumFractionDigits: 1, maximumFractionDigits: 1 })} %`;
    const c = v => v.toLocaleString("de-CH", { minimumFractionDigits: 2, maximumFractionDigits: 2 }).replace("-", "−");
    const names = [tr("wealth.stocks"), tr("wealth.bonds"), tr("returns.inflation")];
    return `<table class="tbl num"><thead><tr><th>${tr("computed.correlation")}</th>${names.map(n => `<th>${n}</th>`).join("")}<th>${tr("computed.mean")}</th><th>${tr("computed.spread")}</th></tr></thead><tbody>
      ${names.map((n, a) => `<tr><td>${n}</td>${names.map((_, b) => `<td>${a === b ? "1" : c(j.corr[a][b])}</td>`).join("")}<td>${a === 1 ? p(j.bondsMean) : a === 2 ? p(j.inflationMean) : tr("computed.model")}</td><td>${a === 1 ? p(j.bondsSd) : a === 2 ? p(j.inflationSd) : tr("computed.model")}</td></tr>`).join("")}</tbody></table>
      <p class="hint">${tr("computed.jointYears", { first: j.first, last: j.last })}</p>`;
  },
  bucket: k => {
    const rm = refMonth(state.pension.pk.at); if (!lastSim) return "–";
    const i = lastSim.m.indexOf(rm); if (i < 0) return "–";
    return fmtMoney(lastSim.pots[i][{ cash: 0, bond: 1, stock: 2 }[k]]);
  },
  mrow: a => {
    const [i, ti, k] = a.split(":").map(Number), p = state.realestate.properties[i];
    const r = trancheSchedule(p, p.mortgage.tranches[ti]).find(x => x.i === k);
    if (!r) return `<span style="color:var(--faint)">${refMonth(p.buy) === Infinity ? tr("computed.startsAtPurchase") : tr("computed.afterSaleEnd")}</span>`;
    if (p.mortgage.amountMode === "maxLoan" && p.status !== "owned") return `${ymShow(i2ym(r.start))} – ${ymShow(i2ym(r.end))}`;
    const span = `<span style="white-space:nowrap">${ymShow(i2ym(r.start))} –</span> <span style="white-space:nowrap">${ymShow(i2ym(r.end))}</span>`;
    if (r.past) return `${span} <span style="color:var(--faint)">· ${tr("computed.expired")}</span>`;
    return `${span}${r.d.type === "saron" ? ` · ${fmtPct(r.rate)} %` : ""}<br><b>${fmtMoney(r.a1)}</b>${!r.direct && r.q ? ` <span style="color:var(--faint)">(+ 3a${r.toAhv ? tr("computed.untilAhv") : ""})</span>` : ""}`;
  },
  mort: i => {
    const p = state.realestate.properties[i]; if (!p?.mortgage) return "";
    if (p.status !== "owned" && p.mortgage.amountMode === "maxLoan") {
      const a = affordability();
      return tr("computed.maxLoan", { money: fmtMoney(p.price * a.maxLoanToValue), maxLoanToValue: fmtPct(a.maxLoanToValue), price: fmtMoney(p.price) });
    }
    const base = p.status === "owned" ? p.worth : p.price, total = mortTotal(p), multi = p.mortgage.tranches.length > 1;
    let interest = 0, direct = 0, indirect = 0; const parts = [];
    for (const t of p.mortgage.tranches) {
      const sch = trancheSchedule(p, t), cur = sch.find(x => !x.past), last = sch[sch.length - 1];
      if (cur) {
        interest += cur.a0 * cur.rate / 12; if (cur.direct) direct += cur.q; else indirect += cur.q;
        let s = `${multi ? `${esc(t.name)}: ` : ""}${p.status === "owned" ? tr("computed.currentPeriod") : tr("computed.firstPeriod")} ${tr("computed.until", { ymShow: ymShow(i2ym(cur.end)) })}`;
        if (p.status === "owned") s += ` ${tr("computed.remaining", { v: ((cur.end - cur.from) / 12).toLocaleString("de-CH", { maximumFractionDigits: 1 }) })}`;
        if (last && t.after === "repay" && last.end <= endM()) s += tr("computed.thenRepay", { money: fmtMoney(last.a1 ?? t.amount) });
        else if (last?.renewal) s += tr("computed.thenExtend");
        parts.push(s);
      }
    }
    return `${tr("computed.mortgageTotal", { total: fmtMoney(total), v: (total / base * 100).toLocaleString("de-CH", { maximumFractionDigits: 1 }) })} ${p.status === "owned" ? tr("computed.today") : tr("computed.atPurchase")} ${tr("computed.interestMonth", { interest: fmtMoney(interest), direct: fmtMoney(direct) })}${indirect ? `, ${tr("computed.indirect", { amount: fmtMoney(indirect) })}` : ""}<br>${parts.join("<br>")}`;
  },
  sweep: i => fmtInt(sweepCount(state.scenarios.sweeps[i])),
  runs: () => {
    const sw = state.scenarios.sweeps.map(sweepCount), combos = sw.reduce((a, b) => a * b, 1), per = runsPerScenario();
    const over = combos > MAX_COMBINATIONS ? ` <span style="color:var(--err)">${tr("computed.maxCombos", { maxCombinations: fmtInt(MAX_COMBINATIONS), combos: fmtInt(combos) })}</span>` : "";
    return `${sw.length ? sw.map(fmtInt).join(" × ") + tr("computed.combos") : tr("computed.oneScenario")} × ${fmtInt(per)} ${per === 1 ? tr("computed.run") : tr("computed.runs")} ${tr("computed.totalRuns", { int: fmtInt(combos * per), over })}`;
  }
};
/** Section of the editor for a main key of the plan (for the messages when deleting a milestone). */
let SECTION_OF;
onLanguage(() => SECTION_OF = { household: tr("section.general"), pension: tr("section.pension"), partner: tr("section.pension"), pillar3a: tr("section.pillar3a"), flows: tr("section.flows"), wealth: tr("section.wealth"), returns: tr("section.returns"),
  withdrawal: tr("section.withdrawal"), optimization: tr("section.optimize"), taxes: tr("section.taxes"), realestate: tr("section.realestate"), scenarios: tr("section.scenarios") });
/** Uses of a milestone outside the milestone table: [{ section, set(value) }]. */
function milestoneRefs(id) {
  const out = [];
  const walk = (o, top) => {
    if (!o || typeof o !== "object") return;
    for (const [k, v] of Object.entries(o)) {
      if (top === "" && k === "milestones") continue;
      if (v === id) out.push({ section: SECTION_OF[top || k] || tr("refs.plan"), parent: o, key: k });
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
      + `<div class="navfoot">${tr("nav.localData")}<a class="navsupport" href="#" data-action="link" data-link="donate" title="${esc(tr("nav.supportTitle"))}">${tr("nav.support")}</a></div>`;
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
  // #200: in «Optimierung» the right column (preview, validation) gives its width to the result table
  $(".layout").classList.toggle("wide", current === "optimize");
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
  // #171: the basis of the preview (message of the engine) as subtitle «Einzellauf mit …» (keys `derived.<code>`)
  const sub = previewBasis ? engineText(previewBasis, key => I18n.text(key.replace(/^engine\./, "derived."))) : tr("derived.singleFixed");
  $("#previewSub").textContent = `${sub} ${tr("derived.live")}`;
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
    : `<li class="ok"><span class="ic">✓</span><span>${tr("validation.noErrors")}</span></li>`;
  // #112: only rebuild if the messages change (otherwise a click on a message would be lost)
  if (html !== renderVal.html) { $("#val").innerHTML = html; renderVal.html = html; }
}

function renderKpis() {
  const s = lastSim, k = $("#kpis");
  if (!s) { k.innerHTML = `<div class="kpi bad"><div class="k">${tr("kpi.preview")}</div><div class="v">–</div><div class="s">${previewError ? tr("kpi.seeValidation") : tr("kpi.computing")}</div></div>`; return; }
  const last = s.total[s.total.length - 1], e = endM();
  k.innerHTML = `
    <div class="kpi"><div class="k">${tr("kpi.wealthAtRetire")}</div><div class="v">${s.totalAtRetire != null ? fmtMio(s.totalAtRetire) : "–"}</div><div class="s">${ymShow(i2ym(s.retireM))}</div></div>
    <div class="kpi ${last > 0 ? "good" : "bad"}"><div class="k">${tr("kpi.wealthAtEnd")}</div><div class="v">${fmtMio(last)}</div><div class="s">${tr("kpi.ageNominal", { e: ageStr(e) })}</div></div>
    <div class="kpi ${s.depleted !== null ? "bad" : "good"}"><div class="k">${tr("kpi.moneyLasts")}</div><div class="v">${s.depleted !== null ? tr("kpi.age") + ageStr(s.depleted) : tr("kpi.endOk")}</div><div class="s">${s.depleted !== null ? ymShow(i2ym(s.depleted)) : tr("kpi.wholePeriod")}</div></div>
    <div class="kpi"><div class="k">${tr("kpi.minLiquid")}</div><div class="v">${fmtMio(Math.max(0, s.minLiquid))}</div><div class="s">${tr("kpi.ageAt", { minLiquidM: ageStr(s.minLiquidM) })}</div></div>`;
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
    tip.innerHTML = `${tr("chart.tip", { ymShow: ymShow(i2ym(s.m[i])), ageStr: ageStr(s.m[i]), money: fmtMoney(s.total[i]) })}<br><span style="color:var(--c1)">■</span> ${tr("chart.liquid", { money: fmtMoney(s.liquid[i]) })}<br><span style="color:var(--c2)">■</span> ${tr("chart.property", { money: fmtMoney(s.prop[i]) })}<br><span style="color:var(--c3)">■</span> ${tr("chart.pension", { money: fmtMoney(s.pk[i]) })}`;
    const left = x(i) * r.width / W; tip.style.left = (left > r.width / 2 ? left - tip.offsetWidth - 10 : left + 10) + "px"; tip.style.top = "8px";
  };
  svg.onmouseleave = () => { $(tipSel).style.display = "none"; svg.querySelector("#hov")?.setAttribute("visibility", "hidden"); };
}

/** #182/AK4: width of a text in pixels (the labels of the timeline are longer in French or Italian than estimated per character). */
const textCtx = document.createElement("canvas").getContext("2d");
function textWidth(text, size, weight = 400, el = document.body) {
  textCtx.font = `${weight} ${size}px ${getComputedStyle(el).fontFamily}`;
  return textCtx.measureText(text).width;
}

function renderTimeline() {
  const svg = $("#tl"), W = svg.clientWidth || 900, s0 = startM(), e = endM();
  $("#tlRange").textContent = Number.isFinite(s0) && e > s0 ? tr("timeline.range", { ymShow: ymShow(i2ym(s0)), ymShow2: ymShow(i2ym(e)), v: ((e - s0) / 12).toFixed(1) }) : tr("timeline.invalid");
  if (!(e > s0)) { svg.innerHTML = ""; return; }
  const P = 14, x = m => P + (m - s0) / (e - s0) * (W - 2 * P);
  /** Lane (row) per label: the first one free from the left; a new lane instead of overlapping (#182/AK4: longer texts). */
  const placeLane = (lanes, lx0, w) => { let k = lanes.findIndex(r => r < lx0 - 6); if (k < 0) k = lanes.push(-Infinity) - 1; lanes[k] = lx0 + w; return k; };
  const items = state.milestones.map(ms => ({ ms, m: msMonth(ms.id) })).filter(o => Number.isFinite(o.m)).sort((a, b) => a.m - b.m);
  const lanes = [-Infinity, -Infinity, -Infinity];
  for (const o of items) {
    o.when = `${ymShow(i2ym(o.m))} · ${ageStr(o.m)} ${tr("unit.yearsShort")}`; o.px = x(clamp(o.m, s0, e));
    o.w = textWidth(msLabel(o.ms), 11.5, 600, svg) + textWidth(" " + o.when, 11.5, 400, svg) + 6;
    o.anchorEnd = o.px + o.w > W - P; o.lx0 = o.anchorEnd ? o.px - o.w : o.px;
    o.lane = placeLane(lanes, o.lx0, o.w);
  }
  const base = 66 + (lanes.length - 3) * 15;
  let h = `<line x1="${P}" x2="${W - P}" y1="${base}" y2="${base}" stroke="${css("--border")}" stroke-width="4" stroke-linecap="round"/>`;
  const ret = refMonth(state.pension.pk.at);
  if (ret > s0 && ret < e) h += `<line x1="${x(ret)}" x2="${W - P}" y1="${base}" y2="${base}" stroke="${css("--accent")}" stroke-opacity=".35" stroke-width="4" stroke-linecap="round"/>`;
  for (let y = Math.ceil(s0 / 60) * 5; y * 12 <= e; y += 5) { const m = y * 12; h += `<line x1="${x(m)}" x2="${x(m)}" y1="${base + 4}" y2="${base + 9}" stroke="${css("--faint")}"/><text x="${x(m)}" y="${base + 21}" font-size="10.5" text-anchor="middle" fill="${css("--faint")}">${y} · ${Math.round(ageAt(m))} ${tr("unit.yearsShort")}</text>`; }
  for (const xx of state.flows.extras) { const m = ym2i(xx.date); if (m >= s0 && m <= e) h += `<rect x="${x(m) - 3.5}" y="${base - 3.5}" width="7" height="7" transform="rotate(45 ${x(m)} ${base})" fill="${css("--warn")}"><title>${tr("timeline.payout", { label: esc(xx.label), amount: fmtMoney(xx.amount), date: ymShow(xx.date), m: ageStr(m) })}</title></rect>`; }
  for (const ev of lastSim?.events || []) if (ev.tick) h += `<line x1="${x(ev.m)}" x2="${x(ev.m)}" y1="${base - 8}" y2="${base + 8}" stroke="${css("--c2")}" stroke-width="2.5"><title>${tr("timeline.event", { label: esc(ev.label), ymShow: ymShow(i2ym(ev.m)), m: ageStr(ev.m) })}</title></line>`;
  if (lastSim?.depleted != null) { const m = lastSim.depleted; h += `<line x1="${x(m)}" x2="${x(m)}" y1="${base - 9}" y2="${base + 9}" stroke="${css("--err")}" stroke-width="3"><title>${tr("timeline.depleted")}</title></line>`; }
  for (const { ms, when, px, w, anchorEnd, lx0, lane } of items) {
    const ly = 14 + lane * 15;
    h += `<g class="ms" data-action="ms-goto" data-id="${ms.id}"><line x1="${px}" x2="${px}" y1="${ly + 3}" y2="${base}" stroke="${css("--faint")}" stroke-width="1"/>
      <circle cx="${px}" cy="${base}" r="6" fill="${css(ms.fixed ? "--muted" : "--accent")}" stroke="${css("--panel")}" stroke-width="2"/>
      <text class="lbl" x="${anchorEnd ? px - 4 : px + 4}" y="${ly}" font-size="11.5" text-anchor="${anchorEnd ? "end" : "start"}" fill="${css("--text")}"><tspan font-weight="600">${esc(msLabel(ms))}</tspan> <tspan fill="${css("--muted")}">${when}</tspan></text>
      <rect x="${lx0}" y="${ly - 12}" width="${w}" height="16" fill="transparent"/></g>`;
  }
  // Property events (only with a simulation result): triangle below the line, label underneath, staggered.
  // Shape and position distinguish them from the milestones (circle on the line, label above).
  const pev = (lastSim?.propEvents || []).filter(o => o.m >= s0 && o.m <= e).sort((a, b) => a.m - b.m);
  const TL_EV = { purchase: tr("timeline.buy"), sale: tr("timeline.sell"), emergencySale: tr("timeline.emergency") }, compact = pev.length > 15;
  const evLanes = [-Infinity, -Infinity, -Infinity], top = base + 38;
  for (const o of pev) {
    const label = `${TL_EV[o.kind]} ${o.property} ${ymShow(i2ym(o.m))}`, w = textWidth(label, 11, 400, svg) + 6, px = x(o.m), color = css(o.kind === "emergencySale" ? "--warn" : "--ok");
    const anchorEnd = px + w > W - P, lx0 = anchorEnd ? px - w : px;
    const ly = top + placeLane(evLanes, lx0, w) * 14;
    h += `<g class="pev" data-kind="${o.kind}"><title>${tr("timeline.eventAge", { label: esc(label), m: ageStr(o.m) })}</title>
      ${compact ? "" : `<line x1="${px}" x2="${px}" y1="${base + 12}" y2="${ly - 9}" stroke="${color}" stroke-width="1" stroke-opacity=".6"/>`}
      <path d="M${px},${base + 3} l-5,9 h10 z" fill="${color}" stroke="${css("--panel")}" stroke-width="1.5"/>
      ${compact ? "" : `<text class="pelbl" x="${anchorEnd ? px - 4 : px + 4}" y="${ly}" font-size="11" text-anchor="${anchorEnd ? "end" : "start"}" fill="${color}">${esc(label)}</text>`}</g>`;
  }
  svg.setAttribute("height", pev.length && !compact ? top + evLanes.length * 14 - 4 : base + 30);
  svg.innerHTML = h;
}

/** Visible range per mini chart (month indices); without an entry the whole period. */
const miniZoom = {};
/** Shared by editor and result charts: zoom buttons, default «the first N years» and visible range. */
let ZOOMS;
onLanguage(() => ZOOMS = [["0", tr("zoom.all")], ...["10", "5", "2"].map(y => [y, `${y} ${tr("unit.yearsShort")}`])]);
const zoomButtons = (action, active, attrs = "") => ZOOMS.map(([y, l]) => `<button data-action="${action}" ${attrs} data-years="${y}" class="${String(active ?? 0) === y ? "on" : ""}">${l}</button>`).join("");
const presetZoom = years => (years ? { from: startM(), to: Math.min(endM(), startM() + years * 12), years } : null);
function zoomRange(z) {
  const s0 = startM(), e = endM();
  if (!z) return [s0, e];
  const span = clamp(z.to - z.from, 6, e - s0), from = clamp(z.from, s0, e - span);
  return [from, from + span];
}
const miniRange = path => zoomRange(miniZoom[path]);
function miniValue(t, v) { return t === "chfyear" ? fmtInt(v * 12) : t === "pct" ? fmtPct(v) + " %" : t === "months" ? `${v} ${tr("unit.monthsShort")}` : t === "years" ? `${v} ${tr("unit.yearsShort")}` : fmtInt(v); }
function drawMini(svg) {
  const path = svg.dataset.mini, ser = getP(path), t = svg.dataset.type;
  if (!ser || !(endM() > startM())) { svg.innerHTML = ""; return; }
  const [a, b] = miniRange(path), W = svg.clientWidth || 220, H = svg.clientHeight || 96, T = 14, Bm = 16;
  svg.setAttribute("viewBox", `0 0 ${W} ${H}`);
  // Exact steps: value at the start of the section and at every change within it.
  // #217: a straight line or table month by month.
  const changes = ser.mode === "timed" ? (ser.rows || []).map(r => refMonth(r.from)).filter(m => m > a && m <= b)
    : isNewForm(ser) ? Array.from({ length: Math.max(0, b - a) }, (_, i) => a + i + 1) : [];
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
    if (mm > a && mm < b) g += `<line x1="${X(mm)}" x2="${X(mm)}" y1="${T}" y2="${H - Bm}" stroke="${css("--accent")}" stroke-opacity=".35" stroke-dasharray="2 3"><title>${esc(msLabel(m))}</title></line>`;
  }
  const dots = isNewForm(ser) ? "" : pts.slice(1).map((m, i) => `<circle cx="${X(m)}" cy="${Y(vals[i + 1])}" r="3" fill="${css("--accent")}"/>`).join("");
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
    tip.innerHTML = tr("mini.tip", { ymShow: ymShow(i2ym(m)), m: ageStr(m), v: miniValue(t, seriesAt(ser, m)) });
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
  svg.innerHTML = `<path d="${d}" fill="none" stroke="${css("--accent")}" stroke-width="2" vector-effect="non-scaling-stroke"/><text x="4" y="11" font-size="10" fill="${css("--muted")}">${tr("mini.rateUpTo", { ymax: fmtPct(ymax) })}</text>`;
}

// ---------- Events ----------
/** Tells the desktop app whether there are unsaved changes (for the question when quitting). */
function pushDirty() { if (Engine.inApp) Engine.call("set_dirty", { dirty }).catch(() => {}); }
function markDirty() { dirty = true; $("#dirty").textContent = tr("file.unsaved"); pushDirty(); History.note(); stepTouched(); }

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
  undo() { this.commit(); if (this.pos <= 0) return toast(tr("history.nothingUndo")); this.pos--; this.apply(); },
  redo() { this.commit(); if (this.pos >= this.stack.length - 1) return toast(tr("history.nothingRedo")); this.pos++; this.apply(); },
  apply() {
    state = JSON.parse(this.stack[this.pos]); invalidFields.clear(); dirty = true; $("#dirty").textContent = tr("file.unsaved"); pushDirty();
    if (!$("#results").hidden) showResults(false);
    renderSection(); refreshSeriesInfo();
  }
};

// ---------- Keyboard shortcuts ----------
const IS_MAC = /Mac|iPhone|iPad/.test(navigator.platform || navigator.userAgent);
const MOD = IS_MAC ? "⌘" : "Ctrl+", SHIFT = IS_MAC ? "⇧" : "Shift+";
let SHORTCUTS;
onLanguage(() => SHORTCUTS = [
  [`${MOD}N`, tr("shortcuts.new"), "new"], [`${MOD}O`, tr("shortcuts.open"), "open"], [`${MOD}S`, tr("shortcuts.save"), "save"], [`${SHIFT}${MOD}S`, tr("shortcuts.saveAs"), "saveas"],
  [`${MOD}I`, tr("shortcuts.import"), "import"], [`${MOD},`, tr("shortcuts.settings"), "settings"], [`${MOD}↩`, tr("shortcuts.run"), "run"], ["Esc", tr("shortcuts.back"), "back"],
  [`${MOD}Z`, tr("shortcuts.undo"), "undo"], [`${SHIFT}${MOD}Z`, tr("shortcuts.redo"), "redo"],
  [`${MOD}1 … ${MOD}9, ${MOD}0`, tr("shortcuts.sections"), null],
  [`${MOD}↑ / ${MOD}↓`, tr("shortcuts.prevNext"), null], [`${MOD}/`, tr("shortcuts.overview"), "help"],
  ["F1", tr("shortcuts.helpTopic"), null], [`${SHIFT}${MOD}?`, tr("shortcuts.manual"), null]
]);
let lastCommand = { id: "", t: 0 };
/** Runs a command (menu of the desktop app or keyboard); a duplicate trigger is ignored. */
function command(id) {
  const now = performance.now();
  if (lastCommand.id === id && now - lastCommand.t < 250) return;
  lastCommand = { id, t: now };
  if (betaLocked) return;
  if ($("#ask").open && id !== "help") return;
  if ($("#welcome").open) return;
  if ($("#disclaimer").open) return;
  if (id === "undo") return History.undo();
  if (id === "redo") return History.redo();
  if (id === "help") return ask(tr("shortcuts.title"), `<div class="kbd">${SHORTCUTS.map(([k, t]) => `<kbd>${esc(k)}</kbd><span>${esc(t)}</span>`).join("")}</div>`);
  if (id === "about") return showAbout();
  if (id === "welcome") return showWelcome();
  if (id === "steps") return Steps.show();
  if (id === "support") return openLink("donate");
  if (id === "videos") return showVideos();
  if (id === "connectai") return showConnectAi();
  if (id === "back") { if (!$("#results").hidden) showResults(false); return; }
  if (/^recent-\d+$/.test(id)) return openRecent(+id.slice(7));
  ACTIONS[id]?.({});
}
// #237: External addresses: fixed list in links.json (id → address), the same file the desktop app compiles in. The desktop
// app opens an entry in the default browser (command open_link takes only ids of the list), the development server
// in a new browser tab. Nothing is loaded from the internet.
let linkTable = null;
async function linkUrl(id) {
  linkTable ??= fetch("links.json").then(r => r.json());
  return (await linkTable)[id];
}
async function openLink(id) {
  try {
    if (Engine.inApp) return await Engine.call("open_link", { id });
    const url = await linkUrl(id);
    if (url) window.open(url, "_blank", "noopener");
  } catch (e) { toast(errText(e)); }
}
// #240: Introductory videos, one per length (2min, 10min) and language; opened like every external address via the
// fixed list (ids video-<length>-<language>), in the language set.
const videoLink = len => `video-${len}-${I18n.language()}`;
function showVideos() {
  const row = (len, min, key) => `<li><a href="#" data-action="video" data-video="${len}"><b>${tr(`videos.${key}`)}</b></a>
    · ${tr("videos.length", { min })}<br><span class="hint">${tr(`videos.${key}Text`)}</span></li>`;
  ask(tr("videos.title"), `<ul id="videoList" style="margin:0;padding-left:18px;line-height:1.6">${row("2min", 2, "short")}${row("10min", 10, "long")}</ul>
    <p class="hint">${tr("videos.browser")}</p>`);
}
// #213: «Mit KI verbinden»: one instruction per AI program with the place of its configuration and a setup line to
// copy. The app only shows and copies; it writes no configuration and loads nothing. #263: every line carries the
// plan folder (--allow-dir) and, if chosen, --read-only. The path of the program comes
// from the desktop app (`cliPath` of app_info, the program next to the app); without it the usual path of the system.
const IS_WIN = /Win/.test(navigator.platform || navigator.userAgent);
const CLI_DEFAULT = IS_MAC ? "/Applications/VivaPlan.app/Contents/MacOS/vivaplan"
  : IS_WIN ? "%LOCALAPPDATA%\\VivaPlan\\vivaplan.exe" : "/usr/bin/vivaplan";
let aiLines = [];
/** #263: what the dialog shows: program path, version, plan folder (--allow-dir) and switch «read only». */
const aiSetup = { cli: "", version: "…", dir: "", readOnly: false };
/** Arguments of the server: #263/AK4 the plan folder as --allow-dir, AK5 «read only» as --read-only. */
const aiArgs = (dir, readOnly) => ["mcp", "--allow-dir", dir, ...(readOnly ? ["--read-only"] : [])];
/** Setup per AI program: name (product names, not translated), text key, configuration file, line to copy. */
function aiPrograms({ cli, version, dir, readOnly }) {
  const home = IS_WIN ? "%USERPROFILE%" : "~";
  const sep = IS_WIN ? "\\" : "/";
  const args = aiArgs(dir, readOnly);
  const json = JSON.stringify({ mcpServers: { vivaplan: { command: cli, args } } });
  const shell = a => /\s/.test(a) ? `"${a}"` : a;
  const claudeConfig = IS_MAC ? "~/Library/Application Support/Claude/claude_desktop_config.json"
    : IS_WIN ? "%APPDATA%\\Claude\\claude_desktop_config.json" : "~/.config/Claude/claude_desktop_config.json";
  return [
    { id: "claudeDesktop", name: "Claude Desktop", config: claudeConfig, line: json,
      extra: IS_MAC ? tr("ai.claudeDesktop.mcpb", { file: `VivaPlan_${version}_aarch64.mcpb` }) : "" },
    { id: "claudeCode", name: "Claude Code", config: "", line: `claude mcp add --scope user vivaplan -- ${[cli, ...args].map(shell).join(" ")}` },
    // TOML: the program as a literal string (backslashes stay); the arguments as basic strings, escaped like JSON
    { id: "chatgpt", name: "ChatGPT Desktop", config: `${home}${sep}.codex${sep}config.toml`,
      line: `[mcp_servers.vivaplan]\ncommand = '${cli}'\nargs = [${args.map(a => JSON.stringify(a)).join(", ")}]` },
    { id: "lmstudio", name: "LM Studio", config: `${home}${sep}.lmstudio${sep}mcp.json`, line: json },
  ];
}
/** #263: writes the plan folder and the setup lines anew after «Ordner wählen …» or the switch «read only». */
function aiRender() {
  aiLines = [aiSetup.cli, ...aiPrograms(aiSetup).map(p => p.line)];
  const dir = $("#aiDir");
  if (dir) dir.textContent = aiSetup.dir;
  document.querySelectorAll("#connectAi section.aiprog pre").forEach((pre, i) => { pre.textContent = aiLines[i + 1]; });
}
async function showConnectAi() {
  let info = null;
  try { info = await Engine.call("app_info"); } catch { /* without the engine the usual path */ }
  Object.assign(aiSetup, { cli: info?.cliPath || CLI_DEFAULT, version: info?.version ?? "…",
    dir: info?.planDir || (IS_WIN ? "%USERPROFILE%\\Documents" : "~/Documents"), readOnly: false });
  const programs = aiPrograms(aiSetup);
  aiLines = [aiSetup.cli, ...programs.map(p => p.line)];
  const copy = i => `<button type="button" class="btn aicopy" data-action="ai-copy" data-copy="${i}" title="${esc(tr("ai.copyTitle"))}">${tr("ai.copy")}</button>`;
  const code = i => `<div class="aicode"><pre>${esc(aiLines[i])}</pre>${copy(i)}</div>`;
  const section = (p, i) => `<section class="aiprog" data-program="${p.id}"><h4>${esc(p.name)}</h4>
    ${p.extra ? `<p class="hint">${p.extra}</p>` : ""}
    <p class="hint">${tr(`ai.${p.id}.steps`, { config: `<code>${esc(p.config)}</code>` })}</p>${code(i + 1)}</section>`;
  const pick = Engine.inApp ? `<button type="button" class="btn aipick" data-action="ai-folder">${tr("ai.chooseFolder")}</button>` : "";
  ask(tr("ai.title"), `<div id="connectAi"><p style="margin-top:0">${tr("ai.intro")}</p>
    <p class="hint">${tr("ai.path")}</p>${code(0)}
    <p class="hint">${tr("ai.folder")}</p><div class="aidir"><pre id="aiDir">${esc(aiSetup.dir)}</pre>${pick}</div>
    <label class="aiopt"><input type="checkbox" id="aiReadOnly"> ${tr("ai.readOnly")}</label>
    ${programs.map(section).join("")}
    <p class="hint">${tr("ai.privacy")} <a href="#" data-action="ai-help" data-topic="privacy">${tr("ai.privacyLink")}</a> ·
    <a href="#" data-action="ai-help" data-topic="ref-mcp">${tr("ai.moreLink")}</a></p></div>`);
}
/** #263/AK4: «Ordner wählen …»: the folder dialog of the system, opened at the shown folder. */
async function aiFolder() {
  try {
    const dir = await Engine.call("pick_plan_folder", { start: aiSetup.dir });
    if (dir) { aiSetup.dir = dir; aiRender(); }
  } catch (e) { toast(errText(e)); }
}
document.addEventListener("change", e => {
  if (e.target?.id === "aiReadOnly") { aiSetup.readOnly = e.target.checked; aiRender(); }
});
/** Copies a setup line; the button confirms it, because a toast would lie behind the open dialog. */
function aiCopy(d, el) {
  const done = ok => { el.textContent = tr(ok ? "ai.copied" : "actions.copyFailed"); setTimeout(() => { el.textContent = tr("ai.copy"); }, 1600); };
  const text = aiLines[+d.copy];
  if (!navigator.clipboard || text === undefined) return done(false);
  navigator.clipboard.writeText(text).then(() => done(true), () => done(false));
}
async function showAbout() {
  let info = null, donate = null;
  try { info = await Engine.call("app_info"); } catch { /* without the engine no details */ }
  try { donate = (await linkUrl("donate"))?.replace(/^https:\/\//, ""); } catch { /* without the list no link */ }
  // #270/AK10: initials and day of the beta activation
  let beta = betaInfo;
  if (!beta) try { beta = await Engine.call("beta_status"); } catch { /* without the engine no activation shown */ }
  const betaDate = beta?.activated ? beta.activated.split("-").reverse().join(".") : "";
  const built = info?.buildEpoch ? new Date(info.buildEpoch * 1000).toLocaleString("de-CH", { dateStyle: "medium", timeStyle: "medium" }) : tr("about.unknownBuild");
  ask(tr("about.title"), `<p style="margin-top:0">${tr("about.tagline")}</p>
    <table class="tbl" id="aboutTbl"><tr><td class="l">${tr("about.version")}</td><td class="l"><b>${esc(info?.version ?? tr("about.unknownVersion"))}</b></td></tr>
    <tr><td class="l">${tr("about.built")}</td><td class="l"><b>${esc(built)}</b></td></tr>
    <tr><td class="l">${tr("about.website")}</td><td class="l">vivaplan-app.ch</td></tr>
    ${beta?.active && beta.who ? `<tr id="aboutBeta"><td class="l">${tr("about.beta")}</td><td class="l"><b>${tr("about.betaValue", { who: esc(beta.who), date: esc(betaDate) })}</b></td></tr>` : ""}
    ${donate ? `<tr id="aboutSupport"><td class="l">${tr("about.support")}</td><td class="l"><a href="#" data-action="link" data-link="donate" title="${esc(tr("nav.supportTitle"))}">${esc(donate)}</a></td></tr>` : ""}</table>
    <p class="hint">${tr("about.local")}</p>
    <p class="hint" id="aboutDisclaimer">${tr("about.disclaimer", { disclaimerFoot: esc(DISCLAIMER_FOOT) })}</p>`);
}
function gotoIndex(i) {
  if (!$("#results").hidden) { const b = [...document.querySelectorAll('[data-action="rtab"]')][i]; if (b) b.click(); return; }
  if (SECTIONS[i]) goto(SECTIONS[i].id);
}
document.addEventListener("keydown", e => {
  if (betaLocked) return;
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
    const r = await ask(tr("quit.title"), tr("quit.text"), null, tr("quit.save"), tr("quit.discard"));
    if (r === null) return;
    if (r === true) { await save(false); if (dirty) return; }
  }
  Engine.call("quit_app").catch(e => toast(errText(e)));
}
if (window.__TAURI__?.event) window.__TAURI__.event.listen("quit-requested", confirmQuit);
async function importData(cmd, title, help) {
  if (!Engine.inApp) return toast(tr("import.appOnly", { replace: cmd.replace("_", "-") }));
  if (!await ask(tr("import.title", { title }), `<p>${help}</p><p>${tr("import.continue")}</p>`)) return;
  try {
    const r = await Engine.call(cmd); if (!r) return;
    const lines = r.written.map(([f, i]) => `<p><b>${esc(f)}</b>: ${ymShow(i.first)} – ${ymShow(i.last)}${i.saronRange ? tr("import.saronUntil", { ymShow: ymShow(i.saronRange[1]) }) : ""}</p>`);
    await ask(tr("import.done"), [...lines, ...r.notes.map(n => `<p>${esc(msgText(n))}</p>`)].join("") || tr("import.nothingChanged"));
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
  const v = ["chf", "chfopt", "chfyear"].includes(t) || el.dataset.int ? parseAmount(el.value) : parseNum(el.value);
  if (v === null) return null;
  return t === "pct" ? v / 100 : t === "chfyear" ? v / 12 : v;
}
/** #274: a row of the deductions from income changed by hand is no longer the one of the tax bill. */
function dropTaxBill(path) {
  const m = /^(taxes\.deductions\.(\d+))\.(from|kind|amount|rate|extra)$/.exec(path);
  if (!m || getP(m[1])?.taxBill == null) return;
  delete getP(m[1]).taxBill;
  document.querySelector(`[data-ded-taxbill="${m[2]}"]`)?.remove();
}
function onEdit(el) {
  const path = el.dataset.path; if (!path) return;
  if (el.dataset.type === "point") { setPointWhen(getP(path), el.value); markDirty(); renderSection(); return; } // #243
  if (el.dataset.type === "carewhen") { setCareWhen(getP(path), el.value, +el.dataset.n); markDirty(); renderSection(); return; } // #271
  if (el.dataset.type === "ref") {
    const prev = getP(path);
    setP(path, el.value === "__date" ? i2ym(Number.isFinite(refMonth(prev)) && refMonth(prev) !== Infinity ? refMonth(prev) : startM()) : el.value);
    dropTaxBill(path); markDirty(); renderSection(); return;
  }
  const v = readInput(el);
  // #57: only text fields wait; checkboxes and radio buttons take effect immediately
  const typing = document.activeElement === el && el.matches("input:not([type=checkbox]):not([type=radio])");
  if (typing) holdField(el);
  if (v === null) {
    const ambiguous = (["chf", "chfopt", "chfyear"].includes(el.dataset.type) || el.dataset.int) && ambiguousThousands(el.value);
    invalidFields.set(path, { section: current, message: el.dataset.type === "month" ? tr("input.invalidDate") : ambiguous ? ambiguousText() : tr("input.invalidNumber") });
    if (!typing) { el.classList.add("invalid"); applyFieldIssues(); }
    return;
  }
  invalidFields.delete(path);
  if (!typing) el.classList.remove("invalid");
  const kindMs = /^milestones\.\d+\.kind$/.test(path) ? getP(path.replace(/\.kind$/, "")) : null;
  const resulting = kindMs ? msMonth(kindMs.id) : NaN; // point in time before the change (column «Ergibt»)
  const kindPay = /^pillar3a\.\d+\.payouts\.\d+\.kind$/.test(path) ? getP(path.replace(/\.kind$/, "")) : null;
  const payPid = kindPay ? state.pillar3a[+path.split(".")[1]].person : null, payAt = kindPay ? payoutMonth(kindPay, payPid) : NaN;
  setP(path, v); dropTaxBill(path); markDirty();
  if (el.dataset.type === "chfyear") { const pm = [...document.querySelectorAll("[data-permonth]")].find(x => x.dataset.permonth === path); if (pm) pm.textContent = perMonthText(v); }
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
  const t = el.dataset.type; if (["chf", "chfopt", "chfyear", "pct", "month"].includes(t)) el.value = showVal(t, getP(el.dataset.path));
});

/**
 * Empty plan for «Neu» (#269/AK1: built by the engine, `blank_plan`, the same plan as `plan_open` with `new: true` of
 * the AI server). Starts in the current month of `now`.
 */
const blankPlan = (now = new Date()) => Engine.call("blank_plan", { start: i2ym(now.getFullYear() * 12 + now.getMonth()) });
const ACTIONS = {
  goto: d => goto(d.sec),
  // #249/AK7: from the warning to the field «Steuerwert» of the property in the card «Abzüge vom Vermögen»
  "wtax-edit": d => {
    wtax.open = true; renderSection();
    const el = $(`#tax-wealth-deductions [data-wtax-field="${d.i}"] select`);
    if (el) { el.scrollIntoView({ block: "center" }); el.focus(); }
  },
  // #273/AK3: an existing plan takes over the default tax value for properties without an own tax value
  "tax-value-default": () => { state.realestate.defaultTaxValue = true; markDirty(); renderSection(); },
  "ms-goto": d => { flashId = d.id; goto("milestones"); flashId = null; },
  set: d => { setP(d.path, d.value); markDirty(); renderSection(); if (d.path === "returns.source") refreshSeriesInfo(); },
  // #100: follow the suggested values of the stock index again
  "param-suggest": () => { state.returns.parametric.auto = true; followSuggestion(); markDirty(); renderSection(); },
  "series-mode": d => {
    const s = seriesDef(d.path); if (s.mode === d.mode) return;
    setP(d.path, convertSeries(s, d.mode)); markDirty(); renderSection();
  },
  // #243: a further support point one year after the last one, with its value
  "point-add": d => {
    const pts = getP(d.path), last = pts[pts.length - 1];
    if (!last) pts.push({ at: "start", value: 0 });
    else if (last.age != null) pts.push({ ...last, age: Math.round((last.age + 1) * 100) / 100 });
    else { const m = pointMonth(last); pts.push({ at: i2ym((Number.isFinite(m) ? m : startM()) + 12), value: last.value }); }
    markDirty(); renderSection();
  },
  "row-add": d => { const rows = getP(d.path), last = rows[rows.length - 1]; rows.push({ from: i2ym(Math.max(startM(), refMonth(last?.from ?? "start") === Infinity ? startM() : refMonth(last?.from ?? "start")) + 12), value: last?.value ?? 0 }); markDirty(); renderSection(); },
  "row-del": d => {
    getP(d.path).splice(+d.i, 1);
    if (d.path === "taxes.deductions") { dedExtraShown.clear(); dedDetailsOpen.clear(); } // #241: indexes shift
    markDirty(); renderSection();
  },
  "list-del": d => {
    if (d.path === "realestate.properties") {
      const id = state.realestate.properties[+d.i].id;
      const users = state.realestate.properties.filter(q => q.buy === "afterSale:" + id || q.from === "afterSale:" + id);
      if (users.length) return toast(tr("actions.stillUsedBy", { map: users.map(q => q.name).join(", ") }));
    }
    getP(d.path).splice(+d.i, 1); markDirty(); renderSection();
  },
  // #271: care home of a person on or off (default: entry at 85, 9 000 per month, no sale)
  "care-toggle": d => {
    const p = state.household.persons[+d.n];
    if (p.careHome) delete p.careHome; else p.careHome = { age: 85, monthly: 9000, sell: false };
    markDirty(); renderSection();
  },
  "extra-add": () => { state.flows.extras.push({ date: i2ym(startM() + 12), amount: -10000, label: tr("actions.newAmount") }); markDirty(); renderSection(); },
  "tax-add": d => { const t = getP("taxes." + d.key), last = t[t.length - 1] || [0, 0]; t.push([last[0] * 2 || 100000, last[1]]); markDirty(); renderSection(); },
  "ms-add": () => { state.milestones.splice(state.milestones.length - 1, 0, { id: "ms" + Date.now().toString(36), name: tr("actions.newMilestone"), kind: "date", date: i2ym(startM() + 60) }); markDirty(); renderSection(); },
  "ms-del": async d => {
    const ms = state.milestones[+d.i];
    if (!ms || msLocked(ms)) return;
    const refs = milestoneRefs(ms.id);
    if (!refs.length) { state.milestones.splice(+d.i, 1); markDirty(); renderSection(); return; }
    const m = msMonth(ms.id);
    if (!Number.isFinite(m)) return toast(tr("actions.noValidDate", { name: msLabel(ms) }));
    const date = i2ym(m), sections = [...new Set(refs.map(r => r.section))];
    const ok = await ask(tr("actions.deleteMilestone"), `${tr("actions.usedIn", { name: esc(msLabel(ms)), length: refs.length })} ${refs.length === 1 ? tr("actions.input") : tr("actions.inputs")} ${tr("actions.usedInTail", { esc: sections.map(esc).join(", "), date: ymShow(date) })}`, null, tr("actions.deleteAndSwitch"));
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
    toast(`${refs.length} ${refs.length === 1 ? tr("actions.input") : tr("actions.inputs")} ${tr("actions.switchedTo", { date: ymShow(date) })}${dups ? ` ${tr("actions.duplicateDate", { dups })}` : ""}`);
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
    state.household.persons.push({ id, name: tr("general.person2"), birth: a.birth });
    state.partner = newPartner();
    // #250: new plans start unmarried; a couple starts married as before (the box can be cleared)
    state.household.married = true;
    const at = state.milestones.length - 1;
    state.milestones.splice(at, 0, { id: "retire2", kind: "age", person: id, years: 65 }, { id: "ahv2", kind: "age", person: id, years: 65 });
    markDirty(); renderSection();
  },
  "partner-del": () => {
    const b = state.household.persons[1], saved = state.partner;
    state.partner = null;
    const own = state.milestones.filter(m => m.kind === "age" && m.person === b.id);
    const used = own.find(m => countRefs(m.id));
    if (used) { state.partner = saved; return toast(tr("actions.usedReplaceFirst", { name: msLabel(used) })); }
    state.milestones = state.milestones.filter(m => !own.includes(m));
    // #250: the tables for married couples stay with «Verheiratet» (they also apply to a single person)
    state.household.persons.splice(1, 1); delete state.partner; if (!state.household.married) delete state.taxes.couple;
    state.pillar3a = state.pillar3a.filter(a => a.person !== b.id);
    markDirty(); renderSection();
  },
  // #272/AK6: switch a plan with own tables to the place of residence (undo like every change); the estimate of the
  // first year with the own tables stays for the hint «vorher, nachher» after the choice of the municipality
  "tax-to-location": () => {
    const before = taxEstimates.find(e => e.kind === "first");
    taxSwitch = before ? { amount: before.amount } : null;
    state.taxes.mode = "location"; state.taxes.location ??= { canton: "", church: "none" };
    // #131/AK4: in the mode «Wohnort» the proposal of the canton is the default (rows without an own value)
    for (const r of state.taxes.deductions ?? []) if (r.kind !== "canton" && !r.amount && !r.rate) { r.kind = "canton"; delete r.amount; delete r.rate; }
    taxOpen.settings = null;
    markDirty(); renderSection();
    $("#tax-muni")?.focus();
  },
  // #151/AK1: add a place of residence one year after the last row (canton and church of the last row, the
  // municipality is to be chosen); removing the last further row removes `relocations` from the file
  "loc-add": () => {
    const rows = locRows(), last = rows[rows.length - 1], lm = refMonth(last.from ?? "start");
    (state.taxes.relocations ??= []).push({ from: i2ym(Math.max(startM(), Number.isFinite(lm) ? lm : startM()) + 12), canton: last.loc.canton ?? "", church: last.loc.church ?? "none" });
    markDirty(); renderSection();
  },
  "loc-del": d => {
    state.taxes.relocations.splice(+d.i, 1);
    if (!state.taxes.relocations.length) delete state.taxes.relocations;
    markDirty(); renderSection();
  },
  // #131: store the shown rows to change them; add a row one year after the last one
  "ded-edit": () => { state.taxes.deductions = structuredClone(dedImplicit()); if (!state.taxes.deductions.length) state.taxes.deductions.push({ from: "start", kind: "own", amount: 0, rate: 0 }); markDirty(); renderSection(); },
  "ded-extra-show": d => { dedExtraShown.add(+d.i); renderSection(); },
  "ded-add": () => {
    const rows = state.taxes.deductions, last = rows[rows.length - 1], lm = refMonth(last?.from ?? "start");
    rows.push({ from: i2ym(Math.max(startM(), Number.isFinite(lm) ? lm : startM()) + 12), ...(isLocation(state) ? { kind: "canton" } : { kind: "own", amount: 0, rate: 0 }) });
    markDirty(); renderSection();
  },
  // #230/AK2: one-off deduction in the year after the start
  // #231/AK1: also for the deductions from wealth (`data-key` oneOffWealthDeductions)
  "ded-oneoff-add": d => {
    (state.taxes[d.key || "oneOffDeductions"] ??= []).push({ year: Math.floor(startM() / 12) + 1, label: tr("deductions.newOneOff"), amount: 0 });
    markDirty(); renderSection();
  },
  // #231/AK1: first row from the start, each further one a year after the previous one
  "wded-add": () => {
    const rows = (state.taxes.wealthDeductions ??= []), last = rows[rows.length - 1], lm = refMonth(last?.from ?? "start");
    rows.push({ from: last ? i2ym(Math.max(startM(), Number.isFinite(lm) ? lm : startM()) + 12) : "start", amount: 0 });
    markDirty(); renderSection();
  },
  "ded-calc-apply": d => {
    const v = dedCalcValue(), i = +($("#ded-calc-row")?.value ?? d.i), r = state.taxes.deductions?.[i];
    if (!Number.isFinite(v) || !r) return toast(tr("deductions.calcMissing"));
    Object.assign(r, { kind: "own", amount: Math.round(v), rate: r.rate ?? 0 }); delete r.taxBill; markDirty(); renderSection();
  },
  "taxbill-open": () => taxBillOpen(),
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
  "prop-add": () => { state.realestate.properties.push({ id: newPropId(), name: tr("actions.newProperty"), status: "planned", buy: "retire", sell: "never", price: 800000, fixCosts: 400, inflation: true, inflationValue: true, inflationFixCosts: true, inflationRent: true, mortgage: newMortgage() }); markDirty(); renderSection(); },
  "period-add": d => { const ps = state.realestate.properties[+d.i].mortgage.tranches[+d.t].periods, last = ps[ps.length - 1]; ps.push(newPeriod(last ? { ...last, term: 5 } : {})); markDirty(); renderSection(); },
  "tranche-add": d => {
    const p = state.realestate.properties[+d.i], ts = p.mortgage.tranches;
    ts.push({ name: tr("actions.trancheN", { v: ts.length + 1 }), amount: 100000, ...(p.status === "owned" ? { start: i2ym(startM()) } : {}), after: "renew", periods: [newPeriod()] });
    markDirty(); renderSection();
  },
  "sweep-add": () => { const sp = SWEEP_PARAMS.find(p => !state.scenarios.sweeps.some(s => s.param === p.id)) || SWEEP_PARAMS[0]; state.scenarios.sweeps.push({ param: sp.id, from: sp.def[0], to: sp.def[1], step: sp.def[2] }); markDirty(); renderSection(); },
  json: () => { $("#dlgPre").textContent = JSON.stringify(state, null, 2); $("#dlg").showModal(); },
  close: () => $("#dlg").close(),
  copy: () => { navigator.clipboard?.writeText(JSON.stringify(state, null, 2)).then(() => toast(tr("actions.copied")), () => toast(tr("actions.copyFailed"))); },
  download: () => download(),
  new: async () => {
    if (!await confirmDiscard()) return;
    await adopt(await blankPlan(), false); setFile(null); History.reset(); goto("general"); refreshSeriesInfo();
  },
  open: async () => {
    if (!Engine.inApp) return $("#fileInput").click();
    if (!await confirmDiscard()) return;
    try { await showOpened(await Engine.call("open_plan")); } catch (e) { toast(errText(e)); }
  },
  save: () => save(false),
  saveas: () => save(true),
  import: async () => {
    if (!Engine.inApp) return toast(tr("actions.importAppOnly"));
    if (!await confirmDiscard()) return;
    const y = await ask(tr("actions.importTitle"), tr("actions.importStartYear"), String(new Date().getFullYear()));
    if (!y) return;
    if (!/^\d{4}$/.test(y)) return toast(tr("actions.enterYear"));
    try {
      const r = await Engine.call("import_legacy", { startYear: +y }); if (!r) return;
      const notes = [...r.notes, ...await adopt(r.plan)]; setFile(null, true); History.reset(); goto("general"); refreshSeriesInfo();
      if (notes.length) await ask(tr("actions.importNotes"), notes.map(n => `<p>${esc(msgText(n))}</p>`).join(""));
    } catch (e) { toast(errText(e)); }
  },
  "import-shiller": () => importData("import_shiller", tr("actions.shiller"),
    tr("actions.shillerHint")),
  "import-snb": () => importData("import_snb", tr("actions.snb"),
    tr("actions.snbHint")),
  run: () => runSimulation(),
  help: () => command("help"),
  about: () => command("about"),
  // #237: external address from the fixed list (links.json), e.g. the donation link
  link: d => openLink(d.link),
  // #240: introductory video in the language set (data-video: 2min or 10min)
  video: d => openLink(videoLink(d.video)),
  // #61/AK4: from the dialog «Über VivaPlan» to the full disclaimer in the help
  "about-disclaimer": () => { $("#ask").close(); Help.show("disclaimer"); },
  back: () => showResults(false),
  detail: d => showDetail(+d.p),
  rtab: d => { resultTab = d.tab; renderResults(lastRun); },
  // #265: a chosen variation step of several runs needs the runs of the other order
  real: d => { realView = d.v === "1"; if (!compareOpen && variationSel && lastRun?.batch && lastRun.scenarios) pickBatchVariation(variationSel); else redrawResults(); },
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
  toast(tr("file.opened", { path: baseName(r.path) }));
  await showNotes(notes);
  await reserveNotice(state);
}
// #44: Double-click on a plan in the Finder while the app is running.
if (window.__TAURI__?.event) window.__TAURI__.event.listen("open-file", async ev => {
  if (betaLocked) return;
  if (!await confirmDiscard()) return;
  try { await showOpened(await Engine.call("open_path", { path: ev.payload })); } catch (e) { toast(errText(e)); }
});
/** #227: «Datei → Zuletzt geöffnet → ‹Planung›»: same question as «Öffnen …»; a file that no longer exists is reported
 *  (the app removes it from the list). */
async function openRecent(index) {
  if (!await confirmDiscard()) return;
  try {
    const r = await Engine.call("open_recent", { index });
    if (r?.missing) return toast(tr("file.recentMissing", { file: r.missing }));
    if (r?.opened) await showOpened(r.opened);
  } catch (e) { toast(errText(e)); }
}
/** #44: Window title «VivaPlan - ‹file name›», without a file just «VivaPlan». */
function showTitle(name) {
  document.title = name ? `VivaPlan - ${name}` : "VivaPlan";
  if (Engine.inApp) Engine.call("set_window_title", { path: currentPath }).catch(() => {});
}
/** File name in the header; without a file «Neue Planung» in the current language (#170). */
function showFileName(name) {
  const el = $("#fileName");
  if (name) { delete el.dataset.i18n; el.textContent = name; } else { el.dataset.i18n = "shortcuts.new"; el.textContent = tr("shortcuts.new"); }
}
/** #170: After a change of language everything is drawn again in the new language; plan, unsaved changes and the
 *  view (section, results tab, time range) stay as they are. */
function redrawTexts() {
  if (!state) return;
  $("#dirty").textContent = dirty ? tr("file.unsaved") : "";
  $("#disclaimerText").textContent = DISCLAIMER_SHORT;
  $("#nav").innerHTML = ""; // built again with the section titles by renderSection → refresh
  renderSection();
  Steps.render();
  if (!$("#results").hidden) {
    if (compareOpen) compareRuns();
    else if (lastRun) renderResults(lastRun);
    else openRuns();
  }
}
I18n.onChange(redrawTexts);
function setFile(path, isDirty = false) {
  currentPath = path; dirty = isDirty;
  showTitle(path ? baseName(path) : null);
  showFileName(path ? baseName(path) : null);
  $("#dirty").textContent = isDirty ? tr("file.unsaved") : "";
  pushDirty();
}
async function confirmDiscard() {
  return !dirty || !!await ask(tr("quit.title"), tr("file.discardText"));
}
async function save(as) {
  if (!Engine.inApp) { download(); setFile(currentPath); return; }
  try {
    const p = await Engine.call("save_plan", { plan: planForFile(state), path: as ? null : currentPath });
    if (!p) return;
    setFile(p); toast(tr("file.saved", { p: baseName(p) }));
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
  a.href = URL.createObjectURL(new Blob([JSON.stringify(planForFile(state), null, 2)], { type: "application/json" }));
  a.download = (state.project.name || tr("file.defaultName")).replace(/[^\w\-äöüÄÖÜ ]/g, "") + ".vplan"; a.click();
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
    if (typeof obj?.format === "string" && !KNOWN_FORMATS.includes(obj.format)) throw Object.assign(new Error(unknownFormatText(obj.format)), { unknownFormat: true });
    if (!KNOWN_FORMATS.includes(obj?.format)) throw 0;
    const notes = await adopt(obj); setFile(null); History.reset(); showFileName(f.name); showTitle(f.name); goto("general"); refreshSeriesInfo();
    toast(tr("file.opened", { path: f.name }));
    await showNotes(notes);
    await reserveNotice(state);
  } catch (err) { toast(err?.unknownFormat ? err.message : tr("file.notProject")); }
  e.target.value = "";
});
window.addEventListener("resize", () => { renderDerived(); if (lastRun) renderResults(lastRun); });

// ---------- Settings (#25) ----------
/** Default values of the settings; the engine knows the same ones (crates/engine/src/settings.rs). */
const LIMIT_DEFAULTS = { ahvMin: 63, ahvMax: 70, pkMin: 58, pkMax: 65, pillar3aMax: 7258, pillar3aMaxLarge: 36288 };
let LIMIT_FIELDS;
onLanguage(() => LIMIT_FIELDS = [["ahvMin", tr("limits.ahvMin"), tr("unit.years")], ["ahvMax", tr("limits.ahvMax"), tr("unit.years")],
  ["pkMin", tr("limits.pkMin"), tr("unit.years")], ["pkMax", tr("limits.pkMax"), tr("unit.years")],
  ["pillar3aMax", tr("limits.pillar3aMax"), null], ["pillar3aMaxLarge", tr("limits.pillar3aMaxLarge"), null]]);
/** Settings that are amounts (thousands separator, #140). */
const LIMIT_AMOUNTS = new Set(["pillar3aMax", "pillar3aMaxLarge"]);
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
  for (const [what, lo, hi] of [[tr("limits.ahv"), l.ahvMin, l.ahvMax], [tr("limits.pk"), l.pkMin, l.pkMax]]) {
    if (!whole(lo) || !whole(hi)) out.push(tr("limits.wholeYears", { what }));
    else if (lo > hi) out.push(tr("limits.lowAboveHigh", { what, lo, hi }));
  }
  if (!(Number.isFinite(l.pillar3aMax) && l.pillar3aMax >= 0)) out.push(tr("limits.pillar3aNegative"));
  if (!(Number.isFinite(l.pillar3aMaxLarge) && l.pillar3aMaxLarge >= 0)) out.push(tr("limits.pillar3aLargeNegative"));
  return out;
}
/** #233: Affordability of the banks (setting, before in the plan); the same defaults as the engine (`Affordability`). */
const AFFORD_DEFAULTS = { sustainability: 1 / 3, imputedRate: 0.05, fixCosts: 0.01, maxLoanToValue: 0.8, maxLoanToValueRetired: 0.65, wealthDrawdownRate: 0.03 };
/** #266: wealth drawdown in the affordability (switch, default off). */
const AFFORD_DRAWDOWN_DEFAULT = false;
/** No longer used by the engine (affordability only uses income); the plan needs it. */
const AFFORD_WEALTH_CONTRIBUTION = 0.03;
let AFFORD_FIELDS;
onLanguage(() => AFFORD_FIELDS = [["sustainability", tr("realestate.maxIncomeShare")], ["imputedRate", tr("realestate.imputedRate")],
  ["fixCosts", tr("realestate.costs")], ["maxLoanToValue", tr("realestate.maxLtv")], ["maxLoanToValueRetired", tr("realestate.maxLtvRetired")],
  ["wealthDrawdownRate", tr("realestate.drawdownRate")]]);
/** Applicable affordability: saved values, otherwise the defaults (shares as decimals). */
function affordability() {
  const saved = Pref.get("affordability", null), a = { ...AFFORD_DEFAULTS };
  if (saved && typeof saved === "object") for (const k of Object.keys(AFFORD_DEFAULTS)) if (Number.isFinite(saved[k])) a[k] = saved[k];
  const wealthDrawdown = typeof saved?.wealthDrawdown === "boolean" ? saved.wealthDrawdown : AFFORD_DRAWDOWN_DEFAULT;
  return { ...a, wealthDrawdown, wealthContribution: AFFORD_WEALTH_CONTRIBUTION };
}
Engine.affordability = affordability;
/** Validation as before in the editor: every value is a number. */
function affordProblems(a) {
  return AFFORD_FIELDS.filter(([k]) => !Number.isFinite(a[k])).map(([, what]) => tr("settings.affordInvalid", { what }));
}
/** Short text of the values («33.3 % des Einkommens, Zins 5 %, …»), one decimal at most. */
const affordPct = v => (Math.round(v * 1000) / 10).toLocaleString("de-CH", { maximumFractionDigits: 1 });
const affordText = a => tr("realestate.affordabilityValues", { income: affordPct(a.sustainability), rate: affordPct(a.imputedRate), costs: affordPct(a.fixCosts), ltv: affordPct(a.maxLoanToValue), ltvRetired: affordPct(a.maxLoanToValueRetired ?? AFFORD_DEFAULTS.maxLoanToValueRetired) })
  + (a.wealthDrawdown ? tr("realestate.affordabilityDrawdown", { drawdown: affordPct(a.wealthDrawdownRate ?? AFFORD_DEFAULTS.wealthDrawdownRate) }) : "");
/** Copy of the plan with the affordability it is calculated with (saved results, #233/AK5). */
function withAffordability(plan) {
  const p = JSON.parse(JSON.stringify(plan));
  p.realestate ??= {};
  p.realestate.affordability ??= affordability();
  return p;
}
/** #233/AK4: The plan file does not hold the affordability. */
function planForFile(plan) {
  const p = JSON.parse(JSON.stringify(plan));
  if (p.realestate) delete p.realestate.affordability;
  return p;
}
/**
 * #233/AK4: An older file with its own affordability: values that differ from the settings are taken over into the
 * settings. Returns the note for the dialog of the changes (adopted and previous values), otherwise null.
 */
function adoptAffordability(file) {
  if (!file || typeof file !== "object") return null;
  const before = affordability(), next = {};
  for (const k of Object.keys(AFFORD_DEFAULTS)) next[k] = Number.isFinite(file[k]) ? file[k] : before[k];
  if (Object.keys(next).every(k => Math.abs(next[k] - before[k]) < 1e-9)) return null;
  // #266: an older file has no switch for the wealth drawdown; the setting stays.
  next.wealthDrawdown = before.wealthDrawdown;
  Pref.set("affordability", next);
  return tr("notes.affordabilityAdopted", { values: affordText(next), before: affordText(before) });
}
/** #110: Question about saving before the simulation (default: on). */
const askSaveOn = () => Pref.get("askSaveBeforeRun", true) !== false;
function settingsFields(l, language = I18n.choice(), theme = Theme.choice(), af = affordability()) {
  const langs = [[I18n.SYSTEM, tr("language.system")], ...Object.entries(I18n.languages())];
  return `<label class="field" style="grid-column:1/-1"><span>${tr("settings.language")}</span><select id="set-language">${langs.map(([k, name]) => `<option value="${k}" ${k === language ? "selected" : ""}>${esc(name)}</option>`).join("")}</select></label>`
    + `<label class="field" style="grid-column:1/-1"><span>${tr("settings.theme")}</span><select id="set-theme" class="plain">${Theme.CHOICES.map(k => `<option value="${k}" ${k === theme ? "selected" : ""}>${tr("theme." + k)}</option>`).join("")}</select></label>`
    + `<label class="check" style="grid-column:1/-1"><input type="checkbox" id="set-askSave" ${askSaveOn() ? "checked" : ""}> ${tr("settings.askSave")}</label>`
    + LIMIT_FIELDS.map(([k, label, unit]) => `<label class="field"><span>${label}</span><div class="inp"><input class="num" id="set-${k}" value="${esc(LIMIT_AMOUNTS.has(k) ? fmtInt(l[k]) : String(l[k]))}"><span class="unit">${unit ?? cur()}</span></div><span class="computed">${tr("settings.default", { v: LIMIT_AMOUNTS.has(k) ? fmtInt(LIMIT_DEFAULTS[k]) : LIMIT_DEFAULTS[k] })}</span></label>`).join("")
    + `<div id="set-afford" style="grid-column:1/-1;margin-top:6px"><b>${tr("realestate.affordability")}</b> <span class="computed">${tr("realestate.forAfterSale")}</span></div>`
    + `<label class="check" style="grid-column:1/-1"><input type="checkbox" id="set-afford-wealthDrawdown" ${af.wealthDrawdown ? "checked" : ""}> ${tr("realestate.drawdown")}</label>`
    + AFFORD_FIELDS.map(([k, label]) => `<label class="field"><span>${label}</span><div class="inp"><input class="num" id="set-afford-${k}" value="${esc(fmtPct(af[k]))}"><span class="unit">%</span></div><span class="computed">${tr("settings.default", { v: `${affordPct(AFFORD_DEFAULTS[k])} %` })}</span></label>`).join("")
    + `<p class="hint" style="grid-column:1/-1">${tr("realestate.affordabilityHint")}</p>`;
}
function readSettings() {
  const l = {};
  // #140: The maximum amount of pillar 3a is an amount («7.258» is ambiguous)
  for (const [k] of LIMIT_FIELDS) { const t = $("#set-" + k).value, v = LIMIT_AMOUNTS.has(k) ? parseAmount(t) : parseNum(t); l[k] = v === null ? NaN : v; }
  return l;
}
/** #233: Affordability of the dialog «Einstellungen» (input in percent). */
function readAffordability() {
  const a = {};
  for (const [k] of AFFORD_FIELDS) { const v = parseNum($("#set-afford-" + k).value); a[k] = v === null ? NaN : v / 100; }
  a.wealthDrawdown = $("#set-afford-wealthDrawdown").checked;
  return a;
}
/** Dialog «Einstellungen»: applies to all plans, is stored locally, never in the plan file. */
function showSettings() {
  const d = $("#settings");
  $("#settingsBody").innerHTML = settingsFields(limits()); $("#settingsErr").textContent = "";
  d.showModal();
}
$("#settingsSave").addEventListener("click", e => {
  const l = readSettings(), af = readAffordability(), problems = [...limitProblems(l), ...affordProblems(af)];
  if (problems.length) { e.preventDefault(); $("#settingsErr").innerHTML = problems.map(esc).join("<br>"); return; }
  Pref.set("limits", l); Pref.set("affordability", af); Pref.set("askSaveBeforeRun", $("#set-askSave").checked);
  Pref.set("theme", $("#set-theme").value); Theme.apply();
  const language = $("#set-language").value;
  $("#settings").close();
  // #170: a new language draws everything again (also the section)
  if (language !== I18n.choice()) I18n.use(language).then(() => toast(tr("settings.saved")), e => noteError(errText(e)));
  else { toast(tr("settings.saved")); renderSection(); }
});
$("#settingsDefaults").addEventListener("click", async e => {
  e.preventDefault();
  if (!await ask(tr("settings.defaults"), tr("settings.resetAsk"), null, tr("settings.reset"))) return;
  Pref.set("limits", { ...LIMIT_DEFAULTS }); Pref.set("affordability", { ...AFFORD_DEFAULTS, wealthDrawdown: AFFORD_DRAWDOWN_DEFAULT }); Pref.set("askSaveBeforeRun", true); Pref.set("theme", "system"); Theme.apply();
  $("#settingsBody").innerHTML = settingsFields(limits(), I18n.SYSTEM, "system"); $("#settingsErr").textContent = "";
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
/**
 * #220: Theme of the user interface (setting «Darstellung»): "system" follows `prefers-color-scheme`, also when the
 * system theme changes while the app runs; "light" and "dark" apply always. Sets `data-theme` on the root element,
 * the CSS variables (and with them the charts) follow it.
 */
const Theme = {
  CHOICES: ["system", "light", "dark"],
  media: matchMedia("(prefers-color-scheme: dark)"),
  choice() { const c = Pref.get("theme", "system"); return this.CHOICES.includes(c) ? c : "system"; },
  effective(choice = this.choice()) { return choice === "system" ? (this.media.matches ? "dark" : "light") : choice; },
  apply() {
    const t = this.effective();
    if (document.documentElement.dataset.theme === t) return;
    document.documentElement.dataset.theme = t;
    if (state) { renderDerived(); if (lastRun) renderResults(lastRun); } // before the start nothing is drawn yet
  },
};
Theme.media.addEventListener("change", () => Theme.apply());
Theme.apply(); // system theme until the settings are loaded
let showTaxCols = false;
/** Years whose 3-bucket derivation is expanded in the yearly values. */
const bucketOpen = new Set();
let resultTab = "overview", realView = true, compPercent = false, detailP = 50, mktPct = false, mktLog = false, resultZoom = null, compareOpen = false;
function showResults(on) {
  if (!on) unpark();
  savedOpt = null;
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
  return !!await ask(tr("run.longTitle"), `${tr("run.longText", { esc: esc(fmtShort(sec)), runs: fmtInt(runs) })} ${runs === 1 ? tr("run.run") : tr("run.runs")}${tr("run.longTail")}`, null, tr("run.startAnyway"));
}
/**
 * #110: If the plan has unsaved changes, the app asks before the simulation whether to save.
 * Only in the desktop app (in the browser «Speichern» is a download); `askSaveDev` in the settings turns it on for tests.
 * `false` if the user cancels or saving does not succeed.
 */
async function confirmSaveBeforeRun() {
  if (!dirty || !askSaveOn() || !(Engine.inApp || Pref.get("askSaveDev", false))) return true;
  const r = await ask(tr("run.saveTitle"), `${tr("run.saveText")}${currentPath ? "" : tr("run.noFileName")}${tr("run.saveTail")}`, null, tr("quit.save"), tr("run.withoutSaving"));
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
  plainTitle(tr("run.running")); $("#resultsInfo").textContent = ""; $("#resultsTabs").innerHTML = "";
  $("#resultsBody").innerHTML = jobCard(tr("run.running"), tr("run.local"));
  try {
    const planAtRun = withAffordability(state);
    lastRun = await withJob(() => Engine.call("run_simulation", { plan: state, path: currentPath, jobId: job.id }));
    lastRunPlan = planAtRun;
    if (isCancelled(lastRun.error)) { lastRun = null; showResults(false); return toast(tr("run.cancelled")); }
    await derive(lastRun);
    renderResults(lastRun); document.documentElement.dataset.rendered = resultTab; rememberRun(lastRun);
    if (!lastRun.error) Steps.mark("run"); }
  catch (e) { plainTitle(tr("run.error")); $("#resultsBody").innerHTML = `<div class="card errcard">${esc(errText(e))}</div>`; }
}
const kpi = (k, v, s = "", cls = "") => `<div class="kpi ${cls}"><div class="k">${k}</div><div class="v">${v}</div><div class="s">${s}</div></div>`;
/** Legend; entry [color, text, opacity, line style]: with line style «solid» or «dashed» a line symbol instead of the box. */
const legend = items => `<div class="legend">${items.map(([c, t, o, line]) => `<span>${line
  ? `<svg class="lsym" width="22" height="10" viewBox="0 0 22 10" aria-hidden="true"><line x1="0" x2="22" y1="5" y2="5" stroke="var(${c})" stroke-width="2" ${line === "dashed" ? 'stroke-dasharray="5 3"' : ""}/></svg>`
  : `<i style="background:var(${c})${o ? `;opacity:${o}` : ""}"></i>`}${t}</span>`).join("")}</div>`;
let WEALTH_LEGEND;
onLanguage(() => WEALTH_LEGEND = legend([["--c1", tr("legend.liquid")], ["--c2", tr("legend.propertyNet")], ["--c3", tr("legend.pension")]]));
const atAge = month => tr("results.atAge", { month: ymShow(month), ageStr: ageStr(ym2i(month)) });
/** Does the plan have mortgages whose renewal is checked for affordability? */
const hasProperties = () => state.realestate.properties.some(p => p.status !== "rent");
/** Property sales of a run, one row «Name: MM.YYYY» per sale; emergency sales with warning icon, without a sale «–». */
function propertySales(res) {
  const sales = (res?.propertyEvents || []).filter(e => e.kind === "sale" || e.kind === "emergencySale").sort((a, b) => ym2i(a.month) - ym2i(b.month));
  return sales.length ? sales.map(e => `<div>${esc(e.property)}: ${ymShow(e.month)}${e.kind === "emergencySale" ? ` ${houseWarn(tr("timeline.emergency"))}` : ""}</div>`).join("") : "–";
}
/** Property events of a run, chronological: purchase, sale, emergency sale (with reason) and mortgage renewal with their amounts. */
function propertyEvents(res) {
  const evs = [...(res.propertyEvents || [])].sort((a, b) => ym2i(a.month) - ym2i(b.month));
  const n = v => fmtInt(v), part = (label, v) => `<span class="pe"><span class="k">${label}</span> ${v}</span>`;
  const TYPE = { purchase: tr("timeline.buy"), sale: tr("timeline.sell"), emergencySale: tr("timeline.emergency"), renewal: tr("propEvents.renewal") };
  const detail = e => {
    if (e.kind === "purchase") return [part(tr("propEvents.price"), n(e.price)), part(tr("propEvents.equity"), n(e.equity)), part(tr("propEvents.mortgage"), n(e.mortgage))];
    if (e.kind === "sale" || e.kind === "emergencySale") {
      const p = [part(tr("propEvents.proceeds"), n(e.proceeds)), part(tr("propEvents.mortgageRepaid"), n(e.debtRepaid))];
      if (e.gainTax > 0) p.push(part(tr("propEvents.gainTax"), fmtTax(e.gainTax))); // #143/AK2
      p.push(part(tr("propEvents.netProceeds"), `<b>${n(e.net)}</b>`));
      if (e.reason) p.push(part(tr("propEvents.reason"), `<span class="bad">${esc(msgText(e.reason))}</span>`));
      if (e.note) p.push(part(tr("propEvents.note"), esc(msgText(e.note))));
      return p;
    }
    const p = [part(tr("propEvents.affordability"), e.affordable ? tr("propEvents.given") : `<span class="bad">${tr("propEvents.notGiven")}</span>`)];
    if (e.reason) p.push(part(tr("propEvents.mortgage"), `<span class="bad">${tr("propEvents.notRenewed")}</span> ${tr("propEvents.emergencyReason", { debtBefore: n(e.debtBefore), reason: esc(msgText(e.reason)) })}`));
    else p.push(part(tr("propEvents.debtBeforeAfter"), `${n(e.debtBefore)} / ${n(e.debtAfter)}`));
    if (e.lumpSum > 0) p.push(part(tr("propEvents.oneOffAffordability"), `<b>${n(e.lumpSum)}</b>`));
    if (e.rate != null) p.push(part(tr("propEvents.rate"), `${(e.rate * 100).toLocaleString("de-CH", { maximumFractionDigits: 2 })} %`));
    return p;
  };
  const body = evs.length
    ? `<div class="tscroll"><table class="tbl"><tr><th class="l">${tr("propEvents.when")}</th><th class="l">${tr("propEvents.event")}</th><th class="l">${tr("propEvents.property")}</th><th class="l">${tr("propEvents.amountsIn", { cur: cur() })}</th></tr>
      ${evs.map(e => `<tr><td class="l">${atAge(e.month)}</td><td class="l ${e.kind === "emergencySale" ? "bad" : ""}">${TYPE[e.kind]}</td><td class="l">${esc(e.property)}</td><td class="l pes">${detail(e).join("")}</td></tr>`).join("")}</table></div>
      <p class="hint">${tr("propEvents.nominalHint")}${evs.some(e => e.gainTax > 0) ? ` ${tr("propEvents.gainTaxHint")} ${tr("taxes.checkBeforeDecision")}` : ""}</p>`
    : `<p class="hint" style="margin:0">${tr("propEvents.none")}</p>`;
  return card(tr("section.realestate"), tr("propEvents.subtitle"), body);
}
/**
 * 3-bucket strategy: derivation of the bucket sizes (spending, deducted income, net need, targets), return criterion,
 * triggered case (January, A, B, C), transfers and bucket balances before and after the refill (each before the spending of the month).
 * The refills of a year appear expandable in the yearly values (`yearTable`).
 */
function bucketDerivation(rs) {
  const n = v => fmtInt(v);
  const pct = v => `${(v * 100).toLocaleString("de-CH", { maximumFractionDigits: 1 })} %`;
  const CASE = { Januar: tr("buckets.januaryRefill"), A: tr("buckets.triggerA"), B: tr("buckets.triggerB"), C1: tr("buckets.triggerC1"), C2: tr("buckets.triggerC2") };
  const side = x => (x === 0 ? tr("buckets.external") : x === 4 ? tr("buckets.reservation") : tr("buckets.bucketN", { x }));
  const rows = rs.map(r => {
    const b = r.basis, inflow = r.trigger === "B";
    const cond = inflow ? "–" : r.conditionIgnored ? `<small>${tr("buckets.ignored")}</small>` : `${r.conditionMet ? tr("buckets.met") : `<span class="bad">${tr("buckets.notMet")}</span>`}<br><small>${pct(r.stockReturn)} (${esc(msgText(r.returnBasis))}) ${r.conditionMet ? "≥" : "<"} ${pct(r.threshold)}</small>`;
    const moves = r.moves?.length ? r.moves.map(x => `${side(x.from)} → ${side(x.to)}: ${n(x.amount)}`).join("<br>") : "–";
    const basis = inflow ? `<td colspan="8" class="l"><small>${tr("buckets.noRefill")}</small></td>`
      : `<td>${n(b.living)}${b.livingSource === "Guardrails" ? tr("buckets.guardrailsSmall") : ""}</td><td>${n(b.housing + (b.care ?? 0))}</td><td>${n(b.pillar3a ?? 0)}</td><td>${fmtTax(b.incomeTax)}</td><td>${n(b.ahv)}</td><td>${n(b.pk)}</td><td>${n(b.earned)}</td><td><b>${n(r.need)}</b></td>`;
    const targets = inflow ? "<td>–</td><td>–</td>" : `<td>${n(r.target1)}</td><td>${n(r.target2)}</td>`;
    const passed = (r.trigger === "A" || r.trigger === "C1") && r.passedMonths > 0 ? `<br><small>${tr("buckets.targetsShortened", { passedMonths: r.passedMonths })}</small>` : "";
    const base = inflow || !b.source ? "" : `<br><small>${esc(msgText(b.source))}</small>`;
    return `<tr><td class="l">${atAge(r.month)}</td><td class="l"><b>${esc(CASE[r.trigger] || r.trigger)}</b><br><small>${esc(msgText(r.reason))}</small>${base}${passed}</td>
      ${basis}${targets}<td class="l">${cond}</td>
      <td class="l">${moves}${r.payment > 0 && r.trigger !== "B" ? `<br><small>${tr("buckets.beforeSpending", { payment: n(r.payment) })}</small>` : ""}</td>
      <td>${n(r.before[0])} / ${n(r.before[1])} / ${n(r.before[2])}</td><td>${n(r.after[0])} / ${n(r.after[1])} / ${n(r.after[2])}</td></tr>`;
  }).join("");
  return `<div class="tscroll"><table class="tbl num"><tr><th class="l">${tr("buckets.when")}</th><th class="l">${tr("buckets.trigger")}</th>
      <th title="${tr("buckets.livingTitle")}">${tr("buckets.living")}</th><th>${tr("buckets.housing")}</th><th title="${tr("buckets.pillar3aTitle")}">3a</th><th title="${tr("buckets.incomeTaxTitle")}">${tr("buckets.incomeTax")}</th><th title="${tr("buckets.ahvTitle")}">${tr("common.ahv")}</th><th title="${tr("buckets.pkTitle")}">${tr("buckets.pkPension")}</th><th title="${tr("buckets.earnedTitle")}">${tr("buckets.earned")}</th><th title="${tr("buckets.netNeedTitle")}">${tr("buckets.netNeed")}</th>
      <th>${tr("buckets.target1")}</th><th>${tr("buckets.target2")}</th><th class="l" title="${tr("buckets.criterionTitle")}">${tr("buckets.criterion")}</th><th class="l" title="${tr("buckets.transfersTitle")}">${tr("buckets.transfers")}</th><th title="${tr("buckets.beforeTitle")}">${tr("buckets.before")}</th><th title="${tr("buckets.afterTitle")}">${tr("buckets.after")}</th></tr>${rows}</table></div>`;
}
const bucketHint = () => `<p class="hint">${tr("buckets.hint", { cur: cur() })}</p>`;
/** 3-bucket bookkeeping (refills, inflows, reservations) is in the yearly values; the messages of the deficit phase stay. */
const isBucketBooking = e => (e.kind === "buckets" && e.code !== "event_buckets_empty")
  || (state.withdrawal.strategy === "three_buckets" && e.kind === "withdrawal" && e.code === "event_sale_reserved");
function eventList(evs) {
  evs = evs.filter(e => !isBucketBooking(e));
  return evs.length ? `<ul class="evlist">${evs.map(e => `<li><span>${atAge(e.month)}</span><span>${esc(msgText(e))}</span></li>`).join("")}</ul>` : `<p class="hint" style="margin:0">${tr("results.noEvents")}</p>`;
}
/** Amount nominal or in prices of the start month. */
const rv = (r, x) => realView ? x / r.priceIndex : x;
const realNote = () => realView ? tr("results.real", { ymShow: ymShow(i2ym(startM())) }) : tr("results.nominalLower");
const xLabel = m => `${m / 12} · ${Math.round(ageAt(m))} ${tr("unit.yearsShort")}`;
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
  const note = custom ? tr("results.customZoom", { v: ((v[0] - startM()) / 12).toLocaleString("de-CH", { maximumFractionDigits: 1 }), v2: ((v[1] - startM()) / 12).toLocaleString("de-CH", { maximumFractionDigits: 1 }) }) : tr("results.zoomHint");
  return `<div class="minizoom">${zoomButtons("res-zoom", resultZoom ? (resultZoom.years ?? -1) : 0)}<span>${note}</span></div>`;
};
const chartBox = (id, h = 300, zoom = false) => `${zoom ? resultZoomBar() : ""}<div class="chartwrap"><svg id="${id}" class="bigchart" style="height:${h}px"></svg><div id="${id}-tip" class="tip"></div></div>`;
let FAN_LEGEND;
onLanguage(() => FAN_LEGEND = legend([["--c1", tr("fan.p5to95"), ".25"], ["--c1", tr("fan.p25to75"), ".5"], ["--c1", tr("fan.median"), null, "solid"], ["--warn", tr("fan.p1p99"), null, "dashed"], ["--err", tr("fan.worst"), null, "solid"]])
  + `<p class="hint">${tr("fan.worstHint")}</p>`);
const pName = p => p === 0 ? tr("fan.worstRun") : tr("fan.percentile", { p });
const pctTxt = v => `${(v * 100).toLocaleString("de-CH", { maximumFractionDigits: 1 })} %`;

/**
 * #209: Yearly values and digest come from the engine (`yearly`, `digest`); the user interface keeps them per run and
 * result and only formats them. `derive` fetches what is missing before a result is drawn, so that the drawing stays
 * synchronous. Rows: balances at year end, flows as the monthly average of the year, taxes as yearly sum, `flows` the
 * sums of the year for the bar charts.
 */
const DERIVED = new WeakMap();
/** Rows of the engine with the year as text (key of the expandable rows) and the withdrawal rate «–» without wealth at the start. */
const yearRowsOf = list => list.map(v => ({ ...v, y: String(v.year), rate: v.rate ?? (v.yearDraw > 0 ? NaN : null) }));
/** Yearly values and digest of runs (`SimResult`) that do not have them yet. */
const deriveRuns = runs => Promise.all([...new Set(runs.filter(run => run && !DERIVED.has(run)))].map(async run => {
  const [yearly, digest] = await Promise.all([Engine.call("yearly", { run }), Engine.call("digest", { result: { single: { summary: run.summary } } })]);
  DERIVED.set(run, { nominal: yearRowsOf(yearly.nominal), real: yearRowsOf(yearly.real), digest });
}));
/** Everything a result (answer of `run_simulation`, or of `rerun_paths` with `details`) needs for drawing. */
async function derive(result) {
  if (!result || result.error) return result;
  const runs = [result.single, ...(result.batch?.details || result.details || []).map(([, run]) => run)];
  await Promise.all([deriveRuns(runs), result.batch && !DERIVED.has(result)
    ? Engine.call("digest", { result: { batch: slimBatch(result.batch) } }).then(digest => DERIVED.set(result, { digest })) : null]);
  return result;
}
/** A saved result as the engine reads it for the digest and the comparison (without monthly values). */
const slimResult = res => ({ single: res?.single ? { summary: res.single.summary } : null, batch: res?.batch ? slimBatch(res.batch) : null });
/** What the engine reads of several runs for the digest and the comparison (without bands and monthly values). */
const slimBatch = b => ({ summary: b.summary, stats: { depletionAges: b.stats?.depletionAges || [] }, details: (b.details || []).map(([p, run]) => [p, { summary: run.summary }]) });
/** Yearly values of a run (nominal or in today's purchasing power depending on the toggle); empty until `derive` has them. */
const yearValues = run => DERIVED.get(run)?.[realView ? "real" : "nominal"] || [];
/** Sums per calendar year for the bar charts. */
const yearFlows = run => yearValues(run).map(v => ({ y: v.y, months: v.months, ...v.flows }));
/** Digest of a result or run (key figures and verdict of the engine). */
const digestOf = x => DERIVED.get(x)?.digest;
const rateTxt = r => r == null ? "" : Number.isNaN(r) ? "–" : `${(r * 100).toLocaleString("de-CH", { minimumFractionDigits: 1, maximumFractionDigits: 1 })} %`;
function yearTable(run, refills = []) {
  const cell = (v, bold) => `<td>${Math.abs(v) < 0.5 ? "" : bold ? `<b>${fmtInt(v)}</b>` : fmtInt(v)}</td>`;
  // #143/AK2: taxes rounded to 100 francs (an amount below 50 shows as empty)
  const taxCell = (v, bold) => cell(roundTax(v), bold);
  // #143/AK1: the headers of the tax columns say «estimated»; the tooltip explains the estimate and the rounding (AK6: check
  // lump-sum and property gains tax before deciding)
  const est = tr("yearTable.taxEstimatedTitle"), check = tr("taxes.checkBeforeDecision");
  const taxHeads = () => [[tr("yearTable.taxIncome"), `${tr("yearTable.taxTitle")} ${est}`], [tr("yearTable.taxWealth"), est], [tr("yearTable.taxLumpsum"), `${est} ${check}`],
    [tr("yearTable.taxProperty"), `${est} ${check}`], [tr("yearTable.taxTotal"), est]].map(([head, title]) => `<th class="tx" title="${title}">${head}</th>`).join("");
  const tx = showTaxCols, vals = yearValues(run), bx = tx && vals.some(v => v.basis), lx = tx && vals.some(v => v.residence);
  const px = tx && vals.some(v => v.basis?.propertyTaxValue != null), wx = tx && vals.some(v => v.basis?.wealth);
  const cx = vals.some(v => v.care); // #271: column of the care home costs only with a care home
  const yx = tx && vals.some(v => v.taxYield); // #149/AK6: column of the taxable yield only with a yield
  const cols = 12 + (cx ? 1 : 0) + (yx ? 1 : 0) + (tx ? 5 : 0) + (bx ? 3 : 0) + (px ? 1 : 0) + (wx ? 2 : 0) + (lx ? 1 : 0);
  // #231/AK5: deductions from wealth with their origin as tooltip and the taxable wealth
  const wealthHeads = () => [[tr("yearTable.wealthDeductions"), tr("yearTable.wealthDeductionsTitle")], [tr("yearTable.taxableWealth"), tr("yearTable.taxableWealthTitle")]]
    .map(([head, title]) => `<th class="tx" title="${title}">${head}</th>`).join("");
  const wealthCells = w => {
    if (!w) return "<td></td><td></td>";
    const origin = [[w.ongoing, tr("deductions.originExtra")], [w.oneOff, tr("deductions.originOneOff")]]
      .filter(([v]) => Math.abs(v) >= 0.5).map(([v, t]) => `${t}: ${fmtInt(v)}`).join("; ");
    return `<td title="${esc(origin || tr("deductions.originNone"))}" data-wded-origin="${esc(origin)}">${fmtInt(w.ongoing + w.oneOff)}</td><td title="${esc(`${tr("yearTable.wealthBefore")}: ${fmtInt(w.wealth)}`)}" data-taxable-wealth>${fmtInt(w.taxable)}</td>`;
  };
  // #131/AK14: tax breakdown: income, deductions (canton) with their origin as tooltip, taxable income
  const basisHeads = () => [[tr("yearTable.taxBasisIncome"), tr("yearTable.taxBasisIncomeTitle")], [tr("yearTable.taxBasisDeductions"), tr("yearTable.taxBasisDeductionsTitle")],
    [tr("yearTable.taxBasisTaxable"), tr("yearTable.taxBasisTaxableTitle")]].map(([head, title]) => `<th class="tx" title="${title}">${head}</th>`).join("");
  const basisCells = b => {
    if (!b) return "<td></td><td></td><td></td>";
    const fed = Math.abs(b.proposalFederal - b.proposalCanton) > 0.5 || Math.abs(b.taxableFederal - b.taxableCanton) > 0.5;
    // #274/AK7: own value and additional amount of rows taken over from the tax bill name it
    const bill = b.taxBill ? ` (${tr("taxBill.origin", { year: b.taxBill })})` : "";
    const origin = [[b.proposalCanton, tr("deductions.originCanton") + (fed ? ` (${tr("deductions.federal")} ${fmtInt(b.proposalFederal)})` : "")], [b.extra ?? 0, tr("deductions.originExtra") + bill], [b.own, tr("deductions.originOwn") + bill], [b.oneOff ?? 0, tr("deductions.originOneOff")], [b.pillar3a, tr("deductions.originPillar3a")],
      [b.careCanton ?? 0, tr("deductions.originCare") + (fed && Math.abs((b.careFederal ?? 0) - (b.careCanton ?? 0)) > 0.5 ? ` (${tr("deductions.federal")} ${fmtInt(b.careFederal)})` : "")]]
      .filter(([v]) => Math.abs(v) >= 0.5).map(([v, t]) => `${t}: ${fmtInt(v)}`).join("; ");
    const ded = b.proposalCanton + (b.extra ?? 0) + b.own + (b.oneOff ?? 0) + b.pillar3a + (b.careCanton ?? 0);
    return `${cell(b.income)}<td title="${esc(origin || tr("deductions.originNone"))}" data-ded-origin="${esc(origin)}">${fmtInt(ded)}</td><td title="${fed ? esc(`${tr("deductions.federal")}: ${fmtInt(b.taxableFederal)}`) : ""}">${fmtInt(b.taxableCanton)}</td>`;
  };
  const refillsOf = y => refills.filter(r => r.month.slice(0, 4) === y);
  const rows = vals.map(v => {
    const y = v.y, part = v.months < 12 ? `<span title="${tr("yearTable.partialYear", { months: v.months })}" style="color:var(--muted)">*</span>` : "";
    const rf = refillsOf(y), open = bucketOpen.has(y);
    const toggle = rf.length ? `<button class="btn ghost small bkbtn" data-action="bk-toggle" data-y="${y}" aria-expanded="${open}" title="${tr("yearTable.bucketTitle", { v: open ? tr("yearTable.bucketHide") : tr("yearTable.bucketShow") })}">${open ? "▾" : "▸"}</button>` : "";
    return `<tr><td>${toggle}${y}${part}</td><td>${v.age.toFixed(1)}</td><td>${fmtInt(v.liquid)}</td><td>${fmtInt(v.property)}</td><td>${fmtInt(v.pension)}</td><td><b>${fmtInt(v.total)}</b></td>
      ${cell(v.spending, true)}${cx ? `<td data-care>${Math.abs(v.care ?? 0) < 0.5 ? "" : fmtInt(v.care)}</td>` : ""}${cell(v.draw)}${cell(v.receipts, true)}<td>${rateTxt(v.rate)}</td>${cell(v.deviation)}<td>${(v.inflation * 100).toLocaleString("de-CH", { minimumFractionDigits: 1, maximumFractionDigits: 1 })} %</td>
      ${yx ? `<td data-tax-yield>${Math.abs(v.taxYield ?? 0) < 0.5 ? "" : fmtInt(v.taxYield)}</td>` : ""}${tx ? `${taxCell(v.taxIncome)}${taxCell(v.taxWealth)}${taxCell(v.taxCapital)}${taxCell(v.taxGain)}${taxCell(v.taxTotal, true)}` : ""}${bx ? basisCells(v.basis) : ""}${px ? `<td data-property-tax-value>${v.basis?.propertyTaxValue != null ? fmtInt(v.basis.propertyTaxValue) : ""}</td>` : ""}${wx ? wealthCells(v.basis?.wealth) : ""}${lx ? `<td class="l" data-residence>${v.residence ? esc(locationText(v.residence)) : ""}</td>` : ""}</tr>`
      + (rf.length && open ? `<tr class="bk"><td colspan="${cols}" class="l">${bucketDerivation(rf)}</td></tr>` : "");
  }).join("");
  const taxHint = tx ? ` ${tr("yearTable.taxHint")}` : "";
  return `<div class="seg" style="margin-bottom:8px"><button class="${tx ? "on" : ""}" data-action="tax-cols" aria-pressed="${tx}" title="${tr("yearTable.taxToggleTitle")}">${tx ? tr("yearTable.hideTaxes") : tr("yearTable.showTaxes")}</button></div>
    <div class="tscroll"><table class="tbl num yt"><tr><th>${tr("yearTable.year")}</th><th>${tr("yearTable.age")}</th><th title="${tr("yearTable.liquidTitle")}">${tr("yearTable.liquid")}</th><th title="${tr("yearTable.propertyTitle")}">${tr("yearTable.property")}</th><th title="${tr("yearTable.pensionTitle")}">${tr("yearTable.pension")}</th><th title="${tr("yearTable.totalTitle")}">${tr("yearTable.total")}</th>
      <th title="${tr("yearTable.spendingTitle")}">${tr("yearTable.spending")}</th>${cx ? `<th title="${tr("yearTable.careTitle")}">${tr("yearTable.care")}</th>` : ""}<th title="${tr("yearTable.withdrawalTitle")}">${tr("yearTable.withdrawal")}</th><th title="${tr("yearTable.incomeTitle")}">${tr("yearTable.income")}</th><th title="${tr("yearTable.rateTitle")}">${tr("yearTable.rate")}</th>
      <th title="${tr("yearTable.deviationTitle")}">${tr("yearTable.deviation")}</th><th title="${tr("yearTable.inflationTitle")}">${tr("yearTable.inflation")}</th>
      ${yx ? `<th class="tx" title="${tr("yearTable.taxYieldTitle")}">${tr("yearTable.taxYield")}</th>` : ""}${tx ? taxHeads() : ""}${bx ? basisHeads() : ""}${px ? `<th class="tx" title="${tr("yearTable.propertyTaxValueTitle")}">${tr("yearTable.propertyTaxValue")}</th>` : ""}${wx ? wealthHeads() : ""}${lx ? `<th class="tx" title="${tr("yearTable.residenceTitle")}">${tr("yearTable.residence")}</th>` : ""}</tr>${rows}</table></div>
    <p class="hint">${tr("yearTable.wealthHint", { taxHint })}</p><p class="hint taxline">${isLocation(lastRunPlan) ? tr("taxes.estimateLineLocation", { year: taxLoc?.year ?? "–" }) : tr("taxes.estimateLine")} ${helpLink("tax-accuracy", tr("taxes.accuracyLink"))}</p>${refills.length ? `<p class="hint">${tr("yearTable.bucketsHint")}</p>${bucketHint()}` : ""}`;
}
function singleKpis(s) {
  const dep = s.depleted ? ym2i(s.depleted) : null;
  return `<div class="rkpis">
    ${kpi(tr("kpi.wealthAtRetire"), s.totalAtRetirement != null ? fmtMio(s.totalAtRetirement) : "–", ymShow(i2ym(refMonth(state.pension.pk.at))))}
    ${kpi(tr("kpi.wealthAtEnd"), fmtMio(realView ? s.endTotalReal : s.endTotal), `${tr("single.ageAt", { ageStr: ageStr(endM()) })} ${realView ? tr("single.todayPower") : tr("results.nominalLower")}`, s.endTotal > 0 ? "good" : "bad")}
    ${kpi(tr("kpi.moneyLasts"), dep !== null ? tr("kpi.age") + ageStr(dep) : tr("kpi.endOk"), dep !== null ? ymShow(s.depleted) : tr("kpi.wholePeriod"), dep !== null ? "bad" : "good")}
    ${kpi(tr("single.pkPension"), fmtMoney(s.pensionFundMonthly), tr("single.pkSub", { lumpsumNet: fmtMio(s.lumpsumNet) }))}
    ${kpi(tr("single.taxesTotal"), fmtMio(s.taxesTotal), tr("single.taxesSub"))}</div>`;
}

/** Charts of a single run (single run or percentile run). */
function runChartsHtml(res, p) {
  const recs = res.records, hasProp = recs.some(r => r.propertyValue > 0), hasPk = recs.some(r => r.pensionFund > 0 || r.pillar3a > 0);
  return card(tr("runCharts.wealth"), realNote(), chartBox(`${p}-wealth`, 300, true) + WEALTH_LEGEND)
    + card(tr("runCharts.liquidByAsset"), realNote(), `<div class="seg" style="margin-bottom:8px"><button class="${compPercent ? "" : "on"}" data-action="comp" data-v="0">${tr("runCharts.amounts")}</button><button class="${compPercent ? "on" : ""}" data-action="comp" data-v="1">${tr("runCharts.shares")}</button></div>`
      + chartBox(`${p}-comp`, 300, true) + legend([["--c1", tr("wealth.stocks")], ["--c2", tr("wealth.bonds")], ["--c3", tr("wealth.cash")]]))
    + card(tr("runCharts.flows"), realNote(), chartBox(`${p}-flows`, 300, true)
      + legend([["--c2", tr("runCharts.earned")], ["--c1", tr("single.pkPension")], ["--c4", tr("common.ahv")], ["--c3", tr("runCharts.living")], ["--warn", tr("runCharts.housing")],
        ...(recs.some(r => r.care) ? [["--accent", tr("runCharts.care")]] : []), ["--err", tr("runCharts.taxesEstimated")]])
      + `<p class="hint">${tr("runCharts.flowsHint")}</p>`)
    + card(tr("runCharts.withdrawal"), tr("runCharts.withdrawalSub"), chartBox(`${p}-draw`, 260, true)
      + `<p class="hint">${tr("runCharts.withdrawalHint")}</p>`)
    + (hasProp ? card(tr("section.realestate"), realNote(), chartBox(`${p}-prop`, 260, true) + legend([["--c2", tr("runCharts.marketValue")], ["--err", tr("runCharts.mortgage")], ["--c1", tr("runCharts.equity")]])) : "")
    + (hasPk ? card(tr("section.pension"), realNote(), chartBox(`${p}-pk`, 240, true) + legend([["--c3", tr("runCharts.pk")], ["--c2", tr("section.pillar3a")]])) : "")
    + card(tr("runCharts.returns"), `${mktPct ? tr("runCharts.change") : tr("runCharts.index")}, ${realNote()}`,
      mktSwitch()
      + chartBox(`${p}-mkt`, 240, true)
      + legend([["--c1", tr("wealth.stocks")], ["--c2", tr("wealth.bonds")], ["--muted", tr("runCharts.inflationIndex")]])
      + `<p class="hint">${mktPct ? tr("runCharts.changeHint") : tr("runCharts.indexHint")}${tr("runCharts.tooltipHint")}</p>`);
}
function drawRunCharts(res, p) {
  const recs = res.records, x = recs.map(r => ym2i(r.month));
  const tipHead = i => `${tr("runTip.head", { ymShow: ymShow(recs[i].month), ageStr: ageStr(x[i]) })}<br>`;
  const marks = res.events.filter(e => ["retirement", "ahvStart", "sale", "emergencySale", "purchase", "depleted"].includes(e.kind)).map(e => ({ x: ym2i(e.month), label: `${atAge(e.month)}: ${msgText(e)}`, color: e.kind === "depleted" || e.kind === "emergencySale" ? "--err" : "--muted" }));
  const w = k => recs.map(r => rv(r, k(r)));
  Charts.time($(`#${p}-wealth`), $(`#${p}-wealth-tip`), { x, xFmt: xLabel, view: resultView(), onZoom: zoomAbs, yFmt: fmtMio, marks,
    stack: [{ color: "--c3", values: w(r => Math.max(0, r.pensionFund + r.pillar3a)) }, { color: "--c2", values: w(r => Math.max(0, r.propertyEquity)) }, { color: "--c1", values: w(r => Math.max(0, r.liquid)) }],
    tip: i => tipHead(i) + `${tr("runTip.total", { money: fmtMoney(rv(recs[i], recs[i].total)) })}<br><span style="color:var(--c1)">■</span> ${tr("runTip.liquid", { money: fmtMoney(rv(recs[i], recs[i].liquid)) })}<br><span style="color:var(--c2)">■</span> ${tr("runTip.property", { money: fmtMoney(rv(recs[i], recs[i].propertyEquity)) })}<br><span style="color:var(--c3)">■</span> ${tr("runTip.pension", { money: fmtMoney(rv(recs[i], recs[i].pensionFund + recs[i].pillar3a)) })}` });
  Charts.time($(`#${p}-comp`), $(`#${p}-comp-tip`), { x, xFmt: xLabel, view: resultView(), onZoom: zoomAbs, yFmt: fmtMio, percent: compPercent,
    stack: [{ color: "--c3", values: w(r => r.cash) }, { color: "--c2", values: w(r => r.bonds) }, { color: "--c1", values: w(r => r.stocks) }],
    tip: i => { const r = recs[i], t = Math.max(1, r.cash + r.bonds + r.stocks), q = v => compPercent ? `${(Math.max(0, v) / t * 100).toFixed(1)} %` : fmtMoney(rv(r, v));
      return tipHead(i) + `<span style="color:var(--c1)">■</span> ${tr("runTip.stocks", { stocks: q(r.stocks) })}<br><span style="color:var(--c2)">■</span> ${tr("runTip.bonds", { bonds: q(r.bonds) })}<br><span style="color:var(--c3)">■</span> ${tr("runTip.cash", { cash: q(r.cash) })}`; } });
  // Bar charts: only the years that touch the zoom range
  const view = resultView(), allYears = yearFlows(res);
  const ys = view ? allYears.filter(a => +a.y * 12 + 11 >= view[0] && +a.y * 12 <= view[1]) : allYears;
  Charts.bars($(`#${p}-flows`), $(`#${p}-flows-tip`), { cats: ys.map(a => a.y), onSelect: (a, b) => setCustomZoom(+ys[a].y * 12 - startM(), +ys[b].y * 12 + 11 - startM()), yFmt: fmtMio,
    series: [{ group: "in", color: "--c2", values: ys.map(a => a.earned) }, { group: "in", color: "--c1", values: ys.map(a => a.pk) }, { group: "in", color: "--c4", values: ys.map(a => a.ahv) },
      { group: "out", color: "--c3", values: ys.map(a => a.living) }, { group: "out", color: "--warn", values: ys.map(a => a.housing) },
      ...(ys.some(a => a.care) ? [{ group: "out", color: "--accent", values: ys.map(a => a.care ?? 0) }] : []), { group: "out", color: "--c4", values: ys.map(a => a.p3a) }, { group: "out", color: "--err", values: ys.map(a => a.taxes) }],
    tip: i => { const a = ys[i]; return `${tr("runTip.flows", { y: a.y, months: a.months, money: fmtMoney(a.income), earned: fmtMoney(a.earned), pk: fmtMoney(a.pk), ahv: fmtMoney(a.ahv), money2: fmtMoney(a.spending), living: fmtMoney(a.living), housing: fmtMoney(a.housing) })}<br>${a.p3a ? `${tr("runTip.pillar3a", { p3a: fmtMoney(a.p3a) })}<br>` : ""}${a.care ? `${tr("runTip.care", { care: fmtMoney(a.care) })}<br>` : ""}${tr("runTip.taxes", { taxes: `${fmtTax(a.taxes)} ${cur()}` })}`; } });
  const rate = ys.map(a => a.rate);
  Charts.bars($(`#${p}-draw`), $(`#${p}-draw-tip`), { cats: ys.map(a => a.y), onSelect: (a, b) => setCustomZoom(+ys[a].y * 12 - startM(), +ys[b].y * 12 + 11 - startM()), yFmt: v => `${(v * 100).toFixed(1)} %`,
    series: [{ color: "--accent", values: rate }],
    tip: i => tr("runTip.withdrawal", { v: ys[i].y, money: fmtMoney(Math.max(0, ys[i].draw)), money2: fmtMoney(ys[i].liquidStart), pctTxt: pctTxt(rate[i]) }) });
  Charts.time($(`#${p}-prop`), $(`#${p}-prop-tip`), { x, xFmt: xLabel, view: resultView(), onZoom: zoomAbs, yFmt: fmtMio, marks,
    lines: [{ color: "--c2", values: w(r => r.propertyValue) }, { color: "--err", values: w(r => r.mortgage) }, { color: "--c1", values: w(r => r.propertyEquity), dash: "5 3" }],
    tip: i => tipHead(i) + tr("runTip.property2", { money: fmtMoney(rv(recs[i], recs[i].propertyValue)), money2: fmtMoney(rv(recs[i], recs[i].mortgage)), money3: fmtMoney(rv(recs[i], recs[i].propertyEquity)) }) });
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
    tip: i => tipHead(i) + (mktPct ? "" : `<span style="color:var(--c1)">■</span> ${tr("runTip.stocksIdx", { idx: idx("stockIndex", i).toFixed(1), stockTxt: stockTxt("stockIndex", i) })}<br><span style="color:var(--c2)">■</span> ${tr("runTip.bondsIdx", { idx: idx("bondIndex", i).toFixed(1), stockTxt: stockTxt("bondIndex", i) })}<br><span style="color:var(--muted)">■</span> ${tr("runTip.priceIdx", { i: priceIdx(i).toFixed(1), i2: inflTxt(i) })}`)
      + (mktPct ? `<span style="color:var(--c1)">■</span> ${tr("runTip.stocksChange", { stockTxt: stockTxt("stockIndex", i) })}<br><span style="color:var(--c2)">■</span> ${tr("runTip.bondsChange", { stockTxt: stockTxt("bondIndex", i) })}<br><span style="color:var(--muted)">■</span> ${tr("runTip.inflation", { i: inflTxt(i) })}` : "") });
  Charts.time($(`#${p}-pk`), $(`#${p}-pk-tip`), { x, xFmt: xLabel, view: resultView(), onZoom: zoomAbs, yFmt: fmtMio,
    lines: [{ color: "--c3", values: w(r => r.pensionFund) }, { color: "--c2", values: w(r => r.pillar3a) }],
    tip: i => tipHead(i) + tr("runTip.pension2", { money: fmtMoney(rv(recs[i], recs[i].pensionFund)), money2: fmtMoney(rv(recs[i], recs[i].pillar3a)) }) });
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
  ? `<p class="hint survclip">${tr("batch.survClip", { survLo: SURV_LO, toLocaleString: min.toLocaleString("de-CH", { maximumFractionDigits: 1 }) })}</p>` : "";
/** #122: Switches of the chart «Aktien, Obligationen und Inflation», the same in the single result and in the comparison. */
const mktSwitch = () => `<div class="seg" style="margin-bottom:8px"><button class="${mktPct ? "" : "on"}" data-action="mkt" data-v="idx">${tr("batch.index")}</button><button class="${mktPct ? "on" : ""}" data-action="mkt" data-v="pct">${tr("batch.change")}</button></div>`
  + (mktPct ? "" : ` <div class="seg" style="margin-bottom:8px"><button class="${mktLog ? "" : "on"}" data-action="mkt-log" data-v="0">${tr("batch.linear")}</button><button class="${mktLog ? "on" : ""}" data-action="mkt-log" data-v="1">${tr("batch.log")}</button></div>`);
function batchChartsHtml(b) {
  const st = b.stats;
  return card(tr("batch.band"), tr("batch.overRuns", { realNote: realNote() }), chartBox("mc-fan", 300, true)
      + FAN_LEGEND)
    + card(tr("batch.survival"), tr("batch.survivalSub"), survSwitch() + chartBox("mc-surv", 260, true) + survClipNote(Math.min(...st.survival) * 100))
    + card(tr("batch.endDist"), realNote(), chartBox("mc-end", 260) + legend([["--text", tr("batch.endDistSub"), null, "dashed"]]))
    + (st.atRetirement ? card(tr("batch.retireDist"), tr("results.nominalLower"), chartBox("mc-ret", 240)) : "")
    + (st.depletionAges.length ? card(tr("batch.depletionAge"), tr("batch.depletionSub"), chartBox("mc-dep", 220)) : "");
}
function drawBatchCharts(b) {
  const bands = realView && b.bands.real ? b.bands.real : b.bands, x = bands.months.map(ym2i), st = b.stats;
  Charts.time($("#mc-fan"), $("#mc-fan-tip"), { x, xFmt: xLabel, view: resultView(), onZoom: zoomAbs, yFmt: fmtMio,
    bands: [{ lo: bands.p5, hi: bands.p95, color: "--c1", opacity: 0.18 }, { lo: bands.p25, hi: bands.p75, color: "--c1", opacity: 0.35 }],
    lines: [{ color: "--c1", values: bands.p50 }, { color: "--warn", dash: "5 3", width: 1.5, values: bands.p99 }, { color: "--warn", dash: "5 3", width: 1.5, values: bands.p1 }, { color: "--err", width: 1.5, values: bands.min }],
    tip: i => `${tr("batch.bandTip", { ymShow: ymShow(bands.months[i]), ageStr: ageStr(x[i]), money: fmtMoney(bands.p99[i]), money2: fmtMoney(bands.p95[i]), money3: fmtMoney(bands.p75[i]), money4: fmtMoney(bands.p50[i]), money5: fmtMoney(bands.p25[i]), money6: fmtMoney(bands.p5[i]), money7: fmtMoney(bands.p1[i]) })}<br><span style="color:var(--err)">${tr("batch.worstTip", { money: fmtMoney(bands.min[i]) })}</span>` });
  const surv = st.survival.map(v => v * 100);
  Charts.time($("#mc-surv"), $("#mc-surv-tip"), { x, xFmt: xLabel, view: resultView(), onZoom: zoomAbs, yFmt: v => `${v} %`, range: survRange(),
    lines: [{ color: "--ok", values: surv }],
    tip: i => tr("batch.survivalTip", { ymShow: ymShow(bands.months[i]), ageStr: ageStr(x[i]), v: surv[i].toLocaleString("de-CH", { maximumFractionDigits: 1 }) }) });
  const h = realView ? st.endTotalReal : st.endTotal;
  const marks = [[0.01, "1."], [0.05, "5."], [0.5, tr("fan.median")], [0.95, "95."], [0.99, "99."]].map(([q, l]) => ({ value: histQuantile(h, q), label: l }));
  Charts.histogram($("#mc-end"), $("#mc-end-tip"), h, { xFmt: fmtMio, marks });
  Charts.histogram($("#mc-ret"), $("#mc-ret-tip"), st.atRetirement, { xFmt: fmtMio, color: "--c2", marks: [{ value: histQuantile(st.atRetirement, 0.5), label: tr("fan.median") }] });
  const runs = b.summary.runs;
  Charts.bars($("#mc-dep"), $("#mc-dep-tip"), { cats: st.depletionAges.map(([a]) => String(a)), yFmt: v => fmtInt(v),
    series: [{ color: "--err", values: st.depletionAges.map(([, n]) => n) }],
    tip: i => tr("batch.depletionTip", { v: st.depletionAges[i][0], int: fmtInt(st.depletionAges[i][1]), pctTxt: pctTxt(st.depletionAges[i][1] / runs) }) });
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
const randomPath = label => ["path_bootstrap", "path_random"].includes(label?.code);
const runInfo = label => randomPath(label) ? variationText() : msgText(label);
/** Optimal value of a variation for a percentile, readable: «73 %» (step), «≈ 73 %» (interpolated), «< 50 %», «≥ 110 %». */
function optimumValueText(item) {
  if (item.kind === "na") return "–";
  const t = sweepType(item.param);
  const v = t === "pct" && typeof item.v === "number" ? Math.round(item.v * 1000) / 1000 : item.v;
  const s = SWEEP_SHOW[t](v);
  const pre = { exact: "", approx: "≈ ", below: "< ", atLeast: "≥ ", edgeLow: "≤ ", edgeHigh: "≥ " }[item.kind] ?? "";
  return pre + s + (item.reaches ? "" : tr("optimum.nowhere"));
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
/** #265: the runs of a step depend on the order of the display (real or nominal). */
const batchKey = sel => selKey(sel) + (realView ? "|real" : "");
const batchRuns = () => variationSel && variationRuns.get(batchKey(variationSel));
const detailsOf = r => batchRuns()?.details || batchDetails(r.batch);
/**
 * #265: Percentiles of several runs and their runs in the order of the display: nominal by the nominal end wealth, in
 * today's purchasing power by the real end wealth. Results saved before #265 have only the nominal order (`oldOrder`):
 * in today's purchasing power they show the real end wealth of the runs of the nominal order, with a notice.
 */
const realOrder = b => realView && !!b.summary.percentilesReal?.length;
const oldOrder = b => realView && !b.summary.percentilesReal?.length;
const pctsOf = b => realOrder(b) ? b.summary.percentilesReal : b.summary.percentiles;
const batchDetails = b => realOrder(b) && b.detailsReal?.length ? b.detailsReal : b.details;
/** End wealth of a percentile in the display; results before #126 take the real value from the detail run. */
const pctEnd = (b, p) => !realView ? p.endTotal
  : p.endTotalReal ?? b.details?.find(([q]) => q === p.percent)?.[1].summary.endTotalReal ?? p.endTotal;
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
      <option value="plan" ${cur == null ? "selected" : ""}>${tr("variation.planValue")}${now != null ? ` (${SWEEP_SHOW[t](now)})` : ""}</option>
      ${vals.map(v => `<option value="${esc(JSON.stringify(v))}" ${JSON.stringify(v) === JSON.stringify(cur) ? "selected" : ""}>${SWEEP_SHOW[t](v)}</option>`).join("")}</select></label>`;
  }).join("");
  const opt = optimumText(sc, 50);
  const sub = r.single ? tr("variation.singleHint")
    : tr("variation.batchHint");
  return card(tr("variation.choose"), sub,
    `<div class="grid3">${selects}</div>
    <p class="computed" style="margin:8px 0 0">${opt ? `${tr("variation.optimum", { opt: esc(opt) })} ` : ""}${o && o.items.every(it => it.grid != null) ? `<button class="btn small" data-action="var-opt" title="${tr("variation.showOptimumTitle")}">${tr("variation.showOptimum")}</button>` : ""}
    ${variationSel ? `<button class="btn small" data-action="var-plan" title="${tr("variation.backTitle")}">${tr("variation.planValues")}</button>` : ""}</p>`);
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
  if (variationSel && !variationRuns.has(batchKey(variationSel))) {
    const level = levelOf(sc, variationSel);
    if (!level || !level.percentiles?.length) {
      variationSel = null;
      toast(tr("variation.notComputed"));
      return renderResults(lastRun);
    }
    const real = realView && level.percentilesReal?.length > 0, pcts = real ? level.percentilesReal : level.percentiles;
    const picks = pcts.filter(p => [0, 1, 5, 50, 95, 99].includes(p.percent)).map(p => [p.percent, p.index]);
    const plan = JSON.parse(JSON.stringify(state));
    plan.scenarios = { sweeps: [], select: sc.params.map((p, i) => ({ param: p, value: variationSel[i] ?? sweepNow(p) })) };
    let out;
    try { out = await Engine.call("rerun_paths", { plan, picks, path: currentPath }); } catch (e) { out = { error: errText(e) }; }
    if (out.error) { variationSel = null; toast(tr("variation.batchFailed") + esc(errText(out.error))); return renderResults(lastRun); }
    await derive(out);
    // If a final wealth differs from the saved value (e.g. changed time series), the display stays available, with a notice
    const off = out.details.filter(([pc, res]) => {
      const e = pcts.find(p => p.percent === pc)?.endTotal;
      return e != null && Math.abs(res.summary.endTotal - e) > 1e-6 * Math.max(1, Math.abs(e));
    }).map(([pc]) => pc);
    variationRuns.set(batchKey(variationSel), { details: out.details, off });
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
    if (out.error || !out.single) { variationSel = null; toast(tr("variation.failed") + (out.error ? errText(out.error) : tr("variation.noResult"))); }
    else { await derive(out); variationRuns.set(selKey(variationSel), out.single); }
  }
  renderResults(lastRun);
}
document.addEventListener("change", e => {
  if (!e.target.matches?.('select[data-action="var-sel"]')) return;
  pickVariation([...document.querySelectorAll('select[data-action="var-sel"]')].map(el => (el.value === "plan" ? null : JSON.parse(el.value))));
});
/** Row for a percentile: optimal variation values, for historical runs additionally the start month. */
const percentInfo = (r, percent, label) => [optimumText(r.scenarios, percent), randomPath(label) ? "" : msgText(label)].filter(Boolean).join(" · ");
function scenarioTable(sc) {
  const types = sc.params.map(sweepType);
  const best = Math.max(...sc.results.map(r => r.error ? -1 : r.successRate));
  const single = sc.results.every(r => r.runs <= 1);
  const rows = sc.results.map(r => {
    const vals = r.values.map((v, i) => `<td class="l">${SWEEP_SHOW[types[i]](v)}</td>`).join("");
    if (r.error) return `<tr>${vals}<td class="l" colspan="6" style="color:var(--err)">${esc(errText(r.error))}</td></tr>`;
    const ok = r.successRate, cls = ok >= 0.95 ? "good" : ok < 0.8 ? "bad" : "";
    return `<tr class="${r.successRate === best ? "best" : ""}">${vals}
      <td class="${cls}">${single ? (ok ? "✓" : "✗") : pctTxt(ok)}</td>
      <td>${r.atRetirement != null ? fmtInt(r.atRetirement) : "–"}</td>
      ${single ? `<td colspan="3">${fmtInt(realView ? r.p50Real : r.p50)}</td>` : `<td>${fmtInt(r.p5)}</td><td><b>${fmtInt(realView ? r.p50Real : r.p50)}</b></td><td>${fmtInt(r.p95)}</td>`}
      <td>${r.earliestDepletion ? tr("kpi.ageAt", { minLiquidM: ageStr(ym2i(r.earliestDepletion)) }) : "–"}</td></tr>`;
  }).join("");
  const heads = sc.params.map(p => `<th class="l">${esc(sweepLabel(p))}</th>`).join("");
  return card(tr("scen.title"), `${tr("scen.combos", { length: fmtInt(sc.results.length), int: fmtInt(sc.results[0]?.runs || 0) })} ${single ? tr("computed.run") : tr("computed.runs")}`,
    `<div class="tscroll"><table class="tbl num sctbl"><tr>${heads}<th>${single ? tr("scen.enough") : tr("scen.successRate")}</th><th>${tr("kpi.wealthAtRetire")}${single ? "" : tr("scen.median")}</th>${single ? `<th colspan="3">${tr("scen.endWealth")}</th>` : `<th>${tr("scen.endP5")}</th><th>${tr("scen.endMedian")}${realView ? tr("scen.todayPower") : ""}</th><th>${tr("scen.endP95")}</th>`}<th>${tr("scen.earliestDepletion")}</th></tr>${rows}</table></div>
    <p class="hint">${tr("scen.nominalIn", { cur: cur() })}${realView ? tr("scen.medianReal") : ""}${tr("scen.footer")}</p>`);
}
const sweepNames = sc => sc.params.map(p => esc(sweepLabel(p))).join(" × ");
function scenarioChartsHtml(sc) {
  const ok = sc.results.filter(r => !r.error);
  if (!ok.length) return "";
  const variation = `<div class="card"><p style="margin:0"><b>${tr("scen.shown")} ${sc.params.length === 1 ? tr("scen.variation") : tr("scen.variations")}:</b> ${sweepNames(sc)}</p></div>`;
  if (sc.params.length === 1) {
    const single = ok.every(r => r.runs <= 1);
    return variation + (single ? "" : card(tr("scen.successPerVariant"), tr("scen.variationLabel", { esc: esc(sweepLabel(sc.params[0])) }), chartBox("sc-succ", 240)))
      + card(tr("scen.endPerVariant"), `${tr("scen.variationLabelSep", { esc: esc(sweepLabel(sc.params[0])) })} ${realView ? tr("scen.medianToday") : tr("scen.nominalBand")}`, chartBox("sc-end", 280))
      + card(tr("scen.retirePerVariant"), tr("scen.variationMedian", { esc: esc(sweepLabel(sc.params[0])) }), chartBox("sc-ret", 220));
  }
  if (sc.params.length === 2) return variation + card(tr("scen.successRate"), tr("scen.variationsLabel", { sc: sweepNames(sc) }), heat(sc, r => r.successRate, pctTxt, "high"))
    + card(tr("scen.endMedianTitle"), tr("scen.variationsReal", { sc: sweepNames(sc), realNote: realNote() }), heat(sc, r => realView ? r.p50Real : r.p50, fmtMio, "high"));
  return card(tr("scen.charts"), "", `<p class="hint" style="margin:0">${tr("scen.chartsLimit", { length: sc.params.length })}</p>`);
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
    lines: [{ color: "--ok", values: ok.map(r => r.successRate * 100) }], tip: i => head(i) + tr("scen.successTip", { pctTxt: pctTxt(ok[i].successRate) }) });
  Charts.category($("#sc-end"), $("#sc-end-tip"), { xs, yFmt: fmtMio,
    bands: realView ? [] : [{ lo: ok.map(r => r.p5), hi: ok.map(r => r.p95), color: "--c1", opacity: 0.18 }],
    lines: [{ color: "--c1", values: ok.map(r => realView ? r.p50Real : r.p50) }],
    tip: i => head(i) + (realView ? tr("scen.medianTip", { money: fmtMoney(ok[i].p50Real) }) : tr("scen.bandTip", { money: fmtMoney(ok[i].p95), money2: fmtMoney(ok[i].p50), money3: fmtMoney(ok[i].p5) })) });
  Charts.category($("#sc-ret"), $("#sc-ret-tip"), { xs, yFmt: fmtMio, lines: [{ color: "--c2", values: ok.map(r => r.atRetirement ?? 0) }],
    tip: i => head(i) + tr("scen.retireTip", { money: fmtMoney(ok[i].atRetirement ?? 0) }) });
}

// ---------- Layout ----------
/** #72: Word of the class of the success rate (the engine classifies, #209): key of the text table. */
const RATE_WORD = { high: "hoch", medium: "mittel", low: "tief" };
const helpLink = (topic, text, title = "") => `<a data-action="help-topic" data-topic="${topic}"${title ? ` title="${esc(title)}"` : ""}>${text}</a>`;
/**
 * #72: Result in words above the key figures of the overview. #209: The verdict (category and values) comes from the
 * engine (`digest`); here it is only worded in the language of the user interface, with the links into the help.
 */
function verdictHtml(r, sg) {
  const d = digestOf(sg || r);
  if (!d) return "";
  const lines = [];
  if (sg) {
    const end = d.endWealth[0];
    lines.push(d.verdict === "depleted" ? tr("verdict.depleted", { depleted: ymShow(d.depleted), ageStr: ageStr(ym2i(d.depleted)) })
      : `${tr("verdict.lasts", { money: fmtMoney(realView ? end.real : end.nominal) })}${realView ? tr("verdict.real") : ""}.`);
  } else {
    const end = endM();
    lines.push(`${tr("verdict.successPre")} <b>${pctTxt(d.successRate)}</b> (${helpLink("r-overview", tr(`verdict.rate_${RATE_WORD[d.verdict]}`), tr("verdict.enoughQuestion"))}${tr("verdict.successTail", { ymShow: ymShow(i2ym(end)), end: ageStr(end) })}`);
    if (d.depleted) lines.push(tr("verdict.worstDepleted", { ageStr: ageStr(ym2i(d.depleted)) }));
  }
  const next = d.next === "optimize"
    ? `${tr("verdict.nextOptPre")} ${helpLink("s-optimize", tr("section.optimize"))} ${tr("verdict.nextOptPost")}`
    : `${tr("verdict.nextScenPre")} ${helpLink("s-scenarios", tr("scen.title"))} ${tr("verdict.nextScenPost")}`;
  return `<div class="card verdict" id="verdict"><p>${lines.join(" ")}</p><p class="next">${next}</p></div>`;
}
function renderResults(r) {
  compareOpen = false;
  const info = `${r.elapsedMs < 1 ? r.elapsedMs.toFixed(2) : r.elapsedMs.toFixed(0)} ms`;
  if (r.error) {
    plainTitle(tr("results.notPossible")); $("#resultsInfo").textContent = ""; $("#resultsTabs").innerHTML = "";
    $("#resultsBody").innerHTML = `<div class="card errcard"><p style="margin-top:0">${esc(errText(r.error))}</p>
      <ul class="val">${r.issues.filter(i => i.severity === "error").map(i => `<li class="err" data-action="back"><span class="ic">!</span><span>${esc(msgText(i))}</span></li>`).join("")}</ul></div>`;
    return;
  }
  const tabs = [["overview", tr("results.overview")], ["charts", tr("scen.charts")]];
  if (r.batch) tabs.push(["runs", tr("results.runs")]);
  if (r.scenarios) tabs.push(["scenarios", tr("scen.title")]);
  tabs.push(["table", tr("results.yearValues")], ["params", tr("optRanges.params")]);
  if (!tabs.some(([k]) => k === resultTab)) resultTab = "overview";
  // Single run with variations: chosen step (otherwise the plan run)
  const sg = r.single ? singleOf(r) : null;
  setResultsTitle(sg ? [tr("results.single"), runInfo(sg.label), r.scenarios ? selectedVariationText() : ""].filter(Boolean).join(" · ") : tr("results.runsN", { runs: fmtInt(r.batch.summary.runs) }));
  const varied = r.batch && r.scenarios && !!variationSel;
  $("#resultsPick").innerHTML = r.single && r.scenarios ? variationPicker(r)
    : r.batch && r.scenarios && (resultTab === "runs" || resultTab === "table") ? variationPicker(r)
    : varied && resultTab !== "params" ? `<div class="card variednote"><span class="muted">${tr("results.planValuesNote", { esc: esc(selectedVariationText()) })}</span></div>` : "";
  $("#resultsInfo").textContent = sg ? tr("results.oneRun", { info }) : info;
  $("#resultsTabs").innerHTML = `<div class="seg tabs">${tabs.map(([k, t], i) => `<button class="${k === resultTab ? "on" : ""}" data-action="rtab" data-tab="${k}" title="${t} (${MOD}${i + 1})">${t}</button>`).join("")}</div>
    <div class="seg"><button class="${realView ? "" : "on"}" data-action="real" data-v="0" title="${tr("results.nominalTitle")}">${tr("results.nominal")}</button><button class="${realView ? "on" : ""}" data-action="real" data-v="1" title="${tr("results.realTitle")}">${tr("results.realLabel")}</button></div>`;
  const body = $("#resultsBody");
  const detailRun = () => sg || (r.batch && (detailsOf(r).find(([q]) => q === detailP)?.[1] || detailsOf(r)[0][1]));
  if (resultTab === "overview") {
    if (sg) {
      const res = sg;
      body.innerHTML = verdictHtml(r, res) + singleKpis(res.summary) + card(tr("runCharts.wealth"), realNote(), chartBox("ov-wealth", 300, true) + WEALTH_LEGEND) + propertyEvents(res) + card(tr("results.events"), "", eventList(res.events));
      drawRunCharts(res, "ov");
    } else {
      const b = r.batch, s = b.summary, pcts = pctsOf(b), pc = q => pcts.find(x => x.percent === q), ed = s.earliestDepletion;
      const end = q => pctEnd(b, pc(q));
      body.innerHTML = verdictHtml(r, null) + `<div class="rkpis">
        ${kpi(tr("scen.successRate"), pctTxt(s.successRate), tr("results.lastsToEnd"), s.successRate >= 0.95 ? "good" : s.successRate < 0.8 ? "bad" : "")}
        ${kpi(tr("results.endMedian"), fmtMio(end(50)), esc(percentInfo(r, 50, pc(50).label)))}
        ${kpi(tr("results.endP5"), fmtMio(end(5)), esc(percentInfo(r, 5, pc(5).label)), end(5) > 0 ? "" : "bad")}
        ${kpi(tr("results.endP1"), fmtMio(end(1)), esc(percentInfo(r, 1, pc(1).label)), end(1) > 0 ? "" : "bad")}
        ${kpi(tr("fan.worstRun"), fmtMio(end(0)), esc(percentInfo(r, 0, pc(0).label)), end(0) > 0 ? "" : "bad")}
        ${kpi(tr("results.endP95"), fmtMio(end(95)), esc(percentInfo(r, 95, pc(95).label)))}
        ${kpi(tr("results.endP99"), fmtMio(end(99)), esc(percentInfo(r, 99, pc(99).label)))}
        ${kpi(tr("scen.earliestDepletion"), ed ? tr("kpi.age") + ageStr(ym2i(ed[0])) : tr("results.none"), ed ? `${ymShow(ed[0])} · ${esc(msgText(ed[1]))}` : tr("results.inNoRun"), ed ? "bad" : "good")}
        ${hasProperties() ? kpi(tr("results.emergencySales"), tr("results.runsN", { runs: fmtInt(s.emergencySaleRuns) }), `${tr("results.ofRuns", { pctTxt: pctTxt(s.emergencySaleRuns / s.runs) })}${s.affordabilitySaleRuns ? tr("results.affordabilityPart", { affordabilitySaleRuns: fmtInt(s.affordabilitySaleRuns) }) : ""}`, s.emergencySaleRuns > 0 ? "bad" : "good") : ""}
        ${hasProperties() && (lastRunPlan || state).realestate.neverSell ? kpi(tr("results.unaffordableRenewals"), tr("results.runsN", { runs: fmtInt(s.unaffordableRenewalRuns || 0) }), tr("results.unaffordableRenewalsSub", { pctTxt: pctTxt((s.unaffordableRenewalRuns || 0) / s.runs) }), s.unaffordableRenewalRuns > 0 ? "bad" : "good") : ""}</div>`
        + card(tr("batch.band"), tr("batch.overRuns", { realNote: realNote() }), chartBox("mc-fan", 300, true) + FAN_LEGEND)
        + card(tr("results.endByPercentile"), realNote(), `<table class="tbl num"><tr><th class="l">${tr("results.percentile")}</th><th>${tr("scen.endWealth")}</th>${hasProperties() ? `<th class="l" title="${tr("results.salesTitle")}">${tr("results.sales")}</th>` : ""}<th class="l">${tr("scen.variation")}</th></tr>
          ${pcts.map(p => `<tr><td class="l">${p.percent === 0 ? tr("fan.worstRun") : p.percent + "."}</td><td>${fmtInt(pctEnd(b, p))}</td>${hasProperties() ? `<td class="l">${propertySales(batchDetails(b).find(([q]) => q === p.percent)?.[1])}</td>` : ""}<td class="l">${esc(percentInfo(r, p.percent, p.label))}</td></tr>`).join("")}</table>
          <p class="hint">${oldOrder(b) ? tr("results.oldOrder") : tr("results.realPctHint")}</p>
          ${r.scenarios ? `<p class="hint">${tr("results.variationHint")}</p>` : ""}`);
      drawBatchCharts(b);
    }
  } else if (resultTab === "charts") {
    if (sg) { body.innerHTML = runChartsHtml(sg, "rc"); drawRunCharts(sg, "rc"); }
    else { body.innerHTML = batchChartsHtml(r.batch); drawBatchCharts(r.batch); }
  } else if (resultTab === "runs") {
    const res = detailRun();
    $("#resultsPick").innerHTML = (r.scenarios ? variationPicker(r) : "") + runPicker(r, res);
    body.innerHTML = runChartsHtml(res, "dr") + propertyEvents(res) + card(tr("results.events"), "", eventList(res.events));
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
    body.innerHTML = card(tr("results.yearValues"), `${tr("results.realAmounts", { realNote: realNote(), cur: cur() })}${info ? ` · ${esc(info)}` : ""}`, yearTable(res, res.bucketRefills || []), "yearcard");
  }
  // #209: a result drawn before the engine delivered its yearly values and digest (e.g. directly after a computation)
  // is drawn again once they are there
  const shown = [sg, r.batch ? detailRun() : null, ...(r.batch?.details || []).map(([, x]) => x)].filter(Boolean);
  if (shown.some(x => !DERIVED.has(x)) || (r.batch && !DERIVED.has(r)))
    Promise.all([derive(r), deriveRuns(shown)]).then(() => { if (lastRun === r && !compareOpen) renderResults(r); });
  if (savedView) $("#resultsPick").insertAdjacentHTML("afterbegin", `<div class="card savedbar"><span>${tr("results.savedNote", { esc: esc(runDate(savedView.created)), name: esc(savedView.name) })}</span><span class="spacer"></span>
    <button class="btn small" data-action="runs-open">${tr("results.backToList")}</button><button class="btn small primary" data-action="run-restore" title="${tr("results.adoptPlanTitle")}">${tr("results.adoptPlan")}</button></div>`);
}
/**
 * Parameters of a plan as sections [title, [[label, value], …]], structured like the editor sections.
 * Person names are intentionally missing (only the birth month), technical values only as far as they make the result reproducible.
 * `full` (#123, comparison of the plans): additionally the remaining inputs of the editor (affordability, inflation of the
 * properties, end of the mortgage, rest of pillar 3a, property gains tax by holding period).
 */
function paramSections(plan, full = false) {
  const c = plan.project?.currency || "";
  const money = v => v == null ? "–" : `${fmtInt(v)} ${c}`, moneyYear = v => money(v == null ? v : v * 12), pc = v => v == null ? "–" : `${fmtPct(v)} %`, num = v => v == null ? "–" : String(v);
  const yn = v => v ? tr("common.yes") : tr("common.no");
  const msName = id => msLabel(plan.milestones?.find(m => m.id === id));
  const at = r => isYM(r) ? ymShow(r) : (msName(r) ?? String(r ?? "–"));
  const ser = (v, f) => !v || typeof v !== "object" ? (v == null ? "–" : f(v))
    : v.mode === "const" ? f(v.value) : isNewForm(v) ? formName(v.mode) : isCurve(v) ? tr("params.curve", { input: stateName(v.input) })
    : (v.rows || []).map(r => tr("params.fromValue", { from: at(r.from), value: f(r.value) })).join("; ") || "–";
  const table = rows => (rows || []).map(([x, r]) => tr("params.fromValue", { from: fmtInt(x), value: `${fmtPct(r)} %` })).join("; ") || "–";
  const out = [];
  const persons = plan.household?.persons || [];
  out.push([tr("params.household"), [[tr("general.currency"), c], ...persons.map((p, i) => [tr("params.personBirth", { v: i + 1 }), ymShow(p.birth)]),
    [tr("params.married"), yn(plan.household?.married)], ...(persons.length > 1 && plan.household?.married ? [[tr("general.ahvCap"), money(plan.household.ahvCoupleCap)]] : [])]]);
  out.push([tr("section.milestones"), (plan.milestones || []).map(m => [m.name, m.kind === "date" ? ymShow(m.date) : tr("params.ageOfPerson", { years: m.years, max: Math.max(1, persons.findIndex(p => p.id === m.person) + 1) })])]);
  const pens = (pre, o, ahv) => [
    [tr("params.pkCapital", { pre }), money(o.pk?.capital)], [tr("params.pkFrom", { pre }), at(o.pk?.at)], [tr("params.lumpsum", { pre }), pc(o.pk?.lumpsum)],
    [tr("params.conversion", { pre }), pc(o.pk?.conversion)], [tr("params.pkContribution", { pre }), ser(o.pk?.contribution, money)], [tr("params.pkInterest", { pre }), ser(o.pk?.interest, pc)],
    [tr("params.ahvMonth", { pre }), money(ahv?.monthly)], [tr("params.ahvFrom", { pre }), at(ahv?.at)], [tr("params.ahvIndexed", { pre }), yn(ahv?.indexed)]];
  const part = plan.partner;
  out.push([tr("section.pension"), [...pens(part ? tr("params.person1") : "", plan.pension || {}, plan.pension?.ahv),
    ...(part ? pens(tr("params.person2"), part, part.ahv) : [])]]);
  out.push([tr("params.flows"), [[tr("params.incomeYear"), ser(plan.flows?.income, moneyYear)], ...(part ? [[tr("params.income2Year"), ser(part.income, moneyYear)]] : []),
    [tr("params.spendingMonth"), ser(plan.flows?.spending, money)],
    ...(plan.flows?.extras || []).map(e => [tr("params.oneOff", { date: ymShow(e.date), v: e.label ? " " + e.label : "" }), money(e.amount)])]]);
  const w = plan.wealth || {};
  out.push([tr("params.wealth"), [[tr("wealth.stocks"), money(w.stocks)], [tr("wealth.bonds"), money(w.bonds)], [tr("wealth.cash"), money(w.cash)],
    [tr("sweep.stockShare"), ser(w.stockShare, pc)]]]);
  const r = plan.returns || {}, f = r.fixed || {};
  const src = r.source;
  const rrows = [[tr("params.source"), sourceName(src)]];
  if (src === "fixed") rrows.push([tr("params.stocksYear"), pc(f.stocks)], [tr("params.bondsYear"), pc(f.bonds)], [tr("params.inflationYear"), pc(f.inflation)], ["SARON", pc(f.saron)]);
  else {
    if (r.series && src !== "parametric") rrows.push([tr("params.series"), seriesShort(r.series)]);
    else if (r.strategy && src !== "parametric") {
      const name = (list, k) => list.find(x => x[0] === k)?.[1] ?? k, st = r.strategy, wt = st.weights || {};
      rrows.push([tr("params.stocksStrategy"), name(STOCK_INDICES, st.stocks)], [tr("params.bondsStrategy"), name(BOND_INDICES, st.bonds)]);
      if (st.stocks === "worldCustom") rrows.push([tr("computed.weights"), tr("params.weightsText", { ch: pc(wt.ch), usa: pc(wt.usa), eu: pc(wt.eu), apac: pc(wt.apac) })]);
    }
    if (src === "historical") rrows.push([tr("sweep.histStart"), ymShow(r.startDate)]);
    if (src === "bootstrap") rrows.push([tr("params.paths"), num(r.bootstrap?.paths)], [tr("params.blockLength"), num(r.bootstrap?.block)], [tr("returns.seed"), num(r.bootstrap?.seed)]);
    if (src === "parametric") { const q = r.parametric || {};
      rrows.push([tr("params.paths"), num(q.paths)], [tr("optRanges.params"), q.auto ? tr("params.suggestedValues") : tr("params.ownValues")], [tr("params.expectedReturn"), pc(q.mu)], [tr("optimize.volatility"), pc(q.sigma)], [tr("params.dof"), num(q.df)], [tr("returns.jumps"), num(q.jumpLambda)], [tr("params.jumpMean"), pc(q.jumpMu)], [tr("params.jumpSpread"), pc(q.jumpSigma)], [tr("returns.seed"), num(q.seed)]);
      if (r.strategy) { const st = r.strategy, name = (list, k) => list.find(x => x[0] === k)?.[1] ?? k;
        rrows.push([tr("params.stocksStrategy"), name(STOCK_INDICES, st.stocks)], [tr("returns.bondsInfl"), tr("params.randomCorrelated", { bondIndices: name(BOND_INDICES, st.bonds) })]); }
      else rrows.push([tr("params.bondsYear"), pc(f.bonds)], [tr("params.inflationYear"), pc(f.inflation)]); }
    if (r.extrapolation) rrows.push([tr("returns.extrapolationTitle"), `${r.extrapolation.method ?? "–"}${r.extrapolation.range ? ` (${r.extrapolation.range} ${tr("unit.yearsShort")})` : ""}`]);
  }
  out.push([tr("params.sourceSeries"), rrows]);
  out.push([tr("section.withdrawal"), [[tr("optResult.strategy"), stratName(stratKey(plan.withdrawal))], ...stratParams(plan)]]);
  const props = plan.realestate?.properties || [];
  const STATUS = { owned: tr("params.owned"), planned: tr("params.planned"), rent: tr("params.rent") };
  out.push([tr("section.realestate"), [[tr("params.emergencyBelow"), ser(plan.realestate?.emergencyMonths, num)], ...props.flatMap(p => {
    const nm = p.name || tr("params.property");
    return [[tr("params.status", { nm }), STATUS[p.status] || p.status], ...(p.price != null ? [[tr("params.price", { nm }), money(p.price)]] : []), ...(p.worth != null ? [[tr("params.marketValue", { nm }), money(p.worth)]] : []),
      // #229: tax value (not for rentals)
      ...(p.taxValue && p.status !== "rent" ? [[tr("params.taxValue", { nm }), p.taxValue.kind === "rate" ? tr("params.taxValueRate", { pc: pc(p.taxValue.value) }) : money(p.taxValue.value)]] : []),
      // #273: the default tax value
      ...(!p.taxValue && p.status !== "rent" && plan.realestate?.defaultTaxValue ? [[tr("params.taxValue", { nm }), tr("params.taxValueDefault", { pc: taxValueDefaultPc() })]] : []),
      ...(p.buy ? [[tr("params.buy", { nm }), at(p.buy)]] : []), ...(p.sell ? [[tr("params.sell", { nm }), at(p.sell)]] : []),
      ...(p.fixCosts != null ? [[tr("params.costsMonth", { nm }), money(p.fixCosts)]] : []), ...(p.rent != null ? [[tr("params.rentMonth", { nm }), money(p.rent)]] : []),
      ...(p.mortgage?.tranches || []).map(t => [`${nm}: ${t.name || tr("mortgage.tranche")}`, `${money(t.amount)}${t.start ? ` ${tr("common.fromDate", { date: ymShow(t.start) })}` : ""}; ${(t.periods || []).map(d => tr("params.periodText", { v: d.type === "saron" ? "SARON" : tr("mortgage.fixedLower"), term: num(d.term), rate: pc(d.rate), margin: pc(d.margin), amort: money(d.amort), v2: d.amortMode === "indirect" ? tr("mortgage.indirect") : tr("mortgage.direct") })).join(" → ")}`]),
    ];
  })]]);
  out.push([tr("section.pillar3a"), (plan.pillar3a || []).flatMap(a => {
    const nm = (plan.household?.persons || []).find(q => q.id === a.person)?.name || a.person;
    return [[tr("params.pillar3aText", { nm }), `${money(a.balance)}, ${pc(a.rate)}, ${money(a.contribution ?? 0)}`],
      ...(a.payouts || []).map(x => [`${tr("params.payout", { nm })} ${x.kind === "age" ? tr("params.atYears", { years: x.years }) : ymShow(x.date)}`, `${x.amount != null ? money(x.amount) : tr("params.wholeBalance")} (${x.use === "repay" ? tr("mortgage.title") : tr("params.wealthShort")})`])];
  })]);
  const t = plan.taxes || {}, g = t.propertyGain;
  // #63/AK10: mode and place of residence with the name of the municipality instead of the own tables
  // #64: the lump-sum tax by the place of residence with the rule of the canton
  // #151/AK7: one line per place of residence («Wohnort ab …»), the lump-sum rule of each canton once
  const places = locRows(plan), cantons = [...new Set(places.map(r => r.loc.canton).filter(Boolean))];
  const lumpsum = isLocation(plan) ? (cantons.length ? cantons.map(c => tr("params.lumpsumLocation", { rule: (cantons.length > 1 ? `${c}: ` : "") + tr(`capitalRule.${c}`) })).join("; ") : tr("taxes.modeLocation")) : table(t.capital);
  const incomeWealth = isLocation(plan) ? [[tr("params.taxMode"), tr("taxes.modeLocation")], ...places.map(r => [r.from == null ? tr("taxes.location") : tr("params.locationFrom", { when: at(r.from) }), locationText(r.loc)])]
    : [[tr("params.taxMode"), tr("taxes.modeOwn")], [tr("taxes.incomeTax"), table(t.income)], [tr("taxes.wealthTax"), table(t.wealth)]];
  // #149: taxable yield of the wealth (stocks / bonds)
  const yieldText = tr("params.investmentYieldValue", { stocks: pc(t.investmentYield?.stocks ?? 0), bonds: pc(t.investmentYield?.bonds ?? 0) });
  // #274/AK7: deductions from income taken over from the tax bill
  const bill = (t.deductions ?? []).find(r => r.taxBill)?.taxBill;
  out.push([tr("params.taxAssumptions"), [[tr("params.multiplier"), num(t.multiplier)], [tr("params.taxIndexed"), yn(t.indexed)], ...incomeWealth, ...(bill ? [[tr("taxBill.params"), tr("taxBill.origin", { year: bill })]] : []), [tr("params.investmentYield"), yieldText], [tr("taxes.lumpsumTax"), lumpsum],
    // #65: in the mode «Wohnort» without own rules the rule of the canton of each property
    [tr("gain.title"), !gainOwn(plan) ? tr("params.gainCanton") : g ? `${g.model === "aargau" ? tr("params.modelAg") : tr("params.steps")}${tr("params.reduction", { pc: pc(g.reduction ?? 0) })}${g.exemptBelow ? tr("params.exemptBelow", { exemptBelow: money(g.exemptBelow) }) : ""}` : tr("results.none")]]]);
  out.push([tr("scen.variations"), (plan.scenarios?.sweeps || []).length ? plan.scenarios.sweeps.map(sw => [sweepLabel(sw.param), tr("params.rangeStep", { v: SWEEP_SHOW[sweepType(sw.param)]?.(sw.from) ?? sw.from, v2: SWEEP_SHOW[sweepType(sw.param)]?.(sw.to) ?? sw.to, step: sw.step })]) : [[tr("scen.variations"), tr("results.none")]]]);
  if (full) {
    const add = (title, rows) => { const sec = out.find(x => x[0] === title); if (sec) sec[1].push(...rows); else if (rows.length) out.push([title, rows]); };
    const a = plan.realestate?.affordability || affordability(); // #233: values of the calculation, otherwise the settings
    add(tr("section.realestate"), [[tr("params.affIncome"), pc(a.sustainability)], [tr("params.affRate"), pc(a.imputedRate)],
      [tr("params.affCosts"), pc(a.fixCosts)], [tr("params.affLtv"), pc(a.maxLoanToValue)],
      ...props.flatMap(p => { const nm = p.name || tr("params.property"); return [
        ...(p.status === "owned" ? [[tr("params.buyDate", { nm }), ymShow(p.bought)]] : []),
        ...(!gainOwn(plan) && p.status !== "rent" ? [[tr("params.gainCantonOf", { nm }), p.canton || cantons.join(" → ") || "–"]] : []),
        ...(p.status === "rent" ? [[tr("params.rentFrom", { nm }), at(p.from)], [tr("params.rentIndexed", { nm }), yn(p.inflationRent)]]
          : [[tr("params.valueIndexed", { nm }), yn(p.inflationValue)], [tr("params.costsIndexed", { nm }), yn(p.inflationFixCosts)]]),
        ...(p.mortgage?.tranches || []).map(t => [`${nm}: ${t.name || tr("mortgage.tranche")}${tr("params.afterLast")}`, t.after === "repay" ? tr("mortgage.repay") : tr("params.extendLower")])]; })]);
    add(tr("section.pillar3a"), (plan.pillar3a || []).map(x => [tr("params.restAtAhv", { v: (plan.household?.persons || []).find(q => q.id === x.person)?.name || x.person }), x.use === "repay" ? tr("mortgage.title") : tr("params.wealthShort")]));
    // Time-dependent values of the strategy as a list instead of «zeitabhängig», so that differences are visible
    const ws = plan.wealth || {}, strat = out.find(x => x[0] === tr("section.withdrawal"));
    if (strat) strat[1] = strat[1].map(([k, v]) => k === tr("withdrawal.reserve") ? [k, ser(ws.reserveMonths, x => tr("params.months", { x }))] : k.startsWith(tr("sweep.stockShare")) ? [k, ser(ws.stockShare, pc)] : [k, v]);
    if (g && gainOwn(plan)) add(tr("params.taxAssumptions"), [[tr("params.gainMultiplier"), yn(g.useMultiplier)],
      ...(g.durationRates?.length ? [[tr("params.gainHolding"), g.durationRates.map(([y, r]) => tr("params.holdingRate", { y, rate: fmtPct(r) })).join("; ")]] : [])]);
  }
  return out;
}
/** Tab «Parameter»: read-only summary of the plan in the state of the run. */
function paramsHtml(plan) {
  if (!plan) return card(tr("optRanges.params"), "", `<p style="margin:0">${tr("params.notAvailable")}</p>`);
  return `<p class="hint" style="margin:0 0 10px">${tr("params.snapshotHint")}</p>`
    + paramSections(plan).map(([title, rows]) => card(title, "", `<table class="tbl params">${rows.map(([k, v]) => `<tr><td class="l">${esc(k)}</td><td class="l">${esc(v)}</td></tr>`).join("")}</table>`)).join("");
}
/** Selection of the run (percentile) together with the values of the variations; sits above the scrolling area. */
function runPicker(r, res) {
  const info = [pName(detailP), percentInfo(r, detailP, res.label)].filter(Boolean).join(" · ");
  return card(tr("runPicker.title"), "", `<div class="seg">${detailsOf(r).map(([p]) => `<button class="${p === detailP ? "on" : ""}" data-action="detail" data-p="${p}">${pName(p)}</button>`).join("")}</div>
    ${batchRuns()?.off?.length ? `<p class="hint" style="color:var(--warn,#b45309)">${tr("runPicker.deviates", { batchRuns: batchRuns().off.map(pName).join(", ") })}</p>` : ""}
    <p class="computed" style="margin-bottom:0">${tr("runPicker.info", { info: esc(info), endTotal: fmtMoney(realView ? res.summary.endTotalReal : res.summary.endTotal) })}${res.summary.depleted ? ` · ${tr("runPicker.depleted", { atAge: atAge(res.summary.depleted) })}` : ""}</p>`);
}
function showDetail(p) { detailP = p; bucketOpen.clear(); if (lastRun) renderResults(lastRun); }

// ---------- Saved simulations ----------
/** #201: saved optimization shown in the results view ({ meta, plan, opt }), otherwise null. */
let savedView = null, parkedState = null, savedOpt = null;
/** #120: Maximum number of results in the comparison; each has its own color (--r1 … --r10 in index.html). */
const CMP_MAX = 10;
const runsSel = new Set(), RUN_COLORS = Array.from({ length: CMP_MAX }, (_, i) => `--r${i + 1}`);
const runDate = iso => new Date(iso).toLocaleString("de-CH", { dateStyle: "short", timeStyle: "short" });
const stratName = id => id === "three_buckets_guardrails" ? tr("opt.bucketsGuardrails") : STRATS.find(x => x[0] === id)?.[1] || id;
const sourceName = id => SOURCES.find(x => x[0] === id)?.[1] || id;
/** Name of a new result: plan · source (for scenarios «Szenarien») · strategy · date, time. */
const runTitle = (base, m, iso) => [base, m.kind === "optimization" ? optKindName(m) : m.kind === "scenarios" ? tr("scen.title") : SOURCE_SHORT[m.source] || sourceName(m.source), STRAT_SHORT[m.strategy] || stratName(m.strategy), runDate(iso)].join(" · ");
/** While viewing a saved simulation its plan applies; the editor state is set aside meanwhile. */
function unpark() { if (parkedState) { state = parkedState; parkedState = null; } savedView = null; }
/** Value of a time series of the plan as text: constant → value, otherwise «zeitabhängig». */
const serTxt = (v, f) => v && typeof v === "object" ? (v.mode === "const" ? f(v.value) : isCurve(v) ? tr("params.curve", { input: stateName(v.input) }) : tr("runs.timeDependent")) : v == null ? "–" : f(v);
/** Parameters of the withdrawal strategy of a plan as pairs [label, value] (labels and units as in the editor). */
function stratParams(plan) {
  const w = plan.withdrawal || {}, ws = plan.wealth || {}, pc = v => `${fmtPct(v)} %`, yrs = v => `${v} ${tr("unit.yearsShort")}`;
  const share = (label = tr("sweep.stockShare")) => [label, serTxt(ws.stockShare, pc)];
  const limit = v => v ? pc(v) : tr("results.none");
  const g = w.guardrails || {};
  const guard = () => [[tr("guardrails.startRate"), pc(g.initialRate)], [tr("stratParams.upper"), pc(g.upper)], [tr("stratParams.lower"), pc(g.lower)], [tr("guardrails.adjust"), pc(g.adjust)], [tr("limits.lower"), limit(w.floor)], [tr("limits.upper"), limit(w.ceiling)]];
  switch (w.strategy) {
    case "three_buckets": { const b = w.buckets || {};
      return [[tr("withdrawal.bucket1"), yrs(b.cashYears)], [tr("withdrawal.bucket2"), yrs(b.bondYears)], [tr("withdrawal.sellRule"), pc(b.refillThreshold ?? 0)], ...(ws.stockShare != null ? [share(tr("stratParams.stockShareRest"))] : []),
        ...(bucketsGuardrails(w) ? guard() : [])]; }
    case "guardrails": return [...guard(), share()];
    case "fixed_rate": return [[tr("withdrawal.rate"), pc((w.fixedRate || {}).rate)], share()];
    case "vpw": { const v = w.vpw || {}; return [[tr("withdrawal.untilAge"), `${v.horizonAge} ${tr("unit.yearsShort")}`], [tr("withdrawal.realReturn"), pc(v.realReturn)], share()]; }
    case "endowment": { const e = w.endowment || {}; return [[tr("withdrawal.targetRate"), pc(e.rate)], [tr("withdrawal.weight"), pc(e.smoothing)], share()]; }
    default: return [share(), [tr("withdrawal.reserve"), serTxt(ws.reserveMonths, v => tr("params.months", { x: v }))]];
  }
}
/** Name of the time series used (without folder and extension); `null` if the return source has no time series. */
function seriesLabel() {
  const r = state.returns;
  if (r.source === "fixed" || r.source === "parametric" || !seriesRef()) return null;
  return (seriesInfo && !seriesInfo.error && (msgText(seriesInfo.title) || seriesInfo.name)) || (r.series ? seriesShort(r.series) : null);
}
/** Display of the column «Zeitreihe» (header data without a value: «–»). */
const seriesCell = m => m.seriesName === undefined ? "–" : m.seriesName ? esc(m.seriesName)
  : m.source === "fixed" ? `<span class="muted">${tr("runs.fixedReturns")}</span>` : m.source === "parametric" ? `<span class="muted">${tr("runs.parametric")}</span>` : "–";
/** Share of runs with an emergency sale from which the warning icon appears (configurable later, #25). */
const EMERGENCY_WARN_SHARE = 0.05;
/** Warning icon (house with warning sign) for the unplanned sale of a property; `tip` is the tooltip. */
const houseWarn = tip => `<span class="hwarn" role="img" aria-label="${esc(tip)}" title="${esc(tip)}">🏠⚠️</span>`;
/** Icon in the list of results: single run with emergency sale, Monte Carlo from the threshold; old entries without a value: none. */
function emergencyMark(m) {
  if (m.kind === "single") return m.emergencySaleRuns > 0 ? ` ${houseWarn(tr("runs.emergencyOne"))}` : "";
  if (m.kind !== "batch" || !(m.runs > 0) || !(m.emergencySaleRuns > 0)) return "";
  const share = m.emergencySaleRuns / m.runs;
  return share >= EMERGENCY_WARN_SHARE ? ` ${houseWarn(tr("runs.emergencyShare", { share: pctTxt(share), emergencySaleRuns: fmtInt(m.emergencySaleRuns), runs: fmtInt(m.runs) }))}` : "";
}
/** Strategy name with the parameters of the result as tooltip (also reachable by keyboard). */
const stratCell = (m, cls = "") => {
  const tip = m.params ? m.params.map(([k, v]) => `${k}: ${v}`).join("\n") : tr("runs.paramsNA");
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
/** #201: «Optimierung» or «Strategievergleich» (comparison without search, #197). */
const optKindName = m => m.compare ? tr("savedOpt.compare") : tr("savedOpt.optimization");
const runKind = m => m.kind === "optimization" ? [optKindName(m), m.goal === "keepProperty" ? tr("savedOpt.goalKeep") : tr("savedOpt.goalConsumption"), tr("savedOpt.paths", { paths: fmtInt(m.runs) })].join(" · ")
  : m.kind === "single" ? tr("results.single") : m.kind === "scenarios" ? tr("runs.scenariosN", { runs: m.runs }) : tr("runs.runsSource", { runs: fmtInt(m.runs), source: sourceName(m.source) });
async function rememberRun(r) {
  if (r.error || (!r.single && !r.batch && !r.scenarios)) return;
  try {
    const plan = withAffordability(lastRunPlan && lastRun === r ? lastRunPlan : state), m = runMeta(r);
    const base = (currentPath && baseName(currentPath).replace(/\.[rv]plan$/, "")) || plan.project.name || tr("refs.plan");
    const now = new Date().toISOString();
    const meta = { id: Date.now().toString(36) + Math.random().toString(36).slice(2, 8), created: now, name: runTitle(base, m, now), note: "", ...m };
    await Runs.add(meta, { plan, result: r });
    // #125: the result just shown gets its name into the header
    if (lastRun === r && !savedView) { lastRunName = meta.name; if (resultHead) setResultsTitle(); }
    document.documentElement.dataset.saved = (+document.documentElement.dataset.saved || 0) + 1;
  } catch (e) {
    // #147: report visibly (e.g. browser storage full); the user interface stays usable, the result stays displayed
    toast(tr("runs.notListed") + errText(e));
    noteError(tr("runs.notSaved") + errText(e));
  }
}
let runsIds = [];
/** #201: ids of the saved optimizations in the list; they can be marked for deleting but are not compared. */
const runsOpt = new Set();
/** Marked simulations (without optimizations): only they are compared. */
const cmpMarked = () => [...runsSel].filter(id => !runsOpt.has(id));
function runsBar() {
  const k = runsSel.size, n = runsIds.length, c = cmpMarked().length, canCompare = c >= 2 && c <= CMP_MAX;
  $("#resultsTabs").innerHTML = `<button class="btn small" data-action="runs-compare" ${canCompare ? "" : "disabled"} title="${tr("runs.compareTitle", { cmpMax: CMP_MAX })}">${tr("runs.compare", { k: c })}</button>${c > CMP_MAX ? `<span class="cmpmax hint" role="status">${tr("runs.compareMax", { cmpMax: CMP_MAX })}</span>` : ""}
    <button class="btn small" data-action="runs-delete" ${k < 1 ? "disabled" : ""}>${tr("runs.deleteMarked", { k })}</button>
    <button class="btn small" data-action="runs-delete-all" ${n < 1 ? "disabled" : ""}>${tr("runs.deleteAll")}</button>`;
  const all = document.querySelector('input[data-action="runs-all"]');
  if (all) { all.checked = n > 0 && k === n; all.indeterminate = k > 0 && k < n; }
}
async function openRuns() {
  showResults(true); lastRun = null; savedView = null; compareOpen = false;
  plainTitle(tr("runs.title"));
  let list = [];
  try { list = await Runs.list(); } catch (e) { noteError(errText(e)); }
  for (const id of [...runsSel]) if (!list.some(m => m.id === id)) runsSel.delete(id);
  runsIds = list.map(m => m.id);
  runsOpt.clear(); list.forEach(m => { if (m.kind === "optimization") runsOpt.add(m.id); });
  $("#resultsInfo").textContent = `${list.length} ${list.length === 1 ? tr("runs.entry") : tr("runs.entries")} ${tr("runs.localOnly")}`;
  runsBar();
  const cell = m => m.kind === "optimization" ? optCell(m) : (m.kind === "batch" ? `<span title="${tr("runs.medianTitle", { end50: fmtMio(m.end50) })}">${pctTxt(m.successRate)} · 5 %: ${fmtMio(m.end5)}</span>` : m.kind === "single" ? tr("runs.endWealth", { endTotal: fmtMio(m.endTotal) }) : "–") + emergencyMark(m);
  $("#resultsBody").innerHTML = list.length ? `<div class="card"><table class="tbl" id="runsTbl">
    <tr><th><input type="checkbox" data-action="runs-all" title="${tr("runs.markAllTitle")}"></th><th>${tr("runs.name")}</th><th>${tr("runs.note")}</th><th>${tr("runs.date")}</th><th>${tr("runs.series")}</th><th>${tr("runs.strategy")}</th><th>${tr("runs.result")}</th><th></th></tr>
    ${list.map(m => `<tr><td><input type="checkbox" data-action="run-sel" data-id="${m.id}" ${runsSel.has(m.id) ? "checked" : ""} title="${m.kind === "optimization" ? tr("savedOpt.markTitle") : tr("runs.markTitle", { cmpMax: CMP_MAX })}"></td>
      <td><input class="runedit" data-id="${m.id}" data-f="name" value="${esc(m.name)}" title="${tr("runs.renameTitle")}"></td>
      <td><input class="runedit" data-id="${m.id}" data-f="note" value="${esc(m.note)}" placeholder="${tr("runs.notePlaceholder")}"></td>
      <td>${esc(runDate(m.created))}</td><td>${seriesCell(m)}</td>${stratCell(m)}<td>${cell(m)}<br><small class="muted">${esc(runKind(m))}</small></td>
      <td style="white-space:nowrap"><button class="btn small" data-action="run-view" data-id="${m.id}">${tr("runs.open")}</button> <button class="btn small" data-action="run-del" data-id="${m.id}" title="${tr("runs.deleteTitle")}">${tr("runs.delete")}</button></td></tr>`).join("")}
    </table><p class="hint">${tr("runs.hint", { cmpMax: CMP_MAX })}</p></div>`
    : `<div class="card"><p style="margin:0">${tr("runs.empty")}</p></div>`;
  document.documentElement.dataset.rendered = "runs";
}
/** #201: Result column of a saved optimization: ruin of the best strategy, with goal 2 the share of unplanned sales. */
const optCell = m => [m.goal === "keepProperty" && m.emergencyShare != null ? `${tr("optResult.unplanned")} ${pctTxt(m.emergencyShare)}` : null,
  m.ruin != null ? `${tr("optResult.ruin")} ${pctTxt(m.ruin)}` : null].filter(Boolean).join(" · ") || "–";
/** #201: A saved optimization in the wide results view, with the table of the section «Optimierung» (#200). */
function viewOpt(meta, data) {
  showResults(true); lastRun = null; savedView = null; compareOpen = false;
  savedOpt = { meta, plan: data.plan, opt: data.opt };
  const r = data.opt.report, plan = normalize(JSON.parse(JSON.stringify(data.plan)));
  const s = state; state = plan; let start; try { start = startM(); } finally { state = s; }
  plainTitle(meta.name);
  $("#resultsInfo").textContent = runKind(meta);
  $("#resultsTabs").innerHTML = "";
  const other = optKey() !== data.opt.planJson;
  $("#resultsPick").innerHTML = `<div class="card savedbar"><span>${tr("savedOpt.note", { date: esc(runDate(meta.created)), name: esc(meta.name), paths: fmtInt(r.checkPaths) })}${other ? `<br><span class="savedother" style="color:var(--warn)">${tr("savedOpt.otherHint")}</span>` : ""}</span><span class="spacer"></span>
    <button class="btn small" data-action="runs-open">${tr("results.backToList")}</button></div>`;
  $("#resultsBody").innerHTML = optResultCard(r, data.opt.req, false, start) + `<footer class="rfoot">${esc(DISCLAIMER_FOOT)}</footer>`;
  document.documentElement.dataset.rendered = "savedOpt";
}
async function viewRun(id) {
  const [meta] = (await Runs.list()).filter(m => m.id === id), data = await Runs.get(id);
  if (!meta || !data) { toast(tr("runs.gone")); return openRuns(); }
  if (meta.kind === "optimization") return viewOpt(meta, data);
  await derive(data.result);
  resetVariation();
  if (!parkedState) parkedState = state;
  state = normalize(JSON.parse(JSON.stringify(data.plan)));
  savedView = meta; lastRun = data.result; lastRunPlan = state; resultTab = "overview"; detailP = 50;
  showResults(true); renderResults(lastRun);
  document.documentElement.dataset.rendered = "saved";
}
// ---------- Comparison of saved results (#115) ----------
// Tabs as for the single result: overview (key figures, range, percentiles, probability, depletion),
// trajectories (charts of the single run or median run), yearly values (one column per result) and plan (differences).
// The results are aligned by months or years since their start.
let compareTab = "overview", cmpYearKey = "total", cmpLiquid = "liquid", cmpFlow = "in", cmpP = 50;
/** #118: Sorting of the key figures in the comparison: { col, dir } (dir 1 ascending, −1 descending); null = order of the list. */
let cmpSort = null;
let CMP_TABS;
onLanguage(() => CMP_TABS = [["overview", tr("results.overview")], ["charts", tr("cmp.courses")], ["table", tr("results.yearValues")], ["plan", tr("refs.plan")]]);
let CMP_YEAR;
onLanguage(() => CMP_YEAR = [["total", tr("cmp.totalWealth")], ["liquid", tr("legend.liquid")], ["property", tr("section.realestate")], ["pension", tr("section.pension")],
  ["receipts", tr("yearTable.income")], ["spending", tr("yearTable.spending")], ["draw", tr("yearTable.withdrawal")], ["rate", tr("yearTable.rate")], ["taxTotal", tr("single.taxesTotal")]]);
let CMP_LIQUID;
onLanguage(() => CMP_LIQUID = [["liquid", tr("yearTable.total")], ["stocks", tr("wealth.stocks")], ["bonds", tr("wealth.bonds")], ["cash", tr("wealth.cash")]]);
let CMP_FLOW;
onLanguage(() => CMP_FLOW = [["in", tr("cmp.income")], ["out", tr("cmp.spending")], ["taxes", tr("runCharts.taxesEstimated")]]);
/**
 * What a saved result provides for the comparison. `run` is the run for trajectories and yearly values: for a single run
 * this one, for multiple runs the run of the chosen percentile (`cmpP`, like «Lauf wählen» in the single result); if this
 * percentile is missing, the median run (`pMissing`).
 */
function cmpItem(m, d, i) {
  const res = d?.result || {}, batch = res.batch || null, single = res.single || null;
  const details = batch ? batchDetails(batch) : [];
  const det = q => details.find(([p]) => p === q)?.[1];
  const exact = batch ? det(cmpP) : null;
  const run = single || exact || (batch && (det(50) || details[0]?.[1])) || null;
  return { m, i, color: RUN_COLORS[i], name: m.name, batch, single, run, pMissing: !!batch && !exact, scenariosOnly: !batch && !single };
}
/** Selection of the percentile for trajectories and yearly values; only for results with multiple runs. */
function cmpPicker(items) {
  const qs = [...new Set(items.filter(it => it.batch).flatMap(it => (it.batch.details || []).map(([p]) => p)))].sort((a, b) => a - b);
  if (!qs.length) return "";
  if (!qs.includes(cmpP)) cmpP = qs.includes(50) ? 50 : qs[0];
  return card(tr("runPicker.title"), tr("cmp.pickerSub"), `<div class="seg">${qs.map(p => `<button class="${p === cmpP ? "on" : ""}" data-action="cmp-p" data-p="${p}">${pName(p)}</button>`).join("")}</div>`);
}
/** Which run is shown per result, with a remark on single runs and missing percentiles. */
function cmpRunNotes(items) {
  const runs = items.filter(it => it.run), out = [];
  if (runs.some(it => it.batch)) out.push(`${tr("cmp.multiRuns", { cmpP: pName(cmpP) })}${cmpP === 50 ? tr("cmp.median") : ""}.`);
  const singles = runs.filter(it => it.single);
  if (singles.length) out.push(tr("cmp.singleNoPct", { map: singles.map(it => esc(it.name)).join(", ") }));
  const miss = runs.filter(it => it.pMissing);
  if (miss.length) out.push(tr("cmp.withoutPct", { cmpP: pName(cmpP), map: miss.map(it => esc(it.name)).join(", ") }));
  items.filter(it => !it.run).forEach(it => out.push(tr("cmp.noRun", { name: esc(it.name) })));
  return out;
}
const cmpSel = (action, opts, cur) => `<div class="seg" style="margin-bottom:8px">${opts.map(([k, t]) => `<button class="${k === cur ? "on" : ""}" data-action="${action}" data-v="${k}">${t}</button>`).join("")}</div>`;
const cmpNote = list => list.length ? `<p class="hint cmpnote">${list.join(" ")}</p>` : "";
const cmpLegend = items => legend(items.map(it => [it.color, esc(it.name)]));
/** Series of equal length (months since start); missing months stay gaps. */
const cmpPad = (n, v) => Array.from({ length: n }, (_, k) => k < v.length ? v[k] : NaN);
const cmpX = n => Array.from({ length: n }, (_, k) => k);
const cmpTime = (id, n, lines, tip, extra = {}) => Charts.time($(`#${id}`), $(`#${id}-tip`), { x: cmpX(n), xFmt: m => `${m / 12} ${tr("unit.yearsShort")}`,
  view: resultZoom ? [resultZoom.from, resultZoom.to] : null, onZoom: setCustomZoom, lines, tip: k => tr("cmp.yearTip", { v: (k / 12).toLocaleString("de-CH", { maximumFractionDigits: 1 }), k: tip(k) }), ...extra });
const cmpRow = (it, v, f) => Number.isFinite(v) ? `<br><span style="color:var(${it.color})">■</span> ${esc(it.name)}: ${f(v)}` : "";
/** Calendar years equal: year number; otherwise «n. Jahr» since start (the year number is in the tooltip of the cell). */
function cmpYears(items, valuesOf) {
  const per = items.map(it => it.run ? valuesOf(it) : null);
  const firsts = per.filter(Boolean).map(v => v[0]?.y);
  const same = firsts.every(y => y === firsts[0]);
  const n = Math.max(0, ...per.map(v => v ? v.length : 0));
  return { per, n, same, label: k => same ? (per.find(v => v && v[k])?.[k].y ?? "") : tr("cmp.yearN", { v: k + 1 }) };
}
/** Saved results of the comparison by id until the list changes (`Runs.version`); the yearly values of the engine stay with them. */
let cmpDatas = { version: -1, byId: new Map() };
/** Key figures of the engine for the selection (ids in order), so that a toggle does not ask the engine again. */
let cmpKeyFigures = { key: null, cmp: null };
async function compareRuns() {
  const metas = (await Runs.list()).filter(m => runsSel.has(m.id) && m.kind !== "optimization").reverse().slice(0, CMP_MAX);
  const version = Runs.version, kept = cmpDatas.version === version ? cmpDatas.byId : new Map();
  const datas = await Promise.all(metas.map(m => kept.get(m.id) || Runs.get(m.id)));
  cmpDatas = { version, byId: new Map(metas.map((m, i) => [m.id, datas[i]])) };
  const items = metas.map((m, i) => cmpItem(m, datas[i], i));
  // #209: key figures side by side from the engine; yearly values of the shown runs
  const key = `${version}:${metas.map(m => m.id).join("|")}`;
  const [cmp] = await Promise.all([cmpKeyFigures.key === key ? cmpKeyFigures.cmp
    : Engine.call("compare_results", { items: datas.map(d => ({ result: slimResult(d?.result), plan: d?.plan || null })) }), deriveRuns(items.map(it => it.run))]);
  cmpKeyFigures = { key, cmp };
  lastRun = null; savedView = null; compareOpen = true;
  if (!CMP_TABS.some(([k]) => k === compareTab)) compareTab = "overview";
  plainTitle(tr("cmp.title", { length: metas.length })); $("#resultsInfo").textContent = realView ? tr("single.todayPower") : tr("results.nominalLower");
  $("#resultsPick").innerHTML = "";
  $("#resultsTabs").innerHTML = `<button class="btn small" data-action="runs-open">${tr("results.backToList")}</button>
    <div class="seg tabs">${CMP_TABS.map(([k, t]) => `<button class="${k === compareTab ? "on" : ""}" data-action="cmp-tab" data-tab="${k}">${t}</button>`).join("")}</div>
    <div class="seg"><button class="${realView ? "" : "on"}" data-action="real" data-v="0" title="${tr("results.nominalTitle")}">${tr("results.nominal")}</button><button class="${realView ? "on" : ""}" data-action="real" data-v="1" title="${tr("cmp.realTitle")}">${tr("results.realLabel")}</button></div>`;
  const body = $("#resultsBody");
  if (compareTab === "overview") cmpOverview(body, items, cmp);
  else if (compareTab === "charts") cmpCharts(body, items);
  else if (compareTab === "table") cmpTable(body, items);
  else cmpPlan(body, items, datas);
  document.documentElement.dataset.rendered = "compare";
}
const cmpSw = it => `<i class="sw" style="background:var(${it.color})"></i>`;
const cmpReal = () => realView ? tr("cmp.realNote") : tr("results.nominalLower");
function cmpOverview(body, items, cmp) {
  const num = v => v == null || !Number.isFinite(v) ? "–" : fmtMio(v);
  /** #209: Key figures of the engine (`compare_results`): final wealth of a percentile, nominal or in today's purchasing
   *  power with the inflation of the run (for results before #126 from the detail runs), and the depletion. */
  const kf = it => cmp.items[it.i];
  const endAt = (it, q) => { const e = kf(it).digest?.endWealth.find(x => x.percent === q); return (e && (realView ? e.real : e.nominal)) ?? null; };
  const depMonth = it => kf(it).depletion;
  const notes = [];
  // #118: Key figures sortable by column; colors and order of the other cards stay
  const depAt = it => depMonth(it) === undefined ? null : depMonth(it) ? ym2i(depMonth(it)) : Infinity;
  const cols = [["name", tr("runs.result"), "l", it => it.name], ["series", tr("runs.series"), "l", it => it.m.seriesName || (it.m.source === "fixed" ? tr("runs.fixedReturns") : it.m.source === "parametric" ? tr("runs.parametric") : null)],
    ["strat", tr("runs.strategy"), "l", it => stratName(it.m.strategy)], ["succ", tr("scen.successRate"), "r", it => kf(it).digest?.successRate ?? null],
    ["p50", tr("cmp.medianOrEnd"), "r", it => endAt(it, 50)], ["p5", tr("cmp.p5"), "r", it => endAt(it, 5)], ["p1", tr("cmp.p1"), "r", it => endAt(it, 1)],
    ["p0", tr("fan.worstRun"), "r", it => endAt(it, 0)], ["dep", tr("scen.earliestDepletion"), "r", depAt]];
  const sc = cmpSort && cols.find(c => c[0] === cmpSort.col);
  const sorted = sc ? sortBy(items, sc[3], cmpSort.dir) : items;
  const head = cols.map(([k, t, al]) => { const on = sc && sc[0] === k, dir = on ? cmpSort.dir : 0;
    return `<th class="${al} sorth" aria-sort="${dir > 0 ? "ascending" : dir < 0 ? "descending" : "none"}"><button class="sortbtn" data-action="cmp-sort" data-col="${k}" title="${tr("cmp.sortTitle", { v: esc(t), v2: on ? (dir > 0 ? tr("cmp.descending") : tr("cmp.ascending")) : "" })}">${t}<span class="sorti${on ? " on" : ""}" aria-hidden="true">${dir > 0 ? "▲" : dir < 0 ? "▼" : "↕"}</span></button></th>`; }).join("");
  const rows = sorted.map(it => { const m = it.m; return `<tr><td class="l">${cmpSw(it)}${esc(m.name)}<br><small class="muted">${esc(runKind(m))}</small></td><td class="l">${seriesCell(m)}</td>${stratCell(m, " l")}
    <td class="r">${kf(it).digest?.successRate != null ? pctTxt(kf(it).digest.successRate) : "–"}</td>
    <td class="r">${num(endAt(it, 50))}</td><td class="r">${num(endAt(it, 5))}</td><td class="r">${num(endAt(it, 1))}</td><td class="r">${num(endAt(it, 0))}</td>
    <td class="r">${depMonth(it) === undefined ? "–" : depMonth(it) ? ymShow(depMonth(it)) : tr("results.none")}</td></tr>`; }).join("");
  const scen = items.filter(it => it.scenariosOnly);
  if (scen.length) notes.push(tr("cmp.onlyKpi", { map: scen.map(it => esc(it.name)).join(", ") }));
  // Range: median (or single run) and 5th/95th percentile
  const bandOf = it => { if (!it.batch) return null; const b = it.batch.bands; return realView ? b.real || null : b; };
  const noReal = items.filter(it => it.batch && realView && !it.batch.bands.real);
  if (noReal.length) notes.push(tr("cmp.noReal", { map: noReal.map(it => esc(it.name)).join(", ") }));
  const partReal = realView ? items.filter(it => it.batch && it.batch.bands.real && it.batch.summary.percentiles.some(p => p.endTotalReal == null)) : [];
  if (partReal.length) notes.push(tr("cmp.partReal", { map: partReal.map(it => esc(it.name)).join(", ") }));
  const oldReal = items.filter(it => it.batch && it.batch.bands.real && oldOrder(it.batch) && !partReal.includes(it));
  if (oldReal.length) notes.push(tr("cmp.realOrder", { map: oldReal.map(it => esc(it.name)).join(", ") }));
  const singles = items.filter(it => it.single);
  if (singles.length) notes.push(tr("cmp.singleNoBand", { map: singles.map(it => esc(it.name)).join(", ") }));
  const fan = items.map(it => it.single ? { med: it.single.records.map(r => rv(r, r.total)) } : bandOf(it) ? { med: bandOf(it).p50, lo: bandOf(it).p5, hi: bandOf(it).p95 } : null);
  const n = Math.max(0, ...fan.map(f => f ? f.med.length : 0));
  const fanLines = [];
  fan.forEach((f, k) => { if (!f) return; fanLines.push({ color: items[k].color, values: cmpPad(n, f.med) });
    if (f.lo) fanLines.push({ color: items[k].color, dash: "5 3", width: 1.2, values: cmpPad(n, f.lo) }, { color: items[k].color, dash: "2 3", width: 1.2, values: cmpPad(n, f.hi) }); });
  // Percentiles: all percentiles of the results as rows
  const batches = items.filter(it => it.batch);
  const qs = cmp.percentiles;
  const pTable = batches.length ? `<div class="tscroll"><table class="tbl num"><tr><th class="l">${tr("results.percentile")}</th>${batches.map(it => `<th class="cmph">${cmpSw(it)}${esc(it.name)}</th>`).join("")}</tr>
    ${qs.map(q => `<tr><td class="l">${q === 0 ? tr("fan.worstRun") : q + "."}</td>${batches.map(it => { const v = endAt(it, q); return `<td>${v == null ? "–" : fmtInt(v)}</td>`; }).join("")}</tr>`).join("")}</table></div>`
      + `<p class="hint">${tr("results.realPctHint")}</p>`
    : `<p class="hint" style="margin:0">${tr("cmp.noMulti")}</p>`;
  // Probability that the money lasts
  const surv = items.map(it => it.batch ? it.batch.stats.survival.map(v => v * 100) : null);
  const ns = Math.max(0, ...surv.map(v => v ? v.length : 0));
  // Age at depletion: share of runs per year of age (results with different numbers of runs stay comparable)
  const ages = cmp.depletionAges;
  body.innerHTML = card(tr("cmp.kpis"), tr("cmp.endWealth", { cmpReal: cmpReal() }), `<div class="tscroll"><table class="tbl cmpkpi"><tr>${head}</tr>${rows}</table></div>`
      )
    + cmpNote(notes)
    + (fanLines.length ? card(tr("batch.band"), tr("cmp.bandSub", { cmpReal: cmpReal() }), chartBox("cmp-wealth", 300, true) + cmpLegend(items.filter((_, k) => fan[k]))) : "")
    + card(tr("results.endByPercentile"), cmpReal(), pTable)
    + (batches.length ? card(tr("batch.survival"), tr("batch.survivalSub"), survSwitch() + chartBox("cmp-surv", 260, true) + cmpLegend(batches)
      + survClipNote(Math.min(...surv.filter(Boolean).flat()))) : "")
    + (batches.length ? card(tr("batch.depletionAge"), tr("cmp.depletionSub"),
      ages.length ? chartBox("cmp-dep", 220) + cmpLegend(batches) : `<p class="hint" style="margin:0">${tr("cmp.noDepletion")}</p>`) : "");
  if (fanLines.length) cmpTime("cmp-wealth", n, fanLines, k => fan.map((f, j) => f ? cmpRow(items[j], cmpPad(n, f.med)[k], fmtMoney) : "").join(""), { yFmt: fmtMio });
  if (batches.length) cmpTime("cmp-surv", ns, surv.map((v, j) => v && { color: items[j].color, values: cmpPad(ns, v) }).filter(Boolean),
    k => surv.map((v, j) => v ? cmpRow(items[j], cmpPad(ns, v)[k], x => `${x.toLocaleString("de-CH", { maximumFractionDigits: 1 })} %`) : "").join(""), { yFmt: v => `${v} %`, range: survRange() });
  if (ages.length) {
    const shares = batches.map(it => kf(it).depletionShares);
    Charts.bars($("#cmp-dep"), $("#cmp-dep-tip"), { cats: ages.map(String), minBar: 3, yFmt: v => `${(v * 100).toLocaleString("de-CH", { maximumFractionDigits: 1 })} %`,
      series: batches.map((it, j) => ({ group: String(j), color: it.color, values: shares[j] })),
      tip: k => tr("cmp.ageTip", { v: ages[k] }) + batches.map((it, j) => cmpRow(it, shares[j][k], pctTxt)).join("") });
  }
}
function cmpCharts(body, items) {
  const runs = items.filter(it => it.run);
  const notes = cmpRunNotes(items);
  if (!runs.length) { body.innerHTML = cmpNote(notes) || `<p class="hint">${tr("cmp.noCourses")}</p>`; return; }
  const hasProp = runs.filter(it => it.run.records.some(r => r.propertyValue > 0)), hasPk = runs.filter(it => it.run.records.some(r => r.pensionFund > 0 || r.pillar3a > 0));
  const noProp = runs.filter(it => !hasProp.includes(it)), noPk = runs.filter(it => !hasPk.includes(it));
  const n = Math.max(...runs.map(it => it.run.records.length));
  const series = (it, f) => cmpPad(n, it.run.records.map(r => rv(r, f(r))));
  const LQ = { liquid: r => r.liquid, stocks: r => r.stocks, bonds: r => r.bonds, cash: r => r.cash };
  // Yearly values of the bars: income, spending, taxes; withdrawal rate as in the single result
  const yrs = cmpYears(runs, it => yearFlows(it.run));
  const flowOf = a => cmpFlow === "in" ? a.income : cmpFlow === "out" ? a.spending : a.taxes;
  const rateOf = a => a.rate;
  const yearSeries = f => yrs.per.map(v => v ? Array.from({ length: yrs.n }, (_, k) => v[k] ? f(v[k]) : NaN) : null).filter(Boolean);
  const cats = Array.from({ length: yrs.n }, (_, k) => String(yrs.label(k)));
  const idx = (it, key) => cmpPad(n, it.run.records.map(r => 100 * r[key] / (realView ? r.priceIndex : 1)));
  body.innerHTML = cmpPicker(items) + cmpNote(notes)
    + card(tr("cmp.totalWealthShort"), cmpReal(), chartBox("cmp-total", 280, true) + cmpLegend(runs))
    + card(tr("runCharts.liquidByAsset"), cmpReal(), cmpSel("cmp-liquid", CMP_LIQUID, cmpLiquid) + chartBox("cmp-liq", 280, true) + cmpLegend(runs))
    + card(tr("runCharts.flows"), tr("cmp.flowsSub", { cmpReal: cmpReal() }), cmpSel("cmp-flow", CMP_FLOW, cmpFlow) + chartBox("cmp-flows", 260) + cmpLegend(runs))
    + card(tr("runCharts.withdrawal"), tr("runCharts.withdrawalSub"), chartBox("cmp-draw", 240) + cmpLegend(runs))
    + (hasProp.length ? card(tr("section.realestate"), tr("cmp.equitySub", { cmpReal: cmpReal() }), chartBox("cmp-prop", 240, true) + cmpLegend(hasProp) + cmpNote(noProp.length ? [tr("cmp.noProperty", { map: noProp.map(it => esc(it.name)).join(", ") })] : [])) : "")
    + (hasPk.length ? card(tr("section.pension"), tr("cmp.pensionSub", { cmpReal: cmpReal() }), chartBox("cmp-pk", 240, true) + cmpLegend(hasPk) + cmpNote(noPk.length ? [tr("cmp.noPension", { map: noPk.map(it => esc(it.name)).join(", ") })] : [])) : "")
    + card(tr("runCharts.returns"), `${mktPct ? tr("runCharts.change") : tr("runCharts.index")}${tr("cmp.returnsSub", { cmpReal: cmpReal() })}${mktPct ? "" : tr("cmp.priceIndexSuffix")}`,
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
  cmpTime("cmp-mkt", n, mk, k => runs.map((it, j) => cmpRow(it, mser[j].s[k], () => mktPct ? tr("cmp.returnsTipChange", { f1: f1(mser[j].s[k]), f12: f1(mser[j].b[k]), f13: f1(mser[j].p[k]) })
    : tr("cmp.returnsTipIndex", { f1: f1(mser[j].s[k]), f12: f1(mser[j].b[k]), f13: f1(mser[j].p[k]) }))).join(""),
    { ref: mktPct ? 0 : 100, log: mktLog && !mktPct, yFmt: mktPct ? v => `${v.toLocaleString("de-CH", { maximumFractionDigits: 1 })} %` : v => v.toLocaleString("de-CH", { maximumFractionDigits: 0 }) });
}
function cmpTable(body, items) {
  const runs = items.filter(it => it.run);
  const yrs = cmpYears(runs, it => yearValues(it.run));
  const key = cmpYearKey, label = CMP_YEAR.find(([k]) => k === key)?.[1] || "";
  const cell = (v, y) => `<td title="${esc(String(y))}">${key === "rate" ? rateTxt(v) : !Number.isFinite(v) ? "–" : key === "taxTotal" ? fmtTax(v) : fmtInt(v)}</td>`; // #143/AK2
  const notes = cmpRunNotes(items);
  const rows = Array.from({ length: yrs.n }, (_, k) => `<tr><td>${yrs.label(k)}</td>${yrs.per.map(v => v[k] ? cell(v[k][key], v[k].y) : "<td>–</td>").join("")}</tr>`).join("");
  body.innerHTML = cmpPicker(items) + cmpNote(notes) + card(tr("results.yearValues"), `${tr("cmp.tableSub", { label, cmpReal: cmpReal(), cur: cur() })}${runs.some(it => it.batch) ? ` · ${pName(cmpP)}` : ""}`,
    `<label class="field" style="max-width:320px"><span>${tr("cmp.measure")}</span><select data-action="cmp-year">${CMP_YEAR.map(([k, t]) => `<option value="${k}" ${k === key ? "selected" : ""}>${t}</option>`).join("")}</select></label>
    <div class="tscroll"><table class="tbl num yt cmpyear"><tr><th>${yrs.same ? tr("yearTable.year") : tr("cmp.yearSinceStart")}</th>${runs.map(it => `<th class="cmph">${cmpSw(it)}${esc(it.name)}</th>`).join("")}</tr>${rows}</table></div>
    <p class="hint">${tr("cmp.tableHint")}${yrs.same ? "" : tr("cmp.tableYearsHint")}</p>${key === "taxTotal" ? `<p class="hint taxline">${tr("taxes.estimateLine")}</p>` : ""}`, "yearcard");
}
document.addEventListener("change", e => { if (e.target.matches?.('select[data-action="cmp-year"]')) { cmpYearKey = e.target.value; compareRuns(); } });
/**
 * #123: Fields of the plan that do not appear in the comparison (technical or without an input field), with reason. All other
 * inputs are shown by `paramSections(plan, true)` with the labels of the editor; a user interface test checks the coverage.
 */
let PLAN_HIDDEN;
onLanguage(() => PLAN_HIDDEN = [
  [/^format$/, tr("planHidden.format")], [/^project\.name$/, tr("planHidden.name")], [/(^|\.)id$/, tr("planHidden.id")],
  [/^household\.persons\.\d+\.name$/, tr("planHidden.persons")], [/^milestones\.\d+\.fixed$/, tr("planHidden.fixedMilestone")],
  [/\.mode$/, tr("planHidden.seriesKind")], [/^flows\.spendingIncludesHousing$/, tr("planHidden.noField")], [/^flows\.amortExcluded$/, tr("planHidden.noField")],
  [/^realestate\.affordability\.wealthContribution$/, tr("planHidden.noField")], [/^withdrawal\.buckets\.reserveMonths$/, tr("planHidden.noField")],
  [/^realestate\.properties\.\d+\.inflation$/, tr("planHidden.legacyField")],
  [/^returns\.parametric\.sampler$/, tr("planHidden.sampling")],
  [/(^|\.)ahv\.thirteenthNoted$/, tr("planHidden.noField")], [/^household\.marriedNoted$/, tr("planHidden.noField")], [/(^|\.)ahv\.referenceAge$/, tr("planHidden.noField")],
]);
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
  body.innerHTML = card(tr("cmpPlan.title"), tr("cmpPlan.sub"),
    n ? `<div class="tscroll"><table class="tbl cmpplan"><tr><th class="l">${tr("cmpPlan.input")}</th>${items.map(it => `<th class="l cmph">${cmpSw(it)}${esc(it.name)}</th>`).join("")}</tr>
      ${groups.map(([title, rows]) => `<tr class="grp"><th class="l" colspan="${cols}">${esc(title)}</th></tr>${rows.map(([k, vs]) => `<tr><td class="l">${esc(k)}</td>${items.map((_, j) => `<td class="l">${esc(vs[j] ?? "—")}</td>`).join("")}</tr>`).join("")}`).join("")}</table></div>`
      : `<p style="margin:0">${tr("cmpPlan.identical")}</p>`);
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
    if (!await ask(tr("runsDelete.allTitle"), `${tr("runsDelete.allText", { n })} ${n === 1 ? tr("runsDelete.resultIs") : tr("runsDelete.resultsAre")} ${tr("runsDelete.deletedTail")}`)) return;
    await Runs.remove([...runsIds]); runsSel.clear(); openRuns();
  },
  "runs-compare": () => compareRuns(),
  "run-del": async d => {
    const [m] = (await Runs.list()).filter(x => x.id === d.id);
    if (!await ask(tr("runsDelete.oneTitle"), tr("runsDelete.oneText", { esc: esc(m?.name || "") }))) return;
    await Runs.remove([d.id]); runsSel.delete(d.id); openRuns();
  },
  "runs-delete": async () => {
    const ids = [...runsSel]; if (!ids.length) return;
    if (!await ask(tr("runsDelete.markedTitle"), `${tr("runsDelete.markedText", { length: ids.length })} ${ids.length === 1 ? tr("runsDelete.resultIs") : tr("runsDelete.resultsAre")} ${tr("runsDelete.deletedTail")}`)) return;
    await Runs.remove(ids); runsSel.clear(); openRuns();
  },
  "run-restore": async () => {
    if (!await confirmDiscard()) return;
    parkedState = null; savedView = null; lastRun = null;
    delete state.realestate.affordability; // #233: the settings apply
    showResults(false); setFile(null, true); History.reset(); goto("general"); refreshSeriesInfo();
    toast(tr("runsDelete.planLoaded"));
  }
});
document.addEventListener("change", e => {
  const el = e.target.closest(".runedit"); if (!el) return;
  Runs.update(el.dataset.id, { [el.dataset.f]: el.value.trim() });
});

// ---------- Welcome (#70) and first steps (#71) ----------
/** #184: Example plan with its names in the language of the user interface. */
const localExample = plan => localizeExample(plan, key => tr(key));
const examplePlan = async () => localExample(await Engine.call("example_plan"));
/** Loads the example plan as a new, unsaved plan (after asking if there are unsaved changes). */
async function loadExample() {
  if (!await confirmDiscard()) return false;
  await adopt(await examplePlan(), false); setFile(null); History.reset(); goto("general"); refreshSeriesInfo();
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
let STEPS;
onLanguage(() => STEPS = [["general", "s-general"], ["milestones", "s-milestones"], ["pension", "s-pension"], ["flows", "s-flows"],
  ["wealth", "s-wealth"], ["returns", "s-returns"], ["taxes", "s-taxes"], ["run", "quickstart"]]
  .map(([id, topic]) => ({ id, topic, title: SECTIONS.find(s => s.id === id)?.title ?? tr("shortcuts.run") })));
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
    if (all && !v.complete) { v.complete = true; toast(tr("steps.allDone")); return this.put(v, false); }
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
    el.innerHTML = `<details${open ? " open" : ""}><summary>${tr("steps.title")} <span class="prog" id="stepsProg">${tr("steps.progress", { length: v.done.length, length2: STEPS.length })}</span>
      <button type="button" class="x" data-action="steps-hide" title="${tr("steps.hideTitle")}" aria-label="${tr("steps.hideLabel")}">✕</button></summary>
      <ol>${STEPS.map((s, i) => { const done = v.done.includes(s.id); return `<li class="${done ? "done" : ""}" data-step="${s.id}">
        <input type="checkbox" data-action="steps-check" data-step="${s.id}"${done ? " checked" : ""} aria-label="${tr("steps.doneLabel", { title: esc(s.title) })}">
        <a data-action="steps-go" data-step="${s.id}">${i + 1}. ${esc(s.title)}</a><button type="button" class="hq" data-help="${s.topic}" title="${tr("steps.helpTitle")}" aria-label="${tr("steps.helpLabel")}">ⓘ</button></li>`; }).join("")}</ol>
      ${v.complete ? `<p class="stepsdone">${tr("steps.doneAll")}</p>` : ""}</details>`;
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
  "ai-copy": aiCopy,
  "ai-folder": aiFolder,
  "ai-help": d => { $("#ask").close(); Help.show(d.topic); },
});

// ---------- Start ----------
(async () => {
  const initial = location.hash.slice(1).split("/");
  await Pref.load();
  Theme.apply();
  showTaxCols = Pref.get("showTaxCols", false) === true;
  if (SECTIONS.some(s => s.id === initial[0])) current = initial[0];
  // #270/AK4: the activation comes before everything else; the plan of the start is loaded only afterwards.
  if (!window.__VIVAPLAN_SMOKE__ && (Engine.inApp || initial.includes("beta"))) {
    try { await betaGate(!Engine.inApp); } catch (e) { document.body.innerHTML = `<p style="padding:24px">${tr("boot.engineUnreachable", { esc: esc(errText(e)) })}</p>`; return; }
  }
  let startup = null;
  try {
    // Desktop app: plan last opened, otherwise the bundled example plan (#4)
    if (Engine.inApp) startup = await Engine.call("startup_plan");
    // #184: without a path the app opens the example plan, with its names in the language of the user interface
    var startNotes = await adopt(startup?.path ? startup.plan : startup ? localExample(startup.plan) : await examplePlan(), !!startup?.path);
    gainTemplates = await Engine.call("gain_templates");
    taxLoc = await Engine.call("tax_locations");
  } catch (e) { document.body.innerHTML = `<p style="padding:24px">${tr("boot.engineUnreachable", { esc: esc(errText(e)) })}</p>`; return; }
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
      message = (r.error && errText(r.error)) || (window.__errors.length ? window.__errors.join("; ") : `${r.single.records.length} Monate gerechnet`); // i18n: allow (message of the smoke test, only in the log)
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
  // #170: the app reports the notice as key of the text table with the file name
  if (startup?.notice) ask(tr("boot.startupTitle"), `<p>${esc(tr(startup.notice.key, { file: startup.notice.file }))}</p>`);
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
  if (initial.includes("nominal")) realView = false;
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
