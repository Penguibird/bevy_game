use bevy::{app::AppExit, prelude::*};
use bevy_egui::{egui, egui::*, EguiContext};

use crate::AppState;

pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MainMenuState>()
            .add_system_set(SystemSet::on_update(AppState::MainMenu).with_system(main_menu))
            .add_system_set(SystemSet::on_update(AppState::GameOver).with_system(game_over));
    }
}

#[derive(Clone, Copy, Debug, Resource)]
pub struct MainMenuState {}

impl Default for MainMenuState {
    fn default() -> Self {
        MainMenuState {}
    }
}

fn main_menu(
    mut ui_state: ResMut<MainMenuState>,
    mut app_state: ResMut<State<AppState>>,
    mut ctx: ResMut<EguiContext>,
    mut exit: EventWriter<AppExit>,

) {
    egui::Window::new("MainMenu").show(ctx.ctx_mut(), |ui| {
        ui.label("Deep Space Defenders 2: Resource Rumble Boogaloo");

        let b = ui.button("Start Game");
        if b.clicked() {
            // This shouldnt throw an error as Im never changing app state anywhere else during menu
            app_state.set(AppState::InGame).unwrap();
        };

        exit_game_button(ui, &mut exit);
    });
}

fn game_over(
    mut app_state: ResMut<State<AppState>>,
    mut ctx: ResMut<EguiContext>,
    mut exit: EventWriter<AppExit>,
) {
    egui::Window::new("GameOver").show(ctx.ctx_mut(), |ui| {
        ui.label("Game over");
        ui.label("Thanks for playing Deep Space Defenders 2: Resource Rumble Boogaloo");

        let b = ui.button("Back to main menu");
        if b.clicked() {
            // This shouldnt throw an error as Im never changing app state anywhere else during game over
            app_state.set(AppState::MainMenu).unwrap();
        };

        exit_game_button(ui, &mut exit);
    });
}

fn exit_game_button(ui: &mut Ui, exit: &mut EventWriter<AppExit>) {
    let b = ui.button("Exit game");
    if b.clicked() {
        exit.send(AppExit)
    };
    // return b;
}
