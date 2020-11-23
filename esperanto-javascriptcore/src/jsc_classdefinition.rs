use std::{ffi::CString, hash::Hash};

use esperanto_shared::errors::JSContextError;
use javascriptcore_sys::{
    JSClassAttributes, JSClassCreate, JSClassDefinition, JSObjectCallAsConstructorCallback,
    JSObjectCallAsFunctionCallback, JSObjectConvertToTypeCallback, JSObjectDeletePropertyCallback,
    JSObjectFinalizeCallback, JSObjectGetPropertyCallback, JSObjectGetPropertyNamesCallback,
    JSObjectHasInstanceCallback, JSObjectHasPropertyCallback, JSObjectInitializeCallback,
    JSObjectSetPropertyCallback, JSPropertyAttributes, OpaqueJSClass,
};
#[derive(Hash, Debug, PartialEq, Eq)]
struct JSCStaticFunction<'a> {
    name: &'a str,
    call_as_function: JSObjectCallAsFunctionCallback,
    attributes: JSPropertyAttributes,
}

#[derive(Hash, Debug, PartialEq, Eq)]
struct JSCStaticValue<'a> {
    name: &'a str,
    get_property: JSObjectGetPropertyCallback,
    set_property: Option<JSObjectSetPropertyCallback>,
    attributes: JSPropertyAttributes,
}

#[derive(Hash, Debug, PartialEq, Eq)]
pub struct JSCClassDefinition<'a> {
    pub class_name: &'a str,
    pub version: i32,
    pub attributes: JSClassAttributes,
    pub parent_class: Option<&'a JSCClassDefinition<'a>>,
    pub static_values: Vec<JSCStaticValue<'a>>,
    pub static_functions: Vec<JSCStaticFunction<'a>>,
    pub initialize: JSObjectInitializeCallback,
    /// The callback invoked when an object is finalized (prepared for garbage
    /// collection). Use this callback to release resources allocated for the
    /// object, and perform other cleanup.
    pub finalize: JSObjectFinalizeCallback,
    /// The callback invoked when determining whether an object has a property.
    ///
    /// If this field is `NULL`, `getProperty` is called instead. The
    /// `hasProperty` callback enables optimization in cases where
    /// only a property's existence needs to be known, not its value,
    /// and computing its value is expensive.
    pub has_property: JSObjectHasPropertyCallback,
    /// The callback invoked when getting a property's value.
    pub get_property: JSObjectGetPropertyCallback,
    /// The callback invoked when setting a property's value.
    pub set_property: JSObjectSetPropertyCallback,
    /// The callback invoked when deleting a property.
    pub delete_property: JSObjectDeletePropertyCallback,
    /// The callback invoked when collecting the names of an object's properties.
    pub get_property_names: JSObjectGetPropertyNamesCallback,
    /// The callback invoked when an object is called as a function.
    pub call_as_function: JSObjectCallAsFunctionCallback,
    /// The callback invoked when an object is used as a constructor in a `new` expression.
    pub call_as_constructor: JSObjectCallAsConstructorCallback,
    /// The callback invoked when an object is used as the target of an `instanceof` expression.
    pub has_instance: JSObjectHasInstanceCallback,
    /// The callback invoked when converting an object to a particular JavaScript type.
    pub convert_to_type: JSObjectConvertToTypeCallback,
}

pub trait JSClassCache<'a> {
    fn get_or_create_class(
        &self,
        def: &'a JSCClassDefinition,
    ) -> Result<*mut OpaqueJSClass, JSContextError>;
}

impl<'a> JSCClassDefinition<'a> {
    pub fn to_jsc_class<Cache: JSClassCache<'a>>(
        &self,
        cache: &Cache,
    ) -> Result<*mut OpaqueJSClass, JSContextError> {
        let name = CString::new(self.class_name)?;
        let def = JSClassDefinition {
            attributes: self.attributes,
            className: name.as_ptr(),
            version: self.version,
            callAsConstructor: self.call_as_constructor,
            callAsFunction: self.call_as_function,
            parentClass: match self.parent_class {
                Some(parent_definition) => cache.get_or_create_class(&parent_definition)?,
                None => std::ptr::null_mut(),
            },
            initialize: self.initialize,
            finalize: self.finalize,
            hasProperty: self.has_property,
            getProperty: self.get_property,
            setProperty: self.set_property,
            deleteProperty: self.delete_property,
            getPropertyNames: self.get_property_names,
            hasInstance: self.has_instance,
            convertToType: self.convert_to_type,
            staticValues: std::ptr::null_mut(),
            staticFunctions: std::ptr::null_mut(),
        };

        Ok(unsafe { JSClassCreate(&def) })
    }
}

// impl Hash for JSCClassDefinition {
//     fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
//         if self.0.parentClass.is_null() == false {
//             panic!("need to implement parent class")
//         }
//         self.0.attributes.hash(state);
//         self.0.className.hash(state);
//         [
//             self.0.callAsConstructor.map(|o| o as *const fn()),
//             self.0.callAsFunction.map(|o| o as *const fn()),
//             self.0.convertToType.map(|o| o as *const fn()),
//             self.0.deleteProperty.map(|o| o as *const fn()),
//             self.0.finalize.map(|o| o as *const fn()),
//             self.0.getProperty.map(|o| o as *const fn()),
//             self.0.getPropertyNames.map(|o| o as *const fn()),
//             self.0.hasInstance.map(|o| o as *const fn()),
//             self.0.hasProperty.map(|o| o as *const fn()),
//             self.0.initialize.map(|o| o as *const fn()),
//             self.0.setProperty.map(|o| o as *const fn()),
//             self.0.staticFunctions.map(|o| o as *const fn()),
//             self.0.staticValues.map(|o| o as *const fn()),
//         ].iter()
//         .for_each(|func| {
//             if let Some(exists) = func {
//                 exists.hash(state);
//             }
//         })
// }
