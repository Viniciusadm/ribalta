---
title: Ribalta
description: Uma linguagem de texto para visuais com tema, inspirada em Markdown.
template: splash
hero:
  tagline: Escreva o texto. O tema cuida do visual.
  actions:
    - text: Escrevendo documentos
      link: guides/writing/
      icon: right-arrow
    - text: Referência da sintaxe
      link: reference/syntax/
      variant: minimal
---

## Um documento em Ribalta

```ribalta preview
--- {align=center}
Relatório trimestral {.kicker}
# Crescemos [38%]{color=yellow} no trimestre
Receita recorrente acima da meta pelo terceiro mês seguido. {.lead}
```

Cada linha `---` começa uma página. Títulos, listas e ênfase são escritos como em Markdown; blocos `:::` montam colunas, cards e gráficos; atributos entre `{…}` ajustam o que for preciso. Cores, fontes e espaçamentos vêm do tema, pelo nome.

## O que você encontra aqui

- **[Guias](/guides/writing/)**: texto, tamanhos de página, layouts, estilos, temas, cabeçalho e rodapé, componentes, dados, interatividade, revelação em etapas, gráficos e imagens.
- **[Referência](/reference/syntax/)**: toda a sintaxe, atributos, blocos, temas, ícones e mensagens de erro.

## Usando a linguagem em um programa

| Pacote | Para quê |
| --- | --- |
| [`@ribaltalang/compiler`](https://www.npmjs.com/package/@ribaltalang/compiler) | Compila o texto no navegador ou no Node.js (WebAssembly) e devolve o documento em JSON com os diagnósticos. |
| [`@ribaltalang/renderer`](https://www.npmjs.com/package/@ribaltalang/renderer) | Desenha o documento com React. |
| [`@ribaltalang/player`](https://www.npmjs.com/package/@ribaltalang/player) | Navegação, etapas, estado, temporizadores, teclado e sons de um documento interativo. |
| [`@ribaltalang/codemirror`](https://www.npmjs.com/package/@ribaltalang/codemirror) | Cores, autocompletar e erros no CodeMirror 6. |
| [`ribalta`](https://github.com/Viniciusadm/ribalta/tree/master/crates/ribalta) | O compilador como biblioteca Rust. |
| [`ribalta-cli`](https://github.com/Viniciusadm/ribalta/tree/master/crates/cli) | O comando `ribalta`: verifica, formata, mostra o mapa de rotas e serve de servidor de linguagem para editores. |
| [Extensão do VS Code](https://github.com/Viniciusadm/ribalta/tree/master/editors/vscode) | Cores, erros enquanto você digita, autocompletar, ir para a definição e formatação. |

O formato do documento compilado está descrito em [IR.md](https://github.com/Viniciusadm/ribalta/blob/master/crates/ribalta/IR.md).
