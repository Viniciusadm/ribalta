---
title: Themes
description: Pick a built-in theme, adjust colors and fonts, and share definitions across the documents in a project.
---

The theme sets the basic look of the document: colors, fonts, heading sizes and margins. You pick a built-in theme and adjust whatever you want with definitions at the start of the text.

## Choosing a theme

Use `@theme` at the start of the document, before the first page. Without `@theme`, the document uses the `paper` theme.

```ribalta
@theme editorial
```

Ribalta has three built-in themes.

### paper

Cream background, bold headings in Archivo Black, text in Barlow and a coral accent. It is the default theme.

```ribalta preview
@theme paper

Annual report {.kicker}
# 2026 results
Revenue, customers and next steps. {.lead}
```

### ink

Dark background, Inter for all text and an amber accent. Good for auditoriums and large screens.

```ribalta preview
@theme ink

Annual report {.kicker}
# 2026 results
Revenue, customers and next steps. {.lead}
```

### editorial

Warm light background, serif headings in Fraunces, text in Source Sans 3, a dark red accent and more generous margins.

```ribalta preview
@theme editorial

Annual report {.kicker}
# 2026 results
Revenue, customers and next steps. {.lead}
```

The full list of colors and fonts for each theme is in the [Themes reference](/docs/en/reference/themes/).

## Colors

`@color name value` creates a new color or replaces a theme color. The value can be a hex code, another theme color or a color with opacity, such as `coral/80`:

```ribalta preview
@theme ink
@color accent #4fd1c5
@color brand #7c5cd6

New product {.kicker}
# Sales dashboard {color=accent}
Launch planned for the second half of the year. {.brand}
```

Replacing `accent` changes every place that uses the highlight color, such as the `.kicker` style. A new color, such as `brand`, can be used in any color attribute (`color=brand`, `bg=brand/20`) and also as a style (`{.brand}`).

Color names use lowercase letters, numbers and hyphens.

## Fonts

`@font name "Family" weights` declares a Google Fonts font. Write the family name in quotes, followed by the weights you will use, from 100 to 900:

```ribalta preview
@font title "Bebas Neue" 400
@font body "Inter" 400 700

# Expansion plan {font=title size=5xl}
Three new markets by the end of the year.
```

A new font, such as `title`, is applied with `font=title`. Using a name that already exists replaces the theme font:

| Name | Where it is used |
| --- | --- |
| `body` | Regular text and smaller headings |
| `display` | Main headings and the `.display` and `.huge` styles |
| `mono` | Code |

:::note
Declare only the weights you actually use. Each weight is one more file to load.
:::

## Project theme

The project theme gathers definitions that apply to every document in a project. It is the right place for the visual identity of a company, a course or a client. The program you write in decides where the project theme is kept and how you edit it; the compiler reads it before each document.

```ribalta
@theme editorial
@color brand #1e83d6
@font title "Bebas Neue" 400
@style label {size=sm weight=bold uppercase tracking=0.14 color=brand}
```

Project theme rules:

- It can only contain definitions, such as `@theme`, `@color`, `@font`, `@style`, `@component` and `@page`. Pages in the project theme are an error.
- `@title` and `@author` are ignored there, with a warning, because they belong to each document.
- Each document can override anything it wants: its own `@theme`, a color, a font, a style, a component with the same name or the [page size](/docs/en/guides/pages/).
- Errors in the project theme are reported when the theme itself is checked, never in the documents.

## Next steps

- [Styling](/docs/en/guides/styling/): use the theme's colors and fonts on your pages.
- [Components](/docs/en/guides/components/): put components in the project theme to reuse them in every document.
- [Themes reference](/docs/en/reference/themes/): every color, font and size for each theme.
