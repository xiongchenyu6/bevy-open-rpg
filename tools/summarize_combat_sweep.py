#!/usr/bin/env python3
"""Summarize real-system sweep CSV: python tools/summarize_combat_sweep.py in.csv out.md."""
import csv
import statistics
import sys
from collections import defaultdict
from pathlib import Path
rows=list(csv.DictReader(Path(sys.argv[1]).open()))
groups=defaultdict(list)
for row in rows:groups[row['enemy'],row['policy']].append(row)
policies=['attack','spell-burst','warning-aware']
text=['# Deterministic combat policy sweep','',f'{len(rows)} fights; {len(set(r["seed"] for r in rows))} fixed seeds per enemy/policy. Production input, turn, scaling and skill systems; full-health entry, fixed four-spell kit, chapter-seal stat growth, no random relics/hexes. This compares policies, not human difficulty or full-run balance.','','Each cell: wins / trials · mean rounds · mean remaining HP.','','| Enemy | Attack only | Spell/burst first | Warning-aware |','|---|---|---|---|']
for enemy in dict.fromkeys(row['enemy'] for row in rows):
    vals=[]
    for policy in policies:
        data=groups[enemy,policy]
        vals.append(f'{sum(r["won"]=="true" for r in data)}/{len(data)} · {statistics.mean(int(r["rounds"]) for r in data):.1f} · {statistics.mean(int(r["hp"]) for r in data):.1f}')
    text.append('| '+enemy+' | '+' | '.join(vals)+' |')
text.extend(['',f'Timeouts: {sum(r["timeout"]=="true" for r in rows)}.',''])
Path(sys.argv[2]).write_text('\n'.join(text))
print('\n'.join(text))
