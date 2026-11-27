---
name: vivaplan-skill
description: Use the VivaPlan command line (`vivaplan`) to answer questions about a Swiss retirement plan (.vplan file) - whether wealth lasts to a given age, how much can be spent, pension or lump sum, stock share, withdrawal strategy. Use when the user mentions VivaPlan, a .vplan file, or asks about their retirement plan computed with VivaPlan.
license: See the VivaPlan license
compatibility: Needs the VivaPlan desktop app installed (it contains the program `vivaplan`) and shell access.
---

# VivaPlan

VivaPlan plans retirement in Switzerland: income, AHV, pension fund (PK), pillar 3a, wealth, real estate, mortgages,
taxes and a withdrawal strategy, month by month, on one or many return paths (Monte Carlo or historical). A plan is a
JSON file with the extension `.vplan`. The program `vivaplan` computes it; never compute results yourself.

## Rules

1. **Privacy first.** Everything you read from a plan or from `vivaplan` output (birth dates, income, wealth,
   pensions, property) is sent to the provider of this AI. Before the first call that reads a plan of the user, tell
   them so in one sentence and go on only with their consent. Prefer showing figures to repeating personal details.
2. **Answer in the language of the user**, even though `vivaplan` itself writes English. Write numbers in the Swiss
   format (1'234'567, 0.95 = 95 %).
3. **Only `vivaplan` computes.** Do not edit `.vplan` files by hand and do not estimate taxes, returns or pensions
   yourself. Change values with `vivaplan set`, then run.
4. **Never overwrite the user's plan** unless they ask. `vivaplan set` writes a new file by default; keep it that way.
5. **Say what the numbers are**: nominal or real, which method (one path or many), how many paths, and the limits
   below. End every answer with results with the disclaimer (see "Limits and disclaimer").

## Find the program

The program is not on the search path. Call it with its full path:

| System | Program |
|---|---|
| macOS | `/Applications/VivaPlan.app/Contents/MacOS/vivaplan` |
| Windows (installer `.exe`) | `%LOCALAPPDATA%\VivaPlan\vivaplan.exe` |
| Windows (`.msi`) | `C:\Program Files\VivaPlan\vivaplan.exe` |
| Linux (`.deb`, `.rpm`) | `/usr/bin/vivaplan` |

Check it with `vivaplan --version`; `vivaplan --help` lists all commands. Below, `vivaplan` stands for the full path.
It uses the data folder of the app (time series, limits from the settings) like the app does; `--data-dir <folder>`
takes another one. If the user does not name a plan file, ask for it; plans are wherever the user saved them. Without a plan file the
user creates one in the app (File → New) or, with the MCP server, you start an empty one (see below); never compute
the user's situation with the example plan.

## MCP server (alternative)

If the AI program connects to VivaPlan as an MCP server (`vivaplan mcp`, set up by the user), use its tools instead
of the command line: `plan_open` (path of a `.vplan`; `new: true` for an empty plan, `example: true` for the example
plan), `plan_get`, `plan_validate`, `plan_schema`, `describe`,
`library_list`, `result_list`, `result_get`, and to compute `simulation_run` (the plan as it is, like `run`) and
`evaluate` (an experiment, like `vivaplan evaluate`; the way to optimize and compare). From 60 s of estimated duration
the answer is `needsConfirmation` with the estimate: tell the user and call again with `"confirm": true` only after
they agree. Long computations report progress and can be cancelled; one computation runs at a time. To change the
plan use `plan_set` (list of `{path, value}`, one step, all or nothing), `plan_changes`, `plan_undo` and `plan_apply`
(take over a variant of an `evaluate` result by rank); they change only a working copy in memory. `plan_save` writes
a new file `<name> (AI).vplan` next to the plan; replace a file (`overwrite: true`) only when the user asks for it, the
server then asks the user itself. The server replaces names with neutral ones (Person 1) and shows only file names. If the user released a plan folder
(`--allow-dir`), the server opens and saves plans only inside it; a plan elsewhere is refused: ask the user to move it
into the plan folder or to choose another plan folder when setting up VivaPlan.
If the user wants to compute with their own figures and has no plan file, open an empty plan with
`plan_open` and `"new": true` (as «New» in the app). Its answer lists the missing inputs (`missing`) and the assumed
values to confirm (`assumed`): ask the user for them, never invent values and never take them from the example plan;
enter the answers with `plan_set` and save with `plan_save` and a file name such as `My plan.vplan`: it lands in the
plan folder (`~/` is the home folder). The example plan (`"example": true`) only serves to try out or demonstrate
VivaPlan.
The rules above apply in the same way.

