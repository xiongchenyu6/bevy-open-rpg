# Visual polish, round 2

What a player sees differently, and how it was checked. Frames come from the
offscreen capture binary; before/after sets are under `screenshots/round2/`
(not tracked).

## Run map

- Fog of war no longer switches square by square. The clear area around the
  hero fades out over about two cells, remembered and unseen ground blend into
  each other, and the clear area slides after the hero.
- Path, grass, water and cliff meet along an irregular, slightly blurred line
  instead of the square grid. Movement and collision are unchanged.
- The miasma damage text is larger, pale and edged; it was small purple text
  on a dim map.
- The vitals panel shows icons with numbers instead of four text labels.
  Resting the pointer on (or pressing) an icon names it.
- The corners of the view are darkened on the run map and in battle.

## Battle

- Each skill's damage, flash and hit-stop land when that skill's effect
  reaches its target (see `combat-feel-2026-11-13.md`, round 2 section).
- A skill's burst of light no longer washes the target in saturated colour
  before the hit, and the soul lantern no longer hides the enemy.

## How the art was made

All new art is procedural (`tools/generate_ui_art.py`: three icons and the
vignette; fog and terrain borders are computed at run time). No image
generation backend was used.

## Not done

- No bloom or tone mapping: both need render features this project keeps
  switched off, so the vignette is the only screen-wide effect.
- Tooltips exist only on the run map vitals. Battle commands keep their
  description panel; markers, relics and skills have no tooltip yet.
- In portrait layout the new stat icons stay 20px while the text grows.
- At 960x540 the last word of the controls hint wraps to a second line.
- The legacy explore mode still uses square fog tiles.
