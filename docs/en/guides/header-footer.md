---
title: Header, footer and overlay
description: Write it once and repeat the same content, in the same place, on every page.
---

Company logo, event name, "Confidential", page number: instead of repeating them on every page, write them once with `@header` and `@footer`. Ribalta places that content in the same spot on every page, **except the cover**.

For things that should float on top of the content (a logo in a corner, a watermark, a "Draft" stamp), use the [overlay](#overlay-overlay), `@overlay`.

## The basics

Write the block at the start of the document, next to `@title` and `@theme`, and close it with `@end`:

```ribalta preview
@title 2027 Plan
@theme paper

@footer
2027 Plan · Confidential
@end

# 2027 Plan

---
# Where we are

---
# Where we are going
```

The first page is the cover and has no footer. On the others, the footer sits at the bottom, inside the page margin.

The content between `@footer` and `@end` uses the same language as the pages: text, images, icons, styles, components and blocks such as `::: columns`.

## Left and right

To put one part on each side, use `::: columns`. Each item becomes a column, so separate the items with a **blank line**:

```ribalta preview
@title Lumen Pitch
@theme ink

@header
::: columns {valign=center}
:: icon rocket {size=lg color=accent}

Lumen Analytics {align=right}
:::
@end

@footer
::: columns
Confidential

$page / $pages {align=right}
:::
@end

# Lumen Analytics

---
# Traction
```

:::tip
Two consecutive lines without a blank line between them form a single paragraph with a line break. That is why they would stay together, in the same column.
:::

## Automatic fields

Inside `@header`, `@footer` and `@overlay`, these fields are filled in on each page:

| Field | Becomes |
| --- | --- |
| `$page` | the page number (the cover counts as 1) |
| `$pages` | the total number of pages |
| `$title` | the document's `@title` |
| `$author` | the document's `@author` |
| `$date` | today's date, in the viewer's language |

A misspelled field shows up as an error, with the closest match suggested: `$pagina` suggests `$page`.

## Appearance

By default, header and footer use small text (`size=sm`) in the theme's `muted` color. On a dark page, the color follows the page.

To change the default, put attributes on the `@header` or `@footer` line. They apply to the whole block:

```ribalta preview
@theme paper

@footer {align=center color=accent weight=bold}
Product Week 2026
@end

# Opening

---
# Agenda
```

Attributes inside the block still work as usual, such as `{align=right}` on a paragraph.

## Overlay (`@overlay`)

`@overlay` works like the header and footer (written once, hidden on the cover), but it does not reserve a band: it is a page-sized layer **on top** of the content. Use it for a logo in a corner, a watermark or a "Draft" stamp.

```ribalta preview
@title Plan 2027
@theme paper

@overlay
Draft {x=1500 y=40 color=coral weight=bold uppercase}
@end

# Plan 2027

---
# Where we are

The stamp sits on top of the content, in the corner.
```

Inside the layer, a block with `x` and `y` is placed from the top-left corner **of the page**, ignoring the margin: `x=0 y=0` is the exact corner.

Without `x` and `y`, the content stays inside the page margin. Use `align` and `valign` on the `@overlay` line to pick a corner without working out coordinates:

```ribalta preview
@theme ink

@overlay {align=right valign=bottom}
:: icon sparkles {size=2xl color=accent}
@end

# Opening

---
# Agenda
```

Unlike the header and footer, the overlay has no default look: text uses the page's normal size and color. For a watermark, use `opacity`, as in `{opacity=0.1}`.

## Cover and exceptions

The cover has no header, footer or overlay. By default, the cover is the first page. On each page's `---` line, you can change that:

| On the `---` line | Effect |
| --- | --- |
| `{cover}` | the page becomes a cover too, with no header, footer or overlay (good for section pages) |
| `{cover=false}` | the first page stops being a cover |
| `{footer=off}` | removes the footer from this page only |
| `{header=off}` | removes the header from this page only |
| `{overlay=off}` | removes the overlay from this page only |
| `{footer=on}` | shows the footer even on a cover |

```ribalta
--- {cover}
# Part 2: Results

--- {footer=off}
![Team photo](https://images.unsplash.com/photo-1522071820081-009f0129c71c?w=1600){fit=cover w=100% h=100%}
```

## In the project theme

A `@header`, `@footer` or `@overlay` written in the [project theme](/docs/en/guides/themes/#project-theme) applies to every document in the project. It is the place for the company's standard footer.

- If a document has its own `@footer`, it **replaces** the theme's.
- To remove the theme's footer from one document, write `@footer none`. The same goes for `@header none` and `@overlay none`.

```ribalta
@title A document without the company footer
@footer none
```

## Under the hood

Header and footer take a reserved band: page content never ends up underneath them. A one-line footer fits inside the theme's margin and does not change the layout. If the block is taller, the content area shrinks. The overlay takes no space: the page layout is the same with or without it.

A few things do not apply inside `@header`, `@footer` and `@overlay`:

- **Step-by-step reveal.** These blocks are always shown in full; `step` and `::: steps` are ignored, with a warning.
- **Notes.** `::: notes` inside the block is ignored, with a warning. Write notes on the page itself.
