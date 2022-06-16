use std::ops::Deref;

pub enum Marking<T> {
    Marked(T),
    Unmarked(T),
}

impl<T> Marking<T> {
    pub fn is_marked(&self) -> bool {
        match self {
            Marking::Marked(_) => true,
            Marking::Unmarked(_) => false,
        }
    }

    pub fn mark(self) -> Self {
        Marking::Marked(self.0)
    }

    pub fn unmark(self) -> Self {
        Marking::Unmarked(self.0)
    }
}

impl<T> Deref for Marking<T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            Marking::Marked(inner) => &inner,
            Marking::Unmarked(inner) => &inner,
        }
    }
}
