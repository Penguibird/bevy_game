use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_egui::{
    egui::{Context, TextureId},
    EguiContext,
};
use bevy_mod_picking::PickableBundle;
use bevy_rapier3d::prelude::{Collider, CollisionGroups, Group, RigidBody};

use crate::{
    audio::audio::AudioType,
    effects::{gun_idle_animations::get_laser_gun_hover_animator, muzzleflash::GunType},
    health::{self, health::Health},
    main_base::main_base::register_main_base,
    ui::building_info,
    AppStage, AppState,
};

use super::{
    defensive_buildings::*,
    resources::{ResourceGenerator, ResourceSet},
};

use super::grid::{Grid, SQUARE_SIZE};

// In this module we define all the possible buildings
// There's a lot of structs holding the various info that we can then clone and insert into the world.

// The actual registering has been split across multiple systems, so we define a plugin for all of them.
pub struct BuildingTemplatesPlugin;

impl Plugin for BuildingTemplatesPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(BuildingTemplates {
            templates: Vec::new(),
        })
        .add_startup_system(register_defensive)
        .add_startup_system(register_main_base)
        .add_startup_system(register_resources);
    }
}

// The bundle of components specific to the different buildign types
#[derive(Bundle, Clone, Debug)]
pub struct GeneratorBuildingBundle {
    pub health: Health,
    pub alien_target: AlienTarget,
    pub generator: ResourceGenerator,
    pub collider: Collider,
}

#[derive(Bundle, Debug, Clone)]
pub struct DefensiveBuildingBundle {
    pub health: Health,
    pub alien_target: AlienTarget,
    pub damage_dealing: DamageDealing,
    pub target_selecting: TargetSelecting,
    pub gun_type: GunType,
    pub collider: Collider,
}
#[derive(Clone, Debug)]
pub enum BuildingBundle {
    GENERATOR(GeneratorBuildingBundle),
    DEFENSIVE(DefensiveBuildingBundle),
}

// The struct containing all the common information for all buildings
#[derive(Clone, Debug)]
pub struct Building {
    pub show_in_menu: bool,
    // pub name: String,
    pub building_info: BuildingInfoComponent,
    pub bundle: BuildingBundle,
    pub cost: ResourceSet,
    pub scene_handle: Handle<Scene>,
    pub scene_offset: Transform,
}

// A struct containing non-game info about the buildings. All UI stuff should go here.
#[derive(Component, Clone, Debug)]
pub struct BuildingInfoComponent {
    // A static reference, because we want the names and descriptions to be on the stack during the whole runtime of the game
    pub name: &'static str,
    pub description: &'static str,
    pub image: TextureId,
}

impl Building {
    // A factory function to abstract the common options and simplify new building creation.
    // You can still define a new building directly, we do so for the main base for example
    fn new_defensive(
        health: i32,
        cost: ResourceSet,
        damage: i32,
        cooldown: u32,
        range: f32,
        gun_type: GunType,
        collider_radius: Option<f32>,
        scale: f32,
        model_name: &str,
        name: &'static str,
        description: &'static str,
        ass: &Res<AssetServer>,
        ctx: &mut ResMut<EguiContext>,
    ) -> Self {
        Building {
            show_in_menu: true,
            building_info: BuildingInfoComponent {
                name,
                image: ctx.add_image(ass.load(format!(
                    "spacekit_2/Isometric_trimmed/{}_SE.png",
                    model_name
                ))),
                description,
            },
            bundle: BuildingBundle::DEFENSIVE(DefensiveBuildingBundle {
                health: Health::new(health),
                alien_target: AlienTarget { priority: 5 },
                damage_dealing: DamageDealing::new(damage, cooldown),
                target_selecting: TargetSelecting {
                    target: None,
                    range,
                },
                gun_type,
                collider: Collider::cylinder(1.0, collider_radius.unwrap_or(0.5)),
            }),
            cost,
            scene_handle: ass.load(format!(
                "spacekit_2/Models/GLTF format/{}.glb#Scene0",
                model_name
            )),
            scene_offset: Transform {
                scale: Vec3::splat(scale),
                translation: Vec3::new(-2., 0.0, -1.5) * scale,
                ..Default::default()
            },
        }
    }
}

impl PartialEq for Building {
    fn eq(&self, other: &Self) -> bool {
        self.building_info.name == other.building_info.name
    }
}

impl Building {
    // command pattern
    // Doesn't take a reference to self, meaning you need to clone it before calling .build()
    // The commands need to be passed in. We can't hold a reference to them for longer than a game tick
    // Returns the entity built.
    pub fn build(self, commands: &mut Commands, point: Vec3) -> Option<Entity> {
        // The scene needs to be inserted as a child so it can be displaced
        let scene = SceneBundle {
            scene: self.scene_handle,
            transform: self.scene_offset,
            ..default()
        };

        // The bundle to be inserted into every building
        // The default components all bundles should have.
        // Some are static marker components, such as AudioType and RigidBody, some depend on the building info.
        let default_bundle = (
            self.cost,
            AudioType::Building,
            PickableBundle::default(),
            self.building_info,
            RigidBody::Fixed,
            CollisionGroups::new(Group::GROUP_1, Group::ALL),
            SpatialBundle {
                transform: Transform {
                    translation: point,
                    rotation: Quat::from_axis_angle(Vec3::Y, PI / 2.),

                    ..Default::default()
                },
                ..Default::default()
            },
        );

        match self.bundle {
            BuildingBundle::DEFENSIVE(b) => {
                let g = b.gun_type.clone();

                let mut c = commands.spawn((default_bundle, b));

                // Add the gun idle animation.
                if g == GunType::LaserGun {
                    c.insert(get_laser_gun_hover_animator());
                }

                c.with_children(|parent| {
                    parent.spawn(scene);
                });

                return c.id().into();
            }
            BuildingBundle::GENERATOR(b) => {
                return commands
                    .spawn((b, default_bundle))
                    .with_children(|parent| {
                        parent.spawn(scene);
                    })
                    .id()
                    .into();
            }
        };
    }
}

