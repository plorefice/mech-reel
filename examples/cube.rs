use std::f32::consts::PI;

use bevy::{color::palettes::css, prelude::*};

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .insert_resource(ClearColor(css::SKY_BLUE.into()))
        .add_systems(Startup, setup)
        .add_systems(Update, animate)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn((
        Mesh3d(meshes.add(Cuboid::default())),
        MeshMaterial3d(materials.add(StandardMaterial {
            base_color: css::SLATE_GREY.into(),
            ..Default::default()
        })),
        Transform::default(),
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: 8000.0,
            ..default()
        },
        Transform::from_xyz(0.0, 2.0, 0.0).with_rotation(Quat::from_rotation_x(-PI / 4.)),
    ));

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0., 3., 5.).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn animate(time: Res<Time>, mut query: Query<&mut Transform, With<Mesh3d>>) {
    for mut transform in &mut query {
        transform.rotation = Quat::from_rotation_y(time.elapsed_secs());
    }
}
