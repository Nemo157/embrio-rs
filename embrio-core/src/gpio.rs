pub trait Output {
    fn state(&self) -> bool;

    fn set_state(&self, state: bool);

    fn is_high(&self) -> bool {
        self.state()
    }

    fn is_low(&self) -> bool {
        !self.state()
    }

    fn set_high(&self) {
        self.set_state(true);
    }

    fn set_low(&self) {
        self.set_state(false);
    }

    fn toggle(&self) {
        self.set_state(!self.state());
    }
}
