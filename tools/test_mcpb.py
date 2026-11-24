"""#213: one-click package for Claude Desktop (tools/mcpb.py). Run: python3 -m unittest tools/test_mcpb.py

Builds the package from the command line of the debug build (built here if missing) and checks it like the package
build in pakete.yml does: manifest, unpacking, start of `vivaplan mcp`, tool list.
"""
import json
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


def cli() -> Path:
    if not BINARY.exists():
        subprocess.run(["cargo", "build", "-q", "-p", "vivaplan-cli"], cwd=mcpb.ROOT, check=True)
    return BINARY


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
        self.assertEqual(mcpb.check(self.package), [])
        m = self.manifest
        self.assertEqual(m["version"], mcpb.app_version())
        self.assertEqual(m["server"]["mcp_config"], {"command": "${__dirname}/server/vivaplan", "args": ["mcp"]})
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
            "instead of ['mcp']": {**good, "server": {**good["server"], "mcp_config": {**good["server"]["mcp_config"], "args": []}}},
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

    def test_first_sentence(self):
        self.assertEqual(mcpb.first_sentence("Open a plan. Then more."), "Open a plan.")
        self.assertEqual(mcpb.first_sentence("No full stop"), "No full stop")
        self.assertEqual(mcpb.first_sentence("Two\nlines. x"), "Two lines.")


if __name__ == "__main__":
    unittest.main()
