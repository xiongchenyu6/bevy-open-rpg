use bevy::prelude::*;

const BASIC_SPELL_COST: i32 = 5;
const LATE_SPELL_COST: i32 = 9;
const SIDE_QUEST_COUNT: usize = 12;
const CHAPTER_COUNT: usize = 7;
const BOND_SCENE_COUNT: usize = 7;
const CAMP_SCENE_COUNT: usize = 7;
const TREASURE_CACHE_COUNT: usize = 10;
const RIVER_LANTERN_ORDER: [RiverLantern; 3] = [
    RiverLantern::Upstream,
    RiverLantern::Midstream,
    RiverLantern::Dock,
];
const ALL_SIDE_QUESTS: [SideQuest; SIDE_QUEST_COUNT] = [
    SideQuest::VillageTrail,
    SideQuest::VillageHerbs,
    SideQuest::MoonCaveCrystals,
    SideQuest::MoonCaveEchoes,
    SideQuest::RiverLanterns,
    SideQuest::RiverCargo,
    SideQuest::PlagueRelief,
    SideQuest::PlagueMedicine,
    SideQuest::CapitalPatrol,
    SideQuest::CapitalRumors,
    SideQuest::SouthernThunder,
    SideQuest::SouthernDrums,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QuestRole {
    SwordSister,
    Linger,
    StarMage,
    Merchant,
    BambooScout,
    CavePriestess,
    HerbHealer,
    RiverBoatman,
    PlagueElder,
    ShrineKeeper,
    CapitalEnvoy,
    MansionSpy,
    SpiritGuide,
    TribalChief,
    FinalOracle,
}

impl QuestRole {
    pub fn name(self) -> &'static str {
        match self {
            Self::SwordSister => "红衣剑姊",
            Self::Linger => "赵灵儿",
            Self::StarMage => "星咒童子",
            Self::Merchant => "行脚商",
            Self::BambooScout => "竹林斥候",
            Self::CavePriestess => "月洞祭司",
            Self::HerbHealer => "草药医",
            Self::RiverBoatman => "摆渡人",
            Self::PlagueElder => "瘴雨村长",
            Self::ShrineKeeper => "祠祝",
            Self::CapitalEnvoy => "宣令使",
            Self::MansionSpy => "偏院内线",
            Self::SpiritGuide => "引路人",
            Self::TribalChief => "百越族长",
            Self::FinalOracle => "守灯人",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum KeyItem {
    TrackingTalisman,
    RoadPass,
    MoonToken,
    MoonSeal,
    HerbPrescription,
    HerbBundle,
    FerryToken,
    RiverPearl,
    PlagueReport,
    ShrineBell,
    ShrineAsh,
    CureCharm,
    CapitalWrit,
    CipherSlip,
    SecretLetters,
    MirrorSeal,
    SpiritRoadPass,
    TotemCharm,
    StormGlyph,
    QilinHorn,
    FinalGateSigil,
    DreamPearl,
    FateSeal,
    HomecomingSeal,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Companion {
    Linger,
    SwordSister,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BossKind {
    MoonWraith,
    RiverDemon,
    MiasmaRoot,
    MirrorMinister,
    ThunderQilin,
    DreamEclipse,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FinalLamp {
    Memory,
    Vow,
    Fate,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MoonCrystal {
    North,
    South,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RiverLantern {
    Upstream,
    Midstream,
    Dock,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlagueWard {
    OldShrine,
    BitterWell,
    Sickroom,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MansionMirrorNode {
    Ledger,
    Witness,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThunderDrum {
    Wind,
    Cloud,
    Oath,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Chapter {
    VillageOath,
    MoonCave,
    RiverMedicine,
    PlagueRain,
    CapitalMirror,
    SouthernThunder,
    FinalDream,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BondScene {
    VillageFirstNight,
    MoonCavePromise,
    RiverLampWish,
    PlagueRainShelter,
    CapitalRooftop,
    SouthernRoadOath,
    FinalGateQuiet,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct BondReward {
    pub exp: u32,
    pub potions: u32,
    pub full_restore: bool,
}

#[derive(Debug, PartialEq, Eq)]
pub struct BondInteraction {
    pub lines: Vec<String>,
    pub reward: Option<BondReward>,
    pub response_choice: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BondResponse {
    Courage,
    Tender,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BondBonus {
    Courage,
    Tender,
}

impl BondBonus {
    pub fn name(self) -> &'static str {
        match self {
            Self::Courage => "勇心护念",
            Self::Tender => "柔心护念",
        }
    }

    pub fn battle_line(self) -> &'static str {
        match self {
            Self::Courage => "下一场战斗攻击与合击更有冲劲，少量护住前排。",
            Self::Tender => "下一场战斗灵儿治疗更强，也会替队伍挡下一点伤害。",
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CampScene {
    VillageHearth,
    MoonCavePool,
    RiverTownInn,
    PlagueSickroom,
    CapitalSafehouse,
    SouthernCampfire,
    FinalStillWater,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CampBonus {
    Warmth,
    Focus,
    Vigil,
}

#[derive(Debug, PartialEq, Eq)]
pub struct CampInteraction {
    pub lines: Vec<String>,
    pub bonus: Option<CampBonus>,
    pub tactic_choice: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SideQuest {
    VillageTrail,
    VillageHerbs,
    MoonCaveCrystals,
    MoonCaveEchoes,
    RiverLanterns,
    RiverCargo,
    PlagueRelief,
    PlagueMedicine,
    CapitalPatrol,
    CapitalRumors,
    SouthernThunder,
    SouthernDrums,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SideQuestReward {
    pub exp: u32,
    pub potions: u32,
    pub gold: u32,
}

#[derive(Debug, PartialEq, Eq)]
pub struct SideQuestInteraction {
    pub lines: Vec<String>,
    pub reward: Option<SideQuestReward>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TreasureCache {
    VillageShrine,
    BambooOffering,
    CaveOffering,
    RiverTownCrystal,
    RiverReedCrystal,
    PlagueShrine,
    CapitalShrine,
    MansionMirror,
    SouthernTotem,
    FinalMemoryCache,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TreasureReward {
    pub exp: u32,
    pub potions: u32,
    pub gold: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CareAftermathReward {
    pub exp: u32,
    pub potions: u32,
    pub gold: u32,
}

#[derive(Debug, PartialEq, Eq)]
pub struct TreasureInteraction {
    pub lines: Vec<String>,
    pub reward: Option<TreasureReward>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShrineBlessing {
    Guard,
    Sword,
    Spirit,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QuestStage {
    NotStarted,
    TalkToLinger,
    FindStarMage,
    DefeatMonsters { remaining: u32 },
    ReturnToSister,
    EscortMerchant,
    FindBambooScout,
    SeekCavePriestess,
    CaveTrial { remaining: u32 },
    ConfrontMoonWraith,
    ReturnToLinger,
    OpeningComplete,
    GatherRiverHerbs { remaining: u32 },
    ReturnToHerbHealer,
    FindRiverBoatman,
    TuneRiverLanterns,
    ConfrontRiverDemon,
    RiverTownComplete,
    SeekPlagueElder,
    SeekShrineKeeper,
    CleansePlagueShrines { remaining: u32 },
    SealPlagueWards,
    ReturnToShrineKeeper,
    ConfrontMiasmaRoot,
    PlagueVillageComplete,
    SeekCapitalEnvoy,
    FindMansionSpy,
    GatherSecretLetters { remaining: u32 },
    AlignMansionMirrors,
    ReturnToMansionSpy,
    ConfrontMirrorMinister,
    CapitalIntrigueComplete,
    SeekSpiritGuide,
    SeekTribalChief,
    CleanseSpiritTotems { remaining: u32 },
    AlignThunderDrums,
    ReturnToTribalChief,
    ConfrontThunderQilin,
    SouthernRoadComplete,
    SeekFinalOracle,
    LightFinalSoulLamps { remaining: u32 },
    ReturnToFinalOracle,
    ConfrontDreamEclipse,
    FinaleComplete,
}

#[derive(Resource, Debug)]
pub struct QuestLog {
    stage: QuestStage,
    completed: u32,
    key_items: u32,
    companions: u32,
    side_active: u32,
    side_completed: u32,
    side_progress: [u32; SIDE_QUEST_COUNT],
    moon_crystals: u32,
    river_lanterns: u32,
    plague_wards: u32,
    mansion_mirrors: u32,
    thunder_drums: u32,
    final_lamps: u32,
    chapter_cards: u32,
    bond_scenes: u32,
    bond_courage: u32,
    bond_tender: u32,
    bond_bonus: u32,
    camp_scenes: u32,
    camp_bonus: u32,
    care_aftermath_claimed: u32,
    treasure_opened: u32,
    shrine_blessing: u32,
}

impl Default for QuestLog {
    fn default() -> Self {
        Self {
            stage: QuestStage::NotStarted,
            completed: 0,
            key_items: 0,
            companions: 0,
            side_active: 0,
            side_completed: 0,
            side_progress: [0; SIDE_QUEST_COUNT],
            moon_crystals: 0,
            river_lanterns: 0,
            plague_wards: 0,
            mansion_mirrors: 0,
            thunder_drums: 0,
            final_lamps: 0,
            chapter_cards: 0,
            bond_scenes: 0,
            bond_courage: 0,
            bond_tender: 0,
            bond_bonus: 0,
            camp_scenes: 0,
            camp_bonus: 0,
            care_aftermath_claimed: 0,
            treasure_opened: 0,
            shrine_blessing: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quest_flow_advances_from_pickup_to_turn_in() {
        let mut quest = QuestLog::default();
        assert_eq!(quest.stage(), QuestStage::NotStarted);

        quest.talk(QuestRole::SwordSister);
        assert_eq!(quest.stage(), QuestStage::TalkToLinger);

        quest.talk(QuestRole::Linger);
        assert_eq!(quest.stage(), QuestStage::FindStarMage);
        assert!(quest.has_key_item(KeyItem::TrackingTalisman));
        assert!(quest.has_companion(Companion::Linger));

        quest.talk(QuestRole::StarMage);
        assert_eq!(quest.stage(), QuestStage::DefeatMonsters { remaining: 2 });

        assert!(quest.record_victory().is_some());
        assert_eq!(quest.stage(), QuestStage::DefeatMonsters { remaining: 1 });

        assert!(quest.record_victory().is_some());
        assert_eq!(quest.stage(), QuestStage::ReturnToSister);

        quest.talk(QuestRole::SwordSister);
        assert_eq!(quest.stage(), QuestStage::EscortMerchant);

        quest.talk(QuestRole::Merchant);
        assert_eq!(quest.stage(), QuestStage::FindBambooScout);
        assert!(quest.has_key_item(KeyItem::RoadPass));

        quest.talk(QuestRole::BambooScout);
        assert_eq!(quest.stage(), QuestStage::SeekCavePriestess);
        assert!(quest.has_key_item(KeyItem::MoonToken));

        quest.talk(QuestRole::CavePriestess);
        assert_eq!(quest.stage(), QuestStage::CaveTrial { remaining: 3 });
        assert_eq!(quest.moon_crystal_marker_for(MoonCrystal::North), "!");

        assert!(quest.record_victory().is_some());
        assert_eq!(quest.stage(), QuestStage::CaveTrial { remaining: 2 });

        let lines = quest.activate_moon_crystal(MoonCrystal::North);
        assert!(lines[0].contains("上弦晶亮起"));
        assert_eq!(quest.moon_crystal_marker_for(MoonCrystal::North), "✓");

        assert!(quest.record_victory().is_some());
        assert_eq!(quest.stage(), QuestStage::CaveTrial { remaining: 1 });

        assert!(quest.record_victory().is_some());
        assert_eq!(quest.stage(), QuestStage::CaveTrial { remaining: 0 });

        let lines = quest.activate_moon_crystal(MoonCrystal::South);
        assert!(lines[0].contains("下弦晶亮起"));
        assert_eq!(quest.stage(), QuestStage::ConfrontMoonWraith);
        assert!(!quest.has_key_item(KeyItem::MoonSeal));

        assert!(quest.record_boss_victory(BossKind::MoonWraith).is_some());
        assert_eq!(quest.stage(), QuestStage::ReturnToLinger);
        assert!(quest.has_key_item(KeyItem::MoonSeal));

        quest.talk(QuestRole::Linger);
        assert_eq!(quest.stage(), QuestStage::OpeningComplete);
    }

    #[test]
    fn inventory_and_party_summary_are_story_driven() {
        let mut quest = QuestLog::default();
        assert_eq!(quest.party_summary(), "李逍遥");
        assert_eq!(quest.key_items_summary(), "无");

        quest.talk(QuestRole::SwordSister);
        quest.talk(QuestRole::Linger);
        assert_eq!(quest.party_summary(), "李逍遥、赵灵儿");
        assert_eq!(quest.key_items_summary(), "寻踪灵符");

        quest.stage = QuestStage::EscortMerchant;
        quest.talk(QuestRole::Merchant);
        assert_eq!(quest.key_items_summary(), "寻踪灵符、商路牌");

        quest.talk(QuestRole::BambooScout);
        assert_eq!(quest.key_items_summary(), "寻踪灵符、商路牌、月洞令");

        quest.talk(QuestRole::CavePriestess);
        quest.record_victory();
        quest.record_victory();
        quest.record_victory();
        quest.activate_moon_crystal(MoonCrystal::North);
        quest.activate_moon_crystal(MoonCrystal::South);
        quest.record_boss_victory(BossKind::MoonWraith);
        assert_eq!(
            quest.key_items_summary(),
            "寻踪灵符、商路牌、月洞令、月魄印"
        );
    }

    #[test]
    fn sword_sister_joins_after_village_turn_in() {
        let mut quest = QuestLog::default();
        quest.talk(QuestRole::SwordSister);
        quest.talk(QuestRole::Linger);
        quest.talk(QuestRole::StarMage);
        quest.record_victory();
        quest.record_victory();

        assert_eq!(quest.stage(), QuestStage::ReturnToSister);
        quest.talk(QuestRole::SwordSister);

        assert!(quest.has_companion(Companion::SwordSister));
        assert_eq!(quest.party_summary(), "李逍遥、赵灵儿、林月衡");
    }

    #[test]
    fn chapter_cards_are_one_time_story_beats() {
        let mut quest = QuestLog::default();
        assert_eq!(quest.current_chapter(), Chapter::VillageOath);
        assert!(!quest.has_seen_chapter_card(Chapter::VillageOath));

        let first = quest.take_chapter_card().expect("first chapter card");
        assert!(first[0].contains("第一卷"));
        assert!(first.iter().any(|line| line.contains("山路妖雾初起")));
        assert!(quest.has_seen_chapter_card(Chapter::VillageOath));
        assert!(quest.take_chapter_card().is_none());

        quest.talk(QuestRole::SwordSister);
        quest.talk(QuestRole::Linger);
        quest.stage = QuestStage::FindBambooScout;
        quest.talk(QuestRole::BambooScout);
        assert_eq!(quest.current_chapter(), Chapter::MoonCave);

        let second = quest.take_chapter_card().expect("second chapter card");
        assert!(second[0].contains("第二卷"));
        assert!(second.iter().any(|line| line.contains("水月洞天开门")));
        assert!(quest.take_chapter_card().is_none());
    }

    #[test]
    fn river_town_act_advances_to_second_boss() {
        let mut quest = QuestLog::default();
        quest.stage = QuestStage::OpeningComplete;

        quest.talk(QuestRole::HerbHealer);
        assert_eq!(quest.stage(), QuestStage::GatherRiverHerbs { remaining: 2 });
        assert!(quest.has_key_item(KeyItem::HerbPrescription));

        assert!(quest.record_victory().is_some());
        assert_eq!(quest.stage(), QuestStage::GatherRiverHerbs { remaining: 1 });

        assert!(quest.record_victory().is_some());
        assert_eq!(quest.stage(), QuestStage::ReturnToHerbHealer);
        assert!(quest.has_key_item(KeyItem::HerbBundle));

        quest.talk(QuestRole::HerbHealer);
        assert_eq!(quest.stage(), QuestStage::FindRiverBoatman);

        quest.talk(QuestRole::RiverBoatman);
        assert_eq!(quest.stage(), QuestStage::TuneRiverLanterns);
        assert!(!quest.has_key_item(KeyItem::FerryToken));
        assert_eq!(quest.river_lantern_marker_for(RiverLantern::Upstream), "!");

        let lines = quest.activate_river_lantern(RiverLantern::Dock);
        assert!(lines[0].contains("全数熄灭"));
        assert_eq!(quest.river_lantern_count(), 0);

        let lines = quest.activate_river_lantern(RiverLantern::Upstream);
        assert!(lines[0].contains("上游灯"));
        assert_eq!(quest.river_lantern_marker_for(RiverLantern::Upstream), "✓");
        assert_eq!(quest.river_lantern_marker_for(RiverLantern::Midstream), "!");

        quest.activate_river_lantern(RiverLantern::Midstream);
        quest.activate_river_lantern(RiverLantern::Dock);
        assert_eq!(quest.stage(), QuestStage::ConfrontRiverDemon);
        assert!(quest.has_key_item(KeyItem::FerryToken));

        assert!(quest.record_boss_victory(BossKind::RiverDemon).is_some());
        assert_eq!(quest.stage(), QuestStage::RiverTownComplete);
        assert!(quest.has_key_item(KeyItem::RiverPearl));
    }

    #[test]
    fn plague_village_act_has_investigation_cleanse_and_boss() {
        let mut quest = QuestLog::default();
        quest.stage = QuestStage::RiverTownComplete;

        assert_eq!(
            quest.objective(),
            "穿过江岸小镇东侧光门，经芦滩前往瘴雨村找村长。"
        );
        quest.talk(QuestRole::PlagueElder);
        assert_eq!(quest.stage(), QuestStage::SeekShrineKeeper);
        assert!(quest.has_key_item(KeyItem::PlagueReport));

        quest.talk(QuestRole::ShrineKeeper);
        assert_eq!(
            quest.stage(),
            QuestStage::CleansePlagueShrines { remaining: 3 }
        );
        assert!(quest.has_key_item(KeyItem::ShrineBell));

        assert!(quest.record_victory().is_some());
        assert_eq!(
            quest.stage(),
            QuestStage::CleansePlagueShrines { remaining: 2 }
        );
        assert!(quest.record_victory().is_some());
        assert_eq!(
            quest.stage(),
            QuestStage::CleansePlagueShrines { remaining: 1 }
        );
        assert!(quest.record_victory().is_some());
        assert_eq!(quest.stage(), QuestStage::SealPlagueWards);
        assert!(!quest.has_key_item(KeyItem::ShrineAsh));
        assert_eq!(quest.plague_ward_marker_for(PlagueWard::OldShrine), "!");

        let lines = quest.seal_plague_ward(PlagueWard::OldShrine);
        assert!(lines[0].contains("旧祠铃"));
        assert_eq!(quest.plague_ward_marker_for(PlagueWard::OldShrine), "✓");
        assert_eq!(quest.stage(), QuestStage::SealPlagueWards);

        quest.seal_plague_ward(PlagueWard::BitterWell);
        quest.seal_plague_ward(PlagueWard::Sickroom);
        assert_eq!(quest.stage(), QuestStage::ReturnToShrineKeeper);
        assert!(quest.has_key_item(KeyItem::ShrineAsh));

        quest.talk(QuestRole::ShrineKeeper);
        assert_eq!(quest.stage(), QuestStage::ConfrontMiasmaRoot);

        assert!(quest.record_boss_victory(BossKind::MiasmaRoot).is_some());
        assert_eq!(quest.stage(), QuestStage::PlagueVillageComplete);
        assert!(quest.has_key_item(KeyItem::CureCharm));
    }

    #[test]
    fn capital_intrigue_act_has_entry_infiltration_and_boss() {
        let mut quest = QuestLog::default();
        quest.stage = QuestStage::PlagueVillageComplete;

        assert_eq!(
            quest.objective(),
            "穿过瘴雨村东侧光门，前往云都府城找宣令使。"
        );
        quest.talk(QuestRole::CapitalEnvoy);
        assert_eq!(quest.stage(), QuestStage::FindMansionSpy);
        assert!(quest.has_key_item(KeyItem::CapitalWrit));
        assert_eq!(
            quest.objective(),
            "穿过云都府城东侧光门，去照影府邸偏院找内线。"
        );

        quest.talk(QuestRole::MansionSpy);
        assert_eq!(
            quest.stage(),
            QuestStage::GatherSecretLetters { remaining: 2 }
        );
        assert!(quest.has_key_item(KeyItem::CipherSlip));

        assert!(quest.record_victory().is_some());
        assert_eq!(
            quest.stage(),
            QuestStage::GatherSecretLetters { remaining: 1 }
        );
        assert!(quest.record_victory().is_some());
        assert_eq!(quest.stage(), QuestStage::AlignMansionMirrors);
        assert!(quest.has_key_item(KeyItem::SecretLetters));
        assert_eq!(
            quest.mansion_mirror_marker_for(MansionMirrorNode::Ledger),
            "!"
        );

        let lines = quest.align_mansion_mirror(MansionMirrorNode::Ledger);
        assert!(lines[0].contains("账镜"));
        assert_eq!(quest.stage(), QuestStage::AlignMansionMirrors);
        assert_eq!(
            quest.mansion_mirror_marker_for(MansionMirrorNode::Ledger),
            "✓"
        );

        quest.align_mansion_mirror(MansionMirrorNode::Witness);
        assert_eq!(quest.stage(), QuestStage::ReturnToMansionSpy);

        quest.talk(QuestRole::MansionSpy);
        assert_eq!(quest.stage(), QuestStage::ConfrontMirrorMinister);

        assert!(
            quest
                .record_boss_victory(BossKind::MirrorMinister)
                .is_some()
        );
        assert_eq!(quest.stage(), QuestStage::CapitalIntrigueComplete);
        assert!(quest.has_key_item(KeyItem::MirrorSeal));
    }

    #[test]
    fn southern_spirit_act_has_pickup_turn_in_boss_and_spell_upgrade() {
        let mut quest = QuestLog::default();
        quest.stage = QuestStage::CapitalIntrigueComplete;

        assert_eq!(
            quest.objective(),
            "穿过云都府城东侧光门，前往南疆灵道找引路人。"
        );
        assert_eq!(quest.quest_status(), "可接任务");
        assert_eq!(quest.spell_name(), "御剑术");

        quest.talk(QuestRole::SpiritGuide);
        assert_eq!(quest.stage(), QuestStage::SeekTribalChief);
        assert!(quest.has_key_item(KeyItem::SpiritRoadPass));

        quest.talk(QuestRole::TribalChief);
        assert_eq!(
            quest.stage(),
            QuestStage::CleanseSpiritTotems { remaining: 3 }
        );
        assert!(quest.has_key_item(KeyItem::TotemCharm));
        assert_eq!(quest.quest_status(), "进行中");

        assert!(quest.record_victory().is_some());
        assert_eq!(
            quest.stage(),
            QuestStage::CleanseSpiritTotems { remaining: 2 }
        );
        assert!(quest.record_victory().is_some());
        assert!(quest.record_victory().is_some());
        assert_eq!(quest.stage(), QuestStage::AlignThunderDrums);
        assert!(!quest.has_key_item(KeyItem::StormGlyph));
        assert_eq!(quest.quest_status(), "进行中");
        assert_eq!(quest.spell_name(), "御剑术");
        assert_eq!(quest.thunder_drum_marker_for(ThunderDrum::Wind), "!");

        let lines = quest.align_thunder_drum(ThunderDrum::Wind);
        assert!(lines[0].contains("风鼓"));
        assert_eq!(quest.thunder_drum_marker_for(ThunderDrum::Wind), "✓");
        assert_eq!(quest.stage(), QuestStage::AlignThunderDrums);

        quest.align_thunder_drum(ThunderDrum::Cloud);
        quest.align_thunder_drum(ThunderDrum::Oath);
        assert_eq!(quest.stage(), QuestStage::ReturnToTribalChief);
        assert!(quest.has_key_item(KeyItem::StormGlyph));
        assert_eq!(quest.quest_status(), "可交任务");
        assert_eq!(quest.spell_name(), "万剑诀");
        assert_eq!(quest.spell_cost(), 9);
        assert_eq!(quest.spell_power_multiplier(), 3);

        quest.talk(QuestRole::TribalChief);
        assert_eq!(quest.stage(), QuestStage::ConfrontThunderQilin);

        assert!(quest.record_boss_victory(BossKind::ThunderQilin).is_some());
        assert_eq!(quest.stage(), QuestStage::SouthernRoadComplete);
        assert!(quest.has_key_item(KeyItem::QilinHorn));
        assert_eq!(quest.quest_status(), "可接任务");
    }

    #[test]
    fn finale_act_has_lamp_puzzle_relationship_scene_and_final_boss() {
        let mut quest = QuestLog::default();
        quest.stage = QuestStage::SouthernRoadComplete;

        assert_eq!(
            quest.objective(),
            "穿过南疆灵道东侧光门，进入灵渊终门找守灯人。"
        );
        assert_eq!(quest.quest_status(), "可接任务");

        quest.talk(QuestRole::FinalOracle);
        assert_eq!(
            quest.stage(),
            QuestStage::LightFinalSoulLamps { remaining: 3 }
        );
        assert!(quest.has_key_item(KeyItem::FinalGateSigil));
        assert_eq!(quest.final_lamp_marker_for(FinalLamp::Memory), "!");

        let lines = quest.light_final_lamp(FinalLamp::Memory);
        assert!(lines[0].contains("忆灯亮起"));
        assert_eq!(
            quest.stage(),
            QuestStage::LightFinalSoulLamps { remaining: 2 }
        );
        assert_eq!(quest.final_lamp_marker_for(FinalLamp::Memory), "✓");

        let lines = quest.light_final_lamp(FinalLamp::Memory);
        assert!(lines[0].contains("已经点亮"));
        assert_eq!(
            quest.stage(),
            QuestStage::LightFinalSoulLamps { remaining: 2 }
        );

        quest.light_final_lamp(FinalLamp::Vow);
        assert_eq!(
            quest.stage(),
            QuestStage::LightFinalSoulLamps { remaining: 1 }
        );

        quest.light_final_lamp(FinalLamp::Fate);
        assert_eq!(quest.stage(), QuestStage::ReturnToFinalOracle);
        assert!(quest.has_key_item(KeyItem::DreamPearl));
        assert_eq!(quest.quest_status(), "可交任务");

        let linger_lines = quest.talk(QuestRole::Linger);
        assert!(linger_lines[0].contains("赵灵儿"));
        assert_eq!(quest.stage(), QuestStage::ReturnToFinalOracle);

        quest.talk(QuestRole::FinalOracle);
        assert_eq!(quest.stage(), QuestStage::ConfrontDreamEclipse);

        assert!(quest.record_boss_victory(BossKind::DreamEclipse).is_some());
        assert_eq!(quest.stage(), QuestStage::FinaleComplete);
        assert!(quest.has_key_item(KeyItem::FateSeal));
        assert_eq!(quest.quest_status(), "终章完成");
        assert_eq!(quest.marker_for(Some(QuestRole::FinalOracle)), "!");
        assert!(quest.objective().contains("回守灯人"));

        let epilogue = quest.talk(QuestRole::FinalOracle);
        assert!(epilogue[0].contains("终章尾声"));
        assert!(epilogue.iter().any(|line| line.contains("归梦印")));
        assert!(epilogue.iter().any(|line| line.contains("勇气和温柔")));
        assert!(epilogue.iter().any(|line| line.contains("旅途回响")));
        assert!(epilogue.iter().any(|line| line.contains("错过")));
        assert!(quest.has_key_item(KeyItem::HomecomingSeal));
        assert_eq!(quest.marker_for(Some(QuestRole::FinalOracle)), "✓");
        assert!(quest.objective().contains("终章尾声已定"));

        let repeat = quest.talk(QuestRole::FinalOracle);
        assert!(repeat[0].contains("归梦印仍在微亮"));
    }

    #[test]
    fn finale_epilogue_reflects_bond_side_and_camp_progress() {
        let mut quest = QuestLog::default();
        quest.stage = QuestStage::FinaleComplete;
        quest.bond_courage = 3;
        quest.bond_tender = 1;
        quest.bond_scenes = (1 << BOND_SCENE_COUNT) - 1;
        quest.side_completed = (1 << SIDE_QUEST_COUNT) - 1;
        quest.camp_scenes = (1 << CAMP_SCENE_COUNT) - 1;

        let lines = quest.finale_epilogue_lines();

        assert!(quest.has_key_item(KeyItem::HomecomingSeal));
        assert!(lines.iter().any(|line| line.contains("并肩回去")));
        assert!(lines.iter().any(|line| line.contains("十二张委托")));
        assert!(lines.iter().any(|line| line.contains("羁绊 7/7，营地 7/7")));
        assert!(lines.iter().any(|line| line.contains("每一次夜谈和休整")));
        assert!(lines.iter().any(|line| line.contains("营火都没有白点")));
    }

    #[test]
    fn side_quest_board_has_accept_progress_turn_in_and_reward() {
        let mut quest = QuestLog::default();
        assert_eq!(quest.side_quest_summary(), "支线 无");
        assert!(quest.active_task_tracker().contains("主线追踪"));
        assert!(
            quest
                .active_task_tracker()
                .contains("委托追踪：暂无已领取委托")
        );
        assert_eq!(
            quest.side_task_summary(SideQuest::VillageTrail),
            "任务板 可领取：山路余妖 - 击退两只草丛妖兽，再回村郊任务板交付。"
        );
        assert_eq!(
            quest.side_task_detail(SideQuest::VillageTrail),
            "【任务板】山路余妖\n状态：可领取\n操作：领取委托\n地点：余杭村郊外缘草丛\n目标：击退两只草丛妖兽，再回村郊任务板交付。\n追踪：踏入村郊草丛引出余妖，击退 2 只。\n交付：回余杭村郊任务板交付。\n进度：0/2\n报酬：经验 +24 / 药水 +1 / 钱 +18文"
        );
        assert_eq!(
            quest.side_task_action_label(SideQuest::VillageTrail),
            "领取"
        );
        assert_eq!(
            quest.side_task_prompt(SideQuest::VillageTrail),
            "委托板 山路余妖 | 可领取 | 空格领取\n地点：余杭村郊外缘草丛\n目标：击退两只草丛妖兽，再回村郊任务板交付。\n报酬：经验 +24 / 药水 +1 / 钱 +18文"
        );
        let preview = quest.side_task_accept_preview(SideQuest::VillageTrail);
        assert!(preview[1].contains("告示写着"));
        assert!(preview.iter().any(|line| line.contains("【追踪预览】")));
        assert!(preview.iter().any(|line| line.contains("【报酬】")));
        assert_eq!(quest.side_marker_for(SideQuest::VillageTrail), "!");

        let interaction = quest.interact_side_quest(SideQuest::VillageTrail);
        assert!(interaction.reward.is_none());
        assert!(interaction.lines[0].contains("【任务板】山路余妖"));
        assert!(interaction.lines[0].contains("状态：进行中"));
        assert!(interaction.lines[0].contains("操作：查看进度"));
        assert!(interaction.lines[1].contains("【领取委托】"));
        assert!(interaction.lines[1].contains("写入任务簿"));
        assert!(interaction.lines[3].contains("【委托地点】余杭村郊外缘草丛"));
        assert!(interaction.lines[4].contains("【任务追踪】踏入村郊草丛"));
        assert!(interaction.lines[5].contains("【交付】回余杭村郊任务板交付"));
        assert!(quest.is_side_quest_active(SideQuest::VillageTrail));
        let tracker = quest
            .active_side_task_tracker()
            .expect("accepted side quest should be tracked");
        assert!(tracker.contains("委托追踪 · 山路余妖 [进行中]"));
        assert!(tracker.contains("地点：余杭村郊外缘草丛"));
        assert!(tracker.contains("交付：回余杭村郊任务板交付"));
        let tracker = quest.active_task_tracker();
        assert!(tracker.contains("主线追踪"));
        assert!(tracker.contains("委托追踪 · 山路余妖 [进行中]"));
        assert_eq!(
            quest.side_task_action_label(SideQuest::VillageTrail),
            "查看"
        );
        assert_eq!(quest.side_quest_summary(), "支线 山路余妖 0/2");
        assert_eq!(
            quest.side_task_summary(SideQuest::VillageTrail),
            "任务板 进行中：山路余妖 0/2 - 击退两只草丛妖兽，再回村郊任务板交付。"
        );
        assert!(
            quest
                .side_task_detail(SideQuest::VillageTrail)
                .contains("状态：进行中\n操作：查看进度\n地点")
        );
        assert_eq!(quest.side_marker_for(SideQuest::VillageTrail), "*");

        assert_eq!(
            quest.record_side_victory(),
            Some("【支线】山路余妖 进度 1/2。".to_string())
        );
        assert_eq!(
            quest.record_side_victory(),
            Some("【支线】山路余妖 条件达成，回任务板交付。".to_string())
        );
        assert_eq!(quest.side_marker_for(SideQuest::VillageTrail), "!");
        assert!(
            quest
                .active_side_task_tracker()
                .expect("ready side quest should remain tracked")
                .contains("委托追踪 · 山路余妖 [可交付]")
        );
        assert!(
            quest
                .side_task_detail(SideQuest::VillageTrail)
                .contains("状态：可交付")
        );
        assert_eq!(
            quest.side_task_action_label(SideQuest::VillageTrail),
            "交付"
        );
        assert_eq!(
            quest.side_task_prompt(SideQuest::VillageTrail),
            "委托板 山路余妖 | 可交付 | 空格交付\n进度：2/2 | 回余杭村郊任务板交付。\n报酬：经验 +24 / 药水 +1 / 钱 +18文"
        );

        let interaction = quest.interact_side_quest(SideQuest::VillageTrail);
        assert_eq!(
            interaction.reward,
            Some(SideQuestReward {
                exp: 24,
                potions: 1,
                gold: 18,
            })
        );
        assert!(quest.is_side_quest_completed(SideQuest::VillageTrail));
        assert_eq!(quest.side_quest_summary(), "支线 已完成 1 项");
        assert_eq!(quest.active_side_task_tracker(), None);
        assert!(
            quest
                .active_task_tracker()
                .contains("委托追踪：暂无已领取委托")
        );
        assert!(interaction.lines[0].contains("状态：可交付"));
        assert!(interaction.lines[0].contains("操作：交付委托"));
        assert!(interaction.lines[1].contains("【支线完成】"));
        assert_eq!(
            quest.side_task_summary(SideQuest::VillageTrail),
            "任务板 已完成：山路余妖"
        );
        assert_eq!(
            quest.side_task_action_label(SideQuest::VillageTrail),
            "查看"
        );
        assert!(
            quest
                .side_task_detail(SideQuest::VillageTrail)
                .contains("状态：已完成")
        );
        assert_eq!(quest.side_marker_for(SideQuest::VillageTrail), "✓");

        let interaction = quest.interact_side_quest(SideQuest::VillageTrail);
        assert!(interaction.reward.is_none());
        assert!(interaction.lines[1].contains("已完成"));
    }

    #[test]
    fn treasure_cache_is_one_time_reward() {
        let mut quest = QuestLog::default();
        assert!(!quest.has_opened_treasure(TreasureCache::VillageShrine));

        let interaction = quest.interact_treasure(TreasureCache::VillageShrine);
        assert_eq!(
            interaction.reward,
            Some(TreasureReward {
                exp: 10,
                potions: 1,
                gold: 8,
            })
        );
        assert!(interaction.lines[0].contains("村郊旧像"));
        assert!(quest.has_opened_treasure(TreasureCache::VillageShrine));

        let interaction = quest.interact_treasure(TreasureCache::VillageShrine);
        assert!(interaction.reward.is_none());
        assert!(interaction.lines[0].contains("已经搜过"));
    }

    #[test]
    fn shrine_blessing_can_be_set_and_consumed() {
        let mut quest = QuestLog::default();
        assert_eq!(quest.active_shrine_blessing(), None);

        assert!(quest.set_shrine_blessing(ShrineBlessing::Sword));
        assert_eq!(quest.active_shrine_blessing(), Some(ShrineBlessing::Sword));
        assert!(!quest.set_shrine_blessing(ShrineBlessing::Guard));
        assert_eq!(quest.active_shrine_blessing(), Some(ShrineBlessing::Sword));

        assert_eq!(quest.take_shrine_blessing(), Some(ShrineBlessing::Sword));
        assert_eq!(quest.active_shrine_blessing(), None);
        assert_eq!(quest.take_shrine_blessing(), None);
    }

    #[test]
    fn camp_scenes_are_chapter_specific_one_time_bonuses() {
        let mut quest = QuestLog::default();
        assert_eq!(quest.camp_summary(), "营地 0/7");
        assert_eq!(quest.current_camp_scene(), CampScene::VillageHearth);

        let interaction = quest.interact_camp_scene();
        assert_eq!(interaction.bonus, Some(CampBonus::Warmth));
        assert!(interaction.tactic_choice);
        assert!(interaction.lines[0].contains("余杭家灯"));
        assert!(
            interaction
                .lines
                .iter()
                .any(|line| line.contains("营地议策"))
        );
        assert_eq!(quest.active_camp_bonus(), Some(CampBonus::Warmth));
        assert_eq!(quest.camp_level(), 1);
        assert!(quest.has_seen_camp_scene(CampScene::VillageHearth));
        assert_eq!(
            quest.travel_care_summary(),
            "旅途照应 羁绊 未启 营地 1/1 周全"
        );

        let tactic = quest.record_camp_tactic(CampBonus::Vigil);
        assert_eq!(quest.active_camp_bonus(), Some(CampBonus::Vigil));
        assert!(tactic[0].contains("灵儿守护"));
        assert!(tactic[1].contains("预警抵消"));

        let interaction = quest.interact_camp_scene();
        assert_eq!(interaction.bonus, None);
        assert!(!interaction.tactic_choice);
        assert!(interaction.lines[0].contains("已经休整过"));
        assert_eq!(quest.active_camp_bonus(), Some(CampBonus::Vigil));
        assert_eq!(quest.take_camp_bonus(), Some(CampBonus::Vigil));
        assert_eq!(quest.active_camp_bonus(), None);

        quest.stage = QuestStage::SeekCavePriestess;
        assert_eq!(quest.current_camp_scene(), CampScene::MoonCavePool);
        assert_eq!(
            quest.travel_care_summary(),
            "旅途照应 羁绊 未启 营地 1/2 缺 1"
        );
        let interaction = quest.interact_camp_scene();
        assert_eq!(interaction.bonus, Some(CampBonus::Focus));
        assert_eq!(quest.active_camp_bonus(), Some(CampBonus::Focus));

        quest.stage = QuestStage::SeekFinalOracle;
        assert_eq!(quest.current_camp_scene(), CampScene::FinalStillWater);
    }

    #[test]
    fn bond_scenes_are_chapter_specific_one_time_rewards() {
        let mut quest = QuestLog::default();
        assert_eq!(quest.bond_summary(), "羁绊 0/7");

        let interaction = quest.interact_bond_scene();
        assert!(interaction.reward.is_none());
        assert!(interaction.lines.iter().any(|line| line.contains("赵灵儿")));
        assert!(!interaction.response_choice);

        quest.talk(QuestRole::SwordSister);
        quest.talk(QuestRole::Linger);
        assert_eq!(
            quest.current_bond_scene(),
            Some(BondScene::VillageFirstNight)
        );
        assert_eq!(
            quest.travel_care_summary(),
            "旅途照应 羁绊 0/1 营地 0/1 缺 2"
        );

        let interaction = quest.interact_bond_scene();
        assert_eq!(
            interaction.reward,
            Some(BondReward {
                exp: 18,
                potions: 1,
                full_restore: true,
            })
        );
        assert!(quest.has_seen_bond_scene(BondScene::VillageFirstNight));
        assert_eq!(quest.bond_level(), 1);
        assert_eq!(quest.bond_summary(), "羁绊 1/7");
        assert_eq!(
            quest.travel_care_summary(),
            "旅途照应 羁绊 1/1 营地 0/1 缺 1"
        );
        assert!(interaction.response_choice);

        let lines = quest.record_bond_response(BondResponse::Courage);
        assert!(lines[0].contains("我先挡着"));
        assert!(lines.iter().any(|line| line.contains("勇心护念")));
        assert_eq!(quest.bond_courage_level(), 1);
        assert_eq!(quest.bond_tender_level(), 0);
        assert_eq!(quest.active_bond_bonus(), Some(BondBonus::Courage));
        assert_eq!(quest.bond_summary(), "羁绊 1/7 勇心护念");

        let lines = quest.record_bond_response(BondResponse::Tender);
        assert!(lines[0].contains("一起停一停"));
        assert!(lines.iter().any(|line| line.contains("柔心护念")));
        assert_eq!(quest.bond_courage_level(), 1);
        assert_eq!(quest.bond_tender_level(), 1);
        assert_eq!(quest.active_bond_bonus(), Some(BondBonus::Tender));
        assert_eq!(quest.take_bond_bonus(), Some(BondBonus::Tender));
        assert_eq!(quest.active_bond_bonus(), None);

        let interaction = quest.interact_bond_scene();
        assert!(interaction.reward.is_none());
        assert!(interaction.lines[0].contains("已经歇过"));
        assert!(!interaction.response_choice);

        quest.stage = QuestStage::LightFinalSoulLamps { remaining: 3 };
        assert_eq!(quest.current_bond_scene(), Some(BondScene::FinalGateQuiet));
    }
}

impl QuestLog {
    pub fn stage(&self) -> QuestStage {
        self.stage
    }

    pub fn has_key_item(&self, item: KeyItem) -> bool {
        self.key_items & key_item_bit(item) != 0
    }

    pub fn has_companion(&self, companion: Companion) -> bool {
        self.companions & companion_bit(companion) != 0
    }

    pub fn is_side_quest_active(&self, side: SideQuest) -> bool {
        self.side_active & side_quest_bit(side) != 0
    }

    pub fn is_side_quest_completed(&self, side: SideQuest) -> bool {
        self.side_completed & side_quest_bit(side) != 0
    }

    pub fn side_quest_progress(&self, side: SideQuest) -> u32 {
        self.side_progress[side.index()]
    }

    pub fn side_quest_goal(&self, side: SideQuest) -> u32 {
        side.goal()
    }

    pub fn side_quest_name(&self, side: SideQuest) -> &'static str {
        side.name()
    }

    pub fn is_final_lamp_lit(&self, lamp: FinalLamp) -> bool {
        self.final_lamps & final_lamp_bit(lamp) != 0
    }

    pub fn is_moon_crystal_lit(&self, crystal: MoonCrystal) -> bool {
        self.moon_crystals & moon_crystal_bit(crystal) != 0
    }

    pub fn is_river_lantern_lit(&self, lantern: RiverLantern) -> bool {
        self.river_lanterns & river_lantern_bit(lantern) != 0
    }

    pub fn is_plague_ward_sealed(&self, ward: PlagueWard) -> bool {
        self.plague_wards & plague_ward_bit(ward) != 0
    }

    pub fn is_mansion_mirror_aligned(&self, node: MansionMirrorNode) -> bool {
        self.mansion_mirrors & mansion_mirror_bit(node) != 0
    }

    pub fn is_thunder_drum_aligned(&self, drum: ThunderDrum) -> bool {
        self.thunder_drums & thunder_drum_bit(drum) != 0
    }

    pub fn current_chapter(&self) -> Chapter {
        Chapter::from_stage(self.stage)
    }

    pub fn has_seen_chapter_card(&self, chapter: Chapter) -> bool {
        self.chapter_cards & chapter_bit(chapter) != 0
    }

    pub fn take_chapter_card(&mut self) -> Option<Vec<String>> {
        let chapter = self.current_chapter();
        if self.has_seen_chapter_card(chapter) {
            return None;
        }

        self.chapter_cards |= chapter_bit(chapter);
        Some(vec![
            format!("【卷章展开】{}", chapter.title()),
            chapter.summary().to_string(),
            chapter.play_hint().to_string(),
            format!("【下一步】{}", self.objective()),
        ])
    }

    pub fn moon_crystal_count(&self) -> u32 {
        [MoonCrystal::North, MoonCrystal::South]
            .iter()
            .filter(|crystal| self.is_moon_crystal_lit(**crystal))
            .count() as u32
    }

    pub fn river_lantern_count(&self) -> u32 {
        RIVER_LANTERN_ORDER
            .iter()
            .filter(|lantern| self.is_river_lantern_lit(**lantern))
            .count() as u32
    }

    pub fn plague_ward_count(&self) -> u32 {
        [
            PlagueWard::OldShrine,
            PlagueWard::BitterWell,
            PlagueWard::Sickroom,
        ]
        .iter()
        .filter(|ward| self.is_plague_ward_sealed(**ward))
        .count() as u32
    }

    pub fn mansion_mirror_count(&self) -> u32 {
        [MansionMirrorNode::Ledger, MansionMirrorNode::Witness]
            .iter()
            .filter(|node| self.is_mansion_mirror_aligned(**node))
            .count() as u32
    }

    pub fn thunder_drum_count(&self) -> u32 {
        [ThunderDrum::Wind, ThunderDrum::Cloud, ThunderDrum::Oath]
            .iter()
            .filter(|drum| self.is_thunder_drum_aligned(**drum))
            .count() as u32
    }

    pub fn has_seen_bond_scene(&self, scene: BondScene) -> bool {
        self.bond_scenes & bond_scene_bit(scene) != 0
    }

    pub fn bond_level(&self) -> u32 {
        (0..BOND_SCENE_COUNT)
            .filter(|index| self.bond_scenes & (1 << index) != 0)
            .count() as u32
    }

    pub fn bond_courage_level(&self) -> u32 {
        self.bond_courage
    }

    pub fn bond_tender_level(&self) -> u32 {
        self.bond_tender
    }

    pub fn active_bond_bonus(&self) -> Option<BondBonus> {
        bond_bonus_from_bits(self.bond_bonus)
    }

    pub fn take_bond_bonus(&mut self) -> Option<BondBonus> {
        let bonus = self.active_bond_bonus();
        self.bond_bonus = 0;
        bonus
    }

    pub fn has_seen_camp_scene(&self, scene: CampScene) -> bool {
        self.camp_scenes & camp_scene_bit(scene) != 0
    }

    pub fn camp_level(&self) -> u32 {
        (0..CAMP_SCENE_COUNT)
            .filter(|index| self.camp_scenes & (1 << index) != 0)
            .count() as u32
    }

    pub fn active_camp_bonus(&self) -> Option<CampBonus> {
        camp_bonus_from_bits(self.camp_bonus)
    }

    pub fn take_camp_bonus(&mut self) -> Option<CampBonus> {
        let bonus = self.active_camp_bonus();
        self.camp_bonus = 0;
        bonus
    }

    pub fn camp_summary(&self) -> String {
        match self.active_camp_bonus() {
            Some(bonus) => format!(
                "营地 {}/{} {}",
                self.camp_level(),
                CAMP_SCENE_COUNT,
                bonus.name()
            ),
            None => format!("营地 {}/{}", self.camp_level(), CAMP_SCENE_COUNT),
        }
    }

    fn available_chapter_scene_count(&self) -> u32 {
        (self.current_chapter().index() as u32 + 1)
            .min(BOND_SCENE_COUNT as u32)
            .min(CAMP_SCENE_COUNT as u32)
    }

    fn available_bond_scene_count(&self) -> u32 {
        if self.has_companion(Companion::Linger) {
            self.available_chapter_scene_count()
        } else {
            0
        }
    }

    fn available_camp_scene_count(&self) -> u32 {
        self.available_chapter_scene_count()
    }

    fn missed_bond_scene_count(&self) -> u32 {
        let available = self.available_bond_scene_count();
        available.saturating_sub(self.bond_level().min(available))
    }

    fn missed_camp_scene_count(&self) -> u32 {
        let available = self.available_camp_scene_count();
        available.saturating_sub(self.camp_level().min(available))
    }

    pub fn travel_care_summary(&self) -> String {
        let bond_available = self.available_bond_scene_count();
        let camp_available = self.available_camp_scene_count();
        let bond = if bond_available == 0 {
            "羁绊 未启".to_string()
        } else {
            format!(
                "羁绊 {}/{}",
                self.bond_level().min(bond_available),
                bond_available
            )
        };
        let camp = format!(
            "营地 {}/{}",
            self.camp_level().min(camp_available),
            camp_available
        );
        let missed = self.missed_bond_scene_count() + self.missed_camp_scene_count();

        if missed == 0 {
            format!("旅途照应 {bond} {camp} 周全")
        } else {
            format!("旅途照应 {bond} {camp} 缺 {missed}")
        }
    }

    pub fn has_late_spell(&self) -> bool {
        self.has_key_item(KeyItem::StormGlyph) || self.has_key_item(KeyItem::QilinHorn)
    }

    pub fn spell_name(&self) -> &'static str {
        if self.has_late_spell() {
            "万剑诀"
        } else {
            "御剑术"
        }
    }

    pub fn spell_cost(&self) -> i32 {
        if self.has_late_spell() {
            LATE_SPELL_COST
        } else {
            BASIC_SPELL_COST
        }
    }

    pub fn spell_power_multiplier(&self) -> i32 {
        if self.has_late_spell() { 3 } else { 2 }
    }

    pub fn key_items_summary(&self) -> String {
        let mut names = Vec::new();
        if self.has_key_item(KeyItem::TrackingTalisman) {
            names.push("寻踪灵符");
        }
        if self.has_key_item(KeyItem::RoadPass) {
            names.push("商路牌");
        }
        if self.has_key_item(KeyItem::MoonToken) {
            names.push("月洞令");
        }
        if self.has_key_item(KeyItem::MoonSeal) {
            names.push("月魄印");
        }
        if self.has_key_item(KeyItem::HerbPrescription) {
            names.push("药庐方笺");
        }
        if self.has_key_item(KeyItem::HerbBundle) {
            names.push("清心药引");
        }
        if self.has_key_item(KeyItem::FerryToken) {
            names.push("渡江木牌");
        }
        if self.has_key_item(KeyItem::RiverPearl) {
            names.push("河心珠");
        }
        if self.has_key_item(KeyItem::PlagueReport) {
            names.push("瘴雨病簿");
        }
        if self.has_key_item(KeyItem::ShrineBell) {
            names.push("净瘴铃");
        }
        if self.has_key_item(KeyItem::ShrineAsh) {
            names.push("祠灰");
        }
        if self.has_key_item(KeyItem::CureCharm) {
            names.push("解瘴符");
        }
        if self.has_key_item(KeyItem::CapitalWrit) {
            names.push("入城符");
        }
        if self.has_key_item(KeyItem::CipherSlip) {
            names.push("暗号纸");
        }
        if self.has_key_item(KeyItem::SecretLetters) {
            names.push("府邸密札");
        }
        if self.has_key_item(KeyItem::MirrorSeal) {
            names.push("照影印");
        }
        if self.has_key_item(KeyItem::SpiritRoadPass) {
            names.push("灵道路引");
        }
        if self.has_key_item(KeyItem::TotemCharm) {
            names.push("百越图腾符");
        }
        if self.has_key_item(KeyItem::StormGlyph) {
            names.push("雷纹玉牒");
        }
        if self.has_key_item(KeyItem::QilinHorn) {
            names.push("雷麟角");
        }
        if self.has_key_item(KeyItem::FinalGateSigil) {
            names.push("终门符");
        }
        if self.has_key_item(KeyItem::DreamPearl) {
            names.push("忆梦珠");
        }
        if self.has_key_item(KeyItem::FateSeal) {
            names.push("宿命印");
        }
        if self.has_key_item(KeyItem::HomecomingSeal) {
            names.push("归梦印");
        }
        if names.is_empty() {
            "无".to_string()
        } else {
            names.join("、")
        }
    }

    pub fn party_summary(&self) -> &'static str {
        match (
            self.has_companion(Companion::Linger),
            self.has_companion(Companion::SwordSister),
        ) {
            (true, true) => "李逍遥、赵灵儿、林月衡",
            (true, false) => "李逍遥、赵灵儿",
            (false, true) => "李逍遥、林月衡",
            (false, false) => "李逍遥",
        }
    }

    pub fn side_quest_summary(&self) -> String {
        let active: Vec<String> = ALL_SIDE_QUESTS
            .iter()
            .copied()
            .filter(|side| self.is_side_quest_active(*side))
            .map(|side| {
                format!(
                    "{} {}/{}",
                    side.name(),
                    self.side_quest_progress(side),
                    side.goal()
                )
            })
            .collect();
        if !active.is_empty() {
            return format!("支线 {}", active.join("、"));
        }

        let completed = ALL_SIDE_QUESTS
            .iter()
            .filter(|side| self.is_side_quest_completed(**side))
            .count();
        if completed > 0 {
            format!("支线 已完成 {completed} 项")
        } else {
            "支线 无".to_string()
        }
    }

    pub fn active_side_task_tracker(&self) -> Option<String> {
        let active: Vec<SideQuest> = ALL_SIDE_QUESTS
            .iter()
            .copied()
            .filter(|side| self.is_side_quest_active(*side) && !self.is_side_quest_completed(*side))
            .collect();
        let side = active
            .iter()
            .copied()
            .find(|side| self.side_quest_progress(*side) >= side.goal())
            .or_else(|| active.first().copied())?;
        let progress = self.side_quest_progress(side).min(side.goal());
        let status = if progress >= side.goal() {
            "可交付"
        } else {
            "进行中"
        };
        let extra = active.len().saturating_sub(1);
        let extra_line = if extra > 0 {
            format!("\n另有 {extra} 份委托在任务簿。")
        } else {
            String::new()
        };

        Some(format!(
            "委托追踪 · {} [{}]\n目标：{}\n进度：{}/{}\n地点：{}\n交付：{}{}",
            side.name(),
            status,
            side.tracking_hint(),
            progress,
            side.goal(),
            side.area(),
            side.turn_in_place(),
            extra_line
        ))
    }

    pub fn active_task_tracker(&self) -> String {
        let mut tracker = format!(
            "主线追踪 · {} [{}]\n目标：{}",
            self.chapter_title(),
            self.quest_status(),
            self.objective()
        );

        if let Some(side_tracker) = self.active_side_task_tracker() {
            tracker.push_str("\n\n");
            tracker.push_str(&side_tracker);
        } else {
            tracker.push_str("\n委托追踪：暂无已领取委托");
        }

        tracker
    }

    pub fn bond_summary(&self) -> String {
        match self.active_bond_bonus() {
            Some(bonus) => format!(
                "羁绊 {}/{BOND_SCENE_COUNT} {}",
                self.bond_level(),
                bonus.name()
            ),
            None => format!("羁绊 {}/{BOND_SCENE_COUNT}", self.bond_level()),
        }
    }

    pub fn objective(&self) -> String {
        match self.stage {
            QuestStage::NotStarted => "去村郊东北找红衣剑姊。".to_string(),
            QuestStage::TalkToLinger => "问赵灵儿关于失踪灵符的线索。".to_string(),
            QuestStage::FindStarMage => "穿过光门，去青竹山径找星咒童子。".to_string(),
            QuestStage::DefeatMonsters { remaining } => {
                format!("击退山路妖兽：还剩 {remaining} 只。")
            }
            QuestStage::ReturnToSister => "回余杭村郊向红衣剑姊交任务。".to_string(),
            QuestStage::EscortMerchant => "去村郊中段找行脚商，确认山路是否重开。".to_string(),
            QuestStage::FindBambooScout => "护送商路后，去青竹山径找竹林斥候。".to_string(),
            QuestStage::SeekCavePriestess => "穿过竹林光门，进入水月洞天找月洞祭司。".to_string(),
            QuestStage::CaveTrial { remaining } => {
                let crystals = self.moon_crystal_count();
                if remaining == 0 {
                    format!("月洞妖气已散，在水月回廊触动两座水月晶阵：{crystals}/2。")
                } else {
                    format!(
                        "从水月洞天东侧光门进入水月回廊，净化妖气 {remaining} 股，触动水月晶阵 {crystals}/2。"
                    )
                }
            }
            QuestStage::ConfrontMoonWraith => "回到月洞祭司处，迎战月魄妖。".to_string(),
            QuestStage::ReturnToLinger => "回余杭村郊，把月洞所见告诉赵灵儿。".to_string(),
            QuestStage::OpeningComplete => {
                "穿过水月洞天东侧光门，前往江岸小镇找草药医。".to_string()
            }
            QuestStage::GatherRiverHerbs { remaining } => {
                format!("在江岸草滩收集清心药引：还需 {remaining} 份。")
            }
            QuestStage::ReturnToHerbHealer => "把清心药引交给江岸草药医。".to_string(),
            QuestStage::FindRiverBoatman => "去码头找摆渡人，查清河灯妖影。".to_string(),
            QuestStage::TuneRiverLanterns => format!(
                "去江岸芦滩按顺序点亮倒流河灯：上游→中洲→渡口（已归位 {}/3）。",
                self.river_lantern_count()
            ),
            QuestStage::ConfrontRiverDemon => "与摆渡人交谈，迎战河魇蛟。".to_string(),
            QuestStage::RiverTownComplete => {
                "穿过江岸小镇东侧光门，经芦滩前往瘴雨村找村长。".to_string()
            }
            QuestStage::SeekPlagueElder => "在瘴雨村找到村长，查清病源。".to_string(),
            QuestStage::SeekShrineKeeper => "带着瘴雨病簿，去村北祠堂找祠祝。".to_string(),
            QuestStage::CleansePlagueShrines { remaining } => {
                format!("从瘴雨村东侧光门进入瘴雨祠道，净化黑草瘴源：还剩 {remaining} 处。")
            }
            QuestStage::SealPlagueWards => format!(
                "瘴源已散，在瘴雨祠道巡回旧祠、苦井、病屋三处净瘴铃位：{}/3。",
                self.plague_ward_count()
            ),
            QuestStage::ReturnToShrineKeeper => "带祠灰回瘴雨村祠堂交给祠祝。".to_string(),
            QuestStage::ConfrontMiasmaRoot => "与祠祝交谈，斩断瘴母根。".to_string(),
            QuestStage::PlagueVillageComplete => {
                "穿过瘴雨村东侧光门，前往云都府城找宣令使。".to_string()
            }
            QuestStage::SeekCapitalEnvoy => "在云都府城找到宣令使，取得入城名帖。".to_string(),
            QuestStage::FindMansionSpy => {
                "穿过云都府城东侧光门，去照影府邸偏院找内线。".to_string()
            }
            QuestStage::GatherSecretLetters { remaining } => {
                format!("从照影府邸东侧光门进入照影镜廊，搜回府邸密札：还剩 {remaining} 份。")
            }
            QuestStage::AlignMansionMirrors => format!(
                "密札已齐，在照影镜廊对照两面镜阵：账镜、证镜（已归位 {}/2）。",
                self.mansion_mirror_count()
            ),
            QuestStage::ReturnToMansionSpy => "把府邸密札交给照影府邸偏院内线。".to_string(),
            QuestStage::ConfrontMirrorMinister => {
                "与府邸偏院内线交谈，揭开照影国师的幻术。".to_string()
            }
            QuestStage::CapitalIntrigueComplete => {
                "穿过云都府城东侧光门，前往南疆灵道找引路人。".to_string()
            }
            QuestStage::SeekSpiritGuide => "在南疆灵道找到引路人，领取百越图腾委托。".to_string(),
            QuestStage::SeekTribalChief => "沿南疆灵道寻找百越族长，询问图腾暴动。".to_string(),
            QuestStage::CleanseSpiritTotems { remaining } => {
                format!("从南疆灵道东侧光门进入雷鼓祭道，安抚暴走雷图腾：还剩 {remaining} 座。")
            }
            QuestStage::AlignThunderDrums => format!(
                "雷声已分三股，在雷鼓祭道巡回风鼓、云鼓、誓鼓三处雷鼓图腾：{}/3。",
                self.thunder_drum_count()
            ),
            QuestStage::ReturnToTribalChief => "带雷纹玉牒回南疆灵道交给百越族长。".to_string(),
            QuestStage::ConfrontThunderQilin => "与百越族长交谈，迎战雷麟。".to_string(),
            QuestStage::SouthernRoadComplete => {
                "穿过南疆灵道东侧光门，进入灵渊终门找守灯人。".to_string()
            }
            QuestStage::SeekFinalOracle => "在灵渊终门找到守灯人，询问宿命水影。".to_string(),
            QuestStage::LightFinalSoulLamps { remaining } => {
                format!("从灵渊终门东侧光门进入旧梦水廊，点亮三盏忆梦灯：还剩 {remaining} 盏。")
            }
            QuestStage::ReturnToFinalOracle => {
                "三盏忆梦灯已亮，回灵渊终门守灯人处交付。".to_string()
            }
            QuestStage::ConfrontDreamEclipse => "与守灯人交谈，迎战宿命水影。".to_string(),
            QuestStage::FinaleComplete => {
                if self.has_key_item(KeyItem::HomecomingSeal) {
                    format!("终章尾声已定：带着归梦印回看人间灯火 x{}。", self.completed)
                } else {
                    "宿命水影已散，回守灯人处定下归路与尾声。".to_string()
                }
            }
        }
    }

    pub fn quest_status(&self) -> &'static str {
        match self.stage {
            QuestStage::NotStarted
            | QuestStage::OpeningComplete
            | QuestStage::RiverTownComplete
            | QuestStage::PlagueVillageComplete
            | QuestStage::CapitalIntrigueComplete
            | QuestStage::SeekSpiritGuide
            | QuestStage::SouthernRoadComplete
            | QuestStage::SeekFinalOracle => "可接任务",
            QuestStage::ReturnToSister
            | QuestStage::ReturnToLinger
            | QuestStage::ReturnToHerbHealer
            | QuestStage::ReturnToShrineKeeper
            | QuestStage::ReturnToMansionSpy
            | QuestStage::ReturnToTribalChief
            | QuestStage::ReturnToFinalOracle => "可交任务",
            QuestStage::ConfrontMoonWraith
            | QuestStage::ConfrontRiverDemon
            | QuestStage::ConfrontMiasmaRoot
            | QuestStage::ConfrontMirrorMinister
            | QuestStage::ConfrontThunderQilin
            | QuestStage::ConfrontDreamEclipse => "首领战",
            QuestStage::FinaleComplete => "终章完成",
            _ => "进行中",
        }
    }

    pub fn main_task_summary(&self) -> String {
        format!("主线 {}：{}", self.quest_status(), self.objective())
    }

    pub fn side_task_summary(&self, side: SideQuest) -> String {
        if self.is_side_quest_completed(side) {
            return format!("任务板 已完成：{}", side.name());
        }

        if self.is_side_quest_active(side) {
            let progress = self.side_quest_progress(side);
            let state = if progress >= side.goal() {
                "可交付"
            } else {
                "进行中"
            };
            return format!(
                "任务板 {state}：{} {}/{} - {}",
                side.name(),
                progress,
                side.goal(),
                side.objective()
            );
        }

        format!("任务板 可领取：{} - {}", side.name(), side.objective())
    }

    pub fn side_task_detail(&self, side: SideQuest) -> String {
        let progress = self.side_quest_progress(side);
        let shown_progress = if self.is_side_quest_completed(side) {
            side.goal()
        } else {
            progress
        };
        format!(
            "【任务板】{}\n状态：{}\n操作：{}\n地点：{}\n目标：{}\n追踪：{}\n交付：{}\n进度：{}/{}\n报酬：{}",
            side.name(),
            self.side_task_status(side),
            self.side_task_operation(side),
            side.area(),
            side.objective(),
            side.tracking_hint(),
            side.turn_in_place(),
            shown_progress.min(side.goal()),
            side.goal(),
            side.reward_text()
        )
    }

    fn side_task_status(&self, side: SideQuest) -> &'static str {
        if self.is_side_quest_completed(side) {
            "已完成"
        } else if self.is_side_quest_active(side) {
            if self.side_quest_progress(side) >= side.goal() {
                "可交付"
            } else {
                "进行中"
            }
        } else {
            "可领取"
        }
    }

    fn side_task_operation(&self, side: SideQuest) -> &'static str {
        if self.is_side_quest_completed(side) {
            "查看记录"
        } else if self.is_side_quest_active(side) {
            if self.side_quest_progress(side) >= side.goal() {
                "交付委托"
            } else {
                "查看进度"
            }
        } else {
            "领取委托"
        }
    }

    pub fn side_task_action_label(&self, side: SideQuest) -> &'static str {
        if self.is_side_quest_completed(side) {
            "查看"
        } else if self.is_side_quest_active(side) {
            if self.side_quest_progress(side) >= side.goal() {
                "交付"
            } else {
                "查看"
            }
        } else {
            "领取"
        }
    }

    pub fn side_task_prompt(&self, side: SideQuest) -> String {
        let header = format!(
            "委托板 {} | {} | 空格{}",
            side.name(),
            self.side_task_status(side),
            self.side_task_action_label(side)
        );

        if self.is_side_quest_completed(side) {
            return format!("{header}\n{}", side.completed_line());
        }

        if self.is_side_quest_active(side) {
            let progress = self.side_quest_progress(side).min(side.goal());
            if progress >= side.goal() {
                return format!(
                    "{header}\n进度：{progress}/{} | {}\n报酬：{}",
                    side.goal(),
                    side.turn_in_place(),
                    side.reward_text()
                );
            }

            return format!(
                "{header}\n进度：{progress}/{} | {}\n交付：{}",
                side.goal(),
                side.tracking_hint(),
                side.turn_in_place()
            );
        }

        format!(
            "{header}\n地点：{}\n目标：{}\n报酬：{}",
            side.area(),
            side.objective(),
            side.reward_text()
        )
    }

    pub fn side_task_accept_preview(&self, side: SideQuest) -> Vec<String> {
        vec![
            self.side_task_detail(side),
            side.accept_line().to_string(),
            format!("【委托地点】{}", side.area()),
            format!("【追踪预览】{}", side.tracking_hint()),
            format!("【交付】{}", side.turn_in_place()),
            format!("【报酬】{}", side.reward_text()),
        ]
    }

    pub fn side_task_completed_line(&self, side: SideQuest) -> &'static str {
        side.completed_line()
    }

    pub fn interact_side_quest(&mut self, side: SideQuest) -> SideQuestInteraction {
        if self.is_side_quest_completed(side) {
            return SideQuestInteraction {
                lines: vec![
                    self.side_task_detail(side),
                    format!("【支线已完成】{}。", side.name()),
                    side.completed_line().to_string(),
                ],
                reward: None,
            };
        }

        if self.is_side_quest_active(side) {
            let progress = self.side_quest_progress(side);
            if progress >= side.goal() {
                let detail = self.side_task_detail(side);
                self.side_active &= !side_quest_bit(side);
                self.side_completed |= side_quest_bit(side);
                return SideQuestInteraction {
                    lines: vec![
                        detail,
                        format!("【支线完成】{}。", side.name()),
                        side.turn_in_line().to_string(),
                    ],
                    reward: Some(side.reward()),
                };
            }

            return SideQuestInteraction {
                lines: vec![
                    self.side_task_detail(side),
                    format!(
                        "【支线进行中】{} {}/{}。",
                        side.name(),
                        progress,
                        side.goal()
                    ),
                    format!("【追踪】{}", side.tracking_hint()),
                    side.progress_line().to_string(),
                ],
                reward: None,
            };
        }

        self.side_active |= side_quest_bit(side);
        self.side_progress[side.index()] = 0;
        let detail = self.side_task_detail(side);
        SideQuestInteraction {
            lines: vec![
                detail,
                format!("【领取委托】《{}》已揭榜，写入任务簿。", side.name()),
                side.accept_line().to_string(),
                format!("【委托地点】{}", side.area()),
                format!("【任务追踪】{}", side.tracking_hint()),
                format!("【交付】{}", side.turn_in_place()),
                format!("【报酬】{}", side.reward_text()),
            ],
            reward: None,
        }
    }

    pub fn record_side_victory(&mut self) -> Option<String> {
        let mut messages = Vec::new();
        for side in ALL_SIDE_QUESTS {
            if !self.is_side_quest_active(side) || self.is_side_quest_completed(side) {
                continue;
            }

            let progress = &mut self.side_progress[side.index()];
            if *progress >= side.goal() {
                continue;
            }

            *progress += 1;
            if *progress >= side.goal() {
                messages.push(format!("【支线】{} 条件达成，回任务板交付。", side.name()));
            } else {
                messages.push(format!(
                    "【支线】{} 进度 {}/{}。",
                    side.name(),
                    *progress,
                    side.goal()
                ));
            }
        }

        if messages.is_empty() {
            None
        } else {
            Some(messages.join("\n"))
        }
    }

    pub fn side_marker_for(&self, side: SideQuest) -> &'static str {
        if self.is_side_quest_completed(side) {
            "✓"
        } else if self.is_side_quest_active(side) {
            if self.side_quest_progress(side) >= side.goal() {
                "!"
            } else {
                "*"
            }
        } else {
            "!"
        }
    }

    pub fn has_opened_treasure(&self, cache: TreasureCache) -> bool {
        self.treasure_opened & treasure_cache_bit(cache) != 0
    }

    pub fn active_shrine_blessing(&self) -> Option<ShrineBlessing> {
        shrine_blessing_from_bits(self.shrine_blessing)
    }

    pub fn set_shrine_blessing(&mut self, blessing: ShrineBlessing) -> bool {
        if self.active_shrine_blessing().is_some() {
            return false;
        }

        self.shrine_blessing = shrine_blessing_bit(blessing);
        true
    }

    pub fn take_shrine_blessing(&mut self) -> Option<ShrineBlessing> {
        let blessing = self.active_shrine_blessing();
        self.shrine_blessing = 0;
        blessing
    }

    pub fn interact_treasure(&mut self, cache: TreasureCache) -> TreasureInteraction {
        if self.has_opened_treasure(cache) {
            return TreasureInteraction {
                lines: vec![
                    format!("【探索】{}已经搜过。", cache.name()),
                    cache.empty_line().to_string(),
                ],
                reward: None,
            };
        }

        self.treasure_opened |= treasure_cache_bit(cache);
        TreasureInteraction {
            lines: vec![
                format!("【探索】{}", cache.name()),
                cache.open_line().to_string(),
                format!("【发现】{}", cache.find_line()),
            ],
            reward: Some(cache.reward()),
        }
    }

    pub fn activate_moon_crystal(&mut self, crystal: MoonCrystal) -> Vec<String> {
        let QuestStage::CaveTrial { remaining } = self.stage else {
            return vec![format!(
                "{}只是映出一圈冷光。{}",
                crystal.name(),
                self.objective()
            )];
        };

        if self.is_moon_crystal_lit(crystal) {
            return vec![format!(
                "{}已经亮起。水月晶阵 {}/2。",
                crystal.name(),
                self.moon_crystal_count()
            )];
        }

        self.moon_crystals |= moon_crystal_bit(crystal);
        let count = self.moon_crystal_count();
        if remaining == 0 && count >= 2 {
            self.stage = QuestStage::ConfrontMoonWraith;
            vec![
                format!("【机关】{}亮起，两座水月晶阵合成月桥。", crystal.name()),
                "月洞祭司：水镜已开，月魄妖被逼出真身。".to_string(),
                format!("【目标】{}", self.objective()),
            ]
        } else {
            vec![
                format!("【机关】{}亮起，水面浮出半道月桥。", crystal.name()),
                format!("【目标】{}", self.objective()),
            ]
        }
    }

    pub fn moon_crystal_marker_for(&self, crystal: MoonCrystal) -> &'static str {
        if self.is_moon_crystal_lit(crystal) {
            "✓"
        } else if matches!(self.stage, QuestStage::CaveTrial { .. }) {
            "!"
        } else {
            "?"
        }
    }

    pub fn activate_river_lantern(&mut self, lantern: RiverLantern) -> Vec<String> {
        if self.stage != QuestStage::TuneRiverLanterns {
            return vec![format!(
                "{}浮在水雾里，灯芯尚未听令。{}",
                lantern.name(),
                self.objective()
            )];
        }

        if self.is_river_lantern_lit(lantern) {
            return vec![format!(
                "{}已经顺流亮起。倒流河灯 {}/3。",
                lantern.name(),
                self.river_lantern_count()
            )];
        }

        let count = self.river_lantern_count();
        let expected = RIVER_LANTERN_ORDER[count as usize];
        if lantern != expected {
            self.river_lanterns = 0;
            return vec![
                format!("【机关】{}逆着水光一闪，三盏河灯全数熄灭。", lantern.name()),
                "摆渡人：顺水问灯，先上游，再中洲，最后渡口；错一步就得重来。".to_string(),
                format!("【目标】{}", self.objective()),
            ];
        }

        self.river_lanterns |= river_lantern_bit(lantern);
        let next_count = self.river_lantern_count();
        if next_count < RIVER_LANTERN_ORDER.len() as u32 {
            let next = RIVER_LANTERN_ORDER[next_count as usize];
            return vec![
                format!("【机关】{}回到顺水光，芦苇间传来船铃声。", lantern.name()),
                format!("【目标】河灯归位 {next_count}/3，下一盏：{}。", next.name()),
            ];
        }

        self.add_key_item(KeyItem::FerryToken);
        self.stage = QuestStage::ConfrontRiverDemon;
        vec![
            "【机关】三盏倒流河灯连成水路，灯影逼出船底黑潮。".to_string(),
            "【获得道具】渡江木牌。".to_string(),
            "摆渡人：木牌一响，它就会现身。少侠，站稳了！".to_string(),
            format!("【目标】{}", self.objective()),
        ]
    }

    pub fn river_lantern_marker_for(&self, lantern: RiverLantern) -> &'static str {
        if self.is_river_lantern_lit(lantern) {
            "✓"
        } else if self.stage == QuestStage::TuneRiverLanterns {
            let next = RIVER_LANTERN_ORDER[self.river_lantern_count() as usize];
            if lantern == next { "!" } else { "*" }
        } else {
            "?"
        }
    }

    pub fn seal_plague_ward(&mut self, ward: PlagueWard) -> Vec<String> {
        if self.stage != QuestStage::SealPlagueWards {
            return vec![format!(
                "{}的铃位沉在雨声里。{}",
                ward.name(),
                self.objective()
            )];
        }

        if self.is_plague_ward_sealed(ward) {
            return vec![format!(
                "{}已经封好。净瘴铃位 {}/3。",
                ward.name(),
                self.plague_ward_count()
            )];
        }

        self.plague_wards |= plague_ward_bit(ward);
        let count = self.plague_ward_count();
        if count < 3 {
            return vec![
                format!("【净瘴铃】{}清响，雨雾退开一圈。", ward.name()),
                format!("【目标】净瘴铃位 {count}/3，继续巡回旧祠、苦井、病屋。"),
            ];
        }

        self.add_key_item(KeyItem::ShrineAsh);
        self.stage = QuestStage::ReturnToShrineKeeper;
        vec![
            "【净瘴铃】三处铃位同声，祠堂香灰里凝出一撮白灰。".to_string(),
            "【获得道具】祠灰。".to_string(),
            "赵灵儿：瘴气不是散了，是被逼回祠下了。".to_string(),
            format!("【目标】{}", self.objective()),
        ]
    }

    pub fn plague_ward_marker_for(&self, ward: PlagueWard) -> &'static str {
        if self.is_plague_ward_sealed(ward) {
            "✓"
        } else if self.stage == QuestStage::SealPlagueWards {
            "!"
        } else {
            "?"
        }
    }

    pub fn align_mansion_mirror(&mut self, node: MansionMirrorNode) -> Vec<String> {
        if self.stage != QuestStage::AlignMansionMirrors {
            return vec![format!(
                "{}只映出一层冷光。{}",
                node.name(),
                self.objective()
            )];
        }

        if self.is_mansion_mirror_aligned(node) {
            return vec![format!(
                "{}已经归位。照影镜阵 {}/2。",
                node.name(),
                self.mansion_mirror_count()
            )];
        }

        self.mansion_mirrors |= mansion_mirror_bit(node);
        let count = self.mansion_mirror_count();
        if count < 2 {
            return vec![
                format!(
                    "【镜阵】{}照出密札暗文，府邸正堂响起一声铜扣。",
                    node.name()
                ),
                format!("【目标】照影镜阵 {count}/2，继续寻找另一面镜。"),
            ];
        }

        self.stage = QuestStage::ReturnToMansionSpy;
        vec![
            "【镜阵】账镜与证镜互相照亮，密札上的人名全部显出。".to_string(),
            "林月衡：这下内线能把照影国师引出来了。".to_string(),
            format!("【目标】{}", self.objective()),
        ]
    }

    pub fn mansion_mirror_marker_for(&self, node: MansionMirrorNode) -> &'static str {
        if self.is_mansion_mirror_aligned(node) {
            "✓"
        } else if self.stage == QuestStage::AlignMansionMirrors {
            "!"
        } else {
            "?"
        }
    }

    pub fn align_thunder_drum(&mut self, drum: ThunderDrum) -> Vec<String> {
        if self.stage != QuestStage::AlignThunderDrums {
            return vec![format!("{}还压着闷雷。{}", drum.name(), self.objective())];
        }

        if self.is_thunder_drum_aligned(drum) {
            return vec![format!(
                "{}已经归位。雷鼓图腾 {}/3。",
                drum.name(),
                self.thunder_drum_count()
            )];
        }

        self.thunder_drums |= thunder_drum_bit(drum);
        let count = self.thunder_drum_count();
        if count < 3 {
            return vec![
                format!("【雷鼓】{}震出一道清雷，草阵亮起半圈雷纹。", drum.name()),
                format!("【目标】雷鼓图腾 {count}/3，继续寻找风鼓、云鼓、誓鼓。"),
            ];
        }

        self.add_key_item(KeyItem::StormGlyph);
        self.stage = QuestStage::ReturnToTribalChief;
        vec![
            "【雷鼓】三面雷鼓同声，雷纹玉牒在图腾座上成形。".to_string(),
            "【获得道具】雷纹玉牒。".to_string(),
            "【仙术领悟】御剑术领悟为万剑诀。".to_string(),
            "林月衡：雷声替剑分路，万剑齐落时就不会散了。".to_string(),
            format!("【目标】{}", self.objective()),
        ]
    }

    pub fn thunder_drum_marker_for(&self, drum: ThunderDrum) -> &'static str {
        if self.is_thunder_drum_aligned(drum) {
            "✓"
        } else if self.stage == QuestStage::AlignThunderDrums {
            "!"
        } else {
            "?"
        }
    }

    pub fn light_final_lamp(&mut self, lamp: FinalLamp) -> Vec<String> {
        let QuestStage::LightFinalSoulLamps { remaining } = self.stage else {
            return vec![format!("{}尚未回应。{}", lamp.name(), self.objective())];
        };

        if self.is_final_lamp_lit(lamp) {
            return vec![format!(
                "{}已经点亮。还剩 {} 盏忆梦灯。",
                lamp.name(),
                remaining
            )];
        }

        self.final_lamps |= final_lamp_bit(lamp);
        if remaining > 1 {
            let next = remaining - 1;
            self.stage = QuestStage::LightFinalSoulLamps { remaining: next };
            vec![
                format!("【机关】{}亮起，灯中映出一段旧梦。", lamp.name()),
                format!("【目标】还剩 {next} 盏忆梦灯。"),
            ]
        } else {
            self.add_key_item(KeyItem::DreamPearl);
            self.stage = QuestStage::ReturnToFinalOracle;
            vec![
                format!("【机关】{}亮起，三盏忆梦灯合成忆梦珠。", lamp.name()),
                "赵灵儿：原来一路所见，都在这几盏灯里等我们回答。".to_string(),
                format!("【目标】{}", self.objective()),
            ]
        }
    }

    pub fn final_lamp_marker_for(&self, lamp: FinalLamp) -> &'static str {
        if self.is_final_lamp_lit(lamp) {
            "✓"
        } else if matches!(self.stage, QuestStage::LightFinalSoulLamps { .. }) {
            "!"
        } else {
            "?"
        }
    }

    pub fn current_bond_scene(&self) -> Option<BondScene> {
        if !self.has_companion(Companion::Linger) {
            return None;
        }

        match self.stage {
            QuestStage::FindStarMage
            | QuestStage::DefeatMonsters { .. }
            | QuestStage::ReturnToSister
            | QuestStage::EscortMerchant
            | QuestStage::FindBambooScout => Some(BondScene::VillageFirstNight),
            QuestStage::SeekCavePriestess
            | QuestStage::CaveTrial { .. }
            | QuestStage::ConfrontMoonWraith
            | QuestStage::ReturnToLinger
            | QuestStage::OpeningComplete => Some(BondScene::MoonCavePromise),
            QuestStage::GatherRiverHerbs { .. }
            | QuestStage::ReturnToHerbHealer
            | QuestStage::FindRiverBoatman
            | QuestStage::TuneRiverLanterns
            | QuestStage::ConfrontRiverDemon
            | QuestStage::RiverTownComplete => Some(BondScene::RiverLampWish),
            QuestStage::SeekPlagueElder
            | QuestStage::SeekShrineKeeper
            | QuestStage::CleansePlagueShrines { .. }
            | QuestStage::SealPlagueWards
            | QuestStage::ReturnToShrineKeeper
            | QuestStage::ConfrontMiasmaRoot
            | QuestStage::PlagueVillageComplete => Some(BondScene::PlagueRainShelter),
            QuestStage::SeekCapitalEnvoy
            | QuestStage::FindMansionSpy
            | QuestStage::GatherSecretLetters { .. }
            | QuestStage::AlignMansionMirrors
            | QuestStage::ReturnToMansionSpy
            | QuestStage::ConfrontMirrorMinister
            | QuestStage::CapitalIntrigueComplete => Some(BondScene::CapitalRooftop),
            QuestStage::SeekSpiritGuide
            | QuestStage::SeekTribalChief
            | QuestStage::CleanseSpiritTotems { .. }
            | QuestStage::AlignThunderDrums
            | QuestStage::ReturnToTribalChief
            | QuestStage::ConfrontThunderQilin
            | QuestStage::SouthernRoadComplete => Some(BondScene::SouthernRoadOath),
            QuestStage::SeekFinalOracle
            | QuestStage::LightFinalSoulLamps { .. }
            | QuestStage::ReturnToFinalOracle
            | QuestStage::ConfrontDreamEclipse
            | QuestStage::FinaleComplete => Some(BondScene::FinalGateQuiet),
            QuestStage::NotStarted | QuestStage::TalkToLinger => None,
        }
    }

    pub fn interact_bond_scene(&mut self) -> BondInteraction {
        let Some(scene) = self.current_bond_scene() else {
            return BondInteraction {
                lines: vec![
                    "【歇脚】灵灯旁只有风声。".to_string(),
                    "赵灵儿还未与你同行，暂时没有可触发的同行剧情。".to_string(),
                ],
                reward: None,
                response_choice: false,
            };
        };

        if self.has_seen_bond_scene(scene) {
            return BondInteraction {
                lines: vec![
                    format!("【歇脚】{}已经歇过。", scene.title()),
                    scene.repeat_line().to_string(),
                ],
                reward: None,
                response_choice: false,
            };
        }

        self.bond_scenes |= bond_scene_bit(scene);
        let mut lines = vec![format!("【羁绊】{}", scene.title())];
        lines.extend(scene.lines().iter().map(|line| (*line).to_string()));
        BondInteraction {
            lines,
            reward: Some(scene.reward()),
            response_choice: true,
        }
    }

    pub fn record_bond_response(&mut self, response: BondResponse) -> Vec<String> {
        match response {
            BondResponse::Courage => {
                self.bond_courage += 1;
                self.bond_bonus = bond_bonus_bit(BondBonus::Courage);
                vec![
                    "【回应】李逍遥：不管前面是什么，我先挡着。".to_string(),
                    format!(
                        "【羁绊倾向】勇心 +1。勇心 {} / 柔心 {}。",
                        self.bond_courage, self.bond_tender
                    ),
                    format!(
                        "【羁绊护念】{}：{}",
                        BondBonus::Courage.name(),
                        BondBonus::Courage.battle_line()
                    ),
                    "赵灵儿：那我也不会站在你身后太远。".to_string(),
                ]
            }
            BondResponse::Tender => {
                self.bond_tender += 1;
                self.bond_bonus = bond_bonus_bit(BondBonus::Tender);
                vec![
                    "【回应】李逍遥：你若累了，我们就一起停一停。".to_string(),
                    format!(
                        "【羁绊倾向】柔心 +1。勇心 {} / 柔心 {}。",
                        self.bond_courage, self.bond_tender
                    ),
                    format!(
                        "【羁绊护念】{}：{}",
                        BondBonus::Tender.name(),
                        BondBonus::Tender.battle_line()
                    ),
                    "赵灵儿：有这句话，路好像也没那么长了。".to_string(),
                ]
            }
        }
    }

    pub fn current_camp_scene(&self) -> CampScene {
        match self.stage {
            QuestStage::NotStarted
            | QuestStage::TalkToLinger
            | QuestStage::FindStarMage
            | QuestStage::DefeatMonsters { .. }
            | QuestStage::ReturnToSister
            | QuestStage::EscortMerchant
            | QuestStage::FindBambooScout => CampScene::VillageHearth,
            QuestStage::SeekCavePriestess
            | QuestStage::CaveTrial { .. }
            | QuestStage::ConfrontMoonWraith
            | QuestStage::ReturnToLinger => CampScene::MoonCavePool,
            QuestStage::OpeningComplete
            | QuestStage::GatherRiverHerbs { .. }
            | QuestStage::ReturnToHerbHealer
            | QuestStage::FindRiverBoatman
            | QuestStage::TuneRiverLanterns
            | QuestStage::ConfrontRiverDemon
            | QuestStage::RiverTownComplete => CampScene::RiverTownInn,
            QuestStage::SeekPlagueElder
            | QuestStage::SeekShrineKeeper
            | QuestStage::CleansePlagueShrines { .. }
            | QuestStage::SealPlagueWards
            | QuestStage::ReturnToShrineKeeper
            | QuestStage::ConfrontMiasmaRoot
            | QuestStage::PlagueVillageComplete => CampScene::PlagueSickroom,
            QuestStage::SeekCapitalEnvoy
            | QuestStage::FindMansionSpy
            | QuestStage::GatherSecretLetters { .. }
            | QuestStage::AlignMansionMirrors
            | QuestStage::ReturnToMansionSpy
            | QuestStage::ConfrontMirrorMinister
            | QuestStage::CapitalIntrigueComplete => CampScene::CapitalSafehouse,
            QuestStage::SeekSpiritGuide
            | QuestStage::SeekTribalChief
            | QuestStage::CleanseSpiritTotems { .. }
            | QuestStage::AlignThunderDrums
            | QuestStage::ReturnToTribalChief
            | QuestStage::ConfrontThunderQilin
            | QuestStage::SouthernRoadComplete => CampScene::SouthernCampfire,
            QuestStage::SeekFinalOracle
            | QuestStage::LightFinalSoulLamps { .. }
            | QuestStage::ReturnToFinalOracle
            | QuestStage::ConfrontDreamEclipse
            | QuestStage::FinaleComplete => CampScene::FinalStillWater,
        }
    }

    pub fn interact_camp_scene(&mut self) -> CampInteraction {
        let scene = self.current_camp_scene();
        if self.has_seen_camp_scene(scene) {
            return CampInteraction {
                lines: vec![
                    format!("【营地】{}已经休整过。", scene.title()),
                    scene.repeat_line().to_string(),
                ],
                bonus: None,
                tactic_choice: false,
            };
        }

        self.camp_scenes |= camp_scene_bit(scene);
        let bonus = scene.bonus();
        self.camp_bonus = camp_bonus_bit(bonus);
        let mut lines = vec![format!("【营地】{}", scene.title())];
        lines.extend(scene.lines().iter().map(|line| (*line).to_string()));
        lines.push(format!(
            "【营地准备】{}：{}",
            bonus.name(),
            bonus.battle_line()
        ));
        lines.push(format!(
            "【营地议策】默认采用{}，也可以改选下一场战术。",
            bonus.name()
        ));
        CampInteraction {
            lines,
            bonus: Some(bonus),
            tactic_choice: true,
        }
    }

    pub fn record_camp_tactic(&mut self, bonus: CampBonus) -> Vec<String> {
        self.camp_bonus = camp_bonus_bit(bonus);
        vec![
            format!(
                "【营地战术】{}：{}",
                bonus.tactic_label(),
                bonus.tactic_line()
            ),
            bonus.battle_line().to_string(),
            self.camp_summary(),
        ]
    }

    pub fn finale_epilogue_lines(&mut self) -> Vec<String> {
        if self.stage != QuestStage::FinaleComplete {
            return vec![format!("守灯人：尾声尚远。{}", self.objective())];
        }

        if self.has_key_item(KeyItem::HomecomingSeal) {
            return vec![
                "【终章尾声】归梦印仍在微亮。".to_string(),
                self.finale_memory_line(),
                "守灯人：去吧，结局已经从水声里走出来了。".to_string(),
            ];
        }

        self.add_key_item(KeyItem::HomecomingSeal);
        let mut lines = vec![
            "【终章尾声】灵渊水面退开，三盏忆梦灯照见归路。".to_string(),
            "【获得道具】归梦印。".to_string(),
            self.finale_bond_line(),
            self.finale_world_line(),
            self.finale_memory_line(),
            format!("【目标】{}", self.objective()),
        ];
        if self.camp_level() >= 5 {
            lines.push("【同伴回响】一路的营火都没有白点，队伍知道明天该往哪里走。".to_string());
        }
        lines
    }

    fn finale_bond_line(&self) -> String {
        if self.bond_courage_level() > self.bond_tender_level() {
            "赵灵儿：你总说走在前面，这一次，我们并肩回去。".to_string()
        } else if self.bond_tender_level() > self.bond_courage_level() {
            "赵灵儿：你记得停下来等我，所以我也记得怎样把你带回人间。".to_string()
        } else {
            "赵灵儿：勇气和温柔都在灯里，我们不用只选其中一个。".to_string()
        }
    }

    fn finale_world_line(&self) -> String {
        let completed_sides = self.side_completed.count_ones();
        if completed_sides >= SIDE_QUEST_COUNT as u32 {
            "守灯人：十二张委托都化作人间灯火，灵渊外有人正在等你们的消息。".to_string()
        } else if completed_sides >= 6 {
            format!("守灯人：你们沿途点亮了 {completed_sides} 处人间小愿，它们替归路留了光。")
        } else {
            "守灯人：还有许多小愿留在人间，归去之后仍有路要走。".to_string()
        }
    }

    fn finale_memory_line(&self) -> String {
        let bond = self.bond_level();
        let camps = self.camp_level();
        let missed_bond = (BOND_SCENE_COUNT as u32).saturating_sub(bond);
        let missed_camp = (CAMP_SCENE_COUNT as u32).saturating_sub(camps);
        let missed = missed_bond + missed_camp;

        if missed == 0 {
            format!(
                "【旅途回响】羁绊 {bond}/{BOND_SCENE_COUNT}，营地 {camps}/{CAMP_SCENE_COUNT}；每一次夜谈和休整都留在回程里。"
            )
        } else {
            format!(
                "【旅途回响】羁绊 {bond}/{BOND_SCENE_COUNT}，营地 {camps}/{CAMP_SCENE_COUNT}；错过 {missed} 次照应，归路仍有未说完的话。"
            )
        }
    }

    pub fn chapter_title(&self) -> &'static str {
        self.current_chapter().title()
    }

    pub fn talk(&mut self, role: QuestRole) -> Vec<String> {
        match (role, self.stage) {
            (QuestRole::SwordSister, QuestStage::NotStarted) => {
                self.stage = QuestStage::TalkToLinger;
                vec![
                    "【接取任务】红衣剑姊托付：巡山除妖。".to_string(),
                    "红衣剑姊：先去问赵灵儿，她知道灵符最后出现在哪里。".to_string(),
                    format!("【目标】{}", self.objective()),
                ]
            }
            (QuestRole::SwordSister, QuestStage::ReturnToSister) => {
                self.completed += 1;
                self.add_companion(Companion::SwordSister);
                self.stage = QuestStage::EscortMerchant;
                vec![
                    "【任务完成】巡山除妖。".to_string(),
                    "【队友加入】林月衡决定与你同行。".to_string(),
                    "红衣剑姊：妖气退了，山路终于能走了。".to_string(),
                    "林月衡：别急着谢我。后面的路若还有妖，我正好也要查个清楚。".to_string(),
                    "林月衡：先去问问行脚商吧，他若敢动身，说明路是真的开了。".to_string(),
                    format!("【目标】{}", self.objective()),
                ]
            }
            (QuestRole::SwordSister, _) => {
                vec![format!("红衣剑姊：还没到交付时机。{}", self.objective())]
            }
            (QuestRole::Linger, QuestStage::TalkToLinger) => {
                self.add_key_item(KeyItem::TrackingTalisman);
                self.add_companion(Companion::Linger);
                self.stage = QuestStage::FindStarMage;
                vec![
                    "【任务推进】赵灵儿给了你一枚寻踪灵符。".to_string(),
                    "【队友加入】赵灵儿暂时与你同行。".to_string(),
                    "赵灵儿：灵符指向青竹山径，星咒童子能看清妖气源头。".to_string(),
                    format!("【目标】{}", self.objective()),
                ]
            }
            (QuestRole::Linger, QuestStage::NotStarted) => {
                vec!["赵灵儿：红衣剑姊似乎有急事找你，先去东北角看看吧。".to_string()]
            }
            (QuestRole::Linger, QuestStage::ReturnToLinger) => {
                self.completed += 1;
                self.stage = QuestStage::OpeningComplete;
                vec![
                    "【章节完成】村誓与水月洞天。".to_string(),
                    "赵灵儿：原来月洞也在等一个答案。".to_string(),
                    "赵灵儿：若要继续往江南去，先穿过月洞东侧光门。".to_string(),
                    format!("【目标】{}", self.objective()),
                ]
            }
            (
                QuestRole::Linger,
                QuestStage::SeekFinalOracle
                | QuestStage::LightFinalSoulLamps { .. }
                | QuestStage::ReturnToFinalOracle
                | QuestStage::ConfrontDreamEclipse,
            ) => {
                vec![
                    "赵灵儿：逍遥哥哥，若灯中照见的是结局，也别急着认输。".to_string(),
                    "赵灵儿：我们一路走到这里，本就不是为了照着宿命走。".to_string(),
                ]
            }
            (QuestRole::Linger, QuestStage::FinaleComplete) => {
                vec![
                    "赵灵儿：水声终于安静了。".to_string(),
                    "赵灵儿：等天亮，我们再一起回去看看那些被救下的人。".to_string(),
                ]
            }
            (QuestRole::Linger, _) => vec![format!("赵灵儿：我会守住这里。{}", self.objective())],
            (QuestRole::StarMage, QuestStage::FindStarMage) => {
                self.stage = QuestStage::DefeatMonsters { remaining: 2 };
                vec![
                    "【任务推进】星咒童子标出了妖气游走的位置。".to_string(),
                    "星咒童子：踏入草丛引出妖兽，击退两只后回村交付。".to_string(),
                    format!("【目标】{}", self.objective()),
                ]
            }
            (QuestRole::StarMage, QuestStage::DefeatMonsters { remaining }) => {
                vec![format!("星咒童子：妖气未散，还剩 {remaining} 只要处理。")]
            }
            (QuestRole::StarMage, QuestStage::NotStarted) => {
                vec!["星咒童子：你身上没有委托灵符，先在村里接任务。".to_string()]
            }
            (QuestRole::StarMage, _) => vec![format!("星咒童子：星盘已定。{}", self.objective())],
            (QuestRole::Merchant, QuestStage::EscortMerchant) => {
                self.add_key_item(KeyItem::RoadPass);
                self.stage = QuestStage::FindBambooScout;
                vec![
                    "【任务推进】商路重开。".to_string(),
                    "【获得道具】商路牌。".to_string(),
                    "行脚商：风声变轻了，我可以试着过山。".to_string(),
                    "行脚商：竹林里有个斥候替我探路，你若遇见他，替我报个平安。".to_string(),
                    format!("【目标】{}", self.objective()),
                ]
            }
            (QuestRole::Merchant, _) => vec![format!("行脚商：我先看住货担。{}", self.objective())],
            (QuestRole::BambooScout, QuestStage::FindBambooScout) => {
                self.add_key_item(KeyItem::MoonToken);
                self.stage = QuestStage::SeekCavePriestess;
                vec![
                    "【任务推进】竹林斥候交出月洞令。".to_string(),
                    "【获得道具】月洞令。".to_string(),
                    "竹林斥候：商路能走，但洞天里的妖气还没散。".to_string(),
                    "竹林斥候：拿着这枚月洞令，去找月洞祭司。".to_string(),
                    format!("【目标】{}", self.objective()),
                ]
            }
            (QuestRole::BambooScout, _) => {
                vec![format!("竹林斥候：竹影里有动静。{}", self.objective())]
            }
            (QuestRole::CavePriestess, QuestStage::SeekCavePriestess) => {
                self.stage = QuestStage::CaveTrial { remaining: 3 };
                self.moon_crystals = 0;
                vec![
                    "【任务推进】月洞试炼开启。".to_string(),
                    "月洞祭司：水脉映心，心乱则妖生。".to_string(),
                    "月洞祭司：从东侧光门入水月回廊，净化三股妖气，再触动两座水月晶阵。"
                        .to_string(),
                    format!("【目标】{}", self.objective()),
                ]
            }
            (QuestRole::CavePriestess, QuestStage::CaveTrial { remaining }) => {
                if remaining == 0 {
                    vec![format!(
                        "月洞祭司：妖气已散，水月回廊的晶阵还差 {}/2。",
                        self.moon_crystal_count()
                    )]
                } else {
                    vec![format!(
                        "月洞祭司：试炼未尽，水月回廊还剩 {remaining} 股妖气，晶阵 {}/2。",
                        self.moon_crystal_count()
                    )]
                }
            }
            (QuestRole::CavePriestess, QuestStage::ConfrontMoonWraith) => {
                vec![
                    "月洞祭司：水纹倒映出了你最怕失去的东西。".to_string(),
                    "月洞祭司：月魄妖已醒，若你退一步，洞门会永远关上。".to_string(),
                    "【Boss】月魄妖现身。".to_string(),
                ]
            }
            (QuestRole::CavePriestess, _) => {
                vec![format!(
                    "月洞祭司：洞天看见了你的来路。{}",
                    self.objective()
                )]
            }
            (QuestRole::HerbHealer, QuestStage::OpeningComplete) => {
                self.add_key_item(KeyItem::HerbPrescription);
                self.stage = QuestStage::GatherRiverHerbs { remaining: 2 };
                vec![
                    "【接取任务】江岸药庐：清心药引。".to_string(),
                    "【获得道具】药庐方笺。".to_string(),
                    "草药医：河雾带着妖毒，寻常药压不住。".to_string(),
                    "草药医：去草滩采两份清心药引，路上妖影会被药香引来。".to_string(),
                    format!("【目标】{}", self.objective()),
                ]
            }
            (QuestRole::HerbHealer, QuestStage::GatherRiverHerbs { remaining }) => {
                vec![format!("草药医：方笺还热着，药引还差 {remaining} 份。")]
            }
            (QuestRole::HerbHealer, QuestStage::ReturnToHerbHealer) => {
                self.stage = QuestStage::FindRiverBoatman;
                vec![
                    "【任务推进】药引入炉，河雾暂退。".to_string(),
                    "草药医：药只能救眼前人，源头还在码头的河灯下。".to_string(),
                    "草药医：摆渡人昨夜见过妖影，去找他问清楚。".to_string(),
                    format!("【目标】{}", self.objective()),
                ]
            }
            (QuestRole::HerbHealer, _) => vec![format!("草药医：药火不能断。{}", self.objective())],
            (QuestRole::RiverBoatman, QuestStage::FindRiverBoatman) => {
                self.river_lanterns = 0;
                self.stage = QuestStage::TuneRiverLanterns;
                vec![
                    "【任务推进】摆渡人指向江岸芦滩。".to_string(),
                    "摆渡人：昨夜河灯自己逆流，水下有东西跟着船。".to_string(),
                    "摆渡人：顺水问灯，先上游，再中洲，最后渡口；三灯归位我才敢下篙。".to_string(),
                    format!("【目标】{}", self.objective()),
                ]
            }
            (QuestRole::RiverBoatman, QuestStage::TuneRiverLanterns) => {
                vec![
                    format!(
                        "摆渡人：河灯还没顺水，已归位 {}/3。",
                        self.river_lantern_count()
                    ),
                    "摆渡人：去芦滩按上游、中洲、渡口的顺序点灯。".to_string(),
                ]
            }
            (QuestRole::RiverBoatman, QuestStage::ConfrontRiverDemon) => {
                vec![
                    "摆渡人：木牌在响，它就在船底，别让河灯全灭！".to_string(),
                    "【Boss】河魇蛟破浪而出。".to_string(),
                ]
            }
            (QuestRole::RiverBoatman, _) => {
                vec![format!("摆渡人：江水今日不太平。{}", self.objective())]
            }
            (
                QuestRole::PlagueElder,
                QuestStage::RiverTownComplete | QuestStage::SeekPlagueElder,
            ) => {
                self.add_key_item(KeyItem::PlagueReport);
                self.stage = QuestStage::SeekShrineKeeper;
                vec![
                    "【接取任务】瘴雨村：病簿与祠灰。".to_string(),
                    "【获得道具】瘴雨病簿。".to_string(),
                    "村长：雨停过三次，病却一次比一次重。".to_string(),
                    "村长：祠堂旧铃或许能引出瘴源，拿病簿去找祠祝。".to_string(),
                    format!("【目标】{}", self.objective()),
                ]
            }
            (QuestRole::PlagueElder, QuestStage::CleansePlagueShrines { remaining }) => {
                vec![format!("村长：村里还压着瘴气，剩 {remaining} 处要净。")]
            }
            (QuestRole::PlagueElder, QuestStage::SealPlagueWards) => {
                vec![format!(
                    "村长：瘴源被你们打散了，旧祠、苦井、病屋三处铃位还差 {}/3。",
                    self.plague_ward_count()
                )]
            }
            (QuestRole::PlagueElder, _) => vec![format!("村长：雨声不对。{}", self.objective())],
            (QuestRole::ShrineKeeper, QuestStage::SeekShrineKeeper) => {
                self.add_key_item(KeyItem::ShrineBell);
                self.stage = QuestStage::CleansePlagueShrines { remaining: 3 };
                vec![
                    "【任务推进】祠祝借出净瘴铃。".to_string(),
                    "【获得道具】净瘴铃。".to_string(),
                    "祠祝：铃响三处，瘴母才会露根。".to_string(),
                    "祠祝：从村口光门进瘴雨祠道，踏入黑草先净三处瘴源。".to_string(),
                    format!("【目标】{}", self.objective()),
                ]
            }
            (QuestRole::ShrineKeeper, QuestStage::CleansePlagueShrines { remaining }) => {
                vec![format!("祠祝：铃声还浑，还剩 {remaining} 处瘴源。")]
            }
            (QuestRole::ShrineKeeper, QuestStage::SealPlagueWards) => {
                vec![
                    format!(
                        "祠祝：瘴源已散，铃还没回音。三处铃位已净 {}/3。",
                        self.plague_ward_count()
                    ),
                    "祠祝：三处铃位都在瘴雨祠道，旧祠定魂，苦井引水，病屋护人。".to_string(),
                ]
            }
            (QuestRole::ShrineKeeper, QuestStage::ReturnToShrineKeeper) => {
                self.stage = QuestStage::ConfrontMiasmaRoot;
                vec![
                    "【任务推进】祠灰归位，瘴母根苏醒。".to_string(),
                    "祠祝：根在祠下，斩不断它，药再多也无用。".to_string(),
                    "【Boss】瘴母根破土而出。".to_string(),
                ]
            }
            (QuestRole::ShrineKeeper, QuestStage::ConfrontMiasmaRoot) => {
                vec!["祠祝：莫让瘴母根重新扎回祠下！".to_string()]
            }
            (QuestRole::ShrineKeeper, _) => vec![format!("祠祝：铃声会认路。{}", self.objective())],
            (
                QuestRole::CapitalEnvoy,
                QuestStage::PlagueVillageComplete | QuestStage::SeekCapitalEnvoy,
            ) => {
                self.add_key_item(KeyItem::CapitalWrit);
                self.stage = QuestStage::FindMansionSpy;
                vec![
                    "【接取任务】云都府城：照影案。".to_string(),
                    "【获得道具】入城符。".to_string(),
                    "宣令使：瘴雨村的解瘴符已送到京中，但府邸里有人压下奏报。".to_string(),
                    "宣令使：拿此符去偏院找内线，查清照影国师在藏什么。".to_string(),
                    format!("【目标】{}", self.objective()),
                ]
            }
            (QuestRole::CapitalEnvoy, QuestStage::GatherSecretLetters { remaining }) => {
                vec![format!("宣令使：府中耳目多，密札还差 {remaining} 份。")]
            }
            (QuestRole::CapitalEnvoy, _) => vec![format!("宣令使：城门已开。{}", self.objective())],
            (QuestRole::MansionSpy, QuestStage::FindMansionSpy) => {
                self.add_key_item(KeyItem::CipherSlip);
                self.stage = QuestStage::GatherSecretLetters { remaining: 2 };
                vec![
                    "【任务推进】偏院内线交出暗号纸。".to_string(),
                    "【获得道具】暗号纸。".to_string(),
                    "偏院内线：国师以镜阵审人，密札藏在照影镜廊两处暗廊。".to_string(),
                    "偏院内线：从东侧光门进去，击退守阵幻影，取回两份密札。".to_string(),
                    format!("【目标】{}", self.objective()),
                ]
            }
            (QuestRole::MansionSpy, QuestStage::GatherSecretLetters { remaining }) => {
                vec![format!("偏院内线：暗号还没失效，密札还差 {remaining} 份。")]
            }
            (QuestRole::MansionSpy, QuestStage::AlignMansionMirrors) => {
                vec![
                    format!(
                        "偏院内线：密札是真的，但镜阵还没对上，账镜与证镜还差 {}/2。",
                        self.mansion_mirror_count()
                    ),
                    "偏院内线：两面镜都在照影镜廊，一面记钱，一面记供词。".to_string(),
                ]
            }
            (QuestRole::MansionSpy, QuestStage::ReturnToMansionSpy) => {
                self.stage = QuestStage::ConfrontMirrorMinister;
                vec![
                    "【任务推进】密札合拢，镜阵显出真名。".to_string(),
                    "偏院内线：照影国师就在正堂，他借镜阵吞了许多人的证词。".to_string(),
                    "【Boss】照影国师现身。".to_string(),
                ]
            }
            (QuestRole::MansionSpy, QuestStage::ConfrontMirrorMinister) => {
                vec!["偏院内线：正堂镜光已开，别看他的眼睛！".to_string()]
            }
            (QuestRole::MansionSpy, _) => vec![format!("偏院内线：墙也有耳。{}", self.objective())],
            (
                QuestRole::SpiritGuide,
                QuestStage::CapitalIntrigueComplete | QuestStage::SeekSpiritGuide,
            ) => {
                self.add_key_item(KeyItem::SpiritRoadPass);
                self.stage = QuestStage::SeekTribalChief;
                vec![
                    "【接取任务】南疆灵道：雷图腾暴动。".to_string(),
                    "【获得道具】灵道路引。".to_string(),
                    "引路人：照影印能开云都东门，却压不住南疆雷火。".to_string(),
                    "引路人：拿着路引去找百越族长，灵道上的图腾已经开始伤人。".to_string(),
                    format!("【目标】{}", self.objective()),
                ]
            }
            (QuestRole::SpiritGuide, _) => {
                vec![format!("引路人：灵道认印，也认人心。{}", self.objective())]
            }
            (QuestRole::TribalChief, QuestStage::SeekTribalChief) => {
                self.add_key_item(KeyItem::TotemCharm);
                self.stage = QuestStage::CleanseSpiritTotems { remaining: 3 };
                vec![
                    "【任务推进】百越族长交出图腾符。".to_string(),
                    "【获得道具】百越图腾符。".to_string(),
                    "百越族长：雷麟守着古路，本不伤人，如今却被镜阵余毒激怒。".to_string(),
                    "百越族长：从东侧光门进雷鼓祭道，安抚三座雷图腾，再回来交付。".to_string(),
                    format!("【目标】{}", self.objective()),
                ]
            }
            (QuestRole::TribalChief, QuestStage::CleanseSpiritTotems { remaining }) => {
                vec![format!(
                    "百越族长：图腾还在鸣雷，还剩 {remaining} 座要安抚。"
                )]
            }
            (QuestRole::TribalChief, QuestStage::AlignThunderDrums) => {
                vec![
                    format!(
                        "百越族长：雷声已经分路，三面雷鼓已归位 {}/3。",
                        self.thunder_drum_count()
                    ),
                    "百越族长：三鼓都在雷鼓祭道，风鼓引路，云鼓收声，誓鼓定心。".to_string(),
                ]
            }
            (QuestRole::TribalChief, QuestStage::ReturnToTribalChief) => {
                self.stage = QuestStage::ConfrontThunderQilin;
                vec![
                    "【任务推进】雷纹玉牒归位，古路灵门显形。".to_string(),
                    "百越族长：玉牒只让雷声听见你，真正的考验还在雷麟面前。".to_string(),
                    "【Boss】雷麟踏云而至。".to_string(),
                ]
            }
            (QuestRole::TribalChief, QuestStage::ConfrontThunderQilin) => {
                vec!["百越族长：雷麟已经现身，别让它重新踏碎灵门！".to_string()]
            }
            (QuestRole::TribalChief, _) => {
                vec![format!("百越族长：山风会把誓言带远。{}", self.objective())]
            }
            (
                QuestRole::FinalOracle,
                QuestStage::SouthernRoadComplete | QuestStage::SeekFinalOracle,
            ) => {
                self.add_key_item(KeyItem::FinalGateSigil);
                self.stage = QuestStage::LightFinalSoulLamps { remaining: 3 };
                self.final_lamps = 0;
                vec![
                    "【接取任务】灵渊终门：三灯问心。".to_string(),
                    "【获得道具】终门符。".to_string(),
                    "守灯人：雷麟角已开终门，但门后的水影仍困着许多旧梦。".to_string(),
                    "守灯人：从终门东侧光门入旧梦水廊，点亮忆、誓、命三盏忆梦灯，再回来交付。"
                        .to_string(),
                    format!("【目标】{}", self.objective()),
                ]
            }
            (QuestRole::FinalOracle, QuestStage::LightFinalSoulLamps { remaining }) => {
                vec![format!("守灯人：灯阵未成，还剩 {remaining} 盏忆梦灯。")]
            }
            (QuestRole::FinalOracle, QuestStage::ReturnToFinalOracle) => {
                self.stage = QuestStage::ConfrontDreamEclipse;
                vec![
                    "【任务推进】忆梦珠归位，灵渊水影显形。".to_string(),
                    "守灯人：你们已经改写灯中的旧梦，剩下的宿命只能亲手斩断。".to_string(),
                    "【Boss】宿命水影卷起灵渊。".to_string(),
                ]
            }
            (QuestRole::FinalOracle, QuestStage::ConfrontDreamEclipse) => {
                vec!["守灯人：宿命水影已醒，灯阵只能撑住片刻！".to_string()]
            }
            (QuestRole::FinalOracle, QuestStage::FinaleComplete) => self.finale_epilogue_lines(),
            (QuestRole::FinalOracle, _) => {
                vec![format!("守灯人：终门尚远。{}", self.objective())]
            }
        }
    }

    pub fn record_victory(&mut self) -> Option<String> {
        match self.stage {
            QuestStage::DefeatMonsters { remaining } => {
                if remaining > 1 {
                    let next = remaining - 1;
                    self.stage = QuestStage::DefeatMonsters { remaining: next };
                    Some(format!("【任务】妖气削弱，还剩 {next} 只。"))
                } else {
                    self.stage = QuestStage::ReturnToSister;
                    Some("【任务】妖兽已清，回村找红衣剑姊交付。".to_string())
                }
            }
            QuestStage::CaveTrial { remaining } => {
                if remaining > 1 {
                    let next = remaining - 1;
                    self.stage = QuestStage::CaveTrial { remaining: next };
                    Some(format!("【试炼】月洞妖气净化，还剩 {next} 股。"))
                } else if self.moon_crystal_count() >= 2 {
                    self.stage = QuestStage::ConfrontMoonWraith;
                    Some("【试炼】妖气与晶阵都已净，回月洞祭司处迎战月魄妖。".to_string())
                } else {
                    self.stage = QuestStage::CaveTrial { remaining: 0 };
                    Some(format!(
                        "【试炼】三股妖气已净，继续在水月回廊触动晶阵 {}/2。",
                        self.moon_crystal_count()
                    ))
                }
            }
            QuestStage::GatherRiverHerbs { remaining } => {
                if remaining > 1 {
                    let next = remaining - 1;
                    self.stage = QuestStage::GatherRiverHerbs { remaining: next };
                    Some(format!("【药引】清心药香聚起，还差 {next} 份。"))
                } else {
                    self.add_key_item(KeyItem::HerbBundle);
                    self.stage = QuestStage::ReturnToHerbHealer;
                    Some("【药引】清心药引已齐，回药庐交给草药医。".to_string())
                }
            }
            QuestStage::CleansePlagueShrines { remaining } => {
                if remaining > 1 {
                    let next = remaining - 1;
                    self.stage = QuestStage::CleansePlagueShrines { remaining: next };
                    Some(format!("【净瘴】铃声清了一分，还剩 {next} 处瘴源。"))
                } else {
                    self.plague_wards = 0;
                    self.stage = QuestStage::SealPlagueWards;
                    Some("【净瘴】三处瘴源已散，去旧祠、苦井、病屋巡回净瘴铃位。".to_string())
                }
            }
            QuestStage::GatherSecretLetters { remaining } => {
                if remaining > 1 {
                    let next = remaining - 1;
                    self.stage = QuestStage::GatherSecretLetters { remaining: next };
                    Some(format!("【潜入】取回一份府邸密札，还差 {next} 份。"))
                } else {
                    self.add_key_item(KeyItem::SecretLetters);
                    self.mansion_mirrors = 0;
                    self.stage = QuestStage::AlignMansionMirrors;
                    Some("【潜入】两份府邸密札已齐，去照影镜廊对照账镜与证镜。".to_string())
                }
            }
            QuestStage::CleanseSpiritTotems { remaining } => {
                if remaining > 1 {
                    let next = remaining - 1;
                    self.stage = QuestStage::CleanseSpiritTotems { remaining: next };
                    Some(format!("【图腾】雷声收束，还剩 {next} 座图腾。"))
                } else {
                    self.thunder_drums = 0;
                    self.stage = QuestStage::AlignThunderDrums;
                    Some("【图腾】三座雷图腾已安抚，去雷鼓祭道巡回风鼓、云鼓、誓鼓。".to_string())
                }
            }
            _ => None,
        }
    }

    pub fn record_boss_victory(&mut self, boss: BossKind) -> Option<String> {
        match (boss, self.stage) {
            (BossKind::MoonWraith, QuestStage::ConfrontMoonWraith) => {
                self.add_key_item(KeyItem::MoonSeal);
                self.stage = QuestStage::ReturnToLinger;
                Some("【Boss】月魄妖退散，获得月魄印。回村找赵灵儿。".to_string())
            }
            (BossKind::RiverDemon, QuestStage::ConfrontRiverDemon) => {
                self.completed += 1;
                self.add_key_item(KeyItem::RiverPearl);
                self.stage = QuestStage::RiverTownComplete;
                Some("【Boss】河魇蛟沉入江心，获得河心珠。江岸小镇暂得安宁。".to_string())
            }
            (BossKind::MiasmaRoot, QuestStage::ConfrontMiasmaRoot) => {
                self.completed += 1;
                self.add_key_item(KeyItem::CureCharm);
                self.stage = QuestStage::PlagueVillageComplete;
                Some("【Boss】瘴母根断裂，获得解瘴符。瘴雨村的雨终于清了。".to_string())
            }
            (BossKind::MirrorMinister, QuestStage::ConfrontMirrorMinister) => {
                self.completed += 1;
                self.add_key_item(KeyItem::MirrorSeal);
                self.stage = QuestStage::CapitalIntrigueComplete;
                Some("【Boss】照影国师镜阵碎裂，获得照影印。云都府邸暗案暂告一段落。".to_string())
            }
            (BossKind::ThunderQilin, QuestStage::ConfrontThunderQilin) => {
                self.completed += 1;
                self.add_key_item(KeyItem::QilinHorn);
                self.stage = QuestStage::SouthernRoadComplete;
                Some("【Boss】雷麟收起天雷，获得雷麟角。南疆灵道重新亮起。".to_string())
            }
            (BossKind::DreamEclipse, QuestStage::ConfrontDreamEclipse) => {
                self.completed += 1;
                self.add_key_item(KeyItem::FateSeal);
                self.stage = QuestStage::FinaleComplete;
                Some("【Boss】宿命水影散入灵渊，获得宿命印。终门后的水声终于平息。".to_string())
            }
            _ => None,
        }
    }

    pub fn marker_for(&self, role: Option<QuestRole>) -> &'static str {
        match (role, self.stage) {
            (Some(QuestRole::SwordSister), QuestStage::NotStarted | QuestStage::ReturnToSister) => {
                "!"
            }
            (Some(QuestRole::Linger), QuestStage::TalkToLinger | QuestStage::ReturnToLinger) => "!",
            (Some(QuestRole::StarMage), QuestStage::FindStarMage) => "!",
            (Some(QuestRole::StarMage), QuestStage::DefeatMonsters { .. }) => "*",
            (Some(QuestRole::Merchant), QuestStage::EscortMerchant) => "!",
            (Some(QuestRole::BambooScout), QuestStage::FindBambooScout) => "!",
            (Some(QuestRole::CavePriestess), QuestStage::SeekCavePriestess) => "!",
            (Some(QuestRole::CavePriestess), QuestStage::CaveTrial { .. }) => "*",
            (Some(QuestRole::CavePriestess), QuestStage::ConfrontMoonWraith) => "!",
            (
                Some(QuestRole::HerbHealer),
                QuestStage::OpeningComplete | QuestStage::ReturnToHerbHealer,
            ) => "!",
            (Some(QuestRole::HerbHealer), QuestStage::GatherRiverHerbs { .. }) => "*",
            (
                Some(QuestRole::RiverBoatman),
                QuestStage::FindRiverBoatman | QuestStage::ConfrontRiverDemon,
            ) => "!",
            (Some(QuestRole::RiverBoatman), QuestStage::TuneRiverLanterns) => "*",
            (
                Some(QuestRole::PlagueElder),
                QuestStage::RiverTownComplete | QuestStage::SeekPlagueElder,
            ) => "!",
            (
                Some(QuestRole::PlagueElder),
                QuestStage::CleansePlagueShrines { .. } | QuestStage::SealPlagueWards,
            ) => "*",
            (
                Some(QuestRole::ShrineKeeper),
                QuestStage::SeekShrineKeeper
                | QuestStage::ReturnToShrineKeeper
                | QuestStage::ConfrontMiasmaRoot,
            ) => "!",
            (
                Some(QuestRole::ShrineKeeper),
                QuestStage::CleansePlagueShrines { .. } | QuestStage::SealPlagueWards,
            ) => "*",
            (
                Some(QuestRole::CapitalEnvoy),
                QuestStage::PlagueVillageComplete | QuestStage::SeekCapitalEnvoy,
            ) => "!",
            (Some(QuestRole::CapitalEnvoy), QuestStage::GatherSecretLetters { .. }) => "*",
            (
                Some(QuestRole::MansionSpy),
                QuestStage::FindMansionSpy
                | QuestStage::AlignMansionMirrors
                | QuestStage::ReturnToMansionSpy
                | QuestStage::ConfrontMirrorMinister,
            ) => "!",
            (Some(QuestRole::MansionSpy), QuestStage::GatherSecretLetters { .. }) => "*",
            (
                Some(QuestRole::SpiritGuide),
                QuestStage::CapitalIntrigueComplete | QuestStage::SeekSpiritGuide,
            ) => "!",
            (
                Some(QuestRole::TribalChief),
                QuestStage::SeekTribalChief
                | QuestStage::AlignThunderDrums
                | QuestStage::ReturnToTribalChief
                | QuestStage::ConfrontThunderQilin,
            ) => "!",
            (Some(QuestRole::TribalChief), QuestStage::CleanseSpiritTotems { .. }) => "*",
            (
                Some(QuestRole::FinalOracle),
                QuestStage::SouthernRoadComplete
                | QuestStage::SeekFinalOracle
                | QuestStage::ReturnToFinalOracle
                | QuestStage::ConfrontDreamEclipse,
            ) => "!",
            (Some(QuestRole::FinalOracle), QuestStage::FinaleComplete) => {
                if self.has_key_item(KeyItem::HomecomingSeal) {
                    "✓"
                } else {
                    "!"
                }
            }
            (Some(QuestRole::FinalOracle), QuestStage::LightFinalSoulLamps { .. }) => "*",
            (Some(_), _) => "?",
            (None, _) => "?",
        }
    }

    fn add_key_item(&mut self, item: KeyItem) {
        self.key_items |= key_item_bit(item);
    }

    fn add_companion(&mut self, companion: Companion) {
        self.companions |= companion_bit(companion);
    }
}

fn key_item_bit(item: KeyItem) -> u32 {
    match item {
        KeyItem::TrackingTalisman => 1 << 0,
        KeyItem::RoadPass => 1 << 1,
        KeyItem::MoonToken => 1 << 2,
        KeyItem::MoonSeal => 1 << 3,
        KeyItem::HerbPrescription => 1 << 4,
        KeyItem::HerbBundle => 1 << 5,
        KeyItem::FerryToken => 1 << 6,
        KeyItem::RiverPearl => 1 << 7,
        KeyItem::PlagueReport => 1 << 8,
        KeyItem::ShrineBell => 1 << 9,
        KeyItem::ShrineAsh => 1 << 10,
        KeyItem::CureCharm => 1 << 11,
        KeyItem::CapitalWrit => 1 << 12,
        KeyItem::CipherSlip => 1 << 13,
        KeyItem::SecretLetters => 1 << 14,
        KeyItem::MirrorSeal => 1 << 15,
        KeyItem::SpiritRoadPass => 1 << 16,
        KeyItem::TotemCharm => 1 << 17,
        KeyItem::StormGlyph => 1 << 18,
        KeyItem::QilinHorn => 1 << 19,
        KeyItem::FinalGateSigil => 1 << 20,
        KeyItem::DreamPearl => 1 << 21,
        KeyItem::FateSeal => 1 << 22,
        KeyItem::HomecomingSeal => 1 << 23,
    }
}

impl FinalLamp {
    pub fn name(self) -> &'static str {
        match self {
            FinalLamp::Memory => "忆灯",
            FinalLamp::Vow => "誓灯",
            FinalLamp::Fate => "命灯",
        }
    }
}

impl MoonCrystal {
    fn name(self) -> &'static str {
        match self {
            MoonCrystal::North => "上弦晶",
            MoonCrystal::South => "下弦晶",
        }
    }
}

impl RiverLantern {
    fn name(self) -> &'static str {
        match self {
            RiverLantern::Upstream => "上游灯",
            RiverLantern::Midstream => "中洲灯",
            RiverLantern::Dock => "渡口灯",
        }
    }
}

impl PlagueWard {
    fn name(self) -> &'static str {
        match self {
            PlagueWard::OldShrine => "旧祠铃",
            PlagueWard::BitterWell => "苦井铃",
            PlagueWard::Sickroom => "病屋铃",
        }
    }
}

impl MansionMirrorNode {
    fn name(self) -> &'static str {
        match self {
            MansionMirrorNode::Ledger => "账镜",
            MansionMirrorNode::Witness => "证镜",
        }
    }
}

impl ThunderDrum {
    fn name(self) -> &'static str {
        match self {
            ThunderDrum::Wind => "风鼓",
            ThunderDrum::Cloud => "云鼓",
            ThunderDrum::Oath => "誓鼓",
        }
    }
}

fn final_lamp_bit(lamp: FinalLamp) -> u32 {
    match lamp {
        FinalLamp::Memory => 1 << 0,
        FinalLamp::Vow => 1 << 1,
        FinalLamp::Fate => 1 << 2,
    }
}

fn moon_crystal_bit(crystal: MoonCrystal) -> u32 {
    match crystal {
        MoonCrystal::North => 1 << 0,
        MoonCrystal::South => 1 << 1,
    }
}

fn river_lantern_bit(lantern: RiverLantern) -> u32 {
    match lantern {
        RiverLantern::Upstream => 1 << 0,
        RiverLantern::Midstream => 1 << 1,
        RiverLantern::Dock => 1 << 2,
    }
}

fn plague_ward_bit(ward: PlagueWard) -> u32 {
    match ward {
        PlagueWard::OldShrine => 1 << 0,
        PlagueWard::BitterWell => 1 << 1,
        PlagueWard::Sickroom => 1 << 2,
    }
}

fn mansion_mirror_bit(node: MansionMirrorNode) -> u32 {
    match node {
        MansionMirrorNode::Ledger => 1 << 0,
        MansionMirrorNode::Witness => 1 << 1,
    }
}

fn thunder_drum_bit(drum: ThunderDrum) -> u32 {
    match drum {
        ThunderDrum::Wind => 1 << 0,
        ThunderDrum::Cloud => 1 << 1,
        ThunderDrum::Oath => 1 << 2,
    }
}

impl Chapter {
    fn index(self) -> usize {
        match self {
            Chapter::VillageOath => 0,
            Chapter::MoonCave => 1,
            Chapter::RiverMedicine => 2,
            Chapter::PlagueRain => 3,
            Chapter::CapitalMirror => 4,
            Chapter::SouthernThunder => 5,
            Chapter::FinalDream => 6,
        }
    }

    fn from_stage(stage: QuestStage) -> Self {
        match stage {
            QuestStage::NotStarted
            | QuestStage::TalkToLinger
            | QuestStage::FindStarMage
            | QuestStage::DefeatMonsters { .. }
            | QuestStage::ReturnToSister
            | QuestStage::EscortMerchant
            | QuestStage::FindBambooScout => Chapter::VillageOath,
            QuestStage::SeekCavePriestess
            | QuestStage::CaveTrial { .. }
            | QuestStage::ConfrontMoonWraith
            | QuestStage::ReturnToLinger => Chapter::MoonCave,
            QuestStage::OpeningComplete
            | QuestStage::GatherRiverHerbs { .. }
            | QuestStage::ReturnToHerbHealer
            | QuestStage::FindRiverBoatman
            | QuestStage::TuneRiverLanterns
            | QuestStage::ConfrontRiverDemon
            | QuestStage::RiverTownComplete => Chapter::RiverMedicine,
            QuestStage::SeekPlagueElder
            | QuestStage::SeekShrineKeeper
            | QuestStage::CleansePlagueShrines { .. }
            | QuestStage::SealPlagueWards
            | QuestStage::ReturnToShrineKeeper
            | QuestStage::ConfrontMiasmaRoot
            | QuestStage::PlagueVillageComplete => Chapter::PlagueRain,
            QuestStage::SeekCapitalEnvoy
            | QuestStage::FindMansionSpy
            | QuestStage::GatherSecretLetters { .. }
            | QuestStage::AlignMansionMirrors
            | QuestStage::ReturnToMansionSpy
            | QuestStage::ConfrontMirrorMinister
            | QuestStage::CapitalIntrigueComplete => Chapter::CapitalMirror,
            QuestStage::SeekSpiritGuide
            | QuestStage::SeekTribalChief
            | QuestStage::CleanseSpiritTotems { .. }
            | QuestStage::AlignThunderDrums
            | QuestStage::ReturnToTribalChief
            | QuestStage::ConfrontThunderQilin
            | QuestStage::SouthernRoadComplete => Chapter::SouthernThunder,
            QuestStage::SeekFinalOracle
            | QuestStage::LightFinalSoulLamps { .. }
            | QuestStage::ReturnToFinalOracle
            | QuestStage::ConfrontDreamEclipse
            | QuestStage::FinaleComplete => Chapter::FinalDream,
        }
    }

    fn title(self) -> &'static str {
        match self {
            Chapter::VillageOath => "第一卷 村誓与灵符",
            Chapter::MoonCave => "第二卷 水月洞天",
            Chapter::RiverMedicine => "第三卷 江岸药庐",
            Chapter::PlagueRain => "第四卷 瘴雨村",
            Chapter::CapitalMirror => "第五卷 云都府邸",
            Chapter::SouthernThunder => "第六卷 南疆灵道",
            Chapter::FinalDream => "终章 灵渊宿梦",
        }
    }

    fn summary(self) -> &'static str {
        match self {
            Chapter::VillageOath => {
                "山路妖雾初起，失踪灵符把李逍遥、赵灵儿与红衣剑姊牵到同一条路上。"
            }
            Chapter::MoonCave => "水月洞天开门，洞中试炼会逼你在妖气、晶阵和心事之间找答案。",
            Chapter::RiverMedicine => "江岸河雾带毒，药庐、码头与倒流河灯指向水下妖影。",
            Chapter::PlagueRain => "瘴雨村病声不断，病簿、祠灰与旧铃会把瘴源逼出地面。",
            Chapter::CapitalMirror => "云都府城灯火太亮，照影府邸的镜阵把证词与人心一并吞下。",
            Chapter::SouthernThunder => "南疆灵道雷声乱走，百越图腾与雷麟会考验队伍的誓言。",
            Chapter::FinalDream => "灵渊终门映出旧梦，三盏灯会问清你们究竟要认命还是改命。",
        }
    }

    fn play_hint(self) -> &'static str {
        match self {
            Chapter::VillageOath => {
                "【卷章玩法】先找人接线索，踏入草丛遭遇妖兽，回村交付并打开山路。"
            }
            Chapter::MoonCave => "【卷章玩法】净化妖气、点亮两座晶阵，再回祭司处触发首领战。",
            Chapter::RiverMedicine => {
                "【卷章玩法】收集药引、查问摆渡人，留意江岸任务板与河灯旁的支线。"
            }
            Chapter::PlagueRain => "【卷章玩法】在黑雨草地清瘴，利用休整和供奉撑过更长的村中战线。",
            Chapter::CapitalMirror => "【卷章玩法】取得入城符后潜入府邸，收集密札并拆穿镜阵。",
            Chapter::SouthernThunder => "【卷章玩法】安抚三座雷图腾，准备好营地加成再迎战雷麟。",
            Chapter::FinalDream => "【卷章玩法】点亮三盏忆梦灯，完成最终同行剧情后迎战宿命水影。",
        }
    }
}

fn chapter_bit(chapter: Chapter) -> u32 {
    debug_assert!(chapter.index() < CHAPTER_COUNT);
    1 << chapter.index()
}

impl BondScene {
    fn index(self) -> usize {
        match self {
            BondScene::VillageFirstNight => 0,
            BondScene::MoonCavePromise => 1,
            BondScene::RiverLampWish => 2,
            BondScene::PlagueRainShelter => 3,
            BondScene::CapitalRooftop => 4,
            BondScene::SouthernRoadOath => 5,
            BondScene::FinalGateQuiet => 6,
        }
    }

    fn title(self) -> &'static str {
        match self {
            BondScene::VillageFirstNight => "村郊夜谈",
            BondScene::MoonCavePromise => "水月誓言",
            BondScene::RiverLampWish => "河灯心愿",
            BondScene::PlagueRainShelter => "瘴雨伞下",
            BondScene::CapitalRooftop => "云都屋脊",
            BondScene::SouthernRoadOath => "南疆风誓",
            BondScene::FinalGateQuiet => "终门静水",
        }
    }

    fn reward(self) -> BondReward {
        match self {
            BondScene::VillageFirstNight => BondReward {
                exp: 18,
                potions: 1,
                full_restore: true,
            },
            BondScene::MoonCavePromise => BondReward {
                exp: 26,
                potions: 1,
                full_restore: true,
            },
            BondScene::RiverLampWish => BondReward {
                exp: 34,
                potions: 1,
                full_restore: true,
            },
            BondScene::PlagueRainShelter => BondReward {
                exp: 44,
                potions: 2,
                full_restore: true,
            },
            BondScene::CapitalRooftop => BondReward {
                exp: 54,
                potions: 1,
                full_restore: true,
            },
            BondScene::SouthernRoadOath => BondReward {
                exp: 64,
                potions: 2,
                full_restore: true,
            },
            BondScene::FinalGateQuiet => BondReward {
                exp: 80,
                potions: 2,
                full_restore: true,
            },
        }
    }

    fn lines(self) -> &'static [&'static str] {
        match self {
            BondScene::VillageFirstNight => &[
                "赵灵儿：村外的灯真小，却能把山路照出一个方向。",
                "李逍遥：那我就负责走前面，你负责提醒我别逞强。",
                "赵灵儿：说好了，别把这句话只当玩笑。",
            ],
            BondScene::MoonCavePromise => &[
                "赵灵儿：水月洞天像一面镜子，照得人不敢说谎。",
                "李逍遥：那我说真话，我不想让你一个人回头。",
                "赵灵儿：我也不想。",
            ],
            BondScene::RiverLampWish => &[
                "赵灵儿：河灯若能带走愿望，我想让江岸的人都睡个安稳觉。",
                "李逍遥：那我多点一盏，愿你路上少些担心。",
                "赵灵儿：逍遥哥哥的愿望，听起来也不轻。",
            ],
            BondScene::PlagueRainShelter => &[
                "赵灵儿：瘴雨落在伞上，像有人一直在哭。",
                "李逍遥：等病源断了，这村子总会有晴天。",
                "赵灵儿：那我们就撑到晴天。",
            ],
            BondScene::CapitalRooftop => &[
                "赵灵儿：云都灯火太亮，反而看不见谁在害怕。",
                "李逍遥：屋脊高，看得远，也容易摔。",
                "赵灵儿：我会拉住你，你也要拉住我。",
            ],
            BondScene::SouthernRoadOath => &[
                "赵灵儿：南疆风里有很多人的誓言。",
                "李逍遥：那我们也留一句，走到终门也不放手。",
                "赵灵儿：风会记得，我也会。",
            ],
            BondScene::FinalGateQuiet => &[
                "赵灵儿：终门前的水太静，像在等我们先开口。",
                "李逍遥：那就告诉它，我们不是来认命的。",
                "赵灵儿：嗯，我们是来把人带回去的。",
            ],
        }
    }

    fn repeat_line(self) -> &'static str {
        match self {
            BondScene::VillageFirstNight => "村郊灵灯仍亮着，山路比刚才安静些。",
            BondScene::MoonCavePromise => "洞天水声温和下来，像替你们守着旧誓。",
            BondScene::RiverLampWish => "河灯慢慢顺流，愿望已经放进水里。",
            BondScene::PlagueRainShelter => "伞下还有余温，雨声不再那么刺耳。",
            BondScene::CapitalRooftop => "云都屋脊风大，但脚下已稳。",
            BondScene::SouthernRoadOath => "南疆风声掠过，誓言没有散。",
            BondScene::FinalGateQuiet => "终门静水照着你们，灯光没有熄。",
        }
    }
}

