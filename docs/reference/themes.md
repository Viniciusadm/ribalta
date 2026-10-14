---
title: Temas embutidos
description: Cores, fontes, tamanhos e estilos de cada tema embutido da Ribalta.
---

A Ribalta tem três temas embutidos: `paper` (o padrão), `ink` e `editorial`. Escolha um com `@theme` no início do documento ou no [tema do projeto](/docs/guides/themes/). Qualquer token pode ser substituído com `@color` e `@font`.

```ribalta
@theme editorial
@color accent #0f766e
@font display "Playfair Display" 700
```

## Visão geral

| | `paper` | `ink` | `editorial` |
| --- | --- | --- | --- |
| Aparência | Clara, papel creme | Escura | Clara, revista |
| Fonte do texto (`body`) | Barlow | Inter | Source Sans 3 |
| Fonte de títulos (`display`) | Archivo Black | Inter | Fraunces |
| Fonte de código (`mono`) | JetBrains Mono | JetBrains Mono | JetBrains Mono |
| Tamanho do texto | 40 px | 38 px | 40 px |
| Entrelinha do texto | 1.35 | 1.4 | 1.4 |
| Margem da página (vertical × horizontal) | 96 × 128 px | 96 × 128 px | 104 × 140 px |
| Espaço entre blocos da página | 32 px | 32 px | 32 px |
| Blocos de código | Fundo escuro | Fundo escuro | Fundo claro |

Em todos os temas, o conteúdo da página começa alinhado à esquerda e centralizado na vertical.

## paper

Tema padrão: fundo creme, texto grafite e destaque coral.

| Token | Valor |
| --- | --- |
| `bg` | `#fbf7ef` |
| `text` | `#1f2430` |
| `muted` | `#6b6f7b` |
| `accent` | `#f2603d` |
| `surface` | `#ffffff` |
| `line` | `#1f243026` |

| Fonte | Família | Pesos carregados |
| --- | --- | --- |
| `body` | Barlow | 300, 400, 500, 600, 700, 900 |
| `display` | Archivo Black | Padrão da família |
| `mono` | JetBrains Mono | 400, 700 |

| Nível | Tamanho | Peso | Fonte | Entrelinha |
| --- | --- | --- | --- | --- |
| `#` | 88 px | 400 | `display` | 1.02 |
| `##` | 64 px | 400 | `display` | 1.05 |
| `###` | 52 px | 700 | `body` | 1.1 |
| `####` | 40 px | 700 | `body` | 1.2 |
| `#####` | 32 px | 700 | `body` | 1.2 |
| `######` | 26 px | 700 | `body` | 1.2 |

```ribalta preview
@theme paper

::: columns {gap=xl}
::: stack {gap=sm}
Tema paper {.kicker}
# Claro e direto
Texto em Barlow, títulos em Archivo Black. {.muted}
:::
::: box {.card}
### Cartão
Fundo `surface` com [destaque]{.accent}.
:::
:::
```

## ink

Tema escuro: fundo quase preto, texto claro e destaque âmbar. Também redefine o token `ink`.

| Token | Valor |
| --- | --- |
| `bg` | `#12141c` |
| `text` | `#f4f1ea` |
| `muted` | `#a3a7b3` |
| `accent` | `#ffb547` |
| `surface` | `#1d2130` |
| `line` | `#ffffff26` |
| `ink` | `#0b0c11` |

| Fonte | Família | Pesos carregados |
| --- | --- | --- |
| `body` | Inter | 300, 400, 500, 600, 700, 800, 900 |
| `display` | Inter | 300, 400, 500, 600, 700, 800, 900 |
| `mono` | JetBrains Mono | 400, 700 |

