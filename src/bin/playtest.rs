//! Ordinary windowed GamePlugin with reproducible review fixtures.
//! Run inside nix develop: cargo run --bin playtest -- [opening|capital|final-battle].
//! Input is normal DesktopInputPlugin; no automatic player or hidden-state controls.
use bevy::prelude::*;
use love_rpg::{
    DesktopInputPlugin, GamePlugin,
    game::{
        AppState, PlayerStats, Rng,
        battle::PendingEncounter,
        roguelike::{
            CHAPTER_CLEAR_REWARDS, FightRank, JOURNEY_MAPS, RunState, battle_mods_for, content,
            encounter_kind_for, graph::NodeKind, narrative, skill::SkillId,
        },
    },
};

fn main() {
    let preset = std::env::args().nth(1).unwrap_or_else(|| "opening".into());
    assert!(
        matches!(preset.as_str(), "opening" | "capital" | "final-battle"),
        "unknown review fixture"
    );
    let mut app = App::new();
    app.add_plugins(
        DefaultPlugins
            .set(WindowPlugin {
                primary_window: Some(Window {
                    title: "御剑行 · 轮回".into(),
                    resolution: (1280u32, 720u32).into(),
                    ..default()
                }),
                ..default()
            })
            .set(AssetPlugin {
                meta_check: bevy::asset::AssetMetaCheck::Never,
                ..default()
            }),
    )
    .add_plugins(GamePlugin)
    .add_plugins(DesktopInputPlugin);
    if preset != "opening" {
        let chapter = if preset == "capital" { 4 } else { 6 };
        let mut rng = Rng(0x28_2026_5EED);
        let mut run = RunState::new(&mut rng);
        run.skills = vec![SkillId::SwordGleam];
        let mut stats = PlayerStats::default();
        stats.atk += 2;
        stats.def += 1;
        stats.potions += 1;
        // A conservative checkpoint: authentic story effects and chapter growth,
        // excluding ordinary relic drops and extra optional quest rewards.
        for (act, clear) in CHAPTER_CLEAR_REWARDS.iter().enumerate().take(chapter) {
            run.chapter = act;
            narrative::apply_outcome(
                &content::pick_story(act, 0).options[act % 2].outcome,
                &mut stats,
                &mut run,
                &mut rng,
            );
            stats.max_hp += clear.hp;
            stats.max_mp += clear.mp;
            stats.atk += clear.atk;
            stats.def += clear.def;
            run.chapter_seals.push(act);
        }
        run.chapter = chapter;
        run.boss = run.chapter_def().bosses[0];
        run.fights_won = chapter as u32 * 4;
        run.card_shown = true;
        if preset == "final-battle" {
            narrative::apply_outcome(
                &content::pick_story(chapter, 0).options[0].outcome,
                &mut stats,
                &mut run,
                &mut rng,
            );
            run.stage = JOURNEY_MAPS[chapter].len() - 1;
            run.current_fight = Some(FightRank::Boss);
            app.insert_resource(battle_mods_for(&run, FightRank::Boss));
            app.insert_resource(PendingEncounter {
                zone: run.chapter_def().zones[0],
                kind: encounter_kind_for(&run, NodeKind::Boss),
            });
            app.world_mut()
                .resource_mut::<NextState<AppState>>()
                .set(AppState::Battle);
        } else {
            app.world_mut()
                .resource_mut::<NextState<AppState>>()
                .set(AppState::NodeMap);
        }
        stats.hp = stats.max_hp;
        stats.mp = stats.max_mp;
        app.insert_resource(run)
            .insert_resource(stats)
            .insert_resource(rng);
    }
    app.run();
}
