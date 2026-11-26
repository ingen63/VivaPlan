# Reading the results

## @r-overview Results: overview
Keywords: success rate, key figures, percentiles, final wealth, depletion, result view

After “Start simulation”, the app shows a **progress bar** with the elapsed and the estimated remaining time and a **Cancel** button. Then the result view opens. Its header first names the **name of the result** as it is stored in the [list](#r-list) (for example ‘Beispiel · MC hist. · Standard · 16.11.26, 14:05’), followed by the type of run (‘50'000 runs’ or ‘Single run · …’). A long name is shortened; the full name appears when you point at it; after renaming in the list, the new name applies. The view has several tabs: **Overview**, **Charts**, **Individual runs** (only with several runs), **Scenarios** (only with variations) and **Yearly values**. All charts with a time axis can be zoomed: **drag a time range with the mouse button held down** (it is highlighted; selections of less than a year count as a click and are ignored), or use the buttons **All, 10 yrs, 5 yrs and 2 yrs**. The buttons set the width of the range **centered on the middle of the current range** (without zoom: of the whole period). If the range extends beyond the beginning or the end, it is shifted and the width stays; if the period is shorter, you see everything. After a mouse selection, no button is active, and the charts show “Custom” with the range. The zoom applies **to all result charts together**, in all tabs and in the comparison, and is kept when you switch tabs; the mini charts in the editor have their own zoom (from the start). The charts of the scenarios have no time axis and therefore no zoom. At the top right, you switch between **Nominal** and **Today's purchasing power**, see [Return, inflation and real values](#ret-concepts); when the results open, the switch is set to **Today's purchasing power**.

**Result in words:** At the very top, the overview summarizes the result in two or three sentences. With several runs: ‘In 87 % (medium) of runs, wealth lasts until 12.2055 (age 95.0).’ The rating in parentheses follows the colors of the success rate: **high** from 95 %, **medium** from 80 % to below 95 %, **low** below 80 %; what counts is the value displayed, rounded to 0.1 %. If the money runs out in a run, a second sentence names the age in the worst run (like “Earliest depletion”). With one run, it states whether wealth lasts until the end and how large it is then (nominal or in today's purchasing power, depending on the switch), or when it is depleted. Below that follows a suggestion for the next step: below 95 % or with depleted wealth the [optimization](#s-optimize), otherwise the [scenarios](#s-scenarios). The text is a reading aid, not an assessment of your situation.

**With a single run** (fixed returns, historical course), the overview shows five key figures: wealth at retirement, wealth at the end, how long the money lasts, the PK pension and the estimated taxes total ([How accurate is the tax?](#tax-accuracy)). Below follow the wealth over time and the list of events.

**With several runs** (Monte Carlo, all start dates), the overview shows:

- **Success rate:** share of the runs in which wealth lasts until the end. Green from 95 %, red below 80 %.
- **Final wealth** at the median, at the 5th, 1st, 95th and 99th percentile and in the **worst run**.
- **Earliest depletion:** the earliest age at which the money was used up in any run.
- the [range of total wealth](#r-fan) and the [percentile table](#r-percentiles).

**How much success rate is enough?** This is not a question of calculation but of your willingness to take risks. 90 % to 95 % is considered decent for cautious planning. Note that a success rate of 95 % means that the money runs out in 1 of 20 simulated futures. This is only an estimate, see [Pitfalls](#ret-pitfalls).

**‘Success’ means:** wealth (liquid and real estate) is not used up until the end of the simulation. It does **not** mean that life was comfortable: a run can be ‘successful’ and still end very tightly.

## @r-percentiles Final wealth by percentile
Keywords: percentile, median, 1st percentile, 99th percentile, worst run, quantile

The runs are sorted by their **final wealth**. The **nth percentile** is the final wealth below which n % of the runs lie.

| Row | Meaning |
|---|---|
| Worst run | the run with the lowest final wealth |
| 1st percentile | only 1 % of the runs end worse, ‘a very bad case’ |
| 5th percentile | only 5 % end worse, ‘a bad case’ |
| 50th (median) | half end better, half worse |
| 95th and 99th percentile | very good cases |

For every row, the table names the **run**, for example the historical start month or the number of the path. A **negative** final wealth means that the money **ran out before the end** and debts (unpaid spending) accumulated afterwards.

**Order by the view:** The runs are ordered by the final wealth of the chosen view: in **today's purchasing power** by the real, **nominal** by the nominal final wealth. Inflation differs from run to run; a run with high inflation ends high nominally but low in real terms. A percentile may therefore belong to a **different run** in each view, but in both views the values rise with the percentile. Key figures, percentile table, property sales, [single runs](#r-runs) and yearly values always show the runs of the chosen order; switching orders them anew. Older saved results without this order show, in today's purchasing power, the real final wealth of the runs in the nominal order and a notice; computed again, they are ordered in real terms.

> The **median** is **not** a realistic normal case in the sense of a forecast. It only says that half of the simulated futures turn out better. For planning, the **lower** percentiles count, because that is where the risk lies.

## @r-fan Range of total wealth
Keywords: fan, band, percentiles, range, worst value, cone, median

The chart shows how **total wealth** is **distributed** over time. In every month, these are formed across all runs:

- the **dark area**: 25th to 75th percentile (the middle half of the runs)
- the **light area**: 5th to 95th percentile (nine out of ten runs)
- the **solid line**: median
- the **dashed lines**: 1st and 99th percentile (in the legend with a dashed line symbol)
- the **red line**: the **worst value** of the month

The red line is the lowest value of all runs per month. This can be a different run from month to month, so it is not a single path but the lower bound of all paths. With the mouse, you see the values in the tooltip.

**Reading:** The range opens up over time like a funnel. The lower the lower band, the greater the danger. If the 5th percentile touches the zero line, the money runs out in more than 5 % of the runs.

## @r-survival Probability that the money lasts
Keywords: survival, probability of success, depletion, course

The share of runs in which wealth has not yet been used up **by this point in time**. The curve only falls, it never rises. At the right edge is the success rate.

A curve that falls **early** indicates risk already in the first years of retirement (sequence risk). A curve that only falls late is a longevity risk. By default, the axis shows **80–100 %**, where most curves lie and differences become visible; note the scale. The switch above the chart changes to **0–100 %**. The choice applies in the single result and in the [comparison](#r-compare) until you change it. If a curve falls below 80 %, it is cut off at the bottom with 80–100 %, and a notice below the chart names the lowest value. The tooltip always shows the real value.

## @r-hist Distribution of final wealth
Keywords: histogram, distribution, frequency, marks, retirement

A **histogram**: every bar counts the runs whose final wealth falls into this class. The dashed marks show the 1st, 5th, 50th, 95th and 99th percentile (legend below the chart). The range extends from the 1st to the 99th percentile; worse or better runs are in the edge classes. The worst run is noted below.

A **skewed** distribution with a long right tail is typical: a few runs end very well, most in between. The lower end shows how bad it can get.

The second chart shows the distribution of **wealth at retirement**. It is the starting point for the withdrawal phase: low wealth at retirement often decides the result in advance.

## @r-depletion Age at depletion
Keywords: depletion, depleted, age, bars

For every year of age, the number of runs in which wealth is used up **in that year**. The chart only appears if there are any depletions at all.

Useful for putting the risk into perspective by age: if the money runs out only rarely and only after 90, that is a different risk than if it happens at 75.

## @r-charts Charts
Keywords: diagrams, tab, chart

The tab “Charts” shows the diagrams in full size:

- With **one run:** [Wealth over time](#r-wealth), [Liquid wealth by asset class](#r-comp), [Income and spending](#r-flows), [Withdrawal from wealth](#r-draw), [Real estate](#r-prop), [Pension assets](#r-pk) and [Stocks and bonds](#r-markets).
- With **several runs:** [Range](#r-fan), [Probability that the money lasts](#r-survival), [Distribution of final wealth](#r-hist) and [Age at depletion](#r-depletion).

Every chart has an ⓘ at its heading. With **F1** over a chart, you get the matching explanation.

**Thick line at 0:** In all charts with a value axis that includes 0, the line at 0 is thicker than the rest of the grid, also in the comparison and in the mini chart in the editor. Below this line, the money is used up. In the [distribution of final wealth](#r-hist), it is vertical at a final wealth of 0. If 0 is outside the range shown, the line is missing.

## @r-runs Individual runs
Keywords: run, detail, percentile run, worst run, path, single run

With several runs, you can look at **individual runs** in detail. Choose which one at the top:

- **Worst run**, **1st**, **5th**, **50th**, **95th** or **99th percentile**: the run whose **final wealth** corresponds to this percentile, in the order of the chosen view (see [Final wealth by percentile](#r-percentiles)).

Below, you see the charts of this run that a single run shows, and the list of events. This way, you recognize **why** a run ends badly: a stock market crash at the beginning, high inflation, an emergency sale of the property.

The **comparison** of the 5th with the 95th percentile is particularly instructive: how do the returns of the first ten years of retirement differ? That is the **sequence risk** in numbers. The app recalculates the detail runs exactly.

## @r-wealth Wealth over time
Keywords: total wealth, stacked, liquid, real estate, pension assets

The **total wealth** of a run as a stacked area of three parts:

- **Liquid wealth** (stocks, bonds, cash)
- **Real estate (net)**, i.e. market value minus mortgage
- **Pension assets** (pension fund, pillar 3a)

Vertical lines mark events (retirement, AHV start, purchase, sale, depletion). The mouse pointer shows the values of the month. **Jumps** usually have a reason: the lump-sum withdrawal from the pension fund shifts pension assets into liquid wealth, and a house sale shifts real estate into liquid wealth.

## @r-comp Liquid wealth by asset class
Keywords: stocks, bonds, cash, shares, composition

How **liquid wealth** is split into stocks, bonds and cash. With the buttons at the top, you show **Amounts** or **Shares in %**.

The share shows whether the strategy works as you expect. With the [3-bucket strategy](#f-wd-three_buckets), you see the buckets; with “Standard”, the fixed stock allocation and the reserve.

## @r-flows Income and spending per year
Keywords: income, spending, taxes, housing costs, living expenses, bars

Two bars per **calendar year**:

- **on the left, income:** earned income, PK pension, AHV
- **on the right, spending:** living expenses, housing costs, estimated taxes

Where the right bar is higher than the left one, **wealth is drawn on**. The first and the last year are incomplete, because the simulation begins or ends in the middle of the year.

The chart shows well how the pensions close the gap from retirement and how large it still is.

## @r-draw Withdrawal from wealth
Keywords: withdrawal rate, sustainable, 4 percent, rule of thumb, withdrawal

Per year: **spending including taxes minus income**, relative to liquid wealth. The line shows the **withdrawal rate**.

A rule of thumb says that a rate **below 4 %** is sustainable. This only applies roughly and for US conditions over 30 years. If the rate rises considerably over the years because wealth shrinks or spending grows, that is a warning sign. With the strategies that follow wealth (fixed rate, VPW, …), you see their effect here.

## @r-prop Real estate
Keywords: market value, mortgage, equity, house

The real estate of a run: **market value**, **mortgage** and **equity** (market value minus mortgage). The chart only appears if there is real estate. Jumps show a purchase, a sale or the repayment of a mortgage.

## @r-pk Pension assets
Keywords: pension fund, pillar 3a, balance

The course of the **pension fund** and **pillar 3a** of a run. The pension fund grows through contributions and interest until the payout and then drops to the pension share. Pillar 3a is only visible with indirect amortization. The chart only appears if there are pension assets.

## @r-markets Stocks and bonds
Keywords: index, return, course, market course, price index, inflation, logarithmic, percent

Shows the **course of returns of this run** as an index, **start = 100**, with distributions reinvested. **Inflation** (price index) runs along like stocks and bonds, dashed. The tooltip additionally shows the change over the last twelve months, for inflation in parentheses.

Two switches change the display: **Performance (start = 100)** or **Change in %** (compared with the same month of the previous year). With the performance, you choose **Linear** or **Logarithmic**; logarithmically, equal distances show equal percentage changes. The percentage view is always linear. The thick line is at **100** (level at the start) for the performance and at **0 %** for the change in %.

This way, you recognize **what happened on the market in this run**: a crash at the start, a long sideways phase or high inflation. This often explains why a run ends badly or well.

## @r-params Parameters
Keywords: parameters, summary, inputs, state of the run, reproduce

The tab **“Parameters”** shows, **read-only**, which inputs the result was calculated with: household (month of birth, no names), milestones, pensions, income and spending, wealth and investment, return source and time series (with random seed), withdrawal strategy with its parameters, real estate, tax assumptions and variations. What applies is the **state of the run**, not the current state in the editor: if you change a number after the calculation, the tab of the calculated result stays unchanged. Saved results show the plan as of that time; if it is missing, “Not available” is shown.

## @r-events Events
Keywords: list, purchase, sale, emergency sale, retirement, depletion

The list of all **events** of a run with date and age: retirement, AHV start, lump-sum withdrawal with lump-sum tax, purchases and sales of real estate (also **emergency sales**), mortgage periods, the **switch from indirect to direct amortization** at the AHV pension start, the **one-off repayment** at a purchase when the mortgage has to be reduced for affordability, payout of pillar 3a, one-off amounts and the **depletion** of wealth. The **transfers of the 3-bucket strategy** (refills, inflows, reservations of sale proceeds) are not in this list but can be expanded in the [yearly values](#r-table); the messages of the deficit phase (“all buckets empty”) remain here.

All amounts are shown **with currency** (project currency).

The **withdrawal event** of strategies such as Guardrails shows the cuts and increases. Emergency sales and depletions are signals that the plan does not hold in this run.

## @r-property Real estate (events)
Keywords: purchase, sale, emergency sale, mortgage renewal, one-off payment, affordability, interest rate, equity

In the overview of a single run (and in “Individual runs”), the section **“Real estate”** lists all real estate events in chronological order, with the point in time (year and age), event, property and the amounts:

- **Purchase:** purchase price, equity (price − mortgage) and mortgage.
- **Sale / emergency sale:** sale proceeds, mortgage paid off, property gains tax and **net proceeds**; for an emergency sale, the reason (“Wealth < 0” or “Emergency sale criterion”), plus a note whether the proceeds were reserved for a purchase in the following month or the [withdrawal strategy](#s-withdrawal) was applied immediately.
- **Mortgage renewal:** affordability, mortgage before and after, one-off payment to restore affordability and interest rate of the new period; if the mortgage is not renewed, the emergency sale with its reason is shown.

Without events, “No real estate events” is shown there. With several runs, the key figure **“Emergency sales”** in the overview shows in how many runs (number and share, of which due to lack of affordability) real estate was sold unplanned. If [Never sell properties](#f-re-never-sell) is switched on, the key figure **“Unaffordable renewals”** is shown next to it: in how many runs (number and share) a mortgage was not affordable at renewal and was renewed in full only because of the switch. A paid one-off payment does not count. The column **“Property sales”** of the percentile table names, for the run of each row, **every sale of a property** with the month (‘Haus: 05.2034’, below it ‘Wohnung: 03.2048’); emergency sales carry the warning symbol (house with a warning sign), planned sales do not; without a sale, ‘–’ is shown.

## @r-scenarios Scenarios
Keywords: variation, heatmap, table, parameter, success rate per variant

The scenarios tab shows the effect of the [variations](#f-sweep): every row of the table is a **combination** of parameter values with which **all return paths** were calculated. The columns show the success rate, wealth at retirement (median), final wealth (5th, 50th, 95th percentile) and the earliest depletion.

- With **one** parameter, two charts show the success rate and the final wealth per value.
- With **two** parameters, a **heatmap** results: the color shows the success rate, so you find the safe area at a glance.

Above the charts and in every chart title, it says **which variation** is shown.

**Single run with variations:** If you calculate with a source without randomness (for example fixed returns) and variations, **“Choose variation”** appears above the tabs. There, you choose a step (or the plan value) for each variation; charts, key figures and yearly values then show the run of this step. “Optimum” names the best value according to the same criteria as with several runs (below); **“Show optimum”** selects the corresponding step. There are no percentiles in a single run; the optimum applies to the one run.

**Step with Monte Carlo and historical runs:** With several runs and variations, a selection field per variation with the calculated steps (for example ‘Stock allocation 30 %’, ‘Lump sum 60 %’) and “Plan value” appears at the top of **“Individual runs”** and **“Yearly values”**. The two tabs then show the runs of this step, i.e. the same runs (worst, 1st, 5th, 50th, 95th, 99th percentile, in today's purchasing power in the real order) on which the table under “Scenarios” is based. For this, the app only recalculates these six runs, not all paths; switching takes a few seconds. The selection applies to both tabs together. The other tabs continue to show the plan values and carry the note “Plan values, not the selected variation”. With saved results, the selection works the same way. If a recalculated final wealth differs from the table (for example because a time series has changed), a notice appears. A step that was not calculated (plan value not in the grid) cannot be selected.

**Optimal value per percentile:** In the overview (final wealth by percentile) and in “Individual runs”, every percentile shows the value of the variation that is optimal for this percentile. Only the variation steps already calculated are used for this.

- **Spending factor:** the **highest** factor at which wealth lasts until the end. If the transition lies between two steps, it is interpolated until the final wealth is exactly 0 (display ‘≈ 73 %’).
- **Stock allocation, pension fund lump sum:** Values at which wealth lasts take precedence; among them, the **highest final wealth** applies. If it lasts nowhere, the value with the latest age at depletion applies (“(does not last in any run)”). There is no interpolation here.
- **Two variations:** If the spending factor is one of them, the highest sustainable factor is determined for every value of the other parameter, and the combination with the highest factor is shown. Otherwise, the best combination according to the criteria above applies.
- **Edge of the range:** ‘< 50 %’ means that even the lowest value examined is not enough; ‘≥ 110 %’ (or ‘≥’ / ‘≤’ for the best value) that the range is not sufficient. Then extend the variation. For a share (stock allocation, pension fund lump sum), 0 % or 100 % is shown without ‘≥’ or ‘≤’, because that is the natural limit, not a range that is too narrow; if the best value of a share is at another edge (for example 50 % with a range of 0 to 50 %), ‘≥’ or ‘≤’ still applies.
- **Historical start month, retirement date, withdrawal rate:** No optimum makes sense here (the start month is not a decision; a later retirement date or a lower rate always gives the highest final wealth). ‘–’ is shown.

All variants use **the same random paths**. Differences therefore come from the parameter, not from chance.

## @r-table Yearly values
Keywords: table, year, export, numbers, withdrawal, deviation from plan, taxes, spending, rent, mortgage, percentile, lump sum, property gain, taxable income

A table with **one row per year** for the chosen run. Amounts in the project currency, nominal or in today's purchasing power depending on the switch. With several runs, you choose the run at the top (worst run, 1st, 5th, 50th, 95th, 99th percentile); the default is the median.

| Columns | Meaning |
|---|---|
| Liquid, Real estate, Pension assets, Total | **wealth at year end**: liquid assets, real estate (net), pension assets (PK, 3a), sum |
| Total spending | **monthly average of the year**: planned spending (living expenses from your plan) plus **rent or mortgage interest**, amortization and other housing costs of the real estate plus **pillar 3a contributions** |
| Care home | Only with a [care home](#f-care-home): monthly average of the care home costs of the year, included in “Total spending” |
| Withdrawal | monthly average of the year: what is **withdrawn** from wealth (actual spending minus income, before taxes) |
| Total income | monthly average of the year: income (earned income and pensions) plus withdrawal |
| Withdrawal rate | **annual withdrawal / total wealth at the start of the year** in percent (column “Total” of the previous year) |
| Deviation from plan | monthly average: total income minus total spending. At **0**, everything goes as planned. With strategies that link spending to wealth (Guardrails, fixed rate, VPW, endowment), a **negative** value is the cut compared with your plan, a positive one the increase |
| Inflation per year | change in the price index in the calendar year, in the first year since the simulation start |
| Taxable income | appears with the tax columns **directly before the taxes**: taxable income of the **tax year** for the canton, i.e. income (net earned income, pension fund pension, AHV, [taxable yield](#f-tax-yield)) minus the [deductions](#f-tax-deductions), never negative; lump-sum withdrawals do not count. If the federal amount differs, the tooltip of the cell shows both values. This shows why the tax changes with the same “Total income” (deductions, yield, tariff, place of residence). Years without an assessment (the last year if the plan ends before December) show 0 |
| Taxes paid in the year | **annual sums of the taxes paid** by type: income tax, wealth tax, lump-sum tax, property gains tax, plus the total. The columns are **collapsed by default**; the button **“Show taxes”** above the table shows and hides them, and the choice is kept. The column headers read **“Estimated …”**; their tooltip mentions the rounding to 100 francs |
| Tax value properties | appears with the tax columns if a property has a [tax value](#f-re-tax-value): sum of the tax values on 31 December (properties without a tax value at market value); the wealth tax uses it minus the mortgage |
| Wealth deductions, Taxable wealth | appear with the tax columns if the plan has [deductions from wealth](#f-tax-wealth-deductions): deductions on 31 December with their origin in the tooltip and wealth minus deductions, never below 0 (in the mode “place of residence” the social deductions of the canton are added) |

The columns are not arranged in groups with headings. That wealth figures are **balances at year end** and the cash flows are **monthly averages** is stated in the hint (tooltip) of the column headers and in the text below the table. The table does not show income alone or the planned spending without housing costs; they are contained in “Total income” and “Total spending”.

- **Withdrawal rate:** The annual withdrawal is the sum of the monthly withdrawals of the calendar year (for incomplete years, the monthly average times 12). The wealth at the start of the year is the total wealth (“Total”) of the previous year, in the first year the balance at the simulation start. The calculation always uses **nominal** values; the switch nominal/today's purchasing power does not change the rates. If the withdrawal is zero or negative (income above spending), the cell stays empty; if wealth is zero or negative, ‘–’ is shown.
- **3-bucket strategy:** Every year with at least one refill has an arrow (▸) in front. It expands the **derivation of the bucket sizes and refills** of this year (trigger, spending basis, regular income, net need, targets, return criterion, transfers, bucket balances before and after). With several runs, it applies to the chosen single run.
- The cash flows are **monthly averages** of the calendar year, because individual months fluctuate (mortgage interest, for example, is paid quarterly).
- Income and wealth tax are assessed in December and paid in **May of the following year**. The table shows them in the row of the **tax year** they apply to: the tax of the tax year 2028 is in row 2028, even though the money only flows out in May 2029. Wealth and the cash balance therefore only drop in the following year. If the plan ends before December, the tax of the last, incomplete year is missing from the columns. Lump-sum tax (pension fund, pillar 3a) and property gains tax are shown in the year of the payout or sale.
- The taxes are **estimated** and **rounded to 100 francs** (each cell on its own, so the total may differ by a few hundred francs from the sum of the rounded columns). Below the table is “Estimated tax with your own tables, rounded to 100 francs. No tax advice.” Check lump-sum tax and property gains tax with the canton’s tax calculator or a professional before deciding, see [How accurate is the tax?](#tax-accuracy).
- The first and the last year are usually incomplete; they carry an asterisk (*), and the mouse pointer names the number of months.

## @r-list List of results
Keywords: history, list, saved, rename, note, delete, results, simulations

After it is started, every simulation is **stored automatically** as a result, with the **plan as of that time** and the **complete result**. With the button **“Results”** in the header, you open the list.

- **Result:** For Monte Carlo runs, it shows ‘**success rate · 5 %: amount**’ (5th percentile of final wealth); the median appears when you point at it. Single runs show the final wealth. The **warning symbol (house with a warning sign)** appears if a property was sold unplanned (emergency sale, also due to lack of affordability at the mortgage renewal; planned sales do not count). With several runs, it applies from **5 % of the runs** with an emergency sale, and the tooltip names the share and number of runs; with a single run, at every emergency sale. Entries from older versions show no symbol.
- **Time series:** name of the time series the result was calculated with (for fixed returns “fixed returns”, for the parametric source “parametric”). Entries from older versions show ‘–’. The type (single run, scenarios, number of runs) is shown in small print below the result.
- **Strategy:** If you move the mouse over the name (or go there with the Tab key), the **parameters of the strategy** as of the result appear. Older entries show “Parameters not available”.
- **Name and note:** Can be edited directly in the row. Default name: project · return source · strategy · point in time, for example ‘Beispiel · MC hist. · 3 buckets + guard · 16.11.26, 14:05’ (with scenarios, ‘Scenarios’ instead of the source). Abbreviations of the return source: **Fixed** (fixed returns), **Hist.** (historical from start date), **All starts** (all historical start dates), **MC hist.** (Monte Carlo · historical), **MC param.** (Monte Carlo · parametric). Abbreviations of the strategy: **Standard**, **3 buckets**, **3 buckets + guard** (3 buckets with guardrails), **Guardrails**, **Fixed rate** (fixed withdrawal rate), **VPW**, **Endowment**. The number of runs is shown in small print below the result. Results from older versions keep their name.
- **Open:** Shows the saved result with all tabs. The **plan as of that time** applies; a notice at the top names the date. **“Load plan into the editor”** loads this plan back into the editor.
- **Delete** removes a result after a confirmation, **“Delete marked”** several and **“Delete all”** all. The checkbox in the header marks all entries.
- **Optimizations** and strategy comparisons appear in the list too, see [Saving and reopening the result](#o-saved). “Compare” counts only simulations and leaves marked optimizations out.
- **Compare:** Mark two to ten entries and click **“Compare”**, see [Comparison](#r-compare). If more than ten are marked, “Compare” is blocked and a notice appears; deleting still works.

The list is stored **locally in the storage of the app**, not in the plan file. It grows until you delete entries: an entry with many runs takes up a few megabytes. If the storage is full, the message “Result not saved in the list” appears; the result stays displayed but is not stored. Then delete older entries.

## @r-compare Comparing results
Keywords: comparison, side by side, differences, plan, key figures, yearly values, purchasing power

The comparison shows up to ten saved results side by side, each in its own color. With many results, wide tables and bar charts can be scrolled horizontally. It offers the same possibilities as a single result, spread over four tabs:

- **Overview:** key figures (time series, strategy, success rate, final wealth at the median, at the 5th and 1st percentile and in the worst run, earliest depletion; a click on a column header sorts the rows in ascending order, another in descending order, and the arrow ▲/▼ shows the direction; missing values are always at the end, and “none” for depletion counts as the latest value), the **range of total wealth** (median solid, 5th percentile dashed, 95th percentile dotted), the **final wealth by percentile** as a table, the **probability that the money lasts** (axis 80–100 % or 0–100 %, see [there](#r-survival)), and the **age at depletion** as a share of the runs (so that results with different numbers of runs stay comparable).
- **Trajectories:** the charts of the single result for one run per result: for a single run that run, for several runs the run of the percentile you set under **“Choose run”** (as under “Individual runs”; default: median). The selection applies to all results and also to the yearly values; if a result lacks the percentile, it shows the median run with a note. Total wealth, liquid wealth (selectable: total, stocks, bonds, cash), income, spending or taxes per year, withdrawal rate, real estate (equity), pension assets, and stocks, bonds and inflation (stocks solid, bonds dashed, inflation dotted). This chart has the same switches as in the single result: **Performance (start = 100)**, linear or logarithmic, or **Change in %** compared with the same month of the previous year; the choice applies in both views.
- **Yearly values:** for the chosen run, one row per year and one column per result for a selectable measure (total wealth, liquid wealth, real estate, pension assets, income, spending, withdrawal, withdrawal rate, taxes). The values are the same as in the [yearly values](#r-table) of the individual results.
- **Plan:** a table of all inputs that differ between the results, with the labels and units of the editor (for example ‘Stock allocation: 30 %’ instead of an internal field name) and grouped by the sections of the editor. Entries in lists carry their name (for example ‘Ferienwohnung: market value’). Time-dependent values appear as a list (‘from 01.2030: …’). This way, you see exactly **what** led to a different result.

At the top right, you switch between **Nominal** and **Today's purchasing power**; every result is converted with its own inflation. As in the single result, the runs of the percentiles are ordered by the final wealth of the chosen view, in today's purchasing power by the real one (see [Final wealth by percentile](#r-percentiles)); older results without this order are named in a note. As in the single result, the zoom applies to all charts with a time axis.

The results are aligned by **years since their start**. If they start in different calendar years, the yearly values show ‘Year 1’, ‘Year 2’ and so on, and the calendar year is in the tooltip. If a result lacks a measure, it is missing from that chart or column, and a note names it: a single run appears in the range as one line and has no percentiles, no probability and no age at depletion; a result without real estate or without pension assets is missing from these charts; a result with only scenarios appears only as a key figure; older results without values in today's purchasing power are missing from the range; results from version 1.0.0-beta.1 and older only have the final wealth in today's purchasing power for the worst run and at the 1st, 5th, 50th, 95th and 99th percentile.

Preferably compare results that differ in **one** point (for example stock allocation or strategy).
