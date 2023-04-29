use bevy::{input::mouse::MouseButtonInput, prelude::*};
use bevy_egui::{
    egui::{self, Align2},
    EguiContext,
};

use crate::{
    buildings::{building_bundles::BuildingInfoComponent, grid::Grid},
    cameras::get_world_point_from_screen::WorldClickEvent,
    effects::muzzleflash::GunType,
    health::health::Health,
    menu::menu::make_window,
};

// Holds the currenlty selected building to be displayed
#[derive(Resource, Clone, Copy, Debug)]
pub struct BuildingInfo {
    selected_entity: Option<Entity>,
}
impl Default for BuildingInfo {
    fn default() -> Self {
        Self {
            selected_entity: None,
        }
    }
}

// Handles selecting the building to be displayed in building info ui
pub fn building_info(
    mut events: EventReader<WorldClickEvent>,
    grid: Res<Grid>,
    mut building_info: ResMut<BuildingInfo>,
    mut ctx: ResMut<EguiContext>,
) {
    if ctx.ctx_mut().is_pointer_over_area() {
        return;
    }
    for ev in events.iter() {
        if ev.mouse_event.button == MouseButton::Left {
            // dbg!(&grid);
            if let Some(e) = grid.get_entity(ev.point) {
                // dbg!(e);

                building_info.selected_entity = Some(*e);
            }
        }
    }
}

// Shows the info about the selected building
// Buildings can be selected by clicking on them when the UI is in panning mode
pub fn building_info_ui(
    query: Query<(&Health, Option<&GunType>, &BuildingInfoComponent)>,
    mut ctx: ResMut<EguiContext>,
    building_info: ResMut<BuildingInfo>,
) {
    if let Some(e) = building_info.selected_entity {
        let w = make_window(Align2::LEFT_BOTTOM, None).show(ctx.ctx_mut(), |ui| {
            if let Ok((h, _, building_info)) = query.get(e) {
                ui.label(building_info.name);
                ui.image(building_info.image, (100., 100.));
                ui.label(format!("Health: {} / {}", h.hp, h.max_hp));
                ui.label(building_info.description);
            };
        });
    }
}
