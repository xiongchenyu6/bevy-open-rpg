use bevy::{ecs::system::SystemParam, prelude::*};
use bevy_firefly::prelude::{Occluder2d, PointLight2d};

use super::animation::{self, AnimationAssets, AnimationClip, SpriteAnimation};
use super::battle::{EncounterKind, EncounterZone, PendingEncounter};
use super::core::{
    EncounterRate, GameFont, Intent, MAP_H, MAP_W, PlayerStats, Rng, TILE, tile_to_world,
};
use super::fog;
use super::lighting::{self, LightingAssets};
use super::paperdoll::{self, PaperdollAssets, PaperdollStyle};
use super::quest::{
    BondResponse, BondReward, BondScene, BossKind, CampBonus, CampScene, Chapter, Companion,
    FinalLamp, MansionMirrorNode, MoonCrystal, PlagueWard, QuestLog, QuestRole, QuestStage,
    RiverLantern, ShrineBlessing, SideQuest, SideQuestReward, ThunderDrum, TreasureCache,
    TreasureReward,
};
use super::state::AppState;

const POTION_PRICE: u32 = 18;
const FAVORED_POTION_PRICE: u32 = 12;
const INN_PRICE: u32 = 24;
const FAVORED_INN_PRICE: u32 = 12;
const SHRINE_OFFERING_PRICE: u32 = 12;

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
    "#P..,,,,....~~~~...####.....>#",
    "#...,,,,....~~~~.............#",
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
    "#....####..........~~~~......#",
    "#....,,,,,......####.........#",
    "##############################",
];

