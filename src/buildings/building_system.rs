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
    ui::{
        error_info::ErrorEvent,
        ui::{UIMode, UIState},
    }, main_base::main_base::MainBaseComponent,
};

use super::{
    building_bundles::Building,
    resources::{ResourceSet, ResourceState},
};

use super::grid::{Grid, SQUARE_SIZE};

// This modules handles the user actions related to construction/demolishing of buildings

// This is the square which highlights the currently hovered spot
// It is either green or red
// Marker component struct, no info specific to it.
#[derive(Component)]
pub struct HighlightSquare {}

// Hide the square if we're not in a ui_mode where it should be visible
pub fn hide_highlight_square(
    query: Query<Entity, With<HighlightSquare>>,
    ui_state: Res<UIState>,
    mut commands: Commands,
) {
    match ui_state.mode {
        // In these modes it should be kept visible, i.e. do nothing to it
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

// We can't use the worldclick event here because we need the hover position as well
// This system handles positioning of the highlight square on the board as well as actually clicking to buid/destroy any building
pub fn building_system(
    mut ctx: ResMut<EguiContext>,
    mut resources: ResMut<ResourceState>,
    // The info necessary to get the world positio from mouse position
    mbutton: Res<Input<MouseButton>>,
    windows: Res<Windows>,
    mut query_set: ParamSet<(
        Query<(&PanOrbitCamera, &Transform, &Projection)>,
        Query<(&mut Transform, &mut Handle<StandardMaterial>), With<HighlightSquare>>,
    )>,
    // So that we know how many resources to refund for the destruction of the building
    resource_cost_query: Query<
        (&ResourceSet, Entity),
        (Without<PanOrbitCamera>, Without<HighlightSquare>, Without<MainBaseComponent>),
    >,
    // Query for the main base so that we can't destroy it
    // We need to exclude the other components to prevent query overlap
    main_base: Query<&MainBaseComponent, (Without<HighlightSquare>, Without<PanOrbitCamera>)>,
    ui_state: Res<UIState>,
    // Technically we could predefine both the mesh and the material for the square, but we don't recreate the square often enough for this to be a significant memory leak
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,

    mut grid: ResMut<Grid>,
    mut commands: Commands,
    mut death_events: EventWriter<DeathEvent>,

    // Notify the player that they cannot build
    mut error_events: EventWriter<ErrorEvent>,
) {
    let cam_query = query_set.p0();
    let (cam, transform, proj) = cam_query.single();

    // Don't do anything if the mouse is over a menu.
    if ctx.ctx_mut().is_pointer_over_area() {
        return;
    }

    // Don't do anything if we're not in an appropriate UI state
    match &ui_state.mode {
        UIMode::BuildingDefensive(_) | UIMode::BuildingResources(_) | UIMode::Destroying => {}
        _ => {
            return;
        }
    };

    // Get the values to get the world position from the mouse
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

    // Get the square if it's alive, if not spawn it.
    if let Ok((mut x, mut m)) = highlight_square_query.get_single_mut() {
        x.translation = Grid::get_plane_pos(point);
        if grid.is_square_blocked(point) {
            *m = red;
        } else {
            *m = blue;
        }
    } else {
        // Spawn the square.
        // If we just spawn it this tick, we actually don't do anything with it,
        // but since we update it immediately the tick afterwards, it doesn't matter.
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

    // Handle building/destroying
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
                    } else {
                        error_events.send(ErrorEvent::NotEnoughResources);
                    }
                } else {
                    error_events.send(ErrorEvent::SpaceOccupied);
                }
            }
            UIMode::Destroying => {
                let entity = grid.get_entity(point);
                if let Some(entity) = entity {
                    // If the main base query can find the entity this means the entity is the main base
                    // We want to prevent deleting that
                    if main_base.get(*entity).is_ok() {
                        error_events.send(ErrorEvent::CantDestroyYourOwnBase);
                        return;
                    }

                    if let Ok((cost, ..)) = resource_cost_query.get(*entity) {
                        resources.resources.add_set(&cost.div(2));
                    }
                    death_events.send(DeathEvent {
                        entity: *entity,
                        killer: None,
                    })
                } else {
                    error_events.send(ErrorEvent::NothingToDestroy)
                }
            }
            _ => {
                return;
            }
        };
    };
}
