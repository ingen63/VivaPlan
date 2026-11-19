#!/bin/sh
# User interface smoke test: starts the local development server and opens every view in headless
# Chrome. Checked: the page starts, no JavaScript errors, results and charts appear.
set -eu
export PATH="$HOME/.cargo/bin:$PATH"
cd "$(dirname "$0")/.."
CHROME="${CHROME:-/Applications/Google Chrome.app/Contents/MacOS/Google Chrome}"
PORT="${PORT:-8811}"
OUT="$(mktemp -d)"
cargo build -q --release -p vivaplan-cli
target/release/vivaplan ui-dev --port "$PORT" >"$OUT/server.log" 2>&1 &
SERVER=$!
trap 'kill $SERVER 2>/dev/null; rm -rf "$OUT"' EXIT
sleep 1

fail=0
check() { # route, at least this many SVG paths
  route="$1"; min="$2"
  "$CHROME" --headless=new --disable-gpu --window-size=1400,1600 --virtual-time-budget=60000 \
    --dump-dom "http://127.0.0.1:$PORT/#$route" 2>/dev/null >"$OUT/page.html" || true
  errors=$(grep -o 'data-errors="[0-9]*"' "$OUT/page.html" | head -1 || true)
  last=$(grep -o 'data-last-error="[^"]*"' "$OUT/page.html" | head -1 || true)
  ready=$(grep -c 'data-ready="1"' "$OUT/page.html" || true)
  paths=$(grep -o '<path' "$OUT/page.html" | wc -l | tr -d ' ')
  rendered=1
  case "$route" in results*|runs*|help*) rendered=$(grep -c 'data-rendered=' "$OUT/page.html" || true) ;; esac
  if [ -n "$errors" ] || [ "$ready" -eq 0 ] || [ "$rendered" -eq 0 ] || [ "$paths" -lt "$min" ]; then
    echo "FEHLER  #$route  $errors $last bereit=$ready gerechnet=$rendered Pfade=$paths"
    fail=1
  else
    echo "ok      #$route  ($paths Pfade)"
  fi
}

for s in general milestones pension flows wealth returns withdrawal taxes realestate scenarios; do check "$s" 3; done
for src in fixed historical rolling bootstrap parametric; do check "returns/$src" 3; done
check "withdrawal/buckets" 3
check "withdrawal/fixedrate" 3
check "withdrawal/guardrails" 3
check "withdrawal/vpw" 3
check "withdrawal/endowment" 3
check "optimize" 3
check "general/partner" 3
check "pension/partner" 3
check "flows/partner" 3
check "taxes/partner" 3
check "results/nosweep" 4
check "results/charts/nosweep" 8
check "results/table/nosweep" 0
check "results/scenarios" 3
check "results/rolling/charts" 6
check "results/rolling/charts/real" 6
check "results/bootstrap" 4
check "results/bootstrap/charts" 6
check "results/bootstrap/runs" 6
check "results/bootstrap/scenarios" 6
check "results/bootstrap/scenarios/sweep2" 0
check "results/parametric/charts/buckets" 6
check "results/historical/charts" 6
check "results/charts/nosweep/partner/guardrails" 8
check "results/bootstrap/charts/fixedrate" 6
check "results/bootstrap/runs/partner" 6
check "about" 0
check "runs/mem" 0
check "runsflow/mem" 6
check "runsflow/mem/view" 8
check "runsflow/mem/del" 0
# Optimization: saved settings appear in the form, changes end up in the plan
"$CHROME" --headless=new --disable-gpu --virtual-time-budget=30000 --dump-dom "http://127.0.0.1:$PORT/#optimize/optset" 2>/dev/null >"$OUT/page.html" || true
if grep -q 'id="opt-maxRuin" value="2"' "$OUT/page.html" && grep -q 'id="opt-weightMedian" value="70"' "$OUT/page.html" && grep -q 'id="opt-maxVol" value="12"' "$OUT/page.html" \
   && grep -q 'id="optS-guardrails" checked' "$OUT/page.html" && ! grep -q 'id="optS-standard" checked' "$OUT/page.html"; then echo "ok      #optimize/optset"; else echo "FEHLER  #optimize/optset (Einstellungen nicht geladen)"; fail=1; fi
