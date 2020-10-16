use esperanto_macros::js_export;

struct Huh {}

#[js_export]
impl Huh {
    fn hmm() -> String {
        "wow".to_string()
    }
}

trait Watt {
    fn two();
}

// #[js_export]
impl Wat for Huh {}
trait Wat {}

trait Wat2<T> {
    fn what();
}

#[test]
fn it_works() {
    Huh::hmm();
    // test22();
}
