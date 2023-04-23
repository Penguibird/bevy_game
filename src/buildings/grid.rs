use std::cmp::Ordering;

use bevy::{prelude::*, utils::HashMap};

// Each building takes up a square.
pub const SQUARE_SIZE: f32 = 3.0;

// The game state - tracks which squares are blocked and by which entity.
// Also contains static methods to calculate square centers etc.
#[derive(Resource, Debug, Clone)]
pub struct Grid {
    // Maps the index of the square to the entity occupying it
    pub blocked_squares: HashMap<(i8, i8), Entity>,

    // The dimensions of the circle which encompasses all the buildings on the map - aka the player's base.
    // This is used for spawning aliens,so that they don't spawn in the middle of the base and the player has time to react to them
    pub base_center: Vec3,
    pub center_radius: f32,
}

impl Grid {
    pub fn new() -> Self {
        Self {
            blocked_squares: HashMap::new(),
            base_center: Vec3::splat(0.),
            center_radius: 5.,
        }
    }

    // Get the index of the square of the point
    pub fn get_square_index(point: Vec3) -> (i8, i8) {
        let x = (point.x / SQUARE_SIZE).floor() as i8;
        let y = (point.z / SQUARE_SIZE).floor() as i8;
        return (x, y);
    }

    // Returns the center of the square that the point is in
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
            .contains_key(&Self::get_square_index(point))
    }

    pub fn _get_square_info(&self, point: Vec3) -> (bool, i8, i8) {
        let t = &Self::get_square_index(point);
        return (self.blocked_squares.contains_key(t), t.0, t.1);
    }
    pub fn get_entity(&self, point: Vec3) -> Option<&Entity> {
        let t = &Self::get_square_index(point);
        return self.blocked_squares.get(t);
    }

    // Returns the number of buildings built
    pub fn get_square_count(&self) -> usize {
        self.blocked_squares.len()
    }

    // We update the base every time something is built, because this is a somewhat expensive computation
    // and we need the result every tick (for alien spawning)
    // Therefore we store it here and update it every time a building gets added/destroyed
    pub fn update_base(&mut self) {
        let points = self.blocked_squares.keys();

        let mut min_x: f32 = 0.;
        let mut max_x: f32 = 0.;
        let mut min_y: f32 = 0.;
        let mut max_y: f32 = 0.;
        for (x, y) in points {
            let x = *x as f32;
            let y = *y as f32;
            if x <= min_x {
                min_x = x;
            }
            if x >= max_x {
                max_x = x;
            }
            if y <= min_y {
                min_y = y;
            }
            if y >= max_y {
                max_y = y;
            }
        }

        let center = Vec3::new((min_x + max_x) / 2., 0., (min_y + max_y) / 2.);
        let center_vec2 = Vec2::new(center.x, center.z);

        let dist = self
            .blocked_squares
            .keys()
            .map(|p| {
                Vec2::new(p.0 as f32, p.1 as f32)
                    .distance(center_vec2)
                    .abs()
            })
            .max_by(|a, b| a.total_cmp(b));

        self.base_center = center * SQUARE_SIZE;
        self.center_radius = dist.unwrap_or(0.) + 5.;
        self.center_radius *= SQUARE_SIZE;
    }

    // Add a building to the grid
    // Called everytime a building is constructed
    pub fn block_square(&mut self, point: (i8, i8), entity: Entity) {
        self.blocked_squares.insert(point, entity);
        self.update_base();
    }

    pub fn block_square_vec3(&mut self, point: Vec3, entity: Entity) {
        self.block_square(Grid::get_square_index(point), entity);
    }
    
    // Used during building destruction
    pub fn unblock_square_vec3(&mut self, point: Vec3) -> Option<Entity> {
        let e = self.blocked_squares.remove(&Grid::get_square_index(point));
        self.update_base();
        return e;
    }
}
