# Codex Tray

[English](README.md) · [Español](README.es.md) · [Français](README.fr.md) · [Português](README.pt.md) · [Deutsch](README.de.md) · [Italiano](README.it.md) · [Русский](README.ru.md) · [简体中文](README.zh-CN.md) · [हिन्दी](README.hi.md) · [العربية](README.ar.md) · [日本語](README.ja.md) · 한국어

---

Codex의 남은 사용량을 확인하는 네이티브 Windows 시스템 트레이 표시기입니다.

## 개요

Codex Tray를 사용하면 Codex 앱이나 CLI를 화면 앞에 띄워 두지 않고도 현재 Codex 사용량을 확인할 수 있습니다. 작은 Windows 시스템 트레이 앱으로 실행되며 현재 사용자의 인증된 Codex CLI 세션을 재사용하고, 아이콘 위에 포인터를 올리면 간결한 사용량 패널을 표시합니다.

앱은 로컬에 설치된 `codex app-server`와만 통신합니다. API 키를 요청하지 않으며 `~/.codex/auth.json`을 직접 읽거나 복사하지 않습니다.

## 기능

- `account/rateLimits/updated` 서버 알림을 통한 실시간 사용량 업데이트와 오래된 데이터에 대한 호버 시 최신 상태 확인.
- 안정적인 `레이블: 값` 행으로 구성된 DPI 대응 패널.
- Windows 밝은/어두운 테마, 강조 색상 및 투명도 지원.
- 사용량 단계와 오류 상태를 위한 픽셀 정렬 트레이 아이콘.
- 아이콘 위에 포인터를 올리면 해당 아이콘이 있는 모니터에 패널을 표시하고 벗어나면 숨김.
- 12개 언어 번역을 내장하며 기본값으로 시스템 언어를 선택.
- 요청 시 새로 고침, 실행 파일 폴더 열기, Windows 시작 설정 및 명확한 종료 동작을 제공하는 컨텍스트 메뉴.
- 실행 파일 옆에 저장되는 포터블 언어 및 자동 시작 설정.
- 트레이 아이콘 위에 시스템 툴팁을 표시하지 않음.
- 로딩, 재연결, 인증, 구독, CLI 없음, 사용량 소진 및 app-server 오류 상태를 구분하여 표시.

## 요구 사항

