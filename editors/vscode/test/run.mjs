import { mkdirSync, mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { runTests } from "@vscode/test-electron";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const binary = process.env.RIBALTA_BIN ?? path.resolve(root, "../../target/debug/ribalta");
const scratch = mkdtempSync(path.join(tmpdir(), "ribalta-vscode-"));
const workspace = path.join(scratch, "workspace");
mkdirSync(path.join(workspace, ".vscode"), { recursive: true });
writeFileSync(path.join(workspace, "theme.ribalta"), "@style destaque {color=accent}\n");
writeFileSync(path.join(workspace, "aula.ribalta"), '--- {#inicio}\n# Olá {.destaque}\n[Ir]{click="go #fim"}\n\n--- {#final}\n# Fim\n');
writeFileSync(path.join(workspace, ".vscode", "settings.json"), JSON.stringify({ "ribalta.path": binary, "ribalta.language": "pt" }));

await runTests({
  vscodeExecutablePath: process.env.VSCODE_EXECUTABLE,
  extensionDevelopmentPath: root,
  extensionTestsPath: path.join(root, "test", "suite.cjs"),
  launchArgs: [workspace, "--disable-extensions", "--disable-workspace-trust", "--user-data-dir", path.join(scratch, "user"), "--extensions-dir", path.join(scratch, "extensions")],
});
