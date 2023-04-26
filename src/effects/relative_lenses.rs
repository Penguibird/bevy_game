use bevy::prelude::*;
use bevy::transform::components::Transform;
use bevy_tweening::Lens;

// The default lenses from bevy_tweening don't take into consideration the current transformation
// This means that the speeder (for example) which has its own rotation based on where it's firing, 
// would always have the entire Transform component replaced.
// The relative lenses only change the component they're actually supposed to change, mutating the transform, rather than replacing it

// A lens in bevy_tweening is a struct with a function that dictates what velues should be transformed
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
