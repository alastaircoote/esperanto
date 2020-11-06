// use esperanto_shared::traits::JSClass;
// use javascriptcore_sys::JSClassRef;

// use crate::JSCGlobalContext;

// pub struct JSCClass {
//     pub raw_ref: JSClassRef,
// }

// impl JSClass for JSCClass {
//     type ContextType = JSCGlobalContext;
//     type RawType = JSClassRef;

//     fn from_raw(
//         raw: Self::RawType,
//         in_context: &std::rc::Rc<Self::ContextType>,
//     ) -> Result<Self, esperanto_shared::errors::JSContextError> {
//         Ok(JSCClass { raw_ref: raw })
//     }
// }
