use crate::components::{ArmatureGraph, Bone, IkData, IkGoal, IkSettings};
use bevy::{
    prelude::*,
    utils::{HashMap, HashSet},
};
use std::collections::VecDeque;

pub fn create_armature_tree(
    bone_parents: Query<(Entity, &Children), With<Bone>>,
    bones: Query<Entity, With<Bone>>,
    mut graph: ResMut<ArmatureGraph>,
) {
    // clear the graph
    graph.out_bones.clear();
    graph.in_bone.clear();
    graph.base_joint.clear();
    graph.pole_joint.clear();
    graph.joint_children.clear();
    graph.joint_parent.clear();

    let mut joint_id = 0;

    for (par_id, children) in bone_parents.iter() {
        let mut has_children = false;
        // gather all children bones - go through all children entities
        for &child_id in children.iter() {
            // if the child is a bone, register the bone as an out_bone of this next joint
            if let Ok(bone_id) = bones.get(child_id) {
                has_children = true;
                graph
                    .out_bones
                    .entry(joint_id)
                    .or_insert(HashSet::new())
                    .insert(bone_id);
            }
        }

        // register this bone as the incoming bone for this joint - but only if it had children
        // leaf bones should not have joints at their end
        if has_children {
            graph.in_bone.insert(joint_id, par_id);
        }

        // increment joint id for the next one
        joint_id += 1;
    }

    // add missing root joints and missing leaf bones
    // (they are not between two bones, so we didn't catch them earlier)
    for bone_id in bones.iter() {
        let mut is_out_bone = false;
        for out_bones in graph.out_bones.values() {
            if out_bones.contains(&bone_id) {
                is_out_bone = true;
            }
        }

        // if this bone is a root bone, add a root joint
        if !is_out_bone {
            graph
                .out_bones
                .entry(joint_id)
                .or_insert(HashSet::new())
                .insert(bone_id);
        }
    }

    // register base joint for each bone
    let mut base_joint = HashMap::<Entity, u32>::new();
    for (jid, out_bones) in graph.out_bones.iter() {
        for out_bone in out_bones {
            assert!(!graph.base_joint.contains_key(out_bone));
            base_joint.insert(*out_bone, *jid);
        }
    }
    graph.base_joint = base_joint;

    // register pole joint for each bone
    let mut pole_joint = HashMap::<Entity, u32>::new();
    for (jid, in_bone) in graph.in_bone.iter() {
        assert!(!graph.pole_joint.contains_key(in_bone));
        pole_joint.insert(*in_bone, *jid);
    }
    graph.pole_joint = pole_joint;

    // joint - joint relations
    let mut joint_children = HashMap::<u32, HashSet<u32>>::new();
    let mut joint_parent = HashMap::<u32, u32>::new();
    for (par_id, out_bones) in graph.out_bones.iter() {
        for out_bone in out_bones.iter() {
            for (child_id, in_bone) in graph.in_bone.iter() {
                if out_bone == in_bone {
                    joint_parent.insert(*child_id, *par_id);
                    joint_children
                        .entry(*par_id)
                        .or_insert(HashSet::new())
                        .insert(*child_id);
                }
            }
        }
    }
    graph.joint_children = joint_children;
    graph.joint_parent = joint_parent;
}

