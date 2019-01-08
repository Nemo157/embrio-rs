#![feature(arbitrary_self_types, futures_api)]

use embrio_core::io::{Read, Write};

mod io;

pub struct EmbrioNative(());

impl EmbrioNative {
    pub fn stdin(&self) -> impl Read {
        self::io::Std(std::io::stdin())
    }

    pub fn stdout(&self) -> impl Write {
        self::io::Std(std::io::stdout())
    }
}

pub fn init() -> EmbrioNative {
    EmbrioNative(())
}
