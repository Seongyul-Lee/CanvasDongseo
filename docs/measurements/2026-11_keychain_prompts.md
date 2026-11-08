# CanvasDongseo — 키체인 허용 창 측정 보고서

| 항목 | 내용 |
|---|---|
| 측정일 | 2026-11-09 |
| 측정 기기 | MacBook (Apple M5), macOS 27.0, Xcode 27.2 |
| 대상 | 앱 비밀 저장(Data Protection 키체인), 앱이 실행하는 `claude` CLI(2.1.319)의 자격 증명 읽기 |
| 목적 | macOS 사용자가 키체인 허용 창·비밀번호 입력을 보는 경우를 없애거나 줄이는 설계를 측정 결과로 정한다 |

---

## 1. 요약

| # | 질문 | 결과 | 설계 반영 |
|---|---|---|---|
| M1 | `apple-native-keyring-store` 1.0.2에서 접근 그룹·정책이 실제로 적용되는가 | `access-group` 설정 키로 지정한 그룹에 저장된다. 정책은 **항목을 만들 때만** 적용되고, 이미 있는 항목에 다른 정책으로 쓰면 값만 바뀌고 정책은 그대로다 | [platform §2](../cross_cutting/02_platform.md), [data_access §2.5](../layers/data_access/README.md) `SessionStore` |
| M2 | 서명이 어긋나면 허용 창이 뜨는가 | **뜨지 않는다.** 프로파일이 없거나 맞지 않으면 실행 즉시 종료되고, entitlement가 없으면 `-34018`이 난다 | [security §5.2](../cross_cutting/01_security.md), [platform §3](../cross_cutting/02_platform.md) |
| M3 | `claude` CLI는 언제 키체인을 읽는가 | 프로세스 **시작 때만** 읽는다(`auth status` 4회, `-p` 1턴 5회). stream-json 세션은 둘째 턴에 읽지 않았다 | [ai_adapters §3](../layers/05_ai_adapters.md) |
| M4 | 화면이 잠긴 동안 백그라운드 읽기가 되는가 | `WhenUnlocked`(crate 기본값): 잠그고 약 10초 뒤부터 풀 때까지 `-25308`. `AfterFirstUnlockThisDeviceOnly`: 계속 성공 | `SessionStore` 정책 |

**결론**: 앱이 직접 관리하는 비밀은 Data Protection 키체인이라 ACL 허용 창이 생기지 않는다(M2). 남는 허용 창은 `claude` CLI가 **로그인 키체인**에 둔 자기 자격 증명에서 생긴다. 앱이 이를 고칠 수는 없고, CLI 실행 횟수와 동시 실행을 줄여 창의 수만 줄일 수 있다(M3, §5).

## 2. 측정 방법

- 측정용 Rust 바이너리(`keyring-core` 1 + `apple-native-keyring-store` 1.0.2 `protected` + `security-framework` 3.7)를 번들 ID `io.github.seongyul-lee.canvasdongseo`의 최소 `.app` 번들로 감쌌다. Tauri 앱과 같은 방식(번들 안 Mach-O + `embedded.provisionprofile` + entitlement)이다.
- entitlement: `com.apple.application-identifier`, `com.apple.developer.team-identifier`, `keychain-access-groups = [468HW42V9P.io.github.seongyul-lee.canvasdongseo]`. 모든 변형에 Hardened Runtime을 켰다.
- 프로파일: Xcode 자동 관리 `Mac Team Provisioning Profile: *`(앱 ID `468HW42V9P.*`, 키체인 그룹 `468HW42V9P.*`).
- 속성은 `kSecReturnAttributes`로 `agrp`(그룹), `pdmn`(정책)을 읽었다.
- 측정용 항목은 측정 후 모두 지웠다.

## 3. 결과

### 3.1 M1 접근 그룹과 정책

| 순서 | 동작 | `agrp` | `pdmn` |
|---|---|---|---|
| 1 | 새 항목, `access-policy=when-unlocked` | `468HW42V9P.io.github.seongyul-lee.canvasdongseo` | `ak` (WhenUnlocked) |
| 2 | 같은 항목에 `after-first-unlock-this-device-only`로 쓰기 | 같음 | **`ak` 그대로** |
| 3 | 지우고 `after-first-unlock-this-device-only`로 새로 만들기 | 같음 | `cku` (AfterFirstUnlockThisDeviceOnly) |
| 4 | 그룹을 생략하고 쓰기 | 같음(entitlement의 첫 그룹) | `cku` 그대로 |

