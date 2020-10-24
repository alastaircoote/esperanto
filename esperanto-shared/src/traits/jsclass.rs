use super::JSContext;

pub trait JSClass: Sized + 'static {
    type ContextType: JSContext<ClassType = Self> + 'static;
}
