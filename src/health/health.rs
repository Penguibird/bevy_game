use bevy::prelude::*;

#[derive(Component)]
pub struct Health {
    pub max_hp: i32,
    pub hp: i32,
    pub dead_for_timer: Timer,
}

impl Health {
    pub fn new(hp: i32) -> Health {
        let mut h = Health {
            max_hp: hp,
            hp,
            dead_for_timer: Timer::from_seconds(1.0, TimerMode::Once),
        };
        h.dead_for_timer.pause();
        return h;
    }
}

pub struct DeathEvent {
    pub entity: Entity,
    pub killer: Option<Entity>,
}

pub fn death_timers(
    time: Res<Time>,
    mut query: Query<&mut Health>,
    mut ev: EventReader<DeathEvent>,
) {
    for mut h in query.iter_mut() {
        h.dead_for_timer.tick(time.delta());
    }
    for e in ev.iter() {
        if let Ok(mut x) = query.get_mut(e.entity) {
            x.dead_for_timer.unpause();
            x.dead_for_timer.reset();
        }
    }
}
