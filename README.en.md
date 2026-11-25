<p align="center"><a href="README.md">Deutsch</a> · <b>English</b> · <a href="README.fr.md">Français</a> · <a href="README.it.md">Italiano</a></p>

<h1 align="center">VivaPlan</h1>

<p align="center">
  <b>Will the money last – and how certain is that?</b><br>
  Retirement planning for Switzerland. Month by month, thousands of market histories, everything on your computer.
</p>

<p align="center">
  <a href="LICENSE"><img src="https://img.shields.io/badge/License-GPL%20v3-blue" alt="License GPL v3"></a>
  <img src="https://img.shields.io/badge/macOS%20%C2%B7%20Windows%20%C2%B7%20Linux-lightgrey" alt="macOS, Windows, Linux">
  <a href="https://github.com/ingen63/VivaPlan/actions/workflows/test-quality.yml"><img src="https://github.com/ingen63/VivaPlan/actions/workflows/test-quality.yml/badge.svg" alt="Test quality"></a>
</p>

<p align="center">
  <img src="docs/bilder/uebersicht-en.png" alt="VivaPlan: the plan of Hans and Anna Meier from the videos with timeline, inputs and live preview of the wealth over time" width="900">
</p>

https://github.com/user-attachments/assets/27a574d7-e356-415d-9bed-2230ae452f8d

<p align="center">
  ▶ <a href="https://archive.org/details/vivaplan-video-en/vivaplan-2min.mp4">VivaPlan in 2 minutes</a> ·
  ▶ <a href="https://archive.org/details/vivaplan-video-en/vivaplan-10min.mp4">VivaPlan in 10 minutes</a>
</p>

<p align="center"><sub>The screenshot and the videos show the same example plan with made-up data.</sub></p>

## What it is about

VivaPlan calculates your financial future from today until the end of the plan, with the Swiss particulars: old-age
pension (AHV), pension fund (BVG) with conversion rate and lump-sum withdrawal, pillar 3a, income tax, wealth tax,
lump-sum tax and property gains tax by canton and municipality, SARON mortgages and the banks' affordability
calculation.

Because nobody knows the stock market, VivaPlan does not calculate one path but thousands: every historical starting
month or Monte Carlo paths. The result is a clear number of how many cases the money lasts in, with the worst case
next to it.

