//! [application] application 오류, CancelToken, 공통 값 타입, BoxFuture
//!
//! 레이어와 의존 규칙: docs/02_system_architecture.md §3.1·§3.2

use std::future::Future;
use std::pin::Pin;

/// 비동기 포트 메서드가 돌려주는 future.
///
/// 포트는 `Arc<dyn Port>`로 주입하는데, trait의 `async fn`은 dyn으로 쓸 수 없다
/// (Rust 1.99.1 기준, build_ci §7). 그래서 포트 메서드는 `async fn` 대신 이 타입을
/// 돌려준다. proc-macro를 쓰지 않고, tokio에도 의존하지 않는다.
///
/// ```compile_fail,E0038
/// // 툴체인을 올려 이 예제가 컴파일되면 `async fn`으로 바꿀 수 있는지 다시 본다.
/// trait Port: Send + Sync {
///     async fn get(&self) -> u32;
/// }
/// fn inject(port: std::sync::Arc<dyn Port>) {}
/// ```
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;
