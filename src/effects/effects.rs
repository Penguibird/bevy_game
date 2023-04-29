use bevy::prelude::*;
use bevy_tweening::TweeningPlugin;

use crate::AppState;

use super::{muzzleflash::*};
pub struct ParticlePlugin;

// Main plugin for all the animations/effects
impl Plugin for ParticlePlugin {
    fn build(&self, app: &mut App) {
        // Setup and register all the handles/images/meshes etc
        app.add_startup_system(setup_muzzleflash)
            // The external plugins
            .add_plugin(TweeningPlugin)
            .add_startup_system(setup_laserflash)
            .init_resource::<EffectsHandles>()
            // Run effects when in game
            .add_system_set(
                SystemSet::on_update(AppState::InGame)
                    .with_system(handle_gun_muzzleflash)
                    .with_system(remove_muzzleflash),
            )
            .add_event::<GunFireEvent>();
    }
}
