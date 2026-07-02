# 10-Hour RPG Scope

This project targets the feel and pacing of a classic Chinese xianxia romance
RPG: village opening, fateful meeting, road journey, puzzle dungeons, companion
bonds, turn-based encounters, and tragic-mystic stakes. It should not copy
original maps, dialogue, scripts, or encounter tables from any existing game.

## Target Runtime

| Segment | Runtime | Content |
|---------|---------|---------|
| Act 1: Village Oath | 60-90 min | home village, first companion, tutorial quest, first dungeon gate |
| Act 2: Island / Cave Trial | 90 min | spiritual trial, puzzle props, first boss, relationship scene |
| Act 3: River Town | 90 min | town hub, side quests, second companion, duel/boss |
| Act 4: Plague Village | 90 min | investigation, cure quest, dungeon loop |
| Act 5: Capital Intrigue | 120 min | mansion/social hub, infiltration, layered boss |
| Act 6: Southern Spirit Road | 120 min | travel maps, tribal hub, late-game powers |
| Finale | 90 min | final dungeon, companion resolution, ending |

The current implementation covers the beginning of Act 1, the entry into an
Act 2-style cave trial with a Water Moon Cave hub plus a separate moon-echo
corridor route, a first chapter boss handoff, the first playable slice of Act 3
in a river town plus its ordered reed-bed river-lantern puzzle, the first
playable slice of Act 4 in a plague village plus a separate plague-shrine route
map, the first playable slice of Act 5 in the capital hub plus an infiltrated
mansion map and a separate mirror-gallery route, the first playable slice of Act
6 on the Southern Spirit Road plus a separate thunder-drum route, and a first
playable Finale slice with a Spirit Abyss Gate hub plus a separate old-dream
waterway route. The code should be expanded in chapters rather than as isolated
scenes.

The finale now has an explicit post-boss epilogue handoff instead of ending on
only a boss-complete flag. After Dream Eclipse falls, the gatekeeper resolves
the ending, grants a homecoming seal, and reflects bond, camp, and side-quest
completion back to the player so the route has a readable close.

Optional task-board side quests now exist as a repeatable content framework:
each chapter hub with a quest board lists two local errands, prioritizes them by
state (`ready to turn in`, `in progress`, `available`, then `completed`), tracks
battle progress, shows board markers, turns in completed work, and grants
exp/potion/money rewards. These side quests are meant to provide pacing filler
between main story beats, not replace authored chapter scenes. The exploration
HUD now also shows a task book with the main objective, the current map's full
local quest-board list, and accepted side-quest progress. Interacting with a
quest board opens a board overview, then a selectable commission menu, then a
structured commission page with status, objective, progress, and reward before
showing the accept/progress/turn-in result. Accepting a commission writes it
into the task book and echoes the execution area, tracking hint, turn-in
location, and reward so pickup has a clear RPG handoff. A persistent tracking
card now stays on the exploration screen: it always shows the current main
objective, and it appends the active commission with progress, location, and
turn-in target after a side quest is accepted. Available and ready-to-turn-in
commissions require an explicit two-option confirmation (`领取并追踪` / `交付领奖`
vs. `先不处理`) so task pickup and reward claiming feel deliberate. The board
marker only changes to completed after both local errands are done.

Main-story NPC handoffs now use the same deliberate pickup shape for claim,
advance, turn-in, and boss handoff moments: the player sees a structured main
task card and confirms (`领取并追踪`, `继续主线`, `交付主线`, or `迎战首领`) before
the quest log changes or a boss battle starts. Confirmed main-task changes raise
a visible task-book update notice and refresh the persistent tracking card.

Quest direction should be visible in-world, not only in HUD text: active NPCs,
task boards, and puzzle props use floating badge markers for available (`!`),
in-progress (`*`), and completed (`✓`) states. Inactive dialogue targets hide
their marker instead of showing placeholder question marks.

