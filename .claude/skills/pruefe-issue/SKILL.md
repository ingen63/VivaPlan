---
name: pruefe-issue
description: Einzelprüfung eines Issues oder Kriteriums auf Anweisung («Prüfe #27», «Prüfe #27/AK2»). Kriterien prüfen, Ergebnis im Issue festhalten, abhaken, schliessen und docs/ABNAHME.md nachführen.
---

# Einzelprüfung «Prüfe #Nr»

Grundlage ist `docs/ABNAHME.md` (Abschnitt «Vorgehen bei der Einzelprüfung»; nur gezielt lesen: `grep -n '^### #<Nr> ' docs/ABNAHME.md`, dann den Zeilenbereich). Alte IDs (`FEA-…`) führt `docs/ISSUE-IDS.md` auf die Nummer.

Diese Anweisung hebt für die Prüfung die Regel auf, dass Claude Kriterien nie abhakt und Issues nie schliesst. Für Umsetzungen gilt die Regel weiter.

1. `gh issue view <Nr> --comments` lesen. Kriterien sind die Häkchen im Abschnitt «Abnahmekriterien» (AK2 = zweites Häkchen); Kriterien aus späteren Kommentaren zählen mit.
2. Jedes Kriterium am heutigen Stand prüfen: vorhandenen Test gezielt laufen lassen (Rust: `cargo nextest run -p <Paket> -E 'test(issue_<Nr>_)'`, e2e: `npx playwright test -g "#<Nr>"` in `e2e/`), sonst im Code oder in der Oberfläche nachsehen. Bei «überholt» nur prüfen, ob das neue Verhalten gilt.
3. Ohne Rückfrage festhalten:
   - Kommentar im Issue mit dem Ergebnis je Kriterium (`gh issue comment`).
   - Erfüllte Kriterien im Issue abhaken (Text mit `gh issue view --json body` holen, `- [ ]` → `- [x]`, `gh issue edit --body-file`). Kriterien nie inhaltlich ändern.
   - Sind alle erfüllt: Issue schliessen (`gh issue close`). Sonst offen lassen bzw. wieder öffnen und den Fehler im Kommentar nennen; einen Fehler als neues Issue vorschlagen (`bug`, `found: acceptance`).
   - In `docs/ABNAHME.md` die Spalten «Test» und «Geprüft» nachführen (Datum, Ergebnis, Art der Prüfung). Ist das Issue danach geschlossen und alles `ok`, den Abschnitt nach `docs/ABNAHME-ARCHIV.md` verschieben (gleiche Form, nach Nummer einsortiert).
4. Die Änderung an `docs/ABNAHME.md` (und am Archiv) nur auf ausdrückliches Wort committen.
5. Im Bericht je Kriterium nennen, was geprüft ist und was nicht.
