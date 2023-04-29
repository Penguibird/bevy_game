use bevy::{app::AppExit, prelude::*};
use bevy_egui::{
    egui::{self, style::Spacing, Vec2},
    egui::{style::Margin, Rect, Style, *},
    EguiContext,
};

use crate::{game_timer::game_timer::WIN_MINUTES, AppState};

// This is the main game menu that you see on the game start
pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(set_styles)
            // .init_resource::<MainMenuState>()
            .add_system_set(SystemSet::on_update(AppState::MainMenu).with_system(main_menu))
            .add_system_set(SystemSet::on_update(AppState::GameOver).with_system(game_over))
            .add_system_set(SystemSet::on_update(AppState::Victory).with_system(victory_screen))
            .add_system_set(
                SystemSet::on_update(AppState::Victory).with_system(instruction_screen),
            );
    }
}

// #[derive(Clone, Copy, Debug, Resource)]
// pub struct MainMenuState {}

// impl Default for MainMenuState {
//     fn default() -> Self {
//         MainMenuState {}
//     }
// }

const LIGHT_BLUE: Color32 = Color32::from_rgb(0, 186, 248);
const DARK_BLUE: Color32 = Color32::from_rgb(3, 33, 59);

// This isnt proper lightening, but works well enough for these purposes
fn lighten_color(color: Color32, factor: f32) -> Color32 {
    Color32::from_rgb(
        (color.r() as f32 * factor).round() as u8,
        (color.g() as f32 * factor).round() as u8,
        (color.b() as f32 * factor).round() as u8,
    )
}

// An abstraciton to make the creation of this struct simpler as we need to pass it in for all the different variations
fn make_widget_visuals(bg: Color32, fg: Color32) -> style::WidgetVisuals {
    style::WidgetVisuals {
        bg_fill: bg,
        bg_stroke: Stroke {
            width: 2.,
            color: fg,
        },
        fg_stroke: Stroke {
            width: 2.,
            color: fg,
        },
        rounding: Rounding::same(0.),
        expansion: 1.,
    }
}

// The default visual struct allowing us to simply lighten or darken it for hovered/active effects
fn def_widget_visuals(factor: f32) -> style::WidgetVisuals {
    make_widget_visuals(
        lighten_color(DARK_BLUE, factor),
        lighten_color(LIGHT_BLUE, factor),
    )
}

// Sets the default theme for all the ui elements
pub fn set_styles(mut ctx: ResMut<EguiContext>) {
let visuals = ctx.ctx_mut().style().visuals.clone();

    ctx.ctx_mut().set_visuals(Visuals {
        dark_mode: true,
        // override_text_color: Color32::WHITE.into(),
        override_text_color: LIGHT_BLUE.into(),
        // widgets: (),
        // selection: (),
        // hyperlink_color: (),
        // faint_bg_color: (),
        // extreme_bg_color: (),
        // code_bg_color: (),
        // warn_fg_color: (),
        // error_fg_color: (),
        window_rounding: Rounding::same(0.),
        // window_shadow: (),
        window_fill: DARK_BLUE,
        window_stroke: Stroke {
            width: 2.,
            color: LIGHT_BLUE,
        },
        panel_fill: DARK_BLUE,
        resize_corner_size: 0.,
        // clip_rect_margin: (),
        button_frame: true,
        // collapsing_header_frame: (),
        widgets: style::Widgets {
            noninteractive: def_widget_visuals(0.5),
            active: def_widget_visuals(1.),
            hovered: def_widget_visuals(1.),
            inactive: def_widget_visuals(1.),
            open: def_widget_visuals(1.),
            // inactive: (),
            // hovered: (),
            // active: (),
            // open: (),
        },

        ..visuals
    });
    // let mut fonts = FontDefinitions::default();
    // fonts.font_data.insert(
    //     TITLE_FONT_NAME.to_string(),
    //     FontData::from_static(include_bytes!("..\\..\\assets\\fonts\\DatcubBold.ttf")),
    // );
    // ctx.ctx_mut().set_fonts(fonts);
    // ctx.ctx_mut().;
}

