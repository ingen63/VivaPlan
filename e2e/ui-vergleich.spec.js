// User interface tests of VivaPlan in the browser: charts and comparison. Every test starts with the example plan.
const { test, expect } = require("@playwright/test");
const fs = require("fs");
const os = require("os");
const path = require("path");
const { MOD, SECTIONS, open, noErrors, field, previewIdle, nav, kpiValue, resultKpi, parseNum, setOpt, msRow, welcome, steps, helpTitle, prefs } = require("./helpers");

test.describe("Reference line in the charts (#114)", () => {
  /** Reference line of a chart: y position and stroke width; plus the y position of a labeled grid line. */
  const refOf = (page, sel) => page.evaluate(sel => {
    const svg = document.querySelector(sel), ref = svg.querySelector("line.ref");
    const tickY = label => { const t = [...svg.querySelectorAll("text")].find(x => x.textContent === label); return t ? Number(t.getAttribute("y")) - 4 : null; };
    const grid = [...svg.querySelectorAll("line:not(.ref):not(.hov)")].find(l => l.getAttribute("y1") === l.getAttribute("y2"));
    return ref && { y: Number(ref.getAttribute("y1")), width: Number(ref.getAttribute("stroke-width")), gridWidth: Number(grid?.getAttribute("stroke-width") ?? 1), tickY };
  }, sel);

  test("#114/AK1, AK3, AK4: zero line thicker, at 0 with negative values, behind the data, none outside the range", async ({ page }) => {
    const errors = await open(page, "results/charts");
    await expect(page.locator("#rc-wealth line.ref")).toHaveCount(1);
    // Example with negative wealth: −0.5 to 3 million; the thick line is at the label «0»
    const r = await page.evaluate(() => {
      const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
      svg.style.width = "600px"; svg.style.height = "240px"; document.body.appendChild(svg);
      const values = Array.from({ length: 25 }, (_, i) => -0.5e6 + i * 3.5e6 / 24);
      Charts.time(svg, null, { x: values.map((_, i) => i * 12), xFmt: m => m / 12, yFmt: v => String(v / 1e6), lines: [{ color: "--c1", values }], tip: () => "" });
      const ref = svg.querySelector("line.ref"), zero = [...svg.querySelectorAll("text")].find(t => t.textContent === "0");
      const kids = [...svg.children], refAt = kids.indexOf(ref), lineAt = kids.findIndex(k => k.tagName === "path");
      // Range 1 to 3 million without 0 (as after a zoom): no reference line
      Charts.time(svg, null, { x: [0, 12, 24], xFmt: m => m, yFmt: String, range: [1e6, 3e6], lines: [{ color: "--c1", values: [1e6, 2e6, 3e6] }], tip: () => "" });
      const outside = svg.querySelectorAll("line.ref").length;
      svg.remove();
      return { y: Number(ref.getAttribute("y1")), zeroY: Number(zero.getAttribute("y")) - 4, width: Number(ref.getAttribute("stroke-width")), behind: refAt < lineAt, outside };
    });
    expect(r.width).toBeGreaterThanOrEqual(2);
    expect(Math.abs(r.y - r.zeroY)).toBeLessThan(0.5);
    expect(r.behind).toBe(true);
    expect(r.outside).toBe(0);
    // Bars (income and spending) and mini chart in the editor have the zero line as well
    await expect(page.locator("#rc-flows line.ref")).toHaveCount(1);
    await expect(page.locator("#chart line.ref")).toHaveCount(1);
    const flows = await refOf(page, "#rc-flows");
    expect(flows.width).toBeGreaterThan(flows.gridWidth);
    await noErrors(page, errors);
  });

  test("#114/AK2: stocks, bonds and inflation: line at 100 (linear and logarithmic), at 0 % in change in %", async ({ page }) => {
    const errors = await open(page, "results/charts");
    await expect(page.locator("#rc-mkt line.ref")).toHaveCount(1);
    // linear: 100 is in the middle between the labels 0 and 200
    let r = await refOf(page, "#rc-mkt");
    const mid = await page.evaluate(() => {
      const svg = document.querySelector("#rc-mkt"), y = l => Number([...svg.querySelectorAll("text")].find(t => t.textContent === l).getAttribute("y")) - 4;
      return (y("0") + y("200")) / 2;
    });
    expect(Math.abs(r.y - mid)).toBeLessThan(0.5);
    expect(r.width).toBeGreaterThanOrEqual(2);
    // logarithmic: the line is at the label «100»
    await page.locator('button[data-action="mkt-log"][data-v="1"]').click();
    await expect(page.locator("#rc-mkt line.ref")).toHaveCount(1);
    r = await refOf(page, "#rc-mkt");
    const at100 = await page.evaluate(() => { const t = [...document.querySelectorAll("#rc-mkt text")].find(x => x.textContent === "100"); return t ? Number(t.getAttribute("y")) - 4 : null; });
    expect(at100).not.toBeNull();
    expect(Math.abs(r.y - at100)).toBeLessThan(0.5);
    // Change in %: the line at 0 %
    await page.locator('button[data-action="mkt"][data-v="pct"]').click();
    await expect(page.locator("#rc-mkt line.ref")).toHaveCount(1);
    r = await refOf(page, "#rc-mkt");
    const at0 = await page.evaluate(() => { const t = [...document.querySelectorAll("#rc-mkt text")].find(x => /^0 ?%$/.test(x.textContent.trim())); return t ? Number(t.getAttribute("y")) - 4 : null; });
    expect(at0).not.toBeNull();
    expect(Math.abs(r.y - at0)).toBeLessThan(0.5);
    await noErrors(page, errors);
  });
});

