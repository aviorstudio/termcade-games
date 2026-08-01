# termcade-games

Every game aviorstudio publishes to the
[termcade](https://github.com/aviorstudio/termcade) marketplace.

The arcade ships with none of them. It compiles no games in and bundles no
starter pack — a fresh termcade is an empty cabinet and a marketplace, and
everything here is installed the same way a stranger's game is:

```sh
termcade add aviorstudio/asteroid
termcade add aviorstudio/tetris
termcade add aviorstudio/brickough@1.0.0    # or pin a version
```

That is the point of this repository existing separately. There is no
privileged path for first-party games: these go through the registry, on the
same terms, using the same commands, and if that path is broken they are as
broken as anybody else's.

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
termcade add asteroid/build/asteroid.tcade
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