And you do not have to type everything in yourself: **connect VivaPlan to an AI** such as Claude, ChatGPT or a local
model in LM Studio, and the AI asks for your figures in conversation, calculates with VivaPlan, compares variants and
explains the results ([more](#using-vivaplan-with-an-ai)).

## What VivaPlan can do

- 🤖 **Plan with an AI:** the AI opens, changes, computes and compares plans and explains the results in words; set up
  via *Help → Connect to AI …*, for Claude Desktop with one click ([more](#using-vivaplan-with-an-ai))
- 🧮 **Month by month:** income, pensions, taxes, real estate and mortgages in one calculation
- 📈 **120 years of market history:** World (11 countries, 1900–2020), Switzerland, S&P 500 since 1872, all in CHF
- 🎲 **Monte Carlo:** bootstrap from historical blocks or a stock model with price jumps; 50'000 paths in a few
  seconds, the same settings always give the same results
- 🪜 **Six withdrawal strategies:** Standard, 3-bucket (optionally with guardrails), fixed withdrawal rate, Guardrails
  (Guyton-Klinger), VPW, Endowment
- 🎯 **Optimization and comparison:** searches for the strategy with the highest and steadiest consumption at a limited
  probability of ruin and adopts it with one click; if you like, *Compare strategies (without optimization)*; results
  of the optimization can be saved and reopened
- 🧾 **Taxes by place of residence:** income, wealth and lump-sum tax by canton and municipality, property gains tax by
  the canton of the property; deductions from income (proposal of the canton or your own lines) and from wealth, tax
  value per property, a place of residence that can change over time. The taxes are an estimate.
- 🏠 **Real estate:** purchase, sale, moving from a house to an apartment, emergency sale, property gains tax;
  affordability by the rules of the banks, adjustable in the settings; switch “Never sell properties”
- 👫 **Two persons:** own pension fund and AHV each, rules for married couples for AHV and taxes; persons without a
  pension fund with a large pillar 3a
- 📐 **Time-dependent values and characteristic curves:** amounts and rates stay constant or run as steps, line,
  table or with a factor; characteristic curves make a value depend on the state of the plan
- 🔀 **Scenarios:** “What if I retire at 63 and take half as a lump sum?” as curves and heat maps
- 🕰️ **What would have happened:** “How would I have fared if I had retired in 1973?”
- 🗂️ **Compare simulations:** every simulation is saved together with the plan as it was then
- ❓ **Help on F1:** for the field you are working on, plus a [manual](crates/app/ui/hilfe/en/00-einstieg.md) with
  search and two introductory videos (*Help → Introductory videos …*)
- 🌗 **Light or dark:** setting “Appearance” with a light and a dark theme or like the system
- 🌐 **German, English, French and Italian:** user interface, menus, messages and help in all four languages
  (choose under *View → Language*); numbers always in Swiss format. The command line `vivaplan` comes with the
  installation package and prints English texts.

The detailed description is in [docs/BESCHREIBUNG.md](docs/BESCHREIBUNG.md) (German).

## Using VivaPlan with an AI

VivaPlan comes with an MCP server (`vivaplan mcp`, Model Context Protocol). Through it an AI works directly with the
calculation core of VivaPlan: it opens a plan, reads and changes values, simulates, compares variants, adopts the best
one and saves the result as a new file. It calculates exactly like the app and explains the figures in your language.
For example:

> “I am 58 and would like to stop working at 63. Will my money last until 95? And what changes if I take half of the
> pension fund as a lump sum?”

- **Starting with your own figures:** if you have no plan yet, the AI starts with an empty plan (like
  *File → New plan*), asks you for the missing details (income, expenses, wealth, pension fund, AHV and whatever else
  belongs to it) and does not invent values. The example plan only serves for trying out and demonstrating.
- **Supported AI programs:** Claude Desktop, Claude Code, the ChatGPT desktop app and LM Studio; other programs with MCP
  can be connected in the same way.
- **Setting up:** the menu *Help → Connect to AI …* shows for each AI program where the entry belongs and a ready-made
  line with the path of the program on your computer to copy. VivaPlan sets up nothing itself. For Claude Desktop on
  macOS there is the one-click package `VivaPlan_<version>_aarch64.mcpb` on the
  [release page](https://github.com/ingen63/VivaPlan/releases): double-click it and Claude Desktop installs VivaPlan
  as an extension.
- **Plan folder and “Read only”:** when setting up you choose a plan folder (the `.mcpb` asks for it during
  installation). The AI opens and saves plans only in this folder and its subfolders. With the switch “Read only” it
  can open, change and compute plans but not save them. Saving creates a new file anyway; the AI replaces an existing
  one only after asking you and with a backup copy.
- **What happens to your data:** what the AI reads from your plan becomes part of the conversation. With a cloud AI
  (Claude, ChatGPT) it goes to the provider, with a local model (for example in LM Studio) everything stays on your
  computer. The server replaces names with neutral ones (“Person 1”), answers name only the file name. VivaPlan itself
  sends nothing.

Everything else is in the help under “Connecting AI programs” ([reference](crates/app/ui/hilfe/en/60-referenz.md)).
The results are model calculations without guarantee, even when an AI explains them.

## Your data stays with you

No account, no cloud, no tracking: VivaPlan never accesses the network, neither the app nor the calculation core nor
the MCP server `vivaplan mcp`. Data leaves the computer only through an AI that you connect yourself. A plan is a
single file (`.vplan`) that you save, copy and archive like any other document.

## Installation

The packages of each version are on the [release page](https://github.com/ingen63/VivaPlan/releases):

| System | Package |
|---|---|
| macOS (Apple chip) | `.dmg` |
| Windows | `.msi` or `-setup.exe` |
| Linux | `.deb` or `.AppImage` (`.rpm` only for versions without “beta”) |
| Claude Desktop on macOS | `.mcpb`, the one-click package for the AI connection |

The packages are not signed. The operating system therefore warns on the first start:

- **macOS:** Drag the app into the “Applications” folder and open it. If a message says it cannot be opened, click
  “Open Anyway” under *System Settings → Privacy & Security*.
- **Windows:** In the blue window “Windows protected your PC” click “More info” and then “Run anyway”.
- **Linux:** Make the AppImage file executable (`chmod +x VivaPlan*.AppImage`) or install the .deb package.

Claude Desktop also warns when installing the `.mcpb` that the extension gets access to everything on the computer.
Claude Desktop shows this for every extension; VivaPlan itself enforces the limit to the plan folder.

## Getting started

1. Start VivaPlan: the welcome dialog offers the example plan, your own plan or an empty plan.
2. The checklist “Getting started” guides you through the eight steps of the quick start and ticks off what is done.
3. Start the simulation: the overview says in words how safe the plan is and suggests the next step.

Or let an AI guide you: set it up with *Help → Connect to AI …* and ask in the AI program whether your money will
last.

> **Note:** VivaPlan is a calculation tool and not investment, tax or pension advice. Past returns are no forecast.

## Data and sources

The returns come from the Jordà-Schularick-Taylor Macrohistory Database (CC BY-NC-SA 4.0), the data of
Robert J. Shiller, the Swiss National Bank and the Federal Statistical Office. They are not covered by the license of
the program. Terms and citations are in [QUELLEN.md](crates/engine/data/QUELLEN.md) (German). Because of the JST
license, VivaPlan remains free of charge.

## Development

Rust workspace with the calculation core (`crates/engine`), the command line (`crates/cli`) and the desktop app
(Tauri 2, `crates/app`, user interface in HTML/JS without external libraries). More than 600 Rust tests, user
interface tests with Playwright, coverage of at least 80 % per package, mutation tests and fuzzing.

- Structure, building, command line, project file and calculation rules: [docs/ENTWICKLUNG.md](docs/ENTWICKLUNG.md)
  (German)
- Test strategy: [docs/TESTS.md](docs/TESTS.md) (German)

```sh
cargo build --release
target/release/vivaplan run crates/engine/assets/example.vplan --events   # output in English (#176)
make test        # all Rust tests with nextest (not cargo test; reason in docs/TESTS.md)
```

Bugs and wishes are welcome as an [issue](https://github.com/ingen63/VivaPlan/issues).

## Support

VivaPlan is free of charge. If you find the project useful, you can support it voluntarily with a coffee:
[buymeacoffee.com/vivaplan](https://buymeacoffee.com/vivaplan).

## License

Program code: [GPL v3](LICENSE). The included time series are not covered by it; the rights to the underlying data
belong to the sources (Robert J. Shiller, SNB, Federal Statistical Office, Jordà-Schularick-Taylor), see
[QUELLEN.md](crates/engine/data/QUELLEN.md).
