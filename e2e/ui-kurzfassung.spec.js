// User interface tests of VivaPlan in the browser: yearly values, digest and comparison computed by the engine (#209).
const { test, expect } = require("@playwright/test");
const fs = require("fs");
const path = require("path");
const { open, noErrors } = require("./helpers");

/**
 * #209/AK3: Reference of the display of the example plan, recorded before the logic moved from `app.js` into the engine
 * (`RECORD_209=1 npx playwright test ui-kurzfassung.spec.js` writes it anew). Compared are the texts and table cells of the
 * result in words, the key figures, the yearly values and the comparison, and the drawn bar charts.
 */
const REFERENCE = path.join(__dirname, "fixtures", "issue-209-display.json");

/** Visible text of an element, whitespace condensed. */
const text = (page, sel) => page.locator(sel).first().evaluate(e => e.textContent.replace(/\s+/g, " ").trim());
/** Cells of a table, row by row. */
const cells = (page, sel) => page.locator(sel).first().evaluate(t => [...t.querySelectorAll("tr")].map(tr => [...tr.children].map(c => c.textContent.replace(/\s+/g, " ").trim())));
/** Geometry of a chart (paths and bars), without the tooltip layer. */
const shape = (page, sel) => page.locator(sel).first().evaluate(s => [...s.querySelectorAll("path, rect")].map(e => e.getAttribute("d") ?? `${e.getAttribute("x")},${e.getAttribute("y")},${e.getAttribute("width")},${e.getAttribute("height")}`));
const tab = (page, name) => page.locator('#resultsTabs button[data-action="rtab"], #resultsTabs button[data-action="cmp-tab"]').filter({ hasText: name }).click();

/** Single result: overview, yearly values (nominal, with taxes, in today's purchasing power) and bar charts. */
async function singleView(page) {
  const out = {};
  await expect(page.locator("#verdict")).toBeAttached();
  out.verdict = await text(page, "#verdict");
  out.kpis = await text(page, "#resultsBody .rkpis");
  await tab(page, "Grafiken");
  await expect(page.locator("#rc-flows rect").first()).toBeAttached();
  out.flows = await shape(page, "#rc-flows");
  out.draw = await shape(page, "#rc-draw");
  await tab(page, "Jahreswerte");
  await expect(page.locator("#resultsBody table.yt")).toBeAttached();
  out.table = await cells(page, "#resultsBody table.yt");
  await page.locator('button[data-action="tax-cols"]').click();
  await expect(page.locator("#resultsBody table.yt th.tx").first()).toBeAttached();
  out.tableTaxes = await cells(page, "#resultsBody table.yt");
  await page.locator('#resultsTabs button[data-action="real"][data-v="1"]').click();
  await expect(page.locator('#resultsTabs button[data-action="real"][data-v="1"].on')).toBeAttached();
  out.tableReal = await cells(page, "#resultsBody table.yt");
  await page.locator('button[data-action="tax-cols"]').click();
  await page.locator('#resultsTabs button[data-action="real"][data-v="0"]').click();
  return out;
}

/** Comparison of two saved results: key figures without the name column, percentiles, yearly values, bar charts. */
async function compareView(page) {
  const out = {};
  await expect(page.locator("html")).toHaveAttribute("data-rendered", "compare");
  await page.evaluate(() => showResults(true));
  await expect(page.locator("table.cmpkpi")).toBeAttached();
  out.kpis = (await cells(page, "table.cmpkpi")).slice(1).map(r => r.slice(3));
  out.percentiles = await cells(page, "#resultsBody .card:nth-of-type(3) table.tbl");
  out.depletion = await page.locator("#cmp-dep").count() ? await shape(page, "#cmp-dep") : null;
  await tab(page, "Jahreswerte");
  out.years = {};
  for (const key of ["total", "liquid", "property", "pension", "receipts", "spending", "draw", "rate", "taxTotal"]) {
    await page.locator('select[data-action="cmp-year"]').selectOption(key);
    await expect(page.locator(`select[data-action="cmp-year"] option[value="${key}"]`)).toHaveJSProperty("selected", true);
    out.years[key] = (await cells(page, ".cmpyear")).slice(1);
  }
  await tab(page, "Verläufe");
  await expect(page.locator("#cmp-flows rect").first()).toBeAttached();
  out.flows = {};
  for (const v of ["in", "out", "taxes"]) {
    await page.locator(`button[data-action="cmp-flow"][data-v="${v}"]`).click();
    await expect(page.locator(`button[data-action="cmp-flow"][data-v="${v}"].on`)).toBeAttached();
    out.flows[v] = await shape(page, "#cmp-flows");
  }
  out.draw = await shape(page, "#cmp-draw");
  return out;
}

