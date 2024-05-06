use bevy::{
    input::{mouse::MouseButtonInput, ButtonState},
    prelude::*,
};
use bevy_egui::EguiContext;
use bevy_kira_audio::{prelude::*, Audio};

use crate::{
    aliens::alien::AlienSpawnEvent,
    effects::muzzleflash::GunFireEvent,
    health::health::{DeathEvent, Health},
    ui::error_info::ErrorEvent,
    AppState,
};

// This modules defines everything related to the audio effects in the game.

pub struct MyAudioPlugin;

impl Plugin for MyAudioPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AudioHandles>()
            .add_startup_system(register_sounds)
            .add_plugin(AudioPlugin)
            .add_system(ui_click)
            .add_system_set(
                SystemSet::on_update(AppState::InGame)
                    .with_system(error_sound)
                    .with_system(alien_spawn_sound)
                    .with_system(gun_fire_sound)
                    .with_system(explosion_on_death),
            )
            .add_system_set(
                SystemSet::on_enter(AppState::Victory).with_system(victory_fanfare_sound),
            )
            .add_system_set(SystemSet::on_enter(AppState::GameOver).with_system(game_over_sound));
    }
}

#[derive(Resource, Clone, Debug)]
pub struct AudioHandles {
    building_explosion: Option<Handle<AudioSource>>,
    alien_death: Option<Handle<AudioSource>>,
    alien_spawn: Option<Handle<AudioSource>>,
    laser_fire: Option<Handle<AudioSource>>,
    gun_fire: Option<Handle<AudioSource>>,
    error: Option<Handle<AudioSource>>,
    click: Option<Handle<AudioSource>>,
    victory: Option<Handle<AudioSource>>,
    game_over: Option<Handle<AudioSource>>,
}

impl Default for AudioHandles {
    fn default() -> Self {
        AudioHandles {
            building_explosion: None,
            alien_death: None,
            alien_spawn: None,
            gun_fire: None,
            laser_fire: None,
            error: None,
            click: None,
            game_over: None,
            victory: None,
        }
    }
}

// Register sounds, so we only load the files once and store a reference in a resource
pub fn register_sounds(ass: Res<AssetServer>, mut audio_handles: ResMut<AudioHandles>) {
    *audio_handles = AudioHandles {
        building_explosion: ass.load("sounds/explosion.wav").into(),
        alien_death: ass.load("sounds/alien_death.wav").into(),
        alien_spawn: ass.load("sounds/alien_spawn.wav").into(),
        gun_fire: ass.load("sounds/machine_gun.mp3").into(),
        laser_fire: ass.load("sounds/laser.wav").into(),
        error: ass.load("sounds/error.mp3").into(),
        click: ass.load("sounds/click.mp3").into(),
        victory: ass.load("sounds/victory.mp3").into(),
        game_over: ass.load("sounds/game_over.mp3").into(),
    }
}

pub fn victory_fanfare_sound(audio: Res<Audio>, audio_handles: Res<AudioHandles>) {
    audio
        .play(audio_handles.victory.clone().unwrap())
        .with_volume(Volume::Amplitude(1.));
}

pub fn game_over_sound(audio: Res<Audio>, audio_handles: Res<AudioHandles>) {
    audio
        .play(audio_handles.game_over.clone().unwrap())
        .with_volume(Volume::Amplitude(1.));
}

#[derive(Component, Clone, Copy, PartialEq, Debug)]
pub enum AudioType {
    Building,
    Alien,
}

pub fn audio_distance_volume(
    camera: &Query<&Transform, With<Camera>>,
    position: Vec3,
    max_dist: Option<f32>,
) -> f32 {
    // We get the camera pos to figure out how far the listener is from the sound source
    let camera_pos = camera
        .get_single()
        .and_then(|q| Ok(q.translation))
        .unwrap_or_else(|_| {
            dbg!("Cannot find camera position");
            Vec3::new(0., 15., 0.)
        });

    let distance = position.distance(camera_pos);

    // Decrease faster than in the real world
    let mut volume = 1. / (f32::log2(distance * 5.));

    if let Some(max_dist) = max_dist {
        if (distance > max_dist) {
            return 0.;
        };
    }

    return volume;
}

