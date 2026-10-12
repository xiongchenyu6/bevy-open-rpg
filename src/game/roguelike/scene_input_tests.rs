//! CPU-only integration checks for Intent → real dialogue/map settlement.
use super::*;
use crate::game::roguelike::event;
use bevy::state::app::StatesPlugin;

fn app(marker: Option<NodeKind>) -> App {
    let mut app = App::new();
    app.add_plugins(StatesPlugin).init_state::<AppState>();
    let mut map = MapData::build(MapKind::MoonEchoCorridor);
    map.tiles = vec![vec![Tile::Path; MAP_W as usize]; MAP_H as usize];
    map.tiles[2][5] = Tile::Portal;
    let scene = RunSceneState {
        map: MapKind::MoonEchoCorridor,
        tiles: map.tiles.clone(),
        revealed: vec![vec![true; MAP_W as usize]; MAP_H as usize],
        col: 2,
        row: 2,
        facing_left: false,
        markers: marker
            .map(|kind| {
                vec![SceneMarker {
                    kind,
                    col: 3,
                    row: 2,
                    cleared: false,
                }]
            })
            .unwrap_or_default(),
        portals: vec![(5, 2)],
        flow: Vec::new(),
        hazards: Vec::new(),
        intro_shown: true,
        cooldown: 0.0,
    };
    let mut rng = Rng(42);
    let mut run = RunState::new(&mut rng);
    run.accept_journey_task();
    app.insert_resource(scene)
        .insert_resource(RunSceneMap(map))
        .insert_resource(run)
        .insert_resource(rng)
        .insert_resource(Time::<()>::default())
        .insert_resource(GameFont(Handle::default()))
        .insert_resource(PlayerStats::default())
        .insert_resource(Intent::default())
        .insert_resource(InventoryOpen::default())
        .insert_resource(RunDialogue::default())
        .add_systems(
            Update,
            (event::run_dialogue_input, run_scene_movement).chain(),
        );
    app
}

#[test]
fn walking_over_story_does_not_trigger_until_confirm_then_real_choice_records_reward() {
    let mut app = app(Some(NodeKind::Story));
    app.world_mut().resource_mut::<Intent>().move_dir = Some(IVec2::new(1, 0));
    app.update();
    assert_eq!(app.world().resource::<RunSceneState>().col, 3);
    assert!(!app.world().resource::<RunDialogue>().active);
    assert!(!app.world().resource::<RunSceneState>().markers[0].cleared);
    app.world_mut().resource_mut::<Intent>().confirm = true;
    app.update();
    assert!(app.world().resource::<RunDialogue>().active);
    assert!(app.world().resource::<RunSceneState>().markers[0].cleared);
    assert!(app.world().resource::<RunState>().narrative_log.is_empty());
    let last = app.world().resource::<RunDialogue>().lines.len() - 1;
    app.world_mut().resource_mut::<RunDialogue>().idx = last;
    app.world_mut().resource_mut::<Intent>().confirm = true;
    app.update();
    assert_eq!(app.world().resource::<RunState>().narrative_log.len(), 1);
    assert!(app.world().resource::<RunDialogue>().resolved);
}

#[test]
fn wrong_puzzle_retreat_and_retry_settle_marker_and_commission_once_only_on_correct_answer() {
    let mut app = app(Some(NodeKind::Puzzle));
    app.world_mut()
        .resource_mut::<RunState>()
        .accept_route_commission(NodeKind::Puzzle);
    let before = app.world().resource::<PlayerStats>().max_mp;
    app.world_mut().resource_mut::<Intent>().confirm = true;
    app.update();
    let answer = app.world().resource::<RunDialogue>().puzzle_answer.unwrap();
    let last = app.world().resource::<RunDialogue>().lines.len() - 1;
    {
        let mut dialogue = app.world_mut().resource_mut::<RunDialogue>();
        dialogue.idx = last;
        dialogue.selected = (answer + 1) % 3;
    }
    app.world_mut().resource_mut::<Intent>().confirm = true;
    app.update();
    assert!(!app.world().resource::<RunSceneState>().markers[0].cleared);
    assert_eq!(app.world().resource::<RunState>().puzzle_mistakes, 1);
    assert_eq!(app.world().resource::<RunState>().route_puzzles_solved, 0);
    assert_eq!(app.world().resource::<RunState>().route_contacts_helped, 0);
    assert_eq!(app.world().resource::<PlayerStats>().max_mp, before);
    app.world_mut().resource_mut::<RunDialogue>().selected = 3;
    app.world_mut().resource_mut::<Intent>().confirm = true;
    app.update();
    assert!(!app.world().resource::<RunDialogue>().active);
    assert!(!app.world().resource::<RunSceneState>().markers[0].cleared);
    app.world_mut().resource_mut::<Intent>().confirm = true;
    app.update();
    let answer = app.world().resource::<RunDialogue>().puzzle_answer.unwrap();
    let last = app.world().resource::<RunDialogue>().lines.len() - 1;
    {
        let mut dialogue = app.world_mut().resource_mut::<RunDialogue>();
        dialogue.idx = last;
        dialogue.selected = answer;
    }
    app.world_mut().resource_mut::<Intent>().confirm = true;
    app.update();
    assert!(app.world().resource::<RunSceneState>().markers[0].cleared);
    assert_eq!(app.world().resource::<RunState>().route_puzzles_solved, 1);
    assert_eq!(app.world().resource::<RunState>().route_contacts_helped, 1);
    assert_eq!(app.world().resource::<PlayerStats>().max_mp, before + 3);
    assert!(
        app.world()
            .resource::<RunDialogue>()
            .pending_commission
            .is_none()
    );
    app.update();
    assert_eq!(app.world().resource::<RunState>().route_puzzles_solved, 1);
    assert_eq!(app.world().resource::<RunState>().route_contacts_helped, 1);
}

#[test]
fn portal_moves_directly_to_next_stage_without_dialogue_and_ignores_optional_guide() {
    let mut app = app(Some(NodeKind::Guide));
    app.world_mut().resource_mut::<RunSceneState>().col = 4;
    app.world_mut().resource_mut::<Intent>().move_dir = Some(IVec2::new(1, 0));
    app.update();
    assert_eq!(app.world().resource::<RunState>().stage, 1);
    assert_eq!(
        app.world()
            .resource::<RunState>()
            .completed_journey_tasks
            .len(),
        1
    );
    assert!(!app.world().resource::<RunDialogue>().active);
    assert!(matches!(
        app.world().resource::<NextState<AppState>>(),
        NextState::Pending(AppState::NodeMap)
    ));
}

#[test]
fn portal_refuses_unfinished_mandatory_story_without_advancing_or_opening_paperwork() {
    let mut app = app(Some(NodeKind::Story));
    app.world_mut().resource_mut::<RunSceneState>().col = 4;
    app.world_mut().resource_mut::<Intent>().move_dir = Some(IVec2::new(1, 0));
    app.update();
    assert_eq!(app.world().resource::<RunState>().stage, 0);
    assert!(!app.world().resource::<RunDialogue>().active);
    assert_eq!(
        app.world()
            .resource::<RunState>()
            .completed_journey_tasks
            .len(),
        0
    );
    assert_eq!(
        app.world_mut()
            .query_filtered::<Entity, With<SceneFloatText>>()
            .iter(app.world())
            .count(),
        1
    );
}
