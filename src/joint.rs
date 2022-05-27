use bevy::prelude::*;

/// Marker struct for joints
#[derive(Component, Default)]
pub struct Joint;

/// Link Length describes the length of the rigid link between two joints
#[derive(Component, Default)]
pub struct LinkLength(pub f32);

///
#[derive(Bundle, Default)]
pub struct JointBundle {
    pub joint: Joint,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub link_length: LinkLength,
    // TODO: constraints
}
