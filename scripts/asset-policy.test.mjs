import { createServer, get } from "node:http";
import { writeFile, mkdtemp, rm } from "node:fs/promises";
import { existsSync, readdirSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, extname } from "node:path";
import { spawn } from "node:child_process";

const rasterExtensions = new Set([".png", ".jpg", ".jpeg", ".gif"]);
const allowedNonWebpPaths = new Set();

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

function findForbiddenRasterAssets(root) {
  if (!existsSync(root)) {
    return [];
  }

  const results = [];
  const visit = (directory) => {
    for (const entry of readdirSync(directory, { withFileTypes: true })) {
      const fullPath = join(directory, entry.name);
      if (entry.isDirectory()) {
        visit(fullPath);
        continue;
      }

      const normalized = fullPath.replaceAll("\\", "/");
      if (rasterExtensions.has(extname(entry.name).toLowerCase()) && !allowedNonWebpPaths.has(normalized)) {
        results.push(normalized);
      }
    }
  };

  visit(root);
  return results;
}

async function getFreePort() {
  const server = createServer();
  await new Promise((resolve) => server.listen(0, "127.0.0.1", resolve));
  const address = server.address();
  await new Promise((resolve) => server.close(resolve));
  return address.port;
}

async function waitForServer(port) {
  const deadline = Date.now() + 5000;
  while (Date.now() < deadline) {
    try {
      await request(port, "/sample.webp");
      return;
    } catch {
      await new Promise((resolve) => setTimeout(resolve, 100));
    }
  }
  throw new Error("static server did not start in time");
}

function request(port, path) {
  return new Promise((resolve, reject) => {
    const req = get({ hostname: "127.0.0.1", port, path }, (response) => {
      response.resume();
      response.on("end", () => resolve(response));
    });
    req.on("error", reject);
  });
}

const forbiddenRasterAssets = findForbiddenRasterAssets("app");
assert(
  forbiddenRasterAssets.length === 0,
  `Use .webp for app raster images. Forbidden assets:\n${forbiddenRasterAssets.join("\n")}`,
);

const tempRoot = await mkdtemp(join(tmpdir(), "alert-timer-assets-"));
const port = await getFreePort();
let child;

try {
  await writeFile(join(tempRoot, "sample.webp"), new Uint8Array([0x52, 0x49, 0x46, 0x46]));
  child = spawn(process.execPath, ["scripts/static-server.mjs", tempRoot, String(port)], {
    stdio: "ignore",
  });

  await waitForServer(port);
  const response = await request(port, "/sample.webp");
  assert(response.statusCode === 200, `expected 200 for .webp asset, got ${response.statusCode}`);
  assert(
    response.headers["content-type"] === "image/webp",
    `expected .webp Content-Type to be image/webp, got ${response.headers["content-type"]}`,
  );
} finally {
  if (child) {
    child.kill();
  }
  await rm(tempRoot, { recursive: true, force: true });
}
