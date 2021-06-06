use crate::JSValueRef;

pub trait AsJSValueRef {
    fn as_jsvalue(&self) -> &JSValueRef;
}

// impl<'a> AsJSValueRef for Retain<JSValueRef<'a>> {
//     fn as_jsvalue(&self) -> &JSValueRef {
//         &self
//     }
// }
