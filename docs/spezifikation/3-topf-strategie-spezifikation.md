# Spezifikation: 3-Topf-Strategie

Diese Spezifikation ersetzt alle bisherigen Anforderungen zur 3-Topf-Strategie. Bestehende Logik, die ihr widerspricht, wird ersetzt. Unklarheiten bitte melden statt still annehmen.

---

## 0. Grundbegriffe

| Begriff | Bedeutung |
|---|---|
| **Topf 1** | Cash. **Alle Ausgaben** werden aus Topf 1 bezahlt. Regelmässige Einnahmen fliessen in Topf 1. |
| **Topf 2** | Obligationen. Puffer zum Nachfüllen von Topf 1. |
| **Topf 3** | Aktien. Langfristige Anlage. |
| **Reservierung** | Verkaufserlös einer Immobilie, der für einen Immobilienkauf im Folgemonat reserviert ist. Gehört zu keinem Topf und darf nur für diesen Kauf verwendet werden. |
| **Verfügbares Vermögen** | Topf 1 + Topf 2 + Topf 3 |
| **Gesamtvermögen** | Verfügbares Vermögen + Reservierung |
| **Netto-Bedarf** | Jahreswert, siehe Abschnitt 2 |
| **Monatsbedarf** | max(0, Netto-Bedarf / 12) |
| **Ausgaben des Monats** | Alle Ausgaben, die im Monat aus Topf 1 bezahlt werden: laufende Ausgaben **plus** alle einmaligen Ausgaben (Restbetrag Immobilienkauf, Amortisation, Einmalzahlung Hypothek, Sondersteuern, Reisen, Anschaffungen usw.) |
| **Vergangene Monate** | Anzahl Monate des laufenden Jahres vor dem aktuellen Monat. Januar = 0, März = 2, April = 3, Dezember = 11 |
| **Monate bis Januar** | 13 − aktueller Monat (aktueller Monat zählt mit). Januar = 12, November = 2, Dezember = 1 |

**Aktivierung:** Vor dem Start der Entnahmephase ist die 3-Topf-Logik inaktiv. Ein PK-Kapitalbezug vor dem Start der Entnahmephase wird normal dem liquiden Vermögen zugeschlagen.

### Einstellungen

| Einstellung | Bedeutung | Default |
|---|---|---|
| Jahre Topf 1 | Grösse Topf 1 in Jahren Netto-Bedarf | bestehend |
| Jahre Topf 2 | Grösse Topf 2 in Jahren Netto-Bedarf | bestehend |
| Renditeschwelle | Schwelle für das Renditekriterium | bestehende Einstellung |
| Threshold | Max. Monate Reserve bei Notauffüllung (C2) | 3 Monate |

---

## 1. Oberstes Kriterium

- **Kein Topf fällt jemals unter 0**, auch nicht zwischenzeitlich innerhalb eines Monats.
- Aus einem Topf wird höchstens so viel umgebucht oder bezahlt, wie er enthält.
- Dies wird durch die monatliche Vorausschau (Abschnitt 7, Sonderfall C) **vor** jeder Zahlung sichergestellt.
- **Einzige Ausnahme:** Defizitphase (Abschnitt 9).

---

## 2. Netto-Bedarf

### 2.1 Grundlage

- Grundlage sind die **geplanten Werte des laufenden Jahres**.
- **Sonderregel Startjahr / Neustartjahr:** Im Kalenderjahr des Starts der Entnahmephase und im Kalenderjahr eines Neustarts nach der Defizitphase wird der Netto-Bedarf aus den Monaten **ab dem Start bzw. Neustart** berechnet und auf 12 Monate hochgerechnet. Dieser Wert gilt für alle Berechnungen dieses Kalenderjahres.

### 2.2 Berechnung

**Ausgaben – nur diese Kategorien:**
- Lebenshaltung
- Wohnen
- Einkommenssteuer (**keine** Sondersteuern wie Grundstücksgewinnsteuer, Kapitalauszahlungssteuer, Erbschafts-/Schenkungssteuer usw.)

Alle weiteren Ausgaben (einmalige Ausgaben, Anschaffungen, Reisen, Amortisationen, Immobilienkäufe usw.) werden **nicht** berücksichtigt.

**Abzüglich regelmässige Einnahmen:**
- AHV-Rente
- PK-Rente
- Erwerbseinkommen wie angegeben

Einmalige Zuflüsse (Kapitalbezüge, 3a, Freizügigkeit, Erbschaft, Verkaufserlöse, Boni usw.) werden **nicht** abgezogen.

