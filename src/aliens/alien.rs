use bevy::{prelude::*, time::Stopwatch};
use bevy_rapier3d::prelude::{
    Collider, CollisionGroups, Friction, Group, LockedAxes, RigidBody, Velocity,
};
use rand::Rng;
use std::{cmp::Ordering, f32::consts::PI, time::Duration};

use crate::{
    audio::audio::AudioType,
    buildings::{
        defensive_buildings::{AlienTarget, DamageDealing, TargetSelecting},
        grid::Grid,
    },
    game_timer::game_timer::InGameTime,
    health::health::{DeathEvent, Health},
    AppStage, AppState,
};
const ALIEN_SPEED: f32 = 5.;

// The period at the start of the game where aliens don't spawn
const GRACE_PERIOD: Duration = Duration::from_secs(10);
#[derive(Resource)]
pub struct AlienCount {
    pub count: u32,
}

impl Default for AlienCount {
    fn default() -> Self {
        AlienCount { count: 0 }
    }
}
pub struct AlienPlugin;
impl Plugin for AlienPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AlienCount>()
            .insert_resource(AlienModel(None))
            .add_system_set(
                SystemSet::on_enter(AppState::InGame)
                    .label(AppStage::RegisterResources)
                    .with_system(register_aliens),
            )
            .init_resource::<AlienSpawnAngle>()
            .add_system_set(
                SystemSet::on_update(AppState::InGame)
                    .with_system(alien_ai)
                    .with_system(spawn_aliens)
                    .with_system(alien_spawning_randomize_angle)
                    .with_system(alien_cleanup)
                    .with_system(alien_death),
            );
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

#[derive(Resource, Clone)]
pub struct AlienModel(Option<Handle<Scene>>);

pub fn register_aliens(ass: Res<AssetServer>, mut res: ResMut<AlienModel>) {
    let gltf = ass.load("spacekit_2/Models/GLTF format/alien.glb#Scene0");
    res.0 = gltf.into();
}

// Returns a modulo-based function which spawns enemies in waves.
// More details in the TDD
pub fn get_probability_to_spawn_an_alien(
    t: Duration,
    building_count: u32,
    alien_count: u32,
) -> f32 {
    // FOR TESTING
    let t = t + Duration::from_secs(30);

    if t < GRACE_PERIOD {
        return 0.;
    } else {
        let mut secs = t.as_millis() as f32 / (1000.);
        secs -= GRACE_PERIOD.as_secs_f32();
        // let mut res = ((f32::sin(x.powf(1.3) / 2.)) + 0.3 + (x / 50.).powf(2.4)) * (x / 15.);
        let x = secs;

        let period = 60_f32;
        let pow = 3;
        let mut sawtooth = (x % period).powi(pow);

        // Clamp to 1
        sawtooth /= (period.powi(pow));

        // Ramp up to full power going on to 10 minutes;
        let final_wave_i = 10_f32;
        let mut res = sawtooth * (x / (final_wave_i * period));

        // Add the building modifier - at a 100 buildings double the number of aliens spawned.
        res *= ((building_count as f32) / 100.) + 1.;

        return res;
    }
}
// A quick test to print the probabilities of the function for a few times. Used for testing
#[cfg(test)]
mod test_alien_spawn_prob {
    use std::time::Duration;

    use crate::aliens::alien::get_probability_to_spawn_an_alien;

    #[test]
    fn print_probabilities_over_time() {
        for i in 1..(30 * 60) {
            let prob = get_probability_to_spawn_an_alien(Duration::from_millis(i * 100), 0, 0);
            println!("{}", prob);
        }

        let total: f32 = (1..120)
            .map(|i| get_probability_to_spawn_an_alien(Duration::from_secs(i), 0, 0))
            .sum();
        println!("Total: {}", total * 12.5);
        assert_eq!(1, 1);
    }
}

#[derive(Resource, Clone, Debug)]
pub struct AlienSpawnAngle {
    // The possible locations at which to spawn are given by the circle from Grid - aka base center and radius
    // and the angle given by this.
    // It changes periodically so that aliens come in batches
    angle: f32,
    // The deviation can also change over time so that aliens don't spawn at the same spot all the time
    deviation: f32,
    timer: Timer,
}
impl Default for AlienSpawnAngle {
    fn default() -> Self {
        AlienSpawnAngle {
            angle: 0.0,
            deviation: 0.5,
            timer: Timer::new(Duration::from_secs(20), TimerMode::Repeating),
        }
    }
}

