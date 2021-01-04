use std::convert::{TryFrom, TryInto};

use crate::errors::EsperantoError;

use super::{jscore_string::JSCoreString, jscore_value::JSCoreValue};

impl TryFrom<&JSCoreValue<'_>> for String {
    type Error = EsperantoError;

    fn try_from(value: &JSCoreValue<'_>) -> Result<Self, Self::Error> {
        let st = &JSCoreString::try_from(value)?;
        st.try_into()
    }
}
