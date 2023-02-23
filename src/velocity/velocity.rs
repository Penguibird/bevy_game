use bevy::prelude::*;

#[derive(Component, )]
pub struct Velocity {
  pub vector: Vec3,
  pub speed: f32,
}

impl Default for Velocity {
  fn default() -> Self {
      Velocity { vector: Vec3::ZERO, speed: 0.001 }
  }
}

pub fn update_positions(mut query: Query<(&Velocity, &mut Transform)>, time: Res<Time>) {
    for (v, mut t) in query.iter_mut() {
        t.translation += v.vector.normalize() * v.speed * time.delta().as_millis() as f32;
    }
}
