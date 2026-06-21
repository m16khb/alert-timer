# AlertTimer

![AlertTimer preview](docs/assets/alerttimer-hero.webp)

**AlertTimer** is a lightweight Windows tray app for MapleStory skill recast reminders.
It watches your configured skill key, runs multiple timers in parallel, and flashes a click-through monitor border when a timer reaches its warning or expired state.

`by 엘리시움 사과팬케이크`

---

## 한국어

### 개요

AlertTimer는 메이플스토리처럼 화면 집중도가 높은 게임을 할 때, 설치형/재설치형 스킬의 재사용 타이밍을 놓치지 않도록 돕는 Windows용 타이머 오버레이 앱입니다.

앱은 게임 입력을 대신 누르지 않습니다. 키 입력을 감지해서 타이머만 갱신하고, 시간이 되면 모니터 테두리를 지정한 색상으로 점멸시켜 알려줍니다.

### 주요 기능

- 여러 스킬 타이머 프로필을 동시에 실행
- 기본 야누스 프로필 포함
- 야누스 기본 키: `]`
- 첫 키 입력 시 타이머 시작/리셋
- 연타형 스킬을 위한 후속 입력 무시 시간 지원
- 점멸 시작 시간과 실제 타이머 시간 분리 설정
- 프로필별 색상, 키, 연타 횟수, 무시 시간, 활성화 여부 설정
- 전체 화면/창모드 전체 화면 위에 표시되는 클릭 통과 오버레이
- 여러 타이머가 동시에 경고 상태일 때 색상 순환 표시
- 시스템 트레이 상주
- NSIS/MSI Windows 설치 파일 생성

### 현재 야누스 기본 동작

기본값은 다음과 같습니다.

| 항목 | 값 |
| --- | --- |
| 스킬 이름 | 야누스 |
| 키 | `]` |
| 타이머 | 120초 |
| 점멸 시작 | 만료 5초 전 |
| 색상 | 빨간색 |
| 스킬 연타 횟수 | 3 |
| 연타 무시 시간 | 10초 |

현재 구현은 “10초 안에 3번 눌러야 리셋” 방식이 아닙니다.

동작은 다음과 같습니다.

1. `]`를 처음 누르면 야누스 타이머가 즉시 리셋됩니다.
2. 이후 같은 키 입력은 설정된 `연타 무시 시간` 동안 무시됩니다.
3. 기본값에서는 첫 입력 후 10초 동안 후속 `]` 입력이 무시됩니다.

즉, `스킬 연타 횟수` 값은 현재 “연타형 스킬로 보고 무시 윈도우를 켜는 설정”에 가깝게 동작합니다.

### 설치 방법

가장 쉬운 설치 파일은 프로젝트 루트의 `installer.exe`입니다.

```powershell
.\installer.exe
```

또는 Tauri가 생성한 원본 설치 파일을 실행해도 됩니다.

```powershell
.\src-tauri\target\release\bundle\nsis\AlertTimer_0.1.0_x64-setup.exe
```

MSI가 필요한 환경에서는 아래 파일을 사용할 수 있습니다.

```powershell
.\src-tauri\target\release\bundle\msi\AlertTimer_0.1.0_x64_en-US.msi
```

설치 없이 바로 실행하려면 릴리스 실행 파일을 실행합니다.

```powershell
.\src-tauri\target\release\AlertTimer.exe
```

Windows SmartScreen이 표시되면 `추가 정보`를 누른 뒤 `실행`을 선택하면 됩니다.

### 사용 방법

1. `installer.exe`로 설치하거나 `AlertTimer.exe`를 실행합니다.
2. 앱에서 야누스 프로필을 확인합니다.
3. 필요하면 키, 타이머 시간, 점멸 시작 시간, 색상, 테두리 두께를 조정합니다.
4. `저장`을 누릅니다.
5. 게임 중 설정한 키를 누르면 타이머가 갱신됩니다.
6. 시간이 가까워지면 오버레이 테두리가 점멸합니다.

