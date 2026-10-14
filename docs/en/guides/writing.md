---
title: Writing pages
description: Headings, paragraphs, emphasis, links, lists, quotes, code, tables, images, comments and notes.
---

You write the content of a page as in Markdown, with one important difference: on a page, every line is intentional, so a line break in the text is a line break on the page.

## Headings

Start the line with `#` followed by a space. Use one to six `#` for the heading levels:

```ribalta preview
# Main heading
## Subheading
### Section
Regular page text.
```

The size, weight and font of each level come from the theme. The first heading on a page also gives the page its title.

## Paragraphs and line breaks

Consecutive lines form a paragraph, and each line appears on its own line on the page. A blank line separates one block from the next:

```ribalta preview
# Next meeting
Thursday, at 2 pm
Room 3, second floor

Bring last month's numbers.
```

:::note
Unlike traditional Markdown, you do not need to end a line with two spaces to break it. To keep two sentences on the same line of the page, write them on the same line of the text.
:::

## Bold, italic, strikethrough and code

| You write | Result |
| --- | --- |
| `**bold**` | **bold** |
| `*italic*` | *italic* |
| `~~strikethrough~~` | ~~strikethrough~~ |
| `` `code` `` | `code` |

The markers can be combined:

```ribalta preview
# Emphasis
Text in **bold**, *italic* and ~~strikethrough~~.
Code in the middle of a sentence: `npm install`.
**Bold with *italic* inside.**
```

## Links

Write the text in brackets and the address in parentheses. Addresses that start with `https://` or `http://` and email addresses with `mailto:` are accepted:

```ribalta preview
# Contact
Learn about the project on [our website](https://example.com).
Questions: [email@email.com](mailto:email@email.com)
```

## Lists

Start each item with `-` (or `*`, or `+`). For numbered lists, use `1.` or `1)`. Indent an item to create a nested list:

```ribalta preview
# Action plan
- Map the current processes
- Set the priorities
  - Short term
  - Long term
- Present to the board

---
# Stages
1. Assessment
2. Proposal
3. Rollout
```

A few useful rules:

- Numbering starts at the first number you write: a list that starts with `3.` continues with 4, 5 and so on.
- An indented line right below an item, without a marker, continues the same item on a new line.
- A blank line ends the list.

## Quotes

Start the lines with `>`. Everything after the `>` is treated as regular content, so you can use emphasis and several lines:

```ribalta preview
> The report that used to take a week is now ready in an hour.
> — Operations manager
```

## Code blocks

Put the code between two lines with three backticks. Add the language after the opening backticks to get syntax highlighting:

````ribalta preview
# Sales query
```sql
SELECT name, total
FROM sales
WHERE year = 2026;
```
````

The content of a code block appears exactly as written. Lines with `---` inside it do not start a new page.

## Tables

Separate the columns with `|` and put a line of dashes right below the header. Use `:` in the dash line to align the column:

```ribalta preview
# Plans
| Plan | Users | Monthly price |
| :--- | :---: | ---: |
| Basic | 5 | $9 |
| Team | 20 | $29 |
| Business | Unlimited | $79 |
```

| Dash line | Alignment |
| --- | --- |
| `---` or `:---` | Left |
| `:---:` | Centered |
| `---:` | Right |

Cells accept bold, italic, code and links. Tables also work as the data source for [charts](/docs/en/guides/charts/).

## Images

Use the Markdown image syntax: `!`, the description in brackets and the address in parentheses. Attributes attached right after adjust the size and cropping:

```ribalta preview
# Our headquarters
![Mountain landscape](https://images.unsplash.com/photo-1501785888041-af3ef285b470?w=1200){h=600 radius=lg}
```

The address must start with `https://` or be an image uploaded to the project (`asset:`). See [Images](/docs/en/guides/images/) for uploaded images and `fit=cover`.

:::caution
Leave a blank line between a paragraph and an image. Without it, the image becomes part of the paragraph, like an image in the middle of the text.
:::

## Comments

A line that starts with `//` is a comment: it does not appear on the page or in the notes. Use it for reminders or to hide some content temporarily:

```ribalta preview
// Confirm the numbers with finance
# Goals for the half
Expand into two new regions.

// Cut costs by 10%.
Improve customer response times.
```

`//` only counts at the start of a line. In the middle of a sentence, as in an `https://` address, it is plain text.

## Escaping characters

Some sequences have a special meaning, such as `*`, `#` at the start of a line or `{` at the end. To write the character itself, put a backslash `\` before it:

```ribalta preview
# Price list \{2026}

Use \*asterisks\* without italics.

\# This line is not a heading.

\- This line is not a list item.
```

The backslash works before any punctuation mark. Inside a table, `\|` writes a vertical bar without creating a new column.

## Notes

The `::: notes` block holds notes for whoever shows the document, such as what you want to say on each page of a presentation. Notes are never drawn on the page; the program that shows the document decides where to display them:

```ribalta preview
# Quarterly results
Revenue grew 38%.

::: notes
Mention that the growth came mostly from new customers.
Open the floor for questions before the next page.
:::
```

The block starts with `::: notes` and ends with a line containing only `:::`. The text inside it is kept as is, without formatting.

## Next steps

- [Layout](/docs/en/guides/layout/): arrange content in columns, rows and grids.
- [Styling](/docs/en/guides/styling/): change colors, sizes and fonts.
- [Syntax reference](/docs/en/reference/syntax/): the whole language on one page.
