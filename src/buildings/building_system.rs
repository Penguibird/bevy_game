use bevy::{
    prelude::*,
    utils::{HashMap, HashSet},
};

use crate::{
    cameras::{
        get_world_point_from_screen::get_plane_point_from_mouse_pos,
        pan_camera::{get_primary_window_size, PanOrbitCamera},
    },
    ui::ui::{UIMode, UIState},
};

use super::{building_bundles::Building, resources::ResourceState};

use super::grid::{Grid, SQUARE_SIZE};
#[derive(Component)]
pub struct HighlightSquare {}

pub fn building_system(
    mut resources: ResMut<ResourceState>,
    mbutton: Res<Input<MouseButton>>,
    windows: Res<Windows>,
    mut query_set: ParamSet<(
        Query<(&PanOrbitCamera, &Transform, &Projection)>,
        Query<(&mut Transform, &mut Handle<StandardMaterial>), With<HighlightSquare>>,
    )>,
    ui_state: Res<UIState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut grid: ResMut<Grid>,
    mut commands: Commands,
) {
    let cam_query = query_set.p0();
    let (cam, transform, proj) = cam_query.single();

    let b: &Building;
    if let UIMode::BuildingDefensive(Some(b_)) = &ui_state.mode {
        b = b_;
    } else if let UIMode::BuildingResources(Some(b_)) = &ui_state.mode {
        b = b_;
    } else {
        return;
    }

    let screen_size_half = get_primary_window_size(&windows) / 2.0;
    let m_pos = windows
        .get_primary()
        .unwrap()
        .cursor_position()
        .unwrap_or(Vec2 { x: 0.0, y: 0.0 });
    let point =
        get_plane_point_from_mouse_pos(m_pos, screen_size_half, proj, cam.zoom_level, transform);

    let blue = materials.add(Color::rgba(0.0, 0.0, 0.9, 0.2).into());
    let red = materials.add(Color::rgba(0.7, 0.0, 0.0, 0.2).into());
    let mut highlight_square_query = query_set.p1();

    if let Ok((mut x, mut m)) = highlight_square_query.get_single_mut() {
        x.translation = Grid::get_plane_pos(point);
        if grid.is_square_blocked(point) {
            *m = red;
        } else {
            *m = blue;
        }
    } else {
        let x = (
            HighlightSquare {},
            PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Plane { size: SQUARE_SIZE })),
                material: blue,
                ..default()
            },
        );
        commands.spawn(x);
    }

    if mbutton.just_pressed(MouseButton::Left) {
        if !grid.is_square_blocked(point) {
            if b.cost <= resources.resources {
                resources.resources.sub(&b.cost);
                let e = b.clone().build(&mut commands, point);
                if let Some(e) = e {
                    grid.block_square_vec3(point, e);
                }
            }
        }
    };
}
