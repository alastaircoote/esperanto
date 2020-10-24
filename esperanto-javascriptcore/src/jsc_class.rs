use esperanto_shared::traits::JSClass;

use crate::JSCGlobalContext;

pub struct JSCClass {}

impl JSClass for JSCClass {
    type ContextType = JSCGlobalContext;
}
