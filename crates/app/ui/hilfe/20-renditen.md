# Simulation und Renditen

## @s-returns Simulation: Überblick
Stichworte: Rendite, Inflation, Quelle, Methode, Zeitreihe, Monte Carlo, SARON, Anlagestrategie

Der Abschnitt «Simulation» bestimmt, **wie die Renditen von Aktien und Obligationen, die Inflation und der SARON über die Zeit verlaufen**: fixe Werte, ein historischer Verlauf oder Monte Carlo. Er hat den grössten Einfluss darauf, wie die Ergebnisse aussehen und wie viel Sie ihnen glauben dürfen.

**Welche Indizes** die Renditen liefern (zum Beispiel Welt-Aktien und Obligationen der Eidgenossenschaft), legen Sie bei der [Anlagestrategie](#f-ret-series) im Abschnitt «Vermögen & Anlage» fest. Eine Zeitreihe wählen Sie nicht mehr selbst; sie ergibt sich aus der Strategie.

Es gibt fünf Methoden. Die Ansicht zeigt nur die Eingaben, die für die gewählte Methode gelten.

| Quelle | Läufe | Wofür |
|---|---|---|
| [Fixe Renditen](#f-ret-fixed) | 1 | Schnelle Abschätzung, Vorschau, Plausibilitätsprüfung |
| [Historisch ab Startzeitpunkt](#f-ret-historical) | 1 | «Was wäre gewesen, wenn ich 1973 in Pension gegangen wäre?» |
| [Alle historischen Startzeitpunkte](#f-ret-rolling) | ca. 400 bis 900 | Wie sich jeder reale Start ausgewirkt hätte |
| [Monte Carlo · historisch (Bootstrap)](#f-ret-bootstrap) | frei, z. B. 50 000 | Breite Streuung aus echten Monatsdaten |
| [Monte Carlo · parametrisch](#f-ret-parametric) | frei, z. B. 50 000 | Modell mit Vorschlagswerten je Index oder eigenen Annahmen |

Wenn Sie nicht wissen, womit beginnen: lesen Sie [Welche Quelle passt?](#ret-which).

Grundlagen, die für alle Quellen gelten, stehen unter [Rendite, Inflation und reale Werte](#ret-concepts). Häufige Denkfehler sind unter [Fallstricke](#ret-pitfalls) beschrieben.

## @ret-which Welche Quelle passt?
Stichworte: Entscheidung, Empfehlung, Auswahl, Quelle wählen

Eine Faustregel:

1. **Zum Eingeben und Ausprobieren:** «Fixe Renditen». Die Vorschau rechts rechnet immer mit festen Werten und reagiert sofort.
2. **Für die eigentliche Antwort «Reicht das Geld?»:** Monte Carlo, am besten **«Monte Carlo · historisch» (Bootstrap)** mit einer Anlagestrategie, die lange zurückreicht (zum Beispiel «Welt (JST)» oder «S&P 500»). Er bewahrt die echten Eigenschaften der Märkte (Krisen, Erholungen, Zusammenhang von Aktien, Obligationen und Inflation) und streut trotzdem breit.
3. **Als Gegenprobe:** «Alle historischen Startzeitpunkte». Sie zeigt konkret, wie der Plan in realen Zeitabschnitten abgelaufen wäre, etwa mit Start 1929, 1966 oder 2000.
4. **Für eigene Annahmen:** «Monte Carlo · parametrisch», wenn Sie die erwartete Rendite und das Risiko selbst vorgeben wollen, zum Beispiel weil Sie in Zukunft tiefere Renditen erwarten als in der Vergangenheit.

**Gute Praxis:** Rechnen Sie wichtige Entscheide mit **mindestens zwei Quellen** (Bootstrap und parametrisch) und prüfen Sie, ob die Aussage dieselbe bleibt. Ein Plan, der nur mit einer einzigen Quelle gut aussieht, ist fragil.

| Frage | Quelle |
|---|---|
| Wie wahrscheinlich reicht mein Geld? | Bootstrap oder parametrisch |
| Wie schlimm kann es kommen? | Bootstrap, 1. Perzentil und schlechtester Lauf |
| Hätte der Plan 1973 funktioniert? | Historisch ab Startzeitpunkt |
| Funktioniert der Plan in jeder historischen Lage? | Alle historischen Startzeitpunkte |
| Was, wenn die Börse künftig weniger bringt? | Parametrisch mit tieferem μ |

## @ret-concepts Rendite, Inflation und reale Werte
Stichworte: nominal, real, Teuerung, geometrisch, Jahresrendite, Kaufkraft

**Jahresrenditen** werden in der App monatlich angewandt. Eine Jahresrendite von 5 % ergibt pro Monat (1.05)^(1/12) − 1, also rund 0.41 %. Der Jahresverlauf stimmt damit genau.

**Inflation** wirkt jeden Monat. Der Teuerungsindex wächst monatlich, und alle Ausgaben, Mieten und (falls gewählt) Immobilienwerte werden damit fortgeschrieben. Ihre Ausgaben erfassen Sie in **heutigen Preisen**, die App rechnet sie mit der Inflation in die Zukunft.

**Nominal und real:** Die Ergebnisse gibt es in zwei Ansichten.

- **Nominal:** Beträge, wie sie dann anfallen (in künftigen Franken).
- **Heutige Kaufkraft:** Beträge in Preisen des Simulationsstarts, bereinigt um die Teuerung. Diese Ansicht ist für Entscheide meist die ehrlichere, weil 2 Millionen in 30 Jahren weniger Kaufkraft haben als heute.

**Arithmetisch und geometrisch:** Eine «durchschnittliche Rendite von 7 %» ist meist ein arithmetischer Durchschnitt. Der **geometrische** Durchschnitt (was Ihr Vermögen wirklich wächst) ist wegen der Schwankungen tiefer. Bei 7 % Erwartung und 15 % Schwankung liegt er bei rund 5.9 %. Die Kurzinfo der Anlagestrategie zeigt die mittlere Jahresrendite geometrisch.

**Gesamtrendite:** Alle Reihen enthalten Dividenden und Kupons (Total Return), keine Anlagekosten, keine Steuern auf Erträge.

## @f-ret-source Quelle der Renditen
Stichworte: Auswahl, fix, historisch, rolling, bootstrap, parametrisch

Wählen Sie die Quelle mit den Karten oben im Abschnitt. Jede Karte hat ein ⓘ mit der ausführlichen Erklärung:

- [Fixe Renditen](#f-ret-fixed)
- [Historisch ab Startzeitpunkt](#f-ret-historical)
- [Alle historischen Startzeitpunkte](#f-ret-rolling)
- [Monte Carlo · historisch](#f-ret-bootstrap)
- [Monte Carlo · parametrisch](#f-ret-parametric)

Wenn Sie nicht sicher sind, welche passt, lesen Sie [Welche Quelle passt?](#ret-which).

## @f-ret-fixed Fixe Renditen
Stichworte: konstant, Aktien, Obligationen, Inflation, SARON, Einzellauf

Ein einziger Lauf mit **konstanten Jahreswerten** für Aktien, Obligationen, Inflation und SARON (für SARON-Hypotheken).

- Es gibt **keine Streuung**: Das Ergebnis ist «Geld reicht» oder «reicht nicht». Eine Erfolgswahrscheinlichkeit gibt es nicht.
- Gut zum Eingeben, für schnelle Vergleiche und für die Vorschau rechts. **Nicht geeignet**, um zu beurteilen, ob der Plan sicher ist: Er ignoriert das Sequenzrisiko vollständig.
- **Wahl der Werte:** Rechnen Sie mit vorsichtigen, **realistischen** Werten, nicht mit den Spitzenjahren der Vergangenheit. Eine nüchterne Annahme wäre zum Beispiel 4 % bis 5 % für Aktien, 1 % bis 2 % für Obligationen, 1 % bis 2 % Inflation.
- Bei den Strategien «Fixe Rate», «Guardrails» und anderen zeigt ein Lauf mit fixen Renditen nur den Durchschnittsfall.

Bei den anderen Quellen gelten die fixen Werte teilweise trotzdem: für den **SARON** ausserhalb des Zeitraums der Reihe und beim [parametrischen Modell](#f-ret-parametric) und nach dem Ende der Reihe bei der [Fortschreibung](#f-ret-extrapolation).

## @f-ret-historical Historisch ab Startzeitpunkt
Stichworte: Rückblick, Startmonat, 1973, 1929, Backtest, Zeitreihe

Ein einziger Lauf mit dem **echten Renditeverlauf** der [Anlagestrategie](#f-ret-series), beginnend im **historischen Startmonat**. Die Simulation startet dann so, als wären Sie in diesem Monat gestartet.

- **Beispiel:** Startmonat 01.1973 prüft, was passiert wäre, wenn Sie mit Ihrer Planung zu Beginn der Ölkrise in Pension gegangen wären (hohe Inflation, schwache Börse).
- Die Simulation läuft über den Zeitraum der Planung, zum Beispiel 32 Jahre ab dem Startmonat. Endet die Reihe früher, greift die [Fortschreibung](#f-ret-extrapolation).
- Bei jährlichen Strategien (alles ausser S&P 500 mit US-Obligationen bzw. mit Obligationen der Eidgenossenschaft) zählt nur das Jahr.
- **Einzelne Läufe sind Anekdoten.** Ein Plan, der 1973 funktioniert hätte, kann in einer anderen Lage scheitern. Für eine Aussage über die Sicherheit nehmen Sie [alle Startzeitpunkte](#f-ret-rolling) oder [Monte Carlo](#f-ret-bootstrap).
- Mit einer [Variation des Startmonats](#f-sweep) lässt sich eine ganze Reihe historischer Starts durchrechnen. Das ist ähnlich wie «alle Startzeitpunkte», aber mit selbst gewähltem Raster.

## @f-ret-rolling Alle historischen Startzeitpunkte
Stichworte: rollierend, Rolling, Iteration, Überlappung, Startzeitpunkt

Die App rechnet **jeden möglichen Startmonat**, für den die Reihe den ganzen Planungszeitraum abdeckt. Bei einer Reihe ab 1922 und einer Planung über 32 Jahre sind das einige hundert Läufe: Start Januar 1922, Februar 1922 und so weiter bis zum letzten Monat, der noch 32 Jahre bis zum Ende der Reihe lässt.

**Stärken:** Jeder Lauf ist ein **realer** Zeitabschnitt mit echten Zusammenhängen von Aktien, Obligationen und Inflation. Sie sehen die historisch schlimmsten Starts (zum Beispiel 1929, 1966, 1973, 2000).

**Schwächen, die Sie kennen sollten:**

- Die Läufe **überlappen stark**. Aus 104 Jahren Daten ergeben sich 800 Monate Startpunkte, aber nur gut **drei wirklich unabhängige 32-Jahres-Abschnitte**. Die Streuung wird deshalb **unterschätzt**, und «98 % der Läufe sind erfolgreich» klingt sicherer, als es ist.
- Es gibt nur **die eine** Vergangenheit. Eine schlimmere Abfolge, die nie eingetreten ist, kommt nicht vor.
- Bei langen Planungszeiträumen und kurzen Reihen gibt es sehr wenige Startpunkte. Mit einer Jahresreihe (zum Beispiel «Schweiz (JST)», 121 Jahre) und einer 32-Jahres-Planung sind es nur rund 90.

Der [Bootstrap](#f-ret-bootstrap) behebt diese Schwächen teilweise und zeigt deshalb breitere Bandbreiten. Nehmen Sie «Alle Startzeitpunkte» als Gegenprobe, nicht als einzige Quelle.

Die **Anzahl** der Läufe zeigt der Abschnitt «Durchlauf» an.

## @f-ret-bootstrap Monte Carlo · historisch (Bootstrap)
Stichworte: Bootstrap, Blocklänge, Pfade, Zufallsstartwert, Seed, stationär, Monte Carlo

Der Bootstrap erzeugt **tausende neue Renditepfade**, indem er zufällige Abschnitte der Zeitreihe der Anlagestrategie aneinanderhängt. So entstehen Verläufe, die es nie gab, aber aus **echten Monatsdaten** bestehen.

**So funktioniert es:**

1. Der Pfad beginnt bei einem zufälligen Monat der Reihe.
2. In jedem Monat endet der aktuelle **Block** mit der Wahrscheinlichkeit 1 / Blocklänge. Dann beginnt ein neuer Block bei einem zufälligen Startmonat. Am Ende der Reihe geht es am Anfang weiter.
3. Aktien, Obligationen, Inflation und SARON stammen immer **aus demselben historischen Monat**. Die Zusammenhänge (zum Beispiel Börsenkrise mit hoher Inflation) bleiben erhalten.

**Eingaben:**

- **Anzahl Pfade:** Mehr Pfade geben glattere Zahlen, aber längere Rechenzeit. 10 000 genügen für eine erste Antwort, 50 000 für Perzentile am Rand (1. Perzentil, schlechtester Lauf). Die Erfolgsquote ist ein Zufallsergebnis: Bei 10 000 Pfaden ist sie auf etwa ±0.4 Prozentpunkte genau, bei 50 000 auf etwa ±0.2 (ungefähr zwei Standardfehler bei einer Erfolgsquote um 95 %).
- **Mittlere Blocklänge (Monate):** Wie lange zusammenhängende Abschnitte im Mittel sind.
  - **Kurz (1 bis 6):** Die Monate werden fast unabhängig gemischt. Das zerstört die Eigenschaft, dass auf schlechte Jahre oft weitere folgen (**Momentum und Erholung**), und macht die Pfade **zu glatt**.
  - **Lang (24 und mehr):** Die Pfade bleiben nah an der Geschichte, die Vielfalt sinkt.
  - **12 Monate** (Standard) ist ein bewährter Kompromiss. Bei Jahresreihen gilt die Länge in Jahren (Blocklänge / 12).
- **Zufallsstartwert (Seed):** Gleicher Wert, gleiche Pfade. Siehe [Zufallsstartwert](#ret-seed).

**Eigenschaften:**

- Die Pfade streuen **breiter** als bei «Alle Startzeitpunkte», weil sie nicht an die eine Reihenfolge der Geschichte gebunden sind. Das ist die ehrlichere Unsicherheit.
- Der Bootstrap kann nur zeigen, was **in den Daten steckt**. Hohe Renditen der Vergangenheit führen zu hohen Renditen in der Zukunft. Wollen Sie vorsichtiger rechnen, nehmen Sie die [parametrische Quelle](#f-ret-parametric) mit tieferem μ oder eine kürzere Reihe, die nicht nur Boomjahre enthält.
- Die Detailansichten (5., 50., 95. Perzentil und die Randläufe) rechnen den jeweiligen Pfad exakt nach.

## @f-ret-parametric Monte Carlo · parametrisch
Stichworte: Student-t, Fat Tails, Volatilität, Drift, Kurssprünge, mu, sigma, Freiheitsgrade, Poisson, Crash, Vorschlagswerte, Korrelation, Obligationen, Inflation

Ein **Aktienmodell mit eigenen oder vorgeschlagenen Annahmen**. Die Aktienkurse folgen einer Zufallsbewegung in **Tagesschritten** (21 pro Monat), ergänzt um seltene **Kurssprünge**. **Obligationen und Inflation** sind ebenfalls zufällig und mit den Aktien **korreliert**, so wie in der Reihe der [Anlagestrategie](#f-ret-series). Der SARON gilt konstant als fixer Wert.

| Eingabe | Vorschlag (Beispiel Welt (JST)) | Bedeutung |
|---|---|---|
| Anzahl Pfade | 50 000 | wie beim Bootstrap |
| Erwartete Rendite (Drift μ) | 12.15 % | Drift der Log-Rendite plus σ²/2; die mittlere (arithmetische) Jahresrendite ohne Sprünge |
| Volatilität σ | 12.25 % | Schwankung der Jahresrenditen ohne die Sprünge |
| Freiheitsgrade | 6 | Dicke der Ränder («Fat Tails»), 0 = Normalverteilung |
| Kurssprünge pro Jahr | 0.15 | Wie oft ein Crash vorkommt (0.15 = etwa alle 7 Jahre im Mittel) |
| Mittlere Sprunghöhe (log) | −25 % | Grösse des Sprungs; −25 % in Log-Rendite sind rund −22 % Kursverlust |
| Streuung der Sprunghöhe | 15 % | Wie unterschiedlich die Sprünge sind |
| Zufallsstartwert | | siehe [Zufallsstartwert](#ret-seed) |

**Vorschlagswerte je Aktienindex:** Die App berechnet die Parameter aus der ganzen Aktienreihe des gewählten Index, so dass die Jahresrenditen des Modells (Mittelwert, Streuung, geometrisches Mittel und 5 %-Quantil) denen des [historischen Bootstraps](#f-ret-bootstrap) entsprechen. Freiheitsgrade (6) und Streuung der Sprunghöhe (15 %) bleiben fest.

| Aktienindex | μ | σ | Sprünge pro Jahr | Sprunghöhe | Jahre |
|---|---|---|---|---|---|
| Welt (JST) | 12.15 % | 12.25 % | 0.15 | −25 % | 1900–2020 |
| Schweiz (JST) | 10.55 % | 14.5 % | 0.26 | −10 % | 1900–2020 |
| S&P 500 (Shiller) | 15.05 % | 16.5 % | 0.26 | −20 % | 1922–2025 |

«Welt individuell» erhält Werte aus seiner eigenen, gewichteten Reihe. Nach einem Update der Shiller-Daten ändern sich die Werte für den S&P 500 mit. Beim S&P 500 zieht der Bootstrap Monate; seine Jahre sind dadurch etwas milder als die Kalenderjahre, und die Vorschlagswerte folgen dem Bootstrap.

- **Neue Planung:** Die Werte folgen dem Aktienindex. Wechseln Sie den Index unter «Vermögen & Anlage», gelten die Vorschlagswerte des neuen Index.
- **Eigene Werte:** Sobald Sie einen der sechs Modellwerte ändern, bleiben Ihre Werte, auch bei einem Wechsel des Index. Neben jedem Feld steht dann der Vorschlag für den gewählten Index. Mit **«Vorschlagswerte übernehmen»** folgen die Werte wieder dem Index.
- **Ältere Planungen**, die schon parametrisch rechneten, behalten ihre Werte. Beim Öffnen erscheint einmal ein Hinweis, dass Obligationen und Inflation neu zufällig sind.

**Obligationen und Inflation:** Die Karte «Obligationen und Inflation» zeigt Mittelwert, Streuung und die Korrelation von Aktien, Obligationen und Inflation, berechnet aus den Jahreswerten derselben Reihe, die auch der Bootstrap verwendet. Sie hängt von der Kombination aus Aktien- und Obligationenindex ab. Beispiele für die Korrelation Aktien/Obligationen: Welt mit Eidgenossen 10 Jahre −0.01, Schweiz mit Eidgenossen 10 Jahre 0.19, S&P 500 mit USA 10 Jahre 0.45 (beide tragen den Dollarkurs). Jedes Kalenderjahr zieht das Modell einen Wert für Obligationen und Inflation, der mit der Aktienrendite desselben Jahres korreliert ist. Die Jahre sind voneinander unabhängig wie im Bootstrap, ein Gedächtnis für Inflation oder Zinsen gibt es nicht. Ohne Zeitreihe gelten für Obligationen und Inflation die fixen Werte.

**Mathematik in Kürze:** Die Log-Rendite je Tag ist (μ − σ²/2)·dt + σ·√dt·Z. Z ist ein **Student-t-verteilter** Schock, auf Varianz 1 skaliert. Dazu kommen **Poisson-Sprünge** mit der Rate «Kurssprünge pro Jahr» und log-normaler Höhe. Obligationen und Inflation eines Jahres sind normalverteilt mit Mittelwert und Streuung der Reihe; ihre Abhängigkeit von der standardisierten Log-Rendite der Aktien ist so gewählt, dass die Korrelation mit der Aktienrendite der Reihe entspricht.

**Wie Sie eigene Werte wählen:**

- **μ:** Langfristig liefern breit gestreute Aktien in der Geschichte real etwa 5 % bis 7 % pro Jahr, nominal entsprechend mehr. Wer vorsichtig plant, nimmt tiefere Werte als die Vergangenheit, weil Bewertungen heute hoch sind. Beachten Sie: Die Kurssprünge ziehen die Rendite zusätzlich nach unten; die erwartete geometrische Rendite steht in der Vorschau rechts.
- **σ:** 12 % bis 20 % ist für Aktien typisch; mit häufigen Sprüngen kleiner, weil die Sprünge einen Teil der Schwankung tragen.
- **Freiheitsgrade:** Tiefe Werte (3 bis 5) erzeugen extreme Tage, 6 ist moderat. Mit 0 verschwinden die Fat Tails, und Krisen werden unrealistisch selten.
- **Kurssprünge:** Zusammen mit den Fat Tails sorgen sie dafür, dass Crashs wie 2008 oder 2020 plausibel vorkommen. Setzen Sie die Rate auf 0, bleibt nur die Zufallsbewegung.

**Wann sinnvoll:** als Gegenprobe zum Bootstrap mit glatten Verteilungen statt der begrenzten Zahl historischer Jahre, und wenn Sie eigene Annahmen setzen wollen (zum Beispiel «künftig 4 % weniger Rendite»). **Einschränkungen:** Schweizer Aktien laufen über mehrere Jahre zum Mittel zurück; das bilden weder Bootstrap noch Modell ab. Die Korrelation ist über alle Jahre gemittelt; in Jahren hoher Inflation fallen Aktien und Obligationen historisch öfter gemeinsam. Bei «S&P 500» mit «Eidgenossenschaft 10 Jahre» deckt die Reihe nur die Jahre ab 1988 ab, die Aktienparameter aber die ganze Reihe ab 1922.

## @f-ret-series Anlagestrategie: Indizes für Aktien und Obligationen
Stichworte: Anlagestrategie, Index, Welt, JST, Pictet, Shiller, S&P 500, Gewichte, Regionen, EU, USA, Schweiz, Asien-Pazifik, Zeitreihe, SNB, Währung, kurzfristig, Geldmarkt

Die Methoden «Historisch», «Alle Startzeitpunkte» und «Bootstrap» brauchen eine Zeitreihe mit Renditen von Aktien, Obligationen und Inflation. Sie ergibt sich aus der **Anlagestrategie** im Abschnitt «Vermögen & Anlage»: je ein Index für die Aktien und für die Obligationen.

**Aktien**

| Index | Inhalt |
|---|---|
| **Welt (JST)** | Aktien von 11 Ländern (Schweiz, USA, Deutschland, Frankreich, Italien, Spanien, Niederlande, Belgien, Grossbritannien, Japan, Australien), jedes Jahr nach dem realen BIP des Vorjahres gewichtet, in CHF ohne Währungsabsicherung. **Jährlich 1900–2020.** |
| Welt individuell (JST) | Wie «Welt (JST)», aber Sie legen die Gewichte von **Schweiz, USA, EU und Asien-Pazifik** selbst fest (zusammen 100 %). Innerhalb der EU (Deutschland, Frankreich, Italien, Spanien, Niederlande, Belgien, Grossbritannien) und von Asien-Pazifik (Japan, Australien) sind die Länder nach BIP gewichtet. Ändert sich das Gewicht der EU, ändert sich der Anteil jedes EU-Landes im gleichen Verhältnis: EU 20 % statt 40 % halbiert jedes EU-Land. Die Gewichte gelten jedes Jahr neu (jährliches Rebalancing). |
| Schweiz (JST) | Schweizer Aktien, Gesamtrendite. Jährlich 1900–2020. |
| **S&P 500 (Shiller)** | US-Aktien (S&P Composite) in CHF, ohne Währungsabsicherung. **Monatlich ab 1922.** |

**Obligationen**

| Index | Inhalt |
|---|---|
| **Eidgenossenschaft 10 Jahre** | 10-jährige Obligationen der Eidgenossenschaft: JST bis 1988, danach aus den monatlichen Renditen der SNB. Jährlich 1900–2024; zusammen mit dem S&P 500 monatlich ab 02.1988. |
| USA 10 Jahre (Shiller) | 10-jährige US-Staatsanleihen in CHF, ohne Währungsabsicherung. Monatlich ab 1922. |
| Eidgenossenschaft kurzfristig | Kurzfristige Papiere der Eidgenossenschaft (Laufzeit etwa 3 Monate): JST bis 2020, danach das Jahresmittel des SARON (SNB). **Jährlich 1900–2025.** Die Rendite entspricht praktisch dem Geldmarktsatz: kaum Schwankung, aber auf lange Sicht deutlich weniger Ertrag als 10-jährige Obligationen. |

**Wie die Zeitreihe entsteht**

- **Monatlich** sind nur «S&P 500» mit «USA 10 Jahre» (ab 01.1922) und «S&P 500» mit «Eidgenossenschaft 10 Jahre» (02.1988 bis 07.2025).
- Alle anderen Kombinationen sind **jährlich**, sobald ein Teil jährlich ist. Monatsrenditen werden dazu zu Kalenderjahren verknüpft. Die Reihe umfasst die Jahre, für die beide Indizes Werte haben: zum Beispiel «Welt (JST)» mit «USA 10 Jahre» 1922–2020.
- Die **Teuerung** stammt aus der Quelle der Aktien (Schweizer Teuerung in allen Fällen), der **SARON** aus den SNB-Werten (bei Jahresreihen als Jahresmittel, ab 2000).
- Alle Indizes sind **in CHF**. Bei einer Planung in EUR oder USD erscheint ein Hinweis.

Unter der Auswahl stehen Zeitraum, Häufigkeit und die mittleren Jahresrenditen (geometrisch) der gewählten Strategie; das ⓘ daneben nennt Quelle und Zeitraum.

**Gemeinsamer Zeitraum:** Die mittleren Renditen gelten für die Jahre, die Aktien und Obligationen gemeinsam haben. Reicht ein Index weiter, wird er auf diesen Zeitraum gekürzt, und ein Hinweis unter der Auswahl nennt den Index, seine eigenen Jahre und den gekürzten Zeitraum. Die Simulation (Monte Carlo und historische Verläufe) nutzt nur den gemeinsamen Zeitraum. Deshalb ändert sich zum Beispiel die mittlere Rendite der Obligationen «USA 10 Jahre», wenn Sie bei den Aktien von «S&P 500» (bis 2025) auf «Schweiz (JST)» (bis 2020) wechseln: Die Obligationen zählen dann nur bis 2020.

**Wie wählen Sie?**

- **Welt (JST):** breit gestreut, lange Geschichte, auch Weltkriege und Hyperinflation. Achtung: Der Index ist eher **optimistisch**. Er liegt für 1900–2020 rund 1,3 Prozentpunkte pro Jahr über dem bekannten Weltindex von Dimson, Marsh und Staunton (5,3 % real in USD), vor allem wegen der jährlichen Gewichtung nach BIP und weil nur Länder enthalten sind, die bis heute bestehen. Siehe [Datenquellen](#ret-data).
- **S&P 500:** monatlich und damit am genauesten für Bootstrap und Rolling; enthält aber nur US-Aktien und die hohe Rendite der US-Börse.
- **Schweiz (JST):** Anlagewelt eines Schweizer Anlegers mit Heimmarkt.
- **Welt individuell:** wenn Sie Ihre eigene Aufteilung abbilden wollen, zum Beispiel 30 % Schweiz, 40 % USA, 20 % EU, 10 % Asien-Pazifik.

**Ältere Planungen:** Eine Planung mit gewählter Zeitreihe erhält beim Öffnen die passende Strategie und rechnet gleich wie vorher: «Shiller CHF» → S&P 500 mit USA 10 Jahre, «Shiller CHF + CH-Obligationen» → S&P 500 mit Eidgenossenschaft 10 Jahre. «Shiller USD» wird S&P 500 mit USA 10 Jahre in CHF; **eigene Zeitreihen** (CSV-Dateien) werden nicht mehr unterstützt und durch «Welt (JST)» mit Obligationen der Eidgenossenschaft ersetzt. **Pictet:** Der Index «Schweiz (Pictet)» wird aus Lizenzgründen nicht mehr mitgeliefert. Eine Planung mit «Pictet CH» oder mit «Schweiz (Pictet)» erhält für die Aktien «Schweiz (JST)» und für die Obligationen «Eidgenossenschaft 10 Jahre». In all diesen Fällen meldet die App das beim Öffnen, und das Ergebnis ändert sich.

**Aktualisieren:** «Shiller-Daten aktualisieren…» liest eine neue Shiller-Datei ein (.xls, .xlsx oder .ods; Download der Datei `ie_data.xls` auf shillerdata.com), «SNB-Daten aktualisieren…» neue Würfel von data.snb.ch. Die App lädt nie selbst Daten aus dem Internet, Sie laden sie von Hand. Die JST-Daten enden 2020 und sind fest eingebaut.

Weitere Hintergründe: [Datenquellen und ihre Eigenheiten](#ret-data).

## @f-ret-extrapolation Fortschreibung der Reihe
Stichworte: Ende der Reihe, Durchschnitt, fix, extrapolieren

Endet die Zeitreihe vor dem Ende der Simulation (zum Beispiel «Historisch ab 2000» mit Planung bis 2058), muss die App die fehlenden Jahre füllen. Die **Methode** legen Sie hier fest:

- **Fixe Werte:** Es gelten die fixen Renditen aus dem Abschnitt «Fixe Renditen».
- **Gleitender Durchschnitt:** Jeder fehlende Wert ist der Durchschnitt (arithmetisch) der letzten *n* Jahre. Das Fenster rutscht weiter und enthält mit der Zeit auch die bereits fortgeschriebenen Werte. Die Fortschreibung läuft darum auf einen ruhigen Mittelwert zu.
- **Gleitender geometrischer Durchschnitt:** wie oben, aber geometrisch gemittelt.

Der Zeitraum («Durchschnitt über … Jahre») gilt für die beiden gleitenden Methoden. Diese Eingabe erscheint nur, wenn die Reihe wirklich endet, bevor die Planung fertig ist.

Beachten Sie: Eine Fortschreibung ist eine **Annahme**, kein historisches Ergebnis. Bei sehr langen Fortschreibungen verliert das Ergebnis seinen historischen Charakter.

## @ret-saron SARON und Hypotheken
Stichworte: SARON, Hypothek, Zins, Marge, Zeitreihe

Der **SARON** wird nur gebraucht, wenn eine Hypothek eine SARON-Periode hat. Er stammt aus der Zeitreihe (ab 06.1999, wenn vorhanden), sonst gilt der **fixe SARON-Wert** der Planung.

- Ausserhalb des SARON-Zeitraums der Reihe, bei «Fixe Renditen», beim parametrischen Modell und nach einer Fortschreibung gilt der fixe Wert. Die Ansicht zeigt das Feld nur, wenn es gebraucht wird.
- Der Zinssatz einer SARON-Tranche ist SARON plus Marge, mit Untergrenze 0.
- Vor 1999 gibt es keinen SARON. Verwenden Sie für historische Läufe vor 1999 den fixen Wert als Annäherung.

Mehr zu Tranchen und Perioden: [Hypotheken](#f-re-mortgage).

## @ret-seed Zufallsstartwert und Reproduzierbarkeit
Stichworte: Seed, Zufall, gleiche Ergebnisse, Reproduzierbar

Monte Carlo ist Zufall, aber **reproduzierbar**. Pfad *i* entsteht immer aus dem Paar (Zufallsstartwert, *i*).

- Gleiche Einstellungen und gleicher Startwert geben **genau dieselben Ergebnisse**.
- Alle **Szenarien** verwenden dieselben Pfade. Auch bei der **Optimierung** sehen alle Kandidaten dieselben Pfade (die Prüfung am Schluss verwendet andere). So kommen Unterschiede von den Parametern, nicht vom Zufall.
- Ändern Sie den Startwert, wenn Sie prüfen wollen, ob Ihr Ergebnis vom Zufall abhängt. Schwankt die Erfolgsquote bei anderem Startwert um mehr als ein paar Zehntel Prozentpunkte, brauchen Sie mehr Pfade.

## @ret-data Datenquellen und ihre Eigenheiten
Stichworte: Shiller, SNB, Pictet, Monatsdurchschnitt, Obligationen, Währung

- **Shiller (Robert J. Shiller, «Irrational Exuberance»):** Aktien als S&P Composite mit Dividenden. Shillers Kurse sind **Monatsdurchschnitte**, Einbrüche erscheinen deshalb geglättet. Monatsrendite = Veränderung von realem Gesamtrenditeindex × CPI gegenüber dem Vormonat. Die Obligationen sind die Gesamtrendite 10-jähriger US-Staatsanleihen.
- **Umrechnung in CHF:** mit dem Monatsmittel des Wechselkurses USD/CHF der SNB, ohne Währungsabsicherung, und mit der Schweizer Teuerung (Landesindex der Konsumentenpreise gegenüber dem Vorjahresmonat). So entsteht der Zusammenhang, dass ein steigender Franken die Rendite eines Schweizer Anlegers in US-Aktien senkt.
- **Schweizer Obligationen:** Die Gesamtrendite einer 10-jährigen Bundesobligation (zu pari) wird aus der Monatsrendite der SNB berechnet: Kupon = Rendite des Vormonats, Bewertung zur neuen Rendite mit knapp 10 Jahren Restlaufzeit, dazu ein Monatskupon. Die SNB führt diese Reihe seit 07.2025 nicht mehr nach, daher endet «Shiller CHF + CH-Obligationen» dort.
- **JST (Jordà-Schularick-Taylor Macrohistory Database, Release 6):** Jahresrenditen von Aktien und Staatsanleihen je Land bis 2020, in CHF umgerechnet mit den Wechselkursen Ende Jahr. Gewicht eines Landes: reales BIP des Vorjahres (BIP pro Kopf nach Maddison × Bevölkerung). Fehlende Länderjahre fallen weg (Deutschland 1945–1946, Japan 1946–1947); Deutschland 1923 ist ausgeschlossen, weil der Wechselkurs in der Hyperinflation nicht zur Jahresrendite passt. Die Schweizer Aktien stützen sich ab 1926 auf Pictet. Teuerung: Schweizer Konsumentenpreise (Jahresmittel). **Lizenz CC BY-NC-SA 4.0:** nur nicht kommerziell, mit Quellenangabe (Jordà, Schularick, Taylor 2017; Jordà, Knoll, Kuvshinov, Schularick, Taylor 2019). Vergleich mit Dimson, Marsh, Staunton (1900–2020): Welt (JST) 6,6 % real pro Jahr in USD, DMS 5,3 %; die Länderreihen selbst stimmen überein (USA 6,7 % gegen 6,6 %).
- **Obligationen der Eidgenossenschaft vor 1989:** aus JST (Gesamtrendite); 1915 fehlt (Börse geschlossen), dort gilt der Coupon als Rendite.
- **Inflation** ist bei allen Reihen die Veränderung des Landesindexes gegenüber dem Vorjahr, so dass der Dezemberwert der Jahresteuerung entspricht.
- **Keine Kosten:** Anlagekosten, Steuern auf Erträge und Depotgebühren sind nicht abgezogen.
- **Rechte und Gewähr:** Die mitgelieferten Reihen sind aus den genannten Quellen berechnete Renditen. Sie fallen nicht unter die Lizenz des Programms (GPL v3); die Rechte an den Grunddaten liegen bei den Quellen (Robert J. Shiller, shillerdata.com; Schweizerische Nationalbank; Bundesamt für Statistik; Jordà, Schularick, Taylor, macrohistory.net, Lizenz CC BY-NC-SA 4.0). Shiller weist darauf hin, dass weder er noch die Yale University eine Gewähr für die Daten geben und dass sie keine Prognose und keine Anlageberatung sind. Das gilt auch für diese App.

## @ret-pitfalls Fallstricke bei den Renditen
Stichworte: Fehler, Vorsicht, Survivorship, Heimmarkt, Überoptimismus, Sequenzrisiko, Währung

- **Vergangenheit ist keine Prognose.** Die US-Börse war im 20. Jahrhundert ein ausserordentlich erfolgreicher Markt. Reihen mit US-Aktien enthalten diesen Erfolg und können zu optimistisch sein («Survivorship»). Das gilt auch für «Welt (JST)», das nur Länder enthält, die bis heute bestehen. Prüfen Sie den Plan auch mit tieferen Renditen (parametrisch mit μ = 4 % bis 5 %) oder mit Schweizer Aktien.
- **Überlappende Zeiträume täuschen Sicherheit vor.** Siehe [Alle historischen Startzeitpunkte](#f-ret-rolling): wenige unabhängige Abschnitte, aber hunderte Läufe.
- **Zu kurze Blocklänge im Bootstrap** macht die Pfade zu glatt. Bleiben Sie bei 12 Monaten oder mehr.
- **Sequenzrisiko:** Entscheidend ist **wann** die schlechten Jahre kommen. Ein Einbruch in den ersten Jahren der Pensionierung wirkt viel stärker als später. Fixe Renditen zeigen das nicht.
- **Währung:** Alle Indizes sind in CHF; eine Planung in EUR oder USD rechnet trotzdem mit Franken-Renditen und Schweizer Teuerung (Hinweis in der Prüfung). Auch CHF-Reihen mit ausländischen Aktien tragen Währungsrisiko. Die Franken-Aufwertung der letzten Jahrzehnte hat die Rendite eines Schweizer Anlegers in US-Aktien gesenkt.
- **Nominal und real verwechseln:** «2 Millionen am Ende» sind nominal. In heutiger Kaufkraft ist es deutlich weniger.
- **Eine einzige Quelle.** Wer nur eine Renditequelle rechnet, bekommt eine **modellabhängige** Antwort. Rechnen Sie wichtige Fragen mit Bootstrap und parametrischem Modell.
- **Die Kosten fehlen.** Echte Anlagekosten (Fonds, Depot) senken die Rendite um Zehntel bis ein Prozent pro Jahr. Ziehen Sie sie bei den fixen oder parametrischen Werten ab.
