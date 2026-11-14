# Hilfe: So funktioniert die 3-Topf-Strategie

Die 3-Topf-Strategie ist eine Methode, um im Ruhestand regelmässig Geld aus dem Vermögen zu entnehmen, **ohne in schlechten Börsenjahren Aktien verkaufen zu müssen**. Dieses Kapitel erklärt, wie die Simulation die Töpfe berechnet, wann sie Geld zwischen den Töpfen verschiebt und wie Sie die Ergebnisse lesen.

---

## 1. Die Grundidee

Ihr liquides Vermögen wird auf drei Töpfe mit unterschiedlichem Zweck verteilt:

| Topf | Anlage | Zweck |
|---|---|---|
| **Topf 1** | Cash | Ihr "Haushaltskonto". Alle Ausgaben werden daraus bezahlt, alle Renten und Löhne fliessen hinein. |
| **Topf 2** | Obligationen | Puffer. Füllt Topf 1 nach, ohne dass Aktien verkauft werden müssen. |
| **Topf 3** | Aktien | Langfristiges Wachstum. Wird nur verkauft, wenn die Börse gut gelaufen ist – oder wenn es nicht anders geht. |

Das Geld fliesst grundsätzlich in eine Richtung: **Topf 3 → Topf 2 → Topf 1 → Ausgaben.**

Der Vorteil: Fällt die Börse, leben Sie einige Jahre aus Topf 1 und Topf 2 und geben den Aktien Zeit, sich zu erholen. Sie verkaufen Aktien bevorzugt nach guten Jahren.

---

## 2. Wie gross werden die Töpfe?

### 2.1 Der Netto-Bedarf

Die Grösse der Töpfe richtet sich nach Ihrem **Netto-Bedarf**, also dem Betrag, den Ihr Vermögen pro Jahr tatsächlich beisteuern muss.

```
Netto-Bedarf = (Lebenshaltung + Wohnen + Einzahlung Säule 3a + Einkommenssteuer)
             − (AHV-Rente + PK-Rente + Erwerbseinkommen)
```

Verwendet werden die **geplanten Werte des laufenden Jahres**.

**Bewusst nicht enthalten sind:**
- Einmalige Ausgaben wie Reisen, Anschaffungen, Amortisationen, Immobilienkäufe
- Sondersteuern wie Grundstücksgewinnsteuer oder Kapitalauszahlungssteuer
- Einmalige Zuflüsse wie Kapitalbezüge, Erbschaften oder Verkaufserlöse

Diese würden die Töpfe sonst für ein ganzes Jahr verzerren. Einmalige Ausgaben werden trotzdem bezahlt (siehe Abschnitt 5).

> **Beispiel 2035:**
> Lebenshaltung 96'000 + Wohnen 12'000 + Einkommenssteuer 6'000 = 114'000
> AHV 24'000 + PK-Rente 36'000 = 60'000
> **Netto-Bedarf = 54'000 CHF** (Monatsbedarf = 4'500 CHF)

**Startjahr:** Beginnt die Entnahmephase z.B. im März, enthält das Jahr noch zwei Monate mit Lohn. Damit der Netto-Bedarf nicht zu tief ausfällt, rechnet die Simulation im Startjahr nur mit den Monaten ab dem Start und rechnet diese auf 12 Monate hoch.

### 2.2 Die Zielgrössen

Sie legen in den Einstellungen fest, wie viele Jahre Netto-Bedarf jeder Topf enthalten soll:

```
Ziel Topf 1 = Jahre Topf 1 × Netto-Bedarf
Ziel Topf 2 = Jahre Topf 2 × Netto-Bedarf
Topf 3     = alles übrige
```

> **Beispiel 2035** mit Topf 1 = 2 Jahre, Topf 2 = 1 Jahr:
> Ziel Topf 1 = 2 × 54'000 = **108'000 CHF**
> Ziel Topf 2 = 1 × 54'000 = **54'000 CHF**

Fallen im selben Monat einmalige Ausgaben an, wird Ziel Topf 1 um diese erhöht, damit sie bezahlt werden können, ohne den Puffer anzugreifen.

