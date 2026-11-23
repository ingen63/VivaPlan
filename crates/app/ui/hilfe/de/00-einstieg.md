# Einstieg

## @start Was VivaPlan kann
Stichworte: Übersicht, Zweck, Pensionsplanung, Simulation

VivaPlan beantwortet die Frage, die sich vor der Pensionierung stellt: **Reicht das Geld, und wie sicher ist das?**

Die App rechnet Ihre finanzielle Zukunft Monat für Monat durch, vom heutigen Tag bis zum Ende der Planung. Sie kennt die Schweizer Eigenheiten:

- Pensionskasse (2. Säule) mit Umwandlungssatz und Kapitalbezug, AHV (1. Säule) und Säule 3a
- Einkommens-, Vermögens-, Kapitalbezugs- und Grundstückgewinnsteuer
- Eigenheim mit SARON- oder Festhypothek, Tragbarkeitsrechnung der Banken, Miete
- eine zweite Person mit eigener Vorsorge (Ehepaar-Regeln)

Die Zukunft ist unsicher, vor allem die Börsenrenditen. Deshalb rechnet die App nicht nur einen Verlauf, sondern je nach Einstellung **tausende** (Monte Carlo oder alle historischen Startzeitpunkte) und zeigt, in wie vielen davon das Geld reicht. Wie die Renditen entstehen, ist das wichtigste Thema der App: siehe [Renditen & Daten](#s-returns).

Der Aufbau:

1. **Editor:** Sie erfassen die Planung in elf Abschnitten (links). Rechts sehen Sie sofort eine Vorschau des Vermögensverlaufs.
2. **Simulation starten:** Die App rechnet alle Läufe und zeigt die [Ergebnisse](#r-overview).
3. **Optimierung:** Die App sucht die [Entnahmestrategie](#s-optimize), die zu Ihren Zielen passt.
4. **Ergebnisse:** Jede Simulation wird als Ergebnis in der [Liste](#r-list) abgelegt, damit Sie sie später ansehen und vergleichen können.

## @quickstart Erste Planung in zehn Minuten
Stichworte: Anleitung, Beginn, Tutorial, Schritte

Starten Sie mit der Beispielplanung, die beim Öffnen geladen ist, und ändern Sie sie Schritt für Schritt auf Ihre Verhältnisse.

Die Checkliste **«Erste Schritte»** rechts über der Prüfung führt durch dieselben acht Schritte. Ein Klick auf einen Schritt öffnet den Abschnitt, ⓘ die Hilfe dazu. Ein Abschnitt gilt als erledigt, sobald Sie darin einen Wert ändern oder ihn von Hand abhaken; «Rückgängig» nimmt den Haken nicht weg. «Simulation starten» ist erledigt, sobald eine Simulation ohne Abbruch durchgelaufen ist. Mit ✕ blenden Sie die Liste aus, über *Hilfe → Erste Schritte* wieder ein. Sind alle Schritte erledigt, klappt sie zu. Der Fortschritt gilt für Sie auf diesem Gerät, nicht für eine einzelne Planung, und steht nicht in der Planungsdatei.

1. **[Haushalt & Projekt](#s-general):** Name und Geburtsmonat eintragen.
2. **[Meilensteine](#s-milestones):** Bezug Pensionskasse, AHV-Bezug, Start und Ende der Simulation festlegen. Das Ende sollte so spät sein, dass Sie es nicht überleben möchten (zum Beispiel Alter 95).
3. **[Vorsorge](#s-pension):** Aktuelles Pensionskassenguthaben, Umwandlungssatz, Kapitalbezug und AHV-Rente aus Ihren Vorsorgeausweisen eintragen.
4. **[Einnahmen & Ausgaben](#s-flows):** Erwerbseinkommen pro Jahr (wie in der Steuererklärung) und Lebenshaltungskosten pro Monat. Die Kosten brauchen einen realistischen Wert, hier steckt der grösste Hebel.
5. **[Vermögen & Anlage](#s-wealth):** Aktien, Obligationen und Cash per Simulationsstart, dazu der angestrebte Aktienanteil.
6. **[Simulation](#s-returns):** Für eine erste Einschätzung «Monte Carlo · historisch» wählen. Die Indizes (zum Beispiel «Welt (JST)») legen Sie bei der [Anlagestrategie](#f-ret-series) unter «Vermögen & Anlage» fest.
7. **[Steuern](#s-taxes):** Die Sätze der Beispielplanung durch die Ihres Wohnorts ersetzen (Steuerrechner Ihres Kantons).
8. **Simulation starten** (Knopf oben rechts oder ⌘↩). Links neben dem Knopf steht eine grobe Schätzung der **Dauer**. Während der Berechnung zeigt ein **Fortschrittsbalken** den Stand und die geschätzte Restzeit; mit **Abbrechen** stoppen Sie die Rechnung und kehren zur Planung zurück.

Die Prüfung rechts unten zeigt Fehler und Hinweise, noch bevor Sie rechnen. Fehler verhindern die Simulation, Hinweise nicht.

> Tipp: Drücken Sie **F1** in einem Eingabefeld, dann erscheint genau die Hilfe zu diesem Feld.

## @bedienung Bedienung und Tastenkürzel
Stichworte: Menü, Tastatur, speichern, öffnen, rückgängig, F1, Sprache

- **Datei:** Neu (eine **leere Planung mit Standardwerten**: Start im aktuellen Monat, ohne Projektname, eine Person ohne Namen, keine Beträge und Immobilien; Renditen, Steuertabellen und Strategieparameter übernehmen die Standardwerte), Öffnen, Speichern, Speichern unter. Eine Planung ist eine einzelne Datei mit der Endung `.vplan`. Sie lässt sich kopieren und archivieren.
- **Start und Beenden:** Die Desktop-App öffnet beim Start die Planung, die Sie zuletzt geöffnet oder gespeichert haben; der Dateiname steht im Kopf. Gibt es keine, ist die Datei verschwunden oder nicht lesbar, öffnet sie die mitgelieferte **Beispielplanung** (mit Hinweis auf den Dateinamen, ausser beim allerersten Start). Die Beispielplanung gilt als neue, ungespeicherte Planung: «Speichern» fragt nach einem Ort, sie wird nie überschrieben. Beim Beenden mit ungespeicherten Änderungen (Schliessen-Knopf des Fensters, ⌘Q bzw. Ctrl+Q oder Datei → Beenden) fragt die App, ob Sie speichern, verwerfen oder abbrechen wollen. «Abbrechen» lässt die App offen; bricht die Ortsabfrage beim Speichern ab, bleibt sie ebenfalls offen. Gemerkt wird nur der Dateipfad, lokal im Datenordner der App.
- **Speichern vor der Simulation:** Hat die Planung ungespeicherte Änderungen, fragt die Desktop-App vor jeder Simulation, ob Sie speichern wollen: «Speichern» (bei einer neuen Planung mit Ortsabfrage), «Ohne Speichern simulieren» oder «Abbrechen». Die Rückfrage lässt sich in den Einstellungen abschalten.
- **Willkommen:** Beim allerersten Start fragt die App, wie Sie beginnen möchten: **Beispiel erkunden** (lädt die Beispielplanung und öffnet die Hilfe [Was VivaPlan kann](#start)), **Eigene Planung beginnen** (lädt die Beispielplanung als Vorlage, öffnet «Haushalt & Projekt» und setzt die Checkliste [Erste Schritte](#quickstart) zurück) oder **Leere Planung** (wie Datei → Neu). Esc schliesst den Dialog ohne Änderung. Danach erscheint er nicht mehr von selbst; *Hilfe → Willkommen …* öffnet ihn jederzeit, bei ungespeicherten Änderungen mit Rückfrage. Wird die App mit einer Datei gestartet (Doppelklick), erscheint er nicht.
- **Rückgängig / Wiederholen:** ⌘Z und ⇧⌘Z (unter Windows und Linux Ctrl+Z und Ctrl+Shift+Z) machen Eingaben rückgängig.
- **Abschnitte wechseln:** ⌘1 bis ⌘9 und ⌘0 springen zu den Abschnitten, ⌘↑ und ⌘↓ gehen einen weiter oder zurück. In den Ergebnissen wechseln dieselben Tasten die Registerkarte.
- **Hilfe:** **F1** öffnet die Hilfe zum aktuellen Thema (je nachdem, wo Sie gerade sind: Feld, Abschnitt, Ergebnisgrafik). Der Knopf «? Hilfe» und die Zeichen ⓘ neben Feldern tun dasselbe. ⌘⇧? öffnet das Inhaltsverzeichnis, Esc schliesst die Hilfe.
- **Zeitleiste:** Über dem Editor zeigt sie alle Meilensteine. Ein Klick darauf öffnet die Meilensteine. Unter der Linie stehen die **Immobilienereignisse** der Vorschau (grüne Dreiecke für Kauf und Verkauf, orange für Notverkäufe) mit Kurzbeschriftung wie «Kauf Wohnung 09.2027». Bei mehr als 15 Ereignissen erscheinen nur die Symbole, der Text steht dann im Tooltip.
- **Zeitabhängige Werte:** Viele Beträge lassen sich als Konstante oder als Abfolge von Werten erfassen, siehe [Zeitabhängige Werte](#f-timed).
- **Sprache:** Menü *Ansicht → Sprache* oder die [Einstellungen](#f-settings). Der Wechsel wirkt sofort; Planung und Ansicht bleiben. Die Hilfe erscheint in der gewählten Sprache; ist ein Kapitel noch nicht übersetzt, erscheint es deutsch.

Die vollständige Liste der Tastenkürzel steht im Menü *Hilfe → Tastenkürzel* und im [Referenzkapitel](#ref-shortcuts).

## @privacy Datenschutz und lokale Daten
Stichworte: Internet, Cloud, Speicherort, Sicherheit, Netzwerk

Alle Daten bleiben auf Ihrem Rechner. Die App verbindet sich nie mit dem Internet, auch nicht für die Daten, Updates oder Auswertungen.

- **Planungen** liegen dort, wo Sie die `.vplan`-Datei speichern. Der Pfad der zuletzt geöffneten oder gespeicherten Planung steht in der Datei `einstellungen.json` im Datenordner der App.
- **Zeitreihen** (die Datenbibliothek) liegen im Datenordner der App. Unter macOS ist das `~/Library/Application Support/ch.vivaplan-app.desktop/Zeitreihen`.
- **Die Liste der Ergebnisse** liegt im lokalen Speicher der App (nicht in der Planungsdatei). Wer Ihren Benutzer-Account auf diesem Rechner hat, kann sie lesen. Löschen Sie Einträge, die Sie nicht aufbewahren möchten.

Neue Daten der Bibliothek (Shiller, SNB) laden Sie selbst von deren Webseiten herunter und lesen sie über die Knöpfe bei der [Anlagestrategie](#f-ret-series) ein.

## @limits Grenzen des Modells
Stichworte: Annahmen, nicht berücksichtigt, Beratung, Haftung, Todesfall

VivaPlan ist ein **Rechenwerkzeug**, keine Anlage-, Steuer- oder Vorsorgeberatung. Ein Ergebnis gilt nur für die eingegebenen Annahmen. Besonders wichtig:

- **Die Zukunft ist nicht die Vergangenheit.** Historische Renditen sind keine Prognose. Siehe [Fallstricke bei den Renditen](#ret-pitfalls).
- **Steuern sind vereinfacht:** Satztabellen, linear interpoliert, mal Steuerfuss. Abzüge, Progression im Detail und Kantonsbesonderheiten fehlen. Die Ergebnisse zeigen deshalb eine **geschätzte Steuer**, siehe [Wie genau ist die Steuer?](#tax-accuracy).
- **Nicht gerechnet werden:** Todesfälle und Hinterlassenenrenten, Erbschaften ausser als einmaliger Betrag, Pflegekosten (ausser Sie erfassen sie als Ausgaben), Gesetzesänderungen, Anlagekosten und Gebühren, Zinsen auf Cash.
- **Immobilien** sind vereinfacht: Marktwert mit Teuerung, Hypotheken in Tranchen, Notverkauf nach einfacher Regel.
- **Optimierte Strategien** gelten nur für die Eingaben, mit denen sie berechnet wurden, siehe [Fallstricke der Optimierung](#o-pitfalls).

Prüfen Sie wichtige Entscheide immer mit Fachpersonen. Den vollständigen [Haftungsausschluss](#disclaimer) finden Sie im nächsten Abschnitt.

## @disclaimer Haftungsausschluss
Stichworte: Haftung, Gewähr, Disclaimer, Beratung, Verantwortung, Recht, Hinweis

Beim ersten Start bestätigen Sie eine Kurzfassung dieses Hinweises. Ändert sich der Text, fragt die App erneut. Alle Ergebnisseiten tragen unten den Vermerk «Modellrechnung ohne Gewähr. Keine Steuer- oder Anlageberatung.».

**1. Kostenloses Angebot.** VivaPlan wird aus der Schweiz unentgeltlich zur Verfügung gestellt. Es besteht kein Anspruch auf Verfügbarkeit, Updates, Fehlerbehebung oder Support.

**2. Keine Beratung.** Die App ist ein Rechenhilfsmittel für die persönliche Planung. Sie ist **keine Steuer- oder Anlageberatung**. Sie bietet keine Anlage-, Finanz-, Steuer-, Vorsorge- oder Rechtsberatung und gibt keine Empfehlungen zum Kauf oder Verkauf von Finanzinstrumenten oder Immobilien.

**3. Modellrechnungen ohne Gewähr.** Alle Ergebnisse beruhen auf Ihren Eingaben sowie auf vereinfachten Annahmen zu Rendite, Kosten, Teuerung, Steuern und Lebenserwartung. Tatsächliche Entwicklungen können erheblich abweichen. Steuerberechnungen sind Schätzungen und bilden nicht alle kantonalen und kommunalen Regeln, Abzüge und Gesetzesänderungen ab. Für die Richtigkeit, Vollständigkeit und Aktualität der Berechnungen wird keine Gewähr übernommen.

**4. Eigene Verantwortung.** Entscheidungen, die Sie auf Grundlage der App treffen, liegen allein in Ihrer Verantwortung. Vor wichtigen finanziellen Entscheidungen, etwa zu Kapitalbezug, Hausverkauf oder Entnahmestrategie, sollten Sie eine qualifizierte Fachperson beiziehen.

**5. Haftung.** Die Haftung für Schäden jeder Art, insbesondere für Vermögensverluste, entgangenen Gewinn, Datenverlust oder Folgeschäden aus der Nutzung oder Nichtnutzung der App, wird ausgeschlossen, soweit gesetzlich zulässig. Ausgenommen ist die Haftung für Vorsatz und grobe Fahrlässigkeit.

**6. Daten.** Die App speichert Ihre Eingaben lokal auf Ihrem Gerät und überträgt keine Daten (siehe [Datenschutz und lokale Daten](#privacy)). Die Sicherung dieser Daten liegt in Ihrer Verantwortung.

**7. Anwendbares Recht.** Es gilt schweizerisches Recht.

Stand: 22. November 2026
