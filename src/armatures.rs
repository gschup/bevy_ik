use bevy::prelude::*;
use std::f32::consts::PI;

use crate::components::{Joint, JointBundle, Link};

/// load a simple armature, consisting of hierarchical joint bundles
pub fn load_chain_armature(mut commands: Commands) {
    // link lengths
    let link_lengths = [3.0, 2.0, 1.0];

    // spawn four joints with three links (last joint has link length 0)
    commands
        .spawn_bundle(JointBundle {
            joint: Joint::new("root"),
            link: Link::new(link_lengths[0]),
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn_bundle(JointBundle {
                    joint: Joint::new("arm_0"),
                    transform: Transform {
                        translation: Vec3::new(0.0, link_lengths[0], 0.0),
                        rotation: Quat::from_rotation_x(PI * 0.25),
                        ..default()
                    },
                    link: Link::new(link_lengths[1]),
                    ..default()
                })
                .with_children(|parent| {
                    parent
                        .spawn_bundle(JointBundle {
                            joint: Joint::new("arm_1"),
                            transform: Transform {
                                translation: Vec3::new(0.0, link_lengths[1], 0.0),
                                rotation: Quat::from_rotation_x(PI * 0.25),
                                ..default()
                            },
                            link: Link::new(link_lengths[2]),
                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn_bundle(JointBundle {
                                joint: Joint::new("hand"),
                                transform: Transform {
                                    translation: Vec3::new(0.0, link_lengths[2], 0.0),
                                    ..default()
                                },
                                link: Link::new(0.0),
                                ..default()
                            });
                        });
                });
        });
}
