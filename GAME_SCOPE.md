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
combat and matching map-action progress, shows board markers, turns in
completed work, and grants exp/potion/money rewards. These side quests are meant
to provide pacing filler between main story beats, not replace authored chapter
scenes. The exploration HUD now also shows a task book with the main objective,
the current map's full local quest-board list, and accepted side-quest progress.
Interacting with a quest board opens a board overview, then a selectable
commission menu, then a structured commission page with status, issuer, receipt id, route hint,
objective, progress, and reward before showing the accept/progress/turn-in
result. Every selectable board row now reads like a commission ticket with the
intended action, progress, goal, and receipt id, and every preview/claim/check-in
screen includes a reusable `委托契约` block with stage, recommended action,
issuer, target area, route, turn-in point, reward, receipt state, full step
plan, and the current next step. Accepting a commission writes a named
commission receipt into the task book, shows a `签收回执` with first step,
field-choice hint, HUD tracking text, and turn-in target, then echoes the
issuer, execution area, route hint, authored field-step preview, tracking hint,
turn-in location, and reward so pickup has a clear RPG handoff. A persistent tracking card now stays
on the exploration screen: it always shows the current main objective, and it
appends the active commission with receipt id, progress, current step, location,
route hint, and turn-in target after a side quest is accepted. Field progress
messages also name the next authored step instead of only showing a counter. The HUD also calls out the
current commission target area and unhandled/handled field choice when the
player is standing in the accepted errand's relevant map. The right-side tracker now also adds `任务引路`, comparing
the current map against the active main route, commission target/turn-in point,
and NPC errand delivery map so accepted tasks say whether this is the target
area or which named place to visit next. Available commissions require explicit `领取并追踪`
confirmation, while ready-to-turn-in commissions now ask for a delivery
resolution: `稳妥封存`, `追查余波`, or `先不处理`. The chosen resolution is archived
on the commission receipt, appears in task-board/NPC text, and `追查余波`
strengthens the matching route's `委托清障` encounter reduction. The board marker
only changes to completed after both local errands are done. Local NPC
commission contacts now mirror the same pickup loop: facing the village elder,
cave gatherer, river guard, plague warder, capital astrologer, or southern
totem witch shows a bottom prompt, floats a task badge, previews the current
local commission, and can open the same accept or turn-in confirmation without
forcing every errand through the physical board.

Ordinary non-board NPCs now also support explicit cross-map `NPC托付` chains.
Eight authored errands (`竹露送药`, `月苔渡江`, `芦滩小信`, `病童护符`,
`星图密片`, `镜廊药引`, `雷草药酒`, `旧梦灯芯`) start from route-side NPCs,
require confirmation before they enter the task book, surface in HUD / tracker /
facing prompts, then complete only when the player talks to the named target NPC
on another map. Delivery pays a one-time `托付回礼` with exp, potion, money, and
small HP/MP recovery, so local people can ask for specific help without every
pickup being a board contract. Completed NPC errands now also leave a
`托付回声`: the matching hub or route lowers encounter pressure by 3%, paired
route errands can stack to 6%, HUD route pressure reports the change, route-mark
checkpoints mention the delivered item, and ordinary local NPCs add follow-up
street-talk.
Accepted commissions can now also advance through first-time matching puzzle or
map-objective interactions: moon crystals, river lanterns, plague wards,
mansion mirrors, thunder drums, and finale lamps can add `委托推进` messages to
their associated local errands, while repeated interaction with the same cleared
prop cannot farm progress. Follow-up commissions also read route exploration:
route marks and route-branch resolutions can advance echo, cargo, medicine,
rumor, drum, and homeward-vow errands when they match the current route. Combat
victory still advances active errands, so older flow remains compatible.
Commission preview, pickup, in-progress check, and turn-in now also add a
`队伍回响` line. The speaker is chosen from the current party: Li Xiaoyao fills
early solo errands, Linger reads unstable spiritual traces, Lin Yueheng comments
on martial / capital routes, and Nanyao takes priority on southern and finale
spirit-road commissions. This makes optional errands part of the travelling
party's story texture instead of isolated board counters. Commission aftermath
now also affects travel instead of stopping at payout: completing one local
errand gives the matching hub/route a small `委托清障 半稳` encounter reduction,
clearing both local errands upgrades it to `委托清障 已清`, HUD route pressure
shows the change, route-mark checkpoints can mention the cleared local work, and
ordinary route NPCs add `委托回声` street-talk for partially or fully cleared
commission chains. Those street-talk lines also report whether completed
commissions were sealed for local stability or left as follow-up route
investigations. Clearing both local errands in a chapter now also unlocks a
one-time `清账回礼` from non-critical local NPCs, paying a small exp, potion, and
money gift with chapter-specific thank-you text, so returning to town after
finishing errands has a concrete payoff beyond the board payout and shop favor.

