# Assets

The project now uses generated raster assets instead of placeholder blocks.
Most art is produced through the remote ComfyUI instance on
`root@101.78.126.6` via `tools/comfy_batch_assets.mjs`. Late-story NPC
cutouts and late chapter-card backdrops can also be generated with Codex
imagegen; their raw outputs live under `assets/generated/imagegen/raw/`.

## Generation

Start an SSH tunnel to the remote ComfyUI server:

```bash
ssh -N -L 18188:127.0.0.1:8188 root@101.78.126.6
```

Generate or refresh the asset batch:

```bash
COMFY_BASE=http://127.0.0.1:18188 COMFY_STEPS=20 node tools/comfy_batch_assets.mjs
```

The script writes raw outputs to `assets/generated/comfy/raw/`, processed game
assets to `assets/`, and metadata to `assets/generated/comfy/manifest.json`.
Cutouts are generated against a simple background, then
`scripts/cutout_harness.sh` uses local U²-Net segmentation to produce alpha and
fail-closed size/coverage checks. `COMFY_CUTOUT_MODE=chroma` remains an explicit
offline fallback; it is not the default because broad color keys can erase robes
and creature details.

## Current Generated Set

| Type | Count | Paths |
|------|-------|-------|
| NPC cutouts | 19 | 18 × `assets/npcs/ai_*.png`, plus `assets/npcs/star_mage_cutout.png` |
| Creature cutouts | 16 | 9 × `assets/creatures/ai_*.png`, 7 × `assets/creatures/boss_*.png` |
| Map props | 8 | `assets/props/ai_*.png` |
| Tiles | 29 | `assets/tiles/*.png` and `assets/tiles/ai_*.png` |
| Effects | 6 | Orb/impact sheet plus contact shadow, two engraved rings and faceted shard |
| Actor sheets | 3 | `hero-motion.png`, `water-serpent-motion.png`, `monster-actions-sheet.png` |

Late-story NPCs currently added through imagegen:
`ai_plague_elder`, `ai_shrine_keeper`, `ai_capital_envoy`,
`ai_mansion_spy`, `ai_spirit_guide`, `ai_tribal_chief`, and
`ai_final_oracle`. Late chapter-card backdrops currently added through imagegen:
`chapter5_art`, `chapter6_art`, and `chapter7_art`.
The three run-party cutouts are `ai_linger_companion`, `ai_sword_sister`, and
`ai_nanyao_companion`; every chapter boss has a matching `boss_*` cutout.

## In-Game Use

- `src/game/explore.rs` loads generated tiles, props, NPC cutouts, lighting, and
  fog-visible map content across the chapter maps. Main quest NPCs from Plague
  Village onward use distinct generated cutouts instead of generic paperdoll
  reuse. The three companions also use their dedicated cutouts in legacy and
  run-mode exploration while retaining sway, facing, trail, and follow motion.
- `src/game/battle.rs` uses generated creature cutouts as the primary enemy
  bodies, gives all seven bosses dedicated generated bodies, renders the three
  companion cutouts with their battle motion, and selects eight coherent painted perspective arenas by encounter zone.
  The opening water serpent uses its own motion atlas through
  `battle/enemy_animation.rs`; other enemy cutouts retain their identities.
- `src/game/quest.rs` drives the first quest chain through generated and
  paperdoll NPCs.

### Water-serpent motion (2026-10-12)

`actors/water-serpent-motion.png` is a transparent 3072×1152 atlas: 8 columns,
3 rows (idle, bite, hit), 384px cells. All poses share a 360px source scale and
12px margin; they are not independently trimmed. A continuous, scaled blue
serpent baseline was generated with Flux on sg-office, then Wan2.2 I2V produced
three 49-frame clips. Eight genuine body/head poses were selected per action;
bite column4 matches the existing battle lunge/recoil peak. Numerical damage
settlement is unchanged.

Workflows/seeds: `generated/comfy/serpent-motion-wan.json`; original clips and
all147 frames: `generated/comfy/raw/serpent-motion/`. Semantic u2net cutout plus
neutral-white background cleanup removes the surrounding and inner-coil white
space while retaining blue fins and pale textured belly. Rejected reference
edits remain documented and are not runtime assets. Geometry/alpha/distinctness
checks run through `tools/verify_polish_assets.py`; actual combat proof uses
`scripts/enemy_motion_harness.sh` and the `rogue-enemy-motion` capture preset.
`tools/prepare_enemy_motion.py` preserves the actual cutout and atlas assembly
steps; its docstring lists dependencies. It rewrites generated outputs, so run
it before verification rather than during a build or capture.

