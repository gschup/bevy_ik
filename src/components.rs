use bevy::{prelude::*, utils::HashMap};
use bevy_inspector_egui::Inspectable;

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

// Components
#[derive(Component, Copy, Clone)]
pub struct IkGoal {
    pub target_joint: Entity,
    pub chain_length: u32,
}

#[derive(Component, Default, Inspectable)]
pub struct Joint {
    #[inspectable(read_only)]
    pub name: String,
    pub fixed: bool, // assume parents of fixed joints are also fixed...
}

#[derive(Component, Default, Inspectable)]
pub struct Link {
    #[inspectable(read_only)]
    pub length: f32,
}

impl Link {
    pub fn new(length: f32) -> Self {
        Self { length }
    }
}

// Bundles
#[derive(Bundle, Default)]
pub struct JointBundle {
    pub joint: Joint,
    pub link: Link,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}

#[derive(Bundle)]
pub struct IkGoalBundle {
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub goal: IkGoal,
}
