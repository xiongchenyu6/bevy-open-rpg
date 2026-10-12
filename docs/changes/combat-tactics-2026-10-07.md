# Tactical combat and presentation

The old shared random-intent pool encouraged repeating attack/spell/burst. Boss
signatures could override the displayed warning, and nonattacking enemy turns
still played recoil/strike animations. Combat now has readable enemy rhythms,
action-specific opportunities and honest telegraphs.

- Assault, armored, trickster and spirit enemies have distinct short patterns.
- Charge is an actual nonattacking beat; its next heavy/signature is advertised.
  Control/break cancels it. Boss signatures execute only their stored intent.
- Swords apply 2 pressure, damaging spells 1, bind/flurry/stun spells 2, bursts 3.
  Brace halves ordinary sword damage and blocks sword pressure; magic penetrates.
- Filling pressure cancels one enemy action and opens one +50% followup. That
  followup produces no pressure, preventing permanent break chains.
- Guarding a real heavy/signature earns momentum and one +50% counter with +3
  pressure. Cancelled moves earn no counter; even a fully shielded real hit does.
  Waiting/healing/warding gives up banked offensive opportunities.
- UI shows pressure, incoming move and contextual counters. Sword attacks move
  into contact, recover and return; guard has a transparent protective aura.
  Nonattacking turns no longer fake hits, and action timings preserve slash poses.
- Boss HP/attack discounts grow by chapter instead of remaining flat while the
  party gains chapter-seal stats. Enemy construction is shared by runtime and sim.
- Support spells no longer cause mirror reflection; signature defenses include
  relic mitigation, shield feedback is retained, and lethal DOT settles victory.

## Verification

204 library tests passed, including real input/turn integration tests for break,
guard, support spells, affordability, telegraphs and deterministic combat.
The ignored policy sweep ran 1,536 real-system fights: nine normals and seven
bosses ×32 seeds ×3 policies, with production chapter scaling and a fixed
four-spell kit. No timeouts. It compares policies, not human/full-run difficulty.

Examples: armored stone guardian averaged 7.0 rounds by attack-only versus 4.0
by spell/burst. Against the final boss, warning-aware play retained 120.2 HP on
average versus 71.3 for attack-only; spell/burst-first lost all32 fixed-kit cases.
This does not claim every spell build is weak or every human should follow the bot.

Proof: `screenshots/result/24/` contains 450 actual GamePlugin frames and a
15-second/30-fps video, real break/counter trace, scaled UI bounds reports and
sweep CSV/summary. The fixed-seed full-run audit passed all41 stages, archived
all41 main tasks, earned all7 seals and won97 fights. It is an automated
regression, not a human89-minute or ten-hour playtest.

Reproduce inside nix develop: `bash scripts/combat_harness.sh <out>`.
Upstream Parley/ICU's CJK word-boundary diagnostic remains; inspected wrapping
renders correctly and engine/version/trimmed features are unchanged.
