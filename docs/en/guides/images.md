---
title: Images
description: Use uploaded images or external addresses, adjust how they fit and use background images.
---

An image in Ribalta is written as in Markdown: `![description](address)`. The address can be an image uploaded to the project or an `https://` link.

## Uploaded images

The program you write in may let you upload image files to the project. An uploaded image is referenced with `asset:` followed by its identifier:

```ribalta
![sales-chart](asset:3f2b8c1e-7a4d-4e9b-9c2a-1d5e6f7a8b9c)
```

Write a description in brackets that says what the image shows.

:::tip
The image belongs to the project. To use it in another document of the same project, copy the `![…](asset:…)` line.
:::

## External images

You can also use an image published on the web, with an address that starts with `https://`:

```ribalta preview
# Our office
![Mountains at sunrise](https://images.unsplash.com/photo-1501785888041-af3ef285b470?w=1200){h=640 radius=lg}
```

:::caution
An external image depends on the site that publishes it. If it is removed or moves to another address, it no longer shows up in your document. For important images, upload the file instead.
:::

## Adjusting the image

An image on a line of its own is a block. Attributes go in braces, right after the address:

| Attribute | What it does |
| --- | --- |
| `fit=contain` | Shows the whole image inside the area. This is the default. |
| `fit=cover` | Fills the whole area, cropping the edges if needed. |
| `w`, `h` | Width and height, in pixels (`480`), percentage (`50%`) or `full`. |
| `radius` | Rounds the corners: `sm`, `md`, `lg` or pixels. |
| `shadow` | Shadow: `sm`, `md`, `lg` or `none`. On its own, it means `md`. |
| `border` | Border, such as `border="4 accent"`. |
| `opacity` | Transparency, from `0` to `1` or as a percentage. |

`fit` only makes a difference when the area has a different shape from the image, for example when you set both `w` and `h`:

```ribalta preview
::: columns {gap=lg}
![Whole landscape](https://images.unsplash.com/photo-1501785888041-af3ef285b470?w=1200){fit=contain h=560}
![Cropped landscape](https://images.unsplash.com/photo-1501785888041-af3ef285b470?w=1200){fit=cover h=560 radius=lg shadow}
:::
```

To place text next to an image, use columns. See [Layout](/docs/en/guides/layout/).

## Images within text

An image inside a paragraph flows with the text and is as tall as the letters. Attributes go right after it, with no space:

```ribalta preview
# Partners
Supported by: ![logo](https://images.unsplash.com/photo-1501785888041-af3ef285b470?w=1200){radius=sm} Horizon Foundation {size=2xl}
```

To change its size, change the paragraph `size` or set `h` on the image.

## Background images

The `image` attribute places an image in the background of a page or a block. It covers the whole area and stays centered:

```ribalta preview
--- {image=https://images.unsplash.com/photo-1501785888041-af3ef285b470?w=1200 color=white align=center}
::: box {bg=ink/60 pad=xl radius=lg}
# Planning 2027 {size=5xl}
Annual leadership meeting {.lead}
:::
```

`image` accepts the same addresses as regular images: `https://` or an uploaded image (`image=asset:…`).

```ribalta
--- {image=asset:3f2b8c1e-7a4d-4e9b-9c2a-1d5e6f7a8b9c}
::: box {image=asset:3f2b8c1e-7a4d-4e9b-9c2a-1d5e6f7a8b9c h=400 radius=lg}
:::
```

:::tip
Text on top of a photo is often hard to read. Put it in a `box` with a semi-transparent background, such as `bg=ink/60`, or pick a dark image and use `color=white`.
:::

## Common errors

| Code | When it happens |
| --- | --- |
| R301 | Address not allowed: it does not start with `https://` and is not an uploaded image (`asset:` followed by the identifier). |
| R301 | `fit` with a value other than `cover` or `contain`. |
| R209 | Unknown attribute on the image, such as `fitt`. |
