use bevy::prelude::*;

use crate::components::{Joint, JointBundle};

/// load a simple armature, consisting of hierarchical joint bundles
pub fn load_chain_armature(mut commands: Commands) {
    // link lengths
    let link_lengths = [3.0, 2.0, 1.0];

    // spawn four joints with three links (last joint has link length 0)
    commands
        .spawn_bundle(JointBundle {
            joint: Joint {
                name: "root".to_owned(),
                fixed: true,
            },
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn_bundle(JointBundle {
                    joint: Joint {
                        name: "arm_0".to_owned(),
                        fixed: false,
                    },
                    transform: Transform::from_xyz(0.0, link_lengths[0], 0.0)
                        .looking_at(Vec3::ZERO, Vec3::X),
                    ..default()
                })
                .with_children(|parent| {
                    parent
                        .spawn_bundle(JointBundle {
                            joint: Joint {
                                name: "arm_1".to_owned(),
                                fixed: false,
                            },
                            transform: Transform::from_xyz(0.0, 0.0, link_lengths[1])
                                .looking_at(Vec3::ZERO, Vec3::Y),
                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn_bundle(JointBundle {
                                joint: Joint {
                                    name: "hand".to_owned(),
                                    fixed: false,
                                },
                                transform: Transform::from_xyz(0.0, link_lengths[2], 0.0)
                                    .looking_at(Vec3::ZERO, Vec3::X),
                                ..default()
                            });
                        });
                });
        });
}
