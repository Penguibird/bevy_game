use bevy::{prelude::*, utils::HashSet};

use crate::{
    cameras::{
        get_world_point_from_screen::get_plane_point_from_mouse_pos,
        orbit_camera::{get_primary_window_size, PanOrbitCamera},
    },
    ui::ui::{UIMode, UIState},
};

use super::{building_bundles::Building, resources::ResourceState};

#[derive(Resource, Debug, Clone)]
pub struct Grid {
    pub blocked_squares: HashSet<(i8, i8)>,
}

pub const SQUARE_SIZE: f32 = 3.0;
impl Grid {
    pub fn new() -> Self {
        Self {
            blocked_squares: HashSet::new(),
        }
    }

    pub fn get_square_index(point: Vec3) -> (i8, i8) {
        let x = (point.x / SQUARE_SIZE).floor() as i8;
        let y = (point.z / SQUARE_SIZE).floor() as i8;
        return (x, y);
    }
    pub fn get_plane_pos(point: Vec3) -> Vec3 {
        let t = Self::get_square_index(point);
        return Vec3::new(
            (t.0 as f32 + 0.5) * SQUARE_SIZE,
            0.01,
            (t.1 as f32 + 0.5) * SQUARE_SIZE,
        );
    }

    pub fn is_square_blocked(&self, point: Vec3) -> bool {
        self.blocked_squares
            .contains(&Self::get_square_index(point))
    }

    pub fn get_square_info(&self, point: Vec3) -> (bool, i8, i8) {
        let t = &Self::get_square_index(point);
        return (self.blocked_squares.contains(t), t.0, t.1);
    }

    // pub fn getPlane(point: Vec3) -> Plane {
    //   let (x,y) = Grid::get_square_index(point);

    //   return Plane {

    //   }
    // }

    pub fn block_square(&mut self, point: (i8, i8)) {
        self.blocked_squares.insert(point);
    }
    pub fn block_square_vec3(&mut self, point: Vec3) {
        self.block_square(Grid::get_square_index(point));
    }
}

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
                b.clone().build(&mut commands, point);
                grid.block_square_vec3(point);
            }
        }
    };
}
