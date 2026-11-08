# 학생지원시스템 (학사행정웹서비스)

| 항목 | 내용 |
|---|---|
| 독자 | 개발자 본인, 후속 문서·구현을 돕는 AI 에이전트 |
| 상태 | 작성됨. 메뉴별 응답 필드는 일부만 확인 |
| 근거 | 2026-11-08 로그인한 학생 세션으로 확인. 화면 소스(서비스 선언)를 읽고, 조회 화면 몇 개만 열어 요청·응답 형식을 기록했다. 쓰기 서비스는 실행하지 않았다 |

> **이 문서의 역할**: 학사행정웹서비스(`im.dongseo.ac.kr`)의 로그인과 데이터 조회 경로를 정의한다. Component `student_portal`.
> 구조(레이어, `<I>`·DS 배치, 공통 오류, HTTP 정책)는 [README](README.md)를 따른다.

---

## 1. 개요

- 주소: `https://im.dongseo.ac.kr` (학생 화면은 `/indexSSO2.do?locale=ko`)
- Canvas·LearningX와 **다른 시스템이고 로그인도 따로**다. 아이디·비밀번호는 같다(SSO 로그인 화면 안내: "로그인 ID/PW는 학사행정웹서비스 계정과 동일").
- 학적, 수업(출결·휴보강·시험시간표·수업계획서), 수강, 성적, 장학, 등록금, 채플, 국제프로그램 데이터가 있다. 메뉴 63개 중 학생 메뉴 그룹 10개.
- **학생의 학과·전공을 여기서 얻을 수 있다**(`SELECT_USER_INFO`의 `DEPT_NM`, `MAJOR_NM`). 학과 공지 레지스트리([06_school_site §3](06_school_site.md))와 연결하는 데 쓴다.

## 2. 로그인과 세션

| 단계 | 요청 | 결과 |
|---|---|---|
| 1 | `GET /indexSSO2.do?locale=ko` | 로그인 화면(`loginForm`, 숨은 필드 `agentId=4`, `id`, `pw`) |
| 2 | `POST /beforeSsoLogin.do` (form: `agentId`, `id`, `pw`, `V_USER_ID`, `V_USER_PWD`) | JSON. `SUCCESS_YN=Y`면 다음 단계. 실패면 `ERROR_CODE`(`LOGIN_001` 등), `LOGIN_FAIL_CNT`(실패 횟수), `LOGIN_LAST_DT` |
| 3 | form을 `POST https://dsso.dongseo.ac.kr/authentication/idpw/loginProcess` (`id`, `pw`, `agentId=4`) | SSO 처리 뒤 `im.dongseo.ac.kr`로 돌아와 세션 쿠키 발급 |

- 별도 SSO 서버 `dsso.dongseo.ac.kr`을 쓴다. Canvas의 SSO(`eclass1 …/xn-sso`)와 다르다.
- **2차 인증(OTP)이 없다.** 화면의 `CONFIRM_OTP`는 비밀번호 찾기의 휴대폰 인증번호다.
- 실패 횟수를 서버가 센다. 잘못된 비밀번호로 반복 시도하지 않는다(잠김 기준은 확인하지 못함).
- 세션은 60분이다. `POST /sessionExtend.do` → `{"SESSION_TIME":3600,"SUCCESS_YN":"Y"}`로 연장한다. 화면 오른쪽 위 "남은 시간·연장"과 같다.
- 3단계 이후의 리디렉션·쿠키 이름은 이미 로그인된 브라우저로 조사해서 확인하지 못했다(미확인 항목).
- 인증은 `data_access/shared`에 학사행정용 세션 관리를 따로 둔다([01_auth](01_auth.md)의 Canvas 세션과 별개). 학번·비밀번호는 Canvas 로그인과 같은 저장 값을 쓴다.

## 3. 데이터 조회 방식

모든 데이터는 범용 창구 하나로 조회한다.

### 3.1 메뉴 목록

`POST /menuLoad.do` (form: `S_MENU_CD=100002&MENU_NM=100002&locale=ko`) → `MENU_LIST[]`

| 필드 | 뜻 |
|---|---|
| `MENU_CD`, `MENU_NM`, `MENU_LEVEL`(3 = 그룹, 4 = 화면), `UP_MENU_CD` | 메뉴 트리 |
| `RUN_URL` | `loadPage.do` |
| `RUN_PARAM` | `jspPage=/stud/A/{그룹}/{화면}` |

### 3.2 화면과 서비스 선언

- 화면: `GET /loadPage.do?jspPage=/stud/A/E/AE020` (HTML)
- 화면 스크립트가 쓸 서비스를 이렇게 선언한다:

