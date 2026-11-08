# CanvasDongseo — Apple Developer 계정과 서명 자산

| 항목 | 내용 |
|---|---|
| 독자 | 개발자 본인, 서명·배포를 돕는 AI 에이전트 |
| 기준일 | 2026-11-09 (포털·키체인을 직접 조회한 결과) |

> **이 문서의 역할**: Apple 계정, Apple Developer Program 멤버십, 인증서, 프로파일, 번들 ID(App ID)의 현황과 그 근거를 기록한다. 서명·공증 설정 값(번들 ID, 접근 그룹, 배포 빌드 조건)은 [platform §3](cross_cutting/02_platform.md)에 두고 여기서는 링크한다.
> 비밀(앱 암호, 개인 키)과 개인 정보(전화, 주소, 이메일)는 적지 않는다.

---

## 1. 계정과 멤버십

| 항목 | 값 |
|---|---|
| Apple 계정 | 개인 Apple 계정 1개. 이 계정이 프로그램을 결제했다 |
| 프로그램 | Apple Developer Program, 등록 유형 **개인(Individual)** |
| 팀 | `Seongyul Lee`, 팀 ID `468HW42V9P` |
| 역할 | **Account Holder** (개인 등록이라 결제한 계정이 곧 Account Holder) |
| 갱신일 | 2027-02-10 |
| 약관 | Program License Agreement 2026-11-02 수락 |

**Xcode의 역할 표시 문제**: Xcode → Settings → Apple Accounts는 같은 계정의 역할을 **Admin**으로 보여 주었고, Manage Certificates에서 Developer ID 인증서를 만들면 "This operation can only be performed by the Account Holder"로 거부됐다(2026-11-09). 웹 포털(developer.apple.com)에서는 같은 계정으로 문제없이 만들어졌다. Developer ID 인증서는 **웹 포털에서 만든다.** 계정을 지웠다가 다시 추가해도 역할은 Admin으로 표시됐다(2026-11-09). Manage Certificates 목록에도 G2 인증서 2개는 보이지 않고 이전 CA Developer ID Application만 보인다. 키체인의 서명 인증서와 프로파일은 로그아웃해도 그대로였다.

**알려진 로컬 증상**: Xcode 계정이 가끔 로그아웃되고, 다시 로그인하면 인증서가 보이지 않고 팀을 다시 골라야 하는 일이 있었다. 2026-10-29에 만든 notarytool 키체인 프로파일(`canvasdongseo-notary`)도 그 뒤 로그인 키체인에서 사라져 있었다. 원인은 확인하지 않았다(§6).

## 2. 인증서

| 종류 | 포털 ID | 발급 CA | 만료 | 개인 키 위치 | 용도 | 상태 |
|---|---|---|---|---|---|---|
| Developer ID Application | `Z85QX747AK` | **G2** | 2027-12-08 | 이 Mac 로그인 키체인 (SHA-1 `68E4B92F1C8452CCC9A768B719A68DE2185FDAD6`) | CanvasDongseo 앱·ffmpeg 번들 코드 서명 | **사용** |
| Developer ID Installer | `J75B8S6M2R` | **G2** | 2027-12-08 | 이 Mac 로그인 키체인 (SHA-1 `A816D5A4197B017574031E08CC2C90C8EFEF9259`) | `.pkg` 서명 | **사용** |
| Apple Development | `TB3G3Y7P62` | WWDR G3 | 2027-08-26 | 이 Mac | 개발 빌드 서명(Xcode 자동 관리) | 사용 |
| iOS Distribution | `FT34H6KF22` | — | 2027-09-18 | 이 Mac에 없음(EAS가 보관) | 여기주차(`com.yeogijucha.app`) Ad Hoc | 다른 프로젝트 |
| Developer ID Application | `P4NF2ZD3UB` | 이전 Sub-CA (`OU=Apple Certification Authority`) | 2027-02-02 | 이 Mac | 이전 버전 앱(2026-10-30 공증) 서명 | **쓰지 않음.** 만료까지 둔다 |
| Developer ID Application Managed | `6AW69MPD75` | — | 2027-02-02 | Apple(클라우드 서명) | Xcode 클라우드 서명 | 포털에서 폐기 불가(교체만) |

