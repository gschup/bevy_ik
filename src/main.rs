pub mod joint;

use std::f32::consts::PI;

use bevy::prelude::*;
use joint::{Joint, JointBundle, JointName, LinkLength};

const LINK_THICKNESS: f32 = 0.1;

pub struct JointVizHandles {
    pub joint_mesh_handle: Handle<Mesh>,
    pub link_mesh_handle: Handle<Mesh>,
    pub joint_material_handle: Handle<StandardMaterial>,
    pub link_material_handle: Handle<StandardMaterial>,
}

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_startup_system_to_stage(StartupStage::PreStartup, setup_render)
        .add_startup_system_to_stage(StartupStage::Startup, setup_armature)
        .add_startup_system_to_stage(StartupStage::PostStartup, setup_joint_visuals)
        .run();
}

/// prepare light, camera and joint/link visualization data
fn setup_render(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // light
    commands.spawn_bundle(PointLightBundle {
        transform: Transform::from_xyz(4.0, 5.0, -4.0),
        ..default()
    });
    // camera
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(10.0, 0.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });

    // prepare joint and link visualization data
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
        1.0,
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

/// load a simple armature,  consisting of hierarchical joint bundles
fn setup_armature(mut commands: Commands) {
    // TMP: link length
    let link_length = 2.0;

    // spawn a joint
    commands
        .spawn_bundle(JointBundle {
            name: JointName("root".to_string()),
            link_length: LinkLength(link_length),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(JointBundle {
                name: JointName("arm".to_string()),
                transform: Transform {
                    translation: Vec3::new(0.0, link_length, 0.0),
                    rotation: Quat::from_rotation_x(PI * 0.25),
                    ..default()
                },
                link_length: LinkLength(link_length),
                ..default()
            });
        });
}

fn setup_joint_visuals(
    mut commands: Commands,
    joints: Query<(Entity, &LinkLength), With<Joint>>,
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
                    translation: Vec3::new(0.0, link_length.0 * 0.5, 0.0),
                    scale: Vec3::new(1.0, link_length.0, 1.0),
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