Ist der Netto-Bedarf null oder negativ (Ihre Einnahmen decken alle laufenden Ausgaben), sind die Ziele von Topf 1 und 2 null und das Vermögen bleibt in Aktien.

---

## 3. Das Renditekriterium

Das Renditekriterium entscheidet, ob gerade ein guter Zeitpunkt ist, Aktien zu verkaufen.

- Gemessen wird die **Aktienrendite der letzten 12 Monate**.
- Ist die Simulation noch keine 12 Monate alt, wird die auf ein Jahr hochgerechnete Rendite seit Start verwendet.
- **Erfüllt**, wenn die Rendite mindestens die eingestellte Renditeschwelle erreicht.

Ist das Kriterium **nicht** erfüllt, werden **keine Aktien verkauft**, um Topf 1 oder 2 aufzufüllen. Einzahlungen in Topf 3 sind immer erlaubt.

Es gibt eine wichtige Ausnahme: Müssen Ausgaben bezahlt werden und Topf 1 und 2 reichen nicht mehr, werden auch in schlechten Zeiten Aktien verkauft – aber nur so wenig wie nötig (siehe Abschnitt 5, Fall C2).

---

## 4. Die jährliche Auffüllung im Januar

Jeden Januar werden die Töpfe wieder auf ihre Zielgrösse gebracht.

### Wenn die Börse gut gelaufen ist (Kriterium erfüllt)

1. **Topf 1 wird aus Topf 2 aufgefüllt.** Reicht Topf 2 nicht, kommt der Rest aus Topf 3.
2. **Topf 2 wird aus Topf 3 aufgefüllt.**
3. Liegt ein Topf über seinem Ziel (z.B. weil eine neue Rente den Bedarf gesenkt hat), geht der Überschuss in Topf 3.

> **Beispiel 01.2035:**
> Vorher: Topf 1 = 50'000 / Topf 2 = 52'000 / Topf 3 = 600'000
> Topf 2 → Topf 1: 52'000 (Topf 2 reicht nicht ganz)
> Topf 3 → Topf 1: 6'000
> Topf 3 → Topf 2: 54'000
> Nachher: Topf 1 = 108'000 / Topf 2 = 54'000 / Topf 3 = 540'000

### Wenn die Börse schlecht gelaufen ist (Kriterium nicht erfüllt)

1. Aus Topf 3 wird **nichts verkauft**.
2. Topf 1 wird nur aus Topf 2 aufgefüllt, so weit es reicht.
3. Topf 2 wird **nicht** nachgefüllt.

So zehren Sie in schlechten Jahren vom Obligationenpuffer und lassen die Aktien in Ruhe.

---

## 5. Was im Laufe des Jahres passieren kann

### A) Start der Entnahmephase, PK-Kapitalbezug, Neustart

Bei diesen Ereignissen werden die Töpfe sofort befüllt – unabhängig von der Börse:

- **Start der Entnahmephase:** Ihr gesamtes liquides Vermögen kommt zuerst in Topf 3, dann werden Topf 1 und 2 befüllt.
- **PK-Kapitalbezug:** Der Bezug kommt in Topf 3, dann werden Topf 1 und 2 wieder auf Ziel gebracht. Das gilt auch bei mehreren, gestaffelten Bezügen.
- **Neustart:** nach einer Defizitphase (siehe Abschnitt 7).

Weil ein Teil des Jahres schon vorbei ist, werden die Ziele **um die bereits vergangenen Monate gekürzt**. Bis zum nächsten Januar braucht es ja weniger.

> **Beispiel:** Start im März (2 Monate vergangen), Topf 1 = 2 Jahre:
> Ziel Topf 1 = 22 Monate statt 24 Monate Netto-Bedarf

### B) Externe Zuflüsse

Erbschaften, Auszahlungen aus der Säule 3a oder Freizügigkeit, Verkaufserlöse einer Immobilie (ohne Folgekauf) gehen **vollständig in Topf 3**. Topf 1 und 2 werden dabei nicht verändert. Das Geld wird im nächsten Januar oder bei Bedarf über die normale Auffüllung verteilt.

