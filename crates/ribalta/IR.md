# Ribalta IR

The compiler turns Ribalta source into a JSON document, the IR (`Document` in `src/ir.rs`, mirrored in `packages/compiler/src/types.ts`). A renderer draws pages from the IR alone. This file is the contract for writing a renderer outside the web app, such as a native one.

Field names are camelCase. Optional fields are left out when empty. Colors are hex strings (`#rrggbb` or `#rrggbbaa`). Sizes are pixels on the canvas of each page, `page.width` by `page.height`. A page with `height: null` has a fixed width and grows with its content (a document, a long screen); otherwise the canvas is fixed and the renderer scales it whole to fit the screen. The host chooses the default size when it compiles (a presentation app, for example, asks for 1920 by 1080); the source can change it with `@page` and on each separator.

## Document

| Field | Meaning |
| --- | --- |
| `version` | Language version the source was written for. |
| `meta` | `title`, `author`, `page` (the default size `{width, height}`, `height` `null` for automatic); for `@mode game`, `mode: "game"` (see [Behavior](#behavior)); `random: true` when some action draws with `random` or `pick`. |
| `theme` | Colors, fonts, text styles and every default the renderer needs (see [Theme](#theme)). |
| `state` | Global variables with their initial values: `bool`, `number`, `string`, a list of strings (a set: no repeated items, in insertion order) or, only for `externs`, a list of records (objects whose fields are `bool`, `number` or `string`; every record has the fields and field types of the first). |
| `icons` | Drawing of every icon the document uses, by name (see the `icon` block). |
| `externs` | Names of global state variables that the host sets (`@extern`). Their entries in `state` are preview values: a host replaces them whenever it wants, actions never change them, and they are not stored in shared addresses or persisted state. |
| `persist` | Names of global state variables to keep on the viewer's device between visits. A viewer may store them (by address and title) whenever they change and use them instead of the initial values when a new session starts; a session restored from saved state such as a URL ignores them. |
| `scorm` | Present when the document has `@scorm`: `score` (name of a numeric global state variable, optional), `complete` (page ids that complete the course, may be empty), `max` (highest score, default `100`) and `pass` (passing score, optional). A viewer packaged for an LMS reports the score clamped to `0..max` whenever it changes, and completion ("passed"/"failed" against `pass`, or "completed") once one of the `complete` pages is reached; without `scorm`, completion is the last page. |
| `pages` | The pages in order. |

## Page

| Field | Meaning |
| --- | --- |
| `id` | Target for `go` actions and `#id` links. |
| `width`, `height` | Size of this page in pixels; `height` is `null` when it follows the content. |
| `steps` | Number of reveal steps. Step `0` shows only blocks without a step. |
| `style` | Page style, merged over `theme.page`. |
| `children` | Content blocks, laid out as a column (see [Layout](#layout)). |
| `header`, `footer` | Optional blocks drawn in bands at the top and bottom. |
| `overlay` | Optional block drawn over everything, inside the page padding, without taking space. |
| `hidden` | Skipped by linear navigation; only reachable through `go`. |
| `next` | Page id that "next" jumps to after the last step. |
| `state` | Variables local to the page, reset every time the page is entered. |
| `transition` | `fade`, `slide`, `zoom` or `none`, played when the page appears (see `theme.motion.transitions`). |
| `locked` | When `true`, linear navigation never leaves the page (see [Behavior](#behavior)). |
| `clock` | When `true`, the page counts seconds while it is shown (it uses `elapsed` or has a `timeout`). |
| `timeout`, `onTimeout` | Seconds after which `onTimeout` (any actions, navigation included) runs once. |
| `onEnter` | Optional list of `set` and `toggle` actions run every time the page is entered (see [Behavior](#behavior)). Never contains navigation actions. |
| `notes` | Notes for the person showing the document, such as speaker notes (plain text, never drawn). |
| `title`, `line`, `endLine` | Editor metadata. |

## Blocks

Every block has `type`, `style`, `line` and may have `step`, `when`, `variants` and `click` (see [Behavior](#behavior)). `endLine` and `source` are editor metadata.

| `type` | Fields | Drawing |
| --- | --- | --- |
| `heading` | `level` 1 to 6, `content` | Text with `theme.headings[level - 1]` under the block style. |
| `paragraph` | `content` | Text. |
| `list` | `ordered`, `start`, `items` | Items with marker, see `theme.elements.list`. Each item has `content`, nested `children` and its own style and behavior. |
| `quote` | `children` | Column of blocks with a side bar, see `theme.elements.quote`. |
| `each` | `list`, `item`, `index?`, `children` | Repetition decided while presenting. Evaluate `list` (always a `state` expression); for every element, draw `children` in place, as if they were children of the parent block, with the element bound to `item` and, when `index` is present, its position (1, 2, ...) bound to `index`. Draws nothing when the list is empty or not a list. Has no style or behavior of its own. |
| `code` | `lang`, `code`, `lines` | `lines` is the code split in lines of tokens `{kind, text}`. Paint each token with `theme.elements.code.syntax[kind]`; an empty line still takes one line of height. `code` is the raw text. |
| `table` | `header`, `aligns`, `rows` | Cells are inline content; `aligns` is `left`, `center` or `right` per column. See `theme.elements.table`. |
| `image` | `src`, `alt`, `fit` | `src` is `{kind: "asset", id}` or `{kind: "url", url}`. An asset id is opaque: the host that shows the document turns it into an address (the Folio app, for example, serves uploaded files at `/api/v1/assets/{id}`); without a host that knows assets, leave the image empty. The image never grows past its container; with a fixed `w`/`h`, `fit` is `cover` (fill and crop) or `contain` (fit inside). |
| `layout` | `layout`, `cols`, `widths`, `wrap`, `children` | Container, see [Layout](#layout). |
| `chart` | `chart`, `labels`, `series`, `legend`, `stacked`, `values`, `scene` | Draw `scene`, see [Charts](#charts). The other fields describe the data. |
| `icon` | `name` | Draw `document.icons[name]`: a list of `{tag, attrs}` SVG elements (`path`, `circle`, `rect`, `line`, `polyline`, `polygon`, `ellipse`) on a 24 by 24 grid, stroked in the text color with round caps and joins and no fill. Scale it to `style.size` or `theme.elements.icon.size`; the stroke is `theme.elements.icon.stroke` in grid units. The full set lives in `icons/icons.json` (Lucide, ISC license). |
| `spacer` | `size` | Empty space of `size` pixels along the parent's main axis. |
| `divider` | | Horizontal rule across the parent with half a line (0.5 em) of space above and below, see `theme.elements.divider`. |
| `input` | `name`, `numeric?`, `placeholder?`, `onSubmit?` | A text field (a number field when `numeric`) showing the state `name`. Every edit runs a `set` of `name` to the typed text (or its numeric reading, 0 when empty). Enter runs `onSubmit`; without `onSubmit`, Enter runs the page's `key` shortcut for `Enter`, if any. Typing and clicking in the field never move the presentation. Where the page is not interactive (thumbnails, print), draw it disabled with the initial value or the placeholder. |
| `sound` | `src`, `name?`, `autoplay?`, `loop?`, `volume?` | Nothing is drawn. While the page is shown live (not on a presenter screen that has an audience screen), load the audio: `autoplay` starts it when the page is entered, `loop` repeats it, `volume` is 0 to 1. Every sound of the page stops when another page is entered. |

## Inline content

| `type` | Fields | Drawing |
| --- | --- | --- |
| `text` | `value` | Plain text. |
| `strong`, `emphasis`, `strike` | `children` | Bold, italic, strikethrough. |
| `code` | `value` | See `theme.elements.inlineCode`. |
| `link` | `href`, `children` | See `theme.elements.link`. `#id` runs `go` to that page; anything else opens outside the document. |
| `span` | `style`, `children`, behavior | Styled run of text. `sourceAttrs` is editor metadata. |
| `image` | `src`, `alt`, `style` | Image one line tall, aligned to the middle of the text. |
| `field` | `name` | `date`: today's date, long form, in the viewer's language. |
| `expr` | `expr` | Evaluate and print the result (see [Expressions](#expressions)). |
| `break` | | Line break. |

## Style

All fields are optional; a block style is merged over the defaults of its kind.

| Field | Meaning |
| --- | --- |
| `color` | Text color, inherited by children. |
| `bg` | `{kind: "solid", color}` or `{kind: "linear", angle, stops: [{color, at}]}`. `angle` is in degrees with CSS meaning (0 points up, 90 points right); `at` goes from 0 to 1. |
| `bgImage` | Image source drawn over `bg`, covering the box and centered. On the page, a translucent solid `bg` is drawn over the theme background. |
| `font` | Font family name (see `theme.fontFamilies`). |
| `size` | Font size in pixels. |
| `weight` | 100 to 900. |
| `italic`, `uppercase`, `underline` | Booleans. |
| `align` | `left`, `center`, `right`: text alignment, and horizontal placement of children in containers. |
| `valign` | `top`, `center`, `bottom`: vertical placement of children in containers. |
| `pad` | `[vertical, horizontal]` padding in pixels. |
| `gap` | Space between children in pixels. |
| `radius` | Corner radius in pixels (9999 means fully round). |
| `border` | `{width, color}`, solid, on all sides. |
| `shadow` | `none`, `sm`, `md`, `lg`, see `theme.elements.shadows`. |
| `w`, `h` | `Length`: `{unit: "px", value}`, `{unit: "pct", value}` (percent of the parent) or `{unit: "auto"}`. Sizes include padding and border. |
| `x`, `y` | `Length`. When either is set the block leaves the flow and is placed at that offset from the top left corner of the page, ignoring the page padding (a missing one counts as 0). It is drawn above the blocks in the flow and keeps the text style it inherits from its parents. |
| `opacity` | 0 to 1. |
| `leading` | Line height as a multiple of the font size. |
| `tracking` | Letter spacing in em. |
| `enter` | Entry animation: `fade`, `up`, `down`, `left`, `right`, `zoom` or `none`. |

## Layout

The page is a box of `page.width` by `page.height` (or as tall as its content when `height` is `null`), padded by `style.pad`. Without header or footer its children form a column with `style.gap` between them, aligned by `align` (children stretch to the full width when it is missing) and `valign` (top when missing). The built-in themes set `align: left` and `valign: center` in `theme.page`.

With a header or footer, the page becomes three rows: header band, content, footer band, separated by `theme.layout.band.gap`. The vertical padding next to a band shrinks to `pad[0] * band.edgeScale`, and a band is at least `pad[0] * band.heightScale - band.gap` tall. The content row takes the remaining height and lays out its children as above. The overlay covers the page, padded like the page.

Layout blocks. The gap is `style.gap`, or `theme.layout.gaps[layout]` when missing:

| `layout` | Rule |
| --- | --- |
| `stack` | Column across the full width of the parent. `align` places children horizontally (stretch when missing), `valign` places the group vertically (top when missing). |
| `box` | Like `stack`, but it follows the parent's alignment instead of always taking the full width. Used for cards: padding, background, border and shadow come from the style. |
| `row` | Children side by side at their natural size. `align` places the group horizontally (start when missing), `valign` aligns children vertically (center when missing). With `wrap`, children move to a new line when out of space. |
| `columns` | Children side by side, sharing the full width in proportion to `widths` (equal shares when missing), each one a column of its own. `valign` aligns columns vertically (center when missing). |
| `grid` | `cols` equal columns (`theme.layout.gridColumns` when missing), filled row by row. Cells stretch to the row height unless `valign` says otherwise. |

Children shrink before they overflow; content that still does not fit is clipped by a page of fixed height (a page of automatic height grows instead; there the content row takes the height of its children and `valign` has no room to act). A chart without `h` is at most `theme.layout.chartMaxHeight` tall.

## Behavior

- `step`: the block is shown once the current step is at least `step`. While hidden it keeps its space. Showing it animates from `theme.motion.enter[style.enter or "up"].from` to the resting pose with `theme.motion.reveal` timing; `enter: "none"` appears at once.
- `when`: an expression; the block is left out entirely while it is not `true`. When it appears and `style.enter` is set (and not `none`), play `theme.motion.enter[style.enter]`.
- `variants`: list of `{when, style}`. Every variant whose `when` holds is merged over the block style, in order. Style changes animate with `theme.motion.variant` timing.
- `click`: list of actions run in order when the block is clicked or tapped. A clickable element should also be reachable with the keyboard (focusable, activated by Enter and Space without navigating).
- `key`: a shortcut for `click`: a lowercase letter or digit, `ArrowUp`, `ArrowDown`, `ArrowLeft`, `ArrowRight`, `Enter` or `Space`. While the page is shown live, pressing that key (without Ctrl, Alt or Meta, and not while typing in a field) runs the `click` of the first visible element that claims it (its `when` holds and its `step` is revealed), and the key does not navigate.

Actions:

| `type` | Effect |
| --- | --- |
| `set` | `name` = value of the expression `value`. |
| `toggle` | `name` = not (`name` is `true`). |
| `add` | Append the printed value of `value` to the list `name`, unless it is already there. |
| `remove` | Remove the printed value of `value` from the list `name`. |
| `clear` | Make the list `name` empty. |
| `play`, `stop` | Start the sound `name` of the current page from the beginning, or stop it. The session records each one as a cue `{id, type, name}` (the last 32 are kept) so that mirrored screens play them too; navigation and state never depend on cues. |
| `go` | Jump to page `target` at step 0, remembering where we were. |
| `back` | Return to the last remembered place and restore its local state; without history, go to the previous step. |
| `next`, `prev` | Same as the viewer's next and previous. |

Navigation: next advances the step and, after the last one, goes to `page.next` or the next page that is not hidden; previous does the reverse and lands on the last step of the previous page. From a hidden page reached by `go`, leaving its last step goes back. Entering a page resets its local `state` and then runs its `onEnter` actions, in order, with `$step` at the step where the page was entered. This happens when the session starts, on next and previous, on `go` and on `page.next`, but not on `back` (which restores the remembered place) nor when a session is restored from saved state such as a URL. Global state lives for the whole session.

Clock: while a page with `clock` is shown live, the viewer adds 1 to `elapsed` every second. Entering a page (any way, including `back` and `go` to the same page) sets `elapsed` to 0. When `elapsed` goes from below `timeout` to `timeout` or more, `onTimeout` runs once, like a click. Static renderings (thumbnails, print) use `elapsed` 0, and a session restored from saved state starts again at 0. An audience screen that mirrors a presenter shows the presenter's `elapsed` instead of counting on its own.

Locked pages (`locked: true`): the viewer's next and previous still move through the page's steps, but stop at its first and last step instead of leaving the page, and leaving a hidden page never goes back on its own. The `next` and `prev` actions are explicit choices and keep working as described above. When `meta.mode` is `"game"`, every page is locked unless it opts out, and viewers should hide page numbers, thumbnails and any jump to the last page.

The files in [`conformance/`](conformance) check an implementation of these rules: `expressions.json` has expressions with their expected values and printed text, `navigation.json` has small documents with sequences of operations and the expected position and state after each one. The web app runs them in `web/src/__tests__/conformance.test.ts`.

## Expressions

`expr` nodes: `lit {value}`, `state {name}` (unknown names are `false`), `item {name, field?}` (inside an `each`: the element bound to `name`, or the field `field` of that record; `false` outside it or for a missing field), `step` (current step), `elapsed` (whole seconds since the page was entered), `hover` (only in variant conditions: `true` while a mouse or pen pointer is over the element or it has keyboard focus, never for touch, and `false` wherever there is no pointer, such as thumbnails and print), `not {arg}`, `neg {arg}`, `binary {operator, left, right}`, `cond {test, then, otherwise}` and `call {name, args}`.

- Conditions hold only when the value is exactly `true`. `and` and `or` return booleans and treat each side as a condition.
- `==` and `!=` compare type and value; the compiler never emits them with a list on either side. `<`, `<=`, `>`, `>=` compare numbers, or strings by code point.
- `in` is `true` when the right side is a list and contains the left side, which is a string; otherwise `false`.
- `+` concatenates when either side is a string (numbers print as below), otherwise adds. `-`, `*`, `/`, `%` work on numbers; a value that is not a number counts as its numeric reading or 0. Division or remainder by 0 gives 0.
- `call` names are closed: `round(x)` and `round(x, digits)` round half away from zero to `digits` decimal places (`digits` truncated to 0..10); `min(...)` and `max(...)` take two or more numbers; `len(x)` is the number of items of a list (of strings or of records) or the number of characters (code points) of a string. `random(a, b)` and `pick(...)` only appear inside actions and draw from the session's generator (see below).
- Random draws: the session keeps a 32-bit generator state `seed`. One draw is mulberry32: `seed = (seed + 0x6D2B79F5) mod 2^32`; `t = imul(seed ^ (seed >>> 15), seed | 1)`; `t ^= t + imul(t ^ (t >>> 7), t | 61)`; the draw is `((t ^ (t >>> 14)) >>> 0) / 2^32`, a number in [0, 1). `random(a, b)` takes `low = ceil(min(a, b))` and `high = max(low, floor(max(a, b)))` and returns `low + floor(draw * (high - low + 1))`. `pick(...)` spreads list arguments into items and returns `items[floor(draw * count)]`, or `""` when there are no items. Calls draw in evaluation order (left to right) and actions run in order, so the same starting seed gives the same results. A new session starts from a random seed; viewers that save the session (such as the shared link's address) should save the seed too when `meta.random` is set.
- Printing: integers print without a decimal point, other numbers in their shortest form (`0.5`), booleans as `true`/`false`, lists as their items joined by `, ` (an empty list prints nothing).

## Theme

| Field | Meaning |
| --- | --- |
| `name` | Built-in theme the document starts from. |
| `colors` | Named colors after the document's overrides (`bg`, `text`, `muted`, `accent`, `surface`, `line`, ...). |
| `fonts` | Role (`body`, `display`, `mono`, custom) to family. |
| `fontFamilies` | `[{family, weights}]` to load. Families the renderer does not bundle come from the host. The Folio app lists their files at `GET /api/v1/fonts/manifest?family=Name:wght@400;700` (spaces as `+`), which returns `{faces: [{family, style, weight_min, weight_max, url, format, unicode_range}]}`. The default is woff2 split by `unicode_range`; add `&format=ttf` for one TrueType file per weight, which Android and iOS load directly. |
| `text` | Base text style of every page. |
| `headings` | Styles for heading levels 1 to 6. |
| `page` | Default page style (background, text color, padding, gap, alignment). |
| `palette` | Colors used in order by chart series and slices. |
| `motion` | Animations, below. |
| `elements` | Decoration of each element, below. |
| `layout` | Layout defaults: `gaps` per layout kind, `gridColumns`, `band {gap, edgeScale, heightScale}`, `chartMaxHeight`. |

`motion`:

- `enter[name]` and `transitions[name]`: `{duration, easing, from}`. Animate from the `from` pose to the resting pose (opacity 1, no offset, scale 1).
- `from` is `{opacity, dx?, dy?, scale?}`; `dx` and `dy` are `Length` (`pct` is relative to the element's own size).
- `reveal` and `variant`: `{duration, easing}`.
- `duration` is in milliseconds. `easing` is a cubic Bézier `[x1, y1, x2, y2]`.
- When the system asks for reduced motion, skip enter and transition animations.

`elements`:

| Field | Values |
| --- | --- |
| `shadows.sm/md/lg` | `{x, y, blur, color}` drop shadow. |
| `list` | `gapEm` between items, `indentEm` from the left, marker `markerColor` and `markerWeight`, nested lists get `nestedGapEm` above and `nestedScale` font size. Ordered lists number from `start` with `1.`, unordered use a bullet. |
| `quote` | `gapEm` between children, `indent` before the text, bar `barWidth` and `barColor` on the left, `italic`. |
| `code` | `font`, `size`, `leading`, `pad`, `radius`, `bg`, `color` (plain text), `syntax` (token kind to `{color, italic}`). Code never wraps; overflow is clipped. |
| `inlineCode` | `font`, `scale` (of the surrounding size), `padEm`, `radiusEm`, background of the text color at `tint` opacity. |
| `link` | `color`, underline `underlineEm` thick at `underlineOffsetEm` below the baseline. |
| `table` | `size`, `cellPad`, rule under each row `ruleWidth` in `ruleColor`, under the header `headerRuleWidth`, header `headerWeight`. |
| `divider` | `thickness`, `radius`, `color`. |
| `icon` | Default `size` and `stroke`. |

## Charts

`scene` is `{width, height, shapes}` in its own coordinate space. Scale it uniformly to fit the block width (and `style.h`, when set), centered. Angles are in degrees, clockwise from the positive x axis (screen coordinates, y down).

Shapes, drawn in order:

| `shape` | Fields |
| --- | --- |
| `rect` | `x`, `y`, `w`, `h`, `radius`, `ink`, `opacity` (fill). |
| `line` | `x1`, `y1`, `x2`, `y2`, `width`, `ink`, `opacity` (stroke). |
| `polyline` | `points`, `width`, `ink`, `opacity`: stroke with round joins and caps. |
| `polygon` | `points`, `ink`, `opacity` (fill). |
| `circle` | `cx`, `cy`, `r`, `ink`, `opacity` (fill). |
| `sector` | `cx`, `cy`, `outer`, `inner` (0 for a full wedge), `start`, `end`, fill `ink`, outline `stroke` of `strokeWidth`. |
| `text` | `x`, `y`, `text`, `anchor` (`start`, `middle`, `end`), `baseline` (`alphabetic`, or `middle` to center on `y`), `size`, `weight`, `ink`, `opacity`, in the page's body font. |

`ink` is `{kind: "text"}` (the block's text color), `{kind: "background"}` (the page background color) or `{kind: "color", color}`.
