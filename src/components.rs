use bevy::{prelude::*, utils::HashMap};

// Resources
pub struct JointVizHandles {
    pub joint_mesh_handle: Handle<Mesh>,
    pub link_mesh_handle: Handle<Mesh>,
    pub joint_material_handle: Handle<StandardMaterial>,
    pub link_material_handle: Handle<StandardMaterial>,
}

pub struct GoalVizHandles {
    pub goal_mesh_handle: Handle<Mesh>,
    pub goal_material_handle: Handle<StandardMaterial>,
}

#[derive(Default)]
pub struct ArmatureGraph {
    pub joint_children: HashMap<Entity, Vec<Entity>>,
    pub joint_parent: HashMap<Entity, Entity>,
}

#[derive(Default)]
pub struct IkSolverData {
    // computed new positions for joints
    pub positions: HashMap<Entity, Vec3>,
    // for each joint, which children do we need info from? (some leaves might not have IK goals)
    pub required_positions: HashMap<Entity, Vec<Entity>>,
    // hashmap of joint_ids to goal_ids
    pub goals_by_joints: HashMap<Entity, Entity>,
    // FABRIK roots - joints defined by not having a parent, or by chain length, or if fixed
    pub roots: Vec<Entity>,
}

// Components
#[derive(Component, Copy, Clone)]
pub struct IkGoal {
    pub goal_id: u32,
    pub target_joint: Entity,
    pub chain_length: u32,
}

#[derive(Component, Default)]
pub struct Joint {
    pub name: String,
    pub fixed: bool, // assume parents of fixed joints are also fixed...
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
