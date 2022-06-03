use bevy::prelude::*;
use bevy_ik::{Bone, BoneBundle, IkGoal, IkGoalBundle};

use crate::components::{BoneVizHandles, GoalVizHandles};

const LINK_THICKNESS: f32 = 0.1;
const GOAL_SIZE: f32 = 0.3;

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

pub fn setup_bone_assets(
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
        LINK_THICKNESS,
        1.0, // scaled by the bone itself
    )));
    let joint_mesh_handle = meshes.add(Mesh::from(shape::Cube { size: 0.2 }));

    commands.insert_resource(BoneVizHandles {
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

pub fn setup_fork_armature(mut commands: Commands) {
    // link lengths
    let link_lengths = [3.0, 2.0, 1.0];

    commands
        .spawn_bundle(BoneBundle {
            bone: Bone {
                name: "root".to_owned(),
            },
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn_bundle(BoneBundle {
                    bone: Bone {
                        name: "fork".to_owned(),
                    },
                    transform: Transform::from_xyz(0.0, link_lengths[0], 0.0),
                    ..default()
                })
                .with_children(|parent| {
                    parent
                        .spawn_bundle(BoneBundle {
                            bone: Bone {
                                name: "arm_1".to_owned(),
                            },
                            transform: Transform::from_xyz(0.0, 0.0, link_lengths[1]),

                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn_bundle(BoneBundle {
                                bone: Bone {
                                    name: "hand_1".to_owned(),
                                },
                                transform: Transform::from_xyz(0.0, link_lengths[2], 0.0),
                                ..default()
                            });
                        });
                    parent
                        .spawn_bundle(BoneBundle {
                            bone: Bone {
                                name: "arm_2".to_owned(),
                            },
                            transform: Transform::from_xyz(0.0, 0.0, link_lengths[1]),
                            ..default()
                        })
                        .with_children(|parent| {
                            parent.spawn_bundle(BoneBundle {
                                bone: Bone {
                                    name: "hand_2".to_owned(),
                                },
                                transform: Transform::from_xyz(0.0, link_lengths[2], 0.0),
                                ..default()
                            });
                        });
                });
        });
}

pub fn setup_goals(
    mut commands: Commands,
    bones: Query<(Entity, &Bone)>,
    assets: Res<GoalVizHandles>,
) {
    let targets = vec![("hand_1", 3), ("hand_2", 3)];

    for (target_bone_name, chain_length) in targets.iter() {
        let target_id = bones
            .iter()
            .filter_map(|(id, bone)| {
                if bone.name == *target_bone_name {
                    Some(id)
                } else {
                    None
                }
            })
            .next()
            .expect("No valid bone found");

        commands
            .spawn_bundle(IkGoalBundle {
                transform: Transform::from_xyz(0.0, 6.0, 0.0),
                global_transform: GlobalTransform::default(),
                goal: IkGoal {
                    target_bone: target_id,
                    chain_length: *chain_length,
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
}

pub fn setup_bone_visuals(
    mut commands: Commands,
    bones: Query<(Entity, &Transform), With<Bone>>,
    viz_handles: Res<BoneVizHandles>,
) {
    for (bone_id, transform) in bones.iter() {
        // joint
        let joint_viz_id = commands
            .spawn_bundle(PbrBundle {
                mesh: viz_handles.joint_mesh_handle.clone(),
                material: viz_handles.joint_material_handle.clone(),
                ..default()
            })
            .id();

        commands.entity(bone_id).push_children(&[joint_viz_id]);

        // link only if there is a displacement
        let link_length = transform.translation.length();
        if link_length > 0.01 {
            let link_viz_id = commands
                .spawn_bundle(PbrBundle {
                    mesh: viz_handles.link_mesh_handle.clone(),
                    material: viz_handles.link_material_handle.clone(),
                    transform: Transform {
                        translation: Vec3::new(0.0, 0.0, -link_length * 0.5),
                        scale: Vec3::new(1.0, 1.0, link_length),
                        ..default()
                    },
                    ..default()
                })
                .id();

            commands.entity(bone_id).push_children(&[link_viz_id]);
        }
    }
}

pub fn rotate_goals(mut goals: Query<&mut Transform, With<IkGoal>>, time: Res<Time>) {
    for (i, mut goal_tf) in goals.iter_mut().enumerate() {
        let rad = 2.;
        let ampl = 2.;
        let height = 2.;
        let dir = ((i % 2) as f32 * 2.) - 1.; // either 1 or -1
        let speed = 0.001 * (i + 1) as f32;
        let ms = time.time_since_startup().as_millis() as f32;
        let t = ms * speed * dir;
        let x = rad * t.cos();
        let z = rad * t.sin();
        let y = height + ampl * (3. * t).sin();
        *goal_tf = Transform::from_xyz(x, y, z);
    }
}
