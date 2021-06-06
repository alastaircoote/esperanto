use crate::shared::engine_impl::export::{JSCallAsConstructorImpl, JSCallAsFunctionImpl};

#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
pub struct JSExportMetadata {
    pub class_name: *const u8,
    pub optional: JSExportMetadataOptional,
}

#[derive(Eq, PartialEq, Hash, Copy, Clone, Debug)]
pub struct JSExportMetadataOptional {
    pub call_as_function: Option<JSCallAsFunctionImpl>,
    pub call_as_constructor: Option<JSCallAsConstructorImpl>,
}

pub const OPTIONAL_METADATA_DEFAULT: JSExportMetadataOptional = JSExportMetadataOptional {
    call_as_function: None,
    call_as_constructor: None,
};

pub enum JSExportClassCall {
    Constructor(JSCallAsConstructorImpl),
    Function(JSCallAsFunctionImpl),
}
// pub trait JSExportClass {
//     const CLASS_DEFINITION: JSClassDefinitionImpl;
//     const PROTOTYPE_DEFINITION: Option<JSClassDefinitionImpl>;
// }

pub trait JSExportClass {
    const METADATA: JSExportMetadata;
}

#[macro_export]
macro_rules! js_export_class {
    { $struct_ident: path as $js_class_name: expr =>
        $(call_as_constructor: ($cc_ctx_ident: pat, $cc_values_ident: pat) $cc_stmt:stmt,)?
        $(call_as_function: ($cf_ctx_ident: pat, $cf_this_ident:pat, $cf_values_ident: pat) $cf_stmt:stmt,)?
        $(call: {
            constructor: ($both_cc_ctx_ident: pat, $both_cc_values_ident: pat) $both_cc_stmt:stmt,
            function: ($both_cf_ctx_ident: pat, $both_cf_this_ident:pat, $both_cf_values_ident: pat) $both_cf_stmt:stmt,
        })?

    } => { paste::paste! {

        $(
            $crate::js_export_class_impl!(CALL_AS_CONSTRUCTOR:
                [< __jsexport_call_as_constructor_ $struct_ident:lower >],
                $struct_ident,
                $cc_ctx_ident,
                $cc_values_ident,
                $cc_stmt
            );
        )?

        $(
            $crate::js_export_class_impl!(CALL_AS_FUNCTION:
                [< __jsexport_call_as_function_ $struct_ident:lower >],
                $cf_ctx_ident,
                $cf_this_ident,
                $cf_values_ident,
                $cf_stmt
            );
        )?

        $(
            $crate::js_export_class_impl!(CALL_AS_CONSTRUCTOR:
                [< __jsexport_call_as_constructor_ $struct_ident:lower >],
                $both_struct_ident,
                $both_cc_ctx_ident,
                $both_cc_values_ident,
                $both_cc_stmt
            );

            $crate::js_export_class_impl!(CALL_AS_FUNCTION:
                [< __jsexport_call_as_function_ $struct_ident:lower >],
                $both_cf_ctx_ident,
                $both_cf_this_ident,
                $both_cf_values_ident,
                $both_cf_stmt,
                [< __jsexport_call_as_constructor_ $struct_ident:lower >],
            );
        )?


        impl $crate::export::JSExportClass for $struct_ident {

            const METADATA: $crate::export::JSExportMetadata = $crate::export::JSExportMetadata {
                class_name: concat!(stringify!($struct_ident), "\0").as_ptr(),
                optional: $crate::export::JSExportMetadataOptional {

                    $(
                        #[doc = "" $cc_ctx_ident ""]
                        call_as_constructor: Some([< __jsexport_call_as_constructor_ $struct_ident:lower >]),
                    )?

                    $(
                        #[doc = "" $cf_ctx_ident ""]
                        call_as_function: Some([< __jsexport_call_as_function_ $struct_ident:lower >]),
                    )?


                    ..$crate::export::OPTIONAL_METADATA_DEFAULT
                }
            };

        }


    }};
}

#[cfg(test)]
mod test {

    struct Test {}

    js_export_class! { Test as "Test" =>
        call_as_function: (ctx, _this_obj, _) {
            Ok(JSValueRef::undefined(&ctx))
        },
    }
}
