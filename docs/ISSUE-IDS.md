# Alte Issue-IDs

Bis #117 hatten die Issues eine eigene ID im Titel (`FEA-…` Funktion, `ISS-…` Fehler, `GEN-…` projektweite Aufgabe).
Seither gilt nur die GitHub-Nummer (`#105`). Die Git-Historie ist unverändert: Alte Commits und Tags nennen weiter die
alte ID. Diese Tabelle führt von der alten ID zur Nummer.

- **Doppelt vergeben:** `ISS-7` gibt es zweimal, #33 (Mixed findings) und #34 (Optimum «≥ 100 %» bei Anteilen). Alle
  Verweise im Repo betrafen #34 und sind von Hand zugeordnet. `FEA-88` war bis zur Umbenennung von #114 doppelt; heute
  gehört die ID nur zu #113.
- **Ausnahmen:** Diese Datei ist die einzige im Repo mit alten IDs. `tools/report/test_report.py` prüft, dass alte
  Kennungen nicht mehr zählen; die Beispiele dort sind zusammengesetzt (`"FE" "A-100/AK2"`), damit `git grep` sie
  nicht findet.
- **Werkzeug:** `tools/github-issues/` (erste Anlage der Issues aus den Spezifikationen) ist entfernt: Es erkannte
  vorhandene Issues am Präfix im Titel und würde ohne Präfix alle Issues neu anlegen.

## Zuordnung

