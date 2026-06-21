const fallbackSettings = {
  profiles: [
    {
      id: "janus",
      name: "야누스",
      key: "]",
      duration_seconds: 120,
      warning_before_seconds: 5,
      color: "#ff3344",
      skill_press_count: 3,
      repeat_ignore_window_seconds: 10,
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

let settings = structuredClone(fallbackSettings);
let snapshots = [];
let selectedId = "janus";
let dirty = false;

document.querySelector("#add-profile").addEventListener("click", addProfile);
document.querySelector("#save-settings").addEventListener("click", saveSettings);

bootstrap();

async function bootstrap() {
  settings = await invoke("get_settings");
  selectedId = settings.profiles[0]?.id ?? null;
  snapshots = await invoke("get_timer_snapshots");
  render();
  await listen("timer://snapshot", (event) => {
    snapshots = event.payload;
    renderStatusOnly();
  });
}

function render() {
  renderProfiles();
  renderEditor();
  renderStatusOnly();
  renderOverlaySettings();
  profileCount.textContent = `${settings.profiles.length} profiles`;
  saveState.textContent = dirty ? "수정됨" : "저장됨";
}

function renderProfiles() {
  profileList.innerHTML = "";
  for (const profile of settings.profiles) {
    const snapshot = snapshotFor(profile.id);
    const item = document.createElement("button");
    item.type = "button";
    item.className = `profile-item ${profile.id === selectedId ? "active" : ""}`;
    item.style.setProperty("--profile-color", profile.color);
    item.innerHTML = `
      <span class="profile-color"></span>
      <span>
        <span class="profile-name">${escapeHtml(profile.name)}</span>
        <span class="profile-meta">
          <span>${escapeHtml(profile.key || "-")}</span>
          <span>${profile.duration_seconds}s</span>
          <span>${profile.skill_press_count}회</span>
        </span>
      </span>
      <span class="phase-badge">${phaseLabel(snapshot?.phase)}</span>
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
      ${numberField("타이머", "duration_seconds", profile.duration_seconds, 5, 3600)}
      ${numberField("점멸 시작", "warning_before_seconds", profile.warning_before_seconds, 1, 3599)}
      ${numberField("스킬 연타 횟수", "skill_press_count", profile.skill_press_count, 1, 10)}
      ${numberField("연타 무시 시간", "repeat_ignore_window_seconds", profile.repeat_ignore_window_seconds, 0, 60)}
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
    row.className = "timer-row";
    row.innerHTML = `
      <span class="timer-dot" style="--profile-color:${profile.color}"></span>
      <span>
        <span class="timer-title">${escapeHtml(profile.name)}</span>
        <span class="timer-sub">${phaseLabel(phase)}</span>
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

function addProfile() {
  const id = `profile-${Date.now()}`;
  settings.profiles.push({
    id,
    name: "새 스킬",
    key: "",
    duration_seconds: 120,
    warning_before_seconds: 5,
    color: "#20c7a7",
    skill_press_count: 1,
    repeat_ignore_window_seconds: 0,
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
    settings = await invoke("save_settings", { settings });
    dirty = false;
    saveState.textContent = "저장됨";
    render();
  } catch (error) {
    saveState.textContent = String(error);
  }
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
  if (fieldName === "name" || fieldName === "key" || fieldName === "color") {
    renderProfiles();
    editorTitle.textContent = profile.name || "프로필";
  }
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
      remaining_ms: null,
      overdue_ms: null,
    }));
  }
  return null;
}

async function mockListen() {
  return () => {};
}
