use bevy::{
    input::{keyboard::KeyboardInput, mouse::MouseWheel},
    prelude::*,
};

use super::utils::get_keybd_vec;

#[derive(Component)]
pub struct TopDownCamera;

pub fn spawn_camera(mut commands: Commands) {
    let translation = Vec3::new(0.0, 10.0, 0.0);
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_translation(translation)
                .looking_at(Vec3::new(translation.x, 0.0, translation.z), Vec3::NEG_Z),
            ..Default::default()
        },
        TopDownCamera,
    ));
}

pub fn move_camera(
    mut cam_query: Query<&mut Transform, With<TopDownCamera>>,
    mut keybd_events: EventReader<KeyboardInput>,
    mut mouse_scroll: EventReader<MouseWheel>,
    // mut mouse_clicks: EventReader<MouseButton>,
) {
    let keybd_vec = get_keybd_vec(&mut keybd_events);
    // keybd_events.clear();

    if let Some(vec) = keybd_vec {
        for mut transform in cam_query.iter_mut() {
            transform.translation += Vec3::new(vec.x, 0.0, vec.y);
        }
    }

    let mut scroll_amomunt = 0.0;
    for e in mouse_scroll.iter() {
        scroll_amomunt += e.y;
    }

    for mut transform in cam_query.iter_mut() {
        if transform.translation.y >= 5.0 && transform.translation.y <= 200.0 {
            transform.translation += Vec3::new(0.0, -scroll_amomunt, 0.0);
            transform.translation.y = f32::clamp(transform.translation.y, 5.0, 200.0)
        }
    }
}