- **이전 Sub-CA 인증서는 2027-02-01 이후 서명에 쓸 수 없다**([Apple Account Help](https://developer.apple.com/help/account/certificates/replace-developer-id-certificates)). 새 인증서는 G2로 만들었다.
- **Developer ID Application 인증서가 2개라서 이름으로 서명하면 모호하다.** 빌드 스크립트는 SHA-1로 지정한다.
- Developer ID 인증서 한도는 종류별로 5개다(Application, Installer 따로).
- 포털에는 Developer ID 인증서의 폐기 버튼이 없다(2026-11-09 확인). 폐기는 Apple 지원에 요청해야 하며, 그 인증서로 서명한 앱이 실행되지 않을 수 있다.
- 새 개인 키는 이 Mac에서 만들었다(RSA 2048, CSR 업로드). 키는 로그인 키체인에만 있으므로 **암호를 건 `.p12`로 따로 백업한다.**

## 3. 프로비저닝 프로파일

| 프로파일 | 종류 | App ID | 인증서 | 만료 | 위치 |
|---|---|---|---|---|---|
| `Mac Team Direct Provisioning Profile: io.github.seongyul-lee.canvasdongseo` | Developer ID (Xcode 자동 관리, 모든 기기) | `468HW42V9P.io.github.seongyul-lee.canvasdongseo` | Developer ID Application 3개(이전 CA, Managed, **G2**) | 2044 | `~/Library/Developer/Xcode/UserData/Provisioning Profiles/` (UUID `5bfd7229-…`가 G2 포함본) |
| `Mac Team Provisioning Profile: *` | Mac 개발(Xcode 자동 관리) | `468HW42V9P.*` | Apple Development | 2027-11-08 | 같은 폴더 |
| `iOS Team Provisioning Profile: *`, `…: com.yeogijucha.app` | iOS 개발(Xcode 자동 관리) | `*`, 여기주차 | Apple Development | 2027 | 같은 폴더 |
| `*[expo] com.yeogijucha.app AdHoc …` | iOS Ad Hoc(EAS) | `com.yeogijucha.app` | iOS Distribution `FT34H6KF22` | 2027-09-18 | 포털 |

- Developer ID 프로파일은 `xcodebuild -exportArchive`(`method = developer-id`, `-allowProvisioningUpdates`)를 실행할 때 Xcode가 만든다. **새 인증서를 만든 뒤에는 export를 다시 해서 프로파일에 새 인증서가 들어갔는지 확인한다.** 첫 export(UUID `a64a61d4-…`)에는 이전 CA 인증서만 들어 있었다.
- Developer ID 서명에 개발 프로파일을 넣으면 앱이 실행 즉시 종료된다([키체인 측정 §3.2](measurements/2026-11_keychain_prompts.md)).
- Xcode 자동 관리 프로파일은 포털 목록에 나오지 않는다.

## 4. 번들 ID (App ID)

| App ID | 이름 | 프로젝트 | 상태 |
|---|---|---|---|
| `io.github.seongyul-lee.canvasdongseo` | XC io github seongyul-lee canvasdongseo | **이번 CanvasDongseo** | 사용. 키체인 접근 그룹 `468HW42V9P.io.github.seongyul-lee.canvasdongseo` |
| `*` | XC Wildcard | Xcode 개발 프로파일 | 사용 |
| `com.yeogijucha.app` | YeogiJucha | 여기주차(개발 중인 모바일 앱) | 사용 |
| `com.lsy.dailyself` | DailySelf | 폐기한 프로젝트 | **삭제 불가.** App Store Connect에서 이미 삭제한 앱에 묶여 있다("Delete all the Apps related to this bundle"). 남겨 둔다 |

- ffmpeg 샌드박스 번들의 식별자 `io.github.seongyul-lee.canvasdongseo.ffmpeg`는 App ID로 등록하지 않는다(entitlement에 프로파일이 필요한 권한이 없다).
- 이전 버전 앱의 번들 ID `io.github.seongyullee.canvasdongseo`(하이픈 없음)는 App ID로 등록돼 있지 않다.

## 5. 공증

| 항목 | 값 |
|---|---|
| 방식 | `xcrun notarytool submit … --keychain-profile canvasdongseo-notary --wait` |
| 자격 증명 | 앱 암호를 notarytool 키체인 프로파일 `canvasdongseo-notary`에 저장(2026-11-09 다시 등록). 앱 암호는 파일·저장소에 두지 않는다 |
| 이전 자격 증명 | 2026-10-29에 만든 앱 암호 "CanvasDongseo notary"는 평문 보관본이 있어 2026-11-09 무효화했다 |

## 6. 정리 기록과 남은 일

**2026-11-09 정리**

| 항목 | 조치 |
|---|---|
| DailySelf Ad Hoc 프로파일 | 삭제 |
| DailySelf용 iOS Distribution 인증서(`FLF9AS5QNQ`, 2027-02-10) | 폐기 |
| App ID `com.lsy.dailyself` | 삭제 시도 → 거부(§4) |
| 이전 CA Developer ID Application(`P4NF2ZD3UB`) | 폐기 시도 → 포털에 폐기 기능 없음(§2) |
| Developer ID Installer, Developer ID Application (G2) | 새로 발급 |

**남은 일**

- Xcode 로그아웃·키체인 항목 소실 증상의 원인 확인. 다시 생기면 시각을 기록해 시스템 로그를 본다
- 새 G2 인증서 개인 키의 `.p12` 백업
- 2027-02-02 이후 이전 CA 인증서가 만료되면 이 문서에서 행을 지운다
- 멤버십 갱신(2027-02-10)
