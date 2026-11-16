# VivaPlan – Beschreibung

## Kurzfassung (GitHub-Projektbeschreibung)

Lokale Desktop-App für die Schweizer Pensionsplanung: Monat für Monat simuliert sie Einkommen, Pensionskasse, AHV,
Steuern, Immobilien und Hypotheken bis zum Lebensende, mit historischen Daten oder Monte Carlo, sechs Entnahmestrategien,
Szenarien und einer Optimierung, die die passende Strategie sucht. Alle Daten bleiben auf dem eigenen Rechner.

## Worum es geht

VivaPlan beantwortet die Frage, die sich vor der Pensionierung stellt: **Reicht das Geld, und wie sicher
ist das?** Die App rechnet die eigene finanzielle Zukunft Monat für Monat durch, vom heutigen Tag bis zum Ende der
Planung. Sie kennt die Schweizer Eigenheiten: Pensionskasse mit Umwandlungssatz und Kapitalbezug, AHV, Einkommens-,
Vermögens-, Kapitalbezugs- und Grundstückgewinnsteuer, SARON-Hypotheken und die Tragbarkeitsrechnung der Banken.

Weil die Börsenrenditen unsicher sind, rechnet die App nicht nur einen Verlauf, sondern je nach Einstellung tausende
(Monte Carlo oder alle historischen Startzeitpunkte) und zeigt, in wie vielen davon das Geld reicht, einschliesslich
des 1. und 99. Perzentils und des schlechtesten Laufs.

Die App läuft auf macOS, Windows und Linux. Sie greift nie aufs Netzwerk zu: Planungen, Zeitreihen und Ergebnisse
bleiben ausschliesslich auf dem eigenen Rechner. Eine Planung ist eine einzelne Datei (`.vplan`), die sich speichern,
kopieren und archivieren lässt; ein Doppelklick auf die Datei öffnet sie in VivaPlan. Beim Start öffnet die Desktop-App die zuletzt geöffnete oder gespeicherte Planung (sonst eine mitgelieferte Beispielplanung); beim Beenden mit ungespeicherten Änderungen fragt sie nach.

VivaPlan hiess früher «RetirementPlanner». Planungen von damals (`.rplan`) lassen sich weiterhin öffnen; beim Speichern
schlägt VivaPlan eine neue Datei mit der Endung `.vplan` vor, die alte Datei bleibt unverändert. Einstellungen, die
zuletzt geöffnete Planung, die Zeitreihen und die gespeicherten Ergebnisse übernimmt der erste Start einmalig.

Das Programm ist ein Rechenwerkzeug und keine Anlage-, Steuer- oder Vorsorgeberatung.

## Planung erfassen

Der Editor führt durch zwölf Abschnitte:

- **Haushalt & Projekt:** Personen mit Geburtsdatum, Währung; auf Wunsch eine **zweite Person** (Double Income) mit
  eigenem Einkommen, eigener Pensionskasse und AHV und den Ehepaar-Regeln (Plafonierung der AHV-Renten,
  eigene Steuertabellen für Verheiratete).
- **Meilensteine:** Zeitpunkte als Datum oder als Alter einer Person («mit 63»). Jede Person hat zwei feste,
  nicht löschbare Meilensteine: **Bezug Pensionskasse** (58 bis 65 Jahre) und **AHV-Bezug** (63 bis 70 Jahre), beide
  standardmässig 65. Alle anderen Angaben können sich auf Meilensteine beziehen; verschiebt man den Bezug der
  Pensionskasse, verschiebt sich alles, was daran hängt. Das Ende des Erwerbseinkommens steht im Einkommen; läuft es nach
  dem Bezug der Pensionskasse weiter, warnt die Prüfung.
- **Vorsorge:** Pensionskasse mit Guthaben, Beiträgen, Verzinsung, Umwandlungssatz und Kapitalbezug in Prozent;
  AHV-Rente mit oder ohne Teuerungsanpassung.
