#!/usr/bin/env node
import { execFileSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";

const BASE = process.env.COMFY_BASE || "http://127.0.0.1:18188";
const RAW_DIR = "assets/generated/comfy/raw";
const MANIFEST = "assets/generated/comfy/manifest.json";
const STEPS = Number(process.env.COMFY_STEPS || 24);
const GUIDANCE = Number(process.env.COMFY_GUIDANCE || 3.8);
const SEED_OFFSET = Number(process.env.COMFY_SEED_OFFSET || 0);
const ONLY_IDS = new Set(
  String(process.env.COMFY_ONLY || "")
    .split(",")
    .map((id) => id.trim())
    .filter(Boolean),
);

const AVOID =
  "No text, no watermark, no logo, no decorative border, not cropped, single subject only, sharp and high quality.";

const ASSETS = [
  {
    id: "ai_sword_sister",
    kind: "cutout",
    out: "assets/npcs/ai_sword_sister.png",
    size: 512,
    prompt:
      "premium 2D game character cutout, full body young xianxia sword cultivator heroine, long black hair in a high warrior ponytail, layered crimson and ivory travel robes with restrained silver embroidery, holding a slender jade-hilt sword in a ready low guard, alert resolute expression, classic Chinese fantasy RPG painted sprite, front three-quarter view, feet fully visible, centered, clean readable silhouette, crisp edges, no shadow, perfectly flat pure #00ff00 green screen background",
  },
  {
    id: "ai_linger_companion",
    kind: "cutout",
    out: "assets/npcs/ai_linger_companion.png",
    size: 512,
    prompt:
      "premium 2D game character cutout, full body young xianxia healer heroine, long black hair tied with a large white ribbon, flowing moon-white and pale blue hanfu travel robes, holding a small golden spirit bell, warm gentle determined expression, classic Chinese fantasy RPG painted sprite, front three-quarter view, feet fully visible, centered, clean readable silhouette, crisp edges, no shadow, perfectly flat pure #00ff00 green screen background",
  },
  {
    id: "ai_nanyao_companion",
    kind: "cutout",
    out: "assets/npcs/ai_nanyao_companion.png",
    size: 512,
    prompt:
      "premium 2D game character cutout, full body young Baiyue spirit witch heroine, long dark braided hair with silver ornaments, layered indigo teal and white ceremonial travel robe, jade serpent motifs, holding a carved spirit staff and bronze bell, poised protective expression, classic Chinese fantasy RPG painted sprite, front three-quarter view, feet fully visible, centered, clean readable silhouette, crisp edges, no shadow, no magenta anywhere on character, perfectly flat pure #ff00ff magenta screen background",
  },
  {
    id: "ai_herb_healer",
    kind: "cutout",
    out: "assets/npcs/ai_herb_healer.png",
    size: 192,
    prompt:
      "crisp 2D game asset, full body young xianxia herbalist healer, medicine gourd, cloth satchel, pale robe with gold trim, anime RPG sprite illustration, centered, clean silhouette, no shadow, perfectly flat pure #00ff00 green screen background",
  },
  {
    id: "ai_mountain_monk",
    kind: "cutout",
    out: "assets/npcs/ai_mountain_monk.png",
    size: 192,
    prompt:
      "crisp 2D game asset, full body old mountain monk hermit, prayer beads, bamboo staff, weathered robe, xianxia RPG sprite illustration, centered, sharp silhouette, no shadow, perfectly flat pure #00ff00 green screen background",
  },
  {
    id: "ai_fox_spirit",
    kind: "cutout",
    out: "assets/npcs/ai_fox_spirit.png",
    size: 192,
    prompt:
      "crisp 2D game asset, full body fox spirit girl NPC, white fox ears, fluffy tails, moonlit blue robe, anime RPG sprite illustration, centered, sharp silhouette, no shadow, perfectly flat pure #00ff00 green screen background",
  },
  {
    id: "ai_fire_wisp",
    kind: "cutout",
    out: "assets/creatures/ai_fire_wisp.png",
    size: 256,
    prompt:
      "crisp 2D game enemy asset, floating fire wisp creature, orange flame core, ember sparks, readable silhouette, centered, no shadow, perfectly flat pure #00ff00 green screen background",
  },
  {
    id: "ai_stone_guardian",
    kind: "cutout",
    out: "assets/creatures/ai_stone_guardian.png",
    size: 256,
    prompt:
      "crisp 2D game enemy asset, squat stone guardian monster, mossy carved rocks, glowing eyes, fantasy RPG creature, centered, readable silhouette, no shadow, perfectly flat pure #00ff00 green screen background",
  },
  {
    id: "ai_water_serpent",
    kind: "cutout",
    out: "assets/creatures/ai_water_serpent.png",
    size: 256,
    prompt:
      "crisp 2D game enemy asset, blue water serpent spirit, curling body, fins and pearl glow, fantasy RPG creature, centered, readable silhouette, no shadow, perfectly flat pure #00ff00 green screen background",
  },
  {
    id: "ai_cave_bat",
    kind: "cutout",
    out: "assets/creatures/ai_cave_bat.png",
    size: 256,
    prompt:
      "crisp 2D game enemy asset, shadow cave bat demon, spread wings, purple eyes, dark fantasy RPG creature, centered, readable silhouette, no shadow, perfectly flat pure #00ff00 green screen background",
  },
  {
    id: "ai_bamboo_path",
    kind: "tile",
    out: "assets/tiles/ai_bamboo_path.png",
    prompt:
      "top-down seamless 2D RPG map tile, bamboo forest dirt path with scattered leaves, hand painted game texture, no characters, no text, tileable square",
  },
  {
    id: "ai_cave_floor",
    kind: "tile",
    out: "assets/tiles/ai_cave_floor.png",
    prompt:
      "top-down seamless 2D RPG map tile, wet cave stone floor with subtle blue mineral veins, hand painted game texture, no characters, no text, tileable square",
  },
  {
    id: "ai_shrine_floor",
    kind: "tile",
    out: "assets/tiles/ai_shrine_floor.png",
    prompt:
      "top-down seamless 2D RPG map tile, ancient shrine stone floor with faint gold rune cracks, hand painted game texture, no characters, no text, tileable square",
  },
  {
    id: "ai_mystic_grass",
    kind: "tile",
    out: "assets/tiles/ai_mystic_grass.png",
    prompt:
      "top-down seamless 2D RPG map tile, mystic night grass with tiny blue spirit flowers, hand painted game texture, no characters, no text, tileable square",
  },
  {
    id: "ai_bamboo_scout",
    kind: "cutout",
    out: "assets/npcs/ai_bamboo_scout.png",
    size: 192,
    prompt:
      "crisp 2D game asset, full body young xianxia bamboo forest scout, green travel cloak, short bow, leaf talisman, anime RPG sprite illustration, centered, clean readable silhouette, sharp edges, no shadow, perfectly flat pure #00ff00 green screen background",
  },
  {
    id: "ai_wandering_merchant",
    kind: "cutout",
    out: "assets/npcs/ai_wandering_merchant.png",
    size: 192,
    prompt:
      "crisp 2D game asset, full body cheerful xianxia wandering merchant, pack of scrolls and medicine bottles, warm layered robe, anime RPG sprite illustration, centered, clean silhouette, no shadow, perfectly flat pure #00ff00 green screen background",
  },
  {
    id: "ai_cave_priestess",
    kind: "cutout",
    out: "assets/npcs/ai_cave_priestess.png",
    size: 192,
    prompt:
      "crisp 2D game asset, full body moon cave priestess NPC, silver blue ritual robe, crescent staff, calm expression, anime RPG sprite illustration, centered, clean silhouette, no shadow, perfectly flat pure #00ff00 green screen background",
  },
  {
    id: "ai_plague_elder",
    kind: "cutout",
    out: "assets/npcs/ai_plague_elder.png",
    size: 192,
    prompt:
      "crisp 2D game asset, full body plague village elder NPC, elderly xianxia village chief with gray beard, moss green rain cloak, wooden cane, medicine tally slips, dignified and weary, anime RPG sprite illustration, centered, clean silhouette, no shadow, perfectly flat pure #00ff00 green screen background",
  },
  {
    id: "ai_shrine_keeper",
    kind: "cutout",
    out: "assets/npcs/ai_shrine_keeper.png",
    size: 192,
    prompt:
      "crisp 2D game asset, full body plague shrine keeper NPC, solemn ritual bell priest with pale robe, bronze cleansing bell, ash pouch, black green talismans, anime RPG sprite illustration, centered, clean silhouette, no shadow, perfectly flat pure #00ff00 green screen background",
  },
  {
    id: "ai_capital_envoy",
    kind: "cutout",
    out: "assets/npcs/ai_capital_envoy.png",
    size: 192,
    prompt:
      "crisp 2D game asset, full body capital envoy NPC, refined xianxia official investigator in blue and ivory robes, jade city writ scroll, small sword, anime RPG sprite illustration, centered, clean silhouette, no shadow, perfectly flat pure #00ff00 green screen background",
  },
  {
    id: "ai_mansion_spy",
    kind: "cutout",
    out: "assets/npcs/ai_mansion_spy.png",
    size: 192,
    prompt:
      "crisp 2D game asset, full body mansion spy NPC, secret informant in dark violet robes with face veil, cipher slip, mirror shard talisman, anime RPG sprite illustration, centered, clean silhouette, no shadow, perfectly flat pure #00ff00 green screen background",
  },
  {
    id: "ai_spirit_guide",
    kind: "cutout",
    out: "assets/npcs/ai_spirit_guide.png",
    size: 192,
    prompt:
      "crisp 2D game asset, full body southern spirit road guide NPC, lean traveler with feathered cloak, bronze compass charm, wind talismans, bamboo staff, anime RPG sprite illustration, centered, clean silhouette, no shadow, perfectly flat pure #00ff00 green screen background",
  },
  {
    id: "ai_tribal_chief",
    kind: "cutout",
    out: "assets/npcs/ai_tribal_chief.png",
    size: 192,
    prompt:
      "crisp 2D game asset, full body Baiyue tribal chief NPC, thunder totem leader with indigo bronze ceremonial armor, braided hair, carved staff, protective stern expression, anime RPG sprite illustration, centered, clean silhouette, no shadow, perfectly flat pure #00ff00 green screen background",
  },
  {
    id: "ai_final_oracle",
    kind: "cutout",
    out: "assets/npcs/ai_final_oracle.png",
    size: 192,
    prompt:
      "crisp 2D game asset, full body final oracle NPC, spirit abyss lamp keeper in flowing black white blue robes, memory pearl lantern, water ribbon motifs, serene otherworldly expression, anime RPG sprite illustration, centered, clean silhouette, no shadow, perfectly flat pure #00ff00 green screen background",
  },
  {
    id: "ai_thunder_roc",
    kind: "cutout",
    out: "assets/creatures/ai_thunder_roc.png",
    size: 256,
    prompt:
      "crisp 2D game enemy asset, thunder roc bird spirit, jagged gold feathers, blue lightning aura, fantasy RPG creature, centered, readable silhouette, no shadow, perfectly flat pure #00ff00 green screen background",
  },
  {
    id: "ai_blood_mantis",
    kind: "cutout",
    out: "assets/creatures/ai_blood_mantis.png",
    size: 256,
    prompt:
      "crisp 2D game enemy asset, crimson blade mantis demon, sharp forearms, dark shell, fantasy RPG creature, centered, readable silhouette, no shadow, perfectly flat pure #00ff00 green screen background",
  },
  {
    id: "ai_shadow_swordsman",
    kind: "cutout",
    out: "assets/creatures/ai_shadow_swordsman.png",
    size: 256,
    prompt:
      "crisp 2D game enemy asset, ghostly shadow swordsman demon, torn black robes, pale mask, cursed sword glow, fantasy RPG creature, centered, readable silhouette, no shadow, perfectly flat pure #00ff00 green screen background",
  },
  {
    id: "ai_moss_turtle",
    kind: "cutout",
    out: "assets/creatures/ai_moss_turtle.png",
    size: 256,
    prompt:
      "crisp 2D game enemy asset, ancient moss turtle guardian, stone shell with tiny plants, glowing jade eyes, fantasy RPG creature, centered, readable silhouette, no shadow, perfectly flat pure #00ff00 green screen background",
  },
  {
    id: "ai_lotus_spirit",
    kind: "cutout",
    out: "assets/creatures/ai_lotus_spirit.png",
    size: 256,
    prompt:
      "crisp 2D game enemy asset, floating lotus spirit, pink petals, pale blue core, ribbon-like water wisps, fantasy RPG creature, centered, readable silhouette, no shadow, perfectly flat pure #00ff00 green screen background",
  },
  {
    id: "boss_mountain_fiend",
    kind: "cutout",
    out: "assets/creatures/boss_mountain_fiend.png",
    size: 512,
    prompt:
      "premium 2D game boss cutout, hulking red-haired mountain ogre demon, charred horned mask, iron club wrapped in prayer rope, scorched hide armor and ember cracks, menacing full body pose, classic Chinese fantasy RPG hand-painted monster, front three-quarter view, centered, dramatic readable silhouette, crisp edges, no floor, no shadow, perfectly flat pure #00ff00 green screen background",
  },
  {
    id: "boss_moon_wraith",
    kind: "cutout",
    out: "assets/creatures/boss_moon_wraith.png",
    size: 512,
    prompt:
      "premium 2D game boss cutout, spectral moon wraith queen, pale female spirit rising from a coiling violet water-serpent tail, torn lunar robes, crescent crown, long black hair floating in ghost wind, cold glowing eyes, classic Chinese fantasy RPG hand-painted monster, full body front three-quarter view, centered, dramatic readable silhouette, crisp edges, no floor, no shadow, perfectly flat pure #00ff00 green screen background",
  },
  {
    id: "boss_river_demon",
    kind: "cutout",
    out: "assets/creatures/boss_river_demon.png",
    size: 512,
    prompt:
      "premium 2D game boss cutout, black-scaled river flood dragon demon with long serpentine body, antler horns, torn river lanterns caught on fins, blue water streaming from claws, fierce full body coiled pose, classic Chinese fantasy RPG hand-painted monster, front three-quarter view, centered, dramatic readable silhouette, crisp edges, no floor, no shadow, perfectly flat pure #00ff00 green screen background",
  },
  {
    id: "boss_miasma_root",
    kind: "cutout",
    out: "assets/creatures/boss_miasma_root.png",
    size: 512,
    prompt:
      "premium 2D game boss cutout, enormous plague banyan root demon, twisted blood-red and charcoal roots forming a snarling mask, hanging medicine talismans, poisonous violet spores and grasping root claws, classic Chinese fantasy RPG hand-painted monster, full body front view, centered, dramatic readable silhouette, crisp edges, no floor, no shadow, perfectly flat pure #00ff00 green screen background",
  },
  {
    id: "boss_mirror_minister",
    kind: "cutout",
    out: "assets/creatures/boss_mirror_minister.png",
    size: 512,
    prompt:
      "premium 2D game boss cutout, sinister imperial mirror sorcerer, tall masked minister in black violet and tarnished gold ceremonial robes, one hand casting with floating cracked bronze mirrors, hidden sword at waist, elegant threatening full body pose, classic Chinese fantasy RPG hand-painted villain, front three-quarter view, centered, dramatic readable silhouette, crisp edges, no floor, no shadow, perfectly flat pure #00ff00 green screen background",
  },
  {
    id: "boss_thunder_qilin",
    kind: "cutout",
    out: "assets/creatures/boss_thunder_qilin.png",
    size: 512,
    prompt:
      "premium 2D game boss cutout, majestic thunder qilin spirit, blue-black scales, branching gold antlers, white mane lifted by lightning, bronze drum charms around the neck, powerful full body stance, classic Chinese fantasy RPG hand-painted beast, front three-quarter view, centered, dramatic readable silhouette, crisp edges, no floor, no shadow, perfectly flat pure #00ff00 green screen background",
  },
  {
    id: "boss_dream_eclipse",
    kind: "cutout",
    out: "assets/creatures/boss_dream_eclipse.png",
    size: 512,
    prompt:
      "premium 2D game final boss cutout, colossal humanoid water-shadow deity formed from midnight waves, pale moon mask with one sorrowful eye and one wrathful eye, many flowing ribbon arms, broken palace lanterns orbiting the body, tragic and terrifying full silhouette, classic Chinese fantasy RPG hand-painted final monster, front view, centered, dramatic readable silhouette, crisp edges, no floor, no shadow, perfectly flat pure #00ff00 green screen background",
  },
  {
    id: "ai_quest_board",
    kind: "cutout",
    out: "assets/props/ai_quest_board.png",
    size: 160,
    prompt:
      "crisp 2D top-down RPG map prop, wooden village quest board with pinned blank parchment sheets and red cords, no readable text, centered, no shadow, perfectly flat pure #00ff00 green screen background",
  },
  {
    id: "ai_spirit_lantern",
    kind: "cutout",
    out: "assets/props/ai_spirit_lantern.png",
    size: 128,
    prompt:
      "crisp 2D top-down RPG map prop, glowing paper spirit lantern on a short wooden post, warm gold light, centered, no text, no shadow, perfectly flat pure #00ff00 green screen background",
  },
  {
    id: "ai_bamboo_gate",
    kind: "cutout",
    out: "assets/props/ai_bamboo_gate.png",
    size: 192,
    prompt:
      "crisp 2D top-down RPG map prop, small bamboo torii gate with green talismans, forest shrine entrance, centered, no text, no shadow, perfectly flat pure #00ff00 green screen background",
  },
  {
    id: "ai_cave_crystal",
    kind: "cutout",
    out: "assets/props/ai_cave_crystal.png",
    size: 160,
    prompt:
      "crisp 2D top-down RPG map prop, cluster of blue cave crystals, magical glow, centered, no text, no shadow, perfectly flat pure #00ff00 green screen background",
  },
  {
    id: "ai_shrine_statue",
    kind: "cutout",
    out: "assets/props/ai_shrine_statue.png",
    size: 192,
    prompt:
      "crisp 2D top-down RPG map prop, ancient stone fox shrine statue with moss and small offering plate, centered, no text, no shadow, perfectly flat pure #00ff00 green screen background",
  },
  {
    id: "ai_village_moss_path",
    kind: "tile",
    out: "assets/tiles/ai_village_moss_path.png",
    prompt:
      "top-down seamless 2D RPG map tile, old village dirt road with moss between worn stones, hand painted game texture, no characters, no text, tileable square",
  },
  {
    id: "ai_bamboo_thicket",
    kind: "tile",
    out: "assets/tiles/ai_bamboo_thicket.png",
    prompt:
      "top-down seamless 2D RPG map tile, dense bamboo thicket canopy, deep green leaves and stalk shadows, hand painted game texture, no characters, no text, tileable square",
  },
  {
    id: "ai_moon_cave_wall",
    kind: "tile",
    out: "assets/tiles/ai_moon_cave_wall.png",
    prompt:
      "top-down seamless 2D RPG map tile, moonlit cave wall rock with violet minerals and damp highlights, hand painted game texture, no characters, no text, tileable square",
  },
];

const selectedAssets = ONLY_IDS.size ? ASSETS.filter((asset) => ONLY_IDS.has(asset.id)) : ASSETS;
if (ONLY_IDS.size && selectedAssets.length !== ONLY_IDS.size) {
  const known = new Set(selectedAssets.map((asset) => asset.id));
  const missing = [...ONLY_IDS].filter((id) => !known.has(id));
  throw new Error(`Unknown COMFY_ONLY asset ids: ${missing.join(", ")}`);
}
if (!Number.isSafeInteger(SEED_OFFSET)) {
  throw new Error(`COMFY_SEED_OFFSET must be an integer, got ${process.env.COMFY_SEED_OFFSET}`);
}

mkdirSync(RAW_DIR, { recursive: true });
for (const asset of selectedAssets) mkdirSync(dirname(asset.out), { recursive: true });

const manifest = [];
for (const asset of selectedAssets) {
  const index = ASSETS.findIndex((candidate) => candidate.id === asset.id);
  console.log(`\n[${manifest.length + 1}/${selectedAssets.length}] ${asset.id}`);
  const raw = join(RAW_DIR, `${asset.id}.png`);
  if (!existsSync(raw) || process.env.FORCE === "1") {
    await generate(asset, raw, 92024000 + index * 97 + SEED_OFFSET);
  } else {
    console.log(`  reuse ${raw}`);
  }

  if (asset.kind === "cutout") {
    removeBackground(raw, asset.out, asset.size);
  } else {
    execFileSync("magick", [raw, "-resize", "512x512!", asset.out], { stdio: "inherit" });
  }
  manifest.push({ ...asset, raw });
}

let manifestAssets = manifest;
if (ONLY_IDS.size && existsSync(MANIFEST)) {
  const previous = JSON.parse(readFileSync(MANIFEST, "utf8"));
  const merged = new Map((previous.assets || []).map((asset) => [asset.id, asset]));
  for (const asset of manifest) merged.set(asset.id, asset);
  manifestAssets = ASSETS.map((asset) => merged.get(asset.id)).filter(Boolean);
}

mkdirSync(dirname(MANIFEST), { recursive: true });
writeFileSync(
  MANIFEST,
  JSON.stringify(
    {
      base: BASE,
      model: "flux1-dev-fp8.safetensors",
      steps: STEPS,
      guidance: GUIDANCE,
      seedOffset: SEED_OFFSET,
      cutoutMode: process.env.COMFY_CUTOUT_MODE || "rembg",
      generatedAt: new Date().toISOString(),
      assets: manifestAssets,
    },
    null,
    2,
  ),
);
console.log(`\nmanifest -> ${MANIFEST}`);

async function generate(asset, out, seed) {
  const workflow = buildWorkflow(asset.prompt, seed, asset.kind === "tile" ? 512 : 512);
  const response = await fetch(`${BASE}/prompt`, {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ prompt: workflow, client_id: crypto.randomUUID() }),
  });
  const text = await response.text();
  if (!response.ok) throw new Error(text);
  const submitted = JSON.parse(text);
  if (submitted.node_errors && Object.keys(submitted.node_errors).length) {
    throw new Error(JSON.stringify(submitted.node_errors, null, 2));
  }
  process.stdout.write(`  prompt_id=${submitted.prompt_id} `);
  const image = await waitForImage(submitted.prompt_id);
  await downloadImage(image, out);
  console.log(` -> ${out}`);
}

