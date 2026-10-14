---
title: Page sizes and documents
description: Set the size of the pages with @page, use automatic height for long documents and give a single page its own size.
---

A Ribalta document is a sequence of **pages**. The first content after the preamble starts page 1, and each `---` starts a new one. Every page has a width and a height in pixels, and every length in the document (`w`, `h`, `x`, `y`, sizes and spacing) is measured in pixels of the page, whatever the screen it is shown on.

## Who picks the size

The program that shows the document sets the default size when it compiles. Without a default from the program, a page is 1920 pixels wide and has [automatic height](#automatic-height).

The document can change that with `@page`, and a single page can change it again on its separator.

## `@page`

Write `@page` in the preamble, with the width and the height separated by an `x`:

```ribalta
@page 1440x1080

# A 4:3 page
```

| Form | Example | Effect |
| --- | --- | --- |
| `width`x`height` | `@page 1920x1080` | Sets both values. |
| `width`x`auto` | `@page 800xauto` | Fixed width, height that grows with the content. |
| Key form | `@page width=800 height=auto` | Same as above, written with names. |
| Only one key | `@page height=auto` | Changes only what it names; the width stays the default. |

Width and height go from 16 to 16384 pixels. Anything else, such as `@page 16:9`, is an error ([R515](/en/reference/errors/#r515--invalid-page-size)), and the default size stays.

## Automatic height

With `height=auto`, the page has a fixed width and grows downward to fit everything in it. Use it for content that is read rather than shown one screen at a time: a report, an article, a long menu, an app screen with scrolling.

```ribalta
@page 800xauto
@title Weekly report

# Weekly report
Three highlights from the team. {.lead}

## Sales
Revenue grew 12% over last week,
driven by the new annual plan.

## Support
::: grid {cols=2 gap=md}
::: box {.card}
### 94%
tickets answered within an hour
:::
::: box {.card}
### 4.8
average rating
:::
:::

## Next week
- Launch the referral program
- Review the pricing page
- Hire two more agents
```

What changes on a page with automatic height:

- The page is as tall as its content, so nothing is ever clipped at the bottom.
- `valign` on the page has no spare room to work with: the content starts at the top.
- The width still limits the content, so long lines wrap as usual.

## A page with its own size

`width` and `height` on the separator give one page its own size and leave the others as they are. A vertical page inside a horizontal document, for example:

```ribalta
@page 1920x1080

# Campaign results

--- {width=1080 height=1920 align=center}
# The vertical ad
Shown in stories, at 1080 × 1920.

---
# Back to 1920 × 1080
```

The separator accepts only one of the two as well. `--- {height=auto}` keeps the width and lets that page grow with its content, handy for an appendix at the end of a presentation:

```ribalta
# Summary
Revenue grew 38%.

--- {height=auto}
## Appendix: every number
| Month | Revenue |
| --- | ---: |
| Jan | 32 |
| Feb | 28 |
| Mar | 35 |
| Apr | 41 |
| May | 45 |
| Jun | 52 |
```

An invalid value, such as `width=10` or `height=tall`, is an error ([R515](/en/reference/errors/#r515--invalid-page-size)), and the page keeps the size it would have had.

## Who wins

Each level overrides the one before it:

| Level | Example |
| --- | --- |
| 1. Default from the program that shows the document | 1920 × 1080 in a presentation app; 1920 × automatic without one |
| 2. `@page` in the [base theme](/en/guides/themes/#base-theme) | `@page 1440x1080` for every document that uses that theme |
| 3. `@page` in the document | `@page 800xauto` |
| 4. `width` and `height` on the separator | `--- {width=1080 height=1920}` |

In the key form, each level only changes what it names. With `@page width=800` in the base theme and `@page height=auto` in the document, the pages are 800 wide with automatic height.

## Page number and total

Inside `@header`, `@footer` and `@overlay`, `$page` is the number of the current page and `$pages` is the total number of pages. Use them for a counter:

```ribalta preview
@footer {align=right}
$page / $pages
@end

# Opening

---
# Agenda

---
# Results
```

Each page also has a title and an id. The title is the first `#` heading of the page, or "Page N" without one. The id comes from `--- {#id}`, or is `page-N` without one, and is what links and `go #id` use. See [Header, footer and overlay](/en/guides/header-footer/) and [Interactivity and navigation](/en/guides/interactive/#navigation).

## Next steps

- [Layout](/en/guides/layout/): arrange the content inside each page.
- [Attributes](/en/reference/attributes/#lengths): lengths, percentages and positions.
- [Syntax](/en/reference/syntax/#page-separator): everything the separator accepts.
