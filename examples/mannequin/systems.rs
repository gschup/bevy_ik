use bevy::prelude::*;
use bevy_ik::{IkGoal, IkGoalBundle, Joint};

use crate::{components::MannequinInstance, AppState};

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
        transform: Transform::from_xyz(5.0, 5.0, 5.0),
        ..default()
    });
    // camera
    commands.spawn_bundle(PerspectiveCameraBundle {
        transform: Transform::from_xyz(0.0, 2.0, 3.0).looking_at(Vec3::Y * 0.75, Vec3::Y),
        ..default()
    });
}

pub fn setup_mannequin(
    asset_server: Res<AssetServer>,
    mut scene_spawner: ResMut<SceneSpawner>,
    mut scene_instance: ResMut<MannequinInstance>,
) {
    let instance_id = scene_spawner.spawn(asset_server.load("mannequin.gltf#Scene0"));
    scene_instance.0 = Some(instance_id);
}

pub fn tag_mannequin(
    names: Query<&Name>,
    mut commands: Commands,
    scene_spawner: Res<SceneSpawner>,
    scene_instance: Res<MannequinInstance>,
    mut app_state: ResMut<State<AppState>>,
) {
    if let Some(instance_id) = scene_instance.0 {
        if let Some(entity_iter) = scene_spawner.iter_instance_entities(instance_id) {
            entity_iter.for_each(|entity| {
                if let Ok(name) = names.get(entity) {
                    if name.contains("bone") {
                        commands.entity(entity).insert(Joint {
                            name: name.to_string(),
                        });
                    }
                }
            });
            app_state.set(AppState::Running).unwrap();
        }
    }
}

pub fn setup_goal(mut commands: Commands, joints: Query<(Entity, &Joint)>) {
    let target_joint_name = "bone_hand.L";
    let chain_length = 3;

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

    commands.spawn_bundle(IkGoalBundle {
        transform: Transform::from_xyz(0.0, 6.0, 0.0),
        global_transform: GlobalTransform::default(),
        goal: IkGoal {
            target_joint: target_id,
            chain_length,
        },
    });
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