```js
var QP = new RequestPacket("sqlSession");
QP.addService("SELECT_GRADE_INFO", '<쿼리 토큰>', "SELECT");
QP.getService(QP.length()-1).setParam("USER_ID", userId);
selectService(QP.paramJson(), callback);
```

- **쿼리 토큰**은 서버가 만든 암호화 문자열이고 화면 HTML에 박혀 있다. 같은 서비스는 여러 번 불러도 같은 토큰이었다(정적). 토큰은 화면 HTML에서 읽어 와야 하며 앱에 고정하지 않는다(학교가 바꾸면 깨진다).
- 서비스 종류: `SELECT`, `SELECTMAP`, `CLOBSELECT`(조회), `INSERT`, `UPDATE`, `DELETE`(쓰기).

### 3.3 조회 요청

`POST /select.do`, 헤더 `AJAX: true`, `X-Requested-With: XMLHttpRequest`, `Content-Type: application/x-www-form-urlencoded`

| form 키 (URL 인코딩 전) | 값 |
|---|---|
| `_SQL_SESSION_NAME_` | `sqlSession` |
| `{순번}0000:SERVICE:{서비스명}:{종류}:false:true` | 쿼리 토큰 |
| `{순번}{매개변수 순번}:PARAM:{서비스명}:true:{매개변수명}` | 매개변수 값 |

- 순번은 `10010000`, `10020000`, …(서비스), `10013001`, `10013002`, …(첫 서비스의 매개변수)처럼 늘어난다. 여러 서비스를 한 요청에 묶을 수 있다.
- 응답: `{"SUCCESS_YN":"Y","ERROR_MESSAGE":"", "{서비스명}":[{…행…}], …}`. 서비스마다 결과 배열이 서비스명 키로 온다.
- 다른 창구: `POST /apiProcess.do`(JSON `{"QUERY":"<토큰>","QUERY_GBN":"S","USER_NO":"<학번>"}` → `{"list":[…],"isSuccess":true}`), `POST /menuLoadNm.do`. 첫 화면 알림·설문 확인에 쓰인다.

**예: 로그인 사용자 정보**

| 서비스 | 매개변수 | 응답 필드 |
|---|---|---|
| `SELECT_USER_INFO` | `S_USER_NO`, `LOCALE` | `USER_NO`, `USER_NM`, `SHREG_NM`(학적 상태), `DEPT_NM`(대학), `MAJOR_NM`(전공), `USER_GB` |
| `SELECT_STD_INFO` (거의 모든 화면) | `USER_ID` | 학번, 이름, 학년, 학적 상태, 전공·부전공·복수전공, 지도교수 |
| `SELECT_MY_TUTOR` | — | 지도교수 이름·연구실·전화·메일 |

### 3.4 서버 측 결함: 매개변수 학번 미검증 (IDOR)

> **치명적 결함 — 학교 시스템 쪽 문제. 2026-11-08 확인.**

- `/select.do`는 요청 매개변수의 학번(`USER_ID`, `S_USER_NO` 등)을 **로그인 세션과 대조하지 않고 그대로 DB 조회에 쓴다.**
- 확인 근거:
  - (이 세션) 본인 계정으로 없는 형식의 학번 `00000000`을 넣어 `SELECT_USER_INFO`를 조회하니, 본인 데이터로 덮이지도 거부되지도 않고 **빈 결과**가 왔다. 세션이 학번을 덮어쓰지 않는다는 뜻이다.
  - (사용자 별도 확인, 친구 동의 하에) 본인 로그인 세션에서 과목출결조회 요청의 학번을 타인 학번(`(타 학생 학번)`)으로 바꿔 보내자 **그 타인의 출결이 정상 응답**했다.
- 즉 로그인한 학생은 **임의 학번의 다른 학생 개인정보(학적·출결·성적 등)를 조회할 수 있다.** IDOR(Insecure Direct Object Reference)다.
- **학교 측에 별도로 보고한다**(사용자 진행). 앱은 이 결함을 이용하지 않는다. 실존 타인 학번으로 추가 확인을 하지 않는다(개인정보 침해).

### 3.5 앱이 지킬 것