Main-story NPC handoffs now use the same deliberate pickup shape for claim,
advance, turn-in, and boss handoff moments: the player sees a structured main
task card and confirms (`领取并追踪`, `继续主线`, `交付主线`, or `迎战首领`) before
the quest log changes or a boss battle starts. Each authored campaign stage now
also has a `主线簿` route-book entry with chapter step, receipt id, place,
contact, and action, so the confirmation card, HUD task book, and persistent
tracking card all show where the current handoff belongs inside the chapter
instead of relying on a single objective sentence. Confirmed main-task changes
raise a visible task-book update notice with the current route-book entry and
refresh the persistent tracking card.
Every story chapter now has a one-time卷章卡 with six beats: title, visual
motif, emotional tone, story summary, gameplay hint, and next objective. The
visual motif / tone lines cover all seven chapters, including the capital,
southern road, and finale, so later acts no longer read like generic text-only
handoffs. Open-RPG chapter cards also raise a full-screen chapter painting while
the six card lines are being read; chapters 5-7 now have dedicated imagegen
backdrops for capital intrigue, southern thunder, and the finale spirit abyss
instead of reusing early-act UI art. Those late backdrops also have 32-frame
atlas loops, and the Explore卷章卡 UI plays the atlas at 16fps so chapter
handoffs are animated instead of static splash screens. Story portals now also
raise a short `界门` transition after a successful map change, naming the origin
and destination, showing the current route-book receipt, and stating whether the
landing map is the active main-task target or only a passage toward it.
Authored exploration chapters now also archive chapter seals. The first village
road handoff records the 余杭 seal when the cave route opens, each later boss
adds its chapter seal, the HUD/tracking card shows `章印 x/7`, and the finale
epilogue reflects how many chapter seals were actually archived. Legacy boss
victories now also grant chapter breakthrough growth to HP, MP, attack, and
defense, so chapter bosses change the party sheet instead of only advancing a
flag.

