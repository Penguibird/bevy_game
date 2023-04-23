use std::f32::consts::PI;

use bevy::{input::mouse::MouseButtonInput, prelude::*};

use crate::cameras::pan_camera::calculate_from_zoom_level;

use super::pan_camera::{get_primary_window_size, PanOrbitCamera};

// Converts a click on screen to a coordinate on the plane

// A utility event which is emited every time the user clicks on a square on the plane
#[derive(Clone, Copy, Debug)]
pub struct WorldClickEvent {
    pub point: Vec3,
    // The original mouse event, containing info about the button clicked etc.
    pub mouse_event: MouseButtonInput,
}

// read all the inputs from the mouse
// convert them into world positions
// emit the world click event
pub fn emit_world_click_events(
    windows: Res<Windows>,
    mut query_set: ParamSet<(Query<(&PanOrbitCamera, &Transform, &Projection)>,)>,
    mut mouse_click: EventReader<MouseButtonInput>,
    mut ev_writer: EventWriter<WorldClickEvent>,
) {
    // These are all the variables we need to calculate the world position
    let cam_query = query_set.p0();
    let (cam, transform, proj) = cam_query.single();
    let screen_size_half = get_primary_window_size(&windows) / 2.0;
    let m_pos = windows
        .get_primary()
        .unwrap()
        .cursor_position()
        .unwrap_or(Vec2 { x: 0.0, y: 0.0 });
    let point =
        get_plane_point_from_mouse_pos(m_pos, screen_size_half, proj, cam.zoom_level, transform);

    // Emit the event
    for ev in mouse_click.iter() {
        ev_writer.send(WorldClickEvent {
            point,
            mouse_event: *ev,
        })
    }
}

// The utility function which uses a bunch of trigonometry to calculate the world point from the screen click

pub fn get_plane_point_from_mouse_pos(
    // Value in pixels on screen
    mouse: Vec2,
    // Value in pixels on screen
    screen_size_half: Vec2,
    proj: &Projection,
    // camera zoom level, ranging from 0 to 1
    zoom_level: f32,
    // Value in game units
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

    // A camera utility to get the displacement based on the current zoom level
    let (_, y, tilt) = calculate_from_zoom_level(zoom_level);

    // The angle between the camera and a normal to the plane
    let get_beta_from_px = |px: f32, screen_half: f32, fov_half: f32| {
        let beta = f32::atan((screen_half - px) / (screen_half / fov_half.tan()));
        return beta;
    };
    // The x and z angles.
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
    let mut x_correction = x_beta.tan() * a;

    // Derived by experimentation - no clue why it works
    x_correction /= 1.2;

    vec = Quat::from_axis_angle(Vec3::Y, transform.rotation.to_euler(EulerRot::YXZ).0) * vec * -1.0;

    vec.x = x_correction;

    // Get the absolute point from relative to camera position
    vec -= transform.translation;

    // The game has no height actually, so we don't care about the y component
    vec.y = 0.0;

    // The vector result is actually flipped
    vec *= -1.0;

    return vec;
}
