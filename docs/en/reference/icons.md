---
title: Icons
description: How to use the built-in icons and the full list of available names.
---

The `icon` block draws an icon from the built-in library, on a single line:

```ribalta
:: icon name {size=… color=…}
```

| Part | Values | Default |
| --- | --- | --- |
| `name` | One of the names in the list below | Required ([R114](/docs/en/reference/errors/)) |
| `size` | `xs`…`7xl` scale or pixels from 8 to 400 ([Sizes](/docs/en/reference/attributes/#sizes)) | 96 px |
| `color` | Any [color](/docs/en/reference/attributes/#colors) | Text color |

Icons also accept the other [style attributes](/docs/en/reference/attributes/), such as `opacity` and `step`. A name outside the list is an error ([R115](/docs/en/reference/errors/)), with a suggestion of the closest name. In an editor with Ribalta support, typing `:: icon ` lists the names.

```ribalta preview
::: grid {cols=3 gap=lg}
::: box {.card align=center}
:: icon rocket {size=3xl color=accent}
### Launch
:::
::: box {.card align=center}
:: icon users {size=3xl color=blue}
### Team
:::
::: box {.card align=center}
:: icon trending-up {size=3xl color=green}
### Growth
:::
:::
```

## Icon list

```ribalta-icons
```
