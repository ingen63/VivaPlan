# Changelog

English translation of `CHANGELOG.md` (German, authoritative) from version 1.0.0-beta.1 on.

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
