---
title: "Interactivity and navigation"
description: "Use @state, clicks, variants and go to build documents that react to whoever uses them and follow different paths."
---

Some documents are not linear: you want to open a detail only if someone asks, compare two scenarios with one click, or jump to a section and come back. Ribalta does this without JavaScript, through a small layer of **state**:

| Feature | What for |
| --- | --- |
| `@state` | A variable that changes while the document is shown. |
| `click="…"` | Actions on click: change state, go to another page, go back. |
| `if` | Show or hide content based on state. |
| `.style=condition` | Apply a style only when the condition is true. |
| `$name` in text | Show the current value of a variable. |
| `@state x = []` | A list of texts that changes while the document is shown, such as an inventory. |
| `random`, `pick` | Draw a number or an item inside an action. |
| `$elapsed`, `timeout` | Count time on the page and act when it runs out. |
| `:: input` | A text field for people to type in. |
| `key=…` | A key that runs an element's click. |
| `:: sound`, `play` | Sounds and sound effects. |
| `@persist` | Keep progress in the browser. |
| `@extern` | A value that comes from the program showing the document, such as the score of a game. |
| `--- {hidden}` | Pages outside the normal flow, reached only by link or `go`. |
| `--- {on-enter="…"}` | Change state every time a page opens. |
| `--- {nav=off}`, `@mode game` | Pages you only leave through their options, for games and choices. |

## State with `@state`

Declare the variable and its initial value at the start of the document:

```ribalta
@state details = false
@state plan = pro
@state clicks = 0
```

The initial value can be true/false, a number or text. Unlike `@let`, the value of a `@state` changes while the document is shown: when you click an element, navigate or enter a page.

## Clicks

The `click` attribute takes one or more actions, separated by `;`:

```ribalta preview
@state details = false

# Why does the cost go down?
[Show details]{click="toggle $details" .accent}
::: if $details
::: box {.card enter=up}
Rendering happens in the browser, so there are no export servers.
:::
:::
```

