use std::{f32::consts::PI, time::Duration};

use bevy::prelude::*;
use bevy_tweening::{
    lens::{TransformPositionLens, TransformRotateXLens},
    Animator, EaseFunction, Tracks, Tween,
};

use super::{
    muzzleflash::GunType,
    relative_lenses::{RelativeTransformPositionLens, RelativeTransformRotateXLens},
};

// All the default gun animations.
// The main one is the laser speeder hovering in place up and down.
// Not actually a system, this just returns a bundle to inject into the laser_gun bundle

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
    
    // We rotate it a little as it moves up and down, so that it always points towards the ground and the aliens it's firing at.
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
