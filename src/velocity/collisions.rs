use bevy::{prelude::*, sprite::collide_aabb::collide};

pub enum ColliderType {
    Default,
    Alien,
}
impl ColliderType {
    fn can_collide_with(&self, rhs: &ColliderType) -> bool {
        use ColliderType::*;
        match (self, rhs) {
            (Alien, Alien) => false,
            _ => true,
        }
    }
}

#[derive(Component)]
pub struct CollisionComponent {
    pub collider_type: ColliderType,
}

impl Default for CollisionComponent {
    fn default() -> Self {
        CollisionComponent {
            collider_type: ColliderType::Default,
        }
    }
}

trait RemoveY {
    fn remove_y(&self) -> Vec2;
    fn to2d(&self) -> Vec3;
}
impl RemoveY for Vec3 {
    fn remove_y(&self) -> Vec2 {
        Vec2::new(self.x, self.z)
    }
    fn to2d(&self) -> Vec3 {
        Vec3::new(self.x, self.z, 0.0)
    }
}

pub struct CollisionEvent {
    pub entities: [Entity; 2],
}

pub fn check_for_collisions(
    query: Query<(&Transform, Entity, &CollisionComponent)>,
    mut ev: EventWriter<CollisionEvent>,
) {
    for (entity1_t, entity1_id, c1) in query.iter() {
        for (entity2_t, entity2_id, c2) in query.iter() {
            if collide(
                entity1_t.translation.to2d(),
                entity1_t.scale.remove_y(),
                entity2_t.translation.to2d(),
                entity2_t.scale.remove_y(),
            )
            .is_some()
                && entity1_id != entity2_id
                && c1.collider_type.can_collide_with(&c2.collider_type)
            {
                println!(
                    " COllision at {:?} {:?} {:?}   {:?} ",
                    entity1_t.translation.to2d(),
                    entity1_t.scale.remove_y(),
                    entity2_t.translation.to2d(),
                    entity2_t.scale.remove_y(),
                );
                ev.send(CollisionEvent {
                    entities: [entity1_id, entity2_id],
                });
            }
        }
    }
}
