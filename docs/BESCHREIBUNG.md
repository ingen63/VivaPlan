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
  zahlt jede Person die Kapitalbezugssteuer auf ihre eigenen Bezüge.
- **Meilensteine:** Zeitpunkte als Datum oder als Alter einer Person («mit 63»). Jede Person hat zwei feste,
  nicht löschbare Meilensteine: **Bezug Pensionskasse** (58 bis 65 Jahre) und **AHV-Bezug** (63 bis 70 Jahre), beide
  standardmässig 65. Alle anderen Angaben können sich auf Meilensteine beziehen; verschiebt man den Bezug der
  Pensionskasse, verschiebt sich alles, was daran hängt. Das Ende des Erwerbseinkommens steht im Einkommen; läuft es nach
  dem Bezug der Pensionskasse weiter, warnt die Prüfung.
- **Vorsorge:** Pensionskasse mit Guthaben, Beiträgen, Verzinsung, Umwandlungssatz und Kapitalbezug in Prozent;
  AHV-Rente mit oder ohne Teuerungsanpassung, eingegeben als Rente mit 65; bei Bezug vor oder nach 65 kürzt bzw.
  erhöht die App sie selbst nach den Sätzen der AHV (Vorbezug 6,8 % je Jahr, höchstens 13,6 %; Aufschub 5,2 % bis
  31,5 %; feste Beträge in Franken, bei Ehepaaren nach dem Plafond auf die ungekürzten Renten), neben dem Feld steht
  die effektive Rente; ab Dezember 2026 jedes Jahr im Dezember die 13. AHV-Rente (1/12 der im
  Jahr ausbezahlten AHV-Renten, anteilig bei Beginn im Laufe des Jahres).
