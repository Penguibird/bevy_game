use std::time::Duration;

use bevy::{prelude::*, time::Stopwatch};
use bevy_egui::{
    egui::{self, Align2, Color32, RichText},
    EguiContext,
};

use crate::{menu::menu::make_window, AppStage, AppState};

/// Handles the in game timer
/// Bevy's time doesn't account for our custom AppState::InGame state so we need to maintain this
/// Also handles the win condition
/// As well as displaying the time left before victory
pub struct GameTimerPlugin;

impl Plugin for GameTimerPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(InGameTime {
            timer: Stopwatch::new(),
        })
        .add_system_set(SystemSet::on_enter(AppState::InGame).with_system(start_in_game_time))
        .add_system_set(SystemSet::on_update(AppState::InGame).with_system(update_in_game_time))
        .add_system_set(SystemSet::on_pause(AppState::InGame).with_system(pause_in_game_time))
        .add_system_set(SystemSet::on_resume(AppState::InGame).with_system(unpause_in_game_time))
        .add_system_set(
            SystemSet::on_update(AppState::InGame)
                .with_system(win_condition)
                .with_system(game_time_ui),
        );
    }
}

// Bevy's in built time doesn't account for game starting/ending/pausing,
// Therefore we have this stopwatch which only runs in-game.
// We can still use the Res<Time> for the delta
#[derive(Resource, Clone, Debug)]
pub struct InGameTime {
    pub timer: Stopwatch,
}

// After restarting the game the timer may have some time in it so we reset it first.
pub fn start_in_game_time(mut time: ResMut<InGameTime>) {
    time.timer.reset();
    time.timer.unpause();
}

// Pause needs to be handled
pub fn pause_in_game_time(mut time: ResMut<InGameTime>) {
    time.timer.pause();
}

pub fn unpause_in_game_time(mut time: ResMut<InGameTime>) {
    time.timer.unpause();
}

// We have to tick the timer every time we need to read it.
// Since this gets awkward, I just tick it every gametick instead in this system
pub fn update_in_game_time(t: Res<Time>, mut time: ResMut<InGameTime>) {
    time.timer.tick(t.delta());
}

// If the player survives this many minutes they win the game
pub const WIN_MINUTES: u64 = 15;
/// used in testing
// pub const WIN_MINUTES: u64 = 100;
pub const WIN_TIME: Duration = Duration::from_secs(60 * WIN_MINUTES);

pub fn win_condition(time: Res<InGameTime>, mut game_state: ResMut<State<AppState>>) {
    if time.timer.elapsed() > WIN_TIME {
        game_state.set(AppState::Victory).unwrap();
    }
}

pub fn pad(string: String) -> String {
    if string.len() < 2 {
        "0".to_string() + &string
    } else {
        string
    }
}
pub fn game_time_ui(time: Res<InGameTime>, mut ctx: ResMut<EguiContext>) {
    make_window(Align2::RIGHT_TOP, None).show(ctx.ctx_mut(), |ui| {
        ui.set_width(80.);
        ui.vertical_centered(|ui| {
            ui.wrap_text();
            ui.label("Time until extraction");
            let time = (if WIN_TIME <= time.timer.elapsed() {
                0
            } else {
                (WIN_TIME - time.timer.elapsed()).as_secs()
            });
            let mins = time / 60;
            let secs = time % 60;
            let text = format!("{}:{}", pad(mins.to_string()), pad(secs.to_string()));
            ui.label(
                RichText::new(text)
                    .font(egui::FontId {
                        size: 15.,
                        family: egui::FontFamily::Monospace,
                    })
                    .color(Color32::WHITE),
            );
        });
    });
}
