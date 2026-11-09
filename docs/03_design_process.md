# CanvasDongseo — Design Process

| 항목 | 내용 |
|---|---|
| 독자 | 개발자 본인, 후속 문서·구현을 돕는 AI 에이전트 |
| 근거 | [PRD](01_prd.md), UX Strategy(작성 예정), [Presentation](layers/03_presentation.md), [스크럼 보드](05_scrum_board.md) |

> **이 문서의 역할**: 화면 디자인을 *어떤 순서로, 어떤 산출물로* 진행하는지 정한다. Actor·시나리오, UX 원칙, 화면 구성, 문구·용어 규칙은 UX Strategy가, 앱 안의 구현 구조(theme·components·screens, 토큰 import)는 [Presentation](layers/03_presentation.md)이, 스프린트 운영은 [스크럼 보드](05_scrum_board.md)가 맡는다.

---

## 1. 흐름

```
[입력]        UX Strategy (Actor → 시나리오, 화면 구성, 문구·용어 규칙)
[처음 한 번]  Foundation
[기능마다]    ① User Flow → ② 상태 목록 → ③ 목업 → ④ 구현 → ⑤ 실제 앱 확인 → ⑥ 시나리오 점검
```

- 앱 전체를 단계별로 한 번씩 훑지 않는다. **기능 하나를 ①~⑥까지 끝내고** 다음 기능으로 간다. 한 스프린트에 여러 기능을 끝낼 수 있다.
- 기능 하나는 백로그 항목 하나다. 항목의 "화면 작업 단계" 체크 목록이 ①~⑥이다([스크럼 보드 §4](05_scrum_board.md#4-템플릿)).
- Foundation은 첫 기능의 ③ 전에 한 번 한다. 이후 바꿀 일이 생기면 그 기능에서 고친다.

| 단계 | 입력 | 산출물 | 위치 |
|---|---|---|---|
| Foundation | UX Strategy, [Presentation §2.2](layers/03_presentation.md#22-디자인-토큰) | 기반 컴포넌트 선택, 원시·의미 토큰, 색·타이포·간격·모서리·그림자·아이콘·포커스 링 목업 | 토큰: `ui/src/theme/tokens.json`, 목업: `design/foundation/` |
| ① User Flow | UX Strategy의 시나리오 | 시작 → 성공까지의 흐름과 실패·중단 갈래 | User Flow 문서(작성 예정) |
| ② 상태 목록 | User Flow, §2.2의 상태 출처 | 이 기능에서 생길 수 있는 상태와 각 상태의 ViewModel 샘플 | `design/samples/<기능>/` |
| ③ 목업 | 상태 목록, 샘플 | 구조 목업 → 토큰을 적용한 목업 | `design/wireframes/`, `design/screens/` |
| ④ 구현 | 샘플, 목업 | [Presentation §2.1](layers/03_presentation.md#21-프런트엔드-spa-내부-구조) 구현 순서를 따른다 | `ui/`, `crates/presentation/*` |
| ⑤ 실제 앱 확인 | 상태 목록 | 상태별 캡처, PR의 before/after | PR(레포에 커밋하지 않는다) |
| ⑥ 시나리오 점검 | 시나리오, User Flow | 시나리오별 통과·막힘 기록 | PR 본문 |

## 2. 단계별 기준

### 2.1 Foundation

- **기반 컴포넌트를 먼저 고른다.** 키보드 조작·포커스·스크린 리더 지원을 갖춘 스타일 없는 컴포넌트 라이브러리를 쓰고, 겉모습만 토큰으로 입힌다. 다이얼로그·메뉴·포커스 처리를 직접 만들지 않는다. 후보와 결정은 [Presentation §5](layers/03_presentation.md#5-미확인-항목)에 둔다.
- **토큰은 2단이다.** 원시 토큰(색 단계, 간격 단계)과 의미 토큰(`surface`, `textMuted`, `warningSurface` 등)으로 나누고, 컴포넌트와 목업은 의미 토큰만 쓴다. 라이트·다크는 의미 토큰이 가리키는 원시 토큰만 바꾼다. 형식은 [Presentation §2.2](layers/03_presentation.md#22-디자인-토큰).
- 모바일과 달리 데스크톱에서 추가로 정한다.
  - 라이트·다크 모드(OS 설정을 따른다)
  - 창 크기 변화에 따른 배치(고정 폭이 아니다)
  - 키보드 조작과 포커스 표시
  - 한글 글꼴: OS 기본(macOS Apple SD Gothic Neo, Windows Malgun Gothic)을 쓸지, 글꼴을 앱에 포함할지
- 문구, 용어, 한글 표기 규칙은 Foundation에서 정하지 않는다. **UX Strategy가 먼저 정하고 우선한다.** Foundation이나 이후 단계에서 바꾸거나 더해야 하면 UX Strategy를 함께 고친다.

### 2.2 User Flow와 상태 목록

- 대상 기능과 화면은 UX Strategy의 시나리오와 화면 구성을 따른다. 이 문서에서는 화면을 정하지 않는다.
- User Flow에는 성공 경로뿐 아니라 실패·중단 갈래를 함께 그린다. ⑥에서 갈래마다 끝까지 가 볼 수 있어야 한다.
- 상태 목록은 화면 단위가 아니라 **그 기능에서 화면에 나타날 수 있는 상태** 단위로 만든다. 빠뜨리지 않도록 아래 출처를 확인한다.

| 출처 | 확인할 것 |
|---|---|
| AI 중립 이벤트([application §4.2](layers/02_application.md#42-ai)) | 스트리밍 중, 도구 사용, 권한 확인, 사용량 한도, 완료, 오류 |
| 확인 요청([application §4.4](layers/02_application.md#44-사용자-상호작용과-로컬-저장)) | `UserConfirmation`, `OtpPrompt` |
| 작업 상태([application §4.6](layers/02_application.md#46-작업-오래-걸리는-유스케이스)) | `queued`, `running`(진행률), `done`, `failed`, `cancelled` |
| application 오류([application §4.5](layers/02_application.md#45-application-오류)) | 오류 종류마다 화면에 보일 모습 |
| 비어 있음·불러오는 중 | 첫 사용, 데이터가 아직 없음 |

- 상태 하나가 ViewModel의 `kind` 값 하나와 샘플 JSON 하나가 된다. `design/samples/<기능>/README.md`에 상태 목록을 두고, 같은 폴더에 상태마다 `<상태>.json`을 둔다.

### 2.3 목업

- 샘플 데이터는 **ViewModel 모양의 JSON**이다. 목업이 곧 ViewModel 타입 초안이 되고, 화면이 조건 분기 없이 그려지는지 이 단계에서 확인한다(예: 배너는 `banner.kind`로 받는다).
- **샘플 JSON은 Presenter 테스트의 기대값이다**(읽는 방법은 [Presentation §2.1](layers/03_presentation.md#21-프런트엔드-spa-내부-구조)). ③에서 목업이 이 파일을 그리고, ④에서 Presenter 테스트가 Presenter 출력과 같은 파일을 비교한다. ViewModel이 바뀌면 테스트가 실패하므로 목업과 앱이 어긋나지 않는다. 샘플을 고치면 목업과 테스트가 함께 따라온다.
- 구조 목업(`design/wireframes/`)에서 배치를 정한 뒤, 토큰을 적용한 목업(`design/screens/`)을 만든다.
- 토큰 원본은 `ui/src/theme/tokens.json` 하나다. `design/`에는 두지 않는다. 목업은 생성된 `tokens.css`를 상대 경로로 불러와 앱과 같은 값을 쓴다.
- 목업에 값을 직접 적지 않는다. 새 값이 필요하면 먼저 토큰으로 추가한다.
- 같은 스프린트 안에서 순서를 고를 수 있으면 컴포넌트를 많이 쓰는 기능부터 한다. 앞 기능에서 정한 토큰과 컴포넌트를 뒤 기능이 다시 쓰게 된다.

### 2.4 실제 앱 확인

| 항목 | 기준 |
|---|---|
| 상태 | 상태 목록의 상태마다 실제 앱에서 캡처한다 |
| 환경 | 라이트·다크, 좁은 창·넓은 창, 키보드만으로 조작 |
| 접근성 | 글자와 배경의 대비, 포커스 표시 |
| PR 증거 | 화면이 바뀌면 before/after 캡처를 PR에 붙인다. 움직임·타이밍이 있으면 짧은 영상을 붙인다. 캡처는 레포에 커밋하지 않는다 |

### 2.5 시나리오 점검

실제 사용자 대상 확인 대신, UX Strategy의 시나리오와 User Flow로 확인한다.

- 이 기능에 걸린 시나리오를 하나씩 실제 앱에서 따라간다. User Flow의 갈래(성공·실패·중단)마다 그 상태에 도달하고, 빠져나오거나 끝낼 수 있는지 본다.
- 막힌 곳은 이 기능 안에서 고친다. 이 기능의 범위를 넘으면 백로그 항목으로 올린다.
- 결과는 PR 본문에 시나리오 번호별로 "통과" 또는 "막힘(이유)"으로 적는다.
- 실제 학생 대상 사용성 확인은 MVP가 끝난 뒤 검토한다.

## 3. 규칙을 늘리는 기준

- 린트·검사 규칙은 **같은 실수가 두 번 나왔을 때** 더한다. 일어나지 않은 실수를 예상해 미리 만들지 않는다.
- 규칙을 더하면 그 규칙이 지키는 내용을 적은 문서에 한 줄로 근거를 남긴다.

## 4. 폴더 구성

```
design/
  foundation/          Foundation 목업
  wireframes/          ③ 구조 목업
  screens/             ③ 토큰을 적용한 목업
  samples/<기능>/
    README.md          ② 상태 목록
    <상태>.json        ViewModel 샘플 = Presenter 테스트 기대값
ui/src/theme/
  tokens.json          토큰 원본 (앱과 목업이 함께 쓴다)
```

## 5. 미확인 항목

- 토큰 빌드 도구와 목업에서 `tokens.css`를 갱신하는 방법(감시 빌드 등)
- 한글 글꼴 포함 여부
- 목업과 SPA(브라우저 실행, command 대역)를 같은 샘플로 캡처하는 도구. Tauri 창 자체의 캡처는 우선 직접 한다
