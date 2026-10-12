//! Exercise production choice resolution, not a second reward model.
use super::super::narrative::{Cost, NarrativeKind, apply_effect};
use super::super::{
    Relic,
    skill::{self, SkillId},
};
use super::*;

fn setup() -> (RunDialogue, PlayerStats, RunState, Rng) {
    let mut rng = Rng(42);
    let run = RunState::new(&mut rng);
    let stats = PlayerStats {
        hp: 40,
        mp: 2,
        gold: 100,
        potions: 3,
        ..default()
    };
    (RunDialogue::default(), stats, run, rng)
}

#[test]
fn narrative_paid_options_charge_on_success_and_failure() {
    // All authored monetary contracts, including chess stake and story aid.
    let contracts = [
        (0, 0, 30),
        (2, 1, 25),
        (4, 0, 50),
        (6, 0, 40),
        (7, 0, 20),
        (10, 0, 10),
        (11, 0, 10),
        (12, 0, 30),
        (content::EV_FOX3_COLD, 1, 40),
        (content::EV_QIN1, 0, 20),
    ];
    for (index, pick, fee) in contracts {
        assert_eq!(content::EVENTS[index].options[pick].cost, Cost::Gold(fee));
        for seed in 1..=64 {
            let (mut d, mut stats, mut run, _) = setup();
            let mut rng = Rng(seed);
            let mut predictor = Rng(seed);
            let option = &content::EVENTS[index].options[pick];
            let outcome = if predictor.chance(option.chance) {
                option.success
            } else {
                option.failure.unwrap_or(option.success)
            };
            let gain = match outcome.effect {
                Effect::GainGold(n) => n,
                _ => 0,
            };
            d.open_event(index);
            resolve_choice(&mut d, pick, &mut stats, &mut run, &mut rng);
            assert!(d.resolved);
            assert_eq!(stats.gold, 100 - fee + gain, "event {index}, seed {seed}");
            assert!(
                d.lines
                    .iter()
                    .any(|line| line.contains(&format!("钱财 -{fee} 文")))
            );
        }
    }
}

#[test]
fn narrative_unaffordable_choices_do_not_change_progress_or_rng() {
    for (index, pick) in [(0, 0), (content::EV_FOX1, 0), (content::EV_QIN1, 0)] {
        let (mut d, mut stats, mut run, mut rng) = setup();
        stats.gold = 0;
        stats.potions = 0;
        d.open_event(index);
        let before = rng.0;
        resolve_choice(&mut d, pick, &mut stats, &mut run, &mut rng);
        assert!(!d.resolved);
        assert!(d.choice_error.is_some());
        assert_eq!(rng.0, before);
        assert_eq!(stats.gold, 0);
        assert_eq!(stats.potions, 0);
        assert_eq!(run.fox_stage, 0);
        assert_eq!(run.qin_stage, 0);
        assert!(run.narrative_log.is_empty());
        assert_eq!((run.qingyuan, run.daoxin), (0, 0));
        // The free alternative remains usable after rejection.
        resolve_choice(&mut d, 1, &mut stats, &mut run, &mut rng);
        assert!(d.resolved);
        assert!(d.choice_error.is_none());
    }
}

#[test]
fn narrative_fox_second_chance_changes_final_encounter_and_receipt() {
    let (mut d, mut stats, mut run, mut rng) = setup();
    d.open_event(content::EV_FOX1);
    resolve_choice(&mut d, 1, &mut stats, &mut run, &mut rng);
    run.chapter = 2;
    assert_eq!(run.draw_chain_or_event(&mut rng), content::EV_FOX2_COLD);
    d.open_event(content::EV_FOX2_COLD);
    let before = stats.potions;
    resolve_choice(&mut d, 0, &mut stats, &mut run, &mut rng);
    assert_eq!(stats.potions, before - 1);
    assert_eq!(run.fox_kind, 2);
    run.chapter = 5;
    assert_eq!(run.draw_chain_or_event(&mut rng), content::EV_FOX3_WARM);
    d.open_event(content::EV_FOX3_WARM);
    resolve_choice(&mut d, 0, &mut stats, &mut run, &mut rng);
    assert!(run.has_relic(Relic::BreathSoil));
    assert_eq!(run.fox_stage, 3);
    assert!(
        run.side_story_echoes(true)
            .iter()
            .any(|s| s.contains("补救"))
    );
}

