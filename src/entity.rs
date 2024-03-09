use bevy::{ecs::bundle::Bundle, math::Vec2};

use crate::{
    BoxCollider, CircleCollider, Mass, Pos, PreSolveVel, PrevPos, Restitution, Vel, SUB_DT,
};

#[derive(Bundle, Default)]
pub struct ParticleBundle {
    pub pos: Pos,
    pub prev_pos: PrevPos,
    pub mass: Mass,
    pub collider: CircleCollider,
    pub vel: Vel,
    pub presolve_vel: PreSolveVel,
    pub restitution: Restitution,
}

#[derive(Bundle, Default)]
pub struct StaticCircleBundle {
    pub pos: Pos,
    pub collider: CircleCollider,
    pub restitution: Restitution,
}

#[derive(Bundle, Default)]
pub struct StaticBoxBundle {
    pub pos: Pos,
    pub collider: BoxCollider,
    pub restitution: Restitution,
}

impl ParticleBundle {
    pub fn new_with_pos_and_vel(pos: Vec2, vel: Vec2) -> Self {
        Self {
            pos: Pos(pos),
            prev_pos: PrevPos(pos - vel * SUB_DT),
            vel: Vel(vel),
            ..Default::default()
        }
    }
}
