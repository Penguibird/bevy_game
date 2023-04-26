#![allow(unused_imports, unused_parens)]
use std::f32::consts::PI;

use aliens::alien::{AlienPlugin, Alien};
use audio::audio::MyAudioPlugin;
use bevy::pbr::DirectionalLightShadowMap;
use bevy::prelude::*;
use bevy_rapier3d::na::Point;
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
use main_base::main_base::{handle_main_base_gameover, spawn_main_base};
use map::map::generate_map;
use menu::menu::MenuPlugin;
use ui::ui::UIPlugin;

use crate::map::map::MAP_SIZE;
mod cameras;

mod aliens;
mod audio;
mod buildings;
mod effects;
mod health;
mod main_base;
mod map;
mod menu;
mod ui;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    MainMenu,
    InGame,
    GameOver,
    Paused,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, SystemLabel)]
enum AppStage {
    // Anything that sets up a Res object to add Handles to it.
    RegisterResources,
}

fn main() {
    // let mut wgpu_settings = WgpuSettings::default();
    // wgpu_settings
    //     .features
    //     .set(VERTEX_WRITABLE_STORAGE, true);

    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(Msaa::default())
        .insert_resource(DirectionalLightShadowMap { size: 8_000 })
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
        .add_system_set(SystemSet::on_update(AppState::InGame).with_system(death_timers))
        //
        // Main menu as well as any other state changing menus
        .add_plugin(MenuPlugin)
        //
        // Setup and testing
        .add_system_set(SystemSet::on_enter(AppState::InGame).with_system(setup))
        .add_system_set(
            // Any in game systems
            SystemSet::on_update(AppState::InGame).with_system(handle_main_base_gameover),
        )
        .add_system_set(
            // Any map initialization
            SystemSet::on_enter(AppState::InGame)
                .after(AppStage::RegisterResources)
                .with_system(spawn_main_base)
                .with_system(generate_map)
                // .with_system(testing_buildings),
        )
        .add_system_set(SystemSet::on_enter(AppState::GameOver).with_system(cleanup))
        // .add_startup_system_to_stage(StartupStage::PostStartup, testing_buildings)
        .run();
}

// Removes all the entities that aren't the camera.
// This runs on game over and is used to clear the board.
// All the entities get respawned again on game start
pub fn cleanup(
    entities: Query<Entity, Without<Camera3d>>,
    mut commands: Commands
) {
    for e in entities.iter() {
        if let Some(mut e) = commands.get_entity(e) {
            e.despawn();
        }
    }
}

fn _testing_buildings(
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
    // mut meshes: ResMut<Assets<Mesh>>,
    // mut materials: ResMut<Assets<StandardMaterial>>,
) {
    println!("Startup system");

    // Spawn light
    commands.spawn(DirectionalLightBundle {
        transform: Transform::from_rotation(
            Quat::from_axis_angle(Vec3::X, -PI / 4.) * Quat::from_axis_angle(Vec3::Y, -PI / 6.),
        ),
        directional_light: DirectionalLight {
            illuminance: 40_000.,
            shadows_enabled: true,
            ..Default::default()
        },
        ..Default::default()
    });
    // commands.spawn(PointLightBundle {
    //     transform: Transform::from_xyz(0.0, 100.0, 500.),
    //     point_light: PointLight {
    //         intensity: 20_000.,
    //         shadows_enabled: true,
    //         range: 50_000.,

    //         ..Default::default()
    //     },
    //     ..default()
    // });
}
