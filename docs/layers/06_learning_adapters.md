# Learning Adapters

| 항목 | 내용 |
|---|---|
| 독자 | 개발자 본인, 후속 문서·구현을 돕는 AI 에이전트 |
| 근거 | [PRD §8.3](../01_prd.md), 2026-11-08~09 실측(Apple M5, LearningX 영상 5개 과목 15개, [측정 보고서](../measurements/2026-11_lecture_media_processing.md)), 2026-10-26 영상→PDF 실측(운영체제 2주차 1차시) |

> **이 문서의 역할**: 강의 동영상 → 슬라이드 PDF·스크립트 기능의 처리 방식과, 학습 포트([application §4.3](02_application.md))의 구현을 정한다. 확장 단계 기능이며 macOS를 먼저 구현한다.

---

## 1. 개요

| 항목 | 내용 |
|---|---|
| 링 | Frameworks & Drivers |
| crate | `learning/media`, `learning/transcribe`, `learning/pdf`(두 OS 공통), `learning/macos`, `learning/windows`(OCR만) |
| 의존할 수 있는 것 | `application/ports/learning`, `application/kernel`, `domain/*` (유스케이스 crate는 모른다) |

- **OS별로 나누는 것은 OCR뿐이다**(S9). 디코딩·전사·PDF는 두 OS가 같은 crate를 쓰고, 빌드 기능 플래그(가속)만 다르다.
- 슬라이드 판정 규칙(구간·묶음·점수)은 어댑터가 아니라 [domain §4.1](01_domain.md)에 있다. 앱이 신뢰하는 것은 앱의 로직뿐이므로, 판정은 외부 도구에 맡기지 않는다.
- composition root에서 `#[cfg(target_os)]`로 OCR 구현을 고른다. 구현이 없는 OS에서는 `NotAvailable`을 돌려준다.
- `bin/app`의 `learning` cargo feature(기본 켬)가 이 레이어의 crate 전부를 묶는다. 개발 중에는 꺼서 whisper.cpp·OS OCR 빌드를 건너뛸 수 있고, 배포 빌드는 항상 켠다([build_ci §3](../cross_cutting/03_build_ci.md#3-로컬-개발)).

## 2. 처리 흐름

```
동영상(.mp4, data_access의 MediaDownloadGateway로 받은 로컬 파일)
 ├─ 슬라이드 PDF (ExtractSlidePdf)
 │   ① FrameDecoder   축소 회색조 프레임 스트림(domain 상수)        learning/media
 │   ② domain         슬라이드 영역 검출 → 구간 → 재등장 묶음 → 후보 고르기(회색조로 충분)
 │   ③ FrameDecoder   후보 전부를 원본 해상도 **컬러**로 추출        learning/media
 │   ④ domain         후보 점수(완전성, 펜 색 = 채도 기반 원본성) → 자동 확정
 │   ⑤ TextRecognizer 확정 장면마다 OCR(한국어·영어)               learning/macos | windows
 │   ⑥ SlidePdfWriter 장면 + 타임스탬프 띠 + 보이지 않는 OCR 글자 층 + 책갈피   learning/pdf
 │   ⑦ LearningOutputRepository  domain 이름 규칙의 키로 원자적 저장        local_store/learning
 └─ 스크립트 (Transcribe)
     ① AudioDecoder   Transcriber가 알린 형식(whisper.cpp는 16kHz 모노)   learning/media
     ② Transcriber    whisper.cpp + VAD → [mm:ss] 문장 줄          learning/transcribe
     ③ LearningOutputRepository  스크립트 저장                       local_store/learning
```

- 원본성 점수는 채도(펜 색)를 보므로 회색조 스트림으로는 매길 수 없다. 점수와 확정은 반드시 ③의 컬러 후보로 한다. 후보는 묶음당 최대 12장이다([domain §4.1](01_domain.md)).
- **AI 검토 단계는 없다.** 묶음마다 domain 규칙으로 자동 확정한다. 2026-10-26 실측에서 검토한 13개 묶음 모두 자동 추천이 AI 판단과 같았고, 단계를 늘려 처리 시간을 늘리지 않는다.
- **학교 서버의 자막·대본은 쓰지 않는다.** 스크립트는 항상 앱이 전사한다([05_commons](data_access/05_commons.md)).
- 두 작업 모두 오래 걸리므로 작업(Job)으로 실행한다([application §4.6](02_application.md)).
- 디코딩 간격·폭(domain 상수)과 음성 형식(`Transcriber::required_audio_format()`)은 유스케이스가 포트에 넘긴다. 아래 수치는 그 값이 무엇인지 적은 것이지 포트 계약이 아니다.

## 3. 구현

| 포트 | 구현 | macOS | Windows |
|---|---|---|---|
| `FrameDecoder`, `AudioDecoder` | `learning/media`: ffmpeg 사이드카(LGPL 정적 빌드)를 자식 프로세스로 띄우고, 동영상은 stdin으로 넣고 결과는 stdout으로 읽는다 | App Sandbox 번들 | 격리 확인 전에는 학습 기능을 끈다(§6) |
| `Transcriber` | `learning/transcribe`: `whisper-rs`(whisper.cpp) | Metal | CPU (Vulkan은 착수 때 측정) |
| `TextRecognizer` | OS API | `learning/macos`: `objc2-vision`(Vision) | `learning/windows`: `windows` crate(Windows.Media.Ocr) |
| `SlidePdfWriter` | `learning/pdf`: Rust PDF crate(착수 때 선정, 한글 글꼴 포함·보이지 않는 글자 층 지원 필수). PDF 바이트만 만들고 경로는 정하지 않는다 | 같음 | 같음 |
| `LearningOutputRepository`, `ComponentStore` | `local_store/learning`(앱 로컬 파일) | 같음 | 같음 |

### 3.1 디코딩

- 신호 추출은 **모든 프레임을 디코딩**한다: `-vf fps=1,scale=320:-2`, `-pix_fmt gray`, rawvideo 파이프.
- **키프레임만 디코딩하지 않는다.** 실측: everlec 녹화(키프레임 1초 간격)는 데이터의 46~71%가 키프레임이라 약 10%만 줄었고, 그 밖의 녹화는 키프레임 간격이 7.7~13.3초라 슬라이드 전환을 놓친다.
- **하드웨어 디코딩(VideoToolbox 등)을 쓰지 않는다.** 실측에서 일반 디코딩보다 약 9배 느렸다(작은 프레임을 하나씩 내보내는 작업이라 복사 비용이 크다).
- 실측 시간(전부 디코딩, 26~35분 강의): everlec 1080p 15fps 11.4~18.1초, 그 밖의 녹화(720p 이하, 1080p 30fps 제작 영상 포함) 2.8~8.4초.
- 원본 해상도 장면 추출은 `-ss (i + 0.46)/fps -frames:v 1`로 한다. fps 필터의 i번째 프레임은 i+0.5초 직전 화면이라, i초로 찾으면 전환 직후에 앞 슬라이드를 뽑는다(2026-10-26 실측).

**신뢰할 수 없는 입력으로 다룬다** ([security R4](../cross_cutting/01_security.md))

학교 서버에서 받은 동영상은 신뢰하지 않는다. ffmpeg는 확장자가 아니라 내용으로 형식을 고르므로, `.mp4` 이름을 단 재생목록이 로컬 파일을 읽을 수 있다(2026-11-09 재현, [security §5.3](../cross_cutting/01_security.md)). 네 겹으로 막는다.

| 겹 | 규칙 |
|---|---|
| 형식 고정 | 입력은 항상 `-f mp4 -i fd:`. 형식 자동 판별을 쓰지 않는다. `pipe:`는 쓰지 않는다: 핸들이 파일이어도 탐색하지 않아 `moov`가 파일 끝에 있는 mp4가 실패하고 장면 탐색(`-ss`)도 안 된다(2026-11-09 실측). `fd:`는 같은 stdin 핸들을 탐색한다 |
| 빌드 축소 | `--disable-everything --disable-network --disable-autodetect` 위에 아래만 켠다(LGPL, 정적, 시스템 프레임워크에만 의존). protocol: `fd,pipe`(`file` 없음). demuxer: `mov,matroska,webm_dash_manifest,mp3,aac,wav`. decoder: `h264,hevc,vp8,vp9,av1,aac,aac_latm,mp3,mp3float,opus,vorbis,pcm_s16le`. parser: `h264,hevc,vp8,vp9,av1,aac,mpegaudio,opus,vorbis`. filter: `fps,scale,format,crop,null,anull,aresample,aformat,copy,trim,atrim,setpts`. encoder: `rawvideo,pcm_s16le`. muxer: `rawvideo,wav,null`. bsf: `h264_mp4toannexb,hevc_mp4toannexb`. **hls·concat 등 재생목록 demuxer, `file` 프로토콜, 네트워크는 빌드에 없다** |
| 입력 방식 | 앱이 동영상 파일을 **읽기 전용**으로 열어 그 핸들을 stdin(fd 0)으로 넘긴다. ffmpeg에 경로를 주지 않는다. 결과(회색조 프레임, rgb24 장면, wav 음성)는 stdout으로만 받는다. 자식에게는 stdin·stdout·stderr 세 개만 넘긴다: macOS는 다른 핸들을 모두 `CLOEXEC`로 열고, Windows는 `PROC_THREAD_ATTRIBUTE_HANDLE_LIST`로 상속 목록을 정한다. 샌드박스는 이미 넘겨받은 핸들을 막지 않으므로 쓰기 핸들·IPC 소켓이 새지 않게 하는 규칙이다 |
| 샌드박스 | macOS: ffmpeg를 `.app` 번들(`Contents/MacOS/ffmpeg` + 번들 식별자 `io.github.seongyul-lee.canvasdongseo.ffmpeg`)로 감싸 **App Sandbox 권한(파일 접근 권한 없음) + Hardened Runtime**으로 Developer ID 서명한다. 파일·네트워크·쓰기가 모두 거부되고 속도 차이는 없다(실측). Windows는 격리(AppContainer 등)가 수용 기준을 통과하기 전까지 **학습 기능(슬라이드 PDF·전사)을 켜지 않는다**(출시 관문, [security §6](../cross_cutting/01_security.md)) |

### 3.2 전사

| 항목 | 값 | 근거 |
|---|---|---|
| 엔진 | whisper.cpp (`whisper-rs`) | Rust에서 쓸 수 있고 두 OS가 같은 코드를 쓴다 |
| 모델 | `ggml-large-v3-turbo-q5_0.bin` (크기는 §4) | 원본(1.6GB)보다 3배 작고 빠르다. mlx-whisper와의 글자 차이 1.7~2.4%, 원본과 0.4~0.7%p 차이이며 대부분 띄어쓰기다 |
| VAD | Silero v5.1.2 (`ggml-silero-v5.1.2.bin`), **항상 켬** | VAD 없이 양자화 모델이 문장 묶음을 통째로 빠뜨린 경우가 있었다(법정교육 구간에서 q5_0의 글자 차이가 VAD 없이 3.8%, VAD를 켜면 2.3%) |
| 설정 | 언어 `ko`, 앞 문맥 미사용(`max_context = 0`) | 반복 환각을 막는다 |
| 출력 | 줄마다 `[mm:ss] 문장` | 학습 기능의 출처 표기(`3주차 강의 12:30`)에 쓴다 |
| 실측 속도 | M5 Metal: 10분 음성 약 20초. CPU만: 약 170초 | 30분 강의 기준 macOS 약 1분, CPU 약 8.5분 이상 |

- 전사 호출은 중간에 멈출 수 없으므로, whisper-rs의 중단 콜백(abort callback)으로 취소를 전달한다. 구간 단위 진행률을 작업 진행률로 보낸다.

### 3.3 OCR과 PDF

- OCR 결과의 글자 상자 위치에 **보이지 않는 글자**(render mode 3)를 깔아 PDF에서 검색·복사가 되게 한다.
- 쪽 = 장면(원본 해상도) + 아래쪽 여백 띠의 타임스탬프(`12:34 (다시 나옴 18:02)`). 쪽 순서는 처음 나온 시각 순이고, 책갈피는 `페이지 n — mm:ss`다.
- 문서 정보에 원본 동영상 제목, `content_id`, 만든 날짜를 넣는다.

### 3.4 상한 ([security R4](../cross_cutting/01_security.md))

초기값이며 실측 최대값의 여러 배로 잡았다. 넘으면 작업을 시작하지 않거나 끊고 `Rejected`로 알린다.

| 대상 | 상한 | 실측 최대(2026-11-09) |
|---|---|---|
| 동영상 파일 크기 | 4GB | 0.5GB(수강 과목 LearningX 목록의 최대) |
| 재생 길이 | 4시간 | 약 67분(같은 목록의 최대) |
| 해상도 | 3840×2160 | 1920×1080 |
| 묶음(슬라이드) 수 | 500 | 수십 개 |
| 묶음당 후보 | 12 ([domain §4.1](01_domain.md)) | 12 |
| ffmpeg 한 번 실행 시간 | 30분 | 18초 |

## 4. 부품

| 부품 | 크기 | 쓰는 곳 |
|---|---|---|
| ffmpeg (LGPL 정적 빌드, 앱이 쓰는 기능만 켬) | 약 6MB(2026-10-30 실측) | `learning/media` |
| `ggml-large-v3-turbo-q5_0.bin` | 574MB | `learning/transcribe` |
| `ggml-silero-v5.1.2.bin` | 0.9MB | `learning/transcribe` |

- 설치 패키지에 넣지 않고 설치 과정에서 자동으로 받는다. 받는 주체·위치·복구는 [platform §3](../cross_cutting/02_platform.md)에 있다.
- **모든 부품은 앱에 고정한 SHA-256으로 확인한 뒤에만 쓴다.** ffmpeg는 번들 안의 실행 파일(`Contents/MacOS/ffmpeg`, 서명과 권한이 이 파일에 들어 있다)을 해시한다.
- 실행 파일은 실행 직전에 서명도 확인한다. macOS: 서명 요구 조건 `identifier "io.github.seongyul-lee.canvasdongseo.ffmpeg" and anchor apple generic and certificate leaf[subject.OU] = "468HW42V9P"`를 만족하고, 권한에 `com.apple.security.app-sandbox`가 참으로 들어 있어야 한다(같은 팀이 서명한 샌드박스 없는 ffmpeg를 거부한다). Windows: Authenticode 서명과 게시자 확인.
- 확인 시점: 설치 프로그램이 받은 직후, 그리고 앱이 작업을 시작할 때마다(`ComponentStore`). 574MB 모델의 SHA-256 계산은 0.2~1.0초다(2026-11-09 M5 실측). **모델 파일**은 확인한 열린 핸들 그대로 쓰고 경로로 다시 열지 않는다(확인과 사용 사이 바꿔치기 방지). **실행 파일**은 핸들로 실행할 수 없어 경로로 실행하므로, 그 사이의 바꿔치기는 경로 권한에 기댄다: macOS는 root 전용 공용 경로, Windows는 ACL 수용 기준([platform §4](../cross_cutting/02_platform.md)).

| 파일 | SHA-256 |
|---|---|
| `ggml-large-v3-turbo-q5_0.bin` | `394221709cd5ad1f40c46e6031ca61bce88931e6e088c188294c6d5a55ffa7e2` |
| `ggml-silero-v5.1.2.bin` | `29940d98d42b91fbd05ce489f3ecf7c72f0a42f027e4875919a28fb4c04ea2cf` |
| ffmpeg (macOS arm64·x86_64, Windows x64) | 앱이 직접 만든 LGPL 정적 빌드를 배포 저장소에 올릴 때 OS·아키텍처별로 고정한다(미정, §6) |

모델 해시는 2026-11-09 Hugging Face에서 받은 파일의 값이다.

- 부품 위치 찾기와 해시 확인은 `ComponentStore` 한곳에서 하고, `learning/media`·`learning/transcribe`는 이를 주입받는다.
- 부품이 없거나 해시·서명이 다르면 `ComponentMissing { 부품, 복구: 설치 프로그램 }`을 돌려준다.
- Windows에서 한국어 OCR 언어 팩이 없으면 `TextRecognizer`가 `ComponentMissing { 부품: OCR 언어, 복구: OS 설정 }`으로 실패한다. 언어 팩은 설치 프로그램이 아니라 Windows 설정에서 설치하므로 복구 방법을 구분한다. 두 OS 구현은 같은 오류 형태로 실패한다(LSP). 오류 정의는 [application §4.5](02_application.md).

## 5. 결과 파일

| 항목 | 위치 |
|---|---|
| 학습 자료 폴더 | macOS `~/CanvasDongseo/`, Windows `%USERPROFILE%\CanvasDongseo\`. 앱이 정하고 MVP에서는 바꿀 수 없다. 내려받은 강의 자료·동영상([FS-12](../specs/FS-12_material_download.md))과 아래 학습 결과물이 모두 이 아래에 생긴다 |
| 슬라이드 PDF | `<학습 자료 폴더>/<학기>/<과목>/슬라이드/{NN}주차_{M}차시_{제목}.pdf` |
| 스크립트 | `<학습 자료 폴더>/<학기>/<과목>/스크립트/{NN}주차_{M}차시_{제목}.txt` |
| 작업 폴더(판정 기록) | `<앱 데이터>/work/<content_id>/`. 판정 기록(구간·묶음·확정 시각)만 둔다. **후보 장면 이미지는 남기지 않고, 다시 만들 때는 동영상에서 다시 뽑는다.** 바뀐 판정 기록은 HMAC이 맞지 않으면 버리고 처음부터 판정한다([security §4.1](../cross_cutting/01_security.md)) |

- 학습 자료 폴더를 홈 폴더 바로 아래에 두는 이유: macOS는 데스크톱·문서·다운로드 폴더를 개인정보 보호(TCC) 대상으로 두어 앱과 앱이 띄운 AI CLI가 접근할 때 권한 창이 뜬다. Windows는 OneDrive가 문서 폴더를 옮겨 가면(알려진 폴더 이동) 큰 동영상이 클라우드로 동기화될 수 있다. 홈 폴더 바로 아래는 둘 다 해당하지 않고, 학생이 Finder·탐색기에서 바로 찾을 수 있다.
- 이름 규칙과 짝 규칙은 [domain §4.2](01_domain.md)에 있다. 이 표는 `LearningOutputRepository` 구현이 키를 바꾸는 실제 위치다.

## 6. 미확인 항목

- 슬라이드 판정 기준값(전환 0.2, 같은 슬라이드 0.55, 펜 색 기준)은 운영체제 everlec 영상으로 조정한 값이다. 레이아웃이 다른 영상(AI기반창업마케팅, 제작 영상)으로 다시 확인한다.
- 전사 절대 정확도: 실측은 mlx-whisper와의 일치도다. 사람이 받아 적은 정답 구간으로 글자 오류율을 잰다.
- Windows: 실제 노트북의 CPU·Vulkan 전사 속도, Windows.Media.Ocr 한국어 품질과 언어 팩 설치 조건
- ffmpeg 배포 빌드의 OS·아키텍처별 SHA-256. 수용 기준: 배포 저장소 릴리스의 파일 해시를 이 문서 §4 표에 적고, 설치 프로그램이 그 값으로 확인한다
- Windows ffmpeg 격리(AppContainer 등). 수용 기준은 [security §7](../cross_cutting/01_security.md). 통과 전에는 Windows 학습 기능을 켜지 않는다
- `whisper-rs` 유지 상태(최근 갱신 2026-03), PDF crate 선정
- `objc2-vision` 바인딩 범위(텍스트 인식 요청 API 포함 여부)
