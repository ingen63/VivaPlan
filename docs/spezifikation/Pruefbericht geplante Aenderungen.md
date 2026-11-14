# Prüfbericht zu «Pensionplaner geplannte Änderungen.md»

Stand 10.11.2026. Reihenfolge laut Vorgabe: zuerst die Issues, dann die Features; ToDo ignoriert.
Kriterien: eindeutig (E), testbar (T), vollständig (V), konsistent (K), machbar (M), essenziell (W), lösungsneutral (L), rückverfolgbar (R).
Legende: ✔ erfüllt, ✖ verletzt, ? offen.
Zeitpunkt: **Jetzt** = wird in dieser Runde bearbeitet (Vorgabe für alle Punkte ausser den folgenden), **Später** = zurückgestellt, bleibt in der Liste, wird nicht umgesetzt. Zurückgestellt: FEA-05. Gelöscht: FEA-04 (war falsch verstanden). Nichts am Code wurde geändert.

## Allgemein (gilt für alle Punkte)

- **R:** Die Punkte haben keine IDs. Vorschlag: `ISS-1`, `ISS-2`, `FEA-01` … (so nummeriert unten) und dieselbe ID in Commits, Tests und Hilfe.
- **L:** Viele Punkte schreiben die Lösung vor (MouseOver, Icon, Shift-Taste, Spalte, eigenes Fenster). Besser: Anforderung («Der Benutzer kann X erkennen») und Lösung getrennt halten.
- **T:** Es fehlen durchwegs Abnahmekriterien («Gegeben … wenn … dann …»).

## Issues

### ISS-1: Steuern 2029→2030 +1000, Einnahmen nur +200

**Entscheid 10.11.2026: False Positive, wird nicht umgesetzt.** Die Zahlen stimmen (Steuer im Mai des Folgejahres bezahlt); die Umstellung auf das Steuerjahr wird nicht verfolgt.

Nachgestellt mit einer Testplanung (feste Renditen, Release-Build, Monatswerte als CSV). Die Beträge stammten aus einer
privaten Planung und sind hier weggelassen (GEN-01).

Die Ausgangsbeobachtung stimmt: Von 2029 auf 2030 steigen die Einnahmen um wenige hundert Franken, die Steuern um
knapp tausend.

**Ursache (Befund, kein Rechenfehler):** Die Engine veranlagt im Dezember und bezahlt im Mai des Folgejahres (`crates/engine/src/sim.rs`, Abschnitt «Steuern»). Die Steuer in der Zeile 2030 gehört also zum Steuerjahr 2029, die Zeile 2029 zum Steuerjahr 2028, das wegen der Pensionierung im Lauf des Jahres ein tieferes Einkommen hat. Der Einkommensunterschied der beiden Steuerjahre mal den Grenzsteuersatz erklärt den Anstieg der Steuer.
Die Jahrestabelle stellt Einnahmen (Anfall) und Steuern (Zahlung) im selben Jahr nebeneinander. Das wirkt wie ein Fehler.

**Bewertung:** E ✖ (Soll: Steuerjahr oder Zahlungsjahr?), T ✖, V ✖, K ✖ (Anzeige mischt Perioden), R ✖ (Berechnung ist aus der Tabelle nicht nachvollziehbar).
**Vorschlag:** Entscheiden, ob Steuern nach Steuerjahr oder Zahlungsjahr gezeigt werden, und die Spaltenüberschrift entsprechend benennen. Abnahmekriterium: «Steuer des Jahres J ist aus Einkommen und Vermögen des Jahres J mit den Steuertabellen nachrechenbar».
**Wechselwirkung:** FEA-07 (Steuerspalten ausblenden) nimmt dem Benutzer genau die Spalten weg, mit denen dieser Fall erkennbar war.
**Einschränkung:** Der Benutzerplan liegt nur als Bug-Datei vor. Bootstrap-Läufe wurden nicht ausgewertet.

### ISS-2: Wohnung wird erst Jahre nach dem Hausverkauf gekauft

**Entscheid 10.11.2026, umgesetzt (e0939c1):** Datum vor Simulationsstart ist eine Warnung; ein nicht finanzierbarer Kauf nach Verkauf entfällt sofort, die Folgeregeln (Miete) greifen.

