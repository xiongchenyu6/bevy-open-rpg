//! Small, readable route riddles. Observation precedes a real player decision.
use super::RunState;
use super::event::{DialogueSource, RunDialogue};
use crate::game::explore::MapKind;
use crate::game::{PlayerStats, Rng};

pub struct Riddle {
    pub title: &'static str,
    pub clues: &'static [&'static str],
    pub options: [&'static str; 3],
    pub answer: usize,
    pub hint: &'static str,
    pub solved: &'static str,
}

pub fn riddle(map: MapKind) -> Riddle {
    match map {
        MapKind::MoonEchoCorridor => Riddle {
            title: "月洞晶石",
            clues: &[
                "三枚晶石分别映着暗月、半月与满月。",
                "门上的刻痕写着：月从无到有，水从静到明。",
            ],
            options: [
                "暗月 → 半月 → 满月",
                "满月 → 半月 → 暗月",
                "半月 → 暗月 → 满月",
            ],
            answer: 0,
            hint: "月光应当渐渐变亮。先从最暗的晶石开始。",
            solved: "月光由暗转明，晶石之间接成一条完整的水线。",
        },
        MapKind::RiverReedBed => Riddle {
            title: "逆水河灯",
            clues: &[
                "三盏河灯里，只有中间一盏把影子投向上游。",
                "船夫说：别把所有灯都转过来，只替走反的那一盏认路。",
            ],
            options: ["转动三盏灯", "只扶正中间的灯", "吹灭两边的灯"],
            answer: 1,
            hint: "观察影子的方向。只有中间那盏逆着水流。",
            solved: "中间的灯影转回下游，三点微光连出安全的渡路。",
        },
        MapKind::PlagueShrinePath => Riddle {
            title: "荒寺瘴铃",
            clues: &[
                "铃绳旁留着老医的布条：先断来处，再护人息。",
                "井边的铃通往瘴根，门边的铃护住病者。",
            ],
            options: ["先门铃，后井铃", "两铃一起敲", "先井铃，后门铃"],
            answer: 2,
            hint: "先止住瘴气源头，才不会把它吹向病者。",
            solved: "井底的瘴气收回根须，门铃随后替病者撑起一圈清光。",
        },
        MapKind::MansionMirrorGallery => Riddle {
            title: "照影镜轴",
            clues: &[
                "三面镜子映着同一支蜡烛，左右两面的火焰都随你呼吸摇动。",
                "正中的火焰却一动不动。林月衡说：照出来的应是眼前之物，不是藏起的画。",
            ],
            options: ["转动左镜", "转动正中那面镜", "转动右镜"],
            answer: 1,
            hint: "真实的烛火会随气流摆动。静止的火焰暴露了假镜。",
            solved: "正中的假影碎开，镜轴后露出通向内院的石阶。",
        },
        MapKind::ThunderDrumPath => Riddle {
            title: "风云誓鼓",
            clues: &[
                "鼓面刻着：风起云聚，云聚雷生，雷过誓成。",
                "南瑶按住鼓槌：旧誓不能抢在雷声前。",
            ],
            options: [
                "风鼓 → 云鼓 → 誓鼓",
                "誓鼓 → 风鼓 → 云鼓",
                "云鼓 → 誓鼓 → 风鼓",
            ],
            answer: 0,
            hint: "从风起开始，让云聚起之后再敲誓鼓。",
            solved: "三声鼓响顺着雷路传开，守鼓人的旧名重新亮起。",
        },
        _ => Riddle {
            title: "旧梦魂灯",
            clues: &[
                "一盏灯映着正在流水的渡口，一盏映着翻动的书页，还有一盏映着没有呼吸的你。",
                "灵儿说：回忆会变，只有水影想把人永远留在原处。",
            ],
            options: ["熄掉渡口的灯", "熄掉书页的灯", "熄掉无息倒影的灯"],
            answer: 2,
            hint: "留意哪个影子没有生命的动静。",
            solved: "没有呼吸的倒影沉入水中，其余两盏灯为活着的人照出前路。",
        },
    }
}

