#!/usr/bin/env python3
"""Validate shipped polish art: uv run --with pillow python tools/verify_polish_assets.py."""
from pathlib import Path
import hashlib
import re
from PIL import Image

root = Path(__file__).resolve().parents[1] / 'assets'
report = []
hero = Image.open(root / 'actors/hero-motion.png').convert('RGBA')
assert hero.size == (1024, 1536), hero.size
frame_digests = []
for index in range(24):
    col, row = index % 4, index // 4
    frame = hero.crop((col * 256, row * 256, (col + 1) * 256, (row + 1) * 256))
    alpha = frame.getchannel('A')
    hist = alpha.histogram()
    coverage = sum(hist[32:]) / (256 * 256)
    assert .08 < coverage < .78, (index, coverage)
    assert hist[0] > 5000, f'frame {index} has no transparent surround'
    frame_digests.append(hash(frame.tobytes()))
for start, action in [(0, 'idle'), (8, 'walk'), (16, 'attack')]:
    assert len(set(frame_digests[start:start + 8])) == 8, f'duplicate {action} frames'
    report.append(f'{action}: 8 distinct nonblank transparent frames')
enemy_contract = (root.parent / 'src/game/battle/enemy_animation.rs').read_text()
cell = int(re.search(r'const CELL_SIZE: u32 = (\d+);', enemy_contract).group(1))
serpent = Image.open(root / 'actors/water-serpent-motion.png').convert('RGBA')
assert serpent.size == (cell * 8, cell * 3), serpent.size
for row, action in enumerate(['idle', 'bite', 'hit']):
    digests = set()
    for col in range(8):
        frame = serpent.crop((col * cell, row * cell, (col + 1) * cell, (row + 1) * cell))
        alpha = frame.getchannel('A')
        hist = alpha.histogram()
        coverage = sum(hist[32:]) / (cell * cell)
        assert .06 < coverage < .85, (action, col, coverage)
        box = alpha.getbbox()
        assert box and box[0] > 0 and box[1] > 0 and box[2] < cell and box[3] < cell, \
            f'{action} frame {col} clips the cell boundary: {box}'
        digests.add(hashlib.sha256(frame.tobytes()).hexdigest())
    assert len(digests) == 8, f'duplicate serpent {action} frames'
    report.append(f'serpent {action}: 8 distinct nonblank alpha frames with unclipped margins')
for name in ['village', 'bamboo', 'cave', 'river', 'plague', 'capital', 'southern', 'dream']:
    image = Image.open(root / f'backgrounds/battle-{name}.png')
    assert image.size == (1024, 576), (name, image.size)
report.append('8 complete 16:9 painted battle environments')
for name in ['attack', 'guard', 'spell', 'combo', 'item', 'flee']:
    image = Image.open(root / f'ui/icons/{name}.png').convert('RGBA')
    assert image.size == (64, 64) and image.getchannel('A').getbbox(), name
for name in ['contact_shadow', 'spell-ring', 'spell-ring-wide', 'spell-shard']:
    image = Image.open(root / f'effects/{name}.png').convert('RGBA')
    alpha = image.getchannel('A')
    assert alpha.getextrema()[0] == 0 and alpha.getbbox(), name
report.append('6 command icons and 4 alpha-shaded effects')
assert (root / 'fonts/NotoSansCJKsc-Regular.otf').stat().st_size > 1000000
assert 'SIL OPEN FONT LICENSE' in (root / 'fonts/OFL.txt').read_text()
report.append('Noto CJK font and redistributable license present')
print('\n'.join(report))