fn bond_scene_bit(scene: BondScene) -> u32 {
    1 << scene.index()
}

fn bond_bonus_bit(bonus: BondBonus) -> u32 {
    match bonus {
        BondBonus::Courage => 1 << 0,
        BondBonus::Tender => 1 << 1,
    }
}

fn bond_bonus_from_bits(bits: u32) -> Option<BondBonus> {
    if bits & bond_bonus_bit(BondBonus::Courage) != 0 {
        Some(BondBonus::Courage)
    } else if bits & bond_bonus_bit(BondBonus::Tender) != 0 {
        Some(BondBonus::Tender)
    } else {
        None
    }
}

impl CampScene {
    fn index(self) -> usize {
        match self {
            CampScene::VillageHearth => 0,
            CampScene::MoonCavePool => 1,
            CampScene::RiverTownInn => 2,
            CampScene::PlagueSickroom => 3,
            CampScene::CapitalSafehouse => 4,
            CampScene::SouthernCampfire => 5,
            CampScene::FinalStillWater => 6,
        }
    }

    fn title(self) -> &'static str {
        match self {
            CampScene::VillageHearth => "余杭家灯",
            CampScene::MoonCavePool => "水月静池",
            CampScene::RiverTownInn => "江岸夜汤",
            CampScene::PlagueSickroom => "病屋守夜",
            CampScene::CapitalSafehouse => "云都暗灯",
            CampScene::SouthernCampfire => "南疆篝火",
            CampScene::FinalStillWater => "终门静坐",
        }
    }

    fn bonus(self) -> CampBonus {
        match self {
            CampScene::VillageHearth | CampScene::RiverTownInn | CampScene::FinalStillWater => {
                CampBonus::Warmth
            }
            CampScene::MoonCavePool | CampScene::CapitalSafehouse => CampBonus::Focus,
            CampScene::PlagueSickroom | CampScene::SouthernCampfire => CampBonus::Vigil,
        }
    }

    fn lines(self) -> &'static [&'static str] {
        match self {
            CampScene::VillageHearth => &[
                "婆婆把热饭摆在灯下，叮嘱你别只顾逞强。",
                "赵灵儿低声道谢，说这盏家灯比山风暖得多。",
                "林月衡把剑靠在门边，第一次没有催你立刻出发。",
            ],
            CampScene::MoonCavePool => &[
                "水月池边无风，月光却在剑刃上轻轻晃动。",
                "赵灵儿说：这里会照出心事，也会照出破绽。",
                "你们把明日要走的晶路重新记了一遍。",
            ],
            CampScene::RiverTownInn => &[
                "客栈掌柜端来热汤，江雾被门帘挡在外头。",
                "赵灵儿替药包重新系绳，林月衡检查窗外河灯。",
                "夜深以后，船板下的水声也显得没那么近。",
            ],
            CampScene::PlagueSickroom => &[
                "药童睡在病屋角落，铃铛仍握在手心。",
                "赵灵儿把窗缝压紧，林月衡守在门口听雨。",
                "你们约好轮流守夜，瘴影若来，先别惊动病人。",
            ],
            CampScene::CapitalSafehouse => &[
                "红衣密探吹灭外灯，只留桌上一盏小火。",
                "林月衡把密札按顺序排开，赵灵儿记下镜阵的缺口。",
                "城中鼓声过三更时，你们终于看清国师的下一步。",
            ],
            CampScene::SouthernCampfire => &[
                "祭草医把雷草投入火中，火星沿着风声向上跳。",
                "百越族长说，古路认得脚步，也认得守夜的人。",
                "你们围着篝火坐到雷声变远，才重新握紧兵刃。",
            ],
            CampScene::FinalStillWater => &[
                "终门水面安静得像一面镜子，却没有照出退路。",
                "赵灵儿说：若宿命要我们低头，我们就把灯举高些。",
                "林月衡没有说话，只把剑放在你伸手就能拿到的地方。",
            ],
        }
    }

    fn repeat_line(self) -> &'static str {
        match self {
            CampScene::VillageHearth => "家灯仍亮，饭香留在门槛边。",
            CampScene::MoonCavePool => "静池月光安稳，已经替你们照过前路。",
            CampScene::RiverTownInn => "热汤已经喝过，江风暂时吹不进屋。",
            CampScene::PlagueSickroom => "病屋守夜已过，铃声不再发抖。",
            CampScene::CapitalSafehouse => "暗灯下的密札已经排清，下一步该进府邸。",
            CampScene::SouthernCampfire => "篝火余烬还热，雷声已退到山外。",
            CampScene::FinalStillWater => "静水记住了你们的话，没有再催促。",
        }
    }
}

