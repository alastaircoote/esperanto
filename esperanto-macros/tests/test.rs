use std::hash::Hash;

use esperanto_macros::js_export;
#[js_export]
trait One: Hash {
    // #[js_function(name = "a")]
    fn one(&self, test: &TestStruct, test2: &TestStruct) -> TestStruct;
}

trait Two: One {
    fn two();
}

const TEST: i8 = 1;

struct TestStruct {}

#[derive(Hash)]
struct DoIt {}

// impl One for DoIt {}

// impl One for TestStruct {
//     fn one() {
//         todo!()
//     }
// }

// impl Two for TestStruct {
//     fn two() {
//         todo!()
//     }
// }

// struct Huh {}

// #[js_export]
// impl Huh {
//     fn hmm() -> String {
//         "wow".to_string()
//     }
// }

// trait Watt {
//     fn two();
// }

// // #[js_export]
// impl Wat for Huh {}
// trait Wat {}

// trait Wat2<T> {
//     fn what();
// }

#[test]
fn it_works() {}