- **Säule 3a:** ein Konto je Person mit Guthaben, Verzinsung, Einzahlung pro Jahr (bis zum Maximalbetrag; für eine
  Person «Ohne Pensionskasse (grosse Säule 3a)» bis 20 % des Erwerbseinkommens, höchstens der Maximalbetrag der grossen
  Säule 3a, #152; zählt zu den Ausgaben, bis vor dem AHV-Bezug) und Auszahlungsplan (frühestens 5 Jahre vor dem AHV-Bezug); der Rest wird spätestens
  im AHV-Bezug ausbezahlt. Der Zeitpunkt einer Auszahlung ist ein Datum oder ein Alter der Kontoinhaberin bzw. des
  Kontoinhabers; jede Zeile zeigt das Guthaben vor und nach der Auszahlung aus der Vorschau, und eine neue Zeile erhält
  als Betrag das Guthaben zu ihrem Zeitpunkt.
- **Einnahmen & Ausgaben:** Erwerbseinkommen, Lebenshaltung und einmalige Beträge (Erbschaft, Autokauf, Reise). Jeder
  Wert kann zeitabhängig sein, zum Beispiel tiefere Ausgaben ab 80.
  Das Erwerbseinkommen wird pro Jahr erfasst wie in der Steuererklärung («Einkünfte aus Erwerb netto / Jahr», Nettolohn
  im Lohnausweis Ziffer 11); daneben steht der Betrag pro Monat. Gespeichert und gerechnet wird pro Monat (ein Zwölftel).
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
- **Steuern:** Modus «Wohnort» (#63): Einkommens- und Vermögenssteuer aus Kanton, Gemeinde (alle Gemeinden des
  Steuerjahres, Auswahl mit Suche) und Kirchensteuer (Konfession) nach den Steuerdaten der ESTV, mit den
  Standardabzügen von Kanton und Bund (Sozialabzüge, Versicherungsprämien und Sparzinsen, mit Erwerb in der Variante
  «mit 2. und 3. Säule», Berufskostenpauschale, bei Ehepaaren mit zwei Erwerbseinkommen der Zweitverdienerabzug;
  statt dessen je Zeitraum ein eigener Wert, dazu die Säule 3a, #131) und
  der direkten Bundessteuer; die Steuerdatei ist eingebettet, ohne
  Netzwerkzugriff. Im Modus «Wohnort» auch die Kapitalbezugssteuer (#64) nach der Regel des Kantons mit den
  Steuerfüssen der Einkommenssteuer und einem Fünftel des Bundestarifs, ohne eigene Tabelle; die Regel steht unter der
  Gemeinde, angenäherte Regeln (Wallis, Zug) mit ⚠. Modus «Eigene Tabellen»: Tabellen für Einkommens-, Vermögens- und
  Kapitalbezugssteuer mit Steuerfuss; bestehende Planungen öffnen in diesem Modus. Grundstückgewinnsteuer: im Modus
  «Wohnort» (#65) der Regelsatz des Kantons, in dem die Immobilie liegt (Feld Kanton je Immobilie, ohne Angabe der
  Wohnkanton; alle 26 Kantone nach dem ESTV-Steuermäppchen 2025 und, wo nötig, dem kantonalen Gesetz; Besitzdauer in
  Monaten; Steuerfuss der Wohngemeinde oder Median des Kantons; Regeltext bei der Immobilie, angenäherte Regeln von
  Basel-Stadt, Basel-Landschaft, St. Gallen, Waadt und Wallis mit ⚠); mit dem Häkchen «Eigene Regeln statt der Regeln
  des Kantons» und im Modus «Eigene Tabellen» die eigenen Regeln. Wohnort zeitabhängig (#151): weitere Wohnorte ab
  Jahr oder Ereignis; Einkommens- und Vermögenssteuer eines Jahres nach dem Wohnort am 31. Dezember für das ganze
  Jahr, Kapitalbezugssteuer nach dem Wohnort im Monat des Bezugs, Grundstückgewinnsteuer einer Immobilie ohne eigenen
  Kanton nach dem Wohnkanton im Monat des Verkaufs (StHG Art. 4b); Parameter und Jahreswerte nennen den Wohnort je
  Zeitraum bzw. Jahr. Die Sätze der Tabellen werden zwischen den Stufen
  linear interpoliert. Mit «Tarife mit der Teuerung» wachsen die Beträge der Tabellen für
  Einkommen, Vermögen und Kapitalbezug mit der simulierten Teuerung (Ausgleich der kalten Progression); für neue
  Planungen eingeschaltet, ältere Planungen und importierte Konfigurationen rechnen ohne.
- **Immobilien:** beliebig viele Liegenschaften und Mietverhältnisse.
- **Szenarien & Simulation:** Variationen und Einstellungen für mehrere Läufe.

**Einstellungen** (macOS: Menü «VivaPlan → Einstellungen…», Windows und Linux: «Datei → Einstellungen…»): Sprache, Darstellung, Grenzen für AHV-Bezug und Bezug Pensionskasse und der
Maximalbeträge der kleinen und der grossen Säule 3a. Sie gelten für alle Planungen auf dem Gerät und werden nicht in der Planungsdatei
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
Grundstückgewinnsteuer an. Hypotheken bestehen aus Tranchen mit aufeinanderfolgenden Perioden, fest oder SARON plus
Marge, mit Verlängerung oder Rückzahlung am Ende. Die Amortisation erfolgt direkt oder indirekt über die Säule 3a;
indirekt nur bis zum AHV-Bezug, danach direkt.

Beim Kauf prüft die App die Tragbarkeit wie eine Bank und kürzt die Hypothek nötigenfalls; auf Wunsch wird mit der
höchsten Belehnung von 80 % finanziert. Wird das liquide Vermögen knapp, verkauft die App eine Liegenschaft als Notverkauf.
Ist keine Liegenschaft mehr im Besitz, beginnen die erfassten Mieten.

Je Liegenschaft im Besitz oder mit geplantem Kauf lässt sich ein **Steuerwert** erfassen, als Betrag oder in % des
Marktwerts, zum Beispiel aus der letzten Steuererklärung (#229). Nur die Vermögenssteuer rechnet damit (Steuerwert
minus Hypothek); ein Betrag behält sein Verhältnis zum Marktwert und wächst mit ihm. Alles andere rechnet mit dem
Marktwert; ohne Steuerwert gilt der Marktwert wie bisher. Die Jahreswerte zeigen bei den Steuerspalten den Steuerwert
der Immobilien als eigene Spalte.

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
Modus «Wohnort»: «… mit den Steuerdaten 2026 der ESTV und den Standardabzügen des Kantons …»), und
bei Kapitalbezugs- und Grundstückgewinnsteuer der Rat, vor dem Entscheid mit dem Steuerrechner des Kantons oder einer
Fachperson zu prüfen. Eine Bandbreite (± %) zeigt die App bewusst nicht: Dafür fehlt eine Grundlage, und eine weitere Zahl
würde die Scheingenauigkeit verstärken.

- **Referenzfälle:** Von Hand nachgerechnete Fälle prüfen die Regeln (Satz linear interpoliert mal Steuerfuss,
  Kapitalbezüge eines Jahres zusammen, Stufen und Besitzdauer der Grundstückgewinnsteuer); die Vorlage Zürich der
  Grundstückgewinnsteuer stimmt mit der amtlichen Tabelle überein. Modus «Wohnort»: 25 erfundene Fälle in fünf Kantonen
  (Rentner, ohne Kinder und Konfession) weichen weniger als 1 % vom ESTV-Steuerrechner 2026 ab (verlangt ±5 %);
  25 Fälle mit Erwerbstätigen (alleinstehend, verheiratet mit einem oder zwei Erwerbseinkommen) ebenfalls unter 1 %.
  Kapitalbezug im Modus «Wohnort»: 48 erfundene Fälle in acht Kantonen
  (100 000 bis 1 Million, alleinstehend und verheiratet) höchstens ±5 %; Wallis und Zug nur angenähert.
  Grundstückgewinnsteuer im Modus «Wohnort»: je Kanton erfundene Fälle gegen von Hand nachgerechnete Werte nach
  Steuermäppchen oder Gesetz.
- **Näherungen:** Durchschnittssätze je Betrag, dazwischen linear interpoliert; ein Steuerfuss für Kanton und Gemeinde;
  Ausgleich der kalten Progression mit der simulierten Teuerung; Grundstückgewinn = Verkaufswert − Kaufpreis.
- **Abzüge vom Einkommen (#131):** zeitabhängige Zeilen, je «Vorschlag des Kantons» (Modus «Wohnort», Standard:
  Standardabzüge der Steuerdatei, Betrag des ersten Jahres mit Bestandteilen für Kanton und Bund) oder «eigener Wert»
  (Betrag pro Jahr in Preisen des Starts, wächst mit «Tarife mit der Teuerung», optional Prozentsatz des Einkommens, für
  Kanton und Bund; Eingabehilfe aus Einkommen und steuerbarem Einkommen der Veranlagung). Neue Planungen: Zeilen «bis
  Pensionierung» und «ab Pensionierung». Dazu abschaltbar die Einzahlungen in die Säule 3a (mit indirekter
  Amortisation), höchstens der Maximalbetrag je Person der Einstellungen; für eine Person ohne Pensionskasse 20 % ihres
  Erwerbseinkommens im Jahr, höchstens der Maximalbetrag der grossen Säule 3a (#152). Steuerbares Einkommen nie negativ;
  Hypothekarzinsen nie automatisch (Wohneigentum gehört in den eigenen Wert). Bestehende Planungen rechnen wie bisher.
  Die Jahreswerte zeigen mit den Steuerspalten Einkommen, Abzüge mit Herkunft und steuerbares Einkommen.
- **Zusätzliche und einmalige Abzüge (#230):** Zum Vorschlag des Kantons lässt sich je Zeile ein Betrag pro Jahr
  dazurechnen («zusätzlich», Preise des Starts, für Kanton und Bund). Einmalige Abzüge (Steuerjahr, Beschreibung,
  Betrag nominal, zum Beispiel PK-Einkauf, Sanierung, Krankheitskosten) zählen nur in ihrem Steuerjahr zusätzlich zur
  geltenden Zeile, bei beiden Arten von Zeilen und in beiden Modi; die Schätzung des Netto-Bedarfs (3-Topf) nutzt nur
  die laufenden Abzüge. Die Herkunft «zusätzlich» und «einmalig» steht im Tooltip der Abzüge.
- **Abzüge vom Vermögen (#231):** Karte «Abzüge vom Vermögen» mit zeitabhängigen Zeilen «zusätzlich … pro Jahr»
  (Preise des Starts, mit «Tarife mit der Teuerung») und einmaligen Abzügen (Steuerjahr, Beschreibung, Betrag nominal),
  zum Beispiel Schulden ausser der Hypothek. Steuerbares Vermögen = Vermögen am 31.12. (Immobilien zum Steuerwert) −
  Abzüge, nie negativ; es zählt die Zeile vom 31.12., ein einmaliger Abzug am 31.12. seines Steuerjahres. Im Modus
  «Wohnort» kommen die Abzüge zu den Sozialabzügen des Kantons in den Vermögenskurven dazu und ersetzen sie nicht; mit
  eigenen Tabellen sind sie der ganze Abzug. Die Steuerzerlegung zeigt die Abzüge mit Herkunft und das steuerbare
  Vermögen.
- **Was fehlt:** Abzüge im Einzelnen (nur als eigener Wert oder in den Sätzen), Eigenmietwert, Schuldzins- und
  Unterhaltsabzug (nur im eigenen Wert), Kirchen- und Personalsteuer sowie direkte Bundessteuer (soweit nicht in Tabellen oder Steuerfuss),
  Investitionen und Kosten beim Grundstückgewinn, bei der Grundstückgewinnsteuer nach Kanton die Selbstnutzung je
  Immobilie und die Anpassung der Anlagekosten an die Teuerung, künftige Gesetzesänderungen.

Im Modus «Wohnort» nennt der Abschnitt Steuern die Steuerfüsse der Gemeinde, das Steuerjahr und die Quelle der Daten
und dass die Steuer eine Schätzung mit den Standardabzügen des Kantons ist; Berufskosten, für die die ESTV-Daten nur
einen Höchstbetrag nennen (BL, BS, TI), sind mit ⚠ markiert. Unter der Gemeinde steht die Regel des Kantons für
Kapitalbezüge, angenäherte Regeln (Wallis, Zug) mit ⚠; die Tabellen für den Kapitalbezug sind ausgeblendet. Die Warnung bei alten Daten folgt mit #143.

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
lassen sich nominal oder in heutiger Kaufkraft anzeigen.

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
früheren Programms lassen sich importieren.

Für Skripte und KI-Assistenten (#207): `run`, `validate`, `optimize`, `bench` und `diff` geben mit `--json` ein festes,
in der Hilfe dokumentiertes Format auf die Standardausgabe aus (kompakt: eine Zeile je Jahr, für die Beispielplanung
unter 20 000 Zeichen). `vivaplan set <planung> <pfad>=<wert> …` ändert Werte über die Feldpfade der Prüfmeldungen, bei
zeitabhängigen Werten auch die Stufe ab einem Zeitpunkt (`wealth.stockShare@retire=0.5`, `@age:62`, `@2035-01`). Der
Rechenkern prüft Pfad und Typ gegen das Schema; bei einem Fehler wird keine Änderung übernommen. `set` schreibt ohne
`-o` eine neue Datei mit der nächsten freien Nummer und ersetzt eine vorhandene nur mit `--overwrite` (nach einer
Sicherungskopie `.bak`). `vivaplan diff <a> <b>` listet je Unterschied Feldpfad, alten und neuen Wert.

## Installation

Das Paket für **macOS** (.dmg) baut ein GitHub-Workflow beim Setzen eines Versions-Tags und legt es als Release-Entwurf
ab. Pakete für Windows und Linux werden bis auf Weiteres nicht gebaut (#202, wieder eingeschaltet mit #203). Das Paket ist
nicht signiert, macOS zeigt deshalb beim ersten Start eine Warnung.

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
