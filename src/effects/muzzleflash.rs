use std::{f32::consts::PI, time::Duration};

use bevy::{prelude::*, transform};
use bevy_tweening::{
    lens::{TransformPositionLens, TransformScaleLens},
    Animator, Delay, EaseFunction, Tween,
};

// All the different firing effects are defined here
// This includes all the machine guns
// As well as the laser speeder

#[derive(Component)]
pub struct Muzzleflash {
    pub timer: Timer,
}
#[derive(Resource, Clone, Default)]
pub struct EffectsHandles {
    pub muzzleflash: Option<TextureHandles>,
    pub muzzleflash_line: Option<TextureHandles>,
    pub laser_flash: Option<TextureHandles>,
    pub laser_flash_line: Option<TextureHandles>,
}

#[derive(Clone, Default)]
pub struct TextureHandles {
    pub mesh: Handle<Mesh>,
    pub material: Handle<StandardMaterial>,
}

pub fn setup_laserflash(
    ass: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut muzzelflash_template: ResMut<EffectsHandles>,
) {
    let img_handle = ass.load("effects/muzzleflash_transparent.png");
    let aspect = 0.4;

    // create a new square mesh. this is what we will apply the texture to
    let square_width = 0.3;
    let square_handle = meshes.add(Mesh::from(shape::Quad::new(Vec2::new(
        square_width,
        square_width * aspect,
    ))));

    // this material renders the texture normally
    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(img_handle.clone()),
        alpha_mode: AlphaMode::Blend,
        base_color: Color::Rgba {
            red: 0.8,
            green: 0.,
            blue: 0.,
            alpha: 1.,
        },
        unlit: true,
        double_sided: true,
        ..default()
    });

    let line = meshes.add(Mesh::from(shape::Capsule {
        radius: 0.003,
        depth: 0.4,
        ..default()
    }));
    let line_mat = materials.add(StandardMaterial {
        base_color: Color::Rgba {
            red: 0.85,
            green: 0.,
            blue: 0.,
            alpha: 1.,
        },
        ..default()
    });
    muzzelflash_template.laser_flash = Some(TextureHandles {
        mesh: square_handle,
        material: material_handle,
    });
    muzzelflash_template.laser_flash_line = Some(TextureHandles {
        mesh: line,
        material: line_mat,
    })
}

pub fn setup_muzzleflash(
    ass: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut muzzelflash_template: ResMut<EffectsHandles>,
) {
    let img_handle = ass.load("effects/muzzleflash_transparent.png");
    let aspect = 0.25;

    // create a new square mesh. this is what we will apply the texture to
    let square_width = 0.4;
    let square_handle = meshes.add(Mesh::from(shape::Quad::new(Vec2::new(
        square_width,
        square_width * aspect,
    ))));

    // this material renders the texture normally
    let material_handle = materials.add(StandardMaterial {
        base_color_texture: Some(img_handle.clone()),
        alpha_mode: AlphaMode::Blend,
        unlit: true,
        double_sided: true,
        ..default()
    });

    let line = meshes.add(Mesh::from(shape::Capsule {
        radius: 0.01,
        depth: 0.3,
        ..default()
    }));
    let line_mat = materials.add(StandardMaterial {
        base_color: Color::Rgba {
            red: 1.,
            green: 1.,
            blue: 0.,
            alpha: 1.,
        },
        ..default()
    });
    muzzelflash_template.muzzleflash = Some(TextureHandles {
        mesh: square_handle,
        material: material_handle,
    });
    muzzelflash_template.muzzleflash_line = Some(TextureHandles {
        mesh: line,
        material: line_mat,
    })
}

// The animation specific fire event
#[derive(Clone, Copy)]
pub struct GunFireEvent {
    pub transform: Transform,
    pub gun_type: GunType,
}
#[derive(Component, PartialEq, Clone, Copy, Debug)]
pub enum GunType {
    MachineGun,
    MachineGunMk2,
    LaserGun,
}

