use bevy::prelude::*;

const BASIC_SPELL_COST: i32 = 5;
const LATE_SPELL_COST: i32 = 9;
const SIDE_QUEST_COUNT: usize = 14;
const NPC_ERRAND_COUNT: usize = 8;
const CHAPTER_COUNT: usize = 7;
const BOND_SCENE_COUNT: usize = 7;
const CAMP_SCENE_COUNT: usize = 7;
const COMPANION_SCENE_COUNT: usize = 5;
const COMPANION_REVISIT_COUNT: usize = 3;
const TREASURE_CACHE_COUNT: usize = 10;
const FIELD_SUPPLY_COUNT: usize = 15;
const COMMISSION_TRACE_COUNT: usize = SIDE_QUEST_COUNT;
const SHOP_GEAR_COUNT: usize = 4;
const ROUTE_MARK_COUNT: usize = 6;
const ROUTE_DETOUR_COUNT: usize = 6;
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
    SideQuest::FinalDreamEchoes,
    SideQuest::FinalHomewardVows,
];
const ALL_NPC_ERRANDS: [NpcErrand; NPC_ERRAND_COUNT] = [
    NpcErrand::BambooDewToCave,
    NpcErrand::MoonMossToRiver,
    NpcErrand::RiverReedLetter,
    NpcErrand::PlagueChildCharm,
    NpcErrand::CapitalStarSlip,
    NpcErrand::MirrorMedicineToSouth,
    NpcErrand::SouthernThunderWine,
    NpcErrand::FinalLampWick,
];
const ALL_COMPANION_REVISITS: [CompanionRevisit; COMPANION_REVISIT_COUNT] = [
    CompanionRevisit::TrailEcho,
    CompanionRevisit::MirrorTrace,
    CompanionRevisit::TotemVow,
];
const ALL_CHAPTERS: [Chapter; CHAPTER_COUNT] = [
    Chapter::VillageOath,
    Chapter::MoonCave,
    Chapter::RiverMedicine,
    Chapter::PlagueRain,
    Chapter::CapitalMirror,
    Chapter::SouthernThunder,
    Chapter::FinalDream,
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
    SpiritWitch,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BossKind {
    MountainFiend,
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
pub enum RouteMark {
    MoonEcho,
    ReedFord,
    PlagueBell,
    MirrorSideDoor,
    ThunderSwitchback,
    DreamReturn,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RouteDetour {
    MoonEchoPool,
    ReedHiddenFord,
    PlagueHerbTrail,
    MirrorServantDoor,
    ThunderRidgeCache,
    DreamBackwater,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RouteDetourApproach {
    Scout,
    PressOn,
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
pub enum CompanionScene {
    SwordSisterTrailGuard,
    SwordSisterCapitalMirror,
    SpiritWitchSouthernTotem,
    SwordSisterFinalReturn,
    SpiritWitchFinalVow,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CompanionRevisit {
    TrailEcho,
    MirrorTrace,
    TotemVow,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompanionSceneReward {
    pub exp: u32,
    pub potions: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CompanionAftermathReward {
    pub exp: u32,
    pub potions: u32,
    pub gold: u32,
}

#[derive(Debug, PartialEq, Eq)]
pub struct CompanionSceneInteraction {
    pub lines: Vec<String>,
    pub reward: Option<CompanionSceneReward>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct CompanionRevisitTurnIn {
    pub lines: Vec<String>,
    pub reward: Option<CompanionAftermathReward>,
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
    FinalDreamEchoes,
    FinalHomewardVows,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum NpcErrand {
    BambooDewToCave,
    MoonMossToRiver,
    RiverReedLetter,
    PlagueChildCharm,
    CapitalStarSlip,
    MirrorMedicineToSouth,
    SouthernThunderWine,
    FinalLampWick,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SideQuestReward {
    pub exp: u32,
    pub potions: u32,
    pub gold: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NpcErrandReward {
    pub exp: u32,
    pub potions: u32,
    pub gold: u32,
    pub hp: i32,
    pub mp: i32,
}

#[derive(Debug, PartialEq, Eq)]
pub struct SideQuestInteraction {
    pub lines: Vec<String>,
    pub reward: Option<SideQuestReward>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct NpcErrandInteraction {
    pub lines: Vec<String>,
    pub reward: Option<NpcErrandReward>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SideQuestResolution {
    Settle,
    Pursue,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SideQuestFieldApproach {
    Investigate,
    Confront,
}

impl SideQuestResolution {
    pub fn name(self) -> &'static str {
        match self {
            Self::Settle => "稳妥封存",
            Self::Pursue => "追查余波",
        }
    }

    fn result_line(self) -> &'static str {
        match self {
            Self::Settle => "把现场证物封入委托签，交由当地人按旧约守住。",
            Self::Pursue => "不急着合上委托签，顺着余波再添一处回访路标。",
        }
    }
}

impl SideQuestFieldApproach {
    pub fn name(self) -> &'static str {
        match self {
            Self::Investigate => "细查现场",
            Self::Confront => "快断余妖",
        }
    }

    pub fn action_label(self) -> &'static str {
        match self {
            Self::Investigate => "细查现场",
            Self::Confront => "快断余妖",
        }
    }

    fn result_line(self, side: SideQuest) -> String {
        match self {
            Self::Investigate => format!(
                "放慢脚步查清{}的来路，委托签会留下回访线索。",
                side.party_focus()
            ),
            Self::Confront => format!(
                "趁{}还没合拢先断其势，委托签会记录一处速断法。",
                side.party_focus()
            ),
        }
    }

    fn contract_line(self, side: SideQuest) -> String {
        match self {
            Self::Investigate => format!(
                "{}：{}已细查，交付时可说明来龙去脉。",
                self.name(),
                side.party_focus()
            ),
            Self::Confront => format!(
                "{}：{}已快断，交付时可说明现场处置。",
                self.name(),
                side.party_focus()
            ),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SideQuestPartyMoment {
    Accept,
    Progress,
    TurnIn,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SideQuestPartyVoice {
    Hero,
    Linger,
    SwordSister,
    SpiritWitch,
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
pub enum FieldSupply {
    VillageHerbs,
    BambooDew,
    CaveMoonMoss,
    MoonCorridorDust,
    RiverTeaChest,
    ReedLotusPods,
    PlagueCleanWater,
    ShrineAshRoots,
    CapitalTeaPacket,
    MansionPantry,
    MirrorPowder,
    SouthernPepper,
    ThunderHerbWine,
    FinalIncense,
    DreamPearlMoss,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ShopGear {
    VillageSwordTassel,
    RiverSilkVest,
    CapitalMirrorGuard,
    SouthernThunderCharm,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct TreasureReward {
    pub exp: u32,
    pub potions: u32,
    pub gold: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SupplyReward {
    pub exp: u32,
    pub potions: u32,
    pub gold: u32,
    pub hp: i32,
    pub mp: i32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CareAftermathReward {
    pub exp: u32,
    pub potions: u32,
    pub gold: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct CommissionAftermathReward {
    pub exp: u32,
    pub potions: u32,
    pub gold: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RouteDetourReward {
    pub exp: u32,
    pub potions: u32,
    pub gold: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RouteDetourReportReward {
    pub exp: u32,
    pub potions: u32,
    pub gold: u32,
}

#[derive(Debug, PartialEq, Eq)]
pub struct TreasureInteraction {
    pub lines: Vec<String>,
    pub reward: Option<TreasureReward>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct SupplyInteraction {
    pub lines: Vec<String>,
    pub reward: Option<SupplyReward>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct RouteDetourResolution {
    pub lines: Vec<String>,
    pub reward: Option<RouteDetourReward>,
    pub tactic: Option<CampBonus>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct RouteDetourReport {
    pub lines: Vec<String>,
    pub reward: Option<RouteDetourReportReward>,
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MainTaskLedger {
    pub chapter: Chapter,
    pub step: u32,
    pub total: u32,
    pub action: &'static str,
    pub place: &'static str,
    pub contact: &'static str,
    pub receipt: &'static str,
}

impl MainTaskLedger {
    pub fn hud_label(self) -> String {
        format!(
            "{} {}/{} {}@{}",
            self.chapter.short_title(),
            self.step,
            self.total,
            self.action,
            self.place
        )
    }

    pub fn tracker_line(self) -> String {
        format!(
            "章程：{}/{} {} · {}\n路书：{} · 地点：{} · 联络：{}",
            self.step,
            self.total,
            self.action,
            self.chapter.title(),
            self.receipt,
            self.place,
            self.contact
        )
    }
}

#[derive(Resource, Debug)]
pub struct QuestLog {
    stage: QuestStage,
    completed: u32,
    chapter_seals: u32,
    key_items: u32,
    companions: u32,
    side_active: u32,
    side_completed: u32,
    side_resolution_pursue: u32,
    side_field_confront: u32,
    side_progress: [u32; SIDE_QUEST_COUNT],
    npc_errand_active: u32,
    npc_errand_completed: u32,
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
    companion_scenes: u32,
    companion_aftermath_claimed: u32,
    companion_revisits_active: u32,
    care_aftermath_claimed: u32,
    commission_aftermath_claimed: u32,
    treasure_opened: u32,
    field_supplies: u32,
    commission_traces: u32,
    shop_gear: u32,
    shrine_blessing: u32,
    route_marks: u32,
    route_detours: u32,
    route_detour_scouted: u32,
    route_detour_reports_claimed: u32,
}

impl Default for QuestLog {
    fn default() -> Self {
        Self {
            stage: QuestStage::NotStarted,
            completed: 0,
            chapter_seals: 0,
            key_items: 0,
            companions: 0,
            side_active: 0,
            side_completed: 0,
            side_resolution_pursue: 0,
            side_field_confront: 0,
            side_progress: [0; SIDE_QUEST_COUNT],
            npc_errand_active: 0,
            npc_errand_completed: 0,
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
            companion_scenes: 0,
            companion_aftermath_claimed: 0,
            companion_revisits_active: 0,
            care_aftermath_claimed: 0,
            commission_aftermath_claimed: 0,
            treasure_opened: 0,
            field_supplies: 0,
            commission_traces: 0,
            shop_gear: 0,
            shrine_blessing: 0,
            route_marks: 0,
            route_detours: 0,
            route_detour_scouted: 0,
            route_detour_reports_claimed: 0,
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
    fn main_task_ledger_tracks_route_book_across_chapters() {
        let mut quest = QuestLog::default();
        let ledger = quest.main_task_ledger();
        assert_eq!(ledger.chapter, Chapter::VillageOath);
        assert_eq!((ledger.step, ledger.total), (1, 7));
        assert_eq!(ledger.receipt, "主线簿 卷1-01");
        assert_eq!(ledger.contact, "红衣剑姊");
        assert!(ledger.hud_label().contains("卷1 1/7 接取"));
        let tracker = quest.active_task_tracker();
        assert!(tracker.contains("章程：1/7 接取"));
        assert!(tracker.contains("路书：主线簿 卷1-01"));

        quest.stage = QuestStage::CaveTrial { remaining: 0 };
        let ledger = quest.main_task_ledger();
        assert_eq!(ledger.chapter, Chapter::MoonCave);
        assert_eq!((ledger.step, ledger.total), (2, 4));
        assert_eq!(ledger.place, "水月回廊");
        assert_eq!(ledger.contact, "水月晶阵");

        quest.stage = QuestStage::TuneRiverLanterns;
        let ledger = quest.main_task_ledger();
        assert_eq!(ledger.receipt, "主线簿 卷3-05");
        assert_eq!(ledger.action, "点河灯");
        assert!(quest.main_task_summary().contains("卷3 5/7 点河灯"));

        quest.stage = QuestStage::ReturnToShrineKeeper;
        let ledger = quest.main_task_ledger();
        assert_eq!(ledger.chapter, Chapter::PlagueRain);
        assert_eq!(ledger.action, "交祠灰");
        assert_eq!(ledger.contact, "祠祝");

        quest.stage = QuestStage::AlignMansionMirrors;
        let ledger = quest.main_task_ledger();
        assert_eq!(ledger.receipt, "主线簿 卷5-04");
        assert_eq!(ledger.place, "照影镜廊");

        quest.stage = QuestStage::AlignThunderDrums;
        let ledger = quest.main_task_ledger();
        assert_eq!(ledger.chapter, Chapter::SouthernThunder);
        assert_eq!(ledger.action, "校雷鼓");

        quest.stage = QuestStage::FinaleComplete;
        let ledger = quest.main_task_ledger();
        assert_eq!(ledger.chapter, Chapter::FinalDream);
        assert_eq!((ledger.step, ledger.total), (5, 5));
        assert_eq!(ledger.receipt, "主线簿 终卷-05");
        assert!(quest.active_task_tracker().contains("定尾声"));
    }

    #[test]
    fn chapter_seals_archive_main_chapter_closures() {
        let mut quest = QuestLog::default();
        assert_eq!(quest.chapter_seal_summary(), "章印 未得");

        quest.talk(QuestRole::SwordSister);
        quest.talk(QuestRole::Linger);
        quest.talk(QuestRole::StarMage);
        quest.record_victory();
        quest.record_victory();
        quest.talk(QuestRole::SwordSister);
        quest.talk(QuestRole::Merchant);
        let village_lines = quest.talk(QuestRole::BambooScout);

        assert!(quest.has_chapter_seal(Chapter::VillageOath));
        assert!(village_lines.iter().any(|line| line.contains("余杭赤火印")));
        assert!(quest.chapter_seal_summary().contains("章印 1/7"));
        assert!(quest.active_task_tracker().contains("余杭赤火印"));

        quest.talk(QuestRole::CavePriestess);
        quest.record_victory();
        quest.record_victory();
        quest.record_victory();
        quest.activate_moon_crystal(MoonCrystal::North);
        quest.activate_moon_crystal(MoonCrystal::South);
        let boss_line = quest
            .record_boss_victory(BossKind::MoonWraith)
            .expect("moon boss should close the cave chapter");

        assert!(boss_line.contains("水月灵誓印"));
        assert!(quest.has_chapter_seal(Chapter::MoonCave));
        assert!(quest.chapter_seal_summary().contains("章印 2/7"));

        quest.stage = QuestStage::FinaleComplete;
        quest.chapter_seals = (1 << CHAPTER_COUNT) - 1;
        let epilogue = quest.finale_epilogue_lines();
        assert!(epilogue.iter().any(|line| line.contains("七枚章印")));
        assert!(epilogue.iter().any(|line| line.contains("心渊照影印")));
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
        assert!(first.iter().any(|line| line.contains("【卷章画面】")));
        assert!(first.iter().any(|line| line.contains("【卷章基调】")));
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
        assert!(second.iter().any(|line| line.contains("水月洞门")));
        assert!(second.iter().any(|line| line.contains("水月洞天开门")));
        assert!(quest.take_chapter_card().is_none());
    }

    #[test]
    fn chapter_cards_cover_every_story_chapter_with_visual_direction() {
        let chapters = [
            (QuestStage::NotStarted, "第一卷", "余杭村灯", "初遇与出走"),
            (
                QuestStage::SeekCavePriestess,
                "第二卷",
                "水月洞门",
                "清冷、试炼",
            ),
            (
                QuestStage::OpeningComplete,
                "第三卷",
                "江岸药炉",
                "人间烟火",
            ),
            (
                QuestStage::SeekPlagueElder,
                "第四卷",
                "黑雨落在病屋",
                "苦雨、病声",
            ),
            (
                QuestStage::SeekCapitalEnvoy,
                "第五卷",
                "云都灯火",
                "繁华表面",
            ),
            (
                QuestStage::SeekSpiritGuide,
                "第六卷",
                "南疆雷草坡",
                "誓言、部族",
            ),
            (QuestStage::SeekFinalOracle, "终章", "灵渊终门", "旧梦回潮"),
        ];

        for (stage, title, visual, tone) in chapters {
            let mut quest = QuestLog::default();
            quest.stage = stage;
            let card = quest
                .take_chapter_card()
                .expect("chapter should have a card");
            assert_eq!(card.len(), 6);
            assert!(card[0].contains(title));
            assert!(card[1].contains("【卷章画面】"));
            assert!(card[1].contains(visual));
            assert!(card[2].contains("【卷章基调】"));
            assert!(card[2].contains(tone));
            assert!(card.iter().any(|line| line.contains("【卷章玩法】")));
            assert!(card.iter().any(|line| line.contains("【下一步】")));
            assert!(quest.take_chapter_card().is_none());
        }
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
        quest.add_companion(Companion::Linger);
        quest.add_companion(Companion::SwordSister);
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
        assert!(quest.has_companion(Companion::SpiritWitch));
        assert_eq!(quest.party_summary(), "李逍遥、赵灵儿、林月衡、南瑶");
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
        quest.companion_scenes = (1 << COMPANION_SCENE_COUNT) - 1;

        let lines = quest.finale_epilogue_lines();

        assert!(quest.has_key_item(KeyItem::HomecomingSeal));
        assert!(lines.iter().any(|line| line.contains("并肩回去")));
        assert!(lines.iter().any(|line| line.contains("全部 14 张委托")));
        assert!(lines.iter().any(|line| line.contains("羁绊 7/7，营地 7/7")));
        assert!(lines.iter().any(|line| line.contains("小传 5/5")));
        assert!(
            lines
                .iter()
                .any(|line| line.contains("每一次夜谈、休整和同伴小传"))
        );
        assert!(lines.iter().any(|line| line.contains("营火都没有白点")));
    }

    #[test]
    fn side_quest_step_plans_cover_every_commission_goal() {
        for side in ALL_SIDE_QUESTS {
            assert_eq!(
                side.field_steps().len() as u32,
                side.goal(),
                "{} should have one authored step per progress point",
                side.name()
            );
            let mut quest = QuestLog::default();
            if let Some(required) = side.prerequisite() {
                quest.side_completed |= side_quest_bit(required);
            }
            quest.side_active |= side_quest_bit(side);
            let tracker = quest
                .active_side_task_tracker()
                .expect("active commission should have a tracker");
            assert!(tracker.contains("步骤：第 1/"));
            assert!(quest.side_task_step_plan(side).contains("签收委托"));
            assert!(quest.side_task_step_plan(side).contains(side.field_step(0)));
        }
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
            "【任务板】山路余妖\n状态：可领取\n操作：领取委托\n委托签：余杭-巡山-壹\n委托人：余杭巡山人\n地点：余杭村郊外缘草丛\n路线：从村郊北侧草坡绕到旧竹栅，靠近草丛就会引出余妖。\n目标：击退两只草丛妖兽，再回村郊任务板交付。\n流程：签收委托 -> 1. 踏查旧竹栅草丛 -> 2. 清理退路余妖 -> 交付裁断：回余杭村郊任务板交付。\n下一步：先签收委托，再做第 1/2 步：踏查旧竹栅草丛。\n现场：领取后可在现场选择细查或快断。\n追踪：踏入村郊草丛引出余妖，击退 2 只。\n交付：回余杭村郊任务板交付。\n进度：0/2\n裁断：未裁断\n报酬：经验 +24 / 药水 +1 / 钱 +18文"
        );
        assert_eq!(
            quest.side_task_action_label(SideQuest::VillageTrail),
            "领取"
        );
        assert_eq!(
            quest.side_task_prompt(SideQuest::VillageTrail),
            "委托板 山路余妖 | 可领取 | 空格领取\n委托签：余杭-巡山-壹 · 余杭巡山人\n地点：余杭村郊外缘草丛\n路线：从村郊北侧草坡绕到旧竹栅，靠近草丛就会引出余妖。\n流程：签收委托 -> 1. 踏查旧竹栅草丛 -> 2. 清理退路余妖 -> 交付裁断：回余杭村郊任务板交付。\n下一步：先签收委托，再做第 1/2 步：踏查旧竹栅草丛。\n目标：击退两只草丛妖兽，再回村郊任务板交付。\n报酬：经验 +24 / 药水 +1 / 钱 +18文"
        );
        assert!(!quest.is_side_quest_unlocked(SideQuest::VillageHerbs));
        assert_eq!(
            quest.side_task_summary(SideQuest::VillageHerbs),
            "任务板 后续：药圃护路 - 需先完成《山路余妖》后开放。"
        );
        assert!(
            quest
                .side_task_detail(SideQuest::VillageHerbs)
                .contains("状态：后续委托\n操作：先完成《山路余妖》")
        );
        assert_eq!(
            quest.side_task_prompt(SideQuest::VillageHerbs),
            "委托板 药圃护路 | 后续委托 | 空格查看\n后续：需先完成《山路余妖》。\n预告签：余杭-药圃-贰 · 药婆\n预告：护送药圃小路，击退两只闻香而来的妖兽。"
        );
        let locked = quest.interact_side_quest(SideQuest::VillageHerbs);
        assert!(locked.reward.is_none());
        assert!(locked.lines[1].contains("后续委托"));
        assert!(!quest.is_side_quest_active(SideQuest::VillageHerbs));
        assert_eq!(quest.side_marker_for(SideQuest::VillageHerbs), "?");
        let preview = quest.side_task_accept_preview(SideQuest::VillageTrail);
        assert!(preview[1].contains("【委托契约】山路余妖"));
        assert!(preview[1].contains("领取票据：确认领取后写入任务簿"));
        assert!(preview.iter().any(|line| line.contains("告示写着")));
        assert!(preview.iter().any(|line| line.contains("【委托签】")));
        assert!(preview.iter().any(|line| line.contains("【路线线索】")));
        assert!(preview.iter().any(|line| line.contains("【步骤预览】")));
        assert!(preview.iter().any(|line| line.contains("【追踪预览】")));
        assert!(preview.iter().any(|line| line.contains("【签收回执】")));
        assert!(preview.iter().any(|line| line.contains("【下一步】")));
        assert!(preview.iter().any(|line| line.contains("【报酬】")));
        assert!(
            quest
                .side_task_board_card(SideQuest::VillageTrail)
                .contains("【委托清单】山路余妖 [可领取]")
        );
        assert_eq!(
            quest.side_task_board_option_label(SideQuest::VillageTrail),
            "领取 · 山路余妖 0/2 · 余杭-巡山-壹"
        );
        assert_eq!(quest.side_marker_for(SideQuest::VillageTrail), "!");

        let interaction = quest.interact_side_quest(SideQuest::VillageTrail);
        assert!(interaction.reward.is_none());
        assert!(interaction.lines[0].contains("【任务板】山路余妖"));
        assert!(interaction.lines[0].contains("状态：进行中"));
        assert!(interaction.lines[0].contains("操作：查看进度"));
        assert!(interaction.lines[1].contains("【委托契约】"));
        assert!(interaction.lines[1].contains("HUD 会持续显示路线"));
        assert!(interaction.lines.iter().any(|line| {
            line.contains("【领取委托】")
                && line.contains("写入任务簿")
                && line.contains("余杭-巡山-壹")
        }));
        assert!(interaction.lines.iter().any(|line| {
            line.contains("【签收回执】") && line.contains("现场选择：未处理现场")
        }));
        assert!(
            interaction
                .lines
                .iter()
                .any(|line| line.contains("【委托人】余杭巡山人"))
        );
        assert!(
            interaction
                .lines
                .iter()
                .any(|line| line.contains("【委托地点】余杭村郊外缘草丛"))
        );
        assert!(
            interaction
                .lines
                .iter()
                .any(|line| line.contains("【路线线索】从村郊北侧"))
        );
        assert!(
            interaction
                .lines
                .iter()
                .any(|line| line.contains("【任务追踪】踏入村郊草丛"))
        );
        assert!(
            interaction
                .lines
                .iter()
                .any(|line| line.contains("【交付】回余杭村郊任务板交付"))
        );
        assert!(quest.is_side_quest_active(SideQuest::VillageTrail));
        let tracker = quest
            .active_side_task_tracker()
            .expect("accepted side quest should be tracked");
        assert!(tracker.contains("委托追踪 · 山路余妖 [进行中]"));
        assert!(tracker.contains("委托签：余杭-巡山-壹"));
        assert!(tracker.contains("步骤：第 1/2 步：踏查旧竹栅草丛。"));
        assert!(tracker.contains("现场：未处理现场"));
        assert!(tracker.contains("地点：余杭村郊外缘草丛"));
        assert!(tracker.contains("路线：从村郊北侧草坡"));
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
                .contains("状态：进行中\n操作：查看进度\n委托签")
        );
        assert_eq!(quest.side_marker_for(SideQuest::VillageTrail), "*");

        assert_eq!(
            quest.record_side_victory(),
            Some("【支线】山路余妖 进度 1/2，下一步：第 2/2 步：清理退路余妖。".to_string())
        );
        assert_eq!(
            quest.record_side_victory(),
            Some(
                "【支线】山路余妖 条件达成，下一步：现场已完成，回余杭村郊任务板交付。".to_string()
            )
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
            "委托板 山路余妖 | 可交付 | 空格交付\n委托签：余杭-巡山-壹 | 进度：2/2\n下一步：现场已完成，回余杭村郊任务板交付。\n交付：回余杭村郊任务板交付。\n报酬：经验 +24 / 药水 +1 / 钱 +18文"
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
        assert!(quest.is_side_quest_unlocked(SideQuest::VillageHerbs));
        assert_eq!(
            quest.side_task_summary(SideQuest::VillageHerbs),
            "任务板 可领取：药圃护路 - 护送药圃小路，击退两只闻香而来的妖兽。"
        );
        assert!(
            quest
                .active_task_tracker()
                .contains("委托追踪：暂无已领取委托")
        );
        assert!(interaction.lines[0].contains("状态：可交付"));
        assert!(interaction.lines[0].contains("操作：交付委托"));
        assert!(interaction.lines[1].contains("【委托契约】"));
        assert!(interaction.lines[1].contains("交付时需选择稳妥封存或追查余波"));
        assert_eq!(
            quest.side_quest_resolution(SideQuest::VillageTrail),
            Some(SideQuestResolution::Settle)
        );
        assert!(
            interaction
                .lines
                .iter()
                .any(|line| line.contains("【委托裁断】稳妥封存"))
        );
        assert!(
            interaction
                .lines
                .iter()
                .any(|line| line.contains("【支线完成】"))
        );
        assert_eq!(
            quest.side_task_summary(SideQuest::VillageTrail),
            "任务板 已完成：山路余妖 · 稳妥封存"
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
    fn npc_errands_are_accepted_delivered_and_tracked_once() {
        let mut quest = QuestLog::default();
        let errand = NpcErrand::BambooDewToCave;

        assert_eq!(quest.npc_errand_summary(), "托付 无");
        assert_eq!(quest.npc_errand_marker_for(errand), "!");
        assert_eq!(quest.npc_errand_delivery_marker_for(errand), "");
        assert!(quest.active_npc_errand_tracker().is_none());

        let preview = quest.npc_errand_accept_preview(errand);
        assert!(preview[0].contains("竹林猎户"));
        assert!(preview[2].contains("托付签：托付-余杭-竹露"));
        assert!(preview[3].contains("目标 NPC"));

        let accepted = quest.accept_npc_errand(errand);
        assert!(accepted.reward.is_none());
        assert!(quest.is_npc_errand_active(errand));
        assert_eq!(quest.npc_errand_marker_for(errand), "*");
        assert_eq!(quest.npc_errand_delivery_marker_for(errand), "!");
        assert!(quest.npc_errand_summary().contains("竹露送药"));
        assert!(
            quest
                .active_npc_errand_tracker()
                .expect("active errand should be tracked")
                .contains("NPC托付 · 竹露送药 [进行中]")
        );
        assert!(quest.active_task_tracker().contains("NPC托付 · 竹露送药"));

        let repeated_accept = quest.accept_npc_errand(errand);
        assert!(repeated_accept.reward.is_none());
        assert!(repeated_accept.lines[0].contains("已经在任务簿"));

        let completed = quest.complete_npc_errand(errand);
        assert_eq!(
            completed.reward,
            Some(NpcErrandReward {
                exp: 18,
                potions: 1,
                gold: 8,
                hp: 10,
                mp: 4,
            })
        );
        assert!(quest.is_npc_errand_completed(errand));
        assert!(!quest.is_npc_errand_active(errand));
        assert_eq!(quest.npc_errand_marker_for(errand), "✓");
        assert_eq!(quest.npc_errand_delivery_marker_for(errand), "✓");
        assert_eq!(quest.active_npc_errand_tracker(), None);
        assert_eq!(quest.npc_errand_summary(), "托付 已完成 1/8");

        let repeated_complete = quest.complete_npc_errand(errand);
        assert!(repeated_complete.reward.is_none());
        assert!(repeated_complete.lines[0].contains("已经送达"));
    }

    #[test]
    fn npc_errands_cover_late_chapter_cross_map_deliveries() {
        let cases = [
            (
                QuestStage::CaveTrial { remaining: 3 },
                Chapter::MoonCave,
                NpcErrand::MoonMossToRiver,
                NpcErrandReward {
                    exp: 28,
                    potions: 0,
                    gold: 18,
                    hp: 8,
                    mp: 8,
                },
            ),
            (
                QuestStage::SeekCapitalEnvoy,
                Chapter::CapitalMirror,
                NpcErrand::CapitalStarSlip,
                NpcErrandReward {
                    exp: 62,
                    potions: 0,
                    gold: 42,
                    hp: 10,
                    mp: 10,
                },
            ),
            (
                QuestStage::FindMansionSpy,
                Chapter::CapitalMirror,
                NpcErrand::MirrorMedicineToSouth,
                NpcErrandReward {
                    exp: 70,
                    potions: 1,
                    gold: 48,
                    hp: 14,
                    mp: 8,
                },
            ),
            (
                QuestStage::SeekFinalOracle,
                Chapter::FinalDream,
                NpcErrand::FinalLampWick,
                NpcErrandReward {
                    exp: 96,
                    potions: 1,
                    gold: 72,
                    hp: 20,
                    mp: 12,
                },
            ),
        ];

        for (stage, chapter, errand, reward) in cases {
            let mut quest = QuestLog::default();
            assert!(!quest.is_npc_errand_available(errand));
            quest.stage = stage;
            assert_eq!(quest.current_chapter(), chapter);
            assert!(quest.is_npc_errand_available(errand));

            let preview = quest.npc_errand_accept_preview(errand);
            assert!(preview[0].contains(errand.issuer()));
            assert!(preview[2].contains(errand.receipt_id()));
            assert!(quest.accept_npc_errand(errand).reward.is_none());
            assert!(quest.is_npc_errand_active(errand));
            assert!(
                quest
                    .active_npc_errand_tracker()
                    .is_some_and(|line| line.contains(errand.turn_in_place()))
            );

            let completed = quest.complete_npc_errand(errand);
            assert_eq!(completed.reward, Some(reward));
            assert!(quest.is_npc_errand_completed(errand));
            assert_eq!(quest.npc_errand_delivery_marker_for(errand), "✓");
        }
    }

    #[test]
    fn side_quest_turn_in_records_resolution_branch() {
        let mut quest = QuestLog::default();
        quest.interact_side_quest(SideQuest::RiverLanterns);
        for _ in 0..quest.side_quest_goal(SideQuest::RiverLanterns) {
            quest.record_side_victory();
        }

        let interaction = quest.interact_side_quest_with_resolution(
            SideQuest::RiverLanterns,
            SideQuestResolution::Pursue,
        );

        assert_eq!(
            quest.side_quest_resolution(SideQuest::RiverLanterns),
            Some(SideQuestResolution::Pursue)
        );
        assert!(
            interaction
                .lines
                .iter()
                .any(|line| line.contains("【委托裁断】追查余波"))
        );
        assert!(
            quest
                .side_task_detail(SideQuest::RiverLanterns)
                .contains("裁断：追查余波")
        );
        assert!(
            quest
                .side_task_summary(SideQuest::RiverLanterns)
                .contains("追查余波")
        );

        let repeat = quest.interact_side_quest(SideQuest::RiverLanterns);
        assert!(repeat.reward.is_none());
        assert!(repeat.lines.iter().any(|line| line.contains("追查余波")));
    }

    #[test]
    fn side_quest_objective_progress_targets_one_active_commission() {
        let mut quest = QuestLog::default();
        assert_eq!(
            quest.record_side_objective(SideQuest::MoonCaveCrystals, "晶阵已净"),
            None
        );

        quest.interact_side_quest(SideQuest::MoonCaveCrystals);
        assert_eq!(
            quest.record_side_objective(SideQuest::MoonCaveCrystals, "晶阵已净"),
            Some(
                "【委托推进】水月晶尘：晶阵已净，进度 1/2，下一步：第 2/2 步：封住晶尘石道。"
                    .to_string()
            )
        );
        assert_eq!(quest.side_quest_progress(SideQuest::MoonCaveCrystals), 1);
        assert_eq!(quest.side_quest_progress(SideQuest::MoonCaveEchoes), 0);
        assert_eq!(
            quest.record_side_objective(SideQuest::MoonCaveEchoes, "回声已压"),
            None
        );
        assert_eq!(
            quest.record_side_objective(SideQuest::MoonCaveCrystals, "晶阵已净"),
            Some("【委托推进】水月晶尘：晶阵已净，条件达成，下一步：现场已完成，回水月洞天石牌交付。".to_string())
        );
        assert_eq!(quest.side_marker_for(SideQuest::MoonCaveCrystals), "!");
        assert_eq!(
            quest.record_side_objective(SideQuest::MoonCaveCrystals, "晶阵已净"),
            None
        );
    }

    #[test]
    fn finale_side_quest_chain_has_board_unlock_and_rewards() {
        let mut quest = QuestLog::default();

        assert_eq!(
            quest.side_task_summary(SideQuest::FinalDreamEchoes),
            "任务板 可领取：梦灯余波 - 压住三段旧梦水影，回灵渊终门任务板交付。"
        );
        assert!(!quest.is_side_quest_unlocked(SideQuest::FinalHomewardVows));
        assert_eq!(
            quest.side_task_prompt(SideQuest::FinalHomewardVows),
            "委托板 归潮旧愿 | 后续委托 | 空格查看\n后续：需先完成《梦灯余波》。\n预告签：灵渊-归潮-贰 · 归潮水签\n预告：护住两枚归潮灯签，回灵渊终门任务板交付。"
        );

        let accepted = quest.interact_side_quest(SideQuest::FinalDreamEchoes);
        assert!(accepted.reward.is_none());
        assert!(accepted.lines.iter().any(|line| line.contains("终门灯簿")));
        assert!(
            quest
                .active_side_task_tracker()
                .expect("final side quest should be tracked")
                .contains("委托追踪 · 梦灯余波 [进行中]")
        );

        for _ in 0..quest.side_quest_goal(SideQuest::FinalDreamEchoes) {
            quest.record_side_victory();
        }
        let completed = quest.interact_side_quest(SideQuest::FinalDreamEchoes);
        assert_eq!(
            completed.reward,
            Some(SideQuestReward {
                exp: 92,
                potions: 2,
                gold: 84,
            })
        );
        assert!(quest.is_side_quest_completed(SideQuest::FinalDreamEchoes));
        assert!(quest.is_side_quest_unlocked(SideQuest::FinalHomewardVows));

        quest.interact_side_quest(SideQuest::FinalHomewardVows);
        for _ in 0..quest.side_quest_goal(SideQuest::FinalHomewardVows) {
            quest.record_side_victory();
        }
        let completed = quest.interact_side_quest(SideQuest::FinalHomewardVows);
        assert_eq!(
            completed.reward,
            Some(SideQuestReward {
                exp: 104,
                potions: 2,
                gold: 96,
            })
        );
        assert!(
            quest
                .side_task_summary(SideQuest::FinalHomewardVows)
                .contains("任务板 已完成")
        );
    }

    #[test]
    fn side_quest_interactions_include_party_echoes_by_companion() {
        let quest = QuestLog::default();
        let preview = quest.side_task_accept_preview(SideQuest::VillageTrail);
        assert!(
            preview
                .iter()
                .any(|line| line.contains("【队伍回响】李逍遥") && line.contains("山路余妖"))
        );

        let mut river = QuestLog::default();
        river.add_companion(Companion::Linger);
        let accepted = river.interact_side_quest(SideQuest::RiverLanterns);
        assert!(
            accepted
                .lines
                .iter()
                .any(|line| line.contains("【队伍回响】赵灵儿") && line.contains("逆流河灯"))
        );
        river.record_side_victory();
        let progress = river.interact_side_quest(SideQuest::RiverLanterns);
        assert!(
            progress
                .lines
                .iter()
                .any(|line| line.contains("赵灵儿") && line.contains("还有回声"))
        );
        river.record_side_victory();
        let completed = river.interact_side_quest(SideQuest::RiverLanterns);
        assert!(
            completed
                .lines
                .iter()
                .any(|line| line.contains("赵灵儿") && line.contains("安静下来了"))
        );

        let mut capital = QuestLog::default();
        capital.add_companion(Companion::SwordSister);
        let accepted = capital.interact_side_quest(SideQuest::CapitalPatrol);
        assert!(
            accepted
                .lines
                .iter()
                .any(|line| line.contains("【队伍回响】林月衡") && line.contains("镜阵残影"))
        );

        let mut finale = QuestLog::default();
        finale.add_companion(Companion::SpiritWitch);
        let preview = finale.side_task_accept_preview(SideQuest::FinalDreamEchoes);
        assert!(
            preview
                .iter()
                .any(|line| line.contains("【队伍回响】南瑶") && line.contains("旧梦水影"))
        );
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
    fn field_supplies_are_one_time_route_pickups() {
        let mut quest = QuestLog::default();
        assert_eq!(quest.field_supply_summary(), "采集 0/15");
        assert_eq!(
            quest.field_supply_marker_for(FieldSupply::VillageHerbs),
            "!"
        );

        let interaction = quest.interact_field_supply(FieldSupply::VillageHerbs);
        assert_eq!(
            interaction.reward,
            Some(SupplyReward {
                exp: 4,
                potions: 1,
                gold: 0,
                hp: 14,
                mp: 0,
            })
        );
        assert!(interaction.lines[0].contains("村郊止血草"));
        assert!(quest.has_collected_supply(FieldSupply::VillageHerbs));
        assert_eq!(
            quest.field_supply_marker_for(FieldSupply::VillageHerbs),
            "✓"
        );
        assert_eq!(quest.field_supply_summary(), "采集 1/15");

        let repeat = quest.interact_field_supply(FieldSupply::VillageHerbs);
        assert!(repeat.reward.is_none());
        assert!(repeat.lines[0].contains("已经采过"));
        assert_eq!(quest.field_supply_summary(), "采集 1/15");
    }

    #[test]
    fn commission_traces_advance_active_side_tasks_once() {
        let mut quest = QuestLog::default();
        assert_eq!(quest.commission_trace_summary(), "委托现场 0/14");
        assert_eq!(
            quest.commission_trace_marker_for(SideQuest::VillageTrail),
            "？"
        );

        let inactive = quest.interact_commission_trace(
            SideQuest::VillageTrail,
            "旧竹栅妖痕",
            "竹栅旁还残着妖爪，正好可登记委托。",
            "旧竹栅妖痕已查",
            "竹栅旁有新断的草叶，像是委托板上的余妖线索。",
            "竹栅旁的妖痕已经理清。",
        );
        assert!(inactive.iter().any(|line| line.contains("先到委托板")));
        assert_eq!(quest.side_quest_progress(SideQuest::VillageTrail), 0);
        assert!(!quest.has_commission_trace(SideQuest::VillageTrail));

        quest.interact_side_quest(SideQuest::VillageTrail);
        assert_eq!(
            quest.commission_trace_marker_for(SideQuest::VillageTrail),
            "!"
        );
        let lines = quest.interact_commission_trace(
            SideQuest::VillageTrail,
            "旧竹栅妖痕",
            "竹栅旁还残着妖爪，正好可登记委托。",
            "旧竹栅妖痕已查",
            "竹栅旁有新断的草叶，像是委托板上的余妖线索。",
            "竹栅旁的妖痕已经理清。",
        );
        assert!(lines.iter().any(|line| line.contains("【委托推进】")));
        assert!(
            lines
                .iter()
                .any(|line| line.contains("【现场处理】细查现场"))
        );
        assert_eq!(quest.side_quest_progress(SideQuest::VillageTrail), 1);
        assert!(quest.has_commission_trace(SideQuest::VillageTrail));
        assert_eq!(
            quest.side_quest_field_approach(SideQuest::VillageTrail),
            Some(SideQuestFieldApproach::Investigate)
        );
        assert!(
            quest
                .side_task_contract(SideQuest::VillageTrail)
                .contains("现场：细查现场")
        );
        assert_eq!(
            quest.commission_trace_marker_for(SideQuest::VillageTrail),
            "✓"
        );
        assert_eq!(quest.commission_trace_summary(), "委托现场 1/14");

        let repeat = quest.interact_commission_trace(
            SideQuest::VillageTrail,
            "旧竹栅妖痕",
            "竹栅旁还残着妖爪，正好可登记委托。",
            "旧竹栅妖痕已查",
            "竹栅旁有新断的草叶，像是委托板上的余妖线索。",
            "竹栅旁的妖痕已经理清。",
        );
        assert!(repeat.iter().any(|line| line.contains("已经写进委托签")));
        assert_eq!(quest.side_quest_progress(SideQuest::VillageTrail), 1);

        let mut fast = QuestLog::default();
        fast.interact_side_quest(SideQuest::VillageTrail);
        let lines = fast.interact_commission_trace_with_approach(
            SideQuest::VillageTrail,
            "旧竹栅妖痕",
            "竹栅旁还残着妖爪，正好可登记委托。",
            "旧竹栅妖痕已查",
            "竹栅旁有新断的草叶，像是委托板上的余妖线索。",
            "竹栅旁的妖痕已经理清。",
            SideQuestFieldApproach::Confront,
        );
        assert!(
            lines
                .iter()
                .any(|line| line.contains("【现场处理】快断余妖"))
        );
        assert_eq!(
            fast.side_quest_field_approach(SideQuest::VillageTrail),
            Some(SideQuestFieldApproach::Confront)
        );
    }

    #[test]
    fn shop_gear_records_one_time_growth_purchases() {
        let mut quest = QuestLog::default();
        assert_eq!(quest.shop_gear_summary(), "装备 0/4");
        assert!(!quest.has_shop_gear(ShopGear::VillageSwordTassel));

        assert!(quest.record_shop_gear(ShopGear::VillageSwordTassel));
        assert!(quest.has_shop_gear(ShopGear::VillageSwordTassel));
        assert_eq!(quest.shop_gear_count(), 1);
        assert_eq!(quest.shop_gear_summary(), "装备 1/4 竹剑穗");

        assert!(!quest.record_shop_gear(ShopGear::VillageSwordTassel));
        assert_eq!(quest.shop_gear_count(), 1);

        assert!(quest.record_shop_gear(ShopGear::SouthernThunderCharm));
        assert_eq!(quest.shop_gear_summary(), "装备 2/4 雷纹符");
    }

    #[test]
    fn route_marks_are_one_time_route_memory() {
        let mut quest = QuestLog::default();
        assert_eq!(quest.route_memory_summary(), "路印 0/6");
        assert_eq!(
            quest.route_mark_encounter_multiplier(RouteMark::MoonEcho),
            1.0
        );
        assert_eq!(quest.route_mark_marker_for(RouteMark::MoonEcho), "!");

        let lines = quest.interact_route_mark(RouteMark::MoonEcho);
        assert!(lines[0].contains("水月回廊路印"));
        assert!(lines[2].contains("路印 1/6 水月回廊路印"));
        assert!(quest.has_route_mark(RouteMark::MoonEcho));
        assert_eq!(quest.route_mark_count(), 1);
        assert_eq!(quest.route_mark_marker_for(RouteMark::MoonEcho), "✓");
        assert_eq!(
            quest.route_mark_encounter_multiplier(RouteMark::MoonEcho),
            0.90
        );
        assert_eq!(
            quest.route_mark_encounter_multiplier(RouteMark::ReedFord),
            1.0
        );

        let repeat = quest.interact_route_mark(RouteMark::MoonEcho);
        assert!(repeat[0].contains("已经记入路线"));
        assert_eq!(quest.route_mark_count(), 1);

        quest.interact_route_mark(RouteMark::DreamReturn);
        assert_eq!(quest.route_mark_count(), 2);
        assert_eq!(quest.route_memory_summary(), "路印 2/6 旧梦归水路印");
    }

    #[test]
    fn route_detours_record_branch_choice_once() {
        let mut quest = QuestLog::default();
        assert_eq!(quest.route_branch_summary(), "分支 0/6");
        assert_eq!(quest.route_report_summary(), "路报 0/6");
        assert_eq!(
            quest.route_detour_marker_for(RouteDetour::MoonEchoPool),
            "？"
        );
        assert_eq!(
            quest.route_detour_encounter_multiplier(RouteDetour::MoonEchoPool),
            1.0
        );

        let preview = quest.route_detour_preview(RouteDetour::MoonEchoPool);
        assert!(preview[0].contains("照水暗池"));
        assert!(preview.iter().any(|line| line.contains("队伍商议")));
        let resolution =
            quest.complete_route_detour(RouteDetour::MoonEchoPool, RouteDetourApproach::Scout);
        assert!(resolution.reward.is_some());
        assert_eq!(resolution.tactic, Some(CampBonus::Warmth));
        assert!(resolution.lines[2].contains("分支 1/6 照水暗池·细查"));
        assert!(
            resolution
                .lines
                .iter()
                .any(|line| line.contains("队伍准备"))
        );
        assert_eq!(quest.active_camp_bonus(), Some(CampBonus::Warmth));
        assert!(quest.has_route_detour(RouteDetour::MoonEchoPool));
        assert_eq!(
            quest.route_detour_approach(RouteDetour::MoonEchoPool),
            Some(RouteDetourApproach::Scout)
        );
        assert_eq!(
            quest.route_detour_marker_for(RouteDetour::MoonEchoPool),
            "✓"
        );
        assert_eq!(
            quest.route_detour_encounter_multiplier(RouteDetour::MoonEchoPool),
            0.92
        );

        let repeat =
            quest.complete_route_detour(RouteDetour::MoonEchoPool, RouteDetourApproach::PressOn);
        assert!(repeat.reward.is_none());
        assert_eq!(repeat.tactic, None);
        assert_eq!(
            quest.route_detour_approach(RouteDetour::MoonEchoPool),
            Some(RouteDetourApproach::Scout)
        );
        assert_eq!(quest.route_detour_count(), 1);

        let swift =
            quest.complete_route_detour(RouteDetour::DreamBackwater, RouteDetourApproach::PressOn);
        assert!(swift.lines[2].contains("旧梦回湾·快走"));
        assert_eq!(quest.route_detour_count(), 2);
        assert_eq!(
            quest.route_detour_encounter_multiplier(RouteDetour::DreamBackwater),
            0.97
        );
    }

    #[test]
    fn route_detour_reports_are_claimed_once_after_branch() {
        let mut quest = QuestLog::default();
        let empty = quest.claim_route_detour_report(RouteDetour::ReedHiddenFord);
        assert!(empty.lines.is_empty());
        assert_eq!(empty.reward, None);
        assert!(!quest.route_detour_report_ready(RouteDetour::ReedHiddenFord));

        quest.complete_route_detour(RouteDetour::ReedHiddenFord, RouteDetourApproach::Scout);
        assert!(quest.route_detour_report_ready(RouteDetour::ReedHiddenFord));

        let report = quest.claim_route_detour_report(RouteDetour::ReedHiddenFord);
        assert_eq!(
            report.reward,
            Some(RouteDetourReportReward {
                exp: 12,
                potions: 0,
                gold: 14,
            })
        );
        assert!(report.lines[0].contains("【报路】芦下隐渡"));
        assert!(report.lines[1].contains("江岸巡货人"));
        assert!(report.lines[2].contains("路报 1/6 芦下隐渡"));
        assert_eq!(quest.route_report_summary(), "路报 1/6 芦下隐渡");
        assert!(!quest.route_detour_report_ready(RouteDetour::ReedHiddenFord));

        let repeat = quest.claim_route_detour_report(RouteDetour::ReedHiddenFord);
        assert_eq!(repeat.reward, None);
        assert!(repeat.lines[0].contains("已经交给当地人"));
        assert_eq!(quest.route_detour_report_count(), 1);
    }

    #[test]
    fn route_detours_turn_party_advice_into_next_battle_tactics() {
        let mut careful = QuestLog::default();
        careful.add_companion(Companion::Linger);
        let resolution =
            careful.complete_route_detour(RouteDetour::MoonEchoPool, RouteDetourApproach::Scout);
        assert_eq!(resolution.tactic, Some(CampBonus::Vigil));
        assert_eq!(careful.active_camp_bonus(), Some(CampBonus::Vigil));
        assert!(resolution.lines.iter().any(|line| line.contains("赵灵儿")));
        assert!(
            resolution
                .lines
                .iter()
                .any(|line| line.contains("灵儿守护"))
        );

        let mut swift = QuestLog::default();
        swift.add_companion(Companion::SwordSister);
        let resolution =
            swift.complete_route_detour(RouteDetour::ReedHiddenFord, RouteDetourApproach::PressOn);
        assert_eq!(resolution.tactic, Some(CampBonus::Focus));
        assert_eq!(swift.active_camp_bonus(), Some(CampBonus::Focus));
        assert!(resolution.lines.iter().any(|line| line.contains("林月衡")));
        assert!(
            resolution
                .lines
                .iter()
                .any(|line| line.contains("月衡破势"))
        );

        let mut southern = QuestLog::default();
        southern.add_companion(Companion::SpiritWitch);
        let resolution = southern
            .complete_route_detour(RouteDetour::ThunderRidgeCache, RouteDetourApproach::Scout);
        assert_eq!(resolution.tactic, Some(CampBonus::Vigil));
        assert!(resolution.lines.iter().any(|line| line.contains("南瑶")));

        let mut preserved = QuestLog::default();
        preserved.add_companion(Companion::Linger);
        preserved.record_camp_tactic(CampBonus::Focus);
        let resolution = preserved
            .complete_route_detour(RouteDetour::DreamBackwater, RouteDetourApproach::Scout);
        assert_eq!(resolution.tactic, None);
        assert_eq!(preserved.active_camp_bonus(), Some(CampBonus::Focus));
        assert!(resolution.lines.iter().any(|line| line.contains("仍在")));
    }

    #[test]
    fn care_aftermath_reward_requires_bond_and_camp_once() {
        let mut quest = QuestLog::default();
        assert!(!quest.care_aftermath_ready(Chapter::VillageOath));
        assert_eq!(quest.claim_care_aftermath(Chapter::VillageOath), None);

        quest.talk(QuestRole::SwordSister);
        quest.talk(QuestRole::Linger);
        quest.interact_bond_scene();
        assert!(!quest.care_aftermath_ready(Chapter::VillageOath));

        quest.interact_camp_scene();
        assert!(quest.care_aftermath_ready(Chapter::VillageOath));
        assert_eq!(
            quest.claim_care_aftermath(Chapter::VillageOath),
            Some(CareAftermathReward {
                exp: 16,
                potions: 1,
                gold: 10,
            })
        );
        assert!(quest.has_claimed_care_aftermath(Chapter::VillageOath));
        assert!(!quest.care_aftermath_ready(Chapter::VillageOath));
        assert_eq!(quest.claim_care_aftermath(Chapter::VillageOath), None);
        assert_eq!(quest.claim_care_aftermath(Chapter::MoonCave), None);
    }

    #[test]
    fn commission_aftermath_reward_requires_local_chain_once() {
        let mut quest = QuestLog::default();
        assert!(!quest.commission_aftermath_ready(Chapter::MoonCave));
        assert_eq!(quest.claim_commission_aftermath(Chapter::MoonCave), None);

        quest.interact_side_quest(SideQuest::MoonCaveCrystals);
        for _ in 0..quest.side_quest_goal(SideQuest::MoonCaveCrystals) {
            quest.record_side_victory();
        }
        quest.interact_side_quest(SideQuest::MoonCaveCrystals);
        assert!(!quest.commission_aftermath_ready(Chapter::MoonCave));

        quest.interact_side_quest(SideQuest::MoonCaveEchoes);
        for _ in 0..quest.side_quest_goal(SideQuest::MoonCaveEchoes) {
            quest.record_side_victory();
        }
        quest.interact_side_quest(SideQuest::MoonCaveEchoes);
        assert!(quest.commission_aftermath_ready(Chapter::MoonCave));
        assert_eq!(
            quest.claim_commission_aftermath(Chapter::MoonCave),
            Some(CommissionAftermathReward {
                exp: 28,
                potions: 1,
                gold: 20,
            })
        );
        assert!(quest.has_claimed_commission_aftermath(Chapter::MoonCave));
        assert!(!quest.commission_aftermath_ready(Chapter::MoonCave));
        assert_eq!(quest.claim_commission_aftermath(Chapter::MoonCave), None);
        assert_eq!(quest.claim_commission_aftermath(Chapter::VillageOath), None);
    }

    #[test]
    fn shrine_blessing_can_be_set_and_consumed() {
        let mut quest = QuestLog::default();
        assert_eq!(quest.active_shrine_blessing(), None);
        assert_eq!(quest.shrine_travel_summary(), "香火 无");
        assert_eq!(quest.shrine_encounter_rate_multiplier(), 1.0);

        assert!(quest.set_shrine_blessing(ShrineBlessing::Sword));
        assert_eq!(quest.active_shrine_blessing(), Some(ShrineBlessing::Sword));
        assert_eq!(quest.shrine_travel_summary(), "香火 剑心香火 引妖练剑");
        assert_eq!(quest.shrine_encounter_rate_multiplier(), 1.20);
        assert!(!quest.set_shrine_blessing(ShrineBlessing::Guard));
        assert_eq!(quest.active_shrine_blessing(), Some(ShrineBlessing::Sword));

        assert_eq!(quest.take_shrine_blessing(), Some(ShrineBlessing::Sword));
        assert_eq!(quest.active_shrine_blessing(), None);
        assert_eq!(quest.shrine_travel_summary(), "香火 无");
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

    #[test]
    fn companion_scenes_unlock_after_bond_and_record_personal_routes() {
        let mut quest = QuestLog::default();
        quest.talk(QuestRole::SwordSister);
        quest.talk(QuestRole::Linger);
        quest.add_companion(Companion::SwordSister);

        assert_eq!(
            quest.current_companion_scene(),
            Some(CompanionScene::SwordSisterTrailGuard)
        );
        assert_eq!(quest.companion_scene_available(), None);
        let blocked = quest.interact_companion_scene();
        assert!(blocked.reward.is_none());
        assert!(
            blocked
                .lines
                .iter()
                .any(|line| line.contains("先把本章灵儿羁绊"))
        );

        quest.interact_bond_scene();
        assert_eq!(
            quest.companion_scene_available(),
            Some(CompanionScene::SwordSisterTrailGuard)
        );
        let trail = quest.interact_companion_scene();
        assert_eq!(
            trail.reward,
            Some(CompanionSceneReward {
                exp: 20,
                potions: 0
            })
        );
        assert!(trail.lines.iter().any(|line| line.contains("林月衡")));
        assert!(trail.lines.iter().any(|line| line.contains("同伴战术")));
        assert!(quest.has_seen_companion_scene(CompanionScene::SwordSisterTrailGuard));
        assert_eq!(quest.active_camp_bonus(), Some(CampBonus::Focus));
        assert_eq!(quest.companion_story_summary(), "小传 1/5");

        quest.stage = QuestStage::SeekCapitalEnvoy;
        quest.bond_scenes |= bond_scene_bit(BondScene::CapitalRooftop);
        let capital = quest.interact_companion_scene();
        assert!(capital.lines.iter().any(|line| line.contains("照影旧案")));
        assert_eq!(quest.companion_scene_level(), 2);

        quest.stage = QuestStage::SeekSpiritGuide;
        quest.add_companion(Companion::SpiritWitch);
        quest.bond_scenes |= bond_scene_bit(BondScene::SouthernRoadOath);
        let southern = quest.interact_companion_scene();
        assert!(
            southern
                .lines
                .iter()
                .any(|line| line.contains("南瑶") && line.contains("不覆盖当前准备"))
        );
        assert_eq!(quest.active_camp_bonus(), Some(CampBonus::Focus));
        assert!(quest.has_seen_companion_scene(CompanionScene::SpiritWitchSouthernTotem));

        quest.stage = QuestStage::SeekFinalOracle;
        quest.bond_scenes |= bond_scene_bit(BondScene::FinalGateQuiet);
        let final_sword = quest.interact_companion_scene();
        assert!(final_sword.lines.iter().any(|line| line.contains("归剑")));
        let final_spirit = quest.interact_companion_scene();
        assert!(
            final_spirit
                .lines
                .iter()
                .any(|line| line.contains("归潮灯誓"))
        );
        assert_eq!(quest.companion_story_summary(), "小传 5/5");
        assert_eq!(quest.current_companion_scene(), None);
    }

    #[test]
    fn companion_aftermath_requires_personal_scene_and_claims_once() {
        let mut quest = QuestLog::default();
        let trail = CompanionScene::SwordSisterTrailGuard;

        assert!(!quest.companion_aftermath_ready(trail));
        assert_eq!(quest.claim_companion_aftermath(trail), None);

        quest.talk(QuestRole::SwordSister);
        quest.talk(QuestRole::Linger);
        quest.add_companion(Companion::SwordSister);
        quest.interact_bond_scene();
        quest.interact_companion_scene();

        assert!(quest.companion_aftermath_ready(trail));
        assert_eq!(
            quest.claim_companion_aftermath(trail),
            Some(CompanionAftermathReward {
                exp: 16,
                potions: 0,
                gold: 10,
            })
        );
        assert!(quest.has_claimed_companion_aftermath(trail));
        assert!(!quest.companion_aftermath_ready(trail));
        assert_eq!(quest.claim_companion_aftermath(trail), None);

        let capital = CompanionScene::SwordSisterCapitalMirror;
        quest.stage = QuestStage::SeekCapitalEnvoy;
        quest.bond_scenes |= bond_scene_bit(BondScene::CapitalRooftop);
        quest.interact_companion_scene();
        assert_eq!(
            quest.claim_companion_aftermath(capital),
            Some(CompanionAftermathReward {
                exp: 32,
                potions: 1,
                gold: 22,
            })
        );
        assert!(quest.has_claimed_companion_aftermath(capital));
    }

    #[test]
    fn missed_companion_scene_can_be_recovered_through_revisit_contract() {
        let revisit = CompanionRevisit::TrailEcho;
        let mut quest = QuestLog::default();
        quest.add_companion(Companion::SwordSister);

        assert!(!quest.companion_revisit_available(revisit));
        assert_eq!(quest.companion_revisit_giver_marker_for(revisit), "");

        quest.stage = QuestStage::SeekPlagueElder;
        assert!(quest.companion_revisit_available(revisit));
        assert_eq!(quest.companion_revisit_status(revisit), "可领取");
        assert_eq!(quest.companion_revisit_giver_marker_for(revisit), "!");
        assert!(
            quest
                .companion_revisit_accept_preview(revisit)
                .iter()
                .any(|line| line.contains("小传补访-01") && line.contains("瘴雨祠道"))
        );

        let accepted = quest.accept_companion_revisit(revisit);
        assert!(accepted.iter().any(|line| line.contains("补访签已接")));
        assert!(quest.companion_revisit_active(revisit));
        assert!(quest.companion_revisit_summary().contains("旧栅余声[寻访]"));
        assert!(
            quest
                .active_companion_revisit_tracker()
                .is_some_and(|line| line.contains("祠道旧铃路印"))
        );
        assert_eq!(quest.companion_revisit_field_marker_for(revisit), "");

        quest.interact_route_mark(RouteMark::PlagueBell);
        assert_eq!(quest.companion_revisit_field_marker_for(revisit), "!");
        assert_eq!(
            quest.active_companion_revisit_at(RouteMark::PlagueBell),
            Some(revisit)
        );

        let field = quest.resolve_companion_revisit(revisit);
        assert_eq!(
            field.reward,
            Some(CompanionSceneReward {
                exp: 20,
                potions: 0,
            })
        );
        assert!(field.lines.iter().any(|line| line.contains("迟来小传")));
        assert!(
            field
                .lines
                .iter()
                .any(|line| line.contains("一起把人带出去"))
        );
        assert!(quest.has_seen_companion_scene(CompanionScene::SwordSisterTrailGuard));
        assert!(quest.companion_revisit_ready(revisit));
        assert_eq!(quest.companion_revisit_status(revisit), "待交付");
        assert_eq!(quest.companion_revisit_field_marker_for(revisit), "");

        let turn_in = quest.complete_companion_revisit(revisit);
        assert!(turn_in.lines.iter().any(|line| line.contains("补访归档")));
        assert_eq!(
            turn_in.reward,
            Some(CompanionAftermathReward {
                exp: 16,
                potions: 0,
                gold: 10,
            })
        );
        assert!(quest.companion_revisit_completed(revisit));
        assert_eq!(quest.companion_revisit_summary(), "补访 已结 1/3");
        assert!(quest.complete_companion_revisit(revisit).reward.is_none());

        assert_eq!(
            CompanionRevisit::for_target_mark(RouteMark::ThunderSwitchback),
            Some(CompanionRevisit::MirrorTrace)
        );
        assert_eq!(
            CompanionRevisit::for_target_mark(RouteMark::DreamReturn),
            Some(CompanionRevisit::TotemVow)
        );

        let mut timely = QuestLog::default();
        timely.add_companion(Companion::SwordSister);
        timely.stage = QuestStage::SeekPlagueElder;
        timely.companion_scenes |= companion_scene_bit(CompanionScene::SwordSisterTrailGuard);
        assert!(!timely.companion_revisit_available(CompanionRevisit::TrailEcho));
        assert_eq!(
            timely.companion_revisit_giver_marker_for(CompanionRevisit::TrailEcho),
            ""
        );
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

    pub fn is_npc_errand_active(&self, errand: NpcErrand) -> bool {
        self.npc_errand_active & npc_errand_bit(errand) != 0
    }

    pub fn is_npc_errand_completed(&self, errand: NpcErrand) -> bool {
        self.npc_errand_completed & npc_errand_bit(errand) != 0
    }

    pub fn is_npc_errand_available(&self, errand: NpcErrand) -> bool {
        !self.is_npc_errand_active(errand)
            && !self.is_npc_errand_completed(errand)
            && self.current_chapter().index() >= errand.chapter().index()
    }

    pub fn npc_errand_marker_for(&self, errand: NpcErrand) -> &'static str {
        if self.is_npc_errand_completed(errand) {
            "✓"
        } else if self.is_npc_errand_active(errand) {
            "*"
        } else if self.is_npc_errand_available(errand) {
            "!"
        } else {
            ""
        }
    }

    pub fn npc_errand_delivery_marker_for(&self, errand: NpcErrand) -> &'static str {
        if self.is_npc_errand_completed(errand) {
            "✓"
        } else if self.is_npc_errand_active(errand) {
            "!"
        } else {
            ""
        }
    }

    pub fn npc_errand_progress_label(&self, errand: NpcErrand) -> &'static str {
        if self.is_npc_errand_completed(errand) {
            "已送达"
        } else if self.is_npc_errand_active(errand) {
            "进行中"
        } else if self.is_npc_errand_available(errand) {
            "可托付"
        } else {
            "未开放"
        }
    }

    pub fn npc_errand_summary(&self) -> String {
        let active: Vec<String> = ALL_NPC_ERRANDS
            .iter()
            .copied()
            .filter(|errand| self.is_npc_errand_active(*errand))
            .map(|errand| format!("{} -> {}", errand.name(), errand.turn_in_place()))
            .collect();
        if !active.is_empty() {
            return format!("托付 {}", active.join("、"));
        }

        let completed = ALL_NPC_ERRANDS
            .iter()
            .filter(|errand| self.is_npc_errand_completed(**errand))
            .count();
        if completed > 0 {
            format!("托付 已完成 {completed}/{NPC_ERRAND_COUNT}")
        } else {
            "托付 无".to_string()
        }
    }

    pub fn active_npc_errand_tracker(&self) -> Option<String> {
        let errand = ALL_NPC_ERRANDS
            .iter()
            .copied()
            .find(|errand| self.is_npc_errand_active(*errand))?;
        Some(format!(
            "NPC托付 · {} [进行中]\n托付签：{} · {} -> {}\n目标：{}\n路线：{}\n交付：{}\n报酬：{}",
            errand.name(),
            errand.receipt_id(),
            errand.issuer(),
            errand.receiver(),
            errand.objective(),
            errand.route_hint(),
            errand.turn_in_place(),
            errand.reward_text()
        ))
    }

    pub fn npc_errand_contract(&self, errand: NpcErrand) -> String {
        format!(
            "【NPC托付】{} [{}]\n托付签：{} · 委托人：{}\n交付人：{}\n目标：{}\n路线：{}\n交付：{}\n状态：{}\n报酬：{}",
            errand.name(),
            self.npc_errand_progress_label(errand),
            errand.receipt_id(),
            errand.issuer(),
            errand.receiver(),
            errand.objective(),
            errand.route_hint(),
            errand.turn_in_place(),
            self.npc_errand_progress_label(errand),
            errand.reward_text()
        )
    }

    pub fn npc_errand_accept_preview(&self, errand: NpcErrand) -> Vec<String> {
        vec![
            format!(
                "【路人托付】{}想把《{}》交给你。",
                errand.issuer(),
                errand.name()
            ),
            errand.accept_line().to_string(),
            self.npc_errand_contract(errand),
            "确认后会写入任务簿；到目标 NPC 处交付后结算回礼。".to_string(),
        ]
    }

    pub fn accept_npc_errand(&mut self, errand: NpcErrand) -> NpcErrandInteraction {
        if self.is_npc_errand_completed(errand) {
            return NpcErrandInteraction {
                lines: vec![
                    format!("【路人托付】《{}》已经送达。", errand.name()),
                    errand.completed_line().to_string(),
                    self.npc_errand_summary(),
                ],
                reward: None,
            };
        }

        if self.is_npc_errand_active(errand) {
            return NpcErrandInteraction {
                lines: vec![
                    format!("【路人托付】《{}》已经在任务簿。", errand.name()),
                    errand.progress_line().to_string(),
                    self.npc_errand_contract(errand),
                ],
                reward: None,
            };
        }

        if !self.is_npc_errand_available(errand) {
            return NpcErrandInteraction {
                lines: vec![
                    format!("【路人托付】《{}》还不到交托的时候。", errand.name()),
                    format!("需要先进入{}。", errand.chapter().title()),
                ],
                reward: None,
            };
        }

        self.npc_errand_active |= npc_errand_bit(errand);
        NpcErrandInteraction {
            lines: vec![
                format!(
                    "【路人托付已接】{}把《{}》交给你。",
                    errand.issuer(),
                    errand.name()
                ),
                errand.progress_line().to_string(),
                self.npc_errand_contract(errand),
                self.npc_errand_summary(),
            ],
            reward: None,
        }
    }

    pub fn complete_npc_errand(&mut self, errand: NpcErrand) -> NpcErrandInteraction {
        if self.is_npc_errand_completed(errand) {
            return NpcErrandInteraction {
                lines: vec![
                    format!("【路人托付】《{}》已经送达。", errand.name()),
                    errand.completed_line().to_string(),
                    self.npc_errand_summary(),
                ],
                reward: None,
            };
        }

        if !self.is_npc_errand_active(errand) {
            return NpcErrandInteraction {
                lines: vec![
                    format!("【路人托付】还没有接下《{}》。", errand.name()),
                    errand.accept_line().to_string(),
                ],
                reward: None,
            };
        }

        self.npc_errand_active &= !npc_errand_bit(errand);
        self.npc_errand_completed |= npc_errand_bit(errand);
        NpcErrandInteraction {
            lines: vec![
                format!(
                    "【路人托付送达】{}收下《{}》。",
                    errand.receiver(),
                    errand.name()
                ),
                errand.turn_in_line().to_string(),
                self.npc_errand_summary(),
            ],
            reward: Some(errand.reward()),
        }
    }

    pub fn side_quest_resolution(&self, side: SideQuest) -> Option<SideQuestResolution> {
        if !self.is_side_quest_completed(side) {
            return None;
        }

        if self.side_resolution_pursue & side_quest_bit(side) != 0 {
            Some(SideQuestResolution::Pursue)
        } else {
            Some(SideQuestResolution::Settle)
        }
    }

    fn record_side_quest_resolution(&mut self, side: SideQuest, resolution: SideQuestResolution) {
        match resolution {
            SideQuestResolution::Settle => self.side_resolution_pursue &= !side_quest_bit(side),
            SideQuestResolution::Pursue => self.side_resolution_pursue |= side_quest_bit(side),
        }
    }

    fn side_task_resolution_status(&self, side: SideQuest) -> &'static str {
        match self.side_quest_resolution(side) {
            Some(resolution) => resolution.name(),
            None if self.is_side_quest_active(side)
                && self.side_quest_progress(side) >= self.side_quest_goal(side) =>
            {
                "待交付裁断"
            }
            _ => "未裁断",
        }
    }

    pub fn side_task_resolution_reaction(&self, side: SideQuest) -> Option<String> {
        self.side_quest_resolution(side)
            .map(|resolution| side.resolution_reaction_line(resolution))
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

    pub fn side_quest_receipt_id(&self, side: SideQuest) -> &'static str {
        side.receipt_id()
    }

    pub fn side_quest_issuer(&self, side: SideQuest) -> &'static str {
        side.issuer()
    }

    pub fn side_quest_route_hint(&self, side: SideQuest) -> &'static str {
        side.route_hint()
    }

    pub fn side_quest_turn_in_place(&self, side: SideQuest) -> &'static str {
        side.turn_in_place()
    }

    pub fn npc_errand_receipt_id(&self, errand: NpcErrand) -> &'static str {
        errand.receipt_id()
    }

    pub fn npc_errand_issuer(&self, errand: NpcErrand) -> &'static str {
        errand.issuer()
    }

    pub fn npc_errand_receiver(&self, errand: NpcErrand) -> &'static str {
        errand.receiver()
    }

    pub fn npc_errand_objective(&self, errand: NpcErrand) -> &'static str {
        errand.objective()
    }

    pub fn npc_errand_route_hint(&self, errand: NpcErrand) -> &'static str {
        errand.route_hint()
    }

    pub fn npc_errand_turn_in_place(&self, errand: NpcErrand) -> &'static str {
        errand.turn_in_place()
    }

    pub fn npc_errand_reward_text(&self, errand: NpcErrand) -> String {
        errand.reward_text()
    }

    pub fn is_side_quest_unlocked(&self, side: SideQuest) -> bool {
        side.prerequisite()
            .map(|required| self.is_side_quest_completed(required))
            .unwrap_or(true)
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

    pub fn has_chapter_seal(&self, chapter: Chapter) -> bool {
        self.chapter_seals & chapter_bit(chapter) != 0
    }

    fn record_chapter_seal(&mut self, chapter: Chapter) -> bool {
        if self.has_chapter_seal(chapter) {
            false
        } else {
            self.chapter_seals |= chapter_bit(chapter);
            true
        }
    }

    fn record_chapter_seal_line(&mut self, chapter: Chapter) -> String {
        let fresh = self.record_chapter_seal(chapter);
        let state = if fresh { "入簿" } else { "已归档" };
        format!(
            "【章印·{}】{}：{}",
            state,
            chapter.seal_name(),
            chapter.seal_line()
        )
    }

    pub fn chapter_seal_summary(&self) -> String {
        let seals: Vec<&str> = ALL_CHAPTERS
            .iter()
            .copied()
            .filter(|chapter| self.has_chapter_seal(*chapter))
            .map(Chapter::seal_name)
            .collect();
        if seals.is_empty() {
            "章印 未得".to_string()
        } else {
            format!(
                "章印 {}/{} {}",
                seals.len(),
                CHAPTER_COUNT,
                seals.join("、")
            )
        }
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
            format!("【卷章画面】{}", chapter.visual_motif()),
            format!("【卷章基调】{}", chapter.tone_line()),
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

    pub fn has_seen_companion_scene(&self, scene: CompanionScene) -> bool {
        self.companion_scenes & companion_scene_bit(scene) != 0
    }

    pub fn camp_level(&self) -> u32 {
        (0..CAMP_SCENE_COUNT)
            .filter(|index| self.camp_scenes & (1 << index) != 0)
            .count() as u32
    }

    pub fn companion_scene_level(&self) -> u32 {
        (0..COMPANION_SCENE_COUNT)
            .filter(|index| self.companion_scenes & (1 << index) != 0)
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

    pub fn companion_story_summary(&self) -> String {
        format!(
            "小传 {}/{}",
            self.companion_scene_level(),
            COMPANION_SCENE_COUNT
        )
    }

    pub fn has_claimed_companion_aftermath(&self, scene: CompanionScene) -> bool {
        self.companion_aftermath_claimed & companion_scene_bit(scene) != 0
    }

    pub fn companion_aftermath_ready(&self, scene: CompanionScene) -> bool {
        self.has_seen_companion_scene(scene) && !self.has_claimed_companion_aftermath(scene)
    }

    pub fn claim_companion_aftermath(
        &mut self,
        scene: CompanionScene,
    ) -> Option<CompanionAftermathReward> {
        if !self.companion_aftermath_ready(scene) {
            return None;
        }

        self.companion_aftermath_claimed |= companion_scene_bit(scene);
        Some(scene.aftermath_reward())
    }

    pub fn has_accepted_companion_revisit(&self, revisit: CompanionRevisit) -> bool {
        self.companion_revisits_active & companion_revisit_bit(revisit) != 0
    }

    pub fn companion_revisit_available(&self, revisit: CompanionRevisit) -> bool {
        !self.has_accepted_companion_revisit(revisit)
            && !self.has_seen_companion_scene(revisit.scene())
            && self.has_companion(revisit.required_companion())
            && self.current_chapter().index() >= revisit.unlock_chapter().index()
    }

    pub fn companion_revisit_active(&self, revisit: CompanionRevisit) -> bool {
        self.has_accepted_companion_revisit(revisit)
            && !self.has_seen_companion_scene(revisit.scene())
    }

    pub fn companion_revisit_ready(&self, revisit: CompanionRevisit) -> bool {
        self.has_accepted_companion_revisit(revisit)
            && self.has_seen_companion_scene(revisit.scene())
            && !self.has_claimed_companion_aftermath(revisit.scene())
    }

    pub fn companion_scene_has_pending_revisit_turn_in(&self, scene: CompanionScene) -> bool {
        ALL_COMPANION_REVISITS
            .iter()
            .copied()
            .any(|revisit| revisit.scene() == scene && self.companion_revisit_ready(revisit))
    }

    pub fn companion_revisit_completed(&self, revisit: CompanionRevisit) -> bool {
        self.has_accepted_companion_revisit(revisit)
            && self.has_seen_companion_scene(revisit.scene())
            && self.has_claimed_companion_aftermath(revisit.scene())
    }

    pub fn companion_revisit_resolved(&self, revisit: CompanionRevisit) -> bool {
        self.has_accepted_companion_revisit(revisit)
            && self.has_seen_companion_scene(revisit.scene())
    }

    pub fn companion_revisit_status(&self, revisit: CompanionRevisit) -> &'static str {
        if self.companion_revisit_completed(revisit) {
            "已归档"
        } else if self.companion_revisit_ready(revisit) {
            "待交付"
        } else if self.companion_revisit_active(revisit) {
            "寻访中"
        } else if self.companion_revisit_available(revisit) {
            "可领取"
        } else {
            "未开放"
        }
    }

    pub fn companion_revisit_giver_marker_for(&self, revisit: CompanionRevisit) -> &'static str {
        if self.companion_revisit_ready(revisit) || self.companion_revisit_available(revisit) {
            "!"
        } else if self.companion_revisit_active(revisit) {
            "*"
        } else {
            ""
        }
    }

    pub fn companion_revisit_field_marker_for(&self, revisit: CompanionRevisit) -> &'static str {
        if self.companion_revisit_active(revisit) && self.has_route_mark(revisit.target_mark()) {
            "!"
        } else {
            ""
        }
    }

    pub fn active_companion_revisit_at(&self, mark: RouteMark) -> Option<CompanionRevisit> {
        ALL_COMPANION_REVISITS.iter().copied().find(|revisit| {
            revisit.target_mark() == mark && self.companion_revisit_active(*revisit)
        })
    }

    pub fn companion_revisit_summary(&self) -> String {
        let tracked: Vec<String> = ALL_COMPANION_REVISITS
            .iter()
            .copied()
            .filter_map(|revisit| {
                if self.companion_revisit_ready(revisit) {
                    Some(format!("{}[待交]", revisit.name()))
                } else if self.companion_revisit_active(revisit) {
                    Some(format!("{}[寻访]", revisit.name()))
                } else {
                    None
                }
            })
            .collect();
        if !tracked.is_empty() {
            return format!("补访 {}", tracked.join("、"));
        }

        let completed = ALL_COMPANION_REVISITS
            .iter()
            .filter(|revisit| self.companion_revisit_completed(**revisit))
            .count();
        if completed > 0 {
            format!("补访 已结 {completed}/{COMPANION_REVISIT_COUNT}")
        } else {
            "补访 无".to_string()
        }
    }

    pub fn active_companion_revisit_tracker(&self) -> Option<String> {
        let revisit = ALL_COMPANION_REVISITS
            .iter()
            .copied()
            .find(|revisit| self.companion_revisit_ready(*revisit))
            .or_else(|| {
                ALL_COMPANION_REVISITS
                    .iter()
                    .copied()
                    .find(|revisit| self.companion_revisit_active(*revisit))
            })?;
        let next = if self.companion_revisit_ready(revisit) {
            format!("回{}交付补访签", revisit.turn_in_place())
        } else {
            format!(
                "前往{}触碰{}",
                revisit.target_place(),
                revisit.target_mark().name()
            )
        };
        Some(format!(
            "同伴补访 · {} [{}]\n补访签：{} · 交托：{}\n补访对象：{}\n下一步：{}\n路线：{}\n交付：{}",
            revisit.name(),
            self.companion_revisit_status(revisit),
            revisit.receipt_id(),
            revisit.issuer(),
            revisit.scene().speaker(),
            next,
            revisit.route_hint(),
            revisit.turn_in_place()
        ))
    }

    pub fn companion_revisit_contract(&self, revisit: CompanionRevisit) -> String {
        format!(
            "【同伴补访】{} [{}]\n补访签：{} · 交托：{}\n补访对象：{} · 旧事：{}\n现场：{} · {}\n路线：{}\n交付：{}",
            revisit.name(),
            self.companion_revisit_status(revisit),
            revisit.receipt_id(),
            revisit.issuer(),
            revisit.scene().speaker(),
            revisit.scene().title(),
            revisit.target_place(),
            revisit.target_mark().name(),
            revisit.route_hint(),
            revisit.turn_in_place()
        )
    }

    pub fn companion_revisit_accept_preview(&self, revisit: CompanionRevisit) -> Vec<String> {
        vec![
            format!(
                "【迟来寻访】{}替{}递来《{}》。",
                revisit.issuer(),
                revisit.scene().speaker(),
                revisit.name()
            ),
            revisit.accept_line().to_string(),
            self.companion_revisit_contract(revisit),
            "确认后写入任务簿；到野外路印接回旧话，再回交托人处归档。".to_string(),
        ]
    }

    pub fn accept_companion_revisit(&mut self, revisit: CompanionRevisit) -> Vec<String> {
        if self.companion_revisit_active(revisit) || self.companion_revisit_ready(revisit) {
            return vec![
                format!("【同伴补访】《{}》已经在任务簿。", revisit.name()),
                self.companion_revisit_contract(revisit),
            ];
        }
        if self.companion_revisit_completed(revisit)
            || self.has_seen_companion_scene(revisit.scene())
        {
            return vec![
                format!("【同伴补访】《{}》已经归档。", revisit.name()),
                "这段旧话已经接上，不必再走一遍。".to_string(),
            ];
        }
        if !self.companion_revisit_available(revisit) {
            return vec![
                format!("【同伴补访】《{}》还不能领取。", revisit.name()),
                format!(
                    "需先进入{}，并让{}同行。",
                    revisit.unlock_chapter().title(),
                    revisit.scene().speaker()
                ),
            ];
        }

        self.companion_revisits_active |= companion_revisit_bit(revisit);
        vec![
            format!(
                "【补访签已接】{}把《{}》写入任务簿。",
                revisit.issuer(),
                revisit.name()
            ),
            revisit.progress_line().to_string(),
            self.companion_revisit_contract(revisit),
            self.companion_revisit_summary(),
        ]
    }

    pub fn resolve_companion_revisit(
        &mut self,
        revisit: CompanionRevisit,
    ) -> CompanionSceneInteraction {
        if self.companion_revisit_ready(revisit) || self.companion_revisit_completed(revisit) {
            return CompanionSceneInteraction {
                lines: vec![
                    format!("【迟来小传】《{}》已经接回。", revisit.name()),
                    revisit.repeat_line().to_string(),
                ],
                reward: None,
            };
        }
        if !self.companion_revisit_active(revisit) {
            return CompanionSceneInteraction {
                lines: vec![
                    format!("【迟来小传】{}只传来一阵旧回声。", revisit.target_place()),
                    format!("先向{}领取《{}》。", revisit.issuer(), revisit.name()),
                ],
                reward: None,
            };
        }

        self.record_companion_scene(
            revisit.scene(),
            format!(
                "【迟来小传】{} · {}",
                revisit.scene().title(),
                revisit.name()
            ),
            revisit.field_lines(),
        )
    }

    pub fn complete_companion_revisit(
        &mut self,
        revisit: CompanionRevisit,
    ) -> CompanionRevisitTurnIn {
        if self.companion_revisit_completed(revisit) {
            return CompanionRevisitTurnIn {
                lines: vec![
                    format!("【补访归档】《{}》已经交清。", revisit.name()),
                    revisit.completed_line().to_string(),
                ],
                reward: None,
            };
        }
        if !self.companion_revisit_ready(revisit) {
            return CompanionRevisitTurnIn {
                lines: vec![
                    format!("【补访未结】《{}》还没有接回旧话。", revisit.name()),
                    revisit.progress_line().to_string(),
                    self.companion_revisit_contract(revisit),
                ],
                reward: None,
            };
        }

        let reward = self.claim_companion_aftermath(revisit.scene());
        CompanionRevisitTurnIn {
            lines: vec![
                format!(
                    "【补访归档】{}收回《{}》补访签。",
                    revisit.issuer(),
                    revisit.name()
                ),
                revisit.turn_in_line().to_string(),
                self.companion_revisit_summary(),
            ],
            reward,
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

    pub fn has_claimed_care_aftermath(&self, chapter: Chapter) -> bool {
        self.care_aftermath_claimed & chapter_bit(chapter) != 0
    }

    pub fn care_aftermath_ready(&self, chapter: Chapter) -> bool {
        let (bond_scene, camp_scene) = chapter.care_scenes();
        self.has_seen_bond_scene(bond_scene)
            && self.has_seen_camp_scene(camp_scene)
            && !self.has_claimed_care_aftermath(chapter)
    }

    pub fn claim_care_aftermath(&mut self, chapter: Chapter) -> Option<CareAftermathReward> {
        if !self.care_aftermath_ready(chapter) {
            return None;
        }

        self.care_aftermath_claimed |= chapter_bit(chapter);
        Some(chapter.care_aftermath_reward())
    }

    pub fn has_claimed_commission_aftermath(&self, chapter: Chapter) -> bool {
        self.commission_aftermath_claimed & chapter_bit(chapter) != 0
    }

    pub fn commission_aftermath_ready(&self, chapter: Chapter) -> bool {
        let [first, second] = chapter.commission_side_quests();
        self.is_side_quest_completed(first)
            && self.is_side_quest_completed(second)
            && !self.has_claimed_commission_aftermath(chapter)
    }

    pub fn claim_commission_aftermath(
        &mut self,
        chapter: Chapter,
    ) -> Option<CommissionAftermathReward> {
        if !self.commission_aftermath_ready(chapter) {
            return None;
        }

        self.commission_aftermath_claimed |= chapter_bit(chapter);
        Some(chapter.commission_aftermath_reward())
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
            self.has_companion(Companion::SpiritWitch),
        ) {
            (true, true, true) => "李逍遥、赵灵儿、林月衡、南瑶",
            (true, true, false) => "李逍遥、赵灵儿、林月衡",
            (true, false, true) => "李逍遥、赵灵儿、南瑶",
            (true, false, false) => "李逍遥、赵灵儿",
            (false, true, true) => "李逍遥、林月衡、南瑶",
            (false, true, false) => "李逍遥、林月衡",
            (false, false, true) => "李逍遥、南瑶",
            (false, false, false) => "李逍遥",
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
        let resolution_hint = if progress >= side.goal() {
            "\n裁断：可选稳妥封存 / 追查余波"
        } else {
            ""
        };

        Some(format!(
            "委托追踪 · {} [{}]\n委托签：{} · {}\n目标：{}\n进度：{}/{}\n步骤：{}\n现场：{}\n地点：{}\n路线：{}\n交付：{}{}{}",
            side.name(),
            status,
            side.receipt_id(),
            side.issuer(),
            side.tracking_hint(),
            progress,
            side.goal(),
            self.side_task_next_step(side),
            self.side_task_field_status(side),
            side.area(),
            side.route_hint(),
            side.turn_in_place(),
            resolution_hint,
            extra_line
        ))
    }

    pub fn side_task_step_plan(&self, side: SideQuest) -> String {
        side.step_plan()
    }

    pub fn side_task_next_step(&self, side: SideQuest) -> String {
        if self.is_side_quest_completed(side) {
            return format!(
                "已归档为{}，{}",
                self.side_task_resolution_status(side),
                side.completed_line()
            );
        }

        if !self.is_side_quest_unlocked(side) {
            if let Some(required) = side.prerequisite() {
                return format!("先完成《{}》，再领取这张后续委托。", required.name());
            }
            return "等待任务板开放。".to_string();
        }

        if !self.is_side_quest_active(side) {
            return format!(
                "先签收委托，再做第 1/{} 步：{}。",
                side.goal(),
                side.field_step(0)
            );
        }

        side.next_step_for_progress(self.side_quest_progress(side))
    }

    pub fn side_quest_field_approach(&self, side: SideQuest) -> Option<SideQuestFieldApproach> {
        if !self.has_commission_trace(side) {
            return None;
        }

        if self.side_field_confront & side_quest_bit(side) != 0 {
            Some(SideQuestFieldApproach::Confront)
        } else {
            Some(SideQuestFieldApproach::Investigate)
        }
    }

    pub fn side_task_field_status(&self, side: SideQuest) -> String {
        match self.side_quest_field_approach(side) {
            Some(approach) => approach.contract_line(side),
            None if self.is_side_quest_active(side) => "未处理现场，先找本地委托线索。".to_string(),
            None if self.is_side_quest_completed(side) => "未登记现场分支。".to_string(),
            None => "领取后可在现场选择细查或快断。".to_string(),
        }
    }

    pub fn main_task_ledger(&self) -> MainTaskLedger {
        match self.stage {
            QuestStage::NotStarted => MainTaskLedger {
                chapter: Chapter::VillageOath,
                step: 1,
                total: 7,
                action: "接取",
                place: "余杭村郊东北",
                contact: "红衣剑姊",
                receipt: "主线簿 卷1-01",
            },
            QuestStage::TalkToLinger => MainTaskLedger {
                chapter: Chapter::VillageOath,
                step: 2,
                total: 7,
                action: "问线索",
                place: "余杭灵泉旁",
                contact: "赵灵儿",
                receipt: "主线簿 卷1-02",
            },
            QuestStage::FindStarMage => MainTaskLedger {
                chapter: Chapter::VillageOath,
                step: 3,
                total: 7,
                action: "查灵符",
                place: "青竹山径",
                contact: "星咒童子",
                receipt: "主线簿 卷1-03",
            },
            QuestStage::DefeatMonsters { .. } => MainTaskLedger {
                chapter: Chapter::VillageOath,
                step: 4,
                total: 7,
                action: "除妖",
                place: "青竹草坡",
                contact: "山路余妖",
                receipt: "主线簿 卷1-04",
            },
            QuestStage::ReturnToSister => MainTaskLedger {
                chapter: Chapter::VillageOath,
                step: 5,
                total: 7,
                action: "交付",
                place: "余杭村郊",
                contact: "红衣剑姊",
                receipt: "主线簿 卷1-05",
            },
            QuestStage::EscortMerchant => MainTaskLedger {
                chapter: Chapter::VillageOath,
                step: 6,
                total: 7,
                action: "开商路",
                place: "村郊中段",
                contact: "行脚商",
                receipt: "主线簿 卷1-06",
            },
            QuestStage::FindBambooScout => MainTaskLedger {
                chapter: Chapter::VillageOath,
                step: 7,
                total: 7,
                action: "通山门",
                place: "青竹山径",
                contact: "竹林斥候",
                receipt: "主线簿 卷1-07",
            },
            QuestStage::SeekCavePriestess => MainTaskLedger {
                chapter: Chapter::MoonCave,
                step: 1,
                total: 4,
                action: "入洞天",
                place: "水月洞天",
                contact: "月洞祭司",
                receipt: "主线簿 卷2-01",
            },
            QuestStage::CaveTrial { .. } => MainTaskLedger {
                chapter: Chapter::MoonCave,
                step: 2,
                total: 4,
                action: "净晶阵",
                place: "水月回廊",
                contact: "水月晶阵",
                receipt: "主线簿 卷2-02",
            },
            QuestStage::ConfrontMoonWraith => MainTaskLedger {
                chapter: Chapter::MoonCave,
                step: 3,
                total: 4,
                action: "迎首领",
                place: "水月洞天",
                contact: "月洞祭司",
                receipt: "主线簿 卷2-03",
            },
            QuestStage::ReturnToLinger => MainTaskLedger {
                chapter: Chapter::MoonCave,
                step: 4,
                total: 4,
                action: "回报",
                place: "余杭村郊",
                contact: "赵灵儿",
                receipt: "主线簿 卷2-04",
            },
            QuestStage::OpeningComplete => MainTaskLedger {
                chapter: Chapter::RiverMedicine,
                step: 1,
                total: 7,
                action: "接药事",
                place: "江岸小镇",
                contact: "草药医",
                receipt: "主线簿 卷3-01",
            },
            QuestStage::GatherRiverHerbs { .. } => MainTaskLedger {
                chapter: Chapter::RiverMedicine,
                step: 2,
                total: 7,
                action: "采药引",
                place: "江岸草滩",
                contact: "清心药引",
                receipt: "主线簿 卷3-02",
            },
            QuestStage::ReturnToHerbHealer => MainTaskLedger {
                chapter: Chapter::RiverMedicine,
                step: 3,
                total: 7,
                action: "交药引",
                place: "江岸药庐",
                contact: "草药医",
                receipt: "主线簿 卷3-03",
            },
            QuestStage::FindRiverBoatman => MainTaskLedger {
                chapter: Chapter::RiverMedicine,
                step: 4,
                total: 7,
                action: "问水路",
                place: "江岸码头",
                contact: "摆渡人",
                receipt: "主线簿 卷3-04",
            },
            QuestStage::TuneRiverLanterns => MainTaskLedger {
                chapter: Chapter::RiverMedicine,
                step: 5,
                total: 7,
                action: "点河灯",
                place: "江岸芦滩",
                contact: "倒流河灯",
                receipt: "主线簿 卷3-05",
            },
            QuestStage::ConfrontRiverDemon => MainTaskLedger {
                chapter: Chapter::RiverMedicine,
                step: 6,
                total: 7,
                action: "迎首领",
                place: "江岸码头",
                contact: "摆渡人",
                receipt: "主线簿 卷3-06",
            },
            QuestStage::RiverTownComplete => MainTaskLedger {
                chapter: Chapter::RiverMedicine,
                step: 7,
                total: 7,
                action: "出江岸",
                place: "江岸东门",
                contact: "瘴雨村路",
                receipt: "主线簿 卷3-07",
            },
            QuestStage::SeekPlagueElder => MainTaskLedger {
                chapter: Chapter::PlagueRain,
                step: 1,
                total: 7,
                action: "查病簿",
                place: "瘴雨村",
                contact: "村长",
                receipt: "主线簿 卷4-01",
            },
            QuestStage::SeekShrineKeeper => MainTaskLedger {
                chapter: Chapter::PlagueRain,
                step: 2,
                total: 7,
                action: "问祠铃",
                place: "村北祠堂",
                contact: "祠祝",
                receipt: "主线簿 卷4-02",
            },
            QuestStage::CleansePlagueShrines { .. } => MainTaskLedger {
                chapter: Chapter::PlagueRain,
                step: 3,
                total: 7,
                action: "清瘴源",
                place: "瘴雨祠道",
                contact: "黑草瘴源",
                receipt: "主线簿 卷4-03",
            },
            QuestStage::SealPlagueWards => MainTaskLedger {
                chapter: Chapter::PlagueRain,
                step: 4,
                total: 7,
                action: "封铃位",
                place: "瘴雨祠道",
                contact: "净瘴铃位",
                receipt: "主线簿 卷4-04",
            },
            QuestStage::ReturnToShrineKeeper => MainTaskLedger {
                chapter: Chapter::PlagueRain,
                step: 5,
                total: 7,
                action: "交祠灰",
                place: "村北祠堂",
                contact: "祠祝",
                receipt: "主线簿 卷4-05",
            },
            QuestStage::ConfrontMiasmaRoot => MainTaskLedger {
                chapter: Chapter::PlagueRain,
                step: 6,
                total: 7,
                action: "斩瘴母",
                place: "瘴雨村",
                contact: "祠祝",
                receipt: "主线簿 卷4-06",
            },
            QuestStage::PlagueVillageComplete => MainTaskLedger {
                chapter: Chapter::PlagueRain,
                step: 7,
                total: 7,
                action: "赴云都",
                place: "瘴雨东门",
                contact: "云都府城",
                receipt: "主线簿 卷4-07",
            },
            QuestStage::SeekCapitalEnvoy => MainTaskLedger {
                chapter: Chapter::CapitalMirror,
                step: 1,
                total: 7,
                action: "取名帖",
                place: "云都府城",
                contact: "宣令使",
                receipt: "主线簿 卷5-01",
            },
            QuestStage::FindMansionSpy => MainTaskLedger {
                chapter: Chapter::CapitalMirror,
                step: 2,
                total: 7,
                action: "找内线",
                place: "照影府邸",
                contact: "偏院内线",
                receipt: "主线簿 卷5-02",
            },
            QuestStage::GatherSecretLetters { .. } => MainTaskLedger {
                chapter: Chapter::CapitalMirror,
                step: 3,
                total: 7,
                action: "搜密札",
                place: "照影镜廊",
                contact: "府邸密札",
                receipt: "主线簿 卷5-03",
            },
            QuestStage::AlignMansionMirrors => MainTaskLedger {
                chapter: Chapter::CapitalMirror,
                step: 4,
                total: 7,
                action: "对镜阵",
                place: "照影镜廊",
                contact: "账镜证镜",
                receipt: "主线簿 卷5-04",
            },
            QuestStage::ReturnToMansionSpy => MainTaskLedger {
                chapter: Chapter::CapitalMirror,
                step: 5,
                total: 7,
                action: "交密札",
                place: "照影偏院",
                contact: "偏院内线",
                receipt: "主线簿 卷5-05",
            },
            QuestStage::ConfrontMirrorMinister => MainTaskLedger {
                chapter: Chapter::CapitalMirror,
                step: 6,
                total: 7,
                action: "破幻术",
                place: "照影府邸",
                contact: "偏院内线",
                receipt: "主线簿 卷5-06",
            },
            QuestStage::CapitalIntrigueComplete => MainTaskLedger {
                chapter: Chapter::CapitalMirror,
                step: 7,
                total: 7,
                action: "赴南疆",
                place: "云都东门",
                contact: "南疆灵道",
                receipt: "主线簿 卷5-07",
            },
            QuestStage::SeekSpiritGuide => MainTaskLedger {
                chapter: Chapter::SouthernThunder,
                step: 1,
                total: 7,
                action: "找引路",
                place: "南疆灵道",
                contact: "引路人",
                receipt: "主线簿 卷6-01",
            },
            QuestStage::SeekTribalChief => MainTaskLedger {
                chapter: Chapter::SouthernThunder,
                step: 2,
                total: 7,
                action: "问族长",
                place: "南疆灵道",
                contact: "百越族长",
                receipt: "主线簿 卷6-02",
            },
            QuestStage::CleanseSpiritTotems { .. } => MainTaskLedger {
                chapter: Chapter::SouthernThunder,
                step: 3,
                total: 7,
                action: "安图腾",
                place: "雷鼓祭道",
                contact: "雷图腾",
                receipt: "主线簿 卷6-03",
            },
            QuestStage::AlignThunderDrums => MainTaskLedger {
                chapter: Chapter::SouthernThunder,
                step: 4,
                total: 7,
                action: "校雷鼓",
                place: "雷鼓祭道",
                contact: "风云誓鼓",
                receipt: "主线簿 卷6-04",
            },
            QuestStage::ReturnToTribalChief => MainTaskLedger {
                chapter: Chapter::SouthernThunder,
                step: 5,
                total: 7,
                action: "交玉牒",
                place: "南疆灵道",
                contact: "百越族长",
                receipt: "主线簿 卷6-05",
            },
            QuestStage::ConfrontThunderQilin => MainTaskLedger {
                chapter: Chapter::SouthernThunder,
                step: 6,
                total: 7,
                action: "迎雷麟",
                place: "南疆灵道",
                contact: "百越族长",
                receipt: "主线簿 卷6-06",
            },
            QuestStage::SouthernRoadComplete => MainTaskLedger {
                chapter: Chapter::SouthernThunder,
                step: 7,
                total: 7,
                action: "入终门",
                place: "南疆东门",
                contact: "灵渊终门",
                receipt: "主线簿 卷6-07",
            },
            QuestStage::SeekFinalOracle => MainTaskLedger {
                chapter: Chapter::FinalDream,
                step: 1,
                total: 5,
                action: "问宿命",
                place: "灵渊终门",
                contact: "守灯人",
                receipt: "主线簿 终卷-01",
            },
            QuestStage::LightFinalSoulLamps { .. } => MainTaskLedger {
                chapter: Chapter::FinalDream,
                step: 2,
                total: 5,
                action: "点忆灯",
                place: "旧梦水廊",
                contact: "三盏忆梦灯",
                receipt: "主线簿 终卷-02",
            },
            QuestStage::ReturnToFinalOracle => MainTaskLedger {
                chapter: Chapter::FinalDream,
                step: 3,
                total: 5,
                action: "交梦灯",
                place: "灵渊终门",
                contact: "守灯人",
                receipt: "主线簿 终卷-03",
            },
            QuestStage::ConfrontDreamEclipse => MainTaskLedger {
                chapter: Chapter::FinalDream,
                step: 4,
                total: 5,
                action: "决宿命",
                place: "灵渊终门",
                contact: "宿命水影",
                receipt: "主线簿 终卷-04",
            },
            QuestStage::FinaleComplete => MainTaskLedger {
                chapter: Chapter::FinalDream,
                step: 5,
                total: 5,
                action: "定尾声",
                place: "灵渊终门",
                contact: "守灯人",
                receipt: "主线簿 终卷-05",
            },
        }
    }

    pub fn active_task_tracker(&self) -> String {
        let ledger = self.main_task_ledger();
        let mut tracker = format!(
            "主线追踪 · {} [{}]\n{}\n目标：{}\n{}",
            self.chapter_title(),
            self.quest_status(),
            ledger.tracker_line(),
            self.objective(),
            self.chapter_seal_summary()
        );

        if let Some(side_tracker) = self.active_side_task_tracker() {
            tracker.push_str("\n\n");
            tracker.push_str(&side_tracker);
        } else {
            tracker.push_str("\n委托追踪：暂无已领取委托");
        }

        if let Some(errand_tracker) = self.active_npc_errand_tracker() {
            tracker.push_str("\n\n");
            tracker.push_str(&errand_tracker);
        }

        if let Some(revisit_tracker) = self.active_companion_revisit_tracker() {
            tracker.push_str("\n\n");
            tracker.push_str(&revisit_tracker);
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
        format!(
            "主线 {} · {}：{}",
            self.quest_status(),
            self.main_task_ledger().hud_label(),
            self.objective()
        )
    }

    pub fn side_task_summary(&self, side: SideQuest) -> String {
        if self.is_side_quest_completed(side) {
            return format!(
                "任务板 已完成：{} · {}",
                side.name(),
                self.side_task_resolution_status(side)
            );
        }

        if !self.is_side_quest_unlocked(side) {
            if let Some(required) = side.prerequisite() {
                return format!(
                    "任务板 后续：{} - 需先完成《{}》后开放。",
                    side.name(),
                    required.name()
                );
            }
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
            "【任务板】{}\n状态：{}\n操作：{}\n委托签：{}\n委托人：{}\n地点：{}\n路线：{}\n目标：{}\n流程：{}\n下一步：{}\n现场：{}\n追踪：{}\n交付：{}\n进度：{}/{}\n裁断：{}\n报酬：{}",
            side.name(),
            self.side_task_status(side),
            self.side_task_operation(side),
            side.receipt_id(),
            side.issuer(),
            side.area(),
            side.route_hint(),
            side.objective(),
            self.side_task_step_plan(side),
            self.side_task_next_step(side),
            self.side_task_field_status(side),
            side.tracking_hint(),
            side.turn_in_place(),
            shown_progress.min(side.goal()),
            side.goal(),
            self.side_task_resolution_status(side),
            side.reward_text()
        )
    }

    pub fn side_task_contract(&self, side: SideQuest) -> String {
        let progress = self.side_quest_progress(side).min(side.goal());
        format!(
            "【委托契约】{} · {}\n阶段：{} · 推荐：{}\n签号：{} · 委托人：{}\n流程：{}\n下一步：{}\n现场：{}\n进度：{}/{}\n目标：{}\n路线：{}\n交付：{}\n裁断：稳妥封存 / 追查余波（当前：{}）\n报酬：{}\n领取票据：{}",
            side.name(),
            side.area(),
            self.side_task_status(side),
            self.side_task_recommended_action(side),
            side.receipt_id(),
            side.issuer(),
            self.side_task_step_plan(side),
            self.side_task_next_step(side),
            self.side_task_field_status(side),
            progress,
            side.goal(),
            side.objective(),
            side.route_hint(),
            side.turn_in_place(),
            self.side_task_resolution_status(side),
            side.reward_text(),
            self.side_task_receipt_state(side)
        )
    }

    pub fn side_task_accept_receipt(&self, side: SideQuest) -> String {
        format!(
            "【签收回执】{} · {}\n第一步：{}\n现场选择：{}\nHUD追踪：{}\n交付：{}",
            side.receipt_id(),
            side.issuer(),
            self.side_task_next_step(side),
            self.side_task_field_status(side),
            side.tracking_hint(),
            side.turn_in_place()
        )
    }

    pub fn side_task_board_card(&self, side: SideQuest) -> String {
        let progress = if self.is_side_quest_completed(side) {
            side.goal()
        } else {
            self.side_quest_progress(side).min(side.goal())
        };
        format!(
            "【委托清单】{} [{}] · {}\n签号：{} · 进度：{}/{} · 报酬：{}\n地点：{} · 交付：{}\n下一步：{}\n现场：{}\n领取票据：{}",
            side.name(),
            self.side_task_status(side),
            self.side_task_recommended_action(side),
            side.receipt_id(),
            progress,
            side.goal(),
            side.reward_text(),
            side.area(),
            side.turn_in_place(),
            self.side_task_next_step(side),
            self.side_task_field_status(side),
            self.side_task_receipt_state(side)
        )
    }

    pub fn side_task_board_option_label(&self, side: SideQuest) -> String {
        let progress = if self.is_side_quest_completed(side) {
            side.goal()
        } else {
            self.side_quest_progress(side).min(side.goal())
        };
        format!(
            "{} · {} {}/{} · {}",
            self.side_task_action_label(side),
            side.name(),
            progress,
            side.goal(),
            side.receipt_id()
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
        } else if !self.is_side_quest_unlocked(side) {
            "后续委托"
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
        } else if !self.is_side_quest_unlocked(side) {
            if let Some(required) = side.prerequisite() {
                match required {
                    SideQuest::VillageTrail => "先完成《山路余妖》",
                    SideQuest::MoonCaveCrystals => "先完成《水月晶尘》",
                    SideQuest::RiverLanterns => "先完成《河灯巡夜》",
                    SideQuest::PlagueRelief => "先完成《瘴雨救急》",
                    SideQuest::CapitalPatrol => "先完成《府邸巡查》",
                    SideQuest::SouthernThunder => "先完成《灵道巡雷》",
                    SideQuest::FinalDreamEchoes => "先完成《梦灯余波》",
                    _ => "先完成前置委托",
                }
            } else {
                "查看预告"
            }
        } else {
            "领取委托"
        }
    }

    fn side_task_recommended_action(&self, side: SideQuest) -> &'static str {
        if self.is_side_quest_completed(side) {
            "查看归档"
        } else if self.is_side_quest_active(side) {
            if self.side_quest_progress(side) >= side.goal() {
                "回委托点交付领奖"
            } else {
                "按路线推进并保持追踪"
            }
        } else if !self.is_side_quest_unlocked(side) {
            "先清前置委托"
        } else {
            "领取并写入任务簿"
        }
    }

    fn side_task_receipt_state(&self, side: SideQuest) -> &'static str {
        if self.is_side_quest_completed(side) {
            match self.side_quest_resolution(side) {
                Some(SideQuestResolution::Pursue) => "委托签已归档为追查余波，奖励已结清。",
                _ => "委托签已归档为稳妥封存，奖励已结清。",
            }
        } else if self.is_side_quest_active(side) {
            if self.side_quest_progress(side) >= side.goal() {
                "条件已满足，交付时需选择稳妥封存或追查余波。"
            } else {
                "已写入任务簿，HUD 会持续显示路线、进度和交付点。"
            }
        } else if !self.is_side_quest_unlocked(side) {
            "暂不可领取，只能查看预告和前置条件。"
        } else {
            "确认领取后写入任务簿，战斗进度会自动登记。"
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
        } else if !self.is_side_quest_unlocked(side) {
            "查看"
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
            return format!(
                "{}\n归档：{}\n{}",
                header,
                side.receipt_id(),
                side.completed_line()
            );
        }

        if !self.is_side_quest_unlocked(side) {
            if let Some(required) = side.prerequisite() {
                return format!(
                    "{header}\n后续：需先完成《{}》。\n预告签：{} · {}\n预告：{}",
                    required.name(),
                    side.receipt_id(),
                    side.issuer(),
                    side.objective()
                );
            }
        }

        if self.is_side_quest_active(side) {
            let progress = self.side_quest_progress(side).min(side.goal());
            if progress >= side.goal() {
                return format!(
                    "{header}\n委托签：{} | 进度：{progress}/{}\n下一步：{}\n交付：{}\n报酬：{}",
                    side.receipt_id(),
                    side.goal(),
                    self.side_task_next_step(side),
                    side.turn_in_place(),
                    side.reward_text()
                );
            }

            return format!(
                "{header}\n委托签：{} | 进度：{progress}/{}\n下一步：{}\n路线：{}\n交付：{}",
                side.receipt_id(),
                side.goal(),
                self.side_task_next_step(side),
                side.route_hint(),
                side.turn_in_place()
            );
        }

        format!(
            "{header}\n委托签：{} · {}\n地点：{}\n路线：{}\n流程：{}\n下一步：{}\n目标：{}\n报酬：{}",
            side.receipt_id(),
            side.issuer(),
            side.area(),
            side.route_hint(),
            self.side_task_step_plan(side),
            self.side_task_next_step(side),
            side.objective(),
            side.reward_text()
        )
    }

    pub fn side_task_accept_preview(&self, side: SideQuest) -> Vec<String> {
        vec![
            self.side_task_detail(side),
            self.side_task_contract(side),
            "【领取流程】签下后先给预支，任务簿记录签号，HUD 持续追踪目标、路线和交付点。"
                .to_string(),
            self.side_task_accept_receipt(side),
            side.accept_line().to_string(),
            self.side_task_party_line(side, SideQuestPartyMoment::Accept),
            format!("【委托签】{} · {}", side.receipt_id(), side.issuer()),
            format!("【委托地点】{}", side.area()),
            format!("【路线线索】{}", side.route_hint()),
            format!("【步骤预览】{}", self.side_task_step_plan(side)),
            format!("【追踪预览】{}", side.tracking_hint()),
            format!("【下一步】{}", self.side_task_next_step(side)),
            format!("【交付】{}", side.turn_in_place()),
            format!("【报酬】{}", side.reward_text()),
        ]
    }

    pub fn side_task_completed_line(&self, side: SideQuest) -> &'static str {
        side.completed_line()
    }

    fn side_task_party_line(&self, side: SideQuest, moment: SideQuestPartyMoment) -> String {
        let voice = self.side_task_party_voice(side);
        format!(
            "【队伍回响】{}：{}",
            voice.name(),
            voice.line(moment, side.party_focus())
        )
    }

    fn side_task_party_voice(&self, side: SideQuest) -> SideQuestPartyVoice {
        if self.has_companion(Companion::SpiritWitch)
            && matches!(
                side,
                SideQuest::SouthernThunder
                    | SideQuest::SouthernDrums
                    | SideQuest::FinalDreamEchoes
                    | SideQuest::FinalHomewardVows
            )
        {
            SideQuestPartyVoice::SpiritWitch
        } else if self.has_companion(Companion::SwordSister)
            && matches!(
                side,
                SideQuest::CapitalPatrol
                    | SideQuest::CapitalRumors
                    | SideQuest::SouthernThunder
                    | SideQuest::SouthernDrums
                    | SideQuest::FinalDreamEchoes
                    | SideQuest::FinalHomewardVows
            )
        {
            SideQuestPartyVoice::SwordSister
        } else if self.has_companion(Companion::Linger) {
            SideQuestPartyVoice::Linger
        } else {
            SideQuestPartyVoice::Hero
        }
    }

    pub fn interact_side_quest(&mut self, side: SideQuest) -> SideQuestInteraction {
        self.interact_side_quest_with_resolution(side, SideQuestResolution::Settle)
    }

    pub fn interact_side_quest_with_resolution(
        &mut self,
        side: SideQuest,
        resolution: SideQuestResolution,
    ) -> SideQuestInteraction {
        if self.is_side_quest_completed(side) {
            let mut lines = vec![
                self.side_task_detail(side),
                format!("【支线已完成】{}。", side.name()),
                side.completed_line().to_string(),
            ];
            if let Some(reaction) = self.side_task_resolution_reaction(side) {
                lines.push(reaction);
            }
            return SideQuestInteraction {
                lines,
                reward: None,
            };
        }

        if !self.is_side_quest_active(side) && !self.is_side_quest_unlocked(side) {
            let required = side
                .prerequisite()
                .expect("locked side quest should have prerequisite");
            return SideQuestInteraction {
                lines: vec![
                    self.side_task_detail(side),
                    format!("【后续委托】《{}》暂未开放。", side.name()),
                    format!("先完成《{}》，任务板才会添上这张新签。", required.name()),
                ],
                reward: None,
            };
        }

        if self.is_side_quest_active(side) {
            let progress = self.side_quest_progress(side);
            if progress >= side.goal() {
                let detail = self.side_task_detail(side);
                let contract = self.side_task_contract(side);
                self.side_active &= !side_quest_bit(side);
                self.side_completed |= side_quest_bit(side);
                self.record_side_quest_resolution(side, resolution);
                return SideQuestInteraction {
                    lines: vec![
                        detail,
                        contract,
                        format!("【支线完成】{}。", side.name()),
                        format!("【归档】委托签 {} 已注销。", side.receipt_id()),
                        format!(
                            "【委托裁断】{}：{} {}",
                            resolution.name(),
                            resolution.result_line(),
                            side.resolution_turn_in_line(resolution)
                        ),
                        format!("【现场回执】{}", self.side_task_field_status(side)),
                        side.turn_in_line().to_string(),
                        self.side_task_party_line(side, SideQuestPartyMoment::TurnIn),
                    ],
                    reward: Some(side.reward()),
                };
            }

            return SideQuestInteraction {
                lines: vec![
                    self.side_task_detail(side),
                    self.side_task_contract(side),
                    format!(
                        "【支线进行中】{} {}/{}。",
                        side.name(),
                        progress,
                        side.goal()
                    ),
                    format!("【追踪】{}", side.tracking_hint()),
                    format!("【下一步】{}", self.side_task_next_step(side)),
                    side.progress_line().to_string(),
                    self.side_task_party_line(side, SideQuestPartyMoment::Progress),
                ],
                reward: None,
            };
        }

        self.side_active |= side_quest_bit(side);
        self.side_progress[side.index()] = 0;
        let detail = self.side_task_detail(side);
        let contract = self.side_task_contract(side);
        SideQuestInteraction {
            lines: vec![
                detail,
                contract,
                "【领取流程】签收后按 HUD 追踪处理现场，条件满后回委托点交付。".to_string(),
                format!(
                    "【领取委托】《{}》已揭榜，委托签 {} 写入任务簿。",
                    side.name(),
                    side.receipt_id()
                ),
                self.side_task_accept_receipt(side),
                side.accept_line().to_string(),
                self.side_task_party_line(side, SideQuestPartyMoment::Accept),
                format!("【委托人】{}", side.issuer()),
                format!("【委托地点】{}", side.area()),
                format!("【路线线索】{}", side.route_hint()),
                format!("【任务追踪】{}", side.tracking_hint()),
                format!("【下一步】{}", self.side_task_next_step(side)),
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
            let next_step = side.next_step_for_progress(*progress);
            if *progress >= side.goal() {
                messages.push(format!(
                    "【支线】{} 条件达成，下一步：{}",
                    side.name(),
                    next_step
                ));
            } else {
                messages.push(format!(
                    "【支线】{} 进度 {}/{}，下一步：{}",
                    side.name(),
                    *progress,
                    side.goal(),
                    next_step
                ));
            }
        }

        if messages.is_empty() {
            None
        } else {
            Some(messages.join("\n"))
        }
    }

    pub fn record_side_objective(&mut self, side: SideQuest, source: &str) -> Option<String> {
        if !self.is_side_quest_active(side) || self.is_side_quest_completed(side) {
            return None;
        }

        let goal = side.goal();
        let progress = &mut self.side_progress[side.index()];
        if *progress >= goal {
            return None;
        }

        *progress += 1;
        let next_step = side.next_step_for_progress(*progress);
        if *progress >= goal {
            Some(format!(
                "【委托推进】{}：{}，条件达成，下一步：{}",
                side.name(),
                source,
                next_step
            ))
        } else {
            Some(format!(
                "【委托推进】{}：{}，进度 {}/{}，下一步：{}",
                side.name(),
                source,
                *progress,
                goal,
                next_step
            ))
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
        } else if !self.is_side_quest_unlocked(side) {
            "?"
        } else {
            "!"
        }
    }

    pub fn has_commission_trace(&self, side: SideQuest) -> bool {
        self.commission_traces & side_quest_bit(side) != 0
    }

    pub fn commission_trace_count(&self) -> u32 {
        self.commission_traces.count_ones()
    }

    pub fn commission_trace_summary(&self) -> String {
        format!(
            "委托现场 {}/{}",
            self.commission_trace_count(),
            COMMISSION_TRACE_COUNT
        )
    }

    pub fn commission_trace_marker_for(&self, side: SideQuest) -> &'static str {
        if self.is_side_quest_completed(side) || self.has_commission_trace(side) {
            "✓"
        } else if self.is_side_quest_active(side) {
            "!"
        } else {
            "？"
        }
    }

    pub fn interact_commission_trace(
        &mut self,
        side: SideQuest,
        trace_name: &str,
        active_line: &str,
        source: &str,
        inactive_line: &str,
        repeat_line: &str,
    ) -> Vec<String> {
        self.interact_commission_trace_with_approach(
            side,
            trace_name,
            active_line,
            source,
            inactive_line,
            repeat_line,
            SideQuestFieldApproach::Investigate,
        )
    }

    pub fn interact_commission_trace_with_approach(
        &mut self,
        side: SideQuest,
        trace_name: &str,
        active_line: &str,
        source: &str,
        inactive_line: &str,
        repeat_line: &str,
        approach: SideQuestFieldApproach,
    ) -> Vec<String> {
        let mut lines = vec![format!("【委托现场】{} · {}", side.name(), trace_name)];

        if self.is_side_quest_completed(side) {
            lines.push(format!(
                "【归档】《{}》已经交付，现场只剩处理过的痕迹。",
                side.name()
            ));
            lines.push(repeat_line.to_string());
            lines.push(self.commission_trace_summary());
            return lines;
        }

        if !self.is_side_quest_unlocked(side) {
            lines.push(format!(
                "【后续线索】这里像是《{}》的现场，但委托还没开放。",
                side.name()
            ));
            if let Some(required) = side.prerequisite() {
                lines.push(format!("先完成《{}》，这条线索才有用。", required.name()));
            }
            return lines;
        }

        if !self.is_side_quest_active(side) {
            lines.push(inactive_line.to_string());
            lines.push(format!(
                "【提示】先到委托板或联系人处领取《{}》，再回来处理这处现场。",
                side.name()
            ));
            return lines;
        }

        if self.has_commission_trace(side) {
            lines.push(repeat_line.to_string());
            lines.push(format!(
                "【委托登记】{}已经写进委托签，现场方式：{}。",
                source,
                self.side_quest_field_approach(side)
                    .map(SideQuestFieldApproach::name)
                    .unwrap_or("未登记")
            ));
            lines.push(self.side_task_contract(side));
            return lines;
        }

        self.commission_traces |= side_quest_bit(side);
        self.record_commission_trace_approach(side, approach);
        lines.push(active_line.to_string());
        lines.push(format!(
            "【现场处理】{}：{}",
            approach.name(),
            approach.result_line(side)
        ));
        if let Some(progress) = self.record_side_objective(side, source) {
            lines.push(progress);
        } else {
            lines.push(format!(
                "【委托推进】{}：条件已足，回委托点交付。",
                side.name()
            ));
        }
        lines.push(self.side_task_contract(side));
        lines.push(self.commission_trace_summary());
        lines
    }

    fn record_commission_trace_approach(
        &mut self,
        side: SideQuest,
        approach: SideQuestFieldApproach,
    ) {
        match approach {
            SideQuestFieldApproach::Investigate => {
                self.side_field_confront &= !side_quest_bit(side)
            }
            SideQuestFieldApproach::Confront => self.side_field_confront |= side_quest_bit(side),
        }
    }

    pub fn has_shop_gear(&self, gear: ShopGear) -> bool {
        self.shop_gear & shop_gear_bit(gear) != 0
    }

    pub fn record_shop_gear(&mut self, gear: ShopGear) -> bool {
        if self.has_shop_gear(gear) {
            return false;
        }

        self.shop_gear |= shop_gear_bit(gear);
        true
    }

    pub fn shop_gear_count(&self) -> u32 {
        self.shop_gear.count_ones()
    }

    pub fn shop_gear_summary(&self) -> String {
        let count = self.shop_gear_count();
        if count == 0 {
            return format!("装备 0/{SHOP_GEAR_COUNT}");
        }

        format!(
            "装备 {count}/{SHOP_GEAR_COUNT} {}",
            self.last_shop_gear_name()
        )
    }

    fn last_shop_gear_name(&self) -> &'static str {
        [
            ShopGear::SouthernThunderCharm,
            ShopGear::CapitalMirrorGuard,
            ShopGear::RiverSilkVest,
            ShopGear::VillageSwordTassel,
        ]
        .into_iter()
        .find(|gear| self.has_shop_gear(*gear))
        .map(ShopGear::short_name)
        .unwrap_or("无")
    }

    pub fn has_opened_treasure(&self, cache: TreasureCache) -> bool {
        self.treasure_opened & treasure_cache_bit(cache) != 0
    }

    pub fn has_collected_supply(&self, supply: FieldSupply) -> bool {
        self.field_supplies & field_supply_bit(supply) != 0
    }

    pub fn field_supply_count(&self) -> u32 {
        self.field_supplies.count_ones()
    }

    pub fn field_supply_summary(&self) -> String {
        format!("采集 {}/{}", self.field_supply_count(), FIELD_SUPPLY_COUNT)
    }

    pub fn field_supply_marker_for(&self, supply: FieldSupply) -> &'static str {
        if self.has_collected_supply(supply) {
            "✓"
        } else {
            "!"
        }
    }

    pub fn active_shrine_blessing(&self) -> Option<ShrineBlessing> {
        shrine_blessing_from_bits(self.shrine_blessing)
    }

    pub fn shrine_travel_summary(&self) -> String {
        match self.active_shrine_blessing() {
            Some(blessing) => format!("香火 {} {}", blessing.name(), blessing.route_label()),
            None => "香火 无".to_string(),
        }
    }

    pub fn shrine_encounter_rate_multiplier(&self) -> f32 {
        self.active_shrine_blessing()
            .map(ShrineBlessing::encounter_rate_multiplier)
            .unwrap_or(1.0)
    }

    pub fn has_route_mark(&self, mark: RouteMark) -> bool {
        self.route_marks & route_mark_bit(mark) != 0
    }

    pub fn route_mark_count(&self) -> u32 {
        self.route_marks.count_ones()
    }

    pub fn route_memory_summary(&self) -> String {
        let count = self.route_mark_count();
        if count == 0 {
            return format!("路印 0/{ROUTE_MARK_COUNT}");
        }

        format!(
            "路印 {count}/{ROUTE_MARK_COUNT} {}",
            self.route_memory_last_name()
        )
    }

    fn route_memory_last_name(&self) -> &'static str {
        [
            RouteMark::DreamReturn,
            RouteMark::ThunderSwitchback,
            RouteMark::MirrorSideDoor,
            RouteMark::PlagueBell,
            RouteMark::ReedFord,
            RouteMark::MoonEcho,
        ]
        .into_iter()
        .find(|mark| self.has_route_mark(*mark))
        .map(RouteMark::name)
        .unwrap_or("未踏勘")
    }

    pub fn route_mark_encounter_multiplier(&self, mark: RouteMark) -> f32 {
        if self.has_route_mark(mark) { 0.90 } else { 1.0 }
    }

    pub fn route_mark_marker_for(&self, mark: RouteMark) -> &'static str {
        if self.has_route_mark(mark) {
            "✓"
        } else {
            "!"
        }
    }

    pub fn interact_route_mark(&mut self, mark: RouteMark) -> Vec<String> {
        if self.has_route_mark(mark) {
            return vec![
                format!("【路印】{}已经记入路线。", mark.name()),
                mark.repeat_line().to_string(),
                self.route_memory_summary(),
            ];
        }

        self.route_marks |= route_mark_bit(mark);
        vec![
            format!("【路印】记下{}。", mark.name()),
            mark.open_line().to_string(),
            format!(
                "【踏勘】这条路线已经摸清一段，之后在本路行走时更不容易被草中妖影截住。{}",
                self.route_memory_summary()
            ),
        ]
    }

    pub fn has_route_detour(&self, detour: RouteDetour) -> bool {
        self.route_detours & route_detour_bit(detour) != 0
    }

    pub fn route_detour_count(&self) -> u32 {
        self.route_detours.count_ones()
    }

    pub fn route_detour_report_count(&self) -> u32 {
        self.route_detour_reports_claimed.count_ones()
    }

    pub fn route_detour_approach(&self, detour: RouteDetour) -> Option<RouteDetourApproach> {
        if !self.has_route_detour(detour) {
            return None;
        }

        if self.route_detour_scouted & route_detour_bit(detour) != 0 {
            Some(RouteDetourApproach::Scout)
        } else {
            Some(RouteDetourApproach::PressOn)
        }
    }

    pub fn route_branch_summary(&self) -> String {
        let count = self.route_detour_count();
        if count == 0 {
            return format!("分支 0/{ROUTE_DETOUR_COUNT}");
        }

        let detour = self.route_branch_last_detour();
        let approach = self
            .route_detour_approach(detour)
            .unwrap_or(RouteDetourApproach::PressOn);
        format!(
            "分支 {count}/{ROUTE_DETOUR_COUNT} {}·{}",
            detour.name(),
            approach.short_label()
        )
    }

    pub fn route_report_summary(&self) -> String {
        let count = self.route_detour_report_count();
        if count == 0 {
            return format!("路报 0/{ROUTE_DETOUR_COUNT}");
        }

        let detour = self.route_report_last_detour();
        format!("路报 {count}/{ROUTE_DETOUR_COUNT} {}", detour.name())
    }

    fn route_branch_last_detour(&self) -> RouteDetour {
        [
            RouteDetour::DreamBackwater,
            RouteDetour::ThunderRidgeCache,
            RouteDetour::MirrorServantDoor,
            RouteDetour::PlagueHerbTrail,
            RouteDetour::ReedHiddenFord,
            RouteDetour::MoonEchoPool,
        ]
        .into_iter()
        .find(|detour| self.has_route_detour(*detour))
        .unwrap_or(RouteDetour::MoonEchoPool)
    }

    fn route_report_last_detour(&self) -> RouteDetour {
        [
            RouteDetour::DreamBackwater,
            RouteDetour::ThunderRidgeCache,
            RouteDetour::MirrorServantDoor,
            RouteDetour::PlagueHerbTrail,
            RouteDetour::ReedHiddenFord,
            RouteDetour::MoonEchoPool,
        ]
        .into_iter()
        .find(|detour| self.has_claimed_route_detour_report(*detour))
        .unwrap_or(RouteDetour::MoonEchoPool)
    }

    pub fn route_detour_marker_for(&self, detour: RouteDetour) -> &'static str {
        if self.has_route_detour(detour) {
            "✓"
        } else {
            "？"
        }
    }

    pub fn route_detour_encounter_multiplier(&self, detour: RouteDetour) -> f32 {
        match self.route_detour_approach(detour) {
            Some(RouteDetourApproach::Scout) => 0.92,
            Some(RouteDetourApproach::PressOn) => 0.97,
            None => 1.0,
        }
    }

    pub fn route_detour_preview(&self, detour: RouteDetour) -> Vec<String> {
        if let Some(approach) = self.route_detour_approach(detour) {
            return vec![
                format!("【路线分支】{}已经处理过。", detour.name()),
                detour.repeat_line(approach).to_string(),
                self.route_branch_summary(),
            ];
        }

        vec![
            format!("【路线分支】发现{}。", detour.name()),
            detour.preview_line().to_string(),
            self.route_detour_party_preview(detour),
            "选择后会写入路线簿；细查更稳，快走给更多历练但路线收益较小。".to_string(),
        ]
    }

    pub fn complete_route_detour(
        &mut self,
        detour: RouteDetour,
        approach: RouteDetourApproach,
    ) -> RouteDetourResolution {
        if let Some(recorded) = self.route_detour_approach(detour) {
            return RouteDetourResolution {
                lines: vec![
                    format!("【路线分支】{}已经处理过。", detour.name()),
                    detour.repeat_line(recorded).to_string(),
                    self.route_branch_summary(),
                ],
                reward: None,
                tactic: None,
            };
        }

        self.route_detours |= route_detour_bit(detour);
        if approach == RouteDetourApproach::Scout {
            self.route_detour_scouted |= route_detour_bit(detour);
        }
        let tactic = self.route_detour_tactic(detour, approach);
        let tactic = if self.active_camp_bonus().is_none() {
            self.camp_bonus = camp_bonus_bit(tactic);
            Some(tactic)
        } else {
            None
        };

        let mut lines = vec![
            format!(
                "【路线分支】{} · {}。",
                detour.name(),
                approach.action_label()
            ),
            detour.result_line(approach).to_string(),
            format!(
                "【路线后果】{} {}",
                approach.effect_line(),
                self.route_branch_summary()
            ),
            self.route_detour_party_line(detour, approach),
        ];
        if let Some(tactic) = tactic {
            lines.push(format!(
                "【队伍准备】{}：{}",
                tactic.tactic_label(),
                tactic.battle_line()
            ));
        } else if let Some(active) = self.active_camp_bonus() {
            lines.push(format!(
                "【队伍准备】{}仍在，岔路收获只写入路线簿。",
                active.name()
            ));
        }

        RouteDetourResolution {
            lines,
            reward: Some(detour.reward(approach)),
            tactic,
        }
    }

    pub fn has_claimed_route_detour_report(&self, detour: RouteDetour) -> bool {
        self.route_detour_reports_claimed & route_detour_bit(detour) != 0
    }

    pub fn route_detour_report_ready(&self, detour: RouteDetour) -> bool {
        self.has_route_detour(detour) && !self.has_claimed_route_detour_report(detour)
    }

    pub fn claim_route_detour_report(&mut self, detour: RouteDetour) -> RouteDetourReport {
        let Some(approach) = self.route_detour_approach(detour) else {
            return RouteDetourReport {
                lines: Vec::new(),
                reward: None,
            };
        };

        if self.has_claimed_route_detour_report(detour) {
            return RouteDetourReport {
                lines: vec![
                    format!("【报路】{}已经交给当地人。", detour.name()),
                    detour.report_repeat_line(approach).to_string(),
                    self.route_report_summary(),
                ],
                reward: None,
            };
        }

        self.route_detour_reports_claimed |= route_detour_bit(detour);
        RouteDetourReport {
            lines: vec![
                format!(
                    "【报路】{} · {}已交回当地路簿。",
                    detour.name(),
                    approach.short_label()
                ),
                detour.report_line(approach).to_string(),
                format!(
                    "【路线簿】{}；{}。本地人会按这份路报改走巡夜路线。",
                    self.route_branch_summary(),
                    self.route_report_summary()
                ),
            ],
            reward: Some(detour.report_reward(approach)),
        }
    }

    fn route_detour_party_preview(&self, detour: RouteDetour) -> String {
        let lead = if self.has_companion(Companion::SpiritWitch)
            && matches!(
                detour,
                RouteDetour::ThunderRidgeCache | RouteDetour::DreamBackwater
            ) {
            "南瑶会看雷纹与梦水"
        } else if self.has_companion(Companion::Linger) {
            "赵灵儿会先听灵息"
        } else if self.has_companion(Companion::SwordSister) {
            "林月衡会替你看退路"
        } else {
            "李逍遥只能临场判断"
        };
        format!("【队伍商议】{lead}；细查偏守，快走偏攻，都会影响下一战准备。")
    }

    fn route_detour_tactic(&self, detour: RouteDetour, approach: RouteDetourApproach) -> CampBonus {
        match approach {
            RouteDetourApproach::Scout => {
                if self.has_companion(Companion::SpiritWitch)
                    && matches!(
                        detour,
                        RouteDetour::ThunderRidgeCache | RouteDetour::DreamBackwater
                    )
                {
                    CampBonus::Vigil
                } else if self.has_companion(Companion::Linger) {
                    CampBonus::Vigil
                } else {
                    CampBonus::Warmth
                }
            }
            RouteDetourApproach::PressOn => {
                if self.has_companion(Companion::SwordSister) {
                    CampBonus::Focus
                } else {
                    CampBonus::Warmth
                }
            }
        }
    }

    fn route_detour_party_line(
        &self,
        detour: RouteDetour,
        approach: RouteDetourApproach,
    ) -> String {
        if self.has_companion(Companion::SpiritWitch)
            && matches!(
                detour,
                RouteDetour::ThunderRidgeCache | RouteDetour::DreamBackwater
            )
        {
            return match approach {
                RouteDetourApproach::Scout => {
                    "南瑶：这条纹路有旧祭声，慢一点，我替你们稳住回响。".to_string()
                }
                RouteDetourApproach::PressOn => {
                    "南瑶：雷息一断就走，我会把落脚声压在鼓点里。".to_string()
                }
            };
        }

        match approach {
            RouteDetourApproach::Scout if self.has_companion(Companion::Linger) => {
                "赵灵儿：先听一听，草木和水声会告诉我们哪里有伏妖。".to_string()
            }
            RouteDetourApproach::PressOn if self.has_companion(Companion::SwordSister) => {
                "林月衡：我开路，你们跟紧，破口只亮这一瞬。".to_string()
            }
            RouteDetourApproach::Scout => {
                "李逍遥：那就先看清楚，别让岔路反过来绕住人。".to_string()
            }
            RouteDetourApproach::PressOn => "李逍遥：趁妖气还没合拢，先冲过这一段。".to_string(),
        }
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

    pub fn interact_field_supply(&mut self, supply: FieldSupply) -> SupplyInteraction {
        if self.has_collected_supply(supply) {
            return SupplyInteraction {
                lines: vec![
                    format!("【采集】{}已经采过。", supply.name()),
                    supply.empty_line().to_string(),
                    self.field_supply_summary(),
                ],
                reward: None,
            };
        }

        self.field_supplies |= field_supply_bit(supply);
        SupplyInteraction {
            lines: vec![
                format!("【采集】{}", supply.name()),
                supply.gather_line().to_string(),
                format!("【收获】{}", supply.find_line()),
                self.field_supply_summary(),
            ],
            reward: Some(supply.reward()),
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

    pub fn current_companion_scene(&self) -> Option<CompanionScene> {
        match self.current_chapter() {
            Chapter::VillageOath | Chapter::MoonCave | Chapter::RiverMedicine => {
                if self.has_companion(Companion::SwordSister)
                    && !self.has_seen_companion_scene(CompanionScene::SwordSisterTrailGuard)
                {
                    Some(CompanionScene::SwordSisterTrailGuard)
                } else {
                    None
                }
            }
            Chapter::PlagueRain => None,
            Chapter::CapitalMirror => {
                if self.has_companion(Companion::SwordSister)
                    && !self.has_seen_companion_scene(CompanionScene::SwordSisterCapitalMirror)
                {
                    Some(CompanionScene::SwordSisterCapitalMirror)
                } else {
                    None
                }
            }
            Chapter::SouthernThunder => {
                if self.has_companion(Companion::SpiritWitch)
                    && !self.has_seen_companion_scene(CompanionScene::SpiritWitchSouthernTotem)
                {
                    Some(CompanionScene::SpiritWitchSouthernTotem)
                } else {
                    None
                }
            }
            Chapter::FinalDream => {
                if self.has_companion(Companion::SwordSister)
                    && !self.has_seen_companion_scene(CompanionScene::SwordSisterFinalReturn)
                {
                    Some(CompanionScene::SwordSisterFinalReturn)
                } else if self.has_companion(Companion::SpiritWitch)
                    && !self.has_seen_companion_scene(CompanionScene::SpiritWitchFinalVow)
                {
                    Some(CompanionScene::SpiritWitchFinalVow)
                } else {
                    None
                }
            }
        }
    }

    pub fn companion_scene_available(&self) -> Option<CompanionScene> {
        let scene = self.current_companion_scene()?;
        if let Some(bond_scene) = self.current_bond_scene() {
            if !self.has_seen_bond_scene(bond_scene) {
                return None;
            }
        }
        Some(scene)
    }

    pub fn interact_companion_scene(&mut self) -> CompanionSceneInteraction {
        let Some(scene) = self.current_companion_scene() else {
            return CompanionSceneInteraction {
                lines: vec![
                    "【同伴小传】灵灯只映出同行人的影子。".to_string(),
                    "眼下没有新的同伴个人剧情。".to_string(),
                ],
                reward: None,
            };
        };

        if self.companion_scene_available() != Some(scene) {
            return CompanionSceneInteraction {
                lines: vec![
                    format!("【同伴小传】{}还在灯影后。", scene.title()),
                    "先把本章灵儿羁绊夜谈说完，其他同行人的心事才会浮上来。".to_string(),
                ],
                reward: None,
            };
        }

        self.record_companion_scene(
            scene,
            format!("【同伴小传】{}", scene.title()),
            scene.lines(),
        )
    }

    fn record_companion_scene(
        &mut self,
        scene: CompanionScene,
        heading: String,
        narrative: &[&str],
    ) -> CompanionSceneInteraction {
        self.companion_scenes |= companion_scene_bit(scene);
        let mut lines = vec![heading];
        lines.extend(narrative.iter().map(|line| (*line).to_string()));

        let bonus = scene.tactic_bonus();
        if self.active_camp_bonus().is_none() {
            self.camp_bonus = camp_bonus_bit(bonus);
            lines.push(format!(
                "【同伴战术】{}：{}",
                bonus.tactic_label(),
                bonus.battle_line()
            ));
        } else if let Some(active) = self.active_camp_bonus() {
            lines.push(format!(
                "【同伴战术】{}仍在；{}的小传先记入旅途，不覆盖当前准备。",
                active.name(),
                scene.speaker()
            ));
        }
        lines.push(self.companion_story_summary());

        CompanionSceneInteraction {
            lines,
            reward: Some(scene.reward()),
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
            self.finale_chapter_seal_line(),
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
            format!(
                "守灯人：全部 {SIDE_QUEST_COUNT} 张委托都化作人间灯火，灵渊外有人正在等你们的消息。"
            )
        } else if completed_sides >= 6 {
            format!("守灯人：你们沿途点亮了 {completed_sides} 处人间小愿，它们替归路留了光。")
        } else {
            "守灯人：还有许多小愿留在人间，归去之后仍有路要走。".to_string()
        }
    }

    fn finale_chapter_seal_line(&self) -> String {
        let seals = self.chapter_seals.count_ones();
        if seals >= CHAPTER_COUNT as u32 {
            format!(
                "【章印回响】七枚章印都在水里亮起：{}。每一卷路都接住了终门的光。",
                self.chapter_seal_summary()
            )
        } else if seals > 0 {
            format!(
                "【章印回响】{}；仍有 {} 卷路印未归档，旧梦里还留着空位。",
                self.chapter_seal_summary(),
                CHAPTER_COUNT as u32 - seals
            )
        } else {
            "【章印回响】章印未得；终门只听见零散脚步，还没有完整旅卷。".to_string()
        }
    }

    fn finale_memory_line(&self) -> String {
        let bond = self.bond_level();
        let camps = self.camp_level();
        let companion = self.companion_scene_level();
        let missed_bond = (BOND_SCENE_COUNT as u32).saturating_sub(bond);
        let missed_camp = (CAMP_SCENE_COUNT as u32).saturating_sub(camps);
        let missed_companion = (COMPANION_SCENE_COUNT as u32).saturating_sub(companion);
        let missed = missed_bond + missed_camp + missed_companion;

        if missed == 0 {
            format!(
                "【旅途回响】羁绊 {bond}/{BOND_SCENE_COUNT}，营地 {camps}/{CAMP_SCENE_COUNT}，小传 {companion}/{COMPANION_SCENE_COUNT}；每一次夜谈、休整和同伴小传都留在回程里。"
            )
        } else {
            format!(
                "【旅途回响】羁绊 {bond}/{BOND_SCENE_COUNT}，营地 {camps}/{CAMP_SCENE_COUNT}，小传 {companion}/{COMPANION_SCENE_COUNT}；错过 {missed} 次照应，归路仍有未说完的话。"
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
                let seal_line = self.record_chapter_seal_line(Chapter::VillageOath);
                self.stage = QuestStage::SeekCavePriestess;
                vec![
                    "【任务推进】竹林斥候交出月洞令。".to_string(),
                    "【获得道具】月洞令。".to_string(),
                    seal_line,
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
                self.add_companion(Companion::SpiritWitch);
                self.stage = QuestStage::CleanseSpiritTotems { remaining: 3 };
                vec![
                    "【任务推进】百越族长交出图腾符。".to_string(),
                    "【获得道具】百越图腾符。".to_string(),
                    "【队友加入】南瑶以灵巫身份加入队伍。".to_string(),
                    "百越族长：雷麟守着古路，本不伤人，如今却被镜阵余毒激怒。".to_string(),
                    "南瑶：我识得雷鼓旧律。你们挥剑，我来稳住灵路。".to_string(),
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
                let seal_line = self.record_chapter_seal_line(Chapter::MoonCave);
                self.stage = QuestStage::ReturnToLinger;
                Some(format!(
                    "【Boss】月魄妖退散，获得月魄印。回村找赵灵儿。\n{seal_line}"
                ))
            }
            (BossKind::RiverDemon, QuestStage::ConfrontRiverDemon) => {
                self.completed += 1;
                self.add_key_item(KeyItem::RiverPearl);
                let seal_line = self.record_chapter_seal_line(Chapter::RiverMedicine);
                self.stage = QuestStage::RiverTownComplete;
                Some(format!(
                    "【Boss】河魇蛟沉入江心，获得河心珠。江岸小镇暂得安宁。\n{seal_line}"
                ))
            }
            (BossKind::MiasmaRoot, QuestStage::ConfrontMiasmaRoot) => {
                self.completed += 1;
                self.add_key_item(KeyItem::CureCharm);
                let seal_line = self.record_chapter_seal_line(Chapter::PlagueRain);
                self.stage = QuestStage::PlagueVillageComplete;
                Some(format!(
                    "【Boss】瘴母根断裂，获得解瘴符。瘴雨村的雨终于清了。\n{seal_line}"
                ))
            }
            (BossKind::MirrorMinister, QuestStage::ConfrontMirrorMinister) => {
                self.completed += 1;
                self.add_key_item(KeyItem::MirrorSeal);
                let seal_line = self.record_chapter_seal_line(Chapter::CapitalMirror);
                self.stage = QuestStage::CapitalIntrigueComplete;
                Some(format!(
                    "【Boss】照影国师镜阵碎裂，获得照影印。云都府邸暗案暂告一段落。\n{seal_line}"
                ))
            }
            (BossKind::ThunderQilin, QuestStage::ConfrontThunderQilin) => {
                self.completed += 1;
                self.add_key_item(KeyItem::QilinHorn);
                let seal_line = self.record_chapter_seal_line(Chapter::SouthernThunder);
                self.stage = QuestStage::SouthernRoadComplete;
                Some(format!(
                    "【Boss】雷麟收起天雷，获得雷麟角。南疆灵道重新亮起。\n{seal_line}"
                ))
            }
            (BossKind::DreamEclipse, QuestStage::ConfrontDreamEclipse) => {
                self.completed += 1;
                self.add_key_item(KeyItem::FateSeal);
                let seal_line = self.record_chapter_seal_line(Chapter::FinalDream);
                self.stage = QuestStage::FinaleComplete;
                Some(format!(
                    "【Boss】宿命水影散入灵渊，获得宿命印。终门后的水声终于平息。\n{seal_line}"
                ))
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

    pub fn title(self) -> &'static str {
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

    fn short_title(self) -> &'static str {
        match self {
            Chapter::VillageOath => "卷1",
            Chapter::MoonCave => "卷2",
            Chapter::RiverMedicine => "卷3",
            Chapter::PlagueRain => "卷4",
            Chapter::CapitalMirror => "卷5",
            Chapter::SouthernThunder => "卷6",
            Chapter::FinalDream => "终卷",
        }
    }

    fn seal_name(self) -> &'static str {
        match self {
            Chapter::VillageOath => "余杭赤火印",
            Chapter::MoonCave => "水月灵誓印",
            Chapter::RiverMedicine => "苏州河灯印",
            Chapter::PlagueRain => "白河清瘴印",
            Chapter::CapitalMirror => "京华破镜印",
            Chapter::SouthernThunder => "南疆雷誓印",
            Chapter::FinalDream => "心渊照影印",
        }
    }

    fn seal_line(self) -> &'static str {
        match self {
            Chapter::VillageOath => "山路妖雾退散，余杭夜路终于能听见人声。",
            Chapter::MoonCave => "月魄妖退散，水月洞天重新映出同伴身影。",
            Chapter::RiverMedicine => "河魇蛟沉入江心，倒流河灯顺水归城。",
            Chapter::PlagueRain => "瘴母根断裂，白河雨声第一次像雨。",
            Chapter::CapitalMirror => "照影国师镜阵碎裂，暗帖有了能见天日的路。",
            Chapter::SouthernThunder => "雷麟收起天雷，南疆灵道承认队伍的誓印。",
            Chapter::FinalDream => "宿命水影散入终门，这一世的选择有了回声。",
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

    fn visual_motif(self) -> &'static str {
        match self {
            Chapter::VillageOath => "余杭村灯、青竹山雾、红衣剑光压在草坡尽头。",
            Chapter::MoonCave => "水月洞门映着冷蓝石纹，两座晶阵在回廊深处呼吸。",
            Chapter::RiverMedicine => "江岸药炉、倒流河灯与芦滩水线被雾色连成一幅夜行图。",
            Chapter::PlagueRain => "黑雨落在病屋瓦檐，旧祠净瘴铃从草影里透出绿光。",
            Chapter::CapitalMirror => "云都灯火照亮朱墙，偏院镜廊把密札与人影切成碎面。",
            Chapter::SouthernThunder => "南疆雷草坡翻起青金电纹，旧战鼓与图腾在山风里发亮。",
            Chapter::FinalDream => "灵渊终门沉在紫蓝梦水中，三盏忆梦灯照出归路与宿命水影。",
        }
    }

    fn tone_line(self) -> &'static str {
        match self {
            Chapter::VillageOath => "初遇与出走，轻快里压着第一层妖雾。",
            Chapter::MoonCave => "清冷、试炼、心事照水，队伍第一次学会并肩。",
            Chapter::RiverMedicine => "人间烟火被河雾侵蚀，救人与追妖并行。",
            Chapter::PlagueRain => "苦雨、病声、旧祠铃音，节奏更长也更沉。",
            Chapter::CapitalMirror => "繁华表面下是疑心与证词，行动要像潜入一样收束。",
            Chapter::SouthernThunder => "誓言、部族与雷声压路，后期力量开始成形。",
            Chapter::FinalDream => "旧梦回潮，温柔和宿命同时逼近终局。",
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

    fn care_scenes(self) -> (BondScene, CampScene) {
        match self {
            Chapter::VillageOath => (BondScene::VillageFirstNight, CampScene::VillageHearth),
            Chapter::MoonCave => (BondScene::MoonCavePromise, CampScene::MoonCavePool),
            Chapter::RiverMedicine => (BondScene::RiverLampWish, CampScene::RiverTownInn),
            Chapter::PlagueRain => (BondScene::PlagueRainShelter, CampScene::PlagueSickroom),
            Chapter::CapitalMirror => (BondScene::CapitalRooftop, CampScene::CapitalSafehouse),
            Chapter::SouthernThunder => (BondScene::SouthernRoadOath, CampScene::SouthernCampfire),
            Chapter::FinalDream => (BondScene::FinalGateQuiet, CampScene::FinalStillWater),
        }
    }

    fn commission_side_quests(self) -> [SideQuest; 2] {
        match self {
            Chapter::VillageOath => [SideQuest::VillageTrail, SideQuest::VillageHerbs],
            Chapter::MoonCave => [SideQuest::MoonCaveCrystals, SideQuest::MoonCaveEchoes],
            Chapter::RiverMedicine => [SideQuest::RiverLanterns, SideQuest::RiverCargo],
            Chapter::PlagueRain => [SideQuest::PlagueRelief, SideQuest::PlagueMedicine],
            Chapter::CapitalMirror => [SideQuest::CapitalPatrol, SideQuest::CapitalRumors],
            Chapter::SouthernThunder => [SideQuest::SouthernThunder, SideQuest::SouthernDrums],
            Chapter::FinalDream => [SideQuest::FinalDreamEchoes, SideQuest::FinalHomewardVows],
        }
    }

    fn care_aftermath_reward(self) -> CareAftermathReward {
        match self {
            Chapter::VillageOath => CareAftermathReward {
                exp: 16,
                potions: 1,
                gold: 10,
            },
            Chapter::MoonCave => CareAftermathReward {
                exp: 24,
                potions: 1,
                gold: 16,
            },
            Chapter::RiverMedicine => CareAftermathReward {
                exp: 32,
                potions: 1,
                gold: 22,
            },
            Chapter::PlagueRain => CareAftermathReward {
                exp: 42,
                potions: 2,
                gold: 28,
            },
            Chapter::CapitalMirror => CareAftermathReward {
                exp: 52,
                potions: 1,
                gold: 38,
            },
            Chapter::SouthernThunder => CareAftermathReward {
                exp: 62,
                potions: 2,
                gold: 44,
            },
            Chapter::FinalDream => CareAftermathReward {
                exp: 76,
                potions: 2,
                gold: 54,
            },
        }
    }

    fn commission_aftermath_reward(self) -> CommissionAftermathReward {
        match self {
            Chapter::VillageOath => CommissionAftermathReward {
                exp: 18,
                potions: 1,
                gold: 14,
            },
            Chapter::MoonCave => CommissionAftermathReward {
                exp: 28,
                potions: 1,
                gold: 20,
            },
            Chapter::RiverMedicine => CommissionAftermathReward {
                exp: 36,
                potions: 1,
                gold: 28,
            },
            Chapter::PlagueRain => CommissionAftermathReward {
                exp: 46,
                potions: 2,
                gold: 34,
            },
            Chapter::CapitalMirror => CommissionAftermathReward {
                exp: 58,
                potions: 1,
                gold: 46,
            },
            Chapter::SouthernThunder => CommissionAftermathReward {
                exp: 68,
                potions: 2,
                gold: 54,
            },
            Chapter::FinalDream => CommissionAftermathReward {
                exp: 84,
                potions: 2,
                gold: 66,
            },
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

impl CompanionScene {
    fn index(self) -> usize {
        match self {
            CompanionScene::SwordSisterTrailGuard => 0,
            CompanionScene::SwordSisterCapitalMirror => 1,
            CompanionScene::SpiritWitchSouthernTotem => 2,
            CompanionScene::SwordSisterFinalReturn => 3,
            CompanionScene::SpiritWitchFinalVow => 4,
        }
    }

    pub fn title(self) -> &'static str {
        match self {
            CompanionScene::SwordSisterTrailGuard => "月衡旧栅守剑",
            CompanionScene::SwordSisterCapitalMirror => "月衡照影旧案",
            CompanionScene::SpiritWitchSouthernTotem => "南瑶雷纹旧愿",
            CompanionScene::SwordSisterFinalReturn => "月衡终门归剑",
            CompanionScene::SpiritWitchFinalVow => "南瑶归潮灯誓",
        }
    }

    pub fn speaker(self) -> &'static str {
        match self {
            CompanionScene::SwordSisterTrailGuard
            | CompanionScene::SwordSisterCapitalMirror
            | CompanionScene::SwordSisterFinalReturn => "林月衡",
            CompanionScene::SpiritWitchSouthernTotem | CompanionScene::SpiritWitchFinalVow => {
                "南瑶"
            }
        }
    }

    fn reward(self) -> CompanionSceneReward {
        match self {
            CompanionScene::SwordSisterTrailGuard => CompanionSceneReward {
                exp: 20,
                potions: 0,
            },
            CompanionScene::SwordSisterCapitalMirror => CompanionSceneReward {
                exp: 42,
                potions: 1,
            },
            CompanionScene::SpiritWitchSouthernTotem => CompanionSceneReward {
                exp: 50,
                potions: 1,
            },
            CompanionScene::SwordSisterFinalReturn => CompanionSceneReward {
                exp: 58,
                potions: 1,
            },
            CompanionScene::SpiritWitchFinalVow => CompanionSceneReward {
                exp: 58,
                potions: 1,
            },
        }
    }

    fn aftermath_reward(self) -> CompanionAftermathReward {
        match self {
            CompanionScene::SwordSisterTrailGuard => CompanionAftermathReward {
                exp: 16,
                potions: 0,
                gold: 10,
            },
            CompanionScene::SwordSisterCapitalMirror => CompanionAftermathReward {
                exp: 32,
                potions: 1,
                gold: 22,
            },
            CompanionScene::SpiritWitchSouthernTotem => CompanionAftermathReward {
                exp: 38,
                potions: 1,
                gold: 26,
            },
            CompanionScene::SwordSisterFinalReturn | CompanionScene::SpiritWitchFinalVow => {
                CompanionAftermathReward {
                    exp: 44,
                    potions: 1,
                    gold: 32,
                }
            }
        }
    }

    fn tactic_bonus(self) -> CampBonus {
        match self {
            CompanionScene::SwordSisterTrailGuard
            | CompanionScene::SwordSisterCapitalMirror
            | CompanionScene::SwordSisterFinalReturn => CampBonus::Focus,
            CompanionScene::SpiritWitchSouthernTotem | CompanionScene::SpiritWitchFinalVow => {
                CampBonus::Vigil
            }
        }
    }

    fn lines(self) -> &'static [&'static str] {
        match self {
            CompanionScene::SwordSisterTrailGuard => &[
                "林月衡把旧竹栅上的断绳重新系好，像在确认一条没人守的退路。",
                "林月衡：我以前只管把妖挡在外面，现在才知道，路也要有人记住。",
                "李逍遥：那以后你记路，我负责别走丢。",
                "林月衡：少贫。真走丢了，我会把你拎回来。",
            ],
            CompanionScene::SwordSisterCapitalMirror => &[
                "林月衡盯着照影府邸的灯影，指尖按在剑鞘旧痕上。",
                "林月衡：镜阵最会把人照成自己怕的样子，我怕的是拔剑太晚。",
                "赵灵儿：所以这一次，我们会一起看清它。",
                "林月衡：嗯。账镜照人，剑也照人。",
            ],
            CompanionScene::SpiritWitchSouthernTotem => &[
                "南瑶把雷纹药草压进掌心，图腾火光映出她袖口的旧铃。",
                "南瑶：百越人说，雷不是怒，是祖灵怕后人忘路。",
                "李逍遥：那你听见的是催我们快走，还是叫我们慢些？",
                "南瑶：都不是。它叫我别再一个人守。",
            ],
            CompanionScene::SwordSisterFinalReturn => &[
                "林月衡在终门前解下剑穗，把它压到归路灯边。",
                "林月衡：若最后只能有一个人回去，我会先把路斩开。",
                "赵灵儿：别说这种话。我们不是来少一个人的。",
                "林月衡：知道。所以我把剑放在这里，提醒自己别只会断后。",
            ],
            CompanionScene::SpiritWitchFinalVow => &[
                "南瑶听着归潮灯签的水声，低声念出百越古誓。",
                "南瑶：水会带走愿，也会带回人。只要有人还记得名字。",
                "李逍遥：那就把大家的名字都记上。",
                "南瑶：已经记了。连你嘴硬的时候也记了。",
            ],
        }
    }
}

fn companion_scene_bit(scene: CompanionScene) -> u32 {
    debug_assert!(scene.index() < COMPANION_SCENE_COUNT);
    1 << scene.index()
}

impl CompanionRevisit {
    fn index(self) -> usize {
        match self {
            CompanionRevisit::TrailEcho => 0,
            CompanionRevisit::MirrorTrace => 1,
            CompanionRevisit::TotemVow => 2,
        }
    }

    pub fn scene(self) -> CompanionScene {
        match self {
            CompanionRevisit::TrailEcho => CompanionScene::SwordSisterTrailGuard,
            CompanionRevisit::MirrorTrace => CompanionScene::SwordSisterCapitalMirror,
            CompanionRevisit::TotemVow => CompanionScene::SpiritWitchSouthernTotem,
        }
    }

    fn required_companion(self) -> Companion {
        match self {
            CompanionRevisit::TrailEcho | CompanionRevisit::MirrorTrace => Companion::SwordSister,
            CompanionRevisit::TotemVow => Companion::SpiritWitch,
        }
    }

    fn unlock_chapter(self) -> Chapter {
        match self {
            CompanionRevisit::TrailEcho => Chapter::PlagueRain,
            CompanionRevisit::MirrorTrace => Chapter::SouthernThunder,
            CompanionRevisit::TotemVow => Chapter::FinalDream,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            CompanionRevisit::TrailEcho => "旧栅余声",
            CompanionRevisit::MirrorTrace => "照影迟问",
            CompanionRevisit::TotemVow => "雷纹归愿",
        }
    }

    pub fn receipt_id(self) -> &'static str {
        match self {
            CompanionRevisit::TrailEcho => "小传补访-01",
            CompanionRevisit::MirrorTrace => "小传补访-02",
            CompanionRevisit::TotemVow => "小传补访-03",
        }
    }

    pub fn issuer(self) -> &'static str {
        match self {
            CompanionRevisit::TrailEcho => "游方僧",
            CompanionRevisit::MirrorTrace => "红衣密探",
            CompanionRevisit::TotemVow => "终门守卫",
        }
    }

    pub fn target_place(self) -> &'static str {
        match self {
            CompanionRevisit::TrailEcho => "瘴雨祠道",
            CompanionRevisit::MirrorTrace => "雷鼓祭道",
            CompanionRevisit::TotemVow => "旧梦水廊",
        }
    }

    pub fn turn_in_place(self) -> &'static str {
        match self {
            CompanionRevisit::TrailEcho => "瘴雨村·游方僧",
            CompanionRevisit::MirrorTrace => "南疆灵道·红衣密探",
            CompanionRevisit::TotemVow => "灵渊终门·终门守卫",
        }
    }

    pub fn target_mark(self) -> RouteMark {
        match self {
            CompanionRevisit::TrailEcho => RouteMark::PlagueBell,
            CompanionRevisit::MirrorTrace => RouteMark::ThunderSwitchback,
            CompanionRevisit::TotemVow => RouteMark::DreamReturn,
        }
    }

    pub fn for_target_mark(mark: RouteMark) -> Option<Self> {
        ALL_COMPANION_REVISITS
            .iter()
            .copied()
            .find(|revisit| revisit.target_mark() == mark)
    }

    pub fn route_hint(self) -> &'static str {
        match self {
            CompanionRevisit::TrailEcho => "从瘴雨村进入祠道，在旧铃路印旁找竹绳结留下的剑痕。",
            CompanionRevisit::MirrorTrace => {
                "从南疆灵道进入雷鼓祭道，在回坡路印下听镜片与雷声相撞。"
            }
            CompanionRevisit::TotemVow => "从灵渊终门进入旧梦水廊，在归水路印中寻南瑶旧铃的回声。",
        }
    }

    fn accept_line(self) -> &'static str {
        match self {
            CompanionRevisit::TrailEcho => {
                "游方僧在瘴雨里捡到一枚竹绳结，月衡认得那是旧栅留下的退路记号。"
            }
            CompanionRevisit::MirrorTrace => {
                "红衣密探从云都带来半片照影镜，月衡看见镜背刻着当年没问完的案号。"
            }
            CompanionRevisit::TotemVow => {
                "终门守卫巡灯时听见水里反复念着百越旧誓，南瑶袖中的旧铃也跟着发响。"
            }
        }
    }

    fn progress_line(self) -> &'static str {
        match self {
            CompanionRevisit::TrailEcho => "带月衡去祠道旧铃路印，让竹绳结接回旧栅那夜。",
            CompanionRevisit::MirrorTrace => "带月衡去雷道回坡路印，把镜背案号放进雷光里照清。",
            CompanionRevisit::TotemVow => "带南瑶去旧梦归水路印，听完水声里迟来的百越旧誓。",
        }
    }

    fn field_lines(self) -> &'static [&'static str] {
        match self {
            CompanionRevisit::TrailEcho => &[
                "祠道旧铃被瘴雨压得低鸣，竹绳结却在铃下轻轻晃动。",
                "林月衡：这是我守旧栅时系的结。那天我只顾着断后，没问你们会不会回头。",
                "李逍遥：我们现在不就绕回来了吗？晚是晚了点，路还在。",
                "林月衡：那就重新记一次。这回不是我守你们走，是我们一起把人带出去。",
            ],
            CompanionRevisit::MirrorTrace => &[
                "雷光穿过半片照影镜，镜背案号映成两道交错的剑痕。",
                "林月衡：我一直怕拔剑太晚，后来才知道，先问清谁在镜后也不算退。",
                "赵灵儿：迟来的问题仍然会有答案，只要我们肯一起看。",
                "林月衡：嗯。旧案不该只剩一把剑记得。",
            ],
            CompanionRevisit::TotemVow => &[
                "归水路印卷起细浪，南瑶袖口旧铃与水里的雷声一同响起。",
                "南瑶：原来祖灵不是催我回去，是在问我为什么又想一个人守。",
                "李逍遥：这题你现在会答了吧？",
                "南瑶：会。我会说，路上有人记得我的名字，也让我记得他们的。",
            ],
        }
    }

    fn turn_in_line(self) -> &'static str {
        match self {
            CompanionRevisit::TrailEcho => {
                "游方僧听完旧栅余声，把竹绳结系在路簿上：迟来的路也是归路。"
            }
            CompanionRevisit::MirrorTrace => {
                "红衣密探将照影碎片封进线报匣，说镜中的旧问已经有人亲口答过。"
            }
            CompanionRevisit::TotemVow => {
                "终门守卫把旧誓抄进归门簿，水声终于不再反复追问南瑶的名字。"
            }
        }
    }

    fn completed_line(self) -> &'static str {
        match self {
            CompanionRevisit::TrailEcho => "竹绳结留在路簿上，旧栅余声已经归档。",
            CompanionRevisit::MirrorTrace => "照影碎片埋在雷草下，迟来的旧问已经归档。",
            CompanionRevisit::TotemVow => "百越旧誓写进灯簿，雷纹归愿已经归档。",
        }
    }

    fn repeat_line(self) -> &'static str {
        match self {
            CompanionRevisit::TrailEcho => "旧铃仍系着竹绳结，月衡已经记住这条共同退路。",
            CompanionRevisit::MirrorTrace => "照影碎片只映出清亮雷光，旧案不再绕回剑鞘。",
            CompanionRevisit::TotemVow => "归水路印轻声回潮，南瑶的旧铃不再独自回应。",
        }
    }
}

fn companion_revisit_bit(revisit: CompanionRevisit) -> u32 {
    debug_assert!(revisit.index() < COMPANION_REVISIT_COUNT);
    1 << revisit.index()
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

impl FieldSupply {
    fn index(self) -> usize {
        match self {
            FieldSupply::VillageHerbs => 0,
            FieldSupply::BambooDew => 1,
            FieldSupply::CaveMoonMoss => 2,
            FieldSupply::MoonCorridorDust => 3,
            FieldSupply::RiverTeaChest => 4,
            FieldSupply::ReedLotusPods => 5,
            FieldSupply::PlagueCleanWater => 6,
            FieldSupply::ShrineAshRoots => 7,
            FieldSupply::CapitalTeaPacket => 8,
            FieldSupply::MansionPantry => 9,
            FieldSupply::MirrorPowder => 10,
            FieldSupply::SouthernPepper => 11,
            FieldSupply::ThunderHerbWine => 12,
            FieldSupply::FinalIncense => 13,
            FieldSupply::DreamPearlMoss => 14,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            FieldSupply::VillageHerbs => "村郊止血草",
            FieldSupply::BambooDew => "竹叶灵露",
            FieldSupply::CaveMoonMoss => "月洞青苔",
            FieldSupply::MoonCorridorDust => "回廊晶尘",
            FieldSupply::RiverTeaChest => "江岸茶包",
            FieldSupply::ReedLotusPods => "芦滩莲实",
            FieldSupply::PlagueCleanWater => "瘴雨净水",
            FieldSupply::ShrineAshRoots => "祠道灰根",
            FieldSupply::CapitalTeaPacket => "府城醒神茶",
            FieldSupply::MansionPantry => "府邸暗厨包",
            FieldSupply::MirrorPowder => "镜廊定影粉",
            FieldSupply::SouthernPepper => "南疆暖椒",
            FieldSupply::ThunderHerbWine => "雷鼓药酒",
            FieldSupply::FinalIncense => "终门安魂香",
            FieldSupply::DreamPearlMoss => "梦水珠苔",
        }
    }

    fn reward(self) -> SupplyReward {
        match self {
            FieldSupply::VillageHerbs => SupplyReward {
                exp: 4,
                potions: 1,
                gold: 0,
                hp: 14,
                mp: 0,
            },
            FieldSupply::BambooDew => SupplyReward {
                exp: 5,
                potions: 0,
                gold: 6,
                hp: 8,
                mp: 4,
            },
            FieldSupply::CaveMoonMoss => SupplyReward {
                exp: 6,
                potions: 0,
                gold: 8,
                hp: 0,
                mp: 8,
            },
            FieldSupply::MoonCorridorDust => SupplyReward {
                exp: 8,
                potions: 1,
                gold: 0,
                hp: 10,
                mp: 6,
            },
            FieldSupply::RiverTeaChest => SupplyReward {
                exp: 8,
                potions: 0,
                gold: 14,
                hp: 16,
                mp: 2,
            },
            FieldSupply::ReedLotusPods => SupplyReward {
                exp: 10,
                potions: 1,
                gold: 4,
                hp: 12,
                mp: 8,
            },
            FieldSupply::PlagueCleanWater => SupplyReward {
                exp: 11,
                potions: 1,
                gold: 0,
                hp: 22,
                mp: 0,
            },
            FieldSupply::ShrineAshRoots => SupplyReward {
                exp: 12,
                potions: 1,
                gold: 8,
                hp: 14,
                mp: 6,
            },
            FieldSupply::CapitalTeaPacket => SupplyReward {
                exp: 12,
                potions: 0,
                gold: 22,
                hp: 10,
                mp: 8,
            },
            FieldSupply::MansionPantry => SupplyReward {
                exp: 14,
                potions: 1,
                gold: 18,
                hp: 18,
                mp: 4,
            },
            FieldSupply::MirrorPowder => SupplyReward {
                exp: 15,
                potions: 0,
                gold: 24,
                hp: 0,
                mp: 14,
            },
            FieldSupply::SouthernPepper => SupplyReward {
                exp: 16,
                potions: 1,
                gold: 10,
                hp: 24,
                mp: 0,
            },
            FieldSupply::ThunderHerbWine => SupplyReward {
                exp: 18,
                potions: 1,
                gold: 18,
                hp: 20,
                mp: 10,
            },
            FieldSupply::FinalIncense => SupplyReward {
                exp: 20,
                potions: 1,
                gold: 28,
                hp: 16,
                mp: 14,
            },
            FieldSupply::DreamPearlMoss => SupplyReward {
                exp: 22,
                potions: 1,
                gold: 32,
                hp: 24,
                mp: 12,
            },
        }
    }

    fn gather_line(self) -> &'static str {
        match self {
            FieldSupply::VillageHerbs => "草叶还带露水，像村人特意留给赶路人的药草。",
            FieldSupply::BambooDew => "竹节里积着清露，入口有一点灵气回甜。",
            FieldSupply::CaveMoonMoss => "石缝青苔泛着月色，摘下时掌心微凉。",
            FieldSupply::MoonCorridorDust => "晶尘贴在岩边，轻轻一拢便化成可入符袋的粉末。",
            FieldSupply::RiverTeaChest => "茶包压在半湿木箱下，还留着江岸人的草木香。",
            FieldSupply::ReedLotusPods => "莲实藏在芦叶后，拨开水珠才看见。",
            FieldSupply::PlagueCleanWater => "雨水流过净草根，瘴味被压下几分。",
            FieldSupply::ShrineAshRoots => "香灰旁长出细根，灰白根须里有一点药性。",
            FieldSupply::CapitalTeaPacket => "茶摊遗下醒神包，封纸上写着小小的云都字样。",
            FieldSupply::MansionPantry => "暗厨木屉没有锁，里面还有巡夜人备下的干粮药包。",
            FieldSupply::MirrorPowder => "镜边落下定影粉，收起时能听见细碎回声。",
            FieldSupply::SouthernPepper => "暖椒挂在藤边，辛香足以驱散一段山寒。",
            FieldSupply::ThunderHerbWine => "药酒坛被雷纹封着，敲开后酒气带一点电光。",
            FieldSupply::FinalIncense => "安魂香沉在灯座边，点燃前已能让心神安定。",
            FieldSupply::DreamPearlMoss => "珠苔伏在梦水边，像把旧愿都凝成了一层细光。",
        }
    }

    fn find_line(self) -> &'static str {
        match self {
            FieldSupply::VillageHerbs => "药水一份，气血稍复。",
            FieldSupply::BambooDew => "六文钱，气血与灵力稍复。",
            FieldSupply::CaveMoonMoss => "八文钱，灵力回复。",
            FieldSupply::MoonCorridorDust => "药水一份，气血与灵力回复。",
            FieldSupply::RiverTeaChest => "十四文钱，气血稍复。",
            FieldSupply::ReedLotusPods => "药水一份，四文钱，气血与灵力回复。",
            FieldSupply::PlagueCleanWater => "药水一份，气血回复。",
            FieldSupply::ShrineAshRoots => "药水一份，八文钱，气血与灵力回复。",
            FieldSupply::CapitalTeaPacket => "二十二文钱，灵力回复。",
            FieldSupply::MansionPantry => "药水一份，十八文钱，气血回复。",
            FieldSupply::MirrorPowder => "二十四文钱，灵力大复。",
            FieldSupply::SouthernPepper => "药水一份，十文钱，气血回复。",
            FieldSupply::ThunderHerbWine => "药水一份，十八文钱，气血与灵力回复。",
            FieldSupply::FinalIncense => "药水一份，二十八文钱，心神回复。",
            FieldSupply::DreamPearlMoss => "药水一份，三十二文钱，气血与灵力回复。",
        }
    }

    fn empty_line(self) -> &'static str {
        match self {
            FieldSupply::VillageHerbs => "草茎已经折过，只剩露水。",
            FieldSupply::BambooDew => "竹节空了，风声从节孔里穿过去。",
            FieldSupply::CaveMoonMoss => "石缝只剩潮气。",
            FieldSupply::MoonCorridorDust => "晶面已经暗下去。",
            FieldSupply::RiverTeaChest => "木箱里只剩湿茶香。",
            FieldSupply::ReedLotusPods => "芦叶合拢，水面恢复平静。",
            FieldSupply::PlagueCleanWater => "净水已经取尽，草根慢慢回潮。",
            FieldSupply::ShrineAshRoots => "香灰被理平，药根已收走。",
            FieldSupply::CapitalTeaPacket => "茶摊角落空了。",
            FieldSupply::MansionPantry => "木屉只剩干粮碎屑。",
            FieldSupply::MirrorPowder => "镜边粉痕已经扫净。",
            FieldSupply::SouthernPepper => "藤上只剩几片发热的叶子。",
            FieldSupply::ThunderHerbWine => "酒坛空了，雷纹也淡了。",
            FieldSupply::FinalIncense => "灯座边只剩灰白香痕。",
            FieldSupply::DreamPearlMoss => "梦水边的细光已经被收尽。",
        }
    }
}

fn field_supply_bit(supply: FieldSupply) -> u32 {
    debug_assert!(supply.index() < FIELD_SUPPLY_COUNT);
    1 << supply.index()
}

impl ShopGear {
    fn index(self) -> usize {
        match self {
            ShopGear::VillageSwordTassel => 0,
            ShopGear::RiverSilkVest => 1,
            ShopGear::CapitalMirrorGuard => 2,
            ShopGear::SouthernThunderCharm => 3,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            ShopGear::VillageSwordTassel => "竹剑穗",
            ShopGear::RiverSilkVest => "江绫护衣",
            ShopGear::CapitalMirrorGuard => "照影护心镜",
            ShopGear::SouthernThunderCharm => "雷纹护符",
        }
    }

    fn short_name(self) -> &'static str {
        match self {
            ShopGear::VillageSwordTassel => "竹剑穗",
            ShopGear::RiverSilkVest => "江绫衣",
            ShopGear::CapitalMirrorGuard => "护心镜",
            ShopGear::SouthernThunderCharm => "雷纹符",
        }
    }
}

fn shop_gear_bit(gear: ShopGear) -> u32 {
    debug_assert!(gear.index() < SHOP_GEAR_COUNT);
    1 << gear.index()
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

    pub fn route_label(self) -> &'static str {
        match self {
            ShrineBlessing::Guard => "避开埋伏",
            ShrineBlessing::Sword => "引妖练剑",
            ShrineBlessing::Spirit => "灵路安定",
        }
    }

    pub fn route_line(self) -> &'static str {
        match self {
            ShrineBlessing::Guard => "行路时更不容易被草中妖影截住。",
            ShrineBlessing::Sword => "剑气会主动引出附近妖影，方便清任务和练级。",
            ShrineBlessing::Spirit => "灵息压住杂乱妖气，赶路时遭遇稍少。",
        }
    }

    pub fn encounter_rate_multiplier(self) -> f32 {
        match self {
            ShrineBlessing::Guard => 0.65,
            ShrineBlessing::Sword => 1.20,
            ShrineBlessing::Spirit => 0.85,
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

impl RouteMark {
    fn index(self) -> usize {
        match self {
            RouteMark::MoonEcho => 0,
            RouteMark::ReedFord => 1,
            RouteMark::PlagueBell => 2,
            RouteMark::MirrorSideDoor => 3,
            RouteMark::ThunderSwitchback => 4,
            RouteMark::DreamReturn => 5,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            RouteMark::MoonEcho => "水月回廊路印",
            RouteMark::ReedFord => "芦滩折水路印",
            RouteMark::PlagueBell => "祠道旧铃路印",
            RouteMark::MirrorSideDoor => "镜廊偏门路印",
            RouteMark::ThunderSwitchback => "雷道回坡路印",
            RouteMark::DreamReturn => "旧梦归水路印",
        }
    }

    fn open_line(self) -> &'static str {
        match self {
            RouteMark::MoonEcho => "你在青竹门柱上刻下回廊水声，记住哪段石壁会先回响。",
            RouteMark::ReedFord => "你把芦苇压成折水记号，认清了涨水时仍能落脚的浅滩。",
            RouteMark::PlagueBell => "晶痕里有旧铃余声，你照着铃响记下祠道瘴风的转向。",
            RouteMark::MirrorSideDoor => "偏门背后映出另一条影路，你用剑鞘在门框上做了暗记。",
            RouteMark::ThunderSwitchback => "雷纹从晶簇绕回山坡，你顺势记住避开乱雷的折返路。",
            RouteMark::DreamReturn => "梦水晶里映出回程灯影，你记下旧梦水廊不会再迷失的水纹。",
        }
    }

    fn repeat_line(self) -> &'static str {
        match self {
            RouteMark::MoonEcho => "回廊水声已经能分辨远近，不必再刻一次。",
            RouteMark::ReedFord => "芦苇折痕仍在，浅滩方向没有改变。",
            RouteMark::PlagueBell => "旧铃声已经记住，瘴风从哪边来很清楚。",
            RouteMark::MirrorSideDoor => "偏门暗记还在，镜影不会再绕乱方向。",
            RouteMark::ThunderSwitchback => "雷道折返处已经踏熟，乱雷暂时绕不开你们。",
            RouteMark::DreamReturn => "归水纹仍在梦里亮着，旧梦水廊的回程已经记下。",
        }
    }
}

impl RouteDetour {
    fn index(self) -> usize {
        match self {
            RouteDetour::MoonEchoPool => 0,
            RouteDetour::ReedHiddenFord => 1,
            RouteDetour::PlagueHerbTrail => 2,
            RouteDetour::MirrorServantDoor => 3,
            RouteDetour::ThunderRidgeCache => 4,
            RouteDetour::DreamBackwater => 5,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            RouteDetour::MoonEchoPool => "照水暗池",
            RouteDetour::ReedHiddenFord => "芦下隐渡",
            RouteDetour::PlagueHerbTrail => "祠旁药径",
            RouteDetour::MirrorServantDoor => "镜仆暗门",
            RouteDetour::ThunderRidgeCache => "雷脊旧藏",
            RouteDetour::DreamBackwater => "旧梦回湾",
        }
    }

    fn preview_line(self) -> &'static str {
        match self {
            RouteDetour::MoonEchoPool => "石壁后有一汪暗池，水声会提前泄露妖影来路。",
            RouteDetour::ReedHiddenFord => "芦苇根下露出一段浅渡，水痕像刚有人拖货经过。",
            RouteDetour::PlagueHerbTrail => "祠旁草色发白，细看能分出病草和可用药茎。",
            RouteDetour::MirrorServantDoor => "偏廊墙缝映出仆从暗门，门后有巡夜人留下的粉记。",
            RouteDetour::ThunderRidgeCache => "雷纹劈开石脊，旧藏口在鼓声间一闪一灭。",
            RouteDetour::DreamBackwater => "梦水倒流进一处回湾，水面浮着前人留下的灯签。",
        }
    }

    fn result_line(self, approach: RouteDetourApproach) -> &'static str {
        match (self, approach) {
            (RouteDetour::MoonEchoPool, RouteDetourApproach::Scout) => {
                "你蹲下听完三次回声，在池边系上布条，记住哪一道水响代表埋伏。"
            }
            (RouteDetour::MoonEchoPool, RouteDetourApproach::PressOn) => {
                "你踏着水声快步穿过，凭直觉避开最深的回响。"
            }
            (RouteDetour::ReedHiddenFord, RouteDetourApproach::Scout) => {
                "你把浮芦压成记号，又从泥里摸出一只湿货小袋。"
            }
            (RouteDetour::ReedHiddenFord, RouteDetourApproach::PressOn) => {
                "你沿浅渡急行，记住了能绕过涨水的直线。"
            }
            (RouteDetour::PlagueHerbTrail, RouteDetourApproach::Scout) => {
                "你分开病草，采下几株能压瘴气的苦药。"
            }
            (RouteDetour::PlagueHerbTrail, RouteDetourApproach::PressOn) => {
                "你捂住口鼻冲过药径，只记下瘴雨最薄的一段。"
            }
            (RouteDetour::MirrorServantDoor, RouteDetourApproach::Scout) => {
                "你按粉记推门，抄下仆从暗道的转角符号。"
            }
            (RouteDetour::MirrorServantDoor, RouteDetourApproach::PressOn) => {
                "你借镜影遮身穿门而过，抓住一道能省脚程的斜廊。"
            }
            (RouteDetour::ThunderRidgeCache, RouteDetourApproach::Scout) => {
                "你等雷声落完才启开石脊，取出族人藏下的药酒和路符。"
            }
            (RouteDetour::ThunderRidgeCache, RouteDetourApproach::PressOn) => {
                "你趁雷纹短暂停顿越过石脊，硬记下下一段避雷步点。"
            }
            (RouteDetour::DreamBackwater, RouteDetourApproach::Scout) => {
                "你把灯签顺水排开，梦水终于露出不会回卷的边线。"
            }
            (RouteDetour::DreamBackwater, RouteDetourApproach::PressOn) => {
                "你踩着回湾倒影疾行，靠一口气冲出梦水回卷。"
            }
        }
    }

    fn repeat_line(self, approach: RouteDetourApproach) -> &'static str {
        match (self, approach) {
            (RouteDetour::MoonEchoPool, RouteDetourApproach::Scout) => {
                "池边布条仍在，回声的远近已经分得清。"
            }
            (RouteDetour::MoonEchoPool, RouteDetourApproach::PressOn) => {
                "你还记得那段快步落点，不必再闯一次暗池。"
            }
            (RouteDetour::ReedHiddenFord, RouteDetourApproach::Scout) => {
                "压下的芦痕还在，浅渡方向没有改。"
            }
            (RouteDetour::ReedHiddenFord, RouteDetourApproach::PressOn) => {
                "浅渡直线已经记住，水涨前还能通行。"
            }
            (RouteDetour::PlagueHerbTrail, RouteDetourApproach::Scout) => {
                "病草和药茎已经分清，药径不再难认。"
            }
            (RouteDetour::PlagueHerbTrail, RouteDetourApproach::PressOn) => {
                "那段薄瘴方向还记得，回头不用久停。"
            }
            (RouteDetour::MirrorServantDoor, RouteDetourApproach::Scout) => {
                "仆从暗门的粉记还在，镜廊转角清楚许多。"
            }
            (RouteDetour::MirrorServantDoor, RouteDetourApproach::PressOn) => {
                "斜廊的镜影还在脑中，能省一段脚程。"
            }
            (RouteDetour::ThunderRidgeCache, RouteDetourApproach::Scout) => {
                "石脊旧藏已经开启，路符仍压住乱雷。"
            }
            (RouteDetour::ThunderRidgeCache, RouteDetourApproach::PressOn) => {
                "避雷步点还记得，乱雷短暂停顿时可以通过。"
            }
            (RouteDetour::DreamBackwater, RouteDetourApproach::Scout) => {
                "灯签仍顺水亮着，回湾不会再把人绕回原处。"
            }
            (RouteDetour::DreamBackwater, RouteDetourApproach::PressOn) => {
                "梦水回卷的节奏已经记住，能趁空穿过去。"
            }
        }
    }

    fn reward(self, approach: RouteDetourApproach) -> RouteDetourReward {
        match (self, approach) {
            (RouteDetour::MoonEchoPool, RouteDetourApproach::Scout) => RouteDetourReward {
                exp: 16,
                potions: 1,
                gold: 10,
            },
            (RouteDetour::MoonEchoPool, RouteDetourApproach::PressOn) => RouteDetourReward {
                exp: 22,
                potions: 0,
                gold: 4,
            },
            (RouteDetour::ReedHiddenFord, RouteDetourApproach::Scout) => RouteDetourReward {
                exp: 22,
                potions: 1,
                gold: 16,
            },
            (RouteDetour::ReedHiddenFord, RouteDetourApproach::PressOn) => RouteDetourReward {
                exp: 28,
                potions: 0,
                gold: 8,
            },
            (RouteDetour::PlagueHerbTrail, RouteDetourApproach::Scout) => RouteDetourReward {
                exp: 28,
                potions: 2,
                gold: 20,
            },
            (RouteDetour::PlagueHerbTrail, RouteDetourApproach::PressOn) => RouteDetourReward {
                exp: 36,
                potions: 1,
                gold: 10,
            },
            (RouteDetour::MirrorServantDoor, RouteDetourApproach::Scout) => RouteDetourReward {
                exp: 34,
                potions: 1,
                gold: 32,
            },
            (RouteDetour::MirrorServantDoor, RouteDetourApproach::PressOn) => RouteDetourReward {
                exp: 42,
                potions: 0,
                gold: 18,
            },
            (RouteDetour::ThunderRidgeCache, RouteDetourApproach::Scout) => RouteDetourReward {
                exp: 40,
                potions: 2,
                gold: 38,
            },
            (RouteDetour::ThunderRidgeCache, RouteDetourApproach::PressOn) => RouteDetourReward {
                exp: 50,
                potions: 1,
                gold: 20,
            },
            (RouteDetour::DreamBackwater, RouteDetourApproach::Scout) => RouteDetourReward {
                exp: 48,
                potions: 2,
                gold: 44,
            },
            (RouteDetour::DreamBackwater, RouteDetourApproach::PressOn) => RouteDetourReward {
                exp: 58,
                potions: 1,
                gold: 24,
            },
        }
    }

    fn report_line(self, approach: RouteDetourApproach) -> &'static str {
        match (self, approach) {
            (RouteDetour::MoonEchoPool, RouteDetourApproach::Scout) => {
                "月洞守夜人把暗池回声记成三段巡路暗号，后来的弟子能照着布条避开伏妖。"
            }
            (RouteDetour::MoonEchoPool, RouteDetourApproach::PressOn) => {
                "月洞守夜人只记下你的急行落点，提醒后来人快步通过，不要久听暗池。"
            }
            (RouteDetour::ReedHiddenFord, RouteDetourApproach::Scout) => {
                "江岸巡货人照着芦痕重画浅渡，夜里能多护一段湿货水线。"
            }
            (RouteDetour::ReedHiddenFord, RouteDetourApproach::PressOn) => {
                "江岸人记住那条涨水前可通的直线，只安排白日急行，不敢夜巡久停。"
            }
            (RouteDetour::PlagueHerbTrail, RouteDetourApproach::Scout) => {
                "瘴雨村人把病草和药茎分开晒好，病屋药锅多了一味能压黑雨的苦药。"
            }
            (RouteDetour::PlagueHerbTrail, RouteDetourApproach::PressOn) => {
                "瘴雨村人记下薄瘴方向，送药时能少绕一段，但采药仍要等风停。"
            }
            (RouteDetour::MirrorServantDoor, RouteDetourApproach::Scout) => {
                "府城暗线按转角粉记重排巡夜，镜廊偏门终于有了可退的暗号。"
            }
            (RouteDetour::MirrorServantDoor, RouteDetourApproach::PressOn) => {
                "府城暗线记下斜廊可用，只把它列作短途退路，仍缺完整暗门图。"
            }
            (RouteDetour::ThunderRidgeCache, RouteDetourApproach::Scout) => {
                "百越巡山人照着路符稳住石脊，乱雷再起时能先把族人带出鼓道。"
            }
            (RouteDetour::ThunderRidgeCache, RouteDetourApproach::PressOn) => {
                "百越巡山人记下避雷步点，知道何时能冲过石脊，却仍要等雷声露空。"
            }
            (RouteDetour::DreamBackwater, RouteDetourApproach::Scout) => {
                "守灯人把顺水灯签录进终门灯簿，旧梦回湾多了一条能照人的归线。"
            }
            (RouteDetour::DreamBackwater, RouteDetourApproach::PressOn) => {
                "守灯人记下回湾回卷的空隙，后来的灯影能趁一息水静穿过去。"
            }
        }
    }

    fn report_repeat_line(self, approach: RouteDetourApproach) -> &'static str {
        match approach {
            RouteDetourApproach::Scout => "这份细查路报已经入簿，当地人正按它慢慢改巡路。",
            RouteDetourApproach::PressOn => "这份快走路报已经入簿，当地人只在急行时照着走。",
        }
    }

    fn report_reward(self, approach: RouteDetourApproach) -> RouteDetourReportReward {
        let rank = self.index() as u32;
        match approach {
            RouteDetourApproach::Scout => RouteDetourReportReward {
                exp: 8 + rank * 4,
                potions: if rank >= 2 { 1 } else { 0 },
                gold: 8 + rank * 6,
            },
            RouteDetourApproach::PressOn => RouteDetourReportReward {
                exp: 12 + rank * 5,
                potions: 0,
                gold: 4 + rank * 4,
            },
        }
    }
}

impl RouteDetourApproach {
    pub fn action_label(self) -> &'static str {
        match self {
            RouteDetourApproach::Scout => "细查岔路",
            RouteDetourApproach::PressOn => "快步穿过",
        }
    }

    fn short_label(self) -> &'static str {
        match self {
            RouteDetourApproach::Scout => "细查",
            RouteDetourApproach::PressOn => "快走",
        }
    }

    fn effect_line(self) -> &'static str {
        match self {
            RouteDetourApproach::Scout => "细查记录会明显压低本路线后续遇妖压力。",
            RouteDetourApproach::PressOn => "快走只记住可通行方向，本路线后续遇妖压力小幅降低。",
        }
    }
}

