import assert from "node:assert/strict";
import { chmod, mkdtemp, rm, writeFile } from "node:fs/promises";
import { tmpdir } from "node:os";
import path from "node:path";
import { spawnSync } from "node:child_process";
import { after, before, test } from "node:test";
import { fileURLToPath } from "node:url";

const repositoryRoot = path.resolve(
  path.dirname(fileURLToPath(import.meta.url)),
  "..",
);
const scriptPath = path.join(repositoryRoot, "scripts/build-macos-signed.sh");

let fakeBinDirectory;

before(async () => {
  fakeBinDirectory = await mkdtemp(path.join(tmpdir(), "xiluolin-signing-"));
  const fakePnpm = path.join(fakeBinDirectory, "pnpm");
  await writeFile(
    fakePnpm,
    [
      "#!/bin/sh",
      'printf "identity=%s\\n" "$APPLE_SIGNING_IDENTITY"',
      'printf "macos_target=%s\\n" "$MACOSX_DEPLOYMENT_TARGET"',
      'printf "cmake_target=%s\\n" "$CMAKE_OSX_DEPLOYMENT_TARGET"',
      'printf "args=%s\\n" "$*"',
    ].join("\n"),
  );
  await chmod(fakePnpm, 0o755);
});

after(async () => {
  await rm(fakeBinDirectory, { recursive: true, force: true });
});

function runBuild(identity) {
  const env = {
    ...process.env,
    PATH: `${fakeBinDirectory}:${process.env.PATH}`,
  };
  if (identity === undefined) {
    delete env.APPLE_SIGNING_IDENTITY;
  } else {
    env.APPLE_SIGNING_IDENTITY = identity;
  }
  return spawnSync("sh", [scriptPath], {
    cwd: repositoryRoot,
    encoding: "utf8",
    env,
  });
}

test("拒绝缺失的 Apple Development 签名身份", () => {
  const result = runBuild(undefined);

  assert.equal(result.status, 2);
  assert.match(result.stderr, /APPLE_SIGNING_IDENTITY/);
});

test("拒绝会导致权限身份漂移的 ad-hoc 签名", () => {
  const result = runBuild("-");

  assert.equal(result.status, 2);
  assert.match(result.stderr, /Apple Development/);
});

test("稳定签名入口向真实构建命令传递身份和部署目标", () => {
  const result = runBuild("Apple Development: Personal Developer (TEAMID)");

  assert.equal(result.status, 0, result.stderr);
  assert.equal(
    result.stdout,
    [
      "identity=Apple Development: Personal Developer (TEAMID)",
      "macos_target=13.0",
      "cmake_target=13.0",
      "args=tauri build --target aarch64-apple-darwin --bundles app,dmg",
      "",
    ].join("\n"),
  );
});
