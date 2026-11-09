//! [bin] composition root
//!
//! 레이어와 의존 규칙: docs/02_system_architecture.md §3.1·§3.2

// 배포 빌드에서 Windows 콘솔 창을 띄우지 않는다
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

fn main() {
    tauri::Builder::default()
        .run(tauri::generate_context!())
        .expect("Tauri 앱 실행 실패");
}