impl CampBonus {
    pub fn name(self) -> &'static str {
        match self {
            CampBonus::Warmth => "同伴余温",
            CampBonus::Focus => "静心看破",
            CampBonus::Vigil => "轮值守夜",
        }
    }

    pub fn tactic_label(self) -> &'static str {
        match self {
            CampBonus::Warmth => "围坐调息",
            CampBonus::Focus => "月衡破势",
            CampBonus::Vigil => "灵儿守护",
        }
    }

    pub fn tactic_line(self) -> &'static str {
        match self {
            CampBonus::Warmth => "三人围坐调息，下一战攻守都有余温照应。",
            CampBonus::Focus => "林月衡拆出敌势破口，下一战更利于强攻与合击。",
            CampBonus::Vigil => "赵灵儿守住伤势与风声，下一战可预先抵消来袭。",
        }
    }

    pub fn battle_line(self) -> &'static str {
        match self {
            CampBonus::Warmth => "下一场战斗攻击与仙术略增，受击也会少受一点伤害。",
            CampBonus::Focus => "下一场战斗攻击、仙术和合击都会追加伤害。",
            CampBonus::Vigil => "下一场战斗受到的伤害会被同伴预警抵消。",
        }
    }
}

fn camp_scene_bit(scene: CampScene) -> u32 {
    1 << scene.index()
}