Nachgestellt mit der Bug-Datei: Haus «Lättendörfli 8» auf Verkauf 2027-08 gesetzt, Wohnung «Kauf nach Verkauf Haus» mit Modus `maxLoan`.

- Verkauf 08.2027, Kauf-Versuch 09.2027 scheitert: «Eigenmittel 800'000 nötig, vorhanden 788'362».
- Grund: Die Tragbarkeit rechnet nur mit Einkommen (Erwerb, Renten). Das Einkommen ist in diesem Plan nach der Pensionierung null, also ist die tragbare Hypothek 0 und der ganze Kaufpreis muss aus Eigenmitteln kommen. Es fehlt ein Teil der Eigenmittel.
- `attempted = true` wird gesetzt, der Kauf wird nie mehr versucht (`after_sales` in `crates/engine/src/sim.rs`). Die Wohnung wird nur noch über andere Auslöser gekauft («nach Verkauf der letzten Immobilie»), also später oder gar nicht.
- **Tranchen sind nicht die Ursache.** Sie verteilen die berechnete Hypothek nur anteilig.
- Auffällig: Ein Verkaufsdatum vor dem Simulationsstart (der Plan startet 11.2026, «Verkauf 08.2026» liegt davor) wird stillschweigend ignoriert. Der Verkauf findet dann nie statt (im Test erst als Notverkauf 2052). Das könnte die Beobachtung des Benutzers ebenfalls erklären, denn eine Warnung fehlt.

**Bewertung:** E ✖ (was heisst «ab Verkauf Haus»: sofort, oder sobald finanzierbar?), T ✖, V ✖ (Fall «nicht finanzierbar» fehlt), K ✖ (Meldung «Kauf nicht möglich» ohne Folge).
**Vorschlag:** Verhalten festlegen (Kauf-Versuch monatlich wiederholen oder Plan-Warnung «Kauf nach Verkauf nicht finanzierbar») und Validierung «Datum vor Simulationsstart» ergänzen. Abnahmetest mit genau diesem Plan.

## Features