impl NpcErrand {
    fn index(self) -> usize {
        match self {
            Self::BambooDewToCave => 0,
            Self::MoonMossToRiver => 1,
            Self::RiverReedLetter => 2,
            Self::PlagueChildCharm => 3,
            Self::CapitalStarSlip => 4,
            Self::MirrorMedicineToSouth => 5,
            Self::SouthernThunderWine => 6,
            Self::FinalLampWick => 7,
        }
    }

    fn chapter(self) -> Chapter {
        match self {
            Self::BambooDewToCave => Chapter::VillageOath,
            Self::MoonMossToRiver => Chapter::MoonCave,
            Self::RiverReedLetter => Chapter::RiverMedicine,
            Self::PlagueChildCharm => Chapter::PlagueRain,
            Self::CapitalStarSlip | Self::MirrorMedicineToSouth => Chapter::CapitalMirror,
            Self::SouthernThunderWine => Chapter::SouthernThunder,
            Self::FinalLampWick => Chapter::FinalDream,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::BambooDewToCave => "竹露送药",
            Self::MoonMossToRiver => "月苔渡江",
            Self::RiverReedLetter => "芦滩小信",
            Self::PlagueChildCharm => "病童护符",
            Self::CapitalStarSlip => "星图密片",
            Self::MirrorMedicineToSouth => "镜廊药引",
            Self::SouthernThunderWine => "雷草药酒",
            Self::FinalLampWick => "旧梦灯芯",
        }
    }

