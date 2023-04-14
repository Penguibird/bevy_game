
use bevy::prelude::*;

#[derive(Clone, Copy, Debug, Resource, PartialEq)]
pub struct MainBaseComponent;
pub fn spawn_main_base(
    templates: Res<BuildingTemplates>,
    mut grid: ResMut<Grid>,
    commands: Commands
) {
    
    let e = templates.templates.iter().find(|b| b.building_info.name == "Main Base").unwrap().build(&mut commands, Vec3::splat(0.)).unwrap();
    commands.get_entity(e).unwrap().insert(MainBaseComponent);
}
pub fn handle_main_base_gameover(
  query: Query<&Health, With<MainBaseComponent>>,
  game_state: ResMut<State<AppState>>,
) {

  if let Ok(health) = query.get_single() {
    if health
  }
  
}