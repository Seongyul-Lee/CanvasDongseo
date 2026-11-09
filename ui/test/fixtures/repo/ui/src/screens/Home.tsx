// 허용: screens → components, stores, theme
import "../theme/app.css";
import { Card } from "../components/Card.tsx";
import { courseStore } from "../stores/course.ts";
export const Home = [Card, courseStore];
