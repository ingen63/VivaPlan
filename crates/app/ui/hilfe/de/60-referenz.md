# Referenz

## @ref-shortcuts Tastenkürzel
Stichworte: Tastatur, Shortcut, Kürzel, Strg, Befehl, F1

Auf dem Mac gilt ⌘ (Befehl), unter Windows und Linux **Strg**. ⇧ ist die Umschalttaste.

| Kürzel | Wirkung |
|---|---|
| **F1** | Hilfe zum aktuellen Thema (Feld, Abschnitt oder Ergebnisgrafik) |
| ⌘⇧? | Handbuch (Inhaltsverzeichnis) |
| Esc | Hilfe schliessen; sonst zurück zum Editor |
| ⌘N | Neue Planung |
| ⌘O | Öffnen |
| ⌘S | Speichern |
| ⇧⌘S | Speichern unter |
| ⌘I | Alte `config.json` importieren |
| ⌘↩ | Simulation starten |
| ⌘Z | Rückgängig |
| ⇧⌘Z | Wiederholen |
| ⌘1 … ⌘9, ⌘0 | Abschnitt 1 bis 10 im Editor, in den Ergebnissen die Registerkarte |
| ⌘↑ und ⌘↓ | vorheriger bzw. nächster Abschnitt |
| ⌘/ | Übersicht der Tastenkürzel |

Auf manchen Mac-Tastaturen liegt F1 auf der Helligkeitstaste. Drücken Sie dann **fn+F1** oder verwenden Sie den Knopf «? Hilfe» oder ⌘⇧?.

## @ref-checks Prüfung: Fehler und Hinweise
Stichworte: Prüfung, Fehler, Hinweis, Warnung, Validierung

Rechts im Editor prüft die App die Planung laufend.

- **Fehler (rot):** Die Planung ist unvollständig oder widersprüchlich. «Simulation starten» ist dann nicht möglich. Ein Klick auf den Fehler springt zum betroffenen Abschnitt. **«Simulation starten» und «Optimieren» sind gesperrt**, solange Fehler vorliegen: Der Versuch zeigt «Die Planung hat N Fehler» und führt in den ersten betroffenen Abschnitt. **Pflichtfelder** (zum Beispiel Name und Geburtsdatum jeder Person) tragen ein Sternchen (*); ein Fehler erscheint direkt am Feld, ein nicht lesbarer Wert (zum Beispiel ein leeres Datum) ebenfalls. Solange Sie in ein Feld tippen, **wartet alles**: Es erscheint keine Meldung, und Vorschau, berechnete Werte und Prüfliste bleiben stehen. Erst wenn Sie das Feld verlassen (Tab, Klick daneben oder Enter), wird neu gerechnet und eine Meldung gezeigt oder entfernt. Ein halb eingetippter Wert wie «6» auf dem Weg zu «65» wird so nicht gemeldet. Auswahllisten und Kontrollkästchen wirken sofort. Die vollständige Liste der Pflichteingaben steht in der Spezifikation `docs/Pflichteingaben.md`.
- **Hinweise (gelb):** Die Simulation läuft, aber ein Punkt verdient Ihre Aufmerksamkeit, zum Beispiel eine Zeitreihe in anderer Währung oder die Zusammenrechnung von Kapitalbezügen.

Typische Fehler: Start nach dem Ende, Pensionierung ausserhalb des Zeitraums, mehr als zwei Personen, eine fehlende Zeitreihe bei einer historischen Quelle, ein ungültiger Parameter einer Strategie. Die Meldung nennt die Ursache.

Die Zeile **Vorschau** rechts rechnet einen Lauf mit fixen Renditen. Sie zeigt, wohin die Eingaben führen, ersetzt aber die Simulation nicht.

## @ref-file Die Planungsdatei (.vplan)
Stichworte: Format, JSON, Datei, speichern, schema, Projektdatei

Eine Planung ist **eine Textdatei** im JSON-Format mit der Endung `.vplan`. Sie lässt sich kopieren, archivieren und mit einem Texteditor ansehen («Datei ansehen» im Editor zeigt den Inhalt).

