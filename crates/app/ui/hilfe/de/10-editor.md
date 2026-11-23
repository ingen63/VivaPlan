# Editor: Abschnitte und Felder

## @s-general Haushalt & Projekt
Stichworte: Name, Währung, Personen, Geburt, zweite Person, Double Income, Simulationszeitraum, ohne Pensionskasse, grosse Säule 3a, Selbständige

Hier legen Sie die Grunddaten fest.

- **Projektname und Währung:** Alle Beträge werden in der Projektwährung erfasst (CHF, EUR oder USD). Die Indizes der [Anlagestrategie](#f-ret-series) sind in CHF; bei einer Planung in EUR oder USD erscheint ein Hinweis.
- **Person 1:** Name und Geburtsmonat (Format MM.JJJJ). Aus dem Geburtsmonat berechnet die App das Alter für Meilensteine («mit 65») und für die Beschriftung der Grafiken.
- **+ Zweite Person:** Fügt eine zweite Person mit eigenem Einkommen, eigener Pensionskasse und eigener AHV hinzu (Double Income). Steuern, Vermögen und Ausgaben gelten für den Haushalt. Siehe [Zweite Person und Ehepaar-Regeln](#f-couple).
- **Ohne Pensionskasse (grosse Säule 3a):** Häkchen je Person für Erwerbstätige ohne Pensionskasse, vor allem Selbständige. Für diese Person gelten Einzahlung und Abzug der [Säule 3a](#s-pillar3a) bis **20 % des Erwerbseinkommens**, höchstens bis zum Maximalbetrag der grossen Säule 3a (36 288 Franken, änderbar in den [Einstellungen](#f-settings)); für alle anderen die kleine Säule 3a (7 258 Franken). Hat die Person in der Planung trotzdem eine Pensionskasse (Kapital oder Beiträge), zeigt die Prüfung eine Warnung.
- **Simulationszeitraum:** Start und Ende sind Meilensteine und werden unter [Meilensteine](#s-milestones) geändert.

Alle Zeitangaben haben das Format **MM.JJJJ** (zum Beispiel 05.2027).

**Zahlen** geben Sie mit oder ohne Tausendertrennzeichen ein: Apostroph oder Leerzeichen (1'234 oder 1 234), als Dezimaltrennzeichen Punkt oder Komma (12.5 oder 12,5). Bei **Beträgen** und **ganzen Zahlen** (zum Beispiel Anzahl Pfade) ist ein Punkt mit genau drei Ziffern danach mehrdeutig («12.500»): Das Feld meldet «Mehrdeutige Zahl» und behält den bisherigen Wert; schreiben Sie 12500 oder 12'500. Prozent- und Jahresfelder nehmen «1.25» wie bisher an. Schreibweisen wie «1e5» oder «0x10» sind nirgends erlaubt.

## @s-milestones Meilensteine
Stichworte: Bezug Pensionskasse, Pensionierung, Start, Ende, AHV-Bezug, AHV-Beginn, Datum, Alter, Zeitpunkt, Grenzen

Ein Meilenstein ist ein benannter Zeitpunkt, auf den sich alle anderen Eingaben beziehen können («ab Bezug Pensionskasse»). Verschieben Sie den Bezug der Pensionskasse, verschiebt sich alles mit, was daran hängt. Das ist der wichtigste Hebel, um Szenarien «früher oder später in Pension» durchzuspielen.

Fest vorgegeben sind **Simulationsstart** und **Simulationsende**. Dazu hat **jede Person** zwei feste Meilensteine, die immer vorhanden sind:

| Meilenstein | Bedeutung | Grenzen | Standard |
|---|---|---|---|
| **Bezug Pensionskasse** (bisher «Pensionierung») | Bezug von Rente und Kapital der Pensionskasse, Start der Entnahmestrategie | 58 bis 65 Jahre | 65 |
| **AHV-Bezug** | Beginn der AHV-Rente; Ende der Einzahlungen und der indirekten Amortisation in die [Säule 3a](#s-pillar3a), Auszahlung des Rests | 63 bis 70 Jahre | 65 |

Sie lassen sich nicht löschen, ihr Zeitpunkt ist aber frei wählbar (Alter oder Datum), solange er innerhalb der Grenzen liegt; die Grenzen selbst sind erlaubt. Die Grenzen sind in den [Einstellungen](#f-settings) änderbar. Bei zwei Personen heissen sie «Bezug Pensionskasse Person 2» und «AHV-Bezug Person 2». Die Namen der festen Meilensteine erscheinen in der gewählten Sprache und werden nicht in der Planungsdatei gespeichert. Das **Ende des Erwerbseinkommens** ist kein Meilenstein: Sie legen es im [Einkommen](#s-flows) fest. Ist das Erwerbseinkommen nach dem Bezug der Pensionskasse noch grösser als 0, zeigt die Prüfung eine Warnung mit dem Betrag.

Eigene Meilensteine fügen Sie mit «+ Meilenstein» hinzu, zum Beispiel «Verkauf Haus» oder «Kinder ausgezogen».

Jeder Meilenstein ist festgelegt als:

- **Datum:** ein fester Monat (05.2027)
- **Alter von:** das Alter einer Person, zum Beispiel «Person 1 mit 64.5 Jahren». Dezimalstellen sind erlaubt: 64.5 Jahre sind 64 Jahre und 6 Monate.

Die Spalte «Ergibt» zeigt den daraus berechneten Monat. **Beim Wechsel** zwischen «Datum» und «Alter von» bleibt der Zeitpunkt derselbe: Das Alter bzw. das Datum aus der Spalte «Ergibt» wird in das neue Feld eingetragen. der Meilenstein lässt sich mit «✕» löschen. **Wird er noch verwendet**, fragt ein Dialog nach: Er nennt die Zahl der Eingaben und die Abschnitte und stellt sie beim Löschen auf das **feste Datum** (Monat/Jahr) des Meilensteins um, bei «Alter von» auf das berechnete Datum. Entsteht dadurch in einer Zeitreihe ein doppelter Zeitpunkt, gilt die spätere Zeile und eine Meldung weist darauf hin. Mit «Rückgängig» (⌘Z) lässt sich alles zurücknehmen. Nicht löschbar sind **Simulationsstart, Simulationsende, Bezug Pensionskasse und AHV-Bezug**; die Schaltfläche ist ausgegraut.

**Ältere Planungen:** Fehlt einer der festen Meilensteine, wird er beim Öffnen mit 65 Jahren ergänzt. Liegt ein Alter ausserhalb der Grenzen, wird es auf die nächste Grenze gesetzt. «Pensionierung» heisst neu «Bezug Pensionskasse». Ein Dialog nennt jede Änderung.

> Das **Ende der Simulation** sollten Sie bewusst spät setzen, etwa auf Alter 95 oder 100. Ein zu frühes Ende macht das Ergebnis besser, als es ist: Das Geld muss nur bis dahin reichen.

## @s-pension Vorsorge
Stichworte: Pensionskasse, AHV, 2. Säule, 1. Säule, Säule 3a, Rente

Pensionskasse (2. Säule) und AHV (1. Säule), bei zwei Personen für jede Person eigene Karten. Die Zahlen finden Sie auf dem **Vorsorgeausweis** Ihrer Pensionskasse und im **AHV-Konto** (Kontoauszug oder Rentenvorausberechnung der Ausgleichskasse).

- Pensionskasse: [Altersguthaben, Bezug, Kapitalbezug, Umwandlungssatz](#f-pk)
- AHV: [Rente und Bezugsbeginn](#f-ahv)

Eine Zeile unter den Karten zeigt, was beim Bezug der Pensionskasse aus dem Guthaben wird (Kapital und Rente).

Die **Säule 3a** hat einen eigenen Abschnitt, siehe [Säule 3a](#s-pillar3a).

## @s-pillar3a Säule 3a
Stichworte: 3a, gebundene Vorsorge, Einzahlung, Auszahlung, Auszahlungsplan, Alter, Datum, Guthaben, Maximalbetrag, indirekte Amortisation

Die Säule 3a ist ein eigenes Konto **je Person**, getrennt von Pensionskasse und Hypothek.

- **Guthaben per Simulationsstart** und **Verzinsung / Rendite p.a.**
- **Einzahlung pro Jahr:** 0 bis zum Maximalbetrag (7 258 Franken; mit «Ohne Pensionskasse (grosse Säule 3a)» bei der Person 36 288 Franken, höchstens 20 % des Erwerbseinkommens; änderbar in den [Einstellungen](#f-settings)). Die App zahlt monatlich ein Zwölftel ein, bis vor dem Monat des **AHV-Bezugs**. Die Einzahlung **zählt zu den Ausgaben**: Sie erscheint in «Ausgaben total» der Jahreswerte, erhöht die Entnahme und gehört zum Netto-Bedarf der 3-Topf-Strategie.
- **Auszahlungsplan:** Zeilen aus Zeitpunkt und Betrag, zum Beispiel 2041 50 000 und 2042 50 000. Den **Zeitpunkt** legen Sie wie bei einem Meilenstein als **Datum** (MM.JJJJ) oder als **Alter** der Person fest, der das Konto gehört (zum Beispiel 61.5 Jahre). Daneben steht der Monat, der sich daraus ergibt, und das Alter. Wechseln Sie zwischen «Datum» und «Alter», bleibt der Zeitpunkt gleich. Ohne Betrag wird das ganze Guthaben ausbezahlt. Eine Auszahlung ist **frühestens 5 Jahre vor dem AHV-Bezug** möglich, auch nach Alter; ein früherer Eintrag ist ein Fehler am Feld. Ist ein Betrag grösser als das Guthaben, wird nur das Guthaben ausbezahlt (Hinweis in den Ereignissen).
- **Guthaben aus der Vorschau:** Jede Zeile zeigt das **Guthaben vor der Auszahlung** und den **Rest danach**, unter dem Plan steht das Guthaben, das im AHV-Bezug ausbezahlt wird. Die Werte stammen aus der Vorschau mit fixen Renditen; hat die Planung einen Fehler, gibt es keine Vorschau, und es steht «–».
- **+ Auszahlung** legt eine Zeile ein Jahr nach der letzten an (ohne Zeile: im frühesten erlaubten Monat, 5 Jahre vor dem AHV-Bezug, nicht vor dem Simulationsstart), höchstens im AHV-Bezug. Als Betrag steht das Guthaben zu diesem Zeitpunkt nach den früheren Auszahlungen, abgerundet auf 1 000. Sie können ihn danach frei ändern.
- **Rest im AHV-Bezug:** Was noch im Konto ist, wird **spätestens im Monat des AHV-Bezugs** ausbezahlt.
- **Verwendung:** ins freie Vermögen oder, wenn eine Tranche mit indirekter Amortisation in dieses Konto besteht, «Hypothek zurückzahlen» (zuerst die Tranchen mit indirekter Amortisation).

Jede Auszahlung ist ein **Kapitalbezug** mit Kapitalbezugssteuer. Bezüge im selben Kalenderjahr (Pensionskasse und Säule 3a) werden für den Satz zusammengerechnet, bei unverheirateten Paaren je Person.

Die **indirekte Amortisation** einer Hypothek fliesst bis vor dem AHV-Bezug in dieses Konto (siehe [Hypotheken](#f-re-mortgage)). Einzahlung und indirekte Amortisation zusammen sollten den Maximalbetrag pro Jahr nicht übersteigen; sonst zeigt die Prüfung eine Warnung. Ohne Pensionskasse warnt die Prüfung ausserdem, sobald die Summe 20 % des Erwerbseinkommens pro Jahr übersteigt (mit dem ersten Monat, in dem das Einkommen zu tief ist); abgezogen wird dann nur dieser Anteil.

> Beispiel: Geburt 06.1980, Guthaben 120 000 ohne Zins, Auszahlung mit 61.5 Jahren (ergibt 12.2041) über 50 000: Guthaben vor der Auszahlung 120 000, Rest danach 70 000. «+ Auszahlung» schlägt eine Zeile mit 62.5 Jahren (12.2042) über 70 000 vor.

> Beispiel: Guthaben 120 000, Auszahlungen 2041 und 2042 je 50 000, AHV-Bezug 2045: Der Rest von 20 000 (plus Zins) wird im Monat des AHV-Bezugs ausbezahlt. Wird der AHV-Bezug auf 2046 verschoben, bleibt der Plan gültig, und der Rest kommt 2046.

## @f-settings Einstellungen
Stichworte: Einstellungen, Grenzen, AHV-Alter, Bezug Pensionskasse, Maximalbetrag, 3a, Standardwerte, Sprache, Darstellung, dunkel, hell

Auf dem Mac Menü «VivaPlan → Einstellungen…» (⌘,), unter Windows und Linux Menü «Datei → Einstellungen…» (Ctrl+,), oder die Schaltfläche «Einstellungen» oben. Die Einstellungen gelten für **alle Planungen auf diesem Gerät** und werden lokal gespeichert, **nicht in der Planungsdatei**.

| Einstellung | Standard |
|---|---|
| Sprache | wie System |
| Darstellung | System |
| AHV-Bezug: untere und obere Grenze | 63 und 70 Jahre |
| Bezug Pensionskasse: untere und obere Grenze | 58 und 65 Jahre |
| Säule 3a: Maximalbetrag der Einzahlung pro Jahr | 7 258 |
| Grosse Säule 3a (ohne Pensionskasse): Maximalbetrag pro Jahr | 36 288 |

**Sprache:** «wie System» folgt der Sprache des Betriebssystems (beim ersten Start; ohne Region, fr_FR gilt als Französisch), sonst gilt die gewählte Sprache. Angeboten werden Deutsch, Englisch, Französisch und Italienisch; fehlt die Sprache des Systems, gilt Deutsch. Die Hilfe erscheint in der gewählten Sprache; ein noch nicht übersetztes Kapitel erscheint deutsch. Dieselbe Wahl steht im Menü «Ansicht → Sprache». Ein Wechsel wirkt sofort, auch auf die Menüs; die Planung, ungespeicherte Änderungen und die Ansicht bleiben. Zahlen, Beträge und Daten erscheinen in jeder Sprache im Schweizer Format (1'234'567.50, 2.5 %, 03.2040), und die Planungsdatei ist in jeder Sprache dieselbe. Namen in der Planung (Personen, Immobilien, Meilensteine) werden nicht übersetzt; nur die Beispielplanung erhält beim Öffnen ihre Namen in der gewählten Sprache, mit denselben Zahlen.

**Darstellung:** «System» folgt dem hellen oder dunklen Erscheinungsbild des Betriebssystems, auch wenn es während der Arbeit wechselt; «Hell» und «Dunkel» gelten unabhängig davon. Das Thema gilt für die ganze App: Eingabefelder, Auswahlmenüs, Dialoge, Hilfe und Diagramme. Die Wahl wirkt sofort beim Speichern.

Die untere Grenze darf nicht über der oberen liegen, Alter sind ganze Jahre, der Maximalbetrag ist 0 oder mehr. «Standardwerte» setzt alles nach Rückfrage zurück. Liegt eine Planung nach einer Änderung ausserhalb der neuen Grenzen, meldet die Prüfung das; dieselbe Planung kann deshalb auf einem anderen Gerät mit anderen Einstellungen eine Meldung ergeben. Die Kommandozeile verwendet immer die Standardwerte.

## @f-pk Pensionskasse
Stichworte: BVG, Altersguthaben, Beiträge, Verzinsung, Vorsorgeausweis

- **Altersguthaben per Simulationsstart (MM.JJJJ):** Das Guthaben laut Vorsorgeausweis (Altersguthaben inkl. überobligatorischer Teil). Die Hochrechnungen im Ausweis (Projektion mit Zins) trägt man nicht hier ein, die rechnet die App selbst.
- **Bezug ab:** immer der Meilenstein **Bezug Pensionskasse** der Person (58 bis 65 Jahre). Den Zeitpunkt ändern Sie unter [Meilensteine](#s-milestones).
- **Beiträge pro Monat:** Summe aus Arbeitnehmer- und Arbeitgeberbeiträgen (und Einkäufen), die bis zum Bezug einbezahlt werden. Mit dem Bezug enden sie.
- **Verzinsung pro Jahr:** Zinssatz auf das Altersguthaben. Die Verzinsung wird im Dezember gutgeschrieben. Realistisch sind 1 % bis 2 %.
- [Anteil Kapitalbezug](#f-pk-lumpsum) und [Umwandlungssatz](#f-pk-conversion) bestimmen, wie viel als Kapital und wie viel als Rente fliesst.

Beiträge und Verzinsung sind [zeitabhängig](#f-timed) eingebbar, zum Beispiel höhere Beiträge ab 55.

## @f-pk-conversion Umwandlungssatz
Stichworte: Rente, Pensionskasse, Prozent, Umwandlung

Der Umwandlungssatz legt fest, wie viel **Jahresrente** Sie pro Franken Altersguthaben erhalten, das Sie nicht als Kapital beziehen.

> Jahresrente = Rentenkapital × Umwandlungssatz. Beispiel: 500 000 Franken × 5.0 % = 25 000 Franken pro Jahr (rund 2 083 pro Monat).

- Das **gesetzliche Minimum** liegt im obligatorischen Teil bei 6.8 %, doch viele Pensionskassen rechnen mit **tieferen** Sätzen (häufig 4.5 % bis 5.5 %), weil sie den überobligatorischen Teil anders umwandeln. Verwenden Sie den Wert Ihres Vorsorgeausweises, nicht das Gesetzesminimum.
- Der Satz hängt vom Alter beim Bezug ab. Verwenden Sie den Satz für Ihr Pensionierungsalter.
- In der App gilt: Rente pro Monat = Rentenkapital × Umwandlungssatz / 12, **fix** (keine Teuerung) bis ans Lebensende.

Mit dem Szenarien-Bereich können Sie den Kapitalbezug variieren (siehe [Szenarien](#f-sweep)).

## @f-pk-lumpsum Anteil Kapitalbezug
Stichworte: Kapitalbezug, Rente, Kapitalsteuer, Einmalauszahlung

Der Anteil des Altersguthabens, der bei der Pensionierung **als Kapital** ausbezahlt wird. 0 % heisst «ganz als Rente», 100 % «ganz als Kapital».

- Das Kapital kommt netto auf Ihr liquides Vermögen. Davon wird die **Kapitalbezugssteuer** abgezogen (siehe [Steuertabellen](#f-tax-tables)). Bezüge im selben Kalenderjahr (Pensionskasse, Säule 3a) werden für den Satz zusammengerechnet, bei unverheirateten Paaren je Person.
- Der Rest wird mit dem [Umwandlungssatz](#f-pk-conversion) zur Rente.
- **Rente:** planbar, lebenslang, aber nicht indexiert und nicht vererbbar. **Kapital:** flexibel und vererbbar, trägt aber das Anlagerisiko und die Verantwortung, es nicht zu früh aufzubrauchen.

Der beste Anteil hängt von der Steuer, vom Umwandlungssatz und von Ihrer Risikobereitschaft ab. Lassen Sie ihn mit einer [Variation](#f-sweep) laufen und sehen Sie, wie sich die Erfolgsquote verändert.

## @f-ahv AHV
Stichworte: Rente, Bezug, Teuerung, Index, 1. Säule, Beitragsjahre, 13. AHV-Rente, Dezember, Vorbezug, Aufschub, Kürzung, Zuschlag

- **Rente pro Monat bei Bezugsbeginn:** Ihre voraussichtliche Monatsrente beim Bezug mit 65 (Referenzalter) laut Rentenvorausberechnung der Ausgleichskasse, in heutigen Preisen, **ohne 13. Rente** (die App rechnet sie selbst dazu, siehe unten).
- **Bezug ab:** immer der Meilenstein **AHV-Bezug** der Person (63 bis 70 Jahre). Liegt er vor oder nach 65, kürzt bzw. erhöht die App die Rente selbst (siehe unten); geben Sie trotzdem die Rente mit 65 ein.

Unter dem Feld steht immer die Rente, mit der die App rechnet, in heutigen Preisen: ohne Vorbezug oder Aufschub gleich der Eingabe, sonst mit dem Satz, z. B. «−13.6 % → effektiv 2'177.00 CHF pro Monat». Bei Verheirateten steht zusätzlich, ab wann beide Anspruch haben und welche Rente dann nach der Obergrenze und nach Kürzung bzw. Zuschlag bleibt, z. B. «ab 03.2032 plafoniert 1'547.00 CHF».

**Vorbezug und Aufschub:** Ein Vorbezug kürzt die Rente um 6,8 % je Jahr, monatsgenau nach der Tabelle der AHV (z. B. 15 Monate 8,5 %), höchstens 13,6 % bei Bezug mit 63. Ein Aufschub um mindestens ein Jahr erhöht sie nach Vierteljahren: 1 Jahr 5,2 %, 2 Jahre 10,8 %, 3 Jahre 17,1 %, 4 Jahre 24,0 %, 5 Jahre 31,5 %; ein Aufschub unter einem Jahr bringt keinen Zuschlag. Kürzung und Zuschlag sind feste Beträge in ganzen Franken; sie gelten für die ganze Bezugsdauer, auch nach 65, und je Person nach ihrem eigenen Alter. Beispiel: Rente 2 520, Bezug mit 63 → 2 520 − 343 = 2 177; Bezug mit 68 → 2 520 + 431 = 2 951, zwischen 65 und 68 keine AHV. Bei Verheirateten gilt die Obergrenze wie bei der AHV für die **ungekürzten** Renten, sobald beide Anspruch haben (Bezug, auch im Vorbezug, oder 65 erreicht, auch im Aufschub); erst danach zieht die App die Kürzung ab bzw. zählt den Zuschlag dazu. Die Kürzung beträgt den Satz mal den Durchschnitt der Renten während des Vorbezugs, der Zuschlag den Satz mal den Durchschnitt der aufgeschobenen Rente, je nachdem schon begrenzt. Beispiel mit zwei Renten 2 520 und Obergrenze 3 780: bezieht die erste Person mit 63 und die zweite mit 65 viel später, gibt es zuerst 2 177, danach 1 890 − 343 = 1 547 und 1 890. Die 13. Rente beruht auf den ausbezahlten Renten. Ein Bezug weniger als ein Jahr nach 65 ist kein Aufschub (kein Zuschlag); ein Bezug vor 63 oder nach 70 ist nicht möglich. In beiden Fällen warnt die Prüfung. Quelle: Merkblatt 3.04 «Flexibler Rentenbezug» (Stand 1.1.2026). Nicht berücksichtigt sind Teilvorbezug und Teilaufschub sowie die besonderen Sätze für Frauen der Jahrgänge 1961 bis 1969. Planungen, die vor dieser Änderung gespeichert wurden, enthielten bei Bezug vor oder nach 65 die bereits angepasste Rente; beim Öffnen rechnet die App sie einmalig auf die Rente mit 65 zurück und meldet es. Das Ergebnis bleibt bis auf die Rundung auf Franken gleich; bei Ehepaaren kann es sich durch die Obergrenze ändern.
- **Rente jährlich an die Teuerung anpassen:** Die AHV wird im Januar um die seit dem letzten Januar aufgelaufene Teuerung erhöht . Lassen Sie das Feld angekreuzt, wenn die Rente mitwachsen soll.

Bei zwei Personen gibt es bei Verheirateten eine Obergrenze für beide Renten zusammen, siehe [Zweite Person und Ehepaar-Regeln](#f-couple).

**13. AHV-Rente:** Ab Dezember 2026 zahlt die AHV jedes Jahr im Dezember eine 13. Rente; die App rechnet sie immer mit, ein Schalter dafür gibt es nicht. Sie beträgt ein Zwölftel der von Januar bis Dezember ausbezahlten AHV-Renten (je Monat auf Rappen gerundet, die Summe auf ganze Franken). Beginnt die Rente im Laufe des Jahres, gibt es sie anteilig: Rente 2 520 ab Juli ergibt 6 × 210 = 1 260. Bei Verheirateten zählt die bereits begrenzte Rente; die 13. Rente selbst wird nicht nochmals begrenzt. Sie zählt im Dezember zum AHV-Einkommen: in den [Jahreswerten](#r-table) bei den Einnahmen, in der Grafik [Einnahmen und Ausgaben pro Jahr](#r-flows) bei der AHV. Sie ist wie die AHV-Rente steuerbar. Endet die Simulation vor Dezember, gibt es in diesem Jahr keine 13. Rente. Beim Öffnen einer Planung, die vor dieser Änderung gespeichert wurde, weist ein Hinweis einmalig darauf hin, dass das Ergebnis deshalb abweichen kann.

## @f-couple Zweite Person und Ehepaar-Regeln
Stichworte: Ehepaar, verheiratet, Plafonierung, 150 Prozent, Partner, Double Income

Mit «+ Zweite Person» erfassen Sie einen Haushalt mit zwei Personen.

- Jede Person hat **eigenes Einkommen, eigene Pensionskasse und eigene AHV**.
- Steuern, Vermögen, Ausgaben und Immobilien gelten für den **Haushalt**.
- **Als pensioniert gilt der Haushalt**, sobald alle Personen mit Bezugsdatum pensioniert sind. Erst dann greifen die Entnahmestrategien.

**Verheiratet** (Ehepaar-Regeln): Haben beide Anspruch auf eine AHV-Rente, werden die Renten mit 65 zusammen auf höchstens **3 780 Franken pro Monat** begrenzt; Kürzung bei Vorbezug und Zuschlag bei Aufschub kommen danach dazu (siehe [AHV](#f-ahv)) (150 % der Maximalrente 2025). Der Wert ist änderbar, falls die Maximalrente steigt. Die Grenze steigt jeden Januar einmal mit der Teuerung, sobald mindestens eine Rente mit Teuerungsanpassung läuft. Unter [Steuern](#s-taxes) können für Verheiratete eigene Tabellen hinterlegt werden. Kapitalbezüge im selben Jahr werden bei Verheirateten zusammengerechnet, bei unverheirateten Paaren je Person.

**Nicht gerechnet:** Todesfall einer Person, Hinterlassenenrenten, Witwenrente. Die Planung rechnet, als lebten beide bis zum Ende.

## @s-flows Einnahmen & Ausgaben
Stichworte: Einkommen, Lebenshaltung, Ausgaben, einmalige Beträge, Erbschaft, Inflation

Monatliche Beträge, das Erwerbseinkommen pro Jahr. **Ausgaben werden in heutigen Preisen erfasst** und von der App mit der Inflation fortgeschrieben. Sie tragen also Ihre heutigen Kosten ein, auch für Jahre in der Zukunft.

- **Einkünfte aus Erwerb netto pro Jahr:** wie in der Steuererklärung der Nettolohn im Lohnausweis (Ziffer 11), inklusive 13. Monatslohn und Boni, also was nach den Abzügen für AHV und Pensionskasse bleibt (Steuern werden separat gerechnet); für Selbständige das Einkommen aus selbständiger Erwerbstätigkeit. Neben dem Feld steht der Betrag pro Monat, zum Beispiel 120 000 pro Jahr «= 10 000 / Monat»; gerechnet und gespeichert wird pro Monat (ein Zwölftel), auch in jeder Zeile eines zeitabhängigen Einkommens und für beide Personen. Bestehende Planungen zeigen den bisherigen Monatsbetrag mal 12. Die Parameter eines Laufs und die Warnung unten nennen den Betrag ebenfalls pro Jahr. Es endet, wenn Sie es [zeitabhängig](#f-timed) auf 0 setzen, zum Beispiel «ab Bezug Pensionskasse 0». Der Bezug der Pensionskasse beendet es nicht von selbst; ist es danach noch grösser als 0, zeigt die Prüfung eine Warnung mit dem Betrag.
- **Steigt mit der Teuerung** (je Person): Mit Häkchen gelten die Beträge, auch jede Zeile eines zeitabhängigen Einkommens, in **heutigen Preisen** (Preise beim Simulationsstart) und wachsen mit der Teuerung des jeweiligen Pfades, genau wie die Lebenshaltung. Beispiel: 8 000 pro Monat bei 2 % Teuerung sind nach einem Jahr 8 160, nach fünf Jahren rund 8 830. Ohne Häkchen bleibt der Betrag nominal gleich. Neue Planungen und eine neu hinzugefügte zweite Person haben das Häkchen gesetzt; ältere Planungen ohne Angabe rechnen wie bisher ohne. Steuern, Jahreswerte und Tragbarkeit verwenden den angepassten Betrag; die Warnung nach dem Bezug der Pensionskasse nennt dann den Betrag «in Preisen des Simulationsstarts».
- **Lebenshaltungskosten pro Monat:** Die **Eingabe** umfasst alles, was Sie ausgeben, **inklusive Miete bzw. Wohnkosten** (Hypothekarzins, Amortisation, Nebenkosten), so wie es auf dem Kontoauszug steht. Die App zieht den **Wohnanteil** ab, das sind die Wohnkosten im Startmonat aus dem Abschnitt [Immobilien](#s-realestate) als Monatsdurchschnitt (Zins und Amortisation werden vierteljährlich bezahlt), mit der Teuerung fortgeschrieben, und zwar von jeder Zeile gleich. **Gerechnet** wird mit der Lebenshaltung ohne Wohnanteil plus den **simulierten** Wohnkosten: Ändert sich der Zins bei einer Erneuerung oder ziehen Sie nach einem Verkauf in eine Miete, ändern sich die Ausgaben entsprechend. Beispiel: 8 000 inklusive 2 000 Wohnkosten; nach einer Zinserhöhung um 300 sind die Ausgaben 8 300, mit einer Miete von 2 500 sind es 8 500. Ist eine Zeile kleiner als der Wohnanteil, meldet die Prüfung einen Fehler. Die Einzahlung in die [Säule 3a](#s-pillar3a) ist nicht enthalten. Ältere Planungen (Lebenshaltung ohne Wohnkosten) rechnet die App beim Öffnen um, ein Hinweis nennt den addierten Betrag. Steuern werden separat gerechnet. Ein realistischer Wert ist der wichtigste Einzelfaktor für das Ergebnis. Schauen Sie dazu zwei bis drei Jahre Kontoauszüge an.
- **Einmalige Beträge:** Datum, Betrag und Beschreibung. Positiv ist ein Zufluss (Erbschaft, Schenkung), negativ eine Ausgabe (Auto, Reise, Renovation).

Mit dem Alter sinken die Ausgaben oft, zum Beispiel weniger Reisen ab 80, dafür steigen Gesundheitskosten. Nutzen Sie dafür [zeitabhängige Werte](#f-timed).

## @f-timed Zeitabhängige Werte
Stichworte: Konstant, Zeitabhängig, Zeile, ab, Verlauf, Minigrafik

Viele Beträge und Sätze gibt es in zwei Formen:

- **Konstant:** Ein Wert für die ganze Simulation.
- **Zeitabhängig:** Eine Abfolge von Zeilen «ab *Zeitpunkt* gilt *Wert*». Der Zeitpunkt ist ein Meilenstein («ab Bezug Pensionskasse») oder ein festes Datum. Jede Zeile gilt, bis die nächste beginnt.

Beispiel Lebenshaltung: «ab Start 9 500», «ab Bezug Pensionskasse 8 500», «ab 80 7 000».

Die kleine Grafik neben dem Feld zeigt den Verlauf. Mit dem Mausrad zoomen Sie hinein, die Knöpfe «Alles», «10 J.», «5 J.» und «2 J.» wählen den Ausschnitt.

Zeilen, die in der Zeit hinten stehen, aber vor einer anderen beginnen, ordnet die App nach dem Zeitpunkt. Verschieben Sie daher ruhig Meilensteine, die Reihenfolge der Werte bleibt konsistent.

## @s-wealth Vermögen & Anlage
Stichworte: Aktien, Obligationen, Cash, Liquidität, Aktienquote, Aktienanteil, Reserve, Portfolio, Anlagestrategie, Index

Die **Bestände per Simulationsstart**, die **Aufteilung** des Portfolios und die **Anlagestrategie**.

- **Aktien, Obligationen, Cash:** Ihre Bestände per Simulationsstart. Immobilien und Vorsorgeguthaben stehen separat, hier nur das liquide Vermögen. Die App übernimmt die Bestände beim Start unverändert. Neben Aktien und Obligationen steht ihr Anteil am Portfolio (ohne Cash, ganze Prozent), zum Beispiel ergeben 80 000 Aktien und 20 000 Obligationen «80 % des Portfolios» bzw. «20 % des Portfolios».
- **Cash zählt nicht zum Portfolio.** Die [Aktienquote](#f-stockshare) bezieht sich nur auf Aktien und Obligationen: 80 000 Aktien, 20 000 Obligationen und 50 000 Cash ergeben eine Aktienquote von 80 %.
- **Bis zur Pensionierung** (Bezug Pensionskasse der ersten Person) gilt keine Entnahmestrategie. Die App hält so viel Cash, wie im Feld **Cash** steht, mit der **Teuerung** fortgeschrieben: Ein Überschuss geht nach der Aktienquote ins Portfolio, ein Fehlbetrag wird aus dem Portfolio nachgefüllt.
- **Ab der Pensionierung** bestimmt die [Entnahmestrategie](#s-withdrawal) das Cash: «Standard» mit der [Cash-Reserve](#f-reserve) aus ihrer Karte, alle anderen mit 3 Monaten bzw. die 3-Topf-Strategie mit ihren Töpfen.

- **Anlagestrategie:** je ein Index für die Aktien (Welt, Welt individuell, Schweiz, S&P 500) und für die Obligationen (Eidgenossenschaft 10 Jahre oder kurzfristig, USA). Bei «Welt individuell» legen Sie die Gewichte von Schweiz, USA, EU und Asien-Pazifik fest; sie müssen zusammen 100 % ergeben. Unter der Auswahl stehen der gemeinsame Zeitraum und die mittleren Renditen; wird ein Index auf den gemeinsamen Zeitraum gekürzt, erscheint ein Hinweis. Einzelheiten: [Anlagestrategie](#f-ret-series).

Das Vermögen wird in der Simulation jeden Monat mit der Rendite von Aktien und Obligationen fortgeschrieben: aus den Indizes der Anlagestrategie, nach der Methode unter [Simulation](#s-returns) (fixe Renditen, historischer Verlauf oder Monte Carlo).

## @f-stockshare Aktienanteil
Stichworte: Risiko, Aktien, Rebalancing, Portfolio

Der Anteil der **Aktien am Portfolio** (Aktien + Obligationen, ohne Cash). Der Rest ist Obligationen.

- Die App stellt diesen Anteil jeden Monat wieder her (**Rebalancing**), soweit die Strategie nichts anderes vorsieht.
- Ein hoher Aktienanteil bringt im Mittel mehr Rendite, aber auch tiefere Tiefstände. Gerade am Anfang der Pensionierung ist das Risiko hoch (**Sequenzrisiko**): Ein Börseneinbruch kurz nach der Pensionierung schadet viel mehr als derselbe Einbruch 20 Jahre später.
- Der Wert kann [zeitabhängig](#f-timed) sein, zum Beispiel 70 % bis zur Pensionierung und 50 % danach.

Wie stark der Aktienanteil das Ergebnis verändert, zeigt eine [Variation](#f-sweep) des Aktienanteils, oder die [Optimierung](#s-optimize), die den Anteil gleich mit sucht.

## @f-reserve Cash-Reserve
Stichworte: Cash, Reserve, Monate, Liquidität, Liquiditätsreserve

Wie viele **Monate der ungedeckten Ausgaben** (Ausgaben minus Einkommen und Renten) **ab der Pensionierung** als Cash gehalten werden, bevor der Rest nach der [Aktienquote](#f-stockshare) investiert wird. Vor der Pensionierung gilt das Feld «Cash» unter [Vermögen & Anlage](#s-wealth).

- Eine hohe Reserve schützt davor, bei tiefen Kursen Aktien verkaufen zu müssen, kostet aber Rendite.
- Einstellbar nur bei der Strategie «Standard» (Karte «Standard» unter «Entnahmestrategie») und wirksam erst ab der Pensionierung. **Alle anderen Strategien** (Guardrails, feste Rate, VPW, Endowment) rechnen ab der Pensionierung mit einer **festen Reserve von 3 Monaten**; bei der [3-Topf-Strategie](#f-wd-three_buckets) ersetzen die Töpfe die Reserve.
- Eine ältere Planung mit anderer Strategie und einer Reserve ungleich 3 Monaten rechnet deshalb mit 3 Monaten und kann leicht andere Ergebnisse liefern. Die App weist einmal darauf hin, beim ersten Öffnen einer solchen Planung; danach erscheint der Hinweis nicht mehr. Der gespeicherte Wert bleibt in der Datei und gilt wieder, wenn Sie zu «Standard» wechseln.
- Zeitabhängig sinnvoll: zum Beispiel 6 Monate bis zur Pensionierung und 12 Monate danach.

## @s-taxes Steuern
Stichworte: Einkommenssteuer, Vermögenssteuer, Kapitalsteuer, Steuerfuss, Tabelle

Zwei Modi stehen zur Wahl: **Wohnort** rechnet die Einkommens-, Vermögens- und Kapitalbezugssteuer aus Kanton, Gemeinde und Kirchensteuer mit den Steuerdaten der ESTV und die Grundstückgewinnsteuer nach dem Kanton der Immobilie ([Steuer nach Wohnort](#f-tax-location)); **Eigene Tabellen** rechnet mit Satztabellen und Steuerfuss, wie unten beschrieben. Bestehende Planungen öffnen mit «Eigene Tabellen».

Einfache **Satztabellen**, die Sie aus dem Steuerrechner Ihres Kantons ableiten können. Zwischen den Stufen wird **linear interpoliert**, der Satz wird mit dem [Steuerfuss](#f-tax-multiplier) multipliziert.

- [Steuer nach Wohnort](#f-tax-location)
- [Abzüge vom Einkommen](#f-tax-deductions)
- [Steuerfuss und Tabellen](#f-tax-tables)
- [Tarife mit der Teuerung](#f-tax-indexed)
- [Wie genau ist die Steuer?](#tax-accuracy)
- Die **Einkommenssteuer** wird im Dezember auf das Jahreseinkommen (inkl. Renten) veranlagt, die **Vermögenssteuer** auf das Portfolio plus Immobilien netto (mit dem [Steuerwert](#f-re-tax-value), wo erfasst, sonst mit dem Marktwert). Bezahlt wird beides im Mai des Folgejahres.
- Eine bei Planende noch offene Steuer wird im letzten Monat abgezogen.
- Die [Grundstückgewinnsteuer](#f-tax-gain) fällt beim Verkauf einer Immobilie an, siehe auch [Immobilien](#s-realestate).

Die Beispielwerte sind nur Platzhalter. Ersetzen Sie sie für Ihr Ergebnis durch Werte Ihres Wohnorts.

## @f-tax-location Steuer nach Wohnort
Stichworte: Wohnort, Kanton, Gemeinde, Kirchensteuer, Konfession, ESTV, Steuerdaten, Standardabzüge, Berufskosten, Bundessteuer, steuerbares Einkommen, Kapitalbezug, Kapitalbezugssteuer, Grundstückgewinnsteuer, Kanton der Liegenschaft, Umzug, Wohnortwechsel, Stichtag

Im Modus **Wohnort** wählen Sie nur **Kanton**, **Gemeinde** (Liste des Kantons mit Suche: Namen eintippen) und die **Kirchensteuer** (keine, reformiert, römisch-katholisch, christkatholisch). Tarife, Abzüge und Steuerfüsse kommen aus der mitgelieferten Steuerdatei der ESTV (Steuerrechner, Grunddaten); die Karte nennt das Steuerjahr, die Quelle und die Steuerfüsse der Gemeinde. Es gibt keinen Netzwerkzugriff.

**Steuerbares Einkommen** = Einkommen des Jahres (Erwerb netto, Pensionskassenrente, AHV) minus die **Standardabzüge** (Vorschlag des Kantons), nie negativ. Statt des Vorschlags gilt je Zeitraum auch ein eigener Wert, dazu die Säule 3a, siehe [Abzüge vom Einkommen](#f-tax-deductions). Kanton und Bund haben je eigene Abzüge:

- **Sozialabzüge** für Alleinstehende oder Verheiratete, soweit der Kanton sie kennt.
- **Versicherungsprämien und Sparzinsen:** in Jahren mit Erwerbseinkommen der Betrag «mit Beiträgen an die 2. und 3. Säule», sonst der Betrag «ohne».
- **Berufskostenpauschale** auf dem Erwerb jeder erwerbstätigen Person: Satz, Mindest- und Höchstbetrag des Kantons, höchstens der Erwerb, ohne Erwerb kein Abzug. Beispiel Zürich: 3 %, mindestens 2 000, höchstens 4 000 Franken, bei 100 000 Erwerb also 3 000. Basel-Landschaft, Basel-Stadt und Tessin nennen in den ESTV-Daten nur einen Höchstbetrag; er gilt als fester Abzug (⚠).
- **Zweitverdienerabzug** bei Ehepaaren, wenn beide Erwerbseinkommen haben: nach der Regel von Kanton und Bund, bezogen auf das tiefere Erwerbseinkommen nach der Berufskostenpauschale und höchstens dieses; Bern rechnet 2 % des gesamten Erwerbseinkommens. Beispiel Bund: 50 %, mindestens 8 600, höchstens 14 100 Franken; Zürich fest 6 200 Franken. Der Thurgau kennt keinen Zweitverdienerabzug.

**Steuer:**

- Einkommenssteuer = Tarif des Kantons auf das steuerbare Einkommen × (Steuerfuss Kanton + Kirche) + Tarif der Gemeinde × Steuerfuss Gemeinde + direkte Bundessteuer auf das steuerbare Einkommen des Bundes. Der Tarif der Gemeinde ist der des Kantons, ausser in Schwyz und Wallis.
- Vermögenssteuer = Tarif des Kantons (Sozialabzüge auf dem Vermögen eingerechnet) × (Steuerfuss Kanton + Gemeinde + Kirche), ohne Bundessteuer.
- **Kapitalbezugssteuer** (Pensionskasse, Säule 3a) = Kapitalregel des Kantons × (Steuerfuss Kanton + Kirche) + Regel der Gemeinde × Steuerfuss Gemeinde + Bundessteuer (ein Fünftel des Tarifs der direkten Bundessteuer, ohne Abzüge). Es gelten die Steuerfüsse der Einkommenssteuer; die Regel gilt für den ganzen Bezug. Bezüge im selben Kalenderjahr werden zusammengerechnet: bei Verheirateten für den Haushalt mit dem Tarif für Verheiratete, bei unverheirateten Paaren je Person mit dem Tarif für Alleinstehende. Unter der Gemeinde steht die Regel des Kantons, zum Beispiel Zürich «Satz des Einkommenstarifs für einen Zwanzigstel des Bezugs, mindestens 2 %». Die Regeln sind aus dem ESTV-Steuerrechner abgeleitet; in Wallis und Zug sind sie nur angenähert (⚠).
- **Verheiratete** (zwei Personen und «Verheiratet» angekreuzt): Tarife und Abzüge für Verheiratete. Unverheiratete Paare rechnen mit denen für Alleinstehende auf dem gemeinsamen Einkommen.
- Mit [Tarifen mit der Teuerung](#f-tax-indexed) wachsen die Beträge der Tarife und Abzüge mit der simulierten Teuerung.
- **Grundstückgewinnsteuer:** Für jede Immobilie gilt der Regelsatz des Kantons, in dem sie liegt; den Kanton wählen Sie bei der Immobilie unter [Immobilien](#s-realestate), ohne Angabe gilt der Wohnkanton. Einzelheiten unter [Grundstückgewinnsteuer](#f-tax-gain). Mit dem Häkchen **«Eigene Regeln statt der Regeln des Kantons»** gelten stattdessen die eigenen Regeln mit dem [Steuerfuss](#f-tax-multiplier) für alle Immobilien.

**Wohnortwechsel:** Mit «Wohnort hinzufügen» legen Sie einen weiteren Wohnort an, ab einem Jahr oder ab einem Ereignis (zum Beispiel der Pensionierung), je mit Kanton, Gemeinde und Kirchensteuer; die erste Zeile gilt ab dem Simulationsstart. Es gelten die Regeln des Steuerharmonisierungsgesetzes (StHG Art. 4b): Die **Einkommens- und Vermögenssteuer** eines Jahres rechnet mit dem Wohnort am **31. Dezember**, für das ganze Jahr – ziehen Sie im Juni um, zahlen Sie das ganze Jahr am neuen Ort. Die **Kapitalbezugssteuer** rechnet mit dem Wohnort im **Monat des Bezugs**; ein Umzug vor dem Kapitalbezug in einen günstigeren Kanton wirkt also nur, wenn er vor dem Bezug liegt. Die **Grundstückgewinnsteuer** einer Immobilie ohne eigenen Kanton rechnet mit dem Wohnkanton im Monat des Verkaufs. Der [Vorschlag des Kantons](#f-tax-deductions) richtet sich nach dem Wohnort des Steuerjahres, ein eigener Wert bleibt. Nicht berechnet werden der Wegzug ins Ausland, Umzugskosten und eine Steuerpflicht nur für einen Teil des Jahres. Die Parameter eines Laufs nennen jeden Wohnort mit seinem Beginn, die Jahreswerte zeigen mit den Steuerspalten den Wohnort jedes Jahres.

**Ausgeblendet:** Die eigenen Tabellen für Einkommen, Vermögen und Kapitalbezug (auch die für Verheiratete), die eigenen Regeln der Grundstückgewinnsteuer und der Steuerfuss (ohne das Häkchen) bleiben in der Datei; zurück auf «Eigene Tabellen» gelten sie wieder.

**Prüfung:** Fehlt in einer Zeile der Kanton oder die Gemeinde, liegt die Gemeinde nicht im Kanton oder beginnt eine Zeile vor der vorherigen, ist das ein Fehler im Abschnitt Steuern, und «Simulation starten» ist gesperrt. Die Übersicht der Annahmen und der Reiter «Parameter» nennen den Wohnort, zum Beispiel «Zürich (ZH), Kirchensteuer keine».

## @f-tax-deductions Abzüge vom Einkommen
Stichworte: Abzüge, Vorschlag des Kantons, eigener Wert, Steuerveranlagung, steuerbares Einkommen, Säule 3a, Eigenmietwert, Hypothekarzinsen, Schuldzinsen, Berufskosten

Die Karte **Abzüge vom Einkommen** im Abschnitt Steuern legt fest, was vom Einkommen des Jahres (Erwerb netto, Pensionskassenrente, AHV) abgezogen wird. Das **steuerbare Einkommen** = Einkommen − Abzüge, nie negativ.

**Zeilen:** Die Abzüge sind zeitabhängig wie das Erwerbseinkommen; eine Zeile gilt ab ihrem Zeitpunkt bis zur nächsten. Neue Planungen haben zwei Zeilen, «ab Simulationsstart» (bis Pensionierung) und «ab Bezug Pensionskasse» (ab Pensionierung); weitere Zeilen sind möglich, zum Beispiel «ab 2029» wegen des Wegfalls des Eigenmietwerts. Wechselt die Zeile im Lauf eines Jahres, zählt jede Zeile für ihre Monate.

Jede Zeile ist entweder:

- **Vorschlag des Kantons** (nur im Modus [Wohnort](#f-tax-location), dort Standard): die Standardabzüge aus den Steuerdaten der ESTV, je für Kanton und Bund – Sozialabzüge, Versicherungsprämien und Sparzinsen, Berufskosten. Die Karte zeigt den Betrag für das erste Jahr der Zeile mit seinen Bestandteilen; angenäherte Berufskosten (Basel-Landschaft, Basel-Stadt, Tessin) sind mit ⚠ markiert. Der Vorschlag rechnet sich neu, wenn Sie Kanton, Gemeinde oder Zivilstand ändern.
- **eigener Wert:** ein Betrag pro Jahr in Preisen des Simulationsstarts (wächst mit [Tarifen mit der Teuerung](#f-tax-indexed)), optional zusätzlich ein Prozentsatz des Einkommens. Er gilt für Kanton und Bund und bleibt, wenn Sie Kanton, Gemeinde oder Zivilstand ändern. Im Modus «Eigene Tabellen» gibt es nur den eigenen Wert; ohne Zeile gilt 0. Beispiel: eigener Wert 40 000, Einkommen 120 000, Steuer wie bei 80 000.

**Eigenen Wert bestimmen:** Die Eingabehilfe rechnet ihn aus dem **Einkommen** und dem **steuerbaren Einkommen laut Veranlagung**, zum Beispiel 120 000 und 80 000 ergeben 40 000; «Als eigenen Wert übernehmen» setzt ihn in die Zeile. Dabei gilt:

- Einkommen ist **netto**, nach AHV- und PK-Beiträgen, wie der Nettolohn im Lohnausweis, mit Renten.
- **Ohne Säule 3a:** Die Einzahlungen kommen aus der Planung (siehe unten).
- **Wohneigentum** (Eigenmietwert, Hypothekarzinsen, Unterhalt) gehört in den eigenen Wert. Ab 2029 gibt es für selbstgenutztes Eigentum keinen Eigenmietwert und keinen Schuldzinsabzug mehr; legen Sie dafür eine Zeile «ab 2029» an. **Hypothekarzinsen werden nicht automatisch abgezogen.**

**Säule 3a:** Mit dem Häkchen **«Einzahlungen in die Säule 3a abziehen»** werden die Einzahlungen eines Jahres aus dem Abschnitt [Säule 3a](#s-pillar3a) (mit der indirekten Amortisation) vom steuerbaren Einkommen abgezogen, für Kanton und Bund, höchstens bis zum Maximalbetrag je Person aus den Einstellungen. Beispiel: Einkommen 100 000, sonst keine Abzüge, Einzahlung 7 000, Steuer wie bei 93 000. Für eine Person «Ohne Pensionskasse (grosse Säule 3a)» gelten 20 % ihres Erwerbseinkommens im Jahr, höchstens der Maximalbetrag der grossen Säule 3a; er wächst wie die anderen Beträge mit «Tarife mit der Teuerung». Beispiel: Erwerb 100 000, Einzahlung 25 000, Abzug 20 000; Erwerb 300 000, Einzahlung 40 000, Abzug 36 288. Neue Planungen haben das Häkchen gesetzt.

**Bestehende Planungen** rechnen wie bisher: im Modus «Wohnort» mit dem Vorschlag des Kantons über die ganze Zeit, mit eigenen Tabellen ohne Abzüge, und ohne Abzug der Säule 3a. Erst «Abzüge anpassen» legt Zeilen an.

**Prüfung:** Ein negativer Betrag oder ein Prozentsatz unter 0 % oder über 100 % ist ein Fehler im Abschnitt Steuern.

**Ergebnisse:** Mit «Steuern zeigen» zeigen die Jahreswerte das Einkommen, die Abzüge (Kanton; die Herkunft – Vorschlag des Kantons, eigener Wert, Säule 3a – im Tooltip der Zelle) und das steuerbare Einkommen (der Bund im Tooltip, wenn er abweicht).

## @f-tax-multiplier Steuerfuss
Stichworte: Kanton, Gemeinde, Multiplikator

Ein Faktor auf alle Sätze der Tabellen, der Steuerfuss von **Kanton und Gemeinde** zusammen (zum Beispiel 2.0 für einen doppelten Satz). So müssen Sie die Tabellen nicht neu schreiben, wenn Sie den Wohnort wechseln, sondern ändern nur den Faktor. Erlaubt sind Werte von 0 bis 10. Im Modus [Wohnort](#f-tax-location) gilt er nur für eigene Regeln der Grundstückgewinnsteuer (Häkchen «Eigene Regeln statt der Regeln des Kantons») und ist sonst ausgeblendet.

## @f-tax-indexed Tarife mit der Teuerung
Stichworte: kalte Progression, Teuerung, Inflation, Stufen, Tarif, heutige Preise

Mit Häkchen gelten die **Beträge** der Tabellen für Einkommen, Vermögen und Kapitalbezug in **heutigen Preisen** (Preise beim Simulationsstart) und wachsen mit der simulierten Teuerung des jeweiligen Pfades; die **Sätze** bleiben gleich. So steigt die Steuerlast nicht allein wegen der Teuerung (Ausgleich der kalten Progression), wie es Bund und Kantone handhaben.

- **Beispiel:** Tabelle 0 → 0 %, 200 000 → 10 %, Teuerung 2 % pro Jahr. Im Jahr 10 liegt die obere Stufe bei 243 799. Ein Einkommen von 121 899 (100 000 mit der Teuerung) hat den Satz 5 % wie 100 000 im Startjahr, ohne Häkchen wären es 6.09 %.
- Massgebend ist der Preisindex im Monat der Veranlagung: im Dezember für Einkommen und Vermögen, im Monat des Bezugs für Kapitalbezüge. Auch die geschätzte Einkommenssteuer im Bedarf der [3-Topf-Strategie](#f-wd-three_buckets) rechnet so.
- Das gilt auch für die Tabellen der Verheirateten.
- Die [Grundstückgewinnsteuer](#f-tax-gain) bleibt unverändert, ihr Tarif bezieht sich auf den einzelnen Gewinn.
- Neue Planungen haben das Häkchen gesetzt. Ältere Planungen ohne Angabe und importierte alte Konfigurationen rechnen wie bisher ohne: Die Beträge gelten dann nominal über die ganze Planung.

## @f-tax-gain Grundstückgewinnsteuer
Stichworte: Grundstückgewinn, Verkauf, Immobilie, Besitzdauer, Zürich, Aargau, Kanton, Freigrenze, Freibetrag, Tarif, Steuerfuss, Wohnort, Näherung

Die Steuer auf dem Gewinn beim **Verkauf** einer Immobilie. Die Regeln sind **von Kanton zu Kanton verschieden**. Im Modus [Wohnort](#f-tax-location) gelten die [Regeln des Kantons](#f-tax-gain-canton) der Immobilie; im Modus «Eigene Tabellen» (oder mit dem Häkchen «Eigene Regeln statt der Regeln des Kantons») die eigenen Regeln, die vollständig editierbar sind. **Pflicht** (nur für eigene Regeln): Enthält die Planung eine Immobilie mit Status «Besitz» oder «Kauf geplant», müssen die Regeln gesetzt sein (Vorlage Zürich, Vorlage Aargau oder eigene Regeln, auch mit Satz 0 %). Fehlen sie, ist das ein Fehler im Abschnitt Steuern, und «Simulation starten» sowie «Optimieren» sind gesperrt. Reine Mietobjekte und Planungen ohne Immobilie brauchen keine Regeln.

Zwei Modelle stehen zur Wahl: **Zürich** (Stufentarif mit Zu- und Abschlägen) und **Aargau** (direkter Steuersatz nach Besitzdauer). Die **eigene Eingabe** basiert auf dem Modell Aargau.

**Berechnung:** Gewinn = Verkaufswert − bezahlter Kaufpreis. Wertvermehrende Investitionen, Handänderungs- und Verkaufskosten werden nicht abgezogen. Dann:

1. **Freigrenze:** Liegt der Gewinn darunter, ist er steuerfrei. Ab der Grenze wird der **ganze** Gewinn besteuert.
2. **Modell Zürich, Stufen:** Der Gewinn wird stufenweise besteuert. Jede Zeile gilt für ihren Gewinnteil («bis» ist die Breite der Stufe), über der letzten Stufe gilt deren Satz.
3. **Modell Zürich, Besitzdauer:** Die Steuer wird um den Zuschlag oder Abzug der passenden Zeile verändert. Es gilt die letzte Zeile, deren Jahre erreicht sind (Besitzdauer in vollen Jahren, aus dem [Kaufdatum](#f-re-bought)).
4. **Feste Ermässigung:** Ein zusätzlicher Abzug in Prozent, unabhängig von der Besitzdauer. Bei den Vorlagen steht er auf 0.
5. **Steuerfuss:** Mit der Option «Mit dem Steuerfuss multiplizieren» wird die Steuer zusätzlich mit dem Steuerfuss von Kanton und Gemeinde multipliziert (Standard: aus).

**Vorlage Kanton Zürich** (Tarif § 225 StG, Stand 09/2025): Stufen 10 % (erste 4 000), 15 % (6 000), 20 % (8 000), 25 % (12 000), 30 % (20 000), 35 % (50 000), 40 % darüber. Zuschlag 50 % bei Besitzdauer unter 1 Jahr, 25 % unter 2 Jahren. Abzug ab 5 vollen Jahren 5 %, dann 3 Prozentpunkte mehr pro Jahr bis 50 % ab 20 Jahren. Gewinne unter 5 000 sind steuerfrei. Der Steuerfuss wirkt nur, wenn die Option oben gesetzt ist.

**Vorlage Kanton Aargau** (§ 109 StG): Im **Modell Aargau** gilt statt der Stufen ein **direkter Steuersatz auf den ganzen Gewinn**, abhängig von der Besitzdauer in vollen Jahren: 40 % im ersten Besitzjahr, dann je Jahr 2 Prozentpunkte weniger bis 20 % im elften Besitzjahr, danach 1 Prozentpunkt weniger je Jahr bis 6 % im 25. Besitzjahr und 5 % ab 25 vollen Jahren. Ohne Kaufdatum gilt der höchste Satz, und die App warnt.

Für einen anderen Kanton passen Sie die Tabellen an. Die Werte finden Sie im Tarifblatt der kantonalen Steuerverwaltung. Prüfen Sie sie vor Gebrauch, die Vorlage ist ein Stand und keine Steuerberatung.

## @f-tax-gain-canton Grundstückgewinnsteuer nach Kanton
Stichworte: Grundstückgewinnsteuer, Kanton, Liegenschaft, Wohnort, Regelsatz, Steuermäppchen, Näherung, Besitzdauer, Monate, Steuerfuss, Median

Im Modus [Wohnort](#f-tax-location) gilt für jede Immobilie automatisch der **Regelsatz des Kantons, in dem sie liegt**. Alle 26 Kantone sind erfasst; Quelle ist das Steuermäppchen «Grundstückgewinnsteuer» der ESTV (Steuerperiode 2025), für Zug, Freiburg, Wallis, Basel-Stadt, Basel-Landschaft, St. Gallen und Waadt zusätzlich das kantonale Gesetz.

- **Kanton der Liegenschaft:** Feld bei jeder Immobilie im Besitz oder mit geplantem Kauf unter [Immobilien](#s-realestate). Ohne Angabe gilt der Wohnkanton. Darunter steht die Regel des Kantons, zum Beispiel Zürich «Stufentarif 10 % bis 40 %, Zuschlag bis 50 % unter 2 Jahren, Ermässigung bis 50 % ab 20 Jahren».
- **Besitzdauer in Monaten:** Die Regeln der Kantone zählen die Besitzdauer in Monaten vom [Kaufdatum](#f-re-bought) bis zum Verkauf; mehrere Kantone rechnen Zuschläge je Monat.
- **Steuerfuss:** Bern, Luzern, Obwalden, Solothurn, Schaffhausen und Jura rechnen eine einfache Steuer mal Steuerfuss. Liegt die Immobilie im Wohnkanton, gilt der Steuerfuss der Wohngemeinde (Kanton und Gemeinde), sonst der mittlere Steuerfuss (Median) der Gemeinden des Kantons.
- **Ablauf:** Freigrenze, Freibetrag, Veränderung des Gewinns nach Besitzdauer, Steuer nach dem Modell des Kantons (Stufentarif, Satz nach Besitzdauer, Satz nach Gewinn oder Satz nach Jahresrendite), Zuschlag über einer Schwelle bei kurzer Besitzdauer, Zu- oder Abschlag nach Besitzdauer, Steuerfuss, Ermässigung, Höchstbelastung.
- **Ohne Kaufdatum** gilt kein Zu- oder Abschlag nach Besitzdauer, bei Kantonen mit Satz nach Besitzdauer der höchste Satz; die Prüfung warnt.
- **Näherungen (⚠):** Bei Basel-Stadt, Basel-Landschaft, St. Gallen, Waadt und Wallis ist die Regel nur angenähert, zum Beispiel ist in Basel-Stadt, St. Gallen und Waadt Selbstnutzung angenommen. Die Oberfläche zeigt bei der Immobilie den Hinweis mit ⚠.
- **Nicht enthalten:** Selbstnutzung je Immobilie, die Ermässigung bei Selbstnutzung in Basel-Landschaft, die Ermässigung für wertvermehrende Aufwendungen und die Anpassung der Anlagekosten an die Teuerung.
- **Prüfung:** Eigene Regeln sind keine Pflicht. Ein unbekannter Kanton einer Immobilie ist ein Fehler am Feld.

Mit dem Häkchen **«Eigene Regeln statt der Regeln des Kantons»** im Abschnitt Steuern gelten die eigenen Regeln und der Steuerfuss für alle Immobilien, wie im Modus «Eigene Tabellen».

## @f-tax-tables Steuertabellen
Stichworte: Satz, Interpolation, Einkommen, Vermögen, Kapitalbezug, Verheiratete

Jede Tabelle hat Stufen aus **Betrag** und **Satz**. Dazwischen interpoliert die App linear.

| Tabelle | Betrag | Satz |
|---|---|---|
| Einkommenssteuer | steuerbares Jahreseinkommen | Anteil des Einkommens, der als Steuer anfällt |
| Vermögenssteuer | steuerbares Vermögen | Anteil des Vermögens pro Jahr |
| Kapitalbezugssteuer | Kapitalbezug im Jahr | Anteil des Bezugs |

- **Sätze sind Dezimalzahlen:** 0.05 steht für 5 %, in der Oberfläche als Prozent eingetippt.
- Der Steuerbetrag ist Satz × Betrag × Steuerfuss, auf Rappen gerundet.
- **Grenzen:** Jeder Satz liegt zwischen 0 % und 100 %, jeder Betrag ist 0 oder grösser. Werte ausserhalb meldet die Prüfung am Feld.
- **Steuer darf nicht sinken:** Ein höherer Betrag ergibt nie eine tiefere Steuer. Sinkt der Satz zwischen zwei Stufen so stark, dass Satz × Betrag kleiner wird, meldet die Prüfung eine **Warnung** an der betroffenen Stufe; gerechnet wird trotzdem. Meist ist dann ein Grenzsteuersatz statt des Durchschnittssatzes erfasst. Bei mehreren Kapitalbezügen im selben Jahr wird die Steuer auf einen weiteren Bezug in diesem Fall nie negativ, sondern 0.
- **Kapitalbezüge** im selben Kalenderjahr (Pensionskasse und Säule 3a) werden für den Satz zusammengerechnet: bei Verheirateten für den Haushalt, bei unverheirateten Paaren je Person (jede Person zahlt die Steuer auf ihre eigenen Bezüge). Verteilen Sie Bezüge auf verschiedene Jahre, sinkt der Satz.
- **Verheiratete:** Bei zwei Personen und angekreuztem «Verheiratet» können Sie eigene Tabellen für Ehepaare hinterlegen. Ohne eigene Tabellen gelten die normalen.
- Gehen Sie von der **effektiven** Steuerbelastung aus, nicht vom Grenzsteuersatz. Der Steuerrechner Ihres Kantons liefert beides.

## @tax-accuracy Wie genau ist die Steuer?
Stichworte: Schätzung, geschätzte Steuer, Genauigkeit, Rundung, gerundet, Referenzfälle, Näherung, Abzüge, Steuerberatung, Bandbreite

Die Steuer in VivaPlan ist eine **Schätzung für die Planung**, kein Steuerbescheid. Die Ergebnisse nennen sie deshalb «geschätzte Steuer» und zeigen Steuerbeträge **auf 100 Franken gerundet**. Gerechnet wird genau, die Rundung gilt nur für die Anzeige; die Kommandozeile und ihr CSV-Export geben die genauen Werte aus.

**Referenzfälle:** Die Rechenregeln sind mit von Hand nachgerechneten Fällen getestet: Satz aus der Tabelle linear interpoliert mal Steuerfuss, Kapitalbezüge eines Jahres für den Satz zusammengerechnet, Stufen und Besitzdauer der Grundstückgewinnsteuer. Die Vorlage Kanton Zürich der Grundstückgewinnsteuer stimmt mit der amtlichen Tabelle überein. VivaPlan rechnet also mit Ihren Tabellen richtig; wie nahe das an Ihrer tatsächlichen Steuer liegt, hängt von den Tabellen ab.

**Modus «Wohnort»:** 25 erfundene Fälle in Zürich, Bern, Luzern, Aarau und St. Gallen (Einkommen 80 000 bis 300 000, Vermögen 300 000 bis 4 Millionen, Rentnerinnen und Rentner, ohne Kinder und Konfession) weichen weniger als 1 % vom ESTV-Steuerrechner 2026 ab; verlangt sind höchstens ±5 %. Für Erwerbstätige (alleinstehend, verheiratet mit einem oder zwei Erwerbseinkommen; Bruttolohn 70 000 bis 210 000, Vermögen 50 000 bis 800 000) liegen 25 erfundene Fälle in denselben Gemeinden ebenfalls weniger als 1 % neben dem ESTV-Steuerrechner. Individuelle Abzüge, Kinder, einkommensabhängige Abzüge, Fahrkosten und Verpflegung sind nicht enthalten.

**Kapitalbezug im Modus «Wohnort»:** 48 erfundene Fälle in Zürich, Bern, Luzern, Aarau, St. Gallen, Genf, Freiburg und Bellinzona (Bezüge 100 000, 500 000 und 1 Million, alleinstehend und verheiratet) weichen höchstens ±5 % vom ESTV-Steuerrechner 2026 ab. Wallis und Zug sind nur angenähert (in Zug bis rund 7 %).

**Grundstückgewinnsteuer im Modus «Wohnort»:** Je Kanton prüft ein Test erfundene Fälle (Gewinn und Besitzdauer) gegen von Hand nachgerechnete Werte nach Steuermäppchen oder Gesetz. In Basel-Stadt, Basel-Landschaft, St. Gallen, Waadt und Wallis ist die Regel nur angenähert (⚠).

**Näherungen:**

- Ihre Tabellen geben einen Durchschnittssatz je Betrag, dazwischen interpoliert VivaPlan linear. Der echte Tarif verläuft anders.
- Ein einziger Steuerfuss für Kanton und Gemeinde gilt für alle Tabellen.
- Mit [Tarifen mit der Teuerung](#f-tax-indexed) wachsen die Stufen mit der simulierten Teuerung; der Ausgleich der Kantone kann davon abweichen.
- Grundstückgewinn = Verkaufswert − bezahlter Kaufpreis.

**Was fehlt:**

- Abzüge im Einzelnen. Sie sind nur enthalten, soweit Sie sie in die Sätze einrechnen.
- Eigenmietwert, Abzüge für Schuldzinsen und Unterhalt.
- Kirchen- und Personalsteuer sowie die direkte Bundessteuer, soweit sie nicht in Tabellen oder Steuerfuss stecken.
- Wertvermehrende Investitionen, Handänderungs- und Verkaufskosten bei der Grundstückgewinnsteuer.
- Künftige Gesetzesänderungen.

Eine **Bandbreite** (± %) zeigt VivaPlan nicht: Dafür fehlt eine Grundlage, und eine weitere Zahl würde eine Genauigkeit vortäuschen.

Bei **Kapitalbezugs- und Grundstückgewinnsteuer** geht es oft um viel Geld: Vor dem Entscheid mit dem Steuerrechner des Kantons oder einer Fachperson prüfen. VivaPlan ist keine Steuerberatung.

## @s-realestate Immobilien
Stichworte: Haus, Wohnung, Hypothek, Miete, Kauf, Verkauf, Tragbarkeit

Eigene, geplante und gemietete Objekte. Sie können beliebig viele erfassen.

- **Eigentum («owned»):** Der Marktwert per Simulationsstart steht in der Kopfzeile neben dem Namen. Darunter folgen in festen Plätzen [Kaufdatum](#f-re-bought), Kaufpreis (bezahlt, für die Grundstückgewinnsteuer), «Verkauf ab» (nie, Meilenstein oder Datum) und Nebenkosten pro Monat. Im Steuermodus [Wohnort](#f-tax-location) folgt der **Kanton der Liegenschaft** mit der Regel der [Grundstückgewinnsteuer](#f-tax-gain-canton) dieses Kantons. Bei «Kauf geplant» stehen dieselben Felder an denselben Plätzen (statt Kaufdatum «Kauf ab»), nur der Marktwert fehlt; beim Statuswechsel bleiben die erfassten Werte erhalten. Der Verkauf muss **nach dem Kaufdatum** liegen, sonst ist das ein Fehler am Feld.
- **Inflation:** Wert und Nebenkosten lassen sich je Objekt **einzeln** mit der Inflation fortschreiben, bei Miete ebenso die Miete.
- **Geplanter Kauf («planned»):** Kauf ab Meilenstein, Kaufpreis, Hypothek. Dazu kommt **«Verkauf ab»** (Vorgabe «Nie», sonst Datum oder Meilenstein); die Simulation verkauft das Objekt dann zu diesem Zeitpunkt. Der Verkauf muss nach «Kauf ab» liegen, sonst ist das ein Fehler am Feld. Bei Kauf «nach Verkauf von …» ist der Kaufzeitpunkt beim Erfassen nicht bekannt, dann entfällt diese Prüfung.
- **Miete («rent»):** Beginn und Miete pro Monat (heutiger Wert, wird standardmässig mit der Teuerung fortgeschrieben; die Option «Miete mit Inflation anpassen» schaltet das aus).
- **Hypothek:** Tranchen mit Perioden, siehe [Hypotheken](#f-re-mortgage).
- **Steuerwert:** optional je Immobilie im Besitz oder mit geplantem Kauf, für die Vermögenssteuer, siehe [Steuerwert](#f-re-tax-value).
- [Notverkauf](#f-re-emergency) und [Tragbarkeit](#f-re-afford) für Käufe «nach Verkauf».
- Der Schalter [Immobilien nie verkaufen](#f-re-never-sell) oben im Reiter schliesst jeden Verkauf aus.

Die [Lebenshaltungskosten](#s-flows) geben Sie **inklusive** Wohnkosten ein. Gerechnet wird mit der Lebenshaltung **ohne Wohnanteil**; Miete, Hypothekarzinsen, Amortisation und Nebenkosten kommen als simulierte **Wohnkosten** dazu. Nach dem Verkauf der letzten Immobilie wird im Folgemonat die nächste mit Kauf «nach Verkauf» gekauft, sofern sie tragbar ist und das Eigenkapital reicht. Sonst beginnen die Mieten «ab nach Verkauf». Ein Kauf «nach Verkauf» wird **einmal** im Folgemonat versucht. Ist er nicht finanzierbar, **entfällt** er mit einer Meldung in den Ereignissen, und die Folgeregeln greifen im nächsten Monat: Mieten «ab nach Verkauf» dieser Immobilie beginnen, weitere Käufe «nach Verkauf» davon werden versucht. Ein **Kauf- oder Verkaufsdatum vor dem Simulationsstart** wird nie erreicht; die Prüfung meldet es als Hinweis.

## @f-re-bought Kaufdatum
Stichworte: Besitzdauer, Grundstückgewinnsteuer, Kauf

Der Monat, in dem Sie die Immobilie gekauft haben. Er bestimmt die **Besitzdauer** und damit den Zu- oder Abschlag bei der [Grundstückgewinnsteuer](#f-tax-gain): Je länger Sie die Immobilie besitzen, desto weniger Steuer fällt in vielen Kantonen an.

- Das Feld gibt es nur bei Immobilien im Besitz. Bei einer **geplanten** Immobilie zählt der Kaufmonat der Simulation.
- Ohne Kaufdatum rechnet die App **ohne** Zu- oder Abschlag (die Steuer nach dem Grundtarif). Sind Besitzdauer-Regeln erfasst, weist die Prüfung darauf hin.
- Massgebend ist die Besitzdauer bis zum Verkaufsmonat: bei eigenen Regeln in **vollen Jahren**, mit den [Regeln des Kantons](#f-tax-gain-canton) (Modus Wohnort) in **Monaten**.

## @f-re-tax-value Steuerwert
Stichworte: Vermögenssteuerwert, Vermögenssteuer, Steuerwert, amtlicher Wert, Schätzung

Der **Steuerwert** (Vermögenssteuerwert) ist der amtliche Wert, mit dem die Immobilie in der Steuererklärung steht. Er liegt oft bei **60–80 % des Marktwerts**, bei älteren Schätzungen auch deutlich tiefer. Nehmen Sie den Wert aus Ihrer **Steuererklärung** oder der letzten Veranlagung.

- Das Feld gibt es bei Immobilien im Besitz und mit geplantem Kauf, nicht bei Mieten. Zur Wahl stehen «wie Marktwert» (keine Angabe), ein **Betrag** oder **% des Marktwerts**.
- Nur die **Vermögenssteuer** nutzt ihn: Steuerbar ist je Immobilie der Steuerwert minus die Hypothek statt Marktwert minus Hypothek. Beispiel: Marktwert 1'000'000, Steuerwert 650'000, Hypothek 400'000 ergibt 250'000 steuerbares Vermögen statt 600'000.
- Vermögen, Ergebnisse, Grundstückgewinnsteuer, Tragbarkeit und Notverkauf rechnen weiter mit dem **Marktwert**.
- Ein Betrag behält sein **Verhältnis zum Marktwert** (bei geplantem Kauf zum Kaufpreis): Steigt der Marktwert mit der Teuerung, steigt der Steuerwert im selben Verhältnis. In Wirklichkeit ändert der Steuerwert nur bei einer Neubewertung; über die Jahre ist das eine Näherung.
- Ohne Steuerwert gilt der Marktwert wie bisher; eine Vorgabe gibt es nicht. Ein negativer Betrag oder Prozent ausserhalb von 0–100 % ist ein Fehler am Feld.
- In den [Jahreswerten](#r-table) zeigen die Steuerspalten die Spalte **Steuerwert Immobilien** (Summe am 31. Dezember, Immobilien ohne Steuerwert mit dem Marktwert).
- Nicht berechnet werden Eigenmietwert, die Steuerausscheidung bei einer Immobilie in einem anderen Kanton und Unternutzungsabzüge.

## @f-re-mortgage Hypotheken
Stichworte: Tranche, SARON, Festhypothek, Amortisation, Marge, Zins

Eine Hypothek besteht aus **Tranchen**. Jede Tranche hat eine Folge von **Perioden**:

- **fest:** fester Zinssatz für die Laufzeit der Periode
- **SARON:** SARON plus Marge (der SARON hat eine Untergrenze von 0). Den SARON-Verlauf liefert die Zeitreihe oder der fixe Wert, siehe [SARON](#ret-saron).

Die erste Periode beginnt am Startdatum der Tranche (beim Kauf), jede weitere am Ende der vorherigen. Danach wird die Tranche zu gleichen Konditionen verlängert oder zurückbezahlt.

- **Zins** = Schuld × Satz / 12 pro Monat. Er läuft auf und wird zusammen mit der Amortisation am **Quartalsende** (März, Juni, September, Dezember) bezahlt.
- **Amortisation:** fixer Betrag pro Quartal, direkt (reduziert die Schuld) oder **indirekt** (Einzahlung in die [Säule 3a](#s-pillar3a) der Person; bei zwei Personen wählen Sie das Konto an der Tranche).
- **Indirekte Amortisation nur bis zum AHV-Bezug:** Ab dem Monat des AHV-Bezugs der Person wird derselbe Betrag **direkt** amortisiert und verringert die Hypothek. Eine Periode, die erst ab dem AHV-Bezug beginnt (auch eine Erneuerung), amortisiert immer direkt; die Auswahl zeigt dann «direkt (ab AHV)». Die Vorschau der Periode vermerkt «+ 3a bis AHV», wenn eine Periode über den AHV-Bezug hinaus läuft. Da jede Person einen AHV-Bezug hat, gilt die Regel immer. Der Wechsel erscheint in den [Ereignissen](#r-events) («Amortisation Hypothek … ab AHV-Bezug direkt statt über die Säule 3a»), wenn die Tranche vorher indirekt amortisiert hat und weiter amortisiert wird.
- Zahlt die Säule 3a mit «Hypothek zurückzahlen» aus, gehen die Beträge zuerst an Tranchen mit indirekter Amortisation in dieses Konto.
- Bei Verkauf werden die aufgelaufenen Zinsen sofort bezahlt.

## @f-re-emergency Notverkauf
Stichworte: Verkauf, Liquidität, Schwelle, Zwangsverkauf

Wird das liquide Vermögen knapp, verkauft die App die Immobilie im Folgemonat. Die **Schwelle** sind Monate ungedeckter Ausgaben: Deckt das liquide Vermögen weniger Monate (Ausgaben minus Einkommen), ist ein Notverkauf nötig.

Ein Notverkauf zeigt, dass die Planung ohne Immobilienverkauf nicht aufgeht. Ist [Immobilien nie verkaufen](#f-re-never-sell) eingeschaltet, gibt es keinen Notverkauf. Er erscheint in den [Ereignissen](#r-events) der Ergebnisse. Je höher die Schwelle, desto früher wird verkauft.

## @f-re-never-sell Immobilien nie verkaufen
Stichworte: Schalter, Verkauf, Notverkauf, Tragbarkeit, ohne Verkauf, behalten

Mit diesem Schalter oben im Reiter «Immobilien» prüfen Sie, ob die Planung auch aufgeht, wenn Sie keine Immobilie verkaufen. Er gilt für **alle** Immobilien der Planung und wird in der Datei gespeichert (Standard: aus).

- **Kein Verkauf:** kein geplanter Verkauf («Verkauf ab»), kein [Notverkauf](#f-re-emergency) und kein Verkauf, weil eine Hypothek bei der Erneuerung nicht [tragbar](#f-re-afford) ist. Die Hypothek wird dann **voll erneuert**; ein Ereignis meldet «nicht tragbar». Bei mehreren Läufen zeigt die Kennzahl **«Nicht tragbare Erneuerungen»** der [Übersicht](#r-property), in wie vielen Läufen das vorkam (Anzahl und Anteil); eine bezahlte Einmalzahlung zählt nicht dazu.
- **Kosten laufen weiter:** Zinsen, Amortisation und Nebenkosten bis zum Ende der Planung.
- **Käufe:** Ein geplanter Kauf mit Datum oder Meilenstein findet statt. Käufe und Mieten «nach Verkauf» finden **nicht** statt; die Prüfung weist darauf hin, ebenso auf geplante Verkäufe.
- **Vermögen aufgebraucht:** Reicht das liquide Vermögen nicht mehr, zählt der Lauf als aufgebraucht, wie ohne Immobilie. Die Erfolgsquote ist deshalb meist tiefer als mit Verkauf.
- **Nur lesbar:** Ist der Schalter an, sind alle übrigen Felder des Reiters grau und nur lesbar. Die Werte bleiben erhalten und gelten beim Ausschalten wieder unverändert.

## @f-re-afford Tragbarkeit
Stichworte: Bank, Belehnung, kalkulatorischer Zins, Eigenmittel, Hypothek

Die Tragbarkeitsrechnung der Banken, angewandt auf Käufe «nach Verkauf» und bei jeder Erneuerung einer Hypothekarperiode. Eine Hypothek ist tragbar, wenn die **kalkulatorischen Kosten** einen Anteil des Einkommens nicht übersteigen.

- **Höchstens vom Einkommen:** üblich ein Drittel (33 %).
- **Kalkulatorischer Zins:** meist 5 %, unabhängig vom aktuellen Zins.
- **Nebenkosten:** üblich 1 % des Preises pro Jahr.
- **Einkommen:** Angerechnet werden nur Erwerbseinkommen und Renten, **nicht** die Entnahme aus dem Vermögen. Reicht das Einkommen nicht, wird die Hypothek gekürzt; der Kauf zeigt dann die **Einmalrückzahlung** aus dem Vermögen als Ereignis.
- **Höchste Belehnung:** meist 80 % des Preises. Mindestens 20 % sind Eigenmittel.

Ist der Kauf nicht tragbar, wird die Hypothek gekürzt. Reichen auch dann die Eigenmittel nicht, wird nicht gekauft und es beginnen die Mieten.

**Bei jeder Erneuerung** einer Hypothekarperiode (auch bei Immobilien im Besitz) prüft die App die Tragbarkeit der ganzen Hypothek neu, mit dem Einkommen des Erneuerungsmonats (Erwerb und Renten). Ist die Hypothek nicht mehr tragbar, berechnet die Simulation die **minimal nötige Einmalzahlung**, damit sie es wieder ist, und zahlt sie aus dem Vermögen (anteilig auf alle Tranchen). Das gilt nur, wenn das Vermögen danach **nicht negativ** ist **und** das [Notverkaufskriterium](#f-re-emergency) nach der Zahlung **nicht** ausgelöst wird (geschätzt mit den geplanten Ausgaben und den Wohnkosten nach der Zahlung). Dann wird die Hypothek mit dem reduzierten Betrag erneuert. Sonst gibt es **keine Zahlung**, die Hypothek wird **nicht erneuert**, und ein **Notverkauf** der Immobilie wird eingeleitet (ausgeführt im Folgemonat). Die Ergebnisansicht protokolliert jede Erneuerung (Datum und Alter, Tragbarkeit, Einmalzahlung, neue Hypothek, Notverkauf mit Grund «Vermögen < 0» oder «Notverkaufskriterium») und zählt bei mehreren Läufen, in wie vielen Läufen Einmalzahlungen und Notverkäufe vorkommen, auch je Perzentil. Typischer Fall: Das Einkommen sinkt mit der Pensionierung.

## @s-scenarios Szenarien & Simulation
Stichworte: Variation, Sweep, Parameter, Läufe, Umfang

Hier legen Sie fest, welche Parameter **systematisch variiert** werden. «Simulation starten» rechnet die Planung wie erfasst und zusätzlich jede Kombination der Variationen mit allen Renditepfaden. Das Ergebnis ist eine Tabelle und eine Heatmap in den [Szenario-Ergebnissen](#r-scenarios).

Die Dauer der Simulation steht ausserdem links neben dem Knopf «Simulation starten». Während der Berechnung können Sie sie mit **Abbrechen** stoppen.

Die Zahl der Kombinationen ist **nicht begrenzt**; erst bei mehr als **1'000'000** Kombinationen lehnt der Rechenkern ab (Speicherschutz). Liegt die geschätzte Rechenzeit über **einer Minute**, fragt die App vor dem Start nach (Dauer und Zahl der Läufe, «Trotzdem starten» oder «Abbrechen»). Die Anzeige «Umfang» zeigt, wie viele Läufe insgesamt entstehen (Variationen × Pfade); ein **gelbes Warnsymbol (⚠)** daneben erscheint, sobald die geschätzte Rechenzeit **90 Sekunden** übersteigt, und eine **grobe Schätzung der Dauer**. Sie beruht auf einer kurzen Messung Ihrer Planung auf diesem Rechner. Die Spanne («meist … bis …») berücksichtigt, dass Läufe je nach Planung unterschiedlich lange dauern. Jede zusätzliche Variation und jeder zusätzliche Pfad verlängert die Rechenzeit entsprechend: 35 Kombinationen mit 50 000 Pfaden dauern bei Bootstrap etwa 10 Sekunden, mit dem parametrischen Modell rund eine Minute.

Die Variationen selbst: [Variationen](#f-sweep).

## @f-sweep Variationen
Stichworte: Sweep, Von, Bis, Schritt, Parameter, Heatmap

Eine Variation hat einen **Parameter**, einen Bereich **Von … Bis** und einen **Schritt**. Die App rechnet jeden Wert und zeigt die Wirkung auf Erfolgsquote und Endvermögen.

| Parameter | Bedeutung |
|---|---|
| Aktienanteil | Aktienquote: Aktien am Portfolio aus Aktien und Obligationen (ohne Cash) |
| Kapitalbezug Pensionskasse | Anteil Kapitalbezug |
| Historischer Startmonat | nur mit der Quelle «historisch» |
| Datum Bezug Pensionskasse | Monat des Meilensteins «Bezug Pensionskasse» |
| Ausgaben-Faktor | Faktor auf die Lebenshaltungskosten ohne den Wohnanteil (0.9 = 10 % weniger); die Wohnkosten bleiben |
| Entnahmerate (fixe Rate / Guardrails-Start) | Rate der fixen Entnahme bzw. Startrate bei Guardrails |

Bei zwei Variationen entsteht eine **Heatmap** über beide Parameter. Alle Kombinationen verwenden **dieselben Renditepfade**. Unterschiede kommen damit nur von den Parametern, nicht vom Zufall.
