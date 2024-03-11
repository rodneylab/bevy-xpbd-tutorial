mod components;
mod contact;
mod entity;
mod resources;

use bevy::{
    app::{App, FixedUpdate, Plugin, Update},
    ecs::{
        entity::Entity,
        query::{With, Without},
        schedule::{IntoSystemConfigs, Schedule, ScheduleLabel, SystemSet},
        system::{Query, Res, ResMut},
        world::World,
    },
    log::debug,
    math::Vec2,
    time::{Fixed, Time},
};

pub use components::{
    Aabb, BoxCollider, CircleCollider, Mass, Pos, PreSolveVel, PrevPos, Restitution, Vel,
};
pub use contact::Contact;
pub use entity::{DynamicBoxBundle, ParticleBundle, StaticBoxBundle, StaticCircleBundle};
pub use resources::Gravity;
use resources::{CollisionPairs, Contacts, StaticContacts};

pub const DELTA_TIME: f32 = 1.0 / 60.0; // 60 fps
pub const NUM_SUBSTEPS: u32 = 10;
pub const SUB_DT: f32 = DELTA_TIME / NUM_SUBSTEPS as f32;

/// Safety margin bigger then DELTA_TIME added to AABBs to account for sudden accelerations
const COLLISION_PAIR_VEL_MARGIN_FACTOR: f32 = 2.0 * DELTA_TIME;

#[derive(Debug, Hash, PartialEq, Eq, Clone, SystemSet)]
struct FixedUpdateSet;

#[derive(Debug, Default)]
pub struct XPBDPlugin;

#[derive(SystemSet, Debug, Hash, PartialEq, Eq, Clone)]
enum Step {
    CollectCollisionPairs,
    Integrate,
    SolvePositions,
    UpdateVelocities,
    SolveVelocities,
    Substeps,
}

#[derive(ScheduleLabel, Debug, Hash, PartialEq, Eq, Clone)]
struct SubstepSchedule;

fn run_substep_schedule(world: &mut World) {
    for _substep in 0..NUM_SUBSTEPS {
        world.run_schedule(SubstepSchedule);
    }
}
impl Plugin for XPBDPlugin {
    fn build(&self, app: &mut App) {
        let mut substep_schedule = Schedule::new(SubstepSchedule);
        substep_schedule
            .add_systems(integrate.in_set(Step::Integrate))
            .add_systems(
                (
                    solve_pos,
                    solve_pos_box_box,
                    solve_pos_statics,
                    solve_pos_static_boxes,
                    solve_pos_static_box_box,
                )
                    .in_set(Step::SolvePositions)
                    .after(Step::Integrate),
            )
            .add_systems(
                update_vel
                    .in_set(Step::UpdateVelocities)
                    .after(Step::SolvePositions),
            )
            .add_systems(
                (solve_vel, solve_vel_statics)
                    .in_set(Step::SolveVelocities)
                    .after(Step::UpdateVelocities),
            );
        app.init_resource::<Gravity>()
            .init_resource::<CollisionPairs>()
            .init_resource::<Contacts>()
            .init_resource::<StaticContacts>();
        app.add_schedule(substep_schedule);
        app.insert_resource(Time::<Fixed>::from_seconds(DELTA_TIME.into()))
            .add_systems(
                FixedUpdate,
                (update_aabb_box, update_aabb_circle).before(Step::CollectCollisionPairs),
            )
            .add_systems(
                Update,
                collect_collision_pairs
                    .in_set(Step::CollectCollisionPairs)
                    .before(Step::Substeps),
            )
            .add_systems(
                Update,
                run_substep_schedule
                    .in_set(Step::Substeps)
                    .before(Step::SolveVelocities),
            )
            .add_systems(Update, sync_transforms.after(Step::Substeps));
    }
}

fn collect_collision_pairs(
    query: Query<(Entity, &Aabb)>,
    mut collision_pairs: ResMut<CollisionPairs>,
) {
    collision_pairs.0.clear();

    unsafe {
        for (entity_a, aabb_a) in query.iter_unsafe() {
            for (entity_b, aabb_b) in query.iter_unsafe() {
                // Ensure safety
                if entity_a <= entity_b {
                    continue;
                }
                if aabb_a.intersects(aabb_b) {
                    collision_pairs.0.push((entity_a, entity_b));
                }
            }
        }
    }
}

//fn clear_contacts(mut contacts: ResMut<Contacts>, mut static_contacts: ResMut<StaticContacts>) {
//    contacts.0.clear();
//    static_contacts.0.clear();
//}

fn integrate(
    mut query: Query<(&mut Pos, &mut PrevPos, &mut Vel, &mut PreSolveVel, &Mass)>,
    gravity: Res<Gravity>,
) {
    for (mut pos, mut prev_pos, mut vel, mut pre_sol_velocity, mass) in query.iter_mut() {
        prev_pos.0 = pos.0;

        let gravitation_force = mass.0 * gravity.0;
        let external_forces = gravitation_force;
        vel.0 += SUB_DT * external_forces / mass.0;
        pos.0 += SUB_DT * vel.0;
        pre_sol_velocity.0 = vel.0;
    }
}

