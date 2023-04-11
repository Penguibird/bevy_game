use std::f32::consts::PI;

use bevy::prelude::*;

use crate::cameras::orbit_camera::calculate_from_zoom_level;


pub fn get_plane_point_from_mouse_pos(
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
    // dbg!(x_beta);
    let mut x_correction = x_beta.tan() * a;

    // Derived by experimentation - no clue why it works
    x_correction /= 1.2;
    // dbg!(x_correction);

    vec = Quat::from_axis_angle(Vec3::Y, transform.rotation.to_euler(EulerRot::YXZ).0) * vec * -1.0;

    vec.x = x_correction;
    // vec.x *= -1.0;

    vec -= transform.translation;
    vec.y = 0.0;

    vec *= -1.0;

    return vec;
}