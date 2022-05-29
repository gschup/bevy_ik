use bevy::prelude::*;

use crate::components::{ArmatureGraph, IkGoal, Joint};

pub fn create_armature_graph(
    joint_parents: Query<(Entity, &Children), With<Joint>>,
    joints: Query<Entity, With<Joint>>,
    mut armature_graph: ResMut<ArmatureGraph>,
) {
    // parent-child relationship graph
    armature_graph.joint_children.clear();
    armature_graph.joint_parent.clear();

    for (par_id, children) in joint_parents.iter() {
        let mut ids = Vec::new();
        // go through all children entities
        for &child_id in children.iter() {
            // if the child is a joint, cache the parent-child relation
            if let Ok(joint_id) = joints.get(child_id) {
                ids.push(joint_id);
                armature_graph.joint_parent.insert(joint_id, par_id);
            }
        }
        // register all joint children
        armature_graph.joint_children.insert(par_id, ids);
    }
}

pub fn apply_ik_goal(
    mut joints: Query<(Entity, &Transform, &GlobalTransform), With<Joint>>,
    goals: Query<((&Transform, &IkGoal), Without<Joint>)>,
    armature_graph: ResMut<ArmatureGraph>,
) {
}