// All the aliens should come from a similar spot, but so that they don't always come from the same one
// we change the bearing from which they come every 20-40s
pub fn alien_spawning_randomize_angle(mut res: ResMut<AlienSpawnAngle>, time: Res<Time>) {
    res.timer.tick(time.delta());
    if res.timer.finished() {
        let mut rng = rand::thread_rng();
        let min_d = 10_f32;
        let max_d = 30_f32;
        let dur = (rng.gen::<f32>() * (max_d - min_d)) + min_d;
        dbg!(dur);
        res.timer.reset();
        res.timer
            .set_duration(Duration::from_millis((dur * 1000.) as u64));

        // Deviation
        let min_d = PI / 8.;
        let max_d = PI / 4.;
        res.deviation = (rng.gen::<f32>() * (max_d - min_d) + min_d);

        res.angle = rng.gen::<f32>() * 2. * PI;
    }
}

pub fn spawn_aliens(
    angle: Res<AlienSpawnAngle>,
    mut commands: Commands,
    mut count: ResMut<AlienCount>,
    grid: Res<Grid>,
    model: Res<AlienModel>,
    time: Res<InGameTime>,
) {
    // let mesh: &Mesh =
    //     Assets::get(Assets, &ass.load("spacekit_2/Models/GLTF format/alien.glb#Scene0")).unwrap();
    let mut rng = rand::thread_rng();

    let prob = get_probability_to_spawn_an_alien(
        time.timer.elapsed(),
        grid.get_square_count() as u32,
        count.count,
    );

    if rng.gen::<f32>() > prob {
        return;
    }

    let angle = angle.angle * rng.gen::<f32>() * angle.deviation;

    let mut x = grid.base_center.x + grid.center_radius * f32::cos(angle);
    let mut z = grid.base_center.z + grid.center_radius * f32::sin(angle);
    x += rng.gen::<f32>() * 2.;
    z += rng.gen::<f32>() * 2.;

    println!("Spawning an alien at {}, {}", x, z);
    count.count += 1;
    commands
        .spawn((
            Alien::default(),
            RigidBody::Dynamic,
            AudioType::Alien,
            Health::new(200),
            LockedAxes::ROTATION_LOCKED_X | LockedAxes::ROTATION_LOCKED_Z,
            SpatialBundle {
                transform: Transform::from_xyz(x, 0.5, z), //.with_scale(Vec3::new(2.0,2.0,2.0)),
                ..default()
            },
            Collider::cylinder(0.4, 0.3),
            Friction::default(),
            CollisionGroups::new(Group::GROUP_10, Group::GROUP_1),
            Velocity { ..default() },
            TargetSelecting {
                range: 2.5,
                target: None,
            },
            DamageDealing {
                cooldown: Timer::from_seconds(0.5, TimerMode::Repeating),
                damage: 5,
            },
        ))
        .with_children(|c| {
            c.spawn((SceneBundle {
                scene: model.0.clone().unwrap(),
                transform: Transform::from_xyz(-2.0, -0.5, -1.5),
                ..default()
            },));
        });
}

// Makes the aliens target the nearest building
// Todo consider AlienTarget.priority
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

// Disables the alien's damage dealing, plays the dying animation and schedules its despawning
pub fn alien_death(
    mut aliens: Query<(
        &mut Transform,
        &mut Alien,
        &mut Velocity,
        &mut DamageDealing,
    )>,
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

// Cleans up the dead aliens
pub fn alien_cleanup(
    time: Res<Time>,
    mut query: Query<(&mut Health, Entity), With<Alien>>,
    mut commands: Commands,
) {
    for mut x in query.iter_mut() {
        x.0.dead_for_timer.tick(time.delta());
        if x.0.dead_for_timer.finished() {
            if let Some(e) = commands.get_entity(x.1) {
                e.despawn_recursive();
            }
        }
    }
}
