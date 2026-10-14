---
title: Components
description: Build reusable blocks with parameters, content and named slots to keep your pages consistent.
---

A component is a named piece of page that you define once and use as many times as you like. It takes parameters, like a template: the text changes, the look stays the same.

Inside a component you can use everything a page can, including `::: if`, `::: for` and expressions on the parameters. See [Variables, data and repetition](/en/guides/data/#components-that-take-data).

## Defining a component

Write the definition at the start of the document, together with `@title`, `@theme` and the other definitions, before the first page:

```ribalta
@component metric {value label color=accent}
::: box {.card align=center}
$value {size=5xl weight=bold color=$color}
$label {.muted}
:::
@end
```

- `@component` is followed by the name and, in braces, the list of parameters.
- Everything between that line and `@end` is the component body, written in the same language as your pages.
- The name uses lowercase letters, numbers and hyphens, such as `metric` or `team-card`.

:::note
The names of the language blocks (`box`, `columns`, `steps`, `chart` and the others) are reserved and cannot be used as component names.
:::

### Required parameters and default values

| Form | Meaning |
| --- | --- |
| `value` | Required parameter. Using the component without it is an error. |
| `color=accent` | Parameter with a default value. If you leave it out, it is `accent`. |
| `title="Untitled"` | Default value with spaces, in quotes. |

### Using the parameters

Inside the body, `$name` is replaced by the parameter value. This works anywhere:

- in text: `## $title` or `Owner: $name`;
- in attribute values: `{color=$color}` or `{bg=$background}`;
- on a line of its own: `$label` becomes a paragraph with the value.

The value can contain text formatting, such as `**bold**`, and it is applied on the page.

## Using a component

There are two ways to use a component, the same as for language blocks:

- **Single line**, with `::`, when it has no content: `:: metric {value=38% label="revenue growth"}`.
- **Container**, with `:::` and a closing `:::`, when it takes content (see the next section).

Parameters go in braces, always by name. Values with spaces go in quotes.

```ribalta preview
@component metric {value label color=accent}
::: box {.card align=center}
$value {size=5xl weight=bold color=$color}
$label {.muted}
:::
@end

# Quarterly results
::: grid {cols=3 gap=lg}
:: metric {value=38% label="revenue growth"}
:: metric {value=94 label="new customers" color=green}
:: metric {value=4.8 label="satisfaction score" color=violet}
:::
```

## Content with `$content`

When you use the component as a container, everything between the opening and closing lines goes into `$content`. Put `$content` on a line of its own, at the point in the body where the content should appear:

```ribalta preview
@component highlight {title}
::: box {.card gap=sm}
## $title
$content
:::
@end

::: highlight {title="Next steps"}
- Review the budget with the board
- Hire two people for support
- Launch the new plan in March
:::
```

The content can be anything: text, lists, images, other blocks and other components.

:::caution
If the component has no `$content` in its body, the content you pass is ignored and the compiler warns.
:::

## Named slots

Sometimes a component needs more than one content area, such as the two columns of a comparison. That is what named slots are for:

1. In the component body, write `$slot-name` on a line of its own, using a name that is **not** a parameter.
2. When using the component, fill the slot with a `::: slot slot-name` … `:::` block.
3. The rest of the content still goes into `$content`.

```ribalta preview
@component comparison {before after}
::: columns {gap=xl}
::: box {.card gap=sm}
## $before
$content
:::
::: box {.card gap=sm bg=accent color=white}
## $after
$result
:::
:::
@end

::: comparison {before="Scattered spreadsheets" after="A single dashboard"}
Each team exports, pastes and formats the numbers by hand.
::: slot result
The data arrives on its own and the report refreshes every hour.
:::
:::
```

Slots you do not fill stay empty, with no error. Filling a slot the component does not have is an error, and the message suggests the closest name.

## Components inside components

A component body can use other components, and so can the content you pass to a component:

```ribalta preview
@component badge {text}
::: box {bg=accent color=white pad=sm radius=lg}
$text {size=sm weight=bold uppercase}
:::
@end

@component plan {name price}
::: box {.card gap=sm}
:: badge {text=$name}
## $price
$content
:::
@end

::: grid {cols=2 gap=lg}
::: plan {name=Essential price="$9"}
Up to 5 users
:::
::: plan {name=Team price="$29"}
Unlimited users
:::
:::
```

A component cannot use itself, either directly or indirectly (for example, `a` uses `b` and `b` uses `a`). Ribalta detects the cycle and shows an error. Components can be nested up to 16 levels deep.

## Revealing a component step by step

The `step` attribute also works on a component call. The whole component appears at the given step:

```ribalta
:: metric {value=38% label="growth" step=1}
:: metric {value=94 label="new customers" step=2}
```

Learn more in [Step-by-step reveal](/en/guides/reveal/).

## Styling a component

Components only accept parameters. Styles such as `.card` and `#id` on the call are an error. To change how one particular use looks, wrap the component in a `box`:

```ribalta
::: box {.dark pad=lg}
:: metric {value=38% label="growth"}
:::
```

If the change should always apply, create a parameter instead, like `color=accent` in the `metric` example.

:::tip
To use the same components in many documents, define them in the base theme. See [Themes](/en/guides/themes/).
:::

## Parameters with accepted values

Separate the accepted values with `|` in the default value. The first one is the default, and any other value is an error ([R418](/en/reference/errors/)). Combine it with one style per variant, such as `.button-$kind`, to avoid repeating properties:

```ribalta preview
@style button-primary {bg=accent color=white}
@style button-ghost {color=accent border="2 accent"}
@component button {label kind=primary|ghost}
::: row {.button-$kind pad="sm lg" radius=full}
**$label**
:::
@end

::: row {gap=md}
:: button {label=Subscribe}
:: button {label="Learn more" kind=ghost}
:::
```

## Records as parameters

A parameter can take a whole `@data` record: `:: card {p=$person}`. In the body, use `$p.name`, `$p.role` and so on. When using the component, `if` and `click` are accepted too: `:: card {p=$person if=$show click="go #team"}`.

## Common errors

| Code | When it happens |
| --- | --- |
| R401 | `$content` was used outside a `@component`. |
| R404 | The component uses itself, directly or indirectly. |
| R405 | Components nested more than 16 levels deep. |
| R406 | The call has a `.style` or `#id`. Wrap the component in a `box`. |
| R407 | The call passes a parameter the component does not have. |
| R408 | The call has loose text instead of parameters in braces. |
| R409 | A required parameter is missing. |
| R410 | The body uses `$name`, but `name` was not declared as a parameter. |
| R411 | Warning: the component received content but does not use `$content`. |
| R412 | The component name is the name of a language block. |
| R413 | The same component was defined twice in the document. |
| R414 | `::: slot` fills a slot the component does not have. |
| R415 | `::: slot` without the slot name. |
| R416 | `::: slot` outside a component call. |
| R417 | The same slot was filled twice. |
| R504 | `@component` without a matching `@end`. |
| R511 | Invalid component name (use lowercase letters, numbers and hyphens). |

Errors inside a component body are reported on the line where the component is **used**, with the note "(in component `name`)". The full list is in [Error messages](/en/reference/errors/).
