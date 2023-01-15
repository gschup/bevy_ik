use bevy::{prelude::*, scene::InstanceId};

// Resources
#[derive(Default, Resource)]
pub struct MannequinInstance(pub Option<InstanceId>);

#[derive(Resource)]
pub struct GoalVizHandles {
    pub goal_mesh_handle: Handle<Mesh>,
    pub goal_material_handle: Handle<StandardMaterial>,
}
