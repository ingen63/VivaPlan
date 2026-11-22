# VivaPlan – Entwicklung

Technische Beschreibung für Entwicklerinnen und Entwickler: Aufbau des Repos, Bauen, Kommandozeile, Datenbibliothek,
Projektdatei und die Rechenregeln Schritt für Schritt. Was die App kann, steht im [README](../README.md) und in
[BESCHREIBUNG.md](BESCHREIBUNG.md).

## Aufbau

| Ordner | Inhalt |
|---|---|
| `crates/engine/` | Bibliothek: Datenmodell, Simulation, Zeitreihen, Steuern, Prüfung, Import alter Konfigurationen; Tests unter `tests/` |
| `crates/engine/data/` | Datenbibliothek: ausgelieferte Zeitreihen (`Shiller CHF.csv`, `Shiller USD.csv`, `Welt (JST).csv`) sowie in `crates/engine/data/quellen/` SNB-Auszüge und die JST-Tabelle mit Auszug (die Shiller-Originaldatei wird nicht mitgeliefert, siehe `QUELLEN.md`) |
| `crates/engine/assets/` | Beispielplanungen (`example.vplan`, Vorlage `new.vplan`) |
| `crates/cli/` | Kommandozeile `vivaplan` (Rechnen, Prüfen, Import, Vergleich, Entwicklungsserver) |
| `crates/app/` | Desktop-App (Tauri 2): Fenster, Dateidialoge, Anbindung an den Rechenkern |
| `crates/app/ui/` | Oberfläche: Editor, Live-Vorschau, Ergebnisansicht (HTML/JS ohne externe Bibliotheken), Hilfe in `hilfe/` |
| `e2e/` | Oberflächentests (Playwright) |
| `fuzz/` | Fuzzing (cargo-fuzz, eigener Workspace) |
| `tools/` | Skripte: Git-Hooks, Build und Installation, App-Icons `make-icons.sh`, Rauchtest `ui-smoke.sh`, Testqualität, Übersichtsseite, Issues |
| `docs/` | Beschreibung, Glossar der Fachbegriffe ([`docs/GLOSSAR.md`](GLOSSAR.md)), Teststrategie ([`docs/TESTS.md`](TESTS.md)), QA-Bericht, Spezifikationen (Pflichteingaben, Weltindex JST, MCP-Konzept) |
| `local/` | Nur lokal (von Git ausgeschlossen): Mutationsläufe, Berichte der Testqualität, eigene Planungen und alte Konfigurationen (`local/eigene/`), Notizen |

## Sprache im Code

