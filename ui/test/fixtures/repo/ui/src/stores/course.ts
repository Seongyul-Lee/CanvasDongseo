// 허용: stores → commands, viewmodels
import { listCourses } from "../commands/course.ts";
import type { Course } from "../viewmodels/course.ts";
export const courseStore: { load: typeof listCourses; items: Course[] } = { load: listCourses, items: [] };
