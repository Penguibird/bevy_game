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

use super::utils::get_keybd_vec;

///https://bevy-cheatbook.github.io/cookbook/pan-orbit-camera.html
/// Tags an entity as capable of panning and orbiting.
#[derive(Component)]
pub struct PanOrbitCamera {
    zoom_level: f32,
    x: f32,
}

impl Default for PanOrbitCamera {
    fn default() -> Self {
        PanOrbitCamera {
            zoom_level: 0.8,
            x: calculate_from_zoom_level(0.8).0,
        }
    }
}

trait RemoveY {
    fn remove_y(&self) -> Self;
}
impl RemoveY for Vec3 {
    fn remove_y(&self) -> Self {
        Vec3::new(self.x, 0.0, self.z)
    }
}

const CAMERA_PIVOT_HEIHGT: f32 = 30.0;

fn get_plane_point_from_mouse_pos(
    mouse: Vec2,
    screen_size_half: Vec2,
    proj: &Projection,
    zoom_level: f32,
    transform: &Transform,
) -> Vec3 {
    // println!("Screen size, {:?}", screen_size_half);

    let fov = (PI / 4.0) / 2.0;
    let mut fov_vec = Vec2::new(fov * 1.0, fov);
    let mut near = 0.1;
    if let Projection::Perspective(projection) = proj {
        fov_vec = Vec2::new(projection.fov * projection.aspect_ratio, projection.fov);
        near = projection.near;
    }

    let fov_vec_half = fov_vec / 2.0;

    let (_, y, tilt) = calculate_from_zoom_level(zoom_level);
    let get_beta_from_px = |px: f32, screen_half: f32, fov_half: f32| {
        let mut beta = f32::atan((screen_half - px) / (screen_half / fov_half.tan()));

        // beta = f32::atan((screen_half-px) / (near));

        // beta *= -1.0;
        // * This one treats it as a circle
        // (This one seems wrong)
        // let beta = px * (fov_half / screen_half);
        // debug!(tilt);

        // let res = (((PI / 2.0) - tilt) * -1.0 + beta).tan() * y;
        // dbg!(tilt, (PI/2.0) - tilt)
        return beta;
    };
    let x_beta = get_beta_from_px(mouse.x, screen_size_half.x, fov_vec_half.x);
    let z_beta = get_beta_from_px(mouse.y, screen_size_half.y, fov_vec_half.y);

    let get_length = |beta: f32| (beta).tan() * y;
    let mut vec = Vec3::new(
        get_length(x_beta),
        0.0,
        get_length(((PI / 2.0) - tilt) * -1.0 + z_beta),
    );

    // x correction
    // a is the length from the camera to the line on which the point lies.
    let a = f32::sqrt(vec.z.powi(2) + y.powi(2)) - near;
    dbg!(x_beta);
    let mut x_correction = x_beta.tan() * a;

    // Derived by experimentation - no clue why it works
    x_correction /= 1.2;
    dbg!(x_correction);

    vec = Quat::from_axis_angle(Vec3::Y, transform.rotation.to_euler(EulerRot::YXZ).0) * vec * -1.0;

    vec.x = x_correction;
    // vec.x *= -1.0;

    vec -= transform.translation;
    vec.y = 0.0;

    vec *= -1.0;

    return vec;
}

pub fn camera_testing(
    mbutton: Res<Input<MouseButton>>,
    windows: Res<Windows>,
    mut query: Query<(&mut PanOrbitCamera, &mut Transform, &Projection)>,
    mut commands: Commands,

    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let (cam, transform, proj) = query.single();
    if mbutton.just_pressed(MouseButton::Left) {
        let m_pos = windows.get_primary().unwrap().cursor_position().unwrap();

        // println!("Mouse {:?}, {:?}", pan_delta, m_pos);
        let screen_size_half = get_primary_window_size(&windows) / 2.0;
        let mut vec = get_plane_point_from_mouse_pos(
            m_pos,
            screen_size_half,
            proj,
            cam.zoom_level,
            transform,
        );

        // vec.x = vec.x + (y - vec.x).abs();
        dbg!(vec);

        commands.spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.1 })),
            material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
            transform: Transform::from_xyz(vec.x, 1.0, vec.z),
            ..default()
        });
    };
}