// ---------------------------------------------------------------------------------------------
test.describe("Comparison like the single result (#115)", () => {
  /** Two Monte Carlo results (#runsflow), comparison open and visible. */
  const openCompare = async page => {
    const errors = await open(page, "runsflow/mem");
    await expect(page.locator("html")).toHaveAttribute("data-rendered", "compare");
    await page.evaluate(() => showResults(true));
    return errors;
  };
  const tab = (page, t) => page.locator(`button[data-action="cmp-tab"]`, { hasText: t }).click();

  test("#115/AK1: toggle nominal / today's purchasing power applies to key figures, charts and annual values, per result with its inflation", async ({ page }) => {
    const errors = await openCompare(page);
    await page.locator('#resultsTabs button[data-action="real"][data-v="1"]').click();
    await expect(page.locator("#resultsInfo")).toHaveText("heutige Kaufkraft");
    await expect(page.locator("#resultsBody .card h3").first()).toContainText("heutige Kaufkraft");
    // Key figure median real = final wealth of the median run in today's purchasing power, per result
    const exp = await page.evaluate(async () => {
      const metas = (await Runs.list()).filter(m => runsSel.has(m.id)).reverse();
      return Promise.all(metas.map(async m => { const d = await Runs.get(m.id); return fmtMio(d.result.batch.detailsReal.find(([q]) => q === 50)[1].summary.endTotalReal); }));
    });
    const rows = page.locator("#resultsBody table").first().locator("tr");
    for (let i = 0; i < exp.length; i++) await expect(rows.nth(i + 1).locator("td").nth(4)).toHaveText(exp[i]);
    // Annual values: total wealth at the end of the first year = nominal / price index of this month (median run)
    await tab(page, "Jahreswerte");
    const want = await page.evaluate(async () => {
      const m = (await Runs.list()).filter(m => runsSel.has(m.id)).reverse()[0], run = (await Runs.get(m.id)).result.batch.detailsReal.find(([q]) => q === 50)[1];
      const y = run.records[0].month.slice(0, 4), last = run.records.filter(r => r.month.startsWith(y)).pop();
      return fmtInt(last.total / last.priceIndex);
    });
    await expect(page.locator(".cmpyear tr").nth(1).locator("td").nth(1)).toHaveText(want);
    await tab(page, "Verläufe");
    await expect(page.locator("#resultsBody .card h3", { hasText: "Gesamtvermögen" })).toContainText("heutige Kaufkraft");
    await noErrors(page, errors);
  });

  test("#115/AK2: overview with range, final wealth by percentile, probability and age at depletion per result", async ({ page }) => {
    const errors = await openCompare(page);
    await page.locator('#resultsTabs button[data-action="real"][data-v="0"]').click();
    const titles = await page.locator("#resultsBody .card h3").allInnerTexts();
    for (const t of ["Kennzahlen", "Bandbreite des Gesamtvermögens", "Endvermögen nach Perzentil", "Wahrscheinlichkeit, dass das Geld reicht"])
      expect(titles.some(x => x.startsWith(t)), t).toBe(true);
    // Range: per result median, 5th and 95th percentile
    await expect(page.locator("#cmp-wealth path")).toHaveCount(6);
    await expect(page.locator("#cmp-surv path")).toHaveCount(2);
    // Percentiles: one column per result, median as in the summary
    const pt = page.locator("#resultsBody .card", { hasText: "Endvermögen nach Perzentil" }).locator("table");
    await expect(pt.locator("tr").first().locator("th")).toHaveCount(3);
    const med = await page.evaluate(async () => {
      const m = (await Runs.list()).filter(m => runsSel.has(m.id)).reverse()[0];
      return fmtInt((await Runs.get(m.id)).result.batch.summary.percentiles.find(p => p.percent === 50).endTotal);
    });
    await expect(pt.locator("tr", { hasText: /^50\./ }).locator("td").nth(1)).toHaveText(med);
    if (titles.some(x => x.startsWith("Alter bei Erschöpfung"))) await expect(page.locator("#cmp-dep rect").first()).toBeAttached();
    await noErrors(page, errors);
  });

  test("#126/AK2, #126/AK3, #129/AK2: final wealth by percentile in today's purchasing power shows every percentile per result, with notice", async ({ page }) => {
    const errors = await openCompare(page);
    const card = page.locator("#resultsBody .card", { hasText: "Endvermögen nach Perzentil" });
    // #265: per result (column) the real final wealth of the runs in the real order, rows descending as in the table
    const want = await page.evaluate(async () => {
      const metas = (await Runs.list()).filter(m => runsSel.has(m.id)).reverse();
      const col = (list, key) => list.slice().sort((a, b) => b.percent - a.percent).map(p => fmtInt(p[key]));
      return Promise.all(metas.map(async m => { const s = (await Runs.get(m.id)).result.batch.summary;
        return { real: col(s.percentilesReal, "endTotalReal"), nominal: col(s.percentiles, "endTotal") }; }));
    });
    expect(want).toHaveLength(2);
    for (let i = 0; i < want.length; i++) await expect(card.locator(`table tr td:nth-child(${i + 2})`)).toHaveText(want[i].real);
    await expect(card.locator(".hint")).toContainText("in heutiger Kaufkraft nach dem realen");
    await page.locator('#resultsTabs button[data-action="real"][data-v="0"]').click();
    for (let i = 0; i < want.length; i++) await expect(card.locator(`table tr td:nth-child(${i + 2})`)).toHaveText(want[i].nominal);
    await noErrors(page, errors);
  });

  test("#128/AK1, #128/AK2: an older result without real values per percentile shows the detail runs and a note", async ({ page }) => {
    const errors = await openCompare(page);
    // store the first result as it was saved before #126: real values only in the detail runs
    const { name, want } = await page.evaluate(async () => {
      const m = (await Runs.list()).filter(m => runsSel.has(m.id)).reverse()[0], d = await Runs.get(m.id);
      const b = structuredClone(d.result.batch);
      b.summary.percentiles.forEach(p => delete p.endTotalReal);
      delete b.summary.percentilesReal; delete b.detailsReal;
      await Runs.add(m, { ...d, result: { ...d.result, batch: b } });
      const det = new Map(b.details.map(([q, r]) => [q, fmtInt(r.summary.endTotalReal)]));
      return { name: m.name, want: b.summary.percentiles.slice().sort((x, y) => y.percent - x.percent).map(p => det.get(p.percent) ?? "–") };
    });
    expect(want.filter(v => v === "–").length).toBeGreaterThan(0);
    await page.locator('#resultsTabs button[data-action="real"][data-v="1"]').click();
    const card = page.locator("#resultsBody .card", { hasText: "Endvermögen nach Perzentil" });
    await expect(card.locator("table tr td:nth-child(2)")).toHaveText(want);
    await expect(page.locator(".cmpnote")).toContainText(`(älteres Ergebnis): ${name}`);
    await expect(page.locator(".cmpnote")).toContainText("nur für den schlechtesten Lauf");
    // nominal: no note
    await page.locator('#resultsTabs button[data-action="real"][data-v="0"]').click();
    await expect(page.locator(".cmpnote", { hasText: "nur für den schlechtesten Lauf" })).toHaveCount(0);
    await noErrors(page, errors);
  });

  test("#115/AK3: courses of the median run per result overlaid, liquid wealth by investment selectable", async ({ page }) => {
    const errors = await openCompare(page);
    await tab(page, "Verläufe");
    await expect(page.locator(".cmpnote").first()).toContainText("50. Perzentil (Median)");
    const titles = await page.locator("#resultsBody .card h3").allInnerTexts();
    for (const t of ["Liquides Vermögen nach Anlage", "Einnahmen und Ausgaben pro Jahr", "Entnahme aus dem Vermögen", "Vorsorge", "Aktien, Obligationen und Inflation"])
      expect(titles.some(x => x.startsWith(t)), t).toBe(true);
    await expect(page.locator("#cmp-liq path")).toHaveCount(2);
    await expect(page.locator("#cmp-mkt path")).toHaveCount(6);
    await expect(page.locator("#cmp-mkt line.ref")).toHaveCount(1);
    const d1 = await page.locator("#cmp-liq path").first().getAttribute("d");
    await page.locator('button[data-action="cmp-liquid"][data-v="stocks"]').click();
    await expect.poll(() => page.locator("#cmp-liq path").first().getAttribute("d")).not.toBe(d1);
    await page.locator('button[data-action="cmp-flow"][data-v="out"]').click();
    await expect(page.locator("#cmp-flows rect").first()).toBeAttached();
    await noErrors(page, errors);
  });

  test("#115/AK4: annual values with one column per result match the annual values of the single result", async ({ page }) => {
    const errors = await openCompare(page);
    await tab(page, "Jahreswerte");
    await expect(page.locator(".cmpyear tr").first().locator("th")).toHaveCount(3);
    for (const [key, col] of [["total", 5], ["draw", 7], ["receipts", 8]]) {
      await page.locator('select[data-action="cmp-year"]').selectOption(key);
      // Table «Jahreswerte» of the single result (median run) for the second result
      const single = await page.evaluate(async col => {
        const m = (await Runs.list()).filter(m => runsSel.has(m.id)).reverse()[1], run = batchDetails((await Runs.get(m.id)).result.batch).find(([q]) => q === 50)[1];
        const div = document.createElement("div");
        await deriveRuns([run]);
        div.innerHTML = yearTable(run);
        return [...div.querySelectorAll("table.yt tr")].slice(1).map(tr => tr.children[col].textContent);
      }, col);
      const cmp = await page.locator(".cmpyear tr").evaluateAll(trs => trs.slice(1).map(tr => tr.children[2].textContent));
      // The table of the single result leaves amounts below 0.5 empty
      const norm = v => ["0", "-0", "–", ""].includes(v) ? "" : v;
      expect(cmp.length).toBeGreaterThan(30);
      expect(cmp.map(norm)).toEqual(single.map(norm));
    }
    await noErrors(page, errors);
  });

  test("#115/AK3, AK4: select run: the selected percentile applies to the courses and annual values of all results", async ({ page }) => {
    const errors = await openCompare(page);
    await tab(page, "Verläufe");
    await expect(page.locator('button[data-action="cmp-p"][data-p="50"].on')).toBeAttached();
    const d50 = await page.locator("#cmp-total path").first().getAttribute("d");
    await page.locator('button[data-action="cmp-p"][data-p="5"]').click();
    await expect(page.locator(".cmpnote").first()).toContainText("5. Perzentil");
    await expect.poll(() => page.locator("#cmp-total path").first().getAttribute("d")).not.toBe(d50);
    // Annual values: the same selection; total wealth = table of the single result for the run of the 5th percentile
    await tab(page, "Jahreswerte");
    await expect(page.locator('button[data-action="cmp-p"][data-p="5"].on')).toBeAttached();
    const single = await page.evaluate(async () => {
      const m = (await Runs.list()).filter(m => runsSel.has(m.id)).reverse()[0], run = batchDetails((await Runs.get(m.id)).result.batch).find(([q]) => q === 5)[1];
      const div = document.createElement("div");
      await deriveRuns([run]);
      div.innerHTML = yearTable(run);
      return [...div.querySelectorAll("table.yt tr")].slice(1).map(tr => tr.children[5].textContent);
    });
    const cmp = await page.locator(".cmpyear tr").evaluateAll(trs => trs.slice(1).map(tr => tr.children[1].textContent));
    expect(cmp).toEqual(single);
    await noErrors(page, errors);
  });

  test("#115/AK5: single run with a later start: aligned by years since start, missing quantities with a note", async ({ page }) => {
    const errors = await openCompare(page);
    await page.evaluate(async () => {
      state.returns.source = "fixed";
      state.milestones.find(m => m.id === "start").date = "2027-03";
      await rememberRun(await Engine.call("run_simulation", { plan: state, path: null }));
      runsSel.clear(); (await Runs.list()).slice(0, 2).forEach(m => runsSel.add(m.id));
      compareTab = "overview"; await compareRuns();
    });
    await expect(page.locator(".cmpnote")).toContainText("Einzellauf ohne Perzentile");
    // Percentiles and probability only for the result with several runs
    await expect(page.locator("#resultsBody .card", { hasText: "Endvermögen nach Perzentil" }).locator("tr").first().locator("th")).toHaveCount(2);
    await expect(page.locator("#cmp-surv path")).toHaveCount(1);
    // Range: single run as one line, Monte Carlo with three
    await expect(page.locator("#cmp-wealth path")).toHaveCount(4);
    await tab(page, "Jahreswerte");
    await expect(page.locator(".cmpyear tr").first().locator("th").first()).toHaveText("Jahr seit Start");
    await expect(page.locator(".cmpyear tr").nth(1).locator("td").first()).toHaveText("1. Jahr");
    await expect(page.locator(".cmpyear tr").nth(1).locator("td").nth(1)).toHaveAttribute("title", /^202[67]$/);
    await noErrors(page, errors);
  });

  test("#115/AK6: zoom applies to all charts of the comparison and remains when switching the tab", async ({ page }) => {
    const errors = await openCompare(page);
    await page.locator('#resultsBody button[data-action="res-zoom"][data-years="10"]').first().click();
    await expect(page.locator('#resultsBody button[data-action="res-zoom"][data-years="10"].on').first()).toBeAttached();
    await tab(page, "Verläufe");
    await expect(page.locator('#resultsBody button[data-action="res-zoom"][data-years="10"].on').first()).toBeAttached();
    const xs = await page.locator("#cmp-total text").allInnerTexts();
    expect(xs.filter(t => / J\.$/.test(t)).length).toBeLessThanOrEqual(3);
    await noErrors(page, errors);
  });
});

