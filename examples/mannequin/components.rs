use bevy::scene::InstanceId;

// Resources
#[derive(Default)]
pub struct MannequinInstance(pub Option<InstanceId>);
