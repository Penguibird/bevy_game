use bevy::prelude::*;
use bevy_egui::{egui::TextureId, EguiContext};

use super::resources::ResourceType;

// This module registers and displays all the resource icons.

#[derive(Copy, Clone, Debug, Resource)]
pub struct ResourceImages {
    pub ore: Option<TextureId>,
    pub gas: Option<TextureId>,
    pub crystal: Option<TextureId>,
    pub uranium: Option<TextureId>,
}

impl ResourceImages {
  pub fn get_image(&self, r: &ResourceType) -> TextureId {
    match r {
      ResourceType::Crystal => self.crystal,
      ResourceType::Gas => self.gas,
      ResourceType::Ore => self.ore,
    }.unwrap()
  }
}

impl Default for ResourceImages {
    fn default() -> Self {
        ResourceImages {
            ore: None,
            gas: None,
            crystal: None,
            uranium: None,
        }
    }
}

pub fn register_resource_images(
    ass: Res<AssetServer>,
    mut images: ResMut<ResourceImages>,
    mut ctx: ResMut<EguiContext>,
) {
    let mut img = |name: &str| Some(ctx.add_image(ass.load(format!("resources/{}.png", name))));
    *images = ResourceImages {
        crystal: img("crystal1"),
        gas: img("gas1"),
        ore: img("ore1"),
        uranium: img("uranium1"),
    };
}
