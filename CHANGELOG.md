# Changelog

## Version 1.0.0-beta.1

Erste Vorabversion von 1.0. Das Windows-Installationspaket `.msi` trägt die Versionsnummer 1.0.0.1, weil das MSI-Format keine Vorabversion wie «beta.1» kennt; die übrigen Pakete heissen 1.0.0-beta.1.

- **Renditekriterium der 3-Topf-Strategie (FEA-80):** Jeder Renditepfad trägt die Aktienrenditen der 12 Monate vor dem Simulationsstart mit. Das Renditekriterium gilt damit auch in den ersten 12 Monaten über volle 12 Monate. **Das kann die Ergebnisse von Planungen mit 3-Topf-Strategie im ersten Jahr ändern.**
- **Vergleich der Ergebnisse (FEA-90):** Der Vergleich gespeicherter Ergebnisse bietet dieselben Möglichkeiten wie ein einzelnes Ergebnis: nominal oder in heutiger Kaufkraft, alle Grafiken und die Jahreswerte, mit Wahl des Laufs (Perzentil).
- **Bezugslinie in den Grafiken (FEA-89):** Die Linie bei 0 (bzw. bei 100 in der Wertentwicklung) ist in allen Ergebnisgrafiken deutlich hervorgehoben.
- **Menü (ISS-21):** Ein Klick ins Menü links oder in die Prüfliste geht nicht mehr verloren, wenn gleichzeitig eine Vorschau eintrifft.
- **Tests und Entwicklung:** Oberflächentests unabhängig von der Rechnerlast (FEA-87), Übersichtsseite Testqualität mit geprüften Kriterien (FEA-88), Speichergrenze für Testprozesse bei Mutationsläufen (ISS-22).

## Version 0.8.1

- **Kommandozeile:** `vivaplan --version` nennt die Version (FEA-75). Abbruch und Fortschritt gelten je Berechnung (FEA-83, behebt ISS-20); Abdeckungsschwelle und Mutationstests auch für Kommandozeile und App (FEA-81, FEA-82).
- **Monte Carlo parametrisch je Aktienindex (FEA-78):** Die Vorschlagswerte für Rendite, Streuung und Kurssprünge entstehen zur Laufzeit aus der Reihe des gewählten Aktienindex (Welt, Schweiz, S&P 500, «Welt individuell») und liegen nahe am historischen Bootstrap. Von Hand geänderte Werte bleiben beim Wechsel des Index.
- **Monte Carlo parametrisch mit Obligationen und Inflation (FEA-79):** Obligationen und Inflation sind zufällig und mit den Aktien korreliert (Matrix je Kombination, im Reiter «Simulation» sichtbar). **Das ändert die Ergebnisse bestehender parametrischer Planungen;** ihre eigenen Parameter bleiben, beim Öffnen erscheint ein einmaliger Hinweis.
- **Steuern nach Steuerjahr (FEA-85):** Die Jahreswerte zeigen die Einkommens- und Vermögenssteuer in der Zeile des Steuerjahres, für das sie gelten. Bezahlt wird sie weiter im Mai des Folgejahres, Vermögen und Liquidität ändern sich nicht.
- **Speichern vor der Simulation (FEA-86):** Bei ungespeicherten Änderungen fragt die Desktop-App vor der Simulation, ob gespeichert werden soll; abschaltbar in den Einstellungen.

## Version 0.8.0: erster öffentlicher Stand

VivaPlan ist ein lokales Planungswerkzeug für die Pensionsplanung in der Schweiz. Alle Daten bleiben auf dem eigenen
Rechner; die App nutzt kein Netzwerk.

Funktionen dieses Standes:

- **Planung:** Personen, Einkommen, Vermögen, Ausgaben, Steuern und Meilensteine; Immobilien mit Kauf, Verkauf,
  Hypothek und Grundstückgewinnsteuer (Modelle Zürich und Aargau).
- **Anlagestrategie:** Zeitreihen aus historischen Daten (Shiller, SNB, JST) mit Gewichtung der Anlageklassen; die
  3-Topf-Strategie für Entnahmen.
- **Simulation:** fixe Renditen, historische Verläufe ab einem Startmonat und Monte Carlo (Bootstrap oder
  parametrisch).
- **Ergebnisse:** Zeitleiste, Jahreswerte, Grafiken, gespeicherte Ergebnisse und Parameter zu jedem Lauf.
- **Variationen und Optimierung:** Vergleich mehrerer Werte und Optimierung nach zwei Zielen, zum Beispiel
  «Kein Notverkauf von Immobilien».
- **Kommandozeile:** `vivaplan` für Berechnung, Prüfung und Import alter Konfigurationen.
- **Hilfe:** ein Handbuch mit Suche in der App.
