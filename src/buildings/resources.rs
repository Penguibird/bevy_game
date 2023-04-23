use std::fmt::Display;

use bevy_egui::{
    egui::{self, Ui},
    EguiContext, EguiPlugin,
};

use bevy::prelude::*;

use crate::AppState;

use super::resource_images::{self, register_resource_images, ResourceImages};

// The amount is aliased here so that we can easily change it later. u8 was too small, but I want to use the smallest possible type here for performance.
type Amount = u16;
#[derive(PartialEq, Clone, Debug, Component)]
pub struct ResourceSet {
    // A vector is actually more performant then a hashmap for this case - due to the small size, the hash function is more complex
    // Normally this would be a struct, but this allows us to add new reource types more easily
    vec: Vec<(ResourceType, Amount)>,
}
impl ResourceSet {
    pub fn new(ore: Amount, gas: Amount, crystal: Amount) -> Self {
        Self {
            vec: vec![
                (ResourceType::Ore, ore),
                (ResourceType::Gas, gas),
                (ResourceType::Crystal, crystal),
            ],
        }
    }


    pub fn add(&mut self, r: ResourceType, amount: Amount) {
        let x = self.vec.iter_mut().find(|x| x.0 == r);
        if let Some(x) = x {
            x.1 += amount;
        };
    }

    pub fn get(&self, r: ResourceType) -> Option<Amount> {
        self.vec.iter().find(|x| x.0 == r).and_then(|x| Some(x.1))
    }

    // Used for the ui
    // Not to be confused with the Display implementation further down
    // Needs the Res<ResourceImages> passed in
    // Renders all the resources into the ui provided
    // show_empty_fields controls whether we want to display resources with amount 0. 
    // We want that for the global ore displays, but not for cost displays for example
    pub fn display(&self, ui: &mut Ui, images: &ResourceImages, show_empty_fields: bool) {
        ui.horizontal(|ui| {
            for (res, amount) in
                self.vec
                    .iter()
                    .filter(|r| if show_empty_fields { true } else { r.1 > 0 })
            {
                ui.vertical(|ui| {
                    ui.image(images.get_image(res), (30., 30.));
                    ui.label(res.to_string());
                    ui.label(amount.to_string());
                });
            }
        });
    }

    // Used to get half of the cost of the machine that gets returned on demolishing a building
    pub fn div(&self, rhs: Amount) -> Self {
        let vec = self
            .vec
            .iter()
            .map(|(r, n)| (*r, n / rhs))
            .collect::<Vec<_>>();
        Self { vec }
    }
}

// Used to quickly compare whether player has enough resources to build a building
impl PartialOrd for ResourceSet {
    fn gt(&self, other: &Self) -> bool {
        // Zip::zip(Zip::from(self.vec), other.vec).all(|x| {x.})
        self.vec
            .iter()
            .zip(other.vec.iter())
            .all(|(x, y)| x.1 > y.1)
    }
    fn ge(&self, other: &Self) -> bool {
        self.vec
            .iter()
            .zip(other.vec.iter())
            .all(|(x, y)| x.1 >= y.1)
    }
    fn le(&self, other: &Self) -> bool {
        self.vec
            .iter()
            .zip(other.vec.iter())
            .all(|(x, y)| x.1 <= y.1)
    }
    fn lt(&self, other: &Self) -> bool {
        self.vec
            .iter()
            .zip(other.vec.iter())
            .all(|(x, y)| x.1 < y.1)
    }
    fn partial_cmp(&self, _other: &Self) -> Option<std::cmp::Ordering> {
        return None;
        // if self.vec.iter().zip(other.vec).all(|(x, y)| x.1 < y.1) { return Ordering::Less;
        // } else {
        //     return Ordering::Greater;
        // }
    }
}

// Maths operations on the set.
// Operator overloading proved too complex for this struct
impl ResourceSet {
    pub fn add_set(&mut self, rhs: &Self) -> () {
        self.vec.iter_mut().enumerate().for_each(|(i, (_, n))| {
            if let Some(x) = rhs.vec.get(i) {
                *n += x.1;
            }
        });
    }
    pub fn sub(&mut self, rhs: &Self) -> () {
        self.vec.iter_mut().enumerate().for_each(|(i, (_, n))| {
            if let Some(x) = rhs.vec.get(i) {
                *n -= x.1;
            }
        });
    }
}

// The plugin that adds all the resources and their systems
pub struct ResourcePlugin;

// A thin wrapper around ResourceSet that is a bevy Resource (aka global state_
#[derive(Resource, PartialEq, Clone, Debug)]
pub struct ResourceState {
    // With low key numbers, vectors are faster than hashmaps, because of the speed of the hash function
    pub resources: ResourceSet,
}

impl ResourceState {
    pub fn add(&mut self, r: ResourceType, amount: Amount) {
        self.resources.add(r, amount)
    }
    pub fn get(&self, r: ResourceType) -> Option<Amount> {
        self.resources.get(r)
    }
}

impl ResourceState {
    pub fn new() -> Self {
        Self {
            resources: ResourceSet {
                vec: vec![
                    (ResourceType::Ore, 100),
                    (ResourceType::Gas, 10),
                    (ResourceType::Crystal, 0),
                ],
            },
        }
    }
}

impl Plugin for ResourcePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ResourceState::new())
            .add_system_set(
                SystemSet::on_update(AppState::InGame)
                    .with_system(resource_generation)
                    .with_system(resource_ui),
            )
            .init_resource::<ResourceImages>()
            .add_startup_system(register_resource_images);
    }
}

// To add a new resource type simply add an enum option here
// Rust will then throw errors everywhere we need to add a case for the enum
#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ResourceType {
    Ore,
    Gas,
    Crystal,
}

// Used for the format! macro, not for ui displaying
impl Display for ResourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match *self {
            ResourceType::Crystal => "Crystal",
            ResourceType::Gas => "Gas",
            ResourceType::Ore => "Ore",
        };
        return f.write_str(s);
    }
}


pub fn resource_ui(
    resources: Res<ResourceState>,
    // mut query: Query<(&ResourceStatus, &mut Text)>,
    resource_images: Res<ResourceImages>,
    mut ctx: ResMut<EguiContext>,
) {
    let w = egui::Window::new("Ore status").show(ctx.ctx_mut(), |ui| {
        resources.resources.display(ui, &resource_images, true);
    });
}

// A resource generator building component
#[derive(Component, Debug, Clone)]
pub struct ResourceGenerator {
    pub resource_type: ResourceType,
    pub amount: Amount,
    pub timer: Timer,
}

impl ResourceGenerator {
    pub fn new(resource_type: ResourceType, amount: Amount, miliseconds: i32) -> Self {
        Self {
            resource_type,
            amount,
            timer: Timer::from_seconds(miliseconds as f32 / 1000.0, TimerMode::Repeating),
        }
    }
}

// Handles adding the appropriate amount of resources for each building that is a generator
pub fn resource_generation(
    mut resource_state: ResMut<ResourceState>,
    mut generators: Query<&mut ResourceGenerator>,
    time: Res<Time>,
) {
    for mut generator in generators.iter_mut() {
        generator.timer.tick(time.delta());
        if generator.timer.finished() {
            resource_state.add(generator.resource_type, generator.amount);

            generator.timer.reset();
        }
    }
}
