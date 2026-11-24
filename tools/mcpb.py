#!/usr/bin/env python3
"""#213: builds and checks the one-click package for Claude Desktop (.mcpb, MCP Bundle).

An .mcpb is a ZIP archive with `manifest.json` at its root (MCPB specification, formerly DXT). This package is of type
`binary`: it contains the command line `vivaplan` (server/vivaplan) and the app icon; Claude Desktop starts
`server/vivaplan mcp` over standard input and output. Only the Python standard library, no network access.

Usage:
  python3 tools/mcpb.py build --binary target/release/vivaplan --out target/release/bundle/mcpb
      writes VivaPlan_<version>_<arch>.mcpb and prints its path; the tool list comes from the server itself
  python3 tools/mcpb.py check FILE.mcpb
      checks the manifest, unpacks the package into a temporary folder, starts the server as Claude Desktop would
      (initialize, tools/list) and compares the tools; exit code 0 = in order, 1 = error
"""
import argparse
import json
import os
import platform
import re
import stat
import subprocess
import sys
import tempfile
import zipfile
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
APP = ROOT / "crates" / "app"
ICON = APP / "icons" / "128x128@2x.png"  # 256 × 256
MANIFEST_VERSION = "0.2"
ENTRY = "server/vivaplan"
PROTOCOL = "2025-06-18"
SEMVER = re.compile(r"^\d+\.\d+\.\d+(?:-[0-9A-Za-z.-]+)?(?:\+[0-9A-Za-z.-]+)?$")
# Required fields of the manifest and their type (MCPB specification, manifest_version 0.2)
REQUIRED = {"manifest_version": str, "name": str, "version": str, "description": str, "author": dict, "server": dict}


def app_version() -> str:
    """Version of the app (tauri.conf.json), the same as the command line's (workspace version)."""
    return json.loads((APP / "tauri.conf.json").read_text())["version"]


def arch() -> str:
    """Architecture in the naming of the Tauri packages (aarch64, x86_64)."""
    m = platform.machine().lower()
    return {"arm64": "aarch64", "amd64": "x86_64"}.get(m, m)


def first_sentence(text: str) -> str:
    """First sentence of a tool description (the manifest shows a short text per tool)."""
    m = re.match(r"(.+?[.!?])(\s|$)", text.strip(), re.S)
    return (m.group(1) if m else text.strip()).replace("\n", " ")


def ask_server(command: list[str], timeout: float = 60) -> tuple[dict, list[dict]]:
    """Starts the MCP server, sends initialize and tools/list, returns (initialize result, tools)."""
    lines = [
        {"jsonrpc": "2.0", "id": 1, "method": "initialize",
         "params": {"protocolVersion": PROTOCOL, "capabilities": {}, "clientInfo": {"name": "mcpb-check", "version": "1"}}},
        {"jsonrpc": "2.0", "method": "notifications/initialized"},
        {"jsonrpc": "2.0", "id": 2, "method": "tools/list", "params": {}},
    ]
    stdin = "".join(json.dumps(m) + "\n" for m in lines)
    run = subprocess.run(command, input=stdin, capture_output=True, text=True, timeout=timeout)
    answers = {}
    for line in run.stdout.splitlines():
        if line.strip():
            msg = json.loads(line)
            if "id" in msg:
                answers[msg["id"]] = msg
    if 1 not in answers or 2 not in answers:
        raise RuntimeError(f"server did not answer (exit {run.returncode}): {run.stderr.strip()[:500]}")
    for k in (1, 2):
        if "error" in answers[k]:
            raise RuntimeError(f"server error: {answers[k]['error']}")
    return answers[1]["result"], answers[2]["result"]["tools"]


def manifest(version: str, tools: list[dict]) -> dict:
    """Manifest of the package; texts in English like the server's."""
    return {
        "manifest_version": MANIFEST_VERSION,
        "name": "vivaplan",
        "display_name": "VivaPlan",
        "version": version,
        "description": "Swiss retirement planning: open, check, simulate, change and save VivaPlan plans on this computer.",
        "long_description": (
            "VivaPlan simulates a Swiss retirement plan month by month (income, pension fund, AHV, pillar 3a, taxes, "
            "assets). This extension starts the local VivaPlan server (`vivaplan mcp`). It reads only plans whose path "
            "you name, computes on this computer and opens no network connection; what a tool returns becomes part of "
            "the conversation with your AI. Names of persons are replaced by neutral ones. Results are model "
            "calculations without warranty, not tax or investment advice."
        ),
        "author": {"name": "VivaPlan", "url": "https://vivaplan-app.ch"},
        "homepage": "https://vivaplan-app.ch",
        "license": "GPL-3.0",
        "icon": "icon.png",
        "keywords": ["retirement", "pension", "switzerland", "planning", "simulation"],
        "server": {
            "type": "binary",
            "entry_point": ENTRY,
            "mcp_config": {"command": "${__dirname}/" + ENTRY, "args": ["mcp"]},
        },
        "tools": [{"name": t["name"], "description": first_sentence(t.get("description", ""))} for t in tools],
        "compatibility": {"platforms": ["darwin"]},
    }


