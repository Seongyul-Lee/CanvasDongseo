// 샘플 모드 예시의 ViewModel·이벤트 타입. 생성 도구를 정하기 전이라 손으로 쓴 임시 파일이다(README.md).
// 첫 기능이 들어오면 지운다(design/samples/example/README.md)

export interface ExampleViewModel {
  title: string;
  description: string;
  /** 없으면 null */
  banner: ExampleBanner | null;
  items: ExampleItem[];
  /** 목록이 비었을 때 보여 줄 문구. 목록이 있으면 null */
  emptyMessage: string | null;
}

export interface ExampleBanner {
  kind: "info" | "success" | "warning" | "error";
  title: string;
  message: string;
}

export interface ExampleItem {
  id: string;
  name: string;
  detail: string;
  badge: { kind: "info" | "success" | "warning" | "outline"; label: string };
}

/** 이벤트 `example://progress`의 내용 */
export interface ExampleProgressEvent {
  label: string;
  /** 0~100 */
  percent: number;
}