## Workflow

1. `vivaplan validate <plan.vplan> --json` - errors and warnings with the field path (`field`). Fix errors first.
2. `vivaplan describe` - the catalog: fields with type, unit and range, series, metrics, experiment, examples,
   performance, limits. Read one section with `vivaplan describe <section>`, for example `vivaplan describe fields`;
   `--plan <plan.vplan>` measures the computing time on that plan; `--json` gives the format `vivaplan-describe/1`.
   Read the catalog instead of guessing field names. `vivaplan schema` prints the full JSON schema of a plan.
   People name a choice with its label in the app («Monte Carlo · historisch»); `vivaplan describe choices` gives
   the value of the plan for every label (`bootstrap`).
3. `vivaplan run <plan.vplan> --json` - compute the plan as it is. Add `--metric '<json>'` for the exact quantity
   the question asks about.
4. For "what if": `vivaplan set` to a new file, then `run`; or one `vivaplan evaluate` experiment for many variants.
5. `vivaplan diff <a.vplan> <b.vplan> --json` - show the user what changed between two plans.

## Commands

### run

`vivaplan run <plan.vplan> --json [--metric <json>]...`

- `--json` alone prints the format `vivaplan-run/1` to the standard output: `error`, `issues`, `elapsedMs` and,
  depending on the method of the plan, `single` (one return path), `batch` (many paths) and `scenarios` (variations
  of the plan); with `--metric` also `metrics`. Notes and errors go to the error output.
- `single`: `endTotal`, `endTotalReal`, `totalAtRetirement`, `depleted`, `minLiquid`, `taxesTotal` ..., and `years`
  as a table (`columns`, one row per year: `year`, `age`, `liquid`, `total`, `income`, `spending`, `taxes` ...).
- `batch`: `runs`, `successRate` (share of paths where wealth lasts to the end), `earliestDepletion`,
  `percentiles` of the end wealth and `years` with `p5` to `p95`, `p50Real` and `survival`.
- `--metric` adds metrics built from blocks, on the same paths. It takes one metric or a list and may be repeated.
  Blocks: `quantity` (`total`, `liquid`, `consumption`, `depleted`, `atFloor`, `taxes` ...), `real` (true = prices
  of the start month), `window` (`from`/`to`: `start`, `retirement`, `end`, a milestone id or `YYYY-MM`;
  `fromAge`/`toAge`; `years`), `reduce` per path (`any`, `atEnd`, `annualSums`, `min`, `mean`, `firstWhen`, `share`
  ...) and `aggregate` over the paths (`quantile` with `q`, `mean`, `probability`, `count`, `conditionalMean`). Each
  result has `value` and, for probabilities and means, `stdError`. All choices: `vivaplan describe metrics`.
- Other options: `--csv <file>` monthly values, `--events` events in the report, `--json <file.json>` saves the
  detailed result to a file.

Example - "Does my wealth last until 95?":

```
vivaplan run plan.vplan --json --metric '{"name":"ruinBy95","quantity":"depleted","window":{"toAge":95},"reduce":"any","aggregate":{"kind":"probability"}}' --metric '{"name":"totalAt95","quantity":"total","real":true,"window":{"toAge":95},"reduce":"atEnd","aggregate":{"kind":"quantile","q":0.1}}'
```

`ruinBy95` is the share of paths in which wealth is used up before 95; `totalAt95` is the real total wealth at 95 that
90 % of the paths reach. With one fixed path (`returns.source` = `fixed`) the probability is 0 or 1: say that this is
one assumed course, not a risk, and offer a Monte Carlo run (next section). If the plan ends before 95, say so.

### set and field paths

`vivaplan set <plan.vplan> <path>[@<time>]=<value>... [-o <plan.vplan>] [--overwrite]`

- A field path names the keys with dots and list entries by number from 0: `wealth.stockShare`,
  `household.persons.0.name`, `pension.pk.lumpsum`. The same paths appear in `field` of the validation messages.
- The value is read as JSON (`0.6`, `true`, `"text"`); anything else counts as text.
- Time-dependent values (type `series`): `path@<time>=<value>` sets the step from that time. `<time>` is a milestone
  id (`@start`, `@retire` ...), `@age:62.5` (age of the first person) or `@2035-01`.
- An unknown path, a wrong type or an unknown time changes nothing and reports an error.
- Without `-o` it writes a new file next to the plan (`plan-1.vplan`, `plan-2.vplan` ...) and prints its name. Use
  `--overwrite` only when the user asks; it keeps a backup `.bak`.

