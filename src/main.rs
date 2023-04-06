use std::f32::consts::PI;

use aliens::alien::AlienPlugin;
use bevy::pbr::DirectionalLightShadowMap;
use bevy::prelude::*;
use bevy_rapier3d::{prelude::*, rapier::prelude::ColliderBuilder};

use bevy_rapier3d::prelude::{Collider, CollisionGroups, Group, LockedAxes, RigidBody, Velocity};
use buildings::buildings::{damage_dealing, speeder_death, speeder_spawning, speeder_targetting};
use cameras::orbit_camera::{pan_orbit_camera, spawn_camera, camera_testing};
use health::health::{death_timers, DeathEvent};
mod cameras;

mod aliens;
mod buildings;
mod health;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(AlienPlugin)
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(DirectionalLightShadowMap { size: 50 })
        // * Camera you can rotate
        .add_startup_system(spawn_camera)
        .add_system(pan_orbit_camera)
        .add_system(camera_testing)
        // *Camera you can only move
        // .add_startup_system(spawn_camera)
        // .add_system(move_camera)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_system(speeder_spawning)
        // .add_system(update_positions)
        // .add_system(check_for_collisions.before(update_positions))
        .add_event::<CollisionEvent>()
        .add_event::<DeathEvent>()
        .add_startup_system(setup)
        .add_system(damage_dealing)
        .add_system(speeder_targetting)
        .add_system(death_timers)
        .add_system(speeder_death)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    ass: Res<AssetServer>,
) {
    // plane
    commands
        .spawn((PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 50.0 })),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            ..default()
        },))
        .with_children(|c| {
            c.spawn((
                Transform::from_xyz(0.0, -0.02, 0.0),
                Collider::cuboid(50.0, 0.01, 50.0),
            ));
        });
    // cube
    // commands.spawn(PbrBundle {
    //     mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
    //     material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
    //     transform: Transform::from_xyz(0.0, 0.5, 0.0),
    //     ..default()
    // });

    let my_gltf = ass.load("spacekit_2/Models/GLTF format/turret_single.glb#Scene0");

    // to position our 3d model, simply use the Transform
    // in the SceneBundle
    commands.spawn(SceneBundle {
        scene: my_gltf,
        transform: Transform::from_xyz(2.0, 0.1, -1.0),
        ..Default::default()
    });
    // light
    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_matrix(Mat4::from_rotation_translation(
            Quat::from_rotation_x(-0.4 * PI) * Quat::from_rotation_y(-0.15 * PI),
            Vec3::new(0.0, 10.0, 0.0),
        )),
        directional_light: DirectionalLight {
            illuminance: 10_000.0,
            shadows_enabled: true,

            ..Default::default()
        },
        ..default()
    });

    // commands.spawn(PointLightBundle {
    //     transform: Transform::from_xyz(8.0,50.0, 5.0),

    //     ..default()
    // });

    // camera
    // commands.spawn(Camera3dBundle {
    //     transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
    //     ..default()
    // });
}
