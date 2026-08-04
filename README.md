# termcade-games

Every game aviorstudio publishes to the
[termcade](https://github.com/aviorstudio/termcade) marketplace.

The arcade compiles no games in: these are `.tcade` packages in the wasm
sandbox, same as anybody's. All three are also vendored into the arcade as a
starter pack and unpacked on first run, so a fresh termcade is playable
before it has an account or a network. They install the ordinary way too:

```sh
termcade add aviorstudio/asteroid
termcade add aviorstudio/tetris
termcade add aviorstudio/brickough
```

The source stays here rather than in the arcade so the two move separately —
a game changes without a release of termcade, and the SDK contract is the
only thing between them.

The bundling is a real cost, worth naming: a game that arrives prebundled
does not exercise resolve/fetch/verify, so the registry path can rot without
a first-party game noticing. It has to be covered on its own rather than by
these three happening to use it.

To refresh the vendored packs after changing a game, from a termcade checkout
beside this one: `go generate ./internal/starter`.

## Releasing

`release.yml` builds a game, tags it, cuts a GitHub release, and tells the
marketplace about it — in that order, because publishing is a claim the
registry verifies by fetching, so the asset has to exist first.

That last step needs a `TERMCADE_TOKEN` secret: a publish key scoped to the
`aviorstudio` handle, made with `termcade keys new`. It publishes and nothing
else — it cannot read a library, mint another key, or touch an account — so a
leak is bounded by this one handle.

Without the secret the release still cuts and the workflow warns, with the
command to publish by hand. A release that failed after tagging is much harder
to unpick than one the registry has not heard about yet.

## The games

| Game | What it is |
| --- | --- |
| **asteroid** | Asteroids-style rock shooter. Inertial ship on a wrapping field; big rocks split into faster small ones. |
| **tetris** | Falling-block stacker. Super Rotation System kicks, a 7-bag randomizer, ghost piece, lock delay, and gravity that tightens by level. |
| **brickough** | Breakout-style brick breaker. Steer the ball with paddle english, clear the wall, survive the speed-up. |

## Layout

One Go module, one directory per game:

```
asteroid/
├── termcade.toml     manifest: id, version, playfield, controls
├── game.go           the sdk.Game implementation
├── *_test.go
└── cmd/wasm/main.go  the wasip1 entrypoint, ~8 lines
```

Games depend on `github.com/aviorstudio/termcade/sdk` and nothing else. One
module for all of them is deliberate: they then share one sdk version, so an
SDK contract change breaks every game here at once and in CI, rather than
drifting game by game until someone tries to rebuild an old one.

## Working on a game

Requires [mise](https://mise.jdx.dev) for the pinned Go (`mise install`).

```sh
go test ./...                       # every game
termcade dev build asteroid         # → asteroid/build/asteroid.tcade
termcade dev install asteroid/build/asteroid.tcade
termcade                            # play it
```

`go test` only proves a game compiles for the host. A game reaches players as
a wasip1 reactor module exporting the termcade ABI, and `dev build` is what
checks that — so CI packages every game on every push, not just tests it.

## Releasing

Bump `version` in the game's `termcade.toml`, merge, then run the **Release a
game** workflow and pick the game.

The manifest decides the version. The workflow reads it, refuses to run if
that version is already tagged, and creates `<game>-v<version>` with the
`.tcade` and its sha256 attached. Nothing else names a version — the registry
reads the same field out of the package it fetches, so a tag derived from
anywhere else could disagree with what the marketplace records.

Tags are `asteroid-v0.0.1`, not `asteroid/v0.0.1`: a slash would make Go read
the tag as a module in a subdirectory and invent a version of a package nobody
imports.

Then point the marketplace at the release:

```sh
termcade publish https://github.com/aviorstudio/termcade-games asteroid-v0.0.1 asteroid.tcade
```

The registry fetches that asset once, validates it against the same manifest
rules the arcade enforces, reads the game's id and version out of it, and
records its sha256. Players download from GitHub and verify against that
digest, so a release asset swapped afterwards fails rather than reaching
anyone.

## Writing your own

Nothing here is special. `termcade dev new you/mygame` scaffolds a game that
already runs, and [the SDK docs](https://github.com/aviorstudio/termcade/blob/main/docs/sdk.md)
are the whole contract — 60 ticks a second, eight keys, a canvas in square
logical units, and a
[frozen wasm ABI](https://github.com/aviorstudio/termcade/blob/main/docs/abi.md)
if you would rather not write Go.
