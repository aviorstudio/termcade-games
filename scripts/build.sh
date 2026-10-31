#!/usr/bin/env bash
set -euo pipefail
cli=.artifacts/termcade-tool/termcade
if [ "${OS:-}" = Windows_NT ]; then cli="$cli.exe"; fi
if [ "$#" -gt 1 ]; then echo 'Usage: scripts/build.sh [asteroid|brickough|tetris]' >&2; exit 2; fi
games=(asteroid brickough tetris)
if [ "$#" -eq 1 ]; then
  case "$1" in asteroid|brickough|tetris) games=("$1") ;; *) echo 'Unknown game' >&2; exit 2 ;; esac
fi
for game in "${games[@]}"; do
  "$cli" dev build "$game"
done
python3 - "${games[@]}" <<'PY'
import hashlib,sys
from pathlib import Path
for game in sys.argv[1:]:
    path=Path(game)/'build'/f'{game}.tcade'
    print(hashlib.sha256(path.read_bytes()).hexdigest(),path)
PY
