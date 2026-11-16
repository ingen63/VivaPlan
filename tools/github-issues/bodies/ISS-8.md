Der Workflow `.github/workflows/coverage.yml` («Tests & Coverage», läuft bei jedem Push und Pull Request) installiert die Linux-Abhängigkeiten von Tauri nicht. `pakete.yml` tut das (`libwebkit2gtk-4.1-dev`, `libxdo-dev`, `libssl-dev` und weitere). Der Schritt «App – Abdeckungsschwelle» (`cargo llvm-cov -p vivaplan-app`) kann daher auf dem Runner schon beim Bauen scheitern.

Vermutung, nicht belegt: Auf GitHub gibt es bisher keinen Lauf dieses Workflows. Zuerst klären, ob er scheitert.

**Erwartet:** Der Workflow läuft für Engine und App durch, wenn die Tests grün sind und die Abdeckung mindestens 80 % je Paket beträgt.

**Abnahme**

- [ ] Ein Lauf von `coverage.yml` auf GitHub ist grün (Engine und App).
- [ ] Die Abhängigkeiten stehen an einer Stelle und werden von `coverage.yml` und `pakete.yml` gemeinsam verwendet, oder sind in beiden gleich.
- [ ] Sinkt die Abdeckung von Engine oder App unter 80 %, schlägt der Lauf fehl (einmal geprüft).
- [ ] Die Laufzeit des Workflows ist im Issue vermerkt.

> Aus der Frage, welche Tests der nächste Push ausführt.
