import { mkdirSync, readdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const root = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const outputPath = resolve(process.argv[2] ?? join(root, "dist/lesson-guide.md"));

const syntaxSections = ["Estrutura do documento", "Markdown suportado", "Atributos", "Escapes"];
const blockSections = ["columns", "stack", "row", "grid", "box", "steps", "notes", "icon", "spacer", "divider"];

function read(path) {
  return readFileSync(join(root, path), "utf8");
}

function splitSections(markdown, level) {
  const marker = "#".repeat(level) + " ";
  const sections = new Map();
  let current = null;
  let inFence = false;
  for (const line of markdown.split("\n")) {
    if (line.startsWith("```")) inFence = !inFence;
    if (!inFence && line.startsWith(marker)) {
      current = { title: line.slice(marker.length).trim(), lines: [] };
      sections.set(current.title, current);
    } else if (current) {
      current.lines.push(line);
    }
  }
  return sections;
}

function section(sections, title) {
  const found = sections.get(title);
  if (!found) throw new Error(`Section not found: ${title}`);
  return found;
}

const forbiddenRowMarkers = [
  "`@version`", "`@page`", "`@color`", "`@font`", "`@style`", "`@component`", "`@state`", "`@transition`",
  "`@extern`", "`@persist`", "`@scorm`", "`@mode`", "`@header`", "`@footer`", "`@overlay`",
  "`@let`", "`@data`", "`![descrição](https://…)`", "`![descrição](https://…){fit=cover}`", "**Colado** no `)`", "`--- {hidden}`", "`--- {next=#id}`", "`--- {width=1080 height=1920}`", "`{.estilo}`", "`.estilo=condição`",
];

function dropForbiddenRows(text) {
  return text
    .split("\n")
    .filter((line) => !line.startsWith("Logo depois do separador"))
    .filter((line) => !(line.startsWith("|") && forbiddenRowMarkers.some((marker) => line.includes(marker))))
    .join("\n");
}

function clean(text) {
  return dropForbiddenRows(text)
    .replace(/^```ribalta .*$/gm, "```ribalta")
    .replace(/\[([^\]]+)\]\((?:\/|\.\.?\/)[^)]*\)/g, "$1")
    .replace(/\s*Veja \[?[^.\n]*\]?\.$/gm, "")
    .replace(/\n{3,}/g, "\n\n")
    .trim();
}

function demote(text) {
  let inFence = false;
  return text
    .split("\n")
    .map((line) => {
      if (line.startsWith("```")) inFence = !inFence;
      return !inFence && /^#{2,5} /.test(line) ? "#" + line : line;
    })
    .join("\n");
}

function renderSection(sections, title, heading) {
  const body = clean(section(sections, title).lines.join("\n"));
  return `### ${heading ?? title}\n\n${demote(body)}`;
}

const syntax = splitSections(read("docs/reference/syntax.md"), 2);
const blocks = splitSections(read("docs/reference/blocks.md"), 2);

const examplesDirectory = join(root, "docs/llm/lesson-examples");
const examples = readdirSync(examplesDirectory)
  .filter((name) => name.endsWith(".ribalta"))
  .sort()
  .map((name) => ({ name, source: readFileSync(join(examplesDirectory, name), "utf8").trimEnd() }));

