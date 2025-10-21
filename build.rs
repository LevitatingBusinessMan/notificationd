extern crate varlink_generator;

fn main() {
    varlink_generator::cargo_build("src/levitating.notificationd.varlink");
    // eprintln!("{:?}", std::env::var_os("OUT_DIR").unwrap());
    // std::process::exit(1);
}
