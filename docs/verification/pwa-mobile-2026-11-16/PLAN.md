# RPG rollout: on-demand content and mobile play

- [x] Startup shell / install guidance / single-language pre-load controls.
- [x] SHA256 cache, explicit update, rollback, immutable old-release resources.
- [x] Core + shared-world + chapter2–7 cinematic groups; legacy chapter1 art optional; exclude source-generation files.
- [x] Engine input fingerprint reuses exact published engine for content-only updates.
- [x] Real Rust asset read gate and chapter transition preparation; no timer bypass.
- [x] Touch movement and existing command/dialogue controls; safe area and stable viewport.
- [x] Required Rust checks, web build, real browser desktop/mobile, cold offline and update tests.
- [x] Published gameplay acceptance at `f603514`; public receipts and screenshots retained.

Keep existing .agents directory untouched. Preserve Bevy 0.19.1 and native behavior.

## Verified so far

- Native tests: 276 passed, 6 pre-existing ignored (includes asset gate and compact dialogue checks).
- Native and wasm32 checks pass (15 baseline gameplay dead-code warnings).
- Python: 14 manifest/retention/source-fingerprint/engine-reuse scenarios pass.
- Real Chromium SW: 10 integrity/update/offline scenarios; 12 demand/pinning/cancel/update scenarios.
- Production launcher with isolated tiny engine: 5 scenarios; language/install screenshots reviewed at 844×390 and 390×844.
- Final native check/build and release WASM build pass. Actual compiled WebGPU gameplay, cold offline restart, joystick release, inventory scrolling and both orientations pass. Landscape objective panel contains every line. Published desktop/mobile/cold-offline acceptance also passed at `f603514` with zero browser errors.

## Actual runtime review

- Actual WASM title and chapter1 render under headed Chromium156 on isolated Xvfb,
  Vulkan/SwiftShader. Headless ANGLE SwiftShader loses its GPU device on this host;
  it is not accepted as rendering proof and readiness was not weakened to bypass it.
- Network receipt proves title only loads core, first chapter only shared world.
  Fresh browser process with networking disabled enters the installed chapter.
- Mobile joystick sets/releases input and leaves the canvas bounds unchanged.
- Screenshot review found tiny inventory typography and overlapping touch controls;
  the responsive inventory pass now passes visual review in both orientations, including finger scrolling. The landscape objective panel receives extra height to contain its final instruction line.

## Release contract and limits

- First title download is about 22.2 MiB (gzip engine + fonts/title/shell).
  First chapter separately asks for 52.6 MiB shared world assets; chapters 2–7
  request only their 8–13 MiB cinematic packs on entry. Already cached hashes
  cost zero resource download bytes. No full-game precache on install.
- Content-only updates reuse the exact published engine when its input fingerprint
  matches. Cache regression receipts prove a one-file update does not fetch
  unchanged files, and a bad hash never replaces the active release.
- Launcher language changes before engine download and survives reload.
  The authored story/combat UI remains Simplified Chinese; the English settings
  explicitly disclose this instead of implying a full translation.
- Download/settings dialogs pause gameplay. Touch controls are contextual;
  inventory hides movement/interact, supports drag scrolling, and retains Back.
- Real browser validation uses Chromium156 with touch emulation and software
  Vulkan on an isolated display. Android/iOS hardware installation and the full
  seven-chapter playthrough are not covered by these checks. WebGPU is required.
- The existing main → release ZIP → gh-pages workflow is retained. Deployment
  is accepted only after public build.json and asset-manifest.json name the
  exact pushed source revision.

## Reproducible evidence

- `runtime-result.json` identifies the tested manifest and compiled-source hash;
  `runtime-requests.json` records the real engine network requests.
- `runtime.log` contains the actual engine assertions. Cache/demand JSON receipts
  cover corrupt resources, rejected updates, old-client pinning and one-file deltas.
- Reviewed desktop and mobile screenshots are stored beside this file.
- Local final capture: `screenshots/result/32/video.mp4` with 900 PNG source frames,
  30 seconds at30fps, showing the offline mobile inventory and finger scrolling. Sampled frames reviewed before publication.
  Browser viewport recording is sampled at30fps without changing elapsed time;
  repeated frames preserve the original capture timing. This is not a GPU FPS benchmark.
  Full X11 capture attempt30 was rejected because its compositor viewport was stale;
  page screenshots and the browser viewport recording show the actual rendered content.
- Physical Android/iOS testing remains outstanding. No hardware support claim is inferred
  from emulation. Existing15 gameplay dead-code warnings remain unchanged.

## Published acceptance

`public/result.json` and `public/receipt.json` identify release `f603514` and verified hosted engine/JS/group-sample hashes. The public actual-game harness passed desktop chapter entry, full browser restart offline, mobile movement/release, finger scrolling and both inventory orientations with no browser errors. Public screenshots were visually reviewed. Workflow: https://github.com/xiongchenyu6/bevy-open-rpg/actions/runs/43880741236 .

A subsequent UI-only correction allows immediate retry while canceled preparation settles. Its deterministic regression holds both preparations independently, proving an old completion cannot unlock or complete its successor, including between-chapter pause/retry. It does not change compiled engine inputs. Hardware Android/iOS remains untested.
