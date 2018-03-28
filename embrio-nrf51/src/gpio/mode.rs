use nrf51::gpio::pin_cnf;

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
pub struct Floating {
    _reserved: (),
}

#[derive(Debug, Copy, Clone)]
pub struct PullUp {
    _reserved: (),
}

#[derive(Debug, Copy, Clone)]
pub struct PullDown {
    _reserved: (),
}

#[derive(Debug, Copy, Clone)]
pub struct PushPull {
    _reserved: (),
}

#[derive(Debug, Copy, Clone)]
pub struct OpenDrain {
    _reserved: (),
}

#[derive(Debug, Copy, Clone)]
pub struct Disabled {
    _reserved: (),
}

#[derive(Debug, Copy, Clone)]
pub struct Input<Mode: InputMode> {
    mode: Mode,
}

#[derive(Debug, Copy, Clone)]
pub struct Output<Mode: OutputMode> {
    mode: Mode,
}

impl InputMode for Floating {
    fn apply<'a>(w: &'a mut pin_cnf::W) -> Self {
        w.pull().disabled();
        Floating { _reserved: () }
    }
}

impl InputMode for PullUp {
    fn apply<'a>(w: &'a mut pin_cnf::W) -> Self {
        w.pull().pullup();
        PullUp { _reserved: () }
    }
}

impl InputMode for PullDown {
    fn apply<'a>(w: &'a mut pin_cnf::W) -> Self {
        w.pull().pulldown();
        PullDown { _reserved: () }
    }
}

impl OutputMode for PushPull {
    fn apply<'a>(w: &'a mut pin_cnf::W) -> Self {
        w.drive().s0s1();
        PushPull { _reserved: () }
    }
}

impl OutputMode for OpenDrain {
    fn apply<'a>(w: &'a mut pin_cnf::W) -> Self {
        w.drive().s0d1();
        OpenDrain { _reserved: () }
    }
}

impl PinMode for Disabled {
    fn apply<'a>(w: &'a mut pin_cnf::W) -> Self {
        w.dir().input().input().disconnect();
        Disabled { _reserved: () }
    }
}

impl<Mode: InputMode> PinMode for Input<Mode> {
    fn apply<'a>(w: &'a mut pin_cnf::W) -> Self {
        w.dir().input().input().connect();
        Input {
            mode: Mode::apply(w),
        }
    }
}

impl<Mode: OutputMode> PinMode for Output<Mode> {
    fn apply<'a>(w: &'a mut pin_cnf::W) -> Self {
        w.dir().output();
        Output {
            mode: Mode::apply(w),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use core::mem;

    // TODO: Static assert with const size_of fn?
    #[test]
    fn zst() {
        assert!(mem::size_of::<Input<Floating>>() == 0);
        assert!(mem::size_of::<Input<PullUp>>() == 0);
        assert!(mem::size_of::<Input<PullDown>>() == 0);
        assert!(mem::size_of::<Output<PushPull>>() == 0);
        assert!(mem::size_of::<Output<OpenDrain>>() == 0);
        assert!(mem::size_of::<Disabled>() == 0);
    }
}