The roguelike route now mirrors that authored handoff at map scale: the 41
generated journey stages are split into seven story chapters (余杭夜雨、水月洞天、
苏州江灯、白河疫雨、京华镜影、南疆雷誓、心渊照影). Each stage has a
one-time入图对白 with the current place, scene setup, party reaction, and
objective before movement resumes. Each stage also binds to an authored
`MapKind` in `JOURNEY_MAPS`, so route beats move through village, bamboo road,
cave, river town, plague road, capital, southern storm path, and finale waterway
tilesets while the internal terrain still rolls procedurally. Those route
openings also issue a visible `主线契约`:
the player can confirm `领取并追踪` or look around first, and unsigned routes
reopen the contract before main markers or the gate resolve. Once the required
markers are clear, the route gate now turns the signed receipt into an
`已归档` entry before advancing, so accepting and turning in a task are both
explicit player-facing interactions. The map HUD,
blocked gate prompt, and Esc inventory all show the same receipt, signed state,
completion condition, and required-marker progress, so claiming a route task
has persistent follow-through instead of being only a transient dialogue line. Run-mode party
presence now also advances with the route: Linger, the sword sister, and the
southern spirit witch appear as paperdoll followers on maps and as visible
companions in run battles once their story segment has joined the road. Their
run-mode battle participation now has mechanics too: Linger can restore HP
after enemy turns, the sword sister can add follow-up cuts to attacks and
spells, and Nanyao can restore MP while also unlocking the late-spell name,
power, element flavor, and VFX cadence for the run route.
Run-mode chapter story choices now leave a `本卷誓记` instead of only moving
the ending counters: 情缘-leaning choices record `护心誓`, 道心-leaning choices
record `破势誓`, and balanced choices record `同心誓`. The current chapter's
vow is shown in the HUD/inventory, is listed on the ending screen, and the
chapter boss reads it at battle start to reduce attack, reduce max HP, or
slightly suppress both. This gives each chapter's relationship choice a local
boss consequence as well as a final-ending consequence.
Required run-mode `缘` markers now have enough authored choice scenes to avoid
repeating dialogue in a normal playthrough: 22 scenes cover the 21 required
story markers, with three southern scenes and seven finale scenes. `RunState`
draws unseen scene indices within each chapter before allowing a reset, reports
current `缘忆` progress in the journey summary, and includes the viewed count in
the ending record. Southern old-drum and finale sword-mirror scenes also select
Nanyao or Lin Yueheng portraits instead of always showing Linger.

Run-mode rest markers are no longer generic recovery stops. They now open
chapter-aware camp scenes using the current route place and party composition,
record `营火照应` in the run summary, and let the player prepare one next-battle
`营策`: `调息` opens with HP/MP recovery and minor damage, `剑守` improves sword
and burst damage, `灵护` adds opening protection plus damage block, and `凝灵`
unlocks after Nanyao joins to improve spell/burst pressure and guard MP recovery.
The prepared tactic is consumed when combat starts and is shown in the battle HUD.

Run-mode maps also add one optional local contact per non-boss route stage.
These `Guide` markers are visible NPC sprites rather than anonymous `?`
markers; each uses a route-appropriate AI portrait and opens a lightweight
`路人签` contract. Accepting it records a local target in the HUD/inventory,
clearing that target auto-files the receipt as `路人回声`, and only then pays
small supplies, money, HP, or MP without blocking the main gate. Completed
receipts now also accumulate chapter-local `路况照应`: at one, two, and three
helped contacts, the current chapter's grass encounter chance is reduced by 8%,
16%, and 24%, and the HUD/inventory report the current road-pressure state.
Those same same-chapter receipts now feed the chapter boss opening as
`首领照应`: two helped contacts grant `乡路照应`, lowering boss attack by 4%
and restoring HP/MP before the fight; three or more grant `熟路照应`, lowering
boss HP by 4%, boss attack by 8%, and giving the stronger opening supply.

Run-mode now also mirrors the chapter puzzle vocabulary at map scale. Route
maps for moon corridors, reed-bed lanterns, plague shrine bells, mansion
mirrors, thunder drums, and dream water lamps spawn visible required
`Puzzle` mechanisms instead of only anonymous encounter markers. Solving one
opens route-specific text, pays a small reward, increments `机关破除` in the run
summary, and counts toward the same main-marker gate condition as fights and
story nodes.

Quest direction should be visible in-world, not only in HUD text: active NPCs,
task boards, and puzzle props use floating badge markers for available (`!`),
in-progress (`*`), and completed (`✓`) states. Inactive dialogue targets hide
their marker instead of showing placeholder question marks.