Non-critical NPCs now react to local optional progress: accepting, advancing,
or completing a chapter's task-board errand adds local street-talk lines, while
searching a map's one-time cache can change nearby rumor text. This keeps side
content from feeling isolated from the chapter world. Local NPCs also remember
chapter照应 beats: if the party stopped for both a bond talk and a camp rest, the
map's street-talk acknowledges it; if the chapter has already moved on without
those moments, returning locals can call out the missed night talk and rest.

Chapter-specific rest scenes now exist as a companion-bond framework: ordinary
spirit lantern props can trigger one-time Linger scenes per story segment, grant
small recovery rewards, show overall bond progress in the exploration HUD, and
strengthen companion support in combat. Higher bond progress improves Linger's
healing support and Lin Yueheng's follow-up damage. Once Linger and Lin Yueheng
are both in the party and at least one bond scene has been seen, combat unlocks
a manual 合击 command that spends MP for a stronger party attack. Bond response
choices now also grant a one-battle protection (`Courage` / `Tender`) that is
consumed on the next battle and appears in battle messages and the HUD.

Party growth now has more than one companion hook: Linger joins during the
opening investigation and provides healing support, while Lin Yueheng joins
after the village commission turn-in and provides a sword follow-up in battle.
Joined companions also appear as overworld paperdoll followers behind the hero,
so the party is visible during exploration instead of only in HUD text.

A lightweight economy loop now exists: battles and task-board commissions grant
money, the HUD shows current funds, merchant NPCs can sell potions, and inn/home
rest services can restore HP/MP. This gives longer routes a consumable/recovery
cycle instead of relying only on fixed starting potions.

Chapter-specific camp rest scenes now sit on top of home/inn/camp services.
Resting at chapter-appropriate NPCs can trigger one-time party camp dialogue,
record camp progress in the HUD, and grant one active camp preparation for the
next battle (`Warmth`, `Focus`, or `Vigil`). Camp preparation is consumed when
combat starts and stacks separately from shrine offerings, so towns and safe
rooms have a tactical reason to revisit them between dungeon pushes.
New camp rests now end with a party tactic choice instead of only assigning a
fixed chapter bonus: the player can choose balanced breathing, Lin Yueheng's
breakthrough plan, or Linger's protection plan for the next encounter.
The HUD also reports `旅途照应`, comparing chapter-available bond and camp scenes
against scenes the player actually saw. The finale epilogue now calls out
whether every night talk and rest was seen, or how many照应 moments were missed.

Chapter maps also contain one-time exploration caches on shrine/crystal/totem
props. These can be searched once for small exp, potion, and money rewards; the
state is stored in `QuestLog`, so re-entering a map cannot farm the same cache.
They are meant to make non-board props feel useful without interfering with
task boards, bond lanterns, or puzzle switches.

Shrine props now support a follow-up offering once their exploration cache has
been handled. Spending money at a shrine grants one active battle blessing
(`Guard`, `Sword`, or `Spirit`) that is consumed by the next encounter and
changes combat numbers through mitigation or bonus damage. This gives small
chapter props a tactical consequence instead of only flavor text.

Exploration maps now spawn chapter-specific ambient motion layers. Village and
river-town routes use small spirit lights, bamboo and reed-bed maps drift with
leaves, cave/finale spaces breathe with mist, plague village has falling rain,
and capital/southern maps carry mirror dust or storm sparks. The ambience is
rebuilt with map content and remains compatible with fog of war and lighting.
Static-looking scene entities are no longer acceptable baseline art: NPCs,
quest boards, spirit lanterns, crystals, shrines, and portals carry idle motion
profiles and synchronized light pulses so exploration scenes have visible life
even before full authored sprite sheets exist for every role.

Combat feedback should stay readable even when enemy art is single-frame: player
attacks, spells, combo attacks, enemy strikes, and healing emit hit flashes plus
floating numbers so every action has visible timing and impact. Battle scenes
also use chapter-specific backdrops from the same tile asset set. Generated
enemy cutouts are now the primary battle bodies, with the shared monster action
sheet reduced to a translucent aura/attack overlay so every encounter keeps its
own silhouette without losing readable motion.

