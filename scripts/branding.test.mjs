import { readFile } from "node:fs/promises";

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

const indexHtml = await readFile(new URL("../app/index.html", import.meta.url), "utf8");
const appJs = await readFile(new URL("../app/app.js", import.meta.url), "utf8");
const readme = await readFile(new URL("../README.md", import.meta.url), "utf8");
const mainRs = await readFile(new URL("../src-tauri/src/main.rs", import.meta.url), "utf8");

assert(
  indexHtml.includes("by 엘리시움 사과팬케이크"),
  "App brand attribution should include: by 엘리시움 사과팬케이크",
);

assert(
  mainRs.includes('windows_subsystem = "windows"'),
  "Release Windows builds should use windows_subsystem = \"windows\" so no console or PowerShell window appears behind the app",
);

assert(
  !appJs.includes("repeat_ignore_window") && !readme.includes("연타 무시 시간"),
  "Ignore-window settings should not appear in the app or README; cycle behavior is count-based",
);
