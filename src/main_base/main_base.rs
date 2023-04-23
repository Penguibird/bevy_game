use bevy::prelude::*;
use bevy_egui::EguiContext;
use bevy_rapier3d::prelude::Collider;

use crate::{
    buildings::{building_bundles::{BuildingTemplates, Building, BuildingInfoComponent, GeneratorBuildingBundle, BuildingBundle}, grid::Grid, defensive_buildings::AlienTarget, resources::{ResourceGenerator, ResourceSet, ResourceType}},
    health::health::Health,
    AppState,
};

// Marker component
#[derive(Clone, Copy, Debug, Component, PartialEq)]
pub struct MainBaseComponent;

// Adds the main base to the building bundles, allowing it to be spawned
pub fn register_main_base(
    mut templates: ResMut<BuildingTemplates>,
    ass: Res<AssetServer>,
    mut ctx: ResMut<EguiContext>,
) {
    let b = Building {
        show_in_menu: false,
        building_info: BuildingInfoComponent {
            name: "Main base",
            image: ctx.add_image(ass.load("spacekit_2/Isometric/hangar_largeA_SW.png")),
            description: "",
        },
        bundle: BuildingBundle::GENERATOR(GeneratorBuildingBundle {
            health: Health::new(1000),
            alien_target: AlienTarget { priority: 8 },
            generator: ResourceGenerator::new(ResourceType::Ore, 1, 10_000),
            collider: Collider::cuboid(1.1 * 0.8, 2.0 * 0.8, 1.28),
        }),
        cost: ResourceSet::new(0, 0, 0),
        scene_handle: ass.load("spacekit_2/Models/GLTF format/hangar_largeA.glb#Scene0"),
        scene_offset: Transform {
            scale: Vec3::new(0.8, 0.8, 0.8),
            translation: Vec3::new(-1.6, 0.0, -1.3),
            ..Default::default()
        },
    };

    templates.templates.push(b);

}

pub fn spawn_main_base(
    templates: Res<BuildingTemplates>,
    mut grid: ResMut<Grid>,
    mut commands: Commands,
) {
    let c = templates.templates.clone();
    let b = c.iter().find(|b| b.building_info.name == "Main base");

    let e = b
        .unwrap()
        .clone()
        .build(&mut commands, Vec3::splat(0.))
        .unwrap();
    grid.block_square_vec3(Vec3::splat(0.), e);
    commands.get_entity(e).unwrap().insert(MainBaseComponent);
}

// Every tick we check that the base is alive and set the game state to game over if it is not
// We don't need a GameOver event, because we can track it using the state
pub fn handle_main_base_gameover(
    query: Query<&Health, With<MainBaseComponent>>,
    mut game_state: ResMut<State<AppState>>,
) {
    if let Ok(health) = query.get_single() {
        if health.hp <= 0 {
            game_state.set(AppState::GameOver).unwrap();
        }
    } else {
        // If the building doesnt exist exit too
        game_state.set(AppState::GameOver).unwrap();
    };
}
