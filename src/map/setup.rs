use bevy::prelude::*;
use bevy_rapier3d::geometry::Collider;

pub fn setup_level(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(PointLightBundle {
        transform: Transform::from_xyz(5.0, 5.0, 5.0),
        ..default()
    });

    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 4000.0,
            // For some reason in Bevy 0.12 shadows no longer work in WASM
            shadows_enabled: !cfg!(target_arch = "wasm32"),
            ..Default::default()
        },
        transform: Transform::default().looking_at(-Vec3::Y, Vec3::Z),
        ..Default::default()
    });
    
    let mut cmd = commands.spawn_empty();
    cmd.insert(PbrBundle {
        mesh: meshes.add(Mesh::from(shape::Plane {
            size: 128.0,
            subdivisions: 0,
        })),
        material: materials.add(Color::WHITE.into()),
        ..Default::default()
    });
    cmd.insert(Collider::halfspace(Vec3::Y).unwrap());

    let obstacles_material = materials.add(Color::GRAY.into());
    for ([width, height, depth], transform) in [
        (
            [20.0, 0.1, 2.0],
            Transform::from_xyz(10.0, 10.0, 0.0).with_rotation(Quat::from_rotation_z(0.6)),
        ),
        ([4.0, 2.0, 2.0], Transform::from_xyz(-4.0, 1.0, 0.0)),
        ([6.0, 1.0, 2.0], Transform::from_xyz(-10.0, 4.0, 0.0)),
        ([6.0, 1.0, 2.0], Transform::from_xyz(0.0, 2.6, -5.0)),
    ] {
        let mut cmd = commands.spawn_empty();
        cmd.insert(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Box::new(width, height, depth))),
            material: obstacles_material.clone(),
            transform,
            ..Default::default()
        });
        cmd.insert(Collider::cuboid(0.5 * width, 0.5 * height, 0.5 * depth));
    }
}