**Netto-Bedarf = Ausgaben (wie oben) − regelmässige Einnahmen**

Ist der Netto-Bedarf ≤ 0, sind die Basis-Zielgrössen von Topf 1 und 2 gleich 0 und der Monatsbedarf ist 0.

### 2.3 Zielgrössen

Zielgrössen werden **immer aus dem Netto-Bedarf** berechnet, nie aus dem aktuellen Topfstand.

Allgemeine Formel mit `m` = vergangene Monate (bei der Januar-Auffüllung ist m = 0):

```
Ziel Topf 1 = (Jahre Topf 1 × 12 − m) / 12 × Netto-Bedarf  +  einmalige Ausgaben dieses Monats
Ziel Topf 2 = (Jahre Topf 2 × 12 − m) / 12 × Netto-Bedarf
```

- Negative Zwischenergebnisse werden auf 0 gesetzt.
- Die einmaligen Ausgaben werden zu Ziel Topf 1 addiert, damit Topf 1 nach deren Zahlung nicht negativ wird.

---

## 3. Renditekriterium

- Gemessen wird die **Aktienrendite der letzten 12 Monate** (rollierend, auch unterjährig).
- Ist die Simulation jünger als 12 Monate: **annualisierte** Aktienrendite seit Simulationsstart.
- **Kriterium erfüllt:** Rendite ≥ Renditeschwelle.
- Ist das Kriterium nicht erfüllt, sind nur **Verkäufe aus Topf 3** gesperrt. Einzahlungen in Topf 3 sind immer erlaubt.
- Angewendet wird das Kriterium bei der Januar-Auffüllung (Abschnitt 6) und bei der Vorausschau (Sonderfall C, Auswahl C1/C2). Sonderfall A ignoriert es.

---

## 4. Standard-Auffüllung (gemeinsame Funktion)

Wird von der Januar-Auffüllung, Sonderfall A und Sonderfall C1 verwendet. Parameter: vergangene Monate `m`, Überschussbuchung ja/nein.

1. **Topf 1** bis Ziel Topf 1 aus **Topf 2** auffüllen. Reicht Topf 2 nicht, den Rest aus **Topf 3**, bis das Ziel erreicht oder Topf 3 leer ist.
2. **Topf 2** bis Ziel Topf 2 aus **Topf 3** auffüllen, bis das Ziel erreicht oder Topf 3 leer ist.
3. **Überschussbuchung** (nur wenn aktiviert, d.h. nur im Januar): Liegt Topf 1 oder Topf 2 nach Schritt 1–2 über seinem Ziel, wird der Überschuss in Topf 3 gebucht.

Durch die Reihenfolge hat Topf 1 bei knappen Mitteln automatisch Vorrang vor Topf 2.

---

## 5. Monatsablauf (verbindliche Reihenfolge)

Jeder Monat wird in genau dieser Reihenfolge abgearbeitet:

1. **Zuflüsse buchen** – direkt an ihr Ziel, nie zuerst in Topf 1 und dann umbuchen:
   - Regelmässige Einnahmen (AHV, PK-Rente, Erwerbseinkommen) → Topf 1
   - Externe Zuflüsse, 3a, Freizügigkeit → Topf 3
   - Immobilienverkauf ohne Kauf im Folgemonat, Notverkaufserlös → Topf 3
   - Immobilienverkauf mit Kauf im Folgemonat → Reservierung
   - PK-Kapitalbezug → Topf 3, Auslöser A vormerken
   - In der Defizitphase: alle Zuflüsse → Topf 1 (Abschnitt 9)
2. **Immobilienkauf** (falls geplant):
   - Eigenmittel zuerst aus der Reservierung.
   - Ein Restbetrag wird als **einmalige Ausgabe dieses Monats vorgemerkt** und **nicht** sofort aus Topf 1 bezahlt.
   - Überschuss der Reservierung → Topf 3.
3. **Auffüllung:**
   - Liegt ein Auslöser A vor → Sonderfall A (ersetzt im Januar die Januar-Auffüllung).
   - Sonst im Januar → Januar-Auffüllung (Abschnitt 6).
4. **Vorausschau:** Deckt Topf 1 die Ausgaben des Monats (inkl. aller vorgemerkten einmaligen Ausgaben)?
   - Ja → weiter mit Schritt 5.
   - Nein → Sonderfall C (C1 oder C2).
   - Reicht auch danach Topf 1 nicht (Topf 2 und 3 leer) → Abschnitt 9.
