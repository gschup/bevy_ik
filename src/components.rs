use bevy::{prelude::*, utils::HashMap};

// Resources

/// The [`ArmatureGraph`] contains information about the [`Bone`] tree. Treat this resource as read-only.
#[derive(Default)]
pub struct ArmatureGraph {
    pub bone_children: HashMap<Entity, Vec<Entity>>,
    pub bone_parent: HashMap<Entity, Entity>,
}

/// [`IkData`] contains intermediate results of the FABRIK algorithm. Treat this resource as read-only.
#[derive(Default)]
pub struct IkData {
    /// computed new positions for bones
    pub positions: HashMap<Entity, Vec3>,
    /// for each bone, which children do we need info from? (some leaves might not have IK goals)
    pub required_positions: HashMap<Entity, Vec<Entity>>,
    /// hashmap of bone ids to goal ids
    pub bones_to_goals: HashMap<Entity, Entity>,
    /// FABRIK roots - bones defined by not having a parent, or by chain length, or if fixed
    pub roots: Vec<Entity>,
}

pub struct IkSettings {
    pub goal_tolerance: f32,
    pub max_iterations: u32,
}

#[derive(Component, Copy, Clone)]
pub struct IkGoal {
    pub target_bone: Entity,
    pub chain_length: u32,
}

#[derive(Component, Default)]
pub struct Bone {
    pub name: String,
}

// Bundles
#[derive(Bundle, Default)]
pub struct BoneBundle {
    pub bone: Bone,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

#[derive(Bundle)]
pub struct IkGoalBundle {
    pub goal: IkGoal,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}
