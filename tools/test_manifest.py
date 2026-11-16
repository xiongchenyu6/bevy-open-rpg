import json
import tempfile
import unittest
from pathlib import Path
from generate_web_manifest import generate, resource_group

class ManifestTest(unittest.TestCase):
    def test_actual_runtime_chapter_partition(self):
        self.assertEqual(resource_group('assets/fonts/NotoSansCJKsc-Regular.otf'), 'core')
        self.assertEqual(resource_group('assets/ui/title_bg.png'), 'core')
        self.assertEqual(resource_group('assets/actors/hero.png'), 'common')
        for chapter in range(1, 8):
            self.assertEqual(resource_group(f'assets/ui/anim/chapter{chapter}_sheet.png'), 'optional' if chapter == 1 else f'chapter{chapter}')
            self.assertEqual(resource_group(f'assets/ui/chapter{chapter}_art.png'), 'optional' if chapter == 1 else f'chapter{chapter}')

    def test_one_changed_file_retains_all_other_addresses(self):
        with tempfile.TemporaryDirectory() as temporary:
            root = Path(temporary)
            (root / 'index.html').write_text('launcher')
            (root / 'engine.wasm').write_bytes(b'engine')
            (root / 'art.png').write_bytes(b'art-v1')
            template = Path(__file__).parent / 'sw-template.js'
            first = generate(root, 'first', template)
            (root / 'art.png').write_bytes(b'art-v2')
            second = generate(root, 'second', template)
            a = {f['path']: f['url'] for f in first['files']}
            b = {f['path']: f['url'] for f in second['files']}
            self.assertEqual([p for p in a if a[p] != b[p]], ['art.png'])
            self.assertNotEqual(first['version'], second['version'])
            self.assertEqual(second['version'], generate(root, 'different-git-revision', template)['version'])
            for f in second['files']:
                self.assertEqual((root / f['url']).read_bytes(), (root / f['path']).read_bytes())

if __name__ == '__main__':
    unittest.main()