fn constrain_body_positions(
    pos_a: &mut Pos,
    pos_b: &mut Pos,
    mass_a: &Mass,
    mass_b: &Mass,
    normal: Vec2,
    penetration_depth: f32,
) {
    let w_a = 1.0 / mass_a.0;
    let w_b = 1.0 / mass_b.0;
    let w_sum = w_a + w_b;
    let pos_impulse = normal * (-penetration_depth / w_sum);
    pos_a.0 += pos_impulse * w_a;
    pos_b.0 -= pos_impulse * w_b;
}

fn constrain_body_position(pos: &mut Pos, normal: Vec2, penetration_depth: f32) {
    pos.0 -= normal * penetration_depth;
}

fn solve_pos(
    query: Query<(&mut Pos, &CircleCollider, &Mass)>,
    //mut contacts: ResMut<Contacts>,
    collision_pairs: Res<CollisionPairs>,
) {
    debug!("  solve_pos");
    for (entity_a, entity_b) in collision_pairs.0.iter().cloned() {
        if let (Ok((mut pos_a, circle_a, mass_a)), Ok((mut pos_b, circle_b, mass_b))) = unsafe {
            assert!(entity_a != entity_b); // Ensure we don't violate memory constraints
            (query.get_unchecked(entity_a), query.get_unchecked(entity_b))
        } {
            if let Some(Contact {
                normal,
                penetration,
            }) = contact::ball_ball(pos_a.0, circle_a.radius, pos_b.0, circle_b.radius)
            {
                constrain_body_positions(
                    &mut pos_a,
                    &mut pos_b,
                    mass_a,
                    mass_b,
                    normal,
                    penetration,
                );
                // contacts.0.push((entity_a, entity_b, normal));
            }
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
            if let Some(Contact {
                normal,
                penetration,
            }) = contact::ball_ball(pos_a.0, circle_a.radius, pos_b.0, circle_b.radius)
            {
                constrain_body_position(&mut pos_a, normal, penetration);
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
            if let Some(Contact {
                normal,
                penetration,
            }) = contact::ball_box(pos_a.0, circle_a.radius, pos_b.0, box_b.size)
            {
                constrain_body_position(&mut pos_a, normal, penetration);
                contacts.0.push((entity_a, entity_b, normal));
            }
        }
    }
}

fn solve_pos_box_box(
    query: Query<(&mut Pos, &BoxCollider, &Mass)>,
    mut contacts: ResMut<Contacts>,
    collision_pairs: Res<CollisionPairs>,
) {
    for (entity_a, entity_b) in collision_pairs.0.iter().cloned() {
        if let (Ok((mut pos_a, box_a, mass_a)), Ok((mut pos_b, box_b, mass_b))) = unsafe {
            assert!(entity_a != entity_b); // Ensure we don't violate memory constraints
            (query.get_unchecked(entity_a), query.get_unchecked(entity_b))
        } {
            if let Some(Contact {
                normal,
                penetration,
            }) = contact::box_box(pos_a.0, box_a.size, pos_b.0, box_b.size)
            {
                constrain_body_positions(
                    &mut pos_a,
                    &mut pos_b,
                    mass_a,
                    mass_b,
                    normal,
                    penetration,
                );
                contacts.0.push((entity_a, entity_b, normal));
            }
        }
    }
}

fn solve_pos_static_box_box(
    mut dynamics: Query<(Entity, &mut Pos, &BoxCollider), With<Mass>>,
    statics: Query<(Entity, &Pos, &BoxCollider), Without<Mass>>,
    mut contacts: ResMut<StaticContacts>,
) {
    for (entity_a, mut pos_a, box_a) in dynamics.iter_mut() {
        for (entity_b, pos_b, box_b) in statics.iter() {
            if let Some(Contact {
                normal,
                penetration,
            }) = contact::box_box(pos_a.0, box_a.size, pos_b.0, box_b.size)
            {
                constrain_body_position(&mut pos_a, normal, penetration);
                contacts.0.push((entity_a, entity_b, normal));
            }
        }
    }
}

fn update_aabb_circle(mut query: Query<(&mut Aabb, &Pos, &Vel, &CircleCollider)>) {
    for (mut aabb, pos, vel, circle) in query.iter_mut() {
        let margin = COLLISION_PAIR_VEL_MARGIN_FACTOR * vel.0.length();
        let half_extents = Vec2::splat(circle.radius + margin);
        aabb.min = pos.0 - half_extents;
        aabb.max = pos.0 + half_extents;
    }
}

fn update_aabb_box(mut query: Query<(&mut Aabb, &Pos, &Vel, &BoxCollider)>) {
    for (mut aabb, pos, vel, r#box) in query.iter_mut() {
        let margin = COLLISION_PAIR_VEL_MARGIN_FACTOR * vel.0.length();
        let half_extents = r#box.size / 2.0 + Vec2::splat(margin);
        aabb.min = pos.0 - half_extents;
        aabb.max = pos.0 + half_extents;
    }
}

fn update_vel(mut query: Query<(&mut Pos, &mut PrevPos, &mut Vel)>) {
    for (pos, prev_pos, mut vel) in query.iter_mut() {
        vel.0 = (pos.0 - prev_pos.0) / SUB_DT;
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