- **Säule 3a:** ein Konto je Person mit Guthaben, Verzinsung, Einzahlung pro Jahr (bis zum Maximalbetrag, zählt zu den
  Ausgaben, bis vor dem AHV-Bezug) und Auszahlungsplan (frühestens 5 Jahre vor dem AHV-Bezug); der Rest wird spätestens
  im AHV-Bezug ausbezahlt. Der Zeitpunkt einer Auszahlung ist ein Datum oder ein Alter der Kontoinhaberin bzw. des
  Kontoinhabers; jede Zeile zeigt das Guthaben vor und nach der Auszahlung aus der Vorschau, und eine neue Zeile erhält
  als Betrag das Guthaben zu ihrem Zeitpunkt.
- **Einnahmen & Ausgaben:** Erwerbseinkommen, Lebenshaltung und einmalige Beträge (Erbschaft, Autokauf, Reise). Jeder
  Wert kann zeitabhängig sein, zum Beispiel tiefere Ausgaben ab 80.
  Das Erwerbseinkommen jeder Person kann wahlweise mit der Teuerung steigen (Beträge in heutigen Preisen; neue
  Planungen: ja).
  Die Lebenshaltungskosten enthalten Miete bzw. Wohnkosten; die App zieht den Wohnanteil zum Simulationsstart ab und
  rechnet mit den simulierten Wohnkosten (Erneuerung, Verkauf, Miete).
- **Vermögen & Anlage:** Aktien, Obligationen und Cash sowie die Aktienquote; neben Aktien und Obligationen steht ihr
  Anteil am Portfolio. Cash zählt nicht zum Portfolio; die
  Aktienquote gilt nur für Aktien und Obligationen. Bis zur Pensionierung hält die App so viel Cash wie erfasst (mit
  der Teuerung), ein Überschuss geht ins Portfolio, ein Fehlbetrag wird daraus nachgefüllt.
  Die **Anlagestrategie** wählt je einen Index für Aktien (Welt (JST), Welt individuell, Schweiz (JST),
  S&P 500) und Obligationen (Eidgenossenschaft 10 Jahre, USA 10 Jahre, Eidgenossenschaft kurzfristig); bei «Welt individuell»
  stellen Sie die Gewichte von Schweiz, USA, EU und Asien-Pazifik ein (zusammen 100 %). Die Reihe umfasst die
  gemeinsamen Jahre beider Indizes; die mittleren Renditen gelten für diesen Zeitraum, ein gekürzter Index wird mit
  einem Hinweis angezeigt.
- **Simulation:** Methode der Renditen (siehe unten); die Ansicht zeigt nur die Eingaben, die für die gewählte
  Methode gelten. Die Zeitreihe ergibt sich aus der Anlagestrategie.
- **Entnahmestrategie:** sechs Strategien (siehe unten).
- **Optimierung:** sucht die beste Strategie für das eigene Szenario (siehe unten).
- **Steuern:** Tabellen für Einkommens-, Vermögens-, Kapital- und Grundstückgewinnsteuer mit Steuerfuss; die Sätze
  werden zwischen den Stufen linear interpoliert.
- **Immobilien:** beliebig viele Liegenschaften und Mietverhältnisse.
- **Szenarien & Simulation:** Variationen und Einstellungen für mehrere Läufe.

**Einstellungen** (macOS: Menü «VivaPlan → Einstellungen…», Windows und Linux: «Datei → Einstellungen…»): Grenzen für AHV-Bezug und Bezug Pensionskasse und der
Maximalbetrag der Säule 3a. Sie gelten für alle Planungen auf dem Gerät und werden nicht in der Planungsdatei
gespeichert. Ältere Planungsdateien werden beim Öffnen übernommen (feste Meilensteine, Säule 3a als eigenes Konto); ein
Dialog nennt jede Änderung.

Neben dem Editor zeigt eine **Live-Vorschau** den Vermögensverlauf, nach jeder Änderung neu gerechnet (bei Eingabefeldern beim Verlassen des Feldes). Eine
laufende **Prüfung** meldet Fehler und Hinweise, bevor gerechnet wird. Eine **Zeitleiste** zeigt alle Meilensteine.

## Renditen: fix, historisch oder Monte Carlo

- **Fix:** ein Lauf mit festen Jahresrenditen für Aktien, Obligationen, Inflation und SARON.
- **Historisch:** ein Lauf mit den tatsächlichen Renditen ab einem gewählten Startmonat, etwa «Was wäre, wenn ich 1973
  pensioniert worden wäre?»
