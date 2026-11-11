// 막아야 한다: 조건 없이 동적 import로 부른다(별도 조각으로 결과에 들어간다)
void import("./commands/sample/demo.ts").then((sample) => {
  document.title = sample.sampleMarker;
});
