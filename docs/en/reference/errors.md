---
title: Error messages
description: Every diagnostic code the compiler can emit, with its cause, an example and the fix.
---

The Ribalta compiler never stops at the first problem: it always produces the best document it can, plus a list of diagnostics. Each diagnostic has:

| Part | Example |
| --- | --- |
| Severity | **Error** (something was dropped or does not work) or **warning** (it works, but probably not the way you wanted). |
| Code | `R209`. The code is stable and does not change between versions. |
| Position | Line and column of the problem. |
| Message | unknown attribute: `colr` |
| Hint | Did you mean `color`? |

Errors inside a component are reported on the line where the component is **used**, with the suffix "(in component `name`)". Errors in a `@style` are reported once, at the definition, not at every use.

| Family | Topic |
| --- | --- |
| [F1xx](#f1xx-structure) | Structure: blocks, tables, code, charts and icons. |
| [F2xx](#f2xx-attributes) | Attributes: brace syntax, styles and attribute names. |
| [F3xx](#f3xx-values) | Attribute values. |
| [F4xx](#f4xx-components) | Components and named slots. |
| [F5xx](#f5xx-definitions-and-base-theme) | Preamble definitions and the base theme. |
| [F6xx](#f6xx-variables-expressions-and-state) | Variables, data, expressions, repetition and state. |
| [F7xx](#f7xx-navigation) | Navigation between pages. |

## F1xx: structure

### R101 · Closing fence with no open block

**Error.** Message: `:::` closes a block, but no block is open here.

A `:::` alone on a line closes the last open block. Here there is no open block, usually because of an extra `:::`.

```ribalta
# Results
We grew 38%.
:::
```

**How to fix:** remove the line or open a block before it, such as `::: columns`.

### R102 · Unclosed block

**Error.** Message: block `columns` was opened but never closed.

Every block opened with `::: name` needs a `:::` line that closes it. Each page is read on its own, so the block must be closed before the next `---`.

```ribalta
::: columns
Left
Right
```

**How to fix:** add a line containing only `:::` at the end of the block.

### R103 · Definition after the first page

**Error.** Message: `@theme` can only be used at the start of the document, before the first page.

Definitions with `@` (`@title`, `@theme`, `@color`, `@style`, `@component` and the others) only work in the preamble. Once the first page starts, they are dropped.

```ribalta
# Opening

---
@theme ink
# Second page
```

**How to fix:** move the definition to the top of the file, before any content.

### R104 · Block without a name

**Error.** Message: missing block name after `:::`.

A line that starts with `:::` or `::` and has other content needs the block name right after it.

```ribalta
::: {.card}
Text
:::
```

**How to fix:** write the block name before the attributes, as in `::: box {.card}`.

### R105 · Unclosed code block

**Error.** Message: unclosed code block.

A code block opened with ` ``` ` needs another ` ``` ` line to end it. Without it, the rest of the page becomes code.

````ribalta
```js
console.log("hi")
````

**How to fix:** close the block with a line containing ` ``` `.

### R106 · Table without a separator line

**Error.** Message: table without a separator line after the header.

The second line of a table must be the separator line, which divides the header from the data.

```ribalta
| Product | Price |
| Coffee | 12 |
```

**How to fix:** add a line like `| --- | --- |` right below the header.

### R107 · Table row with a different number of columns

**Warning.** Message: this row has 3 columns, but the header has 2.

A table row has more or fewer cells than the header.

```ribalta
| Product | Price |
| --- | --- |
| Coffee | 12 | extra |
```

**How to fix:** give every row the same number of cells as the header. To write `|` inside a cell, use `\|`.

### R108 · Unknown block

**Error.** Message: unknown block: `colums`.

The name after `:::` or `::` is neither a built-in block nor a component defined in the document or the base theme. The hint suggests the closest name.

```ribalta
::: colums
Text
:::
```

**How to fix:** fix the name (here, `columns`) or define the component with `@component`. See the list in [Blocks](/en/reference/blocks/).

### R109 · Single-line block with content

**Warning.** Message: `divider` does not accept content; use `:: divider` on a single line.

`icon`, `spacer` and `divider` have no content. When opened with `:::`, the content between them and the closing `:::` is ignored.

```ribalta
::: divider
Text
:::
```

**How to fix:** use the single-line form, `:: divider`, and put the text outside the block.

### R110 · Text ignored after the block name

**Warning.** Message: the text `3` after `grid` will be ignored.

Only `icon` and `spacer` take a value after the name. On the other built-in blocks, loose text is ignored.

```ribalta
::: grid 3
A
B
C
:::
```

**How to fix:** use attributes in braces, as in `::: grid {cols=3}`.

### R111 · Chart without a table

**Error.** Message: the chart needs a table with the data.

The `chart` block draws the data of a Markdown table placed inside it. Without a table, the chart is not drawn.

```ribalta
::: chart {type=bar}
Sales per month
:::
```

**How to fix:** put a table inside the block. The first column holds the labels and the others are the series. See [Charts](/en/guides/charts/).

### R112 · Chart value is not a number

**Error.** Message: `twelve` is not a number.

Every data cell of a chart, except the first column, must be a number. The invalid cell counts as 0.

```ribalta
::: chart
| Month | Sales |
| --- | ---: |
| Jan | twelve |
:::
```

**How to fix:** use numbers such as `42`, `3.5` or `3,5`. A trailing `%` is accepted.

### R113 · Chart without series

**Error.** Message: the chart table needs at least one column of values.

The table only has the label column, so there is nothing to draw.

```ribalta
::: chart
| Month |
| --- |
| Jan |
:::
```

**How to fix:** add at least one column of values to the table.

### R114 · Icon without a name

**Error.** Message: provide the icon name.

```ribalta
:: icon {size=3xl}
```

**How to fix:** write the name after `icon`, as in `:: icon rocket {size=3xl}`. The names are listed in [Icons](/en/reference/icons/).

### R115 · Unknown icon

**Error.** Message: unknown icon: `rockets`.

The name is not in the list of built-in icons. When there is a similar name, the hint suggests it.

```ribalta
:: icon rockets
```

**How to fix:** use a name from the list in [Icons](/en/reference/icons/), such as `rocket`.

### R116 · Field without a variable

**Error.** Message: `input` needs `bind` with the variable that holds the text.

```ribalta
@state name = ""

# Who are you?
:: input {placeholder="Your name"}
```

**How to fix:** bind the field to a text or number `@state` with `bind`: `:: input {bind=$name placeholder="Your name"}`. The same error appears when `bind` does not start with `$`.

### R117 · Sound without an address

**Error.** Message: `sound` needs the address of the audio file.

```ribalta
# The door
:: sound {name=door}
```

**How to fix:** write the address after `sound`: `:: sound https://example.com/door.mp3 {name=door}`.

## F2xx: attributes

### R201 · Dot or hash without a name

**Error.** Message: `.` must be followed by a name.

Inside braces, `.` must be attached to a style name and `#` to an id.

```ribalta
Text {. color=accent}
```

**How to fix:** write the name right after it, as in `{.highlight}` or `{#opening}`, or remove the loose character.

### R202 · Unclosed quotes

**Error.** Message: unclosed quotes in attribute value.

A value that starts with a quote must end with the same quote.

```ribalta
![Photo](https://example.com/photo.jpg){fit="cover}
```

**How to fix:** close the value with the quote that opened it: `{fit="cover"}`. At the end of headings, paragraphs and blocks, braces with an open quote are not recognized as attributes and show up as text on the page.

### R203 · Attribute without a value

**Error.** Message: attribute `color` has no value after `=`.

There is an `=` with nothing after it. It usually comes with an [R301](#r301--invalid-value), because the empty value is also invalid.

```ribalta
# Heading {color=}
```

**How to fix:** provide the value (`{color=accent}`) or remove the attribute.

### R204 · Invalid text in attributes

**Error.** Message: invalid text in attributes: `@highlight`.

Inside braces, only `.style`, `#id`, `key=value`, `key="value with spaces"` and `key` are accepted.

```ribalta
Text {@highlight}
```

**How to fix:** use the right form, here `{.highlight}`. Attribute names start with a letter.

### R205 · Id outside a page or block

**Warning.** Message: `#id` only works on the page separator and at the end of blocks.

Ids identify pages and blocks, for links such as `[back](#opening)`. On inline spans, the id is ignored.

```ribalta
Text [highlight]{#start}
```

**How to fix:** put the id on the page separator, as in `--- {#start}`, or at the end of the block's line.

### R206 · Style that references itself

**Error.** Message: style `.loop` references itself.

A `@style` includes itself, directly or through other styles. Since the style does not exist yet while its own definition is read, the compiler also shows an [R207](#r207--unknown-style) at the same spot.

```ribalta
@style loop {.loop color=accent}

Text {.loop}
```

**How to fix:** remove the circular reference. A style can only include styles defined before it.

### R207 · Unknown style

**Error.** Message: unknown style: `.highlite`.

The name after the `.` is not a built-in style, a `@style` or a color token. The hint suggests the closest name.

```ribalta
Text {.highlite}
```

**How to fix:** fix the name or declare the style at the start of the document with `@style name {attributes}`. See the [built-in styles](/en/reference/themes/#built-in-styles).

### R208 · Repeated page id

**Warning.** Message: another page already uses the id `#summary`.

Two pages have the same id, so a `#summary` link can only lead to one of them.

```ribalta
--- {#summary}
# Summary

--- {#summary}
# Conclusion
```

**How to fix:** give each page a different id.

### R209 · Unknown attribute

**Error.** Message: unknown attribute: `colr`.

The name is neither a style attribute nor an attribute of the block it appears on. The hint suggests the closest name or lists the attributes accepted there.

```ribalta
# Heading {colr=coral}
```

**How to fix:** fix the name. Block attributes, such as `cols` or `fit`, only work on the right blocks. See [Attributes](/en/reference/attributes/).

## F3xx: values

### R210 · Key without a click

**Warning.** Message: `key` does nothing without `click`.

The key runs the element's `click` actions; without them, pressing the key changes nothing.

```ribalta
# Map
[North]{key=up}
```

**How to fix:** say what the key does with `click`: `[North]{click="$y = $y - 1" key=up}`.

### R211 · Repeated block id

**Error.** Message: another block already uses the id `#phase`, or the id `#phase` already belongs to a page.

A block id (`{#phase}` at the end of the opening line) must lead to a single place. It cannot repeat or equal a page id.

```ribalta
First {#phase}

Second {#phase}
```

**How to fix:** give each block a different id.

### R212 · Feature outside the lesson profile

**Error.** Only appears with the `lesson` profile (`ribalta check --profile lesson` or `"profile": "lesson"` in the options). Message: `sound` is not allowed in lessons.

A lesson is text only. The profile rejects images from `http://` or `https://` addresses, `sound`, `input`, `@persist` and the `next` and `hidden` page attributes.

```ribalta
# Photosynthesis

![Chloroplast](https://example.com/chloroplast.png)
```

**How to fix:** store the image in the program you write in (`asset:`) or remove the feature.

### R213 · Unknown profile

**Error.** Only appears when the program that compiles passes a nonexistent profile in the options. Message: unknown profile: `class`.

The other options, such as `page`, still apply and the document is compiled without a profile.

**How to fix:** use `"profile": "lesson"` or omit the field.

### R301 · Invalid value

**Error.** Message: depends on the attribute, for example invalid size: `huge` or unknown color: `corall`.

The attribute exists, but the value is not accepted. The attribute is ignored, and the hint says which values are valid.

```ribalta
# Heading {size=huge color=corall}
```

Common cases:

| Case | Invalid example | Valid example |
| --- | --- | --- |
| Unknown color or malformed hex | `color=corall`, `color=#12` | `color=coral`, `color=#123` |
| Color opacity outside 0 to 100 | `color=accent/150` | `color=accent/40` |
| Name outside the scale or number out of range | `size=huge`, `pad=900` | `size=5xl`, `pad=xl` |
| Font not declared | `font=title` | `font=display` or `@font title …` |
| Invalid weight | `weight=450` | `weight=500` |
| Invalid boolean | `italic=maybe` | `italic` or `italic=false` |
| Invalid alignment | `align=middle` | `align=center` |
| Invalid shadow | `shadow=xl` | `shadow=lg` |
| Border wider than 40 | `border="80 coral"` | `border="4 coral"` |
| Invalid length | `w=200%`, `w=half` | `w=50%`, `w=480` |
| Attribute that needs a value, used alone | `{size}` | `{size=xl}` |
| Address not allowed in a link or image | `[x](javascript:…)`, `![x](file.png)` | `https://…`, `asset:<id>` |
| Invalid image fit | `fit=stretch` | `fit=cover` |
| Grid columns outside 1 to 12 | `cols=20` | `cols=4` |
| Invalid column proportions | `widths="a b"` | `widths="2 1"` |
| Invalid chart type | `type=radar` | `type=line` |
| Invalid `spacer` size | `:: spacer huge` | `:: spacer 3xl` |
| Step outside 1 to 99 | `step=0` | `step=1` |

**How to fix:** use one of the values given in the hint. The full scales and ranges are in [Attributes](/en/reference/attributes/).

### R302 · `widths` does not match the columns

**Warning.** Message: `widths` has 3 values, but there are 1 columns.

The number of proportions in `widths` differs from the number of children of the `columns` block.

```ribalta
::: columns {widths="2 1 1"}
Left
:::
```

**How to fix:** give one proportion per column, or adjust the columns. Remember that each direct child is a column; group several elements with `::: stack`.

## F4xx: components

### R401 · `$content` outside a component

**Error.** Message: `$content` can only be used inside a @component.

`$content` marks where the content passed to a component goes, so it only makes sense in the body of a `@component`.

```ribalta
# Heading
$content
```

**How to fix:** remove the line or move it into a component body. To write the literal text, use `\$content`.

### R404 · Component that uses itself

**Error.** Message: component `echo` uses itself.

A component uses itself, directly or through other components. Components have no recursion, so the use is dropped.

```ribalta
@component echo
:: echo
@end

:: echo
```

**How to fix:** remove the circular use from the component body.

### R405 · Components nested too deeply

**Error.** Message: components nested too deeply (limit: 16 levels).

A component uses another, which uses another, and so on, for more than 16 levels. The example below is abbreviated: with all 17 definitions written out, the error shows up on the use of `n1`.

```ribalta
@component n1
:: n2
@end
@component n2
:: n3
@end
// n3 to n16 follow the same pattern, each one using the next
@component n17
End
@end

:: n1
```

**How to fix:** simplify the chain of components by merging intermediate levels.

### R406 · Style or id on a component use

**Error.** Message: components only accept parameters, not .styles or #id.

When using a component, the braces only accept named parameters.

```ribalta
@component badge {text}
::: box {.card}
$text
:::
@end

:: badge {.dark text=New}
```

**How to fix:** wrap the component in a block with the style (`::: box {.dark}` … `:: badge {text=New}` … `:::`) or add a parameter for it.

### R407 · Unknown parameter

**Error.** Message: component `badge` has no parameter `tone`.

The parameter was not declared in the component definition. The hint lists the existing parameters.

```ribalta
@component badge {text}
::: box {.card}
$text
:::
@end

:: badge {text=New tone=accent}
```

**How to fix:** fix the name or declare the parameter, as in `@component badge {text tone=accent}`.

### R408 · Loose text after the component name

**Error.** Message: components take named parameters in braces.

Components do not take positional values: every parameter needs a name.

```ribalta
@component badge {text=New}
::: box {.card}
$text
:::
@end

:: badge Sale
```

**How to fix:** pass the value as a named parameter: `:: badge {text=Sale}`.

### R409 · Missing required parameter

**Error.** Message: missing required parameter `text` of component `badge`.

Parameters declared without a default value are required.

```ribalta
@component badge {text}
::: box {.card}
$text
:::
@end

:: badge
```

**How to fix:** provide the parameter (`:: badge {text=New}`) or give it a default value in the definition (`@component badge {text=New}`).

### R410 · Undeclared `$` placeholder

**Error.** Message: component `badge` uses `$title`, which is not a declared parameter.

The component body uses `$title` inside text or an attribute, but `title` is not in the parameter list.

```ribalta
@component badge {text}
::: box {.card}
## $title
$text
:::
@end

:: badge {text=New}
```

**How to fix:** declare the parameter (`@component badge {text title}`) or fix the name. Alone on a line, `$name` is a named slot and does not cause this error.

When the name only exists where the component is used (a page's `@state` or `@let`, or a `::: for` variable), the message says so: declare the parameter and pass the value, as in `:: badge {text=New title=$title}`. The `@state` and `@let` at the start of the document do not need this.

### R411 · Content ignored by the component

**Warning.** Message: component `badge` does not use `$content`; the content will be ignored.

The component was used with `:::` and received content, but its body has no `$content`.

```ribalta
@component badge {text}
::: box {.card}
$text
:::
@end

::: badge {text=New}
This paragraph is ignored.
:::
```

**How to fix:** add `$content` to the component body or use the single-line form (`:: badge {text=New}`).

### R412 · Built-in block name

**Error.** Message: `box` is already a language block and cannot be redefined.

The names `stack`, `row`, `columns`, `grid`, `box`, `steps`, `notes`, `chart`, `icon`, `spacer`, `divider` and `slot` are reserved.

```ribalta
@component box
Text
@end
```

**How to fix:** pick another name, such as `panel` or `card-box`.

### R413 · Component defined twice

**Error.** Message: component `badge` was defined more than once.

The same document defines two components with the same name. The second definition is dropped.

```ribalta
@component badge
New
@end
@component badge
Sale
@end
```

**How to fix:** remove or rename one of the definitions. Redefining a base theme component in a document is allowed and does not cause this error.

### R414 · Unknown named slot

**Error.** Message: component `pair` has no slot `side-c`.

`::: slot` fills a slot that the component does not declare. The hint suggests the closest name.

```ribalta
@component pair {title}
## $title
$content
$side-b
@end

::: pair {title=Comparison}
Before
::: slot side-c
After
:::
:::
```

**How to fix:** use the name of an existing slot (here, `side-b`) or declare the slot in the component with `$side-c` alone on a line.

### R415 · `::: slot` without a name

**Error.** Message: provide the slot name.

```ribalta
@component pair {title}
## $title
$content
$side-b
@end

::: pair {title=Comparison}
Before
::: slot
After
:::
:::
```

**How to fix:** write the slot name after `slot`, as in `::: slot side-b`.

### R416 · `::: slot` outside a component

**Error.** Message: `::: slot` can only be used inside a component.

`::: slot` only works as a direct child of a container component use. Outside one, the compiler also warns that the name is ignored ([R110](#r110--text-ignored-after-the-block-name)).

```ribalta
::: slot side-b
After
:::
```

**How to fix:** put the `::: slot` inside `::: component` … `:::`, or remove the block.

### R417 · Slot filled twice

**Error.** Message: slot `side-b` was filled more than once.

```ribalta
@component pair {title}
## $title
$content
$side-b
@end

::: pair {title=Comparison}
::: slot side-b
After
:::
::: slot side-b
Again
:::
:::
```

**How to fix:** merge the content into a single `::: slot side-b`.

### R418 · Value outside the parameter list

**Error.** Message: invalid value for `kind`: `primari`.

The parameter declares its accepted values with `|`, and the value used is not one of them. The component uses the first value of the list instead.

```ribalta
@component button {label kind=primary|secondary}
**$label**
@end

:: button {label=Send kind=primari}
```

**How to fix:** use one of the declared values, such as `kind=primary`.

### R419 · Parameter with a reserved name

**Error.** Message: `key` cannot be a parameter: it is already an attribute of the component's usage.

`step`, `if`, `click` and `key` are attributes that any component usage accepts. A parameter with the same name would hide them: `:: button {key=1}` would fill the parameter, and the keyboard shortcut would be lost.

```ribalta
@component button {key text}
**$key** $text
@end

# Menu
:: button {key=1 text=Start}
```

**How to fix:** give the parameter another name, such as `hotkey`: `@component button {hotkey text}`. On usage, `key=1` is the shortcut again.

## F5xx: definitions and base theme

### R501 · Definition without a value

**Error.** Message: `@title` needs a value.

`@title`, `@author`, `@theme`, `@version` and `@page` need a value on the same line.

```ribalta
@title

# Opening
```

**How to fix:** provide the value, as in `@title Annual report`, or remove the line.

### R502 · Incomplete `@color`

**Error.** Message: `@color` expects a name and a value.

```ribalta
@color

# Opening
```

**How to fix:** provide the token name and the color, as in `@color brand #ff6b4a`.

### R503 · Incomplete `@style`

**Error.** Message: `@style` expects a name and attributes.

```ribalta
@style highlight

# Opening
```

**How to fix:** provide the name and the attributes in braces, as in `@style highlight {color=coral weight=bold}`.

### R504 · Block without `@end`

**Error.** Message: `@component` without `@end` (or `@header`, `@footer`, `@overlay`).

Without `@end`, the rest of the document is read as the block body, and the document ends up with no pages.

```ribalta
@component badge {text}
::: box {.card}
$text
:::

# Opening
```

**How to fix:** end the `@component`, `@header`, `@footer` or `@overlay` with an `@end` line.

### R505 · `@end` without an open block

**Error.** Message: `@end` without an open block.

An `@end` line appears with no open `@component`, `@header`, `@footer` or `@overlay` before it.

```ribalta
@end

# Opening
```

**How to fix:** remove the `@end` or add, before it, the `@component`, `@header`, `@footer` or `@overlay` it should close.

### R506 · Unknown definition

**Error.** Message: unknown definition: `@theem`.

The name after `@` is not a language definition. The hint suggests the closest one.

```ribalta
@theem ink

# Opening
```

**How to fix:** use one of the definitions: `@title`, `@author`, `@theme`, `@version`, `@page`, `@color`, `@font`, `@style`, `@component`, `@header`, `@footer`, `@overlay`, `@let`, `@state`, `@extern`, `@data`, `@transition`, `@mode`, `@persist`, `@scorm` or `@end`.

### R507 · Font name without closing quotes

**Error.** Message: font name without closing quotes.

```ribalta
@font title "Bebas Neue 400

# Opening
```

**How to fix:** close the quotes after the family name: `@font title "Bebas Neue" 400`.

### R508 · Invalid font family

**Error.** Message: invalid font name: `Open-Sans`.

The family name can only contain letters, numbers and spaces, and cannot be empty.

```ribalta
@font title "Open-Sans"

# Opening
```

**How to fix:** use the family name as it appears on Google Fonts, in quotes: `@font title "Open Sans"`.

### R509 · Invalid font weight

**Error.** Message: invalid font weight: `450`.

```ribalta
@font title "Inter" 450

# Opening
```

**How to fix:** use weights from 100 to 900, in steps of 100, such as `400` or `500`.

### R510 · Component without a name

**Error.** Message: `@component` needs a name.

```ribalta
@component {text}
$text
@end
```

**How to fix:** write the name right after `@component`: `@component badge {text}`.

### R511 · Invalid component name

**Error.** Message: invalid component name: `Badge`.

Component names start with a lowercase letter and use only lowercase letters, numbers and hyphens.

```ribalta
@component Badge {text}
$text
@end
```

**How to fix:** use a name such as `badge` or `new-badge`.

### R512 · Component parameter with a style or id

**Error.** Message: component parameters do not accept .style or #id.

The braces after `@component` declare parameters, not styles.

```ribalta
@component badge {.card text}
$text
@end
```

**How to fix:** remove the `.style` or `#id` from the list and apply the style to a block in the body, such as `::: box {.card}`.

### R513 · Unknown theme

**Error.** Message: unknown theme: `dark`.

The theme does not exist, and the document falls back to the `paper` theme.

```ribalta
@theme dark

# Opening
```

**How to fix:** use `paper`, `ink` or `editorial`. See [Built-in themes](/en/reference/themes/).

`@theme` does not create or name a theme: it only picks the base. To have your own theme, keep a base and adjust it with `@color`, `@font` and `@style`. See [Themes](/en/guides/themes/).

### R514 · Unsupported language version

**Error.** Message: unsupported language version: `3`.

```ribalta
@version 3

# Opening
```

**How to fix:** use `@version 2`, the current version, or remove the line.

### R515 · Invalid page size

**Error.** Message: invalid page size: `16:9`.

`@page` takes the size in pixels, not a proportion. Width and height go from 16 to 16384, and the height can be `auto`.

```ribalta
@page 16:9

# Opening
```

**How to fix:** use width x height, as in `@page 1920x1080`, or the key form, as in `@page width=800 height=auto`. See [Page sizes](/en/guides/pages/).

The same code appears for `width` and `height` on the page separator, with the messages invalid page width and invalid page height:

```ribalta
# Opening

--- {width=10 height=tall}
# Vertical
```

**How to fix:** use a number from 16 to 16384 (`width=1080`) and, for the height, a number or `auto` (`height=1920`).

### R516 · Invalid color name

**Error.** Message: invalid color name: `Brand`.

Color tokens start with a lowercase letter and use only lowercase letters, numbers and hyphens.

```ribalta
@color Brand #ff6b4a

# Opening
```

**How to fix:** use a name such as `brand` or `brand-dark`.

### R517 · Invalid font token name

**Error.** Message: invalid font name: `Title`.

The token name (the first word after `@font`) follows the same rule as color tokens.

```ribalta
@font Title "Bebas Neue" 400

# Opening
```

**How to fix:** use a name such as `title`, and apply it with `{font=title}`.

### R518 · Invalid style name

**Error.** Message: invalid style name: `Highlight`.

```ribalta
@style Highlight {color=accent weight=bold}

# Opening
```

**How to fix:** use lowercase letters, numbers and hyphens, such as `highlight`.

### R519 · Pages in the base theme

**Error.** Message: the base theme can only contain definitions, not pages.

It only appears when checking the [base theme](/en/guides/themes/), never in documents. The example below is the text of the base theme.

```ribalta
@theme ink
@color brand #12a37f

# Opening
```

**How to fix:** keep only lines that start with `@` (`@theme`, `@color`, `@font`, `@style` and `@component`) in the theme, plus blank lines and comments. Page content goes in each document.

### R520 · `@title` or `@author` in the base theme

**Warning.** Message: `@title` in the base theme is ignored; set it in each document.

It only appears when checking the base theme. Title and author belong to each document, so they are ignored in the theme.

```ribalta
@title Annual report
@theme ink
```

**How to fix:** remove the line from the theme and write `@title` or `@author` at the start of each document.

### R521 · Repeated `@header`, `@footer` or `@overlay`

**Error.** Message: `@footer` was defined more than once.

Each document has one header, one footer and one overlay. The second block is ignored.

```ribalta
@footer
Confidential
@end

@footer
$page / $pages
@end

# Opening
```

**How to fix:** put all the content in a single block, for example with `::: columns`. See [Header and footer](/en/guides/header-footer/).

### R522 · Unknown field in the header, footer or overlay

**Error.** Message: `@footer` uses `$pagina`, which is not an available field.

Inside `@header`, `@footer` and `@overlay`, `$` followed by a name is an automatic field. The hint suggests the closest field.

```ribalta
@footer
Page $pagina
@end

# Opening

---
# Agenda
```

**How to fix:** use one of the fields: `$page`, `$pages`, `$title`, `$author` or `$date`.

### R523 · Step-by-step reveal in the header, footer or overlay

**Warning.** Message: `@footer` is always shown in full; `step` and `::: steps` are ignored in it.

```ribalta
@footer
::: steps
- First
- Second
:::
@end

# Opening

---
# Agenda
```

**How to fix:** remove `step` and `::: steps` from the block. To reveal something gradually, do it inside the page.

### R524 · Notes in the header, footer or overlay

**Warning.** Message: notes (`::: notes`) inside `@footer` are ignored.

```ribalta
@footer
Confidential
::: notes
Remember to mention the contract.
:::
@end

# Opening

---
# Agenda
```

**How to fix:** write `::: notes` inside the page where the note belongs.

### R525 · Text on the `@header`, `@footer` or `@overlay` line

**Warning.** Message: the text `Confidential` after `@footer` will be ignored.

The `@header`, `@footer` or `@overlay` line only accepts attributes in braces. The content goes on the following lines, up to `@end`.

```ribalta
@footer Confidential
@end

# Opening
```

**How to fix:** move the text to the next line. To turn off the footer that comes from the base theme, use `@footer none`.

### R530 · Definition that does not fit the library item kind

**Error.** Message: `@theme` cannot be used in a palette.

Each library item kind accepts only some definitions: palettes take `@color` and `@font`; styles also take `@style`; components also take `@component`; themes accept any theme definition.

```ribalta
@theme paper
@color brand #349ef4
```

**How to fix:** delete the highlighted line or save the item with another kind, for example **Theme** instead of **Palette**.

### R531 · Library item without a main definition

**Error.** Message: choose which component is the main one.

Library components and styles must say which definition is the main one, the one people call when they use the item.

**How to fix:** say which definition is the main one when saving the item.

### R532 · Main definition not found

**Error.** Message: component not found: `card`.

The chosen main definition does not exist in the item text, or it was renamed.

**How to fix:** pick another main definition or fix the name in the text.

### R533 · Unknown item kind

**Error.** Message: unknown item kind: `page`.

The item was sent with a kind the library does not know. Valid kinds are theme, component, style and palette (templates are validated as themes).

**How to fix:** save the item again with one of the valid kinds.

### R534 · Unknown mode

**Error.** Message: unknown mode: `play`.

```ribalta
@mode play

# Opening
```

**How to fix:** use `@mode default` (the default) or `@mode game`.

## F6xx: variables, expressions and state

### R601 · Unknown variable

**Error.** Message: unknown variable: `$totl`.

An expression, `if`, `for` or `click` uses a name that was not declared with `@let`, `@data` or `@state`, or a field the record doesn't have. Outside expressions, an undeclared `$name` stays as text.

```ribalta
@let total = 10

::: if $totl > 5
Many items
:::
```

**How to fix:** fix the name (the hint suggests the closest one) or declare the variable.

### R602 · Incompatible types

**Error.** Message: expected number, but the value is text.

The value doesn't have the type the operation expects: arithmetic on text, a condition that is not true/false, a field on something that is not a record, or a `@state` receiving another type.

```ribalta
@let name = Ana

# $($name * 2)
```

**How to fix:** use values of the right type. For conditions, compare: `$total > 0`.

### R603 · Invalid expression

**Error.** Message: the expression ended too early.

The expression is badly written: a value, a parenthesis or a quote is missing, or there is a character that is not part of the language.

```ribalta
# Total: $(10 +)
```

**How to fix:** complete the expression. To compare, use `==` (not `=`).

### R604 · Value that depends on state

**Error.** Message: this value depends on the document state and cannot be used here.

`@state` values (and `$step`) can only be used in conditions (`if`), variants (`.style=condition`), text and `click`. Visual attributes, `@let` and `for` lists need fixed values.

```ribalta
@state mode = light

# Hello {color=$mode}
```

**How to fix:** use a variant: `@style dark {color=ink}` and `{.dark="$mode == dark"}`.

### R605 · Variable that cannot change

**Error.** Message: `$total` cannot be changed.

A `click` tries to change a `@let`, a `@data`, a component parameter or a `for` variable. Only `@state` changes while the document is shown.

```ribalta
@let total = 0

[Add]{click="$total = $total + 1"}
```

**How to fix:** declare the variable with `@state total = 0`.

### R606 · Invalid repetition

**Error.** Message: `for` needs a list, but the value is number.

The `for` didn't get a list, is badly written, depends on state or would exceed the limit of 500 repetitions in the document.

```ribalta
@let total = 3

::: for item in $total
$item
:::
```

**How to fix:** use a list (`@data`, `[a, b]`) or a range (`1..3`).

### R607 · `else` without `if`

**Error.** Message: `else` must come right after an `if` block.

`::: else` and `::: else if` must come immediately after a `::: if` (or another `else if`) in the same block. Another block in between, or a second `else`, breaks the sequence.

```ribalta
# Title
::: else
Alternative
:::
```

**How to fix:** place the `::: else` right after the matching `::: if … :::`.

### R608 · Value the state never gets

**Warning.** Message: `$plan` never gets the value `prro`.

A condition compares a text `@state` with a value that is neither the initial value nor assigned by any `click`. It's usually a typo.

```ribalta
@state plan = pro

::: if $plan == prro
Pro plan
:::
```

**How to fix:** fix the value (the hint lists the values in use).

### R609 · Invalid definition

**Error.** Message: `@let` expects a name, `=` and a value.

A `@let` or `@state` without `=` or with an invalid name, a `@data` without a name or a table, or a `@data` field with an invalid name.

```ribalta
@let highlight coral
```

**How to fix:** write `@let highlight = coral`. Names use lowercase letters, numbers, hyphens and underscores.

### R610 · Repeated name

**Error.** Message: `$total` was already defined.

`@let`, `@data` and `@state` share the same names, which must be unique. `step` is reserved for the current step.

```ribalta
@let total = 1
@state total = 2
```

**How to fix:** give the variables different names.

### R611 · Invalid action

**Error.** Message: invalid action: `skip`.

The `click` value is not a known action.

```ribalta
[Next]{click="skip"}
```

**How to fix:** use `$x = value`, `toggle $x`, `go #id`, `go back`, `go next` or `go prev`, separated by `;`.

### R612 · Navigation in `on-enter`

**Error.** Message: `on-enter` can only change state; `go`, `back`, `next` and `prev` are not allowed.

`on-enter` runs when the page is entered. Changing pages there would move to another page without anyone asking for it, and could create a loop.

```ribalta
--- {on-enter="go #end"}
# Introduction
```

**How to fix:** keep only `$x = value` and `toggle $x` in `on-enter`. To choose the following page, use `next=#id` on the separator.

### R613 · Draw outside an action

**Error.** Message: `random` can only be used in actions (`click` and `on-enter`).

`random` and `pick` draw a new value on every call. In text, in a condition or in a `@let`, the result would change every time the screen is drawn.

```ribalta
@state die = 0

# Your turn
You rolled $(random(1, 6))
```

**How to fix:** draw inside an action and store the result in a `@state`: `[Roll]{click="$die = random(1, 6)"}` and `You rolled $die`.

### R614 · `on-timeout` without `timeout`

**Error.** Message: `on-timeout` needs `timeout` on the same page.

```ribalta
# Opening

--- {on-timeout="go next"}
# Introduction
```

**How to fix:** say how many seconds to wait on the same separator: `--- {timeout=30 on-timeout="go next"}`.

### R615 · Unknown sound

**Error.** Message: this page has no sound `dor`.

`play` and `stop` only reach the sounds declared with `:: sound` on the same page.

```ribalta
# The door
:: sound https://example.com/door.mp3 {name=door}
[Open]{click="play dor"}
```

**How to fix:** fix the name or declare the sound on the page with `{name=…}`.

### R616 · Repeated sound

**Error.** Message: another sound `door` already exists on this page.

**How to fix:** give each `:: sound` on the page a different name.

### R617 · `@persist` with something that is not state

**Error.** Message: `@persist` only keeps @state variables, and `$fixed` is not one.

```ribalta
@let fixed = 1
@persist fixed

# Opening
```

**How to fix:** list only variables declared with `@state` at the start of the document.

### R618 · Invalid `@scorm`

**Error.** Message: `score` in @scorm must be a numeric @state variable, and `$name` is not.

```ribalta
@state name = "Ana"
@scorm score=$name

# Opening
```

It also appears for an unknown parameter (only `score`, `complete`, `max` and `pass` exist), for `max` or `pass` that are not positive numbers, for `pass` greater than `max` and when both `score` and `complete` are missing.

**How to fix:** use a numeric `@state` as the score, such as `@state points = 0` and `@scorm score=$points complete=#end`.

### R619 · Changing a value from outside

**Error.** Message: `$round` comes from outside the document (@extern) and is not changed by actions.

```ribalta
@extern round = 1

# Scoreboard
[Next]{click="$round = $round + 1"}
```

It also appears in `:: input {bind=$round}`, in `@persist round` and in `@scorm score=$round`.

**How to fix:** let the program that shows the document set the value, or declare it with `@state` if the document itself should change it.

### R620 · Repetition item in an action

**Error.** Message: actions cannot use `$thing`, which only exists while the repetition is drawn.

```ribalta
@state backpack = [map, key]
@state chosen = ""

# Backpack
::: for thing in $backpack
[$thing]{click="$chosen = $thing"}
:::
```

It appears when a `::: for` walks a list that changes while the document is shown (`@state` or `@extern`) and an action uses the item or its position.

**How to fix:** use the item in text, `if` or variants; for actions, use `@state` values.

## F7xx: navigation

### R701 · Missing page

**Error** in `click="go #id"` and `next=#id`; **warning** in `[text](#id)` links. Message: there is no page with the id `#end`.

```ribalta
[Go to the end]{click="go #end"}
```

**How to fix:** give the destination page that id (`--- {#end}`) or fix the name.

### R702 · Unreachable hidden page

**Warning.** Message: hidden page `#extra` is not reached by any link or `go`.

A page with `hidden` doesn't appear in the normal flow; without a link, `go` or `next` pointing to it, nobody can see it.

```ribalta
# Start
--- {#extra hidden}
Details
```

**How to fix:** link to the page (`[Details](#extra)`) or remove `hidden`.

## F8xx: external files

These only appear in the [command line](https://github.com/Viniciusadm/ribalta/tree/master/crates/cli), with `ribalta check --check-assets`, which goes online to check images and sounds.

### R801 · Unreachable image or sound

**Warning.** Message: the image `https://example.com/photo.jpg` answered 404.

```ribalta
# Team

![Team photo](https://example.com/photo.jpg)
```

It also appears when the address does not answer (wrong domain, no connection) and when it is a web page rather than the file: for example, the link to a photo on Unsplash instead of the image address.

**How to fix:** open the address in the browser. If it is a page, right-click the image and copy its address. If the file does not exist or requires a login, store the image in the program you write in (`asset:`).
