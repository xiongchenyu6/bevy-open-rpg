use bevy::prelude::*;
use bevy_paperdoll::{PaperdollAsset, PaperdollId};

use super::state::AppState;

const HERO_PAPERDOLL_PATH: &str = "paperdoll/hero.ppd";
const CLASS_WEAPON_SLOT: u32 = 10;
const ARMOR_SLOT: u32 = 20;
const CHARM_SLOT: u32 = 21;
const RELIC_SLOT: u32 = 22;

const WEAPON_SWORD: u32 = 100;
const WEAPON_STAFF: u32 = 101;
const WEAPON_HAMMER: u32 = 108;
const GEAR_VOW_PLATE: u32 = 200;
const GEAR_STARWEAVE_ROBE: u32 = 201;
const GEAR_WINDRUNNER_CLOAK: u32 = 202;
const GEAR_THUNDER_CHARM: u32 = 210;
const GEAR_SAINT_BELL: u32 = 211;
const GEAR_FORGE_GAUNTLET: u32 = 220;
const GEAR_CARROT_HALO: u32 = 222;
const STYLE_COUNT: usize = 6;

#[derive(Resource)]
pub struct PaperdollAssets {
    source: Handle<PaperdollAsset>,
}

impl PaperdollAssets {
    pub fn load(assets: &AssetServer) -> Self {
        Self {
            source: assets.load(HERO_PAPERDOLL_PATH),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum PaperdollStyle {
    Hero,
    Elder,
    Linger,
    Ranger,
    Guard,
    Mystic,
}

impl PaperdollStyle {
    const ALL: [Self; STYLE_COUNT] = [
        Self::Hero,
        Self::Elder,
        Self::Linger,
        Self::Ranger,
        Self::Guard,
        Self::Mystic,
    ];

    fn index(self) -> usize {
        match self {
            Self::Hero => 0,
            Self::Elder => 1,
            Self::Linger => 2,
            Self::Ranger => 3,
            Self::Guard => 4,
            Self::Mystic => 5,
        }
    }

    fn doll_id(self) -> u32 {
        match self {
            Self::Hero => 0,   // human body from Spirit Forge rig
            Self::Elder => 2,  // orc-tinted elder body
            Self::Linger => 1, // elf body
            Self::Ranger => 1,
            Self::Guard => 2,
            Self::Mystic => 0,
        }
    }

    fn fragments(self) -> &'static [(u32, u32)] {
        match self {
            Self::Hero => &[
                (CLASS_WEAPON_SLOT, WEAPON_SWORD),
                (ARMOR_SLOT, GEAR_STARWEAVE_ROBE),
                (CHARM_SLOT, GEAR_THUNDER_CHARM),
            ],
            Self::Elder => &[
                (CLASS_WEAPON_SLOT, WEAPON_HAMMER),
                (ARMOR_SLOT, GEAR_VOW_PLATE),
                (RELIC_SLOT, GEAR_FORGE_GAUNTLET),
            ],
            Self::Linger => &[
                (CLASS_WEAPON_SLOT, WEAPON_STAFF),
                (ARMOR_SLOT, GEAR_STARWEAVE_ROBE),
                (CHARM_SLOT, GEAR_SAINT_BELL),
                (RELIC_SLOT, GEAR_CARROT_HALO),
            ],
            Self::Ranger => &[
                (CLASS_WEAPON_SLOT, WEAPON_SWORD),
                (ARMOR_SLOT, GEAR_WINDRUNNER_CLOAK),
                (CHARM_SLOT, GEAR_THUNDER_CHARM),
            ],
            Self::Guard => &[
                (CLASS_WEAPON_SLOT, WEAPON_SWORD),
                (ARMOR_SLOT, GEAR_VOW_PLATE),
                (RELIC_SLOT, GEAR_FORGE_GAUNTLET),
            ],
            Self::Mystic => &[
                (CLASS_WEAPON_SLOT, WEAPON_STAFF),
                (ARMOR_SLOT, GEAR_WINDRUNNER_CLOAK),
                (CHARM_SLOT, GEAR_SAINT_BELL),
            ],
        }
    }
}

#[derive(Resource)]
struct PaperdollRuntime {
    images: [Option<Handle<Image>>; STYLE_COUNT],
    failed: [bool; STYLE_COUNT],
}

impl Default for PaperdollRuntime {
    fn default() -> Self {
        Self {
            images: std::array::from_fn(|_| None),
            failed: [false; STYLE_COUNT],
        }
    }
}

impl PaperdollRuntime {
    fn image(&self, style: PaperdollStyle) -> Option<Handle<Image>> {
        self.images[style.index()].clone()
    }

    fn set_image(&mut self, style: PaperdollStyle, image: Handle<Image>) {
        self.images[style.index()] = Some(image);
        self.failed[style.index()] = false;
    }
}

#[derive(Component)]
struct PaperdollSprite {
    style: PaperdollStyle,
    size: f32,
}

#[derive(Component, Clone, Copy)]
struct AppliedPaperdollStyle(PaperdollStyle);

pub const OVERWORLD_SIZE: f32 = 58.0;
pub const BATTLE_SIZE: f32 = 180.0;

pub struct PaperdollRuntimePlugin;

impl Plugin for PaperdollRuntimePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PaperdollRuntime>().add_systems(
            Update,
            (refresh_paperdoll_images, apply_paperdoll_images).chain(),
        );
    }
}

