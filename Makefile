.PHONY: test test-quiet ui-unit clippy coverage coverage-diff mutants mutants-diff proptest fuzz quality report network-free english ui-texts golden-numbers i18n-table i18n-check icons
test:        ; cargo nextest run --workspace
# #160: dieselben Tests, aber nur Fehlschläge, langsame Tests und die Zusammenfassung (für Claude Code: wenig Kontext)
QUIET = --status-level fail --final-status-level slow --hide-progress-bar --cargo-quiet
# #165: die Sprachprüfung läuft mit, sobald die Übergangsliste in tools/check_english.py leer ist (--gate)
# #170: keine deutschen Oberflächentexte ausserhalb der Texttabelle
# #180: Prüfung der Übersetzungen gegen das Deutsche (Längenhinweise nur in make i18n-check)
test-quiet:  ; cargo nextest run --workspace $(QUIET) && ./tools/check-english.sh --gate && python3 tools/check_ui_texts.py && python3 tools/check_i18n.py && python3 -m unittest -q tools/test_icons.py
# #87: reine Hilfsfunktionen der Oberfläche (crates/app/ui/util.js), ohne Browser
# #147: runs.test.js braucht fake-indexeddb (e2e/node_modules)
ui-unit:     ; [ -d e2e/node_modules/fake-indexeddb ] || (cd e2e && npm ci); node --test e2e/unit/*.test.js
# #157: keine Netzwerk-Bibliothek in den Abhängigkeiten (cargo tree, Sperrliste), samt Tests des Skripts
network-free:; python3 -m unittest discover -s tools -p 'test_check_network_free.py' && ./tools/check-network-free.sh
# #165: deutsche Kommentare und Testnamen (ohne Pfade der Übergangsliste), samt Tests des Skripts
english:     ; python3 -m unittest discover -s tools -p 'test_check_english.py' && ./tools/check-english.sh
# #170: deutsche Oberflächentexte ausserhalb der Texttabelle (crates/app/ui/i18n/de.json), samt Tests des Skripts
ui-texts:    ; python3 -m unittest discover -s tools -p 'test_check_ui_texts.py' && python3 tools/check_ui_texts.py
# #180: Übersetzungen (Glossarbegriffe, Platzhalter und Zahlen, Reste, Aufbau der Hilfe, Anrede, Länge) gegen das
# Deutsche; Ausnahmen in tools/i18n_check_exceptions.json, samt Tests des Skripts
i18n-check:  ; python3 -m unittest discover -s tools -p 'test_check_i18n.py' && python3 tools/check_i18n.py --hints
# #182/AK1: gemeinsame Prüftabelle der Übersetzungen (local/i18n-review/tabelle.csv), samt Tests des Skripts
i18n-table:  ; python3 -m unittest discover -s tools -p 'test_i18n_review_table.py' && python3 tools/i18n_review_table.py
# #176/AK2: Golden-Dateien gegenüber BASE (Vorgabe HEAD) ohne Textfelder vergleichen, samt Tests des Skripts
golden-numbers:; python3 -m unittest discover -s tools -p 'test_golden_numbers.py' && python3 tools/golden_numbers.py --base $${BASE:-HEAD}
# #185: App-Icons und Favicon aus crates/app/icons/icon.svg und icon-small.svg, samt Vorschau (macOS)
icons:       ; ./tools/make-icons.sh --preview && python3 -m unittest tools/test_icons.py
# #88: wie im Workflow, Warnungen sind Fehler
clippy:      ; cargo clippy --workspace --all-targets -- -D warnings && cd fuzz && cargo clippy --all-targets -- -D warnings
coverage:    ; cargo llvm-cov nextest --workspace --html --open
# #39: Abdeckung des geänderten Codes gegenüber BASE (Vorgabe origin/main); ein Hinweis, keine Sperre
coverage-diff:
	mkdir -p local/test-quality && cargo llvm-cov nextest --workspace --lcov --output-path local/test-quality/lcov.info
	python3 tools/coverage_diff.py local/test-quality/lcov.info $${BASE:-origin/main}
# #43: 8 parallele Arbeitskopien (10 Kerne); Profil, Zeitgrenze, Testauswahl und Ausgabeordner (local/mutants.out)
# stehen in .cargo/mutants.toml
mutants:     ; cargo mutants --workspace -j 8 --no-shuffle
mutants-diff:; git diff origin/main...HEAD > /tmp/pr.diff && cargo mutants --in-diff /tmp/pr.diff
proptest:    ; PROPTEST_CASES=10000 cargo nextest run --workspace -E 'test(/proptest/)'
fuzz:        ; cd fuzz && for t in $$(cargo +nightly fuzz list); do mkdir -p corpus/$$t; s=; [ -d seeds/$$t ] && s=seeds/$$t; cargo +nightly fuzz run $$t corpus/$$t $$s -- -max_total_time=60; done
quality:     ; ./tools/test-quality.sh
# #41: Übersichtsseite «Testqualität» aus den vorhandenen Ergebnissen (nach make test, coverage und mutants)
report:      ; python3 tools/report/report.py
