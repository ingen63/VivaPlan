// Oberflächentests (Playwright). Startet den lokalen Entwicklungsserver und nutzt das installierte
// Google Chrome (kein Browser-Download). Es werden keine Daten ins Internet gesendet.
const { defineConfig } = require("@playwright/test");

const PORT = process.env.PORT || 8812;

module.exports = defineConfig({
  testDir: ".",
  timeout: 90_000,
  expect: { timeout: 30_000 },
  fullyParallel: true,
  workers: process.env.CI ? 2 : 4, // der GitHub-Runner hat 2 Kerne; mit 4 Workern fehlte dem Entwicklungsserver die Rechenzeit (Lauf 43746959803)
  reporter: [["list"], ["json", { outputFile: "results.json" }], ["html", { open: "never" }]],
  use: {
    baseURL: `http://127.0.0.1:${PORT}`,
    channel: "chrome",
    viewport: { width: 1440, height: 1000 },
    locale: "de-CH",
    acceptDownloads: true,
    trace: "retain-on-failure",
    screenshot: "only-on-failure",
  },
  projects: [
    { name: "main", testMatch: ["ui.spec.js", "optimum.spec.js"] },
    { name: "cancel", testMatch: "ui-cancel.spec.js", dependencies: ["main"], fullyParallel: false, workers: 1 },
  ],
  webServer: {
    command: `cargo build -q --release -p vivaplan-cli && target/release/vivaplan ui-dev --port ${PORT}`,
    cwd: "..",
    url: `http://127.0.0.1:${PORT}/`,
    reuseExistingServer: true,
    timeout: 300_000,
    env: { PATH: `${process.env.HOME}/.cargo/bin:${process.env.PATH}` },
  },
});
