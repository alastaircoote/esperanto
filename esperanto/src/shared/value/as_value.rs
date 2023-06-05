use crate::JSValue;

pub trait AsJSValueRef {
    fn as_jsvalue(&self) -> &JSValue;
}

// impl<'a> AsJSValueRef for Retain<JSValueRef<'a>> {
//     fn as_jsvalue(&self) -> &JSValueRef {
//         &self
//     }
// }
