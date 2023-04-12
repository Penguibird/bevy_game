use std::{cmp::Ordering, f32::consts::PI, time::Duration, };

use bevy::prelude::*;
use bevy_rapier3d::prelude::{Collider, CollisionGroups, Group, LockedAxes, RigidBody, Velocity, Friction};
use rand::Rng;

use crate::{
    buildings::{defensive_buildings::{AlienTarget, DamageDealing, TargetSelecting}, grid::Grid},
    health::health::{DeathEvent, Health},
};
const ALIEN_SPEED: f32 = 5.;
const ALIEN_SPAWN_TIMER: Duration = Duration::from_millis(1000);

#[derive(Resource)]
pub struct AlienCount {
    pub count: u32,
}

impl Default for AlienCount {
    fn default() -> Self {
        AlienCount { count: 0 }
    }
}

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
        .init_resource::<AlienCount>()
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
    mut count: ResMut<AlienCount>,
    grid: Res<Grid>,
) {
    timer.timer.tick(time.delta());
    if timer.timer.finished() {
        let gltf = ass.load("spacekit_2/Models/GLTF format/alien.glb#Scene0");

        // let mesh: &Mesh =
        //     Assets::get(Assets, &ass.load("spacekit_2/Models/GLTF format/alien.glb#Scene0")).unwrap();
        let mut rng = rand::thread_rng();
        let angle: f32 = rng.gen::<f32>() * 2. * PI;

        dbg!(&grid);
        
        let mut x = grid.base_center.x + grid.center_radius * f32::cos(angle);
        let mut z = grid.base_center.z + grid.center_radius * f32::sin(angle);
        
        x += rng.gen::<f32>() * 1.;
        z += rng.gen::<f32>() * 1.;


        println!("Spawning an alien at {}, {}", x, z);
        count.count += 1;
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
                Collider::cylinder(0.4, 0.3),
                Friction::default(),
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
                    transform: Transform::from_xyz(-2.0, -1.0, -1.5),
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
                linvel: (t.translation - alien.0.translation).normalize() * ALIEN_SPEED,
                angvel: Vec3::ZERO,
            };
            alien.3.target = Some(e);
        }
    }
}

pub fn alien_death(
    mut aliens: Query<(&mut Transform, &mut Alien, &mut Velocity, &mut DamageDealing)>,
    mut ev: EventReader<DeathEvent>,
    mut count: ResMut<AlienCount>,
) {
    for e in ev.iter() {
        if let Ok((mut t, mut a, mut vel, mut dmg)) = aliens.get_mut(e.entity) {
            if a.alive {
                count.count -= 1;
                let r = t.rotation;
                t.rotate_axis(r.mul_vec3(Vec3::X), PI / 2.0);
                vel.linvel = Vec3::splat(0.);
                dmg.damage = 0;
                a.alive = false;
            }
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
            commands.entity(x.1).despawn_recursive();
        }
    }
}
