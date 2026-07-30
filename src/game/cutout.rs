use bevy::prelude::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CutoutPart {
    Lower,
    Torso,
    Head,
}

impl CutoutPart {
    pub const ALL: [Self; 3] = [Self::Lower, Self::Torso, Self::Head];

    pub fn z_offset(self) -> f32 {
        match self {
            Self::Lower => 0.00,
            Self::Torso => 0.02,
            Self::Head => 0.04,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CutoutPartSpec {
    pub part: CutoutPart,
    pub rect: Rect,
    pub size: Vec2,
    pub offset: Vec2,
}

#[derive(Clone, Copy, Debug)]
pub struct CutoutPartMotion {
    pub offset: Vec2,
    pub scale: Vec2,
    pub rotation: f32,
    pub brightness: f32,
}

pub fn cutout_source_px_for_path(path: &str) -> f32 {
    if path.contains("frost_dragon")
        || path.contains("qilin")
        || path.contains("forest_ranger")
        || path.contains("forge_engineer")
        || path.contains("star_mage.png")
        || path.contains("ai_hero")
        || path.contains("ai_linger")
    {
        768.0
    } else if path.starts_with("creatures/") {
        256.0
    } else {
        192.0
    }
}

pub fn cutout_part_specs(source_px: f32, world_size: f32) -> [CutoutPartSpec; 3] {
    [
        part_spec(CutoutPart::Lower, source_px, world_size, 0.60, 1.00),
        part_spec(CutoutPart::Torso, source_px, world_size, 0.32, 0.76),
        part_spec(CutoutPart::Head, source_px, world_size, 0.00, 0.42),
    ]
}

fn part_spec(
    part: CutoutPart,
    source_px: f32,
    world_size: f32,
    top: f32,
    bottom: f32,
) -> CutoutPartSpec {
    let height = bottom - top;
    let center = (top + bottom) * 0.5;
    CutoutPartSpec {
        part,
        rect: Rect::new(0.0, source_px * top, source_px, source_px * bottom),
        size: Vec2::new(world_size, world_size * height),
        offset: Vec2::new(0.0, world_size * (0.5 - center)),
    }
}

pub fn cutout_part_motion(part: CutoutPart, t: f32, intensity: f32) -> CutoutPartMotion {
    let step = t.sin();
    let lift = step.abs();
    let sway = (t * 0.52).sin();
    let breath = (t * 0.74).sin();

    match part {
        CutoutPart::Lower => CutoutPartMotion {
            offset: Vec2::new(sway * 0.80 * intensity, lift * 1.15 * intensity),
            scale: Vec2::new(
                1.0 + lift * 0.010 * intensity,
                1.0 - lift * 0.012 * intensity,
            ),
            rotation: step * 0.010 * intensity,
            brightness: 1.0 + lift * 0.030 * intensity,
        },
        CutoutPart::Torso => CutoutPartMotion {
            offset: Vec2::new(sway * 1.10 * intensity, breath * 1.35 * intensity),
            scale: Vec2::new(
                1.0 + breath * 0.012 * intensity,
                1.0 + lift * 0.018 * intensity,
            ),
            rotation: sway * 0.018 * intensity,
            brightness: 1.0 + breath.max(0.0) * 0.055 * intensity,
        },
        CutoutPart::Head => CutoutPartMotion {
            offset: Vec2::new(sway * 1.42 * intensity, breath * 1.05 * intensity),
            scale: Vec2::new(
                1.0 + sway * 0.006 * intensity,
                1.0 + lift * 0.010 * intensity,
            ),
            rotation: sway * 0.030 * intensity,
            brightness: 1.0 + breath.max(0.0) * 0.070 * intensity,
        },
    }
}

pub fn brighten_color(base: Color, brightness: f32) -> Color {
    let srgba = base.to_srgba();
    Color::srgba(
        (srgba.red * brightness).clamp(0.0, 1.0),
        (srgba.green * brightness).clamp(0.0, 1.0),
        (srgba.blue * brightness).clamp(0.0, 1.0),
        srgba.alpha,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cutout_specs_cover_a_full_body_with_overlap() {
        let specs = cutout_part_specs(192.0, 64.0);

        assert_eq!(specs[0].part, CutoutPart::Lower);
        assert_eq!(specs[2].part, CutoutPart::Head);
        assert!(specs[0].rect.min.y < specs[0].rect.max.y);
        assert!(specs[0].offset.y < 0.0);
        assert!(specs[2].offset.y > 0.0);
        assert!(specs.iter().all(|spec| spec.size.x == 64.0));
    }

    #[test]
    fn cutout_part_motion_separates_body_segments() {
        let lower = cutout_part_motion(CutoutPart::Lower, 0.8, 1.0);
        let head = cutout_part_motion(CutoutPart::Head, 0.8, 1.0);

        assert_ne!(lower.rotation, head.rotation);
        assert!(head.offset.x.abs() > lower.offset.x.abs());
        assert!(head.brightness >= lower.brightness);
    }
}
