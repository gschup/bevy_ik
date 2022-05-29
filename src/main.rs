mod armatures;
mod components;
mod fabrik;
mod setup_systems;

use armatures::load_chain_armature;
use bevy::prelude::*;
use bevy_inspector_egui::{widgets::InspectorQuerySingle, InspectorPlugin};
use components::{ArmatureGraph, IkGoal};
use fabrik::{apply_ik_goal, create_armature_graph};
use setup_systems::{
    setup_camera, setup_goal_assets, setup_joint_assets, setup_joint_visuals, spawn_goal,
};

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(ClearColor(Color::ALICE_BLUE))
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 0.3,
        })
        .init_resource::<ArmatureGraph>()
        .add_plugins(DefaultPlugins)
        .add_plugin(InspectorPlugin::<InspectorQuerySingle<Entity, With<IkGoal>>>::new())
        // setup
        .add_startup_system_to_stage(StartupStage::PreStartup, setup_joint_assets)
        .add_startup_system_to_stage(StartupStage::PreStartup, setup_goal_assets)
        .add_startup_system_to_stage(StartupStage::Startup, load_chain_armature)
        .add_startup_system_to_stage(StartupStage::Startup, setup_camera)
        .add_startup_system_to_stage(StartupStage::PostStartup, spawn_goal)
        .add_startup_system_to_stage(StartupStage::PostStartup, setup_joint_visuals)
        .add_system(create_armature_graph)
        .add_system(apply_ik_goal.after(create_armature_graph))
        .run();
}
