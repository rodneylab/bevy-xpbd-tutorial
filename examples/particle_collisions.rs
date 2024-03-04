#![warn(clippy::all, clippy::pedantic)]

use bevy::prelude::*;
use bevy_xpbd_tutorial::{Gravity, ParticleBundle, XPBDPlugin};

fn startup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let sphere = meshes.add(Sphere::new(0.5).mesh().ico(4).unwrap());

    let white = materials.add(StandardMaterial {
        base_color: Color::WHITE,
        unlit: true,
        ..Default::default()
    });

    commands
        .spawn(PbrBundle {
            mesh: sphere.clone(),
            material: white.clone(),
            ..Default::default()
        })
        .insert(ParticleBundle::new_with_pos_and_vel(
            Vec2::new(-2., 0.),
            Vec2::new(2., 0.),
        ));
    commands
        .spawn(PbrBundle {
            mesh: sphere.clone(),
            material: white.clone(),
            ..Default::default()
        })
        .insert(ParticleBundle::new_with_pos_and_vel(
            Vec2::new(2., 0.),
            Vec2::new(-2., 0.),
        ));

    commands.spawn(Camera3dBundle {
        transform: Transform::from_translation(Vec3::new(0., 0., 100.)),
        projection: bevy::render::camera::Projection::Orthographic(
            bevy::render::camera::OrthographicProjection {
                scale: 0.01,
                ..Default::default()
            },
        ),
        ..Camera3dBundle::default()
    });
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(Msaa::Sample4)
        .add_plugins(DefaultPlugins)
        .add_plugins(XPBDPlugin)
        .insert_resource(Gravity(Vec2::ZERO))
        .add_systems(Startup, startup)
        .add_systems(Update, bevy::window::close_on_esc)
        .run();
}
