import importlib.util
import os
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

spec = importlib.util.spec_from_file_location('release_version', Path(__file__).with_name('release-version.py'))
module = importlib.util.module_from_spec(spec)
spec.loader.exec_module(module)


class ReleaseVersionTests(unittest.TestCase):
    def resolve(self, env, releases=None, draft=False):
        calls = []
        def api(path):
            calls.append(path)
            if path.startswith('releases?'):
                return releases or []
            return {'draft': draft, 'prerelease': False}
        with tempfile.TemporaryDirectory() as directory:
            output = Path(directory) / 'output'
            with patch.dict(os.environ, {**env, 'GITHUB_OUTPUT': str(output)}, clear=True), patch.object(module, 'api', api):
                module.main()
            return output.read_text(), calls

    def test_manual_and_tag(self):
        for env in [{'REQUESTED_VERSION': '1.7.0'}, {'RELEASE_BRANCH': 'v1.7.0'}]:
            output, calls = self.resolve(env)
            self.assertEqual(output, 'version=1.7.0\n')
            self.assertEqual(calls, ['releases/tags/v1.7.0'])

    def test_workflow_dispatch_resolves_commit_not_latest(self):
        releases = [
            {'target_commitish': sha, 'tag_name': tag, 'draft': False, 'prerelease': False}
            for sha, tag in [('newer', 'v1.8.0'), ('tested', 'v1.7.0')]
        ]
        output, _ = self.resolve({'RELEASE_BRANCH': 'main', 'RELEASE_SHA': 'tested'}, releases)
        self.assertEqual(output, 'version=1.7.0\n')

    def test_missing_or_ambiguous_release_fails(self):
        record = {'target_commitish': 'tested', 'tag_name': 'v1.7.0', 'draft': False, 'prerelease': False}
        for releases in [[], [record, record]]:
            with self.assertRaises(SystemExit):
                self.resolve({'RELEASE_BRANCH': 'main', 'RELEASE_SHA': 'tested'}, releases)

    def test_invalid_input_and_draft_fail(self):
        for version in ['1.7.0;echo bad', '../../main', '1.7.0-rc1']:
            with self.assertRaises(SystemExit):
                self.resolve({'REQUESTED_VERSION': version})
        with self.assertRaises(SystemExit):
            self.resolve({'REQUESTED_VERSION': '1.7.0'}, draft=True)


if __name__ == '__main__':
    unittest.main()