// The global resource containing all the templates.
#[derive(Resource)]
pub struct BuildingTemplates {
    pub templates: Vec<Building>,
}

// The base range of a machine gun is defined here, which means all the other ranges can be defined relative to it
const MACHINE_GUN_RANGE: f32 = 8.0;
pub fn register_defensive(
    mut templates: ResMut<BuildingTemplates>,
    ass: Res<AssetServer>,
    mut ctx: ResMut<EguiContext>,
) {
    templates.templates.push(Building::new_defensive(
        100,
        ResourceSet::new(100, 40, 3),
        30,
        500,
        3. * MACHINE_GUN_RANGE,
        GunType::LaserGun,
        (0.5 * 1.15).into(),
        0.4,
        "craft_speederA",
        "Laser speeder",
        "",
        &ass,
        &mut ctx,
    ));
    templates.templates.push(Building::new_defensive(
        100,
        ResourceSet::new(50, 0, 0),
        30,
        100,
        MACHINE_GUN_RANGE,
        GunType::MachineGun,
        (0.5 * 1.15).into(),
        1.,
        "turret_single",
        "Machine gun mk1",
        "",
        &ass,
        &mut ctx,
    ));

    templates.templates.push(Building::new_defensive(
        100,
        ResourceSet::new(200, 10, 0),
        30,
        100,
        MACHINE_GUN_RANGE,
        GunType::MachineGunMk2,
        (0.5 * 1.15).into(),
        1.15,
        "turret_double",
        "Machine gun mk2",
        "",
        &ass,
        &mut ctx,
    ));
    // TODO Add more buildings
    // TODO Tie this to buttons
}

impl Building {
    // A factory function to abstract the common options and simplify new building creation.
    // You can still define a new building directly, we do so for the main base for example
    pub fn new_resource(
        name: &'static str,
        description: &'static str,
        generator: ResourceGenerator,
        health: i32,
        cost: ResourceSet,
        model_name: &str,
        scale: f32,
        ass: &Res<AssetServer>,
        ctx: &mut ResMut<EguiContext>,
    ) -> Self {
        Building {
            show_in_menu: true,
            building_info: BuildingInfoComponent {
                name,
                image: ctx.add_image(ass.load(format!(
                    "spacekit_2/Isometric_trimmed/{}_SE.png",
                    model_name
                ))),
                description,
            },
            bundle: BuildingBundle::GENERATOR(GeneratorBuildingBundle {
                health: Health::new(health),
                alien_target: AlienTarget::default(),
                generator,
                collider: Collider::cylinder(1.0, 0.5 * scale),
            }),
            cost,
            scene_handle: ass.load(format!(
                "spacekit_2/Models/GLTF format/{}.glb#Scene0",
                model_name
            )),
            scene_offset: Transform {
                scale: Vec3::splat(scale),
                translation: Vec3::new(-2., 0.0, -1.5) * scale,
                ..Default::default()
            },
        }
    }
}

pub fn register_resources(
    mut templates: ResMut<BuildingTemplates>,
    ass: Res<AssetServer>,
    mut ctx: ResMut<EguiContext>,
) {
    templates.templates.push(Building::new_resource(
        "Mine tier 1",
        "",
        ResourceGenerator::new(super::resources::ResourceType::Ore, 1, 2_000),
        100,
        ResourceSet::new(25, 0, 0),
        "monorail_trainCargo",
        1.,
        &ass,
        &mut ctx,
    ));

    templates.templates.push(Building::new_resource(
        "Mine tier 2",
        "",
        ResourceGenerator::new(super::resources::ResourceType::Ore, 1, 1_000),
        100,
        ResourceSet::new(100, 50, 0),
        "monorail_trainCargo",
        1.5,
        &ass,
        &mut ctx,
    ));

    templates.templates.push(Building::new_resource(
        "Gas collector",
        "",
        ResourceGenerator::new(super::resources::ResourceType::Gas, 1, 5_000),
        100,
        ResourceSet::new(100, 0, 0),
        "machine_wirelessCable",
        1.,
        &ass,
        &mut ctx,
    ));

    templates.templates.push(Building::new_resource(
        "Monofractioning crystallizer",
        "",
        ResourceGenerator::new(super::resources::ResourceType::Crystal, 1, 5_000),
        100,
        ResourceSet::new(200, 50, 0),
        "satelliteDish_detailed",
        1.,
        &ass,
        &mut ctx,
    ));
}
