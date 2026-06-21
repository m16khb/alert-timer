const miniTauri = window.__TAURI__;
const miniInvoke = miniTauri?.core?.invoke ?? mockInvoke;
const miniListen = miniTauri?.event?.listen ?? mockListen;

const phaseEl = document.querySelector("#mini-phase");
const nextAlertEl = document.querySelector("#next-alert");
const nextColorEl = document.querySelector("#next-color");
const nextLabelEl = document.querySelector("#next-label");
const nextValueEl = document.querySelector("#next-value");
const nextDetailEl = document.querySelector("#next-detail");
const listEl = document.querySelector("#mini-list");

let snapshots = [];

bootstrap();

async function bootstrap() {
  snapshots = await miniInvoke("get_timer_snapshots");
  render();
  await miniListen("timer://snapshot", (event) => {
    snapshots = event.payload;
    render();
  });
}

function render() {
  const next = nextAlert(snapshots);
  renderNext(next);
  renderRows(snapshots);
}

function nextAlert(items) {
  const candidates = items
    .map((snapshot) => ({
      snapshot,
      alertMs: alertMs(snapshot),
    }))
    .filter((item) => item.alertMs != null)
    .sort((a, b) => a.alertMs - b.alertMs);

  return candidates[0] ?? null;
}

function alertMs(snapshot) {
  if (snapshot.phase === "expired" || snapshot.phase === "warning") return 0;
  if (snapshot.phase !== "running" || snapshot.remaining_ms == null) return null;
  return Math.max(0, snapshot.remaining_ms - (snapshot.warning_before_ms ?? 0));
}

function renderNext(next) {
  if (!next) {
    phaseEl.textContent = "대기";
    nextAlertEl.style.setProperty("--next-color", "#20c7a7");
    nextLabelEl.textContent = "다음 알림";
    nextValueEl.textContent = "--";
    nextDetailEl.textContent = "스킬 키를 누르면 시작됩니다.";
    return;
  }

  const { snapshot, alertMs: ms } = next;
  phaseEl.textContent = phaseLabel(snapshot.phase);
  nextAlertEl.style.setProperty("--next-color", snapshot.color);
  nextColorEl.style.setProperty("--next-color", snapshot.color);
  nextLabelEl.textContent = snapshot.phase === "expired" ? "만료됨" : "다음 알림";
  nextValueEl.textContent = ms === 0 ? "지금" : formatDuration(ms);
  nextDetailEl.textContent = `${snapshot.name} ${detailLabel(snapshot)}`;
}

function renderRows(items) {
  listEl.innerHTML = "";
  for (const snapshot of items.slice(0, 4)) {
    const row = document.createElement("div");
    row.className = "mini-row";
    row.innerHTML = `
      <span class="mini-dot" style="--row-color:${escapeAttribute(snapshot.color)}"></span>
      <span class="mini-name">${escapeHtml(snapshot.name)}</span>
      <span class="mini-time">${rowTime(snapshot)}</span>
    `;
    listEl.appendChild(row);
  }
}

function detailLabel(snapshot) {
  if (snapshot.phase === "expired") return `+${formatDuration(snapshot.overdue_ms ?? 0)}`;
  if (snapshot.phase === "warning") return "점멸 중";
  return "점멸까지";
}

function rowTime(snapshot) {
  if (snapshot.phase === "waiting") return "대기";
  if (snapshot.phase === "expired") return `+${formatDuration(snapshot.overdue_ms ?? 0)}`;
  if (snapshot.remaining_ms != null) return formatDuration(snapshot.remaining_ms);
  return "--";
}

function phaseLabel(phase) {
  return {
    waiting: "대기",
    running: "진행",
    warning: "점멸",
    expired: "만료",
  }[phase ?? "waiting"];
}

function formatDuration(ms) {
  const total = Math.max(0, Math.ceil(ms / 1000));
  if (total < 60) return `${total}초`;
  const minutes = Math.floor(total / 60);
  const seconds = total % 60;
  return `${minutes}:${String(seconds).padStart(2, "0")}`;
}

function escapeHtml(value) {
  return String(value)
    .replaceAll("&", "&amp;")
    .replaceAll("<", "&lt;")
    .replaceAll(">", "&gt;")
    .replaceAll('"', "&quot;");
}

function escapeAttribute(value) {
  return escapeHtml(value).replaceAll("'", "&#39;");
}

async function mockInvoke(command) {
  if (command !== "get_timer_snapshots") return null;
  return [
    {
      profile_id: "janus",
      name: "야누스",
      color: "#ff3344",
      phase: "running",
      warning_before_ms: 5_000,
      remaining_ms: 92_000,
      overdue_ms: null,
    },
  ];
}

async function mockListen() {
  return () => {};
}
