use bevy::prelude::*;
use bevy_kira_audio::{prelude::*, Audio};

use crate::health::health::DeathEvent;

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
}

impl Default for AudioHandles {
    fn default() -> Self {
        AudioHandles {
            building_explosion: None,
        }
    }
}

pub fn register_sounds(ass: Res<AssetServer>, mut audio_handles: ResMut<AudioHandles>) {
    audio_handles.building_explosion = ass.load("sounds/explosion.wav").into();
}

pub fn explosion_on_death(
    audio: Res<Audio>,
    // ass: Res<AssetServer>,
    mut events: EventReader<DeathEvent>,
    audio_handles: Res<AudioHandles>,

    camera: Query<&Transform, With<Camera>>,
    positions: Query<(&Transform, Entity), Without<Camera>>,
) {
    let camera_pos = camera
        .get_single()
        .and_then(|q| Ok(q.translation))
        .unwrap_or(Vec3::new(0., 15., 0.));

    for ev in events.iter() {
        let position = positions
            .get(ev.entity)
            .and_then(|q| Ok(q.0.translation))
            .unwrap_or(Vec3::splat(0.));

        let distance = position.distance(camera_pos);

        dbg!(distance);

        // Dont play sounds if the camera is more than 15 units away from the source
        if (distance > 45.) {
            return;
        };

        // Decrease faster than normally
        let mut volume = 1. / (f32::log2(distance * 5.));
        // Decrease the total volume for this effect
        volume += 0.05;

        // let explosion = 
        audio
            .play(audio_handles.building_explosion.clone().unwrap())
            .with_volume(Volume::Amplitude(volume.into()));
    }
}
