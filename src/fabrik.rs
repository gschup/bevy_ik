use crate::components::{ArmatureGraph, IkGoal, Joint, Link};
use bevy::{prelude::*, utils::HashMap};
use std::collections::VecDeque;

const MAX_FABRIK_ITERS: u32 = 1;

pub fn create_armature_tree(
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
    mut joints: Query<(Entity, &Joint, &Link, &Transform, &GlobalTransform), With<Joint>>,
    goals: Query<(Entity, &GlobalTransform, &IkGoal), Without<Joint>>,
    armature_graph: ResMut<ArmatureGraph>,
) {
    // figure out from which joints we need info for a forward pass
    // (some leaves might not have IK goals, also chain lengths)
    // and keep a hashmap of joint_ids to goal_ids
    let mut required_positions = HashMap::<Entity, Vec<Entity>>::new();
    let mut goals_by_joints = HashMap::<Entity, Entity>::new();
    for (goal_id, _, goal) in goals.iter() {
        goals_by_joints.insert(goal.target_joint, goal_id);
        let mut cur_id = goal.target_joint;
        for _ in 0..goal.chain_length {
            if let Some(par_id) = armature_graph.joint_parent.get(&cur_id) {
                // add the child joint as a required joint
                let reqs = required_positions.entry(*par_id).or_insert(Vec::new());
                reqs.push(cur_id);
                cur_id = *par_id;
                // we stop at joints with fixed positions
                if joints.get(*par_id).unwrap().1.fixed {
                    break;
                }
            } else {
                // joint without parent, this is the root
                break;
            }
        }
    }

    // initialize positions
    let mut positions = HashMap::<Entity, Vec3>::new();
    for (id, _, _, _, gt) in joints.iter() {
        positions.insert(id, gt.translation.clone());
    }

    for _ in 0..MAX_FABRIK_ITERS {
        // FORWARD PASS - LEAF TO ROOT
        println!("+++++");

        // initialize todo queue with starting joints (leaves with goals)
        let mut todo_queue = VecDeque::<Entity>::new();
        for (_, _, goal) in goals.iter() {
            // the goal joint is a leaf in the armature graph
            assert_eq!(
                armature_graph
                    .joint_children
                    .get(&goal.target_joint)
                    .unwrap()
                    .len(),
                0,
                "IK goals should always be armature leaves!"
            );
            todo_queue.push_back(goal.target_joint);
        }

        // new positions
        let mut new_positions = HashMap::<Entity, Vec3>::new();
        while let Some(joint_id) = todo_queue.pop_front() {
            // check if all required joint children have a new position computed, otherwise push this joint back into the queue
            let mut ready = true;
            if let Some(reqs) = required_positions.get(&joint_id) {
                for req_id in reqs {
                    if !new_positions.contains_key(req_id) {
                        ready = false;
                        break;
                    }
                }
            }
            if !ready {
                todo_queue.push_back(joint_id);
                continue;
            }

            // figure out the new forward position for this joint
            if let Some(goal_id) = goals_by_joints.get(&joint_id) {
                // in the forward pass, the target joint of the goal is simply set to the goal position
                let goal_transform = goals.get(*goal_id).unwrap().1;
                new_positions.insert(joint_id, goal_transform.translation.clone());
            } else {
                // otherwise the new position is on the line between own old position and new children centroid position
                let old_pos = positions.get(&joint_id).unwrap();
                let mut new_child_centroid = Vec3::ZERO;
                let children = required_positions.get(&joint_id).unwrap();
                for child_id in children {
                    new_child_centroid += *new_positions
                        .get(child_id)
                        .expect("checked that this exists above");
                }
                new_child_centroid *= 1. / children.len() as f32;
                // TODO: compute new pos
            }

            // push the parent to the todo_queue, if it's not already in there
            if let Some(par_id) = armature_graph.joint_parent.get(&joint_id) {
                if !todo_queue.contains(par_id) {
                    todo_queue.push_back(*par_id);
                }
            }
        }

        println!("{:?}", new_positions);
    }
}
