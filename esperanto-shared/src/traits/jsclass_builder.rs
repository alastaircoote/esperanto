use std::rc::Rc;

use crate::errors::JSContextError;

use super::JSContext;

pub trait JSClassBuilder<ContextType: JSContext> {
    type Definition;

    fn class_prototype_from_definition(
        &self,
        definition: Self::Definition,
        in_context: &Rc<ContextType>,
    ) -> Result<&ContextType::ValueType, JSContextError>;
}