fn camp_bonus_bit(bonus: CampBonus) -> u32 {
    match bonus {
        CampBonus::Warmth => 1 << 0,
        CampBonus::Focus => 1 << 1,
        CampBonus::Vigil => 1 << 2,
    }
}

fn camp_bonus_from_bits(bits: u32) -> Option<CampBonus> {
    if bits & camp_bonus_bit(CampBonus::Warmth) != 0 {
        Some(CampBonus::Warmth)
    } else if bits & camp_bonus_bit(CampBonus::Focus) != 0 {
        Some(CampBonus::Focus)
    } else if bits & camp_bonus_bit(CampBonus::Vigil) != 0 {
        Some(CampBonus::Vigil)
    } else {
        None
    }
}

impl TreasureCache {
    fn index(self) -> usize {
        match self {
            TreasureCache::VillageShrine => 0,
            TreasureCache::BambooOffering => 1,
            TreasureCache::CaveOffering => 2,
            TreasureCache::RiverTownCrystal => 3,
            TreasureCache::RiverReedCrystal => 4,
            TreasureCache::PlagueShrine => 5,
            TreasureCache::CapitalShrine => 6,
            TreasureCache::MansionMirror => 7,
            TreasureCache::SouthernTotem => 8,
            TreasureCache::FinalMemoryCache => 9,
        }
    }