    fn receipt_id(self) -> &'static str {
        match self {
            Self::BambooDewToCave => "托付-余杭-竹露",
            Self::MoonMossToRiver => "托付-水月-月苔",
            Self::RiverReedLetter => "托付-江岸-芦信",
            Self::PlagueChildCharm => "托付-瘴雨-童符",
            Self::CapitalStarSlip => "托付-京华-星片",
            Self::MirrorMedicineToSouth => "托付-镜廊-药引",
            Self::SouthernThunderWine => "托付-南疆-药酒",
            Self::FinalLampWick => "托付-终门-灯芯",
        }
    }

    fn issuer(self) -> &'static str {
        match self {
            Self::BambooDewToCave => "竹林猎户",
            Self::MoonMossToRiver => "采月人",
            Self::RiverReedLetter => "江岸货郎",
            Self::PlagueChildCharm => "病童母亲",
            Self::CapitalStarSlip => "观星吏",
            Self::MirrorMedicineToSouth => "夜行货郎",
            Self::SouthernThunderWine => "祭草医",
            Self::FinalLampWick => "灵渊狐影",
        }
    }

    fn receiver(self) -> &'static str {
        match self {
            Self::BambooDewToCave => "洞中采药人",
            Self::MoonMossToRiver => "江岸巡河卫",
            Self::RiverReedLetter => "巡滩猎户",
            Self::PlagueChildCharm => "祠道采药妇",
            Self::CapitalStarSlip => "偏院暗线",
            Self::MirrorMedicineToSouth => "赶山人",
            Self::SouthernThunderWine => "雷鼓祭草医",
            Self::FinalLampWick => "守灯人影",
        }
    }

    fn objective(self) -> &'static str {
        match self {
            Self::BambooDewToCave => "把竹林晨露送给水月洞天采药人。",
            Self::MoonMossToRiver => "把水月回廊采下的月苔交给江岸巡河卫。",
            Self::RiverReedLetter => "把货郎写给巡滩人的小信带到江岸芦滩。",
            Self::PlagueChildCharm => "把病童母亲求来的护符交给祠道采药妇。",
            Self::CapitalStarSlip => "把观星吏藏下的星图密片送进偏院镜廊。",
            Self::MirrorMedicineToSouth => "把夜行货郎配好的照影药引带给南疆赶山人。",
            Self::SouthernThunderWine => "把南疆雷草药酒送到雷鼓祭道。",
            Self::FinalLampWick => "把灵渊狐影交出的旧梦灯芯送进回梦水廊。",
        }
    }

    fn route_hint(self) -> &'static str {
        match self {
            Self::BambooDewToCave => "从青竹山径东门进入水月洞天，找洞中采药人。",
            Self::MoonMossToRiver => "从水月回廊出洞后到江岸小镇，找守在光门旁的巡河卫。",
            Self::RiverReedLetter => "从江岸小镇东门下芦滩，找巡滩猎户。",
            Self::PlagueChildCharm => "从瘴雨村东门进入祠道，把护符交给采药妇。",
            Self::CapitalStarSlip => "从云都府城进入偏院镜廊，找点亮暗号的偏院暗线。",
            Self::MirrorMedicineToSouth => "镜阵事毕后往南疆灵道，找守山的赶山人。",
            Self::SouthernThunderWine => "从南疆灵道东门进雷鼓祭道，找祭草医。",
            Self::FinalLampWick => "从灵渊终门入旧梦水廊，把灯芯交给守灯人影。",
        }
    }

    fn turn_in_place(self) -> &'static str {
        match self {
            Self::BambooDewToCave => "水月洞天 · 洞中采药人",
            Self::MoonMossToRiver => "江岸小镇 · 巡河卫",
            Self::RiverReedLetter => "江岸芦滩 · 巡滩猎户",
            Self::PlagueChildCharm => "瘴雨祠道 · 祠道采药妇",
            Self::CapitalStarSlip => "照影镜廊 · 偏院暗线",
            Self::MirrorMedicineToSouth => "南疆灵道 · 赶山人",
            Self::SouthernThunderWine => "雷鼓祭道 · 雷鼓祭草医",
            Self::FinalLampWick => "旧梦水廊 · 守灯人影",
        }
    }

    fn accept_line(self) -> &'static str {
        match self {
            Self::BambooDewToCave => {
                "竹林猎户把一只封好的露竹筒递来，说洞里采药人缺这味晨露压月寒。"
            }
            Self::MoonMossToRiver => "采月人从石缝刮下一片银蓝月苔，说江岸守路人认得这味洞中寒药。",
            Self::RiverReedLetter => "江岸货郎把湿信塞进油纸里，托你转告芦滩巡路人今夜别走旧浅渡。",
            Self::PlagueChildCharm => "病童母亲把护符攥得发皱，只求祠道采药妇替孩子把药根带回来。",
            Self::CapitalStarSlip => {
                "观星吏把星图背面的半片密文折进袖口，请你带给偏院里还敢点灯的人。"
            }
            Self::MirrorMedicineToSouth => {
                "夜行货郎将一包压镜毒的药引扎紧，说南下灵道有人会用它辨出残影。"
            }
            Self::SouthernThunderWine => {
                "祭草医封住一小坛雷草药酒，说祭道那边的药炉被雷声震得快熄了。"
            }
            Self::FinalLampWick => "灵渊狐影衔来一缕冷灯芯，说旧梦水廊少这一点火，誓灯便照不远。",
        }
    }

    fn progress_line(self) -> &'static str {
        match self {
            Self::BambooDewToCave => "竹露还在筒里轻响，越早送进水月洞天，药性越稳。",
            Self::MoonMossToRiver => "月苔遇热会散，离开水月回廊后尽快带到江岸光门旁。",
            Self::RiverReedLetter => "油纸信不能沾水，沿江岸东门下芦滩去找巡滩猎户。",
            Self::PlagueChildCharm => "护符压着一点温热，去瘴雨祠道找采药妇，别在黑草地久停。",
            Self::CapitalStarSlip => "星图密片不能让府兵搜到，进偏院镜廊后找暗号灯下的线人。",
            Self::MirrorMedicineToSouth => "照影药引带着铜镜味，南疆赶山人能用它辨认镜阵残毒。",
            Self::SouthernThunderWine => "药酒坛口还冒着细雷，送到雷鼓祭道后能稳住下一段药炉。",
            Self::FinalLampWick => "旧梦灯芯怕忘川水声，带进回梦水廊后交给守灯人影。",
        }
    }

    fn turn_in_line(self) -> &'static str {
        match self {
            Self::BambooDewToCave => "采药人把竹露倒进月寒药钵，药气散开时，洞壁水光安静了一瞬。",
            Self::MoonMossToRiver => "巡河卫闻见月苔寒气，立刻把它压进渡口水符，江雾退开半丈。",
            Self::RiverReedLetter => {
                "巡滩猎户读完小信，立刻把旧浅渡从今夜巡路里划掉，又塞给你一包路钱。"
            }
            Self::PlagueChildCharm => {
                "采药妇把护符系到药篮上，说孩子今晚能多睡一会儿，回村时会带药根。"
            }
            Self::CapitalStarSlip => {
                "偏院暗线看完星图密文，在镜框背后添上一笔，终于确认国师换镜的时辰。"
            }
            Self::MirrorMedicineToSouth => {
                "赶山人把药引撒进雷草间，照影残毒立刻显出灰线，南下岔路清楚了一截。"
            }
            Self::SouthernThunderWine => {
                "祭草医把药酒倒入炉口，雷声被药香压低，祭道守夜人终于敢靠近鼓架。"
            }
            Self::FinalLampWick => "守灯人影把灯芯接入誓灯，小小火线沿水面铺开，旧梦不再遮住回程。",
        }
    }

    fn completed_line(self) -> &'static str {
        match self {
            Self::BambooDewToCave => "竹露已经入药，水月洞天的药钵不会再催。",
            Self::MoonMossToRiver => "月苔已压进渡口水符，江岸雾灯会替后来人多照一段。",
            Self::RiverReedLetter => "芦滩小信已送到，巡路人今晚会改走新浅渡。",
            Self::PlagueChildCharm => "病童护符已交，采药妇会把药根带回瘴雨村。",
            Self::CapitalStarSlip => "星图密片已交，偏院暗线知道国师何时换镜。",
            Self::MirrorMedicineToSouth => "照影药引已送到，南疆灵道能辨出京城残毒。",
            Self::SouthernThunderWine => "雷草药酒已送到，雷鼓祭道的药炉稳住了。",
            Self::FinalLampWick => "旧梦灯芯已接入誓灯，回梦水廊的归路亮了一截。",
        }
    }

    fn reward(self) -> NpcErrandReward {
        match self {
            Self::BambooDewToCave => NpcErrandReward {
                exp: 18,
                potions: 1,
                gold: 8,
                hp: 10,
                mp: 4,
            },
            Self::MoonMossToRiver => NpcErrandReward {
                exp: 28,
                potions: 0,
                gold: 18,
                hp: 8,
                mp: 8,
            },
            Self::RiverReedLetter => NpcErrandReward {
                exp: 34,
                potions: 0,
                gold: 28,
                hp: 8,
                mp: 0,
            },
            Self::PlagueChildCharm => NpcErrandReward {
                exp: 52,
                potions: 1,
                gold: 30,
                hp: 18,
                mp: 6,
            },
            Self::CapitalStarSlip => NpcErrandReward {
                exp: 62,
                potions: 0,
                gold: 42,
                hp: 10,
                mp: 10,
            },
            Self::MirrorMedicineToSouth => NpcErrandReward {
                exp: 70,
                potions: 1,
                gold: 48,
                hp: 14,
                mp: 8,
            },
            Self::SouthernThunderWine => NpcErrandReward {
                exp: 78,
                potions: 1,
                gold: 54,
                hp: 16,
                mp: 10,
            },
            Self::FinalLampWick => NpcErrandReward {
                exp: 96,
                potions: 1,
                gold: 72,
                hp: 20,
                mp: 12,
            },
        }
    }

    fn reward_text(self) -> String {
        let reward = self.reward();
        format!(
            "经验 +{} / 药水 +{} / 钱 +{}文 / 气血 +{} / 灵力 +{}",
            reward.exp, reward.potions, reward.gold, reward.hp, reward.mp
        )
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
            SideQuest::FinalDreamEchoes => 12,
            SideQuest::FinalHomewardVows => 13,
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
            SideQuest::FinalDreamEchoes => "梦灯余波",
            SideQuest::FinalHomewardVows => "归潮旧愿",
        }
    }

    fn receipt_id(self) -> &'static str {
        match self {
            SideQuest::VillageTrail => "余杭-巡山-壹",
            SideQuest::VillageHerbs => "余杭-药圃-贰",
            SideQuest::MoonCaveCrystals => "水月-晶尘-壹",
            SideQuest::MoonCaveEchoes => "水月-回声-贰",
            SideQuest::RiverLanterns => "江岸-河灯-壹",
            SideQuest::RiverCargo => "江岸-湿货-贰",
            SideQuest::PlagueRelief => "瘴雨-救急-壹",
            SideQuest::PlagueMedicine => "瘴雨-药童-贰",
            SideQuest::CapitalPatrol => "云都-巡查-壹",
            SideQuest::CapitalRumors => "云都-暗帖-贰",
            SideQuest::SouthernThunder => "南疆-巡雷-壹",
            SideQuest::SouthernDrums => "南疆-战鼓-贰",
            SideQuest::FinalDreamEchoes => "灵渊-梦灯-壹",
            SideQuest::FinalHomewardVows => "灵渊-归潮-贰",
        }
    }

    fn issuer(self) -> &'static str {
        match self {
            SideQuest::VillageTrail => "余杭巡山人",
            SideQuest::VillageHerbs => "药婆",
            SideQuest::MoonCaveCrystals => "月洞石牌",
            SideQuest::MoonCaveEchoes => "月洞守夜人",
            SideQuest::RiverLanterns => "码头灯户",
            SideQuest::RiverCargo => "江岸货主",
            SideQuest::PlagueRelief => "瘴雨村药锅",
            SideQuest::PlagueMedicine => "送药童子",
            SideQuest::CapitalPatrol => "府城密榜",
            SideQuest::CapitalRumors => "偏院暗线",
            SideQuest::SouthernThunder => "百越巡路人",
            SideQuest::SouthernDrums => "旧鼓祭司",
            SideQuest::FinalDreamEchoes => "终门灯簿",
            SideQuest::FinalHomewardVows => "归潮水签",
        }
    }

    pub fn prerequisite(self) -> Option<SideQuest> {
        match self {
            SideQuest::VillageHerbs => Some(SideQuest::VillageTrail),
            SideQuest::MoonCaveEchoes => Some(SideQuest::MoonCaveCrystals),
            SideQuest::RiverCargo => Some(SideQuest::RiverLanterns),
            SideQuest::PlagueMedicine => Some(SideQuest::PlagueRelief),
            SideQuest::CapitalRumors => Some(SideQuest::CapitalPatrol),
            SideQuest::SouthernDrums => Some(SideQuest::SouthernThunder),
            SideQuest::FinalHomewardVows => Some(SideQuest::FinalDreamEchoes),
            SideQuest::VillageTrail
            | SideQuest::MoonCaveCrystals
            | SideQuest::RiverLanterns
            | SideQuest::PlagueRelief
            | SideQuest::CapitalPatrol
            | SideQuest::SouthernThunder
            | SideQuest::FinalDreamEchoes => None,
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
            SideQuest::FinalDreamEchoes => 3,
            SideQuest::FinalHomewardVows => 2,
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
            SideQuest::FinalDreamEchoes => SideQuestReward {
                exp: 92,
                potions: 2,
                gold: 84,
            },
            SideQuest::FinalHomewardVows => SideQuestReward {
                exp: 104,
                potions: 2,
                gold: 96,
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
            SideQuest::FinalDreamEchoes => "压住三段旧梦水影，回灵渊终门任务板交付。",
            SideQuest::FinalHomewardVows => "护住两枚归潮灯签，回灵渊终门任务板交付。",
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
            SideQuest::FinalDreamEchoes => "灵渊终门梦灯水阶",
            SideQuest::FinalHomewardVows => "旧梦水廊归潮灯签",
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
            SideQuest::FinalDreamEchoes | SideQuest::FinalHomewardVows => "回灵渊终门任务板交付。",
        }
    }

    fn route_hint(self) -> &'static str {
        match self {
            SideQuest::VillageTrail => "从村郊北侧草坡绕到旧竹栅，靠近草丛就会引出余妖。",
            SideQuest::VillageHerbs => "沿村郊药圃外圈巡到水塘西侧，香气最重处会起妖风。",
            SideQuest::MoonCaveCrystals => "从水月洞天东门进晶尘石道，先查靠水的晶簇。",
            SideQuest::MoonCaveEchoes => "沿回声窄廊贴着石壁走，听见二重回响就停步压阵。",
            SideQuest::RiverLanterns => "从江岸小镇东门下芦滩，顺着上游、中洲、渡口三点巡夜。",
            SideQuest::RiverCargo => "绕到江岸草滩的湿货残道，水线拐弯处最容易被妖风截住。",
            SideQuest::PlagueRelief => "从瘴雨村外黑雨草地推进，先清靠病屋的瘴草。",
            SideQuest::PlagueMedicine => "沿病屋药路护送，铃声变急时就地迎敌。",
            SideQuest::CapitalPatrol => "从云都府城暗巷绕到府门阴影，镜阵残影会主动现形。",
            SideQuest::CapitalRumors => "查茶肆外暗帖，再顺着墙根追到偏院回廊。",
            SideQuest::SouthernThunder => "沿南疆灵道雷草坡前进，雷纹跳到脚边时停步安抚。",
            SideQuest::SouthernDrums => "贴着旧鼓架外圈巡回，听见低鼓声就靠近压住怨音。",
            SideQuest::FinalDreamEchoes => "从灵渊终门东门入旧梦水廊，先听梦水回声再靠近灯影。",
            SideQuest::FinalHomewardVows => "沿旧梦水廊回湾巡到归潮灯签处，水面回卷时停步护灯。",
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
            SideQuest::FinalDreamEchoes => "在旧梦水廊压住 3 段旧梦水影。",
            SideQuest::FinalHomewardVows => "在旧梦水廊护住 2 枚归潮灯签。",
        }
    }

    fn field_steps(self) -> &'static [&'static str] {
        match self {
            SideQuest::VillageTrail => &["踏查旧竹栅草丛", "清理退路余妖"],
            SideQuest::VillageHerbs => &["护到药圃水塘西侧", "压住闻香妖风"],
            SideQuest::MoonCaveCrystals => &["净化靠水晶簇", "封住晶尘石道"],
            SideQuest::MoonCaveEchoes => &["听定第一段回声", "压住窄廊二重影", "封回石壁尾声"],
            SideQuest::RiverLanterns => &["巡上游逆流灯", "查中洲与渡口灯影"],
            SideQuest::RiverCargo => &["找回水线货袋", "护送湿货回岸"],
            SideQuest::PlagueRelief => &["清病屋前瘴草", "开苦井旁药路", "稳住旧祠雨口"],
            SideQuest::PlagueMedicine => &["护药童到病屋前", "压住返程瘴影"],
            SideQuest::CapitalPatrol => &["巡暗巷镜影", "截府门残阵"],
            SideQuest::CapitalRumors => &["查茶肆暗帖", "追墙根传手", "封偏院回廊线索"],
            SideQuest::SouthernThunder => &["听雷草坡乱雷", "安南侧雷纹", "引雷声归路"],
            SideQuest::SouthernDrums => &["平旧鼓架低怨", "封战鼓回响"],
            SideQuest::FinalDreamEchoes => &["压水阶旧梦影", "守中湾灯下回声", "送尾波归灯外"],
            SideQuest::FinalHomewardVows => &["护第一枚归潮灯签", "稳回湾旧愿水线"],
        }
    }

    fn field_step(self, index: usize) -> &'static str {
        self.field_steps()
            .get(index)
            .copied()
            .unwrap_or_else(|| self.tracking_hint())
    }

    fn step_plan(self) -> String {
        let mut steps = Vec::with_capacity(self.field_steps().len() + 2);
        steps.push("签收委托".to_string());
        for (index, step) in self.field_steps().iter().enumerate() {
            steps.push(format!("{}. {step}", index + 1));
        }
        steps.push(format!("交付裁断：{}", self.turn_in_place()));
        steps.join(" -> ")
    }

    fn next_step_for_progress(self, progress: u32) -> String {
        let progress = progress.min(self.goal());
        if progress >= self.goal() {
            return format!("现场已完成，{}", self.turn_in_place());
        }

        format!(
            "第 {}/{} 步：{}。",
            progress + 1,
            self.goal(),
            self.field_step(progress as usize)
        )
    }

    fn party_focus(self) -> &'static str {
        match self {
            SideQuest::VillageTrail => "山路余妖",
            SideQuest::VillageHerbs => "药圃妖香",
            SideQuest::MoonCaveCrystals => "水月晶尘",
            SideQuest::MoonCaveEchoes => "回声妖影",
            SideQuest::RiverLanterns => "逆流河灯",
            SideQuest::RiverCargo => "草滩湿货",
            SideQuest::PlagueRelief => "瘴草妖影",
            SideQuest::PlagueMedicine => "病屋药路",
            SideQuest::CapitalPatrol => "镜阵残影",
            SideQuest::CapitalRumors => "府城暗帖",
            SideQuest::SouthernThunder => "灵道雷声",
            SideQuest::SouthernDrums => "旧鼓怨音",
            SideQuest::FinalDreamEchoes => "旧梦水影",
            SideQuest::FinalHomewardVows => "归潮灯签",
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
            SideQuest::FinalDreamEchoes => {
                "终门灯簿浮字：梦灯初亮，旧梦余波还会拖住归人，愿行者替灯守夜。"
            }
            SideQuest::FinalHomewardVows => "归潮水签又浮一行：灯签将归，仍有两处旧愿不肯离水。",
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
            SideQuest::FinalDreamEchoes => "梦灯余波还在水阶下回响，继续去旧梦水廊压住水影。",
            SideQuest::FinalHomewardVows => "归潮灯签还未全稳，继续沿回湾巡护。",
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
            SideQuest::FinalDreamEchoes => "终门灯簿合上一页，旧梦水影退到灯外。",
            SideQuest::FinalHomewardVows => "归潮水签顺流而下，守灯人替归路添了一盏小灯。",
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
            SideQuest::FinalDreamEchoes => "梦灯余波已平，终门水阶能听见归声。",
            SideQuest::FinalHomewardVows => "归潮旧愿已护住，归路少了一段回卷。",
        }
    }

    fn resolution_turn_in_line(self, resolution: SideQuestResolution) -> String {
        match resolution {
            SideQuestResolution::Settle => format!(
                "{}把《{}》封进旧签，{}先按原路守住。",
                self.issuer(),
                self.name(),
                self.party_focus()
            ),
            SideQuestResolution::Pursue => format!(
                "{}请你们记下余波回访点，{}若再起会先从任务簿预警。",
                self.issuer(),
                self.party_focus()
            ),
        }
    }

    fn resolution_reaction_line(self, resolution: SideQuestResolution) -> String {
        match resolution {
            SideQuestResolution::Settle => format!(
                "【委托裁断】稳妥封存：{}说《{}》已经按旧约收匣，{}暂时不会惊动旁人。",
                self.issuer(),
                self.name(),
                self.party_focus()
            ),
            SideQuestResolution::Pursue => format!(
                "【委托裁断】追查余波：{}把《{}》余线留在路簿上，{}的回声还会提醒巡路人。",
                self.issuer(),
                self.name(),
                self.party_focus()
            ),
        }
    }
}

