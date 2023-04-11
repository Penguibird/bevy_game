use std::f32::consts::PI;

use bevy::prelude::*;
use bevy_rapier3d::prelude::{Collider, CollisionGroups, Group, RigidBody};

use crate::{
    effects::{gun_idle_animations::get_laser_gun_hover_animator, muzzleflash::GunType},
    health::health::Health,
};

use super::{
    building_system::Grid,
    defensive_buildings::*,
    resources::{ResourceGenerator, ResourceSet},
};
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
    pub name: String,
    pub bundle: BuildingBundle,
    pub cost: ResourceSet,
    pub scene_handle: Handle<Scene>,
    pub scene_offset: Transform,
}

impl PartialEq for Building {
    fn eq(&self, other: &Self) -> bool {
        self.name == other.name
    }
}

impl Building {
    pub fn build(self, commands: &mut Commands, point: Vec3) {
        let scene = SceneBundle {
            scene: self.scene_handle,
            transform: self.scene_offset,
            ..default()
        };
        let x = (
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

                // TODO Refactor to reduce duplication
                if g == GunType::LaserGun {
                    commands
                        .spawn((x, b, get_laser_gun_hover_animator()))
                        .with_children(|parent| {
                            parent.spawn((scene));
                        });
                } else {
                    commands.spawn((b, x)).with_children(|parent| {
                        parent.spawn(scene);
                    });
                }
            }
            BuildingBundle::GENERATOR(b) => {
                commands.spawn((b, x)).with_children(|parent| {
                    parent.spawn(scene);
                });
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
    mut grid: ResMut<Grid>,
) {
    let b = Building {
        show_in_menu: false,
        name: String::from("Main base"),
        bundle: BuildingBundle::GENERATOR(GeneratorBuildingBundle {
            health: Health::new(1000),
            alien_target: AlienTarget { priority: 8 },
            generator: ResourceGenerator::new(super::resources::ResourceType::Ore, 1, 1000),
            collider: Collider::cuboid(1.1 * 0.8, 2.0 * 0.8, 1.28),
        }),
        scene_handle: ass.load("spacekit_2/Models/GLTF format/hangar_largeA.glb#Scene0"),
        cost: ResourceSet::new(0, 0, 0),
        scene_offset: Transform {
            scale: Vec3::new(0.8, 0.8, 0.8),
            translation: Vec3::new(-1.6, 0.0, -1.3),
            ..Default::default()
        },
    };

    // b.clone()
    //     .build(&mut commands, Grid::get_plane_pos(Vec3::ZERO));

    templates.templates.push(b);

    grid.block_square_vec3(Vec3::ZERO);
}

const MACHINE_GUN_RANGE: f32 = 5.0;
pub fn register_defensive(mut templates: ResMut<BuildingTemplates>, ass: Res<AssetServer>) {
    templates.templates.push(Building {
        show_in_menu: true,
        name: String::from("LaserSpeeder"),
        bundle: BuildingBundle::DEFENSIVE(DefensiveBuildingBundle {
            gun_type: GunType::LaserGun,
            target_selecting: TargetSelecting::new(3. * MACHINE_GUN_RANGE),
            health: Health::new(100),
            alien_target: AlienTarget { priority: 4 },
            damage_dealing: DamageDealing::new(30, 500),
            collider: Collider::cylinder(1.0, 0.5 * 1.15),
        }),
        scene_handle: ass.load("spacekit_2/Models/GLTF format/craft_speederA.glb#Scene0"),
        cost: ResourceSet::new(10, 40, 0),
        scene_offset: Transform {
            scale: Vec3::all(0.4),
            translation: Vec3::new(-2., 0.0, -1.5) * 0.4,
            ..Default::default()
        },
    });

    templates.templates.push(Building {
        show_in_menu: true,
        name: String::from("MachineGunMk2"),
        bundle: BuildingBundle::DEFENSIVE(DefensiveBuildingBundle {
            gun_type: GunType::MachineGunMk2,
            target_selecting: TargetSelecting::new(MACHINE_GUN_RANGE),
            health: Health::new(100),
            alien_target: AlienTarget { priority: 4 },
            damage_dealing: DamageDealing::new(30, 100),
            collider: Collider::cylinder(1.0, 0.5 * 1.15),
        }),
        scene_handle: ass.load("spacekit_2/Models/GLTF format/turret_double.glb#Scene0"),
        cost: ResourceSet::new(20, 10, 0),
        scene_offset: Transform {
            // TODO Scale
            scale: Vec3::all(1.15),
            translation: Vec3::new(-2., 0.0, -1.5) * 1.15,
            ..Default::default()
        },
    });
    templates.templates.push(Building {
        show_in_menu: true,
        name: String::from("MachineGunMk1"),
        bundle: BuildingBundle::DEFENSIVE(DefensiveBuildingBundle {
            gun_type: GunType::MachineGun,
            target_selecting: TargetSelecting::new(MACHINE_GUN_RANGE),
            health: Health::new(100),
            alien_target: AlienTarget { priority: 4 },
            damage_dealing: DamageDealing::new(10, 250),
            collider: Collider::cylinder(1.0, 0.5),
        }),
        scene_handle: ass.load("spacekit_2/Models/GLTF format/turret_single.glb#Scene0"),
        cost: ResourceSet::new(5, 0, 0),
        scene_offset: Transform {
            // scale: Vec3::new(0.8, 0.8, 0.8),
            translation: Vec3::new(-2., 0.0, -1.5),
            ..Default::default()
        },
    });
    // TODO Add more buildings
    // TODO Tie this to buttons
}

pub fn register_resources(
    mut templates: ResMut<BuildingTemplates>,
    mut commands: Commands,
    ass: Res<AssetServer>,
    mut grid: ResMut<Grid>,
) {
    templates.templates.push(Building {
        show_in_menu: true,
        name: String::from("Mine tier 1"),
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
