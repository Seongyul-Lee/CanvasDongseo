// 위반: 샘플 구현 → commands/core.ts. core.ts가 샘플 구현을 부르므로 순환이 된다
import { sampleCore } from "../core.ts";
export const bad = sampleCore;
