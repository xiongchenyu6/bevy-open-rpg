# Combat feel

Battle results were shown the instant a command was confirmed: the damage
number, impact sprite and HP bar all changed before the hero's sword reached
the enemy, the enemy flinched across the whole action, and nothing held or
shook on a heavy blow. Feedback now waits for the visible contact.

- Rules are unchanged and still resolve on confirm. `battle/feel.rs` watches
  the HP that was actually lost and queues one contact cue per target. No HP
  loss means no cue: guards, heals, failed casts and skipped turns stay still.
- Contact moments: sword and burst at 30% of the action (0.216s / 0.276s),
  the basic spell 0.38s after the cast (run skills: see below), enemy blows at the peak of their lunge (50%).
  Impact sprites, lights, damage numbers and the HP bars are hidden until then.
- Weight: Light (status ticks, resisted or tiny hits) flashes only. Normal
  sword hits stop 45ms; spells, bursts and enemy heavy/signature blows 70ms;
  counter, pursuit and stance-break hits 95ms. A kill adds 35ms. Stops never
  add up and are capped at 140ms.
- Hit-stop freezes the battle clock only: phase timer, actor motion, effects,
  spell choreography and sprite clips hold; the flash and shake keep playing.
- Camera shake is at most 9px, takes the strongest impact instead of summing,
  cannot be restarted within 90ms and is gone within 0.3s. UI does not shake.
- The struck actor flashes bright, recoils away from the attacker and squashes,
  then settles in 0.36s. The enemy coils back before lunging.
- Damage numbers pop in at contact with a drop shadow. Counter and pursuit
  hits (this game's criticals) are larger, gold and written `42!`.
- Effect fade-out no longer depends on frame rate.
- All transient state is cleared when a battle starts and ends.

Not included: audio (the project builds without `bevy_audio`), a reduced-motion
option (there is no settings screen to put it in) and pause/load handling
(the game has neither).

## Per-skill contact (round 2)

The twenty skill choreographies no longer share one 0.38s contact. Each style
names the moment it visibly lands (`skill_vfx::contact_secs`): chain lightning
0.28s, soul lantern 0.24s (the souls leave the target first), meteor and frost
lance 0.48s, starfall 1.02s and sword array 0.88s on their finale. The
choreography's own burst of light now starts on that moment instead of at the
cast, and the spell turn lasts until 0.4s after the landing. Checked on
captured frames of real casts (`capture ... rogue-contact`).

## Lighting

Captured frames showed saturated green, magenta and orange discs over the
battle backdrops and run maps: lights multiply the scene, the ambient level was
low, and each light used its full colour. `lighting.rs` now raises ambient
brightness (0.42 to 0.62), pulls light colour 60% toward white, and lowers
light intensity and core boost, so painted scenes keep their own palette.