- 그룹을 생략해도 entitlement의 첫 그룹에 들어갔다. 그래도 entitlement가 바뀌면 기본값이 달라지므로 그룹은 명시한다.
- 정책을 바꾸려면 **지우고 다시 만들어야** 한다. 정책을 바꾸는 앱 업데이트에는 이전(지우고 다시 쓰기) 단계가 필요하다.
- `sync` 속성은 비어 있다(`cloud-sync` 기본값 false, iCloud로 동기화되지 않음).

### 3.2 M2 서명 변형

| 변형 | 결과 |
|---|---|
| Apple Development 서명 + 프로파일 + entitlement | 저장·읽기 성공 |
| ad-hoc 서명 + entitlement, 프로파일 없음 | 실행 즉시 종료(종료 코드 137, SIGKILL) |
| Apple Development 서명 + entitlement, 프로파일 없음 | 실행 즉시 종료(137) |
| **Developer ID Application 서명 + 개발 프로파일** | 실행 즉시 종료(137) |
| Apple Development 서명 + 프로파일, entitlement 없음 | 읽기 실패 `-34018`(entitlement 없음) |

- 어느 경우에도 허용 창은 뜨지 않았다. 실패는 "실행되지 않음" 또는 "비밀이 없음"으로 나타난다.
- 종료 사유는 시스템 로그에서 확인하지 못했다. 제한된 entitlement를 프로파일이 승인하지 않아 AMFI가 막은 것으로 추정한다.
- 배포 빌드에는 **Developer ID 프로파일**(Developer ID 인증서를 포함한 프로파일)이 필요하다. 개발 프로파일을 그대로 넣으면 앱이 뜨지 않는다. 배포 조합은 §3.5에서 측정했다.

### 3.3 M3 `claude` CLI의 키체인 읽기

PATH 맨 앞에 `security` 기록용 shim을 두었다. shim은 기록을 남긴 뒤 `/usr/bin/security`로 exec하므로 키체인 판정은 바뀌지 않는다. 처음 쓴 `ps` 폴링은 직접 실행한 `security` 5회도 잡지 못해 버렸다.

| 실행 | `security find-generic-password` 호출 수 |
|---|---|
| `claude auth status` 1회 | 4 |
| `claude auth status` 순차 3회 | 12 |
| `claude auth status` 동시 3회 | 12 |
| `claude -p` 1턴 | 5 |
| stream-json 세션 2턴(첫 턴 뒤 12초 대기 후 둘째 턴) | 5 (둘째 턴에 0) |

