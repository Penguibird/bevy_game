use bevy::prelude::*;
use bevy::transform::components::Transform;
use bevy_tweening::Lens;

pub(crate) struct RelativeTransformPositionLens {
    pub(crate) previous: Vec3,
    pub(crate) start: Vec3,
    pub(crate) end: Vec3,
}

impl Lens<Transform> for RelativeTransformPositionLens {
    fn lerp(&mut self, target: &mut Transform, ratio: f32) {
        let value = self.start + (self.end - self.start) * ratio;
        target.translation += value - self.previous;
        self.previous = value;
    }
}

pub(crate) struct RelativeTransformRotateXLens {
    pub(crate) previous: f32,
    pub(crate) start: f32,
    pub(crate) end: f32,
}

impl Lens<Transform> for RelativeTransformRotateXLens {
    fn lerp(&mut self, target: &mut Transform, ratio: f32) {
        let angle = (self.end - self.start).mul_add(ratio, self.start);
        target.rotate_local_x(angle - self.previous);
        self.previous = angle;
    }
}
