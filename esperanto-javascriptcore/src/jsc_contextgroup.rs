use std::{cell::RefCell, collections::HashMap, hash::Hash, ops::Deref, ops::DerefMut, rc::Rc};

use esperanto_shared::{
    errors::{JSContextError, JSEvaluationError},
    traits::{JSClassBuilder, JSRuntime},
};
use javascriptcore_sys::{
    JSClassCreate, JSClassDefinition, JSClassRelease, JSContextGroupCreate, JSContextGroupRelease,
    JSContextGroupRetain, JSContextRef, JSObjectCallAsFunction, JSObjectGetPrivate, JSObjectMake,
    JSObjectMakeFunctionWithCallback, JSObjectRef, JSObjectSetPrivate, JSObjectSetPrototype,
    JSValueRef, JSValueUnprotect, OpaqueJSContextGroup,
};

use crate::{JSCGlobalContext, JSCValue};

#[derive(Hash, Debug, PartialEq, Eq)]
struct JSCClassMetadata {
    get_class_definition: fn() -> &'static JSClassDefinition,
}

type ClassPrototypeStorage = Rc<RefCell<HashMap<ComparableJSClassDefinition, JSObjectRef>>>;

#[derive(Debug)]
pub struct JSCContextGroup {
    pub(crate) raw_ref: *const OpaqueJSContextGroup,
    class_objects: ClassPrototypeStorage,
}

impl Hash for JSCContextGroup {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        // Not 100% sure if this will be a good idea or not, but we're ingoring the class_objects
        // HashMap because it isn't hashable in itself. But we should still be getting unique values
        // based on the raw pointer.
        self.raw_ref.hash(state)
    }
}

impl JSRuntime for JSCContextGroup {
    type ContextType = JSCGlobalContext;

    fn new() -> Result<Rc<Self>, JSContextError> {
        let raw_ref = unsafe { JSContextGroupCreate() };
        // unsafe { JSContextGroupRetain(raw_ref) };
        Ok(Rc::new(JSCContextGroup {
            raw_ref,
            class_objects: Rc::new(RefCell::new(HashMap::new())),
        }))
    }

    fn create_context(self: &Rc<Self>) -> Result<Rc<Self::ContextType>, JSContextError> {
        JSCGlobalContext::new_with_group(Some(self))
    }
}

/// When JSC is done with the class prototype (i.e. no objects are using it as a prototype any more and
/// a garbage collection has completed) we need to remove the reference from our store. We store this
/// struct as private data in the prototype to ensure we can do that:
struct ClassPrototypeFinalizer {
    group_prototype_storage: ClassPrototypeStorage,
    metadata: &'static ComparableJSClassDefinition,
}

// This gets called whenever a class definition has been garbage collected
unsafe extern "C" fn class_def_finalizer(val: *mut javascriptcore_sys::OpaqueJSValue) {
    // First we grab the ClassPrototypeFinalizer from the private data of the object:
    let finalizer_data = Box::from_raw(JSObjectGetPrivate(val) as *mut ClassPrototypeFinalizer);

    // Then remove this class definition from our store, since it isn't valid any more:
    finalizer_data
        .group_prototype_storage
        .borrow_mut()
        .remove(finalizer_data.metadata);

    if let Some(custom_finalizer) = finalizer_data.metadata.finalize {
        // If the definition has a custom finalizer, call that too
        custom_finalizer(val);
    }

    // At this point the finalizer_data variable will go out of scope and be freed, so the reference count for the
    // context group will be reduced
}

#[derive(Debug)]
struct ComparableJSClassDefinition(JSClassDefinition);

impl Hash for ComparableJSClassDefinition {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        (&self.0 as *const JSClassDefinition).hash(state)
    }
}

impl PartialEq for ComparableJSClassDefinition {
    fn eq(&self, other: &Self) -> bool {
        (&self.0 as *const JSClassDefinition) == (&other.0 as *const JSClassDefinition)
    }
}

impl Eq for ComparableJSClassDefinition {}

