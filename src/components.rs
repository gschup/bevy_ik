use bevy::{prelude::*, utils::HashMap};

// Resources

/// The [`ArmatureGraph`] contains information about the [`Joint`] tree. Treat this resource as read-only.
#[derive(Default)]
pub struct ArmatureGraph {
    pub joint_children: HashMap<Entity, Vec<Entity>>,
    pub joint_parent: HashMap<Entity, Entity>,
}

/// [`IkData`] contains intermediate results of the FABRIK algorithm. Treat this resource as read-only.
#[derive(Default)]
pub struct IkData {
    /// computed new positions for joints
    pub positions: HashMap<Entity, Vec3>,
    /// for each joint, which children do we need info from? (some leaves might not have IK goals)
    pub required_positions: HashMap<Entity, Vec<Entity>>,
    /// hashmap of joint_ids to goal_ids
    pub goals_by_joints: HashMap<Entity, Entity>,
    /// FABRIK roots - joints defined by not having a parent, or by chain length, or if fixed
    pub roots: Vec<Entity>,
}

pub struct IkSettings {
    pub goal_tolerance: f32,
    pub max_iterations: u32,
}

#[derive(Component, Copy, Clone)]
pub struct IkGoal {
    pub target_joint: Entity,
    pub chain_length: u32,
}

#[derive(Component, Default)]
pub struct Joint {
    pub name: String,
}

// Bundles
#[derive(Bundle, Default)]
pub struct JointBundle {
    pub joint: Joint,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

#[derive(Bundle)]
pub struct IkGoalBundle {
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub goal: IkGoal,
}
