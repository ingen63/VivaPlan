"""#309: update manifest `latest.json` for the updater of the desktop app.

The release workflow (pakete.yml) builds the packages with `createUpdaterArtifacts`; Tauri then signs the update
files with the private key (GitHub secret TAURI_SIGNING_PRIVATE_KEY) and writes a `.sig` next to each. This tool
collects them from one folder and writes the manifest the app reads from
`https://github.com/<repo>/releases/latest/download/latest.json`.

Usage:
  python3 tools/update_manifest.py <folder> --tag v1.0.0 --repo owner/name [--complete] [-o latest.json]

--complete: every platform must have its update file (a release for all users); without it the platforms found.

Update files per platform (the first one found counts):
  darwin-aarch64  *.app.tar.gz (the macOS runner builds for Apple silicon only)
  windows-x86_64  *-setup.exe (NSIS), otherwise *.msi
  linux-x86_64    *.AppImage
The changes of the version (`notes`) are a JSON object per language from the CHANGELOG files; the app shows the
language of its user interface. Exit 1 if a platform has a file without signature or no platform was found.
"""

import argparse
import datetime
import json
import pathlib
import sys

TOOLS = pathlib.Path(__file__).resolve().parent
sys.path.insert(0, str(TOOLS))
import changelog  # noqa: E402

ROOT = TOOLS.parent

# platform key of the updater -> ordered patterns of the update file
PLATFORMS = {
    "darwin-aarch64": ["*aarch64*.app.tar.gz", "*.app.tar.gz"],
    "windows-x86_64": ["*-setup.exe", "*.msi"],
    "linux-x86_64": ["*.AppImage"],
}


def find(folder, patterns):
    for pattern in patterns:
        hits = sorted(folder.glob(pattern))
        if hits:
            return hits[0]
    return None


def notes(root, version):
    """Changes of the version per language, as JSON text (missing languages are left out)."""
    out = {}
    for lang in changelog.LANGS:
        path = root / changelog.FILES[lang]
        if not path.exists():
            continue
        entries = {v: body for v, _, body in changelog.parse(path.read_text(encoding="utf-8"))}
        if version in entries:
            out[lang] = "\n".join(entries[version]).strip()
    return json.dumps(out, ensure_ascii=False)


def manifest(folder, tag, repo, root=ROOT, now=None, complete=False):
    """The manifest as dict; raises ValueError for a file without signature, without any platform or, with
    `complete`, for a missing platform."""
    version = tag[1:] if tag.startswith("v") else tag
    platforms = {}
    for key, patterns in PLATFORMS.items():
        file = find(folder, patterns)
        if file is None:
            continue
        sig = file.with_name(file.name + ".sig")
        if not sig.exists():
            raise ValueError(f"{file.name}: signature {sig.name} missing")
        # GitHub replaces spaces in asset names by dots
        asset = file.name.replace(" ", ".")
        platforms[key] = {
            "signature": sig.read_text(encoding="utf-8").strip(),
            "url": f"https://github.com/{repo}/releases/download/{tag}/{asset}",
        }
    if not platforms:
        raise ValueError(f"no update file in {folder}")
    missing = [k for k in PLATFORMS if k not in platforms]
    if complete and missing:
        raise ValueError(f"no update file for {', '.join(missing)} in {folder}")
    now = now or datetime.datetime.now(datetime.timezone.utc)
    return {
        "version": version,
        "notes": notes(root, version),
        "pub_date": now.strftime("%Y-%m-%dT%H:%M:%SZ"),
        "platforms": platforms,
    }


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    ap.add_argument("folder")
    ap.add_argument("--tag", required=True)
    ap.add_argument("--repo", required=True)
    ap.add_argument("--root", default=str(ROOT))
    ap.add_argument("--complete", action="store_true")
    ap.add_argument("-o", "--output")
    args = ap.parse_args(argv)
    try:
        m = manifest(pathlib.Path(args.folder), args.tag, args.repo, pathlib.Path(args.root), complete=args.complete)
    except (OSError, ValueError) as err:
        print(f"update_manifest: {err}", file=sys.stderr)
        return 1
    text = json.dumps(m, ensure_ascii=False, indent=2) + "\n"
    if args.output:
        pathlib.Path(args.output).write_text(text, encoding="utf-8")
    else:
        sys.stdout.write(text)
    print("update_manifest: " + ", ".join(sorted(m["platforms"])), file=sys.stderr)
    return 0


if __name__ == "__main__":
    sys.exit(main())
