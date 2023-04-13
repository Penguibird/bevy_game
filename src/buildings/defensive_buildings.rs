use std::time::Duration;

use bevy::{
    input::{keyboard::KeyboardInput, ButtonState},
    prelude::*,
};
use bevy_rapier3d::prelude::{Collider, CollisionGroups, Group, RigidBody};
use bevy_tweening::{Animator, EaseFunction, Tween, TweenCompleted};

use crate::{
    aliens::alien::Alien,
    effects::{
        muzzleflash::{GunFireEvent, GunType},
        relative_lenses::RelativeTransformPositionLens,
    },
    health::health::{DeathEvent, Health},
};

use super::building_bundles::BuildingInfoComponent;

use super::grid::{Grid, SQUARE_SIZE};
pub struct DefensiveBuildingPlugin;
impl Plugin for DefensiveBuildingPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(damage_dealing)
            .add_system(defensive_buildings_targetting)
            .add_system(defensive_building_death)
            // .add_event::<DespawnEvent>()
            .add_system(despawn_event_handling);
    }
}

#[derive(Component)]
pub struct Speeder;

#[derive(Component, Debug, Clone, Copy)]
pub struct AlienTarget {
    pub priority: i8,
}

impl Default for AlienTarget {
    fn default() -> Self {
        AlienTarget { priority: 1 }
    }
}

fn distance(a: &Vec3, b: &Vec3) -> f32 {
    f32::sqrt(f32::powi((a.x - b.x), 2) + f32::powi((a.z - b.z), 2))
}
#[derive(Component, Clone, Copy, Debug)]
pub struct TargetSelecting {
    pub target: Option<Entity>,
    pub range: f32,
}
impl TargetSelecting {
    pub fn new(range: f32) -> Self {
        Self {
            target: None,
            range,
        }
    }
}

pub fn defensive_buildings_targetting(
    mut speeders: Query<(&mut Transform, &mut TargetSelecting), Without<Alien>>,
    aliens: Query<(&Health, &Alien, &Transform, Entity)>,
    // muzzleflash_template: Res<MuzzleflashTemplate>,
) {
    for (mut speeder_transform, mut speeder_target) in speeders.iter_mut() {
        if let None = speeder_target.target {
            let x = aliens.iter().find(|(_, a, t, _)| {
                a.alive
                    && distance(&t.translation, &speeder_transform.translation)
                        < speeder_target.range
            });
            if let Some(new_target) = x {
                speeder_target.target = Some(new_target.3);
                // println!("Speeder retargetting")
            }
        }
        if let Some(t) = speeder_target.target {
            let target = aliens.get(t);
            if let Ok(target) = target {
                let target = target.2.translation;
                let me = speeder_transform.translation;
                let diff = target - me;
                let diff = diff.normalize();

                let angle = diff.dot(speeder_transform.rotation.mul_vec3(Vec3::X));
                let t = speeder_transform.translation;
                speeder_transform.rotate_around(t, Quat::from_axis_angle(Vec3::Y, -angle));
                let mut transform = speeder_transform.clone();
                transform.translation += Vec3::Y * 2.;
            }
        }
    }
}

#[derive(Component, Debug, Clone)]
pub struct DamageDealing {
    pub damage: i32,
    pub cooldown: Timer,
}

impl DamageDealing {
    pub fn new(damage: i32, milis: u32) -> Self {
        DamageDealing {
            damage,
            cooldown: Timer::from_seconds(milis as f32 / 1000., TimerMode::Repeating),
        }
    }
}

