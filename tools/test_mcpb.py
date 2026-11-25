"""#213: one-click package for Claude Desktop (tools/mcpb.py). Run: python3 -m unittest tools/test_mcpb.py

Builds the package from the command line of the debug build (built here if missing) and checks it like the package
build in pakete.yml does: manifest, unpacking, start of `vivaplan mcp`, tool list.
"""
import json
import os
import stat
import subprocess
import sys
import tempfile
import unittest
import zipfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))
import mcpb  # noqa: E402

BINARY = mcpb.ROOT / "target" / "debug" / "vivaplan"
TOOL = mcpb.ROOT / "target" / "debug" / "beta-token"
# #270/AK11: the debug build checks the beta tokens against the test key of the engine (engine::beta::TEST_SEED)
os.environ["VIVAPLAN_BETA_TEST_KEY"] = "1"
TEST_SEED = b"vivaplan-beta-test-key-NOT-REAL!"


def cli() -> Path:
    if not (BINARY.exists() and TOOL.exists()):
        subprocess.run(["cargo", "build", "-q", "-p", "vivaplan-cli"], cwd=mcpb.ROOT, check=True)
    return BINARY


def test_token(until: str = "2099-12-31", seed: bytes = TEST_SEED) -> str:
    """#270: a beta token of the test key, made with the developer tool beta-token."""
    cli()
    with tempfile.TemporaryDirectory() as tmp:
        key = Path(tmp) / "test.key"
        key.write_text(seed.hex())
        run = subprocess.run([str(TOOL), "issue", "--key", str(key), "--who", "AB", "--until", until],
                             capture_output=True, text=True, check=True)
    return run.stdout.strip()


