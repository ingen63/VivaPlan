Nicht erkannte (missed) Mutanten im Rechenkern werden einzeln beurteilt statt über einen Mutation Score gesteuert. Google zeigt im Review einzelne überlebende Mutanten statt einer Kennzahl; ein Score als Ziel lädt zu Goodhart-Effekten ein. cargo-mutants selbst gibt keine Score-Formel vor. Grundlage: Bericht `reports/Testqualität beurteilen 2026.md`, Empfehlung 3.

**Anforderung**

1. Für jeden missed Mutanten in `vivaplan-engine` gilt eines von beiden: Ein Test wird ergänzt, der ihn tötet, oder er wird mit `#[mutants::skip]` bzw. `exclude_re` in `.cargo/mutants.toml` **mit Begründung** ausgeschlossen (zum Beispiel äquivalenter Mutant, reine Protokollierung).
2. Es gibt keinen Mutation Score als Zielwert.
3. Die Liste der offenen missed Mutanten des letzten vollen Laufs ist einsehbar (Datei oder Lauf-Zusammenfassung) und wird pro Version durchgesehen.
4. Ausschlüsse ohne Begründung sind nicht erlaubt.

**Abnahme**

- [ ] Ein voller Lauf `make mutants` auf `vivaplan-engine` ist ausgewertet; jeder missed Mutant ist getestet oder begründet ausgeschlossen.
- [ ] Jeder Ausschluss trägt eine Begründung im Code oder in `.cargo/mutants.toml`.
- [ ] Die Liste der offenen missed Mutanten ist dokumentiert (Ort in `TESTS.md` genannt).
- [ ] `TESTS.md` beschreibt die Regel «einzeln beurteilen, kein Score-Ziel» und die Durchsicht pro Version.

**Umsetzungshinweis** (nicht Teil der Anforderung): Ergebnis in `mutants.out/missed.txt`; voller Lauf lokal oder per `mutation-full.yml` (8 Shards). Abhängigkeit: FEA-27 nutzt dieselbe Regel für neue Änderungen.

> Aus dem Bericht «Testqualität beurteilen 2026», Empfehlung 3.
