import unittest

from package import macos_versions


class MacOSVersionTests(unittest.TestCase):
    def test_prerelease_bundle_identifies_stage_without_invalid_short_version(self):
        for channel, suffix in [('alpha', 'a'), ('beta', 'b'), ('rc', 'fc')]:
            version = f'2.0.0-{channel}.1'
            self.assertEqual(macos_versions(version), {
                'CFBundleShortVersionString': '2.0.0',
                'CFBundleVersion': f'2.0.0{suffix}1',
                'Course2mdVersion': version,
            })

    def test_stable_removes_prerelease_suffix(self):
        self.assertEqual(macos_versions('2.0.0'), {
            'CFBundleShortVersionString': '2.0.0', 'CFBundleVersion': '2.0.0',
            'Course2mdVersion': '2.0.0',
        })

    def test_invalid_apple_build_suffix_fails(self):
        for version in ['2.0.0-alpha.0', '2.0.0-alpha.256', '2.0.0-alpha.01', '2.0.0-preview.1']:
            with self.assertRaises(ValueError):
                macos_versions(version)


if __name__ == '__main__':
    unittest.main()