impl SideQuestPartyVoice {
    pub fn name(self) -> &'static str {
        match self {
            Self::Hero => "李逍遥",
            Self::Linger => "赵灵儿",
            Self::SwordSister => "林月衡",
            Self::SpiritWitch => "南瑶",
        }
    }

    fn line(self, moment: SideQuestPartyMoment, focus: &'static str) -> String {
        match (self, moment) {
            (Self::Hero, SideQuestPartyMoment::Accept) => {
                format!("{focus}先记清楚，照着委托签一处一处走。")
            }
            (Self::Hero, SideQuestPartyMoment::Progress) => {
                format!("{focus}还没办完，再查一段，别让任务签空着。")
            }
            (Self::Hero, SideQuestPartyMoment::TurnIn) => {
                format!("{focus}已平，回去交签，别让等消息的人白等。")
            }
            (Self::Linger, SideQuestPartyMoment::Accept) => {
                format!("{focus}的灵息不稳，我会一路听着。")
            }
            (Self::Linger, SideQuestPartyMoment::Progress) => {
                format!("{focus}还有回声，慢一点，别让妖气贴近。")
            }
            (Self::Linger, SideQuestPartyMoment::TurnIn) => {
                format!("{focus}安静下来了，等这张签交回去，他们会安心些。")
            }
            (Self::SwordSister, SideQuestPartyMoment::Accept) => {
                format!("{focus}若起妖影，我来断后，你按路线走。")
            }
            (Self::SwordSister, SideQuestPartyMoment::Progress) => {
                format!("{focus}还留着破口，别绕远，顺势压过去。")
            }
            (Self::SwordSister, SideQuestPartyMoment::TurnIn) => {
                format!("{focus}已收住，赏钱不急，先把后路记稳。")
            }
            (Self::SpiritWitch, SideQuestPartyMoment::Accept) => {
                format!("{focus}压着旧祭声，我替你们稳住回响。")
            }
            (Self::SpiritWitch, SideQuestPartyMoment::Progress) => {
                format!("{focus}还在水脉里回卷，听鼓点再往前。")
            }
            (Self::SpiritWitch, SideQuestPartyMoment::TurnIn) => {
                format!("{focus}已经归位，这条路会把你们送回灯下。")
            }
        }
    }
}

fn side_quest_bit(side: SideQuest) -> u32 {
    1 << side.index()
}

fn npc_errand_bit(errand: NpcErrand) -> u32 {
    1 << errand.index()
}

fn route_mark_bit(mark: RouteMark) -> u32 {
    1 << mark.index()
}

fn route_detour_bit(detour: RouteDetour) -> u32 {
    1 << detour.index()
}

fn companion_bit(companion: Companion) -> u32 {
    match companion {
        Companion::Linger => 1 << 0,
        Companion::SwordSister => 1 << 1,
        Companion::SpiritWitch => 1 << 2,
    }
}
