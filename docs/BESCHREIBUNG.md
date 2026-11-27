# VivaPlan – Beschreibung

## Kurzfassung (GitHub-Projektbeschreibung)

Lokale Desktop-App für die Schweizer Pensionsplanung: Monat für Monat simuliert sie Einkommen, Pensionskasse, AHV,
Steuern, Immobilien und Hypotheken bis zum Lebensende, mit historischen Daten oder Monte Carlo, sechs Entnahmestrategien,
Szenarien und einer Optimierung, die die passende Strategie sucht. Gerechnet und gespeichert wird auf dem eigenen Rechner.

## Worum es geht

VivaPlan beantwortet die Frage, die sich vor der Pensionierung stellt: **Reicht das Geld, und wie sicher
ist das?** Die App rechnet die eigene finanzielle Zukunft Monat für Monat durch, vom heutigen Tag bis zum Ende der
Planung. Sie kennt die Schweizer Eigenheiten: Pensionskasse mit Umwandlungssatz und Kapitalbezug, AHV, Einkommens-,
Vermögens-, Kapitalbezugs- und Grundstückgewinnsteuer, SARON-Hypotheken und die Tragbarkeitsrechnung der Banken.

Weil die Börsenrenditen unsicher sind, rechnet die App nicht nur einen Verlauf, sondern je nach Einstellung tausende
(Monte Carlo oder alle historischen Startzeitpunkte) und zeigt, in wie vielen davon das Geld reicht, einschliesslich
des 1. und 99. Perzentils und des schlechtesten Laufs.

