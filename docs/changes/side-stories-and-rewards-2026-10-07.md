# Side stories and narrative rewards

The default walkable run now treats optional stories as resolved choices with
receipts. Opening a scene does not establish a completed relationship, pay a
local commission, or earn its ending echo.

## Side-story arcs

| Arc | Setup and choice | Continuation | Payoff |
| --- | --- | --- | --- |
| 阿狸 · 山径旧缘 | First village encounter: spend one potion to rescue her, or pass by | At Suzhou or later, accept her thanks or repair an earlier refusal with one potion. Repair is guaranteed and changes the relationship | Southern/finale encounter offers 息壤袋 (10 HP at battle opening) or 虎啸符 (heavy attacks −5). Unrepaired relationships instead resolve a confrontation |
| 苏弦 · 雾散半阕 | River/capital encounter: fund her journey for 20 coins, or listen without paying | At the finale shelter, she remembers the actual first choice | Patrons receive 幽冥·噬魂灯 and +6 maximum MP; quiet listeners receive 皓月·清辉轮 and 25% MP recovery. Leaving quietly instead restores HP |
| Local guides | Named map contacts explain an errand's purpose and payment before acceptance | Asking directions leaves the guide available. Accepted errands track a concrete mechanism, story response, event decision, camp plan, or market visit | Reward arrives after resolution; each receipt also improves local travel and chapter-boss preparation. Replays never pay again |

Chain milestones are guaranteed on the next qualifying event node rather than
lost to repeated probability rolls. Random encounters use map-specific pools,
so capital notices and river crossings cannot appear in cave corridors. The
ending distinguishes victory, defeat, unfinished chains, and repaired fox bonds.

## Main-story reward choices

| Chapter primary scene | Protect / connect | Investigate / press onward |
| --- | --- | --- |
| Village | +8 maximum HP, one potion | +1 attack, +4 maximum MP, one potion |
| Moon cave | MoonHalo ward, +4 maximum MP | FrostLance control, +4 maximum MP |
| River | 25% HP recovery, 30 coins | TidalDragon finishing spell |
| Plague | Save child: lose 15% HP, gain two potions and MoonHalo | Recover 30% HP, learn ThornCage control |
| Capital | YinYangMirror reduces opening enemy attack | SwordSutra strengthens offensive spells |
| Southern road | MountainAegis ward, +4 maximum MP | StormChain pressure spell, +4 maximum MP |
| Abyss | Recover 30% HP and MP | +2 attack, recover 30% MP |

Additional scenes retain modest resources/stat boons. Old musician 许伯's paid
farewell now exchanges 30 coins for 引魂灯, rather than charging for a response
with no practical reward. Each scene has a unique title in the story journal.

## Reward contract

- Coins, potions, and blood offerings are checked and spent before resolving
  chance. Insufficient resources leave the choice open without RNG or progress.
- A failed paid attempt still spends the advertised fee; previews disclose its
  probability and loss. Every option has an affordable or free alternative.
- Preview data and actual effects share the same authored reward values. Actual
  receipts report capped HP/MP restoration instead of theoretical gains.
- Already-known gifted spells grant +2 maximum MP and restore 25% MP; owned specified relics convert to
  40 coins. Resolving the same scene again never charges or pays again.
- The inventory's 缘事簿 archives choices and actual receipts. Permanent camp
  boons and relationship scores are awarded once per camp.

## Verification

Run `nix develop --command bash scripts/narrative_harness.sh <out>`. It checks
formatting, compilation, all library regressions, every authored choice's
reward receipt, real-system reward-build combat sweeps, actual keyboard-driven
choice/rejection/receipt/journal captures, a 960×540 view, and the full 41-stage
run. The run audit also requires both side arcs and all seven chapter-primary
scenes to be resolved. Combat comparisons use fixed loot and rested (main story)
or partially rested (side story) encounters; they are controlled comparisons,
not a human playtest or proof of every possible build's balance.

## Integration findings

The journey audit exposed two regressions that isolated reward tests could not
cover. Guides must remain interactable without staying navigation targets;
visited guides now retain their visuals but the flow proceeds to other tasks.
An empty learned-skill list also exposed a pre-existing price mismatch: the
fallback-spell view omitted MoonBite's surcharge. Real casting and its UI now
share `fallback_spell_cost`; a real input test rejects insufficient mana and
casts correctly after funding. `rogue-cost-proof` captures the corrected
7-MP price with only 6 MP available and verifies the failed cast spends nothing.

Modern learned spells now apply their actual spell relic and camp modifiers.
SwordSutra/StarSand add damage once per cast, including multi-hit skills;
KunlunMirror scales eligible elite/boss damage. Offensive bonuses do not
strengthen wards or healing. SpiritPendant reduces learned-spell mana prices.
Battle inputs, spell pages, inventory, and capture control use the same
`skill_cost`; reward previews explicitly label their unmodified base price.

## Verified result

`screenshots/result/26/validation.json`: 226 passing library tests, 102 choice
contracts, 672 main-story and 256 side-story real-system combat comparisons
with no timeouts. The fixed-seed real journey reached VictoryBoth through all
41 stages and seven seals, with fox 3/3, musician 2/2 and 26 guide receipts.
Actual captures include 1530 frames, 15-second side-story and 8-second skill
clips, 960×540 decision layout, corrected 7-MP quotation, and the real ending.
The latest native Wayland launch ran for 12 seconds without a panic or missing
asset error. Local development optimization overrides are documented in
`verification-mode.txt`; comparisons and accelerated journeys are not human
playtests. Upstream Parley/ICU CJK word-segmentation diagnostics remain in logs;
inspected text renders and wraps correctly.
