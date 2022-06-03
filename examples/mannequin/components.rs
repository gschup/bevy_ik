use bevy::{prelude::*, scene::InstanceId};

// Resources
#[derive(Default)]
pub struct MannequinInstance(pub Option<InstanceId>);
pub struct GoalVizHandles {
    pub goal_mesh_handle: Handle<Mesh>,
    pub goal_material_handle: Handle<StandardMaterial>,
}
