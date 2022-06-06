mod components;
mod systems;

use bevy::prelude::*;
use bevy_ik::InverseKinematicsPlugin;
use systems::*;

const TARGETS: [(&str, u32); 1] = [("left_hand", 3) /*, ("right_hand", 3)*/];
const LINK_LENGTHS: [f32; 3] = [3.0, 2.0, 1.0];
const GOAL_INIT: [f32; 3] = [0.0, 4.0, 2.0];

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
        .add_startup_system_to_stage(StartupStage::Startup, setup_bone_assets)
        .add_startup_system_to_stage(StartupStage::Startup, setup_goal_assets)
        .add_startup_system_to_stage(StartupStage::Startup, setup_fork_armature)
        .add_startup_system_to_stage(StartupStage::Startup, setup_camera)
        // spawn_goals and setup_bone_visuals require armature to be spawned
        .add_startup_system_to_stage(StartupStage::PostStartup, setup_goals)
        .add_startup_system_to_stage(StartupStage::PostStartup, setup_bone_visuals)
        // move the goals around so it looks cool
        .add_system(rotate_goals)
        .run();
}
