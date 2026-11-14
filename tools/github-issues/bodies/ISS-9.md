Die Mindestabdeckung ist uneinheitlich: Der pre-push-Hook (`scripts/git-hooks/pre-push`), `coverage.yml` und `CLAUDE.md` verlangen **80 % Zeilenabdeckung je Paket** (Engine und App getrennt). `.github/workflows/test-quality.yml` verlangt dagegen **92 % für den ganzen Arbeitsbereich** (`--fail-under-lines 92`). Ein Pull Request kann deshalb an der zweiten Prüfung scheitern, obwohl Hook und Push grün waren.

**Anforderung:** Überall gilt dieselbe Regel: mindestens 80 % Zeilenabdeckung, getrennt für `pensionsplaner-engine` und `pensionsplaner-app`. Der Wert steht, wo möglich, nur an einer Stelle.

**Abnahme**

- [ ] `test-quality.yml` prüft 80 % je Paket statt 92 % über alles.
- [ ] Hook, `coverage.yml`, `test-quality.yml`, `CLAUDE.md` und `TESTS.md` nennen dieselbe Schwelle.
- [ ] Ein Pull Request, der Hook und `coverage.yml` besteht, besteht auch `test-quality.yml` bei der Abdeckung.
- [ ] Sinkt ein Paket unter 80 %, schlägt `test-quality.yml` fehl (einmal geprüft).

> «2. einheitlich auf 80 % setzen»
