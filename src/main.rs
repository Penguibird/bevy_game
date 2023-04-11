use std::f32::consts::PI;

use aliens::alien::AlienPlugin;
use bevy::pbr::DirectionalLightShadowMap;
use bevy::prelude::*;
use bevy::render::settings::WgpuSettings;
use bevy::time::FixedTimestep;
use bevy::utils::HashSet;
use bevy_egui::EguiPlugin;
use bevy_rapier3d::{prelude::*, rapier::prelude::ColliderBuilder};

use bevy_rapier3d::prelude::{Collider, CollisionGroups, Group, LockedAxes, RigidBody, Velocity};
use buildings::building_bundles::{register_defensive, BuildingTemplates, BuildingTemplatesPlugin};
use buildings::building_system::{Grid, SQUARE_SIZE};
use buildings::defensive_buildings::{
    damage_dealing, defensive_buildings_targetting, speeder_death,
};
use buildings::resources::ResourcePlugin;
use cameras::orbit_camera::{camera_testing, pan_orbit_camera, spawn_camera};
use effects::effects::ParticlePlugin;
use effects::muzzleflash::test_muzzleflash;
use health::health::{death_timers, DeathEvent};
use ui::ui::UIPlugin;
mod cameras;

mod aliens;
mod buildings;
mod effects;
mod health;
mod ui;

fn main() {
    // let mut wgpu_settings = WgpuSettings::default();
    // wgpu_settings
    //     .features
    //     .set(VERTEX_WRITABLE_STORAGE, true);

    App::new()
        .add_plugins(DefaultPlugins)
        // .add_plugin(EguiPlugin)
        .add_plugin(AlienPlugin)
        // .add_system(test_muzzleflash)
        .add_plugin(ResourcePlugin)
        .insert_resource(Msaa::default())
        .insert_resource(DirectionalLightShadowMap { size: 50 })
        .insert_resource(Grid::new())
        // * Camera you can rotate
        .add_startup_system(spawn_camera)
        .add_system(pan_orbit_camera)
        // .add_system(camera_testing)
        // *Camera you can only move
        // .add_startup_system(spawn_camera)
        // .add_system(move_camera)
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_plugin(BuildingTemplatesPlugin)
        .add_plugin(UIPlugin)
        // .add_system(speeder_spawning)
        // .add_system(update_positions)
        // .add_system(check_for_collisions.before(update_positions))
        .add_event::<CollisionEvent>()
        .add_event::<DeathEvent>()
        // .add_event::<BuildEvent>()
        .add_startup_system(setup)
        .add_system(damage_dealing)
        .add_system(defensive_buildings_targetting)
        .add_system(death_timers)
        .add_plugin(ParticlePlugin)
        .add_startup_system_to_stage(StartupStage::PostStartup, testing_buildings)
        // .add_system(speeder_death)
        .run();
}

fn testing_buildings(building_templates: Res<BuildingTemplates>, mut commands: Commands) {
    for (i, b) in building_templates.templates.iter().enumerate() {
        b.clone().build(
            &mut commands,
            Grid::get_plane_pos(Vec3::new(0.0 + (SQUARE_SIZE * i as f32), 0., 5.0)),
        );
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    ass: Res<AssetServer>,
) {
    // plane
    println!("Startup system");
    commands
        .spawn((PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane {
                size: 50.0,
                ..Default::default()
            })),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            ..default()
        },))
        .with_children(|c| {
            c.spawn((
                Transform::from_xyz(0.0, -0.02, 0.0),
                Collider::cuboid(50.0, 0.01, 50.0),
                Friction::default()
            ));
        });
    // cube
    // commands.spawn(PbrBundle {
    //     mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
    //     material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
    //     transform: Transform::from_xyz(0.0, 0.5, 0.0),
    //     ..default()
    // });

    // let _my_gltf = ass.load("spacekit_2/Models/GLTF format/turret_single.glb#Scene0");

    // to position our 3d model, simply use the Transform
    // in the SceneBundle
    // TODO Spawn base

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
}
