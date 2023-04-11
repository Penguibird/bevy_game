use std::{f32::consts::PI, time::Duration};

use bevy::prelude::*;
use bevy_tweening::{
    lens::{TransformPositionLens, TransformRotateXLens},
    Animator, EaseFunction, Lens, Tracks, Tween,
};

use super::muzzleflash::GunType;
struct RelativeTransformPositionLens {
    previous: Vec3,
    start: Vec3,
    end: Vec3,
}

impl Lens<Transform> for RelativeTransformPositionLens {
    fn lerp(&mut self, target: &mut Transform, ratio: f32) {
        let value = self.start + (self.end - self.start) * ratio;
        target.translation += value - self.previous;
        self.previous = value;
    }
}

struct RelativeTransformRotateXLens {
    previous: f32,
    start: f32,
    end: f32,
}

impl Lens<Transform> for RelativeTransformRotateXLens {
    fn lerp(&mut self, target: &mut Transform, ratio: f32) {
        let angle = (self.end - self.start).mul_add(ratio, self.start);
        target.rotate_local_x(angle - self.previous);
        self.previous = angle;
    }
}

pub fn get_laser_gun_hover_animator() -> impl Bundle {
    let duration = Duration::from_millis(4000);

    let start = Vec3::new(0., 1., 0.);
    let end = Vec3::new(0., 1.5, 0.);
    let tween_up = Tween::new(
        EaseFunction::SineInOut,
        duration,
        RelativeTransformPositionLens {
            previous: start,
            start,
            end,
        },
    )
    .with_repeat_count(bevy_tweening::RepeatCount::Infinite)
    .with_repeat_strategy(bevy_tweening::RepeatStrategy::MirroredRepeat);

    let start = 0.;
    let end = -0.05 * PI;
    let tween_rotate_up = Tween::new(
        EaseFunction::SineInOut,
        duration,
        RelativeTransformRotateXLens {
            previous: start,
            start,
            end,
        },
    )
    .with_repeat_count(bevy_tweening::RepeatCount::Infinite)
    .with_repeat_strategy(bevy_tweening::RepeatStrategy::MirroredRepeat);

    return Animator::new(Tracks::new(vec![tween_up, tween_rotate_up]));
}
