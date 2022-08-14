#![allow(dead_code)]

use bevy_ecs::prelude::*;
use bevy_mod_check_filter::*;
use std::ops::Deref;

/// IsVariant will generate three predicates: `IsWalking`, `IsRunning`, and `IsFlying`
#[derive(Component, IsVariant)]
enum EnemyState {
    Walking,
    Running(bool, f32),
    Flying { max_height: f32 },
}

impl Deref for EnemyState {
    type Target = Self;

    fn deref(&self) -> &Self::Target {
        self
    }
}

/// Component that derefs into `EnemyState`
#[derive(Component)]
struct Enemy {
    state: EnemyState,
}
impl Deref for Enemy {
    type Target = EnemyState;

    fn deref(&self) -> &Self::Target {
        &self.state
    }
}

/// works with the enum as a component
fn walking_enemies(_entities: Query<Entity, Check<EnemyState, IsWalking>>) {
    // ...
}

/// and as a Deref target for another component
fn flying_enemies(_entities: Query<Entity, Check<Enemy, IsFlying>>) {
    // ...
}

pub fn main() {}