Joined companions now participate visually in battle instead of standing as
static cutouts: Sword Sister lunges and flashes during follow-up windows, Linger
steps into combo casting and support healing, and party combo attacks spawn
separate cast runes from the hero and both companions before the impact.

Major bosses now have signature enemy-turn moves tied to their chapter identity
instead of sharing only the generic claw/low-HP attack. Moon Wraith drains MP,
River Demon slams the party with waves, Miasma Root mixes damage with spiritual
disruption, Mirror Minister punishes with reflected sword-light, Thunder Qilin
bursts through defense with lightning, and Dream Eclipse can recover HP through
old-dream water. This keeps boss encounters distinct across the longer route.

## Required Pillars

- Exploration: tile maps, fog of war, lighting, map props, NPC dialogue, portals
  with story gates.
- Story: visible chapter title, one-time chapter-intro cards, active objective,
  NPC quest markers, branching responses based on story state.
- Combat: turn-based menu, spells as visible impact effects, different enemy
  art per encounter, level growth, consumables.
- Art: generated NPC/creature/prop/tile assets are produced through the ComfyUI
  batch script and documented in `ASSETS.md`.
- Duration: each chapter needs main steps, optional NPC/side content, dungeon
  loops, and encounter pacing. A chapter is not considered complete by merely
  adding a map.

## Current Chapter State Machine

Implemented states live in `src/game/quest.rs`.

1. `NotStarted` -> talk to red sword sister.
2. `TalkToLinger` -> talk to Linger for the tracking talisman.
3. `FindStarMage` -> enter bamboo path and talk to Star Mage.
4. `DefeatMonsters` -> win two random battles.
5. `ReturnToSister` -> turn in the village commission.
6. `EscortMerchant` -> talk to the merchant and reopen the road.
7. `FindBambooScout` -> find the scout on bamboo path.
8. `SeekCavePriestess` -> enter the moon cave and talk to the priestess.
9. `CaveTrial` -> enter the moon-echo corridor route, win three cave-trial
   battles, and activate the two Water Moon crystal switches.
10. `ConfrontMoonWraith` -> after three cave-trial battles and two moon-crystal
    switches are completed, return to the priestess and trigger the chapter boss.
11. `ReturnToLinger` -> report back to Linger and close the opening chapter.
12. `OpeningComplete` -> travel through the cave exit toward River Town.
13. `GatherRiverHerbs` -> win two river-town grass encounters to gather herbs.
14. `ReturnToHerbHealer` -> return to the river-town healer.
15. `FindRiverBoatman` -> investigate the dock with the boatman.
16. `TuneRiverLanterns` -> enter the reed bed and light upstream, midstream,
    then dock lanterns to force the river demon into view.
17. `ConfrontRiverDemon` -> return to the boatman and trigger the river boss encounter.
18. `RiverTownComplete` -> travel through the river-town gate toward Plague Village.
19. `SeekPlagueElder` -> find the plague-village elder.
20. `SeekShrineKeeper` -> take the plague report to the shrine keeper.
21. `CleansePlagueShrines` -> enter the plague-shrine route from the plague
    village gate and win three plague-source encounters.
22. `SealPlagueWards` -> visit the old shrine, bitter well, and sickroom bell
    points on the plague-shrine route after the plague-source encounters; all
    three must be sealed to recover shrine ash.
23. `ReturnToShrineKeeper` -> return shrine ash to the shrine keeper.
24. `ConfrontMiasmaRoot` -> trigger the plague root boss.
25. `PlagueVillageComplete` -> travel through the plague-village gate toward the capital.
26. `SeekCapitalEnvoy` -> find the capital envoy.
27. `FindMansionSpy` -> meet the mansion insider.
28. `GatherSecretLetters` -> enter the mirror-gallery route from the mansion
    gate and win two infiltration encounters to recover secret letters.
29. `AlignMansionMirrors` -> align the mirror-gallery ledger mirror and witness
    mirror so the secret letters reveal the mirror minister's true name.
