//! Short, readable enemy rhythms and one-round break/riposte opportunities.
use super::EnemyIntent;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum EnemyStyle {
    Assault,
    Armored,
    Trickster,
    Spirit,
}

impl EnemyStyle {
    pub fn name(self) -> &'static str {
        match self {
            Self::Assault => "猛攻",
            Self::Armored => "铁甲",
            Self::Trickster => "诡袭",
            Self::Spirit => "灵术",
        }
    }
    pub fn intent(self, turn: u32) -> EnemyIntent {
        let pattern = match self {
            Self::Assault => [
                EnemyIntent::Strike,
                EnemyIntent::Charge,
                EnemyIntent::Strike,
                EnemyIntent::Strike,
            ],
            Self::Armored => [
                EnemyIntent::Brace,
                EnemyIntent::Charge,
                EnemyIntent::Strike,
                EnemyIntent::Gather,
            ],
            Self::Trickster => [
                EnemyIntent::Drain,
                EnemyIntent::Strike,
                EnemyIntent::Charge,
                EnemyIntent::Brace,
            ],
            Self::Spirit => [
                EnemyIntent::Drain,
                EnemyIntent::Gather,
                EnemyIntent::Charge,
                EnemyIntent::Strike,
            ],
        };
        pattern[(turn as usize) % pattern.len()]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum StrikeKind {
    Sword,
    Spell,
    Breaker,
    Burst,
}

#[derive(Default, Clone, Copy, Debug)]
pub(super) struct Hit {
    pub damage: i32,
    pub broke: bool,
    pub counter: bool,
    pub exposed: bool,
    pub resisted: bool,
}

#[derive(Clone, Debug)]
pub(super) struct Tactics {
    pub style: EnemyStyle,
    pub pressure: u8,
    pub limit: u8,
    pub broken: bool,
    pub exposed: bool,
    pub riposte: bool,
    pub last_hit: Hit,
    pub breaks: u32,
    pub counters: u32,
    pub interrupts: u32,
    pub notice: String,
}

impl Tactics {
    pub fn new(style: EnemyStyle, boss: bool) -> Self {
        let limit = if boss {
            8
        } else if style == EnemyStyle::Armored {
            7
        } else {
            6
        };
        Self {
            style,
            pressure: 0,
            limit,
            broken: false,
            exposed: false,
            riposte: false,
            last_hit: Hit::default(),
            breaks: 0,
            counters: 0,
            interrupts: 0,
            notice: String::new(),
        }
    }

    pub fn hit(&mut self, raw: i32, kind: StrikeKind, intent: EnemyIntent) -> Hit {
        let counter = std::mem::take(&mut self.riposte);
        let exposed = std::mem::take(&mut self.exposed);
        let resisted =
            intent == EnemyIntent::Brace && kind == StrikeKind::Sword && !counter && !exposed;
        let damage = if counter || exposed {
            (raw * 3 / 2).max(1)
        } else if resisted {
            (raw / 2).max(1)
        } else {
            raw
        };
        let pressure = if exposed {
            0
        } else if resisted {
            0
        } else {
            (match kind {
                StrikeKind::Sword => 2,
                StrikeKind::Spell => 1,
                StrikeKind::Breaker => 2,
                StrikeKind::Burst => 3,
            }) + if counter { 3 } else { 0 }
        };
        self.pressure = self.pressure.saturating_add(pressure).min(self.limit);
        let broke = self.pressure >= self.limit && !self.broken;
        if broke {
            self.broken = true;
            self.breaks += 1;
        }
        self.notice = if broke {
            "【破势】架势崩解，打断下一次行动；下一击伤害 +50%！".into()
        } else if counter {
            "【反击】御守借势，这一击伤害 +50%，额外施压 3。".into()
        } else if exposed {
            "【追击】抓住破绽，这一击伤害 +50%。".into()
        } else if resisted {
            "【铁甲】普攻被架开；用仙术破甲或等待出手窗口。".into()
        } else {
            String::new()
        };
        self.last_hit = Hit {
            damage,
            broke,
            counter,
            exposed,
            resisted,
        };
        self.last_hit
    }

    pub fn wait(&mut self) {
        // A player can spend the free opening healing, but cannot bank bonuses forever.
        self.riposte = false;
        self.exposed = false;
        self.last_hit = Hit::default();
        self.notice.clear();
    }

    pub fn cancel_action(&mut self, intent: EnemyIntent) -> bool {
        if !self.broken {
            return false;
        }
        self.broken = false;
        self.pressure = 0;
        self.exposed = true;
        if matches!(
            intent,
            EnemyIntent::Charge | EnemyIntent::Heavy | EnemyIntent::Special
        ) {
            self.interrupts += 1;
        }
        true
    }

    pub fn successful_guard(&mut self, intent: EnemyIntent) {
        if matches!(intent, EnemyIntent::Heavy | EnemyIntent::Special) {
            self.riposte = true;
            self.counters += 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn brace_resists_swords_but_not_breaking_magic() {
        let mut t = Tactics::new(EnemyStyle::Armored, false);
        assert_eq!(t.hit(20, StrikeKind::Sword, EnemyIntent::Brace).damage, 10);
        assert_eq!(t.pressure, 0);
        assert_eq!(
            t.hit(20, StrikeKind::Breaker, EnemyIntent::Brace).damage,
            20
        );
        assert_eq!(t.pressure, 2);
    }
    #[test]
    fn break_cancels_once_and_followup_cannot_chain_breaks() {
        let mut t = Tactics::new(EnemyStyle::Assault, false);
        for _ in 0..3 {
            t.hit(10, StrikeKind::Sword, EnemyIntent::Strike);
        }
        assert!(t.cancel_action(EnemyIntent::Heavy));
        assert!(!t.cancel_action(EnemyIntent::Heavy));
        assert_eq!(t.hit(10, StrikeKind::Sword, EnemyIntent::Strike).damage, 15);
        assert_eq!(t.pressure, 0);
        assert_eq!(t.hit(10, StrikeKind::Sword, EnemyIntent::Strike).damage, 10);
        assert_eq!(t.pressure, 2);
    }
    #[test]
    fn heavy_guard_creates_one_counter_and_waiting_expires_it() {
        let mut t = Tactics::new(EnemyStyle::Assault, false);
        t.successful_guard(EnemyIntent::Strike);
        assert!(!t.riposte);
        t.successful_guard(EnemyIntent::Heavy);
        assert_eq!(t.hit(20, StrikeKind::Sword, EnemyIntent::Strike).damage, 30);
        assert_eq!(t.pressure, 5);
        assert!(!t.riposte);
        t.successful_guard(EnemyIntent::Special);
        t.wait();
        assert!(!t.riposte);
    }
    #[test]
    fn enemy_styles_have_different_repeatable_rhythms() {
        for style in [
            EnemyStyle::Assault,
            EnemyStyle::Armored,
            EnemyStyle::Trickster,
            EnemyStyle::Spirit,
        ] {
            for turn in 0..12 {
                assert_eq!(style.intent(turn), style.intent(turn + 4));
            }
        }
        assert_ne!(EnemyStyle::Assault.intent(0), EnemyStyle::Armored.intent(0));
    }
}
