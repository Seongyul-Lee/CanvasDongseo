// 허용: 샘플 구현 → design/samples/, commands/types.ts, viewmodels
import sample from "../../../../design/samples/course/default.json";
import type { Load } from "../types.ts";
import type { Course } from "../../viewmodels/course.ts";
export const sampleListCourses: Load<Course[]> = async () => [sample as unknown as Course];
