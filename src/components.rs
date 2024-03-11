use bevy::{ecs::component::Component, math::Vec2};

/// Component for Axis-aligned bounding boxes
#[derive(Component, Debug, Default)]
pub struct Aabb {
    pub(crate) min: Vec2,
    pub(crate) max: Vec2,
}

impl Aabb {
    pub fn intersects(&self, other: &Self) -> bool {
        self.max.x >= other.min.x
            && self.max.y >= other.min.y
            && self.min.x <= other.max.x
            && self.min.y <= other.max.x
    }
}

#[derive(Component, Debug)]
pub struct BoxCollider {
    pub size: Vec2,
}

impl Default for BoxCollider {
    fn default() -> Self {
        Self { size: Vec2::ONE }
    }
}

#[derive(Component, Debug)]
pub struct CircleCollider {
    pub radius: f32,
}

impl Default for CircleCollider {
    fn default() -> Self {
        Self { radius: 0.5 }
    }
}

#[derive(Component, Debug, Default)]
pub struct Pos(pub Vec2);

#[derive(Component, Debug, Default)]
pub struct PrevPos(pub Vec2);

#[derive(Component, Debug)]
pub struct Mass(pub f32);

impl Default for Mass {
    fn default() -> Self {
        Self(1.) // Default to 1 kg
    }
}

#[derive(Component, Debug)]
pub struct Restitution(pub f32);

impl Default for Restitution {
    fn default() -> Self {
        Self(0.3)
    }
}

#[derive(Component, Debug, Default)]
pub struct PreSolveVel(pub(crate) Vec2);

#[derive(Component, Debug, Default)]
pub struct Vel(pub(crate) Vec2);

#[cfg(test)]
mod tests {
    use super::CircleCollider;
    use float_cmp::approx_eq;

    #[test]
    fn circle_collider_sets_expected_defaults() {
        // arrange
        // act
        let collider = CircleCollider::default();
        let result = collider.radius;

        // assert
        assert!(approx_eq!(
            f32,
            result,
            0.5,
            epsilon = f32::EPSILON,
            ulps = 2
        ));
    }
}