// Shows the specific event based on the specific gun
// It is wordy due to each gun having its own displacement/several flashes, but it is very simple
// We check the gun type based on the event and then calculate the position(s) of the muzzleflashes that should be spawned
// Then we spawn them using the spawn_muzzleflash_bundle function
// We add little delays to some of them to add the staggered effect of gunfire
pub fn handle_gun_muzzleflash(
    mut events: EventReader<GunFireEvent>,
    res: Res<EffectsHandles>,
    mut commands: Commands,
) {
    for ev in events.iter() {
        match ev.gun_type {
            GunType::MachineGun => {
                // Gun offset
                let mut transform = ev.transform.clone();
                transform.translation += ev.transform.rotation.mul_vec3(Vec3::new(0., 0.5, -0.65));
                transform.rotate_axis(Vec3::Y, -PI / 2.);

                let mut transform_left = transform.clone();
                let mut transform_right = transform.clone();

                transform_left.translation -=
                    ev.transform.rotation.mul_vec3(Vec3::new(0.1, 0., 0.));
                transform_right.translation -=
                    ev.transform.rotation.mul_vec3(Vec3::new(-0.1, 0., 0.));

                spawn_muzzleflash_bundle(&mut commands, &res, transform_left, None);
                spawn_muzzleflash_bundle(
                    &mut commands,
                    &res,
                    transform_right,
                    Duration::from_millis(100).into(),
                );
            }
            GunType::MachineGunMk2 => {
                // Gun offset
                let mut transform = ev.transform.clone();
                transform.translation += ev.transform.rotation.mul_vec3(Vec3::new(0., 0.48, -0.6));
                transform.rotate_axis(Vec3::Y, -PI / 2.);

                let mut transform_left = transform.clone();
                let mut transform_right = transform.clone();

                let offset = 0.36;
                transform_left.translation -=
                    ev.transform.rotation.mul_vec3(Vec3::new(offset, 0., 0.));
                transform_right.translation -=
                    ev.transform.rotation.mul_vec3(Vec3::new(-offset, 0., 0.));

                spawn_muzzleflash_bundle(&mut commands, &res, transform_left, None);
                let mut transform_left_up = transform_left.clone();

                let vertical_offset = 0.185;
                transform_left_up.translation += Vec3::new(0.0, vertical_offset, 0.);
                spawn_muzzleflash_bundle(
                    &mut commands,
                    &res,
                    transform_left_up,
                    Duration::from_millis(100).into(),
                );

                spawn_muzzleflash_bundle(&mut commands, &res, transform_right, None);

                let mut transform_right_up = transform_right.clone();
                transform_right_up.translation += Vec3::new(0.0, vertical_offset, 0.);
                spawn_muzzleflash_bundle(
                    &mut commands,
                    &res,
                    transform_right_up,
                    Duration::from_millis(100).into(),
                );
            }
            GunType::LaserGun => {
                // Gun offset
                let mut transform = ev.transform.clone();
                transform.translation += ev.transform.rotation.mul_vec3(Vec3::new(0., 0.1, -0.2));
                transform.rotate_axis(Vec3::Y, -PI / 2.);

                let mut transform_left = transform.clone();
                let mut transform_right = transform.clone();
                let offset = 0.25;
                transform_left.translation -=
                    ev.transform.rotation.mul_vec3(Vec3::new(offset, 0., 0.));
                transform_right.translation -=
                    ev.transform.rotation.mul_vec3(Vec3::new(-offset, 0., 0.));

                spawn_laserflash_bundle(&mut commands, &res, transform_left, None);
                spawn_laserflash_bundle(&mut commands, &res, transform_right, None);
            }
        };
    }
}

// Since the spawned flashes are their own entities and not children of the gun entity
// they can technically be a little out of position, but in practice this is barely noticeable
// because they only last for a short time

