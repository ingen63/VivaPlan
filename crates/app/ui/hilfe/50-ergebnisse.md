# Ergebnisse lesen

## @r-overview Ergebnisse: Übersicht
Stichworte: Erfolgsquote, Kennzahlen, Perzentile, Endvermögen, Erschöpfung, Ergebnisansicht

Nach «Simulation starten» zeigt die App einen **Fortschrittsbalken** mit vergangener und geschätzter Restzeit und einem Knopf **Abbrechen**. Danach öffnet sich die Ergebnisansicht mit mehreren Registerkarten: **Übersicht**, **Grafiken**, **Einzelne Läufe** (nur bei mehreren Läufen), **Szenarien** (nur mit Variationen) und **Jahreswerte**. Alle Grafiken mit Zeitachse lassen sich zoomen: **Mit gedrückter Maustaste einen Zeitbereich aufziehen** (er wird hervorgehoben, Auswahlen unter einem Jahr zählen als Klick und werden ignoriert), oder mit den Schaltflächen **Alles, 10 J., 5 J. und 2 J.** Die Schaltflächen setzen die Breite des Bereichs **zentriert auf die Mitte des aktuellen Bereichs** (ohne Zoom: der ganzen Laufzeit). Ragt der Bereich über Anfang oder Ende hinaus, wird er verschoben, die Breite bleibt; ist die Laufzeit kürzer, sehen Sie alles. Nach einer Mausauswahl ist keine Schaltfläche aktiv, die Grafiken zeigen «Benutzerdefiniert» mit dem Bereich. Der Zoom gilt **für alle Ergebnisgrafiken gemeinsam**, in allen Registerkarten und im Vergleich, und bleibt beim Wechsel der Registerkarte erhalten; die Minigrafiken im Editor haben ihren eigenen Zoom (ab Start). Die Grafiken der Szenarien haben keine Zeitachse und deshalb keinen Zoom. Oben rechts wechseln Sie zwischen **Nominal** und **Heutige Kaufkraft**, siehe [Rendite, Inflation und reale Werte](#ret-concepts).

**Ergebnis in Worten:** Zuoberst fasst die Übersicht das Ergebnis in zwei, drei Sätzen zusammen. Bei mehreren Läufen: «In 87 % (mittel) der Läufe reicht das Vermögen bis 12.2055 (Alter 95,0).» Die Einordnung in Klammern folgt den Farben der Erfolgsquote: **hoch** ab 95 %, **mittel** von 80 % bis unter 95 %, **tief** unter 80 %; massgebend ist der angezeigte, auf 0,1 % gerundete Wert. Geht in einem Lauf das Geld aus, nennt ein zweiter Satz das Alter im ungünstigsten Lauf (wie «Früheste Erschöpfung»). Bei einem Lauf steht, ob das Vermögen bis zum Ende reicht und wie gross es dann ist (nominal oder in heutiger Kaufkraft, je nach Umschalter), oder wann es aufgebraucht ist. Darunter folgt ein Vorschlag für den nächsten Schritt: unter 95 % bzw. bei aufgebrauchtem Vermögen die [Optimierung](#s-optimize), sonst die [Szenarien](#s-scenarios). Der Text ist eine Lesehilfe, keine Bewertung Ihrer Lage.

**Bei einem einzigen Lauf** (fixe Renditen, historischer Verlauf) zeigt die Übersicht fünf Kennzahlen: Vermögen bei Pensionierung, Vermögen am Ende, bis wann das Geld reicht, die PK-Rente und die gesamten Steuern. Darunter folgen der Vermögensverlauf und die Liste der Ereignisse.

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

Eine Kurve, die **früh** fällt, deutet auf Risiko schon in den ersten Jahren der Pensionierung hin (Sequenzrisiko). Eine Kurve, die erst spät fällt, ist ein Langlebigkeitsrisiko. Die Achse ist auf den relevanten Bereich gezoomt, beachten Sie die Skala.

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

## @r-runs Einzelne Läufe
Stichworte: Lauf, Detail, Perzentillauf, schlechtester Lauf, Pfad, Einzellauf

Bei mehreren Läufen können Sie **einzelne Läufe** im Detail ansehen. Wählen Sie oben, welchen:

- **Schlechtester Lauf**, **1.**, **5.**, **50.**, **95.** oder **99. Perzentil**: der Lauf, dessen **Endvermögen** diesem Perzentil entspricht.

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
- **rechts die Ausgaben:** Lebenshaltung, Wohnkosten, Steuern

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

Zwei Schalter ändern die Darstellung: **Wertentwicklung (Start = 100)** oder **Änderung in %** (gegenüber dem Vorjahresmonat). Bei der Wertentwicklung wählen Sie **Linear** oder **Logarithmisch**; logarithmisch zeigen gleiche Abstände gleiche prozentuale Änderungen. Die Prozentansicht ist immer linear.

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

Ohne Ereignisse steht dort «Keine Immobilienereignisse». Bei mehreren Läufen zeigt die Kennzahl **«Notverkäufe»** der Übersicht, in wie vielen Läufen (Anzahl und Anteil, davon wegen fehlender Tragbarkeit) Immobilien ungeplant verkauft wurden. Die Spalte **«Immobilienverkäufe»** der Perzentil-Tabelle nennt für den Lauf der jeweiligen Zeile **jeden Verkauf einer Immobilie** mit Monat («Haus: 05.2034», darunter «Wohnung: 03.2048»); Notverkäufe tragen das Warnsymbol (Haus mit Warnzeichen), geplante Verkäufe nicht, ohne Verkauf steht «–».

## @r-scenarios Szenarien
Stichworte: Variation, Heatmap, Tabelle, Parameter, Erfolgsquote je Variante

Die Szenarien-Registerkarte zeigt die Wirkung der [Variationen](#f-sweep): Jede Zeile der Tabelle ist eine **Kombination** von Parameterwerten, mit denen **alle Renditepfade** gerechnet wurden. Die Spalten zeigen Erfolgsquote, Vermögen bei Pensionierung (Median), Endvermögen (5., 50., 95. Perzentil) und die früheste Erschöpfung.

- Mit **einem** Parameter zeigen zwei Grafiken Erfolgsquote und Endvermögen je Wert.
- Mit **zwei** Parametern entsteht eine **Heatmap**: Farbe zeigt die Erfolgsquote, so finden Sie den sicheren Bereich auf einen Blick.

Über den Grafiken und in jedem Grafiktitel steht, **welche Variation** dargestellt wird.

**Einzellauf mit Variationen:** Rechnen Sie mit einer Quelle ohne Zufall (zum Beispiel fixe Renditen) und Variationen, erscheint oberhalb der Registerkarten **«Variation wählen»**. Dort wählen Sie je Variation eine Stufe (oder den Planwert); Grafiken, Kennzahlen und Jahreswerte zeigen dann den Lauf dieser Stufe. «Optimum» nennt den besten Wert nach denselben Kriterien wie bei mehreren Läufen (unten), **«Optimum anzeigen»** wählt den entsprechenden Schritt. Perzentile gibt es beim Einzellauf nicht, das Optimum gilt für den einen Lauf.

**Stufe bei Monte Carlo und historischen Läufen:** Mit mehreren Läufen und Variationen erscheint in **«Einzelne Läufe»** und **«Jahreswerte»** oberhalb ein Auswahlfeld pro Variation mit den berechneten Stufen (zum Beispiel «Aktienanteil 30 %», «Kapitalbezug 60 %») und «Planwert». Die beiden Registerkarten zeigen dann die Läufe dieser Stufe, also dieselben Läufe (schlechtester, 1., 5., 50., 95., 99. Perzentil), auf die sich die Tabelle unter «Szenarien» stützt. Die App rechnet dafür nur diese sechs Läufe nach, nicht alle Pfade; der Wechsel dauert wenige Sekunden. Die Auswahl gilt für beide Registerkarten gemeinsam. Die übrigen Registerkarten zeigen weiter die Planwerte und tragen den Vermerk «Planwerte, nicht die gewählte Variation». Bei gespeicherten Ergebnissen funktioniert die Auswahl ebenso. Weicht ein nachgerechnetes Endvermögen von der Tabelle ab (zum Beispiel, weil sich eine Zeitreihe geändert hat), erscheint ein Hinweis. Eine Stufe, die nicht berechnet wurde (Planwert nicht im Raster), lässt sich nicht wählen.

**Optimaler Wert je Perzentil:** In der Übersicht (Endvermögen nach Perzentil) und bei «Einzelne Läufe» steht zu jedem Perzentil der Wert der Variation, der für dieses Perzentil optimal ist. Dafür werden nur die bereits gerechneten Variationsschritte verwendet.

- **Ausgaben-Faktor:** der **höchste** Faktor, bei dem das Vermögen bis zum Ende reicht. Liegt der Übergang zwischen zwei Schritten, wird dazwischen interpoliert, bis das Endvermögen genau 0 ist (Anzeige «≈ 73 %»).
- **Aktienanteil, Kapitalbezug Pensionskasse:** Vorrang haben Werte, bei denen das Vermögen reicht; darunter gilt das **höchste Endvermögen**. Reicht es nirgends, gilt der Wert mit dem spätesten Alter der Erschöpfung («(reicht nirgends)»). Hier wird nicht interpoliert.
- **Zwei Variationen:** Ist der Ausgaben-Faktor dabei, wird für jeden Wert des anderen Parameters der höchste tragbare Faktor bestimmt und die Kombination mit dem höchsten Faktor gezeigt. Sonst gilt die beste Kombination nach den Kriterien oben.
- **Rand des Bereichs:** «< 50 %» heisst, dass schon der tiefste untersuchte Wert nicht reicht; «≥ 110 %» (bzw. «≥» / «≤» beim besten Wert), dass der Bereich nicht ausreicht. Erweitern Sie dann die Variation. Bei einem Anteil (Aktienanteil, Kapitalbezug Pensionskasse) steht 0 % oder 100 % ohne «≥» bzw. «≤», denn das ist die natürliche Grenze, nicht ein zu knapper Bereich; liegt der beste Wert eines Anteils an einem anderen Rand (zum Beispiel 50 % bei einem Bereich von 0 bis 50 %), gilt «≥» bzw. «≤» weiter.
- **Historischer Startmonat, Pensionierungsdatum, Entnahmerate:** Hier ist kein Optimum sinnvoll (der Startmonat ist keine Entscheidung; ein späteres Pensionierungsdatum bzw. eine tiefere Rate ergibt immer das höchste Endvermögen). Angezeigt wird «–».

Alle Varianten verwenden **dieselben Zufallspfade**. Unterschiede stammen daher vom Parameter, nicht vom Zufall.

## @r-table Jahreswerte
Stichworte: Tabelle, Jahr, Export, Zahlen, Entnahme, Abweichung vom Plan, Steuern, Ausgaben, Miete, Hypothek, Perzentil, Kapitalbezug, Grundstückgewinn

Eine Tabelle mit **einer Zeile pro Jahr** für den gewählten Lauf. Beträge in der Projektwährung, nominal oder in heutiger Kaufkraft je nach Schalter. Bei mehreren Läufen wählen Sie oben den Lauf (Schlechtester Lauf, 1., 5., 50., 95., 99. Perzentil); Standard ist der Median.

| Spalten | Bedeutung |
|---|---|
| Liquid, Immobilien, Vorsorge, Total | **Vermögen am Jahresende**: liquide Mittel, Immobilien (netto), Vorsorge (PK, 3a), Summe |
| Ausgaben total | **Monatsdurchschnitt des Jahres**: geplante Ausgaben (Lebenshaltung aus Ihrer Planung) plus **Miete bzw. Hypothekarzins**, Amortisation und übrige Wohnkosten der Immobilien plus **Einzahlung in die Säule 3a** |
| Entnahme | Monatsdurchschnitt des Jahres: was dem Vermögen **entnommen** wird (tatsächliche Ausgaben minus Einkommen, vor Steuern) |
| Einkommen total | Monatsdurchschnitt des Jahres: Einkommen (Erwerbseinkommen und Renten) plus Entnahme |
| Entnahmerate | **jährliche Entnahme / Gesamtvermögen am Jahresanfang** in Prozent (Spalte «Total» des Vorjahres) |
| Abweichung vom Plan | Monatsdurchschnitt: Einkommen total minus Ausgaben total. Bei **0** verläuft alles wie geplant. Bei Strategien, die die Ausgaben an das Vermögen koppeln (Guardrails, Fixe Rate, VPW, Endowment), ist ein **negativer** Wert die Kürzung gegenüber Ihrem Plan, ein positiver die Erhöhung |
| Inflation pro Jahr | Veränderung des Preisindex im Kalenderjahr, im ersten Jahr seit Simulationsstart |
| Steuern bezahlt im Jahr | **Jahressummen der bezahlten Steuern** nach Art: Steuer Einkommen, Steuer Vermögen, Steuer Kapital Bezug, Steuer Grundstück, dazu das Total. Die Spalten sind **standardmässig eingeklappt**; die Schaltfläche **«Steuern anzeigen»** über der Tabelle blendet sie ein und aus, die Wahl bleibt erhalten |

Die Spalten sind nicht in Gruppen mit Überschrift geordnet. Dass die Vermögen **Stände am Jahresende** und die Geldflüsse **Monatsdurchschnitte** sind, steht im Hinweis (Tooltip) der Spaltenköpfe und im Text unter der Tabelle. Das Einkommen allein und die geplanten Ausgaben ohne Wohnkosten zeigt die Tabelle nicht; sie stecken in «Einkommen total» und «Ausgaben total».

- **Entnahmerate:** Die jährliche Entnahme ist die Summe der Monatsentnahmen des Kalenderjahrs (bei unvollständigen Jahren der Monatsdurchschnitt mal 12). Das Vermögen am Jahresanfang ist das Gesamtvermögen («Total») des Vorjahres, im ersten Jahr der Stand zum Simulationsstart. Gerechnet wird immer mit **nominalen** Werten, der Schalter Nominal/Heutige Kaufkraft ändert die Raten nicht. Ist die Entnahme null oder negativ (Einkommen über den Ausgaben), bleibt die Zelle leer; ist das Vermögen null oder negativ, steht «–».
- **3-Topf-Strategie:** Jedes Jahr mit mindestens einer Auffüllung hat vorn einen Pfeil (▸). Er klappt die **Herleitung der Topfgrössen und der Auffüllungen** dieses Jahres auf (Auslöser, Ausgabenbasis, regelmässige Einnahmen, Netto-Bedarf, Ziele, Renditekriterium, Umbuchungen, Topfstände vorher und nachher). Bei mehreren Läufen gilt sie für den gewählten Einzellauf.
- Die Geldflüsse sind **Monatsdurchschnitte** des Kalenderjahrs, weil einzelne Monate schwanken (Hypothekarzinsen werden zum Beispiel quartalsweise bezahlt).
- Einkommens- und Vermögenssteuer werden im Dezember veranlagt und im **Mai des Folgejahres** bezahlt; sie stehen im Jahr der Zahlung. Die Steuer in der Zeile 2030 gehört also zum **Steuerjahr 2029** und ist aus dem Einkommen derselben Zeile nicht nachrechenbar: Ändert sich das Einkommen, ändern sich die Steuern ein Jahr später (zum Beispiel nach der Pensionierung). Kapitalbezugssteuer (Pensionskasse, Säule 3a) und Grundstückgewinnsteuer stehen im Jahr des Bezugs bzw. Verkaufs.
- Das erste und das letzte Jahr sind meist unvollständig; sie tragen einen Stern (*), der Mauszeiger nennt die Zahl der Monate.

## @r-list Liste der Ergebnisse
Stichworte: Verlauf, Liste, gespeichert, umbenennen, Notiz, löschen, Ergebnisse, Simulationen

Jede Simulation wird nach dem Start als Ergebnis **automatisch abgelegt**, mit der **Planung von damals** und dem **vollständigen Ergebnis**. Über die Schaltfläche **«Ergebnisse»** in der Kopfzeile öffnen Sie die Liste.

- **Ergebnis:** Bei Monte-Carlo-Läufen steht «**Erfolgsquote · 5 %: Betrag**» (5. Perzentil des Endvermögens); der Median erscheint beim Daraufzeigen. Einzelläufe zeigen das Endvermögen. Das **Warnsymbol (Haus mit Warnzeichen)** erscheint, wenn eine Immobilie ungeplant verkauft wurde (Notverkauf, auch wegen fehlender Tragbarkeit bei der Hypothekenerneuerung; geplante Verkäufe zählen nicht). Bei mehreren Läufen gilt es ab **5 % der Läufe** mit Notverkauf, der Tooltip nennt Anteil und Zahl der Läufe; bei einem Einzellauf bei jedem Notverkauf. Einträge aus älteren Versionen zeigen kein Symbol.
- **Zeitreihe:** Name der Zeitreihe, mit der das Ergebnis berechnet wurde (bei festen Renditen «feste Renditen», bei der parametrischen Quelle «parametrisch»). Einträge aus älteren Versionen zeigen «–». Die Angabe zur Art (Einzellauf, Szenarien, Anzahl Läufe) steht klein unter dem Ergebnis.
- **Strategie:** Bewegen Sie die Maus über den Namen (oder gehen Sie mit der Tab-Taste hin), erscheinen die **Parameter der Strategie** im Stand des Ergebnisses. Ältere Einträge zeigen «Parameter nicht verfügbar».
- **Name und Notiz:** Direkt in der Zeile anpassbar. Standardname: Projekt, Art des Ergebnisses und Zeitpunkt.
- **Öffnen:** Zeigt das gespeicherte Ergebnis mit allen Registerkarten. Es gilt die **Planung von damals**, ein Hinweis oben nennt das Datum. **«Planung in den Editor übernehmen»** lädt diese Planung zurück in den Editor.
- **Löschen** entfernt ein Ergebnis nach Rückfrage, **«Markierte löschen»** mehrere und **«Alle löschen»** alle. Das Kästchen in der Kopfzeile markiert alle Einträge.
- **Vergleichen:** Markieren Sie zwei bis sechs Einträge und klicken Sie auf **«Vergleichen»**, siehe [Vergleich](#r-compare).

Die Liste liegt **lokal im Speicher der App**, nicht in der Planungsdatei. Sie wächst, bis Sie Einträge löschen: Ein Eintrag mit vielen Läufen belegt einige Megabyte.

## @r-compare Ergebnisse vergleichen
Stichworte: Vergleich, nebeneinander, Unterschiede, Planung, Kennzahlen

Der Vergleich zeigt bis zu sechs gespeicherte Ergebnisse nebeneinander:

- **Kennzahlen:** Zeitreihe, Strategie, Erfolgsquote, Endvermögen am Median, am 5. und 1. Perzentil und im schlechtesten Lauf, früheste Erschöpfung.
- **Gesamtvermögen im Vergleich:** Median (durchgezogen) und 5. Perzentil (gestrichelt) jedes Ergebnisses über die Jahre seit dem Start. Jedes Ergebnis hat eine eigene Farbe.
- **Unterschiede in der Planung:** Eine Tabelle aller Eingaben, die sich zwischen den Ergebnissen unterscheiden. So sehen Sie genau, **was** zu einem anderen Ergebnis geführt hat.

Vergleichen Sie am besten Ergebnisse, die sich in **einem** Punkt unterscheiden (zum Beispiel Aktienanteil oder Strategie). Der Vergleich zeigt nominale Werte. Ergebnisse mit Szenarien erscheinen nur als Kennzahl.