// ---------------------------------------------------------------------------------------------
test.describe("Sort key figures in the comparison (#118)", () => {
  const openCompare = async page => {
    const errors = await open(page, "runsflow/mem");
    await expect(page.locator("html")).toHaveAttribute("data-rendered", "compare");
    await page.evaluate(() => showResults(true));
    return errors;
  };
  const kpi = page => page.locator("table.cmpkpi");
  const head = (page, t) => kpi(page).locator("th", { hasText: t });
  /** Colors of the rows from top to bottom (the color belongs to the result). */
  const colors = page => kpi(page).locator("tr td:first-child i.sw").evaluateAll(l => l.map(x => x.getAttribute("style")));
  /** Colors in the order of the median (ascending), computed from the saved results. */
  const byMedian = page => page.evaluate(async () => {
    const metas = (await Runs.list()).filter(m => runsSel.has(m.id)).reverse();
    const v = await Promise.all(metas.map(async (m, i) => ({ c: `background:var(${RUN_COLORS[i]})`, p: (await Runs.get(m.id)).result.batch.summary.percentiles.find(x => x.percent === 50).endTotal })));
    return v.sort((a, b) => a.p - b.p).map(x => x.c);
  });

  test("#118/AK1, AK2, AK3: click on the column header sorts ascending, then descending, with arrow; another column ascending again", async ({ page }) => {
    const errors = await openCompare(page);
    const asc = await byMedian(page);
    expect(asc[0]).not.toBe(asc[1]);
    const med = head(page, "Median");
    await med.locator("button").click();
    await expect(head(page, "Median")).toHaveAttribute("aria-sort", "ascending");
    await expect(head(page, "Median").locator(".sorti")).toHaveText("▲");
    expect(await colors(page)).toEqual(asc);
    await head(page, "Median").locator("button").click();
    await expect(head(page, "Median")).toHaveAttribute("aria-sort", "descending");
    await expect(head(page, "Median").locator(".sorti")).toHaveText("▼");
    expect(await colors(page)).toEqual([...asc].reverse());
    // the other columns show ↕ and are not sorted
    await expect(head(page, "Erfolgsquote")).toHaveAttribute("aria-sort", "none");
    await expect(head(page, "Erfolgsquote").locator(".sorti")).toHaveText("↕");
    await head(page, "5. Perzentil").locator("button").click();
    await expect(head(page, "5. Perzentil")).toHaveAttribute("aria-sort", "ascending");
    await expect(head(page, "Median")).toHaveAttribute("aria-sort", "none");
    await noErrors(page, errors);
  });

  test("#118/AK5: without a click the order of the list; sorting remains when switching tab and purchasing power, colors stay with the result", async ({ page }) => {
    const errors = await openCompare(page);
    await expect(kpi(page).locator('th[aria-sort="none"]')).toHaveCount(9);
    expect(await colors(page)).toEqual(["background:var(--r1)", "background:var(--r2)"]);
    const asc = await byMedian(page);
    await head(page, "Median").locator("button").click();
    await head(page, "Median").locator("button").click();
    const desc = [...asc].reverse();
    await page.locator('button[data-action="cmp-tab"]', { hasText: "Verläufe" }).click();
    await page.locator('button[data-action="cmp-tab"]', { hasText: "Übersicht" }).click();
    await expect(head(page, "Median")).toHaveAttribute("aria-sort", "descending");
    expect(await colors(page)).toEqual(desc);
    await page.locator('#resultsTabs button[data-action="real"][data-v="1"]').click();
    await expect(head(page, "Median")).toHaveAttribute("aria-sort", "descending");
    // the other cards (here: final wealth by percentile) keep the order of the list
    await expect(page.locator("#resultsBody .card", { hasText: "Endvermögen nach Perzentil" }).locator("th.cmph i.sw").first()).toHaveAttribute("style", /--r1\)/);
    await noErrors(page, errors);
  });

  test("#118/AK6: column headers usable by keyboard and with aria-sort", async ({ page }) => {
    const errors = await openCompare(page);
    await head(page, "Erfolgsquote").locator("button").focus();
    await page.keyboard.press("Enter");
    await expect(head(page, "Erfolgsquote")).toHaveAttribute("aria-sort", "ascending");
    await head(page, "Erfolgsquote").locator("button").focus();
    await page.keyboard.press(" ");
    await expect(head(page, "Erfolgsquote")).toHaveAttribute("aria-sort", "descending");
    await noErrors(page, errors);
  });
});