5. **Zahlung:** Alle Ausgaben des Monats aus Topf 1 bezahlen.

---

## 6. Januar-Auffüllung

Jeden Januar (sofern kein Auslöser A vorliegt), mit m = 0.

### 6.1 Renditekriterium erfüllt

Standard-Auffüllung (Abschnitt 4) **mit** Überschussbuchung:
1. Topf 1 bis Ziel aus Topf 2, Rest aus Topf 3 (bis Topf 3 leer).
2. Topf 2 bis Ziel aus Topf 3 (bis Topf 3 leer).
3. Überschüsse aus Topf 1 und 2 → Topf 3.

### 6.2 Renditekriterium nicht erfüllt

1. Aus Topf 3 wird **nichts verkauft**.
2. Topf 1 bis Ziel aus Topf 2. Reicht Topf 2 nicht, bleibt Topf 1 unter dem Ziel.
3. Topf 2 wird **nicht** nachgefüllt.
4. Überschüsse aus Topf 1 und 2 → Topf 3 (Einzahlung, erlaubt).

Reicht Topf 1 danach nicht für die Ausgaben des Januars, greift in Schritt 4 des Monatsablaufs die Vorausschau (C2).

---

## 7. Unterjährige Sonderfälle

### A) Start / PK-Kapitalbezug / Neustart

**Auslöser:**
- Start der Entnahmephase
- PK-Kapitalbezug (auch erneut, z.B. bei gestaffelten Bezügen)
- Neustart nach der Defizitphase (Abschnitt 9)

**Ablauf:**
1. Beim Start der Entnahmephase und beim Neustart: gesamtes verfügbares Vermögen in Topf 3 (Topf 1 und 2 auf 0). Beim PK-Kapitalbezug: nur der Bezug geht in Topf 3 (bereits in Schritt 1 des Monatsablaufs).
2. Standard-Auffüllung (Abschnitt 4) mit m = vergangene Monate. **Renditekriterium wird ignoriert.**
3. Überschussbuchung **nur im Januar** (m = 0), unterjährig nicht.

Fallen mehrere Auslöser in denselben Monat, läuft A nur einmal.

**Beispiel:** Start im März (m = 2), Topf 1 = 2 Jahre, Topf 2 = 1 Jahr:
- Ziel Topf 1 = 22/12 × Netto-Bedarf + einmalige Ausgaben März
- Ziel Topf 2 = 10/12 × Netto-Bedarf

### B) Externer Zufluss

**Auslöser:** Externe Zuflüsse, 3a, Freizügigkeit, Immobilienverkauf ohne Folgekauf, Überschuss einer Reservierung, Notverkaufserlös.

**Ablauf:** Alles direkt in Topf 3. Keine Auffüllung von Topf 1 oder 2.

### C) Vorausschau: Topf 1 deckt die Ausgaben des Monats nicht

**Auslöser:** In Schritt 4 des Monatsablaufs ist Topf 1 < Ausgaben des Monats (inkl. aller einmaligen Ausgaben).

#### C1) Renditekriterium erfüllt

- Standard-Auffüllung (Abschnitt 4) mit m = vergangene Monate.
- **Keine** Überschussbuchung (Töpfe über dem Ziel bleiben unverändert).

