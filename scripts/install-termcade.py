"""Install the reviewed Rust authoring CLI after checking hosted provenance."""
import json
import os
from pathlib import Path
import platform
import subprocess
import tarfile
import tempfile
import zipfile

ROOT = Path(__file__).resolve().parents[1]
LOCK = json.loads((ROOT / '.github/termcade-cli.json').read_text())
SYSTEM = platform.system()
ARCH = {'AMD64':'x86_64','aarch64':'arm64','arm64':'arm64','x86_64':'x86_64'}.get(platform.machine())
assert (SYSTEM,ARCH) in {('Linux','x86_64'),('Linux','arm64'),('Darwin','x86_64'),('Darwin','arm64'),('Windows','x86_64')}, 'unsupported authoring host'
NAME = 'termcade.exe' if SYSTEM == 'Windows' else 'termcade'
DEST = ROOT / '.artifacts/termcade-tool' / NAME
TARGET = {('Linux','x86_64'):'x86_64-unknown-linux-musl',('Linux','arm64'):'aarch64-unknown-linux-musl',('Darwin','x86_64'):'x86_64-apple-darwin',('Darwin','arm64'):'aarch64-apple-darwin',('Windows','x86_64'):'x86_64-pc-windows-msvc'}[SYSTEM,ARCH]
def verify(path):
    info=json.loads(subprocess.check_output([str(path),'--build-info'],text=True))
    assert info == {'version':LOCK['tag'],'commit':LOCK['source_commit'],'target':TARGET}, 'unexpected authoring CLI identity'
if DEST.is_file():
    verify(DEST)
else:
    suffix='zip' if SYSTEM=='Windows' else 'tar.gz'
    asset=f'termcade_{SYSTEM}_{ARCH}.{suffix}'
    env=os.environ.copy()
    if not env.get('GH_TOKEN') and env.get('MISE_GITHUB_TOKEN'): env['GH_TOKEN']=env['MISE_GITHUB_TOKEN']
    DEST.parent.mkdir(parents=True,exist_ok=True)
    with tempfile.TemporaryDirectory(dir=DEST.parent) as scratch:
        archive=Path(scratch)/asset
        subprocess.run(['gh','release','download',LOCK['tag'],'--repo','aviorstudio/termcade','--pattern',asset,'--dir',scratch],check=True,env=env)
        subprocess.run(['gh','attestation','verify',str(archive),'--repo','aviorstudio/termcade','--signer-workflow','aviorstudio/termcade/.github/workflows/release.yml','--source-ref','refs/heads/main','--source-digest',LOCK['source_commit'],'--signer-digest',LOCK['source_commit'],'--deny-self-hosted-runners','--predicate-type','https://slsa.dev/provenance/v1'],check=True,env=env)
        if suffix=='zip':
            with zipfile.ZipFile(archive) as bundle:
                entries=bundle.infolist();assert len(entries)==1 and entries[0].filename==NAME and not entries[0].is_dir()
                raw=bundle.read(entries[0])
        else:
            with tarfile.open(archive,'r:gz') as bundle:
                entries=bundle.getmembers();assert len(entries)==1 and entries[0].name==NAME and entries[0].isfile()
                raw=bundle.extractfile(entries[0]).read()
        candidate=Path(scratch)/NAME;candidate.write_bytes(raw);candidate.chmod(0o755);verify(candidate);candidate.replace(DEST)
print(f'Authoring CLI {LOCK["tag"]} verified at {LOCK["source_commit"]}')
