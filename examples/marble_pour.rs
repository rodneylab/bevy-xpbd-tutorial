use bevy::{
    app::{App, FixedUpdate, Startup, Update},
    asset::{Assets, Handle},
    core_pipeline::core_3d::Camera3dBundle,
    ecs::{
        entity::Entity,
        system::{Commands, Query, Res, ResMut, Resource},
    },
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
    time::{Fixed, Time},
    transform::components::Transform,
    DefaultPlugins,
};
use bevy_xpbd_tutorial::{
    BoxCollider, CircleCollider, ParticleBundle, Pos, StaticBoxBundle, XPBDPlugin,
};
use rand::random;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb(0.8, 0.8, 0.9)))
        .insert_resource(Msaa::Sample4)
        .add_plugins(DefaultPlugins)
        .add_plugins(XPBDPlugin)
        .add_systems(Startup, startup)
        .insert_resource(Time::<Fixed>::from_seconds(1. / 20.))
        .add_systems(FixedUpdate, spawn_marbles)
        .add_systems(Update, despawn_marbles)
        .add_systems(Update, bevy::window::close_on_esc)
        .run();
}

#[derive(Debug, Resource)]
struct Materials {
    blue: Handle<StandardMaterial>,
}

#[derive(Debug, Resource)]
struct Meshes {
    sphere: Handle<Mesh>,
}

fn startup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let sphere = meshes.add(Sphere::new(1.).mesh().ico(4).unwrap());
    let blue = materials.add(StandardMaterial {
        base_color: Color::rgb(0.4, 0.4, 0.6),
        unlit: true,
        ..Default::default()
    });

    let size = Vec2::new(10., 2.);
    commands
        .spawn(PbrBundle {
            mesh: meshes.add(Mesh::from(Rectangle::from_size(Vec2::ONE))),
            material: blue.clone(),
            transform: Transform::from_scale(size.extend(1.)),
            ..Default::default()
        })
        .insert(StaticBoxBundle {
            pos: Pos(Vec2::new(0., -3.)),
            collider: BoxCollider { size },
            ..Default::default()
        });

    commands.insert_resource(Meshes { sphere });
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

fn spawn_marbles(mut commands: Commands, materials: Res<Materials>, meshes: Res<Meshes>) {
    let radius = 0.1;
    let pos = Vec2::new(random::<f32>() - 0.5, random::<f32>() - 0.5) * 0.5 + Vec2::Y * 3.;
    let vel = Vec2::new(random::<f32>() - 0.5, random::<f32>() - 0.5);
    commands
        .spawn(PbrBundle {
            mesh: meshes.sphere.clone(),
            material: materials.blue.clone(),
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

fn despawn_marbles(mut commands: Commands, query: Query<(Entity, &Pos)>) {
    for (entity, pos) in query.iter() {
        if pos.0.y < -20. {
            commands.entity(entity).despawn();
        }
    }
}
