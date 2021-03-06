//! bevy_ik is a inverse kinematics solver as a bevy plugin.
#![forbid(unsafe_code)] // let us try

mod components;
mod systems;

use bevy::prelude::*;
use components::{ArmatureGraph, IkData, IkSettings};
use systems::*;

// reexports
pub use components::{Bone, BoneBundle, IkGoal, IkGoalBundle};

pub const DEFAULT_GOAL_TOLERANCE: f32 = 0.0001;
pub const DEFAULT_MAX_ITERATIONS: u32 = 100;

pub struct InverseKinematicsPlugin {
    pub goal_tolerance: f32,
    pub max_iterations: u32,
}

impl Default for InverseKinematicsPlugin {
    fn default() -> Self {
        Self {
            goal_tolerance: DEFAULT_GOAL_TOLERANCE,
            max_iterations: DEFAULT_MAX_ITERATIONS,
        }
    }
}

impl Plugin for InverseKinematicsPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(IkSettings {
            goal_tolerance: self.goal_tolerance,
            max_iterations: self.max_iterations,
        })
        .init_resource::<ArmatureGraph>()
        .init_resource::<IkData>()
        .add_system(create_armature_tree)
        .add_system(cache_ik_data.after(create_armature_tree))
        .add_system(compute_joint_positions.after(cache_ik_data))
        .add_system(apply_bone_rotations.after(compute_joint_positions));
    }
}
