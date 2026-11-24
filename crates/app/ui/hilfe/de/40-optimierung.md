# Optimierung

## @s-optimize Optimierung: Überblick
Stichworte: beste Strategie, Parameter, Ruin, Konsum, Suche, Ziel

Die Optimierung beantwortet die Frage: **Welche Entnahmestrategie mit welchen Einstellungen passt am besten zu meinem Basisszenario?**

Sie nimmt Ihre Planung als gegeben und sucht für jede gewählte [Entnahmestrategie](#s-withdrawal) die Parameter, die drei Dinge gleichzeitig erreichen:

1. Die **Ruinwahrscheinlichkeit** bleibt unter Ihrer Grenze (zum Beispiel 5 %).
2. Der **Konsum** ist möglichst hoch (Median und schlechtes Jahr, real).
3. Der Konsum **schwankt** von Jahr zu Jahr möglichst wenig.

Die Strategien werden danach verglichen. Die beste steht oben und trägt einen Stern ★. Mit einem Klick auf «In Planung übernehmen» übernehmen Sie Strategie und Parameter in Ihre Planung.

**Voraussetzungen:**

- Eine Renditequelle mit **Streuung**: Monte Carlo (Bootstrap oder parametrisch) oder «Alle historischen Startzeitpunkte». Bei «Fixe Renditen» und «Historisch ab Startzeitpunkt» gibt es nur einen Pfad, der Ruin wäre 0 % oder 100 %. Die App meldet das und sperrt den Start.
- Eine **Pensionierung** in der Planung (Pensionskasse «Bezug ab»), denn die Strategien gelten ab dann.

**Wie lange dauert es?** Unter dem Knopf «Optimierung starten» steht eine **Schätzung der Dauer**, die sich bei jeder Änderung der Einstellungen anpasst. Sie beruht auf einer Messung: Die App rechnet kurz einige tausend Läufe Ihrer Planung und rechnet damit hoch, wie viele Läufe die Optimierung braucht (Pfade der Suche, Startpunkte und Strategien, dazu die Prüfpfade). Die Angabe ist **grob**: Die Zahl der Suchschritte hängt vom Fall ab, deshalb steht eine Spanne dabei («meist 7 s bis 20 s»). Beispiele auf einem aktuellen Mac mit 10 Kernen: Standardeinstellungen mit allen sechs Strategien dauern bei Bootstrap etwa 15 s, mit dem parametrischen Modell ebenso lang, bei «Alle Startzeitpunkte» 2 bis 3 s. Doppelt so viele Pfade bedeuten etwa doppelt so lange. Die Berechnung läuft lokal und nutzt alle Prozessorkerne.

**Während der Optimierung** zeigt ein Fortschrittsbalken den Stand (der Prozentwert ist eine Schätzung, weil die Zahl der Suchschritte vom Fall abhängt). Mit **Abbrechen** stoppen Sie die Rechnung; es bleibt kein Teilergebnis.

**Welche Parameter variiert werden**, bestimmen Sie je Strategie selbst, siehe [Parameter fix oder variieren](#o-ranges).

**Gespeichert werden die Einstellungen** (Ziel, Strategien, Pfade, Bandbreiten der Parameter) **mit der Planung**: Sie stehen in der `.vplan`-Datei, kommen beim Öffnen wieder und gelten auch für `optimize` im Terminal (Optionen überschreiben sie dort). Das Ergebnis legt die App in der [Liste der Ergebnisse](#r-list) ab, siehe [Ergebnis speichern und wieder öffnen](#o-saved). Eine geänderte Einstellung macht ein früheres Ergebnis nicht ungültig, nur eine geänderte Planung.

Lesen Sie die Ergebnisse mit [Ergebnis lesen](#o-read) und beachten Sie die [Fallstricke](#o-pitfalls). Ein Optimum ist **kein Versprechen**.

## @o-goal-select Zielauswahl
Stichworte: Ziel, Kein Notverkauf von Immobilien, Notverkauf, Mindestkonsum

Die Optimierung hat zwei Ziele; die Wahl wird mit der Planung gespeichert (Standard: Ziel 1).

- **Ziel 1, Konsum bei begrenztem Ruinrisiko:** wie in diesem Kapitel beschrieben: möglichst hoher und gleichmässiger Konsum, solange der Ruin unter der Grenze bleibt.
- **Ziel 2, Kein Notverkauf von Immobilien:** Gesucht wird die Strategie mit dem **geringsten Anteil an Läufen, in denen eine Immobilie ungeplant verkauft wird** (Notverkauf). Bei gleichem Anteil entscheidet die geringere Ruinwahrscheinlichkeit (Ruin wie in Ziel 1). Nebenbedingungen: Der **Median des jährlichen Konsums ab Pensionierung** beträgt mindestens den eingegebenen Anteil der geplanten Ausgaben (Standard 90 %), und die Ruinwahrscheinlichkeit liegt unter der Grenze (Standard 5 %). Konsumgewichte und Schwankung spielen bei Ziel 2 keine Rolle.

Die Ergebnistabelle nennt das gewählte Ziel. Bei Ziel 2 zeigt sie je Strategie den **Anteil der Läufe mit ungeplantem Verkauf**; die Reihenfolge richtet sich nach diesem Anteil, bei Gleichstand nach dem Ruin. Strategien, die die Nebenbedingungen verletzen, stehen unten und sind markiert. Erfüllt keine Strategie die Nebenbedingungen, steht oben die, die ihnen am nächsten kommt, mit einem Hinweis.

Ziel 2 braucht mindestens eine Immobilie in der Planung (gekauft oder geplant); ohne Immobilie ist es nicht wählbar.

## @o-goal Ziel: Ruinwahrscheinlichkeit
Stichworte: Ruin, Grenze, Alter, Erschöpfung, 5 Prozent, Risiko

Die Ruinwahrscheinlichkeit ist die zentrale **Nebenbedingung**. Eine Strategie, die sie nicht einhält, kann nicht gewinnen, egal wie hoch ihr Konsum ist.

- **Höchste Ruinwahrscheinlichkeit** (Standard 5 %): Anteil der Pfade, in denen das Vermögen aufgebraucht ist, der höchstens erlaubt ist. **Ruin** heisst: kein liquides Vermögen und keine Immobilien mehr. Üblich sind 1 % bis 10 %. Je tiefer die Grenze, desto sicherer und desto tiefer der mögliche Konsum.
- **Ruin zählt bis Alter** (leer = bis zum Simulationsende): Wenn Sie nur bis zu einem bestimmten Alter Sicherheit brauchen, tragen Sie es hier ein. Ein Ruin mit 92 zählt dann bei «bis 90» nicht. Das ist eine **bewusste Lockerung**, denken Sie an die Langlebigkeit. Mit leerem Feld gilt die Grenze bis zum Ende der Simulation.

> Welche Grenze ist vernünftig? Eine Ruinwahrscheinlichkeit von 5 % bedeutet, dass in 1 von 20 simulierten Zukünften das Geld ausgeht. Vergleichen Sie das mit Ihren Reserven: AHV und Pensionskasse laufen weiter, das Vermögen ist nur ein Teil der Finanzierung. Bei zwei oder mehr Quellen mit unterschiedlichem Ergebnis ist eine tiefere Grenze (2 % bis 3 %) eine vernünftige Vorsicht.

## @o-consumption Ziel: Konsum (Median und P10)
Stichworte: Median, Perzentil, P10, Gewicht, Konsum, Ausgaben, real

Was optimiert wird, ist der **jährliche Konsum ab der Pensionierung**: die gesamten Ausgaben inklusive Wohnkosten, **vor Steuern**, in **Preisen des Startmonats** (also real). Die App sammelt die Jahreswerte **aller Pfade und aller Jahre** und bildet daraus zwei Kennzahlen:

- **Median:** der typische Jahreskonsum. Die Hälfte aller Pfad-Jahre liegt darüber, die Hälfte darunter.
- **P10 (10. Perzentil):** der Jahreskonsum, der in 10 % der Pfad-Jahre unterschritten wird. Das ist der **schlechte Fall** und misst, wie hart es in Krisen wird.

**Gewicht des Medians im Konsumziel** (Standard 50 %): Wie das Konsumziel aus beiden Kennzahlen gemischt wird. Der Rest geht auf P10.

> Konsumziel = Gewicht × Median + (1 − Gewicht) × P10

- **100 %:** Nur der typische Konsum zählt. Die Optimierung wählt aggressivere Einstellungen mit hohem Median und schlechten Krisenjahren.
- **0 %:** Nur der schlechte Fall zählt. Das führt zu vorsichtigen Einstellungen.
- **50 %** ist ein ausgewogener Ausgangspunkt. Erhöhen Sie das Gewicht des P10, wenn Ihnen **Verlässlichkeit** wichtiger ist als der Durchschnitt.

Ab dem Ruin zählt als Konsum nur noch, was Einkommen und Renten decken, nicht mehr die geplanten Ausgaben. Ein Ruin drückt also den Median und vor allem das P10.

## @o-vol Ziel: Schwankung
Stichworte: Volatilität, Schwankung, Jahr zu Jahr, Abzug, Kürzung, stabil

Die **Schwankung** misst, wie stark sich der Jahreskonsum von einem Jahr zum nächsten ändert: die **mittlere absolute Änderung** gegenüber dem Vorjahr, gemittelt über alle Pfade. 10 % heisst, dass der Konsum im Mittel pro Jahr um 10 % steigt oder sinkt. Feste Ausgaben (Standard, 3-Topf) haben eine Schwankung nahe 0 %, Strategien, die dem Vermögen folgen (fixe Rate, VPW), eine hohe.

Zwei Einstellungen steuern, wie die Schwankung ins Ziel eingeht:

- **Abzug für Schwankung** (Standard 1): Das Konsumziel wird mit dem Faktor (1 − Abzug × Schwankung) multipliziert. Bei Abzug 1 und 10 % Schwankung sinkt das Ziel um 10 %. **0 bedeutet, dass Schwankung egal ist**, dann gewinnen die Strategien mit dem höchsten Konsum, auch wenn er stark schwankt.
- **Höchste Schwankung** (leer = keine Grenze): Eine **harte Obergrenze** als Nebenbedingung, wie bei der Ruingrenze. Beispiel 10 %: Strategien, deren mittlere Jahresänderung höher ist, sind ausgeschlossen. Das ist die Einstellung für «Ich will, dass meine Ausgaben im Mittel nicht mehr als 10 % pro Jahr schwanken».

## @o-score Das Ziel in einer Formel
Stichworte: Formel, Zielwert, Bewertung, Fitness, Score

> **Ziel** = (Gewicht × Median + (1 − Gewicht) × P10) × (1 − Abzug × Schwankung)

Das Ziel wird in Franken pro Jahr (reale Preise) angezeigt. **Ein Kandidat ist zulässig**, wenn die Ruinwahrscheinlichkeit unter der Grenze liegt und (falls gesetzt) die Schwankung unter der Höchstgrenze bleibt. Unter den zulässigen gewinnt der mit dem höchsten Ziel. Haben zwei dasselbe Ziel (zum Beispiel bei festen Ausgaben, wo alle Strategien denselben Konsum haben), gewinnt der mit der **kleineren Ruinwahrscheinlichkeit**. Nicht zulässige Kandidaten werden danach geordnet, wie stark sie die Grenze verletzen.

Bei festen Ausgaben (Standard, 3-Topf) ist der Konsum in allen Läufen gleich. Dort entscheidet die Optimierung also **über die Sicherheit**, nicht über den Konsum. Bei Strategien, die dem Vermögen folgen, ergibt sich der Konsum aus der Strategie.

## @o-strategies Strategien und ihre Parameter
Stichworte: Parameter, Suchbereich, Standard, 3-Topf, Guardrails, VPW, Endowment, Fixe Rate, Aktienanteil

Wählen Sie die Strategien, die verglichen werden sollen. Es empfiehlt sich, **alle** zu rechnen, um zu sehen, wie sich die Ansätze unterscheiden. Für jede Strategie sucht die App diese Parameter in diesen **Standard-Bereichen**. Welche Parameter variiert werden und in welcher Bandbreite, bestimmen Sie selbst, siehe [Parameter fix oder variieren](#o-ranges).

| Strategie | Parameter | Suchbereich |
|---|---|---|
| Standard | Aktienanteil, Reserve (Monate) | 10 % bis 100 %, 3 bis 36 |
| 3-Topf | Topf 1: Cash (Jahre), Topf 2: Obligationen (Jahre), Nachfüllen ab Vorjahresrendite | 0.5 bis 4, 0 bis 10, −10 % bis +20 % |
| Guardrails | Startrate, obere und untere Leitplanke, Anpassung, Aktienanteil | 2 % bis 8 %, je 5 % bis 50 %, 2 % bis 30 %, 10 % bis 100 % |
| Fixe Rate | Entnahmerate, Aktienanteil | 2 % bis 8 %, 10 % bis 100 % |
| VPW | Planungsalter, Realrendite, Aktienanteil | 85 bis 105, −1 % bis 5 %, 10 % bis 100 % |
| Endowment | Zielrate, Glättung, Aktienanteil | 2 % bis 8 %, 5 % bis 100 %, 10 % bis 100 % |

**Wichtig:**

- Der **Aktienanteil** wird **ab der Pensionierung** verändert. Bis zur Pensionierung bleibt der Verlauf Ihrer Planung, auch eine Gerade, Tabelle oder ein Faktor (er wird dazu Monat für Monat als Treppe übernommen). Eine [Kennlinie](#f-curve) lässt sich nicht an einem Zeitpunkt teilen; sie wird durch den optimierten Wert ersetzt, der dann ab Beginn gilt.
- Die **Grenzen der Lebenshaltung** (Untergrenze, Obergrenze) bleiben standardmässig fix auf dem Wert Ihrer Planung. Das gilt bei allen Strategien, die die Lebenshaltung selbst festlegen: Fixe Rate, Guardrails, VPW, Endowment und 3-Topf + Guardrails. Sie können sie dort ebenfalls variieren lassen. Eine Untergrenze kann die Optimierung stark einschränken, weil sie Kürzungen verhindert.
- Die **3-Topf-Strategie** legt den Aktienanteil durch die Töpfe fest, deshalb wird er dort nicht gesucht.
- Ein Parameter am Rand des Bereichs ist ein Hinweis, dass der Bereich begrenzt: Das Optimum liegt am Rand, nicht dahinter.

Die Parameter sind unter den Strategien beschrieben: [Standard](#f-wd-standard), [3-Topf](#f-wd-three_buckets), [Guardrails](#f-wd-guardrails), [Fixe Rate](#f-wd-fixed_rate), [VPW](#f-wd-vpw), [Endowment](#f-wd-endowment).

## @o-ranges Parameter fix oder variieren
Stichworte: Bandbreite, fix, variieren, von, bis, Parameter, Suchbereich, Standard-Bandbreiten, Topf

Unter «Parameter der Strategien» steht für jede Strategie eine Tabelle mit allen ihren Parametern:

- **In der Planung:** der Wert, den Ihre Planung heute hat (oder der Standardwert der Strategie, wenn Sie sie noch nie benutzt haben).
- **variieren:** Mit Haken sucht die Optimierung diesen Parameter innerhalb von «von» bis «bis». **Ohne Haken bleibt er fix** auf dem Wert der Planung.
- **von / bis:** Die Bandbreite in der **Einheit des Parameters**: Jahre in Jahren, Monate in Monaten, Prozent in Prozent, Alter in Jahren.

**Beispiel 3-Topf:** Topf 1 fix (kein Haken), Topf 2 von 0 bis 2 Jahre, Nachfüllen ab einer Vorjahresrendite von −30 % bis +10 %. Die Suche verändert dann nur Topf 2 und die Nachfüll-Schwelle, Topf 1 bleibt, wie er in der Planung steht.

**Gut zu wissen:**

- Eine **enge Bandbreite** macht die Suche schneller und das Ergebnis näher an Ihrer Planung. Eine **weite** findet eher Überraschungen, braucht aber mehr Rechenzeit.
- Sind **alle** Parameter einer Strategie fix, wird nur diese eine Planung gerechnet. Das ist nützlich, um Strategien mit festen Werten fair zu vergleichen. Schneller geht das mit [Strategien vergleichen (ohne Optimierung)](#o-compare), ohne die Haken zu ändern.
- «von» darf nicht grösser sein als «bis». Es gibt äusserste Grenzen je Parameter (z. B. Aktienanteil 0 % bis 100 %); ausserhalb meldet die App einen Fehler.
- **Untergrenze und Obergrenze der Lebenshaltung** (Fixe Rate, Guardrails, VPW, Endowment, 3-Topf + Guardrails) sind zunächst fix. Wer sie variiert, bekommt oft Ergebnisse, die nur mit der Untergrenze «gut» aussehen, weil Kürzungen verhindert werden.
- «Standard-Bandbreiten» setzt die Strategie auf die Voreinstellung zurück.
- Die Einstellungen werden **mit der Planung gespeichert**.

## @o-compare Strategien vergleichen (ohne Optimierung)
Stichworte: vergleichen, ohne Suche, Werte der Planung, Endvermögen, Immobilien nie verkaufen, beide

Der Knopf **«Strategien vergleichen (ohne Optimierung)»** neben «Optimierung starten» rechnet die gewählten Strategien **mit den Parameterwerten Ihrer Planung**, ohne nach besseren Werten zu suchen. Für eine Strategie, die Ihre Planung nie benutzt hat, gelten ihre Standardwerte (Spalte «In der Planung» unter «Parameter der Strategien»).

- Die Haken **«variieren»** und die Bandbreiten bleiben unverändert; der Vergleich beachtet sie nicht. Danach können Sie wie gewohnt optimieren.
- Gerechnet wird auf **denselben Pfaden wie die Simulation** (Anzahl und Zufallsstartwert der Renditequelle). Die Zahlen einer Strategie stimmen daher mit einer Simulation überein, in der Sie diese Strategie eingestellt haben. Die Pfade für Suche und Prüfung unter «Erweitert» gelten nur für die Optimierung.
- Die Tabelle ist dieselbe wie nach einer Optimierung (siehe [Ergebnis lesen](#o-read)); die Zeile unter jeder Strategie zeigt die Werte der Planung.

**Immobilien:** Hat Ihre Planung eine eigene oder geplante Immobilie, legt die Auswahl «Immobilien» fest, womit gerechnet wird, beim Vergleich wie bei der Optimierung:

- **wie geplant:** mit Ihrer Planung, wie sie ist;
- **Immobilien nie verkaufen:** mit eingeschaltetem Schalter [Immobilien nie verkaufen](#f-re-never-sell);
- **beide:** jede Strategie zweimal, beschriftet mit «wie geplant» und «Immobilien nie verkaufen».

Ohne eigene Immobilie ist die Auswahl ausgeblendet. Die Auswahl wird mit der Planung gespeichert.

## @o-advanced Erweiterte Einstellungen
Stichworte: Pfade, Prüfung, Startpunkte, Genauigkeit, Rechenzeit

- **Pfade für die Suche** (Standard 5 000): So viele Zufallspfade bewertet die Suche für jeden Kandidaten. **Mehr Pfade** machen das Ergebnis genauer, aber langsamer. Faustregel: Unter 2 000 Pfaden wird die Bewertung unruhig, über 20 000 lohnt der Aufwand meist kaum.
- **Pfade für die Prüfung** (Standard 10 000): So viele **andere** Pfade rechnen die gefundenen Parameter am Schluss nach. Diese Werte sehen Sie im Ergebnis.
- **Zufällige Startpunkte pro Strategie** (Standard 30): Die Suche beginnt an der Planung, an der Mitte des Suchbereichs und an so vielen Zufallspunkten. Mehr Startpunkte helfen, nicht in einem schlechten lokalen Optimum hängen zu bleiben, kosten aber Zeit, vor allem bei Strategien mit vielen Parametern (Guardrails).

Die Pfade stammen aus der Renditequelle der Planung (Bootstrap oder parametrisch). Die Prüfpfade haben einen **anderen Zufallsstartwert**. Bei «Alle historischen Startzeitpunkte» gibt es keine Zufallspfade. Suche und Prüfung rechnen dann dieselben historischen Abschnitte, die **Prüfung ist dort nicht unabhängig**.

**Obergrenzen:** Eine Optimierung rechnet höchstens **20 Millionen Läufe** (Suche und Prüfung zusammen; die Standardwerte brauchen mit allen Strategien rund 1,7 Millionen). Mit Bootstrap oder parametrischen Renditen gilt für die Pfade der Suche und der Prüfung zudem dieselbe Speichergrenze wie für die Pfade der Planung. Darüber lehnt die App die Optimierung mit einer Meldung ab, statt stundenlang zu rechnen. Eine Strategie, die doppelt gewählt ist, wird einmal optimiert.

## @o-algorithm Wie die Suche funktioniert
Stichworte: Algorithmus, Zufallssuche, Mustersuche, Sicherheitsabstand, Prüfung, unabhängig, Überanpassung

1. **Gemeinsame Pfade:** Alle Kandidaten werden mit **denselben** Renditepfaden bewertet. Unterschiede kommen so von den Parametern, nicht vom Zufall.
2. **Zufallssuche:** Die App bewertet Startpunkte (die aktuelle Planung, die Mitte des Bereichs und zufällige Punkte) und nimmt den besten.
3. **Mustersuche (Koordinatensuche):** Vom besten Punkt aus wird jeder Parameter einzeln erhöht und verringert. Bringt eine Änderung Verbesserung, wird sie übernommen. Findet sich keine, wird die Schrittweite halbiert, bis sie sehr klein ist.
4. **Sicherheitsabstand:** Die Suche würde die Ruingrenze sonst bis zum Anschlag ausnutzen und auf neuen Pfaden knapp darüber landen (Zufallsfehler der Schätzung). Darum sucht sie mit einer **um 1.5 Standardfehler strengeren** Grenze. Bei 5 % Grenze und 5 000 Pfaden sind das rund 0.5 Prozentpunkte.
5. **Unabhängige Prüfung:** Das Ergebnis wird auf **anderen** Pfaden (anderer Zufallsstartwert, mehr Pfade) nachgerechnet. Erst diese Prüfung entscheidet, ob eine Strategie zulässig ist.

Die Prüfung schützt vor **Überanpassung**: Eine Strategie, die nur auf den Suchpfaden gut aussieht, fällt in der Prüfung durch.

## @o-read Ergebnis lesen
Stichworte: Tabelle, Ausgangslage, Stern, ausgegraut, Ruin, Median, P10, Schwankung, Ziel, Balken, Prüfung

Die Ergebnistabelle zeigt für jede Strategie die Kennzahlen, **gerechnet auf den unabhängigen Prüfpfaden**. Der Konsum steht in Franken pro Jahr, in Preisen des Startmonats, inklusive Wohnkosten und vor Steuern.

| Spalte | Bedeutung |
|---|---|
| Strategie | Name; ★ markiert die beste zulässige |
| Ruin | Anteil der Pfade, in denen das Vermögen aufgebraucht ist (bis zum gewählten Alter) |
| Median | typischer Jahreskonsum |
| P10 | Jahreskonsum im schlechten Fall |
| Schwankung | mittlere Änderung des Jahreskonsums gegenüber dem Vorjahr |
| Ziel | der kombinierte Wert nach der [Formel](#o-score), mit Balken zum Vergleich |
| Entnahmerate | Entnahme aus dem Vermögen im **ersten Jahr der Pensionierung** (Lebenshaltung und Wohnkosten minus Einnahmen, **vor Steuern**), in Prozent des liquiden Vermögens zu Beginn; Median über die Pfade |
| Endvermögen 1 % … 99 % | nominales Vermögen am Ende der Simulation bei den Perzentilen 1, 5, 50, 95 und 99 % der Pfade, ermittelt wie im Ergebnis der Simulation; auch für die Ausgangslage |
| Auf Untergrenze | Anteil der Monate ab Pensionierung, in denen die [Untergrenze](#f-wd-limits) die Lebenshaltung festlegt, gemittelt über die Prüfpfade; «–» bei Strategien ohne Untergrenze (Standard, 3-Topf mit Ausgaben wie geplant, Untergrenze 0). Die Spalte steht bei beiden Zielen |

Unter jeder Strategie steht eine **eigene Zeile** über die ganze Tabellenbreite mit den gefundenen Parametern (beim Vergleich ohne Optimierung den Werten der Planung) und dem Knopf «In Planung übernehmen». Damit die Tabelle Platz hat, ist im Abschnitt «Optimierung» die Vorschau mit der Prüfung rechts ausgeblendet.

- Die erste Zeile **«Ausgangslage»** zeigt Ihre Planung unverändert, auf denselben Prüfpfaden. Daran messen Sie den Gewinn der Optimierung.
- **Ausgegraute Zeilen** («Ruingrenze nicht eingehalten») sind Strategien, die Ihre Ruingrenze verfehlen. Sie bleiben sichtbar, damit Sie die Kosten der Sicherheit erkennen. Ist **keine** Strategie zulässig, ist die Grenze mit den vorhandenen Mitteln nicht erreichbar, Sie müssen dann die Ausgaben senken, später in Pension gehen oder die Grenze lockern.
- Die **Entnahmerate** ist ein Vergleichswert, kein Ziel der Suche. Sie zeigt, wie stark eine Strategie das Vermögen zu Beginn beansprucht: Eine hohe Rate bei gleich hohem Konsum heisst, dass die Strategie mehr vom Vermögen zehrt und weniger auf Renten und Einkommen baut. Die Faustregel «unter 4 %» gilt nur grob (siehe [Entnahme aus dem Vermögen](#r-draw)).
- **Auf Untergrenze:** Liegt die Lebenshaltung oft auf der Untergrenze, gibt die Strategie kaum etwas vor: Sie würde weniger erlauben, die Untergrenze hebt die Ausgaben an. Mehrere Strategien rechnen dann praktisch gleich. Ab **50 %** der Monate steht unter der Strategie der Hinweis «Die Untergrenze bestimmt meist die Ausgaben; die Strategie wirkt kaum.» Prüfen Sie dann, ob die Untergrenze zu hoch ist oder die geplanten Ausgaben das Vermögen übersteigen. Gespeicherte Ergebnisse aus älteren Versionen zeigen hier «–».
- Sinkt der Konsum gegenüber der Ausgangslage stark, kostet die Sicherheit entsprechend. Das ist **kein Fehler**, sondern die Antwort.
- Unter der Tabelle stehen Anzahl Pfade für Suche und Prüfung und die Rechenzeit.

Wurde die Planung nach der Optimierung geändert, erscheint ein Hinweis: Das Ergebnis passt dann nicht mehr zur Planung und muss neu gerechnet werden.

## @o-apply In die Planung übernehmen
Stichworte: übernehmen, Parameter, Strategie setzen, anwenden

«In Planung übernehmen» setzt in Ihrer Planung:

- die **Strategie** und ihre Parameter (zum Beispiel Startrate und Leitplanken bei Guardrails),
- den **Aktienanteil** und die **Cash-Reserve** ab der Pensionierung.
- den Schalter **«Immobilien nie verkaufen»**, wenn die Zeile damit gerechnet wurde.

Die Planung gilt danach als geändert (ungespeichert). Mit **Rückgängig** (⌘Z) lässt sich die Übernahme zurücknehmen. Das Ergebnis der Optimierung bleibt sichtbar und gilt weiterhin, solange Sie die Planung nicht anders ändern.

Prüfen Sie anschliessend das Ergebnis mit einer **normalen Simulation** und, wenn möglich, mit einer **anderen Renditequelle**. Ist das Optimum dort noch gut, ist es robust.

## @o-saved Ergebnis speichern und wieder öffnen
Stichworte: gespeichert, Liste, Ergebnisse, wieder öffnen, Strategievergleich, Neustart

Jede **abgeschlossene** Optimierung und jeder Strategievergleich wird automatisch in der [Liste der Ergebnisse](#r-list) abgelegt, mit dem Bericht, den Einstellungen der Optimierung und dem Planungsstand zur Zeit der Rechnung. Ein Abbruch oder ein Fehler legt nichts ab. Wie die Simulationen liegt der Eintrag **lokal im Speicher der App**, nicht in der Planungsdatei, und bleibt nach einem Neustart erhalten. Ist der Speicher der App gesperrt, bleibt er bis zum Schliessen im Arbeitsspeicher.

- **In der Liste** heisst der Eintrag «… · Optimierung · …» bzw. «… · Strategievergleich · …». Er nennt die beste Strategie (ihre Parameter beim Daraufzeigen), ihre Ruinwahrscheinlichkeit und das Ziel; beim Ziel «Kein Notverkauf» zusätzlich den Anteil ungeplanter Verkäufe. Umbenennen, Notiz und Löschen funktionieren wie bei Simulationen.
- **Öffnen** zeigt das Ergebnis in der breiten Ergebnisansicht mit derselben Tabelle wie im Abschnitt «Optimierung», mit Datum und Zahl der geprüften Pfade.
- **«In Planung übernehmen»** übernimmt die Parameter in die offene Planung. Gehört das Ergebnis zu einem anderen Planungsstand (andere Planung oder seither geändert), steht oben ein Hinweis, und die App fragt vor dem Übernehmen nach.
- **Beim Öffnen einer Planung** zeigt der Abschnitt «Optimierung» das neueste gespeicherte Ergebnis, das zu genau diesem Planungsstand passt.
- **Vergleichen** lässt sich eine Optimierung nicht mit Simulationen: Markiert lässt sie sich löschen, «Vergleichen» lässt sie aus.

## @o-pitfalls Fallstricke der Optimierung
Stichworte: Überoptimierung, Robustheit, Modellrisiko, Gegenprobe, Warnung

Eine Optimierung findet das beste Ergebnis **für die Annahmen**. Sie sagt nichts darüber, ob die Annahmen stimmen.

- **Das Optimum gilt nur für diese Renditen.** Wurde mit der Shiller-Reihe optimiert, muss das Ergebnis bei einer anderen Reihe oder bei tieferen Renditen nicht gelten. **Rechnen Sie das Ergebnis mit einer zweiten Quelle gegen**: optimieren Sie mit Bootstrap und prüfen Sie mit dem parametrischen Modell, oder umgekehrt.
- **Genaue Zahlen täuschen Genauigkeit vor.** Ein Aktienanteil von 63.4 % ist nicht besser als 60 %. Runden Sie auf verständliche Werte und prüfen Sie, wie stark sich das Ziel dadurch ändert. Oft ist die Fläche um das Optimum **flach**.
- **Randlösungen:** Liegt ein Parameter am Rand des Suchbereichs (zum Beispiel Aktienanteil 100 % oder Startrate 8 %), ist das ein Zeichen, dass das Ergebnis extrem ist. Misstrauen Sie ihm.
- **Eine zu optimistische Ruingrenze.** Mit 10 % oder mehr sind hohe Konsumwerte leicht möglich, aber mit einer Wahrscheinlichkeit von 1 zu 10 verlieren Sie Ihr Vermögen.
- **Sequenzrisiko und lange Perioden.** Die Ruinschätzung bei Bootstrap und parametrischem Modell hängt stark von den Annahmen zu den Krisen ab.
- **Nicht modellierte Dinge:** Anlagekosten, Gesetzesänderungen, Pflegekosten, Todesfälle. Siehe [Grenzen des Modells](#limits).
- **Anpassung an die Zufallspfade:** Ohne unabhängige Prüfung würde die Optimierung auf Zufallsschwankungen reagieren. Die App prüft deshalb auf anderen Pfaden. Bei «Alle historischen Startzeitpunkte» fehlt diese Unabhängigkeit.

> Eine Modellrechnung ist keine Anlageberatung. Ein Optimum ist ein **Ausgangspunkt für Ihre Überlegungen**, kein Auftrag.

## @o-example Beispiel für die Interpretation
Stichworte: Beispiel, Interpretation, Anleitung, Vorgehen

Die Zahlen sind erfunden und dienen nur dazu, das Lesen zu üben.

| Strategie | Ruin | Median | P10 | Schwankung | Ziel |
|---|---|---|---|---|---|
| Ausgangslage (Standard) | 7.8 % | 72 000 | 72 000 | 0 % | 72 000 |
| Guardrails ★ | 3.9 % | 78 500 | 66 000 | 6 % | 68 000 |
| Fixe Rate | 1.1 % | 84 000 | 51 000 | 21 % | 53 000 |

- Die Ausgangslage hält die Ruingrenze von 5 % **nicht** ein (7.8 %).
- Guardrails erreicht **3.9 %** und liefert dabei im Median **mehr** Konsum. Dafür sinkt der Konsum im schlechten Fall auf 66 000 und schwankt um 6 % pro Jahr. Das ist der Preis der Sicherheit.
- Die fixe Rate wäre noch sicherer, schwankt aber so stark (21 %), dass das Ziel unter Guardrails liegt.

**Was Sie daraus machen:** Prüfen Sie, ob Sie mit 66 000 im schlechten Fall leben könnten (Wohnen, Gesundheit, Mindestlebenshaltung). Wenn ja, ist Guardrails ein guter Kandidat. Rechnen Sie es danach mit einer zweiten Renditequelle nach.

## @o-cli Optimierung auf der Kommandozeile
Stichworte: Terminal, optimize, Befehl, Optionen, Skript

Dieselbe Rechnung gibt es im Terminal:

```
vivaplan optimize <planung.vplan> [Optionen]
```

| Option | Bedeutung |
|---|---|
| `--goal 1\|2` | Ziel: 1 Konsum bei begrenztem Ruinrisiko (Standard), 2 Kein Notverkauf von Immobilien |
| `--min-consumption <Prozent>` | Ziel 2: Mindestanteil der geplanten Ausgaben für den Median des Konsums (Standard 90) |
| `--max-ruin <Prozent>` | höchste Ruinwahrscheinlichkeit |
| `--ruin-age <Alter>` | Ruin zählt nur bis zu diesem Alter |
| `--weight-median <0 bis 1>` | Gewicht des Medians im Konsumziel |
| `--vol-weight <Zahl>` | Abzug für die Schwankung |
| `--max-vol <Prozent>` | höchste Schwankung |
| `--paths`, `--check-paths`, `--samples` | Pfade der Suche und Prüfung, Startpunkte |
| `--seed` | Zufallsstartwert |
| `--strategy a,b` | Strategien (`standard`, `three_buckets`, `guardrails`, `fixed_rate`, `vpw`, `endowment`) |
| `--json [<datei.json>]` | ohne Datei: Ergebnis im festen Format auf die Standardausgabe ([Ausgabe als JSON](#ref-json)); mit Datei: Ergebnis als JSON speichern |
| `--compare` | Strategien ohne Suche mit den Werten der Planung vergleichen, auf den Pfaden der Simulation |
| `--property planned\|never-sell\|both` | Immobilien: wie geplant (Standard), nie verkaufen oder beide (jede Strategie zweimal) |

Sind in der Planung Einstellungen gespeichert, gelten sie als Ausgangspunkt. Jede angegebene Option überschreibt sie. Die **Bandbreiten der Parameter** (`optimization.ranges`) liest der Befehl immer aus der Planung; im Terminal gibt es dafür keine Optionen. Die Ausgabe zeigt je Strategie die Parameter und die Kennzahlen aus Suche und Prüfung; sie ist englisch (siehe [Kommandozeile](#ref-cli)). Mit `--compare` zeigt sie je Strategie die Werte der Planung und eine Zeile mit den Kennzahlen; das Endvermögen bei 1, 5, 50, 95 und 99 % steht immer dabei.
