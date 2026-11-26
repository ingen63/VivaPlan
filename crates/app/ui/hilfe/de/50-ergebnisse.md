# Ergebnisse lesen

## @r-overview Ergebnisse: Übersicht
Stichworte: Erfolgsquote, Kennzahlen, Perzentile, Endvermögen, Erschöpfung, Ergebnisansicht

Nach «Simulation starten» zeigt die App einen **Fortschrittsbalken** mit vergangener und geschätzter Restzeit und einem Knopf **Abbrechen**. Danach öffnet sich die Ergebnisansicht. Ihre Kopfzeile nennt zuerst den **Namen des Ergebnisses**, wie es in der [Liste](#r-list) abgelegt ist (zum Beispiel «Beispiel · MC hist. · Standard · 16.11.26, 14:05»), danach die Art des Laufs («50'000 Läufe» bzw. «Einzellauf · …»). Ein langer Name wird gekürzt, der volle Name erscheint beim Daraufzeigen; nach dem Umbenennen in der Liste gilt der neue Name. Die Ansicht hat mehrere Registerkarten: **Übersicht**, **Grafiken**, **Einzelne Läufe** (nur bei mehreren Läufen), **Szenarien** (nur mit Variationen) und **Jahreswerte**. Alle Grafiken mit Zeitachse lassen sich zoomen: **Mit gedrückter Maustaste einen Zeitbereich aufziehen** (er wird hervorgehoben, Auswahlen unter einem Jahr zählen als Klick und werden ignoriert), oder mit den Schaltflächen **Alles, 10 J., 5 J. und 2 J.** Die Schaltflächen setzen die Breite des Bereichs **zentriert auf die Mitte des aktuellen Bereichs** (ohne Zoom: der ganzen Laufzeit). Ragt der Bereich über Anfang oder Ende hinaus, wird er verschoben, die Breite bleibt; ist die Laufzeit kürzer, sehen Sie alles. Nach einer Mausauswahl ist keine Schaltfläche aktiv, die Grafiken zeigen «Benutzerdefiniert» mit dem Bereich. Der Zoom gilt **für alle Ergebnisgrafiken gemeinsam**, in allen Registerkarten und im Vergleich, und bleibt beim Wechsel der Registerkarte erhalten; die Minigrafiken im Editor haben ihren eigenen Zoom (ab Start). Die Grafiken der Szenarien haben keine Zeitachse und deshalb keinen Zoom. Oben rechts wechseln Sie zwischen **Nominal** und **Heutige Kaufkraft**, siehe [Rendite, Inflation und reale Werte](#ret-concepts); beim Öffnen der Ergebnisse steht der Umschalter auf **Heutige Kaufkraft**.

**Ergebnis in Worten:** Zuoberst fasst die Übersicht das Ergebnis in zwei, drei Sätzen zusammen. Bei mehreren Läufen: «In 87 % (mittel) der Läufe reicht das Vermögen bis 12.2055 (Alter 95,0).» Die Einordnung in Klammern folgt den Farben der Erfolgsquote: **hoch** ab 95 %, **mittel** von 80 % bis unter 95 %, **tief** unter 80 %; massgebend ist der angezeigte, auf 0,1 % gerundete Wert. Geht in einem Lauf das Geld aus, nennt ein zweiter Satz das Alter im ungünstigsten Lauf (wie «Früheste Erschöpfung»). Bei einem Lauf steht, ob das Vermögen bis zum Ende reicht und wie gross es dann ist (nominal oder in heutiger Kaufkraft, je nach Umschalter), oder wann es aufgebraucht ist. Darunter folgt ein Vorschlag für den nächsten Schritt: unter 95 % bzw. bei aufgebrauchtem Vermögen die [Optimierung](#s-optimize), sonst die [Szenarien](#s-scenarios). Der Text ist eine Lesehilfe, keine Bewertung Ihrer Lage.

**Bei einem einzigen Lauf** (fixe Renditen, historischer Verlauf) zeigt die Übersicht fünf Kennzahlen: Vermögen bei Pensionierung, Vermögen am Ende, bis wann das Geld reicht, die PK-Rente und die geschätzten Steuern total ([Wie genau ist die Steuer?](#tax-accuracy)). Darunter folgen der Vermögensverlauf und die Liste der Ereignisse.

**Bei mehreren Läufen** (Monte Carlo, alle Startzeitpunkte) zeigt die Übersicht:

- **Erfolgsquote:** Anteil der Läufe, in denen das Vermögen bis zum Ende reicht. Grün ab 95 %, rot unter 80 %.
- **Endvermögen** am Median, am 5., 1., 95. und 99. Perzentil und im **schlechtesten Lauf**.
- **Früheste Erschöpfung:** Das früheste Alter, in dem in irgendeinem Lauf das Geld aufgebraucht war.
- die [Bandbreite des Gesamtvermögens](#r-fan) und die [Perzentil-Tabelle](#r-percentiles).

**Wie viel Erfolgsquote ist genug?** Das ist keine Rechenfrage, sondern eine Frage Ihrer Risikobereitschaft. 90 % bis 95 % gelten bei vorsichtiger Planung als ordentlich. Beachten Sie, dass eine Erfolgsquote von 95 % bedeutet, dass in 1 von 20 simulierten Zukünften das Geld ausgeht. Das ist nur eine Schätzung, siehe [Fallstricke](#ret-pitfalls).

**«Erfolg» heisst:** Das Vermögen (liquid und Immobilien) wird bis zum Ende der Simulation nicht aufgebraucht. Es heisst **nicht**, dass das Leben angenehm war: Ein Lauf kann «erfolgreich» sein und trotzdem sehr knapp enden.

## @r-percentiles Endvermögen nach Perzentil
Stichworte: Perzentil, Median, 1. Perzentil, 99. Perzentil, schlechtester Lauf, Quantil

Die Läufe werden nach ihrem **Endvermögen** sortiert. Das **n-te Perzentil** ist das Endvermögen, unter dem n % der Läufe liegen.

| Zeile | Bedeutung |
|---|---|
| Schlechtester Lauf | der Lauf mit dem tiefsten Endvermögen |
| 1. Perzentil | nur 1 % der Läufe enden schlechter, «ein sehr schlechter Fall» |
| 5. Perzentil | nur 5 % enden schlechter, «ein schlechter Fall» |
| 50. (Median) | die Hälfte endet besser, die Hälfte schlechter |
| 95. und 99. Perzentil | sehr gute Fälle |

Die Tabelle nennt zu jeder Zeile den **Lauf**, zum Beispiel den historischen Startmonat oder die Nummer des Pfades. Ein **negatives** Endvermögen bedeutet, dass das Geld **vor dem Ende ausging** und danach Schulden (offene Ausgaben) aufliefen.

**Ordnung nach der Darstellung:** Die Läufe werden nach dem Endvermögen der gewählten Darstellung geordnet: in **heutiger Kaufkraft** nach dem realen, **nominal** nach dem nominalen Endvermögen. Die Inflation ist je Lauf verschieden; ein Lauf mit hoher Inflation endet nominal hoch, real aber tief. Ein Perzentil kann deshalb je Darstellung zu einem **anderen Lauf** gehören, die Werte steigen aber in beiden Darstellungen mit dem Perzentil. Kennzahlen, Perzentil-Tabelle, Immobilienverkäufe, [Einzelne Läufe](#r-runs) und Jahreswerte zeigen immer die Läufe der gewählten Ordnung; beim Umschalten wird neu geordnet. Ältere gespeicherte Ergebnisse ohne diese Ordnung zeigen in heutiger Kaufkraft das reale Endvermögen der nominal geordneten Läufe und einen Hinweis; neu berechnet sind sie real geordnet.

> Der **Median** ist **kein** realistischer Normalfall im Sinn einer Prognose. Er sagt nur, dass die Hälfte der simulierten Zukünfte besser ausgeht. Für die Planung zählen die **unteren** Perzentile, weil dort das Risiko liegt.

## @r-fan Bandbreite des Gesamtvermögens
Stichworte: Fan, Band, Perzentile, Bandbreite, schlechtester Wert, Kegel, Median

Die Grafik zeigt, wie sich das **Gesamtvermögen** über die Zeit **verteilt**. In jedem Monat sind über alle Läufe gebildet:

- die **dunkle Fläche**: 25. bis 75. Perzentil (die mittlere Hälfte der Läufe)
- die **helle Fläche**: 5. bis 95. Perzentil (neun von zehn Läufen)
- die **durchgezogene Linie**: Median
- die **gestrichelten Linien**: 1. und 99. Perzentil (in der Legende mit gestricheltem Linien-Symbol)
- die **rote Linie**: der **schlechteste Wert** des Monats

Die rote Linie ist pro Monat der tiefste Wert aller Läufe. Das kann von Monat zu Monat ein anderer Lauf sein, sie ist also kein einzelner Pfad, sondern die untere Grenze aller Pfade. Mit der Maus sehen Sie die Werte im Tooltip.

**Lesen:** Die Bandbreite öffnet sich über die Zeit wie ein Trichter. Je weiter unten das untere Band liegt, desto grösser die Gefahr. Berührt das 5. Perzentil die Nulllinie, geht in mehr als 5 % der Läufe das Geld aus.

## @r-survival Wahrscheinlichkeit, dass das Geld reicht
Stichworte: Überleben, Erfolgswahrscheinlichkeit, Erschöpfung, Verlauf

Der Anteil der Läufe, in denen das Vermögen **bis zu diesem Zeitpunkt** noch nicht aufgebraucht ist. Die Kurve fällt nur, nie steigt sie. Am rechten Rand steht die Erfolgsquote.

Eine Kurve, die **früh** fällt, deutet auf Risiko schon in den ersten Jahren der Pensionierung hin (Sequenzrisiko). Eine Kurve, die erst spät fällt, ist ein Langlebigkeitsrisiko. Die Achse zeigt standardmässig **80–100 %**, wo die meisten Kurven liegen und Unterschiede sichtbar werden; beachten Sie die Skala. Der Schalter über der Grafik wechselt auf **0–100 %**. Die Wahl gilt im Einzelergebnis und im [Vergleich](#r-compare), bis Sie sie ändern. Fällt eine Kurve unter 80 %, ist sie bei 80–100 % unten abgeschnitten, und ein Hinweis unter der Grafik nennt den tiefsten Wert. Der Tooltip zeigt immer den echten Wert.

## @r-hist Verteilung des Endvermögens
Stichworte: Histogramm, Verteilung, Häufigkeit, Marken, Pensionierung

Ein **Histogramm**: Jede Säule zählt die Läufe, deren Endvermögen in diese Klasse fällt. Die gestrichelten Marken zeigen das 1., 5., 50., 95. und 99. Perzentil (Legende unter dem Diagramm). Der Bereich reicht vom 1. bis zum 99. Perzentil, schlechtere bzw. bessere Läufe stecken in den Randklassen. Der schlechteste Lauf steht darunter in der Anmerkung.

Typisch ist eine **schiefe** Verteilung mit langem rechten Ausläufer: Wenige Läufe enden sehr gut, die meisten dazwischen. Das untere Ende zeigt, wie schlimm es werden kann.

Die zweite Grafik zeigt die Verteilung des **Vermögens bei Pensionierung**. Sie ist die Ausgangslage für die Entnahmephase: Ein tiefes Vermögen bei Pensionierung ist oft eine Vorentscheidung für das Ergebnis.

## @r-depletion Alter bei Erschöpfung
Stichworte: Erschöpfung, aufgebraucht, Alter, Balken

Pro Altersjahr die Anzahl Läufe, in denen das Vermögen **in diesem Jahr** aufgebraucht ist. Die Grafik erscheint nur, wenn es überhaupt Erschöpfungen gibt.

Nützlich, um das Risiko im Alter einzuordnen: Geht das Geld nur selten und erst mit über 90 aus, ist das ein anderes Risiko, als wenn es mit 75 passiert.

## @r-charts Grafiken
Stichworte: Diagramme, Tab, Grafik

Die Registerkarte «Grafiken» zeigt die Diagramme in voller Grösse:

- Bei **einem Lauf:** [Vermögensverlauf](#r-wealth), [Liquides Vermögen nach Anlage](#r-comp), [Einnahmen und Ausgaben](#r-flows), [Entnahme aus dem Vermögen](#r-draw), [Immobilien](#r-prop), [Vorsorge](#r-pk) und [Aktien und Obligationen](#r-markets).
- Bei **mehreren Läufen:** [Bandbreite](#r-fan), [Wahrscheinlichkeit, dass das Geld reicht](#r-survival), [Verteilung des Endvermögens](#r-hist) und [Alter bei Erschöpfung](#r-depletion).

Jede Grafik hat ein ⓘ an der Überschrift. Mit **F1** über einer Grafik erhalten Sie die passende Erklärung.

**Dicke Linie bei 0:** In allen Grafiken mit einer Wertachse, die 0 enthält, ist die Linie bei 0 dicker als das übrige Raster, auch im Vergleich und in der Minigrafik im Editor. Unter dieser Linie ist das Geld aufgebraucht. In der [Verteilung des Endvermögens](#r-hist) steht sie senkrecht bei einem Endvermögen von 0. Liegt 0 ausserhalb des gezeigten Bereichs, fehlt die Linie.

## @r-runs Einzelne Läufe
Stichworte: Lauf, Detail, Perzentillauf, schlechtester Lauf, Pfad, Einzellauf

Bei mehreren Läufen können Sie **einzelne Läufe** im Detail ansehen. Wählen Sie oben, welchen:

- **Schlechtester Lauf**, **1.**, **5.**, **50.**, **95.** oder **99. Perzentil**: der Lauf, dessen **Endvermögen** diesem Perzentil entspricht, in der Ordnung der gewählten Darstellung (siehe [Endvermögen nach Perzentil](#r-percentiles)).

Darunter sehen Sie die Grafiken dieses Laufs, die ein Einzellauf zeigt, und die Ereignisliste. So erkennen Sie, **warum** ein Lauf schlecht endet: Börseneinbruch am Anfang, hohe Inflation, ein Notverkauf der Immobilie.

Besonders lehrreich ist der **Vergleich** des 5. mit dem 95. Perzentil: Wie unterscheiden sich die Renditen der ersten zehn Pensionierungsjahre? Das ist das **Sequenzrisiko** in Zahlen. Die Detailläufe rechnet die App exakt nach.

## @r-wealth Vermögensverlauf
Stichworte: Gesamtvermögen, gestapelt, liquid, Immobilien, Vorsorge

Das **Gesamtvermögen** eines Laufs als gestapelte Fläche aus drei Teilen:

- **Liquides Vermögen** (Aktien, Obligationen, Cash)
- **Immobilien (netto)**, also Marktwert minus Hypothek
- **Vorsorge** (Pensionskasse, Säule 3a)

Senkrechte Linien markieren Ereignisse (Pensionierung, AHV-Beginn, Kauf, Verkauf, Erschöpfung). Der Mauszeiger zeigt die Werte des Monats. **Sprünge** haben meist einen Grund: Der Kapitalbezug der Pensionskasse verschiebt Vorsorge in liquides Vermögen, ein Hausverkauf verschiebt Immobilien in liquides Vermögen.

## @r-comp Liquides Vermögen nach Anlage
Stichworte: Aktien, Obligationen, Cash, Anteile, Zusammensetzung

Wie sich das **liquide Vermögen** auf Aktien, Obligationen und Cash verteilt. Mit den Knöpfen oben zeigen Sie **Beträge** oder **Anteile in %**.

Der Anteil zeigt, ob die Strategie so wirkt, wie Sie es erwarten. Bei der [3-Topf-Strategie](#f-wd-three_buckets) sehen Sie die Töpfe, bei «Standard» den festen Aktienanteil und die Reserve.

## @r-flows Einnahmen und Ausgaben pro Jahr
Stichworte: Einnahmen, Ausgaben, Steuern, Wohnkosten, Lebenshaltung, Säulen

Pro **Kalenderjahr** zwei Säulen:

- **links die Einnahmen:** Erwerbseinkommen, PK-Rente, AHV
- **rechts die Ausgaben:** Lebenshaltung, Wohnkosten, geschätzte Steuern

Wo die rechte Säule höher ist als die linke, wird **das Vermögen angezapft**. Das erste und das letzte Jahr sind unvollständig, weil die Simulation mitten im Jahr beginnt oder endet.

Die Grafik zeigt gut, wie die Rentenbezüge ab Pensionierung die Lücke schliessen und wie gross sie noch ist.

## @r-draw Entnahme aus dem Vermögen
Stichworte: Entnahmerate, nachhaltig, 4 Prozent, Faustregel, Entnahme

Pro Jahr: **Ausgaben inklusive Steuern minus Einnahmen**, im Verhältnis zum liquiden Vermögen. Die Linie zeigt die **Entnahmerate**.

Eine Faustregel besagt, dass eine Rate **unter 4 %** nachhaltig sei. Das gilt nur grob und für US-Verhältnisse über 30 Jahre. Steigt die Rate im Lauf der Jahre deutlich, weil das Vermögen schrumpft oder die Ausgaben wachsen, ist das ein Warnzeichen. Bei den dem Vermögen folgenden Strategien (fixe Rate, VPW, …) sehen Sie hier ihre Wirkung.

## @r-prop Immobilien
Stichworte: Marktwert, Hypothek, Eigenkapital, Haus

Die Immobilien eines Laufs: **Marktwert**, **Hypothek** und **Eigenkapital** (Marktwert minus Hypothek). Die Grafik erscheint nur, wenn Immobilien vorkommen. Sprünge zeigen Kauf, Verkauf oder Rückzahlung einer Hypothek.

## @r-pk Vorsorge
Stichworte: Pensionskasse, Säule 3a, Guthaben

Verlauf von **Pensionskasse** und **Säule 3a** eines Laufs. Die Pensionskasse wächst durch Beiträge und Verzinsung bis zum Bezug und fällt dann auf den Rentenanteil. Die 3a ist nur bei indirekter Amortisation sichtbar. Die Grafik erscheint nur, wenn Vorsorgeguthaben vorkommen.

## @r-markets Aktien und Obligationen
Stichworte: Index, Rendite, Verlauf, Marktverlauf, Preisindex, Teuerung, Inflation, logarithmisch, Prozent

Zeigt den **Renditeverlauf dieses Laufs** als Index, **Start = 100**, mit wiederangelegten Ausschüttungen. Die **Inflation** (Preisindex) läuft wie Aktien und Obligationen mit, gestrichelt. Im Tooltip steht zusätzlich die Änderung der letzten zwölf Monate, bei der Inflation in Klammern.

Zwei Schalter ändern die Darstellung: **Wertentwicklung (Start = 100)** oder **Änderung in %** (gegenüber dem Vorjahresmonat). Bei der Wertentwicklung wählen Sie **Linear** oder **Logarithmisch**; logarithmisch zeigen gleiche Abstände gleiche prozentuale Änderungen. Die Prozentansicht ist immer linear. Die dicke Linie liegt bei der Wertentwicklung bei **100** (Stand beim Start), bei der Änderung in % bei **0 %**.

So erkennen Sie, **was in diesem Lauf am Markt passiert ist**: ein Einbruch zum Start, eine lange Seitwärtsphase oder eine hohe Inflation. Das erklärt oft, warum ein Lauf schlecht oder gut endet.

## @r-params Parameter
Stichworte: Parameter, Zusammenfassung, Eingaben, Stand des Laufs, reproduzieren

Der Reiter **«Parameter»** zeigt **schreibgeschützt**, mit welchen Eingaben das Ergebnis berechnet wurde: Haushalt (Geburtsmonat, keine Namen), Meilensteine, Vorsorge, Einnahmen und Ausgaben, Vermögen und Anlage, Renditequelle und Zeitreihe (mit Zufallsstartwert), Entnahmestrategie samt Parametern, Immobilien, Steuerannahmen und Variationen. Es gilt der **Stand des Laufs**, nicht der aktuelle Stand im Editor: Wenn Sie nach dem Rechnen eine Zahl ändern, bleibt der Reiter des berechneten Ergebnisses unverändert. Bei gespeicherten Ergebnissen steht die Planung von damals; fehlt sie, steht «Nicht verfügbar».

## @r-events Ereignisse
Stichworte: Liste, Kauf, Verkauf, Notverkauf, Pensionierung, Erschöpfung

Die Liste aller **Ereignisse** eines Laufs mit Datum und Alter: Pensionierung, AHV-Beginn, Kapitalbezug mit Kapitalsteuer, Käufe und Verkäufe von Immobilien (auch **Notverkäufe**), Hypothekarperioden, der **Wechsel von indirekter auf direkte Amortisation** im AHV-Bezug, die **Einmalrückzahlung** beim Kauf, wenn die Hypothek für die Tragbarkeit gekürzt werden muss, Auszahlung der 3a, einmalige Beträge und die **Erschöpfung** des Vermögens. Die **Buchungen der 3-Topf-Strategie** (Auffüllungen, Zuflüsse, Reservierungen von Verkaufserlösen) stehen nicht in dieser Liste, sondern aufklappbar in den [Jahreswerten](#r-table); die Meldungen der Defizitphase («alle Töpfe leer») bleiben hier.

Alle Beträge stehen **mit Währung** (Projektwährung).

Das **Entnahme-Ereignis** bei Strategien wie Guardrails zeigt die Kürzungen und Erhöhungen. Notverkäufe und Erschöpfungen sind Signale, dass die Planung in diesem Lauf nicht trägt.

## @r-property Immobilien (Ereignisse)
Stichworte: Kauf, Verkauf, Notverkauf, Hypothekenerneuerung, Einmalzahlung, Tragbarkeit, Zinssatz, Eigenmittel

In der Übersicht eines Einzellaufs (und bei «Einzelne Läufe») listet der Abschnitt **«Immobilien»** alle Immobilienereignisse in zeitlicher Reihenfolge, mit Zeitpunkt (Jahr und Alter), Ereignis, Immobilie und den Beträgen:

- **Kauf:** Kaufpreis, Eigenmittel (Preis − Hypothek) und Hypothek.
- **Verkauf / Notverkauf:** Verkaufserlös, abgelöste Hypothek, Grundstückgewinnsteuer und **Nettoerlös**; beim Notverkauf der Grund («Vermögen < 0» oder «Notverkaufskriterium»), dazu ein Hinweis, ob der Erlös für einen Kauf im Folgemonat reserviert wurde oder die [Entnahmestrategie](#s-withdrawal) sofort angewendet wurde.
- **Hypothekenerneuerung:** Tragbarkeit, Hypothek vorher und nachher, Einmalzahlung zur Herstellung der Tragbarkeit und Zinssatz der neuen Periode; wird die Hypothek nicht erneuert, steht der Notverkauf mit Grund.

Ohne Ereignisse steht dort «Keine Immobilienereignisse». Bei mehreren Läufen zeigt die Kennzahl **«Notverkäufe»** der Übersicht, in wie vielen Läufen (Anzahl und Anteil, davon wegen fehlender Tragbarkeit) Immobilien ungeplant verkauft wurden. Ist [Immobilien nie verkaufen](#f-re-never-sell) eingeschaltet, steht daneben die Kennzahl **«Nicht tragbare Erneuerungen»**: in wie vielen Läufen (Anzahl und Anteil) eine Hypothek bei der Erneuerung nicht tragbar war und nur dank des Schalters voll erneuert wurde. Eine bezahlte Einmalzahlung zählt nicht dazu. Die Spalte **«Immobilienverkäufe»** der Perzentil-Tabelle nennt für den Lauf der jeweiligen Zeile **jeden Verkauf einer Immobilie** mit Monat («Haus: 05.2034», darunter «Wohnung: 03.2048»); Notverkäufe tragen das Warnsymbol (Haus mit Warnzeichen), geplante Verkäufe nicht, ohne Verkauf steht «–».

## @r-scenarios Szenarien
Stichworte: Variation, Heatmap, Tabelle, Parameter, Erfolgsquote je Variante

Die Szenarien-Registerkarte zeigt die Wirkung der [Variationen](#f-sweep): Jede Zeile der Tabelle ist eine **Kombination** von Parameterwerten, mit denen **alle Renditepfade** gerechnet wurden. Die Spalten zeigen Erfolgsquote, Vermögen bei Pensionierung (Median), Endvermögen (5., 50., 95. Perzentil) und die früheste Erschöpfung.

- Mit **einem** Parameter zeigen zwei Grafiken Erfolgsquote und Endvermögen je Wert.
- Mit **zwei** Parametern entsteht eine **Heatmap**: Farbe zeigt die Erfolgsquote, so finden Sie den sicheren Bereich auf einen Blick.

Über den Grafiken und in jedem Grafiktitel steht, **welche Variation** dargestellt wird.

**Einzellauf mit Variationen:** Rechnen Sie mit einer Quelle ohne Zufall (zum Beispiel fixe Renditen) und Variationen, erscheint oberhalb der Registerkarten **«Variation wählen»**. Dort wählen Sie je Variation eine Stufe (oder den Planwert); Grafiken, Kennzahlen und Jahreswerte zeigen dann den Lauf dieser Stufe. «Optimum» nennt den besten Wert nach denselben Kriterien wie bei mehreren Läufen (unten), **«Optimum anzeigen»** wählt den entsprechenden Schritt. Perzentile gibt es beim Einzellauf nicht, das Optimum gilt für den einen Lauf.

**Stufe bei Monte Carlo und historischen Läufen:** Mit mehreren Läufen und Variationen erscheint in **«Einzelne Läufe»** und **«Jahreswerte»** oberhalb ein Auswahlfeld pro Variation mit den berechneten Stufen (zum Beispiel «Aktienanteil 30 %», «Kapitalbezug 60 %») und «Planwert». Die beiden Registerkarten zeigen dann die Läufe dieser Stufe, also dieselben Läufe (schlechtester, 1., 5., 50., 95., 99. Perzentil, in heutiger Kaufkraft in der realen Ordnung), auf die sich die Tabelle unter «Szenarien» stützt. Die App rechnet dafür nur diese sechs Läufe nach, nicht alle Pfade; der Wechsel dauert wenige Sekunden. Die Auswahl gilt für beide Registerkarten gemeinsam. Die übrigen Registerkarten zeigen weiter die Planwerte und tragen den Vermerk «Planwerte, nicht die gewählte Variation». Bei gespeicherten Ergebnissen funktioniert die Auswahl ebenso. Weicht ein nachgerechnetes Endvermögen von der Tabelle ab (zum Beispiel, weil sich eine Zeitreihe geändert hat), erscheint ein Hinweis. Eine Stufe, die nicht berechnet wurde (Planwert nicht im Raster), lässt sich nicht wählen.

**Optimaler Wert je Perzentil:** In der Übersicht (Endvermögen nach Perzentil) und bei «Einzelne Läufe» steht zu jedem Perzentil der Wert der Variation, der für dieses Perzentil optimal ist. Dafür werden nur die bereits gerechneten Variationsschritte verwendet.

- **Ausgaben-Faktor:** der **höchste** Faktor, bei dem das Vermögen bis zum Ende reicht. Liegt der Übergang zwischen zwei Schritten, wird dazwischen interpoliert, bis das Endvermögen genau 0 ist (Anzeige «≈ 73 %»).
- **Aktienanteil, Kapitalbezug Pensionskasse:** Vorrang haben Werte, bei denen das Vermögen reicht; darunter gilt das **höchste Endvermögen**. Reicht es nirgends, gilt der Wert mit dem spätesten Alter der Erschöpfung («(reicht nirgends)»). Hier wird nicht interpoliert.
- **Zwei Variationen:** Ist der Ausgaben-Faktor dabei, wird für jeden Wert des anderen Parameters der höchste tragbare Faktor bestimmt und die Kombination mit dem höchsten Faktor gezeigt. Sonst gilt die beste Kombination nach den Kriterien oben.
- **Rand des Bereichs:** «< 50 %» heisst, dass schon der tiefste untersuchte Wert nicht reicht; «≥ 110 %» (bzw. «≥» / «≤» beim besten Wert), dass der Bereich nicht ausreicht. Erweitern Sie dann die Variation. Bei einem Anteil (Aktienanteil, Kapitalbezug Pensionskasse) steht 0 % oder 100 % ohne «≥» bzw. «≤», denn das ist die natürliche Grenze, nicht ein zu knapper Bereich; liegt der beste Wert eines Anteils an einem anderen Rand (zum Beispiel 50 % bei einem Bereich von 0 bis 50 %), gilt «≥» bzw. «≤» weiter.
- **Historischer Startmonat, Pensionierungsdatum, Entnahmerate:** Hier ist kein Optimum sinnvoll (der Startmonat ist keine Entscheidung; ein späteres Pensionierungsdatum bzw. eine tiefere Rate ergibt immer das höchste Endvermögen). Angezeigt wird «–».

Alle Varianten verwenden **dieselben Zufallspfade**. Unterschiede stammen daher vom Parameter, nicht vom Zufall.

## @r-table Jahreswerte
Stichworte: Tabelle, Jahr, Export, Zahlen, Entnahme, Abweichung vom Plan, Steuern, Ausgaben, Miete, Hypothek, Perzentil, Kapitalbezug, Grundstückgewinn, steuerbares Einkommen

Eine Tabelle mit **einer Zeile pro Jahr** für den gewählten Lauf. Beträge in der Projektwährung, nominal oder in heutiger Kaufkraft je nach Schalter. Bei mehreren Läufen wählen Sie oben den Lauf (Schlechtester Lauf, 1., 5., 50., 95., 99. Perzentil); Standard ist der Median.

| Spalten | Bedeutung |
|---|---|
| Liquid, Immobilien, Vorsorge, Total | **Vermögen am Jahresende**: liquide Mittel, Immobilien (netto), Vorsorge (PK, 3a), Summe |
| Ausgaben total | **Monatsdurchschnitt des Jahres**: geplante Ausgaben (Lebenshaltung aus Ihrer Planung) plus **Miete bzw. Hypothekarzins**, Amortisation und übrige Wohnkosten der Immobilien plus **Einzahlung in die Säule 3a** |
| Heimkosten | Nur mit [Pflegeheim](#f-care-home): Monatsdurchschnitt der Heimkosten des Jahres, in «Ausgaben total» enthalten |
| Entnahme | Monatsdurchschnitt des Jahres: was dem Vermögen **entnommen** wird (tatsächliche Ausgaben minus Einkommen, vor Steuern) |
| Einkommen total | Monatsdurchschnitt des Jahres: Einkommen (Erwerbseinkommen und Renten) plus Entnahme |
| Entnahmerate | **jährliche Entnahme / Gesamtvermögen am Jahresanfang** in Prozent (Spalte «Total» des Vorjahres) |
| Abweichung vom Plan | Monatsdurchschnitt: Einkommen total minus Ausgaben total. Bei **0** verläuft alles wie geplant. Bei Strategien, die die Ausgaben an das Vermögen koppeln (Guardrails, Fixe Rate, VPW, Endowment), ist ein **negativer** Wert die Kürzung gegenüber Ihrem Plan, ein positiver die Erhöhung |
| Inflation pro Jahr | Veränderung des Preisindex im Kalenderjahr, im ersten Jahr seit Simulationsstart |
| Steuerbares Einkommen | erscheint mit den Steuerspalten **direkt vor den Steuern**: steuerbares Einkommen des **Steuerjahres** beim Kanton, also Einkommen (Erwerb netto, Pensionskassenrente, AHV, [steuerbarer Ertrag](#f-tax-yield)) minus die [Abzüge](#f-tax-deductions), nie negativ; Kapitalbezüge zählen nicht dazu. Weicht der Bund ab, zeigt der Tooltip der Zelle beide Werte. Damit lässt sich nachvollziehen, warum sich die Steuer bei gleichem «Einkommen total» ändert (Abzüge, Ertrag, Tarif, Wohnort). Jahre ohne Veranlagung (das letzte Jahr, wenn die Planung vor Dezember endet) zeigen 0 |
| Steuern bezahlt im Jahr | **Jahressummen der bezahlten Steuern** nach Art: Steuer Einkommen, Steuer Vermögen, Steuer Kapital Bezug, Steuer Grundstück, dazu das Total. Die Spalten sind **standardmässig eingeklappt**; die Schaltfläche **«Steuern anzeigen»** über der Tabelle blendet sie ein und aus, die Wahl bleibt erhalten. Die Spaltenköpfe heissen **«Steuer geschätzt …»**, ihr Tooltip nennt die Rundung auf 100 Franken |
| Steuerwert Immobilien | erscheint mit den Steuerspalten, wenn eine Immobilie einen [Steuerwert](#f-re-tax-value) hat: Summe der Steuerwerte am 31. Dezember (Immobilien ohne Steuerwert mit dem Marktwert); die Vermögenssteuer rechnet damit minus Hypothek |
| Abzüge Vermögen, Steuerbares Vermögen | erscheinen mit den Steuerspalten, wenn die Planung [Abzüge vom Vermögen](#f-tax-wealth-deductions) hat: Abzüge am 31. Dezember mit Herkunft im Tooltip und Vermögen minus Abzüge, nie unter 0 (im Modus «Wohnort» kommen die Sozialabzüge des Kantons dazu) |

Die Spalten sind nicht in Gruppen mit Überschrift geordnet. Dass die Vermögen **Stände am Jahresende** und die Geldflüsse **Monatsdurchschnitte** sind, steht im Hinweis (Tooltip) der Spaltenköpfe und im Text unter der Tabelle. Das Einkommen allein und die geplanten Ausgaben ohne Wohnkosten zeigt die Tabelle nicht; sie stecken in «Einkommen total» und «Ausgaben total».

- **Entnahmerate:** Die jährliche Entnahme ist die Summe der Monatsentnahmen des Kalenderjahrs (bei unvollständigen Jahren der Monatsdurchschnitt mal 12). Das Vermögen am Jahresanfang ist das Gesamtvermögen («Total») des Vorjahres, im ersten Jahr der Stand zum Simulationsstart. Gerechnet wird immer mit **nominalen** Werten, der Schalter Nominal/Heutige Kaufkraft ändert die Raten nicht. Ist die Entnahme null oder negativ (Einkommen über den Ausgaben), bleibt die Zelle leer; ist das Vermögen null oder negativ, steht «–».
- **3-Topf-Strategie:** Jedes Jahr mit mindestens einer Auffüllung hat vorn einen Pfeil (▸). Er klappt die **Herleitung der Topfgrössen und der Auffüllungen** dieses Jahres auf (Auslöser, Ausgabenbasis, regelmässige Einnahmen, Netto-Bedarf, Ziele, Renditekriterium, Umbuchungen, Topfstände vorher und nachher). Bei mehreren Läufen gilt sie für den gewählten Einzellauf.
- Die Geldflüsse sind **Monatsdurchschnitte** des Kalenderjahrs, weil einzelne Monate schwanken (Hypothekarzinsen werden zum Beispiel quartalsweise bezahlt).
- Einkommens- und Vermögenssteuer werden im Dezember veranlagt und im **Mai des Folgejahres** bezahlt. Die Tabelle zeigt sie in der Zeile des **Steuerjahres**, für das sie gelten: Die Steuer des Steuerjahres 2028 steht in der Zeile 2028, auch wenn das Geld erst im Mai 2029 abfliesst. Das Vermögen und der Cash-Bestand sinken deshalb erst im Folgejahr. Endet die Planung vor Dezember, fehlt die Steuer des letzten, unvollständigen Jahres in den Spalten. Kapitalbezugssteuer (Pensionskasse, Säule 3a) und Grundstückgewinnsteuer stehen im Jahr des Bezugs bzw. Verkaufs.
- Die Steuern sind **geschätzt** und auf **100 Franken gerundet** (jede Zelle für sich, das Total kann deshalb um wenige hundert Franken von der Summe der gerundeten Spalten abweichen). Unter der Tabelle steht «Geschätzte Steuer mit Ihren eigenen Tabellen, auf 100 Franken gerundet. Keine Steuerberatung.» Kapitalbezugs- und Grundstückgewinnsteuer vor dem Entscheid mit dem Steuerrechner des Kantons oder einer Fachperson prüfen, siehe [Wie genau ist die Steuer?](#tax-accuracy).
- Das erste und das letzte Jahr sind meist unvollständig; sie tragen einen Stern (*), der Mauszeiger nennt die Zahl der Monate.

## @r-list Liste der Ergebnisse
Stichworte: Verlauf, Liste, gespeichert, umbenennen, Notiz, löschen, Ergebnisse, Simulationen

Jede Simulation wird nach dem Start als Ergebnis **automatisch abgelegt**, mit der **Planung von damals** und dem **vollständigen Ergebnis**. Über die Schaltfläche **«Ergebnisse»** in der Kopfzeile öffnen Sie die Liste.

- **Ergebnis:** Bei Monte-Carlo-Läufen steht «**Erfolgsquote · 5 %: Betrag**» (5. Perzentil des Endvermögens); der Median erscheint beim Daraufzeigen. Einzelläufe zeigen das Endvermögen. Das **Warnsymbol (Haus mit Warnzeichen)** erscheint, wenn eine Immobilie ungeplant verkauft wurde (Notverkauf, auch wegen fehlender Tragbarkeit bei der Hypothekenerneuerung; geplante Verkäufe zählen nicht). Bei mehreren Läufen gilt es ab **5 % der Läufe** mit Notverkauf, der Tooltip nennt Anteil und Zahl der Läufe; bei einem Einzellauf bei jedem Notverkauf. Einträge aus älteren Versionen zeigen kein Symbol.
- **Zeitreihe:** Name der Zeitreihe, mit der das Ergebnis berechnet wurde (bei festen Renditen «feste Renditen», bei der parametrischen Quelle «parametrisch»). Einträge aus älteren Versionen zeigen «–». Die Angabe zur Art (Einzellauf, Szenarien, Anzahl Läufe) steht klein unter dem Ergebnis.
- **Strategie:** Bewegen Sie die Maus über den Namen (oder gehen Sie mit der Tab-Taste hin), erscheinen die **Parameter der Strategie** im Stand des Ergebnisses. Ältere Einträge zeigen «Parameter nicht verfügbar».
- **Name und Notiz:** Direkt in der Zeile anpassbar. Standardname: Projekt · Renditequelle · Strategie · Zeitpunkt, zum Beispiel «Beispiel · MC hist. · 3-Topf + Guard · 16.11.26, 14:05» (bei Szenarien steht «Szenarien» statt der Quelle). Abkürzungen der Renditequelle: **Fix** (fixe Renditen), **Hist.** (historisch ab Startzeitpunkt), **Alle Starts** (alle historischen Startzeitpunkte), **MC hist.** (Monte Carlo · historisch), **MC param.** (Monte Carlo · parametrisch). Abkürzungen der Strategie: **Standard**, **3-Topf**, **3-Topf + Guard** (3-Topf mit Guardrails), **Guardrails**, **Fixe Rate** (fixe Entnahmerate), **VPW**, **Endowment**. Die Zahl der Läufe steht klein unter dem Ergebnis. Ergebnisse aus älteren Versionen behalten ihren Namen.
- **Öffnen:** Zeigt das gespeicherte Ergebnis mit allen Registerkarten. Es gilt die **Planung von damals**, ein Hinweis oben nennt das Datum. **«Planung in den Editor übernehmen»** lädt diese Planung zurück in den Editor.
- **Löschen** entfernt ein Ergebnis nach Rückfrage, **«Markierte löschen»** mehrere und **«Alle löschen»** alle. Das Kästchen in der Kopfzeile markiert alle Einträge.
- **Optimierungen** und Strategievergleiche erscheinen ebenfalls in der Liste, siehe [Ergebnis speichern und wieder öffnen](#o-saved). «Vergleichen» zählt nur Simulationen und lässt markierte Optimierungen aus.
- **Vergleichen:** Markieren Sie zwei bis zehn Einträge und klicken Sie auf **«Vergleichen»**, siehe [Vergleich](#r-compare). Sind mehr als zehn markiert, ist «Vergleichen» gesperrt und ein Hinweis erscheint; Löschen geht weiterhin.

Die Liste liegt **lokal im Speicher der App**, nicht in der Planungsdatei. Sie wächst, bis Sie Einträge löschen: Ein Eintrag mit vielen Läufen belegt einige Megabyte. Ist der Speicher voll, erscheint die Meldung «Ergebnis nicht in der Liste gespeichert»; das Ergebnis bleibt angezeigt, wird aber nicht abgelegt. Löschen Sie dann ältere Einträge.

## @r-compare Ergebnisse vergleichen
Stichworte: Vergleich, nebeneinander, Unterschiede, Planung, Kennzahlen, Jahreswerte, Kaufkraft

Der Vergleich zeigt bis zu zehn gespeicherte Ergebnisse nebeneinander, jedes in einer eigenen Farbe. Breite Tabellen und Balkengrafiken lassen sich bei vielen Ergebnissen waagrecht scrollen. Er hat dieselben Möglichkeiten wie ein einzelnes Ergebnis, verteilt auf vier Registerkarten:

- **Übersicht:** Kennzahlen (Zeitreihe, Strategie, Erfolgsquote, Endvermögen am Median, am 5. und 1. Perzentil und im schlechtesten Lauf, früheste Erschöpfung; ein Klick auf einen Spaltenkopf sortiert die Zeilen aufsteigend, ein weiterer absteigend, der Pfeil ▲/▼ zeigt die Richtung; fehlende Werte stehen immer am Ende, «keine» Erschöpfung gilt als spätester Wert), die **Bandbreite des Gesamtvermögens** (Median durchgezogen, 5. Perzentil gestrichelt, 95. Perzentil gepunktet), das **Endvermögen nach Perzentil** als Tabelle, die **Wahrscheinlichkeit, dass das Geld reicht** (Achse 80–100 % oder 0–100 %, siehe [dort](#r-survival)), und das **Alter bei Erschöpfung** als Anteil der Läufe (so bleiben Ergebnisse mit verschieden vielen Läufen vergleichbar).
- **Verläufe:** die Grafiken des Einzelergebnisses für einen Lauf je Ergebnis: bei einem Einzellauf dieser Lauf, bei mehreren Läufen der Lauf des Perzentils, das Sie unter **«Lauf wählen»** einstellen (wie unter «Einzelne Läufe»; Standard: Median). Die Auswahl gilt für alle Ergebnisse und auch für die Jahreswerte; fehlt einem Ergebnis das Perzentil, zeigt es den Median-Lauf mit Vermerk. Gesamtvermögen, liquides Vermögen (wählbar: Total, Aktien, Obligationen, Cash), Einnahmen, Ausgaben oder Steuern pro Jahr, Entnahmerate, Immobilien (Eigenkapital), Vorsorge sowie Aktien, Obligationen und Inflation (Aktien durchgezogen, Obligationen gestrichelt, Inflation gepunktet). Diese Grafik hat dieselben Schalter wie im Einzelergebnis: **Wertentwicklung (Start = 100)**, linear oder logarithmisch, oder **Änderung in %** gegenüber dem Vorjahresmonat; die Wahl gilt in beiden Ansichten.
- **Jahreswerte:** für den gewählten Lauf eine Zeile je Jahr und eine Spalte je Ergebnis für eine wählbare Grösse (Gesamtvermögen, liquides Vermögen, Immobilien, Vorsorge, Einkommen, Ausgaben, Entnahme, Entnahmerate, Steuern). Die Werte sind dieselben wie in den [Jahreswerten](#r-table) der einzelnen Ergebnisse.
- **Planung:** eine Tabelle aller Eingaben, die sich zwischen den Ergebnissen unterscheiden, mit den Bezeichnungen und Einheiten des Editors (zum Beispiel «Aktienanteil: 30 %» statt eines internen Feldnamens) und gruppiert nach den Abschnitten des Editors. Einträge in Listen tragen ihren Namen (zum Beispiel «Ferienwohnung: Marktwert»). Zeitabhängige Werte erscheinen als Liste («ab 01.2030: …»). So sehen Sie genau, **was** zu einem anderen Ergebnis geführt hat.

Oben rechts wechseln Sie zwischen **Nominal** und **Heutiger Kaufkraft**; jedes Ergebnis wird mit seiner eigenen Inflation umgerechnet. Wie im Einzelergebnis sind die Läufe der Perzentile nach dem Endvermögen der gewählten Darstellung geordnet, in heutiger Kaufkraft also nach dem realen (siehe [Endvermögen nach Perzentil](#r-percentiles)); ältere Ergebnisse ohne diese Ordnung werden in einer Anmerkung genannt. Der Zoom gilt wie im Einzelergebnis für alle Grafiken mit Zeitachse.

Die Ergebnisse werden nach **Jahren seit ihrem Start** ausgerichtet. Beginnen sie in verschiedenen Kalenderjahren, zeigen die Jahreswerte «1. Jahr», «2. Jahr» usw., die Jahreszahl steht im Tooltip. Fehlt einem Ergebnis eine Grösse, fehlt es in dieser Grafik bzw. Spalte, und ein Vermerk nennt es: ein Einzellauf erscheint in der Bandbreite als eine Linie und hat keine Perzentile, keine Wahrscheinlichkeit und kein Alter bei Erschöpfung; ein Ergebnis ohne Immobilie oder ohne Vorsorgevermögen fehlt in diesen Grafiken; ein Ergebnis nur mit Szenarien erscheint nur als Kennzahl; ältere Ergebnisse ohne Werte in heutiger Kaufkraft fehlen in der Bandbreite; Ergebnisse aus Version 1.0.0-beta.1 und älter haben in heutiger Kaufkraft nur das Endvermögen am schlechtesten Lauf, am 1., 5., 50., 95. und 99. Perzentil.

Vergleichen Sie am besten Ergebnisse, die sich in **einem** Punkt unterscheiden (zum Beispiel Aktienanteil oder Strategie).
