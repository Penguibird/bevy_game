use ::egui::Context;
use bevy::{prelude::*, ui};
use bevy_egui::{egui, EguiContext, EguiPlugin};

use crate::{
    buildings::{
        building_bundles::{Building, BuildingBundle, BuildingTemplates},
        building_system::{self, building_system},
    },
    cameras::pan_camera::{get_primary_window_size, PanOrbitCamera},
};

use super::building_info::{building_info, building_info_ui, BuildingInfo};

#[derive(Clone, Copy)]
pub enum ButtonType {
    TestButton,
    Building,
    Panning,
}

#[derive(Resource, Debug)]

pub struct UIState {
    pub mode: UIMode,
}
#[derive(Debug, PartialEq)]
pub enum UIMode {
    Panning,
    BuildingDefensive(Option<Building>),
    BuildingResources(Option<Building>),
}

pub struct ButtonEvent {
    pub interaction: Interaction,
    pub button_type: ButtonType,
}

#[derive(Component)]
pub struct UIButton {
    mode: UIMode,
}

#[derive(Component)]
pub struct MyButton {
    pub mode: Option<UIMode>,
    pub is_toggle: bool,
    pub toggled: Option<bool>,
    pub button_type: ButtonType,
}

pub struct UIPlugin;
impl Plugin for UIPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ButtonEvent>()
            .add_plugin(EguiPlugin)
            .add_system(button_system)
            .add_system(ui_system)
            .add_system(building_info)
            .init_resource::<BuildingInfo>()
            .add_system(building_info_ui)
            .add_system(ui_buttons)
            .add_system(building_system)
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

fn _spawn_button(
    parent: &mut ChildBuilder,
    ass: &Res<AssetServer>,
    button: MyButton,
    text: impl Into<String>,
) {
    parent
        .spawn(NodeBundle {
            style: Style {
                padding: UiRect::all(Val::Px(2.0)),
                ..Default::default()
            },
            // background_color: Color::rgb_u8(, g, b).into(),
            background_color: BackgroundColor(LIGHT_BLUE),
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                .spawn((
                    button,
                    ButtonBundle {
                        style: Style {
                            size: Size::new(Val::Auto, Val::Auto),
                            padding: UiRect::all(Val::Px(16.0)),
                            // border: UiRect::all(Val::Px(2.0)),
                            align_items: AlignItems::Center,
                            justify_content: JustifyContent::Center,
                            ..default()
                        },
                        background_color: BackgroundColor(DARK_BLUE),
                        ..Default::default()
                    },
                ))
                .with_children(|parent| {
                    parent.spawn(TextBundle::from_section(
                        text,
                        TextStyle {
                            font_size: 20.0,
                            font: ass.load("fonts/DatcubBold.ttf"),
                            color: LIGHT_BLUE,
                        },
                    ));
                });
        });
}

// fn button_listener(mut ev: EventReader<ButtonEvent>, mut ui_state: ResMut<UIState>) {
//     for ev in ev.iter() {
//         match ev.button_type {
//             ButtonType::TestButton => match ev.interaction {
//                 Interaction::Clicked => {
//                     println!("Test button clicked");
//                 }
//                 _ => {}
//             },
//             ButtonType::Building => {
//                 if let Interaction::Clicked = ev.interaction {
//                     // ui_state.mode = UIMode::Building;
//                 }
//             }
//             ButtonType::Panning => {
//                 if let Interaction::Clicked = ev.interaction {
//                     ui_state.mode = UIMode::Panning;
//                 }
//             }
//         }
//     }
// }

fn button_system(
    mut interaction_query: Query<
        (
            &Interaction,
            &mut Style,
            &mut MyButton,
            &mut BackgroundColor,
        ),
        (Changed<Interaction>, With<Button>),
    >,
    mut ev_writer: EventWriter<ButtonEvent>,
) {
    for (i, _style, mut b, mut bacground_color) in interaction_query.iter_mut() {
        ev_writer.send(ButtonEvent {
            interaction: *i,
            button_type: b.button_type,
        });

        match *i {
            Interaction::None => {
                if b.is_toggle == false {
                    *bacground_color = DARK_BLUE.into();
                } else {
                    if b.toggled.unwrap_or(false) {
                        *bacground_color = lighten_color(DARK_BLUE, 0.5).into();
                    } else {
                        *bacground_color = DARK_BLUE.into();
                    }
                }
            }
            Interaction::Clicked => {
                if b.is_toggle {
                    if let Some(x) = b.toggled {
                        b.toggled = Some(!x);
                    } else {
                        b.toggled = Some(true);
                    }

                    // Set either clicked or default according to b.toggled;

                    // if (b.toggled.unwrap_or(false)) {
                    //     *bacground_color = lighten_color(DARK_BLUE, 0.8).into();
                    // } else {
                    //     *bacground_color = DARK_BLUE.into();
                    // }
                } else {
                    *bacground_color = lighten_color(DARK_BLUE, 0.8).into();
                }
            }
            Interaction::Hovered => {
                // set hovered
                *bacground_color = lighten_color(DARK_BLUE, 1.5).into();
            }
        }
    }
}

fn ui_buttons(mut query: Query<(&mut MyButton, &mut BackgroundColor)>, ui_state: Res<UIState>) {
    for (mut button, _bc) in query.iter_mut() {
        if let Some(bm) = &button.mode {
            if *bm != ui_state.mode {
                button.toggled = Some(false);
                // *bc = DARK_BLUE.into();
            }
        }
    }
}
