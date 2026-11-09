// 허용: 샘플 구현 → design/samples/, commands, viewmodels
import sample from "../../../../design/samples/course/default.json";
import type { listCourses } from "../course.ts";
import type { Course } from "../../viewmodels/course.ts";
export const sampleListCourses: typeof listCourses = async () => [sample as unknown as Course];
