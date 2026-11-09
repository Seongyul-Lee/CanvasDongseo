# viewmodels/ — 생성 코드 자리, 손으로 고치지 않는다

이 폴더에는 Rust(`crates/presentation/presenters/*`)의 ViewModel·이벤트 타입에서 **생성한 TypeScript 타입만** 둔다([presentation §2.1](../../../docs/layers/03_presentation.md#21-프런트엔드-spa-내부-구조)).

- 파일을 손으로 만들거나 고치지 않는다. 타입이 달라져야 하면 Rust 타입을 고치고 다시 생성한다.
- 생성 도구(`ts-rs`·`specta` 등)는 아직 정하지 않았다([presentation §5](../../../docs/layers/03_presentation.md#5-미확인-항목)). 정해지면 생성 명령을 여기에 적는다.
- CI는 `presenters/*`가 바뀌면 타입을 다시 생성해 이 폴더와 차이가 없는지 확인한다([build_ci §5.2](../../../docs/cross_cutting/03_build_ci.md#52-영향-범위-계산-cargo-xtask-affected)).
- 이 폴더는 다른 `ui/src` 폴더에 의존하지 않는다(`eslint-plugin-boundaries`로 검사한다).