30. `ReturnToMansionSpy` -> return the decoded letters to the insider.
31. `ConfrontMirrorMinister` -> trigger the capital mansion boss.
32. `CapitalIntrigueComplete` -> travel through the capital gate toward Southern Spirit Road.
33. `SeekSpiritGuide` -> find the southern road guide and receive the spirit road pass.
34. `SeekTribalChief` -> bring the pass to the tribal chief.
35. `CleanseSpiritTotems` -> enter the thunder-drum route from the southern
    road gate and win three encounters to calm thunder totems.
36. `AlignThunderDrums` -> after the thunder totem encounters, align the wind,
    cloud, and oath drums on the thunder-drum route so the storm glyph can form.
37. `ReturnToTribalChief` -> return the storm glyph to the tribal chief.
38. `ConfrontThunderQilin` -> trigger the thunder qilin boss encounter.
39. `SouthernRoadComplete` -> travel through the southern road gate toward the finale.
40. `SeekFinalOracle` -> find the Spirit Abyss gatekeeper.
41. `LightFinalSoulLamps` -> enter the old-dream waterway route and interact
    with three distinct memory lamps.
42. `ReturnToFinalOracle` -> return the dream pearl to the gatekeeper.
43. `ConfrontDreamEclipse` -> trigger the final boss encounter.
44. `FinaleComplete` -> final boss defeated; return to the gatekeeper for the
    route epilogue and homecoming seal.

## Implemented Side Quest Boards

Side quest state lives in `src/game/quest.rs` under `SideQuest`.

1. `VillageTrail` -> village board errand to clear two remaining trail monsters.
2. `VillageHerbs` -> follow-up village board errand to escort the herb path.
3. `MoonCaveCrystals` -> moon cave board errand to purify two crystal disturbances.
4. `MoonCaveEchoes` -> follow-up moon cave board errand to quiet three echo shades.
5. `RiverLanterns` -> river-town board errand to patrol two river lantern anomalies.
6. `RiverCargo` -> follow-up river-town board errand to retrieve two soaked cargo bags.
7. `PlagueRelief` -> plague-village board errand to clear three urgent miasma pockets.
8. `PlagueMedicine` -> follow-up plague-village board errand to escort the medicine child.
9. `CapitalPatrol` -> capital board errand to defeat two mirror-array remnants.
10. `CapitalRumors` -> follow-up capital board errand to intercept three mirror notes.
11. `SouthernThunder` -> southern-road board errand to calm three thunder-path echoes.
12. `SouthernDrums` -> follow-up southern-road board errand to settle two war-drum spirits.

Each board has two visible commissions, and every commission interaction has
explicit accept, progress, turn-in, reward, and completed text. The HUD mirrors
the local board list so accepting a commission does not rely only on transient
dialogue, while an active task tracker stays visible after pickup with the
commission route, progress, and turn-in point.

Side quest rewards now include experience, potions, and money so optional
content feeds the shop/inn economy. Clearing both local board commissions also
unlocks local favor: shop and inn services in that chapter hub acknowledge the
completed work and reduce key service prices, while camp-rest maps show locals
helping the party keep watch.

## Implemented Finale Slice

- Final hub map: `MapKind::FinalSanctum` / 灵渊终门, holding the gatekeeper,
  final boss handoff, and epilogue return.
- Final route map: `MapKind::DreamWaterway` / 旧梦水廊, entered from the
  Spirit Abyss Gate while the lamp objective is active.
- Final puzzle: three remembered lamps (`FinalLamp::Memory`, `FinalLamp::Vow`,
  `FinalLamp::Fate`) live on 旧梦水廊 and must each be lit once; repeated
  interaction with the same lamp does not advance the puzzle.
- Relationship beat: Linger has finale-specific dialogue before and after the
  final boss.
- Final boss: `BossKind::DreamEclipse` / 宿命水影, using a generated creature
  asset and the existing spell impact loop.
