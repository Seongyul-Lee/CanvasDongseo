// PostToolUse 훅: ui/ 아래 JS·TS 파일을 고치면 그 파일에 eslint(폴더 의존 검사 포함)를 돌린다.
// 위반이 있으면 종료 코드 2와 eslint 출력을 stderr로 돌려줘 에이전트가 바로 고치게 한다.
import { spawnSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";

const input = JSON.parse(readFileSync(0, "utf8"));
const filePath = input.tool_input?.file_path;
const projectDir = process.env.CLAUDE_PROJECT_DIR ?? input.cwd ?? process.cwd();
const uiDir = path.join(projectDir, "ui");

if (typeof filePath !== "string") process.exit(0);
const absolute = path.resolve(projectDir, filePath);
const relative = path.relative(uiDir, absolute);
const insideUi = relative !== "" && !relative.startsWith("..") && !path.isAbsolute(relative);
if (!insideUi || !/\.(?:[cm]?[jt]sx?)$/.test(absolute) || !existsSync(absolute)) process.exit(0);
// eslint 설정에서 무시하는 폴더는 건너뛴다(경고만 나온다)
if (/^(?:node_modules|dist|test\/fixtures)\//.test(relative.split(path.sep).join("/"))) process.exit(0);

const eslintBin = path.join(uiDir, "node_modules", ".bin", "eslint");
if (!existsSync(eslintBin)) {
  process.stderr.write("ui/node_modules가 없어 eslint를 건너뛴다. `npm --prefix ui ci`를 먼저 실행한다.\n");
  process.exit(0);
}

const result = spawnSync(eslintBin, [absolute], { cwd: uiDir, encoding: "utf8" });
if (result.status === 0) process.exit(0);
process.stderr.write(`eslint(ui/)가 ${relative}에서 문제를 찾았다:\n${result.stdout}${result.stderr}`);
process.exit(2);
