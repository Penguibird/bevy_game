use bevy::{
    prelude::*,
    utils::{HashMap, HashSet},
};
use bevy_egui::EguiContext;

use crate::{
    cameras::{
        get_world_point_from_screen::get_plane_point_from_mouse_pos,
        pan_camera::{get_primary_window_size, PanOrbitCamera},
    },
    health::health::DeathEvent,
    ui::ui::{UIMode, UIState},
};

use super::{
    building_bundles::Building,
    resources::{ResourceSet, ResourceState},
};

use super::grid::{Grid, SQUARE_SIZE};
#[derive(Component)]
pub struct HighlightSquare {}

pub fn hide_highlight_square(
    query: Query<Entity, With<HighlightSquare>>,
    ui_state: Res<UIState>,
    mut commands: Commands,
) {
    match ui_state.mode {
        UIMode::BuildingDefensive(_) | UIMode::BuildingResources(_) | UIMode::Destroying => {}
        UIMode::Panning => {
            if let Ok(e) = query.get_single() {
                if let Some(e) = commands.get_entity(e) {
                    e.despawn_recursive();
                }
            };
        }
    }
}

pub fn building_system(
    mut resources: ResMut<ResourceState>,
    mbutton: Res<Input<MouseButton>>,
    mut ctx: ResMut<EguiContext>,
    windows: Res<Windows>,
    mut query_set: ParamSet<(
        Query<(&PanOrbitCamera, &Transform, &Projection)>,
        Query<(&mut Transform, &mut Handle<StandardMaterial>), With<HighlightSquare>>,
    )>,
    // So that we know how many resources to refund for the destruction of the building
    resource_cost_query: Query<
        (&ResourceSet, Entity),
        (Without<PanOrbitCamera>, Without<HighlightSquare>),
    >,
    ui_state: Res<UIState>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut grid: ResMut<Grid>,
    mut commands: Commands,
    mut death_events: EventWriter<DeathEvent>,
) {
    let cam_query = query_set.p0();
    let (cam, transform, proj) = cam_query.single();

    if ctx.ctx_mut().is_pointer_over_area() {
        return;
    }

    match &ui_state.mode {
        UIMode::BuildingDefensive(_) | UIMode::BuildingResources(_) | UIMode::Destroying => {}
        _ => {
            return;
        }
    };

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
        match &ui_state.mode {
            UIMode::BuildingDefensive(Some(b)) | UIMode::BuildingResources(Some(b)) => {
                if !grid.is_square_blocked(point) {
                    if b.cost <= resources.resources {
                        resources.resources.sub(&b.cost);
                        let e = b.clone().build(&mut commands, Grid::get_plane_pos(point));
                        if let Some(e) = e {
                            grid.block_square_vec3(point, e);
                        }
                    }
                }
            }
            UIMode::Destroying => {
                let entity = grid.get_entity(point);
                if let Some(entity) = entity {
                    if let Ok((cost, ..)) = resource_cost_query.get(*entity) {
                        resources.resources.add_set(&cost.div(2));
                    }
                    death_events.send(DeathEvent {
                        entity: *entity,
                        killer: None,
                    })
                }
            }
            _ => {
                return;
            }
        };
    };
}