"$CHROME" --headless=new --disable-gpu --virtual-time-budget=30000 --dump-dom "http://127.0.0.1:$PORT/#optimize/optedit" 2>/dev/null >"$OUT/page.html" || true
if grep -q 'data-optsaved="[^"]*maxRuin&quot;:0.03' "$OUT/page.html" && grep -q 'data-optdirty="1"' "$OUT/page.html"; then echo "ok      #optimize/optedit"; else echo "FEHLER  #optimize/optedit $(grep -o 'data-optsaved="[^"]*"' "$OUT/page.html")"; fail=1; fi
# Duration estimate for scenarios and optimization
for r in "optimize/bootstrap:Geschätzte Dauer der Optimierung" "scenarios/bootstrap/nosweep:Geschätzte Dauer der Simulation" "scenarios/bootstrap:Geschätzte Dauer von «Simulation starten»" "scenarios/bootstrap/sweep2:Geschätzte Dauer von «Simulation starten»"; do
  "$CHROME" --headless=new --disable-gpu --virtual-time-budget=30000 --dump-dom "http://127.0.0.1:$PORT/#${r%%:*}" 2>/dev/null >"$OUT/page.html" || true
  if grep -q "${r#*:}" "$OUT/page.html" && ! grep -q 'data-errors=' "$OUT/page.html"; then echo "ok      #${r%%:*} (Dauerschätzung)"; else echo "FEHLER  #${r%%:*} (Dauerschätzung fehlt)"; fail=1; fi
done
# Optimization: parameters fixed or in a range (unit of the parameter) are saved with the plan
"$CHROME" --headless=new --disable-gpu --virtual-time-budget=30000 --dump-dom "http://127.0.0.1:$PORT/#optimize/rangeedit" 2>/dev/null >"$OUT/page.html" || true
if grep -q 'id="optR-three_buckets-bondYears-lo"' "$OUT/page.html" && grep -q 'id="optR-guardrails-floor-var"' "$OUT/page.html" \
   && grep -q 'data-ranges="[^"]*cashYears&quot;:{&quot;fixed&quot;:true' "$OUT/page.html" && grep -q 'bondYears&quot;:{&quot;lo&quot;:0,&quot;hi&quot;:2' "$OUT/page.html" \
   && grep -q 'refillThreshold&quot;:{&quot;lo&quot;:-0.3,&quot;hi&quot;:0.1' "$OUT/page.html" && grep -q 'data-dirtyr="1"' "$OUT/page.html" && ! grep -q 'data-errors=' "$OUT/page.html"; then echo "ok      #optimize/rangeedit"; else echo "FEHLER  #optimize/rangeedit $(grep -o 'data-ranges="[^"]*"' "$OUT/page.html")"; fail=1; fi
# Taxes: property gains tax with the Zurich template (editable tables), purchase date for owned properties
"$CHROME" --headless=new --disable-gpu --virtual-time-budget=30000 --dump-dom "http://127.0.0.1:$PORT/#taxes/gainzh" 2>/dev/null >"$OUT/page.html" || true
if grep -q 'data-path="taxes.propertyGain.brackets.6.1"' "$OUT/page.html" && grep -q 'data-path="taxes.propertyGain.duration.18.1"' "$OUT/page.html" \
   && grep -q 'data-path="taxes.propertyGain.exemptBelow"' "$OUT/page.html" && ! grep -q 'data-errors=' "$OUT/page.html"; then echo "ok      #taxes/gainzh"; else echo "FEHLER  #taxes/gainzh (Grundstückgewinnsteuer)"; fail=1; fi
"$CHROME" --headless=new --disable-gpu --virtual-time-budget=30000 --dump-dom "http://127.0.0.1:$PORT/#realestate" 2>/dev/null >"$OUT/page.html" || true
if grep -q 'data-path="realestate.properties.0.bought"' "$OUT/page.html" && ! grep -q 'data-errors=' "$OUT/page.html"; then echo "ok      #realestate (Kaufdatum)"; else echo "FEHLER  #realestate (Kaufdatum fehlt)"; fail=1; fi
# Annual values: percentile selection, spending and withdrawal columns. The tax columns are collapsed by default
# (#7 to #12) and the smoke test cannot click: it therefore checks the collapsed default (button
# «Steuern anzeigen», no column th.tx). The user interface tests in e2e/ check the expanded tax columns (#75).
"$CHROME" --headless=new --disable-gpu --virtual-time-budget=60000 --dump-dom "http://127.0.0.1:$PORT/#results/bootstrap/table" 2>/dev/null >"$OUT/page.html" || true
if grep -q 'data-action="detail" data-p="5"' "$OUT/page.html" && grep -q 'data-action="tax-cols" aria-pressed="false"[^>]*>Steuern anzeigen' "$OUT/page.html" && ! grep -q '<th class="tx"' "$OUT/page.html" \
   && grep -q 'Ausgaben total' "$OUT/page.html" && grep -q '>Abweichung vom Plan</th>' "$OUT/page.html" && grep -q 'Einkommen total' "$OUT/page.html" && ! grep -q 'Miete / Hypothek' "$OUT/page.html" && ! grep -q 'data-errors=' "$OUT/page.html"; then echo "ok      #results/bootstrap/table (Jahreswerte)"; else echo "FEHLER  #results/bootstrap/table (Jahreswerte unvollständig)"; fail=1; fi
# Duration of the simulation next to the button «Simulation starten»
"$CHROME" --headless=new --disable-gpu --virtual-time-budget=30000 --dump-dom "http://127.0.0.1:$PORT/#scenarios/bootstrap" 2>/dev/null >"$OUT/page.html" || true
if grep -q 'id="est-run"[^>]*>Dauer: ' "$OUT/page.html"; then echo "ok      #scenarios/bootstrap (Dauer beim Knopf)"; else echo "FEHLER  Dauer beim Knopf «Simulation starten» fehlt"; fail=1; fi
# Help: load all topics, check links and mappings
"$CHROME" --headless=new --disable-gpu --virtual-time-budget=30000 --dump-dom "http://127.0.0.1:$PORT/#helpcheck" 2>/dev/null >"$OUT/page.html" || true
if grep -q 'data-help-problems="0"' "$OUT/page.html"; then echo "ok      #helpcheck"; else echo "FEHLER  #helpcheck $(grep -o 'data-help-problems="[^"]*"\|data-help-first="[^"]*"' "$OUT/page.html" | tr '\n' ' ')"; fail=1; fi
exit $fail
