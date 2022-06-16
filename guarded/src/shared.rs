use std::marker::PhantomData;
use seize::{Collector, Guard, Linked};
use std::fmt;
use std::ops::Deref;
use std::ptr;

pub struct Shared<'g, T> {
    pub ptr: *mut Linked<T>,
    _g: PhantomData<&'g ()>,
}

impl<T> fmt::Debug for Shared<'_, T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:p}", self.ptr)
    }
}

impl<'g, T> Shared<'g, T> {
    pub fn null() -> Self {
        Shared::from(ptr::null_mut())
    }

    pub fn boxed(value: T, collector: &Collector) -> Self {
        Shared::from(collector.link_boxed(value))
    }

    pub unsafe fn into_box(self) -> Box<Linked<T>> {
        Box::from_raw(self.ptr)
    }

    pub unsafe fn as_ptr(&self) -> *mut Linked<T> {
        self.ptr
    }

    pub unsafe fn as_ref(&self) -> Option<&'g Linked<T>> {
        self.ptr.as_ref()
    }

    pub unsafe fn deref(&self) -> &'g Linked<T> {
        &*self.ptr
    }

    pub fn is_null(&self) -> bool {
        self.ptr.is_null()
    }
}

impl<'g, T> PartialEq<Shared<'g, T>> for Shared<'g, T> {
    fn eq(&self, other: &Self) -> bool {
        self.ptr == other.ptr
    }
}

impl<T> Eq for Shared<'_, T> {}

impl<T> Clone for Shared<'_, T> {
    fn clone(&self) -> Self {
        Shared::from(self.ptr)
    }
}

impl<T> Copy for Shared<'_, T> {}

impl<T> From<*mut Linked<T>> for Shared<'_, T> {
    fn from(ptr: *mut Linked<T>) -> Self {
        Shared {
            ptr,
            _g: PhantomData,
        }
    }
}

pub trait RetireShared {
    unsafe fn retire_shared<T>(&self, shared: Shared<'_, T>);
}

impl RetireShared for Guard<'_> {
    unsafe fn retire_shared<T>(&self, shared: Shared<'_, T>) {
        self.retire(shared.ptr, seize::reclaim::boxed::<T>);
    }
}

pub enum GuardRef<'g> {
    Owned(Guard<'g>),
    Ref(&'g Guard<'g>),
}

impl<'g> Deref for GuardRef<'g> {
    type Target = Guard<'g>;

    #[inline]
    fn deref(&self) -> &Guard<'g> {
        match *self {
            GuardRef::Owned(ref guard) | GuardRef::Ref(&ref guard) => guard,
        }
    }
}
