use bevy::{input::mouse::MouseButtonInput, prelude::*};
use bevy_egui::{egui, EguiContext};
use bevy_mod_picking::PickingEvent;

use crate::{
    buildings::{grid::Grid, building_bundles::BuildingInfoComponent}, cameras::get_world_point_from_screen::WorldClickEvent,
    effects::muzzleflash::GunType, health::health::Health,
};

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

pub fn building_info(
    mut events: EventReader<WorldClickEvent>,
    grid: Res<Grid>,
    mut building_info: ResMut<BuildingInfo>,
) {
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

pub fn building_info_ui(
    query: Query<(&Health, Option<&GunType>, &BuildingInfoComponent)>,
    mut ctx: ResMut<EguiContext>,
    building_info: ResMut<BuildingInfo>,
) {
    if let Some(e) = building_info.selected_entity {
        egui::Window::new("Building info").show(ctx.ctx_mut(), |ui| {
            if let Ok((h, _, building_info)) = query.get(e) {
                ui.label(building_info.name);
                ui.image(building_info.image, (100., 100.));
                ui.label(format!("Health: {} / {}", h.hp, h.max_hp));
                ui.label(building_info.description);
            };
        });
    }
}
