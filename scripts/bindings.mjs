import { readFileSync } from "node:fs";
import { spawnSync } from "node:child_process";
import path from "node:path";
import process from "node:process";

const root = process.cwd();
const committed = path.join(root, "src/generated/tauri-bindings.ts");
const checkMode = process.argv.includes("--check");
const output = checkMode
  ? path.join(root, "src-tauri/target/bindings-check.ts")
  : committed;

const result = spawnSync(
  "cargo",
  [
    "run",
    "--quiet",
    "--manifest-path",
    "src-tauri/Cargo.toml",
    "--bin",
    "export_bindings",
    "--",
    output,
  ],
  { cwd: root, stdio: "inherit" },
);

if (result.status !== 0) {
  process.exit(result.status ?? 1);
}

if (
  checkMode &&
  readFileSync(output, "utf8") !== readFileSync(committed, "utf8")
) {
  console.error(
    "Tauri TypeScript 绑定已过期，请运行 pnpm bindings:generate。\n",
  );
  process.exit(1);
}

console.log(
  checkMode
    ? "Tauri TypeScript 绑定与 Rust 契约一致。"
    : "已生成 Tauri TypeScript 绑定。",
);
