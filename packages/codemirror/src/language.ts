import { HighlightStyle, StreamLanguage, syntaxHighlighting, type StringStream } from "@codemirror/language";
import { tags } from "@lezer/highlight";

interface State {
  fence: boolean;
  attrs: boolean;
  heading: boolean;
}

const listMarker = /^(\s*)([-*+]|\d{1,9}[.)])\s/;

function attrToken(stream: StringStream, state: State): string | null {
  if (stream.eatSpace()) return null;
  if (stream.eat("}")) {
    state.attrs = false;
    return "brace";
  }
  if (stream.match(/^\.[\w$-]+(?==)/)) return "className";
  if (stream.match(/^\.[\w$-]+/)) return "className";
  if (stream.match(/^#[\w-]+/)) return "labelName";
  if (stream.match(/^[A-Za-z$][\w$-]*(?==)/)) return "propertyName";
  if (stream.eat("=")) {
    if (stream.match(/^"(?:[^"\\]|\\.)*"?/) || stream.match(/^'(?:[^'\\]|\\.)*'?/)) return "string";
    if (stream.match(/^-?\d[\w.%]*/)) return "number";
    if (stream.match(/^\$[\w.-]+/)) return "variableName";
    stream.match(/^[^\s}]+/);
    return "atom";
  }
  if (stream.match(/^[A-Za-z$][\w$-]*/)) return "propertyName";
  stream.next();
  return null;
}

function inlineToken(stream: StringStream, state: State): string | null {
  if (stream.match(/^\\./)) return null;
  if (stream.match(/^\s\{(?=[^{}]*\}\s*$)/, false) || (stream.sol() && stream.match(/^\{(?=[^{}]*\}\s*$)/, false))) {
    stream.eatSpace();
    stream.next();
    state.attrs = true;
    return "brace";
  }
  if (stream.match(/^\]\{/)) {
    state.attrs = true;
    return "brace";
  }
  if (stream.match(/^`[^`]*`/)) return "monospace";
  if (stream.match(/^\*\*[^*]+\*\*/)) return "strong";
  if (stream.match(/^\*[^*\s][^*]*\*/)) return "emphasis";
  if (stream.match(/^~~[^~]+~~/)) return "strikethrough";
  if (stream.match(/^!?\[(?=[^\]]*\][({])/)) return "bracket";
  if (stream.match(/^\]\([^)]*\)/)) return "url";
  if (stream.match(/^\$\([^)]*\)?/)) return "variableName";
  if (stream.match(/^\$[A-Za-z][\w-]*(?:\.[A-Za-z][\w-]*)*/)) return "variableName";
  stream.next();
  while (!stream.eol() && !/[\\`*~![\]$\s{]/.test(stream.peek() ?? "")) stream.next();
  return state.heading ? "heading" : null;
}

export const ribaltaLanguage = StreamLanguage.define<State>({
  name: "ribalta",
  startState: () => ({ fence: false, attrs: false, heading: false }),
  copyState: (state) => ({ ...state }),
  token(stream, state) {
    if (stream.sol()) {
      state.attrs = false;
      state.heading = false;
      const line = stream.string;
      if (/^\s*```/.test(line)) {
        state.fence = !state.fence;
        stream.skipToEnd();
        return "monospace";
      }
      if (state.fence) {
        stream.skipToEnd();
        return "monospace";
      }
      if (/^\s*\/\//.test(line)) {
        stream.skipToEnd();
        return "comment";
      }
      if (/^---+\s*(\{.*\})?\s*$/.test(line)) {
        stream.match(/^-+/);
        return "contentSeparator";
      }
      if (stream.match(/^\s*@\w+/)) return "keyword";
      if (stream.match(/^\s*:{2,3}/)) {
        stream.eatSpace();
        if (stream.match(/^[\w-]+/)) return "typeName";
        return "operator";
      }
      const heading = stream.match(/^\s*#{1,6}(?=\s)/);
      if (heading) {
        state.heading = true;
        return "heading";
      }
      if (stream.match(/^\s*>/)) return "quote";
      if (stream.match(listMarker)) return "list";
      if (stream.match(/^\s*\$content\s*$/)) return "variableName";
    }
    if (state.fence) {
      stream.skipToEnd();
      return "monospace";
    }
    if (state.attrs) return attrToken(stream, state);
    return inlineToken(stream, state);
  },
  tokenTable: {
    contentSeparator: tags.contentSeparator,
    brace: tags.brace,
    className: tags.className,
    labelName: tags.labelName,
    propertyName: tags.propertyName,
    typeName: tags.typeName,
    strikethrough: tags.strikethrough,
    bracket: tags.squareBracket,
    list: tags.list,
  },
});

export const ribaltaHighlight = syntaxHighlighting(
  HighlightStyle.define([
    { tag: tags.contentSeparator, color: "#ff8a65", fontWeight: "700" },
    { tag: tags.heading, color: "#ffd479", fontWeight: "700" },
    { tag: tags.keyword, color: "#c792ea" },
    { tag: tags.typeName, color: "#82aaff", fontWeight: "600" },
    { tag: tags.operator, color: "#89ddff" },
    { tag: tags.className, color: "#f78c6c" },
    { tag: tags.labelName, color: "#f78c6c" },
    { tag: tags.propertyName, color: "#80cbc4" },
    { tag: [tags.string, tags.atom, tags.number], color: "#c3e88d" },
    { tag: tags.variableName, color: "#ff5370" },
    { tag: tags.brace, color: "#89ddff" },
    { tag: tags.comment, color: "#697098", fontStyle: "italic" },
    { tag: tags.monospace, color: "#b2ccd6" },
    { tag: tags.strong, fontWeight: "700", color: "#eeffff" },
    { tag: tags.emphasis, fontStyle: "italic" },
    { tag: tags.strikethrough, textDecoration: "line-through" },
    { tag: tags.squareBracket, color: "#89ddff" },
    { tag: tags.url, color: "#7e8bbd" },
    { tag: tags.quote, color: "#89ddff" },
    { tag: tags.list, color: "#ff8a65" },
  ]),
);
