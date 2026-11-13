# Combat feel

Battle results were shown the instant a command was confirmed: the damage
number, impact sprite and HP bar all changed before the hero's sword reached
the enemy, the enemy flinched across the whole action, and nothing held or
shook on a heavy blow. Feedback now waits for the visible contact.

- Rules are unchanged and still resolve on confirm. `battle/feel.rs` watches
  the HP that was actually lost and queues one contact cue per target. No HP
  loss means no cue: guards, heals, failed casts and skipped turns stay still.
- Contact moments: sword and burst at 30% of the action (0.216s / 0.276s),
  spells 0.38s after the cast, enemy blows at the peak of their lunge (50%).
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
(the game has neither). Skill choreographies share one 0.38s contact time
rather than a per-skill value.
