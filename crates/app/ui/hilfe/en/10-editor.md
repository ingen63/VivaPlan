# Editor: sections and fields

## @s-general Household & project
Keywords: name, currency, persons, birth, second person, double income, simulation period

Here you set the basic data.

- **Project name and currency:** All amounts are entered in the project currency (CHF, EUR or USD). The indices of the [investment strategy](#f-ret-series) are in CHF; with a plan in EUR or USD, a notice appears.
- **Person 1:** Name and month of birth (format MM.YYYY). From the month of birth, the app calculates the age for milestones (‘at 65’) and for the labels of the charts.
- **+ Second person:** Adds a second person with their own income, their own pension fund and their own AHV (double income). Taxes, wealth and spending apply to the household. See [Second person and rules for couples](#f-couple).
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

They cannot be deleted, but their point in time can be chosen freely (age or date) as long as it is within the limits; the limits themselves are allowed. The limits can be changed in the [settings](#f-settings). With two persons, they are called “Pension fund payout person 2” and “AHV pension start person 2”. The names of the fixed milestones appear in the chosen language and are not stored in the plan file. The **end of earned income** is not a milestone: you set it in the [income](#s-flows). If earned income is still greater than 0 after the pension fund payout, the validation shows a warning with the amount.

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

A line below the cards shows what becomes of the balance at the pension fund payout (capital and pension).

**Pillar 3a** has a section of its own, see [Pillar 3a](#s-pillar3a).

## @s-pillar3a Pillar 3a
Keywords: 3a, tied pension provision, contribution, payout, payout plan, age, date, balance, maximum amount, indirect amortization

Pillar 3a is a separate account **per person**, separate from the pension fund and the mortgage.

- **Balance at simulation start** and **interest / return p.a.**
- **Contribution per year:** 0 up to the maximum amount (7 258 francs, changeable in the [settings](#f-settings)). The app pays in one twelfth every month, until the month before the **AHV pension start**. The contribution **counts as spending**: it appears in “Total spending” of the yearly values, increases the withdrawal and is part of the net need of the 3-bucket strategy.
- **Payout plan:** rows of point in time and amount, for example 2041 50 000 and 2042 50 000. As with a milestone, you set the **point in time** as a **date** (MM.YYYY) or as the **age** of the person who owns the account (for example 61.5 years). Next to it are the resulting month and the age. If you switch between “Date” and “Age”, the point in time stays the same. Without an amount, the whole balance is paid out. A payout is possible **at the earliest 5 years before the AHV pension start**, also by age; an earlier entry is an error at the field. If an amount is larger than the balance, only the balance is paid out (notice in the events).
- **Balance from the preview:** Every row shows the **balance before the payout** and the **rest after it**; below the plan is the balance paid out at the AHV pension start. The values come from the preview with fixed returns; if the plan has an error, there is no preview, and ‘–’ is shown.
- **+ Payout** creates a row one year after the last one (without a row: in the earliest allowed month, 5 years before the AHV pension start, not before the simulation start), at the latest at the AHV pension start. The amount is the balance at that point in time after the earlier payouts, rounded down to 1 000. You can change it freely afterwards.
- **Rest at the AHV pension start:** Whatever is still in the account is paid out **at the latest in the month of the AHV pension start**.
- **Use:** into liquid wealth or, if a tranche with indirect amortization into this account exists, “Repay mortgage” (first the tranches with indirect amortization).

Every payout is a **lump-sum withdrawal** with lump-sum tax. Withdrawals in the same calendar year (pension fund and pillar 3a) are added together for the rate; for unmarried couples, per person.

The **indirect amortization** of a mortgage flows into this account until before the AHV pension start (see [Mortgages](#f-re-mortgage)). Contribution and indirect amortization together should not exceed the maximum amount per year; otherwise the validation shows a warning.

> Example: born 06.1980, balance 120 000 without interest, payout at 61.5 years (results in 12.2041) of 50 000: balance before the payout 120 000, rest after it 70 000. “+ Payout” suggests a row at 62.5 years (12.2042) of 70 000.

> Example: balance 120 000, payouts in 2041 and 2042 of 50 000 each, AHV pension start 2045: the rest of 20 000 (plus interest) is paid out in the month of the AHV pension start. If the AHV pension start is moved to 2046, the plan stays valid, and the rest comes in 2046.

## @f-settings Settings
Keywords: settings, limits, AHV age, pension fund payout, maximum amount, 3a, default values, language

On the Mac, menu “VivaPlan → Settings…” (⌘,); on Windows and Linux, menu “File → Settings…” (Ctrl+,); or the button “Settings” at the top. The settings apply to **all plans on this device** and are stored locally, **not in the plan file**.

| Setting | Default |
|---|---|
| Language | as system |
| AHV pension start: lower and upper limit | 63 and 70 years |
| Pension fund payout: lower and upper limit | 58 and 65 years |
| Pillar 3a: maximum contribution per year | 7 258 |

**Language:** “as system” follows the language of the operating system (at the first start; without region, fr_FR counts as French); otherwise the chosen language applies. German, English, French and Italian are offered; if the system's language is missing, German applies. The help appears in the chosen language; a chapter that is not yet translated appears in German. The same choice is in the menu “View → Language”. A change takes effect immediately, also on the menus; the plan, unsaved changes and the view stay. Numbers, amounts and dates appear in the Swiss format in every language (1'234'567.50, 2.5 %, 03.2040), and the plan file is the same in every language. Names in the plan (persons, properties, milestones) are not translated; only the example plan gets its names in the chosen language when it is opened, with the same numbers.

The lower limit must not be above the upper one, ages are whole years, and the maximum amount is 0 or more. “Default values” resets everything after a confirmation. If a plan is outside the new limits after a change, the validation reports this; the same plan can therefore give a message on another device with other settings. The command line always uses the default values.

## @f-pk Pension fund
Keywords: BVG, retirement savings, contributions, interest, pension statement

- **Retirement savings at simulation start (MM.YYYY):** the balance according to the pension statement (retirement savings including the extra-mandatory part). Do not enter the projections in the statement (projection with interest) here; the app calculates them itself.
- **Payout from:** always the milestone **pension fund payout** of the person (58 to 65 years). You change the point in time under [Milestones](#s-milestones).
- **Contributions per month:** sum of employee and employer contributions (and buy-ins) paid in until the payout. They end with the payout.
- **Interest per year:** interest rate on the retirement savings. The interest is credited in December. 1 % to 2 % is realistic.
- [Lump-sum share](#f-pk-lumpsum) and [conversion rate](#f-pk-conversion) determine how much flows as capital and how much as a pension.

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

- **Pension per month at the start of payout:** your expected monthly pension when drawn at 65 (reference age) according to the pension forecast of the compensation office, in today's prices, **without the 13th pension** (the app adds it itself, see below).
- **Payout from:** always the milestone **AHV pension start** of the person (63 to 70 years). If it lies before or after 65, the app reduces or increases the pension itself (see below); still enter the pension at 65.

Below the field you always see the pension the app computes with, in today's prices: without an early payout or deferral the same as the input, otherwise with the rate, e.g. *−13.6 % → effective 2'177.00 CHF per month*. For married couples it also shows from when both are entitled and which pension remains after the upper limit and the reduction or supplement, e.g. *from 03.2032 capped 1'547.00 CHF*.

**Early payout and deferral:** Drawing early reduces the pension by 6.8 % per year, month by month according to the AHV table (e.g. 15 months 8.5 %), at most 13.6 % when drawn at 63. A deferral of at least one year increases it in quarter-year steps: 1 year 5.2 %, 2 years 10.8 %, 3 years 17.1 %, 4 years 24.0 %, 5 years 31.5 %; a deferral of less than one year brings no supplement. Reduction and supplement are fixed amounts in whole francs; they apply to the whole payout period, also after 65, and per person by their own age. Example: pension 2 520, drawn at 63 → 2 520 − 343 = 2 177; drawn at 68 → 2 520 + 431 = 2 951, no AHV between 65 and 68. For married couples the upper limit applies, as with the AHV, to the **unreduced** pensions as soon as both are entitled (payout, also when drawing early, or age 65 reached, also during a deferral); only then does the app subtract the reduction or add the supplement. The reduction is the rate times the average pension during the early payout, the supplement the rate times the average deferred pension, each already limited where applicable. Example with two pensions of 2 520 and an upper limit of 3 780: if the first person draws at 63 and the second at 65 much later, there are first 2 177, then 1 890 − 343 = 1 547 and 1 890. The 13th pension rests on the pensions paid. A payout less than one year after 65 is no deferral (no supplement); a payout before 63 or after 70 is not possible. In both cases the check warns. Source: leaflet 3.04 *Flexible retirement* (as of 1.1.2026). Not included are partial early payout and partial deferral and the special rates for women born 1961 to 1969. Plans saved before this change held the already adjusted pension for a payout before or after 65; when opened, the app converts it once back to the pension at 65 and reports it. The result stays the same except for the rounding to francs; for married couples it may change because of the upper limit.
- **Adjust the pension annually for inflation:** The AHV pension is increased in January by the inflation accumulated since the last January. Leave the box ticked if the pension should grow with it.

With two married persons, there is an upper limit for both pensions together, see [Second person and rules for couples](#f-couple).

**13th AHV pension:** From December 2026 the AHV pays a 13th pension every year in December; the app always includes it, there is no switch for it. It amounts to one twelfth of the AHV pensions paid from January to December (each month rounded to the centime, the sum to whole francs). If the pension starts during the year, it is paid pro rata: a pension of 2 520 from July gives 6 × 210 = 1 260. For married couples the already capped pension counts; the 13th pension itself is not capped again. It counts as AHV income in December: in the [yearly values](#r-table) under receipts, in the chart [Income and spending per year](#r-flows) under AHV. It is taxable like the AHV pension. If the simulation ends before December, there is no 13th pension in that year. When you open a plan saved before this change, a note appears once that the result may therefore differ.

## @f-couple Second person and rules for couples
Keywords: couple, married, cap, 150 percent, partner, double income

With “+ Second person”, you enter a household with two persons.

- Every person has **their own income, their own pension fund and their own AHV**.
- Taxes, wealth, spending and real estate apply to the **household**.
- **The household counts as retired** as soon as all persons with a payout date are retired. Only then do the withdrawal strategies take effect.

**Married** (rules for couples): If both are entitled to an AHV pension, their pensions at 65 are together capped at **3 780 francs per month**; the reduction for drawing early and the supplement for a deferral come after that (see [AHV](#f-ahv)) (150 % of the maximum pension 2025). The value can be changed if the maximum pension rises. The cap rises once every January with inflation as soon as at least one pension with inflation adjustment is running. Under [Taxes](#s-taxes), separate tables can be stored for married couples. Lump-sum withdrawals in the same year are added together for married couples, and per person for unmarried couples.

**Not calculated:** death of a person, survivors' pensions, widow's pension. The plan calculates as if both lived until the end.

## @s-flows Income & spending
Keywords: income, living expenses, spending, one-off amounts, inheritance, inflation

Monthly amounts. **Spending is entered in today's prices** and carried forward by the app with inflation. So you enter your current costs, also for years in the future.

- **Net earned income per month:** what arrives in your account after deductions (AHV, pension fund; taxes are calculated separately). It ends when you set it to 0 with a [time-dependent value](#f-timed), for example ‘from pension fund payout 0’. The pension fund payout does not end it on its own; if it is still greater than 0 afterwards, the validation shows a warning with the amount.
- **Rises with inflation** (per person): with a tick, the amounts, including every row of a time-dependent income, are in **today's prices** (prices at the simulation start) and grow with the inflation of the respective path, just like living expenses. Example: 8 000 per month at 2 % inflation are 8 160 after one year and about 8 830 after five years. Without a tick, the amount stays the same nominally. New plans and a newly added second person have the tick set; older plans without the setting calculate without it as before. Taxes, yearly values and affordability use the adjusted amount; the warning after the pension fund payout then names the amount ‘in prices of the simulation start’.
- **Living expenses per month:** the **input** covers everything you spend, **including rent or housing costs** (mortgage interest, amortization, fixed costs), as shown on your bank statement. The app deducts the **housing share**, which is the housing costs in the start month from the section [Real estate](#s-realestate) as a monthly average (interest and amortization are paid quarterly), carried forward with inflation, and equally from every row. The **calculation** uses living expenses without the housing share plus the **simulated** housing costs: if the interest changes at a renewal or you move into a rental after a sale, spending changes accordingly. Example: 8 000 including 2 000 housing costs; after an interest increase of 300, spending is 8 300, with a rent of 2 500 it is 8 500. If a row is smaller than the housing share, the validation reports an error. The contribution to [pillar 3a](#s-pillar3a) is not included. The app converts older plans (living expenses without housing costs) when they are opened, and a notice names the amount added. Taxes are calculated separately. A realistic value is the most important single factor for the result. Look at two to three years of bank statements for this.
- **One-off amounts:** date, amount and description. Positive is an inflow (inheritance, gift), negative an expense (car, travel, renovation).

Spending often decreases with age, for example less travel from 80, while health costs rise. Use [time-dependent values](#f-timed) for this.

## @f-timed Time-dependent values
Keywords: constant, time-dependent, row, from, course, mini chart

Many amounts and rates come in two forms:

- **Constant:** one value for the whole simulation.
- **Time-dependent:** a sequence of rows ‘from *point in time* applies *value*’. The point in time is a milestone (‘from pension fund payout’) or a fixed date. Every row applies until the next one begins.

Example living expenses: ‘from start 9 500’, ‘from pension fund payout 8 500’, ‘from 80 7 000’.

The small chart next to the field shows the course. With the mouse wheel you zoom in; the buttons “All”, ‘10 yrs’, ‘5 yrs’ and ‘2 yrs’ choose the range.

Rows that are further down but begin before another one are sorted by point in time by the app. So feel free to move milestones; the order of the values stays consistent.

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
- The value can be [time-dependent](#f-timed), for example 70 % until retirement and 50 % afterwards.

How strongly the stock allocation changes the result is shown by a [variation](#f-sweep) of the stock allocation, or by the [optimization](#s-optimize), which searches for the allocation at the same time.

## @f-reserve Cash reserve
Keywords: cash, reserve, months, liquidity reserve

How many **months of uncovered spending** (spending minus income and pensions) are held as cash **from retirement** before the rest is invested according to the [stock allocation](#f-stockshare). Before retirement, the field “Cash” under [Wealth & investment](#s-wealth) applies.

- A high reserve protects against having to sell stocks at low prices, but costs return.
- Adjustable only with the strategy “Standard” (card “Standard” under “Withdrawal strategy”) and only effective from retirement. **All other strategies** (guardrails, fixed rate, VPW, endowment) calculate with a **fixed reserve of 3 months** from retirement; with the [3-bucket strategy](#f-wd-three_buckets), the buckets replace the reserve.
- An older plan with another strategy and a reserve other than 3 months therefore calculates with 3 months and may give slightly different results. The app points this out once, the first time such a plan is opened; after that, the notice no longer appears. The stored value stays in the file and applies again if you switch to “Standard”.
- Useful as a time-dependent value: for example 6 months until retirement and 12 months afterwards.

## @s-taxes Taxes
Keywords: income tax, wealth tax, lump-sum tax, tax multiplier, table

Two modes are available: **Place of residence** calculates the income, wealth and lump-sum tax from canton, municipality and church tax with the tax data of the FTA ([Tax by place of residence](#f-tax-location)); **Own tables** calculates with rate tables and tax multiplier, as described below. Existing plans open with “Own tables”.

Simple **rate tables** that you can derive from the tax calculator of your canton. Between the brackets, the app **interpolates linearly**, and the rate is multiplied by the [tax multiplier](#f-tax-multiplier).

- [Tax by place of residence](#f-tax-location)
- [Tax multiplier and tables](#f-tax-tables)
- [Tariffs with inflation](#f-tax-indexed)
- [How accurate is the tax?](#tax-accuracy)
- **Income tax** is assessed in December on the annual income (including pensions), **wealth tax** on the portfolio plus real estate net. Both are paid in May of the following year.
- A tax still open at the end of the plan is deducted in the last month.
- The [property gains tax](#f-tax-gain) is due when a property is sold, see also [Real estate](#s-realestate).

The example values are only placeholders. For your result, replace them with values of your place of residence.

## @f-tax-location Tax by place of residence
Keywords: place of residence, canton, municipality, church tax, denomination, FTA, tax data, standard deductions, professional expenses, federal tax, taxable income, lump sum, lump-sum tax

In the mode **Place of residence** you only choose the **canton**, the **municipality** (list of the canton with search: type the name) and the **church tax** (none, Reformed, Roman Catholic, Christ Catholic). Tariffs, deductions and tax multipliers come from the bundled tax data file of the FTA (tax calculator, base data); the card names the tax year, the source and the tax multipliers of the municipality. There is no network access.

**Taxable income** = income of the year (earned income net, pension fund pension, AHV) minus the **standard deductions** (proposal of the canton), never negative. Canton and federal tax each have their own deductions:

- **Social deductions** for single persons or married couples, where the canton has them.
- **Insurance premiums and savings interest:** in years with earned income the amount “with contributions to the 2nd and 3rd pillar”, otherwise the amount “without”.
- **Flat deduction for professional expenses** on the earned income of each working person: rate, minimum and maximum of the canton, at most the earned income, no deduction without earned income. Example Zurich: 3 %, at least 2 000, at most 4 000 francs, so 3 000 with 100 000 earned income. Basel-Landschaft, Basel-Stadt and Ticino only name a maximum in the FTA data; it applies as a fixed deduction (⚠).

**Tax:**

- Income tax = tariff of the canton on the taxable income × (tax multiplier canton + church) + tariff of the municipality × tax multiplier municipality + direct federal tax on the federal taxable income. The tariff of the municipality is that of the canton, except in Schwyz and Valais.
- Wealth tax = tariff of the canton (social deductions on the wealth included) × (tax multiplier canton + municipality + church), without federal tax.
- **Lump-sum tax** (pension fund, pillar 3a) = capital rule of the canton × (tax multiplier canton + church) + rule of the municipality × tax multiplier municipality + federal tax (one fifth of the tariff of the direct federal tax, without deductions). The tax multipliers of the income tax apply; the rule applies to the whole lump sum. Lump sums in the same calendar year are added together: for married couples for the household with the tariff for married couples, for unmarried couples per person with the tariff for single persons. Below the municipality the rule of the canton is shown, for example Zurich “rate of the income tariff for one twentieth of the lump sum, at least 2 %”. The rules are derived from the FTA tax calculator; in Valais and Zug they are only approximated (⚠).
- **Married couples** (two persons and “Married” ticked): tariffs and deductions for married couples. Unmarried couples are calculated with those for single persons on the joint income.
- With [tariffs with inflation](#f-tax-indexed) the amounts of the tariffs and deductions grow with the simulated inflation.

**Not in the mode Place of residence:** the [property gains tax](#f-tax-gain) is still calculated with the own rules and the [tax multiplier](#f-tax-multiplier). The own tables for income, wealth and lump sums (also those for married couples) are hidden and stay in the file; back on “Own tables” they apply again.

**Validation:** If the canton or the municipality is missing or the municipality is not in the canton, this is an error in the section Taxes, and “Start simulation” is blocked. The overview of the assumptions and the tab “Parameters” name the place of residence, for example “Zürich (ZH), church tax none”.

## @f-tax-multiplier Tax multiplier
Keywords: canton, municipality, multiplier

A factor on all rates of the tables, the tax multiplier of **canton and municipality** together (for example 2.0 for a double rate). This way, you do not have to rewrite the tables when you move; you only change the factor. Values from 0 to 10 are allowed. In the mode [Place of residence](#f-tax-location) it applies only to the property gains tax.

## @f-tax-indexed Tariffs with inflation
Keywords: bracket creep, inflation, brackets, tariff, today's prices

With the tick, the **amounts** of the tables for income, wealth and lump sums apply in **today's prices** (prices at the simulation start) and grow with the simulated inflation of the respective path; the **rates** stay the same. This way the tax burden does not rise merely because of inflation (compensation of bracket creep), as the Confederation and the cantons handle it.

- **Example:** table 0 → 0 %, 200 000 → 10 %, inflation 2 % per year. In year 10 the upper bracket is at 243 799. An income of 121 899 (100 000 with inflation) has the rate 5 % like 100 000 in the start year; without the tick it would be 6.09 %.
- The price index in the month of the assessment applies: December for income and wealth, the month of the payout for lump sums. The estimated income tax in the need of the [3-bucket strategy](#f-wd-three_buckets) is computed the same way.
- This also applies to the tables for married couples.
- The [property gains tax](#f-tax-gain) stays unchanged; its tariff refers to the single gain.
- New plans have the tick set. Older plans without the setting and imported old configurations compute as before without it: the amounts then apply nominally over the whole plan.

## @f-tax-gain Property gains tax
Keywords: property gain, sale, property, holding period, Zurich, Aargau, canton, exemption limit, tariff, tax multiplier

The tax on the gain when a property is **sold**. The rules **differ from canton to canton** and are therefore fully editable. **Required:** if the plan contains a property with the status “Owned” or “Purchase planned”, the rules must be set (Zurich template, Aargau template or your own rules, also with a rate of 0 %). If they are missing, this is an error in the section Taxes, and “Start simulation” and “Start optimization” are blocked. Pure rental properties and plans without a property need no rules.

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
- **Married:** With two persons and “Married” ticked, you can store separate tables for married couples. Without separate tables, the normal ones apply.
- Start from the **effective** tax burden, not the marginal tax rate. The tax calculator of your canton provides both.

## @tax-accuracy How accurate is the tax?
Keywords: estimate, estimated tax, accuracy, rounding, rounded, reference cases, approximation, deductions, tax advice, range

The tax in VivaPlan is an **estimate for planning**, not a tax assessment. The results therefore call it “estimated tax” and show tax amounts **rounded to 100 francs**. The calculation is exact, the rounding applies to the display only; the command line and its CSV export output the exact values.

**Reference cases:** The calculation rules are tested with cases recalculated by hand: rate from the table interpolated linearly times the tax multiplier, lump-sum withdrawals of a year combined for the rate, steps and holding period of the property gains tax. The Canton of Zurich template of the property gains tax matches the official table. So VivaPlan calculates correctly with your tables; how close this is to your actual tax depends on the tables.

**Mode “Place of residence”:** 25 made-up cases in Zurich, Bern, Lucerne, Aarau and St. Gallen (income 80 000 to 300 000, wealth 300 000 to 4 million, retired persons, without children and denomination) deviate less than 1 % from the FTA tax calculator 2026; at most ±5 % is required. Such comparison values are still missing for working persons. Individual deductions, children, income-dependent deductions, travel and meal costs are not included.

**Lump sums in the mode “Place of residence”:** 48 made-up cases in Zurich, Bern, Lucerne, Aarau, St. Gallen, Geneva, Fribourg and Bellinzona (lump sums of 100 000, 500 000 and 1 million, single and married) deviate at most ±5 % from the FTA tax calculator 2026. Valais and Zug are only approximated (in Zug up to about 7 %).

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

VivaPlan does not show a **range** (± %): there is no basis for it, and another number would suggest a precision that does not exist.

**Lump-sum tax and property gains tax** often involve a lot of money: check with the canton’s tax calculator or a professional before deciding. VivaPlan is not tax advice.

## @s-realestate Real estate
Keywords: house, apartment, mortgage, rent, purchase, sale, affordability

Owned, planned and rented properties. You can enter as many as you like.

- **Owned (‘owned’):** The market value at simulation start is in the header next to the name. Below follow, in fixed places, [purchase date](#f-re-bought), purchase price (paid, for the property gains tax), “Sale from” (never, milestone or date) and fixed costs per month. With “Purchase planned”, the same fields are in the same places (“Purchase from” instead of the purchase date), only the market value is missing; when the status changes, the values entered are kept. The sale must be **after the purchase date**; otherwise it is an error at the field.
- **Inflation:** Value and fixed costs can be carried forward with inflation **individually** for each property, and so can the rent for a rental.
- **Planned purchase (‘planned’):** purchase from milestone, purchase price, mortgage. In addition, there is **“Sale from”** (default ‘Never’, otherwise date or milestone); the simulation then sells the property at that point in time. The sale must be after “Purchase from”; otherwise it is an error at the field. For a purchase ‘after sale of …’, the point of purchase is not known when entering it, and this check is omitted.
- **Rent (‘rent’):** start and rent per month (today's value, carried forward with inflation by default; the option “Adjust rent for inflation” turns this off).
- **Mortgage:** tranches with periods, see [Mortgages](#f-re-mortgage).
- [Emergency sale](#f-re-emergency) and [affordability](#f-re-afford) for purchases “after sale”.
- The switch [Never sell properties](#f-re-never-sell) at the top of the tab rules out any sale.

You enter [living expenses](#s-flows) **including** housing costs. The calculation uses living expenses **without the housing share**; rent, mortgage interest, amortization and fixed costs are added as simulated **housing costs**. After the sale of the last property, the next one with a purchase “after sale” is bought in the following month, provided it is affordable and the equity is sufficient. Otherwise the rents ‘from after sale’ begin. A purchase “after sale” is attempted **once** in the following month. If it cannot be financed, it is **dropped** with a message in the events, and the follow-up rules apply in the next month: rents ‘from after sale’ of this property begin, and further purchases “after sale” of it are attempted. A **purchase or sale date before the simulation start** is never reached; the validation reports it as a warning.

## @f-re-bought Purchase date
Keywords: holding period, property gains tax, purchase

The month in which you bought the property. It determines the **holding period** and thus the surcharge or reduction of the [property gains tax](#f-tax-gain): in many cantons, the longer you own the property, the less tax is due.

- The field only exists for owned properties. For a **planned** property, the month of purchase in the simulation counts.
- Without a purchase date, the app calculates **without** surcharge or reduction (the tax according to the basic tariff). If holding period rules are entered, the validation points this out.
- What counts is the holding period in **full years** until the month of sale.

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

## @f-re-afford Affordability
Keywords: bank, affordability, loan-to-value, imputed interest rate, equity, mortgage

The banks' affordability calculation, applied to purchases “after sale” and at every renewal of a mortgage period. A mortgage is affordable if the **imputed costs** do not exceed a share of the income.

- **Max. share of income:** usually one third (33 %).
- **Imputed interest rate:** usually 5 %, regardless of the current interest rate.
- **Fixed costs:** usually 1 % of the price per year.
- **Income:** Only earned income and pensions are counted, **not** the withdrawal from wealth. If the income is not sufficient, the mortgage is reduced; the purchase then shows the **one-off repayment** from wealth as an event.
- **Maximum loan-to-value:** usually 80 % of the price. At least 20 % is equity.

If the purchase is not affordable, the mortgage is reduced. If the equity is still not sufficient, there is no purchase and the rents begin.

**At every renewal** of a mortgage period (also for owned properties), the app checks the affordability of the whole mortgage again, with the income of the month of renewal (earned income and pensions). If the mortgage is no longer affordable, the simulation calculates the **minimum one-off payment** needed to make it affordable again and pays it from wealth (proportionally across all tranches). This only applies if wealth is **not negative** afterwards **and** the [emergency sale criterion](#f-re-emergency) is **not** triggered after the payment (estimated with the planned spending and the housing costs after the payment). Then the mortgage is renewed with the reduced amount. Otherwise there is **no payment**, the mortgage is **not renewed**, and an **emergency sale** of the property is initiated (carried out in the following month). The result view logs every renewal (date and age, affordability, one-off payment, new mortgage, emergency sale with the reason “Wealth < 0” or “Emergency sale criterion”) and, with several runs, counts in how many runs one-off payments and emergency sales occur, also per percentile. Typical case: income drops at retirement.

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
