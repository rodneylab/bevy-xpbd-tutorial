mod components;
mod entity;

use bevy::{
    app::{App, FixedUpdate, Plugin, Update},
    ecs::{
        entity::Entity,
        query::{With, Without},
        schedule::{IntoSystemConfigs, SystemSet},
        system::{Query, Res, ResMut, Resource},
    },
    math::Vec2,
    time::{Fixed, Time},
};

pub use components::{
    BoxCollider, CircleCollider, Mass, Pos, PreSolveVel, PrevPos, Restitution, Vel,
};
pub use entity::{ParticleBundle, StaticBoxBundle, StaticCircleBundle};

pub const DELTA_TIME: f32 = 1. / 60.; // 60 fps

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
struct FixedUpdateSet;

#[derive(Debug, Default)]
pub struct XPBDPlugin;

#[derive(Default, Debug, Resource)]
pub struct Contacts(pub Vec<(Entity, Entity, Vec2)>);

#[derive(Default, Debug, Resource)]
pub struct StaticContacts(pub Vec<(Entity, Entity, Vec2)>);

#[derive(Debug, Resource)]
pub struct Gravity(pub Vec2);

impl Default for Gravity {
    fn default() -> Self {
        Self(Vec2::new(0., -9.81))
    }
}

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
enum Step {
    CollectCollisionPairs,
    Integrate,
    SolvePositions,
    UpdateVelocities,
    SolveVelocities,
}

impl Plugin for XPBDPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<Gravity>()
            .init_resource::<Contacts>()
            .init_resource::<StaticContacts>();
        app.insert_resource(Time::<Fixed>::from_seconds(DELTA_TIME.into()))
            .add_systems(
                FixedUpdate,
                collect_collision_pairs
                    .in_set(Step::CollectCollisionPairs)
                    .before(Step::Integrate),
            )
            .add_systems(Update, integrate.in_set(Step::Integrate))
            .add_systems(Update, clear_contacts.before(Step::SolvePositions))
            .add_systems(
                Update,
                (solve_pos, solve_pos_statics, solve_pos_static_boxes)
                    .in_set(Step::SolvePositions)
                    .after(Step::Integrate),
            )
            .add_systems(
                Update,
                update_vel
                    .in_set(Step::UpdateVelocities)
                    .after(Step::SolvePositions),
            )
            .add_systems(
                Update,
                (solve_vel, solve_vel_statics)
                    .in_set(Step::SolveVelocities)
                    .after(Step::UpdateVelocities),
            )
            .add_systems(Update, sync_transforms.after(Step::SolveVelocities));
    }
}

fn collect_collision_pairs() {}

fn clear_contacts(mut contacts: ResMut<Contacts>, mut static_contacts: ResMut<StaticContacts>) {
    contacts.0.clear();
    static_contacts.0.clear();
}

fn integrate(
    mut query: Query<(&mut Pos, &mut PrevPos, &mut Vel, &mut PreSolveVel, &Mass)>,
    gravity: Res<Gravity>,
) {
    for (mut pos, mut prev_pos, mut vel, mut pre_sol_velocity, mass) in query.iter_mut() {
        prev_pos.0 = pos.0;

        let gravitation_force = mass.0 * gravity.0;
        let external_forces = gravitation_force;
        vel.0 += DELTA_TIME * external_forces / mass.0;
        pos.0 += DELTA_TIME * vel.0;
        pre_sol_velocity.0 = vel.0;
    }
}

fn solve_pos(
    mut query: Query<(Entity, &mut Pos, &CircleCollider, &Mass)>,
    mut contacts: ResMut<Contacts>,
) {
    let mut iter = query.iter_combinations_mut();
    while let Some(
        [(entity_a, mut pos_a, circle_a, mass_a), (entity_b, mut pos_b, circle_b, mass_b)],
    ) = iter.fetch_next()
    {
        let ab = pos_b.0 - pos_a.0;
        let combined_radius = circle_a.radius + circle_b.radius;
        let ab_sqr_len = ab.length_squared();
        if ab_sqr_len < combined_radius * combined_radius {
            let ab_length = ab_sqr_len.sqrt();
            let penetration_depth = combined_radius - ab_length;
            let normal = ab / ab_length;

            let w_a = 1. / mass_a.0;
            let w_b = 1. / mass_b.0;
            let w_sum = w_a + w_b;

            pos_a.0 -= normal * penetration_depth * w_a / w_sum;
            pos_b.0 += normal * penetration_depth * w_b / w_sum;

            contacts.0.push((entity_a, entity_b, normal));
        }
    }
}

fn solve_pos_statics(
    mut dynamics: Query<(Entity, &mut Pos, &CircleCollider), With<Mass>>,
    statics: Query<(Entity, &Pos, &CircleCollider), Without<Mass>>,
    mut contacts: ResMut<StaticContacts>,
) {
    for (entity_a, mut pos_a, circle_a) in dynamics.iter_mut() {
        for (entity_b, pos_b, circle_b) in statics.iter() {
            let ab = pos_b.0 - pos_a.0;
            let combined_radius = circle_a.radius + circle_b.radius;
            let ab_sqr_len = ab.length_squared();
            if ab_sqr_len < combined_radius * combined_radius {
                let ab_length = ab_sqr_len.sqrt();
                let penetration_depth = combined_radius - ab_length;
                let normal = ab / ab_length;

                pos_a.0 -= normal * penetration_depth;
                contacts.0.push((entity_a, entity_b, normal));
            }
        }
    }
}

