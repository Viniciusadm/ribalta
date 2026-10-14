---
title: Syntax
description: A quick reference to the whole Ribalta language, from document structure to escapes.
---

This page sums up the whole language in short tables. For step-by-step explanations, see the [guides](/en/guides/writing/); for the values each attribute accepts, see [Attributes](/en/reference/attributes/).

## Document structure

A document has two parts, in this order:

1. **Preamble** (optional): lines that start with `@`, only at the top of the file.
2. **Pages**: everything else. The first content that is not a definition starts page 1, and each `---` starts a new page.

```ribalta
@title Quarterly results
@theme paper

# First page

--- {.dark align=center}
# Second page
```

### Preamble definitions

| Definition | Form | Effect |
| --- | --- | --- |
| `@title` | `@title Text` | Document title. |
| `@author` | `@author Name` | Document author. |
| `@theme` | `@theme paper` | Built-in theme: `paper` (default), `ink` or `editorial`. |
| `@version` | `@version 2` | Language version: `1` or `2`. The current version is `2`. |
| `@page` | `@page 1920x1080` | Page size in pixels: `@page 1920x1080`, `@page 800xauto` or `@page width=800 height=auto`. A height of `auto` grows with the content. Without it, the program showing the document picks the size. See [Page sizes](/en/guides/pages/). |
| `@color` | `@color brand #ff6b4a` | Creates or replaces a color token. |
| `@font` | `@font title "Bebas Neue" 400 700` | Creates or replaces a font token (a Google Fonts family and weights from 100 to 900). |
| `@style` | `@style highlight {color=brand weight=bold}` | Creates a reusable style, applied with `.highlight`. |
| `@component` … `@end` | `@component name {param param=default}` | Defines a component. The body runs until the `@end` line. |
| `@let` | `@let spacing = lg` | Creates a fixed variable, used as `$spacing`. See [Variables](/en/guides/data/). |
| `@data` … `@end` | `@data team` followed by a table | Creates a list of records from the table. |
| `@state` | `@state details = false` | Creates a variable that changes while the document is shown. See [Interactivity](/en/guides/interactive/). |
| `@transition` | `@transition fade` | Default transition when entering pages: `fade`, `slide`, `zoom` or `none`. |
| `@extern` | `@extern score = 0` | Creates a variable that the program showing the document sets, such as the score of a game. See [Values from outside](/en/guides/interactive/#values-from-outside-with-extern). |
| `@persist` | `@persist endings coins` | Keeps these `@state` variables in the browser, so progress carries on after the document is closed. See [Saved progress](/en/guides/interactive/#saved-progress). |
| `@scorm` | `@scorm score=$points complete=#end` | Score and completion reported to an LMS by the SCORM package. See [Score and completion in an LMS](/en/guides/interactive/#score-and-completion-in-an-lms-scorm). |
| `@mode` | `@mode game` | `default` (default) or `game`: in `game`, the arrows do not leave pages and the counter is hidden. See [Interactivity](/en/guides/interactive/#locked-pages-and-game-mode). |
| `@header` … `@end` | `@header {attributes}` | Header repeated on every page except the cover. See [Header, footer and overlay](/en/guides/header-footer/). |
| `@footer` … `@end` | `@footer {attributes}` | Footer repeated on every page except the cover. `@footer none` turns off the base theme's footer. |
| `@overlay` … `@end` | `@overlay {attributes}` | Free layer on top of the content, on every page except the cover; blocks with `x`/`y` are placed from the page's corner. `@overlay none` turns off the base theme's overlay. See [Overlay](/en/guides/header-footer/#overlay-overlay). |

Color, font, style and component names use lowercase letters, numbers and hyphens, starting with a letter. Blank lines and comments may appear between definitions. A definition after the first page is an error ([R103](/en/reference/errors/)).

### Page separator

| Form | Effect |
| --- | --- |
| `---` | Starts a new page. Three or more hyphens, alone on the line. |
| `--- {attributes}` | Starts a new page and applies attributes to the whole page. |
| `--- {#id}` | Gives the page an id, used by internal links such as `[back](#id)` and by `click="go #id"`. |
| `--- {hidden}` | Takes the page out of the normal flow; it is only reached by link or `go`. |
| `--- {next=#id}` | Replaces the following page when advancing. |
| `--- {width=1080 height=1920}` | Gives this page its own size; `height=auto` makes it grow with the content. See [Page sizes](/en/guides/pages/). |

Right after the separator, `@let` and `@state` lines create variables that only exist in that page.

- If the document starts with `---` right after the preamble, that separator configures the first page; no empty page is created.
- A `---` inside a code block does not split pages.
- The page title is the first `#` heading of the page; without a heading, it is "Page N". Without `#id`, the page id is `page-N`.

## Supported Markdown

| Construct | Syntax | Notes |
| --- | --- | --- |
| Headings | `# Heading` … `###### Heading` | Six levels. A space after the `#` signs is required. |
| Paragraph | Consecutive lines of text | Every line break becomes a line break on the page. A blank line separates paragraphs. |
| Bold | `**text**` | Asterisks only. |
| Italic | `*text*` | Asterisks only; `_text_` is not italic. |
| Strikethrough | `~~text~~` | |
| Inline code | `` `code` `` | The content is not interpreted. |
| Link | `[text](https://…)` | Accepts `https://`, `http://`, `mailto:` and `#page-id`. |
| Span with attributes | `[text]{color=accent}` | The braces go right after the `]`. |
| Inline image | `![description](https://…)` | Inside a paragraph; accepts attached attributes: `![x](…){h=60}`. |
| Block image | `![description](https://…){fit=cover}` | Alone on its line. Accepts `https://`, `http://`, `data:image/…` and `asset:<id>`. |
| List | `- item`, `* item` or `+ item` | Nest items with indentation. Indented lines continue the item. |
| Numbered list | `1. item` or `1) item` | The first number sets where counting starts. |
| Quote | `> text` | May contain other blocks, such as lists and headings. |
| Code block | ` ```js ` … ` ``` ` | The language is optional; attributes may go at the end of the opening line. |
| Table | `\| A \| B \|` + `\| --- \| --- \|` | The second line is required. `:---`, `:---:` and `---:` align the column. |

HTML is not interpreted: `<b>` shows up as text on the page.

## Attributes

Attributes go in braces and configure the block, span or page they belong to.

| Form | Example | Meaning |
| --- | --- | --- |
| `.style` | `{.card}` | Applies a built-in style, a `@style` or a color token (`{.coral}` is the same as `{color=coral}`). |
| `#id` | `--- {#summary}` | Page id on the separator, or a block anchor at the end of the block's opening line (`Text. {#light-phase}`). Links such as `[see](#light-phase)` and `go #light-phase` lead to it. A block id cannot repeat or match a page id. |
| `key=value` | `{size=5xl}` | Value without spaces. |
| `key="value"` | `{pad="sm lg"}` | Value with spaces, in double or single quotes. Use `\"` for a quote inside the value. |
| `key` | `{italic}` | Turns a boolean attribute on. To turn it off: `italic=false`. |
| `.style=condition` | `{.highlight=$active}` | Applies the style only when the condition is true. With `hover`, while the mouse is over the element. |

Boolean values accept `true`, `yes` and `sim` to turn on, and `false`, `no` and `não` to turn off.

### Where attributes go

| Where | Position | Example |
| --- | --- | --- |
| Headings, paragraphs, list items, opening line of code | At the end of the line, **with a space before** | `# Heading {size=5xl}` |
| `:::` and `::` blocks | At the end of the block line | `::: box {.card}` |
| Page separator | After `---` | `--- {bg=ink color=white}` |
| Text span | **Attached** to the `]` | `[38%]{color=accent}` |
| Image | **Attached** to the `)` | `![Photo](https://…){fit=cover}` |

In a multi-line paragraph, attributes at the end of a line apply to the whole paragraph and end it. Styles (`.style`) are applied first, and `key=value` pairs written on the block itself override them.

## Blocks

| Form | Use |
| --- | --- |
| `::: name {attributes}` … `:::` | Container block. A `:::` alone on a line closes the last open block. |
| `:: name value {attributes}` | Single-line block, with no content. |

Blocks can be nested. Indentation is optional and only helps readability.

```ribalta
::: columns {widths="2 1"}
::: stack
## Left
Text
:::
:: icon rocket {size=3xl color=accent}
:::
```

| Block | Form | Purpose |
| --- | --- | --- |
| `columns` | `:::` | Side-by-side columns; each child is a column. |
| `stack` | `:::` | Stacks its children vertically. |
| `row` | `:::` | Lays its children out horizontally. |
| `grid` | `:::` | Grid with a fixed number of columns. |
| `box` | `:::` | Box with background, border and spacing. |
| `steps` | `:::` | Reveals each child, or each list item, one at a time. |
| `notes` | `:::` | Notes for whoever shows the document; not drawn on the page. |
| `chart` | `:::` or `::` | Chart built from a table or from `data=$list`. |
| `table` | `::` | Table built from `data=$list`. |
| `if` / `else` | `:::` | Shows the content based on a condition. |
| `for` | `:::` | Repeats the content for each item of a list. |
| `slot` | `:::` | Fills a named slot of a component. |
| `icon` | `::` | Icon: `:: icon rocket`. |
| `spacer` | `::` | Empty space: `:: spacer lg`. |
| `divider` | `::` | Divider line. |
| `input` | `::` | Text field bound to a `@state`: `:: input {bind=$name}`. |
| `sound` | `::` | Sound played by `play` and `stop`: `:: sound https://… {name=door}`. |

Details for each block in [Blocks](/en/reference/blocks/).

## Components

| Form | Meaning |
| --- | --- |
| `@component name {a b=default}` | Defines the component `name` with the required parameter `a` and the parameter `b` with a default value. |
| `@end` | Ends the definition. |
| `:: name {a=value}` | Uses the component without content. |
| `::: name {a=value}` … `:::` | Uses the component with content, which goes to `$content`. |
| `::: slot side` … `:::` | Inside a component use, fills the named slot `$side`. |
| `{step=N}` on the use | Reveals the whole component at step `N`. |
| `{if=… click=…}` on the use | Shows the component based on the condition and sets click actions. |
| `{kind=a\|b\|c}` in the definition | Parameter with accepted values; the first one is the default. |

Components only take named parameters: `.style`, `#id` and loose text after the name are errors. Components cannot use themselves, directly or indirectly. A parameter can take a whole record or list: `:: card {p=$person}`.

## `$` placeholders

| Placeholder | Where | Effect |
| --- | --- | --- |
| `$param` | In a component body, in text or an attribute value | Replaced by the parameter value: `## $title`, `{color=$tone}`. |
| `$content` | Alone on a line of the body | Receives the content passed between `::: name` and `:::`. |
| `$name` | Alone on a line of the body, when `name` is not a parameter | Named slot, filled with `::: slot name`. It stays empty if not filled. |
| `$page`, `$pages`, `$title`, `$author`, `$date` | Inside `@header`, `@footer` and `@overlay` | Page number, total pages, title, author and today's date. Any other name is an error ([R522](/en/reference/errors/)). |
| `$variable`, `$item.field` | Anywhere | Value of a `@let`, `@data`, `@state`, `for` variable or record field. |
| `$( expression )` | Anywhere | Result of an expression, such as `$($price * 12)`. |
| `$step` | Anywhere | Current step of the page. |
| `$elapsed` | Anywhere | Seconds since the page opened. See [Time](/en/guides/interactive/#time). |

A `$word` that is not a declared variable is plain text, so `US$ 10` stays the same. The exception is `$content` alone on a line outside a component, which is an error ([R401](/en/reference/errors/)). For a literal `$` before a declared name, write `\$name`.

## Expressions

Expressions appear in `$( … )`, in `@let`, in `if` and `.style=` conditions, in `for` lists and in `click` actions.

| Element | Examples |
| --- | --- |
| Values | `42`, `3.5`, `"with spaces"`, `word`, `true`, `false`, `[a, b, c]`, `1..5` |
| Variables | `$total`, `$person.name`, `$sales.revenue` (whole column) |
| Operators | `+ - * / %`, `== != < <= > >=`, `in`, `and or not`, `condition ? a : b` |
| Functions | `len`, `min`, `max`, `sum`, `round`; in actions, `random` and `pick` |

A word without `$` is text. A hyphen followed by a letter is part of the name (`$short-title`); to subtract one variable from another, use spaces: `$a - $b`.

## Click actions

| Action | Effect |
| --- | --- |
| `$x = expression` | Changes a `@state`. |
| `toggle $x` | Flips a true/false `@state`. |
| `add $x value`, `remove $x value` | Puts a text into a `@state` list or takes it out. |
| `clear $x` | Empties a `@state` list. |
| `play name`, `stop name` | Plays or stops a `:: sound` on the page. |
| `go #id` | Goes to the page `id`. |
| `go back` | Returns to the position before the last jump. |
| `go next`, `go prev` | Moves one step forward or back. |

Separate several actions with `;`: `click="$plan = pro; go #detail"`.

## Comments

A line that starts with `//` (with or without leading spaces) is ignored, both in the preamble and in pages.

```ribalta
// Check these numbers before the meeting
# Results
```

- `//` in the middle of a line is plain text.
- Inside code blocks and `::: notes`, lines with `//` are kept.

## Escapes

| Write | To get |
| --- | --- |
| `\*`, `\~`, `` \` ``, `\[`, `\]`, `\!` | The literal character, with no formatting. |
| `\{` | A literal brace; keeps a trailing `{…}` from becoming attributes. |
| `\\|` | A vertical bar inside a table cell. |
| `\#` at the start of a line | A literal `#`, not a heading. |
| `\"` inside a quoted value | A quote inside the value. |

A backslash escapes any ASCII punctuation mark. Before other characters, it shows up as is.

## Lesson profile

A lesson is a text-only document. The `lesson` profile rejects what belongs to slides and games: images from `http://` or `https://` addresses (stored `asset:` images still work), `sound`, `input`, `@persist` and the `next` and `hidden` attributes on pages. Each use becomes the error [R212](errors.md#r212--feature-outside-the-lesson-profile).

```bash
ribalta check lesson.ribalta --profile lesson
```

When compiling through the library, pick the profile in the options: `{"profile": "lesson"}`, next to `page`. Without a profile nothing changes. `ribalta graph --format json` also lists the document's anchors in `anchors`, each with its `id` and the `page` it is on.

## Complete example

```ribalta preview
// Preamble: definitions only, at the top of the file
@title Quarterly report
@theme paper
@color brand #2f6fdb
@style highlight {color=brand weight=bold}
@component metric {value label tone=accent}
::: box {.card align=center}
$value {size=4xl weight=black color=$tone}
$label {.muted}
:::
@end

// Page 1: the first content starts the page
Quarterly report {.kicker}
# Revenue grew [38%]{.highlight} {size=5xl}
New customers, better retention
and a bigger team. {.lead}

// Page 2: separator with an id and page attributes
--- {#numbers align=center}
## Numbers that matter
::: grid {cols=3 gap=lg}
:: metric {value=+38% label=Revenue}
:: metric {value=12k label=Customers tone=green}
:: metric {value=4.8 label=Rating tone=blue}
:::

::: notes
Point out the customer rating.
:::

// Page 3: gradient background and step-by-step reveal
--- {bg=ink..violet color=white}
## Next steps
::: steps
- Open two new regions
- Launch the annual plan
- Hire the support team
:::
Details on the [numbers page](#numbers). {opacity=0.7}
```
