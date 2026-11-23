# Changelog

## Version 1.0.0-beta.3

Dritte Vorabversion von 1.0. Das Windows-Installationspaket `.msi` trägt die Versionsnummer 1.0.0.3; Installationspakete gibt es vorerst nur für macOS (#202).

### Kann Ergebnisse ändern

- **Steuern nach Kanton und Gemeinde (#63, #64, #65, #66):** Einkommens-, Vermögens- und Kapitalbezugssteuer rechnen im Modus «Wohnort» nach Kanton und Gemeinde, die Grundstückgewinnsteuer nach dem Kanton der Immobilie. Tarife wachsen mit der Teuerung, der Zweitverdienerabzug wirkt für Erwerbstätige.
- **Abzüge und Wohnort (#131, #151, #152, #228–#232):** Abzüge vom Einkommen mit Vorschlag des Kantons oder eigenen Zeilen, zusätzliche und einmalige Abzüge, Abzüge vom Vermögen, Steuerwert je Immobilie, Erwerbseinkommen als Jahresbetrag, zeitabhängiger Wohnort und grosse Säule 3a für Personen ohne Pensionskasse.
- **AHV (#190, #191, #198):** Die 13. AHV-Rente wird im Dezember ausbezahlt und zählt im Netto-Bedarf der 3-Topf-Strategie. Vorbezug und Aufschub kürzen bzw. erhöhen die Rente; der Plafond für Ehepaare gilt gesetzeskonform.
- **Immobilien und Strategien (#199, #204, #239):** Nicht tragbare Erneuerungen bei «Immobilien nie verkaufen» zählen; die Grenzen der Lebenshaltung gelten bei VPW und Endowment auch im Vergleich und in der Optimierung; Leitplanken ohne liquides Vermögen passen nichts an.
- **Erwerbseinkommen (#236):** Auf Wunsch endet das Erwerbseinkommen mit dem Bezug der Pensionskasse.

### Neu, geändert, behoben

- **Strategien vergleichen (#196, #197, #200, #201, #205):** Strategien lassen sich ohne Optimierung vergleichen, Ergebnisse der Optimierung speichern und wieder öffnen; Schalter «Immobilien nie verkaufen»; Hinweis, wenn die Untergrenze die Ausgaben bestimmt; Ergebnistabelle ohne waagrechtes Scrollen.
- **Dunkles Theme (#220, #235):** Einstellung «Darstellung» mit hellem und dunklem Theme; besserer Kontrast im Menü.
- **Bedienung (#143, #227, #233, #234, #241):** Steuern als Schätzung gekennzeichnet, zuletzt geöffnete Planungen im Menü «Datei», Tragbarkeit in den Einstellungen, lesbare Meldungen des Rechenkerns, übersichtlichere Karte «Abzüge vom Einkommen» mit Text für den Zweitverdienerabzug.
- **Einführungsvideos (#240, #242):** «VivaPlan in 2 Minuten» und «in 10 Minuten» in vier Sprachen, aus Menü «Hilfe», Willkommensdialog und Hilfe; sie öffnen den Browser.
- **Unterstützen (#237):** Freiwilliger Spendenlink «Buy me a coffee» im Menü «Hilfe», in der Navigation und unter «Über VivaPlan»; er öffnet den Browser.
- **Kommandozeile für KIs (#207, #215):** JSON-Ausgabe und Werte per Feldpfad ändern; Renditepfade werden für mehrere Varianten wiederverwendet (schneller).

### Tests und Entwicklung

- Golden-Zahlen im Push-Lauf verglichen (#195), Wertänderung der Liegenschaften unabhängig nachgerechnet (#194), Anforderungsbezug der Tests erweitert (#192).
- Robustheitstest aller Rechenbefehle (#221), Funde der Eigenschaftstests als Regressionstests (#223), Rust-Tests unter Windows (#225), übersprungene Tests als Fehler (#224), Laufzeitgrenze je Test (#226), Regeln für den Test-Agenten (#222).
- Release Notes in vier Sprachen mit sprachneutralem Titel (#188); mutants-diff wöchentlich (#202).

## Version 1.0.0-beta.2

Zweite Vorabversion von 1.0. Das Windows-Installationspaket `.msi` trägt die Versionsnummer 1.0.0.2.

### Kann Ergebnisse ändern

- **Kapitalbezugssteuer unverheirateter Paare (#150):** Kapitalbezüge von Partnern, die nicht verheiratet sind, werden je Person besteuert und nicht mehr für den Haushalt zusammengerechnet. Das kann die Ergebnisse solcher Planungen ändern.

### Neu, geändert, behoben

- **Vier Sprachen (#170–#177):** Oberfläche, Menüs, Dateidialoge, Meldungen und Hilfe gibt es auf Deutsch, Englisch, Französisch und Italienisch. Beim ersten Start gilt die Sprache des Systems, danach die Wahl unter «Ansicht → Sprache». Zahlen und Daten bleiben in jeder Sprache im Schweizer Format. Namen aus dem Katalog der Optimierung und der Zeitreihen sind mitübersetzt.
- **Fachbegriffe und Prüfung der Übersetzungen (#179–#182):** Ein Fachglossar legt die amtlichen Begriffe je Sprache fest (z. B. AVS/LPP, caisse de pension, cassa pensione); eine automatische Prüfung vergleicht alle Übersetzungen mit dem Deutschen. Alle Texte haben in jeder Sprache Platz, auch mit den Schriften unter Linux und Windows.
- **Kommandozeile auf Englisch (#176):** `vivaplan` gibt Meldungen und Berichte englisch aus. Die Zahlen sind unverändert.
- **Neues App-Icon (#185):** «Gipfelweg – Abendrot», mit eigener Fassung für kleine Grössen und passendem Favicon.
- **Beispielplanung (#184):** Die Beispielplanung erscheint in der Sprache der Oberfläche.
- **Hilfe (#183):** Widersprüche zu Lebenshaltung, Kombinationen und 3a-Einzahlung behoben.

### Tests und Entwicklung

- Code intern englisch (#165–#169), Netzwerkfreiheit automatisch geprüft (#157), gemeinsame Testfälle der Steuerberechnung für Oberfläche und Rechenkern (#155, #156, #158).
- Beispieldateien englisch benannt und mit festen Zeilenenden (#186), Tests für bisher nicht erkannte Mutanten (#187).
- Release Notes in vier Sprachen mit automatischer Prüfung (#188).

## Version 1.0.0-beta.1

Erste Vorabversion von 1.0. Das Windows-Installationspaket `.msi` trägt die Versionsnummer 1.0.0.1, weil das MSI-Format keine Vorabversion wie «beta.1» kennt; die übrigen Pakete heissen 1.0.0-beta.1.

### Kann Ergebnisse ändern

- **Renditekriterium der 3-Topf-Strategie (#103):** Jeder Renditepfad trägt die Aktienrenditen der 12 Monate vor dem Simulationsstart mit. Das Renditekriterium gilt damit auch in den ersten 12 Monaten über volle 12 Monate. Das kann die Ergebnisse von Planungen mit 3-Topf-Strategie im ersten Jahr ändern.

### Neu, geändert, behoben

- **Vergleich der Ergebnisse (#115):** Der Vergleich gespeicherter Ergebnisse bietet dieselben Möglichkeiten wie ein einzelnes Ergebnis: nominal oder in heutiger Kaufkraft, alle Grafiken und die Jahreswerte, mit Wahl des Laufs (Perzentil).
- **Bezugslinie in den Grafiken (#114):** Die Linie bei 0 (bzw. bei 100 in der Wertentwicklung) ist in allen Ergebnisgrafiken deutlich hervorgehoben.
- **Menü (#112):** Ein Klick ins Menü links oder in die Prüfliste geht nicht mehr verloren, wenn gleichzeitig eine Vorschau eintrifft.

### Tests und Entwicklung

- Oberflächentests unabhängig von der Rechnerlast (#111), Übersichtsseite Testqualität mit geprüften Kriterien (#113), Speichergrenze für Testprozesse bei Mutationsläufen (#116).

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
