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

pub trait JSExportClass: 'static {
    const CLASS_NAME: &'static str;
    const ATTRIBUTES: JSExportAttributes = None;
    const CALL_AS_CONSTRUCTOR: Option<JSClassFunction> = None;
    const CALL_AS_FUNCTION: Option<JSClassFunction> = None;
    // const METADATA: JSExportMetadata;
}

pub type JSExportAttributes = Option<phf::OrderedMap<&'static str, JSExportAttribute<'static>>>;
pub type JSExportName = &'static str;