- **매개변수의 학번은 앱이 주입하지 않는다.** DS·유스케이스·MCP 도구는 학번을 입력 매개변수로 받지 않는다. `student_portal` Component 내부에서, 요청을 만들 때 **로그인 세션의 학번으로만** 채운다.
- **외부에서 넘어온 학번은 덮어쓴다.** AI(MCP 도구)나 상위 계층이 학번을 넘겨도 무시하고 세션 학번으로 교체한다. 경계에서 이 교체를 강제한다.
- **응답 검증.** 조회 응답의 `USER_NO`/`USER_ID`가 로그인 학번과 다르면 폐기하고 오류로 처리한다. 혹시 모를 혼선·캐시·학교 측 동작 변경에 대한 2차 방어선이다.
- 같은 규칙을 스마트출결([04_smart_attendance](04_smart_attendance.md), `user_login`을 쿼리에 넣음)에도 적용한다.
- **쓰기 서비스(`INSERT`·`UPDATE`·`DELETE`)는 부르지 않는다.** 휴학·복학·전공 신청, 강의평가, 학점포기, P/N 신청, 등록금 분할납부, 봉사일지 등은 학사에 영향을 준다([README §1.3](README.md)). DS는 조회 서비스만 구현한다.
- 출력·증명서(`/report/reqReport.jsp`, 외부 증명서 발급 `uni.webminwon.com`)는 대상이 아니다.

## 4. 메뉴별 서비스 목록

2026-11-08 학생 메뉴 화면 소스에서 `addService` 선언을 읽어 만든 표다(서비스를 실행해서 얻은 것이 아니다). 모든 화면이 부르는 `SELECT_STD_INFO [USER_ID]`는 뺐다. 매개변수는 화면이 `setParam`으로 넣는 이름이며, 연도·학기 등 값의 형식은 화면마다 확인한다.

- 조회 서비스가 12개를 넘는 화면은 앞 12개만 적었다(전체는 화면 소스에 있다).
- "쓰기 서비스" 열은 앱이 부르지 않는 서비스다(§3.5).
- 일부 화면은 같은 이름(`LIST`)의 서비스를 다른 매개변수로 여러 번 선언한다. 서비스는 이름이 아니라 (화면, 토큰)으로 구별된다.