| Action | Effect |
| --- | --- |
| `$x = value` | Changes the value. The type must match the `@state` ([R602](/en/reference/errors/)). |
| `toggle $x` | Flips true/false. |
| `add $list value` / `remove $list value` | Puts a text into a list or takes it out. See [State lists](#state-lists). |
| `clear $list` | Empties the list. |
| `go #id` | Goes to the page with that id. |
| `go back` | Returns to where you were before the last `go` or link. |
| `go next` / `go prev` | Moves one step forward or back, like the arrow keys. |

`click` works on any block (`::: box {click="…"}`) and on text spans (`[text]{click="…"}`). Clicking an interactive element does not advance the page. Only `@state` variables can change ([R605](/en/reference/errors/)).

## Content that depends on state

Use `::: if`, `::: else` and the `if` attribute exactly as in [Variables, data and repetition](/en/guides/data/#conditions-with--if). The difference is that, with `@state`, every alternative goes into the document and the choice happens live:

```ribalta preview
@state scenario = optimistic

## Revenue in 2027
[Optimistic]{click="$scenario = optimistic"} · [Conservative]{click="$scenario = conservative"}
::: if $scenario == optimistic
# $20 million {color=green}
:::
::: else
# $14 million {color=coral}
:::
```

An element hidden by `if` leaves the layout and the rest of the page reflows. An element with `step`, on the other hand, keeps its space until it appears.

## Variants: style based on state

`.style=condition` applies the `@style` only when the condition is true. It is the way to highlight the selected item:

```ribalta preview
@state plan = pro
@style chosen {border="4 accent" shadow=lg}
@style faded {opacity=0.45}

::: grid {cols=3 gap=lg}
::: box {.card .chosen="$plan == basic" .faded="$plan != basic" click="$plan = basic"}
### Basic
:::
::: box {.card .chosen="$plan == pro" .faded="$plan != pro" click="$plan = pro"}
### Pro
:::
::: box {.card .chosen="$plan == team" .faded="$plan != team" click="$plan = team"}
### Teams
:::
:::
```

Visual attributes such as `color`, `size` and `w` only accept fixed values. Using `@state` in them is an error ([R604](/en/reference/errors/)): use a variant. That way the compiler checks every color and size before the document is shown.

### On mouse hover

`.style=hover` applies the style while the mouse pointer is over the element. It is a good hint that something is clickable:

```ribalta preview
@style lift {border="3 accent" shadow=lg}

::: grid {cols=2 gap=lg}
::: box {.card .lift=hover click="go next"}
### Through the forest
:::
::: box {.card .lift=hover click="go next"}
### Back to the village
:::
:::
```

- It works on blocks, list items and text spans, and can be combined with other variants. Changes in color, background, border, shadow and opacity are animated.
- Touch screens have no hover, so the style does not show there: do not use `hover` to show important information.
- The condition is exactly `hover`; to combine it with state, use two variants, such as `{.lift=hover .chosen="$plan == pro"}`.

## Values in text

`$name` and `$( … )` show the current value:

```ribalta preview
@state yearly = false

## Pro plan
$ $($yearly ? 490 : 49) {size=4xl weight=black}
$($yearly ? "per year" : "per month") {.muted}
[Yearly billing: **$($yearly ? "on" : "off")**]{click="toggle $yearly"}
```

## State lists

A `@state` can also hold a list of texts: the endings someone has found, the items in a backpack, the questions already answered. The list never repeats an item and keeps the order items came in.

```ribalta preview
@state backpack = [map]

# The locked door
You carry: $backpack ($(len($backpack)) items)

The door opens. {if="key in $backpack"}

[Take the key]{click="add $backpack key"} · [Drop the map]{click="remove $backpack map"} · [Empty]{click="clear $backpack"}
```

| Feature | Effect |
| --- | --- |
| `@state x = []`, `@state x = [a, b]` | Declares the list, empty or with initial items. Items are texts. |
| `add $x value` | Puts the value at the end of the list, if it is not there yet. |
| `remove $x value` | Takes the value out of the list. |
| `clear $x` | Empties the list. |
| `value in $x` | True when the value is in the list. Use `not (value in $x)` for the opposite. |
| `len($x)` | How many items the list has. |
| `$x` in text | The items separated by commas. |

List actions work in `click` and in `on-enter`. With them, a game can count the endings found without one variable per ending:

```ribalta
@state endings = []

# Start

--- {#victory on-enter="add $endings victory"}
# You won
Endings found: $(len($endings)) of 3
```

- `in` also works with a fixed list: `$route in [sea, village]` is the same as `$route == sea or $route == village`.
- State lists are not compared with `==`; use `in` and `len` ([R602](/en/reference/errors/)). They do not change with `=` either: use `add`, `remove` and `clear`.
- Ribalta warns when `value in $x` looks for a text that no action adds, such as a typo ([R608](/en/reference/errors/)).
- `::: for item in $x` also walks state lists: the content is repeated while the document is shown and follows every `add` and `remove`. Inside it, actions do not use `$item` ([R620](/en/reference/errors/)).

## Random draws

`random(lowest, highest)` draws a whole number between the two values, both included. `pick(list)` draws an item from a list, fixed or from the state. Both only work inside actions (`click` and `on-enter`), and the result goes into a `@state`:

```ribalta preview
@state die = 0
@state hand = none

# Rock, paper, scissors
Die: $die · Hand: $hand

[Roll the die]{click="$die = random(1, 6)"} · [Play]{click="$hand = pick([rock, paper, scissors])"}
```

- Every call is a new draw: `$die = random(1, 6) + random(1, 6)` adds two dice.
- With a state list you can draw cards without repeats: `click="$card = pick($deck); remove $deck $card"`.
- Outside an action, such as `$(random(1, 6))` in text or in an `if`, the number would change every time the screen is drawn, so Ribalta refuses it ([R613](/en/reference/errors/)). Store the draw in a `@state` and show the variable.
- The draw seed is part of the state: a program that saves the state, such as in a link, repeats the same results from that point when it restores it.

## Time

`$elapsed` is the number of seconds since the page opened: 0, 1, 2… It goes back to zero every time you enter the page. With it you can show a countdown and reveal something after a while. `timeout` and `on-timeout` on the separator run actions when time runs out, like an automatic click:

```ribalta preview
@state score = 0

--- {timeout=30 on-timeout="go #time-up"}
# What is the capital of Brazil?
$(30 - $elapsed) seconds left

Hint: it starts with B {if="$elapsed >= 15"}

[Brasília]{click="$score = $score + 1; go #right"} · [Rio de Janeiro]{click="go #time-up"}

--- {#right hidden}
# Correct!

--- {#time-up hidden}
# Time is up
```

- `on-timeout` takes the same actions as `click`, `go` included. `--- {timeout=10 on-timeout="go next"}` moves on by itself, handy for a kiosk.
- The clock only runs while the document is being shown. In a static rendering, `$elapsed` stays at 0.
- Coming back to a page with `go back` or reloading the document starts the count again.

## Text input

`:: input {bind=$name}` shows a text field bound to a `@state`. With it, a story can call people by their name, a quiz can take written answers and a door can ask for a password:

```ribalta preview
@state password = ""

# The door asks for a password
:: input {bind=$password placeholder="Password" on-submit="go #inside"}

The door creaks… {if="$password == abracadabra"}

--- {#inside hidden}
# You are in
```

- `on-submit` runs actions when Enter is pressed, `go` included. Without `on-submit`, Enter in the field runs the page element with `key=enter`, like a form's submit button.
- The value changes on every key. To accept variations of an answer, use a list: `{if="$answer in [Brasília, brasília, brasilia]"}`.
- Typing in the field does not change the page, and the arrow keys move the cursor inside it.
- Details in [Blocks › input](/en/reference/blocks/#input).

## Keyboard

`key` gives a clickable element a shortcut key. Pressing the key runs the same actions as the `click`, like moving around a map in a game:

```ribalta preview
@state x = 0
@state y = 0

--- {nav=off}
# Position: $x, $y
[↑]{click="$y = $y - 1" key=up} [↓]{click="$y = $y + 1" key=down} [←]{click="$x = $x - 1" key=left} [→]{click="$x = $x + 1" key=right}

[Start over]{click="$x = 0; $y = 0" key=r}
```

- Accepted keys: a letter, a digit, `up`, `down`, `left`, `right` (the arrows), `Enter` and `Space`.
- A key used by an element stops navigating on that page: with `key=right`, the right arrow moves the character instead of moving forward. Combine it with `nav=off` when the whole page is a game.
- Only elements that are showing count: a false `if` or a step not yet revealed turn the key off.
- Every clickable element can also be reached with **Tab** and activated with **Enter** or **Space**, for people who do not use a mouse. A `.style=hover` variant also shows when the element gets keyboard focus.

## Sound

`:: sound` declares a sound on the page, and the `play` and `stop` actions control it. Sounds stop when you leave the page:

```ribalta
--- {on-enter="play rain"}
:: sound https://example.com/rain.mp3 {name=rain loop volume=40%}
:: sound https://example.com/door.mp3 {name=door}
# The door
[Open the door]{click="play door; go next"}
```

- `play` and `stop` work in `click`, `on-enter`, `on-timeout` and `on-submit`. `play` starts the sound from the beginning.
- `autoplay` plays the sound when the page opens, without an action.
- Sound only plays while the document is being shown, never in a static rendering. The program that shows the document may keep some screens silent, such as a second screen for whoever presents, so the sound does not play twice.
- Browsers only play sound after some click or key press; on the first page of a document just opened, `autoplay` may wait for the first interaction.
- Use addresses that start with `https://`.

## Saved progress

`@persist` lists the `@state` variables kept in the viewer's browser. After closing the document and coming back, progress carries on from where it stopped:

```ribalta
@state endings = []
@state coins = 0
@persist endings coins
```

- Only `@state` variables from the start of the document are accepted ([R617](/en/reference/errors/)). Other variables start from their initial value, as always.
- Progress lives in the browser, separately for each address where the document is shown, and each person has their own.
- A link that already carries the state in its address wins over the saved progress.
- To start from scratch, use the actions themselves: `click="clear $endings; $coins = 0"`.

## Score and completion in an LMS (SCORM)

Trainings and quizzes can run inside an LMS (Moodle, Canvas, TalentLMS…) and report the score and completion to it. Say which `@state` is the score and which pages complete the course:

```ribalta
@state points = 0
@scorm score=$points complete=#end pass=70

--- {#start}
# Which password is stronger?
[Avocado-Moon-Train-42]{click="$points = $points + 100; go #end"}
[123456]{click="go #end"}

--- {#end hidden}
# You scored $points points
```

The program that shows the document builds the SCORM package, a `.zip` you upload to the LMS.

| Parameter | What it does |
| --- | --- |
| `score=$variable` | Numeric `@state` reported as the score whenever it changes. |
| `complete=#page` | Page that completes the course when reached. Several: `complete=#won,#lost`. |
| `max=100` | Highest score (default `100`). The reported score stays between `0` and `max`. |
| `pass=70` | Passing score. With it, completion becomes "passed" or "failed"; without it, "completed". |

- The communication with the LMS uses SCORM 1.2, which practically every LMS accepts.
- The LMS keeps the session: a learner who leaves halfway comes back to the same page with the same state.
- Without `@scorm`, the package only reports completion when the learner reaches the last page.
- Outside an LMS, `@scorm` does nothing.

## Values from outside with `@extern`

Sometimes the document shows something it does not decide itself: the score of a game running on a server, the question of the round, today's exchange rate. `@extern` declares such a value. The program that shows the document, such as an app that embeds Ribalta, sets the value and can change it at any time, and the pages redraw right away.

```ribalta preview
@extern question = "What is the capital of Brazil?"
@extern round = 3
@extern revealed = false

Question $round {.kicker}
# $question
Brasília {if="$revealed" color=green}
```

- The value after `=` is the example: it sets the type (true/false, number, text or a list of text) and is what shows wherever nobody provides the value, such as in a static rendering.
- Use `$name`, `if`, variants and expressions as with `@state`.
- Only the program that shows the document changes an `@extern`: actions, `:: input`, `@persist` and the `score` of `@scorm` do not accept one ([R619](/en/reference/errors/)). For a value the document changes itself, use `@state`.
- Values from outside are not part of the saved state.

### Lists of records

An `@extern` can also be a list of records, such as the scoreboard of a game. The example is a `@data` table, which sets the fields and what shows without a host; `::: for` repeats the content for every row that arrives:

```ribalta preview
@data example
| name  | points |
| ---   | ---    |
| Ana   | 300    |
| Bruno | 200    |
@end
@extern scoreboard = $example

## Scoreboard
::: for player, i in $scoreboard
$i. **$player.name**: $player.points points
:::
```

- The example table needs at least one row: it gives the type of each field.
- The program that shows the document can send more or fewer rows; the `for` follows.
- Inside the `for`, use `$player.field` in text, `if` and variants. Actions do not use the item ([R620](/en/reference/errors/)), and visual attributes such as `size` still take fixed values only.
- `len($scoreboard)` gives the number of rows.

## Navigation

### Links and `go`

A link to `#id` goes to the page with that id. `click="go #id"` does the same and can be combined with other actions; they run in order, so the destination page already shows the new state:

```ribalta
--- {#pricing}
## Pick a plan
[Pro plan →]{click="$plan = pro; go #detail"}
[Teams plan →]{click="$plan = team; go #detail"}
```

### Detours with `hidden`

A page with `hidden` stays out of the normal flow: the arrow keys skip it and it is only reached by link or `go`. When you advance past the last step of a hidden page, you return to where you were.

```ribalta preview
@state plan = pro

# Pick a plan
[See Pro](#detail)
--- {#detail hidden}
::: if $plan == pro
## Pro: up to 25 users
:::
[← Back]{click="go back"}
```

A hidden page that no link or `go` reaches produces a warning ([R702](/en/reference/errors/)).

### Changing the next page

`--- {next=#id}` replaces the following page when you advance, to build alternative paths without duplicating the document.

| Kind | How | What happens |
| --- | --- | --- |
| Normal | arrows, space, click on the page | Next step or next visible page. |
| Jump | `[x](#id)`, `click="go #id"` | Goes to the page and remembers where it came from. |
| With state | `click="$x = v; go #id"` | Changes the state and then goes to the page. |
| Back | `click="go back"` | Returns to the position saved by the last jump. |
| Conditional | two buttons with `if`, or `next=#id` | The condition lives on the element, not on the action. |

### Back, reload and share

When the document is shown in a browser, the program that shows it can keep the position in the address, such as `#/pricing/2?plan=team` (page, step and states that differ from the initial values). In that case:

- the browser **Back** button undoes the last jump;
- **reloading** keeps the page, the step and the state;
- **copying the address** opens the document at the same point.

The arrow keys only record jumps in the browser history, not every step.

## State inside components

A component reads and changes the `@state` variables declared at the start of the document directly: `$points` in text, `if=$open` and `click="$points = $points + 1"` work inside the `@component` just like on any page.

For the same component to serve different variables, pass the `@state` as a parameter (`status=$q1`): inside it, `$status = done` changes the original `$q1`. `go` destinations accept parameters too, such as `go #question-$n`. That way a single component serves many items:

```ribalta preview
@state q1 = pending
@state q2 = pending
@component item {n status}
[Item $n: $status]{click="$status = done"}
@end

:: item {n=1 status=$q1}
:: item {n=2 status=$q2}
```

A page's `@state` (see below), a page's `@let` and `::: for` variables only exist where the component is used, so the component does not see them: pass them as parameters ([R410](/en/reference/errors/)).

## Page state

A `@state` at the top of a page, right after the `---`, only exists in that page. It goes back to its initial value whenever you enter the page, and `go back` restores the value it had:

```ribalta preview
# Start
--- {#demo}
@state open = false
## Demo
[Open]{click="toggle $open"}
Demo content {if=$open}
```

## Actions when entering a page

`on-enter` on the separator runs actions every time the page becomes the current one: when the document starts, when you move forward or back with the arrows, and through `go`, links or `next`. It does not run again when you return with `go back` or when the document is reloaded, because that is going back to a point where the page was already opened.

```ribalta preview
@state visits = 0
@state saw_the_end = false

# Start
This is the opening page.

--- {on-enter="$visits = $visits + 1"}
# Middle
You have been here $visits time(s).

--- {on-enter="$saw_the_end = true"}
# End
```

- The actions are the `click` actions that change state: `$x = value` and `toggle $x`, separated by `;`. Moving to another page (`go`, `go back`, `go next`, `go prev`) is not allowed ([R612](/en/reference/errors/)); to choose the following page, use `next=#id`.
- The page's own state goes back to its initial value before the actions run, so `on-enter` can adjust it too.
- `on-enter` works inside a `@style`, which lets several pages reuse the same action. The style's actions run before the ones written on the separator:

```ribalta
@state endings = 0
@style ending {on-enter="$endings = $endings + 1"}

--- {#victory .ending}
# You won
Endings found: $endings
```

## Locked pages and game mode

In a game or a choose-your-path story, the option you click decides the next page, not the arrow. With `nav=off` on the separator, the arrows, space, a click on the page and swiping do not leave it:

```ribalta
@state route = none

--- {nav=off}
# The crossroads
- A clue appears {step=1}
[Go to the sea]{click="$route = sea; go #sea"}
[Go to the village]{click="$route = village; go #village"}

--- {#sea hidden}
# The sea
[Back]{click="go back"}

--- {#village hidden}
# The village
[Back]{click="go back"}
```

- The page's steps are still revealed and hidden with the arrows.
- Actions written on the page still work, including `go next` and `go prev`.
- On a `hidden` page opened by `go`, the arrow does not take you back to where you were either. Offer a `[Back]{click="go back"}` for that.

`@mode game` at the start of the document (or in the base theme) does this on every page. It also tells the program that shows the document to hide the page counter (such as `3 / 12`) and anything else that would reveal the path, and the **End** key no longer jumps to the last page, so nobody sees the ending too early. A page can go back to normal with `nav=on`:

```ribalta
@mode game

# The Last Light
[Start]{click="go #chapter-1"}

--- {#chapter-1}
# Chapter 1

--- {nav=on}
# Credits
```

## Steps with `$step`

`$step` is the current step of the page. With it, a reveal can **replace** instead of just adding, such as fading out the previous items:

```ribalta preview
@let stages = [Import, "Adjust the theme", Share]
@style faded {opacity=0.35}

## Getting started
::: for t, i in $stages
1. $t {step=$i .faded="$step > $i"}
:::
```

The number of steps in a page considers `step`, `::: steps` and the numbers compared with `$step`. To set another value, use `--- {steps=4}`.

## Animations

| Attribute | Where | Values | Effect |
| --- | --- | --- | --- |
| `enter` | Any block or span | `fade`, `up`, `down`, `left`, `right`, `zoom`, `none` | How the element appears on a step or when an `if` becomes true. Steps default to `up`. |
| `transition` | `---` separator | `fade`, `slide`, `zoom`, `none` | Transition when entering the page. |
| `@transition` | Preamble | Same as `transition` | Default transition for every page. |

Since `enter` is a regular attribute, it can live in a `@style` (`@style pop {enter=zoom}`), keeping animation out of the page structure. Variant changes (color, background, opacity, shadow) are animated automatically.

## Static renderings

Wherever the document is drawn without anyone interacting, such as an image or a printout of the pages, it uses the initial state and shows every step.

## Common errors

| Code | When it happens |
| --- | --- |
| [R604](/en/reference/errors/) | `@state` used in a visual attribute or in a `@let`. |
| [R605](/en/reference/errors/) | `click` trying to change a `@let`, data or a parameter. |
| [R608](/en/reference/errors/) | Comparing with a value the state never gets, such as `$plan == prro`. |
| [R611](/en/reference/errors/) | A badly written `click` action. |
| [R612](/en/reference/errors/) | `on-enter` trying to change pages. |
| [R613](/en/reference/errors/) | `random` or `pick` outside an action. |
| [R614](/en/reference/errors/) | `on-timeout` without `timeout`. |
| [R116](/en/reference/errors/) | `:: input` without `bind`. |
| [R210](/en/reference/errors/) | `key` without `click`. |
| [R615](/en/reference/errors/) | `play` or `stop` with a sound that is not on the page. |
| [R617](/en/reference/errors/) | `@persist` with something that is not a `@state`. |
| [R618](/en/reference/errors/) | `@scorm` with an unknown parameter, a score that is not a numeric `@state` or an invalid number. |
| [R534](/en/reference/errors/) | `@mode` with a value other than `default` or `game`. |
| [R701](/en/reference/errors/) | `go #id`, a link or `next` pointing to a page that does not exist. |
| [R702](/en/reference/errors/) | A `hidden` page nobody reaches. |