const MAP_PLAGUE_VILLAGE: [&str; MAP_H as usize] = [
    "##############################",
    "#P.....####....,,,,.....N>...#",
    "#......####....,,,,..........#",
    "#..N.........~~~~~.....###...#",
    "#............~~~~~.....###...#",
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

const MAP_PLAGUE_SHRINE_PATH: [&str; MAP_H as usize] = [
    "##############################",
    "#P..,,,,....~~~~....####...N>#",
    "#...,,,,....~~~~.............#",
    "#..####..N......,,,,....###..#",
    "#.......####.....,,,,....###.#",
    "#..~~~~......N.......####....#",
    "#..~~~~....,,,,......####....#",
    "#......,,,,,,.....~~~~.....N.#",
    "#..####.....~~~~.....####....#",
    "#..####..N..~~~~.....####....#",
    "#....N......,,,,,,...........#",
    "#...........,,,,,,......N....#",
    "#....####..........~~~~......#",
    "#....####..N.......~~~~......#",
    "#....,,,,,......####.....N...#",
    "##############################",
];

const MAP_CAPITAL: [&str; MAP_H as usize] = [
    "##############################",
    "#P....####....,,,,.....N>....#",
    "#.....####....,,,,...........#",
    "#..N.........~~~~~.....###...#",
    "#............~~~~~.....###...#",
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

const MAP_CAPITAL_MANSION: [&str; MAP_H as usize] = [
    "##############################",
    "#P....####....,,,,.....N...>.#",
    "#.....####....,,,,...........#",
    "#..N.........~~~~~.....###...#",
    "#............~~~~~.....###...#",
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

const MAP_FINAL_SANCTUM: [&str; MAP_H as usize] = [
    "##############################",
    "#P....####....~~~~.....N>....#",
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

const MAP_DREAM_WATERWAY: [&str; MAP_H as usize] = [
    "##############################",
    "#P....####....~~~~.....N>....#",
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
    "#....,,,,,....####.........N.#",
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
        }
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
    if let Some(line) = local_treasure_reaction(kind, quest) {
        lines.push(line.to_string());
    }
    if let Some(line) = local_care_reaction(kind, quest) {
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
        lines.push(match kind {
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
            MapKind::Bamboo
            | MapKind::MoonEchoCorridor
            | MapKind::RiverReedBed
            | MapKind::PlagueShrinePath
            | MapKind::CapitalMansion
            | MapKind::MansionMirrorGallery
            | MapKind::ThunderDrumPath
            | MapKind::FinalSanctum
            | MapKind::DreamWaterway => return lines,
        });
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
        .map(|side| format!("【街谈】{}", quest.side_task_completed_line(side)))
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

const PROPS_RIVER_REED_BED: [PropDef; 6] = [
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

const PROPS_MANSION_MIRROR_GALLERY: [PropDef; 5] = [
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

const PROPS_FINAL_SANCTUM: [PropDef; 4] = [
    PropDef {
        col: 26,
        row: 1,
        path: "props/ai_cave_crystal.png",
        size: 44.0,
        light: [0.42, 0.82, 1.0, 0.28],
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

const SIDE_QUESTS_VILLAGE: [SideQuest; 2] = [SideQuest::VillageTrail, SideQuest::VillageHerbs];
const SIDE_QUESTS_CAVE: [SideQuest; 2] = [SideQuest::MoonCaveCrystals, SideQuest::MoonCaveEchoes];
const SIDE_QUESTS_RIVER_TOWN: [SideQuest; 2] = [SideQuest::RiverLanterns, SideQuest::RiverCargo];
const SIDE_QUESTS_PLAGUE_VILLAGE: [SideQuest; 2] =
    [SideQuest::PlagueRelief, SideQuest::PlagueMedicine];
const SIDE_QUESTS_CAPITAL: [SideQuest; 2] = [SideQuest::CapitalPatrol, SideQuest::CapitalRumors];
const SIDE_QUESTS_SOUTHERN_ROAD: [SideQuest; 2] =
    [SideQuest::SouthernThunder, SideQuest::SouthernDrums];
const MAX_SIDE_BOARD_OPTIONS: usize = 4;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
struct SideBoardTaskOption {
    side: SideQuest,
    status: SideBoardTaskStatus,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SideBoardTaskStatus {
    Ready,
    Active,
    Available,
    Completed,
}

impl SideBoardTaskStatus {
    fn label(self) -> &'static str {
        match self {
            Self::Ready => "可交付",
            Self::Active => "进行中",
            Self::Available => "可领取",
            Self::Completed => "已完成",
        }
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
        MapKind::Bamboo
        | MapKind::MoonEchoCorridor
        | MapKind::RiverReedBed
        | MapKind::PlagueShrinePath
        | MapKind::CapitalMansion
        | MapKind::MansionMirrorGallery
        | MapKind::ThunderDrumPath
        | MapKind::FinalSanctum
        | MapKind::DreamWaterway => &[],
    }
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

fn side_board_task_status(quest: &QuestLog, side: SideQuest) -> SideBoardTaskStatus {
    if quest.is_side_quest_completed(side) {
        SideBoardTaskStatus::Completed
    } else if quest.is_side_quest_active(side) {
        if quest.side_quest_progress(side) >= quest.side_quest_goal(side) {
            SideBoardTaskStatus::Ready
        } else {
            SideBoardTaskStatus::Active
        }
    } else {
        SideBoardTaskStatus::Available
    }
}

fn side_board_status_priority(status: SideBoardTaskStatus) -> usize {
    match status {
        SideBoardTaskStatus::Ready => 0,
        SideBoardTaskStatus::Active => 1,
        SideBoardTaskStatus::Available => 2,
        SideBoardTaskStatus::Completed => 3,
    }
}

fn side_board_task_options(
    kind: MapKind,
    quest: &QuestLog,
) -> [Option<SideBoardTaskOption>; MAX_SIDE_BOARD_OPTIONS] {
    let mut ordered: Vec<SideBoardTaskOption> = side_quests_for_map(kind)
        .iter()
        .copied()
        .map(|side| SideBoardTaskOption {
            side,
            status: side_board_task_status(quest, side),
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
        "委托板 {completed}/{}完成 | 空格打开\n优先：{}[{}]\n{}",
        sides.len(),
        focus.side.name(),
        focus.status.label(),
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
            "{}. [{}] {} {}/{} - {}",
            idx + 1,
            option.status.label(),
            option.side.name(),
            progress,
            quest.side_quest_goal(option.side),
            quest.side_task_summary(option.side)
        ));
    }
    lines.push("选择一份委托查看详情；领取和交付都会再次确认。".to_string());
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

fn side_quest_choice_for(quest: &QuestLog, side: SideQuest) -> Option<DialogueChoice> {
    if quest.is_side_quest_completed(side) {
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
    vec![
        format!(
            "【主线任务】当前委托\n状态：{}\n操作：{}\n委托人：{}\n目标：{}",
            action.status(),
            action.operation(),
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
            MainQuestChoiceAction::Accept => Some(QuestNoticeMessage::new(
                QuestNoticeKind::Main,
                "任务簿更新 · 主线已接取",
                format!(
                    "{}托付：{}\n已加入主线追踪卡",
                    role.name(),
                    quest.objective()
                ),
            )),
            MainQuestChoiceAction::Advance => Some(QuestNoticeMessage::new(
                QuestNoticeKind::Main,
                "任务簿更新 · 主线推进",
                format!("下一步：{}\n主线追踪已更新", quest.objective()),
            )),
            MainQuestChoiceAction::TurnIn => Some(QuestNoticeMessage::new(
                QuestNoticeKind::Complete,
                "任务簿更新 · 主线已交付",
                format!(
                    "{}线索已结，下一步：{}\n主线追踪已更新",
                    role.name(),
                    quest.objective()
                ),
            )),
            MainQuestChoiceAction::Boss => None,
        },
        DialogueChoiceKind::SideQuest { side, action } => match action {
            SideQuestChoiceAction::Accept => Some(QuestNoticeMessage::new(
                QuestNoticeKind::Side,
                "任务簿更新 · 委托已领取",
                format!(
                    "《{}》 {}/{} · 已加入委托追踪卡",
                    quest.side_quest_name(side),
                    quest
                        .side_quest_progress(side)
                        .min(quest.side_quest_goal(side)),
                    quest.side_quest_goal(side)
                ),
            )),
            SideQuestChoiceAction::TurnIn => Some(QuestNoticeMessage::new(
                QuestNoticeKind::Complete,
                "任务簿更新 · 委托已交付",
                format!("《{}》完成 · 报酬已入袋", quest.side_quest_name(side)),
            )),
        },
        DialogueChoiceKind::BondResponse
        | DialogueChoiceKind::CampTactic
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
        },
        DialogueChoiceKind::CampTactic => DialogueChoiceResult {
            lines: quest.record_camp_tactic(choice.selected_camp_bonus()),
            after: DialogueAfter::None,
            choice: None,
        },
        DialogueChoiceKind::SideBoard { .. } => {
            let Some(task) = choice.selected_side_board_task() else {
                return DialogueChoiceResult {
                    lines: vec!["【委托板】没有可查看的委托。".to_string()],
                    after: DialogueAfter::None,
                    choice: None,
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
                        format!(
                            "【交付确认】《{}》条件已达成，确认后发放报酬。",
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
                };
            }

            let stage_before = quest.stage();
            let mut lines = quest.talk(role);
            if quest.stage() != stage_before {
                if let Some(chapter_card) = quest.take_chapter_card() {
                    lines.extend(chapter_card);
                }
            }
            lines.push(quest.main_task_summary());
            DialogueChoiceResult {
                lines,
                after: main_quest_battle_after(role, stage_before),
                choice: None,
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
                };
            }

            let interaction = quest.interact_side_quest(side);
            let mut lines = interaction.lines;
            if let Some(reward) = interaction.reward {
                lines.push(apply_side_quest_reward(stats, reward));
            }
            lines.push(quest.side_task_summary(side));
            lines.push(quest.main_task_summary());
            DialogueChoiceResult {
                lines,
                after: DialogueAfter::None,
                choice: None,
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

fn apply_shrine_offering(
    stats: &mut PlayerStats,
    quest: &mut QuestLog,
    blessing: ShrineBlessing,
) -> Vec<String> {
    if let Some(active) = quest.active_shrine_blessing() {
        return vec![
            format!("【供奉】{}仍在。", active.name()),
            active.battle_line().to_string(),
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
    ]
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

fn apply_npc_service(
    service: NpcService,
    kind: MapKind,
    quest: &QuestLog,
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
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum SideQuestChoiceAction {
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

    fn selected_response(self) -> BondResponse {
        match self.selected {
            0 => BondResponse::Courage,
            _ => BondResponse::Tender,
        }
    }

    fn selected_camp_bonus(self) -> CampBonus {
        camp_tactic_bonus(self.selected)
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
        self.selected == 0
    }

    fn option_count(self) -> usize {
        match self.kind {
            DialogueChoiceKind::CampTactic => 3,
            DialogueChoiceKind::SideBoard { options } => options.into_iter().flatten().count(),
            DialogueChoiceKind::BondResponse
            | DialogueChoiceKind::MainQuest { .. }
            | DialogueChoiceKind::SideQuest { .. } => 2,
        }
    }

    fn prompt(self) -> String {
        match self.kind {
            DialogueChoiceKind::BondResponse => "你要怎样回应赵灵儿？".to_string(),
            DialogueChoiceKind::CampTactic => "下一场战斗采用哪种营地战术？".to_string(),
            DialogueChoiceKind::SideBoard { .. } => "要查看哪一份委托？".to_string(),
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
            } => "要领取这份委托并写入任务簿吗？".to_string(),
            DialogueChoiceKind::SideQuest {
                action: SideQuestChoiceAction::TurnIn,
                ..
            } => "要交付这份委托并领取报酬吗？".to_string(),
        }
    }

    fn option_label(self, index: usize) -> String {
        match self.kind {
            DialogueChoiceKind::BondResponse => match index {
                0 => "我会走在前面。".to_string(),
                _ => "我们慢慢来。".to_string(),
            },
            DialogueChoiceKind::CampTactic => camp_tactic_bonus(index).tactic_label().to_string(),
            DialogueChoiceKind::SideBoard { options } => options
                .get(index)
                .and_then(|option| *option)
                .map(|option| format!("{} · {}", option.side.name(), option.status.label()))
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
                0 => "交付领奖".to_string(),
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
    Lamp(FinalLamp),
    RiverLantern(RiverLantern),
    PlagueWard(PlagueWard),
    MansionMirror(MansionMirrorNode),
    ThunderDrum(ThunderDrum),
    Crystal(MoonCrystal),
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
    map_content: Query<'w, 's, Entity, With<MapContent>>,
    area_banners: Query<'w, 's, Entity, With<AreaBanner>>,
    fog_memory: ResMut<'w, fog::FogMemory>,
}

pub struct ExplorePlugin;

impl Plugin for ExplorePlugin {
    fn build(&self, app: &mut App) {
        let assets = app.world().resource::<AssetServer>().clone();
        app.init_resource::<PlayerPos>()
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
) {
    for row in 0..MAP_H {
        for col in 0..MAP_W {
            let tile = map.at(col, row);
            let p = tile_to_world(col, row);
            let mut tile_entity = commands.spawn((
                MapContent,
                tile_sprite(tile, map.kind, explore_assets),
                Transform::from_xyz(p.x, p.y, 0.0),
                DespawnOnExit(AppState::Explore),
            ));

            if tile == Tile::Wall {
                tile_entity
                    .insert(Occluder2d::rectangle(TILE * 0.92, TILE * 0.92).with_opacity(0.82));
                let (detail_image, detail_color) = match map.kind {
                    MapKind::Village => {
                        (explore_assets.forest.clone(), Color::srgb(0.34, 0.58, 0.35))
                    }
                    MapKind::Bamboo => (
                        explore_assets.bamboo_thicket.clone(),
                        Color::srgb(0.38, 0.70, 0.38),
                    ),
                    MapKind::Cave => (
                        explore_assets.moon_cave_wall.clone(),
                        Color::srgb(0.54, 0.48, 0.72),
                    ),
                    MapKind::MoonEchoCorridor => (
                        explore_assets.moon_cave_wall.clone(),
                        Color::srgb(0.42, 0.52, 0.78),
                    ),
                    MapKind::RiverTown => (
                        explore_assets.shrine_floor.clone(),
                        Color::srgb(0.58, 0.62, 0.52),
                    ),
                    MapKind::RiverReedBed => (
                        explore_assets.bamboo_thicket.clone(),
                        Color::srgb(0.42, 0.58, 0.36),
                    ),
                    MapKind::PlagueVillage => (
                        explore_assets.mystic_grass.clone(),
                        Color::srgb(0.42, 0.52, 0.34),
                    ),
                    MapKind::PlagueShrinePath => (
                        explore_assets.bamboo_thicket.clone(),
                        Color::srgb(0.30, 0.50, 0.30),
                    ),
                    MapKind::Capital => (
                        explore_assets.stone_road.clone(),
                        Color::srgb(0.46, 0.46, 0.58),
                    ),
                    MapKind::CapitalMansion => (
                        explore_assets.stone_road.clone(),
                        Color::srgb(0.34, 0.34, 0.48),
                    ),
                    MapKind::MansionMirrorGallery => (
                        explore_assets.moon_cave_wall.clone(),
                        Color::srgb(0.38, 0.44, 0.66),
                    ),
                    MapKind::SouthernRoad => (
                        explore_assets.bamboo_thicket.clone(),
                        Color::srgb(0.30, 0.64, 0.44),
                    ),
                    MapKind::ThunderDrumPath => (
                        explore_assets.bamboo_thicket.clone(),
                        Color::srgb(0.34, 0.70, 0.58),
                    ),
                    MapKind::FinalSanctum => (
                        explore_assets.moon_cave_wall.clone(),
                        Color::srgb(0.40, 0.42, 0.66),
                    ),
                    MapKind::DreamWaterway => (
                        explore_assets.moon_cave_wall.clone(),
                        Color::srgb(0.38, 0.46, 0.70),
                    ),
                };
                commands.spawn((
                    MapContent,
                    Sprite {
                        image: detail_image,
                        color: detail_color,
                        custom_size: Some(Vec2::splat(TILE * 0.62)),
                        ..default()
                    },
                    Transform::from_xyz(p.x, p.y, 0.5),
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

    for npc in npc_defs(map.kind) {
        spawn_npc(commands, font, asset_server, dolls, lights, npc);
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

pub(crate) fn tile_sprite(tile: Tile, kind: MapKind, assets: &ExploreAssets) -> Sprite {
    let (image, color) = match tile {
        Tile::Wall => match kind {
            MapKind::Village => (assets.forest.clone(), Color::srgb(0.22, 0.43, 0.23)),
            MapKind::Bamboo => (assets.bamboo_thicket.clone(), Color::srgb(0.32, 0.58, 0.32)),
            MapKind::Cave => (assets.moon_cave_wall.clone(), Color::srgb(0.43, 0.40, 0.58)),
            MapKind::MoonEchoCorridor => {
                (assets.moon_cave_wall.clone(), Color::srgb(0.30, 0.38, 0.62))
            }
            MapKind::RiverTown => (assets.shrine_floor.clone(), Color::srgb(0.42, 0.45, 0.38)),
            MapKind::RiverReedBed => (assets.bamboo_thicket.clone(), Color::srgb(0.28, 0.46, 0.28)),
            MapKind::PlagueVillage => (assets.mystic_grass.clone(), Color::srgb(0.24, 0.36, 0.25)),
            MapKind::PlagueShrinePath => {
                (assets.bamboo_thicket.clone(), Color::srgb(0.20, 0.34, 0.24))
            }
            MapKind::Capital => (assets.stone_road.clone(), Color::srgb(0.30, 0.32, 0.42)),
            MapKind::CapitalMansion => (assets.stone_road.clone(), Color::srgb(0.22, 0.22, 0.34)),
            MapKind::MansionMirrorGallery => {
                (assets.moon_cave_wall.clone(), Color::srgb(0.28, 0.34, 0.54))
            }
            MapKind::SouthernRoad => (assets.bamboo_thicket.clone(), Color::srgb(0.20, 0.42, 0.30)),
            MapKind::ThunderDrumPath => {
                (assets.bamboo_thicket.clone(), Color::srgb(0.18, 0.38, 0.34))
            }
            MapKind::FinalSanctum => (assets.moon_cave_wall.clone(), Color::srgb(0.24, 0.24, 0.42)),
            MapKind::DreamWaterway => {
                (assets.moon_cave_wall.clone(), Color::srgb(0.22, 0.28, 0.48))
            }
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
            MapKind::RiverReedBed => (assets.bamboo_path.clone(), Color::srgb(0.78, 0.72, 0.48)),
            MapKind::PlagueVillage => (
                assets.village_moss_path.clone(),
                Color::srgb(0.62, 0.64, 0.48),
            ),
            MapKind::PlagueShrinePath => (
                assets.village_moss_path.clone(),
                Color::srgb(0.58, 0.60, 0.42),
            ),
            MapKind::Capital => (assets.stone_road.clone(), Color::srgb(0.72, 0.72, 0.82)),
            MapKind::CapitalMansion => (assets.shrine_floor.clone(), Color::srgb(0.62, 0.64, 0.78)),
            MapKind::MansionMirrorGallery => {
                (assets.shrine_floor.clone(), Color::srgb(0.58, 0.66, 0.90))
            }
            MapKind::SouthernRoad => (assets.bamboo_path.clone(), Color::srgb(0.82, 0.76, 0.52)),
            MapKind::ThunderDrumPath => (assets.bamboo_path.clone(), Color::srgb(0.74, 0.78, 0.56)),
            MapKind::FinalSanctum => (assets.cave_floor.clone(), Color::srgb(0.66, 0.70, 0.88)),
            MapKind::DreamWaterway => (assets.cave_floor.clone(), Color::srgb(0.62, 0.70, 0.92)),
        },
    };

    Sprite {
        image,
        color,
        custom_size: Some(Vec2::splat(TILE + 0.5)),
        ..default()
    }
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
}

fn spawn_npc(
    commands: &mut Commands,
    font: &GameFont,
    asset_server: &AssetServer,
    dolls: &PaperdollAssets,
    lights: &LightingAssets,
    npc: &NpcDef,
) {
    let p = tile_to_world(npc.col, npc.row);
    let phase = scene_phase(npc.col, npc.row);
    let (entity, light_color, origin) = match npc.visual {
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
            (entity, Color::srgba(1.0, 0.82, 0.42, 0.25), origin)
        }
        NpcVisual::Image { path, size, light } => {
            let origin = Vec3::new(p.x, p.y + 6.0, 5.0);
            let entity = commands
                .spawn((
                    Sprite {
                        image: asset_server.load(path),
                        custom_size: Some(Vec2::splat(size)),
                        ..default()
                    },
                    Transform::from_translation(origin),
                    DespawnOnExit(AppState::Explore),
                ))
                .id();
            (
                entity,
                Color::srgba(light[0], light[1], light[2], light[3]),
                origin,
            )
        }
    };

    commands.entity(entity).insert((
        MapContent,
        SceneMotion::new(SceneMotionKind::Npc, origin, phase),
    ));
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
                if let Some(chapter_card) = quest.take_chapter_card() {
                    lines.extend(chapter_card);
                }
            }
            if !story_advanced && pending_choice.is_none() {
                lines.extend(npc_reaction_lines(map.kind, npc, &quest));
                if let Some(service) = npc_service_for(map.kind, npc) {
                    let service_result = apply_npc_service(service, map.kind, &quest, &mut stats);
                    let rested = service_result.rested;
                    lines.push(service_result.line);
                    if rested {
                        let camp = quest.interact_camp_scene();
                        if camp.tactic_choice {
                            if let Some(default_bonus) = camp.bonus {
                                pending_choice = Some(DialogueChoice::camp_tactic(default_bonus));
                            }
                        }
                        lines.extend(camp.lines);
                    }
                }
            }
            dialogue.active = true;
            dialogue.lines = lines;
            dialogue.idx = 0;
            dialogue.after = after;
            dialogue.choice = pending_choice;
            dialogue.portrait_path = dialogue_portrait_path(npc.visual);
            return;
        }

        if let Some(prop) = prop_defs(map.kind)
            .iter()
            .find(|prop| prop.col == tc && prop.row == tr)
        {
            let mut pending_choice = None;
            let mut lines = if let Some(lantern) = river_lantern_for_prop(map.kind, prop) {
                quest.activate_river_lantern(lantern)
            } else if let Some(node) = mansion_mirror_for_prop(map.kind, prop) {
                quest.align_mansion_mirror(node)
            } else if let Some(ward) = plague_ward_for_prop(map.kind, prop) {
                quest.seal_plague_ward(ward)
            } else if let Some(drum) = thunder_drum_for_prop(map.kind, prop) {
                quest.align_thunder_drum(drum)
            } else if let Some(lamp) = final_lamp_for_prop(map.kind, prop) {
                quest.light_final_lamp(lamp)
            } else if let Some(crystal) = moon_crystal_for_prop(map.kind, prop) {
                quest.activate_moon_crystal(crystal)
            } else if !side_quests_for_board(map.kind, prop).is_empty() {
                pending_choice = Some(DialogueChoice::side_board(map.kind, &quest));
                side_board_overview_lines(map.kind, &quest)
            } else if is_bond_lantern(prop) {
                let interaction = quest.interact_bond_scene();
                let mut lines = interaction.lines;
                if let Some(reward) = interaction.reward {
                    lines.push(apply_bond_reward(&mut stats, reward));
                }
                if interaction.response_choice {
                    pending_choice = Some(DialogueChoice::bond_response());
                }
                lines
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
                        cooldown.0 = 0.18;
                        return;
                    }

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
                    );
                    spawn_area_banner(&mut commands, &spawner.font, &new_map, &quest);
                    commands.insert_resource(new_map);
                    cooldown.0 = 0.18;
                    return;
                }

                pos.col = nc;
                pos.row = nr;
                cooldown.0 = 0.14;
                // Random encounter when stepping into grass.
                if tile == Tile::Grass && rng.chance(rate.0) {
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
    intent: Res<Intent>,
    anims: Res<AnimationAssets>,
    pos: Res<PlayerPos>,
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
    }
    if let Ok(mut t) = lights.single_mut() {
        t.translation.x = p.x;
        t.translation.y = p.y;
    }
}

fn sync_party_followers(
    mut commands: Commands,
    dolls: Res<PaperdollAssets>,
    quest: Res<QuestLog>,
    pos: Res<PlayerPos>,
    intent: Res<Intent>,
    time: Res<Time>,
    mut followers: Query<(Entity, &PartyFollower, &mut Transform, &mut Sprite)>,
) {
    let desired = desired_party_followers(&quest);
    let moving = intent.move_dir.is_some();
    let phase = time.elapsed_secs();
    let mut present = vec![false; desired.len()];

    for (entity, follower, mut transform, mut sprite) in &mut followers {
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
        commands
            .entity(follower)
            .insert((MapContent, PartyFollower { companion }));
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
        let side_quests = compact_hud_text(&quest.side_quest_summary(), 42);
        let main_task = compact_hud_text(&quest.main_task_summary(), 56);
        let local_task = map
            .as_ref()
            .map(|map| compact_hud_text(&side_board_summary(map.kind, &quest), 56))
            .unwrap_or_else(|| "任务板 无".to_string());
        let bonds = quest.bond_summary();
        let camp = quest.camp_summary();
        let care = compact_hud_text(&quest.travel_care_summary(), 42);
        text.0 = format!(
            "{}\n{}\n队伍 {}\n{}  Lv.{}\n气血 {}/{}\n灵力 {}/{}\n药水 x{}  钱 {}文\n道具 {}\n任务簿 {}\n{}\n{}\n{}\n{}\n{}",
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
            key_items,
            main_task,
            local_task,
            side_quests,
            bonds,
            camp,
            care,
        );
    }
}

fn update_task_tracker(
    quest: Res<QuestLog>,
    mut root: Query<&mut Visibility, With<TaskTrackerRoot>>,
    mut text: Query<&mut Text, With<TaskTrackerText>>,
) {
    if let Ok(mut visibility) = root.single_mut() {
        *visibility = Visibility::Inherited;
    }

    if let Ok(mut text) = text.single_mut() {
        text.0 = quest.active_task_tracker();
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
        map.as_ref()
            .and_then(|map| side_board_facing_prompt(map.kind, &pos, &quest))
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
        QuestMarkerKind::Lamp(lamp) => quest.final_lamp_marker_for(lamp),
        QuestMarkerKind::RiverLantern(lantern) => quest.river_lantern_marker_for(lantern),
        QuestMarkerKind::PlagueWard(ward) => quest.plague_ward_marker_for(ward),
        QuestMarkerKind::MansionMirror(node) => quest.mansion_mirror_marker_for(node),
        QuestMarkerKind::ThunderDrum(drum) => quest.thunder_drum_marker_for(drum),
        QuestMarkerKind::Crystal(crystal) => quest.moon_crystal_marker_for(crystal),
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
    asset_server: Res<AssetServer>,
    mut root: Query<&mut Visibility, (With<DialogueRoot>, Without<DialoguePortrait>)>,
    mut line: Query<&mut Text, With<DialogueLine>>,
    mut portrait: Query<
        (&mut Visibility, &mut ImageNode),
        (With<DialoguePortrait>, Without<DialogueRoot>),
    >,
) {
    if let Ok(mut vis) = root.single_mut() {
        *vis = if dialogue.active {
            Visibility::Inherited
        } else {
            Visibility::Hidden
        };
    }
    if let Ok((mut vis, mut image)) = portrait.single_mut() {
        if dialogue.active {
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
        for kind in [
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
        ] {
            let map = MapData::build(kind);
            let (col, row) = map.spawn();
            assert!(map.at(col, row).walkable());
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
        assert!(side_quests_for_map(MapKind::DreamWaterway).is_empty());
        assert!(side_quests_for_board(MapKind::Village, &PROPS_VILLAGE[1]).is_empty());

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
    fn shrine_offering_spends_gold_and_blocks_duplicate_blessings() {
        let mut quest = QuestLog::default();
        let mut stats = PlayerStats::default();

        let lines = apply_shrine_offering(&mut stats, &mut quest, ShrineBlessing::Guard);
        assert_eq!(stats.gold, 80 - SHRINE_OFFERING_PRICE);
        assert_eq!(quest.active_shrine_blessing(), Some(ShrineBlessing::Guard));
        assert!(lines[0].contains("供奉"));
        assert!(lines[1].contains("下一场战斗"));

        let lines = apply_shrine_offering(&mut stats, &mut quest, ShrineBlessing::Sword);
        assert_eq!(stats.gold, 80 - SHRINE_OFFERING_PRICE);
        assert_eq!(quest.active_shrine_blessing(), Some(ShrineBlessing::Guard));
        assert!(lines[0].contains("仍在"));

        quest.take_shrine_blessing();
        stats.gold = 0;
        let lines = apply_shrine_offering(&mut stats, &mut quest, ShrineBlessing::Spirit);
        assert!(lines[0].contains("钱不够"));
        assert_eq!(quest.active_shrine_blessing(), None);
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
        assert_eq!(lines.len(), 1);
        assert!(lines[0].contains("错过的夜谈和休整"));
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
        assert!(prompt.contains("药圃护路[可领取]"));

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
    }

    #[test]
    fn task_board_menu_lists_all_local_commissions_before_confirming() {
        let mut quest = QuestLog::default();
        let mut stats = PlayerStats::default();

        let board = DialogueChoice::side_board(MapKind::Village, &quest);
        assert_eq!(board.option_count(), 2);
        assert_eq!(board.option_label(0), "山路余妖 · 可领取");
        assert_eq!(board.option_label(1), "药圃护路 · 可领取");

        let result = resolve_dialogue_choice(board, &mut quest, &mut stats);
        assert!(result.lines[0].contains("选中《山路余妖》"));
        assert!(
            result
                .lines
                .iter()
                .any(|line| line.contains("【追踪预览】"))
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
        assert_eq!(board.option_label(0), "山路余妖 · 进行中");
        assert_eq!(board.option_label(1), "药圃护路 · 可领取");

        quest.record_side_victory();
        quest.record_side_victory();
        let board = DialogueChoice::side_board(MapKind::Village, &quest);
        assert_eq!(board.option_label(0), "山路余妖 · 可交付");

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
        assert_eq!(board.option_label(0), "药圃护路 · 可领取");
        assert_eq!(board.option_label(1), "山路余妖 · 已完成");
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
        assert_eq!(choice.prompt(), "要领取这份委托并写入任务簿吗？");
        assert_eq!(choice.option_label(0), "领取并追踪");
        assert!(!quest.is_side_quest_active(side));

        let mut cancel = choice;
        cancel.selected = 1;
        let lines = resolve_dialogue_choice(cancel, &mut quest, &mut stats).lines;
        assert!(lines[0].contains("暂不领取"));
        assert!(!quest.is_side_quest_active(side));

        let lines = resolve_dialogue_choice(choice, &mut quest, &mut stats).lines;
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
        assert_eq!(turn_in.option_label(0), "交付领奖");

        let potions_before = stats.potions;
        let gold_before = stats.gold;
        let lines = resolve_dialogue_choice(turn_in, &mut quest, &mut stats).lines;
        assert!(quest.is_side_quest_completed(side));
        assert!(lines.iter().any(|line| line.contains("【支线完成】")));
        assert!(lines.iter().any(|line| line.contains("【奖励】")));
        assert_eq!(stats.potions, potions_before + 1);
        assert_eq!(stats.gold, gold_before + 18);
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
        assert!(notice.title.contains("委托已领取"));
        assert!(notice.body.contains("山路余妖"));
        assert!(notice.body.contains("0/2"));
        assert!(notice.body.contains("委托追踪卡"));

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
        assert!(notice.title.contains("委托已交付"));
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
    fn npc_services_buy_potions_and_restore_at_inn() {
        let mut stats = PlayerStats::default();
        let quest = QuestLog::default();

        assert_eq!(stats.gold, 80);
        assert_eq!(
            apply_npc_service(NpcService::Shop, MapKind::Village, &quest, &mut stats),
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
            apply_npc_service(NpcService::Inn, MapKind::RiverTown, &quest, &mut stats),
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
            apply_npc_service(NpcService::Inn, MapKind::RiverTown, &quest, &mut stats),
            NpcServiceResult {
                line: "【客栈】住店要 24 文，你的钱不够。".to_string(),
                rested: false,
            }
        );
        assert_eq!(stats.hp, 1);
        assert_eq!(stats.mp, 1);

        let result = apply_npc_service(NpcService::CampRest, MapKind::Cave, &quest, &mut stats);
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

        let mut stats = PlayerStats::default();
        assert_eq!(
            apply_npc_service(NpcService::Shop, MapKind::Village, &quest, &mut stats),
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
                &river_quest,
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
    }
}
