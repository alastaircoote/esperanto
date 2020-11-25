use javascriptcore_sys::{JSObjectRef, OpaqueJSClass};
use std::collections::HashMap;

use crate::jsc_classdefinition::{JSCClassDefinition, JSClassCache};
#[derive(Debug)]
pub struct JSCContextState {
    successfully_finalized: bool,
}

#[derive(Debug)]
pub struct JSCContextGroupState {
    context_states: HashMap<JSObjectRef, JSCContextState>,
    static_classes: HashMap<JSCClassDefinition, *mut OpaqueJSClass>,
    class_prototypes: HashMap<JSCClassDefinition, *mut OpaqueJSClass>,
}

impl JSCContextGroupState {
    pub fn new() -> Self {
        JSCContextGroupState {
            context_states: HashMap::new(),
            static_classes: HashMap::new(),
            class_prototypes: HashMap::new(),
        }
    }

    pub fn add_global_object(&mut self, obj: JSObjectRef) {
        self.context_states.insert(
            obj,
            JSCContextState {
                successfully_finalized: false,
            },
        );
    }

    pub fn remove_global_object(&mut self, obj: JSObjectRef) {
        self.context_states.remove(&obj);
    }
}

impl JSClassCache for JSCContextGroupState {
    fn get_or_create_class(
        &self,
        def: &JSCClassDefinition,
    ) -> Result<*mut javascriptcore_sys::OpaqueJSClass, esperanto_shared::errors::JSContextError>
    {
        if let Some(existing) = self.static_classes.get(&def) {
            return Ok(*existing);
        }

        let class = def.to_jsc_class(self)?;
        self.static_classes.insert(*def.clone(), class);
        Ok(class)
    }
}

impl Drop for JSCContextGroupState {
    fn drop(&mut self) {
        if self.context_states.len() > 0 {
            panic!("Context group state is being dropped when contexts still depend on it")
        }
    }
}
