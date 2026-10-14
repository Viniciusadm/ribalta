---
title: Styling
description: Change colors, sizes, fonts, borders and shadows with attributes in braces, and create reusable styles.
---

You adjust the look of any block or piece of text with attributes in braces. Values come from the theme's scales and names, which keeps your pages consistent without effort.

## Attributes in braces

Attributes look like `{.style key=value key="value with spaces" option}`. There are two places to put them:

| Where | How | Applies to |
| --- | --- | --- |
| At the end of a line, **with a space before** | `# Heading {size=5xl color=coral}` | The whole block: heading, paragraph, list item, image or `:::` block |
| Attached to text in brackets, **with no space** | `But it was [not ready yet]{color=coral}` | Only that text |

```ribalta preview
# Results {color=accent}
The project was [done ahead of schedule]{weight=bold color=green}.

Costs within budget. {.muted}
```

In a paragraph with several lines, attributes on the last line apply to the whole paragraph. A line with attributes always ends the paragraph.

Attributes that are just a name, such as `italic`, `uppercase`, `border`, `shadow` and `wrap`, are turned on simply by being there. To turn one off, use `=false`, as in `italic=false`.

:::tip
To write a literal brace in the text, use `\{`.
:::

## Colors

`color` changes the text color and `bg` changes the background. Colors come in three formats:

| Format | Example | Description |
| --- | --- | --- |
| Theme name | `color=accent` | A color defined by the theme or by `@color`. |
| Hexadecimal | `color=#23976a` | `#rgb`, `#rrggbb` or `#rrggbbaa`. |
| With opacity | `bg=coral/40` | Any color followed by `/` and a number from 0 to 100. |

Every theme has these colors:

| Name | Use |
| --- | --- |
| `bg` | Page background |
| `text` | Main text |
| `muted` | Secondary text |
| `accent` | The theme's highlight color |
| `surface` | Card background |
| `line` | Lines, borders and dividers |
| `coral`, `blue`, `green`, `yellow`, `violet` | Supporting colors |
| `white`, `black`, `ink`, `paper`, `transparent` | Neutrals |

The exact values for each theme are in [Themes](/en/reference/themes/).

### Gradients

In `bg`, two colors separated by `..` create a diagonal gradient:

```ribalta preview
--- {bg=ink..violet color=white}
# Gradients and transparency
Text at 70% opacity {color=white/70}

[Coral]{.coral}, [blue]{color=blue}, [hexadecimal]{color=#23976a} and [see-through background]{bg=yellow/30}
```

## Sizes and spacing

Sizes and spacing use named scales. You can also give a number in pixels, such as `size=72`.

### Text size (`size`)

| Name | Pixels | Name | Pixels |
| --- | --- | --- | --- |
| `xs` | 20 | `2xl` | 64 |
| `sm` | 26 | `3xl` | 88 |
| `md` | 32 | `4xl` | 116 |
| `lg` | 40 | `5xl` | 150 |
| `xl` | 52 | `6xl` | 200 |
| | | `7xl` | 260 |

In pixels, `size` accepts 8 to 400.

### Spacing (`pad` and `gap`)

`pad` is the inner margin of a block and `gap` is the space between its items. `pad` accepts one value, or two in quotes (vertical and horizontal), such as `pad="sm lg"`.

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

### Rounded corners (`radius`)

| Name | Pixels |
| --- | --- |
| `none` | 0 |
| `sm` | 8 |
| `md` | 16 |
| `lg` | 28 |
| `xl` | 44 |
| `full` | Fully rounded |

## Typography

| Attribute | Values |
| --- | --- |
| `font` | A theme font: `body`, `display`, `mono` or one declared with `@font` |
| `weight` | `thin`, `light`, `regular`, `medium`, `semibold`, `bold`, `extrabold`, `black`, or 100 to 900 |
| `italic` | Italic |
| `uppercase` | Uppercase |
| `leading` | Line height: `tight`, `normal`, `loose` or a number from 0.6 to 3 |
| `tracking` | Letter spacing: `tight`, `normal`, `wide` or a number in em, from -0.2 to 1 |

```ribalta preview
What's new {.kicker}

# Typography with attributes {font=display}

Light, spaced-out text {weight=light tracking=wide}

Italic uppercase text {italic uppercase size=md}

Loose line height for text that takes
more than one line on the page. {leading=loose size=md}
```

## Borders, shadows and opacity

| Attribute | Values |
| --- | --- |
| `border` | On its own, applies a 3-pixel border in the `line` color. Accepts a width and a color, such as `border="2 coral"`. |
| `shadow` | `sm`, `md`, `lg` or `none`. On its own, it means `md`. |
| `opacity` | From 0 to 1, or a percentage, such as `opacity=60%`. |

```ribalta preview
::: row {gap=lg}
::: box {border pad=md radius=md}
Default border
:::
::: box {border="2 coral" pad=md radius=lg}
Coral border
:::
::: box {shadow=lg bg=surface pad=md radius=xl}
Large shadow
:::
::: box {opacity=50% bg=surface pad=md}
Transparent
:::
:::
```

## Background image

`image` sets a background image that covers the whole block or page. The address must start with `https://` or be a stored image (`asset:`):

```ribalta preview
--- {image=https://images.unsplash.com/photo-1501785888041-af3ef285b470?w=1200 color=white}
::: box {bg=ink/60 pad=lg radius=lg}
# 2026 offsite
Planning the team retreat.
:::
```

## Reusable styles

When the same combination of attributes keeps coming back, give it a name with `@style` at the start of the document. Then apply the style with a dot, such as `{.highlight}`:

```ribalta preview
@color brand #1e83d6
@style highlight {color=brand weight=bold}
@style badge {.card pad="sm md" bg=brand/15 color=brand}

# Business proposal
Delivery takes [ten business days]{.highlight}.

Offer valid until November 30 {.badge}
```

A style can include other styles, such as `.card` inside `.badge`. When you combine a style with attributes in the same place, the attributes written there win: with `{.badge bg=yellow}`, the background is yellow.

Style names use lowercase letters, numbers and hyphens.

## Built-in styles

Every theme comes with these styles:

| Style | Same as |
| --- | --- |
| `.muted` | `color=muted` |
| `.accent` | `color=accent` |
| `.lead` | `size=xl color=muted` |
| `.kicker` | `size=sm weight=bold uppercase tracking=0.14 color=accent` |
| `.display` | `font=display` |
| `.huge` | `font=display size=6xl leading=0.95` |
| `.card` | `bg=surface radius=lg pad=lg shadow=md` |
| `.dark` | `bg=ink color=paper` |
| `.center` | `align=center` |

```ribalta preview
--- {.dark .center}
Built-in styles {.kicker}

# Simple and consistent {.display}

Supporting text that stands out. {.lead}

---
# 98% {.huge .accent}
customer satisfaction {.muted}

::: box {.card}
A card with a background, rounded corners and a shadow.
:::
```

## Colors as styles

Every theme color also works as a style. `{.coral}` is the same as `{color=coral}`, and that includes the colors you create with `@color`:

```ribalta preview
@color brand #1e83d6

# Project status
[Done]{.green} · [In progress]{.yellow} · [Late]{.coral}

Owner: product team {.brand}
```

## Next steps

- [Themes](/en/guides/themes/): pick a theme and define your own colors and fonts.
- [Components](/en/guides/components/): combine styles and layouts into reusable blocks.
- [Attributes reference](/en/reference/attributes/): every attribute and where it applies.
