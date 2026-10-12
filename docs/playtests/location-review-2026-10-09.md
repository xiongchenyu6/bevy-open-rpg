# Location and difficulty review

This pass addresses the four shortcomings observed during ordinary native play:
repeated environment appearance, weak correspondence to story places, confusing
collision edges, and the late-chapter difficulty cliff.

## Required evidence

- All fifteen map kinds render named architecture matching the story and retain
  distinct topology; repeat finale visits change doors/crossings/approaches.
- Buildings reserve the ground actually occupied; bridges and the celestial
  arch's opening are traversable. Water contains no painted islands or trees.
- Collision texture IDs, all41 landmark footprints/approaches, shader runtime,
  library checks and compilation agree.
- Inspect all15 location stills, landscape/portrait screens, actual450-frame
  tour/video, full41-route audit and normal keyboard/pointer native review.
- Use identical novice policies/seeds to compare late-boss tuning. Report fixed
  loot separately from actual story rewards, not as recruited-human feedback.

## Difficulty evidence already produced

Late-boss attack caps at1.20 while HP and the first five chapter curves stay.
Skill practice is +2/+4/+8/+12 in chapters4–7. Fixed-loot Notice70 rose18→32/32;
HP-only still0/32 but reaches the final chapter18/32 instead of1/32. With actual
mixed story rewards, all three policies completed32/32. Their final HP means
were95/163/207, rounds25/23.5/17.9. Players responding to intent thus save health,
time and potions. Neither experiment changes decision policies or adds unlimited
resources; they are controlled28-encounter chains, not full overworld playtests.

CSV/logs currently in screenshots/result/27: imperfect-combat-balanced.csv,
imperfect-combat-with-story.csv, balance-sweep.log and balance-story-sweep.log.
Final visuals and full-route proofs will be recorded in screenshots/result/28.

## Rendered issues found and corrected during this pass

Initial landmark renders still had rocks/trees baked into ground and square
foreign textures below buildings. Replaced those with four pure surface textures
and a blocked Structure ground class. Actual15-map review then found bridge
water cleared under two bridges and a dry row splitting the pier pool. Bridge
cells now preserve water rendering while remaining walkable; banks stay blocked.
The pier retains its surrounding water instead of creating a dry margin.

Native opening review exposed a second issue hidden by full-reveal captures:
the inn's visible edge did not reveal the whole sprite when its centre remained
in fog. Architecture now checks its footprint for current/recalled visibility.
Regression covers edge revelation, front/rear depth, persistent memory and fully
unseen architecture; shadows keep their ground depth. The first version of that
regression incorrectly expected hidden while the player still stood in current
vision; the corrected case moves the player away before clearing memory.

A later native final-battle checkpoint exposed a real combat feedback error:
missing a strong attack reducedHP214→173, but party support replaced the enemy
attack report with healing/mana lines. Support now appends to the primary
result, and real turn/input tests require both damage and healing in the visible
two-line formatter. Guard/camp receipts report actual capped mana restoration.
Checkpoint construction now selects the current chapter's boss explicitly,
rather than leaving a newly created run's chapter-one boss behind.

Markers also avoid architecture overhangs, keeping local contacts and objectives
out of roofs. Tall-window camera clamping follows actual viewport dimensions
rather than assuming16:9, so the player remains visible in portrait captures.

## Native input and final occlusion check

The temporary wtype controller changed one-key keymaps between calls; Escape
could be interpreted as the previous movement key. A persistent standard-US
Wayland keyboard now uploads one verified keymap and sends actual evdev controls.
The tool is `tools/native_wayland_keyboard.sh`; this does not modify game state
or replace hardware game input. Real holds confirmed travel, the building's
collision and its visible绕行 hint (`native-wall-feedback.png`).

Walking behind the inn proved the roof depth but nearly hid the player entirely.
The occluding architecture now temporarily fades while retaining its depth and
collision; it restores opacity on exit. Regression covers this alongside fog
edge revelation and recalled visibility. Actual native screenshots confirm visible-through-roof and restored opacity on exit.


## Final evidence

Final `occlusion-build.log`: formatting, all-targets check,251 library tests
(6 deliberate ignored sweeps), desktop/capture/playtest builds passed. Separate
balance sweeps:192 continuous28-encounter journeys, zero timeouts. Actual mixed
story rewards give32/32 wins for each policy, while HP/potion/round differences
remain. Fixed-loot HP-only remains0/32 and is not misrepresented as fully balanced.

Final full-route audit:41/41 stages,7/7 seals,fox3/3,musician2/2,VictoryBoth.
All fifteen locations and720×1280/960×540 viewport images were inspected. The
actual tour contains450 raw frame files, encoded30fps as15.000 seconds;
first/middle/last states show story progression, rewards and subsequent movement.

Native screenshots confirm the visible inn, obstacle绕行 text, player visible
through the faded roof, opacity restored on exit, and enemy damage retained with
support. Named files: native-inn-visible.png, native-wall-feedback.png,
native-player-visible-through-roof.png, native-roof-opacity-restored.png,
native-report-result.png. Native tests used actual window input, not game-state
mutation; the late fight starts from a conservative developer checkpoint.

No recruited-human enjoyment test has been claimed. Upstream ICU CJK segmentation
warnings remain in logs; inspected glyphs render correctly. Checked final runtime
logs contain no panic, shader error or failed asset load.
