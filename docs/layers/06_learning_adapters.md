# Learning Adapters

| 항목 | 내용 |
|---|---|
| 독자 | 개발자 본인, 후속 문서·구현을 돕는 AI 에이전트 |
| 근거 | [PRD §8.3](../01_prd.md), 2026-11-08 crates.io 조회 |

> **이 문서의 역할**: 학습 포트(`Transcriber`·`TextRecognizer`·`SlideExtractor`, [application §4.3](02_application.md))의 OS별 구현을 정한다. 확장 단계 기능이며 macOS를 먼저 구현한다.

---

## 1. 개요

| 항목 | 내용 |
|---|---|
| 링 | Frameworks & Drivers |
| crate | `learning/macos`, `learning/windows`(나중) |
| 의존할 수 있는 것 | `application`, `domain` |

- composition root에서 `#[cfg(target_os)]`로 구현을 고른다. 구현이 없는 OS에서는 "지원하지 않음"을 돌려준다.

## 2. 구현

| 포트 | macOS (먼저) | Windows (다음) |
|---|---|---|
| `Transcriber` | `whisper-rs` 0.16.0 (whisper.cpp, Metal 가속) | `whisper-rs` (CPU/CUDA) |
| `TextRecognizer` | `objc2-vision` 0.3.2 (Apple Vision) | `windows` crate (Windows.Media.Ocr) |
| `SlideExtractor` | ffmpeg 사이드카 + 장면 비교 | ffmpeg 사이드카 |

- 레거시는 `mlx-whisper`(Python 전용)를 썼다. Rust에서는 whisper.cpp 계열로 바꾼다.
- 모델 파일은 앱에 포함하지 않고 처음 쓸 때 내려받는다(크기·라이선스 확인 필요).

## 3. 미확인 항목

- `whisper-rs` 갱신이 느린 편(최근 2026-03). 착수 때 유지 상태 재확인
- `objc2-vision` 바인딩 범위(텍스트 인식 요청 API 포함 여부)
- 전사 모델 크기·속도와 배포 방식
