# Narrative continuity

The main run shuffled story scenes within each chapter, allowing the third-bell
farewell to precede the finale's companion scenes. `RunState::draw_story` now
selects the first unseen authored scene and preserves history after exhaustion.
Terrain and encounter randomness remain unchanged.

Chapter cards now explain why the party travels onward through the residual
black mist, black scales, diseased water, official seal, southern map, lock
array, and source water. The opening establishes Linger's planned departure;
the next stage confirms she has finished helping the villagers and joined.
Later chapter openings introduce Lin Yueheng and Nanyao at existing party joins.

Dialogue corrections include: no reference to a cave before visiting it; no
search for an already present companion; no implication Linger is waiting at
home during the finale; no liberation of the thunder qilin before breaking its
array; no confirmed attribution of an unsigned childhood wish; and no claim
that remembered villagers are providing real supernatural reinforcements.
Two choice receipts now accurately describe the actual health-only effect.

Southern companion scenes put the drum oath before the camp celebration, with
matching portrait and capture preset. Finale scenes retain their authored
sequence, ending with the third bell.

Verification: `cargo fmt`, `cargo check`, `cargo test --lib`, and
`scripts/story_harness.sh`. The chronology test covers all seven chapter pools
under 64 seeds and checks retained history and the finale's last scene.
Evidence: `screenshots/result/25/verification.log` and its dialogue captures.
This is a continuity pass; it does not establish that every optional event,
legacy exploration scene, or ending branch has received a full literary review.
