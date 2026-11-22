# Pflichteingaben für eine Simulation (#14)

Grundlage sind `crates/engine/src/model.rs` (Felder ohne Standardwert: fehlen sie, lässt sich die Planung nicht laden) und
`crates/engine/src/validate.rs` (Fehler, die den Lauf verhindern). Warnungen sind nicht aufgeführt und blockieren nicht.
Jede Zeile hat einen Test in `crates/engine/tests/mandatory_inputs.rs` (Test = Name der Testfunktion), der den Fehler
auslöst und die Meldung prüft.

Im Editor sind Pflichtfelder mit einem Sternchen (*) gekennzeichnet. Ein Fehler erscheint direkt am Feld (soweit die
Meldung einem Feld zugeordnet ist, Spalte «Feld»), die Abschnittsliste zeigt die Zahl der Fehler. «Simulation starten»
und «Optimieren» sind blockiert, solange Fehler vorliegen («Die Planung hat N Fehler»); der Verweis führt in den ersten
betroffenen Abschnitt.

Jede Meldung trägt neben dem deutschen Text `message` einen Meldungscode `code` und die Parameter `params` (#171); die
Oberfläche zeigt den Text aus ihrer Texttabelle (`engine.<code>`). Die Codes und ihre Vorlagen stehen in
`crates/engine/src/message.rs` (siehe `docs/ENTWICKLUNG.md`, «Meldungscodes des Rechenkerns»).

## Immer nötig

| Bereich | Eingabe | Fehlermeldung (Auszug) | Feld | Test |
|---|---|---|---|---|
| Haushalt | mindestens 1, höchstens 2 Personen | «Mindestens eine Person ist nötig.» / «Es sind höchstens zwei Personen möglich.» | – | `household_needs_one_or_two_persons` |
| Person | Geburtsdatum (Monat/Jahr) | «Das Geburtsdatum … liegt nach dem Simulationsstart» / «… über 120 Jahre alt»; fehlt oder ist es ungültig, ist die Planung unvollständig | `household.persons.N.birth` | `person_needs_a_valid_birth_date` |
| Person | **Name**, nicht leer, höchstens 60 Zeichen | «Person N: Der Name fehlt (Pflichtangabe).» / «… höchstens 60 Zeichen lang sein.» | `household.persons.N.name` | `person_needs_a_name_of_at_most_sixty_characters` |
| Haushalt | Plafonierung AHV-Rente > 0 | «Die Plafonierung der AHV-Renten muss grösser als 0 sein.» | `household.ahvCoupleCap` | `ahv_cap_must_be_positive` |
| Meilensteine | `start` und `end`, Ende nach Start, Zeitraum höchstens 100 Jahre | «Die Meilensteine «start» und «end» müssen vorhanden sein.» / «… Ende muss nach dem Start liegen.» / «… länger als 100 Jahre.» | – | `milestones_start_and_end_are_required_and_ordered` |
| Meilensteine | Altersangaben 0–130 | «Alter N ist nicht möglich.» | – | `age_milestones_must_be_between_zero_and_130` |
| Vorsorge | Pensionskasse: Kapitalbezug 0–100 % (Kapital, Umwandlungssatz, Bezug müssen vorhanden sein) | «Kapitalbezug muss zwischen 0 % und 100 % liegen.» | – | `pension_fund_lump_sum_is_between_zero_and_one_hundred_percent` |
| Vorsorge | AHV-Rente (Betrag, Beginn) | Fehlt sie, ist die Planung unvollständig | – | `required_sections_must_be_present` |
| Einnahmen/Ausgaben | Einkommen und Ausgaben als Reihe mit gültigen Zeitpunkten und Werten | «… keine Einträge.» / «… Werte müssen zwischen … liegen.» / ungültiger Zeitpunkt | – | `income_and_spending_series_need_valid_entries` |
| Vermögen | Aktien, Obligationen, Cash, Aktienanteil | Fehlen sie, ist die Planung unvollständig; Aktienanteil ausserhalb 0–100 % ist ein Fehler | – | `required_sections_must_be_present`, `income_and_spending_series_need_valid_entries` |
| Renditen | Quelle (fest, historisch, Bootstrap, parametrisch) mit ihren Eingaben | siehe «Abhängig von der Auswahl» | – | `return_source_needs_its_inputs` |
| Steuern | Tabellen für Einkommen, Vermögen, Kapital und Steuerfuss | Fehlen sie, ist die Planung unvollständig | – | `required_sections_must_be_present` |

## Abhängig von der Auswahl

| Wenn gewählt | Dann nötig | Test |
|---|---|---|
| Renditequelle historisch | Zeitreihe und Startmonat | `return_source_needs_its_inputs` |
| Bootstrap | Zeitreihe, Pfade 1–1'000'000, Blocklänge ≥ 1 | `return_source_needs_its_inputs` |
| Parametrisch | Pfade 1–1'000'000, Volatilität und Sprungrate ≥ 0, Freiheitsgrade > 2 (0 = Normalverteilung) | `return_source_needs_its_inputs` |
| Entnahme 3-Topf / feste Rate / VPW / Endowment / Guardrails | gültige Töpfe und Raten, Planungsalter, Leitplanken, Untergrenze ≤ Obergrenze | `withdrawal_strategies_need_valid_parameters` |
| Immobilie im Besitz | je Hypothekentranche mindestens eine Periode | `properties_need_their_inputs` |
| Immobilie geplant | Hypothek ≤ Kaufpreis; Kauf «nach Verkauf von …» nur mit existierender Immobilie; Kennung eindeutig; optional «Verkauf ab» (Vorgabe «Nie») nach «Kauf ab» (#28) | `properties_need_their_inputs` |
| Miete | Mietbeginn und Miete | `properties_need_their_inputs` |
| Lebenshaltung inklusive Wohnkosten (#53) | jede Zeile der Lebenshaltung mindestens so gross wie der Wohnanteil (Wohnkosten zum Simulationsstart) | `cash_housing::issue_53_ak4_ak5_check_and_no_housing` |
| Säule 3a (#52, #55) | ein Konto je Person; Guthaben ≥ 0; Zins −50 % bis 100 %; Einzahlung 0 bis Maximalbetrag (Einstellungen); Auszahlungen (Datum oder Alter 0–150 Jahre) frühestens 5 Jahre vor dem AHV-Bezug, Betrag > 0 | `pillar_3a_is_checked` |
| Meilensteine «Bezug Pensionskasse» und «AHV-Bezug» (#51) | Alter innerhalb der Grenzen aus den Einstellungen (Standard 58–65 und 63–70 Jahre) | `milestones_limits::issue_51_ak2_ages_within_limits` |
| Steuertabellen und Steuerfuss (#98) | Einkommens-, Vermögens- und Kapitalbezugssteuer (auch Verheiratete): je Stufe Betrag ≥ 0 und Satz 0 % bis 100 %; Steuerfuss 0 bis 10 (im Modus «Wohnort» nur mit eigenen Regeln der Grundstückgewinnsteuer, #65); Meldung am Feld | `issue_98_tax_tables_and_multiplier_must_be_in_range` |
| Grundstückgewinnsteuer | gültige Besitzdauer-Staffel (Modell Aargau: Satz je Besitzdauer) | `property_gain_tax_rules_must_be_valid` |
| Grundstückgewinnsteuer (Regeln gesetzt) | Bei einer Immobilie mit Status «Besitz» oder «Kauf geplant» müssen Regeln gesetzt sein (Vorlage oder eigene, auch Satz 0 %); Mietobjekte lösen die Pflicht nicht aus (#29); im Modus «Wohnort» nur mit «Eigene Regeln statt der Regeln des Kantons» (#65) | `property_gain_tax_is_mandatory_for_owned_and_planned_properties`, `gain_location::issue_65_ak6_validation_in_location_mode` |
| Kanton der Immobilie (#65) | Im Modus «Wohnort» mit den Regeln des Kantons: Kanton einer Immobilie im Besitz oder mit geplantem Kauf in der Steuerdatei (Fehler am Feld); Kaufdatum einer Immobilie im Besitz (Warnung) | `gain_location::issue_65_ak6_validation_in_location_mode` |
| Variationen | gültiger Parameter, Wertebereich und Schritt | `variations_need_a_valid_parameter` |

## Warnungen (blockieren nicht)

- Ein Kauf- oder Verkaufsdatum vor dem Simulationsstart wird nie erreicht (Abschnitt Immobilien; `planned_purchase_before_start_is_reported`, `sale_before_start_is_reported_as_a_warning`).
- Die zweite Person hat weder Einkommen noch Vorsorge erfasst.
