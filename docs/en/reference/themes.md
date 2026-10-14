---
title: Built-in themes
description: Colors, fonts, sizes and styles of each built-in Ribalta theme.
---

Ribalta has three built-in themes: `paper` (the default), `ink` and `editorial`. Pick one with `@theme` at the start of the document or in the [project theme](/docs/en/guides/themes/). Any token can be replaced with `@color` and `@font`.

```ribalta
@theme editorial
@color accent #0f766e
@font display "Playfair Display" 700
```

## Overview

| | `paper` | `ink` | `editorial` |
| --- | --- | --- | --- |
| Look | Light, cream paper | Dark | Light, magazine |
| Text font (`body`) | Barlow | Inter | Source Sans 3 |
| Heading font (`display`) | Archivo Black | Inter | Fraunces |
| Code font (`mono`) | JetBrains Mono | JetBrains Mono | JetBrains Mono |
| Text size | 40 px | 38 px | 40 px |
| Text line height | 1.35 | 1.4 | 1.4 |
| Page padding (vertical × horizontal) | 96 × 128 px | 96 × 128 px | 104 × 140 px |
| Space between page blocks | 32 px | 32 px | 32 px |
| Code blocks | Dark background | Dark background | Light background |

In every theme, page content starts aligned to the left and vertically centered. On a page with [automatic height](/docs/en/guides/pages/), there is no spare room, so the content simply starts at the top.

## paper

The default theme: cream background, graphite text and a coral accent.

| Token | Value |
| --- | --- |
| `bg` | `#fbf7ef` |
| `text` | `#1f2430` |
| `muted` | `#6b6f7b` |
| `accent` | `#f2603d` |
| `surface` | `#ffffff` |
| `line` | `#1f243026` |

| Font | Family | Loaded weights |
| --- | --- | --- |
| `body` | Barlow | 300, 400, 500, 600, 700, 900 |
| `display` | Archivo Black | Family default |
| `mono` | JetBrains Mono | 400, 700 |

| Level | Size | Weight | Font | Line height |
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
Paper theme {.kicker}
# Clear and direct
Text in Barlow, headings in Archivo Black. {.muted}
:::
::: box {.card}
### Card
`surface` background with an [accent]{.accent}.
:::
:::
```

## ink

Dark theme: near-black background, light text and an amber accent. It also redefines the `ink` token.

| Token | Value |
| --- | --- |
| `bg` | `#12141c` |
| `text` | `#f4f1ea` |
| `muted` | `#a3a7b3` |
| `accent` | `#ffb547` |
| `surface` | `#1d2130` |
| `line` | `#ffffff26` |
| `ink` | `#0b0c11` |

| Font | Family | Loaded weights |
| --- | --- | --- |
| `body` | Inter | 300, 400, 500, 600, 700, 800, 900 |
| `display` | Inter | 300, 400, 500, 600, 700, 800, 900 |
| `mono` | JetBrains Mono | 400, 700 |

| Level | Size | Weight | Font | Line height |
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
Ink theme {.kicker}
# Dark and bold
All in Inter, with headings at weight 800. {.muted}
:::
::: box {.card}
### Card
`surface` background with an [accent]{.accent}.
:::
:::
```

## editorial

Magazine theme: paper background, serif headings and a brick-red accent. The first three heading levels use the `display` font.

| Token | Value |
| --- | --- |
| `bg` | `#f6f1e7` |
| `text` | `#1b1b1b` |
| `muted` | `#6d665c` |
| `accent` | `#b23a2b` |
| `surface` | `#fffdf8` |
| `line` | `#1b1b1b22` |

| Font | Family | Loaded weights |
| --- | --- | --- |
| `body` | Source Sans 3 | 300, 400, 600, 700, 900 |
| `display` | Fraunces | 400, 600, 700, 900 |
| `mono` | JetBrains Mono | 400, 700 |

| Level | Size | Weight | Font | Line height |
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
Editorial theme {.kicker}
# Elegant and serif
Text in Source Sans 3, headings in Fraunces. {.muted}
:::
::: box {.card}
### Card
`surface` background with an [accent]{.accent}.
:::
:::
```

## Shared tokens

These tokens exist in every theme, with the same values (except `ink`, which the `ink` theme redefines).

| Token | Value |
| --- | --- |
| `coral` | `#f2603d` |
| `blue` | `#2f6fdb` |
| `green` | `#23976a` |
| `yellow` | `#f2b632` |
| `violet` | `#7c5cd6` |
| `white` | `#ffffff` |
| `black` | `#000000` |
| `ink` | `#151821` (`#0b0c11` in the `ink` theme) |
| `paper` | `#fbf7ef` |
| `transparent` | `#00000000` |

The chart palette uses, in order, `accent`, `blue`, `green`, `yellow`, `violet` and `coral`, with the values of the current theme.

## Built-in styles

Available in every theme. Apply them with `{.name}`; a `@style` with the same name replaces the built-in style.

| Style | Definition | Use |
| --- | --- | --- |
| `.muted` | `color=muted` | Secondary text. |
| `.accent` | `color=accent` | Text in the accent color. |
| `.lead` | `size=xl color=muted` | Opening paragraph, larger and softer. |
| `.kicker` | `size=sm weight=bold uppercase tracking=0.14 color=accent` | Short label above a heading. |
| `.display` | `font=display` | Uses the heading font. |
| `.huge` | `font=display size=6xl leading=0.95` | Huge text, for numbers and punchlines. |
| `.card` | `bg=surface radius=lg pad=lg shadow=md` | Card with background, corners and shadow. |
| `.dark` | `bg=ink color=paper` | Dark background with light text. |
| `.center` | `align=center` | Centers horizontally. |
