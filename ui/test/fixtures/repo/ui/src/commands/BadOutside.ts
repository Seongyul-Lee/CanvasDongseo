// 위반: ui/ 밖(design/samples/ 이외)을 참조한다
import { outside } from "../../../outside/secret.ts";
export const bad = outside;
