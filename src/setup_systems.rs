use crate::components::{GoalVizHandles, IkGoalBundle, Joint, JointBundle, JointVizHandles, Link};
use bevy::prelude::*;
use std::f32::consts::PI;

const LINK_THICKNESS: f32 = 0.1;
const GOAL_SIZE: f32 = 0.2;

pub fn setup_camera(mut commands: Commands) {
    // light
    commands.spawn_bundle(PointLightBundle {
        transform: Transform::from_xyz(4.0, 5.0, -4.0),
        ..default()
    });
    // camera
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(15.0, 0.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

pub fn setup_joint_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let link_material_handle = materials.add(StandardMaterial {
        base_color: Color::BEIGE,
        ..default()
    });
    let joint_material_handle = materials.add(StandardMaterial {
        base_color: Color::RED,
        ..default()
    });
    let link_mesh_handle = meshes.add(Mesh::from(shape::Box::new(
        LINK_THICKNESS,
        1.0, // scaled by the joint itself
        LINK_THICKNESS,
    )));
    let joint_mesh_handle = meshes.add(Mesh::from(shape::Cube { size: 0.2 }));

    commands.insert_resource(JointVizHandles {
        joint_mesh_handle,
        link_mesh_handle,
        joint_material_handle,
        link_material_handle,
    })
}

pub fn setup_goal_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let goal_material_handle = materials.add(StandardMaterial {
        base_color: Color::YELLOW,
        unlit: true,
        ..default()
    });
    let goal_mesh_handle = meshes.add(Mesh::from(shape::Icosphere {
        radius: GOAL_SIZE * 0.5,
        subdivisions: 4,
    }));

    commands.insert_resource(GoalVizHandles {
        goal_mesh_handle,
        goal_material_handle,
    })
}

/// load a simple armature, consisting of hierarchical joint bundles
pub fn setup_armature(mut commands: Commands) {
    // link lengths
    let link_lengths = [3.0, 2.0, 1.0];

    // spawn three joints with links
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
                    parent.spawn_bundle(JointBundle {
                        joint: Joint::new("arm_1"),
                        transform: Transform {
                            translation: Vec3::new(0.0, link_lengths[1], 0.0),
                            rotation: Quat::from_rotation_x(PI * 0.25),
                            ..default()
                        },
                        link: Link::new(link_lengths[2]),
                        ..default()
                    });
                });
        });
}

pub fn setup_joint_visuals(
    mut commands: Commands,
    joints: Query<(Entity, &Link), With<Joint>>,
    viz_handles: Res<JointVizHandles>,
) {
    for (joint_id, link_length) in joints.iter() {
        // joint
        let joint_viz_id = commands
            .spawn_bundle(PbrBundle {
                mesh: viz_handles.joint_mesh_handle.clone(),
                material: viz_handles.joint_material_handle.clone(),
                ..default()
            })
            .id();

        // link
        let link_viz_id = commands
            .spawn_bundle(PbrBundle {
                mesh: viz_handles.link_mesh_handle.clone(),
                material: viz_handles.link_material_handle.clone(),
                transform: Transform {
                    translation: Vec3::new(0.0, link_length.length * 0.5, 0.0),
                    scale: Vec3::new(1.0, link_length.length, 1.0),
                    ..default()
                },
                ..default()
            })
            .id();

        commands
            .entity(joint_id)
            .push_children(&[joint_viz_id, link_viz_id]);
    }
}

pub fn spawn_goal(mut commands: Commands, assets: Res<GoalVizHandles>) {
    commands
        .spawn_bundle(IkGoalBundle {
            transform: Transform::from_xyz(0.0, 5.0, 5.0),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(PbrBundle {
                mesh: assets.goal_mesh_handle.clone(),
                material: assets.goal_material_handle.clone(),
                ..default()
            });
        });
}
