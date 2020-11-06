// use std::rc::Rc;

// use crate::errors::JSContextError;

// use super::JSContext;

// pub trait JSClass: Sized + 'static {
//     type ContextType: JSContext<ClassType = Self> + 'static;
//     type RawType: Copy;

//     fn from_raw(
//         raw: Self::RawType,
//         in_context: &Rc<Self::ContextType>,
//     ) -> Result<Self, JSContextError>;
// }
