use std::{fmt::Display, time::Duration};

use bevy::{prelude::*, time::Timer};
use bevy_egui::{
    egui::{self, Align2, Color32, Frame, RichText, Stroke, TextStyle},
    EguiContext,
};

use crate::{menu::menu::make_window, AppStage, AppState};

pub struct ErrorMessagePlugin;
impl Plugin for ErrorMessagePlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<ErrorEvent>().add_system_set(
            SystemSet::on_update(AppState::InGame)
                .with_system(tick_and_despawn_errors)
                .with_system(render_errors)
                .with_system(spawn_errors),
        );
    }
}

#[derive(Clone, Copy, Debug)]
pub enum ErrorEvent {
    NothingToDestroy,
    CantDestroyYourOwnBase,
    NotEnoughResources,
    SpaceOccupied,
}

impl Display for ErrorEvent {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use ErrorEvent::*;
        f.write_str(match self {
            NothingToDestroy => "There are no buildings on the selected square",
            NotEnoughResources => "You don't have enough resources to construct this building.",
            CantDestroyYourOwnBase => "You can't demolish your main base",
            SpaceOccupied => "This space is already occupied by another building.",
        })
    }
}

#[derive(Clone, Debug, Component)]
pub struct ErrorMessage {
    pub despawn_timer: Timer,
    pub event: ErrorEvent,
}

impl ErrorMessage {
    pub fn _new(duration: Duration, event: ErrorEvent) -> Self {
        Self {
            despawn_timer: Timer::new(duration, TimerMode::Once),
            event,
        }
    }
    pub fn new(event: ErrorEvent) -> Self {
        Self {
            despawn_timer: Timer::new(ERROR_DURATION, TimerMode::Once),
            event,
        }
    }
}

// This is how long each error message should stay on screen
const ERROR_DURATION: Duration = Duration::from_millis(2_000);

// Reacts to ErrorEvent by spawning the component
pub fn spawn_errors(mut ev: EventReader<ErrorEvent>, mut commands: Commands) {
    for ev in ev.iter() {
        println!("Error: {}", ev);
        commands.spawn(ErrorMessage::new(*ev));
    }
    ev.clear();
}

// Ticks the despawn timer for all ErroMessage entities and despawns them if appropriate
pub fn tick_and_despawn_errors(
    mut errors: Query<(&mut ErrorMessage, Entity)>,
    time: Res<Time>,
    mut commands: Commands,
) {
    for (mut e, entity) in errors.iter_mut() {
        e.despawn_timer.tick(time.delta());
        if e.despawn_timer.finished() {
            if let Some(mut e) = commands.get_entity(entity) {
                e.despawn();
            }
        }
    }
}

// Renders all the erromessages in a window on the left
pub fn render_errors(errors: Query<&ErrorMessage>, mut ctx: ResMut<EguiContext>) {
    make_window(Align2::LEFT_CENTER, Some((0., 100.)))
        .frame(Frame {
            fill: Color32::TRANSPARENT,
            stroke: Stroke {
                width: 0.,
                color: Color32::TRANSPARENT,
            },
            ..Default::default()
        })
        .show(ctx.ctx_mut(), |ui| {
            ui.set_max_height(200.);
            ui.vertical(|ui| {
                // We sort the errors by time elapsed to show the oldest ones at the top
                // To sort them we first need to collect them in a vector
                // It's not efficient, but we won't ever have > 50 errors so it should be fine
                let mut errors = errors.iter().collect::<Vec<&ErrorMessage>>();
                errors.sort_by(|a: &&ErrorMessage, b: &&ErrorMessage| {
                    // params are reversed to show the oldest ones first
                    b.despawn_timer.elapsed().cmp(&a.despawn_timer.elapsed())
                });

                for e in errors.iter() {
                    let m = ERROR_DURATION.as_millis() as f32;
                    let mut alpha = (m - e.despawn_timer.elapsed().as_millis() as f32) / m;
                    dbg!(alpha);
                    alpha *= 255.;
                    let alpha = alpha as u8;
                    dbg!(alpha);
                    ui.label(
                        RichText::new(e.event.to_string())
                            .strong()
                            .size(12.)
                            .color(Color32::from_rgba_unmultiplied(255, 255, 255, alpha)),
                    );
                }
            })
        });
}