Non-critical NPCs now react to local optional progress: accepting, advancing,
or completing a chapter's task-board errand adds local street-talk lines, while
searching a map's one-time cache can change nearby rumor text. Route-branch
choices also feed back into local street-talk: hub and route NPCs in the same
chapter distinguish cautious scouting from rushing through a detour. This keeps
side content from feeling isolated from the chapter world. Local NPCs also
remember chapter照应 beats: if the party stopped for both a bond talk and a camp
rest, the map's street-talk acknowledges it; if the chapter has already moved on
without those moments, returning locals can call out the missed night talk and
rest. Completing both照应 beats in a chapter now also unlocks a one-time local回礼
from non-critical NPCs, paying small exp, potion, and money rewards without
repeating when the player revisits the same chapter maps.

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
opening investigation and provides healing support, Lin Yueheng joins after the
village commission turn-in and provides a sword follow-up in battle, and Nanyao
joins on the Southern Spirit Road as a late-route spirit witch who steadies MP
recovery and expands the party combo. Joined companions also appear as overworld
paperdoll followers behind the hero, so the party is visible during exploration
instead of only in HUD text.
Spirit lanterns now also support companion-specific personal vignettes after the
chapter's Linger bond talk has been seen: Lin Yueheng gets trail, capital, and
finale sword scenes, while Nanyao gets southern totem and finale vow scenes.
These `同伴小传` moments are one-time, tracked in the HUD (`小传 x/5`), grant
small exp/potion rewards, can seed a non-overwriting next-battle tactic, and are
counted in the finale route echo alongside bond and camp completion. Completed
personal scenes now also feed back into route pressure: the matching route maps
gain a `小传照应` encounter-rate reduction, and route-mark checkpoints add
companion-specific lines when Lin Yueheng or Nanyao has already resolved the
relevant personal scene. Missing those chapter-specific personal scenes is also
visible: later route maps show `小传照应 错过`, slightly raise route encounter
pressure, route-mark checkpoints mention the missing companion beat, and local
non-critical NPCs add street-talk for both completed and missed companion
vignettes. Returning to those NPCs after a completed personal scene now also
plays a scene-specific `小传回访` and grants a one-time `小传回礼`; each of the
five vignettes has its own local response, and the two finale vows can both be
settled in the same conversation without repeating on later visits.
Three earlier missed vignettes now have playable cross-chapter recovery chains
instead of permanent penalties: `旧栅余声` starts in Plague Village and resolves
at the shrine-path bell mark, `照影迟问` starts on the Southern Road and resolves
at the thunder switchback, and `雷纹归愿` starts at the Final Sanctum and resolves
in the Dream Waterway. Each uses an explicit NPC pickup, HUD/task-book contract,
field marker scene, return guidance, and NPC turn-in before restoring the scene's
route relief and paying its one-time aftermath reward.

A lightweight economy loop now exists: battles and task-board commissions grant
money, the HUD shows current funds, merchant NPCs can sell potions, and inn/home
rest services can restore HP/MP. Four chapter hubs now also sell one-time gear
before falling back to potions: `竹剑穗`, `江绫护衣`, `照影护心镜`, and
`雷纹护符` improve attack, defense, HP, or MP, and the HUD tracks `装备 x/4`.
This gives longer routes a consumable/recovery cycle plus a persistent
equipment-growth layer instead of relying only on fixed starting potions.

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
Route maps now turn that照应 record into route pressure: if the chapter's bond
talk and camp rest were both seen, grass ambush pressure on that chapter's
route drops; seeing only one scene gives a smaller reduction; pushing into a
route without either preparation raises the encounter pressure, and revisiting a
missed old route raises it further. Route-mark checkpoints echo this party
state in their dialogue, so companion care has a map-level consequence outside
combat instead of only a battle bonus or ending line.
Legacy chapter boss openings now read the same preparation record. Seeing the
chapter night talk and camp rest, finishing both local commissions, and clearing
chapter companion scenes combine into `首领照应`; a fully prepared party opens
with boss HP/ATK pressure and small HP/MP recovery, partial preparation gives a
lighter opening, and skipping preparation lets the boss seize initiative.

