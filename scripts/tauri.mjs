import { spawn } from "node:child_process";
import path from "node:path";
import { fileURLToPath } from "node:url";

const args = process.argv.slice(2);
const hasRunner = args.includes("--runner") || args.includes("-r");
const shouldSignMacosDev =
  process.platform === "darwin" && args[0] === "dev" && !hasRunner;

if (shouldSignMacosDev) {
  const repositoryRoot = path.resolve(
    path.dirname(fileURLToPath(import.meta.url)),
    "..",
  );
  args.splice(
    1,
    0,
    "--runner",
    path.join(repositoryRoot, "scripts/run-macos-personal-dev.sh"),
  );
}

const isWindows = process.platform === "win32";
const tauriCommand = isWindows ? (process.env.ComSpec ?? "cmd.exe") : "tauri";
const tauriArgs = isWindows ? ["/d", "/s", "/c", "tauri.cmd", ...args] : args;
const child = spawn(tauriCommand, tauriArgs, {
  stdio: "inherit",
  shell: false,
});

child.on("error", (error) => {
  console.error(`无法启动 Tauri CLI：${error.message}`);
  process.exitCode = 1;
});

child.on("exit", (code, signal) => {
  if (signal) {
    process.exitCode = 1;
  } else {
    process.exitCode = code ?? 1;
  }
});
