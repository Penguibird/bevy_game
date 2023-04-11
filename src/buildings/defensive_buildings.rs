use bevy::{
    input::{keyboard::KeyboardInput, ButtonState},
    prelude::*,
};
use bevy_rapier3d::prelude::{Collider, CollisionGroups, Group, RigidBody};

use crate::{
    aliens::alien::Alien,
    effects::muzzleflash::{GunFireEvent, GunType},
    health::health::{DeathEvent, Health},
};

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
    time: Res<Time>,
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
                println!("Speeder retargetting")
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
    mut querySet: ParamSet<(
        Query<(
            &mut DamageDealing,
            &mut TargetSelecting,
            Entity,
            &Transform,
            Option<&GunType>,
        )>,
        Query<(&mut Health, Entity)>,
    )>,
    mut ev: EventWriter<DeathEvent>,
    mut gun_fire_event: EventWriter<GunFireEvent>,
) {
    let mut speeders = querySet.p0();
    // let aliens = querySet.p1();
    let mut targets: Vec<(Entity, i32, Entity)> = Vec::new();
    for (mut d, mut t, e, transform, gun_type) in speeders.iter_mut() {
        d.cooldown.tick(time.delta());
        if !d.cooldown.just_finished() {
            continue;
        }
        if let Some(t) = t.target {
            targets.push((t, d.damage, e));

            let gun_transform = transform.clone();

            if let Some(gun_type) = gun_type {
                gun_fire_event.send(GunFireEvent {
                    transform: gun_transform,
                    gun_type: *gun_type,
                });
            };
        }
    }

    for (target, d, killer) in targets {
        let mut killed = false;
        if let Ok((mut h, e)) = querySet.p1().get_mut(target) {
            h.hp -= d;
            if h.hp <= 0 {
                killed = true;
                ev.send(DeathEvent {
                    entity: target,
                    killer: Some(killer),
                })
            }
        }

        if !killed {
            continue;
        }
        if let Ok(mut speeder) = querySet.p0().get_mut(killer) {
            speeder.1.target = None;
        }
    }
}

pub fn speeder_death(
    mut query: Query<(&mut Transform, Entity), With<Speeder>>,
    mut ev: EventReader<DeathEvent>,
    mut commands: Commands,
) {
    for e in ev.iter() {
        if let Ok((mut t, speeder)) = query.get_mut(e.entity) {
            let r = t.rotation;
            t.rotate_axis(r.mul_vec3(Vec3::X), 180.0);
            commands.entity(speeder).despawn_recursive();
            // a.alive = false;
        }
    }
    ev.clear();
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
