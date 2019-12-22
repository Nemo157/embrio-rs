use std::env;

fn main() {
    let target = env::var("TARGET").unwrap();

    match target.split('-').next() {
        Some("thumbv6m") => {
            println!("cargo:rustc-cfg=armv6m");
        }
        Some("thumbv7m") | Some("thumbv7em") => {
            println!("cargo:rustc-cfg=armv7m");
        }
        _ => {}
    }
}
