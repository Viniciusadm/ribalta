---
title: Attributes
description: Every attribute the compiler accepts, with its values, scales and units.
---

Attributes go in braces, as in `{size=5xl color=accent}`. This page lists every attribute the compiler accepts and the valid values for each one. For the brace syntax itself (`.style`, `#id`, quotes, position on the line), see [Syntax](/docs/en/reference/syntax/#attributes).

An unknown attribute is an error ([R209](/docs/en/reference/errors/)), and so is an invalid value ([R301](/docs/en/reference/errors/)); the messages suggest the closest name or value.

## Style attributes

They work on any block, on the page separator (`--- {…}`), on text spans (`[text]{…}`) and on images.

| Attribute | Accepted values | Example | Notes |
| --- | --- | --- | --- |
| `color` | [Color](#colors) | `color=accent` | Text color. Also applies to icons. |
| `bg` | [Color](#colors) or gradient `color..color` | `bg=ink..violet` | Background. The gradient is linear and diagonal (135°). |
| `image` | `https://…`, `http://…`, `data:image/…` or `asset:<id>` | `image=asset:<id>` | Background image, covering the area and centered. |
| `font` | Font token: `body`, `display`, `mono` or a `@font` | `font=display` | Typeface. |
| `size` | [Size scale](#sizes) or pixels from 8 to 400 | `size=5xl`, `size=120` | Text size; on icons, the icon size. |
| `weight` | [Weight by name](#weights) or 100 to 900, in steps of 100 | `weight=bold`, `weight=600` | Font weight. |
| `italic` | Boolean | `italic`, `italic=false` | Italic. |
| `uppercase` | Boolean | `uppercase` | All caps. |
| `align` | `left`, `center`, `right` | `align=center` | **Horizontal** alignment, in any layout. |
| `valign` | `top`, `center` (or `middle`), `bottom` | `valign=bottom` | **Vertical** alignment, in any layout. |
| `pad` | One or two [spacing](#spacing) values | `pad=lg`, `pad="sm xl"` | Inner spacing. With two values: vertical and horizontal. |
| `gap` | [Spacing](#spacing) | `gap=xl` | Space between the children of a block. |
| `radius` | [Radius](#radius) or pixels from 0 to 9999 | `radius=lg` | Rounded corners. |
| `border` | Width (0 to 40 px) and/or [color](#colors), in any order | `border`, `border="2 coral"` | With no value: 3 px in the theme's `line` color. |
| `shadow` | `sm`, `md`, `lg`, `none` | `shadow`, `shadow=lg` | With no value, it is `md`. |
| `w` | [Length](#lengths) or `auto` | `w=50%`, `w=480` | Width. |
| `h` | [Length](#lengths) or `auto` | `h=full` | Height. |
| `x` | [Length](#lengths) | `x=120` | Absolute horizontal position (distance from the page's left edge). |
| `y` | [Length](#lengths) | `y=80` | Absolute vertical position (distance from the page's top edge). With only one of `x`/`y`, the other is 0. |
| `opacity` | Number from 0 to 1 or percentage from 0% to 100% | `opacity=0.6`, `opacity=60%` | Transparency of the whole block. |
| `leading` | `tight`, `normal`, `loose` or a number from 0.6 to 3 | `leading=tight` | Line height. See [line height](#line-height-and-letter-spacing). |
| `tracking` | `tight`, `normal`, `wide` or a number from -0.2 to 1 (in `em`) | `tracking=0.1` | Letter spacing. |
| `step` | Whole number from 1 to 99 | `step=2` | Step at which the block or span appears. See [Reveal](/docs/en/guides/reveal/). |
| `enter` | `fade`, `up`, `down`, `left`, `right`, `zoom`, `none` | `enter=zoom` | How the element appears on a step or when an `if` becomes true. Steps default to `up`. |
| `if` | Condition | `if=$details`, `if="$n > 2"` | Shows the element only when the condition is true. See [Variables](/docs/en/guides/data/#conditions-with--if). |
| `click` | Actions separated by `;` | `click="go #end"` | What happens on click. See [Interactivity](/docs/en/guides/interactive/#clicks). |
| `key` | A letter, a digit, `up`, `down`, `left`, `right`, `Enter` or `Space` | `key=up` | Key that runs the element's `click`. See [Keyboard](/docs/en/guides/interactive/#keyboard). |

In addition, `.style=condition` applies a style only when the condition is true: `{.highlight="$plan == pro"}`. With `.style=hover`, the style applies while the mouse is over the element.

Attribute values accept `$variable` and `$( expression )`, as long as they don't depend on `@state` ([R604](/docs/en/reference/errors/)).

Boolean attributes accept `true`, `yes` or `sim` to turn on and `false`, `no` or `não` to turn off. Writing just the name (`{italic}`) turns the attribute on.

```ribalta preview
--- {bg=paper..surface}
::: columns {gap=xl}
::: box {bg=accent color=white radius=xl pad="md lg" shadow=lg}
## Warm card
`accent` background, `xl` corners
:::
::: box {border="4 blue" radius=md pad=lg}
## Blue border {color=blue}
`border="4 blue"` {italic}
:::
::: box {bg=ink..violet color=white radius=full pad="lg xl" align=center}
## Gradient
`bg=ink..violet` {opacity=0.8}
:::
:::
```

## Block attributes

Besides the style attributes, some blocks accept attributes of their own. Using them anywhere else is an error ([R209](/docs/en/reference/errors/)). In a `@style`, all of them are accepted, and they only take effect when the style is applied to the right block.

| Attribute | Where | Values | Default | Example |
| --- | --- | --- | --- | --- |
| `fit` | Block image | `contain` (whole image) or `cover` (fills and crops) | `contain` | `![Photo](https://…){fit=cover}` |
| `widths` | `columns` | Positive numbers separated by spaces, one per column | Equal columns | `::: columns {widths="2 1"}` |
| `cols` | `grid` | Whole number from 1 to 12 | Number of children, from 1 to 3 | `::: grid {cols=4}` |
| `wrap` | `row` | Boolean: wraps the items onto more than one line | Off | `::: row {wrap}` |
| `type` | `chart` | `bar`, `line`, `area`, `pie`, `donut` | `bar` | `::: chart {type=line}` |
| `legend` | `chart` | Boolean | On with more than one series or for `pie`/`donut` | `{legend=false}` |
| `stacked` | `chart` | Boolean: stacks the series | Off | `{stacked}` |
| `values` | `chart` | Boolean: shows the values on the chart | Off | `{values}` |
| `colors` | `chart` | List of [colors](#colors) separated by spaces, one per series | Theme palette | `{colors="accent blue"}` |
| `data` | `chart`, `table` | A list of records (`@data`) | | `:: chart {data=$sales}` |
| `label` | `chart` with `data` | The field with the labels | The first text field | `{label=month}` |
| `series` | `chart` with `data` | Fields with the values, separated by spaces | Every numeric field | `{series="revenue cost"}` |
| `columns` | `table` | Fields shown, in order | All | `{columns="name role"}` |
| `cover` | `---` separator | Boolean: the page is a cover and has no header, footer or overlay | On for the first page only | `--- {cover}` |
| `header` | `---` separator | `on` or `off`: shows or hides the header on this page | On, except on the cover | `--- {header=off}` |
| `footer` | `---` separator | `on` or `off`: shows or hides the footer on this page | On, except on the cover | `--- {footer=on}` |
| `overlay` | `---` separator | `on` or `off`: shows or hides the overlay on this page | On, except on the cover | `--- {overlay=off}` |
| `hidden` | `---` separator | Boolean: the page leaves the normal flow and is only reached by link or `go` | Off | `--- {#extra hidden}` |
| `next` | `---` separator | Id of the following page when advancing | The next page | `--- {next=#end}` |
| `steps` | `---` separator | Minimum number of steps in the page, from 1 to 99 | Computed | `--- {steps=3}` |
| `transition` | `---` separator | `fade`, `slide`, `zoom`, `none` | `@transition` or none | `--- {transition=slide}` |
| `timeout` | `---` separator | Seconds (1 to 3600) until `on-timeout` runs | None | `--- {timeout=30}` |
| `on-timeout` | `---` separator | Actions when `timeout` is reached, `go` included | None | `--- {timeout=30 on-timeout="go #end"}` |
| `nav` | `---` separator | `on` or `off`: with `off`, arrows, space, click and swipe do not leave the page | `on` (`off` with `@mode game`) | `--- {nav=off}` |
| `on-enter` | `---` separator | State actions separated by `;` (`$x = value`, `toggle $x`) | None | `--- {on-enter="$seen = true"}` |
| `width` | `---` separator | Page width in pixels, from 16 to 16384 | `@page` or the host's default | `--- {width=1080}` |
| `height` | `---` separator | Page height in pixels, from 16 to 16384, or `auto` (grows with the content) | `@page` or the host's default | `--- {width=1080 height=1920}` |

In `widths`, the proportions are relative: `"2 1"` gives two parts to the first column and one to the second. If the number of values does not match the number of columns, the compiler warns ([R302](/docs/en/reference/errors/)).

The default chart palette uses, in order, `accent`, `blue`, `green`, `yellow`, `violet` and `coral`.

## Scales and units

### Sizes

Used by `size`. Pixels from 8 to 400 also work.

| Name | Pixels |
| --- | --- |
| `xs` | 20 |
| `sm` | 26 |
| `md` | 32 |
| `lg` | 40 |
| `xl` | 52 |
| `2xl` | 64 |
| `3xl` | 88 |
| `4xl` | 116 |
| `5xl` | 150 |
| `6xl` | 200 |
| `7xl` | 260 |

```ribalta preview
::: row {gap=lg valign=bottom}
md {size=md}
xl {size=xl}
3xl {size=3xl}
5xl {size=5xl}
7xl {size=7xl}
:::
```

### Spacing

Used by `pad`, `gap` and `:: spacer`. Pixels from 0 to 600 also work.

| Name | Pixels |
| --- | --- |
| `none` | 0 |
| `xs` | 8 |
| `sm` | 16 |
| `md` | 32 |
| `lg` | 48 |
| `xl` | 72 |
| `2xl` | 112 |
| `3xl` | 160 |

### Radius

Used by `radius`. Pixels from 0 to 9999 also work.

| Name | Pixels |
| --- | --- |
| `none` | 0 |
| `sm` | 8 |
| `md` | 16 |
| `lg` | 28 |
| `xl` | 44 |
| `full` | 9999 (pill or circle shape) |

### Weights

Used by `weight`. Numbers from 100 to 900, in steps of 100, also work.

| Name | Weight |
| --- | --- |
| `thin` | 100 |
| `light` | 300 |
| `regular` | 400 |
| `medium` | 500 |
| `semibold` | 600 |
| `bold` | 700 |
| `extrabold` | 800 |
| `black` | 900 |

A weight only shows up if the font has it. Fonts declared with `@font` load the weights listed in the definition.

### Line height and letter spacing

| Attribute | Keyword | Value | Numeric range |
| --- | --- | --- | --- |
| `leading` | `tight` | 1.05 | 0.6 to 3 (multiplies the font size) |
| `leading` | `normal` | 1.35 | |
| `leading` | `loose` | 1.7 | |
| `tracking` | `tight` | -0.02 em | -0.2 to 1 (in `em`) |
| `tracking` | `normal` | 0 | |
| `tracking` | `wide` | 0.08 em | |

### Lengths

Used by `w`, `h`, `x` and `y`.

| Form | Example | Meaning |
| --- | --- | --- |
| Pixels | `480` or `480px` | From -4000 to 4000. The `px` suffix is optional. |
| Percentage | `50%` | From 0% to 100% of the available space. |
| `full` | `w=full` | Same as `100%`. |
| `auto` | `h=auto` | Automatic size. Only for `w` and `h`. |

Lengths are pixels of the page, whatever the screen it is shown on. The page size comes from the program that shows the document (a presentation app usually uses 1920×1080), from `@page` (`@page 1440x1080`) or from `width` and `height` on the separator; without any of them, the page is 1920 wide with automatic height. With automatic height, the page grows with the content. See [Page sizes](/docs/en/guides/pages/).

### Colors

Used by `color`, `bg`, `border` and `colors`.

| Form | Example | Meaning |
| --- | --- | --- |
| Theme token | `accent`, `muted`, `coral` | Color defined by the theme or by `@color`. |
| Hexadecimal | `#f60`, `#ff6600`, `#ff660080` | `#rgb`, `#rgba`, `#rrggbb` or `#rrggbbaa`. |
| Opacity | `accent/40`, `#000000/20` | Color with an opacity from 0 to 100 (%), after the slash. |
| Gradient | `ink..violet`, `accent/80..blue` | Two colors separated by `..`. **Only in `bg`.** |

Tokens available in every theme: `bg`, `text`, `muted`, `accent`, `surface`, `line`, `coral`, `blue`, `green`, `yellow`, `violet`, `white`, `black`, `ink`, `paper` and `transparent`. The values for each theme are in [Built-in themes](/docs/en/reference/themes/).

Every color token also works as a style: `{.coral}` is the same as `{color=coral}`.

```ribalta preview
::: grid {cols=4 gap=md}
::: box {bg=accent pad=md radius=md}
`accent`
:::
::: box {bg=accent/40 pad=md radius=md}
`accent/40`
:::
::: box {bg=#2f6fdb color=white pad=md radius=md}
`#2f6fdb`
:::
::: box {bg=coral..violet color=white pad=md radius=md}
`coral..violet`
:::
:::
```
