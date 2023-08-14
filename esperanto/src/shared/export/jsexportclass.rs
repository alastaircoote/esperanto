use crate::{shared::errors::EsperantoResult, JSContext, JSValue, Retain};

pub enum JSExportAttribute {
    Function(JSClassFunction),
    Property {
        getter: for<'r, 'c, 'v> fn(
            &'c JSContext<'r, 'c>,
            &'v JSValue<'r, 'c>,
        ) -> EsperantoResult<Retain<JSValue<'r, 'c>>>,
        setter: Option<
            for<'r, 'c, 'v> fn(
                &'v JSValue<'r, 'c>,
                &'v JSValue<'r, 'c>,
                &'c JSContext<'r, 'c>,
            ) -> EsperantoResult<JSValue<'r, 'c>>,
        >,
    },
}

pub struct JSClassFunction {
    pub num_args: i32,
    pub func: for<'r, 'c, 'v> fn(
        &'v [&'v JSValue<'r, 'c>],
        &'c JSContext<'r, 'c>,
    ) -> EsperantoResult<Retain<JSValue<'r, 'c>>>,
}

pub trait JSExportClass: 'static {
    const CLASS_NAME: &'static str;
    const ATTRIBUTES: JSExportAttributes = None;
    const CALL_AS_CONSTRUCTOR: Option<JSClassFunction> = None;
    const CALL_AS_FUNCTION: Option<JSClassFunction> = None;
}

pub type JSExportAttributes = Option<phf::OrderedMap<&'static str, JSExportAttribute>>;
pub type JSExportName = &'static str;