## UI backgrounds & portrait cards (2026-07, ComfyUI Flux + imagegen)

| Asset | Path | Use |
|-------|------|-----|
| 标题主视觉 | `assets/ui/title_bg.png` | Title screen backdrop |
| 水墨节点图底 | `assets/ui/map_bg.png` | NodeMap backdrop (dimmed) |
| 战利祭坛 | `assets/ui/reward_bg.png` | Reward screen backdrop |
| 月夜渡口 | `assets/ui/ending_bg.png` | Ending screen backdrop |
| 灵儿立绘卡 | `assets/npcs/ai_linger.png` | Run dialogue portrait (story/rest) |
| 主角头像 | `assets/npcs/ai_hero.png` | Run HUD avatar |
| 青石界门 | `assets/props/ai_spirit_gate.png` | Stage exit gate (violet-tinted for boss gates) |
| 红漆宝箱 | `assets/props/ai_chest.png` | Map loot chest marker (mimic risk) |
| 灵泉石池 | `assets/props/ai_spring.png` | Map spirit-spring marker (one-shot heal) |
| 章节过场画 ×7 | `assets/ui/chapter1..7_art.png` | Full-screen chapter-card backdrop; chapters 1-4 are ComfyUI Flux, chapters 5-7 are imagegen and also used by Explore main-story卷章卡 |
| 章节过场动画 ×7 | `assets/ui/anim/chapter1..7_sheet.png` | 32-frame 8×4 atlas (640×352/frame, 16fps loop); chapters 1-4 are Wan2.2 i2v, chapters 5-7 are imagegen-derived pan/zoom light atlases for Explore卷章卡 |
| 面板 | `src/game/ui.rs` | Code-owned ink surface, gold border, rounded corners and soft shadow |

Raw prompts/seeds: see git history of the generation commands; regenerate via
`python3 .claude/skills/godogen/tools/comfyui_gen.py image --prompt ... -o ...`.

## Per-map terrain sources (2026-07, ComfyUI, 512×512)

每张章节地图现在有专属墙/地贴图(此前多图共用竹林/灵草砖只调色)。
命名 `assets/tiles/ai_<map>_wall.png` / `ai_<map>_floor.png`:

| Map | Wall | Floor |
|-----|------|-------|
| RiverReedBed 芦苇荡 | 芦苇丛 | 湿滩泥地 |
| PlagueVillage / PlagueShrinePath 瘴雨村·祠道 | 紫瘴雾岩 | 米色卵石 |
| Capital 京城 | 朱墙金瓦 | 青砖御道 |
| CapitalMansion 府邸 | 木格漆墙 | 花梨地板 |
| MansionMirrorGallery 镜廊 | 铜镜幽光 | 墨玉镜面 |
| SouthernRoad 南疆道 | 丛林藤蔓 | 红土路 |
| ThunderDrumPath 雷鼓祭道 | 蓝雷纹岩 | 青黑石板 |
| FinalSanctum 灵渊终门 | 紫电幽渊 | 蓝渊石 |
| DreamWaterway 梦水道 | 梦纹水墙 | 幻彩水面 |

生成 prompt 模板:`seamless tileable top-down 2D RPG terrain texture tile
for a Chinese xianxia game, <描述>, uniform flat lighting, fills the entire
frame edge to edge, no border, no vignette, crisp fantasy game art`
(wall/floor 各一固定 seed)。生成模型并不总能兑现 seamless/no-vignette，
因此运行时不再逐格压缩整张图，也不依赖图片首尾像素相接：
`explore::TerrainMaterial` 用地图级 terrain-id mask、居中的 61px 质数采样、
世界坐标折返和噪声边界混合来消除逐格接缝。

纹理回归 harness：`scripts/terrain_harness.sh <out>`。它会运行采样连续性
单测，并分别抓取 RunScene/legacy Explore 固定帧及 contact sheet。

## Font

| Asset | Path | Source | Notes |
|-------|------|--------|-------|
| CJK font | `assets/fonts/NotoSansCJKsc-Regular.otf` | Noto CJK / SIL OFL | Smooth Chinese UI/dialogue glyphs; license at `assets/fonts/OFL.txt`. |