pub fn cache_ik_data(
    bones: Query<(Entity, &Bone, &Transform, &GlobalTransform), With<Bone>>,
    goals: Query<(Entity, &GlobalTransform, &IkGoal), Without<Bone>>,
    graph: Res<ArmatureGraph>,
    mut data: ResMut<IkData>,
) {
    // clear the data
    data.joint_positions.clear();
    data.required_positions.clear();
    data.joints_to_goals.clear();
    data.roots.clear();
    data.bone_lengths.clear();

    // register joint to goal mapping
    for (goal_id, _, goal) in goals.iter() {
        // register target joint from goal
        if let Some(base_joint) = graph.base_joint.get(&goal.target_bone) {
            data.joints_to_goals.insert(*base_joint, goal_id);
        }
    }

    // register roots and required positions
    for (_, _, goal) in goals.iter() {
        let mut cur_id = graph.base_joint.get(&goal.target_bone).unwrap();
        for i in 0..goal.chain_length {
            if let Some(par_id) = graph.joint_parent.get(cur_id) {
                // add the child bone as a required bone for the parent bone
                data.required_positions
                    .entry(*par_id)
                    .or_insert(HashSet::new())
                    .insert(*cur_id);
                cur_id = par_id;
            } else {
                // bone without parent, this is the root
                data.roots.insert(*cur_id);
                break;
            }

            //if we stop going up the tree due to chain length limitation, this node now also counts as a pseudo-root
            if i == goal.chain_length - 1 {
                data.roots.insert(*cur_id);
            }
        }
    }

    // initialize positions
    for (bone_id, _, _, gt) in bones.iter() {
        if let Some(base_joint) = graph.base_joint.get(&bone_id) {
            data.joint_positions.insert(*base_joint, gt.translation);
        }
    }

    // bone lengths
    for (bone_id, _, _, _) in bones.iter() {
        if let Some(pole_joint) = graph.pole_joint.get(&bone_id) {
            let base_joint = graph.base_joint.get(&bone_id).unwrap();
            let pole_pos = data.joint_positions.get(pole_joint).unwrap();
            let base_pos = data.joint_positions.get(base_joint).unwrap();
            let dist = pole_pos.distance(*base_pos);
            data.bone_lengths.insert(bone_id, dist);
        }
    }
}

