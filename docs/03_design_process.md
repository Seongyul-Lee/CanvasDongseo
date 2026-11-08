# CanvasDongseo — Design Process

| 항목 | 내용 |
|---|---|
| 독자 | 개발자 본인, 후속 문서·구현을 돕는 AI 에이전트 |
| 근거 | [PRD](01_prd.md), [Presentation](layers/03_presentation.md) |

> **이 문서의 역할**: 화면 디자인을 *어떤 순서로, 어떤 산출물로* 진행하는지 정한다. UX 방향과 화면 구성은 UX Strategy, 앱 안의 구현 구조(theme·components·screens, 토큰 import)는 [Presentation](layers/03_presentation.md)이 맡는다.

---

## 1. 흐름

```
와이어프레임(HTML 목업) → Foundation(HTML 목업) → 화면별 토큰 채우기(tokens.json + 목업) → 구현
```

| 단계 | 산출물 | 위치 |
|---|---|---|
| 1. 와이어프레임 | 화면별 HTML 목업, ViewModel 모양의 샘플 JSON | 목업: `design/wireframes/`, 샘플: `design/samples/` |
| 2. Foundation | 색·타이포·간격·모서리·그림자·아이콘·포커스 링 목업 | `design/foundation/` |
| 3. 화면별 디자인 | `tokens.json` 값, 토큰을 적용한 화면 목업 | 토큰: `ui/src/theme/tokens.json`, 목업: `design/screens/` |
| 4. 구현 | [Presentation](layers/03_presentation.md) §2.1 구현 순서를 따른다 | `ui/`, `crates/presentation` |

## 2. 단계별 기준

### 2.1 와이어프레임

- 대상 화면은 UX Strategy가 정한 화면 구성을 따른다. 이 문서에서는 화면을 정하지 않는다.
- 목업에 넣는 샘플 데이터는 **ViewModel 모양의 JSON**으로 만든다. 목업이 곧 ViewModel 타입 초안이 되고, 화면이 조건 분기 없이 그려지는지 이 단계에서 확인한다(예: 배너는 `banner.kind`로 받는다).

### 2.2 Foundation

모바일과 달리 데스크톱에서 추가로 정한다.

- 라이트·다크 모드(OS 설정을 따른다)
- 창 크기 변화에 따른 배치(고정 폭이 아니다)
- 키보드 조작과 포커스 표시
- 한글 글꼴: OS 기본(macOS Apple SD Gothic Neo, Windows Malgun Gothic)을 쓸지, 글꼴을 앱에 포함할지

### 2.3 화면별 디자인

- 토큰 원본은 `ui/src/theme/tokens.json` 하나다. `design/`에는 두지 않는다. 목업은 생성된 `tokens.css`를 상대 경로로 불러와 앱과 같은 값을 쓴다.
- 목업에 값을 직접 적지 않는다. 새 값이 필요하면 먼저 토큰으로 추가한다.
- 진행 순서: UX Strategy가 정한 화면 중 컴포넌트를 가장 많이 쓰는 화면부터 한다. 앞 화면에서 정한 토큰을 뒤 화면이 재사용하게 된다.

## 3. 폴더 구성

```
design/
  wireframes/   단계 1
  foundation/   단계 2
  screens/      단계 3
  samples/      ViewModel 모양 샘플 JSON
ui/src/theme/
  tokens.json   토큰 원본 (앱과 목업이 함께 쓴다)
```

## 4. 미확인 항목

- 토큰 빌드 도구와 목업에서 `tokens.css`를 갱신하는 방법(감시 빌드 등)
- 한글 글꼴 포함 여부