#[test]
fn narrative_qin_patron_and_listener_receive_different_reunion_skills() {
    for (first, expected) in [(0, SkillId::SoulLantern), (1, SkillId::MoonHalo)] {
        let (mut d, mut stats, mut run, mut rng) = setup();
        run.chapter = 2;
        d.open_event(content::EV_QIN1);
        resolve_choice(&mut d, first, &mut stats, &mut run, &mut rng);
        assert_eq!(stats.gold, if first == 0 { 80 } else { 100 });
        run.chapter = 6;
        assert_eq!(run.draw_chain_or_event(&mut rng), content::EV_QIN2);
        d.open_event(content::EV_QIN2);
        assert!(
            d.choice_preview(Some(&run))
                .contains(skill::skill(expected).name)
        );
        resolve_choice(&mut d, 0, &mut stats, &mut run, &mut rng);
        assert!(run.skills.contains(&expected));
        assert_eq!(run.qin_stage, 2);
        assert!(
            run.side_story_echoes(false)
                .iter()
                .any(|s| s.contains("尚未让它应验"))
        );
        assert!(
            run.side_story_echoes(true)
                .iter()
                .any(|s| s.contains("不再只是祝愿"))
        );
    }
}

#[test]
fn narrative_replayed_scene_and_event_never_pay_twice_or_charge_twice() {
    let (mut d, mut stats, mut run, mut rng) = setup();
    d.open_story(0, 0);
    resolve_choice(&mut d, 0, &mut stats, &mut run, &mut rng);
    let first = (stats.max_hp, stats.potions, run.qingyuan, rng.0);
    d.open_story(0, 0);
    resolve_choice(&mut d, 1, &mut stats, &mut run, &mut rng);
    assert_eq!((stats.max_hp, stats.potions, run.qingyuan, rng.0), first);
    d.open_event(content::EV_QIN1);
    resolve_choice(&mut d, 0, &mut stats, &mut run, &mut rng);
    let first = (stats.gold, stats.max_mp, run.qingyuan, rng.0);
    d.open_event(content::EV_QIN1);
    resolve_choice(&mut d, 0, &mut stats, &mut run, &mut rng);
    assert_eq!((stats.gold, stats.max_mp, run.qingyuan, rng.0), first);
    assert_eq!(run.narrative_log.len(), 2);
    assert!(run.has_narrative_receipt(NarrativeKind::Story(0, 0)));
}

#[test]
fn narrative_receipts_report_real_capped_recovery_and_duplicate_fallbacks() {
    let (_, mut stats, mut run, mut rng) = setup();
    stats.hp = stats.max_hp;
    stats.mp = stats.max_mp;
    assert_eq!(
        apply_effect(Effect::HealPct(30), &mut stats, &mut run, &mut rng).unwrap(),
        "气血 +0"
    );
    assert_eq!(
        apply_effect(Effect::RestoreMpPct(30), &mut stats, &mut run, &mut rng).unwrap(),
        "灵力 +0"
    );
    run.skills.push(SkillId::MoonHalo);
    let mp = stats.max_mp;
    apply_effect(
        Effect::LearnSkill(SkillId::MoonHalo),
        &mut stats,
        &mut run,
        &mut rng,
    );
    assert_eq!(stats.max_mp, mp + 2);
    assert_eq!(run.skills.len(), 1);
    apply_effect(
        Effect::Relic(Relic::SpiritPendant),
        &mut stats,
        &mut run,
        &mut rng,
    );
    let gold = stats.gold;
    let line = apply_effect(
        Effect::Relic(Relic::SpiritPendant),
        &mut stats,
        &mut run,
        &mut rng,
    )
    .unwrap();
    assert_eq!(stats.gold, gold + 40);
    assert_eq!(run.relics.len(), 1);
    assert!(line.contains("40 文"));
}

#[test]
fn narrative_blood_cost_requires_health_and_never_grants_free_attack() {
    let (mut d, mut stats, mut run, mut rng) = setup();
    stats.hp = 1;
    d.open_event(15);
    let atk = stats.atk;
    resolve_choice(&mut d, 0, &mut stats, &mut run, &mut rng);
    assert_eq!(stats.atk, atk);
    assert_eq!(stats.hp, 1);
    assert!(!d.resolved);
    stats.hp = stats.max_hp;
    resolve_choice(&mut d, 0, &mut stats, &mut run, &mut rng);
    assert_eq!(stats.hp, stats.max_hp - (stats.max_hp * 12 / 100).max(1));
    assert_eq!(stats.atk, atk + 2);
}