function buildWorkflow(prompt, seed, size) {
  return {
    "1": { class_type: "CheckpointLoaderSimple", inputs: { ckpt_name: "flux1-dev-fp8.safetensors" } },
    "2": {
      class_type: "CLIPTextEncodeFlux",
      inputs: {
        clip: ["1", 1],
        clip_l: `${prompt}. ${AVOID}`,
        t5xxl: `${prompt}. ${AVOID}`,
        guidance: GUIDANCE,
      },
    },
    "3": { class_type: "EmptyLatentImage", inputs: { width: size, height: size, batch_size: 1 } },
    "4": { class_type: "RandomNoise", inputs: { noise_seed: seed } },
    "5": { class_type: "KSamplerSelect", inputs: { sampler_name: "euler" } },
    "6": { class_type: "BasicScheduler", inputs: { model: ["1", 0], scheduler: "simple", steps: STEPS, denoise: 1.0 } },
    "7": { class_type: "BasicGuider", inputs: { model: ["1", 0], conditioning: ["2", 0] } },
    "8": {
      class_type: "SamplerCustomAdvanced",
      inputs: { noise: ["4", 0], guider: ["7", 0], sampler: ["5", 0], sigmas: ["6", 0], latent_image: ["3", 0] },
    },
    "9": { class_type: "VAEDecode", inputs: { samples: ["8", 0], vae: ["1", 2] } },
    "10": { class_type: "SaveImage", inputs: { images: ["9", 0], filename_prefix: `bevy_open_rpg/${Date.now()}` } },
  };
}

