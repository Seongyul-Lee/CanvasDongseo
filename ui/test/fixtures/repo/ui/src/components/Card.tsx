// 허용: components → viewmodels, theme, 외부 패키지
import "../theme/app.css";
import type { Course } from "../viewmodels/course.ts";
import { useId } from "react";
export const Card = (course: Course) => [course, useId];
