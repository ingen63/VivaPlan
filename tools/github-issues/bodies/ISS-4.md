In `Neu.rplan` lässt sich der Meilenstein «Verkauf Haus» nicht löschen, obwohl in der Planung keine Verwendung zu sehen ist. Die Meldung nannte 2 Verwendungen.

**Befund:** Die zwei Verwendungen gab es wirklich: «Verkauf» der Immobilie «Haus» und «Kauf ab» der zweiten Immobilie. Sie stehen im Abschnitt Immobilien und waren dort nicht als Verwendung des Meilensteins erkennbar.

**Anforderung:** Wird ein verwendeter Meilenstein gelöscht, nennt die Rückfrage Zahl und Abschnitte der Verwendungen und stellt jede auf das feste Datum des Meilensteins um (siehe FEA-14).

**Abnahme**

- [ ] `Neu.rplan`: «Verkauf Haus» löschen fragt nach und nennt 2 Eingaben im Abschnitt Immobilien.
- [ ] Nach «Löschen und umstellen» tragen Verkauf und Kauf das feste Datum 06.2041.
- [ ] Ein Meilenstein ohne Verwendung lässt sich ohne Rückfrage löschen.

**Stand:** gelöst durch FEA-14 (Commit 0ef787d). Abnahme durch den Menschen offen.
