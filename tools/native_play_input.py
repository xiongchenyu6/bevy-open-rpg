#!/usr/bin/env python3
"""Native input in an owned Wayland compositor; never accesses game state.

Run inside nix develop. For repeated interactive control, keep the C keyboard
controller alive and write commands to it directly instead of compiling per call.
"""
import argparse
import json
import os
from pathlib import Path
import subprocess
import time

parser = argparse.ArgumentParser()
parser.add_argument("--display", required=True)
parser.add_argument("--key", required=True, choices=("W", "A", "S", "D", "w", "a", "s", "d", "Space", "space", "Return", "Escape", "Up", "Down", "Left", "Right"))
parser.add_argument("--hold", type=int, default=200)
parser.add_argument("--repeat", type=int, default=1)
parser.add_argument("--shot")
parser.add_argument("--log", default="screenshots/result/28/native-actions.jsonl")
args = parser.parse_args()
env = {**os.environ, "WAYLAND_DISPLAY": args.display}
controller = Path(__file__).with_name("native_wayland_keyboard.sh")
for _ in range(args.repeat):
    subprocess.run(["bash", str(controller)], input=f"hold {args.key} {max(args.hold, 150)}\nquit\n",
                   text=True, env=env, check=True, capture_output=True)
    with open(args.log, "a") as log:
        log.write(json.dumps({"at": time.time(), "display": args.display, "key": args.key,
                              "hold_ms": max(args.hold, 150), "source": "native fixed-US Wayland keyboard"}) + "\n")
if args.shot:
    subprocess.run(["grim", args.shot], env=env, check=True)
