#!/usr/bin/env bash
set -euo pipefail
export GOWORK=off
for dir in ./*/termcade.toml; do
  game="${dir#./}"
  game="${game%/termcade.toml}"
  go run github.com/aviorstudio/termcade@v0.0.5 dev build "$game"
done
sha256sum -- ./*/build/*.tcade
