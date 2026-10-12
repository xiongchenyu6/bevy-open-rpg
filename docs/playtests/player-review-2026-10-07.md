# Player experience review

## Method and scope

Review the default walkable journey from a new-player perspective, through
actual keyboard/mouse input and rendered screens. A dedicated headless Sway
session hosts the ordinary desktop game without touching the user's desktop.
The reviewer is an AI agent, not a recruited human participant. Record observed
screens and actions separately from controlled imperfect-strategy simulations
and internal navigation regression. Do not call a perfect-policy sweep a human
playtest.

## Evidence-backed priorities

1. Route entry and exit demand long administrative dialogues on every map;
   refusal does not grant a meaningful alternative. Automatically track the
   main goal, retain short story introductions and archive progress on travel.
2. All required objectives use the same question mark. Show readable categories,
   distinguish required versus optional, and name the nearby interaction.
3. Walking onto an NPC interrupts travel; Space elsewhere opens paperwork.
   Use intentional nearby interaction and allow passage without activation.
4. Mechanism prose promises a puzzle but stepping onto it immediately wins.
   Require clue-based decisions with rotated choices, harmless retry and retreat.
5. Title/dialogue are keyboard-only despite clickable battle and reward screens.
   Provide real buttons, previews and consistent keyboard guidance.
6. Combat presents too many concepts at once and hides the principal action
   behind secondary log lines. Teach charge/heavy/counter in real combat,
   retain action plus consequence and disclose escape risk.
7. Fixed rested-boss comparisons do not capture inexperienced mistakes or
   attrition. Add real-input continuous imperfect strategies and report
   deaths, healing, guarding, input counts and damage contribution.

## Acceptance evidence

- Ordinary native window: title start, short opening, movement, deliberate
  interactions, branch choice, first charge/guard/counter, reward and continued
  exploration. Save screenshots and an action/observation diary.
- Repeat representative mouse and keyboard actions; hidden option buttons must
  not accept clicks or replace a keyboard choice merely from idle hover state.
- Wrong puzzle answer and retreat leave the route blocked without spending or
  reward; correct choice completes and pays exactly once through real systems.
- No manual main-task signing or hand-in confirmation; first map has visible
  goals, meaningful controls and no surprise fight before its planned lesson.
- Real escape failure/success, revival and death handoffs remain actionable.
- Full 41-stage regression, scaled landscape/portrait renders, library checks,
  controlled imperfect-strategy results and actual inspected frames.
- Record limitations: recruited-human feedback, perceived pacing and enjoyment
  cannot be established by bot win rates or accelerated time alone.

## Native observations (ordinary desktop build, October 7–8)

- Started with Space, read the chapter and three-line location introduction,
  walked south/east/north without consulting map resources or capture routing.
  The initial screen had no visible objectives; eastward movement stopped at a
  tree. After finding the gold Story marker, walking past did not interrupt;
  Space beside it opened the scene. Selected the first branch with Space and
  observed HP 80→88, potions 4→5 and the vow receipt.
- The forest terrain does not visually establish an inn courtyard despite its
  authored place name. Terrain blends also make some rocks/trees ambiguous as
  collision boundaries. These are remaining environmental-art shortcomings.
- Searching for the other objectives required several direction changes. A
  cyan gate was found before the first fight; this motivated a main-objective
  bearing hint and a first story within three walkable steps of spawn.
- Found the gold Fight marker by sight, pressed Space and entered the real
  first encounter. The enemy announced Charge; the lesson explained attack
  now, guard next. The first sword input dealt 18 damage (51→33), with actual
  forward attack pose and hit effect (`native-first-attack.png`).
- Immediate `wlrctl pointer click` did not prove mouse activation. Its temporary
  virtual device and same-frame press/release need separate diagnosis; this is
  not evidence that a physical mouse handler is broken.
- Completed the first fight through a guarded Heavy attack (HP 81→78), then
  used the visible counter opportunity. The sword dealt 21 and broke stance;
  the next sword defeated the enemy. Saved `native-counter-select.png`,
  `native-counter-hit.png`, and `native-after-first-fight.png`.
- Persistent virtual pointer fixed the test setup: a living device gives Sway
  pointer capability; hovering the middle reward really highlights it. Clicking
  selected 万剑归宗 and returned to the map at 2/3 objectives. The temporary
  `wlrctl` invocation was insufficient to establish input evidence. Use
  `tools/native_wayland_pointer.sh`, not one-shot device creation.
- Controlled combat baseline had 39 timeouts; revised shared boss-healing
  budget and chapter skill growth produced zero in the same 96 seeds/policies.
  HP-only still completed 0/32; 70%-Heavy response completed 18/32; 20%-counter
  fumbles completed 32/32. This experiment excludes exploration rewards and is
  evidence of stalemate removal and strategy differences, not novice balance.


## Verification results

The final native screenshots confirm clickable title/continue/choice controls,
a visible opening story and the new bearing hint (`native-final-title.png`,
`native-final-dialogue-mouse.png`, `native-final-story-mouse-receipt.png`,
`native-final-bearing.png`). The first native battle and reward were operated
normally; remaining chapters were covered by internal navigation regression.

Full regression passed all 41 stages, 7 seals, 7 primary stories, fox3/3 and
musician2/2, reaching VictoryBoth. It solved 15 mechanisms and won 78 encounters.
The 12,000-frame first attempt exhausted its budget at stage20; the retained
60,000-frame run ended at frame25,564. Automated frames:3,977 battle,16,798
dialogue,2,999 exploration; scripted input cadence dominates these figures,
so they cannot be interpreted as a human reading/combat ratio.

The wrong-answer screenshot initially exposed a missing visible error preview.
The preview now prioritizes errors; `puzzle-wrong-fixed/frame00000.png` shows
retry guidance alongside retained clues/options. Library regression includes
that display contract, intentional interactions and one-time puzzle settlement.

Remaining design problems are concrete: repeated forest/cave terrain, weak
visual correspondence to authored locations, ambiguous collision boundaries,
and later difficulty for players ignoring enemy intent. Human enjoyment and
normal-duration pacing still require human feedback; no human playtest has
been represented by the automated audit.


Final validation: `cargo fmt --check`, `cargo check`, 246 passing library
checks (5 deliberate ignored sweeps), and both desktop/capture builds passed.
The 96-journey sweep was run separately and passed. Portrait dialogue initially
shrunk to tiny text; local text/button compensation now renders readable choices
at720×1280 and360×800 without changing the960×540 layout. Actual images in
`final-*-rogue-story-opening` and `final-*-rogue-puzzle-wrong` were inspected.
Owned native game/compositor/pointer processes were shut down after verification.
Known upstream ICU CJK segmentation diagnostics remain; inspected glyph rendering
was correct and checked runtime logs contained no panic or failed asset loads.