// This function configures the ctx to have the large spacing that we use for main menus
fn set_menu_spacing(ctx: &mut ResMut<EguiContext>) {
    let margin = Margin::symmetric(80., 60.);
    let style = ctx.ctx_mut().style();
    ctx.ctx_mut().set_style(bevy_egui::egui::Style {
        spacing: Spacing {
            window_margin: margin,
            menu_margin: margin,
            button_padding: egui::Vec2 { x: 8., y: 4. },
            item_spacing: egui::Vec2 { x: 4., y: 16. },
            ..default()
        },
        ..style.as_ref().clone()
    });
}

const _TITLE_FONT_NAME: &'static str = "Datcub";

// An abstraction above egui::Window which also adds the default options we want for all menus
// Align is the placement of the window - aka the corner of the main window in which it will be positioned
// Optionally pass in offset from the corner
pub fn make_window(align: Align2, offset: Option<(f32, f32)>) -> egui::Window<'static> {
    egui::Window::new({
        // We need to give each window a unique title. We could pass a title as a param to the function
        // but since we will only have one window in each corner at a time- each with different alignment
        // we can just stringify the placement to get a unique string.
        let vec = align.to_sign();
        format!("{},{}", vec.x, vec.y)
    })
    // Placement
    .anchor(align, offset.unwrap_or((0., 0.)))
    .pivot(align)
    .current_pos((0., 0.))
    // Also disable resizing
    .resizable(false)
    // Dont add an arrow to collapse it
    .collapsible(false)
    // Dont show the title bar. If we want a title we can add it as a ui element
    .title_bar(false)
}

// Renders the main menu
fn main_menu(
    mut app_state: ResMut<State<AppState>>,
    mut ctx: ResMut<EguiContext>,
    mut exit: EventWriter<AppExit>,
) {
    // Currently doesnt work - shelved
    // Check if font exists
    // if !ctx.ctx_mut().fonts().families().iter().any(|f| {
    //     if let FontFamily::Name(s) = f {
    //         s.contains(TITLE_FONT_NAME)
    //     } else {
    //         false
    //     }
    // }) {
    //     return;
    // }
    set_menu_spacing(&mut ctx);
    make_window(Align2::CENTER_CENTER, None)
        .min_width(600.)
        .show(ctx.ctx_mut(), |ui| {
            ui.with_layout(
                bevy_egui::egui::Layout::top_down(bevy_egui::egui::Align::Center),
                |ui| {
                    // ui.allocate_space(ui.available_size());

                    // Title
                    ui.vertical_centered(|ui| {
                        // Have the two lines close together
                        ui.spacing_mut().item_spacing = Vec2::splat(1.);

                        let mut heading = |text: &str| {
                            ui.label(RichText::new(text).heading().color(Color32::WHITE).font(
                                FontId {
                                    size: 35.,
                                    family: FontFamily::Monospace,
                                },
                            ));
                        };

                        heading("Deep Space Defenders:");
                        heading("Resource Rumble");
                    });

                    let b = ui.button("Start Game");
                    if b.clicked() {
                        // This shouldnt throw an error as Im never changing app state anywhere else during menu
                        app_state.set(AppState::InGame).unwrap();
                    };

                    exit_game_button(ui, &mut exit);
                },
            )
            // style.text_styles
        });
}

// This is the game over menu that you see after losing the game
fn game_over(
    mut app_state: ResMut<State<AppState>>,
    mut ctx: ResMut<EguiContext>,
    mut exit: EventWriter<AppExit>,
) {
    set_menu_spacing(&mut ctx);

    egui::Window::new("GameOver").show(ctx.ctx_mut(), |ui| {
        ui.label("Game over");
        ui.label("Thanks for playing Deep Space Defenders 2: Resource Rumble Boogaloo");

        main_menu_button(ui, app_state);

        exit_game_button(ui, &mut exit);
    });
}

