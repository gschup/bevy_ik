use crate::components::{GoalVizHandles, IkGoal, IkGoalBundle, Joint, JointVizHandles, Link};
use bevy::prelude::*;

const LINK_THICKNESS: f32 = 0.1;
const GOAL_SIZE: f32 = 0.2;

pub fn setup_camera(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // Plane
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane { size: 100000.0 })),
        material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
        ..default()
    });
    // light
    commands.spawn_bundle(PointLightBundle {
        transform: Transform::from_xyz(4.0, 5.0, -4.0),
        ..default()
    });
    // camera
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(10.0, 5.0, 0.0).looking_at(Vec3::Y * 3.0, Vec3::Y),
        ..default()
    });
}

pub fn setup_joint_assets(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let link_material_handle = materials.add(StandardMaterial {
        base_color: Color::MAROON,
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

pub fn spawn_goal(
    mut commands: Commands,
    joints: Query<(Entity, &Joint)>,
    assets: Res<GoalVizHandles>,
) {
    let target_joint_name = "hand";
    let chain_length = 4;

    let target_id = joints
        .iter()
        .filter_map(|(id, joint)| {
            if joint.name == target_joint_name {
                Some(id)
            } else {
                None
            }
        })
        .next()
        .expect("No valid joint found");

    commands
        .spawn_bundle(IkGoalBundle {
            transform: Transform::from_xyz(0.0, 3.0, -3.0),
            global_transform: GlobalTransform::default(),
            goal: IkGoal {
                target_joint: target_id,
                chain_length,
            },
        })
        .with_children(|parent| {
            parent.spawn_bundle(PbrBundle {
                mesh: assets.goal_mesh_handle.clone(),
                material: assets.goal_material_handle.clone(),
                ..default()
            });
        });
}

pub fn setup_joint_visuals(
    mut commands: Commands,
    joints: Query<(Entity, &Link), With<Joint>>,
    viz_handles: Res<JointVizHandles>,
) {
    for (joint_id, link) in joints.iter() {
        // joint
        let joint_viz_id = commands
            .spawn_bundle(PbrBundle {
                mesh: viz_handles.joint_mesh_handle.clone(),
                material: viz_handles.joint_material_handle.clone(),
                ..default()
            })
            .id();

        commands.entity(joint_id).push_children(&[joint_viz_id]);

        // link only if there is one
        if link.length > 0.0 {
            let link_viz_id = commands
                .spawn_bundle(PbrBundle {
                    mesh: viz_handles.link_mesh_handle.clone(),
                    material: viz_handles.link_material_handle.clone(),
                    transform: Transform {
                        translation: Vec3::new(0.0, link.length * 0.5, 0.0),
                        scale: Vec3::new(1.0, link.length, 1.0),
                        ..default()
                    },
                    ..default()
                })
                .id();

            commands.entity(joint_id).push_children(&[link_viz_id]);
        }
    }
}