Chapter maps also contain one-time exploration caches on shrine/crystal/totem
props. These can be searched once for small exp, potion, and money rewards; the
state is stored in `QuestLog`, so re-entering a map cannot farm the same cache.
They are meant to make non-board props feel useful without interfering with
task boards, bond lanterns, or puzzle switches.
Every authored exploration map now also has one visible field-supply pickup:
herbs, dew, tea packs, pantry bundles, powders, incense, or dream-water moss.
Supply nodes carry their own `!` / `✓` markers, show a bottom prompt, record
`采集 x/15` in the HUD, and reward small route resources plus immediate HP/MP
recovery once. This makes map traversal carry light resource management between
major treasure caches and fights.

Shrine props now support a follow-up offering once their exploration cache has
been handled. Spending money at a shrine grants one active battle blessing
(`Guard`, `Sword`, or `Spirit`) that is consumed by the next encounter and
changes combat numbers through mitigation or bonus damage. Active shrine
offerings also affect route pacing before that battle: guard incense lowers
grass ambush risk, sword incense draws out enemies for grinding or task progress,
and spirit incense lightly steadies the route. This gives small chapter props a
tactical consequence instead of only flavor text.

Dungeon routes now also contain one-time `路印` / route-memory checkpoints.
Investigating a route mark records local traversal knowledge, updates the
exploration HUD (`路印 x/6`), changes its in-world marker from available to
completed, and slightly reduces grass ambush pressure on that same route. These
marks make route maps denser without turning every prop into a required puzzle:
the main moon crystal, river lantern, plague ward, mansion mirror, thunder drum,
and finale lamp objectives still stay mandatory, while route marks are optional
scouting rewards.

Each route map now also has a one-time `路线分支` event separate from the route
mark. These show as a visible full-width `？` marker, open a two-choice
resolution (`细查岔路` or `快步穿过`), record the chosen branch in `QuestLog`,
pay small exp/potion/money rewards, and change that route's future grass-ambush
pressure by choice. The HUD tracks `分支 x/6`, so optional route content now has
a persistent pickup, completion marker, reward, route-level consequence, and
local NPC follow-up. Route branches now also become a small party-tactics beat:
preview text calls out which companion will read the route, resolution text adds
Linger / Lin Yueheng / Nanyao advice when they are present, and the first branch
choice without an active camp tactic grants a one-battle preparation
(`灵儿守护`-style vigilance for careful scouting, `月衡破势`-style focus for
pressing through). If the player already has a camp tactic queued, the branch
keeps that existing preparation instead of overwriting it.

Resolved route branches now also create a local `路报` follow-up. Returning to a
non-critical local NPC after a branch is handled files the branch into the local
route book, grants a one-time `报路回礼`, and records `路报 x/6` in the HUD. The
report text differs for careful scouting vs rushing through, so route choices
have a delayed town-side echo instead of only immediate rewards.

Exploration maps now spawn chapter-specific ambient motion layers. Village and
river-town routes use small spirit lights, bamboo and reed-bed maps drift with
leaves, cave/finale spaces breathe with mist, plague village has falling rain,
and capital/southern maps carry mirror dust or storm sparks. The ambience is
rebuilt with map content and remains compatible with fog of war and lighting.
Run-mode route maps also carry their own per-stage fog memory: unknown cells are
cool and dim, the hero's local vision clears immediately, and previously walked
cells remain readable under a lighter explored veil when the player returns from
battle.
Authored chapter maps should also differ in actual tile layout, not only palette:
river reed-bed, plague village, plague shrine road, capital streets, mansion
infiltration, thunder-drum route, final gate, and old-dream waterway now use
separate row layouts, and tests require every authored map pair to differ by at
least five rows while high-risk former clone pairs differ by at least ten rows.
Run mode now consumes all 15 of those authored layouts as topology blueprints
instead of feeding every place through one generic cellular-automata map. Stage
variants alter approach direction and terrain edges while retaining local
landmarks; generation must keep 150 connected walkable cells, a 24-step route,
and enough separated positions for every required marker, map mechanism, and
route contact. `scripts/map_harness.sh` validates all 41 stages and captures the
15 real rendered maps for visual comparison.
Static-looking scene entities are no longer acceptable baseline art: NPCs,
quest boards, spirit lanterns, crystals, shrines, and portals carry idle motion
profiles and synchronized light pulses so exploration scenes have visible life
even before full authored sprite sheets exist for every role. Generated NPC
cutouts are also sliced into lower-body, torso, and head parts at runtime, with
independent sway, breathing, and brightness motion, so existing AI character art
does not behave like a single pasted card. Explore characters now also spawn
pulsing contact shadows, and moving player/party sprites emit short fading
afterimages, following the protect-carrot baseline of walk animation plus ground
contact instead of static sliding sprites.