    fn name(self) -> &'static str {
        match self {
            TreasureCache::VillageShrine => "村郊旧像",
            TreasureCache::BambooOffering => "竹径供龛",
            TreasureCache::CaveOffering => "水月祭台",
            TreasureCache::RiverTownCrystal => "江岸灵晶",
            TreasureCache::RiverReedCrystal => "芦滩水晶",
            TreasureCache::PlagueShrine => "瘴雨祠灰",
            TreasureCache::CapitalShrine => "府城香案",
            TreasureCache::MansionMirror => "照影暗匣",
            TreasureCache::SouthernTotem => "南疆图腾座",
            TreasureCache::FinalMemoryCache => "终门旧梦匣",
        }
    }

    fn reward(self) -> TreasureReward {
        match self {
            TreasureCache::VillageShrine => TreasureReward {
                exp: 10,
                potions: 1,
                gold: 8,
            },
            TreasureCache::BambooOffering => TreasureReward {
                exp: 14,
                potions: 1,
                gold: 12,
            },
            TreasureCache::CaveOffering => TreasureReward {
                exp: 18,
                potions: 1,
                gold: 18,
            },
            TreasureCache::RiverTownCrystal => TreasureReward {
                exp: 22,
                potions: 1,
                gold: 24,
            },
            TreasureCache::RiverReedCrystal => TreasureReward {
                exp: 24,
                potions: 1,
                gold: 28,
            },
            TreasureCache::PlagueShrine => TreasureReward {
                exp: 30,
                potions: 2,
                gold: 32,
            },
            TreasureCache::CapitalShrine => TreasureReward {
                exp: 34,
                potions: 1,
                gold: 48,
            },
            TreasureCache::MansionMirror => TreasureReward {
                exp: 38,
                potions: 1,
                gold: 56,
            },
            TreasureCache::SouthernTotem => TreasureReward {
                exp: 44,
                potions: 2,
                gold: 62,
            },
            TreasureCache::FinalMemoryCache => TreasureReward {
                exp: 50,
                potions: 2,
                gold: 70,
            },
        }
    }

    fn open_line(self) -> &'static str {
        match self {
            TreasureCache::VillageShrine => "旧像底座松动，香灰下压着村人留下的护身小包。",
            TreasureCache::BambooOffering => "竹叶遮住一只小供龛，里面还有未干的灵露。",
            TreasureCache::CaveOffering => "祭台石缝泛起月光，露出一包祭司备用的药钱。",
            TreasureCache::RiverTownCrystal => "晶簇吸住一串河灯线，线尾绑着江岸人谢礼。",
            TreasureCache::RiverReedCrystal => "芦花下的水晶回应灵符，吐出一枚湿润贝袋。",
            TreasureCache::PlagueShrine => "祠灰里藏着救急药包，旁边压着几枚旧钱。",
            TreasureCache::CapitalShrine => "香案暗格弹开，府城香客留下的供钱还未被取走。",
            TreasureCache::MansionMirror => "镜座背后有个暗匣，里面塞着巡夜人私藏的药散。",
            TreasureCache::SouthernTotem => "图腾座底传来雷响，石缝里滚出百越药酒和铜钱。",
            TreasureCache::FinalMemoryCache => "旧梦匣在水光中打开，几件前人遗物仍有温度。",
        }
    }

    fn find_line(self) -> &'static str {
        match self {
            TreasureCache::VillageShrine => "护身药一份，八文钱。",
            TreasureCache::BambooOffering => "竹露药一份，十二文钱。",
            TreasureCache::CaveOffering => "月洞药一份，十八文钱。",
            TreasureCache::RiverTownCrystal => "江岸药一份，二十四文钱。",
            TreasureCache::RiverReedCrystal => "贝袋药一份，二十八文钱。",
            TreasureCache::PlagueShrine => "救急药两份，三十二文钱。",
            TreasureCache::CapitalShrine => "府城药一份，四十八文钱。",
            TreasureCache::MansionMirror => "暗匣药一份，五十六文钱。",
            TreasureCache::SouthernTotem => "百越药酒两份，六十二文钱。",
            TreasureCache::FinalMemoryCache => "前人灵药两份，七十文钱。",
        }
    }

    fn empty_line(self) -> &'static str {
        match self {
            TreasureCache::VillageShrine => "香灰已平，只剩淡淡草药味。",
            TreasureCache::BambooOffering => "供龛已经空了，竹叶重新盖住石面。",
            TreasureCache::CaveOffering => "祭台月光安静下来，没有新的回应。",
            TreasureCache::RiverTownCrystal => "晶簇里只剩河灯倒影。",
            TreasureCache::RiverReedCrystal => "水晶不再吐光，芦花随风摆动。",
            TreasureCache::PlagueShrine => "祠灰被你收拾干净，药包已经取走。",
            TreasureCache::CapitalShrine => "暗格合上，只留一线香气。",
            TreasureCache::MansionMirror => "镜座背面空空如也。",
            TreasureCache::SouthernTotem => "图腾座雷声平息，没有新东西滚出。",
            TreasureCache::FinalMemoryCache => "旧梦匣重新沉入水光。",
        }
    }
}