// ---------------------------------------------------------------------------------------------
test.describe("Name of a result with abbreviations (#119)", () => {
  test("#119/AK1, AK2, AK3: name plan · source · strategy · point in time, number of runs in the row below", async ({ page }) => {
    const errors = await open(page, "runsflow/mem/del");
    await expect(page.locator("html")).toHaveAttribute("data-rendered", "runs");
    const meta = await page.evaluate(async () => (await Runs.list())[0]);
    expect(meta.name).toMatch(/^.+ · MC hist\. · [^·]+ · \d{2}\.\d{2}\.\d{2}, \d{2}:\d{2}$/);
    expect(meta.name).not.toContain("Läufe");
    const strat = await page.evaluate(m => STRAT_SHORT[m.strategy], meta);
    expect(meta.name).toContain(` · ${strat} · `);
    await expect(page.locator(`#resultsBody tr:has(input[data-id="${meta.id}"]) small`)).toContainText("500 Läufe");
    // scenarios instead of source, other sources and strategies
    const t = await page.evaluate(() => [
      runTitle("P", { kind: "scenarios", source: "bootstrap", strategy: "vpw" }, "2026-11-16T13:05:00Z"),
      runTitle("P", { kind: "batch", source: "parametric", strategy: "three_buckets_guardrails" }, "2026-11-16T13:05:00Z"),
      runTitle("P", { kind: "single", source: "fixed", strategy: "fixed_rate" }, "2026-11-16T13:05:00Z"),
    ]);
    expect(t[0]).toMatch(/^P · Szenarien · VPW · /);
    expect(t[1]).toMatch(/^P · MC param\. · 3-Topf \+ Guard · /);
    expect(t[2]).toMatch(/^P · Fix · Fixe Rate · /);
    await noErrors(page, errors);
  });

  test("#119/AK4: every return source and every strategy has an abbreviation", async ({ page }) => {
    const errors = await open(page);
    const missing = await page.evaluate(() => [
      ...SOURCES.map(s => s[0]).filter(k => !SOURCE_SHORT[k]),
      ...[...STRATS.map(s => s[0]), "three_buckets_guardrails"].filter(k => !STRAT_SHORT[k]),
    ]);
    expect(missing).toEqual([]);
    await noErrors(page, errors);
  });

  test("#119/AK5: saved results keep their name", async ({ page }) => {
    const errors = await open(page, "runsflow/mem/del");
    await expect(page.locator("html")).toHaveAttribute("data-rendered", "runs");
    // Entry of an older version (name in the old structure), then a new result: the old name remains
    const old = "Alt · 500 Läufe · Monte Carlo · historisch · 01.11.26, 10:00";
    const id = await page.evaluate(async old => { const [m] = await Runs.list(); await Runs.update(m.id, { name: old }); return m.id; }, old);
    await page.evaluate(async () => { await rememberRun(await Engine.call("run_simulation", { plan: state, path: null })); await openRuns(); });
    const names = await page.evaluate(async () => (await Runs.list()).map(m => ({ id: m.id, name: m.name })));
    expect(names.length).toBe(2);
    expect(names.find(m => m.id === id).name).toBe(old);
    await noErrors(page, errors);
  });
});

