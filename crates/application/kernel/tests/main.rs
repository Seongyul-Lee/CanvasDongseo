//! 포트 trait가 proc-macro 없이 `Arc<dyn Port>`로 주입되는지 확인한다(build_ci §4·§7).
//!
//! 레이어: application/kernel의 통합 테스트. 여기의 `SamplePort`는 테스트에만 있고
//! 실제 포트는 `application/ports/*`에 둔다.

use std::future::Future;
use std::pin::pin;
use std::sync::Arc;
use std::task::{Context, Poll, Waker};

use app_kernel::BoxFuture;

trait SamplePort: Send + Sync {
    fn get(&self, id: u32) -> BoxFuture<'_, String>;
}

struct SampleAdapter {
    prefix: String,
}

impl SamplePort for SampleAdapter {
    fn get(&self, id: u32) -> BoxFuture<'_, String> {
        Box::pin(async move { format!("{}{id}", self.prefix) })
    }
}

struct UseCase {
    port: Arc<dyn SamplePort>,
}

impl UseCase {
    async fn run(&self, id: u32) -> String {
        self.port.get(id).await
    }
}

/// tokio 없이 한 번 폴링한다. 테스트의 future는 기다리지 않고 바로 끝난다.
fn poll_once<F: Future>(future: F) -> F::Output {
    let mut cx = Context::from_waker(Waker::noop());
    match pin!(future).poll(&mut cx) {
        Poll::Ready(value) => value,
        Poll::Pending => panic!("테스트 future가 바로 끝나지 않았다"),
    }
}

#[test]
fn port_returning_box_future_is_injected_as_arc_dyn() {
    let port: Arc<dyn SamplePort> = Arc::new(SampleAdapter {
        prefix: "course-".into(),
    });
    let use_case = UseCase { port };

    assert_eq!(poll_once(use_case.run(7)), "course-7");
}

fn assert_send<T: Send>(_: &T) {}

#[test]
fn box_future_from_dyn_port_is_send() {
    let port: Arc<dyn SamplePort> = Arc::new(SampleAdapter {
        prefix: String::new(),
    });
    let future = port.get(1);
    assert_send(&future);
}