test.describe("Yearly values, digest and comparison from the engine (#209)", () => {
  test("AK3: the display of the example plan is unchanged (single run, several runs, comparison)", async ({ context }) => {
    test.setTimeout(240_000);
    // A new page per view: only the hash would change, and the app would not start anew
    let page = await context.newPage();
    const shown = {};
    // #265: the reference was recorded starting nominal
    let errors = await open(page, "results/nosweep/nominal");
    await expect(page.locator("html")).toHaveAttribute("data-rendered", "overview");
    shown.single = await singleView(page);
    await noErrors(page, errors);

    // A saved Monte Carlo result (#runsflow/view): result in words, key figures and yearly values of the median run
    page = await context.newPage();
    errors = await open(page, "runsflow/mem/view/nominal");
    await expect(page.locator("html")).toHaveAttribute("data-rendered", "saved");
    await expect(page.locator("#verdict")).toBeAttached();
    shown.batch = { verdict: await text(page, "#verdict"), kpis: await text(page, "#resultsBody .rkpis") };
    await tab(page, "Jahreswerte");
    await expect(page.locator("#resultsBody table.yt")).toBeAttached();
    shown.batch.table = await cells(page, "#resultsBody table.yt");
    await noErrors(page, errors);

    page = await context.newPage();
    errors = await open(page, "runsflow/mem/nominal");
    shown.compare = await compareView(page);
    await noErrors(page, errors);

    if (process.env.RECORD_209) {
      fs.mkdirSync(path.dirname(REFERENCE), { recursive: true });
      fs.writeFileSync(REFERENCE, JSON.stringify(shown, null, 1) + "\n");
    }
    // The only difference allowed: a negative zero shows as «0» instead of «-0». The engine receives the monthly values
    // as JSON, and JSON has no negative zero (`JSON.stringify(-0)` is "0"); the old «-0» was an artifact of the display.
    // Names of saved results contain the point in time of the run.
    const when = v => typeof v === "string" ? v.replace(/\d\d\.\d\d\.\d\d, \d\d:\d\d/g, "<Zeitpunkt>") : v;
    const reference = JSON.parse(fs.readFileSync(REFERENCE, "utf8"), (k, v) => v === "-0" ? "0" : when(v));
    const now = JSON.parse(JSON.stringify(shown), (k, v) => when(v));
    expect(now.single).toEqual(reference.single);
    expect(now.batch).toEqual(reference.batch);
    expect(now.compare).toEqual(reference.compare);
  });

  test("AK2: the user interface takes yearly values, result in words and comparison from the engine", async ({ context }) => {
    const calls = [];
    context.on("request", r => { const m = new URL(r.url()).pathname.match(/^\/api\/(\w+)$/); if (m) calls.push(m[1]); });
    let page = await context.newPage();
    let errors = await open(page, "results/nosweep");
    await expect(page.locator("#verdict")).toBeAttached();
    expect(calls).toEqual(expect.arrayContaining(["run_simulation", "yearly", "digest"]));
    // the former computations of the user interface are gone
    expect(await page.evaluate(() => [typeof yearly, typeof rateClass, typeof flat])).toEqual(["undefined", "undefined", "undefined"]);
    const src = fs.readFileSync(path.join(__dirname, "../crates/app/ui/app.js"), "utf8");
    expect(src).not.toMatch(/function yearly\(|liquidStart > 0|depletionAges\.find|percentiles\.map\(p => p\.percent\)/);
    await noErrors(page, errors);

    calls.length = 0;
    page = await context.newPage();
    errors = await open(page, "runsflow/mem");
    await expect(page.locator("html")).toHaveAttribute("data-rendered", "compare");
    expect(calls).toContain("compare_results");
    await noErrors(page, errors);
  });
});
