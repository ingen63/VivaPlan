#!/usr/bin/env bash
# #165: Reports German comments and German test names in crates/, e2e/, fuzz/ and tools/ with file and line.
#
# Usage:  tools/check-english.sh                 all paths except those on the transition list
#         tools/check-english.sh --all crates/cli  one package, including paths on the transition list
#         tools/check-english.sh --gate          only once the transition list is empty (make test-quiet, workflow)
#         tools/check-english.sh --root DIR …    another repository root (tests)
#
# Rules, word list and transition list: tools/check_english.py. A line with «check-english: allow» is never reported.
# Exit code: 0 no hit, 1 hits, 2 usage error.
set -euo pipefail
exec python3 "$(dirname "$0")/check_english.py" "$@"