| ID | Punkt (gekürzt) | Zeitpunkt | E | T | V | K | M | W | L | R | Befund und Vorschlag |
|----|-----------------|-----------|---|---|---|---|---|---|---|---|----------------------|
| FEA-01 | Zeitleiste-Ereignisse grün beschriften | Jetzt | ✖ | ✖ | ✖ | ? | ✔ | ? | ✖ | ✖ | Welche Ereignisse, welcher Text, nur Farbe? «Grün» ist Lösung. Ziel klären: «Ereignisse der Zeitleiste tragen eine Beschriftung». |
| FEA-02 | App öffnet mit letzter Planung | Jetzt | ✔ | ✔ | ✖ | ✔ | ✔ | ✔ | ✔ | ✖ | Fehlen: Erststart, Datei verschoben/gelöscht, ungespeicherte Änderungen, mehrere Fenster. Datenschutz passt (nur lokaler Pfad). |
| FEA-03 | Parameter im Ergebnis-Tab read-only, «Fenster oder Vorschlag» | Jetzt | ✖ | ✖ | ✖ | ✔ | ✔ | ✔ | ✖ | ✖ | Der Punkt lässt die Wahl offen. Entscheiden: Was ist der Zweck (Ergebnis einer Parameterlage zuordnen)? Dann Umfang (alle Parameter oder Zusammenfassung?) festlegen. |
| FEA-04 | Perzentile im Vergleichsdiagramm wählbar | gelöscht | ✖ | ✖ | ✖ | ✔ | ✔ | ✔ | ✖ | ✖ | «Einzellauf Percentile» ist unklar. Welche Perzentile (1/5/25/50/75/95/99)? «Shift» ist Lösung. Der Hover soll alle dargestellten Linien zeigen. |
| FEA-05 | Guardrails + 3-Topf gemeinsam | **Später** | ✖ | ✖ | ✖ | ✖ | ? | ✔ | ✔ | ✖ | **Grösstes Risiko.** Heute sind Strategien exklusiv (Entnahme- und Optimierungsmodell). Die 3-Topf-Spezifikation leitet die Topfziele vom geplanten Netto-Bedarf ab. Wenn Guardrails die Ausgaben verändern, ist der Netto-Bedarf neu zu definieren. Folgen für Optimierung, Variationen, Hilfe. Eigene Spezifikation nötig (Reihenfolge, Zuständigkeit, Steuer). |
| FEA-06 | Spalte Entnahmerate = Entnahme/Vermögen Total | Jetzt | ✖ | ✖ | ✖ | ✔ | ✔ | ✔ | ✔ | ✖ | «Entnahme» (Ausgaben? Netto-Entnahme aus dem Portfolio? inkl. Steuern, Miete?) und «Vermögen Total» (die Tabelle enthält Immobilien und PK) sind nicht definiert. Zeitpunkt (Jahresanfang oder -ende) fehlt. Formel eindeutig machen. |
| FEA-07 | Steuerspalten nicht anzeigen | Jetzt | ✖ | ✔ | ✖ | ✖ | ✔ | ✖ | ✔ | ✖ | Welche Tabellen? Widerspruch zu ISS-1 (Nachvollziehbarkeit). Sinnvoll wäre «standardmässig eingeklappt». |
| FEA-08 | Tabelle «3-Topf: Herleitung» entfernen | Jetzt | ✔ | ✔ | ✔ | ✖ | ✔ | ✔ | ✔ | ✖ | Widerspricht der Spezifikation und der Hilfe zur 3-Topf-Strategie, die die Nachvollziehbarkeit der Töpfe verlangen. Dort ebenfalls anpassen und UI-Tests nachführen. |
| FEA-09 | Keine 3-Topf-Events in der Ereignisliste | Jetzt | ✔ | ✔ | ? | ✖ | ✔ | ✔ | ✔ | ✖ | Zusammen mit FEA-08 geht die Nachvollziehbarkeit der 3-Topf-Buchungen verloren (Spezifikation fordert sie). «Alle 3-Topf-Ereignisse» definieren (Auffüllungen A/B/C, Zuflüsse). |
| FEA-10 | Gespeicherte Ergebnisse: Strategie-MouseOver zeigt Parameter | Jetzt | ✔ | ✔ | ✖ | ✔ | ✔ | ✔ | ✖ | ✖ | Welche Parameter pro Strategie? Auf Touch/Tastatur unzugänglich. |
| FEA-11 | Spalte «Zeitreihe» ergänzen, «Art» entfernen | Jetzt | ✖ | ✔ | ✖ | ✖ | ✔ | ? | ✔ | ✖ | Was zeigt die Spalte bei festen Renditen oder parametrischen/Bootstrap-Quellen (dort gibt es keine Zeitreihe in diesem Sinn)? Mit «Art» geht Information verloren: Ist sie anderswo sichtbar? |
| FEA-12 | «5 % Vermögen» statt Median, Icon bei ungeplantem Immobilienverkauf | Jetzt | ✖ | ✖ | ✖ | ✖ | ✔ | ✔ | ✖ | ✖ | Die Tabelle hat bereits eine Spalte «5. Perzentil» (Redundanz). Einzelläufe haben keinen Median. «Ungeplant» = Notverkauf in welchem Lauf: im Perzentil-Lauf, in irgendeinem, in mehr als x %? Definition nötig. Icon ist Lösung. |
| FEA-13 | Pflichteingaben prüfen, Liste erstellen | Jetzt | ✔ | ✔ | ✔ | ✔ | ✔ | ✔ | ✔ | ✖ | Auftrag an mich, noch nicht erledigt (siehe unten). Danach je Pflichtfeld Fehlermeldung und Test. |
| FEA-14 | Meilensteine: Spalte «Verwendet» weglassen | Jetzt | ✔ | ✔ | ✔ | ✖ | ✔ | ✖ | ✔ | ✖ | «Verwendet» zeigt, was an einem Meilenstein hängt. Das ist für «verschiebt man die Pensionierung, verschiebt sich alles» wichtig. Ersatz sicherstellen (zum Beispiel Warnung beim Löschen). |
| FEA-15 | «Altersguthaben heute» → «… Datum Simulationsstart» | Jetzt | ✖ | ✔ | ✔ | ✔ | ✔ | ✔ | ✔ | ✖ | Wörtlich oder mit eingesetztem Datum (zum Beispiel «11.2026»)? Ändert sich das Datum, ändert sich der Titel. Gilt auch für Bestände (siehe FEA-16). |
| FEA-16 | «Bestände heute» → «Bestände Datum Simulationsstart» | Jetzt | ✖ | ✔ | ✔ | ✔ | ✔ | ✔ | ✔ | ✖ | Wie FEA-15. Zusammenlegen. |
| FEA-17 | Cash-Reserve nur für Strategie «Standard» | Jetzt | ✔ | ✔ | ✖ | ✖ | ? | ✔ | ✔ | ✖ | Heute Parameter unter «Vermögen & Anlage» (`wealth.reserveMonths`). Verwenden andere Strategien (3-Topf, VPW) ihn mit? Prüfen. Verhalten bestehender Plandateien mit gesetztem Wert festlegen. Die Optimierung kennt `standard.reserveMonths` bereits. |
| FEA-18 | Renditen/Zeitreihen: Erklärungen in MouseOver | Jetzt | ✔ | ✔ | ✖ | ✔ | ✔ | ? | ✖ | ✖ | Lösung statt Anforderung. Auf Touch/Tastatur unzugänglich; Hilfetexte (`crates/app/ui/hilfe`) bleiben als Ort. |
| FEA-19 | Optimierungsziele wählbar (1 wie jetzt; 2 Immobilie nie ungeplant verkaufen, kleinste Ruinwahrscheinlichkeit; 3 «später») | Jetzt | ✖ | ✖ | ✖ | ✔ | ? | ✔ | ✔ | ✖ | Ziel 1 verweist auf «wie jetzt» statt auf eine Beschreibung. Ziel 2: «nicht ungeplant verkaufen» als harte Bedingung (0 Läufe?) oder Grenzwert (x %)? Bei Monte Carlo oft unerfüllbar, dann Verhalten festlegen. Punkt 3 («später») ist keine Anforderung, streichen. |
| FEA-20 | Variationen: Maximum aufheben, Warnung bei erwarteter Rechenzeit > 1 min | Jetzt | ✖ | ✖ | ✖ | ✔ | ✖ | ✔ | ✖ | ✖ | Heute `MAX_COMBINATIONS` (`crates/engine/src/scenarios.rs`). «Erwartete Rechenzeit» ist rechnerabhängig und braucht eine Schätzformel. «Unbegrenzt» ist wegen des Speichers (Ergebnisse je Kombination) nicht machbar. Besser: hohe Obergrenze plus Warnung plus Abbruch. |
| FEA-21 | Gestrichelte Linien (1./99. Perzentil) in die Legende | Jetzt | ✔ | ✔ | ✔ | ✔ | ✔ | ✔ | ✔ | ✖ | Klar. Nur Ort («Übersicht und Grafiken») in beiden Stellen testen. |
| FEA-22 | Übersicht: «Einmalzahlung» weg; «Notverkäufe» → «Immobilienverkäufe» mit Datum aller Verkäufe | Jetzt | ✖ | ✖ | ✖ | ✖ | ✔ | ✔ | ✔ | ✖ | Umbenennen vermischt geplante und Notverkäufe und widerspricht FEA-12 (Unterscheidung «ungeplant»). Welches Datum bei vielen Läufen: pro Perzentil-Lauf? «Einmalzahlung» kommt an zwei Stellen vor (Kennzahl und Spalte). |
| FEA-23 | Einzelne Läufe/Jahreswerte: Variationen auswählbar | Jetzt | ✖ | ✖ | ✖ | ✔ | ✔ | ✔ | ✔ | ✖ | Unterstützt: `scenarios.select` (`crates/engine/src/api.rs`). Offen: mehrere Variationen = eine Kombination? Anzeige der Auswahl. |
| FEA-24 | Einstellungen (Schwelle Warnsymbol, aus FEA-12) | **Später** | ? | ? | ✖ | ✔ | ✔ | ✔ | ✔ | ✔ | Platzhalter, wird später spezifiziert. FEA-12 nutzt bis dahin fest 5 %. |

