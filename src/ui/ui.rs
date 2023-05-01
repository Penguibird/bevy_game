use std::fmt::format;

use ::egui::Context;
use bevy::{
    prelude::{system_adapter::unwrap, *},
    ui,
};
use bevy_egui::{
    egui::{
        self,
        style::{Margin, Spacing},
        Align2, Id, LayerId, Style,
    },
    EguiContext, EguiPlugin,
};

use crate::{
    buildings::{
        building_bundles::{Building, BuildingBundle, BuildingTemplates},
        building_system::{self, building_system, hide_highlight_square},
        resource_images::ResourceImages,
    },
    cameras::pan_camera::{get_primary_window_size, PanOrbitCamera},
    menu::menu::make_window,
    AppState,
};

use super::building_info::{building_info, building_info_ui, BuildingInfo};

/// This module defines all the ingame menus UI

#[derive(Resource, Debug)]

pub struct UIState {
    pub mode: UIMode,
}
// This enum not only shows which tab has been selected but also which item
// BuildingDefensive(None) means that the category has been clicked and should be expanded,
// but no building has been selected yet
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
        app.add_plugin(EguiPlugin)
            .init_resource::<BuildingInfo>()
            .add_system_set(
                SystemSet::on_update(AppState::InGame)
                    .with_system(building_info_ui)
                    .with_system(building_system)
                    .with_system(hide_highlight_square)
                    .with_system(ui_system)
                    .with_system(building_info),
            )
            .add_system_set(
                SystemSet::on_enter(AppState::InGame).with_system(set_in_game_menu_styling),
            )
            .insert_resource(UIState {
                mode: UIMode::Panning,
            });
    }
}

// A system to set the styling for the menus drawn in game
// This runs on game start.
// Style changes are expensive
fn set_in_game_menu_styling(mut ctx: ResMut<EguiContext>) {
    let style = ctx.ctx_mut().style().as_ref().clone();
    ctx.ctx_mut().set_style(Style {
        spacing: Spacing {
            menu_margin: Margin::same(4.),
            window_margin: Margin::same(4.),
            item_spacing: egui::Vec2::splat(6.),
            button_padding: egui::Vec2::new(8., 4.),
            ..Default::default()
        },

        ..style
    });
}

// Aliases for the menu control keys
pub mod keys {
    use bevy::prelude::KeyCode;
    use bevy_egui::egui::Key;

    pub const START_GAME: KeyCode = KeyCode::Return;
    pub const EXIT: KeyCode = KeyCode::Escape;
    pub const INSTRUCTIONS: KeyCode = KeyCode::I;

    pub const DEF_BUILDING: KeyCode = KeyCode::Q;
    pub const RES_BUILDING: KeyCode = KeyCode::W;
    pub const DEMOLISH: KeyCode = KeyCode::E;
    pub const PAN: KeyCode = KeyCode::R;
    pub const NUMPADS: [KeyCode; 10] = [
        KeyCode::Numpad1,
        KeyCode::Numpad2,
        KeyCode::Numpad3,
        KeyCode::Numpad4,
        KeyCode::Numpad5,
        KeyCode::Numpad6,
        KeyCode::Numpad7,
        KeyCode::Numpad8,
        KeyCode::Numpad9,
        KeyCode::Numpad0,
    ];
    pub const NUMERALS: [KeyCode; 10] = [
        KeyCode::Key1,
        KeyCode::Key2,
        KeyCode::Key3,
        KeyCode::Key4,
        KeyCode::Key5,
        KeyCode::Key6,
        KeyCode::Key7,
        KeyCode::Key8,
        KeyCode::Key9,
        KeyCode::Key0,
    ];
}

// pub trait MyToString  {
//     fn string(&self) -> &str;
// }
// impl MyToString for KeyCode {
//     fn string(&self) -> &str {
//         use KeyCode::*;
//         match &self {
//             Q => "Q",
//             W => "W",
//             E => "E",
//             R => "R",
//             _ => "_",
//         }
//     }
// }

