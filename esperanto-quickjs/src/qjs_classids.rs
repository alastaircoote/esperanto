use quickjs_android_suitable_sys::{JSClassID, JS_NewClassID};

static mut CLOSURE_CONTEXT: JSClassID = 0;
static mut CLASSIDS_CREATED: bool = false;

#[derive(Eq, PartialEq, Hash)]
pub enum QJSClassType {
    ClosureContext,
}

pub fn ensure_class_ids_created() {
    unsafe {
        if CLASSIDS_CREATED {
            return;
        }

        CLOSURE_CONTEXT = JS_NewClassID(&mut 0);
        CLASSIDS_CREATED = true;
    }
}
pub fn get_class_id(for_type: QJSClassType) -> JSClassID {
    unsafe {
        match for_type {
            QJSClassType::ClosureContext => CLOSURE_CONTEXT,
        }
    }
}
