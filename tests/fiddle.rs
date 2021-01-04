// use esperanto::{js_export_method, js_export_test, private::jscore_context::JSCoreContext};
use std::any::TypeId;
use std::convert::TryFrom;
// use Hmm::what::wat::hmm;
use esperanto::private::jscore_export::JSExportError;

struct Hmm {}

impl Hmm {
    fn what(strr: &String, num: f64) -> String {
        let mut i = 0.0;
        let mut st: String = "".to_string();
        while i < num {
            i = i + 1.0;
            st.push_str(&strr)
        }
        st
    }
}

esperanto::js_export_definition!("Hmm" => Hmm, {
    static_functions: {
        "testFunction2" => test_f |args| {
            let s = String::try_from(&args
                .next()
                .ok_or(JSExportError::NotEnoughArguments)??)?;

            let f = f64::try_from(&args.next().ok_or(JSExportError::NotEnoughArguments)??)?;
            Ok(Hmm::what(&s,f))
        }
    }
    // "testFunction" => func(what, 1),
    // "testFunction" => func(what, 1, selfa)
});

#[cfg(test)]
mod test {
    use esperanto::private::jscore_runtime::JSCoreRuntime;
    use esperanto::traits::JSRuntime;

    #[test]
    fn creates_global_object() {
        let rt = JSCoreRuntime::new().unwrap();
        let ctx = rt
            .create_context_with_global_object::<super::Hmm>()
            .unwrap();

        loop {
            //hmm
        }

        drop(ctx);
    }
}
