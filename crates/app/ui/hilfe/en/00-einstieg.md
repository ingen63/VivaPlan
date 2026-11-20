# Introduction

## @start What VivaPlan can do
Keywords: overview, purpose, retirement planning, pension planning, simulation

VivaPlan answers the question everyone faces before retirement: **Will the money last, and how certain is that?**

The app calculates your financial future month by month, from today until the end of the plan. It knows the Swiss specifics:

- pension fund (BVG, pillar 2) with conversion rate and lump-sum withdrawal, old-age pension (AHV, pillar 1) and pillar 3a
- income tax, wealth tax, lump-sum tax and property gains tax
- owner-occupied home with SARON or fixed mortgage, the banks' affordability calculation, rent
- a second person with their own pension provision (rules for married couples)

The future is uncertain, above all the stock market returns. That is why the app does not calculate just one course but, depending on the setting, **thousands** (Monte Carlo or all historical start dates) and shows in how many of them the money lasts. Where the returns come from is the most important topic of the app: see [Returns & data](#s-returns).

The structure:

1. **Editor:** You enter the plan in eleven sections (on the left). On the right, you immediately see a preview of the wealth over time.
2. **Start simulation:** The app calculates all runs and shows the [results](#r-overview).
3. **Optimization:** The app searches for the [withdrawal strategy](#s-optimize) that suits your goals.
4. **Results:** Every simulation is stored as a result in the [list](#r-list), so you can look at it and compare it later.

## @quickstart A first plan in ten minutes
Keywords: instructions, beginning, tutorial, steps, getting started

Start with the example plan that is loaded when the app opens, and adapt it step by step to your situation.

The checklist **“Getting started”** on the right above the validation leads through the same eight steps. A click on a step opens the section, ⓘ the help on it. A section counts as done as soon as you change a value in it or tick it off by hand; “Undo” does not remove the tick. “Start simulation” is done as soon as a simulation has run without being canceled. With ✕ you hide the list, and *Help → Getting started* shows it again. Once all steps are done, it collapses. The progress applies to you on this device, not to a single plan, and is not stored in the plan file.

1. **[Household & project](#s-general):** Enter name and month of birth.
2. **[Milestones](#s-milestones):** Set the pension fund payout, the AHV pension start, and the start and end of the simulation. The end should be so late that you would not expect to outlive it (for example age 95).
3. **[Pensions](#s-pension):** Enter the current pension fund balance, conversion rate, lump-sum withdrawal and AHV pension from your pension statements.
4. **[Income & spending](#s-flows):** Earned income and living expenses per month. The expenses need a realistic value; this is where the biggest lever is.
5. **[Wealth & investment](#s-wealth):** Stocks, bonds and cash at the simulation start, plus the target stock allocation.
6. **[Simulation](#s-returns):** For a first assessment, choose “Monte Carlo · historical”. You set the indices (for example “World (JST)”) in the [investment strategy](#f-ret-series) under “Wealth & investment”.
7. **[Taxes](#s-taxes):** Replace the rates of the example plan with those of your place of residence (tax calculator of your canton).
8. **Start simulation** (button at the top right or ⌘↩). To the left of the button is a rough estimate of the **duration**. During the calculation, a **progress bar** shows the status and the estimated remaining time; with **Cancel** you stop the calculation and return to the plan.

The validation at the bottom right shows errors and warnings even before you calculate. Errors prevent the simulation, warnings do not.

> Tip: Press **F1** in an input field, and exactly the help on this field appears.

## @bedienung Operation and keyboard shortcuts
Keywords: menu, keyboard, save, open, undo, F1, language

- **File:** New (an **empty plan with default values**: start in the current month, no project name, one person without a name, no amounts and no real estate; returns, tax tables and strategy parameters take the default values), Open, Save, Save as. A plan is a single file with the extension `.vplan`. It can be copied and archived.
- **Starting and quitting:** At startup, the desktop app opens the plan you last opened or saved; the file name is shown in the header. If there is none, or the file has disappeared or cannot be read, it opens the bundled **example plan** (with a notice naming the file, except at the very first start). The example plan counts as a new, unsaved plan: “Save” asks for a location, it is never overwritten. When you quit with unsaved changes (close button of the window, ⌘Q or Ctrl+Q, or File → Quit), the app asks whether you want to save, discard or cancel. “Cancel” leaves the app open; if you cancel the location dialog while saving, it also stays open. Only the file path is remembered, locally in the data folder of the app.
- **Saving before the simulation:** If the plan has unsaved changes, the desktop app asks before every simulation whether you want to save: “Save” (for a new plan with a location dialog), “Simulate without saving” or “Cancel”. The question can be turned off in the settings.
- **Welcome:** At the very first start, the app asks how you would like to begin: **Explore the example** (loads the example plan and opens the help [What VivaPlan can do](#start)), **Start your own plan** (loads the example plan as a template, opens “Household & project” and resets the checklist [Getting started](#quickstart)) or **Empty plan** (like File → New). Esc closes the dialog without a change. After that, it no longer appears on its own; *Help → Welcome …* opens it at any time, with a question if there are unsaved changes. If the app is started with a file (double click), it does not appear.
- **Undo / Redo:** ⌘Z and ⇧⌘Z (on Windows and Linux Ctrl+Z and Ctrl+Shift+Z) undo inputs.
- **Switching sections:** ⌘1 to ⌘9 and ⌘0 jump to the sections, ⌘↑ and ⌘↓ go one forward or back. In the results, the same keys switch the tab.
- **Help:** **F1** opens the help on the current topic (depending on where you are: field, section, result chart). The button “? Help” and the ⓘ signs next to fields do the same. ⌘⇧? opens the table of contents, Esc closes the help.
- **Timeline:** Above the editor, it shows all milestones. A click on it opens the milestones. Below the line are the **real estate events** of the preview (green triangles for purchase and sale, orange for emergency sales) with a short label such as ‘Purchase Apartment 09.2027’. With more than 15 events, only the symbols appear; the text is then in the tooltip.
- **Time-dependent values:** Many amounts can be entered as a constant or as a sequence of values, see [Time-dependent values](#f-timed).
- **Language:** Menu *View → Language* or the [settings](#f-settings). The change takes effect immediately; plan and view stay. The help is shown in the chosen language; where a chapter is not yet translated, it appears in German.

The complete list of keyboard shortcuts is in the menu *Help → Keyboard shortcuts* and in the [reference chapter](#ref-shortcuts).

## @privacy Privacy and local data
Keywords: internet, cloud, storage location, security, network, data protection

All data stays on your computer. The app never connects to the internet, not even for data, updates or analytics.

- **Plans** are where you save the `.vplan` file. The path of the plan last opened or saved is in the file `einstellungen.json` in the data folder of the app.
- **Time series** (the data library) are in the data folder of the app. On macOS this is `~/Library/Application Support/ch.vivaplan-app.desktop/Zeitreihen`.
- **The list of results** is in the local storage of the app (not in the plan file). Anyone who has your user account on this computer can read it. Delete entries you do not want to keep.

You download new data for the library (Shiller, SNB) yourself from their websites and import it with the buttons in the [investment strategy](#f-ret-series).

## @limits Limits of the model
Keywords: assumptions, not considered, advice, liability, death

VivaPlan is a **calculation tool**, not investment, tax or pension advice. A result only applies to the assumptions entered. Particularly important:

- **The future is not the past.** Historical returns are not a forecast. See [Pitfalls of the returns](#ret-pitfalls).
- **Taxes are simplified:** rate tables, linearly interpolated, times the tax multiplier. Deductions, detailed progression and cantonal particularities are missing.
- **Not calculated:** deaths and survivors' pensions, inheritances except as a one-off amount, care costs (unless you enter them as spending), changes in the law, investment costs and fees, interest on cash.
- **Real estate** is simplified: market value with inflation, mortgages in tranches, emergency sale by a simple rule.
- **Optimized strategies** only apply to the inputs they were calculated with, see [Pitfalls of the optimization](#o-pitfalls).

Always check important decisions with professionals. You will find the full [disclaimer](#disclaimer) in the next section.

## @disclaimer Disclaimer
Keywords: liability, warranty, disclaimer, advice, responsibility, law, notice

At the first start, you confirm a short version of this notice. If the text changes, the app asks again. All result pages carry the note “Model calculation without warranty. Not financial or tax advice.” at the bottom.

**1. Free offering.** VivaPlan is provided free of charge from Switzerland. There is no entitlement to availability, updates, bug fixes or support.

**2. No advice.** The app is a calculation aid for personal planning. It does not provide investment, financial, tax, pension or legal advice and makes no recommendations to buy or sell financial instruments or real estate.

**3. Model calculations without warranty.** All results are based on your inputs and on simplified assumptions about returns, costs, inflation, taxes and life expectancy. Actual developments may differ considerably. Tax calculations are approximations and do not reflect all cantonal and municipal rules, deductions and changes in the law. No warranty is given for the correctness, completeness and timeliness of the calculations.

**4. Own responsibility.** Decisions you make on the basis of the app are your sole responsibility. Before important financial decisions, for example on a lump-sum withdrawal, the sale of a house or the withdrawal strategy, you should consult a qualified professional.

**5. Liability.** Liability for damages of any kind, in particular for loss of assets, lost profit, loss of data or consequential damages arising from the use or non-use of the app, is excluded to the extent permitted by law. Liability for intent and gross negligence is excepted.

**6. Data.** The app stores your inputs locally on your device and does not transmit any data (see [Privacy and local data](#privacy)). Backing up this data is your responsibility.

**7. Applicable law.** Swiss law applies.

As of: 13 November 2026
