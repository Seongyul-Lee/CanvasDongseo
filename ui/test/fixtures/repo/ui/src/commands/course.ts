// 허용: commands → viewmodels, Tauri API
import { invoke } from "@tauri-apps/api/core";
import type { Course } from "../viewmodels/course.ts";
export const listCourses = (): Promise<Course[]> => invoke("list_courses");
