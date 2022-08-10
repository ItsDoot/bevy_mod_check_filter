use std::{marker::PhantomData, ops::Deref};

use bevy_ecs::component::Component;

mod impls;

/// Filter that selects entities with a component `T`, if the given [`Predicate`] is satisfied.
pub struct Check<T: Component, Pred: Predicate<T>>(PhantomData<T>, PhantomData<Pred>);

/// A trait for implementing versatile pre-filtering of [`Component`]s.
pub trait Predicate<T> {
    /// Checks if the given value is acceptable.
    fn test(test: &T) -> bool;
}

/// The negation of [`Check`].
pub type CheckNot<T, P> = Check<T, Not<T, P>>;

/// A [`Predicate`] that returns the negation of the given predicate.
pub struct Not<T: Component, P: Predicate<T>>(PhantomData<(T, P)>);

impl<T: Component, P: Predicate<T>> Predicate<T> for Not<T, P> {
    fn test(test: &T) -> bool {
        !P::test(test)
    }
}

pub type IsTrue<T> = Check<T, Is<true>>;

/// A [`Predicate`] that checks if the provided `T`, coerced into a `bool`, equals the given `VALUE`.
pub struct Is<const VALUE: bool>;

impl<T, const VALUE: bool> Predicate<T> for Is<VALUE>
where
    T: Deref<Target = bool>,
{
    fn test(test: &T) -> bool {
        **test == VALUE
    }
}