/// Pan the camera with middle mouse click, zoom with scroll wheel, orbit with right mouse click.
pub fn pan_orbit_camera(
    _windows: Res<Windows>,
    mut ev_motion: EventReader<MouseMotion>,
    mut ev_scroll: EventReader<MouseWheel>,
    mut ev_keybd: EventReader<KeyboardInput>,
    mut mouse_pos: EventReader<CursorMoved>,
    input_mouse: Res<Input<MouseButton>>,
    windows: Res<Windows>,
    mut query: Query<(&mut PanOrbitCamera, &mut Transform, &Projection)>,
    mut commands: Commands,

    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // change input mapping for orbit and panning here
    let orbit_button = MouseButton::Right;
    let pan_button = MouseButton::Left;

    let mut pan_delta = Vec2::ZERO;
    let mut m_pos: Vec2 = Vec2::ZERO;

    // let mut rotation_move = Vec2::ZERO;
    let mut horizontal_orbit: f32 = 0.0;
    let mut scroll = 0.0;
    let key_vec = get_keybd_vec(&mut ev_keybd);

    if input_mouse.pressed(orbit_button) {
        for ev in ev_motion.iter() {
            // rotation_move += ev.delta;
            // horizontal_orbit += ev.delta.x;
        }
    } else if input_mouse.pressed(pan_button) {
        // Pan only if we're not rotating at the moment


        // dbg!(ev_motion.iter().collect::<Vec<_>>());
        for ev in ev_motion.iter() {
            pan_delta += ev.delta;
        }
        // dbg!(mouse_pos.iter().collect::<Vec<_>>());
        for ev in mouse_pos.iter() {
            m_pos = ev.position;
        }
    }
    mouse_pos.clear();
    ev_motion.clear();

    // m_pos = windows.get_primary().unwrap().cursor_position().unwrap();
    for ev in ev_scroll.iter() {
        scroll += ev.y;
    }

    for (mut cam, mut transform, proj) in query.iter_mut() {
        // * Im putting else if here cause it was in the original implementation that I rewrote. Idk what it was for, but im not questioning it
        if let Some(vec) = key_vec {
            let r = get_quaternion_y_rotation(transform.rotation);
            transform.translation += r.mul_vec3(Vec3::new(vec.x, 0.0, vec.y) * 0.3)
        } else if pan_delta.length_squared() > 0.0 && m_pos.length_squared() > 0.0 {
            // println!("Mouse {:?}, {:?}", pan_delta, m_pos);
            let screen_size_half = get_primary_window_size(&windows) / 2.0;
            // println!("Screen size, {:?}", screen_size_half);

            let old_mouse = m_pos - pan_delta;
            let new_mouse = m_pos;
            // dbg!(m_pos, pan_delta, old_mouse);

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

            vector.z *= -1.0;

            if vector.x.is_nan() || vector.y.is_nan() {
                continue;
            }
            // let vector = Quat::from_axis_angle(Vec3::Y, transform.rotation.to_euler(EulerRot::YXZ).0)
            //     * vector;

            // println!("Translating by {:?}", vector);
            vector /= 1.2;

            transform.translation += vector;
            // (proj as PerspectiveProjection)
        } else if horizontal_orbit.abs() > 0.0 {
            // TODO Improve
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
            // TODO Replace vecZ with vec z times from_axis_angle
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

            // println!(" x: {}, zoomlevel: {}", x, cam.zoom_level);
            println!(
                "Camera at {:?}, x: {}, y: {}, tilt: {}, zoomlevel: {}",
                transform.translation, x, y, tilt, cam.zoom_level
            );
        }
    }
}

const ZOOM_AT_MAX_TILT: f32 = 0.5;

const MAX_TILT: f32 = 0.3 * PI;
// Found by experimentation
const MIN_TILT: f32 = 0.185;

const MIN_HEIGHT: f32 = 1.0;
const MAX_HEIGHT: f32 = 80.0;

fn calculate_from_zoom_level(zoom: f32) -> (f32, f32, f32) {
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

fn get_primary_window_size(windows: &Res<Windows>) -> Vec2 {
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

    let mut transform = Transform::from_translation(translation);
    transform.rotate_axis(Vec3::X, (1.0 / 3.0) * -PI);

    commands.spawn((
        Camera3dBundle {
            transform,
            projection: Projection::Perspective(PerspectiveProjection {
                // near: f32::EPSILON,
                ..Default::default()
            }),
            // camera_3d: Camera3d { clear_color: (), depth_load_op: bevy::core_pipeline::core_3d::Camera3dDepthLoadOp::Clear(()) } {},
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
