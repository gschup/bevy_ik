mod components;
mod systems;

use bevy::prelude::*;
use bevy_ik::InverseKinematicsPlugin;
use systems::*;

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(ClearColor(Color::ALICE_BLUE))
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 0.3,
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(InverseKinematicsPlugin::default())
        // setup
        .add_startup_system_to_stage(StartupStage::Startup, setup_joint_assets)
        .add_startup_system_to_stage(StartupStage::Startup, setup_goal_assets)
        .add_startup_system_to_stage(StartupStage::Startup, setup_fork_armature)
        .add_startup_system_to_stage(StartupStage::Startup, setup_camera)
        // spawn_goals and setup_joint_visuals require armature to be spawned
        .add_startup_system_to_stage(StartupStage::PostStartup, spawn_goals)
        .add_startup_system_to_stage(StartupStage::PostStartup, setup_joint_visuals)
        // move the goals around so it looks cool
        .add_system(rotate_goals)
        .run();
}
