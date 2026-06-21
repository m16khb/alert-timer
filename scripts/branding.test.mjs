import { readFile } from "node:fs/promises";

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

const indexHtml = await readFile(new URL("../app/index.html", import.meta.url), "utf8");

assert(
  indexHtml.includes("by 엘리시움 사과팬케이크"),
  "App brand attribution should include: by 엘리시움 사과팬케이크",
);
