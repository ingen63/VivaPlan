---
name: issue-anlegen
description: GitHub-Issue anlegen oder pflegen (Fehler, Funktion, Aufgabe), Labels, Meilensteine, Fehlalarme schliessen, Kriterien nachträglich ändern.
---

# Issues anlegen und pflegen

- Vorlagen in `.github/ISSUE_TEMPLATE/` (`fehler.md`, `funktion.md`). Titel ohne Präfix, deutsch.
- Eine Anforderung pro Issue. Der Text enthält die Anforderung und die **Abnahmekriterien als Häkchen-Liste** (`- [ ] …`); jedes Kriterium soll sich testen lassen.
- **Labels** (englisch): Art `bug`, `enhancement` oder `task` (genau eines); Bereich `engine`, `ui`, `repo`; `implemented` nur nach der vollständigen Umsetzung (Skill `issue-umgesetzt`); bei Fehlern zusätzlich der Fundort `found: test`, `found: acceptance` oder `found: usage`. Dazu die Standard-Labels von GitHub (`invalid`, `duplicate`, `wontfix`, `documentation` usw.).
- **Meilensteine:** «Nächste Version», «Später».
- Alte IDs (`FEA-…`, `ISS-…`, `GEN-…`) aus der Git-Historie führt `docs/ISSUE-IDS.md` auf die Nummern.
- Abnahmekriterien nach dem Start nicht stillschweigend ändern: Änderungen als Kommentar mit Begründung.
- Fehlalarme mit `invalid` als «not planned» schliessen (`gh issue close <Nr> --reason "not planned"`), mit Begründung im Kommentar.
- Keine privaten Daten (echte Namen, Beträge, Planungsdateien) im Issue.
- Anlegen nur auf Wort des Nutzers; vorher den Entwurf zeigen, wenn er nicht ausdrücklich darauf verzichtet hat.
