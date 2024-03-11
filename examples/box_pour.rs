use bevy::{
    app::{App, FixedUpdate, Startup, Update},
    asset::{Assets, Handle},
    core_pipeline::core_3d::Camera3dBundle,
    ecs::{
        entity::Entity,
        system::{Commands, Query, Res, ResMut, Resource},
    },
    math::{primitives::Rectangle, Vec2, Vec3},
    pbr::{PbrBundle, StandardMaterial},
    render::{camera::ClearColor, color::Color, mesh::Mesh, view::Msaa},
    time::{Fixed, Time},
    transform::components::Transform,
    DefaultPlugins,
};
use bevy_xpbd_tutorial::{BoxCollider, DynamicBoxBundle, Pos, StaticBoxBundle, XPBDPlugin};
use rand::random;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.8, 0.8, 0.9)))
        .insert_resource(Msaa::Sample4)
        .add_plugins(DefaultPlugins)
        .add_plugins(XPBDPlugin)
        .add_systems(Startup, startup)
        .insert_resource(Time::<Fixed>::from_seconds(1. / 2.))
        .add_systems(FixedUpdate, spawn_boxes)
        .add_systems(Update, despawn_boxes)
        .add_systems(Update, bevy::window::close_on_esc)
        .run();
}

#[derive(Debug, Resource)]
struct Materials {
    blue: Handle<StandardMaterial>,
}

#[derive(Debug, Default, Resource)]
struct Meshes {
    quad: Handle<Mesh>,
}

fn startup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let blue = materials.add(StandardMaterial {
        base_color: Color::rgb(0.4, 0.4, 0.6),
        unlit: true,
        ..Default::default()
    });

    let quad = meshes.add(Mesh::from(Rectangle::from_size(Vec2::ONE)));

    let size = Vec2::new(10., 2.);
    commands
        .spawn(PbrBundle {
            mesh: quad.clone(),
            material: blue.clone(),
            transform: Transform::from_scale(size.extend(1.)),
            ..Default::default()
        })
        .insert(StaticBoxBundle {
            pos: Pos(Vec2::new(0., -3.)),
            collider: BoxCollider { size },
            ..Default::default()
        });

    commands.insert_resource(Meshes { quad });
    commands.insert_resource(Materials { blue });

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

fn spawn_boxes(mut commands: Commands, materials: Res<Materials>, meshes: Res<Meshes>) {
    let size = Vec2::splat(0.3);
    let pos = Vec2::new(random::<f32>() - 0.5, random::<f32>() - 0.5) * 0.5 + Vec2::Y * 3.;
    let vel = Vec2::new(random::<f32>() - 0.5, random::<f32>() - 0.5);
    commands
        .spawn(PbrBundle {
            mesh: meshes.quad.clone(),
            material: materials.blue.clone(),
            transform: Transform {
                scale: size.extend(1.),
                translation: pos.extend(0.),
                ..Default::default()
            },
            ..Default::default()
        })
        .insert(DynamicBoxBundle {
            collider: BoxCollider { size },
            ..DynamicBoxBundle::new_with_pos_and_vel(pos, vel)
        });
}

fn despawn_boxes(mut commands: Commands, query: Query<(Entity, &Pos)>) {
    for (entity, pos) in query.iter() {
        if pos.0.y < -20. {
            commands.entity(entity).despawn();
        }
    }
}
