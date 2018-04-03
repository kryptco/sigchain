extern crate cheddar;

fn main() {
    cheddar::Builder::c99().expect("could not read cargo manifest")
        .name("sigchain.h")
        .insert_code("// This header is automatically generated - DO NOT CHANGE IT MANUALLY!")
        .output_directory("../target/include")
        .module("c_api").expect("malformed module path")
        .run_build();
}