// Plays a spatially edited sound for any dying entity
// bevy_kira_audio has a spatial sound option, but it doesn't allow for more volume modification
// This measn I'd have to balance the sound levels externally, which didn't seem right
// As I only need the volume to decrease anyway, I wrote a custom function for that
// This also allows me to tweak how fast sound decreases based on distance
pub fn explosion_on_death(
    audio: Res<Audio>,
    // ass: Res<AssetServer>,
    mut events: EventReader<DeathEvent>,
    audio_handles: Res<AudioHandles>,

    camera: Query<&Transform, With<Camera>>,
    mut dying_entity: Query<(&Transform, Option<&AudioType>, &mut Health, Entity), Without<Camera>>,
) {
    for ev in events.iter() {
        if let Ok((transform, sound, mut health, _)) = dying_entity.get_mut(ev.entity) {
            // Only play sound once
            // The death event can be triggered multiple times for the same entity
            if health.death_sound_played {
                return;
            }
            health.death_sound_played = true;

            // Only play if there is audio
            if let None = sound {
                return;
            }
            let sound = sound.unwrap();

            // Dont play sounds if the camera is more than x units away from the source
            // We can get a different max_dist based on the soudn type.
            let mut volume = audio_distance_volume(
                &camera,
                transform.translation,
                Some(match sound {
                    AudioType::Alien => 45.,
                    _ => 45.,
                }),
            );

            // Decrease the total volume for this effect
            // volume modifiers based on the specific sound
            match sound {
                AudioType::Alien => volume *= 0.01,
                AudioType::Building => volume += 0.05,
            };

            // play sound with the calculated volume
            audio
                .play(
                    match sound {
                        AudioType::Building => audio_handles.building_explosion.clone(),
                        AudioType::Alien => audio_handles.alien_death.clone(),
                    }
                    .unwrap(),
                )
                .with_volume(Volume::Amplitude(volume.into()));
        }
    }
}

pub fn alien_spawn_sound(
    mut ev: EventReader<AlienSpawnEvent>,
    audio_handles: Res<AudioHandles>,

    audio: Res<Audio>,
    camera: Query<&Transform, With<Camera>>,
) {
    for e in ev.iter() {
        let mut volume = audio_distance_volume(&camera, e.point, None);
        volume *= 0.4;

        audio
            .play(audio_handles.alien_spawn.clone().unwrap())
            .with_volume(Volume::Amplitude(volume.into()));
    }
}

// Plays a sound every time a gun fires
pub fn gun_fire_sound(
    mut ev: EventReader<GunFireEvent>,
    audio_handles: Res<AudioHandles>,

    audio: Res<Audio>,
    camera: Query<&Transform, With<Camera>>,
) {
    for e in ev.iter() {
        let mut volume = audio_distance_volume(&camera, e.transform.translation, None);
        volume *= 0.3;

        use crate::effects::muzzleflash::GunType::*;
        audio
            .play(
                match e.gun_type {
                    LaserGun => audio_handles.laser_fire.clone(),
                    MachineGun | MachineGunMk2 => audio_handles.gun_fire.clone(),
                }
                .unwrap(),
            )
            .with_volume(Volume::Amplitude(volume.into()));
    }
}

// Plays a sound on every error
pub fn error_sound(
    mut ev: EventReader<ErrorEvent>,
    audio_handles: Res<AudioHandles>,

    audio: Res<Audio>,
) {
    for _ in ev.iter() {
        audio
            .play(audio_handles.error.clone().unwrap())
            .with_volume(Volume::Amplitude(1.));
    }
}

// Plays a sound on every click in a menu
// We dont wanna play sounds on every click on a map
pub fn ui_click(
    audio_handles: Res<AudioHandles>,

    audio: Res<Audio>,
    mut ctx: ResMut<EguiContext>,
    mut ev: EventReader<MouseButtonInput>,
) {
    for e in ev.iter() {
        // If the left button is clicked
        if e.button == MouseButton::Left && e.state == ButtonState::Pressed {
            // If the mouse is currently over any egui menu
            // Aka if the user clicked on a menu
            if ctx.ctx_mut().is_pointer_over_area() {
                audio
                    .play(audio_handles.click.clone().unwrap())
                    .with_volume(Volume::Amplitude(0.5));
            }
        }
    }
}