| Alte ID | Issue | Titel |
|---|---|---|
| FEA-1 | #3 | Zeitleiste: Immobilienereignisse beschriften |
| FEA-2 | #4 | Beim Start die letzte Planung öffnen |
| FEA-3 | #5 | Ergebnisse: Reiter «Parameter» |
| FEA-5 | #6 | Guardrails und 3-Topf gemeinsam anwenden |
| FEA-6 | #7 | Jahreswerte: Entnahmerate als Spalte |
| FEA-7 | #8 | Jahreswerte: Steuerspalten einklappbar |
| FEA-8 | #9 | 3-Topf-Herleitung aufklappbar in den Jahreswerten |
| FEA-9 | #10 | Ereignisliste ohne 3-Topf-Meldungen |
| FEA-10 | #11 | Gespeicherte Ergebnisse: Strategie-Tooltip mit Parametern |
| FEA-11 | #12 | Gespeicherte Ergebnisse: Spalte «Zeitreihe» statt «Art» |
| FEA-12 | #13 | Gespeicherte Ergebnisse: 5-%-Vermögen und Warnsymbol bei Notverkauf |
| FEA-13 | #14 | Pflichteingaben prüfen und markieren |
| FEA-14 | #15 | Meilensteine: «Verwendet» entfällt, Löschen mit Umstellung |
| FEA-15 | #16 | «Altersguthaben heute» → «per Simulationsstart» |
| FEA-16 | #17 | «Bestände heute» → «per Simulationsstart» |
| FEA-17 | #18 | Liquiditätsreserve nur für die Strategie «Standard» |
| FEA-18 | #19 | Zeitreihen-Erklärungen in ein Info-Popup |
| FEA-19 | #20 | Optimierung: Ziel wählbar |
| FEA-20 | #21 | Variationen ohne feste Obergrenze, Warnung ab 60 s |
| FEA-21 | #22 | Legenden: gestrichelte Linien |
| FEA-22 | #23 | Übersicht: Immobilienverkäufe statt Notverkäufe und Einmalzahlungen |
| FEA-23 | #24 | Berechnete Variationen in «Einzelne Läufe» und «Jahreswerte» wählbar |
| FEA-24 | #25 | Einstellungen (Grenzen AHV-/PK-Bezug, Maximalbetrag 3a) |
| FEA-25 | #31 | Pflicht: Grundstückgewinnsteuer bei Immobilien |
| FEA-26 | #32 | Immobilien: einheitliche Feldreihenfolge |
| FEA-27 | #38 | Test-Agent: Mutationstest statt nur Abdeckung als Ziel |
| FEA-28 | #39 | Abdeckung des geänderten Codes im Pull Request anzeigen |
| FEA-29 | #40 | Nicht erkannte Mutanten im Rechenkern einzeln beurteilen |
| FEA-30 | #41 | Übersichtsseite Testqualität (Stufe 1: Hygiene und Angemessenheit) |
| FEA-31 | #42 | Übersichtsseite Testqualität (Stufe 2: Anforderungsbezug und Ergebnis) |
| FEA-32 | #43 | Mutationstests beschleunigen (Timeout, schwere Tests, Shards) |
| FEA-33 | #47 | Fixe Rate, VPW und Endowment ab erster Pensionierung |
| FEA-34 | #44 | App umbenennen: RetirementPlanner → VivaPlan (im bestehenden Repository, GitHub-Repository am Schluss umbenennen) |
| FEA-35 | #48 | Projektstruktur vereinfachen (Workspace mit crates/, docs/, tools/) |
| FEA-36 | #50 | Indirekte Amortisation nur bis zum AHV-Alter, danach direkt; 3a-Auszahlung beim AHV-Alter |
| FEA-37 | #52 | Säule 3a als eigenes Konto mit Auszahlungsplan, Rest spätestens im AHV-Alter |
| FEA-38 | #51 | Meilensteine AHV-Bezug und Bezug Pensionskasse: immer vorhanden, nicht löschbar, Grenzen und Standard 65 |
| FEA-39 | #53 | Lebenshaltungskosten inklusive Wohnkosten, Simulation mit den echten Wohnkosten |
| FEA-40 | #54 | Cash getrennt vom Portfolio, Strategien erst ab Pensionierung, «Liquidität» heisst «Cash» |
| FEA-41 | #55 | Säule 3a: Auszahlungsplan mit Datum oder Alter, Guthaben anzeigen, Restbetrag vorabfüllen |
| FEA-42 | #56 | Ganzes Projekt mit rustfmt formatieren, Formatierung prüfen, Agentenanleitung anpassen |
| FEA-43 | #58 | Erwerbseinkommen optional mit der Teuerung |
| FEA-44 | #59 | Einstellungen auf macOS im Programmmenü |
| FEA-45 | #61 | Haftungsausschluss: Bestätigung beim ersten Start, Fusszeile in den Ergebnissen, vollständiger Text im Info-Bereich |
| FEA-46 | #62 | App mehrsprachig: Deutsch, Englisch, Italienisch, Französisch (Schweiz), Zahlen immer im Schweizer Format |
| FEA-47 | #63 | Einkommens- und Vermögenssteuer nach Kanton und Gemeinde |
| FEA-48 | #64 | Kapitalbezugssteuer nach Kanton und Gemeinde |
| FEA-49 | #65 | Grundstückgewinnsteuer für alle Kantone |
| FEA-50 | #66 | Steuertarife mit der Teuerung |
| FEA-51 | #67 | Eigener Welt-Aktienindex aus den JST-Daten nach BIP gewichtet, Vergleich mit DMS |
| FEA-52 | #68 | Zeitreihen «Welt (JST)» und Schweizer Staatsanleihen aus SNB und JST |
| FEA-53 | #69 | Anlagestrategie statt Renditewahl, Gewichtung von CH, EU und USA, Reiter «Renditen & Daten» entfällt |
| FEA-54 | #70 | Willkommensdialog beim ersten Start |
| FEA-55 | #71 | Checkliste «Erste Schritte» im Editor |
| FEA-56 | #72 | Ergebnis in Worten in der Übersicht |
| FEA-57 | #73 | Vermögen & Anlage: Quote von Aktien und Obligationen als Info neben den Beständen |
| FEA-58 | #74 | Hinweis zur Cash-Reserve nur einmal, beim ersten Laden einer betroffenen Planung |
| FEA-59 | #76 | Obligationen: kurzfristige Eidgenossen in der Anlagestrategie |
| FEA-60 | #77 | Konzept: VivaPlan in Claude nutzen (MCP) |
| FEA-61 | #78 | Testdateien MSCI.csv und beispiel.rplan durch erfundene Daten ersetzen |
| FEA-62 | #80 | CI: Tests und Abdeckung nur einmal je Push |
| FEA-63 | #81 | Invarianten- und Robustheitstests rechnen nachweislich genug Planungen |
| FEA-64 | #82 | Golden Masters erkennen Änderungen in jedem Monat |
| FEA-65 | #83 | Tests ohne feste Zeitgrenzen und feste Wartezeiten |
| FEA-66 | #84 | Testdoku auf den Stand bringen |
| FEA-67 | #85 | Zufällige Planungen über alle Renditequellen und Entnahmestrategien, mit Verkleinerung |
| FEA-68 | #86 | Fuzz-Ziel für die ganze Rechnung |
| FEA-69 | #87 | Unit-Tests für die Hilfsfunktionen der Oberfläche |
| FEA-70 | #88 | clippy im Workflow |
| FEA-71 | #89 | Oberflächentests nächtlich auf GitHub |
| FEA-72 | #90 | Pictet-Reihe aus App und Repository entfernen |
| FEA-73 | #91 | Hilfe: Zeitreihen selbst beschaffen und als eigene Reihe verwenden |
| FEA-74 | #92 | Test für eigene Konfigurationen ohne feste Dateinamen |
| FEA-75 | #93 | Version 0.8.0 für das neue öffentliche Repository |
| FEA-76 | #94 | Shiller-Originaltabelle nicht im Repository, Bedingungen von SNB und BFS in den Quellen |
| FEA-77 | #97 | Abnahmeliste: Abnahmekriterien aller umgesetzten Issues auf den aktuellen Stand prüfen |
| FEA-78 | #100 | Monte Carlo parametrisch: Aktienparameter je Aktienindex |
| FEA-79 | #101 | Monte Carlo parametrisch: Obligationen und Inflation mit Korrelation je Index |
| FEA-80 | #103 | Renditekriterium der 3-Topf-Strategie bei Simulationen unter 12 Monaten |
| FEA-81 | #105 | Abdeckungsschwelle für die Kommandozeile |
| FEA-82 | #106 | Mutationstests für Kommandozeile und App |
| FEA-83 | #107 | Abbruch-Token je Berechnung |
| FEA-84 | #108 | CSV-Export der Jahreswerte |
| FEA-85 | #109 | Jahreswerte zeigen die Steuer im Steuerjahr |
| FEA-86 | #110 | Vor der Simulation fragen, ob die Planung gespeichert werden soll |
| FEA-87 | #111 | Oberflächentests unabhängig von der Rechnerlast |
| FEA-88 | #113 | Anforderungsbezug erkennt Prüfungen, describe-Blöcke und Nummern mit Null |
| FEA-89 | #114 | Bezugslinie (0 bzw. 100) in den Ergebnisgrafiken dicker zeichnen |
| FEA-90 | #115 | Vergleich der Ergebnisse an das Einzelergebnis angleichen |
| FEA-91 | #118 | Vergleich: Kennzahlen nach Spalte sortieren (auf- und absteigend) |
| FEA-92 | #119 | Name eines Ergebnisses mit Abkürzung von Renditequelle und Strategie |
| FEA-93 | #120 | Vergleich auf 10 Ergebnisse erweitern und begrenzen |
| FEA-94 | #121 | Wahrscheinlichkeit, dass das Geld reicht: Achse 80–100 %, umschaltbar auf 0–100 % |
| FEA-95 | #122 | Vergleich: Aktien, Obligationen und Inflation linear, logarithmisch oder in % |
| FEA-96 | #123 | Vergleich: Unterschiede in der Planung mit Bezeichnungen statt internen Feldnamen |
| FEA-97 | #124 | Nicht erkannte Mutanten aus dem vollen Mutationslauf |
| FEA-98 | #125 | Kopfzeile des Ergebnisses mit dem Namen der Berechnung vor der Anzahl Läufe |
| FEA-100 | #127 | Steuerberechnung konsolidieren |
| ISS-1 | #1 | Steuern 2029→2030 steigen stärker als die Einnahmen (False Positive) |
| ISS-2 | #2 | Kauf nach Verkauf einer Immobilie erfolgt erst Jahre später |
| ISS-3 | #29 | «Neu» setzt die Planung nicht zurück |
| ISS-4 | #30 | Meilenstein «Verkauf Haus» lässt sich nicht löschen |
| ISS-5 | #28 | Immobilien: Verkauf auch bei «Kauf geplant» erfassen |
| ISS-6 | #27 | Beenden mit ungespeicherten Änderungen fragt nicht nach |
| ISS-7 | #33 | Mixed findings |
| ISS-7 | #34 | Optimum «≥ 100 %» bei Anteilen irreführend |
| ISS-8 | #35 | coverage.yml: Tauri-Abhängigkeiten fehlen auf dem Runner |
| ISS-9 | #36 | Einheitliche Mindestabdeckung 80 % je Paket |
| ISS-10 | #37 | Abnahme-Befunde: Jahreswerte, Meilensteine, Texte |
| ISS-11 | #46 | Nächtlicher Fuzz-Job baut für musl und bricht ab |
| ISS-12 | #49 | Optimierung dauert deutlich länger als geschätzt; Laufzeit je Lauf und gleiche Zufallspfade prüfen |
| ISS-13 | #57 | Fehlermeldung am Feld erst beim Verlassen, nicht während der Eingabe |
| ISS-14 | #60 | Zeitreihen-CSV: Absturz bei Jahreszahl ausserhalb des Bereichs (Fuzz parse_csv) |
| ISS-15 | #75 | Rauchtest: Prüfung der Jahreswerte erwartet veraltete Steuerspalten |
| ISS-16 | #79 | Oberflächentests schlagen bei vollem Lauf fehl |
| ISS-17 | #95 | Test «Leere Planung» im Willkommensdialog schlägt fehl (lädt Beispielplanung) |
| ISS-18 | #98 | Steuertabellen und Steuerfuss werden nicht geprüft (negativer Satz ergibt «inf») |
| ISS-19 | #102 | Monte-Carlo-Fall der zufälligen Planungen braucht 11.9 s statt unter 5 s (FEA-67/AK3) |
| ISS-20 | #104 | Rerun der Variation endet nach einem Abbruch mit «Abgebrochen» (rerun_paths ohne eigenen Job) |
| ISS-21 | #112 | Klick ins Menü geht verloren, wenn gleichzeitig eine Vorschau eintrifft |
| ISS-22 | #116 | Mutationslauf bringt den Mac zum Absturz (endlose Schleife füllt den Speicher) |
| ISS-23 | #126 | Vergleich: Endvermögen nach Perzentil in heutiger Kaufkraft zeigt nur sechs Perzentile |
| GEN-01 | #26 | Repository öffentlich machen (Daten- und Plandateien bereinigen) |
| GEN-02 | #96 | README für das öffentliche Repository |
| GEN-03 | #99 | Ordner docs/spezifikation auflösen |

## Labels

Die Labels folgen seit #117 der GitHub-Konvention (englisch, Standard-Labels wo möglich).

| Alt | Neu | Beschreibung |
|---|---|---|
| `feature` | `enhancement` (Standard) | New feature or request |
| `bug` | `bug` (Standard) | Something isn't working |
| `allgemein` | `task` | Project-wide task |
| `engine` | `engine` | Calculation engine |
| `ui` | `ui` | User interface |
| `repo` | `repo` | Repository, tooling and CI |
| `false-positive` | `invalid` (Standard) | Not a bug, closed after review |
| `umgesetzt` | `implemented` | Implemented in main, acceptance pending |
| `gefunden:test` | `found: test` | Bug found by a test |
| `gefunden:abnahme` | `found: acceptance` | Bug found during acceptance |
| `gefunden:nutzung` | `found: usage` | Bug found in use |