### C) Topf 1 reicht für den Monat nicht

Bevor im Monat etwas bezahlt wird, prüft die Simulation, ob Topf 1 alle Ausgaben dieses Monats deckt – laufende und einmalige. Das passiert z.B. nach einer grossen Ausgabe oder wenn Topf 2 in schlechten Jahren aufgebraucht ist. Je nach Börsenlage gibt es zwei Varianten:

#### C1) Börse gut gelaufen

Die Töpfe werden **wie im Januar** aufgefüllt, mit um die vergangenen Monate gekürzten Zielen. Eine einmalige Ausgabe in diesem Monat wird zusätzlich in Topf 1 bereitgestellt.

> **Beispiel 04.2035** (3 Monate vergangen):
> Ziel Topf 1 = 21/12 × 54'000 = 94'500 CHF **+ einmalige Ausgaben im April**
> Ziel Topf 2 = 9/12 × 54'000 = 40'500 CHF

#### C2) Börse schlecht gelaufen

Jetzt müssen die Ausgaben trotzdem bezahlt werden. Topf 1 wird darum **nur in kleinen Portionen** nachgefüllt: genug für die Ausgaben des Monats plus eine kleine Reserve von höchstens *Threshold − 1* Monaten (Einstellung, Standard 3 Monate). Das Geld kommt zuerst aus Topf 2, erst danach aus Topf 3. Topf 2 wird nicht aufgefüllt.

So werden in einer Börsenschwäche nur so wenig Aktien wie nötig verkauft.

| Monat | Monate bis Januar | Reserve bei Threshold 3 |
|---|---|---|
| Juni | 7 | 2 Monate |
| November | 2 | 1 Monat |
| Dezember | 1 | keine |

Im Dezember wird nur der Dezember gedeckt, weil im Januar ohnehin die reguläre Auffüllung folgt.

> **Beispiel 12.2035:**
> Topf 1 vor der Zahlung: 9'800, Ausgaben Dezember: 10'200
> Fehlbetrag 400 → aus Topf 3 nachgefüllt, Topf 1 = 10'200

---

## 6. Immobilien

- **Verkauf mit Kauf im Folgemonat:** Der Erlös wird für den Kauf **reserviert** und gehört keinem Topf. Beim Kauf werden die Eigenmittel aus der Reservierung bezahlt; ein Überschuss geht in Topf 3, ein fehlender Betrag wird wie eine einmalige Ausgabe über Topf 1 bezahlt.
- **Verkauf ohne Folgekauf:** Der Erlös geht in Topf 3.
- **Hypothekenerneuerung mit Einmalzahlung:** Wird wie eine einmalige Ausgabe aus Topf 1 bezahlt; Topf 1 wird vorher bei Bedarf aufgefüllt.
- Eine **Notverkaufs-Regel** der Hypothekenberechnung kann unabhängig von den Töpfen einen Verkauf auslösen.

---

## 7. Wenn das Geld ausgeht

1. Reichen alle drei Töpfe nicht mehr für die Ausgaben, wird eine **Immobilie notverkauft**. Der Erlös geht in Topf 3 und die Töpfe werden wieder aufgefüllt.
2. Ist keine Immobilie mehr vorhanden, beginnt eine **Defizitphase**: Topf 1 wird negativ (das ist der einzige Fall, in dem ein Topf unter null fällt), Topf 2 und 3 sind leer. Alle Einnahmen fliessen in Topf 1, um das Minus abzubauen.
3. Wird das Vermögen wieder positiv (z.B. durch eine Erbschaft), startet die Strategie neu wie beim Start der Entnahmephase.

---

## 8. Ablauf eines Monats

Die Simulation arbeitet jeden Monat in dieser festen Reihenfolge:

