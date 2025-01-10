extern crate mikros_macros;

use mikros_macros::*;

#[derive(Env, Debug)]
struct Example {
    name: String,
}

#[test]
fn test_derive_env() {
    let s = Example {};
    println!("{:?}", s);
}