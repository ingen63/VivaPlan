---
name: issue-umgesetzt
description: Nach dem Commit, der ein Issue vollständig umsetzt - Label implemented setzen und Kommentar «Umgesetzt in <Commit>. Abnahme ausstehend.» schreiben; Umgang mit wieder geöffneten Issues.
---

# Issue als umgesetzt markieren

Nur nach dem Commit, der die Anforderung **vollständig** umsetzt; Teilschritte bekommen das Label nicht.

```bash
gh issue edit <Nr> --add-label implemented
gh issue comment <Nr> --body "Umgesetzt in <kurzer Hash>. Abnahme ausstehend."
```

- Kriterien nicht abhaken, Issue nicht schliessen: Die Abnahme macht der Mensch (Ausnahme: Skill `pruefe-issue`).
- Im Bericht je Abnahmekriterium nennen, welcher Test es belegt und was nicht geprüft ist.
- Wird das Issue wieder geöffnet, weil etwas fehlt: `gh issue edit <Nr> --remove-label implemented`, bis die Nacharbeit committet ist; danach Label und Kommentar wie oben.
