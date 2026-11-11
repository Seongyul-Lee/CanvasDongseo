// 허용: commands/core.ts만 샘플 구현을 (샘플 모드에서만 실행되는 동적 import로) 부른다
import type { Course } from "../viewmodels/course.ts";
export const sampleCore = (): Promise<Course[]> => import("./sample/course.ts").then((m) => m.sampleListCourses());
