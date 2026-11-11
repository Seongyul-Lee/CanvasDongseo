// 위반: components → stores. 별칭(~)으로 불러도 잡혀야 한다
import { courseStore } from "~/stores/course";
export const bad = courseStore;
