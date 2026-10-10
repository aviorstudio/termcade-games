"""Offline checks for inherited Clerk credentials and authenticated retry."""
import io
import json
import os
import textwrap
import unittest
import urllib.error
import urllib.request
from contextlib import redirect_stdout
from pathlib import Path
from unittest.mock import patch

WORKFLOW = Path('.github/workflows/recover-catalog.yml').read_text()
SCRIPT = textwrap.dedent(WORKFLOW.split("          python3 - <<'PY'\n", 1)[1].rsplit('          PY', 1)[0])
KEY = 'ak_' + 'synthetic' * 8

class RecoveryTests(unittest.TestCase):
    def run_script(self, token, post_status=409):
        calls = []
        fixtures = {}
        def fetch(request, timeout):
            calls.append(request)
            if request.method == 'POST':
                self.assertEqual(request.get_header('Authorization'), 'Bearer ' + KEY)
                body = json.loads(request.data)
                self.assertEqual(body['expected_sha256'], fixtures[body['asset']]['sha256'])
                if post_status != 201:
                    raise urllib.error.HTTPError(request.full_url, post_status, 'synthetic', {}, io.BytesIO())
                return io.BytesIO(b'{}')
            game = request.full_url.split('/games/')[1].split('/resolve')[0]
            entry = fixtures[game.split('/')[1] + '.tcade']
            return io.BytesIO(json.dumps(entry).encode())
        class Response:
            def __init__(self, stream): self.stream = stream; self.status = 200
            def __enter__(self): return self
            def __exit__(self, *args): return False
            def read(self, *args): return self.stream.read(*args)
        namespace = {}
        # Build reviewed fixtures from the workflow without executing network calls.
        exec(SCRIPT.split('token = os.environ', 1)[0], namespace)
        for game, tag, asset, digest in namespace['REQUIRED']:
            fixtures[asset] = {'id':game,'version':tag.rsplit('-v',1)[1],'repo':namespace['REPO'],'tag':tag,'asset':asset,'sha256':digest,'abi':1}
        output = io.StringIO()
        with patch.dict(os.environ, {'TERMCADE_API_KEY':token}), patch.object(urllib.request,'urlopen',side_effect=lambda req,timeout: Response(fetch(req,timeout))), redirect_stdout(output):
            exec(compile(SCRIPT, '<recovery-workflow>', 'exec'), {})
        self.assertNotIn(token, output.getvalue())
        return calls

    def test_existing_catalog_still_authenticates_each_publish(self):
        calls = self.run_script(KEY)
        self.assertEqual(sum(r.method == 'POST' for r in calls), 3)
        self.assertEqual(sum(r.method == 'GET' for r in calls), 6)
        self.assertIn('TERMCADE_API_KEY: ${{ secrets.TERMCADE_API_KEY }}', WORKFLOW)

    def test_invalid_key_fails_before_network(self):
        with patch.object(urllib.request,'urlopen') as network, patch.dict(os.environ,{'TERMCADE_API_KEY':'tck_removed'}):
            with self.assertRaisesRegex(SystemExit, 'Clerk publishing key'):
                exec(compile(SCRIPT, '<recovery-workflow>', 'exec'), {})
            network.assert_not_called()

    def test_revoked_key_fails_recovery(self):
        with self.assertRaisesRegex(SystemExit, 'HTTP 401'):
            self.run_script(KEY, post_status=401)

if __name__ == '__main__': unittest.main()
