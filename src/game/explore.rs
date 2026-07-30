use bevy::{
    asset::RenderAssetUsages,
    ecs::system::SystemParam,
    image::ImageSampler,
    prelude::*,
    reflect::TypePath,
    render::render_resource::{AsBindGroup, Extent3d, ShaderType, TextureDimension, TextureFormat},
    shader::ShaderRef,
    sprite_render::{Material2d, Material2dPlugin},
};
use bevy_firefly::prelude::{Occluder2d, PointLight2d};

use super::animation::{self, AnimationAssets, AnimationClip, SpriteAnimation};
use super::battle::{EncounterKind, EncounterZone, PendingEncounter};
use super::core::{
    EncounterRate, GameFont, Intent, MAP_H, MAP_W, PlayerStats, Rng, TILE, tile_to_world,
};
use super::cutout::{
    CutoutPart, brighten_color, cutout_part_motion, cutout_part_specs, cutout_source_px_for_path,
};
use super::fog;
use super::lighting::{self, LightingAssets};
use super::paperdoll::{self, PaperdollAssets, PaperdollStyle};
use super::quest::{
    BondResponse, BondReward, BondScene, BossKind, CampBonus, CampScene, CareAftermathReward,
    Chapter, CommissionAftermathReward, Companion, CompanionAftermathReward, CompanionRevisit,
    CompanionScene, CompanionSceneReward, FieldSupply, FinalLamp, MansionMirrorNode, MoonCrystal,
    NpcErrand, NpcErrandReward, PlagueWard, QuestLog, QuestRole, QuestStage, RiverLantern,
    RouteDetour, RouteDetourApproach, RouteDetourReportReward, RouteDetourReward, RouteMark,
    ShopGear, ShrineBlessing, SideQuest, SideQuestFieldApproach, SideQuestResolution,
    SideQuestReward, SupplyReward, ThunderDrum, TreasureCache, TreasureReward,
};
use super::state::AppState;

const POTION_PRICE: u32 = 18;
const FAVORED_POTION_PRICE: u32 = 12;
const INN_PRICE: u32 = 24;
const FAVORED_INN_PRICE: u32 = 12;
const SHRINE_OFFERING_PRICE: u32 = 12;
const VILLAGE_GEAR_PRICE: u32 = 54;
const RIVER_GEAR_PRICE: u32 = 86;
const CAPITAL_GEAR_PRICE: u32 = 128;
const SOUTHERN_GEAR_PRICE: u32 = 166;
const CHAPTER_CARD_LINE_COUNT: usize = 6;
const CHAPTER_ART_FRAME_COUNT: usize = 32;
const CHAPTER_ART_FRAME_TIME: f32 = 1.0 / 16.0;

// ---------------------------------------------------------------------------
// Map definition
// ---------------------------------------------------------------------------

/// Each row must be exactly `MAP_W` chars and there must be `MAP_H` rows.
/// Legend:
///   `#` tree/wall   `~` water   `.` path   `,` grass encounter
///   `N` npc         `P` spawn    `>` portal to the next map
const MAP_VILLAGE: [&str; MAP_H as usize] = [
    "##############################",
    "#............,,,,,....N....>.#",
    "#.####.......,,,,,...........#",
    "#.####.......,,,,,..N####....#",
    "#.####...............####....#",
    "#........~~~~~.......####....#",
    "#........~~~~~.......####....#",
    "#..P..N..~~~~~...............#",
    "#...............N...,,,,.....#",
    "#.......####........,,,,.....#",
    "#.......####........,,,,.....#",
    "#.......####........,,,,.....#",
    "#.......####.................#",
    "#..........,,,,,.............#",
    "#..........,,,,,.............#",
    "##############################",
];

const MAP_BAMBOO: [&str; MAP_H as usize] = [
    "##############################",
    "#P......,,,,,,.......####...>#",
    "#.......,,,,,,.......####....#",
    "#..####.......~~~~..........N#",
    "#..####.......~~~~...........#",
    "#.............~~~~.....####..#",
    "#.....N...............####...#",
    "#........,,,,,,,,.N..........#",
    "#........,,,,,,,,............#",
    "#..####..............~~~~....#",
    "#..####..............~~~~....#",
    "#.............####...........#",
    "#.............####.....N.....#",
    "#....,,,,,...................#",
    "#....,,,,,...................#",
    "##############################",
];

const MAP_CAVE: [&str; MAP_H as usize] = [
    "##############################",
    "#P....####..............,,,,>#",
    "#.....####..............,,,,.#",
    "#............~~~~~~..........#",
    "#............~~~~~~....N.....#",
    "#..######....................#",
    "#..######.....,,,,,,.........#",
    "#.............,,,,,,.........#",
    "#.........N..................#",
    "#..............######........#",
    "#....~~~~......######........#",
    "#....~~~~....................#",
    "#.............,,,,,,....N....#",
    "#.............,,,,,,.........#",
    "#.............N..............#",
    "##############################",
];

const MAP_MOON_ECHO_CORRIDOR: [&str; MAP_H as usize] = [
    "##############################",
    "#P..,,,,....####......~~~~..>#",
    "#...,,,,....####......~~~~...#",
    "#..N....####.....,,,,....###.#",
    "#.......####.....,,,,....###.#",
    "#..~~~~......N.......####....#",
    "#..~~~~..............####....#",
    "#......,,,,,,.....~~~~.....N.#",
    "#..####.....~~~~.....####....#",
    "#..####.....~~~~.....####....#",
    "#....N......,,,,,,...........#",
    "#...........,,,,,,...........#",
    "#....####..........~~~~......#",
    "#....####...N......~~~~......#",
    "#.........N...,,,,,,.........#",
    "##############################",
];

const MAP_RIVER_TOWN: [&str; MAP_H as usize] = [
    "##############################",
    "#P......####....~~~~~....N>..#",
    "#.......####....~~~~~........#",
    "#..N.........~~~~~~.....###..#",
    "#............~~~~~~.....###..#",
    "#....####...........,,,,.....#",
    "#....####....N......,,,,.....#",
    "#............~~~~..........N.#",
    "#..####......~~~~......####..#",
    "#..####......~~~~......####..#",
    "#........,,,,,,............N.#",
    "#........,,,,,,..............#",
    "#....N..............~~~~.....#",
    "#.............####...~~~~....#",
    "#....,,,,,....####...........#",
    "##############################",
];

const MAP_RIVER_REED_BED: [&str; MAP_H as usize] = [
    "##############################",
    "#P..~~~~....,,,,..####.....>.#",
    "#...~~~~..N.,,,,.............#",
    "#..N....~~~~.....####..###...#",
    "#.......~~~~.....####..###...#",
    "#..,,,,......N....~~~~.......#",
    "#..,,,,..~~~~....N..~~~~.....#",
    "#......####,,,,,,....N..N....#",
    "#..~~~~####.....~~~~.........#",
    "#..~~~~....,,,,..####........#",
    "#....N....N,,,,..####........#",
    "#......~~~~....####..N.......#",
    "#..####..~~~~................#",
    "#..####......,,,,,,..........#",
    "#....,,,,,....~~~~...........#",
    "##############################",
];

const MAP_PLAGUE_VILLAGE: [&str; MAP_H as usize] = [
    "##############################",
    "#P..####..,,,,,,,,....N>.....#",
    "#...####..,,,,,,,,...........#",
    "#..N....~~~~~..####..........#",
    "#.......~~~~~..####..........#",
    "#..####......,,,,,,..........#",
    "#..####....N.,,,,,,..........#",
    "#......~~~~....####......N...#",
    "#..,,,,~~~~....####..........#",
    "#..,,,,....####.....~~~~.....#",
    "#......N...####.....~~~~.....#",
    "#....N.....,,,,,,............#",
    "#....N.....,,,,,,..####......#",
    "#..~~~~.........####.........#",
    "#..~~~~....,,,,,####.........#",
    "##############################",
];

const MAP_PLAGUE_SHRINE_PATH: [&str; MAP_H as usize] = [
    "##############################",
    "#P..~~~~....####....,,,,..N>.#",
    "#...~~~~....####....,,,,.....#",
    "#..####..N..~~~~.....###.....#",
    "#..####.....~~~~..,,,,.......#",
    "#..,,,,......N...####........#",
    "#..,,,,..~~~~....####........#",
    "#......######..~~~~.....N....#",
    "#..####..~~~~....N..,,,,.....#",
    "#..####..N.......~~~~........#",
    "#....N..,,,,,,....~~~~.......#",
    "#.......,,,,,,..####...N.....#",
    "#..~~~~....####......,,,,....#",
    "#..~~~~..N.####......,,,,....#",
    "#....####....,,,,.......N....#",
    "##############################",
];

const MAP_CAPITAL: [&str; MAP_H as usize] = [
    "##############################",
    "#P..####....,,,,.....N>......#",
    "#...####....,,,,..####.......#",
    "#..N....####.....~~~~..###...#",
    "#.......####.....~~~~..###...#",
    "#..,,,,......####............#",
    "#..,,,,..N...####.....####...#",
    "#.......~~~~.........N.####..#",
    "#..####..~~~~....,,,,........#",
    "#..####..........,,,,..####..#",
    "#......####,,,,,,......N.....#",
    "#......####,,,,,,............#",
    "#....N.....~~~~....####......#",
    "#..........~~~~....####......#",
    "#....,,,,.............####...#",
    "##############################",
];

const MAP_CAPITAL_MANSION: [&str; MAP_H as usize] = [
    "##############################",
    "#P..####..,,,,..####..N....>.#",
    "#...####..,,,,..####.........#",
    "#..N....~~~~~.....###........#",
    "#..####.~~~~~..#######.......#",
    "#....##......N....,,,,.......#",
    "#....##..####.....,,,,.......#",
    "#..~~~~..####.........N......#",
    "#..~~~~......####..~~~~......#",
    "#..####......####..~~~~......#",
    "#......,,,,,,....####...N....#",
    "#......,,,,,,....####........#",
    "#....N...####.....~~~~.......#",
    "#........####.....~~~~.......#",
    "#....,,,,....####............#",
    "##############################",
];

const MAP_MANSION_MIRROR_GALLERY: [&str; MAP_H as usize] = [
    "##############################",
    "#P....####....~~~~.....N...>.#",
    "#.....####....~~~~...........#",
    "#..N.........,,,,,.....###...#",
    "#............,,,,,.....###...#",
    "#....####...........~~~~.....#",
    "#....####....N......~~~~.....#",
    "#............,,,,..........N.#",
    "#..####......~~~~......####..#",
    "#..####......~~~~......####..#",
    "#........,,,,,,............N.#",
    "#........,,,,,,..............#",
    "#....N..............~~~~.....#",
    "#.............####...~~~~....#",
    "#....,,,,,....####...........#",
    "##############################",
];

const MAP_SOUTHERN_ROAD: [&str; MAP_H as usize] = [
    "##############################",
    "#P..,,,,....~~~~....####...N>#",
    "#...,,,,....~~~~.............#",
    "#..N....####.....,,,,....###.#",
    "#.......####.....,,,,....###.#",
    "#..~~~~......N.......####....#",
    "#..~~~~..............####....#",
    "#......,,,,,,.....~~~~.....N.#",
    "#..####.....~~~~.....####....#",
    "#..####.....~~~~.....####....#",
    "#....N......,,,,,,...........#",
    "#...........,,,,,,......N....#",
    "#....####..........~~~~......#",
    "#....####..N.......~~~~......#",
    "#....,,,,,......####.....N...#",
    "##############################",
];

const MAP_THUNDER_DRUM_PATH: [&str; MAP_H as usize] = [
    "##############################",
    "#P..~~~~....,,,,....####...N>#",
    "#...~~~~....,,,,....####.....#",
    "#..N..~~####.....,,,,....###.#",
    "#.....~~####.....,,,,....###.#",
    "#..####......N....~~~~.......#",
    "#..####....,,,,....~~~~......#",
    "#......~~~~~~....####.....N..#",
    "#..####..~~~~....####........#",
    "#..####..,,,,....~~~~........#",
    "#....N..,,,,,,....~~~~.......#",
    "#.......####,,,,,,....N......#",
    "#..~~~~.####......,,,,.......#",
    "#..~~~~....N..####,,,,.......#",
    "#....,,,,....####.......N....#",
    "##############################",
];

const MAP_FINAL_SANCTUM: [&str; MAP_H as usize] = [
    "##############################",
    "#P....####~~~~....####.N>....#",
    "#.....####~~~~....####.......#",
    "#..N....,,,,....~~~~....###..#",
    "#..####.,,,,....~~~~....###..#",
    "#..####......N....####.......#",
    "#......~~~~....N...####......#",
    "#..~~~~....,,,,......####.N..#",
    "#..~~~~....,,,,......####....#",
    "#....####..~~~~....,,,,......#",
    "#....####..~~~~..N.,,,,......#",
    "#..N.......####....~~~~......#",
    "#....N.....####....~~~~......#",
    "#......,,,,....####..........#",
    "#....,,,,......####..........#",
    "##############################",
];

const MAP_DREAM_WATERWAY: [&str; MAP_H as usize] = [
    "##############################",
    "#P..~~~~####....~~~~...N>....#",
    "#...~~~~####....~~~~.........#",
    "#..N....,,,,..~~~~....###....#",
    "#..####.,,,,..~~~~....###....#",
    "#..####.....N..,,,,..~~~~....#",
    "#......~~~~....N....~~~~.....#",
    "#..~~~~####....,,,,......N...#",
    "#..~~~~####....,,,,..####....#",
    "#....,,,,..~~~~....####......#",
    "#....,,,,..~~~~..N.####......#",
    "#..####......~~~~....,,,,....#",
    "#....N....####~~~~....,,,,...#",
    "#........####..~~~~.....N....#",
    "#....~~~~....####.....N......#",
    "##############################",
];

struct MapDef {
    name: &'static str,
    rows: &'static [&'static str; MAP_H as usize],
}

const MAP_DEF_VILLAGE: MapDef = MapDef {
    name: "余杭村郊",
    rows: &MAP_VILLAGE,
};
const MAP_DEF_BAMBOO: MapDef = MapDef {
    name: "青竹山径",
    rows: &MAP_BAMBOO,
};
const MAP_DEF_CAVE: MapDef = MapDef {
    name: "水月洞天",
    rows: &MAP_CAVE,
};
const MAP_DEF_MOON_ECHO_CORRIDOR: MapDef = MapDef {
    name: "水月回廊",
    rows: &MAP_MOON_ECHO_CORRIDOR,
};
const MAP_DEF_RIVER_TOWN: MapDef = MapDef {
    name: "江岸小镇",
    rows: &MAP_RIVER_TOWN,
};
const MAP_DEF_RIVER_REED_BED: MapDef = MapDef {
    name: "江岸芦滩",
    rows: &MAP_RIVER_REED_BED,
};
const MAP_DEF_PLAGUE_VILLAGE: MapDef = MapDef {
    name: "瘴雨村",
    rows: &MAP_PLAGUE_VILLAGE,
};
const MAP_DEF_PLAGUE_SHRINE_PATH: MapDef = MapDef {
    name: "瘴雨祠道",
    rows: &MAP_PLAGUE_SHRINE_PATH,
};
const MAP_DEF_CAPITAL: MapDef = MapDef {
    name: "云都府城",
    rows: &MAP_CAPITAL,
};
const MAP_DEF_CAPITAL_MANSION: MapDef = MapDef {
    name: "照影府邸",
    rows: &MAP_CAPITAL_MANSION,
};
const MAP_DEF_MANSION_MIRROR_GALLERY: MapDef = MapDef {
    name: "照影镜廊",
    rows: &MAP_MANSION_MIRROR_GALLERY,
};
const MAP_DEF_SOUTHERN_ROAD: MapDef = MapDef {
    name: "南疆灵道",
    rows: &MAP_SOUTHERN_ROAD,
};
const MAP_DEF_THUNDER_DRUM_PATH: MapDef = MapDef {
    name: "雷鼓祭道",
    rows: &MAP_THUNDER_DRUM_PATH,
};
const MAP_DEF_FINAL_SANCTUM: MapDef = MapDef {
    name: "灵渊终门",
    rows: &MAP_FINAL_SANCTUM,
};
const MAP_DEF_DREAM_WATERWAY: MapDef = MapDef {
    name: "旧梦水廊",
    rows: &MAP_DREAM_WATERWAY,
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MapKind {
    Village,
    Bamboo,
    Cave,
    MoonEchoCorridor,
    RiverTown,
    RiverReedBed,
    PlagueVillage,
    PlagueShrinePath,
    Capital,
    CapitalMansion,
    MansionMirrorGallery,
    SouthernRoad,
    ThunderDrumPath,
    FinalSanctum,
    DreamWaterway,
}

impl Default for MapKind {
    fn default() -> Self {
        Self::Village
    }
}

impl MapKind {
    fn def(self) -> &'static MapDef {
        match self {
            Self::Village => &MAP_DEF_VILLAGE,
            Self::Bamboo => &MAP_DEF_BAMBOO,
            Self::Cave => &MAP_DEF_CAVE,
            Self::MoonEchoCorridor => &MAP_DEF_MOON_ECHO_CORRIDOR,
            Self::RiverTown => &MAP_DEF_RIVER_TOWN,
            Self::RiverReedBed => &MAP_DEF_RIVER_REED_BED,
            Self::PlagueVillage => &MAP_DEF_PLAGUE_VILLAGE,
            Self::PlagueShrinePath => &MAP_DEF_PLAGUE_SHRINE_PATH,
            Self::Capital => &MAP_DEF_CAPITAL,
            Self::CapitalMansion => &MAP_DEF_CAPITAL_MANSION,
            Self::MansionMirrorGallery => &MAP_DEF_MANSION_MIRROR_GALLERY,
            Self::SouthernRoad => &MAP_DEF_SOUTHERN_ROAD,
            Self::ThunderDrumPath => &MAP_DEF_THUNDER_DRUM_PATH,
            Self::FinalSanctum => &MAP_DEF_FINAL_SANCTUM,
            Self::DreamWaterway => &MAP_DEF_DREAM_WATERWAY,
        }
    }

    fn portal_target(self, stage: QuestStage) -> Self {
        match self {
            Self::Village => Self::Bamboo,
            Self::Bamboo => Self::Cave,
            Self::Cave => {
                if matches!(stage, QuestStage::CaveTrial { .. }) {
                    return Self::MoonEchoCorridor;
                }

                if matches!(
                    stage,
                    QuestStage::OpeningComplete
                        | QuestStage::GatherRiverHerbs { .. }
                        | QuestStage::ReturnToHerbHealer
                        | QuestStage::FindRiverBoatman
                        | QuestStage::TuneRiverLanterns
                        | QuestStage::ConfrontRiverDemon
                        | QuestStage::RiverTownComplete
                        | QuestStage::SeekPlagueElder
                        | QuestStage::SeekShrineKeeper
                        | QuestStage::CleansePlagueShrines { .. }
                        | QuestStage::SealPlagueWards
                        | QuestStage::ReturnToShrineKeeper
                        | QuestStage::ConfrontMiasmaRoot
                        | QuestStage::PlagueVillageComplete
                        | QuestStage::SeekCapitalEnvoy
                        | QuestStage::FindMansionSpy
                        | QuestStage::GatherSecretLetters { .. }
                        | QuestStage::AlignMansionMirrors
                        | QuestStage::ReturnToMansionSpy
                        | QuestStage::ConfrontMirrorMinister
                        | QuestStage::CapitalIntrigueComplete
                        | QuestStage::SeekSpiritGuide
                        | QuestStage::SeekTribalChief
                        | QuestStage::CleanseSpiritTotems { .. }
                        | QuestStage::AlignThunderDrums
                        | QuestStage::ReturnToTribalChief
                        | QuestStage::ConfrontThunderQilin
                        | QuestStage::SouthernRoadComplete
                        | QuestStage::SeekFinalOracle
                        | QuestStage::LightFinalSoulLamps { .. }
                        | QuestStage::ReturnToFinalOracle
                        | QuestStage::ConfrontDreamEclipse
                        | QuestStage::FinaleComplete
                ) {
                    Self::RiverTown
                } else {
                    Self::Village
                }
            }
            Self::MoonEchoCorridor => Self::Cave,
            Self::RiverTown => {
                if matches!(
                    stage,
                    QuestStage::RiverTownComplete
                        | QuestStage::TuneRiverLanterns
                        | QuestStage::SeekPlagueElder
                        | QuestStage::SeekShrineKeeper
                        | QuestStage::CleansePlagueShrines { .. }
                        | QuestStage::SealPlagueWards
                        | QuestStage::ReturnToShrineKeeper
                        | QuestStage::ConfrontMiasmaRoot
                        | QuestStage::PlagueVillageComplete
                        | QuestStage::SeekCapitalEnvoy
                        | QuestStage::FindMansionSpy
                        | QuestStage::GatherSecretLetters { .. }
                        | QuestStage::AlignMansionMirrors
                        | QuestStage::ReturnToMansionSpy
                        | QuestStage::ConfrontMirrorMinister
                        | QuestStage::CapitalIntrigueComplete
                        | QuestStage::SeekSpiritGuide
                        | QuestStage::SeekTribalChief
                        | QuestStage::CleanseSpiritTotems { .. }
                        | QuestStage::AlignThunderDrums
                        | QuestStage::ReturnToTribalChief
                        | QuestStage::ConfrontThunderQilin
                        | QuestStage::SouthernRoadComplete
                        | QuestStage::SeekFinalOracle
                        | QuestStage::LightFinalSoulLamps { .. }
                        | QuestStage::ReturnToFinalOracle
                        | QuestStage::ConfrontDreamEclipse
                        | QuestStage::FinaleComplete
                ) {
                    Self::RiverReedBed
                } else {
                    Self::Village
                }
            }
            Self::RiverReedBed => {
                if matches!(
                    stage,
                    QuestStage::RiverTownComplete
                        | QuestStage::SeekPlagueElder
                        | QuestStage::SeekShrineKeeper
                        | QuestStage::CleansePlagueShrines { .. }
                        | QuestStage::SealPlagueWards
                        | QuestStage::ReturnToShrineKeeper
                        | QuestStage::ConfrontMiasmaRoot
                        | QuestStage::PlagueVillageComplete
                        | QuestStage::SeekCapitalEnvoy
                        | QuestStage::FindMansionSpy
                        | QuestStage::GatherSecretLetters { .. }
                        | QuestStage::AlignMansionMirrors
                        | QuestStage::ReturnToMansionSpy
                        | QuestStage::ConfrontMirrorMinister
                        | QuestStage::CapitalIntrigueComplete
                        | QuestStage::SeekSpiritGuide
                        | QuestStage::SeekTribalChief
                        | QuestStage::CleanseSpiritTotems { .. }
                        | QuestStage::AlignThunderDrums
                        | QuestStage::ReturnToTribalChief
                        | QuestStage::ConfrontThunderQilin
                        | QuestStage::SouthernRoadComplete
                        | QuestStage::SeekFinalOracle
                        | QuestStage::LightFinalSoulLamps { .. }
                        | QuestStage::ReturnToFinalOracle
                        | QuestStage::ConfrontDreamEclipse
                        | QuestStage::FinaleComplete
                ) {
                    Self::PlagueVillage
                } else {
                    Self::RiverTown
                }
            }
            Self::PlagueVillage => {
                if matches!(
                    stage,
                    QuestStage::CleansePlagueShrines { .. }
                        | QuestStage::SealPlagueWards
                        | QuestStage::ReturnToShrineKeeper
                        | QuestStage::ConfrontMiasmaRoot
                ) {
                    Self::PlagueShrinePath
                } else if matches!(
                    stage,
                    QuestStage::PlagueVillageComplete
                        | QuestStage::SeekCapitalEnvoy
                        | QuestStage::FindMansionSpy
                        | QuestStage::GatherSecretLetters { .. }
                        | QuestStage::AlignMansionMirrors
                        | QuestStage::ReturnToMansionSpy
                        | QuestStage::ConfrontMirrorMinister
                        | QuestStage::CapitalIntrigueComplete
                        | QuestStage::SeekSpiritGuide
                        | QuestStage::SeekTribalChief
                        | QuestStage::CleanseSpiritTotems { .. }
                        | QuestStage::AlignThunderDrums
                        | QuestStage::ReturnToTribalChief
                        | QuestStage::ConfrontThunderQilin
                        | QuestStage::SouthernRoadComplete
                        | QuestStage::SeekFinalOracle
                        | QuestStage::LightFinalSoulLamps { .. }
                        | QuestStage::ReturnToFinalOracle
                        | QuestStage::ConfrontDreamEclipse
                        | QuestStage::FinaleComplete
                ) {
                    Self::Capital
                } else {
                    Self::RiverTown
                }
            }
            Self::PlagueShrinePath => Self::PlagueVillage,
            Self::Capital => {
                if matches!(
                    stage,
                    QuestStage::CapitalIntrigueComplete
                        | QuestStage::SeekSpiritGuide
                        | QuestStage::SeekTribalChief
                        | QuestStage::CleanseSpiritTotems { .. }
                        | QuestStage::AlignThunderDrums
                        | QuestStage::ReturnToTribalChief
                        | QuestStage::ConfrontThunderQilin
                        | QuestStage::SouthernRoadComplete
                        | QuestStage::SeekFinalOracle
                        | QuestStage::LightFinalSoulLamps { .. }
                        | QuestStage::ReturnToFinalOracle
                        | QuestStage::ConfrontDreamEclipse
                        | QuestStage::FinaleComplete
                ) {
                    Self::SouthernRoad
                } else if matches!(
                    stage,
                    QuestStage::PlagueVillageComplete
                        | QuestStage::SeekCapitalEnvoy
                        | QuestStage::FindMansionSpy
                        | QuestStage::GatherSecretLetters { .. }
                        | QuestStage::AlignMansionMirrors
                        | QuestStage::ReturnToMansionSpy
                        | QuestStage::ConfrontMirrorMinister
                ) {
                    Self::CapitalMansion
                } else {
                    Self::PlagueVillage
                }
            }
            Self::CapitalMansion => {
                if matches!(
                    stage,
                    QuestStage::GatherSecretLetters { .. }
                        | QuestStage::AlignMansionMirrors
                        | QuestStage::ReturnToMansionSpy
                ) {
                    Self::MansionMirrorGallery
                } else {
                    Self::Capital
                }
            }
            Self::MansionMirrorGallery => Self::CapitalMansion,
            Self::SouthernRoad => {
                if matches!(
                    stage,
                    QuestStage::SouthernRoadComplete
                        | QuestStage::SeekFinalOracle
                        | QuestStage::LightFinalSoulLamps { .. }
                        | QuestStage::ReturnToFinalOracle
                        | QuestStage::ConfrontDreamEclipse
                        | QuestStage::FinaleComplete
                ) {
                    Self::FinalSanctum
                } else if matches!(
                    stage,
                    QuestStage::CleanseSpiritTotems { .. }
                        | QuestStage::AlignThunderDrums
                        | QuestStage::ReturnToTribalChief
                        | QuestStage::ConfrontThunderQilin
                ) {
                    Self::ThunderDrumPath
                } else {
                    Self::Capital
                }
            }
            Self::ThunderDrumPath => Self::SouthernRoad,
            Self::FinalSanctum => {
                if matches!(stage, QuestStage::LightFinalSoulLamps { .. }) {
                    Self::DreamWaterway
                } else {
                    Self::SouthernRoad
                }
            }
            Self::DreamWaterway => Self::FinalSanctum,
        }
    }
}

#[derive(Resource)]
pub struct CurrentMap(pub MapKind);

impl Default for CurrentMap {
    fn default() -> Self {
        Self(MapKind::default())
    }
}

#[derive(Resource)]
pub struct ExploreAssets {
    grass: Handle<Image>,
    forest: Handle<Image>,
    water: Handle<Image>,
    bamboo_path: Handle<Image>,
    bamboo_thicket: Handle<Image>,
    cave_floor: Handle<Image>,
    moon_cave_wall: Handle<Image>,
    mystic_grass: Handle<Image>,
    shrine_floor: Handle<Image>,
    stone_road: Handle<Image>,
    village_moss_path: Handle<Image>,
    reed_wall: Handle<Image>,
    reed_floor: Handle<Image>,
    plague_wall: Handle<Image>,
    plague_floor: Handle<Image>,
    capital_wall: Handle<Image>,
    capital_floor: Handle<Image>,
    mansion_wall: Handle<Image>,
    mansion_floor: Handle<Image>,
    mirror_wall: Handle<Image>,
    mirror_floor: Handle<Image>,
    south_wall: Handle<Image>,
    south_floor: Handle<Image>,
    thunder_wall: Handle<Image>,
    thunder_floor: Handle<Image>,
    abyss_wall: Handle<Image>,
    abyss_floor: Handle<Image>,
    dream_wall: Handle<Image>,
    dream_floor: Handle<Image>,
}

impl ExploreAssets {
    fn load(assets: &AssetServer) -> Self {
        Self {
            grass: assets.load("tiles/grass.png"),
            forest: assets.load("tiles/forest_floor.png"),
            water: assets.load("tiles/water_edge.png"),
            bamboo_path: assets.load("tiles/ai_bamboo_path.png"),
            bamboo_thicket: assets.load("tiles/ai_bamboo_thicket.png"),
            cave_floor: assets.load("tiles/ai_cave_floor.png"),
            moon_cave_wall: assets.load("tiles/ai_moon_cave_wall.png"),
            mystic_grass: assets.load("tiles/ai_mystic_grass.png"),
            shrine_floor: assets.load("tiles/ai_shrine_floor.png"),
            stone_road: assets.load("tiles/stone_road.png"),
            village_moss_path: assets.load("tiles/ai_village_moss_path.png"),
            reed_wall: assets.load("tiles/ai_reed_wall.png"),
            reed_floor: assets.load("tiles/ai_reed_floor.png"),
            plague_wall: assets.load("tiles/ai_plague_wall.png"),
            plague_floor: assets.load("tiles/ai_plague_floor.png"),
            capital_wall: assets.load("tiles/ai_capital_wall.png"),
            capital_floor: assets.load("tiles/ai_capital_floor.png"),
            mansion_wall: assets.load("tiles/ai_mansion_wall.png"),
            mansion_floor: assets.load("tiles/ai_mansion_floor.png"),
            mirror_wall: assets.load("tiles/ai_mirror_wall.png"),
            mirror_floor: assets.load("tiles/ai_mirror_floor.png"),
            south_wall: assets.load("tiles/ai_south_wall.png"),
            south_floor: assets.load("tiles/ai_south_floor.png"),
            thunder_wall: assets.load("tiles/ai_thunder_wall.png"),
            thunder_floor: assets.load("tiles/ai_thunder_floor.png"),
            abyss_wall: assets.load("tiles/ai_abyss_wall.png"),
            abyss_floor: assets.load("tiles/ai_abyss_floor.png"),
            dream_wall: assets.load("tiles/ai_dream_wall.png"),
            dream_floor: assets.load("tiles/ai_dream_floor.png"),
        }
    }
}

#[derive(Clone, Copy, Debug, ShaderType)]
struct TerrainMaterialParams {
    floor_tint: Vec4,
    grass_tint: Vec4,
    wall_tint: Vec4,
    water_tint: Vec4,
    /// x/y = map dimensions, z = tile size, w = edge blend half-width.
    map: Vec4,
    /// x = source size, y = crop inset, z = ping-pong span, w = source step.
    sample: Vec4,
    /// x/y = map-specific source phase, z = continuous UV warp amplitude.
    phase: Vec4,
}

#[derive(Asset, TypePath, AsBindGroup, Debug, Clone)]
pub(crate) struct TerrainMaterial {
    #[texture(0)]
    tile_ids: Handle<Image>,
    #[texture(1)]
    #[sampler(2)]
    floor: Handle<Image>,
    #[texture(3)]
    #[sampler(4)]
    grass: Handle<Image>,
    #[texture(5)]
    #[sampler(6)]
    wall: Handle<Image>,
    #[texture(7)]
    #[sampler(8)]
    water: Handle<Image>,
    #[uniform(9)]
    params: TerrainMaterialParams,
}

impl Material2d for TerrainMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/terrain_material.wgsl".into()
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Tile {
    Path,
    Grass,
    Wall,
    Water,
    Npc,
    Portal,
}

impl Tile {
    pub(crate) fn walkable(self) -> bool {
        matches!(self, Tile::Path | Tile::Grass | Tile::Portal)
    }
}

#[derive(Resource)]
pub struct MapData {
    kind: MapKind,
    name: &'static str,
    pub tiles: Vec<Vec<Tile>>,
}

impl MapData {
    pub(crate) fn build(kind: MapKind) -> Self {
        let def = kind.def();
        let mut tiles = Vec::with_capacity(MAP_H as usize);
        for row in def.rows.iter() {
            assert_eq!(
                row.chars().count(),
                MAP_W as usize,
                "map row width mismatch"
            );
            let mut line = Vec::with_capacity(MAP_W as usize);
            for c in row.chars() {
                line.push(match c {
                    '#' => Tile::Wall,
                    '~' => Tile::Water,
                    ',' => Tile::Grass,
                    'N' => Tile::Npc,
                    '>' => Tile::Portal,
                    _ => Tile::Path,
                });
            }
            tiles.push(line);
        }
        Self {
            kind,
            name: def.name,
            tiles,
        }
    }

    /// Build from procedurally generated tiles (roguelike stages).
    pub(crate) fn generated(kind: MapKind, tiles: Vec<Vec<Tile>>) -> Self {
        Self {
            kind,
            name: kind.def().name,
            tiles,
        }
    }

    pub(crate) fn name(&self) -> &'static str {
        self.name
    }

    pub(crate) fn at(&self, col: i32, row: i32) -> Tile {
        if col < 0 || row < 0 || col >= MAP_W || row >= MAP_H {
            return Tile::Wall;
        }
        self.tiles[row as usize][col as usize]
    }

    pub(crate) fn spawn(&self) -> (i32, i32) {
        let rows = self.kind.def().rows;
        for (r, row) in rows.iter().enumerate() {
            if let Some(c) = row.chars().position(|ch| ch == 'P') {
                return (c as i32, r as i32);
            }
        }
        (1, 1)
    }
}

// ---------------------------------------------------------------------------
// NPCs
// ---------------------------------------------------------------------------

struct NpcDef {
    col: i32,
    row: i32,
    visual: NpcVisual,
    quest: Option<QuestRole>,
    lines: &'static [&'static str],
}

#[derive(Clone, Copy)]
enum NpcVisual {
    Paperdoll(PaperdollStyle),
    Image {
        path: &'static str,
        size: f32,
        light: [f32; 4],
    },
}

fn dialogue_portrait_path(visual: NpcVisual) -> Option<&'static str> {
    match visual {
        NpcVisual::Image { path, .. } => Some(path),
        NpcVisual::Paperdoll(_) => None,
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum NpcService {
    HomeRest,
    Inn,
    CampRest,
    Shop,
}

#[derive(Debug, PartialEq, Eq)]
struct NpcServiceResult {
    line: String,
    rested: bool,
}

#[derive(Clone, Copy)]
struct GearOffer {
    gear: ShopGear,
    price: u32,
    atk: i32,
    def: i32,
    max_hp: i32,
    max_mp: i32,
    seller: &'static str,
    flavor: &'static str,
}

const NPCS_VILLAGE: [NpcDef; 4] = [
    NpcDef {
        col: 6,
        row: 7,
        visual: NpcVisual::Paperdoll(PaperdollStyle::Elder),
        quest: None,
        lines: &[
            "李逍遥：婆婆，我回来啦！",
            "婆婆：逍遥啊，近来山中妖兽横行，",
            "婆婆：那片青草丛里最是凶险，切莫大意。",
            "婆婆：去历练一番吧，万事小心。",
        ],
    },
    NpcDef {
        col: 20,
        row: 3,
        visual: NpcVisual::Paperdoll(PaperdollStyle::Linger),
        quest: Some(QuestRole::Linger),
        lines: &[
            "赵灵儿：这位少侠，可否借一步说话？",
            "赵灵儿：我在寻一样要紧的东西……",
            "赵灵儿：若遇强敌，记得施展『御剑术』。",
            "赵灵儿：江湖路远，后会有期。",
        ],
    },
    NpcDef {
        col: 22,
        row: 1,
        visual: NpcVisual::Image {
            path: "npcs/ai_sword_sister.png",
            size: 64.0,
            light: [1.0, 0.64, 0.52, 0.30],
        },
        quest: Some(QuestRole::SwordSister),
        lines: &[
            "红衣剑姊：山路妖气忽起，村里行商被困在外。",
            "红衣剑姊：少侠若肯出手，我会记下这份人情。",
        ],
    },
    NpcDef {
        col: 16,
        row: 8,
        visual: NpcVisual::Image {
            path: "npcs/ai_wandering_merchant.png",
            size: 62.0,
            light: [1.0, 0.78, 0.42, 0.28],
        },
        quest: Some(QuestRole::Merchant),
        lines: &[
            "行脚商：山路若通，我这担药材就能送到镇上。",
            "行脚商：你接了巡山委托？那可得多备些药。",
        ],
    },
];

const NPCS_BAMBOO: [NpcDef; 4] = [
    NpcDef {
        col: 5,
        row: 6,
        visual: NpcVisual::Image {
            path: "npcs/ai_mountain_monk.png",
            size: 66.0,
            light: [0.92, 0.78, 0.42, 0.24],
        },
        quest: None,
        lines: &[
            "竹林猎户：这条山路通往水月洞天。",
            "竹林猎户：雾重时别恋战，先找亮处。",
        ],
    },
    NpcDef {
        col: 28,
        row: 3,
        visual: NpcVisual::Paperdoll(PaperdollStyle::Guard),
        quest: None,
        lines: &[
            "守山力士：前面妖气很重。",
            "守山力士：你若要过去，记得补满气血。",
        ],
    },
    NpcDef {
        col: 23,
        row: 12,
        visual: NpcVisual::Image {
            path: "npcs/star_mage_cutout.png",
            size: 66.0,
            light: [0.62, 0.76, 1.0, 0.34],
        },
        quest: Some(QuestRole::StarMage),
        lines: &[
            "星咒童子：星盘乱了，妖气正绕着山径游走。",
            "星咒童子：要破局，得先把草丛里的妖兽逼出来。",
        ],
    },
    NpcDef {
        col: 18,
        row: 7,
        visual: NpcVisual::Image {
            path: "npcs/ai_bamboo_scout.png",
            size: 64.0,
            light: [0.56, 0.92, 0.48, 0.26],
        },
        quest: Some(QuestRole::BambooScout),
        lines: &[
            "竹林斥候：我听见北面的雷羽妖在啸。",
            "竹林斥候：若你要穿过山径，别只盯着脚下。",
        ],
    },
];

const NPCS_CAVE: [NpcDef; 4] = [
    NpcDef {
        col: 9,
        row: 8,
        visual: NpcVisual::Image {
            path: "npcs/ai_fox_spirit.png",
            size: 68.0,
            light: [0.64, 0.68, 1.0, 0.30],
        },
        quest: None,
        lines: &[
            "洞天术士：此地水脉会放大灵力。",
            "洞天术士：仙术命中时，敌身上应该有清楚的打击特效。",
        ],
    },
    NpcDef {
        col: 22,
        row: 4,
        visual: NpcVisual::Paperdoll(PaperdollStyle::Guard),
        quest: None,
        lines: &[
            "洞口守卫：穿过右上角光门可回到村郊。",
            "洞口守卫：别再让世界只有一张地图了。",
        ],
    },
    NpcDef {
        col: 24,
        row: 12,
        visual: NpcVisual::Paperdoll(PaperdollStyle::Ranger),
        quest: None,
        lines: &[
            "采药人：洞中草丛也会遇敌。",
            "采药人：走位、视野、光照要一起看。",
        ],
    },
    NpcDef {
        col: 14,
        row: 14,
        visual: NpcVisual::Image {
            path: "npcs/ai_cave_priestess.png",
            size: 66.0,
            light: [0.62, 0.78, 1.0, 0.30],
        },
        quest: Some(QuestRole::CavePriestess),
        lines: &[
            "月洞祭司：水月洞天会记住每一个踏入者。",
            "月洞祭司：若迷雾合拢，就沿着晶簇的光走。",
        ],
    },
];

const NPCS_MOON_ECHO_CORRIDOR: [NpcDef; 5] = [
    NpcDef {
        col: 3,
        row: 3,
        visual: NpcVisual::Image {
            path: "npcs/ai_fox_spirit.png",
            size: 62.0,
            light: [0.56, 0.68, 1.0, 0.28],
        },
        quest: None,
        lines: &[
            "回声狐影：此处是水月回廊，声音会替心事绕路。",
            "回声狐影：两座晶阵在回廊深处，不在祭司身边。",
        ],
    },
    NpcDef {
        col: 14,
        row: 5,
        visual: NpcVisual::Paperdoll(PaperdollStyle::Elder),
        quest: None,
        lines: &[
            "守水老人：草丛里的妖气会追着月光走。",
            "守水老人：净完三股妖气，再去触动上弦、下弦两晶。",
        ],
    },
    NpcDef {
        col: 27,
        row: 7,
        visual: NpcVisual::Image {
            path: "npcs/ai_mountain_monk.png",
            size: 64.0,
            light: [0.72, 0.86, 1.0, 0.24],
        },
        quest: None,
        lines: &[
            "回廊守者：右上角光门可回水月洞天。",
            "回廊守者：若晶阵亮起，祭司会知道月桥已成。",
        ],
    },
    NpcDef {
        col: 5,
        row: 10,
        visual: NpcVisual::Paperdoll(PaperdollStyle::Guard),
        quest: None,
        lines: &[
            "执灯守卫：水声乱时可以在这里休整。",
            "执灯守卫：别急着冲 boss，先把灯、晶、药都确认好。",
        ],
    },
    NpcDef {
        col: 12,
        row: 13,
        visual: NpcVisual::Paperdoll(PaperdollStyle::Ranger),
        quest: None,
        lines: &[
            "采月人：回廊比洞天窄，雾里更容易被妖气围住。",
            "采月人：看见晶簇的光，就说明路还没走错。",
        ],
    },
];

const NPCS_RIVER_TOWN: [NpcDef; 5] = [
    NpcDef {
        col: 3,
        row: 3,
        visual: NpcVisual::Image {
            path: "npcs/ai_herb_healer.png",
            size: 66.0,
            light: [0.58, 1.0, 0.68, 0.30],
        },
        quest: Some(QuestRole::HerbHealer),
        lines: &[
            "草药医：江雾有毒，今日药庐不能关门。",
            "草药医：若你从水月洞天来，身上应有月魄印的水光。",
        ],
    },
    NpcDef {
        col: 27,
        row: 7,
        visual: NpcVisual::Paperdoll(PaperdollStyle::Elder),
        quest: Some(QuestRole::RiverBoatman),
        lines: &[
            "摆渡人：江面昨夜亮起了倒流的河灯。",
            "摆渡人：想过江可以，先弄清船底那东西是什么。",
        ],
    },
    NpcDef {
        col: 25,
        row: 1,
        visual: NpcVisual::Paperdoll(PaperdollStyle::Guard),
        quest: None,
        lines: &[
            "巡河卫：光门通向回村的旧路。",
            "巡河卫：药庐与码头之间，最近常有妖影窜动。",
        ],
    },
    NpcDef {
        col: 13,
        row: 6,
        visual: NpcVisual::Image {
            path: "npcs/ai_wandering_merchant.png",
            size: 62.0,
            light: [1.0, 0.78, 0.42, 0.24],
        },
        quest: None,
        lines: &[
            "江岸货郎：镇上不是没人，只是都不敢靠近河边。",
            "江岸货郎：我这还有几瓶止血散，赶路人最常买。",
        ],
    },
    NpcDef {
        col: 5,
        row: 12,
        visual: NpcVisual::Paperdoll(PaperdollStyle::Elder),
        quest: None,
        lines: &[
            "客栈掌柜：江风湿冷，客房里备着热汤。",
            "客栈掌柜：歇一晚再走，别把妖气带进梦里。",
        ],
    },
];

const NPCS_RIVER_REED_BED: [NpcDef; 4] = [
    NpcDef {
        col: 3,
        row: 3,
        visual: NpcVisual::Paperdoll(PaperdollStyle::Ranger),
        quest: None,
        lines: &[
            "巡滩猎户：这片芦苇夜里会自己分路，别只盯着脚下。",
            "巡滩猎户：河灯若倒着走，前面多半有妖影伏着。",
        ],
    },
    NpcDef {
        col: 13,
        row: 5,
        visual: NpcVisual::Image {
            path: "npcs/ai_fox_spirit.png",
            size: 62.0,
            light: [1.0, 0.58, 0.82, 0.28],
        },
        quest: None,
        lines: &[
            "白狐少女：我不是妖影，妖影没有脚印。",
            "白狐少女：再往东就是瘴雨，记得带些清心药。",
        ],
    },
    NpcDef {
        col: 27,
        row: 7,
        visual: NpcVisual::Paperdoll(PaperdollStyle::Guard),
        quest: None,
        lines: &[
            "巡河卫：过了这片芦滩，雨味就会变苦。",
            "巡河卫：你若刚斩了河魇蛟，瘴雨村的人会愿意听你说话。",
        ],
    },
    NpcDef {
        col: 5,
        row: 10,
        visual: NpcVisual::Image {
            path: "npcs/ai_mountain_monk.png",
            size: 64.0,
            light: [0.86, 0.76, 1.0, 0.24],
        },
        quest: None,
        lines: &[
            "行脚僧：芦花能遮人，也能遮心。",
            "行脚僧：若听见水下有人唤名，不要回头。",
        ],
    },
];

const NPCS_PLAGUE_VILLAGE: [NpcDef; 5] = [
    NpcDef {
        col: 3,
        row: 3,
        visual: NpcVisual::Image {
            path: "npcs/ai_plague_elder.png",
            size: 68.0,
            light: [0.62, 0.86, 0.48, 0.28],
        },
        quest: Some(QuestRole::PlagueElder),
        lines: &[
            "村长：瘴雨落了七日，井水和药汤都变苦了。",
            "村长：外乡人若肯查病源，村里会记这份恩。",
        ],
    },
    NpcDef {
        col: 13,
        row: 6,
        visual: NpcVisual::Image {
            path: "npcs/ai_shrine_keeper.png",
            size: 68.0,
            light: [0.76, 0.92, 0.74, 0.30],
        },
        quest: Some(QuestRole::ShrineKeeper),
        lines: &[
            "祠祝：旧祠的铃多年未响，今夜却自己震了三声。",
            "祠祝：瘴源若在地下，便会借草木冒出来。",
        ],
    },
    NpcDef {
        col: 25,
        row: 1,
        visual: NpcVisual::Paperdoll(PaperdollStyle::Guard),
        quest: None,
        lines: &[
            "守村人：光门能回江岸小镇。",
            "守村人：别在黑草地停太久，雨水会咬进骨头。",
        ],
    },
    NpcDef {
        col: 27,
        row: 7,
        visual: NpcVisual::Image {
            path: "npcs/ai_herb_healer.png",
            size: 62.0,
            light: [0.54, 1.0, 0.62, 0.24],
        },
        quest: None,
        lines: &[
            "病童母亲：孩子一到夜里就说听见树根在说话。",
            "病童母亲：若你找到解瘴法，请先救村里的孩子。",
        ],
    },
    NpcDef {
        col: 5,
        row: 12,
        visual: NpcVisual::Image {
            path: "npcs/ai_mountain_monk.png",
            size: 64.0,
            light: [0.82, 0.76, 0.42, 0.22],
        },
        quest: None,
        lines: &[
            "游方僧：瘴雨不是天灾，是地下有怨根。",
            "游方僧：铃声清了，路自然会开。",
        ],
    },
];

const NPCS_PLAGUE_SHRINE_PATH: [NpcDef; 5] = [
    NpcDef {
        col: 27,
        row: 1,
        visual: NpcVisual::Paperdoll(PaperdollStyle::Guard),
        quest: None,
        lines: &[
            "守祠人：光门能回瘴雨村，祠祝在村口等铃声。",
            "守祠人：黑草地里会窜出瘴源，别空着灵力硬闯。",
        ],
    },
    NpcDef {
        col: 9,
        row: 3,
        visual: NpcVisual::Image {
            path: "npcs/ai_shrine_keeper.png",
            size: 64.0,
            light: [0.70, 0.96, 0.62, 0.28],
        },
        quest: None,
        lines: &[
            "守铃童：旧祠、苦井、病屋三处铃位在这条祠道上。",
            "守铃童：瘴源散尽之后，再逐一按铃封住残气。",
        ],
    },
    NpcDef {
        col: 13,
        row: 5,
        visual: NpcVisual::Image {
            path: "npcs/ai_herb_healer.png",
            size: 62.0,
            light: [0.54, 1.0, 0.62, 0.24],
        },
        quest: None,
        lines: &[
            "采药妇：这条路原本通病屋，雨变苦后没人敢走。",
            "采药妇：若铃位亮起，就说明村里的药汤还能救。",
        ],
    },
    NpcDef {
        col: 9,
        row: 9,
        visual: NpcVisual::Image {
            path: "npcs/ai_fox_spirit.png",
            size: 62.0,
            light: [0.64, 0.96, 0.82, 0.24],
        },
        quest: None,
        lines: &[
            "雾中狐影：井边那道绿光不是水，是瘴母根在喘气。",
            "雾中狐影：先打散黑草中的瘴源，铃才听得见你。",
        ],
    },
    NpcDef {
        col: 5,
        row: 10,
        visual: NpcVisual::Image {
            path: "npcs/ai_mountain_monk.png",
            size: 64.0,
            light: [0.82, 0.76, 0.42, 0.22],
        },
        quest: None,
        lines: &[
            "行脚僧：祠灰、井水、病灯，三处各压一口气。",
            "行脚僧：回村前歇一歇，下一战多半在祠下。",
        ],
    },
];

const NPCS_CAPITAL: [NpcDef; 5] = [
    NpcDef {
        col: 3,
        row: 3,
        visual: NpcVisual::Image {
            path: "npcs/ai_capital_envoy.png",
            size: 68.0,
            light: [0.74, 0.84, 1.0, 0.28],
        },
        quest: Some(QuestRole::CapitalEnvoy),
        lines: &[
            "宣令使：云都城门开着，府邸的门却紧闭。",
            "宣令使：若你带着解瘴符来，便该知道奏报为何被压。",
        ],
    },
    NpcDef {
        col: 13,
        row: 6,
        visual: NpcVisual::Paperdoll(PaperdollStyle::Mystic),
        quest: None,
        lines: &[
            "观星吏：府邸铜镜近日全被搬进偏院。",
            "观星吏：宣令使若给你入城符，东门灯影会让路。",
        ],
    },
    NpcDef {
        col: 25,
        row: 1,
        visual: NpcVisual::Paperdoll(PaperdollStyle::Ranger),
        quest: None,
        lines: &[
            "禁军校尉：光门通向瘴雨村旧道。",
            "禁军校尉：府邸今晚封门，偏院却还有灯。",
        ],
    },
    NpcDef {
        col: 27,
        row: 7,
        visual: NpcVisual::Image {
            path: "npcs/ai_wandering_merchant.png",
            size: 62.0,
            light: [1.0, 0.78, 0.42, 0.22],
        },
        quest: None,
        lines: &[
            "城中货郎：京城热闹，消息比货还贵。",
            "城中货郎：照影国师近来买了许多铜镜。",
        ],
    },
    NpcDef {
        col: 5,
        row: 12,
        visual: NpcVisual::Image {
            path: "npcs/ai_sword_sister.png",
            size: 64.0,
            light: [1.0, 0.64, 0.52, 0.24],
        },
        quest: None,
        lines: &[
            "红衣密探：别让国师知道你从瘴雨村来。",
            "红衣密探：密札若齐，镜阵自然会露出裂口。",
        ],
    },
];

const NPCS_CAPITAL_MANSION: [NpcDef; 5] = [
    NpcDef {
        col: 25,
        row: 1,
        visual: NpcVisual::Paperdoll(PaperdollStyle::Guard),
        quest: None,
        lines: &[
            "府邸门卒：偏院今夜闭门查账。",
            "府邸门卒：要回云都府城，走东侧光门。",
        ],
    },
    NpcDef {
        col: 13,
        row: 6,
        visual: NpcVisual::Image {
            path: "npcs/ai_mansion_spy.png",
            size: 68.0,
            light: [0.72, 0.62, 1.0, 0.30],
        },
        quest: Some(QuestRole::MansionSpy),
        lines: &[
            "偏院内线：正堂有镜，暗廊有纸，人人都假装没看见。",
            "偏院内线：想查案，就别走正门，去镜廊阴影里找密札。",
        ],
    },
    NpcDef {
        col: 27,
        row: 7,
        visual: NpcVisual::Image {
            path: "npcs/ai_fox_spirit.png",
            size: 62.0,
            light: [0.64, 0.72, 1.0, 0.28],
        },
        quest: None,
        lines: &[
            "镜中狐影：这里每面镜子都记人影。",
            "镜中狐影：密札藏在会反光的暗处。",
        ],
    },
    NpcDef {
        col: 3,
        row: 3,
        visual: NpcVisual::Paperdoll(PaperdollStyle::Ranger),
        quest: None,
        lines: &[
            "偏院暗哨：照影国师借镜阵审人，没人敢靠近。",
            "偏院暗哨：若被镜光缠住，就以御剑术破影。",
        ],
    },
    NpcDef {
        col: 5,
        row: 12,
        visual: NpcVisual::Image {
            path: "npcs/ai_sword_sister.png",
            size: 64.0,
            light: [1.0, 0.64, 0.52, 0.24],
        },
        quest: None,
        lines: &[
            "红衣密探：密札若齐，偏院内线会引国师入局。",
            "红衣密探：这里不是城街，走错一步就会撞进镜阵。",
        ],
    },
];

const NPCS_MANSION_MIRROR_GALLERY: [NpcDef; 5] = [
    NpcDef {
        col: 25,
        row: 1,
        visual: NpcVisual::Paperdoll(PaperdollStyle::Guard),
        quest: None,
        lines: &[
            "镜廊守卫：光门回偏院，别让镜光照到背后。",
            "镜廊守卫：密札若未齐，先在暗廊草影里逼出守阵幻影。",
        ],
    },
    NpcDef {
        col: 3,
        row: 3,
        visual: NpcVisual::Image {
            path: "npcs/ai_fox_spirit.png",
            size: 62.0,
            light: [0.64, 0.72, 1.0, 0.30],
        },
        quest: None,
        lines: &[
            "镜中狐影：账镜记钱，证镜记供词。",
            "镜中狐影：两面镜不在一处，国师才放心。",
        ],
    },
    NpcDef {
        col: 13,
        row: 6,
        visual: NpcVisual::Image {
            path: "npcs/ai_mansion_spy.png",
            size: 64.0,
            light: [0.72, 0.62, 1.0, 0.24],
        },
        quest: None,
        lines: &[
            "偏院暗线：我不能在这里久留，只能替你点亮第一段暗号。",
            "偏院暗线：密札齐后，对账镜，再找证镜。",
        ],
    },
    NpcDef {
        col: 27,
        row: 7,
        visual: NpcVisual::Image {
            path: "npcs/ai_wandering_merchant.png",
            size: 60.0,
            light: [1.0, 0.78, 0.42, 0.22],
        },
        quest: None,
        lines: &[
            "夜行货郎：京里人买镜，偏院人买药。",
            "夜行货郎：若要打国师，先别省那几瓶药水。",
        ],
    },
    NpcDef {
        col: 5,
        row: 12,
        visual: NpcVisual::Image {
            path: "npcs/ai_sword_sister.png",
            size: 64.0,
            light: [1.0, 0.64, 0.52, 0.24],
        },
        quest: None,
        lines: &[
            "红衣密探：镜阵里的敌人不一定是真人。",
            "红衣密探：但被他们打中，疼是真的。",
        ],
    },
];

const NPCS_SOUTHERN_ROAD: [NpcDef; 8] = [
    NpcDef {
        col: 27,
        row: 1,
        visual: NpcVisual::Paperdoll(PaperdollStyle::Guard),
        quest: None,
        lines: &[
            "灵道守卫：身后光门通回云都府城。",
            "灵道守卫：前路雷草伏着妖气，别离引路人太远。",
        ],
    },
    NpcDef {
        col: 3,
        row: 3,
        visual: NpcVisual::Image {
            path: "npcs/ai_spirit_guide.png",
            size: 66.0,
            light: [0.68, 0.94, 0.76, 0.30],
        },
        quest: Some(QuestRole::SpiritGuide),
        lines: &[
            "引路人：云都来的少侠，照影印的冷光还粘在你袖口。",
            "引路人：南疆灵道不是官道，得先请山风认你。",
        ],
    },
    NpcDef {
        col: 13,
        row: 5,
        visual: NpcVisual::Image {
            path: "npcs/ai_tribal_chief.png",
            size: 70.0,
            light: [1.0, 0.78, 0.34, 0.30],
        },
        quest: Some(QuestRole::TribalChief),
        lines: &[
            "百越族长：雷图腾一夜鸣了三次，族人不敢靠近古路。",
            "百越族长：若你真要见雷麟，先证明你能安抚图腾。",
        ],
    },
    NpcDef {
        col: 27,
        row: 7,
        visual: NpcVisual::Image {
            path: "npcs/ai_herb_healer.png",
            size: 62.0,
            light: [0.58, 1.0, 0.68, 0.24],
        },
        quest: None,
        lines: &[
            "祭草医：雷草被惊醒时，草尖会像针一样发亮。",
            "祭草医：万剑诀若能成形，剑光应该像雨一样落下。",
        ],
    },
    NpcDef {
        col: 5,
        row: 10,
        visual: NpcVisual::Paperdoll(PaperdollStyle::Ranger),
        quest: None,
        lines: &[
            "赶山人：这条灵道的雾不是挡视线，是挡心。",
            "赶山人：雷麟不怕刀剑，只怕你不敢把事做完。",
        ],
    },
    NpcDef {
        col: 24,
        row: 11,
        visual: NpcVisual::Image {
            path: "npcs/ai_wandering_merchant.png",
            size: 62.0,
            light: [1.0, 0.78, 0.42, 0.22],
        },
        quest: None,
        lines: &[
            "灵道货郎：百越符线很贵，但今天没人敢讨价。",
            "灵道货郎：听说雷麟角能照开更深的灵门。",
        ],
    },
    NpcDef {
        col: 11,
        row: 13,
        visual: NpcVisual::Paperdoll(PaperdollStyle::Mystic),
        quest: None,
        lines: &[
            "图腾巫女：雷声若散成三股，就去草阵里找回它。",
            "图腾巫女：每净一座图腾，灵道都会亮一段。",
        ],
    },
    NpcDef {
        col: 25,
        row: 14,
        visual: NpcVisual::Image {
            path: "npcs/ai_sword_sister.png",
            size: 64.0,
            light: [1.0, 0.64, 0.52, 0.24],
        },
        quest: None,
        lines: &[
            "红衣密探：云都镜阵碎了，残毒却顺着官道跑到这里。",
            "红衣密探：别拖太久，越往南，天象越不像天象。",
        ],
    },
];

const NPCS_THUNDER_DRUM_PATH: [NpcDef; 5] = [
    NpcDef {
        col: 27,
        row: 1,
        visual: NpcVisual::Paperdoll(PaperdollStyle::Guard),
        quest: None,
        lines: &[
            "祭道守卫：光门回南疆灵道，族长在那边等玉牒。",
            "祭道守卫：雷草里藏着暴走图腾，别把鼓声当风声。",
        ],
    },
    NpcDef {
        col: 3,
        row: 3,
        visual: NpcVisual::Image {
            path: "npcs/ai_tribal_chief.png",
            size: 66.0,
            light: [1.0, 0.78, 0.34, 0.26],
        },
        quest: None,
        lines: &[
            "鼓阵巫祝：风鼓引路，云鼓收声，誓鼓定心。",
            "鼓阵巫祝：三面鼓都响正，雷麟才会承认你。",
        ],
    },
    NpcDef {
        col: 13,
        row: 5,
        visual: NpcVisual::Image {
            path: "npcs/ai_spirit_guide.png",
            size: 64.0,
            light: [0.68, 0.94, 0.76, 0.26],
        },
        quest: None,
        lines: &[
            "引路人影：雷路会把胆怯的人绕回原点。",
            "引路人影：若任务簿还写着三座图腾，就先去雷草里迎战。",
        ],
    },
    NpcDef {
        col: 27,
        row: 7,
        visual: NpcVisual::Image {
            path: "npcs/ai_herb_healer.png",
            size: 62.0,
            light: [0.58, 1.0, 0.68, 0.22],
        },
        quest: None,
        lines: &[
            "祭草医：雷伤先麻后痛，别等倒下才喝药。",
            "祭草医：雷鼓齐鸣后，万剑诀会比从前更稳。",
        ],
    },
    NpcDef {
        col: 5,
        row: 10,
        visual: NpcVisual::Image {
            path: "npcs/ai_sword_sister.png",
            size: 64.0,
            light: [1.0, 0.64, 0.52, 0.24],
        },
        quest: None,
        lines: &[
            "红衣密探：这里的雷声会诱人拔剑。",
            "红衣密探：等鼓声对齐，再把剑意交给雷麟看。",
        ],
    },
];

const NPCS_FINAL_SANCTUM: [NpcDef; 6] = [
    NpcDef {
        col: 24,
        row: 1,
        visual: NpcVisual::Paperdoll(PaperdollStyle::Guard),
        quest: None,
        lines: &[
            "终门守卫：身后光门可回南疆灵道。",
            "终门守卫：若三灯未亮，灵渊深处只会把人送回原地。",
        ],
    },
    NpcDef {
        col: 3,
        row: 3,
        visual: NpcVisual::Paperdoll(PaperdollStyle::Linger),
        quest: Some(QuestRole::Linger),
        lines: &[
            "赵灵儿：这里的水声像在念旧人的名字。",
            "赵灵儿：若害怕，就看着灯光，一步一步走。",
        ],
    },
    NpcDef {
        col: 13,
        row: 6,
        visual: NpcVisual::Image {
            path: "npcs/ai_final_oracle.png",
            size: 70.0,
            light: [0.62, 0.86, 1.0, 0.34],
        },
        quest: Some(QuestRole::FinalOracle),
        lines: &[
            "守灯人：终门只问三件事，来路、誓言、宿命。",
            "守灯人：答错的人会一直在水声里走回开头。",
        ],
    },
    NpcDef {
        col: 27,
        row: 7,
        visual: NpcVisual::Image {
            path: "npcs/ai_fox_spirit.png",
            size: 66.0,
            light: [0.64, 0.72, 1.0, 0.28],
        },
        quest: None,
        lines: &[
            "灵渊狐影：别碰错灯。",
            "灵渊狐影：忆灯照来路，誓灯照同伴，命灯照该斩断的东西。",
        ],
    },
    NpcDef {
        col: 27,
        row: 10,
        visual: NpcVisual::Image {
            path: "npcs/ai_cave_priestess.png",
            size: 64.0,
            light: [0.62, 0.78, 1.0, 0.26],
        },
        quest: None,
        lines: &[
            "残梦祭司：水月洞天的影子也流到这里了。",
            "残梦祭司：若你还记得最初的月光，就别让它熄灭。",
        ],
    },
    NpcDef {
        col: 5,
        row: 12,
        visual: NpcVisual::Image {
            path: "npcs/ai_sword_sister.png",
            size: 64.0,
            light: [1.0, 0.64, 0.52, 0.24],
        },
        quest: None,
        lines: &[
            "红衣幻影：山路的第一声剑鸣，也被这水记住了。",
            "红衣幻影：少侠，终局不是一个人打完的。",
        ],
    },
];

const NPCS_DREAM_WATERWAY: [NpcDef; 5] = [
    NpcDef {
        col: 24,
        row: 1,
        visual: NpcVisual::Paperdoll(PaperdollStyle::Guard),
        quest: None,
        lines: &[
            "水廊守卫：身后光门回灵渊终门，守灯人会在那里等你。",
            "水廊守卫：三灯不齐，旧梦会把人绕回水声里。",
        ],
    },
    NpcDef {
        col: 3,
        row: 3,
        visual: NpcVisual::Paperdoll(PaperdollStyle::Linger),
        quest: None,
        lines: &[
            "赵灵儿的梦影：忆灯照来路，不是要你沉回过去。",
            "赵灵儿的梦影：看清它，然后继续往前走。",
        ],
    },
    NpcDef {
        col: 13,
        row: 6,
        visual: NpcVisual::Image {
            path: "npcs/ai_final_oracle.png",
            size: 66.0,
            light: [0.62, 0.86, 1.0, 0.30],
        },
        quest: None,
        lines: &[
            "守灯人影：誓灯不问输赢，只问你有没有把同伴放进心里。",
            "守灯人影：灯亮之后，水影会少一层。",
        ],
    },
    NpcDef {
        col: 27,
        row: 7,
        visual: NpcVisual::Image {
            path: "npcs/ai_fox_spirit.png",
            size: 66.0,
            light: [0.64, 0.72, 1.0, 0.28],
        },
        quest: None,
        lines: &[
            "灵渊狐影：命灯最会骗人。",
            "灵渊狐影：它让人以为宿命只剩一条路。",
        ],
    },
    NpcDef {
        col: 5,
        row: 12,
        visual: NpcVisual::Image {
            path: "npcs/ai_sword_sister.png",
            size: 64.0,
            light: [1.0, 0.64, 0.52, 0.24],
        },
        quest: None,
        lines: &[
            "红衣幻影：最后一段路，别只看自己。",
            "红衣幻影：三灯都亮时，剑才知道该斩向哪里。",
        ],
    },
];

fn npc_defs(kind: MapKind) -> &'static [NpcDef] {
    match kind {
        MapKind::Village => &NPCS_VILLAGE,
        MapKind::Bamboo => &NPCS_BAMBOO,
        MapKind::Cave => &NPCS_CAVE,
        MapKind::MoonEchoCorridor => &NPCS_MOON_ECHO_CORRIDOR,
        MapKind::RiverTown => &NPCS_RIVER_TOWN,
        MapKind::RiverReedBed => &NPCS_RIVER_REED_BED,
        MapKind::PlagueVillage => &NPCS_PLAGUE_VILLAGE,
        MapKind::PlagueShrinePath => &NPCS_PLAGUE_SHRINE_PATH,
        MapKind::Capital => &NPCS_CAPITAL,
        MapKind::CapitalMansion => &NPCS_CAPITAL_MANSION,
        MapKind::MansionMirrorGallery => &NPCS_MANSION_MIRROR_GALLERY,
        MapKind::SouthernRoad => &NPCS_SOUTHERN_ROAD,
        MapKind::ThunderDrumPath => &NPCS_THUNDER_DRUM_PATH,
        MapKind::FinalSanctum => &NPCS_FINAL_SANCTUM,
        MapKind::DreamWaterway => &NPCS_DREAM_WATERWAY,
    }
}

fn npc_service_for(kind: MapKind, npc: &NpcDef) -> Option<NpcService> {
    match (kind, npc.col, npc.row) {
        (MapKind::Village, 6, 7) => Some(NpcService::HomeRest),
        (MapKind::Village, 16, 8)
        | (MapKind::RiverTown, 13, 6)
        | (MapKind::Capital, 27, 7)
        | (MapKind::SouthernRoad, 24, 11) => Some(NpcService::Shop),
        (MapKind::Cave, 9, 8)
        | (MapKind::MoonEchoCorridor, 5, 10)
        | (MapKind::PlagueVillage, 27, 7)
        | (MapKind::PlagueShrinePath, 5, 10)
        | (MapKind::Capital, 5, 12)
        | (MapKind::SouthernRoad, 27, 7)
        | (MapKind::ThunderDrumPath, 5, 10)
        | (MapKind::DreamWaterway, 5, 12)
        | (MapKind::FinalSanctum, 27, 10) => Some(NpcService::CampRest),
        (MapKind::RiverTown, 5, 12) => Some(NpcService::Inn),
        _ => None,
    }
}

fn npc_reaction_lines(kind: MapKind, npc: &NpcDef, quest: &QuestLog) -> Vec<String> {
    if npc.quest.is_some() {
        return Vec::new();
    }

    let mut lines = Vec::new();
    lines.extend(local_side_quest_reactions(kind, quest));
    if let Some(line) = local_side_quest_route_reaction(kind, quest) {
        lines.push(line);
    }
    if let Some(line) = local_npc_errand_route_reaction(kind, quest) {
        lines.push(line);
    }
    if let Some(line) = local_treasure_reaction(kind, quest) {
        lines.push(line.to_string());
    }
    if let Some(line) = local_care_reaction(kind, quest) {
        lines.push(line);
    }
    if let Some(line) = local_companion_scene_reaction(kind, quest) {
        lines.push(line);
    }
    if let Some(line) = local_companion_revisit_reaction(kind, quest) {
        lines.push(line);
    }
    if let Some(line) = local_route_branch_reaction(kind, quest) {
        lines.push(line);
    }
    lines
}

fn local_side_quest_reactions(kind: MapKind, quest: &QuestLog) -> Vec<String> {
    let mut lines = Vec::new();
    let sides = side_quests_for_map(kind);
    if sides.is_empty() {
        return lines;
    }

    if side_board_completed_count(sides, quest) == sides.len() {
        let mut line = match kind {
            MapKind::Village => {
                "【街谈】村人说任务板两张红签都撤下了，夜里终于敢出门采药。".to_string()
            }
            MapKind::Cave => "【街谈】洞天术士说晶尘和回声都已安稳，水月石牌不再发冷。".to_string(),
            MapKind::RiverTown => {
                "【街谈】镇民说河灯与湿货都找回来了，码头今晚敢重新点灯。".to_string()
            }
            MapKind::PlagueVillage => {
                "【街谈】村人说救急药和药童都平安了，病屋里终于有了笑声。".to_string()
            }
            MapKind::Capital => {
                "【街谈】府城暗线说巡查和暗帖都已交，镜阵余影少了许多。".to_string()
            }
            MapKind::SouthernRoad => "【街谈】百越族人说雷纹归位，战鼓也安静了。".to_string(),
            MapKind::FinalSanctum => {
                "【街谈】守灯人说梦灯余波和归潮灯签都已归位，终门外的归路亮了一整夜。".to_string()
            }
            MapKind::Bamboo
            | MapKind::MoonEchoCorridor
            | MapKind::RiverReedBed
            | MapKind::PlagueShrinePath
            | MapKind::CapitalMansion
            | MapKind::MansionMirrorGallery
            | MapKind::ThunderDrumPath
            | MapKind::DreamWaterway => return lines,
        };
        if let Some(resolution) = local_commission_resolution_reaction(kind, quest) {
            line.push(' ');
            line.push_str(&resolution);
        }
        if let Some(field) = local_commission_field_reaction(kind, quest) {
            line.push(' ');
            line.push_str(&field);
        }
        lines.push(line);
        return lines;
    }

    if let Some(line) = local_completed_side_quest_reaction(sides, quest) {
        lines.push(line);
    }

    if let Some(line) = local_side_quest_status_reaction(sides, quest) {
        lines.push(line);
    }

    lines
}

fn local_completed_side_quest_reaction(sides: &[SideQuest], quest: &QuestLog) -> Option<String> {
    sides
        .iter()
        .copied()
        .find(|side| quest.is_side_quest_completed(*side))
        .map(|side| {
            let mut line = format!("【街谈】{}", quest.side_task_completed_line(side));
            if let Some(resolution) = quest.side_task_resolution_reaction(side) {
                line.push(' ');
                line.push_str(&resolution);
            }
            if let Some(approach) = quest.side_quest_field_approach(side) {
                line.push(' ');
                line.push_str(&format!(
                    "【现场回声】{}已写进委托签，地方人照此调整巡路。",
                    approach.name()
                ));
            }
            line
        })
}

fn local_side_quest_status_reaction(sides: &[SideQuest], quest: &QuestLog) -> Option<String> {
    let side = current_side_quest(sides, quest)?;
    let completed = side_board_completed_count(sides, quest);
    if completed > 0 && !quest.is_side_quest_active(side) {
        return Some(format!(
            "【街谈】任务板又添了新签：{}，镇上还在等人接手。",
            quest.side_quest_name(side)
        ));
    }

    if !quest.is_side_quest_active(side) {
        return None;
    }

    let progress = quest.side_quest_progress(side);
    let goal = quest.side_quest_goal(side);
    if progress >= goal {
        Some(format!(
            "【街谈】{} 的条件已经办妥，回任务板交付就能领赏。",
            quest.side_quest_name(side)
        ))
    } else {
        Some(format!(
            "【街谈】{} 的委托已揭，眼下进度 {progress}/{goal}。",
            quest.side_quest_name(side)
        ))
    }
}

fn local_side_quest_route_reaction(kind: MapKind, quest: &QuestLog) -> Option<String> {
    if !side_quests_for_map(kind).is_empty() {
        return None;
    }

    let base = match side_quest_route_relief_state(kind, quest) {
        SideQuestRouteReliefState::NoRoute => return None,
        SideQuestRouteReliefState::Partial => match kind {
            MapKind::Bamboo => "【委托回声】村郊一张委托已归档，守山人开始把竹灯往山路深处挪。",
            MapKind::MoonEchoCorridor => {
                "【委托回声】水月洞天已有委托归档，回廊巡灯敢多照亮一段石壁。"
            }
            MapKind::RiverReedBed => {
                "【委托回声】江岸任务板压下一张水签，芦滩巡夜人敢重新靠近浅滩。"
            }
            MapKind::PlagueShrinePath => {
                "【委托回声】瘴雨村药榜少了一张急签，祠道药童敢把铃挂得更远。"
            }
            MapKind::CapitalMansion | MapKind::MansionMirrorGallery => {
                "【委托回声】府城密榜已有回执，偏院暗线敢把巡夜粉记推进一扇门。"
            }
            MapKind::ThunderDrumPath => {
                "【委托回声】百越灵道已有一段雷声安定，巡山人敢把路符压到鼓道边。"
            }
            MapKind::DreamWaterway => {
                "【委托回声】终门灯簿已有一页合上，守灯人敢把小灯送进旧梦水声里。"
            }
            MapKind::Village
            | MapKind::Cave
            | MapKind::RiverTown
            | MapKind::PlagueVillage
            | MapKind::Capital
            | MapKind::SouthernRoad
            | MapKind::FinalSanctum => return None,
        },
        SideQuestRouteReliefState::Cleared => match kind {
            MapKind::Bamboo => "【委托回声】山路余妖与药圃妖香都被压住，竹林外缘的夜路稳了许多。",
            MapKind::MoonEchoCorridor => {
                "【委托回声】晶尘和回声委托都已归档，水月回廊的妖影被巡灯逼退。"
            }
            MapKind::RiverReedBed => "【委托回声】河灯与湿货都找回来了，芦滩夜路重新有了人声。",
            MapKind::PlagueShrinePath => {
                "【委托回声】救急药和药童都平安，祠道瘴影不再敢贴着病屋绕路。"
            }
            MapKind::CapitalMansion | MapKind::MansionMirrorGallery => {
                "【委托回声】巡查与暗帖都交清，镜廊里的余影少了藏身的街谈。"
            }
            MapKind::ThunderDrumPath => {
                "【委托回声】雷纹和旧鼓都安静了，百越巡山人说灵道路脉重新接上。"
            }
            MapKind::DreamWaterway => {
                "【委托回声】梦灯余波与归潮旧愿都归簿，旧梦水廊的回卷退了一层。"
            }
            MapKind::Village
            | MapKind::Cave
            | MapKind::RiverTown
            | MapKind::PlagueVillage
            | MapKind::Capital
            | MapKind::SouthernRoad
            | MapKind::FinalSanctum => return None,
        },
    };

    let mut line = base.to_string();
    if let Some(resolution) = local_commission_resolution_reaction(kind, quest) {
        line.push(' ');
        line.push_str(&resolution);
    }
    if let Some(field) = local_commission_field_reaction(kind, quest) {
        line.push(' ');
        line.push_str(&field);
    }
    Some(line)
}

fn local_npc_errand_route_reaction(kind: MapKind, quest: &QuestLog) -> Option<String> {
    let count = npc_errand_route_relief_count(kind, quest);
    if count == 0 {
        return None;
    }

    Some(match (kind, count) {
        (MapKind::Cave | MapKind::MoonEchoCorridor, _) => {
            "【托付回声】洞中采药人说竹露已入月寒药钵，水月路上的寒妖少追了几步。".to_string()
        }
        (MapKind::RiverTown, _) => {
            "【托付回声】巡河卫把月苔压进渡口水符，江岸雾灯比昨夜亮了半寸。".to_string()
        }
        (MapKind::RiverReedBed, 1) => {
            "【托付回声】月苔寒气稳住一盏渡口灯，芦滩水雾不再贴着脚踝打转。".to_string()
        }
        (MapKind::RiverReedBed, _) => {
            "【托付回声】月苔和芦滩小信都送到，巡路人把新浅渡画进夜巡水图。".to_string()
        }
        (MapKind::PlagueVillage | MapKind::PlagueShrinePath, _) => {
            "【托付回声】病童护符到了祠道，采药妇已经按护符纹路重配一锅苦药。".to_string()
        }
        (MapKind::CapitalMansion | MapKind::MansionMirrorGallery, _) => {
            "【托付回声】星图密片交给偏院暗线，镜廊巡夜开始避开国师换镜的时辰。".to_string()
        }
        (MapKind::SouthernRoad, _) => {
            "【托付回声】照影药引洒进南疆草路，镜阵残毒露出灰线，赶山人敢往前探。".to_string()
        }
        (MapKind::ThunderDrumPath, 1) => {
            "【托付回声】照影药引辨出雷草里的灰毒，鼓道边缘少了一层京华残影。".to_string()
        }
        (MapKind::ThunderDrumPath, _) => {
            "【托付回声】照影药引和雷草药酒都到位，祭道药炉压住雷声，守夜人重新靠近鼓架。"
                .to_string()
        }
        (MapKind::DreamWaterway, _) => {
            "【托付回声】旧梦灯芯接入誓灯，水廊回卷退开一线，守灯人影能看见归路。".to_string()
        }
        (MapKind::Village | MapKind::Bamboo | MapKind::Capital | MapKind::FinalSanctum, _) => {
            return None;
        }
    })
}

fn local_commission_resolution_reaction(kind: MapKind, quest: &QuestLog) -> Option<String> {
    let sides = side_quest_route_relief_sides(kind);
    let pursued = sides
        .iter()
        .copied()
        .filter(|side| quest.side_quest_resolution(*side) == Some(SideQuestResolution::Pursue))
        .count();
    let completed = sides
        .iter()
        .copied()
        .filter(|side| quest.is_side_quest_completed(*side))
        .count();
    if completed == 0 {
        return None;
    }
    let settled = completed.saturating_sub(pursued);

    Some(match (settled, pursued) {
        (_, 0) => format!("【裁断回声】{settled} 张委托稳妥封存，地方账簿先求安稳。"),
        (0, _) => {
            format!("【裁断回声】{pursued} 张委托转为追查余波，巡路人会继续盯着旧线。")
        }
        _ => format!(
            "【裁断回声】{settled} 张委托封存，{pursued} 张委托追查余波，地方账簿分成安民与巡路两路。"
        ),
    })
}

fn local_commission_field_reaction(kind: MapKind, quest: &QuestLog) -> Option<String> {
    let investigated =
        side_quest_route_field_count(kind, quest, SideQuestFieldApproach::Investigate);
    let confronted = side_quest_route_field_count(kind, quest, SideQuestFieldApproach::Confront);
    if investigated == 0 && confronted == 0 {
        return None;
    }

    Some(match (investigated, confronted) {
        (_, 0) => {
            format!("【现场回声】{investigated} 处现场按细查现场归档，巡路人按线索重画夜路。")
        }
        (0, _) => {
            format!("【现场回声】{confronted} 处现场按快断余妖归档，守路人照速断法压住妖口。")
        }
        _ => format!(
            "【现场回声】细查{investigated}处、快断{confronted}处，委托签把查线和断势分开归档。"
        ),
    })
}

fn map_treasure_cache(kind: MapKind) -> Option<TreasureCache> {
    match kind {
        MapKind::Village => Some(TreasureCache::VillageShrine),
        MapKind::Bamboo => Some(TreasureCache::BambooOffering),
        MapKind::Cave => Some(TreasureCache::CaveOffering),
        MapKind::MoonEchoCorridor => None,
        MapKind::RiverTown => Some(TreasureCache::RiverTownCrystal),
        MapKind::RiverReedBed => Some(TreasureCache::RiverReedCrystal),
        MapKind::PlagueVillage => Some(TreasureCache::PlagueShrine),
        MapKind::PlagueShrinePath => Some(TreasureCache::PlagueShrine),
        MapKind::Capital => Some(TreasureCache::CapitalShrine),
        MapKind::CapitalMansion => Some(TreasureCache::MansionMirror),
        MapKind::MansionMirrorGallery => Some(TreasureCache::MansionMirror),
        MapKind::SouthernRoad => Some(TreasureCache::SouthernTotem),
        MapKind::ThunderDrumPath => None,
        MapKind::FinalSanctum => Some(TreasureCache::FinalMemoryCache),
        MapKind::DreamWaterway => None,
    }
}

fn local_treasure_reaction(kind: MapKind, quest: &QuestLog) -> Option<&'static str> {
    let cache = map_treasure_cache(kind)?;
    if !quest.has_opened_treasure(cache) {
        return None;
    }

    Some(match kind {
        MapKind::Village => "【传闻】婆婆瞧见你动过旧像，说那包护身药本就是留给赶路人的。",
        MapKind::Bamboo => "【传闻】竹林猎户说供龛灵露少了一份，山风却比刚才清了。",
        MapKind::Cave => "【传闻】洞天术士认出祭台月光，提醒你别把药钱全花在路上。",
        MapKind::MoonEchoCorridor => "【传闻】回廊狐影说晶阵亮过之后，窄廊的回声不再追着人跑。",
        MapKind::RiverTown => "【传闻】江岸货郎说灵晶吐出的谢礼，是河灯人家攒下的平安钱。",
        MapKind::RiverReedBed => "【传闻】巡滩猎户看见贝袋，断定芦滩水脉已经认得你的灵符。",
        MapKind::PlagueVillage => "【传闻】病童母亲听说祠灰下有药包，忙替你把消息传给病屋。",
        MapKind::PlagueShrinePath => "【传闻】守铃童说祠灰暗格已开，净瘴铃回声清了半分。",
        MapKind::Capital => "【传闻】观星吏说香案暗格已开，府城有人开始相信你的来意。",
        MapKind::CapitalMansion => "【传闻】偏院暗哨听见暗匣机关响，知道你已摸到镜阵背面。",
        MapKind::MansionMirrorGallery => "【传闻】镜中狐影看见暗匣药散，知道你已经摸到镜阵背面。",
        MapKind::SouthernRoad => "【传闻】祭草医闻到百越药酒味，说图腾座认可了你的脚步。",
        MapKind::ThunderDrumPath => "【传闻】鼓阵巫祝说旧图腾座已空，雷声把你的名字记下了。",
        MapKind::FinalSanctum => "【传闻】灵渊狐影说旧梦匣已经认主，前人的遗物不会再沉回深水。",
        MapKind::DreamWaterway => "【传闻】水廊守卫说旧梦水声淡了，三灯正在等最后的回音。",
    })
}

fn local_care_reaction(kind: MapKind, quest: &QuestLog) -> Option<String> {
    let (bond_scene, camp_scene) = map_care_scenes(kind);
    let bond_seen = quest.has_seen_bond_scene(bond_scene);
    let camp_seen = quest.has_seen_camp_scene(camp_scene);
    let any_seen = bond_seen || camp_seen;
    let map_chapter = map_chapter(kind);
    let chapter_passed = chapter_rank(quest.current_chapter()) > chapter_rank(map_chapter);

    if !any_seen && !chapter_passed {
        return None;
    }

    let place = kind.def().name;
    if bond_seen && camp_seen {
        return Some(format!(
            "【照应】{place}的人记得你们既在灵灯旁说过心事，也在歇脚处把行囊整齐。"
        ));
    }

    if bond_seen {
        return Some(format!(
            "【照应】{place}的灯火还留着你们夜谈的影子，只是歇脚处少了一点烟火气。"
        ));
    }

    if camp_seen {
        return Some(format!(
            "【照应】{place}的人记得你们曾停下休整，只是灵灯旁还有话没有说完。"
        ));
    }

    Some(format!(
        "【照应】{place}的旧灯还空着，错过的夜谈和休整让这段路少了一点回声。"
    ))
}

fn local_route_branch_reaction(kind: MapKind, quest: &QuestLog) -> Option<String> {
    let detour = local_route_detour_for_map(kind)?;
    let approach = quest.route_detour_approach(detour)?;
    Some(match (detour, approach) {
        (RouteDetour::MoonEchoPool, RouteDetourApproach::Scout) => {
            "【街谈】水月洞人听说你细查过照水暗池，已经把池边布条画进巡夜路图。".to_string()
        }
        (RouteDetour::MoonEchoPool, RouteDetourApproach::PressOn) => {
            "【街谈】水月洞人听说你快步穿过照水暗池，只提醒后来的弟子别学得太急。".to_string()
        }
        (RouteDetour::ReedHiddenFord, RouteDetourApproach::Scout) => {
            "【街谈】江岸人说芦下隐渡的芦痕压得很稳，夜里巡货可以少绕一段水路。".to_string()
        }
        (RouteDetour::ReedHiddenFord, RouteDetourApproach::PressOn) => {
            "【街谈】江岸人听说你从芦下隐渡快走过去，知道那条浅水线能通，但还不敢夜里走。"
                .to_string()
        }
        (RouteDetour::PlagueHerbTrail, RouteDetourApproach::Scout) => {
            "【街谈】瘴雨村人把你分出的祠旁药径记下，病屋药锅终于多了一味苦药。".to_string()
        }
        (RouteDetour::PlagueHerbTrail, RouteDetourApproach::PressOn) => {
            "【街谈】瘴雨村人知道你冲过祠旁药径，薄瘴方向有了线索，只是采药还得小心。".to_string()
        }
        (RouteDetour::MirrorServantDoor, RouteDetourApproach::Scout) => {
            "【街谈】府城暗线说镜仆暗门的粉记已抄下，偏院巡夜终于有了退路。".to_string()
        }
        (RouteDetour::MirrorServantDoor, RouteDetourApproach::PressOn) => {
            "【街谈】府城暗线听说你借镜仆暗门快走，知道斜廊可用，却还缺完整转角记号。".to_string()
        }
        (RouteDetour::ThunderRidgeCache, RouteDetourApproach::Scout) => {
            "【街谈】百越族人说雷脊旧藏已经启开，路符压住乱雷，巡山人敢再往前。".to_string()
        }
        (RouteDetour::ThunderRidgeCache, RouteDetourApproach::PressOn) => {
            "【街谈】百越族人听说你越过雷脊旧藏，记下避雷步点，却仍劝后人等雷声落完。".to_string()
        }
        (RouteDetour::DreamBackwater, RouteDetourApproach::Scout) => {
            "【街谈】灵渊守灯人说旧梦回湾的灯签顺水亮着，回程水线终于能看清。".to_string()
        }
        (RouteDetour::DreamBackwater, RouteDetourApproach::PressOn) => {
            "【街谈】灵渊守灯人听说你冲出旧梦回湾，知道回卷有空隙，却还不敢久留。".to_string()
        }
    })
}

fn local_companion_scene_reaction(kind: MapKind, quest: &QuestLog) -> Option<String> {
    match companion_route_state(kind, quest) {
        CompanionRouteState::NoRoute => None,
        CompanionRouteState::Prepared | CompanionRouteState::Partial => {
            local_completed_companion_scene_reaction(kind, quest)
        }
        CompanionRouteState::Missed => Some(local_missed_companion_scene_reaction(kind)),
    }
}

fn local_companion_revisit_reaction(kind: MapKind, quest: &QuestLog) -> Option<String> {
    let revisit = match kind {
        MapKind::PlagueVillage | MapKind::PlagueShrinePath => CompanionRevisit::TrailEcho,
        MapKind::SouthernRoad | MapKind::ThunderDrumPath => CompanionRevisit::MirrorTrace,
        MapKind::FinalSanctum | MapKind::DreamWaterway => CompanionRevisit::TotemVow,
        _ => return None,
    };
    if !quest.companion_revisit_resolved(revisit) {
        return None;
    }
    Some(
        match revisit {
            CompanionRevisit::TrailEcho => {
                "【补访回声】祠道人说月衡把竹绳结重新系紧，瘴雨里终于多了一条共同退路。"
            }
            CompanionRevisit::MirrorTrace => {
                "【补访回声】赶山人说月衡已在雷光里问清旧案，照影碎片不再映出追兵。"
            }
            CompanionRevisit::TotemVow => {
                "【补访回声】守灯人听见南瑶答完百越旧誓，归水不再反复追问她的名字。"
            }
        }
        .to_string(),
    )
}

fn local_completed_companion_scene_reaction(kind: MapKind, quest: &QuestLog) -> Option<String> {
    match kind {
        MapKind::Village | MapKind::Bamboo | MapKind::MoonEchoCorridor
            if quest.has_seen_companion_scene(CompanionScene::SwordSisterTrailGuard) =>
        {
            Some("【小传街谈】林月衡在旧栅前重系退路，巡山人说回声窄道少了截后的妖影。".to_string())
        }
        MapKind::Capital | MapKind::CapitalMansion | MapKind::MansionMirrorGallery
            if quest.has_seen_companion_scene(CompanionScene::SwordSisterCapitalMirror) =>
        {
            Some("【小传街谈】府城暗线听说月衡看破照影旧案，镜廊里的假脚步少了许多。".to_string())
        }
        MapKind::SouthernRoad | MapKind::ThunderDrumPath
            if quest.has_seen_companion_scene(CompanionScene::SpiritWitchSouthernTotem) =>
        {
            Some(
                "【小传街谈】百越族人说南瑶听懂雷纹旧愿，乱雷鼓道终于肯让行人先走半步。"
                    .to_string(),
            )
        }
        MapKind::FinalSanctum | MapKind::DreamWaterway
            if quest.has_seen_companion_scene(CompanionScene::SwordSisterFinalReturn)
                && quest.has_seen_companion_scene(CompanionScene::SpiritWitchFinalVow) =>
        {
            Some(
                "【小传街谈】守灯人记下月衡归剑和南瑶灯誓，旧梦水廊的回声不再追着归路翻涌。"
                    .to_string(),
            )
        }
        MapKind::FinalSanctum | MapKind::DreamWaterway
            if quest.has_seen_companion_scene(CompanionScene::SwordSisterFinalReturn) =>
        {
            Some("【小传街谈】守灯人说月衡把归剑压进终门，旧梦水廊少了一道截口。".to_string())
        }
        MapKind::FinalSanctum | MapKind::DreamWaterway
            if quest.has_seen_companion_scene(CompanionScene::SpiritWitchFinalVow) =>
        {
            Some("【小传街谈】守灯人说南瑶把归潮灯誓留在水声里，旧梦回卷慢了半拍。".to_string())
        }
        _ => None,
    }
}

fn local_missed_companion_scene_reaction(kind: MapKind) -> String {
    match kind {
        MapKind::Village | MapKind::Bamboo | MapKind::MoonEchoCorridor => {
            "【小传街谈】旧栅前还有未问出口的小传，回头走这段路时，妖影更爱从后队贴近。".to_string()
        }
        MapKind::Capital | MapKind::CapitalMansion | MapKind::MansionMirrorGallery => {
            "【小传街谈】照影旧案无人细问，府城暗线说镜廊仍会把迟来的脚步绕回原处。".to_string()
        }
        MapKind::SouthernRoad | MapKind::ThunderDrumPath => {
            "【小传街谈】雷纹旧愿没来得及听完，百越巡山人说乱雷比从前更会压住归路。".to_string()
        }
        MapKind::FinalSanctum | MapKind::DreamWaterway => {
            "【小传街谈】终门灯下仍缺一段归路誓言，旧梦水声会趁沉默处卷回。".to_string()
        }
        MapKind::Cave
        | MapKind::RiverTown
        | MapKind::RiverReedBed
        | MapKind::PlagueVillage
        | MapKind::PlagueShrinePath => {
            "【小传街谈】同行人的旧话没有接上，这段路的回声比从前更冷。".to_string()
        }
    }
}

fn local_companion_aftermath_line(scene: CompanionScene) -> &'static str {
    match scene {
        CompanionScene::SwordSisterTrailGuard => {
            "【小传回访】竹林巡山人照着月衡重系的旧栅换上耐雨麻绳，又把省下的巡路钱塞给你。"
        }
        CompanionScene::SwordSisterCapitalMirror => {
            "【小传回访】府城暗线循着月衡留下的镜痕清掉两处假脚印，将封口钱和护身药交来。"
        }
        CompanionScene::SpiritWitchSouthernTotem => {
            "【小传回访】百越巡山人把南瑶听懂的雷纹拓成路符，连同药酒和谢仪系在符后。"
        }
        CompanionScene::SwordSisterFinalReturn => {
            "【小传回访】守灯人拾起月衡留在归路灯边的剑穗，替她补好断线，也备下一份归程盘缠。"
        }
        CompanionScene::SpiritWitchFinalVow => {
            "【小传回访】守灯人把南瑶念过的名字写进灯簿，托你带走一盏药灯和守誓人的谢礼。"
        }
    }
}

fn claim_local_companion_scene_followup(
    kind: MapKind,
    quest: &mut QuestLog,
    stats: &mut PlayerStats,
) -> Vec<String> {
    let mut lines = Vec::new();
    for scene in companion_route_scenes(kind).iter().copied() {
        if quest.companion_scene_has_pending_revisit_turn_in(scene) {
            continue;
        }
        let Some(reward) = quest.claim_companion_aftermath(scene) else {
            continue;
        };
        lines.push(local_companion_aftermath_line(scene).to_string());
        lines.push(apply_companion_aftermath_reward(stats, reward));
    }
    lines
}

fn local_route_detour_for_map(kind: MapKind) -> Option<RouteDetour> {
    match kind {
        MapKind::Cave | MapKind::MoonEchoCorridor => Some(RouteDetour::MoonEchoPool),
        MapKind::RiverTown | MapKind::RiverReedBed => Some(RouteDetour::ReedHiddenFord),
        MapKind::PlagueVillage | MapKind::PlagueShrinePath => Some(RouteDetour::PlagueHerbTrail),
        MapKind::Capital | MapKind::CapitalMansion | MapKind::MansionMirrorGallery => {
            Some(RouteDetour::MirrorServantDoor)
        }
        MapKind::SouthernRoad | MapKind::ThunderDrumPath => Some(RouteDetour::ThunderRidgeCache),
        MapKind::FinalSanctum | MapKind::DreamWaterway => Some(RouteDetour::DreamBackwater),
        MapKind::Village | MapKind::Bamboo => None,
    }
}

fn claim_local_route_detour_report(
    kind: MapKind,
    quest: &mut QuestLog,
    stats: &mut PlayerStats,
) -> Vec<String> {
    let Some(detour) = local_route_detour_for_map(kind) else {
        return Vec::new();
    };
    if !quest.route_detour_report_ready(detour) {
        return Vec::new();
    }

    let report = quest.claim_route_detour_report(detour);
    let mut lines = report.lines;
    if let Some(reward) = report.reward {
        lines.push(apply_route_detour_report_reward(stats, reward));
    }
    lines
}

fn map_care_scenes(kind: MapKind) -> (BondScene, CampScene) {
    match kind {
        MapKind::Village | MapKind::Bamboo => {
            (BondScene::VillageFirstNight, CampScene::VillageHearth)
        }
        MapKind::Cave | MapKind::MoonEchoCorridor => {
            (BondScene::MoonCavePromise, CampScene::MoonCavePool)
        }
        MapKind::RiverTown | MapKind::RiverReedBed => {
            (BondScene::RiverLampWish, CampScene::RiverTownInn)
        }
        MapKind::PlagueVillage | MapKind::PlagueShrinePath => {
            (BondScene::PlagueRainShelter, CampScene::PlagueSickroom)
        }
        MapKind::Capital | MapKind::CapitalMansion | MapKind::MansionMirrorGallery => {
            (BondScene::CapitalRooftop, CampScene::CapitalSafehouse)
        }
        MapKind::SouthernRoad | MapKind::ThunderDrumPath => {
            (BondScene::SouthernRoadOath, CampScene::SouthernCampfire)
        }
        MapKind::FinalSanctum | MapKind::DreamWaterway => {
            (BondScene::FinalGateQuiet, CampScene::FinalStillWater)
        }
    }
}

fn map_chapter(kind: MapKind) -> Chapter {
    match kind {
        MapKind::Village | MapKind::Bamboo => Chapter::VillageOath,
        MapKind::Cave | MapKind::MoonEchoCorridor => Chapter::MoonCave,
        MapKind::RiverTown | MapKind::RiverReedBed => Chapter::RiverMedicine,
        MapKind::PlagueVillage | MapKind::PlagueShrinePath => Chapter::PlagueRain,
        MapKind::Capital | MapKind::CapitalMansion | MapKind::MansionMirrorGallery => {
            Chapter::CapitalMirror
        }
        MapKind::SouthernRoad | MapKind::ThunderDrumPath => Chapter::SouthernThunder,
        MapKind::FinalSanctum | MapKind::DreamWaterway => Chapter::FinalDream,
    }
}

fn chapter_rank(chapter: Chapter) -> u8 {
    match chapter {
        Chapter::VillageOath => 0,
        Chapter::MoonCave => 1,
        Chapter::RiverMedicine => 2,
        Chapter::PlagueRain => 3,
        Chapter::CapitalMirror => 4,
        Chapter::SouthernThunder => 5,
        Chapter::FinalDream => 6,
    }
}

// ---------------------------------------------------------------------------
// Map props
// ---------------------------------------------------------------------------

struct PropDef {
    col: i32,
    row: i32,
    path: &'static str,
    size: f32,
    light: [f32; 4],
}

#[derive(Clone, Copy)]
struct FieldSupplyDef {
    kind: MapKind,
    col: i32,
    row: i32,
    supply: FieldSupply,
    path: &'static str,
    size: f32,
    light: [f32; 4],
}

#[derive(Clone, Copy)]
struct CommissionTraceDef {
    kind: MapKind,
    col: i32,
    row: i32,
    side: SideQuest,
    name: &'static str,
    path: &'static str,
    size: f32,
    light: [f32; 4],
    source: &'static str,
    inactive_line: &'static str,
    active_line: &'static str,
    repeat_line: &'static str,
}

static FIELD_SUPPLY_DEFS: [FieldSupplyDef; 15] = [
    FieldSupplyDef {
        kind: MapKind::Village,
        col: 12,
        row: 13,
        supply: FieldSupply::VillageHerbs,
        path: "props/ai_spring.png",
        size: 36.0,
        light: [0.54, 0.94, 0.44, 0.18],
    },
    FieldSupplyDef {
        kind: MapKind::Bamboo,
        col: 6,
        row: 13,
        supply: FieldSupply::BambooDew,
        path: "props/ai_spring.png",
        size: 34.0,
        light: [0.54, 0.94, 0.62, 0.18],
    },
    FieldSupplyDef {
        kind: MapKind::Cave,
        col: 18,
        row: 7,
        supply: FieldSupply::CaveMoonMoss,
        path: "props/ai_cave_crystal.png",
        size: 34.0,
        light: [0.42, 0.74, 1.0, 0.18],
    },
    FieldSupplyDef {
        kind: MapKind::MoonEchoCorridor,
        col: 16,
        row: 11,
        supply: FieldSupply::MoonCorridorDust,
        path: "props/ai_cave_crystal.png",
        size: 34.0,
        light: [0.54, 0.72, 1.0, 0.20],
    },
    FieldSupplyDef {
        kind: MapKind::RiverTown,
        col: 11,
        row: 11,
        supply: FieldSupply::RiverTeaChest,
        path: "props/ai_chest.png",
        size: 36.0,
        light: [0.70, 0.92, 1.0, 0.14],
    },
    FieldSupplyDef {
        kind: MapKind::RiverReedBed,
        col: 5,
        row: 14,
        supply: FieldSupply::ReedLotusPods,
        path: "props/ai_spring.png",
        size: 34.0,
        light: [0.64, 0.92, 0.54, 0.16],
    },
    FieldSupplyDef {
        kind: MapKind::PlagueVillage,
        col: 12,
        row: 14,
        supply: FieldSupply::PlagueCleanWater,
        path: "props/ai_spring.png",
        size: 36.0,
        light: [0.54, 0.94, 0.58, 0.18],
    },
    FieldSupplyDef {
        kind: MapKind::PlagueShrinePath,
        col: 23,
        row: 12,
        supply: FieldSupply::ShrineAshRoots,
        path: "props/ai_spring.png",
        size: 34.0,
        light: [0.62, 0.96, 0.56, 0.18],
    },
    FieldSupplyDef {
        kind: MapKind::Capital,
        col: 7,
        row: 14,
        supply: FieldSupply::CapitalTeaPacket,
        path: "props/ai_chest.png",
        size: 36.0,
        light: [0.92, 0.78, 1.0, 0.14],
    },
    FieldSupplyDef {
        kind: MapKind::CapitalMansion,
        col: 7,
        row: 14,
        supply: FieldSupply::MansionPantry,
        path: "props/ai_chest.png",
        size: 36.0,
        light: [0.92, 0.78, 1.0, 0.16],
    },
    FieldSupplyDef {
        kind: MapKind::MansionMirrorGallery,
        col: 7,
        row: 14,
        supply: FieldSupply::MirrorPowder,
        path: "props/ai_chest.png",
        size: 36.0,
        light: [0.64, 0.82, 1.0, 0.18],
    },
    FieldSupplyDef {
        kind: MapKind::SouthernRoad,
        col: 7,
        row: 14,
        supply: FieldSupply::SouthernPepper,
        path: "props/ai_spring.png",
        size: 36.0,
        light: [0.78, 1.0, 0.48, 0.18],
    },
    FieldSupplyDef {
        kind: MapKind::ThunderDrumPath,
        col: 7,
        row: 14,
        supply: FieldSupply::ThunderHerbWine,
        path: "props/ai_chest.png",
        size: 38.0,
        light: [0.62, 0.92, 1.0, 0.20],
    },
    FieldSupplyDef {
        kind: MapKind::FinalSanctum,
        col: 7,
        row: 14,
        supply: FieldSupply::FinalIncense,
        path: "props/ai_chest.png",
        size: 38.0,
        light: [0.70, 0.74, 1.0, 0.20],
    },
    FieldSupplyDef {
        kind: MapKind::DreamWaterway,
        col: 6,
        row: 9,
        supply: FieldSupply::DreamPearlMoss,
        path: "props/ai_cave_crystal.png",
        size: 36.0,
        light: [0.52, 0.72, 1.0, 0.22],
    },
];

static COMMISSION_TRACE_DEFS: [CommissionTraceDef; 14] = [
    CommissionTraceDef {
        kind: MapKind::Village,
        col: 15,
        row: 13,
        side: SideQuest::VillageTrail,
        name: "旧竹栅妖痕",
        path: "props/ai_cave_crystal.png",
        size: 30.0,
        light: [0.92, 0.58, 0.32, 0.18],
        source: "旧竹栅妖痕已查",
        inactive_line: "竹栅边草叶断得很新，像是山路余妖留下的爪印。",
        active_line: "你拨开草叶，妖爪从旧竹栅一路拖到村路边，正好写入委托签。",
        repeat_line: "旧竹栅边的爪印已经用石粉圈住，巡山人会照着这里收尾。",
    },
    CommissionTraceDef {
        kind: MapKind::Village,
        col: 13,
        row: 12,
        side: SideQuest::VillageHerbs,
        name: "药圃香风",
        path: "props/ai_spring.png",
        size: 32.0,
        light: [0.62, 0.94, 0.46, 0.20],
        source: "药圃香风已稳",
        inactive_line: "水塘边有一股药香外泄，像是药婆说过的护路麻烦。",
        active_line: "你用湿泥压住药香外泄的缺口，闻香聚来的妖风弱了一截。",
        repeat_line: "药圃边的泥封还稳着，药香不会再把小妖引到路上。",
    },
    CommissionTraceDef {
        kind: MapKind::Cave,
        col: 16,
        row: 6,
        side: SideQuest::MoonCaveCrystals,
        name: "晶尘冷斑",
        path: "props/ai_cave_crystal.png",
        size: 34.0,
        light: [0.44, 0.78, 1.0, 0.24],
        source: "晶尘冷斑已净",
        inactive_line: "洞壁晶尘有一块发黑，像在等水月洞天石牌上的委托。",
        active_line: "你把冷斑上的妖尘拂入符纸，晶路亮回半寸月光。",
        repeat_line: "这块晶尘已经澄清，只剩水光贴着石壁慢慢流动。",
    },
    CommissionTraceDef {
        kind: MapKind::MoonEchoCorridor,
        col: 16,
        row: 14,
        side: SideQuest::MoonCaveEchoes,
        name: "二重回声",
        path: "props/ai_spirit_lantern.png",
        size: 34.0,
        light: [0.58, 0.78, 1.0, 0.24],
        source: "二重回声已压",
        inactive_line: "窄廊尽头回声叠成两层，像是后续委托才会处理的妖影。",
        active_line: "你按住灯火念诀，第二层回声沉进石缝，不再追着脚步走。",
        repeat_line: "窄廊里只剩一层正常回音，灯火照到尽头也不再发抖。",
    },
    CommissionTraceDef {
        kind: MapKind::RiverReedBed,
        col: 6,
        row: 5,
        side: SideQuest::RiverLanterns,
        name: "逆流灯影",
        path: "props/ai_spirit_lantern.png",
        size: 34.0,
        light: [0.48, 0.86, 1.0, 0.26],
        source: "逆流灯影已巡",
        inactive_line: "芦叶间有灯影逆水摇晃，像码头任务板上的巡夜线索。",
        active_line: "你顺着灯影把系绳重新压进浅滩，逆流的光慢慢转回下游。",
        repeat_line: "这盏灯影已经顺流，芦叶间不再有倒走的水光。",
    },
    CommissionTraceDef {
        kind: MapKind::RiverReedBed,
        col: 12,
        row: 10,
        side: SideQuest::RiverCargo,
        name: "湿货草印",
        path: "props/ai_chest.png",
        size: 34.0,
        light: [0.66, 0.88, 1.0, 0.18],
        source: "湿货草印已记",
        inactive_line: "草滩上压着湿麻绳和箱角印，像货主丢失的湿货线索。",
        active_line: "你把箱角印拓在委托签背面，妖风拖货的方向清楚了。",
        repeat_line: "湿货草印已经拓下，水线旁只剩被踩平的芦草。",
    },
    CommissionTraceDef {
        kind: MapKind::PlagueVillage,
        col: 5,
        row: 8,
        side: SideQuest::PlagueRelief,
        name: "瘴草黑结",
        path: "props/ai_spring.png",
        size: 32.0,
        light: [0.46, 0.84, 0.44, 0.22],
        source: "瘴草黑结已清",
        inactive_line: "黑雨草地结着瘴疙瘩，像村中救急榜会记下的病源。",
        active_line: "你用净草压住黑结，瘴雨从草根处断开一小片。",
        repeat_line: "这片黑结已经被净草压住，雨落下来不再冒出腥气。",
    },
    CommissionTraceDef {
        kind: MapKind::PlagueShrinePath,
        col: 9,
        row: 10,
        side: SideQuest::PlagueMedicine,
        name: "药童铃路",
        path: "props/ai_spirit_lantern.png",
        size: 34.0,
        light: [0.58, 0.94, 0.56, 0.24],
        source: "药童铃路已护",
        inactive_line: "祠道草间挂着小铃，像送药童子会走过的瘴路。",
        active_line: "你把小铃重新系牢，铃声顺着药路响了一段，瘴影退开。",
        repeat_line: "小铃还在药路边轻响，药童下次经过会知道往哪边走。",
    },
    CommissionTraceDef {
        kind: MapKind::Capital,
        col: 5,
        row: 5,
        side: SideQuest::CapitalPatrol,
        name: "暗巷镜脚",
        path: "props/ai_cave_crystal.png",
        size: 32.0,
        light: [0.64, 0.72, 1.0, 0.22],
        source: "暗巷镜脚已辨",
        inactive_line: "府城暗巷墙根映着半个脚印，像镜阵幻影巡过这里。",
        active_line: "你用剑鞘点破镜脚，暗巷里少了一道会绕人的假影。",
        repeat_line: "墙根镜脚已经碎成粉，巡查暗号也不再反光。",
    },
    CommissionTraceDef {
        kind: MapKind::MansionMirrorGallery,
        col: 11,
        row: 10,
        side: SideQuest::CapitalRumors,
        name: "茶肆暗帖",
        path: "props/ai_chest.png",
        size: 32.0,
        light: [0.72, 0.74, 1.0, 0.18],
        source: "茶肆暗帖已截",
        inactive_line: "镜廊边压着半张茶肆暗帖，像府城暗线要追的东西。",
        active_line: "你取下暗帖，帖角显出镜阵传手的花押。",
        repeat_line: "暗帖已经收起，镜廊墙根只剩一点烧过的纸灰。",
    },
    CommissionTraceDef {
        kind: MapKind::SouthernRoad,
        col: 8,
        row: 7,
        side: SideQuest::SouthernThunder,
        name: "雷草焦纹",
        path: "props/ai_cave_crystal.png",
        size: 34.0,
        light: [0.62, 0.94, 1.0, 0.26],
        source: "雷草焦纹已安",
        inactive_line: "雷草坡边焦纹乱跳，像百越巡路人说的灵道雷声。",
        active_line: "你按南疆步点踏住焦纹，乱雷往石缝里缩回一截。",
        repeat_line: "焦纹已经顺着石缝排好，雷草坡不再乱闪。",
    },
    CommissionTraceDef {
        kind: MapKind::ThunderDrumPath,
        col: 9,
        row: 10,
        side: SideQuest::SouthernDrums,
        name: "旧鼓低鸣",
        path: "props/ai_spirit_lantern.png",
        size: 34.0,
        light: [0.70, 0.92, 1.0, 0.28],
        source: "旧鼓低鸣已平",
        inactive_line: "雷鼓道深处有低鼓声贴地滚动，像后续战鼓安魂委托。",
        active_line: "你把灵灯压在鼓声起处，低鸣断成三拍后沉入山风。",
        repeat_line: "这段低鼓声已经平下去，只剩远处雷云慢慢散开。",
    },
    CommissionTraceDef {
        kind: MapKind::DreamWaterway,
        col: 16,
        row: 7,
        side: SideQuest::FinalDreamEchoes,
        name: "旧梦水影",
        path: "props/ai_cave_crystal.png",
        size: 34.0,
        light: [0.54, 0.72, 1.0, 0.26],
        source: "旧梦水影已压",
        inactive_line: "梦水里有一截倒影迟迟不散，像终门灯簿上的余波。",
        active_line: "你用灯火压住倒影，旧梦水影从水阶下退开。",
        repeat_line: "这截倒影已经淡了，梦水只映出归路灯色。",
    },
    CommissionTraceDef {
        kind: MapKind::DreamWaterway,
        col: 22,
        row: 11,
        side: SideQuest::FinalHomewardVows,
        name: "归潮灯签",
        path: "props/ai_spirit_lantern.png",
        size: 34.0,
        light: [0.88, 0.72, 1.0, 0.28],
        source: "归潮灯签已护",
        inactive_line: "回湾水面托着一枚灯签，像旧愿还没肯离水。",
        active_line: "你扶正灯签，回潮从签尾分开，归路灯色稳住一段。",
        repeat_line: "灯签已经顺着回潮亮着，旧愿不再把水面拽回去。",
    },
];

const PROPS_VILLAGE: [PropDef; 4] = [
    PropDef {
        col: 21,
        row: 1,
        path: "props/ai_quest_board.png",
        size: 48.0,
        light: [1.0, 0.72, 0.34, 0.20],
    },
    PropDef {
        col: 6,
        row: 8,
        path: "props/ai_spirit_lantern.png",
        size: 42.0,
        light: [1.0, 0.70, 0.28, 0.34],
    },
    PropDef {
        col: 20,
        row: 4,
        path: "props/ai_shrine_statue.png",
        size: 54.0,
        light: [0.62, 0.90, 0.70, 0.16],
    },
    PropDef {
        col: 26,
        row: 1,
        path: "props/ai_spirit_lantern.png",
        size: 38.0,
        light: [0.50, 0.86, 1.0, 0.26],
    },
];

const PROPS_BAMBOO: [PropDef; 4] = [
    PropDef {
        col: 27,
        row: 1,
        path: "props/ai_bamboo_gate.png",
        size: 58.0,
        light: [0.52, 0.92, 0.55, 0.20],
    },
    PropDef {
        col: 5,
        row: 5,
        path: "props/ai_spirit_lantern.png",
        size: 38.0,
        light: [1.0, 0.70, 0.28, 0.28],
    },
    PropDef {
        col: 22,
        row: 12,
        path: "props/ai_cave_crystal.png",
        size: 44.0,
        light: [0.42, 0.82, 1.0, 0.24],
    },
    PropDef {
        col: 12,
        row: 6,
        path: "props/ai_shrine_statue.png",
        size: 48.0,
        light: [0.58, 0.92, 0.62, 0.14],
    },
];

const PROPS_CAVE: [PropDef; 3] = [
    PropDef {
        col: 9,
        row: 7,
        path: "props/ai_shrine_statue.png",
        size: 52.0,
        light: [0.70, 0.72, 1.0, 0.18],
    },
    PropDef {
        col: 27,
        row: 1,
        path: "props/ai_spirit_lantern.png",
        size: 38.0,
        light: [0.52, 0.88, 1.0, 0.28],
    },
    PropDef {
        col: 14,
        row: 13,
        path: "props/ai_quest_board.png",
        size: 44.0,
        light: [0.70, 0.78, 1.0, 0.14],
    },
];

const PROPS_MOON_ECHO_CORRIDOR: [PropDef; 5] = [
    PropDef {
        col: 12,
        row: 3,
        path: "props/ai_cave_crystal.png",
        size: 54.0,
        light: [0.42, 0.82, 1.0, 0.32],
    },
    PropDef {
        col: 20,
        row: 11,
        path: "props/ai_cave_crystal.png",
        size: 46.0,
        light: [0.55, 0.68, 1.0, 0.28],
    },
    PropDef {
        col: 6,
        row: 10,
        path: "props/ai_spirit_lantern.png",
        size: 40.0,
        light: [0.62, 0.86, 1.0, 0.28],
    },
    PropDef {
        col: 27,
        row: 1,
        path: "props/ai_bamboo_gate.png",
        size: 56.0,
        light: [0.52, 0.88, 1.0, 0.22],
    },
    PropDef {
        col: 18,
        row: 6,
        path: "props/ai_cave_crystal.png",
        size: 42.0,
        light: [0.48, 0.76, 1.0, 0.18],
    },
];

const PROPS_RIVER_TOWN: [PropDef; 5] = [
    PropDef {
        col: 24,
        row: 1,
        path: "props/ai_spirit_lantern.png",
        size: 40.0,
        light: [0.42, 0.86, 1.0, 0.28],
    },
    PropDef {
        col: 3,
        row: 4,
        path: "props/ai_quest_board.png",
        size: 46.0,
        light: [0.72, 1.0, 0.56, 0.18],
    },
    PropDef {
        col: 12,
        row: 6,
        path: "props/ai_spirit_lantern.png",
        size: 38.0,
        light: [1.0, 0.70, 0.28, 0.28],
    },
    PropDef {
        col: 26,
        row: 7,
        path: "props/ai_cave_crystal.png",
        size: 40.0,
        light: [0.38, 0.78, 1.0, 0.24],
    },
    PropDef {
        col: 5,
        row: 11,
        path: "props/ai_shrine_statue.png",
        size: 50.0,
        light: [0.62, 0.90, 0.70, 0.16],
    },
];

const PROPS_RIVER_REED_BED: [PropDef; 7] = [
    PropDef {
        col: 9,
        row: 2,
        path: "props/ai_spirit_lantern.png",
        size: 40.0,
        light: [0.42, 0.86, 1.0, 0.30],
    },
    PropDef {
        col: 16,
        row: 6,
        path: "props/ai_spirit_lantern.png",
        size: 40.0,
        light: [0.62, 0.92, 1.0, 0.30],
    },
    PropDef {
        col: 24,
        row: 10,
        path: "props/ai_spirit_lantern.png",
        size: 40.0,
        light: [0.88, 0.72, 1.0, 0.30],
    },
    PropDef {
        col: 18,
        row: 6,
        path: "props/ai_cave_crystal.png",
        size: 42.0,
        light: [0.38, 0.78, 1.0, 0.24],
    },
    PropDef {
        col: 22,
        row: 11,
        path: "props/ai_shrine_statue.png",
        size: 50.0,
        light: [0.62, 0.90, 0.70, 0.16],
    },
    PropDef {
        col: 26,
        row: 1,
        path: "props/ai_bamboo_gate.png",
        size: 56.0,
        light: [0.52, 0.92, 0.55, 0.18],
    },
    PropDef {
        col: 13,
        row: 10,
        path: "props/ai_cave_crystal.png",
        size: 38.0,
        light: [0.44, 0.88, 1.0, 0.22],
    },
];

const PROPS_PLAGUE_VILLAGE: [PropDef; 5] = [
    PropDef {
        col: 24,
        row: 1,
        path: "props/ai_spirit_lantern.png",
        size: 38.0,
        light: [0.46, 0.92, 0.58, 0.26],
    },
    PropDef {
        col: 3,
        row: 4,
        path: "props/ai_quest_board.png",
        size: 46.0,
        light: [0.70, 0.82, 0.48, 0.16],
    },
    PropDef {
        col: 12,
        row: 6,
        path: "props/ai_bamboo_gate.png",
        size: 54.0,
        light: [0.50, 0.90, 0.52, 0.18],
    },
    PropDef {
        col: 26,
        row: 7,
        path: "props/ai_cave_crystal.png",
        size: 40.0,
        light: [0.32, 0.76, 0.52, 0.22],
    },
    PropDef {
        col: 5,
        row: 11,
        path: "props/ai_spirit_lantern.png",
        size: 38.0,
        light: [1.0, 0.70, 0.28, 0.24],
    },
];

const PROPS_PLAGUE_SHRINE_PATH: [PropDef; 5] = [
    PropDef {
        col: 26,
        row: 1,
        path: "props/ai_bamboo_gate.png",
        size: 56.0,
        light: [0.46, 0.92, 0.54, 0.20],
    },
    PropDef {
        col: 12,
        row: 5,
        path: "props/ai_shrine_statue.png",
        size: 54.0,
        light: [0.58, 0.96, 0.62, 0.24],
    },
    PropDef {
        col: 18,
        row: 8,
        path: "props/ai_cave_crystal.png",
        size: 42.0,
        light: [0.34, 0.82, 0.56, 0.26],
    },
    PropDef {
        col: 23,
        row: 11,
        path: "props/ai_spirit_lantern.png",
        size: 40.0,
        light: [1.0, 0.72, 0.32, 0.28],
    },
    PropDef {
        col: 10,
        row: 13,
        path: "props/ai_cave_crystal.png",
        size: 40.0,
        light: [0.42, 0.84, 0.64, 0.20],
    },
];

const PROPS_CAPITAL: [PropDef; 5] = [
    PropDef {
        col: 24,
        row: 1,
        path: "props/ai_spirit_lantern.png",
        size: 38.0,
        light: [0.62, 0.86, 1.0, 0.26],
    },
    PropDef {
        col: 3,
        row: 4,
        path: "props/ai_quest_board.png",
        size: 46.0,
        light: [1.0, 0.72, 0.34, 0.16],
    },
    PropDef {
        col: 12,
        row: 6,
        path: "props/ai_shrine_statue.png",
        size: 52.0,
        light: [0.72, 0.72, 1.0, 0.20],
    },
    PropDef {
        col: 26,
        row: 7,
        path: "props/ai_cave_crystal.png",
        size: 40.0,
        light: [0.50, 0.78, 1.0, 0.24],
    },
    PropDef {
        col: 5,
        row: 11,
        path: "props/ai_spirit_lantern.png",
        size: 38.0,
        light: [1.0, 0.70, 0.28, 0.24],
    },
];

const PROPS_CAPITAL_MANSION: [PropDef; 5] = [
    PropDef {
        col: 26,
        row: 1,
        path: "props/ai_bamboo_gate.png",
        size: 56.0,
        light: [0.52, 0.76, 1.0, 0.18],
    },
    PropDef {
        col: 12,
        row: 6,
        path: "props/ai_cave_crystal.png",
        size: 46.0,
        light: [0.50, 0.78, 1.0, 0.30],
    },
    PropDef {
        col: 20,
        row: 3,
        path: "props/ai_shrine_statue.png",
        size: 52.0,
        light: [0.72, 0.72, 1.0, 0.18],
    },
    PropDef {
        col: 7,
        row: 10,
        path: "props/ai_spirit_lantern.png",
        size: 38.0,
        light: [1.0, 0.70, 0.28, 0.26],
    },
    PropDef {
        col: 24,
        row: 11,
        path: "props/ai_cave_crystal.png",
        size: 42.0,
        light: [0.42, 0.82, 1.0, 0.26],
    },
];

const PROPS_MANSION_MIRROR_GALLERY: [PropDef; 6] = [
    PropDef {
        col: 26,
        row: 1,
        path: "props/ai_bamboo_gate.png",
        size: 56.0,
        light: [0.52, 0.76, 1.0, 0.20],
    },
    PropDef {
        col: 12,
        row: 6,
        path: "props/ai_cave_crystal.png",
        size: 48.0,
        light: [0.54, 0.78, 1.0, 0.34],
    },
    PropDef {
        col: 24,
        row: 11,
        path: "props/ai_cave_crystal.png",
        size: 44.0,
        light: [0.46, 0.82, 1.0, 0.30],
    },
    PropDef {
        col: 20,
        row: 3,
        path: "props/ai_shrine_statue.png",
        size: 52.0,
        light: [0.72, 0.72, 1.0, 0.18],
    },
    PropDef {
        col: 7,
        row: 10,
        path: "props/ai_spirit_lantern.png",
        size: 38.0,
        light: [1.0, 0.70, 0.28, 0.26],
    },
    PropDef {
        col: 6,
        row: 13,
        path: "props/ai_bamboo_gate.png",
        size: 52.0,
        light: [0.58, 0.76, 1.0, 0.18],
    },
];

const PROPS_SOUTHERN_ROAD: [PropDef; 5] = [
    PropDef {
        col: 26,
        row: 1,
        path: "props/ai_bamboo_gate.png",
        size: 58.0,
        light: [0.52, 0.92, 0.55, 0.20],
    },
    PropDef {
        col: 3,
        row: 4,
        path: "props/ai_quest_board.png",
        size: 46.0,
        light: [1.0, 0.78, 0.34, 0.16],
    },
    PropDef {
        col: 12,
        row: 5,
        path: "props/ai_shrine_statue.png",
        size: 52.0,
        light: [0.62, 0.92, 1.0, 0.22],
    },
    PropDef {
        col: 24,
        row: 11,
        path: "props/ai_cave_crystal.png",
        size: 42.0,
        light: [0.42, 0.82, 1.0, 0.28],
    },
    PropDef {
        col: 10,
        row: 13,
        path: "props/ai_spirit_lantern.png",
        size: 38.0,
        light: [1.0, 0.70, 0.28, 0.30],
    },
];

const PROPS_THUNDER_DRUM_PATH: [PropDef; 5] = [
    PropDef {
        col: 26,
        row: 1,
        path: "props/ai_bamboo_gate.png",
        size: 58.0,
        light: [0.52, 0.92, 0.55, 0.22],
    },
    PropDef {
        col: 12,
        row: 5,
        path: "props/ai_shrine_statue.png",
        size: 54.0,
        light: [0.62, 0.92, 1.0, 0.28],
    },
    PropDef {
        col: 24,
        row: 11,
        path: "props/ai_cave_crystal.png",
        size: 44.0,
        light: [0.42, 0.82, 1.0, 0.32],
    },
    PropDef {
        col: 10,
        row: 13,
        path: "props/ai_spirit_lantern.png",
        size: 40.0,
        light: [1.0, 0.70, 0.28, 0.34],
    },
    PropDef {
        col: 18,
        row: 6,
        path: "props/ai_cave_crystal.png",
        size: 40.0,
        light: [0.48, 0.88, 1.0, 0.22],
    },
];

const PROPS_FINAL_SANCTUM: [PropDef; 5] = [
    PropDef {
        col: 26,
        row: 1,
        path: "props/ai_cave_crystal.png",
        size: 44.0,
        light: [0.42, 0.82, 1.0, 0.28],
    },
    PropDef {
        col: 3,
        row: 4,
        path: "props/ai_quest_board.png",
        size: 46.0,
        light: [0.72, 0.86, 1.0, 0.18],
    },
    PropDef {
        col: 8,
        row: 12,
        path: "props/ai_spirit_lantern.png",
        size: 42.0,
        light: [1.0, 0.72, 0.34, 0.34],
    },
    PropDef {
        col: 12,
        row: 6,
        path: "props/ai_shrine_statue.png",
        size: 52.0,
        light: [0.72, 0.72, 1.0, 0.20],
    },
    PropDef {
        col: 18,
        row: 10,
        path: "props/ai_bamboo_gate.png",
        size: 58.0,
        light: [0.52, 0.92, 1.0, 0.20],
    },
];

const PROPS_DREAM_WATERWAY: [PropDef; 5] = [
    PropDef {
        col: 26,
        row: 1,
        path: "props/ai_bamboo_gate.png",
        size: 58.0,
        light: [0.52, 0.92, 1.0, 0.22],
    },
    PropDef {
        col: 8,
        row: 12,
        path: "props/ai_spirit_lantern.png",
        size: 44.0,
        light: [1.0, 0.72, 0.34, 0.38],
    },
    PropDef {
        col: 11,
        row: 5,
        path: "props/ai_spirit_lantern.png",
        size: 44.0,
        light: [0.70, 0.92, 1.0, 0.38],
    },
    PropDef {
        col: 24,
        row: 13,
        path: "props/ai_spirit_lantern.png",
        size: 44.0,
        light: [0.92, 0.62, 1.0, 0.38],
    },
    PropDef {
        col: 18,
        row: 10,
        path: "props/ai_cave_crystal.png",
        size: 42.0,
        light: [0.42, 0.82, 1.0, 0.24],
    },
];

fn prop_defs(kind: MapKind) -> &'static [PropDef] {
    match kind {
        MapKind::Village => &PROPS_VILLAGE,
        MapKind::Bamboo => &PROPS_BAMBOO,
        MapKind::Cave => &PROPS_CAVE,
        MapKind::MoonEchoCorridor => &PROPS_MOON_ECHO_CORRIDOR,
        MapKind::RiverTown => &PROPS_RIVER_TOWN,
        MapKind::RiverReedBed => &PROPS_RIVER_REED_BED,
        MapKind::PlagueVillage => &PROPS_PLAGUE_VILLAGE,
        MapKind::PlagueShrinePath => &PROPS_PLAGUE_SHRINE_PATH,
        MapKind::Capital => &PROPS_CAPITAL,
        MapKind::CapitalMansion => &PROPS_CAPITAL_MANSION,
        MapKind::MansionMirrorGallery => &PROPS_MANSION_MIRROR_GALLERY,
        MapKind::SouthernRoad => &PROPS_SOUTHERN_ROAD,
        MapKind::ThunderDrumPath => &PROPS_THUNDER_DRUM_PATH,
        MapKind::FinalSanctum => &PROPS_FINAL_SANCTUM,
        MapKind::DreamWaterway => &PROPS_DREAM_WATERWAY,
    }
}

fn field_supply_defs(kind: MapKind) -> impl Iterator<Item = &'static FieldSupplyDef> {
    FIELD_SUPPLY_DEFS
        .iter()
        .filter(move |supply| supply.kind == kind)
}

fn commission_trace_defs(kind: MapKind) -> impl Iterator<Item = &'static CommissionTraceDef> {
    COMMISSION_TRACE_DEFS
        .iter()
        .filter(move |trace| trace.kind == kind)
}

const SIDE_QUESTS_VILLAGE: [SideQuest; 2] = [SideQuest::VillageTrail, SideQuest::VillageHerbs];
const SIDE_QUESTS_CAVE: [SideQuest; 2] = [SideQuest::MoonCaveCrystals, SideQuest::MoonCaveEchoes];
const SIDE_QUESTS_RIVER_TOWN: [SideQuest; 2] = [SideQuest::RiverLanterns, SideQuest::RiverCargo];
const SIDE_QUESTS_PLAGUE_VILLAGE: [SideQuest; 2] =
    [SideQuest::PlagueRelief, SideQuest::PlagueMedicine];
const SIDE_QUESTS_CAPITAL: [SideQuest; 2] = [SideQuest::CapitalPatrol, SideQuest::CapitalRumors];
const SIDE_QUESTS_SOUTHERN_ROAD: [SideQuest; 2] =
    [SideQuest::SouthernThunder, SideQuest::SouthernDrums];
const SIDE_QUESTS_FINAL_SANCTUM: [SideQuest; 2] =
    [SideQuest::FinalDreamEchoes, SideQuest::FinalHomewardVows];
const ALL_LOCAL_SIDE_QUESTS: [SideQuest; 14] = [
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
const MAX_SIDE_BOARD_OPTIONS: usize = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SideBoardTaskOption {
    side: SideQuest,
    status: SideBoardTaskStatus,
    progress: u32,
    goal: u32,
    receipt_id: &'static str,
}

struct SideQuestContactHandoff {
    lines: Vec<String>,
    choice: Option<DialogueChoice>,
}

struct NpcErrandHandoff {
    lines: Vec<String>,
    choice: Option<DialogueChoice>,
}

struct CompanionRevisitHandoff {
    lines: Vec<String>,
    choice: Option<DialogueChoice>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SideBoardTaskStatus {
    Ready,
    Active,
    Available,
    Locked,
    Completed,
}

impl SideBoardTaskStatus {
    fn label(self) -> &'static str {
        match self {
            Self::Ready => "可交付",
            Self::Active => "进行中",
            Self::Available => "可领取",
            Self::Locked => "后续",
            Self::Completed => "已完成",
        }
    }

    fn action_label(self) -> &'static str {
        match self {
            Self::Ready => "交付领奖",
            Self::Active => "查看进度",
            Self::Available => "领取追踪",
            Self::Locked => "查看后续",
            Self::Completed => "查看归档",
        }
    }
}

impl SideBoardTaskOption {
    fn option_label(self) -> String {
        format!(
            "{} · {} {}/{} · {}",
            self.status.action_label(),
            self.side.name(),
            self.progress,
            self.goal,
            self.receipt_id
        )
    }
}

fn side_quests_for_board(kind: MapKind, prop: &PropDef) -> &'static [SideQuest] {
    if prop.path != "props/ai_quest_board.png" {
        return &[];
    }

    side_quests_for_map(kind)
}

fn side_quests_for_map(kind: MapKind) -> &'static [SideQuest] {
    match kind {
        MapKind::Village => &SIDE_QUESTS_VILLAGE,
        MapKind::Cave => &SIDE_QUESTS_CAVE,
        MapKind::RiverTown => &SIDE_QUESTS_RIVER_TOWN,
        MapKind::PlagueVillage => &SIDE_QUESTS_PLAGUE_VILLAGE,
        MapKind::Capital => &SIDE_QUESTS_CAPITAL,
        MapKind::SouthernRoad => &SIDE_QUESTS_SOUTHERN_ROAD,
        MapKind::FinalSanctum => &SIDE_QUESTS_FINAL_SANCTUM,
        MapKind::Bamboo
        | MapKind::MoonEchoCorridor
        | MapKind::RiverReedBed
        | MapKind::PlagueShrinePath
        | MapKind::CapitalMansion
        | MapKind::MansionMirrorGallery
        | MapKind::ThunderDrumPath
        | MapKind::DreamWaterway => &[],
    }
}

fn is_side_quest_contact(kind: MapKind, npc: &NpcDef) -> bool {
    if npc.quest.is_some() {
        return false;
    }

    matches!(
        (kind, npc.col, npc.row),
        (MapKind::Village, 6, 7)
            | (MapKind::Cave, 24, 12)
            | (MapKind::RiverTown, 25, 1)
            | (MapKind::PlagueVillage, 25, 1)
            | (MapKind::Capital, 13, 6)
            | (MapKind::SouthernRoad, 11, 13)
            | (MapKind::FinalSanctum, 5, 12)
    )
}

fn npc_speaker_name(npc: &NpcDef) -> &'static str {
    npc.lines
        .iter()
        .filter_map(|line| line.split_once('：').map(|(speaker, _)| speaker))
        .find(|speaker| !speaker.is_empty() && *speaker != "李逍遥")
        .filter(|speaker| !speaker.is_empty())
        .unwrap_or("当地人")
}

fn side_quest_contact_focus(kind: MapKind, quest: &QuestLog) -> Option<SideBoardTaskOption> {
    side_board_task_options(kind, quest)
        .into_iter()
        .flatten()
        .find(|option| option.status != SideBoardTaskStatus::Completed)
}

fn npc_side_quest_contact_prompt(kind: MapKind, npc: &NpcDef, quest: &QuestLog) -> Option<String> {
    if !is_side_quest_contact(kind, npc) {
        return None;
    }

    let focus = side_quest_contact_focus(kind, quest)?;
    let action = match focus.status {
        SideBoardTaskStatus::Ready => "交付",
        SideBoardTaskStatus::Active => "询问",
        SideBoardTaskStatus::Available => "领取",
        SideBoardTaskStatus::Locked => "打听",
        SideBoardTaskStatus::Completed => "查看",
    };
    Some(format!(
        "委托联系人 {} | {}[{}] | 空格{}",
        npc_speaker_name(npc),
        focus.side.name(),
        focus.status.label(),
        action
    ))
}

fn npc_side_quest_handoff(
    kind: MapKind,
    npc: &NpcDef,
    quest: &QuestLog,
) -> Option<SideQuestContactHandoff> {
    if !is_side_quest_contact(kind, npc) {
        return None;
    }

    let focus = side_quest_contact_focus(kind, quest)?;
    let speaker = npc_speaker_name(npc);
    let side = focus.side;
    let mut lines = vec![format!(
        "【委托联系人】{}把《{}》指给你。",
        speaker,
        side.name()
    )];

    let choice = match focus.status {
        SideBoardTaskStatus::Ready => {
            lines.push(quest.side_task_detail(side));
            lines.push(quest.side_task_contract(side));
            lines.push(format!(
                "【催交】{}确认条件已够，可以当场替任务板登记交付裁断。",
                speaker
            ));
            side_quest_choice_for(quest, side)
        }
        SideBoardTaskStatus::Available => {
            lines.push(format!(
                "【领委托】{}说明委托不只是贴在板上，也能由当地人作保写入任务簿。",
                speaker
            ));
            lines.extend(quest.side_task_accept_preview(side));
            side_quest_choice_for(quest, side)
        }
        SideBoardTaskStatus::Active => {
            let progress = quest
                .side_quest_progress(side)
                .min(quest.side_quest_goal(side));
            lines.push(format!(
                "【委托在身】{}核对《{}》进度 {}/{}。",
                speaker,
                side.name(),
                progress,
                quest.side_quest_goal(side)
            ));
            lines.push(quest.side_task_contract(side));
            lines.push(format!("【路线线索】{}", quest.side_quest_route_hint(side)));
            lines.push(quest.side_task_summary(side));
            None
        }
        SideBoardTaskStatus::Locked => {
            if let Some(required) = side.prerequisite() {
                lines.push(format!(
                    "【后续委托】{}说《{}》还不能揭，需先交清《{}》。",
                    speaker,
                    side.name(),
                    required.name()
                ));
            }
            lines.push(quest.side_task_contract(side));
            lines.push(quest.side_task_summary(side));
            None
        }
        SideBoardTaskStatus::Completed => None,
    };

    Some(SideQuestContactHandoff { lines, choice })
}

fn npc_errand_offer_for(kind: MapKind, npc: &NpcDef) -> Option<NpcErrand> {
    if npc.quest.is_some() {
        return None;
    }

    match (kind, npc.col, npc.row) {
        (MapKind::Bamboo, 5, 6) => Some(NpcErrand::BambooDewToCave),
        (MapKind::MoonEchoCorridor, 12, 13) => Some(NpcErrand::MoonMossToRiver),
        (MapKind::RiverTown, 13, 6) => Some(NpcErrand::RiverReedLetter),
        (MapKind::PlagueVillage, 27, 7) => Some(NpcErrand::PlagueChildCharm),
        (MapKind::Capital, 13, 6) => Some(NpcErrand::CapitalStarSlip),
        (MapKind::MansionMirrorGallery, 27, 7) => Some(NpcErrand::MirrorMedicineToSouth),
        (MapKind::SouthernRoad, 27, 7) => Some(NpcErrand::SouthernThunderWine),
        (MapKind::FinalSanctum, 27, 7) => Some(NpcErrand::FinalLampWick),
        _ => None,
    }
}

fn npc_errand_delivery_for(kind: MapKind, npc: &NpcDef) -> Option<NpcErrand> {
    if npc.quest.is_some() {
        return None;
    }

    match (kind, npc.col, npc.row) {
        (MapKind::Cave, 24, 12) => Some(NpcErrand::BambooDewToCave),
        (MapKind::RiverTown, 25, 1) => Some(NpcErrand::MoonMossToRiver),
        (MapKind::RiverReedBed, 3, 3) => Some(NpcErrand::RiverReedLetter),
        (MapKind::PlagueShrinePath, 13, 5) => Some(NpcErrand::PlagueChildCharm),
        (MapKind::MansionMirrorGallery, 13, 6) => Some(NpcErrand::CapitalStarSlip),
        (MapKind::SouthernRoad, 5, 10) => Some(NpcErrand::MirrorMedicineToSouth),
        (MapKind::ThunderDrumPath, 27, 7) => Some(NpcErrand::SouthernThunderWine),
        (MapKind::DreamWaterway, 13, 6) => Some(NpcErrand::FinalLampWick),
        _ => None,
    }
}

fn npc_errand_facing_prompt(kind: MapKind, pos: &PlayerPos, quest: &QuestLog) -> Option<String> {
    let tc = pos.col + pos.facing.x;
    let tr = pos.row + pos.facing.y;
    let npc = npc_defs(kind)
        .iter()
        .find(|npc| npc.col == tc && npc.row == tr)?;

    if let Some(errand) = npc_errand_delivery_for(kind, npc) {
        if quest.is_npc_errand_active(errand) {
            return Some(format!(
                "NPC托付 {} [{}] | 空格交付\n{}",
                errand.name(),
                quest.npc_errand_progress_label(errand),
                quest.npc_errand_contract(errand)
            ));
        }
        if quest.is_npc_errand_completed(errand) {
            return Some(format!(
                "NPC托付 {} [已送达] | 空格查看\n{}",
                errand.name(),
                quest.npc_errand_summary()
            ));
        }
    }

    let errand = npc_errand_offer_for(kind, npc)?;
    if quest.is_npc_errand_available(errand) {
        Some(format!(
            "NPC托付 {} [可托付] | 空格领取\n{}",
            errand.name(),
            quest.npc_errand_contract(errand)
        ))
    } else if quest.is_npc_errand_active(errand) {
        Some(format!(
            "NPC托付 {} [进行中] | 空格查看\n{}",
            errand.name(),
            quest.npc_errand_contract(errand)
        ))
    } else if quest.is_npc_errand_completed(errand) {
        Some(format!(
            "NPC托付 {} [已送达] | 空格查看\n{}",
            errand.name(),
            quest.npc_errand_summary()
        ))
    } else {
        None
    }
}

fn npc_errand_handoff(kind: MapKind, npc: &NpcDef, quest: &QuestLog) -> Option<NpcErrandHandoff> {
    if let Some(errand) = npc_errand_delivery_for(kind, npc) {
        if quest.is_npc_errand_active(errand) {
            return Some(NpcErrandHandoff {
                lines: vec![
                    format!(
                        "【NPC托付】{}认出你带来的《{}》。",
                        npc_speaker_name(npc),
                        errand.name()
                    ),
                    quest.npc_errand_contract(errand),
                    "确认交付后会结算托付回礼。".to_string(),
                ],
                choice: Some(DialogueChoice::npc_errand(
                    errand,
                    NpcErrandChoiceAction::TurnIn,
                )),
            });
        }
        if quest.is_npc_errand_completed(errand) {
            return Some(NpcErrandHandoff {
                lines: vec![
                    format!("【NPC托付】《{}》已经送达。", errand.name()),
                    quest.npc_errand_summary(),
                ],
                choice: None,
            });
        }
    }

    let errand = npc_errand_offer_for(kind, npc)?;
    if quest.is_npc_errand_available(errand) {
        return Some(NpcErrandHandoff {
            lines: quest.npc_errand_accept_preview(errand),
            choice: Some(DialogueChoice::npc_errand(
                errand,
                NpcErrandChoiceAction::Accept,
            )),
        });
    }

    if quest.is_npc_errand_active(errand) {
        return Some(NpcErrandHandoff {
            lines: vec![
                format!("【NPC托付】《{}》已经在任务簿。", errand.name()),
                quest.npc_errand_contract(errand),
                quest.npc_errand_summary(),
            ],
            choice: None,
        });
    }

    if quest.is_npc_errand_completed(errand) {
        return Some(NpcErrandHandoff {
            lines: vec![
                format!("【NPC托付】《{}》已经送达。", errand.name()),
                quest.npc_errand_summary(),
            ],
            choice: None,
        });
    }

    None
}

fn npc_companion_revisit_for(kind: MapKind, npc: &NpcDef) -> Option<CompanionRevisit> {
    if npc.quest.is_some() {
        return None;
    }

    match (kind, npc.col, npc.row) {
        (MapKind::PlagueVillage, 5, 12) => Some(CompanionRevisit::TrailEcho),
        (MapKind::SouthernRoad, 25, 14) => Some(CompanionRevisit::MirrorTrace),
        (MapKind::FinalSanctum, 24, 1) => Some(CompanionRevisit::TotemVow),
        _ => None,
    }
}

fn companion_revisit_giver_map(revisit: CompanionRevisit) -> MapKind {
    match revisit {
        CompanionRevisit::TrailEcho => MapKind::PlagueVillage,
        CompanionRevisit::MirrorTrace => MapKind::SouthernRoad,
        CompanionRevisit::TotemVow => MapKind::FinalSanctum,
    }
}

fn companion_revisit_target_map(revisit: CompanionRevisit) -> MapKind {
    match revisit {
        CompanionRevisit::TrailEcho => MapKind::PlagueShrinePath,
        CompanionRevisit::MirrorTrace => MapKind::ThunderDrumPath,
        CompanionRevisit::TotemVow => MapKind::DreamWaterway,
    }
}

fn npc_companion_revisit_facing_prompt(
    kind: MapKind,
    pos: &PlayerPos,
    quest: &QuestLog,
) -> Option<String> {
    let tc = pos.col + pos.facing.x;
    let tr = pos.row + pos.facing.y;
    let npc = npc_defs(kind)
        .iter()
        .find(|npc| npc.col == tc && npc.row == tr)?;
    let revisit = npc_companion_revisit_for(kind, npc)?;
    if quest.companion_revisit_completed(revisit)
        || (!quest.companion_revisit_available(revisit)
            && !quest.companion_revisit_active(revisit)
            && !quest.companion_revisit_ready(revisit))
    {
        return None;
    }

    let action = if quest.companion_revisit_ready(revisit) {
        "交付"
    } else if quest.companion_revisit_active(revisit) {
        "查看"
    } else {
        "领取"
    };
    Some(format!(
        "同伴补访 {} [{}] | 空格{}\n{}",
        revisit.name(),
        quest.companion_revisit_status(revisit),
        action,
        quest.companion_revisit_contract(revisit)
    ))
}

fn npc_companion_revisit_handoff(
    kind: MapKind,
    npc: &NpcDef,
    quest: &QuestLog,
) -> Option<CompanionRevisitHandoff> {
    let revisit = npc_companion_revisit_for(kind, npc)?;
    if quest.companion_revisit_available(revisit) {
        return Some(CompanionRevisitHandoff {
            lines: quest.companion_revisit_accept_preview(revisit),
            choice: Some(DialogueChoice::companion_revisit(
                revisit,
                CompanionRevisitChoiceAction::Accept,
            )),
        });
    }
    if quest.companion_revisit_active(revisit) {
        return Some(CompanionRevisitHandoff {
            lines: vec![
                format!("【同伴补访】《{}》仍在寻访中。", revisit.name()),
                quest.companion_revisit_contract(revisit),
                quest.companion_revisit_summary(),
            ],
            choice: None,
        });
    }
    if quest.companion_revisit_ready(revisit) {
        return Some(CompanionRevisitHandoff {
            lines: vec![
                format!("【补访待交】{}看见你带回的旧物回声。", revisit.issuer()),
                quest.companion_revisit_contract(revisit),
                "确认交付后会归档迟来小传并结算回礼。".to_string(),
            ],
            choice: Some(DialogueChoice::companion_revisit(
                revisit,
                CompanionRevisitChoiceAction::TurnIn,
            )),
        });
    }

    None
}

fn local_companion_revisit_intake_tracker(kind: MapKind, quest: &QuestLog) -> Option<String> {
    let revisit = npc_defs(kind)
        .iter()
        .filter_map(|npc| npc_companion_revisit_for(kind, npc))
        .find(|revisit| quest.companion_revisit_available(*revisit))?;
    Some(format!(
        "本地补访 · {} [可领取]\n补访签：{} · 交托：{}\n旧事：{}\n现场：{}\n领取：面对交托 NPC 按空格",
        revisit.name(),
        revisit.receipt_id(),
        revisit.issuer(),
        revisit.scene().title(),
        revisit.target_place()
    ))
}

fn local_npc_errand_intake_tracker(kind: MapKind, quest: &QuestLog) -> Option<String> {
    let errand = npc_defs(kind)
        .iter()
        .filter_map(|npc| npc_errand_offer_for(kind, npc))
        .find(|errand| quest.is_npc_errand_available(*errand))?;

    Some(format!(
        "本地托付 · {} [可托付]\n托付签：{} · {} -> {}\n目标：{}\n路线：{}\n领取：面对托付 NPC 按空格",
        errand.name(),
        quest.npc_errand_receipt_id(errand),
        quest.npc_errand_issuer(errand),
        quest.npc_errand_receiver(errand),
        quest.npc_errand_objective(errand),
        compact_hud_text(quest.npc_errand_route_hint(errand), 42),
    ))
}

fn current_side_quest(sides: &[SideQuest], quest: &QuestLog) -> Option<SideQuest> {
    sides
        .iter()
        .copied()
        .find(|side| quest.is_side_quest_active(*side) && !quest.is_side_quest_completed(*side))
        .or_else(|| {
            sides
                .iter()
                .copied()
                .find(|side| !quest.is_side_quest_completed(*side))
        })
        .or_else(|| sides.last().copied())
}

#[cfg(test)]
fn current_side_quest_for_board(
    kind: MapKind,
    prop: &PropDef,
    quest: &QuestLog,
) -> Option<SideQuest> {
    current_side_quest(side_quests_for_board(kind, prop), quest)
}

#[cfg(test)]
fn current_side_quest_for_map(kind: MapKind, quest: &QuestLog) -> Option<SideQuest> {
    current_side_quest(side_quests_for_map(kind), quest)
}

fn side_board_completed_count(sides: &[SideQuest], quest: &QuestLog) -> usize {
    sides
        .iter()
        .filter(|side| quest.is_side_quest_completed(**side))
        .count()
}

fn local_favor_unlocked(kind: MapKind, quest: &QuestLog) -> bool {
    let sides = side_quests_for_map(kind);
    !sides.is_empty() && side_board_completed_count(sides, quest) == sides.len()
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SideQuestRouteReliefState {
    NoRoute,
    Partial,
    Cleared,
}

fn side_quest_route_relief_sides(kind: MapKind) -> &'static [SideQuest] {
    match kind {
        MapKind::Village | MapKind::Bamboo => &SIDE_QUESTS_VILLAGE,
        MapKind::Cave | MapKind::MoonEchoCorridor => &SIDE_QUESTS_CAVE,
        MapKind::RiverTown | MapKind::RiverReedBed => &SIDE_QUESTS_RIVER_TOWN,
        MapKind::PlagueVillage | MapKind::PlagueShrinePath => &SIDE_QUESTS_PLAGUE_VILLAGE,
        MapKind::Capital | MapKind::CapitalMansion | MapKind::MansionMirrorGallery => {
            &SIDE_QUESTS_CAPITAL
        }
        MapKind::SouthernRoad | MapKind::ThunderDrumPath => &SIDE_QUESTS_SOUTHERN_ROAD,
        MapKind::FinalSanctum | MapKind::DreamWaterway => &SIDE_QUESTS_FINAL_SANCTUM,
    }
}

fn side_quest_route_relief_state(kind: MapKind, quest: &QuestLog) -> SideQuestRouteReliefState {
    let sides = side_quest_route_relief_sides(kind);
    let completed = side_board_completed_count(sides, quest);
    if completed == 0 {
        SideQuestRouteReliefState::NoRoute
    } else if completed == sides.len() {
        SideQuestRouteReliefState::Cleared
    } else {
        SideQuestRouteReliefState::Partial
    }
}

fn side_quest_route_pursuit_count(kind: MapKind, quest: &QuestLog) -> usize {
    side_quest_route_relief_sides(kind)
        .iter()
        .copied()
        .filter(|side| quest.side_quest_resolution(*side) == Some(SideQuestResolution::Pursue))
        .count()
}

fn side_quest_route_field_count(
    kind: MapKind,
    quest: &QuestLog,
    approach: SideQuestFieldApproach,
) -> usize {
    side_quest_route_relief_sides(kind)
        .iter()
        .copied()
        .filter(|side| quest.is_side_quest_completed(*side))
        .filter(|side| quest.side_quest_field_approach(*side) == Some(approach))
        .count()
}

fn side_quest_route_field_tail(kind: MapKind, quest: &QuestLog) -> String {
    let investigated =
        side_quest_route_field_count(kind, quest, SideQuestFieldApproach::Investigate);
    let confronted = side_quest_route_field_count(kind, quest, SideQuestFieldApproach::Confront);
    let mut parts = Vec::new();
    if investigated > 0 {
        parts.push(format!("细查{investigated}"));
    }
    if confronted > 0 {
        parts.push(format!("快断{confronted}"));
    }
    if parts.is_empty() {
        String::new()
    } else {
        format!(" · {}", parts.join(" · "))
    }
}

fn side_quest_route_field_bonus_count(kind: MapKind, quest: &QuestLog) -> usize {
    side_quest_route_field_count(kind, quest, SideQuestFieldApproach::Investigate)
        + side_quest_route_field_count(kind, quest, SideQuestFieldApproach::Confront)
}

fn side_quest_route_relief_multiplier(kind: MapKind, quest: &QuestLog) -> f32 {
    let base = match side_quest_route_relief_state(kind, quest) {
        SideQuestRouteReliefState::NoRoute => 1.0,
        SideQuestRouteReliefState::Partial => 0.97,
        SideQuestRouteReliefState::Cleared => 0.90,
    };
    let pursuit_bonus = side_quest_route_pursuit_count(kind, quest) as f32 * 0.02;
    let field_bonus = side_quest_route_field_bonus_count(kind, quest) as f32 * 0.01;
    (base - pursuit_bonus - field_bonus).max(0.82)
}

fn side_quest_route_relief_summary(kind: MapKind, quest: &QuestLog) -> Option<String> {
    let pursuit = side_quest_route_pursuit_count(kind, quest);
    let pursuit_tail = if pursuit > 0 {
        format!(" · 追查{pursuit}")
    } else {
        String::new()
    };
    let field_count = side_quest_route_field_bonus_count(kind, quest);
    let field_tail = side_quest_route_field_tail(kind, quest);
    let tail = format!("{pursuit_tail}{field_tail}");
    match side_quest_route_relief_state(kind, quest) {
        SideQuestRouteReliefState::NoRoute => None,
        SideQuestRouteReliefState::Partial => Some(format!(
            "委托清障 半稳 遇妖-{}%{}",
            3 + pursuit * 2 + field_count,
            tail
        )),
        SideQuestRouteReliefState::Cleared => Some(format!(
            "委托清障 已清 遇妖-{}%{}",
            10 + pursuit * 2 + field_count,
            tail
        )),
    }
}

fn side_board_task_status(quest: &QuestLog, side: SideQuest) -> SideBoardTaskStatus {
    if quest.is_side_quest_completed(side) {
        SideBoardTaskStatus::Completed
    } else if quest.is_side_quest_active(side) {
        if quest.side_quest_progress(side) >= quest.side_quest_goal(side) {
            SideBoardTaskStatus::Ready
        } else {
            SideBoardTaskStatus::Active
        }
    } else if !quest.is_side_quest_unlocked(side) {
        SideBoardTaskStatus::Locked
    } else {
        SideBoardTaskStatus::Available
    }
}

fn side_board_status_priority(status: SideBoardTaskStatus) -> usize {
    match status {
        SideBoardTaskStatus::Ready => 0,
        SideBoardTaskStatus::Active => 1,
        SideBoardTaskStatus::Available => 2,
        SideBoardTaskStatus::Locked => 3,
        SideBoardTaskStatus::Completed => 4,
    }
}

fn side_board_task_options(
    kind: MapKind,
    quest: &QuestLog,
) -> [Option<SideBoardTaskOption>; MAX_SIDE_BOARD_OPTIONS] {
    let mut ordered: Vec<SideBoardTaskOption> = side_quests_for_map(kind)
        .iter()
        .copied()
        .map(|side| {
            let progress = if quest.is_side_quest_completed(side) {
                quest.side_quest_goal(side)
            } else {
                quest
                    .side_quest_progress(side)
                    .min(quest.side_quest_goal(side))
            };
            SideBoardTaskOption {
                side,
                status: side_board_task_status(quest, side),
                progress,
                goal: quest.side_quest_goal(side),
                receipt_id: quest.side_quest_receipt_id(side),
            }
        })
        .collect();
    ordered.sort_by_key(|option| side_board_status_priority(option.status));

    let mut options = [None; MAX_SIDE_BOARD_OPTIONS];
    for (slot, option) in ordered.into_iter().take(MAX_SIDE_BOARD_OPTIONS).enumerate() {
        options[slot] = Some(option);
    }
    options
}

fn side_board_listing(kind: MapKind, quest: &QuestLog) -> String {
    side_board_task_options(kind, quest)
        .into_iter()
        .flatten()
        .map(|option| format!("{}[{}]", option.side.name(), option.status.label()))
        .collect::<Vec<_>>()
        .join("、")
}

fn side_board_summary(kind: MapKind, quest: &QuestLog) -> String {
    let sides = side_quests_for_map(kind);
    if sides.is_empty() {
        return "任务板 无".to_string();
    }

    let completed = side_board_completed_count(sides, quest);
    if completed == sides.len() {
        return format!(
            "任务板 已清：{completed}/{} | {}",
            sides.len(),
            side_board_listing(kind, quest)
        );
    }

    format!(
        "任务板 {completed}/{}完成 | {}",
        sides.len(),
        side_board_listing(kind, quest)
    )
}

fn side_quest_target_matches_map(side: SideQuest, kind: MapKind) -> bool {
    match side {
        SideQuest::VillageTrail => matches!(kind, MapKind::Village | MapKind::Bamboo),
        SideQuest::VillageHerbs => kind == MapKind::Village,
        SideQuest::MoonCaveCrystals => kind == MapKind::Cave,
        SideQuest::MoonCaveEchoes => matches!(kind, MapKind::Cave | MapKind::MoonEchoCorridor),
        SideQuest::RiverLanterns | SideQuest::RiverCargo => kind == MapKind::RiverReedBed,
        SideQuest::PlagueRelief => kind == MapKind::PlagueVillage,
        SideQuest::PlagueMedicine => {
            matches!(kind, MapKind::PlagueVillage | MapKind::PlagueShrinePath)
        }
        SideQuest::CapitalPatrol => matches!(kind, MapKind::Capital | MapKind::CapitalMansion),
        SideQuest::CapitalRumors => matches!(
            kind,
            MapKind::Capital | MapKind::CapitalMansion | MapKind::MansionMirrorGallery
        ),
        SideQuest::SouthernThunder => {
            matches!(kind, MapKind::SouthernRoad | MapKind::ThunderDrumPath)
        }
        SideQuest::SouthernDrums => kind == MapKind::ThunderDrumPath,
        SideQuest::FinalDreamEchoes | SideQuest::FinalHomewardVows => {
            matches!(kind, MapKind::FinalSanctum | MapKind::DreamWaterway)
        }
    }
}

fn side_quest_turn_in_matches_map(side: SideQuest, kind: MapKind) -> bool {
    side_quests_for_map(kind).contains(&side)
}

fn active_area_side_task_summary(kind: MapKind, quest: &QuestLog) -> String {
    let Some(side) = ALL_LOCAL_SIDE_QUESTS.iter().copied().find(|side| {
        quest.is_side_quest_active(*side)
            && !quest.is_side_quest_completed(*side)
            && side_quest_target_matches_map(*side, kind)
    }) else {
        return "当前委托区 无".to_string();
    };

    let progress = quest
        .side_quest_progress(side)
        .min(quest.side_quest_goal(side));
    let status = if progress >= quest.side_quest_goal(side) {
        "可交付"
    } else {
        "进行中"
    };

    format!(
        "当前委托区 {} [{}] {}/{} · {}\n现场：{}",
        quest.side_quest_name(side),
        status,
        progress,
        quest.side_quest_goal(side),
        quest.side_quest_route_hint(side),
        quest.side_task_field_status(side)
    )
}

fn local_task_intake_tracker(kind: MapKind, quest: &QuestLog) -> Option<String> {
    if quest.active_side_task_tracker().is_some() {
        return None;
    }

    let option = side_board_task_options(kind, quest)
        .into_iter()
        .flatten()
        .find(|option| {
            matches!(
                option.status,
                SideBoardTaskStatus::Available | SideBoardTaskStatus::Locked
            )
        })?;
    let side = option.side;

    match option.status {
        SideBoardTaskStatus::Available => Some(format!(
            "本地可领 · {} [可领取]\n签号：{} · {}\n{}\n第一步：{}\n现场：{}\n路线：{}\n领取：面对委托板/联系人按空格",
            side.name(),
            option.receipt_id,
            quest.side_quest_issuer(side),
            compact_hud_text(&quest.side_task_summary(side), 48),
            compact_hud_text(&quest.side_task_next_step(side), 48),
            compact_hud_text(&quest.side_task_field_status(side), 48),
            compact_hud_text(quest.side_quest_route_hint(side), 42)
        )),
        SideBoardTaskStatus::Locked => Some(format!(
            "本地后续 · {} [未开放]\n签号：{} · {}\n{}\n先清前置委托后可领取",
            side.name(),
            option.receipt_id,
            quest.side_quest_issuer(side),
            compact_hud_text(&quest.side_task_summary(side), 54)
        )),
        SideBoardTaskStatus::Ready
        | SideBoardTaskStatus::Active
        | SideBoardTaskStatus::Completed => None,
    }
}

fn main_task_target_matches_map(stage: QuestStage, kind: MapKind) -> bool {
    match stage {
        QuestStage::NotStarted
        | QuestStage::TalkToLinger
        | QuestStage::EscortMerchant
        | QuestStage::ReturnToSister
        | QuestStage::ReturnToLinger => kind == MapKind::Village,
        QuestStage::FindStarMage
        | QuestStage::DefeatMonsters { .. }
        | QuestStage::FindBambooScout => kind == MapKind::Bamboo,
        QuestStage::SeekCavePriestess | QuestStage::ConfrontMoonWraith => kind == MapKind::Cave,
        QuestStage::CaveTrial { .. } => kind == MapKind::MoonEchoCorridor,
        QuestStage::OpeningComplete
        | QuestStage::GatherRiverHerbs { .. }
        | QuestStage::ReturnToHerbHealer
        | QuestStage::FindRiverBoatman
        | QuestStage::ConfrontRiverDemon
        | QuestStage::RiverTownComplete => kind == MapKind::RiverTown,
        QuestStage::TuneRiverLanterns => kind == MapKind::RiverReedBed,
        QuestStage::SeekPlagueElder
        | QuestStage::SeekShrineKeeper
        | QuestStage::ReturnToShrineKeeper
        | QuestStage::ConfrontMiasmaRoot
        | QuestStage::PlagueVillageComplete => kind == MapKind::PlagueVillage,
        QuestStage::CleansePlagueShrines { .. } | QuestStage::SealPlagueWards => {
            kind == MapKind::PlagueShrinePath
        }
        QuestStage::SeekCapitalEnvoy | QuestStage::CapitalIntrigueComplete => {
            kind == MapKind::Capital
        }
        QuestStage::FindMansionSpy
        | QuestStage::ReturnToMansionSpy
        | QuestStage::ConfrontMirrorMinister => kind == MapKind::CapitalMansion,
        QuestStage::GatherSecretLetters { .. } | QuestStage::AlignMansionMirrors => {
            kind == MapKind::MansionMirrorGallery
        }
        QuestStage::SeekSpiritGuide
        | QuestStage::SeekTribalChief
        | QuestStage::ReturnToTribalChief
        | QuestStage::ConfrontThunderQilin
        | QuestStage::SouthernRoadComplete => kind == MapKind::SouthernRoad,
        QuestStage::CleanseSpiritTotems { .. } | QuestStage::AlignThunderDrums => {
            kind == MapKind::ThunderDrumPath
        }
        QuestStage::SeekFinalOracle
        | QuestStage::ReturnToFinalOracle
        | QuestStage::ConfrontDreamEclipse
        | QuestStage::FinaleComplete => kind == MapKind::FinalSanctum,
        QuestStage::LightFinalSoulLamps { .. } => kind == MapKind::DreamWaterway,
    }
}

fn main_task_route_guidance(kind: MapKind, quest: &QuestLog) -> String {
    let ledger = quest.main_task_ledger();
    if main_task_target_matches_map(quest.stage(), kind) {
        format!("主线 · 当前地图：{} · {}", ledger.action, ledger.contact)
    } else {
        format!("主线 · 前往：{} · {}", ledger.place, ledger.contact)
    }
}

fn active_side_task_for_guidance(quest: &QuestLog) -> Option<SideQuest> {
    ALL_LOCAL_SIDE_QUESTS
        .iter()
        .copied()
        .find(|side| quest.is_side_quest_active(*side) && !quest.is_side_quest_completed(*side))
}

fn side_task_route_guidance(kind: MapKind, quest: &QuestLog) -> Option<String> {
    let side = active_side_task_for_guidance(quest)?;
    let progress = quest
        .side_quest_progress(side)
        .min(quest.side_quest_goal(side));
    let goal = quest.side_quest_goal(side);

    if progress >= goal {
        let prefix = if side_quest_turn_in_matches_map(side, kind) {
            "可交付 当前地图"
        } else {
            "可交付 前往"
        };
        return Some(format!(
            "委托 · {prefix}：{}",
            quest.side_quest_turn_in_place(side)
        ));
    }

    if side_quest_target_matches_map(side, kind) {
        Some(format!(
            "委托 · 目标区 当前地图：{} {progress}/{goal}",
            quest.side_quest_name(side)
        ))
    } else {
        Some(format!(
            "委托 · 前往目标区：{} · {progress}/{goal}",
            compact_hud_text(quest.side_quest_route_hint(side), 48)
        ))
    }
}

fn active_npc_errand_for_guidance(quest: &QuestLog) -> Option<NpcErrand> {
    TRACKED_NPC_ERRANDS
        .iter()
        .copied()
        .find(|errand| quest.is_npc_errand_active(*errand))
}

fn npc_errand_delivery_matches_map(errand: NpcErrand, kind: MapKind) -> bool {
    npc_defs(kind)
        .iter()
        .any(|npc| npc_errand_delivery_for(kind, npc) == Some(errand))
}

fn npc_errand_route_guidance(kind: MapKind, quest: &QuestLog) -> Option<String> {
    let errand = active_npc_errand_for_guidance(quest)?;
    if npc_errand_delivery_matches_map(errand, kind) {
        Some(format!(
            "托付 · 交付地 当前地图：{}",
            quest.npc_errand_receiver(errand)
        ))
    } else {
        Some(format!(
            "托付 · 前往交付：{} · {}",
            quest.npc_errand_turn_in_place(errand),
            compact_hud_text(quest.npc_errand_route_hint(errand), 44)
        ))
    }
}

const TRACKED_COMPANION_REVISITS: [CompanionRevisit; 3] = [
    CompanionRevisit::TrailEcho,
    CompanionRevisit::MirrorTrace,
    CompanionRevisit::TotemVow,
];

fn active_companion_revisit_for_guidance(quest: &QuestLog) -> Option<CompanionRevisit> {
    TRACKED_COMPANION_REVISITS
        .iter()
        .copied()
        .find(|revisit| quest.companion_revisit_ready(*revisit))
        .or_else(|| {
            TRACKED_COMPANION_REVISITS
                .iter()
                .copied()
                .find(|revisit| quest.companion_revisit_active(*revisit))
        })
}

fn companion_revisit_route_guidance(kind: MapKind, quest: &QuestLog) -> Option<String> {
    let revisit = active_companion_revisit_for_guidance(quest)?;
    if quest.companion_revisit_ready(revisit) {
        if kind == companion_revisit_giver_map(revisit) {
            Some(format!("补访 · 当前地图交付：{}", revisit.issuer()))
        } else {
            Some(format!("补访 · 返回交付：{}", revisit.turn_in_place()))
        }
    } else if kind == companion_revisit_target_map(revisit) {
        Some(format!(
            "补访 · 当前地图寻访：{} · {}",
            revisit.name(),
            revisit.target_mark().name()
        ))
    } else {
        Some(format!(
            "补访 · 前往{}：{}",
            companion_revisit_target_map(revisit).def().name,
            revisit.target_mark().name()
        ))
    }
}

fn task_route_guidance(kind: MapKind, quest: &QuestLog) -> String {
    let mut lines = vec![
        "任务引路".to_string(),
        main_task_route_guidance(kind, quest),
    ];
    if let Some(side) = side_task_route_guidance(kind, quest) {
        lines.push(side);
    }
    if let Some(errand) = npc_errand_route_guidance(kind, quest) {
        lines.push(errand);
    }
    if let Some(revisit) = companion_revisit_route_guidance(kind, quest) {
        lines.push(revisit);
    }
    lines.join("\n")
}

fn task_tracker_text(kind: MapKind, quest: &QuestLog) -> String {
    let mut tracker = quest.active_task_tracker();
    tracker.push_str("\n\n");
    tracker.push_str(&task_route_guidance(kind, quest));
    if let Some(local) = local_task_intake_tracker(kind, quest) {
        tracker.push_str("\n\n");
        tracker.push_str(&local);
    }
    if quest.active_npc_errand_tracker().is_none() {
        if let Some(local_errand) = local_npc_errand_intake_tracker(kind, quest) {
            tracker.push_str("\n\n");
            tracker.push_str(&local_errand);
        }
    }
    if quest.active_companion_revisit_tracker().is_none() {
        if let Some(local_revisit) = local_companion_revisit_intake_tracker(kind, quest) {
            tracker.push_str("\n\n");
            tracker.push_str(&local_revisit);
        }
    }
    tracker
}

fn side_board_prompt(kind: MapKind, quest: &QuestLog) -> Option<String> {
    let sides = side_quests_for_map(kind);
    if sides.is_empty() {
        return None;
    }

    let completed = side_board_completed_count(sides, quest);
    if completed == sides.len() {
        return Some(format!(
            "委托板 已清 | 空格查看\n{}",
            side_board_listing(kind, quest)
        ));
    }

    let focus = side_board_task_options(kind, quest)
        .into_iter()
        .flatten()
        .next()
        .expect("non-empty side quest board");
    Some(format!(
        "委托板 {completed}/{}完成 | 空格打开\n优先：{}[{}] · {}\n{}",
        sides.len(),
        focus.side.name(),
        focus.status.label(),
        quest.side_task_action_label(focus.side),
        side_board_listing(kind, quest)
    ))
}

fn side_board_overview_lines(kind: MapKind, quest: &QuestLog) -> Vec<String> {
    let sides = side_quests_for_map(kind);
    if sides.is_empty() {
        return vec!["【委托板】这里没有可领取的本地委托。".to_string()];
    }

    let completed = side_board_completed_count(sides, quest);
    let mut lines = vec![format!(
        "【委托板】{} 本地委托 {completed}/{} 已完成。",
        kind.def().name,
        sides.len()
    )];
    for (idx, option) in side_board_task_options(kind, quest)
        .into_iter()
        .flatten()
        .enumerate()
    {
        let progress = if quest.is_side_quest_completed(option.side) {
            quest.side_quest_goal(option.side)
        } else {
            quest
                .side_quest_progress(option.side)
                .min(quest.side_quest_goal(option.side))
        };
        lines.push(format!(
            "{}. {}",
            idx + 1,
            quest.side_task_board_card(option.side)
        ));
        lines.push(format!(
            "【选择提示】{} · 当前进度 {}/{} · 领取和交付都会再次确认。",
            quest.side_task_board_option_label(option.side),
            progress,
            quest.side_quest_goal(option.side)
        ));
    }
    lines.push(
        "选择一份委托契约查看详情；领取后会写入任务簿、显示签收回执，并进入 HUD 追踪。".to_string(),
    );
    lines
}

fn side_board_marker_for(quest: &QuestLog, kind: MapKind) -> &'static str {
    let sides = side_quests_for_map(kind);
    if sides.is_empty() {
        return "?";
    }

    if sides.iter().any(|side| {
        quest.is_side_quest_active(*side)
            && quest.side_quest_progress(*side) >= quest.side_quest_goal(*side)
    }) {
        "!"
    } else if sides
        .iter()
        .any(|side| quest.is_side_quest_active(*side) && !quest.is_side_quest_completed(*side))
    {
        "*"
    } else if sides
        .iter()
        .any(|side| !quest.is_side_quest_completed(*side))
    {
        "!"
    } else {
        "✓"
    }
}

fn final_lamp_for_prop(kind: MapKind, prop: &PropDef) -> Option<FinalLamp> {
    if kind != MapKind::DreamWaterway || prop.path != "props/ai_spirit_lantern.png" {
        return None;
    }

    match (prop.col, prop.row) {
        (8, 12) => Some(FinalLamp::Memory),
        (11, 5) => Some(FinalLamp::Vow),
        (24, 13) => Some(FinalLamp::Fate),
        _ => None,
    }
}

fn river_lantern_for_prop(kind: MapKind, prop: &PropDef) -> Option<RiverLantern> {
    if kind != MapKind::RiverReedBed || prop.path != "props/ai_spirit_lantern.png" {
        return None;
    }

    match (prop.col, prop.row) {
        (9, 2) => Some(RiverLantern::Upstream),
        (16, 6) => Some(RiverLantern::Midstream),
        (24, 10) => Some(RiverLantern::Dock),
        _ => None,
    }
}

fn plague_ward_for_prop(kind: MapKind, prop: &PropDef) -> Option<PlagueWard> {
    if kind != MapKind::PlagueShrinePath {
        return None;
    }

    match (prop.path, prop.col, prop.row) {
        ("props/ai_shrine_statue.png", 12, 5) => Some(PlagueWard::OldShrine),
        ("props/ai_cave_crystal.png", 18, 8) => Some(PlagueWard::BitterWell),
        ("props/ai_spirit_lantern.png", 23, 11) => Some(PlagueWard::Sickroom),
        _ => None,
    }
}

fn mansion_mirror_for_prop(kind: MapKind, prop: &PropDef) -> Option<MansionMirrorNode> {
    if kind != MapKind::MansionMirrorGallery || prop.path != "props/ai_cave_crystal.png" {
        return None;
    }

    match (prop.col, prop.row) {
        (12, 6) => Some(MansionMirrorNode::Ledger),
        (24, 11) => Some(MansionMirrorNode::Witness),
        _ => None,
    }
}

fn thunder_drum_for_prop(kind: MapKind, prop: &PropDef) -> Option<ThunderDrum> {
    if kind != MapKind::ThunderDrumPath {
        return None;
    }

    match (prop.path, prop.col, prop.row) {
        ("props/ai_shrine_statue.png", 12, 5) => Some(ThunderDrum::Wind),
        ("props/ai_cave_crystal.png", 24, 11) => Some(ThunderDrum::Cloud),
        ("props/ai_spirit_lantern.png", 10, 13) => Some(ThunderDrum::Oath),
        _ => None,
    }
}

fn moon_crystal_for_prop(kind: MapKind, prop: &PropDef) -> Option<MoonCrystal> {
    if kind != MapKind::MoonEchoCorridor || prop.path != "props/ai_cave_crystal.png" {
        return None;
    }

    match (prop.col, prop.row) {
        (12, 3) => Some(MoonCrystal::North),
        (20, 11) => Some(MoonCrystal::South),
        _ => None,
    }
}

fn route_mark_for_prop(kind: MapKind, prop: &PropDef) -> Option<RouteMark> {
    match (kind, prop.path, prop.col, prop.row) {
        (MapKind::MoonEchoCorridor, "props/ai_bamboo_gate.png", 27, 1) => Some(RouteMark::MoonEcho),
        (MapKind::RiverReedBed, "props/ai_bamboo_gate.png", 26, 1) => Some(RouteMark::ReedFord),
        (MapKind::PlagueShrinePath, "props/ai_cave_crystal.png", 10, 13) => {
            Some(RouteMark::PlagueBell)
        }
        (MapKind::MansionMirrorGallery, "props/ai_bamboo_gate.png", 26, 1) => {
            Some(RouteMark::MirrorSideDoor)
        }
        (MapKind::ThunderDrumPath, "props/ai_cave_crystal.png", 18, 6) => {
            Some(RouteMark::ThunderSwitchback)
        }
        (MapKind::DreamWaterway, "props/ai_cave_crystal.png", 18, 10) => {
            Some(RouteMark::DreamReturn)
        }
        _ => None,
    }
}

fn route_mark_for_map(kind: MapKind) -> Option<RouteMark> {
    match kind {
        MapKind::MoonEchoCorridor => Some(RouteMark::MoonEcho),
        MapKind::RiverReedBed => Some(RouteMark::ReedFord),
        MapKind::PlagueShrinePath => Some(RouteMark::PlagueBell),
        MapKind::MansionMirrorGallery => Some(RouteMark::MirrorSideDoor),
        MapKind::ThunderDrumPath => Some(RouteMark::ThunderSwitchback),
        MapKind::DreamWaterway => Some(RouteMark::DreamReturn),
        MapKind::Village
        | MapKind::Bamboo
        | MapKind::Cave
        | MapKind::RiverTown
        | MapKind::PlagueVillage
        | MapKind::Capital
        | MapKind::CapitalMansion
        | MapKind::SouthernRoad
        | MapKind::FinalSanctum => None,
    }
}

fn route_detour_for_prop(kind: MapKind, prop: &PropDef) -> Option<RouteDetour> {
    match (kind, prop.path, prop.col, prop.row) {
        (MapKind::MoonEchoCorridor, "props/ai_cave_crystal.png", 18, 6) => {
            Some(RouteDetour::MoonEchoPool)
        }
        (MapKind::RiverReedBed, "props/ai_cave_crystal.png", 13, 10) => {
            Some(RouteDetour::ReedHiddenFord)
        }
        (MapKind::PlagueShrinePath, "props/ai_bamboo_gate.png", 26, 1) => {
            Some(RouteDetour::PlagueHerbTrail)
        }
        (MapKind::MansionMirrorGallery, "props/ai_bamboo_gate.png", 6, 13) => {
            Some(RouteDetour::MirrorServantDoor)
        }
        (MapKind::ThunderDrumPath, "props/ai_bamboo_gate.png", 26, 1) => {
            Some(RouteDetour::ThunderRidgeCache)
        }
        (MapKind::DreamWaterway, "props/ai_bamboo_gate.png", 26, 1) => {
            Some(RouteDetour::DreamBackwater)
        }
        _ => None,
    }
}

fn route_detour_for_map(kind: MapKind) -> Option<RouteDetour> {
    match kind {
        MapKind::MoonEchoCorridor => Some(RouteDetour::MoonEchoPool),
        MapKind::RiverReedBed => Some(RouteDetour::ReedHiddenFord),
        MapKind::PlagueShrinePath => Some(RouteDetour::PlagueHerbTrail),
        MapKind::MansionMirrorGallery => Some(RouteDetour::MirrorServantDoor),
        MapKind::ThunderDrumPath => Some(RouteDetour::ThunderRidgeCache),
        MapKind::DreamWaterway => Some(RouteDetour::DreamBackwater),
        MapKind::Village
        | MapKind::Bamboo
        | MapKind::Cave
        | MapKind::RiverTown
        | MapKind::PlagueVillage
        | MapKind::Capital
        | MapKind::CapitalMansion
        | MapKind::SouthernRoad
        | MapKind::FinalSanctum => None,
    }
}

fn treasure_for_prop(kind: MapKind, prop: &PropDef) -> Option<TreasureCache> {
    match (kind, prop.path, prop.col, prop.row) {
        (MapKind::Village, "props/ai_shrine_statue.png", 20, 4) => {
            Some(TreasureCache::VillageShrine)
        }
        (MapKind::Bamboo, "props/ai_shrine_statue.png", 12, 6) => {
            Some(TreasureCache::BambooOffering)
        }
        (MapKind::Cave, "props/ai_shrine_statue.png", 9, 7) => Some(TreasureCache::CaveOffering),
        (MapKind::RiverTown, "props/ai_cave_crystal.png", 26, 7) => {
            Some(TreasureCache::RiverTownCrystal)
        }
        (MapKind::RiverReedBed, "props/ai_cave_crystal.png", 18, 6) => {
            Some(TreasureCache::RiverReedCrystal)
        }
        (MapKind::PlagueShrinePath, "props/ai_shrine_statue.png", 12, 5) => {
            Some(TreasureCache::PlagueShrine)
        }
        (MapKind::Capital, "props/ai_shrine_statue.png", 12, 6) => {
            Some(TreasureCache::CapitalShrine)
        }
        (MapKind::CapitalMansion, "props/ai_cave_crystal.png", 24, 11) => {
            Some(TreasureCache::MansionMirror)
        }
        (MapKind::SouthernRoad, "props/ai_shrine_statue.png", 12, 5) => {
            Some(TreasureCache::SouthernTotem)
        }
        (MapKind::FinalSanctum, "props/ai_cave_crystal.png", 26, 1) => {
            Some(TreasureCache::FinalMemoryCache)
        }
        _ => None,
    }
}

fn shrine_blessing_for_prop(kind: MapKind, prop: &PropDef) -> Option<ShrineBlessing> {
    if prop.path != "props/ai_shrine_statue.png" {
        return None;
    }

    match (kind, prop.col, prop.row) {
        (MapKind::Village, 20, 4)
        | (MapKind::PlagueShrinePath, 12, 5)
        | (MapKind::FinalSanctum, 12, 6) => Some(ShrineBlessing::Guard),
        (MapKind::Bamboo, 12, 6)
        | (MapKind::CapitalMansion, 20, 3)
        | (MapKind::MansionMirrorGallery, 20, 3)
        | (MapKind::SouthernRoad, 12, 5) => Some(ShrineBlessing::Sword),
        (MapKind::Cave, 9, 7)
        | (MapKind::RiverTown, 5, 11)
        | (MapKind::RiverReedBed, 22, 11)
        | (MapKind::Capital, 12, 6) => Some(ShrineBlessing::Spirit),
        _ => None,
    }
}

fn prop_dialogue(prop: &PropDef) -> Vec<String> {
    let line = match prop.path {
        "props/ai_bamboo_gate.png" => "竹门上挂着旧铃，风过时会亮起一道青光。",
        "props/ai_cave_crystal.png" => "晶簇里映出细碎妖气，像有人刚从这里经过。",
        "props/ai_shrine_statue.png" => "旧像前压着香灰，似乎能听见远处的祷声。",
        "props/ai_spirit_lantern.png" => "灵灯火苗轻轻摇晃，照出脚下尚未探索的路。",
        _ => "这里贴满了旧告示。",
    };
    vec![line.to_string()]
}

fn append_side_objective_progress(
    lines: &mut Vec<String>,
    quest: &mut QuestLog,
    side: SideQuest,
    source: &str,
    was_done: bool,
    is_done: bool,
) {
    if was_done || !is_done {
        return;
    }

    if let Some(line) = quest.record_side_objective(side, source) {
        lines.push(line);
    }
}

fn route_mark_side_objective(mark: RouteMark) -> Option<(SideQuest, &'static str)> {
    match mark {
        RouteMark::MoonEcho => Some((SideQuest::MoonCaveEchoes, "回声路印已记")),
        RouteMark::ReedFord => Some((SideQuest::RiverCargo, "湿货浅渡已标")),
        RouteMark::PlagueBell => Some((SideQuest::PlagueMedicine, "药路旧铃已记")),
        RouteMark::MirrorSideDoor => Some((SideQuest::CapitalRumors, "暗帖偏门已标")),
        RouteMark::ThunderSwitchback => Some((SideQuest::SouthernDrums, "旧鼓回坡已记")),
        RouteMark::DreamReturn => Some((SideQuest::FinalHomewardVows, "归潮水线已记")),
    }
}

fn route_detour_side_objective(detour: RouteDetour) -> Option<(SideQuest, &'static str)> {
    match detour {
        RouteDetour::MoonEchoPool => Some((SideQuest::MoonCaveEchoes, "回声岔路已压")),
        RouteDetour::ReedHiddenFord => Some((SideQuest::RiverCargo, "湿货隐渡已查")),
        RouteDetour::PlagueHerbTrail => Some((SideQuest::PlagueMedicine, "病屋药径已护")),
        RouteDetour::MirrorServantDoor => Some((SideQuest::CapitalRumors, "暗帖斜廊已截")),
        RouteDetour::ThunderRidgeCache => Some((SideQuest::SouthernDrums, "战鼓雷脊已安")),
        RouteDetour::DreamBackwater => Some((SideQuest::FinalHomewardVows, "归潮回湾已护")),
    }
}

fn is_bond_lantern(prop: &PropDef) -> bool {
    prop.path == "props/ai_spirit_lantern.png"
}

fn apply_side_quest_reward(stats: &mut PlayerStats, reward: SideQuestReward) -> String {
    let levels = stats.gain_exp(reward.exp);
    stats.potions += reward.potions;
    stats.gold += reward.gold;
    if levels > 0 {
        format!(
            "【奖励】经验 +{}，药水 +{}，钱 +{}，境界提升至 Lv.{}。",
            reward.exp, reward.potions, reward.gold, stats.level
        )
    } else {
        format!(
            "【奖励】经验 +{}，药水 +{}，钱 +{}。",
            reward.exp, reward.potions, reward.gold
        )
    }
}

fn apply_npc_errand_reward(stats: &mut PlayerStats, reward: NpcErrandReward) -> String {
    let levels = stats.gain_exp(reward.exp);
    stats.potions += reward.potions;
    stats.gold += reward.gold;
    stats.hp = (stats.hp + reward.hp).clamp(0, stats.max_hp);
    stats.mp = (stats.mp + reward.mp).clamp(0, stats.max_mp);

    let mut gains = vec![format!("经验 +{}", reward.exp)];
    if reward.potions > 0 {
        gains.push(format!("药水 +{}", reward.potions));
    }
    if reward.gold > 0 {
        gains.push(format!("钱 +{}", reward.gold));
    }
    if reward.hp > 0 {
        gains.push(format!("气血 +{}", reward.hp));
    }
    if reward.mp > 0 {
        gains.push(format!("灵力 +{}", reward.mp));
    }
    if levels > 0 {
        gains.push(format!("境界提升至 Lv.{}", stats.level));
    }

    format!("【托付回礼】{}。", gains.join("，"))
}

#[derive(Clone, Copy)]
struct SideQuestAdvanceSupply {
    potions: u32,
    gold: u32,
    mp: i32,
}

fn side_quest_advance_supply(side: SideQuest) -> SideQuestAdvanceSupply {
    match side {
        SideQuest::VillageTrail | SideQuest::VillageHerbs => SideQuestAdvanceSupply {
            potions: 1,
            gold: 4,
            mp: 2,
        },
        SideQuest::MoonCaveCrystals | SideQuest::MoonCaveEchoes => SideQuestAdvanceSupply {
            potions: 1,
            gold: 6,
            mp: 4,
        },
        SideQuest::RiverLanterns | SideQuest::RiverCargo => SideQuestAdvanceSupply {
            potions: 1,
            gold: 8,
            mp: 4,
        },
        SideQuest::PlagueRelief | SideQuest::PlagueMedicine => SideQuestAdvanceSupply {
            potions: 2,
            gold: 8,
            mp: 6,
        },
        SideQuest::CapitalPatrol | SideQuest::CapitalRumors => SideQuestAdvanceSupply {
            potions: 1,
            gold: 12,
            mp: 6,
        },
        SideQuest::SouthernThunder | SideQuest::SouthernDrums => SideQuestAdvanceSupply {
            potions: 2,
            gold: 12,
            mp: 8,
        },
        SideQuest::FinalDreamEchoes | SideQuest::FinalHomewardVows => SideQuestAdvanceSupply {
            potions: 2,
            gold: 16,
            mp: 10,
        },
    }
}

fn apply_side_quest_advance_supply(
    stats: &mut PlayerStats,
    quest: &QuestLog,
    side: SideQuest,
) -> String {
    let supply = side_quest_advance_supply(side);
    let mp_before = stats.mp;
    stats.potions += supply.potions;
    stats.gold += supply.gold;
    stats.mp = (stats.mp + supply.mp).clamp(0, stats.max_mp);
    let restored_mp = (stats.mp - mp_before).max(0);

    let mut gains = Vec::new();
    if supply.potions > 0 {
        gains.push(format!("药水 +{}", supply.potions));
    }
    if supply.gold > 0 {
        gains.push(format!("路费 +{}文", supply.gold));
    }
    if restored_mp > 0 {
        gains.push(format!("灵力 +{}", restored_mp));
    }

    format!(
        "【委托预支】{}先给了{}，签下《{}》后即可上路。当前药水 x{}，钱 {}文，灵力 {}/{}。",
        quest.side_quest_issuer(side),
        gains.join("、"),
        quest.side_quest_name(side),
        stats.potions,
        stats.gold,
        stats.mp,
        stats.max_mp
    )
}

fn side_quest_choice_for(quest: &QuestLog, side: SideQuest) -> Option<DialogueChoice> {
    if quest.is_side_quest_completed(side) {
        return None;
    }

    if !quest.is_side_quest_active(side) && !quest.is_side_quest_unlocked(side) {
        return None;
    }

    if quest.is_side_quest_active(side) {
        if quest.side_quest_progress(side) >= quest.side_quest_goal(side) {
            Some(DialogueChoice::side_quest(
                side,
                SideQuestChoiceAction::TurnIn,
            ))
        } else {
            None
        }
    } else {
        Some(DialogueChoice::side_quest(
            side,
            SideQuestChoiceAction::Accept,
        ))
    }
}

fn main_quest_choice_for(quest: &QuestLog, role: QuestRole) -> Option<DialogueChoice> {
    main_quest_action_for(quest.stage(), role)
        .map(|action| DialogueChoice::main_quest(role, action))
}

fn main_quest_action_for(stage: QuestStage, role: QuestRole) -> Option<MainQuestChoiceAction> {
    match (role, stage) {
        (QuestRole::SwordSister, QuestStage::NotStarted)
        | (QuestRole::HerbHealer, QuestStage::OpeningComplete)
        | (QuestRole::PlagueElder, QuestStage::RiverTownComplete | QuestStage::SeekPlagueElder)
        | (
            QuestRole::CapitalEnvoy,
            QuestStage::PlagueVillageComplete | QuestStage::SeekCapitalEnvoy,
        )
        | (
            QuestRole::SpiritGuide,
            QuestStage::CapitalIntrigueComplete | QuestStage::SeekSpiritGuide,
        )
        | (
            QuestRole::FinalOracle,
            QuestStage::SouthernRoadComplete | QuestStage::SeekFinalOracle,
        ) => Some(MainQuestChoiceAction::Accept),
        (QuestRole::SwordSister, QuestStage::ReturnToSister)
        | (QuestRole::Linger, QuestStage::ReturnToLinger)
        | (QuestRole::HerbHealer, QuestStage::ReturnToHerbHealer) => {
            Some(MainQuestChoiceAction::TurnIn)
        }
        (QuestRole::Linger, QuestStage::TalkToLinger)
        | (QuestRole::StarMage, QuestStage::FindStarMage)
        | (QuestRole::Merchant, QuestStage::EscortMerchant)
        | (QuestRole::BambooScout, QuestStage::FindBambooScout)
        | (QuestRole::CavePriestess, QuestStage::SeekCavePriestess)
        | (QuestRole::RiverBoatman, QuestStage::FindRiverBoatman)
        | (QuestRole::ShrineKeeper, QuestStage::SeekShrineKeeper)
        | (QuestRole::MansionSpy, QuestStage::FindMansionSpy)
        | (QuestRole::TribalChief, QuestStage::SeekTribalChief) => {
            Some(MainQuestChoiceAction::Advance)
        }
        (QuestRole::CavePriestess, QuestStage::ConfrontMoonWraith)
        | (QuestRole::RiverBoatman, QuestStage::ConfrontRiverDemon)
        | (QuestRole::ShrineKeeper, QuestStage::ReturnToShrineKeeper)
        | (QuestRole::MansionSpy, QuestStage::ReturnToMansionSpy)
        | (QuestRole::TribalChief, QuestStage::ReturnToTribalChief)
        | (QuestRole::FinalOracle, QuestStage::ReturnToFinalOracle) => {
            Some(MainQuestChoiceAction::Boss)
        }
        _ => None,
    }
}

fn main_quest_preview(
    quest: &QuestLog,
    role: QuestRole,
    action: MainQuestChoiceAction,
) -> Vec<String> {
    let ledger = quest.main_task_ledger();
    vec![
        format!(
            "【主线任务】当前委托\n状态：{}\n操作：{}\n主线签：{}\n章程：{}/{} {}\n地点：{}\n联络：{}\n委托人：{}\n目标：{}",
            action.status(),
            action.operation(),
            ledger.receipt,
            ledger.step,
            ledger.total,
            ledger.action,
            ledger.place,
            ledger.contact,
            role.name(),
            quest.objective()
        ),
        quest.main_task_summary(),
    ]
}

fn main_quest_battle_after(role: QuestRole, stage_before: QuestStage) -> DialogueAfter {
    match (role, stage_before) {
        (QuestRole::CavePriestess, QuestStage::ConfrontMoonWraith) => {
            DialogueAfter::StartBattle(PendingEncounter {
                zone: EncounterZone::Cave,
                kind: EncounterKind::Boss(BossKind::MoonWraith),
            })
        }
        (QuestRole::RiverBoatman, QuestStage::ConfrontRiverDemon) => {
            DialogueAfter::StartBattle(PendingEncounter {
                zone: EncounterZone::RiverTown,
                kind: EncounterKind::Boss(BossKind::RiverDemon),
            })
        }
        (QuestRole::ShrineKeeper, QuestStage::ReturnToShrineKeeper) => {
            DialogueAfter::StartBattle(PendingEncounter {
                zone: EncounterZone::PlagueVillage,
                kind: EncounterKind::Boss(BossKind::MiasmaRoot),
            })
        }
        (QuestRole::MansionSpy, QuestStage::ReturnToMansionSpy) => {
            DialogueAfter::StartBattle(PendingEncounter {
                zone: EncounterZone::Capital,
                kind: EncounterKind::Boss(BossKind::MirrorMinister),
            })
        }
        (QuestRole::TribalChief, QuestStage::ReturnToTribalChief) => {
            DialogueAfter::StartBattle(PendingEncounter {
                zone: EncounterZone::SouthernRoad,
                kind: EncounterKind::Boss(BossKind::ThunderQilin),
            })
        }
        (QuestRole::FinalOracle, QuestStage::ReturnToFinalOracle) => {
            DialogueAfter::StartBattle(PendingEncounter {
                zone: EncounterZone::FinalSanctum,
                kind: EncounterKind::Boss(BossKind::DreamEclipse),
            })
        }
        _ => DialogueAfter::None,
    }
}

struct DialogueChoiceResult {
    lines: Vec<String>,
    after: DialogueAfter,
    choice: Option<DialogueChoice>,
    chapter_art: Option<ChapterArtAssets>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct ChapterArtAssets {
    still: &'static str,
    sheet: &'static str,
}

fn chapter_art_assets(chapter: Chapter) -> ChapterArtAssets {
    match chapter {
        Chapter::VillageOath => ChapterArtAssets {
            still: "ui/chapter1_art.png",
            sheet: "ui/anim/chapter1_sheet.png",
        },
        Chapter::MoonCave => ChapterArtAssets {
            still: "ui/chapter2_art.png",
            sheet: "ui/anim/chapter2_sheet.png",
        },
        Chapter::RiverMedicine => ChapterArtAssets {
            still: "ui/chapter3_art.png",
            sheet: "ui/anim/chapter3_sheet.png",
        },
        Chapter::PlagueRain => ChapterArtAssets {
            still: "ui/chapter4_art.png",
            sheet: "ui/anim/chapter4_sheet.png",
        },
        Chapter::CapitalMirror => ChapterArtAssets {
            still: "ui/chapter5_art.png",
            sheet: "ui/anim/chapter5_sheet.png",
        },
        Chapter::SouthernThunder => ChapterArtAssets {
            still: "ui/chapter6_art.png",
            sheet: "ui/anim/chapter6_sheet.png",
        },
        Chapter::FinalDream => ChapterArtAssets {
            still: "ui/chapter7_art.png",
            sheet: "ui/anim/chapter7_sheet.png",
        },
    }
}

fn append_chapter_card(quest: &mut QuestLog, lines: &mut Vec<String>) -> Option<ChapterArtAssets> {
    let chapter = quest.current_chapter();
    quest.take_chapter_card().map(|chapter_card| {
        lines.extend(chapter_card);
        chapter_art_assets(chapter)
    })
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
enum QuestNoticeKind {
    #[default]
    Main,
    Side,
    Complete,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct QuestNoticeMessage {
    kind: QuestNoticeKind,
    title: String,
    body: String,
}

impl QuestNoticeMessage {
    fn new(kind: QuestNoticeKind, title: impl Into<String>, body: impl Into<String>) -> Self {
        Self {
            kind,
            title: title.into(),
            body: body.into(),
        }
    }
}

#[derive(Resource, Default)]
struct QuestNotice {
    active: bool,
    age: f32,
    duration: f32,
    kind: QuestNoticeKind,
    title: String,
    body: String,
}

impl QuestNotice {
    fn show(&mut self, message: QuestNoticeMessage) {
        self.active = true;
        self.age = 0.0;
        self.duration = 3.6;
        self.kind = message.kind;
        self.title = message.title;
        self.body = message.body;
    }
}

fn quest_notice_for_confirmed_choice(
    choice: DialogueChoice,
    quest: &QuestLog,
) -> Option<QuestNoticeMessage> {
    if !choice.accepted() {
        return None;
    }

    match choice.kind {
        DialogueChoiceKind::MainQuest { role, action } => match action {
            MainQuestChoiceAction::Accept => {
                let ledger = quest.main_task_ledger();
                Some(QuestNoticeMessage::new(
                    QuestNoticeKind::Main,
                    "任务簿更新 · 主线已接取",
                    format!(
                        "{}托付：{}\n{} · {}/{} {}\n已加入主线追踪卡",
                        role.name(),
                        quest.objective(),
                        ledger.receipt,
                        ledger.step,
                        ledger.total,
                        ledger.action
                    ),
                ))
            }
            MainQuestChoiceAction::Advance => {
                let ledger = quest.main_task_ledger();
                Some(QuestNoticeMessage::new(
                    QuestNoticeKind::Main,
                    "任务簿更新 · 主线推进",
                    format!(
                        "下一步：{}\n{} · {}/{} {} · 主线追踪已更新",
                        quest.objective(),
                        ledger.receipt,
                        ledger.step,
                        ledger.total,
                        ledger.action
                    ),
                ))
            }
            MainQuestChoiceAction::TurnIn => {
                let ledger = quest.main_task_ledger();
                Some(QuestNoticeMessage::new(
                    QuestNoticeKind::Complete,
                    "任务簿更新 · 主线已交付",
                    format!(
                        "{}线索已结，下一步：{}\n{} · {}/{} {} · 主线追踪已更新",
                        role.name(),
                        quest.objective(),
                        ledger.receipt,
                        ledger.step,
                        ledger.total,
                        ledger.action
                    ),
                ))
            }
            MainQuestChoiceAction::Boss => None,
        },
        DialogueChoiceKind::SideQuest { side, action } => match action {
            SideQuestChoiceAction::Accept => Some(QuestNoticeMessage::new(
                QuestNoticeKind::Side,
                "任务簿更新 · 委托契约已领取",
                format!(
                    "《{}》 {}/{} · 委托签 {}\n下一步：{}\n现场：{}\n{} · 回委托点交付领奖\nHUD 委托追踪卡已更新",
                    quest.side_quest_name(side),
                    quest
                        .side_quest_progress(side)
                        .min(quest.side_quest_goal(side)),
                    quest.side_quest_goal(side),
                    quest.side_quest_receipt_id(side),
                    quest.side_task_next_step(side),
                    quest.side_task_field_status(side),
                    compact_hud_text(quest.side_quest_route_hint(side), 48)
                ),
            )),
            SideQuestChoiceAction::TurnIn => {
                let resolution = choice.selected_side_quest_resolution();
                Some(QuestNoticeMessage::new(
                    QuestNoticeKind::Complete,
                    "任务簿更新 · 委托契约已交付",
                    format!(
                        "《{}》完成 · {} · 委托签归档\n报酬已入袋，追踪卡已移除",
                        quest.side_quest_name(side),
                        resolution.name()
                    ),
                ))
            }
        },
        DialogueChoiceKind::NpcErrand { errand, action } => match action {
            NpcErrandChoiceAction::Accept => Some(QuestNoticeMessage::new(
                QuestNoticeKind::Side,
                "任务簿更新 · NPC托付已接下",
                format!(
                    "《{}》 · {}\n{} -> {}\n目标：{}\n路线：{}\n交付：{}\n已加入 HUD 托付追踪卡",
                    errand.name(),
                    quest.npc_errand_receipt_id(errand),
                    quest.npc_errand_issuer(errand),
                    quest.npc_errand_receiver(errand),
                    quest.npc_errand_objective(errand),
                    compact_hud_text(quest.npc_errand_route_hint(errand), 48),
                    quest.npc_errand_turn_in_place(errand)
                ),
            )),
            NpcErrandChoiceAction::TurnIn => Some(QuestNoticeMessage::new(
                QuestNoticeKind::Complete,
                "任务簿更新 · NPC托付已送达",
                format!(
                    "《{}》完成 · {}\n回礼已入袋，托付追踪已移除",
                    errand.name(),
                    quest.npc_errand_turn_in_place(errand)
                ),
            )),
        },
        DialogueChoiceKind::CompanionRevisit { revisit, action } => match action {
            CompanionRevisitChoiceAction::Accept => Some(QuestNoticeMessage::new(
                QuestNoticeKind::Side,
                "任务簿更新 · 同伴补访已领取",
                format!(
                    "《{}》 · {}\n补访对象：{}\n现场：{}\n路线：{}\n交付：{}\n已加入 HUD 补访追踪卡",
                    revisit.name(),
                    revisit.receipt_id(),
                    revisit.scene().speaker(),
                    revisit.target_place(),
                    compact_hud_text(revisit.route_hint(), 48),
                    revisit.turn_in_place()
                ),
            )),
            CompanionRevisitChoiceAction::TurnIn => Some(QuestNoticeMessage::new(
                QuestNoticeKind::Complete,
                "任务簿更新 · 同伴补访已归档",
                format!(
                    "《{}》完成 · {}\n迟来小传已补回，回礼已入袋",
                    revisit.name(),
                    revisit.receipt_id()
                ),
            )),
        },
        DialogueChoiceKind::BondResponse
        | DialogueChoiceKind::CampTactic
        | DialogueChoiceKind::RouteDetour { .. }
        | DialogueChoiceKind::CommissionTrace { .. }
        | DialogueChoiceKind::SideBoard { .. } => None,
    }
}

fn resolve_dialogue_choice(
    choice: DialogueChoice,
    quest: &mut QuestLog,
    stats: &mut PlayerStats,
) -> DialogueChoiceResult {
    match choice.kind {
        DialogueChoiceKind::BondResponse => DialogueChoiceResult {
            lines: quest.record_bond_response(choice.selected_response()),
            after: DialogueAfter::None,
            choice: None,
            chapter_art: None,
        },
        DialogueChoiceKind::CampTactic => DialogueChoiceResult {
            lines: quest.record_camp_tactic(choice.selected_camp_bonus()),
            after: DialogueAfter::None,
            choice: None,
            chapter_art: None,
        },
        DialogueChoiceKind::RouteDetour { detour } => {
            let was_done = quest.has_route_detour(detour);
            let resolution = quest.complete_route_detour(detour, choice.selected_detour_approach());
            let mut lines = resolution.lines;
            let is_done = quest.has_route_detour(detour);
            if let Some((side, source)) = route_detour_side_objective(detour) {
                append_side_objective_progress(&mut lines, quest, side, source, was_done, is_done);
            }
            if let Some(reward) = resolution.reward {
                lines.push(apply_route_detour_reward(stats, reward));
            }
            DialogueChoiceResult {
                lines,
                after: DialogueAfter::None,
                choice: None,
                chapter_art: None,
            }
        }
        DialogueChoiceKind::SideBoard { .. } => {
            let Some(task) = choice.selected_side_board_task() else {
                return DialogueChoiceResult {
                    lines: vec!["【委托板】没有可查看的委托。".to_string()],
                    after: DialogueAfter::None,
                    choice: None,
                    chapter_art: None,
                };
            };

            if let Some(next_choice) = side_quest_choice_for(quest, task.side) {
                let mut lines = match next_choice.kind {
                    DialogueChoiceKind::SideQuest {
                        action: SideQuestChoiceAction::Accept,
                        ..
                    } => quest.side_task_accept_preview(task.side),
                    DialogueChoiceKind::SideQuest {
                        action: SideQuestChoiceAction::TurnIn,
                        ..
                    } => vec![
                        quest.side_task_detail(task.side),
                        quest.side_task_contract(task.side),
                        format!(
                            "【交付确认】《{}》条件已达成，可选择稳妥封存或追查余波后发放报酬。",
                            task.side.name()
                        ),
                    ],
                    _ => vec![quest.side_task_detail(task.side)],
                };
                lines.insert(
                    0,
                    format!(
                        "【委托板】选中《{}》：{}。",
                        task.side.name(),
                        task.status.label()
                    ),
                );
                return DialogueChoiceResult {
                    lines,
                    after: DialogueAfter::None,
                    choice: Some(next_choice),
                    chapter_art: None,
                };
            }

            let interaction = quest.interact_side_quest(task.side);
            let mut lines = interaction.lines;
            if let Some(reward) = interaction.reward {
                lines.push(apply_side_quest_reward(stats, reward));
            }
            lines.push(quest.side_task_summary(task.side));
            DialogueChoiceResult {
                lines,
                after: DialogueAfter::None,
                choice: None,
                chapter_art: None,
            }
        }
        DialogueChoiceKind::MainQuest { role, action } => {
            if !choice.accepted() {
                return DialogueChoiceResult {
                    lines: vec![
                        format!("【主线暂缓】{} 暂不{}。", role.name(), action.cancel_verb()),
                        quest.main_task_summary(),
                    ],
                    after: DialogueAfter::None,
                    choice: None,
                    chapter_art: None,
                };
            }

            let stage_before = quest.stage();
            let mut lines = quest.talk(role);
            let mut chapter_art = None;
            if quest.stage() != stage_before {
                chapter_art = append_chapter_card(quest, &mut lines);
            }
            lines.push(quest.main_task_summary());
            DialogueChoiceResult {
                lines,
                after: main_quest_battle_after(role, stage_before),
                choice: None,
                chapter_art,
            }
        }
        DialogueChoiceKind::SideQuest { side, action } => {
            if !choice.accepted() {
                let action_text = match action {
                    SideQuestChoiceAction::Accept => "领取",
                    SideQuestChoiceAction::TurnIn => "交付",
                };
                return DialogueChoiceResult {
                    lines: vec![
                        format!(
                            "【委托暂缓】{} 暂不{}。",
                            quest.side_quest_name(side),
                            action_text
                        ),
                        quest.side_task_summary(side),
                        quest.main_task_summary(),
                    ],
                    after: DialogueAfter::None,
                    choice: None,
                    chapter_art: None,
                };
            }

            let was_active = quest.is_side_quest_active(side);
            let was_completed = quest.is_side_quest_completed(side);
            let interaction = match action {
                SideQuestChoiceAction::Accept => quest.interact_side_quest(side),
                SideQuestChoiceAction::TurnIn => quest.interact_side_quest_with_resolution(
                    side,
                    choice.selected_side_quest_resolution(),
                ),
            };
            let mut lines = interaction.lines;
            if matches!(action, SideQuestChoiceAction::Accept)
                && !was_active
                && !was_completed
                && quest.is_side_quest_active(side)
            {
                lines.push(apply_side_quest_advance_supply(stats, quest, side));
            }
            if let Some(reward) = interaction.reward {
                lines.push(apply_side_quest_reward(stats, reward));
            }
            lines.push(quest.side_task_summary(side));
            lines.push(quest.main_task_summary());
            DialogueChoiceResult {
                lines,
                after: DialogueAfter::None,
                choice: None,
                chapter_art: None,
            }
        }
        DialogueChoiceKind::CommissionTrace {
            side,
            name,
            active_line,
            source,
            inactive_line,
            repeat_line,
        } => {
            if !choice.accepted() {
                return DialogueChoiceResult {
                    lines: vec![
                        format!("【委托现场暂缓】暂不处理《{name}》。"),
                        quest.side_task_summary(side),
                        quest.main_task_summary(),
                    ],
                    after: DialogueAfter::None,
                    choice: None,
                    chapter_art: None,
                };
            }

            let approach = choice.selected_commission_trace_approach();
            let mut lines = quest.interact_commission_trace_with_approach(
                side,
                name,
                active_line,
                source,
                inactive_line,
                repeat_line,
                approach,
            );
            lines.push(quest.side_task_summary(side));
            lines.push(quest.main_task_summary());
            DialogueChoiceResult {
                lines,
                after: DialogueAfter::None,
                choice: None,
                chapter_art: None,
            }
        }
        DialogueChoiceKind::NpcErrand { errand, action } => {
            if !choice.accepted() {
                let action_text = match action {
                    NpcErrandChoiceAction::Accept => "接下",
                    NpcErrandChoiceAction::TurnIn => "交付",
                };
                return DialogueChoiceResult {
                    lines: vec![
                        format!("【托付暂缓】{} 暂不{}。", errand.name(), action_text),
                        quest.npc_errand_summary(),
                        quest.main_task_summary(),
                    ],
                    after: DialogueAfter::None,
                    choice: None,
                    chapter_art: None,
                };
            }

            let interaction = match action {
                NpcErrandChoiceAction::Accept => quest.accept_npc_errand(errand),
                NpcErrandChoiceAction::TurnIn => quest.complete_npc_errand(errand),
            };
            let mut lines = interaction.lines;
            if let Some(reward) = interaction.reward {
                lines.push(apply_npc_errand_reward(stats, reward));
            }
            lines.push(quest.main_task_summary());
            DialogueChoiceResult {
                lines,
                after: DialogueAfter::None,
                choice: None,
                chapter_art: None,
            }
        }
        DialogueChoiceKind::CompanionRevisit { revisit, action } => {
            if !choice.accepted() {
                let action_text = match action {
                    CompanionRevisitChoiceAction::Accept => "领取",
                    CompanionRevisitChoiceAction::TurnIn => "交付",
                };
                return DialogueChoiceResult {
                    lines: vec![
                        format!("【补访暂缓】《{}》暂不{}。", revisit.name(), action_text),
                        quest.companion_revisit_summary(),
                        quest.main_task_summary(),
                    ],
                    after: DialogueAfter::None,
                    choice: None,
                    chapter_art: None,
                };
            }

            let mut lines = match action {
                CompanionRevisitChoiceAction::Accept => quest.accept_companion_revisit(revisit),
                CompanionRevisitChoiceAction::TurnIn => {
                    let turn_in = quest.complete_companion_revisit(revisit);
                    let mut lines = turn_in.lines;
                    if let Some(reward) = turn_in.reward {
                        lines.push(apply_companion_aftermath_reward(stats, reward));
                    }
                    lines
                }
            };
            lines.push(quest.main_task_summary());
            DialogueChoiceResult {
                lines,
                after: DialogueAfter::None,
                choice: None,
                chapter_art: None,
            }
        }
    }
}

fn apply_treasure_reward(stats: &mut PlayerStats, reward: TreasureReward) -> String {
    let levels = stats.gain_exp(reward.exp);
    stats.potions += reward.potions;
    stats.gold += reward.gold;
    if levels > 0 {
        format!(
            "【奖励】经验 +{}，药水 +{}，钱 +{}，境界提升至 Lv.{}。",
            reward.exp, reward.potions, reward.gold, stats.level
        )
    } else {
        format!(
            "【奖励】经验 +{}，药水 +{}，钱 +{}。",
            reward.exp, reward.potions, reward.gold
        )
    }
}

fn apply_supply_reward(stats: &mut PlayerStats, reward: SupplyReward) -> String {
    let levels = stats.gain_exp(reward.exp);
    stats.potions += reward.potions;
    stats.gold += reward.gold;
    stats.hp = (stats.hp + reward.hp).clamp(0, stats.max_hp);
    stats.mp = (stats.mp + reward.mp).clamp(0, stats.max_mp);

    let mut gains = vec![format!("经验 +{}", reward.exp)];
    if reward.potions > 0 {
        gains.push(format!("药水 +{}", reward.potions));
    }
    if reward.gold > 0 {
        gains.push(format!("钱 +{}", reward.gold));
    }
    if reward.hp > 0 {
        gains.push(format!("气血 +{}", reward.hp));
    }
    if reward.mp > 0 {
        gains.push(format!("灵力 +{}", reward.mp));
    }

    if levels > 0 {
        gains.push(format!("境界提升至 Lv.{}", stats.level));
    }

    format!("【采集奖励】{}。", gains.join("，"))
}

fn apply_care_aftermath_reward(stats: &mut PlayerStats, reward: CareAftermathReward) -> String {
    let levels = stats.gain_exp(reward.exp);
    stats.potions += reward.potions;
    stats.gold += reward.gold;
    if levels > 0 {
        format!(
            "【照应回礼】经验 +{}，药水 +{}，钱 +{}，境界提升至 Lv.{}。",
            reward.exp, reward.potions, reward.gold, stats.level
        )
    } else {
        format!(
            "【照应回礼】经验 +{}，药水 +{}，钱 +{}。",
            reward.exp, reward.potions, reward.gold
        )
    }
}

fn apply_companion_aftermath_reward(
    stats: &mut PlayerStats,
    reward: CompanionAftermathReward,
) -> String {
    let levels = stats.gain_exp(reward.exp);
    stats.potions += reward.potions;
    stats.gold += reward.gold;

    let mut gains = vec![format!("经验 +{}", reward.exp)];
    if reward.potions > 0 {
        gains.push(format!("药水 +{}", reward.potions));
    }
    if reward.gold > 0 {
        gains.push(format!("钱 +{}", reward.gold));
    }
    if levels > 0 {
        gains.push(format!("境界提升至 Lv.{}", stats.level));
    }
    format!("【小传回礼】{}。", gains.join("，"))
}

fn apply_commission_aftermath_reward(
    stats: &mut PlayerStats,
    reward: CommissionAftermathReward,
) -> String {
    let levels = stats.gain_exp(reward.exp);
    stats.potions += reward.potions;
    stats.gold += reward.gold;
    if levels > 0 {
        format!(
            "【清账回礼】经验 +{}，药水 +{}，钱 +{}，境界提升至 Lv.{}。",
            reward.exp, reward.potions, reward.gold, stats.level
        )
    } else {
        format!(
            "【清账回礼】经验 +{}，药水 +{}，钱 +{}。",
            reward.exp, reward.potions, reward.gold
        )
    }
}

fn local_commission_aftermath_line(kind: MapKind) -> &'static str {
    match map_chapter(kind) {
        Chapter::VillageOath => {
            "【地方回礼】村人把两张委托签都收进旧木匣，替你们备下一份赶路药钱。"
        }
        Chapter::MoonCave => "【地方回礼】洞天守夜人说晶尘与回声都稳了，便把月露和铜钱托你带上。",
        Chapter::RiverMedicine => {
            "【地方回礼】江岸人把河灯和湿货账一并销去，码头凑出一份夜巡谢礼。"
        }
        Chapter::PlagueRain => "【地方回礼】瘴雨村药锅重新沸起，病屋里的人把省下的药钱交给你们。",
        Chapter::CapitalMirror => {
            "【地方回礼】府城暗线烧掉最后一张密榜，留下封好的线报钱和护身药。"
        }
        Chapter::SouthernThunder => {
            "【地方回礼】百越族人把安静下来的雷纹拓成符，连同药酒交到你手里。"
        }
        Chapter::FinalDream => "【地方回礼】守灯人合上终门灯簿，说归路有人记得你们替人间守过灯。",
    }
}

fn apply_route_detour_reward(stats: &mut PlayerStats, reward: RouteDetourReward) -> String {
    let levels = stats.gain_exp(reward.exp);
    stats.potions += reward.potions;
    stats.gold += reward.gold;
    if levels > 0 {
        format!(
            "【分支收获】经验 +{}，药水 +{}，钱 +{}，境界提升至 Lv.{}。",
            reward.exp, reward.potions, reward.gold, stats.level
        )
    } else {
        format!(
            "【分支收获】经验 +{}，药水 +{}，钱 +{}。",
            reward.exp, reward.potions, reward.gold
        )
    }
}

fn apply_route_detour_report_reward(
    stats: &mut PlayerStats,
    reward: RouteDetourReportReward,
) -> String {
    let levels = stats.gain_exp(reward.exp);
    stats.potions += reward.potions;
    stats.gold += reward.gold;
    if levels > 0 {
        format!(
            "【报路回礼】经验 +{}，药水 +{}，钱 +{}，境界提升至 Lv.{}。",
            reward.exp, reward.potions, reward.gold, stats.level
        )
    } else {
        format!(
            "【报路回礼】经验 +{}，药水 +{}，钱 +{}。",
            reward.exp, reward.potions, reward.gold
        )
    }
}

fn apply_shrine_offering(
    stats: &mut PlayerStats,
    quest: &mut QuestLog,
    blessing: ShrineBlessing,
) -> Vec<String> {
    if let Some(active) = quest.active_shrine_blessing() {
        return vec![
            format!("【供奉】{}仍在。", active.name()),
            active.battle_line().to_string(),
            format!("【行路护持】{}", active.route_line()),
        ];
    }

    if !stats.spend_gold(SHRINE_OFFERING_PRICE) {
        return vec![format!(
            "【供奉】供香需要 {SHRINE_OFFERING_PRICE} 文，你的钱不够。"
        )];
    }

    quest.set_shrine_blessing(blessing);
    vec![
        format!(
            "【供奉】献上 {SHRINE_OFFERING_PRICE} 文供香，获得{}。",
            blessing.name()
        ),
        blessing.battle_line().to_string(),
        format!("【行路护持】{}", blessing.route_line()),
    ]
}

fn route_encounter_rate(base: f32, kind: MapKind, quest: &QuestLog) -> f32 {
    let route = route_mark_for_map(kind)
        .map(|mark| quest.route_mark_encounter_multiplier(mark))
        .unwrap_or(1.0);
    let branch = route_detour_for_map(kind)
        .map(|detour| quest.route_detour_encounter_multiplier(detour))
        .unwrap_or(1.0);
    (base
        * quest.shrine_encounter_rate_multiplier()
        * route
        * branch
        * side_quest_route_relief_multiplier(kind, quest)
        * npc_errand_route_relief_multiplier(kind, quest)
        * route_care_encounter_multiplier(kind, quest)
        * companion_route_encounter_multiplier(kind, quest)
        * companion_revisit_route_multiplier(kind, quest))
    .clamp(0.0, 1.0)
}

const NPC_ERRAND_ROUTE_NONE: [NpcErrand; 0] = [];
const NPC_ERRAND_ROUTE_MOON: [NpcErrand; 1] = [NpcErrand::BambooDewToCave];
const NPC_ERRAND_ROUTE_RIVER_TOWN: [NpcErrand; 1] = [NpcErrand::MoonMossToRiver];
const NPC_ERRAND_ROUTE_REED: [NpcErrand; 2] =
    [NpcErrand::MoonMossToRiver, NpcErrand::RiverReedLetter];
const NPC_ERRAND_ROUTE_PLAGUE: [NpcErrand; 1] = [NpcErrand::PlagueChildCharm];
const NPC_ERRAND_ROUTE_MIRROR: [NpcErrand; 1] = [NpcErrand::CapitalStarSlip];
const NPC_ERRAND_ROUTE_SOUTHERN: [NpcErrand; 1] = [NpcErrand::MirrorMedicineToSouth];
const NPC_ERRAND_ROUTE_THUNDER: [NpcErrand; 2] = [
    NpcErrand::MirrorMedicineToSouth,
    NpcErrand::SouthernThunderWine,
];
const NPC_ERRAND_ROUTE_DREAM: [NpcErrand; 1] = [NpcErrand::FinalLampWick];
const TRACKED_NPC_ERRANDS: [NpcErrand; 8] = [
    NpcErrand::BambooDewToCave,
    NpcErrand::MoonMossToRiver,
    NpcErrand::RiverReedLetter,
    NpcErrand::PlagueChildCharm,
    NpcErrand::CapitalStarSlip,
    NpcErrand::MirrorMedicineToSouth,
    NpcErrand::SouthernThunderWine,
    NpcErrand::FinalLampWick,
];

fn npc_errand_route_relief_errands(kind: MapKind) -> &'static [NpcErrand] {
    match kind {
        MapKind::Cave | MapKind::MoonEchoCorridor => &NPC_ERRAND_ROUTE_MOON,
        MapKind::RiverTown => &NPC_ERRAND_ROUTE_RIVER_TOWN,
        MapKind::RiverReedBed => &NPC_ERRAND_ROUTE_REED,
        MapKind::PlagueVillage | MapKind::PlagueShrinePath => &NPC_ERRAND_ROUTE_PLAGUE,
        MapKind::CapitalMansion | MapKind::MansionMirrorGallery => &NPC_ERRAND_ROUTE_MIRROR,
        MapKind::SouthernRoad => &NPC_ERRAND_ROUTE_SOUTHERN,
        MapKind::ThunderDrumPath => &NPC_ERRAND_ROUTE_THUNDER,
        MapKind::DreamWaterway => &NPC_ERRAND_ROUTE_DREAM,
        MapKind::Village | MapKind::Bamboo | MapKind::Capital | MapKind::FinalSanctum => {
            &NPC_ERRAND_ROUTE_NONE
        }
    }
}

fn npc_errand_route_relief_count(kind: MapKind, quest: &QuestLog) -> usize {
    npc_errand_route_relief_errands(kind)
        .iter()
        .filter(|errand| quest.is_npc_errand_completed(**errand))
        .count()
}

fn npc_errand_route_relief_multiplier(kind: MapKind, quest: &QuestLog) -> f32 {
    match npc_errand_route_relief_count(kind, quest) {
        0 => 1.0,
        1 => 0.97,
        _ => 0.94,
    }
}

fn npc_errand_route_relief_summary(kind: MapKind, quest: &QuestLog) -> Option<String> {
    let count = npc_errand_route_relief_count(kind, quest);
    if count == 0 {
        return None;
    }
    let label = if count >= npc_errand_route_relief_errands(kind).len() && count > 1 {
        "连稳"
    } else {
        "已稳"
    };
    Some(format!("托付回声 {label} 遇妖-{}%", count.min(2) * 3))
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RouteCareState {
    NoRoute,
    Prepared,
    Partial,
    Unprepared,
    Missed,
}

fn route_care_state(kind: MapKind, quest: &QuestLog) -> RouteCareState {
    if route_mark_for_map(kind).is_none() {
        return RouteCareState::NoRoute;
    }

    let route_chapter = map_chapter(kind);
    let current_rank = chapter_rank(quest.current_chapter());
    let route_rank = chapter_rank(route_chapter);
    if current_rank < route_rank {
        return RouteCareState::NoRoute;
    }

    let (bond_scene, camp_scene) = map_care_scenes(kind);
    let bond_seen = quest.has_seen_bond_scene(bond_scene);
    let camp_seen = quest.has_seen_camp_scene(camp_scene);

    match (bond_seen, camp_seen) {
        (true, true) => RouteCareState::Prepared,
        (true, false) | (false, true) => RouteCareState::Partial,
        (false, false) if current_rank > route_rank => RouteCareState::Missed,
        (false, false) => RouteCareState::Unprepared,
    }
}

fn route_care_encounter_multiplier(kind: MapKind, quest: &QuestLog) -> f32 {
    match route_care_state(kind, quest) {
        RouteCareState::NoRoute => 1.0,
        RouteCareState::Prepared => 0.86,
        RouteCareState::Partial => 0.94,
        RouteCareState::Unprepared => 1.06,
        RouteCareState::Missed => 1.12,
    }
}

fn route_care_summary(kind: MapKind, quest: &QuestLog) -> Option<&'static str> {
    match route_care_state(kind, quest) {
        RouteCareState::NoRoute => None,
        RouteCareState::Prepared => Some("路线照应 周全 遇妖-14%"),
        RouteCareState::Partial => Some("路线照应 半备 遇妖-6%"),
        RouteCareState::Unprepared => Some("路线照应 欠备 遇妖+6%"),
        RouteCareState::Missed => Some("路线照应 错过 遇妖+12%"),
    }
}

fn route_care_checkpoint_line(kind: MapKind, quest: &QuestLog) -> Option<&'static str> {
    match route_care_state(kind, quest) {
        RouteCareState::NoRoute => None,
        RouteCareState::Prepared => {
            Some("【同行照应】夜谈和歇脚处的暗号都接上了，灵儿与月衡分头看路，本路遇妖压力降低。")
        }
        RouteCareState::Partial => {
            Some("【同行照应】这段路只记住了一半暗号，队伍仍能互相提醒，但防不住所有妖影。")
        }
        RouteCareState::Unprepared => {
            Some("【同行照应】还没有把本章夜谈和营地休整补齐，草中妖影更容易截住去路。")
        }
        RouteCareState::Missed => {
            Some("【同行照应】这段旧路错过了当时该说的话和该歇的脚，回头再走时妖影更缠人。")
        }
    }
}

const COMPANION_ROUTE_NONE: [CompanionScene; 0] = [];
const COMPANION_ROUTE_TRAIL: [CompanionScene; 1] = [CompanionScene::SwordSisterTrailGuard];
const COMPANION_ROUTE_CAPITAL: [CompanionScene; 1] = [CompanionScene::SwordSisterCapitalMirror];
const COMPANION_ROUTE_SOUTHERN: [CompanionScene; 1] = [CompanionScene::SpiritWitchSouthernTotem];
const COMPANION_ROUTE_FINAL: [CompanionScene; 2] = [
    CompanionScene::SwordSisterFinalReturn,
    CompanionScene::SpiritWitchFinalVow,
];

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CompanionRouteState {
    NoRoute,
    Partial,
    Prepared,
    Missed,
}

fn companion_route_scenes(kind: MapKind) -> &'static [CompanionScene] {
    match kind {
        MapKind::Village | MapKind::Bamboo | MapKind::MoonEchoCorridor => &COMPANION_ROUTE_TRAIL,
        MapKind::Capital | MapKind::CapitalMansion | MapKind::MansionMirrorGallery => {
            &COMPANION_ROUTE_CAPITAL
        }
        MapKind::SouthernRoad | MapKind::ThunderDrumPath => &COMPANION_ROUTE_SOUTHERN,
        MapKind::FinalSanctum | MapKind::DreamWaterway => &COMPANION_ROUTE_FINAL,
        MapKind::Cave
        | MapKind::RiverTown
        | MapKind::RiverReedBed
        | MapKind::PlagueVillage
        | MapKind::PlagueShrinePath => &COMPANION_ROUTE_NONE,
    }
}

fn companion_route_state(kind: MapKind, quest: &QuestLog) -> CompanionRouteState {
    let scenes = companion_route_scenes(kind);
    if scenes.is_empty() {
        return CompanionRouteState::NoRoute;
    }

    let route_chapter = map_chapter(kind);
    let current_rank = chapter_rank(quest.current_chapter());
    let route_rank = chapter_rank(route_chapter);
    if current_rank < route_rank {
        return CompanionRouteState::NoRoute;
    }

    let seen = scenes
        .iter()
        .filter(|scene| quest.has_seen_companion_scene(**scene))
        .count();
    if seen == scenes.len() {
        CompanionRouteState::Prepared
    } else if seen > 0 {
        CompanionRouteState::Partial
    } else if current_rank > route_rank {
        CompanionRouteState::Missed
    } else {
        CompanionRouteState::NoRoute
    }
}

fn companion_route_encounter_multiplier(kind: MapKind, quest: &QuestLog) -> f32 {
    if route_mark_for_map(kind).is_none() {
        return 1.0;
    }

    match companion_route_state(kind, quest) {
        CompanionRouteState::NoRoute => 1.0,
        CompanionRouteState::Partial => 0.96,
        CompanionRouteState::Prepared => 0.92,
        CompanionRouteState::Missed => 1.04,
    }
}

fn companion_revisit_for_route(kind: MapKind) -> Option<CompanionRevisit> {
    match kind {
        MapKind::PlagueShrinePath => Some(CompanionRevisit::TrailEcho),
        MapKind::ThunderDrumPath => Some(CompanionRevisit::MirrorTrace),
        MapKind::DreamWaterway => Some(CompanionRevisit::TotemVow),
        _ => None,
    }
}

fn companion_revisit_route_multiplier(kind: MapKind, quest: &QuestLog) -> f32 {
    companion_revisit_for_route(kind)
        .filter(|revisit| quest.companion_revisit_resolved(*revisit))
        .map(|_| 0.96)
        .unwrap_or(1.0)
}

fn companion_revisit_route_summary(kind: MapKind, quest: &QuestLog) -> Option<&'static str> {
    let revisit = companion_revisit_for_route(kind)?;
    if quest.companion_revisit_resolved(revisit) {
        Some("小传补访 已补 遇妖-4%")
    } else if quest.companion_revisit_active(revisit) {
        Some("小传补访 寻访中")
    } else {
        None
    }
}

fn companion_revisit_checkpoint_line(kind: MapKind, quest: &QuestLog) -> Option<&'static str> {
    let revisit = companion_revisit_for_route(kind)?;
    if !quest.companion_revisit_resolved(revisit) {
        return None;
    }
    Some(match revisit {
        CompanionRevisit::TrailEcho => {
            "【补访照应】月衡把旧栅退路重新记进祠道，瘴雨妖影不再轻易截住后队。"
        }
        CompanionRevisit::MirrorTrace => "【补访照应】月衡借雷光照清旧案，回坡镜影更难从背后绕回。",
        CompanionRevisit::TotemVow => {
            "【补访照应】南瑶接回雷纹旧愿，归水回卷会提前避开同行人的名字。"
        }
    })
}

fn companion_route_summary(kind: MapKind, quest: &QuestLog) -> Option<&'static str> {
    route_mark_for_map(kind)?;

    match companion_route_state(kind, quest) {
        CompanionRouteState::NoRoute => None,
        CompanionRouteState::Partial => Some("小传照应 半备 遇妖-4%"),
        CompanionRouteState::Prepared => Some("小传照应 周全 遇妖-8%"),
        CompanionRouteState::Missed => Some("小传照应 错过 遇妖+4%"),
    }
}

fn route_pressure_summary(kind: MapKind, quest: &QuestLog) -> Option<String> {
    let mut parts = Vec::new();
    if let Some(care) = route_care_summary(kind, quest) {
        parts.push(care.to_string());
    }
    if let Some(companion) = companion_route_summary(kind, quest) {
        parts.push(companion.to_string());
    }
    if let Some(revisit) = companion_revisit_route_summary(kind, quest) {
        parts.push(revisit.to_string());
    }
    if let Some(relief) = side_quest_route_relief_summary(kind, quest) {
        parts.push(relief);
    }
    if let Some(errand) = npc_errand_route_relief_summary(kind, quest) {
        parts.push(errand);
    }

    if parts.is_empty() {
        None
    } else {
        Some(parts.join(" | "))
    }
}

fn companion_route_checkpoint_line(kind: MapKind, quest: &QuestLog) -> Option<&'static str> {
    match kind {
        MapKind::MoonEchoCorridor
            if quest.has_seen_companion_scene(CompanionScene::SwordSisterTrailGuard) =>
        {
            Some("【小传照应】林月衡记下旧栅退路，回声路上的妖影更难截断后队。")
        }
        MapKind::MansionMirrorGallery
            if quest.has_seen_companion_scene(CompanionScene::SwordSisterCapitalMirror) =>
        {
            Some("【小传照应】林月衡认得照影案的旧破口，镜廊巡路更容易看穿虚影。")
        }
        MapKind::ThunderDrumPath
            if quest.has_seen_companion_scene(CompanionScene::SpiritWitchSouthernTotem) =>
        {
            Some("【小传照应】南瑶听懂雷纹旧愿，鼓道乱雷会提前避开你们半步。")
        }
        MapKind::DreamWaterway
            if quest.has_seen_companion_scene(CompanionScene::SwordSisterFinalReturn)
                && quest.has_seen_companion_scene(CompanionScene::SpiritWitchFinalVow) =>
        {
            Some("【小传照应】月衡与南瑶都把归路记进灯里，旧梦水廊的回卷轻了许多。")
        }
        MapKind::DreamWaterway
            if quest.has_seen_companion_scene(CompanionScene::SwordSisterFinalReturn) =>
        {
            Some("【小传照应】林月衡的归剑压住水廊一侧，旧梦妖影少了一个截口。")
        }
        MapKind::DreamWaterway
            if quest.has_seen_companion_scene(CompanionScene::SpiritWitchFinalVow) =>
        {
            Some("【小传照应】南瑶的归潮灯誓稳住水声，旧梦回卷会慢半拍。")
        }
        MapKind::MoonEchoCorridor
            if companion_route_state(kind, quest) == CompanionRouteState::Missed =>
        {
            Some("【小传照应】旧栅小传错过了，回声路上少了月衡留下的后队退路。")
        }
        MapKind::MansionMirrorGallery
            if companion_route_state(kind, quest) == CompanionRouteState::Missed =>
        {
            Some("【小传照应】照影旧案没有问清，镜廊里的虚影仍会绕回你们身后。")
        }
        MapKind::ThunderDrumPath
            if companion_route_state(kind, quest) == CompanionRouteState::Missed =>
        {
            Some("【小传照应】雷纹旧愿错过了，鼓道乱雷会趁沉默处压住归路。")
        }
        MapKind::DreamWaterway
            if companion_route_state(kind, quest) == CompanionRouteState::Missed =>
        {
            Some("【小传照应】终门小传没有补齐，旧梦水廊的回卷仍会追上后队。")
        }
        _ => None,
    }
}

fn side_quest_route_checkpoint_line(kind: MapKind, quest: &QuestLog) -> Option<&'static str> {
    match (kind, side_quest_route_relief_state(kind, quest)) {
        (MapKind::MoonEchoCorridor, SideQuestRouteReliefState::Partial) => {
            Some("【委托清障】水月洞天已有一张委托归档，路印旁的巡灯亮得更远。")
        }
        (MapKind::MoonEchoCorridor, SideQuestRouteReliefState::Cleared) => {
            Some("【委托清障】晶尘与回声都已交清，回廊路印把残余妖影压回石壁。")
        }
        (MapKind::RiverReedBed, SideQuestRouteReliefState::Partial) => {
            Some("【委托清障】江岸已有一张水签归档，芦滩夜路少了一处逆流灯影。")
        }
        (MapKind::RiverReedBed, SideQuestRouteReliefState::Cleared) => {
            Some("【委托清障】河灯与湿货都归位，芦滩路印顺着人声亮起。")
        }
        (MapKind::PlagueShrinePath, SideQuestRouteReliefState::Partial) => {
            Some("【委托清障】瘴雨村急榜少了一张，旧铃路印能压住半段瘴风。")
        }
        (MapKind::PlagueShrinePath, SideQuestRouteReliefState::Cleared) => {
            Some("【委托清障】救急药与药童都已安稳，祠道路印把病屋外的瘴影逼散。")
        }
        (MapKind::MansionMirrorGallery, SideQuestRouteReliefState::Partial) => {
            Some("【委托清障】府城密榜已有回执，镜廊偏门的粉记多了一处退路。")
        }
        (MapKind::MansionMirrorGallery, SideQuestRouteReliefState::Cleared) => {
            Some("【委托清障】巡查与暗帖都交清，镜廊路印照出余影藏身处。")
        }
        (MapKind::ThunderDrumPath, SideQuestRouteReliefState::Partial) => {
            Some("【委托清障】灵道一段雷声已定，回坡路印暂时压低鼓道乱雷。")
        }
        (MapKind::ThunderDrumPath, SideQuestRouteReliefState::Cleared) => {
            Some("【委托清障】雷纹与战鼓都安静了，回坡路印把灵道脉络接稳。")
        }
        (MapKind::DreamWaterway, SideQuestRouteReliefState::Partial) => {
            Some("【委托清障】终门灯簿合上一页，归水路印少受一段旧梦回卷。")
        }
        (MapKind::DreamWaterway, SideQuestRouteReliefState::Cleared) => {
            Some("【委托清障】梦灯余波与归潮旧愿都归簿，旧梦路印照住回程水线。")
        }
        _ => None,
    }
}

fn npc_errand_route_checkpoint_line(kind: MapKind, quest: &QuestLog) -> Option<&'static str> {
    match (kind, npc_errand_route_relief_count(kind, quest)) {
        (MapKind::MoonEchoCorridor, 1..) => {
            Some("【托付回声】竹露送到后，路印旁的月寒药香把回声妖影逼远了一圈。")
        }
        (MapKind::RiverReedBed, 1) => {
            Some("【托付回声】月苔水符压住渡口寒雾，芦叶间的伏妖声低了半拍。")
        }
        (MapKind::RiverReedBed, 2..) => {
            Some("【托付回声】月苔与芦滩小信都已归位，新浅渡路图让夜巡少绕一段险水。")
        }
        (MapKind::PlagueShrinePath, 1..) => {
            Some("【托付回声】病童护符牵住药篮，祠道瘴源不再贴着路印冒气。")
        }
        (MapKind::MansionMirrorGallery, 1..) => {
            Some("【托付回声】星图密片点明换镜时辰，镜廊路印能提前避开假脚步。")
        }
        (MapKind::SouthernRoad, 1..) => {
            Some("【托付回声】照影药引显出草路灰毒，南疆路印少了一层京华残影。")
        }
        (MapKind::ThunderDrumPath, 1) => {
            Some("【托付回声】照影药引压住雷草灰毒，鼓道边缘的伏影退开半步。")
        }
        (MapKind::ThunderDrumPath, 2..) => {
            Some("【托付回声】药引和雷草酒都已送到，雷鼓路印旁的药炉稳住守夜火。")
        }
        (MapKind::DreamWaterway, 1..) => {
            Some("【托付回声】旧梦灯芯亮进誓灯，水廊路印不再被回卷完全吞没。")
        }
        _ => None,
    }
}

fn service_price(service: NpcService, kind: MapKind, quest: &QuestLog) -> u32 {
    match (service, local_favor_unlocked(kind, quest)) {
        (NpcService::Shop, true) => FAVORED_POTION_PRICE,
        (NpcService::Inn, true) => FAVORED_INN_PRICE,
        (NpcService::Shop, false) => POTION_PRICE,
        (NpcService::Inn, false) => INN_PRICE,
        (NpcService::HomeRest | NpcService::CampRest, _) => 0,
    }
}

fn gear_offer_for_map(kind: MapKind) -> Option<GearOffer> {
    match kind {
        MapKind::Village => Some(GearOffer {
            gear: ShopGear::VillageSwordTassel,
            price: VILLAGE_GEAR_PRICE,
            atk: 2,
            def: 1,
            max_hp: 0,
            max_mp: 0,
            seller: "余杭药铺",
            flavor: "旧剑穗压住剑柄浮躁，出手更稳。",
        }),
        MapKind::RiverTown => Some(GearOffer {
            gear: ShopGear::RiverSilkVest,
            price: RIVER_GEAR_PRICE,
            atk: 0,
            def: 2,
            max_hp: 12,
            max_mp: 4,
            seller: "江岸绣铺",
            flavor: "水绫缝进护衣，行船遇妖时能护住心口。",
        }),
        MapKind::Capital => Some(GearOffer {
            gear: ShopGear::CapitalMirrorGuard,
            price: CAPITAL_GEAR_PRICE,
            atk: 1,
            def: 4,
            max_hp: 10,
            max_mp: 0,
            seller: "云都暗铺",
            flavor: "护心镜把照影术反光压低，近战不易被虚招绕住。",
        }),
        MapKind::SouthernRoad => Some(GearOffer {
            gear: ShopGear::SouthernThunderCharm,
            price: SOUTHERN_GEAR_PRICE,
            atk: 3,
            def: 1,
            max_hp: 0,
            max_mp: 8,
            seller: "南疆行商",
            flavor: "雷纹护符贴在腕上，剑势和灵力都会多一分余震。",
        }),
        MapKind::Bamboo
        | MapKind::Cave
        | MapKind::MoonEchoCorridor
        | MapKind::RiverReedBed
        | MapKind::PlagueVillage
        | MapKind::PlagueShrinePath
        | MapKind::CapitalMansion
        | MapKind::MansionMirrorGallery
        | MapKind::ThunderDrumPath
        | MapKind::FinalSanctum
        | MapKind::DreamWaterway => None,
    }
}

fn stat_delta_text(offer: GearOffer) -> String {
    let mut deltas = Vec::new();
    if offer.atk != 0 {
        deltas.push(format!("攻击 +{}", offer.atk));
    }
    if offer.def != 0 {
        deltas.push(format!("防御 +{}", offer.def));
    }
    if offer.max_hp != 0 {
        deltas.push(format!("气血上限 +{}", offer.max_hp));
    }
    if offer.max_mp != 0 {
        deltas.push(format!("灵力上限 +{}", offer.max_mp));
    }
    deltas.join("，")
}

fn apply_gear_offer(
    offer: GearOffer,
    quest: &mut QuestLog,
    stats: &mut PlayerStats,
) -> NpcServiceResult {
    if stats.gold < offer.price {
        return NpcServiceResult {
            line: format!(
                "【装备铺】{}留着{}，要 {} 文；你现在只有 {} 文。",
                offer.seller,
                offer.gear.name(),
                offer.price,
                stats.gold
            ),
            rested: false,
        };
    }

    stats.gold -= offer.price;
    stats.atk += offer.atk;
    stats.def += offer.def;
    stats.max_hp += offer.max_hp;
    stats.hp += offer.max_hp;
    stats.max_mp += offer.max_mp;
    stats.mp += offer.max_mp;
    quest.record_shop_gear(offer.gear);

    NpcServiceResult {
        line: format!(
            "【装备铺】花 {} 文买下{}，{} 剩余 {} 文。{}\n【装备成长】{} · {}",
            offer.price,
            offer.gear.name(),
            offer.flavor,
            stats.gold,
            stat_delta_text(offer),
            quest.shop_gear_summary(),
            "之后再找此处商人会改买药水"
        ),
        rested: false,
    }
}

fn apply_npc_service(
    service: NpcService,
    kind: MapKind,
    quest: &mut QuestLog,
    stats: &mut PlayerStats,
) -> NpcServiceResult {
    match service {
        NpcService::HomeRest => {
            if stats.hp == stats.max_hp && stats.mp == stats.max_mp {
                NpcServiceResult {
                    line: "【歇息】婆婆备好了热饭，你现在状态正好。".to_string(),
                    rested: true,
                }
            } else {
                stats.full_restore();
                NpcServiceResult {
                    line: "【歇息】在家中歇了一会儿，气血和灵力已恢复。".to_string(),
                    rested: true,
                }
            }
        }
        NpcService::Inn => {
            let price = service_price(NpcService::Inn, kind, quest);
            if stats.hp == stats.max_hp && stats.mp == stats.max_mp {
                return NpcServiceResult {
                    line: "【客栈】掌柜端来热汤，你坐下歇脚，状态正好。".to_string(),
                    rested: true,
                };
            }
            if stats.spend_gold(price) {
                stats.full_restore();
                let favor = if price < INN_PRICE {
                    "乡里护持，"
                } else {
                    ""
                };
                NpcServiceResult {
                    line: format!("【客栈】{favor}花 {price} 文住了一晚，气血和灵力已恢复。"),
                    rested: true,
                }
            } else {
                NpcServiceResult {
                    line: format!("【客栈】住店要 {price} 文，你的钱不够。"),
                    rested: false,
                }
            }
        }
        NpcService::CampRest => {
            let favor = if local_favor_unlocked(kind, quest) {
                "当地人替你守夜，"
            } else {
                ""
            };
            if stats.hp == stats.max_hp && stats.mp == stats.max_mp {
                NpcServiceResult {
                    line: format!("【休整】{favor}同伴围坐片刻，确认行囊与伤势都无大碍。"),
                    rested: true,
                }
            } else {
                stats.full_restore();
                NpcServiceResult {
                    line: format!("【休整】{favor}借此处安顿片刻，气血和灵力已恢复。"),
                    rested: true,
                }
            }
        }
        NpcService::Shop => {
            if let Some(offer) = gear_offer_for_map(kind) {
                if !quest.has_shop_gear(offer.gear) {
                    return apply_gear_offer(offer, quest, stats);
                }
            }

            let price = service_price(NpcService::Shop, kind, quest);
            if stats.spend_gold(price) {
                stats.potions += 1;
                let favor = if price < POTION_PRICE {
                    "乡里护持，"
                } else {
                    ""
                };
                NpcServiceResult {
                    line: format!(
                        "【药铺】{favor}花 {price} 文买入一瓶药水，剩余 {} 文。",
                        stats.gold
                    ),
                    rested: false,
                }
            } else {
                NpcServiceResult {
                    line: format!("【药铺】一瓶药水 {price} 文，你的钱不够。"),
                    rested: false,
                }
            }
        }
    }
}

fn apply_bond_reward(stats: &mut PlayerStats, reward: BondReward) -> String {
    let levels = stats.gain_exp(reward.exp);
    stats.potions += reward.potions;
    if reward.full_restore {
        stats.full_restore();
    }

    let restore_text = if reward.full_restore {
        "，气血灵力已恢复"
    } else {
        ""
    };
    if levels > 0 {
        format!(
            "【羁绊奖励】经验 +{}，药水 +{}{}，境界提升至 Lv.{}。",
            reward.exp, reward.potions, restore_text, stats.level
        )
    } else {
        format!(
            "【羁绊奖励】经验 +{}，药水 +{}{}。",
            reward.exp, reward.potions, restore_text
        )
    }
}

fn apply_companion_scene_reward(stats: &mut PlayerStats, reward: CompanionSceneReward) -> String {
    let levels = stats.gain_exp(reward.exp);
    stats.potions += reward.potions;
    if levels > 0 {
        format!(
            "【小传奖励】经验 +{}，药水 +{}，境界提升至 Lv.{}。",
            reward.exp, reward.potions, stats.level
        )
    } else {
        format!(
            "【小传奖励】经验 +{}，药水 +{}。",
            reward.exp, reward.potions
        )
    }
}

fn portal_gate_message(
    current: MapKind,
    target: MapKind,
    stage: QuestStage,
) -> Option<&'static str> {
    match (current, target, stage) {
        (MapKind::Village, MapKind::Bamboo, QuestStage::NotStarted | QuestStage::TalkToLinger) => {
            Some("山路被妖雾封住了。先把村里的委托和灵符线索弄清楚。")
        }
        (
            MapKind::Bamboo,
            MapKind::Cave,
            QuestStage::NotStarted
            | QuestStage::TalkToLinger
            | QuestStage::FindStarMage
            | QuestStage::DefeatMonsters { .. }
            | QuestStage::ReturnToSister
            | QuestStage::EscortMerchant
            | QuestStage::FindBambooScout,
        ) => Some("竹林深处的月洞还未开门。先找到该给你令牌的人。"),
        (
            MapKind::Capital,
            MapKind::CapitalMansion,
            QuestStage::PlagueVillageComplete | QuestStage::SeekCapitalEnvoy,
        ) => Some("府邸东门仍被禁军封住。先在云都府城找宣令使取得入城符。"),
        _ => None,
    }
}

fn portal_transition_flavor(current: MapKind, target: MapKind) -> &'static str {
    match (current, target) {
        (MapKind::Village, MapKind::Bamboo) => {
            "村灯被竹影吞在身后，山径妖雾把第一段江湖路推到脚前。"
        }
        (MapKind::Bamboo, MapKind::Cave) => "青竹尽头的水声忽然贴近，月洞石门在雾里开出冷光。",
        (MapKind::Cave, MapKind::MoonEchoCorridor) => {
            "洞壁水纹向东退开，回声窄廊把每一步都还给你们。"
        }
        (MapKind::MoonEchoCorridor, MapKind::Cave) => {
            "回廊水光渐稳，月洞祭司那边仍留着一盏引路灯。"
        }
        (MapKind::Cave, MapKind::RiverTown) => "月魄水色散进江雾，远处药庐与码头灯火一起亮起。",
        (MapKind::RiverTown, MapKind::RiverReedBed) => {
            "镇口水灯倒映成线，芦滩里的浅水把脚步声压得很低。"
        }
        (MapKind::RiverReedBed, MapKind::PlagueVillage) => {
            "芦叶后的江风忽然发苦，瘴雨村的旧铃声从雾里传来。"
        }
        (MapKind::PlagueVillage, MapKind::PlagueShrinePath) => {
            "黑草贴着祠道生长，净瘴铃在前方断断续续地回应。"
        }
        (MapKind::PlagueShrinePath, MapKind::PlagueVillage) => {
            "祠道瘴声被抛在身后，村中药火仍在苦雨里守着。"
        }
        (MapKind::PlagueVillage, MapKind::Capital) => {
            "瘴雨退成远雾，云都高墙把所有传闻都压进灯下。"
        }
        (MapKind::Capital, MapKind::CapitalMansion) => {
            "府邸门灯无风自晃，照影案的冷光从偏院石阶下渗出。"
        }
        (MapKind::CapitalMansion, MapKind::MansionMirrorGallery) => {
            "偏院纸灯照出第二道人影，镜廊深处传来翻账声。"
        }
        (MapKind::MansionMirrorGallery, MapKind::CapitalMansion) => {
            "镜光退回廊柱之间，偏院内线仍在等密札合拢。"
        }
        (MapKind::Capital, MapKind::SouthernRoad) => {
            "京华灯影在背后熄成一点，南疆雷云贴着山脊压来。"
        }
        (MapKind::SouthernRoad, MapKind::ThunderDrumPath) => {
            "灵道路旁的雷草一齐伏低，鼓声从石坡深处滚上来。"
        }
        (MapKind::ThunderDrumPath, MapKind::SouthernRoad) => {
            "鼓道乱雷被山风带远，南疆灵道重新露出归路。"
        }
        (MapKind::SouthernRoad, MapKind::FinalSanctum) => {
            "雷云尽处水声倒卷，灵渊终门把来路照成一线。"
        }
        (MapKind::FinalSanctum, MapKind::DreamWaterway) => {
            "终门灯火沉入水面，旧梦水廊把未说完的话卷回来。"
        }
        (MapKind::DreamWaterway, MapKind::FinalSanctum) => {
            "水廊回声渐轻，守灯人的灯影仍停在终门之前。"
        }
        _ => "光门把脚下道路换成另一段风声，任务簿也随之翻到当前页。",
    }
}

fn portal_transition_lines(current: MapKind, target: MapKind, quest: &QuestLog) -> Vec<String> {
    let ledger = quest.main_task_ledger();
    let landing = if main_task_target_matches_map(quest.stage(), target) {
        format!(
            "【落点】{}就是当前主线目标地，先找{}。",
            target.def().name,
            ledger.contact
        )
    } else {
        format!(
            "【落点】抵达{}，任务引路仍指向{}。",
            target.def().name,
            ledger.place
        )
    };

    vec![
        format!("【界门】{} -> {}", current.def().name, target.def().name),
        portal_transition_flavor(current, target).to_string(),
        format!(
            "【章程】{} · {} {}/{} · {}",
            quest.chapter_title(),
            ledger.receipt,
            ledger.step,
            ledger.total,
            ledger.action
        ),
        landing,
        format!("【目标】{}", quest.objective()),
    ]
}

fn encounter_zone(kind: MapKind) -> EncounterZone {
    match kind {
        MapKind::Village => EncounterZone::Village,
        MapKind::Bamboo => EncounterZone::Bamboo,
        MapKind::Cave => EncounterZone::Cave,
        MapKind::MoonEchoCorridor => EncounterZone::Cave,
        MapKind::RiverTown => EncounterZone::RiverTown,
        MapKind::RiverReedBed => EncounterZone::RiverTown,
        MapKind::PlagueVillage => EncounterZone::PlagueVillage,
        MapKind::PlagueShrinePath => EncounterZone::PlagueVillage,
        MapKind::Capital => EncounterZone::Capital,
        MapKind::CapitalMansion => EncounterZone::Capital,
        MapKind::MansionMirrorGallery => EncounterZone::Capital,
        MapKind::SouthernRoad => EncounterZone::SouthernRoad,
        MapKind::ThunderDrumPath => EncounterZone::SouthernRoad,
        MapKind::FinalSanctum => EncounterZone::FinalSanctum,
        MapKind::DreamWaterway => EncounterZone::FinalSanctum,
    }
}

fn companion_style(companion: Companion) -> PaperdollStyle {
    match companion {
        Companion::Linger => PaperdollStyle::Linger,
        Companion::SwordSister => PaperdollStyle::Ranger,
        Companion::SpiritWitch => PaperdollStyle::Mystic,
    }
}

fn companion_afterimage_color(companion: Companion) -> Color {
    match companion {
        Companion::Linger => Color::srgba(0.90, 0.70, 1.0, 0.32),
        Companion::SwordSister => Color::srgba(0.62, 0.86, 1.0, 0.34),
        Companion::SpiritWitch => Color::srgba(0.68, 1.0, 0.78, 0.32),
    }
}

fn desired_party_followers(quest: &QuestLog) -> Vec<Companion> {
    let mut followers = Vec::new();
    if quest.has_companion(Companion::Linger) {
        followers.push(Companion::Linger);
    }
    if quest.has_companion(Companion::SwordSister) {
        followers.push(Companion::SwordSister);
    }
    if quest.has_companion(Companion::SpiritWitch) {
        followers.push(Companion::SpiritWitch);
    }
    followers
}

fn follower_slot_position(pos: &PlayerPos, slot: usize, moving: bool, phase: f32) -> Vec3 {
    let base = tile_to_world(pos.col, pos.row);
    let facing = if pos.facing == IVec2::ZERO {
        IVec2::new(0, 1)
    } else {
        pos.facing
    };
    let dir = Vec2::new(facing.x as f32, -(facing.y as f32)).normalize_or_zero();
    let behind = -dir;
    let side_dir = Vec2::new(-dir.y, dir.x);
    let slot_f = slot as f32;
    let side = if slot % 2 == 0 { 1.0 } else { -1.0 };
    let mut p = base + behind * (TILE * (0.72 + slot_f * 0.42)) + side_dir * (TILE * 0.22 * side);
    let bob = if moving {
        (phase * 12.0 + slot_f * 1.4).sin() * 3.0
    } else {
        (phase * 2.4 + slot_f).sin() * 1.4
    };
    p.y += bob;
    Vec3::new(p.x, p.y, 9.4 - slot_f * 0.12)
}

fn follower_faces_left(pos: &PlayerPos) -> bool {
    pos.facing.x < 0
}

// ---------------------------------------------------------------------------
// Resources / components
// ---------------------------------------------------------------------------

#[derive(Resource)]
pub struct PlayerPos {
    pub col: i32,
    pub row: i32,
    pub facing: IVec2,
}

impl Default for PlayerPos {
    fn default() -> Self {
        let (col, row) = MapData::build(MapKind::default()).spawn();
        Self {
            col,
            row,
            facing: IVec2::new(0, 1),
        }
    }
}

#[derive(Resource, Default)]
struct MoveCooldown(f32);

#[derive(Resource, Default)]
pub struct Dialogue {
    pub active: bool,
    lines: Vec<String>,
    idx: usize,
    after: DialogueAfter,
    choice: Option<DialogueChoice>,
    portrait_path: Option<&'static str>,
    chapter_art: Option<ChapterArtAssets>,
}

impl Dialogue {
    fn showing_chapter_art(&self) -> bool {
        if !self.active || self.choice.is_some() {
            return false;
        }
        self.chapter_art.is_some()
            && self
                .lines
                .iter()
                .position(|line| line.starts_with("【卷章展开】"))
                .is_some_and(|start| {
                    self.idx >= start && self.idx < start + CHAPTER_CARD_LINE_COUNT
                })
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct DialogueChoice {
    kind: DialogueChoiceKind,
    selected: usize,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum DialogueChoiceKind {
    BondResponse,
    CampTactic,
    SideBoard {
        options: [Option<SideBoardTaskOption>; MAX_SIDE_BOARD_OPTIONS],
    },
    MainQuest {
        role: QuestRole,
        action: MainQuestChoiceAction,
    },
    SideQuest {
        side: SideQuest,
        action: SideQuestChoiceAction,
    },
    CommissionTrace {
        side: SideQuest,
        name: &'static str,
        active_line: &'static str,
        source: &'static str,
        inactive_line: &'static str,
        repeat_line: &'static str,
    },
    NpcErrand {
        errand: NpcErrand,
        action: NpcErrandChoiceAction,
    },
    CompanionRevisit {
        revisit: CompanionRevisit,
        action: CompanionRevisitChoiceAction,
    },
    RouteDetour {
        detour: RouteDetour,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SideQuestChoiceAction {
    Accept,
    TurnIn,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum NpcErrandChoiceAction {
    Accept,
    TurnIn,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CompanionRevisitChoiceAction {
    Accept,
    TurnIn,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum MainQuestChoiceAction {
    Accept,
    Advance,
    TurnIn,
    Boss,
}

impl MainQuestChoiceAction {
    fn status(self) -> &'static str {
        match self {
            Self::Accept => "可领取",
            Self::Advance => "可推进",
            Self::TurnIn => "可交付",
            Self::Boss => "首领战",
        }
    }

    fn operation(self) -> &'static str {
        match self {
            Self::Accept => "领取主线",
            Self::Advance => "继续主线",
            Self::TurnIn => "交付主线",
            Self::Boss => "迎战首领",
        }
    }

    fn cancel_verb(self) -> &'static str {
        match self {
            Self::Accept => "领取",
            Self::Advance => "推进",
            Self::TurnIn => "交付",
            Self::Boss => "迎战",
        }
    }
}

impl DialogueChoice {
    fn bond_response() -> Self {
        Self {
            kind: DialogueChoiceKind::BondResponse,
            selected: 0,
        }
    }

    fn camp_tactic(default_bonus: CampBonus) -> Self {
        Self {
            kind: DialogueChoiceKind::CampTactic,
            selected: camp_tactic_index(default_bonus),
        }
    }

    fn side_quest(side: SideQuest, action: SideQuestChoiceAction) -> Self {
        Self {
            kind: DialogueChoiceKind::SideQuest { side, action },
            selected: 0,
        }
    }

    fn commission_trace(trace: &CommissionTraceDef) -> Self {
        Self {
            kind: DialogueChoiceKind::CommissionTrace {
                side: trace.side,
                name: trace.name,
                active_line: trace.active_line,
                source: trace.source,
                inactive_line: trace.inactive_line,
                repeat_line: trace.repeat_line,
            },
            selected: 0,
        }
    }

    fn npc_errand(errand: NpcErrand, action: NpcErrandChoiceAction) -> Self {
        Self {
            kind: DialogueChoiceKind::NpcErrand { errand, action },
            selected: 0,
        }
    }

    fn companion_revisit(revisit: CompanionRevisit, action: CompanionRevisitChoiceAction) -> Self {
        Self {
            kind: DialogueChoiceKind::CompanionRevisit { revisit, action },
            selected: 0,
        }
    }

    fn side_board(kind: MapKind, quest: &QuestLog) -> Self {
        Self {
            kind: DialogueChoiceKind::SideBoard {
                options: side_board_task_options(kind, quest),
            },
            selected: 0,
        }
    }

    fn main_quest(role: QuestRole, action: MainQuestChoiceAction) -> Self {
        Self {
            kind: DialogueChoiceKind::MainQuest { role, action },
            selected: 0,
        }
    }

    fn route_detour(detour: RouteDetour) -> Self {
        Self {
            kind: DialogueChoiceKind::RouteDetour { detour },
            selected: 0,
        }
    }

    fn selected_response(self) -> BondResponse {
        match self.selected {
            0 => BondResponse::Courage,
            _ => BondResponse::Tender,
        }
    }

    fn selected_camp_bonus(self) -> CampBonus {
        camp_tactic_bonus(self.selected)
    }

    fn selected_detour_approach(self) -> RouteDetourApproach {
        match self.selected {
            0 => RouteDetourApproach::Scout,
            _ => RouteDetourApproach::PressOn,
        }
    }

    fn selected_side_quest_resolution(self) -> SideQuestResolution {
        match self.selected {
            1 => SideQuestResolution::Pursue,
            _ => SideQuestResolution::Settle,
        }
    }

    fn selected_commission_trace_approach(self) -> SideQuestFieldApproach {
        match self.selected {
            1 => SideQuestFieldApproach::Confront,
            _ => SideQuestFieldApproach::Investigate,
        }
    }

    fn selected_side_board_task(self) -> Option<SideBoardTaskOption> {
        let DialogueChoiceKind::SideBoard { options } = self.kind else {
            return None;
        };

        options
            .get(self.selected)
            .copied()
            .flatten()
            .or_else(|| options.into_iter().flatten().next())
    }

    fn accepted(self) -> bool {
        match self.kind {
            DialogueChoiceKind::SideQuest {
                action: SideQuestChoiceAction::TurnIn,
                ..
            } => self.selected < 2,
            DialogueChoiceKind::CommissionTrace { .. } => self.selected < 2,
            _ => self.selected == 0,
        }
    }

    fn option_count(self) -> usize {
        match self.kind {
            DialogueChoiceKind::CampTactic => 3,
            DialogueChoiceKind::SideBoard { options } => options.into_iter().flatten().count(),
            DialogueChoiceKind::SideQuest {
                action: SideQuestChoiceAction::TurnIn,
                ..
            } => 3,
            DialogueChoiceKind::CommissionTrace { .. } => 3,
            DialogueChoiceKind::BondResponse
            | DialogueChoiceKind::RouteDetour { .. }
            | DialogueChoiceKind::MainQuest { .. }
            | DialogueChoiceKind::SideQuest { .. }
            | DialogueChoiceKind::NpcErrand { .. }
            | DialogueChoiceKind::CompanionRevisit { .. } => 2,
        }
    }

    fn prompt(self) -> String {
        match self.kind {
            DialogueChoiceKind::BondResponse => "你要怎样回应赵灵儿？".to_string(),
            DialogueChoiceKind::CampTactic => "下一场战斗采用哪种营地战术？".to_string(),
            DialogueChoiceKind::SideBoard { .. } => "要查看哪一份委托契约？".to_string(),
            DialogueChoiceKind::RouteDetour { .. } => "要怎样处理这条路线分支？".to_string(),
            DialogueChoiceKind::MainQuest {
                action: MainQuestChoiceAction::Accept,
                ..
            } => "要领取这条主线任务并写入任务簿吗？".to_string(),
            DialogueChoiceKind::MainQuest {
                action: MainQuestChoiceAction::Advance,
                ..
            } => "要推进当前主线并更新目标吗？".to_string(),
            DialogueChoiceKind::MainQuest {
                action: MainQuestChoiceAction::TurnIn,
                ..
            } => "要交付这条主线任务吗？".to_string(),
            DialogueChoiceKind::MainQuest {
                action: MainQuestChoiceAction::Boss,
                ..
            } => "要进入首领战吗？".to_string(),
            DialogueChoiceKind::SideQuest {
                action: SideQuestChoiceAction::Accept,
                ..
            } => "要签下这份委托契约并开始追踪吗？".to_string(),
            DialogueChoiceKind::SideQuest {
                action: SideQuestChoiceAction::TurnIn,
                ..
            } => "要怎样交付这份委托契约？".to_string(),
            DialogueChoiceKind::CommissionTrace { name, .. } => {
                format!("要怎样处理《{name}》这处委托现场？")
            }
            DialogueChoiceKind::NpcErrand {
                action: NpcErrandChoiceAction::Accept,
                ..
            } => "要接下这份 NPC 托付并写入任务簿吗？".to_string(),
            DialogueChoiceKind::NpcErrand {
                action: NpcErrandChoiceAction::TurnIn,
                ..
            } => "要交付这份 NPC 托付吗？".to_string(),
            DialogueChoiceKind::CompanionRevisit {
                action: CompanionRevisitChoiceAction::Accept,
                ..
            } => "要领取这份同伴补访签并开始追踪吗？".to_string(),
            DialogueChoiceKind::CompanionRevisit {
                action: CompanionRevisitChoiceAction::TurnIn,
                ..
            } => "要交付这份同伴补访签吗？".to_string(),
        }
    }

    fn option_label(self, index: usize) -> String {
        match self.kind {
            DialogueChoiceKind::BondResponse => match index {
                0 => "我会走在前面。".to_string(),
                _ => "我们慢慢来。".to_string(),
            },
            DialogueChoiceKind::CampTactic => camp_tactic_bonus(index).tactic_label().to_string(),
            DialogueChoiceKind::RouteDetour { .. } => match index {
                0 => RouteDetourApproach::Scout.action_label().to_string(),
                _ => RouteDetourApproach::PressOn.action_label().to_string(),
            },
            DialogueChoiceKind::SideBoard { options } => options
                .get(index)
                .and_then(|option| *option)
                .map(|option| option.option_label())
                .unwrap_or_else(|| "返回".to_string()),
            DialogueChoiceKind::MainQuest {
                action: MainQuestChoiceAction::Accept,
                ..
            } => match index {
                0 => "领取并追踪".to_string(),
                _ => "先不处理".to_string(),
            },
            DialogueChoiceKind::MainQuest {
                action: MainQuestChoiceAction::Advance,
                ..
            } => match index {
                0 => "继续主线".to_string(),
                _ => "先不处理".to_string(),
            },
            DialogueChoiceKind::MainQuest {
                action: MainQuestChoiceAction::TurnIn,
                ..
            } => match index {
                0 => "交付主线".to_string(),
                _ => "先不处理".to_string(),
            },
            DialogueChoiceKind::MainQuest {
                action: MainQuestChoiceAction::Boss,
                ..
            } => match index {
                0 => "迎战首领".to_string(),
                _ => "先不处理".to_string(),
            },
            DialogueChoiceKind::SideQuest {
                action: SideQuestChoiceAction::Accept,
                ..
            } => match index {
                0 => "领取并追踪".to_string(),
                _ => "先不处理".to_string(),
            },
            DialogueChoiceKind::SideQuest {
                action: SideQuestChoiceAction::TurnIn,
                ..
            } => match index {
                0 => "稳妥封存".to_string(),
                1 => "追查余波".to_string(),
                _ => "先不处理".to_string(),
            },
            DialogueChoiceKind::CommissionTrace { .. } => match index {
                0 => SideQuestFieldApproach::Investigate
                    .action_label()
                    .to_string(),
                1 => SideQuestFieldApproach::Confront.action_label().to_string(),
                _ => "先不处理".to_string(),
            },
            DialogueChoiceKind::NpcErrand {
                action: NpcErrandChoiceAction::Accept,
                ..
            } => match index {
                0 => "接下托付".to_string(),
                _ => "先不处理".to_string(),
            },
            DialogueChoiceKind::NpcErrand {
                action: NpcErrandChoiceAction::TurnIn,
                ..
            } => match index {
                0 => "交付托付".to_string(),
                _ => "先不处理".to_string(),
            },
            DialogueChoiceKind::CompanionRevisit {
                action: CompanionRevisitChoiceAction::Accept,
                ..
            } => match index {
                0 => "领取补访签".to_string(),
                _ => "先不处理".to_string(),
            },
            DialogueChoiceKind::CompanionRevisit {
                action: CompanionRevisitChoiceAction::TurnIn,
                ..
            } => match index {
                0 => "交付补访签".to_string(),
                _ => "先不处理".to_string(),
            },
        }
    }
}

fn camp_tactic_bonus(index: usize) -> CampBonus {
    match index {
        1 => CampBonus::Focus,
        2 => CampBonus::Vigil,
        _ => CampBonus::Warmth,
    }
}

fn camp_tactic_index(bonus: CampBonus) -> usize {
    match bonus {
        CampBonus::Warmth => 0,
        CampBonus::Focus => 1,
        CampBonus::Vigil => 2,
    }
}

#[derive(Clone, Copy, Default)]
enum DialogueAfter {
    #[default]
    None,
    StartBattle(PendingEncounter),
}

#[derive(Component)]
struct PlayerSprite;

#[derive(Component)]
struct PlayerLight;

#[derive(Component)]
struct PartyFollower {
    companion: Companion,
    trail_timer: f32,
}

#[derive(Component, Clone, Copy)]
struct LayeredNpcPart {
    part: CutoutPart,
    origin: Vec3,
    base_offset: Vec2,
    base_size: Vec2,
    phase: f32,
}

#[derive(Component, Clone, Copy)]
struct CharacterGroundShadow {
    owner: Option<Entity>,
    origin: Vec3,
    offset: Vec2,
    base_size: Vec2,
    base_alpha: f32,
    phase: f32,
    speed: f32,
    z: f32,
    track_scene_motion: bool,
}

#[derive(Component, Clone, Copy)]
struct CharacterAfterimage {
    age: f32,
    duration: f32,
    base_color: Color,
    velocity: Vec2,
    start_scale: Vec3,
    end_scale: Vec3,
}

#[derive(Component)]
pub struct MapContent;

#[derive(Component)]
struct HudText;

#[derive(Component)]
struct TaskTrackerRoot;

#[derive(Component)]
struct TaskTrackerText;

#[derive(Component)]
struct AreaBanner {
    age: f32,
    duration: f32,
    base_color: Color,
}

impl AreaBanner {
    fn new(base_color: Color) -> Self {
        Self {
            age: 0.0,
            duration: 2.8,
            base_color,
        }
    }
}

#[derive(Component)]
struct TaskPromptRoot;

#[derive(Component)]
struct TaskPromptLine;

#[derive(Component)]
struct QuestNoticeRoot;

#[derive(Component)]
struct QuestNoticeTitle;

#[derive(Component)]
struct QuestNoticeBody;

#[derive(Component)]
struct DialogueRoot;

#[derive(Component)]
struct DialogueLine;

#[derive(Component)]
struct DialoguePortrait;

#[derive(Component)]
struct ChapterArtRoot {
    layout: Handle<TextureAtlasLayout>,
    timer: f32,
    frame: usize,
    active_sheet: Option<&'static str>,
}

#[derive(Component, Clone, Copy)]
struct QuestMarker {
    kind: QuestMarkerKind,
    base_y: f32,
    phase: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum QuestMarkerKind {
    Role(QuestRole),
    SideBoard(MapKind),
    SideContact(MapKind),
    NpcErrandOffer(NpcErrand),
    NpcErrandDelivery(NpcErrand),
    CompanionRevisitGiver(CompanionRevisit),
    CompanionRevisitField(CompanionRevisit),
    CommissionTrace(SideQuest),
    Lamp(FinalLamp),
    RiverLantern(RiverLantern),
    PlagueWard(PlagueWard),
    MansionMirror(MansionMirrorNode),
    ThunderDrum(ThunderDrum),
    Crystal(MoonCrystal),
    RouteMark(RouteMark),
    RouteDetour(RouteDetour),
    FieldSupply(FieldSupply),
}

#[derive(Component, Clone, Copy)]
enum QuestMarkerPart {
    Glow,
    Badge,
    GlyphShadow,
    Glyph,
}

#[derive(Component)]
struct AmbientParticle {
    kind: AmbientKind,
    origin: Vec3,
    phase: f32,
    age: f32,
    speed: f32,
    amplitude: Vec2,
    color: Color,
}

#[derive(Component, Clone, Copy)]
struct SceneMotion {
    kind: SceneMotionKind,
    origin: Vec3,
    phase: f32,
    base_color: Color,
}

impl SceneMotion {
    fn new(kind: SceneMotionKind, origin: Vec3, phase: f32) -> Self {
        Self {
            kind,
            origin,
            phase,
            base_color: Color::WHITE,
        }
    }

    fn with_color(mut self, color: Color) -> Self {
        self.base_color = color;
        self
    }
}

#[derive(Component, Clone, Copy)]
struct SceneLightPulse {
    kind: SceneMotionKind,
    origin: Vec3,
    phase: f32,
    base_color: Color,
    base_size: f32,
    base_radius: f32,
    base_intensity: f32,
}

impl SceneLightPulse {
    fn new(kind: SceneMotionKind, origin: Vec3, phase: f32, size: f32, color: Color) -> Self {
        let srgba = color.to_srgba();
        Self {
            kind,
            origin,
            phase,
            base_color: color,
            base_size: size,
            base_radius: size * 0.72,
            base_intensity: 0.65 + srgba.alpha * 3.0,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SceneMotionKind {
    Npc,
    QuestBoard,
    SpiritLantern,
    Crystal,
    Shrine,
    Gate,
}

#[derive(Clone, Copy)]
struct SceneMotionProfile {
    y_amp: f32,
    x_amp: f32,
    scale_amp: f32,
    rotation_amp: f32,
    speed: f32,
    brightness_amp: f32,
    alpha_amp: f32,
    light_radius_amp: f32,
    light_intensity_amp: f32,
}

#[derive(Clone, Copy, Debug)]
struct CharacterBodyMotion {
    offset: Vec2,
    scale: Vec2,
    rotation: f32,
    brightness: f32,
}

#[derive(Clone, Copy, Debug)]
struct CharacterShadowFrame {
    scale: Vec3,
    alpha: f32,
}

#[derive(Clone, Copy, Debug)]
struct CharacterAfterimageFrame {
    progress: f32,
    alpha_scale: f32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum AmbientKind {
    Firefly,
    Leaf,
    Mist,
    Rain,
    Spark,
}

#[derive(Clone, Copy)]
struct AmbientProfile {
    kind: AmbientKind,
    count: usize,
    size: Vec2,
    color: Color,
    speed: f32,
    amplitude: Vec2,
}

struct QuestMarkerStyle {
    badge: Color,
    glow: Color,
    glyph: Color,
}

#[derive(SystemParam)]
struct MapSpawnParams<'w, 's> {
    font: Res<'w, GameFont>,
    asset_server: Res<'w, AssetServer>,
    dolls: Res<'w, PaperdollAssets>,
    lights: Res<'w, LightingAssets>,
    anims: Res<'w, AnimationAssets>,
    explore_assets: Res<'w, ExploreAssets>,
    meshes: ResMut<'w, Assets<Mesh>>,
    terrain_materials: ResMut<'w, Assets<TerrainMaterial>>,
    images: ResMut<'w, Assets<Image>>,
    map_content: Query<'w, 's, Entity, With<MapContent>>,
    area_banners: Query<'w, 's, Entity, With<AreaBanner>>,
    fog_memory: ResMut<'w, fog::FogMemory>,
}

pub struct ExplorePlugin;

impl Plugin for ExplorePlugin {
    fn build(&self, app: &mut App) {
        let assets = app.world().resource::<AssetServer>().clone();
        app.add_plugins(Material2dPlugin::<TerrainMaterial>::default())
            .init_resource::<PlayerPos>()
            .init_resource::<CurrentMap>()
            .init_resource::<MoveCooldown>()
            .init_resource::<Dialogue>()
            .init_resource::<QuestNotice>()
            .insert_resource(ExploreAssets::load(&assets))
            .add_systems(OnEnter(AppState::Explore), spawn_explore)
            .add_systems(
                Update,
                (
                    explore_input,
                    sync_player_transform,
                    sync_party_followers,
                    update_scene_motions,
                    update_layered_npc_parts,
                    update_character_shadows,
                    update_character_afterimages,
                    update_ambient_particles,
                    update_area_banners,
                    update_quest_notice,
                    update_hud,
                    update_task_tracker,
                    update_task_prompt,
                    update_quest_markers,
                    update_dialogue_ui,
                )
                    .chain()
                    .run_if(in_state(AppState::Explore)),
            );
    }
}

// ---------------------------------------------------------------------------
// Scene setup
// ---------------------------------------------------------------------------

fn spawn_explore(
    mut commands: Commands,
    font: Res<GameFont>,
    asset_server: Res<AssetServer>,
    mut pos: ResMut<PlayerPos>,
    current: Res<CurrentMap>,
    quest: Res<QuestLog>,
    explore_assets: Res<ExploreAssets>,
    dolls: Res<PaperdollAssets>,
    lights: Res<LightingAssets>,
    anims: Res<AnimationAssets>,
    mut layouts: ResMut<Assets<TextureAtlasLayout>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut terrain_materials: ResMut<Assets<TerrainMaterial>>,
    mut images: ResMut<Assets<Image>>,
    mut fog_memory: ResMut<fog::FogMemory>,
) {
    let map = MapData::build(current.0);
    if !map.at(pos.col, pos.row).walkable() {
        move_player_to_spawn(&map, &mut pos);
    }
    fog_memory.clear();
    spawn_map_content(
        &mut commands,
        &font,
        &asset_server,
        &pos,
        &dolls,
        &lights,
        &anims,
        &explore_assets,
        &map,
        &mut meshes,
        &mut terrain_materials,
        &mut images,
    );
    spawn_area_banner(&mut commands, &font, &map, &quest);
    commands.insert_resource(map);

    // --- HUD (top-left stats) ---
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(12.0),
                left: Val::Px(12.0),
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.55)),
            DespawnOnExit(AppState::Explore),
        ))
        .with_children(|p| {
            p.spawn((
                HudText,
                Text::new(""),
                font.text_font(18.0),
                TextColor(Color::WHITE),
            ));
        });

    // --- Task tracker (top-right, always shows main objective plus active commission) ---
    commands
        .spawn((
            TaskTrackerRoot,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(188.0),
                right: Val::Px(18.0),
                width: Val::Px(430.0),
                min_height: Val::Px(132.0),
                padding: UiRect::axes(Val::Px(14.0), Val::Px(10.0)),
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.04, 0.07, 0.08, 0.84)),
            BorderColor::all(Color::srgba(0.54, 0.94, 0.68, 0.68)),
            Visibility::Inherited,
            DespawnOnExit(AppState::Explore),
        ))
        .with_children(|p| {
            p.spawn((
                TaskTrackerText,
                Text::new(""),
                font.text_font(14.0),
                TextColor(Color::srgb(0.88, 1.0, 0.84)),
            ));
        });

    // --- Hint (bottom, above dialogue) ---
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(12.0),
                left: Val::Px(12.0),
                ..default()
            },
            DespawnOnExit(AppState::Explore),
        ))
        .with_children(|p| {
            p.spawn((
                Text::new("方向键/WASD 移动 · 空格 交谈/接取/交付 · 踏入草丛恐遇妖兽"),
                font.text_font(16.0),
                TextColor(Color::srgb(0.9, 0.9, 0.7)),
            ));
        });

    // --- Context prompt (hidden until facing an interactable task board) ---
    commands
        .spawn((
            TaskPromptRoot,
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(42.0),
                left: Val::Px(12.0),
                max_width: Val::Px(620.0),
                padding: UiRect::axes(Val::Px(12.0), Val::Px(8.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.04, 0.04, 0.10, 0.82)),
            Visibility::Hidden,
            DespawnOnExit(AppState::Explore),
        ))
        .with_children(|p| {
            p.spawn((
                TaskPromptLine,
                Text::new(""),
                font.text_font(16.0),
                TextColor(Color::srgb(1.0, 0.96, 0.72)),
            ));
        });

    // --- Quest notice (hidden until the quest log changes) ---
    commands
        .spawn((
            QuestNoticeRoot,
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(122.0),
                left: Val::Percent(29.0),
                right: Val::Percent(13.0),
                padding: UiRect::axes(Val::Px(18.0), Val::Px(12.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.04, 0.05, 0.10, 0.0)),
            BorderColor::all(Color::srgba(0.92, 0.78, 0.38, 0.0)),
            Visibility::Hidden,
            DespawnOnExit(AppState::Explore),
        ))
        .with_children(|panel| {
            panel.spawn((
                QuestNoticeTitle,
                Text::new(""),
                font.text_font(20.0),
                TextColor(Color::srgb(1.0, 0.91, 0.58)),
            ));
            panel.spawn((
                QuestNoticeBody,
                Text::new(""),
                font.text_font(15.0),
                TextColor(Color::srgb(0.88, 0.94, 1.0)),
            ));
        });

    // --- Chapter art (only visible while a one-time chapter card is being read) ---
    let chapter_art_layout = layouts.add(TextureAtlasLayout::from_grid(
        UVec2::new(640, 352),
        8,
        4,
        None,
        None,
    ));
    commands.spawn((
        ChapterArtRoot {
            layout: chapter_art_layout,
            timer: 0.0,
            frame: 0,
            active_sheet: None,
        },
        Node {
            position_type: PositionType::Absolute,
            left: Val::Px(0.0),
            right: Val::Px(0.0),
            top: Val::Px(0.0),
            bottom: Val::Px(0.0),
            ..default()
        },
        ImageNode::solid_color(Color::NONE),
        Visibility::Hidden,
        GlobalZIndex(42),
        DespawnOnExit(AppState::Explore),
    ));

    // --- Dialogue box (hidden until active) ---
    commands
        .spawn((
            DialogueRoot,
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(40.0),
                left: Val::Px(60.0),
                right: Val::Px(60.0),
                min_height: Val::Px(110.0),
                padding: UiRect::all(Val::Px(18.0)),
                flex_direction: FlexDirection::Row,
                align_items: AlignItems::Center,
                column_gap: Val::Px(18.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.05, 0.05, 0.12, 0.92)),
            Visibility::Hidden,
            GlobalZIndex(55),
            DespawnOnExit(AppState::Explore),
        ))
        .with_children(|p| {
            p.spawn((
                DialoguePortrait,
                Node {
                    width: Val::Px(132.0),
                    height: Val::Px(132.0),
                    flex_shrink: 0.0,
                    ..default()
                },
                ImageNode::solid_color(Color::NONE),
                Visibility::Hidden,
            ));
            p.spawn((Node {
                flex_direction: FlexDirection::Column,
                flex_grow: 1.0,
                ..default()
            },))
                .with_children(|p| {
                    p.spawn((
                        DialogueLine,
                        Text::new(""),
                        font.text_font(22.0),
                        TextColor(Color::WHITE),
                    ));
                });
        });
}

fn spawn_area_banner(commands: &mut Commands, font: &GameFont, map: &MapData, quest: &QuestLog) {
    let [title, route, objective] = area_banner_lines(map, quest);
    let panel_color = Color::srgba(0.04, 0.05, 0.10, 0.86);

    commands
        .spawn((
            AreaBanner::new(panel_color),
            Node {
                position_type: PositionType::Absolute,
                top: Val::Px(14.0),
                left: Val::Px(350.0),
                right: Val::Px(80.0),
                padding: UiRect::axes(Val::Px(18.0), Val::Px(12.0)),
                flex_direction: FlexDirection::Column,
                row_gap: Val::Px(4.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border: UiRect::all(Val::Px(1.0)),
                ..default()
            },
            BackgroundColor(panel_color),
            BorderColor::all(Color::srgba(0.72, 0.56, 1.0, 0.70)),
            DespawnOnExit(AppState::Explore),
        ))
        .with_children(|panel| {
            panel.spawn((
                Text::new(title),
                font.text_font(24.0),
                TextColor(Color::srgb(1.0, 0.92, 0.66)),
            ));
            panel.spawn((
                Text::new(route),
                font.text_font(17.0),
                TextColor(Color::srgb(0.74, 0.92, 1.0)),
            ));
            panel.spawn((
                Text::new(objective),
                font.text_font(16.0),
                TextColor(Color::srgb(0.86, 0.86, 0.92)),
            ));
        });
}

#[allow(clippy::too_many_arguments)]
fn spawn_map_content(
    commands: &mut Commands,
    font: &GameFont,
    asset_server: &AssetServer,
    pos: &PlayerPos,
    dolls: &PaperdollAssets,
    lights: &LightingAssets,
    anims: &AnimationAssets,
    explore_assets: &ExploreAssets,
    map: &MapData,
    meshes: &mut Assets<Mesh>,
    terrain_materials: &mut Assets<TerrainMaterial>,
    images: &mut Assets<Image>,
) {
    let terrain = spawn_terrain_map(
        commands,
        meshes,
        terrain_materials,
        images,
        explore_assets,
        map,
        1.0,
        AppState::Explore,
    );
    commands.entity(terrain.backdrop).insert(MapContent);
    commands.entity(terrain.surface).insert(MapContent);

    for row in 0..MAP_H {
        for col in 0..MAP_W {
            let tile = map.at(col, row);
            let p = tile_to_world(col, row);
            if tile == Tile::Wall {
                commands.spawn((
                    MapContent,
                    Occluder2d::rectangle(TILE * 0.92, TILE * 0.92).with_opacity(0.82),
                    Transform::from_xyz(p.x, p.y, 0.1),
                    DespawnOnExit(AppState::Explore),
                ));
            }

            if tile == Tile::Portal {
                let origin = Vec3::new(p.x, p.y, 3.0);
                let phase = scene_phase(col, row);
                let portal_color = Color::srgba(0.45, 0.86, 1.0, 0.56);
                commands.spawn((
                    MapContent,
                    SceneMotion::new(SceneMotionKind::Gate, origin, phase).with_color(portal_color),
                    Sprite {
                        image: lights.orb.clone(),
                        color: portal_color,
                        custom_size: Some(Vec2::splat(TILE * 1.8)),
                        ..default()
                    },
                    Transform::from_translation(origin),
                    DespawnOnExit(AppState::Explore),
                ));
                let portal_light = lighting::spawn_light(
                    commands,
                    lights,
                    Vec3::new(p.x, p.y, 2.7),
                    TILE * 3.6,
                    Color::srgba(0.45, 0.86, 1.0, 0.28),
                    AppState::Explore,
                );
                commands.entity(portal_light).insert((
                    MapContent,
                    SceneLightPulse::new(
                        SceneMotionKind::Gate,
                        Vec3::new(p.x, p.y, 2.7),
                        phase,
                        TILE * 3.6,
                        Color::srgba(0.45, 0.86, 1.0, 0.28),
                    ),
                ));
                commands.spawn((
                    MapContent,
                    Text2d::new(">"),
                    font.text_font(22.0),
                    TextColor(Color::srgb(0.75, 0.95, 1.0)),
                    Transform::from_xyz(p.x + 1.0, p.y + 2.0, 4.0),
                    DespawnOnExit(AppState::Explore),
                ));
            }
        }
    }

    for prop in prop_defs(map.kind) {
        spawn_prop(commands, font, asset_server, lights, map.kind, prop);
    }

    for supply in field_supply_defs(map.kind) {
        spawn_field_supply(commands, font, asset_server, lights, supply);
    }

    for trace in commission_trace_defs(map.kind) {
        spawn_commission_trace(commands, font, asset_server, lights, trace);
    }

    for npc in npc_defs(map.kind) {
        spawn_npc(commands, font, asset_server, dolls, lights, map.kind, npc);
    }

    spawn_map_ambience(commands, lights, map.kind);

    let pp = tile_to_world(pos.col, pos.row);
    let player = animation::spawn_animated_sprite(
        commands,
        anims,
        AnimationClip::HeroIdle,
        Vec3::new(pp.x, pp.y, 10.0),
        Vec2::splat(82.0),
        AppState::Explore,
    );
    commands.entity(player).insert((MapContent, PlayerSprite));
    spawn_owned_character_shadow(
        commands,
        lights,
        player,
        Vec3::new(pp.x, pp.y, 10.0),
        Vec2::new(0.0, -32.0),
        Vec2::new(58.0, 14.0),
        0.35,
        0.28,
        5.4,
        4.35,
    );
    let player_light = lighting::spawn_light(
        commands,
        lights,
        Vec3::new(pp.x, pp.y, 4.0),
        TILE * 4.1,
        Color::srgba(1.0, 0.86, 0.48, 0.36),
        AppState::Explore,
    );
    commands
        .entity(player_light)
        .insert((MapContent, PlayerLight));

    fog::spawn_explore_fog(commands);
}

#[allow(clippy::too_many_arguments)]
fn spawn_owned_character_shadow(
    commands: &mut Commands,
    lights: &LightingAssets,
    owner: Entity,
    origin: Vec3,
    offset: Vec2,
    base_size: Vec2,
    phase: f32,
    base_alpha: f32,
    speed: f32,
    z: f32,
) {
    spawn_character_shadow(
        commands,
        lights,
        CharacterGroundShadow {
            owner: Some(owner),
            origin,
            offset,
            base_size,
            base_alpha,
            phase,
            speed,
            z,
            track_scene_motion: false,
        },
    );
}

fn spawn_scene_character_shadow(
    commands: &mut Commands,
    lights: &LightingAssets,
    origin: Vec3,
    offset: Vec2,
    base_size: Vec2,
    phase: f32,
    base_alpha: f32,
) {
    spawn_character_shadow(
        commands,
        lights,
        CharacterGroundShadow {
            owner: None,
            origin,
            offset,
            base_size,
            base_alpha,
            phase,
            speed: 2.25,
            z: 4.05,
            track_scene_motion: true,
        },
    );
}

fn spawn_character_shadow(
    commands: &mut Commands,
    lights: &LightingAssets,
    shadow: CharacterGroundShadow,
) {
    commands.spawn((
        MapContent,
        shadow,
        Sprite {
            image: lights.orb.clone(),
            color: Color::srgba(0.0, 0.0, 0.0, shadow.base_alpha),
            custom_size: Some(shadow.base_size),
            ..default()
        },
        Transform::from_translation(Vec3::new(
            shadow.origin.x + shadow.offset.x,
            shadow.origin.y + shadow.offset.y,
            shadow.z,
        )),
        DespawnOnExit(AppState::Explore),
    ));
}

fn spawn_sprite_afterimage(
    commands: &mut Commands,
    source: &Sprite,
    source_transform: &Transform,
    tint: Color,
    duration: f32,
    velocity: Vec2,
) {
    let mut sprite = source.clone();
    sprite.color = tint;
    let mut transform = source_transform.clone();
    transform.translation.z -= 0.42;

    commands.spawn((
        MapContent,
        sprite,
        transform,
        CharacterAfterimage {
            age: 0.0,
            duration,
            base_color: tint,
            velocity,
            start_scale: source_transform.scale,
            end_scale: Vec3::new(
                source_transform.scale.x * 1.08,
                source_transform.scale.y * 1.03,
                source_transform.scale.z,
            ),
        },
        DespawnOnExit(AppState::Explore),
    ));
}

fn spawn_map_ambience(commands: &mut Commands, lights: &LightingAssets, kind: MapKind) {
    let profile = ambient_profile(kind);
    let span_x = MAP_W as f32 * TILE;
    let span_y = MAP_H as f32 * TILE;
    let left = -span_x * 0.5;
    let bottom = -span_y * 0.5;

    for index in 0..profile.count {
        let i = index as u32;
        let x = left + 20.0 + ((i * 83 + ambience_seed(kind)) % (span_x as u32 - 40)) as f32;
        let y = bottom + 20.0 + ((i * 137 + ambience_seed(kind) * 3) % (span_y as u32 - 40)) as f32;
        let phase = index as f32 * 0.73 + ambience_seed(kind) as f32 * 0.011;
        let z = match profile.kind {
            AmbientKind::Mist => 5.05,
            AmbientKind::Rain => 8.35,
            AmbientKind::Leaf => 5.85,
            AmbientKind::Firefly | AmbientKind::Spark => 6.2,
        };
        let mut transform = Transform::from_xyz(x, y, z);
        if profile.kind == AmbientKind::Rain {
            transform.rotation = Quat::from_rotation_z(-0.18);
        } else if profile.kind == AmbientKind::Leaf {
            transform.rotation = Quat::from_rotation_z(phase.sin() * 0.55);
        }

        let sprite = match profile.kind {
            AmbientKind::Rain => Sprite::from_color(profile.color, profile.size),
            AmbientKind::Leaf => Sprite::from_color(profile.color, profile.size),
            AmbientKind::Mist | AmbientKind::Firefly | AmbientKind::Spark => Sprite {
                image: lights.orb.clone(),
                color: profile.color,
                custom_size: Some(profile.size),
                ..default()
            },
        };

        commands.spawn((
            MapContent,
            AmbientParticle {
                kind: profile.kind,
                origin: Vec3::new(x, y, z),
                phase,
                age: phase * 0.19,
                speed: profile.speed,
                amplitude: profile.amplitude,
                color: profile.color,
            },
            sprite,
            transform,
            DespawnOnExit(AppState::Explore),
        ));
    }
}

fn ambience_seed(kind: MapKind) -> u32 {
    match kind {
        MapKind::Village => 17,
        MapKind::Bamboo => 31,
        MapKind::Cave => 43,
        MapKind::MoonEchoCorridor => 47,
        MapKind::RiverTown => 59,
        MapKind::RiverReedBed => 67,
        MapKind::PlagueVillage => 79,
        MapKind::PlagueShrinePath => 89,
        MapKind::Capital => 83,
        MapKind::CapitalMansion => 97,
        MapKind::MansionMirrorGallery => 101,
        MapKind::SouthernRoad => 109,
        MapKind::ThunderDrumPath => 113,
        MapKind::FinalSanctum => 127,
        MapKind::DreamWaterway => 131,
    }
}

fn ambient_profile(kind: MapKind) -> AmbientProfile {
    match kind {
        MapKind::Village => AmbientProfile {
            kind: AmbientKind::Firefly,
            count: 18,
            size: Vec2::splat(18.0),
            color: Color::srgba(1.0, 0.82, 0.32, 0.32),
            speed: 1.15,
            amplitude: Vec2::new(14.0, 10.0),
        },
        MapKind::Bamboo => AmbientProfile {
            kind: AmbientKind::Leaf,
            count: 30,
            size: Vec2::new(10.0, 4.0),
            color: Color::srgba(0.64, 0.92, 0.34, 0.42),
            speed: 0.72,
            amplitude: Vec2::new(24.0, 16.0),
        },
        MapKind::Cave => AmbientProfile {
            kind: AmbientKind::Mist,
            count: 20,
            size: Vec2::splat(72.0),
            color: Color::srgba(0.48, 0.62, 1.0, 0.12),
            speed: 0.42,
            amplitude: Vec2::new(18.0, 8.0),
        },
        MapKind::MoonEchoCorridor => AmbientProfile {
            kind: AmbientKind::Mist,
            count: 28,
            size: Vec2::splat(70.0),
            color: Color::srgba(0.44, 0.70, 1.0, 0.15),
            speed: 0.48,
            amplitude: Vec2::new(26.0, 10.0),
        },
        MapKind::RiverTown => AmbientProfile {
            kind: AmbientKind::Firefly,
            count: 22,
            size: Vec2::splat(16.0),
            color: Color::srgba(0.40, 0.86, 1.0, 0.28),
            speed: 0.96,
            amplitude: Vec2::new(20.0, 9.0),
        },
        MapKind::RiverReedBed => AmbientProfile {
            kind: AmbientKind::Leaf,
            count: 34,
            size: Vec2::new(12.0, 4.0),
            color: Color::srgba(0.78, 0.86, 0.36, 0.45),
            speed: 0.84,
            amplitude: Vec2::new(34.0, 18.0),
        },
        MapKind::PlagueVillage => AmbientProfile {
            kind: AmbientKind::Rain,
            count: 48,
            size: Vec2::new(2.0, 22.0),
            color: Color::srgba(0.62, 0.84, 0.68, 0.36),
            speed: 0.82,
            amplitude: Vec2::new(18.0, 0.0),
        },
        MapKind::PlagueShrinePath => AmbientProfile {
            kind: AmbientKind::Mist,
            count: 34,
            size: Vec2::splat(64.0),
            color: Color::srgba(0.46, 0.86, 0.58, 0.14),
            speed: 0.50,
            amplitude: Vec2::new(24.0, 10.0),
        },
        MapKind::Capital => AmbientProfile {
            kind: AmbientKind::Spark,
            count: 24,
            size: Vec2::splat(14.0),
            color: Color::srgba(0.86, 0.76, 1.0, 0.26),
            speed: 0.68,
            amplitude: Vec2::new(12.0, 14.0),
        },
        MapKind::CapitalMansion => AmbientProfile {
            kind: AmbientKind::Mist,
            count: 26,
            size: Vec2::splat(58.0),
            color: Color::srgba(0.64, 0.68, 1.0, 0.12),
            speed: 0.50,
            amplitude: Vec2::new(24.0, 10.0),
        },
        MapKind::MansionMirrorGallery => AmbientProfile {
            kind: AmbientKind::Spark,
            count: 30,
            size: Vec2::splat(13.0),
            color: Color::srgba(0.58, 0.78, 1.0, 0.30),
            speed: 0.70,
            amplitude: Vec2::new(18.0, 18.0),
        },
        MapKind::SouthernRoad => AmbientProfile {
            kind: AmbientKind::Spark,
            count: 32,
            size: Vec2::splat(16.0),
            color: Color::srgba(0.74, 1.0, 0.42, 0.30),
            speed: 0.92,
            amplitude: Vec2::new(18.0, 20.0),
        },
        MapKind::ThunderDrumPath => AmbientProfile {
            kind: AmbientKind::Spark,
            count: 38,
            size: Vec2::splat(18.0),
            color: Color::srgba(0.68, 0.94, 1.0, 0.34),
            speed: 1.04,
            amplitude: Vec2::new(24.0, 24.0),
        },
        MapKind::FinalSanctum => AmbientProfile {
            kind: AmbientKind::Mist,
            count: 30,
            size: Vec2::splat(76.0),
            color: Color::srgba(0.50, 0.58, 1.0, 0.14),
            speed: 0.46,
            amplitude: Vec2::new(26.0, 12.0),
        },
        MapKind::DreamWaterway => AmbientProfile {
            kind: AmbientKind::Mist,
            count: 36,
            size: Vec2::splat(82.0),
            color: Color::srgba(0.48, 0.64, 1.0, 0.16),
            speed: 0.44,
            amplitude: Vec2::new(30.0, 14.0),
        },
    }
}

const TERRAIN_SOURCE_SIZE: f32 = 512.0;
// A prime-sized sample de-correlates common AI-art divisions (64/128/256px)
// from the 40-unit gameplay grid. The centered crop also discards vignette
// edges that do not belong in a repeating terrain field.
const TERRAIN_SAMPLE_SIZE: f32 = 61.0;
const TERRAIN_SAMPLES_PER_AXIS: i32 = 7;
const TERRAIN_SOURCE_INSET: f32 =
    (TERRAIN_SOURCE_SIZE - TERRAIN_SAMPLE_SIZE * TERRAIN_SAMPLES_PER_AXIS as f32) * 0.5;

#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct TerrainSample {
    pub rect: Rect,
    pub flip_x: bool,
    pub flip_y: bool,
}

fn terrain_axis_sample(index: i32) -> (i32, bool) {
    let phase = index.rem_euclid(TERRAIN_SAMPLES_PER_AXIS * 2);
    if phase < TERRAIN_SAMPLES_PER_AXIS {
        (phase, false)
    } else {
        (TERRAIN_SAMPLES_PER_AXIS * 2 - 1 - phase, true)
    }
}

/// Samples terrain in world space using a mirrored 7-cell sweep. Reversing
/// both the source index and the sprite orientation keeps every shared edge
/// continuous, including textures whose outer edges are not perfectly tiled.
pub(crate) fn terrain_sample(col: i32, row: i32) -> TerrainSample {
    let (source_col, flip_x) = terrain_axis_sample(col);
    let (source_row, flip_y) = terrain_axis_sample(row);
    let x0 = TERRAIN_SOURCE_INSET + source_col as f32 * TERRAIN_SAMPLE_SIZE;
    let y0 = TERRAIN_SOURCE_INSET + source_row as f32 * TERRAIN_SAMPLE_SIZE;
    debug_assert!(x0 + TERRAIN_SAMPLE_SIZE <= TERRAIN_SOURCE_SIZE);
    debug_assert!(y0 + TERRAIN_SAMPLE_SIZE <= TERRAIN_SOURCE_SIZE);

    TerrainSample {
        rect: Rect::new(x0, y0, x0 + TERRAIN_SAMPLE_SIZE, y0 + TERRAIN_SAMPLE_SIZE),
        flip_x,
        flip_y,
    }
}

pub(crate) fn tile_sprite(
    tile: Tile,
    kind: MapKind,
    assets: &ExploreAssets,
    col: i32,
    row: i32,
) -> Sprite {
    let (image, color) = match tile {
        Tile::Wall => match kind {
            MapKind::Village => (assets.forest.clone(), Color::srgb(0.22, 0.43, 0.23)),
            MapKind::Bamboo => (assets.bamboo_thicket.clone(), Color::srgb(0.32, 0.58, 0.32)),
            MapKind::Cave => (assets.moon_cave_wall.clone(), Color::srgb(0.43, 0.40, 0.58)),
            MapKind::MoonEchoCorridor => {
                (assets.moon_cave_wall.clone(), Color::srgb(0.30, 0.38, 0.62))
            }
            MapKind::RiverTown => (assets.shrine_floor.clone(), Color::srgb(0.42, 0.45, 0.38)),
            MapKind::RiverReedBed => (assets.reed_wall.clone(), Color::srgb(0.88, 0.95, 0.90)),
            MapKind::PlagueVillage => (assets.plague_wall.clone(), Color::srgb(0.92, 0.88, 0.92)),
            MapKind::PlagueShrinePath => {
                (assets.plague_wall.clone(), Color::srgb(0.78, 0.76, 0.84))
            }
            MapKind::Capital => (assets.capital_wall.clone(), Color::srgb(0.94, 0.92, 0.94)),
            MapKind::CapitalMansion => (assets.mansion_wall.clone(), Color::srgb(0.92, 0.90, 0.92)),
            MapKind::MansionMirrorGallery => {
                (assets.mirror_wall.clone(), Color::srgb(0.90, 0.94, 1.0))
            }
            MapKind::SouthernRoad => (assets.south_wall.clone(), Color::srgb(0.90, 0.96, 0.90)),
            MapKind::ThunderDrumPath => (assets.thunder_wall.clone(), Color::srgb(0.90, 0.94, 1.0)),
            MapKind::FinalSanctum => (assets.abyss_wall.clone(), Color::srgb(0.94, 0.90, 1.0)),
            MapKind::DreamWaterway => (assets.dream_wall.clone(), Color::srgb(0.88, 0.96, 1.0)),
        },
        Tile::Water => (assets.water.clone(), Color::srgb(0.50, 0.78, 1.0)),
        Tile::Grass => match kind {
            MapKind::Village => (assets.grass.clone(), Color::srgb(0.72, 0.95, 0.62)),
            MapKind::MoonEchoCorridor => {
                (assets.mystic_grass.clone(), Color::srgb(0.48, 0.66, 0.90))
            }
            MapKind::RiverTown => (assets.mystic_grass.clone(), Color::srgb(0.66, 0.82, 0.58)),
            MapKind::RiverReedBed => (assets.mystic_grass.clone(), Color::srgb(0.70, 0.84, 0.48)),
            MapKind::PlagueVillage => (assets.mystic_grass.clone(), Color::srgb(0.54, 0.66, 0.42)),
            MapKind::PlagueShrinePath => {
                (assets.mystic_grass.clone(), Color::srgb(0.42, 0.60, 0.36))
            }
            MapKind::Capital => (assets.shrine_floor.clone(), Color::srgb(0.64, 0.62, 0.76)),
            MapKind::CapitalMansion => (assets.shrine_floor.clone(), Color::srgb(0.56, 0.58, 0.82)),
            MapKind::MansionMirrorGallery => {
                (assets.shrine_floor.clone(), Color::srgb(0.48, 0.58, 0.88))
            }
            MapKind::SouthernRoad => (assets.mystic_grass.clone(), Color::srgb(0.66, 0.88, 0.50)),
            MapKind::ThunderDrumPath => {
                (assets.mystic_grass.clone(), Color::srgb(0.56, 0.76, 0.58))
            }
            MapKind::FinalSanctum => (assets.mystic_grass.clone(), Color::srgb(0.54, 0.62, 0.86)),
            MapKind::DreamWaterway => (assets.mystic_grass.clone(), Color::srgb(0.50, 0.62, 0.86)),
            _ => (assets.mystic_grass.clone(), Color::srgb(0.72, 0.88, 0.82)),
        },
        Tile::Npc | Tile::Path | Tile::Portal => match kind {
            MapKind::Village => (
                assets.village_moss_path.clone(),
                Color::srgb(0.88, 0.82, 0.66),
            ),
            MapKind::Bamboo => (assets.bamboo_path.clone(), Color::srgb(0.94, 0.84, 0.62)),
            MapKind::Cave => (assets.cave_floor.clone(), Color::srgb(0.68, 0.74, 0.82)),
            MapKind::MoonEchoCorridor => (assets.cave_floor.clone(), Color::srgb(0.62, 0.74, 0.94)),
            MapKind::RiverTown => (assets.stone_road.clone(), Color::srgb(0.78, 0.76, 0.66)),
            MapKind::RiverReedBed => (assets.reed_floor.clone(), Color::srgb(0.96, 0.94, 0.88)),
            MapKind::PlagueVillage => (assets.plague_floor.clone(), Color::srgb(0.95, 0.92, 0.88)),
            MapKind::PlagueShrinePath => {
                (assets.plague_floor.clone(), Color::srgb(0.86, 0.84, 0.90))
            }
            MapKind::Capital => (assets.capital_floor.clone(), Color::srgb(0.96, 0.96, 0.98)),
            MapKind::CapitalMansion => {
                (assets.mansion_floor.clone(), Color::srgb(0.96, 0.92, 0.90))
            }
            MapKind::MansionMirrorGallery => {
                (assets.mirror_floor.clone(), Color::srgb(0.94, 0.97, 1.0))
            }
            MapKind::SouthernRoad => (assets.south_floor.clone(), Color::srgb(0.98, 0.94, 0.90)),
            MapKind::ThunderDrumPath => {
                (assets.thunder_floor.clone(), Color::srgb(0.92, 0.94, 1.0))
            }
            MapKind::FinalSanctum => (assets.abyss_floor.clone(), Color::srgb(0.94, 0.92, 1.0)),
            MapKind::DreamWaterway => (assets.dream_floor.clone(), Color::srgb(0.94, 0.98, 1.0)),
        },
    };

    let sample = terrain_sample(col, row);
    Sprite {
        image,
        color,
        custom_size: Some(Vec2::splat(TILE + 0.5)),
        rect: Some(sample.rect),
        flip_x: sample.flip_x,
        flip_y: sample.flip_y,
        ..default()
    }
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct TerrainRenderEntities {
    pub backdrop: Entity,
    pub surface: Entity,
}

fn terrain_id(tile: Tile) -> u8 {
    match tile {
        Tile::Grass => 1,
        Tile::Wall => 2,
        Tile::Water => 3,
        Tile::Npc | Tile::Path | Tile::Portal => 0,
    }
}

fn terrain_tint(color: Color, scale: f32) -> Vec4 {
    let color = color.to_linear();
    Vec4::new(
        color.red * scale,
        color.green * scale,
        color.blue * scale,
        color.alpha,
    )
}

fn terrain_id_image(map: &MapData) -> Image {
    let mut data = Vec::with_capacity((MAP_W * MAP_H * 4) as usize);
    for row in 0..MAP_H {
        for col in 0..MAP_W {
            data.extend_from_slice(&[terrain_id(map.at(col, row)), 0, 0, 255]);
        }
    }
    let mut image = Image::new(
        Extent3d {
            width: MAP_W as u32,
            height: MAP_H as u32,
            depth_or_array_layers: 1,
        },
        TextureDimension::D2,
        data,
        TextureFormat::Rgba8Unorm,
        RenderAssetUsages::MAIN_WORLD | RenderAssetUsages::RENDER_WORLD,
    );
    image.sampler = ImageSampler::nearest();
    image
}

/// Renders the entire logical map as one continuous material. Terrain IDs stay
/// tile-based for gameplay, while the shader samples textures in world space
/// and feathers neighboring terrain types across their shared boundary.
pub(crate) fn spawn_terrain_map(
    commands: &mut Commands,
    meshes: &mut Assets<Mesh>,
    materials: &mut Assets<TerrainMaterial>,
    images: &mut Assets<Image>,
    assets: &ExploreAssets,
    map: &MapData,
    wall_dim: f32,
    state: AppState,
) -> TerrainRenderEntities {
    let floor = tile_sprite(Tile::Path, map.kind, assets, 0, 0);
    let grass = tile_sprite(Tile::Grass, map.kind, assets, 0, 0);
    let mut wall = tile_sprite(Tile::Wall, map.kind, assets, 0, 0);
    let water = tile_sprite(Tile::Water, map.kind, assets, 0, 0);

    let wall_tint = terrain_tint(wall.color, wall_dim);
    let wall_image = wall.image.clone();
    let wall_color = wall.color.to_srgba();
    wall.rect = None;
    wall.flip_x = false;
    wall.flip_y = false;
    wall.custom_size = Some(Vec2::new(
        (MAP_W as f32 + 14.0) * TILE,
        (MAP_H as f32 + 14.0) * TILE,
    ));
    wall.color = Color::srgba(
        wall_color.red * wall_dim * 0.72,
        wall_color.green * wall_dim * 0.72,
        wall_color.blue * wall_dim * 0.72,
        0.96,
    );
    let backdrop = commands
        .spawn((
            wall,
            Transform::from_xyz(0.0, 0.0, -1.0),
            DespawnOnExit(state),
        ))
        .id();

    let kind_seed = map.kind as u32;
    let phase_x = ((kind_seed * 5 + 2) % 14) as f32 + 0.37;
    let phase_y = ((kind_seed * 9 + 4) % 14) as f32 + 0.61;
    let tile_ids = images.add(terrain_id_image(map));
    let material = materials.add(TerrainMaterial {
        tile_ids,
        floor: floor.image,
        grass: grass.image,
        wall: wall_image,
        water: water.image,
        params: TerrainMaterialParams {
            floor_tint: terrain_tint(floor.color, 1.0),
            grass_tint: terrain_tint(grass.color, 1.0),
            wall_tint,
            water_tint: terrain_tint(water.color, 1.0),
            map: Vec4::new(MAP_W as f32, MAP_H as f32, TILE, 0.22),
            sample: Vec4::new(
                TERRAIN_SOURCE_SIZE,
                TERRAIN_SOURCE_INSET,
                TERRAIN_SAMPLE_SIZE * TERRAIN_SAMPLES_PER_AXIS as f32,
                TERRAIN_SAMPLE_SIZE,
            ),
            phase: Vec4::new(phase_x, phase_y, 0.075, 0.0),
        },
    });
    let mesh = meshes.add(Rectangle::new(MAP_W as f32 * TILE, MAP_H as f32 * TILE));
    let surface = commands
        .spawn((
            Mesh2d(mesh),
            MeshMaterial2d(material),
            Transform::from_xyz(0.0, 0.0, 0.0),
            DespawnOnExit(state),
        ))
        .id();

    TerrainRenderEntities { backdrop, surface }
}

fn spawn_quest_marker(
    commands: &mut Commands,
    font: &GameFont,
    lights: &LightingAssets,
    kind: QuestMarkerKind,
    origin: Vec2,
    y_offset: f32,
    phase: f32,
) {
    let marker = QuestMarker {
        kind,
        base_y: origin.y + y_offset,
        phase,
    };

    commands.spawn((
        MapContent,
        marker,
        QuestMarkerPart::Glow,
        Sprite {
            image: lights.orb.clone(),
            color: Color::srgba(1.0, 1.0, 1.0, 0.0),
            custom_size: Some(Vec2::splat(46.0)),
            ..default()
        },
        Transform::from_xyz(origin.x, marker.base_y, 6.00),
        Visibility::Hidden,
        DespawnOnExit(AppState::Explore),
    ));

    commands.spawn((
        MapContent,
        marker,
        QuestMarkerPart::Badge,
        Sprite {
            color: Color::srgba(1.0, 1.0, 1.0, 0.0),
            custom_size: Some(Vec2::splat(25.0)),
            ..default()
        },
        Transform::from_xyz(origin.x, marker.base_y, 6.05)
            .with_rotation(Quat::from_rotation_z(std::f32::consts::FRAC_PI_4)),
        Visibility::Hidden,
        DespawnOnExit(AppState::Explore),
    ));

    commands.spawn((
        MapContent,
        marker,
        QuestMarkerPart::GlyphShadow,
        Text2d::new(""),
        font.text_font(24.0),
        TextColor(Color::srgba(0.0, 0.0, 0.0, 0.72)),
        Transform::from_xyz(origin.x + 1.5, marker.base_y - 2.0, 6.09),
        Visibility::Hidden,
        DespawnOnExit(AppState::Explore),
    ));

    commands.spawn((
        MapContent,
        marker,
        QuestMarkerPart::Glyph,
        Text2d::new(""),
        font.text_font(24.0),
        TextColor(Color::srgb(1.0, 0.98, 0.84)),
        Transform::from_xyz(origin.x, marker.base_y - 3.0, 6.10),
        Visibility::Hidden,
        DespawnOnExit(AppState::Explore),
    ));
}

fn spawn_prop(
    commands: &mut Commands,
    font: &GameFont,
    asset_server: &AssetServer,
    lights: &LightingAssets,
    kind: MapKind,
    prop: &PropDef,
) {
    let p = tile_to_world(prop.col, prop.row);
    let origin = Vec3::new(p.x, p.y + 3.0, 4.2);
    let phase = scene_phase(prop.col, prop.row);
    let motion_kind = scene_motion_for_prop_path(prop.path);
    commands.spawn((
        MapContent,
        SceneMotion::new(motion_kind, origin, phase),
        Sprite {
            image: asset_server.load(prop.path),
            custom_size: Some(Vec2::splat(prop.size)),
            ..default()
        },
        Transform::from_translation(origin),
        DespawnOnExit(AppState::Explore),
    ));

    if prop.light[3] > 0.0 {
        let light_origin = Vec3::new(p.x, p.y, 2.8);
        let light_color = Color::srgba(prop.light[0], prop.light[1], prop.light[2], prop.light[3]);
        let prop_light = lighting::spawn_light(
            commands,
            lights,
            light_origin,
            TILE * 2.8,
            light_color,
            AppState::Explore,
        );
        commands.entity(prop_light).insert((
            MapContent,
            SceneLightPulse::new(motion_kind, light_origin, phase, TILE * 2.8, light_color),
        ));
    }

    if !side_quests_for_board(kind, prop).is_empty() {
        spawn_quest_marker(
            commands,
            font,
            lights,
            QuestMarkerKind::SideBoard(kind),
            p,
            TILE * 0.76,
            prop.col as f32 * 0.19 + prop.row as f32 * 0.07,
        );
    }

    if let Some(lamp) = final_lamp_for_prop(kind, prop) {
        spawn_quest_marker(
            commands,
            font,
            lights,
            QuestMarkerKind::Lamp(lamp),
            p,
            TILE * 0.76,
            prop.col as f32 * 0.19 + prop.row as f32 * 0.07,
        );
    }

    if let Some(lantern) = river_lantern_for_prop(kind, prop) {
        spawn_quest_marker(
            commands,
            font,
            lights,
            QuestMarkerKind::RiverLantern(lantern),
            p,
            TILE * 0.76,
            prop.col as f32 * 0.19 + prop.row as f32 * 0.07,
        );
    }

    if let Some(ward) = plague_ward_for_prop(kind, prop) {
        spawn_quest_marker(
            commands,
            font,
            lights,
            QuestMarkerKind::PlagueWard(ward),
            p,
            TILE * 0.76,
            prop.col as f32 * 0.19 + prop.row as f32 * 0.07,
        );
    }

    if let Some(drum) = thunder_drum_for_prop(kind, prop) {
        spawn_quest_marker(
            commands,
            font,
            lights,
            QuestMarkerKind::ThunderDrum(drum),
            p,
            TILE * 0.76,
            prop.col as f32 * 0.19 + prop.row as f32 * 0.07,
        );
    }

    if let Some(node) = mansion_mirror_for_prop(kind, prop) {
        spawn_quest_marker(
            commands,
            font,
            lights,
            QuestMarkerKind::MansionMirror(node),
            p,
            TILE * 0.76,
            prop.col as f32 * 0.19 + prop.row as f32 * 0.07,
        );
    }

    if let Some(crystal) = moon_crystal_for_prop(kind, prop) {
        spawn_quest_marker(
            commands,
            font,
            lights,
            QuestMarkerKind::Crystal(crystal),
            p,
            TILE * 0.76,
            prop.col as f32 * 0.19 + prop.row as f32 * 0.07,
        );
    }

    if let Some(mark) = route_mark_for_prop(kind, prop) {
        spawn_quest_marker(
            commands,
            font,
            lights,
            QuestMarkerKind::RouteMark(mark),
            p,
            TILE * 0.76,
            prop.col as f32 * 0.19 + prop.row as f32 * 0.07,
        );
        if let Some(revisit) = CompanionRevisit::for_target_mark(mark) {
            spawn_quest_marker(
                commands,
                font,
                lights,
                QuestMarkerKind::CompanionRevisitField(revisit),
                p,
                TILE * 1.18,
                prop.col as f32 * 0.19 + prop.row as f32 * 0.07 + 0.43,
            );
        }
    }

    if let Some(detour) = route_detour_for_prop(kind, prop) {
        spawn_quest_marker(
            commands,
            font,
            lights,
            QuestMarkerKind::RouteDetour(detour),
            p,
            TILE * 0.76,
            prop.col as f32 * 0.19 + prop.row as f32 * 0.07,
        );
    }
}

fn spawn_field_supply(
    commands: &mut Commands,
    font: &GameFont,
    asset_server: &AssetServer,
    lights: &LightingAssets,
    supply: &FieldSupplyDef,
) {
    let p = tile_to_world(supply.col, supply.row);
    let origin = Vec3::new(p.x, p.y + 2.0, 4.15);
    let phase = scene_phase(supply.col, supply.row);
    let motion_kind = match supply.path {
        "props/ai_cave_crystal.png" => SceneMotionKind::Crystal,
        "props/ai_spring.png" => SceneMotionKind::SpiritLantern,
        _ => SceneMotionKind::Shrine,
    };

    commands.spawn((
        MapContent,
        SceneMotion::new(motion_kind, origin, phase),
        Sprite {
            image: asset_server.load(supply.path),
            custom_size: Some(Vec2::splat(supply.size)),
            ..default()
        },
        Transform::from_translation(origin),
        DespawnOnExit(AppState::Explore),
    ));

    if supply.light[3] > 0.0 {
        let light_origin = Vec3::new(p.x, p.y, 2.7);
        let light_color = Color::srgba(
            supply.light[0],
            supply.light[1],
            supply.light[2],
            supply.light[3],
        );
        let supply_light = lighting::spawn_light(
            commands,
            lights,
            light_origin,
            TILE * 2.2,
            light_color,
            AppState::Explore,
        );
        commands.entity(supply_light).insert((
            MapContent,
            SceneLightPulse::new(motion_kind, light_origin, phase, TILE * 2.2, light_color),
        ));
    }

    spawn_quest_marker(
        commands,
        font,
        lights,
        QuestMarkerKind::FieldSupply(supply.supply),
        p,
        TILE * 0.58,
        phase + 0.37,
    );
}

fn spawn_commission_trace(
    commands: &mut Commands,
    font: &GameFont,
    asset_server: &AssetServer,
    lights: &LightingAssets,
    trace: &CommissionTraceDef,
) {
    let p = tile_to_world(trace.col, trace.row);
    let origin = Vec3::new(p.x, p.y + 2.0, 4.18);
    let phase = scene_phase(trace.col, trace.row) + 0.61;
    let motion_kind = scene_motion_for_prop_path(trace.path);

    commands.spawn((
        MapContent,
        SceneMotion::new(motion_kind, origin, phase),
        Sprite {
            image: asset_server.load(trace.path),
            custom_size: Some(Vec2::splat(trace.size)),
            ..default()
        },
        Transform::from_translation(origin),
        DespawnOnExit(AppState::Explore),
    ));

    if trace.light[3] > 0.0 {
        let light_origin = Vec3::new(p.x, p.y, 2.72);
        let light_color = Color::srgba(
            trace.light[0],
            trace.light[1],
            trace.light[2],
            trace.light[3],
        );
        let trace_light = lighting::spawn_light(
            commands,
            lights,
            light_origin,
            TILE * 2.35,
            light_color,
            AppState::Explore,
        );
        commands.entity(trace_light).insert((
            MapContent,
            SceneLightPulse::new(motion_kind, light_origin, phase, TILE * 2.35, light_color),
        ));
    }

    spawn_quest_marker(
        commands,
        font,
        lights,
        QuestMarkerKind::CommissionTrace(trace.side),
        p,
        TILE * 0.58,
        phase + 0.21,
    );
}

fn spawn_npc(
    commands: &mut Commands,
    font: &GameFont,
    asset_server: &AssetServer,
    dolls: &PaperdollAssets,
    lights: &LightingAssets,
    kind: MapKind,
    npc: &NpcDef,
) {
    let p = tile_to_world(npc.col, npc.row);
    let phase = scene_phase(npc.col, npc.row);
    let (motion_entity, light_color, origin) = match npc.visual {
        NpcVisual::Paperdoll(style) => {
            let origin = Vec3::new(p.x, p.y, 5.0);
            let entity = paperdoll::spawn_paperdoll(
                commands,
                dolls,
                style,
                origin,
                paperdoll::OVERWORLD_SIZE,
                AppState::Explore,
            );
            spawn_owned_character_shadow(
                commands,
                lights,
                entity,
                origin,
                Vec2::new(0.0, -paperdoll::OVERWORLD_SIZE * 0.42),
                Vec2::new(
                    paperdoll::OVERWORLD_SIZE * 0.72,
                    paperdoll::OVERWORLD_SIZE * 0.18,
                ),
                phase,
                0.25,
                2.0,
                4.12,
            );
            (Some(entity), Color::srgba(1.0, 0.82, 0.42, 0.25), origin)
        }
        NpcVisual::Image { path, size, light } => {
            let origin = Vec3::new(p.x, p.y + 6.0, 5.0);
            spawn_layered_npc_cutout(commands, asset_server, lights, path, origin, size, phase);
            (
                None,
                Color::srgba(light[0], light[1], light[2], light[3]),
                origin,
            )
        }
    };

    if let Some(entity) = motion_entity {
        commands.entity(entity).insert((
            MapContent,
            SceneMotion::new(SceneMotionKind::Npc, origin, phase),
        ));
    }
    let light_origin = Vec3::new(p.x, p.y, 3.5);
    let npc_light = lighting::spawn_light(
        commands,
        lights,
        light_origin,
        TILE * 3.2,
        light_color,
        AppState::Explore,
    );
    commands.entity(npc_light).insert((
        MapContent,
        SceneLightPulse::new(
            SceneMotionKind::Npc,
            light_origin,
            phase,
            TILE * 3.2,
            light_color,
        ),
    ));
    if let Some(role) = npc.quest {
        spawn_quest_marker(
            commands,
            font,
            lights,
            QuestMarkerKind::Role(role),
            p,
            TILE * 0.72,
            npc.col as f32 * 0.23 + npc.row as f32 * 0.11,
        );
    }
    if is_side_quest_contact(kind, npc) {
        spawn_quest_marker(
            commands,
            font,
            lights,
            QuestMarkerKind::SideContact(kind),
            p,
            TILE * 0.72,
            npc.col as f32 * 0.23 + npc.row as f32 * 0.11 + 0.53,
        );
    }
    if let Some(errand) = npc_errand_offer_for(kind, npc) {
        spawn_quest_marker(
            commands,
            font,
            lights,
            QuestMarkerKind::NpcErrandOffer(errand),
            p,
            TILE * 0.72,
            npc.col as f32 * 0.23 + npc.row as f32 * 0.11 + 0.91,
        );
    }
    if let Some(errand) = npc_errand_delivery_for(kind, npc) {
        spawn_quest_marker(
            commands,
            font,
            lights,
            QuestMarkerKind::NpcErrandDelivery(errand),
            p,
            TILE * 0.72,
            npc.col as f32 * 0.23 + npc.row as f32 * 0.11 + 1.19,
        );
    }
    if let Some(revisit) = npc_companion_revisit_for(kind, npc) {
        spawn_quest_marker(
            commands,
            font,
            lights,
            QuestMarkerKind::CompanionRevisitGiver(revisit),
            p,
            TILE * 0.96,
            npc.col as f32 * 0.23 + npc.row as f32 * 0.11 + 1.43,
        );
    }
}

fn spawn_layered_npc_cutout(
    commands: &mut Commands,
    asset_server: &AssetServer,
    lights: &LightingAssets,
    path: &'static str,
    origin: Vec3,
    size: f32,
    phase: f32,
) {
    let image = asset_server.load(path);
    spawn_scene_character_shadow(
        commands,
        lights,
        origin,
        Vec2::new(0.0, -size * 0.43),
        Vec2::new(size * 0.64, size * 0.15),
        phase,
        0.24,
    );
    for spec in cutout_part_specs(cutout_source_px_for_path(path), size) {
        commands.spawn((
            MapContent,
            LayeredNpcPart {
                part: spec.part,
                origin,
                base_offset: spec.offset,
                base_size: spec.size,
                phase,
            },
            Sprite {
                image: image.clone(),
                rect: Some(spec.rect),
                custom_size: Some(spec.size),
                ..default()
            },
            Transform::from_xyz(
                origin.x + spec.offset.x,
                origin.y + spec.offset.y,
                origin.z + spec.part.z_offset(),
            ),
            DespawnOnExit(AppState::Explore),
        ));
    }
}

fn scene_phase(col: i32, row: i32) -> f32 {
    col as f32 * 0.37 + row as f32 * 0.19
}

fn scene_motion_for_prop_path(path: &str) -> SceneMotionKind {
    match path {
        "props/ai_quest_board.png" => SceneMotionKind::QuestBoard,
        "props/ai_spirit_lantern.png" => SceneMotionKind::SpiritLantern,
        "props/ai_cave_crystal.png" => SceneMotionKind::Crystal,
        "props/ai_shrine_statue.png" => SceneMotionKind::Shrine,
        "props/ai_bamboo_gate.png" => SceneMotionKind::Gate,
        _ => SceneMotionKind::Shrine,
    }
}

fn scene_motion_profile(kind: SceneMotionKind) -> SceneMotionProfile {
    match kind {
        SceneMotionKind::Npc => SceneMotionProfile {
            y_amp: 1.5,
            x_amp: 0.18,
            scale_amp: 0.014,
            rotation_amp: 0.012,
            speed: 2.0,
            brightness_amp: 0.06,
            alpha_amp: 0.0,
            light_radius_amp: 0.035,
            light_intensity_amp: 0.10,
        },
        SceneMotionKind::QuestBoard => SceneMotionProfile {
            y_amp: 0.35,
            x_amp: 0.0,
            scale_amp: 0.008,
            rotation_amp: 0.018,
            speed: 1.35,
            brightness_amp: 0.08,
            alpha_amp: 0.0,
            light_radius_amp: 0.045,
            light_intensity_amp: 0.14,
        },
        SceneMotionKind::SpiritLantern => SceneMotionProfile {
            y_amp: 1.1,
            x_amp: 0.12,
            scale_amp: 0.038,
            rotation_amp: 0.020,
            speed: 2.55,
            brightness_amp: 0.18,
            alpha_amp: 0.10,
            light_radius_amp: 0.10,
            light_intensity_amp: 0.24,
        },
        SceneMotionKind::Crystal => SceneMotionProfile {
            y_amp: 0.55,
            x_amp: 0.0,
            scale_amp: 0.032,
            rotation_amp: 0.0,
            speed: 2.2,
            brightness_amp: 0.22,
            alpha_amp: 0.08,
            light_radius_amp: 0.08,
            light_intensity_amp: 0.22,
        },
        SceneMotionKind::Shrine => SceneMotionProfile {
            y_amp: 0.22,
            x_amp: 0.0,
            scale_amp: 0.012,
            rotation_amp: 0.006,
            speed: 1.1,
            brightness_amp: 0.09,
            alpha_amp: 0.0,
            light_radius_amp: 0.04,
            light_intensity_amp: 0.12,
        },
        SceneMotionKind::Gate => SceneMotionProfile {
            y_amp: 0.0,
            x_amp: 0.0,
            scale_amp: 0.075,
            rotation_amp: 0.0,
            speed: 1.75,
            brightness_amp: 0.22,
            alpha_amp: 0.18,
            light_radius_amp: 0.12,
            light_intensity_amp: 0.28,
        },
    }
}

fn pulse01(t: f32) -> f32 {
    0.5 + 0.5 * t.sin()
}

fn animated_color(base: Color, brightness: f32, alpha_scale: f32) -> Color {
    let srgba = base.to_srgba();
    Color::srgba(
        (srgba.red * brightness).clamp(0.0, 1.0),
        (srgba.green * brightness).clamp(0.0, 1.0),
        (srgba.blue * brightness).clamp(0.0, 1.0),
        (srgba.alpha * alpha_scale).clamp(0.02, 1.0),
    )
}

fn npc_body_motion(t: f32) -> CharacterBodyMotion {
    let breath = t.sin();
    let sway = (t * 0.62).sin();
    let shoulder = (t * 1.7).cos();

    CharacterBodyMotion {
        offset: Vec2::new(sway * 0.70, breath * 2.10 + shoulder.abs() * 0.34),
        scale: Vec2::new(1.0 + shoulder * 0.010, 1.0 + breath.abs() * 0.018),
        rotation: sway * 0.018,
        brightness: 1.0 + breath.max(0.0) * 0.075,
    }
}

fn follower_body_motion(slot: usize, moving: bool, phase: f32) -> CharacterBodyMotion {
    let slot_phase = slot as f32 * 0.77;
    let step_speed = if moving { 9.6 } else { 2.2 };
    let t = phase * step_speed + slot_phase;
    let step = t.sin();
    let lift = step.abs();
    let sway = (t * 0.5).sin();

    if moving {
        CharacterBodyMotion {
            offset: Vec2::new(sway * 2.2, lift * 3.8),
            scale: Vec2::new(1.0 + lift * 0.020, 1.0 - lift * 0.018),
            rotation: step * 0.028,
            brightness: 1.0 + lift * 0.070,
        }
    } else {
        CharacterBodyMotion {
            offset: Vec2::new(sway * 0.44, step * 1.15),
            scale: Vec2::new(1.0 + step * 0.006, 1.0 + step.abs() * 0.010),
            rotation: sway * 0.010,
            brightness: 1.0 + step.max(0.0) * 0.035,
        }
    }
}

fn character_shadow_frame(base_alpha: f32, t: f32) -> CharacterShadowFrame {
    let breath = pulse01(t);
    let step = (t * 1.7).sin().abs();
    CharacterShadowFrame {
        scale: Vec3::new(
            0.90 + breath * 0.10 + step * 0.08,
            0.90 - breath * 0.08 + step * 0.025,
            1.0,
        ),
        alpha: (base_alpha * (0.78 + (1.0 - breath) * 0.18 + step * 0.10)).clamp(0.02, 0.62),
    }
}

fn character_afterimage_frame(age: f32, duration: f32) -> Option<CharacterAfterimageFrame> {
    if duration <= 0.0 || age >= duration {
        return None;
    }
    let progress = (age / duration).clamp(0.0, 1.0);
    Some(CharacterAfterimageFrame {
        progress,
        alpha_scale: (1.0 - progress).powf(1.35),
    })
}

fn area_route_label(kind: MapKind, stage: QuestStage) -> String {
    let target = kind.portal_target(stage);
    if portal_gate_message(kind, target, stage).is_some() {
        format!("下一程：{}（需推进剧情）", target.def().name)
    } else if target == kind {
        "下一程：原地整备".to_string()
    } else {
        format!("下一程：{}", target.def().name)
    }
}

fn area_banner_lines(map: &MapData, quest: &QuestLog) -> [String; 3] {
    [
        format!("抵达 · {}", map.name),
        format!(
            "{} · {}",
            quest.chapter_title(),
            area_route_label(map.kind, quest.stage())
        ),
        format!("目标 · {}", compact_hud_text(&quest.objective(), 54)),
    ]
}

fn move_player_to_spawn(map: &MapData, pos: &mut PlayerPos) {
    let (col, row) = map.spawn();
    pos.col = col;
    pos.row = row;
    pos.facing = IVec2::new(1, 0);
}

// ---------------------------------------------------------------------------
// Systems
// ---------------------------------------------------------------------------

#[allow(clippy::too_many_arguments)]
fn explore_input(
    mut commands: Commands,
    intent: Res<Intent>,
    time: Res<Time>,
    rate: Res<EncounterRate>,
    mut spawner: MapSpawnParams,
    mut cooldown: ResMut<MoveCooldown>,
    mut pos: ResMut<PlayerPos>,
    mut current: ResMut<CurrentMap>,
    map: Option<Res<MapData>>,
    mut dialogue: ResMut<Dialogue>,
    mut quest_notice: ResMut<QuestNotice>,
    mut quest: ResMut<QuestLog>,
    mut stats: ResMut<PlayerStats>,
    mut rng: ResMut<Rng>,
    mut next: ResMut<NextState<AppState>>,
) {
    let Some(map) = map else {
        return;
    };

    // Dialogue mode swallows all other input.
    if dialogue.active {
        if dialogue.choice.is_some() && dialogue.idx >= dialogue.lines.len() {
            if intent.up || intent.down {
                if let Some(choice) = dialogue.choice.as_mut() {
                    let count = choice.option_count();
                    if intent.up {
                        choice.selected = (choice.selected + count - 1) % count;
                    } else {
                        choice.selected = (choice.selected + 1) % count;
                    }
                }
            }
            if intent.confirm {
                if let Some(choice) = dialogue.choice.take() {
                    let result = resolve_dialogue_choice(choice, &mut quest, &mut stats);
                    if let Some(message) = quest_notice_for_confirmed_choice(choice, &quest) {
                        quest_notice.show(message);
                    }
                    dialogue.lines = result.lines;
                    dialogue.after = result.after;
                    dialogue.choice = result.choice;
                    dialogue.idx = 0;
                    dialogue.chapter_art = result.chapter_art;
                }
            }
            return;
        }

        if intent.confirm {
            dialogue.idx += 1;
            if dialogue.idx >= dialogue.lines.len() {
                if dialogue.choice.is_some() {
                    return;
                }
                dialogue.active = false;
                dialogue.portrait_path = None;
                dialogue.chapter_art = None;
                match std::mem::take(&mut dialogue.after) {
                    DialogueAfter::None => {}
                    DialogueAfter::StartBattle(encounter) => {
                        commands.insert_resource(encounter);
                        next.set(AppState::Battle);
                    }
                }
            }
        }
        return;
    }

    // Talk to an NPC the player faces.
    if intent.confirm {
        let tc = pos.col + pos.facing.x;
        let tr = pos.row + pos.facing.y;
        if let Some(npc) = npc_defs(map.kind)
            .iter()
            .find(|n| n.col == tc && n.row == tr)
        {
            let mut lines: Vec<String> = npc.lines.iter().map(|line| (*line).to_owned()).collect();
            let mut after = DialogueAfter::None;
            let mut pending_choice = None;
            let mut story_advanced = false;
            let mut chapter_art = None;
            if let Some(role) = npc.quest {
                if let Some(choice) = main_quest_choice_for(&quest, role) {
                    if let DialogueChoiceKind::MainQuest { action, .. } = choice.kind {
                        lines.extend(main_quest_preview(&quest, role, action));
                    }
                    pending_choice = Some(choice);
                } else {
                    let stage_before = quest.stage();
                    lines.extend(quest.talk(role));
                    story_advanced = quest.stage() != stage_before;
                    after = main_quest_battle_after(role, stage_before);
                }
            }
            if story_advanced {
                chapter_art = append_chapter_card(&mut quest, &mut lines);
            }
            if !story_advanced && pending_choice.is_none() {
                if let Some(handoff) = npc_side_quest_handoff(map.kind, npc, &quest) {
                    lines.extend(handoff.lines);
                    pending_choice = handoff.choice;
                } else if let Some(handoff) = npc_errand_handoff(map.kind, npc, &quest) {
                    lines.extend(handoff.lines);
                    pending_choice = handoff.choice;
                } else if let Some(handoff) = npc_companion_revisit_handoff(map.kind, npc, &quest) {
                    lines.extend(handoff.lines);
                    pending_choice = handoff.choice;
                } else {
                    lines.extend(npc_reaction_lines(map.kind, npc, &quest));
                    if npc.quest.is_none() {
                        if let Some(reward) = quest.claim_care_aftermath(map_chapter(map.kind)) {
                            lines.push(apply_care_aftermath_reward(&mut stats, reward));
                        }
                        lines.extend(claim_local_companion_scene_followup(
                            map.kind, &mut quest, &mut stats,
                        ));
                        if let Some(reward) =
                            quest.claim_commission_aftermath(map_chapter(map.kind))
                        {
                            lines.push(local_commission_aftermath_line(map.kind).to_string());
                            lines.push(apply_commission_aftermath_reward(&mut stats, reward));
                        }
                        lines.extend(claim_local_route_detour_report(
                            map.kind, &mut quest, &mut stats,
                        ));
                    }
                    if let Some(service) = npc_service_for(map.kind, npc) {
                        let service_result =
                            apply_npc_service(service, map.kind, &mut quest, &mut stats);
                        let rested = service_result.rested;
                        lines.push(service_result.line);
                        if rested {
                            let camp = quest.interact_camp_scene();
                            if camp.tactic_choice {
                                if let Some(default_bonus) = camp.bonus {
                                    pending_choice =
                                        Some(DialogueChoice::camp_tactic(default_bonus));
                                }
                            }
                            lines.extend(camp.lines);
                        }
                    }
                }
            }
            dialogue.active = true;
            dialogue.lines = lines;
            dialogue.idx = 0;
            dialogue.after = after;
            dialogue.choice = pending_choice;
            dialogue.portrait_path = dialogue_portrait_path(npc.visual);
            dialogue.chapter_art = chapter_art;
            return;
        }

        if let Some(prop) = prop_defs(map.kind)
            .iter()
            .find(|prop| prop.col == tc && prop.row == tr)
        {
            let mut pending_choice = None;
            let mut lines = if let Some(lantern) = river_lantern_for_prop(map.kind, prop) {
                let was_lit = quest.river_lantern_marker_for(lantern) == "✓";
                let mut lines = quest.activate_river_lantern(lantern);
                let is_lit = quest.river_lantern_marker_for(lantern) == "✓";
                append_side_objective_progress(
                    &mut lines,
                    &mut quest,
                    SideQuest::RiverLanterns,
                    "河灯已巡",
                    was_lit,
                    is_lit,
                );
                lines
            } else if let Some(node) = mansion_mirror_for_prop(map.kind, prop) {
                let was_aligned = quest.mansion_mirror_marker_for(node) == "✓";
                let mut lines = quest.align_mansion_mirror(node);
                let is_aligned = quest.mansion_mirror_marker_for(node) == "✓";
                append_side_objective_progress(
                    &mut lines,
                    &mut quest,
                    SideQuest::CapitalRumors,
                    "镜阵暗文已拓",
                    was_aligned,
                    is_aligned,
                );
                lines
            } else if let Some(ward) = plague_ward_for_prop(map.kind, prop) {
                let was_sealed = quest.plague_ward_marker_for(ward) == "✓";
                let mut lines = quest.seal_plague_ward(ward);
                let is_sealed = quest.plague_ward_marker_for(ward) == "✓";
                append_side_objective_progress(
                    &mut lines,
                    &mut quest,
                    SideQuest::PlagueRelief,
                    "净瘴铃位已封",
                    was_sealed,
                    is_sealed,
                );
                lines
            } else if let Some(drum) = thunder_drum_for_prop(map.kind, prop) {
                let was_aligned = quest.thunder_drum_marker_for(drum) == "✓";
                let mut lines = quest.align_thunder_drum(drum);
                let is_aligned = quest.thunder_drum_marker_for(drum) == "✓";
                append_side_objective_progress(
                    &mut lines,
                    &mut quest,
                    SideQuest::SouthernThunder,
                    "雷鼓图腾已定",
                    was_aligned,
                    is_aligned,
                );
                lines
            } else if let Some(lamp) = final_lamp_for_prop(map.kind, prop) {
                let was_lit = quest.final_lamp_marker_for(lamp) == "✓";
                let mut lines = quest.light_final_lamp(lamp);
                let is_lit = quest.final_lamp_marker_for(lamp) == "✓";
                append_side_objective_progress(
                    &mut lines,
                    &mut quest,
                    SideQuest::FinalDreamEchoes,
                    "忆梦灯已守",
                    was_lit,
                    is_lit,
                );
                lines
            } else if let Some(crystal) = moon_crystal_for_prop(map.kind, prop) {
                let was_lit = quest.moon_crystal_marker_for(crystal) == "✓";
                let mut lines = quest.activate_moon_crystal(crystal);
                let is_lit = quest.moon_crystal_marker_for(crystal) == "✓";
                append_side_objective_progress(
                    &mut lines,
                    &mut quest,
                    SideQuest::MoonCaveCrystals,
                    "晶阵已净",
                    was_lit,
                    is_lit,
                );
                lines
            } else if let Some(mark) = route_mark_for_prop(map.kind, prop) {
                let was_marked = quest.route_mark_marker_for(mark) == "✓";
                let mut lines = quest.interact_route_mark(mark);
                let is_marked = quest.route_mark_marker_for(mark) == "✓";
                if let Some((side, source)) = route_mark_side_objective(mark) {
                    append_side_objective_progress(
                        &mut lines, &mut quest, side, source, was_marked, is_marked,
                    );
                }
                if let Some(revisit) = quest.active_companion_revisit_at(mark) {
                    let interaction = quest.resolve_companion_revisit(revisit);
                    lines.extend(interaction.lines);
                    if let Some(reward) = interaction.reward {
                        lines.push(apply_companion_scene_reward(&mut stats, reward));
                    }
                    lines.push(quest.companion_revisit_contract(revisit));
                }
                if let Some(line) = route_care_checkpoint_line(map.kind, &quest) {
                    lines.push(line.to_string());
                }
                if let Some(line) = companion_route_checkpoint_line(map.kind, &quest) {
                    lines.push(line.to_string());
                }
                if let Some(line) = companion_revisit_checkpoint_line(map.kind, &quest) {
                    lines.push(line.to_string());
                }
                if let Some(line) = side_quest_route_checkpoint_line(map.kind, &quest) {
                    lines.push(line.to_string());
                }
                if let Some(line) = npc_errand_route_checkpoint_line(map.kind, &quest) {
                    lines.push(line.to_string());
                }
                lines
            } else if let Some(detour) = route_detour_for_prop(map.kind, prop) {
                if !quest.has_route_detour(detour) {
                    pending_choice = Some(DialogueChoice::route_detour(detour));
                }
                quest.route_detour_preview(detour)
            } else if !side_quests_for_board(map.kind, prop).is_empty() {
                pending_choice = Some(DialogueChoice::side_board(map.kind, &quest));
                side_board_overview_lines(map.kind, &quest)
            } else if is_bond_lantern(prop) {
                if quest.companion_scene_available().is_some() {
                    let interaction = quest.interact_companion_scene();
                    let mut lines = interaction.lines;
                    if let Some(reward) = interaction.reward {
                        lines.push(apply_companion_scene_reward(&mut stats, reward));
                    }
                    lines
                } else {
                    let interaction = quest.interact_bond_scene();
                    let mut lines = interaction.lines;
                    if let Some(reward) = interaction.reward {
                        lines.push(apply_bond_reward(&mut stats, reward));
                    }
                    if interaction.response_choice {
                        pending_choice = Some(DialogueChoice::bond_response());
                    }
                    lines
                }
            } else if let Some(cache) = treasure_for_prop(map.kind, prop) {
                if quest.has_opened_treasure(cache) {
                    if let Some(blessing) = shrine_blessing_for_prop(map.kind, prop) {
                        apply_shrine_offering(&mut stats, &mut quest, blessing)
                    } else {
                        quest.interact_treasure(cache).lines
                    }
                } else {
                    let interaction = quest.interact_treasure(cache);
                    let mut lines = interaction.lines;
                    if let Some(reward) = interaction.reward {
                        lines.push(apply_treasure_reward(&mut stats, reward));
                    }
                    lines
                }
            } else if let Some(blessing) = shrine_blessing_for_prop(map.kind, prop) {
                apply_shrine_offering(&mut stats, &mut quest, blessing)
            } else {
                prop_dialogue(prop)
            };
            lines.push(quest.main_task_summary());
            dialogue.active = true;
            dialogue.lines = lines;
            dialogue.idx = 0;
            dialogue.after = DialogueAfter::None;
            dialogue.choice = pending_choice;
            dialogue.portrait_path = None;
            dialogue.chapter_art = None;
            return;
        }

        if let Some(supply) =
            field_supply_defs(map.kind).find(|supply| supply.col == tc && supply.row == tr)
        {
            let interaction = quest.interact_field_supply(supply.supply);
            let mut lines = interaction.lines;
            if let Some(reward) = interaction.reward {
                lines.push(apply_supply_reward(&mut stats, reward));
            }
            lines.push(quest.main_task_summary());
            dialogue.active = true;
            dialogue.lines = lines;
            dialogue.idx = 0;
            dialogue.after = DialogueAfter::None;
            dialogue.choice = None;
            dialogue.portrait_path = None;
            dialogue.chapter_art = None;
            return;
        }

        if let Some(trace) =
            commission_trace_defs(map.kind).find(|trace| trace.col == tc && trace.row == tr)
        {
            let can_choose = quest.is_side_quest_active(trace.side)
                && !quest.is_side_quest_completed(trace.side)
                && !quest.has_commission_trace(trace.side);
            let (mut lines, choice) = if can_choose {
                (
                    vec![
                        format!("【委托现场】{} · {}", trace.side.name(), trace.name),
                        trace.active_line.to_string(),
                        "【处理选择】细查会留下回访线索，快断会记录现场速断法。".to_string(),
                        quest.side_task_contract(trace.side),
                    ],
                    Some(DialogueChoice::commission_trace(trace)),
                )
            } else {
                (
                    quest.interact_commission_trace(
                        trace.side,
                        trace.name,
                        trace.active_line,
                        trace.source,
                        trace.inactive_line,
                        trace.repeat_line,
                    ),
                    None,
                )
            };
            lines.push(quest.side_task_summary(trace.side));
            lines.push(quest.main_task_summary());
            dialogue.active = true;
            dialogue.lines = lines;
            dialogue.idx = 0;
            dialogue.after = DialogueAfter::None;
            dialogue.choice = choice;
            dialogue.portrait_path = None;
            dialogue.chapter_art = None;
            return;
        }
    }

    // Movement with a per-step cooldown so holding a direction walks smoothly.
    cooldown.0 -= time.delta_secs();
    if let Some(d) = intent.move_dir {
        pos.facing = d;
        if cooldown.0 <= 0.0 {
            let nc = pos.col + d.x;
            let nr = pos.row + d.y;
            if map.at(nc, nr).walkable() {
                let tile = map.at(nc, nr);
                if tile == Tile::Portal {
                    let target = current.0.portal_target(quest.stage());
                    if let Some(message) = portal_gate_message(current.0, target, quest.stage()) {
                        dialogue.active = true;
                        dialogue.lines = vec![
                            message.to_string(),
                            format!("【目标】{}", quest.objective()),
                        ];
                        dialogue.idx = 0;
                        dialogue.after = DialogueAfter::None;
                        dialogue.choice = None;
                        dialogue.portrait_path = None;
                        dialogue.chapter_art = None;
                        cooldown.0 = 0.18;
                        return;
                    }

                    let transition_lines = portal_transition_lines(current.0, target, &quest);
                    current.0 = target;
                    let new_map = MapData::build(current.0);
                    move_player_to_spawn(&new_map, &mut pos);
                    for entity in &spawner.map_content {
                        commands.entity(entity).despawn();
                    }
                    for entity in &spawner.area_banners {
                        commands.entity(entity).despawn();
                    }
                    spawner.fog_memory.clear();
                    spawn_map_content(
                        &mut commands,
                        &spawner.font,
                        &spawner.asset_server,
                        &pos,
                        &spawner.dolls,
                        &spawner.lights,
                        &spawner.anims,
                        &spawner.explore_assets,
                        &new_map,
                        &mut spawner.meshes,
                        &mut spawner.terrain_materials,
                        &mut spawner.images,
                    );
                    spawn_area_banner(&mut commands, &spawner.font, &new_map, &quest);
                    commands.insert_resource(new_map);
                    dialogue.active = true;
                    dialogue.lines = transition_lines;
                    dialogue.idx = 0;
                    dialogue.after = DialogueAfter::None;
                    dialogue.choice = None;
                    dialogue.portrait_path = None;
                    dialogue.chapter_art = None;
                    cooldown.0 = 0.18;
                    return;
                }

                pos.col = nc;
                pos.row = nr;
                cooldown.0 = 0.14;
                // Random encounter when stepping into grass.
                if tile == Tile::Grass && rng.chance(route_encounter_rate(rate.0, map.kind, &quest))
                {
                    commands.insert_resource(PendingEncounter {
                        zone: encounter_zone(map.kind),
                        kind: EncounterKind::Random,
                    });
                    next.set(AppState::Battle);
                }
            } else {
                cooldown.0 = 0.10;
            }
        }
    } else {
        cooldown.0 = cooldown.0.min(0.0);
    }
}

fn sync_player_transform(
    mut commands: Commands,
    intent: Res<Intent>,
    anims: Res<AnimationAssets>,
    pos: Res<PlayerPos>,
    time: Res<Time>,
    mut trail_timer: Local<f32>,
    mut q: Query<(&mut Transform, &mut Sprite, &mut SpriteAnimation), With<PlayerSprite>>,
    mut lights: Query<&mut Transform, (With<PlayerLight>, Without<PlayerSprite>)>,
) {
    let p = tile_to_world(pos.col, pos.row);
    if let Ok((mut t, mut sprite, mut animation)) = q.single_mut() {
        t.translation.x = p.x;
        t.translation.y = p.y;
        let moving = intent.move_dir.is_some();
        let clip = if moving {
            AnimationClip::HeroWalk
        } else {
            AnimationClip::HeroIdle
        };
        animation::set_clip(&anims, &mut sprite, &mut animation, clip);
        if pos.facing.x != 0 {
            sprite.flip_x = pos.facing.x < 0;
        }
        if moving {
            *trail_timer += time.delta_secs();
            if *trail_timer >= 0.075 {
                *trail_timer = 0.0;
                let facing = if sprite.flip_x { -1.0 } else { 1.0 };
                spawn_sprite_afterimage(
                    &mut commands,
                    &sprite,
                    &t,
                    Color::srgba(0.70, 0.90, 1.0, 0.40),
                    0.34,
                    Vec2::new(-facing * 46.0, -9.0),
                );
            }
        } else {
            *trail_timer = 0.0;
        }
    }
    if let Ok(mut t) = lights.single_mut() {
        t.translation.x = p.x;
        t.translation.y = p.y;
    }
}

fn sync_party_followers(
    mut commands: Commands,
    dolls: Res<PaperdollAssets>,
    lights: Res<LightingAssets>,
    quest: Res<QuestLog>,
    pos: Res<PlayerPos>,
    intent: Res<Intent>,
    time: Res<Time>,
    mut followers: Query<(Entity, &mut PartyFollower, &mut Transform, &mut Sprite)>,
) {
    let desired = desired_party_followers(&quest);
    let moving = intent.move_dir.is_some();
    let phase = time.elapsed_secs();
    let mut present = vec![false; desired.len()];

    for (entity, mut follower, mut transform, mut sprite) in &mut followers {
        let Some(slot) = desired
            .iter()
            .position(|companion| *companion == follower.companion)
        else {
            commands.entity(entity).despawn();
            continue;
        };

        present[slot] = true;
        let body = follower_body_motion(slot, moving, phase);
        let mut target = follower_slot_position(&pos, slot, moving, phase);
        target.x += body.offset.x;
        target.y += body.offset.y;
        transform.translation = target;
        transform.rotation = Quat::from_rotation_z(body.rotation);
        transform.scale = Vec3::new(body.scale.x, body.scale.y, 1.0);
        sprite.flip_x = follower_faces_left(&pos);
        sprite.color = animated_color(Color::WHITE, body.brightness, 1.0);
        if moving {
            follower.trail_timer += time.delta_secs();
            if follower.trail_timer >= 0.11 {
                follower.trail_timer = 0.0;
                let facing = if sprite.flip_x { -1.0 } else { 1.0 };
                spawn_sprite_afterimage(
                    &mut commands,
                    &sprite,
                    &transform,
                    companion_afterimage_color(follower.companion),
                    0.30,
                    Vec2::new(-facing * 30.0, -6.0),
                );
            }
        } else {
            follower.trail_timer = 0.0;
        }
    }

    for (slot, companion) in desired.iter().copied().enumerate() {
        if present.get(slot).copied().unwrap_or(false) {
            continue;
        }

        let follower = paperdoll::spawn_paperdoll(
            &mut commands,
            &dolls,
            companion_style(companion),
            follower_slot_position(&pos, slot, moving, phase),
            paperdoll::OVERWORLD_SIZE * 0.92,
            AppState::Explore,
        );
        let follower_origin = follower_slot_position(&pos, slot, moving, phase);
        commands.entity(follower).insert((
            MapContent,
            PartyFollower {
                companion,
                trail_timer: 0.0,
            },
        ));
        spawn_owned_character_shadow(
            &mut commands,
            &lights,
            follower,
            follower_origin,
            Vec2::new(0.0, -paperdoll::OVERWORLD_SIZE * 0.36),
            Vec2::new(
                paperdoll::OVERWORLD_SIZE * 0.62,
                paperdoll::OVERWORLD_SIZE * 0.15,
            ),
            phase + slot as f32 * 0.61,
            0.22,
            4.8,
            4.18,
        );
    }
}

fn update_scene_motions(
    time: Res<Time>,
    mut animated_sprites: Query<(&SceneMotion, &mut Transform, &mut Sprite)>,
    mut animated_lights: Query<
        (
            &SceneLightPulse,
            &mut Transform,
            &mut Sprite,
            &mut PointLight2d,
        ),
        Without<SceneMotion>,
    >,
) {
    let elapsed = time.elapsed_secs();

    for (motion, mut transform, mut sprite) in &mut animated_sprites {
        let profile = scene_motion_profile(motion.kind);
        let t = elapsed * profile.speed + motion.phase;
        let pulse = pulse01(t);
        if motion.kind == SceneMotionKind::Npc {
            let body = npc_body_motion(t);
            transform.translation.x = motion.origin.x + body.offset.x;
            transform.translation.y = motion.origin.y + body.offset.y;
            transform.translation.z = motion.origin.z;
            transform.rotation = Quat::from_rotation_z(body.rotation);
            transform.scale = Vec3::new(body.scale.x, body.scale.y, 1.0);
            sprite.color = animated_color(motion.base_color, body.brightness, 1.0);
        } else {
            transform.translation.x = motion.origin.x + t.cos() * profile.x_amp;
            transform.translation.y = motion.origin.y + t.sin() * profile.y_amp;
            transform.translation.z = motion.origin.z;
            transform.rotation = Quat::from_rotation_z((t * 0.82).sin() * profile.rotation_amp);
            transform.scale = Vec3::splat(1.0 + profile.scale_amp * (pulse * 2.0 - 1.0));
            sprite.color = animated_color(
                motion.base_color,
                1.0 + profile.brightness_amp * pulse,
                1.0 + profile.alpha_amp * (pulse * 2.0 - 1.0),
            );
        }
    }

    for (pulse_data, mut transform, mut sprite, mut light) in &mut animated_lights {
        let profile = scene_motion_profile(pulse_data.kind);
        let t = elapsed * profile.speed + pulse_data.phase;
        let pulse = pulse01(t);
        transform.translation = pulse_data.origin;
        light.radius = pulse_data.base_radius * (1.0 + profile.light_radius_amp * pulse);
        light.intensity = pulse_data.base_intensity * (1.0 + profile.light_intensity_amp * pulse);
        sprite.custom_size = Some(Vec2::splat(
            pulse_data.base_size * (1.0 + profile.light_radius_amp * 0.34 * pulse),
        ));
        sprite.color = animated_color(
            pulse_data.base_color,
            1.0 + profile.brightness_amp * pulse,
            0.82 + 0.22 * pulse,
        );
    }
}

fn update_layered_npc_parts(
    time: Res<Time>,
    mut parts: Query<(&LayeredNpcPart, &mut Transform, &mut Sprite)>,
) {
    let elapsed = time.elapsed_secs();
    for (part, mut transform, mut sprite) in &mut parts {
        let t = elapsed * 2.25 + part.phase;
        let body = npc_body_motion(t);
        let segment = cutout_part_motion(part.part, t * 1.65 + part.part.z_offset() * 9.0, 1.0);
        let scale = Vec2::new(
            body.scale.x * segment.scale.x,
            body.scale.y * segment.scale.y,
        );

        transform.translation.x =
            part.origin.x + body.offset.x + part.base_offset.x + segment.offset.x;
        transform.translation.y =
            part.origin.y + body.offset.y + part.base_offset.y + segment.offset.y;
        transform.translation.z = part.origin.z + part.part.z_offset();
        transform.rotation = Quat::from_rotation_z(body.rotation + segment.rotation);
        transform.scale = Vec3::new(scale.x, scale.y, 1.0);
        sprite.custom_size = Some(part.base_size);
        sprite.color = brighten_color(Color::WHITE, body.brightness * segment.brightness);
    }
}

fn update_character_shadows(
    time: Res<Time>,
    mut commands: Commands,
    owners: Query<&Transform, Without<CharacterGroundShadow>>,
    mut shadows: Query<(Entity, &CharacterGroundShadow, &mut Transform, &mut Sprite)>,
) {
    let elapsed = time.elapsed_secs();
    for (entity, shadow, mut transform, mut sprite) in &mut shadows {
        let t = elapsed * shadow.speed + shadow.phase;
        let mut anchor = shadow.origin;
        if let Some(owner) = shadow.owner {
            let Ok(owner_transform) = owners.get(owner) else {
                commands.entity(entity).despawn();
                continue;
            };
            anchor = owner_transform.translation;
        } else if shadow.track_scene_motion {
            let body = npc_body_motion(t);
            anchor.x += body.offset.x;
            anchor.y += body.offset.y;
        }

        let frame = character_shadow_frame(shadow.base_alpha, t);
        transform.translation = Vec3::new(
            anchor.x + shadow.offset.x,
            anchor.y + shadow.offset.y,
            shadow.z,
        );
        transform.scale = frame.scale;
        sprite.custom_size = Some(shadow.base_size);
        sprite.color = Color::srgba(0.0, 0.0, 0.0, frame.alpha);
    }
}

fn update_character_afterimages(
    time: Res<Time>,
    mut commands: Commands,
    mut afterimages: Query<(
        Entity,
        &mut CharacterAfterimage,
        &mut Transform,
        &mut Sprite,
    )>,
) {
    let dt = time.delta_secs();
    for (entity, mut afterimage, mut transform, mut sprite) in &mut afterimages {
        afterimage.age += dt;
        let Some(frame) = character_afterimage_frame(afterimage.age, afterimage.duration) else {
            commands.entity(entity).despawn();
            continue;
        };

        transform.translation.x += afterimage.velocity.x * dt;
        transform.translation.y += afterimage.velocity.y * dt;
        transform.scale = afterimage
            .start_scale
            .lerp(afterimage.end_scale, frame.progress);
        sprite.color = color_with_alpha_scale(afterimage.base_color, frame.alpha_scale);
    }
}

fn update_ambient_particles(
    time: Res<Time>,
    mut query: Query<(&mut AmbientParticle, &mut Transform, &mut Sprite)>,
) {
    let span_y = MAP_H as f32 * TILE;
    let top = span_y * 0.5 - 8.0;

    for (mut particle, mut transform, mut sprite) in &mut query {
        particle.age += time.delta_secs();
        let t = particle.age * particle.speed + particle.phase;
        match particle.kind {
            AmbientKind::Rain => {
                let cycle = (particle.age * particle.speed + particle.phase.fract()).fract();
                transform.translation.x = particle.origin.x + t.sin() * particle.amplitude.x;
                transform.translation.y = top - cycle * span_y;
                transform.rotation = Quat::from_rotation_z(-0.18);
                set_sprite_alpha(&mut sprite, particle.color, 0.72 + 0.18 * (t * 3.1).sin());
            }
            AmbientKind::Leaf => {
                transform.translation.x = particle.origin.x + t.sin() * particle.amplitude.x;
                transform.translation.y =
                    particle.origin.y + (t * 0.72).cos() * particle.amplitude.y;
                transform.rotation = Quat::from_rotation_z((t * 0.9).sin() * 0.75);
                set_sprite_alpha(&mut sprite, particle.color, 0.74 + 0.18 * (t * 1.7).sin());
            }
            AmbientKind::Mist => {
                transform.translation.x = particle.origin.x + t.sin() * particle.amplitude.x;
                transform.translation.y =
                    particle.origin.y + (t * 0.56).cos() * particle.amplitude.y;
                transform.scale = Vec3::splat(0.92 + (t * 0.8).sin() * 0.10);
                set_sprite_alpha(&mut sprite, particle.color, 0.70 + 0.20 * (t * 0.64).sin());
            }
            AmbientKind::Firefly => {
                transform.translation.x = particle.origin.x + t.sin() * particle.amplitude.x;
                transform.translation.y =
                    particle.origin.y + (t * 1.35).cos() * particle.amplitude.y;
                transform.scale = Vec3::splat(0.82 + (t * 2.4).sin().abs() * 0.38);
                set_sprite_alpha(
                    &mut sprite,
                    particle.color,
                    0.62 + 0.34 * (t * 2.1).sin().abs(),
                );
            }
            AmbientKind::Spark => {
                transform.translation.x =
                    particle.origin.x + (t * 1.2).sin() * particle.amplitude.x;
                transform.translation.y =
                    particle.origin.y + (t * 1.6).cos() * particle.amplitude.y;
                transform.scale = Vec3::splat(0.74 + (t * 3.6).sin().abs() * 0.46);
                set_sprite_alpha(
                    &mut sprite,
                    particle.color,
                    0.48 + 0.42 * (t * 2.8).sin().abs(),
                );
            }
        }
    }
}

fn set_sprite_alpha(sprite: &mut Sprite, base: Color, alpha_multiplier: f32) {
    let srgba = base.to_srgba();
    sprite.color = Color::srgba(
        srgba.red,
        srgba.green,
        srgba.blue,
        (srgba.alpha * alpha_multiplier).clamp(0.02, 1.0),
    );
}

fn color_with_alpha_scale(base: Color, alpha_scale: f32) -> Color {
    let srgba = base.to_srgba();
    Color::srgba(
        srgba.red,
        srgba.green,
        srgba.blue,
        (srgba.alpha * alpha_scale).clamp(0.0, 1.0),
    )
}

fn update_area_banners(
    time: Res<Time>,
    mut commands: Commands,
    mut banners: Query<(Entity, &mut AreaBanner, &mut BackgroundColor)>,
) {
    for (entity, mut banner, mut background) in &mut banners {
        banner.age += time.delta_secs();
        let p = (banner.age / banner.duration).clamp(0.0, 1.0);
        let fade = if p < 0.72 {
            1.0
        } else {
            1.0 - ((p - 0.72) / 0.28).clamp(0.0, 1.0)
        };
        *background = BackgroundColor(color_with_alpha_scale(banner.base_color, fade));

        if banner.age >= banner.duration {
            commands.entity(entity).despawn();
        }
    }
}

fn quest_notice_colors(kind: QuestNoticeKind) -> (Color, Color) {
    match kind {
        QuestNoticeKind::Main => (
            Color::srgba(0.08, 0.08, 0.18, 0.88),
            Color::srgba(0.86, 0.60, 1.0, 0.72),
        ),
        QuestNoticeKind::Side => (
            Color::srgba(0.05, 0.11, 0.10, 0.88),
            Color::srgba(0.56, 0.96, 0.68, 0.72),
        ),
        QuestNoticeKind::Complete => (
            Color::srgba(0.13, 0.09, 0.03, 0.90),
            Color::srgba(1.0, 0.78, 0.34, 0.76),
        ),
    }
}

fn update_quest_notice(
    time: Res<Time>,
    mut notice: ResMut<QuestNotice>,
    mut root: Query<
        (&mut Visibility, &mut BackgroundColor, &mut BorderColor),
        With<QuestNoticeRoot>,
    >,
    mut title: Query<&mut Text, (With<QuestNoticeTitle>, Without<QuestNoticeBody>)>,
    mut body: Query<&mut Text, (With<QuestNoticeBody>, Without<QuestNoticeTitle>)>,
) {
    if notice.active {
        notice.age += time.delta_secs();
        if notice.age >= notice.duration {
            notice.active = false;
        }
    }

    if let Ok((mut visibility, mut background, mut border)) = root.single_mut() {
        *visibility = if notice.active {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };

        let progress = if notice.duration > 0.0 {
            (notice.age / notice.duration).clamp(0.0, 1.0)
        } else {
            1.0
        };
        let fade_in = (progress / 0.10).clamp(0.0, 1.0);
        let fade_out = if progress < 0.76 {
            1.0
        } else {
            1.0 - ((progress - 0.76) / 0.24).clamp(0.0, 1.0)
        };
        let fade = fade_in.min(fade_out);
        let (panel_color, border_color) = quest_notice_colors(notice.kind);
        *background = BackgroundColor(color_with_alpha_scale(panel_color, fade));
        *border = BorderColor::all(color_with_alpha_scale(border_color, fade));
    }

    if let Ok(mut text) = title.single_mut() {
        text.0 = if notice.active {
            notice.title.clone()
        } else {
            String::new()
        };
    }
    if let Ok(mut text) = body.single_mut() {
        text.0 = if notice.active {
            notice.body.clone()
        } else {
            String::new()
        };
    }
}

fn update_hud(
    stats: Res<PlayerStats>,
    quest: Res<QuestLog>,
    map: Option<Res<MapData>>,
    current: Res<CurrentMap>,
    mut q: Query<&mut Text, With<HudText>>,
) {
    if let Ok(mut text) = q.single_mut() {
        let map_name = map
            .as_ref()
            .map(|map| map.name)
            .unwrap_or_else(|| current.0.def().name);
        let key_items = compact_hud_text(&quest.key_items_summary(), 36);
        let chapter_seals = compact_hud_text(&quest.chapter_seal_summary(), 42);
        let gear = compact_hud_text(&quest.shop_gear_summary(), 42);
        let side_quests = compact_hud_text(&quest.side_quest_summary(), 42);
        let main_task = compact_hud_text(&quest.main_task_summary(), 56);
        let local_task = map
            .as_ref()
            .map(|map| compact_hud_text(&side_board_summary(map.kind, &quest), 56))
            .unwrap_or_else(|| "任务板 无".to_string());
        let area_task = map
            .as_ref()
            .map(|map| compact_hud_text(&active_area_side_task_summary(map.kind, &quest), 42))
            .unwrap_or_else(|| "当前委托区 无".to_string());
        let npc_errands = compact_hud_text(&quest.npc_errand_summary(), 42);
        let bonds = compact_hud_text(
            &format!(
                "{} {} {}",
                quest.bond_summary(),
                quest.companion_story_summary(),
                quest.companion_revisit_summary()
            ),
            42,
        );
        let camp = quest.camp_summary();
        let route_marks = compact_hud_text(&quest.route_memory_summary(), 42);
        let route_branch = compact_hud_text(&quest.route_branch_summary(), 42);
        let route_report = compact_hud_text(&quest.route_report_summary(), 42);
        let route_care = map
            .as_ref()
            .and_then(|map| route_pressure_summary(map.kind, &quest))
            .map(|summary| compact_hud_text(&summary, 42))
            .unwrap_or_else(|| "路线照应 无".to_string());
        let shrine = compact_hud_text(&quest.shrine_travel_summary(), 42);
        let care = compact_hud_text(&quest.travel_care_summary(), 42);
        let supplies = compact_hud_text(
            &format!(
                "{} {}",
                quest.field_supply_summary(),
                quest.commission_trace_summary()
            ),
            42,
        );
        text.0 = format!(
            "{}\n{}\n队伍 {}\n{}  Lv.{}\n气血 {}/{}\n灵力 {}/{}\n药水 x{}  钱 {}文\n装备 {}\n道具 {}\n{}\n任务簿 {}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}\n{}",
            map_name,
            quest.chapter_title(),
            quest.party_summary(),
            stats.name,
            stats.level,
            stats.hp.max(0),
            stats.max_hp,
            stats.mp.max(0),
            stats.max_mp,
            stats.potions,
            stats.gold,
            gear,
            key_items,
            chapter_seals,
            main_task,
            local_task,
            area_task,
            side_quests,
            npc_errands,
            bonds,
            camp,
            route_marks,
            route_branch,
            route_report,
            route_care,
            shrine,
            care,
            supplies,
        );
    }
}

fn update_task_tracker(
    quest: Res<QuestLog>,
    map: Option<Res<MapData>>,
    current: Res<CurrentMap>,
    mut root: Query<&mut Visibility, With<TaskTrackerRoot>>,
    mut text: Query<&mut Text, With<TaskTrackerText>>,
) {
    if let Ok(mut visibility) = root.single_mut() {
        *visibility = Visibility::Inherited;
    }

    if let Ok(mut text) = text.single_mut() {
        let kind = map.as_ref().map(|map| map.kind).unwrap_or(current.0);
        text.0 = task_tracker_text(kind, &quest);
    }
}

fn compact_hud_text(value: &str, limit: usize) -> String {
    if value.chars().count() <= limit {
        return value.to_string();
    }

    let mut compact: String = value.chars().take(limit).collect();
    compact.push_str("...");
    compact
}

#[cfg(test)]
fn side_quest_facing_player(kind: MapKind, pos: &PlayerPos, quest: &QuestLog) -> Option<SideQuest> {
    let tc = pos.col + pos.facing.x;
    let tr = pos.row + pos.facing.y;
    prop_defs(kind)
        .iter()
        .find(|prop| prop.col == tc && prop.row == tr)
        .and_then(|prop| current_side_quest_for_board(kind, prop, quest))
}

fn side_board_facing_prompt(kind: MapKind, pos: &PlayerPos, quest: &QuestLog) -> Option<String> {
    let tc = pos.col + pos.facing.x;
    let tr = pos.row + pos.facing.y;
    prop_defs(kind)
        .iter()
        .find(|prop| prop.col == tc && prop.row == tr)
        .filter(|prop| !side_quests_for_board(kind, prop).is_empty())
        .and_then(|_| side_board_prompt(kind, quest))
}

fn npc_side_quest_facing_prompt(
    kind: MapKind,
    pos: &PlayerPos,
    quest: &QuestLog,
) -> Option<String> {
    let tc = pos.col + pos.facing.x;
    let tr = pos.row + pos.facing.y;
    npc_defs(kind)
        .iter()
        .find(|npc| npc.col == tc && npc.row == tr)
        .and_then(|npc| npc_side_quest_contact_prompt(kind, npc, quest))
}

fn field_supply_facing_prompt(kind: MapKind, pos: &PlayerPos, quest: &QuestLog) -> Option<String> {
    let tc = pos.col + pos.facing.x;
    let tr = pos.row + pos.facing.y;
    field_supply_defs(kind)
        .find(|supply| supply.col == tc && supply.row == tr)
        .map(|supply| {
            let status = if quest.has_collected_supply(supply.supply) {
                "已采"
            } else {
                "可采"
            };
            format!(
                "采集点 {} [{}] | 空格查看\n{}",
                supply.supply.name(),
                status,
                quest.field_supply_summary()
            )
        })
}

fn commission_trace_facing_prompt(
    kind: MapKind,
    pos: &PlayerPos,
    quest: &QuestLog,
) -> Option<String> {
    let tc = pos.col + pos.facing.x;
    let tr = pos.row + pos.facing.y;
    commission_trace_defs(kind)
        .find(|trace| trace.col == tc && trace.row == tr)
        .map(|trace| {
            let status = if quest.is_side_quest_completed(trace.side) {
                "已归档"
            } else if quest.has_commission_trace(trace.side) {
                "已处理"
            } else if quest.is_side_quest_active(trace.side) {
                "可处理"
            } else if quest.is_side_quest_unlocked(trace.side) {
                "未领取"
            } else {
                "后续线索"
            };
            let action = if quest.is_side_quest_active(trace.side)
                && !quest.has_commission_trace(trace.side)
            {
                "处理"
            } else {
                "查看"
            };
            format!(
                "委托现场 {} [{}] | 空格{}\n{}",
                trace.name,
                status,
                action,
                quest.side_task_summary(trace.side)
            )
        })
}

fn companion_revisit_field_facing_prompt(
    kind: MapKind,
    pos: &PlayerPos,
    quest: &QuestLog,
) -> Option<String> {
    let tc = pos.col + pos.facing.x;
    let tr = pos.row + pos.facing.y;
    let mark = prop_defs(kind)
        .iter()
        .find(|prop| prop.col == tc && prop.row == tr)
        .and_then(|prop| route_mark_for_prop(kind, prop))?;
    let revisit = quest.active_companion_revisit_at(mark)?;
    Some(format!(
        "小传补访现场 {} [可寻访] | 空格接回旧话\n{}",
        revisit.name(),
        quest.companion_revisit_contract(revisit)
    ))
}

fn update_task_prompt(
    quest: Res<QuestLog>,
    pos: Res<PlayerPos>,
    map: Option<Res<MapData>>,
    dialogue: Res<Dialogue>,
    mut root: Query<&mut Visibility, With<TaskPromptRoot>>,
    mut line: Query<&mut Text, With<TaskPromptLine>>,
) {
    let prompt = if dialogue.active {
        None
    } else {
        map.as_ref().and_then(|map| {
            side_board_facing_prompt(map.kind, &pos, &quest)
                .or_else(|| npc_side_quest_facing_prompt(map.kind, &pos, &quest))
                .or_else(|| npc_errand_facing_prompt(map.kind, &pos, &quest))
                .or_else(|| npc_companion_revisit_facing_prompt(map.kind, &pos, &quest))
                .or_else(|| companion_revisit_field_facing_prompt(map.kind, &pos, &quest))
                .or_else(|| field_supply_facing_prompt(map.kind, &pos, &quest))
                .or_else(|| commission_trace_facing_prompt(map.kind, &pos, &quest))
        })
    };

    if let Ok(mut vis) = root.single_mut() {
        *vis = if prompt.is_some() {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }

    if let Ok(mut text) = line.single_mut() {
        text.0 = prompt.unwrap_or_default();
    }
}

fn update_quest_markers(
    time: Res<Time>,
    quest: Res<QuestLog>,
    mut q: Query<(
        &QuestMarker,
        &QuestMarkerPart,
        &mut Transform,
        &mut Visibility,
        Option<&mut Sprite>,
        Option<&mut Text2d>,
        Option<&mut TextColor>,
    )>,
) {
    let t = time.elapsed_secs();
    for (marker, part, mut transform, mut visibility, sprite, text, color) in &mut q {
        let glyph = quest_marker_glyph(&quest, marker.kind);
        let Some(style) = quest_marker_style(glyph) else {
            *visibility = Visibility::Hidden;
            continue;
        };

        *visibility = Visibility::Inherited;
        let bob = (t * 2.9 + marker.phase).sin() * 2.8;
        transform.translation.y = marker.base_y + bob + quest_marker_part_y_offset(*part);

        if let Some(mut sprite) = sprite {
            match part {
                QuestMarkerPart::Glow => sprite.color = style.glow,
                QuestMarkerPart::Badge => sprite.color = style.badge,
                QuestMarkerPart::GlyphShadow | QuestMarkerPart::Glyph => {}
            }
        }
        if let Some(mut text) = text {
            text.0 = glyph.to_string();
        }
        if let Some(mut color) = color {
            match part {
                QuestMarkerPart::GlyphShadow => {
                    *color = TextColor(Color::srgba(0.0, 0.0, 0.0, 0.72));
                }
                QuestMarkerPart::Glyph => {
                    *color = TextColor(style.glyph);
                }
                QuestMarkerPart::Glow | QuestMarkerPart::Badge => {}
            }
        }
    }
}

fn quest_marker_glyph(quest: &QuestLog, kind: QuestMarkerKind) -> &'static str {
    match kind {
        QuestMarkerKind::Role(role) => quest.marker_for(Some(role)),
        QuestMarkerKind::SideBoard(kind) => side_board_marker_for(quest, kind),
        QuestMarkerKind::SideContact(kind) => side_board_marker_for(quest, kind),
        QuestMarkerKind::NpcErrandOffer(errand) => quest.npc_errand_marker_for(errand),
        QuestMarkerKind::NpcErrandDelivery(errand) => quest.npc_errand_delivery_marker_for(errand),
        QuestMarkerKind::CompanionRevisitGiver(revisit) => {
            quest.companion_revisit_giver_marker_for(revisit)
        }
        QuestMarkerKind::CompanionRevisitField(revisit) => {
            quest.companion_revisit_field_marker_for(revisit)
        }
        QuestMarkerKind::CommissionTrace(side) => quest.commission_trace_marker_for(side),
        QuestMarkerKind::Lamp(lamp) => quest.final_lamp_marker_for(lamp),
        QuestMarkerKind::RiverLantern(lantern) => quest.river_lantern_marker_for(lantern),
        QuestMarkerKind::PlagueWard(ward) => quest.plague_ward_marker_for(ward),
        QuestMarkerKind::MansionMirror(node) => quest.mansion_mirror_marker_for(node),
        QuestMarkerKind::ThunderDrum(drum) => quest.thunder_drum_marker_for(drum),
        QuestMarkerKind::Crystal(crystal) => quest.moon_crystal_marker_for(crystal),
        QuestMarkerKind::RouteMark(mark) => quest.route_mark_marker_for(mark),
        QuestMarkerKind::RouteDetour(detour) => quest.route_detour_marker_for(detour),
        QuestMarkerKind::FieldSupply(supply) => quest.field_supply_marker_for(supply),
    }
}

fn quest_marker_style(glyph: &str) -> Option<QuestMarkerStyle> {
    match glyph {
        "!" => Some(QuestMarkerStyle {
            badge: Color::srgb(0.92, 0.47, 0.10),
            glow: Color::srgba(1.0, 0.74, 0.22, 0.36),
            glyph: Color::srgb(1.0, 0.98, 0.82),
        }),
        "*" => Some(QuestMarkerStyle {
            badge: Color::srgb(0.12, 0.54, 0.86),
            glow: Color::srgba(0.30, 0.78, 1.0, 0.30),
            glyph: Color::srgb(0.86, 0.98, 1.0),
        }),
        "？" => Some(QuestMarkerStyle {
            badge: Color::srgb(0.68, 0.42, 0.92),
            glow: Color::srgba(0.78, 0.55, 1.0, 0.34),
            glyph: Color::srgb(1.0, 0.94, 1.0),
        }),
        "✓" => Some(QuestMarkerStyle {
            badge: Color::srgb(0.14, 0.66, 0.32),
            glow: Color::srgba(0.38, 1.0, 0.58, 0.28),
            glyph: Color::srgb(0.90, 1.0, 0.88),
        }),
        _ => None,
    }
}

fn quest_marker_part_y_offset(part: QuestMarkerPart) -> f32 {
    match part {
        QuestMarkerPart::Glow | QuestMarkerPart::Badge => 0.0,
        QuestMarkerPart::GlyphShadow => -2.0,
        QuestMarkerPart::Glyph => -3.0,
    }
}

fn update_dialogue_ui(
    dialogue: Res<Dialogue>,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut root: Query<&mut Visibility, (With<DialogueRoot>, Without<DialoguePortrait>)>,
    mut line: Query<&mut Text, With<DialogueLine>>,
    mut portrait: Query<
        (&mut Visibility, &mut ImageNode),
        (With<DialoguePortrait>, Without<DialogueRoot>),
    >,
    mut chapter_art: Query<
        (&mut ChapterArtRoot, &mut Visibility, &mut ImageNode),
        (
            With<ChapterArtRoot>,
            Without<DialogueRoot>,
            Without<DialoguePortrait>,
        ),
    >,
) {
    let showing_chapter_art = dialogue.showing_chapter_art();
    if let Ok((mut art, mut vis, mut image)) = chapter_art.single_mut() {
        if showing_chapter_art {
            if let Some(assets) = dialogue.chapter_art {
                if art.active_sheet != Some(assets.sheet) {
                    art.active_sheet = Some(assets.sheet);
                    art.timer = 0.0;
                    art.frame = 0;
                    image.image = asset_server.load(assets.sheet);
                    image.texture_atlas = Some(TextureAtlas {
                        layout: art.layout.clone(),
                        index: 0,
                    });
                } else {
                    art.timer += time.delta_secs();
                    while art.timer >= CHAPTER_ART_FRAME_TIME {
                        art.timer -= CHAPTER_ART_FRAME_TIME;
                        art.frame = (art.frame + 1) % CHAPTER_ART_FRAME_COUNT;
                    }
                    if let Some(atlas) = image.texture_atlas.as_mut() {
                        atlas.index = art.frame;
                    }
                }
                image.color = Color::srgba(0.86, 0.90, 1.0, 0.92);
                *vis = Visibility::Inherited;
            }
        } else {
            art.active_sheet = None;
            art.timer = 0.0;
            art.frame = 0;
            image.texture_atlas = None;
            *vis = Visibility::Hidden;
        }
    }

    if let Ok(mut vis) = root.single_mut() {
        *vis = if dialogue.active {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
    if let Ok((mut vis, mut image)) = portrait.single_mut() {
        if dialogue.active && !showing_chapter_art {
            if let Some(path) = dialogue.portrait_path {
                image.image = asset_server.load(path);
                image.color = Color::WHITE;
                *vis = Visibility::Inherited;
            } else {
                *vis = Visibility::Hidden;
            }
        } else {
            *vis = Visibility::Hidden;
        }
    }
    if let Ok(mut text) = line.single_mut() {
        if dialogue.active && dialogue.choice.is_some() && dialogue.idx >= dialogue.lines.len() {
            let choice = dialogue.choice.expect("checked choice");
            let mut options = String::new();
            for index in 0..choice.option_count() {
                let cursor = if choice.selected == index { "▶" } else { " " };
                options.push_str(&format!("{cursor} {}\n", choice.option_label(index)));
            }
            text.0 = format!(
                "{}\n\n{}\n        （上下选择 · 空格确定）",
                choice.prompt(),
                options.trim_end()
            );
            return;
        }

        let body = if dialogue.active {
            dialogue
                .lines
                .get(dialogue.idx)
                .cloned()
                .unwrap_or_default()
        } else {
            String::new()
        };
        text.0 = format!("{}\n\n        （空格继续）", body);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn terrain_x_edges(sample: TerrainSample) -> (f32, f32) {
        if sample.flip_x {
            (sample.rect.max.x, sample.rect.min.x)
        } else {
            (sample.rect.min.x, sample.rect.max.x)
        }
    }

    fn terrain_y_edges(sample: TerrainSample) -> (f32, f32) {
        if sample.flip_y {
            (sample.rect.max.y, sample.rect.min.y)
        } else {
            (sample.rect.min.y, sample.rect.max.y)
        }
    }

    #[test]
    fn terrain_samples_keep_all_shared_edges_continuous() {
        for row in -24..=24 {
            for col in -24..=24 {
                let sample = terrain_sample(col, row);
                let right = terrain_x_edges(sample).1;
                let next_left = terrain_x_edges(terrain_sample(col + 1, row)).0;
                assert_eq!(right, next_left, "horizontal edge at ({col}, {row})");

                let bottom = terrain_y_edges(sample).1;
                let next_top = terrain_y_edges(terrain_sample(col, row + 1)).0;
                assert_eq!(bottom, next_top, "vertical edge at ({col}, {row})");

                assert_eq!(sample.rect.width(), TERRAIN_SAMPLE_SIZE);
                assert_eq!(sample.rect.height(), TERRAIN_SAMPLE_SIZE);
                assert!(sample.rect.min.x >= 0.0 && sample.rect.max.x <= TERRAIN_SOURCE_SIZE);
                assert!(sample.rect.min.y >= 0.0 && sample.rect.max.y <= TERRAIN_SOURCE_SIZE);
            }
        }
    }

    #[test]
    fn terrain_samples_flip_only_on_reverse_sweeps() {
        for col in 0..TERRAIN_SAMPLES_PER_AXIS {
            let forward = terrain_sample(col, 0);
            assert_eq!(
                forward.rect.min.x,
                TERRAIN_SOURCE_INSET + col as f32 * TERRAIN_SAMPLE_SIZE
            );
            assert!(!forward.flip_x);

            let reverse_col = TERRAIN_SAMPLES_PER_AXIS + col;
            let reverse = terrain_sample(reverse_col, 0);
            let source_col = TERRAIN_SAMPLES_PER_AXIS - 1 - col;
            assert_eq!(
                reverse.rect.min.x,
                TERRAIN_SOURCE_INSET + source_col as f32 * TERRAIN_SAMPLE_SIZE
            );
            assert!(reverse.flip_x);
        }
    }

    #[test]
    fn terrain_material_ids_match_visual_layers() {
        assert_eq!(terrain_id(Tile::Path), 0);
        assert_eq!(terrain_id(Tile::Npc), 0);
        assert_eq!(terrain_id(Tile::Portal), 0);
        assert_eq!(terrain_id(Tile::Grass), 1);
        assert_eq!(terrain_id(Tile::Wall), 2);
        assert_eq!(terrain_id(Tile::Water), 3);
    }

    #[test]
    fn chapter_arts_cover_all_story_chapters() {
        assert_eq!(
            chapter_art_assets(Chapter::VillageOath),
            ChapterArtAssets {
                still: "ui/chapter1_art.png",
                sheet: "ui/anim/chapter1_sheet.png"
            }
        );
        assert_eq!(
            chapter_art_assets(Chapter::MoonCave),
            ChapterArtAssets {
                still: "ui/chapter2_art.png",
                sheet: "ui/anim/chapter2_sheet.png"
            }
        );
        assert_eq!(
            chapter_art_assets(Chapter::RiverMedicine),
            ChapterArtAssets {
                still: "ui/chapter3_art.png",
                sheet: "ui/anim/chapter3_sheet.png"
            }
        );
        assert_eq!(
            chapter_art_assets(Chapter::PlagueRain),
            ChapterArtAssets {
                still: "ui/chapter4_art.png",
                sheet: "ui/anim/chapter4_sheet.png"
            }
        );
        assert_eq!(
            chapter_art_assets(Chapter::CapitalMirror),
            ChapterArtAssets {
                still: "ui/chapter5_art.png",
                sheet: "ui/anim/chapter5_sheet.png"
            }
        );
        assert_eq!(
            chapter_art_assets(Chapter::SouthernThunder),
            ChapterArtAssets {
                still: "ui/chapter6_art.png",
                sheet: "ui/anim/chapter6_sheet.png"
            }
        );
        assert_eq!(
            chapter_art_assets(Chapter::FinalDream),
            ChapterArtAssets {
                still: "ui/chapter7_art.png",
                sheet: "ui/anim/chapter7_sheet.png"
            }
        );
    }

    #[test]
    fn dialogue_chapter_art_only_shows_during_chapter_card_lines() {
        let mut dialogue = Dialogue {
            active: true,
            lines: vec![
                "守灯人：先接下任务。".to_string(),
                "【卷章展开】终章 灵渊宿梦".to_string(),
                "【卷章画面】灵渊终门沉在紫蓝梦水中。".to_string(),
                "【卷章基调】旧梦回潮。".to_string(),
                "灵渊终门映出旧梦。".to_string(),
                "【卷章玩法】点亮三盏忆梦灯。".to_string(),
                "【下一步】进入旧梦水廊。".to_string(),
                "主线 进行中：进入旧梦水廊。".to_string(),
            ],
            idx: 0,
            after: DialogueAfter::None,
            choice: None,
            portrait_path: Some("npcs/ai_final_oracle.png"),
            chapter_art: Some(chapter_art_assets(Chapter::FinalDream)),
        };

        assert!(!dialogue.showing_chapter_art());
        dialogue.idx = 1;
        assert!(dialogue.showing_chapter_art());
        dialogue.idx = 6;
        assert!(dialogue.showing_chapter_art());
        dialogue.idx = 7;
        assert!(!dialogue.showing_chapter_art());
        dialogue.choice = Some(DialogueChoice::bond_response());
        dialogue.idx = 1;
        assert!(!dialogue.showing_chapter_art());
    }

    #[test]
    fn main_quest_accept_result_carries_chapter_art() {
        let mut quest = QuestLog::default();
        let mut stats = PlayerStats::default();
        let choice =
            DialogueChoice::main_quest(QuestRole::SwordSister, MainQuestChoiceAction::Accept);

        let result = resolve_dialogue_choice(choice, &mut quest, &mut stats);

        assert_eq!(
            result.chapter_art,
            Some(chapter_art_assets(Chapter::VillageOath))
        );
        assert!(
            result
                .lines
                .iter()
                .any(|line| line.contains("【卷章展开】"))
        );
        assert!(quest.has_seen_chapter_card(Chapter::VillageOath));
    }

    fn complete_side_quest(quest: &mut QuestLog, side: SideQuest) {
        let accepted = quest.interact_side_quest(side);
        assert!(accepted.reward.is_none());
        for _ in 0..quest.side_quest_goal(side) {
            quest.record_side_victory();
        }
        let completed = quest.interact_side_quest(side);
        assert!(completed.reward.is_some());
        assert!(quest.is_side_quest_completed(side));
    }

    fn complete_side_quest_with_resolution(
        quest: &mut QuestLog,
        side: SideQuest,
        resolution: SideQuestResolution,
    ) {
        let accepted = quest.interact_side_quest(side);
        assert!(accepted.reward.is_none());
        for _ in 0..quest.side_quest_goal(side) {
            quest.record_side_victory();
        }
        let completed = quest.interact_side_quest_with_resolution(side, resolution);
        assert!(completed.reward.is_some());
        assert!(quest.is_side_quest_completed(side));
        assert_eq!(quest.side_quest_resolution(side), Some(resolution));
    }

    fn quest_at_river_lantern_puzzle() -> QuestLog {
        let mut quest = QuestLog::default();
        quest.talk(QuestRole::SwordSister);
        quest.talk(QuestRole::Linger);
        quest.talk(QuestRole::StarMage);
        quest.record_victory();
        quest.record_victory();
        quest.talk(QuestRole::SwordSister);
        quest.talk(QuestRole::Merchant);
        quest.talk(QuestRole::BambooScout);
        quest.talk(QuestRole::CavePriestess);
        quest.record_victory();
        quest.record_victory();
        quest.record_victory();
        quest.activate_moon_crystal(MoonCrystal::North);
        quest.activate_moon_crystal(MoonCrystal::South);
        quest.record_boss_victory(BossKind::MoonWraith);
        quest.talk(QuestRole::Linger);
        quest.talk(QuestRole::HerbHealer);
        quest.record_victory();
        quest.record_victory();
        quest.talk(QuestRole::HerbHealer);
        quest.talk(QuestRole::RiverBoatman);
        quest
    }

    fn quest_at_moon_route(with_bond: bool, with_camp: bool) -> QuestLog {
        let mut quest = QuestLog::default();
        quest.talk(QuestRole::SwordSister);
        quest.talk(QuestRole::Linger);
        quest.talk(QuestRole::StarMage);
        quest.record_victory();
        quest.record_victory();
        quest.talk(QuestRole::SwordSister);
        quest.talk(QuestRole::Merchant);
        quest.talk(QuestRole::BambooScout);
        if with_bond {
            quest.interact_bond_scene();
        }
        if with_camp {
            quest.interact_camp_scene();
        }
        quest.talk(QuestRole::CavePriestess);
        quest
    }

    fn quest_at_moon_route_with_trail_companion_scene() -> QuestLog {
        let mut quest = QuestLog::default();
        quest.talk(QuestRole::SwordSister);
        quest.talk(QuestRole::Linger);
        quest.talk(QuestRole::StarMage);
        quest.record_victory();
        quest.record_victory();
        quest.talk(QuestRole::SwordSister);
        quest.interact_bond_scene();
        quest.interact_companion_scene();
        quest.talk(QuestRole::Merchant);
        quest.talk(QuestRole::BambooScout);
        quest.talk(QuestRole::CavePriestess);
        quest
    }

    fn quest_at_mansion_mirror_puzzle() -> QuestLog {
        let mut quest = quest_at_plague_ward_puzzle();
        quest.seal_plague_ward(PlagueWard::OldShrine);
        quest.seal_plague_ward(PlagueWard::BitterWell);
        quest.seal_plague_ward(PlagueWard::Sickroom);
        quest.talk(QuestRole::ShrineKeeper);
        quest.record_boss_victory(BossKind::MiasmaRoot);
        quest.talk(QuestRole::CapitalEnvoy);
        quest.talk(QuestRole::MansionSpy);
        quest.record_victory();
        quest.record_victory();
        quest
    }

    fn quest_at_plague_ward_puzzle() -> QuestLog {
        let mut quest = quest_at_river_lantern_puzzle();
        quest.activate_river_lantern(RiverLantern::Upstream);
        quest.activate_river_lantern(RiverLantern::Midstream);
        quest.activate_river_lantern(RiverLantern::Dock);
        quest.record_boss_victory(BossKind::RiverDemon);
        quest.talk(QuestRole::PlagueElder);
        quest.talk(QuestRole::ShrineKeeper);
        quest.record_victory();
        quest.record_victory();
        quest.record_victory();
        quest
    }

    fn quest_at_thunder_drum_puzzle() -> QuestLog {
        let mut quest = quest_at_mansion_mirror_puzzle();
        quest.align_mansion_mirror(MansionMirrorNode::Ledger);
        quest.align_mansion_mirror(MansionMirrorNode::Witness);
        quest.talk(QuestRole::MansionSpy);
        quest.record_boss_victory(BossKind::MirrorMinister);
        quest.talk(QuestRole::SpiritGuide);
        quest.talk(QuestRole::TribalChief);
        quest.record_victory();
        quest.record_victory();
        quest.record_victory();
        quest
    }

    #[test]
    fn story_gates_later_maps() {
        assert!(
            portal_gate_message(MapKind::Village, MapKind::Bamboo, QuestStage::NotStarted)
                .is_some()
        );
        assert!(
            portal_gate_message(MapKind::Village, MapKind::Bamboo, QuestStage::FindStarMage)
                .is_none()
        );
        assert!(
            portal_gate_message(MapKind::Bamboo, MapKind::Cave, QuestStage::FindBambooScout)
                .is_some()
        );
        assert!(
            portal_gate_message(
                MapKind::Bamboo,
                MapKind::Cave,
                QuestStage::SeekCavePriestess
            )
            .is_none()
        );
        assert!(matches!(
            MapKind::Cave.portal_target(QuestStage::ReturnToLinger),
            MapKind::Village
        ));
        assert!(matches!(
            MapKind::Cave.portal_target(QuestStage::CaveTrial { remaining: 2 }),
            MapKind::MoonEchoCorridor
        ));
        assert!(matches!(
            MapKind::MoonEchoCorridor.portal_target(QuestStage::CaveTrial { remaining: 0 }),
            MapKind::Cave
        ));
        assert!(matches!(
            MapKind::Cave.portal_target(QuestStage::OpeningComplete),
            MapKind::RiverTown
        ));
        assert!(matches!(
            MapKind::RiverTown.portal_target(QuestStage::FindRiverBoatman),
            MapKind::Village
        ));
        assert!(matches!(
            MapKind::RiverTown.portal_target(QuestStage::TuneRiverLanterns),
            MapKind::RiverReedBed
        ));
        assert!(matches!(
            MapKind::RiverTown.portal_target(QuestStage::RiverTownComplete),
            MapKind::RiverReedBed
        ));
        assert!(matches!(
            MapKind::RiverReedBed.portal_target(QuestStage::TuneRiverLanterns),
            MapKind::RiverTown
        ));
        assert!(matches!(
            MapKind::RiverReedBed.portal_target(QuestStage::RiverTownComplete),
            MapKind::PlagueVillage
        ));
        assert!(matches!(
            MapKind::PlagueVillage.portal_target(QuestStage::SeekShrineKeeper),
            MapKind::RiverTown
        ));
        assert!(matches!(
            MapKind::PlagueVillage.portal_target(QuestStage::CleansePlagueShrines { remaining: 2 }),
            MapKind::PlagueShrinePath
        ));
        assert!(matches!(
            MapKind::PlagueVillage.portal_target(QuestStage::SealPlagueWards),
            MapKind::PlagueShrinePath
        ));
        assert!(matches!(
            MapKind::PlagueShrinePath.portal_target(QuestStage::ReturnToShrineKeeper),
            MapKind::PlagueVillage
        ));
        assert!(matches!(
            MapKind::PlagueVillage.portal_target(QuestStage::PlagueVillageComplete),
            MapKind::Capital
        ));
        assert!(matches!(
            MapKind::Capital.portal_target(QuestStage::FindMansionSpy),
            MapKind::CapitalMansion
        ));
        assert!(
            portal_gate_message(
                MapKind::Capital,
                MapKind::CapitalMansion,
                QuestStage::PlagueVillageComplete
            )
            .is_some()
        );
        assert!(
            portal_gate_message(
                MapKind::Capital,
                MapKind::CapitalMansion,
                QuestStage::FindMansionSpy
            )
            .is_none()
        );
        assert!(matches!(
            MapKind::CapitalMansion.portal_target(QuestStage::GatherSecretLetters { remaining: 1 }),
            MapKind::MansionMirrorGallery
        ));
        assert!(matches!(
            MapKind::MansionMirrorGallery
                .portal_target(QuestStage::GatherSecretLetters { remaining: 1 }),
            MapKind::CapitalMansion
        ));
        assert!(matches!(
            MapKind::Capital.portal_target(QuestStage::AlignMansionMirrors),
            MapKind::CapitalMansion
        ));
        assert!(matches!(
            MapKind::CapitalMansion.portal_target(QuestStage::AlignMansionMirrors),
            MapKind::MansionMirrorGallery
        ));
        assert!(matches!(
            MapKind::MansionMirrorGallery.portal_target(QuestStage::ReturnToMansionSpy),
            MapKind::CapitalMansion
        ));
        assert!(matches!(
            MapKind::Capital.portal_target(QuestStage::CapitalIntrigueComplete),
            MapKind::SouthernRoad
        ));
        assert!(matches!(
            MapKind::SouthernRoad.portal_target(QuestStage::SeekTribalChief),
            MapKind::Capital
        ));
        assert!(matches!(
            MapKind::SouthernRoad.portal_target(QuestStage::CleanseSpiritTotems { remaining: 2 }),
            MapKind::ThunderDrumPath
        ));
        assert!(matches!(
            MapKind::SouthernRoad.portal_target(QuestStage::AlignThunderDrums),
            MapKind::ThunderDrumPath
        ));
        assert!(matches!(
            MapKind::ThunderDrumPath.portal_target(QuestStage::ReturnToTribalChief),
            MapKind::SouthernRoad
        ));
        assert!(matches!(
            MapKind::SouthernRoad.portal_target(QuestStage::SouthernRoadComplete),
            MapKind::FinalSanctum
        ));
        assert!(matches!(
            MapKind::FinalSanctum.portal_target(QuestStage::LightFinalSoulLamps { remaining: 2 }),
            MapKind::DreamWaterway
        ));
        assert!(matches!(
            MapKind::DreamWaterway.portal_target(QuestStage::ReturnToFinalOracle),
            MapKind::FinalSanctum
        ));
        assert!(matches!(
            MapKind::FinalSanctum.portal_target(QuestStage::ReturnToFinalOracle),
            MapKind::SouthernRoad
        ));
    }

    #[test]
    fn portal_transition_lines_name_destination_and_active_objective() {
        let mut quest = QuestLog::default();
        quest.talk(QuestRole::SwordSister);
        quest.talk(QuestRole::Linger);
        let lines = portal_transition_lines(MapKind::Village, MapKind::Bamboo, &quest);
        assert!(lines[0].contains("余杭村郊 -> 青竹山径"));
        assert!(lines.iter().any(|line| line.contains("主线簿 卷1-03")));
        assert!(
            lines
                .iter()
                .any(|line| line.contains("当前主线目标地") && line.contains("星咒童子"))
        );
        assert!(lines.iter().any(|line| line.contains("【目标】")));

        let river = quest_at_river_lantern_puzzle();
        let lines = portal_transition_lines(MapKind::RiverTown, MapKind::RiverReedBed, &river);
        assert!(lines[0].contains("江岸小镇 -> 江岸芦滩"));
        assert!(
            lines
                .iter()
                .any(|line| line.contains("当前主线目标地") && line.contains("倒流河灯"))
        );

        let backtrack = portal_transition_lines(MapKind::RiverTown, MapKind::Village, &river);
        assert!(
            backtrack
                .iter()
                .any(|line| line.contains("任务引路仍指向") && line.contains("江岸芦滩"))
        );
    }

    #[test]
    fn area_banner_names_region_chapter_route_and_goal() {
        let quest = QuestLog::default();
        let map = MapData::build(MapKind::Village);
        let lines = area_banner_lines(&map, &quest);

        assert_eq!(lines[0], "抵达 · 余杭村郊");
        assert!(lines[1].contains("第一卷 村誓与灵符"));
        assert!(lines[1].contains("青竹山径"));
        assert!(lines[1].contains("需推进剧情"));
        assert!(lines[2].contains("目标"));
        assert!(lines[2].contains("红衣剑姊"));
        assert_eq!(
            area_route_label(MapKind::Cave, QuestStage::OpeningComplete),
            "下一程：江岸小镇"
        );
    }

    #[test]
    fn all_maps_have_valid_rows_and_spawns() {
        for kind in authored_map_kinds() {
            let map = MapData::build(kind);
            let (col, row) = map.spawn();
            assert!(map.at(col, row).walkable());
        }
    }

    #[test]
    fn field_supply_nodes_cover_every_authored_map_on_walkable_tiles() {
        let mut total = 0;
        for kind in authored_map_kinds() {
            let supplies = field_supply_defs(kind).collect::<Vec<_>>();
            assert_eq!(supplies.len(), 1, "{kind:?} should have one field supply");

            let supply = supplies[0];
            let map = MapData::build(kind);
            assert!(
                map.at(supply.col, supply.row).walkable(),
                "{kind:?} supply {:?} should sit on a walkable tile",
                supply.supply
            );
            total += supplies.len();
        }
        assert_eq!(total, authored_map_kinds().len());
    }

    #[test]
    fn commission_trace_nodes_cover_every_side_quest_on_walkable_tiles() {
        assert_eq!(COMMISSION_TRACE_DEFS.len(), ALL_LOCAL_SIDE_QUESTS.len());
        for side in ALL_LOCAL_SIDE_QUESTS {
            let traces = COMMISSION_TRACE_DEFS
                .iter()
                .filter(|trace| trace.side == side)
                .collect::<Vec<_>>();
            assert_eq!(traces.len(), 1, "{side:?} should have one field trace");
            let trace = traces[0];
            assert!(
                side_quest_target_matches_map(side, trace.kind),
                "{side:?} trace should be inside its target area"
            );
        }

        for trace in COMMISSION_TRACE_DEFS {
            let map = MapData::build(trace.kind);
            assert!(
                map.at(trace.col, trace.row).walkable(),
                "{:?} trace {} should sit on a walkable tile",
                trace.side,
                trace.name
            );
            assert!(
                !prop_defs(trace.kind)
                    .iter()
                    .any(|prop| prop.col == trace.col && prop.row == trace.row),
                "{:?} trace {} overlaps a prop",
                trace.side,
                trace.name
            );
            assert!(
                !npc_defs(trace.kind)
                    .iter()
                    .any(|npc| npc.col == trace.col && npc.row == trace.row),
                "{:?} trace {} overlaps an NPC",
                trace.side,
                trace.name
            );
            assert!(
                !field_supply_defs(trace.kind)
                    .any(|supply| supply.col == trace.col && supply.row == trace.row),
                "{:?} trace {} overlaps a field supply",
                trace.side,
                trace.name
            );
        }
    }

    fn authored_map_kinds() -> [MapKind; 15] {
        [
            MapKind::Village,
            MapKind::Bamboo,
            MapKind::Cave,
            MapKind::MoonEchoCorridor,
            MapKind::RiverTown,
            MapKind::RiverReedBed,
            MapKind::PlagueVillage,
            MapKind::PlagueShrinePath,
            MapKind::Capital,
            MapKind::CapitalMansion,
            MapKind::MansionMirrorGallery,
            MapKind::SouthernRoad,
            MapKind::ThunderDrumPath,
            MapKind::FinalSanctum,
            MapKind::DreamWaterway,
        ]
    }

    fn differing_map_rows(a: MapKind, b: MapKind) -> usize {
        a.def()
            .rows
            .iter()
            .zip(b.def().rows.iter())
            .filter(|(left, right)| left != right)
            .count()
    }

    #[test]
    fn authored_maps_keep_distinct_layouts() {
        let maps = authored_map_kinds();
        for (index, left) in maps.iter().copied().enumerate() {
            for right in maps.iter().copied().skip(index + 1) {
                assert!(
                    differing_map_rows(left, right) >= 5,
                    "{} and {} reuse too many map rows",
                    left.def().name,
                    right.def().name
                );
            }
        }

        for (left, right) in [
            (MapKind::RiverReedBed, MapKind::MoonEchoCorridor),
            (MapKind::PlagueVillage, MapKind::RiverTown),
            (MapKind::PlagueShrinePath, MapKind::SouthernRoad),
            (MapKind::Capital, MapKind::PlagueVillage),
            (MapKind::Capital, MapKind::CapitalMansion),
            (MapKind::SouthernRoad, MapKind::ThunderDrumPath),
            (MapKind::MansionMirrorGallery, MapKind::FinalSanctum),
            (MapKind::FinalSanctum, MapKind::DreamWaterway),
        ] {
            assert!(
                differing_map_rows(left, right) >= 10,
                "{} and {} should read as separate chapter spaces",
                left.def().name,
                right.def().name
            );
        }
    }

    #[test]
    fn maps_have_chapter_specific_ambient_profiles() {
        assert_eq!(ambient_profile(MapKind::Village).kind, AmbientKind::Firefly);
        assert_eq!(ambient_profile(MapKind::Bamboo).kind, AmbientKind::Leaf);
        assert_eq!(ambient_profile(MapKind::Cave).kind, AmbientKind::Mist);
        assert_eq!(
            ambient_profile(MapKind::MoonEchoCorridor).kind,
            AmbientKind::Mist
        );
        assert_eq!(
            ambient_profile(MapKind::PlagueVillage).kind,
            AmbientKind::Rain
        );
        assert_eq!(
            ambient_profile(MapKind::PlagueShrinePath).kind,
            AmbientKind::Mist
        );
        assert_eq!(
            ambient_profile(MapKind::MansionMirrorGallery).kind,
            AmbientKind::Spark
        );
        assert_eq!(
            ambient_profile(MapKind::ThunderDrumPath).kind,
            AmbientKind::Spark
        );
        assert_eq!(
            ambient_profile(MapKind::SouthernRoad).kind,
            AmbientKind::Spark
        );
        assert_eq!(
            ambient_profile(MapKind::DreamWaterway).kind,
            AmbientKind::Mist
        );
        assert!(
            ambient_profile(MapKind::RiverReedBed).count > ambient_profile(MapKind::Village).count
        );
        assert!(ambient_profile(MapKind::FinalSanctum).count >= 30);
    }

    #[test]
    fn quest_boards_map_to_side_quests() {
        assert_eq!(
            side_quests_for_board(MapKind::Village, &PROPS_VILLAGE[0]),
            [SideQuest::VillageTrail, SideQuest::VillageHerbs]
        );
        assert_eq!(
            current_side_quest_for_map(MapKind::Village, &QuestLog::default()),
            Some(SideQuest::VillageTrail)
        );
        assert_eq!(
            side_quests_for_board(MapKind::Cave, &PROPS_CAVE[2]),
            [SideQuest::MoonCaveCrystals, SideQuest::MoonCaveEchoes]
        );
        assert!(side_quests_for_map(MapKind::Bamboo).is_empty());
        assert!(side_quests_for_map(MapKind::MoonEchoCorridor).is_empty());
        assert_eq!(
            side_quests_for_board(MapKind::RiverTown, &PROPS_RIVER_TOWN[1]),
            [SideQuest::RiverLanterns, SideQuest::RiverCargo]
        );
        assert!(side_quests_for_map(MapKind::RiverReedBed).is_empty());
        assert_eq!(
            side_quests_for_board(MapKind::PlagueVillage, &PROPS_PLAGUE_VILLAGE[1]),
            [SideQuest::PlagueRelief, SideQuest::PlagueMedicine]
        );
        assert!(side_quests_for_map(MapKind::PlagueShrinePath).is_empty());
        assert_eq!(
            side_quests_for_board(MapKind::Capital, &PROPS_CAPITAL[1]),
            [SideQuest::CapitalPatrol, SideQuest::CapitalRumors]
        );
        assert!(side_quests_for_map(MapKind::CapitalMansion).is_empty());
        assert!(side_quests_for_map(MapKind::MansionMirrorGallery).is_empty());
        assert_eq!(
            side_quests_for_board(MapKind::SouthernRoad, &PROPS_SOUTHERN_ROAD[1]),
            [SideQuest::SouthernThunder, SideQuest::SouthernDrums]
        );
        assert!(side_quests_for_map(MapKind::ThunderDrumPath).is_empty());
        assert_eq!(
            side_quests_for_board(MapKind::FinalSanctum, &PROPS_FINAL_SANCTUM[1]),
            [SideQuest::FinalDreamEchoes, SideQuest::FinalHomewardVows]
        );
        assert_eq!(
            current_side_quest_for_map(MapKind::FinalSanctum, &QuestLog::default()),
            Some(SideQuest::FinalDreamEchoes)
        );
        assert!(side_quests_for_map(MapKind::DreamWaterway).is_empty());
        assert!(side_quests_for_board(MapKind::Village, &PROPS_VILLAGE[1]).is_empty());
        assert!(side_quests_for_board(MapKind::FinalSanctum, &PROPS_FINAL_SANCTUM[0]).is_empty());

        let mut quest = QuestLog::default();
        quest.interact_side_quest(SideQuest::VillageTrail);
        quest.record_side_victory();
        quest.record_side_victory();
        quest.interact_side_quest(SideQuest::VillageTrail);
        assert_eq!(
            current_side_quest_for_map(MapKind::Village, &quest),
            Some(SideQuest::VillageHerbs)
        );
    }

    #[test]
    fn accepted_commissions_surface_current_target_area() {
        let mut quest = QuestLog::default();
        assert_eq!(
            active_area_side_task_summary(MapKind::Village, &quest),
            "当前委托区 无"
        );

        quest.interact_side_quest(SideQuest::VillageTrail);
        let village = active_area_side_task_summary(MapKind::Village, &quest);
        assert!(village.contains("当前委托区 山路余妖 [进行中] 0/2"));
        assert!(village.contains("旧竹栅"));
        assert!(village.contains("现场：未处理现场"));
        assert_eq!(
            active_area_side_task_summary(MapKind::RiverReedBed, &quest),
            "当前委托区 无"
        );

        quest.record_side_victory();
        quest.record_side_victory();
        let ready = active_area_side_task_summary(MapKind::Village, &quest);
        assert!(ready.contains("山路余妖 [可交付] 2/2"));
        let bamboo = active_area_side_task_summary(MapKind::Bamboo, &quest);
        assert!(bamboo.contains("当前委托区 山路余妖 [可交付] 2/2"));
        assert!(bamboo.contains("旧竹栅"));

        let mut final_quest = QuestLog::default();
        final_quest.interact_side_quest(SideQuest::FinalDreamEchoes);
        let final_area = active_area_side_task_summary(MapKind::DreamWaterway, &final_quest);
        assert!(final_area.contains("当前委托区 梦灯余波 [进行中] 0/3"));
        assert!(final_area.contains("旧梦水廊"));
    }

    #[test]
    fn task_tracker_surfaces_local_pickup_before_acceptance() {
        let quest = QuestLog::default();
        let intake = local_task_intake_tracker(MapKind::Village, &quest)
            .expect("village board should advertise a claimable commission");
        assert!(intake.contains("本地可领 · 山路余妖 [可领取]"));
        assert!(intake.contains("签号：余杭-巡山-壹"));
        assert!(intake.contains("第一步：先签收委托"));
        assert!(intake.contains("现场：领取后可在现场选择细查或快断"));
        assert!(intake.contains("面对委托板/联系人按空格"));
        let tracker = task_tracker_text(MapKind::Village, &quest);
        assert!(tracker.contains("委托追踪：暂无已领取委托"));
        assert!(tracker.contains("本地可领 · 山路余妖 [可领取]"));

        let mut accepted = QuestLog::default();
        accepted.interact_side_quest(SideQuest::VillageTrail);
        assert!(local_task_intake_tracker(MapKind::Village, &accepted).is_none());
        let tracker = task_tracker_text(MapKind::Village, &accepted);
        assert!(tracker.contains("委托追踪 · 山路余妖 [进行中]"));
        assert!(!tracker.contains("本地可领"));

        accepted.record_side_victory();
        accepted.record_side_victory();
        accepted.interact_side_quest(SideQuest::VillageTrail);
        let intake = local_task_intake_tracker(MapKind::Village, &accepted)
            .expect("follow-up commission should become claimable after turn-in");
        assert!(intake.contains("本地可领 · 药圃护路 [可领取]"));
        assert!(intake.contains("签号：余杭-药圃-贰"));
    }

    #[test]
    fn task_tracker_guides_main_and_side_tasks_by_current_map() {
        let mut quest = QuestLog::default();
        let village = task_tracker_text(MapKind::Village, &quest);
        assert!(village.contains("任务引路"));
        assert!(village.contains("主线 · 当前地图：接取 · 红衣剑姊"));
        let bamboo = task_tracker_text(MapKind::Bamboo, &quest);
        assert!(bamboo.contains("主线 · 前往：余杭村郊东北 · 红衣剑姊"));

        quest.interact_side_quest(SideQuest::VillageTrail);
        let target = task_tracker_text(MapKind::Bamboo, &quest);
        assert!(target.contains("委托 · 目标区 当前地图：山路余妖 0/2"));
        let away = task_tracker_text(MapKind::Cave, &quest);
        assert!(away.contains("委托 · 前往目标区："));
        assert!(away.contains("0/2"));

        quest.record_side_victory();
        quest.record_side_victory();
        let delivery_here = task_tracker_text(MapKind::Village, &quest);
        assert!(delivery_here.contains("委托 · 可交付 当前地图：回余杭村郊任务板交付。"));
        let delivery_away = task_tracker_text(MapKind::Bamboo, &quest);
        assert!(delivery_away.contains("委托 · 可交付 前往：回余杭村郊任务板交付。"));
    }

    #[test]
    fn task_tracker_guides_active_npc_errand_to_delivery_map() {
        let mut quest = QuestLog::default();
        assert!(
            quest
                .accept_npc_errand(NpcErrand::BambooDewToCave)
                .reward
                .is_none()
        );

        let offer_map = task_tracker_text(MapKind::Bamboo, &quest);
        assert!(offer_map.contains("托付 · 前往交付：水月洞天 · 洞中采药人"));
        assert!(offer_map.contains("从青竹山径东门进入水月洞天"));

        let delivery_map = task_tracker_text(MapKind::Cave, &quest);
        assert!(delivery_map.contains("托付 · 交付地 当前地图：洞中采药人"));

        assert!(
            quest
                .complete_npc_errand(NpcErrand::BambooDewToCave)
                .reward
                .is_some()
        );
        assert!(!task_tracker_text(MapKind::Cave, &quest).contains("托付 ·"));
    }

    #[test]
    fn completed_commissions_reduce_matching_route_pressure() {
        let base = 0.20;
        let mut quest = QuestLog::default();
        assert_eq!(
            side_quest_route_relief_state(MapKind::MoonEchoCorridor, &quest),
            SideQuestRouteReliefState::NoRoute
        );
        assert_eq!(
            side_quest_route_relief_summary(MapKind::MoonEchoCorridor, &quest),
            None
        );
        assert_eq!(
            route_encounter_rate(base, MapKind::MoonEchoCorridor, &quest),
            base
        );

        complete_side_quest(&mut quest, SideQuest::MoonCaveCrystals);
        assert_eq!(
            side_quest_route_relief_state(MapKind::MoonEchoCorridor, &quest),
            SideQuestRouteReliefState::Partial
        );
        assert_eq!(
            side_quest_route_relief_summary(MapKind::MoonEchoCorridor, &quest),
            Some("委托清障 半稳 遇妖-3%".to_string())
        );
        assert!(
            (route_encounter_rate(base, MapKind::MoonEchoCorridor, &quest) - 0.194).abs() < 0.001
        );

        complete_side_quest(&mut quest, SideQuest::MoonCaveEchoes);
        assert_eq!(
            side_quest_route_relief_state(MapKind::MoonEchoCorridor, &quest),
            SideQuestRouteReliefState::Cleared
        );
        assert!(
            route_pressure_summary(MapKind::MoonEchoCorridor, &quest)
                .is_some_and(|line| line.contains("委托清障 已清"))
        );
        assert!(
            (route_encounter_rate(base, MapKind::MoonEchoCorridor, &quest) - 0.18).abs() < 0.001
        );
        assert!(
            side_quest_route_checkpoint_line(MapKind::MoonEchoCorridor, &quest)
                .is_some_and(|line| line.contains("晶尘") && line.contains("回声"))
        );

        let lines = npc_reaction_lines(
            MapKind::MoonEchoCorridor,
            &NPCS_MOON_ECHO_CORRIDOR[0],
            &quest,
        );
        assert!(lines.iter().any(|line| line.contains("委托回声")));
        assert!(
            lines
                .iter()
                .any(|line| line.contains("晶尘") && line.contains("回声"))
        );
    }

    #[test]
    fn npc_errand_deliveries_reduce_matching_route_pressure_and_echo_locally() {
        let base = 0.20;
        let mut moon = QuestLog::default();
        assert_eq!(
            npc_errand_route_relief_summary(MapKind::MoonEchoCorridor, &moon),
            None
        );
        let moon_before = route_encounter_rate(base, MapKind::MoonEchoCorridor, &moon);

        assert!(
            moon.accept_npc_errand(NpcErrand::BambooDewToCave)
                .reward
                .is_none()
        );
        assert!(
            moon.complete_npc_errand(NpcErrand::BambooDewToCave)
                .reward
                .is_some()
        );
        assert_eq!(
            npc_errand_route_relief_summary(MapKind::MoonEchoCorridor, &moon),
            Some("托付回声 已稳 遇妖-3%".to_string())
        );
        assert!(
            (route_encounter_rate(base, MapKind::MoonEchoCorridor, &moon) - moon_before * 0.97)
                .abs()
                < 0.001
        );
        assert!(
            route_pressure_summary(MapKind::MoonEchoCorridor, &moon)
                .is_some_and(|line| line.contains("托付回声 已稳"))
        );
        assert!(
            local_npc_errand_route_reaction(MapKind::MoonEchoCorridor, &moon)
                .is_some_and(|line| line.contains("竹露"))
        );
        assert!(
            npc_errand_route_checkpoint_line(MapKind::MoonEchoCorridor, &moon)
                .is_some_and(|line| line.contains("竹露"))
        );

        let mut river = quest_at_river_lantern_puzzle();
        let river_before = route_encounter_rate(base, MapKind::RiverReedBed, &river);
        assert!(
            river
                .accept_npc_errand(NpcErrand::MoonMossToRiver)
                .reward
                .is_none()
        );
        assert!(
            river
                .complete_npc_errand(NpcErrand::MoonMossToRiver)
                .reward
                .is_some()
        );
        assert_eq!(
            npc_errand_route_relief_summary(MapKind::RiverReedBed, &river),
            Some("托付回声 已稳 遇妖-3%".to_string())
        );
        assert!(
            (route_encounter_rate(base, MapKind::RiverReedBed, &river) - river_before * 0.97).abs()
                < 0.001
        );

        assert!(
            river
                .accept_npc_errand(NpcErrand::RiverReedLetter)
                .reward
                .is_none()
        );
        assert!(
            river
                .complete_npc_errand(NpcErrand::RiverReedLetter)
                .reward
                .is_some()
        );
        assert_eq!(
            npc_errand_route_relief_summary(MapKind::RiverReedBed, &river),
            Some("托付回声 连稳 遇妖-6%".to_string())
        );
        assert!(
            (route_encounter_rate(base, MapKind::RiverReedBed, &river) - river_before * 0.94).abs()
                < 0.001
        );
        assert!(
            local_npc_errand_route_reaction(MapKind::RiverReedBed, &river)
                .is_some_and(|line| line.contains("新浅渡"))
        );
        assert!(
            npc_errand_route_checkpoint_line(MapKind::RiverReedBed, &river)
                .is_some_and(|line| line.contains("新浅渡"))
        );
        let lines = npc_reaction_lines(MapKind::RiverReedBed, &NPCS_RIVER_REED_BED[0], &river);
        assert!(lines.iter().any(|line| line.contains("托付回声")));
    }

    #[test]
    fn commission_resolution_changes_route_pressure_and_npc_echo() {
        let base = 0.20;
        let mut quest = QuestLog::default();

        complete_side_quest_with_resolution(
            &mut quest,
            SideQuest::RiverLanterns,
            SideQuestResolution::Pursue,
        );

        assert_eq!(
            side_quest_route_relief_summary(MapKind::RiverReedBed, &quest),
            Some("委托清障 半稳 遇妖-5% · 追查1".to_string())
        );
        assert!((route_encounter_rate(base, MapKind::RiverReedBed, &quest) - 0.19).abs() < 0.001);
        let route_lines =
            npc_reaction_lines(MapKind::RiverReedBed, &NPCS_RIVER_REED_BED[0], &quest);
        assert!(route_lines.iter().any(|line| line.contains("裁断回声")));
        assert!(route_lines.iter().any(|line| line.contains("追查余波")));

        complete_side_quest_with_resolution(
            &mut quest,
            SideQuest::RiverCargo,
            SideQuestResolution::Settle,
        );
        assert_eq!(
            side_quest_route_relief_summary(MapKind::RiverReedBed, &quest),
            Some("委托清障 已清 遇妖-12% · 追查1".to_string())
        );
        let hub_lines = npc_reaction_lines(MapKind::RiverTown, &NPCS_RIVER_TOWN[2], &quest);
        assert!(hub_lines.iter().any(|line| line.contains("裁断回声")));
        assert!(
            hub_lines
                .iter()
                .any(|line| line.contains("封存") && line.contains("追查余波"))
        );
    }

    #[test]
    fn commission_field_choices_feed_route_pressure_and_street_talk() {
        let base = 0.20;
        let mut quest = QuestLog::default();
        let trace = COMMISSION_TRACE_DEFS
            .iter()
            .find(|trace| trace.side == SideQuest::RiverLanterns)
            .unwrap();

        quest.interact_side_quest(SideQuest::RiverLanterns);
        quest.interact_commission_trace_with_approach(
            trace.side,
            trace.name,
            trace.active_line,
            trace.source,
            trace.inactive_line,
            trace.repeat_line,
            SideQuestFieldApproach::Investigate,
        );
        quest.record_side_victory();
        quest.interact_side_quest_with_resolution(
            SideQuest::RiverLanterns,
            SideQuestResolution::Pursue,
        );

        assert_eq!(
            side_quest_route_relief_summary(MapKind::RiverReedBed, &quest),
            Some("委托清障 半稳 遇妖-6% · 追查1 · 细查1".to_string())
        );
        assert!((route_encounter_rate(base, MapKind::RiverReedBed, &quest) - 0.188).abs() < 0.001);
        let route_lines =
            npc_reaction_lines(MapKind::RiverReedBed, &NPCS_RIVER_REED_BED[0], &quest);
        assert!(route_lines.iter().any(|line| line.contains("现场回声")));
        assert!(route_lines.iter().any(|line| line.contains("细查现场")));
    }

    #[test]
    fn late_story_npcs_use_distinct_generated_cutouts() {
        fn role_image_path(kind: MapKind, role: QuestRole) -> Option<&'static str> {
            npc_defs(kind)
                .iter()
                .find(|npc| npc.quest == Some(role))
                .and_then(|npc| match npc.visual {
                    NpcVisual::Image { path, .. } => Some(path),
                    NpcVisual::Paperdoll(_) => None,
                })
        }

        let role_paths = [
            (
                MapKind::PlagueVillage,
                QuestRole::PlagueElder,
                "npcs/ai_plague_elder.png",
            ),
            (
                MapKind::PlagueVillage,
                QuestRole::ShrineKeeper,
                "npcs/ai_shrine_keeper.png",
            ),
            (
                MapKind::Capital,
                QuestRole::CapitalEnvoy,
                "npcs/ai_capital_envoy.png",
            ),
            (
                MapKind::CapitalMansion,
                QuestRole::MansionSpy,
                "npcs/ai_mansion_spy.png",
            ),
            (
                MapKind::SouthernRoad,
                QuestRole::SpiritGuide,
                "npcs/ai_spirit_guide.png",
            ),
            (
                MapKind::SouthernRoad,
                QuestRole::TribalChief,
                "npcs/ai_tribal_chief.png",
            ),
            (
                MapKind::FinalSanctum,
                QuestRole::FinalOracle,
                "npcs/ai_final_oracle.png",
            ),
        ];

        for (kind, role, expected) in role_paths {
            assert_eq!(role_image_path(kind, role), Some(expected));
        }
    }

    #[test]
    fn dialogue_portraits_use_ai_cutouts_when_available() {
        assert_eq!(
            dialogue_portrait_path(NPCS_VILLAGE[2].visual),
            Some("npcs/ai_sword_sister.png")
        );
        assert_eq!(dialogue_portrait_path(NPCS_VILLAGE[0].visual), None);
    }

    #[test]
    fn treasure_props_map_to_one_time_reward_caches() {
        assert_eq!(
            treasure_for_prop(MapKind::Village, &PROPS_VILLAGE[2]),
            Some(TreasureCache::VillageShrine)
        );
        assert_eq!(
            treasure_for_prop(MapKind::Bamboo, &PROPS_BAMBOO[3]),
            Some(TreasureCache::BambooOffering)
        );
        assert_eq!(
            treasure_for_prop(MapKind::Cave, &PROPS_CAVE[0]),
            Some(TreasureCache::CaveOffering)
        );
        assert_eq!(
            treasure_for_prop(MapKind::RiverTown, &PROPS_RIVER_TOWN[3]),
            Some(TreasureCache::RiverTownCrystal)
        );
        assert_eq!(
            treasure_for_prop(MapKind::RiverReedBed, &PROPS_RIVER_REED_BED[3]),
            Some(TreasureCache::RiverReedCrystal)
        );
        assert_eq!(
            treasure_for_prop(MapKind::PlagueShrinePath, &PROPS_PLAGUE_SHRINE_PATH[1]),
            Some(TreasureCache::PlagueShrine)
        );
        assert_eq!(
            treasure_for_prop(MapKind::Capital, &PROPS_CAPITAL[2]),
            Some(TreasureCache::CapitalShrine)
        );
        assert_eq!(
            treasure_for_prop(MapKind::CapitalMansion, &PROPS_CAPITAL_MANSION[4]),
            Some(TreasureCache::MansionMirror)
        );
        assert_eq!(
            treasure_for_prop(MapKind::SouthernRoad, &PROPS_SOUTHERN_ROAD[2]),
            Some(TreasureCache::SouthernTotem)
        );
        assert_eq!(
            treasure_for_prop(MapKind::FinalSanctum, &PROPS_FINAL_SANCTUM[0]),
            Some(TreasureCache::FinalMemoryCache)
        );

        assert_eq!(treasure_for_prop(MapKind::Village, &PROPS_VILLAGE[0]), None);
        assert_eq!(treasure_for_prop(MapKind::Cave, &PROPS_CAVE[1]), None);
        assert_eq!(
            treasure_for_prop(MapKind::FinalSanctum, &PROPS_FINAL_SANCTUM[1]),
            None
        );
        assert_eq!(
            treasure_for_prop(MapKind::ThunderDrumPath, &PROPS_THUNDER_DRUM_PATH[1]),
            None
        );
    }

    #[test]
    fn route_mark_props_add_optional_route_memory() {
        assert_eq!(
            route_mark_for_prop(MapKind::MoonEchoCorridor, &PROPS_MOON_ECHO_CORRIDOR[3]),
            Some(RouteMark::MoonEcho)
        );
        assert_eq!(
            route_mark_for_prop(MapKind::RiverReedBed, &PROPS_RIVER_REED_BED[5]),
            Some(RouteMark::ReedFord)
        );
        assert_eq!(
            route_mark_for_prop(MapKind::PlagueShrinePath, &PROPS_PLAGUE_SHRINE_PATH[4]),
            Some(RouteMark::PlagueBell)
        );
        assert_eq!(
            route_mark_for_prop(
                MapKind::MansionMirrorGallery,
                &PROPS_MANSION_MIRROR_GALLERY[0]
            ),
            Some(RouteMark::MirrorSideDoor)
        );
        assert_eq!(
            route_mark_for_prop(MapKind::ThunderDrumPath, &PROPS_THUNDER_DRUM_PATH[4]),
            Some(RouteMark::ThunderSwitchback)
        );
        assert_eq!(
            route_mark_for_prop(MapKind::DreamWaterway, &PROPS_DREAM_WATERWAY[4]),
            Some(RouteMark::DreamReturn)
        );

        assert_eq!(
            river_lantern_for_prop(MapKind::RiverReedBed, &PROPS_RIVER_REED_BED[0]),
            Some(RiverLantern::Upstream)
        );
        assert_eq!(
            thunder_drum_for_prop(MapKind::ThunderDrumPath, &PROPS_THUNDER_DRUM_PATH[1]),
            Some(ThunderDrum::Wind)
        );
        assert_eq!(
            route_detour_for_prop(MapKind::MoonEchoCorridor, &PROPS_MOON_ECHO_CORRIDOR[4]),
            Some(RouteDetour::MoonEchoPool)
        );
        assert_eq!(
            route_detour_for_prop(MapKind::RiverReedBed, &PROPS_RIVER_REED_BED[6]),
            Some(RouteDetour::ReedHiddenFord)
        );
        assert_eq!(
            route_detour_for_prop(MapKind::PlagueShrinePath, &PROPS_PLAGUE_SHRINE_PATH[0]),
            Some(RouteDetour::PlagueHerbTrail)
        );
        assert_eq!(
            route_detour_for_prop(
                MapKind::MansionMirrorGallery,
                &PROPS_MANSION_MIRROR_GALLERY[5]
            ),
            Some(RouteDetour::MirrorServantDoor)
        );
        assert_eq!(
            route_detour_for_prop(MapKind::ThunderDrumPath, &PROPS_THUNDER_DRUM_PATH[0]),
            Some(RouteDetour::ThunderRidgeCache)
        );
        assert_eq!(
            route_detour_for_prop(MapKind::DreamWaterway, &PROPS_DREAM_WATERWAY[0]),
            Some(RouteDetour::DreamBackwater)
        );

        let mut quest = QuestLog::default();
        assert_eq!(
            quest_marker_glyph(&quest, QuestMarkerKind::RouteMark(RouteMark::MoonEcho)),
            "!"
        );
        quest.interact_route_mark(RouteMark::MoonEcho);
        assert_eq!(
            quest_marker_glyph(&quest, QuestMarkerKind::RouteMark(RouteMark::MoonEcho)),
            "✓"
        );
        assert_eq!(
            quest_marker_glyph(
                &quest,
                QuestMarkerKind::RouteDetour(RouteDetour::MoonEchoPool)
            ),
            "？"
        );
        quest.complete_route_detour(RouteDetour::MoonEchoPool, RouteDetourApproach::Scout);
        assert_eq!(
            quest_marker_glyph(
                &quest,
                QuestMarkerKind::RouteDetour(RouteDetour::MoonEchoPool)
            ),
            "✓"
        );
    }

    #[test]
    fn puzzle_props_can_advance_matching_commissions_once() {
        let mut quest = quest_at_moon_route(false, false);
        quest.interact_side_quest(SideQuest::MoonCaveCrystals);

        let was_lit = quest.moon_crystal_marker_for(MoonCrystal::North) == "✓";
        let mut lines = quest.activate_moon_crystal(MoonCrystal::North);
        let is_lit = quest.moon_crystal_marker_for(MoonCrystal::North) == "✓";
        append_side_objective_progress(
            &mut lines,
            &mut quest,
            SideQuest::MoonCaveCrystals,
            "晶阵已净",
            was_lit,
            is_lit,
        );
        assert!(lines.iter().any(|line| line.contains("委托推进")));
        assert_eq!(quest.side_quest_progress(SideQuest::MoonCaveCrystals), 1);

        let was_lit = quest.moon_crystal_marker_for(MoonCrystal::North) == "✓";
        let mut repeat = quest.activate_moon_crystal(MoonCrystal::North);
        let is_lit = quest.moon_crystal_marker_for(MoonCrystal::North) == "✓";
        append_side_objective_progress(
            &mut repeat,
            &mut quest,
            SideQuest::MoonCaveCrystals,
            "晶阵已净",
            was_lit,
            is_lit,
        );
        assert!(!repeat.iter().any(|line| line.contains("委托推进")));
        assert_eq!(quest.side_quest_progress(SideQuest::MoonCaveCrystals), 1);

        let was_lit = quest.moon_crystal_marker_for(MoonCrystal::South) == "✓";
        let mut lines = quest.activate_moon_crystal(MoonCrystal::South);
        let is_lit = quest.moon_crystal_marker_for(MoonCrystal::South) == "✓";
        append_side_objective_progress(
            &mut lines,
            &mut quest,
            SideQuest::MoonCaveCrystals,
            "晶阵已净",
            was_lit,
            is_lit,
        );
        assert!(lines.iter().any(|line| line.contains("条件达成")));
        assert_eq!(quest.side_quest_progress(SideQuest::MoonCaveCrystals), 2);
        assert_eq!(quest.side_marker_for(SideQuest::MoonCaveCrystals), "!");
    }

    #[test]
    fn route_marks_and_detours_advance_follow_up_commissions_once() {
        let mut quest = quest_at_moon_route(false, false);
        let mut stats = PlayerStats::default();
        complete_side_quest(&mut quest, SideQuest::MoonCaveCrystals);
        quest.interact_side_quest(SideQuest::MoonCaveEchoes);

        let was_marked = quest.route_mark_marker_for(RouteMark::MoonEcho) == "✓";
        let mut lines = quest.interact_route_mark(RouteMark::MoonEcho);
        let is_marked = quest.route_mark_marker_for(RouteMark::MoonEcho) == "✓";
        let (side, source) = route_mark_side_objective(RouteMark::MoonEcho).unwrap();
        append_side_objective_progress(&mut lines, &mut quest, side, source, was_marked, is_marked);
        assert!(lines.iter().any(|line| line.contains("委托推进")));
        assert_eq!(quest.side_quest_progress(SideQuest::MoonCaveEchoes), 1);

        let was_marked = quest.route_mark_marker_for(RouteMark::MoonEcho) == "✓";
        let mut repeat = quest.interact_route_mark(RouteMark::MoonEcho);
        let is_marked = quest.route_mark_marker_for(RouteMark::MoonEcho) == "✓";
        let (side, source) = route_mark_side_objective(RouteMark::MoonEcho).unwrap();
        append_side_objective_progress(
            &mut repeat,
            &mut quest,
            side,
            source,
            was_marked,
            is_marked,
        );
        assert!(!repeat.iter().any(|line| line.contains("委托推进")));
        assert_eq!(quest.side_quest_progress(SideQuest::MoonCaveEchoes), 1);

        let result = resolve_dialogue_choice(
            DialogueChoice::route_detour(RouteDetour::MoonEchoPool),
            &mut quest,
            &mut stats,
        );
        assert!(result.lines.iter().any(|line| line.contains("委托推进")));
        assert!(
            result
                .lines
                .iter()
                .any(|line| line.contains("回声岔路已压") && line.contains("2/3"))
        );
        assert_eq!(quest.side_quest_progress(SideQuest::MoonCaveEchoes), 2);

        let repeat = resolve_dialogue_choice(
            DialogueChoice::route_detour(RouteDetour::MoonEchoPool),
            &mut quest,
            &mut stats,
        );
        assert!(!repeat.lines.iter().any(|line| line.contains("委托推进")));
        assert_eq!(quest.side_quest_progress(SideQuest::MoonCaveEchoes), 2);
    }

    #[test]
    fn river_lantern_props_form_ordered_reed_bed_puzzle() {
        assert_eq!(
            river_lantern_for_prop(MapKind::RiverReedBed, &PROPS_RIVER_REED_BED[0]),
            Some(RiverLantern::Upstream)
        );
        assert_eq!(
            river_lantern_for_prop(MapKind::RiverReedBed, &PROPS_RIVER_REED_BED[1]),
            Some(RiverLantern::Midstream)
        );
        assert_eq!(
            river_lantern_for_prop(MapKind::RiverReedBed, &PROPS_RIVER_REED_BED[2]),
            Some(RiverLantern::Dock)
        );
        assert_eq!(
            river_lantern_for_prop(MapKind::RiverTown, &PROPS_RIVER_TOWN[2]),
            None
        );

        let quest = quest_at_river_lantern_puzzle();
        assert_eq!(
            quest_marker_glyph(
                &quest,
                QuestMarkerKind::RiverLantern(RiverLantern::Upstream)
            ),
            "!"
        );
        assert_eq!(
            quest_marker_glyph(&quest, QuestMarkerKind::RiverLantern(RiverLantern::Dock)),
            "*"
        );
    }

    #[test]
    fn plague_ward_props_gate_shrine_ash_turn_in() {
        assert_eq!(
            plague_ward_for_prop(MapKind::PlagueShrinePath, &PROPS_PLAGUE_SHRINE_PATH[1]),
            Some(PlagueWard::OldShrine)
        );
        assert_eq!(
            plague_ward_for_prop(MapKind::PlagueShrinePath, &PROPS_PLAGUE_SHRINE_PATH[2]),
            Some(PlagueWard::BitterWell)
        );
        assert_eq!(
            plague_ward_for_prop(MapKind::PlagueShrinePath, &PROPS_PLAGUE_SHRINE_PATH[3]),
            Some(PlagueWard::Sickroom)
        );
        assert_eq!(
            plague_ward_for_prop(MapKind::PlagueVillage, &PROPS_PLAGUE_VILLAGE[2]),
            None
        );

        let quest = quest_at_plague_ward_puzzle();
        assert_eq!(
            quest_marker_glyph(&quest, QuestMarkerKind::PlagueWard(PlagueWard::OldShrine)),
            "!"
        );
        assert_eq!(
            quest_marker_glyph(&quest, QuestMarkerKind::PlagueWard(PlagueWard::BitterWell)),
            "!"
        );
    }

    #[test]
    fn mansion_mirror_props_unlock_capital_boss_handoff() {
        assert_eq!(
            mansion_mirror_for_prop(
                MapKind::MansionMirrorGallery,
                &PROPS_MANSION_MIRROR_GALLERY[1]
            ),
            Some(MansionMirrorNode::Ledger)
        );
        assert_eq!(
            mansion_mirror_for_prop(
                MapKind::MansionMirrorGallery,
                &PROPS_MANSION_MIRROR_GALLERY[2]
            ),
            Some(MansionMirrorNode::Witness)
        );
        assert_eq!(
            mansion_mirror_for_prop(MapKind::CapitalMansion, &PROPS_CAPITAL_MANSION[1]),
            None
        );

        let quest = quest_at_mansion_mirror_puzzle();
        assert_eq!(
            quest_marker_glyph(
                &quest,
                QuestMarkerKind::MansionMirror(MansionMirrorNode::Ledger)
            ),
            "!"
        );
        assert_eq!(
            quest_marker_glyph(
                &quest,
                QuestMarkerKind::MansionMirror(MansionMirrorNode::Witness)
            ),
            "!"
        );
    }

    #[test]
    fn thunder_drum_props_gate_southern_boss_handoff() {
        assert_eq!(
            thunder_drum_for_prop(MapKind::ThunderDrumPath, &PROPS_THUNDER_DRUM_PATH[1]),
            Some(ThunderDrum::Wind)
        );
        assert_eq!(
            thunder_drum_for_prop(MapKind::ThunderDrumPath, &PROPS_THUNDER_DRUM_PATH[2]),
            Some(ThunderDrum::Cloud)
        );
        assert_eq!(
            thunder_drum_for_prop(MapKind::ThunderDrumPath, &PROPS_THUNDER_DRUM_PATH[3]),
            Some(ThunderDrum::Oath)
        );
        assert_eq!(
            thunder_drum_for_prop(MapKind::SouthernRoad, &PROPS_SOUTHERN_ROAD[2]),
            None
        );

        let quest = quest_at_thunder_drum_puzzle();
        assert_eq!(
            quest_marker_glyph(&quest, QuestMarkerKind::ThunderDrum(ThunderDrum::Wind)),
            "!"
        );
        assert_eq!(
            quest_marker_glyph(&quest, QuestMarkerKind::ThunderDrum(ThunderDrum::Cloud)),
            "!"
        );
    }

    #[test]
    fn shrine_props_map_to_battle_blessings() {
        assert_eq!(
            shrine_blessing_for_prop(MapKind::Village, &PROPS_VILLAGE[2]),
            Some(ShrineBlessing::Guard)
        );
        assert_eq!(
            shrine_blessing_for_prop(MapKind::Bamboo, &PROPS_BAMBOO[3]),
            Some(ShrineBlessing::Sword)
        );
        assert_eq!(
            shrine_blessing_for_prop(MapKind::RiverTown, &PROPS_RIVER_TOWN[4]),
            Some(ShrineBlessing::Spirit)
        );
        assert_eq!(
            shrine_blessing_for_prop(MapKind::PlagueShrinePath, &PROPS_PLAGUE_SHRINE_PATH[1]),
            Some(ShrineBlessing::Guard)
        );
        assert_eq!(
            shrine_blessing_for_prop(MapKind::CapitalMansion, &PROPS_CAPITAL_MANSION[2]),
            Some(ShrineBlessing::Sword)
        );
        assert_eq!(
            shrine_blessing_for_prop(MapKind::Village, &PROPS_VILLAGE[0]),
            None
        );
    }

    #[test]
    fn treasure_reward_updates_stats_once() {
        let mut quest = QuestLog::default();
        let mut stats = PlayerStats::default();

        let interaction = quest.interact_treasure(TreasureCache::VillageShrine);
        let reward = interaction
            .reward
            .expect("first treasure interaction rewards");
        let line = apply_treasure_reward(&mut stats, reward);
        assert_eq!(stats.exp, 10);
        assert_eq!(stats.potions, 4);
        assert_eq!(stats.gold, 88);
        assert!(line.contains("经验 +10"));

        let interaction = quest.interact_treasure(TreasureCache::VillageShrine);
        assert!(interaction.reward.is_none());
    }

    #[test]
    fn field_supply_prompt_marker_and_reward_update_stats_once() {
        let mut quest = QuestLog::default();
        let mut stats = PlayerStats {
            hp: 50,
            mp: 10,
            ..default()
        };
        let supply = field_supply_defs(MapKind::Village).next().unwrap();
        let pos = PlayerPos {
            col: supply.col,
            row: supply.row + 1,
            facing: IVec2::new(0, -1),
        };

        let prompt = field_supply_facing_prompt(MapKind::Village, &pos, &quest).unwrap();
        assert!(prompt.contains("村郊止血草"));
        assert!(prompt.contains("可采"));
        assert_eq!(
            quest_marker_glyph(&quest, QuestMarkerKind::FieldSupply(supply.supply)),
            "!"
        );

        let interaction = quest.interact_field_supply(supply.supply);
        let reward = interaction.reward.expect("first field supply rewards");
        let line = apply_supply_reward(&mut stats, reward);
        assert_eq!(stats.exp, 4);
        assert_eq!(stats.potions, 4);
        assert_eq!(stats.hp, 64);
        assert_eq!(stats.mp, 10);
        assert!(line.contains("采集奖励"));

        assert_eq!(
            quest_marker_glyph(&quest, QuestMarkerKind::FieldSupply(supply.supply)),
            "✓"
        );
        let prompt = field_supply_facing_prompt(MapKind::Village, &pos, &quest).unwrap();
        assert!(prompt.contains("已采"));

        let repeat = quest.interact_field_supply(supply.supply);
        assert!(repeat.reward.is_none());
    }

    #[test]
    fn commission_trace_prompt_marker_and_progress_update_once() {
        let mut quest = QuestLog::default();
        let trace = COMMISSION_TRACE_DEFS
            .iter()
            .find(|trace| trace.side == SideQuest::VillageTrail)
            .unwrap();
        let pos = PlayerPos {
            col: trace.col,
            row: trace.row - 1,
            facing: IVec2::new(0, 1),
        };

        let prompt = commission_trace_facing_prompt(MapKind::Village, &pos, &quest).unwrap();
        assert!(prompt.contains("旧竹栅妖痕"));
        assert!(prompt.contains("未领取"));
        assert_eq!(
            quest_marker_glyph(&quest, QuestMarkerKind::CommissionTrace(trace.side)),
            "？"
        );

        quest.interact_side_quest(SideQuest::VillageTrail);
        let prompt = commission_trace_facing_prompt(MapKind::Village, &pos, &quest).unwrap();
        assert!(prompt.contains("可处理"));
        assert_eq!(
            quest_marker_glyph(&quest, QuestMarkerKind::CommissionTrace(trace.side)),
            "!"
        );
        let choice = DialogueChoice::commission_trace(trace);
        assert_eq!(choice.prompt(), "要怎样处理《旧竹栅妖痕》这处委托现场？");
        assert_eq!(choice.option_count(), 3);
        assert_eq!(choice.option_label(0), "细查现场");
        assert_eq!(choice.option_label(1), "快断余妖");

        let lines = quest.interact_commission_trace(
            trace.side,
            trace.name,
            trace.active_line,
            trace.source,
            trace.inactive_line,
            trace.repeat_line,
        );
        assert!(lines.iter().any(|line| line.contains("旧竹栅妖痕已查")));
        assert_eq!(quest.side_quest_progress(SideQuest::VillageTrail), 1);
        assert_eq!(
            quest_marker_glyph(&quest, QuestMarkerKind::CommissionTrace(trace.side)),
            "✓"
        );

        let prompt = commission_trace_facing_prompt(MapKind::Village, &pos, &quest).unwrap();
        assert!(prompt.contains("已处理"));
        let repeat = quest.interact_commission_trace(
            trace.side,
            trace.name,
            trace.active_line,
            trace.source,
            trace.inactive_line,
            trace.repeat_line,
        );
        assert!(repeat.iter().any(|line| line.contains("已经写进委托签")));
        assert_eq!(quest.side_quest_progress(SideQuest::VillageTrail), 1);

        let mut fast_quest = QuestLog::default();
        let mut stats = PlayerStats::default();
        fast_quest.interact_side_quest(SideQuest::VillageTrail);
        let mut fast_choice = DialogueChoice::commission_trace(trace);
        fast_choice.selected = 1;
        let result = resolve_dialogue_choice(fast_choice, &mut fast_quest, &mut stats);
        assert!(result.lines.iter().any(|line| line.contains("快断余妖")));
        assert_eq!(
            fast_quest.side_quest_field_approach(SideQuest::VillageTrail),
            Some(SideQuestFieldApproach::Confront)
        );
        assert_eq!(fast_quest.side_quest_progress(SideQuest::VillageTrail), 1);
    }

    #[test]
    fn care_aftermath_reward_updates_stats_once() {
        let mut quest = QuestLog::default();
        let mut stats = PlayerStats::default();

        quest.talk(QuestRole::SwordSister);
        quest.talk(QuestRole::Linger);
        quest.interact_bond_scene();
        quest.interact_camp_scene();

        let reward = quest
            .claim_care_aftermath(Chapter::VillageOath)
            .expect("completed care should reward once");
        let line = apply_care_aftermath_reward(&mut stats, reward);
        assert!(line.contains("照应回礼"));
        assert_eq!(stats.exp, 16);
        assert_eq!(stats.potions, 4);
        assert_eq!(stats.gold, 90);
        assert_eq!(quest.claim_care_aftermath(Chapter::VillageOath), None);
    }

    #[test]
    fn commission_aftermath_reward_updates_stats_once() {
        let mut quest = QuestLog::default();
        let mut stats = PlayerStats::default();

        assert_eq!(quest.claim_commission_aftermath(Chapter::MoonCave), None);
        complete_side_quest(&mut quest, SideQuest::MoonCaveCrystals);
        assert_eq!(quest.claim_commission_aftermath(Chapter::MoonCave), None);
        complete_side_quest(&mut quest, SideQuest::MoonCaveEchoes);

        let reward = quest
            .claim_commission_aftermath(Chapter::MoonCave)
            .expect("completed local chain should reward once");
        let line = local_commission_aftermath_line(MapKind::MoonEchoCorridor);
        assert!(line.contains("地方回礼"));
        assert!(line.contains("晶尘"));
        let reward_line = apply_commission_aftermath_reward(&mut stats, reward);
        assert!(reward_line.contains("清账回礼"));
        assert!(reward_line.contains("境界提升至 Lv.2"));
        assert_eq!(stats.level, 2);
        assert_eq!(stats.exp, 8);
        assert_eq!(stats.potions, 4);
        assert_eq!(stats.gold, 100);
        assert_eq!(quest.claim_commission_aftermath(Chapter::MoonCave), None);
    }

    #[test]
    fn companion_scene_followup_rewards_when_returning_to_local_npc() {
        let mut quest = quest_at_moon_route_with_trail_companion_scene();
        let mut stats = PlayerStats::default();
        let exp_before = stats.exp;
        let potions_before = stats.potions;
        let gold_before = stats.gold;

        let lines = claim_local_companion_scene_followup(MapKind::Bamboo, &mut quest, &mut stats);
        assert!(lines.iter().any(|line| line.contains("小传回访")));
        assert!(lines.iter().any(|line| line.contains("月衡")));
        assert!(lines.iter().any(|line| line.contains("旧栅")));
        assert!(lines.iter().any(|line| line.contains("小传回礼")));
        assert_eq!(stats.exp, exp_before + 16);
        assert_eq!(stats.potions, potions_before);
        assert_eq!(stats.gold, gold_before + 10);
        assert!(quest.has_claimed_companion_aftermath(CompanionScene::SwordSisterTrailGuard));
        assert!(
            claim_local_companion_scene_followup(MapKind::Bamboo, &mut quest, &mut stats)
                .is_empty()
        );

        let mut missed = quest_at_river_lantern_puzzle();
        assert!(
            claim_local_companion_scene_followup(MapKind::Bamboo, &mut missed, &mut stats)
                .is_empty()
        );
    }

    #[test]
    fn missed_companion_revisit_runs_from_npc_pickup_to_field_and_turn_in() {
        let revisit = CompanionRevisit::TrailEcho;
        let giver = NPCS_PLAGUE_VILLAGE
            .iter()
            .find(|npc| npc.col == 5 && npc.row == 12)
            .unwrap();
        let mut quest = quest_at_plague_ward_puzzle();
        let mut stats = PlayerStats::default();

        assert_eq!(
            npc_companion_revisit_for(MapKind::PlagueVillage, giver),
            Some(revisit)
        );
        let giver_pos = PlayerPos {
            col: giver.col,
            row: giver.row + 1,
            facing: IVec2::new(0, -1),
        };
        let prompt =
            npc_companion_revisit_facing_prompt(MapKind::PlagueVillage, &giver_pos, &quest)
                .unwrap();
        assert!(prompt.contains("旧栅余声 [可领取]"));
        assert!(prompt.contains("小传补访-01"));
        assert!(task_tracker_text(MapKind::PlagueVillage, &quest).contains("本地补访"));

        let handoff = npc_companion_revisit_handoff(MapKind::PlagueVillage, giver, &quest).unwrap();
        assert!(handoff.lines.iter().any(|line| line.contains("迟来寻访")));
        let accept_choice = handoff
            .choice
            .expect("available revisit should offer pickup");
        assert_eq!(
            accept_choice.kind,
            DialogueChoiceKind::CompanionRevisit {
                revisit,
                action: CompanionRevisitChoiceAction::Accept,
            }
        );
        let result = resolve_dialogue_choice(accept_choice, &mut quest, &mut stats);
        assert!(result.lines.iter().any(|line| line.contains("补访签已接")));
        let notice = quest_notice_for_confirmed_choice(accept_choice, &quest).unwrap();
        assert!(notice.title.contains("同伴补访已领取"));
        assert!(notice.body.contains("瘴雨祠道"));
        let tracker = task_tracker_text(MapKind::PlagueShrinePath, &quest);
        assert!(tracker.contains("同伴补访 · 旧栅余声 [寻访中]"));
        assert!(tracker.contains("当前地图寻访"));
        assert!(tracker.contains("祠道旧铃路印"));

        let target = prop_defs(MapKind::PlagueShrinePath)
            .iter()
            .find(|prop| {
                route_mark_for_prop(MapKind::PlagueShrinePath, prop) == Some(RouteMark::PlagueBell)
            })
            .unwrap();
        let field_pos = PlayerPos {
            col: target.col,
            row: target.row + 1,
            facing: IVec2::new(0, -1),
        };
        let field_prompt =
            companion_revisit_field_facing_prompt(MapKind::PlagueShrinePath, &field_pos, &quest)
                .unwrap();
        assert!(field_prompt.contains("小传补访现场"));
        assert!(field_prompt.contains("接回旧话"));

        quest.interact_route_mark(RouteMark::PlagueBell);
        let rate_before = route_encounter_rate(0.20, MapKind::PlagueShrinePath, &quest);
        let field = quest.resolve_companion_revisit(revisit);
        let reward_line = apply_companion_scene_reward(
            &mut stats,
            field
                .reward
                .expect("field revisit should restore scene reward"),
        );
        assert!(field.lines.iter().any(|line| line.contains("迟来小传")));
        assert!(
            field
                .lines
                .iter()
                .any(|line| line.contains("一起把人带出去"))
        );
        assert!(reward_line.contains("小传奖励"));
        let rate_after = route_encounter_rate(0.20, MapKind::PlagueShrinePath, &quest);
        assert!((rate_after - rate_before * 0.96).abs() < 0.001);
        assert!(
            route_pressure_summary(MapKind::PlagueShrinePath, &quest)
                .is_some_and(|line| line.contains("小传补访 已补 遇妖-4%"))
        );
        assert!(
            companion_revisit_checkpoint_line(MapKind::PlagueShrinePath, &quest)
                .is_some_and(|line| line.contains("补访照应"))
        );
        assert!(
            claim_local_companion_scene_followup(MapKind::Bamboo, &mut quest, &mut stats)
                .is_empty()
        );
        assert!(quest.companion_revisit_ready(revisit));

        let handoff = npc_companion_revisit_handoff(MapKind::PlagueVillage, giver, &quest).unwrap();
        let turn_in_choice = handoff
            .choice
            .expect("resolved revisit should offer turn in");
        assert_eq!(
            turn_in_choice.kind,
            DialogueChoiceKind::CompanionRevisit {
                revisit,
                action: CompanionRevisitChoiceAction::TurnIn,
            }
        );
        let gold_before = stats.gold;
        let result = resolve_dialogue_choice(turn_in_choice, &mut quest, &mut stats);
        assert!(result.lines.iter().any(|line| line.contains("补访归档")));
        assert!(result.lines.iter().any(|line| line.contains("小传回礼")));
        assert_eq!(stats.gold, gold_before + 10);
        assert!(quest.companion_revisit_completed(revisit));
        assert!(npc_companion_revisit_handoff(MapKind::PlagueVillage, giver, &quest).is_none());
        assert!(
            npc_reaction_lines(MapKind::PlagueVillage, &NPCS_PLAGUE_VILLAGE[2], &quest)
                .iter()
                .any(|line| line.contains("补访回声"))
        );
    }

    #[test]
    fn companion_revisit_givers_do_not_replace_existing_npc_roles_or_services() {
        let cases = [
            (MapKind::PlagueVillage, 5, 12, CompanionRevisit::TrailEcho),
            (MapKind::SouthernRoad, 25, 14, CompanionRevisit::MirrorTrace),
            (MapKind::FinalSanctum, 24, 1, CompanionRevisit::TotemVow),
        ];

        for (kind, col, row, revisit) in cases {
            let npc = npc_defs(kind)
                .iter()
                .find(|npc| npc.col == col && npc.row == row)
                .unwrap();
            assert_eq!(npc_companion_revisit_for(kind, npc), Some(revisit));
            assert!(npc.quest.is_none());
            assert!(!is_side_quest_contact(kind, npc));
            assert!(npc_service_for(kind, npc).is_none());
            assert!(npc_errand_offer_for(kind, npc).is_none());
            assert!(npc_errand_delivery_for(kind, npc).is_none());
        }
    }

    #[test]
    fn shrine_offering_spends_gold_and_blocks_duplicate_blessings() {
        let mut quest = QuestLog::default();
        let mut stats = PlayerStats::default();

        let lines = apply_shrine_offering(&mut stats, &mut quest, ShrineBlessing::Guard);
        assert_eq!(stats.gold, 80 - SHRINE_OFFERING_PRICE);
        assert_eq!(quest.active_shrine_blessing(), Some(ShrineBlessing::Guard));
        assert!(lines[0].contains("供奉"));
        assert!(lines[1].contains("下一场战斗"));
        assert!(lines[2].contains("行路护持"));
        assert!(lines[2].contains("不容易"));

        let lines = apply_shrine_offering(&mut stats, &mut quest, ShrineBlessing::Sword);
        assert_eq!(stats.gold, 80 - SHRINE_OFFERING_PRICE);
        assert_eq!(quest.active_shrine_blessing(), Some(ShrineBlessing::Guard));
        assert!(lines[0].contains("仍在"));
        assert!(lines[2].contains("行路护持"));

        quest.take_shrine_blessing();
        stats.gold = 0;
        let lines = apply_shrine_offering(&mut stats, &mut quest, ShrineBlessing::Spirit);
        assert!(lines[0].contains("钱不够"));
        assert_eq!(quest.active_shrine_blessing(), None);
    }

    #[test]
    fn shrine_blessings_adjust_route_encounter_rate() {
        let base = 0.20;
        let mut quest = QuestLog::default();
        assert_eq!(route_encounter_rate(base, MapKind::Village, &quest), base);

        assert!(quest.set_shrine_blessing(ShrineBlessing::Guard));
        assert!((route_encounter_rate(base, MapKind::Village, &quest) - 0.13).abs() < 0.001);

        quest.take_shrine_blessing();
        assert!(quest.set_shrine_blessing(ShrineBlessing::Sword));
        assert!((route_encounter_rate(base, MapKind::Village, &quest) - 0.24).abs() < 0.001);

        quest.take_shrine_blessing();
        assert!(quest.set_shrine_blessing(ShrineBlessing::Spirit));
        assert!((route_encounter_rate(base, MapKind::Village, &quest) - 0.17).abs() < 0.001);
        assert!((route_encounter_rate(0.95, MapKind::Village, &quest) - 0.8075).abs() < 0.001);
    }

    #[test]
    fn route_marks_adjust_route_encounter_rate() {
        let base = 0.20;
        let mut quest = QuestLog::default();
        assert_eq!(
            route_encounter_rate(base, MapKind::MoonEchoCorridor, &quest),
            base
        );

        quest.interact_route_mark(RouteMark::MoonEcho);
        assert!(
            (route_encounter_rate(base, MapKind::MoonEchoCorridor, &quest) - 0.18).abs() < 0.001
        );
        assert_eq!(route_encounter_rate(base, MapKind::Village, &quest), base);

        assert!(quest.set_shrine_blessing(ShrineBlessing::Guard));
        assert!(
            (route_encounter_rate(base, MapKind::MoonEchoCorridor, &quest) - 0.117).abs() < 0.001
        );
    }

    #[test]
    fn route_detours_adjust_route_encounter_rate_by_choice() {
        let base = 0.20;
        let mut careful = QuestLog::default();
        assert_eq!(
            route_encounter_rate(base, MapKind::MoonEchoCorridor, &careful),
            base
        );

        let resolution =
            careful.complete_route_detour(RouteDetour::MoonEchoPool, RouteDetourApproach::Scout);
        assert!(resolution.reward.is_some());
        assert!(
            (route_encounter_rate(base, MapKind::MoonEchoCorridor, &careful) - 0.184).abs() < 0.001
        );
        assert_eq!(route_encounter_rate(base, MapKind::Village, &careful), base);

        let mut swift = QuestLog::default();
        swift.complete_route_detour(RouteDetour::MoonEchoPool, RouteDetourApproach::PressOn);
        assert!(
            (route_encounter_rate(base, MapKind::MoonEchoCorridor, &swift) - 0.194).abs() < 0.001
        );
    }

    #[test]
    fn route_care_changes_route_pressure_after_party_scenes() {
        let base = 0.20;
        let early = QuestLog::default();
        assert_eq!(
            route_care_state(MapKind::MoonEchoCorridor, &early),
            RouteCareState::NoRoute
        );
        assert_eq!(route_care_summary(MapKind::MoonEchoCorridor, &early), None);

        let unprepared = quest_at_moon_route(false, false);
        assert_eq!(
            route_care_state(MapKind::MoonEchoCorridor, &unprepared),
            RouteCareState::Unprepared
        );
        assert_eq!(
            route_care_summary(MapKind::MoonEchoCorridor, &unprepared),
            Some("路线照应 欠备 遇妖+6%")
        );
        assert!(
            (route_encounter_rate(base, MapKind::MoonEchoCorridor, &unprepared) - 0.212).abs()
                < 0.001
        );
        assert!(
            route_care_checkpoint_line(MapKind::MoonEchoCorridor, &unprepared)
                .is_some_and(|line| line.contains("更容易"))
        );

        let partial = quest_at_moon_route(true, false);
        assert_eq!(
            route_care_state(MapKind::MoonEchoCorridor, &partial),
            RouteCareState::Partial
        );
        assert!(
            (route_encounter_rate(base, MapKind::MoonEchoCorridor, &partial) - 0.188).abs() < 0.001
        );

        let prepared = quest_at_moon_route(true, true);
        assert_eq!(
            route_care_state(MapKind::MoonEchoCorridor, &prepared),
            RouteCareState::Prepared
        );
        assert!(
            (route_encounter_rate(base, MapKind::MoonEchoCorridor, &prepared) - 0.172).abs()
                < 0.001
        );
        assert!(
            route_care_checkpoint_line(MapKind::MoonEchoCorridor, &prepared)
                .is_some_and(|line| line.contains("压力降低"))
        );

        let missed = quest_at_river_lantern_puzzle();
        assert_eq!(
            route_care_state(MapKind::MoonEchoCorridor, &missed),
            RouteCareState::Missed
        );
        assert!(
            (route_encounter_rate(base, MapKind::MoonEchoCorridor, &missed) - 0.233).abs() < 0.001
        );
        assert_eq!(
            route_encounter_rate(base, MapKind::Village, &prepared),
            base
        );
    }

    #[test]
    fn companion_scenes_reduce_matching_route_pressure_and_checkpoint_lines() {
        let base = 0.20;
        let plain = quest_at_moon_route(false, false);
        assert_eq!(
            companion_route_state(MapKind::MoonEchoCorridor, &plain),
            CompanionRouteState::NoRoute
        );
        assert_eq!(
            companion_route_summary(MapKind::MoonEchoCorridor, &plain),
            None
        );
        assert!(
            (route_encounter_rate(base, MapKind::MoonEchoCorridor, &plain) - 0.212).abs() < 0.001
        );

        let prepared = quest_at_moon_route_with_trail_companion_scene();
        assert_eq!(
            companion_route_state(MapKind::MoonEchoCorridor, &prepared),
            CompanionRouteState::Prepared
        );
        assert_eq!(
            companion_route_summary(MapKind::MoonEchoCorridor, &prepared),
            Some("小传照应 周全 遇妖-8%")
        );
        assert!(
            route_pressure_summary(MapKind::MoonEchoCorridor, &prepared).is_some_and(|line| line
                .contains("路线照应 欠备")
                && line.contains("小传照应 周全"))
        );
        assert!(
            (route_encounter_rate(base, MapKind::MoonEchoCorridor, &prepared) - 0.195).abs()
                < 0.001
        );
        assert!(
            companion_route_checkpoint_line(MapKind::MoonEchoCorridor, &prepared)
                .is_some_and(|line| line.contains("林月衡") && line.contains("退路"))
        );

        let missed = quest_at_river_lantern_puzzle();
        assert_eq!(
            companion_route_state(MapKind::MoonEchoCorridor, &missed),
            CompanionRouteState::Missed
        );
        assert_eq!(
            companion_route_summary(MapKind::MoonEchoCorridor, &missed),
            Some("小传照应 错过 遇妖+4%")
        );
        assert!(
            route_pressure_summary(MapKind::MoonEchoCorridor, &missed).is_some_and(|line| line
                .contains("路线照应 错过")
                && line.contains("小传照应 错过"))
        );
        assert!(
            companion_route_checkpoint_line(MapKind::MoonEchoCorridor, &missed)
                .is_some_and(|line| line.contains("小传错过"))
        );
    }

    #[test]
    fn npc_reactions_reflect_local_side_quest_status() {
        let npc = &NPCS_VILLAGE[0];
        let mut quest = QuestLog::default();
        assert!(npc_reaction_lines(MapKind::Village, npc, &quest).is_empty());

        quest.interact_side_quest(SideQuest::VillageTrail);
        let lines = npc_reaction_lines(MapKind::Village, npc, &quest);
        assert_eq!(lines.len(), 1);
        assert!(lines[0].contains("山路余妖"));
        assert!(lines[0].contains("0/2"));

        quest.record_side_victory();
        quest.record_side_victory();
        let lines = npc_reaction_lines(MapKind::Village, npc, &quest);
        assert!(lines[0].contains("交付"));

        quest.interact_side_quest(SideQuest::VillageTrail);
        let lines = npc_reaction_lines(MapKind::Village, npc, &quest);
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("山路余妖已清"));
        assert!(lines[1].contains("新签"));
        assert!(lines[1].contains("药圃护路"));

        quest.interact_side_quest(SideQuest::VillageHerbs);
        let lines = npc_reaction_lines(MapKind::Village, npc, &quest);
        assert!(lines[0].contains("山路余妖已清"));
        assert!(lines[1].contains("药圃护路"));
        assert!(lines[1].contains("0/2"));

        quest.record_side_victory();
        quest.record_side_victory();
        quest.interact_side_quest(SideQuest::VillageHerbs);
        let lines = npc_reaction_lines(MapKind::Village, npc, &quest);
        assert_eq!(lines.len(), 1);
        assert!(lines[0].contains("两张红签"));

        let final_npc = &NPCS_FINAL_SANCTUM[5];
        let mut finale = QuestLog::default();
        finale.interact_side_quest(SideQuest::FinalDreamEchoes);
        for _ in 0..finale.side_quest_goal(SideQuest::FinalDreamEchoes) {
            finale.record_side_victory();
        }
        finale.interact_side_quest(SideQuest::FinalDreamEchoes);
        finale.interact_side_quest(SideQuest::FinalHomewardVows);
        for _ in 0..finale.side_quest_goal(SideQuest::FinalHomewardVows) {
            finale.record_side_victory();
        }
        finale.interact_side_quest(SideQuest::FinalHomewardVows);
        let lines = npc_reaction_lines(MapKind::FinalSanctum, final_npc, &finale);
        assert_eq!(lines.len(), 1);
        assert!(lines[0].contains("梦灯余波"));
        assert!(lines[0].contains("归潮灯签"));
    }

    #[test]
    fn npc_reactions_reflect_opened_treasure_caches() {
        let npc = &NPCS_VILLAGE[0];
        let mut quest = QuestLog::default();
        assert!(npc_reaction_lines(MapKind::Village, npc, &quest).is_empty());

        quest.interact_treasure(TreasureCache::VillageShrine);
        let lines = npc_reaction_lines(MapKind::Village, npc, &quest);
        assert_eq!(lines.len(), 1);
        assert!(lines[0].contains("旧像"));
        assert!(lines[0].contains("护身药"));

        let main_role_npc = &NPCS_VILLAGE[1];
        assert!(npc_reaction_lines(MapKind::Village, main_role_npc, &quest).is_empty());
    }

    #[test]
    fn npc_reactions_reflect_local_care_scenes() {
        let npc = &NPCS_VILLAGE[0];
        let mut quest = QuestLog::default();
        assert!(npc_reaction_lines(MapKind::Village, npc, &quest).is_empty());

        quest.talk(QuestRole::SwordSister);
        quest.talk(QuestRole::Linger);
        quest.interact_bond_scene();
        quest.interact_camp_scene();
        let lines = npc_reaction_lines(MapKind::Village, npc, &quest);
        assert_eq!(lines.len(), 1);
        assert!(lines[0].contains("照应"));
        assert!(lines[0].contains("灵灯旁"));
        assert!(lines[0].contains("歇脚处"));

        let mut missed = QuestLog::default();
        missed.talk(QuestRole::SwordSister);
        missed.talk(QuestRole::Linger);
        missed.talk(QuestRole::StarMage);
        missed.record_victory();
        missed.record_victory();
        missed.talk(QuestRole::SwordSister);
        missed.talk(QuestRole::Merchant);
        missed.talk(QuestRole::BambooScout);
        let lines = npc_reaction_lines(MapKind::Village, npc, &missed);
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("错过的夜谈和休整"));
        assert!(lines[1].contains("小传街谈"));
        assert!(lines[1].contains("未问出口的小传"));
    }

    #[test]
    fn npc_reactions_reflect_route_branch_choices() {
        let npc = &NPCS_RIVER_TOWN[2];
        let mut careful = QuestLog::default();
        careful.complete_route_detour(RouteDetour::ReedHiddenFord, RouteDetourApproach::Scout);
        let lines = npc_reaction_lines(MapKind::RiverTown, npc, &careful);
        assert_eq!(lines.len(), 1);
        assert!(lines[0].contains("芦下隐渡"));
        assert!(lines[0].contains("芦痕"));
        assert!(lines[0].contains("少绕"));

        let route_lines =
            npc_reaction_lines(MapKind::RiverReedBed, &NPCS_RIVER_REED_BED[0], &careful);
        assert_eq!(route_lines.len(), 1);
        assert!(route_lines[0].contains("芦下隐渡"));

        let mut swift = QuestLog::default();
        swift.complete_route_detour(RouteDetour::ReedHiddenFord, RouteDetourApproach::PressOn);
        let lines = npc_reaction_lines(MapKind::RiverTown, npc, &swift);
        assert_eq!(lines.len(), 1);
        assert!(lines[0].contains("快走"));
        assert!(lines[0].contains("不敢夜里走"));

        let main_role_npc = &NPCS_RIVER_TOWN[0];
        assert!(npc_reaction_lines(MapKind::RiverTown, main_role_npc, &swift).is_empty());
        assert!(npc_reaction_lines(MapKind::Village, &NPCS_VILLAGE[0], &swift).is_empty());
    }

    #[test]
    fn route_detour_reports_reward_when_returning_to_local_npc() {
        let mut quest = QuestLog::default();
        let mut stats = PlayerStats::default();

        assert!(
            claim_local_route_detour_report(MapKind::RiverTown, &mut quest, &mut stats).is_empty()
        );

        quest.complete_route_detour(RouteDetour::ReedHiddenFord, RouteDetourApproach::Scout);
        let gold_before = stats.gold;
        let lines = claim_local_route_detour_report(MapKind::RiverTown, &mut quest, &mut stats);

        assert!(lines.iter().any(|line| line.contains("【报路】芦下隐渡")));
        assert!(lines.iter().any(|line| line.contains("江岸巡货人")));
        assert!(lines.iter().any(|line| line.contains("【报路回礼】")));
        assert_eq!(quest.route_report_summary(), "路报 1/6 芦下隐渡");
        assert_eq!(stats.gold, gold_before + 14);
        assert!(
            claim_local_route_detour_report(MapKind::RiverTown, &mut quest, &mut stats).is_empty()
        );
    }

    #[test]
    fn npc_reactions_reflect_companion_personal_scene_status() {
        let npc = &NPCS_BAMBOO[0];
        let prepared = quest_at_moon_route_with_trail_companion_scene();
        let lines = npc_reaction_lines(MapKind::Bamboo, npc, &prepared);
        assert!(lines.iter().any(|line| line.contains("小传街谈")));
        assert!(lines.iter().any(|line| line.contains("林月衡")));
        assert!(lines.iter().any(|line| line.contains("旧栅")));

        let missed = quest_at_river_lantern_puzzle();
        let lines = npc_reaction_lines(MapKind::Bamboo, npc, &missed);
        assert!(lines.iter().any(|line| line.contains("小传街谈")));
        assert!(lines.iter().any(|line| line.contains("未问出口的小传")));
        assert!(lines.iter().any(|line| line.contains("后队")));

        let main_role_npc = &NPCS_BAMBOO[2];
        assert!(npc_reaction_lines(MapKind::Bamboo, main_role_npc, &missed).is_empty());
    }

    #[test]
    fn facing_task_board_shows_claim_or_turn_in_action() {
        let pos = PlayerPos {
            col: 21,
            row: 2,
            facing: IVec2::new(0, -1),
        };
        let mut quest = QuestLog::default();
        let side = side_quest_facing_player(MapKind::Village, &pos, &quest);
        assert_eq!(side, Some(SideQuest::VillageTrail));

        let prompt = side_board_facing_prompt(MapKind::Village, &pos, &quest).unwrap();
        assert!(prompt.contains("委托板 0/2完成"));
        assert!(prompt.contains("空格打开"));
        assert!(prompt.contains("山路余妖[可领取]"));
        assert!(prompt.contains("药圃护路[后续]"));

        quest.interact_side_quest(SideQuest::VillageTrail);
        assert!(
            quest
                .side_task_prompt(SideQuest::VillageTrail)
                .contains("空格查看")
        );

        quest.record_side_victory();
        quest.record_side_victory();
        assert!(
            quest
                .side_task_prompt(SideQuest::VillageTrail)
                .contains("空格交付")
        );
        quest.interact_side_quest(SideQuest::VillageTrail);
        assert_eq!(
            side_quest_facing_player(MapKind::Village, &pos, &quest),
            Some(SideQuest::VillageHerbs)
        );
        let prompt = side_board_facing_prompt(MapKind::Village, &pos, &quest).unwrap();
        assert!(prompt.contains("委托板 1/2完成"));
        assert!(prompt.contains("药圃护路[可领取]"));
        assert!(prompt.contains("山路余妖[已完成]"));

        let away = PlayerPos {
            col: 21,
            row: 2,
            facing: IVec2::new(0, 1),
        };
        assert_eq!(
            side_quest_facing_player(MapKind::Village, &away, &quest),
            None
        );

        let final_pos = PlayerPos {
            col: 2,
            row: 4,
            facing: IVec2::new(1, 0),
        };
        let final_prompt =
            side_board_facing_prompt(MapKind::FinalSanctum, &final_pos, &QuestLog::default())
                .unwrap();
        assert!(final_prompt.contains("委托板 0/2完成"));
        assert!(final_prompt.contains("梦灯余波[可领取]"));
        assert!(final_prompt.contains("归潮旧愿[后续]"));
    }

    #[test]
    fn npc_contacts_can_offer_and_turn_in_local_commissions() {
        let pos = PlayerPos {
            col: 6,
            row: 8,
            facing: IVec2::new(0, -1),
        };
        let npc = &NPCS_VILLAGE[0];
        let mut quest = QuestLog::default();
        let mut stats = PlayerStats::default();

        let prompt = npc_side_quest_facing_prompt(MapKind::Village, &pos, &quest).unwrap();
        assert!(prompt.contains("委托联系人 婆婆"));
        assert!(prompt.contains("山路余妖[可领取]"));
        assert!(prompt.contains("空格领取"));
        assert_eq!(
            quest_marker_glyph(&quest, QuestMarkerKind::SideContact(MapKind::Village)),
            "!"
        );

        let handoff = npc_side_quest_handoff(MapKind::Village, npc, &quest).unwrap();
        assert!(handoff.lines.iter().any(|line| line.contains("【领委托】")));
        assert!(
            handoff
                .lines
                .iter()
                .any(|line| line.contains("【追踪预览】"))
        );
        let choice = handoff
            .choice
            .expect("available contact should ask to accept");
        assert_eq!(
            choice.kind,
            DialogueChoiceKind::SideQuest {
                side: SideQuest::VillageTrail,
                action: SideQuestChoiceAction::Accept
            }
        );
        resolve_dialogue_choice(choice, &mut quest, &mut stats);
        assert!(quest.is_side_quest_active(SideQuest::VillageTrail));
        assert_eq!(
            quest_marker_glyph(&quest, QuestMarkerKind::SideContact(MapKind::Village)),
            "*"
        );

        let handoff = npc_side_quest_handoff(MapKind::Village, npc, &quest).unwrap();
        assert!(handoff.choice.is_none());
        assert!(handoff.lines.iter().any(|line| line.contains("进度 0/2")));

        quest.record_side_victory();
        quest.record_side_victory();
        let prompt = npc_side_quest_facing_prompt(MapKind::Village, &pos, &quest).unwrap();
        assert!(prompt.contains("山路余妖[可交付]"));
        assert!(prompt.contains("空格交付"));
        let handoff = npc_side_quest_handoff(MapKind::Village, npc, &quest).unwrap();
        let choice = handoff.choice.expect("ready contact should ask to turn in");
        assert_eq!(
            choice.kind,
            DialogueChoiceKind::SideQuest {
                side: SideQuest::VillageTrail,
                action: SideQuestChoiceAction::TurnIn
            }
        );
        let gold_before = stats.gold;
        resolve_dialogue_choice(choice, &mut quest, &mut stats);
        assert!(quest.is_side_quest_completed(SideQuest::VillageTrail));
        assert_eq!(stats.gold, gold_before + 18);

        let handoff = npc_side_quest_handoff(MapKind::Village, npc, &quest).unwrap();
        assert!(handoff.lines.iter().any(|line| line.contains("药圃护路")));
        assert_eq!(
            handoff
                .choice
                .expect("follow-up should now be claimable")
                .kind,
            DialogueChoiceKind::SideQuest {
                side: SideQuest::VillageHerbs,
                action: SideQuestChoiceAction::Accept
            }
        );

        let final_pos = PlayerPos {
            col: 5,
            row: 13,
            facing: IVec2::new(0, -1),
        };
        let final_npc = &NPCS_FINAL_SANCTUM[5];
        let final_prompt =
            npc_side_quest_facing_prompt(MapKind::FinalSanctum, &final_pos, &QuestLog::default())
                .unwrap();
        assert!(final_prompt.contains("委托联系人 红衣幻影"));
        assert!(final_prompt.contains("梦灯余波[可领取]"));
        let handoff =
            npc_side_quest_handoff(MapKind::FinalSanctum, final_npc, &QuestLog::default()).unwrap();
        assert!(handoff.lines.iter().any(|line| line.contains("【领委托】")));
        assert_eq!(
            handoff
                .choice
                .expect("final contact should offer board quest")
                .kind,
            DialogueChoiceKind::SideQuest {
                side: SideQuest::FinalDreamEchoes,
                action: SideQuestChoiceAction::Accept
            }
        );
    }

    #[test]
    fn npc_errands_are_discovered_accepted_and_delivered_across_maps() {
        let mut quest = QuestLog::default();
        let mut stats = PlayerStats::default();
        stats.hp = 50;
        stats.mp = 1;
        let errand = NpcErrand::BambooDewToCave;

        let offer_pos = PlayerPos {
            col: 5,
            row: 7,
            facing: IVec2::new(0, -1),
        };
        let prompt = npc_errand_facing_prompt(MapKind::Bamboo, &offer_pos, &quest).unwrap();
        assert!(prompt.contains("NPC托付 竹露送药 [可托付]"));
        assert!(prompt.contains("空格领取"));
        assert_eq!(
            quest_marker_glyph(&quest, QuestMarkerKind::NpcErrandOffer(errand)),
            "!"
        );
        assert_eq!(
            quest_marker_glyph(&quest, QuestMarkerKind::NpcErrandDelivery(errand)),
            ""
        );

        let handoff = npc_errand_handoff(MapKind::Bamboo, &NPCS_BAMBOO[0], &quest).unwrap();
        assert!(handoff.lines.iter().any(|line| line.contains("竹林猎户")));
        let choice = handoff
            .choice
            .expect("available errand should ask for confirmation");
        assert_eq!(
            choice.kind,
            DialogueChoiceKind::NpcErrand {
                errand,
                action: NpcErrandChoiceAction::Accept
            }
        );
        let result = resolve_dialogue_choice(choice, &mut quest, &mut stats);
        assert!(
            result
                .lines
                .iter()
                .any(|line| line.contains("路人托付已接"))
        );
        assert!(quest.is_npc_errand_active(errand));
        assert!(quest.active_task_tracker().contains("NPC托付 · 竹露送药"));
        assert_eq!(
            quest_marker_glyph(&quest, QuestMarkerKind::NpcErrandOffer(errand)),
            "*"
        );
        assert_eq!(
            quest_marker_glyph(&quest, QuestMarkerKind::NpcErrandDelivery(errand)),
            "!"
        );

        let delivery_pos = PlayerPos {
            col: 24,
            row: 13,
            facing: IVec2::new(0, -1),
        };
        let prompt = npc_errand_facing_prompt(MapKind::Cave, &delivery_pos, &quest).unwrap();
        assert!(prompt.contains("NPC托付 竹露送药 [进行中]"));
        assert!(prompt.contains("空格交付"));

        let delivery = npc_errand_handoff(MapKind::Cave, &NPCS_CAVE[2], &quest).unwrap();
        let choice = delivery
            .choice
            .expect("active errand should be deliverable");
        assert_eq!(
            choice.kind,
            DialogueChoiceKind::NpcErrand {
                errand,
                action: NpcErrandChoiceAction::TurnIn
            }
        );
        let result = resolve_dialogue_choice(choice, &mut quest, &mut stats);
        assert!(
            result
                .lines
                .iter()
                .any(|line| line.contains("路人托付送达"))
        );
        assert!(
            result
                .lines
                .iter()
                .any(|line| line.contains("【托付回礼】"))
        );
        assert!(quest.is_npc_errand_completed(errand));
        assert_eq!(stats.potions, 4);
        assert_eq!(stats.gold, 88);
        assert_eq!(stats.hp, 60);
        assert_eq!(stats.mp, 5);
        assert_eq!(
            quest_marker_glyph(&quest, QuestMarkerKind::NpcErrandOffer(errand)),
            "✓"
        );
        assert_eq!(
            quest_marker_glyph(&quest, QuestMarkerKind::NpcErrandDelivery(errand)),
            "✓"
        );

        let repeat = npc_errand_handoff(MapKind::Cave, &NPCS_CAVE[2], &quest).unwrap();
        assert!(repeat.choice.is_none());
        assert!(repeat.lines[0].contains("已经送达"));
    }

    #[test]
    fn late_npc_errand_contacts_are_mapped_to_existing_npcs() {
        let pairs = [
            (
                MapKind::MoonEchoCorridor,
                &NPCS_MOON_ECHO_CORRIDOR[4],
                MapKind::RiverTown,
                &NPCS_RIVER_TOWN[2],
                NpcErrand::MoonMossToRiver,
            ),
            (
                MapKind::Capital,
                &NPCS_CAPITAL[1],
                MapKind::MansionMirrorGallery,
                &NPCS_MANSION_MIRROR_GALLERY[2],
                NpcErrand::CapitalStarSlip,
            ),
            (
                MapKind::MansionMirrorGallery,
                &NPCS_MANSION_MIRROR_GALLERY[3],
                MapKind::SouthernRoad,
                &NPCS_SOUTHERN_ROAD[4],
                NpcErrand::MirrorMedicineToSouth,
            ),
            (
                MapKind::FinalSanctum,
                &NPCS_FINAL_SANCTUM[3],
                MapKind::DreamWaterway,
                &NPCS_DREAM_WATERWAY[2],
                NpcErrand::FinalLampWick,
            ),
        ];

        for (offer_map, offer_npc, delivery_map, delivery_npc, errand) in pairs {
            assert_eq!(offer_npc.quest, None);
            assert_eq!(delivery_npc.quest, None);
            assert_eq!(npc_errand_offer_for(offer_map, offer_npc), Some(errand));
            assert_eq!(
                npc_errand_delivery_for(delivery_map, delivery_npc),
                Some(errand)
            );
        }
    }

    #[test]
    fn task_board_menu_lists_all_local_commissions_before_confirming() {
        let mut quest = QuestLog::default();
        let mut stats = PlayerStats::default();

        let board = DialogueChoice::side_board(MapKind::Village, &quest);
        assert_eq!(board.option_count(), 2);
        assert_eq!(
            board.option_label(0),
            "领取追踪 · 山路余妖 0/2 · 余杭-巡山-壹"
        );
        assert_eq!(
            board.option_label(1),
            "查看后续 · 药圃护路 0/2 · 余杭-药圃-贰"
        );

        let mut locked_board = board;
        locked_board.selected = 1;
        let result = resolve_dialogue_choice(locked_board, &mut quest, &mut stats);
        assert!(result.choice.is_none());
        assert!(
            result
                .lines
                .iter()
                .any(|line| line.contains("【后续委托】"))
        );
        assert!(!quest.is_side_quest_active(SideQuest::VillageHerbs));

        let result = resolve_dialogue_choice(board, &mut quest, &mut stats);
        assert!(result.lines[0].contains("选中《山路余妖》"));
        assert!(
            result
                .lines
                .iter()
                .any(|line| line.contains("【追踪预览】"))
        );
        assert!(
            result
                .lines
                .iter()
                .any(|line| line.contains("【委托契约】"))
        );
        let accept = result.choice.expect("board selection should ask to accept");
        assert_eq!(
            accept.kind,
            DialogueChoiceKind::SideQuest {
                side: SideQuest::VillageTrail,
                action: SideQuestChoiceAction::Accept
            }
        );

        let lines = resolve_dialogue_choice(accept, &mut quest, &mut stats).lines;
        assert!(lines.iter().any(|line| line.contains("【领取委托】")));
        assert!(quest.is_side_quest_active(SideQuest::VillageTrail));

        let board = DialogueChoice::side_board(MapKind::Village, &quest);
        assert_eq!(
            board.option_label(0),
            "查看进度 · 山路余妖 0/2 · 余杭-巡山-壹"
        );
        assert_eq!(
            board.option_label(1),
            "查看后续 · 药圃护路 0/2 · 余杭-药圃-贰"
        );

        quest.record_side_victory();
        quest.record_side_victory();
        let board = DialogueChoice::side_board(MapKind::Village, &quest);
        assert_eq!(
            board.option_label(0),
            "交付领奖 · 山路余妖 2/2 · 余杭-巡山-壹"
        );

        let result = resolve_dialogue_choice(board, &mut quest, &mut stats);
        let turn_in = result.choice.expect("ready task should ask to turn in");
        assert_eq!(
            turn_in.kind,
            DialogueChoiceKind::SideQuest {
                side: SideQuest::VillageTrail,
                action: SideQuestChoiceAction::TurnIn
            }
        );

        resolve_dialogue_choice(turn_in, &mut quest, &mut stats);
        let board = DialogueChoice::side_board(MapKind::Village, &quest);
        assert_eq!(
            board.option_label(0),
            "领取追踪 · 药圃护路 0/2 · 余杭-药圃-贰"
        );
        assert_eq!(
            board.option_label(1),
            "查看归档 · 山路余妖 2/2 · 余杭-巡山-壹"
        );
    }

    #[test]
    fn task_board_choice_confirms_accept_and_turn_in() {
        let mut quest = QuestLog::default();
        let mut stats = PlayerStats::default();
        let side = SideQuest::VillageTrail;

        let choice = side_quest_choice_for(&quest, side).expect("available side quest");
        assert_eq!(
            choice.kind,
            DialogueChoiceKind::SideQuest {
                side,
                action: SideQuestChoiceAction::Accept
            }
        );
        assert_eq!(choice.prompt(), "要签下这份委托契约并开始追踪吗？");
        assert_eq!(choice.option_label(0), "领取并追踪");
        assert!(!quest.is_side_quest_active(side));

        let mut cancel = choice;
        cancel.selected = 1;
        let lines = resolve_dialogue_choice(cancel, &mut quest, &mut stats).lines;
        assert!(lines[0].contains("暂不领取"));
        assert!(!quest.is_side_quest_active(side));

        let lines = resolve_dialogue_choice(choice, &mut quest, &mut stats).lines;
        assert!(lines.iter().any(|line| line.contains("【委托契约】")));
        assert!(lines.iter().any(|line| line.contains("【领取委托】")));
        assert!(lines.iter().any(|line| line.contains("【任务追踪】")));
        assert!(quest.is_side_quest_active(side));

        quest.record_side_victory();
        quest.record_side_victory();
        let turn_in = side_quest_choice_for(&quest, side).expect("ready side quest");
        assert_eq!(
            turn_in.kind,
            DialogueChoiceKind::SideQuest {
                side,
                action: SideQuestChoiceAction::TurnIn
            }
        );
        assert_eq!(turn_in.prompt(), "要怎样交付这份委托契约？");
        assert_eq!(turn_in.option_count(), 3);
        assert_eq!(turn_in.option_label(0), "稳妥封存");
        assert_eq!(turn_in.option_label(1), "追查余波");
        assert_eq!(turn_in.option_label(2), "先不处理");

        let potions_before = stats.potions;
        let gold_before = stats.gold;
        let mut pursue = turn_in;
        pursue.selected = 1;
        let lines = resolve_dialogue_choice(pursue, &mut quest, &mut stats).lines;
        assert!(quest.is_side_quest_completed(side));
        assert_eq!(
            quest.side_quest_resolution(side),
            Some(SideQuestResolution::Pursue)
        );
        assert!(lines.iter().any(|line| line.contains("【委托契约】")));
        assert!(lines.iter().any(|line| line.contains("【支线完成】")));
        assert!(lines.iter().any(|line| line.contains("追查余波")));
        assert!(lines.iter().any(|line| line.contains("【奖励】")));
        assert_eq!(stats.potions, potions_before + 1);
        assert_eq!(stats.gold, gold_before + 18);
    }

    #[test]
    fn accepting_commission_grants_advance_supplies_once() {
        let mut quest = QuestLog::default();
        let mut stats = PlayerStats {
            mp: 10,
            ..default()
        };
        let choice =
            DialogueChoice::side_quest(SideQuest::VillageTrail, SideQuestChoiceAction::Accept);

        let lines = resolve_dialogue_choice(choice, &mut quest, &mut stats).lines;
        assert!(lines.iter().any(|line| line.contains("【委托预支】")));
        assert!(lines.iter().any(|line| line.contains("药水 +1")));
        assert!(lines.iter().any(|line| line.contains("路费 +4文")));
        assert_eq!(stats.potions, 4);
        assert_eq!(stats.gold, 84);
        assert_eq!(stats.mp, 12);

        let repeat = resolve_dialogue_choice(choice, &mut quest, &mut stats).lines;
        assert!(!repeat.iter().any(|line| line.contains("【委托预支】")));
        assert_eq!(stats.potions, 4);
        assert_eq!(stats.gold, 84);
        assert_eq!(stats.mp, 12);
    }

    #[test]
    fn confirmed_task_choices_raise_quest_notices() {
        let mut quest = QuestLog::default();
        let mut stats = PlayerStats::default();

        let main =
            DialogueChoice::main_quest(QuestRole::SwordSister, MainQuestChoiceAction::Accept);
        resolve_dialogue_choice(main, &mut quest, &mut stats);
        let notice = quest_notice_for_confirmed_choice(main, &quest).expect("main accept notice");
        assert_eq!(notice.kind, QuestNoticeKind::Main);
        assert!(notice.title.contains("主线已接取"));
        assert!(notice.body.contains("红衣剑姊"));
        assert!(notice.body.contains("主线追踪卡"));

        let side =
            DialogueChoice::side_quest(SideQuest::VillageTrail, SideQuestChoiceAction::Accept);
        resolve_dialogue_choice(side, &mut quest, &mut stats);
        let notice = quest_notice_for_confirmed_choice(side, &quest).expect("side accept notice");
        assert_eq!(notice.kind, QuestNoticeKind::Side);
        assert!(notice.title.contains("委托契约已领取"));
        assert!(notice.body.contains("山路余妖"));
        assert!(notice.body.contains("0/2"));
        assert!(notice.body.contains("下一步："));
        assert!(notice.body.contains("现场："));
        assert!(notice.body.contains("HUD 委托追踪卡"));
        assert!(notice.body.contains("回委托点交付领奖"));
        assert!(side_quest_choice_for(&quest, SideQuest::VillageHerbs).is_none());

        let errand =
            DialogueChoice::npc_errand(NpcErrand::BambooDewToCave, NpcErrandChoiceAction::Accept);
        resolve_dialogue_choice(errand, &mut quest, &mut stats);
        let notice =
            quest_notice_for_confirmed_choice(errand, &quest).expect("npc errand accept notice");
        assert_eq!(notice.kind, QuestNoticeKind::Side);
        assert!(notice.title.contains("NPC托付已接下"));
        assert!(notice.body.contains("目标：把竹林晨露"));
        assert!(notice.body.contains("路线：从青竹山径东门"));
        assert!(notice.body.contains("交付：水月洞天"));
        assert!(notice.body.contains("HUD 托付追踪卡"));

        let mut cancel =
            DialogueChoice::side_quest(SideQuest::VillageHerbs, SideQuestChoiceAction::Accept);
        cancel.selected = 1;
        assert!(quest_notice_for_confirmed_choice(cancel, &quest).is_none());

        quest.record_side_victory();
        quest.record_side_victory();
        let turn_in =
            DialogueChoice::side_quest(SideQuest::VillageTrail, SideQuestChoiceAction::TurnIn);
        resolve_dialogue_choice(turn_in, &mut quest, &mut stats);
        let notice =
            quest_notice_for_confirmed_choice(turn_in, &quest).expect("side turn-in notice");
        assert_eq!(notice.kind, QuestNoticeKind::Complete);
        assert!(notice.title.contains("委托契约已交付"));
        assert!(notice.body.contains("委托签归档"));
        assert!(notice.body.contains("报酬已入袋"));

        let board = DialogueChoice::side_board(MapKind::Village, &quest);
        assert!(quest_notice_for_confirmed_choice(board, &quest).is_none());
    }

    #[test]
    fn main_quest_choice_confirms_pickup_and_boss_handoff() {
        let mut quest = QuestLog::default();
        let mut stats = PlayerStats::default();
        let choice = main_quest_choice_for(&quest, QuestRole::SwordSister)
            .expect("opening main quest should be claimable");
        assert_eq!(
            choice.kind,
            DialogueChoiceKind::MainQuest {
                role: QuestRole::SwordSister,
                action: MainQuestChoiceAction::Accept
            }
        );
        assert_eq!(choice.prompt(), "要领取这条主线任务并写入任务簿吗？");
        assert_eq!(choice.option_label(0), "领取并追踪");

        let mut cancel = choice;
        cancel.selected = 1;
        let result = resolve_dialogue_choice(cancel, &mut quest, &mut stats);
        assert!(result.lines[0].contains("主线暂缓"));
        assert_eq!(quest.stage(), QuestStage::NotStarted);
        assert!(matches!(result.after, DialogueAfter::None));

        let result = resolve_dialogue_choice(choice, &mut quest, &mut stats);
        assert!(
            result
                .lines
                .iter()
                .any(|line| line.contains("【接取任务】"))
        );
        assert_eq!(quest.stage(), QuestStage::TalkToLinger);
        assert!(matches!(result.after, DialogueAfter::None));

        quest.talk(QuestRole::Linger);
        quest.talk(QuestRole::StarMage);
        quest.record_victory();
        quest.record_victory();
        quest.talk(QuestRole::SwordSister);
        quest.talk(QuestRole::Merchant);
        quest.talk(QuestRole::BambooScout);
        quest.talk(QuestRole::CavePriestess);
        quest.activate_moon_crystal(MoonCrystal::North);
        quest.activate_moon_crystal(MoonCrystal::South);
        quest.record_victory();
        quest.record_victory();
        quest.record_victory();
        assert_eq!(quest.stage(), QuestStage::ConfrontMoonWraith);

        let boss = main_quest_choice_for(&quest, QuestRole::CavePriestess)
            .expect("boss handoff should be confirmable");
        assert_eq!(boss.option_label(0), "迎战首领");
        let result = resolve_dialogue_choice(boss, &mut quest, &mut stats);
        assert!(
            result
                .lines
                .iter()
                .any(|line| line.contains("【Boss】月魄妖"))
        );
        assert!(matches!(
            result.after,
            DialogueAfter::StartBattle(PendingEncounter {
                zone: EncounterZone::Cave,
                kind: EncounterKind::Boss(BossKind::MoonWraith)
            })
        ));
    }

    #[test]
    fn camp_tactic_choice_supports_three_party_plans() {
        let mut quest = QuestLog::default();
        let mut stats = PlayerStats::default();
        let choice = DialogueChoice::camp_tactic(CampBonus::Focus);

        assert_eq!(choice.kind, DialogueChoiceKind::CampTactic);
        assert_eq!(choice.option_count(), 3);
        assert_eq!(choice.prompt(), "下一场战斗采用哪种营地战术？");
        assert_eq!(choice.option_label(0), "围坐调息");
        assert_eq!(choice.option_label(1), "月衡破势");
        assert_eq!(choice.option_label(2), "灵儿守护");
        assert_eq!(choice.selected_camp_bonus(), CampBonus::Focus);

        let mut selected = choice;
        selected.selected = 2;
        let lines = resolve_dialogue_choice(selected, &mut quest, &mut stats).lines;

        assert_eq!(quest.active_camp_bonus(), Some(CampBonus::Vigil));
        assert!(lines[0].contains("灵儿守护"));
        assert!(lines[1].contains("预警抵消"));
        assert!(lines[2].contains("轮值守夜"));
    }

    #[test]
    fn quest_marker_visuals_hide_inactive_targets() {
        let mut quest = QuestLog::default();

        let sword_marker =
            quest_marker_glyph(&quest, QuestMarkerKind::Role(QuestRole::SwordSister));
        let linger_marker = quest_marker_glyph(&quest, QuestMarkerKind::Role(QuestRole::Linger));
        let board_marker = quest_marker_glyph(&quest, QuestMarkerKind::SideBoard(MapKind::Village));

        assert_eq!(sword_marker, "!");
        assert!(quest_marker_style(sword_marker).is_some());
        assert_eq!(linger_marker, "?");
        assert!(quest_marker_style(linger_marker).is_none());
        assert_eq!(board_marker, "!");
        assert!(quest_marker_style(board_marker).is_some());

        quest.interact_side_quest(SideQuest::VillageTrail);
        let board_marker = quest_marker_glyph(&quest, QuestMarkerKind::SideBoard(MapKind::Village));
        assert_eq!(board_marker, "*");
        assert!(quest_marker_style(board_marker).is_some());

        quest.record_side_victory();
        quest.record_side_victory();
        quest.interact_side_quest(SideQuest::VillageTrail);
        let board_marker = quest_marker_glyph(&quest, QuestMarkerKind::SideBoard(MapKind::Village));
        assert_eq!(board_marker, "!");

        quest.interact_side_quest(SideQuest::VillageHerbs);
        quest.record_side_victory();
        quest.record_side_victory();
        quest.interact_side_quest(SideQuest::VillageHerbs);
        let board_marker = quest_marker_glyph(&quest, QuestMarkerKind::SideBoard(MapKind::Village));
        assert_eq!(board_marker, "✓");
    }

    #[test]
    fn party_followers_activate_in_story_order() {
        let mut quest = QuestLog::default();
        assert!(desired_party_followers(&quest).is_empty());

        quest.talk(QuestRole::SwordSister);
        quest.talk(QuestRole::Linger);
        assert_eq!(desired_party_followers(&quest), vec![Companion::Linger]);

        quest.talk(QuestRole::StarMage);
        quest.record_victory();
        quest.record_victory();
        quest.talk(QuestRole::SwordSister);
        assert_eq!(
            desired_party_followers(&quest),
            vec![Companion::Linger, Companion::SwordSister]
        );

        let mut quest = quest_at_mansion_mirror_puzzle();
        quest.align_mansion_mirror(MansionMirrorNode::Ledger);
        quest.align_mansion_mirror(MansionMirrorNode::Witness);
        quest.talk(QuestRole::MansionSpy);
        quest.record_boss_victory(BossKind::MirrorMinister);
        quest.talk(QuestRole::SpiritGuide);
        quest.talk(QuestRole::TribalChief);
        assert_eq!(
            desired_party_followers(&quest),
            vec![
                Companion::Linger,
                Companion::SwordSister,
                Companion::SpiritWitch,
            ]
        );
    }

    #[test]
    fn follower_position_stays_behind_player_facing() {
        let mut pos = PlayerPos {
            col: 10,
            row: 10,
            facing: IVec2::new(1, 0),
        };
        let base = tile_to_world(pos.col, pos.row);
        let follower = follower_slot_position(&pos, 0, false, 0.0);
        assert!(follower.x < base.x);
        assert!(follower.y > base.y);
        assert!(!follower_faces_left(&pos));

        pos.facing = IVec2::new(-1, 0);
        let follower = follower_slot_position(&pos, 0, false, 0.0);
        assert!(follower.x > base.x);
        assert!(follower_faces_left(&pos));
    }

    #[test]
    fn shop_service_buys_chapter_gear_once_before_potions() {
        let mut quest = QuestLog::default();
        let mut stats = PlayerStats::default();

        let result = apply_npc_service(NpcService::Shop, MapKind::Village, &mut quest, &mut stats);
        assert!(result.line.contains("【装备铺】"));
        assert!(result.line.contains("竹剑穗"));
        assert!(result.line.contains("攻击 +2"));
        assert_eq!(stats.gold, 26);
        assert_eq!(stats.atk, 18);
        assert_eq!(stats.def, 7);
        assert_eq!(stats.potions, 3);
        assert!(quest.has_shop_gear(ShopGear::VillageSwordTassel));

        let result = apply_npc_service(NpcService::Shop, MapKind::Village, &mut quest, &mut stats);
        assert_eq!(
            result,
            NpcServiceResult {
                line: "【药铺】花 18 文买入一瓶药水，剩余 8 文。".to_string(),
                rested: false,
            }
        );
        assert_eq!(stats.potions, 4);

        let mut river_quest = QuestLog::default();
        let mut river_stats = PlayerStats::default();
        river_stats.gold = 100;
        let result = apply_npc_service(
            NpcService::Shop,
            MapKind::RiverTown,
            &mut river_quest,
            &mut river_stats,
        );
        assert!(result.line.contains("江绫护衣"));
        assert_eq!(river_stats.gold, 14);
        assert_eq!(river_stats.def, 8);
        assert_eq!(river_stats.max_hp, 92);
        assert_eq!(river_stats.hp, 92);
        assert_eq!(river_stats.max_mp, 24);
        assert_eq!(river_stats.mp, 24);
        assert!(river_quest.has_shop_gear(ShopGear::RiverSilkVest));
    }

    #[test]
    fn npc_services_buy_potions_and_restore_at_inn() {
        let mut stats = PlayerStats::default();
        let mut quest = QuestLog::default();
        quest.record_shop_gear(ShopGear::VillageSwordTassel);

        assert_eq!(stats.gold, 80);
        assert_eq!(
            apply_npc_service(NpcService::Shop, MapKind::Village, &mut quest, &mut stats),
            NpcServiceResult {
                line: "【药铺】花 18 文买入一瓶药水，剩余 62 文。".to_string(),
                rested: false,
            }
        );
        assert_eq!(stats.potions, 4);
        assert_eq!(stats.gold, 62);

        stats.hp = 12;
        stats.mp = 0;
        assert_eq!(
            apply_npc_service(NpcService::Inn, MapKind::RiverTown, &mut quest, &mut stats),
            NpcServiceResult {
                line: "【客栈】花 24 文住了一晚，气血和灵力已恢复。".to_string(),
                rested: true,
            }
        );
        assert_eq!(stats.hp, stats.max_hp);
        assert_eq!(stats.mp, stats.max_mp);
        assert_eq!(stats.gold, 38);

        stats.hp = 1;
        stats.mp = 1;
        stats.gold = 0;
        assert_eq!(
            apply_npc_service(NpcService::Inn, MapKind::RiverTown, &mut quest, &mut stats),
            NpcServiceResult {
                line: "【客栈】住店要 24 文，你的钱不够。".to_string(),
                rested: false,
            }
        );
        assert_eq!(stats.hp, 1);
        assert_eq!(stats.mp, 1);

        let result = apply_npc_service(NpcService::CampRest, MapKind::Cave, &mut quest, &mut stats);
        assert!(result.rested);
        assert!(result.line.contains("休整"));
        assert_eq!(stats.hp, stats.max_hp);
        assert_eq!(stats.mp, stats.max_mp);
    }

    #[test]
    fn completed_local_commissions_unlock_service_favor() {
        let mut quest = QuestLog::default();
        quest.interact_side_quest(SideQuest::VillageTrail);
        quest.record_side_victory();
        quest.record_side_victory();
        quest.interact_side_quest(SideQuest::VillageTrail);
        quest.interact_side_quest(SideQuest::VillageHerbs);
        quest.record_side_victory();
        quest.record_side_victory();
        quest.interact_side_quest(SideQuest::VillageHerbs);
        assert!(local_favor_unlocked(MapKind::Village, &quest));
        quest.record_shop_gear(ShopGear::VillageSwordTassel);

        let mut stats = PlayerStats::default();
        assert_eq!(
            apply_npc_service(NpcService::Shop, MapKind::Village, &mut quest, &mut stats),
            NpcServiceResult {
                line: "【药铺】乡里护持，花 12 文买入一瓶药水，剩余 68 文。".to_string(),
                rested: false,
            }
        );
        assert_eq!(stats.potions, 4);
        assert_eq!(stats.gold, 68);

        let mut river_quest = QuestLog::default();
        river_quest.interact_side_quest(SideQuest::RiverLanterns);
        river_quest.record_side_victory();
        river_quest.record_side_victory();
        river_quest.interact_side_quest(SideQuest::RiverLanterns);
        river_quest.interact_side_quest(SideQuest::RiverCargo);
        river_quest.record_side_victory();
        river_quest.record_side_victory();
        river_quest.interact_side_quest(SideQuest::RiverCargo);
        assert!(local_favor_unlocked(MapKind::RiverTown, &river_quest));

        let mut stats = PlayerStats::default();
        stats.hp = 12;
        stats.mp = 0;
        assert_eq!(
            apply_npc_service(
                NpcService::Inn,
                MapKind::RiverTown,
                &mut river_quest,
                &mut stats
            ),
            NpcServiceResult {
                line: "【客栈】乡里护持，花 12 文住了一晚，气血和灵力已恢复。".to_string(),
                rested: true,
            }
        );
        assert_eq!(stats.gold, 68);
        assert_eq!(stats.hp, stats.max_hp);
        assert_eq!(stats.mp, stats.max_mp);
    }

    #[test]
    fn dream_waterway_lanterns_map_to_unique_final_lamps() {
        assert_eq!(
            final_lamp_for_prop(MapKind::DreamWaterway, &PROPS_DREAM_WATERWAY[1]),
            Some(FinalLamp::Memory)
        );
        assert_eq!(
            final_lamp_for_prop(MapKind::DreamWaterway, &PROPS_DREAM_WATERWAY[2]),
            Some(FinalLamp::Vow)
        );
        assert_eq!(
            final_lamp_for_prop(MapKind::DreamWaterway, &PROPS_DREAM_WATERWAY[3]),
            Some(FinalLamp::Fate)
        );
        assert_eq!(
            final_lamp_for_prop(MapKind::DreamWaterway, &PROPS_DREAM_WATERWAY[0]),
            None
        );
        assert_eq!(
            final_lamp_for_prop(MapKind::FinalSanctum, &PROPS_FINAL_SANCTUM[1]),
            None
        );
        assert_eq!(
            final_lamp_for_prop(MapKind::SouthernRoad, &PROPS_DREAM_WATERWAY[1]),
            None
        );
    }

    #[test]
    fn moon_cave_crystals_map_to_unique_switches() {
        assert_eq!(
            moon_crystal_for_prop(MapKind::MoonEchoCorridor, &PROPS_MOON_ECHO_CORRIDOR[0]),
            Some(MoonCrystal::North)
        );
        assert_eq!(
            moon_crystal_for_prop(MapKind::MoonEchoCorridor, &PROPS_MOON_ECHO_CORRIDOR[1]),
            Some(MoonCrystal::South)
        );
        assert_eq!(
            moon_crystal_for_prop(MapKind::Bamboo, &PROPS_BAMBOO[2]),
            None
        );
        assert_eq!(moon_crystal_for_prop(MapKind::Cave, &PROPS_CAVE[0]), None);
        assert_eq!(
            moon_crystal_for_prop(MapKind::MoonEchoCorridor, &PROPS_MOON_ECHO_CORRIDOR[4]),
            None
        );
    }

    #[test]
    fn scene_motion_profiles_cover_static_props_and_lights() {
        assert_eq!(
            scene_motion_for_prop_path("props/ai_quest_board.png"),
            SceneMotionKind::QuestBoard
        );
        assert_eq!(
            scene_motion_for_prop_path("props/ai_spirit_lantern.png"),
            SceneMotionKind::SpiritLantern
        );
        assert_eq!(
            scene_motion_for_prop_path("props/ai_cave_crystal.png"),
            SceneMotionKind::Crystal
        );
        assert_eq!(
            scene_motion_for_prop_path("props/ai_bamboo_gate.png"),
            SceneMotionKind::Gate
        );

        let npc = scene_motion_profile(SceneMotionKind::Npc);
        let lantern = scene_motion_profile(SceneMotionKind::SpiritLantern);
        let gate = scene_motion_profile(SceneMotionKind::Gate);

        assert!(npc.y_amp > 0.0);
        assert!(lantern.light_intensity_amp > npc.light_intensity_amp);
        assert!(gate.scale_amp > lantern.scale_amp);
    }

    #[test]
    fn character_body_motions_keep_explore_sprites_alive() {
        let npc_a = npc_body_motion(0.35);
        let npc_b = npc_body_motion(1.70);

        assert!(npc_a.offset.length() > 0.25);
        assert_ne!(npc_a.rotation, npc_b.rotation);
        assert!((npc_a.scale.x - npc_a.scale.y).abs() > 0.002);

        let idle = follower_body_motion(0, false, 0.35);
        let walking = follower_body_motion(0, true, 0.35);

        assert!(walking.offset.y > idle.offset.y.abs());
        assert!(walking.rotation.abs() > idle.rotation.abs());
        assert!((walking.scale.x - walking.scale.y).abs() > (idle.scale.x - idle.scale.y).abs());

        let specs = cutout_part_specs(cutout_source_px_for_path("npcs/ai_sword_sister.png"), 64.0);
        assert_eq!(specs.len(), CutoutPart::ALL.len());
        assert!(specs[0].offset.y < specs[1].offset.y);
        assert!(specs[2].offset.y > specs[1].offset.y);

        let lower = cutout_part_motion(CutoutPart::Lower, 0.8, 1.0);
        let head = cutout_part_motion(CutoutPart::Head, 0.8, 1.0);
        assert_ne!(lower.rotation, head.rotation);
        assert!(head.offset.x.abs() > lower.offset.x.abs());
    }

    #[test]
    fn character_shadow_and_afterimage_frames_are_animated() {
        let shadow_a = character_shadow_frame(0.28, 0.20);
        let shadow_b = character_shadow_frame(0.28, 1.50);

        assert_ne!(shadow_a.scale, shadow_b.scale);
        assert_ne!(shadow_a.alpha, shadow_b.alpha);
        assert!((0.02..=0.62).contains(&shadow_a.alpha));
        assert!((0.02..=0.62).contains(&shadow_b.alpha));

        let early = character_afterimage_frame(0.05, 0.35).expect("active afterimage frame");
        let late = character_afterimage_frame(0.28, 0.35).expect("fading afterimage frame");

        assert!(early.progress < late.progress);
        assert!(early.alpha_scale > late.alpha_scale);
        assert!(character_afterimage_frame(0.35, 0.35).is_none());
    }
}
