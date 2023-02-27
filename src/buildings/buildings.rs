use bevy::{
    input::{keyboard::KeyboardInput, ButtonState},
    prelude::*,
};
use bevy_rapier3d::prelude::RigidBody;

use crate::velocity::collisions::CollisionComponent;

#[derive(Component)]
struct Speeder;

#[derive(Component)]
pub struct AlienTarget {
    pub priority: i8,
}

impl Default for AlienTarget {
    fn default() -> Self {
        AlienTarget { priority: 1 }
    }
}

pub fn speeder_spawning(
    mut keybd_events: EventReader<KeyboardInput>,
    mut commands: Commands,
    query: Query<&mut Transform, With<Camera>>,
    ass: Res<AssetServer>,
) {
    for ev in keybd_events.iter() {
        if let Some(key) = ev.key_code {
            if key == KeyCode::Space && ev.state == ButtonState::Released {
                let my_gltf = ass.load("spacekit_2/Models/GLTF format/craft_speederA.glb#Scene0");
                // if let Some(cam_pos) = query.single_mut() {

                // }
                let cam_pos = query.single().translation;
                println!("Spawning a turret");
                commands.spawn((
                    Speeder,
                    AlienTarget::default(),
                    // CollisionComponent::default(),
                    RigidBody::Fixed,
                    SceneBundle {
                        scene: my_gltf,
                        transform: Transform::from_xyz(cam_pos.x, 0.1, cam_pos.z),
                        ..Default::default()
                    },
                ));
            }
        }
    }
    keybd_events.clear()
}