pub fn spawn_paperdoll(
    commands: &mut Commands,
    _assets: &PaperdollAssets,
    style: PaperdollStyle,
    pos: Vec3,
    size: f32,
    exit_state: AppState,
) -> Entity {
    commands
        .spawn((
            PaperdollSprite { style, size },
            Sprite::from_color(Color::srgba(1.0, 1.0, 1.0, 0.0), Vec2::splat(size)),
            Transform::from_translation(pos),
            DespawnOnExit(exit_state),
        ))
        .id()
}

fn refresh_paperdoll_images(
    assets: Res<PaperdollAssets>,
    mut runtime: ResMut<PaperdollRuntime>,
    mut paperdolls: ResMut<Assets<PaperdollAsset>>,
    mut images: ResMut<Assets<Image>>,
) {
    let Some(mut asset) = paperdolls.get_mut(&assets.source) else {
        return;
    };

    for style in PaperdollStyle::ALL {
        let idx = style.index();
        if runtime.images[idx].is_some() || runtime.failed[idx] {
            continue;
        }

        match compose_style(&mut asset, style) {
            Ok(image) => {
                runtime.set_image(style, images.add(image));
            }
            Err(err) => {
                warn!("paperdoll composition failed: {err}");
                runtime.failed[idx] = true;
            }
        }
    }
}

fn compose_style(asset: &mut PaperdollAsset, style: PaperdollStyle) -> Result<Image, String> {
    let paperdoll_id = asset.create_paperdoll(style.doll_id());
    let result = compose_style_inner(asset, paperdoll_id, style);
    asset.remove_paperdoll(paperdoll_id);
    result
}

fn compose_style_inner(
    asset: &mut PaperdollAsset,
    paperdoll_id: PaperdollId,
    style: PaperdollStyle,
) -> Result<Image, String> {
    for (slot, fragment) in style.fragments() {
        asset
            .slot_use_fragment(paperdoll_id, *slot, *fragment)
            .map_err(|err| format!("slot {slot} -> fragment {fragment}: {err}"))?;
    }

    asset
        .take_texture(paperdoll_id)
        .ok_or_else(|| "paperdoll texture was not produced".to_owned())
}

fn apply_paperdoll_images(
    mut commands: Commands,
    runtime: Res<PaperdollRuntime>,
    mut dolls: Query<(
        Entity,
        &PaperdollSprite,
        &mut Sprite,
        Option<&AppliedPaperdollStyle>,
    )>,
) {
    for (entity, doll, mut sprite, applied) in &mut dolls {
        if applied
            .map(|applied| applied.0 == doll.style)
            .unwrap_or(false)
        {
            continue;
        }
        let Some(image) = runtime.image(doll.style) else {
            continue;
        };
        sprite.image = image;
        sprite.texture_atlas = None;
        sprite.rect = None;
        sprite.color = Color::WHITE;
        sprite.custom_size = Some(Vec2::splat(doll.size));
        commands
            .entity(entity)
            .insert(AppliedPaperdollStyle(doll.style));
    }
}
