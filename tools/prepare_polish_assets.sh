#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
# Generated reference is an exact 2x4 atlas; normalize to even cell boundaries.
magick assets/generated/imagegen/raw/battle-environments.png -resize '2048x2304!' -crop 1024x576 +repage assets/backgrounds/battle-%d.png
names=(village bamboo cave river plague capital southern dream)
for index in "${!names[@]}"; do
  mv "assets/backgrounds/battle-$index.png" "assets/backgrounds/battle-${names[$index]}.png"
done
# Keep the approved idle/attack rows and use the articulated eight-frame gait.
magick \
  \( assets/generated/imagegen/raw/hero-motion-source.png -crop 1024x512+0+0 +repage \) \
  \( assets/generated/imagegen/raw/hero-walk-source.png -resize '1024x512!' \) \
  \( assets/generated/imagegen/raw/hero-motion-source.png -crop 1024x512+0+1024 +repage \) \
  -append assets/actors/hero-motion.png
