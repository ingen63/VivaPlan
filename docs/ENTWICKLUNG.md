# VivaPlan – Entwicklung

Technische Beschreibung für Entwicklerinnen und Entwickler: Aufbau des Repos, Bauen, Kommandozeile, Datenbibliothek,
Projektdatei und die Rechenregeln Schritt für Schritt. Was die App kann, steht im [README](../README.md) und in
[BESCHREIBUNG.md](BESCHREIBUNG.md).

## Aufbau

| Ordner | Inhalt |
|---|---|
| `crates/engine/` | Bibliothek: Datenmodell, Simulation, Zeitreihen, Steuern, Prüfung, Import alter Konfigurationen; Tests unter `tests/` |
| `crates/engine/data/` | Datenbibliothek: ausgelieferte Zeitreihen (`Shiller CHF.csv`, `Shiller USD.csv`, `Welt (JST).csv`), die Steuerdatei `steuern-<Jahr>.json` (#63) sowie in `crates/engine/data/quellen/` SNB-Auszüge und die JST-Tabelle mit Auszug (die Shiller-Originaldatei wird nicht mitgeliefert, siehe `QUELLEN.md`) |
| `crates/engine/assets/` | Beispielplanungen (`example.vplan`, Vorlage `new.vplan`) |
| `crates/cli/` | Kommandozeile `vivaplan` (Rechnen, Prüfen, Import, Vergleich, Entwicklungsserver) |
| `crates/app/` | Desktop-App (Tauri 2): Fenster, Dateidialoge, Anbindung an den Rechenkern |
| `crates/app/ui/` | Oberfläche: Editor, Live-Vorschau, Ergebnisansicht (HTML/JS ohne externe Bibliotheken), Hilfe in `hilfe/` |
| `e2e/` | Oberflächentests (Playwright) |
| `fuzz/` | Fuzzing (cargo-fuzz, eigener Workspace) |
| `tools/` | Skripte: Git-Hooks, Build und Installation, App-Icons `make-icons.sh`, Rauchtest `ui-smoke.sh`, Testqualität, Übersichtsseite, Issues, Steuerdatei `steuerdaten/` (#63) |
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
3. `README.md` (deutsch, massgebend) und die Übersetzungen `README.en.md`, `README.fr.md` und `README.it.md`
   nachführen, wenn sich Funktionen, Installation oder Videos geändert haben (#245): dieselbe Sprachleiste,
   dieselben Abschnitte in derselben Reihenfolge, Begriffe nach dem Glossar. Jede Fassung zeigt den Teaser ihrer
   Sprache (`user-attachments`), verlinkt die Videos ihres archive.org-Items (`vivaplan-video-<de|en|fr|it>`) und das
   Bildschirmfoto `docs/bilder/uebersicht.png` bzw. `uebersicht-<en|fr|it>.png` (Planung der Videos, Namen je
   Sprache).
4. Prüfen: `make changelog` bzw. `python3 tools/changelog.py check` (läuft auch in `make test-quiet` und im Job
   `english`). Gemeldet werden abweichende Versionen, Abschnitte oder Issue-Nummern und verbotene Varianten aus dem
   Glossar.
5. Versionsnummern nachführen, committen und pushen (noch kein Tag).
6. Pakete vorab prüfen (#262): `gh workflow run pakete.yml -f plattformen=macos,windows,linux` (ohne `tag`: Stand des
   Zweigs, nur Artefakte), mit `gh run watch` abwarten. Erst nach grünem Lauf taggen; ein Paketfehler fällt sonst erst
   nach dem Tag auf (v1.0.0-beta.4/5, #261).
7. Das Tag `v…` setzen und pushen. `pakete.yml` bricht ab, wenn die Version in
   einer Sprache fehlt, und setzt den Text aus `python3 tools/changelog.py notes v…` als Beschreibung des
   Release-Entwurfs: Englisch offen, Deutsch, Französisch und Italienisch aufklappbar.
   Der Titel ist sprachneutral, nur Produkt und Version (`python3 tools/changelog.py title v…`, z. B. v1.0.0-beta.2 →
   «VivaPlan 1.0.0 Beta 2»).

## Desktop-App

```sh
cd crates/app && npm install   # einmalig: Tauri-Kommandozeile
npx tauri build --config tauri.cli.conf.json   # erzeugt target/release/bundle/macos/VivaPlan.app und .dmg
npx tauri dev                  # App im Entwicklungsmodus
```

Für Windows bzw. Linux wird `npx tauri build` auf dem jeweiligen System ausgeführt.

**Kommandozeile im Paket (#206):** `tauri.cli.conf.json` ergänzt die Konfiguration um die Kommandozeile als Sidecar
(`bundle.externalBin`). Vor dem Bau ruft sie `tools/cli-sidecar.mjs` auf: baut `vivaplan` (release) und legt es als
`crates/app/binaries/vivaplan-<target-triple>` ab (nicht eingecheckt). Das Paket enthält dann `vivaplan` neben
`vivaplan-app` (macOS `VivaPlan.app/Contents/MacOS/`, Windows im Installationsordner, Linux `/usr/bin`). Die Angabe steht
nicht in `tauri.conf.json`, weil `tauri-build` die Datei sonst bei jedem `cargo build`/`cargo test` der App verlangt.
Ohne `--config` entsteht das Paket wie bisher ohne Kommandozeile. Die Kommandozeile bestimmt den Datenordner der App wie
Tauri (`engine::appdata`, Test in der App) und übernimmt die Einstellungen (Grenzen, Tragbarkeit) wie `api.js`.

**Agent Skill im Paket (#208):** `vivaplan-skill/SKILL.md` (offenes Format, englisch; `name` = Ordnername) erklärt KIs die
Kommandozeile. `tauri.cli.conf.json` legt sie als `bundle.resources` ins Paket (macOS `VivaPlan.app/Contents/Resources/`,
Windows im Installationsordner, Linux `/usr/lib/VivaPlan/`). `crates/cli/tests/skill.rs` prüft, dass alle Befehle und
Optionen der Skill in `vivaplan --help` stehen, die genannten `describe`-Abschnitte existieren und die Beispiel-Kennzahlen
rechnen. Neue oder umbenannte Befehle und Optionen deshalb in der Skill nachführen.

**MCP-Server (#210):** `vivaplan mcp` (`crates/cli/src/mcp/`) spricht JSON-RPC 2.0 über stdio, eine Meldung pro Zeile,
von Hand geschrieben ohne SDK und ohne Netzwerkbibliothek (Entscheid E4 in #77). `mod.rs`: Protokoll (`initialize` mit
Aushandlung der Fassung, `ping`, `tools/list`, `tools/call`, Fehler nach JSON-RPC), Obergrenze 20'000 Zeichen je
Werkzeugergebnis (`fit` kürzt die längste Liste und sagt, wie der Rest abzurufen ist), Haftungsausschluss nach
Ergebnissen. `session.rs`: geöffnete Planungen (`p1`, …) und Ergebnisse (`r1`, …) im Speicher; jede Rechnung über
`Session::call` → `engine::dispatch::call` (wie die App), die Einstellungen über `engine::appdata::apply`, Datenordner
über `app_data()` wie die übrige Kommandozeile. `tools.rs`: Werkzeuge (nur lesend: `plan_open`, `plan_get`,
`plan_schema`, `plan_validate`, `describe`, `result_list`, `result_get`, `library_list`) mit Schemas und
Kennzeichen `readOnlyHint`; Fehler der Eingaben kommen als Werkzeugergebnis mit `isError`, damit die KI sie beheben
kann. Die Daten stehen als kompaktes JSON im Text, ohne `structuredContent` (viele Hosts geben beides ans Modell).
`privacy.rs`: Namen und freie Bezeichnungen (`name`, `label`) werden durch neutrale ersetzt, ausser mit `--names`;
Ordner fallen weg. Die Werkzeugliste ist in `crates/cli/tests/mcp-tools.json` festgehalten; eine Änderung der
Schnittstelle lässt `crates/cli/tests/mcp.rs` fehlschlagen, die neue Liste steht in der Meldung. Ändern und Speichern:
#212.

**Rechnende Werkzeuge im MCP-Server (#211):** `compute.rs`: `simulation_run` (Schätzung wie die App: `bench_plan` ×
Pfade × Kombinationen der Variationen, dann `run_simulation` über `dispatch::call`; das Ergebnis liegt als `RunOut` in
der Sitzung) und `evaluate` (`experiment::evaluate_as` mit Kennung, Schätzung und `confirm` aus #216; Ergebnis in der
Sitzung, `result_get` nur Sicht `summary`). Ab `experiment::CONFIRM_SECONDS` ohne `"confirm": true` Antwort
`needsConfirmation` mit Schätzung. `mod.rs`: Der Hauptthread liest weiter, ein Thread pro Rechnung (`work`) rechnet in
einem weiteren Thread und schickt etwa jede Sekunde `notifications/progress` (nur mit `_meta.progressToken`, aus
`progress_get`); `notifications/cancelled` mit der Kennung der Anfrage ruft `cancel_job` (bei jedem Takt erneut, falls
die Rechnung noch nicht gestartet war) und unterdrückt die Antwort. Eine Rechnung aufs Mal (Slot `Running`); ein
zweiter Rechenaufruf ist ein Werkzeugfehler. Ende der Eingabe bricht ab und wartet auf den Thread. Ausgaben beider
Threads gehen zeilenweise über `Output` (Mutex). Die Kennungen der Rechnungen beginnen bei 2^40.

**Ändern und Speichern im MCP-Server (#212):** `edit.rs`: `plan_set`, `plan_apply` über `api::set_fields` (Feldpfade
aus #207, alles oder nichts), `plan_changes`/`plan_undo` über `api::diff_plans`. Jede `OpenPlan` führt `original`
(Stand der Datei, nach `upgrade_plan`), `history` (Arbeitskopie vor jedem Schritt) und `stamp` (Änderungszeit und
Inhalt der Datei beim Öffnen bzw. letzten Schreiben, Regel 4). `evaluate`-Ergebnisse behalten das `set` des
Experiments für `plan_apply`. `plan_save` prüft die Regeln 1–6 (Konzept, Abschnitt 7) in `prepare_save`; ersetzt es
eine vorhandene Datei, schickt `mod.rs` statt der Antwort eine Anfrage `elicitation/create` (nur wenn das Programm in
`initialize` die Fähigkeit `elicitation` meldet) und merkt sich den Schreibauftrag mit dem Stand der Arbeitskopie;
die Antwort des Programms (`accept` mit `replace: true`) löst `edit::write` aus (Regel 4 erneut, `.bak`, dann
`api::write_plan` über eine Zwischendatei), danach geht die Antwort auf den ursprünglichen Aufruf hinaus.

**Ein-Klick-Paket und «Mit KI verbinden» (#213):** `tools/mcpb.py build --binary <vivaplan> --out <ordner>` schreibt
`VivaPlan_<Version>_<Architektur>.mcpb`, ein ZIP mit `manifest.json` (MCPB, `manifest_version` 0.2, `server.type`
`binary`, `mcp_config` `${__dirname}/server/vivaplan mcp --allow-dir ${user_config.plan_folder}
--read-only=${user_config.read_only}`, `user_config` Planordner und «nur lesen», `compatibility.platforms` `darwin`), dem Programm unter
`server/vivaplan` (Unix-Rechte 755 im ZIP) und `icon.png`. Die Werkzeugliste des Manifests fragt das Skript beim
Server selbst ab (`tools/list`, erster Satz der Beschreibung). `tools/mcpb.py check <paket>` prüft das Manifest, packt
aus und startet den Server wie Claude Desktop; `pakete.yml` baut und prüft das Paket unter macOS, Tests in
`tools/test_mcpb.py` (in `make test-quiet`). Nur Python-Standardbibliothek. Der Dialog (`showConnectAi` in `app.js`)
nimmt den Pfad des Programms aus `app_info.cliPath` (`cli_next_to`: das Programm neben `current_exe`, nicht bei
AppImage und App Translocation), sonst den Standardpfad des Systems; kopiert wird mit `navigator.clipboard`.
`notifications/cancelled` verwirft den Auftrag. `--read-only` nimmt `plan_save` aus `tools/list` (`tools::listed`);
`--allow-dir` (mehrfach) gibt Ordner kanonisch frei. Verknüpfungen (Symlinks) als Ziel werden abgelehnt. #263: Mit
mindestens einem `--allow-dir` liest `plan_open` nur Dateien, deren kanonischer Pfad in einem freigegebenen Ordner liegt
(`Session::readable`), und `plan_save` schreibt auch in Unterordner; `~/` am Anfang steht für den Benutzerordner. Das
Paket übergibt den Schalter «nur lesen» als `--read-only=true|false`, weil das Format ein Argument nicht weglassen kann
(`${user_config.read_only}` wird ersetzt). `tools/mcpb.py check` startet den Server mit einem temporären Planordner:
speichert und öffnet dort eine Planung, erwartet die Ablehnung einer Planung ausserhalb und mit «nur lesen» kein
`plan_save`. Der Dialog nimmt den Planordner aus `app_info.planDir` (`plan_dir`) und wählt ihn mit
`pick_plan_folder`.

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

**Externe Links (#237):** Die feste Adressliste steht an einer Stelle, `crates/app/ui/links.json` (Kennung → Adresse,
nur `https://`). Die App bindet sie mit `include_str!` ein (`crates/app/src/links.rs`); der Befehl `open_link` nimmt
nur eine Kennung der Liste entgegen und öffnet die Adresse mit `tauri_plugin_opener::open_url` im Standardbrowser.
Das Plugin wird nicht registriert, die Oberfläche hat also keinen Zugriff auf seine JavaScript-Schnittstelle und die
Berechtigungen (`capabilities/default.json`) bleiben bei `core:default`. Die Oberfläche ruft `openLink(kennung)` auf;
im Browser (`ui-dev`) öffnet sie die Adresse aus derselben Liste in einem neuen Tab. Eine neue Adresse ist eine Zeile
in `links.json`. Einführungsvideos (#240): je Länge und Sprache eine Kennung `video-<2min|10min>-<de|en|fr|it>`;
die Oberfläche wählt die Sprache der Einstellung (`data-action="video"`, in der Hilfe ein Link mit dem Ziel `video:2min`).

**Ergebnisse:** Registerkarten Übersicht, Grafiken, Einzelne Läufe (5./50./95. Perzentil), Szenarien, Jahreswerte;
alle Beträge wahlweise nominal oder in heutiger Kaufkraft (Preise des Simulationsstarts); beim Öffnen heutige
Kaufkraft (#265). Die Läufe der Perzentile folgen der Darstellung: `batch::summarize` liefert `percentiles` (nach dem
nominalen Endvermögen geordnet) und `percentilesReal` (nach dem realen), `BatchOut` dazu `details` und `detailsReal`
mit den Detailläufen je Ordnung (ein Lauf in beiden Ordnungen wird einmal gerechnet). Die Oberfläche wählt die Ordnung
nach dem Umschalter (`pctsOf`, `batchDetails`); Ergebnisse vor #265 ohne reale Ordnung zeigen in heutiger Kaufkraft das
reale Endvermögen der nominal geordneten Läufe mit einem Hinweis. Grafiken eines Laufs:
Vermögensverlauf, liquides Vermögen nach Anlage (Beträge oder Anteile), Einnahmen und Ausgaben pro Jahr,
Entnahmerate, Immobilien (Wert, Hypothek, Eigenkapital), Vorsorge. Bei mehreren Läufen zusätzlich: Bandbreite,
Wahrscheinlichkeit, dass das Geld reicht (über die Zeit), Verteilung von Endvermögen und Vermögen bei Pensionierung,
Alter bei Erschöpfung. Szenarien: Erfolgsquote und Endvermögen je Variante (eine Variation) bzw. Heatmaps (zwei).

## Beta-Token (#270)

Während der geschlossenen Beta brauchen die Desktop-App und `vivaplan mcp` eine einmalige Aktivierung. Die übrigen
Befehle der Kommandozeile und der Entwicklungsserver bleiben frei. Das Token ist eine Hürde, kein Schutz (GPL).

- **Rechenkern** `crates/engine/src/beta.rs`: Format `VP1.beta.<Kennung>.<Kürzel>.<JJJJ-MM-TT>.<Signatur>`; signiert ist
  alles vor dem letzten Punkt (Ed25519, `ed25519-dalek`; Signatur base64url ohne Auffüllung). Kennung und Kürzel: 1 bis
  16 Buchstaben, Ziffern oder Bindestriche. `redeem` prüft Format, Signatur und Einlösefrist (einschliesslich, UTC),
  `activate` schreibt `beta-aktivierung.json` (Token und Tag) in den Datenordner der App, `Status::of` liest sie und
  prüft nur die Signatur (die Frist zählt nach der Aktivierung nicht mehr). Leerzeichen und Zeilenumbrüche im Token
  werden übergangen. Meldungen: Codes `beta_*` in `message.rs`.
- **App:** Befehle `beta_status` und `beta_activate` (nur Desktop-App, `ipc-commands.json`). `app.js` (`betaGate`) zeigt
  vor allem anderen den Dialog `#beta`; solange er offen ist, laden weder Startplanung noch Menü, Tastenkürzel oder
  «Datei öffnen» etwas. Der Rauchtest (`--smoke`) überspringt ihn. «Über VivaPlan» zeigt Kürzel und Tag.
- **MCP:** `vivaplan mcp` prüft beim Start die Aktivierung im Datenordner (`--data-dir`, sonst wie die App); fehlt sie,
  löst es `--beta-token=<token>` ein (Feld «Beta token» des `.mcpb`, `user_config.beta_token`) und speichert die
  Aktivierung für App und Server. Ein leeres Feld oder der nicht ersetzte Platzhalter `${user_config.beta_token}` zählt
  als kein Token. Sonst beantwortet der Server jeden `tools/call` mit dem Grund (`isError`); `initialize`,
  `tools/list` und `ping` gehen weiter.
- **Entwicklungsserver:** braucht keine Aktivierung (`beta_status`: `required: false`). Der Testzugang `#beta` der
  Oberfläche verhält sich wie eine App ohne Aktivierung; `beta_activate` prüft dann mit dem Testschlüssel und speichert
  nichts (e2e `ui-beta.spec.js`).
- **Testschlüssel:** `beta::TEST_SEED`; Tests und Entwicklungsserver verwenden nur ihn. Ein Debug-Build prüft mit
  gesetzter Umgebungsvariable `VIVAPLAN_BETA_TEST_KEY=1` gegen den Testschlüssel (Tests von `vivaplan mcp`,
  `tools/test_mcpb.py`); ein Release-Build beachtet sie nicht.
- **Abschalten ab 1.0.0:** Cargo-Feature `beta-token` des Rechenkerns, standardmässig an. Ohne es (aus `default` in
  `crates/engine/Cargo.toml` entfernen) braucht nichts eine Aktivierung; danach Dialog, Befehle und Feld entfernen.

**Schlüssel erzeugen und einsetzen (einmal):** Das Werkzeug `beta-token` (`crates/cli/src/beta_token.rs`) ist ein
eigenes Programm für Entwickler und kommt nicht in die Pakete (`tools/cli-sidecar.mjs` kopiert nur `vivaplan`).

```sh
cargo run -q -p vivaplan-cli --bin beta-token -- keygen --out ~/VivaPlan-Beta/beta-privat.key
```

`keygen` schreibt den privaten Schlüssel (nur für den Besitzer lesbar) und gibt den öffentlichen aus. Es verweigert
eine vorhandene Datei und jeden Ort im Repository; `.gitignore` schliesst `*.key`, `*.pem` und `*private*key*` aus. Den
privaten Schlüssel sicher aufbewahren (z. B. Passwortverwaltung), nie weitergeben, nie einchecken. Den öffentlichen
Schlüssel (64 Hexadezimalzeichen) in `crates/engine/src/beta.rs` bei `PUBLIC_KEY` anstelle des Platzhalters einsetzen,
committen und eine neue Version bauen. Solange der Platzhalter dort steht, nimmt die App kein Token an (Meldung
`beta_no_key`). Ein neuer Schlüssel macht alle bisherigen Tokens und Aktivierungen ungültig.

**Tokens ausgeben:**

```sh
cargo run -q -p vivaplan-cli --bin beta-token -- issue --key ~/VivaPlan-Beta/beta-privat.key --who AB --until 2026-12-31
cargo run -q -p vivaplan-cli --bin beta-token -- check VP1.beta.…   # prüft gegen den eingebauten Schlüssel
```

`--who` ist das Kürzel der Testperson (kein voller Name), `--until` die Einlösefrist; ohne `--id` wählt das Werkzeug
eine zufällige Kennung. Das Token als eine Zeile verschicken. `tools/mcpb.py check <paket> --beta-token <token>` prüft
ein Paket samt Aktivierung; ohne Token prüft es nur, dass der Server ohne Aktivierung jeden Aufruf ablehnt.

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
$B run crates/engine/assets/example.vplan --metric '{"quantity": "total", "real": true, "reduce": "atEnd", "aggregate": {"kind": "quantile", "q": 0.05}}'   # eigene Kennzahl (#214)
$B evaluate crates/engine/assets/example.vplan experiment.json --json   # Experiment (#216), ab 60 s nur mit --yes
$B describe [fields|series|state|metrics|experiment|examples|performance|limits] [--json]   # Katalog für KIs (#219)
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

**Pfade für mehrere Varianten (#215):** Variationen und Optimierung rechnen viele Varianten auf denselben Pfaden; das
Erzeugen eines parametrischen Pfades kostet mehr als eine Simulation. Deshalb entstehen die Monte-Carlo-Pfade dort nur
einmal und werden für alle Varianten verwendet:

- Variationen (`api::run_scenarios`): `batch::PathCache` hält die Pfade der letzten Kombination. Hat die nächste
  Kombination dieselben Pfad-Eingaben (`PathKey`: Modell und Parameter samt `seed`, Reihe, fixe Kurven, Start, Monate),
  rechnet sie auf diesen Pfaden; sonst entstehen neue. Der Hauptlauf der Planung verwendet sie danach ebenfalls. Ohne
  Variationen bleibt es beim Erzeugen im Lauf.
- Optimierung (`optimize::run_request`): `batch::prepare` erzeugt Such- und Prüfpfade (bzw. beim Vergleich die Pfade der
  Planung) einmal für alle Bewertungen einer Suche.
- Obergrenze: `batch::PATH_CACHE_BYTES` = **512 MB** für gehaltene Pfade (je Pfad `(4 × Monate + 12) × 8` Bytes plus
  Verwaltung, `batch::path_bytes`; bei 40 Jahren rund 15 KB, also gut 30'000 Pfade). Bei der Optimierung teilen sich Such-
  und Prüfpfade die Grenze (Suchpfade zuerst). Darüber werden die Pfade wie vor #215 im Lauf erzeugt: langsamer, aber ohne
  zusätzlichen Speicher.
- Die Ergebnisse sind bitgleich, weil Pfad *i* nur von seinen Eingaben abhängt (Tests `issue_215_*`).
- Messprogramm: `cargo run --release -p vivaplan-engine --example variants_bench [-- <Pfade> <Wiederholungen>]` rechnet
  auf einem Kern 8 Varianten der Musterplanung auf parametrischen Pfaden (Variation des Ausgabenfaktors in 8 Schritten
  und Strategievergleich mit 8 Bewertungen) und gibt Zeit und einen Fingerabdruck der Ergebnisse aus. Gemessen (Apple
  M1 Pro, 1000 Pfade, Release, bestes von 5, vorher/nachher abwechselnd): Variationen 2143 → 1038 ms (48 %),
  Strategievergleich 2036 → 1082 ms (53 %; dort rechnet jede Bewertung die Monatswerte mit, die Simulation wiegt mehr);
  Fingerabdrücke unverändert.

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
  dieselben Pfade. Die Detailansichten (5./50./95. Perzentil) rechnen den betreffenden Pfad exakt nach (in heutiger
  Kaufkraft den Pfad der realen Ordnung, #265).
- «Alle historischen Startzeitpunkte» unterschätzt die Streuung: 864 Startmonate aus 104 Jahren überlappen stark
  (nur gut drei unabhängige 32-Jahres-Abschnitte). Der Bootstrap zeigt deshalb breitere Bandbreiten.

## Szenarien

`scenarios.sweeps` = Liste von `{"param", "from", "to", "step"}`. Parameter: `wealth.stockShare`, `pension.pk.lumpsum`,
`returns.startDate` (Monat, Schritt in Monaten), `milestone.retire` (Monat; setzt die Pensionierung als Datum),
`flows.spendingFactor` (Faktor auf die Lebenshaltungskosten ohne Wohnanteil). Jede Kombination (technisch höchstens 1'000'000) wird mit allen Pfaden
gerechnet: Erfolgsquote, Vermögen bei Pensionierung (Median), Endvermögen 5./50./95. Perzentil, früheste Erschöpfung.
Der reale Median `p50Real` ist das reale Endvermögen des 50. Perzentils der realen Ordnung; `percentilesReal` nennt
die Läufe dieser Ordnung mit Pfadnummer, damit «Stufe wählen» in heutiger Kaufkraft die Läufe der realen Ordnung
nachrechnet (#265).

## Messebene: Kennzahlen aus Bausteinen (#214)

`engine::measure`: Eine Kennzahl (`Metric`, JSON) ist Grösse (`Quantity`: Feld des `MonthRecord` oder abgeleitet:
`consumption`, `plannedConsumption`, `depleted`, `atFloor`, Zähler aus `Summary`) × `real` × Zeitfenster (`Window`:
`from`/`to` als `start`, `retirement`, `end`, Meilenstein oder Monat, `fromAge`/`toAge`, `years`) × Reduktion je Pfad
(`Reduce`) × Aggregation über Pfade (`Aggregate`, mit `kind`). Kein Ausdrucksparser; das JSON-Schema (`measure::schema`)
beschreibt alle Bausteine samt Doku-Kommentaren (für `describe`, #219).

- **Im Strom:** `Simulator::run_observed` übergibt jeden Monat einem `MonthObserver` (Datensatz und `Summary` nach dem
  Monat), auch ohne `record`. `Measurement` legt gleiche Teile je Pfad (Grösse, Einheit, Fenster, Reduktion,
  Bedingung) zu einem Sammler zusammen und aggregiert danach. `annualSums` sind Blöcke zu 12 Monaten ab Fensterbeginn,
  ein unvollständiges letztes Jahr fällt weg; die Aggregation nimmt alle Jahreswerte aller Pfade zusammen.
- **Quantil:** Wert an Rang round((n − 1) · q) der sortierten Werte, wie `run` und die Optimierung; dazu der Pfad.
- **Statistik:** Wahrscheinlichkeiten mit Standardfehler √(p(1 − p)/n), Mittel je Pfad mit s/√n.
- **Fehler:** `Measurement::new` und `measure::parse` geben alle Fehler als Liste zurück (`metric_invalid` mit dem
  Problem als verschachtelter Meldung); ein leeres Zeitfenster ist ein Fehler, ausser in
  `Measurement::allowing_empty` (feste Kennzahlen der Optimierung).
- **Bestehende Kennzahlen:** `optimize::evaluate` rechnet mit `optimize::evaluate_metrics` über die Messebene (vorher
  Monatsaufzeichnung und `path_stats`); `batch::summarize` lässt sich ausdrücken (Test `measure::issue_214_ak3_…`). Beide
  ergeben bitgleich dieselben Werte wie vorher.
- **Kommandozeile:** `run --metric <json>` (mehrfach, auch Liste) über `api::measure`, im JSON unter `metrics`.

## Experiment (#216)

`engine::experiment`: Ein `Experiment` (JSON, Schema unter `definitions.Experiment` in `schema_json()`) hat `set`,
`vary` (`Variation`: `values`, `range` [von, bis, Schritt], `follows`/`offset` oder `group` aus `GroupMember`),
`metrics` (Messebene), `constraints`, `objective` (`maximize`, `minimize`, `weighted`, `score`), `search` (`grid`,
`pattern`; `samples`, `seed`), `budget` (`paths` 1'000, `margin` 2, `top` 5, `confirmPaths` 10'000) und `confirm`.

- **Prüfung:** `experiment::parse` prüft erst gegen das eigene Schema (unbekannte Schlüssel, Typen, feste Werte, mit
  Pfad wie `vary[0].values[1]` und gültigen Werten), dann den Inhalt: jeder Wert per `fieldpath::set` gegen die Planung,
  Bereiche, Kopplungen, Namen der Kennzahlen, genau eine Zielform, Budget; zuletzt jede Variante des Rasters (bei der
  Mustersuche die Ecken der Bereiche) als Planung und Simulator. Fehler → `status: invalid`, nichts gerechnet.
- **Raum:** `expand` bildet Zweige (feste Werte, Bereiche, Kopplungen); Variationen werden kartesisch kombiniert,
  Gruppenmitglieder als Alternativen. Beim Raster werden Bereiche zu Werten (auf 12 Stellen gerundet), höchstens
  `MAX_VARIANTS` = 10'000. Gekoppelte Werte folgen nach den übrigen Werten der Variante.
- **Zwei Stufen:** Suche auf `optimize::with_paths(base, paths, 0)`, Bestätigung der besten `top` auf
  `with_paths(base, confirmPaths, CHECK_SEED_OFFSET)` (anderer Startwert, wie die Optimierung). Alle Varianten nutzen
  dieselben, einmal erzeugten Pfade (`batch::prepare`, #215); ändert eine Variante, wovon die Pfade abhängen
  (`PathKey`, Renditequelle), bekommt sie eigene. Nebenbedingungen in der Suche mit Marge `margin` × Standardfehler (bei
  Wahrscheinlichkeiten an der Grenze √(p(1 − p)/n), wie `optimize::fit_objective`), in der Bestätigung ohne.
- **Mustersuche:** `optimize::search` mit denselben Startpunkten wie die Optimierung (Wert der Planung, Mitte,
  `samples` Zufallspunkte mit `Rng(seed ^ START_SALT)`), je Zweig. Mit `score` rechnet jede Variante zusätzlich
  `optimize::evaluate` und nutzt `Metrics::fitness`; so ergibt sie dieselben Zahlen wie die Optimierung (Test
  `experiment::issue_216_ak8_…`).
- **Dauer:** Läufe = Varianten × Suchpfade + `top` × Bestätigungspfade (× Durchgänge: Kennzahlen, `score`); Sekunden aus
  `api::bench` (ohne Pfaderzeugung, die einmal je Pfad gezählt wird). Ab `CONFIRM_SECONDS` = 60 ohne `confirm`:
  `status: needsConfirmation` mit `estimate`. Höchstens `optimize::MAX_RUNS`. Fortschritt und Abbruch über `progress`.
- **Ergebnis:** `ranking` (je `settings`, `search`, `confirm`), ohne Ziel `listed` (höchstens 50), `hints`
  (`experiment_edge_low/high`, `experiment_confirm_same_paths`), `paths` (Anzahl, Startwerte, `independent`), das
  vollständige `experiment` und `version`. `api::evaluate` und `vivaplan evaluate` (`--json`, `--yes`); der MCP-Server
  (#210–#212) soll `evaluate` als Hauptwerkzeug nutzen.

## Katalog für KIs (#219)

`engine::describe` erzeugt den Katalog (`api::describe(section, plan, base_dir, library)` → `Catalog`, Format
`vivaplan-describe/1`); `vivaplan describe` (`crates/cli/src/describe.rs`) gibt ihn als Text oder JSON aus. Die
Ausgabe ist vom Rechenkern getrennt, damit der MCP-Server (#210) denselben `Catalog` als Werkzeug liefern kann.

- **Felder:** `schema_leaves()` läuft über `schema_for!(Plan)`: Objekte werden aufgelöst, Listen von Datensätzen als `*`,
  ein zeitabhängiger Wert (`Series`) bzw. eine Kennlinie (`StateSeries`) und Tabellen (Listen von Listen) sind je ein
  Blatt. Typ, erlaubte Werte und Bedeutung kommen aus dem Schema (Titel + Beschreibung, gekürzt mit `short`: erster
  Satz, ohne Issue-Nummern, höchstens `MEANING_CHARS`). Einheit und Bereich stehen in der Tabelle `UNITS`, fehlende
  oder unklare Bedeutungen in `MEANINGS`. Test `describe::issue_219_ak2_…`: beide Tabellen nennen nur Felder des
  Schemas, jedes Feld vom Typ number, integer oder series hat eine Einheit, jedes Feld eine Bedeutung; zusätzlich liegen
  alle Werte der Beispielplanung auf Feldern des Katalogs. **Neues Feld im Modell → Eintrag in `UNITS` (Zahlen) bzw.
  Doc-Kommentar oder `MEANINGS`.**
- **Übrige Abschnitte** aus den Schemas von `measure::Metric` (Grössen, Zeitfenster, Reduktion, Aggregation),
  `experiment::Experiment` (Schlüssel, Ziel, Suchverfahren, Standardwerte) und `StateInput` (Zustandsgrössen); die
  Beispiele (`describe::examples`) laufen im Test auf der Beispielplanung; `performance` misst mit `api::bench` beim
  Aufruf (Beispielplanung mit parametrischen Renditen, sonst `--plan`); `LIMITS` nach Analyse Abschnitt 9.
- **Grösse:** ganzer Katalog und jeder Abschnitt, Text und JSON, unter 20'000 Zeichen (`issue_219_ak4_…`). Die Felder
  stehen in der Übersicht ohne Bedeutung, mit Bedeutung nur in `describe fields`; Zeilen als `name: type, unit, range
  – meaning`, Namenslisten als JSON-Objekt.

## Jahreswerte, Kurzfassung und Vergleich (#209)

`engine::digest` wertet Ergebnisse aus, wie die Oberfläche sie hält und speichert (JSON von `RunOut` und `SimResult`;
Felder, die in Ergebnissen älterer Versionen fehlen, gelten als fehlend oder 0). `MonthRecord`, `Summary`, `TaxBasis` und
`WealthBasis` sind dafür lesbar (`Deserialize`, `serde(default)`). Die Beträge kommen aus den Bausteinen der Messebene:
`Quantity::value` (Grösse, nominal oder real) und `measure::in_prices`.

- **`api::yearly(run)`** (Befehl `yearly`): eine Zeile je Kalenderjahr, nominal und real: Bestände am Jahresende,
  Ausgaben, Entnahme und Einnahmen als Monatsmittel, Entnahme des Jahres und Entnahmequote (gegenüber dem
  Gesamtvermögen zu Jahresbeginn; `null` ohne Entnahme oder ohne Vermögen), Inflation, Steuern als Summe mit
  Veranlagung und Wohnort, dazu `flows` (Jahressummen für die Balkengrafiken und die Entnahmequote am liquiden Vermögen).
  Die Reihenfolge der Additionen ist die der früheren Rechnung in `app.js`, damit die Anzeige bis zur letzten Stelle
  gleich bleibt; einzige Ausnahme ist «-0», das als «0» erscheint (JSON kennt keine negative Null).
- **`api::digest(result)`** (Befehl `digest`): Kennzahlen und Befund. Der Befund ist eine Kategorie, kein Satz:
  `verdict` `lasts`/`depleted` (ein Lauf) oder `high`/`medium`/`low` (Erfolgsquote, gerundet auf 0,1 %: ab 95,0 % hoch,
  ab 80,0 % mittel, #72), `next` `scenarios` oder `optimize`, dazu Erfolgsquote, (früheste) Erschöpfung und Endvermögen je
  Perzentil: `nominal` aus der nominalen, `real` aus der realen Ordnung (#265; `realOrder` sagt, welche gilt: bei
  Ergebnissen vor #265 `false`, real dann vom nominal geordneten Lauf, vor #126 aus den Detailläufen). Die Oberfläche formuliert den Satz in
  ihrer Sprache (`verdictHtml`), Kommandozeile und MCP-Server (#210) englisch.
- **`api::compare(items)`** (Befehl `compare_results`, `[{result, plan}]`): je Ergebnis Art, Kurzfassung, Erschöpfung
  (Monat, `null` = in keinem Lauf, fehlt ohne Lauf) und Anteil der Läufe je Alter bei Erschöpfung; dazu die Perzentile
  und Alter aller Ergebnisse und die Unterschiede der Planungen gegenüber der ersten (`fieldpath::diff`).
- **Oberfläche:** `derive` holt Jahreswerte und Kurzfassung einmal je Lauf und Ergebnis (WeakMap `DERIVED`), bevor
  gezeichnet wird; gezeichnet wird weiter synchron. Ein Ergebnis, das noch keine hat, zeichnet `renderResults` nach dem
  Eintreffen neu. Der Vergleich schickt die Ergebnisse ohne Monatswerte (`slimResult`) und behält die Antwort, bis sich
  die Auswahl oder die Liste ändert (`Runs.version`). `api::run` bleibt unverändert; die Golden-Dateien ändern nicht.

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
`{"mode": "timed", "rows": [{"from": "start", "value": 9500}, {"from": "retire", "value": 8500}]}`, dazu (#217)
`linear` (`start`, `end`), `table` (`points`, `interpolation` `linear` oder `step`) und `factor` (`base` mal Verlauf
`factor`). Ein Stützpunkt (`SeriesPoint`) hat `at` (Datum oder Meilenstein, nicht `never`/`afterSale`) oder `age` (mit
`person`, sonst die erste Person). `Curve::compile` übersetzt alle Formen vor der Simulation in eine Treppe über
Kalendermonate (eine Gerade Monat für Monat, höchstens 2400 Stufen); zur Laufzeit kostet keine Form mehr als eine Treppe.
`wealth.stockShare` ist ein `StateSeries` (#218): ein Verlauf oder eine Kennlinie
`{"mode": "curve", "input": <Zustandsgrösse>, "points": [[x, y], …], "eval": "yearly"|"monthly"}`. Die Kennlinie
(`state::StateLine`) wertet `Run::stock_share` beim Umschichten aus; den Zustand (Entnahmen der letzten 12 Monate,
Höchststand des Aktienindex, Wert bis zur nächsten Auswertung) hält `state::Tracker`, nur wenn die Planung eine
Kennlinie hat. Für `fundedRatio` summiert `Simulator::need_rest` Lebenshaltung minus Erwerbseinkommen vorab von hinten;
die Renten kommen bei der Auswertung abschnittsweise (Bezug PK, Bezug AHV, Referenzalter) dazu. Ohne Kennlinie bleiben
Ergebnisse und Rechenzeit gleich (Messung mit `variants_bench`).
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
   **Ende des Erwerbseinkommens (#236):** `flows.incomeEndsAtRetirement` bzw. `partner.incomeEndsAtRetirement`
   (fehlt: `false`, ältere Planungen und Golden-Dateien unverändert; `api::blank_plan` und eine neue zweite Person: `true`).
   Eingeschaltet ist das Erwerbseinkommen der Person ab dem Monat ihres Meilensteins `retire` bzw. `retire2` 0
   (`PersonCfg::income_end`, `Simulator::person_earned`), unabhängig von `pension.pk.at`. Alles, was das Erwerbseinkommen
   nutzt (Geldflüsse, Tragbarkeit, Berufskosten, Variante `working`, Zweitverdienerabzug, grosse Säule 3a, Netto-Bedarf),
   geht über `person_earned`. Die Warnung `income_after_pk` entfällt dann für diese Person.
   **Wohnanteil (#53, #264):** Mit `flows.spendingIncludesHousing` zieht `Simulator::planned_living` von jedem Wert
   der Lebenshaltung den Wohnanteil des Startmonats ab (`housing::start_housing`): Miete, Nebenkosten und Hypothekarzins
   (Schuld × Satz / 12, SARON aus dem festen Wert zum Start), **ohne Amortisation** (direkt und indirekt); diese bildet
   Vermögen und fliesst weiter als Schuldabbau bzw. Einzahlung in die Säule 3a in die Wohnkosten der Simulation. Die
   Prüfung `spending_below_housing` verwendet denselben Wert. Planungen ohne die Marke `flows.amortExcluded` zogen die
   Amortisation (Quartalsbetrag / 3, `housing::start_amortization`) mit ab: `upgrade.rs` senkt bei ihnen jeden Wert der
   Lebenshaltung um diesen Betrag (Hinweis `note_amort_excluded`), für alle Verlaufsformen (`map_series`: Werte
   `x − a`; beim Faktorverlauf Basis `b − a` und Faktoren `(b·f − a) / (b − a)`), sodass das Ergebnis gleich bleibt;
   die 0 vor der ersten Zeile eines Stufenverlaufs bleibt. Ohne `spendingIncludesHousing` setzt es nur die Marke.
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
   Planende noch offene Steuer wird im letzten Monat abgezogen. **Steuerwert (#229):** Für die Vermögenssteuer zählt
   je Immobilie (Besitz, Kauf geplant) der Steuerwert minus Hypothek statt Marktwert minus Hypothek. Feld
   `taxValue: {"kind": "amount"|"rate", "value": …}`: ein Betrag bezogen auf den Marktwert `worth` (geplant: `price`)
   oder ein Anteil (0.65 = 65 %); daraus wird beim Übersetzen ein fester Anteil am Marktwert, der Steuerwert folgt
   also dem Marktwert mit der Teuerung. Ohne `taxValue` ist der Anteil 1 (bisheriges Verhalten, Golden unverändert);
   bei Mieten wird das Feld ignoriert. Prüfung: Betrag < 0 oder Anteil ausserhalb 0–1 ist ein Fehler am Feld
   `taxValue.value`. Hat eine Immobilie einen Steuerwert, wird die Steuerzerlegung (`taxBasis`) aufgezeichnet, mit
   `propertyTaxValue` (Summe der Steuerwerte am 31. Dezember). Vermögen, Ergebnisse, Grundstückgewinnsteuer,
   Tragbarkeit und Notverkauf rechnen mit dem Marktwert.
7. **Notverkauf prüfen:** wenn liquides Vermögen < Schwelle × (Ausgaben − Einkommen); ausgeführt im Folgemonat.
   Ist danach (oder nach einem geplanten Verkauf) keine Immobilie mehr im Besitz, wird im Folgemonat die nächste
   Immobilie mit Kauf `"afterSale"` gekauft, sofern tragbar (Tragbarkeit wie bei den Banken, Hypothek allenfalls
   gekürzt) und die Eigenmittel reichen; sonst beginnen die Mieten mit `"from": "afterSale"`.
   Mit `"buy": "afterSale:<id>"` wird im Monat nach dem Verkauf einer bestimmten Immobilie gekauft (Verweis über
   deren `id`); klappt der Kauf nicht und ist keine Immobilie mehr im Besitz, beginnen die Mieten.
   Die Tragbarkeit rechnet mit dem Einkommen (Erwerb, Renten): (`sustainability` × Jahreseinkommen − Nebenkosten) /
   `imputedRate`; `wealthContribution` wird nicht mehr verwendet. #266: Mit `wealthDrawdown` (Standard aus) zählt
   `wealthDrawdownRate` (Standard 3 %) × freies Vermögen (Bargeld, Aktien, Obligationen im Monat der Prüfung, negativ = 0)
   zum Jahreseinkommen. Beim Kauf ist das freie Vermögen `liquid − Preis + M` und hängt von der Hypothek M ab; die höchste
   Hypothek löst `imputedRate × M + Nebenkosten = sustainability × (Einkommen + Satz × (liquid − Preis + M))`
   (`purchase_max_mortgage`). Sobald alle Personen pensioniert sind (letzter Bezug der Pensionskasse, `retire_at`), ist die
   Hypothek beim Kauf höchstens `min(maxLoanToValue, maxLoanToValueRetired)` × Preis und bei der Erneuerung höchstens
   `maxLoanToValueRetired` (Standard 65 %) × Marktwert; der Rest wird wie fehlende Tragbarkeit behandelt. Die Werte `realestate.affordability` sind seit #233
   eine Einstellung der App (`affordability` in `einstellungen.json`) und stehen nicht mehr in der Planungsdatei: Die
   Oberfläche setzt sie vor jeder Rechnung in die Planung ein (`Engine.call`, nur wenn die Planung keine eigenen hat, etwa
   die Planung eines gespeicherten Ergebnisses), entfernt sie beim Speichern und übernimmt beim Öffnen die abweichenden Werte
   einer älteren Datei in die Einstellungen (Hinweis im Dialog der Änderungen). Der Rechenkern und die Kommandozeile
   lesen sie unverändert aus der Planung; fehlen sie, gelten die Standardwerte (1/3, 5 %, 1 %, 80 %; #266: aus, 3 %, 65 %). Sie wird auch bei **jeder Erneuerung** einer Hypothekarperiode
   geprüft (Einkommen im Erneuerungsmonat, Nebenkosten wie erfasst, kalkulatorischer Zins `imputedRate`): Der nicht tragbare
   Teil ist die minimale Einmalzahlung. Sie wird aus dem Vermögen anteilig auf alle Tranchen bezahlt, wenn das Vermögen
   danach >= 0 ist und das Notverkaufskriterium nach der Zahlung nicht auslöst (Schätzung: geplante Ausgaben + Zins,
   Amortisation und Nebenkosten nach der Zahlung − Einkommen); die Hypothek wird dann reduziert erneuert. Sonst keine
   Zahlung, keine Erneuerung, Notverkauf im Folgemonat. Jede Erneuerung steht in `renewals` des Laufs
   (`affordable`, `lumpSum`, `debtBefore`, `debtAfter`, `emergencySale` = «Vermögen < 0» oder «Notverkaufskriterium»); die
   Zusammenfassung zählt `lumpSums` und `affordabilitySales`, die Stapelauswertung `lumpSumRuns` und `affordabilitySaleRuns`.
   Mit `"amountMode": "maxLoan"` beträgt die Hypothek beim Kauf die höchste Belehnung (`maxLoanToValue`, Standard 80 %, also Minimum
   Eigenmittel), gekürzt falls nicht tragbar; die Beträge der Tranchen gelten dann als Anteile.
   **«Immobilien nie verkaufen»** (`"realestate": {"neverSell": true}`, #196; fehlt das Feld: aus, «aus» wird nicht
   geschrieben): kein geplanter Verkauf (`sell` wird beim Übersetzen ignoriert, die Tranchen laufen bis zum Planende),
   kein Notverkauf (weder Kriterium noch 3-Topf «Töpfe reichen nicht»), und bei einer nicht tragbaren Erneuerung, die
   sonst einen Notverkauf auslösen würde, wird die Hypothek voll erneuert (Ereignis `event_mortgage_unaffordable_kept`,
   `emergencySale` leer, `affordabilitySales` unverändert; die Zusammenfassung zählt sie in `unaffordableRenewals`, die
   Stapelauswertung die Läufe in `unaffordableRenewalRuns`, beide fehlen im JSON bei 0, #199); eine Einmalzahlung, die
   die Bedingungen oben erfüllt, bleibt und zählt in `lumpSums`.
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

**Pflegeheim (#271):** `household.persons[i].careHome` = `{ age | at, monthly, sell }` (Eintritt als Alter der Person
oder `TimeRef` mit Datum oder Meilenstein; Kosten pro Monat in Preisen des Starts, mit dem Teuerungsindex fortgeschrieben;
`sell` Standard nein). Nur Personen, die im Rechenkern zählen (zweite Person nur mit `partner`). `Simulator::new`
sammelt je Person Eintritt und Kosten (`care`; ein Eintritt vor dem Start gilt ab dem Start), `all_in_care` = Monat, ab
dem alle Personen im Heim sind (spätester Eintritt), `care_sale` = derselbe Monat, wenn eine Person `sell` gewählt hat
und `realestate.neverSell` aus ist. Im Monat:
- Heimkosten = Summe der Personen ab ihrem Eintritt × Teuerungsindex; sie zählen wie die Wohnkosten zu den Ausgaben
  (`MonthRecord.care`, in `spending` und `withdrawal`), zur Ausgaben-Seite der Entnahmeregeln (Fixkosten der
  Ausgabenregel, Notverkaufskriterium, Netto-Bedarf der 3-Topf-Strategie `NeedBasis.care`, `need_rest`) und im
  Kassenbuch zu `Ledger.care`. Ereignis `careHome` im Eintrittsmonat.
- Ab `all_in_care`: keine Miete (auch nicht mit festem Beginn), `after_sales` bricht ab (keine Käufe und Mieten «nach
  Verkauf», auch nach einem Notverkauf), kein reservierter Verkaufserlös für einen Folgekauf. Im Monat `care_sale`
  werden alle aktiven Liegenschaften am Ende der Monatsrechnung verkauft wie ein geplanter Verkauf (Grundstückgewinnsteuer,
  Hypothek abgelöst). Bei Paaren laufen Miete und Liegenschaft weiter, bis beide im Heim sind; ein Todesfall wird nicht
  abgebildet.
- **Steuern (vereinfachte Regel für Bund und alle Kantone):** Die Heimkosten des Steuerjahres (`YearDeductions.care`,
  nominal) sind Krankheits- und Behinderungskosten. Abzug = max(0, Heimkosten − 5 % × Nettoeinkommen), Nettoeinkommen
  = Einkommen minus die übrigen Abzüge (je Kanton und Bund, mindestens 0; `sim::care_deduction`, `CARE_SELF_SHARE`).
  Steuerbares Einkommen = max(0, Nettoeinkommen − Abzug). Beispiel: Nettoeinkommen 60 000, Heimkosten 108 000 → Abzug
  105 000, steuerbar 0. `TaxBasis` (und die Jahreswerte) führen `care`, `careCanton`, `careFederal`; ein Plan mit
  Pflegeheim schreibt die Basis immer.
- **Prüfung:** Eintritt fehlt (`care_home_entry_missing`), Alter ausserhalb 0..150 (`care_home_age_invalid`), Kosten
  negativ (`care_home_costs_invalid`) sind Fehler; `sell` mit «Immobilien nie verkaufen» ist eine Warnung
  (`care_home_never_sell`). `housing_missing_after_sale` (Warnung, Abschnitt Immobilien): Alle Liegenschaften im
  Besitz oder mit Kauf haben einen geplanten Verkauf innerhalb der Planung, und danach folgt weder eine Miete (mit
  «nach Verkauf» oder über den Verkaufsmonat hinaus) noch der Heimeintritt aller Personen (spätestens im Monat nach dem
  Verkauf). Ein Verkauf wegen des Pflegeheims setzt voraus, dass alle im Heim sind; nach ihm gibt es deshalb nie einen
  Hinweis. Notverkäufe kennt die Prüfung nicht.
- Ohne `careHome` rechnet alles wie bisher; `care` erscheint in Ergebnissen, Jahreswerten (`YearRow.care`,
  `YearFlows.care`) und Bericht (`years.columns` + `care`) nur, wenn es Heimkosten gibt (Golden-Dateien unverändert).

**Steuern:** Satz aus der Tabelle linear interpoliert × Steuerfuss, auf Rappen gerundet. Kapitalbezüge im selben
Kalenderjahr (PK, 3a) werden für den Satz zusammengerechnet: für den Haushalt (eine Person, Verheiratete) oder je
Person (unverheiratetes Paar, #150). Die Oberfläche zeigt Steuerbeträge als «geschätzt» und auf 100 Franken gerundet
(#143, nur Anzeige: `roundTax`/`fmtTax` und `taxRounded` für Ereignistexte in `util.js`); Rechenkern, Kommandozeile und
CSV-Export (Spalte `taxes_estimated`) bleiben genau.

**Tarife mit der Teuerung** (`taxes.indexed`, #66): Die Beträge der Tabellen für Einkommen, Vermögen und Kapitalbezug
(auch `taxes.couple`) wachsen mit dem Preisindex des Pfades seit Simulationsstart (`Run::infl_idx`, derselbe wie für
Lebenshaltung und Einkommen); die Sätze bleiben gleich. `tax::tax_indexed(table, x, multiplier, index)` nimmt den Satz
für `x / index` (Betrag in Preisen des Starts), Steuer = `x × Satz × Steuerfuss`, auf Rappen gerundet; bei Index ≤ 0
gilt die Tabelle ohne Anpassung. Massgebend ist der Index im Monat der Veranlagung (Dezember für Einkommen und
Vermögen, Monat des Bezugs für Kapitalbezüge, Januar bzw. Neuberechnung für die geschätzte Einkommenssteuer im Bedarf
der 3-Topf-Strategie). Fehlt das Feld, gilt «nein» und es wird nicht geschrieben; der Import alter `config.json`
setzt «nein», `new.vplan` und «Neu» in der Oberfläche setzen «ja». Die Grundstückgewinnsteuer bleibt unverändert.
Im Modus «Wohnort» wachsen ebenso die Beträge der Tarife (Satz für `x / index`) und der Abzüge (`base`, `min`, `max`
× Index; der Satz `rate` bleibt).

### Steuer nach Wohnort (#63)

Modell: `taxes.mode` ist `"own"` (Standard, wird nicht geschrieben) oder `"location"`; `taxes.location` =
`{ canton: "ZH", municipality: 261, church: "none" | "reformed" | "catholic" | "christCatholic" }`, `municipality` ist
die BFS-Nummer. Die Steuerdatei `crates/engine/data/steuern-2026.json` erzeugt `tools/steuerdaten/build.py` aus den
ESTV-Grunddaten (Aufbau und Ablauf für ein neues Steuerjahr: `tools/steuerdaten/README.md`); `taxdata.rs` bettet sie
ein (`BUILTIN`).

- **Tarif und Abzüge getrennt:** Die Einkommenskurven `[steuerbares Einkommen, Satz]` sind der Tarif der einfachen
  Steuer (Steuerfuss 100 %, Verheiratete mit dem Splitting des Kantons); die Standardabzüge stehen als Regeln
  `{base, rate, min, max}` je Kanton und Bund, Zivilstand und Variante (`retired`: ohne, `working`: mit Beiträgen an
  die 2. und 3. Säule) und `earned` (Berufskostenpauschale). Abzug = `base × Index + rate × Bezug`, mindestens
  `min × Index`, höchstens `max × Index` (0 = ohne); Bezug ist das Einkommen, bei `earned` der Erwerb der Person
  (höchstens der Erwerb, ohne Erwerb 0). Abweichung von VivaPlan-Private: dort sind die Abzüge (Variante «ohne») in
  die Kurven eingerechnet; für Rentner ergibt sich dieselbe Steuer (Abweichung unter 1.5 %).
- **Zweitverdienerabzug** (`secondEarner`, nur Verheiratete, nur wenn beide Erwerbseinkommen haben): Regel wie oben,
  Bezug `lower` = das tiefere Erwerbseinkommen nach der Berufskostenpauschale (Bund, fast alle Kantone) oder `total` =
  Summe der Erwerbseinkommen (Bern); höchstens das tiefere Erwerbseinkommen nach der Berufskostenpauschale. Thurgau hat
  keinen. Bezug und Obergrenze sind aus dem ESTV-Steuerrechner 2026 abgeleitet (ZH, BE, LU, AG, SG, AR, GL, NE, Bund).
- **Steuerbares Einkommen** je Kanton und Bund = Jahreseinkommen (Erwerb netto + PK-Rente + AHV) − Abzüge, nie
  negativ. Variante `working`, sobald im Kalenderjahr eine Person Erwerbseinkommen hat, sonst `retired`; die
  Berufskostenpauschale gilt je Person (`Run::year_earned`).
- **Steuer:** `TaxData::resolve(location, married)` fasst Kantonskurve × (Steuerfuss Kanton + Kirche) und
  Gemeindekurve (Schwyz, Wallis: eigene, sonst die des Kantons) × Steuerfuss Gemeinde mit `combine` exakt auf den
  vereinigten Stützpunkten zusammen; Einkommenssteuer = diese Tabelle auf das steuerbare Einkommen Kanton + Bundeskurve
  auf das steuerbare Einkommen Bund (`Resolved::income_tax`). Vermögenssteuer = Vermögenskurve (Sozialabzüge
  eingerechnet, Satz auf das Vermögen vor Abzügen) × (Kanton + Gemeinde + Kirche), ohne Bund. Verheiratet heisst wie
  bei `taxes.couple`: `household.married` und entweder eine zweite Person oder die Markierung
  `household.marriedNoted` (#250: dann auch eine Person allein; ältere Planungen mit einer Person übernimmt
  `upgrade` als unverheiratet); unverheiratete Paare rechnen mit den Kurven und Abzügen für Alleinstehende auf dem
  Haushaltseinkommen. Die Plafonierung der AHV braucht immer zwei Personen.
- **Abzüge vom Einkommen (#131):** `taxes.deductions` = zeitabhängige Zeilen `{ from, kind: "canton" | "own", amount,
  rate }` (`CDeduction`, sortiert wie eine Kurve), `taxes.deductPillar3a` (Standard nein). Fehlt `deductions`, gilt
  wie vor #131 im Modus «Wohnort» der Vorschlag über die ganze Zeit, mit eigenen Tabellen nichts. `Run::year_ded`
  (`YearDeductions`) sammelt je Monat: Monate mit Vorschlag (nur im Modus «Wohnort»), eigener Betrag und `rate` ×
  Monatseinkommen, Einzahlungen in die Säule 3a je Person (Beitrag und indirekte Amortisation). Im Dezember
  (`Simulator::income_tax`): Anteil = Vorschlagsmonate / Monate des Jahres; Vorschlag = Anteil × `Resolved::proposal`
  (Kanton, Bund); eigener Wert = Summe der Beträge / Monate × Index + Summe `rate` × Einkommen; Säule 3a = Σ je Person
  min(Einzahlung, `pillar3a_cap`) (einzige Stelle der Begrenzung: `limits.pillar3aMax` × Index; #152 je Person, siehe
  unten). Steuerbar Kanton/Bund = Einkommen − (Vorschlag + eigener Wert + 3a), nie negativ; mit eigenen Tabellen
  ohne Abzüge exakt das Einkommen (Golden-Dateien unverändert). Die Schätzung des Netto-Bedarfs (3-Topf, `plan_basis`)
  rechnet mit denselben Zeilen (Anteil am Einkommen mit der 13. AHV-Rente) und den geplanten 3a-Beiträgen, auf 12
  Monate hochgerechnet. `MonthRecord.tax_basis`
  (`TaxBasis`: Einkommen, Vorschlag Kanton/Bund mit Anteil und Bestandteilen je Art, eigener Wert, 3a, steuerbar
  Kanton/Bund) nur im Dezember und nur bei Planungen mit Abzügen (Modus «Wohnort», `deductions` oder
  `deductPillar3a`). Prüfung: Betrag ≥ 0, `rate` 0 bis 1 (Fehler am Feld), Zeitpunkt auflösbar; Zeilen «canton» mit
  eigenen Tabellen ungeprüft und ohne Wirkung.
- **Zusätzliche und einmalige Abzüge (#230):** Zeilen «canton» haben `extra` (Betrag pro Jahr in Preisen des Starts,
  nur gespeichert, wenn ≠ 0; `amount` wird dort nicht umgedeutet, alte Planungen können Werte enthalten).
  `YearDeductions.extra` summiert `extra` über die Vorschlagsmonate (nur Modus «Wohnort»); im Dezember zusätzlich =
  Summe / Monate × Index. `taxes.oneOffDeductions` = Liste `{ year, label, amount }` (nominal, mehrere im selben Jahr
  addiert); `Run` setzt `year_ded.one_off` erst bei der Veranlagung im Dezember, die Schätzung des Netto-Bedarfs
  (`plan_basis`) rechnet deshalb nur mit den laufenden Abzügen. Steuerbar = Einkommen − (Vorschlag + zusätzlich +
  eigener Wert + einmalig + 3a), nie negativ, in beiden Modi. `TaxBasis` hat `extra` und `oneOff`; einmalige Abzüge
  allein schalten die Aufzeichnung ein. Prüfung: `extra` ≥ 0, einmaliger Betrag ≥ 0 und Steuerjahr zwischen dem Jahr
  des Starts und dem Jahr des Endes (Fehler am Feld).
- **Abzüge vom Vermögen (#231):** `taxes.wealthDeductions` = Liste `{ from, amount }` (Betrag in Preisen des Starts,
  × Index bei «Tarife mit der Teuerung»), `taxes.oneOffWealthDeductions` = Liste `{ year, label, amount }` wie
  `oneOffDeductions` (nominal, mehrere im selben Jahr addiert); beide nur gespeichert, wenn nicht leer.
  `Simulator::taxable_wealth` rechnet im Dezember: Vermögen (mit `tax_equity`, #229) − Zeile, die im Dezember gilt, −
  einmalige Abzüge des Jahres, nie negativ; ohne beide Listen genau das Vermögen wie bisher. Im Modus «Wohnort» geht der
  Rest in die Vermögenskurve des Wohnorts (die Sozialabzüge des Kantons stecken darin, #63), mit eigenen Tabellen in die
  Tabelle. `TaxBasis.wealth` = `WealthBasis { wealth, ongoing, oneOff, taxable }`, nur mit Abzügen vom Vermögen; diese
  schalten die Aufzeichnung ein. Prüfung: Zeitpunkt auflösbar, Beträge ≥ 0, Steuerjahr zwischen Start und Ende (Fehler am
  Feld).
- **Zusammenstellung des steuerbaren Vermögens (#249):** Mit `record` legt der Lauf im Dezember je Steuerjahr ein
  `WealthTaxYear { year, liquid, properties: [PropertyTax { index, name, worth, taxValue, hasTaxValue, mortgage }],
  ongoing, oneOff, taxable }` ab (`liquid` = bar + Obligationen + Aktien; `taxValue` ohne Steuerwert = Marktwert;
  `taxable` = derselbe Wert wie für die Vermögenssteuer). Es steht in `SimResult.wealth_tax` (`serde(skip)`, nicht in
  den gespeicherten Ergebnissen und Golden-Dateien) und geht wie `pillar3a` über `PreviewOut.wealthTax` an die
  Oberfläche (Hinweis und Details der Karte «Abzüge vom Vermögen»).
- **Grosse Säule 3a (#152):** `household.persons[i].noPensionFund` (Standard nein, nur gespeichert, wenn gesetzt) =
  erwerbstätig ohne Pensionskasse. Für diese Person ist `pillar3a_cap` = min(20 % (`LARGE_PILLAR3A_SHARE`) × ihr
  Erwerbseinkommen des Kalenderjahres, `limits.pillar3aMaxLarge` × Index); Index = Teuerung bei «Tarife mit der
  Teuerung» (#66), sonst 1. Das Erwerbseinkommen je Person (`Run::year_earned`) wird dafür immer gesammelt, auch mit
  eigenen Tabellen. Alle anderen Personen: `limits.pillar3aMax` × Index wie bisher.
  `limits.pillar3aMaxLarge` Standard 36 288 (BSV, FAQ «Welche Beiträge kann ich in die Säule 3a einzahlen?», Stand
  2025/26; Fachmitteilung admin.ch vom 5.10.2026 «Höchstabzüge Säule 3a im Steuerjahr 2027»: bis Ende 2026 36 288).
  Einzahlungsgrenze der Planung (`validate.rs`, dieselbe Regel): Einzahlung 0 bis `pillar3a_max_of` (grosse oder
  kleine Säule 3a, Fehler am Feld); Einzahlung + indirekte Amortisation über diesem Betrag: Warnung
  `pillar3a_over_max`; sonst ohne Pensionskasse Warnung `pillar3a_over_income` für den ersten Monat vom Start bis vor
  dem AHV-Bezug, in dem 20 % × 12 × Monatseinkommen darunter liegt. Ohne das Feld rechnen bestehende
  Planungen unverändert (Golden-Dateien unverändert).
- **Ohne Pensionskasse rechnet die Pensionskasse mit 0 (#253):** Ist `noPensionFund` gesetzt, setzt `Simulator::new`
  (`person_cfg`) für diese Person Guthaben, Beiträge (Kurve 0), Kapitalbezugsanteil und Umwandlungssatz auf 0: kein
  Guthaben, keine Beiträge, keine Rente, kein Kapitalbezug und keine Kapitalbezugssteuer. Der Bezugszeitpunkt bleibt
  (Meilenstein «Bezug Pensionskasse», Beginn der Pensionierung). Die Werte unter `pension.pk` bzw. `partner.pk` bleiben
  unverändert in der Datei und werden weiter geprüft; die Warnung `pillar3a_large_with_pk` (#152) gibt es nicht mehr.
  Oberfläche: Schalter zuoberst in der Karte «Pensionskasse» (`PensionCards`), die Felder darunter stehen in einem
  `fieldset.locked-area` (`#pkBody0`/`#pkBody1`), das bei gesetztem Schalter `disabled` ist (wie #196).
- **Kapitalbezüge (#64):** je Kanton `capital` (Satz der einfachen Steuer auf den ganzen Bezug, Steuerfuss 100 %,
  Alleinstehende und Verheiratete), wo nötig `capitalMunicipal` (eigene Regel der Gemeindesteuer: Schwyz, Wallis,
  Genf; Freiburg wegen `cantonMultiplier`), `capitalRule` (Text) und `capitalApprox` (Näherung: Wallis, Zug); dazu
  `federalCapital` (ein Fünftel des Bundestarifs, ohne Abzüge). `resolve` liefert `capital` = Kantonskurve × (Steuerfuss
  Kanton + Kirche) + Gemeindekurve × Steuerfuss Gemeinde + Bundeskurve, mit den Steuerfüssen der Einkommenssteuer.
  `Run::capital_tax` nimmt im Modus «Wohnort» diese Tabelle mit Steuerfuss 1 statt `taxes.capital` × `taxes.multiplier`;
  Bezüge eines Kalenderjahres zusammen, bei Verheirateten für den Haushalt (Kurve für Verheiratete), bei unverheirateten
  Paaren je Person (Kurve für Alleinstehende, #150). «Tarife mit der Teuerung» gilt (Satz für `x / index`). Herkunft
  der Regeln: `tools/steuerdaten/kapitalregeln.json` und README dort. `tax_locations` liefert je Kanton `capitalRule`
  und `capitalApprox`; die Oberfläche zeigt die Regel aus den Texten `capitalRule.<Kanton>` (deutsch = `capitalRule`
  der Steuerdatei, e2e prüft das).
- **Grundstückgewinnsteuer (#65):** je Kanton `propertyGain` (Regelsatz im Format von `PropertyGainTax`),
  `propertyGainRule` (Text), `propertyGainApprox` (Näherung: BS, BL, SG, VD, VS) und `gainMultiplier` (Median von
  Steuerfuss Kanton + Gemeinde der Einkommenssteuer über die Gemeinden des Kantons); dazu `propertyGainSource`.
  `TaxData::property_gain(location, canton)` liefert Regelsatz und Steuerfuss für eine Immobilie im Kanton `canton`
  (fehlt oder leer: Wohnkanton): im Wohnkanton den Steuerfuss der Wohngemeinde, sonst `gainMultiplier`. Der Simulator
  legt beim Aufbau je Immobilie die Regeln fest (`GainRule`): im Modus «Wohnort» ohne `taxes.propertyGainOwn` die des
  Kantons, Besitzdauer in Monaten; sonst `taxes.propertyGain` mit `taxes.multiplier`, Besitzdauer in vollen Jahren wie
  bisher. Mietobjekte brauchen keine Regel. Herkunft der Regeln: `tools/steuerdaten/grundstueckgewinn.py` und README
  dort; `tax_locations` liefert `propertyGainRule` und `propertyGainApprox`, die Oberfläche zeigt die Texte
  `propertyGainRule.<Kanton>` und `propertyGainApprox.<Kanton>` (deutsch = Steuerdatei, e2e prüft das).
- **Wohnort zeitabhängig (#151):** `taxes.location` ist die erste Zeile (ab Start, Form von #63 unverändert),
  `taxes.relocations` = weitere Zeilen `{ from, canton, municipality, church }` (`model::Relocation`, Feld fehlt bei
  einem einzigen Wohnort und wird dann nicht geschrieben). `Simulator::locations` (`CLocation { from, place, resolved }`)
  hält je Zeile den aufgelösten Wohnort, stabil nach Monat sortiert (gleicher Monat: die spätere Zeile gilt; Zeitpunkt
  «nie» oder dynamisch: ohne Wirkung); `location_at(m)` liefert die Zeile im Monat `m`. Stichtage nach dem
  Steuerharmonisierungsgesetz (StHG, SR 642.14, Stand 1. Januar 2025, auf fedlex.admin.ch geprüft am 23.11.2026):
  - Einkommen und Vermögen: `location_of_year(m)` = Wohnort im Dezember des Steuerjahres, für das ganze Jahr
    (`income_tax`, `wealth_tax`, auch die Schätzung des Netto-Bedarfs). Art. 4b Abs. 1 Satz 1: bei Wechsel des
    Wohnsitzes innerhalb der Schweiz besteht die Steuerpflicht für die laufende Steuerperiode im Kanton, in dem die
    Person am Ende der Periode Wohnsitz hat; Art. 17 Abs. 1: das Vermögen bemisst sich nach dem Stand am Ende der
    Steuerperiode. Der Wechsel der Gemeinde im selben Kanton folgt derselben Regel (kantonales Recht, Vereinfachung).
  - Kapitalleistungen aus Vorsorge (Art. 11 Abs. 3): `capital_tax` mit dem Wohnort im Monat des Bezugs, Art. 4b Abs. 1
    Satz 2 (Kanton des Wohnsitzes im Zeitpunkt der Fälligkeit). Bezüge eines Kalenderjahres werden weiter
    zusammengerechnet, auch über einen Wechsel hinweg (Vereinfachung: der Zuschlag rechnet mit der Tabelle des neuen
    Orts).
  - Grundstückgewinnsteuer (#65): `gain_rules` je Immobilie und Wohnort (`GainRule` je Zeile von `locations`, mit
    eigenen Regeln ein Eintrag); beim Verkauf gilt der Eintrag des Wohnorts im Monat des Verkaufs: ohne `canton` der
    Wohnkanton, im Wohnkanton der Steuerfuss der Wohngemeinde, sonst der Median.
  - Abzüge «Vorschlag des Kantons» (#131) mit dem `Resolved` des Steuerjahres; ein eigener Wert bleibt.
  - Ergebnis: `TaxBasis.residence` (Wohnort am 31. Dezember) nur, wenn die Planung mehr als einen Wohnort hat (sonst
    nicht geschrieben); die Jahreswerte zeigen ihn mit den Steuerspalten, die Parameter je Zeile «Wohnort ab …».
  - Prüfung je Zeile am Feld `taxes.relocations.N.*` (`relocation_canton_missing`, `relocation_canton_unknown`,
    `relocation_municipality_missing`, `relocation_municipality_not_in_canton`, Zeilennummer wie in der Oberfläche:
    `taxes.location` ist Zeile 1) und `relocation_before_previous` am Feld `from`, wenn eine Zeile vor der vorherigen
    (oder vor dem Start) beginnt. Mit eigenen Tabellen werden die Zeilen weder geprüft noch angewendet.
  - Nicht enthalten: Wegzug ins Ausland, Umzugskosten, Steuerpflicht für einen Teil des Jahres (pro rata).
  - Tests: `crates/engine/tests/relocation.rs` (`issue_151_…`), e2e `ui-steuern.spec.js` («(#151)»).
- Die eigenen Tabellen für Einkommen, Vermögen und Kapitalbezug (auch `taxes.couple`) und die nicht geltenden eigenen
  Regeln der Grundstückgewinnsteuer bleiben in der Datei, die Prüfung übergeht sie.
- **Prüfung:** Kanton fehlt oder unbekannt (`taxes.location.canton`), Gemeinde fehlt oder liegt nicht im Kanton
  (`taxes.location.municipality`): Fehler im Abschnitt Steuern; `Simulator::new` lehnt die Planung ebenso ab.
- **Befehl `tax_locations`** (App und Entwicklungsserver): Steuerjahr, Quelle, Hinweis, Abzugsregeln von Bund und
  Kantonen und je Kanton die Gemeinden mit den Steuerfüssen für Einkommen und Vermögen. Die Oberfläche lädt ihn beim
  Start, damit Übersicht und Reiter «Parameter» den Gemeindenamen ohne geöffneten Abschnitt Steuern kennen.
- Tests: `crates/engine/tests/tax_location.rs` (`issue_63_ak…`, Referenzfälle ESTV 2026),
  `crates/engine/tests/tax_capital_location.rs` (`issue_64_…`, 48 Referenzfälle Kapitalbezug),
  `tools/steuerdaten/test_build.py`, e2e `ui-steuern.spec.js`.

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
  `Metrics.floorShare` (#205): Anteil der Monate ab Pensionierung mit `MonthRecord.at_floor` (die Untergrenze hebt die
  Lebenshaltung der Ausgaberegel an, `rule_living_bound`), gemittelt über die Pfade; fehlt ohne Untergrenze
  (`Simulator::has_floor`) und in Berichten vor #205. Ab `FLOOR_HINT_SHARE` (50 %) zeigen App und CLI einen Hinweis.
- **Ergebnisse:** Bandbreite mit 1. und 99. Perzentil und dem monatlich schlechtesten Wert; unter «Einzelne Läufe»
  zusätzlich 1., 99. Perzentil und der Lauf mit dem tiefsten Endvermögen. Einzelläufe zeigen den Verlauf von
  Aktien und Obligationen als Index.
- **Installationspakete:** `.github/workflows/pakete.yml` baut auf GitHub Windows (.msi, .exe), Linux (.deb, .AppImage;
  .rpm nur ohne Vorabversion) und macOS (.dmg). Ein Tag `v…` baut alle drei und legt einen Entwurf unter «Releases» an.
  Von Hand gestartet (Eingaben `tag` und `plattformen`, Vorgabe nur macOS, #202/#203) baut er die gewählten Plattformen;
  mit `tag` den Stand dieses Tags und lädt die Pakete in dessen Release (auch Entwurf), z. B.
  `gh workflow run pakete.yml -f tag=v1.0.0-beta.3 -f plattformen=windows,linux`.
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

**Regelsätze der Kantone (#65).** Im Modus «Wohnort» gilt ohne `taxes.propertyGainOwn` für jede Immobilie der Regelsatz
ihres Kantons (`realestate.properties[].canton`, fehlt oder leer: Wohnkanton) aus der Steuerdatei. Dafür kennt
`PropertyGainTax` zwei weitere Modelle und Ergänzungen:

- `curve`: Satz nach Gewinn (`rateCurve`, linear interpoliert, für den ganzen Gewinn; Basel-Landschaft);
- `return`: Satz = Rendite pro Jahr (Gewinn / Anlagekosten / Besitzjahre; bis 5 Jahre in angefangenen Monaten,
  darüber in angefangenen Jahren) zwischen `returnBounds`, Höchstsatz nach Besitzdauer `returnMaxDuration` (Zug);
  Anlagekosten = bezahlter Kaufpreis;
- `allowance` (Freibetrag), `uniformAbove` (ab einem Gewinn ein Satz für den ganzen Gewinn), `longBrackets` (andere
  Stufen ab einer Besitzdauer, Wallis), `gainDuration` (Veränderung des Gewinns nach Besitzdauer, Bern),
  `shortSurcharge` (Zuschlag auf dem Gewinnanteil über einer Schwelle bei kurzer Besitzdauer, Freiburg),
  `durationAbove` (eigene Zu- und Abschläge für den Gewinnanteil über einer Schwelle, Steuer im Verhältnis der
  Gewinnanteile aufgeteilt, St. Gallen), `maxShare` (Höchstbelastung als Anteil des Gewinns).

Ablauf (`tax::property_gain_tax_full`): Freigrenze, Freibetrag, Veränderung des Gewinns, Steuer nach dem Modell,
Zuschlag über der Schwelle, Zu- oder Abschlag nach Besitzdauer (nicht im Modell `aargau`), Steuerfuss (falls
`useMultiplier`), feste Ermässigung, Höchstbelastung, nie negativ. Die Jahre der Tabellen werden mit Monate / 12
verglichen. Eigene Regeln rechnen weiter mit vollen Jahren; damit rechnen bestehende Planungen unverändert.

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
(Fixe Rate, Guardrails, VPW, Endowment, 3-Topf + Guardrails) sind ohne Eintrag fix (#204). Der Befehl `opt_params` liefert der Oberfläche die Parameter mit Einheit,
Standard-Bandbreite und aktuellem Wert; `optimize` auf der Kommandozeile liest die Bandbreiten aus der Planung und führt
die fixen Parameter mit dem Wert der Planung auf (Zeile `fixed:`, beim Vergleich als Parameter).

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

Arten der Einträge (`kind` in den Kopfdaten): `single`, `batch`, `scenarios` (Simulation, Inhalt `{plan, result}`) und
`optimization` (#201, Optimierung oder mit `compare: true` Strategievergleich, Inhalt `{plan, opt: {report, req,
planJson}}`). `planJson` ist `optKey()` beim Start (Planung ohne `optimization` und `limits`, mit der Tragbarkeit der Einstellungen, Schlüssel sortiert), die
Kopfdaten tragen davon einen Hash (`planHash`). Damit findet die App beim Öffnen einer Planung das neueste passende
Ergebnis, ohne alle Inhalte zu laden. Optimierungen lassen sich markieren und löschen, aber nicht vergleichen.

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
| Wirksamkeit | cargo-mutants (PR: geänderte Zeilen gegenüber dem Basis-Zweig; wöchentlich, #202: geänderte Zeilen des Rechenkerns seit dem letzten erfolgreichen wöchentlichen Lauf; voll: lokal oder manuell in Actions) | `make mutants`, `make mutants-diff` | `local/mutants.out/` (`missed.txt`) |
| Eingabebreite | proptest (`crates/engine/tests/proptest_parsers.rs`, zufällige Planungen in `proptest_plans.rs`), cargo-fuzz (`fuzz/`, ganze Rechnung im Ziel `run_plan`) | `make proptest`, `make fuzz` | Fehlerfälle, Korpora |

Miri und loom sind nicht eingerichtet: der Code enthält kein `unsafe` und nur einfache Atomic-Flags (`progress.rs`).

Workflows: `.github/workflows/test-quality.yml` (Push auf `main`, Pull Requests, nächtlich und von Hand) und `mutation-full.yml` (nur manuell gestartet; ein voller Lauf dauert viele Stunden und passt nicht ins Free-Kontingent privater Repos).
Reports liegen als Actions-Artefakte und in der Job-Summary. Geplante Läufe werden **übersprungen**, wenn es seit dem
letzten erfolgreichen Lauf keinen neuen Commit gibt. Lokal ohne CI: `make quality` (`FORCE=1 make quality` erzwingt
einen Lauf; Reports unter `local/test-quality/reports/`). Nötig: `cargo install cargo-nextest cargo-llvm-cov cargo-mutants`.

## Offen für spätere Etappen

weitere Datenquellen (z. B. Schweizer Aktien und Obligationen), Export der Ergebnisse.
