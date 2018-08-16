fn main() -> Result<!, echo::Error> {
    let native = embrio_native::init();
    echo::main(native.stdin(), native.stdout())?;
}
