// use std::{collections::HashMap, marker::PhantomData, rc::Rc};

// use super::closures::{wrap_one_argument_closure, wrap_zero_argument_closure, FunctionToInvoke};
// use crate::{
//     errors::JSContextError, errors::JSEvaluationError, traits::FromJSValue, traits::JSContext,
//     traits::ToJSValue,
// };

// pub trait JSClassBuilderOutput<ContextType: JSContext> {
//     fn build_class(
//         self,
//         in_context: &Rc<ContextType>,
//     ) -> Result<ContextType::ValueType, JSContextError>;
// }

// pub struct JSClassBuilder<ContextType: JSContext, NativeObject> {
//     pub name: &'static str,
//     pub constructor: Option<
//         Box<
//             dyn Fn(
//                 &Rc<ContextType>,
//                 Vec<ContextType::ValueType>,
//             ) -> Result<NativeObject, JSContextError>,
//         >,
//     >,
//     pub methods: HashMap<
//         &'static str,
//         Box<
//             dyn Fn(
//                 &Rc<NativeObject>,
//                 &Rc<ContextType>,
//                 Vec<ContextType::ValueType>,
//             ) -> Result<ContextType::ValueType, JSContextError>,
//         >,
//     >,
// }

// impl<ContextType: JSContext, NativeObject> JSClassBuilder<ContextType, NativeObject> {
//     pub fn new(name: &'static str) -> Self {
//         JSClassBuilder {
//             name,
//             constructor: None,
//             methods: HashMap::new(),
//         }
//     }

//     pub fn set_constructor_zero_args<F: Fn() -> Result<NativeObject, JSContextError> + 'static>(
//         mut self,
//         func: F,
//     ) -> Self {
//         self.constructor = Some(Box::new(move |_, _| func()));
//         self
//     }

//     pub fn set_constructor_one_arg<
//         I: FromJSValue<ContextType::ValueType> + 'static,
//         F: Fn(I) -> Result<NativeObject, JSContextError> + 'static,
//     >(
//         mut self,
//         func: F,
//     ) -> Self {
//         self.constructor = Some(Box::new(
//             move |ctx, mut args: Vec<ContextType::ValueType>| {
//                 if args.len() < 1 {
//                     return Err(JSEvaluationError::NotEnoughArguments {
//                         expected: 1,
//                         actual: args.len(),
//                     }
//                     .into());
//                 }

//                 let arg = args.remove(0);
//                 func(I::from_js_value(arg)?)
//             },
//         ));
//         self
//     }

//     // pub fn set_one_argument_constructor<
//     //     I: FromJSValue<ContextType::ValueType> + 'static,
//     //     F: Fn(I) -> Result<NativeObject, JSContextError> + 'static,
//     // >(
//     //     mut self,
//     //     func: F,
//     // ) -> Self {
//     //     let c = wrap_one_argument_closure(func, &self.context);
//     //     self.constructor = Some(c);
//     //     self
//     // }

//     // pub fn add_one_argument_method<
//     //     I: FromJSValue<ContextType::ValueType> + 'static,
//     //     F: Fn(I) -> Result<NativeObject, JSContextError> + 'static,
//     // >(
//     //     mut self,
//     //     name: &'static str,
//     //     func: F,
//     // ) -> Self {
//     //     let c = wrap_one_argument_closure(func, &self.context);
//     //     self.methods.insert(name, c);
//     //     self
//     // }

//     pub fn add_zero_argument_method<
//         O: ToJSValue<ContextType::ValueType>,
//         F: (Fn(&NativeObject) -> Result<O, JSContextError>) + 'static,
//     >(
//         mut self,
//         name: &'static str,
//         func: F,
//     ) -> Self {
//         self.methods.insert(
//             name,
//             Box::new(move |self_object, ctx, _| func(&self_object)?.to_js_value(&ctx)),
//         );
//         self
//     }
// }

// #[cfg(test)]
// mod test {
//     fn test() {}
// }