- Post-boss epilogue: talking to the gatekeeper after the final boss grants
  `HomecomingSeal` / 归梦印 and shows ending lines influenced by bond tendency,
  side-quest completion, and camp-scene progress.

## Implemented Chapter Puzzles

- Moon cave crystal array: `MapKind::Cave` / 水月洞天 now acts as the priestess,
  task-board, rest, and boss-handoff hub. During `CaveTrial`, its east gate
  leads to `MapKind::MoonEchoCorridor` / 水月回廊, where two distinct cave
  crystals (`MoonCrystal::North`, `MoonCrystal::South`) must be activated.
  Clearing the three enemy waves without the corridor crystal switches leaves
  the player in the cave objective until the moon bridge is complete.
- River reed-bed lantern order: after the boatman names the river demon threat,
  three river lanterns (`RiverLantern::Upstream`, `Midstream`, `Dock`) must be
  lit in order. A wrong lantern resets the puzzle with feedback; the correct
  sequence grants the ferry token and unlocks the river boss handoff.
- Plague shrine-route ward bells: after the three plague-source encounters,
  `PlagueWard::OldShrine`, `BitterWell`, and `Sickroom` must each be sealed on
  the separate 瘴雨祠道 map. The shrine ash is only awarded after all three ward
  points are complete, then the shrine keeper back in 瘴雨村 can trigger the
  miasma-root boss.
- Capital mansion mirror routing: after both secret letters are recovered in the
  separate 照影镜廊 route map, `MansionMirrorNode::Ledger` and
  `MansionMirrorNode::Witness` must both be aligned before the insider back in
  照影府邸 can expose the mirror minister.
- Southern thunder drums: after three thunder totem encounters on the separate
  雷鼓祭道 route map, `ThunderDrum::Wind`, `Cloud`, and `Oath` must each be
  aligned. Only the complete drum circuit grants the storm glyph and upgrades
  the player spell to 万剑诀 before the thunder qilin handoff back on 南疆灵道.
- Finale memory lamps: the three final lamps are on the separate 旧梦水廊 route,
  keeping the 灵渊终门 hub focused on the gatekeeper, boss handoff, and epilogue.

## Implemented Bond Rest Scenes

Bond scene state lives in `src/game/quest.rs` under `BondScene`.

1. `VillageFirstNight` -> early village-road rest scene after Linger joins.
2. `MoonCavePromise` -> cave / moon-water promise scene.
3. `RiverLampWish` -> river-town lantern wish scene.
4. `PlagueRainShelter` -> plague-village rain shelter scene.
5. `CapitalRooftop` -> capital rooftop scene.
6. `SouthernRoadOath` -> southern spirit road oath scene.
7. `FinalGateQuiet` -> finale gate quiet-water scene.

Bond response choices grant the next battle `勇心护念` or `柔心护念`: courage
leans into extra attack / combo power, while tender improves spell pressure,
small damage blocking, and Linger's support heal.

## Implemented Camp Rest Scenes

Camp scene state lives in `src/game/quest.rs` under `CampScene`.

1. `VillageHearth` -> home-lamp rest with婆婆, Linger, and Lin Yueheng.
2. `MoonCavePool` -> cave pool planning scene.
3. `RiverTownInn` -> river-town inn night soup scene.
4. `PlagueSickroom` -> plague-village sickroom watch scene.
5. `CapitalSafehouse` -> capital safehouse planning scene.
6. `SouthernCampfire` -> southern-road campfire watch scene.
7. `FinalStillWater` -> finale still-water party resolve scene.

## Next Content Needed

- Add more authored optional quest variants, NPC reactions, and small branching
  consequences on top of the task-board and bond-scene frameworks.
- Add deeper party interactions outside battle: companion-specific scene
  variants and stronger mechanical aftermath for missed or completed照应 scenes.
- Expand puzzle props beyond the moon-crystal, river-lantern, plague-ward,
  mansion-mirror, thunder-drum, and finale-lamp frameworks: shrine-offering
  consequences and denser dungeon routing.
