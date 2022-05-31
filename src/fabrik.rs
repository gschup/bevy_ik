use crate::components::{ArmatureGraph, IkGoal, Joint};
use bevy::{prelude::*, utils::HashMap};
use std::collections::VecDeque;

const MAX_FABRIK_ITERS: u32 = 100;
const GOAL_ACCURACY: f32 = 0.01;

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
    mut joints: Query<(Entity, &Joint, &mut Transform, &GlobalTransform), With<Joint>>,
    goals: Query<(Entity, &GlobalTransform, &IkGoal), Without<Joint>>,
    armature_graph: ResMut<ArmatureGraph>,
) {
    // queue we are going to use for multiple purposes
    let mut todo_queue = VecDeque::<Entity>::new();
    // for each joint, which children do we need info from? (some leaves might not have IK goals)
    let mut required_positions = HashMap::<Entity, Vec<Entity>>::new();
    // hashmap of joint_ids to goal_ids
    let mut goals_by_joints = HashMap::<Entity, Entity>::new();
    // FABRIK roots - joints defined by not having a parent, or by chain length, or if fixed
    let mut roots = Vec::new();

    // fill data structures defined above
    for (goal_id, _, goal) in goals.iter() {
        goals_by_joints.insert(goal.target_joint, goal_id);
        let mut cur_id = goal.target_joint;
        for i in 0..goal.chain_length {
            if let Some(par_id) = armature_graph.joint_parent.get(&cur_id) {
                // add the child joint as a required joint
                required_positions
                    .entry(*par_id)
                    .or_insert(Vec::new())
                    .push(cur_id);
                cur_id = *par_id;
                // we stop at joints with fixed positions
                if joints.get(*par_id).unwrap().1.fixed {
                    roots.push(*par_id);
                    break;
                }
            } else {
                // joint without parent, this is the root
                roots.push(cur_id);
                break;
            }

            //if we stop going up the tree due to chain length limitation, this node now also counts as a pseudo-root
            if i == goal.chain_length - 1 {
                roots.push(cur_id);
            }
        }
    }

    // initialize positions
    let mut positions = HashMap::<Entity, Vec3>::new();
    for (id, _, _, gt) in joints.iter() {
        positions.insert(id, gt.translation.clone());
    }

    for _ in 0..MAX_FABRIK_ITERS {
        // check if target joints are close enough to the goals
        let mut highest_dist: f32 = 0.0;
        for (_, goal_tf, goal) in goals.iter() {
            let joint_tf = joints.get(goal.target_joint).unwrap().3;
            let dist = (goal_tf.translation - joint_tf.translation).length();
            highest_dist = highest_dist.max(dist);
        }
        if highest_dist < GOAL_ACCURACY {
            break;
        }
        /*
         * FORWARD PASS - LEAF TO ROOT
         */

        // initialize todo queue with starting joints (leaves with goals)
        todo_queue.clear(); // should be empty anyway, but just to make sure
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

        // actual forward pass
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
                // otherwise compute a new position for each child
                // the new position is the centroid of those positions
                let old_pos = positions.get(&joint_id).unwrap();
                let children = required_positions.get(&joint_id).unwrap();
                let mut new_pos_centroid = Vec3::ZERO;
                for child_id in children {
                    let child_link_length = joints.get(*child_id).unwrap().2.translation.length();
                    let new_child_pos = new_positions.get(child_id).unwrap();
                    let dir = (*old_pos - *new_child_pos).normalize() * child_link_length;
                    let new_pos = *new_child_pos + dir;
                    new_pos_centroid += new_pos;
                }
                new_pos_centroid *= 1. / children.len() as f32;
                new_positions.insert(joint_id, new_pos_centroid);
            }

            // push the parent to the todo_queue, if it's not already in there
            if let Some(par_id) = armature_graph.joint_parent.get(&joint_id) {
                if !todo_queue.contains(par_id) {
                    todo_queue.push_back(*par_id);
                }
            }
        }

        /*
         * BACKWARD PASS - ROOT TO LEAF
         */

        // prepare todo queue for backward pass
        todo_queue.clear(); // should be empty anyway, but just to make sure
        for root in roots.iter() {
            todo_queue.push_back(*root);
        }

        // actual backward pass
        while let Some(joint_id) = todo_queue.pop_front() {
            // useful joint data
            let (_, _, transf, _) = joints.get(joint_id).unwrap();
            let link_length = transf.translation.length();
            // if this joint is one of the roots or pseudo-roots, we just set them back to their original position
            if roots.contains(&joint_id) {
                let old_pos = positions.get(&joint_id).unwrap();
                new_positions.insert(joint_id, *old_pos);
            } else {
                let par_id = armature_graph.joint_parent.get(&joint_id).unwrap();
                let par_pos = new_positions.get(par_id).unwrap();
                let forward_pos = new_positions.get(&joint_id).unwrap();
                let dir = (*forward_pos - *par_pos).normalize() * link_length;
                let backward_pos = *par_pos + dir;
                new_positions.insert(joint_id, backward_pos);
            }

            // put all required children in the todo queue - those who lead to a leaf joint with IK goal
            if let Some(children) = required_positions.get(&joint_id) {
                for child_id in children {
                    todo_queue.push_back(*child_id);
                }
            }
        }

        // "flip the buffer"
        positions = new_positions;
    }

    // apply position changes - start from root children (roots should not move)
    // joints and their global transforms
    let mut global_transforms = HashMap::<Entity, Mat4>::new();
    todo_queue.clear(); // should be empty anyway, but just to make sure
    for root in roots.iter() {
        // register root global transform - unchanged from before
        let global_mat = joints.get(*root).unwrap().3.compute_matrix();
        global_transforms.insert(*root, global_mat);
        // enqueue children
        if let Some(children) = required_positions.get(&root) {
            for child_id in children {
                todo_queue.push_back(*child_id);
            }
        }
    }

    // work through the tree
    while let Some(joint_id) = todo_queue.pop_front() {
        let par_id = armature_graph.joint_parent.get(&joint_id).unwrap();
        let par_mat = global_transforms.get(par_id).unwrap();
        let pos = positions.get(&joint_id).unwrap();
        let par_pos = positions.get(par_id).unwrap();

        // compute global and local mat
        let global_mat = Transform::from_translation(*pos)
            .looking_at(*par_pos, Vec3::X)
            .compute_matrix();
        let local_mat = par_mat.inverse() * global_mat;

        // register global mat
        global_transforms.insert(joint_id, global_mat);

        // update local tf
        *joints.get_mut(joint_id).unwrap().2 = Transform::from_matrix(local_mat);

        // enqueue relevant children
        if let Some(children) = required_positions.get(&joint_id) {
            for child_id in children {
                todo_queue.push_back(*child_id);
            }
        }
    }
}