Code, Kommentare, Bezeichner und Testnamen sind englisch; Oberflächentexte, Hilfe, Doku, Issues und Commits sind
deutsch (#165). Die Oberflächentexte stehen seit #170 in Texttabellen (siehe «Mehrsprachigkeit»); sie sprechen die
Nutzer mit «Sie» an. Die englischen Namen für Schweizer Fachbegriffe legt [`GLOSSAR.md`](GLOSSAR.md) fest; Schweizer
Kürzel (AHV, PK, 3a, SARON …) bleiben als Bezeichner erlaubt. Serialisierte Namen (Schlüssel der Projektdatei,
Ergebnisfelder, IPC-Befehle, `data-*`-Attribute, CSS-Klassen) bleiben unverändert, auch wenn sie deutsch sind.

Neuer und geänderter Code ist sofort englisch. Der bestehende Code ist je Paket umgestellt (#169, #166, #167, #168); die
Übergangsliste ist leer, die Prüfung läuft deshalb auch in `make test-quiet` und im Workflow (`--gate`).
Die Prüfung `make english` (`tools/check-english.sh`, Regeln in `tools/check_english.py`) meldet deutsche Kommentare
(auch Doku-Kommentare und Python-Docstrings) und deutsche Testnamen in `crates/`, `e2e/`, `fuzz/` und `tools/` mit Datei
und Zeile (Rust, JavaScript, Python, Shell):

- Deutsch ist ein Text mit Umlaut oder ß, mit einem Wort aus der Wortliste oder mit einer typisch deutschen Endung
  (`-ung`, `-keit`, `-iert` …). Die Erkennung ist eine Heuristik; fehlt ein Wort, wird es in der Liste ergänzt.
- Nicht geprüft: Zeichenketten (Oberflächentexte), andere Bezeichner als Testnamen, `crates/engine/tests/golden/`.
  In Kommentaren zählen Code in Backticks, Beschriftungen in «…», URLs und Wörter in Grossbuchstaben nicht.
- Fehlalarm: Zeile mit `check-english: allow` markieren (im Kommentar derselben Zeile).
- Übergang: `PENDING` in `tools/check_english.py` listet die noch nicht umgestellten Pfade; sie werden übersprungen.
  Jede Paket-Umstellung entfernt ihren Pfad. Den Stand eines Pakets zeigt `tools/check-english.sh --all crates/cli`.
  Ist die Liste leer, läuft die Prüfung in `make test-quiet` und im Workflow (Job `english`) mit (Option `--gate`);
  bis dahin laufen dort nur die Tests des Skripts.

## Mehrsprachigkeit (#62, #170)

Alle Texte der Oberfläche, der Tastenkürzel-Übersicht, der Grafiken, der Zeitleiste, der nativen Menüs und der
Dateidialoge stehen in Texttabellen je Sprache: `crates/app/ui/i18n/<sprache>.json`, flache Schlüssel nach Bereich
(`menu.file.open`, `results.overview` …), Platzhalter `{name}`, Werte dürfen einfaches HTML enthalten (`<b>`, `<br>`).
`crates/app/ui/i18n/languages.json` nennt die vorhandenen Sprachen, jede in ihrer eigenen Sprache: `de.json`,
`en.json` (#172), `fr.json` (#174) und `it.json` (#175); Begriffe nach den Abschnitten Englisch, Französisch und
Italienisch in `docs/GLOSSAR.md`. Neue Texte kommen sofort als Schlüssel in
alle Tabellen, nie als Zeichenkette in den Code; der englische Text darf vorläufig sein, der Vollständigkeitstest
verlangt ihn aber.

- **Oberfläche:** `i18n.js` lädt die Tabellen ohne Build-Schritt (`fetch`) und danach die übrigen Skripte der Seite
  (Attribut `data-load` in `index.html`). `tr("results.runsN", { runs })` liefert den Text, `I18n.format` ersetzt
  Platzhalter. Statische Texte in `index.html` tragen `data-i18n="…"` (Inhalt) bzw. `data-i18n-title`, `-aria-label`,
  `-placeholder`. Konstanten mit Texten werden mit `onLanguage(() => X = …)` definiert und nach einem Wechsel neu
  gesetzt; danach zeichnet `redrawTexts` (app.js) alles neu, Planung, ungespeicherte Änderungen und Ansicht bleiben.
  Ein unbekannter Schlüssel erscheint als Schlüssel und zählt als Fehler der Oberfläche (`window.__errors`).
- **Desktop-App:** `crates/app/src/i18n.rs` bindet die Tabellen mit `include_str!` ein (native Menüs, Dateidialoge;
  eine neue Sprache braucht dort eine Zeile in `TABLES`). Das Menü «Ansicht → Sprache» zeigt «wie System» und die
  vorhandenen Sprachen mit Häkchen; ein Klick meldet `language-<wahl>` an die Oberfläche. Nach jedem Wechsel meldet die
  Oberfläche Sprache und Wahl mit `set_menu_language`, die App baut die Menüs neu. Die Startmeldung kommt als Schlüssel
  mit Dateiname (`startup.notFound`, `startup.unreadable`).
- **Sprachwahl:** Einstellung `language` (`settings_get`/`settings_set`, im Browser `localStorage` `rp.settings`), nie in
  der Planungsdatei: `system` (Vorgabe) oder ein Sprachcode. Beim allerersten Start (noch keine Einstellungsdatei, #70)
  liefert die App die Systemsprache (`first_start_language`, Crate `sys-locale`), sonst gilt bei «wie System»
  `navigator.languages`. Nur die Sprache zählt, nicht die Region (fr_FR → fr); ohne Tabelle gilt Deutsch.
- **Zahlen und Daten** bleiben in jeder Sprache im Schweizer Format: `fmtInt`, `fmtMoney`, `fmtPct`, `ymShow` in
  `util.js` formatieren mit `toLocaleString("de-CH")` (1'234'567.50, 2.5 %, 03.2040), `parseAmount`/`parseNum` lesen
  Eingaben unabhängig von der Sprache. Neue Zahlen immer über diese Funktionen formatieren.
- **Feste Meilensteine** (`start`, `end`, `retire`, `ahv`, `retire2`, `ahv2`) speichern keinen Namen; die Oberfläche zeigt
  `milestone.<id>` aus der Tabelle (`msLabel`). `upgrade.rs` entfernt die deutschen Standardnamen älterer Dateien, ein
  eigener Name von Start oder Ende bleibt (`model::stored_milestone_name`). Der Rechenkern setzt für Meldungen und
  Ereignisse den englischen Namen anhand der `id` ein (`Milestone::label`, `model::fixed_milestone_name`, #176); als Parameter einer Meldung ist ein fester Meilenstein die Meldung
  `milestone_<id>` (`Milestone::name_arg`, `model::fixed_milestone_message`), die Oberfläche übersetzt sie.
- **Prüfungen:** `tools/check_ui_texts.py` (in `make test-quiet`, `make ui-texts`, Job `english`) meldet deutsche Texte
  in `crates/app/ui/*.js`, `index.html` und `crates/app/src/*.rs` ausserhalb der Tabelle; Ausnahme mit
  `i18n: allow (Grund)` auf der Zeile. Der Rust-Test `issue_170_ak4_tables_are_complete` vergleicht die Schlüssel
  aller Tabellen mit `de.json` (vorhanden, nicht leer, gleiche Platzhalter). `e2e/ui-sprache.spec.js` prüft Wechsel,
  Rückfall und Formate mit einer Pseudo-Sprache.
- **Prüftabelle für die Übersetzer** (#182): `make i18n-table` schreibt `local/i18n-review/tabelle.csv` (Strichpunkt,
  UTF-8 mit BOM für Excel): je Schlüssel der Texttabellen und je Block der Hilfe (Überschrift, Zeile «Stichworte:»,
  Absatz) eine Zeile mit Ort, Kennzeichnung «kritisch», DE, EN, FR, IT und leeren Korrekturspalten je Sprache. Die
  Hilfedateien müssen in allen Sprachen gleich viele Blöcke in derselben Reihenfolge haben (auch das Glossar in
  `60-referenz.md` folgt in allen Sprachen der deutschen Reihenfolge); bei abweichender Anzahl bricht das Skript
  mit der Datei ab. Dieselben Zeilen stehen in Teilen für ein Sprachmodell unter `local/i18n-review/teile/teil-NN.txt`
  (kritische zuerst, `--part-chars`), dazu der Abschnitt «Übersetzungen» aus `docs/GLOSSAR.md` als
  `local/i18n-review/glossar.md`.

### Meldungscodes des Rechenkerns (#171)

Jede Meldung, die der Rechenkern an die Oberfläche gibt, ist eine `message::Message`: ein stabiler Code (englisch,
snake_case, z. B. `pillar3a_over_max`), Parameter als Werte (Zahlen, Namen, Monate `JJJJ-MM`, Listen und verschachtelte
Meldungen, nie formatierter Text) und der daraus gebildete englische Text (seit #176). Dazu gehören Prüfmeldungen (`Issue`:
`message`, `code`, `params`), Ereignistexte der Simulation (`Event`: `text`, `code`, `params`), Gründe und Hinweise in
den Ergebnissen (`PropertyEvent.reason`/`note`, `Renewal.emergencySale`, `BucketRefill.reason`/`returnBasis`,
`NeedBasis.source`), die Bezeichnung des Laufs (`label`, z. B. `path_bootstrap`), die Basis der Vorschau (`basis`),
Hinweise von `upgrade_plan`, `import_legacy`, `import_shiller`/`import_snb` (`notes`) und alle Fehler
(`Error::Invalid`/`Io` tragen eine Meldung, `Error::message()`; `error` in den Antworten). Als eigenes Objekt
erscheint eine Meldung als `{ "code", "params", "text" }`.

- **Tabelle:** `crates/engine/src/message.rs`, Makro `codes!` am Ende: Code ⇒ englische Vorlage, dieselbe wie
  `engine.<code>` in `en.json` (Enum `Code`, ein
  Tippfehler im Code ist ein Übersetzungsfehler). Aufbau mit `msg!(code, name = wert, …)`. Platzhalter `{name}` oder
  `{name:format}`: ohne Format Text, Zahl, Liste (", ") oder verschachtelte Meldung; `int` gerundet mit `'`
  (12'500), `fix0`–`fix2` feste Nachkommastellen, `pct0`–`pct2` Anteil × 100, `ym` Monat als MM.JJJJ, `lines` Liste
  zeilenweise. Geldbeträge der Ereignisse sind Zahlen mit `{x:fix0} {currency}` (Parameter `currency`). Parameter
  sind `message::Arg` (Zahl, Text, Liste, Meldung), keine JSON-Maps: Läufe mit vielen Pfaden erzeugen viele
  Ereignisse; der Text entsteht erst bei Gebrauch, und `batch::run_all`/`run_paths`/`run_with_bands` behalten nur
  Zusammenfassung und Bezeichnung (`summary_only`), damit die Ereignislisten parallel freigegeben werden.
- **Oberfläche:** dieselben Vorlagen stehen als `engine.<code>` in `crates/app/ui/i18n/<sprache>.json`. `engineText` in
  `util.js` (Aufruf `msgText(m)`) formatiert genau wie der Rechenkern (auch Rundung bei Gleichstand auf die gerade
  Ziffer wie `format!`); fehlt der Code in der Tabelle, erscheint der mitgelieferte `text`. Fehler der Befehle kommen
  in der Desktop-App als Meldung (`CmdError`), vom Entwicklungsserver als `{error, code, params}`; `api.js` macht daraus
  einen `Error` mit `code` und `params`, `errText` übersetzt ihn. Die Oberfläche wertet keine Texte des Rechenkerns
  mehr aus, nur Codes (`isCancelled`: `cancelled`, `randomPath`: `path_bootstrap`/`path_random`).
- **Kommandozeile und Golden-Dateien** verwenden den englischen Text (#176). Die Kommandozeile gibt auch ihre eigenen
  Texte (Hilfe, Überschriften, Fehler) englisch aus, Zahlen im Schweizer Format, auch die Namen aus dem Katalog der
  Optimierung und der Zeitreihen (#177). Der
  Golden-Vergleich (`crates/engine/tests/golden.rs`, `without_codes`) entfernt `code` und `params` und vergleicht eine
  alleinstehende Meldung mit ihrem Text. Die Golden-Dateien wurden mit #176 einmal neu erzeugt; `make golden-numbers
  BASE=<rev>` (`tools/golden_numbers.py`) vergleicht zwei Stände ohne Textfelder (Zahlen, Schlüssel, Listenlängen,
  Monate); bei jedem Push prüft der Job `golden` dasselbe gegen den Stand vor dem Push (#195, `docs/TESTS.md`).
- **Englische Texte:** Eine Korrektur einer englischen Vorlage gilt für `en.json` und `codes!` gemeinsam (Test AK1);
  danach Beispieldatei neu schreiben und Golden-Dateien neu erzeugen.
- **Tests (AK3):** `issue_171_ak3_examples_file_is_current` (Rechenkern) schreibt je Code ein Beispiel
  (`message::example`: Werte nach Format der Platzhalter) nach `e2e/unit/engine-messages.json`; ein neuer Code macht
  die Datei veraltet (neu schreiben mit `MESSAGES_UPDATE=1 cargo nextest run -p vivaplan-engine -E
  'test(issue_171_ak3)'`). `e2e/unit/engine-messages.test.js` rendert jedes Beispiel mit `engineText` und der Tabelle
  `en` und verlangt genau den Text des Rechenkerns (#176/AK1);
  `issue_171_ak3_engine_codes_render_the_engine_text` (App, in `make test-quiet`) prüft dasselbe mit `message::render`
  gegen `en.json` und meldet Schlüssel `engine.*` ohne Code. `e2e/ui-sprache.spec.js` (#176/AK4) prüft, dass die
  deutsche Oberfläche keinen englischen Text des Rechenkerns zeigt.
- **Neue Meldung:** Code und englische Vorlage in `codes!`, gleicher Schlüssel `engine.<code>` in `en.json` (gleiche
  Vorlage), `de.json` und den übrigen Tabellen, Beispieldatei neu schreiben.
- **Namen als Meldungen (#177):** Auch Namen sind Meldungen mit Code, damit die Oberfläche sie übersetzt: Strategien
  und Parameter der Optimierung (`opt_strategy_<key>`, `opt_param_<key>`, Einheiten `unit_*`; `CatalogStrategy.name`,
  `CatalogParam.label`/`unit`, `Outcome.strategy`/`params`, `Reference.strategy`), Indizes und Anlagestrategie
  (`index_*`, `investment_*`; `strategy::label`, `stock_name`, `bond_name`, `SeriesPart.label`) und die Texte der
  mitgelieferten Zeitreihen (`series_title_*`, `series_info_*`, `series_note_*`, `series_source_*`; `market::Meta`
  und `SeriesInfo`: `title`, `source`, `info`, `notes`). Freie Texte eigener Reihen haben den Code `plain_text`.
  `series_text::recognize` ordnet die Kommentarzeilen einer CSV-Datei beim Lesen ihrem Code zu, englisch oder deutsch
  (`STORED_DE`: Texte von Dateien früherer Versionen in der Datenbibliothek = `engine.<code>` in `de.json`, Test
  `issue_177_ak1_stored_german_series_texts_are_the_table_de`). Die Schlüssel in der Datei (`optimization.ranges`,
  Kennungen der Strategien und Indizes) bleiben unverändert; gespeicherte Ergebnisse mit Namen als Text zeigt
  `msgText` unverändert an.

### Release Notes (#188)

`CHANGELOG.md` (deutsch) ist massgebend; `CHANGELOG.en.md`, `CHANGELOG.fr.md` und `CHANGELOG.it.md` übersetzen jede
Version ab 1.0.0-beta.1, ältere Versionen stehen nur deutsch. Jeder Eintrag gliedert sich in höchstens drei Abschnitte in
fester Reihenfolge: «Kann Ergebnisse ändern» (nur wenn es solche Änderungen gibt), «Neu, geändert, behoben» und «Tests und
Entwicklung»; die Überschriften je Sprache stehen in `tools/changelog.py` (`SECTIONS`).

Ablauf pro Release:

1. Den deutschen Eintrag in `CHANGELOG.md` schreiben, mit Issue-Nummern.
2. Ihn in die drei Übersetzungen übertragen: dieselben Abschnitte und Issue-Nummern, Begriffe nach dem Glossar
   (`docs/GLOSSAR.md`).
3. Prüfen: `make changelog` bzw. `python3 tools/changelog.py check` (läuft auch in `make test-quiet` und im Job
   `english`). Gemeldet werden abweichende Versionen, Abschnitte oder Issue-Nummern und verbotene Varianten aus dem
   Glossar.
4. Versionsnummern nachführen, committen, das Tag `v…` setzen und pushen. `pakete.yml` bricht ab, wenn die Version in
   einer Sprache fehlt, und setzt den Text aus `python3 tools/changelog.py notes v…` als Beschreibung des
   Release-Entwurfs: Englisch offen, Deutsch, Französisch und Italienisch aufklappbar.
   Der Titel ist sprachneutral, nur Produkt und Version (`python3 tools/changelog.py title v…`, z. B. v1.0.0-beta.2 →
   «VivaPlan 1.0.0 Beta 2»).

## Desktop-App

```sh
cd crates/app && npm install   # einmalig: Tauri-Kommandozeile
npx tauri build                # erzeugt target/release/bundle/macos/VivaPlan.app und .dmg
npx tauri dev                  # App im Entwicklungsmodus
```

Für Windows bzw. Linux wird `npx tauri build` auf dem jeweiligen System ausgeführt.

**App-Icon (#185):** Quellen sind `crates/app/icons/icon.svg` (Variante «Gipfelweg – Abendrot», ab 48 px) und die
vereinfachte Fassung `crates/app/icons/icon-small.svg` (ohne Weg, grössere Sonne; 16, 24 und 32 px). Nach einer Änderung
an einer der beiden Dateien alle Rasterdateien neu erzeugen (macOS):

```sh
make icons                     # = tools/make-icons.sh --preview
```

Das Skript rendert mit `npx tauri icon` (`@tauri-apps/cli`), setzt `icon.ico` (Python) und `icon.icns` (`iconutil`) aus
beiden Fassungen zusammen, kopiert `icon-small.svg` als Favicon nach `crates/app/ui/favicon.svg` und schreibt die
Vorschau `docs/bilder/icon-vorschau.png` (16/32/128/512 px auf hellem und dunklem Grund; braucht ImageMagick).
Das Ergebnis ist reproduzierbar: zweimal erzeugt, gleiche Dateien. `tools/test_icons.py` (in `make test-quiet`) prüft
Farben, Grössen der Rasterdateien, die Einträge von `icon.ico` und das Favicon.

Unter macOS baut der Git-Hook `tools/git-hooks/post-commit` nach jedem Commit, der Rechenkern,
Oberfläche oder Cargo-Dateien ändert, die App im Hintergrund und ersetzt `/Applications/VivaPlan.app`
(Skript `tools/build-install.sh`, Protokoll in `target/build-install.log`). Reine Test- und Dokumentationscommits
lösen keinen Build aus. Für einen einzelnen Commit abschalten: `BUILD_INSTALL=0 git commit …`.
Nach jedem Push auf `main` holt `tools/git-hooks/pre-push` im Hintergrund die Übersichtsseite «Testqualität» des
GitHub-Laufs nach `local/test-quality/report/` (`tools/fetch-report.sh`, docs/TESTS.md; abschalten mit `FETCH_REPORT=0 git push`).
Die Hooks gelten nach `git config core.hooksPath tools/git-hooks`.
Zum Testen der Oberfläche im Browser (ohne Dateidialoge): `vivaplan ui-dev` aus dem Projektordner starten
und <http://127.0.0.1:8787/> öffnen; der Server lauscht nur lokal.

**Bedienung:** Menüs Datei, Bearbeiten, Ansicht (Sprache) und Hilfe; Tastenkürzel (macOS ⌘, sonst Ctrl): ⌘N neu, ⌘O öffnen, ⌘S speichern, ⇧⌘S speichern unter,
⌘I alte config.json importieren, ⌘↩ Simulation starten, Esc zurück zum Editor, ⌘Z / ⇧⌘Z rückgängig / wiederholen
(ganze Planung), ⌘1 … ⌘0 Abschnitt bzw. Registerkarte, ⌘↑ / ⌘↓ voriger / nächster Abschnitt, ⌘/ Übersicht.

**Ergebnisse:** Registerkarten Übersicht, Grafiken, Einzelne Läufe (5./50./95. Perzentil), Szenarien, Jahreswerte;
alle Beträge wahlweise nominal oder in heutiger Kaufkraft (Preise des Simulationsstarts). Grafiken eines Laufs:
Vermögensverlauf, liquides Vermögen nach Anlage (Beträge oder Anteile), Einnahmen und Ausgaben pro Jahr,
Entnahmerate, Immobilien (Wert, Hypothek, Eigenkapital), Vorsorge. Bei mehreren Läufen zusätzlich: Bandbreite,
Wahrscheinlichkeit, dass das Geld reicht (über die Zeit), Verteilung von Endvermögen und Vermögen bei Pensionierung,
Alter bei Erschöpfung. Szenarien: Erfolgsquote und Endvermögen je Variante (eine Variation) bzw. Heatmaps (zwei).

## Befehle

```sh
cargo build --release
B=target/release/vivaplan

$B run crates/engine/assets/example.vplan --events          # Jahresübersicht, Kennzahlen, Ereignisse
$B run crates/engine/assets/example.vplan --csv monate.csv  # alle Monatswerte
$B validate crates/engine/assets/example.vplan              # Fehler und Hinweise
$B import-legacy local/eigene/config.json --start-year 2026 --base local/eigene -o meine.vplan
$B bench crates/engine/assets/example.vplan --paths 50000   # Rechenleistung
$B optimize crates/engine/assets/example.vplan --max-ruin 5 --ruin-age 95   # beste Entnahmestrategie suchen
$B optimize crates/engine/assets/example.vplan --compare --property both    # Strategien ohne Suche vergleichen (#197)
$B schema > planung.schema.json                  # JSON-Schema der Projektdatei
$B library                                       # Zeitreihen der Datenbibliothek (Ordner crates/engine/data/)
$B import-shiller ie_data.xls                    # Shiller-Daten neu einlesen (.xls, .xlsx, .ods)
$B import-snb devkum.csv rendoblim.csv           # SNB-Daten aktualisieren, CHF-Reihen neu berechnen

make test                                        # alle Tests (nextest, Grund in docs/TESTS.md)
make ui-unit                                     # Hilfsfunktionen der Oberfläche (node --test)
make clippy                                      # clippy ohne Warnungen (läuft bei jedem Push)
make english                                     # Code intern englisch: deutsche Kommentare und Testnamen (#165)
tools/ui-smoke.sh                                # Oberflächentest
cargo llvm-cov --workspace --exclude vivaplan-app --summary-only   # Testabdeckung
```

`returns.strategy` (#69) wählt die Indizes: `{"stocks": "worldJst" | "worldCustom" | "chJst" | "sp500",
"bonds": "ch" | "usa" | "chShort", "weights": {"ch", "usa", "eu", "apac"}}` (Gewichte nur für `worldCustom`, zusammen 1).
Daraus entsteht die Zeitreihe (`crates/engine/src/strategy.rs`); ein gesetztes `returns.series` (ältere Planungen,
Kommandozeile) hat Vorrang und wird beim Öffnen in der App zur Strategie.

`returns.source` steuert die Läufe: `fixed` (ein Lauf), `historical` (ab `startDate`), `rolling`
(jeder historische Startzeitpunkt, der den ganzen Zeitraum abdeckt), `bootstrap` und `parametric` (Monte Carlo).
Alle Läufe werden parallel auf allen Kernen gerechnet; Monte-Carlo-Pfade entstehen erst im Lauf (wenig Speicher).
Die Bandbreiten halten aber je Pfad und Monat das Vermögen (nominal und real, `f32`); zusammen mit den Ergebnissen je Lauf
sind das rund 23 Bytes je Pfad und Monat. Die Prüfung begrenzt deshalb Pfade × Monate auf 50 Mio.
(`validate::MAX_PATH_MONTHS`, #139; an der Grenze rund 1,3 GB bei der schwersten Testplanung).

## Monte Carlo

- **`bootstrap`** (`{"paths": 50000, "block": 12, "seed": 42}`): stationärer Bootstrap aus der gewählten Zeitreihe.
  Pro Monat endet ein Block mit Wahrscheinlichkeit 1/`block`, dann folgt ein zufälliger neuer Startmonat; am Ende
  der Reihe geht es am Anfang weiter. Aktien, Obligationen, Inflation und SARON stammen immer aus demselben
  historischen Monat. Bei Jahresreihen gilt die Blocklänge in Jahren (`block` / 12).
- **`parametric`** (`{"paths", "mu", "sigma", "df", "jumpLambda", "jumpMu", "jumpSigma", "seed", "auto", "sampler"}`): Aktienmodell
  des bisherigen Programms in Tagesschritten (21 pro Monat): Log-Rendite (μ − σ²/2)·dt + σ·√dt·Z mit Student-t-Schocks Z
  (auf Varianz 1 skaliert, `df` = 0 → Normalverteilung) plus Poisson-Sprünge (Rate `jumpLambda` pro Jahr, Höhe
  log-normal mit `jumpMu`, `jumpSigma`). Der SARON folgt dem fixen Wert. Die Zeitreihe der Planung (Anlagestrategie)
  wird auch für `parametric` geladen (`batch::load_series`); ohne Reihe folgen auch Obligationen und Inflation den
  fixen Werten.
- **Zufallsziehung des Aktienmodells (#132, `montecarlo::SAMPLER` = 1):** Student-t nach der Polar-Methode von Bailey:
  (u, v) gleichverteilt im Einheitskreis, beide aus einer 64-Bit-Zufallszahl (je 32 Bit), w = u² + v², dann ist
  u·√(df·(w^(−2/df) − 1)/w) exakt t-verteilt; bei `df` = 6 mit ∛w statt der allgemeinen Potenz. Ob ein Punkt angenommen
  wird, entscheidet reine Arithmetik (w < 1), keine Funktion der Mathematikbibliothek. Kurssprünge: Statt jeden Tag eine
  Poisson-Zahl zu ziehen, wird der Abstand zum nächsten Tag mit Sprung gezogen (geometrisch, ⌊−ln U / λ⌋) und an diesem
  Tag die Anzahl unter der Bedingung «mindestens einer». Die Verteilung ist dieselbe wie vorher, die Pfade sind andere.
  Gemessen (Apple M1 Pro, Musterplanung, 50'000 Pfade, Release): vorher 2,85 s, nachher 1,32 s; je Pfad auf einem Kern
  Pfaderzeugung 373 → 139 µs, die Simulation bleibt bei rund 56 µs. `returns.parametric.sampler` hält das
  Verfahren, mit dem die Planung gespeichert wurde; fehlt es, nennt `upgrade` beim Öffnen die anderen Pfade
  (`SAMPLER_NOTE`, nur wenn die Planung parametrisch rechnet).
- **Vorschlagswerte (#100, `calibrate.rs`):** aus der ganzen Aktienreihe des Index (`DataSet::stock_series`, nicht
  nur die gemeinsamen Jahre mit den Obligationen). Ziel sind die Jahresrenditen, wie der Bootstrap sie zieht: bei
  Jahresreihen die Jahre selbst, bei Monatsreihen (S&P 500 mit US- oder 10-jährigen Eidgenossen) eine feste
  Stichprobe von 100'000 Jahren aus Blöcken mit mittlerer Länge 12 Monate. `df` = 6 und `jumpSigma` = 0.15 sind fest.
  Für jede Kombination aus `jumpLambda` (0 bis 0.30, Schritt 0.01) und `jumpMu` (−0.05 bis −0.50, Schritt 0.05) folgen
  μ und σ geschlossen aus Mittelwert und Streuung (die Jahresrendite ist eine Poisson-Mischung von
  Lognormalverteilungen); gewählt wird die Kombination mit dem kleinsten Fehler bei 5 %-Quantil und geometrischem
  Mittel. Gerundet auf 0.05 Prozentpunkte. Mit `auto` = wahr rechnet der Rechenkern mit den Vorschlagswerten
  (`calibrate::resolve`), sonst mit den gespeicherten Werten; fehlt `auto` (ältere Planung), gilt falsch.
  `upgrade_plan` setzt `auto` = falsch bei Planungen, die parametrisch rechnen (mit Hinweis), sonst wahr.
- **Obligationen und Inflation (#101):** Mittelwert, Streuung (Nenner n) und Korrelation (Aktien, Obligationen,
  Inflation) der Jahreswerte der Reihe der Anlagestrategie (Monatsreihen zu Kalenderjahren verknüpft, Inflation des
  Dezembers). Je Kalenderjahr der Simulation: z = (L − E[L]·f) / (sd(L)·√f) aus der Log-Rendite L der Aktien in den
  f·12 Monaten dieses Jahres; Obligationen = μB + σB·(aB·z + l11·e1), Inflation = μI + σI·(aI·z + l21·e1 + l22·e2) mit
  unabhängigen Standardnormalen e1, e2 aus einem eigenen Zufallsstrom (die Aktienpfade bleiben gleich). a = Korrelation
  der Reihe / corr(exp(L) − 1, L), damit die Korrelation mit der einfachen Aktienrendite stimmt; l aus der Zerlegung der
  Restkovarianz (begrenzt, falls die Matrix nicht positiv definit wäre). Werte unter −95 % werden auf −95 % gesetzt. Die
  Jahre sind unabhängig, wie im Bootstrap.
- **Vorlauf (#103, `MarketPath::pre_stocks`):** Jeder Pfad trägt die Aktien-Monatsrenditen der 12 Monate vor dem
  Start mit, nur für das Renditekriterium der 3-Topf-Strategie (kein Vermögen, kein Topf). Historisch: die echten Monate
  vor dem historischen Start (Jahresreihen nach Kalenderjahr, wie der Pfad), am Anfang der Reihe entsprechend weniger.
  Bootstrap: die echten Monate vor dem ersten Block, zirkulär (Jahresreihen: Monate des Startjahres aus dem ersten
  gezogenen Jahr, die übrigen aus dem Jahr davor). Parametrisch: 12 Monate aus demselben Zufallsstrom, vor dem Pfad
  gezogen. Fixe Renditen: der Satz des Startmonats.
- Pfad *i* entsteht immer aus (`seed`, *i*): gleiche Einstellungen ⇒ gleiche Ergebnisse; alle Szenarien verwenden
  dieselben Pfade. Die Detailansichten (5./50./95. Perzentil) rechnen den betreffenden Pfad exakt nach.
- «Alle historischen Startzeitpunkte» unterschätzt die Streuung: 864 Startmonate aus 104 Jahren überlappen stark
  (nur gut drei unabhängige 32-Jahres-Abschnitte). Der Bootstrap zeigt deshalb breitere Bandbreiten.

## Szenarien

`scenarios.sweeps` = Liste von `{"param", "from", "to", "step"}`. Parameter: `wealth.stockShare`, `pension.pk.lumpsum`,
`returns.startDate` (Monat, Schritt in Monaten), `milestone.retire` (Monat; setzt die Pensionierung als Datum),
`flows.spendingFactor` (Faktor auf die Lebenshaltungskosten ohne Wohnanteil). Jede Kombination (technisch höchstens 1'000'000) wird mit allen Pfaden
gerechnet: Erfolgsquote, Vermögen bei Pensionierung (Median), Endvermögen 5./50./95. Perzentil, früheste Erschöpfung.

## Datenbibliothek

Planungen verweisen mit `"series": "library:Shiller CHF.csv"` auf eine Reihe der Bibliothek, sonst mit einem
Pfad (absolut oder relativ zur Projektdatei). Die Desktop-App führt die Bibliothek im App-Datenordner
(macOS: `~/Library/Application Support/ch.vivaplan-app.desktop/Zeitreihen`) und legt dort fehlende
ausgelieferte Reihen an; Kommandozeile und Entwicklungsserver verwenden `crates/engine/data/` (oder `--library <ordner>`).

| Reihe | Inhalt |
|---|---|
| `Shiller USD.csv` | Monatlich ab 01.1872: S&P Composite inkl. Dividenden, 10-jährige US-Staatsanleihen (Gesamtrendite), US-CPI |
| `Shiller CHF.csv` | Monatlich ab 01.1922: dieselben Anlagen in CHF (Monatsmittel USD/CHF der SNB, ohne Absicherung), Schweizer Landesindex der Konsumentenpreise, SARON ab 06.1999 |
| `Shiller CHF + CH-Obligationen.csv` | Monatlich 02.1988 – 07.2025: Aktien und Inflation wie `Shiller CHF`, Obligationen = 10-jährige Obligationen der Eidgenossenschaft |
| `Welt (JST).csv` | Jährlich 1900 – 2020: Welt-Aktienindex aus den JST-Daten (11 Länder, nach realem BIP gewichtet) in CHF, Obligationen der Eidgenossenschaft (JST bis 1988, danach SNB), Schweizer Teuerung; Lizenz CC BY-NC-SA 4.0 |

- **Shiller-Import:** Monatsrendite = Veränderung von realem Gesamtrendite-Index × CPI gegenüber dem Vormonat
  (Aktien entsprechen (P + D/12) / P₋₁ − 1, Obligationen Shillers «Monthly Total Bond Returns», dort um einen Monat
  früher eingetragen). Shillers Kurse sind Monatsdurchschnitte; Einbrüche erscheinen dadurch geglättet.
  Inflation = CPI gegenüber Vorjahresmonat, damit der Dezemberwert der Jahresteuerung entspricht.
- **Schweizer Obligationen:** Gesamtrendite einer 10-jährigen Obligation zu pari aus der Monatsrendite (SNB
  `rendoblim`, 10 Jahre): Coupon = Rendite des Vormonats, Bewertung zur neuen Rendite mit 9 11/12 Jahren Restlaufzeit,
  plus ein Monatscoupon. Die SNB führt diese Reihe seit 07.2025 nicht mehr nach.
- **SNB-Würfel** von data.snb.ch (CSV): `devkum` (Reihe M0/USD1), `plkopr` (LD2010100), `zimoma` (SARON), `rendoblim` (10J).
  Ein Auszug ist eingebaut; neuere Downloads ersetzen ihn (`crates/engine/data/quellen/snb-<würfel>.csv`).
- **CSV-Format:** Kommentarzeilen `# Titel: …`, `# Währung: CHF`, `# Quelle: …`, `# Hinweis: …` vor der Kopfzeile.
  Leere SARON-Werte sind erlaubt; dort gilt der fixe SARON-Wert der Planung. Die mitgelieferten Reihen haben seit
  #177 englische Texte; ältere deutsche Texte in der Datenbibliothek werden erkannt und übersetzt (`series_text.rs`).
- Ist die Reihe in einer anderen Währung als die Planung, meldet die Prüfung einen Hinweis.
- **Rechte:** Die Zeitreihen fallen nicht unter die GPL; Quellen, Bedingungen und Haftungsausschluss in
  `crates/engine/data/QUELLEN.md`.

## Projektdatei (`.vplan`, JSON)

Kennung `"format": "vivaplan/1"`. Ältere Dateien (`.rplan`, Kennung `pensionsplaner/1`) bleiben lesbar und werden
beim Speichern als neue `.vplan`-Datei geschrieben. Eine Datei ohne Kennung gilt als `vivaplan/1` (Desktop-App,
Kommandozeile; im Browser des Entwicklungsservers wird sie abgelehnt). Jede andere Kennung (z. B. `vivaplan/2`) stammt
aus einer neueren Version: Öffnen, `vivaplan run` und `vivaplan validate` lehnen sie mit der Meldung «Die Datei hat das
Format «…». Sie braucht eine neuere Version von VivaPlan und wird nicht geöffnet.» ab (`model::check_format`, #144).
Gespeichert wird atomar (`api::write_plan`, #136): zuerst in eine temporäre Datei im selben Ordner, dann ersetzt sie
die Zieldatei; bei einem Fehler bleibt die bisherige Datei unverändert. Ein symbolischer Link bleibt, ersetzt wird sein Ziel.

Alle Zeitpunkte sind Kalendermonate (`"2027-05"`) oder Verweise auf Meilensteine (`"retire"`), `"never"` = nie.
Meilensteine sind ein Datum oder das Alter einer Person (`{"kind": "age", "person": "A", "years": 65}`).
`start` und `end` sind Pflicht. Die festen Meilensteine (`start`, `end`, `retire`, `ahv`, `retire2`, `ahv2`) haben
keinen `name`; die Oberfläche zeigt ihren Namen in der gewählten Sprache (#170). Die Datei enthält keine Texte der
Oberfläche, eine Planung ist in jeder Sprache dieselbe Datei. Zeitabhängige Werte sind eine Zahl oder
`{"mode": "timed", "rows": [{"from": "start", "value": 9500}, {"from": "retire", "value": 8500}]}`.
Beträge gelten pro Monat, Sätze pro Jahr als Dezimalzahl (0.05 = 5 %). Das vollständige Format liefert `vivaplan schema`.

## Rechenregeln

Reihenfolge pro Monat:

1. **Teuerung:** jeden Monat Teuerungsindex × (1 + Inflation des Vormonats)^(1/12). Ausgaben, Mieten und (falls
   gewählt) Immobilienwerte und Nebenkosten werden damit fortgeschrieben (je Objekt einzeln wählbar:
   `inflationValue`, `inflationFixCosts`, `inflationRent`; fehlen sie, gilt `inflation` für Wert und Nebenkosten und die
   Miete wird fortgeschrieben); Ausgaben werden in Preisen bei
   Simulationsstart erfasst. Die AHV (falls indexiert) wird nur im Januar um die seit dem letzten Januar
   aufgelaufene Teuerung angepasst.
2. **Ereignisse:** einmalige Beträge; Pensionierung (Kapitalbezug mit Kapitalsteuer, Rente = Restkapital ×
   Umwandlungssatz / 12); AHV-Beginn (**Vorbezug, Aufschub und Plafond, #191:** `monthly` ist die Rente mit 65. Die
   Rechnung steht in `crates/engine/src/ahv.rs` (`Calc`, Sätze mit Gültigkeit und Quelle, Merkblatt 3.04, Stand
   1.1.2026, Ziffern 4, 11–14, 24–26): Anspruch hat eine Person ab Bezug (auch Vorbezug) oder Referenzalter (auch
   Aufschub). Bei Ehepaaren mit Anspruch beider werden die Renten mit 65 anteilig auf den Plafond gekürzt. Im Vorbezug
   wird die (plafonierte) Rente jeden Monat um den Satz gekürzt; ab dem Referenzalter gilt ein fester Kürzungsbetrag =
   Satz × Durchschnitt der (plafonierten) Renten im Vorbezug. Ein Aufschub ab 12 Monaten gibt ab Bezug einen festen
   Zuschlag = Satz × Durchschnitt der (plafonierten) aufgeschobenen Rente. Beträge auf ganze Franken; Kürzung und
   Zuschlag liegen ausserhalb des Plafonds und steigen im Januar mit der Rente. Vorbezug über 24 Monate wie 24, Aufschub
   über 5 Jahre wie 5, unter 12 Monaten ohne Zuschlag; die Prüfung warnt bei Bezug vor 63, nach 70 und bei einem
   Aufschub unter einem Jahr. Monate eines Vorbezugs oder Aufschubs vor dem Simulationsstart zählen mit den Beträgen
   beim Start. Die Vorschau gibt je Person `ahv` = Eingabe, Satz, effektive Rente ohne Plafond und bei Ehepaaren die
   Rente nach Plafond und Kürzung/Zuschlag ab dem Monat, in dem beide Anspruch haben. Ältere Planungen ohne Marke
   `pension.ahv.referenceAge` hielten die bereits angepasste Rente: `upgrade.rs` rechnet sie auf die Rente mit 65 zurück
   (Betrag / (1 + Satz), auf Rappen) und meldet es je Person einmalig); Immobilien (Notverkauf aus dem Vormonat, Kauf, Verkauf mit
   Grundstückgewinnsteuer, Hypothekarperioden); Säule 3a (Verzinsung, Auszahlung).
3. **Geldflüsse** in das Cash: Einkommen + Renten + Zuflüsse − Lebenshaltung − Wohnkosten.
   **13. AHV-Rente (#190):** ab 2026 im Dezember zusätzlich zur AHV des Monats; Höhe 1/12 der von Januar bis Dezember
   ausbezahlten AHV des Haushalts (nach dem Plafond für Ehepaare, keine zweite Deckelung), je Monat auf Rappen, die Summe
   kaufmännisch auf ganze Franken gerundet (KS 13. AR Rz 4001–4005). Monate des Startjahrs vor dem Simulationsstart zählen
   mit den dann fälligen Renten; Monate vor 2026 zählen nicht. Ohne AHV im Dezember (auch: Simulation endet vorher)
   keine 13. Rente. Sie zählt zum AHV-Einkommen der Jahreswerte und zum steuerbaren Einkommen; kein Schalter. Tragbarkeit und
   Notverkaufskriterium rechnen mit den Monatsrenten ohne 13. Rente; der Netto-Bedarf der 3-Topf-Strategie zählt sie (#198). Ältere
   Planungen (ohne die Marke `pension.ahv.thirteenthNoted`) bekommen beim Öffnen einmalig einen Hinweis (`upgrade.rs`,
   nur mit AHV-Rente); sonst bleibt die Datei unverändert.
4. **Rendite** auf Aktien und Obligationen (Jahresrendite geometrisch auf Monate umgerechnet), sofern das Portfolio positiv ist.
5. **Pensionskasse** bis zum Bezug: Beiträge monatlich, Verzinsung im Dezember.
6. **Steuern:** Im Dezember werden die Einkommenssteuer auf das Jahreseinkommen (inkl. Renten) und die
   Vermögenssteuer auf Portfolio + Immobilien netto veranlagt; bezahlt wird sie im Mai des Folgejahres. Eine bei
   Planende noch offene Steuer wird im letzten Monat abgezogen.
7. **Notverkauf prüfen:** wenn liquides Vermögen < Schwelle × (Ausgaben − Einkommen); ausgeführt im Folgemonat.
   Ist danach (oder nach einem geplanten Verkauf) keine Immobilie mehr im Besitz, wird im Folgemonat die nächste
   Immobilie mit Kauf `"afterSale"` gekauft, sofern tragbar (Tragbarkeit wie bei den Banken, Hypothek allenfalls
   gekürzt) und die Eigenmittel reichen; sonst beginnen die Mieten mit `"from": "afterSale"`.
   Mit `"buy": "afterSale:<id>"` wird im Monat nach dem Verkauf einer bestimmten Immobilie gekauft (Verweis über
   deren `id`); klappt der Kauf nicht und ist keine Immobilie mehr im Besitz, beginnen die Mieten.
   Die Tragbarkeit rechnet nur mit dem Einkommen (Erwerb, Renten): (`sustainability` × Jahreseinkommen − Nebenkosten) /
   `imputedRate`; `wealthContribution` wird nicht mehr verwendet. Sie wird auch bei **jeder Erneuerung** einer Hypothekarperiode
   geprüft (Einkommen im Erneuerungsmonat, Nebenkosten wie erfasst, kalkulatorischer Zins `imputedRate`): Der nicht tragbare
   Teil ist die minimale Einmalzahlung. Sie wird aus dem Vermögen anteilig auf alle Tranchen bezahlt, wenn das Vermögen
   danach >= 0 ist und das Notverkaufskriterium nach der Zahlung nicht auslöst (Schätzung: geplante Ausgaben + Zins,
   Amortisation und Nebenkosten nach der Zahlung − Einkommen); die Hypothek wird dann reduziert erneuert. Sonst keine
   Zahlung, keine Erneuerung, Notverkauf im Folgemonat. Jede Erneuerung steht in `renewals` des Laufs
   (`affordable`, `lumpSum`, `debtBefore`, `debtAfter`, `emergencySale` = «Vermögen < 0» oder «Notverkaufskriterium»); die
   Zusammenfassung zählt `lumpSums` und `affordabilitySales`, die Stapelauswertung `lumpSumRuns` und `affordabilitySaleRuns`.
   Mit `"amountMode": "maxLoan"` beträgt die Hypothek beim Kauf die höchste Belehnung (80 %, also Minimum
   Eigenmittel), gekürzt falls nicht tragbar; die Beträge der Tranchen gelten dann als Anteile.
   **«Immobilien nie verkaufen»** (`"realestate": {"neverSell": true}`, #196; fehlt das Feld: aus, «aus» wird nicht
   geschrieben): kein geplanter Verkauf (`sell` wird beim Übersetzen ignoriert, die Tranchen laufen bis zum Planende),
   kein Notverkauf (weder Kriterium noch 3-Topf «Töpfe reichen nicht»), und bei einer nicht tragbaren Erneuerung, die
   sonst einen Notverkauf auslösen würde, wird die Hypothek voll erneuert (Ereignis `event_mortgage_unaffordable_kept`,
   `emergencySale` leer, `affordabilitySales` unverändert); eine Einmalzahlung, die die Bedingungen oben erfüllt, bleibt.
   Käufe mit Datum oder Meilenstein finden statt, Käufe und Mieten «nach Verkauf» nicht (die Prüfung warnt:
   `property_never_sell_sale`, `property_never_sell_after_sale`). «Aufgebraucht» gilt dann auch mit Immobilie im Besitz.
   Für einen Lauf setzt man das Feld in einer Kopie der Planung (`plan.realestate.never_sell = true`) vor `Simulator::new`.
8. **Entnahmestrategie «Standard»:** Cash = Reserve (Monate × ungedeckte Ausgaben), Rest nach Aktienanteil.
   **«3-Topf»** (`"strategy": "three_buckets"`, `"buckets": {"cashYears", "bondYears", "refillThreshold", "reserveMonths"}`)
   ab dem Start der Entnahmephase (die erste Pensionierung), davor «Standard». Topf 1 = Cash (alle Ausgaben werden
   daraus bezahlt), Topf 2 = Obligationen, Topf 3 = Aktien; ein für einen Kauf im Folgemonat reservierter Verkaufserlös
   gehört zu keinem Topf. **Kein Topf fällt je unter 0** (einzige Ausnahme: alle Töpfe leer und keine Immobilie mehr).
   **Netto-Bedarf** = Lebenshaltung + Wohnen (ohne Amortisation) + Einkommenssteuer − AHV − PK-Rente − Erwerbseinkommen,
   aus den geplanten Werten des laufenden Jahres (Beginn mit einem Auslöser A: Monate ab dem Auslöser, auf 12 Monate
   hochgerechnet, auch die Einkommenssteuer); einmalige Ausgaben, Käufe, Amortisationen und Sondersteuern zählen nicht.
   Bedarf ≤ 0 → Ziele 0. **AHV mit 13. Rente (#198):** ab 2026 AHV-Summe der geplanten Monate (hochgerechnet) × 13/12,
   im vollen Jahr also 13 Monatsrenten, vor 2026 12; Basis ist die plafonierte AHV des Haushalts ohne zweite Deckelung.
   Die Einkommenssteuer im Netto-Bedarf rechnet mit diesem Jahreseinkommen inklusive 13. Rente.
   Der Netto-Bedarf wird im Januar und bei **jedem** Auslöser A neu bestimmt, also auch bei einem PK-Kapitalbezug mitten im
   Jahr (Monate ab dem Bezug, hochgerechnet), und gilt bis zum nächsten Januar oder Auslöser A (#99, abweichend von der
   früheren Spezifikation, die nur beim Start und Neustart neu hochrechnete).
   Ziel Topf 1/2 = Jahre × Netto-Bedarf, nie aus dem Topfstand. **Renditekriterium:** Aktienrendite der letzten 12 Monate
   einschliesslich des laufenden (Indexstand Ende Monat im Verhältnis zum Stand 12 Monate davor, nicht annualisiert); in
   den ersten 12 Monaten liefert der Vorlauf des Renditepfads die Monate vor dem Start (#103). Fehlen Vorlaufmonate
   (Reihe beginnt später), gilt die Rendite seit dem frühesten bekannten Stand («seit Start»). Rendite ≥ `refillThreshold`,
   sonst sind nur Verkäufe aus Topf 3 gesperrt.
   **Januar-Auffüllung:** Topf 1 aus Topf 2, der Rest aus Topf 3, danach Topf 2 aus Topf 3 (bei nicht erfülltem Kriterium: nur
   Topf 1 aus Topf 2, Topf 2 wird nicht nachgefüllt); Überschuss in Topf 1 oder 2 → Topf 3.
   **A** (Start der Entnahmephase, PK-Kapitalbezug, Neustart nach «alle Töpfe leer»; einmal pro Monat): Kapitalbezug bzw. das
   gesamte liquide Vermögen zuerst in Topf 3, dann wie im Januar ohne Renditekriterium, Ziele um die vergangenen Monate des
   Jahres gekürzt ((Jahre × 12 − Monate) / 12 × Netto-Bedarf). **B** (externe Zuflüsse, 3a, Verkauf ohne Folgekauf, Notverkäufe):
   nur Topf 3, keine Auffüllung. **C** (Vorausschau, jeden Monat; deckt Topf 1 die Ausgaben des Monats (laufende und einmalige) nicht):
   **C1** bei erfülltem Renditekriterium: Ablauf wie die Januar-Auffüllung, Ziele um die seit Januar vergangenen Monate
   gekürzt ((Jahre × 12 − Monate) / 12 × Netto-Bedarf), Ziel Topf 1 plus die einmaligen Ausgaben des Monats, keine
   Überschussbuchung; reicht Topf 1 danach nicht, folgt C2. **C2** bei nicht erfülltem Kriterium: Topf 1 aus Topf 2, dann
   Topf 3 so, dass er die Ausgaben deckt und danach (Min(`reserveMonths`, Monate bis Januar) − 1) Monatsbedarfe enthält;
   Topf 2 wird nicht aufgefüllt. Das Ziel von Topf 1 enthält auch in der Januar-Auffüllung und bei A die einmaligen Ausgaben
   des Monats; Überschussbuchungen in Topf 3 gibt es nur in der Januar-Auffüllung. Reichen die Töpfe nicht: Notverkauf einer Immobilie
   (Erlös in Topf 3, dann C erneut); ohne Immobilie darf Topf 1 negativ werden, Topf 2 und 3 sind stillgelegt, alle Zuflüsse
   fliessen in Topf 1, und sobald das Vermögen wieder positiv ist, läuft A. Reihenfolge im Monat: Einnahmen in Topf 1, Zuflüsse
   direkt an ihr Ziel (Topf 3 oder Reservierung; Protokoll «extern → Topf 3», «extern → Reservierung», «Reservierung → Topf 3»),
   Käufe (Eigenmittel aus der Reservierung, Rest als einmalige Ausgabe des Monats vorgemerkt), Januar-Auffüllung bzw. A, C mit
   allen Ausgaben des Monats, dann die Zahlung aus Topf 1. Die Umbuchungen werden nach dem Renditeschritt gebucht; die
   Entscheidungen gelten für den Stand vor den Ausgaben des Monats. Protokoll `bucketRefills` (`trigger` Januar/A/B/C1/C2,
   Bezug, Ausgaben, AHV, PK-Rente, Erwerb, Bedarf, Ziele, `passedMonths`, Aktienrendite, Kriterium/`conditionIgnored`,
   Umbuchungen `moves` (Topf 0 = extern), Stände vor und nach, `payment`).
   **Sofortige Anwendung bei Zuflüssen** gilt für die dynamischen Strategien (Budget neu berechnen); 3-Topf behandelt Zuflüsse
   nach Fall B. Für die dynamischen Strategien wirken sie zusätzlich zum Januar im Monat eines Zuflusses: einmaliger Betrag
   > 0, PK-Kapitalbezug, Verkaufserlös. Reihenfolge im Monat: Zuflüsse und Verkäufe, Käufe, dann die Strategie (einmal pro
   Monat, auch wenn der Monat ein Januar ist). Ist im Folgemonat der Kauf einer anderen Immobilie geplant (fester Monat M+1
   oder «nach Verkauf»), wird der Verkaufserlös reserviert (bleibt liquide, «Standard» legt ihn nicht an) und die Strategie
   erst im Kaufmonat nach dem Kauf angewendet.
   Die neue Jahresentnahme gilt ab dem Zuflussmonat, nicht rückwirkend. Protokoll: Ereignis «Entnahmestrategie angewendet
   nach …» (nur ausserhalb des Januars und nicht beim Start der Strategie) und `propertyEvents[].note` beim Verkauf.

**Hypotheken:** Tranchen mit einer Abfolge von Perioden (fest oder SARON + Marge, SARON mit Untergrenze 0).
Die erste Periode beginnt am Startdatum der Tranche (bzw. beim Kauf), jede weitere am Ende der vorherigen;
danach Verlängerung zu gleichen Konditionen oder Rückzahlung. Zins = Schuld × Satz / 12 pro Monat, aufgelaufen und zusammen mit der Amortisation am Quartalsende (März, Juni, September, Dezember) bezahlt; beginnt die Hypothek im April, ist die erste Zahlung im Juni fällig (April bis Juni). Bei einem Verkauf werden die aufgelaufenen Zinsen sofort, am Planende im letzten Monat bezahlt. Die Lebenshaltung enthält keine Miete und keine Hypothekarzinsen; diese kommen als Wohnkosten hinzu.
Amortisation als fixer Betrag pro Quartal, fällig Ende März/Juni/September/Dezember; direkt (reduziert die Schuld)
oder indirekt (Einzahlung in die Säule 3a). Die 3a-Auszahlung zahlt zuerst Tranchen mit indirekter Amortisation zurück.

**Steuern:** Satz aus der Tabelle linear interpoliert × Steuerfuss, auf Rappen gerundet. Kapitalbezüge im selben
Kalenderjahr (PK, 3a) werden für den Satz zusammengerechnet: für den Haushalt (eine Person, Verheiratete) oder je
Person (unverheiratetes Paar, #150).

## Import alter Konfigurationen

`import-legacy` übernimmt eine `config.json` des früheren Programms als Ausgangspunkt. Das frühere Programm gilt
nicht als Referenz; die Rechenregeln sind in diesem Dokument beschrieben und mit eigenen Referenzfällen getestet.
Immobilien werden nur näherungsweise übernommen (Hypotheken zu gleichen Konditionen verlängert, Amortisation pro
Quartal); der Import meldet das. Mehrere Kapitalbezüge (`LumpsumRatio`) werden zu einem Bezug zusammengefasst.

## Zweite Person, weitere Entnahmestrategien, Pakete

- **Double Income:** `household.persons` mit zwei Personen und `partner` (`income`, `pk`, `ahv`, wie bei der
  Hauptperson). Pensionskasse und AHV laufen pro Person; Steuern, Vermögen und Ausgaben gelten für den Haushalt.
  Als pensioniert gilt der Haushalt (Entnahmestrategien), sobald alle Personen mit Bezugsdatum pensioniert sind.
  Ehepaar-Regeln (`household.married`): beide AHV-Renten zusammen höchstens `ahvCoupleCap` (Standard 3 780 pro
  Monat = 150 % der Maximalrente 2025); die Grenze steigt jeden Januar einmal mit der Teuerung, sobald mindestens
  eine Rente mit Teuerungsanpassung läuft (#145). Optional eigene Steuertabellen `taxes.couple`. Hinterlassenenrenten
  und Todesfälle werden nicht berechnet.
- **Fixe Entnahmerate** (`"strategy": "fixed_rate"`, `"fixedRate": {"rate"}`) und **Guardrails** (`"guardrails":
  {"initialRate", "upper", "lower", "adjust"}`, nach Guyton-Klinger): ab Pensionierung bestimmt die Strategie die
  Lebenshaltung. Entnahme = Lebenshaltung + Wohnkosten − Einkommen und Renten (vor Steuern). Fixe Rate: im Januar
  `rate` × liquides Vermögen. Guardrails: Startentnahme bei Pensionierung, danach im Januar mit der Teuerung
  (nicht nach einem Aktien-Verlustjahr, solange die Rate über der Startrate liegt); Rate über Startrate × (1 +
  `upper`) ⇒ Kürzung um `adjust`, unter Startrate × (1 − `lower`) ⇒ Erhöhung. `withdrawal.floor` / `ceiling`
  begrenzen die Lebenshaltung (Anteil der geplanten Ausgaben, 0 = keine Grenze). Variation `withdrawal.rate`.
- **VPW** (`"strategy": "vpw"`, `"vpw": {"horizonAge", "realReturn"}`) und **Endowment** (`"endowment": {"rate",
  "smoothing"}`): wie fixe Rate bestimmt die Strategie die Lebenshaltung. VPW entnimmt im Januar den Anteil des
  liquiden Vermögens, der es bis `horizonAge` (Alter der Hauptperson) mit der Realrendite gleichmässig aufbraucht
  (Annuitätenfaktor). Endowment (Yale/Tobin): Entnahme = (1 − `smoothing`) × Vorjahr mit Teuerung + `smoothing` ×
  `rate` × Vermögen. Beide kennen `floor` / `ceiling`.
- **Optimieren:** In der App unter «Optimierung» (Ergebnis lässt sich mit einem Klick in die Planung übernehmen; die Einstellungen werden in der Planung unter `optimization` gespeichert und gelten auch für `optimize`, Optionen überschreiben sie), im Terminal mit `optimize <planung>`: sucht für jede Strategie die Parameter, die den Konsum (Median und P10,
  real) maximieren, solange die Ruinwahrscheinlichkeit unter der Grenze bleibt, und prüft sie auf unabhängigen Pfaden.
  Vergleich ohne Suche (#197): `Request.compare` (Knopf «Strategien vergleichen (ohne Optimierung)», CLI `--compare`)
  rechnet jede Strategie mit den Werten der Planung (`optimize::compare_strategy`, `with_params` ohne Parameter) auf den
  Pfaden der Planung wie `run`; Bandbreiten und Pfade der Optimierung gelten dann nicht. `Request.property`
  (`optimization.property`, CLI `--property planned|never-sell|both`) rechnet wie geplant, mit `realestate.neverSell`
  oder beides (`Outcome.neverSell`, `patch.neverSell`); ohne eigene Immobilie nur «wie geplant». `Metrics.endTotal`:
  nominales Endvermögen bei 1, 5, 50, 95 und 99 % (`END_PERCENTS`), gewählt wie in `batch::summarize`.
- **Ergebnisse:** Bandbreite mit 1. und 99. Perzentil und dem monatlich schlechtesten Wert; unter «Einzelne Läufe»
  zusätzlich 1., 99. Perzentil und der Lauf mit dem tiefsten Endvermögen. Einzelläufe zeigen den Verlauf von
  Aktien und Obligationen als Index.
- **Installationspakete:** `.github/workflows/pakete.yml` baut auf GitHub Windows (.msi, .exe), Linux (.deb, .rpm,
  .AppImage) und macOS (.dmg) – von Hand gestartet oder durch ein Tag `v…` (dann als Entwurf unter «Releases»).
  Lokal geht jeweils nur das eigene System (`npx tauri build`). Die Pakete sind nicht signiert.

### Grundstückgewinnsteuer

Unter «Steuern» sind die Regeln der Grundstückgewinnsteuer vollständig editierbar (`taxes.propertyGain`): Stufen (Breite und
Satz), Zu- und Abschläge nach Besitzdauer (`duration`: ab vollen Jahren, Änderung der Steuer), Freigrenze (`exemptBelow`) und
eine feste Ermässigung (`reduction`). Die Besitzdauer folgt aus dem Kaufdatum der Immobilie (`bought`, bei geplanten Käufen
der Kaufmonat). Die Schaltfläche «Vorlage Kanton Zürich» setzt den Tarif § 225 StG (StA eForm. 225.1, Stand 09/2025): Stufen
10 % bis 40 %, Zuschlag 50 % unter 1 Jahr und 25 % unter 2 Jahren, Abzug 5 % ab 5 Jahren plus 3 Prozentpunkte pro Jahr bis
50 % ab 20 Jahren, Gewinne unter 5 000 steuerfrei. Tests (`crates/engine/tests/reference.rs`) vergleichen mit der amtlichen Tabelle.
Für andere Kantone die Tabellen anpassen; ohne Kaufdatum gilt kein Zu- oder Abschlag.

`taxes.propertyGain.model` wählt das Modell: `zurich` (Standard, wie oben) oder `aargau`. Das Modell Aargau kennt keine Stufen:
`durationRates` (ab vollen Jahren, Steuersatz) gibt den **direkten Satz auf den ganzen Gewinn** nach Besitzdauer an (Vorlage
§ 109 StG: 40 % im 1. Besitzjahr, 2 Prozentpunkte weniger je Jahr bis 20 % im 11. Jahr, danach 1 Punkt weniger bis 6 % im
25. Jahr, 5 % ab 25 vollen Jahren); ohne Kaufdatum gilt der höchste Satz. Freigrenze und feste Ermässigung gelten in
beiden Modellen. `useMultiplier` multipliziert die Steuer zusätzlich mit dem Steuerfuss (`taxes.multiplier`, Standard
aus). «Eigene Regeln erfassen» beginnt mit einer Zeile im Modell Aargau.

### Bandbreiten der Optimierung

Für jede Strategie lässt sich pro Parameter festlegen, ob er fix auf dem Wert der Planung bleibt oder in einer Bandbreite
variiert wird (Einheit des Parameters: Jahre, Monate, Prozent, Alter). Beispiel 3-Topf: Topf 1 fix, Topf 2 von 0 bis 2 Jahre,
Nachfüllen ab Vorjahresrendite −30 % bis +10 %. Gespeichert wird das in der Planung:

```json
"optimization": { "ranges": { "three_buckets": {
  "cashYears": { "fixed": true },
  "bondYears": { "lo": 0, "hi": 2 },
  "refillThreshold": { "lo": -0.3, "hi": 0.1 } } } }
```

Ohne Eintrag gelten die Standard-Bandbreiten (`engine::optimize::param_defs`); Untergrenze und Obergrenze der Lebenshaltung
(Guardrails, Fixe Rate) sind ohne Eintrag fix. Der Befehl `opt_params` liefert der Oberfläche die Parameter mit Einheit,
Standard-Bandbreite und aktuellem Wert; `optimize` auf der Kommandozeile liest die Bandbreiten aus der Planung.

### Dauerschätzung

Unter «Optimierung» und «Szenarien & Simulation» steht eine grobe Schätzung der Rechendauer. Der Befehl `bench_plan` misst
die Zeit eines Laufs (einige tausend Läufe, meist unter einer Sekunde) und trennt das Erzeugen der Renditepfade vom Rechnen;
die Oberfläche rechnet damit die Läufe hoch (bei der Optimierung mit den gemessenen Suchschritten je Strategie). Gegenprobe mit
echten Läufen: Abweichungen von 0.8 bis 1.4, deshalb zeigt die Anzeige eine Spanne. Die Dauer der Simulation steht
ausserdem links neben «Simulation starten».

### Fortschritt und Abbruch

Simulation, Szenarien und Optimierung zeigen einen Fortschrittsbalken mit vergangener und geschätzter Restzeit und einen
Knopf «Abbrechen». Jede Berechnung ist ein Job mit eigener Kennung (`jobId`, von der Oberfläche beim Start mitgegeben), eigenem
Zähler und eigenem Abbruch (`engine::progress`, #107). Die Befehle `progress_get` und `cancel_job` nehmen die Kennung; ohne Kennung
liefert `progress_get` die zuletzt gestartete Berechnung und `cancel_job` bricht alle laufenden ab. Der Job hängt am Thread der
Berechnung; ohne Job (zum Beispiel bei der Messung der Rechenzeit) gibt es weder Abbruch noch Zähler (#104). Die Läufe werden in
Blöcken gerechnet, nach jedem Block wird der Abbruch geprüft. Zwei Berechnungen beeinflussen sich nicht.

### Liste der Simulationen

Jede Simulation wird nach dem Start automatisch abgelegt, mit der Planung von damals und dem vollständigen Ergebnis.
Die Schaltfläche «Simulationen» öffnet die Liste: Name und Notiz sind direkt in der Zeile editierbar, «Öffnen» zeigt das
gespeicherte Ergebnis (mit der Planung von damals; «Planung in den Editor übernehmen» lädt sie zurück), «Löschen» und
«Markierte löschen» entfernen Einträge. Zwei bis sechs markierte Einträge lassen sich vergleichen: Kennzahlen nebeneinander,
Verlauf des Gesamtvermögens (Median und 5. Perzentil) und eine Tabelle der geänderten Eingaben. Die Liste liegt lokal im
Speicher der App (IndexedDB) und nicht in der Planungsdatei.

### Hilfe

Die Hilfe öffnet als Panel rechts und ist kontextabhängig: **F1** zeigt das Thema des Felds mit dem Fokus, sonst des
aktuellen Abschnitts bzw. der Ergebnisansicht; der Knopf «? Hilfe», die Zeichen ⓘ neben Feldern, Kartenüberschriften und
Grafiken sowie das Menü *Hilfe* (Handbuch, ⌘⇧?) führen ebenfalls hin. Die Texte liegen als Markdown je Sprache in
`crates/app/ui/hilfe/<sprache>/` (#173, #174, #175; `de/`, `en/`, `fr/`, `it/`) mit denselben Dateinamen und Themenkennungen in jeder Sprache
(ein Thema beginnt mit `## @kennung Titel`, Verweise als `[Text](#kennung)`, die erste Zeile `Stichworte:` bzw.
`Keywords:` enthält Suchbegriffe); neue Felder ordnet man in `crates/app/ui/help.js` zu (`FIELD_TOPICS`,
`HEADING_TOPICS`). Die Hilfe lädt die Kapitel der gewählten Sprache, ein fehlendes Kapitel deutsch; nach einem
Sprachwechsel zeigt eine offene Hilfe dasselbe Thema in der neuen Sprache. `tools/ui-smoke.sh` (`#helpcheck`) prüft,
dass alle Verweise und Zuordnungen auf ein Thema zeigen und jede Sprache dieselben Themen hat. Der Rust-Test
`crates/app/tests/help.rs` prüft je Datei gleiche Themen, Unterüberschriften, Verweise und Stichwortzeilen wie im
Deutschen (`COMPLETE`: Sprachen mit allen Dateien) und dass die in der Hilfe zitierten Bildschirmbegriffe (Englisch:
in “…”, Französisch und Italienisch: in «…») in der Texttabelle der Sprache stehen; andere Zitate (Beispiele, Eingaben) stehen in ‘…’
bzw. ‹…›.

## Test quality

Die Testqualität wird auf mehreren Ebenen gemessen (Details und Schichten: `docs/TESTS.md`):

| Ebene | Werkzeug | Lokal | Report |
|---|---|---|---|
| Ausführung | cargo-nextest (Profil `ci`, Wiederholungen, flaky = Fehler) | `make test` | `junit.xml` |
| Statische Prüfung | clippy mit `-D warnings` (Workspace und `fuzz/`, Job `clippy`) | `make clippy` | Log des Laufs |
| Sprache im Code | `tools/check-english.sh` (Job `english` und `make test-quiet`, Übergangsliste leer seit #168) | `make english` | Datei:Zeile im Log |
| Abdeckung | cargo-llvm-cov (Gate: Zeilen ≥ 80 % je Paket, `tools/coverage-min`) | `make coverage` | HTML, LCOV, Cobertura |
| Wirksamkeit | cargo-mutants (PR und nächtlich: nur geänderte Zeilen seit dem letzten erfolgreichen nächtlichen Lauf; voll: lokal oder manuell in Actions) | `make mutants`, `make mutants-diff` | `local/mutants.out/` (`missed.txt`) |
| Eingabebreite | proptest (`crates/engine/tests/proptest_parsers.rs`, zufällige Planungen in `proptest_plans.rs`), cargo-fuzz (`fuzz/`, ganze Rechnung im Ziel `run_plan`) | `make proptest`, `make fuzz` | Fehlerfälle, Korpora |

Miri und loom sind nicht eingerichtet: der Code enthält kein `unsafe` und nur einfache Atomic-Flags (`progress.rs`).

Workflows: `.github/workflows/test-quality.yml` (Push auf `main`, Pull Requests, nächtlich und von Hand) und `mutation-full.yml` (nur manuell gestartet; ein voller Lauf dauert viele Stunden und passt nicht ins Free-Kontingent privater Repos).
Reports liegen als Actions-Artefakte und in der Job-Summary. Geplante Läufe werden **übersprungen**, wenn es seit dem
letzten erfolgreichen Lauf keinen neuen Commit gibt. Lokal ohne CI: `make quality` (`FORCE=1 make quality` erzwingt
einen Lauf; Reports unter `local/test-quality/reports/`). Nötig: `cargo install cargo-nextest cargo-llvm-cov cargo-mutants`.

## Offen für spätere Etappen

weitere Datenquellen (z. B. Schweizer Aktien und Obligationen), Export der Ergebnisse.
