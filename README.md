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
- 야누스 기본 대상 앱: `MapleStory`
- 첫 키 입력 시 타이머 시작/리셋
- 다중 입력 스킬을 위한 사이클 입력 수 설정
- 점멸 시작 시간과 실제 타이머 시간 분리 설정
- 프로필별 색상, 키, 대상 앱, 사이클 입력 수, 활성화 여부 설정
- 전체 화면/창모드 전체 화면 위에 표시되는 클릭 통과 오버레이
- 여러 타이머가 동시에 경고 상태일 때 색상 순환 표시
- 시스템 트레이 상주
- `X`로 창을 닫아도 트레이에 계속 상주
- 다음 알림까지 남은 시간을 보는 미니 타이머 창
- NSIS/MSI Windows 설치 파일 생성

### 현재 야누스 기본 동작

기본값은 다음과 같습니다.

| 항목 | 값 |
| --- | --- |
| 스킬 이름 | 야누스 |
| 키 | `]` |
| 대상 앱 | `MapleStory` |
| 타이머 | 120초 |
| 점멸 시작 | 만료 5초 전 |
| 색상 | 빨간색 |
| 한 사이클 키 입력 수 | 3 |
| 미완성 사이클 초기화 | 첫 입력 후 30초 |

현재 구현은 “10초 안에 3번 눌러야 리셋” 방식도 아니고, 시간 기반 무시 방식도 아닙니다.

동작은 다음과 같습니다.

1. `]`를 처음 누르면 야누스 타이머가 즉시 리셋됩니다.
2. 두 번째와 세 번째 `]` 입력은 같은 설치 사이클에 포함되므로 타이머를 다시 리셋하지 않습니다.
3. 네 번째 `]` 입력은 다음 사이클의 첫 입력이므로 타이머를 다시 리셋합니다.
4. 첫 입력 후 30초 안에 사이클이 끝나지 않으면 카운터가 0으로 초기화되고, 다음 `]` 입력은 새 사이클의 첫 입력으로 처리됩니다.
5. 점멸 또는 만료 상태에서는 다음 `]` 입력이 남은 사이클 입력 수와 관계없이 즉시 타이머를 리셋합니다.

즉, `한 사이클 키 입력 수` 값 `3`은 “같은 키 3번까지를 한 사이클로 친다”는 의미입니다. 별도의 무시 시간 입력 필드는 없습니다.

### 설치 방법

가장 쉬운 설치 파일은 프로젝트 루트의 `installer.exe`입니다.

```powershell
.\installer.exe
```

업데이트 설치 중 기존 AlertTimer가 실행 중이면 설치 프로그램이 종료 확인을 요청합니다. 설치 위치는 사용자 폴더(`%LOCALAPPDATA%\AlertTimer`)를 유지하지만, MapleStory 입력 감지를 위해 AlertTimer를 관리자 권한으로 실행한 상태에서도 업데이트할 수 있도록 설치 프로그램은 Windows UAC 승인을 요청할 수 있습니다.

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

### MapleStory 키 입력이 감지되지 않을 때

MapleStory가 관리자 권한 또는 보호된 프로세스로 실행되면 일반 권한 AlertTimer가 전역 키 입력을 받지 못할 수 있습니다. 이 경우 앱 상단의 `관리자 권한 재시작` 버튼을 눌러 AlertTimer만 관리자 권한으로 다시 실행하세요. Windows UAC 창은 사용자가 직접 승인해야 합니다.

### 사용 방법

1. `installer.exe`로 설치하거나 `AlertTimer.exe`를 실행합니다.
2. 앱에서 야누스 프로필을 확인합니다.
3. 상단 상태가 `일반 권한`이고 MapleStory 키 입력이 감지되지 않으면 `관리자 권한 재시작`을 누릅니다.
4. 필요하면 키, 타이머 시간, 점멸 시작 시간, 색상, 테두리 두께를 조정합니다.
5. `저장`을 누릅니다.
6. 게임 중 설정한 키를 누르면 타이머가 갱신됩니다.
7. 시간이 가까워지면 오버레이 테두리가 점멸합니다.
8. 트레이 메뉴의 `미니 타이머 열기`로 다음 알림까지 남은 시간을 확인할 수 있습니다.

### 설정 설명

| 설정 | 설명 |
| --- | --- |
| 이름 | 프로필 표시 이름 |
| 스킬 키 | 타이머를 리셋할 키 |
| 대상 앱 | 입력을 인정할 foreground 앱 이름. 비우면 모든 앱에서 입력을 인정 |
| 타이머 | 실제 스킬 재설치/재사용 주기 |
| 점멸 시작 | 만료 몇 초 전부터 점멸할지 |
| 한 사이클 키 입력 수 | 같은 키 몇 번까지를 한 사이클로 묶을지 |
| 색상 | 해당 프로필의 오버레이 색상 |
| 활성화 | 프로필 사용 여부 |
| 테두리 두께 | 모니터 테두리 점멸 두께 |

