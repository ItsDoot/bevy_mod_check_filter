#![allow(dead_code, clippy::type_complexity)]

use bevy_ecs::prelude::*;
use bevy_mod_check_filter::*;

/// IsVariant will generate three predicates: `IsWalking`, `IsRunning`, and `IsFlying`
#[derive(Component, IsVariant)]
enum EnemyState {
    Walking,
    Running(bool, f32),
    Flying { max_height: f32 },
}

/// Component that derefs into `EnemyState`
#[derive(Component, FieldCheckable)]
struct Enemy {
    state: EnemyState,
    condition: EnemyCondition,
}

#[derive(FieldCheckable)]
struct EnemyCondition {
    alive: EnemyAlive,
    health: u32,
}

#[derive(FieldCheckable)]
struct EnemyAlive {
    alive: bool,
}

/// works with the enum as a component
fn walking_enemies(_entities: Query<Entity, Check<EnemyState, IsWalking>>) {
    // ...
}

/// works with lens
fn running_enemies(_entities: Query<&Enemy, Check<lens!(Enemy::state), IsRunning>>) {
    // ...
}

// complex lenses
fn alive_enemies(
    _entities: Query<
        &Enemy,
        Check<lens!(Enemy::condition, EnemyCondition::alive, EnemyAlive::alive), Is<true>>,
    >,
) {
    // ...
}

pub fn main() {}