| Nível | Tamanho | Peso | Fonte | Entrelinha |
| --- | --- | --- | --- | --- |
| `#` | 92 px | 800 | `display` | 1.0 |
| `##` | 66 px | 800 | `display` | 1.05 |
| `###` | 50 px | 700 | `body` | 1.1 |
| `####` | 40 px | 700 | `body` | 1.2 |
| `#####` | 32 px | 600 | `body` | 1.2 |
| `######` | 26 px | 600 | `body` | 1.2 |

```ribalta preview
@theme ink

::: columns {gap=xl}
::: stack {gap=sm}
Tema ink {.kicker}
# Escuro e marcante
Tudo em Inter, com títulos em peso 800. {.muted}
:::
::: box {.card}
### Cartão
Fundo `surface` com [destaque]{.accent}.
:::
:::
```

## editorial

Tema de revista: fundo de papel, títulos serifados e destaque vermelho-tijolo. Os três primeiros níveis de título usam a fonte `display`.

| Token | Valor |
| --- | --- |
| `bg` | `#f6f1e7` |
| `text` | `#1b1b1b` |
| `muted` | `#6d665c` |
| `accent` | `#b23a2b` |
| `surface` | `#fffdf8` |
| `line` | `#1b1b1b22` |

| Fonte | Família | Pesos carregados |
| --- | --- | --- |
| `body` | Source Sans 3 | 300, 400, 600, 700, 900 |
| `display` | Fraunces | 400, 600, 700, 900 |
| `mono` | JetBrains Mono | 400, 700 |

| Nível | Tamanho | Peso | Fonte | Entrelinha |
| --- | --- | --- | --- | --- |
| `#` | 96 px | 600 | `display` | 1.0 |
| `##` | 68 px | 600 | `display` | 1.05 |
| `###` | 52 px | 600 | `display` | 1.1 |
| `####` | 40 px | 700 | `body` | 1.2 |
| `#####` | 32 px | 700 | `body` | 1.2 |
| `######` | 26 px | 700 | `body` | 1.2 |

```ribalta preview
@theme editorial

::: columns {gap=xl}
::: stack {gap=sm}
Tema editorial {.kicker}
# Elegante e serifado
Texto em Source Sans 3, títulos em Fraunces. {.muted}
:::
::: box {.card}
### Cartão
Fundo `surface` com [destaque]{.accent}.
:::
:::
```

## Tokens compartilhados

Estes tokens existem em todos os temas, com os mesmos valores (exceto `ink`, redefinido pelo tema `ink`).

| Token | Valor |
| --- | --- |
| `coral` | `#f2603d` |
| `blue` | `#2f6fdb` |
| `green` | `#23976a` |
| `yellow` | `#f2b632` |
| `violet` | `#7c5cd6` |
| `white` | `#ffffff` |
| `black` | `#000000` |
| `ink` | `#151821` (`#0b0c11` no tema `ink`) |
| `paper` | `#fbf7ef` |
| `transparent` | `#00000000` |

A paleta dos gráficos usa, na ordem, `accent`, `blue`, `green`, `yellow`, `violet` e `coral`, com os valores do tema em uso.

## Estilos embutidos

Disponíveis em todos os temas. Aplique com `{.nome}`; um `@style` com o mesmo nome substitui o estilo embutido.

| Estilo | Definição | Uso |
| --- | --- | --- |
| `.muted` | `color=muted` | Texto secundário. |
| `.accent` | `color=accent` | Texto na cor de destaque. |
| `.lead` | `size=xl color=muted` | Parágrafo de abertura, maior e mais suave. |
| `.kicker` | `size=sm weight=bold uppercase tracking=0.14 color=accent` | Rótulo curto acima de um título. |
| `.display` | `font=display` | Usa a fonte de títulos. |
| `.huge` | `font=display size=6xl leading=0.95` | Texto enorme, para números e frases de impacto. |
| `.card` | `bg=surface radius=lg pad=lg shadow=md` | Cartão com fundo, cantos e sombra. |
| `.dark` | `bg=ink color=paper` | Fundo escuro com texto claro. |
| `.center` | `align=center` | Centraliza na horizontal. |
