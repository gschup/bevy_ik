pub mod joint;

use bevy::prelude::*;
use joint::{JointBundle, LinkLength};

const LINK_THICKNESS: f32 = 0.1;

fn main() {
    App::new()
        .insert_resource(Msaa { samples: 4 })
        .add_plugins(DefaultPlugins)
        .add_startup_system(load_armature)
        .add_startup_system(setup_viz.after(load_armature))
        .run();
}

/// for each joint, add a debug visualization box
fn setup_viz(mut commands: Commands) {
    // light
    commands.spawn_bundle(PointLightBundle {
        transform: Transform::from_xyz(4.0, 5.0, -4.0),
        ..default()
    });
    // camera
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(5.0, 10.0, 10.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    });
}

/// load a simple armature,  consisting of hierarchical joint bundles
fn load_armature(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // TMP: link length
    let link_length = 1.0;

    // create a material for the links
    let link_material_handle = materials.add(StandardMaterial {
        base_color: Color::rgb(0.8, 0.7, 0.6),
        ..default()
    });

    // create the mesh
    let link_mesh_handle = meshes.add(Mesh::from(shape::Box::new(
        LINK_THICKNESS,
        link_length,
        LINK_THICKNESS,
    )));

    // spawn a joint
    commands
        .spawn_bundle(JointBundle {
            transform: Transform::from_xyz(0.0, link_length / 2., 0.0),
            link_length: LinkLength(link_length),
            ..default()
        })
        .with_children(|parent| {
            parent.spawn_bundle(PbrBundle {
                mesh: link_mesh_handle.clone(),
                material: link_material_handle.clone(),
                transform: Transform::from_xyz(0.0, 0.0, 0.0),
                ..default()
            });
        });

    // debug origin marker
    // cube
    commands.spawn_bundle(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Cube { size: 0.2 })),
        material: materials.add(Color::rgb(1.0, 0.0, 0.0).into()),
        ..default()
    });
}
