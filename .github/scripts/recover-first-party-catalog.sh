#!/usr/bin/env bash
set -euo pipefail

if [[ -z "${TERMCADE_TOKEN:-}" ]]; then
  echo 'TERMCADE_TOKEN is required to recover the first-party catalog' >&2
  exit 1
fi

# Immutable, reviewed release coordinates only. Publishing fetches and
# validates each package before the registry writes its catalog records.
while read -r tag asset; do
  go run github.com/aviorstudio/termcade@v0.0.5 publish \
    https://github.com/aviorstudio/termcade-games "$tag" "$asset"
done <<'RELEASES'
asteroid-v0.0.2 asteroid.tcade
tetris-v0.0.2 tetris.tcade
brickough-v0.0.2 brickough.tcade
RELEASES