- Das Format wird mit `"format": "vivaplan/1"` gekennzeichnet. Ältere Planungen mit der Endung `.rplan` und der Kennung `pensionsplaner/1` lassen sich weiterhin öffnen; gespeichert wird immer als `.vplan` (die alte Datei bleibt unverändert).
- **Zeitpunkte** sind Kalendermonate (`"2027-05"`) oder Verweise auf Meilensteine (`"retire"`), `"never"` heisst nie.
- **Beträge** gelten pro Monat, **Sätze** pro Jahr als Dezimalzahl (0.05 = 5 %).
- **Zeitabhängige Werte** sind eine Zahl oder `{"mode": "timed", "rows": [{"from": "start", "value": 9500}, {"from": "retire", "value": 8500}]}`.
- **Weitere Formen** ([Zeitabhängige Werte](#f-timed)): Gerade `{"mode": "linear", "start": {"age": 55, "value": 0.9}, "end": {"age": 65, "value": 0.5}}`, Tabelle `{"mode": "table", "interpolation": "step", "points": [{"at": "start", "value": 1}, {"age": 80, "value": 0.8}]}` (`interpolation` `linear` oder `step`, ohne Angabe `linear`), Faktor `{"mode": "factor", "base": 6000, "factor": <Verlauf>}`. Ein Stützpunkt hat entweder `at` (Datum oder Meilenstein) oder `age` (Alter, mit `person`, sonst der ersten Person).
- **Kennlinie** (nur `wealth.stockShare`, [Kennlinie](#f-curve)): `{"mode": "curve", "input": "withdrawalRate", "points": [[0.02, 0.8], [0.06, 0.4]], "eval": "yearly"}`; `input` ist `age`, `liquidReal`, `withdrawalRate`, `fundedRatio`, `stockReturn12m` oder `drawdown`, `eval` `yearly` (Standard) oder `monthly`.
- **Optimierung:** Ziel, Strategien, Pfade und die **Bandbreiten der Parameter** je Strategie stehen unter `optimization` (`ranges`: Strategie, Parameter, dann `fixed` oder `lo`/`hi`, Anteile als Dezimalzahl).
- **Grundstückgewinnsteuer:** `taxes.propertyGain` mit `brackets` (Breite, Satz), `duration` (ab vollen Jahren, Zu- oder Abschlag), `exemptBelow` und `reduction`; das Kaufdatum einer Immobilie im Besitz steht in `bought`. Im Modus «Wohnort» gilt der Regelsatz des Kantons der Immobilie (`canton` je Immobilie, ohne Angabe der Wohnkanton); `taxes.propertyGainOwn: true` schaltet auf die eigenen Regeln um.
- **Wohnort:** `taxes.location` (`canton`, `municipality` als BFS-Nummer, `church`) gilt ab dem Start, mit `taxes.mode: "location"` (neue Planungen; ohne `mode` gelten die eigenen Tabellen); weitere Wohnorte stehen in `taxes.relocations` als Zeilen `{"from": "2031-01", "canton": "ZG", "municipality": 1711, "church": "none"}`.
- **Steuerbarer Ertrag:** `taxes.investmentYield` mit `stocks` und `bonds` als Satz pro Jahr (0.02 = 2 %); fehlt er, gilt 0 %.
- **Steuerwert-Vorgabe:** `realestate.defaultTaxValue: true` (neue Planungen); Immobilien ohne `taxValue` zählen dann mit 50 % des Marktwerts. Fehlt das Feld, gilt der Marktwert.
- **Herkunft Steuerrechnung:** Zeilen von `taxes.deductions`, die aus der Steuerrechnung übernommen sind ([Abzüge aus der Steuerrechnung](#f-tax-bill)), tragen `taxBill` mit dem Steuerjahr, zum Beispiel `{"from": "start", "kind": "own", "amount": 25000, "taxBill": 2025}`; es ändert die Rechnung nicht.
- Das vollständige Format liefert `vivaplan schema` im Terminal.

Die Datei enthält **keine Ergebnisse**. Ergebnisse legt die App in der [Liste der Ergebnisse](#r-list) ab.

## @ref-cli Kommandozeile
Stichworte: Terminal, vivaplan, Befehl, Skript, CLI

Der Rechenkern steht auch im Terminal zur Verfügung (Programm `vivaplan`):

| Befehl | Zweck |
|---|---|
| `run <planung>` | Planung rechnen (`--csv`, `--json`, `--events`, `--metric`, siehe [Eigene Kennzahlen](#ref-metric)) |
| `validate <planung>` | Planung prüfen (`--json`) |
| `set <planung> <pfad>=<wert>…` | Werte per Feldpfad ändern, siehe [Werte ändern](#ref-set) |
| `diff <a> <b>` | Unterschiede zweier Planungen auflisten (`--json`) |
| `optimize <planung>` | Strategie optimieren, siehe [Kommandozeile](#o-cli) |
| `describe [<abschnitt>]` | Katalog für KIs: was sich ändern, variieren und messen lässt (`--plan`, `--json`), siehe [Katalog für KIs](#ref-describe) |
| `mcp` | MCP-Server für KI-Programme, liest, rechnet, ändert und speichert (`--data-dir`, `--library`, `--names`, `--read-only`, `--allow-dir`), siehe [KI-Programme verbinden](#ref-mcp) |
| `evaluate <planung> <experiment.json>` | Experiment rechnen (`--json`, `--yes`), siehe [Experiment](#ref-experiment) |
| `bench <planung>` | Rechenzeit messen (`--paths`, `--json`) |
| `schema` | JSON-Schema der Planungsdatei ausgeben |
| `library` | Zeitreihen der Datenbibliothek auflisten |
| `import-shiller <datei>` | Shiller-Daten neu einlesen (.xls, .xlsx, .ods) |
| `import-snb <datei>…` | SNB-Würfel einlesen (devkum, plkopr, zimoma, rendoblim) |
| `import-legacy <config.json>` | Konfiguration des früheren Programms als Ausgangspunkt übernehmen |
| `ui-dev` | Oberfläche im Browser, nur lokal (Entwicklung) |

Die Kommandozeile gibt ihre Texte, Meldungen und Ereignisse auf Englisch aus, unabhängig von der Sprache der Oberfläche; Zahlen und Daten stehen im Schweizer Format.

**Programm und Datenordner.** Das Programm `vivaplan` ist im Installationspaket enthalten und liegt neben der App. Es wird nicht in den Suchpfad eingetragen; rufen Sie es mit dem vollen Pfad auf:

| System | Pfad des Programms |
|---|---|
| macOS | `/Applications/VivaPlan.app/Contents/MacOS/vivaplan` |
| Windows (Installationsprogramm `.exe`) | `%LOCALAPPDATA%\VivaPlan\vivaplan.exe` |
| Windows (`.msi`) | `C:\Program Files\VivaPlan\vivaplan.exe` |
| Linux (`.deb`, `.rpm`) | `/usr/bin/vivaplan` |

Beim AppImage liegt das Programm im Abbild und lässt sich nicht direkt aufrufen. `vivaplan --help` zeigt alle Befehle. Für KI-Assistenten liegt eine [Anleitung](#ref-skill) bei.

Die Kommandozeile rechnet wie die App mit deren Datenordner: mit den Zeitreihen im Unterordner `Zeitreihen`, auch mit in der App eingelesenen Reihen (`library:<Datei>`), und mit den Grenzen und der Tragbarkeit aus den [Einstellungen](#f-settings) (`einstellungen.json`). Der Datenordner ist unter macOS `~/Library/Application Support/ch.vivaplan-app.desktop`, unter Windows `%APPDATA%\ch.vivaplan-app.desktop` und unter Linux `~/.local/share/ch.vivaplan-app.desktop`. `--data-dir <ordner>` nimmt einen anderen Datenordner, `--library <ordner>` eine andere Datenbibliothek. Fehlt der Datenordner, rechnet die Kommandozeile mit den eingebauten Reihen und den Standardgrenzen und meldet das einmal.

## @ref-json Ausgabe als JSON
Stichworte: JSON, maschinenlesbar, KI, Skript, Standardausgabe, format

`run`, `validate`, `optimize`, `bench` und `diff` geben mit `--json` ohne Dateinamen das Ergebnis in einem festen Format auf die Standardausgabe aus, für Skripte und KI-Assistenten. Hinweise und Fehler gehen auf die Fehlerausgabe; bei einem Fehler endet der Befehl mit einem Fehlercode. `--json <datei.json>` speichert wie bisher das ausführliche Ergebnis in einer Datei.

Jede Ausgabe beginnt mit `format` und der Version. Beträge sind auf ganze Einheiten der Währung gerundet, Anteile auf vier Stellen (0.9512 = 95,12 %), Monate stehen als `"JJJJ-MM"`. Meldungen haben `code`, `params` und den englischen `text`; Prüfmeldungen zusätzlich `severity`, `section` und `field` (Feldpfad).

| `format` | Inhalt |
|---|---|
| `vivaplan-run/1` | `error` (oder `null`), `issues`, `elapsedMs` und je nach Methode `single`, `batch` und `scenarios`; mit `--metric` zusätzlich `metrics` |
| `vivaplan-validate/1` | `valid` (keine Fehler), `issues` |
| `vivaplan-optimize/1` | Einstellungen, `reference` (die Planung, wie sie ist) und `outcomes` (beste Strategie zuerst, mit `params`, `validated`, `feasible` und `patch`) |
| `vivaplan-bench/1` | `runs`, `months`, `elapsedMs`, `usPerRun`, `cores`, `check` |
| `vivaplan-diff/1` | `changes`: je Unterschied `path`, `old`, `new` |

**`single`** (ein Renditepfad): Kennzahlen des Laufs (`endTotal`, `endTotalReal`, `totalAtRetirement`, `depleted`, `minLiquid`, `taxesTotal` …), `years` als Tabelle mit `columns` und einer Zeile je Jahr (`year`, `age`, `liquid`, `propertyEquity`, `pensionFund`, `pillar3a`, `total`: Stand Ende Jahr; `income`, `spending`, `taxes`: Summe des Jahres) und `events` als Paare aus Monat und Text.

**`batch`** (mehrere Pfade): `runs`, `successRate`, `earliestDepletion`, `percentiles` (Endvermögen `endTotal` und `endTotalReal` bei 0 = schlechtester Lauf, 1, 5, 10, 25, 50, 75, 90, 95 und 99 %, Läufe nach dem nominalen Endvermögen geordnet), `percentilesReal` (dasselbe mit den Läufen nach dem Endvermögen in heutiger Kaufkraft geordnet; der Lauf eines Perzentils kann sich unterscheiden) und `years` mit `p5` bis `p95` des Gesamtvermögens, dem realen Median `p50Real` und dem Anteil der Läufe mit Vermögen `survival`.

**`scenarios`** (Variationen): `params` und je Kombination `values`, `successRate`, `p5`, `p50`, `p95`, `p50Real`, `atRetirement`, `earliestDepletion`.

Für die Beispielplanung umfasst die Ausgabe von `run --json` weniger als 20 000 Zeichen.

## @ref-set Werte per Feldpfad ändern
Stichworte: set, diff, Feldpfad, ändern, Variante, KI, vergleichen

`vivaplan set <planung> <pfad>=<wert> …` ändert Werte einer Planung, ohne die Datei von Hand zu bearbeiten. Ein **Feldpfad** nennt die Schlüssel der Planungsdatei mit Punkten, Listeneinträge mit ihrer Nummer ab 0: `wealth.stockShare`, `household.persons.0.name`, `milestones.1.years`. Es sind dieselben Pfade, die die Prüfmeldungen unter `field` nennen.

- Der **Wert** wird als JSON gelesen (`0.6`, `true`, `null`, `"Text"`, `{…}`); alles andere gilt als Text, zum Beispiel `project.name=Variante B` oder `returns.startDate=1990-01`.
- **Zeitpunkt** bei zeitabhängigen Werten: `pfad@<zeitpunkt>=<zahl>` ändert die Stufe ab diesem Zeitpunkt; gibt es dort keine, beginnt dort eine neue. Ein Zeitpunkt ist ein Meilenstein (`@retire`), ein Alter der ersten Person (`@age:62.5`) oder ein Monat (`@2035-01`). Beispiel: `wealth.stockShare@start=0.8 wealth.stockShare@retire=0.5` setzt den Aktienanteil vor und nach der Pensionierung; ein konstanter Wert wird dabei zeitabhängig. Bei einer Geraden, einer Tabelle, einem Faktor oder einer Kennlinie ist kein Zeitpunkt möglich; `set` meldet dann einen Fehler.
- **Prüfung:** Ein unbekannter Pfad, ein falscher Typ oder ein unbekannter Zeitpunkt ergibt eine Meldung, und **keine** der Änderungen wird übernommen. Die übrigen Prüfmeldungen der geänderten Planung erscheinen als Hinweise; die Datei wird trotzdem geschrieben.
- **Datei:** Ohne `-o` schreibt `set` eine neue Datei neben die alte, unter der nächsten freien Nummer (`plan-1.vplan`, `plan-2.vplan`, …), und gibt ihren Namen aus. Mit `-o <datei>` schreibt es dorthin. Eine vorhandene Datei ersetzt `set` nur mit `--overwrite`, nach einer Sicherungskopie `<datei>.bak`. `--overwrite` ohne `-o` ersetzt die Planung selbst.

`vivaplan diff <a> <b>` listet je Unterschied den Feldpfad, den alten und den neuen Wert, zum Beispiel `withdrawal.floor: 0 → 0.8`; mit `--json` im Format `vivaplan-diff/1`.

## @ref-metric Eigene Kennzahlen (--metric)
Stichworte: Kennzahl, metric, Messung, Quantil, Median, Wahrscheinlichkeit, Zeitfenster, Konsum, KI

`vivaplan run <planung> --metric '<json>'` berechnet zusätzlich Kennzahlen, die Sie aus Bausteinen zusammensetzen, auf denselben Renditepfaden wie `run` (ohne Variationen). `--metric` ist mehrfach möglich und nimmt eine Kennzahl oder eine Liste davon. Mit `--json` stehen die Werte unter `metrics`, sonst am Ende des Berichts. Die Kennzahlen werden während der Simulation gesammelt; dafür werden keine Monatswerte gespeichert.

| Baustein | Schlüssel | Auswahl |
|---|---|---|
| Grösse | `quantity` | ein Feld der Monatswerte (`total`, `liquid`, `income`, `spending`, `taxes`, `withdrawal`, `housing`, `mortgage`, `pensionFund`, `age` …) oder abgeleitet: `consumption` (Ausgaben; nach der Erschöpfung nur, was das Einkommen deckt), `plannedConsumption`, `depleted` (1 ab der Erschöpfung), `atFloor` (1, wenn die Untergrenze die Ausgaben bestimmt), die Zähler `emergencySales`, `lumpSums`, `affordabilitySales`, `unaffordableRenewals` |
| Einheit | `real` | `true`: in Preisen des Startmonats (nur Beträge); sonst nominal |
| Zeitfenster | `window` | `from`, `to` (einschliesslich): `start`, `retirement`, `end`, Kennung eines Meilensteins oder `JJJJ-MM`; `fromAge`, `toAge`: Alter der ersten Person; `years`: nur die ersten n Jahre. Ohne Angabe die ganze Dauer |
| Reduktion je Pfad | `reduce` | `annualSums` (Summe je 12 Monate, eine Jahresreihe), `sum`, `mean`, `min`, `max`, `atStart`, `atEnd`, `firstWhen` (Alter im ersten Monat mit der Bedingung), `share` (Anteil Monate mit der Bedingung), `any`, `volatility` (mittlere Änderung der Jahressummen), `shareOfStartLiquid` (Summe im Verhältnis zum flüssigen Vermögen vor dem Fenster) |
| Aggregation über die Pfade | `aggregate` | `{"kind": "quantile", "q": 0.5}`, `mean`, `min`, `max`, `probability` und `count` (mit `condition`), `conditionalMean` (mit `lowest` oder `highest` als Anteil, oder `condition`) |

Eine Bedingung lautet zum Beispiel `{"op": "lt", "value": 50000}` (`gt`, `ge`, `lt`, `le`); ohne Angabe gilt «grösser als 0», also «ja» bei `depleted` und `atFloor`. `condition` neben `reduce` prüft die Monate (`firstWhen`, `share`, `any`), `condition` in `aggregate` die Werte je Pfad. Bei `annualSums` nimmt die Aggregation alle Jahreswerte aller Pfade zusammen, wie die Optimierung. `name` benennt die Kennzahl im Ergebnis.

Jeder Eintrag in `metrics` hat `name`, `value` (oder `null`), `values` (Anzahl Werte), bei Wahrscheinlichkeiten und bei Mitteln je Pfad den Standardfehler `stdError`, bei Quantil, Minimum und Maximum den Pfad `path` sowie die Definition `metric`. Fehler in den Kennzahlen kommen gesammelt als eine Meldung mit der Liste; gerechnet wird dann nicht. `vivaplan schema` beschreibt die Planung, nicht die Kennzahlen.

Beispiele:

```
--metric '{"name": "Konsum Jahre 1–15, Median", "quantity": "consumption", "real": true, "window": {"from": "retirement", "years": 15}, "reduce": "annualSums", "aggregate": {"kind": "quantile", "q": 0.5}}'
--metric '{"name": "Ruin bis 85", "quantity": "depleted", "window": {"toAge": 85}, "reduce": "any", "aggregate": {"kind": "probability"}}'
--metric '{"name": "Anteil Monate an der Untergrenze", "quantity": "atFloor", "window": {"from": "retirement"}, "reduce": "share", "aggregate": {"kind": "mean"}}'
```

Median und P10 von `consumption` (real, ganze Dauer ab `retirement`) sind dieselben Werte wie unter `optimize --compare`.

## @ref-experiment Experiment (evaluate)
Stichworte: Experiment, evaluate, Variation, Raster, Mustersuche, Nebenbedingung, Ziel, Budget, KI

`vivaplan evaluate <planung> <experiment.json> [--json] [--yes]` stellt eine Frage an die Planung als eine JSON-Struktur und rechnet sie in einem Aufruf: Änderungen, Variationen, [Kennzahlen](#ref-metric), Nebenbedingungen und Ziel. Alles ist strukturiertes JSON ohne Formeln in Zeichenketten; das Schema steht unter `definitions.Experiment` in `vivaplan schema`.

| Schlüssel | Inhalt |
|---|---|
| `set` | Änderungen für alle Varianten: Feldpfad → Wert, wie bei [`set`](#ref-set), auch mit Zeitpunkt (`"wealth.stockShare@retire": 0.4`) |
| `vary` | Variationen, jede mit jeder kombiniert: `{"path", "values": […]}` (Liste), `{"path", "range": [von, bis, Schritt]}` (Bereich), `{"path", "follows", "offset"}` (gekoppelt: Wert des anderen Pfads plus `offset`) oder `{"group": [{"set", "vary"}, …]}` (Gruppe: die Mitglieder werden nicht miteinander kombiniert) |
| `metrics` | Kennzahlen wie bei `--metric`, je mit `name` |
| `constraints` | Nebenbedingungen `{"metric", "op", "value"}` mit `op` = `le`, `lt`, `ge`, `gt` |
| `objective` | genau eines: `{"maximize": "<kennzahl>"}`, `{"minimize": "<kennzahl>"}`, `{"weighted": [{"metric", "weight"}, …]}` (gewichtete Summe maximieren) oder `{"score": {…}}` (Ziel der [Optimierung](#o-cli) mit `maxRuin`, `weightMedian`, `volWeight` …). Ohne Ziel werden alle Varianten aufgelistet (höchstens 50) |
| `search` | `method`: `grid` (Raster, alle Varianten; Standard) oder `pattern` (Mustersuche in den Bereichen wie die Optimierung, je Kombination der übrigen Werte; `samples` Zufallsstartpunkte, Standard 30; `seed`) |
| `budget` | `paths` Suchpfade (1'000), `margin` Marge auf die Nebenbedingungen in Standardfehlern (2), `top` Anzahl bestätigter Varianten (5), `confirmPaths` Bestätigungspfade (10'000) |
| `confirm` | `true`: auch bei langer Rechnung rechnen |

**Ablauf.** Zuerst wird das ganze Experiment geprüft: Fehler kommen als Liste mit Pfad im Experiment (zum Beispiel `vary[0].values[1]`), Grund und gültigen Werten, und es wird nichts gerechnet. Dann wird die Dauer geschätzt; ab 60 Sekunden rechnet `evaluate` nur mit `--yes` (bzw. `"confirm": true`) und nennt sonst die Schätzung. Gesucht wird mit wenigen Pfaden; eine Nebenbedingung muss dabei mit der Marge erfüllt sein. Die besten `top` Varianten werden auf **unabhängigen** Pfaden (anderer Startwert) bestätigt; die Rangliste ordnet nach den bestätigten Werten. Alle Varianten rechnen auf denselben Pfaden, so sind Unterschiede zwischen ihnen genauer als die einzelnen Werte. Bei festen Pfaden (historisch, alle Startdaten) gibt es keine unabhängigen Pfade; ein Hinweis sagt das.

**Ergebnis** (mit `--json` unter `result`): `ranking` mit `rank`, den Werten der Variante (`settings`), `search` und `confirm` (je `objective`, `feasible`, `figures` mit `value` und `stdError`; bei `score` zusätzlich `optimization`), `hints` (zum Beispiel am Rand des Rasters: die beste Variante liegt am Rand eines Bereichs), `paths` (Anzahl und Startwerte beider Stufen, `independent`), das vollständige `experiment` mit allen Standardwerten und die `version` des Rechenkerns. `status` ist `done`, `invalid`, `needsConfirmation`, `cancelled` oder `failed`; `estimate` nennt Varianten, Läufe und Sekunden. Die Ausgabe bleibt kurz (unter 20'000 Zeichen).

Beispiel: 10 Jahre bis zur Rente, Aktienanteil vorher 40, 60 oder 80 % und nachher 30 oder 50 %, Entnahme mit fixer Rate 3, 4 oder 5 % oder Endowment 3 oder 4 %, möglichst hoher realer Konsum in den ersten 15 Rentenjahren bei höchstens 2 % Ruin – 30 Varianten:

```
{
  "vary": [
    { "path": "wealth.stockShare@start", "values": [0.4, 0.6, 0.8] },
    { "path": "wealth.stockShare@retire", "values": [0.3, 0.5] },
    { "group": [
      { "set": { "withdrawal.strategy": "fixed_rate" },
        "vary": [ { "path": "withdrawal.fixedRate.rate", "values": [0.03, 0.04, 0.05] } ] },
      { "set": { "withdrawal.strategy": "endowment" },
        "vary": [ { "path": "withdrawal.endowment.rate", "values": [0.03, 0.04] } ] }
    ] }
  ],
  "metrics": [
    { "name": "ruin", "quantity": "depleted", "reduce": "any", "aggregate": { "kind": "probability" } },
    { "name": "cons15", "quantity": "consumption", "real": true, "window": { "from": "retire", "years": 15 },
      "reduce": "annualSums", "aggregate": { "kind": "mean" } }
  ],
  "constraints": [ { "metric": "ruin", "op": "le", "value": 0.02 } ],
  "objective": { "maximize": "cons15" }
}
```

Variationen, Strategievergleich und Optimierung lassen sich so formulieren: Variationen sind ein Raster ohne Ziel; der Strategievergleich ist eine Gruppe der Strategien mit `"objective": {"score": {}}` und `budget.paths` = Pfade der Planung; die Optimierung ist eine Mustersuche mit `score` und `"margin": 1.5`.

## @ref-describe Katalog für KIs (describe)
Stichworte: describe, Katalog, KI, Felder, Einheit, Wertebereich, Zustandsgrösse, Kennzahl, Rechenzeit

`vivaplan describe [<abschnitt>] [--plan <planung>] [--json]` sagt einem Programm oder einer KI ohne Quellcode, was sie an einer Planung ändern, variieren und messen kann. Der Katalog wird aus dem Schema und dem Code erzeugt und ist englisch wie die übrige Kommandozeile. Er bleibt unter 20'000 Zeichen; einzelne Abschnitte gibt es mit `describe <abschnitt>`:

| Abschnitt | Inhalt |
|---|---|
| `fields` | Felder der Planung nach Objekt gruppiert, je mit Typ, Einheit, Wertebereich und (nur in diesem Abschnitt) Bedeutung; `*` steht für die Nummer in einer Liste |
| `series` | Formen eines zeitabhängigen Werts (`const`, `timed`, `linear`, `table`, `factor`, `curve`) und die Zeitpunkte (Meilenstein, `JJJJ-MM`, `age:<jahre>`, `pfad@<zeitpunkt>`) |
| `state` | Zustandsgrössen einer [Kennlinie](#f-curve) |
| `metrics` | Bausteine einer [Kennzahl](#ref-metric): Grössen, Zeitfenster, Reduktion je Pfad, Aggregation über die Pfade, Vergleiche |
| `experiment` | Formen des [Experiments](#ref-experiment): Variationen, Nebenbedingung, Ziel, Suchverfahren, Standardwerte |
| `examples` | drei Beispiel-Experimente (Ausgaben bei höchstens 5 % Ruin, Rente oder Kapital, Aktienanteil und Entnahmeregel), die auf der Beispielplanung laufen |
| `performance` | Mikrosekunden je Lauf, beim Aufruf gemessen (rund eine halbe Sekunde) an der Planung von `--plan`, sonst an der Beispielplanung mit parametrischen Renditen; daraus lässt sich die Dauer eines Experiments abschätzen |
| `limits` | Grenzen des Modells: unabhängige Jahre im parametrischen Modell, Auswahlfehler bei vielen Varianten, Unsicherheit, nominal und real, Rechenzeit |
| `choices` | nur einzeln abgefragt: jeder Wert jedes Auswahlfelds mit seiner Bezeichnung in der App (in der Sprache der Oberfläche) und seiner Bedeutung, etwa `bootstrap` = «Monte Carlo · historisch» |

Mit `--json` kommt der Katalog im festen Format `vivaplan-describe/1`. Die vollständigen Beschreibungen und den genauen Aufbau zeigt `vivaplan schema`.

## @ref-skill Anleitung für KI-Assistenten (Agent Skill)
Stichworte: Skill, Agent Skill, SKILL.md, Claude, Claude Code, KI, Assistent, einrichten

Dem Installationspaket liegt eine Anleitung für KI-Assistenten bei, die Befehle ausführen können: die Agent Skill `vivaplan-skill` im offenen Format (Ordner mit der Datei `SKILL.md`). Sie erklärt der KI die [Kommandozeile](#ref-cli), die wichtigen Felder mit ihren Einheiten (Anteile statt Prozent), nominal und real, die Grenzen der Aussagen und den Haftungsausschluss. Die Anleitung ist englisch; die KI antwortet trotzdem in Ihrer Sprache.

| System | Ordner der Skill |
|---|---|
| macOS | `/Applications/VivaPlan.app/Contents/Resources/vivaplan-skill` |
| Windows (Installationsprogramm `.exe`) | `%LOCALAPPDATA%\VivaPlan\vivaplan-skill` |
| Windows (`.msi`) | `C:\Program Files\VivaPlan\vivaplan-skill` |
| Linux (`.deb`, `.rpm`) | `/usr/lib/VivaPlan/vivaplan-skill` |

**Einrichten in Claude Code und in Claude Desktop (Bereich «Code»):** Beide lesen persönliche Skills aus dem Ordner `.claude/skills` in Ihrem Benutzerordner. Kopieren Sie den Ordner `vivaplan-skill` dorthin, unter macOS zum Beispiel im Terminal:

```
mkdir -p ~/.claude/skills && cp -R /Applications/VivaPlan.app/Contents/Resources/vivaplan-skill ~/.claude/skills/
```

Unter Windows kopieren Sie den Ordner nach `%USERPROFILE%\.claude\skills`. Starten Sie danach eine neue Sitzung. Fragen Sie dann zum Beispiel: «Reicht mein Vermögen bis 95? Meine Planung ist ~/Dokumente/planung.vplan». Claude ruft dafür `vivaplan` auf und fragt vorher, ob es Befehle ausführen darf. Nach einer neuen Version von VivaPlan kopieren Sie den Ordner erneut.

**Datenschutz:** Was die KI aus Ihrer Planung oder aus den Ergebnissen liest (Geburtsdaten, Einkommen, Vermögen, Renten), geht an den Anbieter der KI. Die Skill weist die KI an, Sie vor dem ersten Lesen darauf hinzuweisen. VivaPlan selbst sendet nichts.

Die Ergebnisse sind Modellrechnungen ohne Gewähr, keine Steuer- oder Anlageberatung; Steuern sind geschätzt.

## @ref-mcp KI-Programme verbinden (MCP-Server)
Stichworte: MCP, Model Context Protocol, Server, vivaplan mcp, LM Studio, Claude Desktop, KI, Werkzeug, Datenschutz, Speichern, Rückgängig, plan_set, plan_save, Planordner, Nur lesen, Warnung, allow-dir, neue Planung, eigene Zahlen, Inkognito, Gedächtnis

`vivaplan mcp` ist ein MCP-Server: KI-Programme, die das Model Context Protocol unterstützen (zum Beispiel Claude Desktop, Claude Code, ChatGPT Desktop oder LM Studio), starten ihn selbst und sprechen über die Standardein- und -ausgabe mit ihm. Er öffnet keine Netzwerkverbindung. Der Server ändert eine Arbeitskopie im Speicher; auf die Platte schreibt nur `plan_save`, nach festen Regeln (siehe unten).

| Werkzeug | Zweck |
|---|---|
| `plan_open` | Planung (`.vplan`, `.rplan`), eine leere Planung (`new`) oder die Beispielplanung öffnen; Übersicht ohne Werte und Zahl der Prüfmeldungen |
| `plan_get` | Abschnitt oder einzelnes Feld (Feldpfad) lesen |
| `plan_schema` | Aufbau der Planungsdatei nachschlagen |
| `plan_validate` | Planung prüfen wie die App |
| `describe` | [Katalog für KIs](#ref-describe) |
| `result_list`, `result_get` | Ergebnisse der Sitzung (Kurzfassung, Jahreswerte, Ereignisse, Bandbreite, Variationen) |
| `library_list` | Zeitreihen der [Datenbibliothek](#ref-library) |
| `simulation_run` | Planung simulieren wie die App (mit ihren Variationen); Kennzahlen und Ergebnis in Worten |
| `evaluate` | [Experiment](#ref-experiment) auf der Planung: Änderungen, Variationen, Kennzahlen, Bedingungen, Ziel; darüber laufen Optimierung und Vergleich |
| `plan_set` | Werte der Arbeitskopie per Feldpfad ändern (wie [`vivaplan set`](#ref-cli)); alle Änderungen eines Aufrufs sind ein Schritt, mit altem und neuem Wert und den Prüfmeldungen danach |
| `plan_changes` | Alle Unterschiede der Arbeitskopie zur Datei |
| `plan_undo` | Letzten Schritt zurücknehmen, mit `all` alle |
| `plan_apply` | Eine Variante eines `evaluate`-Ergebnisses (nach Rang) in die Planung übernehmen, als ein Schritt |
| `plan_save` | Arbeitskopie als `.vplan` speichern; schreibt als einziges Werkzeug auf die Platte |

Vor jeder Rechnung schätzt der Server die Dauer. Ab 60 Sekunden rechnet er nur, wenn die KI ausdrücklich bestätigt (`"confirm": true`); sie soll Sie vorher fragen. Lange Rechnungen melden den Fortschritt und lassen sich im KI-Programm abbrechen. Es läuft immer nur eine Rechnung aufs Mal.

**Mit eigenen Zahlen beginnen:** Bitten Sie die KI, mit Ihren eigenen Zahlen zu rechnen, und haben Sie noch keine Planungsdatei, öffnet sie mit `plan_open` und `new` eine leere Planung, dieselbe wie *Datei → Neu* in der App. Die Antwort nennt die noch fehlenden Angaben (Name oder Bezeichnung der Person, Einkommen, Ausgaben, Vermögen, Pensionskasse, AHV) und die angenommenen Werte (Geburtsdatum, Pensionierungsalter, Steuertarife). Die KI soll Sie danach fragen und keine Werte erfinden; Partner, Säule 3a, Liegenschaften und einmalige Ausgaben ergänzt sie nur, wenn Sie welche haben. Gespeichert wird die neue Planung mit `plan_save` als neue Datei im (ersten) Planordner; dafür genügt ein Dateiname wie `Meine Planung.vplan`. Die Beispielplanung dient nur zum Ausprobieren und Vorführen.

**Speichern:** `plan_save` hält diese Regeln selbst ein, unabhängig davon, was die KI tut: (1) Ohne Pfad entsteht eine **neue Datei** neben der geöffneten, `<name> (AI).vplan` (bei Bedarf mit Zähler); eine vorhandene Datei bleibt unberührt. (2) Eine vorhandene Datei ersetzt der Server nur mit `overwrite` und erst, nachdem das KI-Programm Sie gefragt und die Änderungen gezeigt hat; kann es nicht fragen, lehnt er ab. (3) Vorher entsteht eine Sicherungskopie `<datei>.bak`; geschrieben wird über eine Zwischendatei. (4) Wurde die Datei seit dem Öffnen geändert, ersetzt er sie nicht. (5) Alte `.rplan`-Dateien ersetzt er nie; gespeichert wird als `.vplan`. (6) Geschrieben wird nur in den Ordner der geöffneten Planung oder in einen mit `--allow-dir <ordner>` freigegebenen (samt Unterordnern). Ein Pfad darf mit `~/` für Ihren Benutzerordner beginnen, wie bei `--allow-dir`. Nennt die KI einen Ordner, den es nicht gibt oder der nicht freigegeben ist, antwortet der Server, dass ein Dateiname allein im Planordner speichert; Ordnernamen nennt er dabei nicht. Mit `--read-only` bietet der Server `plan_save` gar nicht an. Bearbeiten Sie eine Planung nicht gleichzeitig in der App und in der KI: Die App bemerkt Änderungen an der Datei nicht.

**Planordner und «Nur lesen»:** Beim Einrichten wählen Sie einen Planordner; er geht als `--allow-dir <ordner>` an den Server (mehrere Ordner: `--allow-dir` mehrmals). Dann öffnet und speichert der Server Planungen nur in diesem Ordner und seinen Unterordnern. Pfade mit `..` und symbolische Links, die aus dem Ordner hinausführen, lehnt er ab; für eine Planung ausserhalb meldet er, sie liege ausserhalb des Planordners. Der Datenordner der App (Zeitreihen, Einstellungen) bleibt lesbar, damit gleich gerechnet wird wie in der App. Öffnen, Rechnen, Ergebnisse abrufen, Ändern und Speichern im Planordner gehen wie ohne Planordner. Mit dem Schalter «Nur lesen» (`--read-only`, Standard aus) kann die KI Planungen öffnen, ändern und rechnen, aber nicht speichern. Ohne `--allow-dir` liest der Server jede genannte Planung und schreibt nur in ihren Ordner.

**Die Warnung von Claude Desktop:** Beim Installieren des Pakets warnt Claude Desktop, die Erweiterung erhalte Zugriff auf alles auf dem Computer. Das zeigt Claude Desktop bei jeder Erweiterung, denn das Paketformat kennt keine Rechte: Der Server läuft mit Ihren Benutzerrechten, in den anderen KI-Programmen ebenso, nur ohne Warnung. Die Begrenzung auf den Planordner setzt VivaPlan selbst durch, nicht das Betriebssystem. Der Server liest Planungen nur im Planordner, dazu den Datenordner der App und die mitgelieferten Daten; er schreibt nur mit `plan_save` in den Planordner und öffnet keine Netzwerkverbindung.

**Einrichten mit einem Klick:** Das Menü *Hilfe → Mit KI verbinden …* zeigt für Claude Desktop, Claude Code, die ChatGPT-Desktop-App (über die Konfiguration von Codex) und LM Studio, wohin der Eintrag gehört, und die passende Zeile mit dem Pfad des Programms auf Ihrem Rechner zum Kopieren. Jede Zeile enthält den Planordner: vorgeschlagen ist der Ordner der zuletzt geöffneten Planung, sonst der Dokumente-Ordner; *Ordner wählen …* ändert ihn. Der Schalter «Nur lesen» ergänzt `--read-only`. VivaPlan richtet nichts selbst ein. Für Claude Desktop unter macOS liegt jeder Version das Paket `VivaPlan_<Version>_aarch64.mcpb` bei: Ein Doppelklick installiert VivaPlan als Erweiterung in Claude Desktop; das Paket enthält das Programm `vivaplan` selbst. Beim Installieren fragt Claude Desktop nach dem Planordner (Plan folder, Pflicht) und dem Schalter «Nur lesen» (Read only). Während der Beta fragt es zudem nach dem Feld «Beta token»: Ist die App auf diesem Rechner schon aktiviert, bleibt es leer; sonst aktiviert das Token den Server und die App ([Beta-Token](#beta)). Ohne Aktivierung beantwortet der Server jeden Aufruf mit dem Hinweis, VivaPlan sei noch nicht aktiviert.

**Einrichten:** Im KI-Programm tragen Sie das Programm `vivaplan` mit vollem Pfad (siehe [Kommandozeile](#ref-cli)) mit den Argumenten `mcp --allow-dir <planordner>` ein (`~` steht für Ihren Benutzerordner). In Claude Desktop und LM Studio sieht der Eintrag unter macOS so aus:

```
{
  "mcpServers": {
    "vivaplan": {
      "command": "/Applications/VivaPlan.app/Contents/MacOS/vivaplan",
      "args": ["mcp", "--allow-dir", "~/Documents"]
    }
  }
}
```

In Claude Code genügt `claude mcp add --scope user vivaplan -- /Applications/VivaPlan.app/Contents/MacOS/vivaplan mcp --allow-dir ~/Documents`. Der Server rechnet wie die Kommandozeile mit dem Datenordner der App (Grenzen, Zeitreihen); `--data-dir` und `--library` übersteuern ihn.

**Datenschutz:** Alles, was ein Werkzeug zurückgibt, wird Teil des Gesprächs mit Ihrer KI und geht an deren Anbieter; mit einem lokalen Modell (zum Beispiel in LM Studio) bleibt alles auf dem Rechner. Der Server liest nur Planungen, deren Pfad ausdrücklich genannt wird, mit Planordner nur solche darin, und durchsucht keine Ordner. Namen von Personen und frei eingegebene Bezeichnungen ersetzt er durch neutrale (Person 1, Property 1), ausser mit `--names`; Antworten nennen nur den Dateinamen, nicht den Ordner. Keine Antwort ist länger als 20'000 Zeichen. Texte des Servers sind englisch; die KI antwortet trotzdem in Ihrer Sprache. Siehe auch [Datenschutz und lokale Daten](#privacy).

**Inkognito-Chats:** In einem Inkognito-Chat von Claude steht VivaPlan nicht zur Verfügung: Die KI sieht dort keine VivaPlan-Werkzeuge. Das entscheidet das KI-Programm; VivaPlan kann nichts dagegen tun. Wenn Sie Ihre Planungsdaten trotzdem zurückhaltend teilen wollen: (1) Benutzen Sie einen normalen Chat und löschen Sie ihn danach; was der Anbieter danach noch aufbewahrt, regeln seine Datenschutzbestimmungen. (2) Schalten Sie im Datenschutz-Bereich der Einstellungen Ihres KI-Programms das Gedächtnis für diese Gespräche aus, damit es sich nichts daraus merkt. (3) Verwenden Sie ein lokales Modell in LM Studio; dann verlassen die Daten den Rechner nicht.

Die Ergebnisse sind Modellrechnungen ohne Gewähr, keine Steuer- oder Anlageberatung; Steuern sind geschätzt.

## @ref-library Datenbibliothek: Ordner und Dateien
Stichworte: Ordner, Zeitreihen, Speicherort, Application Support, CSV

Die Desktop-App führt die Bibliothek im Datenordner der App und legt fehlende mitgelieferte Reihen dort an. Die [Anlagestrategie](#f-ret-series) verwendet daraus «Shiller CHF», «Shiller CHF + CH-Obligationen» und die SNB-Daten im Unterordner `quellen`, so dass aktualisierte Shiller- und SNB-Daten wirken; fehlt eine Datei, gilt die eingebaute. Eigene Dateien im Ordner verwendet die App nicht mehr. Unter macOS: `~/Library/Application Support/ch.vivaplan-app.desktop/Zeitreihen`. Planungen verweisen mit `library:Name.csv` auf eine Reihe der Bibliothek, unabhängig davon, wo die Planung gespeichert ist.

Eine bereits vorhandene Bibliothek ergänzt die App nicht von selbst um neu mitgelieferte Reihen. Kopieren Sie sie bei Bedarf in den Ordner. Die Indizes stehen unter [Anlagestrategie](#f-ret-series).

## @glossar Glossar
Stichworte: Begriffe, Lexikon, Bedeutung, Abkürzungen

**AHV:** Alters- und Hinterlassenenversicherung, 1. Säule. Die App rechnet nur die Altersrente. Siehe [AHV](#f-ahv).

**Altersguthaben:** Das bei der Pensionskasse angesparte Kapital. Siehe [Pensionskasse](#f-pk).

**Annuität:** Gleichbleibende Zahlungen, die ein Kapital über eine bestimmte Zeit aufbrauchen. Grundlage der [VPW](#f-wd-vpw).

**Bootstrap:** Verfahren, aus echten Daten zufällig neue Verläufe zusammenzusetzen. Siehe [Bootstrap](#f-ret-bootstrap).

**Erfolgsquote:** Anteil der Läufe, in denen das Vermögen bis zum Ende reicht. Siehe [Übersicht](#r-overview).

**Fat Tails:** «Dicke Ränder» der Renditeverteilung: Extreme Ereignisse sind häufiger als bei einer Normalverteilung. Siehe [parametrisches Modell](#f-ret-parametric).

**Guardrails:** Leitplanken für die Entnahme. Siehe [Guardrails](#f-wd-guardrails).

**Kapitalbezug:** Auszahlung von Vorsorgekapital als Einmalbetrag statt Rente, mit Kapitalbezugssteuer. Siehe [Anteil Kapitalbezug](#f-pk-lumpsum).

**Median:** Mittlerer Wert: Die Hälfte der Werte liegt darüber, die Hälfte darunter.

**Monte Carlo:** Berechnung vieler Zufallsverläufe, um Wahrscheinlichkeiten abzuschätzen. Siehe [Simulation](#s-returns).

**Nominal / real:** Nominal sind Beträge in künftigen Franken, real in Kaufkraft von heute. Siehe [Rendite, Inflation und reale Werte](#ret-concepts).

**Perzentil:** Das *n*-te Perzentil ist der Wert, den *n* % der Werte unterschreiten. Siehe [Endvermögen nach Perzentil](#r-percentiles).

**Pensionskasse (PK, BVG):** Berufliche Vorsorge, 2. Säule. Siehe [Pensionskasse](#f-pk).

**Rebalancing:** Wiederherstellen der gewünschten Aufteilung zwischen Aktien und Obligationen. Siehe [Aktienanteil](#f-stockshare).

**Ruin:** Das Vermögen (liquid und Immobilien) ist aufgebraucht. Siehe [Optimierung: Ziel](#o-goal).

**SARON:** Referenzzinssatz für Hypotheken in der Schweiz. Siehe [SARON](#ret-saron).

**Säule 3a:** Gebundene private Vorsorge, ein Konto je Person. Siehe [Säule 3a](#s-pillar3a).

**Seed (Zufallsstartwert):** Startwert des Zufallsgenerators, der Ergebnisse reproduzierbar macht. Siehe [Zufallsstartwert](#ret-seed).

**Sequenzrisiko:** Das Risiko, dass schlechte Börsenjahre gerade am Anfang der Entnahmephase kommen. Siehe [Fallstricke](#ret-pitfalls).

**Tragbarkeit:** Bankregel, nach der die Hypothekarkosten einen Anteil des Einkommens nicht übersteigen dürfen. Siehe [Tragbarkeit](#f-re-afford).

**Tranche:** Teil einer Hypothek mit eigener Laufzeit und eigenem Zins. Siehe [Hypotheken](#f-re-mortgage).

**Umwandlungssatz:** Anteil des Rentenkapitals, der jährlich als Rente ausbezahlt wird. Siehe [Umwandlungssatz](#f-pk-conversion).

**VPW:** Variable Prozententnahme. Siehe [VPW](#f-wd-vpw).

**Volatilität:** Mass für die Schwankung von Renditen, meist als Standardabweichung pro Jahr. Siehe [parametrisches Modell](#f-ret-parametric).