fn main_menu_button(ui: &mut Ui, mut app_state: ResMut<State<AppState>>) {
    let b = ui.button("Back to main menu");
    if b.clicked() {
        // This shouldnt throw an error as Im never changing app state anywhere else during game over
        app_state.set(AppState::MainMenu).unwrap();
    };
}

// The exit game button, since we render it in both the main menu and the game over menu
fn exit_game_button(ui: &mut Ui, exit: &mut EventWriter<AppExit>) {
    let b = ui.button("Exit game");
    if b.clicked() {
        exit.send(AppExit)
    };
    // return b;
}

fn victory_screen(
    app_state: ResMut<State<AppState>>,
    mut ctx: ResMut<EguiContext>,
    mut exit: EventWriter<AppExit>,
) {

    set_menu_spacing(&mut ctx);
    egui::Window::new("Victory!").show(ctx.ctx_mut(), |ui| {
        ui.label("Victory!");
        ui.label("Congratulations and thanks for playing the game!");
        ui.label("We'd love to tell you you unlocked a harder difficulty, or a new gun, but none of that's been implemented yet.");
        ui.label("In the meantime, follow us for more updates.");

        main_menu_button(ui, app_state);
        exit_game_button(ui, &mut exit)
    });
}
pub fn instruction_screen(app_state: ResMut<State<AppState>>, mut ctx: ResMut<EguiContext>) {
    egui::Window::new("Victory!").show(ctx.ctx_mut(), |ui| {
        ui.label("Here's how to play the game:");
        ui.wrap_text();

        ui.label("You've found yourself crash landed on an alien planet. There's only a handful of survivors and hordes of aliens who don't seem friendly. Luckily the planet seems to be rich in resources! Use them to defend your base! Hopefully you won't have to struggle for too long, a nearby rescue ship has caught your distress signal and is coming to save you!");

        ui.label("Lose/Win conditions");
        ui.label("The hangar in the middle of the map is your main base. If it gets destroyed you lose. Don't let it fall!");
        ui.label(format!("To win the game you need to survive until the rescue ship arrive to extract you - {} minutes after the start of the game. You have a time in-game to keep track of remaining time until extraction", WIN_MINUTES));

        ui.label("Controls:");
        ui.label("The game should be played with a mouse. You can use a keyboard for the menu navigations. In case of assistive devices we recommend rebinding thouse to mouse movements and controlling the menu with arrows.");
        ui.label("By default your cursor is in the Panning mode. This means you can move the camera around by dragging the map, zoom using the scrollbar and also click on a building to view its details in the corner.");
        ui.label("Note: The map can also be moved using WASD");
        ui.label("To build a building you can click on either the Build Defensive or the Build Resource option in the main menu, depending on the category of your desired building. This will expand a list of all the possible buildings. By hovering on a building you can view its details, including its costs. To construct a building successfuly, you need to have enough resources. After selecting your building, click on an empty square on the map to build it.");
        ui.label("If you want to replace a building you can use the demolish option. Demolishing a building returns half its building costs into your inventory.");

        ui.label("Resources:");
        ui.label("You start the game with a limited amount of resources. You'll notice that a little bit of ore trickles in slowly. This comes from your main base, which functions as a resource generator. To increase your resource generation, you will need to construct resource generating building, such as mines or gas collectors. These will increase the rate at which resources generate as long as they're constructed.");

        ui.label("Enemies:");
        ui.label("Pretty quickly aliens will start rushing towards your buildings and damaging them. They can destroy buildings quickly, especialyl in hordes. Make sure you always have enough defensive buildings such as machine guns and that you cover all sides of your base, as the aliens can come from anywhere. Different buildings have different stats, make sure you pay attention to those and build a good mix of the different building types");

        ui.label("Note: Any terrain in the game is purely decorative. You can build over it, aliens run through it and it gives no bonuses as of now");

        main_menu_button(ui, app_state);
    });
}