Example - Monte Carlo with 2'000 paths instead of one fixed path:

```
vivaplan set plan.vplan returns.source=parametric returns.parametric.paths=2000
vivaplan run plan-1.vplan --json --metric '...'
```

### evaluate (experiments)

`vivaplan evaluate <plan.vplan> <experiment.json> [--json] [--yes]`

An experiment is one JSON object: `set` (changes for all variants), `vary` (values, ranges, coupled values, groups),
`metrics`, `constraints` (`{"metric":"ruin","op":"le","value":0.02}`), `objective` (`maximize`, `minimize`,
`weighted`, `score`; without one all variants are listed side by side), `search` (`grid` or `pattern`) and `budget`.
The best variants are confirmed on independent paths; report the confirmed values. Copy a fitting example from
`vivaplan describe examples` and adapt it; the structure is in `vivaplan describe experiment`.

`evaluate` prints its estimated duration. From an estimated 60 s on it computes only with `--yes`. Do not add `--yes`
on your own: tell the user the estimate and ask, or reduce paths and variants in `budget` and `vary`.

### optimize

`vivaplan optimize <plan.vplan> --json` searches the parameters of the withdrawal strategies (`--strategy all` or one
of `standard`, `three_buckets`, `guardrails`, `fixed_rate`, `vpw`, `endowment`) under the plan's goal. Useful options:
`--goal 1|2`, `--min-consumption`, `--max-ruin`, `--ruin-age`, `--paths`, `--check-paths`, `--seed`. `--compare`
compares the strategies with the plan's own values, without a search. The result `vivaplan-optimize/1` has
`reference` (the plan as it is) and `outcomes` (best first, each with a `patch` of field paths). It can take minutes.

### validate, diff, describe, schema

- `vivaplan validate <plan.vplan> --json` - `valid` and `issues` (`code`, `severity`, `section`, `field`, `text`).
- `vivaplan diff <a.vplan> <b.vplan> --json` - `changes` with `path`, `old`, `new`.
- `vivaplan describe [<section>] [--plan <plan.vplan>] [--json]` - catalog (see Workflow).
- `vivaplan schema` - JSON schema of the plan file.

## Fields and units

The catalog (`vivaplan describe fields`) gives each field its unit. The most important:

- **Shares and rates are fractions, not percent**: `wealth.stockShare` 0.6 = 60 %, `pension.pk.lumpsum` 0.5 = half
  as a lump sum, `withdrawal.fixedRate.rate` 0.04 = 4 % per year, `returns.parametric.mu` 0.07 = 7 % per year.
  Output shares too: `successRate` 0.9512 = 95.12 %.
- **Amounts** are in the plan's currency (CHF), rounded to whole francs in the output. `flows.spending`,
  `flows.income`, `pension.ahv.monthly` are per **month**; tax tables and many results are per year.
- **Ages** are decimal years (64.5 = 64 years 6 months), of the first person unless stated.
- **Months** are written `YYYY-MM`.
- Important paths: `flows.spending`, `flows.spendingFactor`, `wealth.cash`/`bonds`/`stocks`, `wealth.stockShare`,
  `pension.ahv.monthly`, `pension.pk.capital`, `pension.pk.lumpsum`, `pension.pk.conversion`, `milestones.*.years`,
  `withdrawal.strategy`, `withdrawal.floor`, `returns.source`, `returns.parametric.paths`, `returns.parametric.seed`.

## Nominal and real

Amounts are **nominal** (francs of each month) unless a value is marked real (`endTotalReal`, `p50Real`, a metric
with `"real": true`). Real values are in prices of the start month of the plan and are the right basis to compare
amounts decades apart. Always say which one you report; for "how much can I spend" use real values.

## Limits and disclaimer

- One fixed return path is an assumption, not a forecast. Probabilities need many paths (`parametric`, `bootstrap`,
  `historical`, `rolling`); they carry a standard error (`stdError`).
- Parametric returns are drawn independently each year; historical paths repeat the past.
- The best of many variants is overestimated on its search paths; trust the confirmed values.
- The model does not know what is not in the plan (care costs, inheritance, changes of law). Taxes are estimates from
  tables; AHV and pension fund follow the entered values.
- `vivaplan describe limits` lists the limits of the current version.

End each answer with results with this disclaimer in the language of the user: "Model calculation without
guarantee. No tax or investment advice. Taxes are estimates."