pub fn open(dialogue: &mut RunDialogue, index: usize, map: MapKind, rng: &mut Rng) {
    let data = riddle(map);
    let rotation = rng.range(0, 2) as usize;
    let mut options = data.options.to_vec();
    options.rotate_left(rotation);
    let answer = (data.answer + 3 - rotation) % 3;
    let mut lines = data.clues.iter().map(|s| s.to_string()).collect::<Vec<_>>();
    lines.push(format!(
        "{}\n根据线索选择；判断错了可以重试，也可以先退开。",
        data.clues.join(" ")
    ));
    options.push("先退开，再观察一会儿");
    *dialogue = RunDialogue {
        active: true,
        title: format!("机关 · {}", data.title),
        lines,
        options: options.into_iter().map(str::to_string).collect(),
        source: DialogueSource::Puzzle { index },
        puzzle_answer: Some(answer),
        puzzle_hint: Some(data.hint),
        puzzle_solved_line: Some(data.solved),
        ..Default::default()
    };
}

pub fn choice_preview(_index: usize, pick: usize) -> String {
    if pick == 3 {
        "先退开不会损失资源；机关仍可再次查看。".into()
    } else {
        "看清场景线索后再判断。错选不损失资源，也不会领取奖励。".into()
    }
}

pub fn resolve_choice(
    dialogue: &mut RunDialogue,
    pick: usize,
    _stats: &mut PlayerStats,
    run: &mut RunState,
    _rng: &mut Rng,
) {
    if dialogue.resolved {
        return;
    }
    let DialogueSource::Puzzle { index } = dialogue.source else {
        return;
    };
    if pick == 3 {
        dialogue.active = false;
        dialogue.options.clear();
        return;
    }
    if Some(pick) != dialogue.puzzle_answer {
        run.puzzle_mistakes += 1;
        dialogue.choice_error = Some(format!(
            "光纹没有接上。{}",
            dialogue.puzzle_hint.unwrap_or("再留意线索。")
        ));
        return;
    }
    dialogue.choice_error = None;
    dialogue.puzzle_cleared_after = Some(index);
    dialogue
        .lines
        .push("机关应着你的判断亮起，挡路的光纹终于稳住。".into());
    dialogue.resolved = true;
    dialogue.options.clear();
    dialogue.idx += 1;
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn wrong_choice_and_retreat_never_claim_completion() {
        let mut rng = Rng(42);
        let mut run = RunState::new(&mut rng);
        let mut stats = PlayerStats::default();
        let mut dialogue = RunDialogue::default();
        open(&mut dialogue, 4, MapKind::RiverReedBed, &mut rng);
        let answer = dialogue.puzzle_answer.unwrap();
        resolve_choice(
            &mut dialogue,
            (answer + 1) % 3,
            &mut stats,
            &mut run,
            &mut rng,
        );
        assert!(dialogue.active && !dialogue.resolved);
        assert!(dialogue.puzzle_cleared_after.is_none());
        assert_eq!(run.route_puzzles_solved, 0);
        assert_eq!(run.puzzle_mistakes, 1);
        assert!(dialogue.choice_preview(Some(&run)).contains("光纹没有接上"));
        resolve_choice(&mut dialogue, 3, &mut stats, &mut run, &mut rng);
        assert!(!dialogue.active);
        assert!(dialogue.puzzle_cleared_after.is_none());
    }
    #[test]
    fn clues_and_solutions_work_for_all_maps_and_rotated_choices() {
        for map in [
            MapKind::MoonEchoCorridor,
            MapKind::RiverReedBed,
            MapKind::PlagueShrinePath,
            MapKind::MansionMirrorGallery,
            MapKind::ThunderDrumPath,
            MapKind::DreamWaterway,
        ] {
            let mut positions = std::collections::HashSet::new();
            for seed in 1..=32 {
                let mut rng = Rng(0x9e3779b97f4a7c15u64.wrapping_mul(seed));
                let mut run = RunState::new(&mut rng);
                let mut stats = PlayerStats::default();
                let mut d = RunDialogue::default();
                open(&mut d, 2, map, &mut rng);
                d.idx = d.lines.len() - 1;
                let answer = d.puzzle_answer.unwrap();
                positions.insert(answer);
                assert_eq!(d.options[answer], riddle(map).options[riddle(map).answer]);
                resolve_choice(&mut d, answer, &mut stats, &mut run, &mut rng);
                assert!(d.resolved);
                assert_eq!(d.puzzle_cleared_after, Some(2));
            }
            assert_eq!(positions.len(), 3);
        }
    }
}
