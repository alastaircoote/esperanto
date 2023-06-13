use crate::{shared::errors::EsperantoResult, JSContext, JSValue, Retain};

pub enum JSExportAttribute<'a> {
    Function(JSClassFunction),
    Property {
        getter: &'a dyn for<'c> Fn(
            &'c JSContext<'c>,
            &'c JSValue<'c>,
        ) -> EsperantoResult<Retain<JSValue<'c>>>,
        setter: Option<
            &'a dyn for<'c> Fn(
                &'c JSValue<'c>,
                &'c JSValue<'c>,
                &'c JSContext<'c>,
            ) -> EsperantoResult<JSValue<'c>>,
        >,
    },
}

pub struct JSClassFunction {
    pub num_args: i32,
    pub func:
        for<'c> fn(&'c Vec<JSValue<'c>>, &'c JSContext<'c>) -> EsperantoResult<Retain<JSValue<'c>>>,
}

// pub enum JSExportCall<T: 'static> {
//     AsConstructor(&'static dyn for<'a> Fn(&Vec<JSValue<'a>>) -> EsperantoResult<T>),
// }

pub struct JSExportMetadata {
    pub class_name: &'static str,
    pub attributes: Option<phf::OrderedMap<&'static str, JSExportAttribute<'static>>>,
    pub call_as_constructor: Option<JSClassFunction>,
    pub call_as_function: Option<JSClassFunction>,
}

pub trait JSExportClass {
    const METADATA: JSExportMetadata;
}
