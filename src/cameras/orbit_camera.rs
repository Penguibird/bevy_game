use std::f32::consts::PI;

use bevy::{
    input::{
        keyboard::KeyboardInput,
        mouse::{MouseMotion, MouseWheel},
        ButtonState,
    },
    prelude::*,
    transform,
};

use super::utils::get_keybd_vec;

///https://bevy-cheatbook.github.io/cookbook/pan-orbit-camera.html
/// Tags an entity as capable of panning and orbiting.
#[derive(Component)]
pub struct PanOrbitCamera;

impl Default for PanOrbitCamera {
    fn default() -> Self {
        PanOrbitCamera {}
    }
}

const CAMERA_PIVOT_HEIHGT: f32 = 30.0;


/// Pan the camera with middle mouse click, zoom with scroll wheel, orbit with right mouse click.
pub fn pan_orbit_camera(
    _windows: Res<Windows>,
    mut ev_motion: EventReader<MouseMotion>,
    mut ev_scroll: EventReader<MouseWheel>,
    mut ev_keybd: EventReader<KeyboardInput>,
    input_mouse: Res<Input<MouseButton>>,
    mut query: Query<(&mut PanOrbitCamera, &mut Transform, &Projection)>,
) {
    // change input mapping for orbit and panning here
    let orbit_button = MouseButton::Right;
    let pan_button = MouseButton::Left;

    let mut pan = Vec2::ZERO;
    // let mut rotation_move = Vec2::ZERO;
    let mut horizontal_orbit = 0.0;
    let mut scroll = 0.0;
    let key_vec = get_keybd_vec(&mut ev_keybd);

    // if input_mouse.pressed(orbit_button) {
    //     for ev in ev_motion.iter() {
    //         // rotation_move += ev.delta;
    //         horizontal_orbit += ev.delta.x;
    //     }
    // } else if input_mouse.pressed(pan_button) {
    //     // Pan only if we're not rotating at the moment
    //     for ev in ev_motion.iter() {
    //         pan += ev.delta;
    //     }
    // }
    // for ev in ev_scroll.iter() {
    //     scroll += ev.y;
    // }

    // for (_, mut transform, _projection) in query.iter_mut() {
    //     // * Im putting else if here cause it was in the original implementation that I rewrote. Idk what it was for, but im not questioning it
    //     if let Some(vec) = key_vec {
    //         let r = get_quaternion_y_rotation(transform.rotation);
    //         transform.translation += r.mul_vec3(Vec3::new(vec.x, 0.0, vec.y) * 0.3)
    //     } else if pan.length_squared() > 0.0 {
    //         // TODO Improve
    //         transform.translation += Vec3::new(pan.x, 0.0, pan.y) * -0.05;
    //     } else if horizontal_orbit.abs() > 0.0 {
    //         // TODO Improve
    //         let t = transform.translation;
    //         let r = transform.rotation;
    //         transform.rotate_around(
    //             t + r.mul_vec3(Vec3::Z * -t.y),
    //             Quat::from_rotation_y(-horizontal_orbit * 0.01 * PI),
    //         );
    //     } else if scroll.abs() > 0.0 {
    //         if transform.translation.y <= CAMERA_PIVOT_HEIHGT {
    //             let r = transform.rotation;
    //             let mut center_of_rot = transform.translation + r.mul_vec3(Vec3::Z * -30.0);
    //             center_of_rot.y = CAMERA_PIVOT_HEIHGT;
    //             let normal_to_plane_of_rotation = r.mul_vec3(Vec3::Z).cross(Vec3::Y).normalize();

    //             // TODO Put min here
    //             transform.rotate_around(
    //                 center_of_rot,
    //                 Quat::from_axis_angle(normal_to_plane_of_rotation, -scroll * 0.01 * PI),
    //             )
    //         } else if transform.translation.y <= 50.0 {
    //             transform.translation.y += -scroll;
    //         }

    //         transform.translation.y = f32::max(transform.translation.y, 5.0);
    //         transform.translation.y = f32::min(transform.translation.y, 50.0);
    //     }
    // }
}

fn _get_primary_window_size(windows: &Res<Windows>) -> Vec2 {
    let window = windows.get_primary();
    if let None = window {
        return Vec2::ZERO;
    }
    let window = window.unwrap();
    let window = Vec2::new(window.width() as f32, window.height() as f32);
    window
}

/// Spawn a camera like this
pub fn spawn_camera(mut commands: Commands) {
    let translation = Vec3::new(-2.0, 10.5, 5.0);

    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(translation)
                .looking_at(Vec3::new(translation.x, 0.0, translation.z), Vec3::Z),
            ..Default::default()
        },
        PanOrbitCamera {
            ..Default::default()
        },
    ));
}

// https://forum.unity.com/threads/quaternion-to-remove-pitch.822768/
fn get_quaternion_y_rotation(quaternion: Quat) -> Quat {
    let a = f32::sqrt((quaternion.w * quaternion.w) + (quaternion.y * quaternion.y));
    return Quat::from_xyzw(0.0, quaternion.y, 0.0, quaternion.w / a);
}
