#![allow(dead_code)]

use bevy_ecs::prelude::*;
use bevy_mod_check_filter::*;

/// IsVariant will generate three predicates: `IsWalking`, `IsRunning`, and `IsFlying`
#[derive(Component, IsVariant)]
enum EnemyState {
    Walking,
    Running(bool, f32),
    Flying { max_height: f32 },
}

/// works with the enum as a component
fn walking_enemies(_entities: Query<Entity, Check<EnemyState, IsWalking>>) {
    // ...
}

pub fn main() {}
