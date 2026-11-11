// T3 Code의 apps/web/src/lib/utils.ts에서 cn과 tailwind-merge 설정만 가져왔다(MIT, ui/LICENSE-t3code).
// 원본의 나머지(ID 생성, 플랫폼 판별)는 T3 Code 내부 패키지에 묶여 있어 가져오지 않는다. 기록은 ui/UPSTREAM.md
import { type CxOptions, cx } from "class-variance-authority";
import { extendTailwindMerge } from "tailwind-merge";

// Register extra font sizes and background images so tailwind-merge does not
// treat text-2xs or bg-composer-seam-below as colors.
const twMerge = extendTailwindMerge({
  extend: {
    theme: { text: ["2xs", "3xs", "4xs", "5xs"] },
    classGroups: { "bg-image": [{ bg: ["composer-seam-below", "composer-seam-above"] }] },
  },
});

export function cn(...inputs: CxOptions) {
  return twMerge(cx(inputs));
}
