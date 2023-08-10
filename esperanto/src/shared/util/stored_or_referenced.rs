use std::{cell::Ref, ops::Deref};

#[derive(Debug, Eq, PartialEq)]
pub(crate) enum StoredOrReferenced<'lifetime, StoredType>
where
    StoredType: Eq,
{
    StoredInternally(Box<StoredType>),
    Referenced(&'lifetime StoredType),
}

impl<'lifetime, StoredType> From<Box<StoredType>> for StoredOrReferenced<'lifetime, StoredType>
where
    StoredType: Eq,
{
    fn from(value: Box<StoredType>) -> Self {
        Self::StoredInternally(value)
    }
}

impl<'lifetime, StoredType> From<&'lifetime StoredType>
    for StoredOrReferenced<'lifetime, StoredType>
where
    StoredType: Eq,
{
    fn from(value: &'lifetime StoredType) -> Self {
        Self::Referenced(value)
    }
}

impl<'lifetime, StoredType> Deref for StoredOrReferenced<'lifetime, StoredType>
where
    StoredType: Eq,
{
    type Target = StoredType;

    fn deref(&self) -> &Self::Target {
        match self {
            Self::Referenced(r) => r,
            Self::StoredInternally(s) => s,
        }
    }
}