- Windows 11 x86-64.
- `PATH`에서 사용할 수 있는 [Codex CLI](https://learn.chatgpt.com/docs/codex/cli).
- `codex login`으로 생성한 인증된 Codex CLI 세션.

Codex Tray는 현재 네이티브 Windows backend만 구현합니다. Linux, macOS 및 Windows ARM64용 platform backend가 구현되고 테스트되기 전에는 해당 빌드 결과물을 배포하지 않습니다.

## 설치

1. [최신 GitHub Release](https://github.com/psimonov/codex-tray/releases/latest)를 엽니다.
2. `codex-tray-<version>-windows-x86_64.exe`와 해당 `.sha256` 파일을 다운로드합니다.
3. SHA-256 체크섬을 확인합니다.
4. 실행 파일을 쓰기 가능한 영구 폴더로 옮긴 뒤 실행합니다.

PowerShell에서 체크섬을 확인하는 예시:

```powershell
Get-FileHash .\codex-tray-0.4.1-windows-x86_64.exe -Algorithm SHA256
```

설치 프로그램은 필요하지 않습니다. Release는 하나의 포터블 실행 파일로 제공되며 `codex` 명령은 외부 런타임 요구 사항으로 유지됩니다.

## 빠른 시작

```powershell
codex login
.\codex-tray-0.4.1-windows-x86_64.exe
```

앱은 숨김 상태로 시작되고 Windows 시스템 트레이에 아이콘을 추가합니다.

## 사용법

- 같은 모니터에 사용량 패널을 표시하려면 트레이 아이콘 위에 포인터를 올립니다.
- 패널을 숨기려면 아이콘에서 포인터를 벗어납니다.
- 마우스 오른쪽 버튼으로 클릭하면 패널을 숨기고 컨텍스트 메뉴를 엽니다.
- **언어** 하위 메뉴를 열고 **시스템 언어** 또는 특정 언어를 선택합니다. 변경 사항은 즉시 적용됩니다.
- **지금 새로 고침**을 선택하면 기존 app-server 연결을 통해 `account/read`와 `account/rateLimits/read`를 즉시 다시 실행합니다.
- **애플리케이션 폴더 열기**를 선택하면 실행 중인 파일이 있는 폴더를 엽니다.
- **Windows 시작 시 실행**을 전환하면 현재 실행 파일 경로를 사용자 `Run` 키에 등록하거나 제거합니다.
- **닫기**를 선택하면 Codex Tray와 app-server 자식 프로세스를 종료합니다.

사용량 업데이트는 `codex app-server`와의 단일 지속 연결을 통해 수신됩니다. Codex Tray는 연결할 때 계정과 제한을 한 번 읽고 이후의 부분 업데이트 알림을 보존하여 재귀적으로 병합하며, app-server가 예기치 않게 종료되면 다시 연결합니다. 명시적으로 새로 고치면 두 요청을 모두 반복합니다. 패널을 표시할 때 스냅샷이 30초 이상 오래되었으면 Codex Tray가 `account/rateLimits/read`로 한 번 조정하며, 백그라운드에서 주기적으로 폴링하지 않습니다.

## 구성

처음 실행할 때 Codex Tray는 실행 파일 옆에 `codex-tray.json`을 생성합니다. 이 파일에는 선택한 언어와 Windows 자동 시작 설정이 저장됩니다.

```json
{
  "language": "system",
  "start_with_windows": false
}
```

`language`에는 `system`, `en`, `es`, `fr`, `pt`, `de`, `it`, `ru`, `zh-CN`, `hi`, `ar`, `ja`, `ko`를 사용할 수 있습니다. 구성 파일이 설정의 원본입니다. 사용자의 Windows `Run` 항목은 `start_with_windows`에서 동기화되며 항상 실행 중인 파일에서 동적으로 감지한 경로를 사용합니다. 구성을 처음 생성할 때 기존 자동 시작 항목을 가져옵니다.

## 소스에서 빌드

저장소에는 필요한 Rust toolchain 버전이 고정되어 있습니다.

```powershell
git clone https://github.com/psimonov/codex-tray.git
Set-Location codex-tray
cargo build --release --locked
```

생성되는 실행 파일은 `target\release\codex-tray.exe`입니다.

## 테스트

```powershell
cargo fmt --all -- --check
cargo test --locked
cargo clippy --all-targets --all-features --locked -- -D warnings
```

## 릴리스

버전 태그는 `vMAJOR.MINOR.PATCH` 형식을 사용합니다. GitHub Actions는 태그와 `Cargo.toml`의 버전이 일치하는지 확인하고 프로젝트 검사를 실행한 뒤 Windows x86-64 실행 파일을 빌드하여 SHA-256 체크섬과 함께 하나의 GitHub Release에 게시합니다.

현재 프로젝트는 Windows만 지원하므로 Windows x86-64 결과물만 게시합니다. 이는 명시적인 플랫폼 결정이며 검증되지 않은 크로스 플랫폼 지원을 주장하는 것이 아닙니다.

## 보안

지원 버전과 비공개 취약점 신고 방법은 [SECURITY.md](SECURITY.md)를 참조하세요. 공개 issue에 취약점을 공개하지 마세요.

## 기여

개발 절차와 commit 요구 사항은 [CONTRIBUTING.md](CONTRIBUTING.md)를 참조하세요.

## 라이선스

Codex Tray는 [MIT License](LICENSE)로 제공됩니다.

## 프로토콜 참고 자료

- [Codex App Server](https://learn.chatgpt.com/docs/app-server)