- **Alle historischen Startzeitpunkte:** jeder Startmonat, der den ganzen Planungszeitraum abdeckt.
- **Bootstrap:** Monte Carlo aus historischen Daten. Zufällig lange Blöcke aufeinanderfolgender Monate werden neu
  zusammengesetzt; Aktien, Obligationen, Inflation und Zinsen stammen dabei immer aus demselben Monat, damit ihr
  Zusammenhang erhalten bleibt.
- **Parametrisch:** Monte Carlo mit einem Aktienmodell in Tagesschritten, mit breiten Rändern (Student-t) und
  Kurssprüngen. Die Parameter folgen dem gewählten Aktienindex (Vorschlagswerte, so gewählt, dass die Jahresrenditen
  dem historischen Bootstrap entsprechen) oder werden selbst gesetzt. Obligationen und Inflation sind zufällig und mit
  den Aktien korreliert, mit Mittelwert, Streuung und Korrelation der Reihe aus der Anlagestrategie.

Die Läufe werden parallel auf allen Prozessorkernen gerechnet, 50 000 Monte-Carlo-Pfade dauern wenige Sekunden. Gleiche
Einstellungen ergeben immer gleiche Ergebnisse.

## Entnahmestrategien

- **Standard:** Cash-Reserve (nur hier einstellbar, alle anderen Strategien rechnen mit fest 3 Monaten) plus fester Aktienanteil, die Ausgaben bleiben wie geplant.
- **3-Topf:** Cash, Obligationen und Aktien mit regelbasiertem Auffüllen im Januar.
  Wahlweise steuern Guardrails die Ausgaben («3-Topf + Guardrails»): Die Töpfe bestimmen die Anlage, die Topfziele
  rechnen mit der Lebenshaltung der Guardrails.
- **Fixe Entnahmerate:** jedes Jahr ein fester Anteil des Vermögens.
- **Guardrails** (Guyton-Klinger): Startrate mit Leitplanken, die die Ausgaben kürzen oder erhöhen.
- **VPW** (variable Prozententnahme): so viel, dass das Vermögen bis zu einem Planungsalter reicht.
- **Endowment** (Yale/Tobin): geglättete Entnahme.

Bei den vier letzten (und bei 3-Topf mit Guardrails) folgen die Ausgaben dem Vermögen; eine Unter- und Obergrenze der
Lebenshaltung begrenzt die Schwankung. Alle Strategien, auch «Standard», gelten ab der Pensionierung (Bezug Pensionskasse) der ersten Person; davor gilt
nur die Cash-Regel aus «Vermögen & Anlage».
Ein für einen Kauf reservierter Verkaufserlös zählt nicht zum Vermögen, aus dem die Entnahme berechnet wird.

## Optimierung

Die Optimierung sucht für jede Strategie die Parameter, die den Konsum (Median und schlechtes Jahr, real) möglichst hoch
und gleichmässig halten, solange die Ruinwahrscheinlichkeit unter einer gewählten Grenze bleibt. Das Ergebnis wird auf
unabhängigen Pfaden geprüft, damit es nicht nur auf dem Zufall der Suche beruht, und lässt sich mit einem Klick in die
Planung übernehmen. Die Einstellungen werden mit der Planung gespeichert, und eine **Schätzung der Dauer** zeigt vor dem
Start, wie lange die Rechnung mit den gewählten Pfaden dauert. Dasselbe gibt es im Terminal (`optimize`).
Zwei Ziele stehen zur Wahl: **Konsum bei begrenztem Ruinrisiko** (Standard) oder **Kein Notverkauf von Immobilien**, also der geringste
Anteil an Läufen mit ungeplantem Immobilienverkauf bei einem Mindestkonsum und begrenztem Ruin.

## Immobilien und Hypotheken

Eine Liegenschaft kann bereits im Besitz sein, zu einem Datum gekauft und verkauft werden oder erst nach dem Verkauf
einer anderen gekauft werden, etwa der Wechsel vom Haus in eine kleinere Wohnung. Beim Verkauf fällt die
Grundstückgewinnsteuer an. Hypotheken bestehen aus Tranchen mit aufeinanderfolgenden Perioden, fest oder SARON plus
Marge, mit Verlängerung oder Rückzahlung am Ende. Die Amortisation erfolgt direkt oder indirekt über die Säule 3a;
indirekt nur bis zum AHV-Bezug, danach direkt.

