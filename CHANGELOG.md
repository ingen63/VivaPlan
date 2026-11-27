# Changelog

## Version 1.0.0-beta.3

Dritte Vorabversion von 1.0. Sie fasst die nicht veröffentlichten Vorabversionen 1.0.0-beta.4 bis 1.0.0-beta.6 zusammen und enthält alle Änderungen seither. Das Windows-Installationspaket `.msi` trägt die Versionsnummer 1.0.0.3; Installationspakete gibt es für macOS, Windows und Linux. Ab dieser Version sucht die App auf Wunsch selbst nach neuen Versionen.

### Kann Ergebnisse ändern

- **Teuerung (#306, #279):** Beträge, die in der Realität mit der Teuerung wachsen, wachsen immer mit der Teuerung des Pfades; Ausnahmen für Stresstests unter «Experte». Die Teuerung im parametrischen Modell beginnt 1950, das Startjahr ist einstellbar. Bestehende Planungen rechnen wie bisher.
- **AHV (#284, #280):** Die AHV-Rente folgt alle zwei Jahre dem Mischindex, auch vor dem Bezug, mit der neuen Annahme «Reallohnsteigerung»; nominal sinkt sie nie unter den Vorjahreswert.
- **Hypothekarzinsen (#307):** SARON und erneuerte Festhypotheken folgen dem Zins- und Teuerungsumfeld des Pfades; die Margen stehen in den Einstellungen.
- **Steuerdaten aus Gesetzen (#287–#292, #296, #297):** Tarife, Abzüge und Regeln von Bund und Kantonen stammen aus Gesetzen und Verordnungen statt aus dem ESTV-Steuerrechner; das Wallis rechnet mit Indexierung und der Ermässigung für Verheiratete.
- **Neue Planungen (#303):** Sie beginnen mit Aktien «Welt (JST)» und kurzfristigen Obligationen.
- **Wohnanteil der Lebenshaltungskosten (#264):** Der abgezogene Wohnanteil umfasst Miete, Zins und Nebenkosten, nicht mehr die Amortisation. Ältere Planungen werden beim Öffnen angepasst (mit Hinweis); ihr Ergebnis bleibt gleich.
- **Perzentile (#265):** Die Ergebnisse öffnen in heutiger Kaufkraft; die Perzentile sind nach der gewählten Darstellung (nominal oder real) geordnet.
- **Steuerbarer Ertrag des Vermögens (#149):** Ein Satz pro Jahr für Aktien und Obligationen zählt zum steuerbaren Einkommen. Neue Planungen beginnen mit 2 % und 1 %, bestehende mit 0 %.
- **Ohne Pensionskasse (#253):** Der Schalter steht neu in der Karte «Pensionskasse» im Abschnitt Vorsorge. Ist er gesetzt, rechnet die Pensionskasse dieser Person mit 0 (kein Guthaben, keine Beiträge, keine Rente, kein Kapitalbezug); die erfassten Werte bleiben gespeichert, die Felder sind gesperrt.
- **Optimierung (#248):** Bei allen Verlaufsformen bleibt der Verlauf vor der Pensionierung erhalten; der optimierte Wert gilt ab der Pensionierung.
- **Verheiratet mit einer Person (#250):** «Verheiratet» gilt auch ohne zweite Person; dann rechnen die Steuern mit Tarifen und Abzügen für Verheiratete. Ältere Planungen mit einer Person werden beim Öffnen als unverheiratet übernommen und rechnen wie bisher (Hinweis); neue Planungen beginnen unverheiratet.
- **Steuern nach Kanton und Gemeinde (#63, #64, #65, #66):** Einkommens-, Vermögens- und Kapitalbezugssteuer rechnen im Modus «Wohnort» nach Kanton und Gemeinde, die Grundstückgewinnsteuer nach dem Kanton der Immobilie. Tarife wachsen mit der Teuerung, der Zweitverdienerabzug wirkt für Erwerbstätige.
- **Abzüge und Wohnort (#131, #151, #152, #228–#232):** Abzüge vom Einkommen mit Vorschlag des Kantons oder eigenen Zeilen, zusätzliche und einmalige Abzüge, Abzüge vom Vermögen, Steuerwert je Immobilie, Erwerbseinkommen als Jahresbetrag, zeitabhängiger Wohnort und grosse Säule 3a für Personen ohne Pensionskasse.
- **AHV (#190, #191, #198):** Die 13. AHV-Rente wird im Dezember ausbezahlt und zählt im Netto-Bedarf der 3-Topf-Strategie. Vorbezug und Aufschub kürzen bzw. erhöhen die Rente; der Plafond für Ehepaare gilt gesetzeskonform.
- **Immobilien und Strategien (#199, #204, #239):** Nicht tragbare Erneuerungen bei «Immobilien nie verkaufen» zählen; die Grenzen der Lebenshaltung gelten bei VPW und Endowment auch im Vergleich und in der Optimierung; Leitplanken ohne liquides Vermögen passen nichts an.
- **Erwerbseinkommen (#236):** Auf Wunsch endet das Erwerbseinkommen mit dem Bezug der Pensionskasse.

### Neu, geändert, behoben

- **Säule 3a (#300, #301, #310):** Bezug in Tranchen ab 60 bis 65 Jahren und Vorbezug für Wohneigentum in jedem Alter.
- **Automatische Updates (#309):** Beim ersten Start fragt die App, ob sie nach neuen Versionen suchen darf; ohne Zustimmung bleibt sie ganz ohne Netzwerk. Updates sind signiert.
- **Einstellungen in Reitern (#308):** Allgemein, Vorsorge, Simulation und Immobilien.
- **Steuern (#143, #281):** Steuerjahr, Quelle und Herkunft der Abzüge sind überall sichtbar, die Prüfung warnt bei alten Steuerdaten; das steuerbare Einkommen steht als Spalte neben den Steuern.
- **KI-Anbindung (#302, #304, #298):** Eine neue Planung lässt sich mit einem Dateinamen oder `~/` speichern; die KI kennt die Bezeichnungen der App für alle Auswahlfelder; Hinweis auf Inkognito-Chats und Datenschutz.
- **AHV-Rente in der Hilfe (#282):** erklärt als erwartete Rente mit 65.
- **Beta-Token (#270):** App und KI-Paket werden einmalig mit einem Beta-Token aktiviert.
- **Pflegeheim (#271):** Eigener Baustein mit Eintrittsalter, Kosten und Verkauf der Wohnung; Hinweis, wenn nach dem Verkauf der letzten Immobilie keine Wohnkosten mehr anfallen.
- **Steuern (#272–#275):** Neue Planungen rechnen mit den Steuern am Wohnort; der Abschnitt zeigt zuerst Gemeinde und Konfession, die übrigen Angaben als Annahmen, Einzelheiten unter «Experte». Abzüge lassen sich aus der letzten Steuerrechnung übernehmen, der Steuerwert der Immobilien hat eine Vorgabe in Prozent des Marktwerts, ein Einkauf in die Pensionskasse lässt sich direkt als Abzug erfassen.
- **Abzüge vom Vermögen (#249)** übersichtlich wie die Abzüge vom Einkommen.
- **Tragbarkeit (#266):** Wahlweise mit Vermögensverzehr und tieferer Belehnung ab der Pensionierung.
- **KI-Anbindung (#263, #269):** Der MCP-Server öffnet und speichert Planungen nur im freigegebenen Planordner, wahlweise nur lesend; eine neue Planung beginnt leer und die KI fragt die fehlenden Angaben ab.
- **README (#267, #277)** in vier Sprachen: Die App ist zu 100 % lokal, nur die KI-Anbindung sendet Daten an die gewählte KI.
- **Darstellung (#220):** Auswahl «Darstellung» im Einstellungsdialog mit Hintergrund des Themas.
- **VivaPlan mit einer KI nutzen (#208–#214, #216, #219):** Der MCP-Server `vivaplan mcp` öffnet, ändert, rechnet und vergleicht Planungen für Claude, ChatGPT, LM Studio und andere KIs, mit Fortschritt, Abbruch und Bestätigung langer Rechnungen. Menü «Hilfe» › «Mit KI verbinden» zeigt die Einrichtung; für Claude Desktop gibt es ein Ein-Klick-Paket `.mcpb` (#213). Daten gehen nur an die KI, die Sie selbst anbinden.
- **Verlaufsformen und Kennlinien (#217, #218, #243):** Werte können linear, nach Tabelle oder mit Faktor verlaufen und im Editor bearbeitet werden; Kennlinien machen Parameter vom Zustand abhängig.
- **Kommandozeile im Installationspaket (#206):** `vivaplan` nutzt denselben Datenordner wie die App.
- **Darstellung (#244, #255, #257, #259):** Zahlenfelder und die Auswahl «Steuerwert» werden in allen Sprachen nicht mehr abgeschnitten.
- **Steuerbares Vermögen in der Karte «Abzüge vom Vermögen» (#249):** Hinweis mit dem geschätzten steuerbaren Vermögen des ersten Steuerjahres und dem Anteil der Liegenschaften, Warnung bei fehlendem Steuerwert; die Details setzen es je Steuerjahr zusammen wie die Steuererklärung und bieten das Feld «Steuerwert» je Liegenschaft.
- **Eingabehilfe aus der Veranlagung (#251):** Neues Feld «davon Säule 3a laut Veranlagung», damit die Säule 3a mit «Einzahlungen in die Säule 3a abziehen» nicht doppelt abgezogen wird; Hinweis, solange das Feld leer ist.
- **Strategien vergleichen (#196, #197, #200, #201, #205):** Strategien lassen sich ohne Optimierung vergleichen, Ergebnisse der Optimierung speichern und wieder öffnen; Schalter «Immobilien nie verkaufen»; Hinweis, wenn die Untergrenze die Ausgaben bestimmt; Ergebnistabelle ohne waagrechtes Scrollen.
- **Dunkles Theme (#220, #235):** Einstellung «Darstellung» mit hellem und dunklem Theme; besserer Kontrast im Menü.
- **Bedienung (#143, #227, #233, #234, #241):** Steuern als Schätzung gekennzeichnet, zuletzt geöffnete Planungen im Menü «Datei», Tragbarkeit in den Einstellungen, lesbare Meldungen des Rechenkerns, übersichtlichere Karte «Abzüge vom Einkommen» mit Text für den Zweitverdienerabzug.
- **Einführungsvideos (#240, #242):** «VivaPlan in 2 Minuten» und «in 10 Minuten» in vier Sprachen, aus Menü «Hilfe», Willkommensdialog und Hilfe; sie öffnen den Browser.
- **Unterstützen (#237):** Freiwilliger Spendenlink «Buy me a coffee» im Menü «Hilfe», in der Navigation und unter «Über VivaPlan»; er öffnet den Browser.
- **Kommandozeile für KIs (#207, #215):** JSON-Ausgabe und Werte per Feldpfad ändern; Renditepfade werden für mehrere Varianten wiederverwendet (schneller).

### Tests und Entwicklung

- Schnellere und stabilere Tests (#226, #305, #311), Release Notes und Tests unter Windows (#278), veraltete Verweise auf die ESTV nachgeführt (#286).
- Linux-Paket enthält `vivaplan` wieder (#261), Speicherbedarf grosser Rechnungen gesenkt (#249), Clippy-Regel (#260), Messung des Steuereinflusses für weitere Haushalte (#276).
- Oberflächentests nachgeführt (#246, #247), Zeitgrenze im MCP-Test unabhängig von der Rechnergeschwindigkeit (#258).
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
