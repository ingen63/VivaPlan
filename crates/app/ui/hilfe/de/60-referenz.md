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
- **Optimierung:** Ziel, Strategien, Pfade und die **Bandbreiten der Parameter** je Strategie stehen unter `optimization` (`ranges`: Strategie, Parameter, dann `fixed` oder `lo`/`hi`, Anteile als Dezimalzahl).
- **Grundstückgewinnsteuer:** `taxes.propertyGain` mit `brackets` (Breite, Satz), `duration` (ab vollen Jahren, Zu- oder Abschlag), `exemptBelow` und `reduction`; das Kaufdatum einer Immobilie im Besitz steht in `bought`. Im Modus «Wohnort» gilt der Regelsatz des Kantons der Immobilie (`canton` je Immobilie, ohne Angabe der Wohnkanton); `taxes.propertyGainOwn: true` schaltet auf die eigenen Regeln um.
- **Wohnort:** `taxes.location` (`canton`, `municipality` als BFS-Nummer, `church`) gilt ab dem Start; weitere Wohnorte stehen in `taxes.relocations` als Zeilen `{"from": "2031-01", "canton": "ZG", "municipality": 1711, "church": "none"}`.
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
| `bench <planung>` | Rechenzeit messen (`--paths`, `--json`) |
| `schema` | JSON-Schema der Planungsdatei ausgeben |
| `library` | Zeitreihen der Datenbibliothek auflisten |
| `import-shiller <datei>` | Shiller-Daten neu einlesen (.xls, .xlsx, .ods) |
| `import-snb <datei>…` | SNB-Würfel einlesen (devkum, plkopr, zimoma, rendoblim) |
| `import-legacy <config.json>` | Konfiguration des früheren Programms als Ausgangspunkt übernehmen |
| `ui-dev` | Oberfläche im Browser, nur lokal (Entwicklung) |

Die Kommandozeile gibt ihre Texte, Meldungen und Ereignisse auf Englisch aus, unabhängig von der Sprache der Oberfläche; Zahlen und Daten stehen im Schweizer Format.

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

**`batch`** (mehrere Pfade): `runs`, `successRate`, `earliestDepletion`, `percentiles` (Endvermögen bei 0 = schlechtester Lauf, 1, 5, 10, 25, 50, 75, 90, 95 und 99 %) und `years` mit `p5` bis `p95` des Gesamtvermögens, dem realen Median `p50Real` und dem Anteil der Läufe mit Vermögen `survival`.

**`scenarios`** (Variationen): `params` und je Kombination `values`, `successRate`, `p5`, `p50`, `p95`, `p50Real`, `atRetirement`, `earliestDepletion`.

Für die Beispielplanung umfasst die Ausgabe von `run --json` weniger als 20 000 Zeichen.

## @ref-set Werte per Feldpfad ändern
Stichworte: set, diff, Feldpfad, ändern, Variante, KI, vergleichen

`vivaplan set <planung> <pfad>=<wert> …` ändert Werte einer Planung, ohne die Datei von Hand zu bearbeiten. Ein **Feldpfad** nennt die Schlüssel der Planungsdatei mit Punkten, Listeneinträge mit ihrer Nummer ab 0: `wealth.stockShare`, `household.persons.0.name`, `milestones.1.years`. Es sind dieselben Pfade, die die Prüfmeldungen unter `field` nennen.

- Der **Wert** wird als JSON gelesen (`0.6`, `true`, `null`, `"Text"`, `{…}`); alles andere gilt als Text, zum Beispiel `project.name=Variante B` oder `returns.startDate=1990-01`.
- **Zeitpunkt** bei zeitabhängigen Werten: `pfad@<zeitpunkt>=<zahl>` ändert die Stufe ab diesem Zeitpunkt; gibt es dort keine, beginnt dort eine neue. Ein Zeitpunkt ist ein Meilenstein (`@retire`), ein Alter der ersten Person (`@age:62.5`) oder ein Monat (`@2035-01`). Beispiel: `wealth.stockShare@start=0.8 wealth.stockShare@retire=0.5` setzt den Aktienanteil vor und nach der Pensionierung; ein konstanter Wert wird dabei zeitabhängig. Bei einer Geraden, einer Tabelle oder einem Faktor ist kein Zeitpunkt möglich; `set` meldet dann einen Fehler.
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
