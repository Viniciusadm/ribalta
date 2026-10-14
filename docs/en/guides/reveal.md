---
title: Step-by-step reveal
description: Show a page's content gradually, one block, item or phrase at a time.
---

Revealing content step by step helps people follow your reasoning: each point appears when you get to it, not all at once. Ribalta offers two ways to do this: the `::: steps` block and the `step` attribute.

## The `steps` block

Everything inside `::: steps` appears in order, one at a time, each time you move forward:

```ribalta preview
# Why change now
::: steps {gap=md}
Operating costs rose 18% in one year.

Competitors have already automated customer service.

Customers expect same-day answers.
:::
```

Each child block (a paragraph, a heading, an image, another block) is one step.

### Lists

When the child is a list, each **item** becomes a step:

```ribalta preview
# Plan for the semester
::: steps
- Map the current processes
- Choose the tools
- Train the teams
- Measure the results
:::
```

The `steps` block arranges its children in a vertical stack and accepts the same attributes as `::: stack`, such as `gap` and `align`.

## Explicit steps with `step`

To control the order yourself, add the `step=N` attribute to any block. The block appears at step `N`:

```ribalta preview
::: columns {gap=xl}
::: box {.card}
## Today
Reports built by hand, once a month.
:::
::: box {.card step=1}
## In six months
Automatic reports, updated every day.
:::
:::
```

The attribute also works on:

- **list items**: `- Third point {step=3}`;
- **text spans**: `The goal is to [double revenue]{step=1 color=accent}.`;
- **components**: `:: metric {value=38% label="growth" step=2}`.

```ribalta preview
# Goal for 2027
Revenue was $12 million. [The goal is to reach $20 million.]{step=1 color=accent weight=bold}
```

Step numbers go from 1 to 99. Several elements with the same number appear together.

## How steps combine

- Elements without a step are visible from the start of the page.
- The page has as many steps as the highest number used in it.
- A `::: steps` block continues counting from the highest step that appeared before it on the page. If a heading has `{step=2}` and a `::: steps` with three items follows it, the items land on steps 3, 4 and 5.
- Inside a `::: steps` block, the order is set by the block itself. Do not use `step` on its direct children.

```ribalta preview
# Results {step=1}
::: steps
- Revenue: +38%
- Customers: +94
- Satisfaction: 4.8
:::
```

In this example, the heading appears at step 1 and the items at steps 2, 3 and 4.

## Replacing instead of adding

Regular steps only add content. With `$step`, the current step of the page, you can highlight the current item, fade out the previous ones or swap one piece of content for another:

```ribalta preview
@style faded {opacity=0.35}

## Plan for the semester
- Map the processes {step=1 .faded="$step > 1"}
- Pick the tools {step=2 .faded="$step > 2"}
- Train the teams {step=3}
Only on the last step: **all set!** {if="$step == 3" enter=zoom}
```

The page has as many steps as the largest number used in `step`, in `::: steps` or in a comparison with `$step`. For more, use `--- {steps=N}`. See also [Interactivity and navigation](/en/guides/interactive/#steps-with-step).

## How elements appear

The `enter` attribute picks the animation: `fade`, `up` (default), `down`, `left`, `right`, `zoom` or `none`. It can live in a `@style` to apply to several elements: `@style pop {enter=zoom}`.

## Navigating through steps

While the document is shown, each move forward shows the next step of the current page. Once every step is visible, the next move goes to the following page.

- **→**, **Space** or a click move forward one step.
- **←** goes back one step. Going back from the start of a page takes you to the previous page with all its steps already revealed.

:::note
A static rendering of the document, such as an image or a printout, shows each page in its final state, with every step revealed.
:::

## Common errors

| Code | When it happens |
| --- | --- |
| R301 | `step` with no value or with a value that is not a whole number from 1 to 99, such as `{step}`, `step=0` or `step=a`. |
