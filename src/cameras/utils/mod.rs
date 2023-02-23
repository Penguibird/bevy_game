use bevy::{
    input::{keyboard::KeyboardInput, ButtonState},
    prelude::*,
};

pub fn get_keybd_vec(ev_keybd: &mut EventReader<KeyboardInput>) -> Option<Vec2> {
    let mut key_vec: Option<Vec2> = None;
    if !ev_keybd.is_empty() {
        for e in ev_keybd.iter() {
            if e.state != ButtonState::Pressed {
                continue;
            }
            if let Some(key) = e.key_code {
                key_vec = match key {
                    KeyCode::D | KeyCode::Right => Some(Vec2::new(1.0, 0.0)),
                    KeyCode::A | KeyCode::Left => Some(Vec2::new(-1.0, 0.0)),
                    KeyCode::W | KeyCode::Up => Some(Vec2::new(0.0, -1.0)),
                    KeyCode::S | KeyCode::Down => Some(Vec2::new(0.0, 1.0)),
                    _ => None,
                }
            }
        }
    }

    return key_vec;
}