**Beispiel 04.2035** (m = 3, Topf 1 = 2 J., Topf 2 = 1 J., Netto-Bedarf 54'000):
- Ziel Topf 1 = 21/12 × 54'000 + einmalige Ausgaben April = 94'500 + einmalige Ausgaben
- Ziel Topf 2 = 9/12 × 54'000 = 40'500

#### C2) Renditekriterium nicht erfüllt

- Renditekriterium ignorieren (Topf 3 darf verkauft werden, weil die Ausgaben gedeckt werden müssen).
- Topf 1 so weit auffüllen, dass er die Ausgaben des Monats deckt **und danach** noch
  `(Min(Threshold, Monate bis Januar) − 1) × Monatsbedarf` enthält.
- Quelle: zuerst Topf 2 bis 0, danach Topf 3.
- Topf 2 wird **nicht** aufgefüllt.

**Beispiele** (Threshold 3, Monatsbedarf 4'500):
- Juni (7 Monate bis Januar): Ausgaben des Monats + 2 × 4'500
- November (2): Ausgaben des Monats + 1 × 4'500
- Dezember (1): nur Ausgaben des Monats

---

## 8. Immobilien

- **Verkauf mit Kauf im Folgemonat** (Verkauf im Monat M, Kauf im Monat M+1): Erlös → Reservierung.
- **Kauf:** Eigenmittel zuerst aus der Reservierung, Restbetrag als einmalige Ausgabe des Monats (über Vorausschau gedeckt), Überschuss der Reservierung → Topf 3.
- **Verkauf ohne Folgekauf:** Erlös → Topf 3 (Sonderfall B).
- **Verkauf im Dezember, Kauf im Januar:** Der Kauf (Schritt 2) wird vor der Januar-Auffüllung (Schritt 3) abgewickelt.
- **Hypothekenerneuerung mit Einmalzahlung** (bestehende Logik): Die Einmalzahlung ist eine einmalige Ausgabe aus Topf 1 (über Vorausschau gedeckt). Die Prüfung "Vermögen nach Zahlung ≥ 0" verwendet das verfügbare Vermögen (ohne Reservierung).
- Das **Notverkaufskriterium** der Hypotheken-Logik gilt weiterhin parallel und kann vor Abschnitt 9 auslösen.

---

## 9. Alle Töpfe leer / Defizitphase

1. Reichen Topf 1, 2 und 3 nicht für die Ausgaben des Monats: **Notverkauf einer Immobilie** (bestehende Notverkaufslogik). Erlös → Topf 3, danach Vorausschau (C) erneut.
2. Ist keine Immobilie (mehr) vorhanden → **Defizitphase**:
   - Topf 1 darf negativ werden (einzige Ausnahme zu Abschnitt 1).
   - Topf 2 und Topf 3 sind stillgelegt.
   - Alle Einnahmen und Zuflüsse fliessen in Topf 1, um das Minus abzubauen.
3. Sobald das verfügbare Vermögen wieder > 0 ist: **Neustart über Sonderfall A** (mit Hochrechnung des Netto-Bedarfs gemäss 2.1).

---

## 10. Anzeige / Protokoll

Anzeigeort: die **Jahreswerte** eines Einzellaufs. Jedes Jahr mit mindestens einer Auffüllung hat eine aufklappbare Zeile (standardmässig zugeklappt) mit den Auffüllungen dieses Jahres; die Buchungen erscheinen **nicht** in der Karte «Ereignisse». Die Meldungen der Defizitphase (Abschnitt 9) bleiben in der Ereignisliste.

Pro Auffüll-Ereignis anzeigen:
- Zeitpunkt, Alter, Auslöser (Januar, A, B, C1, C2, Notverkauf, Defizitphase)
- Ausgabenbasis (Lebenshaltung, Wohnen, Einkommenssteuer), regelmässige Einnahmen (AHV, PK-Rente, Erwerb), Netto-Bedarf, Hinweis auf Hochrechnung
- Vergangene Monate m, Zielgrössen Topf 1 und 2, einmalige Ausgaben des Monats
- Renditekriterium: Wert, Schwelle, erfüllt ja/nein bzw. ignoriert
- Bei C2: Threshold, Monate bis Januar, Reserve in Monaten
- Alle Umbuchungen inkl. Zuflüsse ("extern → Topf 3", "extern → Reservierung")
- Stand Topf 1 / 2 / 3 und Reservierung **vor** und **nach** dem Ereignis. "Vorher" ist der Stand vor jedem Zufluss.
- Ausgaben des Monats

---

## 11. Abweichungen der aktuellen Umsetzung (Stand letzter Testlauf)

Bitte zuerst analysieren und erklären, dann beheben:

1. **Topf 1 negativ vor der Vorausschau (April):** Topf 1 stand vor der Vorausschau im Minus, weil eine einmalige Ausgabe (vermutlich Immobilienkauf/Amortisation) vor der Vorausschau bezahlt wurde. → Reihenfolge gemäss Abschnitt 5 umsetzen.
2. **Zuflüsse laufen über Topf 1:** Bei zwei Immobilienverkäufen landete der Verkaufserlös zuerst in Topf 1, ebenso die Startbuchung von Topf 1 nach Topf 3 beim Simulationsstart. (Daten und Beträge des Testlaufs stammten aus einer privaten Planung und sind weggelassen, GEN-01.) → Direkt buchen gemäss Abschnitt 5, Schritt 1.
3. **Sonderfall C1 fehlt:** Bisher wird bei der Vorausschau immer die Threshold-Logik verwendet. → Aufteilung C1/C2 gemäss Abschnitt 7.
4. **Einmalige Ausgaben in Zielgrössen:** Ziel Topf 1 enthält bisher keine einmaligen Ausgaben des Monats. → Abschnitt 2.3.
5. **Überschussbuchung unterjährig:** Prüfen, ob A unterjährig Überschüsse in Topf 3 bucht. → Nur im Januar.
6. **Hochrechnung im Startjahr:** Prüfen, ob die Hochrechnung für alle Berechnungen des Startjahres gilt (nicht nur für A).

Alle weiteren Widersprüche zwischen Code und dieser Spezifikation melden.

---

## 12. Tests

Ergebnisse tabellarisch (vor/nach der Änderung) zeigen. In keinem Test (ausser Defizitphase) darf ein Topf negativ werden.

**Netto-Bedarf und Ziele**
- [ ] Ausgaben 128'000, regelmässige Einnahmen 84'000 → Netto-Bedarf 44'000, Ziel Topf 1 = 88'000, Ziel Topf 2 = 44'000 (Topf 1 = 2 J., Topf 2 = 1 J., Januar, keine Einmalausgaben)
- [ ] Grundstücksgewinnsteuer und einmalige Ausgaben verändern den Netto-Bedarf nicht
- [ ] Erwerbseinkommen senkt den Netto-Bedarf
- [ ] Netto-Bedarf ≤ 0 → Basisziele 0, Überschüsse im Januar in Topf 3
- [ ] Netto-Bedarf sinkt (z.B. AHV-Beginn) → Überschuss im Januar in Topf 3

**Januar**
- [ ] 10 Jahre fixe Rendite über der Schwelle: Ziele werden jedes Jahr erreicht, Topf 1 schrumpft nicht
- [ ] Topf 2 kleiner als Fehlbetrag Topf 1 → Rest aus Topf 3, danach Topf 2 aus Topf 3
- [ ] Topf 3 zu klein → zuerst Topf 1, dann Topf 2
- [ ] Kriterium nicht erfüllt → kein Verkauf aus Topf 3, Topf 1 nur aus Topf 2, Topf 2 nicht nachgefüllt
- [ ] Grosse Einmalausgabe im Januar → durch Januar-Auffüllung gedeckt, kein zusätzliches C

**Renditekriterium**
- [ ] Simulation jünger als 12 Monate → annualisierte Rendite seit Start
- [ ] Rolling 12 Monate auch unterjährig korrekt

**Sonderfall A**
- [ ] Start der Entnahmephase im März → m = 2, Netto-Bedarf hochgerechnet, gesamtes Vermögen zuerst in Topf 3
- [ ] PK-Kapitalbezug im Oktober → A erneut, m = 9, keine Überschussbuchung
- [ ] Start und PK-Kapitalbezug im selben Monat → A nur einmal

**Sonderfall B**
- [ ] Externer Zufluss, 3a, Freizügigkeit → direkt Topf 3, keine Auffüllung
- [ ] "Vorher" zeigt den Stand vor dem Zufluss

**Sonderfall C**
- [ ] 04.2035 mit grosser Einmalausgabe, Kriterium erfüllt → C1: Topf 1 = 21/12 × Netto-Bedarf + Einmalausgabe, Topf 2 = 9/12 × Netto-Bedarf
- [ ] Gleicher Fall, Kriterium nicht erfüllt → C2 mit Threshold
- [ ] C1 im Dezember → Topf 2 über dem gekürzten Ziel wird nicht umgebucht
- [ ] C2 im Juni (Threshold 3) → 2 Monate Reserve; November → 1 Monat; Dezember → 0
- [ ] C2 mit Threshold 9 im April → 8 Monate Reserve

**Immobilien**
- [ ] Verkauf Juni, Kauf Juli → Reservierung, Kauf aus Reservierung, Überschuss → Topf 3
- [ ] Verkauf Juni, Kauf September → kein Folgekauf, Erlös → Topf 3
- [ ] Verkauf Dezember, Kauf Januar → Kauf vor Januar-Auffüllung
- [ ] Kauf mit Restbetrag > Topf 1 → durch Vorausschau gedeckt, Topf 1 nie negativ
- [ ] Hypothekenerneuerung mit Einmalzahlung → über Vorausschau gedeckt

**Notfälle**
- [ ] Alle Töpfe leer, Immobilie vorhanden → Notverkauf, Erlös Topf 3, C erneut
- [ ] Alle Töpfe leer, keine Immobilie → Defizitphase, Zuflüsse in Topf 1, Neustart über A sobald Vermögen > 0
