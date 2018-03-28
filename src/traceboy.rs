#![feature(custom_attribute)]
#![feature(slice_patterns)]
#[macro_use]
extern crate enum_primitive;
extern crate num;
extern crate serde_json;

mod gb_opcodes;
mod tracelog;

use tracelog::TraceLog;

fn main() {
    println!("Hi, I'm TraceBoy!");
}
