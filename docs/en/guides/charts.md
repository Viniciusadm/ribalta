---
title: Charts
description: Turn a table into a bar, line, area, pie or donut chart with the chart block.
---

In Ribalta, a chart is a table inside a `::: chart` block. You write the numbers as text and Ribalta draws the chart with the theme colors.

## From text to chart

```ribalta preview
# Sales by quarter
::: chart
| Quarter | Sales |
| --- | ---: |
| Q1 | 120 |
| Q2 | 145 |
| Q3 | 162 |
| Q4 | 190 |
:::
```

The table follows a simple rule:

- **The first column** holds the labels (the horizontal axis, or the slices of a pie).
- **Each of the other columns** is a series of values. The column header is the series name.

With no attributes, the chart is a bar chart.

## Chart types

Choose the type with the `type` attribute:

| Value | Chart |
| --- | --- |
| `bar` | Vertical bars. This is the default. |
| `line` | Lines. |
| `area` | Lines with the area below them filled in. |
| `pie` | Pie. |
| `donut` | Donut (a pie with a hollow center). |

### Bar

```ribalta preview
# Revenue by region
::: chart {type=bar values}
| Region | 2025 | 2026 |
| --- | ---: | ---: |
| North | 32 | 41 |
| South | 28 | 45 |
| East | 40 | 52 |
:::
```

### Line

```ribalta preview
# Active users
::: chart {type=line}
| Month | Web | Mobile |
| --- | ---: | ---: |
| Jan | 1200 | 800 |
| Feb | 1350 | 1100 |
| Mar | 1500 | 1480 |
| Apr | 1620 | 1900 |
:::
```

### Area

```ribalta preview
# Website traffic
::: chart {type=area}
| Week | Visits |
| --- | ---: |
| W1 | 4200 |
| W2 | 5100 |
| W3 | 4800 |
| W4 | 6300 |
:::
```

### Pie and donut

In `pie` and `donut`, only the **first series** is used, and each table row becomes a slice:

```ribalta preview
# Where customers come from
::: columns {gap=xl}
::: chart {type=pie values}
| Channel | Customers |
| --- | ---: |
| Referral | 45 |
| Search | 30 |
| Events | 25 |
:::
::: chart {type=donut values}
| Channel | Customers |
| --- | ---: |
| Referral | 45 |
| Search | 30 |
| Events | 25 |
:::
:::
```

## Options

| Attribute | What it does |
| --- | --- |
| `type=bar` | Chart type: `bar`, `line`, `area`, `pie` or `donut`. |
| `values` | Shows the value of each point or bar. In `pie` and `donut`, shows the percentage of each slice. |
| `legend` | Shows the legend. By default, it appears when there is more than one series and in `pie` and `donut`. Use `legend=false` to hide it. |
| `stacked` | Stacks the series instead of placing them side by side. Applies to bar charts. |
| `colors="brand coral"` | Series colors, in column order, separated by spaces. Accepts theme colors and `#hex`. |
| `w`, `h` | Chart width and height, such as `h=480` or `w=60%`. |

The yes-or-no attributes (`values`, `legend`, `stacked`) can be written on their own to turn them on, or with `=false` to turn them off.

```ribalta preview
@color brand #1e83d6

# Costs by department
::: chart {stacked values colors="brand coral" h=640}
| Department | People | Tools |
| --- | ---: | ---: |
| Sales | 40 | 12 |
| Support | 25 | 8 |
| Product | 55 | 20 |
:::
```

:::note
Without `colors`, the series use the theme palette, in order. In `pie` and `donut`, the slices always use the theme palette.
:::

## Charts from data

With a table declared in `@data`, the chart can be a single line. `label` picks the field with the labels and `series` the fields with values:

```ribalta preview
@data sales
| quarter | revenue | cost |
| ---     | ---     | ---  |
| Q1      | 120     | 80   |
| Q2      | 150     | 95   |
| Q3      | 190     | 110  |
@end

:: chart {data=$sales type=area series=revenue}
```

The same data can feed other charts, tables (`:: table {data=$sales}`) and repetitions. See [Variables, data and repetition](/en/guides/data/).

## Numbers

- Use a dot or a comma as the decimal separator: `3.5` and `3,5` mean the same.
- A trailing `%` is accepted and ignored: `38%` means `38`.
- Spaces inside a number are ignored: `1 200` means `1200`.
- Do not use commas or dots as thousands separators: `1,200` is read as one point two.
- A cell missing from a row counts as zero.

## Common errors

| Code | When it happens |
| --- | --- |
| R111 | The `chart` block has no table. |
| R112 | A value cell is not a number, such as `abc` or `$10`. The chart uses zero instead. |
| R113 | The table only has the label column, with no value columns. |
| R301 | Invalid chart type, unknown color in `colors` or invalid value in `values`, `legend` or `stacked`. |
