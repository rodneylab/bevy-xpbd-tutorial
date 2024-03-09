use bevy::{
    ecs::{entity::Entity, system::Resource},
    math::Vec2,
};

#[derive(Debug, Default, Resource)]
pub(crate) struct CollisionPairs(pub Vec<(Entity, Entity)>);

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

#[derive(Debug, Default, Resource)]
pub struct LoopState {
    pub has_added_time: bool,
    pub accumulator: f32,
    pub current_substep: u32,
    pub substepping: bool,
}
