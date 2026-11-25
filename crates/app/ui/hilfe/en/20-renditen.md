# Simulation and returns

## @s-returns Simulation: overview
Keywords: return, inflation, source, method, time series, Monte Carlo, SARON, investment strategy

The section “Simulation” determines **how the returns of stocks and bonds, inflation and the SARON develop over time**: fixed values, a historical course or Monte Carlo. It has the greatest influence on what the results look like and how much you can trust them.

**Which indices** supply the returns (for example world stocks and bonds of the Swiss Confederation) you set in the [investment strategy](#f-ret-series) in the section “Wealth & investment”. You no longer choose a time series yourself; it follows from the strategy.

There are five methods. The view only shows the inputs that apply to the chosen method.

| Source | Runs | What for |
|---|---|---|
| [Fixed returns](#f-ret-fixed) | 1 | Quick estimate, preview, plausibility check |
| [Historical from start date](#f-ret-historical) | 1 | ‘What if I had retired in 1973?’ |
| [All historical start dates](#f-ret-rolling) | approx. 400 to 900 | How every real start would have turned out |
| [Monte Carlo · historical (bootstrap)](#f-ret-bootstrap) | free, e.g. 50 000 | Wide dispersion from real monthly data |
| [Monte Carlo · parametric](#f-ret-parametric) | free, e.g. 50 000 | Model with suggested values per index or your own assumptions |

If you do not know where to start, read [Which source suits you?](#ret-which).

Basics that apply to all sources are under [Return, inflation and real values](#ret-concepts). Common mistakes in reasoning are described under [Pitfalls](#ret-pitfalls).

## @ret-which Which source suits you?
Keywords: decision, recommendation, choice, choosing a source

A rule of thumb:

1. **For entering and trying things out:** “Fixed returns”. The preview on the right always calculates with fixed values and reacts immediately.
2. **For the actual answer to ‘Will the money last?’:** Monte Carlo, preferably **“Monte Carlo · historical” (bootstrap)** with an investment strategy that goes back a long way (for example “World (JST)” or “S&P 500 (Shiller)”). It preserves the real properties of the markets (crises, recoveries, the relationship between stocks, bonds and inflation) and still has a wide dispersion.
3. **As a cross-check:** “All historical start dates”. It shows concretely how the plan would have turned out in real periods, for example with a start in 1929, 1966 or 2000.
4. **For your own assumptions:** “Monte Carlo · parametric”, if you want to specify the expected return and the risk yourself, for example because you expect lower returns in the future than in the past.

**Good practice:** Calculate important decisions with **at least two sources** (bootstrap and parametric) and check whether the conclusion stays the same. A plan that only looks good with a single source is fragile.

| Question | Source |
|---|---|
| How likely is it that my money lasts? | Bootstrap or parametric |
| How bad can it get? | Bootstrap, 1st percentile and worst run |
| Would the plan have worked in 1973? | Historical from start date |
| Does the plan work in every historical situation? | All historical start dates |
| What if the stock market delivers less in the future? | Parametric with a lower μ |

## @ret-concepts Return, inflation and real values
Keywords: nominal, real, inflation, geometric, annual return, purchasing power

**Annual returns** are applied monthly in the app. An annual return of 5 % gives (1.05)^(1/12) − 1 per month, i.e. about 0.41 %. The course over the year is therefore exact.

**Inflation** takes effect every month. The price index grows monthly, and all spending, rents and (if chosen) property values are carried forward with it. You enter your spending in **today's prices**; the app projects it into the future with inflation.

**Nominal and real:** The results come in two views.

- **Nominal:** amounts as they occur at the time (in future francs).
- **Today's purchasing power:** amounts in prices of the simulation start, adjusted for inflation. This view is usually the more honest one for decisions, because 2 million in 30 years has less purchasing power than today. The results open in this view.

With several runs, the percentiles belong to the chosen view: in today's purchasing power the runs are ordered by the real final wealth, nominal by the nominal one (see [Final wealth by percentile](#r-percentiles)).

**Arithmetic and geometric:** An ‘average return of 7 %’ is usually an arithmetic average. The **geometric** average (how much your wealth really grows) is lower because of the fluctuations. With an expectation of 7 % and a volatility of 15 %, it is about 5.9 %. The short info of the investment strategy shows the mean annual return geometrically.

**Total return:** All series include dividends and coupons (total return), no investment costs, no taxes on income.

## @f-ret-source Source of the returns
Keywords: choice, fixed, historical, rolling, bootstrap, parametric

Choose the source with the cards at the top of the section. Every card has an ⓘ with the detailed explanation:

- [Fixed returns](#f-ret-fixed)
- [Historical from start date](#f-ret-historical)
- [All historical start dates](#f-ret-rolling)
- [Monte Carlo · historical](#f-ret-bootstrap)
- [Monte Carlo · parametric](#f-ret-parametric)

If you are not sure which one suits you, read [Which source suits you?](#ret-which).

## @f-ret-fixed Fixed returns
Keywords: constant, stocks, bonds, inflation, SARON, single run

A single run with **constant annual values** for stocks, bonds, inflation and SARON (for SARON mortgages).

- There is **no dispersion**: the result is that the money either lasts or does not. There is no probability of success.
- Good for entering data, for quick comparisons and for the preview on the right. **Not suitable** for judging whether the plan is safe: it ignores the sequence risk completely.
- **Choice of values:** Calculate with cautious, **realistic** values, not with the peak years of the past. A sober assumption would be, for example, 4 % to 5 % for stocks, 1 % to 2 % for bonds and 1 % to 2 % inflation.
- With the strategies “Fixed rate”, “Guardrails” and others, a run with fixed returns only shows the average case.

With the other sources, the fixed values still partly apply: for the **SARON** outside the period of the series and with the [parametric model](#f-ret-parametric), and after the end of the series with the [extrapolation](#f-ret-extrapolation).

## @f-ret-historical Historical from start date
Keywords: review, start month, 1973, 1929, backtest, time series

A single run with the **real course of returns** of the [investment strategy](#f-ret-series), starting in the **historical start month**. The simulation then runs as if you had started in that month.

- **Example:** Start month 01.1973 checks what would have happened if you had retired with your plan at the beginning of the oil crisis (high inflation, weak stock market).
- The simulation runs over the period of the plan, for example 32 years from the start month. If the series ends earlier, the [extrapolation](#f-ret-extrapolation) takes over.
- With annual strategies (everything except S&P 500 with US bonds or with bonds of the Swiss Confederation), only the year counts.
- **Single runs are anecdotes.** A plan that would have worked in 1973 can fail in another situation. For a statement about security, use [all start dates](#f-ret-rolling) or [Monte Carlo](#f-ret-bootstrap).
- With a [variation of the start month](#f-sweep), a whole series of historical starts can be calculated. This is similar to “All historical start dates”, but with a grid of your own choosing.

## @f-ret-rolling All historical start dates
Keywords: rolling, iteration, overlap, start date

The app calculates **every possible start month** for which the series covers the whole planning period. With a series from 1922 and a plan over 32 years, that is a few hundred runs: start in January 1922, February 1922 and so on up to the last month that still leaves 32 years until the end of the series.

**Strengths:** Every run is a **real** period with real relationships between stocks, bonds and inflation. You see the historically worst starts (for example 1929, 1966, 1973, 2000).

**Weaknesses you should know about:**

- The runs **overlap heavily**. 104 years of data give 800 months of start points, but only a little more than **three truly independent 32-year periods**. The dispersion is therefore **underestimated**, and ‘98 % of the runs are successful’ sounds safer than it is.
- There is only **the one** past. A worse sequence that never happened does not occur.
- With long planning periods and short series, there are very few start points. With an annual series (for example “Switzerland (JST)”, 121 years) and a 32-year plan, there are only about 90.

The [bootstrap](#f-ret-bootstrap) partly fixes these weaknesses and therefore shows wider ranges. Use “All historical start dates” as a cross-check, not as the only source.

The section “Run” shows the **number** of runs.

## @f-ret-bootstrap Monte Carlo · historical (bootstrap)
Keywords: bootstrap, block length, paths, random seed, seed, stationary, Monte Carlo

The bootstrap creates **thousands of new return paths** by joining random sections of the time series of the investment strategy. This produces courses that never existed but consist of **real monthly data**.

**How it works:**

1. The path starts at a random month of the series.
2. In every month, the current **block** ends with the probability 1 / block length. Then a new block starts at a random start month. At the end of the series, it continues at the beginning.
3. Stocks, bonds, inflation and SARON always come **from the same historical month**. The relationships (for example a stock market crisis with high inflation) are preserved.

**Inputs:**

- **Number of paths:** More paths give smoother numbers, but longer calculation time. 10 000 are enough for a first answer, 50 000 for percentiles at the edge (1st percentile, worst run). The success rate is a random result: with 10 000 paths, it is accurate to about ±0.4 percentage points, with 50 000 to about ±0.2 (roughly two standard errors at a success rate around 95 %). **Upper limit:** paths × months of the plan at most 50 million, so that the memory is sufficient (with 30 years about 138 000 paths, with 100 years 41 666). Above that, the validation reports an error at the field with the highest possible number.
- **Mean block length (months):** How long connected sections are on average.
  - **Short (1 to 6):** The months are mixed almost independently. This destroys the property that bad years are often followed by more bad years (**momentum and recovery**) and makes the paths **too smooth**.
  - **Long (24 and more):** The paths stay close to history, and the variety drops.
  - **12 months** (default) is a proven compromise. With annual series, the length applies in years (block length / 12).
- **Random seed:** Same value, same paths. See [Random seed](#ret-seed).

**Properties:**

- The paths are dispersed **more widely** than with “All historical start dates”, because they are not bound to the one order of history. That is the more honest uncertainty.
- The bootstrap can only show what is **in the data**. High returns in the past lead to high returns in the future. If you want to calculate more cautiously, use the [parametric source](#f-ret-parametric) with a lower μ, or a shorter series that does not only contain boom years.
- The detail views (5th, 50th, 95th percentile and the extreme runs) recalculate the respective path exactly.

## @f-ret-parametric Monte Carlo · parametric
Keywords: Student t, fat tails, volatility, drift, price jumps, mu, sigma, degrees of freedom, Poisson, crash, suggested values, correlation, bonds, inflation

A **stock model with your own or suggested assumptions**. Stock prices follow a random walk in **daily steps** (21 per month), supplemented by rare **price jumps**. **Bonds and inflation** are also random and **correlated** with the stocks, as in the series of the [investment strategy](#f-ret-series). The SARON is held constant at the fixed value.

| Input | Suggestion (example World (JST)) | Meaning |
|---|---|---|
| Number of paths | 50 000 | as with the bootstrap |
| Expected return (drift μ) | 12.15 % | drift of the log return plus σ²/2; the mean (arithmetic) annual return without jumps |
| Volatility σ | 12.25 % | fluctuation of the annual returns without the jumps |
| Degrees of freedom | 6 | thickness of the tails (fat tails), 0 = normal distribution |
| Price jumps per year | 0.15 | how often a crash occurs (0.15 = about every 7 years on average) |
| Mean jump size (log) | −25 % | size of the jump; −25 % in log return is about −22 % price loss |
| Spread of the jump size | 15 % | how different the jumps are |
| Random seed | | see [Random seed](#ret-seed) |

**Suggested values per stock index:** The app calculates the parameters from the whole stock series of the chosen index, so that the annual returns of the model (mean, dispersion, geometric mean and 5 % quantile) match those of the [historical bootstrap](#f-ret-bootstrap). Degrees of freedom (6) and spread of the jump size (15 %) stay fixed.

| Stock index | μ | σ | Jumps per year | Jump size | Years |
|---|---|---|---|---|---|
| World (JST) | 12.15 % | 12.25 % | 0.15 | −25 % | 1900–2020 |
| Switzerland (JST) | 10.55 % | 14.5 % | 0.26 | −10 % | 1900–2020 |
| S&P 500 (Shiller) | 15.05 % | 16.5 % | 0.26 | −20 % | 1922–2025 |

“World, custom (JST)” gets values from its own weighted series. After an update of the Shiller data, the values for the S&P 500 change as well. For the S&P 500, the bootstrap draws months; its years are therefore somewhat milder than the calendar years, and the suggested values follow the bootstrap.

- **New plan:** The values follow the stock index. If you change the index under “Wealth & investment”, the suggested values of the new index apply.
- **Your own values:** As soon as you change one of the six model values, your values stay, even if the index changes. Next to every field, the suggestion for the chosen index is then shown. With **“Adopt suggested values”**, the values follow the index again.
- **Older plans** that already calculated parametrically keep their values. When they are opened, a notice appears once that bonds and inflation are now random.

**Bonds and inflation:** The card “Bonds and inflation” shows mean, dispersion and the correlation of stocks, bonds and inflation, calculated from the yearly values of the same series that the bootstrap uses. It depends on the combination of stock and bond index. Examples of the stock/bond correlation: World with Swiss Confederation 10 years −0.01, Switzerland with Swiss Confederation 10 years 0.19, S&P 500 with USA 10 years 0.45 (both carry the dollar exchange rate). Every calendar year, the model draws a value for bonds and inflation that is correlated with the stock return of the same year. The years are independent of each other as in the bootstrap; there is no memory for inflation or interest rates. Without a time series, the fixed values apply to bonds and inflation.

**Mathematics in brief:** The log return per day is (μ − σ²/2)·dt + σ·√dt·Z. Z is a **Student t distributed** shock, scaled to variance 1. In addition, there are **Poisson jumps** with the rate “Price jumps per year” and a log-normal size. Bonds and inflation of a year are normally distributed with the mean and dispersion of the series; their dependence on the standardized log return of the stocks is chosen so that the correlation with the stock return matches the series.

**How to choose your own values:**

- **μ:** In the long run, broadly diversified stocks have historically delivered about 5 % to 7 % per year in real terms, correspondingly more in nominal terms. If you plan cautiously, take lower values than the past, because valuations are high today. Note: the price jumps pull the return down further; the expected geometric return is shown in the preview on the right.
- **σ:** 12 % to 20 % is typical for stocks; smaller with frequent jumps, because the jumps carry part of the fluctuation.
- **Degrees of freedom:** Low values (3 to 5) produce extreme days, 6 is moderate. With 0, the fat tails disappear, and crises become unrealistically rare.
- **Price jumps:** Together with the fat tails, they ensure that crashes like 2008 or 2020 occur plausibly. If you set the rate to 0, only the random walk remains.

**When it makes sense:** as a cross-check of the bootstrap with smooth distributions instead of the limited number of historical years, and when you want to set your own assumptions (for example ‘4 % less return in the future’). **Limitations:** Swiss stocks revert to the mean over several years; neither the bootstrap nor the model reflects this. The correlation is averaged over all years; in years of high inflation, stocks and bonds have historically fallen together more often. With “S&P 500 (Shiller)” and “Swiss Confederation 10 years”, the series only covers the years from 1988, but the stock parameters cover the whole series from 1922.

## @f-ret-series Investment strategy: indices for stocks and bonds
Keywords: investment strategy, index, world, JST, Pictet, Shiller, S&P 500, weights, regions, EU, USA, Switzerland, Asia-Pacific, time series, SNB, currency, short-term, money market

The methods “Historical from start date”, “All historical start dates” and bootstrap need a time series with returns of stocks, bonds and inflation. It follows from the **investment strategy** in the section “Wealth & investment”: one index each for the stocks and for the bonds.

**Stocks**

| Index | Content |
|---|---|
| **World (JST)** | Stocks of 11 countries (Switzerland, USA, Germany, France, Italy, Spain, Netherlands, Belgium, United Kingdom, Japan, Australia), weighted every year by the real GDP of the previous year, in CHF without currency hedging. **Annual 1900–2020.** |
| World, custom (JST) | Like “World (JST)”, but you set the weights of **Switzerland, USA, EU and Asia-Pacific** yourself (100 % in total). Within the EU (Germany, France, Italy, Spain, Netherlands, Belgium, United Kingdom) and Asia-Pacific (Japan, Australia), the countries are weighted by GDP. If the weight of the EU changes, the share of every EU country changes in the same proportion: EU 20 % instead of 40 % halves every EU country. The weights apply anew every year (annual rebalancing). |
| Switzerland (JST) | Swiss stocks, total return. Annual 1900–2020. |
| **S&P 500 (Shiller)** | US stocks (S&P Composite) in CHF, without currency hedging. **Monthly from 1922.** |

**Bonds**

| Index | Content |
|---|---|
| **Swiss Confederation 10 years** | 10-year bonds of the Swiss Confederation: JST until 1988, then from the monthly yields of the SNB. Annual 1900–2024; together with the S&P 500 monthly from 02.1988. |
| USA 10 years (Shiller) | 10-year US Treasury bonds in CHF, without currency hedging. Monthly from 1922. |
| Swiss Confederation short-term | Short-term paper of the Swiss Confederation (term about 3 months): JST until 2020, then the annual average of the SARON (SNB). **Annual 1900–2025.** The return practically equals the money market rate: hardly any fluctuation, but in the long run considerably less return than 10-year bonds. |

**How the time series is created**

- **Monthly** are only “S&P 500 (Shiller)” with “USA 10 years (Shiller)” (from 01.1922) and “S&P 500 (Shiller)” with “Swiss Confederation 10 years” (02.1988 to 07.2025).
- All other combinations are **annual** as soon as one part is annual. Monthly returns are compounded to calendar years for this. The series covers the years for which both indices have values: for example “World (JST)” with “USA 10 years (Shiller)” 1922–2020.
- **Inflation** comes from the source of the stocks (Swiss inflation in all cases), the **SARON** from the SNB values (for annual series as an annual average, from 2000).
- All indices are **in CHF**. With a plan in EUR or USD, a notice appears.

Below the selection are the period, the frequency and the mean annual returns (geometric) of the chosen strategy; the ⓘ next to it names the source and the period.

**Common period:** The mean returns apply to the years that stocks and bonds have in common. If one index goes further, it is cut to this period, and a notice below the selection names the index, its own years and the shortened period. The simulation (Monte Carlo and historical courses) only uses the common period. This is why, for example, the mean return of the bonds “USA 10 years (Shiller)” changes when you switch the stocks from “S&P 500 (Shiller)” (until 2025) to “Switzerland (JST)” (until 2020): the bonds then only count until 2020.

**How do you choose?**

- **World (JST):** broadly diversified, long history, including world wars and hyperinflation. Caution: the index is rather **optimistic**. For 1900–2020, it is about 1.3 percentage points per year above the well-known world index of Dimson, Marsh and Staunton (5.3 % real in USD), mainly because of the annual weighting by GDP and because it only contains countries that still exist today. See [Data sources](#ret-data).
- **S&P 500:** monthly and therefore the most accurate for bootstrap and rolling; but it only contains US stocks and the high return of the US stock market.
- **Switzerland (JST):** the investment universe of a Swiss investor who invests in the home market.
- **World, custom:** if you want to reflect your own allocation, for example 30 % Switzerland, 40 % USA, 20 % EU, 10 % Asia-Pacific.

**Older plans:** A plan with a chosen time series gets the matching strategy when it is opened and calculates the same as before: ‘Shiller CHF’ → S&P 500 with USA 10 years, ‘Shiller CHF + CH-Obligationen’ → S&P 500 with Swiss Confederation 10 years. ‘Shiller USD’ becomes S&P 500 with USA 10 years in CHF; **your own time series** (CSV files) are no longer supported and are replaced by “World (JST)” with bonds of the Swiss Confederation. **Pictet:** The index “Switzerland (Pictet)” is no longer bundled for licensing reasons. A plan with ‘Pictet CH’ or with “Switzerland (Pictet)” gets “Switzerland (JST)” for the stocks and “Swiss Confederation 10 years” for the bonds. In all these cases, the app reports this when the plan is opened, and the result changes.

**Updating:** “Update Shiller data…” imports a new Shiller file (.xls, .xlsx or .ods; download the file `ie_data.xls` from shillerdata.com), “Update SNB data…” new cubes from data.snb.ch. The app never downloads data from the internet itself; you download it by hand. The JST data end in 2020 and are built in.

More background: [Data sources and their particularities](#ret-data).

## @f-ret-extrapolation Extrapolation of the series
Keywords: end of the series, average, fixed, extrapolate

If the time series ends before the end of the simulation (for example ‘Historical from 2000’ with a plan until 2058), the app has to fill the missing years. You set the **method** here:

- **Fixed values:** The fixed returns from the section “Fixed returns” apply.
- **Moving average:** Every missing value is the (arithmetic) average of the last *n* years. The window moves on and, over time, also contains the values already extrapolated. The extrapolation therefore converges toward a calm mean.
- **Moving geometric average:** as above, but averaged geometrically.

The period (“Average over” … years) applies to the two moving methods. This input only appears if the series really ends before the plan is finished.

Note: an extrapolation is an **assumption**, not a historical result. With very long extrapolations, the result loses its historical character.

## @ret-saron SARON and mortgages
Keywords: SARON, mortgage, interest, margin, time series

The **SARON** is only needed if a mortgage has a SARON period. It comes from the time series (from 06.1999, if available); otherwise the **fixed SARON value** of the plan applies.

- Outside the SARON period of the series, with “Fixed returns”, with the parametric model and after an extrapolation, the fixed value applies. The view only shows the field when it is needed.
- The interest rate of a SARON tranche is SARON plus margin, with a floor of 0.
- Before 1999, there is no SARON. For historical runs before 1999, use the fixed value as an approximation.

More on tranches and periods: [Mortgages](#f-re-mortgage).

## @ret-seed Random seed and reproducibility
Keywords: seed, chance, same results, reproducible

Monte Carlo is random, but **reproducible**. Path *i* always arises from the pair (random seed, *i*).

- The same settings and the same seed give **exactly the same results**.
- All **scenarios** use the same paths. In the **optimization**, too, all candidates see the same paths (the check at the end uses different ones). This way, differences come from the parameters, not from chance.
- The results are the same on macOS, Windows and Linux.
- With a new version, the procedure that creates the random paths can change. The app mentions this when an older plan is opened. For example, “Monte Carlo · parametric” has created the paths faster since #132: the distribution of the returns stays the same, but the individual paths are different, and the result changes slightly.
- Change the seed if you want to check whether your result depends on chance. If the success rate fluctuates by more than a few tenths of a percentage point with a different seed, you need more paths.

## @ret-data Data sources and their particularities
Keywords: Shiller, SNB, Pictet, monthly average, bonds, currency

- **Shiller (Robert J. Shiller, *Irrational Exuberance*):** stocks as the S&P Composite with dividends. Shiller's prices are **monthly averages**, so crashes appear smoothed. Monthly return = change of the real total return index × CPI compared with the previous month. The bonds are the total return of 10-year US Treasury bonds.
- **Conversion to CHF:** with the monthly average of the USD/CHF exchange rate of the SNB, without currency hedging, and with Swiss inflation (Swiss consumer price index compared with the same month of the previous year). This creates the relationship that a rising franc lowers the return of a Swiss investor in US stocks.
- **Swiss bonds:** The total return of a 10-year federal bond (at par) is calculated from the monthly yield of the SNB: coupon = yield of the previous month, valuation at the new yield with just under 10 years of remaining term, plus one monthly coupon. The SNB has no longer updated this series since 07.2025, so ‘Shiller CHF + CH-Obligationen’ ends there.
- **JST (Jordà-Schularick-Taylor Macrohistory Database, Release 6):** annual returns of stocks and government bonds per country until 2020, converted to CHF with the year-end exchange rates. Weight of a country: real GDP of the previous year (GDP per capita according to Maddison × population). Missing country-years are dropped (Germany 1945–1946, Japan 1946–1947); Germany 1923 is excluded because the exchange rate during the hyperinflation does not match the annual return. The Swiss stocks are based on Pictet from 1926. Inflation: Swiss consumer prices (annual average). **License CC BY-NC-SA 4.0:** non-commercial only, with attribution (Jordà, Schularick, Taylor 2017; Jordà, Knoll, Kuvshinov, Schularick, Taylor 2019). Comparison with Dimson, Marsh, Staunton (1900–2020): World (JST) 6.6 % real per year in USD, DMS 5.3 %; the country series themselves agree (USA 6.7 % versus 6.6 %).
- **Bonds of the Swiss Confederation before 1989:** from JST (total return); 1915 is missing (stock exchange closed), and there the coupon applies as the return.
- **Inflation** is, for all series, the change of the consumer price index compared with the previous year, so that the December value equals the annual inflation.
- **No costs:** Investment costs, taxes on income and custody fees are not deducted.
- **Rights and warranty:** The bundled series are returns calculated from the sources named. They are not covered by the license of the program (GPL v3); the rights to the underlying data belong to the sources (Robert J. Shiller, shillerdata.com; Swiss National Bank; Federal Statistical Office; Jordà, Schularick, Taylor, macrohistory.net, license CC BY-NC-SA 4.0). Shiller points out that neither he nor Yale University gives any warranty for the data and that they are neither a forecast nor investment advice. The same applies to this app.

## @ret-pitfalls Pitfalls of the returns
Keywords: mistakes, caution, survivorship, home market, over-optimism, sequence risk, currency

- **The past is not a forecast.** The US stock market was an exceptionally successful market in the 20th century. Series with US stocks contain this success and can be too optimistic (survivorship bias). This also applies to “World (JST)”, which only contains countries that still exist today. Also check the plan with lower returns (parametric with μ = 4 % to 5 %) or with Swiss stocks.
- **Overlapping periods create a false sense of security.** See [All historical start dates](#f-ret-rolling): few independent periods, but hundreds of runs.
- **A block length that is too short in the bootstrap** makes the paths too smooth. Stay at 12 months or more.
- **Sequence risk:** What matters is **when** the bad years come. A crash in the first years of retirement has a much stronger effect than later. Fixed returns do not show this.
- **Currency:** All indices are in CHF; a plan in EUR or USD still calculates with franc returns and Swiss inflation (notice in the validation). CHF series with foreign stocks also carry currency risk. The appreciation of the franc over the last decades has lowered the return of a Swiss investor in US stocks.
- **Confusing nominal and real:** ‘2 million at the end’ is nominal. In today's purchasing power, it is considerably less.
- **A single source.** If you only calculate with one return source, you get a **model-dependent** answer. Calculate important questions with bootstrap and the parametric model.
- **The costs are missing.** Real investment costs (funds, custody) lower the return by a few tenths up to one percent per year. Deduct them from the fixed or parametric values.
