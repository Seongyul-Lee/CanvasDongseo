// 위반: components → Tauri API
import { invoke } from "@tauri-apps/api/core";
export const bad = invoke;
