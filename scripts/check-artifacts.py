#!/usr/bin/env python3
"""Bind each built game archive to its source manifest and compiled WASM bytes."""
import hashlib
import os
from pathlib import Path
import sys
import tomllib
import zipfile


GAMES = {'asteroid', 'brickough', 'tetris'}


def verify(archive, source, module):
    for path in [archive, source, module]:
        if path.is_symlink() or not path.is_file():
            raise ValueError('artifact inputs must be regular files')
    if archive.stat().st_size > 66 * 1024 * 1024:
        raise ValueError('game package exceeds its byte limit')
    with zipfile.ZipFile(archive) as bundle:
        if sorted(bundle.namelist()) != ['game.wasm', 'termcade.toml']:
            raise ValueError('game archive must contain exactly its manifest and module')
        if (bundle.getinfo('game.wasm').file_size > 64 * 1024 * 1024
                or bundle.getinfo('termcade.toml').file_size > 1024 * 1024):
            raise ValueError('game archive member exceeds its byte limit')
        manifest = tomllib.loads(bundle.read('termcade.toml').decode())
        if manifest != tomllib.loads(source.read_text()):
            raise ValueError('packaged game identity/configuration differs from source')
        wasm = bundle.read('game.wasm')
        if not wasm.startswith(b'\0asm\x01\0\0\0') or wasm != module.read_bytes():
            raise ValueError('packaged WASM differs from the compiled module')
    return hashlib.sha256(archive.read_bytes()).hexdigest()


def main():
    games = sys.argv[1:] or sorted(GAMES)
    if not set(games) <= GAMES:
        raise ValueError('unknown game')
    root = Path(__file__).resolve().parents[1]
    target = Path(os.environ.get('CARGO_TARGET_DIR', str(root/'.artifacts/rust-target')))
    for game in games:
        archive = root/'crates'/game/'build'/f'{game}.tcade'
        module = target/'wasm32-wasip1/release'/f'termcade_{game}.wasm'
        digest = verify(archive, root/'crates'/game/'termcade.toml', module)
        print(game, digest, 'manifest and compiled module verified')


if __name__ == '__main__':
    main()
