# Commons (강의 자료·동영상 원본)

| 항목 | 내용 |
|---|---|
| 독자 | 개발자 본인, 후속 문서·구현을 돕는 AI 에이전트 |
| 근거 | 2026-11-08 확인(별도 표시가 없으면) |

> **이 문서의 역할**: LearningX의 `content_id`로 강의 PDF·동영상·첨부 원본을 내려받는 경로를 정의한다. Source `commons`.
> 구조(레이어, `<I>`·DS 배치, 공통 오류, HTTP 정책)는 [README](README.md)를 따른다.

---

## 1. 공통

- 인증 필요 없음(2026-10 확인). 목록(`content_id`)은 LearningX에서 받는다.
- `content_id` 다운로드 경로는 학교 측이 "학교 서버에 이 방식으로 접근하는 것이라면 공식적으로 허용한다"고 회신했다(2026-11-08, 사용자 구두 전달, 서면 사본 미확보, 범위는 `content_id` 다운로드로 한정. 서면 확보는 배포 전 과제, [PRD §9.2](../../01_prd.md)). 다만 공개 API가 아니므로 형식이 바뀌면 `SourceChanged`로 알린다.

## 2. 경로

| 단계 | 요청 | 결과 |
|---|---|---|
| 1 | `GET https://dcms.dongseo.ac.kr/viewer/ssplayer/uniplayer_support/content.php?content_id=<id>` | XML(최대 1MB, `DOCTYPE`·`ENTITY` 거부) |
| 2-PDF | XML의 `content_download_uri`를 `https://dcms.dongseo.ac.kr/` 기준으로 합친 주소 | `index.php?module=xn_media_content2013&act=dispXn_media_content2013DownloadWebFile&…` |
| 2-동영상 | `main_media_list@default_media_id`와 같은 `main_media`의 이름(예: `screen.mp4`)을 `media_uri[@method="progressive"][@target="all"]`의 `[MEDIA_FILE]` 자리에 넣은 주소 | 호스트 `dongseo-cms.edge.naverncp.com`. **`Referer: https://dcms.dongseo.ac.kr/` 필요**(없으면 403) |
| 2-일반 파일 | `content.php`가 "Not Supported Content Type"을 주는 수업자료실 파일 | `https://dcms.dongseo.ac.kr/index.php?module=xn_media_content2013&act=dispXn_media_content2013DownloadContent&content_id=<id>`. 확장자는 `Content-Disposition`에서 가져온다 |

## 3. 다운로드 동작

- 크기는 `HEAD`로 확인하고, 거부되면 `GET Range: bytes=0-0`으로 확인한다.
- 강한 ETag가 있으면 `Range`/`If-Range`로 이어 받는다. 서버가 Range를 무시하면(200) 처음부터 받는다.
- 동영상은 스트리밍으로 받고 메모리에 올리지 않는다. `.part`로 받은 뒤 크기를 확인하고 이름을 바꾼다.
- HTML 로그인 화면 같은 예상 밖 응답은 완료 파일로 만들지 않는다.

---

## 미확인 항목

- 동영상 자막·대본을 서버에서 받을 수 있는지
