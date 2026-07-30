//! Marker kinds for the map stages (formerly the abstract node graph).

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NodeKind {
    Fight,  // 遭遇
    Elite,  // 强袭
    Event,  // 奇遇
    Story,  // 情缘 / 剧情
    Rest,   // 歇脚
    Market, // 集市
    Puzzle, // 机关(晶石/河灯/瘴铃/镜/雷鼓/梦灯;主线必清)
    Boss,   // 首领
    Chest,  // 宝箱(可见、可选;开出财物/法宝,也可能是宝箱妖)
    Spring, // 灵泉(可见、可选;一次性回血)
    Guide,  // 路人签(可见、可选;接取本程小委托并回执)
}

impl NodeKind {
    /// One-glyph CJK marker rendered on the map (unifont-safe).
    pub fn glyph(self) -> &'static str {
        match self {
            NodeKind::Fight => "战",
            NodeKind::Elite => "袭",
            NodeKind::Event => "遇",
            NodeKind::Story => "缘",
            NodeKind::Rest => "歇",
            NodeKind::Market => "市",
            NodeKind::Puzzle => "机",
            NodeKind::Boss => "魔",
            NodeKind::Chest => "宝",
            NodeKind::Spring => "泉",
            NodeKind::Guide => "人",
        }
    }

    /// 可选目标(宝箱/灵泉/路人)不计入「清完标记开门」的判定。
    pub fn optional(self) -> bool {
        matches!(self, NodeKind::Chest | NodeKind::Spring | NodeKind::Guide)
    }

    pub fn label(self) -> &'static str {
        match self {
            NodeKind::Fight => "遭遇战",
            NodeKind::Elite => "精英强袭",
            NodeKind::Event => "奇遇",
            NodeKind::Story => "剧情",
            NodeKind::Rest => "歇脚",
            NodeKind::Market => "集市",
            NodeKind::Puzzle => "机关",
            NodeKind::Boss => "章末首领",
            NodeKind::Chest => "宝箱",
            NodeKind::Spring => "灵泉",
            NodeKind::Guide => "路人委托",
        }
    }
}
