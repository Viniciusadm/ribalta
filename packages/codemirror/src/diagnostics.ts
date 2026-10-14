import type { Diagnostic as EditorDiagnostic } from "@codemirror/lint";
import type { Text } from "@codemirror/state";
import type { Diagnostic } from "@ribaltalang/compiler";

export function toEditorDiagnostics(doc: Text, diagnostics: Diagnostic[]): EditorDiagnostic[] {
  return diagnostics.map((diagnostic) => {
    const line = doc.line(Math.min(Math.max(diagnostic.line, 1), doc.lines));
    const from = Math.min(line.from + Math.max(diagnostic.column - 1, 0), line.to);
    let to = Math.min(line.from + Math.max(diagnostic.endColumn - 1, 0), line.to);
    if (to <= from) to = line.to > from ? line.to : from;
    return {
      from,
      to,
      severity: diagnostic.severity,
      source: diagnostic.code,
      message: diagnostic.hint ? `${diagnostic.message}\n${diagnostic.hint}` : diagnostic.message,
    };
  });
}
