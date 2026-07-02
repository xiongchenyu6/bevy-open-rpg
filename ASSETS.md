# Assets

The project now uses generated raster assets instead of placeholder blocks.
Most art is produced through the remote ComfyUI instance on
`root@101.78.126.6` via `tools/comfy_batch_assets.mjs`. Late-story NPC
cutouts can also be generated with Codex imagegen on flat chroma backgrounds;
their raw outputs live under `assets/generated/imagegen/raw/`.

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
Cutouts are generated on a flat chroma background and locally converted to alpha.

## Current Generated Set

| Type | Count | Paths |
|------|-------|-------|
| NPC cutouts | 14 | `assets/npcs/ai_*.png`, plus `assets/npcs/star_mage_cutout.png` |
| Creature cutouts | 9 | `assets/creatures/ai_*.png` |
| Map props | 5 | `assets/props/ai_*.png` |
| Tiles | 11 | `assets/tiles/*.png` and `assets/tiles/ai_*.png` |
| Effects | 2 | `assets/effects/light_orb.png`, `assets/effects/skill-vfx-sheet.png` |
| Actor sheets | 4 | `assets/actors/*.png` |

Late-story NPCs currently added through imagegen:
`ai_plague_elder`, `ai_shrine_keeper`, `ai_capital_envoy`,
`ai_mansion_spy`, `ai_spirit_guide`, `ai_tribal_chief`, and
`ai_final_oracle`.

## In-Game Use

- `src/game/explore.rs` loads generated tiles, props, NPC cutouts, lighting, and
  fog-visible map content across the chapter maps. Main quest NPCs from Plague
  Village onward use distinct generated cutouts instead of generic paperdoll
  reuse.
- `src/game/battle.rs` uses generated creature cutouts as the primary enemy
  bodies, layers actor sprite sheets as translucent idle/attack aura motion, and
  builds zone-specific battle backdrops from generated map tiles.
- `src/game/quest.rs` drives the first quest chain through generated and
  paperdoll NPCs.

## Font

| Asset | Path | Source | Notes |
|-------|------|--------|-------|
| CJK font | `assets/fonts/unifont.otf` | GNU Unifont 17.0.04 | Covers Chinese UI/dialogue text. |
