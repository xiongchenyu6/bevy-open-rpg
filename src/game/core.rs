use bevy::prelude::*;

// ---------------------------------------------------------------------------
// World geometry constants
// ---------------------------------------------------------------------------

pub const TILE: f32 = 40.0;
pub const MAP_W: i32 = 30;
pub const MAP_H: i32 = 16;

/// Convert a tile coordinate (col, row) into a centered world position.
/// Row 0 is the top of the map.
pub fn tile_to_world(col: i32, row: i32) -> Vec2 {
    let x = (col as f32 - (MAP_W - 1) as f32 / 2.0) * TILE;
    let y = ((MAP_H - 1) as f32 / 2.0 - row as f32) * TILE;
    Vec2::new(x, y)
}

// ---------------------------------------------------------------------------
// Shared font handle
// ---------------------------------------------------------------------------

/// Loaded once at startup. Unifont covers CJK glyphs (the embedded default font
/// is Latin-only), so all in-game Chinese text uses this handle.
#[derive(Resource)]
pub struct GameFont(pub Handle<Font>);

impl GameFont {
    pub fn text_font(&self, size: f32) -> TextFont {
        TextFont {
            font: self.0.clone().into(),
            font_size: size.into(),
            ..default()
        }
    }
}

// ---------------------------------------------------------------------------
// Input intent — decouples game logic from the raw input source so the desktop
// build (keyboard) and the capture build (scripted timeline) can drive the same
// systems deterministically.
// ---------------------------------------------------------------------------

#[derive(Resource, Default)]
pub struct Intent {
    /// Currently-held movement direction (overworld), if any.
    pub move_dir: Option<IVec2>,
    /// Edge: confirm / talk / advance-dialogue (one frame only).
    pub confirm: bool,
    /// Edge: menu cursor up.
    pub up: bool,
    /// Edge: menu cursor down.
    pub down: bool,
}

impl Intent {
    pub fn clear(&mut self) {
        self.move_dir = None;
        self.confirm = false;
        self.up = false;
        self.down = false;
    }
}

/// Probability a step into grass triggers a battle. The capture build raises
/// this to make encounters deterministic for the proof video.
#[derive(Resource)]
pub struct EncounterRate(pub f32);

impl Default for EncounterRate {
    fn default() -> Self {
        Self(0.16)
    }
}

// ---------------------------------------------------------------------------
// Persistent player progression (survives state transitions)
// ---------------------------------------------------------------------------

#[derive(Resource)]
pub struct PlayerStats {
    pub name: String,
    pub level: u32,
    pub hp: i32,
    pub max_hp: i32,
    pub mp: i32,
    pub max_mp: i32,
    pub atk: i32,
    pub def: i32,
    pub potions: u32,
    pub exp: u32,
    pub gold: u32,
}

impl Default for PlayerStats {
    fn default() -> Self {
        Self {
            name: "李逍遥".into(),
            level: 1,
            hp: 80,
            max_hp: 80,
            mp: 20,
            max_mp: 20,
            atk: 16,
            def: 6,
            potions: 3,
            exp: 0,
            gold: 80,
        }
    }
}

impl PlayerStats {
    pub const SPELL_COST: i32 = 5;
    pub const POTION_HEAL: i32 = 40;

    pub fn exp_to_next(&self) -> u32 {
        self.level * 20
    }

    /// Apply exp and return the number of levels gained.
    pub fn gain_exp(&mut self, amount: u32) -> u32 {
        self.exp += amount;
        let mut gained = 0;
        while self.exp >= self.exp_to_next() {
            self.exp -= self.exp_to_next();
            self.level += 1;
            self.max_hp += 15;
            self.max_mp += 4;
            self.atk += 3;
            self.def += 1;
            self.hp = self.max_hp;
            self.mp = self.max_mp;
            gained += 1;
        }
        gained
    }

    pub fn full_restore(&mut self) {
        self.hp = self.max_hp;
        self.mp = self.max_mp;
    }

    pub fn spend_gold(&mut self, amount: u32) -> bool {
        if self.gold < amount {
            return false;
        }

        self.gold -= amount;
        true
    }
}

// ---------------------------------------------------------------------------
// Tiny deterministic RNG (avoids pulling in the `rand` crate)
// ---------------------------------------------------------------------------

#[derive(Resource)]
pub struct Rng(pub u64);

impl Default for Rng {
    fn default() -> Self {
        // Any non-zero seed works for xorshift.
        Self(0x9E3779B97F4A7C15)
    }
}

impl Rng {
    fn next_u64(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    /// Uniform float in [0, 1).
    pub fn unit(&mut self) -> f32 {
        (self.next_u64() >> 40) as f32 / (1u64 << 24) as f32
    }

    pub fn chance(&mut self, p: f32) -> bool {
        self.unit() < p
    }

    /// Inclusive integer range.
    pub fn range(&mut self, lo: i32, hi: i32) -> i32 {
        if hi <= lo {
            return lo;
        }
        lo + (self.next_u64() % ((hi - lo + 1) as u64)) as i32
    }
}