pub fn compute_joint_positions(
    goals: Query<(&GlobalTransform, &IkGoal)>,
    graph: Res<ArmatureGraph>,
    settings: Res<IkSettings>,
    mut data: ResMut<IkData>,
) {
    // queue to walk through the armature graph
    let mut todo_queue = VecDeque::<u32>::new();

    for _ in 0..settings.max_iterations {
        // check if target bones are close enough to the goals
        let mut highest_dist: f32 = 0.0;
        for (goal_tf, goal) in goals.iter() {
            let goal_joint = graph.base_joint.get(&goal.target_bone).unwrap();
            let pos = data.joint_positions.get(goal_joint).unwrap();
            let dist = (goal_tf.translation - *pos).length();
            highest_dist = highest_dist.max(dist);
        }
        if highest_dist < settings.goal_tolerance {
            break;
        }
        /*
         * FORWARD PASS - LEAF TO ROOT
         */

        // initialize todo queue with starting joints (joints with goals)
        todo_queue.clear();
        for (_, goal) in goals.iter() {
            let goal_joint = graph.base_joint.get(&goal.target_bone).unwrap();
            todo_queue.push_back(*goal_joint);
        }

        // new positions
        let mut new_positions = HashMap::<u32, Vec3>::new();

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
            if let Some(goal_id) = data.joints_to_goals.get(&joint_id) {
                // in the forward pass, the target bone of the goal is simply set to the goal position
                let (goal_transform, _) = goals.get(*goal_id).unwrap();
                new_positions.insert(joint_id, goal_transform.translation);
            } else {
                // otherwise compute a new position for each child
                // the new position is the centroid of those positions
                let old_pos = data.joint_positions.get(&joint_id).unwrap();
                let children = data.required_positions.get(&joint_id).unwrap();
                let mut new_pos_centroid = Vec3::ZERO;
                for child_id in children {
                    let bone_id = graph.in_bone.get(child_id).unwrap();
                    let child_link_length = data.bone_lengths.get(bone_id).unwrap();

                    let new_child_pos = new_positions.get(child_id).unwrap();
                    let dir = (*old_pos - *new_child_pos).normalize() * *child_link_length;
                    let new_pos = *new_child_pos + dir;
                    new_pos_centroid += new_pos;
                }
                new_pos_centroid *= 1. / children.len() as f32;
                new_positions.insert(joint_id, new_pos_centroid);
            }

            // if we are not at the root, push the parent to the todo_queue, if it's not already in there
            if !data.roots.contains(&joint_id) {
                if let Some(par_id) = graph.joint_parent.get(&joint_id) {
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
        todo_queue.clear();
        for root in data.roots.iter() {
            todo_queue.push_back(*root);
        }

        // actual backward pass
        while let Some(joint_id) = todo_queue.pop_front() {
            // if this bone is one of the roots or pseudo-roots, we just set them back to their original position
            if data.roots.contains(&joint_id) {
                let old_pos = data.joint_positions.get(&joint_id).unwrap();
                new_positions.insert(joint_id, *old_pos);
            } else {
                let in_bone_id = graph.in_bone.get(&joint_id).unwrap();
                let bone_length = data.bone_lengths.get(in_bone_id).unwrap();
                let par_id = graph.joint_parent.get(&joint_id).unwrap();
                let par_pos = new_positions.get(par_id).unwrap();
                let forward_pos = new_positions.get(&joint_id).unwrap();
                let dir = (*forward_pos - *par_pos).normalize() * *bone_length;
                let backward_pos = *par_pos + dir;
                new_positions.insert(joint_id, backward_pos);
            }

            // put all required children in the todo queue - those who lead to a leaf bone with IK goal
            if let Some(children) = data.required_positions.get(&joint_id) {
                for child_id in children {
                    todo_queue.push_back(*child_id);
                }
            }
        }

        // "flip the buffer"
        data.joint_positions = new_positions;
    }
}

const EPS: f32 = 0.001;
pub fn apply_bone_rotations(
    mut bones: Query<(Entity, &mut Transform), With<Bone>>,
    parents: Query<&Parent>,
    global_tfs: Query<&GlobalTransform>,
    graph: Res<ArmatureGraph>,
    data: Res<IkData>,
) {
    // queue to walk through the armature graph
    let mut todo_queue = VecDeque::<Entity>::new();

    // updated - global transforms
    let mut par_tfs_global = HashMap::<Entity, GlobalTransform>::new();

    // enqueue bones connected to a root joint
    for (bone_id, _) in bones.iter() {
        let base_joint = graph.base_joint.get(&bone_id).unwrap();
        // check if this bone is associated to a root joint
        if data.roots.contains(base_joint) {
            // enqueue the bone
            todo_queue.push_back(bone_id);
            // register the global transform of the parent, if no parent exists, register identity transform
            if let Ok(parent) = parents.get(bone_id) {
                let global_tf = global_tfs.get(parent.0).unwrap(); // each parent should have a global transform
                par_tfs_global.insert(bone_id, *global_tf);
            } else {
                par_tfs_global.insert(bone_id, GlobalTransform::identity());
            }
        }
    }

    // apply position changes by rotation only - from root to children
    while let Some(bone_id) = todo_queue.pop_front() {
        let base_tf_local = bones.get_mut(bone_id).unwrap().1;
        let par_tf_global = *par_tfs_global.get(&bone_id).unwrap();
        let base_tf_global = par_tf_global.mul_transform(*base_tf_local);
        let base_joint = graph.base_joint.get(&bone_id).unwrap();
        let base_pos_global = *data.joint_positions.get(base_joint).unwrap();

        // check that the base is already at the correct position
        assert!(base_tf_global.translation.distance(base_pos_global) < EPS);

        if let Some(pole_joint) = graph.pole_joint.get(&bone_id) {
            if let Some(&new_pole_pos_global) = data.joint_positions.get(pole_joint) {
                // ASSUMPTION: ALL CHILD BONES HAVE THE SAME LOCAL TRANSLATION
                let pole_tf_local = *graph
                    .out_bones
                    .get(pole_joint)
                    .unwrap() // if the bone has a pole_joint, it has to have child bones
                    .iter()
                    .map(|&bid| bones.get(bid).unwrap().1)
                    .next()
                    .unwrap(); // if the bone has child_bones, it has to have at least one

                let pole_tf_global = base_tf_global.mul_transform(pole_tf_local);

                let old_pole_pos_global = pole_tf_global.translation;

                let old_dir = old_pole_pos_global - base_pos_global;
                let new_dir = new_pole_pos_global - base_pos_global;

                // generate quaternion and apply
                let rot = Quat::from_rotation_arc(old_dir.normalize(), new_dir.normalize());
                let mut base_tf_local = bones.get_mut(bone_id).unwrap().1;
                base_tf_local.rotate(rot);

                // update global transform
                let base_tf_global = par_tf_global.mul_transform(*base_tf_local);

                // check that the updated global base of the bone is at the correct position
                assert!(base_tf_global.translation.distance(base_pos_global) < EPS);

                // check that the updated global pole of the bone is at the correct position
                let pole_tf_global = base_tf_global.mul_transform(pole_tf_local);
                assert!(pole_tf_global.translation.distance(new_pole_pos_global) < EPS);

                // register new global tf for all bone children and add them to the queue
                for child_bone in graph.out_bones.get(pole_joint).unwrap() {
                    todo_queue.push_back(*child_bone);
                    par_tfs_global.insert(*child_bone, base_tf_global);
                }
            }
        }
    }
}
