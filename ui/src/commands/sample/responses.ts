// 샘플 구현(commands/sample): command 이름 → 응답으로 돌려줄 ViewModel의 기능(design/samples/<기능>/).
// 기능의 command 래퍼를 만들 때 여기에 한 줄을 더한다. 표에 없는 command는 샘플 모드에서 실패한다
export const sampleResponses: Record<string, string> = {
  // 예시. 첫 기능이 들어오면 지운다(design/samples/example/README.md)
  get_example: "example",
};
