mod components;
mod setup_systems;

use bevy::prelude::*;
use bevy_inspector_egui::{widgets::InspectorQuerySingle, InspectorPlugin};
use components::IkGoal;
use setup_systems::{
    setup_armature, setup_camera, setup_goal_assets, setup_joint_assets, setup_joint_visuals,
    spawn_goal,
};

fn main() {
    App::new()
        // plugins
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_plugin(InspectorPlugin::<InspectorQuerySingle<Entity, With<IkGoal>>>::new())
        // setup
        .add_startup_system_to_stage(StartupStage::PreStartup, setup_joint_assets)
        .add_startup_system_to_stage(StartupStage::PreStartup, setup_goal_assets)
        .add_startup_system_to_stage(StartupStage::Startup, setup_armature)
        .add_startup_system_to_stage(StartupStage::Startup, setup_camera)
        .add_startup_system_to_stage(StartupStage::Startup, spawn_goal)
        .add_startup_system_to_stage(StartupStage::PostStartup, setup_joint_visuals)
        .run();
}
