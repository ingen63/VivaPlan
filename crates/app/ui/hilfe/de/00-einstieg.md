# Einstieg

## @start Was VivaPlan kann
Stichworte: Übersicht, Zweck, Pensionsplanung, Simulation

VivaPlan beantwortet die Frage, die sich vor der Pensionierung stellt: **Reicht das Geld, und wie sicher ist das?**

> Video: [VivaPlan in 2 Minuten](video:2min) zeigt den Überblick (öffnet den Browser).

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

> Video: [VivaPlan in 10 Minuten](video:10min) führt durch die erste Planung (öffnet den Browser).

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
Stichworte: Menü, Tastatur, speichern, öffnen, rückgängig, F1, Sprache, zuletzt geöffnet, unterstützen, Spende, Kaffee, Video, Einführungsvideo

- **Datei:** Neu (eine **leere Planung mit Standardwerten**: Start im aktuellen Monat, ohne Projektname, eine Person ohne Namen, keine Beträge und Immobilien; Renditen, Steuertabellen und Strategieparameter übernehmen die Standardwerte), Öffnen, Zuletzt geöffnet, Speichern, Speichern unter. Eine Planung ist eine einzelne Datei mit der Endung `.vplan`. Sie lässt sich kopieren und archivieren.
- **Zuletzt geöffnet:** Das Untermenü unter Datei → Öffnen zeigt die bis zu 10 Planungen, die Sie zuletzt geöffnet oder gespeichert haben, die neueste zuoberst (Dateiname ohne `.vplan`; bei gleichen Namen mit dem Ordner). Ein Klick öffnet die Planung wie «Öffnen …»; bei ungespeicherten Änderungen fragt die App zuerst nach. Gibt es die Datei nicht mehr, meldet die App das und entfernt den Eintrag. «Liste löschen» leert die Liste; ist sie leer, steht dort «Keine Einträge».
- **Start und Beenden:** Die Desktop-App öffnet beim Start die Planung, die Sie zuletzt geöffnet oder gespeichert haben; der Dateiname steht im Kopf. Gibt es keine, ist die Datei verschwunden oder nicht lesbar, öffnet sie die mitgelieferte **Beispielplanung** (mit Hinweis auf den Dateinamen, ausser beim allerersten Start). Die Beispielplanung gilt als neue, ungespeicherte Planung: «Speichern» fragt nach einem Ort, sie wird nie überschrieben. Beim Beenden mit ungespeicherten Änderungen (Schliessen-Knopf des Fensters, ⌘Q bzw. Ctrl+Q oder Datei → Beenden) fragt die App, ob Sie speichern, verwerfen oder abbrechen wollen. «Abbrechen» lässt die App offen; bricht die Ortsabfrage beim Speichern ab, bleibt sie ebenfalls offen. Gemerkt wird nur der Dateipfad, lokal im Datenordner der App.
- **Speichern vor der Simulation:** Hat die Planung ungespeicherte Änderungen, fragt die Desktop-App vor jeder Simulation, ob Sie speichern wollen: «Speichern» (bei einer neuen Planung mit Ortsabfrage), «Ohne Speichern simulieren» oder «Abbrechen». Die Rückfrage lässt sich in den Einstellungen abschalten.
- **Willkommen:** Beim allerersten Start fragt die App, wie Sie beginnen möchten: **Beispiel erkunden** (lädt die Beispielplanung und öffnet die Hilfe [Was VivaPlan kann](#start)), **Eigene Planung beginnen** (lädt die Beispielplanung als Vorlage, öffnet «Haushalt & Projekt» und setzt die Checkliste [Erste Schritte](#quickstart) zurück) oder **Leere Planung** (wie Datei → Neu). Esc schliesst den Dialog ohne Änderung. Danach erscheint er nicht mehr von selbst; *Hilfe → Willkommen …* öffnet ihn jederzeit, bei ungespeicherten Änderungen mit Rückfrage. Wird die App mit einer Datei gestartet (Doppelklick), erscheint er nicht.
- **Rückgängig / Wiederholen:** ⌘Z und ⇧⌘Z (unter Windows und Linux Ctrl+Z und Ctrl+Shift+Z) machen Eingaben rückgängig.
- **Abschnitte wechseln:** ⌘1 bis ⌘9 und ⌘0 springen zu den Abschnitten, ⌘↑ und ⌘↓ gehen einen weiter oder zurück. In den Ergebnissen wechseln dieselben Tasten die Registerkarte.
- **Hilfe:** **F1** öffnet die Hilfe zum aktuellen Thema (je nachdem, wo Sie gerade sind: Feld, Abschnitt, Ergebnisgrafik). Der Knopf «? Hilfe» und die Zeichen ⓘ neben Feldern tun dasselbe. ⌘⇧? öffnet das Inhaltsverzeichnis, Esc schliesst die Hilfe.
- **Zeitleiste:** Über dem Editor zeigt sie alle Meilensteine. Ein Klick darauf öffnet die Meilensteine. Unter der Linie stehen die **Immobilienereignisse** der Vorschau (grüne Dreiecke für Kauf und Verkauf, orange für Notverkäufe) mit Kurzbeschriftung wie «Kauf Wohnung 09.2027». Bei mehr als 15 Ereignissen erscheinen nur die Symbole, der Text steht dann im Tooltip.
- **Zeitabhängige Werte:** Viele Beträge lassen sich als Konstante oder als Abfolge von Werten erfassen, siehe [Zeitabhängige Werte](#f-timed).
- **Sprache:** Menü *Ansicht → Sprache* oder die [Einstellungen](#f-settings). Der Wechsel wirkt sofort; Planung und Ansicht bleiben. Die Hilfe erscheint in der gewählten Sprache; ist ein Kapitel noch nicht übersetzt, erscheint es deutsch.
- **Einführungsvideos:** *Hilfe → Einführungsvideos …* zeigt die Videos «VivaPlan in 2 Minuten» und «VivaPlan in 10 Minuten»; der Willkommensdialog hat den Link «VivaPlan in 2 Minuten ansehen». Ein Klick öffnet das Video in der eingestellten Sprache in Ihrem Browser (Internet Archive).
- **Mit KI verbinden:** *Hilfe → Mit KI verbinden …* zeigt, wie Sie VivaPlan in Claude Desktop, Claude Code, der ChatGPT-Desktop-App oder LM Studio einrichten, je mit dem Ort der Einrichtung und einer Zeile zum Kopieren ([KI-Programme verbinden](#ref-mcp)). VivaPlan richtet dabei nichts selbst ein.
- **VivaPlan unterstützen:** VivaPlan ist kostenlos. Wenn Sie den Autor freiwillig mit einem Kaffee unterstützen möchten: *Hilfe → VivaPlan unterstützen…*, die Zeile «☕ VivaPlan unterstützen» unten in der Navigation oder der Link im Fenster «Über VivaPlan» öffnen die Seite buymeacoffee.com/vivaplan in Ihrem Browser. Die App selbst lädt dabei nichts aus dem Internet.

Die vollständige Liste der Tastenkürzel steht im Menü *Hilfe → Tastenkürzel* und im [Referenzkapitel](#ref-shortcuts).

## @beta Beta-Token und Aktivierung
Stichworte: Beta, Token, Aktivierung, aktivieren, Einlösefrist, Closed Beta, Testperson, Update

Während der geschlossenen Beta lässt sich VivaPlan erst nach einer **einmaligen Aktivierung** benutzen. Dafür erhalten Sie ein persönliches Beta-Token, eine Zeile, die mit `VP1.beta.` beginnt. Beim ersten Start zeigt die App nur den Dialog «Beta-Token eingeben»; fügen Sie dort das ganze Token ein und klicken Sie auf *Aktivieren*. Zeilenumbrüche und Leerzeichen aus einer E-Mail stören nicht.

- **Einmalig:** Die Aktivierung gilt für diesen Rechner und Ihr Benutzerkonto. Danach fragt VivaPlan nie mehr, auch nicht nach einem Update auf eine neuere Version.
- **Einlösefrist:** Das Datum im Token sagt nur, bis wann es sich eingeben lässt. Ist die Frist vorbei, lehnt VivaPlan das Token ab; bitten Sie dann um ein neues. Eine schon aktivierte Installation läuft auch nach der Frist weiter.
- **Ohne Internet:** VivaPlan prüft das Token auf Ihrem Rechner. Es sendet nichts.
- **KI-Paket:** App und KI-Anbindung (`vivaplan mcp`) teilen die Aktivierung. Ist die App aktiviert, braucht das Paket für Claude Desktop kein Token. Sonst geben Sie das Token beim Installieren des Pakets im Feld «Beta token» ein; das aktiviert auch die App ([KI-Programme verbinden](#ref-mcp)).
- **Anzeige:** *Hilfe → Über VivaPlan* zeigt Ihr Kürzel und den Tag der Aktivierung.

Die Aktivierung liegt in der Datei `beta-aktivierung.json` im Datenordner der App (siehe [Datenschutz](#privacy)). Ab Version 1.0.0 braucht VivaPlan kein Token mehr.

## @privacy Datenschutz und lokale Daten
Stichworte: Internet, Cloud, Speicherort, Sicherheit, Netzwerk, Updates

Berechnen und Speichern laufen ganz auf Ihrem Rechner, ohne Internet; die App lädt keine Daten und sendet keine Auswertungen. Daten verlassen den Rechner nur, wenn Sie selbst eine KI anbinden und nutzen (siehe den Punkt zu KI-Programmen unten). **Die App selbst verbindet sich nur für die Suche nach Updates mit dem Internet, und nur mit Ihrer Zustimmung:** Beim ersten Start fragt VivaPlan, ob es beim Start nach neuen Versionen suchen darf. Mit «Ja» fragt die App die Download-Seite von VivaPlan auf GitHub ab; übertragen werden nur die Version von VivaPlan und das Betriebssystem (und wie bei jedem Internetzugriff die IP-Adresse), nie Planungsdaten. Mit «Nein» bleibt die App ganz ohne Internet. Die Wahl lässt sich jederzeit ändern ([Einstellungen](#f-settings), Reiter «Allgemein»). Nur zwei Arten von Links öffnen Ihren Browser: die [Einführungsvideos](#start) und der Spendenlink «VivaPlan unterstützen» ([Bedienung](#bedienung)). Die Adressen stehen fest in der App; die App selbst lädt dabei nichts.

- **Planungen** liegen dort, wo Sie die `.vplan`-Datei speichern. Die Pfade der zuletzt geöffneten oder gespeicherten Planungen (Liste «Zuletzt geöffnet») stehen in der Datei `einstellungen.json` im Datenordner der App.
- **Zeitreihen** (die Datenbibliothek) liegen im Datenordner der App. Unter macOS ist das `~/Library/Application Support/ch.vivaplan-app.desktop/Zeitreihen`.
- **Die Liste der Ergebnisse** liegt im lokalen Speicher der App (nicht in der Planungsdatei). Wer Ihren Benutzer-Account auf diesem Rechner hat, kann sie lesen. Löschen Sie Einträge, die Sie nicht aufbewahren möchten.
- **KI-Programme:** Verbinden Sie VivaPlan mit einer KI ([KI-Programme verbinden](#ref-mcp)), liest die KI über das Programm `vivaplan mcp` Daten Ihrer Planung. Mit einer Cloud-KI (zum Beispiel Claude oder ChatGPT) gehen diese Daten an deren Anbieter. Mit einem lokalen Modell (zum Beispiel in LM Studio) bleiben sie auf Ihrem Rechner. VivaPlan selbst sendet nichts.

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