| 메뉴 그룹 | 메뉴 | 화면(`jspPage`) | 조회 서비스 [매개변수] | 쓰기 서비스 |
|---|---|---|---|---|
| 채플 | 채플출석조회 | `/stud/A/A/AA010` | `SELECT_YEAR_HAKGI`<br>`SELECT_HAKGI_COMBO`<br>`LIST` [SUG010_YEAR, SUG010_SMT, USER_ID]<br>`SELECT_GAMJUM_SUM` [SUG010_YEAR, SUG010_SMT, USER_ID] |  |
| 채플 | 채플설문조사 | `/stud/A/A/AA020` | `SELECT_YEAR_HAKGI`<br>`SELECT_LESSON_STD_CHK` [SUG010_YEAR, SUG010_SMT, USER_ID]<br>`SELECT_SUBJECT_INFO` [SUE100_YEAR, SUE100_SMT, SUE100_CURI_NUM, SUE100_COURSE_CLS]<br>`SELECT_TERM_CHK` [SUD070_YEAR, SUD070_SMT]<br>`SELECT_SURVEY_CHK` [S_YEAR, S_SMT, S_CURI_NUM, S_COURSE_CLS, S_SDIV, USER_ID]<br>`SELECT_QUESTION_INFO` [S_YEAR, S_SMT, S_SDIV, S_CURI_NUM, S_COURSE_CLS, USER_ID]<br>`SELECT_ANSWER_INFO` [S_YEAR, S_SMT, S_SDIV] | `LONG_UPATE` |
| 채플 | 채플신입생설문조사 | `/stud/A/A/AA030` | `SELECT_YEAR_HAKGI`<br>`SELECT_LESSON_STD_CHK` [SUG010_YEAR, SUG010_SMT, USER_ID]<br>`SELECT_TERM_CHK` [SUD070_YEAR, SUD070_SMT]<br>`SELECT_SURVEY_CHK` [S_YEAR, S_SMT, S_CURI_NUM, S_COURSE_CLS, S_SDIV, USER_ID]<br>`SELECT_QUESTION_INFO` [S_YEAR, S_SMT, S_SDIV, S_CURI_NUM, S_COURSE_CLS, USER_ID]<br>`SELECT_ANSWER_INFO` [S_YEAR, S_SMT, S_SDIV] | `LONG_UPATE` |
| 학적 | 학생기본정보 | `/stud/A/B/AB010` | `SELECT_STD_INFO_CHG` [USER_ID]<br>`SELECT_INFO` [USER_ID]<br>`SELECT_INFO` [USER_ID, SUD200_GRAD_DIV] (SELECTMAP)<br>`LIST` [USER_ID] | `INSERT`, `UPDATE` |
| 학적 | 휴학신청 | `/stud/A/B/AB030` | `SELECT_HYU_DT`<br>`SELECT_BOOK_CNT` [USER_ID]<br>`SELECT_CONSULTING_NO` [USER_ID]<br>`SELECT_HYU_CHK_DATA_INFO` [SUD070_YEAR, SUD070_SMT, USER_ID]<br>`LIST` [USER_ID]<br>`SELECT_STD_PROF_LIST` [HAK_YEAR, HAK_SMT, ABS010_PROF_CD, USER_ID, ABS010_STUDENT_NM, ABS010_SCHOOL_STS] |  |
| 학적 | 복학신청 | `/stud/A/B/AB040` | `SELECT_RESCHOOL_SMT`<br>`SELECT_STD_CHG_DT_TODAY` [USER_ID]<br>`SELECT_RESCHOOL_DATA` [USER_ID]<br>`SELECT_HYU_APLY_DT` [USER_ID]<br>`SELECT_WEB_RESCHOOL_APLY_TERM_CHK` [SUD070_YEAR, SUD070_SMT]<br>`SELECT_STANDARD_RESCHOOL_TERM` [SUD070_YEAR, SUD070_SMT]<br>`SELECT_MILITARY_RESCHOOL_TERM` [SUD070_YEAR, SUD070_SMT]<br>`SELECT_HYU_YEAR_QUARTER` [SUD070_YEAR, SUD070_SMT]<br>`SELECT_SMT_ISU_CHK` [SUH030_STUDENT_YEAR, USER_ID, SUH030_SMT] | `UPDATE` |
| 학적 | 이수지정과목조회 | `/stud/A/B/AB060` | `LIST` [USER_ID] |  |
| 학적 | 전공/부,복수,융합전공/전과 신청 | `/stud/A/B/AB050_M` | `SELECT_UP_CD` [SYS010_DPT_CD]<br>`UPSERT_MAJOR_MERGE01` [USER_ID, SUD150_MAJOR_CD, SUD150_GBN, SUD150_YEAR, SUD150_SMT]<br>`UPSERT_MAJOR_MERGE02` [USER_ID, SUD150_MAJOR_CD, SUD150_GBN, SUD150_YEAR, SUD150_SMT]<br>`UPSERT_MAJOR_MERGE03` [USER_ID, SUD150_MAJOR_CD, SUD150_GBN, SUD150_YEAR, SUD150_SMT]<br>`UPSERT_MAJOR_MERGE04` [USER_ID, SUD150_MAJOR_CD, SUD150_GBN, SUD150_YEAR, SUD150_SMT]<br>`UPSERT_MAJOR_MERGE` [USER_ID, SUD190_AFT_MAJOR_CD, SUD190_BEF_MAJOR_CD, SUD190_GBN, SUD190_YEAR, SUD190_SMT]<br>`SELECT_LOG` [USER_ID]<br>`SELECT_COMBO_BOX` [USER_ID]<br>`SELECT_TERM_CHK`<br>`SELECT_RETURN_SCHOOL_STD` [USER_ID]<br>`SELECT_SCORE_CHK` [USER_ID]<br>`SELECT_MAJOR_ASSIGN_APLY_TIME` [SUD130_YEAR, USER_ID]<br>외 15개 | `UPDATE` |
| 학적 | 국제학생증신청 | `/stud/A/B/AB090` | `SELECT_STD_CD` [USER_ID] |  |
| 수업 | 교과과정조회 | `/stud/A/C/AC010` | `SELECT_DEPT_CD` |  |
| 수업 | 대체과목조회 | `/stud/A/C/AC020` | `LIST` [USER_ID] |  |
| 수업 | 동일과목조회 | `/stud/A/C/AC030` | `S_YYSHYTM`<br>`LIST` [S_YEAR, S_SHTM, S_SBJT] |  |
| 수업 | 개설강좌(수업계획서)조회 | `/stud/A/C/AC040` | `S_YYSHYTM`<br>`SELECT_DEPT_CD`<br>`SELECT_COMDIV_CD` |  |
| 수업 | 강의평가점수조회 | `/stud/A/C/AC050` | `S_YYSHYTM`<br>`SELECT_DEPT_CD`<br>`SELECT_COMDIV_CD` |  |
| 수업 | 휴보강조회 | `/stud/A/C/AC100` | `S_YYSHYTM`<br>`LIST` [S_YEAR, S_SHTM, USER_ID] |  |
| 수업 | 과목출결조회 | `/stud/A/C/AC110` | `S_YYSHYTM`<br>`LIST` [S_YEAR, S_SHTM, USER_ID]<br>`SELECT_STATS` [S_YEAR, S_SHTM, CURI_NUM, COURSE_CLS, USER_ID] |  |
| 수업 | 유고결석신청 | `/stud/A/C/AC120` | `S_YYSHTM` [OPEN_YN]<br>`S_YYSHTM_CURI`<br>`LIST` [S_YEAR, S_SHTM, USER_ID] |  |
| 수업 | 시험시간표조회 | `/stud/A/C/AC080` | `SELECT_TEST_TIME_TABLE_CHK`<br>`SELECT_PRE_EVAL_CHK` [S_YEAR, S_SHTM, USER_ID]<br>`SELECT_TEST_TIME_TABLE` [S_YEAR, S_SHTM, S_EXAM_TP, USER_ID] |  |
| 수업 | 강의실시간표조회 | `/stud/A/C/AC060` | `SELECT_BLD_CD`<br>`S_YYSHYTM`<br>`SELECT_LECT_INFO` [S_YEAR, S_SHTM, S_BLD_CD, S_ROOM_CD]<br>`SELECT_ROOM_CD` [S_BLD_CD] |  |
| 수업 | 교수시간표조회 | `/stud/A/C/AC070` | `S_YYSHYTM`<br>`LIST` [S_PROF_NM]<br>`SELECT_LECT_INFO` [S_PROF_ID, S_YEAR, S_SHTM] |  |
| 수업 | 글쓰기역량측정평가 | `/stud/A/C/AC130` | `SELECT_OBJ_CHK` [USER_ID]<br>`SELECT_INS_CHK` [USER_ID]<br>`SELECT_EVAL_INFO` | `INSERT` |
| 수강 | 예비수강신청조회 | `/stud/A/D/AD015` | `S_YYSHYTM`<br>`LIST` [S_YEAR, S_SMT, USER_ID]<br>`SELECT_APPLY_PNY` [S_YEAR, S_SMT, USER_ID] |  |
| 수강 | 수강신청조회 | `/stud/A/D/AD020` | `S_YYSHYTM`<br>`LIST` [S_YEAR, S_SMT, USER_ID]<br>`SELECT_APPLY_PNY` [S_YEAR, S_SMT, USER_ID] |  |
| 수강 | 시간표조회 | `/stud/A/D/AD010` | `S_YYSHYTM`<br>`SELECT_LECT_INFO` [S_YEAR, S_SHTM, USER_ID] |  |
| 수강 | 선택형 PN평가과목신청 | `/stud/A/D/AD030` | `SELECT_YYSHTM`<br>`SELECT_AVAILABLE_PNT`<br>`SELECT_AVAILABLE_DT`<br>`SELECT_APLY_P_TOTAL` [USER_ID] | `DELETE_HISTORY`, `DELETE`, `INSERT_HISTORY`, `INSERT` |
| 수강 | 중간강의평가 | `/stud/A/D/AD040` | `SELECT_EVAL_DT`<br>`SELECT_EVAL_OBJ` [S_YEAR, S_SHTM, USER_ID]<br>`LIST` [S_YEAR, S_SHTM, USER_ID]<br>`SELECT_BASIC` [USER_ID]<br>`SELECT_BASIC_QT` [S_YEAR, S_SHTM, CURI_NUM, COURSE_CLS]<br>`SELECT_EVAL_CHK` [S_YEAR, S_SHTM, CURI_NUM, COURSE_CLS, USER_ID]<br>`SELECT_EVAL_INFO` [S_YEAR, S_SHTM, CURI_NUM, COURSE_CLS, USER_ID] | `INSERT` |
| 수강 | 강의평가(기말,계절,프론티어) | `/stud/A/D/AD050` | `SELECT_EVAL_DT`<br>`SELECT_EVAL_OBJ` [S_YEAR, S_SHTM, USER_ID]<br>`LIST` [S_YEAR, S_SHTM, USER_ID]<br>`SELECT_BASIC` [USER_ID]<br>`SELECT_BASIC_QT` [S_YEAR, S_SHTM, CURI_NUM, COURSE_CLS]<br>`SELECT_EVAL_CHK` [S_YEAR, S_SHTM, CURI_NUM, COURSE_CLS, USER_ID]<br>`SELECT_EVAL_INFO` [S_YEAR, S_SHTM, CURI_NUM, COURSE_CLS, USER_ID] | `INSERT` |
| 성적 | 중간성적 조회 | `/stud/A/E/AE011` | `S_YY_SHTM` [USER_ID]<br>`SELECT_TERM_CHK`<br>`SELECT_TERM_CNT`<br>`SELECT_SUM_INFO` [USER_ID]<br>`LIST` [S_YEAR, S_SHTM, USER_ID] |  |
| 성적 | 확정전 성적조회 | `/stud/A/E/AE010` | `S_YY_SHTM` [USER_ID]<br>`SELECT_TERM_DATE`<br>`SELECT_TERM_CHK`<br>`SELECT_TERM_CNT`<br>`SELECT_SUM_INFO` [USER_ID] |  |
| 성적 | 성적조회 | `/stud/A/E/AE020` | `S_YY` [USER_ID]<br>`SELECT_GRADE_INFO` [USER_ID]<br>`SELECT_COMDIV_CD`<br>`SELECT_OPEN_POP_CHK`<br>`SELECT_EVAL_NOTICE_YN` [USER_ID]<br>`LIST` [USER_ID]<br>`S_WARN_DISPLAY` [S_YEAR, S_SHTM, USER_ID]<br>`S_SHTM` [USER_ID, S_YEAR] |  |
| 성적 | 전과전성적조회 | `/stud/A/E/AE030` | `S_CHANGE_OBJ` [USER_ID]<br>`SELECT_GRADE_INFO` [USER_ID]<br>`S_YY` [USER_ID]<br>`LIST` [S_YEAR, S_SHTM, USER_ID]<br>`LIST` [S_YEAR, S_SHTM, USER_ID, ALL_YN]<br>`LIST` [USER_ID] |  |
| 성적 | 학점포기신청 | `/stud/A/E/AE050` | `SELECT_GIVE_UP_DT`<br>`LIST` [USER_ID]<br>`SELECT_GIVE_UP_APPLY_CHK` [CURI_NUM, USER_ID]<br>`SELECT_GIVE_UP_APPLY_CHK_YY_SHTM` [USER_ID]<br>`SELECT_GIVE_UP_APPLY_CHK_ALL` [USER_ID] | `DELETE`, `INSERT` |
| 성적 | EST점수(예체능계열) | `/stud/A/E/AE060` | `SELECT_DEPT_CD`<br>`LIST` [USER_ID] |  |
| 성적 | CBT점수 | `/stud/A/E/AE070` | `SELECT_DEPT_CD`<br>`LIST` [USER_ID] |  |
| 성적 | 졸업시뮬레이션 | `/stud/A/E/AE040` | `SELECT_YY_SHTM`<br>`SELECT_TAB_3` [USER_ID, DIV_GR]<br>`SELECT_STD_DETAIL` [S_YEAR, S_SHTM, USER_ID]<br>`SELECT_GRADE_CHK` [USER_ID]<br>`SELECT_CURI_CHK` [S_YEAR, S_SHTM, USER_ID]<br>`SELECT_RC_CHK` [S_YEAR, S_SHTM, USER_ID, GBN_DIV]<br>`SELECT_MAJOR_INFO` [USER_ID]<br>`SELECT_GRADE_FAIL_REASON` [S_YEAR, S_SHTM, USER_ID]<br>`SELECT_GRADE_TOTAL_POINT_TABLE` [S_YEAR, S_SHTM, USER_ID]<br>`SELECT_GRADE_ISU_PNT` [S_YEAR, S_SHTM, SUG_CHECK, USER_ID]<br>`SELECT_GRADE_ISU_PNT_DETAIL` [S_YEAR, S_SHTM, APPLY_YEAR, SUG_CHECK, USER_ID]<br>`SELECT_TAB_LIST` [USER_ID]<br>외 7개 |  |
| 장학 | 주요장학안내 | `/stud/A/F/AF010` | — |  |
| 장학 | 장학사항조회 | `/stud/A/F/AF020` | `SELECT_SCHOLAR_OPEN_YN`<br>`LIST2` [USER_ID]<br>`LIST` [USER_ID] |  |
| 장학 | 형제·자매 , 교육자·교역자 자녀 장학금 | `/stud/A/F/AF030` | `SELECT_DATE_CHK`<br>`SELECT_APPLY_LIST` [USER_ID]<br>`SELECT_SCORE` [USER_ID]<br>`SELECT_SCHOLAR_LIST` [USER_ID]<br>`SELECT_APPLY_LIST2` [USER_ID] | `INSERT`, `UPDATE` |
| 장학 | 한마음장학 | `/stud/A/F/AF040` | `SELECT_APPLY_CHECK` [USER_ID]<br>`SELECT_APPLY_TERM_CHECK`<br>`SELECT_SCORE` [USER_ID] |  |
| 장학 | 봉사일지 | `/stud/A/F/AF050` | `SELECT_TODAY`<br>`SELECT_STD` [R_YEAR, R_SMT, USER_ID]<br>`SELECT_TOT_TIME` [R_YEAR, R_SMT, R_MONTH, USER_ID] (SELECTMAP)<br>`SELECT_DATE` [R_YEAR, R_SMT, R_MONTH, USER_ID]<br>`SELECT_INS_DT_CHK` [V_YEAR, V_MONTH, DAYCOMBO]<br>`SELECT_LEC_TABLE_CHECK` [R_YEAR, R_SMT, R_MONTH, DAYCOMBO, FORM_ST_HH, FORM_ED_HH, USER_ID]<br>`SELECT_RE_TOT_TIME` [R_YEAR, R_SMT, R_MONTH, USER_ID] (SELECTMAP) | `DELETE` |
| 등록 | 등록금납부조회 | `/stud/A/G/AG010` | `LIST` [USER_ID] |  |
| 등록 | 등록금고지서출력 | `/stud/A/G/AG030` | `SELECT_PUB_CHECK` [USER_ID] (SELECTMAP)<br>`SELECT_DIVISION_CHECK` [USER_ID] (SELECTMAP)<br>`SELECT_PRINT_TERM` (SELECTMAP)<br>`SELECT_PRINT_CHECK` [USER_ID] (SELECTMAP)<br>`SELECT_FULL_SCH` [USER_ID]<br>`SELECT_FLYWIRE_CHECK` [USER_ID] (SELECTMAP)<br>`SELECT_FLYWIRE_DT_CHECK` (SELECTMAP)<br>`SELECT_PAY_DT_CHK`<br>`SELECT_PAY_FULL_CHK` [USER_ID] | `INSERT` |
| 등록 | 등록금분할납부신청 | `/stud/A/G/AG020` | `SELECT_INS_CHK`<br>`SELECT_OBJ_CHK` [USER_ID]<br>`SELECT_DECIDE_CHK` [USER_ID]<br>`SELECT_APLY_INFO` [USER_ID]<br>`SELECT_FEE_DT` [HALF_NUM]<br>`SELECT_REGI_DT` [USER_ID] | `DELETE`, `INSERT`, `UPDATE` |
| 등록 | 분할납부고지서출력 | `/stud/A/G/AG050` | `SELECT_PRINT` [USER_ID] (SELECTMAP)<br>`SELECT_PAYMENT` [USER_ID] (SELECTMAP) |  |
| 등록 | 계절학기고지서출력 | `/stud/A/G/AG040` | `SELECT_STUDENT_INFO` [USER_ID]<br>`SELECT_MONEY` [USER_ID]<br>`SELECT_TERM_CHECK` (SELECTMAP)<br>`SELECT_TERM_PLACE`<br>`LIST` [USER_ID] |  |
| 등록 | 0학점등록신청 | `/stud/A/G/AG070` | `SELECT_STUDENT_INFO` [USER_ID]<br>`SELECT_TERM_CHECK`<br>`SELECT_INSERT_CHECK` [USER_ID]<br>`SELECT_APPLY_TERM`<br>`SELECT_APPLY_CHECK` [USER_ID] | `DELETE`, `INSERT` |
| 등록 | 교육비 납입증명서 | `/stud/A/G/AG090` | `LIST` [USER_ID]<br>`INS_CERT_NO` [USER_ID, YY] |  |
| 취업 | 졸업생 특화 프로그램 신청 | `/stud/A/K/AK010` | `SELECT_STD` [USER_ID] (SELECTMAP) | `UPDATE` |
| 국제프로그램 | 해외교류지원서신청 | `/stud/A/I/AI010` | `SELECT_STD_INFO_CHG` [USER_ID]<br>`SELECT_PROGRAM` [USER_ID]<br>`SELECT_CHECK_CNT` [USER_ID]<br>`SELECT_LISENSE_CHECK_FIR` [USER_ID, SEQ, LISENSE_DIV]<br>`SELECT_LISENSE_CHECK_SEC` [USER_ID, SEQ, LISENSE_DIV]<br>`SELECT_LISENSE_CHECK_THR` [USER_ID, SEQ, LISENSE_DIV]<br>`SELECT_LANGUAGE_FIR` [USER_ID, SEQ, LISENSE_DIV]<br>`SELECT_LANGUAGE_SEC` [USER_ID, SEQ, LISENSE_DIV]<br>`SELECT_LANGUAGE_THR` [USER_ID, SEQ, LISENSE_DIV]<br>`SELECT_LEVEL_01_01` [USER_ID, SEQ, FOREIGN_DIV, EVALU_DIV]<br>`SELECT_LEVEL_01_02` [USER_ID, SEQ, FOREIGN_DIV, EVALU_DIV]<br>`SELECT_LEVEL_01_03` [USER_ID, SEQ, FOREIGN_DIV, EVALU_DIV]<br>외 9개 | `DELETE_FIRST_ONE`, `DELETE_FIRST_THREE`, `DELETE_FIRST_TWO`, `DELETE_SECOND_ONE`, `DELETE_SECOND_THREE`, `DELETE_SECOND_TWO`, `DELETE_THIRD_ONE`, `DELETE_THIRD_THREE`, `DELETE_THIRD_TWO`, `INSERT_FIRST_ONE`, `INSERT_FIRST_THREE`, `INSERT_FIRST_TWO`, `INSERT_LANGUAGE_FIR`, `INSERT_LANGUAGE_SEC`, `INSERT_LANGUAGE_THR`, `INSERT_LISENSE_FIR`, `INSERT_LISENSE_SEC`, `INSERT_LISENSE_THR`, `INSERT_SECOND_THREE`, `INSERT_SECOND_TWO`, `INSERT_THIRD_ONE`, `UPDATE_FIRST_ONE`, `UPDATE_FIRST_THREE`, `UPDATE_FIRST_TWO`, `UPDATE_FIR`, `UPDATE_LANGUAGE_FIR`, `UPDATE_LANGUAGE_SEC`, `UPDATE_LANGUAGE_THR`, `UPDATE_SECOND_ONE`, `UPDATE_SECOND_THREE`, `UPDATE_SECOND_TWO`, `UPDATE_SEC`, `UPDATE_THR`, `UPDATE` |
| 국제프로그램 | 과목이수계획서신청 | `/stud/A/I/AI020` | `SELECT_OBJ_CHK` [USER_ID]<br>`SELECT_DT_CHK` | `DELETE`, `UPDATE` |
| 국제프로그램 | 학점인정신청 | `/stud/A/I/AI030` | `SELECT_OBJ_CHK` [USER_ID]<br>`SELECT_DT_CHK` [PGM_CD]<br>`SELECT_SURVEY_CHK` [PGM_CD, USER_ID]<br>`SELECT_OBJ_INFO` [M_YEAR, M_SHTM, PGM_CD, USER_ID] |  |
| 학생서비스 | 예비군전산신고 | `/stud/A/H/AH020` | `SELECT_DT_CHK`<br>`SELECT_AM_COMM_CD_ASD03` [LARGE_DIV]<br>`SELECT_AM_COMM_CD_ASD04` [LARGE_DIV]<br>`SELECT_STD` [USER_ID]<br>`SELECT_ARMY_INFO` [USER_ID] | `INSERT` |
| 학생서비스 | 설문조사 | `/stud/A/J/AJ030` | `SELECT_YYSHTM`<br>`SELECT_SURV_LIST` [STD_NO, SURV_DEPT_CD] (CLOBSELECT) |  |

