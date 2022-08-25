use super::health::Damage;
use crate::common::*;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub enum Team {
    Player,
    Enemy,
}

#[derive(Component)]
pub struct DamageCircle {
    pub damage: Damage,
    pub radius: f32,
    pub team: Team,
}

#[derive(Component)]
pub struct DamageRect {
    pub damage: Damage,
    pub size: Vec2,
    pub team: Team,
}

#[derive(Component)]
pub struct DieOnContact;

//

pub struct DamagePlugin;

impl Plugin for DamagePlugin {
    fn build(&self, app: &mut App) {
        //
    }
}
