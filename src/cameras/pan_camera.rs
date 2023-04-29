use bevy_kira_audio::prelude::*;
use std::f32::consts::PI;

use bevy::{
    input::{
        keyboard::KeyboardInput,
        mouse::{MouseMotion, MouseWheel},
        ButtonState,
    },
    prelude::*,
    render::primitives::Frustum,
    transform,
};
use bevy_rapier3d::na::{clamp, Quaternion};

use crate::{
    cameras::get_world_point_from_screen::get_plane_point_from_mouse_pos,
    ui::ui::{UIMode, UIState},
};

use super::utils::get_keybd_vec;

// A pan camera based on:
///https://bevy-cheatbook.github.io/cookbook/pan-orbit-camera.html
// Allows the user to move and zoom the camera using the mouse as is typical for strategy games

#[derive(Component)]
pub struct PanOrbitCamera {
    // Value between 0 and 1
    pub zoom_level: f32,
    x: f32,
}

impl Default for PanOrbitCamera {
    fn default() -> Self {
        PanOrbitCamera {
            zoom_level: 0.5,
            x: calculate_from_zoom_level(0.8).0,
        }
    }
}

// A utility.
// We need to define a trait for adding methods to external structs
trait RemoveY {
    fn remove_y(&self) -> Self;
}
impl RemoveY for Vec3 {
    fn remove_y(&self) -> Self {
        Vec3::new(self.x, 0.0, self.z)
    }
}


// Spawns a cube at the point where we click in the world. Used during testing
pub fn _camera_testing(
    mbutton: Res<Input<MouseButton>>,
    windows: Res<Windows>,
    query: Query<(&PanOrbitCamera, &Transform, &Projection)>,
    mut commands: Commands,

    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let (cam, transform, proj) = query.single();
    if mbutton.just_pressed(MouseButton::Left) {
        let m_pos = windows.get_primary().unwrap().cursor_position().unwrap();

        // println!("Mouse {:?}, {:?}", pan_delta, m_pos);
        let screen_size_half = get_primary_window_size(&windows) / 2.0;
        let vec = get_plane_point_from_mouse_pos(
            m_pos,
            screen_size_half,
            proj,
            cam.zoom_level,
            transform,
        );

        // vec.x = vec.x + (y - vec.x).abs();

        commands.spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.1 })),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(vec.x, 1.0, vec.z),
            ..default()
        });
    };
}

