use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum JSValueError {
    #[error("Cannot upgrade the lifetime of a value from a different context")]
    CannotUpgradeWithDifferentContext,

    #[error("This operation requires the JSValue to be an object, but you have provided a value")]
    IsNotAnObject,

    #[error("This value is not a number")]
    IsNotANumber,

    #[error("You must use 'new' when running a constructor function.")]
    MustUseNewWithConstuctor,

    #[error("Could not store private data in this value")]
    CouldNotStorePrivateData,
}