const introduction = `# Guia para escrever aulas em Ribalta

Você vai escrever uma **aula**: um texto de estudo em Ribalta, uma linguagem de documentos baseada em Markdown. A aula é lida em rolagem contínua num aplicativo de flashcards, e cartões do baralho podem apontar para trechos dela. Escreva a aula em português, mas mantenha em inglês as palavras-chave da linguagem (\`lesson\`, \`columns\`, \`steps\`, \`box\` e afins).

Entregue apenas o código-fonte Ribalta da aula, sem cercas de código em volta e sem explicações.

## Regras essenciais

1. A aula é um documento de texto. Use títulos, parágrafos, listas, tabelas, citações, negrito e itálico. Blocos \`:::\` são opcionais e só servem para organizar (colunas, caixas).
2. Comece com \`@title Título da aula\` e, depois, o conteúdo. Só definições (\`@title\`, \`@theme\`) vêm antes do conteúdo.
3. Divida a aula em seções com \`---\`. Cada \`---\` começa uma página; no aplicativo as páginas aparecem empilhadas numa rolagem contínua. Dê um id a cada página: \`--- {#nome-da-secao}\`.
4. Dê um **id de âncora** a todo trecho importante, com \`{#id}\` no fim da linha de abertura do bloco. Os cartões do baralho serão ligados a esses ids, então cada conceito, definição, passo ou item que um cartão possa cobrir precisa de um.
5. Ids usam kebab-case ASCII: só letras minúsculas sem acento, números e hífens, começando por letra (\`fase-clara\`, \`queda-da-bastilha\`). Nada de espaços, acentos ou maiúsculas.
6. Ids são **únicos no documento inteiro**, incluindo os ids das páginas. Repetir um id, ou usar num bloco o mesmo id de uma página, é erro de compilação.
7. Âncoras só podem ficar em **títulos, parágrafos e itens de lista**. Citações (\`>\`) e tabelas não aceitam id: para tornar uma tabela ou citação alcançável, coloque um parágrafo ou título com id logo antes dela.
8. Sem id explícito, a página se chama \`page-N\`. Prefira sempre ids explícitos e descritivos.
9. Escreva texto de estudo claro e fiel: nada de links externos, imagens ou áudio.

## Como os cartões apontam para a aula

Um cartão referencia um trecho da aula por um alvo:

- \`page:<id>\` leva ao início da página com aquele id (\`page:fase-clara-pagina\`).
- \`anchor:<id>\` leva ao bloco com aquele id e o destaca (\`anchor:fotolise\`).

Por isso: quanto mais precisos e bem nomeados os ids, melhor o vínculo. Dê id a cada passagem que mereça um cartão (definições, fórmulas, datas, causas, etapas, exemplos). Um id deve descrever o conteúdo, não a posição (\`ciclo-de-calvin\`, não \`paragrafo-3\`).

## Sintaxe da âncora

\`\`\`ribalta
# Fotossíntese {#fotossintese}

A fase clara acontece nos tilacoides. {#fase-clara}

- Clorofila absorve luz {#clorofila}

--- {#fase-escura-pagina}
\`\`\`

O \`{#id}\` fica no fim da linha, depois de um espaço. Num parágrafo de várias linhas, ele vai na última linha e vale para o parágrafo todo. Podem vir outros atributos juntos: \`# Título {#id align=center}\`.

## Perfil \`lesson\`

A aula é validada com \`ribalta check --profile lesson\`. Esse perfil recusa (erro R212):

- imagens com endereço \`http://\` ou \`https://\`;
- o bloco \`sound\` e o bloco \`input\`;
- a definição \`@persist\`;
- os atributos \`next\` e \`hidden\` nas páginas.

Também não use \`@state\`, \`@data\`, \`@component\`, \`click\`, \`if\`, \`for\` nem expressões \`$( )\`: uma aula é texto estático. Evite \`$\` seguido de letras no texto; para um cifrão literal antes de um nome, escreva \`\\$nome\`.

## Layout vertical

As páginas se empilham numa rolagem contínua, sem tamanho fixo de slide: o conteúdo cresce com o texto. Use \`---\` para separar seções grandes (cada uma com seu \`{#id}\` de página e um título \`#\`). Não force alturas nem larguras. Prefira páginas curtas, cada uma com um tema.

## Boas práticas

- Um título \`#\` por página; subtítulos \`##\` e \`###\` dentro dela.
- Parágrafos curtos, uma ideia por parágrafo, cada um com id quando for matéria de cartão.
- Listas para enumerações, tabelas para comparações, \`::: steps\` para sequências.
- Linhas que começam com \`//\` são comentários e são ignoradas; não são necessárias.
- Depois de escrever, confira: todo id é único? todo id é kebab-case ASCII? nenhum id em tabela ou citação? nenhum recurso proibido pelo perfil?`;

const syntaxPart = `## Referência da sintaxe

${syntaxSections.map((title) => renderSection(syntax, title).replace(/^### /, "### ")).join("\n\n")}`;

const blocksPart = `## Blocos úteis em aulas

Todos os blocos de uma linha (\`:: nome\`) não aceitam conteúdo; os de contêiner (\`::: nome\`) fecham com \`:::\` sozinho na linha.

${blockSections.map((title) => renderSection(blocks, title)).join("\n\n")}`;

const examplesPart = `## Aulas de exemplo completas

As duas aulas abaixo passam em \`ribalta check --profile lesson\`. Siga o mesmo padrão: título, ids de página, âncoras em títulos, parágrafos e itens.

${examples.map(({ name, source }, index) => `### Exemplo ${index + 1} (${name})\n\n\`\`\`ribalta\n${source}\n\`\`\``).join("\n\n")}`;

const guide = [introduction, syntaxPart, blocksPart, examplesPart].join("\n\n") + "\n";

mkdirSync(dirname(outputPath), { recursive: true });
writeFileSync(outputPath, guide);