1. **Geld kommt rein:** Renten und Lohn → Topf 1. Externe Zuflüsse → Topf 3. Verkaufserlöse → Topf 3 oder Reservierung.
2. **Immobilienkauf** (falls geplant) aus der Reservierung.
3. **Auffüllung:** im Januar die Jahresauffüllung, bei Start oder PK-Kapitalbezug Fall A.
4. **Vorausschau:** Reicht Topf 1 für alle Ausgaben dieses Monats? Wenn nein: Fall C1 oder C2.
5. **Geld geht raus:** Alle Ausgaben werden aus Topf 1 bezahlt.

Dank dieser Reihenfolge fällt Topf 1 nie unter null.

---

## 9. Die Herleitung der Topfgrössen lesen

Die Herleitung steht in den **Jahreswerten**: Jedes Jahr mit mindestens einer Auffüllung hat vorn einen Pfeil (▸), der die folgende Tabelle aufklappt.

| Spalte | Bedeutung |
|---|---|
| **Zeitpunkt** | Monat und Ihr Alter |
| **Ausgelöst durch** | Januar-Auffüllung, A (Start/PK-Bezug/Neustart), B (externer Zufluss), C1/C2 (Vorausschau) |
| **Lebenshaltung, Wohnen, Einkommenssteuer** | Ausgabenbasis für den Netto-Bedarf |
| **AHV, PK-Rente, Erwerb** | Regelmässige Einnahmen, die abgezogen werden |
| **Netto-Bedarf** | Was das Vermögen pro Jahr beisteuern muss |
| **Ziel Topf 1 / Ziel Topf 2** | Zielgrössen, ggf. gekürzt um vergangene Monate |
| **Renditekriterium** | erfüllt / nicht erfüllt / ignoriert, mit Wert und Schwelle |
| **Umbuchungen** | Welche Beträge zwischen den Töpfen verschoben wurden |
| **Vorher / Nachher T1 / T2 / T3** | Stand der Töpfe vor und nach dem Ereignis |

---

## 10. Häufige Fragen

**Warum wird Topf 1 im Januar oft aus Topf 2 *und* Topf 3 aufgefüllt?**
Topf 2 wurde im Vorjahr auf genau ein Jahr Netto-Bedarf gefüllt. Durch Inflation steigt der Bedarf jedes Jahr leicht, deshalb reicht Topf 2 knapp nicht und die Differenz kommt aus Topf 3.

**Warum steht in Topf 1 nach einer Auffüllung mehr als der Netto-Bedarf?**
Topf 1 muss die *Bruttoausgaben* bezahlen. Renten fliessen monatlich hinein, Ausgaben gehen monatlich hinaus. Netto schrumpft Topf 1 pro Monat nur um den Monatsbedarf.

**Warum werden im Dezember manchmal nur wenige Franken umgebucht?**
In schlechten Börsenzeiten (C2) wird Topf 1 nur so weit gefüllt, dass der Monat gedeckt ist. Im Dezember ist das oft nur ein kleiner Fehlbetrag, weil im Januar die reguläre Auffüllung kommt.

**Warum ist Topf 2 manchmal leer?**
Nach einem oder mehreren schlechten Börsenjahren wurde Topf 1 aus Topf 2 nachgefüllt, Topf 2 selbst aber nicht aus Aktien. Das ist gewollt. Sobald die Börse wieder gut läuft, wird Topf 2 im nächsten Januar wieder aufgefüllt.

**Warum ändern sich die Ziele, wenn ich AHV oder PK-Rente beziehe?**
Höhere regelmässige Einnahmen senken den Netto-Bedarf. Dadurch werden die Ziele kleiner und der Überschuss geht im Januar zurück in Aktien.

**Welche Einstellungen beeinflussen die Strategie?**
- *Jahre Topf 1 / Jahre Topf 2*: grössere Töpfe = mehr Sicherheit in Börsenkrisen, aber weniger Aktienanteil und damit tiefere erwartete Rendite.
- *Renditeschwelle*: höhere Schwelle = Aktien werden seltener verkauft, Topf 2 wird öfter aufgebraucht.
- *Threshold*: grösser = in schlechten Zeiten werden Aktien in grösseren Portionen und seltener verkauft.
