#!/usr/bin/env python3
"""Summarize production-system reward sweeps; stdlib only."""
import csv
from collections import defaultdict
from pathlib import Path
import sys

root = Path(sys.argv[1])
sections = []
for filename, title, scope in [
    ("reward-combat.csv", "Chapter story builds", "Rested boss encounters, cumulative primary story choices, fixed starter/loot. Heart, resolve, and mixed each face the same seeds."),
    ("side-combat.csv", "Side-story gifts", "60% HP/MP, fixed starter/loot, completed real side-chain choices. Qin gifts against the thunder boss are a controlled comparison, not an early in-game unlock."),
]:
    path = root / filename
    groups = defaultdict(list)
    rows = list(csv.DictReader(path.open()))
    for row in rows:
        label = row.get("build", row.get("route", ""))
        groups[(row["chapter"], row["boss"], label)].append(row)
    sections += [f"## {title}", "", scope, "", "| Chapter / Boss | Choices | Wins | Mean ending HP | Mean rounds | Timeouts |", "| --- | --- | ---: | ---: | ---: | ---: |"]
    for (chapter, boss, label), samples in sorted(groups.items(), key=lambda item: (int(item[0][0]), item[0][2])):
        wins = sum(r["won"] == "true" for r in samples)
        timeouts = sum(r["timeout"] == "true" for r in samples)
        hp = sum(int(r["hp"]) for r in samples) / len(samples)
        rounds = sum(int(r["rounds"]) for r in samples) / len(samples)
        sections.append(f"| {chapter} · {boss} | {label} | {wins}/{len(samples)} | {hp:.1f} | {rounds:.1f} | {timeouts} |")
    sections += ["", f"{len(rows)} real-system fights. Ending HP includes losses. These comparisons do not establish all-build or human-play balance.", ""]
(root / "reward-comparison.md").write_text("# Narrative reward comparison\n\n" + "\n".join(sections))
