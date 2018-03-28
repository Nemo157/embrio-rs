pub mod mode;
mod pin;

use nrf51::GPIO;

pub use self::pin::Pin;

macro_rules! pin {
    ($s:expr) => {
        Pin<'a, mode::Unconfigured>
    }
}

macro_rules! pins {
    ($($i:expr),*) => {
        #[derive(Debug)]
        pub struct Pins<'a>($(pub pin!($i)),*);

        impl<'a> Pins<'a> {
            pub fn new(gpio: &'a mut GPIO) -> Self {
                Pins($(Pin::new(&*gpio, $i)),*)
            }
        }
    }
}

pins! {
     0,  1,  2,  3,  4,  5,  6,  7,
     8,  9, 10, 11, 12, 13, 14, 15,
    16, 17, 18, 19, 20, 21, 22, 23,
    24, 25, 26, 27, 28, 29, 30, 31
}
