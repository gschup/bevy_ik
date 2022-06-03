mod components;
mod systems;

use bevy::prelude::*;
use bevy_ik::InverseKinematicsPlugin;
use bevy_inspector_egui::WorldInspectorPlugin;
use components::MannequinInstance;
use systems::*;

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum AppState {
    Loading,
    Running,
}

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(ClearColor(Color::ALICE_BLUE))
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 0.3,
        })
        .init_resource::<MannequinInstance>()
        .add_plugins(DefaultPlugins)
        .add_plugin(WorldInspectorPlugin::new())
        .add_plugin(InverseKinematicsPlugin::default())
        .add_state(AppState::Loading)
        .add_system_set(
            SystemSet::on_enter(AppState::Loading)
                .with_system(setup_mannequin)
                .with_system(setup_camera)
                .with_system(setup_goal_assets),
        )
        .add_system_set(SystemSet::on_update(AppState::Loading).with_system(tag_mannequin))
        .add_system_set(SystemSet::on_enter(AppState::Running).with_system(setup_goal))
        .add_system_set(SystemSet::on_update(AppState::Running).with_system(rotate_goal))
        .run();
}
