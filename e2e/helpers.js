// Shared helper functions of the user interface tests (e2e/ui-*.spec.js).
const { expect } = require("@playwright/test");

const MOD = process.platform === "darwin" ? "Meta" : "Control";
const SECTIONS = ["Haushalt & Projekt", "Meilensteine", "Vorsorge", "Säule 3a", "Einnahmen & Ausgaben", "Vermögen & Anlage",
  "Simulation", "Entnahmestrategie", "Optimierung", "Steuern", "Immobilien", "Szenarien & Simulation"];

/**
 * Opens the user interface (optionally with test access via the address) and collects JavaScript errors.
 * #111: without measuring the computing time (#nobench), since it would otherwise run in the background of every test and
 * make it depend on the machine load. Tests that check the real measurement open with `{ bench: true }`.
 */
async function open(page, hash = "", { bench = false } = {}) {
  const errors = [];
  page.on("pageerror", e => errors.push(e.message));
  const h = bench ? hash : [hash, "nobench"].filter(Boolean).join("/");
  await page.goto("/" + (h ? "#" + h : ""));
  await page.waitForSelector("html[data-ready='1']");
  await expect(page.locator("#chart path").first()).toBeAttached();
  return errors;
}
const noErrors = async (page, errors) => {
  expect(errors).toEqual([]);
  expect(await page.evaluate(() => window.__errors)).toEqual([]);
};
const field = (page, p) => page.locator(`input[data-path="${p}"]`);
/** #111: waits until no preview is pending any more (otherwise a delayed preview half takes along the next input). */
const previewIdle = page => expect(page.locator("html")).toHaveAttribute("data-preview", "idle");
/** #111: click into the menu. Every preview redraws the menu; a click that falls into a preview would get lost. */
const nav = async (page, title) => { await previewIdle(page); await page.locator("#nav a", { hasText: title }).first().click(); };
const kpiValue = (page, label) => page.locator(`#kpis .kpi:has(.k:text-is("${label}")) .v`);
const resultKpi = (page, label) => page.locator(`#resultsBody .kpi:has(.k:text-is("${label}")) .v`);
const parseNum = s => Number(s.replace(/[’'\sk]/g, ""));
/** Sets up the optimization (e.g. small computation) and returns what is in the fields. The values go through the
 *  plan (`state.optimization`), since `saveOptCfg` reads the fields and overwrites a directly set `optCfg` (#79). */
const setOpt = (page, cfg) => page.evaluate(cfg => { state.optimization = optToPlan({ ...optCfg, ...cfg }); renderSection(); return readOptCfg(); }, cfg);
/** Table row of a milestone (the name is in an input field, therefore via the index of the ID). */
async function msRow(page, id) {
  const i = await page.evaluate(id => state.milestones.findIndex(m => m.id === id), id);
  // Fixed milestones (#51) have no editable name; every row has the selection «Festgelegt als», though.
  return page.locator(`#main table.tbl tr:has(select[data-path="milestones.${i}.kind"])`);
}

// ---------------------------------------------------------------------------------------------
const welcome = page => page.locator("#welcome");
const steps = page => page.locator("#steps");
const helpTitle = page => page.locator("#help:not([hidden]) #helpBody h1").first();
const prefs = page => page.evaluate(() => JSON.parse(localStorage.getItem("rp.settings") || "{}"));

module.exports = { MOD, SECTIONS, open, noErrors, field, previewIdle, nav, kpiValue, resultKpi, parseNum, setOpt, msRow, welcome, steps, helpTitle, prefs };