fn solve_pos_static_boxes(
    mut dynamics: Query<(Entity, &mut Pos, &CircleCollider), With<Mass>>,
    statics: Query<(Entity, &Pos, &BoxCollider), Without<Mass>>,
    mut contacts: ResMut<StaticContacts>,
) {
    for (entity_a, mut pos_a, circle_a) in dynamics.iter_mut() {
        for (entity_b, pos_b, box_b) in statics.iter() {
            let box_to_circle = pos_a.0 - pos_b.0;
            let box_to_circle_abs = box_to_circle.abs();
            let half_extents = box_b.size / 2.;
            let corner_to_centre = box_to_circle_abs - half_extents;
            let r = circle_a.radius;
            if corner_to_centre.x > r || corner_to_centre.y > r {
                continue;
            }

            let s = box_to_circle.signum();

            let (normal, penetration_depth) = if corner_to_centre.x > 0. && corner_to_centre.y > 0.
            {
                // corner case
                let corner_to_centre_sqr = corner_to_centre.length_squared();
                if corner_to_centre_sqr > r * r {
                    continue;
                }
                let corner_dist = corner_to_centre_sqr.sqrt();
                let penetration_depth = r - corner_dist;
                let normal = corner_to_centre / corner_dist * -s;
                (normal, penetration_depth)
            } else if corner_to_centre.x > corner_to_centre.y {
                // closer to vertical edge
                (Vec2::X * -s.x, -corner_to_centre.x + r)
            } else {
                (Vec2::Y * -s.y, -corner_to_centre.y + r)
            };

            pos_a.0 -= normal * penetration_depth;
            contacts.0.push((entity_a, entity_b, normal));
        }
    }
}

fn update_vel(mut query: Query<(&mut Pos, &mut PrevPos, &mut Vel)>) {
    for (pos, prev_pos, mut vel) in query.iter_mut() {
        vel.0 = (pos.0 - prev_pos.0) / DELTA_TIME;
    }
}

fn solve_vel(query: Query<(&mut Vel, &PreSolveVel, &Mass, &Restitution)>, contacts: Res<Contacts>) {
    for (entity_a, entity_b, normal) in contacts.0.iter().cloned() {
        let (
            (mut vel_a, pre_solve_vel_a, mass_a, restitution_a),
            (mut vel_b, pre_solve_vel_b, mass_b, restitution_b),
        ) = unsafe {
            // Ensure safety
            assert!(entity_a != entity_b);
            (
                query.get_unchecked(entity_a).unwrap(),
                query.get_unchecked(entity_b).unwrap(),
            )
        };
        //let normal = (pos_b.0 - pos_a.0).normalize();
        let pre_solve_relative_vel = pre_solve_vel_a.0 - pre_solve_vel_b.0;
        let pre_solve_normal_vel = Vec2::dot(pre_solve_relative_vel, normal);

        let relative_vel = vel_a.0 - vel_b.0;
        let normal_vel = Vec2::dot(relative_vel, normal);
        let restitution = (restitution_a.0 + restitution_b.0) / 2.;

        let w_a = 1. / mass_a.0;
        let w_b = 1. / mass_b.0;
        let w_sum = w_a + w_b;

        let restitution_velocity = (-restitution * pre_solve_normal_vel).min(0.);
        let vel_impulse = normal * ((-normal_vel + restitution_velocity) / w_sum);

        vel_a.0 += vel_impulse * w_a;
        vel_b.0 -= vel_impulse * w_b;
    }
}

fn solve_vel_statics(
    mut dynamics: Query<(&mut Vel, &PreSolveVel, &Restitution), With<Mass>>,
    statics: Query<&Restitution, Without<Mass>>,
    contacts: Res<StaticContacts>,
) {
    for (entity_a, entity_b, normal) in contacts.0.iter().cloned() {
        let (mut vel_a, pre_solve_vel_a, restitution_a) = dynamics.get_mut(entity_a).unwrap();
        let restitution_b = statics.get(entity_b).unwrap();
        let pre_solve_normal_vel = Vec2::dot(pre_solve_vel_a.0, normal);
        let normal_vel = Vec2::dot(vel_a.0, normal);
        let restitution = (restitution_a.0 + restitution_b.0) / 2.;
        vel_a.0 += normal * (-normal_vel - restitution * pre_solve_normal_vel);
    }
}

/// copies positions from the physics world to bevy Transforms
fn sync_transforms(mut query: Query<(&mut bevy::transform::components::Transform, &Pos)>) {
    for (mut transform, pos) in query.iter_mut() {
        transform.translation = pos.0.extend(0.);
    }
}
