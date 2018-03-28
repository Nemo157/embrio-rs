use nrf51::GPIO;

use embrio;

use zst_ref::ZstRef;

use super::mode::{Disabled, Floating, Input, OpenDrain, Output, OutputMode,
                  PinMode, PullDown, PullUp, PushPull, Unconfigured};

#[derive(Debug)]
pub struct Pin<'a, Mode> {
    gpio: ZstRef<'a, GPIO>,
    // TODO: move pin number to const generic once available
    pin: usize,
    mode: Mode,
}

trait Reconfigure<'a, Mode> {
    fn reconfigure(self) -> Pin<'a, Mode>;
}

impl<'a> Pin<'a, Unconfigured> {
    #[inline(always)]
    pub(crate) fn new(gpio: &'a GPIO, pin: usize) -> Self {
        Pin {
            gpio: ZstRef::new(gpio),
            pin,
            mode: Unconfigured::new(),
        }
    }
}

impl<'a, Mode, NewMode: PinMode> Reconfigure<'a, NewMode> for Pin<'a, Mode> {
    default fn reconfigure(self) -> Pin<'a, NewMode> {
        let Pin { gpio, pin, .. } = self;
        let mut mode = None;
        gpio.pin_cnf[pin].write(|w| {
            mode = Some(NewMode::apply(w));
            w
        });
        let mode = mode.expect("write is guaranteed to set this");
        Pin {
            gpio,
            pin,
            mode,
        }
    }
}

impl<'a, Mode> Reconfigure<'a, Unconfigured> for Pin<'a, Mode> {
    #[inline(always)]
    fn reconfigure(self) -> Pin<'a, Unconfigured> {
        Pin {
            gpio: self.gpio,
            pin: self.pin,
            mode: Unconfigured::new(),
        }
    }
}

impl<'a, Mode> Reconfigure<'a, Input<Unconfigured>> for Pin<'a, Mode> {
    #[inline(always)]
    fn reconfigure(self) -> Pin<'a, Input<Unconfigured>> {
        Pin {
            gpio: self.gpio,
            pin: self.pin,
            mode: Input::new(),
        }
    }
}

impl<'a, Mode> Reconfigure<'a, Output<Unconfigured>> for Pin<'a, Mode> {
    #[inline(always)]
    fn reconfigure(self) -> Pin<'a, Output<Unconfigured>> {
        Pin {
            gpio: self.gpio,
            pin: self.pin,
            mode: Output::new(),
        }
    }
}

impl<'a, Mode> Pin<'a, Mode> {
    #[inline(always)]
    pub(crate) fn get_id(&self) -> usize {
        self.pin
    }

    #[inline(always)]
    pub fn disable(self) -> Pin<'a, Disabled> {
        self.reconfigure()
    }

    #[inline(always)]
    pub fn output(self) -> Pin<'a, Output<Unconfigured>> {
        self.reconfigure()
    }

    #[inline(always)]
    pub fn input(self) -> Pin<'a, Input<Unconfigured>> {
        self.reconfigure()
    }
}

impl<'a, Mode> Pin<'a, Input<Mode>> {
    #[inline(always)]
    pub fn floating(self) -> Pin<'a, Input<Floating>> {
        self.reconfigure()
    }

    #[inline(always)]
    pub fn pull_up(self) -> Pin<'a, Input<PullUp>> {
        self.reconfigure()
    }

    #[inline(always)]
    pub fn pull_down(self) -> Pin<'a, Input<PullDown>> {
        self.reconfigure()
    }
}

impl<'a, Mode> Pin<'a, Output<Mode>> {
    #[inline(always)]
    pub fn open_drain(self) -> Pin<'a, Output<OpenDrain>> {
        self.reconfigure()
    }

    #[inline(always)]
    pub fn push_pull(self) -> Pin<'a, Output<PushPull>> {
        self.reconfigure()
    }
}

impl<'a, Mode: OutputMode> embrio::gpio::Output for Pin<'a, Output<Mode>> {
    fn state(&self) -> bool {
        (self.gpio.out.read().bits() & (1 << self.pin)) == (1 << self.pin)
    }

    fn set_state(&self, state: bool) {
        if state {
            self.gpio
                .outset
                .write(|w| unsafe { w.bits(1 << self.pin) });
        } else {
            self.gpio
                .outclr
                .write(|w| unsafe { w.bits(1 << self.pin) });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use core::mem;

    // TODO: Static assert with const size_of fn?
    // TODO: Will be zst once pin number is moved to const generic
    #[test]
    fn almost_zst() {
        assert!(
            mem::size_of::<Pin<Input<Floating>>>() == mem::size_of::<usize>()
        );
    }
}
