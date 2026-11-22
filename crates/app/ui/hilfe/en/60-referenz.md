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
- **Optimization:** goal, strategies, paths and the **ranges of the parameters** per strategy are under `optimization` (`ranges`: strategy, parameter, then `fixed` or `lo`/`hi`, shares as a decimal number).
- **Property gains tax:** `taxes.propertyGain` with `brackets` (width, rate), `duration` (from full years, surcharge or reduction), `exemptBelow` and `reduction`; the purchase date of an owned property is in `bought`. In the mode “Place of residence” the rule set of the canton of the property applies (`canton` per property, without an entry the canton of residence); `taxes.propertyGainOwn: true` switches to the own rules.
- The complete format is provided by `vivaplan schema` in the terminal.

The file contains **no results**. The app stores results in the [list of results](#r-list).

## @ref-cli Command line
Keywords: terminal, vivaplan, command, script, CLI

The engine is also available in the terminal (program `vivaplan`):

| Command | Purpose |
|---|---|
| `run <plan>` | Calculate a plan (`--csv`, `--json`, `--events`) |
| `validate <plan>` | Validate a plan (`--json`) |
| `set <plan> <path>=<value>…` | Change values by field path, see [Changing values](#ref-set) |
| `diff <a> <b>` | List the differences between two plans (`--json`) |
| `optimize <plan>` | Optimize the strategy, see [Command line](#o-cli) |
| `bench <plan>` | Measure the calculation time (`--paths`, `--json`) |
| `schema` | Output the JSON schema of the plan file |
| `library` | List the time series of the data library |
| `import-shiller <file>` | Import Shiller data again (.xls, .xlsx, .ods) |
| `import-snb <file>…` | Import SNB cubes (devkum, plkopr, zimoma, rendoblim) |
| `import-legacy <config.json>` | Take over the configuration of the earlier program as a starting point |
| `ui-dev` | User interface in the browser, local only (development) |

The command line outputs its texts, messages and events in English, regardless of the language of the user interface; numbers and dates are in the Swiss format.

## @ref-json Output as JSON
Keywords: JSON, machine-readable, AI, script, standard output, format

With `--json` and no file name, `run`, `validate`, `optimize`, `bench` and `diff` print the result in a fixed format on the standard output, for scripts and AI assistants. Notes and errors go to the error output; on an error the command ends with an error code. `--json <file.json>` saves the detailed result in a file, as before.

Every output starts with `format` and the version. Amounts are rounded to whole units of the currency, shares to four decimals (0.9512 = 95.12 %), months are given as `"YYYY-MM"`. Messages have `code`, `params` and the English `text`; validation messages also have `severity`, `section` and `field` (field path).

| `format` | Content |
|---|---|
| `vivaplan-run/1` | `error` (or `null`), `issues`, `elapsedMs` and, depending on the method, `single`, `batch` and `scenarios` |
| `vivaplan-validate/1` | `valid` (no errors), `issues` |
| `vivaplan-optimize/1` | settings, `reference` (the plan as it is) and `outcomes` (best strategy first, with `params`, `validated`, `feasible` and `patch`) |
| `vivaplan-bench/1` | `runs`, `months`, `elapsedMs`, `usPerRun`, `cores`, `check` |
| `vivaplan-diff/1` | `changes`: per difference `path`, `old`, `new` |

**`single`** (one return path): key figures of the run (`endTotal`, `endTotalReal`, `totalAtRetirement`, `depleted`, `minLiquid`, `taxesTotal` …), `years` as a table with `columns` and one row per year (`year`, `age`, `liquid`, `propertyEquity`, `pensionFund`, `pillar3a`, `total`: value at the end of the year; `income`, `spending`, `taxes`: sum of the year) and `events` as pairs of month and text.

**`batch`** (several paths): `runs`, `successRate`, `earliestDepletion`, `percentiles` (final wealth at 0 = worst run, 1, 5, 10, 25, 50, 75, 90, 95 and 99 %) and `years` with `p5` to `p95` of the total wealth, the real median `p50Real` and the share of runs with wealth left, `survival`.

**`scenarios`** (variations): `params` and per combination `values`, `successRate`, `p5`, `p50`, `p95`, `p50Real`, `atRetirement`, `earliestDepletion`.

For the example plan, the output of `run --json` has fewer than 20 000 characters.

## @ref-set Changing values by field path
Keywords: set, diff, field path, change, variant, AI, compare

`vivaplan set <plan> <path>=<value> …` changes values of a plan without editing the file by hand. A **field path** names the keys of the plan file separated by dots, list entries by their number from 0: `wealth.stockShare`, `household.persons.0.name`, `milestones.1.years`. These are the same paths that the validation messages give under `field`.

- The **value** is read as JSON (`0.6`, `true`, `null`, `"text"`, `{…}`); anything else counts as text, for example `project.name=Variant B` or `returns.startDate=1990-01`.
- **Point in time** for time-dependent values: `path@<time>=<number>` changes the step from this point in time; if there is none, a new one begins there. A point in time is a milestone (`@retire`), an age of the first person (`@age:62.5`) or a month (`@2035-01`). Example: `wealth.stockShare@start=0.8 wealth.stockShare@retire=0.5` sets the stock allocation before and after retirement; a constant value becomes time-dependent.
- **Checking:** an unknown path, a wrong type or an unknown point in time gives a message, and **none** of the changes is applied. The other validation messages of the changed plan appear as notes; the file is written all the same.
- **File:** without `-o`, `set` writes a new file next to the old one, under the next free number (`plan-1.vplan`, `plan-2.vplan`, …), and prints its name. With `-o <file>` it writes there. `set` replaces an existing file only with `--overwrite`, after a backup `<file>.bak`. `--overwrite` without `-o` replaces the plan itself.

`vivaplan diff <a> <b>` lists per difference the field path, the old and the new value, for example `withdrawal.floor: 0 → 0.8`; with `--json` in the format `vivaplan-diff/1`.

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
