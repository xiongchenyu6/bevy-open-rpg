# Water-serpent motion review

## Scope

The opening water serpent previously used a single creature cutout, with
translation, breathing scale and hit tint supplying all movement. This pass
adds authored idle, bite and recoil poses for that enemy. Other enemies retain
their own identity sprites; combat rules and damage values are unchanged.

## Asset review

The original cutout contains transparent gaps inside its scales. Initial
Kontext edits inherited those gaps as white or black holes and barely changed
the pose. Those attempts are retained under
`assets/generated/comfy/raw/serpent-motion/` and were rejected.

A clean baseline generated on sg-office has a continuous scaled body and a
physically connected tail. Acceptance requires eight distinct poses per action,
consistent anatomy and framing, transparent surroundings, and visible internal
body/head movement. Different hashes alone do not establish animation quality.

## Verification

Timing tests cover non-looping attack and hit recovery, phase-midpoint contact,
idle during charge/gather/brace, and absence of hit motion for support or
zero-damage actions. Initial run: 256 library tests passed, 6 ignored.

`scripts/enemy_motion_harness.sh` runs formatting, all-target compilation,
library tests, build and a 450-frame real combat capture. Its
`rogue-enemy-motion` fixture increases enemy HP to keep the demonstration alive;
it operates through ordinary scripted Intent and records the actual rendered
atlas indices. It is a visual regression fixture, not a balance experiment or
a human playtest.

## Result

Final formatting, all-target compilation, build and library tests passed:
256 passed, 0 failed, 6 ignored. Asset checks confirm all24 distinct nonblank
alpha frames, fixed geometry and unclipped margins. Actual capture rendered
all8 idle, all8 bite and all8 hit poses, with two breaks and one guarded counter.
All450 raw frames and a 15-second video are in `screenshots/result/29/`.

Inspected the three original49-frame sequences, final dark-background alpha
contact, all24 rendered poses, and full idle/bite/hit screenshots. The new
creature retains continuous anatomy, the coil background is transparent, and
recoil is visible while the hero reaches the target. Bite column4 coincides
with existing movement/recoil; numerical damage and floaters still begin at
the original phase entry. Other enemies continue using static cutouts.

The exact asset checksum and verification summary are in `verification.json`;
`game-motion-contact.png`, `game-idle.png`, `game-bite.png`, `game-hit.png` and
`video.mp4` provide visual evidence. Existing vendor/private-interface warnings
and upstream ICU CJK segmentation warnings remain; no asset-loading error or
panic was found in this capture. This pass did not repeat balance sweeps or
the complete story-route audit because it changes rendering only.
