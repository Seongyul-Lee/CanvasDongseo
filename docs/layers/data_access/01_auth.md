# 인증과 세션 (SSO · Canvas 웹 세션 · LearningX 토큰)

| 항목 | 내용 |
|---|---|
| 독자 | 개발자 본인, 후속 문서·구현을 돕는 AI 에이전트 |
| 근거 | 2026-11-08 확인(별도 표시가 없으면) |

> **이 문서의 역할**: 학교 계정으로 로그인하고 세션을 유지·갱신·종료하는 경로를 정의한다. Source가 아니라 `data_access/auth`의 인증 구성이 쓴다.
> 구조(레이어, `<I>`·DS 배치, 공통 오류, HTTP 정책)는 [README](README.md)를 따른다.

---

## 1. 자격 증명

| 이름 | 쓰는 곳 | 수명 | 저장 |
|---|---|---|---|
| 학번·비밀번호 | SSO 로그인 | — | 키체인 |
| OTP 6자리 | LearningX 2단계 인증 | 30초 | 저장하지 않음. 사용자가 입력 |
| `pseudonym_credentials` 쿠키 | Canvas 웹 세션(로그인 유지), Canvas API | 14일. 갱신할 때마다 다시 14일. **쓸 때마다 값이 바뀜** | 키체인 |
| `_csrf_token` 쿠키 | Canvas 쓰기 요청(`X-CSRF-Token`), 로그인 폼 | 세션 | 메모리 |
| `xn_api_token` (JWT) | LearningX API (`Authorization: Bearer`) | 2시간(`exp`) | 키체인 |

## 2. 최초 로그인 (OTP 필요)

2026-10-25에 확인한 흐름이다. OTP 화면 상태가 언제 만료되는지 모르므로, OTP를 받은 직후 한 번의 시도 안에서 1~6을 이어서 처리한다.

