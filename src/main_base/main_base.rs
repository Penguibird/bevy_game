use bevy::prelude::*;

use crate::{
    buildings::{building_bundles::BuildingTemplates, grid::Grid},
    health::health::Health,
    AppState,
};

#[derive(Clone, Copy, Debug, Component, PartialEq)]
pub struct MainBaseComponent;

pub fn spawn_main_base(
    templates: Res<BuildingTemplates>,
    mut grid: ResMut<Grid>,
    mut commands: Commands,
) {
    let c = templates.templates.clone();
    let b = c.iter().find(|b| b.building_info.name == "Main base");

// if let None = b {
//     return;
// }
    let e = b
        .unwrap()
        .clone()
        .build(&mut commands, Vec3::splat(0.))
        .unwrap();
    grid.block_square_vec3(Vec3::splat(0.), e);
    commands.get_entity(e).unwrap().insert(MainBaseComponent);
}

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
