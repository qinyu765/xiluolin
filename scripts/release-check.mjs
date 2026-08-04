import { readFile } from "node:fs/promises";
import process from "node:process";

const packageJson = JSON.parse(await readFile("package.json", "utf8"));
const tauriConfig = JSON.parse(
  await readFile("src-tauri/tauri.conf.json", "utf8"),
);
const cargoToml = await readFile("src-tauri/Cargo.toml", "utf8");
const cargoVersion = cargoToml.match(/^version\s*=\s*"([^"]+)"$/m)?.[1];

const versions = new Map([
  ["package.json", packageJson.version],
  ["src-tauri/Cargo.toml", cargoVersion],
  ["src-tauri/tauri.conf.json", tauriConfig.version],
]);
const expectedVersion = packageJson.version;
const semverPattern =
  /^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)(?:-[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?(?:\+[0-9A-Za-z-]+(?:\.[0-9A-Za-z-]+)*)?$/;

if (!semverPattern.test(expectedVersion)) {
  throw new Error(`package.json version 不是有效 SemVer: ${expectedVersion}`);
}

for (const [source, version] of versions) {
  if (version !== expectedVersion) {
    throw new Error(
      `版本不一致: ${source}=${version ?? "<missing>"}, expected=${expectedVersion}`,
    );
  }
}

const releaseTag = process.env.RELEASE_TAG;
if (releaseTag && releaseTag !== `v${expectedVersion}`) {
  throw new Error(
    `发布标签不匹配: RELEASE_TAG=${releaseTag}, expected=v${expectedVersion}`,
  );
}

console.log(
  releaseTag
    ? `版本与发布标签一致: ${releaseTag}`
    : `项目版本一致: ${expectedVersion}`,
);