// The system which updates the camera position
/// Pan the camera with mouse click, zoom with scroll wheel
pub fn pan_orbit_camera(
    _windows: Res<Windows>,
    mut ev_motion: EventReader<MouseMotion>,
    mut ev_scroll: EventReader<MouseWheel>,
    mut ev_keybd: EventReader<KeyboardInput>,
    mut mouse_pos: EventReader<CursorMoved>,
    input_mouse: Res<Input<MouseButton>>,
    windows: Res<Windows>,
    mut query: Query<(&mut PanOrbitCamera, &mut Transform, &Projection)>,

    ui_state: Res<UIState>,
) {
    // change input mapping for panning here
    let pan_button = MouseButton::Left;

    let mut pan_delta = Vec2::ZERO;
    let mut m_pos: Vec2 = Vec2::ZERO;

    let horizontal_orbit: f32 = 0.0;
    let mut scroll = 0.0;
    let key_vec = get_keybd_vec(&mut ev_keybd);

    if input_mouse.pressed(pan_button) {
        // Pan only if we're not rotating at the moment
        // Pan only if UIMode panning
        if ui_state.mode == UIMode::Panning {
            for ev in ev_motion.iter() {
                pan_delta += ev.delta;
            }
            for ev in mouse_pos.iter() {
                m_pos = ev.position;
            }
        }
    }
    for ev in ev_scroll.iter() {
        scroll += ev.y;
    }

    for (mut cam, mut transform, proj) in query.iter_mut() {
        // The else makes the maths more robust, because zooming and panning at the same time creates issues
        if let Some(vec) = key_vec {
            let r = get_quaternion_y_rotation(transform.rotation);
            transform.translation += r.mul_vec3(Vec3::new(vec.x, 0.0, vec.y) * 0.3)
        } else if pan_delta.length_squared() > 0.0 && m_pos.length_squared() > 0.0 {
            let screen_size_half = get_primary_window_size(&windows) / 2.0;

            let old_mouse = m_pos - pan_delta;
            let new_mouse = m_pos;

            let mut vector = (get_plane_point_from_mouse_pos(
                old_mouse,
                screen_size_half,
                proj,
                cam.zoom_level,
                &transform,
            ) - get_plane_point_from_mouse_pos(
                new_mouse,
                screen_size_half,
                proj,
                cam.zoom_level,
                &transform,
            ));

            // Flip the vector
            vector.z *= -1.0;

            if vector.x.is_nan() || vector.y.is_nan() {
                continue;
            }
            
            // Found by experimentation. For whatever reason this seems to work.
            vector /= 1.3;

            transform.translation += vector;
        } else if horizontal_orbit.abs() > 0.0 {
            let t = transform.translation;
            let r = transform.rotation;
            transform.rotate_around(
                t + r.mul_vec3(Vec3::Z * -t.y),
                Quat::from_rotation_y(-horizontal_orbit * 0.01 * PI),
            );
        } else if scroll.abs() > 0.0 {
            cam.zoom_level += scroll * 0.01;

            cam.zoom_level = clamp(cam.zoom_level, 0.0, 1.0);

            let (x, y, tilt) = calculate_from_zoom_level(cam.zoom_level);

            // Y pos
            transform.translation.y = y;

            // Horizontal positioning
            let t = -1.0 * (transform.rotation * Vec3::Z).remove_y().normalize() * (x - cam.x);
            transform.translation += t;
            cam.x = x;

            // *Tilting
            // The horizontal angle
            let a = transform.rotation.to_euler(EulerRot::YZX).0;
            // Replace rotation
            transform.rotation = Quat::from_axis_angle(Vec3::NEG_X, tilt);
            // Add horizontal rotation to correct for the replaced val.
            transform.rotate_y(a);
        }
    }
}

const ZOOM_AT_MAX_TILT: f32 = 0.5;

const MAX_TILT: f32 = 0.3 * PI;
// Found by experimentation
const MIN_TILT: f32 = 0.185;

const MIN_HEIGHT: f32 = 1.0;
const MAX_HEIGHT: f32 = 80.0;

// Calculates the camera offset from its zoom level inbetween 0 and 1
// Returns (x, y, tilt)
pub fn calculate_from_zoom_level(zoom: f32) -> (f32, f32, f32) {
    let (x, y, tilt): (f32, f32, f32);

    if zoom < ZOOM_AT_MAX_TILT {
        tilt = ((zoom / ZOOM_AT_MAX_TILT) * (MAX_TILT - MIN_TILT)) + MIN_TILT;
    } else {
        // Max tilt
        tilt = MAX_TILT;
    }
    x = (zoom + 0.1).sqrt() * -20.0;
    y = ((zoom).powi(2) * (MAX_HEIGHT - MIN_HEIGHT)) + MIN_HEIGHT;

    return (x, y, tilt);
}

// A utility t oget the primary window size in pixels
pub fn get_primary_window_size(windows: &Res<Windows>) -> Vec2 {
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
    let p = PanOrbitCamera {
        ..Default::default()
    };
    let (x, y, tilt) = calculate_from_zoom_level(p.zoom_level);
    let transform =
        Transform::from_xyz(0., y, x+20.).with_rotation(Quat::from_axis_angle(Vec3::NEG_X, tilt));
    commands.spawn((
        Camera3dBundle {
            transform,
            projection: Projection::Perspective(PerspectiveProjection::default()),
            // camera_3d: Camera3d { clear_color: (), depth_load_op: bevy::core_pipeline::core_3d::Camera3dDepthLoadOp::Clear(()) } {},
            ..Default::default()
        },
        p,
    ));
}

// https://forum.unity.com/threads/quaternion-to-remove-pitch.822768/
fn get_quaternion_y_rotation(quaternion: Quat) -> Quat {
    let a = f32::sqrt((quaternion.w * quaternion.w) + (quaternion.y * quaternion.y));
    return Quat::from_xyzw(0.0, quaternion.y, 0.0, quaternion.w / a);
}
