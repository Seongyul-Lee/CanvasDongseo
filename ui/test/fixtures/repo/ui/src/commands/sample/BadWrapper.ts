// 위반: 샘플 구현 → command 래퍼. commands/에서는 types.ts만 부를 수 있다
import { listCourses } from "../course.ts";
export const bad = listCourses;
