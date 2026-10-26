<!-- Generated from private documentation source. Do not edit directly. Source SHA256: aa6b78a79f7c934c2c185647415443377630aa42afe55fddcc9a3fbce3eaf184 -->

# Termcade games

Asteroid, Tetris and Brickough are WebAssembly games for [Termcade](https://github.com/aviorstudio/termcade).

```sh
termcade add aviorstudio/asteroid
termcade add aviorstudio/tetris
termcade add aviorstudio/brickough
```

Each game has a `termcade.toml` manifest and builds into its own `build/` directory. Run it through a caller-owned Termcade session.

## Development

Install the [Komizo release](https://github.com/nicodes/komizo#install) matching `komizo.lock`.

```sh
komizo check        # Complete project checks
komizo run build    # Build the declared distributable artifacts
komizo describe     # Inspect tools and focused stages
```
