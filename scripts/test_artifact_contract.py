import importlib.util
from pathlib import Path
import tempfile
import unittest
import zipfile


spec=importlib.util.spec_from_file_location('artifact_contract',Path(__file__).with_name('check-artifacts.py'))
contract=importlib.util.module_from_spec(spec);spec.loader.exec_module(contract)


class GameArtifactContract(unittest.TestCase):
    def setUp(self):
        self.temp=tempfile.TemporaryDirectory();self.addCleanup(self.temp.cleanup)
        self.root=Path(self.temp.name);self.archive=self.root/'game.tcade'
        self.source=self.root/'termcade.toml';self.source.write_text('[game]\nid="example/game"\nversion="1.0.0"\n')
        self.module=self.root/'game.wasm';self.module.write_bytes(b'\0asm\x01\0\0\0')

    def package(self, manifest=None, wasm=None, extra=False):
        with zipfile.ZipFile(self.archive,'w') as archive:
            archive.writestr('termcade.toml',manifest if manifest is not None else self.source.read_bytes())
            archive.writestr('game.wasm',wasm if wasm is not None else self.module.read_bytes())
            if extra:archive.writestr('../unexpected','unsafe')

    def test_accepts_exact_source_and_compiled_bytes(self):
        self.package();self.assertEqual(len(contract.verify(self.archive,self.source,self.module)),64)

    def test_changed_identity_module_and_extra_members_refuse(self):
        for args in [{'manifest':b'[game]\nid="other/game"'},{'wasm':b'not wasm'},{'extra':True}]:
            with self.subTest(args=args):
                self.package(**args)
                with self.assertRaises(ValueError):contract.verify(self.archive,self.source,self.module)

    def test_linked_inputs_refuse(self):
        self.package();linked=self.root/'linked.tcade';linked.symlink_to(self.archive)
        with self.assertRaises(ValueError):contract.verify(linked,self.source,self.module)


if __name__=='__main__':unittest.main()