### 설정 설명

| 설정 | 설명 |
| --- | --- |
| 이름 | 프로필 표시 이름 |
| 스킬 키 | 타이머를 리셋할 키 |
| 타이머 | 실제 스킬 재설치/재사용 주기 |
| 점멸 시작 | 만료 몇 초 전부터 점멸할지 |
| 스킬 연타 횟수 | 연타형 스킬 여부를 판단하는 값 |
| 연타 무시 시간 | 첫 입력 후 후속 입력을 무시할 시간 |
| 색상 | 해당 프로필의 오버레이 색상 |
| 활성화 | 프로필 사용 여부 |
| 테두리 두께 | 모니터 테두리 점멸 두께 |

### 주의 사항

- 이 앱은 키 입력을 자동으로 보내지 않습니다.
- 매크로, 자동 사냥, 자동 입력 기능이 아닙니다.
- 게임 화면 위에 클릭 통과 오버레이를 표시하고, 사용자가 누른 키를 기준으로 타이머를 갱신합니다.
- 기존에 저장된 설정이 있으면 새 기본값이 바로 반영되지 않을 수 있습니다. 그 경우 UI에서 야누스 키를 `]`로 변경한 뒤 저장하세요.
- 전역 키 감지는 Windows에서만 지원됩니다.

### 개발 환경

필요한 도구:

- Windows
- Node.js 22+
- pnpm 10+
- Rust stable
- Microsoft Visual Studio Build Tools with MSVC
- WebView2 Runtime

의존성 설치:

```powershell
pnpm install
```

개발 모드 실행:

```powershell
pnpm dev
```

전체 테스트:

```powershell
pnpm test
```

Rust 코어 커버리지 확인:

```powershell
pnpm coverage:core
```

커버리지는 Rust 코어 라인 기준 80% 미만이면 실패합니다.

릴리스 설치 파일 빌드:

```powershell
pnpm build
```

### 테스트 구성

| 명령 | 설명 |
| --- | --- |
| `pnpm test:assets` | 앱 이미지 자산 정책 검사 |
| `pnpm test:branding` | 브랜드 크레딧 문구 검사 |
| `pnpm test:core` | 순수 Rust 타이머 엔진 테스트 |
| `pnpm test:tauri` | Tauri 앱 모델/키 매핑 테스트 |
| `pnpm coverage:core` | Rust 코어 커버리지 측정 |

### 프로젝트 구조

```text
app/                         Static frontend UI
crates/alert-timer-core/     Pure Rust timer engine
docs/assets/                 README and documentation images
scripts/                     Test and utility scripts
src-tauri/                   Tauri Windows shell and native integration
installer.exe                Easy-to-run NSIS installer copy
```

### 이미지 자산 정책

README와 앱 UI에서 사용하는 일반 래스터 이미지는 용량을 줄이기 위해 WebP를 사용합니다.

Tauri/Windows 앱 아이콘은 플랫폼 요구사항 때문에 `src-tauri/icons/`의 PNG/ICO 파일을 유지합니다.

---

## English

### Overview

AlertTimer is a lightweight Windows timer overlay for MapleStory skill recast reminders.
It is designed for skills that need to be reinstalled or refreshed on a fixed timer while you are focused on gameplay.

The app does not press keys for you. It only listens for your configured key press, resets the matching timer, and flashes a click-through monitor border when the timer reaches the warning or expired state.

### Features

- Multiple skill timer profiles running in parallel
- Built-in Janus profile
- Default Janus key: `]`
- First key press starts or resets the timer
- Repeat-input ignore window for multi-press skills
- Separate timer duration and warning start time
- Per-profile key, color, repeat count, ignore window, and enabled state
- Click-through fullscreen overlay border
- Alternating colors when multiple timers alert at the same time
- System tray app
- Windows NSIS/MSI installer output

