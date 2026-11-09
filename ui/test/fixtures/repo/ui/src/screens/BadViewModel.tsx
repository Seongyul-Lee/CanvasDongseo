// 위반: screens → viewmodels(가공하지 않고 store·component를 거친다)
import type { Course } from "../viewmodels/course.ts";
export const bad = (c: Course) => c;
