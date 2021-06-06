// use super::errors::EsperantoError;

// pub trait TryAs<T> {
//     fn try_as(&self) -> Result<T, EsperantoError>;
// }

// pub trait TryFromRef<T>: Sized {
//     fn try_from_ref(value: &T) -> Result<Self, EsperantoError>;
// }

// impl<Base, T> TryAs<T> for Base
// where
//     T: TryFromRef<Base>,
// {
//     fn try_as(&self) -> Result<T, EsperantoError> {
//         T::try_from_ref(self)
//     }
// }
