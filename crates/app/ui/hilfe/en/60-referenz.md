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
- **Property gains tax:** `taxes.propertyGain` with `brackets` (width, rate), `duration` (from full years, surcharge or reduction), `exemptBelow` and `reduction`; the purchase date of an owned property is in `bought`.
- The complete format is provided by `vivaplan schema` in the terminal.

The file contains **no results**. The app stores results in the [list of results](#r-list).

## @ref-cli Command line
Keywords: terminal, vivaplan, command, script, CLI

The engine is also available in the terminal (program `vivaplan`):

| Command | Purpose |
|---|---|
| `run <plan>` | Calculate a plan (`--csv`, `--json`, `--events`) |
| `validate <plan>` | Validate a plan |
| `optimize <plan>` | Optimize the strategy, see [Command line](#o-cli) |
| `bench <plan>` | Measure the calculation time (`--paths`) |
| `schema` | Output the JSON schema of the plan file |
| `library` | List the time series of the data library |
| `import-shiller <file>` | Import Shiller data again (.xls, .xlsx, .ods) |
| `import-snb <file>…` | Import SNB cubes (devkum, plkopr, zimoma, rendoblim) |
| `import-legacy <config.json>` | Take over the configuration of the earlier program as a starting point |
| `ui-dev` | User interface in the browser, local only (development) |

The command line outputs its texts, messages and events in English, regardless of the language of the user interface; numbers and dates are in the Swiss format.

## @ref-library Data library: folders and files
Keywords: folder, time series, storage location, Application Support, CSV

The desktop app keeps the library in the data folder of the app and creates missing bundled series there. The [investment strategy](#f-ret-series) uses `Shiller CHF.csv`, `Shiller CHF + CH-Obligationen.csv` and the SNB data in the subfolder `quellen` from it, so that updated Shiller and SNB data take effect; if a file is missing, the built-in one applies. The app no longer uses your own files in the folder. On macOS: `~/Library/Application Support/ch.vivaplan-app.desktop/Zeitreihen`. Plans refer to a series of the library with `library:Name.csv`, regardless of where the plan is saved.

The app does not add newly bundled series to an existing library on its own. Copy them into the folder if needed. The indices are described under [Investment strategy](#f-ret-series).

## @glossar Glossary
Keywords: terms, lexicon, meaning, abbreviations

**Affordability:** Bank rule by which the mortgage costs must not exceed a share of the income (Tragbarkeit). See [Affordability](#f-re-afford).

**AHV:** Old-age and survivors' insurance (Alters- und Hinterlassenenversicherung), pillar 1. The app only calculates the old-age pension. See [AHV](#f-ahv).

**Annuity:** Constant payments that use up a capital over a certain time. Basis of the [VPW](#f-wd-vpw).

**Bootstrap:** Method of randomly assembling new courses from real data. See [Bootstrap](#f-ret-bootstrap).

**Conversion rate:** Share of the pension capital that is paid out annually as a pension (Umwandlungssatz). See [Conversion rate](#f-pk-conversion).

**Fat tails:** Thick edges of the return distribution: extreme events are more frequent than with a normal distribution. See [parametric model](#f-ret-parametric).

**Guardrails:** Limits for the withdrawal. See [Guardrails](#f-wd-guardrails).

**Lump-sum withdrawal:** Payout of pension capital as a single amount instead of a pension, with lump-sum tax (Kapitalbezug). See [Share of lump sum](#f-pk-lumpsum).

**Median:** Middle value: half of the values are above it, half below.

**Monte Carlo:** Calculation of many random courses to estimate probabilities. See [Simulation](#s-returns).

**Nominal / real:** Nominal amounts are in future francs, real amounts in today's purchasing power. See [Return, inflation and real values](#ret-concepts).

**Pension fund (PK, BVG):** Occupational pension provision, pillar 2. See [Pension fund](#f-pk).

**Percentile:** The *n*th percentile is the value that *n* % of the values fall below. See [Final wealth by percentile](#r-percentiles).

**Pillar 3a:** Tied private pension provision (Säule 3a), one account per person. See [Pillar 3a](#s-pillar3a).

**Rebalancing:** Restoring the desired split between stocks and bonds. See [Stock allocation](#f-stockshare).

**Retirement savings:** The capital saved with the pension fund (Altersguthaben). See [Pension fund](#f-pk).

**Ruin:** The wealth (liquid and real estate) is used up. See [Optimization: goal](#o-goal).

**SARON:** Reference interest rate for mortgages in Switzerland. See [SARON](#ret-saron).

**Seed (random seed):** Start value of the random generator that makes results reproducible. See [Random seed](#ret-seed).

**Sequence risk:** The risk that bad stock market years come right at the beginning of the withdrawal phase. See [Pitfalls](#ret-pitfalls).

**Success rate:** Share of the runs in which the wealth lasts until the end. See [Overview](#r-overview).

**Tranche:** Part of a mortgage with its own term and its own interest rate. See [Mortgages](#f-re-mortgage).

**Volatility:** Measure of the fluctuation of returns, usually as a standard deviation per year. See [parametric model](#f-ret-parametric).

**VPW:** Variable percentage withdrawal. See [VPW](#f-wd-vpw).
