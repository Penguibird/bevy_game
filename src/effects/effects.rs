use bevy::prelude::*;
use bevy_hanabi::HanabiPlugin;
use bevy_tweening::TweeningPlugin;

use super::{firework, muzzleflash::*};
pub struct ParticlePlugin;

impl Plugin for ParticlePlugin {
    fn build(&self, app: &mut App) {
        app.add_startup_system(setup_muzzleflash)
            .add_startup_system(setup_laserflash)
            .add_system(handle_gun_muzzleflash)
            .add_system(remove_muzzleflash)
            .init_resource::<EffectsHandles>()
            .add_event::<GunFireEvent>()
            .add_plugin(TweeningPlugin)
            // .add_startup_system(firework::firework)
            .add_plugin(HanabiPlugin);
    }
}
