use bevy::prelude::*;
use bevy_kira_audio::{prelude::*, Audio};

use crate::health::health::{DeathEvent, Health};

pub struct MyAudioPlugin;

impl Plugin for MyAudioPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(explosion_on_death)
            .insert_resource(SpacialAudio { max_distance: 1. })
            .init_resource::<AudioHandles>()
            .add_startup_system(register_sounds)
            .add_plugin(AudioPlugin);
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

pub fn explosion_on_death(
    audio: Res<Audio>,
    // ass: Res<AssetServer>,
    mut events: EventReader<DeathEvent>,
    audio_handles: Res<AudioHandles>,

    camera: Query<&Transform, With<Camera>>,
    mut dying_entity: Query<(&Transform, Option<&AudioType>, &mut Health, Entity), Without<Camera>>,
) {
    let camera_pos = camera
        .get_single()
        .and_then(|q| Ok(q.translation))
        .unwrap_or(Vec3::new(0., 15., 0.));

    for ev in events.iter() {

        if let Ok((transform, sound, mut health, _)) = dying_entity.get_mut(ev.entity) {
            // Only play sound once
            if health.death_sound_played {
                return;
            }
            health.death_sound_played  = true;

            let distance = transform.translation.distance(camera_pos);

            // Only play if there is audio
            if let None = sound {
                return;
            }
            let sound = sound.unwrap();

            dbg!(distance);

            // Dont play sounds if the camera is more than 15 units away from the source
            let max_dist: f32 = match sound {
                AudioType::Alien => 30.,
                _ => 45.,
            };
            if (distance > max_dist) {
                return;
            };

            // Decrease faster than normally
            let mut volume = 1. / (f32::log2(distance * 5.));
            // Decrease the total volume for this effect
            match sound {
                AudioType::Alien => volume *= 0.01,
                AudioType::Building => volume += 0.05,
                _ => {}
            };

            // let explosion =
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
