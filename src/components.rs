use bevy::prelude::*;
use bevy_inspector_egui::Inspectable;

#[derive(Component, Default)]
pub struct IkGoal;

#[derive(Bundle, Default)]
pub struct IkGoalBundle {
    pub transform: Transform,
    pub global_transform: GlobalTransform,
    pub marker: IkGoal,
}

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

#[derive(Component, Default, Inspectable)]
pub struct Joint {
    #[inspectable(read_only)]
    name: String,
}

impl Joint {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
        }
    }
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

#[derive(Bundle, Default)]
pub struct JointBundle {
    pub joint: Joint,
    pub link: Link,
    pub transform: Transform,
    pub global_transform: GlobalTransform,
}
