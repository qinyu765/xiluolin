import fs from "node:fs";
import path from "node:path";
import Module from "node:module";
import { pathToFileURL } from "node:url";

const pnpmDir = path.resolve("node_modules/.pnpm");
const vitePackageDir = fs.existsSync(pnpmDir)
  ? fs.readdirSync(pnpmDir).find((name) => name.startsWith("vite@"))
  : undefined;

const nodePaths = [path.resolve("node_modules")];

if (vitePackageDir) {
  nodePaths.unshift(path.resolve(pnpmDir, vitePackageDir, "node_modules"));
}

process.env.NODE_PATH = [...nodePaths, process.env.NODE_PATH]
  .filter(Boolean)
  .join(path.delimiter);
Module._initPaths();

const viteEntry = vitePackageDir
  ? pathToFileURL(
      path.resolve(
        pnpmDir,
        vitePackageDir,
        "node_modules/vite/dist/node/index.js",
      ),
    ).href
  : "vite";

const { createServer } = await import(viteEntry);

const server = await createServer();

await server.listen();
server.printUrls();
