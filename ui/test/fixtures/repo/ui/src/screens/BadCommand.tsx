// 위반: screens → commands(store 액션을 거쳐야 한다)
import { listCourses } from "../commands/course.ts";
export const bad = listCourses;
