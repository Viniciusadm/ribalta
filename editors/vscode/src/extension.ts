import { statSync } from "node:fs";
import * as path from "node:path";
import * as vscode from "vscode";
import { LanguageClient, type LanguageClientOptions, type ServerOptions } from "vscode-languageclient/node";

const portuguese = vscode.env.language.toLowerCase().startsWith("pt");
const text = {
  missing: (command: string) =>
    portuguese
      ? `Não foi possível iniciar \`${command} lsp\`. Instale o CLI da Ribalta ou ajuste a configuração ribalta.path.`
      : `Could not start \`${command} lsp\`. Install the Ribalta CLI or set ribalta.path.`,
  saveFirst: portuguese ? "Salve o arquivo antes de usar este comando." : "Save the file before using this command.",
  openSettings: portuguese ? "Abrir configurações" : "Open settings",
};

let client: LanguageClient | undefined;

export async function activate(context: vscode.ExtensionContext) {
  context.subscriptions.push(
    vscode.workspace.onDidChangeConfiguration(async (event) => {
      if (event.affectsConfiguration("ribalta")) await restart();
    }),
    vscode.commands.registerCommand("ribalta.restartServer", restart),
    vscode.commands.registerCommand("ribalta.graph", () => runInTerminal((file) => ["graph", file, ...themeArgs(file)])),
  );
  await start();
}

export async function deactivate() {
  await client?.stop();
  client = undefined;
}

function settings() {
  const config = vscode.workspace.getConfiguration("ribalta");
  const language = config.get<string>("language", "auto");
  return {
    command: config.get<string>("path", "ribalta").trim() || "ribalta",
    theme: config.get<string>("theme", "").trim(),
    language: language === "auto" ? (portuguese ? "pt" : "en") : language,
  };
}

async function start() {
  const { command, theme, language } = settings();
  const server: ServerOptions = { command, args: ["lsp"], options: { env: { ...process.env, RIBALTA_LANG: language } } };
  const options: LanguageClientOptions = {
    documentSelector: [{ language: "ribalta" }],
    initializationOptions: { theme },
  };
  client = new LanguageClient("ribalta", "Ribalta", server, options);
  try {
    await client.start();
  } catch {
    client = undefined;
    const choice = await vscode.window.showErrorMessage(text.missing(command), text.openSettings);
    if (choice) await vscode.commands.executeCommand("workbench.action.openSettings", "ribalta.path");
  }
}

async function restart() {
  await deactivate();
  await start();
}

function themeArgs(file: string) {
  const { theme } = settings();
  const resolved = theme ? path.resolve(path.dirname(file), theme) : path.join(path.dirname(file), "theme.ribalta");
  if (resolved === file) return [];
  return theme || vscode.workspace.textDocuments.some((document) => document.fileName === resolved) || exists(resolved) ? ["--theme", resolved] : [];
}

function exists(file: string) {
  try {
    return statSync(file).isFile();
  } catch {
    return false;
  }
}

async function runInTerminal(argsFor: (file: string) => string[]) {
  const document = vscode.window.activeTextEditor?.document;
  if (!document || document.languageId !== "ribalta") return;
  if (document.isUntitled) {
    await vscode.window.showWarningMessage(text.saveFirst);
    return;
  }
  await document.save();
  const { command, language } = settings();
  const terminal = vscode.window.createTerminal({ name: "Ribalta", cwd: path.dirname(document.fileName), env: { RIBALTA_LANG: language } });
  terminal.show();
  terminal.sendText([command, ...argsFor(document.fileName)].map(quote).join(" "));
}

function quote(value: string) {
  return /^[\w@%+=:,./-]+$/.test(value) ? value : `'${value.replaceAll("'", `'\\''`)}'`;
}