Die App läuft auf macOS, Windows und Linux. Berechnen und Speichern laufen ohne Internet: Die App, der Rechenkern und
der MCP-Server `vivaplan mcp` laden nichts und senden nichts; Planungen, Zeitreihen und Ergebnisse liegen auf dem
eigenen Rechner (#238). Einzige Ausnahme ist die Suche nach Updates in der App, und nur mit
Zustimmung (#309): Beim ersten Start (nach der Beta-Aktivierung; bestehende Installationen einmal nach dem Update) fragt
die App, ob sie beim Start nach neuen Versionen suchen darf. Mit «Ja» fragt sie bei jedem Start im Hintergrund das
Manifest der Releases von VivaPlan auf GitHub ab (übertragen werden nur Version und Betriebssystem, keine
Planungsdaten) und zeigt eine neue Version mit ihren Änderungen, «Jetzt installieren» und «Später»; installiert wird nur
nach dem Klick, danach startet die App neu. Updates sind signiert; eine Version mit ungültiger Signatur wird nicht
installiert, ein Hinweis nennt den Grund. Ohne Internet oder bei einem Fehler des Servers startet die App normal, der
Fehler steht nur im Protokoll. Mit «Nein» oder ohne Antwort baut die App nie eine Verbindung auf. Die Wahl steht in den
Einstellungen (Reiter «Allgemein»). Daten verlassen den Rechner nur, wenn man VivaPlan selbst mit einer KI
verbindet (`vivaplan mcp`, Kommandozeile), und dann nur das, was die KI abfragt. Links (Einführungsvideos, Spendenlink) öffnen den Standardbrowser mit einer festen
Adresse; die App selbst lädt dabei nichts. Eine Planung ist eine einzelne Datei (`.vplan`), die sich speichern,
kopieren und archivieren lässt; ein Doppelklick auf die Datei öffnet sie in VivaPlan. Beim Start öffnet die Desktop-App die zuletzt geöffnete oder gespeicherte Planung (sonst eine mitgelieferte Beispielplanung); beim Beenden mit ungespeicherten Änderungen fragt sie nach. Das Menü «Datei → Zuletzt geöffnet» zeigt die letzten 10 geöffneten oder gespeicherten Planungen zum schnellen Wechseln; eine nicht mehr vorhandene Datei meldet die App und entfernt sie aus der Liste, «Liste löschen» leert sie. Die Liste enthält nur die Dateipfade und liegt in den lokalen Einstellungen der App.

VivaPlan hiess früher «RetirementPlanner». Planungen von damals (`.rplan`) lassen sich weiterhin öffnen; beim Speichern
schlägt VivaPlan eine neue Datei mit der Endung `.vplan` vor, die alte Datei bleibt unverändert. Einstellungen, die
zuletzt geöffnete Planung, die Zeitreihen und die gespeicherten Ergebnisse übernimmt der erste Start einmalig.

Das Programm ist ein Rechenwerkzeug und keine Anlage-, Steuer- oder Vorsorgeberatung.

## Planung erfassen

Der Editor führt durch zwölf Abschnitte:

- **Haushalt & Projekt:** Personen mit Geburtsdatum, Währung; auf Wunsch eine **zweite Person** (Double Income) mit
  eigenem Einkommen, eigener Pensionskasse und AHV und den Ehepaar-Regeln (Plafonierung der AHV-Renten,
  eigene Steuertabellen für Verheiratete, Kapitalbezüge eines Jahres zusammen besteuert); bei unverheirateten Paaren
  zahlt jede Person die Kapitalbezugssteuer auf ihre eigenen Bezüge. «Verheiratet» gibt es auch für eine Person
  allein (Ehepartner ohne Einkommen und Rente): Dann gelten die Steuertarife und Abzüge für Verheiratete (#250).
- **Meilensteine:** Zeitpunkte als Datum oder als Alter einer Person («mit 63»). Jede Person hat zwei feste,
  nicht löschbare Meilensteine: **Bezug Pensionskasse** (58 bis 65 Jahre) und **AHV-Bezug** (63 bis 70 Jahre), beide
  standardmässig 65. Alle anderen Angaben können sich auf Meilensteine beziehen; verschiebt man den Bezug der
  Pensionskasse, verschiebt sich alles, was daran hängt. Das Erwerbseinkommen einer Person endet mit ihrem Bezug der
  Pensionskasse, solange der Schalter dazu im Einkommen eingeschaltet ist (#236); läuft es bei ausgeschaltetem Schalter
  nach dem Bezug der Pensionskasse weiter, warnt die Prüfung.
- **Vorsorge:** Pensionskasse mit Guthaben, Beiträgen, Verzinsung, Umwandlungssatz und Kapitalbezug in Prozent;
  AHV-Rente nach dem Mischindex wie bei der AHV (#284: Mittel aus Preis- und Lohnindex, Lohnindex aus der Teuerung
  und der Annahme «Reallohnsteigerung», Vorgabe 0,5 % pro Jahr, unter «Simulation» › «Experte»; Anpassung im Januar
  ungerader Jahre, bei mehr als 4 % Teuerung seit dem letzten Januar auch in geraden Jahren; nominal nie tiefer als
  zuvor: nach einer Deflation steigt sie erst, wenn der Index den bisher höchsten Stand übersteigt, #280), eingegeben
  als erwartete Rente mit 65 laut Rentenvorausberechnung in heutigen Werten (bis zum Bezug nach dem Mischindex
  angepasst, #282, #284); bei Bezug vor oder nach 65 kürzt bzw.
  erhöht die App sie selbst nach den Sätzen der AHV (Vorbezug 6,8 % je Jahr, höchstens 13,6 %; Aufschub 5,2 % bis
  31,5 %; feste Beträge in Franken, bei Ehepaaren nach dem Plafond auf die ungekürzten Renten), neben dem Feld steht
  die effektive Rente; ab Dezember 2026 jedes Jahr im Dezember die 13. AHV-Rente (1/12 der im
  Jahr ausbezahlten AHV-Renten, anteilig bei Beginn im Laufe des Jahres). Zuoberst in der Karte «Pensionskasse» jeder
  Person steht der Schalter «Ohne Pensionskasse (grosse Säule 3a)» (#152, seit #253 hier statt unter «Haushalt &
  Projekt»): Ist er gesetzt, sind die Felder der Pensionskasse grau und schreibgeschützt, und die Berechnung rechnet
  für diese Person ohne Pensionskasse (kein Guthaben, keine Beiträge, keine Rente, kein Kapitalbezug und keine
  Kapitalbezugssteuer); die erfassten Werte bleiben gespeichert und gelten wieder, sobald der Schalter aus ist.
  Je Person eine Karte **«Pflegeheim»** (#271): Eintritt als Alter, Meilenstein oder Datum, Heimkosten pro Monat in
  heutigen Preisen (mit der Teuerung) und «Wohneigentum beim Eintritt verkaufen». Ab dem Eintritt zählen die
  Heimkosten zu den Ausgaben (Spalte «Heimkosten» in den Jahreswerten). Die Wohnkosten des Haushalts (Miete,
  Immobilie) enden erst, wenn alle Personen im Heim sind; dann verkauft die App mit «verkaufen» die Liegenschaften im
  Eintrittsmonat (mit Grundstückgewinnsteuer, nicht mit «Immobilien nie verkaufen»), und Mieten oder Käufe «nach
  Verkauf» beginnen nicht, auch nicht nach einem Notverkauf. Ein Todesfall wird nicht abgebildet. Bei den Steuern
  gelten die Heimkosten vereinfacht als Krankheitskosten: abziehbar bei Bund und Kanton, soweit sie 5 % des
  Nettoeinkommens übersteigen. Folgt nach dem Verkauf der letzten Liegenschaft weder eine Miete noch ein Pflegeheim,
  weist die Prüfung darauf hin.
- **Säule 3a:** ein Konto je Person mit Guthaben, Verzinsung, Einzahlung pro Jahr (bis zum Maximalbetrag; für eine
  Person «Ohne Pensionskasse (grosse Säule 3a)» bis 20 % des Erwerbseinkommens, höchstens der Maximalbetrag der grossen
  Säule 3a, #152; zählt zu den Ausgaben, bis vor dem AHV-Bezug) und Auszahlungsplan (frühestens 5 Jahre vor dem AHV-Bezug, ausser Vorbezug für Wohneigentum); der Rest wird spätestens
  im AHV-Bezug ausbezahlt. Der Zeitpunkt einer Auszahlung ist ein Datum oder ein Alter der Kontoinhaberin bzw. des
  Kontoinhabers; jede Zeile zeigt das Guthaben vor und nach der Auszahlung aus der Vorschau, und eine neue Zeile erhält
  als Betrag das Guthaben zu ihrem Zeitpunkt.
  Mit dem Schalter **«In Tranchen beziehen»** (ab Alter 60 bis 65, 1 bis 6 Tranchen, #300) bezieht die App statt des
  Auszahlungsplans jedes Jahr im Geburtsmonat eine Tranche: Guthaben ÷ Zahl der verbleibenden Tranchen, die letzte ist
  der Rest, spätestens im AHV-Bezug. Das senkt die Kapitalbezugssteuer. Vereinfachung: ein Konto mit Teilbezügen statt
  n Konten, die gesetzlich je ganz bezogen werden; steuerlich gleich.
  **Vorbezug für Wohneigentum (#301):** Eine Auszahlung mit der Verwendung «Wohneigentum (Vorbezug)» oder «Hypothek
  zurückzahlen» darf in jedem Alter liegen (BVV 3 Art. 3), als Teilbetrag oder ganzes Guthaben. «Wohneigentum
  (Vorbezug)» geht ins freie Vermögen; Kauf oder Renovation werden separat erfasst (geplante Immobilie, einmalige
  Ausgabe). Die Prüfung verlangt im Monat der Auszahlung eine selbst bewohnte Immobilie (im Besitz oder im selben Monat
  gekauft) und zwischen zwei Vorbezügen für Wohneigentum vor der Grenze von 5 Jahren vor dem AHV-Bezug mindestens 5
  Jahre. Besteuert wird der Vorbezug wie jede Auszahlung als Kapitalbezug; Einzahlungen gehen weiter. Mit Tranchen
  gelten Vorbezüge für Wohneigentum vor dieser Grenze weiter, die Tranchen rechnen mit dem kleineren Guthaben.
  Zustimmung des Ehepartners und Nachweis bei der Stiftung nennt die Hilfe; die App prüft sie nicht.
- **Einnahmen & Ausgaben:** Erwerbseinkommen, Lebenshaltung und einmalige Beträge (Erbschaft, Autokauf, Reise). Jeder
  Wert kann zeitabhängig sein, zum Beispiel tiefere Ausgaben ab 80.
  Zeitabhängige Werte (Verläufe) gibt es als Konstante, Treppe («ab Zeitpunkt gilt Wert»), Gerade (von Wert a zu
  Zeitpunkt t1 auf Wert b zu Zeitpunkt t2, davor a, danach b), Tabelle (Stützpunkte nach Alter, Datum oder Meilenstein,
  dazwischen gleitend oder als Treppe) und Faktor (Grundwert mal Verlauf, zum Beispiel Lebenshaltung 100 % bis 80, danach
  80 %), #217. Gerade, Tabelle und Faktor gelten für jedes zeitabhängige Feld. Im Editor wählt eine Leiste beim Feld
  die Form (#243); Stützpunkte von Gerade und Tabelle liegen bei einem Alter einer Person, einem Meilenstein oder einem
  festen Datum, Tabellenpunkte lassen sich hinzufügen und löschen, der Faktor ist selbst ein Verlauf in Prozent. Beim
  Wechsel der Form bleiben Zeitpunkte und Werte erhalten, soweit die neue Form sie fassen kann; die Minigrafik zeigt den
  Verlauf.
  Der Aktienanteil kann statt eines Verlaufs eine **Kennlinie** über eine Zustandsgrösse sein (#218): Stützpunkte
  «bei Zustandswert gilt Aktienanteil», dazwischen linear. Zustandsgrössen sind Alter, liquides Vermögen in Preisen des
  Starts, Entnahmerate (Entnahme der letzten 12 Monate / liquides Vermögen), Deckungsgrad (liquides Vermögen / restliche
  Lücke aus Lebenshaltung minus Einkommen und Renten), Aktienrendite der letzten 12 Monate und Abstand vom Höchststand.
  Ausgewertet wird beim Umschichten, einmal pro Jahr oder jeden Monat. Beispiel: 2 % → 80 %, 6 % → 40 % über der
  Entnahmerate ergibt bei 4 % einen Aktienanteil von 60 %. Der Editor zeigt die Kennlinie nur an; erfasst wird sie in
  der Planungsdatei oder über die KI-Schnittstelle. Die Hilfe nennt die Grenze: Im parametrischen Modell sind die Jahre
  unabhängig, Regeln, die auf Erholung nach Einbrüchen setzen, lassen sich dort nicht sinnvoll prüfen.
  Das Erwerbseinkommen wird pro Jahr erfasst wie in der Steuererklärung («Einkünfte aus Erwerb netto / Jahr», Nettolohn
  im Lohnausweis Ziffer 11); daneben steht der Betrag pro Monat. Gespeichert und gerechnet wird pro Monat (ein Zwölftel).
  Das Erwerbseinkommen jeder Person steigt mit der Teuerung (Beträge in heutigen Preisen; Ausnahme «nominal» unter
  «Experte», siehe [Teuerung](#teuerung)).
  Je Person gibt es den Schalter «Erwerbseinkommen endet mit dem Bezug der Pensionskasse» (#236; neue Planungen und
  eine neu hinzugefügte zweite Person: ein, ältere Planungen ohne Angabe: aus). Eingeschaltet ist das
  Erwerbseinkommen ab dem Monat ihres Meilensteins «Bezug Pensionskasse» 0, auch wenn die Zeitreihe danach einen Wert
  hat; das gilt auch für grosse Säule 3a, Berufskosten, Abzüge für Erwerbstätige und Zweitverdienerabzug.
  Ausgeschaltet (Teilpensionierung, Arbeit nach der Pensionierung) gilt die Zeitreihe wie eingegeben.
  Die Lebenshaltungskosten enthalten Miete bzw. Wohnkosten; die App zieht den Wohnanteil zum Simulationsstart ab und
  rechnet mit den simulierten Wohnkosten (Erneuerung, Verkauf, Miete). Der Wohnanteil umfasst Miete, Hypothekarzins
  und Nebenkosten, aber keine Amortisation (#264): Sie bildet Vermögen und fliesst in der Simulation als Schuldabbau
  bzw. Einzahlung in die Säule 3a weiter.
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
- **Steuern:** Modus «Wohnort» (#63): Einkommens- und Vermögenssteuer aus Kanton, Gemeinde (alle Gemeinden des
  Steuerjahres, Auswahl mit Suche) und Kirchensteuer (Konfession) nach den Steuergesetzen von Bund und Kantonen (#286, Wallis mit indexierten Tarifen von Kanton und Gemeinden #296 und der Ermässigung für Verheiratete #297; Steuerfüsse noch aus
  den ESTV-Grunddaten, #285), mit den
  Standardabzügen von Kanton und Bund (Sozialabzüge, Versicherungsprämien und Sparzinsen, mit Erwerb in der Variante
  «mit 2. und 3. Säule», Berufskostenpauschale, bei Ehepaaren mit zwei Erwerbseinkommen der Zweitverdienerabzug;
  statt dessen je Zeitraum ein eigener Wert, dazu die Säule 3a, #131) und
  der direkten Bundessteuer; die Steuerdatei ist eingebettet, ohne
  Netzwerkzugriff. Im Modus «Wohnort» auch die Kapitalbezugssteuer (#64) nach der Regel des Kantons mit den
  Steuerfüssen der Einkommenssteuer und einem Fünftel des Bundestarifs, ohne eigene Tabelle; die Regel steht unter der
  Gemeinde, angenäherte Regeln (Wallis) mit ⚠. Wohnort als Standard (#272): Neue Planungen beginnen im Modus
  «Wohnort» ohne Gemeinde; nötig sind nur die Gemeinde (Suche über alle Gemeinden, der Kanton ergibt sich daraus) und
  die Konfession. Die übrigen Einstellungen gelten mit Vorgaben und stehen als Zeile «Annahmen» in Worten da
  (Standardabzüge des Kantons bis und ab Pensionierung, Tarife mit oder ohne Teuerung, Abzug der Säule 3a ja, Steuerwert der
  Immobilien 50 % des Marktwerts, steuerbarer Ertrag); ein Klick öffnet die Einstellungen. Oben auf der Karte stehen
  zwei Kennzahlen aus der Vorschau des Rechenkerns: Einkommens- und Vermögenssteuer im ersten vollen Kalenderjahr und
  im ersten vollen Kalenderjahr nach der Pensionierung, in heutigen Preisen auf 100 Franken gerundet (eine Zahl, wenn
  die Pensionierung vor dem Start liegt). Modus «Eigene Tabellen»: Tabellen für Einkommens-, Vermögens- und
  Kapitalbezugssteuer mit Steuerfuss, nur noch für bestehende Planungen, die ihn benutzen (unter «Experte»); sie
  öffnen unverändert, und der Knopf «Auf Wohnort umstellen» zeigt nach der Wahl der Gemeinde die geschätzte Steuer
  des ersten Jahres vorher und nachher. Grundstückgewinnsteuer: im Modus
  «Wohnort» (#65) der Regelsatz des Kantons, in dem die Immobilie liegt (Feld Kanton je Immobilie, ohne Angabe der
  Wohnkanton; alle 26 Kantone nach dem kantonalen Gesetz mit Artikel (#287–#291); Besitzdauer in
  Monaten; Steuerfuss der Wohngemeinde oder Median des Kantons; Regeltext bei der Immobilie, angenäherte Regeln von
  Basel-Stadt, Basel-Landschaft, St. Gallen, Waadt und Wallis mit ⚠); mit dem Häkchen «Eigene Regeln statt der Regeln
  des Kantons» und im Modus «Eigene Tabellen» die eigenen Regeln. Wohnort zeitabhängig (#151): weitere Wohnorte ab
  Jahr oder Ereignis; Einkommens- und Vermögenssteuer eines Jahres nach dem Wohnort am 31. Dezember für das ganze
  Jahr, Kapitalbezugssteuer nach dem Wohnort im Monat des Bezugs, Grundstückgewinnsteuer einer Immobilie ohne eigenen
  Kanton nach dem Wohnkanton im Monat des Verkaufs (StHG Art. 4b); Parameter und Jahreswerte nennen den Wohnort je
  Zeitraum bzw. Jahr. Die Sätze der Tabellen werden zwischen den Stufen
  linear interpoliert. Die Beträge der Tabellen für Einkommen, Vermögen und Kapitalbezug und die laufenden Abzüge
  wachsen mit der simulierten Teuerung (Ausgleich der kalten Progression); die Ausnahme «nominal» steht unter
  «Simulation» › «Experte» (#306), ältere Planungen und importierte Konfigurationen behalten so ihre Rechnung. Steuerbarer Ertrag des
  Vermögens (#149): je Satz pro Jahr für Aktien (Ausschüttungsrendite) und Obligationen (Coupon); der durchschnittliche
  Bestand eines Jahres × Satz zählt zum steuerbaren Einkommen, in beiden Modi vor den Abzügen. Der Ertrag ist Teil der
  Rendite, kein zusätzlicher Zufluss, Kapitalgewinne bleiben steuerfrei; Cash hat keinen Ertrag. Neue Planungen
  beginnen mit 2 % und 1 % (Richtwerte: Dividendenrendite eines globalen Aktienindex, Rendite des Swiss Bond Index),
  bestehende mit 0 %. Die 3-Topf-Strategie rechnet die Steuer darauf in den Netto-Bedarf ein; die Jahreswerte zeigen
  die Spalte «Steuerbarer Ertrag».
- **Immobilien:** beliebig viele Liegenschaften und Mietverhältnisse.
- **Szenarien & Simulation:** Variationen und Einstellungen für mehrere Läufe.

<a id="teuerung"></a>**Teuerung wie in der Realität (#306):** Alle Beträge, die in der Realität mit der Teuerung wachsen, sind
in Preisen beim Simulationsstart erfasst und wachsen mit der Teuerung des Pfades: Lebenshaltung, Pflegeheim,
Erwerbseinkommen, Beiträge an die Pensionskasse, Einzahlung und Höchstbetrag der Säule 3a (die Einzahlung höchstens bis
zum Höchstbetrag des Jahres; die Prüfung vergleicht in Preisen des Starts), AHV-Rente (ab dem Anspruch), Steuertarife
und laufende Abzüge, einmalige Beträge, einmalige Abzüge vom Einkommen und vom Vermögen, Wert und Nebenkosten der
Liegenschaften. Immer nominal bleiben die Rente der Pensionskasse, die Hypothek (Schuld, Zins, Amortisation) und die
Guthaben von Pensionskasse und Säule 3a samt Verzinsung. Eine Miete wächst bis zum Monat des Einzugs mit der Teuerung,
danach bleibt sie nominal; eine beim Start laufende Miete ist von Anfang an nominal (2 500 heute, Einzug in 5 Jahren,
2 % Teuerung: rund 2 760, danach gleichbleibend). Unter «Simulation» › «Experte» lässt sich je Betrag die Ausnahme
«nominal, ohne Teuerung» setzen (je Miete: bis zum Einzug, immer oder nominal); die Zeile «Annahmen» darüber und die
Parameter eines Laufs nennen die gesetzten Ausnahmen. Ohne Teuerung rechnen heisst Teuerung 0 %. Neue Planungen haben
keine Ausnahme (Markierung `indexedByDefault`); ältere Planungen übernehmen beim Öffnen ihre bisherigen Regeln
(Schalter für Erwerbseinkommen, AHV, Tarife und Liegenschaften, bisher nominale einmalige Beträge und Abzüge, PK- und
3a-Beiträge) als Ausnahmen mit einem Hinweis und rechnen unverändert.

**Einstellungen** (macOS: Menü «VivaPlan → Einstellungen…», Windows und Linux: «Datei → Einstellungen…»), in Reitern gegliedert (Allgemein, Vorsorge, Simulation, Immobilien, #308): Sprache, Darstellung, Grenzen für AHV-Bezug und Bezug Pensionskasse, die
Maximalbeträge der kleinen und der grossen Säule 3a, das Startjahr der Teuerung im parametrischen Modell (Standard 1950) und die Tragbarkeit der Banken (höchstens vom Einkommen, kalkulatorischer
Zins, Nebenkosten, höchste Belehnung; Standard 33 %, 5 %, 1 %, 80 %; dazu Vermögensverzehr in der Tragbarkeit, Standard aus
mit 3 % pro Jahr, und höchste Belehnung ab der Pensionierung, Standard 65 %, #266). Sie gelten für alle Planungen auf dem Gerät und werden
nicht in der Planungsdatei gespeichert; der Abschnitt «Immobilien» nennt die geltende Tragbarkeit in einem Hinweis. Ältere
Planungsdateien werden beim Öffnen übernommen (feste Meilensteine, Säule 3a als eigenes Konto, eigene Werte der Tragbarkeit
gehen in die Einstellungen über); ein Dialog nennt jede Änderung. Gespeicherte Ergebnisse behalten die Tragbarkeit, mit der
gerechnet wurde.

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
  den Aktien korreliert, mit Mittelwert, Streuung und Korrelation der Reihe aus der Anlagestrategie. Die Teuerung
  schätzt das Modell nur aus den Jahren ab einem Startjahr (Einstellung, Standard 1950), weil die Kriegsjahre ihre
  Streuung von rund 2.3 % auf 5 % pro Jahr treiben; der Tab «Simulation» nennt den Zeitraum (z. B. «Teuerung 1950–2020»).
  Der SARON folgt der gezogenen Teuerung (Teuerung plus realer Kurzfristzins, Vorgabe aus der Reihe).

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
Der Knopf **«Strategien vergleichen (ohne Optimierung)»** rechnet die gewählten Strategien ohne Suche mit den Werten der
Planung (Standardwerte für Strategien, die die Planung nie benutzt hat), auf denselben Pfaden wie die Simulation. Die
Ergebnistabelle zeigt je Strategie zusätzlich das nominale **Endvermögen** bei 1, 5, 50, 95 und 99 %. Mit Immobilien legt
eine Auswahl fest, ob «wie geplant», mit «Immobilien nie verkaufen» oder mit beidem gerechnet wird (jede Strategie zweimal).
Die gefundenen Parameter und der Knopf zum Übernehmen stehen je Strategie in einer eigenen Zeile unter den Kennzahlen; im
Abschnitt «Optimierung» ist die Vorschau ausgeblendet, damit die Tabelle ohne waagrechtes Scrollen Platz hat.
Die Spalte **«Auf Untergrenze»** zeigt je Strategie mit Untergrenze, in wie vielen Monaten ab Pensionierung die
Untergrenze die Lebenshaltung festlegt (gemittelt über die Prüfpfade; «–» ohne Untergrenze). Ab 50 % steht der Hinweis,
dass die Untergrenze meist die Ausgaben bestimmt und die Strategie kaum wirkt; dann rechnen mehrere Strategien praktisch
gleich. Die Kommandozeile (`optimize`, auch `--compare`) zeigt dieselbe Kennzahl (`floorShare` im JSON).
Jedes abgeschlossene Ergebnis einer Optimierung oder eines Strategievergleichs legt die App wie die Simulationen in
«Gespeicherte Ergebnisse» ab, mit Bericht, Einstellungen und Planungsstand (lokal, nicht in der Planungsdatei). Der
Eintrag nennt die beste Strategie, ihre Ruinwahrscheinlichkeit und das Ziel, öffnet sich in der breiten Ergebnisansicht
mit derselben Tabelle und erlaubt «In Planung übernehmen» (mit Rückfrage, wenn er zu einem anderen Planungsstand gehört).
Beim Öffnen einer Planung zeigt der Abschnitt «Optimierung» das neueste dazu passende gespeicherte Ergebnis.

## Immobilien und Hypotheken

Eine Liegenschaft kann bereits im Besitz sein, zu einem Datum gekauft und verkauft werden oder erst nach dem Verkauf
einer anderen gekauft werden, etwa der Wechsel vom Haus in eine kleinere Wohnung. Beim Verkauf fällt die
Grundstückgewinnsteuer an. Hypotheken bestehen aus Tranchen mit aufeinanderfolgenden Perioden, fest, «Fest (Markt)»
oder SARON plus Marge, mit Verlängerung oder Rückzahlung am Ende. Die Zinsen folgen dem Zins- und Teuerungsumfeld des
Pfades: Der SARON stammt aus den historischen Daten (vor seiner Einführung 1999 der kurzfristige Zins der
Eidgenossenschaft) oder folgt im parametrischen Modell der gezogenen Teuerung; eine Periode «Fest (Markt)» erhält bei der
Erneuerung die Rendite der Bundesobligationen 10 Jahre des Pfades plus eine Marge aus den Einstellungen. Die
Ereignisse nennen bei jeder Erneuerung den neuen Satz und seine Herkunft. Die Amortisation erfolgt direkt oder indirekt
über die Säule 3a; indirekt nur bis zum AHV-Bezug, danach direkt.

Beim Kauf prüft die App die Tragbarkeit wie eine Bank (Werte in den Einstellungen) und kürzt die Hypothek nötigenfalls;
auf Wunsch wird mit der höchsten Belehnung (Standard 80 %) finanziert. Eingeschaltet zählt ein Anteil des freien Vermögens
(Bargeld, Aktien, Obligationen; beim Kauf nach Abzug des Eigenkapitals) als Einkommen; sobald alle Personen pensioniert sind,
gilt beim Kauf und bei jeder Erneuerung höchstens die Belehnung ab der Pensionierung (#266). Die Praxis der Banken ist verschieden. Wird das liquide Vermögen knapp, verkauft die App eine Liegenschaft als Notverkauf.
Ist keine Liegenschaft mehr im Besitz, beginnen die erfassten Mieten, ausser alle Personen leben im
[Pflegeheim](#planung-erfassen) (#271).

Je Liegenschaft im Besitz oder mit geplantem Kauf lässt sich ein **Steuerwert** erfassen, als Betrag oder in % des
Marktwerts, zum Beispiel aus der letzten Steuererklärung (#229). Nur die Vermögenssteuer rechnet damit (Steuerwert
minus Hypothek); ein Betrag behält sein Verhältnis zum Marktwert und wächst mit ihm. Alles andere rechnet mit dem
Marktwert. Ohne eigenen Steuerwert rechnen neue Planungen mit der **Vorgabe von 50 % des Marktwerts** (#273), einer
einheitlichen, gekennzeichneten Annahme für alle Kantone; die Planung speichert nur das Kennzeichen, den Anteil bestimmt
der Rechenkern. Ältere Planungen rechnen ohne Steuerwert weiter mit dem Marktwert; ein Hinweis in der Karte der
Liegenschaft bietet die Vorgabe an. Die Jahreswerte zeigen bei den Steuerspalten den Steuerwert der Immobilien als
eigene Spalte.

Der Schalter **«Immobilien nie verkaufen»** prüft, ob die Planung auch ohne Verkauf aufgeht: Für alle Liegenschaften
entfallen geplante Verkäufe, Notverkäufe und Verkäufe wegen fehlender Tragbarkeit bei der Erneuerung (die Hypothek wird
dann voll erneuert). Käufe mit Datum finden statt, Käufe und Mieten «nach Verkauf» nicht. Reicht das liquide Vermögen
nicht mehr, gilt es als aufgebraucht. Jeder Lauf zählt die Erneuerungen, die nicht tragbar waren und nur dank des Schalters
voll erneuert wurden; die Ergebnisse vieler Läufe zeigen neben den Notverkäufen, in wie vielen Läufen das vorkam
(Kennzahl «Nicht tragbare Erneuerungen»), die Kommandozeile im Bericht und im JSON.

## Wie genau ist die Steuer?

Die Steuer ist eine **Schätzung für die Planung**, kein Steuerbescheid (#143). Die Oberfläche nennt sie «geschätzte
Steuer» (Kennzahl, Diagrammlegenden, Jahreswerte, Vergleich, Übersicht der Annahmen) und zeigt Steuerbeträge auf
**100 Franken gerundet**; gerechnet wird genau. Die Kommandozeile und ihr CSV-Export geben die genauen Werte aus und
nennen sie ebenfalls geschätzt (Spalte `taxes_estimated`). Der Abschnitt Steuern zeigt einen festen Hinweis, unter den
Jahreswerten steht «Geschätzte Steuer mit Ihren eigenen Tabellen, auf 100 Franken gerundet. Keine Steuerberatung.» (im
Modus «Wohnort»: «… mit den Steuerdaten 2026 (Quelle: …) und den Standardabzügen des Kantons …», Quelle aus der
Steuerdatei, Herkunft der Abzüge je Planung: Standardabzüge des Kantons, eigene Abzüge oder beides; dieselbe Zeile im
Vergleich unter den Jahreswerten «Geschätzte Steuern total» und unter der Grafik der Geldflüsse mit den Steuern), und
bei Kapitalbezugs- und Grundstückgewinnsteuer der Rat, vor dem Entscheid mit dem Steuerrechner des Kantons oder einer
Fachperson zu prüfen. Im Modus «Wohnort» nennt die Karte «Wohnort» fest Steuerjahr und Quelle der Steuerdatei und die
Näherungen des Kantons (⚠: Berufskosten, Kapitalbezug, Grundstückgewinn in den Kantonen der Immobilien). Ist das laufende
Kalenderjahr grösser als das Steuerjahr der Daten + 1, meldet die Prüfung eine Warnung (`tax_data_outdated`). Eine Bandbreite (± %) zeigt die App bewusst nicht: Dafür fehlt eine Grundlage, und eine weitere Zahl
würde die Scheingenauigkeit verstärken.

- **Referenzfälle:** Von Hand nachgerechnete Fälle prüfen die Regeln (Satz linear interpoliert mal Steuerfuss,
  Kapitalbezüge eines Jahres zusammen, Stufen und Besitzdauer der Grundstückgewinnsteuer); die Vorlage Zürich der
  Grundstückgewinnsteuer stimmt mit der amtlichen Tabelle überein. Modus «Wohnort»: 25 erfundene Fälle in fünf Kantonen
  (Rentner, ohne Kinder und Konfession) weichen weniger als 1 % vom ESTV-Steuerrechner 2026 ab (verlangt ±5 %; bleiben als zweite Prüfung, #292);
  25 Fälle mit Erwerbstätigen (alleinstehend, verheiratet mit einem oder zwei Erwerbseinkommen) ebenfalls unter 1 %.
  Kapitalbezug im Modus «Wohnort»: 48 erfundene Fälle in acht Kantonen
  (100 000 bis 1 Million, alleinstehend und verheiratet) höchstens ±5 %; Wallis angenähert, drei Bezüge in Sitten unter 1 % neben dem Walliser Steuerrechner (#296); Zug nach dem Gesetz,
  10 Fälle gegen den Zuger Steuerrechner auf 0,5 % genau (#288). Zusätzlich Fälle aus den kantonalen
  Steuerrechnern von Zürich, Aargau, Graubünden und Wallis (Sitten, Agarn; Verheiratete in Sitten mit Ermässigung, #297) und aus den amtlichen Tarifblättern von LU, SO, BL, VD, JU und GE
  (#287–#291).
  Grundstückgewinnsteuer im Modus «Wohnort»: je Kanton erfundene Fälle gegen von Hand nachgerechnete Werte nach
  Gesetz.
- **Näherungen:** Durchschnittssätze je Betrag, dazwischen linear interpoliert; ein Steuerfuss für Kanton und Gemeinde;
  Ausgleich der kalten Progression mit der simulierten Teuerung; Grundstückgewinn = Verkaufswert − Kaufpreis.
- **Abzüge vom Einkommen (#131):** zeitabhängige Zeilen, je «Vorschlag des Kantons» (Modus «Wohnort», Standard:
  Standardabzüge der Steuerdatei, Betrag des ersten Jahres mit Bestandteilen für Kanton und Bund) oder «eigener Wert»
  (Betrag pro Jahr in Preisen des Starts, wächst mit der Teuerung der Tarife, optional Prozentsatz des Einkommens, für
  Kanton und Bund; Eingabehilfe aus Einkommen und steuerbarem Einkommen der Veranlagung). Neue Planungen: Zeilen «bis
  Pensionierung» und «ab Pensionierung». Dazu abschaltbar die Einzahlungen in die Säule 3a (mit indirekter
  Amortisation), höchstens der Maximalbetrag je Person der Einstellungen; für eine Person ohne Pensionskasse 20 % ihres
  Erwerbseinkommens im Jahr, höchstens der Maximalbetrag der grossen Säule 3a (#152). Steuerbares Einkommen nie negativ;
  Hypothekarzinsen nie automatisch (Wohneigentum gehört in den eigenen Wert). Bestehende Planungen rechnen wie bisher.
  Die Jahreswerte zeigen mit den Steuerspalten Einkommen und Abzüge mit Herkunft.
- **Steuerbares Einkommen in den Jahreswerten (#281):** Mit den Steuerspalten steht direkt vor den Steuern die Spalte
  «Steuerbares Einkommen»: steuerbares Einkommen des Steuerjahres beim Kanton (nach den Abzügen, nie negativ, ohne
  Kapitalbezüge); weicht der Bund ab, zeigt der Tooltip beide Werte. Jahre ohne Veranlagung zeigen 0. Die Spalte folgt
  der Umschaltung nominal / heutige Kaufkraft. Kommandozeile (`vivaplan run --json`, Spalte `taxableIncome`) und
  `vivaplan mcp` (`result_get`, Ansicht `yearly`, Feld `taxableIncome`) liefern den Wert ebenfalls.
- **Zusätzliche und einmalige Abzüge (#230):** Zum Vorschlag des Kantons lässt sich je Zeile ein Betrag pro Jahr
  dazurechnen («zusätzlich», Preise des Starts, für Kanton und Bund). Einmalige Abzüge (Steuerjahr, Beschreibung,
  Betrag nominal, zum Beispiel PK-Einkauf, Sanierung, Krankheitskosten) zählen nur in ihrem Steuerjahr zusätzlich zur
  geltenden Zeile, bei beiden Arten von Zeilen und in beiden Modi; die Schätzung des Netto-Bedarfs (3-Topf) nutzt nur
  die laufenden Abzüge. Die Herkunft «zusätzlich» und «einmalig» steht im Tooltip der Abzüge.
- **Abzüge aus der Steuerrechnung (#274):** Im Modus «Wohnort» öffnet «Aus der Steuerrechnung übernehmen» (Karte
  «Wohnort») einen Dialog für die erste Seite der letzten definitiven Steuerrechnung: Steuerjahr, Total der Einkünfte,
  steuerbares Einkommen (Kanton), steuerbares Vermögen, freiwillig Total Steuern. Bis zur Pensionierung gilt der Abzug
  (Einkünfte − steuerbares Einkommen, ohne die separat abgezogene Säule 3a) als eigener Wert ab Start; ab der
  Pensionierung der Vorschlag des Kantons ohne Erwerb plus der persönliche Rest (Abzug − Vorschlag mit Erwerb des
  Haushalts, nie negativ), so dass Berufskosten, Zweitverdienerabzug und Säule 3a wegfallen. Liegt die Pensionierung vor
  dem Start, gilt der Abzug ab Start. Zum steuerbaren Vermögen zeigt der Dialog die Differenz zur Planung (31.12. des
  ersten Jahres, nach den aus dem Vermögenstarif geschätzten Sozialabzügen) und schlägt einen Anteil des Steuerwerts
  oder einen Abzug vom Vermögen vor, übernommen nur mit Häkchen. Mit Total Steuern zeigt er die eigene Schätzung für
  dieselben Werte und die Abweichung, ab 15 % mit möglichen Ursachen; kein Korrekturfaktor. Die Herkunft «aus
  Steuerrechnung <Jahr>» steht in der Karte, in den Annahmen, im Tooltip der Abzüge und in den Parametern des Laufs.
- **Darstellung der Abzüge vom Einkommen (#241):** Je Zeile eine Kopfzeile (ab, Vorschlag des Kantons oder eigener
  Wert, Summe «Kanton … · Bund …» bzw. der eigene Betrag); «Details» klappt die Teile (Sozialabzüge,
  Versicherungsprämien, Sparzinsen, Berufskosten, Zweitverdienerabzug, zusätzlich) für Kanton und Bund mit Total für
  das erste Jahr der Zeile auf. Das Feld «zusätzlich» erscheint nur mit Betrag oder nach «+ Zusätzlicher Abzug»; die
  Erklärungen und die Höchstbeträge der Säule 3a stehen im ⓘ; einmalige Abzüge unter eigenem Titel.
- **Abzüge im Einzelnen unter «Experte» (#275):** Die Zeilen der Abzüge vom Einkommen (mit «zusätzlich» und der
  Hilfe «eigener Wert»), die Abzüge vom Vermögen, der Hinweis zu den Tarifen mit der Teuerung und bei bestehenden
  Planungen die eigenen Tabellen stehen im zugeklappten Bereich «Experte» (Abzüge im Einzelnen, Abzüge vom Vermögen,
  Tarife) unter den Annahmen. Er öffnet sich von selbst, wenn die Planung dort eigene Werte hat (Zeile mit eigenem
  Wert, zusätzlichem Abzug oder aus der Steuerrechnung, Abzüge vom Vermögen, eigene Tabellen).
  Das Häkchen der Säule 3a bleibt bei den Annahmen; die einmaligen Abzüge vom Einkommen (#230) stehen in einer eigenen
  Karte «Einmalige Abzüge» direkt im Abschnitt. Die Karte der Pensionskasse hat den Link «Einkauf als Abzug
  erfassen»: Er legt einen einmaligen Abzug «PK-Einkauf» (bei zwei Personen mit dem Namen) im Jahr nach dem Start an,
  wechselt zu den Steuern und setzt den Fokus auf das Steuerjahr. Rechnung und Dateiformat unverändert.
- **Abzüge vom Vermögen (#231):** Karte «Abzüge vom Vermögen» mit zeitabhängigen Zeilen «zusätzlich … pro Jahr»
  (Preise des Starts, mit der Teuerung der Tarife) und einmaligen Abzügen (Steuerjahr, Beschreibung, Betrag in
  Preisen des Starts, wächst mit der Teuerung bis zum Steuerjahr, #306),
  zum Beispiel Schulden ausser der Hypothek. Steuerbares Vermögen = Vermögen am 31.12. (Immobilien zum Steuerwert) −
  Abzüge, nie negativ; es zählt die Zeile vom 31.12., ein einmaliger Abzug am 31.12. seines Steuerjahres. Im Modus
  «Wohnort» kommen die Abzüge zu den Sozialabzügen des Kantons in den Vermögenskurven dazu und ersetzen sie nicht; mit
  eigenen Tabellen sind sie der ganze Abzug. Die Steuerzerlegung zeigt die Abzüge mit Herkunft und das steuerbare
  Vermögen.
- **Steuerbares Vermögen in der Karte (#249):** Die Karte «Abzüge vom Vermögen» zeigt zuoberst das geschätzte
  steuerbare Vermögen des ersten Steuerjahres mit dem Anteil der Liegenschaften und warnt, wenn einer Immobilie der
  Steuerwert fehlt (dann zählt der Marktwert). Die Details setzen es für ein wählbares Jahr zusammen wie die
  Steuererklärung (Vermögen ohne Immobilien, Steuerwert mit Herkunft «Vorgabe» (#273) und Hypothek je Liegenschaft,
  Abzüge) und bieten das Feld
  «Steuerwert» je Liegenschaft.
- **Was fehlt:** Abzüge im Einzelnen (nur als eigener Wert oder in den Sätzen), Eigenmietwert, Schuldzins- und
  Unterhaltsabzug (nur im eigenen Wert), Kirchen- und Personalsteuer sowie direkte Bundessteuer (soweit nicht in Tabellen oder Steuerfuss),
  Investitionen und Kosten beim Grundstückgewinn, bei der Grundstückgewinnsteuer nach Kanton die Selbstnutzung je
  Immobilie und die Anpassung der Anlagekosten an die Teuerung, künftige Gesetzesänderungen.

Im Modus «Wohnort» nennt der Abschnitt Steuern die Steuerfüsse der Gemeinde, das Steuerjahr und die Quelle der Daten
und dass die Steuer eine Schätzung mit den Standardabzügen des Kantons ist; Berufskosten, für die nur ein fester Betrag
erfasst ist (BL, BS, TI), sind mit ⚠ markiert. Unter der Gemeinde steht die Regel des Kantons für
Kapitalbezüge, angenäherte Regeln (Wallis) mit ⚠; die Tabellen für den Kapitalbezug sind ausgeblendet. Die Warnung bei alten Daten folgt mit #143.

## Datenbibliothek

Die Renditen stammen aus den Indizes der Anlagestrategie. Dahinter stehen diese Daten:

- **Welt (JST):** Aktien von 11 Ländern aus der Jordà-Schularick-Taylor Macrohistory Database (Release 6), jedes Jahr
  nach realem BIP gewichtet, in CHF, jährlich 1900–2020; dazu 10-jährige Obligationen der Eidgenossenschaft (JST bis
  1988, danach SNB). Daraus auch «Schweiz (JST)» und «Welt individuell». Lizenz CC BY-NC-SA 4.0 (nicht kommerziell).
  Der Index liegt rund 1,3 Prozentpunkte pro Jahr über dem Weltindex von Dimson, Marsh und Staunton (Begründung in
  `docs/Weltindex JST.md`).
- **Shiller USD und CHF:** Robert Shillers US-Daten ab 1872 (Aktien inklusive Dividenden, Staatsanleihen, Inflation),
  dieselben Daten in Franken umgerechnet mit dem Schweizer Landesindex der Konsumentenpreise und dem SARON, dazu eine
  Variante mit 10-jährigen Obligationen der Eidgenossenschaft.

Neuere Shiller- und SNB-Dateien lassen sich selbst herunterladen und importieren. Eigene Zeitreihen im CSV-Format
verwendet die App nicht mehr. Die Reihe «Pictet CH» wird aus Lizenzgründen nicht mehr mitgeliefert (#90); ältere Planungen erhalten beim Öffnen die passende Anlagestrategie. Alle Indizes sind in
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
lassen sich nominal oder in heutiger Kaufkraft anzeigen; die Ansicht öffnet sich in heutiger Kaufkraft. Bei mehreren
Läufen sind die Perzentile nach dem Endvermögen der gewählten Darstellung geordnet (in heutiger Kaufkraft nach dem
realen, nominal nach dem nominalen); weil die Inflation je Lauf verschieden ist, kann ein Perzentil je Darstellung zu
einem anderen Lauf gehören. Kennzahlen, Tabelle, Einzelläufe und Jahreswerte gehören immer zum selben Lauf. Bericht
(`--json`) und Kurzfassung für die KI nennen beide Ordnungen.

- **Ein Lauf:** Vermögensverlauf; liquides Vermögen nach Aktien, Obligationen und Cash; Einnahmen und Ausgaben pro
  Jahr; Entnahmerate; Immobilien; Vorsorgeguthaben; der Verlauf von Aktien und Obligationen als Index; Ereignisliste.
- **Mehrere Läufe:** Bandbreite des Vermögens (5. bis 95. Perzentil mit 1. und 99. Perzentil und dem schlechtesten
  Wert), Wahrscheinlichkeit, dass das Geld reicht, Verteilungen von Endvermögen und Vermögen bei Pensionierung, Alter bei
  Erschöpfung. Die Läufe am 1., 5., 50., 95. und 99. Perzentil und der schlechteste Lauf lassen sich im Detail ansehen.
- **Szenarien:** Kurven je Variante, bei zwei Parametern Heatmaps.

**Liste der Simulationen:** Jede Simulation wird mit der Planung von damals gespeichert. Der automatische Name nennt
Planung, Renditequelle und Strategie in Kurzform (z. B. «MC hist. · 3-Topf + Guard»). Die Liste lässt sich umbenennen,
mit Notizen versehen, aufräumen und zum **Vergleichen** von bis zu zehn Ergebnissen nutzen (Optimierungen stehen
ebenfalls in der Liste, werden beim Vergleichen aber ausgelassen). Der Vergleich bietet dieselben
Möglichkeiten wie ein einzelnes Ergebnis: Kennzahlen (nach jeder Spalte sortierbar), Bandbreite, Perzentile,
Wahrscheinlichkeit und Erschöpfungsalter, die Verläufe des Einzel- bzw. Median-Laufs (Aktien, Obligationen und Inflation
auch als Änderung in % oder logarithmisch), Jahreswerte mit einer Spalte je Ergebnis und die Eingaben, die sich
unterscheiden, mit den Bezeichnungen des Editors; nominal oder in heutiger Kaufkraft.

## Hilfe und Bedienung

Eine **kontextabhängige Hilfe** öffnet auf **F1** rechts neben dem Editor und zeigt das Thema des Felds, des Abschnitts
oder der Grafik, an der man gerade arbeitet. Das Handbuch mit Suche erklärt jeden Abschnitt, vor allem die Renditen und die
Optimierung ausführlich. Menüs und die üblichen Tastenkürzel (neu, öffnen, speichern, Simulation starten, Rückgängig,
Wiederholen, Wechsel der Abschnitte) sind vorhanden, ebenso ein «Über»-Fenster mit Version und Buildzeit.

**VivaPlan unterstützen (#237):** Wer VivaPlan nützlich findet, kann den Autor freiwillig über «Buy me a coffee»
unterstützen: Menü *Hilfe → VivaPlan unterstützen…*, eine unauffällige graue Zeile «☕ VivaPlan unterstützen» unten in der
Navigation und die Zeile «Unterstützen» im «Über»-Fenster öffnen https://buymeacoffee.com/vivaplan im Standardbrowser des
Systems. Kein Popup, kein Hinweis beim Start. Die App lädt dafür nichts aus dem Netz (kein Logo, kein Zähler, keine
Prüfung der Seite); sie öffnet nur Adressen aus einer festen Liste (`crates/app/ui/links.json`).

**Einführungsvideos (#240):** Menü *Hilfe → Einführungsvideos …* zeigt die Videos «VivaPlan in 2 Minuten» (Überblick)
und «VivaPlan in 10 Minuten» (erste Planung) mit Länge und dem Hinweis, dass ein Klick den Browser öffnet. Der
Willkommensdialog hat unter den drei Auswahlen den Link «VivaPlan in 2 Minuten ansehen», die Hilfeseiten «Was VivaPlan
kann» und «Erste Planung in zehn Minuten» verlinken das passende Video. Jedes Video gibt es in Deutsch, Englisch,
Französisch und Italienisch (Internet Archive); die App öffnet das Video in der eingestellten Sprache im Standardbrowser.

**Links in den Browser:** Nur diese Links öffnen den Browser: die Einführungsvideos und der Spendenlink. Ihre Adressen
stehen fest in der App (`crates/app/ui/links.json`); andere Adressen öffnet die App nicht, und sie selbst lädt dabei
nichts aus dem Netz.

**Darstellung:** «System» (Vorgabe), «Hell» oder «Dunkel» (#220). «System» folgt dem hellen oder dunklen
Erscheinungsbild des Betriebssystems, auch wenn es zur Laufzeit wechselt. Das gewählte Thema gilt für die ganze
Oberfläche, auch für Auswahlmenüs, Eingabefelder, Dialoge, Hilfe und Diagramme; das helle Thema sieht unter einem
dunklen Systemthema gleich aus wie unter einem hellen. Die Wahl gilt für das Gerät, nicht für die Planung.

**Sprache:** Die Oberfläche bezieht alle Texte aus Texttabellen je Sprache (#170). Beim ersten Start gilt die Sprache des
Betriebssystems (ohne Region, fr_FR → Französisch); fehlt die Sprache, gilt Deutsch. Gewählt wird sie im Menü
«Ansicht → Sprache» oder in den Einstellungen («wie System» oder eine vorhandene Sprache); der Wechsel wirkt sofort, auch
auf die Menüs, und lässt Planung, ungespeicherte Änderungen und Ansicht unverändert. Die Wahl gilt für das Gerät, nicht
für die Planung. Zahlen, Beträge und Daten bleiben in jeder Sprache im Schweizer Format (1'234'567.50 CHF, 2.5 %,
03.2040). Verfügbar sind Deutsch, Englisch (#172), Französisch (#174) und Italienisch (#175): Oberfläche, Menüs, Dateidialoge und Meldungen des Rechenkerns,
auch die Namen der Strategien und Parameter der Optimierung, der Indizes und die Beschreibungen der mitgelieferten
Zeitreihen (#177);
Schweizer Fachbegriffe stehen auf Englisch mit dem Kürzel in Klammern, z. B. «old-age pension (AHV)», «pillar 3a»,
auf Französisch und Italienisch als Schweizer Amtsbegriffe, z. B. «rente AVS», «caisse de pension», «pilier 3a», «impôt
sur les prestations en capital» bzw. «rendita AVS», «cassa pensione», «pilastro 3a», «imposta sulle prestazioni in
capitale» (Glossar `docs/GLOSSAR.md`). Inhalte der Planung (Namen) bleiben unübersetzt; nur die Beispielplanung erhält beim Öffnen ihre Namen in der gewählten Sprache, mit denselben Zahlen (#184). Die Hilfe (Handbuch, F1, Suche) erscheint in der gewählten Sprache
(#173, #174, #175: Deutsch, Englisch, Französisch und Italienisch); fehlt ein Kapitel in einer Sprache, erscheint es
deutsch.

**Einstieg für neue Benutzer:** Beim allerersten Start fragt ein Willkommensdialog, ob man das Beispiel erkunden, eine
eigene Planung beginnen oder leer starten möchte (wieder über *Hilfe → Willkommen …*). Die Checkliste «Erste Schritte»
neben der Prüfung führt durch die acht Schritte des Schnellstarts und hakt erledigte Abschnitte selbst ab. Die
Ergebnisübersicht beginnt mit dem Ergebnis in Worten: Erfolgsquote mit Einordnung (hoch, mittel, tief), ungünstigster
Lauf und ein Vorschlag für den nächsten Schritt.

Für Fortgeschrittene gibt es eine Kommandozeile: rechnen, prüfen, optimieren, Monatswerte exportieren, Daten
importieren und die Rechenleistung messen. Sie gibt Texte, Meldungen und Ereignisse auf Englisch aus (#176), Zahlen und
Daten im Schweizer Format. Das Dateiformat ist als JSON-Schema dokumentiert. Konfigurationen eines
früheren Programms lassen sich importieren. Die Kommandozeile `vivaplan` liegt im Installationspaket neben der App
(macOS, Windows, Linux; ohne Eintrag im Suchpfad, die Hilfe nennt den Pfad) und rechnet wie die App mit deren
Datenordner: Zeitreihen, Grenzen und Tragbarkeit aus den Einstellungen; `--data-dir` nimmt einen anderen Ordner (#206).

Für Skripte und KI-Assistenten (#207): `run`, `validate`, `optimize`, `bench` und `diff` geben mit `--json` ein festes,
in der Hilfe dokumentiertes Format auf die Standardausgabe aus (kompakt: eine Zeile je Jahr, für die Beispielplanung
unter 20 000 Zeichen). `vivaplan set <planung> <pfad>=<wert> …` ändert Werte über die Feldpfade der Prüfmeldungen, bei
zeitabhängigen Werten auch die Stufe ab einem Zeitpunkt (`wealth.stockShare@retire=0.5`, `@age:62`, `@2035-01`). Der
Rechenkern prüft Pfad und Typ gegen das Schema; bei einem Fehler wird keine Änderung übernommen. `set` schreibt ohne
`-o` eine neue Datei mit der nächsten freien Nummer und ersetzt eine vorhandene nur mit `--overwrite` (nach einer
Sicherungskopie `.bak`). `vivaplan diff <a> <b>` listet je Unterschied Feldpfad, alten und neuen Wert.

Eigene Kennzahlen (#214): `vivaplan run <planung> --metric '<json>'` setzt eine Kennzahl aus Bausteinen zusammen –
Grösse (zum Beispiel Konsum, Gesamtvermögen, «Untergrenze bestimmt die Ausgaben»), nominal oder real, Zeitfenster
(ab/bis Meilenstein, Alter, die ersten n Jahre), Reduktion je Pfad (Jahressumme, Mittel, Minimum, Wert am Ende, erster
Zeitpunkt, Anteil Monate, Schwankung) und Aggregation über die Pfade (Quantil, Mittel, Wahrscheinlichkeit mit
Standardfehler, bedingtes Mittel). So lässt sich etwa «Median des realen Konsums in den ersten 15 Jahren ab
Pensionierung» ohne neuen Code beantworten. Die Kennzahlen werden während der Simulation gesammelt; die Optimierung
rechnet ihre Kennzahlen auf demselben Weg.

Experiment (#216): `vivaplan evaluate <planung> <experiment.json>` stellt eine Frage als eine JSON-Struktur – Änderungen
(`set`), Variationen (`vary`: Liste, Bereich, gekoppelter Wert, Gruppe; Feldpfade auch mit Zeitpunkt), Kennzahlen,
Nebenbedingungen (`{"metric", "op", "value"}`) und Ziel (maximieren, minimieren, gewichtete Liste oder das Ziel der
Optimierung). Das Experiment wird vor dem Rechnen vollständig geprüft (Fehlerliste mit Pfad, Grund, gültigen Werten).
Gesucht wird im Raster oder mit der Mustersuche auf wenigen Pfaden mit einer Marge auf die Nebenbedingungen; die besten
Varianten werden auf unabhängigen Pfaden bestätigt. Das Ergebnis ist eine kurze Rangliste mit beiden Werten,
Standardfehlern, Hinweisen («am Rand des Rasters») und dem vollständigen Experiment mit Version und Startwert. Ab 60 s
geschätzter Dauer rechnet `evaluate` nur mit `--yes`. Variationen, Strategievergleich und Optimierung sind Spezialfälle.

Katalog für KIs (#219): `vivaplan describe [<abschnitt>] [--plan <planung>] [--json]` sagt einem Programm oder einer
KI ohne Quellcode, was es ändern, variieren und messen kann: die Felder der Planung mit Typ, Einheit, Wertebereich und
Bedeutung, die Formen zeitabhängiger Werte, die Zustandsgrössen der Kennlinien, die Bausteine der Kennzahlen, die
Formen von Variationsraum und Ziel, drei Beispiel-Experimente, die beim Aufruf gemessenen Mikrosekunden je Lauf (rund
0,5 s, an der angegebenen oder der Beispielplanung) und die Grenzen des Modells. Der Katalog wird aus Schema und Code
erzeugt, ist englisch und bleibt unter 20'000 Zeichen; einzelne Abschnitte mit `describe <abschnitt>`.
Auswahlfelder (#304): Der Abschnitt `choices` (nur einzeln abgefragt) erklärt jeden Wert jedes Felds mit festen Werten
einzeln mit seiner Bezeichnung in der App und seiner Bedeutung, etwa `returns.source`: `bootstrap` ist «Monte Carlo ·
historisch», `rolling` «Alle historischen Startzeitpunkte»; bei den Entnahmestrategien auch «3-Topf + Guardrails»
(`three_buckets` mit `withdrawal.buckets.spending` = `guardrails`). Die Bezeichnungen stammen aus den Texttabellen der
Oberfläche (`ui/i18n`), in der Sprache der Oberfläche (Einstellung der App, bei «wie System» die Sprache der Umgebung,
sonst Deutsch); Werte ohne Auswahl in der App (z. B. Grundstückgewinnsteuer-Modelle `curve` und `return` aus den
Steuerdaten) haben nur die Bedeutung.

Agent Skill (#208): Dem Paket liegt die Anleitung `vivaplan-skill` (Ordner mit `SKILL.md`, offenes Format) für
KI-Assistenten mit Zugriff auf die Kommandozeile bei, etwa Claude Code oder Claude Desktop (Bereich «Code»). Sie
erklärt Befehle, wichtige Felder mit Einheiten (Anteile statt Prozent), nominal und real, die Grenzen der Aussagen und
den Haftungsausschluss. Sie ist englisch, die KI antwortet in der Sprache des Menschen. Die Skill weist die KI an, vor
dem ersten Lesen einer Planung darauf hinzuweisen, dass die gelesenen Daten an den Anbieter der KI gehen. Die Hilfe sagt,
wo die Skill liegt und wie man sie einrichtet.

MCP-Server (#210): `vivaplan mcp [--data-dir <ordner>] [--library <ordner>] [--names] [--read-only] [--allow-dir <ordner>]` verbindet VivaPlan mit
KI-Programmen, die das Model Context Protocol unterstützen (Claude Desktop, Claude Code, ChatGPT Desktop, LM Studio und
andere). Das Programm spricht über Standardein- und -ausgabe und öffnet keine Netzwerkverbindung. Lesend: `plan_open`, `plan_get`, `plan_schema`, `plan_validate`, `describe`, `result_list`,
`result_get`, `library_list`; rechnend (#211): `simulation_run` (die Planung wie in der App, mit ihren Variationen) und
`evaluate` (Experiment wie `vivaplan evaluate`; darüber laufen Optimierung und Vergleich). Vor jeder Rechnung schätzt
der Server die Dauer; ab 60 Sekunden rechnet er nur mit `"confirm": true`. Lange Rechnungen melden den Fortschritt
(`notifications/progress`) und lassen sich abbrechen (`notifications/cancelled`); es läuft eine Rechnung aufs Mal.
Ändernd (#212): `plan_set` (Werte per Feldpfad, ein Aufruf ist ein Schritt, alles oder nichts), `plan_changes`
(Unterschiede zur Datei), `plan_undo` (letzter Schritt oder alle) und `plan_apply` (Variante eines
`evaluate`-Ergebnisses nach Rang übernehmen) ändern nur die Arbeitskopie im Speicher. Nur `plan_save` schreibt, nach
sechs Regeln: ohne Pfad eine neue Datei `<name> (AI).vplan` neben der geöffneten; eine vorhandene Datei nur mit
`overwrite` und nach einer Rückfrage des KI-Programms an den Nutzer (Elicitation, sonst Ablehnung); vorher eine
Sicherungskopie `<datei>.bak`, geschrieben über eine Zwischendatei; keine seit dem Öffnen geänderte Datei; nie eine
`.rplan`-Datei; nur in den Ordner der Planung oder einen mit `--allow-dir` freigegebenen. `--read-only` bietet
`plan_save` nicht an.
Neue Planung (#269): `plan_open` mit `new: true` öffnet eine leere Planung ohne Datei, dieselbe wie *Datei → Neu* in
der App; beide baut der Rechenkern (`api::blank_plan`, Befehl `blank_plan`). Die Antwort nennt die noch fehlenden
Angaben mit Feldpfad (`missing`: Name der Person, Einkommen, Ausgaben, Vermögen, Pensionskasse, AHV), die angenommenen
Werte (`assumed`: Geburtsdatum, Alter der Meilensteine, Steuertarife) und was zusätzlich zu erfragen ist (Partner,
Säule 3a, Liegenschaften, einmalige Ausgaben). `new` zusammen mit `path` oder `example` ist ein Fehler. Die Anweisungen
des Servers sagen der KI, für eigene Zahlen so zu beginnen, Fehlendes beim Menschen zu erfragen und keine Werte zu
erfinden; die Beispielplanung dient nur zum Vorführen. Ausserdem (#304): Menschen nennen Auswahlfelder mit den
Bezeichnungen der App, die KI schlägt den Wert vor dem Setzen in `describe` mit Abschnitt `choices` nach (darauf
verweist auch die Beschreibung von `simulation_run` für die Renditequelle). Bei Fragen zu Entnahme, Vermögensverzehr
oder Ruinrisiko nennt sie die eingestellte Entnahmestrategie mit ihrer Bezeichnung in der App, weist auf die übrigen
hin und wechselt die Strategie nur auf Wunsch des Menschen. `plan_save` speichert eine Planung ohne Datei nur mit einem Pfad
in einem freigegebenen Ordner; ein relativer Pfad gilt dann im ersten freigegebenen Ordner, ein Dateiname allein
genügt (#302). Das sagen die Beschreibung von `plan_save.path` und der Hinweis `next` von `new: true`. `~/` am Anfang
von `path` steht bei `plan_save` und `plan_open` für den Benutzerordner wie bei `--allow-dir`; danach gilt Regel 6
unverändert. Fehlt der Zielordner oder liegt er ausserhalb, nennt die Meldung als Ausweg den Dateinamen allein, ohne
Ordnernamen.
Planordner (#263): Mit `--allow-dir <ordner>` (auch mehrfach) öffnet und speichert der Server Planungen nur in diesen
Ordnern und ihren Unterordnern; verglichen werden kanonische Pfade, so dass `..` und symbolische Links aus dem Ordner
hinaus abgewiesen werden (Fehler «outside the plan folder», nur mit dem Dateinamen). Der Datenordner der App bleibt
lesbar. Ohne `--allow-dir` bleibt alles wie oben. Die Begrenzung setzt der Server selbst durch, nicht das Betriebssystem
(keine Sandbox). Claude Desktop warnt beim Installieren jeder Erweiterung, sie erhalte Zugriff auf alles auf dem
Computer, weil das Paketformat keine Rechte kennt; der Server läuft mit den Rechten des Benutzers, in den anderen
KI-Programmen ebenso, nur ohne Warnung. Die Hilfe «KI-Programme verbinden» erklärt, was der Server tatsächlich tut.
Jede Rechnung geht über denselben Befehlsverteiler wie die App; Grenzen und Datenbibliothek kommen aus
dem Datenordner der App. Namen von Personen und frei eingegebene Bezeichnungen ersetzt der Server durch neutrale
(«Person 1»), ausser mit `--names`, und nennt nur Dateinamen; keine Antwort ist länger als 20'000 Zeichen. Das erste
Öffnen einer Planung weist darauf hin, dass die abgefragten Daten Teil des Gesprächs mit der KI werden und an deren
Anbieter gehen, und zeigt den vollen Haftungsausschluss; jede Antwort mit Ergebnissen endet mit der Kurzfassung.

Mit KI verbinden (#213): Menü *Hilfe → Mit KI verbinden …* zeigt Anleitungen für Claude Desktop, Claude Code, die
ChatGPT-Desktop-App (über die Konfiguration von Codex, `~/.codex/config.toml`) und LM Studio, je mit dem Ort der
Einrichtung und einer Einrichtungszeile mit dem Pfad des Programms `vivaplan` neben der installierten App, zum Kopieren
in die Zwischenablage. Jede Zeile enthält den Planordner als `--allow-dir` (#263): vorgeschlagen ist der Ordner der
zuletzt geöffneten Planung, sonst der Dokumente-Ordner, änderbar über «Ordner wählen …»; der Schalter «Nur lesen»
(Standard aus) ergänzt `--read-only`. Die App richtet nichts selbst ein und greift nicht aufs Netz zu. Für Claude Desktop unter macOS
enthält jede Version das Ein-Klick-Paket `VivaPlan_<Version>_aarch64.mcpb` (MCP Bundle mit `manifest.json`, dem
Programm `vivaplan` und dem Symbol); ein Doppelklick installiert VivaPlan als Erweiterung. Beim Installieren fragt
Claude Desktop nach dem Planordner (`user_config` vom Typ Ordner, Pflicht, Vorschlag Dokumente-Ordner) und dem Schalter
«Nur lesen» (Read only, Standard aus); das Paket übergibt sie als `--allow-dir <ordner>` und `--read-only=true|false`. Die Hilfe zum Datenschutz
erklärt: Mit einer Cloud-KI gehen die gelesenen Daten an deren Anbieter, mit einem lokalen Modell bleiben sie auf dem
Rechner. Der Dialog sagt dazu, dass VivaPlan Namen ersetzt, Beträge und Daten aber an den Anbieter gehen, und dass
VivaPlan in Inkognito-Chats von Claude nicht zur Verfügung steht (#298); der Absatz «Inkognito-Chats» unter
«KI-Programme verbinden» nennt die Alternativen (normaler Chat und danach löschen, Gedächtnis ausschalten, lokales
Modell).

## Installation

Pakete für **Windows** (.msi, .exe), **Linux** (.deb, .AppImage; .rpm nur ohne Vorabversion) und **macOS** (.dmg) baut
ein GitHub-Workflow beim Setzen eines Versions-Tags und legt sie als Release-Entwurf ab (#203). Von Hand gestartet baut er
nur die gewählten Plattformen, auf Wunsch für ein bestehendes Tag. Die Pakete sind nicht signiert, Windows und macOS
zeigen deshalb beim ersten Start eine Warnung.

**Beta-Token (#270):** Während der geschlossenen Beta lassen sich die Desktop-App und die KI-Anbindung (`vivaplan mcp`)
erst nach einer einmaligen Aktivierung mit einem persönlichen, signierten Beta-Token benutzen (eine Zeile
`VP1.beta.<Kennung>.<Kürzel>.<Einlösefrist>.<Signatur>`, Ed25519). Ohne Aktivierung zeigt die App nur den Dialog
«Beta-Token eingeben»; der MCP-Server beantwortet jeden Werkzeugaufruf mit dem Hinweis, VivaPlan sei nicht aktiviert.
Das Datum im Token ist eine **Einlösefrist**: Nur bis dann (einschliesslich) lässt sich das Token eingeben; eine
aktivierte Installation läuft danach dauerhaft weiter, auch nach Updates. Die Aktivierung (Token und Tag) liegt in
`beta-aktivierung.json` im Datenordner der App; App und MCP-Server teilen sie. Das Ein-Klick-Paket fragt bei der
Installation nach dem Feld «Beta token» (freiwillig, wenn die App schon aktiviert ist); ein gültiges Token aktiviert
dann auch die App. «Über VivaPlan» zeigt Kürzel und Tag der Aktivierung. Die Prüfung läuft im Rechenkern, ohne
Netzwerk. Die übrigen Befehle der Kommandozeile und der Entwicklungsserver brauchen keine Aktivierung. Das Token ist
eine Hürde, kein Schutz: Der Code ist öffentlich (GPL), die Uhr lässt sich beim Einlösen zurückstellen und eine
kopierte Aktivierungsdatei aktiviert auch einen anderen Rechner. Ab Version 1.0.0 entfällt die Prüfung.

## Verlässlichkeit

Die Rechenregeln sind in `docs/ENTWICKLUNG.md` Schritt für Schritt beschrieben. Jede Regel ist mit einem von Hand nachgerechneten
Testfall belegt. Dazu kommen:

- Prüfungen an Hunderten zufälliger Planungen (zum Beispiel ändert sich das Gesamtvermögen jeden Monat genau um Einnahmen,
  Ausgaben, Steuern, Renditen und Zinsen: Kein Geld entsteht oder verschwindet, auch nicht bei Kauf und Verkauf von
  Liegenschaften, Hypotheken, Pensionskasse und Säule 3a),
- statistische Tests der Monte-Carlo-Pfade,
- Robustheitstests mit absichtlich fehlerhaften Eingaben,
- zehn eingecheckte Referenzplanungen (Golden Masters), die jede unbeabsichtigte Änderung der Ergebnisse melden,
- ein Vergleich mit einer Referenztabelle, die auf den Rappen übereinstimmen muss,
- automatische Tests aller Ansichten und der Bedienung der Oberfläche.

**Haftungsausschluss:** VivaPlan ist kostenlos und liefert Modellrechnungen ohne Gewähr, keine Steuer- oder
Anlageberatung (#143); die Steuern sind Schätzungen. Beim ersten Start bestätigt man eine Kurzfassung (bei einem neuen Text erneut), die Ergebnisseiten und
das Ergebnis der Optimierung tragen eine Fusszeile, der vollständige Text steht in der Hilfe und ist über
«Über VivaPlan» erreichbar. Die Kommandozeile nennt die Kurzfassung in ihrer Hilfe.

## Technik

Rechenkern und Kommandozeile in Rust, Desktop-App mit Tauri 2, Oberfläche in HTML und JavaScript ohne externe
Bibliotheken. Lizenz: GPL v3 für den Programmcode; die mitgelieferten Zeitreihen fallen nicht darunter.
