use crate::{
    errors::{JSContextError, JSEvaluationError},
    traits::{FromJSValue, JSContext, JSValue, ToJSValue},
};
use std::rc::{Rc, Weak};

pub struct FunctionInvocationContext<ContextType: JSContext> {
    pub _this_val: <ContextType::ValueType as JSValue>::RawType,
    pub number_of_arguments: usize,
    pub arguments: *mut <ContextType::ValueType as JSValue>::RawType,
}

pub type FunctionInvocation<ContextType> = Box<
    dyn Fn(
        FunctionInvocationContext<ContextType>,
    ) -> Result<<ContextType as JSContext>::ValueType, JSContextError>,
>;

fn convert_arguments<'a, ContextType: JSContext>(
    closure_context: FunctionInvocationContext<ContextType>,
    expected_arguments: usize,
) -> Result<&'a [<ContextType::ValueType as JSValue>::RawType], JSContextError> {
    if closure_context.number_of_arguments < expected_arguments {
        return Err(JSEvaluationError::NotEnoughArguments {
            expected: expected_arguments,
            actual: closure_context.number_of_arguments,
        }
        .into());
    }

    unsafe {
        Ok(std::slice::from_raw_parts(
            closure_context.arguments,
            expected_arguments,
        ))
    }
}

fn upgrade_context<ContextType: JSContext>(
    weak_ctx: &Weak<ContextType>,
) -> Result<Rc<ContextType>, JSContextError> {
    weak_ctx
        .upgrade()
        .ok_or(JSContextError::ContextAlreadyDestroyed)
}

pub fn wrap_one_argument_closure<Input, Output, ClosureType, ContextType>(
    closure: ClosureType,
    in_context: &Rc<ContextType>,
) -> FunctionInvocation<ContextType>
where
    Input: FromJSValue<ContextType::ValueType> + 'static,
    Output: ToJSValue<ContextType::ValueType> + 'static,
    ClosureType: (Fn(Input) -> Result<Output, JSContextError>) + 'static,
    ContextType: JSContext,
{
    let weak_context = Rc::downgrade(in_context);

    Box::new(move |closure_context| {
        let context = upgrade_context(&weak_context)?;
        let arguments = convert_arguments(closure_context, 1)?;
        let input = Input::from_js_value(ContextType::ValueType::from_raw(arguments[0], &context))?;
        let output = closure(input)?;
        Ok(output.to_js_value(&context)?)
    })
}

pub fn wrap_two_argument_closure<Input1, Input2, Output, ClosureType, ContextType>(
    closure: ClosureType,
    in_context: &Rc<ContextType>,
) -> FunctionInvocation<ContextType>
where
    Input1: FromJSValue<ContextType::ValueType> + 'static,
    Input2: FromJSValue<ContextType::ValueType> + 'static,
    Output: ToJSValue<ContextType::ValueType> + 'static,
    ClosureType: (Fn(Input1, Input2) -> Result<Output, JSContextError>) + 'static,
    ContextType: JSContext,
{
    let weak_context = Rc::downgrade(in_context);

    Box::new(move |closure_context| {
        let context = upgrade_context(&weak_context)?;
        let arguments = convert_arguments(closure_context, 2)?;
        let input1 =
            Input1::from_js_value(ContextType::ValueType::from_raw(arguments[0], &context))?;
        let input2 =
            Input2::from_js_value(ContextType::ValueType::from_raw(arguments[1], &context))?;
        let output = closure(input1, input2)?;
        Ok(output.to_js_value(&context)?)
    })
}
