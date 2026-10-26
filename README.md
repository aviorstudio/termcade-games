<!-- Generated from private documentation source. Do not edit directly. Source SHA256: 796b6b47dd25a56f905e53f57b59c83a7e46dcdcfb5af0d934664ba69697b304 -->

# termcade games

Asteroid, Tetris and Brickough are WebAssembly games for [termcade](https://github.com/aviorstudio/termcade).

```sh
termcade add aviorstudio/asteroid
termcade add aviorstudio/tetris
termcade add aviorstudio/brickough
```

Each game has a `termcade.toml` manifest and builds into its own `build/` directory. Run it through a caller-owned termcade session.

## Development

Install the [komizo release](https://github.com/nicodes/komizo#install) matching `komizo.lock`.

```sh
komizo check        # Complete project checks
komizo run build    # Build the declared distributable artifacts
komizo describe     # Inspect tools and focused stages
```
