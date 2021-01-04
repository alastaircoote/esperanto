use crate::errors::EsperantoResult;
use thiserror::Error;

pub trait Retained {
    fn retain(&self);
    fn release(&self);
    fn is_still_valid(&self) -> bool;
}

#[derive(Debug, Error)]
pub enum RetainError {
    #[error("The underlying value for this unretained instance is gone")]
    UnderlyingValueIsGone,
}

pub struct Unretained<T: Retained> {
    underlying_value: T,
}

impl<T: Retained> Unretained<T> {
    fn new(wrapping: T) -> Self {
        wrapping.release();
        Unretained {
            underlying_value: wrapping,
        }
    }

    fn borrow(&self) -> EsperantoResult<&T> {
        if self.underlying_value.is_still_valid() {
            Ok(&self.underlying_value)
        } else {
            Err(RetainError::UnderlyingValueIsGone.into())
        }
    }
}

#[cfg(test)]
mod test {
    use std::cell::Cell;

    use super::Retained;

    struct DummyRetained {
        retain_count: Cell<usize>,
    }

    impl Retained for DummyRetained {
        fn retain(&self) {
            self.retain_count.set(self.retain_count.get() + 1)
        }
        fn release(&self) {
            self.retain_count.set(self.retain_count.get() - 1);
        }

        fn is_still_valid(&self) -> bool {
            self.retain_count.get() > 0
        }
    }

    #[test]
    fn releases_on_create() {}
}

// impl<T: Retainable> Drop for Retained<T> {
//     fn drop(&mut self) {
//         self.0.release();
//     }
// }

// impl<T: Retainable> Clone for Retained<T>
// where
//     T: Clone,
// {
//     fn clone(&self) -> Self {
//         Self::new(self.0.clone())
//     }
// }
