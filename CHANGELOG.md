# Changelog

## Version 1.0.0-beta.2

Zweite Vorabversion von 1.0. Das Windows-Installationspaket `.msi` trägt die Versionsnummer 1.0.0.2.

- **Vier Sprachen (#170–#177):** Oberfläche, Menüs, Dateidialoge, Meldungen und Hilfe gibt es auf Deutsch, Englisch, Französisch und Italienisch. Beim ersten Start gilt die Sprache des Systems, danach die Wahl unter «Ansicht → Sprache». Zahlen und Daten bleiben in jeder Sprache im Schweizer Format. Namen aus dem Katalog der Optimierung und der Zeitreihen sind mitübersetzt.
- **Fachbegriffe und Prüfung der Übersetzungen (#179–#182):** Ein Fachglossar legt die amtlichen Begriffe je Sprache fest (z. B. AVS/LPP, caisse de pension, cassa pensione); eine automatische Prüfung vergleicht alle Übersetzungen mit dem Deutschen. Alle Texte haben in jeder Sprache Platz.
- **Kommandozeile auf Englisch (#176):** `vivaplan` gibt Meldungen und Berichte englisch aus. Die Zahlen sind unverändert.
- **Kapitalbezugssteuer unverheirateter Paare (#150):** Kapitalbezüge von Partnern, die nicht verheiratet sind, werden je Person besteuert und nicht mehr für den Haushalt zusammengerechnet. **Das kann die Ergebnisse solcher Planungen ändern.**
- **Hilfe (#183):** Widersprüche zu Lebenshaltung, Kombinationen und 3a-Einzahlung behoben.
- **Neues App-Icon (#185):** «Gipfelweg – Abendrot», mit eigener Fassung für kleine Grössen und passendem Favicon.
- **Beispielplanung (#184):** Die Beispielplanung erscheint in der Sprache der Oberfläche.
- **Tests und Entwicklung:** Code intern englisch (#165–#169), Netzwerkfreiheit automatisch geprüft (#157), gemeinsame Testfälle der Steuerberechnung für Oberfläche und Rechenkern (#155, #156, #158), Textbreiten auch mit den Schriften unter Linux und Windows geprüft (#182), Beispieldateien englisch benannt und mit festen Zeilenenden (#186), Tests für bisher nicht erkannte Mutanten (#187).

## Version 1.0.0-beta.1

Erste Vorabversion von 1.0. Das Windows-Installationspaket `.msi` trägt die Versionsnummer 1.0.0.1, weil das MSI-Format keine Vorabversion wie «beta.1» kennt; die übrigen Pakete heissen 1.0.0-beta.1.

- **Renditekriterium der 3-Topf-Strategie (#103):** Jeder Renditepfad trägt die Aktienrenditen der 12 Monate vor dem Simulationsstart mit. Das Renditekriterium gilt damit auch in den ersten 12 Monaten über volle 12 Monate. **Das kann die Ergebnisse von Planungen mit 3-Topf-Strategie im ersten Jahr ändern.**
- **Vergleich der Ergebnisse (#115):** Der Vergleich gespeicherter Ergebnisse bietet dieselben Möglichkeiten wie ein einzelnes Ergebnis: nominal oder in heutiger Kaufkraft, alle Grafiken und die Jahreswerte, mit Wahl des Laufs (Perzentil).
- **Bezugslinie in den Grafiken (#114):** Die Linie bei 0 (bzw. bei 100 in der Wertentwicklung) ist in allen Ergebnisgrafiken deutlich hervorgehoben.
- **Menü (#112):** Ein Klick ins Menü links oder in die Prüfliste geht nicht mehr verloren, wenn gleichzeitig eine Vorschau eintrifft.
- **Tests und Entwicklung:** Oberflächentests unabhängig von der Rechnerlast (#111), Übersichtsseite Testqualität mit geprüften Kriterien (#113), Speichergrenze für Testprozesse bei Mutationsläufen (#116).

## Version 0.8.1

- **Kommandozeile:** `vivaplan --version` nennt die Version (#93). Abbruch und Fortschritt gelten je Berechnung (#107, behebt #104); Abdeckungsschwelle und Mutationstests auch für Kommandozeile und App (#105, #106).
- **Monte Carlo parametrisch je Aktienindex (#100):** Die Vorschlagswerte für Rendite, Streuung und Kurssprünge entstehen zur Laufzeit aus der Reihe des gewählten Aktienindex (Welt, Schweiz, S&P 500, «Welt individuell») und liegen nahe am historischen Bootstrap. Von Hand geänderte Werte bleiben beim Wechsel des Index.
- **Monte Carlo parametrisch mit Obligationen und Inflation (#101):** Obligationen und Inflation sind zufällig und mit den Aktien korreliert (Matrix je Kombination, im Reiter «Simulation» sichtbar). **Das ändert die Ergebnisse bestehender parametrischer Planungen;** ihre eigenen Parameter bleiben, beim Öffnen erscheint ein einmaliger Hinweis.
- **Steuern nach Steuerjahr (#109):** Die Jahreswerte zeigen die Einkommens- und Vermögenssteuer in der Zeile des Steuerjahres, für das sie gelten. Bezahlt wird sie weiter im Mai des Folgejahres, Vermögen und Liquidität ändern sich nicht.
- **Speichern vor der Simulation (#110):** Bei ungespeicherten Änderungen fragt die Desktop-App vor der Simulation, ob gespeichert werden soll; abschaltbar in den Einstellungen.

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
