const fallbackSettings = {
  profiles: [
    {
      id: "janus",
      name: "야누스",
      key: "]",
      app_filter: "MapleStory",
      duration_seconds: 120,
      warning_before_seconds: 5,
      color: "#ff3344",
      cycle_key_count: 3,
      enabled: true,
    },
  ],
  overlay: {
    border_thickness_px: 8,
  },
};

const tauri = window.__TAURI__;
const invoke = tauri?.core?.invoke ?? mockInvoke;
const listen = tauri?.event?.listen ?? mockListen;

const profileList = document.querySelector("#profile-list");
const editorPanel = document.querySelector("#editor-panel");
const timerStack = document.querySelector("#timer-stack");
const overlaySettings = document.querySelector("#overlay-settings");
const editorTitle = document.querySelector("#editor-title");
const profileCount = document.querySelector("#profile-count");
const saveState = document.querySelector("#save-state");
const privilegeState = document.querySelector("#privilege-state");
const restartAdmin = document.querySelector("#restart-admin");

let settings = structuredClone(fallbackSettings);
let snapshots = [];
let selectedId = "janus";
let dirty = false;
let loadWarnings = [];
let privilegeStatus = {
  is_elevated: false,
  can_relaunch_as_admin: false,
};

document.querySelector("#add-profile").addEventListener("click", addProfile);
document.querySelector("#save-settings").addEventListener("click", saveSettings);
restartAdmin.addEventListener("click", relaunchAsAdmin);

bootstrap();

async function bootstrap() {
  settings = normalizeSettings(await invokeOrFallback("get_settings", fallbackSettings));
  selectedId = settings.profiles[0]?.id ?? null;
  snapshots = snapshotsForSettings(settings);
  render();

  snapshots = await invokeOrFallback("get_timer_snapshots", snapshotsForSettings(settings));
  privilegeStatus = await invokeOrFallback("get_privilege_status", privilegeStatus);
  render();

  try {
    await listen("timer://snapshot", (event) => {
      snapshots = Array.isArray(event.payload) ? event.payload : snapshotsForSettings(settings);
      renderStatusOnly();
    });
  } catch (error) {
    recordLoadWarning("타이머 이벤트 연결 실패", error);
    render();
  }
}

function render() {
  renderProfiles();
  renderEditor();
  renderStatusOnly();
  renderOverlaySettings();
  renderPrivilegeStatus();
  profileCount.textContent = `${settings.profiles.length} profiles`;
  saveState.textContent = loadWarnings.length > 0 ? "로드 경고" : dirty ? "수정됨" : "저장됨";
}

function renderProfiles() {
  profileList.innerHTML = "";
  for (const profile of settings.profiles) {
    const snapshot = snapshotFor(profile.id);
    const phase = snapshot?.phase ?? "waiting";
    const item = document.createElement("button");
    item.type = "button";
    item.className = `profile-item ${profile.id === selectedId ? "active" : ""} ${phaseClass(phase)}`;
    item.style.setProperty("--profile-color", profile.color);
    item.style.setProperty("--timer-progress", `${progressPercent(snapshot)}%`);
    item.innerHTML = `
      <span class="profile-color"></span>
      <span class="profile-main">
        <span class="profile-name">${escapeHtml(profile.name)}</span>
        <span class="profile-meta">
          <span>${escapeHtml(profile.key || "-")}</span>
          <span>${escapeHtml(profile.app_filter || "모든 앱")}</span>
          <span>${profile.duration_seconds}s</span>
          <span>${profile.cycle_key_count}회/사이클</span>
        </span>
        <span class="profile-progress" aria-hidden="true"><span class="profile-progress-fill"></span></span>
      </span>
      <span class="phase-badge">${phaseLabel(phase)}</span>
    `;
    item.addEventListener("click", () => {
      selectedId = profile.id;
      render();
    });
    profileList.appendChild(item);
  }
}

