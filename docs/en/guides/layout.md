---
title: Layout
description: Configure the page, arrange content in columns, stacks, rows and grids, and place elements precisely.
---

By default, the content of a page is stacked from top to bottom, aligned to the left and centered vertically. To change that, you configure the page on the `---` line and use layout blocks.

## Page attributes

Attributes in braces on the `---` line apply to the whole page:

```ribalta preview
--- {align=center valign=bottom bg=blue color=white pad="lg 2xl"}
# Wrapping up
Thank you for your time.
```

The most common ones:

| Attribute | What it does | Example |
| --- | --- | --- |
| `align` | Horizontal alignment of the content: `left`, `center` or `right` | `align=center` |
| `valign` | Vertical alignment of the content: `top`, `center` or `bottom` | `valign=top` |
| `bg` | Background color or gradient | `bg=ink..violet` |
| `color` | Text color | `color=white` |
| `image` | Background image | `image=https://…` |
| `pad` | Inner margin: one value, or two (vertical and horizontal) | `pad="lg 2xl"` |
| `gap` | Space between the blocks of the page | `gap=lg` |
| `#id` | Page identifier | `#cover` |
| `.style` | Applies a built-in style or one of your own | `.dark` |
| `width`, `height` | Size of this page, in pixels; `height=auto` grows with the content. See [Page sizes](/en/guides/pages/) | `width=1080 height=1920` |

A page accepts any style attribute. See the full list in [Attributes](/en/reference/attributes/).

:::tip
To configure the first page, start the text with a `--- {…}` line. It does not create an empty page before it.
:::

## Layout blocks

A layout block starts with `:::` followed by its name and ends with a line containing only `:::`. The block attributes go in braces on the opening line:

```ribalta
::: columns {gap=xl}
Content of the first column

Content of the second column
:::
```

Each direct block inside a layout is an item: a heading, a paragraph, a list, an image or another layout block. A `:::` on its own always closes the most recently opened block. Indentation is optional and only helps readability.

## Columns

`::: columns` places each item side by side. Use `widths` for the proportions and `gap` for the space between the columns:

```ribalta preview
# Why change now
::: columns {widths="2 1" gap=xl valign=top}
::: stack {gap=sm}
## The landscape
The market doubled in two years.
Customers expect faster answers.
:::
::: box {.card}
## Goal
Cut lead time in half.
:::
:::
```

| Attribute | What it does |
| --- | --- |
| `widths` | Proportion of each column, separated by spaces. `widths="2 1"` makes the first column twice as wide as the second. Without it, the columns share the space equally. |
| `gap` | Space between the columns. |
| `valign` | Vertical alignment of the columns: `top`, `center` (default) or `bottom`. |

:::note
The number of values in `widths` should match the number of columns. If it does not, the compiler warns.
:::

## Stack

`::: stack` stacks its items from top to bottom. It is useful to group several blocks into a single column or to control the space between them with `gap`:

```ribalta preview
::: stack {gap=xs}
Chapter 1 {.kicker}
# Where we stand
An overview of the market in 2026. {.lead}
:::
```

## Row

`::: row` places its items side by side, each as wide as its own content. With `wrap`, items that do not fit move to the next line:

```ribalta preview
# Technologies
::: row {gap=lg wrap}
:: icon database {size=3xl color=accent}
:: icon cloud {size=3xl color=blue}
:: icon shield {size=3xl color=green}
:: icon cpu {size=3xl color=violet}
:::
```

| Attribute | What it does |
| --- | --- |
| `gap` | Space between the items. |
| `wrap` | Wraps onto a new line when the items do not fit. |
| `align` | Horizontal position of the items: `left`, `center` or `right`. |
| `valign` | Vertical alignment of the items: `top`, `center` (default) or `bottom`. |

## Grid

`::: grid` lays its items out in columns of equal width and creates new rows as needed. Set the number of columns with `cols`, from 1 to 12. Without `cols`, the grid uses one column per item, up to a maximum of three:

```ribalta preview
# Key figures
::: grid {cols=2 gap=md}
::: box {.card}
## 94
active customers {.muted}
:::
::: box {.card}
## 38%
annual growth {.muted}
:::
::: box {.card}
## 4.8
satisfaction score {.muted}
:::
::: box {.card}
## 12
countries served {.muted}
:::
:::
```

## Box

`::: box` groups stacked items, like a stack, and is the simplest way to give a set of blocks a background, border, rounded corners and shadow. The built-in `.card` style applies all of that at once:

```ribalta preview
::: box {.card align=center gap=xs}
:: icon rocket {size=3xl color=accent}
## Launch
Available to every customer in March.
:::
```

## Nesting blocks

Layout blocks can sit inside one another. Each `:::` on its own closes the most recently opened block, so the structure reads from the inside out:

```ribalta preview
::: columns {gap=xl}
::: stack {gap=sm}
## Before
- Manual reports
- Mismatched data
:::
::: box {.card gap=sm}
## After
::: row {gap=sm}
:: icon check {size=xl color=green}
Automatic reports
:::
::: row {gap=sm}
:: icon check {size=xl color=green}
A single source of data
:::
:::
:::
```

## Alignment

In Ribalta, `align` is **always horizontal** and `valign` is **always vertical**, in any block and any layout direction. You do not need to think about axes:

```ribalta preview
::: columns {gap=lg}
::: box {.card h=600 align=center valign=center}
## Center
:::
::: box {.card h=600 align=right valign=bottom}
## Corner
:::
:::
```

| Attribute | Values |
| --- | --- |
| `align` | `left`, `center`, `right` |
| `valign` | `top`, `center`, `bottom` |

## Size and absolute position

The `w` and `h` attributes set width and height. They accept pixels (`480`), a percentage (`50%`), `full` or `auto`.

The `x` and `y` attributes take a block out of the flow and place it at an exact point on the page, measured in pixels from the page's top-left corner. In the example, the page is 1920 × 1080 pixels; the size of each page comes from [`@page`](/en/guides/pages/) or from the program that shows the document:

```ribalta preview
# Launching in March
The schedule is confirmed for every team.

Confidential {x=1560 y=64 .kicker}

::: box {x=1360 y=760 w=440 bg=accent color=white pad=md radius=lg}
Final deadline: March 31
:::
```

:::caution
Elements placed with `x` and `y` do not push other blocks around and can end up on top of them. Use them sparingly, for badges, marks and details.
:::

## Spacer and divider

Two single-line blocks help adjust the rhythm of the page:

- `:: spacer` creates empty space. Add the size after the name, using the spacing scale (`xs` to `3xl`) or pixels. Without a value, the space is `md`.
- `:: divider` draws a horizontal line. Change its color with `bg` and its width with `w`.

```ribalta preview
# Agenda
:: divider {w=160 bg=accent}
:: spacer lg
1. Context
2. Proposal
3. Next steps
```

## Next steps

- [Styling](/en/guides/styling/): colors, sizes, borders and reusable styles.
- [Components](/en/guides/components/): turn repeated layouts into reusable blocks.
- [Blocks reference](/en/reference/blocks/): every block and its attributes.
