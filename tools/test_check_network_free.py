"""Tests for tools/check-network-free.sh (#157/AK1).

Usage: python3 -m unittest discover -s tools -p 'test_check_network_free.py'
The cases check given lists in test mode (`--liste`), without cargo.
"""

import pathlib
import subprocess
import tempfile
import unittest

SCRIPT = pathlib.Path(__file__).resolve().parent / "check-network-free.sh"


def check(lines):
    with tempfile.NamedTemporaryFile("w", suffix=".txt", delete=False) as f:
        f.write("\n".join(lines) + "\n")
        path = f.name
    try:
        return subprocess.run(["bash", str(SCRIPT), "--liste", path], capture_output=True, text=True)
    finally:
        pathlib.Path(path).unlink()


class NetworkFreeTest(unittest.TestCase):
    # #157/AK1: without a blocked library the check passes.
    def test_ak1_clean_list_passes(self):
        r = check(["vivaplan-app v1.0.0 (/path)", "serde v1.0.229", "tokio v1.0.0 (*)", "http v1.4.0"])
        self.assertEqual(r.returncode, 0, r.stderr)

    # #157/AK1: a hit from the blocklist fails the check and is named.
    def test_ak1_blocked_crate_fails(self):
        for crate in ["reqwest", "hyper", "ureq", "curl", "tokio-tungstenite", "tauri-plugin-http", "tauri-plugin-updater"]:
            with self.subTest(crate=crate):
                r = check(["serde v1.0.229", f"{crate} v0.1.0", f"{crate} v0.1.0 (*)"])
                self.assertEqual(r.returncode, 1)
                self.assertEqual(r.stderr.count(f"  {crate}\n"), 1, r.stderr)

    # #157/AK1: comparison by exact names, not substrings (dispatch2 is not h2, hyperx is not hyper).
    def test_ak1_exact_names_only(self):
        r = check(["dispatch2 v0.3.0", "hyperx v1.0.0", "curl-parser v0.1.0", "my-reqwest v0.1.0"])
        self.assertEqual(r.returncode, 0, r.stderr)

    # #157/AK1: wrong usage is a separate error (2), not a silent pass.
    def test_ak1_missing_list_is_a_usage_error(self):
        r = subprocess.run(["bash", str(SCRIPT), "--liste", "/does/not/exist"], capture_output=True, text=True)
        self.assertEqual(r.returncode, 2)


if __name__ == "__main__":
    unittest.main()