- 읽기는 프로세스 시작에 몰려 있다. 대화마다 프로세스 하나를 오래 쓰면 턴이 늘어도 읽기가 늘지 않는다.
- 시각 비교(stream-json 세션 1회): 5번의 읽기가 모두 첫 `system/init` 이벤트보다 약 2.1초 먼저 끝났다. 그래서 "시작 구간"을 실행부터 첫 `system/init`까지로 잡는다.
- 동시 실행은 총 횟수를 바꾸지 않는다. 그러나 허용이 풀린 상태에서는 읽기마다 창이 하나씩 떠서, 동시에 실행하면 **창이 겹쳐 쌓인다**([claude-code#87348](https://github.com/anthropics/claude-code/issues/87348)).
- 이 Mac의 항목 상태(값은 읽지 않음): `Claude Code-credentials`와 설정 폴더별 `Claude Code-credentials-68f1a6ad`가 로그인 키체인에 있다. 파티션은 `apple-tool:`, 허용 앱은 `/usr/bin/security`라서 지금은 창 없이 읽힌다. `~/.claude/.credentials.json`(권한 600)도 있다.

### 3.4 M4 화면 잠금 중 읽기

두 프로세스가 2초마다 각 정책의 항목을 읽게 하고, `CGSSessionScreenIsLocked`로 잠금 상태를 함께 기록했다.

| 구간 | `WhenUnlocked` | `AfterFirstUnlockThisDeviceOnly` |
|---|---|---|
| 잠금 전 | 성공 | 성공 |
| 잠금 직후 ~10초 | 성공 | 성공 |
| 잠금 10초 뒤 ~ 잠금 해제 | **`-25308`** (User interaction is not allowed) | 성공 |
| 잠금 해제 후 | 2초 안에 다시 성공 | 성공 |

- 화면 잠금 중에 도는 정기 갱신은 `WhenUnlocked`이면 비밀을 읽지 못한다. 이때 앱이 "로그아웃"으로 오판하면 사용자에게 재로그인을 요구하게 된다.
- 첫 잠금 해제 전(재부팅 직후 로그인 화면)은 측정하지 않았다.

### 3.5 배포 경로(공증한 `.pkg` 설치본)

같은 측정용 바이너리를 배포와 같은 경로로 만들어 설치했다. 인증서·프로파일 현황은 [Apple Developer 계정](../04_apple_developer_account.md)에 있다.

| 단계 | 내용 | 결과 |
|---|---|---|
| 프로파일 | `xcodebuild -exportArchive`(`developer-id`, `-allowProvisioningUpdates`)로 만든 Developer ID 프로파일. 새 G2 인증서를 만든 뒤 export를 다시 해서 프로파일에 G2 인증서가 들어간 것을 확인했다 | 첫 프로파일에는 이전 CA 인증서만 있었다 |
| 앱 서명 | Developer ID Application(G2, SHA-1로 지정) + Hardened Runtime + 타임스탬프 + entitlement(§2) | `codesign -v --strict` 통과 |
| 패키지 | `pkgbuild` → `productsign`(Developer ID Installer, G2) | `pkgutil --check-signature` 통과 |
| 공증 | `notarytool submit --wait` | **Accepted**, issues 없음. 새 인증서로 처음 제출해 약 30분 넘게 걸렸다(이전 제출은 약 4분) |
| staple·평가 | `stapler staple`, quarantine 속성을 붙인 뒤 `spctl -t install` | `accepted`, `source=Notarized Developer ID` |
| 설치 | `sudo installer -pkg … -target /` → `/Applications` | 설치본 `spctl` `accepted` |
| 키체인(직접 실행) | `AfterFirstUnlockThisDeviceOnly`로 저장·읽기 | 성공, `agrp` = 앱 접근 그룹, `pdmn` = `cku` |
| 키체인(`open`으로 실행) | LaunchServices·Gatekeeper 경로 | 실행 종료 없이 읽기 성공 |
| 개발 빌드와 공유 | Apple Development 서명 개발 빌드가 설치본이 만든 항목 읽기 | 성공 |

- 배포본과 개발 빌드가 같은 접근 그룹을 쓰므로, **개발 기기에서는 개발 빌드가 배포본의 비밀을 읽고 쓴다.** 개발 중 시험이 실사용 세션(쿠키는 쓸 때마다 바뀜)을 깨뜨릴 수 있으므로 개발 빌드는 서비스 이름에 접미사를 붙여 항목을 나눈다.
- 화면 잠금 중 읽기(M4)를 설치본으로 다시 측정했다: 잠그고 약 11초 뒤부터 `WhenUnlocked`는 `-25308`, `AfterFirstUnlockThisDeviceOnly`는 계속 성공해 §3.4와 같았다. 잠금 해제 뒤 복구는 측정 시간 안에 풀지 않아 §3.4 결과로 대신한다.

## 4. 결정

| 결정 | 근거 |
|---|---|
| 앱 비밀은 모두 `AfterFirstUnlockThisDeviceOnly`로 만든다. `RequireUserPresence`는 쓰지 않는다 | M4. "this device only"는 백업·다른 기기로 옮겨지지 않게 한다(D6: 세션 소유자는 하나) |
| `access-group`을 명시하고 `cloud-sync`는 켜지 않는다 | M1 |
| 정책이 다른 기존 항목을 발견하면 값을 읽어 지우고 새 정책으로 다시 만든다 | M1 순서 2 |
| `-25308`은 "잠겨 있어 지금 읽을 수 없음"으로, `-34018`은 "서명·entitlement 오류"로, `-25300`은 "저장된 값 없음"으로 구분한다. 앞의 둘을 로그아웃으로 다루지 않는다 | M2, M4 |
| 배포 빌드는 Developer ID 프로파일을 번들에 넣는다. 서명 후 검사에서 프로파일·entitlement·인증서 조합을 확인한다 | M2, §3.5 |
| 개발 빌드는 키체인 서비스 이름에 접미사를 붙여 배포본과 항목을 나눈다 | §3.5 |
| `claude` 프로세스는 대화마다 하나를 오래 쓰고, CLI의 시작 구간(상태 확인 포함)만 한 줄로 세운다. 실행 전체를 세우면 대화 중 격리 요약 세션과 교착된다 | M3 |

## 5. 앱이 하지 않는 것

- `Claude Code-credentials` 항목을 읽거나, 지우거나, 파티션·ACL을 바꾸지 않는다. 다른 프로그램의 자격 증명이고 S6(AI 자격 증명을 다루지 않음)에 어긋난다.
- `CLAUDE_CODE_OAUTH_TOKEN`이나 `.credentials.json` 내보내기를 권하지 않는다. 앞은 S6 위반으로 이미 기각했고([ai_adapters §2](../layers/05_ai_adapters.md)), 뒤는 평문 저장이다.

## 6. 측정하지 못한 것

- 재부팅 뒤 첫 잠금 해제 전 `AfterFirstUnlock` 항목 읽기
- `claude` 자격 증명의 허용이 풀린 상태에서 실제로 뜨는 창의 수(지금은 허용 상태라 창이 뜨지 않음)