// ---------------------------------------------------------------------------------------------
test.describe("Comparison with up to 10 results (#120)", () => {
  /** n results as copies of the first Monte Carlo result (with slightly different final wealth), all marked. */
  const withRuns = async (page, n) => {
    const errors = await open(page, "runsflow/mem/del");
    await expect(page.locator("html")).toHaveAttribute("data-rendered", "runs");
    await page.evaluate(async n => {
      const [m] = await Runs.list(), d = await Runs.get(m.id);
      for (let i = 1; i < n; i++) await Runs.add({ ...m, id: `kopie-${i}`, name: `Kopie ${i}`, created: new Date(Date.parse(m.created) + i * 1000).toISOString() }, d);
      await openRuns();
    }, n);
    await page.evaluate(() => showResults(true));
    return errors;
  };
  const selectAll = page => page.locator('input[data-action="runs-all"]').check();

  test("#120/AK1, AK2: compare active with 2 to 10 marked; from 11 blocked with notice, marking and deleting stay possible", async ({ page }) => {
    const errors = await withRuns(page, 11);
    const btn = page.locator('button[data-action="runs-compare"]');
    await expect(btn).toHaveAttribute("title", "Zwei bis 10 Ergebnisse markieren");
    await selectAll(page);
    await expect(btn).toBeDisabled();
    await expect(btn).toHaveText("Vergleichen (11)");
    await expect(page.locator(".cmpmax")).toHaveText("Höchstens 10 Ergebnisse lassen sich vergleichen.");
    await expect(page.locator('button[data-action="runs-delete"]')).toBeEnabled();
    await page.locator('input[data-action="run-sel"]').first().uncheck();
    await expect(btn).toBeEnabled();
    await expect(page.locator(".cmpmax")).toHaveCount(0);
    await noErrors(page, errors);
  });

  for (const scheme of ["light", "dark"]) {
    test(`#120/AK3: ten distinguishable colors, the same in all cards (${scheme})`, async ({ page }) => {
      await page.emulateMedia({ colorScheme: scheme });
      const errors = await withRuns(page, 10);
      await selectAll(page);
      await page.locator('button[data-action="runs-compare"]').click();
      await expect(page.locator("html")).toHaveAttribute("data-rendered", "compare");
      const rgb = await page.evaluate(() => RUN_COLORS.map(c => getComputedStyle(document.documentElement).getPropertyValue(c).trim()));
      expect(new Set(rgb).size).toBe(10);
      const hex = h => [1, 3, 5].map(i => parseInt(h.slice(i, i + 2), 16));
      let min = Infinity;
      for (let i = 0; i < 10; i++) for (let j = i + 1; j < 10; j++) min = Math.min(min, Math.hypot(...hex(rgb[i]).map((v, k) => v - hex(rgb[j])[k])));
      expect(min).toBeGreaterThan(50);
      // Key figures and percentiles: the same color per result
      const kpi = await page.locator("table.cmpkpi tr td:first-child i.sw").evaluateAll(l => l.map(x => x.getAttribute("style")));
      const pt = await page.locator("#resultsBody .card", { hasText: "Endvermögen nach Perzentil" }).locator("th.cmph i.sw").evaluateAll(l => l.map(x => x.getAttribute("style")));
      expect(kpi).toEqual(Array.from({ length: 10 }, (_, i) => `background:var(--r${i + 1})`));
      expect(pt).toEqual(kpi);
      await noErrors(page, errors);
    });
  }

  test("#120/AK4: with 10 results tables and bars scroll horizontally instead of shrinking", async ({ page }) => {
    await page.setViewportSize({ width: 1000, height: 900 });
    const errors = await withRuns(page, 10);
    await selectAll(page);
    await page.locator('button[data-action="runs-compare"]').click();
    await expect(page.locator("html")).toHaveAttribute("data-rendered", "compare");
    // page does not get wider
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth + 1)).toBe(true);
    await expect(page.locator("table.cmpkpi")).toHaveCount(1);
    expect(await page.locator("table.cmpkpi").evaluate(t => t.parentElement.classList.contains("tscroll"))).toBe(true);
    await page.locator('button[data-action="cmp-tab"]', { hasText: "Verläufe" }).click();
    const w = await page.locator("#cmp-flows rect:not(.hov)").evaluateAll(l => Math.min(...l.map(r => +r.getAttribute("width"))));
    expect(w).toBeGreaterThanOrEqual(2);
    expect(await page.locator("#cmp-flows").evaluate(s => s.parentElement.classList.contains("hscroll"))).toBe(true);
    for (const t of ["Jahreswerte", "Planung", "Übersicht"]) {
      await page.locator('button[data-action="cmp-tab"]', { hasText: t }).click();
      expect(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth + 1), t).toBe(true);
    }
    await noErrors(page, errors);
  });
});

