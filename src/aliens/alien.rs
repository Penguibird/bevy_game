use std::{cmp::Ordering, time::Duration};

use bevy::prelude::*;
use bevy_rapier3d::prelude::{Collider, CollisionGroups, Group, LockedAxes, RigidBody, Velocity};
use rand::Rng;

use crate::{
    buildings::buildings::{AlienTarget, DamageDealing, TargetSelecting},
    health::health::{DeathEvent, Health},
};

const ALIEN_SPAWN_TIMER: Duration = Duration::from_millis(20000);

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
        .add_system(spawn_aliens)
        .add_system(alien_cleanup)
        .add_system(alien_death);
    }
}

#[derive(Component)]
pub struct Alien {
    pub alive: bool,
}

impl Default for Alien {
    fn default() -> Self {
        Alien { alive: true }
    }
}

// #[derive(Bundle)]
// pub struct AlienBundle {
//     alien: Alien,
//     scene: SceneBundle,
//     velocity: Velocity,
//     body: RigidBody,
// }

// impl Default for AlienBundle {
//     fn default() -> Self {
//         AlienBundle {
//             alien: Alien::default(),
// scene: SceneBundle::default(),
//             body: RigidBody::Dynamic,
//             velocity: Velocity::default(),
//         }
//     }
// }

pub fn spawn_aliens(
    mut commands: Commands,
    time: Res<Time>,
    mut timer: ResMut<AlienSpawnTimer>,
    ass: Res<AssetServer>,
) {
    timer.timer.tick(time.delta());
    if timer.timer.finished() {
        let gltf = ass.load("spacekit_2/Models/GLTF format/alien.glb#Scene0");

        // let mesh: &Mesh =
        //     Assets::get(Assets, &ass.load("spacekit_2/Models/GLTF format/alien.glb#Scene0")).unwrap();
        let mut rng = rand::thread_rng();
        let x: f32 = rng.gen::<f32>() * 30.0 - 15.0;
        let z: f32 = rng.gen::<f32>() * 30.0 - 15.0;

        println!("Spawning an alien at {}, {}", x, z);
        commands
            .spawn((
                Alien::default(),
                RigidBody::Dynamic,
                Health::new(50),
                LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z,
                SpatialBundle {
                    transform: Transform::from_xyz(x, 0.0, z), //.with_scale(Vec3::new(2.0,2.0,2.0)),
                    ..default()
                },
                Collider::cylinder(1.0, 0.3),
                CollisionGroups::new(Group::GROUP_10, Group::GROUP_1),
                Velocity { ..default() },
                TargetSelecting {
                    range: 1.0,
                    target: None,
                },
                DamageDealing {
                    cooldown: Timer::from_seconds(0.5, TimerMode::Repeating),
                    damage: 2,
                },
            ))
            .with_children(|c| {
                c.spawn((SceneBundle {
                    scene: gltf,
                    transform: Transform::from_xyz(-2.0, 0.0, -1.5),
                    ..default()
                },));
            });
    }
}

pub fn alien_ai(
    mut aliens: Query<(&Transform, &mut Velocity, &Alien, &mut TargetSelecting)>,
    targets: Query<(&Transform, &AlienTarget, Entity), Without<Alien>>,
    _time: Res<Time>,
) {
    for mut alien in aliens.iter_mut() {
        if !alien.2.alive {
            continue;
        }

        let alien_pos = alien.0.translation;
        let target = targets
            .iter()
            .min_by(|(transform_a, _, _), (transform_b, _, _)| {
                alien_pos
                    .distance(transform_a.translation)
                    .total_cmp(&alien_pos.distance(transform_b.translation))
            });
        if let Some((t, _, e)) = target {
            // println!("Alien moving towards a target at {:?}", t.translation);
            *alien.1 = Velocity {
                linvel: t.translation - alien.0.translation,
                angvel: Vec3::ZERO,
            };
            alien.3.target = Some(e);
        }
    }
}

pub fn alien_death(
    mut aliens: Query<(&mut Transform, &mut Alien)>,
    mut ev: EventReader<DeathEvent>,
) {
    for e in ev.iter() {
        if let Ok((mut t, mut a)) = aliens.get_mut(e.entity) {
            let r = t.rotation;
            t.rotate_axis(r.mul_vec3(Vec3::X), 90.0);
            a.alive = false;
        }
    }
}

pub fn alien_cleanup(
    time: Res<Time>,
    mut query: Query<(&mut Health, Entity), With<Alien>>,
    mut commands: Commands,
) {
    for mut x in query.iter_mut() {
        x.0.dead_for_timer.tick(time.delta());
        if x.0.dead_for_timer.finished() {
            commands.entity(x.1).despawn();
        }
    }
}
