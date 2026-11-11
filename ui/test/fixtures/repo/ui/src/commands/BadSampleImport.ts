// 위반: commands/core.ts가 아닌 commands가 샘플 구현을 부른다
import { sampleListCourses } from "./sample/course.ts";
export const bad = sampleListCourses;
