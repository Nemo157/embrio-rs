#![feature(never_type)]

fn main() -> Result<(), hello::Error> {
    let native = embrio_native::init();
    hello::main(native.stdin(), native.stdout())?;
    Ok(())
}
