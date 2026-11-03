<!-- Generated from private documentation source. Do not edit directly. Source SHA256: d5b4ff5a692e83a686293c29488c141bc448ef923ac1a8009134475e84da7d0e -->

# termcade games

Asteroid, Tetris and Brickough are WebAssembly games for [termcade](https://github.com/aviorstudio/termcade).

```sh
termcade add aviorstudio/asteroid
termcade add aviorstudio/tetris
termcade add aviorstudio/brickough
```

Each game is implemented in Rust and exports Termcade’s WebAssembly ABI through `termcade-sdk`. The games keep their existing directories, `termcade.toml` manifests and independent `build/` outputs. Run an installed package through a caller-owned Termcade session.

The Cargo workspace locks the SDK to an exact Git revision. Rust 1.94 builds `wasm32-wasip1` reactors. Native tests cover gameplay rules and seeded replays; WebAssembly checks exercise the exported ABI at all four rendering densities.

Game releases retain independent `<game>-vX.Y.Z` tags. Published versions are immutable. The release workflow builds with a pinned Rust Termcade CLI, verifies hosted build provenance, publishes the package, then indexes it in the marketplace. `THIRD_PARTY_NOTICES.txt` accompanies release assets.

## Contributor verification

Tool versions and native tasks live in `mise.toml`. Prepare the pinned helper source and run the complete gate:

```sh
git clone https://github.com/nicodes/tools.git .ci-tools
git -C .ci-tools checkout --detach f412061a7a3f0914db9e09d1159e69504aee8706
mise install
mise run check
```

Native build and test commands retain source and output checks, with receipts in `.artifacts/verification`. Existing product release rules and artifacts remain owned by this repository. End users install the product’s published distribution.

## Contributor task interface

Contributor tool versions and task entry points live in the native Mise configuration. Use `mise run lint` for source checks, `mise run test` for the applicable behavioral tests, and `mise run check` for the complete supported gate. `mise run clean` removes declared generated outputs while preserving source, credentials and retained data. This package has no application development service.
