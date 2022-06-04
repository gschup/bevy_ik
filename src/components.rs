use bevy::{
    prelude::*,
    utils::{HashMap, HashSet},
};

// Resources

/// The [`ArmatureGraph`] contains information about the [`Bone`] tree.
/// It is updated once per frame. Treat this resource as read-only.
#[derive(Default)]
pub struct ArmatureGraph {
    /// joint ids and their outgoing bones
    pub out_bones: HashMap<u32, HashSet<Entity>>,
    /// joint ids and their parent bone (only a single parent, tree assumption)
    pub in_bone: HashMap<u32, Entity>,
    /// for each bone, contains the base joint
    pub base_joint: HashMap<Entity, u32>,
    /// children joints
    pub joint_children: HashMap<u32, HashSet<u32>>,
    /// parent joint
    pub joint_parent: HashMap<u32, u32>,
}

/// [`IkData`] contains intermediate results of the FABRIK algorithm. Treat this resource as read-only.
#[derive(Default, Debug)]
pub struct IkData {
    /// armature joints and their global positions. A joint is between two bones.
    pub joint_positions: HashMap<u32, Vec3>,
    /// Length of each bone (distance between joints)
    pub bone_lengths: HashMap<Entity, f32>,
    /// for each joint, which children joints do we need info from? (some joints might not have IK goals)
    pub required_positions: HashMap<u32, HashSet<u32>>,
    /// hashmap of joint ids to goal ids
    pub joints_to_goals: HashMap<u32, Entity>,
    /// FABRIK roots - joints defined by not having a parent, or by chain length, or if fixed
    pub roots: HashSet<u32>,
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