fn treasure_cache_bit(cache: TreasureCache) -> u32 {
    debug_assert!(cache.index() < TREASURE_CACHE_COUNT);
    1 << cache.index()
}

impl ShrineBlessing {
    pub fn name(self) -> &'static str {
        match self {
            ShrineBlessing::Guard => "护身香火",
            ShrineBlessing::Sword => "剑心香火",
            ShrineBlessing::Spirit => "灵息香火",
        }
    }

    pub fn battle_line(self) -> &'static str {
        match self {
            ShrineBlessing::Guard => "护身香火绕在身侧，下一场战斗受到的伤害会降低。",
            ShrineBlessing::Sword => "剑心香火附在剑锋，下一场战斗攻击会追加伤害。",
            ShrineBlessing::Spirit => "灵息香火沉入气海，下一场战斗仙术与合击会更强。",
        }
    }
}

fn shrine_blessing_bit(blessing: ShrineBlessing) -> u32 {
    match blessing {
        ShrineBlessing::Guard => 1 << 0,
        ShrineBlessing::Sword => 1 << 1,
        ShrineBlessing::Spirit => 1 << 2,
    }
}

fn shrine_blessing_from_bits(bits: u32) -> Option<ShrineBlessing> {
    if bits & shrine_blessing_bit(ShrineBlessing::Guard) != 0 {
        Some(ShrineBlessing::Guard)
    } else if bits & shrine_blessing_bit(ShrineBlessing::Sword) != 0 {
        Some(ShrineBlessing::Sword)
    } else if bits & shrine_blessing_bit(ShrineBlessing::Spirit) != 0 {
        Some(ShrineBlessing::Spirit)
    } else {
        None
    }
}

