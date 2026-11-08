# Presentation

| 항목 | 내용 |
|---|---|
| 독자 | 개발자 본인, 후속 문서·구현을 돕는 AI 에이전트 |
| 근거 | [System Architecture](../02_system_architecture.md), 레거시 `docs/15`(UI가 오류 코드를 해석해 분기하지 않는다) |

> **이 문서의 역할**: 화면(WebView)과 코어 사이의 View · ViewModel · Presenter · Controller 역할과 위치를 정한다. 화면 구성과 흐름은 UX Strategy·User Flow가 맡는다.

---

## 1. 개요

| 항목 | 내용 |
|---|---|
| 링 | Interface Adapters (View는 Frameworks & Drivers) |
| crate | `presentation` (+ 프런트엔드 SPA) |
| 의존할 수 있는 것 | `application` |

## 2. 구조

```
View(SPA) ─command─▶ Controller(tauri command) ─▶ 유스케이스
   ▲                                                  │ 출력 DTO
   └──── ViewModel(JSON) ◀── Presenter ◀──────────────┘
```

| 역할 | 위치 | 하는 일 |
|---|---|---|
| View | 프런트엔드 SPA 컴포넌트 | ViewModel을 그리고 입력을 command로 보낸다. 조건 분기·계산·변환을 하지 않는다 |
| ViewModel 상태 보관 | 프런트엔드의 얇은 store | 받은 ViewModel·이벤트를 보관하고 View에 연결한다. 값을 새로 만들지 않는다 |
| ViewModel 타입 | `presentation` | 화면용 DTO(과목 카드, 배너 종류, 버튼 활성 여부, 표시용 마감 문구 등) |
| Presenter | `presentation` | 출력 DTO·오류를 ViewModel로 바꾼다. 어떤 배너·안내 행동을 줄지 같은 판단을 한다 |
| Controller | `presentation`의 tauri command | 입력을 받아 유스케이스를 부르고 Presenter에 넘긴다. 로직 없음 |

- 판단이 들어간 표시 문구는 Presenter에서, 순수 서식(로캘 숫자·날짜 포맷)은 View에서 만든다.
- ViewModel 타입은 Rust에서 TypeScript로 생성해 어긋나지 않게 한다(`ts-rs`·`specta` 등, 미조사).
- Tauri capabilities로 WebView가 부를 수 있는 command를 제한한다.
- AI용 Presenter는 이 레이어가 아니라 [entrypoints](04_entrypoints.md)에 있다.

## 3. 대화 화면의 슬래시 커맨드

| 분류 | 예 | 처리 |
|---|---|---|
| 앱이 대신 처리 | `/clear`, `/login`, `/model` | 새 대화, 제공자 로그인 화면, 모델 선택으로 연결. 대화 기록의 원본은 앱이다 |
| 그대로 전달 | `/compact`, 사용자 커스텀 커맨드, 스킬·플러그인 커맨드 | 제공자 CLI에 그대로 보낸다 |
| 대화형 전용 | `/config`, `/mcp` 화면, `/resume` 선택기 | "앱에서 지원하지 않음" 안내 |

- 분류는 제공자마다 다르므로 `AiSession.capabilities()`로 받는다([application §4.2](02_application.md)). 사용할 수 있는 커맨드·스킬 목록은 대화 시작 이벤트에서 받아 입력창 자동 완성에 쓴다.
- AI가 요청한 권한 확인(`PermissionRequest`)과 개인 데이터 쓰기 확인은 앱 화면에서 받는다.

## 4. 이웃 레이어와의 계약

| 이웃 | 계약 |
|---|---|
| application | 유스케이스만 호출한다. 출력 DTO를 받는다 |
| 프런트엔드 | command 이름과 ViewModel 타입(생성된 TypeScript)이 계약이다 |

## 5. 미확인 항목

- 프런트엔드 프레임워크(Svelte 또는 React). UX Strategy 뒤 확정
- Rust → TypeScript 타입 생성 도구