async function waitForImage(promptId) {
  for (let i = 0; i < 240; i++) {
    await sleep(2000);
    const history = await fetch(`${BASE}/history/${promptId}`).then((r) => r.json());
    const entry = history[promptId];
    if (!entry) {
      process.stdout.write(".");
      continue;
    }
    const images = Object.values(entry.outputs || {}).flatMap((output) => output.images || []);
    if (images.length) return images[0];
    const status = entry.status?.status_str;
    if (status && status !== "success" && status !== "running") throw new Error(`Comfy status ${status}`);
    process.stdout.write(".");
  }
  throw new Error(`Timed out waiting for ${promptId}`);
}

async function downloadImage(image, out) {
  const url = new URL(`${BASE}/view`);
  url.searchParams.set("filename", image.filename);
  url.searchParams.set("subfolder", image.subfolder || "");
  url.searchParams.set("type", image.type || "output");
  const buffer = Buffer.from(await fetch(url).then((r) => r.arrayBuffer()));
  writeFileSync(out, buffer);
}

function removeBackground(input, output, size) {
  if ((process.env.COMFY_CUTOUT_MODE || "rembg") !== "chroma") {
    execFileSync("bash", ["scripts/cutout_harness.sh", input, output, String(size)], {
      stdio: "inherit",
    });
    return;
  }

  const fuzz = process.env.COMFY_CUTOUT_FUZZ || "8%";
  const edgePoints = [
    [0, 0],
    [128, 0],
    [256, 0],
    [384, 0],
    [511, 0],
    [0, 128],
    [511, 128],
    [0, 256],
    [511, 256],
    [0, 384],
    [511, 384],
    [0, 511],
    [128, 511],
    [256, 511],
    [384, 511],
    [511, 511],
  ];
  const floodFills = edgePoints.flatMap(([x, y]) => ["-draw", `color ${x},${y} floodfill`]);
  execFileSync(
    "magick",
    [
      input,
      "-alpha",
      "on",
      "-fuzz",
      fuzz,
      "-fill",
      "none",
      ...floodFills,
      "-trim",
      "+repage",
      "-resize",
      `${size}x${size}`,
      "-background",
      "none",
      "-gravity",
      "center",
      "-extent",
      `${size}x${size}`,
      output,
    ],
    { stdio: "inherit" },
  );
}

function sleep(ms) {
  return new Promise((resolve) => setTimeout(resolve, ms));
}