impl SideQuest {
    fn index(self) -> usize {
        match self {
            SideQuest::VillageTrail => 0,
            SideQuest::VillageHerbs => 1,
            SideQuest::MoonCaveCrystals => 2,
            SideQuest::MoonCaveEchoes => 3,
            SideQuest::RiverLanterns => 4,
            SideQuest::RiverCargo => 5,
            SideQuest::PlagueRelief => 6,
            SideQuest::PlagueMedicine => 7,
            SideQuest::CapitalPatrol => 8,
            SideQuest::CapitalRumors => 9,
            SideQuest::SouthernThunder => 10,
            SideQuest::SouthernDrums => 11,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            SideQuest::VillageTrail => "山路余妖",
            SideQuest::VillageHerbs => "药圃护路",
            SideQuest::MoonCaveCrystals => "水月晶尘",
            SideQuest::MoonCaveEchoes => "回声妖影",
            SideQuest::RiverLanterns => "河灯巡夜",
            SideQuest::RiverCargo => "湿货寻踪",
            SideQuest::PlagueRelief => "瘴雨救急",
            SideQuest::PlagueMedicine => "药童护送",
            SideQuest::CapitalPatrol => "府邸巡查",
            SideQuest::CapitalRumors => "暗帖追查",
            SideQuest::SouthernThunder => "灵道巡雷",
            SideQuest::SouthernDrums => "战鼓安魂",
        }
    }

