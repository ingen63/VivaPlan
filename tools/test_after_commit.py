"""Tests for the Claude Code hook after a commit (.claude/hooks/after-commit.sh, #160).

Usage: python3 -m unittest discover -s tools -p 'test_after_commit.py'

Every test creates a small git repository and replaces `cargo` with a script that records its arguments and
prints a summary like nextest. This checks which packages the hook tests and what it reports to Claude, without
running the real tests.
"""
import json
import os
import subprocess
import tempfile
import unittest
from pathlib import Path

HOOK = Path(__file__).resolve().parents[1] / ".claude" / "hooks" / "after-commit.sh"

FAKE_CARGO = """#!/bin/sh
echo "$@" >> "$FAKE_CARGO_ARGS"
if [ "${FAKE_CARGO_EXIT:-0}" != 0 ]; then
  echo "        FAIL [   0.010s] vivaplan-engine::all module::red"
  echo "     Summary [   1.000s] 3 tests run: 2 passed, 1 failed"
  exit 1
fi
echo "     Summary [   1.000s] 3 tests run: 3 passed"
"""


class AfterCommitHook(unittest.TestCase):
    def setUp(self):
        self.tmp = tempfile.TemporaryDirectory()
        root = Path(self.tmp.name)
        self.repo = root / "repo"
        self.bin = root / "bin"
        self.bin.mkdir()
        cargo = self.bin / "cargo"
        cargo.write_text(FAKE_CARGO)
        cargo.chmod(0o755)
        self.args = root / "cargo-args.txt"
        self.repo.mkdir()
        self.git("init", "-q")
        self.git("config", "user.email", "test@example.invalid")
        self.git("config", "user.name", "Test")
        self.write("Cargo.toml", "[workspace]\n")
        for p in ("engine/src/lib.rs", "engine/tests/all.rs", "app/src/main.rs", "cli/src/main.rs"):
            self.write("crates/" + p, "// start\n")
        self.write("docs/README.md", "start\n")
        self.write("fuzz/src/lib.rs", "// start\n")
        self.commit("Start")
        # The first commit counts as checked
        self.run_hook(exit_code=0)
        self.args.unlink(missing_ok=True)

    def tearDown(self):
        self.tmp.cleanup()

    def git(self, *args):
        subprocess.run(["git", *args], cwd=self.repo, check=True, capture_output=True)

    def write(self, rel, text):
        p = self.repo / rel
        p.parent.mkdir(parents=True, exist_ok=True)
        p.write_text(text)

    def append(self, rel, text):
        with open(self.repo / rel, "a") as f:
            f.write(text)

    def commit(self, msg):
        self.git("add", "-A")
        self.git("commit", "-q", "-m", msg)

    def run_hook(self, command="git commit -m x", exit_code=0):
        env = dict(os.environ)
        env.update(
            PATH=f"{self.bin}:{env['PATH']}",
            CLAUDE_PROJECT_DIR=str(self.repo),
            FAKE_CARGO_ARGS=str(self.args),
            FAKE_CARGO_EXIT=str(exit_code),
        )
        r = subprocess.run(
            ["bash", str(HOOK)],
            input=json.dumps({"tool_input": {"command": command}}),
            env=env,
            capture_output=True,
            text=True,
        )
        self.assertEqual(r.returncode, 0, r.stderr)
        return json.loads(r.stdout) if r.stdout.strip() else None

    def cargo_calls(self):
        return self.args.read_text().splitlines() if self.args.exists() else []

    # #160/AK6: green tests with a test in the commit: only the summary, no subagent
    def test_issue_160_ak6_green_with_test_reports_summary_only(self):
        self.append("crates/engine/src/lib.rs", "fn f() {}\n#[test]\nfn t() {}\n")
        self.commit("Engine with test")
        out = self.run_hook()
        self.assertNotIn("decision", out)
        ctx = out["hookSpecificOutput"]["additionalContext"]
        self.assertIn("grün", ctx)
        self.assertIn("3 tests run: 3 passed", ctx)
        self.assertNotIn("rust-test-agent", ctx)
        calls = self.cargo_calls()
        self.assertEqual(len(calls), 1)
        self.assertIn("nextest run --workspace", calls[0])
        self.assertIn("--status-level fail", calls[0])

    # #160/AK6: an engine change without a test in the commit asks for the subagent
    def test_issue_160_ak6_engine_change_without_test_asks_for_agent(self):
        self.append("crates/engine/src/lib.rs", "fn g() {}\n")
        self.commit("Engine without test")
        out = self.run_hook()
        self.assertEqual(out["decision"], "block")
        self.assertIn("rust-test-agent", out["reason"])
        self.assertIn("ohne neuen oder geänderten Test", out["reason"])

    # #160/AK6: a changed integration test counts as a test
    def test_issue_160_ak6_integration_test_counts_as_test(self):
        self.append("crates/engine/src/lib.rs", "fn h() {}\n")
        self.append("crates/engine/tests/all.rs", "// new test\n")
        self.commit("Engine and integration test")
        out = self.run_hook()
        self.assertNotIn("decision", out)

    # #160/AK6: failures ask for the subagent and name the tests
    def test_issue_160_ak6_failures_ask_for_agent_with_failed_tests(self):
        self.append("crates/cli/src/main.rs", "fn k() {}\n")
        self.commit("Command line")
        out = self.run_hook(exit_code=1)
        self.assertEqual(out["decision"], "block")
        self.assertIn("fehlgeschlagen", out["reason"])
        self.assertIn("module::red", out["reason"])
        self.assertIn("rust-test-agent", out["reason"])
        self.assertIn("1 failed", out["reason"])

    # #160/AK6: only the changed packages (without the engine: no --workspace)
    def test_issue_160_ak6_only_changed_packages(self):
        self.append("crates/app/src/main.rs", "fn a() {}\n")
        self.append("crates/cli/src/main.rs", "fn c() {}\n")
        self.commit("App and command line")
        out = self.run_hook()
        self.assertIn("grün", out["hookSpecificOutput"]["additionalContext"])
        (call,) = self.cargo_calls()
        self.assertIn("-p vivaplan-app -p vivaplan-cli", call)
        self.assertNotIn("--workspace", call)

    def test_no_reaction_without_rust_changes_other_commands_or_agent_commits(self):
        self.append("docs/README.md", "more\n")
        self.commit("Docs only")
        self.assertIsNone(self.run_hook())
        self.append("fuzz/src/lib.rs", "fn z() {}\n")
        self.commit("Fuzz target only (separate workspace)")
        self.assertIsNone(self.run_hook())
        self.append("crates/cli/src/main.rs", "fn d() {}\n")
        self.commit("test(auto): tests from the agent")
        self.assertIsNone(self.run_hook())
        self.assertIsNone(self.run_hook(command="git status"))
        self.assertEqual(self.cargo_calls(), [])

    def test_same_commit_is_checked_only_once(self):
        self.append("crates/cli/src/main.rs", "fn e() {}\n")
        self.commit("Command line")
        self.assertIsNotNone(self.run_hook())
        # e.g. a failed `git commit` afterwards: HEAD unchanged
        self.assertIsNone(self.run_hook())
        self.assertEqual(len(self.cargo_calls()), 1)


if __name__ == "__main__":
    unittest.main()
