//! 허용 표 — docs/02_system_architecture.md §3.2 "의존 규칙" 표를 코드로 옮긴 것(데이터만 둔다).
//!
//! - 이 표와 문서가 어긋나면 문서가 기준이다. 표를 바꿀 때는 문서와 이 파일을 함께 고친다.
//! - 대상은 **워크스페이스 crate 사이의** 의존이다. 외부 crate는 §3.2가 정한 `domain/*` 제한만 본다
//!   (그 밖의 외부 crate는 docs/cross_cutting/02_platform.md §1에서 정하고 이 검사는 보지 않는다).
//! - `data_access/*`는 §3.2가 내부 규칙으로 가리키는 docs/layers/data_access/README.md §2.6 표를 따른다.
//!   §3.2 행(`ports/school`, `kernel`, `domain/core`, `auth`만 `platform/keychain`)은 data_access 전체가 쓸 수 있는
//!   바깥 경계이고, §2.6이 crate별로 더 좁힌다. 아직 없는 `contracts/<묶음>`은 crate를 만들 때 행을 더한다.
//! - 각 행의 `doc`은 문서의 어느 행에서 왔는지 적는다. 위반 메시지에 그대로 나온다.
//! - 표에 행이 없는 crate는 위반이다. crate를 더하면 문서와 이 표에 행을 함께 더한다.

use super::check::{Allowed, External, Rule, XTASK};

use Allowed::{AnyExceptXtask, Crate, Prefix};

const KERNEL: Allowed = Crate("app-kernel");
const CORE: Allowed = Crate("domain-core");
/// `application/ports/*`
const PORTS: Allowed = Prefix("ports-");
/// `application/usecases/*`
const USECASES: Allowed = Prefix("usecases-");
/// `presentation/presenters/*`
const PRESENTERS: Allowed = Prefix("presenters-");
/// `platform/*`
const PLATFORM: Allowed = Prefix("platform-");
const DS_CORE: Allowed = Crate("data-access-ds-core");
const HTTP: Allowed = Crate("data-access-http");

/// §2.6 "Source 의존": Source는 `ds_core`, 자기가 구현하는 `contracts/<묶음>`, `http`에만 의존한다
const SOURCE: &[Allowed] = &[DS_CORE, HTTP];
const SOURCE_DOC: &str = "data_access §2.6 Source 의존";

/// §3.2 `ai/*`, `learning/*`
const AI: &[Allowed] = &[Crate("ports-ai"), KERNEL, CORE];
const LEARNING: &[Allowed] = &[Crate("ports-learning"), KERNEL, CORE];

/// §3.2 "그 밖의 `application/usecases/*`"
const OTHER_USECASES: &[Allowed] = &[KERNEL, CORE, PORTS];
const OTHER_USECASES_DOC: &str = "§3.2 그 밖의 application/usecases/*";

/// §3.2 `application/ports/*`: 포트 crate끼리 서로 의존하지 않는다
const PORT: &[Allowed] = &[KERNEL, CORE];
const PORT_DOC: &str = "§3.2 application/ports/*";

const fn rule(krate: &'static str, internal: &'static [Allowed], doc: &'static str) -> Rule {
    Rule {
        krate,
        internal,
        external: External::Any,
        doc,
    }
}

