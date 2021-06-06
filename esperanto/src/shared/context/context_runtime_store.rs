// use std::ops::Deref;

// use crate::shared::engine_impl::JSContextInternalImpl;
// use crate::shared::runtime::{JSRuntime, JSRuntimeError};

// /// It's very common for a developer to want to use just one JSContext. In order to let
// /// them do that easily we have JSContextRuntimeStore, which stores either a reference
// /// to an external runtime or creates a runtime of its own that it retains internally.
// #[derive(Debug)]
// pub(crate) enum JSContextRuntimeStore<'r> {
//     /// Used when sharing a runtime between contexts.
//     FetchFromContext(JSContextInternalImpl),
//     /// When a context is created individually we use this to store the runtime inside
//     /// the context struct.
//     StoredInternally(JSRuntime<'r>),
// }

// impl<'r> JSContextRuntimeStore<'r> {
//     pub(crate) fn new(wrap_existing: Option<&'r JSRuntime<'r>>) -> Result<Self, JSRuntimeError> {
//         Ok(match wrap_existing {
//             Some(existing) => JSContextRuntimeStore::Reference(existing),
//             None => JSContextRuntimeStore::StoredInternally(JSRuntime::new()?),
//         })
//     }
// }

// impl<'r> Deref for JSContextRuntimeStore<'r> {
//     type Target = JSRuntime<'r>;

//     fn deref(&self) -> &Self::Target {
//         match self {
//             JSContextRuntimeStore::Reference(r) => r,
//             JSContextRuntimeStore::StoredInternally(r) => r,
//         }
//     }
// }

// #[cfg(test)]
// mod test {
//     use super::JSContextRuntimeStore;
//     use crate::shared::runtime::JSRuntime;

//     #[test]
//     fn can_wrap_existing_runtime() {
//         let runtime = JSRuntime::new().unwrap();
//         JSContextRuntimeStore::new(Some(&runtime)).unwrap();
//     }

//     #[test]
//     fn can_create_new_runtime() {
//         JSContextRuntimeStore::new(None).unwrap();
//     }
// }
