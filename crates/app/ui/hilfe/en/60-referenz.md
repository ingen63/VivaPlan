# Reference

## @ref-shortcuts Keyboard shortcuts
Keywords: keyboard, shortcut, Ctrl, Command, F1

On the Mac, ⌘ (Command) applies; on Windows and Linux, **Ctrl**. ⇧ is the Shift key.

| Shortcut | Effect |
|---|---|
| **F1** | Help on the current topic (field, section or result chart) |
| ⌘⇧? | Manual (table of contents) |
| Esc | Close the help; otherwise back to the editor |
| ⌘N | New plan |
| ⌘O | Open |
| ⌘S | Save |
| ⇧⌘S | Save as |
| ⌘I | Import old `config.json` |
| ⌘↩ | Start simulation |
| ⌘Z | Undo |
| ⇧⌘Z | Redo |
| ⌘1 … ⌘9, ⌘0 | Section 1 to 10 in the editor, the tab in the results |
| ⌘↑ and ⌘↓ | previous or next section |
| ⌘/ | Overview of the keyboard shortcuts |

On some Mac keyboards, F1 is on the brightness key. Then press **fn+F1**, or use the button “? Help” or ⌘⇧?.

## @ref-checks Validation: errors and warnings
Keywords: validation, error, warning, note, check

On the right of the editor, the app checks the plan continuously.

- **Errors (red):** The plan is incomplete or contradictory. “Start simulation” is then not possible. A click on the error jumps to the section concerned. **“Start simulation” and “Start optimization” are blocked** as long as there are errors: an attempt shows “The plan has N errors …” and leads to the first section concerned. **Required fields** (for example name and date of birth of every person) carry an asterisk (*); an error appears directly at the field, and so does a value that cannot be read (for example an empty date). As long as you are typing in a field, **everything waits**: no message appears, and preview, computed values and validation list stay as they are. Only when you leave the field (Tab, click elsewhere or Enter) is everything recalculated and a message shown or removed. A half-typed value such as ‘6’ on the way to ‘65’ is therefore not reported. Selection lists and checkboxes take effect immediately. The complete list of required inputs is in the specification `docs/Pflichteingaben.md`.
- **Warnings (yellow):** The simulation runs, but a point deserves your attention, for example a time series in another currency or the combination of several lump-sum withdrawals.

Typical errors: start after the end, retirement outside the period, more than two persons, a missing time series for a historical source, an invalid parameter of a strategy. The message names the cause.

The line **Preview** on the right calculates one run with fixed returns. It shows where the inputs lead, but does not replace the simulation.

## @ref-file The plan file (.vplan)
Keywords: format, JSON, file, save, schema, project file

A plan is **a text file** in JSON format with the extension `.vplan`. It can be copied, archived and viewed with a text editor (“View file” in the editor shows the content).

