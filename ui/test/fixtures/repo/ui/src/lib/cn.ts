// 허용: lib → 외부 패키지
import { useId } from "react";
export const cn = (...names: string[]) => [names.join(" "), useId];
