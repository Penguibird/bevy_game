use bevy_egui::{egui, EguiContext, EguiPlugin};

use bevy::prelude::*;

type Amount = u16;
#[derive(PartialEq, Clone, Debug)]
pub struct ResourceSet {
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
}

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

impl ResourceSet {
    pub fn sub(&mut self, rhs: &Self) -> () {
        self.vec.iter_mut().enumerate().for_each(|(i, (_, n))| {
            if let Some(x) = rhs.vec.get(i) {
                *n -= x.1;
            }
        });
        // .zip(rhs.vec)
        // .map(|(x, y)| (x.0, x.1 - y.1))
        // .collect::<Vec<_>>();
    }
}

pub struct ResourcePlugin;

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
                    (ResourceType::Ore, 0),
                    (ResourceType::Gas, 0),
                    (ResourceType::Crystal, 0),
                ],
            },
        }
    }
}

impl Plugin for ResourcePlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(ResourceState::new())
            .add_system(resource_generation)
            .add_system(resource_ui);
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum ResourceType {
    Ore,
    Gas,
    Crystal,
}

impl ToString for ResourceType {
    fn to_string(&self) -> String {
        match *self {
            ResourceType::Crystal => "Crystal",
            ResourceType::Gas => "Gas",
            ResourceType::Ore => "Ore",
        }
        .to_string()
    }
}

#[derive(Component, Clone, Copy, PartialEq, Debug)]
pub struct ResourceStatus {
    resource_type: ResourceType,
}

pub fn resource_ui(
    resources: Res<ResourceState>,
    // mut query: Query<(&ResourceStatus, &mut Text)>,
    mut ctx: ResMut<EguiContext>,
) {
    egui::Window::new("Ore status").show(ctx.ctx_mut(), |ui| {
        ui.horizontal(|ui| {
            resources.resources.vec.iter().for_each(|(r, n)| {
                ui.vertical(|ui| {
                    ui.label(r.to_string());
                    ui.label(n.to_string());
                });
            })
        })
    });
}

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