pub const RULES: &[Rule] = &[
    // domain — §3.2에서 외부 crate까지 정하는 유일한 레이어
    Rule {
        krate: "domain-core",
        internal: &[],
        external: External::Only(&["jiff"]),
        doc: "§3.2 domain/core",
    },
    Rule {
        krate: "domain-slides",
        internal: &[],
        external: External::Only(&[]),
        doc: "§3.2 domain/slides",
    },
    // application
    rule("app-kernel", &[], "§3.2 application/kernel"),
    rule("ports-school", PORT, PORT_DOC),
    rule("ports-ai", PORT, PORT_DOC),
    rule("ports-learning", PORT, PORT_DOC),
    rule("ports-common", PORT, PORT_DOC),
    rule(
        "usecases-learning",
        &[
            KERNEL,
            CORE,
            Crate("domain-slides"),
            PORTS,
            Crate("usecases-jobs"),
        ],
        "§3.2 application/usecases/learning",
    ),
    rule(
        "usecases-jobs",
        &[KERNEL, CORE, Crate("ports-common")],
        "§3.2 application/usecases/jobs",
    ),
    rule("usecases-school", OTHER_USECASES, OTHER_USECASES_DOC),
    rule("usecases-ai", OTHER_USECASES, OTHER_USECASES_DOC),
    // data_access — §3.2 data_access/* + data_access §2.6
    rule("data-access-ds-core", &[], "data_access §2.6 ds_core"),
    rule(
        "data-access-repositories",
        &[Crate("ports-school"), KERNEL, CORE, DS_CORE],
        "data_access §2.6 repositories",
    ),
    rule("data-access-http", &[DS_CORE], "data_access §2.6 http"),
    rule(
        "data-access-auth",
        &[
            HTTP,
            DS_CORE,
            Crate("ports-school"),
            KERNEL,
            Crate("platform-keychain"),
        ],
        "data_access §2.6 auth",
    ),
    rule("data-access-canvas", SOURCE, SOURCE_DOC),
    rule("data-access-learningx", SOURCE, SOURCE_DOC),
    rule("data-access-smart-attendance", SOURCE, SOURCE_DOC),
    rule("data-access-commons", SOURCE, SOURCE_DOC),
    rule("data-access-school-site", SOURCE, SOURCE_DOC),
    rule("data-access-student-portal", SOURCE, SOURCE_DOC),
    // ai, learning
    rule("ai-claude-code", AI, "§3.2 ai/*"),
    rule("learning-media", LEARNING, "§3.2 learning/*"),
    rule("learning-transcribe", LEARNING, "§3.2 learning/*"),
    rule("learning-pdf", LEARNING, "§3.2 learning/*"),
    rule("learning-macos", LEARNING, "§3.2 learning/*"),
    rule("learning-windows", LEARNING, "§3.2 learning/*"),
    // local_store — "구현하는 포트 crate"는 §3.1의 포트 배치를 따른다
    rule(
        "local-store-conversations",
        // ConversationRepository는 ports/ai에 있다
        &[Crate("ports-ai"), KERNEL, CORE, PLATFORM],
        "§3.2 local_store/*",
    ),
    rule(
        "local-store-learning",
        // LearningOutputRepository·ComponentStore는 ports/learning에 있다
        &[Crate("ports-learning"), KERNEL, CORE, PLATFORM],
        "§3.2 local_store/*",
    ),
    rule(
        "local-store-settings",
        // SettingsRepository는 ports/common에 있다
        &[Crate("ports-common"), KERNEL, CORE, PLATFORM],
        "§3.2 local_store/*",
    ),
    // platform — 기초 crate와 OS 연동 crate만
    rule("platform-keychain", &[], "§3.2 platform/*"),
    rule("platform-fs", &[], "§3.2 platform/*"),
    // presentation
    rule(
        "presenters-common",
        &[KERNEL],
        "§3.2 presentation/presenters/common",
    ),
    rule(
        "presenters-school",
        &[Crate("usecases-school"), Crate("presenters-common"), KERNEL],
        "§3.2 presentation/presenters/<묶음>",
    ),
    rule(
        "presenters-ai",
        &[Crate("usecases-ai"), Crate("presenters-common"), KERNEL],
        "§3.2 presentation/presenters/<묶음>",
    ),
    rule(
        "presenters-learning",
        &[
            Crate("usecases-learning"),
            Crate("presenters-common"),
            KERNEL,
        ],
        "§3.2 presentation/presenters/<묶음>",
    ),
    rule(
        "presenters-jobs",
        &[Crate("usecases-jobs"), Crate("presenters-common"), KERNEL],
        "§3.2 presentation/presenters/<묶음>",
    ),
    rule(
        "presenters-shell",
        &[
            Crate("usecases-school"),
            Crate("usecases-ai"),
            Crate("usecases-jobs"),
            Crate("presenters-common"),
            KERNEL,
        ],
        "§3.2 presentation/presenters/shell",
    ),
    rule(
        "tauri-commands",
        &[USECASES, PRESENTERS, Crate("ports-common"), KERNEL],
        "§3.2 presentation/tauri_commands",
    ),
    // entrypoints, ipc
    rule(
        "ipc-server",
        &[USECASES, Crate("ipc-wire"), KERNEL],
        "§3.2 entrypoints/ipc_server",
    ),
    rule("ipc-wire", &[], "§3.2 ipc/wire"),
    rule("ipc-cli", &[Crate("ipc-wire")], "§3.2 ipc/cli"),
    // bin
    rule("app", &[AnyExceptXtask], "§3.2 bin/app (composition root)"),
    rule(
        "cdx",
        &[Crate("ipc-wire"), Crate("ipc-cli")],
        "§3.2 bin/cdx",
    ),
    // xtask — 앱 crate에 의존하지 않는다. 앱 crate도 xtask에 의존하지 않는다(어느 행도 xtask를 허용하지 않는다)
    rule(XTASK, &[], "§3.2 xtask"),
];
