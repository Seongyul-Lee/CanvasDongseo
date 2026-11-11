// 코어를 부르는 통로의 모양(commands). 실제 구현(tauri.ts)과 샘플 구현(sample/)이 함께 지킨다(presentation §2.4)

/** 이벤트 구독을 끊는다 */
export type Unlisten = () => void;

export interface Core {
  /** command를 부르고 응답(ViewModel)을 받는다 */
  invoke<T>(command: string, args?: Record<string, unknown>): Promise<T>;
  /** 코어 이벤트를 구독한다 */
  listen<T>(event: string, handler: (payload: T) => void): Promise<Unlisten>;
}
