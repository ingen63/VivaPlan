# Changelog

English translation of `CHANGELOG.md` (German, authoritative) from version 1.0.0-beta.1 on.

## Version 1.0.0-beta.4

Fourth pre-release of 1.0, corrects the tax compared with 1.0.0-beta.3. The Windows installer `.msi` carries the version number 1.0.0.4; for now installers are built for macOS only (#202).

### May change results

- **Married with one person (#250):** «Married» also applies without a second person; taxes are then calculated with the tariffs and deductions for married couples. Older plans with one person are taken over as unmarried when opened and calculate as before (note); new plans start unmarried.

### New, changed, fixed

- **Input help from the assessment (#251):** New field «of which pillar 3a according to the assessment», so that pillar 3a is not deducted twice with «Deduct contributions to pillar 3a»; a note is shown while the field is empty.

## Version 1.0.0-beta.3

Third pre-release of 1.0. The Windows installer `.msi` carries the version number 1.0.0.3; for now installers are built for macOS only (#202).

### May change results

- **Taxes by canton and municipality (#63, #64, #65, #66):** In the mode «Place of residence», income, wealth and lump-sum tax follow canton and municipality, the property gains tax the canton of the property. Tariffs grow with inflation, the second-earner deduction applies to employed persons.
- **Deductions and residence (#131, #151, #152, #228–#232):** Income deductions as proposed by the canton or as own rows, additional and one-off deductions, wealth deductions, tax value per property, earned income as an annual amount, residence changing over time and the large pillar 3a for persons without a pension fund.
- **AHV (#190, #191, #198):** The 13th AHV pension is paid in December and counts in the net need of the 3-pot strategy. Early withdrawal and deferral reduce or increase the pension; the cap for married couples follows the law.
- **Real estate and strategies (#199, #204, #239):** Unaffordable renovations under «Never sell real estate» are counted; the limits on living expenses apply to VPW and Endowment also in the comparison and the optimization; guardrails without liquid assets adjust nothing.
- **Earned income (#236):** On request, earned income ends with the pension fund withdrawal.

### New, changed, fixed

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
