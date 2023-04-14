use ::egui::Context;
use bevy::{
    prelude::{system_adapter::unwrap, *},
    ui,
};
use bevy_egui::{egui, EguiContext, EguiPlugin};

use crate::{
    buildings::{
        building_bundles::{Building, BuildingBundle, BuildingTemplates},
        building_system::{self, building_system},
    },
    cameras::pan_camera::{get_primary_window_size, PanOrbitCamera},
    AppState,
};

use super::building_info::{building_info, building_info_ui, BuildingInfo};

#[derive(Resource, Debug)]

pub struct UIState {
    pub mode: UIMode,
}
#[derive(Debug, PartialEq)]
pub enum UIMode {
    Panning,
    Destroying,
    BuildingDefensive(Option<Building>),
    BuildingResources(Option<Building>),
}

pub struct UIPlugin;
impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_plugin(EguiPlugin)
            .init_resource::<BuildingInfo>()
            .add_system_set(
                SystemSet::on_update(AppState::InGame)
                    .with_system(building_info_ui)
                    .with_system(building_system)
                    .with_system(ui_system)
                    .with_system(building_info),
            )
            .insert_resource(UIState {
                mode: UIMode::Panning,
            });
    }
}
const LIGHT_BLUE: Color = Color::rgba(0.0, 186.0 / 256.0, 248.0 / 256.0, 1.0);

const DARK_BLUE: Color = Color::rgba(3.0 / 256.0, 33.0 / 256.0, 59.0 / 256.0, 1.0);

fn lighten_color(color: Color, lighten: f32) -> Color {
    let [h, s, mut l, a] = color.as_hsla_f32();
    l *= lighten;
    return Color::Hsla {
        hue: h,
        saturation: s,
        lightness: l,
        alpha: a,
    };
}

fn ui_system(
    mut ui_state: ResMut<UIState>,
    mut ctx: ResMut<EguiContext>,
    templates: Res<BuildingTemplates>,
) {
    egui::Window::new("MainMenu").show(ctx.ctx_mut(), |ui| {
        ui.horizontal(|ui| {
            let b = ui.selectable_label(
                if let UIMode::BuildingDefensive(_) = ui_state.mode {
                    true
                } else {
                    false
                },
                "Defensive buildings",
            );
            if b.clicked() {
                ui_state.mode = UIMode::BuildingDefensive(None);
            }

            let b = ui.selectable_label(
                if let UIMode::BuildingResources(_) = ui_state.mode {
                    true
                } else {
                    false
                },
                "Resource buildings",
            );
            if b.clicked() {
                ui_state.mode = UIMode::BuildingResources(None);
            }

            let b = ui.selectable_label(ui_state.mode == UIMode::Destroying, "Demolish");
            if b.clicked() {
                ui_state.mode = UIMode::Destroying;
            }

            let a = ui.selectable_label(ui_state.mode == UIMode::Panning, "Pan");

            if a.clicked() {
                ui_state.mode = UIMode::Panning;
            }
        });
        if let UIMode::BuildingDefensive(_) = &ui_state.mode {
            templates.templates.iter().for_each(|b| {
                if !b.show_in_menu {
                    return;
                }
                if let BuildingBundle::DEFENSIVE(_) = b.bundle {
                    ui.horizontal(|ui| {
                        let but = ui.selectable_label(b == b, b.building_info.name.to_string());
                        if but.clicked() {
                            ui_state.mode = UIMode::BuildingDefensive(Some(b.clone()));
                        };
                    });
                }
            });
        };

        if let UIMode::BuildingResources(_) = &ui_state.mode {
            templates.templates.iter().for_each(|b| {
                if !b.show_in_menu {
                    return;
                }
                if let BuildingBundle::GENERATOR(_) = b.bundle {
                    ui.horizontal(|ui| {
                        let but = ui.selectable_label(b == b, b.building_info.name.to_string());
                        if but.clicked() {
                            ui_state.mode = UIMode::BuildingResources(Some(b.clone()));
                        };
                    });
                }
            });
        };
    });
    // ctx.ctx_mut().
    // handle_ui_click(&mut res, &w);

    // commands
    //     .spawn(NodeBundle {
    //         style: Style {
    //             flex_direction: FlexDirection::Column,
    //             align_items: AlignItems::Stretch,
    //             justify_content: JustifyContent::SpaceBetween,
    //             ..default()
    //         },
    //         ..default()
    //     })
    //     .with_children(|parent| {
    //         parent
    //             .spawn(NodeBundle {
    //                 style: Style {
    //                     flex_direction: FlexDirection::Row,
    //                     align_items: AlignItems::Stretch,
    //                     justify_content: JustifyContent::FlexStart,
    //                     ..Default::default()
    //                 },
    //                 ..Default::default()
    //             })
    //             .with_children(|parent: &mut ChildBuilder| {
    //                 spawn_button(
    //                     parent,
    //                     &ass,
    //                     MyButton {
    //                         mode: Some(UIMode::Building),
    //                         is_toggle: true,
    //                         toggled: Some(false),
    //                         button_type: ButtonType::Building,
    //                     },
    //                     "Build",
    //                 );
    //                 spawn_button(
    //                     parent,
    //                     &ass,
    //                     MyButton {
    //                         mode: Some(UIMode::Panning),
    //                         is_toggle: true,
    //                         toggled: Some(true),
    //                         button_type: ButtonType::Panning,
    //                     },
    //                     "Pan camera",
    //                 );
    //             });
    //     });
}