- The format is marked with `"format": "vivaplan/1"`. Older plans with the extension `.rplan` and the identifier `pensionsplaner/1` can still be opened; saving always produces a `.vplan` file (the old file remains unchanged).
- **Points in time** are calendar months (`"2027-05"`) or references to milestones (`"retire"`); `"never"` means never.
- **Amounts** apply per month, **rates** per year as a decimal number (0.05 = 5 %).
- **Time-dependent values** are a number or `{"mode": "timed", "rows": [{"from": "start", "value": 9500}, {"from": "retire", "value": 8500}]}`.
- **Further forms** ([Time-dependent values](#f-timed)): straight line `{"mode": "linear", "start": {"age": 55, "value": 0.9}, "end": {"age": 65, "value": 0.5}}`, table `{"mode": "table", "interpolation": "step", "points": [{"at": "start", "value": 1}, {"age": 80, "value": 0.8}]}` (`interpolation` `linear` or `step`, `linear` if missing), factor `{"mode": "factor", "base": 6000, "factor": <course>}`. A support point has either `at` (date or milestone) or `age` (age, with `person`, otherwise of the first person).
- **Characteristic curve** (only `wealth.stockShare`, [Characteristic curve](#f-curve)): `{"mode": "curve", "input": "withdrawalRate", "points": [[0.02, 0.8], [0.06, 0.4]], "eval": "yearly"}`; `input` is `age`, `liquidReal`, `withdrawalRate`, `fundedRatio`, `stockReturn12m` or `drawdown`, `eval` `yearly` (default) or `monthly`.
- **Optimization:** goal, strategies, paths and the **ranges of the parameters** per strategy are under `optimization` (`ranges`: strategy, parameter, then `fixed` or `lo`/`hi`, shares as a decimal number).
- **Property gains tax:** `taxes.propertyGain` with `brackets` (width, rate), `duration` (from full years, surcharge or reduction), `exemptBelow` and `reduction`; the purchase date of an owned property is in `bought`. In the mode “Place of residence” the rule set of the canton of the property applies (`canton` per property, without an entry the canton of residence); `taxes.propertyGainOwn: true` switches to the own rules.
- **Place of residence:** `taxes.location` (`canton`, `municipality` as BFS number, `church`) applies from the start, with `taxes.mode: "location"` (new plans; without `mode` the own tables apply); further places of residence are in `taxes.relocations` as rows `{"from": "2031-01", "canton": "ZG", "municipality": 1711, "church": "none"}`. Tax multipliers by hand are in `multipliers` per place of residence (`canton`, `municipal`, `church` as a factor, 1.19 = 119 %); each one given replaces the value of the tax data file.
- **Taxable yield:** `taxes.investmentYield` with `stocks` and `bonds` as a rate per year (0.02 = 2 %); if missing, 0 % applies.
- **Default tax value:** `realestate.defaultTaxValue: true` (new plans); properties without `taxValue` then count at 50 % of the market value. If the field is missing, the market value applies.
- **Origin tax bill:** rows of `taxes.deductions` taken over from the tax bill ([Deductions from the tax bill](#f-tax-bill)) carry `taxBill` with the tax year, for example `{"from": "start", "kind": "own", "amount": 25000, "taxBill": 2025}`; it does not change the calculation.
- The complete format is provided by `vivaplan schema` in the terminal.

The file contains **no results**. The app stores results in the [list of results](#r-list).

## @ref-cli Command line
Keywords: terminal, vivaplan, command, script, CLI

The engine is also available in the terminal (program `vivaplan`):

| Command | Purpose |
|---|---|
| `run <plan>` | Calculate a plan (`--csv`, `--json`, `--events`, `--metric`, see [Custom metrics](#ref-metric)) |
| `validate <plan>` | Validate a plan (`--json`) |
| `set <plan> <path>=<value>…` | Change values by field path, see [Changing values](#ref-set) |
| `diff <a> <b>` | List the differences between two plans (`--json`) |
| `optimize <plan>` | Optimize the strategy, see [Command line](#o-cli) |
| `describe [<section>]` | Catalog for AIs: what can be changed, varied and measured (`--plan`, `--json`), see [Catalog for AIs](#ref-describe) |
| `mcp` | MCP server for AI programs, reads, computes, changes and saves (`--data-dir`, `--library`, `--names`, `--read-only`, `--allow-dir`), see [Connecting AI programs](#ref-mcp) |
| `evaluate <plan> <experiment.json>` | Compute an experiment (`--json`, `--yes`), see [Experiment](#ref-experiment) |
| `bench <plan>` | Measure the calculation time (`--paths`, `--json`) |
| `schema` | Output the JSON schema of the plan file |
| `library` | List the time series of the data library |
| `import-shiller <file>` | Import Shiller data again (.xls, .xlsx, .ods) |
| `import-snb <file>…` | Import SNB cubes (devkum, plkopr, zimoma, rendoblim) |
| `import-legacy <config.json>` | Take over the configuration of the earlier program as a starting point |
| `ui-dev` | User interface in the browser, local only (development) |

The command line outputs its texts, messages and events in English, regardless of the language of the user interface; numbers and dates are in the Swiss format.

**Program and data folder.** The program `vivaplan` is part of the installation package and lies next to the app. It is not added to the search path; call it with the full path:

| System | Path of the program |
|---|---|
| macOS | `/Applications/VivaPlan.app/Contents/MacOS/vivaplan` |
| Windows (installer `.exe`) | `%LOCALAPPDATA%\VivaPlan\vivaplan.exe` |
| Windows (`.msi`) | `C:\Program Files\VivaPlan\vivaplan.exe` |
| Linux (`.deb`, `.rpm`) | `/usr/bin/vivaplan` |

With the AppImage the program lies inside the image and cannot be called directly. `vivaplan --help` shows all commands. A [guide](#ref-skill) for AI assistants is included.

Like the app, the command line calculates with the app's data folder: with the time series in the subfolder `Zeitreihen`, including series imported in the app (`library:<file>`), and with the limits and the affordability from the [settings](#f-settings) (`einstellungen.json`). The data folder is `~/Library/Application Support/ch.vivaplan-app.desktop` on macOS, `%APPDATA%\ch.vivaplan-app.desktop` on Windows and `~/.local/share/ch.vivaplan-app.desktop` on Linux. `--data-dir <folder>` takes another data folder, `--library <folder>` another data library. If the data folder is missing, the command line calculates with the built-in series and the default limits and says so once.

## @ref-json Output as JSON
Keywords: JSON, machine-readable, AI, script, standard output, format

With `--json` and no file name, `run`, `validate`, `optimize`, `bench` and `diff` print the result in a fixed format on the standard output, for scripts and AI assistants. Notes and errors go to the error output; on an error the command ends with an error code. `--json <file.json>` saves the detailed result in a file, as before.

Every output starts with `format` and the version. Amounts are rounded to whole units of the currency, shares to four decimals (0.9512 = 95.12 %), months are given as `"YYYY-MM"`. Messages have `code`, `params` and the English `text`; validation messages also have `severity`, `section` and `field` (field path).

| `format` | Content |
|---|---|
| `vivaplan-run/1` | `error` (or `null`), `issues`, `elapsedMs` and, depending on the method, `single`, `batch` and `scenarios`; with `--metric` also `metrics` |
| `vivaplan-validate/1` | `valid` (no errors), `issues` |
| `vivaplan-optimize/1` | settings, `reference` (the plan as it is) and `outcomes` (best strategy first, with `params`, `validated`, `feasible` and `patch`) |
| `vivaplan-bench/1` | `runs`, `months`, `elapsedMs`, `usPerRun`, `cores`, `check` |
| `vivaplan-diff/1` | `changes`: per difference `path`, `old`, `new` |

**`single`** (one return path): key figures of the run (`endTotal`, `endTotalReal`, `totalAtRetirement`, `depleted`, `minLiquid`, `taxesTotal` …), `years` as a table with `columns` and one row per year (`year`, `age`, `liquid`, `propertyEquity`, `pensionFund`, `pillar3a`, `total`: value at the end of the year; `income`, `spending`, `taxes`: sum of the year) and `events` as pairs of month and text.

**`batch`** (several paths): `runs`, `successRate`, `earliestDepletion`, `percentiles` (final wealth `endTotal` and `endTotalReal` at 0 = worst run, 1, 5, 10, 25, 50, 75, 90, 95 and 99 %, runs ordered by the nominal final wealth), `percentilesReal` (the same with the runs ordered by the final wealth in today's purchasing power; the run of a percentile may differ) and `years` with `p5` to `p95` of the total wealth, the real median `p50Real` and the share of runs with wealth left, `survival`.

**`scenarios`** (variations): `params` and per combination `values`, `successRate`, `p5`, `p50`, `p95`, `p50Real`, `atRetirement`, `earliestDepletion`.

For the example plan, the output of `run --json` has fewer than 20 000 characters.

## @ref-set Changing values by field path
Keywords: set, diff, field path, change, variant, AI, compare

`vivaplan set <plan> <path>=<value> …` changes values of a plan without editing the file by hand. A **field path** names the keys of the plan file separated by dots, list entries by their number from 0: `wealth.stockShare`, `household.persons.0.name`, `milestones.1.years`. These are the same paths that the validation messages give under `field`.

- The **value** is read as JSON (`0.6`, `true`, `null`, `"text"`, `{…}`); anything else counts as text, for example `project.name=Variant B` or `returns.startDate=1990-01`.
- **Point in time** for time-dependent values: `path@<time>=<number>` changes the step from this point in time; if there is none, a new one begins there. A point in time is a milestone (`@retire`), an age of the first person (`@age:62.5`) or a month (`@2035-01`). Example: `wealth.stockShare@start=0.8 wealth.stockShare@retire=0.5` sets the stock allocation before and after retirement; a constant value becomes time-dependent. A straight line, a table, a factor or a characteristic curve takes no point in time; `set` then reports an error.
- **Checking:** an unknown path, a wrong type or an unknown point in time gives a message, and **none** of the changes is applied. The other validation messages of the changed plan appear as notes; the file is written all the same.
- **File:** without `-o`, `set` writes a new file next to the old one, under the next free number (`plan-1.vplan`, `plan-2.vplan`, …), and prints its name. With `-o <file>` it writes there. `set` replaces an existing file only with `--overwrite`, after a backup `<file>.bak`. `--overwrite` without `-o` replaces the plan itself.

`vivaplan diff <a> <b>` lists per difference the field path, the old and the new value, for example `withdrawal.floor: 0 → 0.8`; with `--json` in the format `vivaplan-diff/1`.

## @ref-metric Custom metrics (--metric)
Keywords: metric, measurement, quantile, median, probability, window, consumption, AI

`vivaplan run <plan> --metric '<json>'` also computes metrics that you compose from building blocks, on the same return paths as `run` (without variations). `--metric` can be given several times and takes one metric or a list of them. With `--json` the values are under `metrics`, otherwise at the end of the report. The metrics are collected during the simulation; no monthly values are stored for them.

| Building block | Key | Choices |
|---|---|---|
| Quantity | `quantity` | a field of the monthly values (`total`, `liquid`, `income`, `spending`, `taxes`, `withdrawal`, `housing`, `mortgage`, `pensionFund`, `age` …) or derived: `consumption` (spending; after depletion only what income covers), `plannedConsumption`, `depleted` (1 from depletion), `atFloor` (1 when the floor sets the spending), the counters `emergencySales`, `lumpSums`, `affordabilitySales`, `unaffordableRenewals` |
| Unit | `real` | `true`: in prices of the start month (amounts only); otherwise nominal |
| Window | `window` | `from`, `to` (inclusive): `start`, `retirement`, `end`, a milestone id or `YYYY-MM`; `fromAge`, `toAge`: age of the first person; `years`: only the first n years. Without it the whole duration |
| Reduction per path | `reduce` | `annualSums` (sum per 12 months, a yearly series), `sum`, `mean`, `min`, `max`, `atStart`, `atEnd`, `firstWhen` (age in the first month meeting the condition), `share` (share of months meeting the condition), `any`, `volatility` (mean change of the annual sums), `shareOfStartLiquid` (sum relative to the liquid wealth before the window) |
| Aggregation over paths | `aggregate` | `{"kind": "quantile", "q": 0.5}`, `mean`, `min`, `max`, `probability` and `count` (with `condition`), `conditionalMean` (with `lowest` or `highest` as a share, or `condition`) |

A condition reads, for example, `{"op": "lt", "value": 50000}` (`gt`, `ge`, `lt`, `le`); without one, “greater than 0” applies, that is “yes” for `depleted` and `atFloor`. `condition` next to `reduce` checks the months (`firstWhen`, `share`, `any`), `condition` in `aggregate` the values per path. With `annualSums` the aggregation takes all yearly values of all paths together, as the optimization does. `name` names the metric in the result.

Every entry in `metrics` has `name`, `value` (or `null`), `values` (number of values), the standard error `stdError` for probabilities and for means per path, the path `path` for quantile, minimum and maximum, and the definition `metric`. Errors in the metrics come together as one message with the list; nothing is computed then. `vivaplan schema` describes the plan, not the metrics.

Examples:

```
--metric '{"name": "Consumption years 1–15, median", "quantity": "consumption", "real": true, "window": {"from": "retirement", "years": 15}, "reduce": "annualSums", "aggregate": {"kind": "quantile", "q": 0.5}}'
--metric '{"name": "Ruin up to 85", "quantity": "depleted", "window": {"toAge": 85}, "reduce": "any", "aggregate": {"kind": "probability"}}'
--metric '{"name": "Share of months at the floor", "quantity": "atFloor", "window": {"from": "retirement"}, "reduce": "share", "aggregate": {"kind": "mean"}}'
```

Median and P10 of `consumption` (real, whole duration from `retirement`) are the same values as in `optimize --compare`.

## @ref-experiment Experiment (evaluate)
Keywords: experiment, evaluate, variation, grid, pattern search, constraint, objective, budget, AI

`vivaplan evaluate <plan> <experiment.json> [--json] [--yes]` asks the plan a question as one JSON structure and computes it in one call: changes, variations, [metrics](#ref-metric), constraints and objective. Everything is structured JSON without formulas in strings; the schema is under `definitions.Experiment` in `vivaplan schema`.

| Key | Content |
|---|---|
| `set` | Changes for all variants: field path → value, as with [`set`](#ref-set), also with a point in time (`"wealth.stockShare@retire": 0.4`) |
| `vary` | Variations, each combined with each: `{"path", "values": […]}` (list), `{"path", "range": [from, to, step]}` (range), `{"path", "follows", "offset"}` (coupled: value of the other path plus `offset`) or `{"group": [{"set", "vary"}, …]}` (group: the members are not combined with each other) |
| `metrics` | Metrics as with `--metric`, each with a `name` |
| `constraints` | Constraints `{"metric", "op", "value"}` with `op` = `le`, `lt`, `ge`, `gt` |
| `objective` | exactly one: `{"maximize": "<metric>"}`, `{"minimize": "<metric>"}`, `{"weighted": [{"metric", "weight"}, …]}` (maximize the weighted sum) or `{"score": {…}}` (objective of the [optimization](#o-cli) with `maxRuin`, `weightMedian`, `volWeight` …). Without an objective all variants are listed (at most 50) |
| `search` | `method`: `grid` (all variants; default) or `pattern` (pattern search within the ranges like the optimization, per combination of the other values; `samples` random starting points, default 30; `seed`) |
| `budget` | `paths` search paths (1'000), `margin` margin on the constraints in standard errors (2), `top` number of confirmed variants (5), `confirmPaths` confirmation paths (10'000) |
| `confirm` | `true`: compute even if it takes long |

**Flow.** First the whole experiment is checked: errors come as a list with the path in the experiment (for example `vary[0].values[1]`), the reason and the valid values, and nothing is computed. Then the duration is estimated; from 60 seconds `evaluate` computes only with `--yes` (or `"confirm": true`) and otherwise reports the estimate. The search uses few paths; a constraint must be met with the margin. The best `top` variants are confirmed on **independent** paths (other seed); the ranking orders by the confirmed values. All variants compute on the same paths, so differences between them are more precise than the single values. With fixed paths (historical, all start dates) there are no independent paths; a hint says so.

**Result** (with `--json` under `result`): `ranking` with `rank`, the values of the variant (`settings`), `search` and `confirm` (each `objective`, `feasible`, `figures` with `value` and `stdError`; with `score` also `optimization`), `hints` (for example at the edge of the grid: the best variant lies at the edge of a range), `paths` (number and seeds of both stages, `independent`), the complete `experiment` with all defaults and the `version` of the engine. `status` is `done`, `invalid`, `needsConfirmation`, `cancelled` or `failed`; `estimate` gives variants, runs and seconds. The output stays short (under 20'000 characters).

Example: 10 years to retirement, stock allocation 40, 60 or 80 % before and 30 or 50 % after, withdrawal with a fixed rate of 3, 4 or 5 % or endowment 3 or 4 %, the highest real consumption in the first 15 years of retirement with at most 2 % ruin – 30 variants:

```
{
  "vary": [
    { "path": "wealth.stockShare@start", "values": [0.4, 0.6, 0.8] },
    { "path": "wealth.stockShare@retire", "values": [0.3, 0.5] },
    { "group": [
      { "set": { "withdrawal.strategy": "fixed_rate" },
        "vary": [ { "path": "withdrawal.fixedRate.rate", "values": [0.03, 0.04, 0.05] } ] },
      { "set": { "withdrawal.strategy": "endowment" },
        "vary": [ { "path": "withdrawal.endowment.rate", "values": [0.03, 0.04] } ] }
    ] }
  ],
  "metrics": [
    { "name": "ruin", "quantity": "depleted", "reduce": "any", "aggregate": { "kind": "probability" } },
    { "name": "cons15", "quantity": "consumption", "real": true, "window": { "from": "retire", "years": 15 },
      "reduce": "annualSums", "aggregate": { "kind": "mean" } }
  ],
  "constraints": [ { "metric": "ruin", "op": "le", "value": 0.02 } ],
  "objective": { "maximize": "cons15" }
}
```

Variations, comparison of strategies and optimization can be expressed this way: variations are a grid without an objective; the comparison of strategies is a group of the strategies with `"objective": {"score": {}}` and `budget.paths` = paths of the plan; the optimization is a pattern search with `score` and `"margin": 1.5`.

## @ref-describe Catalog for AIs (describe)
Keywords: describe, catalog, AI, fields, unit, range, state variable, metric, computing time

`vivaplan describe [<section>] [--plan <plan>] [--json]` tells a program or an AI without the source code what it can change, vary and measure in a plan. The catalog is generated from the schema and the code and is in English like the rest of the command line. It stays under 20'000 characters; single sections with `describe <section>`:

| Section | Content |
|---|---|
| `fields` | Fields of the plan grouped by object, each with type, unit, range and (only in this section) meaning; `*` stands for the number in a list |
| `series` | Forms of a value over time (`const`, `timed`, `linear`, `table`, `factor`, `curve`) and the points in time (milestone, `YYYY-MM`, `age:<years>`, `path@<point in time>`) |
| `state` | State variables of a [characteristic curve](#f-curve) |
| `metrics` | Building blocks of a [metric](#ref-metric): quantities, window, reduction per path, aggregation over the paths, comparisons |
| `experiment` | Forms of the [experiment](#ref-experiment): variations, constraint, objective, search methods, defaults |
| `examples` | three example experiments (spending at most 5 % ruin, pension or lump sum, stock allocation and withdrawal rule) that run on the example plan |
| `performance` | Microseconds per run, measured at the call (about half a second) on the plan of `--plan`, otherwise on the example plan with parametric returns; it gives an estimate of the duration of an experiment |
| `limits` | Limits of the model: independent years in the parametric model, selection error with many variants, uncertainty, nominal and real, computing time |
| `choices` | only asked for on its own: every value of every choice field with its label in the app (in the language of the user interface) and its meaning, e.g. `bootstrap` = «Monte Carlo · historical» |

With `--json` the catalog comes in the fixed format `vivaplan-describe/1`. The full descriptions and the exact structure are shown by `vivaplan schema`.

## @ref-skill Guide for AI assistants (agent skill)
Keywords: skill, agent skill, SKILL.md, Claude, Claude Code, AI, assistant, set up

The installation package contains a guide for AI assistants that can run commands: the agent skill `vivaplan-skill` in the open format (folder with the file `SKILL.md`). It explains to the AI the [command line](#ref-cli), the important fields with their units (shares instead of percent), nominal and real, the limits of the results and the disclaimer. The guide is in English; the AI still answers in your language.

| System | Folder of the skill |
|---|---|
| macOS | `/Applications/VivaPlan.app/Contents/Resources/vivaplan-skill` |
| Windows (installer `.exe`) | `%LOCALAPPDATA%\VivaPlan\vivaplan-skill` |
| Windows (`.msi`) | `C:\Program Files\VivaPlan\vivaplan-skill` |
| Linux (`.deb`, `.rpm`) | `/usr/lib/VivaPlan/vivaplan-skill` |

**Set up in Claude Code and in Claude Desktop (section *Code*):** Both read personal skills from the folder `.claude/skills` in your user folder. Copy the folder `vivaplan-skill` there, on macOS for example in the Terminal:

```
mkdir -p ~/.claude/skills && cp -R /Applications/VivaPlan.app/Contents/Resources/vivaplan-skill ~/.claude/skills/
```

On Windows copy the folder to `%USERPROFILE%\.claude\skills`. Then start a new session. Ask for example: *Does my wealth last until 95? My plan is ~/Documents/plan.vplan*. Claude calls `vivaplan` for this and asks beforehand whether it may run commands. After a new version of VivaPlan copy the folder again.

**Privacy:** What the AI reads from your plan or from the results (birth dates, income, wealth, pensions) goes to the provider of the AI. The skill tells the AI to point this out to you before it reads the first time. VivaPlan itself sends nothing.

The results are model calculations without guarantee, no tax or investment advice; taxes are estimates.

## @ref-mcp Connecting AI programs (MCP server)
Keywords: MCP, Model Context Protocol, server, vivaplan mcp, LM Studio, Claude Desktop, AI, tool, privacy, save, undo, plan_set, plan_save, plan folder, read only, warning, allow-dir, new plan, own figures, incognito, memory

`vivaplan mcp` is an MCP server: AI programs that support the Model Context Protocol (for example Claude Desktop, Claude Code, ChatGPT Desktop or LM Studio) start it themselves and talk to it over the standard input and output. It opens no network connection. The server changes a working copy in memory; only `plan_save` writes to the disk, by fixed rules (see below).

| Tool | Purpose |
|---|---|
| `plan_open` | Open a plan (`.vplan`, `.rplan`), an empty plan (`new`) or the example plan; overview without values and number of check messages |
| `plan_get` | Read a section or a single field (field path) |
| `plan_schema` | Look up the structure of the project file |
| `plan_validate` | Check the plan as the app does |
| `describe` | [Catalog for AIs](#ref-describe) |
| `result_list`, `result_get` | Results of the session (summary, yearly values, events, bands, variations) |
| `library_list` | Time series of the [data library](#ref-library) |
| `simulation_run` | Simulate the plan as the app does (with its variations); key figures and the result in words |
| `evaluate` | [Experiment](#ref-experiment) on the plan: changes, variations, metrics, constraints, objective; optimization and comparison run through it |
| `plan_set` | Change values of the working copy by field path (as [`vivaplan set`](#ref-cli)); all changes of a call are one step, with old and new value and the check messages afterwards |
| `plan_changes` | All differences of the working copy against the file |
| `plan_undo` | Take back the last step, with `all` all of them |
| `plan_apply` | Take over a variant of an `evaluate` result (by rank) into the plan, as one step |
| `plan_save` | Save the working copy as `.vplan`; the only tool that writes to the disk |

Before every computation the server estimates its duration. From 60 seconds it computes only if the AI confirms explicitly (`"confirm": true`); it should ask you first. Long computations report their progress and can be cancelled in the AI program. Only one computation runs at a time.

**Starting with your own figures:** If you ask the AI to compute with your own figures and you have no plan file yet, it opens an empty plan with `plan_open` and `new`, the same as *File → New* in the app. The answer lists the inputs still missing (name or label of the person, income, spending, wealth, pension fund, AHV) and the assumed values (date of birth, retirement age, tax tariffs). The AI should ask you for them and not invent any values; it adds a partner, pillar 3a, properties and one-off expenses only if you have them. The new plan is saved with `plan_save` as a new file in the (first) plan folder; a file name such as `My plan.vplan` is enough. The example plan only serves to try out and demonstrate.

**Saving:** `plan_save` keeps these rules itself, whatever the AI does: (1) Without a path a **new file** is created next to the opened one, `<name> (AI).vplan` (with a number if needed); an existing file stays untouched. (2) The server replaces an existing file only with `overwrite` and only after the AI program has asked you and shown the changes; if it cannot ask, the server refuses. (3) A backup `<file>.bak` is made first; writing goes through a temporary file. (4) If the file was changed since it was opened, it is not replaced. (5) Old `.rplan` files are never replaced; plans are saved as `.vplan`. (6) The server writes only into the folder of the opened plan or into a folder released with `--allow-dir <folder>` (subfolders included). A path may start with `~/` for your home folder, as with `--allow-dir`. If the AI names a folder that does not exist or is not released, the server answers that a file name alone saves into the plan folder; it names no folders. With `--read-only` the server does not offer `plan_save` at all. Do not edit a plan in the app and in the AI at the same time: the app does not notice changes to the file.

**Plan folder and “Read only”:** When setting up you choose a plan folder; it reaches the server as `--allow-dir <folder>` (several folders: `--allow-dir` several times). The server then opens and saves plans only in this folder and its subfolders. It refuses paths with `..` and symbolic links leading out of the folder; for a plan outside it reports that the plan is outside the plan folder. The data folder of the app (time series, settings) stays readable, so that the server computes as the app does. Opening, computing, fetching results, changing and saving in the plan folder work as without a plan folder. With the switch “Read only” (`--read-only`, off by default) the AI can open, change and compute plans but not save them. Without `--allow-dir` the server reads every plan named and writes only into its folder.

**The warning of Claude Desktop:** When installing the package, Claude Desktop warns that the extension gets access to everything on the computer. Claude Desktop shows this for every extension, because the package format knows no permissions: the server runs with your user rights, in the other AI programs as well, only without a warning. VivaPlan itself enforces the limit to the plan folder, not the operating system. The server reads plans only in the plan folder, plus the data folder of the app and the data that comes with it; it writes only with `plan_save` into the plan folder and opens no network connection.

**Set up with one click:** The menu *Help → Connect to AI …* shows for Claude Desktop, Claude Code, the ChatGPT desktop app (through the configuration of Codex) and LM Studio where the entry belongs, and the matching line with the path of the program on your computer to copy. Every line contains the plan folder: suggested is the folder of the plan opened last, otherwise the documents folder; *Choose folder …* changes it. The switch “Read only” adds `--read-only`. VivaPlan sets up nothing itself. For Claude Desktop on macOS every version comes with the package `VivaPlan_<version>_aarch64.mcpb`: a double click installs VivaPlan as an extension in Claude Desktop; the package contains the program `vivaplan` itself. When installing, Claude Desktop asks for the plan folder (Plan folder, required) and the switch “Read only”. During the beta it also asks for the field “Beta token”: if the app on this computer is activated already, leave it empty; otherwise the token activates the server and the app ([Beta token](#beta)). Without an activation the server answers every call with the note that VivaPlan is not activated yet.

**Set up:** In the AI program, enter the program `vivaplan` with its full path (see [command line](#ref-cli)) with the arguments `mcp --allow-dir <plan folder>` (`~` stands for your home folder). In Claude Desktop and LM Studio the entry looks like this on macOS:

```
{
  "mcpServers": {
    "vivaplan": {
      "command": "/Applications/VivaPlan.app/Contents/MacOS/vivaplan",
      "args": ["mcp", "--allow-dir", "~/Documents"]
    }
  }
}
```

In Claude Code, `claude mcp add --scope user vivaplan -- /Applications/VivaPlan.app/Contents/MacOS/vivaplan mcp --allow-dir ~/Documents` is enough. Like the command line, the server computes with the data folder of the app (limits, time series); `--data-dir` and `--library` override it.

**Privacy:** Everything a tool returns becomes part of the conversation with your AI and goes to its provider; with a local model (for example in LM Studio) everything stays on the computer. The server only reads plans whose path is named explicitly, with a plan folder only those inside it, and searches no folders. It replaces names of persons and freely entered labels with neutral ones (Person 1, Property 1), except with `--names`; answers name only the file name, not the folder. No answer is longer than 20'000 characters. The texts of the server are in English; the AI still answers in your language. See also [Privacy and local data](#privacy).

**Incognito chats:** In an incognito chat of Claude VivaPlan is not available: the AI sees no VivaPlan tools there. The AI program decides this; VivaPlan cannot do anything about it. If you still want to share your plan data sparingly: (1) Use a normal chat and delete it afterwards; what the provider keeps after that is governed by its privacy policy. (2) Switch off the memory for these conversations in the privacy area of the settings of your AI program, so that it remembers nothing from them. (3) Use a local model in LM Studio; then the data does not leave the computer.

The results are model calculations without warranty, not tax or investment advice; taxes are estimates.

## @ref-library Data library: folders and files
Keywords: folder, time series, storage location, Application Support, CSV

The desktop app keeps the library in the data folder of the app and creates missing bundled series there. The [investment strategy](#f-ret-series) uses `Shiller CHF.csv`, `Shiller CHF + CH-Obligationen.csv` and the SNB data in the subfolder `quellen` from it, so that updated Shiller and SNB data take effect; if a file is missing, the built-in one applies. The app no longer uses your own files in the folder. On macOS: `~/Library/Application Support/ch.vivaplan-app.desktop/Zeitreihen`. Plans refer to a series of the library with `library:Name.csv`, regardless of where the plan is saved.

The app does not add newly bundled series to an existing library on its own. Copy them into the folder if needed. The indices are described under [Investment strategy](#f-ret-series).

## @glossar Glossary
Keywords: terms, lexicon, meaning, abbreviations

**AHV:** Old-age and survivors' insurance (Alters- und Hinterlassenenversicherung), pillar 1. The app only calculates the old-age pension. See [AHV](#f-ahv).

**Retirement savings:** The capital saved with the pension fund (Altersguthaben). See [Pension fund](#f-pk).

**Annuity:** Constant payments that use up a sum of capital over a certain period. Basis of the [VPW](#f-wd-vpw).

**Bootstrap:** Method of randomly assembling new courses from real data. See [Bootstrap](#f-ret-bootstrap).

**Success rate:** Share of the runs in which the wealth lasts until the end. See [Overview](#r-overview).

**Fat tails:** Thick tails of the return distribution: extreme events are more frequent than with a normal distribution. See [parametric model](#f-ret-parametric).

**Guardrails:** Bounds around the initial rate that cut or raise the withdrawal when it moves too far from it. See [Guardrails](#f-wd-guardrails).

**Lump-sum withdrawal:** Payout of pension capital as a single amount instead of a pension, with lump-sum tax (Kapitalbezug). See [Lump-sum share](#f-pk-lumpsum).

**Median:** Middle value: half of the values are above it, half below.

**Monte Carlo:** Calculation of many random courses to estimate probabilities. See [Simulation](#s-returns).

**Nominal / real:** Nominal amounts are in future francs, real amounts in today's purchasing power. See [Return, inflation and real values](#ret-concepts).

**Percentile:** The *n*th percentile is the value that *n* % of the values fall below. See [Final wealth by percentile](#r-percentiles).

**Pension fund (PK, BVG):** Occupational pension provision, pillar 2. See [Pension fund](#f-pk).

**Rebalancing:** Restoring the desired split between stocks and bonds. See [Stock allocation](#f-stockshare).

**Ruin:** The wealth (liquid and real estate) is used up. See [Optimization: goal](#o-goal).

**SARON:** Reference interest rate for mortgages in Switzerland. See [SARON](#ret-saron).

**Pillar 3a:** Tied private pension provision (Säule 3a), one account per person. See [Pillar 3a](#s-pillar3a).

**Seed (random seed):** Start value of the random generator that makes results reproducible. See [Random seed](#ret-seed).

**Sequence risk:** The risk that bad stock market years come right at the beginning of the withdrawal phase. See [Pitfalls](#ret-pitfalls).

**Affordability:** Bank rule by which the mortgage costs must not exceed a share of the income (Tragbarkeit). See [Affordability](#f-re-afford).

**Tranche:** Part of a mortgage with its own term and its own interest rate. See [Mortgages](#f-re-mortgage).

**Conversion rate:** Share of the pension capital that is paid out annually as a pension (Umwandlungssatz). See [Conversion rate](#f-pk-conversion).

**VPW:** Variable percentage withdrawal. See [VPW](#f-wd-vpw).

**Volatility:** Measure of the fluctuation of returns, usually as a standard deviation per year. See [parametric model](#f-ret-parametric).