function renderEditor() {
  const profile = selectedProfile();
  if (!profile) {
    editorPanel.innerHTML = `<div class="empty">프로필 없음</div>`;
    editorTitle.textContent = "프로필";
    return;
  }

  editorTitle.textContent = profile.name || "프로필";
  editorPanel.innerHTML = `
    <div class="form-grid">
      ${field("이름", "name", profile.name)}
      ${field("스킬 키", "key", profile.key, "key-input", true)}
      ${field("대상 앱", "app_filter", profile.app_filter ?? "")}
      ${numberField("타이머", "duration_seconds", profile.duration_seconds, 5, 3600)}
      ${numberField("점멸 시작", "warning_before_seconds", profile.warning_before_seconds, 1, 3599)}
      ${numberField("한 사이클 키 입력 수", "cycle_key_count", profile.cycle_key_count, 1, 10)}
      <label class="field">
        <span>색상</span>
        <input data-field="color" type="color" value="${profile.color}" />
      </label>
      <label class="switch-row">
        <span>활성화</span>
        <input data-field="enabled" type="checkbox" ${profile.enabled ? "checked" : ""} />
      </label>
    </div>
    <div class="editor-actions">
      <button class="danger" id="delete-profile" type="button">삭제</button>
      <button class="primary" id="save-profile" type="button">저장</button>
    </div>
  `;

  editorPanel.querySelectorAll("[data-field]").forEach((input) => {
    input.addEventListener("input", (event) => updateProfileField(profile, event.currentTarget));
  });

  const keyInput = editorPanel.querySelector('[data-field="key"]');
  keyInput.addEventListener("keydown", (event) => {
    event.preventDefault();
    profile.key = keyFromEvent(event);
    keyInput.value = profile.key;
    markDirty();
    renderProfiles();
  });

  document.querySelector("#delete-profile").addEventListener("click", deleteSelectedProfile);
  document.querySelector("#save-profile").addEventListener("click", saveSettings);
}

function renderStatusOnly() {
  timerStack.innerHTML = "";
  for (const profile of settings.profiles) {
    const snapshot = snapshotFor(profile.id);
    const phase = snapshot?.phase ?? "waiting";
    const row = document.createElement("div");
    row.className = `timer-row ${phaseClass(phase)}`;
    row.style.setProperty("--profile-color", profile.color);
    row.style.setProperty("--timer-progress", `${progressPercent(snapshot)}%`);
    row.innerHTML = `
      <span class="timer-dot"></span>
      <span class="timer-copy">
        <span class="timer-title">${escapeHtml(profile.name)}</span>
        <span class="timer-sub">${phaseLabel(phase)}</span>
        <span class="timer-progress" aria-hidden="true"><span class="timer-progress-fill"></span></span>
      </span>
      <span class="timer-value">${timeLabel(snapshot)}</span>
    `;
    timerStack.appendChild(row);
  }
}

function renderOverlaySettings() {
  overlaySettings.innerHTML = `
    <label class="field">
      <span>테두리 두께</span>
      <input data-overlay-field="border_thickness_px" type="number" min="2" max="32" value="${settings.overlay.border_thickness_px}" />
    </label>
  `;
  overlaySettings.querySelector("input").addEventListener("input", (event) => {
    settings.overlay.border_thickness_px = numberValue(event.currentTarget, 8);
    markDirty();
  });
}

function renderPrivilegeStatus() {
  privilegeState.classList.toggle("elevated", Boolean(privilegeStatus.is_elevated));
  privilegeState.classList.toggle("not-elevated", !privilegeStatus.is_elevated);
  privilegeState.textContent = privilegeStatus.is_elevated ? "관리자 권한" : "일반 권한";
  restartAdmin.hidden = !privilegeStatus.can_relaunch_as_admin;
}

async function relaunchAsAdmin() {
  saveState.textContent = "재시작 요청";
  try {
    await invoke("relaunch_as_admin");
  } catch (error) {
    saveState.textContent = String(error);
  }
}

function addProfile() {
  const id = `profile-${Date.now()}`;
  settings.profiles.push({
    id,
    name: "새 스킬",
    key: "",
    app_filter: "",
    duration_seconds: 120,
    warning_before_seconds: 5,
    color: "#20c7a7",
    cycle_key_count: 1,
    enabled: true,
  });
  selectedId = id;
  markDirty();
  render();
}

function deleteSelectedProfile() {
  settings.profiles = settings.profiles.filter((profile) => profile.id !== selectedId);
  selectedId = settings.profiles[0]?.id ?? null;
  markDirty();
  render();
}

async function saveSettings() {
  try {
    settings = normalizeSettings(await invoke("save_settings", { settings }));
    dirty = false;
    loadWarnings = [];
    saveState.textContent = "저장됨";
    render();
  } catch (error) {
    saveState.textContent = String(error);
  }
}

async function invokeOrFallback(command, fallbackValue, payload) {
  let lastError = null;
  for (let attempt = 0; attempt < 20; attempt += 1) {
    try {
      return payload === undefined ? await invoke(command) : await invoke(command, payload);
    } catch (error) {
      lastError = error;
      await delay(100);
    }
  }

  recordLoadWarning(`${command} 실패`, lastError);
  return structuredClone(fallbackValue);
}

function recordLoadWarning(label, error) {
  const message = `${label}: ${String(error)}`;
  loadWarnings.push(message);
  console.warn(message);
}

function delay(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}

