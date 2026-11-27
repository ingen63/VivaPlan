# Editor: sections and fields

## @s-general Household & project
Keywords: name, currency, persons, birth, second person, double income, simulation period

Here you set the basic data.

- **Project name and currency:** All amounts are entered in the project currency (CHF, EUR or USD). The indices of the [investment strategy](#f-ret-series) are in CHF; with a plan in EUR or USD, a notice appears.
- **Person 1:** Name and month of birth (format MM.YYYY). From the month of birth, the app calculates the age for milestones (‘at 65’) and for the labels of the charts.
- **+ Second person:** Adds a second person with their own income, their own pension fund and their own AHV (double income). Taxes, wealth and spending apply to the household. See [Second person and rules for couples](#f-couple).
- **Without pension fund (large pillar 3a):** The switch is in the section [Pensions](#s-pension) for each person, at the top of the card [Pension fund](#f-pk).
- **Simulation period:** Start and end are milestones and are changed under [Milestones](#s-milestones).

All dates have the format **MM.YYYY** (for example 05.2027).

You enter **numbers** with or without thousands separators: apostrophe or space (1'234 or 1 234), and point or comma as the decimal separator (12.5 or 12,5). For **amounts** and **whole numbers** (for example the number of paths), a point followed by exactly three digits is ambiguous (‘12.500’): the field reports “Ambiguous number” and keeps the previous value; write 12500 or 12'500. Percentage and year fields accept ‘1.25’ as before. Notations such as ‘1e5’ or ‘0x10’ are not allowed anywhere.

## @s-milestones Milestones
Keywords: pension fund payout, retirement, start, end, AHV pension start, AHV start, date, age, point in time, limits

A milestone is a named point in time to which all other inputs can refer (‘from pension fund payout’). If you move the pension fund payout, everything that depends on it moves along. This is the most important lever for playing through scenarios of ‘retiring earlier or later’.

**Simulation start** and **simulation end** are predefined. In addition, **every person** has two fixed milestones that always exist:

| Milestone | Meaning | Limits | Default |
|---|---|---|---|
| **Pension fund payout** (formerly ‘Retirement’) | payout of pension and capital from the pension fund, start of the withdrawal strategy | 58 to 65 years | 65 |
| **AHV pension start** | start of the AHV pension; end of the contributions and of indirect amortization into [pillar 3a](#s-pillar3a), payout of the rest | 63 to 70 years | 65 |

They cannot be deleted, but their point in time can be chosen freely (age or date) as long as it is within the limits; the limits themselves are allowed. The limits can be changed in the [settings](#f-settings). With two persons, they are called “Pension fund payout person 2” and “AHV pension start person 2”. The names of the fixed milestones appear in the chosen language and are not stored in the plan file. The **end of earned income** is not a milestone of its own: with the switch “Earned income ends with the pension fund payout” in the [income](#s-flows), it ends with the person’s pension fund payout and moves with it. If the switch is off and earned income is still greater than 0 after the pension fund payout, the validation shows a warning with the amount.

You add your own milestones with “+ Milestone”, for example ‘Verkauf Haus’ or ‘Kinder ausgezogen’.

Every milestone is defined as:

- **Date:** a fixed month (05.2027)
- **Age of:** the age of a person, for example ‘Person 1 at 64.5 years’. Decimals are allowed: 64.5 years are 64 years and 6 months.

The column “Results in” shows the month calculated from it. **When you switch** between “Date” and “Age of”, the point in time stays the same: the age or the date from the column “Results in” is entered into the new field. The milestone can be deleted with ‘✕’. **If it is still in use**, a dialog asks first: it names the number of inputs and the sections, and on deletion changes them to the **fixed date** (month/year) of the milestone, for “Age of” to the calculated date. If this creates a duplicate point in time in a time series, the later row applies and a message points this out. With “Undo” (⌘Z), everything can be taken back. **Simulation start, simulation end, pension fund payout and AHV pension start** cannot be deleted; their button is grayed out.

**Older plans:** If one of the fixed milestones is missing, it is added at 65 years when the plan is opened. If an age is outside the limits, it is set to the nearest limit. ‘Retirement’ is now called “Pension fund payout”. A dialog names every change.

> Set the **end of the simulation** deliberately late, for example at age 95 or 100. An end that is too early makes the result better than it is: the money only has to last until then.

## @s-pension Pensions
Keywords: pension fund, AHV, pillar 2, pillar 1, pillar 3a, pension

Pension fund (BVG, pillar 2) and old-age pension (AHV, pillar 1); with two persons, each person has separate cards. You find the numbers on the **pension statement** of your pension fund and in the **AHV account** (account statement or pension forecast of the compensation office).

- Pension fund: [retirement savings, payout, lump sum, conversion rate](#f-pk)
- AHV: [pension and start of payout](#f-ahv)
- Care home: [entry, care home costs, sale of the owned home](#f-care-home)

A line below the cards shows what becomes of the balance at the pension fund payout (capital and pension).

**Pillar 3a** has a section of its own, see [Pillar 3a](#s-pillar3a).

## @s-pillar3a Pillar 3a
Keywords: 3a, tied pension provision, contribution, payout, payout plan, age, date, balance, maximum amount, indirect amortization

Pillar 3a is a separate account **per person**, separate from the pension fund and the mortgage.

- **Balance at simulation start** and **interest / return p.a.**
- **Contribution per year:** 0 up to the maximum amount (7 258 francs; with “Without pension fund (large pillar 3a)” for the person 36 288 francs, at most 20 % of the earned income; changeable in the [settings](#f-settings)). The app pays in one twelfth every month, until the month before the **AHV pension start**. The contribution **counts as spending**: it appears in “Total spending” of the yearly values, increases the withdrawal and is part of the net need of the 3-bucket strategy.
- **Payout plan:** rows of point in time and amount, for example 2041 50 000 and 2042 50 000. As with a milestone, you set the **point in time** as a **date** (MM.YYYY) or as the **age** of the person who owns the account (for example 61.5 years). Next to it are the resulting month and the age. If you switch between “Date” and “Age”, the point in time stays the same. Without an amount, the whole balance is paid out. A payout is possible **at the earliest 5 years before the AHV pension start**, also by age; an earlier entry is an error at the field. If an amount is larger than the balance, only the balance is paid out (notice in the events).
- **Balance from the preview:** Every row shows the **balance before the payout** and the **rest after it**; below the plan is the balance paid out at the AHV pension start. The values come from the preview with fixed returns; if the plan has an error, there is no preview, and ‘–’ is shown.
- **+ Payout** creates a row one year after the last one (without a row: in the earliest allowed month, 5 years before the AHV pension start, not before the simulation start), at the latest at the AHV pension start. The amount is the balance at that point in time after the earlier payouts, rounded down to 1 000. You can change it freely afterwards.
- **Rest at the AHV pension start:** Whatever is still in the account is paid out **at the latest in the month of the AHV pension start**.
- **Use:** into liquid wealth or, if a tranche with indirect amortization into this account exists, “Repay mortgage” (first the tranches with indirect amortization).

Every payout is a **lump-sum withdrawal** with lump-sum tax. Withdrawals in the same calendar year (pension fund and pillar 3a) are added together for the rate; for unmarried couples, per person.

The **indirect amortization** of a mortgage flows into this account until before the AHV pension start (see [Mortgages](#f-re-mortgage)). Contribution and indirect amortization together should not exceed the maximum amount per year; otherwise the validation shows a warning. Without a pension fund, the validation also warns as soon as the sum exceeds 20 % of the earned income per year (with the first month in which the income is too low); only this share is then deducted.

> Example: born 06.1980, balance 120 000 without interest, payout at 61.5 years (results in 12.2041) of 50 000: balance before the payout 120 000, rest after it 70 000. “+ Payout” suggests a row at 62.5 years (12.2042) of 70 000.

> Example: balance 120 000, payouts in 2041 and 2042 of 50 000 each, AHV pension start 2045: the rest of 20 000 (plus interest) is paid out in the month of the AHV pension start. If the AHV pension start is moved to 2046, the plan stays valid, and the rest comes in 2046.

## @f-settings Settings
Keywords: settings, limits, AHV age, pension fund payout, maximum amount, 3a, default values, language, appearance, dark, light, affordability, loan-to-value, inflation, first year

On the Mac, menu “VivaPlan → Settings…” (⌘,); on Windows and Linux, menu “File → Settings…” (Ctrl+,); or the button “Settings” at the top. The settings apply to **all plans on this device** and are stored locally, **not in the plan file**.

| Setting | Default |
|---|---|
| Language | as system |
| Appearance | System |
| AHV pension start: lower and upper limit | 63 and 70 years |
| Pension fund payout: lower and upper limit | 58 and 65 years |
| Pillar 3a: maximum contribution per year | 7 258 |
| Large pillar 3a (without pension fund): maximum per year | 36 288 |
| Inflation in the parametric model from year | 1950 |
| Affordability: max. share of income | one third (33 %) |
| Affordability: imputed interest rate | 5 % |
| Affordability: fixed costs p.a. (of the price) | 1 % |
| Affordability: maximum loan-to-value | 80 % |

**Language:** “as system” follows the language of the operating system (at the first start; without region, fr_FR counts as French); otherwise the chosen language applies. German, English, French and Italian are offered; if the system's language is missing, German applies. The help appears in the chosen language; a chapter that is not yet translated appears in German. The same choice is in the menu “View → Language”. A change takes effect immediately, also on the menus; the plan, unsaved changes and the view stay. Numbers, amounts and dates appear in the Swiss format in every language (1'234'567.50, 2.5 %, 03.2040), and the plan file is the same in every language. Names in the plan (persons, properties, milestones) are not translated; only the example plan gets its names in the chosen language when it is opened, with the same numbers.

**Appearance:** “System” follows the light or dark appearance of the operating system, also when it changes while you work; “Light” and “Dark” apply regardless of it. The theme applies to the whole app: input fields, selection menus, dialogs, help and charts. The choice takes effect as soon as you save.

**Affordability:** The four values of the [affordability](#f-re-afford) are rules of the bank, not details of your own plan; every value must be a number. A change takes effect immediately on the preview and the simulation. The section “Real estate” shows the values that apply in a note. If you open an older plan file that still contains its own values and they differ from the settings, the app takes these values over into the settings; the dialog “Notes on the plan” names the adopted and the previous values. Saved results and the comparison of the plans show the values that were used in the calculation. The command line uses the values of an older file, otherwise, like the app, the values of the settings (see [command line](#ref-cli)).

**Inflation in the parametric model from year:** “Monte Carlo · parametric” estimates mean, dispersion and correlations of the inflation only from the years of the series from this year on (default 1950). Otherwise the war and post-war years drive the dispersion up strongly: for “World (JST)” 5.0 % per year with all years from 1900, 2.3 % from 1950, with almost the same mean (2.3 % and 2.2 %). Stocks and bonds still use the whole series; the historical sources (fixed, historical, rolling, bootstrap) are not affected. A year before the start of the series uses the whole series. If fewer than 10 years remain from the first year, the simulation reports an error and does not compute. See [Monte Carlo · parametric](#f-ret-parametric).

The lower limit must not be above the upper one, ages are whole years, and the maximum amount is 0 or more. “Default values” resets everything after a confirmation. If a plan is outside the new limits after a change, the validation reports this; the same plan can therefore give a message on another device with other settings. The command line uses the same limits as the app (see [command line](#ref-cli)).

## @f-pk Pension fund
Keywords: BVG, retirement savings, contributions, interest, pension statement, without pension fund, large pillar 3a, self-employed

- **Without pension fund (large pillar 3a):** Switch at the top of the card, per person, for gainfully employed persons without a pension fund, especially the self-employed. For this person, contribution and deduction of [pillar 3a](#s-pillar3a) go up to **20 % of the earned income**, at most the maximum of the large pillar 3a (36 288 francs, changeable in the [settings](#f-settings)); for everyone else the small pillar 3a applies (7 258 francs). When the switch is on, all pension fund fields of this person are grey and read-only, and the calculation runs **without a pension fund** for them: no savings, no contributions, no pension and no lump sum (and so no lump-sum tax). The entered values stay saved in the plan file and apply again as soon as you switch it off. The AHV stays unchanged.
- **Retirement savings at simulation start (MM.YYYY):** the balance according to the pension statement (retirement savings including the extra-mandatory part). Do not enter the projections in the statement (projection with interest) here; the app calculates them itself.
- **Payout from:** always the milestone **pension fund payout** of the person (58 to 65 years). You change the point in time under [Milestones](#s-milestones).
- **Contributions per month:** sum of employee and employer contributions (and buy-ins) paid in until the payout. They end with the payout.
- **Interest per year:** interest rate on the retirement savings. The interest is credited in December. 1 % to 2 % is realistic.
- [Lump-sum share](#f-pk-lumpsum) and [conversion rate](#f-pk-conversion) determine how much flows as capital and how much as a pension.
- **Enter buy-in as a deduction:** A buy-in into the pension fund is deductible from the taxable income in the year of the buy-in. The link adds a [one-off deduction](#f-tax-deductions) “PK buy-in” in the section Taxes and goes there; enter the tax year and the amount there. The buy-in itself (payment, higher capital) is entered as before through contributions or capital.

Contributions and interest can be entered [time-dependent](#f-timed), for example higher contributions from 55.

## @f-pk-conversion Conversion rate
Keywords: pension, pension fund, percent, conversion

The conversion rate determines how much **annual pension** you receive per franc of retirement savings that you do not withdraw as capital.

> Annual pension = pension capital × conversion rate. Example: 500 000 francs × 5.0 % = 25 000 francs per year (about 2 083 per month).

- The **legal minimum** in the mandatory part is 6.8 %, but many pension funds calculate with **lower** rates (often 4.5 % to 5.5 %) because they convert the extra-mandatory part differently. Use the value from your pension statement, not the legal minimum.
- The rate depends on the age at payout. Use the rate for your retirement age.
- In the app: pension per month = pension capital × conversion rate / 12, **fixed** (no inflation adjustment) for life.

With the scenarios, you can vary the lump sum (see [Scenarios](#f-sweep)).

## @f-pk-lumpsum Lump-sum share
Keywords: lump sum, pension, lump-sum tax, one-off payout

The share of the retirement savings paid out **as capital** at retirement. 0 % means ‘entirely as a pension’, 100 % ‘entirely as capital’.

- The capital is added net to your liquid wealth. The **lump-sum tax** is deducted from it (see [Tax tables](#f-tax-tables)). Withdrawals in the same calendar year (pension fund, pillar 3a) are added together for the rate; for unmarried couples, per person.
- The rest becomes a pension with the [conversion rate](#f-pk-conversion).
- **Pension:** predictable, for life, but not indexed and not inheritable. **Capital:** flexible and inheritable, but carries the investment risk and the responsibility not to use it up too early.

The best share depends on the tax, the conversion rate and your willingness to take risks. Run it through a [variation](#f-sweep) and see how the success rate changes.

## @f-ahv AHV
Keywords: pension, payout, inflation, index, pillar 1, contribution years, 13th AHV pension, December, early payout, deferral, reduction, supplement

- **Pension per month at the start of payout:** your expected monthly pension when drawn at 65 (reference age) according to the pension forecast of the compensation office, as a nominal amount (until entitlement the app computes without inflation), **without the 13th pension** (the app adds it itself, see below).
- **Payout from:** always the milestone **AHV pension start** of the person (63 to 70 years). If it lies before or after 65, the app reduces or increases the pension itself (see below); still enter the pension at 65.

Below the field you always see the pension the app computes with at entitlement, nominal before the first inflation adjustment: without an early payout or deferral the same as the input, otherwise with the rate, e.g. *−13.6 % → effective 2'177.00 CHF per month*. For married couples it also shows from when both are entitled and which pension remains after the upper limit and the reduction or supplement, e.g. *from 03.2032 capped 1'547.00 CHF*.

**Early payout and deferral:** Drawing early reduces the pension by 6.8 % per year, month by month according to the AHV table (e.g. 15 months 8.5 %), at most 13.6 % when drawn at 63. A deferral of at least one year increases it in quarter-year steps: 1 year 5.2 %, 2 years 10.8 %, 3 years 17.1 %, 4 years 24.0 %, 5 years 31.5 %; a deferral of less than one year brings no supplement. Reduction and supplement are fixed amounts in whole francs; they apply to the whole payout period, also after 65, and per person by their own age. Example: pension 2 520, drawn at 63 → 2 520 − 343 = 2 177; drawn at 68 → 2 520 + 431 = 2 951, no AHV between 65 and 68. For married couples the upper limit applies, as with the AHV, to the **unreduced** pensions as soon as both are entitled (payout, also when drawing early, or age 65 reached, also during a deferral); only then does the app subtract the reduction or add the supplement. The reduction is the rate times the average pension during the early payout, the supplement the rate times the average deferred pension, each already limited where applicable. Example with two pensions of 2 520 and an upper limit of 3 780: if the first person draws at 63 and the second at 65 much later, there are first 2 177, then 1 890 − 343 = 1 547 and 1 890. The 13th pension rests on the pensions paid. A payout less than one year after 65 is no deferral (no supplement); a payout before 63 or after 70 is not possible. In both cases the check warns. Source: leaflet 3.04 *Flexible retirement* (as of 1.1.2026). Not included are partial early payout and partial deferral and the special rates for women born 1961 to 1969. Plans saved before this change held the already adjusted pension for a payout before or after 65; when opened, the app converts it once back to the pension at 65 and reports it. The result stays the same except for the rounding to francs; for married couples it may change because of the upper limit.
- **Adjustment for inflation:** Until entitlement (payout, or 65 when deferred) the entered pension stays unchanged; the first adjustment comes in the following January. The AHV pension is increased in January by the inflation accumulated since the last January. It never falls in nominal terms: with negative inflation it stays the same, and afterwards it rises again only once prices exceed their highest level so far, and only by the part above it. Example: pension 30 000 per year, inflation −8.4 % → 30 000 in the following year; then +15.7 % → 30 000 × 0.916 × 1.157 ≈ 31 794 (+6.0 %), not 34 710. The reduction, the supplement and the cap for married couples follow the same rule. A pension without adjustment can be calculated as an exception under “Simulation” › “Expert”, see [What grows with inflation](#inflation).

With two married persons, there is an upper limit for both pensions together, see [Second person and rules for couples](#f-couple).

**13th AHV pension:** From December 2026 the AHV pays a 13th pension every year in December; the app always includes it, there is no switch for it. It amounts to one twelfth of the AHV pensions paid from January to December (each month rounded to the centime, the sum to whole francs). If the pension starts during the year, it is paid pro rata: a pension of 2 520 from July gives 6 × 210 = 1 260. For married couples the already capped pension counts; the 13th pension itself is not capped again. It counts as AHV income in December: in the [yearly values](#r-table) under receipts, in the chart [Income and spending per year](#r-flows) under AHV. It is taxable like the AHV pension. If the simulation ends before December, there is no 13th pension in that year. When you open a plan saved before this change, a note appears once that the result may therefore differ.

## @f-couple Second person and rules for couples
Keywords: couple, married, cap, 150 percent, partner, double income

With “+ Second person”, you enter a household with two persons.

- Every person has **their own income, their own pension fund and their own AHV**.
- Taxes, wealth, spending and real estate apply to the **household**.
- **The household counts as retired** as soon as all persons with a payout date are retired. Only then do the withdrawal strategies take effect.

**Married** (rules for couples): If both are entitled to an AHV pension, their pensions at 65 are together capped at **3 780 francs per month**; the reduction for drawing early and the supplement for a deferral come after that (see [AHV](#f-ahv)) (150 % of the maximum pension 2025). The value can be changed if the maximum pension rises. The cap rises once every January with inflation as soon as at least one pension with inflation adjustment is running. Under [Taxes](#s-taxes), separate tables can be stored for married couples. Lump-sum withdrawals in the same year are added together for married couples, and per person for unmarried couples.

**Married with one person:** The “Married” box is also available without a second person, for example if the spouse has neither income nor a pension. The tax tariffs and deductions for married couples then apply (in the “Place of residence” mode canton, municipality and federal tax, with your own tables the tables for married couples); the tax is the same as for a married couple with a second person without income. The cap on the AHV pensions only applies with two persons. Older plans with one person had “Married” ticked without effect; they are taken over as unmarried when opened and calculate as before, a note says so. New plans start unmarried.

**Not calculated:** death of a person, survivors' pensions, widow's pension. The plan calculates as if both lived until the end.

## @f-care-home Care home
Keywords: care home, nursing home, care home costs, care, age, entry, sell the owned home, costs of illness, deduction

One card of its own per person in the tab “Pensions”. With **“Plan a care home”** you enter:

- **Entry:** age of the person (e.g. 85), a milestone or a fixed date.
- **Care home costs per month (today’s prices):** board, care and support as far as you pay them yourself. The costs rise with inflation. From the month of the entry they count as spending; the [yearly values](#r-table) show them in the column “Care home”, the chart [Income and spending per year](#r-flows) as a bar of their own, and the [events](#r-events) show the entry.
- **Sell the owned home at the entry:** the self-occupied properties are sold in the month of the entry, like a planned sale with property gains tax; mortgage, interest and fixed costs end.

**Housing costs:** When a single person lives in the care home, rent and other housing costs end from the month of the entry. Rents “after sale” do not start, not even after a sale or an [emergency sale](#f-re-emergency); nor does a purchase “after sale” take place. Without “sell” the property stays with its costs and the care home costs come on top; an emergency sale with scarce wealth still applies.

**Couples:** The household’s housing costs go on as long as one person lives at home; the care home costs of the person in the care home come on top. Only when **both** live in a care home do rent and housing costs end, and the property is sold if “sell” is chosen for one of the two persons. The plan does not model a death.

**[Never sell properties](#f-re-never-sell)** has priority: no sale at the entry, the costs of the property go on; the validation points this out.

**Taxes (simplified):** The care home costs count as costs of illness and disability. They are deductible for the federal and the cantonal tax as far as they exceed 5 % of the net income; the net income is the income after the other [deductions](#f-tax-deductions). Example: net income 60 000, care home costs 108 000 a year → deduction 108 000 − 3 000 = 105 000; the taxable income never drops below 0. The app applies this rule for every canton; some cantons calculate differently, for example only with the share for care. The app does not calculate supplementary benefits.

**Validation:** A missing entry, an age outside 0 to 150 years or negative costs are errors. After the sale of the last property (planned or because of the care home) the validation points out when neither a rent nor a care home follows, because then the plan would have no housing costs.

## @s-flows Income & spending
Keywords: income, living expenses, spending, one-off amounts, inheritance, inflation

Monthly amounts, earned income per year. **All amounts are entered in today's prices** and carried forward by the app with inflation, including earned income and one-off amounts (see [What grows with inflation](#inflation)). So you enter your current costs, also for years in the future.

- **Net earned income per year:** as in the tax return, the net salary on the salary statement (item 11), including the 13th monthly salary and bonuses, i.e. what remains after the deductions for AHV and pension fund (taxes are calculated separately); for the self-employed the income from self-employment. Next to the field is the amount per month, for example 120 000 per year ‘= 10 000 / month’; the app calculates and saves per month (one twelfth), also in every row of a time-dependent income and for both persons. Existing plans show the previous monthly amount times 12. The parameters of a run and the warning below also give the amount per year. When it ends is set by the switch below; without the switch it ends when you set it to 0 with a [time-dependent value](#f-timed).
- **Inflation of earned income** (per person): the amounts, including every row of a time-dependent income, are in **today's prices** (prices at the simulation start) and grow with the inflation of the respective path, just like living expenses. Example: 8 000 per month at 2 % inflation are 8 160 after one year and about 8 830 after five years. The exception “nominal, without inflation” is under “Simulation” › “Expert” ([What grows with inflation](#inflation)); older plans keep their previous choice there. Taxes, yearly values and affordability use the adjusted amount; the warning after the pension fund payout then names the amount ‘in prices of the simulation start’.
- **Earned income ends with the pension fund payout** (per person): with a tick, earned income is 0 from the month of this person’s milestone [Pension fund payout](#s-milestones), even if a time-dependent income still has an amount afterwards. If you move the milestone, the end moves with it. Example: pension fund payout in July 2030, income 120 000 per year; 60 000 count in 2030, nothing from 2031. Everything that depends on earned income follows it: the large pillar 3a (20 % of earned income), the professional expenses and the deductions for working persons in the proposal of the canton, and the second earner deduction. Without a tick, the income applies as entered, for example for partial retirement or work after retirement; if it is then still greater than 0 after the pension fund payout, the validation shows a warning with the amount. New plans and a newly added second person have the tick set; older plans without the setting calculate without it as before.
- **Living expenses per month:** the **input** covers everything you spend, **including rent or housing costs** (mortgage interest, fixed costs), as shown on your bank statement, but **without amortization**: it builds wealth (debt reduction or contribution to pillar 3a) and is added as an expense in the simulation; its amount does not change the living expenses without housing. The app deducts the **housing share**, which is the housing costs in the start month from the section [Real estate](#s-realestate) as a monthly average (interest is paid quarterly), carried forward with inflation, and equally from every row. The **calculation** uses living expenses without the housing share plus the **simulated** housing costs: if the interest changes at a renewal or you move into a rental after a sale, spending changes accordingly. Example: 8 000 including 2 000 housing costs; after an interest increase of 300, spending is 8 300, with a rent of 2 500 it is 8 500. If a row is smaller than the housing share, the validation reports an error. The contribution to [pillar 3a](#s-pillar3a) is not included. The app converts older plans (living expenses without housing costs) when they are opened, and a notice names the amount added. For plans whose housing share still contained the amortization, the app lowers the living expenses by the amortization of the start month when they are opened, so that the result stays the same; a notice names the amount. Taxes are calculated separately. A realistic value is the most important single factor for the result. Look at two to three years of bank statements for this.
- **One-off amounts:** date, amount and description. Positive is an inflow (inheritance, gift), negative an expense (car, travel, renovation).

Spending often decreases with age, for example less travel from 80, while health costs rise. Use [time-dependent values](#f-timed) for this.

## @f-timed Time-dependent values
Keywords: constant, time-dependent, steps, line, table, factor, support point, row, from, course, mini chart, glide path

Many amounts and rates are time-dependent. At the top right of the field you choose the form of the course:

- **Constant:** one value for the whole simulation.
- **Steps:** a sequence of rows ‘from *point in time* applies *value*’. The point in time is a milestone (‘from pension fund payout’) or a fixed date. Every row applies until the next one begins. Example living expenses: ‘from start 9 500’, ‘from pension fund payout 8 500’, ‘from 80 7 000’.
- **Line:** a straight line ‘from’ a value at one point in time evenly, month by month, ‘to’ a second value at a later point in time. Before it the first value applies, after it the second. Example stock allocation: 90 % at 55, 50 % at 65 gives 70 % at 60.
- **Table:** support points ‘at *point in time* applies *value*’. “+ Point” adds another one, ✕ deletes it. “In between” chooses whether the value between two support points runs **gradual** like a line or as **steps**. Before the first support point its value applies, after the last one the last value.
- **Factor:** a **base value** times a course of factors in percent. You enter the factor itself again as a constant, steps, line or table. Example living expenses: base value 6 000, factor as a table with steps: at simulation start 100 %, at 80 80 %.

For line and table the point in time of a support point is an **age** of a person (with decimals, 64.5 = 64 years and 6 months), a **milestone** or a **fixed date**. If you change the kind, the point in time stays the same; a milestone becomes, for example, the age at which it lies.

If you change the form, the app takes over what can be taken over: steps, line and table keep their points in time and values, “Constant” takes the value at simulation start, “Factor” takes it as the base value and the previous course as factors.

The small chart next to the field shows the course (for a factor the result base value times factor). With the mouse wheel you zoom in; the buttons “All”, ‘10 yrs’, ‘5 yrs’ and ‘2 yrs’ choose the range.

Rows and support points that are further down in the list but begin before another one are sorted by point in time by the app. So feel free to move milestones; the order of the values stays consistent. All forms are also written like this in the [plan file](#ref-file).

## @s-wealth Wealth & investment
Keywords: stocks, bonds, cash, liquidity, stock allocation, reserve, portfolio, investment strategy, index

The **balances at simulation start**, the **split** of the portfolio and the **investment strategy**.

- **Stocks, bonds, cash:** your balances at simulation start. Real estate and pension assets are separate; only liquid wealth is entered here. The app takes over the balances unchanged at the start. Next to stocks and bonds is their share of the portfolio (without cash, whole percent); for example, 80 000 stocks and 20 000 bonds give ‘80 % of the portfolio’ and ‘20 % of the portfolio’.
- **Cash does not count toward the portfolio.** The [stock allocation](#f-stockshare) only refers to stocks and bonds: 80 000 stocks, 20 000 bonds and 50 000 cash give a stock allocation of 80 %.
- **Until retirement** (pension fund payout of the first person), no withdrawal strategy applies. The app holds as much cash as is in the field **Cash**, carried forward with **inflation**: a surplus goes into the portfolio according to the stock allocation, and a shortfall is refilled from the portfolio.
- **From retirement**, the [withdrawal strategy](#s-withdrawal) determines the cash: “Standard” with the [cash reserve](#f-reserve) from its card, all others with 3 months, or the 3-bucket strategy with its buckets.

- **Investment strategy:** one index each for the stocks (World, “World, custom”, Switzerland, S&P 500) and for the bonds (Swiss Confederation 10 years or short-term, USA). With “World, custom”, you set the weights of Switzerland, USA, EU and Asia-Pacific; together they must add up to 100 %. Below the selection are the common period and the mean returns; if an index is cut to the common period, a notice appears. Details: [Investment strategy](#f-ret-series).

In the simulation, wealth is carried forward every month with the return of stocks and bonds: from the indices of the investment strategy, using the method under [Simulation](#s-returns) (fixed returns, historical course or Monte Carlo).

## @f-stockshare Stock allocation
Keywords: risk, stocks, rebalancing, portfolio

The share of **stocks in the portfolio** (stocks + bonds, without cash). The rest is bonds.

- The app restores this share every month (**rebalancing**), unless the strategy provides otherwise.
- A high stock allocation brings more return on average, but also deeper lows. The risk is high especially at the beginning of retirement (**sequence risk**): a stock market crash shortly after retirement does much more damage than the same crash 20 years later.
- The value can be [time-dependent](#f-timed), for example 70 % until retirement and 50 % afterwards, or a [characteristic curve](#f-curve) over the state of the simulation, for example over the withdrawal rate.

How strongly the stock allocation changes the result is shown by a [variation](#f-sweep) of the stock allocation, or by the [optimization](#s-optimize), which searches for the allocation at the same time.

## @f-curve Characteristic curve
Stichworte: characteristic curve, curve, state, state variable, withdrawal rate, funded ratio, drawdown, peak, dynamic stock allocation

Instead of a value or a course over time, the [stock allocation](#f-stockshare) can be a **characteristic curve**: it then depends on the **state** of the simulation, for example on the withdrawal rate. The curve consists of support points, each a *value of the state variable* with the *stock allocation* that applies there; in between it runs straight, below the first support point its value applies, above the last one the last value. The stock allocations lie between 0 and 1.

**State variables:**

- **Age** (`age`): age of the first person in years.
- **Liquid wealth (real)** (`liquidReal`): cash, bonds and stocks in prices of the simulation start (divided by the inflation since the start).
- **Withdrawal rate** (`withdrawalRate`): withdrawal of the last 12 months divided by the liquid wealth. The withdrawal of a month is the expenses (living expenses, housing costs, taxes) minus the income (earned income, pensions, savings contributions); one-off amounts, the purchase and sale of properties and pillar 3a do not count. In the first year the months so far count, extrapolated to 12; at the simulation start the uncovered expenses of the start month times 12.
- **Funded ratio** (`fundedRatio`): liquid wealth divided by the remaining gap until the end of the simulation, both in prices of the simulation start. The gap is the living expenses minus earned income and pensions (AHV and pension fund, as far as known in the month of the evaluation), without discounting, without housing costs and taxes. From 1 the wealth covers the gap without returns. If income and pensions cover everything, the last value of the curve applies.
- **Stock return 12 months** (`stockReturn12m`): return of the stock index over the last 12 months, 0.1 = 10 %.
- **Distance from the peak** (`drawdown`): how far the stock index lies below its highest level so far (including the lead-in of the return path), 0 at the peak, 0.3 = 30 % below it.

**Evaluation:** when **rebalancing**. With `"eval": "yearly"` (default) at the first rebalancing of every calendar year; the value then applies into the next year. With `"eval": "monthly"` every month. Like a course, the stock allocation applies with the [3-bucket strategy](#f-wd-three_buckets) only until retirement.

**Example:** support points 2 % → 80 % and 6 % → 40 % over the withdrawal rate. At a withdrawal rate of 4 % a stock allocation of 60 % applies, at 1 % 80 %, at 8 % 40 %.

**Entering:** In the editor the curve is shown only (state variable, evaluation, support points). Enter and change it in the [project file](#ref-file) or through the AI interface, for example `"stockShare": {"mode": "curve", "input": "withdrawalRate", "points": [[0.02, 0.8], [0.06, 0.4]], "eval": "yearly"}`. Another form at the top of the field replaces the curve with the value of its first support point.

**Limit of the parametric model:** The [parametric model](#f-ret-parametric) draws the return of every year independently of the previous years; a crash is not more likely to be followed by a recovery there. Rules that rely on a recovery after crashes (for example more stocks at a large distance from the peak or after bad 12 months) cannot be checked meaningfully with it: they cannot show an advantage there. Check such rules with historical paths ([historical](#f-ret-historical), [all starting points](#f-ret-rolling) or [bootstrap](#f-ret-bootstrap) with blocks), and keep in mind the risk of fitting the rule to the past.

## @f-reserve Cash reserve
Keywords: cash, reserve, months, liquidity reserve

How many **months of uncovered spending** (spending minus income and pensions) are held as cash **from retirement** before the rest is invested according to the [stock allocation](#f-stockshare). Before retirement, the field “Cash” under [Wealth & investment](#s-wealth) applies.

- A high reserve protects against having to sell stocks at low prices, but costs return.
- Adjustable only with the strategy “Standard” (card “Standard” under “Withdrawal strategy”) and only effective from retirement. **All other strategies** (guardrails, fixed rate, VPW, endowment) calculate with a **fixed reserve of 3 months** from retirement; with the [3-bucket strategy](#f-wd-three_buckets), the buckets replace the reserve.
- An older plan with another strategy and a reserve other than 3 months therefore calculates with 3 months and may give slightly different results. The app points this out once, the first time such a plan is opened; after that, the notice no longer appears. The stored value stays in the file and applies again if you switch to “Standard”.
- Useful as a time-dependent value: for example 6 months until retirement and 12 months afterwards.

## @s-taxes Taxes
Keywords: income tax, wealth tax, lump-sum tax, tax multiplier, table

New plans calculate with the **place of residence**: income, wealth and lump-sum tax from canton, municipality and church tax according to the tax laws of the Confederation and the cantons, the property gains tax by the canton of the property ([Tax by place of residence](#f-tax-location)). Only the municipality and the denomination are needed; everything else applies with defaults that are visible as **assumptions**. **Own tables** (rate tables and tax multiplier, as described below) are only left for existing plans that already use them: there they are under **“Expert”**, and the card offers the button **“Switch to place of residence”**. Existing plans open unchanged and calculate the same.

Simple **rate tables** that you can derive from the tax calculator of your canton. Between the brackets, the app **interpolates linearly**, and the rate is multiplied by the [tax multiplier](#f-tax-multiplier).

- [Tax by place of residence](#f-tax-location)
- [Deductions from income](#f-tax-deductions)
- [Deductions from the tax bill](#f-tax-bill)
- [Taxable yield of the wealth](#f-tax-yield)
- [Tax multiplier and tables](#f-tax-tables)
- [Tariffs with inflation](#f-tax-indexed)
- [How accurate is the tax?](#tax-accuracy)
- **Income tax** is assessed in December on the annual income (including pensions and the [taxable yield of the wealth](#f-tax-yield)), **wealth tax** on the portfolio plus real estate net (with the [tax value](#f-re-tax-value) where entered, otherwise with the default or, in older plans, with the market value). Both are paid in May of the following year.
- A tax still open at the end of the plan is deducted in the last month.
- The [property gains tax](#f-tax-gain) is due when a property is sold, see also [Real estate](#s-realestate).

The example values are only placeholders. For your result, replace them with values of your place of residence.

## @f-tax-location Tax by place of residence
Keywords: place of residence, canton, municipality, church tax, denomination, FTA, tax data, standard deductions, professional expenses, federal tax, taxable income, lump sum, lump-sum tax, property gains tax, canton of the property, move, change of residence, reference date

In the mode **Place of residence** you only enter the **municipality** and the **church tax** (none, Reformed, Roman Catholic, Christ Catholic). The search covers all municipalities of the tax year: type the name and choose from the list (name with canton); the **canton** follows from the municipality and is shown next to it. Tariffs and deductions come from the bundled tax data file according to the tax laws of the Confederation and the cantons, the tax multipliers from the base data of the FTA tax calculator; the card names the tax year, the source and the tax multipliers of the municipality. There is no network access.

**Estimated tax:** At the top of the card there are two numbers in today’s prices, rounded to 100 francs: the income and wealth tax in the **first full calendar year** and in the **first full calendar year after retirement** (after the household’s last pension fund payout), without lump sums and property gains. Compare them with your last tax bill. If retirement is before the start, there is only one number. The numbers come from the preview; as long as details are missing (for example the municipality), a note stands instead of the numbers. The deductions become more accurate with **“Take over from the tax bill”** below the numbers, see [Deductions from the tax bill](#f-tax-bill).

**Assumptions:** Below the place of residence there is a line with the defaults that apply, in words: standard deductions of the canton until and from retirement ([Deductions from income](#f-tax-deductions)), [tariffs with or without inflation](#f-tax-indexed), pillar 3a deduction (yes or no), [tax value](#f-re-tax-value) of the properties (default 50 % of the market value) and [taxable yield](#f-tax-yield) of the wealth. A click on the line opens the settings; the line shows changes at once. New plans have these defaults set; the measurement with several made-up households shows that without the deductions the tax would be clearly too high in working years.

**Expert:** Below them is the closed area **“Expert”** (deductions in detail, deductions from wealth, tariffs) with the rows of the [Deductions from income](#f-tax-deductions) (period, proposal or own value, additional deduction), the [Deductions from wealth](#f-tax-wealth-deductions), the note on the [tariffs with inflation](#f-tax-indexed) and, in existing plans, the own tables. Most plans do not need it. If a plan has own values there (a row with an own value or an additional deduction, values taken over from the tax bill, deductions from wealth or own tables), the area opens by itself. The check box for pillar 3a is among the assumptions, the [one-off deductions](#f-tax-deductions) are in a card of their own directly in the section. Opening and closing changes neither the plan nor the calculation.

**Taxable income** = income of the year (earned income net, pension fund pension, AHV) minus the **standard deductions** (proposal of the canton), never negative. Instead of the proposal an own value can apply per period, plus pillar 3a, see [Deductions from income](#f-tax-deductions). Canton and federal tax each have their own deductions:

- **Social deductions** for single persons or married couples, where the canton has them.
- **Insurance premiums and savings interest:** in years with earned income the amount “with contributions to the 2nd and 3rd pillar”, otherwise the amount “without”.
- **Flat deduction for professional expenses** on the earned income of each working person: rate, minimum and maximum of the canton, at most the earned income, no deduction without earned income. Example Zurich: 3 %, at least 2 000, at most 4 000 francs, so 3 000 with 100 000 earned income. In Basel-Landschaft, Basel-Stadt and Ticino a fixed flat amount according to the law is recorded (500, 4 200 and 3 500 francs); the actual costs can be higher (⚠).
- **Second earner deduction** for married couples when both have earned income: according to the rule of canton and federal tax, based on the lower earned income after the flat deduction for professional expenses and at most this amount; Bern takes 2 % of the total earned income. Example federal tax: 50 %, at least 8 600, at most 14 100 francs; Zurich a fixed 6 200 francs. Thurgau has no second earner deduction.

**Tax:**

- Income tax = tariff of the canton on the taxable income × (tax multiplier canton + church) + tariff of the municipality × tax multiplier municipality + direct federal tax on the federal taxable income. The tariff of the municipality is that of the canton, except in Schwyz and Valais. In Valais the tariff of the canton and of the municipalities is indexed: it applies to an income reduced by the index (canton 173 %, municipality 110 to 176 % depending on the municipality). In Valais, married couples receive a reduction of 35 % on the tax of the canton and of the municipality (after the tax multiplier, without church tax), at least 680 and at most 4 900 francs each (2026, indexed; LF VS Art. 32 and 178). The reduction for single parents is not included because the plan has no children.
- Wealth tax = tariff of the canton (social deductions on the wealth included) × (tax multiplier canton + municipality + church), without federal tax.
- **Lump-sum tax** (pension fund, pillar 3a) = capital rule of the canton × (tax multiplier canton + church) + rule of the municipality × tax multiplier municipality + federal tax (one fifth of the tariff of the direct federal tax, without deductions). The tax multipliers of the income tax apply; the rule applies to the whole lump sum. Lump sums in the same calendar year are added together: for married couples for the household with the tariff for married couples, for unmarried couples per person with the tariff for single persons. Below the municipality the rule of the canton is shown, for example Zurich “rate of the income tariff for one twentieth of the lump sum, at least 2 %”. The rules come from the cantonal tax laws; in Valais they are only approximated (⚠).
- **Married couples** (“Married” ticked, also for a single person): tariffs and deductions for married couples. Unmarried couples are calculated with those for single persons on the joint income.
- With [tariffs with inflation](#f-tax-indexed) the amounts of the tariffs and deductions grow with the simulated inflation.
- **Property gains tax:** for each property the rule set of the canton in which it lies applies; you choose the canton with the property under [Real estate](#s-realestate), without an entry the canton of residence applies. Details under [Property gains tax](#f-tax-gain). With the check box **“Own rules instead of the rules of the canton”** the own rules with the [tax multiplier](#f-tax-multiplier) apply to all properties instead.

**Change of residence:** With “Add place of residence” you add a further place of residence, from a year or from an event (for example retirement), each with canton, municipality and church tax; the first row applies from the simulation start. The rules of the Tax Harmonisation Act apply (StHG Art. 4b): the **income and wealth tax** of a year is computed with the place of residence on **31 December**, for the whole year – if you move in June, you pay the whole year at the new place. The **lump-sum tax** is computed with the place of residence in the **month of the payout**; moving to a cheaper canton therefore only helps if the move is before the payout. The **property gains tax** of a property without its own canton is computed with the canton of residence in the month of the sale. The [proposal of the canton](#f-tax-deductions) follows the place of residence of the tax year, an own value stays. Not computed are moving abroad, moving costs and tax liability for only part of the year. The parameters of a run name each place of residence with its start, the yearly values show the place of residence of each year with the tax columns.

**Hidden:** the own tables for income, wealth and lump sums (also those for married couples), the own rules of the property gains tax and the tax multiplier (without the check box) stay in the file. **“Switch to place of residence”** in a plan with own tables changes to the mode “Place of residence” (deductions without an own value become the proposal of the canton); after choosing the municipality a note names the estimated tax of the first year before (own tables) and after (place of residence). It can be undone like every change.

**Validation:** If the municipality of a row is missing, the municipality is not in the canton or a row starts before the previous one, this is an error in the section Taxes, and “Start simulation” is blocked. The overview of the assumptions and the tab “Parameters” name the place of residence, for example “Zürich (ZH), church tax none”.

## @f-tax-deductions Deductions from income
Keywords: deductions, proposal of the canton, own value, tax assessment, taxable income, pillar 3a, imputed rental value, mortgage interest, debt interest, professional expenses, second earner deduction, details

The card **Deductions from income** in the section Taxes (under “Expert”) sets what is deducted from the income of the year (earned income net, pension fund pension, AHV). The **taxable income** = income − deductions, never negative.

**Rows:** The deductions are time-dependent like the earned income; a row applies from its point in time until the next one. New plans have two rows with the proposal of the canton, “from the simulation start” (until retirement) and “from the pension fund payout” (from retirement); more rows are possible, for example “from 2029” because the imputed rental value is abolished. If the row changes during a year, each row counts for its months.

Each row is either (the explanations are also in the ⓘ next to the title of the card):

- **Proposal of the canton** (only in the mode [place of residence](#f-tax-location), the default there): the standard deductions according to the tax laws, for canton and federal tax – social deductions, insurance premiums and savings interest, professional expenses, for married couples the second earner deduction. The head line of the row shows the sum for canton and federal tax in the first year of the row; **Details** opens a table with the parts (rows) for canton and federal tax (columns) and the total, parts with 0 are left out. Approximated professional expenses (Basel-Landschaft, Basel-Stadt, Ticino) are marked with ⚠. The proposal is computed anew if you change the canton, the municipality or the marital status. **Additional deduction:** “+ Additional deduction” shows the field for it (with a stored amount it is always shown). An own amount in prices of the simulation start (grows with “Tariffs with inflation”) is added to the proposal, for canton and federal tax. Example: proposal 20 000, additionally 3 000, deduction 23 000.
- **own value:** an amount per year in prices of the simulation start (grows with [tariffs with inflation](#f-tax-indexed)), optionally plus a percentage of the income. It applies to canton and federal tax and stays when you change the canton, the municipality or the marital status. In the mode “Own tables” there is only the own value; without a row 0 applies. Example: own value 40 000, income 120 000, tax as on 80 000.

**Determining the own value:** The input help computes it from the **income** and the **taxable income according to the assessment**, for example 120 000 and 80 000 give 40 000; “Use as own value” puts it into the row. Note:

- Income is **net**, after AHV and PK contributions, like the net salary on the salary statement, including pensions.
- **Without pillar 3a:** the contributions come from the plan (see below). The deduction according to the assessment already includes them: if “Deduct contributions to pillar 3a” is switched on, enter pillar 3a according to the assessment under **“of which pillar 3a”**; the input help takes it off, otherwise it counts twice. Example: 120 000, taxable 80 000, of which pillar 3a 7 000 gives 33 000 (without the check box 40 000). If the field is empty, the input help points this out.
- **Home ownership** (imputed rental value, mortgage interest, maintenance) belongs in the own value. From 2029 there is no imputed rental value and no interest deduction for owner-occupied property; add a row “from 2029” for it. **Mortgage interest is not deducted automatically.**

**One-off deductions:** They are in a card of their own, **“One-off deductions”**, directly in the section Taxes, not under “Expert”, so that they are quickly at hand for decisions. “+ One-off deduction” adds a deduction for a single tax year, with tax year, description and amount (in today's prices, grows with inflation until the tax year), for example a pension fund buy-in, a renovation or costs of illness. The amount counts in this tax year in addition to the row that applies, with the proposal of the canton as with the own value and in both modes. Example: income 120 000, own value 30 000, one-off 50 000 in the year 2027 gives a taxable income of 40 000 in 2027, 90 000 in the other years. The estimate of the net need of the [3-bucket strategy](#f-wd-three_buckets) uses only the ongoing deductions. **Pension fund buy-in:** In the card of the pension fund (section Pension) the link **“Enter buy-in as a deduction”** adds a one-off deduction “PK buy-in” (with the name if there are two persons) and goes to this card. The tax year (default: the year after the start) is ready for input; enter the year and the amount of the buy-in.

**Pillar 3a:** With the check box **“Deduct contributions to pillar 3a”** the contributions of a year from the section [Pillar 3a](#s-pillar3a) (including the indirect amortization) are deducted from the taxable income, for canton and federal tax, at most the maximum per person from the settings (the maximum amounts are in the ⓘ next to the check box). Example: income 100 000, no other deductions, contribution 7 000, tax as on 93 000. For a person “Without pension fund (large pillar 3a)”, 20 % of their earned income in the year applies, at most the maximum of the large pillar 3a; like the other amounts it grows with “Tariffs with inflation”. Example: earned income 100 000, contribution 25 000, deduction 20 000; earned income 300 000, contribution 40 000, deduction 36 288. New plans have the check box set.

**Existing plans** compute as before: in the mode “place of residence” with the proposal of the canton throughout, with own tables without deductions, and without the pillar 3a deduction. Only “Adjust deductions” creates rows.

**Validation:** A negative amount (also “additionally” and one-off), a percentage below 0 % or above 100 % and a tax year outside the simulation are errors in the section Taxes.

**Results:** With “Show taxes” the yearly values show the income and the deductions (canton; the origin – proposal of the canton, additionally, own value, one-off, pillar 3a – in the tooltip of the cell); the taxable income is in the column **Taxable income** directly before the taxes (the federal one in the tooltip if it differs).

**Care home:** The app also deducts the care home costs from the card [Care home](#f-care-home) as costs of illness as far as they exceed 5 % of the net income (federal tax and canton each with their own net income). The yearly values name the deduction in the origin of the deductions.

## @f-tax-bill Deductions from the tax bill
Keywords: tax bill, assessment, deductions, taxable income, taxable wealth, personal rest, check

The most accurate way to the [deductions from income](#f-tax-deductions) are the numbers of your last **final tax bill**. The button **“Take over from the tax bill”** in the card [Place of residence](#f-tax-location) (only with the tax by place of residence) opens a dialog with fields from the first page of the tax bill: **tax year**, **total income**, **taxable income** of the canton, **taxable wealth** and optionally **total taxes** (canton, municipality, federal). The app recalculates with every input.

- **Until the retirement:** deduction = total income − taxable income, never negative. It becomes the first row (from the simulation start) as own value in prices of the start. The app deducts the contributions to pillar 3a of the plan separately; they therefore come off the own value, otherwise they would count twice.
- **From the retirement** professional expenses, the second earner deduction and pillar 3a fall away. The app therefore takes the proposal of the canton without earned income and adds the **personal rest** as additional deduction: rest = deduction without pillar 3a − proposal of the canton with earned income, never negative. The app takes the proposal with earned income from the first full year of the plan with earned income, for couples for the whole household. The row starts with the pension fund payout; for couples with that of the person who retires last. Example: income 120 000, taxable 95 000 → deduction 25 000; proposal with earned income 21 000 → rest 4 000; proposal without earned income 9 000 → from retirement 13 000.
- **Already retired:** if the retirement is before the start, the deduction applies from the start; there is no rest.
- **Taxable wealth:** the dialog shows the taxable wealth of the plan on 31 December of the first year after the social deductions of the canton (estimated from the wealth tariff) and the difference to the tax bill. If it differs, it proposes with properties a share of the [tax value](#f-re-tax-value) of the market value, without properties a [deduction from wealth](#f-tax-wealth-deductions) from the start. The proposal is only taken over with a tick. If the tax bill shows more wealth without properties, check the wealth in the plan.
- **Check:** with total taxes the dialog shows the estimate of the app for the same taxable values (the federal tax on the taxable income of the canton) and the deviation in percent. From 15 % a hint names possible causes: municipality, marital status, denomination or income. The app continues without a correction factor.

**“Take over”** replaces the rows of the deductions from income; one-off deductions stay. The rows taken over name their origin (from tax bill and the tax year) in the card Deductions from income, in the line Assumptions, in the tooltip of the deductions in the yearly values and in the tab “Parameters” of a run. If you change a row by hand, its origin is dropped.

**Check of the inputs:** a tax year after the start, a negative amount and an empty required field are errors at the field; “Take over” is then disabled.

## @f-tax-wealth-deductions Deductions from wealth
Keywords: deductions, taxable wealth, wealth tax, debts, social deductions, one-off

The card **Deductions from wealth** in the section Taxes (under “Expert”) sets what is deducted from the wealth on 31 December, for example debts other than the mortgage or cantonal deductions missing in the tariffs. The **taxable wealth** = wealth − deductions, never negative. Properties count at their [tax value](#f-re-tax-value) minus the mortgage.

**Estimated taxable wealth:** At the top, a note shows the taxable wealth on 31 December of the first tax year, rounded to 1 000 francs, with the share of the properties (tax value minus mortgage), for example 500 000 on 31.12.2026, of which properties 250 000. Compare it with your last tax return. If a property has no tax value, the note warns: the market value is then used, and the taxable wealth is often far too high. The link leads to the field “Tax value”. In the mode “Place of residence”, cantonal allowances contained in the tariffs are not deducted; the tax return may therefore be lower.

**Details:** The composition shows for a selectable tax year how the taxable wealth comes about, structured as in the tax return: wealth without properties (cash, bonds, stocks; without pension fund and pillar 3a), per property the tax value (without a tax value the market value, with ⚠) and the mortgage, the deductions from wealth and the taxable wealth. Below it, each property has the field [Tax value](#f-re-tax-value), the same as in the section Real estate.

**Rows:** “+ Row” adds a time-dependent row with the field “+ additionally” (per year); it applies from its point in time until the next one. For a tax year the row that applies on 31 December counts. The amount is in prices of the simulation start and grows with [tariffs with inflation](#f-tax-indexed).

**One-off deductions:** “+ One-off deduction” adds a deduction for a single tax year, with tax year, description and amount (in today's prices, grows with inflation until the tax year). It applies on the reference date 31 December of its tax year, in addition to the row that applies. Example: wealth 800 000, ongoing 50 000, one-off 100 000 in the year 2027 gives a taxable wealth of 650 000 in 2027, 750 000 in the other years.

**Mode “place of residence”:** The social deductions of the canton are already included in the wealth tariffs of the tax data. Your deductions are added; the deductions of the canton cannot be replaced. The card shows a hint about this.

**Own tables:** The amounts are the whole deduction from wealth.

**Existing plans** have no deductions from wealth and compute as before.

**Validation:** A negative amount and a tax year outside the simulation are errors in the section Taxes.

**Results:** With “Show taxes” the yearly values show the deductions from wealth (the origin – additionally, one-off – in the tooltip of the cell) and the taxable wealth (the wealth before deductions in the tooltip).

## @f-tax-yield Taxable yield of the wealth
Keywords: yield, dividends, interest, coupon, distribution, distribution yield, taxable income, stocks, bonds, capital gain, accumulating

Dividends and interest of the wealth are taxable income. The card **Taxable yield of the wealth** in the section Taxes assumes a rate per year for each asset class:

- **Stocks (distribution yield):** dividends as a percentage of the balance.
- **Bonds (coupon):** interest as a percentage of the balance.
- **Cash** and reserved sale proceeds have no taxable yield (the simulation pays no interest on them).

**Computation:** The taxable yield of a year is the average balance of each asset class in the year × rate. Example: stocks 500 000 on average at 2 %, bonds 300 000 at 1 %: taxable yield 13 000. In the first and last year only the months of the plan count. The yield counts to the income for the income tax, the same in both tax modes; in the mode [Place of residence](#f-tax-location) before the deductions of canton and federal tax. Which deductions apply (working or retired) still depends only on the earned income. The yield also belongs to the net income on which the self-share of the costs of a [care home](#f-care-home) is computed. The rate does not grow with [tariffs with inflation](#f-tax-indexed), because it is a share of the balance.

**Part of the return:** The yield is not an additional inflow. The return of the simulation stays the same (the [return series](#s-returns) already contain dividends and coupons); only the tax on it is new. **Capital gains** are not taxable. The rate covers distributing and accumulating funds, because the retained yield of an accumulating fund is taxable too.

**Guide values:** The dividend yield of a global stock index is about 2 %, the yield of the Swiss Bond Index about 1 %. New plans start with stocks 2 % and bonds 1 %. **Existing plans** open with 0 % and compute as before.

**3-bucket strategy:** The net need of the [3-bucket strategy](#f-wd-three_buckets) contains the income tax on the yield (current balances × rates). The yield itself does not count as a receipt there, because it is part of the return.

**Not included:** withholding tax and its refund, foreign withholding tax, deduction of the costs of asset management.

**Validation:** A rate below 0 % or above 100 % is an error in the section Taxes.

**Results:** With “Show taxes” the yearly values show the column **Taxable yield** before the income tax, if a rate is entered.

## @f-tax-multiplier Tax multiplier
Keywords: canton, municipality, multiplier

A factor on all rates of the tables, the tax multiplier of **canton and municipality** together (for example 2.0 for a double rate). This way, you do not have to rewrite the tables when you move; you only change the factor. Values from 0 to 10 are allowed. In the mode [Place of residence](#f-tax-location) it applies only to own rules of the property gains tax (check box “Own rules instead of the rules of the canton”) and is hidden otherwise.

## @f-tax-indexed Tariffs with inflation
Keywords: bracket creep, inflation, brackets, tariff, today's prices

The **amounts** of the tables for income, wealth and lump sums apply in **today's prices** (prices at the simulation start) and grow with the simulated inflation of the respective path; the **rates** stay the same. This way the tax burden does not rise merely because of inflation (compensation of bracket creep), as the Confederation and the cantons handle it.

- **Example:** table 0 → 0 %, 200 000 → 10 %, inflation 2 % per year. In year 10 the upper bracket is at 243 799. An income of 121 899 (100 000 with inflation) has the rate 5 % like 100 000 in the start year; with nominal tariffs it would be 6.09 %.
- The price index in the month of the assessment applies: December for income and wealth, the month of the payout for lump sums. The estimated income tax in the need of the [3-bucket strategy](#f-wd-three_buckets) is computed the same way.
- This also applies to the tables for married couples.
- The [property gains tax](#f-tax-gain) stays unchanged; its tariff refers to the single gain.
- The ongoing deductions from income and from wealth grow in the same way.
- The exception “nominal, without inflation” is under “Simulation” › “Expert” ([What grows with inflation](#inflation)): the amounts then apply nominally over the whole plan. Older plans without the setting and imported old configurations keep their previous calculation this way.

## @f-tax-gain Property gains tax
Keywords: property gain, sale, property, holding period, Zurich, Aargau, canton, exemption limit, tariff, tax multiplier, allowance, place of residence, approximation

The tax on the gain when a property is **sold**. The rules **differ from canton to canton**. In the mode [Place of residence](#f-tax-location) the [rules of the canton](#f-tax-gain-canton) of the property apply; in the mode “Own tables” (or with the check box “Own rules instead of the rules of the canton”) the own rules, which are fully editable. **Required** (only for own rules): if the plan contains a property with the status “Owned” or “Purchase planned”, the rules must be set (Zurich template, Aargau template or your own rules, also with a rate of 0 %). If they are missing, this is an error in the section Taxes, and “Start simulation” and “Start optimization” are blocked. Pure rental properties and plans without a property need no rules.

Two models are available: **Zurich** (bracket tariff with surcharges and reductions) and **Aargau** (direct tax rate by holding period). **Your own input** is based on the Aargau model.

**Calculation:** gain = sale value − purchase price paid. Value-adding investments, transfer and sales costs are not deducted. Then:

1. **Exemption limit:** If the gain is below it, it is tax-free. From the limit on, the **whole** gain is taxed.
2. **Zurich model, brackets:** The gain is taxed bracket by bracket. Every row applies to its part of the gain (“to” is the width of the bracket); above the last bracket, its rate applies.
3. **Zurich model, holding period:** The tax is changed by the surcharge or reduction of the matching row. The last row whose years are reached applies (holding period in full years, from the [purchase date](#f-re-bought)).
4. **Fixed reduction:** an additional reduction in percent, independent of the holding period. In the templates, it is set to 0.
5. **Tax multiplier:** With the option “Multiply by the tax multiplier”, the tax is additionally multiplied by the tax multiplier of canton and municipality (default: off).

**Template canton of Zurich** (tariff § 225 StG, as of 09/2025): brackets 10 % (first 4 000), 15 % (6 000), 20 % (8 000), 25 % (12 000), 30 % (20 000), 35 % (50 000), 40 % above that. Surcharge of 50 % for a holding period under 1 year, 25 % under 2 years. Reduction from 5 full years 5 %, then 3 percentage points more per year up to 50 % from 20 years. Gains below 5 000 are tax-free. The tax multiplier only applies if the option above is set.

**Template canton of Aargau** (§ 109 StG): In the **Aargau model**, instead of the brackets, a **direct tax rate on the whole gain** applies, depending on the holding period in full years: 40 % in the first year of ownership, then 2 percentage points less per year down to 20 % in the eleventh year of ownership, after that 1 percentage point less per year down to 6 % in the 25th year of ownership and 5 % from 25 full years. Without a purchase date, the highest rate applies, and the app warns.

For another canton, adapt the tables. You find the values in the tariff sheet of the cantonal tax administration. Check them before use; the template reflects a certain date and is not tax advice.

## @f-tax-gain-canton Property gains tax by canton
Keywords: property gains tax, canton, property, place of residence, rule set, tax guide, approximation, holding period, months, tax multiplier, median

In the mode [Place of residence](#f-tax-location), the **rule set of the canton in which the property lies** applies automatically to each property. All 26 cantons are covered; the source is the cantonal tax law (as of 2025/2026).

- **Canton of the property:** field with each property that is owned or planned for purchase under [Real estate](#s-realestate). Without an entry the canton of residence applies. Below it is the rule of the canton, for example Zurich “bracket tariff 10 % to 40 %, surcharge up to 50 % under 2 years, reduction up to 50 % from 20 years”.
- **Holding period in months:** the rules of the cantons count the holding period in months from the [purchase date](#f-re-bought) to the sale; several cantons calculate surcharges per month.
- **Tax multiplier:** Bern, Lucerne, Obwalden, Solothurn, Schaffhausen and Jura calculate a simple tax times the tax multiplier. If the property lies in the canton of residence, the tax multiplier of the municipality of residence (canton and municipality) applies, otherwise the median tax multiplier of the municipalities of the canton.
- **Order:** exemption limit, allowance, change of the gain by holding period, tax by the model of the canton (bracket tariff, rate by holding period, rate by gain or rate by annual return), surcharge above a threshold for a short holding period, surcharge or reduction by holding period, tax multiplier, reduction, maximum burden.
- **Without a purchase date** no surcharge or reduction by holding period applies, for cantons with a rate by holding period the highest rate; the validation warns.
- **Approximations (⚠):** for Basel-Stadt, Basel-Landschaft, St. Gallen, Vaud and Valais the rule is only approximated; for example, owner occupation is assumed in Basel-Stadt, St. Gallen and Vaud. The user interface shows the note with ⚠ at the property.
- **Not included:** owner occupation per property, the reduction for owner occupation in Basel-Landschaft, the reduction for value-enhancing expenses and the adjustment of the investment costs to inflation.
- **Validation:** own rules are not required. An unknown canton of a property is an error at the field.

With the check box **“Own rules instead of the rules of the canton”** in the section Taxes, the own rules and the tax multiplier apply to all properties, as in the mode “Own tables”.

## @f-tax-tables Tax tables
Keywords: rate, interpolation, income, wealth, lump sum, married

Every table has brackets of **amount** and **rate**. In between, the app interpolates linearly.

| Table | Amount | Rate |
|---|---|---|
| Income tax | taxable annual income | share of income due as tax |
| Wealth tax | taxable wealth | share of wealth per year |
| Lump-sum tax | lump-sum withdrawal in the year | share of the withdrawal |

- **Rates are decimal numbers:** 0.05 stands for 5 %, typed as a percentage in the user interface.
- The tax amount is rate × amount × tax multiplier, rounded to the centime.
- **Limits:** every rate is between 0 % and 100 %, every amount is 0 or greater. The validation reports values outside these limits at the field.
- **Tax must not decrease:** a higher amount never results in a lower tax. If the rate drops so sharply between two brackets that rate × amount becomes smaller, the validation reports a **warning** at the bracket concerned; the calculation still runs. Usually a marginal tax rate has then been entered instead of the average rate. With several lump-sum withdrawals in the same year, the tax on a further withdrawal is then never negative but 0.
- **Lump-sum withdrawals** in the same calendar year (pension fund and pillar 3a) are added together for the rate: for married couples for the household, for unmarried couples per person (every person pays the tax on their own withdrawals). If you spread withdrawals over different years, the rate drops.
- **Married:** With “Married” ticked (also for a single person), you can store separate tables for married couples. Without separate tables, the normal ones apply.
- Start from the **effective** tax burden, not the marginal tax rate. The tax calculator of your canton provides both.

## @tax-accuracy How accurate is the tax?
Keywords: estimate, estimated tax, accuracy, rounding, rounded, reference cases, approximation, deductions, tax advice, range

The tax in VivaPlan is an **estimate for planning**, not a tax assessment. The results therefore call it “estimated tax” and show tax amounts **rounded to 100 francs**. The calculation is exact, the rounding applies to the display only; the command line and its CSV export output the exact values.

**Reference cases:** The calculation rules are tested with cases recalculated by hand: rate from the table interpolated linearly times the tax multiplier, lump-sum withdrawals of a year combined for the rate, steps and holding period of the property gains tax. The Canton of Zurich template of the property gains tax matches the official table. So VivaPlan calculates correctly with your tables; how close this is to your actual tax depends on the tables.

**Mode “Place of residence”:** 25 made-up cases in Zurich, Bern, Lucerne, Aarau and St. Gallen (income 80 000 to 300 000, wealth 300 000 to 4 million, retired persons, without children and denomination) deviate less than 1 % from the FTA tax calculator 2026; at most ±5 % is required. For working persons (single, married with one or two earned incomes; gross salary 70 000 to 210 000, wealth 50 000 to 800 000), 25 made-up cases in the same municipalities are also less than 1 % off the FTA tax calculator. In addition, cases from the tax calculators of Zurich, Aargau, Graubünden and Valais (Sion and Agarn; married couples in Sion with the reduction) and from official tariff tables (Lucerne, Solothurn, Basel-Landschaft, Vaud, Jura, Geneva) match. Individual deductions, children, income-dependent deductions, travel and meal costs are not included.

**Lump sums in the mode “Place of residence”:** 48 made-up cases in Zurich, Bern, Lucerne, Aarau, St. Gallen, Geneva, Fribourg and Bellinzona (lump sums of 100 000, 500 000 and 1 million, single and married) deviate at most ±5 % from the FTA tax calculator 2026. In Valais the rule is approximated (annuity for a person aged 65); three lump sums in Sion deviate less than 1 % from the tax calculator of Valais. In Zug the rule matches the tax calculator of the canton to the centime.

**Property gains tax in the mode “Place of residence”:** per canton, a test checks invented cases (gain and holding period) against values computed by hand according to the law. In Basel-Stadt, Basel-Landschaft, St. Gallen, Vaud and Valais the rule is only approximated (⚠).

**Approximations:**

- Your tables give an average rate per amount; in between, VivaPlan interpolates linearly. The real tariff runs differently.
- A single tax multiplier for canton and municipality applies to all tables.
- With [tariffs with inflation](#f-tax-indexed), the steps grow with the simulated inflation; the cantons’ adjustment may differ.
- Property gain = sale value − purchase price paid.

**What is missing:**

- Individual deductions. They are only included as far as you build them into the rates.
- Imputed rental value, deductions for debt interest and maintenance.
- Church and poll tax and the direct federal tax, unless they are in the tables or the multiplier.
- Value-adding investments, transfer and selling costs for the property gains tax.
- Future changes in the law.

**Tax year and source:** In the mode “Place of residence” the card “Place of residence” in the section Taxes always names the tax year and the source of the tax data, below it the approximations of the canton with ⚠: professional expenses, lump-sum tax and property gains tax in the cantons of your properties. Below the yearly values with the tax breakdown and in the comparison (yearly values “Estimated taxes total”, chart of the cash flows with the taxes) a line names the tax year, the source, the origin of the deductions (standard deductions of the canton, own deductions or both) and “No tax advice”. If the current calendar year is more than one year later than the tax year of the data, the check shows a warning: tariffs, deductions and multipliers may have changed since.

VivaPlan does not show a **range** (± %): there is no basis for it, and another number would suggest a precision that does not exist.

**Lump-sum tax and property gains tax** often involve a lot of money: check with the canton’s tax calculator or a professional before deciding. VivaPlan is not tax advice.

## @s-realestate Real estate
Keywords: house, apartment, mortgage, rent, purchase, sale, affordability

Owned, planned and rented properties. You can enter as many as you like.

- **Owned (‘owned’):** The market value at simulation start is in the header next to the name. Below follow, in fixed places, [purchase date](#f-re-bought), purchase price (paid, for the property gains tax), “Sale from” (never, milestone or date) and fixed costs per month. In the tax mode [Place of residence](#f-tax-location) the **canton of the property** follows, with the rule of the [property gains tax](#f-tax-gain-canton) of this canton. With “Purchase planned”, the same fields are in the same places (“Purchase from” instead of the purchase date), only the market value is missing; when the status changes, the values entered are kept. The sale must be **after the purchase date**; otherwise it is an error at the field.
- **Inflation:** Value and fixed costs grow with inflation; for each property they can be calculated nominally under “Simulation” › “Expert” ([What grows with inflation](#inflation)).
- **Planned purchase (‘planned’):** purchase from milestone, purchase price, mortgage. In addition, there is **“Sale from”** (default ‘Never’, otherwise date or milestone); the simulation then sells the property at that point in time. The sale must be after “Purchase from”; otherwise it is an error at the field. For a purchase ‘after sale of …’, the point of purchase is not known when entering it, and this check is omitted.
- **Rent (‘rent’):** start and rent per month (today's value; grows with inflation until moving in, then stays nominal; a rent running at the start is nominal from the beginning; exceptions under “Simulation” › “Expert”).
- **Mortgage:** tranches with periods, see [Mortgages](#f-re-mortgage).
- **Tax value:** optional for each owned property or planned purchase, for the wealth tax, see [Tax value](#f-re-tax-value).
- [Emergency sale](#f-re-emergency) for purchases “after sale” and a note with the values of the [affordability](#f-re-afford); they are in the [settings](#f-settings).
- The switch [Never sell properties](#f-re-never-sell) at the top of the tab rules out any sale.

You enter [living expenses](#s-flows) **including** housing costs. The calculation uses living expenses **without the housing share**; rent, mortgage interest, amortization and fixed costs are added as simulated **housing costs**. After the sale of the last property, the next one with a purchase “after sale” is bought in the following month, provided it is affordable and the equity is sufficient. Otherwise the rents ‘from after sale’ begin. A purchase “after sale” is attempted **once** in the following month. If it cannot be financed, it is **dropped** with a message in the events, and the follow-up rules apply in the next month: rents ‘from after sale’ of this property begin, and further purchases “after sale” of it are attempted. A **purchase or sale date before the simulation start** is never reached; the validation reports it as a warning.

## @f-re-bought Purchase date
Keywords: holding period, property gains tax, purchase

The month in which you bought the property. It determines the **holding period** and thus the surcharge or reduction of the [property gains tax](#f-tax-gain): in many cantons, the longer you own the property, the less tax is due.

- The field only exists for owned properties. For a **planned** property, the month of purchase in the simulation counts.
- Without a purchase date, the app calculates **without** surcharge or reduction (the tax according to the basic tariff). If holding period rules are entered, the validation points this out.
- What counts is the holding period until the month of sale: with own rules in **full years**, with the [rules of the canton](#f-tax-gain-canton) (mode Place of residence) in **months**.

## @f-re-tax-value Tax value
Stichworte: tax value, wealth tax, official value, valuation

The **tax value** is the official value at which the property appears in the tax return. It is often **60–80 % of the market value**, with older valuations also considerably lower. Take the value from your **tax return** or the last assessment.

- The field exists for owned properties and planned purchases, not for rentals. You can choose the **default** (no entry; in older plans «as market value»), an **amount** or **% of market value**.
- Only the **wealth tax** uses it: for each property the tax value minus the mortgage is taxable instead of the market value minus the mortgage. Example: market value 1'000'000, tax value 650'000, mortgage 400'000 gives 250'000 taxable wealth instead of 600'000.
- Wealth, results, property gains tax, affordability and emergency sale keep using the **market value**.
- An amount keeps its **ratio to the market value** (for a planned purchase to the purchase price): if the market value rises with inflation, the tax value rises in the same ratio. In reality the tax value only changes with a revaluation; over the years this is an approximation.
- **Default:** New plans use **50 % of the market value** for properties without an own tax value (field: «default (50 % of market value)»). The default is a uniform assumption of VivaPlan for all cantons, not an official figure: depending on the canton and the age of the valuation, tax values are mostly well below the market value. An own amount or share replaces it. The plan only stores that the default applies; the program determines the share. Example: market value 1'000'000, mortgage 400'000 gives 100'000 taxable wealth with the default.
- **Older plans** keep using the market value without a tax value. A note in the card of the property offers the default; the link takes it over for all properties without an own tax value. The [composition of the taxable wealth](#f-tax-wealth-deductions) names the origin «default».
- A negative amount or a percentage outside 0–100 % is an error at the field.
- In the [annual values](#r-table) the tax columns include the column **Tax value properties** (sum on 31 December, properties without a tax value at the default or the market value).
- Not calculated: imputed rental value, the tax allocation for a property in another canton, and deductions for underuse.

## @f-re-mortgage Mortgages
Keywords: tranche, SARON, fixed mortgage, amortization, margin, interest

A mortgage consists of **tranches**. Every tranche has a sequence of **periods**:

- **Fixed:** fixed interest rate for the term of the period
- **SARON:** SARON plus margin (the SARON has a floor of 0). The course of the SARON comes from the time series or the fixed value, see [SARON](#ret-saron).

The first period starts on the start date of the tranche (at the purchase), every further one at the end of the previous one. After that, the tranche is extended on the same terms or repaid.

- **Interest** = debt × rate / 12 per month. It accrues and is paid together with the amortization at the **end of the quarter** (March, June, September, December).
- **Amortization:** fixed amount per quarter, direct (reduces the debt) or **indirect** (contribution to the [pillar 3a](#s-pillar3a) of the person; with two persons, you choose the account at the tranche).
- **Indirect amortization only until the AHV pension start:** From the month of the person's AHV pension start, the same amount is amortized **directly** and reduces the mortgage. A period that only begins from the AHV pension start (also a renewal) always amortizes directly; the selection then shows “direct (from AHV)”. The preview of the period notes ‘+ 3a until AHV’ if a period runs beyond the AHV pension start. Since every person has an AHV pension start, the rule always applies. The switch appears in the [events](#r-events) (‘Amortization of mortgage … from the start of the AHV pension directly instead of via pillar 3a’) if the tranche previously amortized indirectly and continues to amortize.
- If pillar 3a pays out with “Repay mortgage”, the amounts go first to tranches with indirect amortization into this account.
- At a sale, the accrued interest is paid immediately.

## @f-re-emergency Emergency sale
Keywords: sale, liquidity, threshold, emergency sale, forced sale

If liquid wealth runs low, the app sells the property in the following month. The **threshold** is months of uncovered spending: if liquid wealth covers fewer months (spending minus income), an emergency sale is necessary.

An emergency sale shows that the plan does not work out without selling the property. If [Never sell properties](#f-re-never-sell) is switched on, there is no emergency sale. It appears in the [events](#r-events) of the results. The higher the threshold, the earlier the sale.

## @f-re-never-sell Never sell properties
Keywords: switch, sale, emergency sale, affordability, without sale, keep

With this switch at the top of the “Real estate” tab you check whether the plan also works out if you never sell a property. It applies to **all** properties of the plan and is saved in the file (default: off).

- **No sale:** no planned sale (“Sale from”), no [emergency sale](#f-re-emergency) and no sale because a mortgage is not [affordable](#f-re-afford) at renewal. The mortgage is then **renewed in full**; an event reports “not affordable”. With several runs, the key figure **“Unaffordable renewals”** in the [overview](#r-property) shows in how many runs this happened (number and share); a paid one-off payment does not count.
- **Costs continue:** interest, amortization and fixed costs until the end of the plan.
- **Purchases:** a planned purchase with a date or milestone takes place. Purchases and rents “after sale” do **not** take place; the validation points this out, as well as planned sales.
- **Wealth depleted:** if liquid wealth is no longer sufficient, the run counts as depleted, as without a property. The success rate is therefore usually lower than with a sale.
- **Read-only:** while the switch is on, all other fields of the tab are grey and read-only. The values are kept and apply again unchanged when it is switched off.
- **Care home:** Even with “Sell the owned home at the entry” nothing is sold at the entry into the [care home](#f-care-home); the costs of the property go on.

## @f-re-afford Affordability
Keywords: bank, affordability, loan-to-value, imputed interest rate, equity, mortgage, wealth drawdown, retirement

The banks' affordability calculation, applied to purchases “after sale” and at every renewal of a mortgage period. A mortgage is affordable if the **imputed costs** do not exceed a share of the income. The values are in the [settings](#f-settings) and apply to all plans on this device.

- **Max. share of income:** usually one third (33 %).
- **Imputed interest rate:** usually 5 %, regardless of the current interest rate.
- **Fixed costs:** usually 1 % of the price per year.
- **Income:** Earned income and pensions are counted. If the income is not sufficient, the mortgage is reduced; the purchase then shows the **one-off repayment** from wealth as an event.
- **Wealth drawdown in the affordability** (switch, default off) with a **rate per year** (default 3 %): When switched on, this share of the **free wealth** (cash, stocks, bonds in the month of the check) also counts as income; at a purchase only the wealth after deducting the equity for this property. Pillar 3a and vested benefits do not count. Example: pension 60'000, free wealth 2'000'000, rate 3 % → counted income 120'000.
- **Maximum loan-to-value:** usually 80 % of the price. At least 20 % is equity.
- **Maximum loan-to-value from retirement** (default 65 %): Once all persons of the household are retired (pension fund payout), the mortgage at a purchase and at every renewal is at most this share of the market value in the month of the check. Anything above is treated like insufficient affordability.

Banks' practice varies: some count 3 to 5 % of the free wealth, others no fixed rate; from retirement they usually lend 60 to 67 %. Adjust the values to your bank.

If the purchase is not affordable, the mortgage is reduced. If the equity is still not sufficient, there is no purchase and the rents begin.

**At every renewal** of a mortgage period (also for owned properties), the app checks the affordability of the whole mortgage again, with the income of the month of renewal (earned income and pensions, with wealth drawdown if switched on) and, once everyone is retired, with the loan-to-value from retirement. If the mortgage is no longer affordable, the simulation calculates the **minimum one-off payment** needed to make it affordable again and pays it from wealth (proportionally across all tranches). This only applies if wealth is **not negative** afterwards **and** the [emergency sale criterion](#f-re-emergency) is **not** triggered after the payment (estimated with the planned spending and the housing costs after the payment). Then the mortgage is renewed with the reduced amount. Otherwise there is **no payment**, the mortgage is **not renewed**, and an **emergency sale** of the property is initiated (carried out in the following month). The result view logs every renewal (date and age, affordability, one-off payment, new mortgage, emergency sale with the reason “Wealth < 0” or “Emergency sale criterion”) and, with several runs, counts in how many runs one-off payments and emergency sales occur, also per percentile. Typical case: income drops at retirement.

## @s-scenarios Scenarios & simulation
Keywords: variation, sweep, parameter, runs, size

Here you set which parameters are **varied systematically**. “Start simulation” calculates the plan as entered and, in addition, every combination of the variations with all return paths. The result is a table and a heatmap in the [scenario results](#r-scenarios).

The duration of the simulation is also shown to the left of the button “Start simulation”. During the calculation, you can stop it with **Cancel**.

The number of combinations is **not limited**; only beyond **1'000'000** combinations does the engine refuse (memory protection). If the estimated calculation time is over **one minute**, the app asks before the start (duration and number of runs, “Start anyway” or “Cancel”). The display “Size” shows how many runs result in total (variations × paths) and a **rough estimate of the duration**; a **yellow warning symbol (⚠)** appears next to it as soon as the estimated calculation time exceeds **90 seconds**. It is based on a short measurement of your plan on this computer. The range (‘mostly … to …’) takes into account that runs take different amounts of time depending on the plan. Every additional variation and every additional path lengthens the calculation time accordingly: 35 combinations with 50 000 paths take about 10 seconds with bootstrap, and about one minute with the parametric model.

The variations themselves: [Variations](#f-sweep).

## @f-sweep Variations
Keywords: sweep, from, to, step, parameter, heatmap

A variation has a **parameter**, a range **From … To** and a **step**. The app calculates every value and shows the effect on the success rate and final wealth.

| Parameter | Meaning |
|---|---|
| Stock allocation | stocks in the portfolio of stocks and bonds (without cash) |
| Pension fund lump sum | lump-sum share |
| Historical start month | only with the source “Historical from start date” |
| Date of pension fund payout | month of the milestone “Pension fund payout” |
| Spending factor | factor on the living expenses without the housing share (0.9 = 10 % less); housing costs stay |
| Withdrawal rate (fixed rate / guardrails initial rate) | rate of the fixed withdrawal or initial rate of guardrails |

With two variations, a **heatmap** over both parameters results. All combinations use **the same return paths**. Differences therefore come only from the parameters, not from chance.
