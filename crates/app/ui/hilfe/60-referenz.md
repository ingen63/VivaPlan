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
- **Optimierung:** Ziel, Strategien, Pfade und die **Bandbreiten der Parameter** je Strategie stehen unter `optimization` (`ranges`: Strategie, Parameter, dann `fixed` oder `lo`/`hi`, Anteile als Dezimalzahl).
- **Grundstückgewinnsteuer:** `taxes.propertyGain` mit `brackets` (Breite, Satz), `duration` (ab vollen Jahren, Zu- oder Abschlag), `exemptBelow` und `reduction`; das Kaufdatum einer Immobilie im Besitz steht in `bought`.
- Das vollständige Format liefert `vivaplan schema` im Terminal.

Die Datei enthält **keine Ergebnisse**. Ergebnisse legt die App in der [Liste der Ergebnisse](#r-list) ab.

## @ref-cli Kommandozeile
Stichworte: Terminal, vivaplan, Befehl, Skript, CLI

Der Rechenkern steht auch im Terminal zur Verfügung (Programm `vivaplan`):

| Befehl | Zweck |
|---|---|
| `run <planung>` | Planung rechnen (`--csv`, `--json`, `--events`) |
| `validate <planung>` | Planung prüfen |
| `optimize <planung>` | Strategie optimieren, siehe [Kommandozeile](#o-cli) |
| `bench <planung>` | Rechenzeit messen (`--paths`) |
| `schema` | JSON-Schema der Planungsdatei ausgeben |
| `library` | Zeitreihen der Datenbibliothek auflisten |
| `import-shiller <datei>` | Shiller-Daten neu einlesen (.xls, .xlsx, .ods) |
| `import-snb <datei>…` | SNB-Würfel einlesen (devkum, plkopr, zimoma, rendoblim) |
| `import-legacy <config.json>` | Konfiguration des früheren Programms als Ausgangspunkt übernehmen |
| `ui-dev` | Oberfläche im Browser, nur lokal (Entwicklung) |

## @ref-library Datenbibliothek: Ordner und Dateien
Stichworte: Ordner, Zeitreihen, Speicherort, Application Support, CSV

Die Desktop-App führt die Bibliothek im Datenordner der App und legt fehlende mitgelieferte Reihen dort an. Die [Anlagestrategie](#f-ret-series) verwendet daraus «Shiller CHF», «Shiller CHF + CH-Obligationen» und die SNB-Daten im Unterordner `quellen`, so dass aktualisierte Shiller- und SNB-Daten wirken; fehlt eine Datei, gilt die eingebaute. Eigene Dateien im Ordner verwendet die App nicht mehr (FEA-53). Unter macOS: `~/Library/Application Support/ch.vivaplan-app.desktop/Zeitreihen`. Planungen verweisen mit `library:Name.csv` auf eine Reihe der Bibliothek, unabhängig davon, wo die Planung gespeichert ist.

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
