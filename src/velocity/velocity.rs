use bevy::{prelude::*, sprite::collide_aabb::collide};

use super::collisions::CollisionEvent;

#[derive(Component)]
pub struct Velocity {
    pub vector: Vec3,
    pub speed: f32,
}

impl Default for Velocity {
    fn default() -> Self {
        Velocity {
            vector: Vec3::ZERO,
            speed: 0.001,
        }
    }
}
pub fn update_positions(
    mut query: Query<(&mut Velocity, &mut Transform, Entity)>,
    time: Res<Time>,
    mut ev: EventReader<CollisionEvent>,
) {
    for (mut v, mut t, ent) in query.iter_mut() {
        if ev.iter().any(|e| e.entities.contains(&ent)) {
            // t.translation +=
            //     v.vector.normalize() * -10.0 * v.speed * time.delta().as_millis() as f32;
            v.vector = Vec3::ZERO;
            v.speed = 0.0;
        } else {
            let new_translation =
                t.translation + v.vector.normalize() * v.speed * time.delta().as_millis() as f32;
            t.translation = new_translation;
        }
    }
}
