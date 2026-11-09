// 위반: 샘플 구현은 Tauri API를 부르지 않는다
import { invoke } from "@tauri-apps/api/core";
export const bad = invoke;
