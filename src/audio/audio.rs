use bevy::prelude::*;
use bevy_kira_audio::{prelude::*, Audio};

use crate::{
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
            .add_system_set(SystemSet::on_update(AppState::InGame).with_system(explosion_on_death));
    }
}

#[derive(Resource, Clone, Debug)]
pub struct AudioHandles {
    building_explosion: Option<Handle<AudioSource>>,
    alien_death: Option<Handle<AudioSource>>,
}

impl Default for AudioHandles {
    fn default() -> Self {
        AudioHandles {
            building_explosion: None,
            alien_death: None,
        }
    }
}

// Register sounds, so we only load the files once and store a reference in a resource
pub fn register_sounds(ass: Res<AssetServer>, mut audio_handles: ResMut<AudioHandles>) {
    *audio_handles = AudioHandles {
        building_explosion: ass.load("sounds/explosion.wav").into(),
        alien_death: ass.load("sounds/alien_death.wav").into(),
    }
}

#[derive(Component, Clone, Copy, PartialEq, Debug)]
pub enum AudioType {
    Building,
    Alien,
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
    // We get the camera pos to figure out how far the listener is from the sound source
    let camera_pos = camera
        .get_single()
        .and_then(|q| Ok(q.translation))
        .unwrap_or(Vec3::new(0., 15., 0.));

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

            let distance = transform.translation.distance(camera_pos);

            // Dont play sounds if the camera is more than x units away from the source
            // We can get a different max_dist based on the soudn type.
            let max_dist: f32 = match sound {
                AudioType::Alien => 30.,
                _ => 45.,
            };
            if (distance > max_dist) {
                return;
            };

            // Decrease faster than in the real world
            let mut volume = 1. / (f32::log2(distance * 5.));

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
