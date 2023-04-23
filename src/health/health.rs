use bevy::prelude::*;

#[derive(Component, Debug, Clone)]
pub struct Health {
    pub max_hp: i32,
    pub hp: i32,
    // Started on the actual death event, once it runs out, the entity will be cleaned up and despawned
    // This allows us to have death animations/dead bodies for a while even after the death event
    pub dead_for_timer: Timer,
    // Used to only play death sound once per death because the death event can be triggered multiple times per tick
    pub death_sound_played: bool,
}

impl Health {
    pub fn new(hp: i32) -> Health {
        let mut h = Health {
            max_hp: hp,
            hp,
            dead_for_timer: Timer::from_seconds(1.0, TimerMode::Once),
            death_sound_played: false,
        };
        h.dead_for_timer.pause();
        return h;
    }
}

// Sent when the entity dies
pub struct DeathEvent {
    pub entity: Entity,
    // For possible statistics implementations etc
    pub killer: Option<Entity>,
}

// Ticks and starts the alien death timers
// so that we don't have to remember to tick the death timer every time we check it
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
            x.dead_for_timer.reset();
            x.dead_for_timer.unpause();
        }
    }
}
