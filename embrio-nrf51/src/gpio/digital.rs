use core::fmt;

use nrf51::GPIO;
use nrf51::gpio::pin_cnf;

use embrio_core;

pub trait InputMode: Sized {
    fn apply(w: &mut pin_cnf::W) -> Self;
}

pub trait OutputMode: Sized {
    fn apply(w: &mut pin_cnf::W) -> Self;
}

pub trait PinMode: Sized {
    fn apply(w: &mut pin_cnf::W) -> Self;
}

#[derive(Debug, Copy, Clone)]
pub struct Floating { _reserved: () }

impl InputMode for Floating {
    fn apply<'a>(w: &'a mut pin_cnf::W) -> Self {
        w.pull().disabled();
        Floating { _reserved: () }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct PullUp { _reserved: () }

impl InputMode for PullUp {
    fn apply<'a>(w: &'a mut pin_cnf::W) -> Self {
        w.pull().pullup();
        PullUp { _reserved: () }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct PullDown { _reserved: () }

impl InputMode for PullDown {
    fn apply<'a>(w: &'a mut pin_cnf::W) -> Self {
        w.pull().pulldown();
        PullDown { _reserved: () }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct PushPull { _reserved: () }

impl OutputMode for PushPull {
    fn apply<'a>(w: &'a mut pin_cnf::W) -> Self {
        w.drive().s0s1();
        PushPull { _reserved: () }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct OpenDrain { _reserved: () }

impl OutputMode for OpenDrain {
    fn apply<'a>(w: &'a mut pin_cnf::W) -> Self {
        w.drive().s0d1();
        OpenDrain { _reserved: () }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Disabled { _reserved: () }

impl PinMode for Disabled {
    fn apply<'a>(w: &'a mut pin_cnf::W) -> Self {
        w.dir().input().input().disconnect();
        Disabled { _reserved: () }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Input<Mode: InputMode> { mode: Mode }

impl<Mode: InputMode> PinMode for Input<Mode> {
    fn apply<'a>(w: &'a mut pin_cnf::W) -> Self {
        w.dir().input().input().connect();
        Input { mode: Mode::apply(w) }
    }
}

#[derive(Debug, Copy, Clone)]
pub struct Output<Mode: OutputMode> { mode: Mode }

impl<Mode: OutputMode> PinMode for Output<Mode> {
    fn apply<'a>(w: &'a mut pin_cnf::W) -> Self {
        w.dir().output();
        Output { mode: Mode::apply(w) }
    }
}

pub struct Pin<'a, Mode> {
    gpio: &'a GPIO,
    // TODO: move pin number to const generic once available
    pin: usize,
    mode: Mode,
}

impl<'a> Pin<'a, Disabled> {
    pub(crate) unsafe fn new(gpio: &'a GPIO, pin: usize) -> Self {
        Pin { gpio, pin, mode: Disabled { _reserved: () } }.set_mode()
    }
}

impl<'a, Mode: PinMode> Pin<'a, Mode> {
    fn set_mode<NewMode: PinMode>(self) -> Pin<'a, NewMode> {
        let mut new_mode = None;
        self.gpio.pin_cnf[self.pin].write(|w| {
            new_mode = Some(NewMode::apply(w));
            w
        });
        Pin {
            gpio: self.gpio,
            pin: self.pin,
            mode: new_mode.expect("write is guaranteed to set this"),
        }
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
