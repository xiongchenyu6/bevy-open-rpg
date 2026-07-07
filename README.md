# 御剑行 · 轮回 (Love RPG)

仙侠风 **肉鸽回合制 RPG**,用 Bevy 0.19 代码优先构建,美术全部由
ComfyUI(Flux 静帧 + Wan2.2 过场动画)生成。

**▶ 在浏览器游玩:<https://xiongchenyu6.github.io/bevy-open-rpg/>**(需要
WebGPU:最新 Chrome / Edge,或 Firefox 开启 `dom.webgpu.enabled`)

![章节开卷](docs/preview.png)
![Boss 真身战](docs/preview_battle.png)

## 玩法

一局一世,全程走在真实可行走的程序化地图上:

- **地图即世界**:元胞自动机生成有机地形,散布未知「?」迷雾标记——
  走上去才揭晓是战斗、奇遇、剧情、歇脚还是集市;宝箱可能是宝箱妖,
  灵泉回血,瘴气毒格可见可绕。清完标记走青石界门进下一程。
- **读招回合制**:敌人每回合预告下一手(张爪/蓄力重击/凝气/摄灵),
  「御守」卸力回灵、气势满三层放「绝技·剑气爆发」——每回合都是决策。
- **六位章末 Boss**:战前对峙台词,半血「真身」变身,各有专属机制
  (镜界反噬、雷劫连环、根须再生……);终章宿命水影会读你这一世的
  道心与情缘,决定以情乱心还是以剑势压人。
- **无等级成长**:22 件法宝按持有叠加;道心/情缘里程碑直接反哺战斗;
  章末境界突破。
- **有记忆的剧情**:29 个奇遇(含白狐三遇、盲女琴师二遇等跨图事件链,
  初遇的选择决定后文),11 场「缘」剧情,四种结局 + 按本局抉择生成的
  结局回响。

## 操作

- 方向键/WASD 移动 · 走到「?」揭晓 · ESC 行囊(纸娃娃/法宝/里程碑)
- 战斗:上/下选指令,空格确认;对话:空格推进,上/下选选项

## 本地运行

需要 Rust 1.95+(NixOS 用户直接 `nix develop` / direnv):

```bash
cargo run --bin love-rpg          # 桌面(Wayland)
bash scripts/build_web.sh         # 构建 wasm 到 web/
python3 scripts/serve_web.py      # 本地预览 web 构建
```

## 架构与美术管线

- `STRUCTURE.md` — 模块架构;`PLAN.md` — 十三轮迭代记录;
  `ASSETS.md` — 生成式美术管线(远程 ComfyUI:Flux 静帧、地砖、
  立绘、UI 框,Wan2.2 i2v 章节过场动画)。
- CI(`.github/workflows/deploy-wasm-pages.yml`)在每次 push main 时
  构建 wasm → 发布 `web-latest` Release → 部署 GitHub Pages。

姊妹项目:[protect-carrot](https://github.com/xiongchenyu6/protect-carrot) ·
[bevy-open-rts](https://github.com/xiongchenyu6/bevy-open-rts)
