.PHONY: test test-quiet ui-unit clippy coverage coverage-diff mutants mutants-diff proptest fuzz quality report network-free
test:        ; cargo nextest run --workspace
# #160: dieselben Tests, aber nur Fehlschläge, langsame Tests und die Zusammenfassung (für Claude Code: wenig Kontext)
QUIET = --status-level fail --final-status-level slow --hide-progress-bar --cargo-quiet
test-quiet:  ; cargo nextest run --workspace $(QUIET)
# #87: reine Hilfsfunktionen der Oberfläche (crates/app/ui/util.js), ohne Browser
# #147: runs.test.js braucht fake-indexeddb (e2e/node_modules)
ui-unit:     ; [ -d e2e/node_modules/fake-indexeddb ] || (cd e2e && npm ci); node --test e2e/unit/*.test.js
# #157: keine Netzwerk-Bibliothek in den Abhängigkeiten (cargo tree, Sperrliste), samt Tests des Skripts
network-free:; python3 -m unittest discover -s tools -p 'test_check_network_free.py' && ./tools/check-network-free.sh
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
