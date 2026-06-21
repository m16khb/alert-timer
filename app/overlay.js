const tauriOverlay = window.__TAURI__;
const listenOverlay = tauriOverlay?.event?.listen;

if (listenOverlay) {
  listenOverlay("overlay://frame", (event) => applyFrame(event.payload));
}

function applyFrame(payload) {
  document.documentElement.style.setProperty("--alert-color", payload.color || "transparent");
  document.documentElement.style.setProperty(
    "--border-size",
    `${Math.max(0, payload.border_thickness_px ?? 0)}px`,
  );
  document.body.classList.toggle("is-active", Boolean(payload.active && payload.visible));
  document.body.classList.toggle("is-expired", payload.intensity === "expired");
}
