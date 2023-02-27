use std::{cmp::Ordering, time::Duration};

use bevy::{prelude::*, reflect::erased_serde::__private::serde::__private::de};
use bevy_rapier3d::prelude::RigidBody;
use rand::Rng;

use crate::{
    buildings::buildings::AlienTarget,
    velocity::{
        collisions::{ColliderType, CollisionComponent},
        velocity::Velocity,
    },
};

const ALIEN_SPAWN_TIMER: Duration = Duration::from_millis(200);
#[derive(Resource)]
pub struct AlienSpawnTimer {
    timer: Timer,
}

pub struct AlienPlugin;
impl Plugin for AlienPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(AlienSpawnTimer {
            timer: Timer::new(ALIEN_SPAWN_TIMER, TimerMode::Repeating),
        })
        .add_system(alien_ai)
        .add_system(spawn_aliens);
    }
}

#[derive(Component, Default)]
pub struct Alien;

#[derive(Bundle)]
pub struct AlienBundle {
    alien: Alien,
    scene: SceneBundle,
    velocity: Velocity,
    collision: CollisionComponent,
    rigid_body: RigidBody,
}

impl Default for AlienBundle {
    fn default() -> Self {
        AlienBundle {
            collision: CollisionComponent {
                collider_type: ColliderType::Alien,
            },
            alien: Alien::default(),
            scene: SceneBundle::default(),
            velocity: Velocity::default(),
            rigid_body: RigidBody::Dynamic,
        }
    }
}

pub fn spawn_aliens(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<AlienSpawnTimer>,
    ass: Res<AssetServer>,
) {
    timer.timer.tick(time.delta());
    if timer.timer.finished() {
        let gltf = ass.load("spacekit_2/Models/GLTF format/alien.glb#Scene0");

        let mut rng = rand::thread_rng();
        let x: f32 = rng.gen::<f32>() * 30.0 - 15.0;
        let z: f32 = rng.gen::<f32>() * 30.0 - 15.0;

        println!("Spawning an alien at {}, {}", x, z);
        commands.spawn(AlienBundle {
            scene: {
                SceneBundle {
                    scene: gltf,
                    transform: Transform::from_xyz(x, 0.1, z), //.with_scale(Vec3::new(2.0,2.0,2.0)),
                    ..default()
                }
            },
            velocity: Velocity {
                vector: Vec3::X,
                ..default()
            },
            ..default()
        });
    }
}

pub fn alien_ai(
    mut aliens: Query<(&Transform, &mut Velocity), With<Alien>>,
    targets: Query<(&Transform, &AlienTarget), Without<Alien>>,
    _time: Res<Time>,
) {
    for mut alien in aliens.iter_mut() {
        let alien_pos = alien.0.translation;
        let target = targets.iter().min_by(|(transformA, _), (transformB, _)| {
            alien_pos
                .distance(transformA.translation)
                .total_cmp(&alien_pos.distance(transformB.translation))
        });
        if let Some((t, _)) = target {
            // println!("Alien moving towards a target at {:?}", t.translation);
            alien.1.vector = t.translation - alien.0.translation;
        }
    }
}
//
