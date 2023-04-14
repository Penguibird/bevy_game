use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_egui::{
    egui::{Context, TextureId},
    EguiContext,
};
use bevy_mod_picking::PickableBundle;
use bevy_rapier3d::prelude::{Collider, CollisionGroups, Group, RigidBody};

use crate::{
    effects::{gun_idle_animations::get_laser_gun_hover_animator, muzzleflash::GunType},
    health::{self, health::Health}, audio::audio::AudioType,
};

use super::{
    defensive_buildings::*,
    resources::{ResourceGenerator, ResourceSet},
};

use super::grid::{Grid, SQUARE_SIZE};
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
#[derive(Component, Clone, Debug)]
pub struct BuildingInfoComponent {
    pub name: &'static str,
    pub image: TextureId,
    pub description: &'static str,
}

impl Building {
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
                image: ctx
                    .add_image(ass.load(format!("spacekit_2/Isometric_trimmed/{}_SE.png", model_name))),
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
    pub fn build(self, commands: &mut Commands, point: Vec3) -> Option<Entity> {
        let scene = SceneBundle {
            scene: self.scene_handle,
            transform: self.scene_offset,
            ..default()
        };

        // The bundle to be inserted into every building
        let default_bundle = (
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

#[derive(Resource)]
pub struct BuildingTemplates {
    pub templates: Vec<Building>,
}
pub trait All {
    fn all(f: f32) -> Vec3;
}

impl All for Vec3 {
    fn all(f: f32) -> Vec3 {
        Self { x: f, y: f, z: f }
    }
}

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

pub fn register_main_base(
    mut templates: ResMut<BuildingTemplates>,
    ass: Res<AssetServer>,
    mut ctx: ResMut<EguiContext>,
) {
    let b = Building {
        show_in_menu: false,
        building_info: BuildingInfoComponent {
            name: "Main base",
            image: ctx.add_image(ass.load("spacekit_2/Isometric/hangar_largeA_SW.png")),
            description: "",
        },
        bundle: BuildingBundle::GENERATOR(GeneratorBuildingBundle {
            health: Health::new(1000),
            alien_target: AlienTarget { priority: 8 },
            generator: ResourceGenerator::new(super::resources::ResourceType::Ore, 1, 1000),
            collider: Collider::cuboid(1.1 * 0.8, 2.0 * 0.8, 1.28),
        }),
        cost: ResourceSet::new(0, 0, 0),
        scene_handle: ass.load("spacekit_2/Models/GLTF format/hangar_largeA.glb#Scene0"),
        scene_offset: Transform {
            scale: Vec3::new(0.8, 0.8, 0.8),
            translation: Vec3::new(-1.6, 0.0, -1.3),
            ..Default::default()
        },
    };

    // b.clone()
    //     .build(&mut commands, Grid::get_plane_pos(Vec3::ZERO));

    templates.templates.push(b);

    // grid.block_square_vec3(Vec3::ZERO);
}

const MACHINE_GUN_RANGE: f32 = 5.0;
pub fn register_defensive(
    mut templates: ResMut<BuildingTemplates>,
    ass: Res<AssetServer>,
    mut ctx: ResMut<EguiContext>,
) {
    templates.templates.push(Building::new_defensive(
        100,
        ResourceSet::new(10, 40, 0),
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
    // templates.templates.push(Building {
    //     show_in_menu: true,
    //     name: String::from("LaserSpeeder"),
    //     bundle: BuildingBundle::DEFENSIVE(DefensiveBuildingBundle {
    //         gun_type: GunType::LaserGun,
    //         target_selecting: TargetSelecting::new(3. * MACHINE_GUN_RANGE),
    //         health: Health::new(100),
    //         alien_target: AlienTarget { priority: 4 },
    //         damage_dealing: DamageDealing::new(30, 500),
    //         collider: Collider::cylinder(1.0, 0.5 * 1.15),
    //     }),
    //     scene_handle: ass.load("spacekit_2/Models/GLTF format/craft_speederA.glb#Scene0"),
    //     scene_offset: Transform {
    //         scale: Vec3::all(0.4),
    //         translation: Vec3::new(-2., 0.0, -1.5) * 0.4,
    //         ..Default::default()
    //     },
    // });

    templates.templates.push(Building::new_defensive(
        100,
        ResourceSet::new(20, 10, 0),
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
    // templates.templates.push(Building { show_in_menu: true,
    //     name: String::from("MachineGunMk2"),
    //     bundle: BuildingBundle::DEFENSIVE(DefensiveBuildingBundle {
    //         gun_type: GunType::MachineGunMk2,
    //         target_selecting: TargetSelecting::new(MACHINE_GUN_RANGE),
    //         health: Health::new(100),
    //         alien_target: AlienTarget { priority: 4 },
    //         damage_dealing: DamageDealing::new(30, 100),
    //         collider: Collider::cylinder(1.0, 0.5 * 1.15),
    //     }),
    //     scene_handle: ass.load("spacekit_2/Models/GLTF format/turret_double.glb#Scene0"),
    //     cost: ResourceSet::new(20, 10, 0),
    //     scene_offset: Transform {
    //         // TODO Scale
    //         scale: Vec3::all(1.15),
    //         translation: Vec3::new(-2., 0.0, -1.5) * 1.15,
    //         ..Default::default()
    //     },
    // });
    templates.templates.push(Building::new_defensive(
        100,
        ResourceSet::new(5, 0, 0),
        10,
        250,
        MACHINE_GUN_RANGE,
        GunType::MachineGun,
        0.5.into(),
        1.,
        "turret_single",
        "Machine gun mk1",
        "",
        &ass,
        &mut ctx,
    ));
    // templates.templates.push(Building {
    //     show_in_menu: true,
    //     name: String::from("MachineGunMk1"),
    //     bundle: BuildingBundle::DEFENSIVE(DefensiveBuildingBundle {
    //         gun_type: GunType::MachineGun,
    //         target_selecting: TargetSelecting::new(MACHINE_GUN_RANGE),
    //         health: Health::new(100),
    //         alien_target: AlienTarget { priority: 4 },
    //         damage_dealing: DamageDealing::new(10, 250),
    //         collider: Collider::cylinder(1.0, 0.5),
    //     }),
    //     scene_handle: ass.load("spacekit_2/Models/GLTF format/turret_single.glb#Scene0"),
    //     cost: ResourceSet::new(5, 0, 0),
    //     scene_offset: Transform {
    //         // scale: Vec3::new(0.8, 0.8, 0.8),
    //         translation: Vec3::new(-2., 0.0, -1.5),
    //         ..Default::default()
    //     },
    // });
    // TODO Add more buildings
    // TODO Tie this to buttons
}

pub fn register_resources(
    mut templates: ResMut<BuildingTemplates>,
    ass: Res<AssetServer>,
    mut ctx: ResMut<EguiContext>,
) {
    templates.templates.push(Building {
        show_in_menu: true,
        building_info: BuildingInfoComponent {
            name: "Mine tier 1",
            image: ctx.add_image(ass.load("spacekit_2/Isometric/monorail_trainCargo_SW.png")),
            description: "",
        },
        bundle: BuildingBundle::GENERATOR(GeneratorBuildingBundle {
            health: Health::new(100),
            collider: Collider::cylinder(1.0, 0.5),
            generator: ResourceGenerator::new(super::resources::ResourceType::Ore, 1, 1000),
            alien_target: AlienTarget { priority: 3 },
        }),
        scene_handle: ass.load("spacekit_2/Models/GLTF format/monorail_trainCargo.glb#Scene0"),
        cost: ResourceSet::new(5, 0, 0),
        scene_offset: Transform {
            // scale: Vec3::new(0.8, 0.8, 0.8),
            translation: Vec3::new(-2., 0.0, -1.5),
            ..Default::default()
        },
    });
}