### Zusammenfassung der Features

- **Widersprüche, vor der Umsetzung klären:** FEA-05 (gegenüber heutigem Modell), FEA-07 mit ISS-1, FEA-08/09 mit der 3-Topf-Spezifikation, FEA-12 mit FEA-22, FEA-14.
- **Nicht machbar wie geschrieben:** FEA-20 («unbegrenzt»).
- **Sofort umsetzbar:** FEA-02, FEA-10, FEA-14 (nach Entscheid), FEA-21, FEA-23, FEA-15/16 (nach Wortlaut), FEA-18.
- **FEA-13** ist unten erledigt (Pflichteingaben).

## FEA-13: Pflichteingaben für eine Simulation

Quelle: `crates/engine/src/model.rs` (Felder ohne Standardwert, sonst lässt sich die Plandatei nicht laden) und `crates/engine/src/validate.rs` (Fehler, die den Lauf verhindern). Warnungen sind nicht aufgeführt. Die Oberfläche selbst habe ich nicht geprüft; ob sie die Felder als Pflicht markiert, ist offen.

**Immer nötig**

| Bereich | Eingabe | Prüfung heute |
|---------|---------|---------------|
| Haushalt | mindestens 1, höchstens 2 Personen | Fehler bei 0 oder mehr als 2 |
| Person | Geburtsdatum (Monat/Jahr) | Fehler, wenn nach Simulationsstart oder Alter über der Grenze am Ende; fehlt es, lässt sich die Datei nicht laden |
| Person | **Name** | nur zur Anzeige, **keine Prüfung** (leerer Name erlaubt) |
| Haushalt | Plafonierung AHV-Rente (> 0) | Fehler |
| Meilensteine | `start` und `end`, Ende nach Start, Zeitraum höchstens 100 Jahre | Fehler |
| Meilensteine | Altersangaben 0–130 | Fehler |
| Vorsorge | Pensionskasse: Kapital, Umwandlungssatz, Kapitalbezug 0–100 % | Kapitalbezug geprüft |
| Vorsorge | AHV-Rente (Betrag, Beginn) | muss im Plan vorhanden sein, Wert nicht geprüft |
| Einnahmen/Ausgaben | Einkommen, Ausgaben, Sparen als Reihe mit gültigen Zeitpunkten | Fehler bei ungültigem Zeitpunkt, leerer Reihe oder Wert ausserhalb des Bereichs |
| Vermögen | Aktien, Obligationen, Cash, Aktienanteil | muss vorhanden sein |
| Renditen | Quelle (fest, historisch, Bootstrap, parametrisch …) | Fehler je nach Quelle (siehe unten) |
| Steuern | Tabellen für Einkommen, Vermögen, Kapital und Steuerfuss | muss vorhanden sein |