// Spawns the predefined lazerflash bundle
//
fn spawn_laserflash_bundle(
    commands: &mut Commands,
    res: &Res<EffectsHandles>,
    transform: Transform,
    delay: Option<Duration>,
) -> () {
    // To imitate the explosion of the effect, we scale it up
    let duration = Duration::from_millis(500);
    let scale = Tween::new(
        EaseFunction::QuadraticOut,
        duration,
        TransformScaleLens {
            start: Vec3::splat(1.),
            end: Vec3::splat(1.7),
        },
    );

    // The minimum duration cannot be 0, so if none was provided we use 1ms
    let delay1 = Delay::new(delay.unwrap_or(Duration::from_millis(1)));
    let delay2 = Delay::new(delay.unwrap_or(Duration::from_millis(1)));

    // This is the effect for the yellow "bullet trail" line that flies off from the gun
    let fly = Tween::new(
        EaseFunction::QuadraticOut,
        duration,
        TransformPositionLens {
            start: Vec3::splat(0.),
            end: Vec3::new(-0.8, 0., 0.),
        },
    );
    commands
        .spawn((
            Animator::new(delay1.then(scale)),
            Muzzleflash {
                timer: Timer::new(duration, TimerMode::Once),
            },
            PbrBundle {
                mesh: res.laser_flash.clone().unwrap().mesh.clone(),
                material: res.laser_flash.clone().unwrap().material.clone(),
                transform,
                // .with_rotation(Quat::from_rotation_x(-PI / 5.0))
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn(PbrBundle {
                mesh: res.laser_flash.clone().unwrap().mesh.clone(),
                material: res.laser_flash.clone().unwrap().material.clone(),
                transform: Transform::from_rotation(Quat::from_axis_angle(Vec3::X, PI / 2.)),
                // .with_rotation(Quat::from_rotation_x(-PI / 5.0))
                ..default()
            });

            parent.spawn((
                Animator::new(delay2.then(fly)),
                PbrBundle {
                    mesh: res.laser_flash_line.clone().unwrap().mesh,
                    material: res.laser_flash_line.clone().unwrap().material,
                    transform: Transform::from_rotation(Quat::from_axis_angle(Vec3::Z, PI / 2.)),
                    ..Default::default()
                },
            ));
        });
}

// A single flash with the line coming from it. Used for the machine guns
// Very similar to above but it defines different structs with different delays
fn spawn_muzzleflash_bundle(
    commands: &mut Commands,
    res: &Res<EffectsHandles>,
    transform: Transform,
    delay: Option<Duration>,
) -> () {
    let duration = Duration::from_millis(200);
    let scale = Tween::new(
        EaseFunction::QuadraticOut,
        duration,
        TransformScaleLens {
            start: Vec3::splat(1.),
            end: Vec3::splat(1.7),
        },
    );

    // We can't have 0 delay, so the default one is 1ms
    let delay1 = Delay::new(delay.unwrap_or(Duration::from_millis(1)));
    let delay2 = Delay::new(delay.unwrap_or(Duration::from_millis(1)));

    let fly = Tween::new(
        EaseFunction::QuadraticOut,
        duration,
        TransformPositionLens {
            start: Vec3::splat(0.),
            end: Vec3::new(-0.8, 0., 0.),
        },
    );
    commands
        .spawn((
            Animator::new(delay1.then(scale)),
            Muzzleflash {
                timer: Timer::new(duration, TimerMode::Once),
            },
            PbrBundle {
                mesh: res.muzzleflash.clone().unwrap().mesh.clone(),
                material: res.muzzleflash.clone().unwrap().material.clone(),
                transform,
                // .with_rotation(Quat::from_rotation_x(-PI / 5.0))
                ..default()
            },
        ))
        .with_children(|parent| {
            parent.spawn(PbrBundle {
                mesh: res.muzzleflash.clone().unwrap().mesh.clone(),
                material: res.muzzleflash.clone().unwrap().material.clone(),
                transform: Transform::from_rotation(Quat::from_axis_angle(Vec3::X, PI / 2.)),
                // .with_rotation(Quat::from_rotation_x(-PI / 5.0))
                ..default()
            });

            parent.spawn((
                Animator::new(delay2.then(fly)),
                PbrBundle {
                    mesh: res.muzzleflash_line.clone().unwrap().mesh,
                    material: res.muzzleflash_line.clone().unwrap().material,
                    transform: Transform::from_rotation(Quat::from_axis_angle(Vec3::Z, PI / 2.)),
                    ..Default::default()
                },
            ));
        });
}

// Used for testing and correctly positioning the gunfire effects. Makes the guns fire all the time
pub fn _test_muzzleflash(
    query: Query<(&Transform, &GunType)>,
    mut ev_w: EventWriter<GunFireEvent>,
) {
    for (t, g) in query.iter() {
        ev_w.send(GunFireEvent {
            transform: *t,
            gun_type: *g,
        });
    }
}

// Despawn the muzzleflash entity after the effect runs out.
pub fn remove_muzzleflash(
    mut query: Query<(Entity, &mut Muzzleflash)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (e, mut t) in query.iter_mut() {
        t.timer.tick(time.delta());
        if t.timer.finished() {
            if let Some(e) = commands.get_entity(e) {
                e.despawn_recursive();
            }
        }
    }
}
