use esperanto_macros::js_export;

struct Huh {}

#[js_export]
impl Huh {}

#[test]
fn it_works() {
    Huh::new()
}