Combat feedback should stay readable even when enemy art is single-frame: player
attacks, spells, combo attacks, enemy strikes, and healing emit hit flashes plus
floating numbers so every action has visible timing and impact. Player仙术 stays
focused as an enemy-side hit effect, but the impact palette and combat line now
track the current chapter: village sword qi, moon water, river current, miasma
cleansing, mirror light, thunder, and final-dream water each tint the same
readable hit loop. Spell damage also splits into staged enemy-body hit numbers
(御剑术 as three cuts, 万剑诀 as seven cuts) so the spell reads as impact rather
than a projectile or static card. Battle scenes also use chapter-specific
backdrops from the same tile asset set.
Generated enemy cutouts are now the primary battle bodies and are sliced into
the same three-part runtime rig, so every encounter keeps its own silhouette
without falling back to a mismatched shared monster sheet.

Joined companions now participate visually in battle instead of standing as
static cutouts: Sword Sister lunges and flashes during follow-up windows, Linger
steps into combo casting and support healing, Nanyao raises a green spirit-rune
motion during spell/combo turns and restores MP between enemy turns, and party
combo attacks spawn separate cast runes from the hero and each joined companion
before the impact.

Major bosses now have signature enemy-turn moves tied to their chapter identity
instead of sharing only the generic claw/low-HP attack. Moon Wraith drains MP,
River Demon slams the party with waves, Miasma Root mixes damage with spiritual
disruption, Mirror Minister punishes with reflected sword-light, Thunder Qilin
bursts through defense with lightning, and Dream Eclipse can recover HP through
old-dream water. Boss victories now also resolve with a chapter seal: the reward
screen names the earned 章印, closes the chapter with a short epilogue line,
applies that chapter's breakthrough gains, and carries the seal list into the
ending. This keeps boss encounters distinct across the longer route.

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
  adding a map. The run-mode spine now names 41 authored journey beats across
  seven chapters, each with a place, objective, route-gate prompt, one-shot
  entry dialogue, and a persistent main-task receipt, so the procedural maps
  read as a continuous RPG trip instead of isolated random rooms. Each receipt
  moves through `待签收`, `已追踪`, and gate-side `已归档` states, and each chapter
  boss adds a named 章印 so chapter completion has persistent end-of-act feedback.
  Run-mode now has an explicit 600-minute target ledger (`CHAPTER_PACING`) split
  across the seven chapters, and each stage derives an estimated minute budget
  from its required marker mix. Main-task contracts and run HUD summaries show
  the current stage's required content list, approximate minutes, chapter focus,
  and `全旅程目标 10小时`, so the 10-hour target is visible inside the route rather
  than only documented here. Each beat also owns a required content mix, so
  village/town beats can guarantee story, market, or rest moments while dungeon
  beats guarantee fights, elite pressure, events, and visible route puzzle maps
  inject required mechanism beats.
  Non-boss run stages also place optional local NPC contacts, so the long route
  has recurring small errands and recovery beats beyond combat.
  The HUD, inventory, journey summary, and ending also expose actual total and
  per-chapter play-time ledgers. `scripts/run_audit.sh` now completes a fixed-seed
  traversal through all 41 stages, archives all 41 main-task receipts, earns all
  seven chapter seals, and reaches a victory ending through normal movement,
  dialogue, battle-menu, and reward inputs. Its accelerated fixed-step clock is
  a reachability/bookkeeping proof, not measured human playtime. The 600-minute
  ledger remains a design target until human playtests establish the real curve.

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
13. `FinalDreamEchoes` -> finale board errand to settle three dream-water echoes.
14. `FinalHomewardVows` -> follow-up finale board errand to guard two homeward lamp vows.

