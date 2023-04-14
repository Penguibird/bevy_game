#![allow(unused_imports, unused_parens)]
use std::f32::consts::PI;

use aliens::alien::AlienPlugin;
use audio::audio::MyAudioPlugin;
use bevy::pbr::DirectionalLightShadowMap;
use bevy::prelude::*;
use bevy_rapier3d::prelude::*;

use bevy_rapier3d::prelude::{Collider, CollisionGroups, Group, LockedAxes, RigidBody, Velocity};
use buildings::building_bundles::{register_defensive, BuildingTemplates, BuildingTemplatesPlugin};
use buildings::defensive_buildings::DefensiveBuildingPlugin;
use buildings::grid::{Grid, SQUARE_SIZE};
use buildings::resources::ResourcePlugin;
use cameras::get_world_point_from_screen::{emit_world_click_events, WorldClickEvent};
use cameras::pan_camera::{pan_orbit_camera, spawn_camera};
use effects::effects::ParticlePlugin;

use health::health::{death_timers, DeathEvent};
use ui::ui::UIPlugin;
mod cameras;

mod aliens;
mod audio;
mod buildings;
mod effects;
mod health;
mod ui;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
enum AppState {
    MainMenu,
    InGame,
    GameOver,
    Paused,
}

fn main() {
    // let mut wgpu_settings = WgpuSettings::default();
    // wgpu_settings
    //     .features
    //     .set(VERTEX_WRITABLE_STORAGE, true);

    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Msaa::default())
        .insert_resource(DirectionalLightShadowMap { size: 50 })
        .add_state(AppState::MainMenu)
        //
        // Physics
        .add_plugin(RapierPhysicsPlugin::<NoUserData>::default())
        .add_plugin(RapierDebugRenderPlugin::default())
        .add_event::<CollisionEvent>()
        //
        // Camera and worldclicking
        .insert_resource(Grid::new())
        // * Camera you can rotate
        .add_startup_system(spawn_camera)
        .add_event::<WorldClickEvent>()
        .add_system(emit_world_click_events)
        .add_system(pan_orbit_camera)
        //
        // Building
        .add_plugin(BuildingTemplatesPlugin)
        .add_plugin(ParticlePlugin)
        .add_plugin(DefensiveBuildingPlugin)
        .add_plugin(UIPlugin)
        // Aliens
        .add_plugin(AlienPlugin)
        // Resource management
        .add_plugin(ResourcePlugin)
        // Audio
        .add_plugin(MyAudioPlugin)
        //
        // Health management
        .add_event::<DeathEvent>()
        .add_system(death_timers)
        //
        // Setup and testing
        .add_startup_system(setup)
        .add_startup_system_to_stage(StartupStage::PostStartup, testing_buildings)
        .run();
}

fn testing_buildings(
    building_templates: Res<BuildingTemplates>,
    mut commands: Commands,
    mut grid: ResMut<Grid>,
) {
    // Spawn all defined buildings in a row in the middle of hte base
    for (i, b) in building_templates.templates.iter().enumerate() {
        let point = Vec3::new(0.0 + (SQUARE_SIZE * i as f32), 0., 5.0);
        let e = b.clone().build(&mut commands, Grid::get_plane_pos(point));
        if let Some(e) = e {
            grid.block_square_vec3(point, e)
        }
    }
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Todo replace with proper map
    // Spawn basic plane
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
                Friction::default(),
            ));
        });
    // Spawn light
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
