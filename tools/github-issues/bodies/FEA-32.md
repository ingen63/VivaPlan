Die Mutationstests dauern zu lange, um sie vor jedem Push und in Pull Requests einzusetzen (FEA-27, FEA-29). Messung des letzten vollen Laufs lokal (9./10.11.2026, `-j 4`): 1352 Mutanten im Rechenkern, rund 24 h Wanduhrzeit, Median 144 s je Mutant, 90. Perzentil 568 s, längster 1201 s; 46 Mutanten liefen in den Timeout. Hochrechnung GitHub (2 Kerne, 8 Shards): über 5 h je Shard, mit Timeouts über der Grenze von 350 min. Dieses Issue muss vor FEA-27 umgesetzt sein.

**Anforderung**

1. **Timeout begrenzen:** Jeder Mutant hat eine feste Höchstdauer (Vorschlag 2 min für den Testlauf), eingestellt in `.cargo/mutants.toml`, nicht nur auf der Kommandozeile. Ein Timeout zählt weiterhin als erkannt.
2. **Schwere Tests ausnehmen:** Mutationsläufe nutzen ein eigenes nextest-Profil, das die schweren Tests überspringt (zum Beispiel `mutated_plans_never_crash` und Last-/Stresstests der QA-Matrix). Welche Tests ausgenommen sind, steht mit Begründung in der Konfiguration. Die ausgenommenen Tests laufen weiterhin in `make test` und in CI.
3. **Mehr Shards auf GitHub:** Der volle Lauf (`mutation-full.yml`) verwendet so viele Shards, dass jeder Shard mit Reserve unter der Zeitgrenze bleibt (Vorschlag 16).
4. **Messung:** Vor und nach der Änderung wird je ein Lauf gemessen; die Zahlen stehen im Issue.
5. Die Aussagekraft bleibt erhalten: Die Zahl der erkannten Mutanten sinkt durch die Änderungen höchstens um wenige Prozent; neu nicht erkannte Mutanten werden geprüft und im Issue genannt.

**Abnahme**

- [ ] `.cargo/mutants.toml` setzt den Timeout; ein Mutant mit Endlosschleife bricht nach der Höchstdauer ab.
- [ ] Mutationsläufe verwenden das eigene nextest-Profil ohne die schweren Tests; `make test` führt sie weiterhin aus.
- [ ] Voller Lauf lokal (`make mutants`): Wanduhrzeit vorher und nachher im Issue, Ziel unter 8 h.
- [ ] `--in-diff` für eine mittlere Änderung (50–100 Zeilen im Rechenkern) dauert lokal unter 10 min.
- [ ] `mutation-full.yml` läuft auf GitHub mit allen Shards innerhalb der Zeitgrenze durch.
- [ ] Vergleich vorher/nachher: Anteil erkannter Mutanten und Liste neu nicht erkannter Mutanten im Issue.
- [ ] `TESTS.md` beschreibt Timeout, Profil und ausgenommene Tests.

**Umsetzungshinweis** (nicht Teil der Anforderung): `timeout_multiplier` oder `minimum_test_timeout` in `.cargo/mutants.toml`, `test_tool = "nextest"` mit `additional_cargo_test_args`/Profil `mutants` in `.config/nextest.toml`; schnelles Build-Profil und Linker prüfen. Doctests laufen unter nextest nicht (siehe Bericht).

- Abhängigkeit: Voraussetzung für FEA-27 (#38) und FEA-29 (#40).

> Aus der Laufzeitschätzung des Test-Agenten; entspricht dem ToDo «geplante Verkürzungen, vor allem der nextest-Filter für die schweren Tests».
