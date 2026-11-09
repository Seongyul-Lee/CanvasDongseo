// 허용: 생성 타입이 같은 폴더의 타입을 참조
import type { Banner } from "./banner.ts";
export type Course = { name: string; banner: Banner };
