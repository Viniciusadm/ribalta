const assert = require("node:assert/strict");
const vscode = require("vscode");

async function eventually(read) {
  for (let attempt = 0; attempt < 100; attempt++) {
    const value = await read();
    if (value) return value;
    await new Promise((resolve) => setTimeout(resolve, 100));
  }
  throw new Error("timed out");
}

exports.run = async () => {
  const folder = vscode.workspace.workspaceFolders[0].uri;
  const document = await vscode.workspace.openTextDocument(vscode.Uri.joinPath(folder, "aula.ribalta"));
  await vscode.window.showTextDocument(document);
  assert.equal(document.languageId, "ribalta");

  const diagnostics = await eventually(() => vscode.languages.getDiagnostics(document.uri).length && vscode.languages.getDiagnostics(document.uri));
  assert.deepEqual(diagnostics.map((item) => item.code), ["R701"]);
  assert.equal(diagnostics[0].range.start.character, 15);
  assert.match(diagnostics[0].message, /dica:/);

  const completions = await vscode.commands.executeCommand("vscode.executeCompletionItemProvider", document.uri, new vscode.Position(2, 17), "#");
  assert.ok(completions.items.some((item) => item.label === "inicio"));

  const definitions = await vscode.commands.executeCommand("vscode.executeDefinitionProvider", document.uri, new vscode.Position(1, 10));
  assert.equal(definitions.length, 1);
  assert.ok((definitions[0].uri ?? definitions[0].targetUri).fsPath.endsWith("theme.ribalta"));

  const symbols = await vscode.commands.executeCommand("vscode.executeDocumentSymbolProvider", document.uri);
  assert.deepEqual(symbols.map((symbol) => symbol.name), ["#inicio · Olá", "#final · Fim"]);
  console.log("ribalta extension: diagnostics, completion, definition and symbols ok");
};