## Cohesive UI/battle art (2026-10-06)

- `actors/hero-motion.png`: imagegen, using `npcs/ai_hero.png` as identity
  reference; transparent 1024×1536 atlas, 4×6 cells at 256×256; 8 idle,
  8 walking and 8 actual sword-slash poses. Replaces the three low-resolution
  rig sheets. Idle/attack source: `generated/imagegen/raw/hero-motion-source.png`;
  the articulated walk cycle comes from `hero-walk-source.png`.
  `prepare_polish_assets.sh` composes the three strips. No background or contact
  shadow is baked into the atlas.
- `backgrounds/battle-{village,bamboo,cave,river,plague,capital,southern,dream}.png`:
  eight painted 1024×576 perspective arenas from one consistent imagegen
  2×4 atlas (`generated/imagegen/raw/battle-environments.png`). Run
  `bash tools/prepare_polish_assets.sh` to split the source into runtime assets.
- `ui/icons/{attack,guard,spell,combo,item,flee}.png`: consistent 64px drawn
  gold/ivory symbols. `effects/contact_shadow.png`, `spell-ring*.png` and
  `spell-shard.png` supply alpha-shaded ground contacts and spell surfaces.
  Reproduce code-owned art using `uv run --with pillow python tools/generate_ui_art.py`.
- `fonts/NotoSansCJKsc-Regular.otf`: upstream Noto CJK Sans SC Regular,
  sourced from `notofonts/noto-cjk/Sans/OTF/SimplifiedChinese`; `fonts/OFL.txt`
  contains its SIL Open Font License. Replaces pixel-style Unifont.
- Generation direction: restrained hand-painted Chinese fantasy, same red /
  ochre topknot hero across animation and portrait; eight empty landscape
  arenas with a clear ground plane, subdued atmospheric color, no neon,
  repeated tiles, people, text or UI. Prompt layout is recorded in the asset
  contracts above; imported atlases were visually checked before integration.

Asset sanity check: `uv run --with pillow python tools/verify_polish_assets.py` validates atlas geometry, 24 distinct nonblank alpha frames, eight environments, icons, effects and font license.

## Location identity pass (October 8)

Sixteen matching hand-painted dimensional landmarks come from the built-in
imagegen tool: `assets/props/landmark-*.png` (256×256RGBA, feet at128,250).
The source sprite atlas and generation prompts/alpha checks are retained in
`assets/generated/imagegen/raw/environment-landmarks.png` and
`assets/generated/imagegen/environment-landmarks.manifest.json`.
They depict the actual inn, pier/bridge, village medicine hut, ruined shrine,
capital gate, manor/mirror dais, southern hut/drum and final/dream landmarks.
`assets/tiles/water_surface.png` replaces the old water illustration that contained
an island and trees; water now samples a plain painted ripple texture.
Scene placement and matching collision footprints live in `scene_places.rs`.

The first in-game location renders exposed repeated rocks/trees inside old floor
illustrations. Four dedicated surface textures now replace them:
`tiles/ground-earth.png`, `ground-grass.png`, `ground-stone.png`, and
`ground-rock.png`; source/prompts in `assets/generated/imagegen/ground-textures.manifest.json`.
These contain only small-scale top-down ground detail. Painted natural blockers
remain separate. `Tile::Structure` renders the matching ground beneath architecture
while remaining blocked; open bridge/arch passages are explicitly walkable.
Bridge spans retain the water surface below their transparent arch, and pier
reservation preserves adjoining water. Architecture reveal uses its occupied
footprint, rather than hiding the entire cutout until its centre is discovered.

## sg-office recovery verification (October 10)

ComfyUI API and real image generation are verified again. The local forward is
managed by `love-rpg-comfy-tunnel.service`; direct-key SSH uses
`IdentityAgent=none` because the local SSH agent currently refuses signing.
The restored server exposes both RTX4090 devices. The batch workflow now routes
model/CLIP/VAE through the live device-selection nodes, defaulting to`gpu:1`;
override with`COMFY_DEVICE` when another device is intentionally selected.

A real Flux dev fp8,512×512,20-step herb-basket draft was generated and inspected:
`assets/generated/comfy/raw/service-recovery-herb-basket.png`. It is a generation
smoke-test draft with a background, not an imported runtime cutout. Prompt, seed,
workflow and successful job history are in`screenshots/service-check/generation.json`.