class McpbTest(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.tmp = tempfile.TemporaryDirectory()
        cls.package = mcpb.build(cli(), Path(cls.tmp.name))
        with zipfile.ZipFile(cls.package) as z:
            cls.names = set(z.namelist())
            cls.manifest = json.loads(z.read("manifest.json"))
            cls.entry_mode = z.getinfo(mcpb.ENTRY).external_attr >> 16

    @classmethod
    def tearDownClass(cls):
        cls.tmp.cleanup()

    def test_issue_213_ak1_package_has_valid_manifest_and_starts_vivaplan_mcp(self):
        # #213/AK1: name of the file per version and architecture, manifest without errors, server starts
        self.assertRegex(self.package.name, r"^VivaPlan_\d+\.\d+\.\d+.*_(aarch64|x86_64)\.mcpb$")
        self.assertEqual(mcpb.check_manifest(self.manifest, self.names), [])
        notes = []
        self.assertEqual(mcpb.check(self.package, notes=notes), [])
        self.assertTrue(any("needs an activation" in n for n in notes), notes)
        # #270: with a beta token the plan checks run as well
        notes = []
        self.assertEqual(mcpb.check(self.package, test_token(), notes), [])
        self.assertEqual(notes, [])
        m = self.manifest
        self.assertEqual(m["version"], mcpb.app_version())
        self.assertEqual(m["server"]["mcp_config"]["command"], "${__dirname}/server/vivaplan")
        self.assertEqual(m["server"]["mcp_config"]["args"][0], "mcp")
        self.assertEqual(m["compatibility"]["platforms"], ["darwin"])
        self.assertTrue(stat.S_ISREG(self.entry_mode) and self.entry_mode & 0o111, oct(self.entry_mode))
        self.assertEqual(self.names, {"manifest.json", "icon.png", "server/vivaplan"})

    def test_issue_213_ak1_tool_list_matches_the_server(self):
        # the tool list of the manifest is the server's (crates/cli/tests/mcp-tools.json), each with a short text
        tools = json.loads((mcpb.ROOT / "crates" / "cli" / "tests" / "mcp-tools.json").read_text())
        self.assertEqual([t["name"] for t in self.manifest["tools"]], [t["name"] for t in tools])
        for t in self.manifest["tools"]:
            self.assertTrue(t["description"].endswith((".", "!", "?")), t)
            self.assertNotIn("\n", t["description"])

    def test_issue_213_ak1_check_reports_broken_manifests(self):
        good = self.manifest
        cases = {
            "field version missing": {k: v for k, v in good.items() if k != "version"},
            "is not semantic versioning": {**good, "version": "1.0"},
            "does not start the entry point": {**good, "server": {**good["server"], "mcp_config": {"command": "vivaplan", "args": ["mcp"]}}},
            "mcp_config.args": {**good, "server": {**good["server"], "mcp_config": {**good["server"]["mcp_config"], "args": ["mcp"]}}},
            "is not in the package": {**good, "server": {**good["server"], "entry_point": "server/other"}},
            "instead of 'binary'": {**good, "server": {**good["server"], "type": "node"}},
            "only lower case": {**good, "name": "VivaPlan"},
            "platforms": {**good, "compatibility": {}},
        }
        for want, m in cases.items():
            with self.subTest(want):
                errors = mcpb.check_manifest(m, self.names)
                self.assertTrue(any(want in e for e in errors), errors)

    def test_issue_213_ak1_check_rejects_package_without_executable_or_manifest(self):
        with tempfile.TemporaryDirectory() as tmp:
            bad = Path(tmp) / "bad.mcpb"
            with zipfile.ZipFile(bad, "w") as z:
                z.writestr("readme.txt", "x")
            self.assertEqual(mcpb.check(bad), ["manifest.json missing at the root of the package"])
            # entry point without execute permission and not a program: both are reported
            with zipfile.ZipFile(bad, "w") as z:
                z.writestr("manifest.json", json.dumps(self.manifest))
                z.writestr("icon.png", b"")
                info = zipfile.ZipInfo(mcpb.ENTRY)
                info.create_system = 3
                info.external_attr = (stat.S_IFREG | 0o644) << 16
                z.writestr(info, b"not a program")
            errors = mcpb.check(bad)
            self.assertTrue(any("not executable" in e for e in errors), errors)
            self.assertTrue(any("server does not start" in e for e in errors), errors)

    def test_issue_213_ak1_every_version_builds_and_uploads_the_package_for_macos(self):
        # the package build (pakete.yml) builds the package from the program in the .app, checks it and uploads it
        flow = (mcpb.ROOT / ".github" / "workflows" / "pakete.yml").read_text()
        step = flow[flow.index("name: Ein-Klick-Paket für Claude Desktop"):]
        step = step[: step.index("- name:", 1)]
        self.assertIn("if: matrix.name == 'macos'", step)
        self.assertIn("tools/mcpb.py build --binary target/release/bundle/macos/VivaPlan.app/Contents/MacOS/vivaplan", step)
        self.assertIn('tools/mcpb.py check "$mcpb"', step)
        self.assertIn("target/release/bundle/mcpb/*.mcpb", flow[flow.index("upload-artifact"):])

    def test_issue_263_ak1_ak5_manifest_asks_for_plan_folder_and_read_only(self):
        # #263/AK1: one folder, required, passed as --allow-dir; AK5: switch «read only», default off, as --read-only=…
        config = self.manifest["user_config"]
        self.assertEqual(config["plan_folder"]["type"], "directory")
        self.assertIs(config["plan_folder"]["required"], True)
        self.assertNotIn("multiple", config["plan_folder"])
        self.assertEqual(config["read_only"]["type"], "boolean")
        self.assertIs(config["read_only"]["default"], False)
        args = self.manifest["server"]["mcp_config"]["args"]
        self.assertEqual(args, ["mcp", "--allow-dir", "${user_config.plan_folder}", "--read-only=${user_config.read_only}",
                                "--beta-token=${user_config.beta_token}"])
        self.assertEqual(mcpb.with_config(args, {"plan_folder": Path("/a b"), "read_only": True, "beta_token": "VP1.x"}),
                         ["mcp", "--allow-dir", "/a b", "--read-only=true", "--beta-token=VP1.x"])
        self.assertEqual(mcpb.with_config(args, {"plan_folder": "/a", "read_only": False})[-2], "--read-only=false")
        self.assertIn("plan folder", self.manifest["long_description"])

    def test_issue_263_ak1_check_reports_missing_plan_folder_or_switch(self):
        good = self.manifest
        config = good["user_config"]
        cases = {
            "plan_folder": {**config, "plan_folder": {**config["plan_folder"], "required": False}},
            "plan_folder ": {**config, "plan_folder": {**config["plan_folder"], "multiple": True}},
            "plan_folder  ": {k: v for k, v in config.items() if k != "plan_folder"},
            "read_only": {**config, "read_only": {**config["read_only"], "default": True}},
            "without title": {**config, "read_only": {**config["read_only"], "title": ""}},
        }
        for want, c in cases.items():
            with self.subTest(want):
                errors = mcpb.check_manifest({**good, "user_config": c}, self.names)
                self.assertTrue(any(want.strip() in e for e in errors), errors)

    def test_issue_263_ak2_ak3_check_proves_the_plan_folder(self):
        # #263/AK2, AK3: with the plan folder saving and opening there work, a plan outside is refused; a server that
        # ignores the folder is reported
        with tempfile.TemporaryDirectory() as tmp:
            plans, other = Path(tmp) / "plans", Path(tmp) / "other"
            plans.mkdir()
            other.mkdir()
            data = Path(tmp) / "data"
            data.mkdir()
            limited = [str(cli()), "mcp", "--data-dir", str(data), f"--beta-token={test_token()}", "--allow-dir", str(plans)]
            self.assertEqual(mcpb.check_plan_folder(limited, plans, other), [])
            self.assertTrue((plans / "check.vplan").is_file())
            (plans / "check.vplan").unlink()
            too_wide = [*limited, "--allow-dir", str(other)]
            errors = mcpb.check_plan_folder(too_wide, plans, other)
            self.assertTrue(any("outside the plan folder is opened" in e for e in errors), errors)
            (plans / "check.vplan").unlink()
            self.assertNotEqual(mcpb.check_plan_folder([str(cli()), "mcp", "--data-dir", str(data)], plans, other), [])
            # with «read only» plan_save is not a tool of the server
            with self.assertRaisesRegex(RuntimeError, "Unknown tool plan_save"):
                mcpb.check_plan_folder([*limited, "--read-only=true"], plans, other)

    def test_issue_270_ak8_manifest_asks_for_the_beta_token(self):
        # #270/AK8: optional, secret text field «Beta token», passed as --beta-token=…; the check reports a wrong field
        token = self.manifest["user_config"]["beta_token"]
        self.assertEqual((token["type"], token["required"], token["sensitive"]), ("string", False, True))
        self.assertEqual(token["title"], "Beta token")
        self.assertIn("beta token", self.manifest["long_description"])
        config = self.manifest["user_config"]
        for c in ({k: v for k, v in config.items() if k != "beta_token"},
                  {**config, "beta_token": {**token, "required": True}},
                  {**config, "beta_token": {**token, "type": "boolean"}}):
            errors = mcpb.check_manifest({**self.manifest, "user_config": c}, self.names)
            self.assertTrue(any("user_config.beta_token" in e for e in errors), errors)

    def test_issue_270_ak8_ak9_check_knows_locked_and_activated_servers(self):
        # #270/AK9: without an activation every call is refused (and the server answers); AK8: a token activates
        with tempfile.TemporaryDirectory() as tmp:
            data = Path(tmp)
            plain = [str(cli()), "mcp", "--data-dir", str(data)]
            self.assertTrue(mcpb.is_locked(plain))
            self.assertTrue(mcpb.is_locked([*plain, f"--beta-token={test_token('2020-01-01')}"]))
            self.assertTrue(mcpb.is_locked([*plain, f"--beta-token={test_token(seed=bytes(32))}"]))
            self.assertFalse(mcpb.is_locked([*plain, f"--beta-token={test_token()}"]))
            self.assertFalse(mcpb.is_locked(plain), "the activation stays")

    def test_first_sentence(self):
        self.assertEqual(mcpb.first_sentence("Open a plan. Then more."), "Open a plan.")
        self.assertEqual(mcpb.first_sentence("No full stop"), "No full stop")
        self.assertEqual(mcpb.first_sentence("Two\nlines. x"), "Two lines.")


if __name__ == "__main__":
    unittest.main()
