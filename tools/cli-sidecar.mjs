// #206: Builds the command line `vivaplan` (release) and puts it where Tauri expects a sidecar:
// crates/app/binaries/vivaplan-<target triple>[.exe]. Started by `npx tauri build --config tauri.cli.conf.json`
// (beforeBuildCommand); the package then contains `vivaplan` next to `vivaplan-app`.
import { execFileSync } from "node:child_process";
import { copyFileSync, mkdirSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

const root = join(dirname(fileURLToPath(import.meta.url)), "..");
const host = execFileSync("rustc", ["-vV"], { encoding: "utf8" }).match(/^host: (\S+)$/m)[1];
const triple = process.env.TAURI_ENV_TARGET_TRIPLE || host;
const exe = triple.includes("windows") ? ".exe" : "";
const args = ["build", "--release", "-p", "vivaplan-cli"];
if (triple !== host) args.push("--target", triple);
execFileSync("cargo", args, { cwd: root, stdio: "inherit" });

const built = join(root, "target", ...(triple !== host ? [triple] : []), "release", `vivaplan${exe}`);
const dir = join(root, "crates", "app", "binaries");
mkdirSync(dir, { recursive: true });
copyFileSync(built, join(dir, `vivaplan-${triple}${exe}`));
console.log(`cli-sidecar: ${built} -> ${join(dir, `vivaplan-${triple}${exe}`)}`);
