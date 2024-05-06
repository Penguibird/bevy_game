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
    AppState,
};

use super::building_bundles::BuildingInfoComponent;

use super::grid::{Grid, SQUARE_SIZE};

// This module defines all the defensive building functions - damage dealing, dying etc.

pub struct DefensiveBuildingPlugin;
impl Plugin for DefensiveBuildingPlugin {
    fn build(&self, app: &mut App) {
        app.add_system_set(
            SystemSet::on_update(AppState::InGame)
                .with_system(damage_dealing)
                .with_system(defensive_buildings_targetting)
                .with_system(building_death)
                .with_system(despawn_event_handling),
            // .add_event::<DespawnEvent>()
        );
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
// Used to keep consistent targets across game ticks
#[derive(Component, Clone, Copy, Debug)]
pub struct TargetSelecting {
    // The target entity that the owner of this component is currently firing at
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

// This system handles only the targeting for defensive buildings
// ALiens are handled in alien_ai
pub fn defensive_buildings_targetting(
    mut defensive_buildings: Query<(&mut Transform, &mut TargetSelecting), Without<Alien>>,
    aliens: Query<(&Health, &Alien, &Transform, Entity)>,
    // muzzleflash_template: Res<MuzzleflashTemplate>,
) {
    for (mut gun_transform, mut gun_target) in defensive_buildings.iter_mut() {
        // Choose a new target if needed
        if let None = gun_target.target {
            let alien = aliens.iter().find(|(_, a, t, _)| {
                a.alive && distance(&t.translation, &gun_transform.translation) < gun_target.range
            });
            if let Some(new_target) = alien {
                gun_target.target = Some(new_target.3);
                // println!("Speeder retargetting")
            }
        }

        // After target acquired, turn towards it
        if let Some(t) = gun_target.target {
            let target = aliens.get(t);
            if let Ok(target) = target {
                
                // If the target is dead, choose a new target
                if target.0.hp <= 0 {
                    gun_target.target = None;
                    return;
                }

                let target = target.2.translation;
                let me = gun_transform.translation;
                let diff = target - me;
                let diff = diff.normalize();

                // Reposition the building
                let angle = diff.dot(gun_transform.rotation.mul_vec3(Vec3::X));
                let t = gun_transform.translation;
                gun_transform.rotate_around(t, Quat::from_axis_angle(Vec3::Y, -angle));
                let mut transform = gun_transform.clone();
                transform.translation += Vec3::Y * 2.;
            }
        }
    }
}

// This system handles the actual damage dealing, for both defensive buildings and aliens
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
            Option<&Health>,
        )>,
        Query<(&mut Health, &Transform, Entity)>,
    )>,
    mut ev: EventWriter<DeathEvent>,
    mut gun_fire_event: EventWriter<GunFireEvent>,
) {
    let mut damage_dealers = query_set.p0();

    // Due to rust borrowing, we can't mutate the targets at the same time as the damage dealers, as these could overlap.
    // (They shouldn't in the current game implementation, but this allows the system to be more generic)
    // We therefore store all the target info in this tuple, in the form:
    // (target, damage, killer, hitter_translation, hitter_range)
    // And process them afterwards
    let mut targets: Vec<(Entity, i32, Entity, Vec3, f32)> = Vec::new();

    // Figure out all the targets and necessary info
    for (mut d, target_selecting, e, transform, gun_type, health) in damage_dealers.iter_mut() {
        // If the entity has a health component - means it can be killed - means we need to check if its alive.
        // Dead entities can still exist for a while - during their death animation
        if let Some(h) = health {
            if h.hp <= 0 {
                continue;
            }
        };

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

    // Process damage dealing to the targets
    for (target, damage, killer, hitter_translation, hitter_range) in targets {
        let mut killed = false;
        if let Ok((mut h, transform, _)) = query_set.p1().get_mut(target) {
            if transform.translation.distance(hitter_translation).abs() <= hitter_range {
                h.hp -= damage;
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
        // If the target dies, the killer's target has to be reset.
        if let Ok(mut speeder) = query_set.p0().get_mut(killer) {
            speeder.1.target = None;
        }
    }
}

// Handle dying, including death animation and sounds triggering
// This handles resource buildings as well
pub fn building_death(
    mut query: Query<(&mut Transform, Entity), With<BuildingInfoComponent>>,
    mut ev: EventReader<DeathEvent>,
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

            // Death animation
            // Shift the building down as if it crumpled to the ground
            let tween = Tween::new(
                EaseFunction::QuadraticOut,
                Duration::from_millis(250),
                RelativeTransformPositionLens {
                    previous: start,
                    start,
                    end,
                },
            )
            .with_completed_event(DESPAWN_EVENT_CODE);
            let e = commands.get_entity(e);
            if let Some(mut e) = e {
                e.insert(Animator::new(tween));
            }
            // a.alive = false;
        }
    }
    ev.clear();
}
// The animation_complete event cannot be our own user-defined event, all we get is to pass a code through.
const DESPAWN_EVENT_CODE: u64 = 13;

// Despawn the building after the animation finishes
pub fn despawn_event_handling(mut commands: Commands, mut ev: EventReader<TweenCompleted>) {
    for ev in ev.iter() {
        if ev.user_data == DESPAWN_EVENT_CODE {
            if let Some(e) = commands.get_entity(ev.entity) {
                e.despawn_recursive();
            }
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
