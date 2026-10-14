# @ribaltalang/player

Headless runtime for [Ribalta](https://viniciusadm.github.io/ribalta/) documents: page navigation and reveal steps, `@state` actions, timers, keyboard and swipe, sounds, saved progress, SCORM and values that the hosting page provides with `@extern`. It has no UI of its own; pair it with [`@ribaltalang/renderer`](https://www.npmjs.com/package/@ribaltalang/renderer).

```sh
npm install @ribaltalang/player
```

## Feeding a document from your app

A Ribalta source declares the values your page provides, with an example that sets the type and is shown while editing:

```ribalta
@extern score = 0
@data example
| name | points |
| --- | --- |
| Ana | 300 |
@end
@extern ranking = $example

# Score: $score
::: for player, i in $ranking
$i. $player.name: $player.points
:::
```

Your page sends real values, and `externState` keeps only declared names with the declared types:

```ts
import { externState } from "@ribaltalang/player/extern";

const { state, ignored } = externState(doc, {
  score: 42,
  ranking: [{ name: "Bruno", points: 500 }],
});
// <PageView doc={doc} page={doc.pages[0]} state={state} />
```

## Playing interactive documents

`@ribaltalang/player/navigation` has `startSession`, `advance`, `goTo`, `runActions` and `sessionHash`/`parseSessionHash` for documents with steps, clicks and branches; `clock`, `keys`, `sounds`, `persist` and `scorm` cover timers, shortcuts, audio, saved progress and LMS reporting.

## License

MIT or Apache-2.0, at your option.