#[test]
fn narrative_local_commission_waits_for_affordable_resolved_choice() {
    let (mut d, mut stats, mut run, mut rng) = setup();
    stats.gold = 0;
    run.accept_route_commission(NodeKind::Event);
    d.open_event(0);
    d.pending_commission = Some((MapKind::RiverTown, NodeKind::Event));
    resolve_choice(&mut d, 0, &mut stats, &mut run, &mut rng);
    assert!(!run.is_route_commission_completed());
    assert!(d.pending_commission.is_some());
    resolve_choice(&mut d, 1, &mut stats, &mut run, &mut rng);
    assert!(run.is_route_commission_completed());
    assert_eq!(stats.gold, 25);
    assert_eq!(run.route_contacts_helped, 1);
}

#[test]
fn narrative_market_commission_completes_only_after_goodbye() {
    let (mut d, mut stats, mut run, mut rng) = setup();
    run.accept_route_commission(NodeKind::Market);
    d.open_market();
    d.pending_commission = Some((MapKind::Capital, NodeKind::Market));
    resolve_choice(&mut d, 0, &mut stats, &mut run, &mut rng);
    assert!(!run.is_route_commission_completed());
    assert_eq!(stats.gold, 60);
    resolve_choice(&mut d, 3, &mut stats, &mut run, &mut rng);
    assert!(run.is_route_commission_completed());
    assert_eq!(stats.gold, 95);
    assert!(d.resolved);
}

#[test]
fn narrative_chapter_seeds_keep_random_events_in_their_geography() {
    for seed in 1..=64 {
        let mut rng = Rng(seed);
        let mut run = RunState::new(&mut rng);
        for chapter in 0..super::super::CHAPTER_COUNT {
            run.chapter = chapter;
            for stage in 0..run.stage_count() {
                run.stage = stage;
                for _ in 0..30 {
                    let index = run.draw_event(&mut rng);
                    assert!(narrative::event_eligible(index, run.journey_map()));
                    if matches!(run.journey_map(), MapKind::Cave | MapKind::MoonEchoCorridor) {
                        assert_ne!(index, 20, "capital notice in a cave");
                        assert_ne!(index, 10, "river crossing in a cave");
                    }
                }
            }
        }
    }
}

#[test]
#[ignore = "writes every authored choice contract and real receipt for review"]
fn narrative_reward_audit() {
    let Some(path) = std::env::var_os("NARRATIVE_REWARD_OUT") else {
        return;
    };
    let mut rows = vec!["kind,chapter,title,choice,cost,preview,receipt".to_string()];
    let quote = |s: String| format!("\"{}\"", s.replace('"', "\"\"").replace('\n', " / "));
    for chapter in 0..super::super::CHAPTER_COUNT {
        for roll in 0..content::story_count(chapter) {
            for pick in 0..2 {
                let (mut d, mut stats, mut run, mut rng) = setup();
                run.chapter = chapter;
                d.open_story(chapter, roll);
                d.selected = pick;
                let preview = d.choice_preview(Some(&run));
                let option = content::pick_story(chapter, roll).options[pick];
                resolve_choice(&mut d, pick, &mut stats, &mut run, &mut rng);
                assert!(d.resolved);
                let receipt = run.narrative_log.last().unwrap();
                rows.push(format!(
                    "story,{},{},{},{},{},{}",
                    chapter + 1,
                    quote(receipt.title.clone()),
                    quote(receipt.choice.clone()),
                    quote(option.cost.label()),
                    quote(preview),
                    quote(receipt.receipt.clone())
                ));
            }
        }
    }
    for index in 0..content::EVENTS.len() {
        for pick in 0..2 {
            let (mut d, mut stats, mut run, mut rng) = setup();
            if index == content::EV_QIN2 {
                run.qin_kind = 1;
            }
            d.open_event(index);
            d.selected = pick;
            let preview = d.choice_preview(Some(&run));
            resolve_choice(&mut d, pick, &mut stats, &mut run, &mut rng);
            assert!(d.resolved);
            let receipt = run.narrative_log.last().unwrap();
            rows.push(format!(
                "event,1,{},{},{},{},{}",
                quote(receipt.title.clone()),
                quote(receipt.choice.clone()),
                quote(content::EVENTS[index].options[pick].cost.label()),
                quote(preview),
                quote(receipt.receipt.clone())
            ));
        }
    }
    std::fs::write(path, rows.join("\n") + "\n").unwrap();
    eprintln!(
        "narrative_reward_audit: {} reviewed choices",
        rows.len() - 1
    );
}
