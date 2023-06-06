mod as_value;
mod result;
mod value;
mod value_conversion;
mod value_error;
mod value_internal;

pub use as_value::AsJSValueRef;
// pub use result as JSResult;
pub use value::JSValue;
pub use value_conversion::{JSValueFrom, TryJSValueFrom};
pub use value_error::JSValueError;
pub(crate) use value_internal::JSValueInternal;

pub mod js_result {
    pub use super::result::convert;
    pub use super::result::retain;
}
pub use result::JSResultProcessor;
