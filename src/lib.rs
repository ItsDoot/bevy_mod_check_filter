//! [Component]: bevy_ecs::component::Component
//! [Query]: bevy_ecs::system::Query
//! [Check]: crate::Check
//! [Is]: crate::Is
//! [IsTrue]: crate::IsTrue
#![doc = include_str!("../README.md")]

use std::marker::PhantomData;

use bevy_ecs::component::Component;

pub use bevy_mod_check_filter_macros::{lens, FieldCheckable, IsVariant};

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
pub struct Check<T: Checkable, Pred: Predicate<T>>(PhantomData<T>, PhantomData<Pred>);

pub trait Checkable {
    type Component;
    /// Type that is checked against
    type Checked;

    fn get(v: &Self::Component) -> &Self::Checked;
}
impl<T: Component> Checkable for T {
    type Component = Self;
    type Checked = Self;

    fn get(v: &Self::Component) -> &Self::Checked {
        v
    }
}

/// A trait for implementing versatile pre-filtering of [`Component`]s.
pub trait Predicate<T: Checkable> {
    /// Checks if the given value is acceptable.
    fn test(test: &T::Checked) -> bool;
}

/// The negation of [`Check`].
/// 
/// See [`Check`] for an example of non-negation.
pub type CheckNot<T, P> = Check<T, Not<T, P>>;

/// A [`Predicate`] that returns the negation of the given predicate.
/// 
/// Use [`CheckNot`] when you need negation.
pub struct Not<T: Checkable, P: Predicate<T>>(PhantomData<(T, P)>);

impl<T: Checkable, P: Predicate<T>> Predicate<T> for Not<T, P> {
    fn test(test: &T::Checked) -> bool {
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
    T: Checkable<Checked = bool>,
{
    fn test(test: &T::Checked) -> bool {
        *test == VALUE
    }
}

pub struct Compose<A: Checkable, B: Checkable>(PhantomData<(A, B)>);
impl<A, B> Checkable for Compose<A, B>
where
    A: Checkable,
    B: Checkable<Component = A::Checked>,
    A::Checked: 'static,
{
    type Component = A::Component;
    type Checked = B::Checked;

    fn get(v: &Self::Component) -> &Self::Checked {
        <B as Checkable>::get(<A as Checkable>::get(v))
    }
}
