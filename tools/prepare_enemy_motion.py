#!/usr/bin/env python3
"""Reproduce the accepted serpent atlas from saved Wan source frames.

Run from repository root:
  OMP_NUM_THREADS=4 uv run --python 3.11 --with 'rembg[cpu]' --with pillow \
    --with numpy python tools/prepare_enemy_motion.py

Uses the cached u2net semantic segmentation model (rembg downloads it when
absent). Requires assets/generated/comfy/raw/serpent-motion/wan-{action}/NNN.png.
The output overwrites the atlas, selected frames, contact sheet, and manifest;
do not run during a build or capture using the accepted assets.
"""
from pathlib import Path
import hashlib
import json

import numpy as np
from PIL import Image, ImageDraw
from rembg import new_session, remove

ROOT = Path(__file__).resolve().parents[1]
SELECTION = {
    "idle": [16, 22, 28, 36, 42, 34, 26, 18],
    "attack": [0, 3, 5, 9, 17, 28, 35, 48],
    "hit": [0, 6, 12, 18, 24, 32, 40, 48],
}


def main():
    session = new_session("u2net")
    frame_dir = ROOT / "assets/generated/comfy/serpent-motion-frames"
    frame_dir.mkdir(parents=True, exist_ok=True)
    atlas = Image.new("RGBA", (3072, 1152))
    contact = Image.new("RGB", (1536, 648), "#273b40")
    draw = ImageDraw.Draw(contact)
    records = []
    for row, (action, indices) in enumerate(SELECTION.items()):
        for column, source_index in enumerate(indices):
            source = Path(
                f"assets/generated/comfy/raw/serpent-motion/wan-{action}/{source_index:03}.png"
            )
            image = Image.open(ROOT / source).convert("RGB")
            # Preserve the two resizing passes used for the accepted artifact.
            cutout = remove(image, session=session).convert("RGBA")
            cutout = cutout.resize((384, 384), Image.Resampling.LANCZOS)
            cutout = cutout.resize((360, 360), Image.Resampling.LANCZOS)
            frame = Image.new("RGBA", (384, 384))
            frame.alpha_composite(cutout, (12, 12))
            pixels = np.array(frame)
            rgb = pixels[:, :, :3]
            # u2net keeps enclosed source-background voids between coils.
            # Only neutral near-white is removed; blue-tinted belly stays.
            background = (rgb.min(axis=2) > 242) & (
                (rgb.max(axis=2) - rgb.min(axis=2)) < 12
            )
            pixels[background, 3] = 0
            frame = Image.fromarray(pixels)
            path = Path(f"assets/generated/comfy/serpent-motion-frames/{action}-{column:02}.png")
            frame.save(ROOT / path)
            alpha = frame.getchannel("A")
            bbox = alpha.getbbox()
            coverage = float(pixels[:, :, 3].mean() / 255)
            if not bbox or not (0 < bbox[0] < bbox[2] < 384 and 0 < bbox[1] < bbox[3] < 384):
                raise ValueError(f"Clipped or empty frame: {path}, bbox={bbox}")
            if not 0.06 < coverage < 0.85:
                raise ValueError(f"Invalid alpha coverage: {path}, {coverage}")
            records.append({
                "action": action, "column": column, "index": row * 8 + column,
                "source": str(source), "path": str(path), "size": [384, 384],
                "alpha_extrema": alpha.getextrema(), "alpha_bbox": bbox,
                "transparent_pixels": int((pixels[:, :, 3] == 0).sum()),
                "alpha_coverage": coverage,
                "sha256": hashlib.sha256((ROOT / path).read_bytes()).hexdigest(),
            })
            atlas.alpha_composite(frame, (column * 384, row * 384))
            thumbnail = frame.resize((192, 192))
            contact.paste(thumbnail, (column * 192, row * 216), thumbnail)
            draw.text((column * 192 + 3, row * 216 + 194), f"{action} {column}", fill="white")
            print(f"{action} {column}: source {source_index}, coverage {coverage:.3f}", flush=True)
    if len({record["sha256"] for record in records}) != 24:
        raise ValueError("Atlas must contain 24 unique frames")
    destination = "assets/actors/water-serpent-motion.png"
    (ROOT / destination).parent.mkdir(parents=True, exist_ok=True)
    atlas.save(ROOT / destination)
    contact_path = ROOT / "screenshots/service-check/serpent-motion-final-contact.png"
    contact_path.parent.mkdir(parents=True, exist_ok=True)
    contact.save(contact_path)
    manifest = {
        "path": destination, "cell": [384, 384], "grid": [8, 3],
        "size": [3072, 1152], "row_order": list(SELECTION),
        "attack_peak_column": 4,
        "source_workflow": "assets/generated/comfy/serpent-motion-wan.json",
        "selection": SELECTION,
        "processing": "Semantic u2net rembg; uniform512-to384-to360 resizing,12px padding; neutralwhite RGB min>242 spread<12 removal. No per-frame trim or translation.",
        "frames": records,
        "validation": {
            "all24_unique": True, "all24_alpha_bbox_strictly_inside": True,
            "coverage_range": [min(r["alpha_coverage"] for r in records), max(r["alpha_coverage"] for r in records)],
        },
    }
    (ROOT / "assets/generated/comfy/serpent-motion-atlas.json").write_text(
        json.dumps(manifest, indent=2) + "\n"
    )
    print(f"Atlas SHA256: {hashlib.sha256((ROOT / destination).read_bytes()).hexdigest()}")


if __name__ == "__main__":
    main()