def build(binary: Path, out_dir: Path) -> Path:
    """Writes the package; the tool list is read from the server (which proves that it starts)."""
    if not os.access(binary, os.X_OK):
        raise SystemExit(f"not an executable program: {binary}")
    version = app_version()
    info, tools = ask_server([str(binary), "mcp"])
    if info.get("serverInfo", {}).get("version") != version:
        raise SystemExit(f"version of the program {info.get('serverInfo')} differs from the app's {version}")
    out_dir.mkdir(parents=True, exist_ok=True)
    target = out_dir / f"VivaPlan_{version}_{arch()}.mcpb"
    with zipfile.ZipFile(target, "w", zipfile.ZIP_DEFLATED) as z:
        z.writestr("manifest.json", json.dumps(manifest(version, tools), indent=2, ensure_ascii=False) + "\n")
        z.write(ICON, "icon.png")
        entry = zipfile.ZipInfo(ENTRY, date_time=(2020, 1, 1, 0, 0, 0))
        entry.create_system = 3  # Unix, so that the permissions below count
        entry.external_attr = (stat.S_IFREG | 0o755) << 16
        entry.compress_type = zipfile.ZIP_DEFLATED
        z.writestr(entry, binary.read_bytes())
    return target


def check_manifest(m: dict, names: set[str]) -> list[str]:
    """Errors of the manifest against the specification (as far as known locally) and the package contents."""
    errors = []
    for key, typ in REQUIRED.items():
        if not isinstance(m.get(key), typ):
            errors.append(f"field {key} missing or not {typ.__name__}")
    if errors:
        return errors
    if m["manifest_version"] != MANIFEST_VERSION:
        errors.append(f"manifest_version {m['manifest_version']} instead of {MANIFEST_VERSION}")
    if not re.fullmatch(r"[a-z0-9-]+", m["name"]):
        errors.append(f"name {m['name']!r}: only lower case letters, digits, hyphen")
    if not SEMVER.match(m["version"]):
        errors.append(f"version {m['version']!r} is not semantic versioning")
    if not m["author"].get("name"):
        errors.append("author.name missing")
    server = m["server"]
    if server.get("type") != "binary":
        errors.append(f"server.type {server.get('type')!r} instead of 'binary'")
    entry = server.get("entry_point")
    if entry not in names:
        errors.append(f"server.entry_point {entry!r} is not in the package")
    cfg = server.get("mcp_config", {})
    if cfg.get("command") != "${__dirname}/" + str(entry):
        errors.append(f"mcp_config.command {cfg.get('command')!r} does not start the entry point")
    if cfg.get("args") != ["mcp"]:
        errors.append(f"mcp_config.args {cfg.get('args')!r} instead of ['mcp']")
    if m.get("icon") and m["icon"] not in names:
        errors.append(f"icon {m['icon']!r} is not in the package")
    if m.get("compatibility", {}).get("platforms") != ["darwin"]:
        errors.append("compatibility.platforms must be ['darwin']")
    for t in m.get("tools", []):
        if not (t.get("name") and t.get("description")):
            errors.append(f"tool without name or description: {t}")
    return errors


def check(package: Path) -> list[str]:
    """Checks the package like Claude Desktop would install it: unpack, manifest, start, tool list."""
    with zipfile.ZipFile(package) as z:
        names = set(z.namelist())
        if "manifest.json" not in names:
            return ["manifest.json missing at the root of the package"]
        m = json.loads(z.read("manifest.json"))
        errors = check_manifest(m, names)
        if errors:
            return errors
        entry = z.getinfo(m["server"]["entry_point"])
        if not (entry.external_attr >> 16) & 0o111:
            errors.append(f"{entry.filename} is not executable in the package")
        with tempfile.TemporaryDirectory() as tmp:
            z.extractall(tmp)
            program = Path(tmp) / entry.filename
            program.chmod((entry.external_attr >> 16) & 0o777 or 0o644)
            cfg = m["server"]["mcp_config"]
            command = [cfg["command"].replace("${__dirname}", tmp), *cfg["args"]]
            try:
                info, tools = ask_server(command)
            except (OSError, RuntimeError, subprocess.TimeoutExpired) as e:
                return errors + [f"server does not start: {e}"]
    if info.get("serverInfo", {}).get("name") != "vivaplan":
        errors.append(f"unexpected server {info.get('serverInfo')}")
    if info.get("serverInfo", {}).get("version") != m["version"]:
        errors.append(f"server version {info.get('serverInfo', {}).get('version')} differs from manifest {m['version']}")
    listed = [t["name"] for t in m.get("tools", [])]
    if listed != [t["name"] for t in tools]:
        errors.append(f"tools in the manifest {listed} differ from the server's {[t['name'] for t in tools]}")
    return errors


def main(argv: list[str]) -> int:
    p = argparse.ArgumentParser(description="One-click package for Claude Desktop (.mcpb)")
    sub = p.add_subparsers(dest="cmd", required=True)
    b = sub.add_parser("build")
    b.add_argument("--binary", type=Path, required=True)
    b.add_argument("--out", type=Path, required=True)
    c = sub.add_parser("check")
    c.add_argument("package", type=Path)
    a = p.parse_args(argv)
    if a.cmd == "build":
        print(build(a.binary, a.out))
        return 0
    errors = check(a.package)
    for e in errors:
        print(f"mcpb: {e}", file=sys.stderr)
    if not errors:
        print(f"mcpb: {a.package.name} in order")
    return 1 if errors else 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
