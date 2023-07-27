use std::{ops::Deref, rc::Rc, sync::Arc};
/// Enum used to represent a concrete varying pointer type which only needs to be accessed by ref.
/// We avoid adding a `Val(T)` variant in order to keep the size of the enum minimal
pub enum Refs<'a, T> {
    Ref(&'a T),
    Arc(Arc<T>),
    Rc(Rc<T>),
    Box(Box<T>),
}

impl<T> AsRef<T> for Refs<'_, T> {
    fn as_ref(&self) -> &T {
        match self {
            Refs::Ref(r) => r,
            Refs::Arc(a) => a,
            Refs::Rc(r) => r,
            Refs::Box(b) => b,
        }
    }
}

impl<T> Deref for Refs<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            Refs::Ref(r) => r,
            Refs::Arc(a) => a,
            Refs::Rc(r) => r,
            Refs::Box(b) => b,
        }
    }
}

impl<'a, T> From<&'a T> for Refs<'a, T> {
    fn from(r: &'a T) -> Self {
        Self::Ref(r)
    }
}

impl<T> From<Arc<T>> for Refs<'_, T> {
    fn from(a: Arc<T>) -> Self {
        Self::Arc(a)
    }
}

impl<T> From<Rc<T>> for Refs<'_, T> {
    fn from(r: Rc<T>) -> Self {
        Self::Rc(r)
    }
}

impl<T> From<Box<T>> for Refs<'_, T> {
    fn from(b: Box<T>) -> Self {
        Self::Box(b)
    }
}