Beim Kauf prüft die App die Tragbarkeit wie eine Bank und kürzt die Hypothek nötigenfalls; auf Wunsch wird mit der
höchsten Belehnung von 80 % finanziert. Wird das liquide Vermögen knapp, verkauft die App eine Liegenschaft als Notverkauf.
Ist keine Liegenschaft mehr im Besitz, beginnen die erfassten Mieten.

## Datenbibliothek

Die Renditen stammen aus den Indizes der Anlagestrategie. Dahinter stehen diese Daten:

- **Welt (JST):** Aktien von 11 Ländern aus der Jordà-Schularick-Taylor Macrohistory Database (Release 6), jedes Jahr
  nach realem BIP gewichtet, in CHF, jährlich 1900–2020; dazu 10-jährige Obligationen der Eidgenossenschaft (JST bis
  1988, danach SNB). Daraus auch «Schweiz (JST)» und «Welt individuell». Lizenz CC BY-NC-SA 4.0 (nicht kommerziell).
  Der Index liegt rund 1,3 Prozentpunkte pro Jahr über dem Weltindex von Dimson, Marsh und Staunton (Begründung in
  `docs/FEA-51 FEA-52 Weltindex JST.md`).
- **Shiller USD und CHF:** Robert Shillers US-Daten ab 1872 (Aktien inklusive Dividenden, Staatsanleihen, Inflation),
  dieselben Daten in Franken umgerechnet mit dem Schweizer Landesindex der Konsumentenpreise und dem SARON, dazu eine
  Variante mit 10-jährigen Obligationen der Eidgenossenschaft.

Neuere Shiller- und SNB-Dateien lassen sich selbst herunterladen und importieren. Eigene Zeitreihen im CSV-Format
verwendet die App nicht mehr. Die Reihe «Pictet CH» wird aus Lizenzgründen nicht mehr mitgeliefert (FEA-72); ältere Planungen erhalten beim Öffnen die passende Anlagestrategie. Alle Indizes sind in
CHF; die App weist darauf hin, wenn die Planung in einer anderen Währung ist.

Die Zeitreihen fallen nicht unter die Lizenz des Programms (GPL v3). Die Rechte an den Grunddaten liegen bei den
Quellen; die App liefert nur daraus berechnete Renditen mit Quellenangabe aus (`crates/engine/data/QUELLEN.md`).

## Szenarien

Ein oder zwei Parameter lassen sich in Schritten variieren: Aktienanteil, Kapitalbezug der Pensionskasse,
Datum des Bezugs der Pensionskasse, historischer Startmonat, Entnahmerate oder ein Faktor auf die Lebenshaltungskosten ohne Wohnanteil. Jede Kombination wird mit
denselben Renditepfaden gerechnet, damit sich die Varianten fair vergleichen lassen. Eine Schätzung zeigt die erwartete
Rechendauer.

## Ergebnisse und Grafiken

Die Ergebnisansicht hat die Registerkarten Übersicht, Grafiken, Einzelne Läufe, Szenarien und Jahreswerte. Alle Beträge
lassen sich nominal oder in heutiger Kaufkraft anzeigen.

- **Ein Lauf:** Vermögensverlauf; liquides Vermögen nach Aktien, Obligationen und Cash; Einnahmen und Ausgaben pro
  Jahr; Entnahmerate; Immobilien; Vorsorgeguthaben; der Verlauf von Aktien und Obligationen als Index; Ereignisliste.
- **Mehrere Läufe:** Bandbreite des Vermögens (5. bis 95. Perzentil mit 1. und 99. Perzentil und dem schlechtesten
  Wert), Wahrscheinlichkeit, dass das Geld reicht, Verteilungen von Endvermögen und Vermögen bei Pensionierung, Alter bei
  Erschöpfung. Die Läufe am 1., 5., 50., 95. und 99. Perzentil und der schlechteste Lauf lassen sich im Detail ansehen.
- **Szenarien:** Kurven je Variante, bei zwei Parametern Heatmaps.

