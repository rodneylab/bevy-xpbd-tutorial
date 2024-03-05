use bevy::prelude::*;

use crate::{CircleCollider, Mass, Pos, PreSolveVel, PrevPos, Restitution, Vel, DELTA_TIME};

#[derive(Bundle, Default)]
pub struct ParticleBundle {
    pub pos: Pos,
    pub prev_pos: PrevPos,
    pub mass: Mass,
    pub collider: CircleCollider,
    pub vel: Vel,
    pub presolve_vel: PreSolveVel,
    restitution: Restitution,
}

impl ParticleBundle {
    pub fn new_with_pos_and_vel(pos: Vec2, vel: Vec2) -> Self {
        Self {
            pos: Pos(pos),
            prev_pos: PrevPos(pos - vel * DELTA_TIME),
            vel: Vel(vel),
            ..Default::default()
        }
    }
}
