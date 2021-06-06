use std::ops::Deref;

use quickjs_android_suitable_sys::JSContext;

#[derive(Debug, Clone, Copy)]
pub struct QuickJSContextPointer {
    ptr: *mut JSContext,
    // _phantom: PhantomData<&'c ()>,
    pub(super) retained: bool,
}

impl QuickJSContextPointer {
    pub(crate) fn wrap_retained(ptr: *mut JSContext) -> Self {
        QuickJSContextPointer {
            ptr,
            retained: true,
        }
    }
}

impl<'c> PartialEq for QuickJSContextPointer {
    fn eq(&self, other: &Self) -> bool {
        let const_self = self.deref();
        let const_other = other.deref();
        const_self == const_other
    }
}

impl<'c> Deref for QuickJSContextPointer {
    type Target = *mut JSContext;

    fn deref(&self) -> &Self::Target {
        &self.ptr
    }
}

// impl DerefMut for QuickJSContextPointer<'_> {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         match self {
//             QuickJSContextPointer::Retained(r) => r,
//             QuickJSContextPointer::Unretained(u) => u,
//         }
//     }
// }

impl<'c> From<*mut JSContext> for QuickJSContextPointer {
    fn from(val: *mut JSContext) -> Self {
        // unretained by default
        QuickJSContextPointer {
            ptr: val,
            retained: false,
        }
    }
}

// impl<'c> AsRawMutPtr<JSContext> for QuickJSContextPointer<'c> {
//     fn as_mut_raw_ptr(&mut self) -> *mut JSContext {
//         match self {
//             QuickJSContextPointer::Retained(r) => *r,
//             QuickJSContextPointer::Unretained(u) => *u,
//         }
//     }
// }

// impl From<QuickJSContextPointer> for *mut JSContext {
//     fn from(val: QuickJSContextPointer) -> Self {
//         match val {
//             QuickJSContextPointer::Retained(p) => p,
//             QuickJSContextPointer::Unretained(p) => p,
//         }
//     }
// }

// impl From<&QuickJSContextPointer> for *mut JSContext {
//     fn from(val: &QuickJSContextPointer) -> Self {
//         match val {
//             QuickJSContextPointer::Retained(p) => *p,
//             QuickJSContextPointer::Unretained(p) => *p,
//         }
//     }
// }
