# FEA-54, FEA-55, FEA-56 Einstieg für neue Benutzer

Issues #70, #71, #72. Drei Hilfen, damit neue Benutzer die wichtigsten Funktionen kennenlernen.

## FEA-54 Willkommensdialog

- **Erster Start:** `startup_plan` liefert `first: true`, wenn es noch keine `einstellungen.json` gibt und keine Datei
  übergeben wurde (Doppelklick, Kommandozeile). Die Oberfläche merkt sich sofort `welcomeShown: true`; damit entsteht
  die Datei, und der Dialog kommt nicht wieder, auch nicht, wenn die App ohne Wahl beendet wird.
- **Wahl:**

| Knopf | Wirkung |
|---|---|
| Beispiel erkunden | Lädt die Beispielplanung (`example_plan`) und öffnet die Hilfe `@start`. |
| Eigene Planung beginnen | Lädt die Beispielplanung, öffnet «Haushalt & Projekt», blendet die Checkliste ein und setzt sie zurück. |
| Leere Planung | Wie Datei → Neu. |
| Esc, Schliessen | Keine Änderung, keine Hilfe. Beim ersten Start ist das die Beispielplanung. |

- Vor dem Laden fragt die App bei ungespeicherten Änderungen nach (wie Datei → Neu).
- **Beispielplanung:** Die Desktop-App öffnet beim Start ohne gemerkte Datei die mitgelieferte, fast leere
  `Neu.vplan`. Der Dialog lädt für «Beispiel erkunden» und «Eigene Planung beginnen» die ausgefüllte Beispielplanung
  (`beispiel.vplan`, Befehl `example_plan`), damit es etwas zu erkunden gibt.
- **Menü:** *Hilfe → Willkommen …* (Kennung `welcome`).
- **Testzugang im Browser:** `#welcome`.

## FEA-55 Checkliste «Erste Schritte»

- Ort: rechte Spalte des Editors, zwischen den Kennzahlen der Vorschau und der Prüfung.
- Schritte (Hilfethema in Klammern): Haushalt & Projekt (`s-general`), Meilensteine (`s-milestones`), Vorsorge
  (`s-pension`), Einnahmen & Ausgaben (`s-flows`), Vermögen & Anlage (`s-wealth`), Simulation (`s-returns`), Steuern
  (`s-taxes`), Simulation starten (`quickstart`). Der Schnellstart nennt seit FEA-53 «Simulation» statt
  «Renditen & Daten».
- **Erledigt:** Jede Änderung im Editor (`markDirty`) hakt den gerade offenen Abschnitt ab, wenn er ein Schritt ist.
  Von Hand abhaken und wieder lösen geht mit dem Kästchen. «Simulation starten» ist erledigt nach einer Simulation ohne
  Abbruch und ohne Fehler. Rückgängig nimmt keinen Haken weg.
- **Alle erledigt:** einmal der Hinweis «Alle ersten Schritte erledigt …», danach ist die Liste zugeklappt.
- **Ausblenden:** ✕; wieder einblenden mit *Hilfe → Erste Schritte* (Kennung `steps`).
- **Speicherung:** Einstellung `firstSteps = { done: [...], hidden, complete }` in `einstellungen.json` (im Browser
  im Browserspeicher). Die Planungsdatei bleibt unverändert.

## FEA-56 Ergebnis in Worten

Karte `#verdict` zuoberst in der Übersicht, gebildet in der Oberfläche aus den Kennzahlen; der Rechenkern bleibt
unverändert.

| Fall | Text |
|---|---|
| Mehrere Läufe | «In {Erfolgsquote} ({Einordnung}) der Läufe reicht das Vermögen bis {Ende} (Alter {Alter}).» |
| … mit Erschöpfung | zusätzlich «Im ungünstigsten Lauf ist das Geld mit Alter {Alter} aufgebraucht.» (wie «Früheste Erschöpfung») |
| Ein Lauf, reicht | «Das Vermögen reicht bis zum Ende der Planung und beträgt dann {Betrag}.» (mit «in heutiger Kaufkraft») |
| Ein Lauf, aufgebraucht | «Das Vermögen ist im {MM.JJJJ} (Alter {Alter}) aufgebraucht.» |

- **Einordnung:** auf 0,1 % gerundete Erfolgsquote (wie angezeigt); ab 95 % «hoch», ab 80 % «mittel», sonst «tief».
  Sie verweist auf die Hilfe `@r-overview`.
- **Nächster Schritt:** Einordnung nicht «hoch» oder Vermögen aufgebraucht: Optimierung (`@s-optimize`), sonst
  Szenarien (`@s-scenarios`).
- Gilt auch für gespeicherte Ergebnisse aus der Liste.