### 주의 사항

- 이 앱은 키 입력을 자동으로 보내지 않습니다.
- 매크로, 자동 사냥, 자동 입력 기능이 아닙니다.
- 게임 화면 위에 클릭 통과 오버레이를 표시하고, 사용자가 누른 키를 기준으로 타이머를 갱신합니다.
- 기존에 저장된 설정이 있으면 새 기본값이 바로 반영되지 않을 수 있습니다. 그 경우 UI에서 야누스 키를 `]`로 변경한 뒤 저장하세요.
- 메이플스토리가 관리자 권한으로 실행 중이면 AlertTimer도 관리자 권한으로 실행해야 키 감지가 일관적일 수 있습니다.
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
- Default Janus target app: `MapleStory`
- First key press starts or resets the timer
- Count-based cycle grouping for multi-press skills
- Separate timer duration and warning start time
- Per-profile key, target app, color, cycle key count, and enabled state
- Click-through fullscreen overlay border
- Alternating colors when multiple timers alert at the same time
- System tray app
- Windows hide to tray when closed with `X`
- Mini timer window for checking the next alert countdown
- Windows NSIS/MSI installer output

### Default Janus Behavior

Default values:

| Setting | Value |
| --- | --- |
| Skill name | Janus |
| Key | `]` |
| Target app | `MapleStory` |
| Timer | 120 seconds |
| Warning start | 5 seconds before expiration |
| Color | Red |
| Cycle key count | 3 |
| Incomplete cycle reset | 30 seconds after the first keydown |

The app does not require pressing the key 3 times within 10 seconds, and it does not use a time-based ignore window.

Current behavior:

1. The first `]` press immediately resets the Janus timer.
2. The second and third `]` presses belong to the same skill cycle and do not reset the timer again.
3. The fourth `]` press starts the next cycle and resets the timer again.
4. If the cycle is not completed within 30 seconds after the first keydown, the counter resets to zero and the next `]` press becomes the first press of a new cycle.
5. While the timer is warning or expired, the next `]` press resets the timer immediately regardless of remaining cycle count.

In other words, `cycle_key_count = 3` means “treat up to three matching keydowns as one skill cycle.” There is no separate ignore-time input field.

### Installation

The easiest installer is available at the project root:

```powershell
.\installer.exe
```

If AlertTimer is already running during an update, the installer asks to close it before copying files. The install location stays in the current-user folder (`%LOCALAPPDATA%\AlertTimer`), but the installer may request Windows UAC approval so it can update AlertTimer even when the existing app is running with administrator privileges for MapleStory input detection.

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

### When MapleStory Key Input Is Not Detected

If MapleStory runs as administrator or as a protected process, a normal AlertTimer process may not receive global key input. In that case, click `관리자 권한 재시작` in the app top bar to restart only AlertTimer with administrator privileges. The Windows UAC prompt must be approved by the user.

### Usage

1. Install with `installer.exe` or run `AlertTimer.exe`.
2. Open the app and check the Janus profile.
3. If the top bar shows `일반 권한` and MapleStory key input is not detected, click `관리자 권한 재시작`.
4. Adjust key, timer duration, warning start, color, and border thickness if needed.
5. Click `저장` to save.
6. While playing, press the configured key to reset the timer.
7. When the timer approaches expiration, the overlay border flashes.
8. Use `미니 타이머 열기` from the tray menu to check the next alert countdown.

### Settings

| Setting | Description |
| --- | --- |
| Name | Profile display name |
| Skill key | Key that resets the timer |
| Target app | Foreground app name accepted for input. Leave blank to accept all apps |
| Timer | Actual skill recast/reinstall interval |
| Warning start | How many seconds before expiration the border starts flashing |
| Cycle key count | Number of matching keydowns treated as one skill cycle |
| Color | Overlay color for the profile |
| Enabled | Whether the profile is active |
| Border thickness | Flashing monitor border thickness |

### Safety Notes

- AlertTimer does not send key input.
- It is not a macro, bot, auto-hunt, or auto-input tool.
- It only watches user key presses and displays a timer overlay.
- If you already saved settings before the default key changed, the saved value may still be `A`. Change the Janus key to `]` in the UI and save.
- If MapleStory runs as administrator, run AlertTimer as administrator as well for consistent key detection.
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
