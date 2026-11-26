# Changelog

English translation of `CHANGELOG.md` (German, authoritative) from version 1.0.0-beta.1 on.

## Version 1.0.0-beta.6

Sixth pre-release of 1.0. The Windows installer `.msi` carries the version number 1.0.0.6; installers are built for macOS, Windows and Linux, by hand for all platforms before the tag (#262).

### May change results

- **Housing share of living costs (#264):** The deducted housing share covers rent, interest and ancillary costs, no longer amortization. Older plans are adjusted when opened (with a notice); their result stays the same.
- **Percentiles (#265):** Results open in today's purchasing power; percentiles are ordered by the chosen view (nominal or real).
- **Taxable income from assets (#149):** A rate per year for shares and bonds counts as taxable income. New plans start with 2 % and 1 %, existing ones with 0 %.

### New, changed, fixed

- **Beta token (#270):** App and AI package are activated once with a beta token.
- **Care home (#271):** Own building block with entry age, costs and sale of the home; notice when no housing costs remain after the sale of the last property.
- **Taxes (#272–#275):** New plans calculate taxes at the place of residence; the section first shows municipality and denomination, the other details as assumptions, details under «Expert». Deductions can be taken from the last tax bill, the tax value of real estate has a default in percent of the market value, a pension fund buy-in can be entered directly as a deduction.
- **Deductions from wealth (#249)** laid out like the deductions from income.
- **Affordability (#266):** Optionally with wealth drawdown and lower mortgage ratio from retirement.
- **AI connection (#263, #269):** The MCP server opens and saves plans only in the shared plan folder, optionally read-only; a new plan starts empty and the AI asks for the missing details.
- **README (#267, #277)** in four languages: the app is 100 % local, only the AI connection sends data to the chosen AI.
- **Display (#220):** Choice «Appearance» in the settings dialog with the theme's background.

### Tests and development

- Linux package contains `vivaplan` again (#261), memory use of large calculations reduced (#249), Clippy rule (#260), measurement of the tax influence for more households (#276).

## Version 1.0.0-beta.5

Fifth pre-release of 1.0. The Windows installer `.msi` carries the version number 1.0.0.5; installers are built again for macOS, Windows and Linux (#203).

### May change results

- **Without pension fund (#253):** The switch is now in the card «Pension fund» in the section Pensions. When set, the pension fund of this person counts as 0 (no assets, no contributions, no pension, no lump sum); the entered values stay saved, the fields are locked.
- **Optimization (#248):** For all course forms, the course before retirement is kept; the optimized value applies from retirement.

### New, changed, fixed

- **Using VivaPlan with an AI (#208–#214, #216, #219):** The MCP server `vivaplan mcp` opens, changes, calculates and compares plans for Claude, ChatGPT, LM Studio and other AIs, with progress, cancelling and confirmation of long calculations. Menu «Help» › «Connect to AI» shows the setup; for Claude Desktop there is a one-click package `.mcpb` (#213). Data only goes to the AI you connect yourself.
- **Course forms and characteristic curves (#217, #218, #243):** Values can follow a line, a table or a factor and be edited in the editor; characteristic curves make parameters depend on the state.
- **Command line in the installer (#206):** `vivaplan` uses the same data folder as the app.
- **Display (#244, #255, #257, #259):** Number fields and the choice «Tax value» are no longer cut off in any language.

### Tests and development

- User interface tests updated (#246, #247), time limit in the MCP test independent of the machine's speed (#258).

## Version 1.0.0-beta.4

Fourth pre-release of 1.0; it also contains all changes of the unpublished 1.0.0-beta.3. The Windows installer `.msi` carries the version number 1.0.0.4; for now installers are built for macOS only (#202).

### May change results

- **Married with one person (#250):** «Married» also applies without a second person; taxes are then calculated with the tariffs and deductions for married couples. Older plans with one person are taken over as unmarried when opened and calculate as before (note); new plans start unmarried.
- **Taxes by canton and municipality (#63, #64, #65, #66):** In the mode «Place of residence», income, wealth and lump-sum tax follow canton and municipality, the property gains tax the canton of the property. Tariffs grow with inflation, the second-earner deduction applies to employed persons.
- **Deductions and residence (#131, #151, #152, #228–#232):** Income deductions as proposed by the canton or as own rows, additional and one-off deductions, wealth deductions, tax value per property, earned income as an annual amount, residence changing over time and the large pillar 3a for persons without a pension fund.
- **AHV (#190, #191, #198):** The 13th AHV pension is paid in December and counts in the net need of the 3-pot strategy. Early withdrawal and deferral reduce or increase the pension; the cap for married couples follows the law.
- **Real estate and strategies (#199, #204, #239):** Unaffordable renovations under «Never sell real estate» are counted; the limits on living expenses apply to VPW and Endowment also in the comparison and the optimization; guardrails without liquid assets adjust nothing.
- **Earned income (#236):** On request, earned income ends with the pension fund withdrawal.

### New, changed, fixed

- **Taxable wealth in the card «Deductions from wealth» (#249):** A note with the estimated taxable wealth of the first tax year and the share of the properties, a warning when a tax value is missing; the details build it up per tax year as in the tax return and offer the field «Tax value» per property.
- **Input help from the assessment (#251):** New field «of which pillar 3a according to the assessment», so that pillar 3a is not deducted twice with «Deduct contributions to pillar 3a»; a note is shown while the field is empty.
- **Compare strategies (#196, #197, #200, #201, #205):** Strategies can be compared without optimization, optimization results saved and reopened; switch «Never sell real estate»; a hint when the lower limit determines spending; result table without horizontal scrolling.
- **Dark theme (#220, #235):** Setting «Appearance» with light and dark theme; better contrast in the menu.
- **Usability (#143, #227, #233, #234, #241):** Taxes marked as an estimate, recently opened plans in the «File» menu, affordability in the settings, readable engine messages, clearer card «Income deductions» with a text for the second-earner deduction.
- **Introductory videos (#240, #242):** «VivaPlan in 2 minutes» and «in 10 minutes» in four languages, from the «Help» menu, the welcome dialog and the help; they open the browser.
- **Support (#237):** Voluntary donation link «Buy me a coffee» in the «Help» menu, in the navigation and under «About VivaPlan»; it opens the browser.
- **Command line for AIs (#207, #215):** JSON output and changing values by field path; return paths are reused for several variants (faster).

### Tests and development

- Golden numbers compared in the push run (#195), change in property values recalculated independently (#194), requirement tracing of tests extended (#192).
- Robustness test of all engine commands (#221), property test findings as regression tests (#223), Rust tests on Windows (#225), skipped tests count as errors (#224), time limit per test (#226), rules for the test agent (#222).
- Release notes in four languages with a language-neutral title (#188); mutants-diff weekly (#202).

## Version 1.0.0-beta.2

Second pre-release of 1.0. The Windows installer `.msi` carries the version number 1.0.0.2.

### May change results

- **Lump-sum tax for unmarried couples (#150):** Lump-sum withdrawals of partners who are not married are taxed per person and no longer added up for the household. This may change the results of such plans.

### New, changed, fixed

- **Four languages (#170–#177):** User interface, menus, file dialogs, messages and help are available in German, English, French and Italian. On first start the system language applies, afterwards the choice under «View → Language». Numbers and dates keep the Swiss format in every language. Names from the optimization catalogue and the time series are translated as well.
- **Terms and translation check (#179–#182):** A glossary sets the official terms per language (e.g. AVS/LPP, caisse de pension, cassa pensione); an automatic check compares all translations with German. All texts fit in every language, also with the fonts on Linux and Windows.
- **Command line in English (#176):** `vivaplan` prints messages and reports in English. The numbers are unchanged.
- **New app icon (#185):** «Summit path – sunset glow», with its own version for small sizes and a matching favicon.
- **Example plan (#184):** The example plan opens in the language of the user interface.
- **Help (#183):** Contradictions about living expenses, combinations and 3a contributions removed.

### Tests and development

- Code internally in English (#165–#169), network freedom checked automatically (#157), shared tax test cases for user interface and engine (#155, #156, #158).
- Example files named in English and with fixed line endings (#186), tests for previously undetected mutants (#187).
- Release notes in four languages with automatic check (#188).

## Version 1.0.0-beta.1

First pre-release of 1.0. The Windows installer `.msi` carries the version number 1.0.0.1 because the MSI format has no pre-release such as «beta.1»; the other packages are named 1.0.0-beta.1.

### May change results

- **Return criterion of the 3-bucket strategy (#103):** Every return path carries the stock returns of the 12 months before the simulation start. The return criterion therefore covers a full 12 months even in the first 12 months. This may change the results of plans with the 3-bucket strategy in the first year.

### New, changed, fixed

- **Comparison of results (#115):** Comparing saved results offers the same options as a single result: nominal or in today's purchasing power, all charts and the yearly values, with a choice of run (percentile).
- **Reference line in the charts (#114):** The line at 0 (or at 100 in the performance chart) is clearly highlighted in all result charts.
- **Menu (#112):** A click in the menu on the left or in the checklist is no longer lost when a preview arrives at the same time.

### Tests and development

- User interface tests independent of machine load (#111), test quality overview with checked criteria (#113), memory limit for test processes in mutation runs (#116).