### Default Janus Behavior

Default values:

| Setting | Value |
| --- | --- |
| Skill name | Janus |
| Key | `]` |
| Timer | 120 seconds |
| Warning start | 5 seconds before expiration |
| Color | Red |
| Skill press count | 3 |
| Repeat ignore window | 10 seconds |

The current implementation does not require pressing the key 3 times within 10 seconds.

Current behavior:

1. The first `]` press immediately resets the Janus timer.
2. Follow-up presses of the same key are ignored during the configured repeat ignore window.
3. With the default settings, follow-up `]` presses are ignored for 10 seconds after the first press.

In other words, `skill_press_count` currently behaves more like a switch that enables the repeat-ignore behavior for multi-press skills.

### Installation

The easiest installer is available at the project root:

```powershell
.\installer.exe
```

You can also run the original NSIS setup file:

```powershell
.\src-tauri\target\release\bundle\nsis\AlertTimer_0.1.0_x64-setup.exe
```

For MSI-based deployment, use:

```powershell
.\src-tauri\target\release\bundle\msi\AlertTimer_0.1.0_x64_en-US.msi
```

To run without installing:

```powershell
.\src-tauri\target\release\AlertTimer.exe
```

If Windows SmartScreen appears, choose `More info`, then `Run anyway`.

### Usage

1. Install with `installer.exe` or run `AlertTimer.exe`.
2. Open the app and check the Janus profile.
3. Adjust key, timer duration, warning start, color, and border thickness if needed.
4. Click `저장` to save.
5. While playing, press the configured key to reset the timer.
6. When the timer approaches expiration, the overlay border flashes.

### Settings

| Setting | Description |
| --- | --- |
| Name | Profile display name |
| Skill key | Key that resets the timer |
| Timer | Actual skill recast/reinstall interval |
| Warning start | How many seconds before expiration the border starts flashing |
| Skill press count | Marks the skill as a multi-press skill |
| Repeat ignore window | Time window for ignoring follow-up presses after the first key press |
| Color | Overlay color for the profile |
| Enabled | Whether the profile is active |
| Border thickness | Flashing monitor border thickness |

### Safety Notes

- AlertTimer does not send key input.
- It is not a macro, bot, auto-hunt, or auto-input tool.
- It only watches user key presses and displays a timer overlay.
- If you already saved settings before the default key changed, the saved value may still be `A`. Change the Janus key to `]` in the UI and save.
- Global key detection is supported on Windows only.

### Development

Requirements:

- Windows
- Node.js 22+
- pnpm 10+
- Rust stable
- Microsoft Visual Studio Build Tools with MSVC
- WebView2 Runtime

Install dependencies:

```powershell
pnpm install
```

Run in development mode:

```powershell
pnpm dev
```

Run all tests:

```powershell
pnpm test
```

Check Rust core coverage:

```powershell
pnpm coverage:core
```

The coverage script fails when Rust core line coverage drops below 80%.

Build release installers:

```powershell
pnpm build
```

### Test Commands

| Command | Description |
| --- | --- |
| `pnpm test:assets` | Checks app image asset policy |
| `pnpm test:branding` | Checks brand attribution text |
| `pnpm test:core` | Runs pure Rust timer engine tests |
| `pnpm test:tauri` | Runs Tauri model and key mapping tests |
| `pnpm coverage:core` | Measures Rust core coverage |

### Project Layout

```text
app/                         Static frontend UI
crates/alert-timer-core/     Pure Rust timer engine
docs/assets/                 README and documentation images
scripts/                     Test and utility scripts
src-tauri/                   Tauri Windows shell and native integration
installer.exe                Easy-to-run NSIS installer copy
```

### Image Asset Policy

Regular raster images used by the README or app UI should use WebP to keep the bundle small.

Tauri/Windows app icons remain PNG/ICO under `src-tauri/icons/` because the Windows installer and app icon pipeline require platform icon formats.
