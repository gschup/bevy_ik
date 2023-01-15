use bevy::prelude::*;

// Resources
#[derive(Resource)]
pub struct BoneVizHandles {
    pub joint_mesh_handle: Handle<Mesh>,
    pub link_mesh_handle: Handle<Mesh>,
    pub joint_material_handle: Handle<StandardMaterial>,
    pub link_material_handle: Handle<StandardMaterial>,
}

#[derive(Resource)]
pub struct GoalVizHandles {
    pub goal_mesh_handle: Handle<Mesh>,
    pub goal_material_handle: Handle<StandardMaterial>,
}