| 단계 | 요청 | 결과 |
|---|---|---|
| 1 | `GET https://canvas.dongseo.ac.kr/login` | eclass1 SSO로 이동, `xn_sso_csrf_token_for_this_login` 쿠키 |
| 2 | `POST https://eclass1.dongseo.ac.kr/xn-sso/gw-cb.php?from=web_redirect&login_type=standalone&return_url=https://canvas.dongseo.ac.kr/learningx/login`<br>form: `csrf_token`, `login_mode=idpw`, `login_user_id`, `login_user_password` | OTP 화면(`/learningx/google2fa/callback`). 화면의 `data-user_login`, `data-result`를 읽는다. 없을 때의 판정은 §2.1 |
| 3 | `POST /learningx/api/v1/google2fa/authenticate`<br>multipart: `user_login`, `one_time_password`, 헤더 `Authorization: Bearer null` | 204 성공 / 401 코드 틀림(`fail_count`, `retry_limit`) / 403 잠김(`remaining_seconds`) |
| 4 | `GET /learningx/google2fa/login/callback?result=<R>` | `xn_api_token` 쿠키. 화면에 Canvas 로그인 폼과 RSA로 암호화된 Canvas 비밀번호. `data-result`는 이미 percent-encoded이므로 한 번만 인코딩해 보낸다 |
| 5 | 암호문을 화면의 개인키로 복호화(PKCS#1 v1.5) | 서버가 만든 Canvas 비밀번호(32자) |
| 6 | `POST /login/canvas`<br>form: `utf8`, `authenticity_token`(`_csrf_token`), `pseudonym_session[unique_id]`, `pseudonym_session[password]`, `pseudonym_session[remember_me]=1` | `pseudonym_credentials`(14일) |

### 2.1 로그인 수단과 OTP 미등록 계정

2026-11-08 SSO 로그인 페이지(`eclass1 …/xn-sso/login.php`)와 학교 매뉴얼(`eclass1 …/pages/68b00cabee9cbf984906331a`, `…/68b00cb0effc7c7c76004756`)을 확인한 결과다.

- 로그인 수단은 두 가지다: **OTP 인증**(학번·비밀번호 → Google OTP, `login_mode=idpw`)과 **공인인증서**(학번 → KICA 인증서 서명, `login_mode=pki`). 화면 하단의 "관리자 로그인"(`/login/canvas`)은 관리자용이다.
- 매뉴얼상 OTP는 **필수 2차 인증**이다. 1차 로그인 뒤 OTP를 등록하지 않은 계정에는 OTP 입력 대신 **"OTP 등록하기" 화면**(QR 코드, 수동 입력용 설정 키, `등록 완료하기` 단추, 등록한 앱의 6자리 입력)이 나온다. 등록은 최초 1회다.
- OTP를 다시 등록하려면(5회 이상 인증 실패, 휴대폰 분실·변경, 앱 삭제) 학사행정웹서비스에서 이메일을 Gmail로 바꾸고, 다음 날 1차 로그인 → `OTP 재등록` → 메일 인증코드 순서로 진행한다.
- **2차 인증 없이 로그인되는 학생 계정은 매뉴얼에서 찾지 못했다.** 예외 계정이 정말 없는지는 확인하지 못했다(아래 미확인 항목).

**1차 로그인(단계 2) 응답 판정**

OTP 화면 속성(`data-user_login`, `data-result`)이 없다고 모두 "비밀번호 거부"로 처리하면, OTP 미등록 학생은 비밀번호가 맞아도 "틀렸다"는 안내를 받고 로그인할 수 없다. 그래서 응답을 아래처럼 나눈다.

| 응답 화면 | 판정 | 앱 동작 |
|---|---|---|
| OTP 입력 화면: `…/js/Google2FA/Authentication/authentication.app.js`를 불러온다 | `OtpRequired` | `#root`의 `data-user_login`, `data-result`를 읽고 OTP 6자리를 받아 단계 3으로 |
| OTP 등록 화면: `…/js/Google2FA/Registration/registration.app.js`를 불러온다 | `OtpNotRegistered` | 앱 안에서 등록하지 않는다. 학교 매뉴얼 링크와 함께 브라우저에서 한 번 등록하도록 안내하고, 등록 후 다시 로그인하게 한다 |
| 2차 인증 없이 Canvas 세션이 생김 (`pseudonym_credentials` 발급, 단계 4~6에 해당하는 화면) | `NoSecondFactor` | 단계 3을 건너뛰고 이어서 진행한다 (존재 여부 미확인, 대비만 한다) |
| SSO 로그인 화면으로 다시 옴 + 오류 문구 | `PasswordRejected` | 학번·비밀번호 확인 안내 |
| 그 밖의 화면 | `SourceChanged("password_submit")` | 비밀번호 거부로 단정하지 않는다 |

- **두 화면의 스크립트는 `#root`의 `data-user_login`과 `data-result`를 똑같이 읽는다.** 등록 화면에도 이 속성이 있다는 뜻이므로, 두 속성이 있다는 것만으로 OTP 입력 화면이라고 판정하면 OTP 미등록 학생에게 OTP를 물어보게 된다. 어느 화면인지는 불러오는 스크립트로 가른다(아래 §2.2). 인증 화면이 실제로 그 스크립트를 불러오는지는 다음 로그인 실측 때 화면 HTML로 확인한다.
- 앱이 OTP 등록을 대신하지 않는 이유: 등록 화면의 설정 키는 OTP를 만들어 낼 수 있는 비밀이다. 앱이 다루면 2차 인증의 목적(본인 기기 인증, 대리출석 방지)을 해친다.
- 공인인증서만 쓰는 학생은 지원하지 않는다. 인증서 서명은 KICA 앱·PC 모듈이 필요하다. 이 학생들에게는 OTP 등록을 안내한다(앱 설계에서 확정).
- 등록 화면의 판정 조건은 화면 스크립트를 읽어 정했다(§2.2). OTP를 등록하지 않은 계정으로 단계 2의 실제 응답(주소, HTML)을 받아 보지는 못했다(아래 미확인 항목).

### 2.2 OTP 등록 화면과 인증 화면의 동작 (2026-11-12, 화면 스크립트 분석)

LearningX가 공개로 내려 주는 화면 스크립트를 읽어 확인했다. 번들 목록은 `/learningx/mix-manifest.json`에 있고, 두 화면의 번들이 따로 있다. 스크립트를 실행하거나 OTP 미등록 계정으로 로그인해 보지는 않았다. 서버가 어느 화면을 내려 주는지는 서버의 판단이라 스크립트로는 알 수 없다.

| 항목 | OTP 인증 화면 (등록한 계정) | OTP 등록 화면 (등록하지 않은 계정) |
|---|---|---|
| 번들 | `/learningx/js/Google2FA/Authentication/authentication.app.js` | `/learningx/js/Google2FA/Registration/registration.app.js` |
| `#root`에서 읽는 속성 | `data-user_login`, `data-user_email`, `data-result`, `data-catalog_url`, `data-from`, `data-goback` | `data-user_login`, `data-user_email`, `data-result`, `data-catalog_url`, `data-from`, `data-re` |
| 첫 화면 | OTP 6자리 입력("2단계 인증") | "2단계 인증을 위해서는 Google OTP 등록이 필요합니다."와 `Google OTP 등록` 단추 |
| 화면이 열릴 때 부르는 것 | `GET …/google2fa/guide`(안내문) | `POST …/google2fa/register`(`user_login`) → `QR_Image`, `secret_key`. 이어서 `GET …/google2fa/guide` |

경로는 모두 `/learningx/api/v1` 아래다.

- **OTP 미등록 계정은 바로 로그인되지 않는다.** 두 화면 모두 세션을 만드는 다음 단계(`/learningx/google2fa/login/callback?result=…`, §2의 단계 4)로 넘어가는 곳이 두 군데뿐이다. `POST …/google2fa/authenticate`가 204를 줄 때와 `POST …/google2fa/mail/verify`가 204를 줄 때다. 2차 인증을 건너뛰고 넘어가는 경로는 스크립트에 없다.
- **등록 화면의 순서**: QR 코드나 설정 키를 OTP 앱에 등록 → `등록 완료하기`(`POST …/google2fa/register_complete`, `user_login`·`secret_key`) → 같은 화면에서 OTP 6자리 입력(`POST …/google2fa/authenticate`) → 단계 4. 등록만 하고 끝나지 않고 그 자리에서 인증까지 해야 로그인된다.
- **메일 인증이 있다.** 두 화면 모두 "스마트폰이 없는 경우에는 임시로 메일 인증을 통해 로그인할 수 있습니다."라며 메일로 일회성 인증키를 보내고(`POST …/google2fa/mail/send`, `user_login`·`user_email`) 확인한다(`POST …/google2fa/mail/verify`, `user_login`·`auth_key`). 204면 단계 4로 간다. 실패는 OTP와 같이 401(틀림)·403(잠김)이고 `remained_time`, `retry_limit`, `current_fail_count`를 준다.
- **OTP 재등록**: 인증 화면에서 메일 인증키로 `POST …/google2fa/mail/verify/rereg`가 204를 주면 등록 정보가 지워지고 `/learningx/login?result=…&re=true`로 간다. `data-re`가 있는 등록 화면은 안내 화면을 건너뛰고 등록 단계부터 시작한다.
- 안내문(`GET …/google2fa/guide`)의 제목은 "대리출석 방지를 위한 2단계 인증 의무화 안내"이고, "OTP 등록하기 (처음 1회)" 뒤에 "로그인 시 2단계 인증 이용하기"로 적는다.
- 앱은 `…/google2fa/register`를 부르지 않는다. 부르면 설정 키가 만들어진다(등록을 대신하지 않는 이유는 §2.1).

## 3. 세션 갱신 (OTP 없이)

| 단계 | 요청 | 결과 |
|---|---|---|
| 1 | 쿠키만 담아 `GET /courses/{course_id}/external_tools/{주차학습 도구 번호}` | 정상이면 `id="tool_form"` 폼(LTI 1.1). 로그인 화면으로 가면 `SessionExpired` |
| 2 | 응답의 새 `pseudonym_credentials`를 **다른 일보다 먼저** 저장 | 이전 값을 다시 쓰면 바로 로그인 화면으로 튕긴다 |
| 3 | `tool_form` 필드를 그대로 `POST /learningx/lti/modulebuilder` | 새 `xn_api_token` |

- 주차학습 도구 번호는 고정하지 않고 [02_canvas §3](02_canvas.md)의 과목 탭 API에서 찾는다(확인한 과목에서는 `70`).
- 갱신 시점(기본값): 사용 중에는 토큰 만료 5분 전, 유휴 상태에서는 마지막 갱신 뒤 24시간, 앱 시작·잠자기에서 깨어날 때 바로 확인. 쿠키 만료가 2일 안이면 미리 안내한다.
- 로그아웃 상태에서는 어떤 자동 동작도 세션을 되살리지 않는다. OTP 로그인 성공으로만 벗어난다.

## 4. 로그아웃

- 쿠키로 `GET /` → `_csrf_token`을 받은 뒤 `POST /logout`(form: `_method=delete`, `authenticity_token`). 서버에서 로그인 유지 쿠키도 무효가 된다(2026-10-26 확인).
- 로컬에서 지워도 서버 쪽 `xn_api_token`은 `exp`(최대 2시간)까지 유효하다. 서버에서 폐기할 방법은 찾지 못했다.

## 5. 방식 A로 바꿀 때 (배포 전 검토)

| 항목 | 내용 |
|---|---|
| 필요한 것 | 학교 관리자가 발급한 Developer Key(client_id, client_secret), redirect URI |
| 흐름 | `GET /login/oauth2/auth?client_id&response_type=code&state&redirect_uri` → code → `POST /login/oauth2/token`(`grant_type=authorization_code`) |
| 갱신 | access token 1시간. `POST /login/oauth2/token`(`grant_type=refresh_token`). refresh token은 바뀌지 않고 계속 쓴다 |
| 로그아웃 | `DELETE /login/oauth2/token` |
| 401 구분 | `WWW-Authenticate` 헤더가 있으면 토큰 만료·삭제, 없으면 권한 없음 |
| 남는 문제 | LearningX는 여전히 웹 세션과 `xn_api_token`이 필요하다. (A)로 바꿔도 §2~3.3은 LearningX용으로 남는다 |
| 쓰지 않는 것 | 공식 모바일 앱(`iOS Student`, `AndroidStudent`)의 client_id 재사용(사칭), deprecated된 InstAccess token(`POST /api/v1/inst_access_tokens`) |

---

## 위험과 미확인 항목

### 위험

| 위험 | 대응 |
|---|---|
| 쿠키가 쓸 때마다 바뀜 → 동시 갱신 시 세션을 잃음 | 세션을 다루는 주체를 앱 코어 하나로 둔다(README D6). 그 안에서 갱신을 하나만 돌린다 |
| 바뀐 쿠키 저장 실패 → 세션을 잃음 | 저장 재시도 설계 필요 |
| 서버 쪽 `xn_api_token`을 폐기할 수 없음 | 로그아웃 뒤 최대 2시간 유효. 남는 위험으로 둔다 |

### 미확인

- 첫 OTP 로그인 뒤 14일이 지나도 갱신이 계속되는지(서버 쪽 절대 기한)
- 방학처럼 활성 과목이 없을 때 LTI 갱신이 되는지(지난 학기 과목으로 되는지)
- OTP 미등록 계정이 단계 2에서 받는 실제 응답(주소, HTML). 판정 조건은 화면 스크립트로 정했고(§2.2), 실제 계정으로 받아 보지는 못했다. 등록 화면이 인증 화면과 같은 주소(`/learningx/google2fa/callback`)로 오는지도 모른다
- 2차 인증 없이 로그인되는 학생 계정이 있는지. 화면 스크립트에는 그런 경로가 없다(§2.2). 서버가 예외 계정에 다른 화면을 주는지는 확인하지 못했다
- 공인인증서만 쓰는 학생이 학번·비밀번호로 로그인했을 때 OTP 등록 화면이 나오는지. 그런 계정을 확보하지 못했다
- 메일 인증(§2.2)을 앱이 지원할지. 학교가 스마트폰이 없는 학생에게 주는 임시 수단이고, 지금 앱은 OTP 입력만 받는다
- 학교 관리자에게 Developer Key를 받을 수 있는지(README D2)