**Liste der Simulationen:** Jede Simulation wird mit der Planung von damals gespeichert. Der automatische Name nennt
Planung, Renditequelle und Strategie in Kurzform (z. B. «MC hist. · 3-Topf + Guard»). Die Liste lässt sich umbenennen,
mit Notizen versehen, aufräumen und zum **Vergleichen** von bis zu zehn Ergebnissen nutzen. Der Vergleich bietet dieselben
Möglichkeiten wie ein einzelnes Ergebnis: Kennzahlen (nach jeder Spalte sortierbar), Bandbreite, Perzentile,
Wahrscheinlichkeit und Erschöpfungsalter, die Verläufe des Einzel- bzw. Median-Laufs (Aktien, Obligationen und Inflation
auch als Änderung in % oder logarithmisch), Jahreswerte mit einer Spalte je Ergebnis und die Eingaben, die sich
unterscheiden, mit den Bezeichnungen des Editors; nominal oder in heutiger Kaufkraft.

## Hilfe und Bedienung

Eine **kontextabhängige Hilfe** öffnet auf **F1** rechts neben dem Editor und zeigt das Thema des Felds, des Abschnitts
oder der Grafik, an der man gerade arbeitet. Das Handbuch mit Suche erklärt jeden Abschnitt, vor allem die Renditen und die
Optimierung ausführlich. Menüs und die üblichen Tastenkürzel (neu, öffnen, speichern, Simulation starten, Rückgängig,
Wiederholen, Wechsel der Abschnitte) sind vorhanden, ebenso ein «Über»-Fenster mit Version und Buildzeit.

**Einstieg für neue Benutzer:** Beim allerersten Start fragt ein Willkommensdialog, ob man das Beispiel erkunden, eine
eigene Planung beginnen oder leer starten möchte (wieder über *Hilfe → Willkommen …*). Die Checkliste «Erste Schritte»
neben der Prüfung führt durch die acht Schritte des Schnellstarts und hakt erledigte Abschnitte selbst ab. Die
Ergebnisübersicht beginnt mit dem Ergebnis in Worten: Erfolgsquote mit Einordnung (hoch, mittel, tief), ungünstigster
Lauf und ein Vorschlag für den nächsten Schritt.

Für Fortgeschrittene gibt es eine Kommandozeile: rechnen, prüfen, optimieren, Monatswerte exportieren, Daten
importieren und die Rechenleistung messen. Das Dateiformat ist als JSON-Schema dokumentiert. Konfigurationen eines
früheren Programms lassen sich importieren.

## Installation

Pakete für **Windows** (.msi, .exe), **Linux** (.deb, .rpm, .AppImage) und **macOS** (.dmg) baut ein GitHub-Workflow beim
Setzen eines Versions-Tags und legt sie als Release-Entwurf ab. Die Pakete sind nicht signiert, Windows und macOS zeigen
deshalb beim ersten Start eine Warnung.

## Verlässlichkeit

Die Rechenregeln sind in `docs/ENTWICKLUNG.md` Schritt für Schritt beschrieben. Jede Regel ist mit einem von Hand nachgerechneten
Testfall belegt. Dazu kommen:

- Prüfungen an Hunderten zufälliger Planungen (zum Beispiel muss das Gesamtvermögen immer der Summe seiner Teile entsprechen),
- statistische Tests der Monte-Carlo-Pfade,
- Robustheitstests mit absichtlich fehlerhaften Eingaben,
- zehn eingecheckte Referenzplanungen (Golden Masters), die jede unbeabsichtigte Änderung der Ergebnisse melden,
- ein Vergleich mit einer Referenztabelle, die auf den Rappen übereinstimmen muss,
- automatische Tests aller Ansichten und der Bedienung der Oberfläche.

**Haftungsausschluss:** VivaPlan ist kostenlos und liefert Modellrechnungen ohne Gewähr, keine Finanz- oder
Steuerberatung. Beim ersten Start bestätigt man eine Kurzfassung (bei einem neuen Text erneut), die Ergebnisseiten und
das Ergebnis der Optimierung tragen eine Fusszeile, der vollständige Text steht in der Hilfe und ist über
«Über VivaPlan» erreichbar. Die Kommandozeile nennt die Kurzfassung in ihrer Hilfe.

## Technik

Rechenkern und Kommandozeile in Rust, Desktop-App mit Tauri 2, Oberfläche in HTML und JavaScript ohne externe
Bibliotheken. Lizenz: GPL v3 für den Programmcode; die mitgelieferten Zeitreihen fallen nicht darunter.
