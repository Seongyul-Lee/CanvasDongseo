// 허용: components → lib, hooks. T3 Code에서 가져온 부품처럼 별칭(~)과 확장자 없는 상대 경로를 쓴다
import { cn } from "~/lib/cn";
import { useThing } from "~/hooks/useThing";
import { merge } from "../../lib/merge";
export const Tip = () => [cn, useThing, merge];
