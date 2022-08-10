# bevy_mod_check_filter

A query filter to allow `Enabled`-style marker components without losing the
ergonomics of `ZST`-style marker component filtering!

## Example

Without `bevy_mod_check_filter`:

```rust
#[derive(Component)]
struct Poisoned;

fn all_poisoned(entities: Query<&Name, With<Poisoned>>) {
    // ...
}
```

With `bevy_mod_check_filter`:

```rust
#[derive(Component, Deref)]
struct Poisoned(pub bool);

fn all_poisoned(entities: Query<&Name, Check<Poisoned, Is<true>>>) {
    // ...
}

// OR with one of the provided type aliases:
fn all_poisoned(entities: Query<&Name, IsTrue<Poisoned>>) {
    // ...
}
```

## License

All code in this repository is dual-licensed under either:

- MIT License (LICENSE-MIT file or http://opensource.org/licenses/MIT)
- Apache License, Version 2.0 (LICENSE-APACHE file or
  http://www.apache.org/licenses/LICENSE-2.0)
