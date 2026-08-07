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
| Effects | 2 | `assets/effects/light_orb.png`, `assets/effects/skill-vfx-sheet.png` |
| Actor sheets | 4 | `assets/actors/*.png` |

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
  companion cutouts with their battle motion, and builds zone-specific battle
  backdrops from generated map tiles.
- `src/game/quest.rs` drives the first quest chain through generated and
  paperdoll NPCs.

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
| 金纹面板框 | `assets/ui/panel_frame.png` | Nine-slice UI panel (dialogue/battle/menus/inventory), `event::panel_slicer()` |

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
| CJK font | `assets/fonts/unifont.otf` | GNU Unifont 17.0.04 | Covers Chinese UI/dialogue text. |
