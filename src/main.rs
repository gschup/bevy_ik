mod armatures;
mod components;
mod fabrik;
mod setup_systems;

use armatures::load_chain_armature;
use bevy::prelude::*;
use components::{ArmatureGraph, IkSolverData};
use fabrik::{
    apply_joint_positions, cache_joint_data, compute_joint_positions, create_armature_tree,
};
use setup_systems::{
    rotate_goals, setup_camera, setup_goal_assets, setup_joint_assets, setup_joint_visuals,
    spawn_goal,
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
        .init_resource::<IkSolverData>()
        .add_plugins(DefaultPlugins)
        // setup
        .add_startup_system_to_stage(StartupStage::PreStartup, setup_joint_assets)
        .add_startup_system_to_stage(StartupStage::PreStartup, setup_goal_assets)
        .add_startup_system_to_stage(StartupStage::Startup, load_chain_armature)
        .add_startup_system_to_stage(StartupStage::Startup, setup_camera)
        .add_startup_system_to_stage(StartupStage::PostStartup, spawn_goal)
        .add_startup_system_to_stage(StartupStage::PostStartup, setup_joint_visuals)
        .add_system(rotate_goals)
        .add_system(create_armature_tree)
        .add_system(cache_joint_data.after(create_armature_tree))
        .add_system(compute_joint_positions.after(cache_joint_data))
        .add_system(apply_joint_positions.after(compute_joint_positions))
        .run();
}
