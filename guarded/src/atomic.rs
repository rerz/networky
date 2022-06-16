use crate::shared::Shared;
use seize::{Guard, Linked};
use std::fmt;
use std::sync::atomic::Ordering;

pub struct CompareExchangeError<'g, T> {
    pub current: Shared<'g, T>,
    pub new: Shared<'g, T>,
}

pub struct Atomic<T>(seize::AtomicPtr<T>);

impl<T> Atomic<T> {
    pub fn null() -> Self {
        Self(seize::AtomicPtr::default())
    }

    pub fn load<'g>(&self, ordering: Ordering, guard: &'g Guard<'_>) -> Shared<'g, T> {
        guard.protect(&self.0, ordering).into()
    }

    pub fn store(&self, new: Shared<'_, T>, ordering: Ordering) {
        self.0.store(new.ptr, ordering);
    }

    pub unsafe fn into_box(self) -> Box<Linked<T>> {
        Box::from_raw(self.0.into_inner())
    }

    pub fn swap<'g>(&self, new: Shared<'_, T>, ord: Ordering, _: &'g Guard<'_>) -> Shared<'g, T> {
        self.0.swap(new.ptr, ord).into()
    }

    pub fn compare_exchange<'g>(
        &self,
        current: Shared<'_, T>,
        new: Shared<'g, T>,
        success: Ordering,
        failure: Ordering,
        _: &'g Guard<'_>,
    ) -> Result<Shared<'g, T>, CompareExchangeError<'g, T>> {
        match self
            .0
            .compare_exchange(current.ptr, new.ptr, success, failure)
        {
            Ok(ptr) => Ok(ptr.into()),
            Err(current) => Err(CompareExchangeError {
                current: current.into(),
                new,
            }),
        }
    }
}

impl<T> From<Shared<'_, T>> for Atomic<T> {
    fn from(shared: Shared<'_, T>) -> Self {
        Atomic(shared.ptr.into())
    }
}

impl<T> Clone for Atomic<T> {
    fn clone(&self) -> Self {
        Atomic(self.0.load(Ordering::Relaxed).into())
    }
}

impl<T> fmt::Debug for Atomic<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:p}", self.0.load(Ordering::SeqCst))
    }
}