    fn goal(self) -> u32 {
        match self {
            SideQuest::VillageTrail => 2,
            SideQuest::VillageHerbs => 2,
            SideQuest::MoonCaveCrystals => 2,
            SideQuest::MoonCaveEchoes => 3,
            SideQuest::RiverLanterns => 2,
            SideQuest::RiverCargo => 2,
            SideQuest::PlagueRelief => 3,
            SideQuest::PlagueMedicine => 2,
            SideQuest::CapitalPatrol => 2,
            SideQuest::CapitalRumors => 3,
            SideQuest::SouthernThunder => 3,
            SideQuest::SouthernDrums => 2,
        }
    }

    fn reward(self) -> SideQuestReward {
        match self {
            SideQuest::VillageTrail => SideQuestReward {
                exp: 24,
                potions: 1,
                gold: 18,
            },
            SideQuest::VillageHerbs => SideQuestReward {
                exp: 28,
                potions: 2,
                gold: 16,
            },
            SideQuest::MoonCaveCrystals => SideQuestReward {
                exp: 32,
                potions: 1,
                gold: 24,
            },
            SideQuest::MoonCaveEchoes => SideQuestReward {
                exp: 42,
                potions: 1,
                gold: 30,
            },
            SideQuest::RiverLanterns => SideQuestReward {
                exp: 40,
                potions: 2,
                gold: 32,
            },
            SideQuest::RiverCargo => SideQuestReward {
                exp: 46,
                potions: 1,
                gold: 40,
            },
            SideQuest::PlagueRelief => SideQuestReward {
                exp: 54,
                potions: 2,
                gold: 42,
            },
            SideQuest::PlagueMedicine => SideQuestReward {
                exp: 58,
                potions: 2,
                gold: 46,
            },
            SideQuest::CapitalPatrol => SideQuestReward {
                exp: 64,
                potions: 1,
                gold: 56,
            },
            SideQuest::CapitalRumors => SideQuestReward {
                exp: 72,
                potions: 1,
                gold: 66,
            },
            SideQuest::SouthernThunder => SideQuestReward {
                exp: 76,
                potions: 2,
                gold: 68,
            },
            SideQuest::SouthernDrums => SideQuestReward {
                exp: 84,
                potions: 2,
                gold: 74,
            },
        }
    }

    fn reward_text(self) -> String {
        let reward = self.reward();
        format!(
            "经验 +{} / 药水 +{} / 钱 +{}文",
            reward.exp, reward.potions, reward.gold
        )
    }

    fn objective(self) -> &'static str {
        match self {
            SideQuest::VillageTrail => "击退两只草丛妖兽，再回村郊任务板交付。",
            SideQuest::VillageHerbs => "护送药圃小路，击退两只闻香而来的妖兽。",
            SideQuest::MoonCaveCrystals => "净化两处洞天妖气，回水月洞天任务板交付。",
            SideQuest::MoonCaveEchoes => "压住三段回声妖影，再回水月洞天石牌交付。",
            SideQuest::RiverLanterns => "巡查两盏逆流河灯，回江岸任务板交付。",
            SideQuest::RiverCargo => "沿草滩追回两袋湿货，回江岸任务板交付。",
            SideQuest::PlagueRelief => "清掉三处瘴草妖影，回瘴雨村任务板交付。",
            SideQuest::PlagueMedicine => "护送药童穿过两段瘴路，回瘴雨村任务板交付。",
            SideQuest::CapitalPatrol => "击退两名镜阵幻影，回云都府城任务板交付。",
            SideQuest::CapitalRumors => "截下三张镜阵暗帖，回云都府城任务板交付。",
            SideQuest::SouthernThunder => "安抚三段灵道雷声，回南疆任务板交付。",
            SideQuest::SouthernDrums => "平息两处战鼓怨音，回南疆任务板交付。",
        }
    }

    fn area(self) -> &'static str {
        match self {
            SideQuest::VillageTrail => "余杭村郊外缘草丛",
            SideQuest::VillageHerbs => "余杭村药圃小路",
            SideQuest::MoonCaveCrystals => "水月洞天晶尘石道",
            SideQuest::MoonCaveEchoes => "水月洞天回声窄廊",
            SideQuest::RiverLanterns => "江岸小镇河灯浅滩",
            SideQuest::RiverCargo => "江岸草滩湿货残道",
            SideQuest::PlagueRelief => "瘴雨村黑雨草地",
            SideQuest::PlagueMedicine => "瘴雨村病屋药路",
            SideQuest::CapitalPatrol => "云都府城暗巷与府门",
            SideQuest::CapitalRumors => "云都府城茶肆暗帖处",
            SideQuest::SouthernThunder => "南疆灵道雷草坡",
            SideQuest::SouthernDrums => "南疆灵道旧鼓架",
        }
    }

    fn turn_in_place(self) -> &'static str {
        match self {
            SideQuest::VillageTrail | SideQuest::VillageHerbs => "回余杭村郊任务板交付。",
            SideQuest::MoonCaveCrystals | SideQuest::MoonCaveEchoes => "回水月洞天石牌交付。",
            SideQuest::RiverLanterns | SideQuest::RiverCargo => "回江岸小镇任务板交付。",
            SideQuest::PlagueRelief | SideQuest::PlagueMedicine => "回瘴雨村任务板交付。",
            SideQuest::CapitalPatrol | SideQuest::CapitalRumors => "回云都府城任务板交付。",
            SideQuest::SouthernThunder | SideQuest::SouthernDrums => "回南疆灵道任务板交付。",
        }
    }

    fn tracking_hint(self) -> &'static str {
        match self {
            SideQuest::VillageTrail => "踏入村郊草丛引出余妖，击退 2 只。",
            SideQuest::VillageHerbs => "沿药圃小路巡护，遇妖后击退 2 只。",
            SideQuest::MoonCaveCrystals => "在洞天晶路触发妖气，净化 2 处。",
            SideQuest::MoonCaveEchoes => "穿过回声窄廊，压住 3 段妖影。",
            SideQuest::RiverLanterns => "沿河灯浅滩巡夜，处理 2 处逆流灯影。",
            SideQuest::RiverCargo => "去江岸草滩找湿货，被妖风截住时击退 2 次。",
            SideQuest::PlagueRelief => "踏入黑雨草地，清掉 3 处瘴草妖影。",
            SideQuest::PlagueMedicine => "护送药童穿过病屋外路，处理 2 段瘴影。",
            SideQuest::CapitalPatrol => "巡查府城暗巷，击退 2 名镜阵幻影。",
            SideQuest::CapitalRumors => "追查茶肆暗帖，截下 3 张镜阵线索。",
            SideQuest::SouthernThunder => "沿雷草坡前进，安抚 3 段乱走雷声。",
            SideQuest::SouthernDrums => "靠近旧鼓架，平息 2 处战鼓怨音。",
        }
    }

    fn accept_line(self) -> &'static str {
        match self {
            SideQuest::VillageTrail => "告示写着：山路虽开，余妖未尽，愿助巡山者可取赏。",
            SideQuest::VillageHerbs => "药婆补上一张新签：药圃夜里有妖闻香而来，求护路人同行。",
            SideQuest::MoonCaveCrystals => "石牌留字：洞天晶尘混入妖气，净其二处可换灵药。",
            SideQuest::MoonCaveEchoes => "石牌背面又浮出小字：回声成魅，压住三段方能安眠。",
            SideQuest::RiverLanterns => "任务板贴着河灯图：夜灯逆流，巡两处便知妖影走向。",
            SideQuest::RiverCargo => "码头货签被水泡皱：湿货被妖风卷进草滩，追回者另有谢礼。",
            SideQuest::PlagueRelief => "村中急榜：瘴草缠屋，清三处者可领药钱。",
            SideQuest::PlagueMedicine => "药童把小铃挂到榜下：送药路上瘴影缠人，求少侠护一程。",
            SideQuest::CapitalPatrol => "府城密榜：镜阵余影夜巡暗廊，击退两名可换封赏。",
            SideQuest::CapitalRumors => {
                "暗榜露出第二层：镜阵暗帖正在城中传手，截下三张可换线报钱。"
            }
            SideQuest::SouthernThunder => "兽皮榜写着：灵道雷声乱走，安三段雷鸣者可取族中药酒。",
            SideQuest::SouthernDrums => "鼓架旁新刻了符文：旧战鼓夜里自鸣，平息两处可领祭酒。",
        }
    }

    fn progress_line(self) -> &'static str {
        match self {
            SideQuest::VillageTrail => "任务板旁的竹签还没移到交付位，继续去草丛引妖。",
            SideQuest::VillageHerbs => "药圃香气还在外泄，护路还没走完。",
            SideQuest::MoonCaveCrystals => "晶尘仍浑，洞中妖气还需再净。",
            SideQuest::MoonCaveEchoes => "水月洞里仍有回声撞墙，继续压住妖影。",
            SideQuest::RiverLanterns => "河灯图还缺一角亮色，继续巡查江岸草滩。",
            SideQuest::RiverCargo => "湿货签还少一角，草滩里还有妖风拖着货袋。",
            SideQuest::PlagueRelief => "救急榜上的药包还未备齐，继续清瘴。",
            SideQuest::PlagueMedicine => "药童铃声仍在发抖，送药路还没走完。",
            SideQuest::CapitalPatrol => "密榜暗号仍未变红，府邸阴影里还有幻影。",
            SideQuest::CapitalRumors => "暗帖还在坊间传手，继续截下镜阵线索。",
            SideQuest::SouthernThunder => "兽皮上的雷纹仍在乱跳，灵道还没安静。",
            SideQuest::SouthernDrums => "鼓声仍在远处回荡，怨音还没完全散去。",
        }
    }

    fn turn_in_line(self) -> &'static str {
        match self {
            SideQuest::VillageTrail => "任务板铃声一响，村人添上了赏钱和一瓶药。",
            SideQuest::VillageHerbs => "药婆把新采的醒神草扎好，悄悄多塞了一瓶药。",
            SideQuest::MoonCaveCrystals => "晶尘归匣，月洞的水光亮了一寸。",
            SideQuest::MoonCaveEchoes => "回声沉入石壁，洞天终于能听见自己的水声。",
            SideQuest::RiverLanterns => "河灯图补全，码头人递来药包和谢礼。",
            SideQuest::RiverCargo => "湿货晾上绳架，货主把压箱钱交给你。",
            SideQuest::PlagueRelief => "瘴草退开，村中药锅终于能分给病人。",
            SideQuest::PlagueMedicine => "药童铃声稳了下来，病屋门口有人低声道谢。",
            SideQuest::CapitalPatrol => "密榜封蜡融开，府中暗线送来封赏。",
            SideQuest::CapitalRumors => "暗帖烧成灰，内线把线报钱藏进空茶盏。",
            SideQuest::SouthernThunder => "雷纹归位，百越族人在任务板下放好药酒。",
            SideQuest::SouthernDrums => "战鼓安静，祭司把温好的药酒递到你手上。",
        }
    }

    fn completed_line(self) -> &'static str {
        match self {
            SideQuest::VillageTrail => "山路余妖已清，村郊暂时安全。",
            SideQuest::VillageHerbs => "药圃护路已毕，村里的夜药不会再断。",
            SideQuest::MoonCaveCrystals => "水月晶尘已净，洞天灵光稳定。",
            SideQuest::MoonCaveEchoes => "回声妖影已散，洞天夜里不再有人惊醒。",
            SideQuest::RiverLanterns => "河灯巡夜已结，江岸人心稍定。",
            SideQuest::RiverCargo => "湿货寻踪已结，码头的账簿少了一页坏账。",
            SideQuest::PlagueRelief => "救急药包已经送完，瘴雨村记下了这份情。",
            SideQuest::PlagueMedicine => "药童护送已毕，病屋药路重新通了。",
            SideQuest::CapitalPatrol => "府邸巡查已交，暗线不会再重复悬赏。",
            SideQuest::CapitalRumors => "暗帖追查已交，府城流言暂时断线。",
            SideQuest::SouthernThunder => "灵道巡雷已毕，雷纹不再乱走。",
            SideQuest::SouthernDrums => "战鼓安魂已毕，旧怨暂时沉入山风。",
        }
    }
}

fn side_quest_bit(side: SideQuest) -> u32 {
    1 << side.index()
}

fn companion_bit(companion: Companion) -> u32 {
    match companion {
        Companion::Linger => 1 << 0,
        Companion::SwordSister => 1 << 1,
    }
}