function updateProfileField(profile, input) {
  const fieldName = input.dataset.field;
  if (input.type === "checkbox") {
    profile[fieldName] = input.checked;
  } else if (input.type === "number") {
    profile[fieldName] = numberValue(input, profile[fieldName]);
  } else {
    profile[fieldName] = input.value;
  }
  markDirty();
  if (fieldName === "name" || fieldName === "key" || fieldName === "app_filter" || fieldName === "color") {
    renderProfiles();
    editorTitle.textContent = profile.name || "프로필";
  }
}

function normalizeSettings(nextSettings) {
  const normalized = structuredClone(nextSettings ?? fallbackSettings);
  if (!Array.isArray(normalized.profiles) || normalized.profiles.length === 0) {
    normalized.profiles = structuredClone(fallbackSettings.profiles);
  }
  normalized.overlay = {
    ...structuredClone(fallbackSettings.overlay),
    ...(normalized.overlay ?? {}),
  };
  for (const profile of normalized.profiles) {
    profile.app_filter ??= "";
  }
  return normalized;
}

function snapshotsForSettings(nextSettings) {
  return nextSettings.profiles.map((profile) => ({
    profile_id: profile.id,
    name: profile.name,
    color: profile.color,
    phase: "waiting",
    duration_ms: profile.duration_seconds * 1000,
    warning_before_ms: profile.warning_before_seconds * 1000,
    remaining_ms: null,
    overdue_ms: null,
  }));
}

function selectedProfile() {
  return settings.profiles.find((profile) => profile.id === selectedId);
}

function snapshotFor(profileId) {
  return snapshots.find((snapshot) => snapshot.profile_id === profileId);
}

function markDirty() {
  dirty = true;
  saveState.textContent = "수정됨";
}

function field(label, name, value, className = "", readonly = false) {
  return `
    <label class="field">
      <span>${label}</span>
      <input class="${className}" data-field="${name}" value="${escapeAttribute(value)}" ${readonly ? "readonly" : ""} />
    </label>
  `;
}

function numberField(label, name, value, min, max) {
  return `
    <label class="field">
      <span>${label}</span>
      <input data-field="${name}" type="number" min="${min}" max="${max}" value="${value}" />
    </label>
  `;
}

function numberValue(input, fallback) {
  const value = Number(input.value);
  return Number.isFinite(value) ? value : fallback;
}

function phaseLabel(phase) {
  return {
    waiting: "대기",
    running: "진행",
    warning: "점멸",
    expired: "만료",
  }[phase ?? "waiting"];
}

function phaseClass(phase) {
  return ["waiting", "running", "warning", "expired"].includes(phase) ? phase : "waiting";
}

function progressPercent(snapshot) {
  if (!snapshot || snapshot.phase === "waiting") return 0;
  if (snapshot.phase === "expired" || snapshot.overdue_ms != null) return 100;

  const duration = Number(snapshot.duration_ms ?? 0);
  const remaining = Number(snapshot.remaining_ms ?? duration);
  if (!Number.isFinite(duration) || duration <= 0 || !Number.isFinite(remaining)) return 0;

  const elapsed = duration - Math.max(0, remaining);
  return Math.min(100, Math.max(0, (elapsed / duration) * 100));
}

function timeLabel(snapshot) {
  if (!snapshot) return "--";
  if (snapshot.remaining_ms != null) return formatMs(snapshot.remaining_ms);
  if (snapshot.overdue_ms != null) return `+${formatMs(snapshot.overdue_ms)}`;
  return "--";
}

function formatMs(ms) {
  const total = Math.ceil(ms / 1000);
  const minutes = Math.floor(total / 60);
  const seconds = total % 60;
  return `${minutes}:${String(seconds).padStart(2, "0")}`;
}

function keyFromEvent(event) {
  if (event.key.length === 1) return event.key.toUpperCase();
  const aliases = {
    " ": "Space",
    Esc: "Escape",
    PageDown: "PageDown",
    PageUp: "PageUp",
  };
  return aliases[event.key] ?? event.key;
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

async function mockInvoke(command, payload) {
  if (command === "get_settings") return structuredClone(fallbackSettings);
  if (command === "save_settings") {
    Object.assign(fallbackSettings, structuredClone(payload.settings));
    return structuredClone(fallbackSettings);
  }
  if (command === "get_timer_snapshots") {
    return fallbackSettings.profiles.map((profile) => ({
      profile_id: profile.id,
      name: profile.name,
      color: profile.color,
      phase: "waiting",
      duration_ms: profile.duration_seconds * 1000,
      warning_before_ms: profile.warning_before_seconds * 1000,
      remaining_ms: null,
      overdue_ms: null,
    }));
  }
  if (command === "get_privilege_status") {
    return {
      is_elevated: false,
      can_relaunch_as_admin: true,
    };
  }
  if (command === "relaunch_as_admin") return null;
  return null;
}

async function mockListen() {
  return () => {};
}
