.PHONY: test ui-unit clippy coverage coverage-diff mutants mutants-diff proptest fuzz quality report
test:        ; cargo nextest run --workspace
# FEA-69: reine Hilfsfunktionen der Oberfläche (crates/app/ui/util.js), ohne Browser
ui-unit:     ; node --test e2e/unit/*.test.js
# FEA-70: wie im Workflow, Warnungen sind Fehler
clippy:      ; cargo clippy --workspace --all-targets -- -D warnings && cd fuzz && cargo clippy --all-targets -- -D warnings
coverage:    ; cargo llvm-cov nextest --workspace --html --open
# FEA-28: Abdeckung des geänderten Codes gegenüber BASE (Vorgabe origin/main); ein Hinweis, keine Sperre
coverage-diff:
	mkdir -p local/test-quality && cargo llvm-cov nextest --workspace --lcov --output-path local/test-quality/lcov.info
	python3 tools/coverage_diff.py local/test-quality/lcov.info $${BASE:-origin/main}
# FEA-32: 8 parallele Arbeitskopien (10 Kerne); Profil, Zeitgrenze, Testauswahl und Ausgabeordner (local/mutants.out)
# stehen in .cargo/mutants.toml
mutants:     ; cargo mutants -p vivaplan-engine -j 8 --no-shuffle
mutants-diff:; git diff origin/main...HEAD > /tmp/pr.diff && cargo mutants --in-diff /tmp/pr.diff
proptest:    ; PROPTEST_CASES=10000 cargo nextest run --workspace -E 'test(/proptest/)'
fuzz:        ; cd fuzz && for t in $$(cargo +nightly fuzz list); do mkdir -p corpus/$$t; s=; [ -d seeds/$$t ] && s=seeds/$$t; cargo +nightly fuzz run $$t corpus/$$t $$s -- -max_total_time=60; done
quality:     ; ./tools/test-quality.sh
# FEA-30: Übersichtsseite «Testqualität» aus den vorhandenen Ergebnissen (nach make test, coverage und mutants)
report:      ; python3 tools/report/report.py
