use bevy::app::Update;
use bevy::diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin};
use bevy::{
    app::{App, Startup},
    asset::Assets,
    core_pipeline::core_3d::Camera3dBundle,
    ecs::system::{Commands, ResMut},
    math::{
        primitives::{Rectangle, Sphere},
        Vec2, Vec3,
    },
    pbr::{PbrBundle, StandardMaterial},
    render::{
        camera::ClearColor,
        color::Color,
        mesh::{Mesh, Meshable},
        view::Msaa,
    },
    transform::components::Transform,
    DefaultPlugins,
};
use bevy_xpbd_tutorial::{
    BoxCollider, CircleCollider, ParticleBundle, Pos, StaticBoxBundle, XPBDPlugin,
};

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.8, 0.8, 0.9)))
        .insert_resource(Msaa::Sample4)
        .add_plugins(DefaultPlugins)
        .add_plugins(LogDiagnosticsPlugin::default())
        .add_plugins(FrameTimeDiagnosticsPlugin)
        .add_plugins(XPBDPlugin)
        .add_systems(Startup, spawn_camera)
        .add_systems(Startup, spawn_balls)
        .add_systems(Update, bevy::window::close_on_esc)
        .run();
}

fn spawn_camera(mut commands: Commands) {
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

fn spawn_balls(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    let sphere = meshes.add(Sphere::new(1.).mesh().ico(4).unwrap());
    let blue = materials.add(StandardMaterial {
        base_color: Color::rgb(0.4, 0.4, 0.6),
        unlit: true,
        ..Default::default()
    });

    let size = Vec2::new(20., 2.);
    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(Rectangle::from_size(Vec2::ONE))),
            material: blue.clone(),
            transform: Transform::from_scale(size.extend(1.)),
            ..Default::default()
        })
        .insert(StaticBoxBundle {
            pos: Pos(Vec2::new(0., -4.)),
            collider: BoxCollider { size },
            ..Default::default()
        });

    let radius = 0.15;
    let stacks = 5;
    for i in 0..15 {
        for j in 0..stacks {
            let pos = Vec2::new(
                (j as f32 - stacks as f32 / 2.) * 2.5 * radius,
                2. * radius * i as f32 - 2.,
            );
            let vel = Vec2::ZERO;
            commands
                .spawn(PbrBundle {
                    mesh: sphere.clone(),
                    material: blue.clone(),
                    transform: Transform {
                        scale: Vec3::splat(radius),
                        translation: pos.extend(0.),
                        ..Default::default()
                    },
                    ..Default::default()
                })
                .insert(ParticleBundle {
                    collider: CircleCollider { radius },
                    ..ParticleBundle::new_with_pos_and_vel(pos, vel)
                });
        }
    }
}