// ---------------------------------------------------------------------------------------------
test.describe("Probability: axis 80–100 % or 0–100 % (#121)", () => {
  /** Lowest label of the y axis of a chart. */
  const lowest = (page, sel) => page.locator(`${sel} text[text-anchor="end"]`).evaluateAll(l => l.map(t => t.textContent).sort((a, b) => parseFloat(a) - parseFloat(b))[0]);
  const openSingle = async page => {
    const errors = await open(page, "runsflow/mem/view");
    await expect(page.locator("html")).toHaveAttribute("data-rendered", "saved");
    await page.evaluate(() => showResults(true));
    await page.locator('button[data-action="rtab"][data-tab="charts"]').click();
    await expect(page.locator("#mc-surv")).toBeAttached();
    return errors;
  };
  const openCompare = async page => {
    const errors = await open(page, "runsflow/mem");
    await expect(page.locator("html")).toHaveAttribute("data-rendered", "compare");
    await page.evaluate(() => showResults(true));
    return errors;
  };

  test("#121/AK1, AK2: default 80–100 % in the single result and in the comparison; the toggle applies in both and remains", async ({ page }) => {
    const errors = await openSingle(page);
    expect(await lowest(page, "#mc-surv")).toBe("80 %");
    await expect(page.locator('button[data-action="surv-scale"].on')).toHaveText("80–100 %");
    await page.locator('button[data-action="surv-scale"][data-v="0"]').click();
    expect(await lowest(page, "#mc-surv")).toBe("0 %");
    // Switch the tab and back: the choice remains
    await page.locator('button[data-action="rtab"][data-tab="overview"]').click();
    await page.locator('button[data-action="rtab"][data-tab="charts"]').click();
    expect(await lowest(page, "#mc-surv")).toBe("0 %");
    // the same choice applies in the comparison
    await page.evaluate(async () => { (await Runs.list()).forEach(m => runsSel.add(m.id)); return compareRuns(); });
    await expect(page.locator("#cmp-surv")).toBeAttached();
    expect(await lowest(page, "#cmp-surv")).toBe("0 %");
    await page.locator('button[data-action="surv-scale"][data-v="80"]').click();
    expect(await lowest(page, "#cmp-surv")).toBe("80 %");
    await noErrors(page, errors);
  });

  test("#121/AK3, AK4: cut off below 80 % with notice, tooltip with the real value", async ({ page }) => {
    const errors = await openSingle(page);
    await expect(page.locator(".survclip")).toHaveCount(0);
    // let the curve drop artificially to 60 %
    await page.evaluate(() => { const s = lastRun.batch.stats.survival; lastRun.batch.stats.survival = s.map((v, i) => i > s.length / 2 ? 0.6 : v); renderResults(lastRun); });
    await expect(page.locator(".survclip")).toContainText("tiefster Wert 60 %");
    expect(await page.locator("#mc-surv g.clipped path").count()).toBe(1);
    expect(await page.locator("#mc-surv clipPath rect").count()).toBe(1);
    // the chart must be in the visible area, otherwise the mouse does not reach it (the header row can wrap)
    await page.locator("#mc-surv").scrollIntoViewIfNeeded();
    const box = await page.locator("#mc-surv").boundingBox();
    await page.mouse.move(box.x + box.width - 20, box.y + box.height / 2);
    await expect(page.locator("#mc-surv-tip")).toContainText("60 %");
    await page.locator('button[data-action="surv-scale"][data-v="0"]').click();
    await expect(page.locator(".survclip")).toHaveCount(0);
    await noErrors(page, errors);
  });

  test("#121/AK3: notice in the comparison when a curve falls below 80 %", async ({ page }) => {
    const errors = await openCompare(page);
    await page.evaluate(async () => {
      const [m] = (await Runs.list()).filter(m => runsSel.has(m.id)), d = await Runs.get(m.id);
      d.result.batch.stats.survival = d.result.batch.stats.survival.map(v => Math.min(v, 0.7));
      await Runs.add({ ...m, id: "tief", name: "tief" }, d); runsSel.add("tief"); await compareRuns();
    });
    await expect(page.locator(".survclip")).toContainText("tiefster Wert 70 %");
    await noErrors(page, errors);
  });
});

// ---------------------------------------------------------------------------------------------
test.describe("Comparison: stocks, bonds and inflation like in the single result (#122)", () => {
  const openCharts = async page => {
    const errors = await open(page, "runsflow/mem");
    await expect(page.locator("html")).toHaveAttribute("data-rendered", "compare");
    await page.evaluate(() => showResults(true));
    await page.locator('button[data-action="cmp-tab"]', { hasText: "Verläufe" }).click();
    await expect(page.locator("#cmp-mkt")).toBeAttached();
    return errors;
  };
  const card = page => page.locator("#resultsBody .card", { has: page.locator("#cmp-mkt") });
  const labels = page => page.locator('#cmp-mkt text[text-anchor="end"]').allTextContents();

  test("#122/AK1, AK3: toggles performance / change in %, linear / logarithmic like in the single result", async ({ page }) => {
    const errors = await openCharts(page);
    await expect(card(page).locator('button[data-action="mkt"]')).toHaveText(["Wertentwicklung (Start = 100)", "Änderung in %"]);
    await expect(card(page).locator('button[data-action="mkt-log"]')).toHaveText(["Linear", "Logarithmisch"]);
    const lin = await labels(page);
    await card(page).locator('button[data-action="mkt-log"][data-v="1"]').click();
    const log = await labels(page);
    expect(log).not.toEqual(lin);
    // logarithmic: ticks 1-2-5 per power of ten, «100» is among them
    expect(log).toContain("100");
    await expect(page.locator("#cmp-mkt line.ref")).toHaveCount(1);
    await noErrors(page, errors);
  });

  test("#122/AK2, AK5, AK6: change in % per result like in the single result, reference line at 0, also in today's purchasing power", async ({ page }) => {
    const errors = await openCharts(page);
    await page.locator('#resultsTabs button[data-action="real"][data-v="0"]').click();   // #265: starts real
    await card(page).locator('button[data-action="mkt"][data-v="pct"]').click();
    await expect(card(page).locator("h3")).toContainText("Änderung gegenüber dem Vorjahresmonat in %");
    await expect(card(page).locator('button[data-action="mkt-log"]')).toHaveCount(0);
    expect((await labels(page)).every(t => t.endsWith("%"))).toBe(true);
    await expect(page.locator("#cmp-mkt path")).toHaveCount(6);
    // Tooltip in year 2: stocks = change over the last 12 months of the median run of the first result
    const want = await page.evaluate(async () => {
      const m = (await Runs.list()).filter(m => runsSel.has(m.id)).reverse()[0], r = batchDetails((await Runs.get(m.id)).result.batch).find(([q]) => q === 50)[1].records;
      return (100 * (r[18].stockIndex / r[6].stockIndex - 1)).toLocaleString("de-CH", { maximumFractionDigits: 1 });
    });
    const box = await page.locator("#cmp-mkt").boundingBox();
    const n = await page.evaluate(() => document.querySelector("#cmp-mkt path").getAttribute("d").split(/[ML]/).filter(Boolean).length);
    // Month 18 is at 18/(n-1) of the plot area (L = 60, R = 14 in the viewBox = width in px)
    await page.mouse.move(box.x + 60 + 18 / (n - 1) * (box.width - 74), box.y + box.height / 2);
    await expect(page.locator("#cmp-mkt-tip")).toContainText(`Aktien ${want} %`);
    // reference line at 0 %
    const r = await page.evaluate(() => { const l = document.querySelector("#cmp-mkt line.ref"), t = [...document.querySelectorAll("#cmp-mkt text")].find(x => /^0 ?%$/.test(x.textContent.trim()));
      return { y: +l.getAttribute("y1"), t: t ? +t.getAttribute("y") - 4 : null }; });
    expect(r.t).not.toBeNull();
    expect(Math.abs(r.y - r.t)).toBeLessThan(0.5);
    // today's purchasing power: values change, display stays
    const nominal = await page.locator("#cmp-mkt path").first().getAttribute("d");
    await page.locator('#resultsTabs button[data-action="real"][data-v="1"]').click();
    await expect(card(page).locator('button[data-action="mkt"][data-v="pct"]')).toHaveClass(/on/);
    expect(await page.locator("#cmp-mkt path").first().getAttribute("d")).not.toBe(nominal);
    await noErrors(page, errors);
  });

  test("#122/AK4: the choice applies jointly in the single result and in the comparison", async ({ page }) => {
    const errors = await openCharts(page);
    await card(page).locator('button[data-action="mkt"][data-v="pct"]').click();
    expect(await page.evaluate(() => mktPct)).toBe(true);
    // Open the single result: «Änderung in %» is selected there
    await page.evaluate(async () => { const [m] = await Runs.list(); await viewRun(m.id); });
    await expect(page.locator("html")).toHaveAttribute("data-rendered", "saved");
    await page.locator('button[data-action="rtab"][data-tab="runs"]').click();
    await expect(page.locator('button[data-action="mkt"][data-v="pct"]')).toHaveClass(/on/);
    await page.locator('button[data-action="mkt"][data-v="idx"]').click();
    expect(await page.evaluate(() => mktPct)).toBe(false);
    await noErrors(page, errors);
  });
});