// The main menu for building/demolishing/panning
fn ui_system(
    mut ui_state: ResMut<UIState>,
    mut ctx: ResMut<EguiContext>,
    resource_images: Res<ResourceImages>,
    templates: Res<BuildingTemplates>,
    // For keyboard navigation
    keys: Res<Input<KeyCode>>,
) {
    let mut building_details_to_show: Option<&Building> = None;

    make_window(Align2::LEFT_TOP, None).show(ctx.ctx_mut(), |ui| {
        ui.horizontal(|ui| {
            let b = ui.selectable_label(
                if let UIMode::BuildingDefensive(_) = ui_state.mode {
                    true
                } else {
                    false
                },
                "(Q) Defensive buildings",
            );
            if b.clicked() || keys.pressed(keys::DEF_BUILDING) {
                ui_state.mode = UIMode::BuildingDefensive(None);
            }

            let b = ui.selectable_label(
                if let UIMode::BuildingResources(_) = ui_state.mode {
                    true
                } else {
                    false
                },
                "(W) Resource buildings",
            );
            if b.clicked() || keys.pressed(keys::RES_BUILDING) {
                ui_state.mode = UIMode::BuildingResources(None);
            }

            let b = ui.selectable_label(ui_state.mode == UIMode::Destroying, "(E) Demolish");
            if b.clicked() || keys.pressed(keys::DEMOLISH) {
                ui_state.mode = UIMode::Destroying;
            }

            let a = ui.selectable_label(ui_state.mode == UIMode::Panning, "(R) Pan");

            if a.clicked() || keys.pressed(keys::PAN) {
                ui_state.mode = UIMode::Panning;
            }
        });
        ui.horizontal_top(|ui| {
            ui.set_max_width(100.);
            ui.vertical_centered_justified(|ui| {
                if let UIMode::BuildingDefensive(ref mut selected_building) = &mut ui_state.mode {
                    templates
                        .templates
                        .iter()
                        .filter(|b| b.show_in_menu)
                        .filter(|b| {
                            if let BuildingBundle::DEFENSIVE(_) = &b.bundle {
                                true
                            } else {
                                false
                            }
                        })
                        .enumerate()
                        .for_each(|(i, b)| {
                            let checked = Some(b) == selected_building.as_ref();
                            let but = ui.selectable_label(
                                checked,
                                format!("({}) {}", i + 1 , b.building_info.name.to_string()),
                            );
                            let pressed = keys.pressed(keys::NUMERALS[i])
                                || keys.pressed(keys::NUMPADS[i]);
                            if but.clicked() || pressed {
                                *selected_building = Some(b.clone());
                            };
                            if but.hovered() || pressed {
                                building_details_to_show = Some(b);
                            };
                        });
                };

                if let UIMode::BuildingResources(ref mut selected_building) = &mut ui_state.mode {
                    templates
                        .templates
                        .iter()
                        .filter(|b| b.show_in_menu)
                        .filter(|b| {
                            if let BuildingBundle::GENERATOR(_) = &b.bundle {
                                true
                            } else {
                                false
                            }
                        })
                        .enumerate()
                        .for_each(|(i, b)| {
                            let checked = Some(b) == selected_building.as_ref();
                            let but = ui.selectable_label(
                                checked,
                                format!("({}) {}", i + 1, b.building_info.name.to_string()),
                            );

                            let pressed = keys.pressed(keys::NUMERALS[i])
                                || keys.pressed(keys::NUMPADS[i]);
                            if but.clicked() || pressed {
                                *selected_building = Some(b.clone());
                            };

                            if but.hovered() || pressed {
                                building_details_to_show = Some(b);
                            };
                        });
                };
            });
            ui.vertical(|ui| {
                // This is the building selected to construct
                let mut selected_building: Option<&Building> = None;
                if let UIMode::BuildingDefensive(Some(b)) | UIMode::BuildingResources(Some(b)) =
                    &ui_state.mode
                {
                    selected_building = Some(b);
                };

                if let Some(b) = building_details_to_show.or(selected_building) {
                    match &b.bundle {
                        BuildingBundle::GENERATOR(bundle) => {
                            ui.vertical(|ui| {
                                ui.image(b.building_info.image, (100., 100.));
                                ui.label(b.building_info.description);
                                ui.label(format!(
                                    "Generates {} of {} every {} s",
                                    bundle.generator.amount,
                                    bundle.generator.resource_type,
                                    bundle.generator.timer.duration().as_millis() as f32 / 1000.,
                                ));
                                ui.label(format!("Health: {}", bundle.health.max_hp));
                                ui.label("Cost: ");
                                b.cost.display(ui, &resource_images, false);
                            });
                        }
                        BuildingBundle::DEFENSIVE(bundle) => {
                            ui.vertical(|ui| {
                                ui.image(b.building_info.image, (100., 100.));
                                ui.label(b.building_info.description);
                                ui.label(format!("Range: {}", bundle.target_selecting.range));
                                ui.label(format!(
                                    "Damage: {} every {} s",
                                    bundle.damage_dealing.damage,
                                    bundle.damage_dealing.cooldown.duration().as_millis() as f32
                                        / 1000.
                                ));
                                ui.label(format!("Health: {}", bundle.health.max_hp));
                                ui.label("Cost: ");
                                b.cost.display(ui, &resource_images, false);
                            });
                        }
                    }
                }
            })
        });
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
