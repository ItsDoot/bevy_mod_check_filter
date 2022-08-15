//! [Component]: bevy_ecs::component::Component
//! [Query]: bevy_ecs::system::Query
//! [Check]: crate::Check
//! [Is]: crate::Is
//! [IsTrue]: crate::IsTrue
#![doc = include_str!("../README.md")]

use std::{marker::PhantomData, ops::Deref};

use bevy_ecs::component::Component;

pub use bevy_mod_check_filter_macros::IsVariant;

mod impls;

/// Filter that selects entities with a component `T`, if the given [`Predicate`] is satisfied.
/// 
/// This can be used in a [`Query`](bevy_ecs::system::Query) if entities with the component `T`
/// must satisfy an arbitrary condition.
/// 
/// # Examples
/// 
/// ```
/// # use bevy_ecs::component::Component;
/// # use bevy_ecs::system::IntoSystem;
/// # use bevy_ecs::system::Query;
/// # use bevy_mod_check_filter::Check;
/// # use bevy_mod_check_filter::Is;
/// # use bevy_mod_check_filter::IsTrue;
/// 
/// #[derive(Component)]
/// struct Poisoned(bool);
/// 
/// impl std::ops::Deref for Poisoned {
///     type Target = bool;
/// 
///     fn deref(&self) -> &Self::Target {
///         &self.0
///     }
/// }
/// 
/// # #[derive(Component)]
/// # struct Name { name: &'static str };
/// 
/// fn find_all_poisoned(query: Query<&Name, Check<Poisoned, Is<true>>>) {
///     for name in &query {
///         println!("{} is poisoned!", name.name);
///     }
/// }
/// # bevy_ecs::system::assert_is_system(find_all_poisoned);
/// 
/// // With type alias:
/// fn all_poisoned(query: Query<&Name, IsTrue<Poisoned>>) {
///     for name in &query {
///         println!("{} is poisoned!", name.name);
///     }
/// } 
/// # bevy_ecs::system::assert_is_system(all_poisoned);
/// ```
pub struct Check<T: Component, Pred: Predicate<T>>(PhantomData<T>, PhantomData<Pred>);

/// A trait for implementing versatile pre-filtering of [`Component`]s.
pub trait Predicate<T> {
    /// Checks if the given value is acceptable.
    fn test(test: &T) -> bool;
}

/// The negation of [`Check`].
/// 
/// See [`Check`] for an example of non-negation.
pub type CheckNot<T, P> = Check<T, Not<T, P>>;

/// A [`Predicate`] that returns the negation of the given predicate.
/// 
/// Use [`CheckNot`] when you need negation.
pub struct Not<T: Component, P: Predicate<T>>(PhantomData<(T, P)>);

impl<T: Component, P: Predicate<T>> Predicate<T> for Not<T, P> {
    fn test(test: &T) -> bool {
        !P::test(test)
    }
}

/// Query filter for checking if the component `T` is coercable as `true`.
/// 
/// See [`Check`] for example usage.
pub type IsTrue<T> = Check<T, Is<true>>;

/// Query filter for checking if the component `T` is coercable as `false`.
/// 
/// See [`Check`] for example usage of its negation, [`IsTrue`].
pub type IsFalse<T> = Check<T, Is<false>>;

/// A [`Predicate`] that checks if the provided `T`, coerced into a `bool`, equals the given `VALUE`.
/// 
/// This predicate is used as part of [`IsTrue`] and [`IsFalse`].
pub struct Is<const VALUE: bool>;

impl<T, const VALUE: bool> Predicate<T> for Is<VALUE>
where
    T: Deref<Target = bool>,
{
    fn test(test: &T) -> bool {
        **test == VALUE
    }
}
