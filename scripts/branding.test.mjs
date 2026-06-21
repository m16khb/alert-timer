import { readFile } from "node:fs/promises";

function assert(condition, message) {
  if (!condition) {
    throw new Error(message);
  }
}

const indexHtml = await readFile(new URL("../app/index.html", import.meta.url), "utf8");
const miniHtml = await readFile(new URL("../app/mini.html", import.meta.url), "utf8");
const appJs = await readFile(new URL("../app/app.js", import.meta.url), "utf8");
const miniJs = await readFile(new URL("../app/mini.js", import.meta.url), "utf8");
const stylesCss = await readFile(new URL("../app/styles.css", import.meta.url), "utf8");
const miniCss = await readFile(new URL("../app/mini.css", import.meta.url), "utf8");
const overlayCss = await readFile(new URL("../app/overlay.css", import.meta.url), "utf8");
const packageJson = JSON.parse(await readFile(new URL("../package.json", import.meta.url), "utf8"));
const readme = await readFile(new URL("../README.md", import.meta.url), "utf8");
const cargoToml = await readFile(new URL("../src-tauri/Cargo.toml", import.meta.url), "utf8");
const mainRs = await readFile(new URL("../src-tauri/src/main.rs", import.meta.url), "utf8");
const libRs = await readFile(new URL("../src-tauri/src/lib.rs", import.meta.url), "utf8");
const trayRs = await readFile(new URL("../src-tauri/src/tray.rs", import.meta.url), "utf8");
const tauriConfig = JSON.parse(
  await readFile(new URL("../src-tauri/tauri.conf.json", import.meta.url), "utf8"),
);
const mainWindow = tauriConfig.app.windows.find((window) => window.label === "main");

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

assert(
  appJs.includes("app_filter") && appJs.includes("대상 앱") && appJs.includes("MapleStory"),
  "The settings UI should expose an application filter for MapleStory-focused key input",
);

assert(
  miniHtml.includes("mini.js") && miniHtml.includes("AlertTimer Mini"),
  "Mini timer window should have its own HTML entry point",
);

assert(
  miniJs.includes("timer://snapshot") && miniJs.includes("next-alert"),
  "Mini timer window should subscribe to timer snapshots and render the next alert",
);

assert(
  appJs.includes("progressPercent") &&
    appJs.includes("timer-progress") &&
    stylesCss.includes(".timer-progress-fill"),
  "Main timer status should render clear running-state progress bars",
);

assert(
  miniJs.includes("progressPercent") &&
    miniJs.includes("mini-progress") &&
    miniCss.includes(".mini-progress-fill"),
  "Mini timer window should render compact timer progress bars",
);

assert(
  tauriConfig.app.windows.some((window) => window.label === "mini" && window.url === "mini.html"),
  "Tauri config should define a mini timer window",
);

assert(mainWindow, "Tauri config should define a main window");

assert(
  mainWindow.width >= 1180 && mainWindow.minWidth >= 1080,
  "Main window should open wide enough for the fixed sidebar, editor, and live monitor without horizontal scroll",
);

assert(
  stylesCss.includes("overflow: hidden"),
  "The app shell should prevent page-level scrollbars on initial launch",
);

assert(
  trayRs.includes("미니 타이머 열기"),
  "Tray menu should expose the mini timer window",
);

assert(
  trayRs.includes(".icon(") && trayRs.includes("default_window_icon"),
  "Tray icon should explicitly use the bundled app icon",
);

assert(
  libRs.includes('window.label() == "mini"'),
  "Closing the mini timer window should hide it to the tray instead of exiting",
);

assert(
  overlayCss.includes("position: fixed") &&
    overlayCss.includes("inset: 0") &&
    !overlayCss.includes("100vw"),
  "Overlay border should be fixed to all viewport edges instead of using 100vw sizing",
);

assert(
  packageJson.version === tauriConfig.version &&
    cargoToml.includes(`version = "${tauriConfig.version}"`),
  "package.json, Cargo.toml, and tauri.conf.json should share the same app version",
);

assert(
  tauriConfig.version !== "0.1.0",
  "App version should be bumped after the icon refresh so Windows treats installer.exe as an update",
);
