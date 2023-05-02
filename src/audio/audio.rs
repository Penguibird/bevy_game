use bevy::prelude::*;
use bevy_kira_audio::{prelude::*, Audio};

use crate::{
    aliens::alien::AlienSpawnEvent,
    health::health::{DeathEvent, Health},
    AppState,
};

// This modules defines everything related to the audio effects in the game.

pub struct MyAudioPlugin;

impl Plugin for MyAudioPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<AudioHandles>()
            .add_startup_system(register_sounds)
            .add_plugin(AudioPlugin)
            .add_system(alien_spawn_sound)
            .add_system_set(SystemSet::on_update(AppState::InGame).with_system(explosion_on_death));
    }
}

#[derive(Resource, Clone, Debug)]
pub struct AudioHandles {
    building_explosion: Option<Handle<AudioSource>>,
    alien_death: Option<Handle<AudioSource>>,
    alien_spawn: Option<Handle<AudioSource>>,
}

impl Default for AudioHandles {
    fn default() -> Self {
        AudioHandles {
            building_explosion: None,
            alien_death: None,
            alien_spawn: None,
        }
    }
}

// Register sounds, so we only load the files once and store a reference in a resource
pub fn register_sounds(ass: Res<AssetServer>, mut audio_handles: ResMut<AudioHandles>) {
    *audio_handles = AudioHandles {
        building_explosion: ass.load("sounds/explosion.wav").into(),
        alien_death: ass.load("sounds/alien_death.wav").into(),
        alien_spawn: ass.load("sounds/alien_spawn.wav").into(),
    }
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
        .unwrap_or(Vec3::new(0., 15., 0.));

    let distance = position.distance(camera_pos);

    // Decrease faster than in the real world
    let mut volume = 1. / (f32::log2(distance * 5.));

    if let Some(max_dist) = max_dist {
        if (distance > max_dist) {
            return 0.;
        };
    }

    return distance;
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
                _ => {}
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
        let volume = audio_distance_volume(&camera, e.point, None);

        audio
            .play(audio_handles.alien_spawn.clone().unwrap())
            .with_volume(Volume::Amplitude(volume.into()));
    }
}
