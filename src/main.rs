#![cfg_attr(feature = "nightly", feature(async_fn_in_trait))]

use std::any::Any;

trait SomeTrait: Any {
    fn some_fn(&self);
}

struct Test;

impl SomeTrait for Test {
    fn some_fn(&self) {
        println!("hello test");
    }
}

struct Test2;

impl SomeTrait for Test2 {
    fn some_fn(&self) {
        println!("hello test");
    }
}

fn some_downcasted(value: &dyn Any) {
    let downcasted = value.downcast_ref::<Box<dyn SomeTrait>>().unwrap();

    downcasted.some_fn();
}

#[tokio::main]
async fn main() {
    let test = Test2;

    let test: Box<dyn SomeTrait + 'static> = Box::new(test);

    let test_any: &dyn Any = &test;

    some_downcasted(test_any);
}