**앱에서 쓸 만한 조회 화면** (기능 설계 때 확정)

| 데이터 | 화면 | 비고 |
|---|---|---|
| 학적·전공·지도교수 | 학생기본정보 `AB010`, 사용자 정보 `SELECT_USER_INFO` | 학과 공지 연결 |
| 시간표 | 시간표조회 `AD010` (`SELECT_LECT_INFO`) | |
| 수강 과목 | 수강신청조회 `AD020` | |
| 과목별 출결(최종) | 과목출결조회 `AC110` (`LIST`, `SELECT_STATS`) | 전자출결은 다음 날 여기로 이관된다([04_smart_attendance](04_smart_attendance.md)) |
| 휴보강 | 휴보강조회 `AC100` | |
| 시험 시간표 | 시험시간표조회 `AC080` (`SELECT_TEST_TIME_TABLE`) | 기간 외에는 빈 결과 |
| 수업계획서 | 개설강좌(수업계획서)조회 `AC040` | |
| 성적 | 중간성적 `AE011`, 확정전 성적 `AE010`, 성적조회 `AE020` | |
| 졸업 요건 | 졸업시뮬레이션 `AE040` | 탭별 서비스 다수 |
| 장학 | 장학사항조회 `AF020` | |
| 등록금 | 등록금납부조회 `AG010` | |
| 채플 출석 | 채플출석조회 `AA010` | |

---

## 미확인 항목

- dsso 로그인(3단계) 뒤의 리디렉션 경로와 세션 쿠키 이름. 로그아웃 상태에서 한 번 기록해야 한다
- 로그인 실패 잠김 기준(몇 회, 얼마 동안)
- 쿼리 토큰이 바뀌는 조건(배포·세션·날짜). 지금은 화면을 열 때마다 HTML에서 읽는 방식으로 대비한다
- 화면별 목록 서비스(`LIST` 등)의 응답 필드. 화면을 열고 조회해야 나오는 것들은 일부만 확인했다
- 세션 연장(`sessionExtend.do`)만으로 60분 넘게 유지되는지, 최대 유지 시간
