# example — 샘플 모드 예시

샘플 모드([Presentation §2.4](../../../docs/layers/03_presentation.md#24-샘플-모드))가 동작하는지 보여 주는 예시다. 실제 기능이 아니고 Presenter도 없다. **첫 기능의 샘플이 들어오면 지운다.**

## 보는 방법

```bash
npm --prefix ui run dev:sample
```

브라우저에서 `http://localhost:5173/example.html`을 연다. 화면 오른쪽 아래 패널에서 상태를 고른다. 고른 상태는 주소에 담긴다(`?sample.example=working`).

## 상태 목록

| 상태 | 파일 | 보여 주는 것 |
|---|---|---|
| 비어 있음 | `empty.json` | 안내 배너, 빈 목록 문구 |
| 불러오기 실패 | `error.json` | 오류 배너 |
| 목록 있음 | `ready.json` | command 응답 하나로 그린 목록 |
| 받는 중 | `working.json`, `working.events.json` | 이벤트를 순서대로 받아 바뀌는 진행 표시 |

## 지울 때 함께 지울 것

- `design/samples/example/`
- `ui/example.html`, `ui/src/example.tsx`
- `ui/src/screens/ExampleScreen.tsx`, `ui/src/components/ExampleView.tsx`, `ui/src/stores/example.ts`, `ui/src/commands/example.ts`, `ui/src/viewmodels/example.ts`
- `ui/src/commands/sample/responses.ts`의 `get_example` 줄
- `ui/boundaries.config.js`의 `example.tsx` 엔트리 줄, `ui/test/boundaries.test.js`의 `src/example.tsx`와 픽스처 `ui/test/fixtures/repo/ui/src/example.tsx`
- `ui/test/sample-guard.test.js`의 실제 프로젝트 빌드 테스트에서 `example.html` 입력(그때는 `index.html`이 `commands/core.ts`를 부른다)
- `ui/src/viewmodels/README.md`의 임시 예외 문단
