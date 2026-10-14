---
title: Blocks
description: Every built-in block in the language, with its form, content, own attributes and examples.
---

Blocks organize the content of a page. There are two forms:

| Form | Syntax | Use |
| --- | --- | --- |
| Container | `::: name {attributes}` … `:::` | Blocks with content. A `:::` alone on a line closes the last open block. |
| Single line | `:: name value {attributes}` | Blocks without content, such as icons and spacers. |

Every block accepts the [style attributes](/en/reference/attributes/#style-attributes). Container blocks can be nested freely, and indentation is optional.

## Summary

| Block | Form | Children | Own attributes |
| --- | --- | --- | --- |
| [`columns`](#columns) | Container | Each child is a column | `widths` |
| [`stack`](#stack) | Container | Stacked vertically | |
| [`row`](#row) | Container | Side by side, horizontally | `wrap` |
| [`grid`](#grid) | Container | Each child is a cell | `cols` |
| [`box`](#box) | Container | Stacked inside the box | |
| [`steps`](#steps) | Container | Revealed one at a time | |
| [`notes`](#notes) | Container | Plain text of the notes | |
| [`chart`](#chart) | Container or one line | A table with the data, or `data=$list` | `type`, `legend`, `stacked`, `values`, `colors`, `data`, `label`, `series` |
| [`table`](#table) | One line | None; data comes from `data=$list` | `data`, `columns` |
| [`if` and `else`](#if-and-else) | Container | The content shown based on the condition | |
| [`for`](#for) | Container | The content repeated for each item | |
| [`slot`](#slot) | Container | Content of a named slot | |
| [`icon`](#icon) | Single line | None; the value is the icon name | |
| [`spacer`](#spacer) | Single line | None; the value is the size | |
| [`divider`](#divider) | Single line | None | |
| [`input`](#input) | Single line | None | `bind`, `placeholder`, `on-submit` |
| [`sound`](#sound) | Single line | None; the value is the audio address | `name`, `autoplay`, `loop`, `volume` |

`icon`, `spacer` and `divider` do not accept content ([R109](/en/reference/errors/)). On the other built-in blocks, text after the name is ignored ([R110](/en/reference/errors/)); use attributes in braces.

## columns

Places its children side by side. **Each direct child is a column**: a paragraph, an image or, to group several elements in one column, a `::: stack`.

| Attribute | Values | Default |
| --- | --- | --- |
| `widths` | Proportions separated by spaces, one per column: `widths="2 1"` | Equal columns |
| `gap` | Space between columns | `md` (32 px) |
| `valign` | Vertical alignment of the columns | `center` |

```ribalta preview
::: columns {widths="2 1" gap=xl}
::: stack {gap=sm}
## Expansion plan
Two new regions by December,
with the same support team.
:::
::: box {.card align=center}
:: icon map-pin {color=accent}
2 regions
:::
:::
```

## stack

Stacks its children vertically. It is the block used to group several elements inside a column or a cell.

| Attribute | Use | Default |
| --- | --- | --- |
| `gap` | Space between children | `md` (32 px) |
| `align` | Horizontal alignment of the children | They take the full width |
| `valign` | Vertical alignment of the children | `top` |

```ribalta preview
::: stack {gap=xs align=center}
Chapter 1 {.kicker}
# Where it all began
A story in three acts {.muted}
:::
```

## row

Places its children in a horizontal line, each at the size of its own content. Useful for icons, badges and short lists of items.

| Attribute | Values | Default |
| --- | --- | --- |
| `wrap` | Boolean: wraps the items onto more lines when they do not fit | Off |
| `gap` | Space between items | 16 px |
| `align` | Horizontal distribution: `left`, `center`, `right` | `left` |
| `valign` | Vertical alignment of the items | `center` |

```ribalta preview
::: row {gap=md align=center}
:: icon star {size=lg color=yellow}
:: icon star {size=lg color=yellow}
:: icon star {size=lg color=yellow}
4.8 out of 5 from 1,200 reviews {size=xl}
:::
```

## grid

Grid with a fixed number of columns. Each child takes one cell, filling rows from left to right.

| Attribute | Values | Default |
| --- | --- | --- |
| `cols` | Number of columns, from 1 to 12 | Number of children, up to 3 |
| `gap` | Space between cells | `md` (32 px) |
| `valign` | Vertical alignment of the cells | Stretched to the same height |

```ribalta preview
::: grid {cols=3 gap=lg}
::: box {.card}
### Fast
Preview on every keystroke.
:::
::: box {.card}
### Consistent
Reusable themes and styles.
:::
::: box {.card}
### Versioned
History of every change.
:::
:::
```

## box

A box that stacks its children vertically and takes a background, border, corners and spacing. Combine it with the `.card` style (`bg=surface radius=lg pad=lg shadow=md`) or build the look with attributes.

| Attribute | Use | Default |
| --- | --- | --- |
| `gap` | Space between children | 20 px |
| `bg`, `border`, `radius`, `pad`, `shadow` | Look of the box | No background, no border |

```ribalta preview
::: columns {gap=lg}
::: box {.card}
### With `.card`
Background, corners and shadow included.
:::
::: box {border="3 accent" radius=md pad=lg}
### With attributes
Just a border and spacing.
:::
:::
```

## steps

Reveals its children one at a time while the document is shown: each child is a step. If a child is a list, **each item** of the list becomes a step. In a static rendering of the page, everything shows at once.

It accepts the same attributes as `stack`. To control the order by hand, use `{step=N}` on any block or span. See [Reveal](/en/guides/reveal/).

```ribalta preview
## Three reasons
::: steps {gap=sm}
- Less time formatting
- More time on content
- Consistent documents
:::
```

## notes

Notes for whoever shows the document, such as speaker notes in a presentation. The content is never drawn on the page; the program that shows the document decides where to display it.

- The text is kept as is, without interpreting formatting or blocks.
- Several `notes` blocks on the same page are joined, separated by a blank line.
- Attributes on the `::: notes` line are ignored.

```ribalta
# Results
We grew 38% this quarter.

::: notes
Pause after the number.
Ask if anyone has questions.
:::
```

## chart

Draws a chart from a Markdown table inside the block. The first column holds the labels; each following column is a series, and its header gives the series name. Cells accept numbers such as `42`, `3.5`, `3,5` and `12%`.

| Attribute | Values | Default |
| --- | --- | --- |
| `type` | `bar`, `line`, `area`, `pie`, `donut` | `bar` |
| `legend` | Boolean | On with more than one series or for `pie`/`donut` |
| `stacked` | Boolean: stacks the series | Off |
| `values` | Boolean: shows the values | Off |
| `colors` | Colors separated by spaces, one per series: `colors="accent blue"` | Theme palette |

```ribalta preview
::: chart {type=bar values}
| Month | 2025 | 2026 |
| --- | ---: | ---: |
| Jan | 32 | 41 |
| Feb | 28 | 45 |
| Mar | 35 | 52 |
:::
```

With `data=$list`, the data comes from a `@data` table and the block can be a single line: `:: chart {data=$sales label=month series="revenue cost"}`.

More examples in [Charts](/en/guides/charts/).

## table

Builds a table from a list of records. `columns` picks the fields and their order; columns with only numbers are right-aligned.

```ribalta preview
@data team
| name  | role   | years |
| ---   | ---    | ---   |
| Ana   | CTO    | 7     |
| Bruno | Design | 3     |
@end

:: table {data=$team columns="name years"}
```

For a hand-written table, use the Markdown syntax `| A | B |`.

## if and else

`::: if condition` shows the content when the condition is true. Right after it can come `::: else if condition` and `::: else`. The chosen content goes into the parent block, with no extra box.

```ribalta preview
@let version = short

::: if $version == short
Summary in one sentence.
:::
::: else
Full version, with every detail.
:::
```

The `if=condition` attribute does the same for a single element. With `@state`, the choice happens while the document is shown; see [Interactivity](/en/guides/interactive/).

## for

`::: for item in $list` repeats the content for each item, and the generated blocks go into the parent block. `::: for item, i in $list` also gives the position (from 1) and `::: for i in 1..5` repeats over a range.

```ribalta preview
::: row {gap=md}
::: for n in 1..5
:: icon star {color="$($n <= 4 ? accent : muted)"}
:::
:::
```

Markdown lists generated by a `for` are merged into a single list. See [Variables, data and repetition](/en/guides/data/).

## slot

Fills a named slot of a component. It can only be used directly inside the use of a container component ([R416](/en/reference/errors/) anywhere else). The slot name goes after `slot`.

```ribalta preview
@component compare {before after}
::: columns {gap=xl}
::: box {.card}
### $before
$content
:::
::: box {.card}
### $after
$side-b
:::
:::
@end

::: compare {before=Spreadsheets after="Live dashboard"}
Export, paste and format by hand.
::: slot side-b
The data arrives on its own.
:::
:::
```

See [Components](/en/guides/components/) for the details of `$content` and named slots.

## icon

Draws an icon from the built-in library. The name goes after `icon`; the full list is in [Icons](/en/reference/icons/).

| Attribute | Use | Default |
| --- | --- | --- |
| `size` | Icon size: `xs`…`7xl` scale or pixels | 96 px |
| `color` | Stroke color | Text color |

```ribalta preview
::: row {gap=xl align=center}
:: icon rocket {size=3xl color=accent}
:: icon lightbulb {size=3xl color=yellow}
:: icon chart-line {size=3xl color=blue}
:::
```

## spacer

Empty space between two blocks. The value is a size from the [spacing scale](/en/reference/attributes/#spacing) (`none` to `3xl`) or pixels from 0 to 600. With no value, it is `md` (32 px).

```ribalta preview
# Before the space
:: spacer 3xl
After 160 px {.muted}
```

## divider

Horizontal divider line. It accepts style attributes, such as `w` to limit its width and `opacity`.

```ribalta preview
## Section
:: divider {w=40%}
Text after the line {.muted}
```

## input

Text field bound to a `@state`. What the person types goes straight into the variable, so the text around it changes right away. Bound to a number `@state`, the field only takes numbers.

| Attribute | Use | Default |
| --- | --- | --- |
| `bind` | The variable that holds the value, such as `bind=$name` (required) | |
| `placeholder` | Example text shown while the field is empty | None |
| `on-submit` | Actions when Enter is pressed, `go` included | None |

```ribalta preview
@state name = ""

## What is your name?
:: input {bind=$name placeholder="Your name"}

Hello, $name! {if="$name != \"\""}
```

Typing in the field does not change the page. In a static rendering, the field shows its initial value. See [Text input](/en/guides/interactive/#text-input).

## sound

A sound on the page. It is not drawn on the page: it plays with `autoplay` when the page opens, or when a `play` action asks for it. Every sound stops when you leave the page, and nothing plays in a static rendering.

| Attribute | Use | Default |
| --- | --- | --- |
| `name` | Name used by `play` and `stop`, such as `name=door` | None |
| `autoplay` | Plays as soon as the page opens | Off |
| `loop` | Repeats forever | Off |
| `volume` | From 0 to 1, or a percentage | `1` |

```ribalta
:: sound https://example.com/rain.mp3 {name=rain loop volume=40%}
:: sound https://example.com/door.mp3 {name=door}
# The door
[Open]{click="play door"} · [Silence]{click="stop rain"}
```

The address starts with `https://`. See [Sound](/en/guides/interactive/#sound).
