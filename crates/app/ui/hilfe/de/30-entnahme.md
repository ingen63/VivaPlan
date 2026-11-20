# Entnahmestrategie

## @s-withdrawal Entnahmestrategie: Überblick
Stichworte: Entnahme, Strategie, Pensionierung, Leitplanken, Lebenshaltung

Die Strategie legt fest, **wie nach der Pensionierung Geld aus dem Vermögen entnommen und umgeschichtet wird**. Es gibt sechs Strategien, die sich in zwei Gruppen teilen. Die 3-Topf-Strategie lässt sich zudem mit Guardrails verbinden:

**Die Ausgaben sind fix, das Vermögen schwankt** (Entscheid: Wie ist das Vermögen angelegt?)

- [Standard](#f-wd-standard): Cash-Reserve plus fester Aktienanteil
- [3-Topf](#f-wd-three_buckets): Cash, Obligationen und Aktien in drei Töpfen
- [3-Topf mit Guardrails](#f-wd-three_buckets_guardrails): die Töpfe bestimmen die Anlage, die Guardrails die Ausgaben

**Die Ausgaben folgen dem Vermögen** (Entscheid: Wie viel gebe ich aus?)

- [Fixe Entnahmerate](#f-wd-fixed_rate): jedes Jahr ein fester Anteil des Vermögens
- [Guardrails](#f-wd-guardrails): Startrate mit Leitplanken (Guyton-Klinger)
- [VPW](#f-wd-vpw): Variable Prozententnahme, so dass das Vermögen bis zu einem Alter reicht
- [Endowment](#f-wd-endowment): geglättete Entnahme (Yale/Tobin)

**Sicherheit gegen Komfort:** Bei den ersten beiden Strategien bleiben Ihre Ausgaben wie geplant, dafür kann das Vermögen aufgebraucht werden. Bei den vier anderen wird das Vermögen **nie ganz aufgebraucht**, dafür schwanken die Ausgaben mit dem Börsenverlauf. Ob Sie das möchten, ist eine persönliche Frage. Mit den [Grenzen der Lebenshaltung](#f-wd-limits) setzen Sie eine Untergrenze.

**Wann gilt die Strategie?** Erst **ab Pensionierung**, bei zwei Personen ab der Pensionierung der **ersten** Person. Das gilt **auch für «Standard»**. Davor gilt keine Strategie: Die App hält so viel Cash, wie unter [Vermögen & Anlage](#s-wealth) erfasst (mit der Teuerung), und legt den Rest nach der Aktienquote an. Arbeitet die zweite Person noch, zählt ihr Einkommen zu «Einkommen und Renten»: Bei den Strategien, deren Ausgaben dem Vermögen folgen, ist die Lebenshaltung dann um dieses Einkommen höher. Fällt es mit der zweiten Pensionierung weg, sinkt die Lebenshaltung, die Entnahme bleibt gleich.

**Zuflüsse wirken sofort:** Bei den vier Strategien, deren Ausgaben dem Vermögen folgen (bei 3-Topf gilt stattdessen: Zuflüsse gehen nur in Topf 3; bei 3-Topf mit Guardrails gilt beides: die Guardrails rechnen die Entnahme neu, das Geld geht in Topf 3), wendet die App die Strategie nicht nur im Januar an, sondern **auch im Monat eines Zuflusses**: bei einem einmaligen Betrag (zum Beispiel Erbschaft), einer Pensionskassenauszahlung und dem Erlös eines Immobilienverkaufs. Sie verhält sich dann wie im Januar (die jährliche Entnahme wird neu berechnet), und das gilt ab diesem Monat für den Rest des Jahres, nicht rückwirkend. Mehrere Zuflüsse im selben Monat ergeben **eine** Anwendung nach allen Zuflüssen und Käufen. «Standard» legt ohnehin jeden Monat an.

**Verkauf mit Folgekauf:** Ist im Monat nach dem Verkauf der Kauf einer anderen Immobilie geplant (fester Kaufmonat oder «nach Verkauf»), wird der Erlös **reserviert** und die Strategie im Verkaufsmonat **nicht** angewendet. Im Kaufmonat wird zuerst gekauft, danach wendet die Strategie auf den Überschuss an. Bei Verkauf im Dezember und Kauf im Januar gilt die Reihenfolge Kauf, dann die übliche Januar-Anwendung (einmal). Ein reservierter Erlös zählt **nicht** zum Vermögen, aus dem die Strategien ihre Entnahme berechnen. Die Ereignisliste und der Abschnitt «Immobilien» nennen, was geschehen ist («Verkaufserlös reserviert für Kauf im Folgemonat», «Entnahmestrategie angewendet nach …»).

**Welche passt?** Siehe [Welche Strategie passt?](#wd-compare). Eine rechnerische Antwort liefert die [Optimierung](#s-optimize).

## @wd-compare Welche Strategie passt?
Stichworte: Vergleich, Empfehlung, Auswahl, Vor- und Nachteile

| Strategie | Ausgaben | Vermögen | Gut für | Nachteil |
|---|---|---|---|---|
| Standard | fix, wie geplant | kann ausgehen | einfache Planung, hohes Sicherheitsbedürfnis bei Ausgaben | Ausgehen des Geldes ist möglich, kein Mechanismus gegen Krisen |
| 3-Topf | fix, wie geplant | kann ausgehen | wer Krisenjahre aus Reserven überbrücken will | Regeln sind komplizierter, Parameter müssen passen |
| 3-Topf mit Guardrails | folgt dem Vermögen mit Leitplanken | geht fast nie aus | planbare Ausgaben und Krisenjahre aus Reserven | viele Parameter, tieferer Aktienanteil |
| Fixe Rate | folgt dem Vermögen | geht nie aus | einfachste «Nie-aufbrauchen»-Regel | grosse Schwankung der Ausgaben |
| Guardrails | folgt dem Vermögen mit Leitplanken | geht fast nie aus | planbare Ausgaben mit Korrektur bei Krisen | Kürzungen von 10 % möglich |
| VPW | folgt dem Vermögen, steigt mit dem Alter | bis zum Planungsalter aufgebraucht | wer das Vermögen aufbrauchen möchte | Ausgaben schwanken, Planungsalter ist eine Annahme |
| Endowment | folgt dem Vermögen verzögert | geht fast nie aus | ruhige Ausgaben | reagiert langsam, kann im Crash zu viel entnehmen |

**Praktischer Weg:** Starten Sie mit «Standard» als Ausgangslage, rechnen Sie dann die [Optimierung](#s-optimize) über alle Strategien und schauen Sie, wie viel Konsum und wie viel Sicherheit die Alternativen bringen.

## @f-wd-choice Strategie wählen
Stichworte: Auswahl, Karte

Wählen Sie die Strategie mit den Karten oben im Abschnitt. Jede Karte hat ein ⓘ mit der ausführlichen Beschreibung. Die Eingaben der gewählten Strategie erscheinen darunter.

Die Strategien, deren Ausgaben dem Vermögen folgen, und 3-Topf mit Guardrails legen **ab der Pensionierung die Lebenshaltung** fest. Die geplanten Ausgaben (Abschnitt «Einnahmen & Ausgaben») gelten dann nur noch bis zur Pensionierung.

Siehe [Überblick](#s-withdrawal) und [Welche Strategie passt?](#wd-compare).

## @f-wd-standard Standard
Stichworte: Reserve, Aktienanteil, Rebalancing

**Die Ausgaben sind fix und folgen Ihrer Planung.** Das Vermögen wird so aufgeteilt:

- Das **Cash** wird auf die [Cash-Reserve](#f-reserve) gehalten (hier einstellbar; alle anderen Strategien nehmen fest 3 Monate), ausgedrückt in Monaten der ungedeckten Ausgaben (Ausgaben minus Einkommen und Renten).
- Der **Rest** wird nach dem [Aktienanteil](#f-stockshare) in Aktien und Obligationen aufgeteilt.
- Das Verhältnis von Aktien und Obligationen wird monatlich wiederhergestellt (Rebalancing).

Wie alle Strategien gilt sie erst ab der Pensionierung; davor gilt die Cash-Regel aus [Vermögen & Anlage](#s-wealth). Einfach, transparent, aber ohne Schutz davor, dass in einer Krise zu viel verkauft wird.

## @f-wd-three_buckets 3-Topf-Strategie
Stichworte: Topf, Liquidität, Obligationen, Aktien, Nachfüllen, Januar, Schwelle

Das Vermögen wird **ab der Pensionierung** in drei Töpfe geteilt. Die Ausgaben bleiben **wie geplant**:

| Topf | Inhalt | Ziel |
|---|---|---|
| Topf 1 | Cash | Jahre × Netto-Bedarf, deckt die laufenden Ausgaben |
| Topf 2 | Obligationen | Jahre × Netto-Bedarf, Puffer |
| Topf 3 | Aktien | der Rest |

**Eingaben:**

- **Topf 1 (Jahre):** Wie viele Jahre des Netto-Bedarfs als Cash gehalten werden. Üblich: 1 bis 3.
- **Topf 2 (Jahre):** Wie viele Jahre als Obligationen. Üblich: 3 bis 8.
- **Aus Topf 3 nur verkaufen, wenn Aktienrendite ≥:** Renditekriterium. Gemessen wird die **Aktienrendite der letzten 12 Monate** einschliesslich des laufenden Monats. In den ersten 12 Monaten der Simulation zählen auch die Monate vor dem Start: Jeder Renditepfad bringt sie mit (historisch und im Bootstrap die echten Monate davor, parametrisch zufällig gezogene, bei fixen Renditen der feste Satz). Fehlen sie, weil die Zeitreihe erst später beginnt, gilt die Rendite seit dem Start. Ist das Kriterium nicht erfüllt, sind nur **Verkäufe aus Topf 3** in der Januar-Auffüllung gesperrt. Bei 0 % wird nur nach einem Gewinnjahr verkauft; negative Werte erlauben das Auffüllen auch nach leichten Verlusten.
- **Reserve in Topf 1 unterjährig (Monate):** Wird Topf 1 unterjährig knapp (Fall C), bleiben nach den Ausgaben des Monats noch Min(Reserve, Monate bis Januar) − 1 Monatsbedarfe darin. Standard: 3.

**Regeln:**

- Alle Ausgaben werden aus **Topf 1** bezahlt. **Kein Topf fällt unter 0**; einzige Ausnahme siehe «Alle Töpfe leer».
- **Ziele** (immer aus dem Netto-Bedarf, nie aus dem Topfstand): Ziel Topf 1 = Jahre × Netto-Bedarf, Ziel Topf 2 = Jahre × Netto-Bedarf.
- **Januar-Auffüllung (jeden Januar):** Ist das Renditekriterium erfüllt, wird Topf 1 aus Topf 2 aufgefüllt, der Rest kommt aus Topf 3; danach wird Topf 2 aus Topf 3 aufgefüllt. Ist es nicht erfüllt, wird nichts aus Topf 3 verkauft, Topf 1 wird nur aus Topf 2 aufgefüllt (reicht Topf 2 nicht, bleibt Topf 1 unter dem Ziel) und Topf 2 wird nicht nachgefüllt. Liegt Topf 1 oder Topf 2 über dem Ziel, geht der Überschuss in Topf 3.
- **A · Start der Entnahmephase, PK-Kapitalbezug, Neustart:** Beim Start (sobald die **erste** Person pensioniert ist) kommt das gesamte liquide Vermögen zuerst in Topf 3, ein PK-Kapitalbezug später ebenfalls. Danach läuft die Auffüllung wie im Januar, **ohne** Renditekriterium und (unterjährig) ohne Überschussbuchung in Topf 3. Die Ziele sind um die in diesem Jahr schon vergangenen Monate gekürzt ((Jahre × 12 − Monate) / 12 × Netto-Bedarf; Start im März: 2 Monate). A läuft bei jedem Auslöser, mehrere im selben Monat ergeben einen Lauf.
- **B · externe Zuflüsse:** Einmalige Zuflüsse, 3a-Auszahlungen, ein Immobilienverkauf ohne Folgekauf und Notverkaufserlöse gehen **nur in Topf 3**. Topf 1 und 2 werden nicht aufgefüllt.
- **C · Vorausschau (jeden Monat):** Die App prüft, ob Topf 1 die Ausgaben des Monats (laufende und einmalige, auch Steuern) deckt. Wenn nicht, hängt der Ablauf vom Renditekriterium ab:
  - **C1 (Kriterium erfüllt):** Ablauf wie die Januar-Auffüllung: Topf 1 aus Topf 2, der Rest aus Topf 3, dann Topf 2 aus Topf 3. Die Ziele sind um die seit Januar vergangenen Monate gekürzt ((Jahre × 12 − Monate) / 12 × Netto-Bedarf; April = 3 Monate, Dezember = 11), das Ziel von Topf 1 zusätzlich um die **einmaligen Ausgaben dieses Monats** erhöht. Es gibt **keine Überschussbuchung** in Topf 3. Beispiel April, Topf 1 = 2 Jahre, Topf 2 = 1 Jahr: Ziel Topf 1 = 1,75 × Netto-Bedarf + Einmalausgaben, Ziel Topf 2 = 0,75 × Netto-Bedarf.
  - **C2 (Kriterium nicht erfüllt):** Topf 1 wird aus **Topf 2, dann Topf 3** so aufgefüllt, dass er die Ausgaben deckt und danach noch (Min(Reserve, Monate bis Januar) − 1) Monatsbedarfe enthält («Monate bis Januar» zählt den aktuellen Monat mit: Dezember = 1). Topf 2 wird nicht aufgefüllt.
  - Reicht Topf 1 nach C1 für die Zahlungen des Monats noch nicht, folgt C2. Reichen Topf 2 und 3 nicht, folgt der Notverkauf (siehe «Alle Töpfe leer»).
- **Einmalige Ausgaben und Ziele:** In der Januar-Auffüllung und bei A ist das Ziel von Topf 1 ebenfalls um die einmaligen Ausgaben des Monats erhöht, damit eine grosse Einmalausgabe im Januar nicht zusätzlich C auslöst. **Überschussbuchungen** in Topf 3 gibt es nur in der Januar-Auffüllung.
- **Immobilien:** Ein Verkaufserlös, für den im Folgemonat ein Kauf geplant ist, wird **reserviert** (gehört zu keinem Topf). Beim Kauf zahlt zuerst die Reservierung die Eigenmittel, ein Rest kommt aus Topf 1 (über C); ein Überschuss der Reservierung geht in Topf 3. Ein Kauf im Januar nach einem Verkauf im Dezember wird vor der Januar-Auffüllung abgewickelt.
- **Reihenfolge im Monat:** Regelmässige Einnahmen in Topf 1, Zuflüsse direkt an ihr Ziel (Topf 3 bzw. Reservierung, nie zuerst in Topf 1), Immobilienkäufe (Eigenmittel aus der Reservierung; ein Rest gilt als einmalige Ausgabe des Monats und belastet Topf 1 nicht vorher), Januar-Auffüllung bzw. A, Vorausschau (C) mit **allen** Ausgaben des Monats (laufende, einmalige, Kaufrest, Amortisationen und Einmalzahlungen der Hypothek, Steuern), danach werden die Ausgaben aus Topf 1 bezahlt. Die Umbuchungen werden technisch nach dem Renditeschritt des Monats gebucht; die Tabelle zeigt den Stand vor den Ausgaben des Monats.
- **Alle Töpfe leer:** Reichen Topf 1, 2 und 3 nicht, wird eine Immobilie notverkauft (Erlös in Topf 3, dann C erneut). Das Notverkaufskriterium der Hypothekenlogik gilt parallel und kann früher auslösen. Ist keine Immobilie mehr da, darf Topf 1 negativ werden, Topf 2 und 3 sind stillgelegt, und alle Einnahmen und Zuflüsse fliessen in Topf 1. Sobald das Vermögen wieder positiv ist, startet A neu.

**Netto-Bedarf (pro Jahr, geplante Werte des laufenden Jahres):** Lebenshaltung + Wohnen + Einzahlung in die Säule 3a + Einkommenssteuer − regelmässige Einnahmen (AHV-Rente, PK-Rente, Erwerbseinkommen wie angegeben). Nicht gezählt werden einmalige Ausgaben, Anschaffungen, Reisen, Amortisationen, Immobilienkäufe, Sondersteuern (Grundstückgewinn-, Kapitalbezugs- und Vermögenssteuer) und einmalige Zuflüsse (Kapitalbezug, Erbschaft, Verkaufserlöse). Die Einkommenssteuer ist die Steuer auf dem Jahreseinkommen (Renten und Erwerbseinkommen). Ist der Netto-Bedarf null oder negativ, sind alle Ziele null. Bei einem Auslöser A werden die Monate ab dem Auslöser auf 12 Monate hochgerechnet, **auch die Einkommenssteuer**. Dieser Netto-Bedarf gilt für alle Auffüllungen bis zum nächsten Januar oder zum nächsten Auslöser A, auch für die Vorausschau C. Steigen die Ausgaben (Teuerung), wachsen die Töpfe mit. In der Ergebnisansicht zeigt die Tabelle «3-Topf-Strategie» die Herleitung (Ausgaben, abgezogene Einnahmen, Netto-Bedarf, Ziele), das Renditekriterium, den ausgelösten Fall (Januar, A, B, C) und alle Umbuchungen.

**Idee:** Nach einem Börseneinbruch müssen keine Aktien zu tiefen Kursen verkauft werden. Topf 1 und Topf 2 überbrücken mehrere Jahre, bis sich die Aktien erholt haben. Die Kosten sind der tiefere Aktienanteil (mehr Obligationen und Cash) und damit tendenziell weniger Rendite.

## @f-wd-three_buckets_guardrails 3-Topf mit Guardrails
Stichworte: Kombination, Töpfe, Leitplanken, Ausgaben, Anlage, Netto-Bedarf

Auf der Karte «3-Topf» wählen Sie unter **Ausgaben ab Pensionierung** zwischen «wie geplant» und «mit Guardrails steuern». Mit Guardrails gilt:

- **Die Guardrails steuern die Ausgaben.** Startrate, Teuerung, Leitplanken und [Grenzen](#f-wd-limits) wirken genau wie bei der Strategie [Guardrails](#f-wd-guardrails). Bei gleichem Vermögensverlauf ist die Entnahme in beiden gleich.
- **Die Töpfe steuern die Anlage.** Alle Regeln der [3-Topf-Strategie](#f-wd-three_buckets) gelten unverändert (Januar-Auffüllung, A, B, C, Renditekriterium, Notverkauf, «alle Töpfe leer»).
- **Netto-Bedarf und Topfziele** rechnen mit der Lebenshaltung der Guardrails statt mit den geplanten Ausgaben, nach Unter- und Obergrenze. Ohne Grenze und ohne Wohnkosten ist der Netto-Bedarf die Guardrails-Entnahme plus Einkommenssteuer. Die Tabelle «3-Topf-Strategie» vermerkt «Guardrails» bei der Lebenshaltung.
- **Reihenfolge im Januar:** zuerst die Guardrails (Teuerung, Leitplanken), dann Netto-Bedarf und Ziele, dann die Auffüllung. Kürzt die Leitplanke die Entnahme um 10 %, werden auch die Ziele von Topf 1 und 2 um 10 % kleiner.
- **Start:** Töpfe und Guardrails starten gemeinsam mit der Pensionierung der ersten Person. Zuerst wird die Startentnahme festgelegt, danach teilt A das Vermögen auf.
- **Zuflüsse:** Die Guardrails rechnen die Entnahme neu, das Geld geht in Topf 3. Die Topfziele bleiben bis zum nächsten Januar; die Vorausschau (C) rechnet aber sofort mit der neuen monatlichen Lebenshaltung.
- **Zwei Renditemasse:** Die Guardrails setzen die Teuerung nach einem Verlustjahr im **Kalenderjahr** aus, das Renditekriterium der Töpfe misst die **letzten 12 Monate**. Im Januar ist das dasselbe.

**Beispiel:** Startrate 4 %, Vermögen 1 Mio., Topf 1 = 1 Jahr, Topf 2 = 2 Jahre, keine Renten. Die Entnahme ist 40'000 pro Jahr, Topf 1 hält 40'000, Topf 2 80'000, der Rest liegt in Aktien. Nach einem Crash kürzt die Leitplanke auf 36'000: Im nächsten Januar sind die Ziele 36'000 und 72'000.

**Idee:** Die Töpfe vermeiden Verkäufe von Aktien zu tiefen Kursen, die Leitplanken verhindern, dass zu viel entnommen wird. In der [Optimierung](#s-optimize) heisst die Kombination «3-Topf + Guardrails».

## @f-wd-fixed_rate Fixe Entnahmerate
Stichworte: Prozent, Rate, 4 Prozent, Trinity, konstante Rate

Jedes Jahr wird **dieselbe Rate** des liquiden Vermögens entnommen. Die Ausgaben **folgen** damit dem Vermögen.

- Jeden **Januar** (und bei Pensionierung) wird die Jahresentnahme als **Rate × liquides Vermögen** neu festgelegt.
- Die Entnahme ist **Lebenshaltung + Wohnkosten + Einzahlung in die Säule 3a minus Einkommen und Renten** (vor Steuern). Die Lebenshaltung ergibt sich daraus. Die geplanten Ausgaben aus dem Abschnitt «Einnahmen & Ausgaben» gelten nur bis zur Pensionierung.
- **Das Vermögen wird nie ganz aufgebraucht**, weil immer nur ein Anteil entnommen wird. Dafür schwanken die Ausgaben stark: Fällt das Vermögen um 30 %, fällt die Entnahme um 30 %.

**Welche Rate?** Die oft genannte «4-Prozent-Regel» stammt aus einer US-Untersuchung für 30 Jahre mit fixer, teuerungsangepasster Entnahme (**nicht** mit einer Rate auf das aktuelle Vermögen). Für eine Rate auf das aktuelle Vermögen sind 3 % bis 5 % üblich. Beachten Sie: Die Rate gilt für die **Entnahme**, also für Ihre Ausgaben **minus** AHV, Pensionskassenrente und Einkommen. Die Lebenshaltung ist damit Rate × Vermögen **plus** Renten und Einkommen. Mit hohen Renten sind die Ausgaben deshalb viel weniger vom Vermögen abhängig, als die Rate vermuten lässt.

Die **Untergrenze** der [Grenzen der Lebenshaltung](#f-wd-limits) verhindert zu tiefe Ausgaben.

## @f-wd-guardrails Guardrails (Guyton-Klinger)
Stichworte: Leitplanken, Startrate, obere, untere, Anpassung, Guyton, Klinger

Die Entnahme startet mit einer **Startrate** und wird jedes Jahr mit der Teuerung angepasst. **Leitplanken** greifen ein, wenn die Entnahme im Verhältnis zum Vermögen zu hoch oder zu tief wird.

**Eingaben** (Standard in Klammern):

- **Startrate (5 %):** Anteil des liquiden Vermögens, der bei der Pensionierung (der ersten Person) entnommen wird. Ein für einen Kauf reservierter Verkaufserlös zählt nicht dazu, auch nicht bei den Leitplanken.
- **Obere Leitplanke (20 %):** Übersteigt die aktuelle Rate die Startrate um diesen Anteil, wird gekürzt.
- **Untere Leitplanke (20 %):** Liegt die aktuelle Rate um diesen Anteil unter der Startrate, wird erhöht.
- **Anpassung (10 %):** Um wie viel bei einer verletzten Leitplanke gekürzt bzw. erhöht wird.

**Ablauf jeden Januar:**

1. Die Entnahme des Vorjahres wird mit der **Teuerung** angepasst, **ausser** nach einem Verlustjahr der Aktien, solange die aktuelle Rate über der Startrate liegt (keine Teuerungsanpassung in der Krise).
2. Dann wird die aktuelle Rate (Entnahme / Vermögen) mit den Leitplanken verglichen:
   - **Über Startrate × (1 + obere Leitplanke):** Die Entnahme wird um die Anpassung gekürzt.
   - **Unter Startrate × (1 − untere Leitplanke):** Die Entnahme wird um die Anpassung erhöht.

**Beispiel:** Startrate 5 %, obere Leitplanke 20 %, Anpassung 10 %. Fällt das Vermögen so stark, dass die aktuelle Rate über 6 % steigt, kürzt die App die Entnahme um 10 %.

**Idee:** Sie geben anfangs viel aus (die Startrate ist oft höher als bei einer fixen Rate), akzeptieren aber, in Krisen zu sparen. Die Kürzungen sind selten und klein, die Ausgaben bleiben **planbarer** als bei der reinen fixen Rate.

Die Entnahme ist wie bei der fixen Rate **Lebenshaltung + Wohnkosten + Einzahlung in die Säule 3a minus Einkommen und Renten** (vor Steuern). [Grenzen der Lebenshaltung](#f-wd-limits) sind möglich.

## @f-wd-vpw Variable Prozententnahme (VPW)
Stichworte: VPW, Planungsalter, Realrendite, Annuität, Bogleheads

Jeden Januar wird der Anteil des liquiden Vermögens entnommen, der es bei einer angenommenen **Realrendite** **gleichmässig bis zum Planungsalter aufbraucht**. Das ist eine **Annuität**.

**Eingaben:**

- **Planungsalter** (Alter der Hauptperson, Standard 100): Bis dahin soll das Vermögen reichen. Wählen Sie es hoch genug, damit die Sicherheit bleibt.
- **Realrendite** (Standard 2 %): Die reale Jahresrendite des Vermögens, die die Strategie annimmt. Eine zu hohe Annahme führt zu Entnahmen, die das Vermögen schneller aufbrauchen.

**Eigenschaften:**

- Die entnommene **Rate steigt mit dem Alter**, weil die verbleibende Zeit kürzer wird (zum Beispiel bei Realrendite 0 % und noch 25 Jahren: 4 %, bei 10 Jahren: 10 %).
- Im Planungsalter wäre das Vermögen aufgebraucht. Die Strategie ist also für **«das Geld soll am Ende weg sein»** gebaut, nicht für Vererben.
- Die Ausgaben folgen dem Vermögen und schwanken entsprechend. Eine Untergrenze begrenzt das.

## @f-wd-endowment Endowment-Regel (Yale/Tobin)
Stichworte: Glättung, Zielrate, Yale, Tobin, geglättet

Die Entnahme ist ein **gewichteter Mittelwert** aus dem Vorjahr (mit Teuerung) und einer Rate auf das aktuelle Vermögen:

> Entnahme im Januar = (1 − Glättung) × Vorjahresentnahme mit Teuerung + Glättung × Zielrate × aktuelles Vermögen

**Eingaben:**

- **Zielrate** (Standard 4 %): Rate, der sich die Entnahme langfristig annähert.
- **Glättung** (Standard 30 %): Gewicht des aktuellen Vermögens. Bei 100 % ist es die [fixe Rate](#f-wd-fixed_rate), bei 0 % wird nur das Vorjahr mit Teuerung fortgeschrieben (wie eine fixe Entnahme).

**Wirkung:** Mit kleiner Glättung ändern sich die Ausgaben von Jahr zu Jahr wenig. Sie folgen dem Vermögen aber **langsam**. Fällt das Vermögen stark, entnimmt man kurzfristig zu viel, das gleicht sich erst über Jahre aus. Diese Regel verwenden Stiftungen und Universitäten, um stabile Ausschüttungen zu erreichen.

## @f-wd-limits Grenzen der Lebenshaltung
Stichworte: Untergrenze, Obergrenze, Floor, Ceiling, Mindestausgaben

Bei den Strategien, die die Lebenshaltung selbst festlegen (fixe Rate, Guardrails, VPW, Endowment, 3-Topf mit Guardrails), können Sie eine **Untergrenze** und eine **Obergrenze** setzen. Beide sind **Anteile der geplanten Ausgaben**. **0 bedeutet keine Grenze.**

- **Untergrenze:** Die Lebenshaltung sinkt nie unter diesen Anteil der geplanten Ausgaben (in heutigen Preisen, mit der Teuerung angepasst). Beispiel: 80 % heisst, Sie geben mindestens 80 % dessen aus, was Sie geplant haben, auch wenn die Strategie weniger erlauben würde. **Dann kann das Vermögen aufgebraucht werden.** Die Untergrenze macht den Preis der Sicherheit bei den Ausgaben sichtbar.
- **Obergrenze:** Die Lebenshaltung steigt nie über diesen Anteil. Beispiel: 130 % verhindert, dass Sie in guten Jahren wesentlich mehr ausgeben, als Sie geplant haben.

Die Grenzen sind eine Möglichkeit, **feste Ansprüche** (Wohnen, Gesundheit) gegen die flexible Entnahme abzusichern. Bei der [Optimierung](#s-optimize) bleiben die Grenzen wie in der Planung.

Die Grenzen gelten nicht für «Standard» und «3-Topf» mit Ausgaben wie geplant, weil dort die Ausgaben ohnehin fix sind.