// ---------------------------------------------------------------------------------------------
test.describe("Comparison: differences in the plan with labels (#123)", () => {
  const openPlan = async page => {
    const errors = await open(page, "runsflow/mem");
    await expect(page.locator("html")).toHaveAttribute("data-rendered", "compare");
    await page.evaluate(() => showResults(true));
    await page.locator('button[data-action="cmp-tab"]', { hasText: "Planung" }).click();
    await expect(page.locator("#resultsBody .card h3")).toContainText("Unterschiede in der Planung");
    return errors;
  };

  test("#123/AK1, AK2, AK3: label with section, values as in the editor, grouped by sections", async ({ page }) => {
    const errors = await openPlan(page);
    // runsflow differs in the stock share (30 %)
    const t = page.locator("table.cmpplan");
    const row = t.locator("tr", { hasText: "Aktienanteil" }).first();
    await expect(row.locator("td").nth(2)).toHaveText("30 %");
    await expect(row.locator("td").nth(1)).toHaveText(/^\d+(\.\d+)? %$/);
    // Section as a group row in front, no internal field names and no JSON
    const groups = await t.locator("tr.grp th").allTextContents();
    expect(groups.length).toBeGreaterThan(0);
    const order = ["Haushalt", "Meilensteine", "Vorsorge", "Einnahmen und Ausgaben", "Vermögen und Anlage", "Renditequelle und Zeitreihe", "Entnahmestrategie", "Immobilien", "Säule 3a", "Annahmen zur geschätzten Steuer", "Variationen"];
    expect(groups.map(g => order.indexOf(g))).toEqual([...groups.map(g => order.indexOf(g))].sort((a, b) => a - b));
    expect(groups.every(g => order.includes(g))).toBe(true);
    const text = await t.innerText();
    expect(text).not.toMatch(/\b[a-z]+[A-Z]?[a-z]*\.[a-z][A-Za-z]*\b/); // no «wealth.stockShare»
    expect(text).not.toMatch(/[{}]|"mode"/);
    await expect(t.locator("code")).toHaveCount(0);
    await noErrors(page, errors);
  });

  test("#123/AK2, AK4: selection fields as text, lists with names instead of index", async ({ page }) => {
    const errors = await openPlan(page);
    await page.evaluate(async () => {
      const [a] = (await Runs.list()).filter(m => runsSel.has(m.id)), d = structuredClone(await Runs.get(a.id)), p = d.plan;
      p.returns.source = "parametric";
      const pr = p.realestate.properties[0]; if (pr) pr.name = "Ferienwohnung Test";
      if (pr?.worth != null) pr.worth += 50000;
      await Runs.add({ ...a, id: "geaendert", name: "geändert" }, d); runsSel.clear(); runsSel.add(a.id); runsSel.add("geaendert"); await compareRuns();
    });
    const t = page.locator("table.cmpplan");
    const src = t.locator("tr").filter({ has: page.locator("td.l", { hasText: /^Renditequelle$/ }) });
    await expect(src.locator("td")).toHaveCount(3);
    expect((await src.locator("td").allTextContents()).slice(1).sort()).toEqual(["Monte Carlo · historisch", "Monte Carlo · parametrisch"]);
    const props = await page.evaluate(async () => (await Runs.get("geaendert")).plan.realestate.properties.length);
    if (props) await expect(t.locator("td", { hasText: /^Ferienwohnung Test: / }).first()).toBeAttached();
    expect(await t.innerText()).not.toMatch(/properties|[a-z]\.\d+\./);
    await noErrors(page, errors);
  });

  test("#123/AK5: every field of the example plan has a label or is explicitly hidden", async ({ page }) => {
    const errors = await open(page);
    const missing = await page.evaluate(() => {
      const base0 = JSON.parse(JSON.stringify(state)), ctxs = [];
      // Contexts: every return source × strategy (3-bucket also with guardrails), two persons married, own weighting,
      // milestones as date and as age, real estate as owned, planned and rented
      for (const src of SOURCES.map(x => x[0])) for (const st of [...STRATS.map(x => x[0]), "tbg"]) {
        const p = JSON.parse(JSON.stringify(base0)); p.returns.source = src;
        p.withdrawal.strategy = st === "tbg" ? "three_buckets" : st; if (st === "tbg") p.withdrawal.buckets = { ...(p.withdrawal.buckets || {}), spending: "guardrails" };
        ctxs.push(p);
      }
      const v = f => { const p = JSON.parse(JSON.stringify(base0)); f(p); ctxs.push(p); };
      v(p => { p.household.persons.push({ ...p.household.persons[0], id: "p2x" }); p.household.married = true; (p.milestones || []).forEach(m => { if (m.person) m.person = "p2x"; }); });
      v(p => { p.returns.series = null; p.returns.source = "bootstrap"; p.returns.strategy = { ...(p.returns.strategy || {}), stocks: "worldCustom" }; });
      v(p => (p.milestones || []).forEach(m => { if (m.date) m.kind = "date"; }));
      v(p => (p.milestones || []).forEach(m => { if (m.years != null) m.kind = "age"; }));
      for (const status of ["owned", "planned", "rent"]) v(p => (p.realestate?.properties || []).forEach(pr => { pr.status = status; }));
      const ENUM = ["repay", "renew", "saron", "fixed", "indirect", "direct", "date", "age", "owned", "planned", "rent", "wealth", "aargau", "zurich", "worldCustom"];
      // Field paths of the plan with their values (empty lists and objects as "[]" and "{}"); #209: no longer in app.js
      const flat = (o, p = "", out = {}) => {
        if (o && typeof o === "object") { const ks = Object.keys(o); if (!ks.length) out[p] = Array.isArray(o) ? "[]" : "{}"; for (const k of ks) flat(o[k], p ? `${p}.${k}` : k, out); }
        else out[p] = o;
        return out;
      };
      const covered = new Set(), keys = new Set();
      for (const base of ctxs) {
        const s0 = JSON.stringify(paramSections(base, true)), f = flat(base);
        for (const [k, val] of Object.entries(f)) {
          if (val === "[]" || val === "{}") continue;
          keys.add(k); if (covered.has(k)) continue;
          const cands = typeof val === "number" ? [val * 1.37 + 1] : typeof val === "boolean" ? [!val] : val === null ? [5, "2001-01"]
            : typeof val === "string" ? (isYM(val) ? ["1999-07"] : [val + "x", ...ENUM.filter(e => e !== val), "p2x"]) : [];
          for (const nv of cands) {
            const p = JSON.parse(JSON.stringify(base)); setPath(p, k, nv);
            try { if (JSON.stringify(paramSections(p, true)) !== s0) { covered.add(k); break; } } catch (e) { /* invalid value */ }
          }
        }
      }
      return [...keys].filter(k => !covered.has(k) && !PLAN_HIDDEN.some(([re]) => re.test(k)));
    });
    expect(missing).toEqual([]);
    await noErrors(page, errors);
  });
});