impl Deref for ComparableJSClassDefinition {
    type Target = JSClassDefinition;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

unsafe impl Sync for ComparableJSClassDefinition {}

impl JSCContextGroup {
    fn get_or_create_prototype_object(
        self: &Rc<Self>,
        prototype: &'static ComparableJSClassDefinition,
        in_context: &Rc<JSCGlobalContext>,
    ) -> Result<JSObjectRef, JSContextError> {
        if let Some(existing_value) = self.class_objects.borrow().get(&prototype) {
            // If we've already created the prototype for this class then just return that
            // immediately
            return Ok(*existing_value);
        }

        let saved_finalizer_data = Box::new(ClassPrototypeFinalizer {
            group_prototype_storage: self.class_objects.clone(),
            metadata: prototype,
        });

        // Our dirty secret is that despite using the JSClassDefinition as an identifier everywhere
        // we don't actually use it to create the class itself, since we need to replace the
        // finalizer specified in the definition with our own one. The definition's own finalizer
        // still gets called after our own finalizer has done its thing.
        let mut modified_definition: JSClassDefinition = prototype.0;
        modified_definition.finalize = Some(class_def_finalizer);

        let class = unsafe { JSClassCreate(&modified_definition) };
        let obj = unsafe {
            JSObjectMake(
                in_context.raw_ref,
                class,
                Box::into_raw(saved_finalizer_data) as *mut std::ffi::c_void,
            )
        };

        // We rely on JSC handling the class reference from now on: since it's connected to the JSObject
        // withing JSC's garbage collector it'll be removed whenever the prototype object is.
        unsafe { JSClassRelease(class) };
        Ok(obj as JSObjectRef)
    }

    fn apply_prototype_to_object(
        self: &Rc<Self>,
        prototype: &'static ComparableJSClassDefinition,
        target_object: &JSCValue,
        in_context: &Rc<JSCGlobalContext>,
    ) -> Result<(), JSContextError> {
        let prototype_object = self.get_or_create_prototype_object(prototype, &in_context)?;
        let obj_ref = target_object
            .raw_ref
            .get_jsobject()
            .ok_or(JSEvaluationError::IsNotAnObject)?;
        unsafe { JSObjectSetPrototype(in_context.raw_ref, obj_ref, prototype_object) };
        unsafe { JSValueUnprotect(in_context.raw_ref, prototype_object) };
        Ok(())
    }
}

impl Drop for JSCContextGroup {
    fn drop(&mut self) {
        unsafe { JSContextGroupRelease(self.raw_ref) }
    }
}

#[cfg(test)]
mod test {
    use std::{cell::RefCell, rc::Rc};

    use crate::test_util::finalize_checker::{make_finalize_checker, FinalizerChecker};
    use esperanto_shared::traits::JSRuntime;
    use javascriptcore_sys::{
        JSClassCreate, JSClassDefinition, JSObjectMake, JSObjectSetPrototype,
    };

    use super::{ComparableJSClassDefinition, JSCClassMetadata, JSCContextGroup};

    #[test]
    // This is largely just for me to ensure that retain/release stuff works the way I think it does.
    fn check_values_release_as_expected() {
        let group = JSCContextGroup::new().unwrap();
        let ctx = group.create_context().unwrap();

        let (finalize_check, js_val) = make_finalize_checker(&ctx);

        // Normally these references would stay valid until the end of the function, so our object
        // wouldn't get finalized until after our check. Instead, let's manually drop them:
        drop(ctx);
        drop(group);
        drop(js_val);

        // Now we can run our finalizer check and make sure we didn't make a giant memory leak
        // or something:
        assert_eq!(finalize_check.is_finalized(), true);
    }

    macro_rules! default_js_class_definition {
        ($var:ident, $final:ident) => {
            static $var: ComparableJSClassDefinition =
                ComparableJSClassDefinition(JSClassDefinition {
                    version: 0,
                    className: std::ptr::null_mut(),
                    attributes: 0,
                    callAsConstructor: None,
                    callAsFunction: None,
                    parentClass: std::ptr::null_mut(),
                    staticValues: std::ptr::null_mut(),
                    staticFunctions: std::ptr::null_mut(),
                    initialize: None,
                    finalize: Some($final),
                    hasProperty: None,
                    setProperty: None,
                    getProperty: None,
                    deleteProperty: None,
                    getPropertyNames: None,
                    hasInstance: None,
                    convertToType: None,
                });
        };
    }

    #[test]
    fn check_prototypes_release() {
        static mut DID_FINALIZE: bool = false;

        unsafe extern "C" fn finalize(_: *mut javascriptcore_sys::OpaqueJSValue) {
            println!("FINALIZE?!?!?");
            DID_FINALIZE = true;
        }

        default_js_class_definition!(META, finalize);

        let group = JSCContextGroup::new().unwrap();
        let context = group.create_context().unwrap();

        let (finalize_check, js_val) = make_finalize_checker(&context);

        // let cl = unsafe { JSClassCreate(&META.0) };
        // let obj = unsafe { JSObjectMake(context.raw_ref, cl, std::ptr::null_mut()) };
        // unsafe {
        //     JSObjectSetPrototype(context.raw_ref, js_val.raw_ref.get_jsobject().unwrap(), obj)
        // };
        group
            .apply_prototype_to_object(&META, &js_val, &context)
            .unwrap();

        drop(context);
        drop(group);
        drop(js_val);

        assert_eq!(finalize_check.is_finalized(), true);
        unsafe { assert_eq!(DID_FINALIZE, true) }
    }
}
