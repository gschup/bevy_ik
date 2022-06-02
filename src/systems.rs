use crate::components::{ArmatureGraph, IkData, IkGoal, IkSettings, Joint};
use bevy::{prelude::*, utils::HashMap};
use std::collections::VecDeque;

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

pub fn cache_joint_data(
    joints: Query<(Entity, &Joint, &Transform, &GlobalTransform), With<Joint>>,
    goals: Query<(Entity, &GlobalTransform, &IkGoal), Without<Joint>>,
    armature_graph: Res<ArmatureGraph>,
    mut data: ResMut<IkData>,
) {
    // reset data
    data.required_positions.clear();
    data.goals_by_joints.clear();
    data.roots.clear();
    data.positions.clear();

    // fill data structures with updated information
    for (goal_id, _, goal) in goals.iter() {
        data.goals_by_joints.insert(goal.target_joint, goal_id);
        let mut cur_id = goal.target_joint;
        for i in 0..goal.chain_length {
            if let Some(par_id) = armature_graph.joint_parent.get(&cur_id) {
                // add the child joint as a required joint
                data.required_positions
                    .entry(*par_id)
                    .or_insert(Vec::new())
                    .push(cur_id);
                cur_id = *par_id;
                // we stop at joints with fixed positions
                if joints.get(*par_id).unwrap().1.fixed {
                    data.roots.push(*par_id);
                    break;
                }
            } else {
                // joint without parent, this is the root
                data.roots.push(cur_id);
                break;
            }

            //if we stop going up the tree due to chain length limitation, this node now also counts as a pseudo-root
            if i == goal.chain_length - 1 {
                data.roots.push(cur_id);
            }
        }
    }

    // initialize positions
    for (id, _, _, gt) in joints.iter() {
        data.positions.insert(id, gt.translation.clone());
    }
}

pub fn compute_joint_positions(
    joints: Query<(Entity, &Joint, &Transform, &GlobalTransform), With<Joint>>,
    goals: Query<(Entity, &GlobalTransform, &IkGoal), Without<Joint>>,
    armature_graph: Res<ArmatureGraph>,
    settings: Res<IkSettings>,
    mut data: ResMut<IkData>,
) {
    // queue to walk through the armature graph
    let mut todo_queue = VecDeque::<Entity>::new();

    for _ in 0..settings.max_iterations {
        // check if target joints are close enough to the goals
        let mut highest_dist: f32 = 0.0;
        for (_, goal_tf, goal) in goals.iter() {
            let joint_tf = joints.get(goal.target_joint).unwrap().3;
            let dist = (goal_tf.translation - joint_tf.translation).length();
            highest_dist = highest_dist.max(dist);
        }
        if highest_dist < settings.goal_tolerance {
            break;
        }
        /*
         * FORWARD PASS - LEAF TO ROOT
         */

        // initialize todo queue with starting joints (leaves with goals)
        todo_queue.clear(); // should be empty anyway, but just to make sure
        for (_, _, goal) in goals.iter() {
            todo_queue.push_back(goal.target_joint);
        }

        // new positions
        let mut new_positions = HashMap::<Entity, Vec3>::new();

        // actual forward pass
        while let Some(joint_id) = todo_queue.pop_front() {
            // check if all required joint children have a new position computed, otherwise push this joint back into the queue
            let mut ready = true;
            if let Some(reqs) = data.required_positions.get(&joint_id) {
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
            if let Some(goal_id) = data.goals_by_joints.get(&joint_id) {
                // in the forward pass, the target joint of the goal is simply set to the goal position
                let goal_transform = goals.get(*goal_id).unwrap().1;
                new_positions.insert(joint_id, goal_transform.translation.clone());
            } else {
                // otherwise compute a new position for each child
                // the new position is the centroid of those positions
                let old_pos = data.positions.get(&joint_id).unwrap();
                let children = data.required_positions.get(&joint_id).unwrap();
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

            // if we are not at the root, push the parent to the todo_queue, if it's not already in there
            if !data.roots.contains(&joint_id) {
                if let Some(par_id) = armature_graph.joint_parent.get(&joint_id) {
                    if !todo_queue.contains(par_id) {
                        todo_queue.push_back(*par_id);
                    }
                }
            }
        }

        /*
         * BACKWARD PASS - ROOT TO LEAF
         */

        // prepare todo queue for backward pass
        todo_queue.clear(); // should be empty anyway, but just to make sure
        for root in data.roots.iter() {
            todo_queue.push_back(*root);
        }

        // actual backward pass
        while let Some(joint_id) = todo_queue.pop_front() {
            // useful joint data
            let (_, _, transf, _) = joints.get(joint_id).unwrap();
            let link_length = transf.translation.length();
            // if this joint is one of the roots or pseudo-roots, we just set them back to their original position
            if data.roots.contains(&joint_id) {
                let old_pos = data.positions.get(&joint_id).unwrap();
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
            if let Some(children) = data.required_positions.get(&joint_id) {
                for child_id in children {
                    todo_queue.push_back(*child_id);
                }
            }
        }

        // "flip the buffer"
        data.positions = new_positions;
    }
}

pub fn apply_joint_positions(
    mut joint_transforms: Query<(&mut Transform, &GlobalTransform), With<Joint>>,
    armature_graph: Res<ArmatureGraph>,
    data: Res<IkData>,
) {
    // queue to walk through the armature graph
    let mut todo_queue = VecDeque::<Entity>::new();

    // apply position changes - start from root children (roots should not move)
    // joints and their global transforms
    let mut global_transforms = HashMap::<Entity, Mat4>::new();

    for root in data.roots.iter() {
        // register root global transform - unchanged from before
        let global_mat = joint_transforms.get(*root).unwrap().1.compute_matrix();
        global_transforms.insert(*root, global_mat);
        // enqueue children
        if let Some(children) = data.required_positions.get(&root) {
            for child_id in children {
                todo_queue.push_back(*child_id);
            }
        }
    }

    // work through the tree
    while let Some(joint_id) = todo_queue.pop_front() {
        let par_id = armature_graph.joint_parent.get(&joint_id).unwrap();
        let par_mat = global_transforms.get(par_id).unwrap();
        let pos = data.positions.get(&joint_id).unwrap();
        let par_pos = data.positions.get(par_id).unwrap();

        // compute global and local mat
        let global_mat = Transform::from_translation(*pos)
            .looking_at(*par_pos, Vec3::X)
            .compute_matrix();
        let local_mat = par_mat.inverse() * global_mat;

        // register global mat
        global_transforms.insert(joint_id, global_mat);

        // update local tf
        *joint_transforms.get_mut(joint_id).unwrap().0 = Transform::from_matrix(local_mat);

        // enqueue relevant children
        if let Some(children) = data.required_positions.get(&joint_id) {
            for child_id in children {
                todo_queue.push_back(*child_id);
            }
        }
    }
}