// ---------------------------------------------------------------------------------------------
test.describe("Header row of the result with the name (#125)", () => {
  const title = page => page.locator("#resultsTitle");

  test("#125/AK1, AK3: freshly computed with several runs: name · 500 runs", async ({ page }) => {
    const errors = await open(page);
    await page.evaluate(() => { state.returns.source = "bootstrap"; Object.assign(state.returns.bootstrap, { paths: 500, seed: 42 }); return runSimulation("overview", { noConfirm: true }); });
    await expect(title(page).locator(".rtkind")).toHaveText("500 Läufe");
    const name = await page.evaluate(async () => (await Runs.list())[0].name);
    expect(name).toContain("MC hist.");
    await expect(title(page).locator(".rtname")).toHaveText(name);
    await expect(title(page)).toHaveText(`${name} · 500 Läufe`);
    await expect(title(page)).toHaveAttribute("title", name);
    await noErrors(page, errors);
  });

  test("#125/AK2: single run: name in front, then single run · …", async ({ page }) => {
    const errors = await open(page, "results");
    await expect(title(page).locator(".rtname")).toBeAttached();
    const name = await page.evaluate(async () => (await Runs.list())[0].name);
    await expect(title(page).locator(".rtname")).toHaveText(name);
    await expect(title(page).locator(".rtkind")).toHaveText(/^Einzellauf/);
    await noErrors(page, errors);
  });

  test("#125/AK3, AK4: a saved result from the list shows its (new) name", async ({ page }) => {
    const errors = await open(page, "runsflow/mem/del");
    await expect(page.locator("html")).toHaveAttribute("data-rendered", "runs");
    const id = await page.evaluate(async () => { const [m] = await Runs.list(); await Runs.update(m.id, { name: "Variante Aktien 30 %" }); return m.id; });
    await page.evaluate(id => viewRun(id), id);
    await expect(page.locator("html")).toHaveAttribute("data-rendered", "saved");
    await expect(title(page)).toHaveText("Variante Aktien 30 % · 500 Läufe");
    // back to the list: header row without names and without tooltip
    await page.evaluate(() => openRuns());
    await expect(title(page)).toHaveText("Gespeicherte Ergebnisse");
    expect(await title(page).getAttribute("title")).toBeNull();
    await noErrors(page, errors);
  });

  test("#125/AK5: a long name is shortened, the number of runs stays visible, full name in the tooltip", async ({ page }) => {
    await page.setViewportSize({ width: 1000, height: 800 });
    const errors = await open(page, "runsflow/mem/del");
    await expect(page.locator("html")).toHaveAttribute("data-rendered", "runs");
    const long = "Sehr lange Bezeichnung einer Berechnung ".repeat(8).trim();
    const id = await page.evaluate(async long => { const [m] = await Runs.list(); await Runs.update(m.id, { name: long }); return m.id; }, long);
    await page.evaluate(id => viewRun(id), id);
    await expect(title(page)).toHaveAttribute("title", long);
    const r = await page.evaluate(() => { const n = document.querySelector("#resultsTitle .rtname"), k = document.querySelector("#resultsTitle .rtkind").getBoundingClientRect();
      return { cut: n.scrollWidth > n.clientWidth, overflow: getComputedStyle(n).textOverflow, right: k.right, w: window.innerWidth, kw: k.width }; });
    expect(r.cut).toBe(true);
    expect(r.overflow).toBe("ellipsis");
    expect(r.kw).toBeGreaterThan(0);
    expect(r.right).toBeLessThanOrEqual(r.w);
    await expect(title(page).locator(".rtkind")).toBeInViewport();
    await noErrors(page, errors);
  });

  test("#125/AK6: without a saved name the header row stays as before", async ({ page }) => {
    const errors = await open(page);
    await page.evaluate(() => { Runs.add = async () => { throw new Error("Speicher voll (Test)"); }; state.returns.source = "bootstrap"; Object.assign(state.returns.bootstrap, { paths: 500, seed: 42 }); return runSimulation("overview", { noConfirm: true }); });
    await expect(title(page)).toHaveText("500 Läufe");
    await expect(title(page).locator(".rtname")).toHaveCount(0);
    expect(await title(page).getAttribute("title")).toBeNull();
    // expected: only the reported storage error
    expect(errors.filter(e => !/Speicher voll/.test(e))).toEqual([]);
    expect((await page.evaluate(() => window.__errors)).filter(e => !/Speicher voll/.test(String(e?.message ?? e)))).toEqual([]);
  });
});
