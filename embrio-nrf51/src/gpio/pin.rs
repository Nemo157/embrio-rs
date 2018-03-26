use core::fmt;

use nrf51::GPIO;

use embrio_core;

use super::mode::{
    InputMode, OutputMode, PinMode,
    Floating, PullUp, PullDown, PushPull, OpenDrain,
    Disabled, Input, Output,
};

pub struct Pin<'a, Mode> {
    gpio: &'a GPIO,
    // TODO: move pin number to const generic once available
    pin: usize,
    mode: Mode,
}

fn set_mode<Mode: PinMode>(gpio: &'a GPIO, pin: usize) -> Pin<'a, Mode> {
    let mut mode = None;
    gpio.pin_cnf[pin].write(|w| {
        mode = Some(Mode::apply(w));
        w
    });
    let mode = mode.expect("write is guaranteed to set this");
    Pin { gpio, pin, mode }
}

impl<'a> Pin<'a, Disabled> {
    pub(crate) fn new(gpio: &'a GPIO, pin: usize) -> Self {
        set_mode(gpio, pin)
    }
}

impl<'a, Mode: PinMode> Pin<'a, Mode> {
    fn set_mode<NewMode: PinMode>(self) -> Pin<'a, NewMode> {
        set_mode(self.gpio, self.pin)
    }

    pub fn output(self) -> Pin<'a, Output<PushPull>> {
        self.set_mode()
    }

    pub fn input(self) -> Pin<'a, Input<Floating>> {
        self.set_mode()
    }
}

impl<'a, Mode: InputMode> Pin<'a, Input<Mode>> {
    pub fn floating(self) -> Pin<'a, Input<Floating>> {
        self.set_mode()
    }

    pub fn pull_up(self) -> Pin<'a, Input<PullUp>> {
        self.set_mode()
    }

    pub fn pull_down(self) -> Pin<'a, Input<PullDown>> {
        self.set_mode()
    }
}

impl<'a, Mode: OutputMode> Pin<'a, Output<Mode>> {
    pub fn open_drain(self) -> Pin<'a, Output<OpenDrain>> {
        self.set_mode()
    }

    pub fn push_pull(self) -> Pin<'a, Output<PushPull>> {
        self.set_mode()
    }
}

impl<'a, Mode: OutputMode> embrio_core::gpio::Output for Pin<'a, Output<Mode>> {
    fn state(&self) -> bool {
        (self.gpio.out.read().bits() & (1 << self.pin)) == (1 << self.pin)
    }

    fn set_state(&self, state: bool) {
        if state {
            self.gpio.outset.write(|w| unsafe { w.bits(1 << self.pin) });
        } else {
            self.gpio.outclr.write(|w| unsafe { w.bits(1 << self.pin) });
        }
    }
}

impl<'a, Mode> fmt::Debug for Pin<'a, Mode> where Mode: fmt::Debug {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        f.debug_struct("Pin")
            .field("gpio", &"GPIO")
            .field("pin", &self.pin)
            .field("mode", &self.mode)
            .finish()
    }
}
