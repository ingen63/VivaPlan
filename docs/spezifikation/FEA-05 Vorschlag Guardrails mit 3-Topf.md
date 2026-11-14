# FEA-05 Vorschlag: Guardrails und 3-Topf gemeinsam anwenden

Vorschlag nach Review (Issue #6), mit den Entscheiden zu F1 bis F5 (Abschnitt 8). Am Code ist nichts geändert.

**Anforderung (Issue):** Die Guardrails steuern die Ausgaben, die 3-Topf-Strategie steuert die Anlage.

## 1. Ausgangslage im Code

Die beiden Teile sind in der Engine schon weitgehend getrennt:

| Teil | Wo | Was es tut |
|---|---|---|
| Ausgabenregel | `Simulator.dynamic` (`Rule::Guardrails`), `update_budget()` | legt die Jahresentnahme fest; daraus folgt die Lebenshaltung des Monats |
| Topf-Logik | `Simulator.buckets`, `bucket_step()` | verteilt das Vermögen auf Topf 1–3 und bezahlt die Ausgaben aus Topf 1 |

Heute ist immer nur eines der beiden Felder gesetzt, weil `withdrawal.strategy` exklusiv ist. Technisch können beide gleichzeitig laufen: Die Lebenshaltung wird vor dem Renditeschritt bestimmt, die Töpfe erst danach.

Es gibt **eine echte Lücke**. Die Topfziele kommen aus dem Netto-Bedarf (`plan_basis()`), und dieser rechnet heute mit den **geplanten** Ausgaben (`s.spending`). Mit Guardrails müssen die Ziele aus der **Guardrails-Lebenshaltung** kommen. Sonst sind die Töpfe zu gross oder zu klein. Das hält auch der Prüfbericht als Hauptrisiko fest.

## 2. Datenmodell

### Variante A (empfohlen): Ausgabenregel als Option der 3-Topf-Strategie

```json
"withdrawal": {
  "strategy": "three_buckets",
  "buckets": { "cashYears": 2, "bondYears": 5, "refillThreshold": 0, "reserveMonths": 3,
               "spending": "guardrails" },
  "guardrails": { "initialRate": 0.05, "upper": 0.2, "lower": 0.2, "adjust": 0.1 },
  "floor": 0.8, "ceiling": 0
}
```

- Neues Feld `buckets.spending`: `"planned"` (Standard, wie heute) oder `"guardrails"`.
- Die Parameter in `withdrawal.guardrails`, `floor` und `ceiling` werden **wiederverwendet**. Sie gelten unverändert, und ein Wechsel zwischen den Strategien behält die Werte.
- Bestehende Dateien haben das Feld nicht und gelten als `"planned"`. Die Ergebnisse bleiben gleich, und die Golden-Dateien ändern sich nicht.

### Variante B: zwei unabhängige Achsen

Die Wahl wird in «Ausgabenregel» (geplant, fixe Rate, Guardrails, VPW, Endowment) und «Anlage» (Aktienanteil oder 3 Töpfe) aufgeteilt. Das ist allgemeiner, aber deutlich grösser:

- Die Dateien müssen migriert werden.
- Die Optimierung bekommt 5 × 2 Kombinationen.
- Die Reserve aus FEA-17 muss neu geklärt werden.
- Alle Karten, Vergleiche und Hilfetexte ändern sich.

**Empfehlung: A.** Das Issue verlangt nur Guardrails. A bleibt rückwärtskompatibel, und die Engine wird so gebaut, dass später auch andere Ausgabenregeln mit den Töpfen laufen könnten. Diese Erweiterung wäre dann ein eigenes Issue.

## 3. Rechenregeln

Alles, was hier nicht steht, bleibt wie bei der heutigen 3-Topf-Strategie und den heutigen Guardrails.

**Grundsatz:** Die Ausgabenregel verhält sich in der reinen Strategie «Guardrails» und in der Kombination **identisch**. Die Punkte 2, 5 und 6 ändern deshalb auch die reine Strategie «Guardrails». Fixe Rate, VPW und Endowment starten nach FEA-33 (#47) ebenfalls ab der ersten Pensionierung.

1. **Ausgaben:** Die Lebenshaltung ab Pensionierung bestimmen die Guardrails, genau wie heute:
   - Jahresentnahme = Lebenshaltung + Wohnen − Einkommen und Renten (vor Steuern).
   - Teuerungsanpassung jeden Januar, ausgesetzt nach einem Verlustjahr.
   - Leitplanken kürzen oder erhöhen die Entnahme.
   - Untergrenze und Obergrenze gelten.
2. **Vermögen für die Rate:** Die aktuelle Rate ist Entnahme / liquides Vermögen (bei 3-Topf: Topf 1 + Topf 2 + Topf 3). Ein für einen Kauf **reservierter** Verkaufserlös zählt nicht dazu (Entscheid F2). Das gilt für die Startentnahme und für die Leitplanken, in der reinen Strategie und in der Kombination.
3. **Netto-Bedarf und Topfziele:** Im Netto-Bedarf ersetzt die **Guardrails-Lebenshaltung** (nach Unter- und Obergrenze) die geplante Lebenshaltung. Wohnen, Einkommenssteuer und die abgezogenen Einnahmen bleiben gleich.
   - Wenn keine Grenze greift, gilt: Netto-Bedarf = Guardrails-Entnahme + Einkommenssteuer.
   - Ziel Topf 1 = Jahre × Netto-Bedarf, Ziel Topf 2 = Jahre × Netto-Bedarf, wie bisher.
4. **Reihenfolge im Januar:**
   1. Guardrails: Teuerung und Leitplanken bestimmen die neue Entnahme.
   2. Netto-Bedarf und Ziele werden daraus berechnet.
   3. Januar-Auffüllung der Töpfe.

   Das entspricht der heutigen Reihenfolge im Monat (Geldflüsse vor Renditeschritt, Töpfe danach).
5. **Start (Entscheid F1):** Die Guardrails starten, sobald die **erste** Person pensioniert ist. Das gilt in der reinen Strategie und in der Kombination. Vor FEA-05 startete die reine Strategie erst, wenn alle pensioniert waren.
   - In der Kombination starten Töpfe und Guardrails damit gemeinsam. Zuerst legen die Guardrails die Startentnahme fest (Startrate × liquides Vermögen). Danach läuft Auslöser A mit dem daraus berechneten Netto-Bedarf.
   - Die reine Strategie legt ab demselben Monat das Vermögen wie die anderen Strategien an (fester Aktienanteil, Reserve 3 Monate).
   - Ein Erwerbseinkommen der zweiten Person zählt zu «Einkommen und Renten». Solange es fliesst, ist die Lebenshaltung entsprechend höher. Fällt es mit der zweiten Pensionierung weg, sinkt die Lebenshaltung, und die Entnahme bleibt gleich. Die Hilfe nennt diese Folge.
6. **Zuflüsse** (Erbschaft, Kapitalbezug, Verkaufserlös):
   - Das Geld geht wie bei 3-Topf nur in Topf 3 (Fall B).
   - Die Guardrails wenden im Zuflussmonat wie heute an, also Neuberechnung der Entnahme.
   - Die Ziele bleiben bis zum nächsten Januar (Entscheid F3). Die Vorausschau (Fall C) rechnet aber ab sofort mit der neuen monatlichen Lebenshaltung.
   - Die Neuberechnung der Entnahme ist in der reinen Strategie und in der Kombination gleich. Die reine Strategie hat keine Töpfe, der Teil zu den Zielen betrifft nur die Kombination.
7. **Zwei verschiedene Renditemasse, unverändert:**
   - Die Guardrails setzen die Teuerung nach einem Verlustjahr im **Kalenderjahr** aus.
   - Das Renditekriterium der Töpfe misst die **letzten 12 Monate**.

   Im Januar ergeben beide dasselbe. Die Hilfe nennt den Unterschied.
8. **Notverkauf, Phase 8, Steuern:** Diese Regeln ändern sich nicht. Liegt das Vermögen bei 0, ist die Rate unendlich. Die Leitplanke kürzt dann, und die Untergrenze hält die Ausgaben trotzdem.

## 4. Bedienung in der Oberfläche

### Editor, Abschnitt «Entnahmestrategie»

Die Karten bleiben gleich, es gibt keine siebte Karte. Auf der Karte **3-Topf** erscheint unter den Töpfen eine Auswahl:

```
┌ Töpfe ───────────────────────────────────────────────────────────┐
│ Topf 1 · Cash [2] Jahre   Topf 2 · Obligationen [5] Jahre  │
│ Aus Topf 3 nur verkaufen, wenn Aktienrendite ≥ [0] %             │
│ Reserve in Topf 1 unterjährig [3] Monate                         │
│                                                                  │
│ Ausgaben ab Pensionierung  (•) wie geplant                       │
│                            ( ) mit Guardrails steuern  ⓘ         │
└──────────────────────────────────────────────────────────────────┘
┌ Guardrails  nach Guyton-Klinger ─────────────────────────────────┐   ← nur bei «mit Guardrails»
│ Startrate [5] %                Anpassung bei Leitplanke [10] %   │
│ Obere Leitplanke [20] %        Untere Leitplanke [20] %          │
└──────────────────────────────────────────────────────────────────┘
┌ Grenzen der Lebenshaltung ───────────────────────────────────────┐   ← nur bei «mit Guardrails»
│ Untergrenze [80] %             Obergrenze [0] %                  │
└──────────────────────────────────────────────────────────────────┘
```

- Die Guardrails-Karte und die Grenzen-Karte sind **dieselben Bausteine** wie bei der Strategie «Guardrails». Werte, die dort schon eingegeben sind, erscheinen gleich.
- Das ⓘ öffnet die neue Hilfestelle «3-Topf mit Guardrails».
- Der Text der 3-Topf-Karte wird ergänzt: «Ausgaben wie geplant oder mit Guardrails gesteuert».

### Namen und Anzeige

- Strategiename überall (Ergebnisse, gespeicherte Ergebnisse, Vergleich, Optimierung, Bericht): **«3-Topf + Guardrails»**.
- Tooltip der Strategie (FEA-10): alle 3-Topf-Parameter, danach die Guardrails-Parameter und die Grenzen.

### Ergebnisansicht

- **Tabelle «3-Topf-Strategie»:** Die Zeile Lebenshaltung zeigt die Quelle, also «Lebenshaltung (Guardrails)» statt «Lebenshaltung (geplant)».
- **Ereignisliste:** Wie heute. Die Einträge «Entnahme … pro Jahr» und «Leitplanke: Entnahme um … gekürzt/erhöht» stehen neben den Topf-Umbuchungen.
- **Diagramme:** Keine neuen Diagramme. Die Lebenshaltung und die Töpfe werden schon heute gezeigt.

### Hilfe (`crates/app/ui/hilfe/30-entnahme.md`)

- Neue Stelle `@f-wd-three_buckets_guardrails` mit Idee, Regeln (Abschnitt 3) und einem Beispiel.
- Vergleichstabelle «Welche Strategie passt?» um eine Zeile ergänzen.
- Den Satz «Die Grenzen gelten nicht für Standard und 3-Topf» präzisieren: Sie gelten für 3-Topf, wenn die Ausgaben mit Guardrails gesteuert werden.
- Den Absatz «Zuflüsse wirken sofort» um den Kombinationsfall ergänzen.
- Den Absatz «Wann gilt die Strategie?» ändern: Alle Strategien ausser «Standard» starten ab der ersten Pensionierung (mit FEA-33).
- In der Stelle «Guardrails» nennen, dass ein reservierter Verkaufserlös nicht zur Rate zählt und dass ein noch laufendes Erwerbseinkommen der zweiten Person die Lebenshaltung erhöht.

## 5. Optimierung

- Neuer Eintrag **«3-Topf + Guardrails»** in der Liste der Strategien (Schlüssel `three_buckets_guardrails`), standardmässig **nicht** angewählt.
- Parameter:
  - Topf 1, Topf 2, Renditeschwelle
  - Startrate, obere Leitplanke, untere Leitplanke, Anpassung
  - Untergrenze und Obergrenze bleiben fix, wie bei Guardrails.
- Sieben variierte Parameter machen die Suche teuer. Deshalb sind untere Leitplanke und Anpassung standardmässig **fix** und lassen sich in den Einstellungen freigeben (Entscheid F4).
- Die Szenarien- und Variationslogik (`scenarios.rs`, «Startrate der Guardrails») gilt auch hier.

## 6. Validierung

- Ist `buckets.spending = "guardrails"`, gelten die Prüfungen der Guardrails und der Grenzen (Untergrenze ≤ Obergrenze), wie bei der Strategie «Guardrails».
- Ein unbekannter Wert für `spending` ist ein Fehler.

## 7. Vorgeschlagene Abnahmekriterien (für Issue #6)

- [ ] Auf der Karte «3-Topf» kann «Ausgaben ab Pensionierung» zwischen «wie geplant» und «mit Guardrails steuern» gewählt werden; bei «mit Guardrails» erscheinen die Karten «Guardrails» und «Grenzen der Lebenshaltung».
- [ ] Bei «mit Guardrails» bestimmen Startrate, Teuerung, Leitplanken und Grenzen die Lebenshaltung genau wie bei der Strategie «Guardrails»; bei gleichen Eingaben und gleichem Vermögensverlauf ist die Entnahme in beiden identisch.
- [ ] Die Guardrails (rein und kombiniert) starten mit der Pensionierung der ersten Person; in der Kombination gemeinsam mit den Töpfen.
- [ ] Ein für einen Kauf reservierter Verkaufserlös zählt für die Guardrails-Rate (rein und kombiniert) nicht zum Vermögen.
- [ ] Nach einem Zufluss rechnen die Guardrails (rein und kombiniert) die Entnahme neu; in der Kombination geht das Geld in Topf 3, und die Topfziele bleiben bis zum nächsten Januar.
- [ ] Netto-Bedarf und Topfziele werden aus der Guardrails-Lebenshaltung berechnet; die Tabelle «3-Topf-Strategie» nennt diese Quelle.
- [ ] Nach einer Kürzung durch die Leitplanke sinken die Ziele von Topf 1 und 2 in der nächsten Januar-Auffüllung entsprechend.
- [ ] Die Topf-Regeln (Januar-Auffüllung, A, B, C, Renditekriterium, Notverkauf, Phase 8) gelten unverändert.
- [ ] Planungen ohne das neue Feld rechnen unverändert, ausser mit der Strategie «Guardrails» bei zwei Personen mit verschiedenem Pensionierungsdatum oder mit reserviertem Verkaufserlös (Golden-Dateien unverändert, keine verwendet Guardrails).
- [ ] Der Strategiename «3-Topf + Guardrails» erscheint in Ergebnissen, gespeicherten Ergebnissen, Vergleich und Tooltip (FEA-10).
- [ ] Die Optimierung bietet «3-Topf + Guardrails» als eigene Strategie an (standardmässig nicht angewählt; untere Leitplanke und Anpassung standardmässig fix).
- [ ] Hilfe, `BESCHREIBUNG.md` und Spezifikation beschreiben die Kombination.

## 8. Entscheide (Review)

- **Grundsatz:** Die reine Strategie «Guardrails» und die Kombination verhalten sich bei F1 bis F3 identisch.
- **F1 Start bei zwei Personen:** Die Guardrails starten ab der **ersten** Pensionierung, in der Kombination gemeinsam mit den Töpfen.
- **F2 Reservierter Verkaufserlös:** Er zählt für die Guardrails-Rate **nicht** mit.
- **F3 Zufluss:** Die Entnahme wird neu berechnet. In der Kombination bleiben die Topfziele bis zum nächsten Januar.
- **F4 Optimierung:** Standardmässig werden 5 der 7 Parameter variiert. Untere Leitplanke und Anpassung sind fix und lassen sich freigeben.
- **F5 Umfang:** Nur Guardrails, wie im Issue. Andere Ausgabenregeln mit Töpfen wären ein eigenes Issue.

## 9. Aufwand und Risiko

| Teil | Umfang |
|---|---|
| Engine: `spending`-Feld, `plan_basis()` mit Guardrails-Lebenshaltung, gemeinsamer Start ab erster Pensionierung | mittel, Kern des Risikos |
| Engine: reine Guardrails ab erster Pensionierung, ohne reservierten Erlös; bestehende Tests dazu anpassen | klein |
| Validierung, Optimierung (`optimize.rs`), Szenarien | klein bis mittel |
| Oberfläche: Auswahl, Karten einblenden, Namen, Tooltip | klein |
| Hilfe, `BESCHREIBUNG.md`, Spezifikation | klein |
| Tests: Regression (alte Dateien unverändert), Rechenbeispiele pro Kriterium, UI-Test der Auswahl | mittel |

Das Risiko liegt in der Wechselwirkung zwischen Leitplanke und Topfzielen. Gegenstand sind die Golden-Tests, ein deterministischer Fall mit Crash im ersten Jahr (Kürzung, dann kleinere Ziele) und ein Fall mit einem Zufluss.