pub fn damage_dealing(
    time: Res<Time>,
    mut query_set: ParamSet<(
        Query<(
            &mut DamageDealing,
            &mut TargetSelecting,
            Entity,
            &Transform,
            Option<&GunType>,
        )>,
        Query<(&mut Health, &Transform, Entity)>,
    )>,
    mut ev: EventWriter<DeathEvent>,
    mut gun_fire_event: EventWriter<GunFireEvent>,
) {
    let mut speeders = query_set.p0();
    // let aliens = querySet.p1();
    let mut targets: Vec<(Entity, i32, Entity, Vec3, f32)> = Vec::new();
    for (mut d, target_selecting, e, transform, gun_type) in speeders.iter_mut() {
        d.cooldown.tick(time.delta());
        if !d.cooldown.just_finished() {
            continue;
        }
        if let Some(t) = target_selecting.target {
            targets.push((
                t,
                d.damage,
                e,
                transform.translation,
                target_selecting.range,
            ));

            let gun_transform = transform.clone();

            if let Some(gun_type) = gun_type {
                gun_fire_event.send(GunFireEvent {
                    transform: gun_transform,
                    gun_type: *gun_type,
                });
            };
        }
    }

    for (target, d, killer, hitter_translation, hitter_range) in targets {
        let mut killed = false;
        if let Ok((mut h, transform, _)) = query_set.p1().get_mut(target) {
            if transform.translation.distance(hitter_translation).abs() <= hitter_range {
                h.hp -= d;
                if h.hp <= 0 {
                    killed = true;
                    ev.send(DeathEvent {
                        entity: target,
                        killer: Some(killer),
                    })
                }
            }
        }

        if !killed {
            continue;
        }
        if let Ok(mut speeder) = query_set.p0().get_mut(killer) {
            speeder.1.target = None;
        }
    }
}


pub struct DefensiveBuildingDestroyedEvent;

pub fn defensive_building_death(
    mut query: Query<(&mut Transform, Entity), With<BuildingInfoComponent>>,
    mut ev: EventReader<DeathEvent>,
    mut ev_w: EventWriter<DefensiveBuildingDestroyedEvent>,
    mut grid: ResMut<Grid>,
    mut commands: Commands,
) {
    for e in ev.iter() {
        if let Ok((t, e)) = query.get_mut(e.entity) {
            // t.translation += Vec3::new(0. -5., 0.);
            let start = Vec3::new(0., 0., 0.);
            let end = Vec3::new(0., -3., 0.);
            let point = t.translation.clone();
            grid.unblock_square_vec3(point);
            let tween = Tween::new(
                EaseFunction::BounceIn,
                Duration::from_millis(1000),
                RelativeTransformPositionLens {
                    previous: start,
                    start,
                    end,
                },
            )
            .with_completed_event(DESPAWN_EVENT_CODE);
            commands.entity(e).insert(Animator::new(tween));
            // a.alive = false;
        }
    }
    ev.clear();
}
const DESPAWN_EVENT_CODE: u64 = 13;

pub fn despawn_event_handling(mut commands: Commands, mut ev: EventReader<TweenCompleted>) {
    for ev in ev.iter() {
        if ev.user_data == DESPAWN_EVENT_CODE {
            commands.entity(ev.entity).despawn_recursive();
        }
    }
}

// pub fn speeder_spawning(
//     mut build_event: EventReader<BuildEvent>,
//     mut commands: Commands,
//     ass: Res<AssetServer>,
// ) {
//     for ev in build_event.iter() {
//         if let BuildEvent::Speeder(ev) = ev {
//             let my_gltf = ass.load("spacekit_2/Models/GLTF format/craft_speederA.glb#Scene0");
//             // if let Some(cam_pos) = query.single_mut() {

//             // }
//             println!("Spawning a turret");
//             commands
//                 .spawn((
//                     Speeder,
//                     TargetSelecting {
//                         target: None,
//                         range: 10.0,
//                     },
//                     DamageDealing {
//                         cooldown: Timer::new(Duration::from_millis(100), TimerMode::Repeating),
//                         damage: 10,
//                     },
//                     Health::new(100),
//                     AlienTarget::default(),
//                     // CollisionComponent::default(),
//                     Collider::cylinder(1.0, 1.0),
//                     RigidBody::Fixed,
//                     CollisionGroups::new(Group::GROUP_1, Group::ALL),
//                     SpatialBundle {
//                         transform: Transform::from_xyz(ev.point.x, 0.0, ev.point.z),
//                         ..default()
//                     },
//                 ))
//                 .with_children(|c| {
//                     c.spawn((SceneBundle {
//                         scene: my_gltf,
//                         transform: Transform::from_xyz(-2.0, 0.0, -1.5),
//                         ..Default::default()
//                     },));
//                 });
//         }
//     }
// }