Each chapter hub and the finale hub have two visible commissions arranged as a
local chain: the first can be accepted immediately, while the second is
previewed as a follow-up commission and only opens after the first is completed.
Every commission interaction has explicit accept, progress, turn-in, reward,
locked-follow-up, and completed text. The HUD mirrors the local board list so
accepting a commission does not rely only on transient dialogue, while an active
task tracker stays visible after pickup with the commission receipt, issuer,
route, progress, current authored step, and turn-in point. The current-map HUD
adds `当前委托区` when an accepted commission matches that area, so the player can
tell they have walked into the right place. Each of the 14 commissions now has
one authored field-step label per progress point, and task-board detail,
contract preview, accepted tracker, and `委托推进` feedback all reuse the same
step plan.

Every local commission also has one authored on-map field trace in its target
area: old bamboo fence claw marks, herb-path wind, moon crystal dust, reed-bed
cargo tracks, plague medicine bells, capital mirror-footprints, thunder drum
low notes, and final dream-water lamp slips. These traces use visible props with
`？` before pickup, `!` while the matching commission is active, and `✓` after
the trace is handled; interacting with a trace advances that commission only
once and then records the handled现场 in the HUD as `委托现场 x/14`. Active
trace interaction now asks how to process the field: `细查现场` keeps a slower
investigation trail, while `快断余妖` records a fast suppression method. The
field choice is stored on the commission receipt, appears in the contract,
turn-in receipt, route-pressure summary, and local NPC `现场回声`.

Side quest rewards now include experience, potions, and money so optional
content feeds the shop/inn economy. Accepting a commission now also grants a
small one-time advance supply package, so task pickup has an immediate gameplay
payoff before the final turn-in reward. Clearing both local board commissions
also unlocks local favor: shop and inn services in that chapter hub acknowledge
the completed work and reduce key service prices, while camp-rest maps show
locals helping the party keep watch.

## Implemented Finale Slice

- Final hub map: `MapKind::FinalSanctum` / 灵渊终门, holding the gatekeeper,
  finale task board, final boss handoff, and epilogue return.
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

## Implemented Companion Personal Scenes

Companion personal scene state lives in `src/game/quest.rs` under
`CompanionScene`. These scenes trigger from spirit lanterns only after the
current chapter's Linger bond scene has been viewed, so they deepen the party
instead of replacing the main romance bond chain.

1. `SwordSisterTrailGuard` -> Lin Yueheng's early road-guard scene.
2. `SwordSisterCapitalMirror` -> Lin Yueheng's capital mirror-case scene.
3. `SpiritWitchSouthernTotem` -> Nanyao's southern totem memory scene.
4. `SwordSisterFinalReturn` -> Lin Yueheng's finale return-sword scene.
5. `SpiritWitchFinalVow` -> Nanyao's finale homeward-vow scene.

## Next Content Needed

- Add more authored optional quest variants beyond the current commission
  chains, especially variants with unique follow-up scenes after the existing
  field-choice and turn-in branches have been recorded.
- Add finale/epilogue recovery consequences for the two final companion
  vignettes. The three earlier missed小传 beats now have full NPC pickup -> field
  revisit -> turn-in chains; the finale pair still needs a post-boss or ending
  variant when left unresolved.
- Expand puzzle props beyond the moon-crystal, river-lantern, plague-ward,
  mansion-mirror, thunder-drum, finale-lamp, shrine-offering, and route-mark
  frameworks with deeper multi-step local variants and stronger NPC follow-up
  reactions.
