// 통과해야 한다: commands/core.ts와 같은 모양. 개발 서버에서만 실행되는 갈래라 배포 빌드에서 지워진다
const used: Promise<string> =
  import.meta.env.DEV && import.meta.env.VITE_SAMPLE_MODE === "1"
    ? import("./commands/sample/demo.ts").then((sample) => sample.sampleMarker)
    : Promise.resolve("real");
void used.then((title) => {
  document.title = title;
});