**Abhängig von der Auswahl**

| Wenn gewählt | Dann nötig |
|--------------|------------|
| Renditequelle historisch | Zeitreihe und Startmonat |
| Bootstrap (Monte Carlo aus Historie) | Zeitreihe, Pfade 1–1'000'000, Blocklänge ≥ 1 |
| Parametrisch | Pfade 1–1'000'000, Volatilität und Sprungrate ≥ 0, Freiheitsgrade > 2 (0 = Normalverteilung) |
| Entnahme 3-Topf | Töpfe ≥ 0 |
| Entnahme feste Rate / VPW / Endowment / Guardrails | Raten 0–100 %, VPW Planungsalter und Realrendite, Guardrails Leitplanken und Anpassung, Untergrenze ≤ Obergrenze |
| Immobilie im Besitz | je Hypothekentranche mindestens eine Periode, Beginn der laufenden Periode, Laufzeit 0–100 Jahre; Hypothek ≤ Kaufpreis |
| Immobilie geplant | Kaufzeitpunkt (Datum, Meilenstein oder «nach Verkauf von …» mit existierender Immobilie), Kennung eindeutig |
| Miete | Mietbeginn und Miete |
| Säule 3a bei Immobilie | Zins −50 % bis 100 %, Bezug spätestens 5 Jahre nach ordentlichem Rentenalter |
| Grundstückgewinnsteuer | gültige Besitzdauer-Staffel (Modell Aargau: Satz je Besitzdauer) |
| Variationen | gültiger Parameter, Wertebereich und Schritt |

**Lücken (Vorschlag)**

1. Name der Person wird nicht geprüft.
2. Ein Verkaufs- oder Kaufdatum vor dem Simulationsstart wird nicht beanstandet und stillschweigend ignoriert (siehe ISS-2).
3. Die Meilensteine `retire` und `ahv` sind für viele Eingaben Bezugspunkte, aber nicht zwingend. Fehlt ein referenzierter Meilenstein, meldet die Validierung einen Fehler beim Verweis, nicht beim Meilenstein.
4. Die Oberfläche sollte jedes Pflichtfeld markieren und die Fehlermeldung beim Feld zeigen (Abnahmetest je Zeile der Tabelle).
