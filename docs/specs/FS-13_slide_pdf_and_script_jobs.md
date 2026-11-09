# FS-13 슬라이드 PDF·스크립트 작업

| 항목 | 내용 |
|---|---|
| 독자 | 개발자 본인, 후속 문서·구현을 돕는 AI 에이전트 |
| 상태 | 초안 |
| 시나리오 | S7-1, S7-2, S2-6 ([UX Strategy §4](../06_ux_strategy.md#4-actor와-시나리오)) |
| 화면 | 대화(작업 카드, 앱 메시지), 좌측 사이드바의 대화별 상태 ([UX Strategy §7.2](../06_ux_strategy.md#72-mvp-화면), [§7.4](../06_ux_strategy.md#74-대화별-상태)) |
| 유스케이스 | `ExtractSlidePdf`, `Transcribe`, 작업 상태 조회·취소 ([application §3](../layers/02_application.md#3-유스케이스-후보)), `JobRegistry`·`JobSpawner`·`CancelToken`·`ProgressSink` ([application §4.6](../layers/02_application.md#46-작업-오래-걸리는-유스케이스)), 학습 포트 ([application §4.3](../layers/02_application.md#43-학습)) |
| 근거 | [learning_adapters](../layers/06_learning_adapters.md), [domain §4.1·§4.2](../layers/01_domain.md#41-슬라이드-판정-강의-동영상--슬라이드-pdf), [security](../cross_cutting/01_security.md), [강의 동영상 처리 측정](../measurements/2026-11_lecture_media_processing.md), [presentation §1](../layers/03_presentation.md#1-개요), [entrypoints](../layers/04_entrypoints.md) |

> **이 문서의 역할**: 내려받은 강의 동영상으로 슬라이드 PDF와 스크립트를 만드는 오래 걸리는 작업의 시작·진행·취소·실패와, 결과 파일의 저장·무결성 기록을 정한다.
> 처리 단계와 수치(디코딩 방식, 전사 설정, 상한, 부품 목록, 결과 위치)는 [learning_adapters](../layers/06_learning_adapters.md), 슬라이드 판정 규칙과 이름 규칙은 [domain §4](../layers/01_domain.md#4-업무-규칙-후보)가 맡는다. 동영상 내려받기는 [FS-12](FS-12_material_download.md), 만든 스크립트로 공부하기는 [FS-14](FS-14_study_with_script.md)다.

---

## 1. 범위

| 하는 것 | 하지 않는 것 |
|---|---|
| 동영상 → 슬라이드 PDF(`ExtractSlidePdf`), 동영상 → 스크립트(`Transcribe`)를 작업으로 실행하기 | 동영상 내려받기(FS-12) |
| 작업 상태를 대화 안 작업 카드와 사이드바 대화별 상태로 알리기 | 동영상별 세부 진행률 표시(MVP에서 보이지 않는다, [UX Strategy §7.4](../06_ux_strategy.md#74-대화별-상태)) |
| 작업 취소 | 앱 창이 뒤에 있을 때의 OS 알림(MVP에 없다) |
| 시작 전 부품 확인과 상한 확인 | 부품 받기·다시 받기(설치 프로그램만 한다, [platform §3](../cross_cutting/02_platform.md#3-설치배포)) |
| 결과 파일 저장과 해시·HMAC 기록 | 슬라이드를 AI나 사람이 검토해 고르기(AI 검토 단계 없음, [learning_adapters §2](../layers/06_learning_adapters.md#2-처리-흐름)) |
| 학교 서버에서 받은 동영상을 신뢰하지 않는 입력으로 다루기(R4) | 학교 서버의 자막·대본 쓰기([05_commons §3](../layers/data_access/05_commons.md#3-다운로드-동작)) |
| | 요약·퀴즈·예상 시험지 만들기(FS-14) |
| | Windows에서 학습 기능 켜기(ffmpeg 격리 확인 전, F2) |

## 2. 기능 요구사항

### F1 작업 시작

- **입력**: 대상 동영상과 작업 종류(슬라이드 PDF, 스크립트). 동영상은 FS-12로 받아 학습 자료 폴더에 있는 로컬 파일이다([learning_adapters §2](../layers/06_learning_adapters.md#2-처리-흐름)). 동영상을 무엇으로 가리키는지(결과물 키, `content_id`, 경로)는 `cdx` 명령을 정할 때 정한다(§6).
- **경로**: 학생이 대화에서 요청하면 대화 AI가 `cdx` 명령으로 작업을 시작한다(추론, §6). 파일 경로 인자는 학습 자료 폴더 아래만 받는다([entrypoints §4](../layers/04_entrypoints.md#4-cdx-명령)).
- **동작**: F2의 확인을 통과하면 `JobRegistry`가 작업 ID를 발급하고 `queued` 상태로 등록한 뒤, `JobSpawner`가 작업 본문(`ExtractSlidePdf` 또는 `Transcribe`)을 백그라운드에서 실행한다.
- 작업은 **시작한 대화 ID**를 가진다. `cdx`로 시작했으면 대화 ID는 IPC 토큰에서 코어가 정하고, AI가 넘긴 값을 쓰지 않는다([application §4.6](../layers/02_application.md#46-작업-오래-걸리는-유스케이스)).
- **출력**: `cdx`는 작업이 끝나기를 기다리지 않고 작업 ID와 시작 상태를 바로 돌려준다(추론: 작업이 수 분 걸리므로 AI 제공자의 도구 호출 제한 시간을 넘긴다, §6).
- 학생이 "슬라이드 PDF와 스크립트를 만들어 줘"처럼 둘을 함께 요청하면 작업 두 개가 따로 생긴다(추론: 유스케이스가 둘이다, §6).

### F2 시작 전 확인

작업을 등록하기 전에 아래 순서로 확인하고, 하나라도 실패하면 작업을 만들지 않고 오류를 돌려준다(추론, §6). 오류는 그 대화의 앱 메시지로 보인다([UX Strategy §11](../06_ux_strategy.md#11-대응표) 부품 확인 행).

1. **OS 관문**: Windows는 ffmpeg 격리가 수용 기준을 통과하기 전까지 학습 기능(슬라이드 PDF·전사)을 켜지 않는다([security §6](../cross_cutting/01_security.md#6-수용한-위험) 출시 관문). 꺼져 있으면 ffmpeg를 실행하지 않고, 이 OS에서는 아직 쓸 수 없다는 안내를 돌려준다(오류 종류는 §6).
2. **부품**: `ComponentStore`가 필요한 부품의 위치를 찾고 SHA-256을 확인한다. ffmpeg는 실행 직전에 서명 요구 조건과 App Sandbox 권한도 확인한다([learning_adapters §4](../layers/06_learning_adapters.md#4-부품)). 확인은 작업을 시작할 때마다 한다.
   - 슬라이드 PDF: ffmpeg, OCR 언어. 스크립트: ffmpeg, 전사 모델, VAD 모델.
   - 없거나 해시·서명이 다르면 `ComponentMissing`(부품, 복구 방법)을 돌려준다. 앱은 부품을 스스로 받지 않는다.
3. **상한**: 동영상 파일 크기·재생 길이·해상도가 [learning_adapters §3.4](../layers/06_learning_adapters.md#34-상한-security-r4)의 상한을 넘으면 작업을 시작하지 않고 `Rejected(LimitExceeded)`를 돌려준다. 재생 길이·해상도를 시작 전에 어떻게 아는지는 §6.

### F3 작업 상태

- 상태는 `queued` → `running` → `done`·`failed`·`cancelled`이고, 전이 규칙은 `JobRegistry`가 갖는다([application §4.6](../layers/02_application.md#46-작업-오래-걸리는-유스케이스)). `JobRegistry`는 실행하지 않는다.
- 진행률은 `ProgressSink`로 화면에 보내고 `JobRegistry`가 보관한다. MVP 화면은 진행률을 그리지 않는다(F4).
- 화면(Tauri command)과 `cdx`(작업 ID로 상태 조회)는 같은 작업을 같은 상태로 본다.
- 처리 중 상한(묶음 수 500, ffmpeg 한 번 실행 시간 30분, [learning_adapters §3.4](../layers/06_learning_adapters.md#34-상한-security-r4))을 넘으면 작업을 끊고 `failed`로 끝내며, 사유는 `Rejected(LimitExceeded)`다.
- `done`이면 결과 파일의 위치(F7)를 작업 결과로 보관한다.

### F4 작업 카드와 대화별 상태

- **작업 카드**: 작업을 시작한 대화 안에 작업마다 카드 하나를 둔다. 카드에는 **상태 문구와 취소 버튼만** 둔다([UX Strategy §7.4](../06_ux_strategy.md#74-대화별-상태)). 취소 버튼은 `queued`·`running`일 때만 누를 수 있다(추론).
- **결과 위치**: `done`이 되면 그 대화에 결과 파일 위치가 보인다(S7-1). 카드의 상태 문구에 넣을지 앱 메시지로 남길지는 §6.
- **사이드바**: 작업을 시작한 대화의 상태는 작업이 `queued`·`running`인 동안 Working(경과 시간 표시)이다. `failed`면 Failed, 학생이 보지 않은 `done`이면 Done이다. 한 대화에 여러 상태가 겹치면 우선순위가 높은 하나만 보인다. 판정은 `presenters/shell`이 한다([presentation §1](../layers/03_presentation.md#1-개요)).
- **다른 대화 계속 쓰기**: 작업은 백그라운드에서 돌므로 작업 중에도 같은 대화와 다른 대화에서 질문하고 답을 받을 수 있다(S7-1, P6).
- 작업 카드와 사이드바 표시는 앱의 말이다. AI 메시지와 구별한다(P2).

### F5 취소

- 학생이 작업 카드의 취소 버튼을 누르면 작업 취소 유스케이스가 그 작업의 `CancelToken`에 취소 신호를 보낸다. `cdx`로 취소할 수 있는지는 §6.
- 유스케이스는 단계 사이에서 `CancelToken`을 확인한다. 전사 중에는 whisper-rs의 중단 콜백으로 취소를 전달한다([learning_adapters §3.2](../layers/06_learning_adapters.md#32-전사)). 디코딩 중인 ffmpeg 자식 프로세스는 끝낸다(추론, §6).
- 취소되면 상태는 `cancelled`이고, **결과 파일(슬라이드 PDF·스크립트)과 그 무결성 기록을 남기지 않는다**. 반쯤 만든 파일이 남지 않는 것은 `LearningOutputRepository`의 원자적 쓰기가 보장한다(S7-2).
- `queued`인 작업을 취소하면 실행하지 않고 `cancelled`가 된다(추론).

### F6 처리

- **슬라이드 PDF**: [learning_adapters §2](../layers/06_learning_adapters.md#2-처리-흐름)의 ①~⑦을 따른다. 판정(영역·구간·묶음·후보·점수·확정)은 `ExtractSlidePdf`가 [domain §4.1](../layers/01_domain.md#41-슬라이드-판정-강의-동영상--슬라이드-pdf) 규칙으로 직접 한다. 확정은 자동이며 AI나 사람에게 묻지 않는다.
- PDF 쪽에는 장면, 타임스탬프 띠, 보이지 않는 OCR 글자 층, 책갈피, 문서 정보가 들어간다([learning_adapters §3.3](../layers/06_learning_adapters.md#33-ocr과-pdf)).
- **스크립트**: `Transcriber`가 알린 음성 형식으로 디코딩하고, VAD를 켜고 전사해 줄마다 `[mm:ss] 문장`으로 쓴다([learning_adapters §3.2](../layers/06_learning_adapters.md#32-전사)).
- **신뢰하지 않는 입력**: 동영상은 학교 서버에서 받은 것이라 신뢰하지 않는다(R4). 디코딩은 형식 고정, 축소 빌드, 읽기 전용 stdin 핸들, App Sandbox의 네 겹으로 한다([learning_adapters §3.1](../layers/06_learning_adapters.md#31-디코딩)). 동영상 형식이 맞지 않으면 디코딩이 실패하고 작업은 `failed`가 된다.

### F7 결과 저장과 무결성 기록

- 결과 위치는 [domain §4.2](../layers/01_domain.md#42-학습-결과물-이름-규칙)의 키(학기, 과목, 종류, `{NN}주차_{M}차시_{제목}`)로 정하고, `LearningOutputRepository`가 [learning_adapters §5](../layers/06_learning_adapters.md#5-결과-파일)의 실제 경로로 바꾼다. 같은 차시의 동영상·슬라이드 PDF·스크립트는 키에서 종류만 다른 짝이다.
- 저장은 원자적으로 한다. 저장 직후 결과 파일의 해시를 앱 데이터에 기록하고 기록마다 HMAC을 붙인다. HMAC 키는 Data Protection 키체인에 있고 `SecretStore`로만 읽는다([application §4.3](../layers/02_application.md#43-학습), [security §4.1](../cross_cutting/01_security.md#41-신뢰-등급)).
- 이 기록이 결과물의 신뢰 등급을 정한다. 읽을 때 HMAC과 해시가 맞으면 `Official`, 맞지 않거나 기록이 없으면 `Peer`다. 읽는 쪽 동작은 FS-14가 정한다.
- **작업 폴더**: `<앱 데이터>/work/<content_id>/`에는 판정 기록(구간·묶음·확정 시각)만 둔다. 후보 장면 이미지는 남기지 않는다. 판정 기록도 HMAC으로 확인하고, 맞지 않으면 버리고 처음부터 판정한다. 다시 만들 때 OCR 입력은 언제나 동영상에서 새로 뽑은 장면이다([learning_adapters §5](../layers/06_learning_adapters.md#5-결과-파일)).
- 같은 키의 결과 파일이 이미 있을 때의 동작(덮어쓰기, 건너뛰기, 묻기)은 §6.

## 3. 예외

| 상황 | 판정 | application 오류 | 다음 행동(P5) |
|---|---|---|---|
| 부품이 없거나 해시가 다름(ffmpeg, 전사 모델, VAD 모델) | `ComponentStore` 확인 실패 | `ComponentMissing(Installer)` | 어떤 부품인지와 "설치 프로그램 다시 실행" 안내(S2-6) |
| ffmpeg 서명 요구 조건 불일치, 샌드박스 권한 없음 | 실행 직전 서명 확인 실패 | `ComponentMissing(Installer)` | 위와 같음 |
| Windows 한국어 OCR 언어 팩 없음 | `TextRecognizer` 지원 언어 없음 | `ComponentMissing(OsSettings)` | OS 설정 열기. Windows 학습 기능이 켜진 뒤에만 생긴다 |
| 동영상 크기·길이·해상도가 상한을 넘음 | 시작 전 확인 | `Rejected(LimitExceeded)` | 거절된 이유. 작업을 만들지 않는다 |
| 묶음 수·ffmpeg 실행 시간이 처리 중 상한을 넘음 | 처리 중 확인 | `Rejected(LimitExceeded)` | 작업 `failed`, 거절된 이유. 결과 파일 없음 |
| Windows에서 학습 기능 요청(격리 확인 전) | OS 관문 | 미정(§6) | 이 OS에서는 아직 쓸 수 없다는 안내. ffmpeg를 실행하지 않는다 |
| 학생이 취소 | `CancelToken` | — | 작업 `cancelled`. 결과 파일 없음 |
| 대상 동영상을 아직 받지 않음 | — | 미정(§6) | 먼저 내려받기(FS-12) |
| 동영상이 mp4가 아니거나 손상됨(재생목록 등 포함) | 디코딩 실패 | 미정(§6) | 작업 `failed`. 결과 파일 없음 |
| 결과 저장 실패(디스크 공간 등) | 원자적 쓰기 실패 | 미정(§6) | 작업 `failed`. 반쯤 만든 파일 없음 |
| 작업 폴더 판정 기록의 HMAC이 맞지 않음 | HMAC 확인 실패 | — | 학생에게 보이지 않음. 기록을 버리고 처음부터 판정 |

- 화면은 `ComponentMissing`의 복구 방법과 `Rejected`의 사유([application §4.5](../layers/02_application.md#45-application-오류))만 보고 안내를 고른다([UX Strategy §8](../06_ux_strategy.md#8-상태-오류-긴-작업)).

## 4. 수용 기준

| # | 기준 | 확인 방법 | 시나리오 |
|---|---|---|---|
| AC1 | 대화에서 슬라이드 PDF를 요청하면 `cdx`가 작업 ID를 바로 돌려주고, 그 대화에 작업 카드(상태 문구, 취소 버튼)가 생기며, 사이드바의 그 대화가 Working(경과 시간)이 된다 | 내려받은 동영상으로 요청한다 | S7-1 |
| AC2 | 작업이 `running`인 동안 다른 대화에서 질문하면 답이 나온다 | AC1 중에 다른 대화에서 질문한다 | S7-1 |
| AC3 | 작업이 끝나면 상태가 `done`이 되고, `<학습 자료 폴더>/<학기>/<과목>/슬라이드/`(스크립트는 `스크립트/`)에 이름 규칙대로 파일이 생기며, 그 대화에 결과 파일 위치가 보인다. 학생이 그 대화를 보지 않았으면 사이드바가 Done이 된다 | AC1을 끝까지 기다린다 | S7-1 |
| AC4 | 작업 카드에는 상태 문구와 취소 버튼만 있고 진행률(퍼센트, 단계)은 보이지 않는다 | AC1 중에 카드를 본다 | S7-1 |
| AC5 | 작업을 시작한 대화 ID는 IPC 토큰의 대화로 정해지고, 카드와 Working은 그 대화에만 보인다 | 대화 두 개에서 각각 작업을 시작한다 | S7-1 |
| AC6 | 작업 카드의 취소를 누르면 상태가 `cancelled`가 되고, 결과 폴더에 그 차시의 결과 파일·임시 파일이 없고, 무결성 기록도 없다. 디코딩 중, OCR 중, 전사 중 각각 확인한다 | 단계마다 취소하는 테스트(메모리 포트) + 실제 동영상 | S7-2 |
| AC7 | 전사 중 취소하면 전사가 끝나기를 기다리지 않고 멈춘다(중단 콜백) | `CancelToken`을 켠 `Transcriber` 테스트 | S7-2 |
| AC8 | 전사 모델 파일을 지우고 스크립트를 요청하면 작업이 생기지 않고, 대화 앱 메시지에 어떤 부품이 없는지와 "설치 프로그램 다시 실행"이 보인다 | 부품을 지운 상태에서 요청한다 | S2-6 |
| AC9 | ffmpeg 번들의 해시는 맞지만 샌드박스 권한이 없는(또는 다른 식별자로 서명한) ffmpeg를 두면 실행하지 않고 `ComponentMissing(Installer)`가 된다 | 서명을 바꾼 ffmpeg로 요청한다 | S2-6 |
| AC10 | 상한(크기·길이·해상도)을 넘는 동영상은 작업이 생기지 않고 `Rejected(LimitExceeded)` 안내가 나온다. 처리 중 묶음 수가 상한을 넘으면 작업이 `failed`가 되고 결과 파일이 없다 | 상한을 낮춘 테스트 + 큰 동영상 | — |
| AC11 | Windows 빌드에서 학습 기능을 요청하면 ffmpeg가 실행되지 않고 쓸 수 없다는 안내가 나온다. 데이터 조회·대화는 그대로 동작한다 | Windows 빌드에서 요청하고 프로세스 목록을 본다 | — |
| AC12 | `.mp4` 이름을 단 HLS 재생목록을 동영상으로 넣으면 작업이 `failed`가 되고, 로컬 파일 내용이 결과에 들어가지 않는다 | [security §5.3](../cross_cutting/01_security.md#53-ffmpeg-샌드박스-2026-11-09-macos-270-apple-m5)의 재현 입력으로 요청한다 | — |
| AC13 | 결과 저장 직후 앱 데이터에 해시와 HMAC 기록이 생기고, 결과 파일을 1바이트 바꾸면 조회 등급이 `Peer`가 된다 | 저장 뒤 파일을 바꾸고 `LearningOutputRepository`로 조회한다(메모리 `SecretStore`) | S7-1 |
| AC14 | 작업 폴더에는 판정 기록만 있고 장면 이미지가 없다. 판정 기록을 바꾸면 버리고 처음부터 판정한다 | 작업 뒤 작업 폴더를 보고, 기록을 바꿔 다시 만든다 | — |
| AC15 | 슬라이드 PDF에서 글자 검색·복사가 되고, 쪽 아래에 타임스탬프 띠, 책갈피 `페이지 n — mm:ss`가 있다 | 결과 PDF를 연다 ★ | S7-1 |
| AC16 | 스크립트의 모든 줄이 `[mm:ss] 문장` 형식이다 | 결과 스크립트를 검사한다 ★ | S7-1 |
| AC17 | 같은 작업을 화면과 `cdx` 상태 조회가 같은 상태로 보인다 | AC1 중과 끝난 뒤 `cdx`로 조회한다 | S7-1 |

★ 실제 학교 계정과 서버가 있어야 확인할 수 있다(실제 강의 동영상을 받아야 한다).

## 5. User Flow (UF-13)

이 기능의 [Design Process ①](../03_design_process.md)에서 그린다. 그릴 갈래: 작업 시작과 완료(F1·F3·F4), 시작 전 확인 실패(OS 관문·부품·상한, F2), 취소(F5), 처리 중 실패(상한·디코딩·저장).

## 6. 미확인 항목

- **작업을 시작하는 `cdx` 명령**: [entrypoints §9](../layers/04_entrypoints.md#9-미확인-항목)에서 `cdx` 명령 목록이 미정이다. 명령 이름, 동영상을 가리키는 인자(결과물 키, `content_id`, 경로), 작업 ID를 바로 돌려주는지(F1의 추론)를 정한다. [application §4.6](../layers/02_application.md#46-작업-오래-걸리는-유스케이스)은 "`cdx`로 시작했으면 IPC 토큰에서 코어가 정한다"고 써서 화면에서 시작하는 경로도 있는 것처럼 읽히지만, MVP 화면([UX Strategy §7.2](../06_ux_strategy.md#72-mvp-화면))에는 작업을 시작하는 화면이 없다. MVP의 시작 경로가 `cdx`뿐인지 확인한다
- **`cdx`로 취소**: [application §4.6](../layers/02_application.md#46-작업-오래-걸리는-유스케이스)은 `cdx`에 "작업 ID로 상태 조회"만 적는다. AI가 작업을 취소할 수 있게 할지 정한다
- **결과 위치를 보이는 곳**: S7-1은 "끝나면 … 대화에 결과 파일 위치가 보인다"고 하고, [UX Strategy §7.4](../06_ux_strategy.md#74-대화별-상태)는 "대화 안의 작업 카드에는 상태 문구와 취소 버튼만 둔다"고 한다. 위치를 카드의 상태 문구에 넣을지, 완료 때 앱 메시지를 남길지 정한다. 작업이 끝나도 대화 AI는 알지 못하므로(`cdx`는 이미 돌아왔다), AI가 위치를 말하는 방식은 쓸 수 없다(추론)
- **대상 동영상이 없을 때**: 받지 않은 차시를 요청하면 오류로 돌려줄지, 작업 안에서 FS-12의 내려받기를 먼저 할지 아키텍처에 없다. 오류로 돌려준다면 [application §4.5](../layers/02_application.md#45-application-오류)의 어느 오류인지 정한다
- **시작 전 상한 확인 방법**: 재생 길이·해상도를 시작 전에 알아야 `Rejected(LimitExceeded)`로 막을 수 있는데, [application §4.3](../layers/02_application.md#43-학습)의 `FrameDecoder`·`AudioDecoder`에는 동영상 정보를 읽는 동작이 없다. 포트에 더할지, 첫 디코딩 결과로 판정해 처리 중 상한으로 다룰지 정한다
- **실패의 application 오류**: 손상된 동영상(디코딩 실패), 결과 저장 실패가 [application §4.5](../layers/02_application.md#45-application-오류)의 어느 오류인지 정해져 있지 않다. 같은 표에 Windows 학습 기능 꺼짐(`NotAvailable`로 볼지)도 함께 정한다. Windows에서 `ToolGuide`에 학습 명령을 빼는지도 정한다
- **시작 전 확인 실패 때 작업을 만드는지**: F2는 작업을 만들지 않고 오류를 돌려준다고 추론했다. [learning_adapters §4](../layers/06_learning_adapters.md#4-부품)는 "앱이 작업을 시작할 때마다" 확인한다고만 쓴다. 작업을 만든 뒤 `failed`로 끝내는 쪽이면 사이드바가 Failed가 되므로 S2-6의 표시가 달라진다
- **같은 결과가 이미 있을 때**: 같은 키의 슬라이드 PDF·스크립트가 있을 때 덮어쓸지, 건너뛸지, 학생에게 물을지. 덮어쓰면 기존 무결성 기록도 바꾼다
- **동시 작업과 앱 종료**: 동시에 몇 개를 실행하는지(`queued`가 쓰이는 조건), 앱을 끄면 진행 중 작업과 `JobRegistry`의 상태가 어떻게 되는지(다시 시작, `cancelled`, `failed`)가 아키텍처에 없다
- **ffmpeg 자식 프로세스 취소**: `CancelToken`은 단계 사이와 전사에만 적혀 있다([application §4.6](../layers/02_application.md#46-작업-오래-걸리는-유스케이스), [learning_adapters §3.2](../layers/06_learning_adapters.md#32-전사)). 디코딩 중 취소 때 ffmpeg를 끝내는지(F5의 추론)를 `learning/media` 계약에 적는다
- **둘을 함께 요청할 때**: S7-1은 "슬라이드 PDF와 스크립트를 만든다"를 한 시나리오로 쓴다. 작업 두 개(카드 두 장)로 볼지(F1의 추론), 카드 하나로 묶을지 정한다
- **작업 폴더 판정 기록의 쓰임**: 다시 만들 때 판정 기록을 재사용해 판정 단계를 건너뛰는지, 취소·실패 때 판정 기록을 남기는지가 [learning_adapters §5](../layers/06_learning_adapters.md#5-결과-파일)에 없다
- **`failed` 뒤 다음 행동**: 처리 중 실패한 작업의 카드에 다시 시도 버튼을 둘지. [UX Strategy §7.4](../06_ux_strategy.md#74-대화별-상태)의 "상태 문구와 취소 버튼만"과 함께 정한다
