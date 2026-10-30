<!-- Generated from private documentation source. Do not edit directly. Source SHA256: 45eb0c74494592c725648b8af1b505c3c81df07bf06e04c3ac1a5503ce8a5b51 -->

# termcade games

Asteroid, Tetris and Brickough are WebAssembly games for [termcade](https://github.com/aviorstudio/termcade).

```sh
termcade add aviorstudio/asteroid
termcade add aviorstudio/tetris
termcade add aviorstudio/brickough
```

Each game has a `termcade.toml` manifest and builds into its own `build/` directory. Run it through a caller-owned termcade session.

## Contributor verification

Tool versions and native tasks live in `mise.toml`. Prepare the pinned helper source and run the complete gate:

```sh
git clone https://github.com/nicodes/tools.git .ci-tools
git -C .ci-tools checkout --detach 091101094fedf5ce2e67c339c1c9c842d6a662dc
mise install
mise run check
```

Native build and test commands retain source and output checks, with receipts in `.artifacts/verification`. Existing product release rules and artifacts remain owned by this repository. End users install the product’s published distribution.
