# Optimization

## @s-optimize Optimization: overview
Keywords: best strategy, parameters, ruin, consumption, search, goal

The optimization answers the question: **Which withdrawal strategy with which settings suits my base scenario best?**

It takes your plan as given and searches, for every selected [withdrawal strategy](#s-withdrawal), for the parameters that achieve three things at the same time:

1. The **probability of ruin** stays below your limit (for example 5 %).
2. **Consumption** is as high as possible (median and poor year, real).
3. Consumption **fluctuates** as little as possible from year to year.

The strategies are then compared. The best one is at the top and carries a star ★. Click “Apply to plan” to transfer the strategy and parameters to your plan.

**Requirements:**

- A return source with **dispersion**: Monte Carlo (bootstrap or parametric) or “All historical start dates”. With “Fixed returns” and “Historical from start date”, there is only one path, and ruin would be 0 % or 100 %. The app reports this and blocks the start.
- A **retirement** in the plan (pension fund “Payout from”), because the strategies apply from then on.

**How long does it take?** Below the button “Start optimization” there is an **estimate of the duration**, which adapts to every change of the settings. It is based on a measurement: the app briefly calculates a few thousand runs of your plan and extrapolates from this how many runs the optimization needs (paths of the search, start points and strategies, plus the paths for checking). The figure is **rough**: the number of search steps depends on the case, so a range is given (‘mostly 7 s to 20 s’). Examples on a current Mac with 10 cores: default settings with all six strategies take about 15 s with bootstrap, just as long with the parametric model, and 2 to 3 s with “All historical start dates”. Twice as many paths take about twice as long. The calculation runs locally and uses all processor cores.

**During the optimization**, a progress bar shows the status (the percentage is an estimate, because the number of search steps depends on the case). With **Cancel** you stop the calculation; no partial result remains.

**Which parameters are varied** you decide yourself for each strategy, see [Fixed or varied parameters](#o-ranges).

**The settings are saved** (goal, strategies, paths, ranges of the parameters) **with the plan**: they are in the `.vplan` file, come back when you open it and also apply to `optimize` in the terminal (options override them there). The result of an optimization is not saved. A changed setting does not invalidate an earlier result; only a changed plan does.

Read the results with [Reading the result](#o-read) and note the [pitfalls](#o-pitfalls). An optimum is **not a promise**.

## @o-goal-select Choosing the goal
Keywords: goal, no emergency sale of real estate, emergency sale, minimum consumption

The optimization has two goals; the choice is saved with the plan (default: goal 1).

- **Goal 1, consumption with limited risk of ruin:** as described in this chapter: consumption as high and as steady as possible, as long as ruin stays below the limit.
- **Goal 2, no emergency sale of real estate:** The search looks for the strategy with the **lowest share of runs in which a property is sold unplanned** (emergency sale). If the share is equal, the lower probability of ruin decides (ruin as in goal 1). Constraints: the **median of annual consumption from retirement** is at least the entered share of the planned spending (default 90 %), and the probability of ruin is below the limit (default 5 %). Consumption weights and volatility play no role in goal 2.

The result table names the chosen goal. For goal 2, it shows the **share of runs with an unplanned sale** for each strategy; the order follows this share, and in case of a tie, the ruin. Strategies that violate the constraints are at the bottom and marked. If no strategy meets the constraints, the one that comes closest is at the top, with a notice.

Goal 2 requires at least one property in the plan (owned or planned); without a property, it cannot be selected.

## @o-goal Goal: probability of ruin
Keywords: ruin, limit, age, depletion, 5 percent, risk

The probability of ruin is the central **constraint**. A strategy that does not meet it cannot win, no matter how high its consumption.

- **Maximum probability of ruin** (default 5 %): the highest share of paths allowed in which wealth is depleted. **Ruin** means: no liquid wealth and no real estate left. 1 % to 10 % is usual. The lower the limit, the safer, and the lower the possible consumption.
- **Ruin counts until age** (empty = until the simulation end): If you only need security up to a certain age, enter it here. A ruin at 92 then does not count with ‘until 90’. This is a **deliberate relaxation**; think of longevity. With an empty field, the limit applies until the end of the simulation.

> Which limit is reasonable? A probability of ruin of 5 % means that the money runs out in 1 of 20 simulated futures. Compare this with your reserves: AHV and pension fund continue, and wealth is only part of the financing. With two or more sources that give different results, a lower limit (2 % to 3 %) is a reasonable precaution.

## @o-consumption Goal: consumption (median and P10)
Keywords: median, percentile, P10, weight, consumption, spending, real

What is optimized is **annual consumption from retirement**: total spending including housing costs, **before taxes**, in **prices of the start month** (i.e. real). The app collects the yearly values of **all paths and all years** and forms two key figures from them:

- **Median:** the typical annual consumption. Half of all path-years are above it, half below.
- **P10 (10th percentile):** the annual consumption that 10 % of the path-years fall below. This is the **poor case** and measures how hard things get in crises.

**Weight of the median in the consumption goal** (default 50 %): How the consumption goal is mixed from the two key figures. The rest goes to P10.

> Consumption goal = weight × median + (1 − weight) × P10

- **100 %:** Only typical consumption counts. The optimization chooses more aggressive settings with a high median and poor crisis years.
- **0 %:** Only the poor case counts. This leads to cautious settings.
- **50 %** is a balanced starting point. Increase the weight of P10 if **reliability** matters more to you than the average.

From ruin on, only what income and pensions cover counts as consumption, no longer the planned spending. A ruin therefore depresses the median and above all P10.

## @o-vol Goal: volatility
Keywords: volatility, fluctuation, year to year, penalty, cut, stable

**Volatility** measures how strongly annual consumption changes from one year to the next: the **mean absolute change** compared with the previous year, averaged over all paths. 10 % means that consumption rises or falls by 10 % per year on average. Fixed spending (Standard, 3 buckets) has a volatility close to 0 %, strategies that follow wealth (fixed rate, VPW) a high one.

Two settings control how volatility enters the goal:

- **Penalty for volatility** (default 1): The consumption goal is multiplied by the factor (1 − penalty × volatility). With a penalty of 1 and 10 % volatility, the goal drops by 10 %. **0 means that volatility does not matter**; then the strategies with the highest consumption win, even if it fluctuates strongly.
- **Maximum volatility** (empty = no limit): A **hard upper limit** as a constraint, as with the ruin limit. Example 10 %: strategies whose mean annual change is higher are excluded. This is the setting for ‘I want my spending to fluctuate by no more than 10 % per year on average’.

## @o-score The goal in one formula
Keywords: formula, goal value, rating, fitness, score

> **Goal** = (weight × median + (1 − weight) × P10) × (1 − penalty × volatility)

The goal is shown in francs per year (real prices). **A candidate is admissible** if the probability of ruin is below the limit and (if set) volatility stays below the maximum. Among the admissible ones, the one with the highest goal wins. If two have the same goal (for example with fixed spending, where all strategies have the same consumption), the one with the **lower probability of ruin** wins. Candidates that are not admissible are ranked by how strongly they violate the limit.

With fixed spending (Standard, 3 buckets), consumption is the same in all runs. There, the optimization therefore decides **about security**, not about consumption. With strategies that follow wealth, consumption results from the strategy.

## @o-strategies Strategies and their parameters
Keywords: parameters, search range, Standard, 3 buckets, guardrails, VPW, endowment, fixed rate, stock allocation

Select the strategies to be compared. It is advisable to calculate **all** of them to see how the approaches differ. For every strategy, the app searches for these parameters in these **default ranges**. Which parameters are varied and in which range you decide yourself, see [Fixed or varied parameters](#o-ranges).

| Strategy | Parameters | Search range |
|---|---|---|
| Standard | stock allocation, reserve (months) | 10 % to 100 %, 3 to 36 |
| 3 buckets | bucket 1: cash (years), bucket 2: bonds (years), refill from a previous-year return of | 0.5 to 4, 0 to 10, −10 % to +20 % |
| Guardrails | initial rate, upper and lower guardrail, adjustment, stock allocation | 2 % to 8 %, 5 % to 50 % each, 2 % to 30 %, 10 % to 100 % |
| Fixed rate | withdrawal rate, stock allocation | 2 % to 8 %, 10 % to 100 % |
| VPW | planning age, real return, stock allocation | 85 to 105, −1 % to 5 %, 10 % to 100 % |
| Endowment | target rate, smoothing, stock allocation | 2 % to 8 %, 5 % to 100 %, 10 % to 100 % |

**Important:**

- The **stock allocation** is changed **from retirement**. Earlier rows of a time-dependent value remain.
- The **limits on living expenses** (floor, ceiling) stay fixed at the value of your plan by default. With guardrails and fixed rate, you can have them varied as well. A floor can restrict the optimization considerably because it prevents cuts.
- The **3-bucket strategy** sets the stock allocation through the buckets, so it is not searched there.
- A parameter at the edge of the range is a sign that the range is limiting: the optimum is at the edge, not beyond it.

The parameters are described with the strategies: [Standard](#f-wd-standard), [3 buckets](#f-wd-three_buckets), [Guardrails](#f-wd-guardrails), [Fixed rate](#f-wd-fixed_rate), [VPW](#f-wd-vpw), [Endowment](#f-wd-endowment).

## @o-ranges Fixed or varied parameters
Keywords: range, fixed, vary, from, to, parameters, search range, default ranges, bucket

Under “Strategy parameters”, there is a table for every strategy with all its parameters:

- **In the plan:** the value your plan has today (or the default value of the strategy if you have never used it).
- **vary:** With a tick, the optimization searches this parameter within “from” to “to”. **Without a tick, it stays fixed** at the value of the plan.
- **from / to:** The range in the **unit of the parameter**: years in years, months in months, percent in percent, age in years.

**Example 3 buckets:** bucket 1 fixed (no tick), bucket 2 from 0 to 2 years, refill from a previous-year return of −30 % to +10 %. The search then only changes bucket 2 and the refill threshold; bucket 1 stays as it is in the plan.

**Good to know:**

- A **narrow range** makes the search faster and the result closer to your plan. A **wide** one is more likely to find surprises, but needs more calculation time.
- If **all** parameters of a strategy are fixed, only this one plan is calculated. This is useful for comparing strategies with fixed values fairly. This is quicker with [Compare strategies (without optimization)](#o-compare), without changing the check marks.
- “from” must not be greater than “to”. There are outer limits for every parameter (e.g. stock allocation 0 % to 100 %); outside them, the app reports an error.
- **Floor and ceiling of living expenses** (guardrails, fixed rate) are fixed at first. If you vary them, you often get results that only look ‘good’ because of the floor, since cuts are prevented.
- “Default ranges” resets the strategy to the presets.
- The settings are **saved with the plan**.

## @o-compare Compare strategies (without optimization)
Keywords: compare, without search, values of the plan, end wealth, never sell properties, both

The button **“Compare strategies (without optimization)”** next to “Start optimization” computes the chosen strategies **with the parameter values of your plan**, without searching for better values. For a strategy your plan has never used, its default values apply (column “in the plan” under “Strategy parameters”).

- The **“vary”** check marks and the ranges stay unchanged; the comparison ignores them. Afterwards you can optimize as usual.
- It computes on **the same paths as the simulation** (number and random seed of the return source). The figures of a strategy therefore match a simulation in which you have set this strategy. The paths for search and checking under “Advanced” only apply to the optimization.
- The table is the same as after an optimization (see [Reading the result](#o-read)); the parameter column shows the values of the plan.

**Real estate:** If your plan has an owned or planned property, the selection “Real estate” determines what is computed, for the comparison as well as for the optimization:

- **as planned:** with your plan as it is;
- **Never sell properties:** with the switch [Never sell properties](#f-re-never-sell) on;
- **both:** every strategy twice, labelled “as planned” and “never sell properties”.

Without a property of your own the selection is hidden. The selection is saved with the plan.

## @o-advanced Advanced settings
Keywords: paths, checking, start points, accuracy, calculation time

- **Paths for the search** (default 5 000): The search evaluates this many random paths for every candidate. **More paths** make the result more accurate, but slower. Rule of thumb: below 2 000 paths the evaluation becomes unsteady, above 20 000 the effort is rarely worth it.
- **Paths for checking** (default 10 000): This many **other** paths recalculate the parameters found at the end. You see these values in the result.
- **Random start points per strategy** (default 30): The search starts at the plan, at the middle of the search range and at this many random points. More start points help to avoid getting stuck in a poor local optimum, but cost time, especially for strategies with many parameters (guardrails).

The paths come from the return source of the plan (bootstrap or parametric). The paths for checking have a **different random seed**. With “All historical start dates”, there are no random paths. Search and checking then calculate the same historical periods, so **the checking is not independent there**.

## @o-algorithm How the search works
Keywords: algorithm, random search, pattern search, safety margin, checking, independent, overfitting

1. **Common paths:** All candidates are evaluated with **the same** return paths. Differences thus come from the parameters, not from chance.
2. **Random search:** The app evaluates start points (the current plan, the middle of the range and random points) and takes the best one.
3. **Pattern search (coordinate search):** From the best point, every parameter is increased and decreased individually. If a change brings an improvement, it is adopted. If none is found, the step size is halved until it is very small.
4. **Safety margin:** Otherwise, the search would exploit the ruin limit to the full and end up just above it on new paths (random error of the estimate). It therefore searches with a limit that is **stricter by 1.5 standard errors**. With a 5 % limit and 5 000 paths, that is about 0.5 percentage points.
5. **Independent checking:** The result is recalculated on **other** paths (different random seed, more paths). Only this check decides whether a strategy is admissible.

The check protects against **overfitting**: a strategy that only looks good on the search paths fails the check.

## @o-read Reading the result
Keywords: table, baseline, star, grayed out, ruin, median, P10, volatility, goal, bar, checking

The result table shows the key figures for every strategy, **calculated on the independent paths for checking**. Consumption is in francs per year, in prices of the start month, including housing costs and before taxes.

| Column | Meaning |
|---|---|
| Strategy | name; ★ marks the best admissible one |
| Ruin | share of paths in which wealth is depleted (until the chosen age) |
| Median | typical annual consumption |
| P10 | annual consumption in the poor case |
| Volatility | mean change of annual consumption compared with the previous year |
| Goal | the combined value according to the [formula](#o-score), with a bar for comparison |
| Withdrawal rate | withdrawal from wealth in the **first year of retirement** (living expenses and housing costs minus income, **before taxes**), as a percentage of liquid wealth at the beginning; median across the paths |
| End wealth 1 % … 99 % | nominal wealth at the end of the simulation at the percentiles 1, 5, 50, 95 and 99 % of the paths, determined as in the simulation result; also for the starting point |
| Parameters | the settings found (in the comparison without optimization the values of the plan); button for applying them |

- The first row **“Baseline”** shows your plan unchanged, on the same paths for checking. It is the measure of what the optimization gains.
- **Grayed-out rows** (“Ruin limit not met”) are strategies that miss your ruin limit. They stay visible so that you can see the cost of security. If **no** strategy is admissible, the limit cannot be reached with the available means; you then have to reduce spending, retire later or relax the limit.
- The **withdrawal rate** is a value for comparison, not a goal of the search. It shows how much a strategy draws on wealth at the beginning: a high rate with equally high consumption means that the strategy consumes more of the wealth and relies less on pensions and income. The rule of thumb ‘below 4 %’ only applies roughly (see [Withdrawal from wealth](#r-draw)).
- If consumption drops sharply compared with the baseline, security costs accordingly. That is **not an error** but the answer.
- Below the table are the numbers of paths for search and checking and the calculation time.

If the plan was changed after the optimization, a notice appears: the result then no longer fits the plan and has to be calculated again.

## @o-apply Applying to the plan
Keywords: apply, parameters, set strategy, adopt

“Apply to plan” sets in your plan:

- the **strategy** and its parameters (for example initial rate and guardrails for Guardrails),
- the **stock allocation** and the **cash reserve** from retirement.
- the switch **“Never sell properties”** if the row was computed with it.

The plan then counts as changed (unsaved). With **Undo** (⌘Z), you can take back the change. The result of the optimization stays visible and remains valid as long as you do not change the plan otherwise.

Afterwards, check the result with a **normal simulation** and, if possible, with a **different return source**. If the optimum is still good there, it is robust.

## @o-pitfalls Pitfalls of the optimization
Keywords: over-optimization, robustness, model risk, cross-check, warning

An optimization finds the best result **for the assumptions**. It says nothing about whether the assumptions are right.

- **The optimum only applies to these returns.** If you optimized with the Shiller series, the result need not hold with another series or with lower returns. **Cross-check the result with a second source**: optimize with bootstrap and check with the parametric model, or vice versa.
- **Precise numbers suggest a false precision.** A stock allocation of 63.4 % is not better than 60 %. Round to understandable values and check how much the goal changes as a result. Often the area around the optimum is **flat**.
- **Edge solutions:** If a parameter is at the edge of the search range (for example stock allocation 100 % or initial rate 8 %), this is a sign that the result is extreme. Distrust it.
- **A ruin limit that is too optimistic.** With 10 % or more, high consumption values are easily possible, but with a probability of 1 in 10 you lose your wealth.
- **Sequence risk and long periods.** The estimate of ruin with bootstrap and the parametric model depends strongly on the assumptions about crises.
- **Things not modeled:** investment costs, changes in the law, care costs, deaths. See [Limits of the model](#limits).
- **Fitting to the random paths:** Without independent checking, the optimization would react to random fluctuations. The app therefore checks on other paths. With “All historical start dates”, this independence is missing.

> A model calculation is not investment advice. An optimum is a **starting point for your considerations**, not an instruction.

## @o-example Example of interpretation
Keywords: example, interpretation, instructions, procedure

The numbers are invented and only serve to practice reading.

| Strategy | Ruin | Median | P10 | Volatility | Goal |
|---|---|---|---|---|---|
| Baseline (Standard) | 7.8 % | 72 000 | 72 000 | 0 % | 72 000 |
| Guardrails ★ | 3.9 % | 78 500 | 66 000 | 6 % | 68 000 |
| Fixed rate | 1.1 % | 84 000 | 51 000 | 21 % | 53 000 |

- The baseline does **not** meet the ruin limit of 5 % (7.8 %).
- Guardrails reaches **3.9 %** and delivers **more** consumption in the median. In return, consumption in the poor case drops to 66 000 and fluctuates by 6 % per year. That is the price of security.
- The fixed rate would be even safer, but fluctuates so strongly (21 %) that its goal is below Guardrails.

**What you make of it:** Check whether you could live with 66 000 in the poor case (housing, health, minimum living expenses). If so, Guardrails is a good candidate. Then recalculate it with a second return source.

## @o-cli Optimization on the command line
Keywords: terminal, optimize, command, options, script

The same calculation is available in the terminal:

```
vivaplan optimize <plan.vplan> [options]
```

| Option | Meaning |
|---|---|
| `--goal 1\|2` | goal: 1 consumption with limited risk of ruin (default), 2 no emergency sale of real estate |
| `--min-consumption <percent>` | goal 2: minimum share of the planned spending for the median of consumption (default 90) |
| `--max-ruin <percent>` | maximum probability of ruin |
| `--ruin-age <age>` | ruin only counts until this age |
| `--weight-median <0 to 1>` | weight of the median in the consumption goal |
| `--vol-weight <number>` | penalty for volatility |
| `--max-vol <percent>` | maximum volatility |
| `--paths`, `--check-paths`, `--samples` | paths of the search and checking, start points |
| `--seed` | random seed |
| `--strategy a,b` | strategies (`standard`, `three_buckets`, `guardrails`, `fixed_rate`, `vpw`, `endowment`) |
| `--json <file>` | save the result as JSON |
| `--compare` | compare the strategies without search with the values of the plan, on the paths of the simulation |
| `--property planned\|never-sell\|both` | real estate: as planned (default), never sell or both (every strategy twice) |

If settings are saved in the plan, they serve as the starting point. Every option given overrides them. The command always reads the **ranges of the parameters** (`optimization.ranges`) from the plan; there are no options for them in the terminal. The output shows the parameters and the key figures from search and checking for every strategy; it is in English (see [Command line](#ref-cli)). With `--compare` it shows the values of the plan and one line of key figures per strategy; the end wealth at 1, 5, 50, 95 and 99 % is always included.
